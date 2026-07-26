# 広い戻り値と末尾呼び出し: 実装計画

戻り値がターゲットの戻り値レジスタに収まらない末尾呼び出しは、通常の呼び出しとして生成される。
このためモナドループが O(n) のスタックを消費する。本計画は、そのような戻り値を out-pointer
引数で返すことで末尾呼び出しを取り戻し、続いて、同じ限界を回避するために `Basic` が抱えている
`Std::fix` の defunctionalization を外す。Phase 1 まで完了。

## 問題

関数の戻り値は、スカラ leaf に平坦化された形で返される (`object.rs` の
`lambda_function_type`)。LLVM の戻り値規約はレジスタクラスごとに決まった本数しか持たず、それを
超えると SelectionDAG が戻り値を隠しポインタに降格させる。そして
`X86TargetLowering::IsEligibleForTailCallOptimization` は、caller か callee がその形で返すとき
sibcall 最適化を諦める。IR 上の `tail` マーカーは全ての IR パスを生き延び、命令選択の段階で
落とされる。

`Std::IO` の `bind` は `(f(a).@runner)(iostate)` を末尾位置に持つので、モナドループは間接末尾
呼び出しの連鎖であり、定数スタックで回るには sibcall 最適化を必要とする。`Array` は 3 leaf
(`storage`, `size`, `capacity`) を占めるため、普通のモナドコードが x86-64 の上限を越える。結果に
`Array` を載せるループ全般と、`IOFail` のループすべて (`Result ErrMsg a` の payload が `String`
を含むので、union はタグ 1 個 + 3 ワード) が該当する。

`Max` はこれを直しているのではなく隠している。`inline` + `unwrap_newtype` + `decapturing` が連鎖を
直接自己再帰に潰し、LLVM の IR レベルの末尾再帰除去が ABI に到達する前にループへ変換するためで
ある。これは保証ではない。呼び先がデータ構造から来る末尾呼び出しは間接のまま残り、どの最適化
レベルでも O(n) のスタックを消費する。

### 実測した閾値

末尾呼び出しが成立する最大の戻り値を、ターゲットとレジスタクラス別に測った (`llc -O2`、
`%r = tail call T @f(...)` / `ret T %r` の形の手書き IR):

| target | 整数 / ポインタ | 浮動小数 |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | 3 | 4 |
| `x86_64-apple-macosx` | 3 | 4 |
| `x86_64-pc-windows-msvc` | 3 | 4 |
| `arm64-apple-macosx` | 8 | 8 |
| `aarch64-unknown-linux-gnu` | 8 | 8 |

クラスごとに独立した予算を持つ。x86-64 では `{i64,i64,i64,double,double,double}` (6 leaf) は
末尾呼び出しになり、`{i64,i64,i64,i64,double}` (5 leaf) はならない。したがって単一の leaf 数
では規則を表現できない。

挙動は LLVM 17.0.6, 20.1.2, 21.1.8, 22.1.8 で同一である。

### 引数の側にも別の限界がある

戻り値とは独立に、**x86-64 の sibcall は値の変わる引数を 6 本しか運べない**。7 本目からは
スタック渡しになり、バックエンドは「呼び出し前後で同じスタックスロットに同じ値が載っている」
場合しか末尾ジャンプにしない (`llc -O2`、整数 n 本を `+1` して末尾呼び出しする IR で実測)。
AArch64 は 14 本まで通る。

| target | 値が変わる整数引数 |
| --- | --- |
| `x86_64-unknown-linux-gnu` | 6 |
| `aarch64-unknown-linux-gnu` | 14 以上 |

`tailcc` 呼び出し規約はこちらを解消する (13 本 + out-pointer でも `jmp`)。ただし**戻り値の側は
解消しない** (`tailcc` でも 4 leaf で `callq`、`swifttailcc` は 3 から 4 に上げるだけ)。
つまり 2 つの限界は独立で、out-pointer は `tailcc` を採っても必要である。

### 同じ限界に由来する別件

`defunctionalize_fix` が `self` の multi-use 部分適用を direct self-call に畳めず、`-O basic` で
深い再帰がスタックオーバーフローする件 (#100) も、これと同じ限界による。同じ形のまま戻り値を
1 leaf にすると `-O basic` で定数スタックになるので、溢れさせているのは self-call が indirect で
あること自体ではなく、indirect かつ戻り値が広いことである。本計画が入れば #100 の `-O basic`
での症状は消える。#100 自体は残る (`-O max` で devirtualization に依存しなくなること、反復ごとの
funptr closure 構築が消えること、パスの doc コメントが謳う保証が回復すること)。

## 変更

戻り値がターゲットの戻り値レジスタを超える関数型について、out-pointer を渡して `void` を返す:

