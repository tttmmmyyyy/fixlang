# Array/Storage 表現の再設計 — 設計

ステータス: 設計のみ、未実装。`investigation-notes.md`(コード実地調査の生データ)を土台とする。
目的は `Array::@size` を register 読み出しにして、back end が write loop からそれを巻き上げ
(hoist)、要素ごとの bounds check を畳み、vectorize できるようにすること — bounds-check
elimination の write-loop 側(`../2026-07-18-bounds-check-elim/`)。read-loop 側(iterator の
終了条件変更 + RC-IR simplifier)は既に出荷済み。

## 0. 概要

**やること**: `Array` を **primitive tycon のまま**、その値レイアウトを boxed から **unbox 3-word
`{ storage:Ptr, size:I64, cap:I64 }`** に変える。要素の heap 領域(refcount + 生要素)は **Fix の型ではなく
codegen のオブジェクトレイアウト**として持つ(`storage` はそこへの生ポインタ)。狙いは `@size`/`@capacity` が
**register 読み出し**になること — write loop の bounds check と `push_back` の容量チェックが hoist/畳まれて
vectorize する(write-loop BCE)。read-loop BCE は既に別途出荷済み。

**確定した設計判断**(詳細は各節):
- **要素の heap 領域(storage)は Fix の型にしない** — codegen 専用の ObjectType としてのみ存在し、tycon も
  `Boxed` instance も持たない。ユーザーにも std にも「storage という型」は現れず、Fix インターフェースは `Array`
  のもの(public/private・safe/unsafe)だけ(§2.2/§4/§11.2)
- `Array` は primitive を維持。値レイアウト unbox `{ Ptr, size, cap }`、storage object は `{ ControlBlock, buffer }`
  (§2.2)
- size/cap は value(3 word、C++ `std::vector` 流)、refcount だけ storage(§2.1)
- 要素解放は Array の custom traverser が value の `size` で駆動、storage は free のみ(§3, §3.1)
- RC-unit は PunchedArray と同じ「不可分 unit」扱い、名前付き述語で寄せる(§3.2)
- Array の uniqueness は専用 `Array::_unsafe_is_storage_unique`(storage の refcount を retain せず覗く)。generic
  `unsafe_is_unique` は存続(§3.3)
- bulk op(fill 等)は Fix-source(最適化器がベクトル化して InlineLLVM と同等になる、§4)
- FFI ポインタ系は Array の InlineLLVM ヘルパ経由。retained-ptr は size/cap を運べない(§7)
- 事前手動 unique を要する unsafe primitive を safe 版へ寄せて縮小(§3.3/§11.3)

**進め方**: tests green を保つ5段階の移行(§9)。実装は設計確定後。

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
`array_mod` は命令数が約 3 分の 1 に落ち `arrayrw` が vectorize する = check が vectorize の上限)。size が
heap にあることが root cause なので、直し方は解析パスの追加ではなく表現そのものの変更である。

## 2. 表現

`Array` は今も **primitive tycon**(`TyConVariant::Array`)のままとし、その **値レイアウトだけ**を「長さと容量を
value に持つ unbox 3-word」に変える。要素を格納する heap 領域(refcount + 生要素)は、**Fix の型ではなく
codegen のオブジェクトレイアウト**として持つ — `Array` の値がその領域への生ポインタを1本持つ。

```
Array a  (primitive tycon)   // 値レイアウト: unbox { storage : Ptr, size : I64, cap : I64 }
   storage --> heap object { ControlBlock, elem[FAM] }   // codegen 専用 ObjectType。Fix の型ではない
```

storage の指す heap object は refcount と生要素だけを持ち、メタ情報(size/cap)は value 側にある。この heap
object は **どの Fix 型にも対応しない**: `ty_to_object_ty` の入口以外で ObjectType を消費する経路
(`to_struct_type` / `size_of` / `build_traverse` / debug)は tycon を引かず ObjectType を直接扱うので、`Array` の
codegen が `elem_ty` から ad-hoc に組んだ ObjectType `{ ControlBlock, buffer }` で足りる(debug の array arm が
既に tycon 無しの ad-hoc ObjectType を作る前例がある)。よって `Storage` を Fix の型として導入する必要はなく、
ユーザーにも std にも「storage という型」は現れない(§2.2、§11.2)。

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

代償: `Array` が by-value 3 word になる — 配列を渡す関数の ABI がやや太り、`Array (Array a)` のような
入れ子では要素あたり 1 word 増える(メタ情報のメモリ)。C++ vector も 3 word なので許容範囲とする。

### 2.2 `Array` プリミティブの定義(何を primitive にするか)

`Storage` を型にしない以上、「`Array` プリミティブが何であるか」を codegen の言葉で決めきる必要がある。以下が
その定義。**要は、現行の boxed-primitive `Array` の codegen(alloc / get / set / traverse / clone / size_of / debug)を
残しつつ、(i) object を size/cap 抜きに縮め、(ii) size/cap を value 側の unbox 3-word に出し、(iii) その value を
1つの不可分 custom-RC unit にする**、という変更に集約される。

**(1) tycon**: `Array` は `TyConVariant::Array` を維持。変わるのは `ty_to_object_ty` の `Array` arm が返す
ObjectType だけ。

**(2) 値レイアウト(unbox ObjectType)**: `is_unbox = true`、`field_types = [ Ptr, I64, I64 ]`。

| index | 意味 | 定数(改称) |
| --- | --- | --- |
| 0 | storage への生ポインタ | `ARRAY_STORAGE_IDX` |
| 1 | size(構築済み要素数) | `ARRAY_SIZE_IDX`(旧 `ARRAY_LEN_IDX`) |
| 2 | capacity | `ARRAY_CAP_IDX` |

field 0 は生 `Ptr` = generic RC の対象外。refcount と要素寿命は (4) の custom unit が扱う。

**(3) storage object レイアウト(codegen 専用 ObjectType、boxed)**: `is_unbox = false`、
`field_types = [ ControlBlock, <element buffer> ]`。

| index | 意味 | 定数 |
| --- | --- | --- |
| 0 | ControlBlock(refcount) | `STORAGE_CTRL_IDX` |
| 1 | 生要素の FAM | `STORAGE_BUF_IDX` |

size/cap を持たない。この ObjectType は `elem_ty` から組む codegen ヘルパ `array_storage_object_ty(elem_ty)` で
得る(`name` は debug 用の任意文字列で、tycon 参照は無い)。要素バッファは現行の `ObjectFieldType::Array(elem)`
が担う FAM 機構を再利用する。現行の `Array` variant は「capacity i64 + FAM」を兼ねる(`to_basic_type` が i64 を
返し `to_struct_type` が FAM を append)ので、これを **「ControlBlock/header を前提としない純 FAM」に整理し直す**
(capacity i64 を落とし、名前も buffer を表す変種へ改める)。storage は `{ ControlBlock, buffer }` ちょうど 2 field
になる。整理には `to_basic_type` / `to_struct_type` / `size_of` / traverse / debug の各 match arm を触るが、
redesign 後この variant の利用先は storage object だけなので迷いなく行える。

