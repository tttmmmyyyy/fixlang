# Locality 推論の設計: global 由来の値を前向きに追う

`plan.md` の測定を受けた設計。実行時の参照カウント状態バイトは残し、RC IR 上の may 解析で
参照カウント操作ごとに `RcState::Local` を証明する。

**誤りの 2 方向は対称ではない。** 証明を諦めて `MayExt` に倒すのは無害で、その操作は今日の
実行時ディスパッチのまま — だから精度はいくら落としても正しさは動かない。逆向きの誤り、
実際には global から到達できる値を `Local` と言う方は、その場でメモリを壊す。global
オブジェクトの参照カウントは維持されていない（`build_retain` の `global_bb` は何もしない）
一方で、`insert_rc` は global の読み出しに retain を入れず callee は release するので、
`plan.md` で測ったとおり読むたびにカウントが 1 ずつ減る勘定になっている。今はその release が
`global_bb` へ落ちて何もしないので釣り合っているが、`Local` と注釈された release は
ディスパッチせず直接デクリメントするため、最初の消費読み出しでカウントが 0 に落ち、global
オブジェクトが解放される。以後の読み出しは use-after-free になる。

したがってこの設計は全体として、**証明できたと明示的に言えるときだけ `Local`、それ以外は
すべて `MayExt`** という向きに倒れていなければならない。扉の数え上げ（発生源）・転送の
既定（転送）・手続き間の受け皿（`specialize`）は、どれもその向きを保つために書かれている。
以下、各所でその向きを明示する。

本文は段階 1（非 threaded ビルドの `Retain`/`Release`）を詳細化し、末尾の 2 節が段階 2
（`is_unique` サイト）と段階 3（threaded ビルド）を設計する。

## 性質

束縛 `x` とその型の boxed leaf パス `π` について、2 値を割り当てる:

> `Local` — `x.π` が指すオブジェクトも、**そこから到達可能な任意のオブジェクト**も、global から
> 到達可能ではない（= `REFCNT_STATE_LOCAL` である）ことが証明できた。
>
> `MayExt` — 証明できなかった。

`Local` ⊑ `MayExt` の 2 点束で、join は `MayExt` 側。unit パス `π` に対する `Retain`/`Release`
が `RcState::Local` を出せるのは、`π` 以下のすべての boxed leaf が `Local` のとき。

非 threaded ビルドでは、あるオブジェクトが global から到達可能かどうかは**そのオブジェクトが
できた時点で決まり、後から変わらない**（`mark_global` は global 初期化子の結果グラフに 1 回
掛かるだけで、生きている束縛のオブジェクトを巻き込まないことを次節で論証する）。よって性質は
時間を量化せず、値のフローを前から追うだけの may 解析で正確に計算できる。Fix にループは無く、
本体は直線とパターン分岐の木なので、1 本体の走査は前向き 1 パスで済む（分岐の合流は join、
再帰は次節のクローンのキーが受け持つ）。

到達閉包を取るのは意図的で、これが性質を合成的にする。値からの射影、新しい集約への注入、
boxed コンテナからの要素読み出しが、どれも「オペランドの join」になり、エイリアスの問いが
消える。**`MayExt` から取り出したものは必ず `MayExt`** というのがこの閉包の実務上の姿である。

閉包の代償は、`MayExt` な要素を保持する新品のコンテナが自身も `MayExt` になること
（`let a = [g.@(0)]` — `a` のストレージは実際には local だが、解析は `MayExt` にする）。
エイリアスの問いを一切立てないことの対価であり、安全側に倒れる。

**`unique_check_elim` と同じ形である。** あちらは provenance の記号的サマリを不動点で求め、
uniqueness をキーに関数を複製し、キーが証明するチェックをクローンの中で畳み込む。こちらは
locality の記号的サマリを不動点で求め、同じキーに locality を足して複製し、キーが証明する
site をクローンの中で `Local` に印を付ける。だから新パスを起こさず既存パスを広げる（後述）。

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
呼ばれるすべての関数の中 — はまだ `LOCAL` なオブジェクトに対して行われる。よって `MayExt` は
「global シンボルの読み出し」に付き、値を作ったコードには付かない。初期化子の本体も他のコードと
同じように解析・注釈できる。別の global の初期化子の中で global を読む場合、そのアクセサは先に
完走しているのでマーク済みであり、通常の読み出し規則が `MayExt` にする。

## 健全性は `threaded = false` に依存する

