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
**主張はすべて命題である。** 枠に置くか証明ファイルに置くかの違いしかない。

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
# **見出しは「太字が名前を名乗り、題を括弧で持つ行」である。** 箇条書きの `- ` が付くかどうかと、
# 括弧が太字の内と外のどちらに在るかは、書き手によって違う -- 実測で 3 通りあり、
# `- **D27 (bump の帰属)** --` の形を落として**その定義への引用 115 件が宙に浮いていた。**
# 括弧を要求するのは、散文の太字 (`**D24 の網羅の節**`) と分けるためである。
FRAME_ITEM = re.compile(r"^(?:- )?\*\*(T|[DAP]\d+[a-z]*)\s*"
                        r"(?:\([^)]*\)\*\*|\*\*\s*\()")
# **命題の見出しとは、命題の名前を名乗っている見出しである。** 名前の後に何が続くかは問わない --
# 実測で名乗り方が 6 通りあり、括弧を要求していたために `## 2. L6 -- D9 の…` の形を落として、
# **そのファイルの引用が 1 本もグラフに入っていなかった。**
# **見出しが命題を名乗るのは、名前が題の位置に在り、区切りで終わるときである。** 名前を文中に
# 含むだけの散文の見出し (`### L4 の前件 -- 解析が呼ぶ鍵に限ること`) を命題として登録すると、
# その名前を引く段が散文を指す。実測で、その形で 10 件の引用が命題でなく散文へ向いていた。
CLAIM_HEAD = re.compile(
    r"^#+\s+(?:[\d.]+[a-z]?\s+)?`?(L\d+[a-z]*)`?\s*(?:\(|--|:|、|$)"
    r"|^\*\*`?(L\d+[a-z]*)`?(?:\s*\([^)]*\))?\*\*")
# ファイルの題が名乗る枠の命題。**言明が枠に、証明がこのファイルに在る**形である。
# 局所の命題に属さない引用は、この命題のものとして数える -- 節の前書きも、
# 局所の命題を 1 つも立てずに枠の命題を直接示すファイルも、これで拾う。
FILE_PROVES = re.compile(r"\b(T|[AP]\d+[a-z]*)\b")
# **項も項目である。** `A19 (ii-c)` は独立した主張で、独立に引かれる -- 実測で 16 の項目が項の形で
# 引かれ、引用は 598 回あった。項目を丸ごと 1 つと数えると、**`(i)` を 1 語直しただけで `(ii-b)` しか
# 引いていない 50 か所が「読み直せ」に出る。** 項を項目にすると波及がその項だけに閉じる。
# **項を項目にするのは、節と散文を構文で分けられないので保留する。** 散文の書き出し
# `**(ii-b) は保存では通らない**` は節の見出しと同じ形をしており、推測すると実測で重複した項目が
# 9 個でき、辺 52 本が節でなく散文に着いた。**過大報告は安全側、誤った宛先は危険側である。**
# 分けるには節に印を付ける要があり、それは書式の変更なので別に決める。
CLAUSE = None

THEOREM = re.compile(r"^#+\s+(T)\b")
CITATION = re.compile(r"\b([DAP]\d+[a-z]*|L\d+[a-z]*)\b")
# 変換後の参照。**格納されるのは id だけで、題は描画のときに命題から取る。**
REF = re.compile(r"<ref id=([0-9a-f]{7})/>")
# 項の粒度の引用。いまは項が項目でないので使わない。
CITED_CLAUSE = re.compile(r"\b([DAP]\d+[a-z]*)\s+\(([a-z]|i+|ii-[a-c]|iii|S-[a-c]|[EFXRK]\d?)\)")


def digest(text):
    """言明の指紋。同一性の印は除く。

    印はこの道具自身の帳簿なので、指紋に入れると**印を振った瞬間に全項目が「動いた」になる。**
    実測で、印を振った直後の一覧が 102 項目すなわち枠の全部を挙げた。
    `proof_links.py` が `// PROOF:` のコメントを除くのと同じ理由である。"""
    return hashlib.sha256(
        re.sub(r"\s+", "", IDENTITY.sub("", text)).encode("utf-8")).hexdigest()[:7]


