# `unwrap_newtype` を作り替えて `remove_hktvs` を消す — 引き継ぎ

設計書を書き起こす前の、確定した事実と方針の控え。数字はすべて実測である。

## 0. 決めたこと

issue #245 — phantom な引数位置で増大する型族が `-O max` を止め、メモリを食い尽くす — を、
**バリデーションを足すのではなく `remove_hktvs` パスを消すことで閉じる**。そのために
`unwrap_newtype` を作り替える。言語の契約は変えない（増大する型は書けるままにする）。

退けた案（バリデーションで落とす）は作りきってあり、ブランチ `reject-growing-types`
（コミット `42eac869`）に一式が残っている。7 節を見よ。

## 1. なぜ `remove_hktvs` を消すと #245 が閉じるのか

病的な 6 プログラムを 3 つの構成で走らせた。いずれも `-O max`。

| プログラム | 増大の経路 | 今日 | 両パス停止 | `remove_hktvs` のみ停止 |
|---|---|---|---|---|
| `Y a = { p : Ph (Y (Array a)), h : H Array }` | phantom な引数 | 止まらない | `0` | `0` |
| `Y a = { step : Y (Array a) -> I64, ... }` | 関数型フィールド | 止まらない | `3` | `3` |
| `Grow f = { g : Ph (Grow (Wrap (f I64))), n }` | 高階の引数位置 | 止まらない | `7` | `7` |
| `A a = { y : B A (a,a) }` + `B f b = { x : Ph (f b) }` | 高階パラメータ経由 | 止まらない | `2` | `2` |
| `C g = { y : Ph (C (Wrap g)), n }` | 高階引数だけで増大 | 止まらない | `9` | `9` |
| `C a = { p : Phantom (C (a,a)), n }` | 有限だが伸びる | 通る | `5` | `5` |

**止まらないのは `remove_hktvs` の歩きだけ**である。コード生成も型検査もこれらの型で困らない。
`unwrap_newtype` の `unwrap_type` の再帰も止まる（畳む対象の newtype のフィールド型にしか降りず、
`is_acyclic_newtype` が名前の上の自己参照を弾いているため）。

`remove_hktvs` を止めただけでスイートを回すと 1344/1348 で、失敗 4 件は
`test_unwrap_newtype_partial_application` / `test_unwrap_newtype_cont` /
`test_regression_issue_64_boxed` / `test_regression_issue_64_more_complex_boxed`。
両方止めると 1341/1348 で、失敗 7 件はすべて `test_provenance` の RC IR ダンプ比較。
**`remove_hktvs` を必要としているのは `unwrap_newtype` だけ**である。

## 2. いまの実装がなぜ `remove_hktvs` を要求するのか

`unwrap_newtype` は畳んだ型構築子の**宣言を型環境から取り除く**（`unwrapped_tycons` が
`is_unwrappable` なものを飛ばす）。取り除くと、**飽和していない出現**が宣言されていない型構築子を
名指すので壊れる。飽和しない出現が現れうるのは高階パラメータの引数位置だけである。

壊れる場所は具体的である。`type [f : *->*] Foo f = box struct { data : f () };` の宣言のフィールド型
`f ()` は、`unwrap_type` を通しても `f` が型変数なので何も起きず `f ()` のまま残る。壊れるのはその後、
コード生成が `field_types(Foo IO)` を呼んで `f := IO` を代入し、`IO ()` を得て、`IO` を型環境に
引きに行った時である（実測では `Option::unwrap()` on `None`）。

## 3. 作り替えの方針

- 畳んだ型構築子の**宣言を型環境に残す**。
- `fields_with_instance_types`（`field_types` / `unpunched_field_types` / `fields` がすべて通る
  1 つの漏斗）の**出口で `unwrap_type` を通す**。
- **式の型を畳む部分は今日のまま**。`run_on_symbol` の走査が各節点に `unwrap_type` を当てる。
  最適化に効いているのはここである。
- 構造体操作を消す部分（`MakeStruct` と `InlineLLVMStructGetBody` などの潰し）も今日のまま。

これで足りる理由は、**代入 (`field_types`) が特殊化そのものをやっている**からである。
`remove_hktvs` は代入の結果を新しい 0 パラメータ型構築子として前もって実体化しているだけで、
畳みを代入の後に置けば実体化は要らない。

