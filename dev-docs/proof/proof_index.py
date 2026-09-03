#!/usr/bin/env python3
"""項目の依存グラフを作り、動いた項目を引いている項目を挙げる。

使い方:

    python3 dev-docs/proof/proof_index.py <証明のディレクトリ>            # 検査する
    python3 dev-docs/proof/proof_index.py <証明のディレクトリ> --frontier  # 読み直す要のある項目
    python3 dev-docs/proof/proof_index.py <証明のディレクトリ> --assign    # 同一性を振る
    python3 dev-docs/proof/proof_index.py <証明のディレクトリ> --accept    # 読み直した印を進める
    python3 dev-docs/proof/proof_index.py <証明のディレクトリ> --show a3f9c21  # id の中身を出す

**項目**とは、定義 (`D*`)・仮定 (`A*`)・主張 (`P*` と各証明の `L*`) である。3 つは
「証明を持つか、持たないなら誰が果たすか」で分かれる -- 定義と仮定は証明を持たず、主張は持つ。
**補題と命題の区別は無い。** どちらも主張であり、置き場所が違うだけだった。

**同一性は 7 桁の 16 進のランダムで、項目の見出しに `<!--#a3f9c21-->` として置く。**
番号 (`D12`・`P28`・`L14`) は表示であって同一性ではない -- 番号を同一性にすると番号を振り直せなく
なり、後から項目を挟むたびに枝番 (`P7a`・`A26a`・`D11a`) が増える。実測で、枝番の項目が
「A1 から A26」という数え上げから落ちて証明が 2 段ぶん閉じなくなった。

**指紋は言明の指紋である。証明の指紋ではない。** 証明が変わっても、その主張を引く側は影響を受けない。
**変わったときに引く側が読み直す要があるのは、言明が変わったときだけである。**

**番号はソースに置かない。読む用の版を生成するときに入れる。** 番号がソースに在ると、人はそれを
散文へ写す -- `BY A19 (ii-c)`、「A19 の (ii-c)」、「第 4 節の仮定は A1 から A26」。**写された番号は
道具の外に出るので、項目が増えたときに黙って古くなる。** 実測で、その形の欠陥がこの証明に 4 つあった
(定義の数え上げ・仮定の前書き・仮定の集合・「各点についての言明」の一覧)。
**ソースに番号が無ければ、写せる番号が存在しない。**

**番号を生成し直す道具は要らない。** この証明を人が通読することはもう無く、読むのはエージェントで
ある。**エージェントに要るのは id から中身へ速く引けることだけで、それは `items.tsv` が与える。**
**`a3f9c21` は何も示唆しないので、引きに行くほかに使い道が無い** -- `A19 (ii-c)` が
「仮定 19 の第 2 群の第 3 項」と読めて中身を再構成したくなるのと違う。実測で、番号を持つ項目の中身を
記憶から書いた捏造引用が 8 か所あった。

**引用は、それが現れた項目に帰属させる。** 「このファイルが D24 を引いている」では、読み直す者が
10 万トークンを読むことになる。**「P14 の言明と `p20` の `L27` が D24 を引いている」なら、
その 2 つだけを読めばよい。**
"""
import hashlib
import os
import re
import secrets
import sys

IDENTITY = re.compile(r"<!--#([0-9a-f]{7})-->")
# 枠の項目は太字の見出し (`**D12 (…)**`、`- **P28** (…)`)、証明の主張は `#` の見出しである。
# 項目の見出しは題を括弧で持つ (`**D1 (プログラム)**`、`- **P28** (参照の持ち手は…)`)。
# 散文の太字 (`**D24 の網羅の節**`) と分けるのはこの括弧である。
# 見出しは 2 つの形しかない -- `**D1 (プログラム)**` と `- **P28** (参照の持ち手は…)`。
# **太字が括弧の直後で閉じることを条件にする。** そうしないと散文の
# `**P5 (a) はこの数え上げに載っている。**` を見出しと読む (実測で 3 度やった)。
FRAME_ITEM = re.compile(r"^\*\*(T|[DAP]\d+[a-z]*)\s*\([^)]*\)\*\*"
                        r"|^- \*\*(T|[DAP]\d+[a-z]*)\*\*\s*\(")
