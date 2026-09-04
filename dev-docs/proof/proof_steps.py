#!/usr/bin/env python3
"""証明の段の構造を読み、形式の規則を全数検査する。

使い方: `python3 dev-docs/proof/proof_steps.py [ファイルまたはディレクトリ ...]`。
引数を省くと `dev-docs/proof` の下の証明ファイルを全部見る。違反が 1 件でもあれば終了コード 1。

`--cites <名前>` を付けると、検査の代わりに**その名前を `BY` で引く段の一覧**を出す
(`--cites A19` のように使う)。仮定の項が動いたとき、それを引く段を全部挙げるのに使う --
**実測で、その一覧を作らせたときにだけ出た誤読が 3 度ある。**

Lamport の構造化証明は段 `<k>n` の木であり、規則はどれも構文的なので機械が全数を見られる。
検査は 5 つ。

- **段の数**: `<k>n.` の総数。報告の「段の総数」はこれである。
- **`BY` の分類**: `BY` が挙げるトークンを、事実 (`D`/`A`/`P` と命題名と `<k>n` と仮定)・
  `DEF <名前>`・`CODE <ファイル>: <記号>`・`EXT <名前>` の 4 群へ分ける。**どれにも当たらない
  トークンを挙げる** -- 名札の綴り違いと、根拠でないもの (節の番号など) がここに出る。
- **名札の実在**: `DEF <名前>`・`EXT <名前>` として引かれた名前が、その文書で宣言されているか。
- **スコープ規則**: 段が引ける `<k>n` は、**厳密に先行する兄弟と、祖先の先行する兄弟**だけである。
  兄弟の証明の内側を引く形をここで挙げる。
- **支えの無い段**: `BY` も部分証明も持たない段。**記法を導入するだけの `DEFINE` の段は除く** --
  主張を持たないので支えも要らない。
- **読点の落ちたトークン**: 1 つのトークンの中に 2 つ目の**事実**の名札が現れる形。`BY A, B C` の
  ように読点が落ちると、`B` の分類が通ったまま `C` が消える。**接頭辞だけで分類すると、後ろに
  付いたものが見えなくなる。**
  事実の後ろに続く `DEF` と `EXT` は落ちではない -- `BY <事実> DEF <名前>` は Lamport の正規の形で
  ある。ただしその `<名前>` も名札として実在を検査するので、`DEF` の後ろに定義でないものを書いた形は
  「名札の不在」に出る。
- **名札の接頭辞が無い引用**: その文書が `DEF` / `EXT` として宣言した名前を、接頭辞なしで引いた形。
  規則違反ではあるが指すものは定まっているので、未分類とは分けて数える。

**「未分類」は違反の報告ではなく、仕分けの依頼である。** 4 群のどれとも読めなかったトークンが
ここに落ちる -- 節の番号を `BY` に置いた形 (規則違反) と、この道具が知らない局所の名前 (違反でない) の
両方が混じる。**読む者が 1 件ずつ判定する。**

`BY` の外の散文が段を名指すのは道標なので、スコープ規則は `BY` の中だけに掛ける。
段の見出しは 4 通りに書かれる (`<1>0.`、`` `<1>0.` ``、`### <1>1.`、`**<2>1.**`) ので、その全部を読む。
`BY` が行を跨ぐのは並びが読点で終わるときだけなので、継続はそれで決める。
"""
import os
import re
import sys

