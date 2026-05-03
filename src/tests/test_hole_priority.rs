//! Tests for the diagnostic cascade in `check_type`:
//!
//!   hole > cannot-infer > predicate > equality
//!
//! When earlier-tier diagnostics fire, later-tier ones are suppressed
//! (they're usually consequences of the earlier failure).
//!
//! Plus a multi-hole test: every hole occurrence in a single value
//! must produce its own ERR_HOLE.

use crate::{
    configuration::Configuration,
    tests::test_util::{run_source_assert_failed, test_source_fail, test_source_fail_excludes},
};

/// Substring used to recognise an ERR_HOLE diagnostic in stderr.
const HOLE_ERR_MARKER: &str = "Expected expression of type";

// ----- holes shadow later layers ------------------------------------

/// When a hole leaves an outer pattern's type indeterminate, the
/// cascade reports only ERR_HOLE, not the cannot-infer diagnostic
/// that would otherwise also fire on the pattern.
#[test]
fn hole_suppresses_cannot_infer() {
    let source = r#"
        module Main;
        f : I64 = (let x = (let y = 10; ); );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
    test_source_fail_excludes(
        source,
        Configuration::develop_mode(),
        "Cannot infer the type",
    );
}

// ----- multi-hole reporting ----------------------------------------

/// Every hole in a distinct top-level value produces its own
/// ERR_HOLE; no value's diagnostic masks the others.
#[test]
fn multiple_holes_in_distinct_values_all_reported() {
    let source = r#"
        module Main;
        a : I64 = ;
        b : I64 = ;
        c : I64 = ;
        main : IO () = pure();
    "#;
    let stderr = run_source_assert_failed(source, Configuration::develop_mode());
    let n = stderr.matches(HOLE_ERR_MARKER).count();
    assert_eq!(
        n, 3,
        "expected 3 ERR_HOLE diagnostics, got {}.\nstderr:\n{}",
        n, stderr,
    );
}

/// Multiple holes inside a single value (here, both branches of an
/// `if`) each emit their own ERR_HOLE.
#[test]
fn multiple_holes_in_one_value_all_reported() {
    let source = r#"
        module Main;
        f : I64 = if true { } else { };
        main : IO () = pure();
    "#;
    let stderr = run_source_assert_failed(source, Configuration::develop_mode());
    let n = stderr.matches(HOLE_ERR_MARKER).count();
    assert_eq!(
        n, 2,
        "expected 2 ERR_HOLE diagnostics for if-then-else with both branches empty, got {}.\nstderr:\n{}",
        n, stderr,
    );
}