この解析は束縛の値を 1 回だけ決める。それが健全なのは、**既に束縛されエイリアスされた
オブジェクトを `LOCAL` から遷移させる操作が存在しない**場合だけである:

- `threaded = false` のビルドでは、マークする遷移は `mark_global` だけで、その対象は初期化子の
  結果グラフ。初期化子は引数を取らず global しか読めないので、生きているローカル束縛が指す
  オブジェクトがマークに巻き込まれることはない — グラフは初期化子が自分で作ったオブジェクト
  （他の誰も保持していない。初期化中に FFI へ生ポインタを退避しても、戻りは扉 3 を通る）と、
  他の global のオブジェクト（その時点でマーク済みで、読み出し規則が `MayExt` にする）から成る。
- `mark_threaded : a -> a` は引数で壊す。引数を消費して物理的に同一のオブジェクトへの
  ハンドルを返す op なので、マーク後も元の束縛を使うコードは

  ```
  retain a;                 -- a は下でも使う
  let b = a.mark_threaded;  -- 同じオブジェクトが THREADED になる
  ... a を使う ...          -- a の解析値は呼び出しの前に決まっており、Local のまま
  ```

  という形になる。`b` を `MayExt` にするだけでは足りず、同じオブジェクトを指す `a` が
  取り残される。threaded ビルドで `Local` を証明するには「何が `mark_threaded` に流れ込み
  うるか」の escape 推論が必要 — threaded の段階（plan の段階 3、#96 待ち）ごと先送りする。

**段階 1 の注釈は `config.threaded` が偽のときだけ走る。** threaded ビルドは今日のまま全部
ディスパッチする。

## 束

**クローンの中**（注釈をする場所）では、boxed leaf ごとに `Local` / `MayExt` の 2 値。入力の
値はクローンのキーから来る具体値で、本体の走査で伝播する。

**記号的サマリ**（相 1、まだ呼び出し元が決まっていない段階）では、leaf ごとに「この leaf が
`MayExt` になりうる理由」を持つ。理由は 2 種類しかない。

- `Ext` — 扉のどれかから来た、または解析が追わないもの（間接呼び出しの結果、単位外呼び出しの
  結果）。呼び出し元が何であっても `MayExt`。
- `Arg(i, σ)` — 入力 `i`（パラメータ列、その後に capture）の leaf `σ` 次第。クローンのキーが
  具体値を与えると解決する。

`Ext` は吸収元である。`Ext` を含む集合は入力が何であっても `MayExt` に解決するので、
そこに並ぶ `Arg` は結果を一切変えない。これを不変条件として手で保つのではなく、形で持つ:

```
enum LeafOrigins {
    Ext,                            // 入力に関わらず MayExt。束の頂
    Args(Set<(usize, FieldPath)>),  // この入力 leaf のどれかが MayExt なら MayExt
}
```

join は `Ext ⊔ _ = Ext`、`Args(a) ⊔ Args(b) = Args(a ∪ b)`。**`Args(∅)` が束の底 = `Local`**
で、その場で確保した値がこれになる — `create_obj` は `LOCAL` で初期化するので、理由が 1 つも
無い。確保したコンテナに `MayExt` な値を入れた場合は、その値の理由が転送規則（`Merge`）で
入ってくるので `Args(∅)` にはならない。leaf パスは型で、入力は有限で抑えられるので、束は
有限で不動点は停止する。

吸収を形で持つことは、複製のゲートの精度にそのまま効く。ゲートは「RC site が入力に依存しない
関数は複製しない」で、依存の判定は `Args(s)` の `s` が空でないこと。集合を素朴に合併すると
`{Ext, Arg(0, σ)}` が `Arg` を含むので「依存する」と読まれ、キーを変えても結果が動かない関数
が複製されてしまう。`Ext` に潰しておけばそれが起きない。

## 転送

RC IR ノードごとに、各ローカル束縛を leaf ごとの値に写す環境の上で。global シンボルを値と
して使う場所（「global アトム」）は読んだ値の全 leaf を `Ext` にする — これが扉 1 であり、
名前が local かどうかを見る**唯一の**規則である。

- `let x = y`（move）: コピー。
- `let x = <global アトム>`: 全 leaf `{Ext}`。（funptr 型の global は boxed leaf を持たないので
  何も起きない。closure 型の global は capture leaf が `Ext` — その capture object はマーク
  されているので正しい。）