STEP = re.compile(r"^(\s*)(?:#+\s*)?(?:\*\*)?`?<(\d+)>(\d+[a-z]*)`?\.")
BY = re.compile(r"^\s*BY\s+(.*)$")
REFERENCE = re.compile(r"<(\d+)>(\d+[a-z]*)")
LABEL = re.compile(r"^(DEF|EXT)\s+(.+)$")
CODE = re.compile(r"^CODE\s+[A-Za-z0-9_/.]+\.(rs|fix|pest)\s*:")
FACT = re.compile(
    r"^("
    r"[DAP]\d+[a-z]*"          # 定義・仮定・命題
    r"|[LR]\d+[a-z]*"          # その文書の局所命題と反例
    r"|DEF-\d+"                # 枝番を持つ局所定義
    r"|<\d+>\d+[a-z]*"         # 段
    r"|[A-Z]$|[A-Z] |命題 "      # 1 文字の名前を持つ局所命題
    r"|p\d\d[A-Za-z0-9_-]*(?:\.md)?\s*の\s*"   # 別のファイルの命題・反例 (枠の第 5 節が認める形)
    r"|\(H[\d-]*[a-z]?\)"      # 言明の仮説
    r"|ASSUME|PROVE|NEW|CASE|IH"
    r"|本命題の仮定|本場合の仮定|背理法の仮定"   # 言明・CASE・背理法が置く仮定
    r"|前提|言明|仮定|帰納法の仮定|帰納の仮定|系\d*"
    r")"
)
SECOND_LABEL = re.compile(r"\s(?:CODE\s|[DAP]\d+[a-z]*(?:\s|$)|[LR]\d+[a-z]*(?:\s|$))")
TRAILING_LABEL = re.compile(r"\b(DEF|EXT)\s+(.+)$")
HEDGE = ["明らかに", "自明", "同様にして", "容易に", "であろう", "と思われる", "おそらく", "はずである"]


def normalize_label(name):
    """名札の名前を、引用と宣言で同じ形にする。`` ` `` と空白は書き方の差なので落とす。"""
    return re.sub(r"[\s`]+", "", name)


def declared_labels(text):
    """その文書が宣言した名札。

    宣言の形は 4 通りある -- `**DEF 名前**`、見出しの `## DEF 名前`、記法を並べる箇条書きが使う
    `` - **`名前`** ``、そして名札の語ごとバッククォートに入れる `` - **`EXT 名前`** `` である。
    3 つ目は `DEF` の語を伴わないので、引く側も接頭辞なしで引く。

    **括弧より前を名前として登録する。** 引く側は `(` の前で切った形で引くので
    (`EXT derive(Clone)` を `derive` として照合する)、宣言の側も同じ形を持たないと、その名札を引く
    段が全部「名札の不在」に出る。"""
    names = set()
    for match in re.finditer(r"\*\*(DEF|EXT)\s+([^*]+?)\*\*", text):
        names.add((match.group(1), normalize_label(match.group(2))))
        names.add((match.group(1), normalize_label(match.group(2).split("(")[0])))
    for match in re.finditer(r"^#+\s*(DEF|EXT)\s+(.+?)\s*$", text, re.M):
        names.add((match.group(1), normalize_label(match.group(2))))
        names.add((match.group(1), normalize_label(match.group(2).split("(")[0])))
    for match in re.finditer(r"\*\*`([^`]+)`\*\*", text):
        inner = match.group(1)
        kinds = ("DEF", "EXT")
        labelled = re.match(r"(DEF|EXT)\s+(.+)$", inner)
        if labelled:
            kinds, inner = (labelled.group(1),), labelled.group(2)
        stem = normalize_label(inner.split("(")[0].split(":=")[0])
        if stem:
            for kind in kinds:
                names.add((kind, stem))
    return names


def split_tokens(text):
    """`BY` の並びをトークンへ割る。読点は括弧と `` ` `` の外のものだけを区切りに使う。"""
    tokens, depth, quoted, start = [], 0, False, 0
    for index, char in enumerate(text):
        if char == "`":
            quoted = not quoted
        elif not quoted and char in "(（":
            depth += 1
        elif not quoted and char in ")）":
            depth = max(0, depth - 1)
        elif not quoted and depth == 0 and char in ",、":
            tokens.append(text[start:index])
            start = index + 1
    tokens.append(text[start:])
    return [token.strip().strip("`").strip() for token in tokens if token.strip()]