- シグネチャ: `void f(ptr %out, <引数の leaf 群...>, <クロージャなら capture ptr>)`
- 本体: 各 return は leaf を `%out` 経由で store する
- 末尾呼び出し側: caller 自身の `%out` をそのまま渡して `tail call void` を出し、`ret void` で
  終える。そのバッファは畳まれるフレームではなく祖先フレームのものなので、置き換えられる
  フレームより長生きする
- それ以外の呼び出し側: バッファを `alloca` して渡し、戻ってから leaf を load する

実測: 属性なしの `ptr` 引数なら、上記のどの LLVM バージョンでも、直接呼び出し (`jmp f@PLT`)
と間接呼び出し (`jmpq *%rdx`) の両方で末尾呼び出しになる。

**このポインタに `sret` 属性を付けない。** 属性を付けると `isCalleeStructRet` が立ち、LLVM 17 から
21 では通常の呼び出しに戻る。(LLVM 22 は転送された `sret` ポインタを受け入れるが、属性なしの
ポインタが全バージョンで動くので、属性から得るものはない。)

`noalias nocapture writeonly` は付けてよい。これらは末尾呼び出しを保ったまま alias 解析に情報を
与える (実測)。

### 閾値の決め方

正確な述語は `TargetLowering::CanLowerReturn` だが、これは LLVM の C API にも inkwell にも
露出していない。ターゲットをキーに、レジスタクラスごとの leaf 数を数えるテーブルを持つ:

- x86-64: 整数/ポインタ 3、浮動小数 4
- AArch64: 整数/ポインタ 8、浮動小数 8

判断に迷うときは out-pointer 側に倒す。収まる値を out-pointer にした場合に失うのは IR 最適化の
余地だけだが、収まらない値を見落とした場合は O(n) のスタックが何の signal もなく残る。ターゲット
を増やすときと LLVM を上げるときはテーブルを見直す。エントリが古くなったことを可視化するのは、
Phase 1 の定数スタックテストである。

判定は**モジュールの内容に一切依存しない純粋な型駆動**でなければならない。`Basic` では分割
コンパイルにより 100 個規模のユニットが別々に処理されるので、あるユニットの定義と別ユニットの
宣言で判定が食い違うと ABI が壊れる。

### C ABI を保つ境界

`ExportStatement::implement`、`build_main_function`、`fixruntime_*` の宣言は C との境界なので
現状のままにする。

### オブジェクトキャッシュ

規則はターゲットに依存し、生成されるコードは規則に依存する。オブジェクトキャッシュのハッシュが
既にターゲットを区別していることを確認したうえで依拠する。

## 予想される性能影響

広い戻り値が IR の時点でメモリになるため、ポインタが escape する場面で SROA がそれを追えなく
なる。影響が及ばないと考えられる根拠と、残るリスクを先に置く。

影響が及ばないと考えられる根拠:

- ループ状態は**引数**を通る。今回変えるのは戻り値だけで、引数のスカラ化には触らない。
  `2026-07-23-array-flip-scalar-abi/findings.md` が扱った退行クラスは引数と phi の話である。
- **out-pointer でも TRE はループ化する** (実測)。`void @self_outptr(ptr %out, i64 %n)` の直接
  自己再帰は `opt -O2` で後退辺を持つループになる。末尾再帰の fold はループのままで、ループを
  回るのは引数のスカラである。
- **inline された呼び出しは完全に回収される** (実測)。非末尾で 4 leaf を受ける形を
  `alwaysinline` にして `opt -O2` にかけると、alloca も store も load も消えて呼び出し側は
  算術 1 命令になる。
- 機械語コストは増えない。LLVM は今すでに同じ降格を行っており、加えて現状のコードはフレーム
  ごとに余分なバッファを確保して結果を caller のバッファへコピーし直している。

残るリスク: **inline されず、末尾でもなく、広い戻り値を返す呼び出し**。
`2026-07-23-array-flip-scalar-abi/findings.md` が `bounds_check_indexable` を「戻り値が
aggregate のまま残る事例」として記録している形がこれにあたる。閾値が 4 leaf なので
`(Array a, b)` が該当し、対象は狭くない。

この不確実性が、Phase 2 と Phase 3 の 2 つの実装を用意して比較する理由である。

## フェーズ

### Phase 1 — 定数スタックのテスト (完了)

`src/tests/test_wide_return_tail_call.rs`。モナドループを 100 万段深く再帰させ、完走することだけを
確かめる。完走するかスタックが溢れるかの二値の結果なので、マシンの負荷に影響されず、混んでいる
CI でも走らせられる。

機構の異なる形を網羅する:

- `(Array String, I64)` を返す直接自己再帰のモナドループ
- 状態に配列を持たない `IOFail` のループ
- `break_m` の結果が広い `loop_m`
- ユーザ定義の `StateT` 形式のモナド変換子を `IO` の上に重ねたもの
- 次に呼ぶ関数を配列から取り出すループ (どの最適化レベルでも末尾呼び出しが間接のまま残る)
- 3 つの `Array` = 9 leaf を返すループ。全ターゲットの予算を超えるので AArch64 でも意味を持つ