**(4) RC = custom 不可分 unit**: field 0 が生 `Ptr` なので、generic RC はこの値を「全 unboxed = RC 不要」と見なし
**何も出さない**(`build_retain` / `build_traverse` の `Ptr`/`I64` arm が no-op)。放置すると leak/二重解放。so
`Array` を不可分 unit 述語(`rc_units_go` / `clamp_unit` / `boxed_leaf_paths`)へ加え、`build_retain` /
`build_traverse` に `is_array()` の custom short-circuit を足す(既存 `is_punched_array` short-circuit と同型、§3.2)。
custom の中身:
- **retain**: value field 0(storage Ptr)の ControlBlock refcount を +1。
- **release / mark**: value field 1(size)と field 0(storage Ptr)を読み、`[0..size)` の要素を解放してから storage
  を free(unique 時)/ refcount -1(shared 時)。既存 `build_release_mark_nonnull_boxed_with` に「buffer を size 個
  traverse する closure」を渡す形(現 `Array` / `PunchedArray` と同じ)。
- **clone(COW)**: 新 storage を alloc し `[0..size)` を retain コピー、value field 0 を差し替え。

低レベル refcount ヘルパ(`retain_nonnull_boxed` / `build_release_boxed_with` / `build_branch_by_is_unique`)は
**ポインタだけを見る**ので storage Ptr にそのまま効く。呼ぶために storage Ptr を placeholder ty の `Object` で
包む小さな hack を使う(`.value` しか読まれないので安全。ty を dispatch する経路〔特に `Destructor` 判定〕へ
渡さないことだけ守る)。

**(5) アクセサ**: `Array` は struct ではないので `@field` の自動 getter は無い。`@size` / `@capacity` は現状と同じ
**手登録の builtin InlineLLVM**で、heap load でなく value への `extractvalue`(index 1 / 2)にする。storage Ptr
(index 0)を読むのは codegen 内部だけで、Fix レベルの op にはしない。

**(6) primitive op**(すべて上のレイアウトに対して codegen。契約の詳細は §4 / §13):
- alloc `_unsafe_empty_capacity_unchecked(cap)`: `array_storage_object_ty` を `size_of(cap)` で malloc、
  ControlBlock を refcount 1 に初期化、value `{ ptr, 0, cap }` を構築。
- `_unsafe_get_bounds_unchecked(i, arr)`: storage buffer の i 番目を load して retain(borrow arr)。
- `_unsafe_initialize(i, v, arr)`: buffer の i 番目へ store(release / COW なし、線形)。
- `set` / `swap` / `mod`(punch/plug)/ `act` / `_unsafe_pop_back_nonempty` / `mutate_elements`: COW 内蔵の in-place mutator。
- `_unsafe_grow_size(n, arr)`: COW してから value field 1(size)を n に。
- array literal: alloc + fill + value 構築(既存 literal codegen を新レイアウトへ)。
- FFI `borrow_elements` / `mutate_elements`: storage Ptr を retain / borrow して buffer 先頭ポインタを渡す(§7。
  storage は Boxed 値でないので、`with_retained` 相当を codegen で直接実装する)。

## 3. 要素の寿命 — 中核をなす判断

現状、`Array` の destructor は「何要素を release すべきか」を `len` から知る。`len` は storage と同じ heap
object にある(`build_traverse` の `Array` arm: `size = extract_field(ARRAY_LEN_IDX)`、続けて
`release_or_mark_array_buf(size, buf, ..)`)。`_size` が value に移ると、`Storage` 単体では live 要素数を
知れない。

**採用: `Array` value が要素 release を駆動する。** `Array` の release/mark/clone を custom traverser にし、
その各点で手元にある value の `_size` を使って `_storage` の生要素領域を歩く(`Storage` は untyped な生メモリとして
扱う)。`_size` を value だけに保てて(再設計の眼目)、しかも *現行* `Array` destructor が既に回しているロジック
そのもの — 変わるのは `size` の出所が heap load から value field になる点だけ。(検討して退けた代替案は §12。)
具体的には:

- **retain** `Array` = `_storage` の control-block refcount を +1(shallow、変更なし。COW で要素は共有のまま)。
  これは通常の boxed-field 伝播なので、retain については「boxed field を1つ持つただの unbox struct」で正しい。
- **release** `Array` = `_storage` が unique なら `for i in 0.._size { release(buf[i]) }` して `_storage` を free、
  そうでなければ `_storage` の refcount を -1。`_size` は value から来る。`Array` は現行 `Array`・`PunchedArray`
  と同じく custom な `build_traverse` arm を保ち、要素数駆動の traversal を value 読みに移すだけ。
- **clone-if-shared**(refcount >= 2 での mutate 時の COW)は `_size` 個の要素を新しい `Storage` へ複製し、
  各要素を retain する。clone 側も `_size` を手元に持つ。
- storage object の boxed destructor は **生メモリを free するだけ** — 要素 release は決してしない。所有側の
  `Array` が `_size` で駆動して既に release 済みだから。storage は codegen 専用 object(Fix の型でない、§2.2)で、
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
  alloc/reserve 後に size を伸ばす)。**減少は `_pop_back_nonempty` が担い、要素を release してから len を
  直接書く**(`_unsafe_set_size` を使わない)。**任意の safe な shrink は既存 `truncate`(効率化して COW + 切り詰め
  要素の release を1 op で内包、§13.3-1)が担う** — `String::from_bytes` 等の `Array U8` shrink 経路はこれを使う
  (unique-check-elim + unboxed release の no-op 化で、その経路では「`_size` を下げるだけ」に畳まれる)。
- そこで **増加専用の `_unsafe_grow_size`(「未初期化スロットへ size を伸ばす」、`n >= _size`)に置き換え、
  内部で unique check(= COW、shared なら clone)する**(名前を grow-only に合わせる — 現行 `_unsafe_set_size`
  は増加しかしない)。すると「呼び出し側が事前に unique を保証する」footgun が消え(op 自身が unique な `_storage`
  にしか size を書かない)、その内部 check は unique-check-elim が provably-unique で畳んで同性能にする
  (§11.3 の方向)。呼び出し側の `_unsafe_force_unique` は畳み込める。残る unsafe 契約は「新スロット
  `[old_size..n)` は未初期化(呼び出し側が埋める)」「`n <= _cap`」のみ。

### 3.2 RC-unit 機構との整合(PunchedArray と同じ特別扱い)

RC 挿入は値を **RC unit(boxed leaf)単位**に分解して retain/release を置く(`borrow.rs` の `rc_units_go` /
`clamp_unit`)。ただし `is_box` / `is_union` / `is_punched_array` は「1つの不可分 unit」として扱い、中へ
descend しない — その unit の retain/release は値全体の(custom)traverser 経由になる。現行の `Array` は
`is_box` なので自然にこの1 unit で、release すると Array の custom destructor が走る。

