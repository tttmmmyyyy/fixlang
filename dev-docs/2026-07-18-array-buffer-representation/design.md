# Array/Storage 表現の再設計 — 設計

ステータス: 設計のみ、未実装。`investigation-notes.md`(コード実地調査の生データ)を土台とする。
目的は `Array::@size` を register 読み出しにして、back end が write loop からそれを巻き上げ
(hoist)、要素ごとの bounds check を畳み、vectorize できるようにすること — bounds-check
elimination の write-loop 側(`../2026-07-18-bounds-check-elim/`)。read-loop 側(iterator の
終了条件変更 + RC-IR simplifier)は既に出荷済みで、その doc が本件へ先送りした部分にあたる。

## 0. 概要

**やること**: `Array a` を boxed primitive から、boxed `Storage a`(refcount + 生要素だけ)を内包する
**unbox struct `{ _storage, _size, _cap }`** に変える。狙いは `@size`/`@capacity` が **register 読み出し**に
なること — write loop の bounds check と `push_back` の容量チェックが hoist/畳まれて vectorize する
(write-loop BCE)。read-loop BCE は既に別途出荷済み。

**確定した設計判断**(詳細は各節):
- `_size`/`_cap` は value(3 word、C++ `std::vector` 流)、refcount だけ Storage(§2.1)
- 要素解放は Array の custom traverser が value の `_size` で駆動、Storage は free のみ(§3, §3.1)
- RC-unit は PunchedArray と同じ「不可分 unit」扱い、名前付き述語で寄せる(§3.2)
- Array の uniqueness は専用 `Array::_unsafe_is_storage_unique`(`_storage` を retain せず覗く)。generic
  `unsafe_is_unique` は触らない(§3.3)
- bulk op(fill 等)は Fix-source(最適化器が InlineLLVM に並ぶことを実証、§4)
- FFI ポインタ系は `Storage` 経由。retained-ptr は `_size`/`_cap` を運べない(§7)
- 事前手動 unique を要する unsafe primitive を safe 版へ寄せて縮小(§3.3/§11.3)

**進め方**: tests green を保つ5段階の移行(§9)。実装は設計確定後。

**やらないこと**: `Storage` の public 化(internal と決定、§11.2)。

## 1. 問題

`Array a` は boxed primitive で、1つの heap allocation として次のレイアウトを持つ:

```
{ ControlBlock{refcnt i32, state i8}, len i64, cap i64, buf[FAM] }
```

そのため `get_size` は `extract_field(ARRAY_LEN_IDX)` — つまり **heap object からの load** に
lower される。`arr = arr.set(i, v)` を loop state に通す write loop では、`buf` への store が同じ
allocation を alias し、flexible-array-member (FAM) の GEP が宣言された struct 境界の外に出るため、
LLVM は `len` の load を loop-invariant と証明できない。よって bounds check `i < get_size(arr)` は
毎回 size を再 load し、残った check が vectorize を阻む(実証: `--no-runtime-check` で `array_mod`
は 1.42M -> 470k Ir に落ち、`arrayrw` が vectorize する)。size が heap にあることが root cause なので、
直し方は解析パスの追加ではなく表現そのものの変更である。

## 2. 表現

boxed primitive `Storage a`(refcount 付きの生の要素 storage)を導入し、`Array` を「長さと容量を value に
持つ unbox struct」として再定義する:

```
Storage a : boxed primitive   // { ControlBlock, buf[FAM] } — refcount と生要素のみ、メタ情報は持たない
type Array a = unbox struct { _storage : Storage a, _size : I64, _cap : I64 }
```

すると `get_size = arr.@_size = extractvalue` も `get_capacity = arr.@_cap` も **register 読み出し**。
write loop に通すと `_size` は loop 越しに運ばれる scalar になり、`set` は同じ `_size` を持つ新しい
`Array` value を返すので、LLVM は bound を loop-invariant と見なして `i < _size` を畳み、vectorize する。
同様に `push_back` ループの容量チェック `_size < _cap` も register で回る(§2.1)。size-normalization
パスも invariant-parameter の引き回しも要らず、標準の LICM/SCEV から自然に出てくる。

`Array` は 3 word の by-value aggregate `{ ptr, i64, i64 }` になる。`Array` の retain/release は既存の
unbox struct 機構を通じて *reference count* を `_storage` へ伝播する。ただし *要素* の寿命だけは custom な
一手が要る(§3)。C++ `std::vector` と同じレイアウト思想 — heap は生要素、メタ情報は value — だが、
COW 共有のため refcount だけは storage 側に持つ点が異なる。

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
  そうでなければ `_storage` の refcount を -1。`_size` は value から来る。`Array` は custom な `build_traverse`
  arm を保つ(現行 `Array` も `PunchedArray` も既に custom なので hack ではない) — value を読むように
  要素数駆動の traversal を移すだけ。
