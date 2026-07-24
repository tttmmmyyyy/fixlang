# Array representation investigation (raw material for the Buffer/Array redesign)

Goal of the redesign: today `Array a` is a boxed primitive laid out as ONE heap
allocation `{ ControlBlock(refcnt,state), len:i64, cap:i64, buf:[T..] }`, so
`get_size` is a heap load that the back end cannot hoist across a write loop. Plan:
introduce a boxed primitive `Buffer a` (refcounted raw element storage) and redefine
`Array a = unbox struct { _buf : Buffer a, _size : I64, _cap : I64 }`, moving `_size`
(and maybe `_cap`) into the VALUE so `get_size` becomes an `extractvalue` (register),
and BCE/vectorization in write loops falls out of standard LLVM LICM/SCEV.

Investigated by three subagents (2026-07-18) reading `src/object.rs`, `src/constants.rs`,
`src/fixstd/builtin.rs`, `src/fixstd/stdlib.rs`, `src/fixstd/std.fix`, `src/generator.rs`,
`src/ast/types.rs`. All paths under the `bce` worktree.

## 1. Current layout (constants.rs)

- `CONTROL_BLOCK_IDX = 0`; `BOXED_TYPE_DATA_IDX = 1`
- `ARRAY_LEN_IDX = 1`, `ARRAY_CAP_IDX = 2`, `ARRAY_BUF_IDX = 3` (constants.rs:112-114)
- ControlBlock = LLVM `{ i32 refcnt (CTRL_BLK_REFCNT_IDX=0), i8 refcnt_state (=1) }`
  (refcnt_state: LOCAL=0/THREADED=1/GLOBAL=2)
- `DEBUG_ARRAY_ASSUMED_LEN = 100`
- Concrete LLVM struct: `{ {i32,i8} ctrl, i64 len, i64 cap, <elem> }` where the trailing
  `<elem>` is the flexible-array-member (FAM) base; real allocation is `offset_of(buf) +
  elem_size*cap`. A boxed Array value is a POINTER to this struct.

## 2. How the type is built (object.rs)

- `ty_to_object_ty` Array arm (object.rs:1372-1383): asserts `!is_unbox`; field list
  `[ControlBlock, I64 len, Array(elem)]`; `assert_eq!(len_pushed, ARRAY_LEN_IDX)` and
  `ARRAY_CAP_IDX` pin the indices.
- `to_struct_type` FAM trick (object.rs:1057-1095): `ObjectFieldType::Array` contributes
  TWO llvm fields — the i64 capacity (ARRAY_CAP_IDX) and one embedded element base
  (ARRAY_BUF_IDX). `assert!(i == len-1)` (Array must be last) + `assert!(!is_unbox)`.
- `size_of` (object.rs:1097-1146): array alloc = `offset_of(ARRAY_BUF_IDX) + elem*cap`.
- `to_embedded_type` (object.rs:1151-1162): `is_unbox` -> llvm struct by value; else `ptr`.
  This is where "boxed = single pointer" is decided. Flipping Array to unbox makes it a
  by-value multi-word struct -> ABI change for every fn taking/returning Array
  (`lambda_function_type`, `traverser_type`, retain/release signatures use
  `get_embedded_type`).
- `create_obj` (object.rs:1435-1565): `assert!(array_capacity.is_some() == is_array())`;
  malloc(size_of(struct,cap)); sets control block; stores capacity into ARRAY_CAP_IDX;
  DOES NOT set ARRAY_LEN_IDX — every array-producing builtin sets len afterward via
  `insert_field(ARRAY_LEN_IDX)` on the boxed pointer.

## 3. RC traversal (object.rs / generator.rs)

