# RC IR validator

RC IR の well-formedness を静的に検査する、コンパイラ開発時専用のチェッカ。実装は `src/rc_ir/validate.rs`。
消費モデルの仕様は `rc-ownership-model.md` にある。

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

## 参照カウントの検査（未マージ）

参照収支・use-after-consume・アーム出口の一致を見る検査は、**issue #105** の一部として実装済みで、
ブランチ `rc-ir-validate-reference-counting` にある。マージは次に RC のパスを触るときまで保留している。

保留の理由は保守コストである。この検査は `origin` の別名近似と `borrowed_units` の宣言に依存し、unbox
union（`Option` / `Result` / `LoopState` が該当する）を丸ごと除外している。モデルが codegen からずれると
偽陽性が panic として出て、他人のテストを止める。網が実際に働く場面 — RC のパスを新設または改造する
とき — に戻すのが釣り合う。

戻すときの陽性対照は 2 つある。修正前のコンパイラを worktree に建てて、その RC IR で検査が発火すること
を見る。

- `83a65cc8` 時点のコンパイラ + `match u { some(a) => a.@(0) + (if u.is_some {1} else {0}), none(_) => 0 }`
  （`box union`）。scrutinee をアーム内で読む形で、当時の RC 挿入は解放後使用と二重 release を出す。
- 同コンパイラ + boxed struct 引数を destructure するだけの関数。ownership 推論が `Borrow` と誤推論し、
  borrow 版が所有していないコンテナを release する。

## 検査を変えるときに通す検証

1. **単体テスト**: `validate.rs` 内で malformed な RcExpr を組み、`should_panic` で検出を確認する。
2. **偽陽性ゼロ**: 全テストスイートを 3 つの opt レベル（default / basic / none）で通す。加えて gate を
   一時的に無条件 true にして、fixlang_minilib 全サブプロジェクト（`fix test -O max`）と project_euler の
   ビルドを回す。実プログラム 60 本規模・毎回約 2,000 関数が掛かるので、除外規則の抜けはここで出る。