`Basic` と `Max` で走らせる。`None` は意図的に末尾呼び出しも行わないので、`test_util` の
`tail_call_optimization_enabled` で飛ばす (`test_defunctionalize_fix.rs` の `should_skip_at_none`
をここへ移した)。4 leaf の形は AArch64 の予算 8 に収まるので、それらが働くのは x86-64 の CI
セルである。

再帰の深さは、1 フレームあたりの消費が形によって変わることを見込んで余裕をとる。実測では
最小の形で 112 バイト/フレーム、minilib の Free モナドで 800-1600 バイト/反復だった。

実測 (`ulimit -s 8192`): `Basic` は 6 本すべてがスタックオーバーフローで落ち、`Max` は配列越しの
間接ディスパッチのものだけが落ちる。所要は `Basic` 24 秒、`Max` 11 秒。

#### 既存テストのガード: `test_regression_issue_63`

`test_basic.rs` の `test_regression_issue_63` は、`U64` 12 個を状態に持つユーザ定義 State モナドを
1000 万回回すもので、`Basic` 以下を飛ばしている。out-pointer を入れても `-O basic` では依然
スタックが溢れる。`run : s -> (s, a)` は引数 13 本・戻り値 14 leaf で、**引数の側の限界**
(x86-64 で 6 本) に掛かっているためである。`tailcc` を採るならこのテストのガードを
`tail_call_optimization_enabled` に変えて `Basic` でも走らせる。

### Phase 2 — codegen で out-pointer にする (オラクル)

閾値テーブルと、`Configuration` の述語 (ABI を切り替えられるようにする) をここで作る。以降の
フェーズと共用する。

変更箇所:

- `lambda_function_type` (`object.rs`) — out-pointer 付きのシグネチャを組む。
- `apply_lambda` (`generator.rs`) — 末尾呼び出しでは `%out` を転送し、それ以外では確保して
  load する。
- `build_return_object` / `build_tail` / `unpack_return` (`generator.rs`) — 平坦な戻り値を組む
  代わりに `%out` 経由で store する。
- `implement_rc_function` と `declare_rc_function` (`rc_ir/codegen.rs`) — out-pointer が引数
  leaf の前に来るので、パラメータの束縛が 1 つずれる。平坦化したパラメータが関数のパラメータを
  ちょうど使い切ることを確かめる develop モードの assertion が、ここでの食い違いを捕まえる。
- `InlineLLVMFixBody::generate_tail` (`fixstd/builtin.rs`) — 実際の末尾呼び出しを出しており、
  同じ転送が必要。

検証: Phase 1 のテストが通ること。`test_regression_issue_63` のガードを
`tail_call_optimization_enabled` に置き換えて `Basic` で通ること。`Basic` と `Max` での
`cargo test --release`。RC-IR テストの memcheck。speedtest 一式と project_euler でプログラムの
出力が変わらないこと。

この実装は Phase 3 の**正しさの基準**として使う。Phase 3 は最適化済み IR を書き換えるため、
取りこぼしが静かな miscompile になる。同じプログラムが Phase 2 版と Phase 3 版で同じ出力・同じ
定数スタックになることを突き合わせられる独立な基準を持つ意味は大きい。

### Phase 3 — 最適化後に書き換える

codegen は Phase 2 より前の形に戻し、`optimize_and_verify` と `write_to_object_file`
(`build_object_files.rs`) の間でモジュールを書き換える。こうすれば全ての IR パスはスカラの
戻り値を見たままになる。inkwell は LLVM のパスマネージャにパスを登録する手段を出していないので、
これは最適化済みモジュールを inkwell の API で Rust 側から書き換える形になる。

opaque pointer のおかげで関数型は `call` 命令自身に書かれている。定義・宣言・呼び出しを型駆動で
全走査すれば ABI は整合する。**網羅性が命であり、取りこぼしは片側だけ ABI が変わる miscompile に
なる。**

扱う必要のある形:

- **マージされた return**。SimplifyCFG は複数の return を 1 つのブロックに沈め、その `ret` は
  呼び出し結果の phi を取る。incoming が広い呼び出しの結果である各 predecessor について、その
  呼び出しに `%out` を転送し、predecessor を `ret void` で終える。(手を加えない場合の末尾位置は
  `CodeGenPrepare` が復元する。現行ビルドがこのマージで何も失っていないのはそのためである。)
