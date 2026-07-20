# provenance を保守側の既定値に置いたまま記録されていない取りこぼし 2 件

Array/Storage 再設計(`dev-docs/2026-07-18-array-buffer-representation/design.md`)の検討中に、
RC IR dump と LLVM IR の実測で見つかったもの。どちらも健全性のバグではなく、
「安全な既定値に落としたまま、理由も損失も記録されていない」ための取りこぼし。
実測は bce ブランチ(unique-check-elim をマージ済み)で取った。

## 問題 1: punch に result_prov が無く、直後の plug のチェックが畳めない

現状:

- `InlineLLVMArrayPunchBody`(src/fixstd/builtin.rs)は `result_prov` を実装しておらず、
  trait の既定である「全 leaf `Dyn`」になっている。`InlineLLVMStructPunchBody` も同じ。
- 一方 `InlineLLVMPunchedArrayPlugBody` と `InlineLLVMStructPlugInBody` は宣言している
  (前者は `uniform(Fresh)`)。

影響(今すでに出ている):

- `Array::_unsafe_act_bounds_unchecked` の unique 枝は「no-COW punch -> COW plug」。
  COW plug は `unique_check_operand = Some{container_index: 1, path: [0]}` を持つが、
  その operand は punch の結果なので `Dyn`。specialize で引数が Unique と分かっている
  呼び出しでも畳めず、runtime check と clone path が残る。
- `Array::mod` の plug を COW 版へ寄せると同じ理由で畳めない。mod/act を回す
  マイクロベンチ(-O max)で、uniqueness check が RC-IR 段 11 -> 15、LLVM O3 後 7 -> 8、
  malloc(clone path)が 17 -> 18 に増えることを実測した。

なぜ落ちていたか(構造上の理由。手抜きではないと思う):

- punch の結果は `(PunchedArray a, a)` という異種 2 leaf。コンテナ leaf は(force_unique 版なら)
  `Fresh` だが、**move out した要素 leaf は `Dyn` でなければならない** — 要素は retain せずに
  取り出しており他所から参照され得るので、`Fresh` と宣言すると後続の in-place 更新が
  共有データを壊す(unsound)。
- 既存の便利コンストラクタは `Provenance::uniform`(全 leaf 一律)だけなので、正しく付けるには
  `Provenance::build_shape` で leaf ごとに組む必要がある。健全側に落とした判断自体は妥当。

やってほしいこと:

1. `InlineLLVMArrayPunchBody::result_prov` を per-leaf で実装する。
   - `force_unique == true`: コンテナ leaf(path `[0]`)= `Fresh`、要素 leaf(path `[1]`)= `Dyn`。
   - `force_unique == false`: コンテナ leaf を引数 passthrough にできるかは問題 2 の罠を
     踏まえて判断する。安全側なら `Dyn` のままでよい。
2. `InlineLLVMStructPunchBody` も同じ形。
3. 検証: `Array::mod` を「COW punch + COW plug」に書き換えたとき、上のマイクロベンチで
   uniqueness check が baseline(RC-IR 11 / LLVM 後 7)まで戻ること。
   `--emit-rc-ir all` の post dump(`.fixlang/rc_ir.post.txt`)で `[unique]` マーカーを見るのが早い。

## 問題 2: unsafe_is_unique の true 枝で provenance が精密化されない

まず前提として、`InlineLLVMIsUniqueFunctionBody::result_prov` の全 leaf `Dyn` は
**意図的で、変えてはいけない**。コメントより詳しく機構を書きます。

- `result_prov` に `Arg(j, p)` を書くと、borrow パスの 2 箇所がそれを読む:
  - `root_inner`(borrow.rs): その結果 leaf は引数 leaf の pure projection = **エイリアス**と
    みなされ、RC unit の root が引数まで辿られる。
  - `rhs_consumes`(borrow.rs): passthrough と宣言された引数 leaf は、その op に
    **消費されない**扱いになる。
