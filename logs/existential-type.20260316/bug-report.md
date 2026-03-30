# Opaque Type: Remaining Test Failures (2026-03-30)

Test result: **28 passed, 4 failed** (out of 35 `test_opaque` tests)

---

## 2. Opaque type variable not recognized in impl annotation (1 test)

**Test**: `test_opaque_in_impl_annotation`

**Symptom**: `Unknown type variable '?it'` — the opaque type variable in the impl annotation (`x.Array::to_iter : ?it`) is not in scope.

**Fix direction**: Opaque type variables need to be brought into scope within impl bodies / annotations.

---

## 5. Equality constraint deduction failure with multiple associated types (3 tests)

**Tests**: `test_opaque_multiple_associated_types_in_equality`, `test_opaque_higher_arity_associated_type_in_equality`, `test_opaque_with_higher_kinded_assoc_type`

**Symptom**: `Std::Array = Main::...::?c` cannot be deduced. Also triggers indeterminate type variable.

**Fix direction**: Equality constraint solver needs to handle opaque type constructors in deduction.

---

## Priority

1. **Issue 2** (opaque tyvar in impl annotation) — need scoping fix.
2. **Issue 5** (equality constraint deduction) — solver change for opaque TyCons.
