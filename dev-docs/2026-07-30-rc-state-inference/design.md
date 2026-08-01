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

対象は `config.threaded` が偽のビルドの全 RC site。threaded ビルドへの拡張は `threaded.md`。

## 性質

束縛 `x` とその型の boxed leaf パス `π` について、**3 点鎖**の値を割り当てる:

> `DeepLocal` — `x.π` が指すオブジェクトも、**そこから到達可能な任意のオブジェクト**も
> `REFCNT_STATE_LOCAL` であることが証明できた。
>
> `Local` — `x.π` が指すオブジェクト**自身**は `REFCNT_STATE_LOCAL` であることが証明できた。
> そこから到達できるオブジェクトについては何も言わない。
>
> `MayExt` — 何も証明できなかった。

`DeepLocal ⊑ Local ⊑ MayExt` で、join は `MayExt` 側。

**2 つの事実を分けるのが要点である。** 参照カウント操作は浅い — `Retain` は根オブジェクトの
カウントを増やすだけ、`Release` は根のカウントを減らし、0 になったときだけ型 traverser が
子を release する（その内部操作は `Unknown` のままディスパッチする）。`is_unique` も根の状態
バイトを読むだけ。**したがって注釈に必要なのは根の事実だけで、下の事実は要らない。** 下の事実
が要るのは、コンテナから取り出した値について言うときである。

```
let a = [g, g];   -- g は global。a のストレージは新規確保なので LOCAL
                  -- a は `Local`（根は LOCAL、下は分からない）
retain a;         -- 根だけの操作なので Local 注釈でよい
let x = a.@(0);   -- a が `Local` 止まりなので、取り出した x は `MayExt`
```

これが 3 点にする理由のすべてである。2 点（到達閉包だけ）にすると `a` 自身が `MayExt` に落ち、
`a` に対する retain/release/`is_unique` がすべてディスパッチに戻る。逆に根だけを追うと、
`DeepLocal` な配列から取り出した要素まで `MayExt` になり、`plan.md` が測った取りこぼしの
主因（boxed コンテナからの読み出し）がそのまま残る。3 点鎖は両方を保つ。

site が状態ディスパッチを外せるのは、**その site が触る leaf がすべて `Local` 以下**のとき
（`DeepLocal` か `Local`）— `Retain`/`Release` なら unit パス `π` 以下の全 leaf、`is_unique`
ならチェック対象の leaf、`Destructure` ならノードが retain/release する leaf 全部（「注釈する
site」の節）。

非 threaded ビルドでは、あるオブジェクトが global から到達可能かどうかは**そのオブジェクトが
できた時点で決まり、後から変わらない**（`mark_global` は global 初期化子の結果グラフに 1 回
掛かるだけで、生きている束縛のオブジェクトを巻き込まないことを次節で論証する）。よって性質は
時間を量化せず、値のフローを前から追うだけの may 解析で正確に計算できる。Fix にループは無く、
本体は直線とパターン分岐の木なので、1 本体の走査は前向き 1 パスで済む（分岐の合流は join、
再帰は次節のクローンのキーが受け持つ）。

**根が非 LOCAL になる道はちょうど 3 つ（次節の扉）で、それ以外の操作は根を `Local` に保つ。**
新規確保は `create_obj` が `LOCAL` で初期化し、in-place 更新も clone も根を動かさない。一方
**下の事実は、非 `DeepLocal` な値をコンテナに入れれば普通に壊れる** — `a.set(0, g)` でも、
生ポインタ経由の `mutate_elements` でも同じことが起きる。2 つの事実を分けたので、この 2 つは
同じ 1 つの規則（「入れたものの分だけ下が汚れる」）で扱えて、どちらも扉にはならない。

**`unique_check_elim` と同じ形である。** あちらは provenance の記号的サマリを不動点で求め、
uniqueness をキーに関数を複製し、キーが証明するチェックをクローンの中で畳み込む。こちらは
locality の記号的サマリを不動点で求め、locality をキーに複製し、キーが証明する site を
クローンの中で `Local` に印を付ける。同じ骨格の、別のパスになる（後述）。

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

**扉は根の話である。** コンテナに何を入れても、コンテナ自身のオブジェクトの状態バイトは
動かない。`a.set(0, g)` も、生ポインタ経由の `mutate_boxed` / `mutate_elements` も、汚すのは
`deep` だけで `root` は `Local` のまま — 前者はオペランドから見えるので普通の転送規則で、後者は
「要素型・payload 型に boxed leaf があれば結果の `deep` を `Always`」という 1 行の規則で覆える。
どちらも扉ではない。（`borrow_boxed` / `borrow_elements` は「借りたポインタを通して変更しては
ならない」と `std.fix` が定めているので、`deep` すら汚さない。）

発生源でないことを確認したもの: `String::unsafe_from_c_str_ptr` は新しい配列へ複製する。
`FFI_EXPORT` は boxed 値を不透明ポインタとして受け渡せる（`has_c_abi` が `is_box` を許す。
#114 が禁じたのは集約型と `Bool`）ので、エクスポート関数の引数には状態の分からない boxed 値が
入ってくるが、**どのオブジェクトの状態も変えない**ので扉ではない。未知の状態は手続き間の節で
受ける。C ランタイムは参照カウント対象を作らない。`argc`/`argv` は生スカラーで、
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
  結果グラフ全体（結果の値から到達可能なオブジェクトすべて）である。初期化子は**遅延実行**で、
  その global を最初に読んだ時点、つまり呼び出し元のフレームが生きたまま中断している状態で
  走る。したがって「初期化子の結果グラフに、生きている束縛が指すオブジェクトが含まれない」
  ことを言う必要がある。

  初期化子が名指せる値を数え上げると、これは言える:

  1. **引数を取らない。** 呼び出し元の値は届かない。
  2. **他の global。** 読んだ時点でそのアクセサは完走しており、マーク済み。読み出し規則が
     `MayExt` にするので、`Local` と証明された束縛ではない。
  3. **自分で確保したオブジェクト。** まだ誰にも渡していないので他に保持者がいない。

  純粋な Fix のコードで書ける初期化子は 1-3 で尽きるので、**この前提は成り立つ**。

  **抜け道は unsafe な経路 1 つだけである。** 外部が保持しているハンドルを持ち込む
  `boxed_from_retained_ptr`（`[a : Boxed]` なので boxed 型に限る）か、Fix のオブジェクトを
  返す FFI 呼び出しを初期化子の中で使うと、4 番目の入口ができる。形としては
  「`main` が `x.boxed_to_retained_ptr` で C にポインタを預け、その後で読まれた global の
  初期化子がそれを復元して結果に含める」。壊れ方は**混ざった対**で、`array_get` などの op
  内部の retain は実行時にディスパッチして GLOBAL になった今は no-op になる一方、RC IR の
  `Release` ノードは `Local` 注釈のまま実カウントを減らすので、読むたびにカウントが 1 減り、
  まだ参照されているのに解放される。

  そこで「**global の初期化子は、外部が保持している Fix オブジェクトを結果グラフに取り込んでは
  ならない**」を契約として課し、`boxed_from_retained_ptr` のドキュメントにも書く。純粋な Fix
  では表現できないことを禁じるだけなので、書けるプログラムは 1 つも失われない。P2 を担保して
  いるのは上の数え上げとこの契約である。

  **「扉は読み出し側に付く」の論証はここを覆わない。** あちらは *初期化子の中で* 実行される
  操作の話（マークより前なのでまだ `LOCAL` を見る）で、ここで問題になるのは *初期化子の外で*、
  中断している呼び出し元の、既に注釈済みの操作である。

  初期化を遅延でなく `main` の前に一括で行う形にすれば、初期化中に生きている束縛が存在しえない
  ので契約なしに閉じる（#156 で定数初期化子を静的データにする作業は、その global について
  `mark_global` 自体を消すので、この穴を狭める方向に働く）。