- 実際に `Arg(...)` を宣言している op は `InlineLLVMStructGetBody` /
  `InlineLLVMMakeStructBody` / `InlineLLVMMakeUnionBody` / `InlineLLVMUnionAsBody` の 4 つだけで、
  いずれも結果が引数の**部分**(同じ格納域)であり、新しい値を生まない。
- `is_unique` の第 2 成分は「引数の部分」ではなく、独立に使える値として返される。ここを
  `Arg(0)` と宣言すると、呼び出し側が `arr`(後で使う)と `arr2` の両方を保持していても、
  解析からは「1 本の参照を何度か使っているだけ」に見える。すると**値が複製されたことを表す
  retain が不要と判断され、挿入されない**。実行時 refcount は 1 のままなので `is_unique` は
  `true` を返し、他の生存ホルダーが観測している値を in-place で書き換えて壊す。
  これが test95 の落ち方。
- 対比: `set` は `Fresh` を宣言している。「結果はエイリアスではなく新しい値」という宣言なので
  引数は消費され、引数を後で使えば retain が入り、`set` 内部の COW が rc==2 を見て clone する。
- 要するに `Arg(...)` は「読み出しのための表現エイリアス」の宣言であって、
  「同じ値を所有権つきで返す」宣言ではない。`is_unique` は refcount そのものを答えにする op なので、
  所有権の帳簿を省略させてはいけない。

そのうえで、取りこぼしはこれです:

- `let (unique, arr) = arr.unsafe_is_unique; if unique { ... }` の **true 枝の中でも
  `arr` の provenance は `Dyn` のまま**。`provenance.rs` の `interp_match` は分岐条件を見ないので、
  経路依存の精密化が一切かからない。
- 結果、true 枝に置いた COW op のチェックは実行時には必ず成立するのに畳めない
  (`Array::_unsafe_act_bounds_unchecked` がまさにこの形)。Max では LLVM の GVN + jump-threading が
  背中合わせの 2 チェックを 1 つに統合してくれることを実測で確認したが、これは「間に call も
  store も無い」場合に限る。-O basic では Std が別 compilation unit なので統合されない。

やってほしいこと:

- **足りないのは provenance の宣言ではなく分岐の知識**なので、修正は `interp_match` に入れる。
  match の scrutinee が `unsafe_is_unique` の結果の Bool 成分であるとき、true 枝では、
  その `unique_check_operand` が指すコンテナ leaf を `Unique` として解釈する。
- **`result_prov` は `Dyn` のまま**にすること。ここを触ると上の事故が再発する。

健全性:

- true 枝に入った時点で refcount == 1 なのは事実。その後 retain が挿入されて共有になっても、
  `provenance.rs` の `RcExpr::Retain` が `p.demote(path)` で provenance を落とすので、
  精密化した値も同じ経路で無効化される。したがって精密化を足しても不健全にはならない
  (実装時にこの demote が効いていることを確認してください)。

検証:

- `act` を回すベンチで、unique 枝の COW plug の runtime check が消えること。
- `cargo test --release`(既定 = Max)。特に test95 系。
- opt レベルは Max だけでよい。`borrow_ify` / `cancel` / `specialize`(`analyze_program` を含む)は
  `config.enable_borrow_optimization()`(= `fix_opt_level >= Max`)の内側でしか走らず、
  `result_prov` と `interp_match` を読むのはその中だけなので、basic / none は結果が変わりようがない。
  ゲート外(std.fix、lowering、`rc_insert`、`split_rc_units`、codegen、そして `rc_insert` が
  全レベルで読む `borrows_operand`)に触ったときだけ全レベルを回すこと。

## 併せてお願いしたい規約

今回の 2 件はどちらも「健全側の既定値に落とすのは正しい判断だったが、そう判断したことも、
それによって何が畳めなくなるかも、コードに書かれていない」ために、ベンチを取るまで
誰も気づけない状態でした。

以後、`result_prov` や `borrows_operand` を保守側の既定値のままにする op には、
「なぜ既定値なのか」「それで何を逃しているか」を 1 行コメントで残してください。