- **clone-if-shared**(refcount >= 2 での mutate 時の COW)は `_size` 個の要素を新しい `Storage` へ複製し、
  各要素を retain する。clone 側も `_size` を手元に持つ。
- `Storage` 自身の boxed destructor は **生メモリを free するだけ** — 要素 release は決してしない。所有側の
  `Array` が `_size` で駆動して既に release 済みだから。`Storage` は internal primitive で、ユーザコードは
  `Array a` を持ち、裸の `Storage a` は持たない。よって裸の `Storage` が自前で要素数を必要とする場面はない。
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
(実装時に `clamp_unit` の不揃いが本質か latent bug かを見極めてから統一する)。この統一リファクタと `clamp_unit`
不揃いの調査は、**RC/provenance 機構の本拠である `unique-check-elim` ブランチ側で行う**(redesign と独立した
cleanup で、両ブランチが同じ述語箇所を触る conflict も避けられる)。redesign(bce)は統一後の述語に `Array`
(unbox)を1行足すだけにし、`unique-check-elim` のマージでリファクタを取り込む。

### 3.3 `unsafe_is_unique` は Array の unbox 化で壊れる — 要修正

`unsafe_is_unique`(`InlineLLVMIsUniqueFunctionBody`)は現状 `if obj.is_box() { refcount を読む } else
{ const true }`(「unboxed object は常に unique」)。**Array が unbox になると else 枝に落ち、`_storage` が共有
(refcount >= 2)でも無条件に `true` を返す。** すると `mod`/`act`/COW の `if is_unique { in-place } else
{ clone }` が常に in-place を選び、**共有 storage を破壊(データ破損)**する。redesign の重大な破損点。

修正(generic `unsafe_is_unique` に Array 特別扱いを入れない): **`Array` 専用のビルトイン
`Array::_unsafe_is_storage_unique : Array a -> (Bool, Array a)` を追加**し、value の `_storage` の refcount を
**retain せずにその場で覗く**。`act` など Array の COW 判定はこれを使う。generic `unsafe_is_unique` は
現状のまま触らない — boxed 型(`Destructor`〔mutate_unique_io〕、FFI の gmp/mpfr 等、generic `assert_unique`)に
`is_box` で効き続けるので **不要にはならない**。unbox な Array *value* に generic を呼ぶと `const true`
(値としては常に unique)を返すが、COW 判定は Array 専用版を使うので破壊は起きない。

- `arr.@_storage.unsafe_is_unique` と書く案は不可: `@_storage` が `_storage` を **retain する**ので、borrow 化されない
  限り refcount >= 2 になり大抵 false を返す。専用ビルトインなら Array value を受けて `_storage` を extract せずに
  refcount を読めるので確実。
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
free-or-decrement**(`build_release_mark_nonnull_boxed_with`)、(3) **`is_unique` 関数**。これらは削除対象では
なく **COW/release の機構そのもの**で、unique-check-elim が (1)(3) を **コンパイル時に畳んで消す**((2) は基本
判定で残す)。redesign では (1) が Array の `_storage` に移るだけ。

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
`Storage::_unsafe_initialize` 側に集約される。

## 4. `Storage` primitive API

`Storage` は、`Array` の std コードを組み直す土台となる boxed storage primitive。`_unsafe_` はメモリ安全性を
壊し得る op(bounds unchecked / unretained / unreleased / uniqueness unchecked、未初期化 storage の生成)に
だけ付ける — size/capacity の読み出しのような安全な op には付けない(それらは value の field 読み出し、
または public な `get_*` になる)。以下はいずれも現行の array primitive と同じ unsafe 契約を持つ:

- `Storage::_unsafe_allocate : I64 -> Storage a` — 指定要素数ぶんの領域を確保、未初期化、refcount 1。
  要素数は malloc サイズの計算に使うだけで `Storage` には保存しない(`Array` の `_cap` が覚える)。
- `Storage::_unsafe_get : I64 -> Storage a -> a` — 要素読み出し(bounds unchecked)。返り値を **retain する**
  (boxed 要素なら refcount +1、caller が所有・release する)。通常の read(`@` 等)用。
