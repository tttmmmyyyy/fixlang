# Array/Storage 表現の再設計 — 設計

ステータス: 設計のみ、未実装。`investigation-notes.md`(コード実地調査の生データ)を土台とする。
目的は `Array::@size` を register 読み出しにして、back end が write loop からそれを巻き上げ
(hoist)、要素ごとの bounds check を畳み、vectorize できるようにすること — bounds-check
elimination の write-loop 側(`../2026-07-18-bounds-check-elim/`)。read-loop 側(iterator の
終了条件変更 + RC-IR simplifier)は既に出荷済み。

## 0. 概要

**やること**: `Array` を **primitive tycon のまま**、その値レイアウトを boxed から **unbox 3-word
`{ SubObject(#ArrayStorage), size:I64, cap:I64 }`** に変える。要素の heap 領域(refcount + 生要素)は、クロージャの
キャプチャ領域と同型の **内部 tycon `#ArrayStorage`** として持ち、value からは `SubObject`(ポインタ)で指す。狙いは
`@size`/`@capacity` が **register 読み出し**になること — write loop の bounds check と `push_back` の容量チェックが
hoist/畳まれて vectorize する(write-loop BCE)。read-loop BCE は既に別途出荷済み。

**確定した設計判断**(詳細は各節):
- **storage は内部 tycon `#ArrayStorage`(`#DynamicObject` 流)** — `#`-prefix でユーザーが名前を書けず(漏れない)、
  `Boxed` instance も持たない。ユーザーにも std にも storage 型は現れず、Fix インターフェースは `Array` のもの
  (public/private・safe/unsafe)だけ(§2.2/§4/§11.1)
- `Array` は primitive を維持。値 unbox `{ SubObject(#ArrayStorage), size, cap }`、`#ArrayStorage` は
  `{ ControlBlock, buffer }`。クロージャ `{ 関数ポインタ, SubObject(#DynamicObject) }` と構造が並行で RC も同経路
  (placeholder-ty hack なし、§2.2)
- size/cap は value(3 word、C++ `std::vector` 流)、refcount だけ storage(§2.1)
- 要素解放は Array の custom traverser が value の `size` で駆動、storage は free のみ(§3, §3.1)
- RC-unit は PunchedArray と同じ「不可分 unit」扱い、名前付き述語で寄せる(§3.2)
- Array の uniqueness は専用 `Array::_unsafe_is_storage_unique`(storage の refcount を retain せず覗く)。generic
  `unsafe_is_unique` は存続(§3.3)
- bulk op(fill / append / reserve)は COW を内包した InlineLLVM primitive。`fill` はその 1 回呼び出し(§4)
- FFI ポインタ系は Array の InlineLLVM ヘルパ経由。retained-ptr は size/cap を運べない(§7)
- Fix レベルの uniqueness-check-less な mutate primitive は全廃し、mutate は全て COW 内包 + 値として純粋に揃える
  (`_unsafe_force_unique` / unreleased write / punch-plug の uniqueness-unchecked 版を削除、§5/§3.3/§11.2)

**進め方**: 表現の反転は 1 コミットで行い、それと独立に検証できる作業を前後に分ける(§9)。実装は設計確定後。

**やらないこと**: zero-copy slice(要素寿命の健全性が「全 sharer が同一 `_size`」に依存するため、§3.1)。

## 1. 問題

`Array a` は boxed primitive で、1つの heap allocation として次のレイアウトを持つ:

```
{ ControlBlock{refcnt i32, state i8}, len i64, cap i64, buf[FAM] }
```

そのため `get_size` は `extract_field(ARRAY_LEN_IDX)` — つまり **heap object からの load** に
lower される。`arr = arr.set(i, v)` を loop state に通す write loop では、`buf` への store が同じ
allocation を alias し、flexible-array-member (FAM) の GEP が宣言された struct 境界の外に出るため、
LLVM は `len` の load を loop-invariant と証明できない。よって bounds check `i < get_size(arr)` は
毎回 size を再 load し、残った check が vectorize を阻む(`--no-runtime-check` で bounds check を外すと
`array_mod` は命令数が -30.4%、`arrayrw` は vectorize して -95.0%(19.9 倍)= check が vectorize の上限。
`../2026-07-18-bounds-check-elim/speedtest-baseline.md`)。size が
heap にあることが root cause なので、直し方は解析パスの追加ではなく表現そのものの変更である。

## 2. 表現

`Array` は今も **primitive tycon**(`TyConVariant::Array`)のままとし、その **値レイアウトだけ**を「長さと容量を
value に持つ unbox 3-word」に変える。要素を格納する heap 領域(refcount + 生要素)は、**クロージャのキャプチャ領域
(`#DynamicObject`)と同型の内部 tycon `#ArrayStorage`** として持ち、`Array` の値はそこへの `SubObject`(ポインタ)を
1本持つ。

```
Array a  (primitive tycon)   // 値レイアウト: unbox { storage : SubObject(#ArrayStorage a), size : I64, cap : I64 }
   storage --> #ArrayStorage a  { ControlBlock, elem[FAM] }   // 内部 tycon。# 始まりでユーザーは名前を書けない
```

`#ArrayStorage` の指す heap object は refcount と生要素だけを持ち、メタ情報(size/cap)は value 側にある。
`#ArrayStorage` は `#DynamicObject` と同じ **内部 tycon**(`#`-prefix によりユーザーが名前を書けない)で、`Boxed`
instance を持たない。so ユーザーにも std にも storage 型は現れず、Fix インターフェースは `Array` のものだけ(§2.2、
§11.1)。value の field 0 を **`SubObject`**(生 `Ptr` でなく)にすることで、storage の RC がクロージャの capture と
同じ経路(フィールドの本物の ty で `Object` を作り直す)に載り、placeholder-ty hack が要らない(§2.2(4))。

すると `@size = extractvalue(arr, 1)`、`@capacity = extractvalue(arr, 2)` はどちらも **register 読み出し**。write
loop に通すと size は loop 越しに運ばれる scalar になり、`set` は同じ size を持つ新しい `Array` value を返すので、
LLVM は bound を loop-invariant と見なして `i < size` を畳み、vectorize する。同様に `push_back` の容量チェック
`size < cap` も register で回る(§2.1)。size-normalization パスも invariant-parameter の引き回しも要らず、
標準の LICM/SCEV から自然に出てくる。

`Array` は 3 word の by-value aggregate `{ ptr, i64, i64 }` になる。retain/release/要素寿命は generic な unbox 機構に
載せず、`Array` を **1つの不可分な custom-RC unit** として扱う(§3.2)— retain は storage の refcount を +1、
release は value の size で `[0..size)` の要素を解放して storage を free、という custom traverser が担う(§3)。C++
`std::vector` と同じレイアウト思想(heap は生要素、メタ情報は value)で、COW 共有のため refcount だけ storage
側に持つ点が異なる。

### 2.1 `_cap` をどこに置くか — 判断: value 側(C++ `std::vector` 流)

`push_back` は毎回 `@capacity` を読む(`if arr.@capacity < len + 1 { reserve } else { arr }` の容量
チェック)。よって配列を逐次構築する `push_back` ループでは `_cap` も **hot path**。ここが判断を決める:

- `_cap` を `Storage`(heap)に置くと、`push_back` ループの容量チェックは **heap load** になり、直後の
  要素 store が同じ Storage allocation に書くため **FAM-alias** でその load を hoist できず、毎回再 load
  になる — `_size` で直したのと同じ病気を `_cap` で再発させる。本再設計の趣旨(hot loop から metadata の
  heap-load を消す)と一貫しない。
- `_cap` を value に置くと(`{ _storage, _size, _cap }`、Storage は `{ ControlBlock, buf }`)、容量チェック
  `_size < _cap` は **register** で回り hoist 可能。`push_back` ループが tight になる。これが C++ `std::vector`
  が `{ ptr, size, cap }` を value に持つ理由。**採用。**

read-index ループ(本件の主目的)では `_cap` は loop 状態に乗るが未使用なので LLVM が DCE で落とす —
3 word 目は read loop ではタダ、`push_back` ループでは効く。value に置くのは `_size` と `_cap` の2つだけ
(それぞれ bounds check と容量チェックで hot に読まれる)。それ以外のメタ情報は無い。

代償: `Array` が by-value 3 word になる — 配列を渡す関数の ABI が太り、`Array (Array a)` のような入れ子では
要素あたりメタ情報のぶんメモリが増える。**cap を heap に置く案(value 2 word)と比べて +1 word、現行
(value 1 word = ポインタのみ)と比べて +2 word。** C++ vector も 3 word なので許容範囲とする(§10)。

### 2.2 `Array` プリミティブの定義(何を primitive にするか)

「`Array` プリミティブが何であるか」を codegen の言葉で決めきる。**要は、現行の boxed-primitive `Array` の codegen
(alloc / get / set / traverse / clone / size_of / debug)を残しつつ、(i) heap object を size/cap 抜きに縮め、(ii)
size/cap を value 側の unbox 3-word に出し、(iii) その heap 領域を、クロージャのキャプチャ領域(`#DynamicObject`)と
同型の内部 tycon `#ArrayStorage` にして value から `SubObject` で指す**、という変更に集約される。

**クロージャとの並行**が設計の背骨: クロージャは unbox 値 `{ 関数ポインタ, SubObject(#DynamicObject) }`、再設計後の
`Array` は unbox 値 `{ SubObject(#ArrayStorage), size, cap }`。どちらも「unbox 値 + refcount 付き heap 領域への
`SubObject`」で構造が並行になり、storage ポインタの RC がクロージャの capture と同じ経路に載る — **placeholder-ty
hack が要らない**(`Object` が本物の ty を持つため)。

**(1) tycon**: `Array` は `TyConVariant::Array` を維持(value レイアウトだけ変える)。storage 用に **内部 tycon
`#ArrayStorage a` を新設** — `#DynamicObject` と同型で、`#`-prefix によりユーザーが名前を書けない(grammar の
`capital_name` が英大文字始まりを要求、`#` は不可)内部 tycon。`Boxed` instance は与えない(FFI は Array レベル、
§7)。ユーザーからは storage 型が見えないので、「storage は漏れない」性質を型システムで保証できる。

**(2) 値レイアウト(unbox ObjectType)**: `is_unbox = true`、`field_types = [ SubObject(#ArrayStorage a), I64, I64 ]`。

| index | 意味 | 定数(改称) |
| --- | --- | --- |
| 0 | `#ArrayStorage a` への `SubObject`(= ポインタ) | `ARRAY_STORAGE_IDX` |
| 1 | size(構築済み要素数) | `ARRAY_SIZE_IDX`(旧 `ARRAY_LEN_IDX`) |
| 2 | capacity | `ARRAY_CAP_IDX` |

field 0 は `SubObject`(boxed へのポインタ)なので **generic RC が boxed leaf として認識する** — クロージャの capture
フィールド(`SubObject(#DynamicObject)`)と同じ扱い。retain は generic に回り(下 (4))、要素寿命だけ custom。

**(3) `#ArrayStorage a` レイアウト(内部 tycon、boxed)**: `is_unbox = false`、
`field_types = [ ControlBlock, <非 traverse な要素 FAM> ]`。

| index | 意味 | 定数 |
| --- | --- | --- |
| 0 | ControlBlock(refcount) | `STORAGE_CTRL_IDX` |
| 1 | 生要素の FAM(RC-inert) | `STORAGE_BUF_IDX` |

size/cap を持たない。`#ArrayStorage` の destructor は **free-only**(生メモリ解放のみ)で、`#DynamicObject` が持つ
**stored `TraverseFunction` は不要**(要素は型付きで、Array value の traverser が解放する)。**要素 FAM には新しい
RC-inert(非 traverse)な `ObjectFieldType` variant を1つ設ける**: 現行 `ObjectFieldType::Array(elem)` は length で
要素を traverse する(`build_traverse` が length を読んで loop)ため storage には流用できない(storage は length を
持たず、要素解放は Array value が駆動するので、二重解放や存在しない length 読みになる)。新 variant は `build_retain`
/ `build_traverse` で **no-op**、`to_struct_type` の「+1 要素分」サイズ計算だけ `Array` variant と同じにする。これで
`#ArrayStorage` の release は生メモリ free だけになる。(`to_struct_type` の `!is_unbox` assert は boxed な
`#ArrayStorage` 側に残し、unbox な `Array` value には課さない。)

**(4) RC**: value field 0 が `SubObject(#ArrayStorage)`(boxed leaf)なので、機構はクロージャの capture と同じ経路に
載る — retain/release は **フィールドの本物の ty で `Object` を作り直して**低レベルヘルパへ渡す(`generator.rs` の
`SubObject` 分岐、`object.rs` の traverse 分岐)。`Object` が本物の `#ArrayStorage a` ty を持つので **placeholder-ty
hack は不要**。
- **retain = generic**: unbox `Array` の retain が field 0(`SubObject(#ArrayStorage)`)を retain = storage の
  refcount を +1(shallow、COW で要素は共有のまま)。custom コード不要。
- **release / mark = custom**: `#ArrayStorage` を generic に release すると free-only destructor が要素を残して漏らす。
  そこで `Array` value に custom `build_traverse` arm を置き、**storage の `Object` を
  `build_release_mark_nonnull_boxed_with(storage, work, traverse_refs)` に渡して、`traverse_refs` クロージャの中で
  value の `_size` を読んで `[0.._size)` を解放する**。refcount の減算と destruction 分岐は共通機構が持ち、
  **`traverse_refs` は refcount が 1 -> 0 に落ちるときだけ呼ばれる**ので、共有されている配列の要素は解放されない
  (要素解放を自分で書いて `build_release_mark` を呼ぶ形にすると、refcount を見る前に無条件解放することになり、
  boxed 要素を持つ共有配列で use-after-free になる)。`#DynamicObject` が「captures を解放 -> 自身を free」するのと
  同じ骨格で、現行 `ObjectFieldType::Array` の custom traverse arm(value から length を読んで loop)と PunchedArray の
  hole-skip arm(同じヘルパに hole を飛ばすクロージャを渡す)が雛形になる。mark work では共通機構が `traverse_refs` を
  無条件に呼ぶので、mark 側も同じ形で正しい。