def statement_of(lines, start, end, frame):
    """項目の言明。

    **枠の項目は全体が言明である。証明の項目は言明の段落だけである** -- 証明が変わっても、その主張を
    引く側は影響を受けない。言明は `**言明**` の段落、無ければ見出しの直後の段落である。"""
    if frame:
        return "\n".join(lines[start:end])
    for index in range(start, min(end, start + 12)):
        if lines[index].startswith("**言明**"):
            start = index
            break
    else:
        start += 1
        while start < end and not lines[start].strip():
            start += 1
    # **言明は段が始まるまでである。空行では切らない。** 言明が (i)(ii) や場合分けや式を持つと
    # 段落をまたぐので、空行で切ると先が落ちる -- 実測で 5 つの命題の言明が第 1 段落だけになり、
    # それを引く 13 か所の段が束だけでは検証できなかった。
    body = []
    while start < end:
        line = lines[start]
        if STEP_LINE.match(line) or line.startswith("#"):
            break
        body.append(line)
        start += 1
    while body and not body[-1].strip():
        body.pop()
    return "\n".join(body)


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
    # **印の付いた行は項目である。** 項目の中の 1 つの節を独立に引く段があるので、その節が動いた
    # ときに起きるのはその節を引く段だけでよい -- 実測で、`A19` の 1 つの節を直すと、ほかの節しか
    # 引いていない 50 か所が「読み直せ」に出ていた。
    # **親の範囲は子を含んだままにする。** 親を引く段は全体に依拠しているので、子が動いたら
    # 起きるべきである。過大に起こすのは安全な側で、起こし損ねるのは危険な側である。
    children = []
    if frame:
        parents = {index for index in span
                   if any(pattern.match(lines[index]) for pattern in patterns)}
        children = [(index, None) for index in span
                    if index not in parents and IDENTITY.search(lines[index])]
    heads, owner = [], None
    if not frame:
        # 局所の命題に属さない本文は、このファイルが証明する枠の命題のものである。
        proves = FILE_PROVES.findall(lines[0]) if lines else []
        for name in dict.fromkeys(proves):
            heads.append((0, name))
    for index in span:
        # **印を外してから形を見る。** 印は道具自身が行末に書き足すものなので、
        # 付けた瞬間に「名前で終わる見出し」が名前で終わらなくなる。
        line = IDENTITY.sub("", lines[index]).rstrip()
        for pattern in patterns:
            match = pattern.match(line)
            if match:
                owner = next(g for g in match.groups() if g)
                heads.append((index, owner))
                break
        else:
            if CLAUSE is not None and clauses and owner:
                clause = CLAUSE.match(line)
                if clause:
                    heads.append((index, f"{owner} ({next(g for g in clause.groups() if g)})"))
    # **項目は自分の節で終わる。** 次の項目の見出しまでで切ると、節の最後の項目が次の節の前書きを
    # 抱え込み、**その前書きを直しただけで、その項目を引く全員が読み直しに挙がる** (実測で 79 件)。
    section_heads = [i for i, l in enumerate(lines) if re.match(r"^#+ ", l)]
    parent_starts = [index for index, _ in heads]
    found = []
    for order, (index, name) in enumerate(sorted(heads + children)):
        later = [start for start in parent_starts + [i for i, _ in children] if start > index]
        if (index, name) in children:
            end = min(later) if later else len(lines)
        else:
            after = [start for start in parent_starts if start > index]
            end = min(after) if after else len(lines)
        # **節の境界で切るのは枠だけである。** 枠の項目は節の中の 1 段落だが、証明の命題は
        # その証明の全体であり、節をいくつ跨いでもよい。
        if frame:
            end = min([end] + [i for i in section_heads if i > index])
        identity = IDENTITY.search(lines[index])
        found.append({
            "name": name,
            "heading": lines[index],
            "identity": identity.group(1) if identity else None,
            "line": index + 1,
            "statement": statement_of(lines, index, end, frame),
            "span": (index, end),
            "file": path,
        })
    return found, lines


# **証明でない文書は、自分でそう名乗る。** 名前で見分けると、名前の付け方が変わった日に黙って
# 証明が 1 本落ちる。
NOT_A_PROOF = "<!--not-a-proof-->"