CLAIM_HEAD = re.compile(r"^#+\s+(?:[\d.]+[a-z]?\s+)?`?(L\d+[a-z]*)`?\s*\(")
# **項も項目である。** `A19 (ii-c)` は独立した主張で、独立に引かれる -- 実測で 16 の項目が項の形で
# 引かれ、引用は 598 回あった。項目を丸ごと 1 つと数えると、**`(i)` を 1 語直しただけで `(ii-b)` しか
# 引いていない 50 か所が「読み直せ」に出る。** 項を項目にすると波及がその項だけに閉じる。
CLAUSE = re.compile(r"^(?:- )?\*\*\(([a-z]|i+|ii-[a-c]|iii|S-[a-c]|[EFXRK]\d?)\)")
THEOREM = re.compile(r"^#+\s+(T)\b")
CITATION = re.compile(r"\b([DAP]\d+[a-z]*|L\d+[a-z]*)\b")
# **引用も項の粒度で読む。** `A19 (ii-c)` は `A19` ではない。
CITED_CLAUSE = re.compile(r"\b([DAP]\d+[a-z]*)\s+\(([a-z]|i+|ii-[a-c]|iii|S-[a-c]|[EFXRK]\d?)\)")


def digest(text):
    """言明の指紋。同一性の印は除く。

    印はこの道具自身の帳簿なので、指紋に入れると**印を振った瞬間に全項目が「動いた」になる。**
    実測で、印を振った直後の一覧が 102 項目すなわち枠の全部を挙げた。
    `proof_links.py` が `// PROOF:` のコメントを除くのと同じ理由である。"""
    return hashlib.sha256(
        re.sub(r"\s+", "", IDENTITY.sub("", text)).encode("utf-8")).hexdigest()[:7]


def statement_of(lines, start, end):
    """項目の言明。証明の主張は `**言明**。` の段落、枠の項目は見出しから次の項目までである。"""
    for index in range(start, min(end, start + 12)):
        if lines[index].startswith("**言明**"):
            body, cursor = [], index
            while cursor < end and lines[cursor].strip():
                body.append(lines[cursor])
                cursor += 1
            return "\n".join(body)
    return "\n".join(lines[start:end])


def items_in(path):
    """1 つの文書の項目。(番号, 同一性, 行, 言明, 範囲) を順に返す。

    **定義・仮定・命題は枠にしか置かれない。** 証明の中に同じ形で現れるのは、その項目の再掲であって
    定義ではない -- 手で写した本文であり、この仕組みが置き換える当のものである。"""
    lines = open(path, encoding="utf-8").read().split("\n")
    frame = os.path.basename(path) == "README.md"
    patterns = (FRAME_ITEM, THEOREM) if frame else (CLAIM_HEAD,)
    # **枠の項目は定義・仮定・命題の節にしかない。** その外の散文にも `**A19 (ii) が…**` のように
    # 見出しと同じ形の太字が現れるので、節で囲まないと散文が項目になり、**その手前の項目が
    # 残りの節を丸ごと抱え込む** (実測で 1 つの項目が 321 行と辺 68 本を持った)。
    clauses = frame
    span = range(len(lines))
    if frame:
        heads_of_sections = [i for i, l in enumerate(lines) if re.match(r"^## \d+\.", l)]
        starts = [i for i in heads_of_sections if re.match(r"^## 3\.", lines[i])]
        ends = [i for i in heads_of_sections if re.match(r"^## 6\.", lines[i])]
        if starts and ends:
            span = range(starts[0], ends[0])
    heads, owner = [], None
    for index in span:
        line = lines[index]
        for pattern in patterns:
            match = pattern.match(line)
            if match:
                owner = next(g for g in match.groups() if g)
                heads.append((index, owner))
                break
        else:
            clause = CLAUSE.match(line) if clauses and owner else None
            if clause:
                heads.append((index, f"{owner} ({clause.group(1)})"))
    found = []
    for order, (index, name) in enumerate(heads):
        end = heads[order + 1][0] if order + 1 < len(heads) else len(lines)
        identity = IDENTITY.search(lines[index])
        found.append({
            "name": name,
            "identity": identity.group(1) if identity else None,
            "line": index + 1,
            "statement": statement_of(lines, index, end),
            "span": (index, end),
            "file": path,
        })
    return found, lines


