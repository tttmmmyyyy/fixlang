//! The two techniques of the closure specialization pass, read off the `--emit-rc-ir` dump: a
//! lambda is lifted to a global function, and the function it is passed to gets a copy that calls
//! that function by name.
//!
//! The dump is what these assert against because a program cannot observe either one — both leave
//! the answer unchanged, so a suite that only runs the program stays green with the whole pass
//! switched off.

#[cfg(test)]
mod integration_tests {
    use crate::constants::CLOSURE_SPEC_SUFFIX;
    use crate::misc::{Map, Set};
    use crate::tests::test_util::{fix_command_at_opt_level, read_rc_ir_dump, setup_case_projects};
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::TempDir;

    /// What `specialized_fold` prints: the sum of `3 * i` over `0..9`.
    const SPECIALIZED_FOLD_OUTPUT: &str = "135";

    /// What `derived_closure` prints: `relay` sums `terminal(shifted, i)` over `0..n`, adds
    /// `shifted(n)`, and recurses on `n - 1`, with `shifted = |x| x * 3 + 1` and `n = 4`.
    const DERIVED_CLOSURE_OUTPUT: &str = "89";

    /// What `opaque_boundary` prints: `through_struct` 30, `through_array` 30, `through_union` -2.
    const OPAQUE_BOUNDARY_OUTPUT: &str = "58";

    /// What `mixed_capture_field` prints: `relay` sums `op(i) + terminal(op, i) + opaque(op, i)`
    /// over `0..n` and recurses on `n - 1`, with `op = |x| x * 5 + 1` and `n = 4`.
    const MIXED_CAPTURE_FIELD_OUTPUT: &str = "205";

    /// What `monadic_combinator` prints: the sum of `i * i` over `0..99`, each round weighting the
    /// counter it reads by its index, plus the hundred rounds it ran.
    const MONADIC_COMBINATOR_OUTPUT: &str = "328450";

