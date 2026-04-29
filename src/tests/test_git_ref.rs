// Integration tests for git ref (rev/tag) pinning in dependencies.

#[cfg(test)]
mod integration_tests {
    use crate::constants::LOCK_FILE_PATH;
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use std::{fs, path::{Path, PathBuf}, process::Command};
    use tempfile::TempDir;

    // Initialize a local upstream repo containing a minimal fix project, commit it,
    // and create an annotated tag at the given version.
    fn create_annotated_tag_upstream(repo_dir: &Path, version: &str) {
        fs::create_dir_all(repo_dir).expect("Failed to create upstream dir");

        let run = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(repo_dir)
                .status()
                .unwrap_or_else(|e| panic!("Failed to run `git {:?}`: {}", args, e));
            assert!(status.success(), "`git {:?}` failed", args);
        };

        run(&["init", "-q", "-b", "main"]);
        // Local identity so commit succeeds regardless of the env's git config.
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "Test"]);

        let proj_toml = format!(
            "[general]\n\
             name = \"annotated-mock\"\n\
             version = \"{}\"\n\
             fix_version = \"*\"\n\
             \n\
             [build]\n\
             files = [\"lib.fix\"]\n",
            version
        );
        fs::write(repo_dir.join("fixproj.toml"), proj_toml)
            .expect("Failed to write fixproj.toml");
        fs::write(
            repo_dir.join("lib.fix"),
            "module AnnotatedMock;\n\
             \n\
             annotated_mock_value : I64;\n\
             annotated_mock_value = 42;\n",
        )
        .expect("Failed to write lib.fix");

        run(&["add", "fixproj.toml", "lib.fix"]);
        run(&["commit", "-q", "-m", "init"]);
        // -a creates an annotated tag (a tag object) rather than a lightweight tag.
        run(&["tag", "-a", version, "-m", &format!("release {}", version)]);
    }

    // Write a minimal consumer fix project that depends on `upstream_dir` via git.
    // If `tag` is Some, the dep pins to that tag; otherwise `version` is required.
    fn write_consumer_project(
        consumer_dir: &Path,
        upstream_dir: &Path,
        tag: Option<&str>,
        version: Option<&str>,
    ) {
        fs::create_dir_all(consumer_dir).expect("Failed to create consumer dir");
        let url = upstream_dir.to_string_lossy().replace('\\', "/");
        let dep_line = match (tag, version) {
            (Some(t), None) => format!("git = {{ url = \"{}\", tag = \"{}\" }}", url, t),
            (None, Some(v)) => format!(
                "version = \"{}\"\ngit = {{ url = \"{}\" }}",
                v, url
            ),
            _ => panic!("write_consumer_project: pass exactly one of tag/version"),
        };
        let proj_toml = format!(
            "[general]\n\
             name = \"consumer\"\n\
             version = \"0.1.0\"\n\
             fix_version = \"*\"\n\
             \n\
             [build]\n\
             files = [\"main.fix\"]\n\
             \n\
             [[dependencies]]\n\
             name = \"annotated-mock\"\n\
             {}\n",
            dep_line
        );
        fs::write(consumer_dir.join("fixproj.toml"), proj_toml)
            .expect("Failed to write consumer fixproj.toml");
        fs::write(
            consumer_dir.join("main.fix"),
            "module Main;\n\
             \n\
             import AnnotatedMock;\n\
             \n\
             main : IO ();\n\
             main = println(AnnotatedMock::annotated_mock_value.to_string);\n",
        )
        .expect("Failed to write main.fix");
    }

    // Get the path to the git_ref_tests directory.
    fn get_git_ref_test_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_dependencies/cases/git_ref_tests");
        path
    }

    // Create a temporary test environment with copied project files for a specific test case.
    fn setup_git_ref_test_env(case_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src = get_git_ref_test_dir().join(case_name);
        let dst = temp_dir.path().join(case_name);
        copy_dir_recursive(&src, &dst).expect("Failed to copy test case");
        (temp_dir, dst)
    }

    // Clean up lock files and build artifacts before running test.
    fn cleanup_test_project(project_dir: &PathBuf) {
        let _ = fs::remove_file(project_dir.join(LOCK_FILE_PATH));
        let _ = fs::remove_dir_all(project_dir.join(".fix"));
        let _ = Command::new("fix")
            .arg("clean")
            .current_dir(project_dir)
            .output();
    }

    // Test 1: rev pinning builds successfully.
    #[test]
    fn test_git_rev_basic() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_basic");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("7602fba"),
            "Lock file should contain the pinned rev"
        );
        assert!(
            lock_content.contains("version = \"1.1.0\""),
            "Lock file should show version 1.1.0"
        );
    }

    // Test 2: tag pinning builds successfully.
    #[test]
    fn test_git_tag_basic() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("tag_basic");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("7602fba"),
            "Lock file should contain the rev for tag 1.1.0"
        );
        assert!(
            lock_content.contains("version = \"1.1.0\""),
            "Lock file should show version 1.1.0"
        );
    }

    // Test 3: rev + version (version requirement satisfied).
    #[test]
    fn test_git_rev_with_version_ok() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_with_version_ok");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test 4: rev + version (version requirement NOT satisfied → error).
    #[test]
    fn test_git_rev_with_version_fail() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_with_version_fail");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            !output.status.success(),
            "fix build should fail due to version mismatch"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("not satisfied"),
            "Error should mention version not satisfied: {}",
            stderr
        );
    }

    // Test 5: rev and tag both specified → validation error.
    #[test]
    fn test_git_rev_and_tag_conflict() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_and_tag_conflict");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            !output.status.success(),
            "fix build should fail due to rev+tag conflict"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Only one of"),
            "Error should mention mutual exclusion: {}",
            stderr
        );
    }

    // Test 6: fix deps update does not change tag-pinned dependency.
    #[test]
    fn test_git_tag_update_stable() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("tag_update_stable");
        cleanup_test_project(&project_dir);

        // First build.
        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(lock_content.contains("7602fba"));
        assert!(lock_content.contains("version = \"1.1.0\""));

        // Run fix deps update.
        let output = Command::new("fix")
            .args(&["deps", "update"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix deps update");
        assert!(
            output.status.success(),
            "fix deps update failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Verify lock file is unchanged (still pinned to 1.1.0, not updated to 1.2.1).
        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("7602fba"),
            "Lock file should still contain the rev for tag 1.1.0 after deps update"
        );
        assert!(
            lock_content.contains("version = \"1.1.0\""),
            "Lock file should still show version 1.1.0 after deps update"
        );
    }

    // Test 7: Root pins with tag, transitive dep A requires compatible version range → success.
    #[test]
    fn test_transitive_root_pins_ok() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_root_pins_ok");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("7602fba"),
            "Lock file should contain rev for tag 1.1.0"
        );
        assert!(
            lock_content.contains("version = \"1.1.0\""),
            "Lock file should show version 1.1.0"
        );
    }

    // Test 8: Root pins with tag, transitive dep A requires incompatible version range → error.
    #[test]
    fn test_transitive_root_pins_fail() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_root_pins_fail");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            !output.status.success(),
            "fix build should fail due to version incompatibility"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Failed to resolve dependencies"),
            "Error should mention failed resolution: {}",
            stderr
        );
    }

    // Test 9: Root uses version range, A pins with tag → A's pin is ignored, SemVer resolution used.
    // A pins math to tag v0.1.3 which is OUTSIDE root's ^1.0 range.
    // If A's pin were respected, the build would fail. Success proves the pin is ignored.
    #[test]
    fn test_transitive_local_pins_ignored() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_local_pins");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        // A pins math to v0.1.3 (outside root's ^1.0), but the pin is ignored for transitive deps.
        // The build succeeds because the resolver uses root's ^1.0 range via normal SemVer resolution.
        assert!(
            output.status.success(),
            "fix build failed (A's pin should be ignored): {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        // The resolved version should be in the ^1.0 range, not 0.1.3.
        assert!(
            !lock_content.contains("version = \"0.1.3\""),
            "Lock file should NOT show version 0.1.3 (A's pin should be ignored): {}",
            lock_content
        );
    }

    // Test 10: Both root and A pin with different tags → Root's pin is used, A's pin is ignored.
    #[test]
    fn test_transitive_both_pin() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_both_pin");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("7602fba"),
            "Lock file should contain rev for Root's tag 1.1.0"
        );
        assert!(
            lock_content.contains("version = \"1.1.0\""),
            "Lock file should show version 1.1.0 (Root's pin)"
        );
    }

    // Test: an annotated tag pinned via `tag = ...`.
    // Regression: previously the tag-foreach path stored the tag-object OID,
    // not the underlying commit; with `tag` set this hits resolve_pinned_ref
    // (which already peels) but we cover it for completeness.
    #[test]
    fn test_git_annotated_tag_pinned() {
        install_fix();

        let upstream_tmp = TempDir::new().expect("Failed to create upstream temp dir");
        create_annotated_tag_upstream(upstream_tmp.path(), "1.0.0");

        let consumer_tmp = TempDir::new().expect("Failed to create consumer temp dir");
        let consumer_dir = consumer_tmp.path().join("consumer");
        write_consumer_project(&consumer_dir, upstream_tmp.path(), Some("1.0.0"), None);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&consumer_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test: an unpinned dep that resolves through SemVer-tag enumeration.
    // This is the path that exercises get_versions_from_repo's tag_foreach;
    // before the fix, the stored OID was a tag-object OID and a later
    // find_commit failed for annotated tags.
    #[test]
    fn test_git_annotated_tag_version_resolution() {
        install_fix();

        let upstream_tmp = TempDir::new().expect("Failed to create upstream temp dir");
        create_annotated_tag_upstream(upstream_tmp.path(), "1.0.0");

        let consumer_tmp = TempDir::new().expect("Failed to create consumer temp dir");
        let consumer_dir = consumer_tmp.path().join("consumer");
        write_consumer_project(&consumer_dir, upstream_tmp.path(), None, Some("1.0"));

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&consumer_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content = fs::read_to_string(consumer_dir.join(LOCK_FILE_PATH))
            .expect("Lock file not found");
        assert!(
            lock_content.contains("version = \"1.0.0\""),
            "Lock file should show resolved version 1.0.0:\n{}",
            lock_content
        );
    }
}
