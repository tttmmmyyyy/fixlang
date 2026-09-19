//! Integration tests for the `fix docs` command.
//!
//! They run the command over real Fix projects: the `Std` module of the compiler itself, and the
//! projects under `src/tests/test_docs/`.

use crate::tests::test_util::fix_command;

/// `fix docs` documents the `Std` module of the compiler being tested.
///
/// It writes into the `std_doc` project, whose generated `Std.md` the repository carries, so that
/// the file follows a change to `std.fix` or to a document under `src/docs/`.
#[test]
pub fn test_generate_documents() {
    let output = fix_command()
        .arg("docs")
        .arg("-m")
        .arg("Std")
        .arg("-o")
        .arg(".")
        .current_dir("std_doc")
        .output()
        .expect("Failed to run fix doc.");
    assert!(
        output.status.success(),
        "documenting `Std` failed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, fix_command};
    use std::{fs, iter, path::PathBuf};
    use tempfile::TempDir;

    /// The section a generated document gives the value `name`: its heading and the lines under it,
    /// up to the heading that opens the next item; the empty string where the document has no
    /// heading for `name`.
    ///
    /// A value is headed at level four, and a part of one, such as its parameter list, is headed at
    /// level five. So the section ends at the next heading whose level is four or less.
    fn documented_section(document: &str, name: &str) -> String {
        let heading = format!("#### {}", name);
        let opens_an_item = |line: &&str| {
            let heading_level = line.len() - line.trim_start_matches('#').len();
            (1..=4).contains(&heading_level)
        };
        let mut lines = document
            .lines()
            .skip_while(|line| line.trim_end() != heading);
        let Some(heading_line) = lines.next() else {
            return String::new();
        };
        iter::once(heading_line)
            .chain(lines.take_while(|line| !opens_an_item(line)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// A value whose body the compiler supplies has no declaration in a source to carry its
    /// documentation, and takes it from the text `Program::add_global_value` is given instead.
    /// `fix docs` renders that text under the value's own namespace, with the type, what the value
    /// does, and every parameter it names.
    #[test]
    fn test_a_value_the_compiler_defines_is_documented() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_dir = temp_dir.path().join("std_doc");
        fs::create_dir(&project_dir).expect("Failed to create the copy of the std_doc project");
        // The project's own two files, and nothing else `std_doc` holds: `test_generate_documents`
        // regenerates `Std.md` in that directory and leaves a build directory beside it, and it
        // runs alongside this test.
        let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("std_doc");
        for file in ["fixproj.toml", "main.fix"] {
            fs::copy(source_dir.join(file), project_dir.join(file))
                .unwrap_or_else(|e| panic!("Failed to copy std_doc/{}: {}", file, e));
        }

        let output = fix_command()
            .args(&["docs", "-m", "Std", "-o", "."])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix docs");
        assert!(
            output.status.success(),
            "documenting `Std` failed.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        let document =
            fs::read_to_string(project_dir.join("Std.md")).expect("Failed to read Std.md");

        for (name, type_, summary, parameters) in [
            (
                "add_offset",
                "Std::I64 -> Std::Ptr -> Std::Ptr",
                "Adds an offset to a pointer.",
                ["`offset`", "`ptr`"],
            ),
            (
                "offset_from",
                "Std::Ptr -> Std::Ptr -> Std::I64",
                "The distance in bytes from one pointer to another.",
                ["`origin`", "`ptr`"],
            ),
        ] {
            let section = documented_section(&document, name);
            assert!(
                !section.is_empty(),
                "`Std::Ptr::{}` should be documented in the generated `Std.md`",
                name,
            );
            assert!(
                section.contains(&format!("Type: `{}`", type_)),
                "the documentation of `Std::Ptr::{}` should give its type:\n{}",
                name,
                section,
            );
            assert!(
                section.contains(summary),
                "the documentation of `Std::Ptr::{}` should say what it does:\n{}",
                name,
                section,
            );
            for parameter in parameters {
                assert!(
                    section.contains(parameter),
                    "the documentation of `Std::Ptr::{}` should name its parameter {}:\n{}",
                    name,
                    parameter,
                    section,
                );
            }
            // `offset_from` takes two pointers, so the order the two are listed in is the only
            // thing that says which of them the distance is measured from.
            let where_named = parameters
                .iter()
                .map(|parameter| {
                    section
                        .find(parameter)
                        .expect("the parameter is named above")
                })
                .collect::<Vec<_>>();
            assert!(
                where_named.windows(2).all(|pair| pair[0] < pair[1]),
                "the documentation of `Std::Ptr::{}` should name its parameters in the order the \
                 value takes them, {:?}:\n{}",
                name,
                parameters,
                section,
            );
        }
    }

    /// The directory of the Fix project the tests document: `Main` in its build, `Test` in its test
    /// build, and the projects of `cases/` beneath it.
    fn get_test_project_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_docs");
        path
    }

    /// A copy of the project `get_test_project_dir` names, in a temporary directory of its own so
    /// that tests running at the same time each document their own copy: the directory, which
    /// deletes the copy once it is dropped, and the path of the copy.
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_project_src = get_test_project_dir();
        let test_project_dst = temp_dir.path().join("test_docs_project");

        // Copy test project directory
        copy_dir_recursive(&test_project_src, &test_project_dst)
            .expect("Failed to copy test project");

        (temp_dir, test_project_dst)
    }

    /// Removes a project's `docs` directory, so that what a later run of `fix docs` writes is the
    /// whole of its content.
    fn cleanup_test_docs(project_dir: &PathBuf) {
        let docs_dir = project_dir.join("docs");
        if docs_dir.exists() {
            let _ = fs::remove_dir_all(&docs_dir);
        }
    }

    /// `fix docs` documents the modules of the build alone: it writes `docs/Main.md`, holding
    /// `hello`, and the test module `Test` is documented only when it is asked for.
    #[test]
    fn test_docs_default_mode() {
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs` in the test project directory
        let output = fix_command()
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

    /// `--test` documents the test modules alongside those of the build: `docs/Main.md` holds
    /// `hello` and `docs/Test.md` holds `test_helper`.
    #[test]
    fn test_docs_test_mode() {
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs --test` in the test project directory
        let output = fix_command()
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

    /// `--mods` picks the modules to document: naming `Test` alone writes `docs/Test.md`, holding
    /// `test_helper`, and that document alone.
    #[test]
    fn test_docs_test_mode_specific_module() {
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs --test --mods Test` in the test project directory
        let output = fix_command()
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

    /// The document `fix docs` generates for a project exercising structs, unions, traits and type
    /// aliases matches `expected_docs/Main.md` character for character, so a change in how any of
    /// those is rendered shows up as a difference against the file the repository carries.
    #[test]
    fn test_docs_comprehensive_output() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_project_dir().join("cases/comprehensive_docs");
        let test_case_dst = temp_dir.path().join("comprehensive_docs");

        // Copy test case directory
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");

        // Clean up any existing generated documentation
        cleanup_test_docs(&test_case_dst);

        // Run `fix docs` in the test case directory
        let output = fix_command()
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

    /// `--with-compiler-defined-methods` documents the accessors the compiler defines, which carry
    /// no syntactic type scheme: `@field`, `set_field`, `mod_field` and `act_field` of a public
    /// struct field, and `as_variant`, `is_variant` and `mod_variant` of a public union variant.
    /// The accessors of a field or variant whose name opens with an underscore stay private.
    #[test]
    fn test_docs_with_compiler_defined_methods() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_project_dir().join("cases/comprehensive_docs");
        let test_case_dst = temp_dir.path().join("comprehensive_docs");
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
        cleanup_test_docs(&test_case_dst);

        let output = fix_command()
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

    /// `--with-private` documents the items an opening underscore keeps private: a top-level
    /// value, the subsection of a struct field and that of a union variant, and, together with
    /// `--with-compiler-defined-methods`, the accessors of that field and that variant.
    #[test]
    fn test_docs_with_private_and_compiler_defined_methods() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_project_dir().join("cases/comprehensive_docs");
        let test_case_dst = temp_dir.path().join("comprehensive_docs");
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
        cleanup_test_docs(&test_case_dst);

        let output = fix_command()
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
