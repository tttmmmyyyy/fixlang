# Opaque Type Test Failure Report (2026-03-29)

Test result: **565 passed, 28 failed** (`test_external_projects_cp_library` is unrelated; 27 failures analyzed below)

## Problem 1: `resolve_opaque_tycon_in_expr` does not update types in patterns / lambda variables

**Affected tests** (5): `test_opaque_repeat`, `test_opaque_doubled_evens`, `test_opaque_with_associated_type_basic`, `test_opaque_multiple_calls_different_type_args`, `test_opaque_saturated_associated_type_in_equality` (partial)

**Symptom**: Assertion failure in `optimization::remove_hktvs` via `expr_let_typed` — left side shows opaque TyCon name (e.g. `Main::repeat::?it Std::I64`), right side shows the concrete type (`MapIterator RangeIterator I64 I64`).

**Root cause**: `resolve_opaque_tycon_in_expr` in `src/elaboration/desugar_opaque.rs` (around line 602) recurses into `val` and `body` of `Expr::Let`, but does **not** resolve types in the `_pat` (pattern node). Same issue for `Expr::Lam` variables and `Expr::Match` branch patterns. Later, `expr_let_typed` asserts that the pattern type matches the bound expression type and panics.

**Fix direction**: Add opaque type resolution for pattern types and lambda variable types in `resolve_opaque_tycon_in_expr`.

---

## Problem 2: Opaque TyCon remains unresolved when reaching trait method selection during instantiation

**Affected tests** (8): `test_opaque_predicate_only`, `test_opaque_higher_kinded`, `test_opaque_higher_kinded_functor`, `test_opaque_associated_type_reduction`, `test_opaque_multi_opaque_with_shared_assoc_type`, `test_opaque_multiple_calls_same_type_args`, `test_opaque_partition`, `test_opaque_saturated_associated_type_in_equality`

**Symptom**: `called Option::unwrap() on a None value` at `src/ast/program.rs:1327` (`opt_e.unwrap()` in `instantiate_symbol`).

**Root cause**: When instantiating a symbol, `sym.ty` still contains the opaque TyCon (e.g. `Main::to_string_opaque::?s -> Std::String`). The method selector tries to find a trait implementation matching this type — but no concrete implementation matches the opaque TyCon, so all implementations are skipped. `opt_e` remains `None`.

Confirmed via logging: `Std::ToString::to_string` is instantiated with `sym.ty = Main::to_string_opaque::?s -> Std::String`, and none of the 20 `ToString` implementations match.

The opaque type resolution (`resolve_opaque_type_in_type`) is applied to a symbol's own `sym.ty` **after** instantiation, but sub-expressions that reference other symbols create new `Symbol` entries with opaque TyCons in their types **before** resolution can occur.

**Fix direction**: Resolve opaque types in `sym.ty` **before** method selection, or propagate opaque type resolutions into deferred instantiation entries.

---

## Problem 3: Trait definition validation rejects opaque-related equality constraints

**Affected tests** (3): `test_opaque_to_iter`, `test_opaque_to_iter_multiple_impls`, `test_opaque_in_impl_annotation`

**Symptom**: `error: Type variable 'c' used in trait definition cannot be constrained in the type of a member.`

**Root cause**: `validate_trait_defn` in `src/ast/traits.rs` (line 875) checks whether the trait's type variable `c` appears in any constraint of a member's qualified type. For `to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it`, the equality constraint `Item ?it = Elem c` contains `c`. This validation runs **before** opaque desugaring, which would remove these opaque-related constraints.

`find_var_in_constraint` in `src/ast/qual_type.rs` (line 101) does not distinguish opaque-related constraints from regular ones.

**Fix direction**: Either run opaque desugaring before this validation, or make `find_var_in_constraint` skip constraints that are "on" an opaque type variable (i.e. the constraint's predicate/equality primarily involves an opaque tyvar).

---

## Problem 4: Type variable grammar does not allow underscores

**Affected tests** (1): `test_opaque_zip_with_index`

**Symptom**: `error: Expected type_arrow.` at position 4:29

**Root cause**: In `src/parse/grammer.pest` line 216, `tyvar_char = _{ ASCII_ALPHA | ASCII_DIGIT }` — underscores are not included. The test uses `it_in` as a type variable name, but the parser only recognizes `it` as the type variable, leaving `_in` unparsed.

Normal identifiers (`name_char`, line 23) do include underscores: `name_char = _{ ASCII_ALPHA | ASCII_DIGIT | "_" }`.

**Fix direction**: Either change the test to avoid underscores in type variable names (e.g. `itIn`), or add `"_"` to `tyvar_char` in the grammar.

---

## Problem 5: Missing validations — code that should fail compiles successfully

**Affected tests** (3): `test_opaque_branch_type_mismatch`, `test_opaque_not_in_return_type`, `test_opaque_unused_cannot_determine`

**Symptom**: `error: The source code was expected to fail, but succeeded.`

**Root cause**: These tests expect compile-time errors that are not yet implemented:
- **`test_opaque_branch_type_mismatch`**: if-then-else branches return different concrete types under the same opaque type — should be caught as a type mismatch but isn't.
- **`test_opaque_not_in_return_type`**: opaque type appears in constraints but not in the return type — should be reported as an undetermined type.
- **`test_opaque_unused_cannot_determine`**: opaque type is unconstrained in the function body — should be reported as ambiguous.

**Fix direction**: Add validation passes for these cases, likely during or after opaque desugaring.

---

## Problem 6: Other individual issues (7 tests)

| Test | Issue |
|---|---|
| `test_opaque_in_impl_type_param` | Error message mismatch. `impl ?x : Foo` produces generic "Implementing trait for type `?x` is not allowed" (`src/ast/types.rs:536`), but the test expects "Opaque type variable". Need to add an opaque-specific check/message in `is_implementable`. |
| `test_opaque_with_higher_kinded_assoc_type` | `Kind mismatch in 'f'. Expect: *->*, found: *` — kind inference for equality constraint `Container ?c = f` doesn't correctly propagate higher-kinded information. |
| `test_opaque_with_higher_arity_assoc_type` | `Name 'from_array' is ambiguous` — name collision between the test's `from_array` and `Std::Iterator::from_array`. Test naming issue or missing opaque-aware name resolution. |
| `test_opaque_higher_arity_associated_type` | Same `from_array` name ambiguity as above. |
| `test_opaque_unsaturated_associated_type_in_equality_lhs` | Error message mismatch. Expects "associated type has to be saturated" but gets "The left side of an equality constraint should be the application of an associated type". |
| `test_opaque_unsaturated_higher_arity_associated_type_in_equality` | Equality constraint deduction failure with opaque types and higher-arity associated types. |
| `test_opaque_multiple_associated_types_in_equality` | Equality constraint deduction failure — `Std::Array = Main::opaque_first::?c` cannot be deduced. |

---

## Priority

1. **Problem 1** (pattern type resolution) — affects the most tests, clear and localized fix.
2. **Problem 3** (trait validation vs opaque constraints) — blocks all trait-method-with-opaque use cases.
3. **Problem 2** (method selection with unresolved opaque types) — may partially resolve after Problem 1, but likely needs independent fix for instantiation ordering.
4. **Problem 5** (missing validations) — needed for correctness but lower urgency.
5. **Problems 4 & 6** — smaller individual fixes.