- **clone(COW)**: 新 `#ArrayStorage` を alloc し `[0..size)` を retain コピー、value field 0 を差し替え。

retain が generic に回るので、field 0 を生 `Ptr` にした場合の「全 unboxed と誤判定 -> RC を何も出さず leak」問題は
そもそも起きない。custom は release/mark の要素ループだけに閉じる(§3.2)。

**(5) アクセサ**: `Array` は struct ではないので `@field` の自動 getter は無い。`@size` / `@capacity` は現状と同じ
**手登録の builtin InlineLLVM**で、heap load でなく value への `extractvalue`(index 1 / 2)にする。storage Ptr
(index 0)を読むのは codegen 内部だけで、Fix レベルの op にはしない。

**(6) primitive op**(すべて上のレイアウトに対して codegen。契約の詳細は §4 / §13):
- alloc `_unsafe_empty_capacity_unchecked(cap)`: `#ArrayStorage a` の object を `size_of(cap)` で malloc(標準の
  boxed alloc 経路)、ControlBlock を refcount 1 に初期化、value `{ SubObject, 0, cap }` を構築。
- `_unsafe_get_bounds_unchecked(i, arr)`: storage buffer の i 番目を load して retain(borrow arr)。
- `_unsafe_append_value_capacity_unchecked(x, n, arr)`: COW してから buffer の `[_size, _size+n)` へ `x` を store
  (未初期化スロットなので旧値 release なし)、value の `_size` を n 増やす。`arr` を消費して新しい Array value を返す(§4)。
- `set` / `swap` / `mod`(punch/plug)/ `act` / `_unsafe_truncate_bounds_unchecked` / `mutate_elements`: COW 内蔵の in-place mutator。
- `_unsafe_grow_size(n, arr)`: COW してから value field 1(size)を n に。
- array literal: alloc + fill + value 構築(既存 literal codegen を新レイアウトへ)。
- FFI `borrow_elements`(arr を Borrow operand 宣言、内部 retain 不要)/ `mutate_elements`(COW 内蔵): buffer 先頭
  ポインタを callback へ渡す(§7、§13.3-2)。

## 3. 要素の寿命 — 中核をなす判断

現状、`Array` の destructor は「何要素を release すべきか」を `len` から知る。`len` は storage と同じ heap
object にある(`build_traverse` の `Array` arm: `size = extract_field(ARRAY_LEN_IDX)`、続けて
`release_or_mark_array_buf(size, buf, ..)`)。`_size` が value に移ると、`Storage` 単体では live 要素数を
知れない。

**採用: `Array` value が要素 release を駆動する。** `Array` の release/mark/clone を custom traverser にし、
その各点で手元にある value の `_size` を使って `_storage` の生要素領域を歩く(`Storage` は untyped な生メモリとして
扱う)。`_size` を value だけに保てて(再設計の眼目)、しかも *現行* `Array` destructor が既に回しているロジック
そのもの — 変わるのは `size` の出所が heap load から value field になる点だけ。(検討して退けた代替案は §11 末尾。)
具体的には:

- **retain** `Array` = `_storage` の control-block refcount を +1(shallow、変更なし。COW で要素は共有のまま)。
  これは通常の boxed-field 伝播なので、retain については「boxed field を1つ持つただの unbox struct」で正しい。
- **release** `Array` = `_storage` が unique なら `for i in 0.._size { release(buf[i]) }` して `_storage` を free、
  そうでなければ `_storage` の refcount を -1。`_size` は value から来る。`Array` は現行 `Array`・`PunchedArray`
  と同じく custom な `build_traverse` arm を保ち、要素数駆動の traversal を value 読みに移すだけ。
- **clone-if-shared**(refcount >= 2 での mutate 時の COW)は `_size` 個の要素を新しい `Storage` へ複製し、
  各要素を retain する。clone 側も `_size` を手元に持つ。
- storage object の boxed destructor は **生メモリを free するだけ** — 要素 release は決してしない。所有側の
  `Array` が `_size` で駆動して既に release 済みだから。storage は内部 tycon `#ArrayStorage`(ユーザーが名前を書けない、§2.2)で、
  ユーザコードは `Array a` を持ち、裸の storage は持たない。よって裸の storage が自前で要素数を必要とする場面はない。
- free は `free(ptr)`(現行の `build_free` と同じ、サイズ不要)なので `Storage` は cap を持たなくてよい。
  `_cap` を使うのは allocation 時(malloc バイト数 = `offset_of(buf) + elem*_cap`)と `push_back` の容量
  チェックだけで、どちらも value の `_cap` で足りる。

### 3.1 共有の不変条件(なぜ (b) が heap count なしで健全か)

共有は `retain` からのみ生じ、`retain` は `Array` value 全体を複製する — よって同じ `Storage` を共有する者は
全員 **同じ** `_size` を持つ。refcount >= 2 での最初の mutate が `Storage` を clone(COW)し、その共有者は
自分専用の `Storage` を得る。共有中に `_size` が食い違うことはない。したがって最後の release(`_storage` の
refcount 1 -> 0)は常に正しい `[0.._size)` に対して要素 release を駆動する。これが成り立つのは、コア設計に
**zero-copy slice が無い**からこそである(slice があれば同じ `Storage` を小さい `_size` で見る共有者ができ、
共有者間で `_size` が食い違って不変条件が壊れる)。よってコア設計は常に `_size` == `Storage` の構築済み要素数
を保つ(zero-copy slice はやらない)。

**この不変条件は「`_size` を変える op はすべて unique な `_storage` にだけ適用される」ことに依存する。**
redesign では `_size` が value にあるので、**共有された `_storage` に size を書くと、その holder だけ `_size` が
食い違い、release 駆動がズレて要素を leak する**(小さい `_size` の view の外の要素を誰も release しない。現状の
heap len なら共有 len を書き換える別の誤り。redesign の方が危険)。
これを op 自身で保証する:

- `_unsafe_set_size` は要素側 Array ではほぼ **増加専用**(from_map/fill/reserve/push_back/append は
  alloc/reserve 後に size を伸ばす)。**減少は `_unsafe_truncate_bounds_unchecked`(危険トランケート = COW + `release_range([n,size))`
  + `size=n`、契約 `0<=n<=size`)が担う**。**安全な shrink は `truncate`/`pop_back`(公開)が size チェック後に
  `_unsafe_truncate_bounds_unchecked` を呼ぶ**(§13.2、§13.3-1)— `String::from_bytes` 等の `Array U8` shrink 経路は `truncate` を使う
  (unique-check-elim + unboxed release の no-op 化で、その経路では「`_size` を下げるだけ」に畳まれる)。
- そこで **増加専用の `_unsafe_grow_size`(「未初期化スロットへ size を伸ばす」、`n >= _size`)に置き換え、
  内部で unique check(= COW、shared なら clone)する**(名前を grow-only に合わせる — 現行 `_unsafe_set_size`
  は増加しかしない)。すると「呼び出し側が事前に unique を保証する」footgun が消え(op 自身が unique な `_storage`
  にしか size を書かない)、その内部 check は unique-check-elim が provably-unique で畳んで同性能にする
  (§11.2 の方向)。呼び出し側の `_unsafe_force_unique` は畳み込める。残る unsafe 契約は「新スロット
  `[old_size..n)` は未初期化(呼び出し側が埋める)」「`n <= _cap`」のみ。

### 3.2 RC-unit 機構との整合(PunchedArray と同じ特別扱い)

RC 挿入は値を **RC unit(boxed leaf)単位**に分解して retain/release を置く(`borrow.rs` の `rc_units_go` /
`clamp_unit`)。ただし `is_box` / `is_union` / `is_punched_array` は「1つの不可分 unit」として扱い、中へ
descend しない — その unit の retain/release は値全体の(custom)traverser 経由になる。現行の `Array` は
`is_box` なので自然にこの1 unit で、release すると Array の custom destructor が走る。

再設計後の `Array` は **unbox** なので、何もしないと generic な「field へ descend」枝に落ち、`_storage`(Storage)を
boxed leaf として**単独で RC** してしまう -> Storage の free-only destructor が走り要素を leak する。これが §3 の
coupling の機構的な正体。

**解決 = 新 `Array` を上記の不可分 unit 境界に加える**。こうすると `Array` は path `[]` の 1 unit になり、
retain/release/mark がすべて Array の custom traverser 経由(value の `_size` 駆動)になって、`_storage` が単独で
RC されることはない。**これは `PunchedArray` が既に取っている扱いそのもの**で、unique-check-elim / borrow /
provenance / codegen は「custom traverser 型を 1 unit として扱う」機構を既に持つ。

uniqueness(`set` の make_unique)は「`Array` unit = その `_storage` の refcount が unique か」で判定でき、provenance が
追う `_storage` leaf を `clamp_unit` が `Array` unit に丸めて突き合わせる(現行の union/is_box と同じ経路)。よって
per-unit の retain/release とも uniqueness 判定とも噛み合う。`PunchedArray` 自身は、custom traverser が読む値が
「内側 array の heap `len`」から「内側 `Array` の value `_size`」に変わるだけで、依然 1 unit・hole skip のまま。

**反転後、型レベルの `field_types` は値レイアウトを表さなくなる。** `field_types(Array a)` は **要素型**を返す
(`types.rs` のコメント「For Array, return the element type」)。現行は `Array` が `is_box` なので型を辿る walker が
全部そこで止まり、この食い違いが露出していない。反転すると露出する。よって不変条件を 1 つ置く:
**型を辿って値のレイアウトを記述する walker は、`Array` を必ず不可分 unit として止める。** 直す箇所は 3 つ。

- **`TypeNode::is_rc_unit_root` に `is_array()` を足す。** この述語は既に導入済みで、`rc_units_go` / `clamp_unit` /
  ownership shape / `subtree_type` が経由するので、これで揃う。
- **`TypeNode::is_fully_unboxed` を `rc_units` の上に定義し直す**(「RC unit を 1 つも持たない」)。現在の
  `is_fully_unboxed` は `is_box` でなければ `field_types` を再帰するので、反転後は `Array U8` / `String` /
  `Array (Array I64)` を含む**すべての `Array`** を「RC 不要」と判定してしまう。この述語は
  `rc_insert::needs_rc`(retain/release ノードを出さない)、`object::create_traverser`(custom traverser を
  生成しない)、`borrow.rs` の 2 箇所、`provenance::boxed_leaf_paths` で load-bearing で、**しかも `borrow.rs` では
  `is_rc_unit_root` の判定より前に early return する**。放置すると storage が解放されず(全 `String` が leak)、
  retain が出ないので COW が発動せず(`let b = a.set(..)` が `a` を破壊)、`reserve` の realloc で use-after-free に
  なる。`is_array()` で早期 return する形にすると 2 つの述語が別々の再帰を持ち続けて再び乖離し得るので、
  **定義を 1 本にする**(`rc_units` は `src/rc_ir/borrow.rs`、`is_fully_unboxed` は `src/ast/types.rs` にあるが、
  層をまたぐことより定義の一本化を優先する)。`rc_units` は `Vec` を確保するので、短絡する
  `has_rc_unit(ty) -> bool` を同じ走査から作り、`is_fully_unboxed = !has_rc_unit` とする。
- **`provenance::boxed_leaf_paths` に `Array` で止まる分岐を独立に足す**(`is_box` の枝と同形で、その時点の path を
  1 leaf として push)。この関数は union を variant へ展開する必要があるため `is_rc_unit_root` を意図的に
  使っておらず、述語の統一では届かない。定義を一本化した `is_fully_unboxed` だけでは、`Array U8` が
  「fully unboxed ではない」を通過した後に `field_types` = `[U8]` へ descend して leaf ゼロになる。

### 3.3 `unsafe_is_unique` は Array の unbox 化で壊れる — 要修正

`unsafe_is_unique`(`InlineLLVMIsUniqueFunctionBody`)は現状 `if obj.is_box() { refcount を読む } else
{ const true }`(「unboxed object は常に unique」)。**Array が unbox になると else 枝に落ち、`_storage` が共有
(refcount >= 2)でも無条件に `true` を返す。** すると `mod`/`act`/COW の `if is_unique { in-place } else
{ clone }` が常に in-place を選び、**共有 storage を破壊(データ破損)**する。redesign の重大な破損点。

修正(generic `unsafe_is_unique` に Array 特別扱いを入れない): **`Array` 専用のビルトイン
`Array::_unsafe_is_storage_unique : Array a -> (Bool, Array a)` を追加**し、value の `_storage` の refcount を
**retain せずにその場で覗く**。`act` など Array の COW 判定はこれを使う。generic `unsafe_is_unique` は
現状のまま存続する — boxed 型(`Destructor`〔mutate_unique_io〕、FFI の gmp/mpfr 等、generic `assert_unique`)に
`is_box` で効き続けるため。unbox な Array *value* に generic を呼ぶと `const true`(値としては常に unique)を返すが、
COW 判定は Array 専用版を使うので破壊は起きない。

- storage を値として取り出して uniqueness を見る案は不可: 取り出しには retain が要り、borrow 化されない限り
  refcount >= 2 になり大抵 false を返す。専用ビルトインなら Array value を受けて storage Ptr(field 0)を retain せず
  その場で refcount を読めるので確実(そもそも `#ArrayStorage` はユーザーが名前を書けず、値として取り出す API も無い、§2.2)。
- unique-check-elim の static fold は Array 専用版に適用(`_storage` が provably-unique なら const-`true` に畳む)。
  ランタイム版(`InlineLLVMIsUniqueFunctionBody` 相当を Array 用に)と fold の両方を用意する。
- **属性は generic `unsafe_is_unique` と同型にする**: `unique_check_operand = Some{0, []}`、
  `assuming_unique` が const-`true` 版を返す、`borrows_operand` は無し(operand 0 を意図的に consume)、
  `result_prov` は **`Dyn` 固定**。最後の 1 つは generic 版と同じ理由による — 第 2 成分は引数そのものだが、
  passthrough(`Arg`)と宣言すると borrow パスがそれをエイリアス兼「非消費」と解釈し、後続 use が引数を
  shared に読ませる retain を抑止するので、共有 storage を unique と誤判定する。
  同型にしておくと、unique-check-elim 側で検討されている「`is_unique` の true 枝で operand を `Unique` と
  解釈する経路依存の精密化」(`unique-check-elim` ブランチの
  `dev-docs/2026-06-28-unique-check-elim/findings-2026-07-20-provenance-gaps.md`)が
  入ったとき、この op もそのまま対象になる。精密化が op の具象型で判定する実装になった場合は、この op を
  認識対象に加えること。`act` の unique 枝の COW punch はそれで畳めるようになる。