- `Storage::_unsafe_get_unretained`(read-fold 用の非 retain 版)— 同じ読み出しだが **retain しない**借用ビューを
  返す。retain/release のコストが無い代わりに、caller は保持・release してはいけない(`Storage` が生きていて未変更の
  間だけ有効)。`Array String` のような boxed 要素の合計ループで毎要素 retain/release を避けるための最適化。
  現行の `_unsafe_get_linear_bounds_unchecked_unretained` に対応(配列を一緒に返して借用中の所有権を引き回す形)。
- `Storage::_unsafe_initialize : I64 -> a -> Storage a -> Storage a` — release **せず** に書き込む(live value を
  持たないスロットへの初回書き込み。fresh capacity を埋める用)。builder(from_map/reserve/append/fill)が呼ぶ。

「**上書き + 旧要素 release**」の write プリミティブは Fix レベルには **置かない**。上書き系(`set`/`swap`)は
InlineLLVM で、その body の codegen が Storage の生ストレージへ直接 write + 旧要素 release する(Fix 関数
`Storage::_unsafe_set` を経由しない)。`mod`/`act` は punch/plug(hole へ書くので release 不要)。よって
Fix-source から「Storage への上書き + release」を呼ぶ経路が無く、`Storage::_unsafe_set` は caller ゼロになる。
`_unsafe_get`(read)と `_unsafe_initialize`(新規スロット write)は Fix-source が呼ぶので Fix プリミティブとして
残す。
- FFI のデータポインタ(`Storage` の生要素先頭)は **generic `_get_boxed_ptr : [a : Boxed] a -> Ptr`** で取れる
  (`Storage` は Boxed)。専用 accessor は新設しない。redesign で要るのは codegen 側で
  `get_data_pointer_from_boxed_value` の array 分岐を `Storage` の buf レイアウトに合わせることだけ(§7)。
  `borrow_boxed` / `mutate_boxed` も同じく generic のまま `Storage` に効く。capacity は `Array` value の
  `_cap` が持つので、`Storage` に capacity op は置かない。

**bulk op は Fix-source を基本とし、measurement で回帰するものだけ InlineLLVM を残す**。`fill` は Fix-source の
要素ループ(`Storage::_unsafe_initialize`)で書けて、`from_map` ベンチで現行 InlineLLVM `fill` と同値と
実証済み(Ir 7,640,507 vs 7,640,521、差 0.0002%)— 最適化器がループをベクトル化して手書き LLVM に並ぶ。よって
`fill` は Fix-source にし、InlineLLVM の `array_unsafe_fill` は削除する。COW clone / reserve の storage コピーは
retain-per-slot の別 storage コピーで別パターンなので、実装時に個別に測って Fix-source 化するか判断する
(`clone_array_buf` を `Storage` に retarget して残すのが安全側の初期値)。

これらは現行 `Array` の InlineLLVM body(`_unsafe_get_bounds_unchecked`、`_unsafe_set_bounds_uniqueness_unchecked_unreleased`、
`_unsafe_empty_capacity_unchecked`、`create_obj`)と 1 対 1 に対応し、`Array` object の `ARRAY_BUF_IDX`
ではなく `Storage` の生要素領域(index 0)に対して動くように移す。bounds check(`_check_range`)は value の
`_size` に対する `Array` レベルの op として残す。

## 5. `Array` primitive の移行

現行の `Array` InlineLLVM primitive はそれぞれ、`{ _storage, _size, _cap }` + `Storage` primitive の上に再構成する
(InlineLLVM のまま `Storage` 上で動くものと、Fix-src(`Storage` primitive の合成や value field の参照)に
なるものがある)。「実装」列がその別を示す。完全な棚卸しは `investigation-notes.md` §5:

| 現行 `Array` primitive | 移行後 | 実装 |
| --- | --- | --- |
| `@size`(`extract_field(ARRAY_LEN_IDX)`) | `arr.@_size` — register 読み出し(目標) | Fix-src(`arr.@_size` を返すだけ。現行の InlineLLVM `extract_field` は不要に) |
| `@capacity`(`extract_field(ARRAY_CAP_IDX)`) | `arr.@_cap` — register 読み出し | Fix-src(`arr.@_cap` の field 参照) |
| `_unsafe_get_bounds_unchecked` | `Storage::_unsafe_get(i, arr.@_storage)` | Fix-src(`Storage::_unsafe_get` = InlineLLVM を呼ぶ。read なので borrow 化で `_storage` の retain を除く) |
| `set`(make_unique, check, write, 旧要素 release) | `_storage` を unique 化(COW)+ `_check_range(i, _size)` + write(旧要素 release)を1 body で | **InlineLLVM**(現行 `InlineLLVMArraySetBody` を `Storage` 上へ re-target。in-place mutator ルール) |
| `_unsafe_set_bounds_uniqueness_unchecked_unreleased` | `Storage::_unsafe_initialize`(この Array 版は §11.3 で削除) | **InlineLLVM**(`Storage` primitive) |
| `_unsafe_set_size` | `_unsafe_grow_size`(増加専用)へ改名: 内部 unique check(COW、optimizer 除去)+ value `_size` を伸ばす(新スロット未初期化)。減少は `_pop_back_nonempty` が release+shrink | **InlineLLVM**(in-place、内部 COW) |
| `_unsafe_empty_capacity_unchecked(cap)` | `Array { _storage: Storage::_unsafe_allocate(cap), _size: 0, _cap: cap }` | Fix-src(struct 構築 + `Storage::_unsafe_allocate` = InlineLLVM) |
| `_unsafe_fill_size_unchecked(n, x)` | `_storage(n)` 確保, `Storage::_unsafe_initialize` の loop で埋め, `_size = n, _cap = n`(最適化器が InlineLLVM 同等にする、実証済み) | Fix-src |
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
  で storage を読むが、`_arr.@_storage`(`Storage`)と value の `_size` へ移す。
- hole を飛ばす RC traversal(`build_traverse` の `is_punched_array` 特別扱い、`borrow.rs` の
  punched-array unit)は内側 array の `ARRAY_LEN_IDX`/`ARRAY_BUF_IDX` を読むが、内側 `Array` の value
  `_size` と `_storage` へ移し、hole index を除く `[0.._size)` を release する。これは §3 と同じ要素数駆動の
  traversal に index を1つ飛ばすだけ。
- hole の所有は `Storage` 粒度に留まる: `punch` は `_storage` から要素を1つ move out して hole を残し `_size` は
  不変、`plug` は release せずに hole へ書き戻す。

## 7. FFI, `Boxed`, `String`

`Array` が boxed でなくなるのが FFI に効く。**FFI のポインタ系関数はすべて `[a : Boxed]` 制約**
(`_get_boxed_ptr`、`mutate_boxed`/`borrow_boxed`、`boxed_to_retained_ptr`/`boxed_from_retained_ptr`、
`with_retained`)なので、Array が `Boxed` を外れると **どれも Array を直接受け取れなくなり、`array.@_storage`
(= `Storage`、これが Boxed)を通す**ことになる。

- **`Array a : Boxed` instance を削除し `Storage a` に与える**(`stdlib.rs` のハードコード instance)。これで
  上記 FFI 関数はすべて `Storage` を受ける。**ユーザー可視の破壊的変更**(Array に直接 FFI していたコードは
  `_storage` 経由へ書き換え)— 許容とする。
- **payload/data ポインタ**(`get_data_pointer_from_boxed_value` の `is_array` 分岐、`_get_boxed_ptr`、
  `mutate_boxed`、`borrow_boxed`): boxed 値の data への生ポインタ。`borrow_boxed : [a : Boxed] (Ptr -> b) -> a
  -> b` が公開 API で、FFI(String -> C 文字列、`Array U8` の to/from_bytes、ユーザーの subprocess/cairo/
  curl/gmp/mpfr 等)で多用される。移行後は `Storage` のデータ先頭ポインタを返す(= 現状の要素領域と
  同じ番地)。`get_data_pointer_from_boxed_value` の `is_array` 分岐を `Storage`(生要素は data index 側)へ差し替え。
  - **`String` の公開 API(`_get_c_str`/`borrow_c_str`)は不変** — 内部で `s.@_data.@_storage` に経路変更するだけ。
    String FFI ユーザーは影響なし。std の byte-array FFI(to/from_bytes)も `bs.@_storage.borrow_boxed` に内部変更。
  - **ユーザーが Array に直接 `array.borrow_boxed(...)` している箇所は壊れる**(Array が Boxed でない)。
    Array 専用の FFI ヘルパ **`Array::borrow_elements`(+ `borrow_elements_io` / 可変版 `mutate_elements` /
    `mutate_elements_io`)** を用意する。内部で `arr.@_storage.borrow_boxed(f)` に委譲する(`String::borrow_c_str` が
    `str.@_data.borrow_boxed(f)` へ委譲するのと同じパターン)。**名前を `borrow_boxed` にしないのは、Array が
    Boxed でなくなり "boxed" が事実に反するため**(`borrow_c_str` が中身を表す名前にしているのと同趣旨)。よって
    ユーザー FFI コードは `array.borrow_boxed` -> `array.borrow_elements` の書き換えが要る(Array が Boxed を外れる
    user-visible break の一部、許容)。ユーザー自作の boxed 構造体への `borrow_boxed` は不変(まだ Boxed)。