- `mark_threaded : a -> a` は引数で壊す。引数を消費して物理的に同一のオブジェクトへの
  ハンドルを返す op なので、マーク後も元の束縛を使うコードは

  ```
  retain a;                 -- a は下でも使う
  let b = a.mark_threaded;  -- 同じオブジェクトが THREADED になる
  ... a を使う ...          -- a の解析値は呼び出しの前に決まっており、Local のまま
  ```

  という形になる。`b` を `MayExt` にするだけでは足りず、同じオブジェクトを指す `a` が
  取り残される。threaded ビルドで `Local` を証明するには「何が `mark_threaded` に流れ込み
  うるか」の escape 推論が必要 — `threaded.md` へ送る。

**注釈は `config.threaded` が偽のときだけ走る。** threaded ビルドは今日のまま全部
ディスパッチする。

## 束と用語

**層が 2 つある。** 解決後の層（クローンの中。呼び出し元が決まっていて、値が具体的）と、
記号的な層（相 1 のサマリと `locality_flow`。まだ呼び出し元が決まっていない）。既存の
provenance / uniqueness と同じ 2 層構造で、名前も層ごとに分ける:

| | leaf 1 個 | 値 1 個 | 対応する既存の型 |
| --- | --- | --- | --- |
| **解決後** | `Locality` = `DeepLocal` / `Local` / `MayExt` | `LocalityKey(Map<FieldPath, Locality>)` | `SharingVerdict` / `Uniqueness` |
| **記号的** | `LeafCond` = `root` と `deep` の 2 条件 | `ExtShape(Map<FieldPath, LeafCond>)` | `LeafOrigins` / `Provenance` |

```
enum Locality { DeepLocal, Local, MayExt }

/// 1 つの条件。「これが成り立つと非 LOCAL」。
enum ExtCond {
    /// 入力に関わらず成り立つ。条件の束の頂。
    Always,
    /// 挙げた入力 leaf のどれかが該当すれば成り立つ。空集合が底で、決して成り立たない。
    IfAny(Set<(usize, FieldPath)>),
}

/// leaf 1 個の記号的な値。不変条件 `root ⊑ deep` を構成で保つ
/// （`deep` を書くときは常に `root` と join する）。
struct LeafCond {
    root: ExtCond,   // この leaf のオブジェクト自身が非 LOCAL になる条件
    deep: ExtCond,   // この leaf から到達できる先に非 LOCAL が居る条件
}
```

**この文書では、`DeepLocal`/`Local`/`MayExt` は解決後の層でだけ、`Always`/`IfAny` は記号的な
層でだけ使う。** `ExtCond::Always` は「無条件に成り立つ」の意味であって、`MayExt` の別名では
ない。

`IfAny` の添字 `(i, σ)` は、サマリでは関数の入力 `i`（パラメータ列、その後に capture）の
leaf `σ`、`locality_flow` では op のオペランド `i` の leaf `σ` を指す。`Provenance` が
`LeafOrigin::Arg(i, path)` を 2 つの索引空間で使い回しているのと同じ。条件の中で入力 leaf を
「該当する」と判定する基準は、`root` 側なら入力が `MayExt` であること、`deep` 側なら入力が
`DeepLocal` でないことである。

解決は `resolve(LeafCond, inputs: &[LocalityKey]) -> Locality`:

| `root` | `deep` | 結果 |
| --- | --- | --- |
| 成り立たない | 成り立たない | `DeepLocal` |
| 成り立たない | 成り立つ | `Local` |
| 成り立つ | （不変条件より成り立つ） | `MayExt` |

### `Always` は吸収元

`Always` を含む条件は入力が何であっても成り立つので、そこに並ぶ入力 leaf は結果を一切変え
ない。これを不変条件として手で保つのではなく、`ExtCond` の形で持つ。join は
`Always ⊔ _ = Always`、`IfAny(a) ⊔ IfAny(b) = IfAny(a ∪ b)`。`LeafCond` の join は成分ごと。

`root = IfAny(∅)` になるのはその場で確保した値と、確保済みのコンテナを並べ替えただけの値
である — `create_obj` は `LOCAL` で初期化し、扉以外は状態バイトを動かさない。
`deep = IfAny(∅)` になるのは、さらに中身も `DeepLocal` なものだけを入れた場合。leaf パスは型で、
入力は有限で抑えられるので、束は有限で不動点は停止する。

吸収を形で持つことは、複製のゲートの精度にそのまま効く。ゲートは「RC site が入力に依存しない
関数は複製しない」で、依存の判定は `IfAny(s)` の `s` が空でないこと。原始的に「条件の集合」
として持って合併すると、`{Always, (0, σ)}` が入力への言及を含むので「依存する」と読まれ、
キーを変えても結果が動かない関数が複製されてしまう。`Always` に潰しておけばそれが起きない。

## 転送

RC IR ノードごとに、各ローカル束縛を `ExtShape` に写す環境の上で。**規則は記号的な層で 1 回だけ
定義する。** クローンの中の走査は同じ規則を走らせて、入力の `ExtCond` をキーで解決したもので
ある — 入力が具体値なので、環境の各条件は「成り立つ」か「成り立たない」に潰れ、実装上は
leaf ごとに 3 値を運ぶ走査になる。

**変数の参照はすべて名前の局所性を見る。** 環境は局所束縛だけを持ち、**環境に無い名前
（global シンボル）は全 leaf `Always` に解決する**。これが扉 1 であり、名前が local かどうかを
見る唯一の規則である。global は `let` の右辺に限らず、Llvm/App のオペランド、`Match` の
被検査値、`Destructure` のコンテナ、`Ret` のどこにでも現れる（`lower_var` は global を束縛せず
アトムのまま渡す。RC IR ダンプで `destructure Main::gpair#… { .0 -> … }` と
`array_lit(Main::garr#…, Main::garr#…)` を確認済み）ので、規則はノード種別ごとではなく
**参照の解決 1 か所**に置く。`Retain`/`Release` の対象が局所であることは `insert_rc` が保証
しているが、解決規則はそれを覚えていなくても安全な側に倒れる形にする。

- `let x = y`（move）: コピー。
- global アトムの参照: 全 leaf `Always`。（funptr 型の global は boxed leaf を持たないので
  何も起きない。closure 型の global は capture leaf が `Always` — その capture object はマーク
  されているので正しい。）
- `let x = Closure(f, caps)`: capture leaf は `root = IfAny(∅)`（capture object は新規確保）、
  `deep` は caps の全 leaf の `root ⊔ deep` の join。
