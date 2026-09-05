#!/usr/bin/env python3
"""枠が証明ファイルを読み手として名指す箇所を、ファイルごとの一覧に落とす。

使い方: `python3 dev-docs/proof/proof_readers.py <README.md のパス> <出力ディレクトリ> [対象の版]`。

枠は「読む者は `pNN-....md` の `L9b` である」の形で、どの証明がその節を使うかを書く。
**名指された段がその節を実際に `BY` で引いていなければ、その段は枠が埋めた節を使い損ねているか、
もう必要でない前提を負っている。** その照合の材料をここで作る。

**この一覧は下界である。** 拾うのはファイル名で名指す箇所だけなので、命題名や補題名だけで読み手を
名指す節 (「読む者は P3、P4、P5 (a)、P6 である」の形) は入らない。読む者はそのファイルが証明する
命題の番号でも枠を走査する。

README のパスを引数に取るのは、相対で開くと別のディレクトリの `README.md` を読んで 0 件を返し、
その 0 件で一覧を上書きするからである。名指しが 1 つも見つからなければ、書かずに止まる。
"""
import re, io, sys, os

files = ["p05-holders", "p10-leaves-and-units", "p11-origin-soundness", "p12-identity-and-consumes",
         "p13-disposals-and-pending", "p15-ownership-uniformity", "p20-borrow-ify", "p30-cancel-walk",
         "p40-cancel-soundness", "p50-observation", "p51-runs", "p60-insert-rc", "p70-main-theorem"]


def annotate(lines):
    """各行について、直前の見出しと、直前の仮定・命題の番号を返す。"""
    heads, items = [], []
    head, item = "", ""
    for line in lines:
        if line.startswith("#"):
            head, item = line.strip("# ").strip(), ""
        match = re.match(r"- \*\*([A-Z]\d+[a-z]*)\*\*", line)
        if match:
            item = match.group(1)
        heads.append(head)
        items.append(item)
    return heads, items


def paragraph(lines, index):
    """`index` 行を含む段落の範囲を返す。"""
    start = index
    while start > 0 and lines[start - 1].strip():
        start -= 1
    end = index
    while end + 1 < len(lines) and lines[end + 1].strip():
        end += 1
    return start, end + 1


def main(readme, out_dir, revision):
    lines = io.open(readme, encoding="utf-8").read().split("\n")
    heads, items = annotate(lines)
    total = sum(1 for line in lines for name in files if name + ".md" in line)
    if total == 0:
        raise SystemExit(f"{readme} は証明ファイルを 1 つも名指していない。読む README を間違えている。")
    os.makedirs(out_dir, exist_ok=True)
    for name in files:
        rows, seen = [], set()
        for index, line in enumerate(lines):
            if name + ".md" not in line:
                continue
            start, end = paragraph(lines, index)
            text = "\n".join(lines[start:end])
            if text in seen:
                continue
            seen.add(text)
            rows.append((heads[index], items[index], text))
        with io.open(os.path.join(out_dir, name + ".md"), "w", encoding="utf-8") as out:
            out.write(f"# README が `{name}.md` を名指す箇所 (対象 `{revision}`、{len(rows)} 段落)\n\n")
            out.write("**この一覧の各段落について、名指された段が実際にその節を `BY` で引いているかを"
                      "1 件ずつ確かめること。**\n**引いていなければ、その段はもう必要でない前提を"
                      "負っているか、枠が埋めた節を使い損ねている。**\n\n"
                      "**この一覧は下界である。** 拾うのはファイル名で名指す箇所だけなので、"
                      "**命題名や補題名だけで読み手を名指す節** (「読む者は P3、P4、P5 (a)、P6 である」"
                      "のような形) は入っていない。**そのファイルが証明する命題の番号でも README を"
                      "走査すること。**\n\n")
            for head, item, text in rows:
                out.write(f"## {head}" + (f" -- {item}" if item else "") + "\n\n" + text + "\n\n")
        print(f"{name:32s} {len(rows):2d} 段落")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3] if len(sys.argv) > 3 else "HEAD")
