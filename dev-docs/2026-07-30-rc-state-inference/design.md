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

本文は段階 1（非 threaded ビルドの全 RC site）を詳細化し、末尾の 1 節が段階 2
（threaded ビルド）を設計する。

## 性質

束縛 `x` とその型の boxed leaf パス `π` について、2 値を割り当てる:

> `Local` — `x.π` が指すオブジェクトも、**そこから到達可能な任意のオブジェクト**も、global から
> 到達可能ではない（= `REFCNT_STATE_LOCAL` である）ことが証明できた。
>
> `MayExt` — 証明できなかった。

`Local` ⊑ `MayExt` の 2 点束で、join は `MayExt` 側。site が状態ディスパッチを外せるのは、
その site が触る leaf がすべて `Local` のとき — `Retain`/`Release` なら unit パス `π` 以下の
全 leaf、`is_unique` ならチェック対象の leaf、`Destructure` ならノードが retain/release する
leaf 全部（「注釈する site」の節）。

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
  うるか」の escape 推論が必要 — 段階 2（#96 待ち）ごと先送りする。

**段階 1 の注釈は `config.threaded` が偽のときだけ走る。** threaded ビルドは今日のまま全部
ディスパッチする。

## 束と用語

**層が 2 つある。** 解決後の層（クローンの中。呼び出し元が決まっていて、値が具体的）と、
記号的な層（相 1 のサマリと `locality_flow`。まだ呼び出し元が決まっていない）。既存の
provenance / uniqueness と同じ 2 層構造で、名前も層ごとに分ける:

| | leaf 1 個 | 値 1 個 | 対応する既存の型 |
| --- | --- | --- | --- |
| **解決後** | `Locality` = `Local` / `MayExt` | `LocalityKey(Map<FieldPath, Locality>)` | `SharingVerdict` / `Uniqueness` |
| **記号的** | `ExtCond` = `Always` / `IfAny(集合)` | `ExtShape(Map<FieldPath, ExtCond>)` | `LeafOrigins` / `Provenance` |

```
enum Locality { Local, MayExt }

/// leaf が `MayExt` になる条件。
enum ExtCond {
    /// 入力に関わらず `MayExt`。束の頂。
    Always,
    /// 挙げた入力 leaf のどれかが `MayExt` なら `MayExt`。
    /// 空集合が束の底で、入力に関わらず `Local`。
    IfAny(Set<(usize, FieldPath)>),
}
```

**この文書では、`MayExt` は解決後の層でだけ、`Always`/`IfAny` は記号的な層でだけ使う。**
`ExtCond::Always` は「無条件に `MayExt` になる」の意味であって、`MayExt` の別名ではない。

`IfAny` の添字 `(i, σ)` は、サマリでは関数の入力 `i`（パラメータ列、その後に capture）の
leaf `σ`、`locality_flow` では op のオペランド `i` の leaf `σ` を指す。`Provenance` が
`LeafOrigin::Arg(i, path)` を 2 つの索引空間で使い回しているのと同じ。

解決は `resolve(ExtCond, inputs: &[LocalityKey]) -> Locality`: `Always` は `MayExt`、
`IfAny(s)` は `s` のどれかの入力 leaf が `MayExt` なら `MayExt`、でなければ `Local`。

### `Always` は吸収元

`Always` を含む値は入力が何であっても `MayExt` に解決するので、そこに並ぶ入力 leaf は結果を
一切変えない。これを不変条件として手で保つのではなく、`ExtCond` の形で持つ。join は
`Always ⊔ _ = Always`、`IfAny(a) ⊔ IfAny(b) = IfAny(a ∪ b)`。

`IfAny(∅)` になるのはその場で確保した値である — `create_obj` は `LOCAL` で初期化するので、
`MayExt` になる条件が 1 つも無い。確保したコンテナに `MayExt` な値を入れた場合は、その値の
条件が転送規則（`merge`）で入ってくるので `IfAny(∅)` にはならない。leaf パスは型で、入力は
有限で抑えられるので、束は有限で不動点は停止する。

吸収を形で持つことは、複製のゲートの精度にそのまま効く。ゲートは「RC site が入力に依存しない
関数は複製しない」で、依存の判定は `IfAny(s)` の `s` が空でないこと。原始的に「条件の集合」
として持って合併すると、`{Always, (0, σ)}` が入力への言及を含むので「依存する」と読まれ、
キーを変えても結果が動かない関数が複製されてしまう。`Always` に潰しておけばそれが起きない。