- さらに **generic `unsafe_is_unique` に `[a : Boxed]` 制約を付ける**(現状は無制約で unboxed に `const true`
  を返す)。こうすると Array を unbox にした瞬間 `arr.unsafe_is_unique` が **型エラー**になり、silent な
  誤 const-true を型システムが弾いて `Array::_unsafe_is_storage_unique` へ誘導できる。`else { const true }` 枝は
  dead になり除去可。現状 unboxed 型に呼んでいる箇所は無い(`assert_unique` も Array=boxed にしか使われて
  いない)ので今は無害で、intended な破壊は redesign で Array が unbox になる時だけ。波及: `assert_unique :
  Lazy String -> a -> a` も `unsafe_is_unique` を呼ぶので `[a : Boxed]` になり、redesign 後 `arr.assert_unique`
  は型エラー(本来 arr には誤答なので望ましい破壊)。Array 用の uniqueness assert(`_unsafe_is_storage_unique`
  ベース)を別途用意する。

is_unique 分岐(`build_branch_by_is_unique`、**Rust/コンパイラ側のコード**)の用途は3つ: (1) **COW mutate**
(`make_array_unique_with_hole`〔set/mod/swap/punch〕、`make_struct_union_unique`)、(2) **release の
free-or-decrement**(`build_release_mark_nonnull_boxed_with`)、(3) **`is_unique` 関数**。これら(Rust/
コンパイラ側の COW/release 機構)は存続し、unique-check-elim が (1)(3) を **コンパイル時に畳んで消す**((2) は
基本判定で残す)。redesign では (1) が Array の `_storage` に移るだけ。

**Fix レベルの uniqueness-check-less な mutate primitive は全廃する**(規則と理由は §5): `_unsafe_force_unique`
(`append`/`push_back`/`resize`/`sort_by`/`reverse` の前処理)、
`_unsafe_set_bounds_uniqueness_unchecked_unreleased`(同 builder 群)、**punch/plug の uniqueness-unchecked 版**
(`mod` / `_unsafe_act_bounds_unchecked_identity` / `_unsafe_act_bounds_unchecked_tuple2` の plug、
`_unsafe_act_bounds_unchecked` の punch)。移行先はいずれも COW を op 内に内包した版で、`_unsafe_grow_size`(§3.1)は
既にこの形。builder 群の移行先は §4 の 4 primitive。

**punch/plug を COW 版へ寄せるには `InlineLLVMArrayPunchBody` に `result_prov` の実装が要る**(punched-array leaf =
`Fresh`〔force_unique 版〕、moved-out 要素 leaf = `Dyn`)。既定の全 leaf `Dyn` のままだと、`mod` の plug の operand が
`Dyn` になり unique-check-elim が畳めず、runtime check と clone path が残る。`act` の punch は operand が
`unsafe_is_unique` の結果(`Dyn` 固定、上述)なので RC-IR では畳めず、背中合わせの 2 チェックを LLVM の GVN が
統合する Max でだけ消える。

注: `unreleased`(未初期化スロットへの書き込み)の unsafe さは uniqueness とは直交で、
`_unsafe_append_value_capacity_unchecked` が capacity 契約だけを残す形に整理される(§4)。

## 4. `Array` を組む primitive(storage は codegen 内部に閉じる)

`#ArrayStorage` は **`Array` の内部表現でしかない**(§2.2)。ユーザーが名前を書けない内部 tycon で、storage を直接
扱う Fix 関数 API も無い。生ストレージへの操作(領域確保 / 要素 read / 未初期化 write / データポインタ取得)は **`Array` レベルの
InlineLLVM body の codegen の中**で行い、Fix ソースからは `Array` のインターフェース(public/private・safe/unsafe)
だけを触る。狙いは、`unique-check-elim` plan §8(2)(a)〔DOESNT-FIT〕型の **「隠れ穴を作る composable primitive」**
— no-retain read で要素を move-out する raw op のように、2 属性(`borrows_operand` / `result_prov`)で表せず RC
解析が借用要素を所有と誤認する op — を **Fix レベルに一切露出させない**こと。生ストレージ op を `Array` の
InlineLLVM に閉じ込めれば、借用や穴は op の codegen の内側で完結し、値として escape しない。

`_unsafe_` は memory 安全性を壊し得る op(bounds unchecked / 未初期化スロット write / 未初期化 storage の生成)に
だけ付ける。safe な size/capacity 読み出しは value の field 読み(または public `get_*`)。

**Fix レベルに置く `Array` primitive(生ストレージに触るが露出は `Array` 型に閉じる):**

- `Array::_unsafe_get_bounds_unchecked : I64 -> Array a -> a` — 要素 read(bounds unchecked)。返り値を
  **retain する**(boxed 要素なら refcount +1、caller が所有・release)。InlineLLVM が `arr.@_storage` を直接読む。
  **非 retain(unretained)版は作らない** — plan §8(2)(a) の「型に現れない穴」(runtime index には static な
  provenance path が無く、借用要素が所有と誤認されて double-free になる)を再現するため。read-fold の要素アクセスは
  この retaining getter を通す(`ArrayIterator::advance` もこれを使う)。
  **boxed 要素の read-fold では per-element の retain/release が残る。** 現状の retain はこの getter の codegen の
  内側で発行されるので、明示 `Retain`/`Release` ノードのペアを消す相殺(§2.2)には対象が見えない — borrow 化した
  fold body にも要素の `release` が 1 反復ごとに残る(RC IR dump で確認)。LLVM も、release の「rc==1 か」の分岐を
  消すには retain 前に rc>=1 を証明する必要があるため畳まない。これは `Array::@` / boxed struct のフィールド読み /
  union payload 読みに共通する既存の性質で、**この再設計が悪化させるものではない**(削除する unretained getter の
  実利用は unboxed 要素の 1 箇所だけなので、既存コードの性能はこの削除で変わらない)。0 にするには「op の内側で
  retain する」現在の機構を変える(retain を明示ノードとして出し、相殺の対象にする等)必要があり、それは `Array`
  表現とは直交する optimizer 側の課題。**unretained getter の再導入では埋めない**。
- `Array::_unsafe_append_value_capacity_unchecked : a -> I64 -> Array a -> Array a` — **末尾へ `x` を n 個**:
  COW(内部 unique check)-> `build_retain(x, n)` -> `[_size, _size+n)` へ write(未初期化スロットなので旧値
  release 無し)-> `_size += n` -> **`release(x)` を 1 回**、を **1 op で**行う。契約は **`n >= 0`** と
  **`_size + n <= _cap`**(uniqueness も初期化状態も op 内で閉じる)。`x` の所有権を 1 個受け取って n 個の参照を
  格納するので、正味必要な追加は **n-1**。retain n 回ぶん + release 1 回にすると、**`n == 0` も自然に正しくなる**
  (retain 0 回 + release 1 回 = `x` を手放す)。旧 `_unsafe_set_bounds_uniqueness_unchecked_unreleased` と
  `array_unsafe_fill` の後継。**write と `_size` 更新が同じ op に入るので、op 境界で配列 value が常に valid**
  (`_size` に含まれる未初期化スロットが Fix レベルに現れない)。値としても純粋なので §5 の CSE 制約も受けない。
- `Array::_unsafe_append_capacity_bounds_unchecked : Array a -> I64 -> I64 -> Array a -> Array a` —
  **src の `[begin, end)` を dst の末尾へ**: dst を COW -> src が unique かつ範囲が src 全体なら memcpy して
  src の `_size` を 0 に(move、RC 操作ゼロ)、それ以外は `load ptr -> retain -> store` の融合 1 パスでコピー ->
  src を release -> `dst._size += end - begin`。契約は `0 <= begin <= end <= src._size` と
  `dst._size + (end - begin) <= dst._cap`(呼び出し側)。`append` は全範囲の呼び出し、`get_sub` と mergesort の
  merge もこれで書ける。詳細は下の表の後。
- `Array::_unsafe_set_capacity_bounds_unchecked : I64 -> Array a -> Array a` — **容量の張り替え**:
  unique なら storage を `realloc`(要素に触らない)、shared なら新 storage へ retain 付きコピー。
  契約は `new_cap >= _size`(呼び出し側)。`reserve` がこれで書ける。詳細は下の表の後。
- `Array::_unsafe_grow_size : I64 -> Array a -> Array a` — COW + value の `_size` を伸ばす(新スロットは未初期化)。
  **用途は FFI 出力バッファのみ**(`Array::empty(k)` を確保し `mutate_elements` で C 側に埋めさせる
  `to_bytes`/`read_n_bytes`/`unsafe_from_c_str_ptr`)。この形では埋め終わるまで `_size` に未初期化スロットが
  含まれるので、**要素型が boxed 値を含まないこと**(U8/I64 等)が条件になる — traverse が要素に触れないので
  未初期化スロットが release されない。**この用途と条件を doc コメントに明記する**(現 `_unsafe_set_size` の
  doc は「長さを検証せずに更新する」としか書いておらず、これが builder 用の汎用 op に見えることが
  `String::from_bytes` が共有配列を縮めていた不具合の下地になった)。
- `Array::_unsafe_empty_capacity_unchecked : I64 -> Array a` — 指定 capacity の空 `Array` を確保(storage 未初期化、
  refcount 1)。InlineLLVM が内部で `#ArrayStorage` を alloc し Array 値 `{ SubObject(#ArrayStorage), 0, cap }` を構築。`empty`
  の後継(name/contract 不変)。要素数は malloc サイズ計算に使うだけで storage には保存しない(`_cap` が覚える)。

「**上書き + 旧要素 release**」の write(`set`/`swap`)と in-place mutate(`mod`/`act`/`mutate_elements` 等)は、
いずれも **COW を op 内に内包した `Array` レベルの InlineLLVM**(§5)。生ストレージへの直接 write + 旧要素 release は
その body の codegen が行い、Fix ソースから「storage への上書き + release」を呼ぶ経路は無い。`mod`/`act` は punch/plug
(hole へ書くので release 不要)。`push_back`/`append`/`resize` は **これらの InlineLLVM プリミティブを合成する
Fix-source ビルダー**(単独のプリミティブではない、§5)。

- bounds check(`_check_range`)は value の `_size` に対する `Array` レベル op として残す。

**実行時チェックの規則(新しく書くチェックすべてに適用)。**

1. **`--no-runtime-check` で必ず消えること。** Fix ソースで書くなら `_check_range` / `_check_size` を通す。
   InlineLLVM で書くなら `if gc.config.runtime_check() { ... }` で囲む(`set` / `swap` の既存 body が例)。
   新設する `unsafe_set_bounds_unchecked` のように「チェックを省くこと」が名前の意味である op は、
   safe な呼び出し側がチェックを持つ。
2. **gate されたチェックは純粋な assertion であること** — 失敗経路は abort(`fixruntime_index_out_of_range` /
   `undefined`)に限る。**失敗しても定義された結果を返す分岐を gate してはならない。** gate されたチェックを
   外したときに挙動が変わってはならず、`--no-runtime-check` は「元々チェックに引っかからないプログラムが、
   挙動そのままで速くなる」だけのものだからである。
3. したがって以下は **gate しない**(意味論の一部であり、チェックではない): `push_back` の
   `if @capacity < len+1 { reserve }`、`pop_back` の `if _size == 0 { arr }`、`truncate` の
   `if n >= _size { arr }`、`reserve` の早期脱出。gate するのは `@` / `set` / `mod` / `act` の
   `_check_range`、`fill` / `empty` の `_check_size` のように、失敗がプログラムの誤りであるものだけ。
   `truncate` の `n < 0 -> undefined` は素の Fix の `undefined` なので常時有効な assertion であり、
   `--no-runtime-check` でも残る。
4. 新しく gate 付きチェックを足したら、`--no-runtime-check` で消えることのテストを添える
   (`test_array_bounds_check.rs` の `test_set_bounds_check_respects_no_runtime_check` が雛形)。
- **FFI のデータポインタ**は `Array` の FFI ヘルパ経由(§7 `Array::borrow_elements` / `mutate_elements`)。storage は
  Boxed 値でないので generic `borrow_boxed` へは委譲せず、いずれも **Array の専用 InlineLLVM**が storage を直接
  扱う(`borrow_elements` は arr を Borrow operand と宣言し buffer 先頭 ptr を callback へ渡すだけ〔内部 retain 不要〕、
  `mutate_elements` は COW 内蔵、§13.3-2)。codegen 側は `get_data_pointer_from_boxed_value` の array 分岐を storage の
  buf レイアウトへ合わせる(§7)。capacity は value の cap が持つ。

**builder が要る storage 操作は 4 種類なので、alloc と合わせて primitive は 5 つ**。いずれも uniqueness の
判定を op 内に持つので §5 の規則に適合し、値としても純粋:

| primitive | 契約 | 束ねる関数 |
| --- | --- | --- |
| `_unsafe_empty_capacity_unchecked(cap)` | — | `empty` |
| `_unsafe_set_capacity_bounds_unchecked(new_cap, arr)` — 容量を張り替える | `new_cap >= _size` | `reserve`(将来の `shrink_to_fit`) |
| `_unsafe_append_value_capacity_unchecked(x, n, arr)` — 末尾へ `x` を n 個 | `n >= 0`、`_size + n <= _cap` | `push_back`(n=1)、`fill`、`resize` の伸長 |
| `_unsafe_append_capacity_bounds_unchecked(src, begin, end, dst)` — src の `[begin, end)` を dst の末尾へ | `0 <= begin <= end <= src._size`、`dst._size + (end - begin) <= dst._cap` | `append`(全範囲)、`get_sub` / `String::get_sub`、mergesort の merge |
| `_unsafe_truncate_bounds_unchecked(n, arr)` — `_size` を n に | `0 <= n <= _size` | `pop_back`(n=`_size-1`)、`truncate` |