    /// Copies the case projects into a temporary directory of their own, so that parallel test runs
    /// do not share a build directory, and returns the directory of the named case.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        setup_case_projects("src/tests/test_closure_specialization/cases", case)
    }

    /// Builds the case at `opt_level`, runs it, and returns the RC IR of every module.
    ///
    /// Running the program is what keeps the dump honest: the names asserted on come off a build
    /// that is known to answer correctly.
    ///
    /// # Arguments
    /// * `opt_level` - the level the case is built at, which decides which specializations exist.
    /// * `expected_output` - what the case prints on stdout.
    fn build_run_and_read_rc_ir(
        project_dir: &Path,
        opt_level: &str,
        expected_output: &str,
    ) -> String {
        let build = fix_command_at_opt_level("build", opt_level)
            .args(["--emit-rc-ir", "all"])
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix build");
        assert!(
            build.status.success(),
            "the build should succeed at -O {}.\nstdout: {}\nstderr: {}",
            opt_level,
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr),
        );

        let run = Command::new(project_dir.join("a.out"))
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute the built program");
        assert!(
            run.status.success(),
            "the built program should run cleanly at -O {}, but exited with {}.\nstderr: {}",
            opt_level,
            run.status,
            String::from_utf8_lossy(&run.stderr),
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout).trim(),
            expected_output,
            "the program should answer the same at -O {}",
            opt_level
        );

        read_rc_ir_dump(project_dir)
    }

    /// The names of the functions in `dump` whose name contains `name_part`.
    ///
    /// A lambda inside a function is dumped under the enclosing function's name followed by
    /// `::closure#<n>`, and those are left out: they are parts of one function, so counting them
    /// would say a function was copied when it was not.
    fn functions_named_with<'a>(dump: &'a str, name_part: &str) -> Vec<&'a str> {
        dump.lines()
            .filter_map(|line| line.strip_prefix("fn "))
            .map(func_name)
            .filter(|name| !name.contains("::closure#"))
            .filter(|name| name.contains(name_part))
            .collect()
    }

    /// The names in `text` carrying all of `needles`, deduplicated. A name stands wherever the
    /// program mentions it — a function's own line, a call, or the type of a capture list named
    /// after the copy that receives it.
    fn names_in<'a>(text: &'a str, needles: &[&str]) -> Vec<&'a str> {
        let mut names = text
            .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':' || c == '#' || c == '@'))
            .filter(|token| needles.iter().all(|needle| token.contains(needle)))
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }

    /// Every function `dump` names, as the line naming it and the body under it.
    ///
    /// A blank line ends a body, so what follows the last function — the initializer of every global
    /// — stays out of it. A header line closes with the return type and a colon, which is what tells
    /// a function from the lines the dump opens with.
    fn functions_in<'a>(dump: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        dump.split("\nfn ")
            .filter_map(|function| function.split_once('\n'))
            .filter(|(header, _)| header.ends_with(':'))
            .map(|(header, body)| (header, body.split("\n\n").next().unwrap()))
    }

    /// The copies of the function named by `func_prefix` in `dump`, each as the line naming it and
    /// the body under it. One copy of `Std::loop` stands for one loop body of the case.
    fn spec_copy_bodies<'a>(dump: &'a str, func_prefix: &str) -> Vec<(&'a str, &'a str)> {
        functions_in(dump)
            .filter(|(header, _)| {
                let name = func_name(header);
                name.starts_with(func_prefix) && name.contains(CLOSURE_SPEC_SUFFIX)
            })
            .collect()
    }

    /// The name the header line of a function opens with, without the parameter list that follows
    /// it.
    fn func_name(header: &str) -> &str {
        header.split('(').next().unwrap().trim()
    }

    /// The body of every function `dump` names, keyed by the function's name.
    fn function_bodies(dump: &str) -> Map<&str, &str> {
        functions_in(dump)
            .map(|(header, body)| (func_name(header), body))
            .collect()
    }

    /// The functions of `bodies` that the ones named by `roots` reach by name, `roots` included,
    /// in name order.
    ///
    /// A call names the function it calls, so this walk collects the bodies a root runs. A function
    /// reached through a closure is named by no call, so it stays out, though the body that builds
    /// that closure is in.
    fn functions_reached_from<'a>(
        bodies: &Map<&'a str, &'a str>,
        roots: Vec<&'a str>,
    ) -> Vec<&'a str> {
        let mut reached: Set<&'a str> = roots.iter().copied().collect();
        let mut queue = roots;
        while let Some(name) = queue.pop() {
            for token in names_in(bodies[name], &[]) {
                if bodies.contains_key(token) && reached.insert(token) {
                    queue.push(token);
                }
            }
        }
        let mut names = reached.into_iter().collect::<Vec<_>>();
        names.sort();
        names
    }

    /// A closure the pass declines to follow — out through a struct field, an array element and a
    /// union payload, then back to a call — is left as a closure the program can still call, and
    /// answers the same as it does at the level the pass does not run at.
    #[test]
    pub fn test_a_closure_survives_the_boundaries_the_pass_declines_to_follow() {
        let (_temp_dir, project_dir) = setup_test_env("opaque_boundary");
        for opt_level in ["basic", "max", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, OPAQUE_BOUNDARY_OUTPUT);
        }
    }

    /// A narrowed capture field serves three readers at once: a call made there, a function the
    /// table copies for it, and a function it does not. The third one has no way in to reach, so
    /// the field's value has to wrap back into a closure at that place alone.
    #[test]
    pub fn test_a_narrowed_capture_field_serves_a_reader_that_needs_a_closure() {
        let (_temp_dir, project_dir) = setup_test_env("mixed_capture_field");
        for opt_level in ["basic", "max", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, MIXED_CAPTURE_FIELD_OUTPUT);
        }
    }

    /// The pass runs from the `max` optimization level up, so a build below it carries neither of
    /// the names it mints.
    #[test]
    pub fn test_the_pass_leaves_a_lower_optimization_level_alone() {
        let (_temp_dir, project_dir) = setup_test_env("specialized_fold");
        let dump = build_run_and_read_rc_ir(&project_dir, "basic", SPECIALIZED_FOLD_OUTPUT);

        let minted = functions_named_with(&dump, "#closure_");
        assert!(
            minted.is_empty(),
            "the pass should not run below `max`, but the dump names: {:?}",
            minted
        );
    }

    /// A closure a function builds from the one it was given becomes a capture list, and the lambda
    /// that carries it into `fold` holds it in a capture field. Specializing the function narrows
    /// the inner capture list, so what that field holds changes type. The field has to follow that
    /// type: what it holds is a capture list, and the wrap back into a closure is what a field
    /// holding a closure falls back on.
    #[test]
    pub fn test_a_capture_field_follows_the_value_it_holds() {
        let (_temp_dir, project_dir) = setup_test_env("derived_closure");
        build_run_and_read_rc_ir(&project_dir, "max", DERIVED_CLOSURE_OUTPUT);
    }

    /// A round of a combinator that answers in a monad builds no closure. The body of the callback
    /// goes into the copy of the combinator made for it, applied to the arguments the round
    /// supplies, and the copy reduces that application to the body itself. An application left
    /// standing is a closure the round builds on the heap and calls through, once per call of the
    /// callback.
    ///
    /// The dump is what this asserts against because the program cannot observe it: the fold
    /// answers the same either way.
    #[test]
    pub fn test_a_round_of_a_specialized_combinator_builds_no_closure() {
        let (_temp_dir, project_dir) = setup_test_env("monadic_combinator");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", MONADIC_COMBINATOR_OUTPUT);

        let bodies = function_bodies(&dump);
        let copies = spec_copy_bodies(&dump, "Main::range_fold_m#")
            .into_iter()
            .map(|(header, _)| func_name(header))
            .collect::<Vec<_>>();
        assert!(
            !copies.is_empty(),
            "`range_fold_m` should be too large for the inliner to substitute at its call site, so \
             that the callback reaches this pass, and the dump names no copy of it: {:?}",
            functions_named_with(&dump, "Main::")
        );

        for name in functions_reached_from(&bodies, copies) {
            assert!(
                !bodies[name].contains("= closure "),
                "a round of `range_fold_m` should build no closure, and `{}` builds one:\n{}",
                name,
                bodies[name]
            );
        }
    }
}
