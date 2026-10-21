# `RcState::Global` の健全性（ラフな証明）

対象: fixlang の branch `global-field-read-rc` の commit `aac039931`。worktree は `/home/maruyama/fixlang-wt/global-field-read-rc`。
変更前の commit は `fa7fec873`。変更の全体は `git diff fa7fec873 aac039931 -- src/` で見られる。

## 主定理

変更後のコンパイラが生成するプログラムの実行は、変更前のコンパイラが生成するプログラムの実行と、
参照カウントの操作を除いて同じ振る舞いをする。具体的には次の 2 つが成り立つ。

- **T1.** `RcState::Global` を付けた箇所が、変更前のコードでカウントを増減していたオブジェクトは、実行のその時点で状態が `GLOBAL` である。
- **T2.** 状態が `GLOBAL` のオブジェクトに対しては、変更前のコードの増減（`RcState::Unknown` の分岐）は何もしない。したがって、その操作を生成しなくても振る舞いは変わらない。

ここで言う「振る舞い」には、出力、終了状態、どのオブジェクトがいつ解放されるかが含まれる。

## 仮定

- **A1（unsafe の契約）.** 利用者のコードは、文書化された unsafe の契約を守る。次の 2 つが該当する。
  - `borrow_boxed` や `Array::borrow_elements` で借りたポインタを通して、オブジェクトを書き換えない。
  - `boxed_from_retained_ptr` には、本当にその型のオブジェクトを指すポインタを渡す。
- **A2（std の不変条件）.** std は、自分が渡した内部ポインタを通して Fix のオブジェクトの参照を書き込まない。根拠は `dev-docs/2026-07-30-rc-state-inference/design.md` の「発生源」の節で、locality 推論の既存の前提と同じものである。
- **A3（名前の一意性）.** RC IR の 1 つの関数本体の中で、局所変数の名前（`FullName`）は一意である。
  - `src/rc_ir/provenance.rs` の `Interp::record` が、どのビルドでも assert する（`is bound twice in one function`）。
  - この解析は `unique_check_elim::specialize` の冒頭で必ず走り、locality 推論の直前に当たる。その間の複製（`fresh_rename_function`）は、名前を 1 対 1 で付け替える。
  - グローバル値の名前と局所変数の名前は、`FullName::is_global` で区別できる。コード生成の `get_scoped_value` も `is_local` で分岐するので、グローバル値の名前は必ずアクセサを通って読まれる。関数ポインタのグローバル値は例外だが、boxed leaf を持たない。

## L1. 初期化の直後、グローバル値から到達できるオブジェクトはすべて `GLOBAL`

`src/rc_ir/codegen.rs` の `implement_rc_global` は、初期化子の値を評価し終えたあと、その値に `mark_global` を掛けてから、値を格納してフラグを立てる。

- `mark_global` は `Generator::build_traverser_work` に `TraverserWorkType::mark_global()` を渡す（`src/generator.rs`）。ボックス型のオブジェクトについては `build_mark_boxed_with` に入る。
  - そのオブジェクトの状態を `GLOBAL` にする。
  - 次に、型の traverser で子をたどる（`traverse_boxed_refs`）。
  - 状態がすでに `GLOBAL` 以上のオブジェクトでは止まる。そのようなオブジェクトは、以前の `mark_global` で、到達できるものまで含めて印が付いている。帰納法の仮定に当たり、L2 と合わせて閉じる。
- traverser がたどる子は、release の traverser がたどる子と同じである。ここに含まれるものは次のとおり。
  - 配列の `[0, size)` の要素
  - クロージャのキャプチャ（動的オブジェクトの traverser の `TRAVERSER_WORK_MARK_GLOBAL` の場合分け）
  - `Destructor` の `_value` と `_dtor`
  - ユニオンの、現に保持しているバリアント

  配列の `size` より先は値の一部ではなく、punch された穴はまだ値を保持していない。
- 定数として静的データに置いたストレージ（`add_global_array_storage` など）は、状態 `GLOBAL` で置かれ、要素は全部アンボックス型である（同じ関数が assert する）。

## L2. `GLOBAL` のオブジェクトは `GLOBAL` のままで、子も変わらない

**状態のバイトを書く場所。** `set_refcnt_state` の呼び出し元は、`create_obj`、`build_mark_boxed_with`（`mark_global` と `mark_threaded` が共有する）、`unique_threaded_bb` の 3 か所である。ほかに、状態を変えずに書く場所が 2 つある。定数のストレージ（`add_global_array_storage`）は `GLOBAL` で置かれ、要素はアンボックス型だけである。`realloc_array` は control block ごと移すが、一意な配列にだけ働く。それぞれの働きは次のとおり。