再設計後の `Array` は **unbox** なので、何もしないと generic な「field へ descend」枝に落ち、`_storage`(Storage)を
boxed leaf として**単独で RC** してしまう -> Storage の free-only destructor が走り要素を leak する。これが §3 の
coupling の機構的な正体。

**解決 = 新 `Array` を上記の不可分 unit 境界に加える**(`is_box || is_union || is_punched_array || is_array()`
相当)。追加先は `rc_units_go`、`clamp_unit`、`codegen::project_rc_unit`(全体 `{_storage,_size,_cap}` を projection
して custom traverser が `_size` を読めるようにする)、`provenance::build_shape`。こうすると `Array` は path `[]`
の1 unit になり、retain/release/mark がすべて Array の custom traverser 経由(value の `_size` 駆動)になって、
`_storage` が単独で RC されることはない。**これは `PunchedArray` が既に取っている扱いそのもの**で、
unique-check-elim / borrow / provenance / codegen は「custom traverser 型を1 unit として扱う」機構を既に持つ。

uniqueness(`set` の make_unique)は「`Array` unit = その `_storage` の refcount が unique か」で判定でき、provenance が
追う `_storage` leaf を `clamp_unit` が `Array` unit に丸めて突き合わせる(現行の union/is_box と同じ経路)。よって
per-unit の retain/release とも uniqueness 判定とも噛み合う。`PunchedArray` 自身は、custom traverser が読む値が
「内側 array の heap `len`」から「内側 `Array` の value `_size`」に変わるだけで、依然1 unit・hole skip のまま。

なお「不可分 unit 境界」の判定は現状 `is_box`/`is_union`/`is_punched_array` の disjunction が各パスに散在している
(しかも `clamp_unit` は `is_punched_array` を含まないなど不揃い)。`is_array()` を各所へ足して回る shotgun surgery
を避け、**「custom traverser を持つ不可分 RC unit か」を表す名前付き述語を1つ導入して既存の判定を寄せる**方針とする
(実装時に `clamp_unit` の不揃いが本質か latent bug かを見極めてから統一する)。この述語統一は redesign と独立した
cleanup であり、redesign 側は統一後の述語に `Array`(unbox)を1行足す。

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
  その場で refcount を読めるので確実(そもそも storage は型でないので値として取り出す API も無い、§2.2)。
- unique-check-elim の static fold は Array 専用版に適用(`_storage` が provably-unique なら const-`true` に畳む)。
  ランタイム版(`InlineLLVMIsUniqueFunctionBody` 相当を Array 用に)と fold の両方を用意する。
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

**削除したいのは Fix レベルの「呼び出し側が事前に手動 unique を保証する」primitive**:
`_unsafe_set_bounds_uniqueness_unchecked_unreleased`(from_map/push_back/reserve/append/resize)、**その前に置く
`_unsafe_force_unique`**(上記 + sort_by/reverse — いずれも in-place mutation の前処理)、**punch/plug の
uniqueness-unchecked 版 `_unsafe_punch/plug_bounds_uniqueness_unchecked`**(mod/act)。これらは (1) の COW check を
**skip する**ために存在した。**特に `_unsafe_force_unique` は redundant なだけでなく危険**: doc
(`std_array_force_unique.md`)のとおり将来 CSE(共通部分式除去)が入ると壊れる — `f(x); f(x)` の inline 後に生じる
2つの `x._unsafe_force_unique` を CSE が1つに纏めると、2つ目の consumer が non-unique な配列を掴んで破壊する
(inline/CSE の抑止に依存する)。unique-check-elim が safe 版の check を確実に畳む今、呼び出し側を **COW 内蔵の
safe op(`make_array_unique`、punch/plug は force_unique=true の COW 版、sort/reverse は COW 内蔵 `swap`)に
置き換えれば削除でき、かつ CSE-safe になる**(COW が mutate op の codegen 内にあり各 mutate が新しい配列 value を
線形に返すので、同一 `x` への `force_unique(x)` の重複が生じず CSE が纏める対象が無い)。§11.3、surviving unsafe
RMW primitive 削除計画。上の増加専用の `_unsafe_grow_size`(§3.1)もこのパターンの一例。

**punch/plug も例外にしない。** `act` の「action 失敗時に書き戻しも clone も走らせない」保証(`act` の doc)を
担保しているのは *punch/plug が unchecked なこと* ではなく、`act` の `if unique { punch/plug } else { read + set }`
という **分岐構造**である。punch は unique 枝でだけ呼ばれ(unique な配列にしか当たらない)、COW 版 punch でも実際には
clone しない(provable なら畳み、実行時 unique なら COW check が no-clone に落ちる)。shared は `else` の遅延 clone
(成功時のみ `set`)へ行き punch を呼ばない。よって COW 版に寄せても no-clone-on-fail は保たれ、unique-check-elim が
unique 枝の冗長 check を畳む。コスト: 非 provable かつ runtime-unique の時だけ punch の COW check が1回冗長に走るが
安く、clone が実際に起きる場面では clone コストが支配的。

注: `unreleased`(未初期化スロットへの書き込み)の unsafe さは uniqueness とは直交で残り、
`Array::_unsafe_initialize` 側に集約される。

## 4. `Array` を組む primitive(`Storage` は codegen 内部に閉じる)

`Storage` は **`Array` の内部表現でしかない**。user-visible な型にせず、`Storage::` の Fix 関数 API も作らない。
生ストレージへの操作(領域確保 / 要素 read / 未初期化 write / データポインタ取得)は **`Array` レベルの
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
  **非 retain(unretained)版は作らない**: read-fold の per-element RC は、この retaining getter に optimizer
  (borrow 化 §2.1 + retain/release 相殺 §2.2)を効かせて落とす方針(§11.4 の read-fold note)。read-fold の要素
  アクセスはこの retaining getter を通す(`ArrayIterator::advance` もこれを使う)。
- `Array::_unsafe_initialize : I64 -> a -> Array a -> Array a` — **未初期化スロットへの write**(release せず、
  COW せず、新しい `Array` value を線形に返す)。fresh capacity を埋める builder(from_map/fill/reserve/append)が
  Fix-source ループから呼ぶ。InlineLLVM が `arr.@_storage` の idx へ write。**optimization-safe**: (i) 線形(毎回
  新しい `Array` を返し、同一 `x` への重複呼びが生じない)ので、`_unsafe_force_unique` を壊した CSE 融合ハザードが
  無い。(ii) 適用先は allocate / reserve / grow 直後の **provably-unique** な storage だけ — uniqueness は fresh
  alloc から来て解析と実体が一致するので、uniqueness-divergence の miscompile が無い(外部から uniqueness を
  「表明」する `_unsafe_force_unique` と違い、uniqueness を「確立」する alloc を解析が追える)。残る `_unsafe_`
  契約は「スロットは未初期化(caller が埋める)」「in-bounds(`< _cap`)」だけ(uniqueness とは直交、§3.3 注)。旧
  `_unsafe_set_bounds_uniqueness_unchecked_unreleased` の後継。
