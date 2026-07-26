# 広い戻り値と末尾呼び出し: 実装計画

戻り値がターゲットの戻り値レジスタに収まらない末尾呼び出しは、通常の呼び出しとして生成される。
このためモナドループが O(n) のスタックを消費する。本計画は、そのような戻り値を out-pointer
引数で返すことで末尾呼び出しを取り戻す。

**完了。** Phase 1 (テスト) と Phase 2 (codegen の out-pointer) を実装し、加えて Phase 2 の
途中で見つかった引数側の限界を `tailcc` で解いた。Phase 3 と Phase 5 は計測により破棄した
(それぞれの節に結果がある)。

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

Fix は unbox struct を leaf に展開して渡すので、状態を引数で運ぶループはこの 6 本をすぐ使い切る。
out-pointer 1 本 + 4 leaf の状態 + capture ポインタで、もう埋まる。`m` を挟まない State モナド
(`run : s -> (s, a)`) がこの形である。`IO` の上に重ねた `StateT` は状態が capture に載るので
掛からない。

**採否はユーザの判断による設計変更である。** 実装は `return_abi.rs` の
`LAMBDA_CALLING_CONVENTION` と、それを設定する 3 箇所 (`declare_lambda_function`、
`declare_rc_function`、`apply_lambda`) だけなので、コミット単位で戻せる。`main`、export した
ラッパ、`fixruntime_*`、FFI 宣言、traverser、RC ヘルパ、`Get#<symbol>` は C の規約のままである。

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
1000 万回回すもので、`Basic` 以下を飛ばしていた。`run : s -> (s, a)` は引数 13 本・戻り値 14 leaf
なので、out-pointer と `tailcc` の両方が要る。両方入れた状態で `-O basic`・スタック 8 MiB・
1000 万段が完走することを確認したので、ガードを `tail_call_optimization_enabled` に変えた。

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

### Phase 3 — 最適化後に書き換える (破棄)

Phase 4 の計測により破棄した。この案は「codegen 段階で out-pointer を入れると IR 最適化の余地が
減って性能が落ちる」という懸念のために用意したものだったが、実測はその逆だった。従来は LLVM が
命令選択の段階で、つまり全ての IR パスの後で、勝手に隠しポインタへ降格させていたので、IR 最適化は
その往復を見ることも消すこともできなかった。out-pointer を IR の最初から見せると SROA が inline
された箇所で完全に消す (`-O max` で `out@call_lambda` の alloca が最適化後 0 個になる)。

以下は破棄した案の記録である。

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

### Phase 4 — 計測して 1 つに絞る (完了)

Phase 3 を作らなかったので絞る対象は無く、baseline / Phase 2 / Phase 2 + `tailcc` の 3 本を
speedtest 41 ケースで比較した (`-O experimental`、cachegrind の命令数)。

| 区間 | 中央値 | 最良 | 最悪 | 不変 |
| --- | --- | --- | --- | --- |
| baseline -> out-pointer | 0.00% | -53.94% | +0.06% | 27/41 |
| out-pointer -> +`tailcc` | 0.00% | 0.00% | +1.96% | 29/41 |
| baseline -> 両方 | 0.00% | -53.94% | +1.51% | 25/41 |

全 41 ケースの合計は 16,386,177,355 -> 15,866,106,886 命令 (-3.17%)。

改善はすべて out-pointer 側から来ている。`mandelbrot` と `mandelbrot_fold` が -53.94%、
`index_syntax` が -3.33%、`cp_lib_conv_zp` が -1.99%。out-pointer 単独での退行は `sort` の
+0.06% だけである。計画がリスクとして名指しした `bounds_check_indexable` は完全に不変だった。

退行はすべて `tailcc` 側から来ている。`fannkuch` +1.96%、`cp_lib_bipartite` +1.83%、
`cp_lib_unionfind` +1.56% など。改善はゼロ。

### Phase 5 — `defunctionalize_fix` を `Basic` から外す (破棄)

述語を `Max` に上げて計測し、破棄した。`sum_by_fix` を `-O basic` で:

| | 実行命令数 | コンパイル時間 |
| --- | --- | --- |
| パスあり | 47,162,415 | 4.90 / 4.94 秒 |
| パスなし | 686,163,411 | 4.83 / 4.91 秒 |

**実行が 14.5 倍悪化し、コンパイル時間は変わらない。** このパスが作るのは末尾ジャンプの連鎖では
なくループで、`fix` コンビネータが反復ごとに構築するクロージャ (ヒープ確保 + 参照カウント更新)
ごと消える。ABI 修正が与えるのはジャンプだけなので、クロージャ構築は残る。

`Basic` に入れた当初の理由 (スタックオーバーフロー回避) は ABI 修正で確かに消えたが、別の理由で
価値があった。述語は `Basic` のままとし、この計測値を `enable_defunctionalize_fix` の doc
コメントに残した。

### Phase 6 — 戻り値をレジスタ部とメモリ部に分ける (未着手)

いまの実装は LLVM と同じ all-or-nothing で、1 leaf でも予算を超えると戻り値全体がメモリへ行く。
クラスごとの予算まではレジスタで返し、溢れた分だけ `%out` に置く形も取れる。

分割規則は型だけで決まる。`flatten_to_scalar_leaves` の順に leaf を走り、そのクラスの予算が
残っていればレジスタ、尽きていればメモリ。呼び出し側と呼び先が同じ型から同じ結論に達するので、
分割コンパイルでも整合する。

**末尾呼び出しは保たれる** (`llc -O2` で実測)。レジスタ部を普通に返しつつ `%out` を受け取り、
末尾呼び出しに `%out` を転送してレジスタ部をそのまま返す形は、`ccc` でも `tailcc` でも、x86-64
でも AArch64 でも末尾ジャンプになる。`CanLowerReturn` はレジスタ部だけを見て真になり、
out-pointer は属性なしの引数なので降格判定に掛からない。

得はメモリに行く leaf が減ること。損はバッファ自体が消えないこと。`-O max` では SROA が
バッファごと消す (実測で `out@call_lambda` の alloca が最適化後 0 個) ので、効くとすれば
`-O basic` と、inline されない広い戻り値の呼び出しである。着手するならまず計測する。

## 検討して採らなかった案

- **LLVM のバージョンを上げる。** 17.0.6, 20.1.2, 21.1.8, 22.1.8 のいずれも、降格された戻り値
  では sibcall を諦め、閾値も同じ位置にある。LLVM 22 は **転送された `sret` ポインタ** について
  末尾呼び出しを獲得したが (`llvm/llvm-project` PR #146575、`release/22.x` と `main` に存在)、
  それは上記の属性なしポインタが LLVM 17 で既に達成していることである。上流には明示的な `sret`
  の側の報告が open で残っている (`llvm/llvm-project` issue #8605)。降格された戻り値の側は
  見当たらないので、報告する価値がある。
- **戻り値の閾値を上げる呼び出し規約。** `swifttailcc` は x86-64 の整数の閾値を 3 から 4 に
  上げるだけで止まる。`fastcc` と `tailcc` は戻り値の閾値を変えない (`tailcc` は引数の側を
  解消するので採用したが、それとは別の軸である)。
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