- `create_obj` は `LOCAL` で初期化する。これは新しいオブジェクトにだけ働く。
- `mark_global` は `GLOBAL` にする。
- `mark_threaded` は、状態が `THREADED` 以上のオブジェクトでは止まるので、`GLOBAL` を変えない。
- `build_branch_by_is_unique` の `unique_threaded_bb` は、`THREADED` から `LOCAL` に戻す。この場合分けに入るのは `THREADED` のオブジェクトだけで、`GLOBAL` のオブジェクトは `global_bb` から `shared_bb` へ行く。

**フィールドや要素を書き換える場所。** その場で書き換える op は、どれも次のどちらかを経る。

- `build_branch_by_is_unique` の一意性の検査。`GLOBAL` は共有の側へ行き、複製が作られる。
- 呼び出し側が一意性を保証する `assuming_unique` の版。この保証は `unique_check_elim` の証明から来る。provenance はグローバル値の atom を `Unknown` とするので（`prov_of`）、グローバル値に由来するオブジェクトが一意と証明されることはない。

`RcState::Local` の一意性の検査は状態を読まず、カウントだけを比べる。ヒープ上の `GLOBAL` のオブジェクトは印を付けた時点のカウントを保つので、Local の検査が当たれば一意と判定されてしまう。これが起きないのは、グローバル値の leaf がどの入力のもとでも `MayExt` に解決されるからである（`LeafCond::global` は `root` と `deep` を `Always` にする）。グローバル値に由来する leaf に Local が付かないことは、変更前からある locality 推論の健全性に依る。

生のポインタを通した書き換えは、A1 と A2 で除かれる。`mutate_boxed` と `mutate_elements` は、先に force-unique で一意にしてから書く。

**解放。** `GLOBAL` のオブジェクトは、`build_release_boxed_with` の `global_bb` に入るので、解放されない。

**帰結（G）.** `GLOBAL` のオブジェクトから到達できるオブジェクトは、すべて `GLOBAL` である。到達できる集合は印を付けた時点で決まり（L1）、その後は変わらない（L2）。

## L3. グローバル値の名前を評価すると、その値のどの boxed leaf も `GLOBAL` のオブジェクトを指す

**単一スレッドのビルド。** グローバル値の名前を評価すると、アクセサ（`Get#...`）を通る。

- アクセサは初期化フラグを確かめ、未初期化なら初期化子を走らせる。そのあと `mark_global`、`GlobalVar` への格納、フラグの設定の順で進む（L1）。
- 初期化子の中で自分自身を読むと、フラグが立つ前にもう一度初期化子に入るので、止まらない。止まらない実行には、主張すべきことがない。

**threaded のビルド。** `pthread_once` の中で同じ順に進むので、戻ってきた時点で印が付いている。

**静的に置いた定数。** グローバル値が数の配列リテラルや文字列リテラルのときは、その名前が定数に置き換わる。この定数は L1 の静的データで、状態は `GLOBAL` である。

以上と G から、読み出した値のどの leaf も `GLOBAL` である。

## L4. 解析の不変条件

`Walk`（`src/rc_ir/locality.rs`）が持つ環境 `env` について、次が成り立つ。

> 関数本体の束縛 `x` と、その leaf の path `p` について、`shape_of(x).leaf_at(p).global` が真なら、
> 実行中に `x` が束縛される値の path `p` にあるオブジェクトは（その leaf が現に値を持つなら）`GLOBAL` である。

walk の各規則についての帰納法で示す。`global` を真にする構成子は `LeafCond::global` だけで、それを引き継ぐのは `take_out_of` と `join` だけである。ほかの構成子（`new`、`bottom`、`always`、`input_leaf`、`substitute`）は、どれも `global` を偽にする。

- **グローバル値の名前**（`shape_of` が `ExtShape::global` を返す）: L3 から成り立つ。
- **`RcRhs::Var(y)`**: 値は `y` と同じなので、`y` についての仮定から成り立つ。
- **アンボックス型のコンテナの `Destructure` と、アンボックス型のユニオンの `Match` の射影**（`project`）: 取り出した値の leaf は、コンテナの対応する leaf と同じオブジェクトを指す。
- **ボックス型のコンテナの `Destructure`、ボックス型のユニオンの `Match` のバリアントのアーム**（`take_out_of`）: 取り出した値は、コンテナのオブジェクトから到達できるので、G から `GLOBAL` である。
- **`_` などで残りすべての場合を受けるアーム**: 場合分けする値（scrutinee）そのものなので、仮定から成り立つ。
- **合流**（`walk_match` の `join`）: `global` が真なのは両側が真のときだけで、値はどちらかの側の値なので、成り立つ。
- **op の結果、呼び出しの結果、クロージャの生成、引数**: `global` は偽なので、主張することがない。

