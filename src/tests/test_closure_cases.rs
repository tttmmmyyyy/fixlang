//! Programs whose functions travel through values and combinators before they are called, built as
//! projects and run at the optimization levels below and above the one the lambda sets run at. One
//! of them also reads the `--emit-rc-ir` dump, for what a program cannot observe.

#[cfg(test)]
mod integration_tests {
    use crate::misc::{Map, Set};
    use crate::tests::test_util::{build_project_run_and_read_rc_ir, setup_case_projects};
    use std::path::PathBuf;
    use tempfile::TempDir;

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
        setup_case_projects("src/tests/test_closure_cases/cases", case)
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

    /// The names of the functions in `dump` whose name starts with `func_prefix`: the versions the
    /// compiler made of the global it names.
    fn functions_starting_with<'a>(dump: &'a str, func_prefix: &str) -> Vec<&'a str> {
        functions_in(dump)
            .map(|(header, _)| func_name(header))
            .filter(|name| name.starts_with(func_prefix))
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

    /// A function that leaves through a struct field, an array element and a union payload before
    /// it is called answers the same at every level.
    #[test]
    pub fn test_a_function_called_out_of_a_struct_an_array_and_a_union_answers_the_same() {
        let (_temp_dir, project_dir) = setup_test_env("opaque_boundary");
        for opt_level in ["basic", "max", "experimental"] {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, OPAQUE_BOUNDARY_OUTPUT);
        }
    }

    /// A function a lambda captures, called there, handed to a function that calls it, and handed to
    /// a function that puts it into an array, answers the same at every level.
    #[test]
    pub fn test_a_captured_function_called_three_ways_answers_the_same() {
        let (_temp_dir, project_dir) = setup_test_env("mixed_capture_field");
        for opt_level in ["basic", "max", "experimental"] {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, MIXED_CAPTURE_FIELD_OUTPUT);
        }
    }

    /// A function built from the one a function was given, called directly and through a lambda
    /// that captures it and is handed to `fold`, answers the same at every level.
    #[test]
    pub fn test_a_function_built_from_the_one_given_answers_the_same() {
        let (_temp_dir, project_dir) = setup_test_env("derived_closure");
        for opt_level in ["basic", "max", "experimental"] {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, DERIVED_CLOSURE_OUTPUT);
        }
    }

    /// A round of a combinator that answers in a monad builds no closure. The combinator calls the
    /// callback by name and the body of the callback goes into the round, applied to the arguments
    /// the round supplies. An application left standing is a closure the round builds on the heap
    /// and calls through, once per call of the callback.
    ///
    /// The dump is what this asserts against because the program cannot observe it: the fold
    /// answers the same either way.
    #[test]
    pub fn test_a_round_of_a_monadic_combinator_builds_no_closure() {
        let (_temp_dir, project_dir) = setup_test_env("monadic_combinator");
        let dump = build_project_run_and_read_rc_ir(&project_dir, "max", MONADIC_COMBINATOR_OUTPUT);

        let bodies = function_bodies(&dump);
        let copies = functions_starting_with(&dump, "Main::range_fold_m#");
        assert!(
            !copies.is_empty(),
            "`range_fold_m` should be too large for the inliner to substitute at its call site, so \
             that the call to it stands, and the dump names no function of it: {:?}",
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
