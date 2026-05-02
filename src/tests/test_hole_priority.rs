// Tests for the diagnostic cascade in `check_type`:
//
//   hole > cannot-infer > predicate > equality
//
// When earlier-tier diagnostics fire, later-tier ones are suppressed
// (they're usually consequences of the earlier failure).
//
// Plus a multi-hole test: every hole occurrence in a single value
// must produce its own ERR_HOLE.

use crate::{
    configuration::Configuration,
    tests::test_util::{test_source_fail, test_source_fail_excludes},
};

const HOLE_ERR_MARKER: &str = "Expected expression of type";

// ----- holes shadow later layers ------------------------------------

#[test]
fn hole_suppresses_cannot_infer() {
    // The inner let body is a hole, so the inner let has type `a`,
    // making `let x = inner` introduce `x : a`. Without the cascade,
    // Fix would also report a "Cannot infer the type of this pattern"
    // error for `x`. With the cascade we expect only ERR_HOLE.
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

#[test]
fn multiple_holes_in_distinct_values_all_reported() {
    // Three holes across three top-level values. Each must produce
    // its own ERR_HOLE.
    let source = r#"
        module Main;
        a : I64 = ;
        b : I64 = ;
        c : I64 = ;
        main : IO () = pure();
    "#;
    let stderr = run_and_get_stderr(source);
    let n = stderr.matches(HOLE_ERR_MARKER).count();
    assert_eq!(
        n, 3,
        "expected 3 ERR_HOLE diagnostics, got {}.\nstderr:\n{}",
        n, stderr,
    );
}

#[test]
fn multiple_holes_in_one_value_all_reported() {
    // Two holes in the same value (an `if` with both branches empty).
    let source = r#"
        module Main;
        f : I64 = if true { } else { };
        main : IO () = pure();
    "#;
    let stderr = run_and_get_stderr(source);
    let n = stderr.matches(HOLE_ERR_MARKER).count();
    assert_eq!(
        n, 2,
        "expected 2 ERR_HOLE diagnostics for if-then-else with both branches empty, got {}.\nstderr:\n{}",
        n, stderr,
    );
}

// ----- helper ------------------------------------------------------

fn run_and_get_stderr(source: &str) -> String {
    use crate::misc::save_temporary_source;
    let mut config = Configuration::develop_mode();
    let src = match save_temporary_source(source, "main_run") {
        Ok(s) => s,
        Err(errs) => return errs.to_string(),
    };
    config.add_user_source_file(src.file_path);
    match crate::commands::run::run(config, false) {
        Err(errs) => errs.to_string(),
        Ok(Err(e)) => e.to_string(),
        Ok(Ok(output)) => String::from_utf8_lossy(&output.stderr).to_string(),
    }
}
