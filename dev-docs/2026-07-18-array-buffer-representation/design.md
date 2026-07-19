# Array/Buffer 表現の再設計 — 設計

ステータス: 設計のみ、未実装。`investigation-notes.md`(コード実地調査の生データ)を土台とする。
目的は `Array::@size` を register 読み出しにして、back end が write loop からそれを巻き上げ
(hoist)、要素ごとの bounds check を畳み、vectorize できるようにすること — bounds-check
elimination の write-loop 側(`../2026-07-18-bounds-check-elim/`)。read-loop 側(iterator の
終了条件変更 + RC-IR simplifier)は既に出荷済みで、その doc が本件へ先送りした部分にあたる。

## 0. 概要

**やること**: `Array a` を boxed primitive から、boxed `Buffer a`(refcount + 生要素だけ)を内包する
**unbox struct `{ _buf, _size, _cap }`** に変える。狙いは `@size`/`@capacity` が **register 読み出し**に
なること — write loop の bounds check と `push_back` の容量チェックが hoist/畳まれて vectorize する
(write-loop BCE)。read-loop BCE は既に別途出荷済み。

**確定した設計判断**(詳細は各節):
- `_size`/`_cap` は value(3 word、C++ `std::vector` 流)、refcount だけ Buffer(§2.1)
- 要素解放は Array の custom traverser が value の `_size` で駆動、Buffer は free のみ(§3, §3.1)
- RC-unit は PunchedArray と同じ「不可分 unit」扱い、名前付き述語で寄せる(§3.3)
- Array の uniqueness は専用 `Array::unsafe_is_buffer_unique`(`_buf` を retain せず覗く)。generic
  `unsafe_is_unique` は触らない(§3.4)
- bulk op(fill 等)は Fix-source(最適化器が InlineLLVM に並ぶことを実証、§4)
- FFI ポインタ系は `Buffer` 経由。retained-ptr は `_size`/`_cap` を運べない(§7)
- 事前手動 unique を要する unsafe primitive を safe 版へ寄せて縮小(§3.4/§11.4)

**進め方**: tests green を保つ5段階の移行(§9)。実装は設計確定後。

**やらないこと(先送り)**: zero-copy slice(§3.2)、`Buffer` の public 化(§11.3)。

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

boxed primitive `Buffer a`(refcount 付きの生の要素 storage)を導入し、`Array` を「長さと容量を value に
持つ unbox struct」として再定義する:

```
Buffer a : boxed primitive   // { ControlBlock, buf[FAM] } — refcount と生要素のみ、メタ情報は持たない
type Array a = unbox struct { _buf : Buffer a, _size : I64, _cap : I64 }
```

すると `get_size = arr.@_size = extractvalue` も `get_capacity = arr.@_cap` も **register 読み出し**。
write loop に通すと `_size` は loop 越しに運ばれる scalar になり、`set` は同じ `_size` を持つ新しい
`Array` value を返すので、LLVM は bound を loop-invariant と見なして `i < _size` を畳み、vectorize する。
同様に `push_back` ループの容量チェック `_size < _cap` も register で回る(§2.1)。size-normalization
パスも invariant-parameter の引き回しも要らず、標準の LICM/SCEV から自然に出てくる。

`Array` は 3 word の by-value aggregate `{ ptr, i64, i64 }` になる。`Array` の retain/release は既存の
unbox struct 機構を通じて *reference count* を `_buf` へ伝播する。ただし *要素* の寿命だけは custom な
一手が要る(§3)。C++ `std::vector` と同じレイアウト思想 — heap は生要素、メタ情報は value — だが、
COW 共有のため refcount だけは buffer 側に持つ点が異なる。

### 2.1 `_cap` をどこに置くか — 判断: value 側(C++ `std::vector` 流)

`push_back` は毎回 `@capacity` を読む(`if arr.@capacity < len + 1 { reserve } else { arr }` の容量
チェック)。よって配列を逐次構築する `push_back` ループでは `_cap` も **hot path**。ここが判断を決める:

- `_cap` を `Buffer`(heap)に置くと、`push_back` ループの容量チェックは **heap load** になり、直後の
  要素 store が同じ Buffer allocation に書くため **FAM-alias** でその load を hoist できず、毎回再 load
  になる — `_size` で直したのと同じ病気を `_cap` で再発させる。本再設計の趣旨(hot loop から metadata の
  heap-load を消す)と一貫しない。
