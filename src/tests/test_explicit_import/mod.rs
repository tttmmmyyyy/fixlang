use crate::tests::util::install_fix;

#[test]
pub fn test_edit_explicit_import() {
    install_fix();
    // Iterate through the "cases" subdirectory in the directory where this source file is located

    let cases_dir = std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join("cases");
    let entries = std::fs::read_dir(&cases_dir).expect("Failed to read cases directory");
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

pub fn run_test_case(case_path: &std::path::Path) {
    // Copy main.from.fix to main.fix (create if it doesn't exist)
    let from_path = case_path.join("main.from.fix");
    let to_path = case_path.join("main.to.fix");
    let target_path = case_path.join("main.fix");
    std::fs::copy(&from_path, &target_path).expect("Failed to copy from main.from.fix to main.fix");

    // Execute fix edit explicit-import
    let output = std::process::Command::new("fix")
        .arg("edit")
        .arg("explicit-import")
        .current_dir(case_path)
        .output()
        .expect("Failed to run fix edit explicit-import");
    if !output.status.success() {
        panic!(
            "fix edit explicit-import failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Check that "fix build" succeeds after the edit
    let build_output = std::process::Command::new("fix")
        .arg("build")
        .current_dir(case_path)
        .output()
        .expect("Failed to run fix build");
    if !build_output.status.success() {
        panic!(
            "fix build failed after edit:\n{}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Compare main.to.fix and main.fix to verify they match
    let expected_content = std::fs::read_to_string(&to_path).expect("Failed to read main.to.fix");
    let actual_content = std::fs::read_to_string(&target_path).expect("Failed to read main.fix");
    assert_eq!(
        expected_content,
        actual_content,
        "Test case failed: {}",
        case_path.display()
    );
}
