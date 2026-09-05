#!/usr/bin/env python3
"""証明の書式。**この 1 か所だけが、証明の構文を定める。**

道具が 4 本あり、どれも段・`BY`・引用・同一性を読む。それぞれが自分の正規表現を持っていたので、
**同じ概念が 2 か所から 3 か所に書かれ、片方だけを直すことが起きた** -- 実測で 2 度起きている。

- 段の見出しを「番号の後に `.` を持つ行」と直したのは 1 本だけで、残る 2 本は `<2>1` という
  引用を見出しと読み続け、`BY` の続きの行がそこで切れていた (引用 3 件が変換に届かなかった)。
- `#[cfg(test)]` の範囲の取り方を直したのも 1 本だけだった。

**書式を変えるときは、この 1 ファイルだけを変える。**
"""
import re

# 項目の同一性。項目の見出しの行末に置く。
IDENTITY = re.compile(r"<!--#([0-9a-f]{7})-->")

# 変換後の参照。格納されるのは id だけで、題は描画のときに項目から取る。
REF = re.compile(r"<ref id=([0-9a-f]{7})/>")

# 段の見出し。**番号の後に `.` を持つ。** 持たない `<2>1` は引用であって見出しではない。
# 見出しの形で書かれる段があるので `#` の接頭も許す。
STEP = re.compile(r"^(\s*)(?:#+\s*)?(?:\*\*)?`?<(\d+)>(\d+[a-z]*)`?\.")

# `BY` の行。続きの行は深く字下げされて並ぶ。
BY = re.compile(r"^\s*BY\s+(.*)$")

# 段の引用 (`<k>n`)。見出しと違い `.` を持たない。
STEP_REFERENCE = re.compile(r"<(\d+)>(\d+[a-z]*)")

# 名前による引用。定義・仮定・命題と、その文書の局所命題。
CITATION = re.compile(r"\b([DAP]\d+[a-z]*|L\d+[a-z]*)\b")

# 別のファイルの命題を引く形。前半がファイルの接頭、後半がその命題の名前。
CROSS_FILE = re.compile(r"(p\d{2})[A-Za-z0-9_-]*(?:\.md)?\s*の\s*`?(L\d+[a-z]*)`?")

# 別のファイルの名札・反例を引く形。命題は id で引くので、こちらは接頭だけを見る。
CROSS_FILE_PREFIX = re.compile(r"^(p\d\d)[A-Za-z0-9_-]*(?:\.md)?\s*の\s*")

# 証明でない文書は、自分でそう名乗る。
NOT_A_PROOF = "<!--not-a-proof-->"


def by_block(lines, index):
    """`BY` の行 `index` から始まる範囲 (半開区間)。

    **続きの行は「より深く字下げされた行」で決める。書き出しの語では決めない** --
    語の一覧で決めると、その一覧に無い書き出しの続きの行が黙って落ち、そこに在る引用が
    トークンとしても数えられない (実測で 2 件が未分類にすら出ていなかった)。
    """
    indent = len(lines[index]) - len(lines[index].lstrip())
    at = index + 1
    while at < len(lines):
        following = lines[at]
        # **引用の列が続くのは、前の行が区切りで終わっているときだけである。** 区切りなしで終わった
        # `BY` はそこで完結しており、その後ろの深い行はその段の説明の散文である。
        if not lines[at - 1].rstrip().endswith((",", "、")):
            break
        if (not following.strip() or STEP.match(following) or BY.match(following)
                or len(following) - len(following.lstrip()) <= indent):
            break
        at += 1
    return index, at


# 名札の名前。`BY` の中では読点までが 1 つの名札の名前であり、その中の `D12` のような綴りは
# 項目への参照ではない -- 変換がそこを書き替えると、名札の名前が壊れる (実測で 1 件)。
LABEL_NAME = re.compile(r"(?:DEF|EXT|前提)\s+[^,、\n]*")


def is_step(line):
    """その行が段の見出しか。"""
    return bool(STEP.match(line))


def is_by(line):
    """その行が `BY` の行か。"""
    return bool(BY.match(line))


def normalize_label(name):
    """名札の名前を、引用と宣言で同じ形にする。

    `` ` `` と空白は書き方の差なので落とす。**末尾の句読点も落とす** -- 宣言が
    `**DEF 共通接頭の段の中の対応。**` と句点を太字の内側に置く形があり、引く側は句点を付けないので、
    その名札を引く段が全部「名札の不在」に出ていた (実測で 1 ファイル 9 件、全部空振り)。"""
    return re.sub(r"[\s`]+", "", name).rstrip("。、.,:：")
