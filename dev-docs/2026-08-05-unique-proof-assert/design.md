# 一意性証明の開発モード検査 (#198)

## 1. この検査が捕まえるもの

RC IR の最適化は、ある値について「ここでは他に保持者がいない」と証明できたとき、その値を書き換える
操作から**クローンを落とす**。落とすのは、実行時に参照カウントを読んで 1 かどうかを見る検査である。

証明が誤っていた場合に起きることは、実行時エラーでもクラッシュでもない。他の保持者から見える値を
その場で上書きし、プログラムはそのまま走り続ける。壊れた値が読まれるのは別の関数、別のフェーズ、
別のスレッドかもしれない。**症状が原因から任意に離れる**という点で、このコンパイラが起こしうる
最悪の壊れ方に属する。

落とした検査そのものが、その誤りを捕まえる唯一の観測である。だから開発モードでは同じ観測をもう一度
行い、違反を**書き込みの地点で**止める。これがこの変更の対象である。

## 2. 証明はどこから来るか

### 2.1 解析による証明

`src/rc_ir/unique_check_elim.rs` の `specialize` が、provenance 解析の結果を `leaf_is_unique` に
問い、真なら `LLVMGen::assuming_unique()` を呼ぶ。`assuming_unique()` は op のフラグを書き換える。

- クローンを出す 16 op: `force_unique` を **false** にする
- 検査の結果を定数に畳む 2 op (`is_unique`, `Array::is_storage_unique`): `assume_unique` を
  **true** にする

同じ事実が逆極性の 2 つのフィールド名で保持されている。証明を受け取る op はこの 18 個で、
issue が数えた 16 個は前者だけである。**後者では誤った証明が「プログラムに嘘の答えを返す」形で
出る**ので、対象に含める。

この pass が走るのは `Configuration::enable_borrow_optimization()` が真のとき、すなわち
`-O max` 以上に限られる。

### 2.2 解析が global を unique と証明しないこと

`src/rc_ir/provenance.rs` の `LeafOrigin::Unknown` の定義は「boxed コンテナから読んだ値、
**global**、`Retain` で複製された値」であり、`resolve_leaf` はこれを `SharingVerdict::Dynamic` に
落とす。`leaf_is_unique` が真を返すのは `Unique` のときだけなので、**解析は global を unique と
証明しない**。証明が global を指したなら、それは解析の誤りである。この不変条件は検査の global 腕
（下記 4.2）が乗っているものなので、テストで固定する。

### 2.3 登録による証明

`force_unique == false` は解析以外からも来る。`src/ast/program.rs` は全 struct について
`#punch_` / `#plug_in_` を `force_unique: false` で登録する。`act_x` はこれを使って、
一意性を確かめたあとにフィールドを出し入れする。この経路の証明は解析とは無関係に立つ。

検査はどちらの由来でも同じく立つ。ただし unboxed struct が到達しうるのはこの経路だけであり、
そこには参照カウントが無いので検査の対象にならない（下記 5.3）。

## 3. 参照カウントのモデル

boxed オブジェクトの制御ブロックは 2 つのフィールドを持つ。

| | 内容 |
|---|---|
| `CTRL_BLK_REFCNT_IDX` | 参照カウント |
| `CTRL_BLK_REFCNT_STATE_IDX` | state バイト |

state は `REFCNT_STATE_LOCAL` / `REFCNT_STATE_THREADED` / `REFCNT_STATE_GLOBAL` の 3 値である。
**「unique か」はカウント単独では答えられない。**

- **global**: `Generator::mark_global_one` は state を書くだけでカウントに触らない。global な
  オブジェクトのカウントは `create_obj` が入れた 1 のままなので、カウントだけを見れば unique に
  見える。実際にはプログラムが走る限り共有されている。
- **threaded**: カウントは他スレッドが原子的に更新する。素の load はそれ自体がデータ競合になる。

本来の検査 `Generator::build_branch_by_is_unique` はこの構造どおりに答える。まず
`build_branch_by_refcnt_state` で state を読んで 3 分岐し、

- **local**: 素の load でカウントを読み、1 と比較
- **threaded**: **acquire の** atomic load で同上
- **global**: カウントを読まず、無条件に shared 側へ

acquire を load 自身に置くことには理由がある。ThreadSanitizer は standalone の `fence acquire`
から happens-before を引かないので、acquire を fence に移すとコードは正しいまま**レース検出器が
後続の書き込みを誤報する**。

## 4. 変更前の検査が持っていた穴

PR #196 が入れた `assert_proven_unique` は、**カウントだけを読んでいた**。issue が挙げた 3 点は、
この 1 つの設計の誤りの帰結として並ぶ。

1. **global の誤証明を素通しする**。global のカウントは 1 なので、カウントだけを見る検査は
   「プログラム全体で共有される値を unique と誤証明する」という最悪のケースを通す。