- **`_unsafe_set_capacity_bounds_unchecked` は unique なら `realloc` する。** storage は単一の malloc ブロックで、
  ControlBlock は `{refcnt, refcnt_state}` だけ(traverser を持たない)なので、`realloc(storage, size_of(new_cap))`
  がそのまま使える。返ったポインタを value の field 0 に入れ `_cap` を更新するだけで、**要素には一切触らない**
  (refcount 不変、しばしばその場で伸び、大きい配列では `mremap` でコピーごと消える)。shared なら新 storage を
  alloc して `[0,_size)` を retain 付きでコピー(既存の `clone_array_buf` に capacity 引数を足した形)。
- **`_unsafe_append_capacity_bounds_unchecked` は move と copy を選ぶ**:
  - **src が unique かつ `begin == 0 && end == src._size`**: `memcpy` してから **src の `_size` を 0 にして**
    release。`_size` は value 側なのでこの書き換えはローカルで、release は要素を辿らず storage を free する
    だけになる。**RC 操作ゼロ**。
  - それ以外: 他の holder も同じ要素を参照し得るので、dst 用の参照が要る。`load ptr -> retain -> store` の
    融合 1 パスでコピーし、src は release(decrement のみ)。retain の回数は現行と同じで、**余分な alloc は無い**。
  - unboxed 要素ではこの分岐自体が消え、`memcpy` + release だけになる。
  - **move が全範囲に限られる理由**: 部分範囲を move すると、範囲外に残った要素を解放する主体がいなくなる。
    要素の解放は value の `_size` が駆動する(§3.1)ので、「中だけ抜けた storage」を表現できない。全範囲かどうかの
    判定は register 比較 2 回で、`append` からは常に真になる。
  - **COW するのは dst だけ**(src 側は move/copy を選ぶ読み取りテスト)。よって宣言する
    `unique_check_operand` は 1 つで足り、現状の単数機構のままでよい。src 側のテストも畳みたくなったら、
    そのとき機構を複数形にする(畳めなくても refcount の load と分岐 1 回で、得るのは RC 操作 2n の削減)。
  - **memcpy でよく、オーバーラップ判定は要らない**: dst は COW 済み、src は unique 枝でのみ move される。
    unique = refcount 1 なので、2 つの unique な値が同じ storage を指すことは定義上あり得ない。
- **append-n が n を取る利点**: boxed 要素の `fill(n, x)` で、`x` の retain を **増分 n の 1 回**にできる
  (n 回インクリメントしない)。unboxed 要素なら本体は store だけ。
  **並行実装は作らず、`build_retain` に増分の引数を足す。** 呼び出しは 6 箇所しかないので、ラッパを挟まず
  `build_retain(obj, amount)` にして既存呼び出しは定数 1 を渡す。増分は末端まで通す —
  unbox 集約の `SubObject` 再帰、`ObjectFieldType::retain_union`、そして `retain_nonnull_boxed` の
  `refcnt_state` 3 分岐(release/mark と共用のヘルパがあれば retain 経路にだけ引数を足す)。
  retain は単なる加算ではないので、並行実装を作ると 3 つの分岐を写し間違える: dynamic object(クロージャのキャプチャ領域)の **null 分岐**
  (キャプチャ無しクロージャの capture は null なので、`Array::fill(10, |x| x+1)` が segfault する)、
  unbox 集約の **per-field dispatch**(union はタグで dispatch。「要素の refcount」が 1 つとは限らない)、
  `refcnt_state` の **3 分岐**(LOCAL は非 atomic add、THREADED は `atomicrmw add`、GLOBAL は no-op。素の加算は
  `--threaded` で lost update から use-after-free、GLOBAL では触ってはいけない語を汚す)。
- **truncate-n はループを生まない**: `_size` の更新は 1 回の減算。要素 release は boxed 要素のときだけ n 回必要で、
  それは本質的な仕事。`pop_back` は release 1 回 + 減算 1 回に落ちる。
- **早期脱出は primitive でなく Fix-source 側に置く。** `reserve` の `new_cap <= _cap` や `append` の
  `src._size == 0` を primitive の中で早期 return にすると、**dst を COW しない経路**ができ、結果が unique とは
  限らないのに `result_prov = Fresh` を宣言することになる(嘘)。`result_prov` は経路ごとに変えられないので
  `Dyn` に落とすしかなくなり、下流の COW チェックが畳めなくなる。Fix-source の `if` に置けば、`if` の結果の
  provenance が「素通りした値のもの ∨ `Fresh`」の合流になって正しい。範囲のクランプ(`get_sub` の
  `max(0, s)` / `min(len, e)`)も同じ理由で Fix-source 側に置き、primitive は契約を満たす範囲だけを受ける。
- `from_map` は `_unsafe_empty_capacity_unchecked` + `push_back` ループの素朴実装でよい。コールバック呼び出しが
  支配的で、利用頻度も低い。

**現行実装との比較**(容量が足りている `append`、n = src の要素数)。現行は v1 を `_unsafe_force_unique` してから
`v2.@(idx)`(bounds check + retain)をループする:

| ケース | 現行 | 新 |
| --- | --- | --- |
| src unique / boxed | bounds check ×n + retain ×n + store ×n + src の destruction で release ×n + free | memcpy + `_size=0` + free(**RC 操作 2n が消滅**) |
| src shared / boxed | bounds check ×n + retain ×n + store ×n + decrement | 融合コピー(retain ×n)+ decrement |
| unboxed | bounds check ×n + store ×n | memcpy |

**bulk であること自体は規則に反しない** — §5 が禁じるのは uniqueness-check-less な primitive であって、
1 op で複数スロットを触ることではない。よって測定で回帰が出た場合の受け皿は、uniqueness チェックを持たない
write primitive を足すことではなく、**COW 内蔵の bulk op を足すこと**。`_unsafe_fill_size_unchecked` は削除する
(`fill` は append-n の 1 回呼び出しになる)。

これらは現行 `Array` の InlineLLVM body(`_unsafe_get_bounds_unchecked`、
`_unsafe_set_bounds_uniqueness_unchecked_unreleased`、`_unsafe_empty_capacity_unchecked`、`create_obj`)と 1 対 1 に
対応し、`Array` object の `ARRAY_BUF_IDX` ではなく `#ArrayStorage` の生要素領域(`STORAGE_BUF_IDX`)に対して動くように移す。

## 5. `Array` primitive の移行

現行の `Array` InlineLLVM primitive はそれぞれ、value `{ _storage, _size, _cap }` と `#ArrayStorage` の上に
再構成する(InlineLLVM のまま storage を直接扱うものと、他の primitive を合成する Fix-source になるものがある)。
「実装」列がその別を示す。完全な一覧は `investigation-notes.md` §5:

| 現行 `Array` primitive | 移行後 | 実装 |
| --- | --- | --- |
| `@size`(`extract_field(ARRAY_LEN_IDX)` = heap load) | `extractvalue(value, ARRAY_SIZE_IDX)` — register 読み出し | **InlineLLVM**(手登録 builtin のまま。body を heap load から value への `extractvalue` に。Array は primitive で struct getter は無い、§2.2(5)) |
| `@capacity`(`extract_field(ARRAY_CAP_IDX)`) | `extractvalue(value, ARRAY_CAP_IDX)` — register 読み出し | **InlineLLVM**(同上、field 2) |
| `_unsafe_get_bounds_unchecked` | `arr.@_storage` を直接 retaining read | **InlineLLVM**(read。borrow 化で `_storage` の retain を除く。unretained 版は作らない、§4) |
| `set`(make_unique, check, write, 旧要素 release) | `_storage` を unique 化(COW)+ `_check_range(i, _size)` + write(旧要素 release)を1 body で | **InlineLLVM**(現行 `InlineLLVMArraySetBody` を `Storage` 上へ re-target。in-place mutator ルール) |
| `_unsafe_set_bounds_uniqueness_unchecked_unreleased` | `Array::_unsafe_append_value_capacity_unchecked`(COW + 末尾へ `x` を n 個 + `_size` += n を 1 op で) | **InlineLLVM**(Array レベル、内部 COW、§4) |
| (新規)`Array::unsafe_set_bounds_unchecked(i, v, arr)` | COW + write(旧要素 release)。bounds check だけ省く `set` | **InlineLLVM**(`InlineLLVMArraySetBody` の `bounds_checked: false` 版。`unsafe_swap_bounds_unchecked` と対) |
| `reserve`(Fix-src: 新 storage 確保 + 要素ごとに retain してコピー) | `_unsafe_set_capacity_bounds_unchecked`(unique なら `realloc`、要素に触らない) | **InlineLLVM**(内部 COW、§4) |
| `append`(Fix-src: force_unique + 要素ごとに retain して書き込むループ) | `_unsafe_append_capacity_bounds_unchecked` の全範囲呼び出し(dst を COW、src が unique なら move) | **InlineLLVM**(内部 COW、§4) |
| `get_sub` / `String::get_sub`(Fix-src: `push_back(arr.@(i))` ループ) | 同 op の部分範囲呼び出し(融合コピー、unboxed 要素では memcpy) | **InlineLLVM**(内部 COW、§4) |
| `_unsafe_set_size` | `_unsafe_grow_size`(増加専用)へ改名: 内部 unique check(COW、optimizer 除去)+ value `_size` を伸ばす(新スロット未初期化)。減少は `_unsafe_truncate_bounds_unchecked` が release_range+shrink | **InlineLLVM**(in-place、内部 COW) |
| `_unsafe_empty_capacity_unchecked(cap)` | `#ArrayStorage` を内部 alloc し Array 値 `{ SubObject(#ArrayStorage), 0, cap }` を構築 | **InlineLLVM**(storage alloc は codegen 内部、§4) |
| `_unsafe_fill_size_unchecked(n, x)` | **削除** — `fill` は `_unsafe_empty_capacity_unchecked(n)` + `_unsafe_append_value_capacity_unchecked(x, n)` の 2 呼び出し(ループ無し) | 削除(§13.2) |
| `_pop_back_nonempty` | **削除** — `pop_back` は `_unsafe_truncate_bounds_unchecked(size-1)` を呼ぶ Fix-source(empty guard 付き)に。末尾1要素の release は `release_range([size-1, size))` に一般化される | 削除(`_unsafe_truncate_bounds_unchecked` に統合、§13.2) |
| (新規)`_unsafe_truncate_bounds_unchecked(n, arr)` | `pop_back`/`truncate` 共通コア: COW + `release_range([n,size))` + `size=n`(契約 `0<=n<=size`、size チェックなし) | **InlineLLVM**(in-place、内部 COW、畳める) |
| array literal `[..]` | `_storage` 確保, 埋め, `_size = len, _cap = len` | compiler lowering(既存の array-literal codegen を `Storage` allocate + initialize に向ける) |

`push_back` / `resize` / `append` / `fill` / `from_map` は **プリミティブではなく、他プリミティブを合成する Fix-source
ビルダー**(`reserve` は primitive、§4)。例えば
`push_back(e, arr)` は `if arr.@capacity < len+1 { arr.reserve(...) }`(register の容量チェック + 必要時のみ
`reserve`)-> `arr._unsafe_append_value_capacity_unchecked(e, 1)`。hot path(unique + 空きあり)では、容量チェックが
register compare、COW check が unique-check-elim で畳まれて slot への store と `size` field の +1 だけ —
**alloc も clone も無く register 中心**に落ちる。専用プリミティブ化しても速くならない(§13.2 の builder 群)。

**in-place write は COW を op 内に内包するのを既定とする。** `set` / `mod` / `act` / `swap` /
`_unsafe_grow_size` / `_unsafe_truncate_bounds_unchecked` の書き込みは、unique check(`if unique { in-place } else
{ clone }`)を Array レベルの InlineLLVM(現行 `set` の `force_unique`)に内包し、unique-check-elim が
provably-unique のとき check を畳んで in-place にする(`push_back`/`append` 等の Fix-source ビルダーはこれらを
合成するので、COW もこの InlineLLVM 内で畳まれる)。COW を内包した op は入力が何であれ健全(shared なら clone)で、
値としても純粋(同じ入力から同じ値を作る)なので、optimizer の fold も将来の CSE も安全な簡約になる。上書き +
旧要素 release は InlineLLVM(`set`/`swap`)の body の codegen が生ストレージへ直接行う(Fix レベルの storage write
プリミティブは無い、§4)。

**`result_prov = Fresh` の根拠は「返る配列が一意所有であること」で、その保証者は op 自身か optimizer の
どちらかである。** force-unique 分岐を持つ版は自分で保証する(shared なら clone 済み、unique ならそのまま)。
`assuming_unique()` が返す非 force-unique 版は、unique-check-elim が一意性を証明した場所にしか置かれないので
optimizer が保証する。よって **`result_prov` は `force_unique` フィールドで分岐させず、両版とも `Fresh` を返す**。
この宣言が builder の連鎖をつなぐ — `fill` / `push_back` / `append` / `from_map` が組んだ配列は、ループを抜けた
後も unique と分かり、以降の `set` などが自分の check を畳める。

一意性の保証を**呼び手の契約に委ねる** primitive を足す場合も同じく `Fresh` を宣言する。`_unsafe_` の名が
言うのは「一意性チェックが無い」であって「一意性が不要」ではなく、契約が満たされている限り結果は一意所有
だからである。契約違反はその op の書き込み自体が共有データを壊すので、`Fresh` を宣言することで破壊の条件が
増えることはない。

**uniqueness-check-less な mutate primitive は作らない。** 「呼び出し側が別 op で uniqueness を確立してから、
チェック無しで書く」形の primitive は、次の 2 つの理由で成立しない。

**理由 1: 隣接して書いても rc==1 は保証されない。** rc 挿入は「own な operand が、その文より後でも live なら
op の直前に retain を置く」規則で動く(全 opt レベル)。したがって

```fix
let (unique, arr) = arr._unsafe_is_storage_unique;
if unique {
    let (parr, e) = PunchedArray::_unsafe_punch_bounds_uniqueness_unchecked(0, arr);
    ...                 // ここで arr を後から参照すると、punch の直前に retain が入る
```

のように operand を op の後でも参照する形が 1 つ混じるだけで、uniqueness 判定の直後でも実行時 refcount は 2 になり、
unchecked op が共有配列を破壊する。この条件は型でも解析でも検査されず、破れても silent に壊れる。