def order_key(number):
    """`12a` のような枝番つきの段の番号を、並び順に使える鍵へ。"""
    match = re.match(r"(\d+)([a-z]*)", number)
    return (int(match.group(1)), match.group(2))


def parse(text):
    """段を読み、各段について (水準, 番号, 祖先の道, `BY` の本文, 支えを持つか) を返す。"""
    lines = text.split("\n")
    steps, path = [], []
    index = 0
    while index < len(lines):
        match = STEP.match(lines[index])
        if not match:
            index += 1
            continue
        level, number = int(match.group(2)), match.group(3)
        del path[level - 1:]
        path.append((level, number))
        body, index = [lines[index][match.end():]], index + 1
        while index < len(lines) and not STEP.match(lines[index]):
            body.append(lines[index])
            index += 1
        has_substeps = index < len(lines) and int(STEP.match(lines[index]).group(2)) > level
        defines = bool(re.match(r"^\s*(?:\*\*)?DEFINE\b", "\n".join(body).strip() or "x"))

        reasons = []
        for offset, line in enumerate(body):
            by = BY.match(line)
            if not by:
                continue
            text_of_by = [by.group(1)]
            for following in body[offset + 1:]:
                if not text_of_by[-1].rstrip().endswith((",", "、")):
                    break
                if not re.match(r"^\s*(CODE|DEF|EXT|`?[A-Z]|`?<\d|\(H)", following):
                    break
                text_of_by.append(following.strip())
            reasons.append(" ".join(text_of_by))
        steps.append((level, number, list(path[:-1]), reasons,
                      bool(reasons) or has_substeps or defines))
    return steps


def in_scope(path, level, number):
    """`<level>number` を、`path` (自分を末尾に含む) の段から引いてよいか。

    引けるのは、厳密に先行する兄弟と、祖先の先行する兄弟だけである。自分より深い水準は引けない --
    それは兄弟の証明の内側である。"""
    if level > len(path):
        return False
    return order_key(number) < order_key(path[level - 1][1])


CROSS_FILE = re.compile(r"^(p\d\d)[A-Za-z0-9_-]*(?:\.md)?\s*の\s*")


def labels_of_sibling(path_of_file, prefix):
    """`p13 の DEF 名前の活性` の名札は、引く側でなく `p13` の文書が宣言する。

    その文書を同じディレクトリから引き、宣言を返す。見つからなければ `None` -- 引く側の文書を
    その名札の不在で責めないためである。"""
    directory = os.path.dirname(os.path.abspath(path_of_file))
    for name in sorted(os.listdir(directory)):
        if name.startswith(prefix) and name.endswith(".md"):
            return declared_labels(open(os.path.join(directory, name), encoding="utf-8").read())
    return None