- **retained ポインタ**(`boxed_to_retained_ptr` / `boxed_from_retained_ptr`): retained pointer は **box
  (= Storage)しか捕捉しない**。Array の `_size`/`_cap` は value にあって box に無いので、**Array を retained
  pointer に往復させると `_size`/`_cap` が失われる**(現状は Array 自体が boxed で len/cap も heap にあるため
  往復で保存される)。よって retained-ptr の往復は `Storage` 単位(生の要素領域)に限られ、完全な Array を
  opaque pointer として保持したい FFI は `_size`/`_cap` を別途持ち運ぶ必要がある。これは新しい設計上の制約。
  **`borrow_elements` と違い、Array 版の retained-ptr ヘルパは用意しない**(box が無いので `Storage` しか渡せず
  誤解を招く)。完全な Array を C に opaque に渡したいユーザーは、**Array を自作の boxed 型で包んで対処する**
  (包めば Boxed になり `boxed_to_retained_ptr` が使え、`_size`/`_cap` も box 内フィールドとして保存される) —
  または `Storage` の retained-ptr に `_size`/`_cap` を別送する。ユーザー側で対処する方針。
- **String**: `String = unbox struct { _data : Array U8 }` の C-interop chain(`_get_c_str`、
  `_unsafe_from_c_str`、`borrow_c_str`)は `Array U8 : Boxed` に依存するので `_data.@_storage`(`Storage U8`)を
  通す。C 文字列ポインタ = `Storage U8` のデータポインタ。数値の `to_bytes`/`from_bytes`(Array U8 に
  `mutate_boxed`/`borrow_boxed`)も追随。
- FFI body 全体の `is_box`/`is_unbox` assert(Array を boxed と仮定していた箇所)は `Storage` に適用される。

## 8. Debug info

`<array buffer>` debug 型と `<array size>` メンバ(`to_debug_type` / `ty_to_debug_struct_ty`)を書き直す:
`Array` は 3 field の value struct(`_storage` pointer、`_size` i64、`_cap` i64)になり、FAM/
`DEBUG_ARRAY_ASSUMED_LEN` の要素配列記述は boxed `Storage` の debug 型へ移る。

## 9. 段階的移行(tests を green に保つ)

素朴にやると、この変更は ~40 の layout-constant 箇所と型/FFI/RC/debug 機構を一度に触る。commit 間で suite が
green を保つよう段階化する:

1. **`Storage a` を導入**: boxed primitive として allocate/get/set/initialize/data-pointer の op と、
   自前の RC(free のみの destructor)を持たせる。まだ `Array` からは未使用。`Storage` を直接 unit-test する。
   (未使用の dead-code 警告が「配線待ち」を示す。)
2. **`Array` の InlineLLVM body を `Storage` 経由に付け替える**。ただし `Array` は *まだ boxed* のまま —
   つまり `Array` が内部に `Storage` を持つが現行の object shape を保つ — として、ABI を反転する前に既存
   test で `Storage` op を検証する。(この中間 shape が表現できなければ step 3 に畳む。)
3. **`Array` を `unbox struct { _storage, _size, _cap }` に反転**。`ty_to_object_ty`、`to_embedded_type`、
   `create_obj`、`size_of`、custom `build_traverse` arm(§3)、および layout-constant 箇所すべて
   (`investigation-notes.md` §8)を一斉に更新。`Array` の `Boxed` instance を削除して `Storage` へ移し、
   `String`/FFI chain(§7)と PunchedArray(§6)を書き換える。
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
2. **決定 — `Storage` は internal に留める(public `@buf`/`@_storage` accessor は公開しない)。** 裸の `Storage` 値が
   ユーザーへ漏れると、要素寿命 (b) の「ユーザーは `Array` しか持たず裸の `Storage` を持たない」不変条件(§3)が
   壊れる(`Storage` の destructor は生メモリを free するだけで要素 release は Array の `_size` が駆動するので、
   Array より長生きした裸 `Storage` は use-after-free)。FFI の公開面は §7 の scoped な Array borrow ヘルパ
   (`Array::borrow_elements` 系、コールバック中だけ有効な `Ptr` を渡す)だけにする。
