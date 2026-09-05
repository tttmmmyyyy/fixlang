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

**1 字ずつ一致していても、別の項目の文でありうる。** 引用は枠のどこかの部分文字列でありさえすれば
一致するので、`D10` の文を `D9` として引く段も、隣り合う 2 つの項目を 1 つの引用に畳んだ段も通る。
実測で、前者が 4 件、後者が 12 件あった。だから**引用がどの項目から来たかを、その文の在りかで決め、
引く段の `BY` がその項目 (またはそれを覆う項目) を挙げているかを見る。**

- **出どころの食い違い (帰属)**: 引く段の `BY` が、引用の出どころの項目を挙げていない。
- **出どころの食い違い (境界)**: 引用が隣り合う 2 つの項目にまたがる。**読み手が 2 文を 1 文に
  畳んでいる形で、後ろの文が置く限定が落ちる。**

錨で分けるのは、証明が自分の言明や `CODE` の doc コメントも「」で引くからである。全部を枠に
当てると、そちらが食い違いとして出てしまう。

**引用は「」だけではない。** 仮定の文面を再掲するとき、証明は blockquote (`> `) を使うことがある。
実測で、**その形に A19 (ii-c) の量化を落とした引用が隠れていた** -- 「」だけを見る道具はそこを通す。
blockquote も同じ錨の規則で当てる。
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import proof_index
import proof_syntax

QUOTE = re.compile(r"「(.+?)」", re.S)
BLOCKQUOTE = re.compile(r"(?:^>[^\n]*\n?)+", re.M)
ANCHOR = 24


IDENTITY = proof_syntax.IDENTITY


def strip_spaces(text):
    """空白と同一性の印を落とす。証明ファイルは枠の 1 文を複数行に折って引くので、行の折り方の差を消す。

    **同一性の印を落とすのは、印が文の途中に置かれるからである。** 印を残すと、印をまたぐ引用が
    すべて食い違いに出る (実測で 69 件中 62 件)。
    """
    return IDENTITY.sub("", re.sub(r"\s+", "", text))


def strip_emphasis(text):
    """強調の `*` を落とす。太字の範囲だけが違う引用を、本文の違う引用と分けるために使う。"""
    return text.replace("*", "")


def offsets(text):
    """空白と印を落とした文字列と、その各文字が元の本文の何文字目かの表。

    **引用がどの項目から来たかを決めるのに要る。** 照合は空白を落とした上でやるので、
    当たった位置を元の本文へ戻せないと、どの項目の中かが分からない。"""
    kept, index = [], []
    at = 0
    while at < len(text):
        marker = IDENTITY.match(text, at)
        if marker:
            at = marker.end()
            continue
        if not text[at].isspace():
            kept.append(text[at])
            index.append(at)
        at += 1
    return "".join(kept), index


def source_of(frame, index, items, quote):
    """引用が枠のどの項目から来たか。錨の当たった位置を項目の範囲に当てる。"""
    for anchor, offset in anchors(quote):
        # **同じ錨が 2 か所に在るときは帰属を決めない。** 最初の出現を答えると、同じ書き出しを
        # 持つ項目が並ぶところで別の項目に帰属する -- 実測で `P6` の引用を `P3` と答えた。
        if frame.count(anchor) != 1:
            continue
        head = frame.find(anchor) - offset
        if head < 0:
            continue
        start = index[min(head, len(index) - 1)]
        stop = index[min(head + len(quote), len(index) - 1)]
        covering = [item for item in items if item["offset"] <= start < item["end"]]
        if not covering:
            return None
        # **覆う項目は複数ある** -- 親と子が重なる。引く側は親を挙げてもよいので、全部返す。
        inner = min(covering, key=lambda item: item["end"] - item["offset"])
        return covering, inner, stop > inner["end"]
    return None


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
    """引用の頭・真ん中・末尾から取った錨と、それぞれが引用の中で始まる位置。

    **位置を返すのは、錨の当たった先から引用の範囲を戻すためである。** 錨の位置を引用の先頭と
    見ると、真ん中や末尾の錨が当たったときに範囲が錨の分だけ後ろへずれ、項目の末尾を越えて
    「引用が 2 つの項目にまたがる」を誤って出す。"""
    if len(quote) < ANCHOR:
        return [(quote, 0)]
    middle = (len(quote) - ANCHOR) // 2
    return [(quote[:ANCHOR], 0), (quote[middle:middle + ANCHOR], middle),
            (quote[-ANCHOR:], len(quote) - ANCHOR)]


def nearest(frame, quote):
    """引用が枠のどのあたりを引こうとしたかを、錨の位置で示す。"""
    for anchor, offset in anchors(quote):
        at = frame.find(anchor)
        if at >= 0:
            head = at - offset
            return frame[max(0, head - 20):head + len(quote) + 20]
    return None


STEP_START = proof_syntax.STEP
BY_LINE = proof_syntax.BY
REF = proof_syntax.REF


def frame_items(directory):
    """枠の項目を、本文の中の文字位置つきで。"""
    path = os.path.join(directory, "README.md")
    found, lines = proof_index.items_in(path)
    starts, at = [], 0
    for line in lines:
        starts.append(at)
        at += len(line) + 1
    out = []
    for item in found:
        first, last = item["span"]
        out.append({"identity": item["identity"], "name": item["name"],
                    "offset": starts[first],
                    "end": starts[last] if last < len(starts) else at})
    return [item for item in out if item["identity"]]