- `Array::_unsafe_empty_capacity_unchecked : I64 -> Array a` — 指定 capacity の空 `Array` を確保(storage 未初期化、
  refcount 1)。InlineLLVM が内部で storage box を alloc し `Array { _storage, _size:0, _cap:cap }` を構築。`empty`
  の後継(name/contract 不変)。要素数は malloc サイズ計算に使うだけで storage には保存しない(`_cap` が覚える)。

「**上書き + 旧要素 release**」の write(`set`/`swap`)と in-place mutate(`mod`/`act`/`push_back`/`mutate_elements`
等)は、いずれも **COW を op 内に内包した `Array` レベルの InlineLLVM**(§5)。生ストレージへの直接 write + 旧要素
release はその body の codegen が行い、Fix ソースから「storage への上書き + release」を呼ぶ経路は無い。`mod`/`act`
は punch/plug(hole へ書くので release 不要)。

- bounds check(`_check_range`)は value の `_size` に対する `Array` レベル op として残す。
- **FFI のデータポインタ**は `Array` の FFI ヘルパ経由(§7 `Array::borrow_elements` / `mutate_elements`)。storage は
  Boxed 値でないので generic `borrow_boxed` へは委譲せず、いずれも **Array の専用 InlineLLVM**が storage を直接
  扱う(`borrow_elements` は storage を retain して buffer 先頭 ptr を callback へ渡し release、`mutate_elements` は
  COW 内蔵、§13.3-2)。codegen 側は `get_data_pointer_from_boxed_value` の array 分岐を storage の buf レイアウトへ
  合わせる(§7)。capacity は value の cap が持つ。

**bulk op は Fix-source を基本とし、measurement で回帰するものだけ InlineLLVM を残す**。`fill` は
`_unsafe_empty_capacity_unchecked` + `_unsafe_initialize` ループの Fix-source で書け、最適化器がループを
ベクトル化して InlineLLVM `fill` と同等のコードになる。よって `fill` は Fix-source にし、InlineLLVM の
`array_unsafe_fill` は削除する。COW clone /
reserve の storage コピーは retain-per-slot の別パターンなので、Fix-source 化を測定で確かめ、ベクトル化が届かず
回帰する場合のみ `clone_array_buf` を storage に retarget した InlineLLVM を残す。

これらは現行 `Array` の InlineLLVM body(`_unsafe_get_bounds_unchecked`、
`_unsafe_set_bounds_uniqueness_unchecked_unreleased`、`_unsafe_empty_capacity_unchecked`、`create_obj`)と 1 対 1 に
対応し、`Array` object の `ARRAY_BUF_IDX` ではなく storage box の生要素領域(index 0)に対して動くように移す。

## 5. `Array` primitive の移行

現行の `Array` InlineLLVM primitive はそれぞれ、`{ _storage, _size, _cap }` + `Storage` primitive の上に再構成する
(InlineLLVM のまま `Storage` 上で動くものと、Fix-src(`Storage` primitive の合成や value field の参照)に
なるものがある)。「実装」列がその別を示す。完全な一覧は `investigation-notes.md` §5:

| 現行 `Array` primitive | 移行後 | 実装 |
| --- | --- | --- |
| `@size`(`extract_field(ARRAY_LEN_IDX)`) | `arr.@_size` — register 読み出し(目標) | Fix-src(`arr.@_size` を返すだけ。現行の InlineLLVM `extract_field` は不要に) |
| `@capacity`(`extract_field(ARRAY_CAP_IDX)`) | `arr.@_cap` — register 読み出し | Fix-src(`arr.@_cap` の field 参照) |
| `_unsafe_get_bounds_unchecked` | `arr.@_storage` を直接 retaining read | **InlineLLVM**(read。borrow 化で `_storage` の retain を除く。unretained 版は作らない、§4) |
| `set`(make_unique, check, write, 旧要素 release) | `_storage` を unique 化(COW)+ `_check_range(i, _size)` + write(旧要素 release)を1 body で | **InlineLLVM**(現行 `InlineLLVMArraySetBody` を `Storage` 上へ re-target。in-place mutator ルール) |
| `_unsafe_set_bounds_uniqueness_unchecked_unreleased` | `Array::_unsafe_initialize`(未初期化スロットへ線形 write) | **InlineLLVM**(Array レベル、COW/release なし、§4) |
| `_unsafe_set_size` | `_unsafe_grow_size`(増加専用)へ改名: 内部 unique check(COW、optimizer 除去)+ value `_size` を伸ばす(新スロット未初期化)。減少は `_pop_back_nonempty` が release+shrink | **InlineLLVM**(in-place、内部 COW) |
| `_unsafe_empty_capacity_unchecked(cap)` | storage box を内部 alloc し `Array { _storage, _size:0, _cap:cap }` を構築 | **InlineLLVM**(storage alloc は codegen 内部、§4) |
| `_unsafe_fill_size_unchecked(n, x)` | `_unsafe_empty_capacity_unchecked(n)` 確保, `Array::_unsafe_initialize` の loop で埋め(最適化器が InlineLLVM 同等にする) | Fix-src |
| `_pop_back_nonempty` | **`_unsafe_pop_back_nonempty` へ改名**(empty で呼ぶと `last_idx = -1` の範囲外読み出し + ゴミを boxed 要素として release する UB = memory-unsafe なので、規約どおり `_unsafe_` を付ける)。`_storage` を unique 化, 末尾要素を noretain read して release, value `_size -= 1`(empty check は caller の `pop_back` が担う) | **InlineLLVM**(in-place、COW) |
| array literal `[..]` | `_storage` 確保, 埋め, `_size = len, _cap = len` | compiler lowering(既存の array-literal codegen を `Storage` allocate + initialize に向ける) |

`push_back` / `reserve` / `resize` は既に Fix レベル。これらは今後 value の `_cap` を読み(register)、伸ばす
ときに新しい `_storage` を確保して `_cap` を更新し、value の `_size` を設定する。