**理由 2: 値として純粋でないので、将来の CSE / PRE(部分冗長性除去。一部の経路でだけ冗長な式にも計算を挿入して
束ねる、CSE の一般化)が壊す。** 例えば `mod` が uniqueness-unchecked な plug を使っていると:

```fix
let a1 = arr.mod(i, f);
let a2 = arr.mod(i, g);
// inline 後、同じ punch(i, arr) が 2 つ並ぶ。CSE がこれを 1 つに畳むと `parr` が 2 回使われ、
// rc 挿入が retain を入れ、unchecked な plug が同じ storage を 2 回書く(a1 と a2 が別配列にならない)。
```

COW を内包した op はこの制約を受けない。`a1`/`a2` が同じ値に畳まれても、以後の変更が COW するので観測結果は正しい。

**同じ理由で `_unsafe_force_unique` も削除する。** これは値としては恒等関数なのに所有権を移すので、
doc(`std_array_force_unique.md`)のとおり `f(x); f(x)` の inline 後に生じる 2 つの `force_unique(x)` を CSE が
1 つに畳むと、2 つ目の consumer が non-unique な配列を掴む。**この危険は checked にしても消えない**(恒等関数で
あること自体が原因)。`sort_by`/`reverse` は COW 内蔵 `swap` へ、builder は §4 の 4 primitive
へ寄せて置き換える。

**ループでの check 回数は増えない。** `_unsafe_force_unique` はループの外で 1 回チェックしていたが、COW 内蔵 op へ
寄せてもループ全体で 1 回のままになる。specialize が**再帰呼び出しを引数の uniqueness で keying する**ので、
ループが自分自身を 1 周 peel するため:

- 1 周目は canonical(全 `Dynamic`)クローンを通り、body の COW check をランタイムで 1 回行う。
- body の結果は `Fresh` なので、末尾の再帰呼び出しは **Unique keyed クローン**へ向かう。
- 2 周目以降は body も Unique keyed クローンになり、`set [unique]` = check 畳み済み・clone path 無し。
  よってステディ状態のループ本体はベクトル化も阻害されない。

入口の配列が shared でも unique でも、この形に落ちる(1 周目が入口の `Dyn` を吸収する)。

**これは Max の主張である。** specialize も borrow 化も `config.enable_borrow_optimization()`(= `fix_opt_level >= Max`)の
内側でしか走らないので、`-O basic` / `-O none` では op 内の COW check はそのまま残る。その水準での増減は:

- **減る**: `append`(要素ごとの unchecked write ループが bulk op 1 回になり、check が n 回から 1 回へ)、
  `sort_by` / `reverse`(ループ本体は今も COW 版 `swap` なので、`_unsafe_force_unique` の 1 回が消えるだけ)。
- **同数**: `push_back`(現在も呼び出しあたり `_unsafe_force_unique` の 1 回、移行後も op 内 COW の 1 回)。
- **増える**: `mod`(COW punch + no-COW plug の 1 回 -> COW punch + COW plug の 2 回)、`act`(`is_unique` +
  no-COW punch/plug の 1 回 -> `is_unique` + COW punch + COW plug の 3 回)。削除する uniqueness-unchecked
  primitive を直接呼んでいた外部コード(cp-library)は、移行先が COW 内蔵なので要素あたり 1 回増える。

Max ではこれらが畳まれる — plug は punch の `result_prov = Fresh` により、`act` の背中合わせの 2 チェックは
LLVM の GVN による(§3.3)。非 Max を速くすることは本再設計の目的ではないので、この増分は受け入れる。

primitive を 5 つに絞る話(§4)は builder が使う storage 操作の話で、per-element の in-place mutator は InlineLLVM に残す。

## 6. PunchedArray

`type PunchedArray a = unbox struct { _arr : Array a, _idx : I64 }` は既に `Array` を内包するので、新しい
`Array` レイアウトを継承する。変更点:

- `punch` / `plug`(`InlineLLVMArrayPunchBody` / `PunchedArrayPlugBody`)は現状 `gep_boxed(ARRAY_BUF_IDX)`
  で storage を読むが、内側 `Array` の storage Ptr(value field 0)と value の `_size` 経由へ移す。
- hole を飛ばす RC traversal(`build_traverse` の `is_punched_array` 特別扱い、`borrow.rs` の
  punched-array unit)は内側 array の `ARRAY_LEN_IDX`/`ARRAY_BUF_IDX` を読むが、内側 `Array` value の `_size` と
  storage Ptr へ移し、hole index を除く `[0.._size)` を release する。これは §3 と同じ要素数駆動の traversal に
  index を1つ飛ばすだけ。
- hole の所有は storage 粒度に留まる: `punch` は storage から要素を1つ move out して hole を残し `_size` は
  不変、`plug` は release せずに hole へ書き戻す。

## 7. FFI, `Boxed`, `String`

`Array` が boxed でなくなり、しかも `#ArrayStorage` は **`Boxed` instance を持たない**(§2.2)ので、FFI のポインタ系
generic(`_get_boxed_ptr`、`mutate_boxed`/`borrow_boxed`、`boxed_to_retained_ptr`/`boxed_from_retained_ptr`、
`with_retained`。すべて `[a : Boxed]`)は **Array にも storage にも直接は効かない**。Array のデータポインタ FFI は
**Array レベルの InlineLLVM ヘルパ**に集約し、その codegen が storage の refcount と buffer ポインタを直接扱う。

- **`Array a : Boxed` instance を削除する**(`stdlib.rs` のハードコード instance)。`#ArrayStorage` にも `Boxed`
  instance を与えない。**ユーザー可視の破壊的変更**: Array へ直接 `array.borrow_boxed(...)` /
  `array.boxed_to_retained_ptr` していたコードは型エラーになり、下の `Array::borrow_elements` 等へ書き換え。許容。
  **`Document.md` / `Document-ja.md` の retained-ptr 節も書き換える** — `boxed_to_retained_ptr` /
  `boxed_from_retained_ptr` を説明する唯一の実例が `Array I64`(`create_fix_array` / `get_fix_array_element`)なので、
  同 doc が別途示している「配列をフィールドに持つ自作 boxed struct」の形へ差し替える(これは Array を C へ
  opaque に渡したいユーザーへの推奨手順そのもの、下の retained ポインタの項)。
- **データポインタ**(生要素先頭): Array の FFI ヘルパ **`Array::borrow_elements`(+ `_io` / 可変版
  `mutate_elements` / `_io`)** で取る。**InlineLLVM にするのは同期版 2 つと `ios` を引数に取る
  `_mutate_elements_ios` の 3 つで、`_io` 版は Fix-source ラッパ**(既存の `borrow_boxed` / `_mutate_boxed_internal` /
  `_mutate_boxed_ios_internal` と `borrow_boxed_io` / `mutate_boxed_io` の対応をそのまま写す)。**`_io` 版を
  InlineLLVM にしてはならない** — `IO b` を返す op は「生ポインタを閉じ込めた IO アクションを構築して返す」だけで、
  それが走るのは op が返った後 = borrow 窓の外であり、arr は既に release され得る。既存 2 関数は Fix ソース側で
  これを避けている: `borrow_boxed_io` は内側 IO の `@runner` を **borrow callback の中で `ios` に適用**し、
  `mutate_boxed_io` は `ios` を取る InlineLLVM へ **`ios` を渡し込む**。
  なお **borrow 側に `_ios` 版が要らないのは、同期版のコールバックが純粋(`Ptr -> b`)だから** — `b` を
  `(IOState, b')` に取れば IOState をただのデータとして同期版に通せる。`mutate_boxed` はコールバックが既に
  `Ptr -> IO b` で、同期版が内部で IOState を捏造して走らせるため、本物の `ios` を渡す入口が別に要る。
  同期版のコールバックを純粋に保つことが、入口を 1 つ減らす条件。
  codegen が buffer 先頭ポインタ(現行
  `get_data_pointer_from_boxed_value` の `is_array` 分岐相当 = storage の buffer index)を callback へ渡す。
  `borrow_elements` は **arr を Borrow operand と宣言**するので、呼び出し側が callback 中も arr を生存させ、
  内部 retain なしで ptr が有効(§13.3-2。生 ptr が dangling する RC 問題は §8(2)(b))。返る番地は現状の要素領域と
  同じ。ポインタは callback 中のみ有効・`borrow_elements` は書き換え不可・`mutate_elements` は COW 後に可変。
  - **`String` の公開 API(`_get_c_str`/`borrow_c_str`)は不変** — 内部を `_data`(= `Array U8`)の `borrow_elements`
    経由へ差し替えるだけ。String FFI ユーザーは影響なし。std の byte-array FFI(to/from_bytes)も
    `borrow_elements`/`mutate_elements` へ内部変更。**`_get_c_str` は既に deprecated 済み**(dangling を返す危険関数、
    `borrow_c_str` へ誘導。redesign と独立に std.fix でマーク)。redesign 後の実装は
    **`s.@_data.borrow_elements(|ptr| ptr)`**(scoped borrow から ptr を漏らす = 従来どおり dangling し得るので deprecated
    が妥当)。`_get_ptr` 直接版は本再設計で削除する(§13.1(5))ので、raw ptr は borrow_elements から漏らす形になる。
  - 名前を `borrow_boxed` にしないのは、Array が Boxed でなく "boxed" が事実に反するため(`borrow_c_str` が中身を
    表す名前にしているのと同趣旨)。ユーザー自作の boxed 構造体への `borrow_boxed` は不変(まだ Boxed)。
- **retained ポインタ**(`boxed_to_retained_ptr` / `boxed_from_retained_ptr`): storage は Boxed 値でないうえ Array の
  size/cap は value にあって heap に無いので、**Array を retained pointer に往復させる generic 経路は無い**(現状は
  Array 自体が boxed で len/cap も heap にあるため往復で保存された)。**Array 版の retained-ptr ヘルパは用意しない**。
  完全な Array を C へ opaque に渡したいユーザーは、**Array を自作の boxed 型で包んで対処する**(包めば Boxed に
  なり `boxed_to_retained_ptr` が使え、size/cap も box 内フィールドとして保存される)。ユーザー側で対処する方針。
- **String**: `String = unbox struct { _data : Array U8 }` の C-interop chain(`_get_c_str`、`_unsafe_from_c_str`、
  `borrow_c_str`)は `_data` の `borrow_elements`/`mutate_elements` 経由へ差し替える(公開 sig 不変)。C 文字列
  ポインタ = `Array U8` の buffer データポインタ。数値の `to_bytes`/`from_bytes` も追随。
- `[a : Boxed]` の FFI primitive(`_get_boxed_ptr` / `_mutate_boxed_internal` / `_mutate_boxed_ios_internal`)の body に
  ある **Array 特別扱いは dead code になるので削除する**: `get_data_pointer_from_boxed_value` の `is_array` ->
  `ARRAY_BUF_IDX` 分岐と、mutate 側の `is_array` 分岐。Array が `Boxed` を外れる(§7 冒頭)ため、これらの generic に
  Array は到達しなくなる(`#ArrayStorage` も Boxed instance を持たないので同様)。**`assert!(is_box)` 自体は不変**
  (以後 Array を受け取らないだけ)。Array のデータポインタは `Array::borrow_elements` / `mutate_elements` の
  InlineLLVM が自前で計算する(value field 0 の storage ptr -> `#ArrayStorage` の `STORAGE_BUF_IDX` へ GEP)。

## 8. Debug info

`<array buffer>` debug 型と `<array size>` メンバ(`to_debug_type` / `ty_to_debug_struct_ty`)を書き直す:
`Array` の debug 型は 3 field の value struct(storage pointer、size i64、cap i64)になり、FAM/
`DEBUG_ARRAY_ASSUMED_LEN` の要素配列記述は `#ArrayStorage` の debug 型へ移る。

## 9. 実装の進め方

素朴にやると、この変更は ~40 の layout-constant 箇所と型/FFI/RC/debug 機構を一度に触る。表現の反転(step 3)は
一度に行い、それと独立に検証できる作業を前後に分ける。

0. **baseline を取る。** 実装に入る前の tip で **speedtest を全ケース実行**し、cachegrind の Ir とウォール
   クロックを記録する。step 5 の比較対象なので、コードを触る前に取ること。**先に `String::get_sub` を回す
   ケースを speedtest に追加する** — 範囲コピーの移行(§4)が効く場所なのに現行 38 ケースに該当が無く、
   追加してから baseline を取らないと比較対象を持てない。

1. **表現と独立な unsafe primitive の整理**(§3.3/§5)。現行の boxed レイアウトのまま実装・検証でき、
   ここで書いた op の body は反転後に storage 経由へ retarget するだけで作り直しにならない:
   - `InlineLLVMArrayPunchBody` に `result_prov` を実装(§3.3)。`InlineLLVMStructPunchBody` も同様。
   - §4 の builder primitive(`_unsafe_set_capacity_bounds_unchecked` / `_unsafe_append_value_capacity_unchecked` /
     `_unsafe_append_capacity_bounds_unchecked` / `_unsafe_truncate_bounds_unchecked`)と
     `Array::unsafe_set_bounds_unchecked` を追加。早期脱出と範囲クランプは Fix-source 側に置く(§4)。
   - builder(`append`/`from_map`/`reserve`/`push_back`/`resize`)、`sort_by`/`reverse`、`mod`/`act`、
     範囲コピー(`get_sub` / `String::get_sub` の実体と mergesort の merge)を COW 内蔵 op へ移行。
   - `_unsafe_force_unique` / `_unsafe_set_bounds_uniqueness_unchecked_unreleased` / punch・plug の
     uniqueness-unchecked 版を削除。cp-library を移行し、`test_external_project_cp_library` の pin を更新する。
   - 検証: `cargo test --release`、および `mod`/`act` と builder のマイクロベンチで uniqueness check が
     増えていないこと(`--emit-rc-ir all` の post dump の `[unique]` マーカーと LLVM IR)。
2. **`#ArrayStorage` 内部 tycon を導入**: tycon 登録 + レイアウト arm `{ ControlBlock, buffer }`(§2.2(3))+ 非
   traverse な要素 FAM variant + free-only RC + alloc / 要素 read / 未初期化 write / data-pointer の codegen ヘルパ。
   まだ `Array` からは未使用(dead-code 警告が「配線待ち」を示す)。`#ArrayStorage` はユーザーが名前を書けないので
   Fix レベルの直接 unit-test はできず、step 3 の Array op 経由で検証する(必要なら小さな InlineLLVM smoke)。