- **デバッグ情報**。`-g` の後に `-O basic` / `-O max` を指定できるので、書き換えとデバッグ情報は
  共存しうる。verifier は命令の `!dbg` のスコープがそれを含む関数の `DISubprogram` に属することを
  要求するので、ブロックを新しい関数へ移すときは `DISubprogram` も移し替える (LLVM 自身の
  `ArgumentPromotion` / `DeadArgumentElimination` と同じ扱い)。デバッグ情報の**品質**は最適化
  レベルを上書きした時点で保証の対象外だが、verify を通ることは必要である。
- **分割コンパイル**。ユニットごとに書き換えるので、判定が純粋な型駆動であることに依拠する。

最後に `verify` パスを回す。`optimize_and_verify` が既に verify を回しているので、破綻は静かな
破損ではなくハードエラーとして出る。テスト行列に `-g -O basic` と `-g -O max` を 1 本ずつ足す。

検証: Phase 2 と同じ検証一式に加えて、Phase 2 版のビルドとの出力突き合わせ。

### Phase 4 — 計測して 1 つに絞る

baseline / Phase 2 / Phase 3 の 3 本で cachegrind の命令数を比較する。注視するのは
`bounds_check_indexable` と、fold / iterator の読み出しループ一式。

差が小さければ Phase 3 の複雑さを持ち続ける価値を再考する。どちらを残すか決めたら、もう一方と
`Configuration` の切り替え述語を削除する。2 つの ABI を長期に併存させない。

### Phase 5 — `defunctionalize_fix` を `Basic` から外す

`defunctionalize_fix` が `Basic` で動いているのは、`fix` の自己呼び出しが関数ポインタ経由であり、
同じ戻り値幅の限界によってバックエンドがそれをジャンプに変換できないためである。ABI が直れば
その理由は消える。実測では、`Basic` において 1 leaf を返す深い末尾再帰の `fix` はこのパスを
無効にしても定数スタックで走り、パスを必要とするのは広い戻り値の場合だけだった。

`enable_defunctionalize_fix` を `Max` 以上に変える。コミット前に検討すること:

- このパスはループも形成する。これは末尾ジャンプの連鎖より強い。生成される直接自己呼び出しに
  LLVM の IR レベル末尾再帰除去が適用され、クロージャの間接も消えるためである。`Basic` から
  外すことは、ループをジャンプに交換することを意味する。計測すること。
- 未解決の不具合を抱えている。ネストした multi-site の `fix` でコンパイル時に lift が指数爆発
  する件 (#99) と、`self` の multi-use 部分適用で self-call が間接のまま残る件 (#100) である。
  前者はコンパイル時間の危険であり、`Basic` が避けるために存在するコストそのものである。

`Max` では残す。`fix` は inline の対象外 (`is_std_fix`) なので、ループを形成できるのはこの
パスだけである。

検証: `test_defunctionalize_fix.rs` の `*_runs_in_constant_stack` が `Basic` で通ること。
minilib の crypto の事例が定数スタックで走ること。

## 検討して採らなかった案

- **LLVM のバージョンを上げる。** 17.0.6, 20.1.2, 21.1.8, 22.1.8 のいずれも、降格された戻り値
  では sibcall を諦め、閾値も同じ位置にある。LLVM 22 は **転送された `sret` ポインタ** について
  末尾呼び出しを獲得したが (`llvm/llvm-project` PR #146575、`release/22.x` と `main` に存在)、
  それは上記の属性なしポインタが LLVM 17 で既に達成していることである。上流には明示的な `sret`
  の側の報告が open で残っている (`llvm/llvm-project` issue #8605)。降格された戻り値の側は
  見当たらないので、報告する価値がある。
- **戻り値レジスタの多い呼び出し規約。** `swifttailcc` は x86-64 の整数の閾値を 3 から 4 に
  上げるだけで止まる。`fastcc` と `tailcc` は何も変えない。
- **`inline` / `unwrap_newtype` / `decapturing` を `Basic` に降ろす。** 3 つ揃って初めて効き、
  相互再帰にはさらに `inline_local` と分割コンパイルの停止が要り、それでもデータ構造越しの
  末尾呼び出しは O(n) のままである。結果は名前を変えた `Max` になる。
- **C++ shim 経由で `CanLowerReturn` を呼ぶ。** `cc` でビルドする shim は LLVM のヘッダに対して
  コンパイルもリンクも通る (`-fno-rtti` は `llvm-config --cxxflags` から来る) が、この述語は
  `MachineFunction` を必要とするため、shim は LLVM のバージョン間で動く CodeGen の内部構造に
  依存する。これはテーブルと同じ保守コストを透明性なしに払うことであり、加えて C++ ツールチェーン
  と LLVM ヘッダが全プラットフォームでビルド要件になる。

## 対象外

真に末尾位置にない再帰は O(n) のスタックを消費し、それが正しい結果である。`-O max` でその一部が
ループになるのは LLVM の accumulator 末尾再帰除去によるもので、結合的な累算に限って成立する。
