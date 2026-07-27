//! LSP integration test for stdin EOF handling.
//!
//! When the parent editor process dies it closes the pipe connected to
//! the language server's stdin. `read_line` then returns `Ok(0)` on
//! every call, and the server's read loop must recognize that as EOF and
//! terminate, so that no orphaned process is left behind.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{copy_dir_recursive, fix_command, wait_within};
    use std::{path::PathBuf, process::Stdio, thread::sleep, time::Duration};
    use tempfile::TempDir;

    /// Absolute path to the LSP `cases/` directory.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    /// Copy the named test project into a fresh temp directory and
    /// return both the temp dir handle (to keep it alive) and the path
    /// of the copied project.
    fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_cases_dir().join(project_name);
        let test_case_dst = temp_dir.path().join(project_name);
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
        (temp_dir, test_case_dst)
    }

    /// Verifies that the language server terminates promptly once its
    /// stdin reaches EOF (parent editor closed the pipe).
    #[test]
    fn test_lsp_exits_on_stdin_eof() {
        let (_temp_dir, project_dir) = setup_test_env("completion");

        let mut child = fix_command()
            .arg("language-server")
            .current_dir(&project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn fix language-server");

        // Give the server time to start and block on its stdin read loop.
        sleep(Duration::from_millis(500));

        // Sanity check: the server should still be running here (blocked
        // waiting for input), not exited for some unrelated reason.
        assert!(
            child
                .try_wait()
                .expect("Failed to poll server status")
                .is_none(),
            "Server exited before stdin was closed"
        );

        // Simulate the parent editor process dying: close stdin so the
        // server observes EOF.
        drop(child.stdin.take().expect("stdin handle already taken"));

        // The server must terminate promptly once it observes the EOF.
        wait_within(
            &mut child,
            Duration::from_secs(10),
            "the LSP server after stdin reached EOF",
        );
    }
}
