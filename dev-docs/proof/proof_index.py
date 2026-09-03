#!/usr/bin/env python3
"""項目の依存グラフを作り、動いた項目を引いている項目を挙げる。

使い方:

    python3 dev-docs/proof/proof_index.py <証明のディレクトリ>            # 検査する
    python3 dev-docs/proof/proof_index.py <証明のディレクトリ> --frontier  # 読み直す要のある項目
    python3 dev-docs/proof/proof_index.py <証明のディレクトリ> --assign    # 同一性を振る
    python3 dev-docs/proof/proof_index.py <証明のディレクトリ> --accept    # 読み直した印を進める

**項目**とは、定義 (`D*`)・仮定 (`A*`)・主張 (`P*` と各証明の `L*`) である。3 つは
「証明を持つか、持たないなら誰が果たすか」で分かれる -- 定義と仮定は証明を持たず、主張は持つ。
**補題と命題の区別は無い。** どちらも主張であり、置き場所が違うだけだった。

**同一性は 7 桁の 16 進のランダムで、項目の見出しに `<!--#a3f9c21-->` として置く。**
番号 (`D12`・`P28`・`L14`) は表示であって同一性ではない -- 番号を同一性にすると番号を振り直せなく
なり、後から項目を挟むたびに枝番 (`P7a`・`A26a`・`D11a`) が増える。実測で、枝番の項目が
「A1 から A26」という数え上げから落ちて証明が 2 段ぶん閉じなくなった。

**指紋は言明の指紋である。証明の指紋ではない。** 証明が変わっても、その主張を引く側は影響を受けない。
**変わったときに引く側が読み直す要があるのは、言明が変わったときだけである。**

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
FRAME_ITEM = re.compile(r"^(?:- )?\*\*(T|[DAP]\d+[a-z]*)(?:\s*\(|\*\*\s*\()")
CLAIM_HEAD = re.compile(r"^#+\s+(?:[\d.]+[a-z]?\s+)?`?(L\d+[a-z]*)`?\s*\(")
THEOREM = re.compile(r"^#+\s+(T)\b")
CITATION = re.compile(r"\b([DAP]\d+[a-z]*|L\d+[a-z]*)\b")


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
    span = range(len(lines))
    if frame:
        heads_of_sections = [i for i, l in enumerate(lines) if re.match(r"^## \d+\.", l)]
        starts = [i for i in heads_of_sections if re.match(r"^## 3\.", lines[i])]
        ends = [i for i in heads_of_sections if re.match(r"^## 6\.", lines[i])]
        if starts and ends:
            span = range(starts[0], ends[0])
    heads = []
    for index in span:
        line = lines[index]
        for pattern in patterns:
            match = pattern.match(line)
            if match:
                heads.append((index, match.group(1)))
                break
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
            for name in set(CITATION.findall(text)):
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
        for citing, cited in sorted(edges):
            read = was.get((citing, cited), items[cited]["digest"])
            out.write(f"{citing}\t{cited}\t{read}\t"
                      f"{os.path.basename(items[citing]['file'])}:{items[citing]['name']}\t"
                      f"{os.path.basename(items[cited]['file'])}:{items[cited]['name']}\n")


def main(arguments):
    roots = [a for a in arguments if not a.startswith("--")] or ["dev-docs/proof/rc_ir/borrow-cancel"]
    for directory in roots:
        if "--assign" in arguments:
            print(f"{directory}: 同一性を {assign(directory)} 個振った")
        items, edges = build(directory)
        was = read_ledger(directory)
        moved = [(citing, cited) for citing, cited in sorted(edges)
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