def documents(directory):
    for name in sorted(os.listdir(directory)):
        if name.endswith(".md") and name not in ("citations.tsv",):
            yield os.path.join(directory, name)


def assign(directory):
    """同一性を持たない項目に振る。既に持つものは触らない。

    **番号を振り直す要は無い。** 同一性が大域の名前であり、番号 (`L14`) はその文書の中の表示である。
    `L1` が 7 つの文書に在っても、同一性は 7 つとも別である。"""
    taken = set()
    for path in documents(directory):
        taken.update(IDENTITY.findall(open(path, encoding="utf-8").read()))
    added = 0
    for path in documents(directory):
        found, lines = items_in(path)
        for item in found:
            if item["identity"]:
                continue
            while True:
                fresh = secrets.token_hex(4)[:7]
                if fresh not in taken:
                    break
            taken.add(fresh)
            index = item["line"] - 1
            lines[index] = lines[index].rstrip() + f" <!--#{fresh}-->"
            added += 1
        if added:
            open(path, "w", encoding="utf-8").write("\n".join(lines))
    return added


CROSS_FILE = re.compile(r"(p\d{2})[a-z0-9-]*(?:\.md)?\s*の\s*`?(L\d+[a-z]*)`?")


def build(directory):
    """全項目と、項目から項目への引用の辺。"""
    items, by_file = {}, {}
    for path in documents(directory):
        found, lines = items_in(path)
        by_file[os.path.basename(path).split("-")[0]] = {i["name"]: i for i in found}
        for item in found:
            item["digest"] = digest(item["statement"])
            if item["identity"]:
                items[item["identity"]] = item
    frame = by_file.get("README.md", by_file.get("README", {}))
    edges = set()
    for path in documents(directory):
        found, lines = items_in(path)
        key = os.path.basename(path).split("-")[0]
        for item in found:
            text = "\n".join(lines[item["span"][0] + 1:item["span"][1]])
            targets = set()
            for match in CROSS_FILE.finditer(text):
                owner = by_file.get(match.group(1), {})
                if match.group(2) in owner:
                    targets.add(owner[match.group(2)]["identity"])
            # 項つきの引用を先に取り、その親は取らない -- `A19 (ii-c)` を引く段は `A19` の
            # ほかの項が動いても読み直す要が無い。
            clauses = set()
            for match in CITED_CLAUSE.finditer(text):
                full = f"{match.group(1)} ({match.group(2)})"
                if full in frame:
                    targets.add(frame[full]["identity"])
                    clauses.add(match.group(1))
            for name in set(CITATION.findall(text)) - clauses:
                home = frame if name[0] in "DAP" else by_file.get(key, {})
                if name in home:
                    targets.add(home[name]["identity"])
            targets.discard(item["identity"])
            edges.update((item["identity"], target) for target in targets)
    return items, edges


def ledger_path(directory):
    return os.path.join(directory, "items.tsv")


