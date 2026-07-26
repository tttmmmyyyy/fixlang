# RC IR validator

RC IR の well-formedness を静的に検査する、コンパイラ開発時専用のチェッカ。実装は `src/rc_ir/validate.rs`。
消費モデルの仕様は `rc-ownership-model.md` を参照する（この文書はそれを重複させない）。

## 位置づけ

valgrind と uniqueness assert は「その入力でその経路を踏んで初めて」壊れる動的な網である。validator は
静的・全域で、**RC を書き換えたパスの直後**に走るので、壊した張本人のパスを名指しできる。両者は補完関係にある。

検査が失敗したら panic する。RC IR が malformed であることはコンパイラの内部エラーで、そのまま進めば
誤ったバイナリを吐くため、続行より停止が正しい。

## 走る場所と gate

`optimize_rc_program`（`src/build/build_object_files.rs`）が各パスの直後に呼ぶ:
insert_rc / split_rc_units / borrow_ify / cancel / specialize。

`config.develop_mode` gate の中にあり、`develop_mode` を true にするのは `Configuration::develop_mode()`
だけである。CLI 引数のパス（`Configuration::new` / `release_mode`）からは到達しないので、通常の `fix`
ビルドでは走らず、検査用の `symbol_names` 収集コストも掛からない。

`symbol_names` は**全プログラム**のシンボル名で、`optimize_rc_program` が受け取って渡す。RcProgram は
コンパイル単位ごとなので、他の単位が定義するグローバルへの参照（funptr atom・グローバルオペランド）を
「未束縛」と誤判定しないためにこれが要る。ローカル名は大域一意な fresh 名なので、シンボル名を許しても
ぶら下がったローカルを見逃すことはない。

外部プロジェクト（fixlang_minilib / project_euler 等）で試すときは、`optimize_rc_program` の gate を
一時的に無条件 true にしてビルドする。**この一時パッチはコミットしない。**

## 構造の検査

関数内で束縛名が一意（シャドー禁止）であること、各変数使用がスコープ内の束縛かグローバル（関数名・
グローバル値）に解決されること。match アームの payload はそのアームの本体でだけスコープに入る
（兄弟アームの間では外れる）。

さらに、型が示すだけで型検査では強制されない構造の不変条件:

- `Retain`/`Release` の path が `rc_units` の要素の上（またはちょうどその位置）で止まること。
- capture パラメータが closure ABI のときちょうど存在すること。
- match が 1 つ以上のアームを持ち、catch-all アームがあれば最後にあること。
- `Llvm` op の埋め込みオペランド名が `args` と一致すること。
- **closure の捕捉順**: `Closure(FuncRef, captures)` の格納レイアウトと、対象関数の
  `InlineLLVMCaptureProjectBody` が持つ `cap_tys` が一致すること。両者は同じレイアウトの二重記録で、
  片方だけ書き換えた rewrite は全射影を別スロットに向ける。射影自身についても、その関数の capture
  パラメータを読むこと・レイアウトにあるスロットを指すこと・関数内の他の射影とレイアウトが一致する
  ことを見る。

## 参照カウントの検査

**キーは `(root オブジェクト, unit)`**（`cancel` と同じ `unit_key` = `origin` の identity を
`truncate_to_unit` で unit に切り詰めたもの）。binding 単位の線形性は `cancel` が別 binding をまたいで
retain/release を対消滅させるため成立しない。

- **初期化**: 各パラメータ/capture の `rc_units` について、`borrowed_units` に無ければ 1、あれば
  borrowed（カウンタを持たない = 0）。
- **producer**: binding `x` の unit は `unit_key(x, leaf) == (x, unit)` のときだけ +1。別名（`Move`、
  unbox の `Field`、unbox union と catch-all の `Payload`、`result_prov` 素通し）は +0。boxed union の
  variant payload（アーム入口で暗黙に retain される）と boxed コンテナの `Destructure` フィールドは
  producer。
- **消費**: `rc-ownership-model.md` の表のとおり -1。`Destructure` は codegen の意味論（boxed は
  コンテナ全消費、unbox は名前の付かないフィールドのみ）で数える。同じ unit に属する複数の leaf
  （unbox union の各 variant）は 1 回だけ数える。
- **`Match`**: 各アームを分岐前状態のコピーから走査し、**全アームの出口カウントが一致**すること。
  アームの `Ret` は match binding への**移動**として数える（キーが同じなら増減なし）。