- `_cap` を value に置くと(`{ _buf, _size, _cap }`、Buffer は `{ ControlBlock, buf }`)、容量チェック
  `_size < _cap` は **register** で回り hoist 可能。`push_back` ループが tight になる。これが C++ `std::vector`
  が `{ ptr, size, cap }` を value に持つ理由。**採用。**

read-index ループ(本件の主目的)では `_cap` は loop 状態に乗るが未使用なので LLVM が DCE で落とす —
3 word 目は read loop ではタダ、`push_back` ループでは効く。value に置くのは `_size` と `_cap` の2つだけ
(それぞれ bounds check と容量チェックで hot に読まれる)。それ以外のメタ情報は無い。

代償: `Array` が by-value 3 word になる — 配列を渡す関数の ABI がやや太り、`Array (Array a)` のような
入れ子では要素あたり 1 word 増える(メタ情報のメモリ)。C++ vector も 3 word なので許容範囲とする。

## 3. 要素の寿命 — 中核をなす判断

現状、`Array` の destructor は「何要素を release すべきか」を `len` から知る。`len` は buffer と同じ heap
object にある(`build_traverse` の `Array` arm: `size = extract_field(ARRAY_LEN_IDX)`、続けて
`release_or_mark_array_buf(size, buf, ..)`)。`_size` が value に移ると、`Buffer` 単体では live 要素数を
知れない。

**採用: `Array` value が要素 release を駆動する。** `Array` の release/mark/clone を custom traverser にし、
その各点で手元にある value の `_size` を使って `_buf` の生 storage を歩く(`Buffer` は untyped storage として
扱う)。`_size` を value だけに保てて(再設計の眼目)、しかも *現行* `Array` destructor が既に回しているロジック
そのもの — 変わるのは `size` の出所が heap load から value field になる点だけ。(検討して退けた代替案は §12。)
具体的には:

- **retain** `Array` = `_buf` の control-block refcount を +1(shallow、変更なし。COW で要素は共有のまま)。
  これは通常の boxed-field 伝播なので、retain については「boxed field を1つ持つただの unbox struct」で正しい。
- **release** `Array` = `_buf` が unique なら `for i in 0.._size { release(buf[i]) }` して `_buf` を free、
  そうでなければ `_buf` の refcount を -1。`_size` は value から来る。`Array` は custom な `build_traverse`
  arm を保つ(現行 `Array` も `PunchedArray` も既に custom なので hack ではない) — value を読むように
  要素数駆動の traversal を移すだけ。
- **clone-if-shared**(refcount >= 2 での mutate 時の COW)は `_size` 個の要素を新しい `Buffer` へ複製し、
  各要素を retain する。clone 側も `_size` を手元に持つ。
- `Buffer` 自身の boxed destructor は **生メモリを free するだけ** — 要素 release は決してしない。所有側の
  `Array` が `_size` で駆動して既に release 済みだから。`Buffer` は internal primitive で、ユーザコードは
  `Array a` を持ち、裸の `Buffer a` は持たない。よって裸の `Buffer` が自前で要素数を必要とする場面はない。
- free は `free(ptr)`(現行の `build_free` と同じ、サイズ不要)なので `Buffer` は cap を持たなくてよい。
  `_cap` を使うのは allocation 時(malloc バイト数 = `offset_of(buf) + elem*_cap`)と `push_back` の容量
  チェックだけで、どちらも value の `_cap` で足りる。

### 3.1 共有の不変条件(なぜ (b) が heap count なしで健全か)

共有は `retain` からのみ生じ、`retain` は `Array` value 全体を複製する — よって同じ `Buffer` を共有する者は
全員 **同じ** `_size` を持つ。refcount >= 2 での最初の mutate が `Buffer` を clone(COW)し、その共有者は
自分専用の `Buffer` を得る。共有中に `_size` が食い違うことはない。したがって最後の release(`_buf` の
refcount 1 -> 0)は常に正しい `[0.._size)` に対して要素 release を駆動する。これが成り立つのは、コア設計に
**zero-copy slice がない**からこそである。

**この不変条件は「`_size` を変える op はすべて unique な `_buf` にだけ適用される」ことに依存する。**
redesign では `_size` が value にあるので、**共有された `_buf` に size を書くと、その holder だけ `_size` が
食い違い §3.2 の leak を生む**(現状の heap len なら共有 len を書き換える別の誤り。redesign の方が危険)。
これを op 自身で保証する:

- `_unsafe_set_size` は実際 **増加専用**にしか使われていない(from_map/fill/reserve/push_back/append は
  alloc/reserve 後に size を伸ばす)。**減少は `_pop_back_nonempty` が担い、要素を release してから len を
  直接書く**(`_unsafe_set_size` を使わない。truncate は pop_back のループ)。
- そこで **`_unsafe_set_size` を「未初期化スロットへ size を伸ばす」増加専用 op にし、内部で unique check
  (= COW、shared なら clone)する**。すると「呼び出し側が事前に unique を保証する」footgun が消え(op 自身が
  unique な `_buf` にしか size を書かない)、その内部 check は unique-check-elim が provably-unique で畳んで
  同性能にする(§11.4 の方向)。呼び出し側の `_unsafe_force_unique` は畳み込める。残る unsafe 契約は「新スロット
  `[old_size..n)` は未初期化(呼び出し側が埋める)」「`n <= _cap`」のみ。

### 3.2 Zero-copy slice — 先送り

`_size` を value に置くと、共有 `_buf` + より小さい `_size` の slice が「作れてしまう」。これは §3.1 を壊す:
小さい `_size` の最後の保持者が、自分の view の外の要素を leak する。これを支えるには、`Buffer` に destruct
用の真の「構築済み要素数」を view の `_size` とは別に持たせる必要があり — 本件のスコープ外の実拡張になる。
コア再設計は `_size` == `Buffer` の構築済み要素数 を保ち、slice は将来課題とする。

### 3.3 RC-unit 機構との整合(PunchedArray と同じ特別扱い)

RC 挿入は値を **RC unit(boxed leaf)単位**に分解して retain/release を置く(`borrow.rs` の `rc_units_go` /
`clamp_unit`)。ただし `is_box` / `is_union` / `is_punched_array` は「1つの不可分 unit」として扱い、中へ
descend しない — その unit の retain/release は値全体の(custom)traverser 経由になる。現行の `Array` は
`is_box` なので自然にこの1 unit で、release すると Array の custom destructor が走る。

再設計後の `Array` は **unbox** なので、何もしないと generic な「field へ descend」枝に落ち、`_buf`(Buffer)を
boxed leaf として**単独で RC** してしまう -> Buffer の free-only destructor が走り要素を leak する。これが §3 の
coupling の機構的な正体。

**解決 = 新 `Array` を上記の不可分 unit 境界に加える**(`is_box || is_union || is_punched_array || is_array()`
相当)。追加先は `rc_units_go`、`clamp_unit`、`codegen::project_rc_unit`(全体 `{_buf,_size,_cap}` を projection
して custom traverser が `_size` を読めるようにする)、`provenance::build_shape`。こうすると `Array` は path `[]`
の1 unit になり、retain/release/mark がすべて Array の custom traverser 経由(value の `_size` 駆動)になって、
`_buf` が単独で RC されることはない。**これは `PunchedArray` が既に取っている扱いそのもの**で、
unique-check-elim / borrow / provenance / codegen は「custom traverser 型を1 unit として扱う」機構を既に持つ。

uniqueness(`set` の make_unique)は「`Array` unit = その `_buf` の refcount が unique か」で判定でき、provenance が
追う `_buf` leaf を `clamp_unit` が `Array` unit に丸めて突き合わせる(現行の union/is_box と同じ経路)。よって
per-unit の retain/release とも uniqueness 判定とも噛み合う。`PunchedArray` 自身は、custom traverser が読む値が
「内側 array の heap `len`」から「内側 `Array` の value `_size`」に変わるだけで、依然1 unit・hole skip のまま。

なお「不可分 unit 境界」の判定は現状 `is_box`/`is_union`/`is_punched_array` の disjunction が各パスに散在している
(しかも `clamp_unit` は `is_punched_array` を含まないなど不揃い)。`is_array()` を各所へ足して回る shotgun surgery
を避け、**「custom traverser を持つ不可分 RC unit か」を表す名前付き述語を1つ導入して既存の判定を寄せる**方針とする
(実装時に `clamp_unit` の不揃いが本質か latent bug かを見極めてから統一する)。

### 3.4 `unsafe_is_unique` は Array の unbox 化で壊れる — 要修正