3. **決定(採用した方向) — 事前手動 unique-check を要する unsafe 関数を減らす。** `_uniqueness_unchecked` 系の
   「呼び出し側が事前に unique を保証する」primitive を、自前で unique-check する safe 版に寄せる(unique-check-elim
   が provably-unique で畳んで同性能)。size 書き込み(`_unsafe_force_unique` + 旧 `_unsafe_set_size`)は §3.1 の
   `_unsafe_grow_size` が内部 check 化して既にこの一例。fill が Fix-source で InlineLLVM と同値だった実証と同じ
   方向で、redesign を機に unsafe API の表面積を縮める(削除対象は §3.3。punch/plug の uniqueness-unchecked 版も
   含む = 例外なし)。前提は §3.3(is_unique)と §3.1(unique-only な size 書き込み)が正しく効くこと。surviving
   unsafe RMW primitive の削除計画に接続する。

## 12. あとがき: 検討して退けた案

**要素の寿命(§3)の代替案**:
- **(a) count を `Storage` に持つ。** すると `get_size` が再び heap から読む — 本改修の意味が消える — ただし
  count を value に *複製* する場合を除く。複製すると size 変更のたび 2 箇所に書いて同期する必要が出る。却下。
- **(c) generic な custom-traversal ヘルパに `len` を渡す**(既存の hole path
  `build_release_mark_nonnull_boxed_with`)。採用案(Array value が release を駆動)の実装手段であって別モデル
  ではないので、独立の選択肢としては扱わない。

## 13. 付録: 影響を受ける関数・InlineLLVM の全一覧(契約付き)

`std.fix` / `builtin.rs` / `stdlib.rs` / `object.rs` を通読して棚卸しした、本再設計が **追加 / 変更 / 削除 / 改名**
する対象の完全一覧。各項に契約(何をするか + `_unsafe_` の場合は caller が守るべき前提)を付す。末尾 §13.3 に
survey が surface した2つの設計ギャップを記す。

### 13.1 InlineLLVM / builtin / codegen

現行はすべて Array を boxed とみなし buffer を `gep_boxed(ARRAY_BUF_IDX)`、len/cap を
`extract/insert_field(ARRAY_LEN_IDX|ARRAY_CAP_IDX)` で触る。変更系の re-target は共通で **buffer -> `arr.@_storage`
の Storage buf、len/cap -> value の `_size`/`_cap` field**。

**追加:**

| 名前 | 契約(+ caller 前提) |
| --- | --- |
| `Storage::_unsafe_allocate : I64 -> Storage a` | cap 要素分 malloc、refcount 1、未初期化。cap は保存しない。caller: 露出する分だけ後で書く |
| `Storage::_unsafe_get : I64 -> Storage a -> a` | 要素 read、**retain する**。bounds unchecked。caller: idx 範囲内・スロット初期化済み |
| `Storage::_unsafe_get_unretained : I64 -> Storage a -> a` | 要素 read、**retain しない**借用ビュー。caller: 保持・release 不可、Storage 生存中のみ有効 |
| `Storage::_unsafe_initialize : I64 -> a -> Storage a -> Storage a` | 未初期化スロットへ write、旧要素 release せず、uniqueness check せず。caller: スロット未初期化・Storage unique・idx 範囲内・value 所有権を渡す |
| `Array::_unsafe_is_storage_unique : Array a -> (Bool, Array a)` | `_storage` の refcount を retain せず覗く。`result_prov` は `Dyn`(is_unique と同じ trap)。COW/BCE 判定用 |
| `truncate` の in-place shrink body(現状 Fix-source の pop_back ループ -> InlineLLVM 化。**shrink_size は追加しない**) | `n >= _size` は no-op、そうでなければ COW if shared + 切り詰め要素 `[n, _size)` を release + value `_size = n`。unique-check-elim + unboxed release で `Array U8`/unique では「`_size` 下げ」に畳む。`String::from_bytes` 等の shrink 経路は `truncate` を呼ぶ |
| (型)`Storage a : boxed primitive` | ControlBlock + 生要素 FAM。`ty_to_object_ty`/`create_obj`/`STORAGE_BUF_IDX(=1)` を新設。cap store は無し |
| `Boxed` instance | Array の hardcoded Boxed instance を **Storage a** へ移す |
| `Array` tycon | boxed primitive -> `unbox struct { _storage, _size, _cap }`。`@_storage`/`@_size`/`@_cap` を自動生成 |

**変更(re-target。特記のみ記載):**

