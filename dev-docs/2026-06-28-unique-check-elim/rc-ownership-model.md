# RC IR の所有権・消費モデル

RC IR 上で「どの構文がどの参照を消費するか」の仕様。実装は `src/rc_ir/ownership.rs`。RC 挿入・borrow 化・
相殺・特殊化がこのモデルを共有する。validator の参照収支検査も同じモデルの上にあり、これは未マージで
`rc-ir-validator.md` の「参照カウントの検査（未マージ）」が場所を示す。

## 1. 単位

- **RC unit** = `(変数名, Path)`（`VarPath`）。`rc_units` が列挙し、`is_rc_unit_root`（boxed / union / punched array）で降下を止める。
  punched フィールドはスキップする。
- **boxed leaf** = `boxed_leaf_paths` が列挙する末端。unbox union は**各 variant の中まで降り**、punched フィールドも
  スキップしない。
- 消費と provenance は **leaf 空間**、`Retain`/`Release` ノードは **unit 空間**に住む。橋渡しは
  `truncate_to_unit`（leaf path を unit path に切り詰める）と `units_under`。
- 参照数を数える鍵 `unit_key`（`origin` の identity を `truncate_to_unit` で切り詰めたもの）が答える path は、
  根の型の `rc_units` の要素である。切り詰めは下向きにしか効かないので、identity の path が unit root より
  上で止まるとどの unit も指さない鍵ができる。`unit_of` の assertion がこれを検査する。
- `origin` は leaf でない path も受ける。unit path は leaf とは限らない（unbox union の unit は根、その leaf は
  各 variant の中。punched array の unit は `[i]`、その leaf は `[i, 0]`）。`result_prov` に該当する leaf が
  無いときは、**その下の leaf 群がどのオペランドの unit から射影されたか**で答える（`origin_from_leaves_under`）。
  下の leaf が 1 つのオペランド unit に揃えば `Exactly`、複数に分かれるか射影でない leaf を含むなら、その値が
  取りうるオブジェクトを全部並べた `Join`。⊥（不在 variant）の leaf は参照を持たないので読み飛ばす。

## 2. 消費する構文と読むだけの構文

権威は `ownership.rs` の `collect_consumes_go` / `rhs_consumes` / `destructure_consumes`。

**消費する（leaf 粒度）**

| 構文 | 消費されるもの |
|---|---|
| `Ret(x)` | `x` の全 boxed leaf |
| `App(callee, args)` | callee の全 boxed leaf（closure の捕捉）＋ callee のその位置が `Own` である引数 leaf |
| `Closure(f, caps)` | 各 capture の全 boxed leaf |
| `Llvm(gen, args)` | `borrows_operand(i)` が偽のオペランドのうち、`result_prov` が `Arg(i, π)` として素通しを宣言していない leaf |
| `Destructure`（boxed コンテナ） | コンテナの全 boxed leaf（明示 `Release` ノードは無く、codegen の `get_struct_fields` が release する） |
| `Destructure`（unbox コンテナ） | **名前が付いていないフィールド**の leaf のみ（名前付きフィールドは move＝別名） |
| `Match`（boxed union の variant アーム） | scrutinee コンテナ。ただし RC 挿入が**明示 `Release` ノード**を各アーム先頭に置く |
| `Release(v, π)` | その unit 1 個 |

**読むだけ（消費しない）**

- `Var(y)`（move-bind）= 別名。codegen でも同一 LLVM 値。
- `Match` ノード自体（消費は各アームの中）。
- `Retain`/`Release` ノードに現れる変数。
- `borrows_operand(i)` が真のオペランド（read getter 群）。
- `result_prov` が `Arg(i, π)` の leaf（素通し）。

**ノードに現れない暗黙の RC**（検査を書くときに必ず要る）

- boxed コンテナの `Destructure`: コンテナを release し、各名前付きフィールドを retain する。
- boxed union の variant アーム: payload を retain する。
- unbox コンテナの `Destructure` / unbox union・catch-all の payload: **別名**であって新しい参照ではない。

## 3. 別名関係（`origin`）

`origin(vars, type_env, var, path)` が別名辺を遡り、参照を生んだ変数と path を返す。全域かつ決定的。
辺は次のとおり:

- `Binding::Move(y)` -> `y`。
- `Binding::Field(container, idx)`: **unbox コンテナのときだけ**別名（boxed は retain するので producer）。
- `Binding::Payload(scrut, variant)`: catch-all は scrutinee そのもの、**unbox union の variant** は別名、
  **boxed union の variant** は producer。
- `Binding::Llvm`: `result_prov` の leaf が単一の `Arg(j, p)` なら引数 `j` の別名。leaf でない path は
  その下の leaf 群から決める（`origin_from_leaves_under`）。unbox union の構築
  （`InlineLLVMMakeUnionBody`）もこの規則で解ける。whole-union path の下の leaf は、構築された variant
  では payload の leaf、他の variant では ⊥ なので、答えは payload の unit 群になる。
- `Binding::Param` / `Binding::Producer` はそこで止まる。

## 4. ステージごとの不変条件

| ステージ | 不変条件 |
|---|---|
| `insert_rc` 直後 | 全 `Retain`/`Release` は path `[]`・`RcState::Unknown`。全パラメータ/capture が `Own`。各 binding の各参照はどのパスでもちょうど 1 回消費される |
| `split_rc_units` 直後 | 同上、ただしキーが `(binding, unit)`。全 RC ノードの path が `rc_units(v.ty)` の要素 |
| `borrow_ify` 直後 | `borrowed_units` に載る unit と、それに根を持つ値は**消費も RC 操作もされない**（`owns_unit` が判定）。それ以外は各パスちょうど 1 回消費 |
| `cancel` / `specialize` 直後 | **binding 単位の線形性は失われる**。`cancel` は `unit_key`（`origin` の identity を `truncate_to_unit` で切り詰めたもの）のキーで別 binding をまたいで retain/release を対消滅させる。成立するのは **(root オブジェクト, unit) 単位の参照数保存**: 所有パラメータ/capture unit を 1、borrowed を 0 で初期化し、producer で +1・消費/`Release` で -1 したカウンタが、どのパスでも負にならず関数出口で 0 |

`specialize` は RC ノードを素通しコピーし、`assuming_unique` は `LLVMGen` を差し替えるだけなので、消費モデルは
特殊化の前後で不変（`result_prov` を force-unique 有無で変える op は存在しない）。

## 5. 参照収支検査の注意

この節は未マージの参照収支検査の仕様である（場所は `rc-ir-validator.md`）。

関数本体は木（分岐は `Match` のアームのみ、ループは呼び出しでしか作れない）なので、不動点計算は不要で
1 パスの木走査＋アーム出口の一致検査で全パスを尽くせる。キーは `cancel` と同じ `unit_key`
（`origin` の identity を `truncate_to_unit` で切り詰めたもの）を使う。**per-binding のトークン照合は
`cancel` 後に必ず破綻する**。

偽陽性を避けるために除外・特別扱いが要るもの:

- **root がグローバル名の unit**: グローバルは線形規律の外（読むたびに新しい参照が生まれ、refcount 操作は
  no-op）。丸ごとスキップする。
- **punched フィールドの下の leaf**: `rc_units` はスキップするが `boxed_leaf_paths` はしない。leaf 側でも
  ミラーして除外する。
- **unbox union が参照を数えるキー**: union の参照は生きている variant の参照で、どのオブジェクトに属するかは
  タグで決まる。`origin` は union の根に対して payload の unit 群を候補に持つ `Join` を答え、variant の中の
  leaf に対してはその payload の unit を答える。union の根の鍵と、その下の leaf が解決する鍵の両方を検査
  対象外にする。
- **fully-unboxed 値**（`needs_rc` が偽）には RC ノードが無い。funptr 型もここに入る。
- **1 変数が複数 unit を持ち、unit ごとに所有権が違う**（`split_rc_units` 以降）。
- **他コンパイル単位のシンボル**: `prog.funcs` に無い callee は全 `Own` とみなす（borrow 最適化が走るのは
  単一ユニット時のみなので、実際に曖昧になることはない）。
- **同じ変数を複数オペランドに渡す**（`MakeStruct(a, a)` 等）: 位置ごとに消費が積まれる。

検査は「宣言されたモデルへの適合」を見るのであって、`borrows_operand` / `result_prov` の宣言が実装と
一致しているかは見ない。宣言と実装の乖離は別の手段（テスト・valgrind）で捕まえる。
