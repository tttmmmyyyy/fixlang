//! Module-dependency-hash regression tests for absolute-path symbol
//! references.
//!
//! `module_dependency_hash_map` (in `src/ast/program.rs`) walks only
//! `import` statements when computing the set of dependent modules.
//! Absolute paths like `::Lib::message` let `Main` reach a symbol in
//! `Lib` without importing it (see `FullName::is_absolute` and the
//! corresponding skips in `src/elaboration/typecheck.rs`), so editing
//! `lib.fix` does not change `Main`'s module-dependency hash. The
//! typecheck cache for `Main::main` then stays valid across the edit
//! and the stale typed expression is reused — a type-soundness bug.
//!
//! The tests probe the soundness directly by making `lib.fix` change
//! the *type* of `Lib::message` (`String` → `I64`). After the edit
//! `println(::Lib::message)` in `main.fix` is no longer well typed,
//! so a second `fix check` must report a type error. With the bug,
//! the cache hit on `Main::main` masks the change and the second
//! `fix check` silently succeeds.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};
    use tempfile::TempDir;

    /// Returns the directory holding the sample Fix projects used by these tests.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_abs_path_dep_hash/cases");
        path
    }

    /// Copies the named test case into a fresh temporary directory and returns the temp-dir guard along with the path to the copied project.
    fn setup_test_env(case_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src = get_test_cases_dir().join(case_name);
        let dst = temp_dir.path().join(case_name);
        copy_dir_recursive(&src, &dst).expect("Failed to copy test case");
        (temp_dir, dst)
    }

    /// Runs `fix check` in `project_dir` and returns its captured output.
    fn run_check(project_dir: &Path) -> Output {
        Command::new("fix")
            .arg("check")
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix check")
    }

    /// Rewritten `lib.fix` whose `message` is `I64` (not `String`),
    /// so `println(::Lib::message)` in `main.fix` no longer
    /// type-checks.
    const LIB_FIX_AFTER_TYPE_CHANGE: &str = "module Lib;\n\nmessage : I64;\nmessage = 42;\n";

    /// Runs the lib-type-change scenario for `case_name`: verifies the
    /// initial project type-checks, edits `lib.fix` so the type of
    /// `Lib::message` changes from `String` to `I64`, then returns the
    /// output of a second `fix check`. The caller decides whether the
    /// second run should succeed or fail.
    fn run_lib_type_change_scenario(case_name: &str) -> Output {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env(case_name);

        let first = run_check(&project_dir);
        assert!(
            first.status.success(),
            "first `fix check` should succeed for case `{}`.\nstdout: {}\nstderr: {}",
            case_name,
            String::from_utf8_lossy(&first.stdout),
            String::from_utf8_lossy(&first.stderr),
        );

        let lib_path = project_dir.join("lib.fix");
        fs::write(&lib_path, LIB_FIX_AFTER_TYPE_CHANGE).expect("Failed to write lib.fix");

        run_check(&project_dir)
    }

    /// `Main` references `Lib::message` only via the absolute path
    /// `::Lib::message`, with no `import Lib;`. After editing `lib.fix`
    /// so `Lib::message`'s type changes (`String` → `I64`), the second
    /// `fix check` must report a type error in `Main::main` —
    /// `println` expects `String` but the new `Lib::message` is `I64`.
    /// If the dependency hash misses the absolute-path edge, the stale
    /// typecheck cache for `Main::main` would mask the change.
    #[test]
    fn abs_path_no_import_propagates_lib_type_change() {
        let second = run_lib_type_change_scenario("abs_path_no_import");
        assert!(
            !second.status.success(),
            "After changing `Lib::message` from `String` to `I64`, the second `fix check` \
             must report a type error in `Main::main`. It succeeded, meaning the stale typecheck \
             cache for `Main::main` masked the change — the module-dependency-hash bug.\n\
             stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&second.stdout),
            String::from_utf8_lossy(&second.stderr),
        );
    }

    /// Control. `Main` has an explicit `import Lib;`, so the
    /// dependency hash picks up the type change today and the second
    /// `fix check` correctly reports a type error. If this test
    /// fails, the observation harness itself is broken.
    #[test]
    fn with_import_propagates_lib_type_change() {
        let second = run_lib_type_change_scenario("with_import");
        assert!(
            !second.status.success(),
            "With explicit `import Lib;`, the second `fix check` must report a type error after \
             `Lib::message` changes type. Got success instead, which suggests the observation \
             harness is broken.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&second.stdout),
            String::from_utf8_lossy(&second.stderr),
        );
    }

    /// An absolute path naming a module that isn't linked must produce
    /// a "Cannot find module" error pointing at the module token in
    /// the user's expression — not a span-less error from the
    /// parser-synthesised implicit import, and not a fallthrough
    /// "Unknown name" that hides the real cause.
    #[test]
    fn missing_module_via_absolute_path_reports_at_module_token() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("abs_path_missing_module");

        let output = run_check(&project_dir);
        assert!(
            !output.status.success(),
            "`fix check` should fail when an absolute path names an unknown module.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Cannot find module `NoSuchModule`"),
            "stderr should report the missing module by name. Got:\n{}",
            stderr,
        );
        assert!(
            stderr.contains("in \"main.fix\""),
            "stderr should attribute the error to the user's source file, not nowhere. Got:\n{}",
            stderr,
        );
    }

    /// An absolute path naming an existing module but a missing value
    /// (`::Lib::messag` — `message` typoed) must produce a span-bearing
    /// "Cannot find value named `Lib::messag`" error from
    /// `validate_import_statements`. The synthesised implicit import
    /// carries the source span of the leaf-name token, so the error
    /// points at the user's typo rather than nowhere.
    #[test]
    fn value_typo_via_absolute_path_reports_unknown_name_with_span() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("abs_path_value_typo");

        let output = run_check(&project_dir);
        assert!(
            !output.status.success(),
            "`fix check` should fail on a typoed value name via absolute path.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Cannot find value named `Lib::messag`"),
            "stderr should report the typoed value by qualified name. Got:\n{}",
            stderr,
        );
        assert!(
            stderr.contains("in \"main.fix\""),
            "stderr should attribute the error to the user's source file, not nowhere. Got:\n{}",
            stderr,
        );
    }
}