- `let x = App(callee, args)`:
  - callee がこの単位の `RcProgram` の関数を名指す — 直接呼び出し。手続き間の節へ。
  - それ以外（closure 値の変数、他単位の関数、global な closure 値）: 結果の全 leaf `Always`。
- `let x = Llvm(op, args)`: op の `locality_flow`。次節。
- `Destructure`: boxed コンテナ — **取り出し規則**。各フィールドの全 leaf に
  `root = deep = コンテナ leaf の deep`。unboxed — フィールドごとに射影（結果 leaf `σ` <-
  コンテナ leaf `[i]++σ`、`root`/`deep` ともそのまま）。
- `Match`: payload は `Destructure` と同様（variant ごと）。arm の結果は join。
- `Retain`/`Release`/`Eval`: 環境は不変。
- `ret x`: 関数の結果に `x` を join。

**boxed コンテナからの取り出しは独立の規則である。** 取り出した値の**根**は、コンテナの
`deep`（コンテナの下に非 LOCAL が居るか）で決まる:

```
DeepLocal なコンテナから取り出す  ->  DeepLocal
Local なコンテナから取り出す      ->  MayExt
MayExt なコンテナから取り出す     ->  MayExt
```

記号的には `結果の root = 結果の deep = コンテナ leaf の deep`。2 点束のときは到達閉包の
おかげでこれが「オペランドの join」に潰れていたが、根と下を分けた以上、取り出しは合流ではなく
**下から根への昇格**なので別扱いになる。該当するのは `array_get`、boxed コンテナに対する
`Destructure` / `struct_get` / `union_as` / capture projection、boxed union の `Match` payload。

## `locality_flow`

`LLVMGen` に足すメソッド。`result_prov` と同じ引数を取る — 配線が型に依存する（コンテナが
boxed か unboxed かで結果 leaf の出どころが変わる）ため。

```rust
fn locality_flow(
    &self,
    result_ty: &Arc<TypeNode>,
    arg_tys: &[Arc<TypeNode>],
    type_env: &TypeEnv,
) -> ExtShape;

impl ExtShape {
    /// 確保と並べ替え。結果の全 leaf は `root = IfAny(∅)`（新しい/既存のコンテナの根は LOCAL）、
    /// `deep` は全オペランドの全 leaf の `root ⊔ deep` の join。
    fn merge(result_ty, arg_tys, type_env) -> ExtShape;
    /// 結果の全 leaf が `root = deep = Always`。
    fn always(result_ty, type_env) -> ExtShape;
    /// 結果の全 leaf が `root = deep = IfAny(∅)`。発散して値を返さない op 用。
    fn bottom(result_ty, type_env) -> ExtShape;
    /// 結果 leaf ごとに指定する。`Provenance::build_shape` と同じ形で、結果型の boxed leaf
    /// を全部歩いて呼ぶので、leaf の書き落としが起こらない。取り出し規則
    /// （`root = deep = コンテナの deep`）もここで書く。
    fn build_shape(result_ty, type_env, f: &dyn Fn(&FieldPath) -> LeafCond) -> ExtShape;
}
```

**サマリと同じ `ExtShape` を返すことに意味がある。** `Llvm` ノードの転送は「op が宣言した
写像にオペランドの値を代入する」で、直接呼び出しの転送は「callee のサマリに引数の値を代入
する」。`IfAny` の添字が指す先が違うだけで操作は同一なので、代入は 1 つ書けば両方が使う。
`merge` / `always` はその写像を作る構成子であって、別扱いの分岐ではない。

`merge` が健全なのは「新しいオブジェクトを確保するか、オペランドから到達可能なオブジェクトを
並べ替えることしかできない」op に対してだけである。`root = IfAny(∅)` と置けるのは、扉以外の
どの op も状態バイトを動かさないから。**boxed コンテナからの読み出しは `merge` では書けない** —
取り出した値の根はコンテナの `deep` から来るので、`build_shape` で取り出し規則を書く。

**既定実装を置かない。** `merge` を既定にすると、オペランドから到達できない boxed オブジェクト
を作る op を将来足したときに、何も書かなくても `Local` が通る — 冒頭で述べた「壊れる側」の誤りが
黙って入ることになる。`always` を既定にすれば安全側だが、今度は書き忘れが黙って精度を殺し、症状が
出ないので気付けない。どちらの黙り方も避けたいので必須メソッドにして、op を足す人に必ず選ばせる。

`result_prov` の読み替えにせず独立のメソッドにするのは、provenance が別の問いに答えているから
（あちらの `Unknown` は「追跡していない共有」であって状態の共有ではない）。片方から導出すると、
uniqueness の都合の編集が健全性の論証を静かに変える。

### 全 op の値

`impl LLVMGen` は 77 個。構成子で分類すると `always` 7 個、`bottom` 2 個、`build_shape` 14 個、
`merge` 54 個。

**`always`（8）** — 結果が解析の外から来る。

| op | 理由 |
| --- | --- |
| `InlineLLVMBoxedFromRetainedPtrIOS` | 扉 3。生ポインタから値を復元する |
| `InlineLLVMMarkThreadedFunctionBody` | 扉 2 |
| `InlineLLVMFixBody` | 関数オペランドを呼び、その結果を返す |
| `InlineLLVMWithRetainedFunctionBody` | 同上 |
| `InlineLLVMArrayBorrowElementsBody` | 同上 |
| `InlineLLVMUnionModBody` | payload に関数を適用し、その結果を union に入れる |
| `InlineLLVMFFICallBody` | C を呼ぶ |

**`bottom`（2）** — 発散するので結果の値が存在しない。`unreachable` を出すか abort するので、
何を主張しても空虚に真であり、束の底（全 leaf `IfAny(∅)`）が健全かつ最も精密である。頂に
すると、`if bad { undefined(msg) } else { v }` のようなガードで arm の join が汚染され、
`msg` の leaf が `v` に伝わってしまう。`result_prov` が `undefined` に `uniform_bottom` を
使っているのと同じ理由。

| op | |
| --- | --- |
| `InlineLLVMUndefinedInternalBody` | abort する |
| `InlineLLVMHoleBody` | `unreachable` を出す |

関数オペランドを呼ぶ op がこの表の半分を占める。呼ばれる関数は別の `RcFunc` として解析される
が、この op から見ると結果は間接呼び出しの結果であり、関数本体が global を読んで返すことを
`merge` は捉えられない。**`merge` を既定にしていたら、この 5 個が黙って `Local` を通していた。**

**`build_shape`（14）** — 集約の配管と、コンテナからの取り出し。成分ごとに別の値を保つ。

