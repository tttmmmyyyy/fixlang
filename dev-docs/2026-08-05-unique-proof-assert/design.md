# 一意性証明の開発モード検査を全経路に出す (#198)

## 目的

一意性解析が「この値は unique なのでクローンを省いてよい」と判断した箇所で、その証明を
開発モード時に参照カウントと突き合わせる。誤った証明は、他の保持者から見える値をその場で
上書きすることを意味し、症状は実行時のサイレントな破壊になる。この検査は、破壊を書き込みの
地点で止めるために置かれている。

## 確認した事実

issue の 3 つの指摘は、コードを読んで確認できた。

### 1. 検査は 18 の op のうち 2 つにしか出ない

証明を受け取る op、すなわち `assuming_unique` を実装する inline-LLVM op は 18 個ある。
`force_unique` フィールドを持つ 16 個が証明を受けるとクローンを落とし、残る 2 個は
検査の結果そのものを定数 `true` に畳む。

| 群 | 数 | 証明を受けたときの形 | 検査 |
|---|---|---|---|
| 配列（`make_array_unique` 経由） | 9 | `if force_unique { make_array_unique(..) } else { array }` | なし |
| 配列（hole 付き、`PunchedArrayPlugBody`） | 1 | `make_array_unique_with_hole` | なし |
| 配列（`ArraySetCapacityBoundsUnchecked`） | 1 | `if !force_unique { realloc_array(..) }` の早期 return | なし |
| 構造体（`StructPunch` / `StructPlugIn` / `StructSet`） | 3 | `if force_unique { make_struct_unique(..) }` | なし |
| boxed（`_mutate_boxed_internal` / `_ios` 版） | 2 | `force_unique_boxed(..)` | **あり** |
| 定数に畳む（`is_unique` / `Array::is_storage_unique`） | 2 | フラグを `true` にして返す | なし |

RC IR の `specialize` が実際に一意性を証明するのは主に配列の書き込み経路なので、検査は
それを最も必要とする 11 の配列 op すべてに届いていない。定数に畳む 2 つでは、誤った証明は
「プログラムに嘘の答えを返す」形で出る。

### 2. global の誤証明を素通しする

本来の検査 `Generator::build_branch_by_is_unique` は、まず
`build_branch_by_refcnt_state` で state を見て、**GLOBAL のオブジェクトをカウントに関わらず
shared 側へ流す**。一方 `Generator::mark_global_one` は state を書き換えるだけでカウントに
触らないので、global なオブジェクトのカウントは `create_obj` が入れた 1 のままである。

現行の `assert_proven_unique` はカウントだけを `> 1` と比べるため、
「プログラム全体で共有される値を unique と誤証明する」という最悪のケースを通す。

### 3. threaded 状態でカウントを非アトミックに読む

本来の検査は threaded 状態では acquire の atomic load を使う（TSan が standalone fence から
happens-before を引かないため、acquire を load 自身に置くことが要る）。`assert_proven_unique`
は素の load である。

`Configuration::develop_mode()` は `threaded` を立てず、`develop_mode` を立てる経路は
インプロセステストだけなので、この穴は現状どのビルドからも到達しない。ただし下の設計では、
この穴は個別の対処ではなく「本来の検査と同じ機構を使う」ことの帰結として閉じる。

### 4. 検査が発火しうることを固定するテストがない

`UGT` を `ULT` に反転しても、`specialize` が `force_unique` をクリアしなくなっても、
テストスイートは緑のままになる。


## 終状態

### A. `Generator::build_assert_unique`

`build_assert_refcnt_state_local` の隣に置く。

```rust
fn build_assert_unique(&mut self, obj_ptr: PointerValue<'c>, state: RcState)
```

`build_branch_by_is_unique` と同じ骨格で、shared 側を abort にしたもの。

- `develop_mode` でなければ何も出さない
- `build_branch_by_refcnt_state(obj_ptr, state)` を再利用して local / threaded / global を得る
- local: 素の load で `refcnt == 1` を検査
- threaded: acquire の atomic load で同上。**state は書き換えない**（`build_branch_by_is_unique`
  が unique 側で行う `mark_local_one` は、検査には属さない）
- global: 無条件に abort

`state.dispatches()` が false のときは `build_branch_by_refcnt_state` が
`build_assert_refcnt_state_local` を出したうえで現在ブロックを返すので、local 相当の検査だけが
残る。これは locality 注釈の検査と一意性証明の検査が同じ地点で重なる、正しい姿である。