## 転送

RC IR ノードごとに、各ローカル束縛を `ExtShape` に写す環境の上で。**規則は記号的な層で 1 回だけ
定義する。** クローンの中の走査は同じ規則を走らせて、入力の `ExtCond` をキーで解決したもので
ある — 入力が具体値なので、環境の値は各 leaf `Always`（= `MayExt`）か `IfAny(∅)`（= `Local`）
のどちらかに潰れ、実装上は 2 値の走査になる。

global シンボルを値として使う場所（「global アトム」）は読んだ値の全 leaf を `Always` にする —
これが扉 1 であり、名前が local かどうかを見る**唯一の**規則である。

- `let x = y`（move）: コピー。
- `let x = <global アトム>`: 全 leaf `Always`。（funptr 型の global は boxed leaf を持たないので
  何も起きない。closure 型の global は capture leaf が `Always` — その capture object はマーク
  されているので正しい。）
- `let x = Closure(f, caps)`: capture leaf に caps の全 leaf の join。
- `let x = App(callee, args)`:
  - callee がこの単位の `RcProgram` の関数を名指す — 直接呼び出し。手続き間の節へ。
  - それ以外（closure 値の変数、他単位の関数、global な closure 値）: 結果の全 leaf `Always`。
- `let x = Llvm(op, args)`: op の `locality_flow`。次節。
- `Destructure`: boxed コンテナ — 各フィールドの全 leaf にコンテナ leaf の値。unboxed —
  フィールドごとに射影（結果 leaf `σ` <- コンテナ leaf `[i]++σ`）。
