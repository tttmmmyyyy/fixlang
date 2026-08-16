// Tests for the RC IR term simplifier: what it removes, read from the `--emit-rc-ir` dump, and what
// the simplified program computes.
//
// A read loop over `range(0, size).fold` lowers to a specialized fold driver whose loop-carried state
// is the `Option` that `range`'s `advance` builds and `fold` immediately matches. The simplifier
// cancels that union (case-of-case + case-of-known-constructor), so the driver keeps only the plain
// `RangeIterator` two-scalar state and no union construction — the property the integration tests
// assert. The value tests compile and run Fix programs written to drive the same rewrite, and check
// what each one computes. The build-time tests bound how long a program shaped to drive it may take
// to compile.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, fix_command_at_opt_level};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    /// The directory holding the case projects, `src/tests/test_simplify/cases` in the source tree.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_simplify/cases");
        path
    }

    /// Copy the test cases into a fresh temporary directory so parallel test runs do not conflict,
    /// and return the directory of the named case project.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dst = temp_dir.path().to_path_buf();
        copy_dir_recursive(&get_test_cases_dir(), &dst).expect("Failed to copy test cases");
        let project_dir = dst.join(case);
        (temp_dir, project_dir)
    }

    /// Build the case project at `max` (where the simplifier runs) with `--emit-rc-ir all`,
    /// returning the dumped RC IR of every module. The `range.fold` driver is a specialized
    /// `Std::Iterator` symbol, so the whole-program dump is needed to see it. Also leaves a runnable
    /// executable.
    fn emit_all_rc_ir(project_dir: &Path) -> String {
        let output = fix_command_at_opt_level("build", "max")
            .arg("--emit-rc-ir")
            .arg("all")
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix build --emit-rc-ir");

        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix build --emit-rc-ir failed");
        }

        let dump_path = project_dir.join(".fixlang/rc_ir.post.txt");
        std::fs::read_to_string(&dump_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dump_path.display(), e))
    }

    /// Each `fn` block of the dump whose header line contains all of `needles`: the header line
    /// itself and every line up to the next `fn` header.
    fn fn_bodies_matching<'a>(dump: &'a str, needles: &[&str]) -> Vec<String> {
        let mut bodies = Vec::new();
        let mut current: Option<String> = None;
        for line in dump.lines() {
            if line.starts_with("fn ") {
                if let Some(body) = current.take() {
                    bodies.push(body);
                }
                if needles.iter().all(|n| line.contains(n)) {
                    current = Some(String::new());
                }
            }
            if let Some(body) = current.as_mut() {
                body.push_str(line);
                body.push('\n');
            }
        }
        if let Some(body) = current.take() {
            bodies.push(body);
        }
        bodies
    }

    /// Build the case project and assert that every `fn` of its RC IR dump whose header matches
    /// `needles` — which `what` names in a failure message — builds no union, and that the
    /// executable the build leaves prints `expected`. The dump shows what the simplifier removed,
    /// and the run shows the removal left what the program computes intact.
    fn assert_union_cancelled(case: &str, needles: &[&str], what: &str, expected: &str) {
        let (_temp_dir, project_dir) = setup_test_env(case);
        let dump = emit_all_rc_ir(&project_dir);

        let bodies = fn_bodies_matching(&dump, needles);
        assert!(
            !bodies.is_empty(),
            "no {} in the RC IR dump:\n{}",
            what,
            dump
        );
        for body in &bodies {
            assert!(
                !body.contains("union_"),
                "{} still builds a union — the simplifier did not cancel it:\n{}",
                what,
                body
            );
        }

        let run = std::process::Command::new(project_dir.join("a.out"))
            .output()
            .expect("failed to run the built executable");
        assert!(
            run.status.success(),
            "the built executable did not run cleanly"
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), expected);
    }

    /// The `range.fold` driver the simplifier leaves behind builds no union: the `Option` that
    /// `range`'s `advance` returns and `fold` immediately matches is cancelled, so the loop-carried
    /// state is the plain `RangeIterator` alone. The built program still sums the range — 0 + 1 + ..
    /// + 99 — so the removal leaves what the loop computes intact.
    #[test]
    fn test_range_fold_union_removed() {
        // The `range.fold` drivers (own and borrow version) — identified by the `RangeIterator` loop
        // state in their signature.
        assert_union_cancelled(
            "read_fold",
            &["Iterator::fold", "RangeIterator"],
            "the `range.fold` driver (an `Iterator::fold` over a `RangeIterator`)",
            "4950",
        );
    }

    /// `scale`'s inner arms build one constructor, so moving the outer `some` arm into them places a
    /// copy in each. The copy is smaller than the construction and the match it replaces, so the
    /// simplifier takes the move and `scale` builds no union. The built program still computes
    /// `scale` over 0..10, so the removal leaves the values intact.
    #[test]
    fn test_one_variant_union_removed() {
        assert_union_cancelled(
            "one_variant",
            &["Main::scale"],
            "`Main::scale`",
            "[0, 12, 15, 18, 21, 15, 27, 30, 33, 36]",
        );
    }
}

#[cfg(test)]
mod value_tests {
    use crate::{configuration::Configuration, tests::test_util::test_source};