`unsafe_is_unique`(`InlineLLVMIsUniqueFunctionBody`)は現状 `if obj.is_box() { refcount を読む } else
{ const true }`(「unboxed object は常に unique」)。**Array が unbox になると else 枝に落ち、`_buf` が共有
(refcount >= 2)でも無条件に `true` を返す。** すると `mod`/`act`/COW の `if is_unique { in-place } else
{ clone }` が常に in-place を選び、**共有 buffer を破壊(データ破損)**する。redesign の重大な破損点。

修正(generic `unsafe_is_unique` に Array 特別扱いを入れない): **`Array` 専用のビルトイン
`Array::unsafe_is_buffer_unique : Array a -> (Bool, Array a)` を追加**し、value の `_buf` の refcount を
**retain せずにその場で覗く**。`act` など Array の COW 判定はこれを使う。generic `unsafe_is_unique` は
現状のまま触らない — boxed 型(`Destructor`〔mutate_unique_io〕、FFI の gmp/mpfr 等、generic `assert_unique`)に
`is_box` で効き続けるので **不要にはならない**。unbox な Array *value* に generic を呼ぶと `const true`
(値としては常に unique)を返すが、COW 判定は Array 専用版を使うので破壊は起きない。

- `arr.@_buf.unsafe_is_unique` と書く案は不可: `@_buf` が `_buf` を **retain する**ので、borrow 化されない
  限り refcount >= 2 になり大抵 false を返す。専用ビルトインなら Array value を受けて `_buf` を extract せずに
  refcount を読めるので確実。
- unique-check-elim の static fold は Array 専用版に適用(`_buf` が provably-unique なら const-`true` に畳む)。
  ランタイム版(`InlineLLVMIsUniqueFunctionBody` 相当を Array 用に)と fold の両方を用意する。
- さらに **generic `unsafe_is_unique` に `[a : Boxed]` 制約を付ける**(現状は無制約で unboxed に `const true`
  を返す)。こうすると Array を unbox にした瞬間 `arr.unsafe_is_unique` が **型エラー**になり、silent な
  誤 const-true を型システムが弾いて `Array::unsafe_is_buffer_unique` へ誘導できる。`else { const true }` 枝は
  dead になり除去可。現状 unboxed 型に呼んでいる箇所は無い(`assert_unique` も Array=boxed にしか使われて
  いない)ので今は無害で、intended な破壊は redesign で Array が unbox になる時だけ。波及: `assert_unique :
  Lazy String -> a -> a` も `unsafe_is_unique` を呼ぶので `[a : Boxed]` になり、redesign 後 `arr.assert_unique`
  は型エラー(本来 arr には誤答なので望ましい破壊)。Array 用の uniqueness assert(`unsafe_is_buffer_unique`
  ベース)を別途用意する。

is_unique 分岐(`build_branch_by_is_unique`、**Rust/コンパイラ側のコード**)の用途は3つ: (1) **COW mutate**
(`make_array_unique_with_hole`〔set/mod/swap/punch〕、`make_struct_union_unique`)、(2) **release の
free-or-decrement**(`build_release_mark_nonnull_boxed_with`)、(3) **`is_unique` 関数**。これらは削除対象では
なく **COW/release の機構そのもの**で、unique-check-elim が (1)(3) を **コンパイル時に畳んで消す**((2) は基本
判定で残す)。redesign では (1) が Array の `_buf` に移るだけ。

**削除したいのは Fix レベルの「呼び出し側が事前に手動 unique を保証する」primitive**:
`_unsafe_set_bounds_uniqueness_unchecked_unreleased` と、その前に置く `_unsafe_force_unique`(std の
from_map/push_back/reserve/append/resize が使う)。これらは (1) の COW check を **skip する**ために存在した。
unique-check-elim が safe 版の check を確実に畳む今、std builder を **safe な Array op(COW 内蔵 =
`make_array_unique`)に置き換えれば削除できる**(§11.4、surviving unsafe RMW primitive 削除計画。上の
増加専用 set_size もこのパターンの一例)。

**例外: punch/plug ペア(`_unsafe_punch/plug_bounds_uniqueness_unchecked`)は残す。** `act` は `if unique`
枝で punch(所有権ごと要素を取り出し)し、書き戻しを functor の `map` の中の plug に置くことで、**action が
失敗(例: Option None)したとき書き戻しも clone も走らない**保証(`act` の doc)を実現している。plug を
check ありにするとこの no-clone-on-fail が壊れうるので、uniqueness-unchecked のまま残す。