- **use-after-consume**: カウンタが 1 -> 0 に落ちたキーを dead とし、以降の値としての読み出し
  （`Ret` / オペランド / match の scrutinee / `Destructure` のコンテナ / `Eval`。`Retain`/`Release`
  ノードへの出現は読み出しではない）を検出する。borrowed キーは dead にならない。
- **エラー**: カウンタが負になる／関数出口で非 0 のキーが残る／アーム出口が不一致／dead キーの読み出し。

関数本体は木（分岐は match のアームだけ、ループは呼び出しでしか作れない）なので、不動点計算は不要で
1 パスの木走査＋アーム出口の一致検査で全パスを尽くせる。

### unit は leaf 経由で解決する

`origin` は **boxed leaf の上でしか定義されていない**。unit path そのものは leaf とは限らず
（punched array の unit は `[i]`、その leaf は `[i, 0]`）、leaf でない path を `origin` に渡すと
`result_prov` に該当する leaf が無く producer と誤判定される。よって unit は必ず、その unit に属する
boxed leaf を `unit_key` に渡して解決する（`value_keys`）。

### 検査から外すもの

- **root がグローバル名の unit**: グローバルは線形規律の外（読むたびに新しい参照が生まれ、refcount
  操作は no-op）。
- **root の `rc_units` に無い unit**: punched フィールドの下の leaf。`rc_units` はスキップするが
  `boxed_leaf_paths` はしないので、leaf 側でミラーして落とす。
- **unbox union が参照を数えるキー**: union の参照は生きている variant の参照であり、どのオブジェクトに
  属するかはタグで決まるのに、`origin` は構築時に置かれたオブジェクトを答える。別の variant を通る
  パスでは持っていないオブジェクトに課金してしまうので、union の unit path と各 variant leaf が解決する
  キーをまとめて除外する（`union_keys`）。別名の先（union に入れられた配列など）も同じキーなので一緒に
  外れる。
- **fully-unboxed 値**（`needs_rc` が偽、funptr 型を含む）には unit が無い。

他コンパイル単位のシンボルは `resolve_callee_params` が `None` を返して全 Own とみなされる
（borrow 最適化が走るのは単一ユニット時のみなので、実際に曖昧になることはない）。

## 循環への注意（設計上の要点）

参照収支を `infer_ownership` の上に載せると**循環**する。ownership 推論自身が `collect_consumes` から
`borrowed_units` を決めているので、同じ推論から導いた検査は同じ穴を共有し、「宣言された所有権」と
「実際の消費」が食い違っていても一致してしまう。

そこで消費は **codegen が実際に行う RC**（`destructure_consumes` / `get_struct_fields` /
`get_union_value` / 各 op の `borrows_operand`・`result_prov`）から導き、所有権は IR に載っている宣言
（`borrowed_units`、`all_owned_units` で読む）を使う。検査は「宣言モデルへの適合」を見るのであって、
`borrows_operand` / `result_prov` の宣言が実装と一致しているかまでは見ない（そこはテストと valgrind の
守備範囲）。

## 検査を変えるときに通す検証

1. **陽性対照**（検査が本当に発火することの確認）。修正前のコンパイラを worktree に建てて、その RC IR で
   検査が発火することを見る。使える既知の陽性対照は 2 つある。
   - `83a65cc8` 時点のコンパイラ + `match u { some(a) => a.@(0) + (if u.is_some {1} else {0}), none(_) => 0 }`
     （`box union`）。scrutinee をアーム内で読む形で、当時の RC 挿入は解放後使用と二重 release を出す。
   - 同コンパイラ + boxed struct 引数を destructure するだけの関数。ownership 推論が `Borrow` と誤推論し、
     borrow 版が所有していないコンテナを release する。
2. **単体テスト**: `validate.rs` 内で malformed な RcExpr を組み、`should_panic` で検出を確認する。
3. **偽陽性ゼロ**: 全テストスイートを 3 つの opt レベル（default / basic / none）で通す。加えて gate を
   一時的に無条件 true にして、fixlang_minilib 全サブプロジェクト（`fix test -O max`）と project_euler の
   ビルドを回す。実プログラム 60 本規模・毎回約 2,000 関数が掛かるので、除外規則の抜けはここで出る。