| 名前 | 契約 + 変更点 |
| --- | --- |
| `set`(`InlineLLVMArraySetBody`) | bounds-checked write + 旧要素 release + 内部 COW(unique-check-elim が畳む)。**上書き+release はここに内在**。COW は Storage を clone |
| `swap` / `unsafe_swap_bounds_unchecked`(`InlineLLVMArraySwapBody`) | 2スロット read+cross-store、要素 release 無し、内部 COW |
| `punch`(`InlineLLVMArrayPunchBody`) | 要素を hole へ move out(noretain)、size 不変、内部 COW。`force_unique` は常時 true に collapse |
| `plug`(`InlineLLVMPunchedArrayPlugBody`) | hole へ write back(release 無し)、内部 COW。`force_unique` 常時 true |
| `get_data_pointer_from_boxed_value` | boxed payload ポインタ。`is_array` 分岐 -> Storage(`STORAGE_BUF_IDX`) |
| `unsafe_is_unique`(`InlineLLVMIsUniqueFunctionBody`) | `(Bool,a)` 返す。scheme に **`[a:Boxed]` 追加**(unbox Array を弾く)。body 不変、unbox 枝(const-true)は dead に |
| `make_array_unique` / `_with_hole` | clone-if-shared(hole skip 可)。Storage の uniqueness で分岐、`_cap` 分の Storage を確保し `_size` 要素 copy |
| array literal(`InlineLLVMArrayLitBody`) | `Storage(len)` 確保 + 要素 write、value struct 構築 |
| `_unsafe_empty_capacity_unchecked` | **-> Fix-src** の struct 構築(InlineLLVM 削除、`Storage::_unsafe_allocate` を使う) |
| `_unsafe_get_bounds_unchecked` | retain read、borrow。-> re-target または Fix-src(`Storage::_unsafe_get`) |
| `_unsafe_get_linear_bounds_unchecked_unretained`(+`_forceunique`) | `(Array,a)` noretain read、`force_unique` COW。-> `Storage::_unsafe_get_unretained` へ |
| `make_byte_array_copy`(codegen helper) | `Array U8` 確保 + memcpy。Storage へ(string literal 等が使う) |
| `_undefined_internal` | `Array U8` メッセージの C-str を print+abort。msg buffer -> `@_storage` |
| `_mutate_boxed_internal` / `get_funptr_*` | `is_array` 分岐が Array で unreachable に(Storage 経由へ) |
| object.rs layout codegen | `ty_to_object_ty`(Array arm -> unbox struct)、`create_obj`、`size_of`、`build_traverse`(Array/punched arm を **value `_size` 駆動**へ)、debug-info(`<array size>`/`<array buffer>`) |

**削除:**

| 名前 | 理由 / 置換 |
| --- | --- |
| `_unsafe_force_unique` | CSE 脆弱で危険(§3.3)。置換: mutator 内蔵 COW + `_unsafe_is_storage_unique` |
| `_unsafe_set_bounds_uniqueness_unchecked_unreleased` | uniqueness-check-less mutate 撤廃。置換: `Storage::_unsafe_initialize` |
| `_unsafe_punch/plug_bounds_uniqueness_unchecked`(no-COW 版) | COW 版に一本化(§3.3。`force_unique` param が消える) |
| `_unsafe_fill_size_unchecked`(`array_unsafe_fill`) | `fill` を Fix-src 化(実測で InlineLLVM と同値) |
| `@size` / `@capacity`(`InlineLLVMArrayGetSize/Capacity Body`) | value field 読みへ(Fix-src wrapper `arr.@_size`/`@_cap`) |
| `_get_ptr`(既に deprecated) | `borrow_boxed` / `borrow_elements` へ |

**改名:**

| old -> new | 変更 |
| --- | --- |
| `_unsafe_set_size` -> `_unsafe_grow_size` | 増加専用(`n >= _size`)+ 内部 COW。新スロット `[old_size..n)` は未初期化 |
| `_pop_back_nonempty` -> `_unsafe_pop_back_nonempty` | `_unsafe_` 付与(empty で UB)。唯一の size 減少経路(要素 release + `_size -= 1`) |

**不変で再利用:** `ObjectFieldType` の buffer helpers(`read/write/clone/release_or_mark_array_buf` 等 — buffer と size を
引数で受け layout 非依存)、`_check_range`/`_check_size`、`make_struct_union_unique`、`with_retained`、retained-ptr 系。
caller が buffer を `@_storage` gep で、size を value `_size` で得るよう変わるだけ。