注: `unreleased`(未初期化スロットへの書き込み)の unsafe さは uniqueness とは直交で残り、
`Buffer::_unsafe_initialize` 側に集約される。

## 4. `Buffer` primitive API

`Buffer` は、`Array` の std コードを組み直す土台となる boxed storage primitive。`_unsafe_` はメモリ安全性を
壊し得る op(bounds unchecked / unretained / unreleased / uniqueness unchecked、未初期化 storage の生成)に
だけ付ける — size/capacity の読み出しのような安全な op には付けない(それらは value の field 読み出し、
または public な `get_*` になる)。以下はいずれも現行の array primitive と同じ unsafe 契約を持つ:

- `Buffer::_unsafe_allocate : I64 -> Buffer a` — 指定要素数ぶんの領域を確保、未初期化、refcount 1。
  要素数は malloc サイズの計算に使うだけで `Buffer` には保存しない(`Array` の `_cap` が覚える)。
- `Buffer::_unsafe_get : I64 -> Buffer a -> a` — 要素読み出し(bounds unchecked)。read-fold 用に
  `_unretained` 版。
- `Buffer::_unsafe_set : I64 -> a -> Buffer a -> Buffer a` — 要素書き込み、旧占有者を **release** する
  (初期化済み slot 向け)。
- `Buffer::_unsafe_initialize : I64 -> a -> Buffer a -> Buffer a` — release **せず** に書き込む(live value を
  持たないスロットへの初回書き込み。fresh capacity を埋める用)。`_unsafe_set`(上書き + 旧要素 release)と対。
- FFI 用の data-pointer accessor(`Buffer` の生 storage 先頭ポインタ)。capacity は `Array` value の
  `_cap` が持つので、`Buffer` に capacity op は置かない。

**bulk op は Fix-source を基本とし、measurement で回帰するものだけ InlineLLVM を残す**。`fill` は Fix-source の
要素ループ(`Buffer::_unsafe_initialize`)で書けて、`from_map` ベンチで現行 InlineLLVM `fill` と同値と
実証済み(Ir 7,640,507 vs 7,640,521、差 0.0002%)— 最適化器がループをベクトル化して手書き LLVM に並ぶ。よって
`fill` は Fix-source にし、InlineLLVM の `array_unsafe_fill` は削除する。COW clone / reserve の buffer コピーは
retain-per-slot の別 buffer コピーで別パターンなので、実装時に個別に測って Fix-source 化するか判断する
(`clone_array_buf` を `Buffer` に retarget して残すのが安全側の初期値)。

これらは現行 `Array` の InlineLLVM body(`_unsafe_get_bounds_unchecked`、`_unsafe_set_..._unreleased`、
`_unsafe_empty_capacity_unchecked`、`create_obj`)と 1 対 1 に対応し、`Array` object の `ARRAY_BUF_IDX`
ではなく `Buffer` の生 storage(index 0)に対して動くように移す。bounds check(`_check_range`)は value の
`_size` に対する `Array` レベルの op として残す。

## 5. `Array` primitive の移行

現行の `Array` InlineLLVM primitive はそれぞれ、`{ _buf, _size, _cap }` + `Buffer` primitive の上の Fix
レベルコードになる。代表的な対応(完全な棚卸しは `investigation-notes.md` §5):

| 現行 `Array` primitive | 移行後 |
| --- | --- |
| `@size`(`extract_field(ARRAY_LEN_IDX)`) | `arr.@_size` — register 読み出し(目標) |
| `@capacity`(`extract_field(ARRAY_CAP_IDX)`) | `arr.@_cap` — register 読み出し |
| `_unsafe_get_bounds_unchecked` | `Buffer::_unsafe_get(i, arr.@_buf)` |
| `set`(make_unique, check, write, 旧要素 release) | `_buf` を unique 化, `_check_range(i, _size)`, `Buffer::_unsafe_set` |
| `_unsafe_set_..._unreleased` | `Buffer::_unsafe_initialize` |
| `_unsafe_set_size`(増加専用) | 内部で unique check(COW、optimizer 除去)+ value `_size` を伸ばす(新スロット未初期化)。減少は `_pop_back_nonempty` が release+shrink |
| `_unsafe_empty_capacity_unchecked(cap)` | `Array { _buf: Buffer::_unsafe_allocate(cap), _size: 0, _cap: cap }` |
| `_unsafe_fill_size_unchecked(n, x)` | Fix-source: `_buf(n)` 確保, `Buffer::_unsafe_initialize` の loop で埋め, `_size = n, _cap = n`(最適化器が InlineLLVM 同等にする、実証済み) |
| `_pop_back_nonempty` | unique 化, 最後を release して `Buffer::_unsafe_set`, `_size -= 1` |
| array literal `[..]` | `_buf` 確保, 埋め, `_size = len, _cap = len` |