**uniqueness-check-less な mutate primitive は作らない — in-place write は COW を op 内に内包する。** `set` /
`mod` / `act` / `swap` / `push_back` / `_unsafe_grow_size` の書き込みは、unique check(`if unique { in-place }
else { clone }`)を Array レベルの InlineLLVM(現行 `set` の `force_unique`)に内包し、unique-check-elim が
provably-unique のとき check を畳んで in-place にする。**「caller が `_unsafe_force_unique` してから
`_unsafe_set_bounds_uniqueness_unchecked_unreleased` で無条件に書く」ような、uniqueness を別 op が確立した前提で
書く mutate primitive は新設しない。既存のこのペアも削除する(§11.3)。** 理由: unchecked な in-place mutate は
「この配列は unique」という **verify されない表明**で、正しさが op の外(caller の事前 force-unique)に依存する。
borrow / provenance / unique-check-elim はどこで値が unique か shared かを追って RC を置き check を畳む解析なので、
この外部不変条件を op から検証できず、解析の uniqueness モデルが実体とズレると shared 配列への write(miscompile)を
招く。check を op 内に持てば入力が何であれ健全(shared なら clone)で、optimizer の fold は純粋に安全な簡約になる。
上書き + 旧要素 release は InlineLLVM(`set`/`swap`)の body の codegen が Storage の生ストレージへ直接行う
(Fix レベルの Storage write プリミティブは無い、§4)。`mod`/`act` が使う punch/plug も COW 内蔵の版に寄せる
(uniqueness-unchecked 版は削除、§3.3)ので、この規則に例外は無い。「bulk op は Fix-source を基本」(§4)は
fill/reserve のコピーの話で、per-element の in-place mutator は InlineLLVM に残す。

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

`Array` が boxed でなくなり、しかも storage も **Boxed 値ではない**(型ですらない、§2.2)ので、FFI のポインタ系
generic(`_get_boxed_ptr`、`mutate_boxed`/`borrow_boxed`、`boxed_to_retained_ptr`/`boxed_from_retained_ptr`、
`with_retained`。すべて `[a : Boxed]`)は **Array にも storage にも直接は効かない**。Array のデータポインタ FFI は
**Array レベルの InlineLLVM ヘルパ**に集約し、その codegen が storage の refcount と buffer ポインタを直接扱う。

- **`Array a : Boxed` instance を削除する**(`stdlib.rs` のハードコード instance)。storage は型でないので `Boxed`
  instance を持たない。**ユーザー可視の破壊的変更**: Array へ直接 `array.borrow_boxed(...)` /
  `array.boxed_to_retained_ptr` していたコードは型エラーになり、下の `Array::borrow_elements` 等へ書き換え。許容。
- **データポインタ**(生要素先頭): Array の FFI ヘルパ **`Array::borrow_elements`(+ `_io` / 可変版
  `mutate_elements` / `_io`)** で取る。これらは **Array の InlineLLVM**で、codegen が storage を retain(borrow)して
  buffer 先頭ポインタ(現行 `get_data_pointer_from_boxed_value` の `is_array` 分岐相当 = storage の buffer index)を
  callback へ渡す。storage は Boxed 値でないので、`with_retained`/`borrow_boxed` の意味論(callback 中だけ生存・
  clone しない)を **codegen で直接実装する**。返る番地は現状の要素領域と同じ。ポインタは
  callback 中のみ有効・`borrow_elements` は書き換え不可・`mutate_elements` は COW 後に可変。
  - **`String` の公開 API(`_get_c_str`/`borrow_c_str`)は不変** — 内部を `_data`(= `Array U8`)の `borrow_elements`
    経由へ差し替えるだけ。String FFI ユーザーは影響なし。std の byte-array FFI(to/from_bytes)も
    `borrow_elements`/`mutate_elements` へ内部変更。
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
- FFI body 全体の `is_box`/`is_unbox` assert(Array を boxed と仮定していた箇所)は、Array が unbox・storage が
  型でない生 object になったことに合わせて更新する。

## 8. Debug info

`<array buffer>` debug 型と `<array size>` メンバ(`to_debug_type` / `ty_to_debug_struct_ty`)を書き直す:
`Array` の debug 型は 3 field の value struct(storage pointer、size i64、cap i64)になり、FAM/
`DEBUG_ARRAY_ASSUMED_LEN` の要素配列記述は storage object の debug 型(codegen が組む ad-hoc ObjectType、
tycon 無し)へ移る。

## 9. 段階的移行(tests を green に保つ)

素朴にやると、この変更は ~40 の layout-constant 箇所と型/FFI/RC/debug 機構を一度に触る。commit 間で suite が
green を保つよう段階化する:

1. **storage object の codegen 基盤を導入**: `array_storage_object_ty(elem_ty)`(ad-hoc ObjectType
   `{ ControlBlock, buffer }`、§2.2(3))+ alloc / 要素 read / 未初期化 write / data-pointer / free-only RC の
   codegen ヘルパ。まだ `Array` からは未使用(dead-code 警告が「配線待ち」を示す)。storage は Fix の型でない
   ので Fix レベルの直接 unit-test はできず、stage 2 の Array op 経由で検証する(必要なら小さな InlineLLVM smoke)。
2. **`Array` の InlineLLVM body を storage codegen 経由に付け替える**。`Array` の object shape を *まだ現行 boxed*
   のままにできれば(storage を内部に持つ中間形)、ABI 反転前に既存 test で storage op を検証する。中間形が
   表現できなければ stage 3 に畳む。
3. **`Array` の値レイアウトを unbox `{ Ptr, size, cap }` に反転**(§2.2)。`ty_to_object_ty` の `Array` arm、
   `to_embedded_type`、`create_obj`、`size_of`、custom `build_traverse`/`build_retain` arm(§3/§2.2(4))、
   不可分 unit 述語(§3.2)、layout-constant 箇所すべて(`investigation-notes.md` §8)を一斉に更新。`Array a : Boxed`
   instance を削除、`@size`/`@capacity` を extractvalue 版へ、`String`/FFI chain(§7)と PunchedArray(§6)を
   書き換える。
4. **Debug info**(§8)。
5. **検証**: 全 opt レベルで `cargo test --release`。array/string/punched-array/FFI の test。minilib +
   project_euler を memcheck 下で。要素 release を `_size` で駆動する点が最もリスクが高い変更 —
   shared/unique/COW/pop/resize/punch を跨いだ adversarial な memcheck。その後、write-loop の speedtest
   (`array_mod`、`arrayrw`、`write_by_range_fold`、`prime_table`、`push_back`)を再設計前 baseline と比較し、
   畳まれた check と vector op を LLVM IR で確認する。`push_back` は容量チェック `_size < _cap` が register に
   なることの、`write_by_range_fold` は bounds check が畳まれることの、それぞれ測定点。

## 10. ABI と性能

- **利点**: `get_size`/`get_capacity` が register 読み出しになる -> write-loop の bounds-check elimination と
  vectorize、および `push_back` ループの容量チェックの hoist が自然に出てくる(write ケースの
  `--no-runtime-check` 天井に安全に届く)。
- **コスト**: `Array` が by-value 3 word になるので、`Array` を受け/返す関数はすべて pointer 1つでなく
  `{ptr, i64, i64}` を渡す — ABI が太る。retain/release/traverser の signature と closure ABI も波及する。
  bounds-check/容量チェックの利点が array-heavy コードでは支配的なはず。array 非依存コードと入れ子配列は
  小さな by-value/メモリコストを払う。仮定せず測る。
- **リスク**: 要素の寿命(§3)が正しさに直結する部分 — count を誤ると leak か double-free。step 9.5 の段階的
  memcheck がその番人。

## 11. 方針・未決事項