- `Match`: payload は `Destructure` と同様（variant ごと）。arm の結果は join。
- `Retain`/`Release`/`Eval`: 環境は不変。
- `ret x`: 関数の結果に `x` を join。

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
    /// 結果の全 leaf に、全オペランドの全 leaf を挙げた `IfAny`。
    fn merge(result_ty, arg_tys, type_env) -> ExtShape;
    /// 結果の全 leaf が `Always`。
    fn always(result_ty, type_env) -> ExtShape;
    /// 結果 leaf ごとに指定する。`Provenance::build_shape` と同じ形で、結果型の boxed leaf
    /// を全部歩いて呼ぶので、leaf の書き落としが起こらない。
    fn build_shape(result_ty, type_env, f: &dyn Fn(&FieldPath) -> ExtCond) -> ExtShape;
}
```

**サマリと同じ `ExtShape` を返すことに意味がある。** `Llvm` ノードの転送は「op が宣言した
写像にオペランドの値を代入する」で、直接呼び出しの転送は「callee のサマリに引数の値を代入
する」。`IfAny` の添字が指す先が違うだけで操作は同一なので、代入は 1 つ書けば両方が使う。
`merge` / `always` はその写像を作る構成子であって、別扱いの分岐ではない。

`merge` が健全なのは「新しいオブジェクトを確保するか、オペランドから到達可能なオブジェクトを
並べ替えることしかできない」op に対してだけである。boxed コンテナからの読み出し（`array_get`、
boxed struct の getter）も `merge` で正しい — 到達閉包を取っているので、要素はコンテナから
到達可能だった。

**既定実装を置かない。** `merge` を既定にすると、オペランドから到達できない boxed オブジェクト
を作る op を将来足したときに、何も書かなくても `Local` が通る — 冒頭で述べた「壊れる側」の誤りが
黙って入ることになる。`always` を既定にすれば安全側だが、今度は書き忘れが黙って精度を殺し、症状が
出ないので気付けない。どちらの黙り方も避けたいので必須メソッドにして、op を足す人に必ず選ばせる。

`result_prov` の読み替えにせず独立のメソッドにするのは、provenance が別の問いに答えているから
（あちらの `Unknown` は「追跡していない共有」であって状態の共有ではない）。片方から導出すると、
uniqueness の都合の編集が健全性の論証を静かに変える。

### 全 op の値

`impl LLVMGen` は 77 個。構成子で分類すると `always` 8 個、`build_shape` 13 個、`merge` 56 個。

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
| `InlineLLVMHoleBody` | `unreachable` を出す。値は存在しない |

関数オペランドを呼ぶ op がこの表の半分を占める。呼ばれる関数は別の `RcFunc` として解析される
が、この op から見ると結果は間接呼び出しの結果であり、関数本体が global を読んで返すことを
`merge` は捉えられない。**`merge` を既定にしていたら、この 5 個が黙って `Local` を通していた。**

**`build_shape`（13）** — 集約の配管。`MayExt` な成分と `Local` な成分を分けて保つ。

| op | 配線 |
| --- | --- |
| `InlineLLVMStructGetBody` | unboxed コンテナ: 結果 `σ` <- 引数 0 の `[field_idx]++σ`。boxed コンテナ: 結果の全 leaf <- 引数 0 の `[]` |
| `InlineLLVMMakeStructBody` | 結果 `[i]++σ` <- 引数 `i` の `σ`。boxed 結果は全 leaf にコンテナの join |
| `InlineLLVMStructSetBody` | 結果 `[field_idx]++σ` <- 引数 0（値）の `σ`、他の leaf <- 引数 1（struct）の同じパス。boxed 結果は全 leaf にコンテナの join |
| `InlineLLVMStructPlugInBody` | 結果 `[field_idx]++σ` <- 引数 1（field）の `σ`、他 <- 引数 0（punched）。boxed 結果は全 leaf にコンテナの join |
| `InlineLLVMStructPunchBody` | 結果は `(field, punched_struct)`。`[0]++σ` <- 引数 0 の `[field_idx]++σ`、`[1]` 以下 <- 引数 0 の残り |
| `InlineLLVMArrayPunchBody` | 結果は `(elem, punched_array)`。両成分 <- 引数の配列 leaf |
| `InlineLLVMMakeUnionBody` | 結果 `[variant]++σ` <- 引数 0 の `σ`。他の variant は底（`IfAny(∅)`） |
| `InlineLLVMUnionAsBody` | unboxed union: 結果 `σ` <- 引数 0 の `[variant]++σ`。boxed union: 結果の全 leaf <- 引数 0 の `[]` |
| `InlineLLVMCaptureProjectBody` | capture object から 1 個取り出す。boxed capture: 結果の全 leaf <- capture の `[]`。unboxed `#CapList`: 射影 |
| `InlineLLVMUnsafeMutateBoxedInternalFunctionBody` | `[0]`（値）以下 <- 引数の値。残り（コールバックの結果）は `Always` |
| `InlineLLVMUnsafeMutateBoxedIOSInternalBody` | 同上、値の位置は `[1, 0]` |
| `InlineLLVMArrayMutateElementsInternalBody` | `[0]`（配列）<- 引数の配列。残りは `Always` |
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
- 確保のみ（オペランドの leaf を join する。オペランドに leaf が無ければ `IfAny(∅)` = `Local`。6）:
  `InlineLLVMStringBuf`, `InlineLLVMArrayUnsafeEmpty`, `InlineLLVMArrayLitBody`,
  `InlineLLVMIOStateUnsafeCreate`, `InlineLLVMUndefinedInternalBody`, `InlineLLVMDestructorMake`
- 配列を通す・配列を読む（結果はオペランドの配列から到達可能。14）:
  `InlineLLVMArrayUnsafeGetBoundsUnchecked`, `InlineLLVMArrayTruncateBoundsUnchecked`,
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

ここで `array_set[unique]` の flow を「空集合（`Local` 証明済み）」にすると、`merge` が
`[g, g]` から正しく付けた `MayExt` が上書きされて消える。そのあと `a.@(0)` を読むと、`merge` に
より結果も `Local` になり、その `Release` が非 atomic デクリメントとして出る — 対象は `g` の
ストレージで、GLOBAL オブジェクトの参照カウントは維持されていないから、最初の release で解放
される。冒頭の「壊れる側」の誤りそのものである。

なお「静的に Unique なら根オブジェクトは `LOCAL`」自体は成り立つ（`Fresh` に遡れて、
`create_obj` は `LOCAL` で初期化し、`mark_global` は初期化子の本体が終わってから走る。実行時
にも `is_unique` の `global_bb` は `shared_bb` へ行くので GLOBAL は決して unique にならない）。
ただしそれは根 1 個の事実であって、この解析が leaf に持たせている「グラフ全体」の事実ではない
ので、`IfAny(∅)` として書くことはできない。根だけの事実を別に持つ設計（注釈は根の事実で足りる
一方、コンテナ読み出しの伝播にはグラフの事実が要る）は精度を上げうるが、束とキーが 2 倍になる
ので、実測が要求してから検討する。

## 手続き間: 独立した specialize パス

2 相に分ける。

