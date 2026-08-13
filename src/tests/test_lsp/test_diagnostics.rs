//! Compile errors as the language server publishes them.
//!
//! Most programmers meet an error in their editor rather than in the output of `fix build`, so a
//! check that the compiler reports something says nothing until the same report reaches the editor,
//! anchored to the source it is about.

#[cfg(test)]
mod tests {
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::copy_dir_recursive;
    use serde_json::Value;
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use tempfile::TempDir;

    /// Copies the named case project into a temporary directory and returns it with the project's
    /// path inside it.
    fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cases = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_lsp/cases");
        let project_dir = temp_dir.path().join(project_name);
        copy_dir_recursive(&cases.join(project_name), &project_dir)
            .expect("Failed to copy test case");
        (temp_dir, project_dir)
    }

    /// The diagnostics the server publishes for `file` of the project, after opening and saving it.
    fn diagnostics_of(project_dir: &Path, file: &Path) -> Vec<Value> {
        let mut client = LspClient::new(project_dir).expect("Failed to start LSP");
        client
            .initialize(project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");
        client.open_document(file).expect("Failed to open document");
        client.save_document(file).expect("Failed to save document");
        client.wait_for_server(Duration::from_secs(10));
        client.get_diagnostics(file)
    }

    /// Two implementations of one trait that can apply to the same type are reported in the editor,
    /// on the first `impl`, with the second named as a related location.
    ///
    /// The pair is found once the kinds of the type variables in the implementation heads are set,
    /// which happens later than the rest of the checks on a trait environment; a report that stops
    /// at the compiler's exit status would not tell whether it survives to where it is read.
    #[test]
    fn test_overlapping_instances_are_reported_on_both_impl_lines() {
        let (_temp_dir, project_dir) = setup_test_env("overlapping_instances");
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let overlapping: Vec<&Value> = diagnostics
            .iter()
            .filter(|diag| {
                diag["message"]
                    .as_str()
                    .map_or(false, |m| m.contains("are overlapping"))
            })
            .collect();
        assert_eq!(
            overlapping.len(),
            1,
            "one report is expected, but the diagnostics are {:?}",
            diagnostics
        );
        let diag = overlapping[0];

        // `main.fix` writes the two implementations on the 9th and the 13th line, which the
        // protocol counts from zero.
        assert_eq!(diag["range"]["start"]["line"], 8, "at the first `impl`");
        assert_eq!(diag["severity"], 1, "as an error");
        assert_eq!(
            diag["relatedInformation"][0]["location"]["range"]["start"]["line"], 12,
            "naming the second `impl`, but the report is {:?}",
            diag
        );
        assert!(
            diag["relatedInformation"][0]["location"]["uri"]
                .as_str()
                .expect("the related location carries a URI")
                .ends_with("/main.fix"),
            "the related location is in `main.fix`, but the report is {:?}",
            diag
        );
    }

    /// Assert that `main.fix` of the named case project draws one report of a repeated struct
    /// field, at `line` and `character`, naming the first occurrence of the name at
    /// `first_character` of the same line as a related location. The protocol counts lines and
    /// columns from zero.
    fn assert_sole_duplicate_field_report(
        project_name: &str,
        line: u64,
        character: u64,
        first_character: u64,
    ) {
        let (_temp_dir, project_dir) = setup_test_env(project_name);
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let duplicates: Vec<&Value> = diagnostics
            .iter()
            .filter(|diag| {
                diag["message"]
                    .as_str()
                    .map_or(false, |m| m.contains("Duplicate field"))
            })
            .collect();
        assert_eq!(
            duplicates.len(),
            1,
            "one report is expected, but the diagnostics are {:?}",
            diagnostics
        );
        let diag = duplicates[0];

        assert_eq!(
            diag["range"]["start"]["line"], line,
            "on the field list's line, but the report is {:?}",
            diag
        );
        assert_eq!(
            diag["range"]["start"]["character"], character,
            "at the repeated field name, but the report is {:?}",
            diag
        );
        assert_eq!(diag["severity"], 1, "as an error");
        assert_eq!(
            diag["relatedInformation"][0]["location"]["range"]["start"]["character"],
            first_character,
            "naming the first occurrence, but the report is {:?}",
            diag
        );
    }

    /// A struct literal that gives one field twice is reported in the editor, on the repeated field
    /// name, with the first occurrence of that name as a related location.
    ///
    /// The two names are what the programmer chooses between, so a report anchored to the whole
    /// literal would leave them to find the pair themselves.
    #[test]
    fn test_duplicate_struct_field_is_reported_on_both_field_names() {
        // `main.fix` writes the literal on the 6th line, with the repeated `x` at the 36th column
        // and the first `x` at the 20th.
        assert_sole_duplicate_field_report("duplicate_struct_field", 5, 35, 19);
    }

    /// A struct pattern that matches one field twice is reported in the editor on the repeated
    /// field name, with the first occurrence of that name as a related location, just as a struct
    /// literal is.
    ///
    /// A pattern's field-name spans come from a different parser path than a literal's, so the
    /// literal's report reaching the editor says nothing about the pattern's.
    #[test]
    fn test_duplicate_struct_pattern_field_is_reported_on_both_field_names() {
        // `main.fix` writes the pattern on the 7th line, with the repeated `x` at the 25th column
        // and the first `x` at the 19th.
        assert_sole_duplicate_field_report("duplicate_struct_pattern_field", 6, 24, 18);
    }
}
