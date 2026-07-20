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

位置づけ(判断材料):

- Fix ソースから到達できる mutate 経路は、`_unsafe_force_unique` と `*_uniqueness_unchecked` を
  廃止すると**すべて COW 内蔵**になる。`set` / `swap` / `mod` / `act` / punch・plug の force-unique 版 /
  `truncate` はもちろん、`FFI::mutate_boxed` も codegen が書き込み前に `make_array_unique` /
  `make_struct_union_unique` を通す。foreign resource は `Destructor::mutate_unique` が
  ユーザ提供のコピーコンストラクタで複製する(gmp / mpfr がこの形)。`borrow_boxed` は
  doc のとおり書き込み禁止。
- したがって `is_unique` の分岐は**両方の枝が個別に正しい**、純粋な性能ヒントになる。
  判定が楽観側にずれても、unique 枝の op が自分で COW するので破壊は起きない。
- つまりこの精密化は**最適化であって、健全性の修正ではない**。実装中に判断がつかない個別ケース
  (scrutinee が本当に `is_unique` 由来か辿りきれない等)は、精密化せずに通してよい — 取りこぼしても
  遅くなるだけ。ただしこれは個別ケースを諦めてよいという話で、課題自体を落とすと `act` は
  ランタイムチェックと clone path を抱えたままになる。
- **着手前に、実利があるかを測ってほしい。** `act` では `is_unique` と punch の間に call も store も
  無いので、Max では LLVM の GVN + jump-threading が 2 つのチェックを統合し、shared 枝ごと
  消してしまう可能性がある(背中合わせの 2 チェックが統合されることは別途実測で確認した)。
  その場合この精密化が効くのは主に `-O basic`(Std が別 compilation unit なので統合されない)で、
  Max での実利は小さい。`act` を回すベンチの LLVM IR を先に見て、残っているチェックの有無を
  確かめてから実装量を決めるのがよい。

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

## 対応 (2026-07-20)

### 問題 1: punch の result_prov

`InlineLLVMArrayPunchBody` と `InlineLLVMStructPunchBody` に leaf ごとの `result_prov` を実装した。

- force-unique する版: punch されたコンテナ leaf = `Fresh`、move out した要素 leaf = `Dyn`。
- force-unique しない版: すべて `Dyn`。passthrough は採らなかった。理由は
  `InlineLLVMIsUniqueFunctionBody::result_prov` と同じで、passthrough は「引数を消費しない」
  という宣言でもあり、それが抑止する retain が後続のチェックを正しく保つ。この判断と、
  それで逃すもの (`_unsafe_punch_bounds_uniqueness_unchecked` を直に呼んだ先の plug が
  チェックを保つ) はコードのコメントに残した。
- 構造体 punch は、punch された構造体が boxed のときだけ `Fresh`。unbox 構造体の force-unique は
  何もしないので、その中の boxed フィールドの共有については何も言えない。

検証: `Array::mod` を「COW punch + COW plug」に書き換えたマイクロベンチで、
RC IR に残るチェックが 9 -> 1 になった。COW plug 8 個すべてが畳まれ、残る 1 個はループが
自己 peel した初回イテレーションの punch である。

### 問題 2: 実測してから実装

**実測**: Max でも LLVM は畳んでいなかった。最適化後の LLVM IR の、
インラインされた act ループの中に refcount 比較と malloc/memcpy の clone path が残っている。
1000 要素の配列を 200 周 sweep する act ベンチ (functor = `Option`) の cachegrind Ir:

| 版 | Ir |
|---|---|
| 実装前 | 3,577,179 |
| plug のチェックを std から手で外した上限値 | 1,576,607 |
| 実装後 | 1,576,577 |

**-56% (2.27 倍)**。上限値に一致した = act の unique 枝からチェックが完全に消えた。

**実装**: `provenance.rs` の `interp_match` に入れた (`result_prov` は `Dyn` のまま)。
`is_unique` の結果を destructure して得た Bool を match する `true` アームで、同じ op が返した
値の leaf を `Fresh` として解釈する。あわせて std の `_unsafe_act_bounds_unchecked` の punch を
チェック付きの版に替えた。この 2 つが揃って初めて plug まで鎖が届く (チェックなしの punch は
結果を `Dyn` と宣言するので、そこで切れる)。unique 枝では punch のチェックは必ず成立するので
実行時の意味は変わらず、畳めなかった場合のコストも従来と同じ 1 チェックである。

**罠 (誤コンパイルを実際に再現した)**: refinement を無条件に効かせると、`is_unique` と分岐の
間で値を共有した場合に、共有済みの値が `Fresh` に「戻って」しまう:

    let (u, a) = arr.unsafe_is_unique;             // rc == 1 なので u = true
    let keep = [a];                                // ここで retain -> rc == 2
    let b = if u { a.set(0, 99) } else { ... };    // チェックが畳まれ keep が壊れる

-O max で `keep` が壊れることを実測した (-O none では正常)。retain は 2 本目の参照が生まれる
唯一の印なので、`Retain` ノードを見たら保留中の flag をすべて捨てる。retain が指すのが値
そのものではなく別名のこともあり、それを見分けるには borrow パスが持つオブジェクト同一性が
要るので、粒度は粗く取った。このケースは `test_basic.rs` の
`test_is_unique_true_branch_invalidated_by_sharing` に回帰テストとして入れた。

### 規約

`LLVMGen::result_prov` の doc コメントに規約を明記し、`borrows_operand` からそこを参照した。
あわせて `_unsafe_set_bounds_uniqueness_unchecked_unreleased` と `unsafe_set_size` に、既定値の
ままにする理由とそれで逃すものを書いた。

### テスト

- `test_array_rmw.rs`: `map` が関数を複数回走らせる functor (`Array`、および完全にインライン
  可能な自作 `Twice`) での `act`、shared な配列への `act`。correctness と memcheck の両方。
  複数回走る場合に plug のチェックが残るのは、値が複数回使われる以上 retain が入り、それが
  provenance を落とすからである。
- `test_provenance.rs`: `unique_elim` ケースの dump で punch と plug がチェックなしの形に
  なることを表明する (実装前は plug 側が満たされないことを確認済み)。
- `test_basic.rs`: 上記の罠の回帰テスト。