| op | 配線 |
| --- | --- |
| `InlineLLVMStructGetBody` | unboxed コンテナ: 結果 `σ` <- 引数 0 の `[field_idx]++σ`（成分そのまま）。boxed コンテナ: 取り出し規則 — 結果の全 leaf に `root = deep = 引数 0 の [] の deep` |
| `InlineLLVMMakeStructBody` | 結果 `[i]++σ` <- 引数 `i` の `σ`。boxed 結果は `merge` と同じ（全オペランドの join） |
| `InlineLLVMStructSetBody` | 結果 `[field_idx]++σ` <- 引数 0（値）の `σ`、他の leaf <- 引数 1（struct）の同じパス。boxed 結果は `merge` と同じ（全オペランドの join。値の側を落とさない） |
| `InlineLLVMStructPlugInBody` | 結果 `[field_idx]++σ` <- 引数 1（field）の `σ`、他 <- 引数 0（punched）。boxed 結果は `merge` と同じ（全オペランドの join） |
| `InlineLLVMStructPunchBody` | 結果は `(field, punched_struct)`（`PUNCHED_STRUCT_FIELD = 1`）。unboxed コンテナ: `[0]++σ` <- 引数 0 の `[field_idx]++σ`、`[1]` 以下 <- 引数 0 の残り。boxed コンテナ: 両成分 <- 引数 0 の `[]` |
| `InlineLLVMArrayUnsafeGetBoundsUnchecked` | 取り出し規則 — 結果の全 leaf に `root = deep = 引数の配列 leaf の deep` |
| `InlineLLVMArrayPunchBody` | 結果は `(PunchedArray a, a)`（`PUNCHED_ARRAY_FIELD = 0`。`StructPunch` とは順序が逆）。両成分 <- 引数の配列 leaf |
| `InlineLLVMMakeUnionBody` | unboxed union: 結果 `[variant]++σ` <- 引数 0 の `σ`、他の variant は底（`IfAny(∅)`）。boxed union: 結果の唯一の leaf `[]` <- 引数 0 の全 leaf |
| `InlineLLVMUnionAsBody` | unboxed union: 結果 `σ` <- 引数 0 の `[variant]++σ`。boxed union: 取り出し規則 |
| `InlineLLVMCaptureProjectBody` | boxed capture: 取り出し規則。unboxed `#CapList`: 射影 |
| `InlineLLVMUnsafeMutateBoxedInternalFunctionBody` | `[0]`（値）以下 <- 引数の値。ただし payload 型に boxed leaf があれば、その `deep` を `Always`（コールバックが生ポインタ経由で参照を書き込みうる。`root` は動かない）。残り（コールバックの結果）は `Always` |
| `InlineLLVMUnsafeMutateBoxedIOSInternalBody` | 同上、値の位置は `[1, 0]` |
| `InlineLLVMArrayMutateElementsInternalBody` | `[0]`（配列）<- 引数の配列。ただし要素型に boxed leaf があれば、その `deep` を `Always`。残りは `Always` |
| `InlineLLVMArrayMutateElementsIosInternalBody` | 同上、配列の位置は `[1, 0]` |

配線の位置は `result_prov` が使っている定数と同じもの（`STRUCT_SET_VALUE_ARG`,
`PLUG_IN_PUNCHED_ARG`, `PUNCHED_STRUCT_FIELD`, `MUTATE_BOXED_VALUE_FIELD` など）を読む。

**`merge`（56）**

- スカラーのみ（オペランドにも結果にも boxed leaf が無いので配線は空。29）:
  `InlineLLVMIntLit`, `InlineLLVMFloatLit`, `InlineLLVMNullPtrLit`,
  `InlineLLVMCastIntegralBody`, `InlineLLVMCastFloatBody`, `InlineLLVMCastIntToFloatBody`,
  `InlineLLVMCastFloatToIntBody`, `InlineLLVMShiftBody`, `InlineLLVMBitwiseOperationBody`,
  `InlineLLVMBitNotBody`, `InlineLLVMIntEqBody`, `InlineLLVMPtrEqBody`, `InlineLLVMFloatEqBody`,
  `InlineLLVMIntLessThanBody`, `InlineLLVMFloatLessThanBody`, `InlineLLVMIntLessThanOrEqBody`,
  `InlineLLVMFloatLessThanOrEqBody`, `InlineLLVMIntAddBody`, `InlineLLVMFloatAddBody`,
  `InlineLLVMIntSubBody`, `InlineLLVMFloatSubBody`, `InlineLLVMIntMulBody`,
  `InlineLLVMFloatMulBody`, `InlineLLVMIntDivBody`, `InlineLLVMFloatDivBody`,
  `InlineLLVMIntRemBody`, `InlineLLVMIntNegBody`, `InlineLLVMFloatNegBody`,
  `InlineLLVMBoolNegBody`
- 確保のみ（オペランドの leaf を join する。オペランドに leaf が無ければ `IfAny(∅)` = `Local`。5）:
  `InlineLLVMStringBuf`, `InlineLLVMArrayUnsafeEmpty`, `InlineLLVMArrayLitBody`,
  `InlineLLVMIOStateUnsafeCreate`, `InlineLLVMDestructorMake`
- 配列を通す（結果の根は LOCAL、下はオペランドから。13）:
  `InlineLLVMArrayTruncateBoundsUnchecked`,
  `InlineLLVMArrayAppendValueCapacityUnchecked`, `InlineLLVMArraySetCapacityBoundsUnchecked`,
  `InlineLLVMArrayAppendCapacityBoundsUnchecked`, `InlineLLVMArrayGrowSizeBody`,
  `InlineLLVMArraySetBody`, `InlineLLVMArraySwapBody`, `InlineLLVMPunchedArrayPlugBody`,
  `InlineLLVMArrayCheckRange`, `InlineLLVMArrayCheckSize`, `InlineLLVMArrayGetPtrBody`,
  `InlineLLVMArrayGetSizeBody`, `InlineLLVMArrayGetCapacityBody`
- 値をそのまま返す・スカラーを返す（7）:
  `InlineLLVMUnionIsBody`, `InlineLLVMIsUniqueFunctionBody`, `InlineLLVMArrayIsStorageUniqueBody`,
  `InlineLLVMBoxedToRetainedPtrIOS`, `InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody`,
  `InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody`, `InlineLLVMGetBoxedDataPtrFunctionBody`

### uniqueness のモードは locality を動かさない

いくつかの op は `assuming_unique` で「チェックしない版」に差し替わる（`array_set` ->
`array_set[unique]` など）。この 2 つの版の `locality_flow` は同じ `merge` である。

チェック無し版が呼ばれている時点で uniqueness が証明されている、という事実を locality に
使うことはできない。**uniqueness は根オブジェクトの参照カウントの話で、locality はそこから
到達できるグラフ全体の話**だからである。根が unique でも、要素が global 由来でありうる:

```
g : Array I64;
g = [1, 2, 3];          -- 初期化後 GLOBAL にマークされる

let a = [g, g];         -- Array (Array I64)。新規確保なので静的に Unique
                        -- a 自身のストレージは LOCAL、要素は GLOBAL
let a = a.set(0, g);    -- array_set[unique]（チェックは畳まれている）
```

この `a` は転送規則により `root = IfAny(∅)`、`deep = Always`、すなわち `Local` である。
`array_set[unique]` の flow を「`root` も `deep` も `IfAny(∅)`」すなわち `DeepLocal` にすると、
`a.@(0)` を読んだときに取り出し規則が `DeepLocal` を返し、その `Release` が非 atomic
デクリメントとして出る — 対象は `g` のストレージで、GLOBAL オブジェクトの参照カウントは維持
されていないから、最初の release で解放される。