- retain is SHALLOW: only bumps the control-block refcnt; elements stay shared (COW).
- release/mark walk the buffer: `build_traverse` Array arm (object.rs:1758-1770) reads
  `size = extract_field(ARRAY_LEN_IDX)` (heap load), `buffer = ptr_to_field(ARRAY_BUF_IDX)`,
  then `release_or_mark_array_buf(size, buffer, elem_ty, work, None)` which loops
  `idx in [0,size)` and invokes each element's own release/mark.
- **DEEPEST COUPLING**: the buffer destructor learns the live element count from `len`,
  which lives in the SAME heap object as the buffer. If `_size` moves into the Array value,
  `Buffer`'s traverser has no length. Design must decide how Buffer knows its element count:
  (a) store a count in Buffer, (b) have Array's destructor drive the element loop treating
  Buffer as raw storage, or (c) pass len to a custom traversal (like the existing hole path
  `build_release_mark_nonnull_boxed_with`).
- Element-storage helpers (object.rs): `loop_over_array_buf` (332), `array_buf_after_hole`
  (423), `release_or_mark_array_range` (445), `release_or_mark_array_buf` (487),
  `initialize_array_buf_by_value` (508), `panic_if_out_of_range` (545),
  `panic_if_size_negative` (570), `read_from_array_buf_noretain` (597),
  `read_from_array_buf` (626, retains), `write_to_array_buf` (639, `release_old` flag),
  `clone_array_range` (675), `clone_array_buf` (717, optional hole). These are the
  candidates to move to `Buffer`.

## 4. Debug info (object.rs)

- `to_debug_type` Array arm (231-327): `<array buffer>` debug struct: capacity member at
  offset 0, elements member at offset 1 = DWARF array of DEBUG_ARRAY_ASSUMED_LEN (100).
- `ty_to_debug_struct_ty` (1803-1950): names ARRAY_LEN_IDX member `<array size>` (1892-1894);
  overrides struct size to cover the 100 claimed elements (1922-1931). Must be re-authored:
  an unbox Array value struct describing `_size`/`_cap` inline + a boxed Buffer debug type
  carrying the FAM/100 elements.

## 5. Primitive inventory (builtin.rs / stdlib.rs / std.fix)

Registration block: stdlib.rs:385-585 (+ is_unique 370, _get_boxed_ptr 656).
Every InlineLLVM body treats array as a boxed pointer: `array.gep_boxed(ARRAY_BUF_IDX)` for
buffer, `array.extract_field/insert_field(ARRAY_LEN_IDX|ARRAY_CAP_IDX)` for len/cap.

InlineLLVM primitives (Fix name -> struct @builtin.rs line -> what it touches):
- `@size` -> InlineLLVMArrayGetSizeBody @2964: `extract_field(ARRAY_LEN_IDX)` (THE target load)
- `@capacity` -> InlineLLVMArrayGetCapacityBody @3021: `extract_field(ARRAY_CAP_IDX)`
- `_check_range` @2779 / `_check_size` @2839: pure I64 guards (no array obj), gated by
  `config.runtime_check()`
- `_unsafe_get_bounds_unchecked` @1840: buf read, RETAINS elem
- `_unsafe_get_linear_bounds_unchecked_unretained` (+`_forceunique`) @1933: buf read, NO retain,
  threads array; forceunique clones-if-shared
- `set` -> InlineLLVMArraySetBody @2249: make_unique, bounds check, buf write, RELEASES old elem
- `swap` / `unsafe_swap_bounds_unchecked` -> InlineLLVMArraySwapBody @2366: make_unique, two
  reads+writes, no elem RC