3. **`Array` の値レイアウトを unbox `{ SubObject(#ArrayStorage), size, cap }` に反転**(§2.2)。`ty_to_object_ty` の
   `Array` arm、`to_embedded_type`、`create_obj`、`size_of`、custom `build_traverse` arm(§3/§2.2(4))、不可分 unit
   述語(§3.2)、layout-constant 箇所すべて(`investigation-notes.md` §8)を一斉に更新。`Array a : Boxed` instance を
   削除、`@size`/`@capacity` を extractvalue 版へ、`String`/FFI chain(§7)と PunchedArray(§6)を書き換える。
   **`Array` を boxed のまま storage を内包する中間形は作らない** — その形でしか動かないコードを書いて捨てる
   ことになるため、反転は 1 コミットで行う。あわせて **型を辿る walker を掃除する**: `field_types` の呼び出し箇所を
   1 つずつ「要素型の取得(そのままでよい。`field_types(..)[0]` の形が大半)」と「値レイアウトの走査(`Array` で
   止める)」に分類する。後者は §3.2 の 3 箇所と `codegen::project_rc_unit`(path が常に `[]` なら到達しないので、
   §13.1 の assert で担保する)。
4. **Debug info**(§8)。
5. **検証**:
   - 全 opt レベルで `cargo test --release`。array/string/punched-array/FFI の test。
   - minilib + project_euler を memcheck 下で。要素 release を `_size` で駆動する点が最もリスクが高い変更 —
     shared/unique/COW/pop/resize/punch を跨いだ adversarial な memcheck。
   - **speedtest を全ケース実行し、step 0 の baseline と比較する。** 判定は (a) 劣化しているケースが無いこと、
     (b) write-loop 系で高速化が出ていること。個別の測定点: `push_back` は容量チェック `_size < _cap` が
     register になること、`write_by_range_fold` は bounds check が畳まれること、`array_mod` / `arrayrw` /
     `prime_table` は畳まれた check と vector op が LLVM IR に出ること。**step 0 で足した `String::get_sub` の
     ケースは、要素ごとのループが消えて `memcpy` になること**(`Array U8` なので retain が無く、範囲コピーが
     そのまま memcpy に落ちる、§4)。**read 系(`sum_by_loop_iter` /
     `sum_by_loop_arr`)は、太った `Option (ArrayIterator a, a)` の loop state が LLVM IR でスカラのまま
     (payload の store/load が出ていない)であること** — read ループは既に出荷済みの最適化で check が
     落ちているので、この再設計が持ち込むのは利得でなくこのリスクだけ(§10)。劣化ケースが出たら、
     §10 が挙げる戻り値の集約型・union payload・入れ子配列のメモリ増を疑う。

## 10. ABI と性能

- **利点**: `get_size`/`get_capacity` が register 読み出しになる -> write-loop の bounds-check elimination と
  vectorize、および `push_back` ループの容量チェックの hoist が自然に出てくる(write ケースの
  `--no-runtime-check` 天井に安全に届く)。
- **コスト**: `Array` が by-value 3 word になるので、`Array` を受け/返す関数はすべて pointer 1つでなく
  `{ptr, i64, i64}` を渡す — ABI が太る。retain/release/traverser の signature と closure ABI も波及する。
  bounds-check/容量チェックの利点が array-heavy コードでは支配的なはず。仮定せず測る(§9 step 5)。内訳:
  - **戻り値の集約型**: `String = unbox struct { _data : Array U8 }` が 1 word から 3 word になり、`String` を
    返す全関数と `Array` を返す `push_back` / `append` / `reserve` / `sort_by` が 24 バイトの集約を返す。unbox 値は
    `Object::to_embedded_type` が LLVM の集約型そのものを signature に出す形で、このコンパイラは `sret` /
    `byval` 属性を一切付けないので、lowering は LLVM 既定(x86-64 では隠しメモリ経由)になる。inline されれば
    SROA が消すが、されない呼び出しには残る。3 word 超の tuple や union は現状も同じ経路を通っているので、
    新しい機構ではなく適用範囲が広がる。
  - **union payload**: `ArrayIterator a = unbox struct { arr : Array a, idx : I64 }` が 2 word から 4 word になり、
    `advance` が返す `Option (ArrayIterator a, a)` の payload も同じだけ太る。出荷済みの read-loop 最適化は
    この loop state が SROA でスカラ化されることに依存するので、§9 step 5 で名指しして確認する。
  - **入れ子配列**: `Array (Array a)` は要素あたり 1 word から 3 word(§2.1)。
  - **非 Max の uniqueness check**: `mod` / `act` が呼び出しあたり 1 回 / 2 回増える(§5)。
- **リスク**: 要素の寿命(§3)が正しさに直結する部分 — count を誤ると leak か double-free。§9 step 5 の
  memcheck がその番人。

## 11. 方針

1. **決定 — storage は内部 tycon `#ArrayStorage`(`#DynamicObject` 流)。`#`-prefix でユーザーが名前を書けず、
   `Boxed` instance も持たない。Fix 露出は `Array` インターフェースだけ(§2.2/§4)。** 生ストレージ op(allocate /
   get / 未初期化 write / data-ptr)は `Array` の InlineLLVM body の codegen 内に閉じる。**`#ArrayStorage` を
   ユーザーが名前で書けないので「裸の storage 値」を Fix コードで作れず、ユーザーへ漏れようがない** — これが要素
   寿命 (b) の「ユーザーは `Array` しか持たず裸の storage を持たない」不変条件(§3)を型レベルで保証する
   (`#ArrayStorage` の destructor は生メモリを free するだけで要素 release は Array の `size` が駆動するので、
   Array より長生きした裸 storage は use-after-free になるが、そういう値を作れない)。FFI の公開面は §7 の scoped な
   Array borrow ヘルパ(`Array::borrow_elements` 系、コールバック中だけ有効な `Ptr` を渡す)だけにする。これにより
   plan §8(2)(a) 型の composable な隠れ穴 primitive(unretained element getter 等)が Fix レベルに存在しなくなる。
   `#ArrayStorage` にすることで、raw `Ptr` フィールドで要った placeholder-ty hack も消える(§2.2(4))。
2. **決定 — Fix レベルの uniqueness-check-less な mutate primitive を全廃する(§5)。** 対象は
   `_unsafe_force_unique`、`_unsafe_set_bounds_uniqueness_unchecked_unreleased`、punch/plug の
   uniqueness-unchecked 版。移行先は COW を内包した op(`set`/`swap`/`unsafe_set_bounds_unchecked`/
   `_unsafe_grow_size`/`_unsafe_truncate_bounds_unchecked`/force-unique 版 punch/plug)と、
   §4 の builder primitive(§4)。これで Array の mutate は全て「COW 内包 + 値として純粋」に揃い、
   将来の CSE / PRE や inline に対して安全になる。前提は §3.3(is_unique)と §3.1(unique-only な size 書き込み)が
   正しく効くこと、および punch への `result_prov` 実装(§3.3)。surviving unsafe RMW primitive の削除計画に接続する。

**要素の寿命(§3)の代替案**:
- **(a) count を `Storage` に持つ。** すると `get_size` が再び heap から読む — 本改修の意味が消える — ただし
  count を value に *複製* する場合を除く。複製すると size 変更のたび 2 箇所に書いて同期する必要が出る。却下。
- **(c) generic な custom-traversal ヘルパに `len` を渡す**(既存の hole path
  `build_release_mark_nonnull_boxed_with`)。採用案(Array value が release を駆動)の実装手段であって別モデル
  ではないので、独立の選択肢としては扱わない。

## 12. 本再設計が安全にする後続の最適化

本再設計の実装対象ではない。反転が済んだ後に、独立に検討する。

### 12.1 リテラルと空配列の immortal storage

現状、文字列リテラルは `make_string_lit` -> `InlineLLVMStringBuf` -> `make_byte_array_copy` に落ち、**評価のたびに
malloc + memcpy** する(バイト列自体は既に read-only global にあるのに、その複製を作っている)。
`Array::empty(0)`(`impl Array a : Zero` の実体)も評価のたびに malloc する。

やることは「storage を 1 つに固定し `refcnt_state = GLOBAL` にする」。実装の道は 2 つある:

- **(a) global 値へ持ち上げる**(Fix / RC IR レベル)。リテラルを top-level の global 値に括り出すだけで、
  既存機構だけで済む — 初期化時に 1 回 malloc し、`mark_global` が GLOBAL を書く。代償はアクセスごとの
  初期化フラグ判定(`--threaded` では `pthread_once` 呼び出し)。
- **(b) constant global を吐く**(codegen レベル)。`{ ControlBlock{1, GLOBAL}, [N x i8] }` を `.rodata` に置き、
  値を `{ const ptr, len, len }` にする。初期化も判定も無く完全な定数になる。`.rodata` に置くのは、書き込む
  経路が万一残っていたら silent corruption ではなく segfault で落ちるため。

健全性は両案共通で既存の 3 機構が担保する: `build_branch_by_is_unique` は GLOBAL を必ず shared 側へ流すので
書き込み系は COW する。retain は GLOBAL 分岐で何もしない。`mark_threaded` は `refcnt_state == LOCAL` のときだけ
書くので `--threaded` でも触らない。

**本再設計との関係**: 現行は `_size` / `_cap` が storage 側にあり、`_unsafe_set_size` などが共有配列でも COW せず
そこへ書く。immortal storage をその経路に当てるとリテラルが恒久的に壊れる。再設計後は `_size` / `_cap` が
value 側にあり、§5 の規則で storage への書き込みがすべて COW を経るので、この危険が消える。

### 12.2 要素バッファのアラインメント

`#ArrayStorage` の要素オフセットは **8**(ControlBlock `{i32, i8}` のサイズが 8 で、どの要素型でもそこに収まる)。
オブジェクト先頭は malloc が 16 バイト境界に置くので、要素バッファの番地は常に `16k + 8` になる。

揃えるなら、**`#ArrayStorage` のレイアウトに 8 バイトのパディングを入れて要素オフセットを 16 にする**。
`control_block_type` は触らない — 全 boxed オブジェクト(自作 boxed struct、boxed union、クロージャの
キャプチャ領域)が共有しており、vectorize の利益があるのは配列だけなので、そこに足すと配列以外まで太る。

**コストは配列 1 個あたり 8 バイト。** glibc malloc は usable size を 16 バイト粒度に丸めるので吸収されることも
あるが、mimalloc の小サイズビンは 8 バイト粒度なので素直に増える。短い `String`(= `Array U8`)を大量に持つ
プログラムで効いてくる。

**32 バイト境界は採らない。** オブジェクト先頭自体を 32 に揃える必要があり `aligned_alloc` 系が要るうえ、
`realloc` は過剰アラインメントを保存しないので §4 の「`reserve` は unique なら realloc」と両立しない。

**判断の材料**: x86-64 の AVX では unaligned なベクタ命令とアラインされた版の速度差は小さく、LLVM の
vectorizer もアラインメントを要求しない。**先に `arrayrw` / `write_by_range_fold` の LLVM IR を見て、
アラインを揃えるプロローグ(peeling)が実際に出ているかを確かめる。** 出ていなければ利得はゼロなので入れない。

## 13. 付録: 影響を受ける関数・InlineLLVM の全一覧(契約付き)

本再設計が **追加 / 変更 / 削除 / 改名** する `std.fix` / `builtin.rs` / `stdlib.rs` / `object.rs` の対象の完全一覧。
各項に契約(何をするか + `_unsafe_` の場合は caller が守るべき前提)を付す。末尾 §13.3 に2つの設計ギャップを記す。

### 13.1 InlineLLVM / builtin / codegen — 実装・フラグ・borrow 属性

各項目について **実装形態**(Fix-src の式 / InlineLLVM の疑似コード)、**force-unique 分岐**(unique-check-elim が
COW を畳むための「force unique しない」機構)、**borrow 化属性**(`borrows_operand` / `result_prov`)を示す。

**属性の語彙**(`src/ast/inline_llvm.rs::LLVMGen`、consume 判定は `rc_ir/borrow.rs`・`provenance.rs`・
`unique_elim.rs`):
- **force-unique 分岐**: op が `unique_check_operand() -> Some { container_index, path }` で「どの operand(slot)の
  どの boxed leaf の uniqueness で COW するか」を報告し、`assuming_unique()` が **その分岐を落とした(COW 無しの)版**
  を返す。unique-check-elim がその leaf を Unique と証明したとき op を `assuming_unique()` に差し替える。実体は body の
  `force_unique: bool` フィールド。**この分岐を持つ op だけが COW を畳める(= 求められている「force unique しない」
  フラグ)。**
  - **`container_index` は `free_vars_mut()` の宣言順であって、Fix の引数順ではない。** 現行 `set` は
    `free_vars_mut = [array, idx, value]` を返すので `container_index: 0` が arr を指す(Fix の型では arr は
    第 3 引数)。**新設する op は配列オペランドを `free_vars_mut` の先頭に宣言する**ことにし、Array 系の宣言を
    すべて `Some{0, []}` に揃える。
  - **`path` は RC unit ツリー上の位置。`Array` 値そのものに対する path は常に `[]`**(§3.2 で `Array` は
    不可分 unit なので、中へ降りる unit path は存在しない)。`plug` の `Some{1,[0]}` だけは別で、operand 1 が
    `PunchedArray`(unbox struct)であり、その field 0 が `Array` unit を指す。
  - **TRAP: 不正な path は silent に `Unique` へ解決され、COW が全経路で消える。** `Provenance::leaf_at` は
    未登録 path に空集合を返し、空集合は束の底なので `Unique` に解決され、`maybe_elide` が無条件に
    `assuming_unique()` へ差し替える。共有配列が in-place 破壊されるのに警告もパニックも出ない
    (`result_prov` の `Dyn` 固定 TRAP と同格の危険度)。**この silent fallback は実装時までに残っていたら潰す** —
    `leaf_at` 自体は「非 leaf path を探る」正当な用途(`borrow.rs::root_inner` が pure projection か調べる)で
    空集合を返す設計なのでそのままにし、**欠けていたら宣言が誤りである消費側**で落とす: `leaf_is_unique`
    (`maybe_elide` からのみ呼ばれる)で path が記録済み leaf であることを assert し、宣言を記録する
    `provenance.rs::interp_rhs` で `boxed_leaf_paths(arg_ty)` に含まれることを assert する(op 名を出せるので
    こちらが診断しやすい)。`debug_assert!` ではなく素の `assert!`(テストはリリースビルドで走るため)。
