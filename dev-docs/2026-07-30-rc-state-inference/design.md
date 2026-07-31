# Locality 推論の設計: 非 LOCAL の発生源からの値レベル汚染

`plan.md` の段階 1 の設計。実行時の参照カウント状態バイトは残し、RC IR 上の may 解析で
参照カウント操作ごとに `RcState::Local` を証明する。証明できなかった操作は今日の実行時
ディスパッチのままなので、諦める証明は何のコストも生まない。不正確さでプログラムが壊れる
ことはなく、壊れうるのは不健全さだけである。この文書はその不健全さが無いことを論証する。

## 性質

束縛 `x` とその型の boxed leaf パス `π` について:

> `shared(x.π)` — `x.π` が指すオブジェクト、**またはそこから到達可能な任意のオブジェクト**が、
> この束縛の生存中のどこかの時点で非 `LOCAL` 状態にありうる。

unit パス `π` に対する `Retain`/`Release` が `RcState::Local` を出せるのは、`π` 以下のすべての
boxed leaf `σ` で `shared(x.σ)` が偽のとき。

「生存中のどこかの時点で」により、この性質は時間的である: 後の操作が非 `LOCAL` にマークする
オブジェクトは、マークより前に実行される site でも shared と数える。マーク前の retain は実行時
には実際 `LOCAL` を読むのだが、「この site は必ずマークより前に走る」の証明はプログラムの
フローについての順序推論であり、site ごとの注釈 1 個は「その site がいつ走っても」成り立つ
必要があるので、定義は生存期間全体を織り込む。`threaded = false` のビルドではこの区別は空 —
生きている束縛のオブジェクトを遷移させる操作が無い（後述の論証）— なので前方データフローが
この性質を正確に計算でき、threaded の段階に escape 推論が要るのもこれが理由である。

到達閉包を取るのは意図的で、これが性質を合成的に
する。値からの射影、新しい集約への注入、boxed コンテナからの要素読み出しが、どれも
「オペランドの汚染の合併」になり、エイリアスの問いが消える。

閉包の代償は、共有された要素を保持する新品のコンテナが自身も共有扱いになること
（`let a = [g.@(0)]` — `a` のストレージは証明可能に local だが、解析は `a` を汚染する）。
エイリアスの問いを一切立てないことの対価であり、安全側に倒れる。

## 発生源

オブジェクトが `LOCAL` を離れる扉はちょうど 3 つ。状態バイトの書き手は 4 つ —
`create_obj`（`LOCAL` で初期化）、`mark_global_one`、`mark_threaded_one`、`mark_local_one`
（unique-threaded 経路での `THREADED` から `LOCAL` への絞り込み。共有を増やす側ではない）—
なので、マークする 2 つの書き手の呼び出し元を数え上げれば扉は尽きる:

1. **global 値の読み出し。** `implement_rc_global` は初期化子の値を評価し終えた後、その値が
   到達するグラフ全体に `mark_global` を掛ける。global シンボルを値として使う場所はすべて、
   そのマーク済みグラフの読み出しである。
2. **`Std::mark_threaded`。** 引数のグラフを `THREADED` にマークする。`threaded = false` の
   ビルドはコンパイル時に拒否するので、そこにはこの扉が存在しない。
3. **`Std::boxed_from_retained_ptr`。** 生ポインタから値を復元する。状態は何も分からない —
   スレッドを跨いだかもしれないし、global のグラフ由来かもしれない。

発生源でないことを確認したもの: `String::unsafe_from_c_str_ptr` は新しい配列へ複製する。
`FFI_EXPORT` は非集約スカラーしか許さない（#114）ので、エクスポート関数の引数から boxed 値は
入ってこない。C ランタイムは参照カウント対象を作らない。`argc`/`argv` は生スカラーで、
`Std::get_args` は新しい文字列を作る。`boxed_to_retained_ptr` は状態を変えずにポインタを
貸し出すだけで、値の帰り道は扉 3。将来の 4 つ目の扉は静的メモリの作業（issue #122 の追記）:
静的に確保されたストレージは `create_obj` を通らないので、その作業は状態を宣言し、この一覧を
見直す必要がある。