**ただし `root` の側は、uniqueness から実際に従う。** 静的に Unique なら根オブジェクトは
`LOCAL` である（`Fresh` に遡れて、`create_obj` は `LOCAL` で初期化し、`mark_global` は初期化子
の本体が終わってから走る。実行時にも `is_unique` の `global_bb` は `shared_bb` へ行くので
GLOBAL は決して unique にならない）。そしてこの解析は `root` と `deep` を分けて持っているので、
その事実は素直に書ける — もっとも、扉以外の op は `root` を `IfAny(∅)` に保つので、`merge` が
既に同じ結論を出しており、uniqueness の情報を持ち込む必要は無い。

## 手続き間: 独立した specialize パス

2 相に分ける。

**相 1 — 記号的サマリ。** `provenance.rs` の `analyze_program` の phase 1 と同じ形。

状態は `summary : Map<FuncRef, ExtShape>` の 1 本だけ。`summary[f]` は「`f` の**結果**の各 leaf
の `ExtCond`」で、条件は `f` の入力（パラメータ列、その後に capture）の leaf を指す。

**初期値は束の底。** 全関数について、結果型の全 boxed leaf を両成分 `IfAny(∅)`（入力に関わらず
`DeepLocal`）に置く。

**1 回の更新**は「全関数の本体を 1 回ずつ走査して `summary` を上げる」:

1. 環境を恒等サマリで初期化する — パラメータ `i` の leaf `σ` に `IfAny({(i, σ)})`、capture
   （添字は `params.len()`）も同様。
2. 本体を転送規則（「転送」の節）で前から走査する。直接呼び出し `let x = App(g, args)` では
   `summary[g]` を取り、その各 leaf の `ExtCond` に現れる `(j, σ)` を `env[args[j]][σ]` で
   置き換える（`Always` はそのまま）。`locality_flow` の代入と同じ操作。callee がこの単位に
   無い、または間接呼び出しなら、結果の全 leaf を `Always` にする。
   **`j` が callee の capture の添字（`g.params.len()`）のときは `Always` に解決する** — 直接
   呼び出しは capture を引数として渡さないので `args` に対応物が無い。`CaptureProject` の配線が
   capture leaf を本体へ流すので、closure-ABI 関数のサマリには実際に capture 添字が現れうる。
3. 走査の終端 `ret` の値の `ExtShape` が候補。`summary[f] = summary[f] ⊔ 候補` と join する。

1 つでも動いたらもう 1 周。何も動かなくなったら収束。

**停止する**のは昇鎖が有限だから。1 leaf の `ExtCond` は
`IfAny(∅) ⊑ … ⊑ IfAny(全入力 leaf) ⊑ Always` の高さ ≤ |入力 leaf| + 2 で、leaf 数も関数数も
有限、更新は join なので単調にしか動かない。

**繰り返しが要るのは直接呼び出しのグラフに循環があるから。** Fix にループは無いが再帰はあり、
`-O max` の specialized `fold` クローンはループ本体を名前で直接呼ぶ。循環が無ければ逆トポロジ
順の 1 パスで済む。

**底から始めるのは精度のため。** 健全なのは post-fixpoint であることによるので（次節）、頂から
始めても健全である。違うのは精度で、頂から始めると再帰関数が「自分はまだ `MayExt` かもしれ
ない」を自分に食わせて `Always` に落ち着く:

```
k() = if c { 新規確保 } else { k() }
```

底から始めれば両アームとも `IfAny(∅)` で `Local`。頂から始めると 2 番目のアームが `Always` に
なって join も `Always` になる。Kleene 反復を底から回すと最小の post-fixpoint に着くので、
健全なものの中で最も精密なものが得られる。

**収束後にもう 1 周**して、各関数と各 global 初期化子の本体を走査し、RC site ごとの `ExtCond`
を記録する。これが相 2 のゲートの判定材料になる。global 初期化子は呼ばれる側ではないので
不動点には参加せず、この 1 周でだけ扱う。

### 健全性

抽象解釈の標準形で言う。具体側の意味論を最小不動点として定め、`γ` で抽象側とつなぎ、
**局所健全性**だけを手で確かめる。あとは Knaster-Tarski が結論を出す。

**具体側**（プログラムが実際に何をするかを近似なしに述べた対象。計算はしない）。関数ごとに
「引数の組と、それを与えたとき返る値」の対の集合を取る:

```
C = Π_f P(In_f x Out_f)        順序は成分ごとの ⊆
G(R)_f = { (a, v) | f の本体を a で走らせ、直接呼び出し g では R_g の対を使ったとき v を返す }
```

- `In_f` は `f` の引数の組（パラメータ列 + capture）としてありうるもの全体、`Out_f` は返り値
  としてありうるもの全体。どちらも型が決める universe である。ここでの「値」は、レジスタ上の
  値だけでなく**そこから到達するオブジェクトのグラフと各オブジェクトの状態バイトまで込み**で
  ある — `γ` がそれを見るため。
- `P` はべき集合。`P(In_f x Out_f)` の 1 要素が「`f` の意味論の候補」1 つで、決定的な全域関数
  ならその要素は関数のグラフになる。べき集合の直積なので `C` は完備束（`⊔` は成分ごとの `∪`、
  `⊥` は全成分 `∅`）。`In_f` は無限（任意長の配列など）なので `C` も無限だが、べき集合は濃度に
  よらず完備なので差し支えない — `C` の上では一度も反復せず、定理を 1 回適用するだけである。

`G` は単調で、`lfp G` が関数の表示的意味論そのものである。相互再帰する関数の意味論は他方の
意味論なしには書けないので、1 段展開の最小不動点として定めるのが標準の定義であり、`C` は
その定義が住む場所である。

**戻らない呼び出しは対を 1 つも生まないので `lfp G` に現れない。** 階乗
`f = |n| if n == 0 { 1 } else { n * f(n-1) }` なら、`∅` から `G` を回して
`{(0,1)}`、`{(0,1),(1,1)}`、… と増え、`lfp G = { (n, n!) | n >= 0 }` に落ち着く。`f(-1)` は
発散するので負の `n` の対は最後まで現れない。発散の扱いはこの定義に吸収されていて、後で
場合分けする必要がない。

**抽象側**（コンパイラが実際に持って計算するもの）。`summary` の住む束を `A` とする:

```
A          = Π_f ExtShape_f                          -- Map<FuncRef, ExtShape>
ExtShape_f = Π_{π ∈ leaves(ret_f)} LeafCond_f        -- Map<FieldPath, LeafCond>
LeafCond_f = ExtCond_f x ExtCond_f （root ⊑ deep の部分束）
ExtCond_f  = P(InputLeaves(f)) に頂 Always を付け足した束
InputLeaves(f) = { (i, σ) | i は f の入力の添字、σ は入力 i の型の boxed leaf パス }
```

`⊥_A` は全 leaf 両成分 `IfAny(∅)`、`⊤_A` は全 leaf 両成分 `Always`。`ExtCond_f` の高さは
`|InputLeaves(f)| + 2` で、`LeafCond_f` はその 2 乗以下、`leaves(ret_f)` も関数の数も有限なので `A` は有限、したがって完備。
`F : A -> A` は「全関数の本体を 1 回走査する」写像で、単調 (P3)。