- **borrows_operand(i)**: operand i を borrow(consume しない)か。default は全 operand consume。`borrow.rs` は
  `borrows_operand(i)` か result_prov に `Arg(i, ·)` として現れる operand のみ非 consume とする。
- **result_prov**: 結果の各 boxed leaf の provenance — `Fresh`(新規 unique)/ `Arg(k, path)`(operand k の passthrough
  alias。`root()` が alias とみなし retain を省く)/ `Dyn`(保守的)。**`force_unique` で分岐させない**(§5)—
  非 force-unique 版の一意性は optimizer が保証する。
- **記法(§2.2 準拠)**: 以下の pseudocode で `arr.@_storage` は **`#ArrayStorage` への `SubObject`(value field 0)**
  を指す codegen 上の読み出しであり、Fix レベルの struct getter ではない(`Array` は primitive)。
  同様に `arr.@_size`/`@_cap` は value field 1/2 への extractvalue。`Array` 値は **1つの不可分 custom-RC unit**
  なので、borrow/provenance の属性は「`Array` unit」に対するもの(struct field leaf ではない)。
- redesign 共通: size/cap は value field 読み。COW helper `make_array_unique` は要素数を value `size` から読み、
  storage を unique 化する(内部名は実装時に確定)。flat boxed array での provenance `path:[]`/`Fresh` は、この
  `Array` custom unit を指すよう再表現する。

**(1) force-unique 分岐あり — COW を畳める(`unique_check_operand`/`assuming_unique` + `force_unique` field)**

- **`set`**(`InlineLLVMArraySetBody`、InlineLLVM)— 疑似: `if force_unique { arr = make_storage_unique(arr) }`;
  `if runtime_check() { check idx < _size }`; `write(arr.@_storage, idx, value, release_old=true)`; `ret arr`。
  force-unique: `unique_check_operand = Some{0, []}`(operand 0 = arr、`Array` unit は path `[]`)、`assuming_unique` で
  `force_unique=false`。bounds は `runtime_check()` gate(field でない)。borrows: なし(arr[0]・value[2] consume)。
  prov: `Fresh`(Storage leaf)。**上書き + 旧要素 release はここに内在**。
- **`swap` / `unsafe_swap_bounds_unchecked`**(`InlineLLVMArraySwapBody`、InlineLLVM)— 2スロットを noretain read して
  cross-write(release 無し)。フィールド **`force_unique: bool` + `bounds_checked: bool`**(bounds_checked は
  registration 固定・非 fold、swap=true / unsafe=false)。force-unique: `Some{0,[]}`。borrows: なし。prov: `Fresh`。
- **`punch`**(`InlineLLVMArrayPunchBody`、InlineLLVM)— `(PunchedArray a, a)` を返す。`if force_unique {
  make_storage_unique }`; `elem = noretain_read(idx)`(hole を残す、size 不変); `ret (PunchedArray{_arr:arr, _idx:idx},
  elem)`。force-unique: `Some{0,[]}`。borrows: なし。prov: **per-leaf を実装する**(`Provenance::build_shape`)—
  punched-array leaf(path `[0]`)= `Fresh`、moved-out 要素 leaf(path `[1]`)= `Dyn`。要素は retain せずに
  取り出しており他所から参照され得るので、そこを `Fresh` にすると後続の in-place 更新が共有要素を壊す。
  これが無いと `mod` の COW plug の operand が `Dyn` になり畳めない(§3.3)。**登録シンボルは COW 版だけにする**
  (`_unsafe_punch_bounds_uniqueness_unchecked` は削除)。`force_unique` フィールド自体は残す — `assuming_unique`
  が畳んだ版を作るのに使う。`InlineLLVMStructPunchBody` も同じ形。
- **`plug`**(`InlineLLVMPunchedArrayPlugBody`、InlineLLVM)— `PunchedArray{_arr,_idx}` を分解、`if force_unique {
  make_storage_unique_with_hole(_arr, Some(idx)) }`; `write(idx, elem, release_old=false)`; `ret arr`。
  force-unique: `Some{container_index:1, path:[0]}`(operand 1 = punched、その field 0 = `_arr` の `Array` unit。
  Array 値そのものへの path が `[]` なのとは別の話)。PunchedArray は
  Fix struct なので field 0 で `_arr` に届き、`_arr`(Array custom unit)の storage uniqueness を見る。borrows: なし
  (elem[0]・punched[1] consume)。prov: `Fresh`。**登録シンボルは COW 版だけにする**
  (`_unsafe_plug_bounds_uniqueness_unchecked` は削除)。`force_unique` フィールドは `assuming_unique` 用に残す。
- **`unsafe_is_unique`**(`InlineLLVMIsUniqueFunctionBody`、InlineLLVM)— `(Bool, a)`。`if !assume_unique &&
  obj.is_box { flag = build_branch_by_is_unique(obj) } else { flag = const true }`。フィールド `assume_unique: bool`。
  force-unique: `unique_check_operand = Some{0, []}` iff `!assume_unique`; `assuming_unique` が `assume_unique=true`
  (flag が const true に畳み、`if unique{}else{}` を back end が消す)。borrows: **なし(operand 0 を意図的に consume)**。
  prov: **`Dyn` 固定(TRAP)** — 第2成分は引数そのものだが passthrough にすると「後続 use が arg を shared に読ませる
  retain」を抑止し fold が誤って on になる。**Dyn を保つ**。redesign: `[a:Boxed]` 追加。Array には下記
  `_unsafe_is_storage_unique` を使う。
- **NEW `_unsafe_is_storage_unique`**(`Array::_unsafe_is_storage_unique : Array a -> (Bool, Array a)`、InlineLLVM)
  — Array value の field 0(storage Ptr)の refcount を **retain せずに**読み、`(Bool, Array a)` を返す。
  `unsafe_is_unique` と**同型の属性**にする: force-unique: `unique_check_operand = Some{0, []}` iff
  `!assume_unique`; `assuming_unique` が `assume_unique=true`(flag が const `true`)。borrows: **なし
  (operand 0 を意図的に consume)**。prov: **`Dyn` 固定(同じ TRAP)**。generic 版と同型にする理由は §3.3 —
  経路依存の精密化が入ったときにこの op もそのまま対象になる。
- **NEW `_unsafe_append_capacity_bounds_unchecked`**(InlineLLVM)— `(src, begin, end, dst) -> Array a`。dst を COW;
  `n = end - begin`; src の storage が unique かつ `begin == 0 && end == src._size` なら
  `memcpy(dst.storage + dst._size, src.storage, n * elem_size)` + `src._size = 0`(move、**要素の RC に触らない**)、
  それ以外は `load ptr -> retain -> store` の融合 1 パス; src を release; `dst._size += n`。
  契約: `0 <= begin <= end <= src._size`、`dst._size + n <= dst._cap`(呼び出し側)。**COW するのは dst だけ**なので
  force-unique: `Some{0,[]}`(`free_vars_mut` を `[dst, src, begin, end]` の順に宣言する)の 1 つで足りる(src 側は move/copy を選ぶ読み取りテスト)。borrows: なし。
  prov: `Fresh`。memcpy でよい理由(オーバーラップし得ない)、move が全範囲に限られる理由、`n == 0` の早期脱出と
  範囲クランプを Fix-source 側に置く理由は §4。
- **NEW `_unsafe_set_capacity_bounds_unchecked`**(InlineLLVM)— `(new_cap, arr) -> Array a`。unique なら
  `realloc(storage, size_of(new_cap))` -> value の field 0 を差し替え `_cap = new_cap`(**要素に触らない**)、
  shared なら新 storage を alloc して `[0,_size)` を retain 付きコピー(`clone_array_buf` に capacity 引数を
  足した形)+ 旧配列を release。契約: `new_cap >= _size`(呼び出し側)。force-unique: `Some{0,[]}`(`free_vars_mut` の先頭を arr にする)。
  borrows: なし。prov: `Fresh`。`realloc` が使えるのは storage が単一の malloc ブロックで、ControlBlock が
  `{refcnt, refcnt_state}` だけ(traverser を持たない)だから。`new_cap <= _cap` の早期脱出は Fix-source 側(§4)。
- **NEW `_unsafe_grow_size`**(`_unsafe_set_size` から改名、InlineLLVM)— 旧 body は `insert_field(LEN, n)` のみで
  COW 無し。**redesign で force-unique 分岐を新設**(`force_unique` field + `unique_check_operand=Some{0,[]}` +
  `assuming_unique`)— value `_size` を n に伸ばす前に Storage を COW。理由: `_size` を書くのは unique な `_storage` に
  だけ(§3.1)。畳めるので provably-unique では同性能。borrows: なし。prov: `Fresh`。
- **NEW `_unsafe_truncate_bounds_unchecked`**(InlineLLVM。`pop_back` と `truncate` の共通コア)— 契約 `0 <= n <= _size`(呼び出し側が
  保証、size チェックなし)。`if force_unique { make_storage_unique }`; `release_range(arr.@_storage, [n, _size))`;
  `ret arr{_size=n}`。force-unique 分岐あり(`Some{0,[]}`、畳める)。borrows: なし。prov: `Fresh`。**安全な公開版は
  Fix-source ラッパ**(§13.2): `truncate(n)` = `if n<0 { undefined }; if n>=_size { arr }; arr._unsafe_truncate_bounds_unchecked(n)`、
  `pop_back` = `if _size==0 { arr }; arr._unsafe_truncate_bounds_unchecked(_size-1)`。§13.3-1。
- **NEW `mutate_elements` / `_io`**(専用 InlineLLVM)— `if force_unique { make_storage_unique }`; `ptr =
  data_ptr(arr.@_storage)`; `r = act(ptr)`; `ret (arr, r)`。force-unique 分岐あり(`Some{0,[]}`)。§13.3-2。
- **NEW `_unsafe_append_value_capacity_unchecked`**(InlineLLVM)— `(x, n, arr)`。`if !unique { clone }`;
  `build_retain(x, n)`(増分 n の 1 回。n 回のインクリメントではない); `[_size, _size+n)` へ `x` を
  write(未初期化スロットなので旧値 release なし); `_size += n`; **`release(x)` を 1 回**(正味 n-1。n=0 でも正しい)。
  契約は `n >= 0` と `_size + n <= _cap`。増分の引数は `build_retain` 自体に足す(§4)。
  borrows: なし(全 operand consume)。force-unique: `Some{0,[]}`(`free_vars_mut` を `[arr, x, n]` の順に宣言する)。prov: `Fresh`(storage leaf)。
  旧 `_unsafe_set_bounds_uniqueness_unchecked_unreleased` と `array_unsafe_fill` の後継(§4)。
  `push_back` は n にリテラル 1 を渡す — Max で LLVM が定数伝播してループを消すことに依存するので、
  `push_back` / `pop_back` の speedtest ケースでそれが崩れたら気付けるようにする(§9 step 5)。
- **NEW `unsafe_set_bounds_unchecked`**(`InlineLLVMArraySetBody` の `bounds_checked: false` 版)— COW + write +
  旧要素 release。bounds check だけ省く。`unsafe_swap_bounds_unchecked` と対で、cp-library のような
  「範囲が自明な in-place 書き込みループ」の移行先(§11.2)。force-unique: `Some{0,[]}`。prov: `Fresh`。

**(2) COW 固定(畳めない)**

- 該当なし。旧 `_pop_back_nonempty`(無条件 COW・非 fold)は削除し、`pop_back` は上記 `_unsafe_truncate_bounds_unchecked(_size-1)` を
  呼ぶ Fix-source ラッパに置き換える(COW も畳めるようになる)。in-place COW op はすべて (1) の畳める force-unique
  分岐を持つ。これは §5 の規則の帰結で、**このグループは構造的に空になる** — in-place mutate は COW を op 内に持ち、
  その COW は必ず `unique_check_operand` で宣言されるので (1) に入る。ここに op を足したくなったら、それは規則違反のサイン。

**(3) uniqueness に関わらない(read-only / 新規確保 / 純 guard)**

- **`_unsafe_get_bounds_unchecked`**(InlineLLVM)— `arr = noretain(arr)`(borrow); `elem = retaining_read(
  arr.@_storage, idx)`。**borrows: operand 0 = borrow**。prov: `Dyn`(共有 container から retain 済み要素)。存続。
  boxed 要素では per-element の retain/release が残る(retain が op の内側にあり相殺の対象にならない、§4)。
  **unretained 版は作らない**(§4・plan §8(2)(a) の再導入回避)。
- **`_unsafe_empty_capacity_unchecked`**(InlineLLVM)— `#ArrayStorage` を内部 alloc し Array 値
  `{ SubObject(#ArrayStorage), 0, cap }` を構築。borrows: なし。prov: `Fresh`。storage alloc は codegen 内部(Fix 関数化しない)。
- **`_check_range` / `_check_size`**(InlineLLVM、存続不変)— 純 I64 guard(`runtime_check()` gate)。属性なし。
- **array literal**(`InlineLLVMArrayLitBody`、InlineLLVM 存続)— `Storage(len) 確保`; 各 elem を noretain write
  (release 無し); `Array{_storage,_size=len,_cap=len}`。borrows: 全 element consume。prov: `Fresh`。
- **`@size` / `@capacity`**: 現行の InlineLLVM body(heap load)を value field 1 / 2 への `extractvalue` に変える
  (手登録 builtin のまま、§2.2(5))。scalar なので prov 空・register 読み。storage Ptr(field 0)は codegen 内部で
  のみ読む。

**(4) codegen helper(Rust、非 LLVMGen)**

- **`make_array_unique` / `_with_hole`** -> **`make_storage_unique`**: `build_branch_by_is_unique(storage)`; shared 枝で
  `_cap` 分の Storage を確保し value `_size` 個を `clone_array_buf`(各要素 retain、hole 版は1スロット skip)して元を
  release。上記 force-unique 分岐すべてが経由。