- `_unsafe_set_bounds_uniqueness_unchecked_unreleased` @1752: buf write, NO clone/bounds/release
- `_unsafe_set_size` @2110: overwrite LEN only
- `_unsafe_empty_capacity_unchecked` @1672: alloc cap, len=0, buf uninit
- `_unsafe_fill_size_unchecked` @1581: alloc, fill all slots (retain per slot)
- `_unsafe_force_unique` @2718: clone-if-shared (reads refcnt)
- `_pop_back_nonempty` @2035: force-unique, release last, len-=1
- `_get_ptr` @2889 [DEPRECATED]: pointer to ARRAY_BUF_IDX
- array literal `[..]` -> InlineLLVMArrayLitBody @3219 (compiler-internal)
- PunchedArray: `_unsafe_punch_*` -> InlineLLVMArrayPunchBody @2505 (buf read noretain, hole);
  `_unsafe_plug_*` -> InlineLLVMPunchedArrayPlugBody @2612 (buf write into hole, no release)
- shared helper `make_array_unique` / `make_array_unique_with_hole` @2180/2186 (clone-if-shared,
  touches CAP/LEN/BUF); `make_byte_array_copy` @778 (String/FFI internal)
- `unsafe_is_unique` -> InlineLLVMIsUniqueFunctionBody @5149: reads control-block refcnt

Fix-level (std.fix) ops built on the above — split across the Buffer/Array boundary:
`@`(113)=check+unsafe_get; `mod`(167)=check+punch+plug; `act`/`_unsafe_act`(216/231)=
check+is_unique+punch/plug/get/set (functor-specialized `_identity`/`_const`/`_tuple2`
selected by src/optimization/optimize_act.rs); `append`(250), `from_map`(304), `push_back`(378),
`pop_back`/`truncate`(366/655), `reserve`(395), `resize`(671), `reverse`/`sort_by`/`_introsort`/
`_heap*`/`dedup`(410-724). No primitive for push_back/reserve/resize — all Fix-level.

Unsafe-family qualifier semantics: `bounds_unchecked`=drop bounds check;
`unretained`=don't retain the returned elem; `unreleased`=don't release the overwritten slot
(uninitialized target); `uniqueness_unchecked`=skip clone-if-shared (assume unique).
=> the user's requested Buffer primitives map to: `unsafe_get` (=get_bounds_unchecked,
retained or not), `unsafe_set` (=set releasing old), `unsafe_set_uninitialized`
(=_unsafe_set_..._unreleased, no release — for filling fresh capacity).

## 6. Type-system / Boxed / FFI break points

- Array registered at builtin.rs:202-218: kind `*->*`, `variant: TyConVariant::Array`,
  `is_unbox:false` (the primary "is boxed" fact), doc says "This is a boxed type."
- `is_box`/`is_unbox` (types.rs:1096-1102) read `toplevel_tycon_info().is_unbox`.
- **Hardcoded Boxed instance for Array**: stdlib.rs:253-262 (`Array a` + `#DynamicObject`
  as `builtin_boxed`). Flipping Array to unbox makes this a LYING instance -> must delete
  Array's Boxed instance and give it to `Buffer a` instead.
- FFI functions (all assert `is_box`): `Array::_get_ptr`(2889, buf ptr), `FFI::_get_boxed_ptr`
  (5657 -> get_data_pointer_from_boxed_value 5699: array->ARRAY_BUF_IDX, struct->DATA_IDX,
  union->DATA+UNION_DATA), `borrow_boxed`(std.fix:924 = with_retained + _get_boxed_ptr),
  `boxed_to_retained_ptr`(5287, returns BOX pointer = obj.value, not data),
  `boxed_from_retained_ptr`(5376), `mutate_boxed`(5744: make_unique + data ptr),
  `with_retained`(5068, no ptr exposure, generic — neutral).
  => Only `boxed_to_retained_ptr` hands C the box/control-block pointer; the rest hand the
  DATA (element buffer) pointer. `get_data_pointer_from_boxed_value`'s `if is_array` (5705)
  must route through `_buf`. `String = unbox struct { _data : Array U8 }` (std.fix:2691)
  chains: `_get_c_str`(2701), `_unsafe_from_c_str`(2709), `borrow_c_str`(2774) all rely on
  `Array U8 : Boxed` -> rewrite to go through `_buf : Buffer U8`. Numeric bytes conversions
  (std.fix:3694+) using mutate_boxed/borrow_boxed on Array U8 are affected.