**相 1 — 記号的サマリ。** `provenance.rs` の `analyze_program` の phase 1 と同じ形。

状態は `summary : Map<FuncRef, ExtShape>` の 1 本だけ。`summary[f]` は「`f` の**結果**の各 leaf
の `ExtCond`」で、条件は `f` の入力（パラメータ列、その後に capture）の leaf を指す。

**初期値は束の底。** 全関数について、結果型の全 boxed leaf を `IfAny(∅)`（入力に関わらず
`Local`）に置く。

**1 回の更新**は「全関数の本体を 1 回ずつ走査して `summary` を上げる」:

1. 環境を恒等サマリで初期化する — パラメータ `i` の leaf `σ` に `IfAny({(i, σ)})`、capture
   （添字は `params.len()`）も同様。
2. 本体を転送規則（「転送」の節）で前から走査する。直接呼び出し `let x = App(g, args)` では
   `summary[g]` を取り、その各 leaf の `ExtCond` に現れる `(j, σ)` を `env[args[j]][σ]` で
   置き換える（`Always` はそのまま）。`locality_flow` の代入と同じ操作。callee がこの単位に
   無い、または間接呼び出しなら、結果の全 leaf を `Always` にする。
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

**具体側。** 関数ごとに「引数の組と、それを与えたとき返りうる値の組」の集合を取る:

```
C = Π_f P(In_f x Out_f)        順序は成分ごとの ⊆
G(R)_f = { (a, v) | f の本体を a で走らせ、直接呼び出し g では R_g の対を使ったとき v を返す }
```

`G` は単調で、`lfp G` が関数の表示的意味論そのものである。**戻らない呼び出しは対を 1 つも
生まないので `lfp G` に現れない** — 発散の扱いはこの定義に吸収されていて、後で場合分けする
必要がない。

**抽象側。** `F` は「全関数の本体を 1 回走査する」写像で、状態は `summary : Map<FuncRef,
ExtShape>`。有限束の上で単調 (P3)。

**つなぎ。** `γ(S)_f` を、`S` が主張することを実際に満たす対の集合とする:

```
γ(S)_f = { (a, v) | 結果の各 leaf π について、
                    resolve(S_f[π], a の実 locality) = Local ならば
                    v.π から到達できるオブジェクトがすべて REFCNT_STATE_LOCAL }
```

`S` が大きいほど（`Always` が多いほど）主張が弱く集合が広いので、`γ` は単調。`α` は要らない —
concretization だけの枠組みで足りる。

**局所健全性（手で確かめるのはここだけ）。**

> すべての `S` について `G(γ(S)) ⊆ γ(F(S))`。

`f` の本体 1 本を、直接呼び出しの振る舞いを `γ(S)` から取って走らせたとき、返る対が `F(S)_f`
の主張を満たす、という言明である。本体は前向き 1 パスの有限な木なので、その構造に関する場合
分けで済み、再帰は現れない。各ケースの根拠は転送規則の節に書いたとおりで、`Llvm` op が (P1)、
束縛の生存中に事実が変わらないことが (P2)、直接呼び出しが `γ` の定義そのもの、global アトム・
間接呼び出し・単位外呼び出しは `Always` なので主張が無く自明に成立する。

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

連続性は要らない — 単調性だけでよいのが Tarski の定理の要点である。ここで使うのはこの系
だけで、示すのは「`γ(S)` が `G` の post-fixpoint であること」の 1 点になる。

`C = Π_f P(In_f x Out_f)` はべき集合の直積なので完備束、`G` は単調（callee の振る舞いが
増えれば caller の振る舞いも増える）。抽象側の束は有限なので完備。

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
いることが要る。これは前向きの到達可能性で、同じ型の議論になる:

```
Reach ⊆ FuncRef x LocalityKey
H(Reach) = { エントリと global 初期化子の (f, canonical) }
         ∪ { (g, k) | (f, k') ∈ Reach かつ f のクローン k' の中の call site が g を k で呼ぶ }
```

`specialize` の worklist はこの `H` の Kleene 反復そのもので、キューが空になった時点が
post-fixpoint である。具体側は「実行中に現れる活性化の (関数, 実引数の locality)」の集合で、
その最小不動点が `γ(Reach)` に収まることが同じ 2 行で出る。局所健全性にあたるのは
「call site がキーを組むときに使う `ExtCond` の解決が正しい」で、これはサマリ側の局所健全性の
系である。

