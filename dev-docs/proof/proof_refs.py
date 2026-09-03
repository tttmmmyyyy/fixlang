#!/usr/bin/env python3
"""枠の塊に名札を付け、証明がそれを引く仕組み。

使い方:

    python3 dev-docs/proof/proof_refs.py <証明のディレクトリ>            # 検査する
    python3 dev-docs/proof/proof_refs.py <証明のディレクトリ> --write    # 埋め込みを生成し直す
    python3 dev-docs/proof/proof_refs.py <証明のディレクトリ> --accept   # 読み直した印を進める
    python3 dev-docs/proof/proof_refs.py --new                           # 未使用の名前を 1 つ作る

証明は枠 (`README.md`) の定義・仮定・命題を引く。**引くときに番号や節名や個数を書くと、枠が動いた
分だけ黙って古くなる** -- 実測で、その形の欠陥がこの証明で最も多かった。仕組みはこうである。

    枠:    <label name=d050777>**(ii-c) (段内の点の非負性)。…**</label>

    証明:  <quote name=d050777 hash=3f9c210/>
           > (道具が埋める)

           <ref name=d050777 hash=3f9c210/>

- **名前は 7 桁の 16 進のランダムである。** 内容を表す名前を付けると、内容を直したときに名前を
  直したくなり、直せば引く側を全部直すことになる。**名前は識別子であって説明ではない。**
- **`<label>` は塊を囲む。** 範囲を「次の空行まで」のような形で推すと、**塊の途中に空行が 1 つ
  入っただけで引用が黙って縮み、`--write` がその縮んだ本文を全ファイルへ書き込む。**
  空行の挿入は無害に見える編集なので、気付く道が無い。囲めばその道は閉じる。
- **`hash` は「この版を読んだ」という記録である。** 枠が動けば不一致になる。
  **`--write` は埋め込みを直すが `hash` は動かさない** -- 埋め込みの再生成は本文を揃える一方で
  「読み直す必要」を隠すからである。実測で最も多い欠陥は「引用から限定が落ちて、段が主張できる
  範囲が変わった」であり、それは本文を揃えるだけでは直らない。**`hash` を進めるのは、読み直した者が
  `--accept` で行う。**
- **不一致の集合が、そのまま「読み直す要のある段」である。** 枠の 2 つのコミットを差分する要は無い。
- **引きたい塊に名札が無ければ、名札を付けに行く。** 付けるのは追記なので誰がやってもよい。
  **枠の本文を変えるのは枠の持ち主だけである。**

閉じないもの: **枠が自分自身を数え上げている箇所。** 仮定の前書きが自分の項を並べる形は、
枠の持ち主が見る。
"""
import hashlib
import os
import re
import secrets
import sys

LABEL = re.compile(r"<label\s+name=([0-9a-f]{7})\s*>(.*?)</label>", re.S)
CITE = re.compile(r"<(quote|ref)\s+name=([0-9a-f]{7})\s+hash=([0-9a-f]{7})\s*/>")
NAME_IN_LABEL = re.compile(r"<label\s+name=([0-9a-f]{7})\s*>")
ITEM = re.compile(r"\*\*((?:D|A|P|T)\d+[a-z]*)\b")


def digest(body):
    """塊の内容の指紋。行の折り方の差は消す -- 折り直しは読み直しを要さないからである。"""
    return hashlib.sha256(re.sub(r"\s+", "", body).encode("utf-8")).hexdigest()[:7]


def labels(text):
    """名札の名前から (内容, 指紋, 行) への表。"""
    found = {}
    for match in LABEL.finditer(text):
        body = match.group(2).strip("\n")
        found[match.group(1)] = (body, digest(body), text.count("\n", 0, match.start()) + 1)
    return found


def duplicated(text):
    seen, twice = set(), []
    for match in NAME_IN_LABEL.finditer(text):
        if match.group(1) in seen:
            twice.append(match.group(1))
        seen.add(match.group(1))
    return twice


def proof_files(directory):
    for name in sorted(os.listdir(directory)):
        if name.endswith(".md") and name != "README.md":
            yield os.path.join(directory, name)


def visit(path, known, write, accept):
    """1 つの証明を見る。(枠に無い名前, 指紋の不一致, 埋め込みの食い違い) を返す。"""
    lines = open(path, encoding="utf-8").read().split("\n")
    out, index = [], 0
    unknown, changed, stale = [], [], 0
    while index < len(lines):
        line = lines[index]
        match = CITE.search(line)
        if not match:
            out.append(line)
            index += 1
            continue
        kind, name, seen = match.group(1), match.group(2), match.group(3)
        if name not in known:
            unknown.append((index + 1, name))
            out.append(line)
            index += 1
            continue
        body, now, _ = known[name]
        if seen != now:
            changed.append((index + 1, name, seen, now))
            if accept:
                line = line.replace(f"hash={seen}", f"hash={now}")
        out.append(line)
        index += 1
        if kind != "quote":
            continue
        indent = re.match(r"\s*", match.string[:match.start()]).group(0)
        had = []
        while index < len(lines) and lines[index].lstrip().startswith(">"):
            had.append(lines[index])
            index += 1
        want = [f"{indent}> {row}" if row.strip() else f"{indent}>" for row in body.split("\n")]
        if had != want:
            stale += 1
        out.extend(want if (write or accept) else had)
    if write or accept:
        open(path, "w", encoding="utf-8").write("\n".join(out))
    return unknown, changed, stale


def main(arguments):
    if "--new" in arguments:
        print(secrets.token_hex(4)[:7])
        return 0
    write, accept = "--write" in arguments, "--accept" in arguments
    roots = [a for a in arguments if not a.startswith("--")] or ["dev-docs/proof/rc_ir/borrow-cancel"]
    problems = 0
    for directory in roots:
        frame_path = os.path.join(directory, "README.md")
        frame = open(frame_path, encoding="utf-8").read()
        known = labels(frame)
        for name in duplicated(frame):
            print(f"{frame_path}: 名札 {name} が 2 度付いている")
            problems += 1
        cited = set()
        for path in proof_files(directory):
            unknown, changed, stale = visit(path, known, write, accept)
            cited.update(match.group(2) for match in CITE.finditer(open(path, encoding="utf-8").read()))
            for line, name in unknown:
                print(f"{path}:{line}: 名札 {name} が枠に無い")
            for line, name, seen, now in changed:
                print(f"{path}:{line}: {name} は読んだ版 {seen} から {now} へ動いた -- 読み直すこと")
            if stale and not (write or accept):
                print(f"{path}: 埋め込みが枠と食い違う引用 {stale} 件 (`--write` で直る)")
            problems += len(unknown) + len(changed) + (0 if (write or accept) else stale)
        for name, (_, _, line) in sorted(known.items(), key=lambda pair: pair[1][2]):
            if name not in cited:
                print(f"{frame_path}:{line}: 名札 {name} を引く証明が無い")
        print(f"{directory}: 枠の名札 {len(known)}、引かれているもの {len(cited & set(known))}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