def documents(directory):
    for name in sorted(os.listdir(directory)):
        if not name.endswith(".md"):
            continue
        path = os.path.join(directory, name)
        with open(path, encoding="utf-8") as handle:
            if NOT_A_PROOF in handle.read(400):
                continue
        yield path


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
            # **ファイルの題が名乗る命題には振らない。** 言明が枠に、証明がこのファイルに在るだけで、
            # 同一性は枠の項目のものである。振ると 1 つの命題が 2 つの同一性を持つ。
            if item["line"] == 1:
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


def mark_claims(directory):
    """定義・仮定の中の主張の行に印を振る。

    **振り過ぎても安全である** -- 親の範囲は子を含んだままなので、親を引く段は今までどおり
    子の変更でも起きる。主張でない行に振ったときに起きるのは、誰も引かない項目が 1 つ増えることだけ。
    **振らないでおくと危険である** -- 1 つの節を直すたびに、ほかの節しか引いていない段まで起きる。"""
    taken = set()
    for path in documents(directory):
        taken.update(IDENTITY.findall(open(path, encoding="utf-8").read()))
    added = 0
    for path, line, _, _ in reversed(unproved_claims(directory)):
        lines = open(path, encoding="utf-8").read().split("\n")
        while True:
            fresh = secrets.token_hex(4)[:7]
            if fresh not in taken:
                break
        taken.add(fresh)
        lines[line - 1] = lines[line - 1].rstrip() + f" <!--#{fresh}-->"
        open(path, "w", encoding="utf-8").write("\n".join(lines))
        added += 1
    return added


CROSS_FILE = re.compile(r"(p\d{2})[a-z0-9-]*(?:\.md)?\s*の\s*`?(L\d+[a-z]*)`?")


def build(directory):
    """全項目と、項目から項目への引用の辺。"""
    items, by_file, parsed = {}, {}, {}
    for path in documents(directory):
        parsed[path] = items_in(path)
        found, _ = parsed[path]
        by_file[os.path.basename(path).split("-")[0]] = {i["name"]: i for i in found}
    frame = by_file.get("README.md", by_file.get("README", {}))
    # **ファイルの題が名乗る命題は、枠に在るその命題そのものである。** 言明が枠に、証明がこの
    # ファイルに在るだけなので、同一性は 1 つである。
    for found, _ in parsed.values():
        for item in found:
            if item["identity"] is None and item["name"] in frame:
                item["identity"] = frame[item["name"]]["identity"]
            item["digest"] = digest(item["statement"])
            if item["identity"] and item["identity"] not in items:
                items[item["identity"]] = item
    edges = set()
    for path, (found, lines) in parsed.items():
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
            targets.update(REF.findall(text))
            for name in set(CITATION.findall(text)) - clauses:
                home = frame if name[0] in "DAP" else by_file.get(key, {})
                if name in home:
                    targets.add(home[name]["identity"])
            targets.discard(item["identity"])
            edges.update((item["identity"], target) for target in targets)
    return items, edges


TITLE = re.compile(r"^\*\*(?:T|[DAP]\d+[a-z]*)\s*\((.*?)\)\*\*"
                   r"|^- \*\*(?:T|[DAP]\d+[a-z]*)\*\*\s*\((.*?)\)"
                   r"|^#+\s+(?:[\d.]+[a-z]?\s+)?`?L\d+[a-z]*`?\s*\((.*?)\)")


def title_of(line):
    """項目の見出しが持つ題。定義なら定義される語、仮定と主張なら言明の要約である。

    **これは表示であって同一性ではない。** 項目から取ってくるので、項目の題が変われば表示も変わり、
    引く側は 1 か所も古くならない。**引く側に書いてよいのは id だけである。**"""
    match = TITLE.match(line.strip())
    return next((g for g in match.groups() if g), None) if match else None


def with_annotations(lines, start):
    """項目の定義の行より上に在る doc コメントと属性まで範囲を広げる。

    **証明はそこを引く。** 実測で、`#[derive(Clone)]` と `/// … in no particular order` が
    落ちた束を渡された検証者が、それを確かめるためにリポジトリを開き直した。"""
    at = start
    while at > 0:
        above = lines[at - 1].strip()
        if above.startswith(("///", "//!", "//", "#[")) or above.endswith("]") and above.startswith("#"):
            at -= 1
        else:
            break
    return at


