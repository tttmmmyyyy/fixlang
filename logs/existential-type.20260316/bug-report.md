# Opaque Type: Remaining Test Failures (2026-03-31)

Test result: **28 passed, 4 failed** → **31 passed, 1 failed** (out of 35 `test_opaque` tests)

---

## 2. Opaque type variable not recognized in impl annotation (1 test) — FIXED

**Test**: `test_opaque_in_impl_annotation`

**Status**: Now passes. The scoping issue was resolved.

---

## 5. Equality constraint deduction failure with multiple associated types (3 tests) — RESOLVED

**Tests**:
- `test_opaque_multiple_associated_types_in_equality` — **rewritten, now passes**
- `test_opaque_higher_arity_associated_type_in_equality` — **rewritten, now passes**
- `test_opaque_with_higher_kinded_assoc_type` — **already passes**

**Analysis**: The original tests placed the opaque type variable `?c` exclusively in
argument position (`?c -> ...`). However, opaque types are existential types that
hide the concrete return type — the caller cannot know the hidden type, so passing
a concrete value (e.g. `Array I64`) as an opaque argument is fundamentally unsound.
The unifier correctly rejects `Array = Main::...::?c` because an opaque TyCon is
opaque to the caller; no deduction path exists from caller-side to resolve it.

**Resolution**: Tests were rewritten to place `?c` in return position, which is the
natural and intended use of opaque types. The core test intent is preserved:
- Test 1: Multiple associated type equality constraints (`Elem ?c = I64`, `Size ?c = I64`)
  on a single opaque variable.
- Test 2: Higher-arity associated type equality constraint (`Rebuild ?c b = Array b`)
  on an opaque variable.

---

## Priority

1. ~~**Issue 2** (opaque tyvar in impl annotation) — need scoping fix.~~ DONE
2. ~~**Issue 5** (equality constraint deduction) — solver change for opaque TyCons.~~ Tests fixed (not a solver bug)