**つなぎ。** `γ : A -> C`。成分ごとには
`γ_f : ExtShape_f -> P(In_f x Out_f)` で、`S` が主張することを実際に満たす対の集合を返す:

```
γ(S)_f = { (a, v) | 結果の各 leaf π について、r = resolve(S_f[π], a の実 locality) とおくと
                    r ⊑ Local     ならば v.π のオブジェクトが REFCNT_STATE_LOCAL
                    r = DeepLocal ならば さらに v.π から到達できるオブジェクトもすべて同様 }
```

`S` が大きいほど（`Always` が多いほど）主張が弱く集合が広いので、`γ` は単調。逆向きの
`α : C -> A` は要らない — concretization だけの枠組みで足りる。

**局所健全性（手で確かめるのはここだけ）。**

> すべての `S` について `G(γ(S)) ⊆ γ(F(S))`。

`f` の本体 1 本を、直接呼び出しの振る舞いを `γ(S)` から取って走らせたとき、返る対が `F(S)_f`
の主張を満たす、という言明である。本体は前向き 1 パスの有限な木なので、その構造に関する場合
分けで済み、再帰は現れない。各ケースの根拠は転送規則の節に書いたとおりで、`Llvm` op が (P1)、
束縛の生存中に事実が変わらないことが (P2)、`Release` が根だけ `Local` な値でも壊れないことが
(P4)、直接呼び出しが `γ` の定義そのもの、global アトム・間接呼び出し・単位外呼び出しは
`Always` なので主張が無く自明に成立する。

**使う定理。**

> **Knaster-Tarski。** `(L, ≤)` を完備束、`F : L -> L` を単調写像とする。このとき `F` の
> 不動点全体もまた完備束をなし、とくに最小不動点 `lfp F` が存在して
>
> ```
> lfp F = ⊓ { x ∈ L | F(x) ≤ x }
> ```
>
> が成り立つ。ここで `F(x) ≤ x` を満たす `x` を本稿では **post-fixpoint** と呼ぶ（文献に
> よっては pre-fixed point と呼ぶ）。
>
> **系（不動点帰納法）。** `F(x) ≤ x` を示せば `lfp F ≤ x` が従う。

**有限性も連続性も要らない**（単調性と完備性だけでよい）のがこの定理の要点である。ここで使う
のはこの系だけで、示すのは「`γ(S)` が `G` の post-fixpoint であること」の 1 点になる。

`C` は完備（前述）、`G` は単調（callee の振る舞いが増えれば caller の振る舞いも増える）。
`A` は有限なので完備。定理を適用するのは `C` の側で、反復するのは `A` の側という分業になる。

**結論。** アルゴリズムは `summary[f] = summary[f] ⊔ 候補` で更新して「1 つも動かない」で
抜けるので、抜けた時点で `F(S) ⊑ S`、すなわち `S` は `F` の post-fixpoint である。すると

```
G(γ(S)) ⊆ γ(F(S))     -- 局所健全性
        ⊆ γ(S)         -- F(S) ⊑ S と γ の単調性
```

なので `γ(S)` は `G` の post-fixpoint であり、系から `lfp G ⊆ γ(S)`。すなわち `S` は
プログラムの意味論を過大近似している。

Tarski が出すのは健全性だけで、計算手続きは別である。抽象側の束は昇鎖条件を満たす（有限）
ので Kleene 反復が停止し、底から回せば `lfp F` に到達する — こちらが実装の根拠になる。

**この論証に「底から始めた」は現れない。** 健全なのは post-fixpoint であることだけによる。
底から始めるのは、Kleene 反復が**最小の** post-fixpoint に到達する、つまり健全なものの中で
最も精密なものが得られるからであって、健全性のためではない。頂から始めても健全で、精度だけ
落ちる。

**注釈の健全性はもう 1 つの不動点で言う。** 上はサマリ（後ろ向き・表示的）の話で、注釈は
クローンの中で「キーが `Local` を証明する site に印を付ける」ので、キーが実際に満たされて
いることが要る。これは前向きの到達可能性で、束をもう 1 組立てて同じ型の議論をする:

```
C_R = P( ⋃_f {f} x In_f )        -- 実行中に現れる活性化（関数と実引数）の集合。具体側
A_R = P( FuncRef x LocalityKey ) -- 到達しうる (関数, キー) の対。抽象側。有限
γ_R : A_R -> C_R
γ_R(Reach) = { (f, a) | ある (f, k) ∈ Reach について a が k を満たす }

H(Reach) = { (f, canonical) | f は単位の関数すべて }   -- specialize が実際に全関数の
                                                      -- canonical を要求するのと同じ。
                                                      -- エントリ・global 初期化子・
                                                      -- FFI エクスポート・間接呼び出しの
                                                      -- 受け皿を一括で覆う
         ∪ { (g, k) | (f, k') ∈ Reach かつ f のクローン k' の中の call site が g を k で呼ぶ }
```

`LocalityKey` は leaf パスから 3 値への有限写像なので `A_R` は有限のべき集合束、`C_R` も
べき集合束で完備。`specialize` の worklist はこの `H` の Kleene 反復そのもので、キューが空に
なった時点が post-fixpoint である。具体側の 1 段展開の最小不動点が `γ_R(Reach)` に収まる
ことが同じ 2 行で出る。局所健全性にあたるのは
「call site がキーを組むときに使う `ExtCond` の解決が正しい」で、これはサマリ側の局所健全性の
系である。

基底は全関数の canonical 版（全 leaf `MayExt`）で、何も主張しないキーなので無条件に満たされる。
外部から届く経路 — エントリポイント、`FFI_EXPORT`、間接呼び出し、単位外呼び出し — はすべて
ここに落ちる。

**寄りかかっているのは (P1)、(P2)、(P4) である。** 77 個の手書き宣言のどれか 1 つが誤っていれば
局所健全性が破れ、結論が崩れる。だから `develop_mode` の実行時 assert（`Local` と注釈した
site で状態バイトを読んで検査する）を実装と同時に入れ、結論そのものを全テストプログラムで
直接検査する。

**相 2 — locality をキーにした複製。** キーは「パラメータごと x leaf ごとの 3 値」。

- 全関数の canonical 版（全 leaf `MayExt`）を残す。間接呼び出しと単位外呼び出しの受け皿で、
  今日と同じく全部ディスパッチする。
- クローンの実体化が call site を歩き、引数の locality（呼び出し元クローンの具体的入力 +
  相 1 のサマリで解決）から callee のキーを組んで worklist に積む。
- ゲート: 複製するのは「自分の RC site が入力に依存する関数」と「入力依存の leaf を、ゲートを
  通った callee へ直接呼び出しで渡す関数」。`funcs_reaching_unique_check` と同じく直接呼び出し
  グラフ上の最小不動点で閉じる。推移的に閉じないと、自分では RC site を持たない転送関数
  （`h(x) = g(x)`）が canonical のままになり、その中で組む `g` のキーが全 `MayExt` に落ちて、
  `h` を経由する呼び出し元すべてで `g` の証明が黙って失われる。