def fix_value_span(lines, symbol):
    """Fix のグローバル値の定義の行の範囲。宣言の行から、次の最上位の宣言の手前まで。"""
    name = re.escape(symbol)
    # 値の宣言・定義 (`f : ...` / `f = ...`)、型の宣言 (`type F a = ...`)、trait の宣言。
    # 名前空間の中では字下げされるので、行頭の空白を許す。
    head = re.compile(rf"^(\s*)(?:{name}\s*[:=]|(?:type|trait)\s+{name}\b)")
    for index, line in enumerate(lines):
        match = head.match(line)
        if not match:
            continue
        indent = len(match.group(1))
        end = index + 1
        while end < len(lines):
            following = lines[end]
            if following.strip() and len(following) - len(following.lstrip()) <= indent \
                    and not head.match(following):
                break
            end += 1
        return index, end
    return None


def cited_code(path, repo):
    """その証明が `CODE` で引く記号の本体。**読む者に開かせるのでなく、束が運ぶ。**

    実測で、1 つのファイルの検証が 208 記号を開くのに tool 呼び出し 130 回を使った。
    引用の一覧は `proof_links` が持っているので、器を 2 つ書かない。"""
    import proof_links
    text = open(path, encoding="utf-8").read()
    out, seen = [], set()
    for source, symbol in proof_links.citations_in(text):
        if (source, symbol) in seen:
            continue
        seen.add((source, symbol))
        full = os.path.join(repo, source)
        if not os.path.exists(full):
            out.append((source, symbol, None))
            continue
        lines = open(full, encoding="utf-8", errors="ignore").read().split("\n")
        # 引く先はソースの項目であり、その形は言語で決まる -- Rust の項目、pest の規則、Fix の値。
        if source.endswith(".pest"):
            span = proof_links.rule_span(lines, symbol)
        elif source.endswith(".fix"):
            span = fix_value_span(lines, symbol)
        else:
            span = proof_links.item_span(lines, symbol)
        if span:
            out.append((source, symbol, "\n".join(lines[with_annotations(lines, span[0]):span[1]])))
        else:
            out.append((source, symbol, None))
    return out


def bundle(directory, path, only=None):
    """1 つのファイル (または 1 つの項目) が引く項目の本文を、読む順に組み立てる。

    **これがエージェントの読むものである。** 引く側には id しか無いので、id の指す本文を束が運ぶ。
    束が本文を運ぶなら、引く場所ごとに本文を再掲する要は無い -- **同じ本文が束と証明の 2 か所に
    在ると、片方だけが古くなる。**"""
    items, edges = build(directory)
    # **そのファイルに在る項目は、そのファイルを読んで決める。** 言明が枠に在る命題は同一性を枠と
    # 共有するので、全体の表を引くと枠のファイルの項目として返り、このファイルの分が 1 つも取れない --
    # 実測で、局所の補題を持たない 3 本の束が空になった。
    frame_names = {item["name"]: item["identity"]
                   for item in items_in(os.path.join(directory, "README.md"))[0] if item["name"]}
    mine = set()
    for item in items_in(path)[0]:
        identity = item["identity"] or frame_names.get(item["name"])
        if identity:
            mine.add(identity)
    here = [items[identity] for identity in mine if identity in items]
    if only:
        here = [i for i in here if only in (i["name"], i["identity"])]
        if not here:
            sys.exit(f"{path} に {only} は無い")
    wanted = {t for identity, t in edges if identity in {i["identity"] for i in here}}
    cited = sorted((items[t] for t in wanted if t in items),
                   key=lambda i: (i["file"], i["line"]))
    out = [f"# {os.path.basename(path)}" + (f" の {only}" if only else "") + " が引く項目",
           "", "道具が組み立てた束である。手で編集しない。", ""]
    for item in cited:
        lines = open(item["file"], encoding="utf-8").read().split("\n")
        title = title_of(lines[item["line"] - 1]) or item["name"]
        out.append(f"## {title} ({item['identity']}) -- {os.path.basename(item['file'])}")
        out.append("")
        out.append(item["statement"])
        out.append("")
    if not only:
        # **どの命題にも属さない枠の本文も運ぶ。** 節の前書きは項目ではないが、証明はそこを引く --
        # 実測で、検証者が「束が本文を持たない」と 3 か所で報告した。
        found, lines = items_in(os.path.join(directory, "README.md"))
        covered = set()
        for item in found:
            covered.update(range(*item["span"]))
        rest = [line for index, line in enumerate(lines) if index not in covered]
        if any(line.strip() for line in rest):
            out.append("# 枠のうち、どの命題にも属さない本文")
            out.append("")
            out.extend(rest)
            out.append("")
        repo = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
        code = cited_code(path, repo)
        out.append("# この証明が `CODE` で引くコード")
        out.append("")
        out.append("**引用の台帳はこれで作る。** 見つからなかった記号は `BAD-CITATION` である。")
        out.append("")
        for source, symbol, text in code:
            out.append(f"## `{source}: {symbol}`")
            out.append("")
            if text is None:
                out.append("**この記号は見つからない。**")
            else:
                out.append("```rust")
                out.append(text)
                out.append("```")
            out.append("")
    return "\n".join(out), len(cited)