`push_back` / `reserve` / `resize` は既に Fix レベル。これらは今後 value の `_cap` を読み(register)、伸ばす
ときに新しい `_buf` を確保して `_cap` を更新し、value の `_size` を設定する。

## 6. PunchedArray

`type PunchedArray a = unbox struct { _arr : Array a, _idx : I64 }` は既に `Array` を内包するので、新しい
`Array` レイアウトを継承する。変更点:

- `punch` / `plug`(`InlineLLVMArrayPunchBody` / `PunchedArrayPlugBody`)は現状 `gep_boxed(ARRAY_BUF_IDX)`
  で buffer を読むが、`_arr.@_buf`(`Buffer`)と value の `_size` へ移す。
- hole を飛ばす RC traversal(`build_traverse` の `is_punched_array` 特別扱い、`borrow.rs` の
  punched-array unit)は内側 array の `ARRAY_LEN_IDX`/`ARRAY_BUF_IDX` を読むが、内側 `Array` の value
  `_size` と `_buf` へ移し、hole index を除く `[0.._size)` を release する。これは §3 と同じ要素数駆動の
  traversal に index を1つ飛ばすだけ。
- hole の所有は `Buffer` 粒度に留まる: `punch` は `_buf` から要素を1つ move out して hole を残し `_size` は
  不変、`plug` は release せずに hole へ書き戻す。

## 7. FFI, `Boxed`, `String`

`Array` が boxed でなくなるのが FFI に効く。**FFI のポインタ系関数はすべて `[a : Boxed]` 制約**
(`_get_boxed_ptr`、`mutate_boxed`/`borrow_boxed`、`boxed_to_retained_ptr`/`boxed_from_retained_ptr`、
`with_retained`)なので、Array が `Boxed` を外れると **どれも Array を直接受け取れなくなり、`array.@_buf`
(= `Buffer`、これが Boxed)を通す**ことになる。

- **`Array a : Boxed` instance を削除し `Buffer a` に与える**(`stdlib.rs` のハードコード instance)。これで
  上記 FFI 関数はすべて `Buffer` を受ける。**ユーザー可視の破壊的変更**(Array に直接 FFI していたコードは
  `_buf` 経由へ書き換え)— 許容とする。
- **payload/data ポインタ**(`get_data_pointer_from_boxed_value` の `is_array` 分岐、`_get_boxed_ptr`、
  `mutate_boxed`、`borrow_boxed`): boxed 値の data への生ポインタ。`borrow_boxed : [a : Boxed] (Ptr -> b) -> a
  -> b` が公開 API で、FFI(String -> C 文字列、`Array U8` の to/from_bytes、ユーザーの subprocess/cairo/
  curl/gmp/mpfr 等)で多用される。移行後は `Buffer` の storage 先頭ポインタを返す(= 現状の要素 buffer と
  同じ番地)。`get_data_pointer_from_boxed_value` の `is_array` 分岐を `Buffer`(storage は data index)へ差し替え。
  - **`String` の公開 API(`_get_c_str`/`borrow_c_str`)は不変** — 内部で `s.@_data.@_buf` に経路変更するだけ。
    String FFI ユーザーは影響なし。std の byte-array FFI(to/from_bytes)も `bs.@_buf.borrow_boxed` に内部変更。
  - **ユーザーが Array に直接 `array.borrow_boxed(...)` している箇所は壊れる**(Array が Boxed でない)。
    緩和: `arr.@_buf.borrow_boxed(f)` へ委譲する **`Array` 用 FFI ヘルパを用意**すれば `array.borrow_boxed(f)`
    のユーザーコードも透過的に動く。ユーザー自作の boxed 構造体への borrow_boxed は不変(まだ Boxed)。
- **retained ポインタ**(`boxed_to_retained_ptr` / `boxed_from_retained_ptr`): retained pointer は **box
  (= Buffer)しか捕捉しない**。Array の `_size`/`_cap` は value にあって box に無いので、**Array を retained
  pointer に往復させると `_size`/`_cap` が失われる**(現状は Array 自体が boxed で len/cap も heap にあるため
  往復で保存される)。よって retained-ptr の往復は `Buffer` 単位(生 storage)に限られ、完全な Array を
  opaque pointer として保持したい FFI は `_size`/`_cap` を別途持ち運ぶ必要がある。これは新しい設計上の制約。