- **`get_data_pointer_from_boxed_value` と mutate 側の `is_array` 分岐**: **削除**(§7)。`Array a : Boxed` を外すので
  これらの generic に Array は到達しない。`_get_boxed_ptr` は borrow(`borrows_operand(0)=true`)のまま boxed 型専用。
  Array のデータポインタは `borrow_elements` / `mutate_elements` の InlineLLVM が value field 0 の storage ptr から
  `STORAGE_BUF_IDX` へ GEP して自前で計算する。
- **Array value の field 参照・構築**: `Array` は primitive なので struct getter / `MakeStruct` は使わず、value の
  field 0/1/2 への `extractvalue`(read)/ `insertvalue`(rebuild)を codegen が直接出す。`@size`/`@capacity` は
  field 1/2 の extractvalue、storage Ptr は field 0(codegen 内部のみ)。`mutate_elements` 等の in-place rebuild は
  field 0 を新 storage Ptr で差し替える insertvalue。

**(5) 削除**(理由は前掲): `_unsafe_force_unique`、`_unsafe_set_bounds_uniqueness_unchecked_unreleased`(->
`Array::_unsafe_append_value_capacity_unchecked`)、punch/plug の uniqueness-unchecked 版、
`_unsafe_get_linear_bounds_unchecked_unretained`(両変種)、`_unsafe_fill_size_unchecked`(Rust 側は
`array_unsafe_fill`。`fill` は append-n の 1 回呼び出しへ)、`_pop_back_nonempty`(->
`_unsafe_truncate_bounds_unchecked` に統合)、`_get_ptr`。(`@size`/`@capacity` は削除で
なく body 変更 = heap load -> `extractvalue`、§2.2(5)。)

**(6) 型・登録の変更**(op ではないが必要): `Array` は `TyConVariant::Array` のまま、`ty_to_object_ty` の `Array` arm を
**unbox `{ SubObject(#ArrayStorage), size, cap }`** へ変更(index 定数 `ARRAY_STORAGE_IDX` / `ARRAY_SIZE_IDX` /
`ARRAY_CAP_IDX`)。**内部 tycon `#ArrayStorage` を新設**(`#DynamicObject` 流、tycon 登録 + レイアウト arm
`{ ControlBlock, buffer }` + 非 traverse な要素 FAM variant + `STORAGE_CTRL_IDX` / `STORAGE_BUF_IDX`、§2.2)。
hardcoded `Array a : Boxed` instance を **削除**、`Array` を不可分 unit 述語へ追加(§3.2)。詳細は §2.2/§7。

### 13.2 std.fix Fix 関数・trait instance(public シグネチャは特記以外すべて不変)

**追加(いずれも Array の InlineLLVM。storage は Boxed 値でないので codegen が storage を直接扱う):**

| 名前 | 契約 |
| --- | --- |
| `Array::borrow_elements : (Ptr -> b) -> Array a -> b` | 要素先頭 Ptr を callback に借用。**専用 InlineLLVM**: arr を Borrow operand と宣言し、buffer 先頭 ptr を `f` へ渡すだけ(内部 retain 不要 = 呼び出し側が f 中も arr を生存させる、clone なし、§13.3-2)。`array.borrow_boxed` の後継。ポインタは callback 中のみ有効・書き換え不可 |
| `Array::borrow_elements_io` | **Fix-source ラッパ**(`borrow_boxed_io` と同型: 内側 IO の `@runner` を borrow callback の中で `ios` に適用)。InlineLLVM にすると生 ptr が borrow 窓の外へ漏れる(§7) |
| `Array::mutate_elements` | Ptr 経由 in-place mutate。**専用 InlineLLVM**(`set` と同じくその場で COW -> data ptr -> act -> value rebuild、§13.3-2) |
| `Array::_mutate_elements_ios` | `ios` を引数に取る **専用 InlineLLVM**(`_mutate_boxed_ios_internal` と同型。COW 内蔵) |
| `Array::mutate_elements_io` | **Fix-source ラッパ**(`mutate_boxed_io` と同型: `_mutate_elements_ios` へ `ios` を渡し込む) |
| `Array::_unsafe_is_storage_unique : Array a -> (Bool, Array a)` | storage の refcount を **retain せずに**覗く。**専用 InlineLLVM**(属性は generic `unsafe_is_unique` と同型 = `unique_check_operand` あり・`result_prov` は `Dyn` 固定、§3.3) |
| Array 用 uniqueness assert(名前 TBD) | `_unsafe_is_storage_unique` ベース。`arr.assert_unique` の後継 |

**変更:**

- builder: `push_back` / `resize` の伸長 -> `_unsafe_append_value_capacity_unchecked`、`append` -> `_unsafe_append_capacity_bounds_unchecked`(全範囲)、`reserve` -> `_unsafe_set_capacity_bounds_unchecked`、`from_map` -> `empty` + push_back ループ。早期脱出(`reserve` の `new_cap <= _cap`、`append` の空 src)は Fix-source 側に置く(§4)
- 範囲コピー -> `_unsafe_append_capacity_bounds_unchecked`(部分範囲): `_get_sub_size_with_length_and_additional_capacity`(`get_sub` / `String::get_sub` の実体。範囲クランプは Fix-source 側に残す)、`_mergesort_range_using_buffer` の片側が尽きたあとの流し込み 2 枝
- `reserve`: Fix-source から **InlineLLVM primitive** へ(unique なら storage を `realloc`、shared なら alloc + clone、§4)
- `fill`: `_check_size` + `_unsafe_empty_capacity_unchecked(n)` + `_unsafe_append_value_capacity_unchecked(x, n)` の Fix-source に(ループ無し)。`_unsafe_fill_size_unchecked` は削除
- `mod`/`act`(punch/plug を COW 版へ、`unsafe_is_unique` -> `_unsafe_is_storage_unique`、act の分岐構造は維持、§3.3): `mod`, `_unsafe_act_bounds_unchecked_identity`, `_unsafe_act_bounds_unchecked_tuple2`, `_unsafe_act_bounds_unchecked`
- `sort_by`, `reverse`: `_unsafe_force_unique` 撤去(COW `swap` が make-unique 済み)
- `pop_back`: `if size==0 { arr }; arr._unsafe_truncate_bounds_unchecked(size-1)` の Fix-source に(`_pop_back_nonempty` は削除)
- `truncate`: pop_back ループを廃止し、`if n<0 { undefined }; if n>=size { arr }; arr._unsafe_truncate_bounds_unchecked(n)` の Fix-source に(§13.3-1)。`_unsafe_from_c_str` の切り詰めを `truncate` へ移す(`String::from_bytes` は既に `truncate` を呼ぶ)
- `@size`/`@capacity`: 手登録 builtin InlineLLVM のまま、body を heap load から value への `extractvalue` に。`get_size`/`get_capacity` alias は不変
- `_unsafe_empty_capacity_unchecked`: InlineLLVM のまま body を `#ArrayStorage` の alloc + value 構築へ(`empty` は name/contract 不変)
- String C-interop(`_data` の `borrow_elements` / `mutate_elements` 経由へ、sig 不変): `_get_c_str`, `borrow_c_str`, `_unsafe_from_c_str`, `unsafe_from_c_str_ptr`(`_io`)
- IO byte 関数(`mutate/borrow_boxed` -> `_elements`): `_read_line_inner`, `read_n_bytes`, `write_bytes`
- `assert_unique`: **`[a:Boxed]` 制約追加**(`arr.assert_unique` は compile error 化 -> Array 版へ誘導)
- 数値 trait instance(`mutate/borrow_boxed` -> `_elements`、`_unsafe_set_size` -> `_unsafe_grow_size`): `ToBytes`/`FromBytes`/`ToString` の U8..F64 一式(+ `to_string_exp`/`_precision`)

**削除:**

- Rust 登録プリミティブ(§13.1(5) と同一の一覧): `_unsafe_force_unique`, `_unsafe_set_bounds_uniqueness_unchecked_unreleased`, punch/plug の uniqueness-unchecked 版, `_unsafe_get_linear_bounds_unchecked_unretained`(両変種), `_unsafe_fill_size_unchecked`, `_pop_back_nonempty`(-> `_unsafe_truncate_bounds_unchecked` に統合), `_get_ptr`
- trait instance: **`impl Array a : Boxed` を削除**(`#ArrayStorage` にも Boxed instance を与えない)。**user-visible break**: `array.borrow_boxed` / `array.boxed_to_retained_ptr` が型エラー -> `borrow_elements` か自作 boxed 型でラップ。`Document.md` / `Document-ja.md` の retained-ptr 節の例も差し替える(§7)
- `unsafe_is_unique` の unbox 枝(const-true)が `[a:Boxed]` 追加で dead

**改名(呼び出し側更新):**

- `_unsafe_set_size` -> `_unsafe_grow_size`。残る呼び出しは **FFI 出力バッファの経路だけ**: `read_n_bytes`/`unsafe_from_c_str_ptr`(`_io`)/数値 `to_bytes` 一式(builder 群は §4 の primitive へ移る)。**真の shrink を行う `_unsafe_from_c_str` の切り詰め経路は効率化した `truncate`(safe shrink、§13.3-1)を使う** — これで「共有配列の `_size` をその場で書き換える」経路が無くなる

**新規 Rust 登録プリミティブ:** builder 系(いずれも §4)— `_unsafe_truncate_bounds_unchecked`(`pop_back`/`truncate`)、`_unsafe_append_value_capacity_unchecked`(`push_back`/`fill`/`resize`)、`_unsafe_append_capacity_bounds_unchecked`(`append`/`get_sub`/merge)、`_unsafe_set_capacity_bounds_unchecked`(`reserve`、unique なら realloc)、`unsafe_set_bounds_unchecked`(bounds check だけ省いた `set`)。加えて FFI ヘルパ `borrow_elements` / `mutate_elements` / `_mutate_elements_ios`(§7、上の追加表)と、Array の uniqueness を覗く `_unsafe_is_storage_unique`(§3.3)。

**不変(変更された callee を通すだけ):** `@`, `get_first`/`get_last`, `is_empty`, `find_by`, `get_sub`, `dedup`, `empty`,
`act`, `from_iter`/`to_iter`, sort 内部一式(`_introsort`/`_heap*`/`_insertion*`/`_mergesort*`/`sort`/`sort_stable*`。
merge の流し込み 2 枝を書き換える `_mergesort_range_using_buffer` は上の変更側)、
全 Array trait impl(`Zero`/`Add`/`Eq`/`LessThan`/`Functor`/`Monad`/`ToString`/`Indexable`)、FFI 定義
(`mutate_boxed`/`borrow_boxed`/retained-ptr — Array を受けなくなるだけ)、`Destructor::mutate_unique_io`(box なので `[a:Boxed]` OK)、
String の大半、PunchedArray 型(新レイアウトを継承、punch/plug/traverse の Rust body だけ retarget)。

### 13.3 要検討(設計ギャップ)

1. **shrink 経路 — 危険トランケート `_unsafe_truncate_bounds_unchecked` を1本作り、`pop_back`/`truncate` がそれを共有する(決定)。**
   `_unsafe_grow_size`(前提 `n >= _size`)は `_unsafe_from_c_str`(null terminator の後にバイトが続くと真の shrink)や
   数値 `to_string` の over-allocate 経路を表せない。shrink の public API は既に `truncate`
   があるので **新しい `shrink_size` は追加しない**。コアとして **`_unsafe_truncate_bounds_unchecked(n, arr)`**(size チェックなし、契約
   `0<=n<=_size`)を InlineLLVM で作る — `if force_unique { make_storage_unique }`; `release_range([n, _size))`;
   `_size = n`(§13.1(1))。**安全な公開版は size チェック後にこれを呼ぶ Fix-source**: `truncate(n)` = `if n<0
   { undefined }; if n>=_size { arr }; arr._unsafe_truncate_bounds_unchecked(n)`、`pop_back` = `if _size==0 { arr };
   arr._unsafe_truncate_bounds_unchecked(_size-1)`。これで現状の pop_back ループ(O(size-n))が **1 回の range-release** になり、
   `pop_back` の COW も畳めるようになる(旧 `_pop_back_nonempty` は無条件 COW・非 fold だった)。**最適化で消える**:
   unique-check-elim が provably-unique で COW を畳み、unboxed 要素(`Array U8` 等)では要素 release が no-op になる —
   よって `String::from_bytes`(unique な `Array U8`)では `truncate` が実質「`_size` を下げるだけ」に落ち、旧
   `_unsafe_set_size` の shrink と**同性能かつ安全**。切り詰め経路は `truncate` を使う(`String::from_bytes` の
   `truncate(null_idx + 1)` がその形)。boxed 要素の配列でも安全に使える。
2. **`borrow_elements` / `mutate_elements` / `_mutate_elements_ios` は Array の専用 InlineLLVM(決定。`_io` 版は
   Fix-source ラッパ、§7)。** storage は Boxed 値でない
   (§2.2)ので、両者とも codegen が storage を直接扱う:
   - **`borrow_elements`(同期版)**: **arr を Borrow operand と宣言**(base-level `borrows_operand=true`、
     `_unsafe_get_bounds_unchecked` と同型)。呼び出し側が call 全体(callback `f` を含む)の間 arr を生存させるので、
     codegen は **buffer 先頭 ptr を `f` へ渡すだけ(内部 retain 不要)**。**clone しない**。生 ptr は unboxed で RC が
     「arr の使用」と見なさないため、arr を Own にすると RC 挿入器が ptr 抽出時点で arr を release -> f 中に storage が
     free -> dangling(§8(2)(b))。Borrow 宣言でこの dangling を全 opt レベルで防ぐ(汎用 `with_retained` が常に retain
     するのは引数を Borrow 宣言できない汎用 op だから。専用 op はできるので retain を出さない)。**mutation 防止の
     retain も不要**: `f` はクロージャなので同じ配列をキャプチャし得るが、そのキャプチャ自体が retain を出して
     rc >= 2 になるため、`f` の中の mutate は COW して別 storage に書く。生 ptr への書き込みはそもそも retain では
     止まらない。
   - **`mutate_elements` / `_mutate_elements_ios`**: `set` と同じく storage をその場で `make_array_unique`(実際に shared のときだけ
     COW)-> data ポインタ取得 -> act -> value を新 storage で rebuild。