def resolution(directory):
    """名前から id への表。名前はファイルの中でしか一意でないので、ファイルごとに持つ。"""
    frame, per_file = {}, {}
    for path in documents(directory):
        key = os.path.basename(path).split("-")[0]
        table = {}
        for item in items_in(path)[0]:
            if item["identity"]:
                table[item["name"]] = item["identity"]
        per_file[key] = table
        if os.path.basename(path) == "README.md":
            frame = table
    return frame, per_file


# **引用と再掲の中は書き替えない。** 枠の文をそのまま写した箇所を書き替えると、写しが枠と
# 一致しなくなり、引用の照合が全部食い違いに出る。**行ごとに見てはならない** -- 実測で、
# 1 行の中で閉じる引用だけを守った版が、行を跨ぐ引用 17 件を壊した。
PROTECTED = re.compile(r"「.*?」|`[^`\n]*`|^>[^\n]*", re.S | re.M)
BY_LINE = re.compile(r"^(\s*)BY\s", re.M)
STEP_LINE = re.compile(r"^\s*(?:\*\*)?`?<\d+>")


def by_spans(text):
    """`BY` の行と、それに続く深く字下げされた行の、文字位置の範囲。"""
    lines = text.split("\n")
    offset, spans, inside, indent, start = 0, [], False, 0, 0
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("BY "):
            if inside:
                spans.append((start, offset))
            inside, indent, start = True, len(line) - len(line.lstrip()), offset
        elif inside and (not stripped or STEP_LINE.match(line)
                         or len(line) - len(line.lstrip()) <= indent):
            spans.append((start, offset))
            inside = False
        offset += len(line) + 1
    if inside:
        spans.append((start, offset))
    return spans


def convert(directory, path):
    """`BY` が挙げる命題・定義・仮定を `<ref id=.../>` にする。

    **書き替えるのは `BY` の行だけで、その中の引用は除く。** `BY` の外の散文で名前を綴りから探すと、
    参照でないものを参照にする -- 実測で、Rust の型変数 `&T` が主定理 `T` の id に潰れた。"""
    frame, per_file = resolution(directory)
    here = per_file.get(os.path.basename(path).split("-")[0], {})
    text = open(path, encoding="utf-8").read()
    done, left = 0, []

    def one(match):
        nonlocal done
        name = match.group(0)
        identity = (frame if name[0] in "DAP" else here).get(name)
        if not identity:
            left.append(name)
            return name
        done += 1
        return f"<ref id={identity}/>"

    def cross(match):
        """`p13 の L14` -- 別のファイルの命題。**組で先に処理する。**

        名前だけを見て変換すると、引く側に同じ名前の命題が在るときそちらへ潰れる。実測で、
        往復の検査が辺 14 本の食い違いとして出した。"""
        nonlocal done
        identity = per_file.get(match.group(1), {}).get(match.group(2))
        if not identity:
            left.append(match.group(0))
            return match.group(0)
        done += 1
        return f"<ref id={identity}/>"

    guarded = [m.span() for m in PROTECTED.finditer(text)]
    out, cursor = [], 0
    for start, stop in by_spans(text):
        out.append(text[cursor:start])
        piece, at = text[start:stop], start
        # 引用に当たる部分はそのまま、それ以外だけを書き替える。
        inner, seen = [], start
        for a, b in guarded:
            if b <= start or a >= stop:
                continue
            inner.append((max(a, start), min(b, stop)))
        rebuilt = []
        for a, b in inner:
            rebuilt.append(CITATION.sub(one, CROSS_FILE.sub(cross, text[seen:a])))
            rebuilt.append(text[a:b])
            seen = b
        rebuilt.append(CITATION.sub(one, CROSS_FILE.sub(cross, text[seen:stop])))
        out.append("".join(rebuilt))
        cursor = stop
    out.append(text[cursor:])
    return "".join(out), done, left


