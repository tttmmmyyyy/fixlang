#!/usr/bin/env python3
"""証明の地の文から、枠に当てる要のある文と、コードに当てる要のある文を抜き出す。

使い方: `python3 dev-docs/proof/proof_prose.py <証明ファイル> ...`。

`proof_quotes.py` が見るのは「」で引いた文だけである。**枠を引かずに、枠が何を書いているかを
言い替えた地の文**はそこに掛からない -- 実測で、その族だけが捕まえた FALSE が複数の周で出ている。
同じく、**「`impl LLVMGen for` は 78 個」のようにコードを数えた文**も引用ではないので掛からない。

この道具は判定しない。**当てる要のある文を漏れなく並べる**ところまでをやる。
判定は読む者がやる -- 枠の本文とコードのどちらに当てるかは、文ごとに違うからである。

3 つを出す。

- **枠について述べる文**: `README`・`枠`・`A<n>`・`D<n>`・`P<n>`・`(E<n>)`・`(S-x)`・`第 <n> 節` を
  含み、「」の外に在る文。**枠を名指す文と、枠の項目について限定を述べる文に分ける** --
  実測で偽はその 2 つに集まり、項目の番号を挙げるだけの文には出ない。
- **数を主張する文**: 数と助数詞を持ち、**`src/` に実在する識別子**を伴う文。実在で絞るのは、
  数式の記号 (`C_1` など) を落とすためである。**その識別子を並べて出す**ので、読む者はそれを
  クレート全体に当てるだけでよい。
- **表明に触れる文**: `assert!`・`panic!`・`expect`・`unreachable!` に触れる文。
  表明を命題の根拠にしている段がここに出る。
"""
import os
import re
import sys

NAMES_FRAME = re.compile(r"README|枠が|枠は|枠の")
FRAME_ITEM = re.compile(r"[ADP]\d+[a-z]*|\(E\d+\)|\(S-[a-z]\)|D\d+ の")
DOUBT = re.compile(r"だけ|のみ|しか|持たない|尽き|尽くす|に限|でない|ではない|書いていない|述べない"
                   r"|より弱|より強|1 つも|どれも|すべて|全部|唯一|無い")
COUNT = re.compile(r"\d+\s*(個|か所|箇所|件|種|行|本|通り)")
IDENTIFIER = re.compile(r"`([A-Za-z_][A-Za-z0-9_:<>]*(?:\([^`]*\))?)`|`(src/[A-Za-z0-9_/.]+)`")
ASSERTION = re.compile(r"assert!|assert_eq!|panic!|panic_with_msg|expect\(|unreachable!|debug_assert")
SENTENCE = re.compile(r"[^。]*。")


def crate_identifiers(root="src"):
    """`src/` に現れる Rust の項目の名前。数の主張が数式でなくコードについてであることを、これで決める。"""
    names = set()
    declaration = re.compile(r"\b(?:fn|struct|enum|trait|type|const|static|mod)\s+([A-Za-z_][A-Za-z0-9_]*)")
    for directory, _, files in os.walk(root):
        for name in files:
            if not name.endswith(".rs"):
                continue
            with open(os.path.join(directory, name), encoding="utf-8", errors="ignore") as source:
                names.update(declaration.findall(source.read()))
    return names


def strip_quotations(text):
    """「」の中を伏せる。引用は `proof_quotes.py` が見るので、ここでは地の文だけを残す。"""
    return re.sub(r"「[^」]*」", "「」", text, flags=re.S)


def sentences(text):
    """行をまたぐ文を 1 つに繋いでから、句点で割る。証明ファイルは 1 文を数行に折る。"""
    body = []
    for block in re.split(r"\n\s*\n", text):
        if block.lstrip().startswith(("```", "|", "$")):
            continue
        body.append(re.sub(r"\s*\n\s*", "", block))
    return [s.strip() for block in body for s in SENTENCE.findall(block) if s.strip()]


def scan(path, crate):
    text = strip_quotations(open(path, encoding="utf-8").read())
    frame, counts, assertions = [], [], []
    for sentence in sentences(text):
        names_frame = bool(NAMES_FRAME.search(sentence))
        if names_frame or (FRAME_ITEM.search(sentence) and DOUBT.search(sentence)):
            frame.append((names_frame, sentence))
        if COUNT.search(sentence):
            names = [a or b for a, b in IDENTIFIER.findall(sentence)]
            names = [n for n in names if n.split("(")[0].split("::")[-1] in crate or n.startswith("src/")]
            if names:
                counts.append((names, sentence))
        if ASSERTION.search(sentence):
            assertions.append(sentence)
    return frame, counts, assertions


def main(paths):
    crate = crate_identifiers()
    for path in paths:
        frame, counts, assertions = scan(path, crate)
        named = [s for flagged, s in frame if flagged]
        implied = [s for flagged, s in frame if not flagged]
        print(f"=== {path}")
        print(f"枠を名指す文 {len(named)} 件、枠の項目について限定を述べる文 {len(implied)} 件、"
              f"数を主張する文 {len(counts)} 件、表明に触れる文 {len(assertions)} 件")
        print("\n-- 枠に当てる文 (枠を名指すもの)")
        for sentence in named:
            print(f"  {sentence}")
        print("\n-- 枠に当てる文 (項目について限定を述べるもの)")
        for sentence in implied:
            print(f"  {sentence}")
        print("\n-- コードに当てる文 (数を主張するもの)")
        for names, sentence in counts:
            print(f"  [{' '.join(names)}] {sentence}")
        print("\n-- 表明に触れる文")
        for sentence in assertions:
            print(f"  {sentence}")
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
