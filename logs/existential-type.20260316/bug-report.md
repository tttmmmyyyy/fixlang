# Opaque Type: Remaining Test Failures (2026-03-30)

Test result: **26 passed, 9 failed** (out of 35 `test_opaque` tests)

---

## 1. Error message mismatches (3 tests)

| Test | Actual error | Expected error |
|---|---|---|
| `test_opaque_in_impl_type_param` | "Implementing trait for type `?x` is not allowed" (`src/ast/types.rs:536`) | "Opaque type variable" |
| `test_opaque_unsaturated_associated_type_in_equality_lhs` | "The left side of an equality constraint should be the application of an associated type" | "associated type has to be saturated" |
| `test_opaque_unsaturated_higher_arity_associated_type_in_equality` | "Invalid number of arguments for associated type `Main::Rebuildable::Rebuild`. Expect: 2, found: 1." | "associated type has to be saturated" |

**Fix direction**: Add opaque-specific check in `is_implementable` for the first. Update test expectations or error messages for the other two.

---

## 2. Opaque type variable not recognized in impl annotation (1 test)

**Test**: `test_opaque_in_impl_annotation`

**Symptom**: `Unknown type variable '?it'` — the opaque type variable in the impl annotation (`x.Array::to_iter : ?it`) is not in scope.

**Fix direction**: Opaque type variables need to be brought into scope within impl bodies / annotations.

---

## 3. Kind inference failure with higher-kinded associated type (1 test)

**Test**: `test_opaque_with_higher_kinded_assoc_type`

**Symptom**: `Kind mismatch in 'f'. Expect: *->*, found: *` — equality constraint `Container ?c = f` doesn't propagate kind `*->*` to `f`.

**Fix direction**: Kind inference for equality constraints needs to propagate the kind of the associated type's result to the RHS type variable.

---

## 4. Name ambiguity with `from_array` / `to_array` (3 tests)

| Test | Ambiguous name | Collision |
|---|---|---|
| `test_opaque_with_higher_arity_assoc_type` | `from_array` | `Main::from_array` vs `Std::Iterator::from_array` |
| `test_opaque_higher_arity_associated_type` | `from_array` | same |
| `test_opaque_higher_arity_associated_type_in_equality` | `to_array` | `Main::Rebuildable::to_array` vs `Std::Iterator::to_array` |

The third test also has a cascading error: `Std::Array = Main::from_rebuildable::?c cannot be deduced`.

**Fix direction**: Rename test functions to avoid collision, or add `hiding` in test code.

---

## 5. Equality constraint deduction failure with multiple associated types (1 test)

**Test**: `test_opaque_multiple_associated_types_in_equality`

**Symptom**: `Std::Array = Main::opaque_first::?c` cannot be deduced. Also triggers indeterminate type variable `#a145`.

**Fix direction**: Equality constraint solver needs to handle opaque type constructors in deduction.

---

## Priority

1. **Issue 1** (error message mismatches) — easy, update tests or messages.
2. **Issue 4** (name ambiguity) — easy, rename test functions or add `hiding`.
3. **Issue 2** (opaque tyvar in impl annotation) — need scoping fix.
4. **Issue 3** (kind inference) — kind propagation for equality constraints.
5. **Issue 5** (equality constraint deduction) — solver change for opaque TyCons.