def render(directory, text):
    """`<ref id=.../>` を、その命題の題と id に展開する。**題は命題から取るので保守されない。**"""
    titles = {}
    for path in documents(directory):
        found, lines = items_in(path)
        for item in found:
            if item["identity"]:
                titles[item["identity"]] = (title_of(lines[item["line"] - 1]) or item["name"],
                                            os.path.basename(path))
    def one(match):
        title, where = titles.get(match.group(1), ("?", "?"))
        return f"{title} ({match.group(1)})"
    return REF.sub(one, text)


def uncovered(directory):
    """命題に属さない本文と、解決しない引用。**形で命題を見つける以上、漏れは必ず出る。**

    出るようにしておけば、漏れは静かに消えるかわりに毎回数えられる -- 実測で、命題の名乗り方が
    7 通りあり、そのうち 2 通りを落として**本文の 44% がどの命題にも属していなかった**。"""
    items, edges = build(directory)
    known = set()
    for path in documents(directory):
        for item in items_in(path)[0]:
            known.add(item["name"])
    lost, orphan = [], []
    for path in documents(directory):
        found, lines = items_in(path)
        covered = set()
        for item in found:
            covered.update(range(*item["span"]))
        live = [i for i, l in enumerate(lines) if l.strip()]
        outside = [i for i in live if i not in covered]
        if outside:
            orphan.append((path, len(outside), len(live)))
        for index in live:
            for name in CITATION.findall(lines[index]):
                if name not in known and not CROSS_FILE.search(lines[index]):
                    lost.append((path, index + 1, name))
    return orphan, lost


# 項目の中で太字から始まる行。**主張はこの形で書かれる** -- 定義の本文が語の意味だけを述べる
# あいだ、それ以外の太字は「…である」と主張している。
CLAIM_LINE = re.compile(r"^\*\*[^*]")


def unproved_claims(directory):
    """項目の中の、印を持たない主張の行。**印が無い主張は、引くことも指紋を取ることもできない。**

    数えるだけで、印は振らない -- 形で推測して振ると散文に振る。実測で、推測した版が
    重複した項目 9 個と、散文に着いた辺 52 本を作った。どこが主張かは人が決める。"""
    out, seen = [], set()
    for path in documents(directory):
        # **証明ファイルの太字は段の一部である。** そこは囲む命題に属していて、独立に引かれない。
        # governed されていない主張が生まれるのは、証明を持たない項目 -- 定義と仮定 -- の中である。
        if os.path.basename(path) != "README.md":
            continue
        found, lines = items_in(path)
        for item in found:
            for index in range(*item["span"]):
                # **1 つの行は 1 度だけ数える。** 親と子は範囲が重なるので、覆う項目ごとに
                # 数えると同じ行を 2 度挙げ、印が 2 つ付く (実測で 58 行)。
                if (CLAIM_LINE.match(lines[index]) and not IDENTITY.search(lines[index])
                        and (path, index) not in seen):
                    seen.add((path, index))
                    out.append((path, index + 1, item["name"], lines[index].strip()))
    return out