**タイミングにより、global の扉は「読み出し側」の扉になる。** `mark_global` は初期化子の値が
完成した後に走るので、初期化の*最中*に実行される参照カウント操作 — 初期化子の本体とそこから
呼ばれるすべての関数の中 — はまだ `LOCAL` なオブジェクトに対して行われる。よって汚染は
「global シンボルの読み出し」に付き、値を作ったコードには付かない。初期化子の本体も他のコードと
同じように解析・注釈できる。別の global の初期化子の中で global を読む場合、そのアクセサは先に
完走しているのでマーク済みであり、通常の読み出し規則が汚染する。

## 健全性は `threaded = false` に依存する

この解析は束縛の汚染を 1 回だけ決める。それが健全なのは、**既に束縛されエイリアスされた
オブジェクトを `LOCAL` から遷移させる操作が存在しない**場合だけである:

- `threaded = false` のビルドでは、マークする遷移は `mark_global` だけで、その対象は初期化子の
  結果グラフ。初期化子は引数を取らず global しか読めないので、生きているローカル束縛が指す
  オブジェクトがマークに巻き込まれることはない — グラフは初期化子が自分で作ったオブジェクト
  （他の誰も保持していない。初期化中に FFI へ生ポインタを退避しても、戻りは扉 3 を通る）と、
  他の global のオブジェクト（その時点でマーク済み、読み出しで汚染済み）から成る。
- `mark_threaded` は引数で壊す: `retain a; eval a.mark_threaded; ... a を使う` は `a` がまだ
  指しているオブジェクトをマークするが、`a` の汚染は呼び出しの前に決まっている。threaded
  ビルドで `Local` を証明するには「何が `mark_threaded` に流れ込みうるか」の escape 推論が
  必要 — threaded の段階（plan の段階 3、#96 待ち）ごと先送りする。

**段階 1 の注釈は `config.threaded` が偽のときだけ走る。** threaded ビルドは今日のまま全部
ディスパッチする。

## 束

boxed leaf ごとに origin の集合。`Origin ∈ { Ext, Arg(input, σ) }`。

- 空集合 = その leaf は local と証明された。
- `Ext` — 扉のどれかで汚染された、または解析が追わないもの（間接呼び出しの結果、単位外呼び出しの
  結果、単位の外から呼ばれうる関数）。
- `Arg(i, σ)` — 入力 `i`（パラメータ列、その後に capture）の leaf `σ` と同じだけ汚染。
  過渡的にのみ存在し、手続き間パスが呼び出し元の具体的な汚染に対して解決する。

join は合併。leaf パスは型で、入力は有限で抑えられるので、束は有限で不動点は停止する。

## 転送

RC IR ノードごとに、各ローカル束縛を leaf ごとの汚染に写す環境の上で。global シンボルを値と
して使う場所（「global アトム」）は読んだ値の全 leaf を `Ext` にする — これが扉 1 であり、
名前が local かどうかを見る**唯一の**規則である。

- `let x = y`（move）: コピー。
- `let x = <global アトム>`: 全 leaf `{Ext}`。（funptr 型の global は boxed leaf を持たず何も
  汚染しない。closure 型の global は capture leaf が `Ext` — その capture object はマーク
  されているので正しい。）
- `let x = Closure(f, caps)`: capture leaf に caps の全汚染の合併。
- `let x = App(callee, args)`:
  - callee がこの単位の `RcProgram` の関数を名指す — 直接呼び出し。手続き間の節へ。
  - それ以外（closure 値の変数、他単位の関数、global な closure 値）: 結果の全 leaf `{Ext}`。