def read_ledger(directory):
    path = ledger_path(directory)
    if not os.path.exists(path):
        return {}
    rows = {}
    for line in open(path, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        citing, cited, read = line.rstrip("\n").split("\t")[:3]
        rows[(citing, cited)] = read
    return rows


def write_ledger(directory, items, edges, was):
    path = ledger_path(directory)
    with open(path, "w", encoding="utf-8") as out:
        out.write("# generated by dev-docs/proof/proof_index.py -- do not edit\n")
        out.write("citing\tcited\tread\tciting_name\tcited_name\n")
        for citing, cited in sorted(e for e in edges if all(e)):
            read = was.get((citing, cited), items[cited]["digest"])
            out.write(f"{citing}\t{cited}\t{read}\t"
                      f"{os.path.basename(items[citing]['file'])}:{items[citing]['name']}\t"
                      f"{os.path.basename(items[cited]['file'])}:{items[cited]['name']}\n")


def show(directory, wanted):
    """id が指す項目の本文と、それを引いている項目を出す。

    **id から中身へ引く道は道具が与える。** 「台帳を見て `grep` してください」は道具ではない --
    引くたびに人が段取りを組み直すことになり、**引くのが面倒だから引かない**という道が開く。
    記憶から書いた引用がこの証明に 8 か所あった。"""
    items, edges = build(directory)
    if wanted not in items:
        print(f"{wanted}: そのような項目は無い")
        return 1
    item = items[wanted]
    print(f"{item['file']}:{item['line']}  {item['name']}  指紋 {item['digest']}\n")
    print(item["statement"])
    citers = sorted((items[a]["name"], os.path.basename(items[a]["file"]))
                    for a, b in edges if b == wanted)
    print(f"\n-- これを引いている項目 {len(citers)} 個")
    for name, where in citers:
        print(f"   {name} ({where})")
    return 0


def main(arguments):
    if "--show" in arguments:
        at = arguments.index("--show")
        roots = [a for a in arguments if not a.startswith("--")]
        directory = roots[0] if len(roots) > 1 else "dev-docs/proof/rc_ir/borrow-cancel"
        return show(directory, arguments[at + 1])
    roots = [a for a in arguments if not a.startswith("--")] or ["dev-docs/proof/rc_ir/borrow-cancel"]
    for directory in roots:
        if "--assign" in arguments:
            print(f"{directory}: 同一性を {assign(directory)} 個振った")
        items, edges = build(directory)
        was = read_ledger(directory)
        moved = [(citing, cited) for citing, cited in sorted(e for e in edges if all(e))
                 if was.get((citing, cited), items[cited]["digest"]) != items[cited]["digest"]]
        if "--accept" in arguments:
            write_ledger(directory, items, edges, {})
            print(f"{directory}: 読み直した印を {len(moved)} 件進めた")
            continue
        if not os.path.exists(ledger_path(directory)) or "--write" in arguments:
            write_ledger(directory, items, edges, was)
        for citing, cited in moved:
            here, there = items[citing], items[cited]
            print(f"{here['file']}:{here['line']}: {here['name']} が引く "
                  f"{there['name']} ({os.path.basename(there['file'])}) が動いた -- 読み直すこと")
        if "--frontier" in arguments:
            print(f"{directory}: 読み直す要のある項目 {len({c for c, _ in moved})} 個、"
                  f"引用の辺 {len(moved)} 本")
            continue
        print(f"{directory}: 項目 {len(items)} 個、引用の辺 {len(edges)} 本、動いた辺 {len(moved)} 本")
        continue
        kinds = {"定義": 0, "仮定": 0, "命題": 0, "主張 (証明の中)": 0, "定理": 0}
        total, unnamed = 0, 0
        for path in documents(directory):
            found, _ = items_in(path)
            for item in found:
                total += 1
                unnamed += item["identity"] is None
                head = item["name"][0]
                kinds["定義" if head == "D" else "仮定" if head == "A" else
                      "命題" if head == "P" else "定理" if head == "T" else "主張 (証明の中)"] += 1
        print(f"{directory}: 項目 {total} 個、同一性が未設定 {unnamed} 個")
        for kind, count in kinds.items():
            print(f"  {kind}: {count}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
