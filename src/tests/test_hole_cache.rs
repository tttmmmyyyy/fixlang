//! Cache integrity tests for the hole feature.
//!
//! `Program::check_body` reports the tolerated errors of a check (holes, cannot-infer,
//! unsatisfied predicates, disjoint equalities) as diagnostics, and a body whose check produced
//! any is kept out of the type-check cache. Without this, the cache would memoise a typed
//! expression for a value that should be reporting an error — so the next `fix check` would skip
//! the value and never re-emit the diagnostic.
//!
//! These tests exercise that contract end-to-end via the `fix` binary: run `fix check`, confirm
//! ERR_HOLE fires, then look into the on-disk cache files to confirm the offending value is not
//! recorded there.

#[cfg(test)]
mod integration_tests {
    use crate::constants::TYPED_MODULES_PATH;
    use crate::tests::test_util::{cache_file_names, copy_dir_recursive, fix_command};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use tempfile::TempDir;

    /// Absolute path to the `cases/` directory shipped alongside this
    /// test file.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_hole_cache/cases");
        path
    }

    /// Copy the case named `case_name` into a fresh temp directory and
    /// return the temp dir handle (kept alive by the caller) plus the
    /// absolute path of the copied project.
    fn setup_test_env(case_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src = get_test_cases_dir().join(case_name);
        let dst = temp_dir.path().join(case_name);
        copy_dir_recursive(&src, &dst).expect("Failed to copy test case");
        (temp_dir, dst)
    }

    /// Run `fix check` in `project_dir` and return the full process
    /// output.
    fn run_check(project_dir: &Path) -> process::Output {
        fix_command()
            .arg("check")
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix check")
    }

    /// The names of the files under `TYPED_MODULES_PATH` in the project. Empty if the directory
    /// doesn't exist.
    fn list_cache_files(project_dir: &Path) -> Vec<String> {
        cache_file_names(project_dir, TYPED_MODULES_PATH)
    }

    /// Reports whether a cache file records a body of the global value `value_name`.
    ///
    /// A file records each body under its `EntityIdentity`, the value's full name and its type on
    /// two lines, and holds the identity as the bytes it is spelled with.
    fn has_cache_entry_for(project_dir: &Path, value_name: &str) -> bool {
        let identity_head = format!("{}\n", value_name);
        list_cache_files(project_dir).iter().any(|file_name| {
            let bytes = fs::read(project_dir.join(TYPED_MODULES_PATH).join(file_name))
                .expect("Failed to read a cache file");
            bytes
                .windows(identity_head.len())
                .any(|window| window == identity_head.as_bytes())
        })
    }

    /// A hole-bearing value must not be recorded in the type-check
    /// cache, even though `fix check` runs to completion and emits
    /// ERR_HOLE.
    #[test]
    fn hole_value_is_not_cached() {
        let (_temp_dir, project_dir) = setup_test_env("with_hole");

        // First run: should emit ERR_HOLE and exit non-zero.
        let output = run_check(&project_dir);
        assert!(
            !output.status.success(),
            "fix check on a hole-bearing project should fail.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Expected expression of type"),
            "fix check should report ERR_HOLE. Got:\n{}",
            stderr,
        );

        // The hole-bearing value must not be recorded. (Other values in
        // the project — e.g. `Main::main` — may legitimately be cached,
        // so we only check `Main::hole_val`.)
        assert!(
            !has_cache_entry_for(&project_dir, "Main::hole_val"),
            "Hole-bearing value unexpectedly recorded in the cache.\nCache contents: {:?}",
            list_cache_files(&project_dir),
        );
    }

    /// Re-running `fix check` on the same hole-bearing source must
    /// continue to report ERR_HOLE — i.e. the cache from the first
    /// run cannot mask the diagnostic on the second.
    #[test]
    fn hole_diagnostic_persists_across_runs() {
        let (_temp_dir, project_dir) = setup_test_env("with_hole");

        // First run.
        let first = run_check(&project_dir);
        assert!(
            !first.status.success(),
            "first run of fix check on hole-bearing project should fail",
        );

        // Second run on the same source. If the cache had been saved
        // for the hole-bearing value, the second run would short-
        // circuit on cache hit and skip ERR_HOLE.
        let second = run_check(&project_dir);
        assert!(
            !second.status.success(),
            "second run should still fail with ERR_HOLE; cache must not mask it.\nstderr: {}",
            String::from_utf8_lossy(&second.stderr),
        );
        let stderr = String::from_utf8_lossy(&second.stderr);
        assert!(
            stderr.contains("Expected expression of type"),
            "second run should still report ERR_HOLE. Got:\n{}",
            stderr,
        );
    }

    /// Once the user fills in the hole, `fix check` succeeds and the
    /// cache records the now-clean value: the cache is
    /// withheld for as long as the value reports an error, and written
    /// again once it type-checks cleanly.
    #[test]
    fn fixed_value_is_cached_after_edit() {
        let (_temp_dir, project_dir) = setup_test_env("with_hole");

        // First run — fails.
        let first = run_check(&project_dir);
        assert!(!first.status.success());

        // Edit the source to fill the hole. After this the value
        // should type-check cleanly and be recorded in the cache.
        let main_path = project_dir.join("main.fix");
        let new_source = fs::read_to_string(&main_path)
            .expect("Failed to read main.fix")
            .replace("hole_val : I64 = ;", "hole_val : I64 = 0;");
        fs::write(&main_path, new_source).expect("Failed to write main.fix");

        let after_edit = run_check(&project_dir);
        assert!(
            after_edit.status.success(),
            "fix check should succeed after the hole is filled.\nstderr: {}",
            String::from_utf8_lossy(&after_edit.stderr),
        );
        assert!(
            has_cache_entry_for(&project_dir, "Main::hole_val"),
            "The value should be recorded once the hole is filled.\nCache contents: {:?}",
            list_cache_files(&project_dir),
        );
    }

    /// A test build and a plain build over one project directory each produce their own program,
    /// out of one type-check cache.
    ///
    /// The two builds see different sets of tuple sizes, so the trait implementations the compiler
    /// generates for tuples differ between them, and each build's entries name the sources that
    /// build is made of. Both builds read and write one directory of entries, and what each entry
    /// is filed under is what keeps the second build from taking the first one's answers.
    #[test]
    fn a_test_build_and_a_plain_build_share_the_cache_without_corrupting_it() {
        let (_temp_dir, project_dir) = setup_test_env("test_and_build");

        let test_run = fix_command()
            .arg("test")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix test");
        assert_eq!(
            String::from_utf8_lossy(&test_run.stdout).trim(),
            "(1, 2, 3, 4, 5, 6, 7)",
            "the test entry point prints the seven-tuple.\nstderr: {}",
            String::from_utf8_lossy(&test_run.stderr),
        );

        let build_run = fix_command()
            .arg("run")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix run");
        assert_eq!(
            String::from_utf8_lossy(&build_run.stdout).trim(),
            "(1, 2)",
            "the main entry point prints the pair.\nstderr: {}",
            String::from_utf8_lossy(&build_run.stderr),
        );
    }
}