def ambiguous(directory):
    """1 つの名前が 2 つの項目に付いている箇所と、項目でないものを指す参照。

    **名前が一意でないと、引用は黙って片方を選ぶ。** 実測で、命題の名前を文中に含む散文の見出しが
    命題として登録され、その名前を引く 34 件の参照が命題でなく散文を指していた --
    引く側は正しい id に見えるので、読んでも分からない。"""
    names, dangling = {}, []
    live = set()
    for path in documents(directory):
        for item in items_in(path)[0]:
            if item["identity"]:
                live.add(item["identity"])
            if item["name"]:
                names.setdefault((os.path.basename(path), item["name"]), []).append(item)
    for path in documents(directory):
        text = open(path, encoding="utf-8").read()
        for match in REF.finditer(text):
            if match.group(1) not in live:
                dangling.append((path, text.count("\n", 0, match.start()) + 1, match.group(1)))
    # **枠の命題の名前は、コーパス全体で 1 つの同一性を持つ。** 言明が枠に、証明が別のファイルに
    # 在る形では、両方に印を振ると 1 つの命題が 2 つの同一性を持つ -- 実測で、`--assign` が
    # 14 本の題に印を振り、引用の辺が 921 本ずれた。
    across = {}
    for path in documents(directory):
        for item in items_in(path)[0]:
            if item["name"] and item["name"][0] in "DAPT" and item["identity"]:
                across.setdefault(item["name"], set()).add(item["identity"])
    doubled = {("コーパス全体", name): [{"line": 0}] * len(ids)
               for name, ids in across.items() if len(ids) > 1}
    return {**{key: rows for key, rows in names.items() if len(rows) > 1}, **doubled}, dangling


# 仮定の表の行。**枠の仮定の集合を数え上げる文書が持つ形である。**
ASSUMPTION_ROW = re.compile(r"^\| \*{0,2}(A\d+[a-z]?)\*{0,2} \|")


def enumerations(directory):
    """枠の仮定を表で数え上げている文書が、落としている仮定。

    **数え上げは引用ではないので、指紋のグラフでは見張れない。** 表は「仮定の集合の全体」を
    主張していて、個々の仮定を引いていない -- 実測で、枠に仮定を 1 つ足した同じコミットが、
    その集合を数え上げる表に行を足し忘れ、`--frontier` は 1 件も挙げなかった。"""
    frame = {item["name"] for item in items_in(os.path.join(directory, "README.md"))[0]
             if item["name"] and item["name"].startswith("A")}
    out = []
    for path in documents(directory):
        if os.path.basename(path) == "README.md":
            continue
        listed = set()
        for line in open(path, encoding="utf-8"):
            row = ASSUMPTION_ROW.match(line)
            if row:
                listed.add(row.group(1))
        # 表を持たない文書は数え上げていない。持つ文書は全部を持たなければならない。
        if len(listed) < len(frame) // 2:
            continue
        for missing in sorted(frame - listed):
            out.append((path, missing))
    return out


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


def default_directory():
    """引数の無いときに見るディレクトリ。証明が 1 つだけならそれを、複数あるなら名指しを求める。"""
    root = "dev-docs/proof"
    found = [os.path.join(base, "") for base, _, files in os.walk(root) if "README.md" in files
             and os.path.abspath(base) != os.path.abspath(root)]
    if len(found) == 1:
        return found[0]
    sys.exit(f"証明のディレクトリを引数で指定すること -- {root} の下に {len(found)} 個ある")


