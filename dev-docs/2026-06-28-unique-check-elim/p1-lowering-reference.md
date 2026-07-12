# P1 lowering implementation reference

Working notes for implementing P1 (RC IR + AST->RC IR lowering + codegen swap). The design is in
`plan.md` sections 1.1-1.7; this file records the concrete implementation state and the map of the
current code generator's implicit reference counting that the lowering must reproduce.

**STATUS UPDATE (2026-07-12): P1 is near the completion gate; parts of "## State" below are
superseded.** The key change: a struct/tuple `let`-destructure is NO LONGER lowered to per-field
projection getters + a "uniform retain-getter". It is lowered to a single
`RcExpr::Destructure(container, [(field_idx, var)], cont)` node that mirrors the old back end's
`get_scoped_obj` (retain-if-used-after) + `get_struct_fields` (boxed: retain fields + release
container; unbox: move fields out + drop the rest) EXACTLY. This restores array-iteration
instruction parity (per-field getters made it +63.5%) and removed the `StructProjectBody`
inline-LLVM op. `rc_insert::process_destructure` retains the container iff it is used after the
destructure and releases dead fields; codegen calls `ObjectFieldType::get_struct_fields`. So the
`[#F4]`/`[#R12-3]` "unbox getter = pure projection" and "uniform retain-getter" discussion below now
applies only to the closure-capture getter. NOTE: this DEVIATES from plan §1.2's "no dedicated
projection node" decision — justified because in P1's whole-value RC, decomposed getters cannot
express move-out (ownership splits across getters; that needs §2.1 per-leaf RC); the unit node is
the P1 bridge and the 2-pass split (lower + rc_insert) is confirmed the right design. Also this
session: `eval <global>` now materializes the global so its (possibly effectful) initializer runs;
the three RC-IR recursive passes (`lower_to_var`/`process`/`eval_rc_expr`) are guarded with
`stacker::maybe_grow`. Validated: test_basic 355/355 all opt, broad suite 875/1, cp-library
valgrind-clean on both paths at exact parity. Full status + TODO: memory `project-rc-ir-implementation`.

## State

**Phase A (structural lowering) DONE + validated (unstaged).** The AST->RC IR traversal
`src/rc_ir/lower.rs` produces the RC IR skeleton (ANF, fresh globally-unique names, all lambdas
lifted, `If`/destructure desugared to `Match`/getters, all-`Own`, NO explicit `Retain`/`Release`
nodes — only the retain baked into boxed getters). Validated by inspection via a `DUMP_RC_IR` env
hook (in `build/build_object_files.rs`, after `optimization::run`; value = module name or `all`) on
real programs exercising tuple construct/destructure, `If`->`Match` (Bool tag 1=then/0=else), union
`match` (correct variant tags + payload types), array literals, closures (both cap-based
`CaptureProject` and optimizer-decap'd `#CapList` funptrs), multi-capture (sorted order matches
projection index), FFI with `is_io` IOState threading, and LLVM-operand name rewriting (generator's
embedded names rewritten to fresh names, matching the operand list). `MatchArm.variant` changed to
`Option<usize>` for catch-all arms. `cargo build` clean; `test_ffi_call{,_ios,_io}` pass.
`lower::lower_program(type_env, &[Symbol]) -> RcProgram` lowers a symbol subset (references are
by-name, so the set need not be closed). Panics (believed-unreachable) on: union pattern in a `let`,
struct pattern as a top-level match arm, unbound LLVM operand — revisit if any fires.

**Phase B (RC insertion) DONE + validated by inspection (unstaged).** `src/rc_ir/rc_insert.rs`
(`insert_rc(prog, type_env)`) runs the backward last-use pass over each function and global
initializer, inserting whole-value `Retain`/`Release` (`RcState = Unknown`), skipping fully-unboxed
values. Operand ownership is `LLVMGenerator::borrows_operand(i)` (the ten read-getters borrow operand
0; everything else owns — from the §8 audit). Wired into the `DUMP_RC_IR` hook after `lower_program`.
Validated via `DUMP_RC_IR` on a program exercising: a doubly-used borrow getter (one release after the
last use, no retain), borrow getters released at last use (`@size`/`get_data_ptr`/`#project`
container/`#cap` container), a boxed/unbox `Match` (unbox union = no container release; payload
released after use), tuple destructure, capture projection + cap release, `with_retained` (owned
operand retained before a non-last use, internal RC kept baked), and unused fully-unboxed bindings
(no RC). `cargo build` clean.

