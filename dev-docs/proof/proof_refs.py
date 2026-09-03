#!/usr/bin/env python3
"""枠の塊にタグを振り、証明がそれを引く仕組み。

使い方:

    python3 dev-docs/proof/proof_refs.py <証明のディレクトリ>            # 検査する
    python3 dev-docs/proof/proof_refs.py <証明のディレクトリ> --write    # 生成し直す
    python3 dev-docs/proof/proof_refs.py --new                           # 未使用のタグを 1 つ作る

証明は枠 (`README.md`) の定義・仮定・命題を引く。**引くときに番号や節名や個数を書くと、枠が動いた
分だけ黙って古くなる** -- 実測で、その形の欠陥がこの証明で最も多かった。仕組みはこうである。

- **枠の塊にタグを振る。** タグの行 `<!-- #a3f9c21 -->` の**次の行から次の空行まで**がその塊である。
  段落 1 つ、箇条書きの項目 1 つ、表 1 つがちょうど 1 つの塊になる。
- **タグは 7 桁の 16 進のランダムである。** 内容を表す名前を付けると、内容を直したときに名前を
  直したくなり、直せば引く側を全部直すことになる。**タグは識別子であって説明ではない。**
- **証明が枠の本文を引くときは `QUOTE {タグ}` と書き、本文は道具が埋める。**
  手で写した本文は、枠が動いた瞬間に古くなる。**再生成すれば必ず一致する。**
- **証明が枠を指すだけのときは `表示テキスト {タグ}` と書く。表示テキストも道具が生成する。**
  番号は生成物なので、枠が番号を振り直しても引く側は動かない。
- **引きたい塊にタグが無ければ、タグを入れに行く。** タグを足すのは追記なので誰がやってもよい。
  **枠の本文を変えるのは枠の持ち主だけである。**

この仕組みが閉じるもの:

- **引用の陳腐化。** 再生成が一致を保証するので、照合が要らない。
- **帰属の推測。** どの塊の引用かがタグで決まるので、錨で当てる要が無い (誤検知も消える)。
- **枠の数え上げ。** 「D34 の 6 行」と書けなくなる -- 指すのは塊であって、その要素数ではない。
- **波及の計算。** 枠への参照が全部タグの辺になるので、`proof_impact.py` の一覧が網羅になる。

閉じないもの: **枠が自分自身を数え上げている箇所。** 仮定の前書きが自分の項を並べる形は、
枠の持ち主が見る。
"""
import os
import re
import secrets
import sys

TAG = re.compile(r"^<!--\s*#([0-9a-f]{7})\s*-->\s*$")
QUOTE = re.compile(r"^(\s*)QUOTE\s*\{([0-9a-f]{7})\}\s*$")
REFERENCE = re.compile(r"(?:[^\s{}]+\s*)?\{([0-9a-f]{7})\}")
DISPLAY = re.compile(r"(?:\[[^\]]*\]\s*)?\{([0-9a-f]{7})\}")
ITEM = re.compile(r"\*\*((?:D|A|P|T)\d+[a-z]*)\b")


def blocks(text):
    """タグとその塊。塊はタグの次の行から次の空行までである。"""
    found = {}
    lines = text.split("\n")
    for index, line in enumerate(lines):
        match = TAG.match(line)
        if not match:
            continue
        body, cursor = [], index + 1
        while cursor < len(lines) and lines[cursor].strip():
            body.append(lines[cursor])
            cursor += 1
        found[match.group(1)] = (index + 1, "\n".join(body))
    return found


def owning_item(text, at):
    """その塊が属する項目の名前 (`D34`、`A19` など)。表示テキストの生成に使う。"""
    name = None
    for match in ITEM.finditer(text[:at]):
        name = match.group(1)
    return name or "枠"


def duplicates(text):
    """同じタグが 2 度振られている箇所。"""
    seen, twice = set(), []
    for match in TAG.finditer(text):
        if match.group(1) in seen:
            twice.append(match.group(1))
        seen.add(match.group(1))
    return twice


def proof_files(directory):
    for name in sorted(os.listdir(directory)):
        if name.endswith(".md") and name != "README.md":
            yield os.path.join(directory, name)


def rewrite(path, tagged, source, write):
    """`QUOTE {タグ}` の本文と、参照の表示テキストを生成し直す。差分の件数を返す。"""
    text = open(path, encoding="utf-8").read()
    lines = text.split("\n")
    out, index, stale, missing = [], 0, 0, []
    while index < len(lines):
        quote = QUOTE.match(lines[index])
        if not quote:
            for reference in DISPLAY.finditer(lines[index]):
                if reference.group(1) not in tagged:
                    missing.append((index + 1, reference.group(1)))
            out.append(lines[index])
            index += 1
            continue
        indent, tag = quote.group(1), quote.group(2)
        if tag not in tagged:
            missing.append((index + 1, tag))
            out.append(lines[index])
            index += 1
            continue
        out.append(lines[index])
        index += 1
        had = []
        while index < len(lines) and lines[index].lstrip().startswith(">"):
            had.append(lines[index])
            index += 1
        at, body = tagged[tag]
        want = [indent + "> " + line if line.strip() else indent + ">"
                for line in body.split("\n")]
        if had != want:
            stale += 1
        out.extend(want)
    if write and (stale or True):
        open(path, "w", encoding="utf-8").write("\n".join(out))
    return stale, missing


def main(arguments):
    if "--new" in arguments:
        print(secrets.token_hex(4)[:7])
        return 0
    write = "--write" in arguments
    roots = [a for a in arguments if not a.startswith("--")] or ["dev-docs/proof/rc_ir/borrow-cancel"]
    failed = 0
    for directory in roots:
        frame_path = os.path.join(directory, "README.md")
        frame = open(frame_path, encoding="utf-8").read()
        tagged = blocks(frame)
        twice = duplicates(frame)
        for tag in twice:
            print(f"{frame_path}: タグ {tag} が 2 度振られている")
        failed += len(twice)
        used = set()
        for path in proof_files(directory):
            stale, missing = rewrite(path, tagged, frame, write)
            text = open(path, encoding="utf-8").read()
            used.update(match.group(1) for match in REFERENCE.finditer(text))
            for line, tag in missing:
                print(f"{path}:{line}: タグ {tag} が枠に無い")
            if stale and not write:
                print(f"{path}: 埋め込みが枠と食い違う引用 {stale} 件 (`--write` で直る)")
            failed += len(missing) + (0 if write else stale)
        for tag, (line, _) in sorted(tagged.items(), key=lambda pair: pair[1][0]):
            if tag not in used:
                print(f"{frame_path}:{line}: タグ {tag} を引く証明が無い")
        print(f"{directory}: 枠のタグ {len(tagged)}、引かれているもの {len(used & set(tagged))}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