2. **threaded でカウントを非アトミックに読む**。検査自身がデータ競合になる。
3. **18 の op のうち 2 つにしか出ない**。`force_unique_boxed` を通る `_mutate_boxed_internal` と
   その `_ios` 版だけで、`specialize` が実際に証明する主戦場である配列の書き込み 11 op には
   1 つも届いていない。

したがって対処も 3 つ別々ではない。**本来の検査と同じ機構を使う**ことにすれば 1 と 2 は設計の
帰結として閉じ、残るのは 3 の配線だけになる。

## 5. 終状態

### 5.1 `Generator::build_assert_unique`

`build_assert_refcnt_state_local` の隣に置く。

```rust
pub fn build_assert_unique(&mut self, obj_ptr: PointerValue<'c>)
```

`build_branch_by_is_unique` と同じ骨格で、shared 側を abort にしたもの。

- `develop_mode` でなければ何も出さない
- `build_branch_by_refcnt_state(obj_ptr, RcState::Unknown)` で local / threaded / global を得る
- local: 素の load、threaded: acquire の atomic load、global: 無条件に abort
- **state は書き換えない**。`build_branch_by_is_unique` は threaded かつ unique の側で
  `mark_local_one` を呼ぶが、検査が守っている操作はその書き換えを持たない。開発モードのビルド
  だけが実行時 state を変えれば、そこから先の RC の振る舞いが released build と分岐する。

カウントを読んで 1 と比べる列は、本来の検査と合わせて 4 箇所に現れる。これは
`build_is_refcnt_one(obj_ptr, acquire, name_suffix)` に抽出し、acquire を load に置く理由も
そこ 1 箇所に書く。

### 5.2 state を実行時に読む（宣言の契約）

`build_assert_unique` は `RcState` を引数に取らない。理由は locality 推論の契約にある。

op は自分が出す一意性検査を `LLVMGen::unique_check_operand` で宣言する。`assuming_local` の doc は
この宣言について次を定めている。

> A `generate` that emits a check or a reference count it does not declare leaves that one
> reading the state, so the declarations stay honest about what the annotation covers.

すなわち **宣言していない検査は state を読み続ける**。ところが 18 op すべての
`unique_check_operand` は

```rust
if !self.force_unique { return None; }   // あるいは if self.assume_unique { return None; }
```

で始まる。**証明が受理された瞬間に宣言が取り下げられる**。そして検査が立つのはまさにその地点である。
よって、取り下げられた宣言に乗った locality 注釈は、この地点の対象について何も言っていない。
`assumed_state(self.assume_local)` を渡すと、誰も証明していない locality を根拠に
`build_assert_refcnt_state_local` が出て、正しい一意性証明に対して locality のメッセージで
誤 abort しうる。

コストは開発モードでの state load 1 回である。なお threaded ビルドでは
`locality::specialize` 自体が走らない（`build_object_files.rs` が `!config.threaded` で
ガードしている）ので、そこでは `assume_local` はそもそも立たない。

### 5.3 証明でクローンを落とす判断を集める

16 の `generate` に散っていた `if self.force_unique { make_X_unique(..) } else { .. }` を
`force_unique_or_assert` に載せ替える。この関数は値の形で分岐する。

| 値の形 | `force_unique` | `!force_unique` |
|---|---|---|
| 配列 | `make_array_unique_with_hole` | storage を対象に検査（カウントは storage が持つ） |
| boxed struct / union | `make_struct_union_unique` | 値そのものを対象に検査 |
| unboxed struct / union | そのまま返す | そのまま返す（カウントが無い） |

unboxed が到達するのは 2.3 の登録経路だけである。ここに配列や `_mutate_boxed*` の値が来ることは
無い（前者は先に分岐し、後者は `[a : Boxed]` 束縛を持つ）ので、unboxed 側では struct か union で
あることを表明する。

### 5.4 一般の入口に載らない 3 つ

- `ArraySetCapacityBoundsUnchecked`: unique 側が `realloc` による in-place の縮小・拡大で、
  「unique にした値を返す」形をしていない。早期 return の先頭で検査を直接呼ぶ。
- `PunchedArrayPlugBody`: クローンが穴を残す必要があるので、hole を取る変種を使う。
- `is_unique` / `Array::is_storage_unique`: 値を作らずフラグを定数 `true` に畳むので、
  その地点で検査を直接呼ぶ。

## 6. 何が測れて何が測れないか

### 6.1 測れないもの

**18 箇所の検査をすべて削除しても、テストスイートは緑のままである。** これは違反を強制できない
assertion 一般の性質であって、この検査に固有の弱さではない。テストで固定するには、誤った証明を
作る口をコンパイラ側に用意するしかなく、それは production コードをテストのために歪めることになる。