Two decisions made in this slice (present for review; recorded in plan §8):
- **Uniform retain-getter for struct projection (`build_struct_project`).** The destructure getter now
  retains the extracted field for BOTH boxed and unbox containers, and the container is a `Borrow`
  operand whose whole-value `Release` runs at its last use. This keeps the container `Release` sound
  for either layout and makes partial patterns fall out for free (the container `Release` traverses
  and drops the omitted leaves; the extracted ones survive because retained), so **no separate
  partial-pattern handling is needed**. It *deviates* from the plan's "unbox getter = pure projection"
  ([#F4]/[#R12-3]): for an unbox container it adds a retain/release pair (per boxed leaf) that the
  a per-leaf pure-projection pass (the plan's original getter model) removes. Behavior and
  leak-freedom are unaffected; only RC count/perf on unbox destructure differs from the current
  generator. **This is the one known non-parity item introduced by Phase B, and it must be resolved
  before the P1 parity gate.** Note it is **not** removed by §2.2 cancellation: the extra retain is
  baked into `build_struct_project` (not an explicit node), and it would cancel against the container
  `Release(s)`'s *traversal* release of a *different* variable's leaf — cross-variable, which §2.2's
  same-variable cancellation does not cover. Closing it needs pure-projection getters (no baked
  retain) with per-leaf handling of the container's dropped/used-later leaves, or the §6 move-out
  rewrite.
- **Match scrutinee = owned, container release explicit.** `process_match` emits `Release(scrut)` at
  each arm head for a boxed union (the payload retain-getter stays baked in `Match` codegen, unit 3),
  nothing for an unbox union, and `Retain(scrut)` before the match iff the scrutinee is used after it.

**unit 3 (codegen from RC IR) CORE DONE + validated end-to-end (unstaged).** `src/rc_ir/codegen.rs`
(`Generator::generate_rc_program`) declares every `RcFunc` (funptrs also registered as global objects),
implements each body via `eval_rc_expr`, and builds closures/matches/RC nodes as designed below.
Gated behind the `USE_RC_IR` env var in `build_object_files` (parallel to the old `implement_symbol`
loop; first cut assumes a single compilation unit). Added `rc_ir_mode` to `Generator` (+ the
`is_var_used_later` override) and `RcFunc.fn_ty`; made `scope_push`/`scope_pop`/`build_retain` public.
**Validated:** at `FIX_MAX_OPT_LEVEL=max`, a program exercising a boxed-union-free `match`, tuple
destructure (the uniform retain-getter), a capturing closure, `@size` borrow getters, and multiple
uses builds through the new path and prints the **same output as the old path** (`116`), and is
**valgrind-clean (0 lost, 0 errors)**. A minimal funptr program also matches (`6`).

**VALIDATION CAVEAT: Fix caches object files under `.fixlang/` and `USE_RC_IR` is NOT in the cache
key** — a `USE_RC_IR=1` build after an old-path build serves the cached OLD binary. **`rm -rf .fixlang`
before EVERY A/B build.** (An early round of "matches" were cached old binaries; disregard any
validation not done with cache clearing.)

**DONE (validated clean at none/basic/max — output matches old path, valgrind 0 err/0 lost — on
sieve+global, trivial/closures/match/tuple/string/100k-deep-tail-recursion):**
- Global initializers: `implement_rc_global` (mirrors `implement_symbol`'s call-once + `GlobalVar#`/
  `InitFlag#` + init via `eval_rc_expr` + `mark_global` + store). A real global (e.g. a sieve table)
  survives max opt as a shared `GlobalVar#` (computed once). (`from_map(N,f).@(i)` fuses to `f(i)` so
  those never materialize — a good optimization, not recomputation.) Also fixed `lower_llvm` panicking
  on global LLVM operands (`Std::IO::stdout`) — non-local operands become global atoms via
  `Lowerer.symbol_types`; `lower_program(type_env, symbols_to_lower, all_symbols)`.
- Separated compilation: the `USE_RC_IR` path runs `declare_symbol` for ALL symbols in each unit, then
  lowers + implements only `unit.symbols()` (`implement_rc_program`: `module.get_function(name)` or
  `declare_rc_function` for lifted lambdas). No duplicate definitions.
- The UAF from `with_retained`: its transient retain is now unconditional in `rc_ir_mode`
  (`if gc.rc_ir_mode || !gc.is_var_used_later(x)`), so a boxed value stays alive while its aliasing
  raw `Ptr` (`get_data_ptr`) is used by an FFI. (The single `is_var_used_later=true` lever serves the
  borrow getters but is the wrong sign for `with_retained`'s retain guard.)

**Remaining for P1:**
1. The unbox-destructure per-leaf parity fix (deferred; the one known non-parity item — redundant but
   cancelling RC, not a correctness bug).
2. Deferred: `FixBody` tail position, Phase B's tail-call `Release` exception.
3. Full A/B validation: the whole `cargo test --release` suite at all opt levels, benchmarks,
   debug-info parity, external libraries. Then flip the default and delete the old path.

**unit 3 design (as built):**
- **`rc_ir_mode` gating is minimal — one field + one line.** In the RC IR path, never increment
  `used_later` (so `get_scoped_obj` reads plain — it retains iff the `ScopedValue.used_later` field
  `> 0`, which stays 0), and make `Generator::is_var_used_later` (generator.rs:748) return `true` when
  `rc_ir_mode`. That single override makes the 8 borrow getters (`if !is_var_used_later { release }`)
  skip their release (the explicit `Release` node disposes the container) and `with_retained` always
  retain (its two `is_var_used_later` guards both fire = the unconditional semantic retain/release the
  audit requires). The force-unique clone, retain-getter, and other baked-in RC are untouched. No
  other `is_var_used_later` callers exist (audit: exactly those 10 sites).
- **Function setup is simpler than `implement_lambda_function`.** For an `RcFunc`: push the builder /
  entry BB / debug scope / value scope, push each param (and the cap, if a closure) onto the scope as
  a plain `Object` from the LLVM parameter, then `eval_rc_expr(body, tail=true)`. Do NOT read back
  captures (they are `CaptureProjectBody` `Llvm` statements at the head of the body) and do NOT
  release unused args/cap (Phase B emitted those as `Release` nodes). The whole body is self-contained.
- **`RcFunc` needs its arrow/funptr type** to declare the LLVM function via `lambda_function_type(ty)`
  (object.rs:1259, which reads `get_lambda_srcs`/`is_closure`/`get_lambda_dst`). Add `fn_ty:
  Arc<TypeNode>` to `RcFunc` (= Phase A's `lam_ty`; a unit-1 refinement) — the params/ret_ty alone do
  not reconstruct a funptr-vs-closure arrow type. `App`'s callee `RcVar.ty` already is that arrow type,
  so `apply_lambda(callee_obj, arg_objs, tail)` (generator.rs:976) handles the funptr/closure ABI.
- **RcRhs -> codegen:** `Var(x)` = plain get; `App` = `apply_lambda`; `Closure(func, caps)` = replicate
  `eval_lam`'s closure build ({funptr@`CLOSURE_FUNPTR_IDX`, cap-obj@`CLOSURE_CAPTURE_IDX`}, caps
  written into a fresh dynamic object; null cap for empty); `Llvm(gen, args)` = `gen.generate()`
  unchanged (reads operands plain in `rc_ir_mode`); `Match` = tag switch + per-arm payload retain-getter
  (`get_union_value`) + phi (non-tail) mirroring `eval_match`, but driven by the `tail_of` lookahead.
- **RC nodes:** `Retain(x, [], _)` -> `build_retain(x_obj)`; `Release(x, [], _)` -> `self.release(x_obj)`;
  `Ret(x)` -> `build_tail(x_obj, true)` (build_return). First cut handles `path=[]` only; per-leaf
  `path` handling is added with the unbox-parity fix.
- **Globals:** `RcGlobalInit` -> the lazy-init accessor of `implement_symbol` (generator.rs:2788+, the
  call-once flag + `GlobalVar#`/`InitFlag#` globals) with the init computed by `eval_rc_expr` and
  `mark_global` applied before storing.
- **Driver + flag:** a new `src/rc_ir/codegen.rs` that declares every `RcFunc` (map name ->
  `FunctionValue`, register directly-referenceable funptrs as global objects) then implements each, and
  emits globals; gate the whole new path behind a config/env flag in `build_object_files` parallel to
  the old `implement_symbol` loop. Then A/B validation (all opt levels, benchmarks, debug-info parity,
  external libraries).

### Toolkit (also done): the RC IR construction/getter LLVM primitives + printer

- **Five new `LLVMGenerator` variants** added so lowering can express, as `RcRhs::Llvm(gen, vars)`,
  the operations that the current generator does *inline* (with no reusable generator). This
  realizes the plan's decision (§1.2/§8) to converge all construction to alloc-family `LLVM`
  primitives. The old inline `eval_*` paths are **left untouched** (parallel A/B rollout); the small
  transient duplication of the allocate+fill shell is by design and is deleted at the flip.
  - `InlineLLVMMakeStructBody { field_names }` — struct/tuple construction (`Expr::MakeStruct`).
  - `InlineLLVMArrayLitBody { elem_names }` — array-literal construction (`Expr::ArrayLit`).
  - `InlineLLVMFFICallBody { fun_name, ret_tycon, param_tycons, is_var_args, is_io, arg_names }` —
    C call (`Expr::FFICall`). When `is_io`, the last operand is the input `IOState` token: it is a
    free var (threads IO order) but is not passed to C. Delegates to the new
    `Generator::build_ffi_call_core` (extracted from `eval_ffi_call`; old path stays byte-identical:
    the return object is still created first, and the C-function declaration is module-level so no
    instruction reorders).
  - `InlineLLVMCaptureProjectBody { cap_name, cap_idx, cap_tys }` — retain-getter that reads capture
    `cap_idx` out of a lifted closure's capture object (mirrors `implement_lambda_function`'s
    read-back). `cap_tys` (all capture types) are needed to rebuild the capture struct layout.
    Delegates to the new `Generator::build_capture_project`.
  - `InlineLLVMStructProjectBody { var_name, field_idx }` — one struct/tuple field for pattern
    destructure. Retain-getter when the container is boxed, pure projection when unboxed; it does
    NOT release the container. Delegates to `Generator::build_struct_project` (uses
    `ObjectFieldType::move_out_struct_field`). Union-payload extraction needs no such op — it is
    intrinsic to `Match` codegen.
  - All five read operands via `get_scoped_obj_noretain` (RC IR contract: plain get; explicit
    `Retain`/`Release` handle duplication). Generators are wired into `LLVMGenerator`'s
    `generate`/`free_vars_mut`/`name`. Validated: `cargo build` clean; `test_ffi_call{,_ios,_io}`
    pass (the only old-path change is behavior-preserving).
- **RC IR pretty-printer** (`src/rc_ir/print.rs`): `program_to_string` renders funcs + globals with
  the continuation-nested body as statements; operands are names, bindings show `name : type`.

### Design decisions resolved while mapping the lowering (record; apply when writing `lower.rs`)

- **Getter RC model (plan [#R10-1]/§1.2).** Phase A emits: boxed-container field extraction =
  retain-getter (retain baked in the op, per [#R10-1] to avoid cancellation UAF); unboxed-container
  field extraction = pure projection. No explicit `Release`/`Retain` *nodes* in Phase A — the
  container release, dropped-leaf releases, and non-last-use retains are all added by Phase B.
- **Capture read-back = retain-getter baked in the op** (not a pure get + separate `Retain`), same
  [#R10-1] reasoning. `Release(cap)` is the Phase-B node.
- **Construction consumes its operands (move-in).** The alloc generators read atoms `noretain` and
  store them; a value used again elsewhere gets a Phase-B `Retain` before this use.
- **`If -> Match` on the Bool union.** Bool = `unbox union { _false, _true }`; tag 0 = `_false`,
  tag 1 = `_true`. `eval_if` branches true-tag -> then. So `If(c,t,e)` lowers to
  `Match(c, [_true(1) => t, _false(0) => e])`.
- **Catch-all match arms exist** (the current `eval_match` allows a non-variant `Var` arm as the
  last/default case). The RC IR `MatchArm.variant: usize` cannot express this. **Decision: change to
  `variant: Option<usize>`** (`Some(tag)` = variant arm with that payload; `None` = catch-all whose
  `payload` is the whole scrutinee). Codegen keeps "last arm = default" (mirrors `eval_match`'s
  switch). This is a unit-1 refinement to `ast.rs`, to be made when writing `lower.rs` and presented
  with that slice. (Do NOT desugar a catch-all into per-variant arms — that duplicates the arm body,
  which the plan's ANF explicitly avoids.)
- **Evaluation order** (from `eval_app`): the callee is lowered/evaluated first, then arguments
  left-to-right; ANF preserves this by appending the callee's statements before the args'.
- **Global vs local reference.** A `Var` whose name is in the fresh-name env is a local (reuse the
  mapped `RcVar`); otherwise it is a global reference (an atom `RcVar` carrying the global's
  absolute name, materialized by codegen). LLVM-generator operands are always locals.

## Unit 1 details

- **Unit 1 (RC IR data types) DONE**, unstaged, reviewed: `src/rc_ir/ast.rs` + `src/rc_ir/mod.rs` +
  `pub mod rc_ir;` in `lib.rs`. Compiles clean. Design decisions confirmed with the user:
  - Type + span live on `RcVar`; a node's value type is derived from its final `Ret` var (not
    stored on `RcExprNode`). `RcExprNode` carries `source` (span) only.
  - `RcVar.source` is kept (future hook for local-variable debug info; the current codegen does not
    use per-variable spans — `create_debug_local_variable` hardcodes line 0 / current debug
    location, an existing TODO).
  - `cap: Option<RcVar>` kept (uniform with `params: Vec<RcVar>`; the cap is a normal parameter and
    gets a fresh globally-unique name, not the fixed `#CAP`; the body references it via getters).
  - `Path = Vec<usize>` kept (indices, LLVM-native). A pretty-printer resolves index->field name
    from the value's type for readable visualization; names are NOT stored in the path.
- **Rollout decision: flag-gated parallel** (plan option 1). Build `AST -> RC IR -> codegen` behind
  a config/env flag; keep the current implicit-RC path as default until the new pipeline passes all
  gates (all `FIX_MAX_OPT_LEVEL`, benchmarks, debug-info parity via A/B), then flip the default and
  delete the old path + flag. Final code is identical to a big-bang replacement (scaffold removed).

## Debug-info generation points (span placement was verified against these)

- Per-instruction debug location: `eval_expr` does `push_debug_location(expr.source)` (generator.rs
  1662) / `pop` (1705); `create_debug_location` (1885). RC IR: driven by `RcExprNode.source`
  (per-statement; ANF gives one node per sub-expression, so lowering propagating AST spans
  reproduces the granularity). `RcRhs` needs no own span (covered by the enclosing `Let` node).
- Function subprogram: `create_debug_subprogram(fn_name, lam.source)` (1831). RC IR: `RcFunc.source`.
- Local variable: `create_debug_local_variable` (2117/2383/1973) uses line 0 / current debug
  location (TODO), so per-variable spans are unused by current debug gen.

## Map of the current code generator's implicit RC (the spec the lowering reproduces)

All line numbers in `src/generator.rs` unless noted (`object.rs` for container release / move-out).

### Overall contract
- Every `Object` carries one owned reference (`Own`). Producers hand ownership to consumers; the
  last owner must `release`.
- **Callee-consumes arguments**: a function/closure/builtin owns its arguments on entry
  (`implement_lambda_function` pushes params with `used_later == 0`) and must release any it does
  not forward; the caller (`eval_app`) transfers ownership into the call and emits no post-call
  release. Unused args + unused CAP are released at function entry (1982/1989).
- **Last-use = move, live-later = retain**: the `used_later` counter, incremented over the free
  vars of not-yet-evaluated sibling expressions, is the whole mechanism. `get_scoped_obj` (708-716)
  retains only when `used_later > 0`; else the variable is moved.
- GLOBAL objects are refcount-exempt (marked at symbol init via `mark_global`, 2836).

### used_later / last-use tracking
- `ScopedValue { accessor, used_later: u32 }` (91-95); `Scope.data: Map<FullName, Vec<ScopedValue>>`
  (375-378, shadowing stack per name; act on `.last()`).
- `modify/increment/decrement_used_later` (404-422), `is_used_later` (425-427).
- `scope_lock_as_used_later(names)` (730-736) / `scope_unlock_as_used_later` (739-745).
- `is_var_used_later` (748-753): globals always `true`; locals defer to the counter.
- Globals: `add_global_object` (644-667) sets boxed globals `used_later=0` (always moved out) and
  unboxed globals a huge sentinel (never consumed).

### Variable access / retain decision
- `get_scoped_obj_noretain` (702-704): materialize, never retain.
- `get_scoped_obj` (708-716): retain iff `used_later > 0` (build_retain at 713). THE retain point.
- `get_scoped_obj_field` (720-727): `get_scoped_obj` then `extract_field` (retains whole container).

### Release emission sites
- `eval_eval` (2075-2090): release the discarded left value (2087).
- `eval_let` (2093-2127): release each destructured sub-object unused in the body (2113).
- `eval_if` (2182-2279): release cond after extracting the i1 (2197); release dead-branch vars
  (then 2218-2226/2224, else 2243-2251/2249).
- `eval_match` (2281-2431): release vars used only in other cases (2364-2369/2367); release
  destructured sub-objects unused in the arm value (2379).
- `implement_lambda_function` (1903-1995): release CAP if unused (1982); release each unused arg
  (1987-1991/1989).
- Container release / move-out: `object.rs::get_struct_fields` (997-1034) releases the boxed struct
  container (1020) or the un-extracted unboxed fields (1028); `object.rs::get_union_value` (882-899)
  releases the boxed union container (894); for an unboxed union the retain/release cancel (nothing).

### RC primitives
- `build_retain` (1063-1172) inline; `retain` (1033-1060) out-of-line per-type.
- `build_release_mark` (1232-1284) master dispatch; `build_release_mark_nonnull_boxed(_with)`
  (1177-1229) handles `Std::Destructor` uniqueness branch; `build_release_boxed_with` (1319-1431)
  frees at refcnt 0; `release` (1460-1487) out-of-line per-type (what the sites call);
  `release_nonnull_boxed` (1490-1492).

### Per-expression RC (dispatch: `eval_expr` 1658-1712; `build_tail` 1715-1727)
- **Var** `eval_var` (1730-1733): `get_scoped_obj` (retain iff used later).
- **App** `eval_app` (1736-1762): lock all args' free vars before evaluating `fun` (1743-1745);
  evaluate `fun` (1748); per arg, unlock just before evaluating it (1753/1756) so the last consumer
  moves, earlier ones retain. `apply_lambda` (976-1019) consumes callee + all args (callee-consumes).
- **Let** `eval_let` (2093-2127): lock val's free vars (minus pattern-bound) while evaluating bound
  (2106); `destructure_object_by_pattern` (2108/2131-2179) moves out + releases container; push
  bound sub-objects or release unused (2111/2113).
- **Match** `eval_match` (2281-2431): lock case-used vars while evaluating scrutinee (2294-2296);
  `switch` on union tag (2337-2354); per case release other-case-only vars + destructure (releases
  scrutinee container per case, move-out payload); non-tail phi (2415-2430).
- **Lam** `eval_lam` (1998-2072): `calculate_captured_vars_of_lambda` (1776-1807, noretain);
  declare+implement lambda fn; allocate closure (funptr at CLOSURE_FUNPTR_IDX); if captures,
  allocate dynamic capture object and write each captured var via `get_scoped_obj` (2047, retains iff
  used later = closure owns one ref per capture); capture ptr (or null) at CLOSURE_CAPTURE_IDX.
- **LLVM builtin** `eval_llvm` (1765-1772) -> `LLVMGenerator::generate` (inline_llvm.rs 90-191); each
  body reads args via `get_scoped_obj` (usually `used_later==0` => moved), callee-consumes.
- **If** `eval_if` (2182-2279): still live; lock then/else vars while evaluating cond (2191-2195);
  release cond (2197); dead-branch releases; non-tail phi.
- **MakeStruct** (2434-2466) / **ArrayLit** (2564-2602): lock each field/elem's free vars, move each
  value into the struct/buffer (no per-field release). **FFICall** (2468-2562): lock/unlock arg free
  vars, pass extracted scalars, args consumed.

### Function-body entry (`implement_lambda_function` 1903-1995)
1. new builder + entry BB; new scope.
2. push each param owned (`used_later==0`, 1938-1944).
3. if closure, push CAP (1947-1955); for each capture, `extract_field_as` + `build_retain(cap_obj)`
   (1969) + push (each capture becomes an owned ref).
4. release unused CAP (1982) + unused args (1989).
5. `eval_expr(body, true)` in tail position (1994) -> `build_return` transfers the one owned return
   ref to the caller.

## Unit 2 plan (Phase A structural + Phase B RC insertion)

**Phase A - structural lowering (AST -> RcExpr, no explicit RC yet):**
- ANF: sibling positions become let-bound atoms; effectful subexprs become `let`.
- Fresh naming: name counter + `AST FullName -> RcVar` env, resolve shadowing.
- Lambda lift: every `Expr::Lam` -> a top-level `RcFunc`; compute captures; use sites become
  `Closure(func, captures)`. No nested lambdas remain. Capture order = capture-object slot order =
  the lifted function's projection order (keep consistent across all transforms).
- Desugar: `If` -> `Match` (Bool is an unbox union); pattern destructure in `Let`/`Match` -> getter
  (`Llvm`) sequences. boxed struct: retain-getter + `Release(container)`; unbox struct: pure
  projection getters + explicit `Release(s@path)` for dropped leaves (no container). Follow
  `object.rs::get_struct_fields`/`get_union_value` move-out minimization.
- Construction is `Llvm` too (no dedicated Construct node): MakeStruct/tuple/ArrayLit/union-variant
  -> alloc `Llvm` primitives.
- Preserve IOState threading as data dependency (order auto-preserved).

**Phase B - RC insertion (backward whole-function last-use pass) — detailed spec.**

*Goal & parity.* Insert explicit `Retain`/`Release` nodes into Phase A's skeleton so the RC
operations reproduce the current generator's implicit RC. The current generator is **whole-value**
granular: `build_retain`/`build_release` traverse all boxed leaves of an object, and `get_scoped_obj`
retains the whole variable iff used-later. So Phase B emits whole-value `Retain(x, [])`/`Release(x,
[])` (per-leaf paths come from later passes) and `RcState = Unknown` (sound). Precision (borrow-ify
§2.1, cancellation §2.2, provenance §3) is deferred; the P1 target is A/B parity.

*Operand ownership (Own vs Borrow).* Per boxed leaf; scalars carry no RC.
- **Own** (the op consumes it — moves into result / force-unique-returns / releases internally): `App`
  callee+args, `Closure` captures, `RcRhs::Var` (move), and most `Llvm` operands (construction
  `MakeStruct`/`ArrayLit`, `set`/`mod`/`swap`/`punch`/`plug`, FFI args, `DestructorMake`,
  `mark_threaded`, `unsafe_mutate_*`, `FixBody`). Rule: `Retain(v)` before a non-last use; at the last
  use the op consumes it (no release node).
- **Borrow** (the op reads without consuming — needs an explicit `Release` after the last use): the
  read-getters — array element get / array ptr / `@size` / `@capacity` / union `is` / retain·release
  fn-ptr / boxed data ptr, `StructProjectBody`(boxed container), `CaptureProjectBody`(cap),
  `_unsafe_get_ptr`/`_get_boxed_ptr`, the type-witness `get_{retain,release}_function_of_boxed_value`.
  Rule: no `Retain` (a borrow doesn't duplicate); `Release(v)` after its last use. `with_retained`
  keeps its cross-call retain/release **inside** the op (opaque; not exposed as nodes). Exact per-op
  list comes from the §8 audit (in progress).
- **Match scrutinee** = **Own**: `Retain(s)` before the match iff `s` is used after it; the match
  consumes `s` by releasing its container **per arm** — an explicit `Release(s)` at each arm head for a
  **boxed** union (mirrors `get_union_value` 891-894), nothing for an **unbox** union (payload-retain
  and container-release cancel). The payload retain-getter stays baked in `Match` codegen.

*Algorithm (backward, per function).* Walk the body backward maintaining `live` = local variables
referenced in the already-processed suffix (⇒ "used later"). Globals are refcount-exempt: excluded
from `live` and emit no RC (their runtime state makes retain/release no-ops — parity of the exact call
count is an open item, reconcile via A/B).
- `Ret(x)`: x consumed; `live := {x}`.
- `Let(x, rhs, k)`: process `k` → `live_k`. If `x ∉ live_k` (dead binding), emit `Release(x)` at the
  head of `k` (rule c). Then, for `rhs`'s operands (right-to-left, so repeated operands give k uses ⇒
  k−1 retains): an **Own** operand `v` live after its occurrence → `Retain(v)` before the stmt (rule
  a); a **Borrow** operand `v` not live after → `Release(v)` after the stmt (rule b). Then `live :=
  (live_k \ {x}) ∪ operands(rhs)`.
- `Let(x, Match(s, arms), k)`: process `k` → `live_k`; `L := live_k \ {x}`. For each arm: process the
  arm body (payload/subpattern vars bound) toward `L`; at the arm head emit `Release(v)` for each `v`
  used in some other arm but not this one and `v ∉ L` (dead-branch, rule c); for a boxed-union `s`
  emit `Release(s)` (container) at the arm head. `Retain(s)` before the match iff `s ∈ L`.
- Function entry: params + the cap enter owned; any not in `live` at body entry → `Release` at the
  entry (rule c: unused param/cap, mirrors `implement_lambda_function` 1982/1989).

*Tail-call exception (rules b, c).* A `Release` must not follow a tail call. When a value's last use is
an argument of a tail `App`, keep it `Own` and let the callee release it (no trailing `Release`);
likewise never place a dead-var `Release` after a tail call. Tail position is derived by the `tail_of`
lookahead (plan §1.2), not stored.

*Partial patterns (resolved).* `validate_pattern` (`elaboration/typecheck.rs:1481`) allows a struct
pattern to **omit** fields. With the uniform retain-getter above this needs no special handling: the
whole-value container `Release` traverses and drops the omitted leaves, while the extracted leaves
survive because the getter retained them (for both boxed and unbox containers). So
`destructure_pattern` projects only the mentioned fields, as it already does.

*Corner cases resolved by A/B, not by analysis:* unbox-union scrutinees with boxed payloads, exact
global-reference RC call counts, and precise release ordering. Match the current generator
structurally, then reconcile against the A/B harness (RC op count/order + valgrind), which is the
ground truth.

**Unit 3 - codegen from RC IR:** variable get is a plain get (no retain decision); `Retain`/`Release`
nodes -> inc/dec (release traverses structure); the `used_later`/`get_scoped_obj` retain branch is
deleted. Non-RC parts (closure creation, FFI, struct/array layout, LLVM construction) are ported as-is.
Gate behind the flag; validate new vs old via A/B (all opt levels, benchmarks, debug-info parity),
then flip default + delete the old path.

**Validation gate (plan 1.6):** `cargo test --release` at all `FIX_MAX_OPT_LEVEL`; RC count/order/free
behavior matches; debug-info parity; no benchmark regression (speedtest + fix-bench); then ask the
user to run external-library tests.