キーの値が 2 から 3 に増えるので、クローン数は原理上ふくらむ。ただし `Local`（根だけ）が
`DeepLocal` と分かれるのは「global 由来の値をコンテナに入れた」場合だけで、`plan.md` の測定
ではコーパスの大半が marked object を 1 つも作らない。実際にどれだけ増えるかは実測で確かめる。

クローンの中では入力が**具体値**なので、1 つのヘルパが `MayExt` な引数と `DeepLocal` な引数の
両方で呼ばれても、それぞれのクローンが別々に証明される。monovariant（関数入力ごとに全呼び
出し元の join を 1 つ持つ）も検討したが、この混合文脈で丸ごと de-prove する弱点があるので
複製を採る。クローン数は locality キーが実際に異なる関数でしか増えない見込みで、実測で
確かめる。

### `unique_check_elim` とは別のパスにする

**キーを 1 本に混ぜない。** 2 つの性質は独立で、必要とするゲートが違う。uniqueness のゲートは
`reaches_unique_check`、locality のゲートは「RC site が入力に依存するか」で、後者はほぼ全関数
を通す。混合キーにすると、uniqueness チェックを持たない関数が呼び出し元ごとに違う uniqueness
成分を受け取って別クローンになり、中身が同一のクローンが増える。これを避けるには「その関数が
実際に使う成分だけにキーを射影する」正規化が要るが、その仕組みは結合させたことだけを理由に
生まれる。パスを分ければゲートは各パスのものがそのまま働く。

分けられるのは**2 つのパスが可換**だからである。uniqueness の畳み込みは op を
`array_set` -> `array_set[unique]` に差し替えるが、両モードの `locality_flow` は同じ `merge`
なので locality のサマリは変わらない（前節）。locality の注釈は `RcState` フィールドを書くだけ
で、provenance はそれを読まない。よってどちらの順でも各本体に付く注釈は同じで、到達する
(uniqueness, locality) の組も同じなのでクローン総数も変わらない。

**順序は `unique_check_elim` の後。** 結果が同じなら、決め手は `is_unique` の注釈になる。
locality が注釈するのは**実行時チェックが残っている** site だけで、uniqueness に畳まれて
`array_set[unique]` になった site には読む状態バイトがもう無い。畳み込みを先に済ませておけば、
locality パスは残った site だけを見ればよく、ゲートも「実行時チェックにまだ到達する関数」で
組める。逆順にすると、消える運命の site のためにキーを分ける。

skeleton（worklist、クローン命名、canonical 版、call の retarget、`borrowed_units` の
renaming 追従）は 2 つのパスで同一なので、`unique_check_elim` から括り出して
「キー型・callee キーの算出・本体の書き換え・ゲート」で径数化し、2 回インスタンス化する。

hot 経路がキーで届くことは確認済み: `-O max` では decapturing がループ本体の識別を
specialized `fold` クローンに焼き込み、その本体はループ本体を**名前で**直接呼ぶ（RC IR
ダンプで確認: `fold#…#specialized_…` が `main#…#decap_lam1#funptr3#borrow` を直接呼ぶ）。
また uncurry/decap で capture は普通のパラメータになっているので、closure 由来の値もキーの
対象に入る（specialize のキーが capture を除外するのは closure-ABI 版だけで、そちらは
canonical のまま — 今日の uniqueness と同じ扱い）。

単位の外から呼ばれうる関数（プログラムシンボル）は canonical しか参照されようがないので
自然に `MayExt` 側に落ちる。

**`FFI_EXPORT` の入口も同じ受け皿で足りる。** エクスポート関数は boxed 値を不透明ポインタと
して受け取れる（`has_c_abi` が `is_box` を許す。#114 が禁じたのは集約型と `Bool`）ので、C から
状態の分からない boxed 値が入ってくる。生成されるラッパは、エクスポートされた**global の
closure 値**を読み、`apply_lambda` で適用する（`export_statement.rs`）— すなわち
**global アトムの読み出しと間接呼び出し**である。specialize は closure を特殊化せず
（`g.capture.is_some()` なら retarget しない）、間接呼び出しを retarget することもないので、
C から届くのは常に canonical 版であり、そのキーは全 leaf `MayExt` で何も主張しない。健全性を
担っているのは「入力に boxed leaf が無いこと」ではなく、この経路である。

単位間サマリの保存は測定が要求したときの将来課題。

## 注釈する site

状態バイトを読む site は 3 種類あり、**3 つとも注釈する**。

| site | 状態バイトを読む所 | 注釈の置き場 |
| --- | --- | --- |
| `Retain` / `Release` | `retain_nonnull_boxed` / `build_release_boxed_with` | ノードの `RcState` フィールド（既存） |
| `is_unique` チェック | `build_branch_by_is_unique`。unique-check op の `generate` の中 | op インスタンスの属性フィールド |
| `Destructure` | `get_struct_fields`。ノード自身が retain/release する | ノードに `RcState` を足す |
| 破棄の traverser | `build_traverse`。カウントが 0 になったとき子を release する | site ごとの置き場が無い（traverser は型ごとの関数） |

**上の 3 つは一緒に出す。** `plan.md` の上限表（`sort` -13.87%、`levenshtein` -6.25% など）は
`build_branch_by_refcnt_state` のディスパッチを全部外して測ったものなので、比較できるのは
覆った実装だけである。`is_unique` はディスパッチの過半を占めることがあり（`fannkuch` 57%、
`cp_lib_lsegtree` 15%）、これを落とすと `fannkuch` の測定は上限のごく一部しか動かない。

**4 番目の traverser は、まず健全性の前提として押さえる。** `Release` が 0 に到達したとき
呼ぶ型 traverser は、子ごとに状態バイトを読んでディスパッチする（`build_traverse` ->
`build_release_mark_nonnull_boxed_with` -> `build_release_boxed_with` ->
`build_branch_by_refcnt_state`）。**根だけ `Local` な値の release を `Local` と注釈できるのは、
このディスパッチが子を正しく捌くからである。** `[g, g]` のような値は自分のストレージが LOCAL
なので根の非 atomic デクリメントは正しく、GLOBAL な子は traverser の `global_bb` で no-op に
なる。traverser をディスパッチなしに変えると、この注釈がそのまま壊れる。局所健全性 (P1) の
`Release` のケースはこの性質に寄りかかっている。

そのうえで、**`DeepLocal` は状態なし traverser を呼んでよい条件そのもの**である。`DeepLocal`
は「ここから到達できる任意のオブジェクトが LOCAL」なので、破棄が辿る先は定義上すべて LOCAL で、
子の状態を読む理由が無い。traverser は call site ごとではなく**型ごと**の関数なので site に
属性を付ける方式が使えず、実体としては型ごとに 2 本目（状態なし版）を出して `Release(DeepLocal)`
から呼び分けることになる。一族はちょうど 2 本で、`Release(DeepLocal)` が実際に出た型についてだけ
生成すればよい。上限表は traverser 内部の release も含めて測っているので、これを外したままでは
破棄が再帰する形（`binary_trees` の木の解体など）で上限に届かない。段階 1 の実測で、`Local` と
`DeepLocal` の内訳と traverser 経由の release の割合を数えてから足す。