- `let x = Llvm(op, args)`: op の *locality flow*（`LLVMGen` の新メソッド）で:
  - **デフォルト: 結果の各 leaf に、全オペランドの全 leaf 汚染の合併。`Ext` は加えない。**
    これは「新しいオブジェクトを確保するか、オペランドから到達可能なオブジェクトを並べ替える
    ことしかできない」操作すべてに対して健全 — つまり扉を除くすべての builtin。boxed
    コンテナからの読み出し（`array_get`、boxed struct の getter）は到達閉包によりデフォルトで
    正しい: 要素はコンテナから到達可能だった。
  - オーバーライド（各 op と同じ場所、`builtin.rs` に共置。op 固有属性と同じパターン）:
    - `boxed_from_retained_ptr`（と、念のため `mark_threaded`）: 全 leaf `{Ext}`。
    - unboxed 集約の配管 op — struct/tuple の make・get・set・mod・punch/plug-in、union の
      make/as/mod、capture projection — は leaf ごとに配線する（結果 leaf `.i.σ` ←
      オペランド leaf `σ` など）。汚染された成分と無汚染の成分を持つタプルを分けて保てる。
      ループ状態（`(tree, rng, sum)`）はここに住むので、hot loop が見る精度はこれで決まる。
      集合は列挙されていて小さく、各オーバーライドは数行。
  - flow を `result_prov` の読み替えにせず独立のメソッドにするのは、provenance が別の問いに
    答えているから（あちらの `Unknown` は「追跡していない共有」であって状態の共有ではない）。
    片方から導出すると、uniqueness の都合の編集が健全性の論証を静かに変える。
- `Destructure`: boxed コンテナ — 各フィールドにコンテナ leaf の汚染。unboxed — フィールド
  ごとに射影。
- `Match`: payload は `Destructure` と同様（variant ごと）。arm の結果は join。
- `Retain`/`Release`/`Eval`: 環境は不変。
- `ret x`: 関数の結果汚染に `x` を join。

## 手続き間: 単一の具体的不動点、monovariant

2 つの写像を、単位内の全関数の上で一緒に不動点まで回す:

- `input_taint[f]` — 入力 leaf ごとに、これまでに見た `f` の*直接*呼び出し site の引数汚染の
  join。下の保守的シードを加える。
- `result_taint[f]` — 現在の `input_taint[f]` の下での `f` の結果汚染。

各ラウンドで全関数の本体を現在の `input_taint` の下で解釈し、直接呼び出し site では
`result_taint` を使い、site の引数汚染を callee の `input_taint` へ join する。両写像は有限
束の中で単調に育つので停止する。記号的サマリは持たない: provenance が記号的なのは
uniqueness が特殊化で call site ごとに解決されるからで、locality は monovariant — 関数入力
ごとに 1 つの汚染を、呼び出し元全体で join する。

`input_taint` の保守的シード:

- 間接呼び出しで到達しうる関数 — どこかの `Closure(f, …)` が名指す関数 — は*パラメータ*
  leaf に `Ext`（capture leaf は閉包を作った site の汚染の join で、これは見える）。
- 単位の外から呼ばれうる関数は全入力 leaf に `Ext`。単一単位のビルドではその集合はエントリ
  ポイントと FFI エクスポート関数で、どちらの入力も boxed leaf を持たない（`main` は引数
  なし、エクスポートはスカラーのみ）ので、シードは空。複数単位のビルドではプログラムシンボル
  全部がシードされるが、単位ローカルなクローン — specialized・borrow・uncurry・decap 版、
  つまり hot loop が住む関数 — はプログラムシンボルではないので精度を保つ。これが分離
  コンパイルの正直な代償で、単位間サマリの保存は測定が要求したときの将来課題。

speedtest corpus は全ケース 1 単位でコンパイルされ、`-O max` の hot 経路は最後まで直接呼び
出しである — decapturing がループ本体の識別を specialized `fold` クローンに焼き込み、その
本体はループ本体を**名前で**呼ぶ（RC IR ダンプで確認: `fold#…#specialized_…` が
`main#…#decap_lam1#funptr3#borrow` を直接呼ぶ）。よって monovariant で重要な site に届き、
間接呼び出しの機構は不要。monovariance が負ける形（1 つのヘルパが汚染された引数と綺麗な
引数の両方で呼ばれる）への備えは、`unique_check_elim::specialize` のキーを広げる
polyvariance — まず測り、必要になってから作る。