def check(path_of_file):
    text = open(path_of_file, encoding="utf-8").read()
    declared = declared_labels(text)
    siblings = {}
    steps = parse(text)
    unclassified, missing, violations, unsupported, bare, run_on = [], [], [], [], [], []
    tokens_seen = 0
    for level, number, ancestors, reasons, supported in steps:
        here = ancestors + [(level, number)]
        if not supported:
            unsupported.append(f"<{level}>{number}")
        for reason in reasons:
            in_code = False
            for token in split_tokens(reason):
                tokens_seen += 1
                if CODE.match(token):
                    in_code = True
                    continue
                label = LABEL.match(token)
                if label:
                    in_code = False
                    name = normalize_label(label.group(2).split("(")[0])
                    if (label.group(1), name) not in declared:
                        missing.append(f"<{level}>{number}: {label.group(1)} {name}")
                    continue
                if not FACT.match(token):
                    if in_code:
                        continue
                    stem = normalize_label(token.split("(")[0])
                    if any(stem == name for _, name in declared):
                        bare.append(f"<{level}>{number}: {token[:60]}")
                    else:
                        unclassified.append(f"<{level}>{number}: {token[:60]}")
                    continue
                in_code = False
                rest = token[FACT.match(token).end():].split("(")[0].split("「")[0]
                trailing = TRAILING_LABEL.search(rest)
                cross = CROSS_FILE.match(token)
                if trailing:
                    name = normalize_label(trailing.group(2).split("(")[0])
                    home = declared
                    if cross:
                        prefix = cross.group(1)
                        if prefix not in siblings:
                            siblings[prefix] = labels_of_sibling(path_of_file, prefix)
                        home = siblings[prefix]
                    if home is not None and (trailing.group(1), name) not in home:
                        missing.append(f"<{level}>{number}: {trailing.group(1)} {name}")
                elif SECOND_LABEL.search(rest):
                    run_on.append(f"<{level}>{number}: {token[:60]}")
                for reference in REFERENCE.finditer(token):
                    referenced_level = int(reference.group(1))
                    if not in_scope(here, referenced_level, reference.group(2)):
                        violations.append(
                            f"<{level}>{number} -> <{referenced_level}>{reference.group(2)}")
    hedges = [word for word in HEDGE if word in text]
    return {
        "steps": len(steps),
        "tokens": tokens_seen,
        "unclassified": unclassified,
        "missing": missing,
        "violations": violations,
        "unsupported": unsupported,
        "bare": bare,
        "run_on": run_on,
        "hedges": hedges,
    }


def citing(path_of_file, name):
    """`name` を `BY` で引く段。仮定や定義が動いたとき、読み直す段を挙げるのに使う。"""
    text = open(path_of_file, encoding="utf-8").read()
    found = []
    for level, number, ancestors, reasons, _ in parse(text):
        for reason in reasons:
            if any(token.startswith(name) for token in split_tokens(reason)):
                path = "".join(f"<{l}>{n} " for l, n in ancestors)
                found.append((f"{path}<{level}>{number}".strip(), reason[:120]))
    return found


def files_under(roots):
    for root in roots:
        if os.path.isfile(root):
            yield root
            continue
        for directory, _, names in os.walk(root):
            for name in sorted(names):
                if name.endswith(".md") and name != "README.md":
                    yield os.path.join(directory, name)


def main(roots):
    failed = 0
    for path_of_file in files_under(roots):
        result = check(path_of_file)
        problems = (result["unclassified"] + result["missing"] + result["violations"]
                    + result["unsupported"] + result["bare"] + result["run_on"]
                    + result["hedges"])
        print(f"{path_of_file}: 段 {result['steps']}、BY のトークン {result['tokens']}、"
              f"未分類 {len(result['unclassified'])}、名札の不在 {len(result['missing'])}、"
              f"スコープ違反 {len(result['violations'])}、支えの無い段 {len(result['unsupported'])}、"
              f"接頭辞の無い名札 {len(result['bare'])}、読点の落ち {len(result['run_on'])}、"
              f"ぼかし語 {len(result['hedges'])}")
        for kind, items in (("未分類", result["unclassified"]), ("名札の不在", result["missing"]),
                            ("スコープ違反", result["violations"]),
                            ("支えの無い段", result["unsupported"]),
                            ("接頭辞の無い名札", result["bare"]),
                            ("読点の落ち", result["run_on"]), ("ぼかし語", result["hedges"])):
            for item in items:
                print(f"  {kind}: {item}")
        failed += len(problems)
    return 1 if failed else 0


if __name__ == "__main__":
    arguments = sys.argv[1:]
    if "--cites" in arguments:
        at = arguments.index("--cites")
        name = arguments[at + 1]
        roots = arguments[:at] + arguments[at + 2:]
        total = 0
        for path_of_file in files_under(roots or ["dev-docs/proof"]):
            for step, reason in citing(path_of_file, name):
                total += 1
                print(f"{path_of_file}: {step}")
                print(f"  BY {reason}")
        print(f"\n{name} を引く段 {total} 件")
        sys.exit(0)
    sys.exit(main(arguments or ["dev-docs/proof"]))
