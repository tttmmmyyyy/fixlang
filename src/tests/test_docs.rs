// ==================== Integration Tests for `fix docs` Command ====================
// These tests use actual Fix projects in src/tests/test_docs/

use crate::tests::test_util::install_fix;
use std::process::Command;

#[test]
pub fn test_generate_documents() {
    install_fix();

    // Run `fix doc -m Std` in `std_doc` directory.
    let _ = Command::new("fix")
        .arg("docs")
        .arg("-m")
        .arg("Std")
        .arg("-o")
        .arg(".")
        .current_dir("std_doc")
        .output()
        .expect("Failed to run fix doc.");
}

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use std::{fs, path::PathBuf, process::Command};
    use tempfile::TempDir;

    // Get the path to the test project directory
    fn get_test_project_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_docs");
        path
    }

    // Create a temporary test environment with copied project files
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_project_src = get_test_project_dir();
        let test_project_dst = temp_dir.path().join("test_docs_project");

        // Copy test project directory
        copy_dir_recursive(&test_project_src, &test_project_dst)
            .expect("Failed to copy test project");

        (temp_dir, test_project_dst)
    }

    // Clean up generated documentation before running test
    fn cleanup_test_docs(project_dir: &PathBuf) {
        let docs_dir = project_dir.join("docs");
        if docs_dir.exists() {
            let _ = fs::remove_dir_all(&docs_dir);
        }
    }

    #[test]
    fn test_docs_default_mode() {
        // This test verifies that `fix docs` (without --test flag):
        // 1. Generates documentation only for Main module
        // 2. Does NOT generate documentation for Test module

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs` in the test project directory
        let output = Command::new("fix")
            .arg("docs")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix docs");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs command failed");
        }

        // Verify docs directory exists
        let docs_dir = project_dir.join("docs");
        assert!(
            docs_dir.exists(),
            "docs directory should be created by `fix docs`"
        );

        // Verify Main.md exists
        let main_md = docs_dir.join("Main.md");
        assert!(
            main_md.exists(),
            "Main.md should be generated in default mode"
        );

        // Verify Test.md does NOT exist
        let test_md = docs_dir.join("Test.md");
        assert!(
            !test_md.exists(),
            "Test.md should NOT be generated in default mode (without --test flag)"
        );

        // Verify Main.md contains expected content
        let main_content = fs::read_to_string(&main_md).expect("Failed to read Main.md");
        assert!(
            main_content.contains("hello"),
            "Main.md should contain 'hello' function"
        );
    }

    #[test]
    fn test_docs_test_mode() {
        // This test verifies that `fix docs --test`:
        // 1. Generates documentation for Main module
        // 2. Also generates documentation for Test module

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs --test` in the test project directory
        let output = Command::new("fix")
            .args(&["docs", "--test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix docs --test");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs --test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs --test command failed");
        }

        // Verify docs directory exists
        let docs_dir = project_dir.join("docs");
        assert!(
            docs_dir.exists(),
            "docs directory should be created by `fix docs --test`"
        );

        // Verify Main.md exists
        let main_md = docs_dir.join("Main.md");
        assert!(
            main_md.exists(),
            "Main.md should be generated with --test flag"
        );

        // Verify Test.md exists
        let test_md = docs_dir.join("Test.md");
        assert!(
            test_md.exists(),
            "Test.md should be generated with --test flag"
        );

        // Verify Main.md contains expected content
        let main_content = fs::read_to_string(&main_md).expect("Failed to read Main.md");
        assert!(
            main_content.contains("hello"),
            "Main.md should contain 'hello' function"
        );

        // Verify Test.md contains expected content
        let test_content = fs::read_to_string(&test_md).expect("Failed to read Test.md");
        assert!(
            test_content.contains("test_helper"),
            "Test.md should contain 'test_helper' function"
        );
    }

    #[test]
    fn test_docs_test_mode_specific_module() {
        // This test verifies that `fix docs --test --mods Test`:
        // 1. Generates documentation only for Test module
        // 2. Does NOT generate documentation for Main module

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs --test --mods Test` in the test project directory
        let output = Command::new("fix")
            .args(&["docs", "--test", "--mods", "Test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix docs --test --mods Test");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs --test --mods Test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs --test --mods Test command failed");
        }

        // Verify docs directory exists
        let docs_dir = project_dir.join("docs");
        assert!(
            docs_dir.exists(),
            "docs directory should be created by `fix docs --test --mods Test`"
        );

        // Verify Test.md exists
        let test_md = docs_dir.join("Test.md");
        assert!(
            test_md.exists(),
            "Test.md should be generated when specified with --mods"
        );

        // Verify Main.md does NOT exist
        let main_md = docs_dir.join("Main.md");
        assert!(
            !main_md.exists(),
            "Main.md should NOT be generated when only Test is specified with --mods"
        );

        // Verify Test.md contains expected content
        let test_content = fs::read_to_string(&test_md).expect("Failed to read Test.md");
        assert!(
            test_content.contains("test_helper"),
            "Test.md should contain 'test_helper' function"
        );
    }

    #[test]
    fn test_docs_comprehensive_output() {
        // This test verifies that `fix docs` generates documentation
        // that matches the expected output for a comprehensive test case
        // containing various language features (structs, unions, traits, type aliases, etc.)

        install_fix();

        // Set up test environment with comprehensive test case
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_project_dir().join("cases/comprehensive_docs");
        let test_case_dst = temp_dir.path().join("comprehensive_docs");

        // Copy test case directory
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");

        // Clean up any existing generated documentation
        cleanup_test_docs(&test_case_dst);

        // Run `fix docs` in the test case directory
        let output = Command::new("fix")
            .arg("docs")
            .current_dir(&test_case_dst)
            .output()
            .expect("Failed to execute fix docs");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs command failed");
        }

        // Read expected and actual documentation
        let expected_doc_path = test_case_dst.join("expected_docs/Main.md");
        let actual_doc_path = test_case_dst.join("docs/Main.md");

        assert!(
            expected_doc_path.exists(),
            "Expected documentation file should exist at {:?}",
            expected_doc_path
        );
        assert!(
            actual_doc_path.exists(),
            "Generated documentation file should exist at {:?}",
            actual_doc_path
        );

        let expected_content =
            fs::read_to_string(&expected_doc_path).expect("Failed to read expected documentation");
        let actual_content =
            fs::read_to_string(&actual_doc_path).expect("Failed to read generated documentation");

        // Compare the contents
        assert_eq!(
            actual_content, expected_content,
            "Generated documentation does not match expected output.\n\
            Expected file: {:?}\n\
            Actual file: {:?}\n\
            \n\
            If the difference is intentional, update the expected documentation by running:\n\
            cd src/tests/test_docs/cases/comprehensive_docs && fix docs && cp docs/Main.md expected_docs/Main.md",
            expected_doc_path, actual_doc_path
        );
    }

    #[test]
    fn test_docs_with_compiler_defined_methods() {
        // This test verifies that `fix docs --with-compiler-defined-methods`:
        // 1. Does not panic (regression: compiler-defined methods have syn_scm = None).
        // 2. Includes accessors for public fields/variants.
        // 3. Excludes accessors for private (underscore-prefixed) fields/variants.

        install_fix();

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_project_dir().join("cases/comprehensive_docs");
        let test_case_dst = temp_dir.path().join("comprehensive_docs");
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
        cleanup_test_docs(&test_case_dst);

        let output = Command::new("fix")
            .args(&["docs", "--with-compiler-defined-methods"])
            .current_dir(&test_case_dst)
            .output()
            .expect("Failed to execute fix docs --with-compiler-defined-methods");

        if !output.status.success() {
            eprintln!("fix docs --with-compiler-defined-methods failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs --with-compiler-defined-methods command failed");
        }

        let main_md = test_case_dst.join("docs/Main.md");
        let content = fs::read_to_string(&main_md).expect("Failed to read Main.md");

        // Public struct field `field` of `MyStruct`: accessors should be present.
        for name in &["@field", "set_field", "mod_field", "act_field"] {
            assert!(
                content.contains(&format!("#### {}", name)),
                "Expected accessor `{}` for public field to appear in docs",
                name
            );
        }
        // Public union variant `variant` of `MyUnion`: accessors should be present.
        for name in &["as_variant", "is_variant", "mod_variant"] {
            assert!(
                content.contains(&format!("#### {}", name)),
                "Expected accessor `{}` for public variant to appear in docs",
                name
            );
        }
        // Private struct field `_secret` of `MyStruct`: accessors should be hidden.
        for name in &["@_secret", "set__secret", "mod__secret", "act__secret"] {
            assert!(
                !content.contains(&format!("#### {}", name)),
                "Accessor `{}` for private field should NOT appear in docs",
                name
            );
        }
        // Private union variant `_hidden` of `MyUnion`: accessors should be hidden.
        for name in &["as__hidden", "is__hidden", "mod__hidden"] {
            assert!(
                !content.contains(&format!("#### {}", name)),
                "Accessor `{}` for private variant should NOT appear in docs",
                name
            );
        }
    }

    #[test]
    fn test_docs_with_private_and_compiler_defined_methods() {
        // This test verifies that `fix docs --with-private --with-compiler-defined-methods`
        // un-hides all private items that are otherwise filtered:
        // private top-level values, private field/variant subsections,
        // and accessors for private fields/variants.

        install_fix();

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_project_dir().join("cases/comprehensive_docs");
        let test_case_dst = temp_dir.path().join("comprehensive_docs");
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
        cleanup_test_docs(&test_case_dst);

        let output = Command::new("fix")
            .args(&["docs", "--with-private", "--with-compiler-defined-methods"])
            .current_dir(&test_case_dst)
            .output()
            .expect("Failed to execute fix docs --with-private --with-compiler-defined-methods");

        if !output.status.success() {
            eprintln!("fix docs --with-private --with-compiler-defined-methods failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs --with-private --with-compiler-defined-methods command failed");
        }

        let main_md = test_case_dst.join("docs/Main.md");
        let content = fs::read_to_string(&main_md).expect("Failed to read Main.md");

        // Private top-level value should appear.
        assert!(
            content.contains("#### _private_value"),
            "Private top-level value should appear with --with-private"
        );
        // Private field/variant subsections should appear.
        assert!(
            content.contains("##### field `_secret`"),
            "Private field subsection should appear with --with-private"
        );
        assert!(
            content.contains("##### variant `_hidden`"),
            "Private variant subsection should appear with --with-private"
        );
        // Private struct field accessors should appear.
        for name in &["@_secret", "set__secret", "mod__secret", "act__secret"] {
            assert!(
                content.contains(&format!("#### {}", name)),
                "Accessor `{}` for private field should appear with --with-private",
                name
            );
        }
        // Private union variant accessors should appear.
        for name in &["as__hidden", "is__hidden", "mod__hidden"] {
            assert!(
                content.contains(&format!("#### {}", name)),
                "Accessor `{}` for private variant should appear with --with-private",
                name
            );
        }
    }
}