- **String**: `String = unbox struct { _data : Array U8 }` の C-interop chain(`_get_c_str`、
  `_unsafe_from_c_str`、`borrow_c_str`)は `Array U8 : Boxed` に依存するので `_data.@_buf`(`Buffer U8`)を
  通す。C 文字列ポインタ = `Buffer U8` の storage ポインタ。数値の `to_bytes`/`from_bytes`(Array U8 に
  `mutate_boxed`/`borrow_boxed`)も追随。
- FFI body 全体の `is_box`/`is_unbox` assert(Array を boxed と仮定していた箇所)は `Buffer` に適用される。

## 8. Debug info

`<array buffer>` debug 型と `<array size>` メンバ(`to_debug_type` / `ty_to_debug_struct_ty`)を書き直す:
`Array` は 3 field の value struct(`_buf` pointer、`_size` i64、`_cap` i64)になり、FAM/
`DEBUG_ARRAY_ASSUMED_LEN` の要素配列記述は boxed `Buffer` の debug 型へ移る。

## 9. 段階的移行(tests を green に保つ)

素朴にやると、この変更は ~40 の layout-constant 箇所と型/FFI/RC/debug 機構を一度に触る。commit 間で suite が
green を保つよう段階化する:

1. **`Buffer a` を導入**: boxed primitive として allocate/get/set/initialize/data-pointer の op と、
   自前の RC(free のみの destructor)を持たせる。まだ `Array` からは未使用。`Buffer` を直接 unit-test する。
   (未使用の dead-code 警告が「配線待ち」を示す。)
2. **`Array` の InlineLLVM body を `Buffer` 経由に付け替える**。ただし `Array` は *まだ boxed* のまま —
   つまり `Array` が内部に `Buffer` を持つが現行の object shape を保つ — として、ABI を反転する前に既存
   test で `Buffer` op を検証する。(この中間 shape が表現できなければ step 3 に畳む。)
3. **`Array` を `unbox struct { _buf, _size, _cap }` に反転**。`ty_to_object_ty`、`to_embedded_type`、
   `create_obj`、`size_of`、custom `build_traverse` arm(§3)、および layout-constant 箇所すべて
   (`investigation-notes.md` §8)を一斉に更新。`Array` の `Boxed` instance を削除して `Buffer` へ移し、
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

## 11. 未決事項

1. **Zero-copy slice**(§3.2) — 先送り。近い将来に不要と確認するか、そうでなければ `Buffer` の count モデルが
   変わる。
2. **step 2 の可否** — `Array` を boxed のまま `Buffer` を内包する中間段が作れるか、それとも反転は atomic で
   なければならない(step 2 を step 3 に畳む)か。
3. **`Buffer` の公開可否** — ここでは internal に留めた。public 型にすべきなら、裸の `Buffer` の要素寿命に
   item 1(slice)の count モデルが要る。
4. **事前 unique-check が要る unsafe 関数を減らす(方向)** — unique-check-elim があるので、`_uniqueness_unchecked`
   系や `_unsafe_force_unique` + `_unsafe_set_size` のような「呼び出し側が事前に unique を保証する」primitive を、
   自前で unique-check する safe 版に寄せる(unique-check-elim が provably-unique で畳んで同性能にする)。fill が
   Fix-source で InlineLLVM と同値だった実証と同じ方向で、この redesign を機に unsafe API の表面積を縮める。
   ただし §3.4(is_unique)と §3.1(unique-only な size 書き込み)が正しく効いていることが前提。
   surviving unsafe RMW primitive の削除計画と接続する。

## 12. あとがき: 検討して退けた案

**要素の寿命(§3)の代替案**:
- **(a) count を `Buffer` に持つ。** すると `get_size` が再び heap から読む — 本改修の意味が消える — ただし
  count を value に *複製* する場合を除く。複製すると size 変更のたび 2 箇所に書いて同期する必要が出る。却下。
- **(c) generic な custom-traversal ヘルパに `len` を渡す**(既存の hole path
  `build_release_mark_nonnull_boxed_with`)。採用案(Array value が release を駆動)の実装手段であって別モデル
  ではないので、独立の選択肢としては扱わない。
