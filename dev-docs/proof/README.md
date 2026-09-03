# 証明の機械検査

`dev-docs/proof` の下の構造化証明について、構文的に決まる規則を全数検査する道具である。
**証明を書く者と検証する者は、これを走らせてから自分の仕事を始める。**
手で書いた走査は 1 本あたり数十万トークンかかり、実装が人ごとに違って結果が揃わない。

| 道具 | 見るもの |
|---|---|
| `proof_links.py` | `CODE` 引用が名指す記号の実在と、ソース側の `// PROOF:` コメント (`--write` で再生成) |
| `proof_quotes.py` | 証明が「」で引く枠の文が、枠の現在の本文と 1 字ずつ一致するか |
| `proof_steps.py` | 段の数、`BY` のトークンの分類、`DEF`/`EXT` 名札の実在、`<k>n` のスコープ規則、支えの無い段 |
| `proof_readers.py` | 枠が証明ファイルを読み手として名指す箇所の一覧 (照合の材料) |

## 走らせ方

```
python3 dev-docs/proof/proof_links.py
python3 dev-docs/proof/proof_quotes.py dev-docs/proof/rc_ir/borrow-cancel
python3 dev-docs/proof/proof_steps.py dev-docs/proof/rc_ir/borrow-cancel
python3 dev-docs/proof/proof_readers.py dev-docs/proof/rc_ir/borrow-cancel/README.md <出力先>
```

**枠 (`README.md`) を動かしたら、`proof_quotes.py` をその場で走らせる。**
枠の 1 文を書き替えると、その文を引いていた証明の引用が黙って古くなる。
実測で、古い引用が**限定を落とす**向きに働き、限定の無い言明を支えていたことが 7 回あった。

## 道具が見ないもの

**これらは構文でなく内容を見る検査なので、読む者がやる。**

- 枠を「」で引かずに、枠が何を書いているかを**言い替えた**地の文。
- 枠に在るのに**引いていない**節 (`proof_readers.py` は材料を出すだけで、判定はしない)。
- `BY` が挙げた根拠が結論を**支えているか**。
- 数え上げが**尽きているか** (在りかを述語で書き、その述語で全項目を分類し直す)。