1. **未決** — step 2 の可否。`Array` を boxed のまま `Storage` を内包する中間段が作れるか、それとも反転は atomic で
   なければならない(step 2 を step 3 に畳む)か。実装時に判断する。
2. **決定 — storage を Fix の型にしない。codegen 専用の object layout としてのみ存在させ、Fix 露出は `Array`
   インターフェースだけにする(§2.2/§4)。** storage には tycon も `Boxed` instance も与えない。生ストレージ op
   (allocate / get / 未初期化 write / data-ptr)は `Array` の InlineLLVM body の codegen 内に閉じる。**storage を型に
   しないので「裸の storage 値」はそもそも Fix の値として存在せず、ユーザーへ漏れようがない** — これが要素寿命
   (b) の「ユーザーは `Array` しか持たず裸の storage を持たない」不変条件(§3)を型レベルで保証する(storage の
   destructor は生メモリを free するだけで要素 release は Array の `size` が駆動するので、Array より長生きした裸
   storage は use-after-free になるが、そういう値を作れない)。FFI の公開面は §7 の scoped な Array borrow ヘルパ
   (`Array::borrow_elements` 系、コールバック中だけ有効な `Ptr` を渡す)だけにする。これにより plan §8(2)(a) 型の
   composable な隠れ穴 primitive(unretained element getter 等)が Fix レベルに存在しなくなる。
3. **決定(採用した方向) — 事前手動 unique-check を要する unsafe 関数を減らす。** `_uniqueness_unchecked` 系の
   「呼び出し側が事前に unique を保証する」primitive を、自前で unique-check する safe 版に寄せる(unique-check-elim
   が provably-unique で畳んで同性能)。size 書き込み(`_unsafe_force_unique` + 旧 `_unsafe_set_size`)は §3.1 の
   `_unsafe_grow_size` が内部 check 化して既にこの一例。fill を Fix-source 化するのと同じ方向で、redesign を機に
   unsafe API の表面積を縮める(削除対象は §3.3。punch/plug の uniqueness-unchecked 版も含む = 例外なし)。前提は
   §3.3(is_unique)と §3.1(unique-only な size 書き込み)が正しく効くこと。surviving
   unsafe RMW primitive の削除計画に接続する。
4. **未決(測定) — boxed 要素の zero-RC read-fold。** 削除する unretained getter が担っていた「boxed 要素の合計
   ループで per-element retain/release を 0 にする」性能は、redesign 後は retaining `_unsafe_get_bounds_unchecked`
   + optimizer(borrow 化 §2.1 + retain/release 相殺 §2.2)で出す方針(§4)。これは `Storage`/`Array` 境界とは
   直交する既存の optimizer 課題で、解が何であれ `Array` レベルに閉じる(`Storage` を露出しても解決しない)。
   retaining getter の per-element retain/release を optimizer が実際に消し切れているかは RC IR dump / cachegrind で
   測る。消し切れていなければ、要素 borrow のより良い表現か atomic な fold op という別途の `Array` レベル最適化で
   埋める — plan §8(2)(a) の DOESNT-FIT(2 属性で表せない unretained element)を再現する unsound な primitive の
   再導入では埋めない。

## 12. あとがき: 検討して退けた案

**要素の寿命(§3)の代替案**:
- **(a) count を `Storage` に持つ。** すると `get_size` が再び heap から読む — 本改修の意味が消える — ただし
  count を value に *複製* する場合を除く。複製すると size 変更のたび 2 箇所に書いて同期する必要が出る。却下。
- **(c) generic な custom-traversal ヘルパに `len` を渡す**(既存の hole path
  `build_release_mark_nonnull_boxed_with`)。採用案(Array value が release を駆動)の実装手段であって別モデル
  ではないので、独立の選択肢としては扱わない。

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
- **borrows_operand(i)**: operand i を borrow(consume しない)か。default は全 operand consume。`borrow.rs` は
  `borrows_operand(i)` か result_prov に `Arg(i, ·)` として現れる operand のみ非 consume とする。
- **result_prov**: 結果の各 boxed leaf の provenance — `Fresh`(新規 unique)/ `Arg(k, path)`(operand k の passthrough
  alias。`root()` が alias とみなし retain を省く)/ `Dyn`(保守的)。
- **記法(§2.2 準拠)**: 以下の pseudocode で `arr.@_storage` は **storage への生ポインタ(value field 0)** を指す
  codegen 上の読み出しであり、Fix レベルの struct getter ではない(`Array` は primitive、storage は型でない)。
  同様に `arr.@_size`/`@_cap` は value field 1/2 への extractvalue。`Array` 値は **1つの不可分 custom-RC unit**
  なので、borrow/provenance の属性は「`Array` unit」に対するもの(struct field leaf ではない)。
- redesign 共通: size/cap は value field 読み。COW helper `make_array_unique` は要素数を value `size` から読み、
  storage を unique 化する(内部名は実装時に確定)。flat boxed array での provenance `path:[]`/`Fresh` は、この
  `Array` custom unit を指すよう再表現する。

**(1) force-unique 分岐あり — COW を畳める(`unique_check_operand`/`assuming_unique` + `force_unique` field)**

- **`set`**(`InlineLLVMArraySetBody`、InlineLLVM)— 疑似: `if force_unique { arr = make_storage_unique(arr) }`;
  `if runtime_check() { check idx < _size }`; `write(arr.@_storage, idx, value, release_old=true)`; `ret arr`。
  force-unique: `unique_check_operand = Some{0, [0]}`(operand 0 = arr の Storage leaf)、`assuming_unique` で
  `force_unique=false`。bounds は `runtime_check()` gate(field でない)。borrows: なし(arr[0]・value[2] consume)。
  prov: `Fresh`(Storage leaf)。**上書き + 旧要素 release はここに内在**。
- **`swap` / `unsafe_swap_bounds_unchecked`**(`InlineLLVMArraySwapBody`、InlineLLVM)— 2スロットを noretain read して
  cross-write(release 無し)。フィールド **`force_unique: bool` + `bounds_checked: bool`**(bounds_checked は
  registration 固定・非 fold、swap=true / unsafe=false)。force-unique: `Some{0,[0]}`。borrows: なし。prov: `Fresh`。
- **`punch`**(`InlineLLVMArrayPunchBody`、InlineLLVM)— `(PunchedArray a, a)` を返す。`if force_unique {
  make_storage_unique }`; `elem = noretain_read(idx)`(hole を残す、size 不変); `ret (PunchedArray{_arr:arr, _idx:idx},
  elem)`。force-unique: `Some{0,[0]}`。borrows: なし。prov: 現状 default `Dyn`(要 override 検討)。redesign では
  `mod`/`act` が使う COW 版のみ残す(no-COW 版削除)。