HEADING = re.compile(r"^#+\s")


def citing_by(lines, line):
    """引用の在る段の `BY` の行。引用が段の中に無ければ `None`。

    **段の外の引用にはこの検査を当てない。** 散文の引用に手近な `BY` を結び付けると、
    その段が引いていないものを引用の出どころとして責める -- 実測で 3 件そうなった。
    散文の引用を見るのは `proof_prose.py` の側である。"""
    for index in range(line, -1, -1):
        if STEP_START.match(lines[index]):
            break
        if HEADING.match(lines[index]):
            return None
    else:
        return None
    for index in range(line, min(line + 60, len(lines))):
        if index > line and STEP_START.match(lines[index]):
            break
        if BY_LINE.match(lines[index]):
            first, last = proof_syntax.by_block(lines, index)
            return " ".join(lines[first:last])
    return None


def check(directory):
    """1 つの証明のディレクトリを見て、食い違った引用を返す。"""
    frame = strip_spaces(open(os.path.join(directory, "README.md"), encoding="utf-8").read())
    found = []
    for name in sorted(os.listdir(directory)):
        if not name.endswith(".md") or name == "README.md":
            continue
        text = open(os.path.join(directory, name), encoding="utf-8").read()
        if proof_syntax.NOT_A_PROOF in text[:400]:
            continue
        pieces = list(QUOTE.finditer(text))
        pieces += [match for match in BLOCKQUOTE.finditer(text)]
        for match in pieces:
            body = match.group(1) if match.re is QUOTE else re.sub(r"^>\s?", "", match.group(0), flags=re.M)
            quote = strip_spaces(body)
            if not quote or contains(frame, quote):
                continue
            if not any(anchor in frame for anchor, _ in anchors(quote)):
                continue
            line = text.count("\n", 0, match.start()) + 1
            emphasis = contains(strip_emphasis(frame), strip_emphasis(quote))
            found.append((name, line, body, nearest(frame, quote), emphasis))
    return found


def misattributed(directory):
    """枠と一致する引用のうち、引く段の `BY` がその項目を挙げていないもの。

    **引用が部分文字列として正しくても、別の項目の文でありうる。** 実測で、`D10` の文を `D9` として
    引く段が 3 つ、`D24` の文を `D10` として引く段が 1 つあった -- どれも 1 字ずつ一致するので、
    照合だけの検査は通す。**引用がどの項目から来たかは、その文の在りかで決まる。**

    **項目の境界をまたぐ引用も挙げる。** 項目の途中で切れた引用は、その項目が続けて置く限定を
    落としている可能性がある -- 実測で最も多い欠陥である。"""
    raw = open(os.path.join(directory, "README.md"), encoding="utf-8").read()
    frame, index = offsets(raw)
    items = frame_items(directory)
    out = []
    for name in sorted(os.listdir(directory)):
        if not name.endswith(".md") or name == "README.md":
            continue
        text = open(os.path.join(directory, name), encoding="utf-8").read()
        if proof_syntax.NOT_A_PROOF in text[:400]:
            continue
        lines = text.split("\n")
        for match in list(QUOTE.finditer(text)) + list(BLOCKQUOTE.finditer(text)):
            body = (match.group(1) if match.re is QUOTE
                    else re.sub(r"^>\s?", "", match.group(0), flags=re.M))
            quote = strip_spaces(body)
            if len(quote) < ANCHOR or not contains(frame, quote):
                continue
            where = source_of(frame, index, items, quote)
            if not where:
                continue
            covering, item, crosses = where
            line = text.count("\n", 0, match.start())
            by = citing_by(lines, line)
            if by is None:
                continue
            named = set(REF.findall(by))
            # 名前のまま残っている引用も引用である。変換が届いていない形 (`D24 の (E7)`、
            # 括弧の中の `L10 (b)`) を違反として挙げない。
            named |= {one["name"] for one in covering
                      if one["name"] and re.search(rf"\b{one['name']}\b", by)}
            if not named & ({one["identity"] for one in covering}
                            | {one["name"] for one in covering if one["name"]}):
                out.append((name, line + 1, covering, quote[:60], "帰属"))
            elif crosses:
                out.append((name, line + 1, covering, quote[:60], "境界"))
    return out


def report_attribution(directory):
    """帰属の食い違いを印字し、件数を返す。"""
    rows = misattributed(directory)
    for name, line, covering, quote, kind in rows:
        names = [one["name"] or one["identity"] for one in covering]
        label = ("引く段の `BY` がこの引用の出どころを挙げていない" if kind == "帰属"
                 else "引用が 2 つの項目にまたがる -- 2 文を 1 文に畳んでいる")
        print(f"{os.path.join(directory, name)}:{line}: {label}")
        print(f"  出どころ: {' < '.join(names)}")
        print(f"  引用: {quote}")
    return rows


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
        attribution = sum(len(report_attribution(directory)) for directory in directories)
        print(f"\n本文の食い違い {text_differences} 件、強調の範囲だけの差 {emphasis_differences} 件、"
              f"出どころの食い違い {attribution} 件")
    return 1 if text_differences or emphasis_differences else 0


if __name__ == "__main__":
    roots = sys.argv[1:]
    if not roots:
        roots = [path for path, _, files in os.walk("dev-docs/proof") if "README.md" in files]
    sys.exit(main(roots))
