// ==================== Integration Tests ====================
// These tests use actual Fix projects in src/tests/test_dependencies/cases/

#[cfg(test)]
mod integration_tests {
    use crate::constants::{LOCK_FILE_PATH, LOCK_FILE_TEST_PATH};
    use crate::tests::test_util::{copy_dir_recursive, fix_command};
    use std::{fs, path::PathBuf, process::Output};
    use tempfile::TempDir;

    // Get the path to the test cases directory
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_dependencies/cases");
        path
    }

    /// Copy the test cases into a temporary directory and return it beside the directory of the
    /// project at `project_path` within it. The whole set is copied every time, so that a project
    /// reaches the ones it depends on by the relative paths its project file writes.
    fn setup_case_env(project_path: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_cases_dst = temp_dir.path().to_path_buf();
        copy_dir_recursive(&get_test_cases_dir(), &test_cases_dst)
            .expect("Failed to copy test cases");
        let project_dir = test_cases_dst.join(project_path);
        (temp_dir, project_dir)
    }

    /// The `fix <subcommand>` run of the project at `project_path`, in a temporary copy of the test
    /// cases. The temporary directory is returned so that it outlives the output.
    fn run_case(project_path: &str, subcommand: &str) -> (TempDir, Output) {
        let (temp_dir, project_dir) = setup_case_env(project_path);
        cleanup_test_project(&project_dir);
        let output = fix_command()
            .arg(subcommand)
            .current_dir(&project_dir)
            .output()
            .unwrap_or_else(|err| panic!("Failed to execute fix {}: {}", subcommand, err));
        assert!(
            output.status.success(),
            "fix {} failed:\nstdout: {}\nstderr: {}",
            subcommand,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        (temp_dir, output)
    }

    /// The build of the project at `project_path`, run in a temporary copy of the test cases.
    fn build_case(project_path: &str) -> (TempDir, Output) {
        run_case(project_path, "build")
    }

    // Clean up lock files and build artifacts before running test
    fn cleanup_test_project(project_dir: &PathBuf) {
        let _ = fs::remove_file(project_dir.join(LOCK_FILE_PATH));
        let _ = fs::remove_file(project_dir.join(LOCK_FILE_TEST_PATH));
        let _ = fix_command().arg("clean").current_dir(project_dir).output();
    }

    #[test]
    fn test_dependencies_build_mode() {
        // This test verifies that in build mode:
        // 1. Only fixdeps.lock is created
        // 2. fixdeps.test.lock is NOT created
        // 3. Only normal dependencies are included
        // 4. Test dependencies of normal dependencies are NOT included

        let (_temp_dir, project_dir) = setup_case_env("dependencies_for_test/main_project");
        cleanup_test_project(&project_dir);

        // Run `fix build` in the test project directory
        let output = fix_command()
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix build failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix build command failed");
        }

        // Verify fixdeps.lock exists
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            lock_file.exists(),
            "fixdeps.lock should be created in build mode"
        );

        // Verify fixdeps.test.lock does NOT exist
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            !test_lock_file.exists(),
            "fixdeps.test.lock should NOT be created in build mode"
        );

        // Read and verify lock file contents
        let lock_content = fs::read_to_string(&lock_file).expect("Failed to read lock file");

        // Check that normal-dep is included
        assert!(
            lock_content.contains("normal-dep"),
            "Lock file should contain normal-dep"
        );

        // Check that test-dep is NOT included (neither as main project's test dependency
        // nor as normal-dep's test dependency)
        assert!(
            !lock_content.contains("test-dep"),
            "Lock file should NOT contain test-dep in build mode (test dependencies of dependencies should also be excluded)"
        );
    }

    #[test]
    fn test_dependencies_test_mode() {
        // This test verifies that `fix test` automatically handles test dependencies:
        // 1. fixdeps.test.lock is created if not present
        // 2. Test dependencies are properly available during test execution
        // Note: test-dep appears in fixdeps.test.lock because main-project directly depends on it,
        // not because normal-dep has it as a test dependency (dependency's test dependencies don't propagate)

        let (_temp_dir, project_dir) = setup_case_env("dependencies_for_test/main_project");
        cleanup_test_project(&project_dir);

        // Run `fix test` directly (should auto-generate lock file and install dependencies)
        let output = fix_command()
            .arg("test")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix test");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix test command failed");
        }

        // Verify fixdeps.test.lock was created
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            test_lock_file.exists(),
            "fixdeps.test.lock should be created by `fix test`"
        );

        // Verify fixdeps.lock was NOT created
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            !lock_file.exists(),
            "fixdeps.lock should NOT be created by `fix test`"
        );

        // Read and verify test lock file contents
        let test_lock_content =
            fs::read_to_string(&test_lock_file).expect("Failed to read test lock file");

        // Check that both dependencies are included in test lock file
        assert!(
            test_lock_content.contains("normal-dep"),
            "Test lock file should contain normal-dep"
        );
        assert!(
            test_lock_content.contains("test-dep"),
            "Test lock file should contain test-dep"
        );

        // Verify the test output shows success
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("PASS"),
            "Test should pass with correct output"
        );
    }

    #[test]
    fn test_dependencies_build_workflow() {
        // This test verifies the explicit build workflow:
        // `fix deps update` → `fix deps install` → `fix build`

        let (_temp_dir, project_dir) = setup_case_env("dependencies_for_test/main_project");
        cleanup_test_project(&project_dir);

        // Step 1: Update dependencies
        let update_output = fix_command()
            .args(&["deps", "update"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps update");

        if !update_output.status.success() {
            eprintln!("fix deps update failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&update_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&update_output.stderr));
            panic!("fix deps update command failed");
        }

        // Verify fixdeps.lock was created
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            lock_file.exists(),
            "fixdeps.lock should be created by `fix deps update`"
        );

        // Verify fixdeps.test.lock was NOT created
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            !test_lock_file.exists(),
            "fixdeps.test.lock should NOT be created by `fix deps update` (without --test)"
        );

        // Step 2: Install dependencies
        let install_output = fix_command()
            .args(&["deps", "install"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps install");

        if !install_output.status.success() {
            eprintln!("fix deps install failed:");
            eprintln!(
                "stdout: {}",
                String::from_utf8_lossy(&install_output.stdout)
            );
            eprintln!(
                "stderr: {}",
                String::from_utf8_lossy(&install_output.stderr)
            );
            panic!("fix deps install command failed");
        }

        // Step 3: Build
        let build_output = fix_command()
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build");

        if !build_output.status.success() {
            eprintln!("fix build failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&build_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&build_output.stderr));
            panic!("fix build command failed");
        }

        // Verify lock file contents
        let lock_content = fs::read_to_string(&lock_file).expect("Failed to read lock file");
        assert!(
            lock_content.contains("normal-dep"),
            "Lock file should contain normal-dep"
        );
        assert!(
            !lock_content.contains("test-dep"),
            "Lock file should NOT contain test-dep (neither as main project's test dependency nor as normal-dep's test dependency)"
        );
    }

    #[test]
    fn test_dependencies_test_workflow() {
        // This test verifies the explicit test workflow:
        // `fix deps update --test` → `fix deps install --test` → `fix test`

        let (_temp_dir, project_dir) = setup_case_env("dependencies_for_test/main_project");
        cleanup_test_project(&project_dir);

        // Step 1: Update test dependencies
        let update_output = fix_command()
            .args(&["deps", "update", "--test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps update --test");

        if !update_output.status.success() {
            eprintln!("fix deps update --test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&update_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&update_output.stderr));
            panic!("fix deps update --test command failed");
        }

        // Verify fixdeps.test.lock was created
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            test_lock_file.exists(),
            "fixdeps.test.lock should be created by `fix deps update --test`"
        );

        // Verify fixdeps.lock was NOT created
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            !lock_file.exists(),
            "fixdeps.lock should NOT be created by `fix deps update --test`"
        );

        // Step 2: Install test dependencies
        let install_output = fix_command()
            .args(&["deps", "install", "--test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps install --test");

        if !install_output.status.success() {
            eprintln!("fix deps install --test failed:");
            eprintln!(
                "stdout: {}",
                String::from_utf8_lossy(&install_output.stdout)
            );
            eprintln!(
                "stderr: {}",
                String::from_utf8_lossy(&install_output.stderr)
            );
            panic!("fix deps install --test command failed");
        }

        // Step 3: Run test
        let test_output = fix_command()
            .arg("test")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix test");

        if !test_output.status.success() {
            eprintln!("fix test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&test_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&test_output.stderr));
            panic!("fix test command failed");
        }

        // Verify test lock file contents
        let test_lock_content =
            fs::read_to_string(&test_lock_file).expect("Failed to read test lock file");
        assert!(
            test_lock_content.contains("normal-dep"),
            "Test lock file should contain normal-dep"
        );
        assert!(
            test_lock_content.contains("test-dep"),
            "Test lock file should contain test-dep"
        );

        // Verify test output
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        assert!(
            stdout.contains("PASS"),
            "Test should pass with correct output"
        );
    }

    /// A project importing a module of a project it does not declare is warned about, and the
    /// dependency it does declare is not. `root` declares `undeclared-depa` alone, and imports both
    /// `DepA` (of that project) and `DepB` (of `undeclared-depb`, which `undeclared-depa` declares).
    #[test]
    fn test_import_of_undeclared_transitive_dependency_warns() {
        let (_temp_dir, output) = build_case("undeclared_dependency/root");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(
            stderr.matches("does not declare as a dependency").count(),
            1,
            "the import of `DepB` is the one import to warn about, and the import of `DepA` is \
             declared:\n{}",
            stderr
        );
        assert!(
            stderr.contains(
                "Module `DepB` belongs to the project \"undeclared-depb\", which the project \
                 \"undeclared-root\" does not declare as a dependency."
            ),
            "the warning names the module, the project that provides it, and the project that \
             imports it:\n{}",
            stderr
        );
        assert!(
            stderr.contains("import DepB;"),
            "the warning points at the import statement:\n{}",
            stderr
        );
    }

    /// An absolute path reaches a module without an import statement written for it, and is warned
    /// about the same way. `root_abs` imports `DepA` and writes `::DepB::secret_value`.
    #[test]
    fn test_absolute_path_to_undeclared_transitive_dependency_warns() {
        let (_temp_dir, output) = build_case("undeclared_dependency/root_abs");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(
            stderr.matches("does not declare as a dependency").count(),
            1,
            "the absolute path is the one thing to warn about:\n{}",
            stderr
        );
        assert!(
            stderr.contains(
                "Module `DepB` belongs to the project \"undeclared-depb\", which the project \
                 \"undeclared-root-abs\" does not declare as a dependency."
            ),
            "the warning names the module, the project that provides it, and the project that \
             writes the path:\n{}",
            stderr
        );
        assert!(
            stderr.contains("::DepB::secret_value"),
            "the warning points at the absolute path:\n{}",
            stderr
        );
    }

    /// An import that crosses no project boundary needs no declaration: a module of the importing
    /// project itself, and `Std`, whose files belong to no project at all.
    #[test]
    fn test_import_within_one_project_does_not_warn() {
        let (_temp_dir, output) = build_case("undeclared_dependency/one_project");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !stderr.contains("does not declare as a dependency"),
            "a project's own module and `Std` are declared by nothing and warned about by \
             nothing:\n{}",
            stderr
        );
    }

    /// The test sources of a test build are judged by the test declarations. `root_test` declares
    /// `undeclared-depa` as a test dependency alone, and `test.fix` imports both `DepA` (of that
    /// project) and `DepB` (of `undeclared-depb`, which `undeclared-depa` declares).
    #[test]
    fn test_undeclared_dependency_in_test_sources_warns() {
        let (_temp_dir, output) = run_case("undeclared_dependency/root_test", "test");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(
            stderr.matches("does not declare as a dependency").count(),
            1,
            "the import of `DepB` is the one import to warn about, and the test dependency \
             `undeclared-depa` is declared:\n{}",
            stderr
        );
        assert!(
            stderr.contains(
                "Module `DepB` belongs to the project \"undeclared-depb\", which the project \
                 \"undeclared-root-test\" does not declare as a dependency."
            ),
            "the warning names the module, the project that provides it, and the project that \
             imports it:\n{}",
            stderr
        );
        assert!(
            stderr.contains("in \"test.fix\""),
            "the warning points at the import in the test source:\n{}",
            stderr
        );
    }

    /// A test dependency is declared for the test sources alone, so an ordinary source that imports
    /// one is warned about even in a test build, where the module is there to import.
    /// `root_build_uses_test_dep` declares `undeclared-depa` as a test dependency, and its
    /// `main.fix` imports `DepA`.
    #[test]
    fn test_ordinary_source_importing_a_test_dependency_warns() {
        let (_temp_dir, output) =
            run_case("undeclared_dependency/root_build_uses_test_dep", "test");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(
            stderr.matches("does not declare as a dependency").count(),
            1,
            "the import in `main.fix` is the one import to warn about:\n{}",
            stderr
        );
        assert!(
            stderr.contains(
                "Module `DepA` belongs to the project \"undeclared-depa\", which the project \
                 \"undeclared-root-build-uses-test-dep\" does not declare as a dependency."
            ),
            "the warning names the test dependency the ordinary source reached:\n{}",
            stderr
        );
        assert!(
            stderr.contains("in \"main.fix\""),
            "the warning points at the import in the ordinary source:\n{}",
            stderr
        );
    }

    /// One declaration answers every import between two projects, so the two of them stand for one
    /// warning, pointing at the import that comes first in the source. `root_two_imports` imports
    /// `DepB2` and then `DepB`, both of `undeclared-depb`, and declares neither.
    #[test]
    fn test_undeclared_dependency_warns_once_at_the_earliest_import() {
        let (_temp_dir, output) = build_case("undeclared_dependency/root_two_imports");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(
            stderr.matches("does not declare as a dependency").count(),
            1,
            "the two imports of `undeclared-depb` are one project to declare:\n{}",
            stderr
        );
        assert!(
            stderr.contains("Module `DepB2` belongs to the project \"undeclared-depb\""),
            "the warning names the module of the import that comes first, which the later import \
             of `DepB` does not displace:\n{}",
            stderr
        );
    }
}