これで指摘 2 と 3 は、別々の対処ではなく設計の帰結として閉じる。

### B. 証明でクローンを落とす判断を集める

現在 16 の `generate` に散っている `if self.force_unique { ... } else { ... }` を、
`force_unique` を引数に取る関数へ載せ替える。`builtin.rs` の `generate` から
`if self.force_unique` を消し、判断が残るのは次の 3 箇所になる。

1. `force_unique_or_assert(gc, val, force_unique, state) -> Object`
   現在の `force_unique_boxed` を一般化する。unbox な値はカウントを持たないのでそのまま返し、
   配列は storage を、それ以外の boxed は値そのものを対象にする。13 の op がここに載る。
2. hole 付きの変種。`PunchedArrayPlugBody` 1 箇所のために残す。
3. `ArraySetCapacityBoundsUnchecked`。unique 側が `realloc` の in-place 縮小・拡大で、
   「unique にした値を返す」形をしていない。早期 return の先頭で `build_assert_unique` を直接呼ぶ。

### C. 定数に畳む 2 つの op

`is_unique` と `Array::is_storage_unique` は返す値を作らないので、上の関数には載らない。
フラグを `true` にする地点で `build_assert_unique` を直接呼ぶ。

## 実装手順

1. `build_assert_unique` を追加する。
2. `force_unique_or_assert` へ 13 の op を載せ替える。
3. hole 付き変種と `ArraySetCapacityBoundsUnchecked` を処理する。
4. 定数に畳む 2 つの op に検査を入れる。
5. 現行の `assert_proven_unique` を削除する。
6. スイートを `-O none` / `-O max` で回す。

## 検証

### global を即 abort にしてよいか

よい。`rc_ir/provenance.rs` の `LeafOrigin::Unknown` は「boxed コンテナから読んだ値、
**global**、`Retain` で複製された値」であり、`resolve_leaf` はこれを `Dynamic` に落とす。
`leaf_is_unique` が `true` を返すのは `SharingVerdict::Unique` のときだけなので、
**解析は global を unique と証明しない**。証明が global を指したならそれは解析の誤りである。

### 発火の証明（1 回きり、テストには残さない）

`unique_check_elim` の `leaf_is_unique` の結果を常に `true` に差し替え、共有された配列への
`Array::set` を develop mode で走らせる。

| | 結果 |
|---|---|
| 嘘の証明を注入 | `A value proven uniquely owned was reached while shared.` で abort |
| 注入なし（対照） | プログラムは正常終了し、abort を期待するプローブが落ちる |

対照側を測らないと「注入が原因で鳴った」とは言えないので、両方を走らせる。

### 検査が経路に届いていることの確認

比較 `EQ` を `NE` に反転させると、配列テストが軒並み落ちる
（`test_array_bounds_check::test_set` は期待する "Index out of range" ではなく abort する）。
issue が挙げた「反転してもスイートは緑のまま」は、検査が 2 経路から 18 経路に増えたことで
解消している。**既存の 1207 件が、検査そのものの正しさを毎回検証する形になった。**

### スイートとコスト

`-O none` 1207 件、`-O max` 1207 件、いずれも 0 失敗。所要は none 468 秒 / max 402 秒で、
これまでの範囲内にある。検査は develop mode だけなので released build には出ない。

## 恒久テストを置かない理由

「証明が通った経路に検査が出ている」ことを IR で固定する案は退けた。develop mode のビルドは
インプロセスでしか起きず、オブジェクトキャッシュがカレントディレクトリ（全テスト共有）に載る。
同じソースの 2 回目のビルドはコード生成ごと飛ぶので、IR ファイルが出ない。テスト側では直せず、
develop mode を CLI から立てられるようにするのは公開仕様の変更になる。

代わりの証拠は上の 2 つ、注入と対照の組、および反転が経路に届いていることの確認である。

## 残る改善案（この変更には含めない）

検査の呼び出しを op から取り上げ、inline-LLVM op のコード生成を行う唯一の地点で、
`unique_check_operand` の鏡（証明が通ったときに操作対象を返す宣言）を読んで出す形にすれば、
**新しい op が検査を書き忘れること自体が起きなくなる**。今回は各 op が検査を持つ形に留める。