基底は canonical 版（全 leaf `MayExt`）で、何も主張しないキーなので無条件に満たされる。

**寄りかかっているのは (P1) と (P2) である。** 77 個の手書き宣言のどれか 1 つが誤っていれば
局所健全性が破れ、結論が崩れる。だから `develop_mode` の実行時 assert（`Local` と注釈した
site で状態バイトを読んで検査する）を実装と同時に入れ、結論そのものを全テストプログラムで
直接検査する。

**相 2 — locality をキーにした複製。** キーは「パラメータごと x leaf ごとの `Local`/`MayExt`」。

- 全関数の canonical 版（全 leaf `MayExt`）を残す。間接呼び出しと単位外呼び出しの受け皿で、
  今日と同じく全部ディスパッチする。
- クローンの実体化が call site を歩き、引数の locality（呼び出し元クローンの具体的入力 +
  相 1 のサマリで解決）から callee のキーを組んで worklist に積む。
- ゲート: RC site の値がどれも入力に依存しない関数（サマリの `IfAny` がどれも空）はキーで
  結果が変わらないので複製しない。

クローンの中では入力が**具体値**なので、1 つのヘルパが `MayExt` な引数と `Local` な引数の
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
自然に `MayExt` 側に落ちる。単一単位のビルド（speedtest corpus は全ケースこれ）ではエントリ
ポイントと FFI エクスポートだけが外部から届き、どちらの入力も boxed leaf を持たないので、
何も失わない。単位間サマリの保存は測定が要求したときの将来課題。

## 注釈する site

状態バイトを読む site は 3 種類あり、**段階 1 は 3 つとも注釈する**。

| site | 状態バイトを読む所 | 注釈の置き場 |
| --- | --- | --- |
| `Retain` / `Release` | `retain_nonnull_boxed` / `build_release_boxed_with` | ノードの `RcState` フィールド（既存） |
| `is_unique` チェック | `build_branch_by_is_unique`。unique-check op の `generate` の中 | op インスタンスの属性フィールド |
| `Destructure` | `get_struct_fields`。ノード自身が retain/release する | ノードに `RcState` を足す |

**3 つを一緒に出す。** `plan.md` の上限表（`sort` -13.87%、`levenshtein` -6.25% など）は
3 種類すべてのディスパッチを外して測ったものなので、比較できるのは 3 つを覆った実装だけで
ある。`is_unique` はディスパッチの過半を占めることがあり（`fannkuch` 57%、`cp_lib_lsegtree`
15%）、これを落とすと `fannkuch` の測定は上限のごく一部しか動かない。

### 注釈のしかた

クローンの実体化のとき、入力の具体値の下で本体を前向きに 1 回走査する。

- `Retain(x, π, Unknown)` / `Release(x, π, Unknown)`: `π` 以下の全 leaf が `Local` なら
  `RcState::Local` に書き換える。
- unique-check op: `unique_check_operand` が指す leaf が `Local` なら、op を「対象は `LOCAL`」
  版に差し替える。差し替えは `assuming_unique` と同じパターン — 対象 op の struct にフィールドを
  足し、それを立てたクローンを返すメソッドを生やす。`unique_check_elim` を先に走らせてあるので
  （後述の順序）、ここで見るのは実行時チェックが残った site だけである。
- `Destructure`: ノードが行う参照カウント操作**すべて**が `LOCAL` なオブジェクトに対するもの
  なら `RcState::Local`。boxed コンテナならフィールドの retain とコンテナの release で、到達
  閉包によりコンテナ leaf の値が両方を決める。unboxed コンテナなら名前の付かなかったフィールド
  の release で、それらの meet を取る。フィールドごとに状態を分ける精密化は、実測が要求したら
  足す。

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
型 traverser の内部ディスパッチは `Unknown` のまま — 状態ごとの traverser 一族は生成コードを
倍にするので対象外。

## コードだけでなく解析を検証する

- **`develop_mode` の実行時 assert**: `Local` と注釈された**3 種類すべての** site で状態バイトを
  読み、`REFCNT_STATE_LOCAL` でなければ abort。「壊れる側」の誤りを、静かなメモリ破壊から
  その場の abort に変えるもので、局所健全性 (P1) の穴に対する唯一の実効的な防御なので、実装と
  同時に入れる（後追いにしない）。テストスイート全体が `develop_mode` で走るので、注釈された全
  site が全テストプログラムで動的に検査される。わざと 1 site を誤注釈してスイートが落ちる
  ことを一度示し、その破壊を戻す。