- `let x = Closure(f, caps)`: capture leaf に caps の全 leaf の join。
- `let x = App(callee, args)`:
  - callee がこの単位の `RcProgram` の関数を名指す — 直接呼び出し。手続き間の節へ。
  - それ以外（closure 値の変数、他単位の関数、global な closure 値）: 結果の全 leaf `{Ext}`。
- `let x = Llvm(op, args)`: op の *locality flow*（`LLVMGen` の新メソッド、各 op と同じ場所に
  `builtin.rs` で共置。op 固有属性と同じパターン）。3 つの答えのどれかを返す:
  - **`Merge` — 結果の各 leaf に、全オペランドの全 leaf の join。`Ext` は加えない。**
    「新しいオブジェクトを確保するか、オペランドから到達可能なオブジェクトを並べ替える
    ことしかできない」操作に対して健全で、現在の builtin では扉 3 を除く全部がこれに当たる。
    boxed コンテナからの読み出し（`array_get`、boxed struct の getter）も `Merge` で正しい —
    到達閉包を取っているので、要素はコンテナから到達可能だった。
  - **`Ext` — 結果の全 leaf `{Ext}`。** `boxed_from_retained_ptr`（扉 3）と、念のため
    `mark_threaded`。
  - **`Wired` — leaf ごとに配線。** unboxed 集約の配管 op — struct/tuple の make・get・set・
    mod・punch/plug-in、union の make/as/mod、capture projection — は結果 leaf `.i.σ` ←
    オペランド leaf `σ` のように結ぶ。`MayExt` な成分と `Local` な成分を持つタプルを分けて
    保てる。ループ状態（`(tree, rng, sum)`）はここに住むので、hot loop が見る精度はこれで
    決まる。集合は列挙されていて小さく、各配線は数行。

  **このメソッドに既定実装を置かない。** `Merge` を既定にすると、オペランドから到達できない
  boxed オブジェクトを作る op を将来足したときに、何も書かなくても `Local` が通る — 冒頭で
  述べた「壊れる側」の誤りが黙って入ることになる。`Ext` を既定にすれば安全側だが、今度は
  書き忘れが黙って精度を殺し、しかも症状が出ないので気付けない。どちらの黙り方も避けたいので
  `LLVMGen` の必須メソッドにして、op を足す人に 3 つから必ず選ばせる。既存の 77 個の
  `impl LLVMGen` は大半が 1 行の `Merge` になる。
  - flow を `result_prov` の読み替えにせず独立のメソッドにするのは、provenance が別の問いに
    答えているから（あちらの `Unknown` は「追跡していない共有」であって状態の共有ではない）。
    片方から導出すると、uniqueness の都合の編集が健全性の論証を静かに変える。
- `Destructure`: boxed コンテナ — 各フィールドにコンテナ leaf の値。unboxed — フィールド
  ごとに射影。
- `Match`: payload は `Destructure` と同様（variant ごと）。arm の結果は join。
- `Retain`/`Release`/`Eval`: 環境は不変。
- `ret x`: 関数の結果に `x` を join。

## 手続き間: `specialize` のキー拡張による複製

2 相に分ける。

**相 1 — 記号的サマリ。** 関数ごとに、結果の各 leaf の origin 集合（`Ext` / `Arg(i, σ)`）を、
プログラム全体の不動点まで計算する。provenance の phase 1 と同型（有限束・単調 join・直接
呼び出しは callee のサマリを代入・間接と単位外は全 leaf `Ext`）。

**相 2 — 既存の `specialize`（`unique_check_elim`）のキーを広げる。** `SpecializationKey` は
今 `Vec<Uniqueness>`（パラメータごと）である。これを uniqueness と leaf ごとの locality の
組に広げる。specialize の構造はそのまま使える:

- 全関数の canonical 版（uniqueness は全 `Dynamic`、locality は全 `Ext`）を残す。間接呼び出し
  と単位外呼び出しの受け皿で、今日と同じく全部ディスパッチする。
- クローンの実体化が call site を歩き、引数の locality（呼び出し元クローンの具体的入力 +
  相 1 のサマリで解決）から callee のキーを組んで worklist に積む — uniqueness が今やって
  いるのと同じ流れ。
- `reaches_unique_check` に対応するゲートも同じ形で作る: RC site の値がどれも入力に依存
  しない関数（サマリに `Arg` が現れない）はキーで結果が変わらないので複製しない。