def main(arguments):
    if "--render" in arguments:
        path = arguments[arguments.index("--render") + 1]
        directory = os.path.dirname(path) or "."
        print(render(directory, open(path, encoding="utf-8").read()), end="")
        return 0
    if "--convert" in arguments:
        path = arguments[arguments.index("--convert") + 1]
        directory = os.path.dirname(path) or "."
        out, done, left = convert(directory, path)
        open(path, "w", encoding="utf-8").write(out)
        print(f"{path}: {done} 件を id にした" + (f"、解決しなかった {len(left)} 件" if left else ""))
        return 0
    if "--bundle" in arguments:
        at = arguments.index("--bundle")
        path = arguments[at + 1]
        only = arguments[arguments.index("--item") + 1] if "--item" in arguments else None
        text, count = bundle(os.path.dirname(path) or ".", path, only)
        print(text)
        print(f"<!-- 引く項目 {count} 個 -->", file=sys.stderr)
        return 0
    if "--show" in arguments:
        at = arguments.index("--show")
        roots = [a for a in arguments if not a.startswith("--")]
        directory = roots[0] if len(roots) > 1 else default_directory()
        return show(directory, arguments[at + 1])
    # 値を取る旗の直後の引数は、見に行くディレクトリではない。
    takes_value = {"--file", "--item", "--show", "--bundle", "--render", "--convert"}
    roots = [a for i, a in enumerate(arguments)
             if not a.startswith("--") and (i == 0 or arguments[i - 1] not in takes_value)]
    roots = roots or [default_directory()]
    for directory in roots:
        if "--mark-claims" in arguments:
            print(f"{directory}: 主張の行に印を {mark_claims(directory)} 個振った")
        if "--assign" in arguments:
            print(f"{directory}: 同一性を {assign(directory)} 個振った")
        items, edges = build(directory)
        was = read_ledger(directory)
        moved = [(citing, cited) for citing, cited in sorted(e for e in edges if all(e))
                 if was.get((citing, cited), items[cited]["digest"]) != items[cited]["digest"]]
        if "--accept" in arguments:
            # **読み直した印を進めてよいのは、読み直した者だけである。** 道具の都合で指紋の取り方が
            # 変わったときの再基準化と、枠が動いたときの読み直しは、同じ操作に見えて別物である --
            # 実測で、再基準化のつもりで全体へ掛けて、`A5` が動いて挙がっていた 56 項目の印を、
            # 誰も読み直さないまま進めた。**全体へ掛けるには `--all` を明示する。**
            only = arguments[arguments.index("--file") + 1] if "--file" in arguments else None
            if not only and "--all" not in arguments:
                sys.exit("--accept は読み直した範囲を要求する -- `--file <名前>`、"
                         "または指紋の取り方が変わったときだけ `--all`")
            if only:
                kept = {(citing, cited): was.get((citing, cited), items[cited]["digest"])
                        for citing, cited in edges if all((citing, cited))}
                for citing, cited in moved:
                    if os.path.basename(items[citing]["file"]) == os.path.basename(only):
                        kept[(citing, cited)] = items[cited]["digest"]
                write_ledger(directory, items, edges, kept)
                done = sum(1 for c, _ in moved
                           if os.path.basename(items[c]["file"]) == os.path.basename(only))
                print(f"{directory}: {only} の読み直した印を {done} 件進めた")
            else:
                write_ledger(directory, items, edges, {})
                print(f"{directory}: 読み直した印を {len(moved)} 件進めた (--all)")
            continue
        if not os.path.exists(ledger_path(directory)) or "--write" in arguments:
            write_ledger(directory, items, edges, was)
        # **1 つのファイルを直す者には、そのファイルの分だけを渡す。** 全体の一覧を渡すと、
        # 自分の担当でない行を読むことになる。
        only = arguments[arguments.index("--file") + 1] if "--file" in arguments else None
        for citing, cited in moved:
            here, there = items[citing], items[cited]
            if only and os.path.basename(here["file"]) != os.path.basename(only):
                continue
            print(f"{here['file']}:{here['line']}: {here['name']} が引く "
                  f"{there['name']} ({os.path.basename(there['file'])}) が動いた -- 読み直すこと")
        if "--frontier" in arguments:
            print(f"{directory}: 読み直す要のある項目 {len({c for c, _ in moved})} 個、"
                  f"引用の辺 {len(moved)} 本")
            continue
        orphan, lost = uncovered(directory)
        outside = sum(n for _, n, _ in orphan)
        live = sum(n for _, _, n in orphan)
        print(f"{directory}: 命題 {len(items)} 個、引用の辺 {len(edges)} 本、動いた辺 {len(moved)} 本")
        print(f"{directory}: どの命題にも属さない本文 {outside} 行、解決しない引用 {len(set(lost))} 件")
        names, dangling = ambiguous(directory)
        print(f"{directory}: 1 つの名前が 2 つの項目に付く箇所 {len(names)} 件、"
              f"項目でないものを指す参照 {len(dangling)} 件")
        for (where, name), rows in sorted(names.items()):
            print(f"  {where} の {name}: " + "、".join(f"{one['line']} 行" for one in rows))
        for path, line, identity in dangling:
            print(f"  {os.path.basename(path)}:{line}: {identity} は項目でない")
        missed = enumerations(directory)
        print(f"{directory}: 仮定の数え上げが落としている仮定 {len(missed)} 件")
        for path, name in missed:
            print(f"  {os.path.basename(path)}: {name} が表に無い")
        claims = unproved_claims(directory)
        print(f"{directory}: 項目の中で印を持たない主張の行 {len(claims)} 件")
        for path, n, total in orphan:
            print(f"  {os.path.basename(path)}: {n}/{total} 行がどの命題にも属さない")
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