### 13.2 std.fix Fix 関数・trait instance(public シグネチャは特記以外すべて不変)

**追加(Fix-source):**

| 名前 | 契約 |
| --- | --- |
| `Array::borrow_elements : (Ptr -> b) -> Array a -> b` | 要素先頭 Ptr を callback に借用。**Fix-source** `arr.@_storage.borrow_boxed(f)`。`array.borrow_boxed` の後継。ポインタは callback 中のみ有効・書き換え不可。`@_storage` retain は無害な RC churn(clone なし、§13.3-2) |
| `Array::borrow_elements_io` | IO 版(Fix-source) |
| `Array::mutate_elements` | Ptr 経由 in-place mutate。**専用 InlineLLVM**(naive な `@_storage.mutate_boxed` 委譲は常に clone するため。`set` と同じくその場で COW -> data ptr -> act -> 再構築、§13.3-2) |
| `Array::mutate_elements_io` | IO 版(専用 InlineLLVM) |
| Array 用 uniqueness assert(名前 TBD) | `_unsafe_is_storage_unique` ベース。`arr.assert_unique` の後継 |

**変更:**

- builder(`_unsafe_force_unique` 撤去 + `_unsafe_set_size`->`_unsafe_grow_size` + unreleased write -> `Storage::_unsafe_initialize`): `append`, `from_map`, `reserve`, `push_back`, `resize`
- `fill`: 削除プリミティブ -> **Fix-source**(`Storage::_unsafe_initialize` ループ)
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
- trait instance: **`impl Array a : Boxed`**(-> Storage a へ)。**user-visible break**: `array.borrow_boxed` / `array.boxed_to_retained_ptr` が型エラー -> `borrow_elements` か自作 boxed 型でラップ
- `unsafe_is_unique` の unbox 枝(const-true)が `[a:Boxed]` 追加で dead

**改名(呼び出し側更新):**

- `_unsafe_set_size` -> `_unsafe_grow_size`(grow 経路)。呼び出し: `append`/`from_map`/`push_back`/`reserve`/`resize`/`read_n_bytes`/`unsafe_from_c_str_ptr`(`_io`)/数値 `to_bytes` 一式。**真の shrink を行う `String::from_bytes` と `_unsafe_from_c_str` の切り詰め経路は効率化した `truncate`(safe shrink、§13.3-1)を使う**
- `_pop_back_nonempty` -> `_unsafe_pop_back_nonempty`。呼び出し: `pop_back`

**不変(変更された callee を通すだけ):** `@`, `get_first`/`get_last`, `is_empty`, `find_by`, `get_sub`, `dedup`, `empty`,
`act`, `from_iter`/`to_iter`, sort 内部一式(`_introsort`/`_heap*`/`_insertion*`/`_mergesort*`/`sort`/`sort_stable*`)、
全 Array trait impl(`Zero`/`Add`/`Eq`/`LessThan`/`Functor`/`Monad`/`ToString`/`Indexable`)、FFI 定義
(`mutate_boxed`/`borrow_boxed`/retained-ptr — Array を受けなくなるだけ)、`Destructor::mutate_unique_io`(box なので `[a:Boxed]` OK)、
String の大半、PunchedArray 型(新レイアウトを継承、punch/plug/traverse の Rust body だけ retarget)。

### 13.3 要検討(survey が surface した設計ギャップ)

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
2. **`mutate_elements` は専用 in-place InlineLLVM(決定)。`borrow_elements` は Fix-source のままで可。**
   `arr.@_storage` は `_storage` を retain するので、naive な `@_storage.mutate_boxed(f)` は refcount >= 2 を見て
   **常に clone**する(§3.3 の `unsafe_is_unique` と同じ hazard、O(n) コピー)。よって `mutate_elements` /
   `mutate_elements_io` は **専用 InlineLLVM body** にする: `set` と同じく arr の `_storage` をその場で
   `make_array_unique`(arr が実際に shared のときだけ COW)-> Storage の data ポインタ取得 -> act -> Array 再構築。
   一方 **`borrow_elements` / `_io` は Fix-source `arr.@_storage.borrow_boxed(f)` のままで正しい**: `borrow_boxed` は
   **clone せず**(`with_retained` で callback の間だけ retain してポインタを渡すだけ)なので、`@_storage` の retain は
   **無害な RC churn**(retain/release が1組増えるだけ)にとどまり、返るポインタは arr の要素バッファを正しく指し
   callback 中は生存する。この churn は borrow 化で消せるが正しさには無関係。