- **`plug`**(`InlineLLVMPunchedArrayPlugBody`、InlineLLVM)— `PunchedArray{_arr,_idx}` を分解、`if force_unique {
  make_storage_unique_with_hole(_arr, Some(idx)) }`; `write(idx, elem, release_old=false)`; `ret arr`。
  force-unique: `Some{container_index:1, path:[0]}`(operand 1 = punched、その field 0 = `_arr`)。PunchedArray は
  Fix struct なので field 0 で `_arr` に届き、`_arr`(Array custom unit)の storage uniqueness を見る。borrows: なし
  (elem[0]・punched[1] consume)。prov: `Fresh`。
- **`unsafe_is_unique`**(`InlineLLVMIsUniqueFunctionBody`、InlineLLVM)— `(Bool, a)`。`if !assume_unique &&
  obj.is_box { flag = build_branch_by_is_unique(obj) } else { flag = const true }`。フィールド `assume_unique: bool`。
  force-unique: `unique_check_operand = Some{0, []}` iff `!assume_unique`; `assuming_unique` が `assume_unique=true`
  (flag が const true に畳み、`if unique{}else{}` を back end が消す)。borrows: **なし(operand 0 を意図的に consume)**。
  prov: **`Dyn` 固定(TRAP)** — 第2成分は引数そのものだが passthrough にすると「後続 use が arg を shared に読ませる
  retain」を抑止し fold が誤って on になる。**Dyn を保つ**。redesign: `[a:Boxed]` 追加。Array には下記
  `_unsafe_is_storage_unique` を使う。
- **NEW `_unsafe_grow_size`**(`_unsafe_set_size` から改名、InlineLLVM)— 旧 body は `insert_field(LEN, n)` のみで
  COW 無し。**redesign で force-unique 分岐を新設**(`force_unique` field + `unique_check_operand=Some{0,[0]}` +
  `assuming_unique`)— value `_size` を n に伸ばす前に Storage を COW。理由: `_size` を書くのは unique な `_storage` に
  だけ(§3.1)。畳めるので provably-unique では同性能。borrows: なし。prov: `Fresh`。
- **NEW `truncate`**(現状 Fix-source の pop_back ループ -> InlineLLVM 化)— `if n >= _size { ret arr }`; `if
  force_unique { make_storage_unique }`; `release_range(arr.@_storage, [n,_size))`; `ret arr{_size=n}`。force-unique
  分岐あり(`Some{0,[0]}`、畳める)。borrows: なし。prov: `Fresh`。§13.3-1。
- **NEW `mutate_elements` / `_io`**(専用 InlineLLVM)— `if force_unique { make_storage_unique }`; `ptr =
  data_ptr(arr.@_storage)`; `r = act(ptr)`; `ret (arr, r)`。force-unique 分岐あり(`Some{0,[0]}`)。§13.3-2。

**(2) COW 固定(畳めない)**

- **`_unsafe_pop_back_nonempty`**(`_pop_back_nonempty` から改名、InlineLLVM)— `arr = make_storage_unique(arr)`
  (**無条件**、`unique_check_operand` override 無し = 非 fold); `last = _size-1`; `elem = noretain_read(last)`;
  `release(elem)`; `_size = last`。borrows: なし。prov: `Fresh`。empty で UB。

**(3) COW/uniqueness 分岐なし(caller が unique 保証、または read-only)**

- **`_unsafe_get_bounds_unchecked`**(InlineLLVM)— `arr = noretain(arr)`(borrow); `elem = retaining_read(
  arr.@_storage, idx)`。**borrows: operand 0 = borrow**。prov: `Dyn`(共有 container から retain 済み要素)。存続。
  read-fold の per-element RC は optimizer が落とす(§11.4)。**unretained 版は作らない**(§4・plan §8(2)(a) の再導入回避)。
- **NEW `_unsafe_initialize`**(`Array::_unsafe_initialize`、InlineLLVM)— `write(arr.@_storage, idx, value,
  release_old=false)`(未初期化スロット、COW 無し・線形)。borrows: なし(arr[0]・value[1] consume)。prov:
  `Fresh`(storage leaf)。旧 `_unsafe_set_bounds_uniqueness_unchecked_unreleased` の後継(Array レベル)。
  optimization-safe の理由は §4。
- **`_unsafe_empty_capacity_unchecked`**(InlineLLVM)— storage box を内部 alloc し `Array { _storage, _size:0,
  _cap:cap }` を構築。borrows: なし。prov: `Fresh`。storage alloc は codegen 内部(Fix 関数化しない)。
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
- **`get_data_pointer_from_boxed_value` / `_get_boxed_ptr`**: `is_array` 分岐を Storage(`STORAGE_BUF_IDX`)へ。
  `_get_boxed_ptr` は borrow(`borrows_operand(0)=true`)。`borrow_boxed` / FFI は `@_storage` 経由。
- **Array value の field 参照・構築**: `Array` は primitive なので struct getter / `MakeStruct` は使わず、value の
  field 0/1/2 への `extractvalue`(read)/ `insertvalue`(rebuild)を codegen が直接出す。`@size`/`@capacity` は
  field 1/2 の extractvalue、storage Ptr は field 0(codegen 内部のみ)。`mutate_elements` 等の in-place rebuild は
  field 0 を新 storage Ptr で差し替える insertvalue。

**(5) 削除**(理由は前掲): `_unsafe_force_unique`、`_unsafe_set_bounds_uniqueness_unchecked_unreleased`(->
`Array::_unsafe_initialize`)、`_unsafe_get_linear_bounds_unchecked_unretained`(両変種)、punch/plug の
uniqueness-unchecked(no-COW)版、`array_unsafe_fill`(fill -> Fix-src)、`@size`/`@capacity` body(-> field 読み)、
`_get_ptr`。

**(6) 型・登録の変更**(op ではないが必要): `Array` は `TyConVariant::Array` のまま、`ty_to_object_ty` の `Array` arm を
**unbox `{ Ptr, size, cap }`** へ変更(index 定数 `ARRAY_STORAGE_IDX` / `ARRAY_SIZE_IDX` / `ARRAY_CAP_IDX`)。storage
は codegen 専用 ObjectType として持つ(ad-hoc コンストラクタ `array_storage_object_ty` と `STORAGE_CTRL_IDX` /
`STORAGE_BUF_IDX` を追加、§2.2)。hardcoded `Array a : Boxed` instance を **削除**、`Array` を不可分 unit 述語へ
追加(§3.2)。詳細は §2.2/§7。

### 13.2 std.fix Fix 関数・trait instance(public シグネチャは特記以外すべて不変)

**追加(いずれも Array の InlineLLVM。storage は Boxed 値でないので codegen が storage を直接扱う):**