    /// Checks the values a nest of matches computes when every inner arm builds the same variant.
    /// That is the shape case-of-case floats the outer match into, and the rewrite has to leave
    /// every value as the source computes it.
    #[test]
    pub fn test_nested_matches_over_one_variant() {
        let source = r#"
            module Main;

            f : I64 -> I64;
            f = |n| (
                match (if n % 5 == 0 { Option::some(n) } else { Option::some(n + 3) }) {
                    some(v1) => (
                        match (if v1 % 4 == 0 { Option::some(v1) } else { Option::some(v1 + 2) }) {
                            some(v2) => (
                                match (if v2 % 3 == 0 { Option::some(v2) } else { Option::some(v2 + 1) }) {
                                    some(v3) => v3 * 3 + n,
                                    none() => 0
                                }
                            ),
                            none() => 0
                        }
                    ),
                    none() => 0
                }
            );

            main : IO ();
            main = (
                assert_eq(
                    |_|"nested matches over one variant",
                    Iterator::range(0, 5).map(f).to_array, [0, 16, 26, 30, 31]
                );;
                pure()
            );
        "#;
        test_source(&source, Configuration::develop_mode());
    }

    /// The outer match answers the `none` an inner arm builds with a catch-all arm, which binds the
    /// whole union rather than that constructor's payload. Moving such an arm into the inner arm
    /// would bind it to the payload instead, so the rewrite declines and the values stay what the
    /// source computes.
    #[test]
    pub fn test_catch_all_outer_arm() {
        let source = r#"
            module Main;

            f : I64 -> I64;
            f = |n| (
                match (if n % 5 == 0 { Option::some(n) } else { Option::none() }) {
                    some(v) => v * 3,
                    rest => rest.as_some_or(-1) - 7
                }
            );

            main : IO ();
            main = (
                assert_eq(
                    |_|"a catch-all arm of the outer match",
                    Iterator::range(0, 6).map(f).to_array, [0, -8, -8, -8, -8, 15]
                );;
                pure()
            );
        "#;
        test_source(&source, Configuration::develop_mode());
    }
}

#[cfg(test)]
mod build_time_tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// Deep enough that a term doubling per level is out of reach, and shallow enough that a term
    /// growing with the level compiles in under a second.
    const DEPTH: usize = 16;

    /// Generous next to the fraction of a second the build takes, and short enough to report a
    /// regression as a failure instead of occupying the machine.
    const TIMEOUT: Duration = Duration::from_secs(60);

    /// The modulus level `level` of the nest tests its value against.
    fn level_modulus(level: usize) -> i64 {
        3 + (level % 5) as i64
    }

    /// What level `level` of the nest adds to its value where the modulus does not divide it.
    fn level_addend(level: usize) -> i64 {
        level as i64 + 1
    }

    /// What the nest computes for `n`: each level leaves the value alone when its modulus divides it
    /// and adds its addend otherwise, and the innermost body triples the result.
    fn nested_matches_value(n: i64) -> i64 {
        let mut v = n;
        for level in 1..=DEPTH {
            if v % level_modulus(level) != 0 {
                v += level_addend(level);
            }
        }
        v * 3
    }

    /// A nest of `DEPTH` matches over an `Option` that both branches of an `if` build with the same
    /// constructor. Case-of-case moves an outer arm into the inner arm that builds its constructor,
    /// and here one constructor is built by both arms, so a rewrite that served both would leave the
    /// outer match — which holds the whole nest below this level — in each of them, doubling the term
    /// at every level.
    fn nested_matches_source() -> String {
        let mut source = String::from("module Main;\n\nf : I64 -> I64;\nf = |n| (\n");
        let mut indent = String::from("    ");
        for level in 1..=DEPTH {
            let scrutinee = if level == 1 {
                "n".to_string()
            } else {
                format!("v{}", level - 1)
            };
            source += &format!(
                "{indent}match (if {scrutinee} % {} == 0 {{ Option::some({scrutinee}) }} \
                 else {{ Option::some({scrutinee} + {}) }}) {{\n",
                level_modulus(level),
                level_addend(level)
            );
            source += &format!("{indent}    some(v{level}) => (\n");
            indent += "        ";
        }
        source += &format!("{indent}v{DEPTH} * 3\n");
        for _ in 0..DEPTH {
            indent.truncate(indent.len() - 8);
            source += &format!("{indent}    ),\n{indent}    none() => 0\n{indent}}}\n");
        }
        source += ");\n\nmain : IO ();\nmain = println(f(7).to_string);\n";
        source
    }

    /// The nest compiles in a time that grows with its depth rather than doubling with it, and the
    /// program it compiles to computes what the source says. `max` is where the simplifier runs.
    #[test]
    fn test_nested_matches_compile_in_reasonable_time() {
        let printed = build_within_and_run(
            &nested_matches_source(),
            "max",
            TIMEOUT,
            &format!("a nest of {} matches", DEPTH),
        );
        assert_eq!(
            printed,
            nested_matches_value(7).to_string(),
            "the nest of {} matches returned a wrong value",
            DEPTH
        );
    }
}