## 注釈

不動点の後、各関数と各 global 初期化子本体をもう 1 回解釈し、各
`Retain(x, π, Unknown)` / `Release(x, π, Unknown)` で `π` 以下の全 leaf が無汚染なら状態を
`RcState::Local` にする。それ以外は `Unknown` のまま。

パイプラインでの位置: 最後の RC IR パス。`specialize` の後、`implement_rc_program` の直前 —
クローンが出揃い、参照カウント操作が最終形である必要がある。他の Max 以上のパスと同じゲート
に加えて `!config.threaded`。

## コード生成

`implement_rc_program` の `Retain`/`Release` アームは今 `Unknown` を assert している。`Local`
アームを足す:

- `Retain(Local)`: 非 atomic インクリメント。状態ロードなし、分岐なし（今日の `local_bb` の
  本体）。
- `Release(Local)`: 非 atomic デクリメント、読んだカウントが 1 なら破棄 — こちらも今日の
  local アームからディスパッチを外したもの。

null チェックの包み（`skip_null_check`、dynamic object のチェック）は直交で不変。破棄が呼ぶ
型 traverser の内部ディスパッチは `Unknown` のまま — 状態ごとの traverser 一族は生成コードを
倍にするので段階 1 ではやらない。

`is_unique` のディスパッチ（状態バイトの 3 番目の読み手、unique-check op の中）は**段階 2**:
読みは `LLVMGen::generate` の中で起きるので、注釈は op の属性として届ける必要がある —
`unique_check_elim` が証明済み unique オペランドのチェックを畳み込むのに既に使っている
共置属性のパターン。やる価値はある: `fannkuch` のディスパッチは 57% が `is_unique`、
`cp_lib_lsegtree` は 15%。段階 1 はこれ抜きで出して測る。

## コードだけでなく解析を検証する

- **`develop_mode` の実行時 assert**: `Local` と注釈されたすべての操作で状態バイトを読み、
  `REFCNT_STATE_LOCAL` でなければ abort。テストスイート全体が `develop_mode` で走るので、
  注釈された全 site が全テストプログラムで動的に検査される — plan の「特殊化された操作が
  主張を検査する」項目を、後からでなく段階と同時に納品する。わざと 1 site を誤注釈して
  スイートが落ちることを一度示し、その破壊を戻す。
- **カバレッジ測定**（一時プローブ、読んだら revert）: speedtest corpus で実行された
  `Local` / `Unknown` 操作を数え、`plan.md` の上限表（`arg`+`local` 行）と突き合わせる —
  monovariant の不動点が解決し損ねた割合が、数字付きの polyvariance 案件になる。
- **全スイート** 3 水準、**`benchmark/speedtest`** を現 `main` の行と比較。捨てた設計で
  裏返ったナイフエッジ（`nbody`、`nbody_fold`）を注視する。

## ファイル

| ファイル | 変更 |
| --- | --- |
| `src/rc_ir/locality.rs`（新規） | 束、転送、不動点、注釈 |
| `src/ast/inline_llvm.rs` | `LLVMGen::locality_flow`（合併デフォルト） |
| `src/fixstd/builtin.rs` | 扉のオーバーライドと集約配管のオーバーライド |
| `src/rc_ir/codegen.rs` | `Retain`/`Release` の `Local` アーム、`develop_mode` assert |
| `src/generator.rs` | 状態を見る retain/release 生成ヘルパ |
| `src/build/build_object_files.rs` | `specialize` の後に注釈を実行（Max 以上、非 threaded） |

`RcState::Local` とダンプの `@local` 形は既にある。`validate` は状態を見ない。

## 対象外

- threaded ビルド（plan の段階 3。上のエイリアス論証が理由）。
- `is_unique` site（段階 2、属性の配管）。
- 単位間サマリ。
- 状態ごとの traverser 一族。
- changelog: 観測可能な振る舞いは変わらない。
