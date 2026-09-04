#!/usr/bin/env python3
"""証明ファイルが「」で引く枠の文を、枠の本文と 1 字ずつ照合する。

使い方: `python3 dev-docs/proof/proof_quotes.py [証明のディレクトリ ...]`。
引数を省くと `dev-docs/proof` の下で `README.md` を持つディレクトリを全部見る。
食い違いが 1 件でもあれば終了コード 1 を返す。

証明は枠 (`README.md`) の定義・仮定・命題を「」で引いて `BY` の根拠にする。枠が動いたとき、
その引用は黙って古くなる -- 引用の中の 1 語が枠の現在の本文と違っても、証明は今までどおり読める。
実測では、古い引用が**限定を落とす**向きに働き、限定の無い言明を支えていたことが 7 回あった。

食い違いは 4 つに分ける。

- **一致**: 空白を除いた引用が、空白を除いた枠の本文の部分文字列である。`…` と `...` は
  引用の省略なので、その前後を順に探して繋がれば一致とする。
- **食い違い**: 引用の中に枠と 24 字以上一致する部分が在るのに、全体としては一致しない。
  **枠が動いたか、引用が捏造されている。** これを報告する。
- **強調だけの差**: `*` を落とすと一致する。太字の範囲がずれている。別に数えて報告する --
  実測では、太字の中に在る限定を太字の外へ出した引用が、限定の無い言明を支えていた。
- **枠の外**: 24 字の錨がどこにも無い。自分の文書の再掲か、コードの doc コメントの引用である。

錨で分けるのは、証明が自分の言明や `CODE` の doc コメントも「」で引くからである。全部を枠に
当てると、そちらが食い違いとして出てしまう。

**引用は「」だけではない。** 仮定の文面を再掲するとき、証明は blockquote (`> `) を使うことがある。
実測で、**その形に A19 (ii-c) の量化を落とした引用が隠れていた** -- 「」だけを見る道具はそこを通す。
blockquote も同じ錨の規則で当てる。
"""
import os
import re
import sys

QUOTE = re.compile(r"「(.+?)」", re.S)
BLOCKQUOTE = re.compile(r"(?:^>[^\n]*\n?)+", re.M)
ANCHOR = 24


IDENTITY = re.compile(r"<!--#[0-9a-f]{7}-->")


def strip_spaces(text):
    """空白と同一性の印を落とす。証明ファイルは枠の 1 文を複数行に折って引くので、行の折り方の差を消す。

    **同一性の印を落とすのは、印が文の途中に置かれるからである。** 印を残すと、印をまたぐ引用が
    すべて食い違いに出る (実測で 69 件中 62 件)。
    """
    return IDENTITY.sub("", re.sub(r"\s+", "", text))


def strip_emphasis(text):
    """強調の `*` を落とす。太字の範囲だけが違う引用を、本文の違う引用と分けるために使う。"""
    return text.replace("*", "")


def contains(frame, quote):
    """`…` と `...` を省略として、引用が枠に順に現れるか。"""
    pieces = [piece for piece in re.split(r"…|\.\.\.", quote) if piece]
    at = 0
    for piece in pieces:
        at = frame.find(piece, at)
        if at < 0:
            return False
        at += len(piece)
    return True


def anchors(quote):
    """引用の頭・真ん中・末尾から取った錨。どれか 1 つが枠に在れば、枠を引いた引用である。"""
    if len(quote) < ANCHOR:
        return [quote]
    middle = (len(quote) - ANCHOR) // 2
    return [quote[:ANCHOR], quote[middle:middle + ANCHOR], quote[-ANCHOR:]]


def nearest(frame, quote):
    """引用が枠のどのあたりを引こうとしたかを、錨の位置で示す。"""
    for anchor in anchors(quote):
        at = frame.find(anchor)
        if at >= 0:
            return frame[max(0, at - 20):at + len(quote) + 20]
    return None


def check(directory):
    """1 つの証明のディレクトリを見て、食い違った引用を返す。"""
    frame = strip_spaces(open(os.path.join(directory, "README.md"), encoding="utf-8").read())
    found = []
    for name in sorted(os.listdir(directory)):
        if not name.endswith(".md") or name == "README.md":
            continue
        text = open(os.path.join(directory, name), encoding="utf-8").read()
        pieces = list(QUOTE.finditer(text))
        pieces += [match for match in BLOCKQUOTE.finditer(text)]
        for match in pieces:
            body = match.group(1) if match.re is QUOTE else re.sub(r"^>\s?", "", match.group(0), flags=re.M)
            quote = strip_spaces(body)
            if not quote or contains(frame, quote):
                continue
            if not any(anchor in frame for anchor in anchors(quote)):
                continue
            line = text.count("\n", 0, match.start()) + 1
            emphasis = contains(strip_emphasis(frame), strip_emphasis(quote))
            found.append((name, line, body, nearest(frame, quote), emphasis))
    return found


def main(directories):
    text_differences = emphasis_differences = 0
    for directory in directories:
        for name, line, quote, near, emphasis in check(directory):
            if emphasis:
                emphasis_differences += 1
                kind = "強調の範囲だけが枠と違う引用"
            else:
                text_differences += 1
                kind = "枠と食い違う引用"
            print(f"{os.path.join(directory, name)}:{line}: {kind}")
            print(f"  引用: {strip_spaces(quote)[:160]}")
            if near:
                print(f"  枠　: {near[:200]}")
    if text_differences or emphasis_differences:
        print(f"\n本文の食い違い {text_differences} 件、強調の範囲だけの差 {emphasis_differences} 件")
    return 1 if text_differences or emphasis_differences else 0


if __name__ == "__main__":
    roots = sys.argv[1:]
    if not roots:
        roots = [path for path, _, files in os.walk("dev-docs/proof") if "README.md" in files]
    sys.exit(main(roots))