- **カバレッジ測定**（一時プローブ、読んだら revert）: speedtest corpus で実行された
  `Local` / `Unknown` 操作を site の種類ごとに数え、`plan.md` の上限表（`arg`+`local` 行）と
  突き合わせる。併せてクローン数（specialize の出力関数数）を拡張の前後で比べる。
- **全スイート** 3 水準、**`benchmark/speedtest`** を現 `main` の行と比較。捨てた設計で
  裏返ったナイフエッジ（`nbody`、`nbody_fold`）を注視する。

## ファイル

| ファイル | 変更 |
| --- | --- |
| `src/rc_ir/locality.rs`（新規） | `Locality` / `ExtCond` / `ExtShape` / `LocalityKey`、転送、相 1 の記号的サマリ、相 2 と注釈 |
| `src/rc_ir/specialize.rs`（新規） | `unique_check_elim` から括り出した複製 skeleton |
| `src/rc_ir/ast.rs` | `Destructure` に `RcState` |
| `src/ast/inline_llvm.rs` | `LLVMGen::locality_flow`（既定実装なし）、`assuming_local` |
| `src/fixstd/builtin.rs` | 全 77 op の `locality_flow`、unique-check を持つ 19 op の属性と `assuming_local` |
| `src/rc_ir/unique_check_elim.rs` | skeleton を括り出し、uniqueness 固有部分だけ残す |
| `src/rc_ir/codegen.rs` | `Retain`/`Release`/`Destructure` の `Local` アーム、`develop_mode` assert |
| `src/generator.rs` | 状態を見る retain/release/is_unique 生成ヘルパ |
| `src/rc_ir/print.rs`, `validate.rs`, `simplify.rs`, `rc_insert.rs`, `borrow.rs`, `cancel` | `Destructure` のフィールド追加に追従 |
| `src/build/build_object_files.rs` | `specialize` の後に locality パスを差し込む |

`RcState::Local` とダンプの `@local` 形は既にある。`validate` は状態を見ない。

## 段階 2 — threaded ビルド

同じ束を `THREADED` に向ける。証明が通れば atomic RMW が非 atomic の増減になり、分岐 1 個を
消すより 1 操作あたりの取り分が大きい。追加で要るのは「性質」の節で述べた時間性への対処で、
問題の形はこうなる:

```
let a = ...;              -- a の値はここで決まる（Local）
retain a;
let b = a.mark_threaded;  -- b は MayExt。だがオブジェクトは THREADED になった
release a;                -- a は Local のまま → Local 注釈 → 非 atomic release。不健全
```

結果だけを `MayExt` にする forward 伝播では、マークされたオブジェクトに届く既存の束縛
（エイリアス）が漏れる。エイリアスはオリジン（確保点・パラメータ）を共有するので、
**オリジンを `Always` にする**のがエイリアス解析なしで漏れなく覆う最も粗い過大近似
（「mark_threaded に流れ込みうる値は生まれた時点から `Always`」）。機構は `borrow_ify` の
`infer_ownership` の第 3 の実例として作れる: あちらは consume site を種に、`origin`
（本体内の逆向き値追跡）でパラメータまで遡り、own フラグをプログラム全体の不動点まで単調に
育てる。こちらは種を `mark_threaded` op のオペランドに、フラグを「mark_threaded に流れ込み
うる」に替える。`origin` の遡り先にはパラメータ（手続き間の伝播）とローカルの確保束縛
（`Always` の付け先）の両方が出る。

ただしオリジン汚染は保守的すぎる面がある: マークより**前**に実行される操作は実行時には
LOCAL を見るので、本当は Local にできる。とくに「単スレッドで構築してから公開する」形では
構築フェーズの操作が全部それに当たる。これはフロー感度の精密化で健全に拾える —
**値が公開されうる位置（`mark_threaded` のオペランド、または公開されうる値への取り込み）に
まだ流れていない区間**では、オブジェクトを保持するのは現スレッドだけなので Local を出せる。
窓が閉じるのはマークの時点ではなく「公開されうる値に取り込まれた時点」（コンテナ経由の
間接マークがあるため）。ループ・再帰でマークを跨ぐ site — ある反復でマークされ次の反復で
同じ site が触る形 — は、再帰呼び出しがパラメータに「公開済みかもしれない」を join する
ことで自動的に `MayExt` 側へ落ちる。

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