クローンの中では入力が**具体値**なので、1 つのヘルパが `MayExt` な引数と `Local` な引数の
両方で呼ばれても、それぞれのクローンが別々に証明される。monovariant（関数入力ごとに全呼び
出し元の join を 1 つ持つ）も検討したが、この混合文脈で丸ごと de-prove する弱点があり、
段階 2 が specialize の中で op を書き換える（後述）ことを考えると、locality も同じパスに
載っている方が接続が素直なので、複製を採る。クローン数は locality キーが実際に異なる関数
でしか増えない見込みで、実測で確かめる。

hot 経路がキーで届くことは確認済み: `-O max` では decapturing がループ本体の識別を
specialized `fold` クローンに焼き込み、その本体はループ本体を**名前で**直接呼ぶ（RC IR
ダンプで確認: `fold#…#specialized_…` が `main#…#decap_lam1#funptr3#borrow` を直接呼ぶ）。
また uncurry/decap で capture は普通のパラメータになっているので、closure 由来の値もキーの
対象に入る（specialize のキーが capture を除外するのは closure-ABI 版だけで、そちらは
canonical のまま — 今日の uniqueness と同じ扱い）。

単位の外から呼ばれうる関数（プログラムシンボル）は canonical しか参照されようがないので
自然に `Ext` 側に落ちる。単一単位のビルド（speedtest corpus は全ケースこれ）ではエントリ
ポイントと FFI エクスポートだけが外部から届き、どちらの入力も boxed leaf を持たないので、
何も失わない。単位間サマリの保存は測定が要求したときの将来課題。

## 注釈

クローンの実体化のとき、入力の具体値の下で本体を前向きに 1 回走査し、各
`Retain(x, π, Unknown)` / `Release(x, π, Unknown)` で `π` 以下の全 leaf が `Local` なら状態を
`RcState::Local` に書き換える。それ以外は `Unknown` のまま。global 初期化子本体は入力なしで
同様に解釈する（specialize が今 `&[]` でやっているのと同じ形）。

パイプラインでの位置: 相 1 のサマリ計算を `cancel` の後に足し、相 2 と注釈は**既存の
`specialize` の中**で行う。つまり
`… → borrow_ify → cancel → [locality サマリ] → specialize（キー拡張 + 注釈） → implement`。
ゲートは他の Max 以上のパスと同じものに加えて、locality 成分だけ `!config.threaded`
（threaded ではキーの locality を常に全 `Ext` にし、注釈もしない）。

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

`is_unique` のディスパッチ（状態バイトの 3 番目の読み手、unique-check op の中）は段階 2
（後述）。段階 1 はこれ抜きで出して測る。

## コードだけでなく解析を検証する

- **`develop_mode` の実行時 assert**: `Local` と注釈されたすべての操作で状態バイトを読み、
  `REFCNT_STATE_LOCAL` でなければ abort。冒頭の「壊れる側」の誤りを、静かなメモリ破壊から
  その場の abort に変えるもので、解析の穴に対する唯一の実効的な防御なので、段階 1 と同時に
  入れる（後追いにしない）。テストスイート全体が `develop_mode` で走るので、注釈された全
  site が全テストプログラムで動的に検査される。わざと 1 site を誤注釈してスイートが落ちる
  ことを一度示し、その破壊を戻す。
- **カバレッジ測定**（一時プローブ、読んだら revert）: speedtest corpus で実行された
  `Local` / `Unknown` 操作を数え、`plan.md` の上限表（`arg`+`local` 行）と突き合わせる。
  併せてクローン数（specialize の出力関数数）を拡張の前後で比べる。
- **全スイート** 3 水準、**`benchmark/speedtest`** を現 `main` の行と比較。捨てた設計で
  裏返ったナイフエッジ（`nbody`、`nbody_fold`）を注視する。

## ファイル

| ファイル | 変更 |
| --- | --- |
| `src/rc_ir/locality.rs`（新規） | 束、転送、相 1 の記号的サマリ |
| `src/ast/inline_llvm.rs` | `LLVMGen::locality_flow`（合併デフォルト） |
| `src/fixstd/builtin.rs` | 扉のオーバーライドと集約配管のオーバーライド |
| `src/rc_ir/unique_check_elim.rs` | キー拡張、クローン実体化時の注釈 |
| `src/rc_ir/codegen.rs` | `Retain`/`Release` の `Local` アーム、`develop_mode` assert |
| `src/generator.rs` | 状態を見る retain/release 生成ヘルパ |
| `src/build/build_object_files.rs` | `cancel` の後にサマリ計算を差し込む |

`RcState::Local` とダンプの `@local` 形は既にある。`validate` は状態を見ない。