`x : Foo IO` は `Foo` が box なので畳まれず、`Foo IO` のまま残る（今日は `#RHKTV<Foo IO>` という
名前になるだけで、レイアウトもフィールドが裸のクロージャであることも同じ）。`Foo` が unbox 1
フィールドなら `x` 自身がクロージャ型まで畳まれる。どちらも今日と一致する。

## 4. 受け入れ条件

`unwrap_newtype` の価値は「`IO`・State モナド・イテレータを関数にして、`inline` と
`closure_specialization` に乗せること」であり、そのためには**各式に付く型がクロージャになる**必要が
ある。したがって次を満たすこと。

1. **式に付く型が今日と一致する。** 畳む型の集合を決めるのは今日と同じ `unwrap_type` なので構成から
   従うはずだが、確定させるのは実測である。
2. **コーパス全件で `--emit-llvm` と `--emit-rc-ir` の出力が今日と一致する。** 差が出た箇所だけ読み、
   説明を付ける。差が出るとすれば `Foo IO` 形の型が絡む場所である。
3. **speedtest の数字が退行しない。**
4. スイート全件が通る。`test_provenance` のダンプ比較 7 件が通ることは、`IO` が畳まれている証拠に
   なるので特に見る。

## 5. 危険な所

- **漏斗を通さず `TyConInfo.fields` を直接読んでいる箇所。** コンパイラ本体
  （`object.rs` / `ast/program.rs` / `ast/types.rs` / `optimization/` / `fixstd/builtin.rs`）で
  **11 か所**、LSP・ドキュメント生成まで含めると 87 か所。前者は全数監査が要る。1 つ見落とすと
  畳まれていない型を見る箇所が残り、miscompile になる。後者はレイアウトに関わらないので実害は
  無いはずだが、確認はすること。
- **punched 型。** `punched_from` を持つ宣言は、畳むかどうかを「穴を開けた元の構造体」で決めている
  （`NewtypeUnwrapping::new` の表明）。漏斗の出口で畳む形にしたとき、この対応が保たれること。
- **`unwrap_type` の停止性。** `is_acyclic_newtype` が「フィールド型に自分の名前が出る newtype は
  畳まない」を保証している。増大する newtype `N a = { x : N (Array a) }` も名前の上で自己参照する
  ので畳む対象にならない。作り替えでこの判定を変えないこと。
- **型環境が正規化関数を持つ形になる配管。** 漏斗が `unwrap_type` を呼べるようにする必要があり、
  `TypeEnv` か `Map<TyCon, TyConInfo>` の側に持たせることになる。パスが走る前は恒等でなければ
  ならない（畳む前の段階で畳んではいけない）。

## 6. 残る穴

バリデーションを入れないので、「最適化レベルで挙動が変わらない」ことの保証は**いまの実装について**の
ものであり、構成から従うものではない。将来のパスが「宣言のフィールド型に現れる型」の関係を歩けば、
同じ穴が開く。増大する型はプログラムに書けるままである。

## 7. 退けた案（作りきってある）

ブランチ `reject-growing-types`（コミット `42eac869`）に、最適化の前に走るバリデーションで増大する
型族を落とす版の一式がある。捨てるには惜しい中身が 2 つある。

- `dev-docs/2026-08-11-growing-type-arguments/design.md` の 2 節 —
  **expansive recursion（Kennedy と Pierce の判定、.NET がジェネリクスに課しているもの）が Fix では
  成り立たない**論証。高階パラメータの適用に辺が引けないこと、保守側に倒すと minilib の 11 宣言
  （`StateT` `WriterT` `ResultT` `OptionT` `Free` `Cofree` `StoreT` `TracedT` `PairLT` `PairRT`
  `JmpBuf`）が落ちること。#245 の本文はこの判定を採る前提で書かれているので、閉じるときに訂正が要る。
- `src/tests/test_basic.rs` の病的プログラム 6 本と対照 3 本。作り替え版では**これらが通る**ことを
  主張するテストに書き直せる（今日は `-O max` で止まるので、通ることを主張するテストには今日の
  コンパイラでは書けない）。

スイート全件、minilib 16 サブモジュール、cp-library + project_euler 100 プロジェクトの結果も
同文書の 5 節にある。