A3 から、`env` の 1 つの名前は 1 つの束縛だけを指す。`Mode::Clone` の walk も、`Mode::Survey` の walk と同じ規則で `env` を作る。

## T1 の証明

注釈が付く箇所は 4 種類ある。

- **`annotate_rc_site`**: path の下にある leaf がすべて `global` であり、少なくとも 1 つある（assert で確かめる）。この `Retain` / `Release` が数えるのは、この leaf のオブジェクトだけである。アンボックス型のユニオンでは、現に保持しているバリアントの leaf だけを数える。L4 から、それらは `GLOBAL` である。
- **`annotate_take_out` を経る `Destructure`**: コンテナの根の leaf が `global` である。`get_struct_fields` は、取り出した各フィールドを retain し、コンテナを release する。フィールドはコンテナから到達できるので、L4 と G から、両方とも `GLOBAL` である。
- **`annotate_take_out` を経る `Match` のアーム**: retain するのは、取り出したペイロードの leaf で、scrutinee から到達できる。上と同じ理由で `GLOBAL` である。
- **`get_scoped_obj`**: アンボックス型のグローバル値の部分オブジェクトを retain する。L3 から `GLOBAL` である。この箇所は解析を経ないので、`--threaded` のビルドや `-O none` でも成り立つ。

## T2 の証明（コード生成）

**変更前の注釈。** 変更前、T1 の 4 種類の箇所は同じ leaf について `Unknown` だった。グローバル値の形は `ExtShape::always` で、取り出した値は `take_out_of(always)` なので、`resolve` は `MayExt` になる。`get_scoped_obj` は `RcState::Unknown` を直接渡していた。変更は `Unknown` を `Global` に置き換えるだけで、`Local` の注釈は増えていない。

**`Global` がカウントを省くのは、注釈された箇所のオブジェクトに限られること。** `Global` の状態は、ヘルパー関数（`retain` / `release`、`_global` の接尾辞）と、アンボックス型の値の traverser（`create_traverser(..., state)`）を通る。いずれも、その値のアンボックス型の構造の中の leaf ごとに `retain_nonnull_boxed` か `build_release_boxed_with` を呼ぶ。この 2 つの関数は、`Global` なら develop mode の assert だけを生成して return する。

ボックス型のオブジェクトの子に進む `traverse_boxed_refs` は、状態を `Unknown` にしてから渡す。そのうえ、親が `Global` なら、そこへは到達しない。したがって、`Global` がカウントを省くのは T1 のオブジェクトに限られる。

**変更前のコードとの比較。** 変更前は、同じオブジェクトに `RcState::Unknown` で次の分岐を生成していた。

- `retain_nonnull_boxed`: `build_branch_by_refcnt_state` で分岐し、`global_bb` では何もせずに `cont_bb` へ行く。
- `build_release_boxed_with`: 同じく、`global_bb` では何もせずに `end_release_bb` へ行く。

threaded のビルドでも、`global_bb` は同じである。T1 から、実行時には常に `global_bb` を通るので、生成しなくても状態もカウントも変わらない。develop mode の assert は状態を読むだけで、`GLOBAL` なら何もしない。

**`RcState::dispatches` と `name_suffix`.** `dispatches` は `Global` について unreachable である。上の 2 つの関数が先に return するので、`Global` で呼ばれることはない。`name_suffix` は、`Global` に別の名前を与えるので、ほかの状態のヘルパー関数と取り違えることはない。

## 自分で分かっている穴

- **L1 の「traverser がたどる子が値のすべての参照である」**: これは型ごとの traverser の実装に依存する。ここでは網羅していない。
- **L2 の「その場で書き換える op はどれも一意性の検査か保証を経る」**: これは op ごとの宣言（P5）に依存する。ここでは網羅していない。
- **threaded のビルドでの競合**: 初期化子が `unsafe_perform` で他のスレッドと値を共有すると、`mark_global` の atomic でない書き込みが、他のスレッドの `THREADED` の操作と競合しうる。読み手のスレッドは `pthread_once` の happens-before で `GLOBAL` を見る。他のスレッドで起きうるのは解放漏れまでで、反例は組めていない。この競合は変更前からある。

## レビュー

文脈を渡さない別のエージェントが、各段を引いたコードと突き合わせた。期間は 1 周に限った。

- **反例:** 0 件
- **偽:** 1 件。A3 は release のビルドでは検査されない、と書いていたが、実際は検査されていた。直した。
- **穴:** 7 件。上の「穴」の節に残したものと、本文に補ったものがある。

レビュー役は、`act_xs` で punch した値をクロージャに持つグローバル値と、`Array::act` を経たグローバル値のプログラムも作り、ハーネスの 10 構成で確かめた。結果は異常なしだった。
