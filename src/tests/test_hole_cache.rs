//! Cache integrity tests for the hole feature.
//!
//! `resolve_namespace_and_check_type_sub` skips `cache.save_cache(...)`
//! whenever `check_type` produced any tolerated error (holes,
//! cannot-infer, unsatisfied predicates, disjoint equalities).
//! Without this, the cache would memoise a typed expression for a
//! value that should be reporting an error — so the next `fix check`
//! would see the cache hit, return early, and never re-emit the
//! diagnostic.
//!
//! These tests exercise that contract end-to-end via the `fix`
//! binary: run `fix check`, confirm ERR_HOLE fires, then poke at the
//! on-disk cache directory to confirm no entry was written for the
//! offending value.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
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
    fn run_check(project_dir: &Path) -> std::process::Output {
        Command::new("fix")
            .arg("check")
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix check")
    }

    /// List the names of cache files under `<project>/.fixlang/cache/typecheck/`.
    /// Returns an empty Vec if the directory doesn't exist.
    fn list_cache_files(project_dir: &Path) -> Vec<String> {
        let cache_dir = project_dir.join(".fixlang/cache/typecheck");
        if !cache_dir.exists() {
            return vec![];
        }
        fs::read_dir(&cache_dir)
            .expect("Failed to read cache directory")
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|e| e.file_name().into_string().ok())
            })
            .collect()
    }

    /// `cache_file_name` (in `typecheckcache.rs`) replaces non-alphanumeric
    /// characters of the value name with `_`, so `Main::hole_val` becomes
    /// `Main__hole_val`. The full file name is
    /// `Main__hole_val_<scheme_md5>_<version_hash>`. We just match by
    /// prefix.
    fn has_cache_entry_for(project_dir: &Path, value_prefix: &str) -> bool {
        list_cache_files(project_dir)
            .iter()
            .any(|name| name.starts_with(value_prefix))
    }

    /// A hole-bearing value must not have a typecheck cache file
    /// written for it, even though `fix check` runs to completion and
    /// emits ERR_HOLE.
    #[test]
    fn hole_value_is_not_cached() {
        install_fix();
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

        // The hole-bearing value must not have a cache file. (Other
        // values in the project — e.g. `Main::main` — may legitimately
        // be cached, so we only check `Main__hole_val_*`.)
        assert!(
            !has_cache_entry_for(&project_dir, "Main__hole_val_"),
            "Cache file unexpectedly created for hole-bearing value.\nCache contents: {:?}",
            list_cache_files(&project_dir),
        );
    }

    /// Re-running `fix check` on the same hole-bearing source must
    /// continue to report ERR_HOLE — i.e. the cache from the first
    /// run cannot mask the diagnostic on the second.
    #[test]
    fn hole_diagnostic_persists_across_runs() {
        install_fix();
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

    /// Once the user fills in the hole, `fix check` succeeds and a
    /// cache file appears for the now-clean value (i.e. the cache
    /// suppression is gated on having errors, not permanent).
    #[test]
    fn fixed_value_is_cached_after_edit() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("with_hole");

        // First run — fails.
        let first = run_check(&project_dir);
        assert!(!first.status.success());

        // Edit the source to fill the hole. After this the value
        // should type-check cleanly and the cache should be written.
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
            has_cache_entry_for(&project_dir, "Main__hole_val_"),
            "Cache file should be created once the hole is filled.\nCache contents: {:?}",
            list_cache_files(&project_dir),
        );
    }
}
