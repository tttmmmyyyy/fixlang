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

## 実装済みの検査

**(i) 構造**: 関数内で束縛名が一意（シャドー禁止）であること、各変数使用がスコープ内の束縛か
グローバル（関数名・グローバル値）に解決されること。

match アームの payload はそのアームの本体でだけスコープに入る（兄弟アームの間では外れる）。

## 未実装の検査

### (ii) use-after-consume

所有キーが 1 -> 0 に落ちた時点でそのキーを dead とし、以降の**値としての読み出し**
（`Ret` / オペランド / match の scrutinee / `Destructure` のコンテナ。`Retain`/`Release` ノードへの
出現は読み出しではない）を検出する。borrowed キーは dead にならない。

### (iii) 参照収支

**キーは `(root オブジェクト, unit)`**（`cancel` の `key()` と同じ `root` + `clamp_unit`）。
binding 単位の線形性は `cancel` が別 binding をまたいで retain/release を対消滅させるため成立しない。

- **初期化**: 各パラメータ/capture の `rc_units` について、`borrowed_units` にあれば borrowed として
  マーク（カウンタは持たず、減算されたらエラー）、無ければ 1。
- **producer**: binding `x` の unit `u` は `root(x, u) == (x, u)` のときだけ +1。別名（`Def::Move`、
  unbox の `Def::Field`、unbox union と catch-all の `Def::Payload`、`result_prov` 素通し）は +0。
  boxed union の variant payload と boxed コンテナの `Destructure` フィールドは producer。
- **消費**: `rc-ownership-model.md` の表のとおり -1。`Destructure` は codegen の意味論
  （boxed はコンテナ全消費、unbox は名前の付かないフィールドのみ）で数える。
- **`Match`**: 各アームを分岐前状態のコピーから走査し、**全アームの出口状態が一致**すること。その後
  結果 `x` を producer として +1。
- **エラー**: カウンタが負になる／関数出口で非 0 のキーが残る／アーム出口が不一致／borrowed キーの
  消費・`Release`。

関数本体は木（分岐は match のアームだけ、ループは呼び出しでしか作れない）なので、不動点計算は不要で
1 パスの木走査＋アーム出口の一致検査で全パスを尽くせる。`root` は全域かつ決定的（may-alias ではない）
なので、キーは近似なしで正確に求まる。

### (iv) closure の捕捉順

`Closure(FuncRef, captures)` の格納順と、lifted 関数が cap から射影する順が一致すること。

## 循環への注意（設計上の要点）

(iii) を `borrow.rs::collect_consumes` の上に載せると**循環**する。ownership 推論自身が
`collect_consumes` から `borrowed_units` を決めているので、同じ関数から導いた検査は同じ穴を共有し、
「宣言された所有権」と「実際の消費」が食い違っていても一致してしまう。

非循環にするには、消費を **codegen が実際に行う RC**（`destructure_consumes` / `get_struct_fields` /
`get_union_value` / 各 op の `borrows_operand`・`result_prov`）から導き、**宣言された所有権
（`borrowed_units`）との不一致**を突き合わせる形にする。検査は「宣言モデルへの適合」を見るのであって、
`borrows_operand` / `result_prov` の宣言が実装と一致しているかまでは見ない（そこはテストと valgrind の
守備範囲）。

## 実装したら通す検証

1. **陽性対照**（検査が本当に発火することの確認）。修正前のコンパイラを worktree に建てて、その RC IR で
   検査が発火することを見る。使える既知の陽性対照は 2 つある。
   - `83a65cc8` 時点のコンパイラ + `match u { some(a) => a.@(0) + (if u.is_some {1} else {0}), none(_) => 0 }`
     （`box union`）。scrutinee をアーム内で読む形で、当時の RC 挿入は解放後使用と二重 release を出す
     -> (ii) と (iii) が発火するはず。
   - 同コンパイラ + boxed struct 引数を destructure するだけの関数。ownership 推論が `Borrow` と誤推論し、
     borrow 版が所有していないコンテナを release する -> (iii) の「borrowed キーの消費」が発火するはず。
2. **単体テスト**: `validate.rs` 内で malformed な RcExpr を組み、`should_panic` で検出を確認する。
3. **偽陽性ゼロ**: 全テストスイートを 3 つの opt レベル（default / basic / none）で通す。加えて gate を
   一時的に無条件 true にして、fixlang_minilib 全サブプロジェクト（`fix test -O max`）と project_euler の
   ビルドを回す。実プログラム 60 本規模・毎回約 2,000 関数が掛かるので、除外規則の抜けはここで出る。
