use crate::tests::test_util::fix_command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
pub fn test_edit_explicit_import() {
    // Iterate through the "cases" subdirectory in the directory where this source file is located

    let cases_dir = Path::new(file!()).parent().unwrap().join("cases");
    let entries = fs::read_dir(&cases_dir).expect("Failed to read cases directory");
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        // Skip if the directory name starts with an underscore
        let dir_name = path.file_name().unwrap().to_str().unwrap();
        if dir_name.starts_with('_') {
            continue;
        }
        if path.is_dir() {
            // Run test for each test case directory
            run_test_case(&path);
        }
    }
}

/// Runs one `fix edit explicit-import` case: applies the edit to `main.from.fix`,
/// checks the project still builds, and asserts the result matches `main.to.fix`.
///
/// The case runs in a temporary directory holding a copy of its source files, so that the edited
/// `main.fix` and the build's outputs stay out of the source tree.
pub fn run_test_case(case_path: &Path) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp_dir.path();
    copy_case_sources(case_path, project_dir);
    let to_path = project_dir.join("main.to.fix");
    let target_path = project_dir.join("main.fix");
    fs::copy(project_dir.join("main.from.fix"), &target_path)
        .expect("Failed to copy from main.from.fix to main.fix");

    run_explicit_import(project_dir);

    // Check that "fix build" succeeds after the edit
    let build_output = fix_command()
        .arg("build")
        .current_dir(project_dir)
        .output()
        .expect("Failed to run fix build");
    if !build_output.status.success() {
        panic!(
            "fix build failed after edit:\n{}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Compare main.to.fix and main.fix to verify they match
    let expected_content = fs::read_to_string(&to_path).expect("Failed to read main.to.fix");
    let actual_content = fs::read_to_string(&target_path).expect("Failed to read main.fix");
    assert_eq!(
        expected_content,
        actual_content,
        "Test case failed: {}",
        case_path.display()
    );

    // The import statements the command writes are the ones it reads back, so a second run over
    // the rewritten project leaves the file byte for byte as the first run left it.
    run_explicit_import(project_dir);
    let rerun_content = fs::read_to_string(&target_path).expect("Failed to read main.fix");
    assert_eq!(
        actual_content,
        rerun_content,
        "A second run changed the file: {}",
        case_path.display()
    );
}

/// Copies the files of the case at `case_path` into `project_dir`. A checkout where cases ran in
/// place holds the edited `main.fix`, the `a.out` and the `.fixlang` directory of those runs beside
/// the sources; those are left behind.
fn copy_case_sources(case_path: &Path, project_dir: &Path) {
    for entry in fs::read_dir(case_path).expect("Failed to read the case directory") {
        let entry = entry.expect("Failed to read directory entry");
        let name = entry.file_name();
        let is_file = entry
            .file_type()
            .expect("Failed to read the file type")
            .is_file();
        if is_file && name != "main.fix" && name != "a.out" {
            fs::copy(entry.path(), project_dir.join(&name)).expect("Failed to copy a case file");
        }
    }
}

/// Runs `fix edit explicit-import` in `project_dir`, panicking with the command's stderr on
/// failure.
fn run_explicit_import(project_dir: &Path) {
    let output = fix_command()
        .arg("edit")
        .arg("explicit-import")
        .current_dir(project_dir)
        .output()
        .expect("Failed to run fix edit explicit-import");
    if !output.status.success() {
        panic!(
            "fix edit explicit-import failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
