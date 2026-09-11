//! The copy of a test project a session runs over.

use crate::tests::test_util::copy_dir_recursive;
use std::path::PathBuf;
use tempfile::TempDir;

/// The directory holding the LSP test projects, one subdirectory per
/// project, named as the tests name it.
fn get_test_cases_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/tests/test_lsp/cases");
    path
}

/// Copy the test project `project_name` into a temporary directory of its
/// own, so tests that build and edit it can run in parallel.
///
/// # Returns
/// The guard whose drop deletes the copy, and the canonicalized path of
/// the copied project. Canonicalizing resolves the symlinks a temporary
/// directory sits behind (`/tmp` -> `/private/tmp` on macOS), so the path
/// matches the root URI the server is initialized with and the URIs it
/// publishes under.
pub fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_case_src = get_test_cases_dir().join(project_name);
    let test_case_dst = temp_dir.path().join(project_name);
    copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
    let test_case_dst = test_case_dst
        .canonicalize()
        .expect("Failed to canonicalize test case path");
    (temp_dir, test_case_dst)
}