IR を読んで「証明が通った経路に検査が出ている」ことを固定する案も検討したが、使えない。
develop mode のビルドはインプロセスでしか起きず（`Configuration::develop_mode()` を呼ぶのは
テストだけで、CLI から立てる手段が無い）、オブジェクトキャッシュがカレントディレクトリ
（全テスト共有）に載る。**同じソースの 2 回目のビルドはコード生成ごと飛んで `.ll` が出ない。**
`fix_build_source_command` を使う既存の IR テストは `current_dir(temp_dir)` でキャッシュごと
隔離しているのでこの罠を踏まない。これは #197 と同じ土俵の問題である。

恒久的な歯止めにするなら 8 の構造化が要る。

### 6.2 測れるもの

- **検査が誤った証明に反応すること**（注入と対照の組、7.1）
- **検査が経路に届いていること**（比較の反転、7.2）
- **証明が受理される地点が実在すること**。RC IR ダンプの `[unique]` マーカで固定する。
  検査そのものではなく「検査が立つ地点」を固定するもので、将来 provenance/specialize の変更で
  その op が証明を受けなくなれば落ちる。

## 7. 検証

### 7.1 検査が発火することの証明

`unique_check_elim` の `leaf_is_unique` の結果を常に `true` に差し替え、共有された配列への
`Array::set` を develop mode で走らせる。

| | 結果 |
|---|---|
| 嘘の証明を注入 | `A value proven uniquely owned was reached while shared.` で abort |
| 注入なし（対照） | プログラムは正常終了し、abort を期待するプローブが落ちる |

対照側を測らないと「注入が原因で鳴った」とは言えないので、両方を走らせる。

### 7.2 検査が経路に届いていることの確認

比較 `EQ` を `NE` に反転させると、配列テストが軒並み落ちる
（`test_array_bounds_check::test_set` は期待する "Index out of range" ではなく abort する）。
issue が挙げた「反転してもスイートは緑のまま」は、検査が 2 経路から 18 経路に増えたことで解消した。
既存の 1209 件が、検査そのものの正しさを毎回検証する形になっている。

### 7.3 追加したテスト

- `test_unique_check_elim_local_fresh` に 5 つのマーカ。`struct_punch_0` / `struct_plug_in_0` /
  `is_unique` / `mutate_boxed` / `mutate_boxed_ios` は、証明が受理されることを固定するものが
  何も無かった。追加した Fix コードを外すとテストが落ちることを確認済み。
- `test_global_value_keeps_its_check`（新ケース `unique_elim_global`）。2.2 の不変条件を固定する。
- `test_threaded_build_checks_its_uniqueness_proofs`。develop mode と `--threaded` が同時に立つ
  唯一の構成で、state 分岐の threaded 腕がコード生成され verifier を通る。実行が取るのは
  local 腕である（threaded 腕だけを反転しても落ちないことで確認した）。

### 7.4 スイートと影響範囲

`-O none` 1209 件 / `-O max` 1209 件、いずれも 0 失敗。所要は none 471 秒 / max 401 秒で、
これまでの範囲内にある。

**develop mode 以外への影響が無いことを実測した。** `origin/main` (`d41046e5`) のコンパイラと
本ブランチのコンパイラで `examples/` の 17 プログラムを `-O none` / `-O basic` / `-O max` で
コンパイルし、モジュールハッシュを正規化した LLVM IR を突き合わせて **51/51 が一致**
（計 290 モジュール）。比較器が差を報告できることは、同一プログラムを 2 つの最適化レベルで
ビルドして確かめた。51 対すべての IR に `is_unique` が現れるので、コーパスは 5.1 で抽出した
`build_is_refcnt_one`（released build にも出る経路）に到達している。

## 8. 残る設計課題

- **19 個目の op が検査を書き忘れる余地**。検査の呼び出しを op から取り上げ、inline-LLVM op の
  コード生成を行う唯一の地点で `unique_check_operand` の鏡（証明が受理されたときに検査対象を
  返す宣言）を読んで出す形にすれば、書き忘れ自体が起きなくなる。boxed leaf を `FieldPath` で
  辿ってポインタを得るヘルパが要る（配列は storage、それ以外は値そのもの、という
  `force_unique_or_assert` が今は手で書いている区別）。
- **`force_unique` と `assume_unique` が同じ事実を逆極性で持っている**（2.1）。読み手は op を
  移るたびに極性を反転させる必要がある。
- `get_array_storage(gc, x).value(gc).into_pointer_value()` が 3 箇所にある。`object.rs` の
  `get_array_storage_buf` の隣に `get_array_storage_ptr` が要る。
- `_mutate_boxed_internal` / `_ios` と `mutate_elements` / `_ios` の 4 つの op 本体が同じ核を
  繰り返している。
- `src/fixstd/builtin.rs` が 8400 行で、配列 / struct・union / FFI の builtin が同居している。