## 7. PunchedArray (user's explicit concern)

- Def: `type PunchedArray a = unbox struct { _arr : Array a, _idx : I64 }` (std.fix:2636).
  Holds the ARRAY and the hole index; not the element.
- punch: array_punch @2580 + InlineLLVMArrayPunchBody @2505 — make_unique (if forceunique),
  `buf = gep_boxed(ARRAY_BUF_IDX)` (2528), read_noretain (move elem out, leave hole, len
  unchanged), build PunchedArray.
- plug: punched_array_plug @2687 + InlineLLVMPunchedArrayPlugBody @2611 — move_out `_arr`/`_idx`,
  make_array_unique_with_hole(Some(idx)), `buf = gep_boxed(ARRAY_BUF_IDX)` (2635), write into
  hole (no release of old).
- RC traversal of PunchedArray: build_traverse (object.rs:1710-1725) special-cases
  `is_punched_array()` — reads inner array's ARRAY_LEN_IDX (1718) + ARRAY_BUF_IDX (1720),
  `release_or_mark_array_buf(..., Some(idx))` skipping the hole, reusing the inner array's
  refcount. rc_ir/borrow.rs:471-498 treats a punched array as one indivisible RC unit.
- Redesign impact: punch/plug/make_array_unique_with_hole all bang on
  `gep_boxed(ARRAY_BUF_IDX)` + `extract_field(LEN/CAP)`. Must move to `_buf`(Buffer) + value
  `_size`/`_cap`. PunchedArray's own representation reconsidered: hole ownership tracking must
  move to Buffer granularity. (Separate generic struct-field punch mechanism `is_punched`
  — constants.rs:93-97 — is layout-independent, not this.)

## 8. All layout-constant sites (the concrete rewrite list)

object.rs: 247, 273 (debug offsets), 1131 (size_of), 1378, 1380 (ty_to_object_ty asserts),
1546 (create_obj cap), 1718, 1720 (punched traverse), 1759, 1760, 1761 (Array traverse),
1892 (debug `<array size>` name).
builtin.rs: 793,794 (make_byte_array_copy), 1598-1599 (fill), 1698 (empty), 1767 (unsafe_set),
1867 (unsafe_get), 1952 (linear get), 2048,2055,2058 (pop_back), 2122 (set_size),
2207,2216-2217,2219-2220 (make_unique_with_hole), 2275,2279 (set), 2396,2400 (swap),
2528 (punch), 2635 (plug), 2900 (get_ptr), 2968 (get_size), 3025 (get_capacity),
3231-3232 (array lit), 4941 (panic msg String buf), 5705 (get_data_pointer_from_boxed_value).
C runtime (runtime.c/runtime.rs): does NOT hardcode array layout — all layout coupling is in
Rust codegen.

## 9. Break-point summary (what the redesign must touch)

1. boxed-premise asserts (`!is_unbox` at object.rs:1078,1375; `is_box` asserts across the FFI
   bodies) break when Array is unbox.
2. hardcoded Boxed instance (stdlib.rs:255) becomes a lie -> move to Buffer; String C-interop
   chains (std.fix:2701,2709,2774) follow.
3. `ty_to_object_ty` Array arm + the ~40 layout-constant sites (section 8) re-map to
   `_buf`(Buffer) gep + value `_size`/`_cap` fields.
4. PunchedArray/punch/plug + hole-skipping RC traversal (object.rs:1710, borrow.rs:483)
   move to Buffer-granularity hole management.
5. RC-traversal element count: Buffer needs to know its element count once len leaves the
   heap object (section 3, deepest coupling).
6. ABI: Array becomes multi-word by-value (section 2, to_embedded_type) — wide ripple;
   perf trade-off (measure; BCE win should dominate; keeping `_cap` in Buffer keeps Array 2 words).
