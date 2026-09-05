//! Compile errors as the language server publishes them.
//!
//! Programmers meet a compile error in their editor, so a check that the compiler reports
//! something says nothing until the same report reaches the editor, anchored to the source it is
//! about.

#[cfg(test)]
mod tests {
    use super::super::completion_harness::LspCompletionCtx;
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::copy_dir_recursive;
    use serde_json::{json, Value};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};
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
        let mut client = open_session(project_dir, file, Duration::from_secs(5));
        client.save_document(file).expect("Failed to save document");
        client.wait_for_server(Duration::from_secs(10));
        client.get_diagnostics(file)
    }

    /// The diagnostics whose message contains `text`.
    fn diagnostics_containing<'a>(diagnostics: &'a [Value], text: &str) -> Vec<&'a Value> {
        diagnostics
            .iter()
            .filter(|diag| diag["message"].as_str().map_or(false, |m| m.contains(text)))
            .collect()
    }

    /// The diagnostic whose message contains `text`, of which the test expects exactly one.
    fn sole_diagnostic_containing<'a>(diagnostics: &'a [Value], text: &str) -> &'a Value {
        let matching = diagnostics_containing(diagnostics, text);
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

    /// A trait member and a value written under a namespace named after the trait carry one name,
    /// and the collision is reported in the editor, on the member's declaration, with the value's
    /// declaration named as a related location.
    ///
    /// Either of the two names is the one to change, so the report has to reach the editor
    /// anchored to both of them.
    #[test]
    fn test_trait_member_and_value_of_the_traits_namespace_are_reported_on_both_declarations() {
        let (_temp_dir, project_dir) = setup_test_env("trait_member_name_collision");
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let diag = sole_diagnostic_containing(
            &diagnostics,
            "Duplicate definition for global value: `Main::Foo::bar`.",
        );

        // `main.fix` declares the member on the 4th line and the value on the 12th, each at the
        // 5th column, which the protocol counts from zero.
        assert_eq!(
            diag["range"]["start"]["line"], 3,
            "at the member's declaration, but the report is {:?}",
            diag
        );
        assert_eq!(diag["range"]["start"]["character"], 4);
        assert_eq!(diag["severity"], 1, "as an error");
        assert_eq!(
            diag["relatedInformation"][0]["location"]["range"]["start"]["line"], 11,
            "naming the value's declaration, but the report is {:?}",
            diag
        );
    }

    /// Two traits, the full name of one ending with the full name of the other, are both declared,
    /// and a reference that could mean either one is reported as ambiguous in the editor, on the
    /// reference, naming both.
    ///
    /// The pair is met while the namespaces of the declarations are resolved, which every run of
    /// the diagnostics performs; a program that stops the compiler there takes the editor's reports
    /// on every other file down with it, for the rest of the session.
    #[test]
    fn test_a_reference_to_a_trait_whose_name_another_ends_with_is_reported_as_ambiguous() {
        let (_temp_dir, project_dir) = setup_test_env("trait_name_suffix_collision");
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let diag = sole_diagnostic_containing(&diagnostics, "Name `Foo` is ambiguous");

        // `main.fix` writes the implementation's head on the 8th line, whose `I64 : Foo` starts at
        // the 6th column; the protocol counts both from zero.
        assert_eq!(
            diag["range"]["start"]["line"], 7,
            "at the implementation's head, but the report is {:?}",
            diag
        );
        assert_eq!(diag["range"]["start"]["character"], 5);
        assert_eq!(diag["severity"], 1, "as an error");
        let message = diag["message"]
            .as_str()
            .expect("the report carries a message");
        for trait_name in ["`Lib::Main::Foo`", "`Main::Foo`"] {
            assert!(
                message.contains(trait_name),
                "naming {}, but the report is {:?}",
                trait_name,
                diag
            );
        }
    }

    /// A trait member whose type leaves the trait's type variable to a constraint is reported in
    /// the editor, on the member's declaration.
    ///
    /// The member's declaration is the line the programmer has to change, so the report has to
    /// reach the editor anchored there.
    #[test]
    fn test_trait_member_not_fixing_the_trait_variable_is_reported_on_its_declaration() {
        let (_temp_dir, project_dir) = setup_test_env("trait_member_unfixed_variable");
        let diagnostics = diagnostics_of(&project_dir, Path::new("main.fix"));

        let diag = sole_diagnostic_containing(
            &diagnostics,
            "Type variable `c` is not fixed by this type signature",
        );

        // `main.fix` declares the member on the 4th line, at the 5th column, which the protocol
        // counts from zero.
        assert_eq!(
            diag["range"]["start"]["line"], 3,
            "at the member's declaration, but the report is {:?}",
            diag
        );
        assert_eq!(diag["range"]["start"]["character"], 4);
        assert_eq!(diag["severity"], 1, "as an error");
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

        let unknown_field_reports = diagnostics_containing(&diagnostics, "Unknown field");
        assert_eq!(
            unknown_field_reports.len(),
            2,
            "the pattern and the literal are both reported, but the diagnostics are {:?}",
            diagnostics
        );

        // `main.fix` writes the pattern's `z` on the 7th line at the 19th column, and the
        // literal's `w` on the 13th line at the 28th; the protocol counts both from zero.
        let pattern_report = unknown_field_reports
            .iter()
            .find(|diag| diag["message"].as_str().unwrap().contains("`z`"))
            .expect("the pattern's unknown field is reported");
        assert_eq!(pattern_report["range"]["start"]["line"], 6);
        assert_eq!(
            pattern_report["range"]["start"]["character"], 18,
            "at the pattern's field name, but the report is {:?}",
            pattern_report
        );

        let literal_report = unknown_field_reports
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

    /// The time one diagnostics pass is given to end.
    const PASS_TIMEOUT: Duration = Duration::from_secs(180);

    /// A session over the project, with `file` opened. `initialize_timeout` is how long the
    /// response to `initialize` is waited for.
    fn open_session(project_dir: &Path, file: &Path, initialize_timeout: Duration) -> LspClient {
        let mut client = LspClient::new(project_dir).expect("Failed to start LSP");
        client
            .initialize(project_dir, initialize_timeout)
            .expect("Failed to initialize LSP");
        client.open_document(file).expect("Failed to open document");
        client
    }

    /// The diagnostics of `file` once they answer `settled`, and the ones on screen when the wait
    /// runs out.
    ///
    /// An edit asks for a pass, and what ends next can be a pass asked for earlier — the one a
    /// session starts with, or the one a save asked for — so the count of passes that have ended
    /// tells one from the other only where no other pass is in flight. Waiting for the reports
    /// themselves is what does tell them apart. A report that never arrives leaves the assertion
    /// that follows to fail on what is on screen at the end of the wait.
    fn wait_until_diagnostics(
        client: &mut LspClient,
        file: &Path,
        settled: impl Fn(&[Value]) -> bool,
    ) -> Vec<Value> {
        let deadline = Instant::now() + PASS_TIMEOUT;
        loop {
            let diagnostics = client.get_diagnostics(file);
            if settled(&diagnostics) || Instant::now() >= deadline {
                return diagnostics;
            }
            client.wait_for_server(Duration::from_millis(100));
        }
    }

    /// Whether the reports carry the one whose message contains `text`.
    fn carries_report(diagnostics: &[Value], text: &str) -> bool {
        !diagnostics_containing(diagnostics, text).is_empty()
    }

    /// Saves `file` and waits until the pass the save asks for has ended. `expectation` names
    /// what the wait is for, and is shown when the wait times out.
    fn save_and_wait_for_a_pass(client: &mut LspClient, file: &Path, expectation: &str) {
        let passes_before = client.count_progress_end_messages();
        client.save_document(file).expect("Failed to save document");
        client
            .wait_for_progress_end_count(passes_before + 1, PASS_TIMEOUT)
            .expect(expectation);
    }

    /// A program carrying one ordinary error, which the analysis finishes and reports.
    const PROGRAM_WITH_AN_UNKNOWN_NAME: &str =
        "module Main;\n\nmain : IO ();\nmain = println(nonexistent_name);\n";

    /// The error the analysis reports on `PROGRAM_WITH_AN_UNKNOWN_NAME`.
    const UNKNOWN_NAME_REPORT: &str = "Unknown name `nonexistent_name`";

    /// A program the analysis finishes with nothing to report.
    const PROGRAM_WITHOUT_AN_ERROR: &str =
        "module Main;\n\nmain : IO ();\nmain = println(\"x\");\n";

    /// A project directory holding the given files, under a fresh temporary directory.
    ///
    /// The path handed back is canonical, which is what makes the URI a test builds from it the
    /// URI the client builds: a temporary directory reaches its files through a symbolic link on
    /// some systems, and the server keeps one record per URI, so two spellings of one path leave
    /// two records of one file and the analysis reads whichever the map hands it first.
    fn project_with(files: &[(&str, &str)]) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_dir = temp_dir.path().join("proj");
        fs::create_dir_all(&project_dir).expect("Failed to create the project directory");
        for (name, content) in files {
            fs::write(project_dir.join(name), content).expect("Failed to write a project file");
        }
        let project_dir = project_dir
            .canonicalize()
            .expect("Failed to canonicalize the project directory");
        (temp_dir, project_dir)
    }

    /// A diagnostics pass whose analysis ends in a panic ends that pass alone: the program the
    /// editor writes next is analyzed, and its report reaches the editor without the server being
    /// restarted.
    ///
    /// A program being repaired passes through shapes the compiler answers with a panic, and the
    /// case project holds one of them. A panic that takes the diagnostics thread with it leaves
    /// the session with no report on any file, however the program is repaired afterwards.
    #[test]
    fn test_the_next_program_is_analyzed_after_a_pass_panics() {
        let (_temp_dir, project_dir) = setup_test_env("diagnostics_after_panic");
        let main_fix = Path::new("main.fix");

        let mut client = open_session(&project_dir, main_fix, Duration::from_secs(10));
        save_and_wait_for_a_pass(
            &mut client,
            main_fix,
            "the pass over the program that panics is expected to end",
        );

        // What the rest of this test measures exists only after a pass has panicked, so the panic
        // is asserted rather than assumed. A pass that fails publishes nothing, and the case
        // project's program declares one type variable twice, which the compiler reports where it
        // does not panic: a report arriving here says it has learned to analyze the program, and
        // this test then asks for another one that panics.
        let reports = client.get_all_diagnostics();
        assert!(
            reports.values().all(|diagnostics| diagnostics.is_empty()),
            "the analysis of the case project is expected to fail, publishing nothing, but the \
             reports are {:?}",
            reports
        );

        // The repair the editor writes, which carries one ordinary error.
        fs::write(project_dir.join(main_fix), PROGRAM_WITH_AN_UNKNOWN_NAME)
            .expect("Failed to write the repaired program");
        client
            .change_document(main_fix)
            .expect("Failed to change document");
        save_and_wait_for_a_pass(
            &mut client,
            main_fix,
            "the pass over the repaired program is expected to end",
        );

        let diagnostics = wait_until_diagnostics(&mut client, main_fix, |d| {
            carries_report(d, UNKNOWN_NAME_REPORT)
        });
        sole_diagnostic_containing(&diagnostics, UNKNOWN_NAME_REPORT);
    }

    /// The reports of the pass before a failing one stay on the files they name, and the pass that
    /// finishes next takes them back.
    ///
    /// A pass whose analysis fails produces no reports of its own, so what the editor shows is what
    /// the pass before it published. Those files have to stay named for as long as that: a pass
    /// which forgot them would leave their squiggles on screen for the rest of the session, on a
    /// program that no longer carries the error.
    #[test]
    fn test_the_reports_of_the_pass_before_a_failing_one_are_taken_back_by_the_next_one() {
        let (_temp_dir, project_dir) = setup_test_env("diagnostics_after_panic");
        let main_fix = Path::new("main.fix");
        let program_the_analysis_fails_on = fs::read_to_string(project_dir.join(main_fix))
            .expect("Failed to read the case project's program");

        // The program the session starts from, which carries one ordinary error.
        fs::write(project_dir.join(main_fix), PROGRAM_WITH_AN_UNKNOWN_NAME)
            .expect("Failed to write the program the session starts from");

        let mut client = open_session(&project_dir, main_fix, Duration::from_secs(10));
        save_and_wait_for_a_pass(
            &mut client,
            main_fix,
            "the pass over the program carrying an ordinary error is expected to end",
        );
        let diagnostics = wait_until_diagnostics(&mut client, main_fix, |d| {
            carries_report(d, UNKNOWN_NAME_REPORT)
        });
        sole_diagnostic_containing(&diagnostics, UNKNOWN_NAME_REPORT);

        // The program written next, which the analysis fails on.
        fs::write(project_dir.join(main_fix), &program_the_analysis_fails_on)
            .expect("Failed to write the program the analysis fails on");
        save_and_wait_for_a_pass(
            &mut client,
            main_fix,
            "the pass over the program the analysis fails on is expected to end",
        );

        // The report of the pass before, still where that pass put it. It is also what says the
        // pass failed: a pass that analyzed this program would report its duplicated type
        // variable in place of the error of the program before it.
        sole_diagnostic_containing(&client.get_diagnostics(main_fix), UNKNOWN_NAME_REPORT);

        // The program written last, which the analysis finishes with nothing to report.
        fs::write(project_dir.join(main_fix), PROGRAM_WITHOUT_AN_ERROR)
            .expect("Failed to write the program carrying no error");
        save_and_wait_for_a_pass(
            &mut client,
            main_fix,
            "the pass over the program carrying no error is expected to end",
        );

        let diagnostics = wait_until_diagnostics(&mut client, main_fix, |d| d.is_empty());
        assert!(
            diagnostics.is_empty(),
            "the report of the pass before the failing one is expected to be taken back, but \
             `main.fix` carries {:?}",
            diagnostics
        );
    }

    /// A repair the programmer types without saving takes the report back, so the squiggle leaves
    /// the screen as the error does.
    ///
    /// The pass an edit asks for runs over the buffers the editor holds, and the file on disk still
    /// carries the error at that point. A pass reading the disk instead would answer with the error
    /// the programmer has just removed.
    #[test]
    fn test_a_repair_the_editor_has_not_saved_takes_the_report_back() {
        let (_temp_dir, project_dir) = project_with(&[
            (
                "fixproj.toml",
                "[general]\nname = \"unsaved\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"main.fix\"]\n",
            ),
            ("main.fix", PROGRAM_WITH_AN_UNKNOWN_NAME),
        ]);
        let main_fix = Path::new("main.fix");

        let mut client = open_session(&project_dir, main_fix, Duration::from_secs(10));
        save_and_wait_for_a_pass(&mut client, main_fix, "the first pass is expected to end");
        let diagnostics = wait_until_diagnostics(&mut client, main_fix, |d| {
            carries_report(d, UNKNOWN_NAME_REPORT)
        });
        sole_diagnostic_containing(&diagnostics, UNKNOWN_NAME_REPORT);

        // The repair, which stays in the editor: the file on disk keeps the error.
        let uri = format!("file://{}", project_dir.join(main_fix).display());
        let passes_before = client.count_progress_end_messages();
        client
            .send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": uri, "version": 2 },
                    "contentChanges": [ { "text": PROGRAM_WITHOUT_AN_ERROR } ]
                }),
            )
            .expect("Failed to send didChange");
        client
            .wait_for_progress_end_count(passes_before + 1, PASS_TIMEOUT)
            .expect("the pass over the repaired buffer is expected to end");

        let diagnostics = wait_until_diagnostics(&mut client, main_fix, |d| d.is_empty());
        assert!(
            diagnostics.is_empty(),
            "the report is expected to be taken back once the buffer carries no error, but \
             `main.fix` carries {:?}",
            diagnostics
        );
    }
}