## 段階 2 — `is_unique` サイト

状態バイトの 3 番目の読み手は uniqueness チェック（`build_branch_by_is_unique`。配列の `set`
が in-place 更新できるか調べる所など）で、読みは unique-check op の `LLVMGen::generate` の
**中**で起きる。`Retain`/`Release` と違って RC IR ノードに `RcState` フィールドが無いので、
注釈の結果は op インスタンスの属性フィールドとして届ける — `unique_check_elim` が証明済み
unique オペランドのチェックを畳み込むのに使っているのと同じ、対象 op の struct にフィールド
を足して書き込むパターン。書き込む場所も同じクローン実体化の中なので、段階 1 が specialize
に載っていればここは配管だけになる。

やる価値: `fannkuch` のディスパッチは 57% が `is_unique`、`cp_lib_lsegtree` は 15%。段階 1 の
実測を見てから足す。

## 段階 3 — threaded ビルド

同じ束を `THREADED` に向ける。証明が通れば atomic RMW が非 atomic の増減になり、分岐 1 個を
消すより 1 操作あたりの取り分が大きい。追加で要るのは「性質」の節で述べた時間性への対処で、
問題の形はこうなる:

```
let a = ...;              -- a の値はここで決まる（Local）
retain a;
let b = a.mark_threaded;  -- b は Ext。だがオブジェクトは THREADED になった
release a;                -- a は Local のまま → Local 注釈 → 非 atomic release。不健全
```

結果だけを `Ext` にする forward 伝播では、マークされたオブジェクトに届く既存の束縛
（エイリアス）が漏れる。エイリアスはオリジン（確保点・パラメータ）を共有するので、
**オリジンを `Ext` にする**のがエイリアス解析なしで漏れなく覆う最も粗い過大近似
（「mark_threaded に流れ込みうる値は生まれた時点から `Ext`」）。機構は `borrow_ify` の
`infer_ownership` の第 3 の実例として作れる: あちらは consume site を種に、`origin`
（本体内の逆向き値追跡）でパラメータまで遡り、own フラグをプログラム全体の不動点まで単調に
育てる。こちらは種を `mark_threaded` op のオペランドに、フラグを「mark_threaded に流れ込み
うる」に替える。`origin` の遡り先にはパラメータ（手続き間の伝播）とローカルの確保束縛
（`Ext` の付け先）の両方が出る。

ただしオリジン汚染は保守的すぎる面がある: マークより**前**に実行される操作は実行時には
LOCAL を見るので、本当は Local にできる。とくに「単スレッドで構築してから公開する」形では
構築フェーズの操作が全部それに当たる。これはフロー感度の精密化で健全に拾える —
**値が公開されうる位置（`mark_threaded` のオペランド、または公開されうる値への取り込み）に
まだ流れていない区間**では、オブジェクトを保持するのは現スレッドだけなので Local を出せる。
窓が閉じるのはマークの時点ではなく「公開されうる値に取り込まれた時点」（コンテナ経由の
間接マークがあるため）。ループ・再帰でマークを跨ぐ site — ある反復でマークされ次の反復で
同じ site が触る形 — は、再帰呼び出しがパラメータに「公開済みかもしれない」を join する
ことで自動的に `Ext` 側へ落ちる。

ベースライン = オリジン汚染、精密化 = 未 escape 窓、の 2 層で設計し、精密化は構築フェーズの
実測が要求してから足す。誤証明はデータ競合で単スレッドのテストには見えないので、#96 の競合
検出を待つ。`develop_mode` の assert（状態バイトの検査）はそのままここでも安全網になる。

**escape 推論が要るのは `mark_threaded` だけである。** 3 つの扉のうち、既存の束縛が指している
オブジェクトの状態を変えるのはこれだけで、`mark_global` は初期化子の結果グラフにしか掛からず
（前述）、`boxed_from_retained_ptr` は新しいハンドルを作るだけなので、どちらも前向きの規則で
足りる。したがって、スレッド間で共有せず複製するモードを入れて `mark_threaded : a -> a` を
deep clone として実装する場合 — 引数のオブジェクトは `LOCAL` のまま、結果は新しいグラフ —
この節の escape 推論は不要になり、threaded ビルドの locality は段階 1 と同じ前向き 1 パスに
なる。シグネチャが値を返す形になっているので、この差し替えは呼び出し側から見て互換である。

## 対象外

- 単位間サマリ。
- 状態ごとの traverser 一族。
- changelog: 観測可能な振る舞いは変わらない。