| 名前 | 契約 |
| --- | --- |
| `Array::borrow_elements : (Ptr -> b) -> Array a -> b` | 要素先頭 Ptr を callback に借用。**専用 InlineLLVM**: codegen が storage を retain -> buffer 先頭 ptr を `f` へ -> release(`with_retained`/`borrow_boxed` 意味論を codegen で再現、clone なし、§13.3-2)。`array.borrow_boxed` の後継。ポインタは callback 中のみ有効・書き換え不可 |
| `Array::borrow_elements_io` | IO 版(専用 InlineLLVM) |
| `Array::mutate_elements` | Ptr 経由 in-place mutate。**専用 InlineLLVM**(`set` と同じくその場で COW -> data ptr -> act -> value rebuild、§13.3-2) |
| `Array::mutate_elements_io` | IO 版(専用 InlineLLVM) |
| Array 用 uniqueness assert(名前 TBD) | `_unsafe_is_storage_unique` ベース。`arr.assert_unique` の後継 |

**変更:**

- builder(`_unsafe_force_unique` 撤去 + `_unsafe_set_size`->`_unsafe_grow_size` + unreleased write -> `Array::_unsafe_initialize`): `append`, `from_map`, `reserve`, `push_back`, `resize`
- `fill`: 削除プリミティブ -> **Fix-source**(`Array::_unsafe_initialize` ループ)
- `mod`/`act`(punch/plug の no-COW -> COW、`unsafe_is_unique` -> `_unsafe_is_storage_unique`、act の分岐構造は維持): `mod`, `_unsafe_act_bounds_unchecked_identity`, `_unsafe_act_bounds_unchecked_tuple2`, `_unsafe_act_bounds_unchecked`
- `sort_by`, `reverse`: `_unsafe_force_unique` 撤去(COW `swap` が make-unique 済み)
- `pop_back`: 呼ぶ先が `_unsafe_pop_back_nonempty` に改名
- shrink 経路: `truncate` を効率化(現状の pop_back ループ -> 1 op の in-place shrink、§13.3-1)。`String::from_bytes` / `_unsafe_from_c_str` の切り詰めは `truncate` を呼ぶ
- `@size`/`@capacity`: Rust InlineLLVM -> Fix-src wrapper。`get_size`/`get_capacity` alias は不変
- `_unsafe_empty_capacity_unchecked`: Fix-src struct 構築(`empty` は name/contract 不変)
- String C-interop(`_data.@_storage` 経由へ、sig 不変): `_get_c_str`, `borrow_c_str`, `_unsafe_from_c_str`, `unsafe_from_c_str_ptr`(`_io`), `String::from_bytes`
- IO byte 関数(`mutate/borrow_boxed` -> `_elements`): `_read_line_inner`, `read_n_bytes`, `write_bytes`
- `assert_unique`: **`[a:Boxed]` 制約追加**(`arr.assert_unique` は compile error 化 -> Array 版へ誘導)
- 数値 trait instance(`mutate/borrow_boxed` -> `_elements`、`_unsafe_set_size` -> `_unsafe_grow_size`): `ToBytes`/`FromBytes`/`ToString` の U8..F64 一式(+ `to_string_exp`/`_precision`)

**削除:**

- Rust 登録プリミティブ: `_unsafe_force_unique`, `_unsafe_set_bounds_uniqueness_unchecked_unreleased`, punch/plug の uniqueness-unchecked 版, `_unsafe_fill_size_unchecked`
- trait instance: **`impl Array a : Boxed` を削除**(storage は型でないので Boxed instance を持たない)。**user-visible break**: `array.borrow_boxed` / `array.boxed_to_retained_ptr` が型エラー -> `borrow_elements` か自作 boxed 型でラップ
- `unsafe_is_unique` の unbox 枝(const-true)が `[a:Boxed]` 追加で dead

**改名(呼び出し側更新):**

- `_unsafe_set_size` -> `_unsafe_grow_size`(grow 経路)。呼び出し: `append`/`from_map`/`push_back`/`reserve`/`resize`/`read_n_bytes`/`unsafe_from_c_str_ptr`(`_io`)/数値 `to_bytes` 一式。**真の shrink を行う `String::from_bytes` と `_unsafe_from_c_str` の切り詰め経路は効率化した `truncate`(safe shrink、§13.3-1)を使う**
- `_pop_back_nonempty` -> `_unsafe_pop_back_nonempty`。呼び出し: `pop_back`

**不変(変更された callee を通すだけ):** `@`, `get_first`/`get_last`, `is_empty`, `find_by`, `get_sub`, `dedup`, `empty`,
`act`, `from_iter`/`to_iter`, sort 内部一式(`_introsort`/`_heap*`/`_insertion*`/`_mergesort*`/`sort`/`sort_stable*`)、
全 Array trait impl(`Zero`/`Add`/`Eq`/`LessThan`/`Functor`/`Monad`/`ToString`/`Indexable`)、FFI 定義
(`mutate_boxed`/`borrow_boxed`/retained-ptr — Array を受けなくなるだけ)、`Destructor::mutate_unique_io`(box なので `[a:Boxed]` OK)、
String の大半、PunchedArray 型(新レイアウトを継承、punch/plug/traverse の Rust body だけ retarget)。

### 13.3 要検討(設計ギャップ)

1. **shrink 経路 — 既存 `truncate` を効率化し、それを使う(決定)。** `_unsafe_grow_size`(前提 `n >= _size`)は
   `String::from_bytes`(null terminator の後にバイトが続くと真の shrink)や `_unsafe_from_c_str` の一部・数値
   `to_string` の over-allocate 経路を表せない。ただし shrink の public API は **既に `truncate`(先頭 n 要素を残す、
   `n >= size` で no-op)が存在する**ので、**新しい `shrink_size` は追加しない**。`truncate` を、現状の pop_back
   ループから **1 op の in-place safe shrink**(`n >= _size` なら no-op、そうでなければ COW if shared + 切り詰め要素
   `[n, _size)` を release + value `_size = n`)へ作り直す(in-place で `_storage` を触るので InlineLLVM)。
   **どちらも最適化で消える**: unique-check-elim が provably-unique で COW を畳み、unboxed 要素(`Array U8` 等)では
   要素 release が no-op になる — よって `String::from_bytes`(unique な `Array U8`)では `truncate` が実質「`_size` を
   下げるだけ」に落ち、旧 `_unsafe_set_size` の shrink と**同性能かつ安全**。`String::from_bytes` は
   `truncate(null_idx+1)` を、その他の切り詰め経路も `truncate` を使う。boxed 要素の配列でも安全に使える。
2. **`mutate_elements` / `borrow_elements` はいずれも Array の専用 InlineLLVM(決定)。** storage は Boxed 値でない
   (§2.2)ので、両者とも codegen が storage を直接扱う:
   - **`borrow_elements` / `_io`**: storage を retain -> buffer 先頭 ptr を `f` へ -> release(`with_retained` 相当を
     codegen で再現)。**clone しない**。返るポインタは要素バッファを正しく指し callback 中は生存する。この
     retain/release 1組は borrow 化で消せるが正しさには無関係。
   - **`mutate_elements` / `_io`**: `set` と同じく storage をその場で `make_array_unique`(実際に shared のときだけ
     COW)-> data ポインタ取得 -> act -> value を新 storage で rebuild。
