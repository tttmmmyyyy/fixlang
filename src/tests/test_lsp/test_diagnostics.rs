//! Compile errors as the language server publishes them.
//!
//! Most programmers meet an error in their editor rather than in the output of `fix build`, so a
//! check that the compiler reports something says nothing until the same report reaches the editor,
//! anchored to the source it is about.

#[cfg(test)]
mod tests {
    use super::super::completion_harness::LspCompletionCtx;
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
        let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_lsp/cases");
        let project_dir = temp_dir.path().join(project_name);
        copy_dir_recursive(&cases_dir.join(project_name), &project_dir)
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

    /// The diagnostic whose message contains `text`, of which the test expects exactly one.
    fn sole_diagnostic_containing<'a>(diagnostics: &'a [Value], text: &str) -> &'a Value {
        let matching: Vec<&Value> = diagnostics
            .iter()
            .filter(|diag| diag["message"].as_str().map_or(false, |m| m.contains(text)))
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "one report containing `{}` is expected, but the diagnostics are {:?}",
            text,
            diagnostics
        );
        matching[0]
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

        let diag = sole_diagnostic_containing(&diagnostics, "are overlapping");

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

    /// Two values whose opaque return types are written in terms of each other are reported in the
    /// editor, on the first declaration, with the second named as a related location.
    ///
    /// Determining the concrete type behind an opaque type is the work of type-checking, so this
    /// report is made where a run for the editor and a run for `fix build` part company; and a
    /// program the editor leaves unreported here is one whose build does not terminate.
    #[test]
    fn test_opaque_types_written_in_terms_of_each_other_are_reported_on_both_declarations() {
        let (_temp_dir, project_dir) = setup_test_env("opaque_type_cycle");
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let diag = sole_diagnostic_containing(&diagnostics, "written in terms of each other");

        // `main.fix` declares `f` on the 3rd line and `g` on the 6th, which the protocol counts
        // from zero.
        assert_eq!(
            diag["range"]["start"]["line"], 2,
            "at the declaration of `f`"
        );
        assert_eq!(diag["severity"], 1, "as an error");
        assert_eq!(
            diag["relatedInformation"][0]["location"]["range"]["start"]["line"], 5,
            "naming the declaration of `g`, but the report is {:?}",
            diag
        );
    }

    /// Assert that `main.fix` of the named case project draws one report of a repeated struct
    /// field, at `line` and `repeat_character`, naming the first occurrence of the name at
    /// `first_occurrence_character` of the same line as a related location. The protocol counts
    /// lines and columns from zero.
    fn assert_sole_duplicate_field_report(
        project_name: &str,
        line: u64,
        repeat_character: u64,
        first_occurrence_character: u64,
    ) {
        let (_temp_dir, project_dir) = setup_test_env(project_name);
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let diag = sole_diagnostic_containing(&diagnostics, "Duplicate field");

        assert_eq!(
            diag["range"]["start"]["line"], line,
            "on the field list's line, but the report is {:?}",
            diag
        );
        assert_eq!(
            diag["range"]["start"]["character"], repeat_character,
            "at the repeated field name, but the report is {:?}",
            diag
        );
        assert_eq!(diag["severity"], 1, "as an error");
        assert_eq!(
            diag["relatedInformation"][0]["location"]["range"]["start"]["character"],
            first_occurrence_character,
            "naming the first occurrence, but the report is {:?}",
            diag
        );
    }

    /// A struct literal that gives one field twice is reported in the editor, on the repeated field
    /// name, with the first occurrence of that name as a related location.
    ///
    /// The programmer chooses between the two names, so the test checks the position of the report
    /// and of its related location.
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

    /// A name the struct does not declare is reported in the editor on that name, in a struct
    /// pattern as in a struct literal.
    ///
    /// The name is the text the programmer has to fix, so a report anchored on the whole literal
    /// would put the squiggle on the fields that are right as well.
    #[test]
    fn test_unknown_struct_field_is_reported_on_the_field_name() {
        let (_temp_dir, project_dir) = setup_test_env("unknown_struct_field");
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let unknown: Vec<&Value> = diagnostics
            .iter()
            .filter(|diag| {
                diag["message"]
                    .as_str()
                    .map_or(false, |m| m.contains("Unknown field"))
            })
            .collect();
        assert_eq!(
            unknown.len(),
            2,
            "the pattern and the literal are both reported, but the diagnostics are {:?}",
            diagnostics
        );

        // `main.fix` writes the pattern's `z` on the 7th line at the 19th column, and the
        // literal's `w` on the 13th line at the 28th; the protocol counts both from zero.
        let pattern_report = unknown
            .iter()
            .find(|diag| diag["message"].as_str().unwrap().contains("`z`"))
            .expect("the pattern's unknown field is reported");
        assert_eq!(pattern_report["range"]["start"]["line"], 6);
        assert_eq!(
            pattern_report["range"]["start"]["character"], 18,
            "at the pattern's field name, but the report is {:?}",
            pattern_report
        );

        let literal_report = unknown
            .iter()
            .find(|diag| diag["message"].as_str().unwrap().contains("`w`"))
            .expect("the literal's unknown field is reported");
        assert_eq!(literal_report["range"]["start"]["line"], 12);
        assert_eq!(
            literal_report["range"]["start"]["character"], 27,
            "at the literal's field name, but the report is {:?}",
            literal_report
        );
    }

    /// A completion request leaves the error of another file reported.
    ///
    /// A completion re-checks the program in error-tolerant mode, which reports no diagnostic
    /// whatever it finds. Should such a run reach an entity of a file the user is not editing and
    /// its result be kept, the next strict run would answer from it and publish that file as
    /// clean — the project would not compile while the editor showed nothing.
    #[test]
    fn test_a_completion_leaves_the_error_of_another_file_reported() {
        let mut ctx =
            LspCompletionCtx::setup("diagnostics-after-completion", &["lib.fix", "main.fix"]);
        let lib_file = Path::new("lib.fix");

        let diagnostics_before = ctx.client.get_diagnostics(lib_file);
        assert_eq!(
            diagnostics_before.len(),
            1,
            "`lib.fix` is expected to be reported before any completion, but its diagnostics are {:?}",
            diagnostics_before
        );

        // The dot of `v.@y.to_string`. The cursor sits in the body of a trait-implementation
        // member, whose symbol the completion cannot narrow its check to — the symbol is built
        // during elaboration, later than the narrowing reads the parsed buffer — so the tolerant
        // run covers every value of the project, the broken implementation in `lib.fix` among
        // them.
        let items = ctx.complete_with_timeout("main.fix", 7, 20, Duration::from_secs(60));

        // A dot completion ranks its candidates, and the tolerant re-check is what ranks them: a
        // reply whose candidates carry no sort key was answered without that run, and says
        // nothing about what such a run leaves behind.
        assert!(
            items.iter().any(|item| item.get("sortText").is_some()),
            "the completion after the dot is expected to rank its candidates, but none of its {} items carries a sort key",
            items.len()
        );

        ctx.client
            .trigger_and_wait_for_diagnostics(Path::new("main.fix"));
        let diagnostics_after = ctx.client.get_diagnostics(lib_file);
        assert_eq!(
            diagnostics_after, diagnostics_before,
            "the same report on `lib.fix` is still expected, but its diagnostics are {:?}",
            diagnostics_after
        );

        ctx.shutdown();
    }
}