### 注釈のしかた

クローンの実体化のとき、入力の具体値の下で本体を前向きに 1 回走査する。

- `Retain(x, π, Unknown)` / `Release(x, π, Unknown)`: `π` 以下の全 leaf が `Local` 以下
  （`DeepLocal` か `Local`）なら `RcState::Local` に書き換える。
- unique-check op: `unique_check_operand` が指す leaf が `Local` 以下なら、op を「対象は `LOCAL`」
  版に差し替える。差し替えは `assuming_unique` と同じパターン — 対象 op の struct にフィールドを
  足し、それを立てたクローンを返すメソッドを生やす。`unique_check_elim` を先に走らせてあるので
  （後述の順序）、ここで見るのは実行時チェックが残った site だけである。
- `Destructure`: ノードが行う参照カウント操作**すべて**が `LOCAL` なオブジェクトに対するもの
  なら `RcState::Local`。boxed コンテナならコンテナの release（コンテナ leaf の `root`）と各
  フィールドの retain（取り出し規則により、コンテナが `DeepLocal` のときだけフィールドの根が
  `Local` 以下）なので、**コンテナが `DeepLocal` のときに限る**。unboxed コンテナなら名前の
  付かなかったフィールドの release で、**そのすべてが `Local` 以下のときに限る**。状態を
  コンテナ側とフィールド側に分ければ boxed の `Local` コンテナでも release だけは畳めるが、
  その精密化は実測が要求したら足す。

それ以外は `Unknown` のまま。global 初期化子本体は入力なしで同様に解釈する（specialize が今
`&[]` でやっているのと同じ形）。

パイプラインでの位置:
`… → borrow_ify → cancel → unique_check_elim::specialize → locality::specialize → implement`。
相 1 のサマリ計算は locality パスの入口で、`unique_check_elim` の出力に対して行う。ゲートは
他の Max 以上のパスと同じものに加えて `!config.threaded`（threaded ビルドではパスごと走らせ
ない）。

## コード生成

`implement_rc_program` の `Retain`/`Release` アームは今 `Unknown` を assert している。`Local`
アームを足す:

- `Retain(Local)`: 非 atomic インクリメント。状態ロードなし、分岐なし（今日の `local_bb` の
  本体）。
- `Release(Local)`: 非 atomic デクリメント、読んだカウントが 1 なら破棄 — こちらも今日の
  local アームからディスパッチを外したもの。
- `Destructure(Local)`: `get_struct_fields` が呼ぶ retain/release を上の 2 つに差し替える。
- unique-check op（対象が `LOCAL` 版）: `build_branch_by_is_unique` の状態ディスパッチを外し、
  参照カウントを 1 と比べる分岐だけを出す（今日の `local_bb` の本体）。

null チェックの包み（`skip_null_check`、dynamic object のチェック）は直交で不変。破棄が呼ぶ
型 traverser は**ディスパッチしたままにする** — 上で述べたとおり、根だけ `Local` な値の release
の健全性がこれに寄りかかっている。`DeepLocal` 専用の状態なし版を足すのは後段の精密化。

## コードだけでなく解析を検証する

- **`develop_mode` の実行時 assert**: `Local` と注釈された**3 種類すべての** site で状態バイトを
  読み、`REFCNT_STATE_LOCAL` でなければ abort。「壊れる側」の誤りを、静かなメモリ破壊から
  その場の abort に変えるもので、局所健全性 (P1) の穴に対する唯一の実効的な防御なので、実装と
  同時に入れる（後追いにしない）。テストスイート全体が `develop_mode` で走るので、注釈された全
  site が全テストプログラムで動的に検査される。わざと 1 site を誤注釈してスイートが落ちる
  ことを一度示し、その破壊を戻す。
- **ダンプ水準の回帰テスト**（`--emit-rc-ir`）: 実行時 assert は「誤注釈した site を非 LOCAL な
  オブジェクトに対して実行するテストがある」ときしか鳴らない。コーパスの大半は marked object を
  1 つも作らない（`plan.md`）ので、global の近くでしか誤らない分類ミスは動的検査を素通りする。
  扉の形ごとに、ダンプが `Always`（ディスパッチ維持）になることを静的に固定する:
  `destructure <global>`、`let a = [g, g]; a.set(0, g)`、`boxed_from_retained_ptr` の流れ、
  コンテナ経由の global 読み出し、boxed 要素型に対する `mutate_elements`。
- **カバレッジ測定**（一時プローブ、読んだら revert）: speedtest corpus で実行された
  `Local` / `Unknown` 操作を site の種類ごとに数え、`plan.md` の上限表（`arg`+`local` 行）と
  突き合わせる。併せてクローン数（specialize の出力関数数）を拡張の前後で比べる。
- **コンパイル時間**: 全プログラムの不動点 1 本と 2 本目の複製パスが増えるので、パス自体の
  所要時間を測る。RC パイプラインはコンパイル時間に敏感な履歴がある（#144、#76）。
- **全スイート** 3 水準、**`benchmark/speedtest`** を現 `main` の行と比較。捨てた設計で
  裏返ったナイフエッジ（`nbody`、`nbody_fold`）を注視する。

## ファイル

| ファイル | 変更 |
| --- | --- |
| `src/rc_ir/locality.rs`（新規） | `Locality` / `ExtCond` / `ExtShape` / `LocalityKey`、転送、相 1 の記号的サマリ、相 2 と注釈 |
| `src/rc_ir/specialize.rs`（新規） | `unique_check_elim` から括り出した複製 skeleton |
| `src/rc_ir/ast.rs` | `Destructure` に `RcState` |
| `src/ast/inline_llvm.rs` | `LLVMGen::locality_flow`（既定実装なし）、`assuming_local` |
| `src/fixstd/builtin.rs` | 全 77 op の `locality_flow`、unique-check を持つ 18 op の属性と `assuming_local` |
| `src/rc_ir/unique_check_elim.rs` | skeleton を括り出し、uniqueness 固有部分だけ残す |
| `src/rc_ir/codegen.rs` | `Retain`/`Release`/`Destructure` の `Local` アーム、`develop_mode` assert |
| `src/generator.rs` | 状態を見る retain/release/is_unique 生成ヘルパ |
| `src/rc_ir/` の `lower.rs`, `print.rs`, `validate.rs`, `simplify.rs`, `rc_insert.rs`, `borrow.rs`, `ownership.rs`, `provenance.rs`, `rename.rs`, `unique_check_elim.rs` | `Destructure` のフィールド追加に追従（`RcExpr::Destructure` を触る全 11 ファイルから `codegen.rs` を除いたもの） |
| `src/build/build_object_files.rs` | `specialize` の後に locality パスを差し込む |
| `src/rc_ir/mod.rs` | 新規 2 モジュールの宣言 |

`RcState::Local` とダンプの `@local` 形は既にある。`validate` は状態を見ない。

## 対象外

- 単位間サマリ。
- `DeepLocal` 用の状態なし traverser（前述。実測で内訳を見てから）。
- threaded ビルド（`threaded.md`）。
- changelog: 観測可能な振る舞いは変わらない。
