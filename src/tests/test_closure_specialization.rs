//! The two techniques of the closure specialization pass, read off the `--emit-rc-ir` dump: a
//! lambda is lifted to a global function, and the function it is passed to gets a copy that calls
//! that function by name. A recursion that hands the next round a closure built from the one it
//! was given could ask for one copy per round, and runs out instead; a function whose closure
//! parameters are decided independently could ask for one copy per combination of them, and is held
//! to a budget instead.
//!
//! The dump is what these assert against because a program cannot observe either one — both leave
//! the answer unchanged, so a suite that only runs the program stays green with the whole pass
//! switched off.

#[cfg(test)]
mod integration_tests {
    use crate::constants::{CAP_LIST_PREFIX, CLOSURE_LAM_SUFFIX, CLOSURE_SPEC_SUFFIX};
    use crate::tests::test_util::{copy_dir_recursive, fix_command_at_opt_level};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::TempDir;

    /// What `specialized_fold` prints: the sum of `3 * i` over `0..9`.
    const SPECIALIZED_FOLD_OUTPUT: &str = "135";

    /// What `changing_closure` prints: `grow`'s four wrappers around the identity answer 10, and
    /// `tick` / `tock`'s two wrappers around `|x| x * 100` answer 302.
    const CHANGING_CLOSURE_OUTPUT: &str = "312";

    /// What `relayed_closure` prints: `relay(op, n)` sums `terminal(op, i)` over `0..n` and recurses
    /// on `n - 1`, with `op = |x| x * 2` and `n = 4`.
    const RELAYED_CLOSURE_OUTPUT: &str = "30";

    /// How many copies `changing_closure` asks for today, counted over the `Main::` functions whose
    /// name carries `#closure_spec`. The chain of requests is what bounds this: each function, way
    /// in and lambda gets committed to one value.
    const CHANGING_CLOSURE_COPIES: usize = 13;

    /// What `two_narrowed_fields` prints: `relay(f, g, n)` sums `terminal_a(f, i)` and
    /// `terminal_b(g, i)` over `0..n` and recurses on `n - 1`, with `f = |x| x * 2`,
    /// `g = |x| x * 3` and `n = 4`.
    const TWO_NARROWED_FIELDS_OUTPUT: &str = "120";

    /// What `derived_closure` prints: `relay` sums `terminal(shifted, i)` over `0..n`, adds
    /// `shifted(n)`, and recurses on `n - 1`, with `shifted = |x| x * 3 + 1` and `n = 4`.
    const DERIVED_CLOSURE_OUTPUT: &str = "89";

    /// What `opaque_boundary` prints: `through_struct` 30, `through_array` 30, `through_union` -2.
    const OPAQUE_BOUNDARY_OUTPUT: &str = "58";

    /// What `struct_field_closure` prints: `stepping` sums `3 * n` over `4..1` for 30, and
    /// `swapping` sums `3 * 4` and then `n + 1` over `3..1` for 21.
    const STRUCT_FIELD_CLOSURE_OUTPUT: &str = "51";

    /// What `struct_field_replaced` prints: `modding` adds one more to what the field holds on every
    /// round of `4..1` for 36, and `setting` sums `3 * 4` and then `n + 1` over `3..1` for 21.
    const STRUCT_FIELD_REPLACED_OUTPUT: &str = "57";

    /// What `struct_field_partial_application` prints: `step` sums `3 * n` over `4..1` from 0 and
    /// from 1 for 30 and 31, and `step2` sums it over `2..1` for 9.
    const STRUCT_FIELD_PARTIAL_APPLICATION_OUTPUT: &str = "70";

    /// What `struct_field_two_closures` prints: `stepping` sums `3 * n` scaled by 100 and `n + 7`
    /// over `4..1` for 3034 and over `2..1` for 921, the scaling telling the two fields apart.
    const STRUCT_FIELD_TWO_CLOSURES_OUTPUT: &str = "3955";

    /// What `struct_field_named_struct` prints: `stepping` sums `3 * n` over `4..1` for 30 and over
    /// `2..1` for 9.
    const STRUCT_FIELD_NAMED_STRUCT_OUTPUT: &str = "39";

    /// What `struct_field_reachable_elsewhere` prints: `wrapped` sums `3 * n` and the wrapped
    /// `n + 1000` and the tag over `4..1` and `2..1` for 6087, scaled by 10000 so that either half
    /// shows on its own, and `shadowed` sums `3 * n` once and `n + 1000` thereafter for 4025.
    const STRUCT_FIELD_REACHABLE_ELSEWHERE_OUTPUT: &str = "60874025";

    /// What `struct_field_iterator_chain` prints: the fold sums `3 * (i % 7)` over 64 elements for
    /// 567, and the collected array has 64 elements.
    const STRUCT_FIELD_ITERATOR_CHAIN_OUTPUT: &str = "631";

    /// What `mixed_capture_field` prints: `relay` sums `op(i) + terminal(op, i) + opaque(op, i)`
    /// over `0..n` and recurses on `n - 1`, with `op = |x| x * 5 + 1` and `n = 4`.
    const MIXED_CAPTURE_FIELD_OUTPUT: &str = "205";

    /// What `independent_slots` prints: `g(p0, p1, p2, p3, n)` sums the four recursive calls that
    /// each wrap one of the closures, modulo 1000, with `n = 2` and the four lambdas main builds.
    const INDEPENDENT_SLOTS_OUTPUT: &str = "176";

    /// How many copies of `g` `independent_slots` may have. What bounds it is the budget on the
    /// copies a function may have out of combining its slots: four slots decided independently reach
    /// 95 copies without it, and every further slot multiplies that again. The bound sits between
    /// the two, so that adjusting the allowance moves the count without moving the claim.
    const INDEPENDENT_SLOTS_COPIES: usize = 40;

    /// What `wrapped_chain` prints: `g(op, n)` sums four recursive calls, each handing on `op`
    /// wrapped a different way, with `n = 3` and `op = |x| (x * 3) % 97`.
    const WRAPPED_CHAIN_OUTPUT: &str = "480";

    /// How many copies of `g` `wrapped_chain` may have. Counting a copy by the lambdas it names is
    /// what bounds this: four wrappers reach 129 copies when only the ways in are counted, and every
    /// further wrapper multiplies that again. The bound sits between the two, so that adjusting the
    /// allowance moves the count without moving the claim.
    const WRAPPED_CHAIN_COPIES: usize = 50;

    /// How many copies the lambdas lifted out of `g` in `wrapped_chain` may have. Each of those
    /// lambdas is a function the budget counts on its own, and there are more of them than there are
    /// copies of `g`, so a bound on one says nothing about the other. Counting a copy by the ways in
    /// it substitutes reaches 516; the bound sits between the two.
    const WRAPPED_CHAIN_LAMBDA_COPIES: usize = 200;

    /// What `closure_swap` prints: `ping` and `pong` calling each other five rounds deep, each round
    /// swapping which of the two closures sits in which way in and wrapping one of them.
    const CLOSURE_SWAP_OUTPUT: &str = "938";

    /// How many copies `closure_swap` asks for today. What bounds it is the commitment a chain
    /// records: raising the allowance the budget grants leaves this count where it is.
    const CLOSURE_SWAP_COPIES: usize = 9;

    /// What `wide_capture` prints: `relay` over four closures with `n = 2`, summing `terminal` on
    /// each and recursing four ways.
    const WIDE_CAPTURE_OUTPUT: &str = "18";

    /// How many closures `wide_capture` builds a capture list out of, and so how many of them one
    /// copy of the lambda receiving it can call by name.
    const WIDE_CAPTURE_KNOWN_CLOSURES: usize = 4;

    /// How many copies of `relay` `wide_capture` may have. Four capture fields decided independently
    /// reach 95 copies without the budget on the copies that combine slots; the bound sits between
    /// the two, so that adjusting the allowance moves the count without moving the claim.
    const WIDE_CAPTURE_COPIES: usize = 50;

    /// What `deep_relay` prints: the cycle `a` -> `b` -> `c` entered with `n = 6`, wrapping the
    /// closure one level deeper on each turn of the cycle.
    const DEEP_RELAY_OUTPUT: &str = "569";

    /// How many copies `deep_relay` asks for today. What bounds it is the commitment a chain
    /// records: raising the allowance the budget grants leaves this count where it is.
    const DEEP_RELAY_COPIES: usize = 7;

    /// Copies the case projects into a temporary directory of their own, so that parallel test runs
    /// do not share a build directory, and returns the directory of the named case.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        let mut cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cases_dir.push("src/tests/test_closure_specialization/cases");
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        copy_dir_recursive(&cases_dir, &temp_dir.path().to_path_buf())
            .expect("Failed to copy test cases");
        let project_dir = temp_dir.path().join(case);
        (temp_dir, project_dir)
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

        let dump_path = project_dir.join(".fixlang/rc_ir.post.txt");
        fs::read_to_string(&dump_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dump_path.display(), e))
    }

    /// The names of the functions in `dump` whose name contains `name_part`.
    ///
    /// A lambda inside a function is dumped under the enclosing function's name followed by
    /// `::closure#<n>`, and those are left out: they are parts of one function, so counting them
    /// would say a function was copied when it was not.
    fn functions_named_with<'a>(dump: &'a str, name_part: &str) -> Vec<&'a str> {
        dump.lines()
            .filter_map(|line| line.strip_prefix("fn "))
            .map(|rest| rest.split('(').next().unwrap().trim())
            .filter(|name| !name.contains("::closure#"))
            .filter(|name| name.contains(name_part))
            .collect()
    }

    /// The copies that `dump` names under `func_prefix`, deduplicated, each paired with whether it
    /// copies a lambda lifted out of that function rather than the function itself.
    ///
    /// A copy is named by appending `#closure_spec_<hash>` to the name of what it copies, and the
    /// stages after this pass append segments of their own, so what identifies one copy is the name
    /// up to the end of that segment. A lambda lifted out of the function carries a `#closure_lam`
    /// segment ahead of the `#closure_spec_` one, which is what tells the two populations apart.
    fn spec_copies_under(dump: &str, func_prefix: &str) -> Vec<(String, bool)> {
        let mut copies = functions_named_with(dump, &format!("{}_", CLOSURE_SPEC_SUFFIX))
            .into_iter()
            .filter(|name| name.starts_with(func_prefix))
            .filter_map(copy_name)
            .map(|copy| (copy.to_string(), copy.contains(CLOSURE_LAM_SUFFIX)))
            .collect::<Vec<_>>();
        copies.sort();
        copies.dedup();
        copies
    }

    /// The part of `name` that names one copy, which is the name up to the end of the
    /// `#closure_spec_` segment, or `None` where `name` carries no such segment.
    fn copy_name(name: &str) -> Option<&str> {
        let spec_start = name.find(&format!("{}_", CLOSURE_SPEC_SUFFIX))?;
        let end = name[spec_start + 1..]
            .find('#')
            .map_or(name.len(), |offset| spec_start + 1 + offset);
        Some(&name[..end])
    }

    /// The most capture lists any copy of the function whose name begins with `func_prefix` takes
    /// as a parameter.
    ///
    /// A copy takes one per way in it is specialized on, so this says how many of the closures
    /// reaching that function at once are known: one for a closure handed over as an argument, and
    /// one more for each read out of a field of a struct argument.
    fn most_capture_lists_taken_by(dump: &str, func_prefix: &str) -> usize {
        dump.lines()
            .filter_map(|line| line.strip_prefix("fn "))
            .filter(|line| line.starts_with(func_prefix) && line.contains(CLOSURE_SPEC_SUFFIX))
            .filter_map(|line| {
                let open = line.find('(')?;
                let close = line.rfind(") ->")?;
                Some(line[open..close].matches(CAP_LIST_PREFIX).count())
            })
            .max()
            .unwrap_or(0)
    }

    /// The copies that `dump` names of the function whose name begins with `func_prefix`.
    fn copies_of(dump: &str, func_prefix: &str) -> Vec<String> {
        spec_copies_under(dump, func_prefix)
            .into_iter()
            .filter(|(_, is_lambda_copy)| !is_lambda_copy)
            .map(|(name, _)| name)
            .collect()
    }

    /// The copies that `dump` names of the lambdas lifted out of the function whose name begins
    /// with `func_prefix`. Each such lambda is a function of its own, copied once per value it is
    /// given, so bounding the copies of the function bounds nothing about these.
    fn lambda_copies_of(dump: &str, func_prefix: &str) -> Vec<String> {
        spec_copies_under(dump, func_prefix)
            .into_iter()
            .filter(|(_, is_lambda_copy)| *is_lambda_copy)
            .map(|(name, _)| name)
            .collect()
    }

    /// The largest number of distinct copies of the function named by `callee_prefix` that any one
    /// body in `dump` calls by name.
    ///
    /// A closure whose identity a capture field holds is called by name from the body receiving that
    /// capture list, so this says how many of the closures one capture list carries are known at
    /// once. The line naming a function is left out of its own body, so a copy does not count itself.
    fn most_copies_called_from_one_body(dump: &str, callee_prefix: &str) -> usize {
        dump.split("\nfn ")
            .map(|function| {
                let body = function.split_once('\n').map_or("", |(_, body)| body);
                let mut called = body
                    .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':' || c == '#'))
                    .filter(|token| token.starts_with(callee_prefix))
                    .filter_map(copy_name)
                    .collect::<Vec<_>>();
                called.sort();
                called.dedup();
                called.len()
            })
            .max()
            .unwrap_or(0)
    }

    /// A lambda passed to a global function is lifted to a global function of its own, and that
    /// global function is copied into a version specialized on it.
    #[test]
    pub fn test_a_lambda_passed_to_a_function_is_lifted_and_specialized_on() {
        let (_temp_dir, project_dir) = setup_test_env("specialized_fold");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", SPECIALIZED_FOLD_OUTPUT);

        let lifted = functions_named_with(&dump, CLOSURE_LAM_SUFFIX);
        assert!(
            !lifted.is_empty(),
            "the pass should lift the lambda to a global function, but the dump names none: {}",
            dump.lines().take(20).collect::<Vec<_>>().join("\n")
        );
        let specialized = functions_named_with(&dump, CLOSURE_SPEC_SUFFIX);
        assert!(
            !specialized.is_empty(),
            "the pass should specialize the function the lambda is passed to, but the dump names \
             none. It lifted: {:?}",
            lifted
        );
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

    /// A function that reads a closure out of a struct argument and calls it gets a copy taking the
    /// capture list of that closure beside the struct, and calls the lambda by name. A function whose
    /// struct argument can hold a different closure on the next round gets no such copy.
    #[test]
    pub fn test_a_closure_in_a_struct_argument_is_called_by_name() {
        let (_temp_dir, project_dir) = setup_test_env("struct_field_closure");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, STRUCT_FIELD_CLOSURE_OUTPUT);
        }
        let dump = build_run_and_read_rc_ir(&project_dir, "max", STRUCT_FIELD_CLOSURE_OUTPUT);

        let stepping = copies_of(&dump, "Main::stepping");
        assert_eq!(
            stepping.len(),
            1,
            "`stepping` reads the closure out of its struct argument and calls it, so it should have \
             one copy, but the dump names: {:?}",
            stepping
        );
        let calls_the_lambda_by_name = dump
            .split("\nfn ")
            .filter(|function| function.starts_with("Main::stepping"))
            .any(|function| function.contains(CLOSURE_LAM_SUFFIX));
        assert!(
            calls_the_lambda_by_name,
            "the copy of `stepping` should call the lifted lambda by name, but no body of it names \
             one"
        );

        let swapping = copies_of(&dump, "Main::swapping");
        assert!(
            swapping.is_empty(),
            "`swapping` puts another closure in the field on every round, so no copy of it can name \
             the one it was given, but the dump names: {:?}",
            swapping
        );
    }

    /// A struct the body builds by replacing the field — through `mod_` or `set_` — holds a function
    /// the one that arrived did not, so a call reading that field out cannot name the function the
    /// argument was built with.
    #[test]
    pub fn test_a_field_replaced_in_the_body_is_not_the_one_the_argument_carried() {
        let (_temp_dir, project_dir) = setup_test_env("struct_field_replaced");
        for opt_level in ["basic", "max", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, STRUCT_FIELD_REPLACED_OUTPUT);
        }
    }

    /// The capture list a copy receives stands directly after the struct argument it belongs to, so
    /// a call supplying the struct alone is still the partial application it was.
    #[test]
    pub fn test_a_copy_taking_a_capture_list_can_be_partly_applied() {
        let (_temp_dir, project_dir) = setup_test_env("struct_field_partial_application");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(
                &project_dir,
                opt_level,
                STRUCT_FIELD_PARTIAL_APPLICATION_OUTPUT,
            );
        }
        let dump =
            build_run_and_read_rc_ir(&project_dir, "max", STRUCT_FIELD_PARTIAL_APPLICATION_OUTPUT);

        let copies = copies_of(&dump, "Main::stepping");
        assert_eq!(
            copies.len(),
            1,
            "both call sites give `stepping` the same closure at the same field, so it should have \
             one copy, but the dump names: {:?}",
            copies
        );
    }

    /// A struct holding a closure at two of its fields hands over both capture lists, in the order
    /// the fields are declared, which is the order the copy binds the parameters receiving them. The
    /// destructuring names them in another order, so the position a field is read at is the one the
    /// declaration gives it rather than the one the source writes it at.
    #[test]
    pub fn test_a_struct_holding_two_closures_hands_over_both_capture_lists() {
        let (_temp_dir, project_dir) = setup_test_env("struct_field_two_closures");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, STRUCT_FIELD_TWO_CLOSURES_OUTPUT);
        }
        let dump = build_run_and_read_rc_ir(&project_dir, "max", STRUCT_FIELD_TWO_CLOSURES_OUTPUT);

        let copies = copies_of(&dump, "Main::stepping");
        assert_eq!(
            copies.len(),
            1,
            "both call sites give `stepping` the same two closures, so it should have one copy, but \
             the dump names: {:?}",
            copies
        );
    }

    /// A struct built and given a name hands what its fields hold to that name, so a call it is then
    /// passed to says what the argument carries.
    #[test]
    pub fn test_a_struct_named_before_it_is_handed_over_still_says_what_it_holds() {
        let (_temp_dir, project_dir) = setup_test_env("struct_field_named_struct");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, STRUCT_FIELD_NAMED_STRUCT_OUTPUT);
        }
        let dump = build_run_and_read_rc_ir(&project_dir, "max", STRUCT_FIELD_NAMED_STRUCT_OUTPUT);

        let copies = copies_of(&dump, "Main::stepping");
        assert_eq!(
            copies.len(),
            1,
            "`stepping` is handed a named struct holding the same closure at both call sites, so it \
             should have one copy, but the dump names: {:?}",
            copies
        );
    }

    /// A struct of the type under question can reach the body without being named there: carried
    /// inside another struct a call gives back, or bound to the name the destructuring gave the
    /// field. Neither one holds the function the argument arrived with.
    #[test]
    pub fn test_a_struct_reaching_the_body_another_way_is_not_the_one_given() {
        let (_temp_dir, project_dir) = setup_test_env("struct_field_reachable_elsewhere");
        for opt_level in ["basic", "max", "experimental"] {
            build_run_and_read_rc_ir(
                &project_dir,
                opt_level,
                STRUCT_FIELD_REACHABLE_ELSEWHERE_OUTPUT,
            );
        }
    }

    /// The chain `map` and `fold` build puts the mapped function in a struct field and reads it out
    /// of the struct the fold is handed, which is the shape this way in was added for.
    #[test]
    pub fn test_a_mapped_iterator_gives_the_fold_a_lambda_it_can_name() {
        let (_temp_dir, project_dir) = setup_test_env("struct_field_iterator_chain");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, STRUCT_FIELD_ITERATOR_CHAIN_OUTPUT);
        }
        let dump =
            build_run_and_read_rc_ir(&project_dir, "max", STRUCT_FIELD_ITERATOR_CHAIN_OUTPUT);

        let capture_lists = most_capture_lists_taken_by(&dump, "Std::Iterator::fold");
        assert!(
            capture_lists >= 2,
            "the fold is handed the operator as an argument and reads the mapped function out of \
             the struct, so a copy of it should take two capture lists, but the most any takes is {}",
            capture_lists
        );
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

    /// A recursion that wraps the closure it was given on every round could ask for one specialized
    /// copy per round. Along one chain of requests the pass commits each function, argument and
    /// lambda origin to a single lambda, so the copies run out: none of them is itself specialized
    /// again. Both shapes are covered — the recursion a function does on its own, and the one that
    /// goes around a cycle of two.
    #[test]
    pub fn test_a_recursion_carrying_a_new_closure_each_round_runs_out_of_copies() {
        let (_temp_dir, project_dir) = setup_test_env("changing_closure");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", CHANGING_CLOSURE_OUTPUT);

        let lifted = functions_named_with(&dump, CLOSURE_LAM_SUFFIX);
        // The two the inliner leaves standing as recursions of their own.
        for recursive_fn in ["Main::grow#", "Main::tock#"] {
            assert!(
                lifted.iter().any(|name| name.starts_with(recursive_fn)),
                "the pass should lift the lambda that `{}` carries into the next round, but the \
                 dump names only: {:?}",
                recursive_fn,
                lifted
            );
        }

        let specialized = functions_named_with(&dump, CLOSURE_SPEC_SUFFIX)
            .into_iter()
            .filter(|name| name.starts_with("Main::"))
            .collect::<Vec<_>>();
        assert!(
            !specialized.is_empty(),
            "the pass should specialize these recursions, but the dump names no copy of them"
        );
        assert!(
            specialized.len() <= CHANGING_CLOSURE_COPIES,
            "the chain of requests should run out after {} copies, but the dump names {}: {:?}",
            CHANGING_CLOSURE_COPIES,
            specialized.len(),
            specialized
        );
    }

    /// A function taking four closures and calling every one of them has each of the four decided on
    /// its own, so the copies it can be asked for are keyed on the combinations of those decisions
    /// and grow exponentially in the number of closures. The chain of requests cannot see that: each
    /// combination meets a commitment of its own and none of them disagrees. What bounds it is the
    /// budget on the copies one function may have out of combining its slots.
    #[test]
    pub fn test_a_function_whose_closures_are_decided_independently_stays_within_its_budget() {
        let (_temp_dir, project_dir) = setup_test_env("independent_slots");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", INDEPENDENT_SLOTS_OUTPUT);

        let copies = copies_of(&dump, "Main::g#");
        assert!(
            !copies.is_empty(),
            "`g` should be specialized on the lambdas it is given, but the dump names no copy of it"
        );
        assert!(
            copies.len() <= INDEPENDENT_SLOTS_COPIES,
            "`g` should have at most {} copies, but the dump names {}",
            INDEPENDENT_SLOTS_COPIES,
            copies.len()
        );
    }

    /// A function that hands the next round the closure it was given, wrapped a different way each
    /// time, is bounded across the *orderings* of those wrappers. Every copy here substitutes one
    /// way in and every capture list holds one closure, so a rule that reads how many ways in a copy
    /// substitutes sees nothing to bound; what grows is the number of wrappers a value has been
    /// through, which is what counting the lambdas a copy names reads.
    #[test]
    pub fn test_a_closure_wrapped_a_different_way_each_round_stays_within_its_budget() {
        let (_temp_dir, project_dir) = setup_test_env("wrapped_chain");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", WRAPPED_CHAIN_OUTPUT);

        let copies = copies_of(&dump, "Main::g#");
        assert!(
            !copies.is_empty(),
            "`g` should be specialized on the lambdas it is given, but the dump names no copy of it"
        );
        assert!(
            copies.len() <= WRAPPED_CHAIN_COPIES,
            "`g` should have at most {} copies, but the dump names {}",
            WRAPPED_CHAIN_COPIES,
            copies.len()
        );

        let lambda_copies = lambda_copies_of(&dump, "Main::g#");
        assert!(
            !lambda_copies.is_empty(),
            "the lambdas `g` wraps its argument in should be copied, but the dump names none"
        );
        assert!(
            lambda_copies.len() <= WRAPPED_CHAIN_LAMBDA_COPIES,
            "the lambdas `g` lifts should have at most {} copies, but the dump names {}",
            WRAPPED_CHAIN_LAMBDA_COPIES,
            lambda_copies.len()
        );
    }

    /// Two closures going round a cycle of two functions, swapping which way in each sits in on
    /// every round while one of them is wrapped. A value's identity has to survive both the cycle
    /// and the swap, and the commitment a chain records is per way in, so the two must not be
    /// confused for one another.
    #[test]
    pub fn test_two_closures_swapping_places_around_a_cycle_keep_their_identity() {
        let (_temp_dir, project_dir) = setup_test_env("closure_swap");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, CLOSURE_SWAP_OUTPUT);
        }
        let dump = build_run_and_read_rc_ir(&project_dir, "max", CLOSURE_SWAP_OUTPUT);

        assert!(
            !copies_of(&dump, "Main::pong#").is_empty(),
            "`pong` receives the two closures in the ways in `ping` did not, so it gets a copy only \
             if their identity survived both the cycle and the swap. The dump names: {:?}",
            copies_of(&dump, "Main::")
        );
        let apply_twice = copies_of(&dump, "Main::apply_twice#");
        assert!(
            apply_twice.len() >= 2,
            "`apply_twice` is called with each of the two closures in turn, so it should have a \
             copy per closure. The dump names {}: {:?}",
            apply_twice.len(),
            apply_twice
        );
        let copies = copies_of(&dump, "Main::");
        assert!(
            copies.len() <= CLOSURE_SWAP_COPIES,
            "the chain of requests should run out after {} copies, but the dump names {}: {:?}",
            CLOSURE_SWAP_COPIES,
            copies.len(),
            copies
        );
    }

    /// One capture list carrying four closures whose identity is known, where each round narrows a
    /// different one of them.
    #[test]
    pub fn test_a_capture_list_carrying_four_known_closures() {
        let (_temp_dir, project_dir) = setup_test_env("wide_capture");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, WIDE_CAPTURE_OUTPUT);
        }
        let dump = build_run_and_read_rc_ir(&project_dir, "max", WIDE_CAPTURE_OUTPUT);

        let known_at_once = most_copies_called_from_one_body(&dump, "Main::terminal#");
        assert!(
            known_at_once >= WIDE_CAPTURE_KNOWN_CLOSURES,
            "the lambda `relay` hands to `fold` captures all four closures and calls `terminal` on \
             each, so one of its copies should call {} copies of `terminal` by name. The most any \
             body calls is {}",
            WIDE_CAPTURE_KNOWN_CLOSURES,
            known_at_once
        );
        let copies = copies_of(&dump, "Main::relay#");
        assert!(
            copies.len() <= WIDE_CAPTURE_COPIES,
            "`relay` should have at most {} copies, but the dump names {}",
            WIDE_CAPTURE_COPIES,
            copies.len()
        );
    }

    /// A cycle of three functions where one turn of the cycle wraps the closure a level deeper, so
    /// the chain of requests runs out on a cycle longer than the two the suite already covers.
    #[test]
    pub fn test_a_cycle_of_three_wrapping_one_level_per_turn_runs_out() {
        let (_temp_dir, project_dir) = setup_test_env("deep_relay");
        for opt_level in ["basic", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, DEEP_RELAY_OUTPUT);
        }
        let dump = build_run_and_read_rc_ir(&project_dir, "max", DEEP_RELAY_OUTPUT);

        assert!(
            !copies_of(&dump, "Main::a#").is_empty(),
            "`a` wraps the closure it was given and hands it round the cycle, so it should be \
             specialized on what comes back. The dump names: {:?}",
            copies_of(&dump, "Main::")
        );
        let copies = copies_of(&dump, "Main::");
        assert!(
            copies.len() <= DEEP_RELAY_COPIES,
            "the chain of requests should run out after {} copies, but the dump names {}: {:?}",
            DEEP_RELAY_COPIES,
            copies.len(),
            copies
        );
    }

    /// A closure a function builds from the one it was given becomes a capture list, and the lambda
    /// that carries it into `fold` holds it in a capture field. Specializing the function narrows
    /// the inner capture list, so what that field holds changes type — and a field that cannot
    /// follow has no closure to fall back on, since it is not one.
    #[test]
    pub fn test_a_capture_field_follows_the_value_it_holds() {
        let (_temp_dir, project_dir) = setup_test_env("derived_closure");
        build_run_and_read_rc_ir(&project_dir, "max", DERIVED_CLOSURE_OUTPUT);
    }

    /// One capture list carrying two closures whose identity is known. Each field is decided on its
    /// own, so both have to follow the value they hold: the function each closure is relayed to gets
    /// a copy only if the field carrying it was narrowed.
    #[test]
    pub fn test_two_capture_fields_of_one_lambda_are_narrowed() {
        let (_temp_dir, project_dir) = setup_test_env("two_narrowed_fields");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", TWO_NARROWED_FIELDS_OUTPUT);

        let specialized = functions_named_with(&dump, CLOSURE_SPEC_SUFFIX);
        for relayed_to in ["Main::terminal_a#", "Main::terminal_b#"] {
            assert!(
                specialized.iter().any(|name| name.starts_with(relayed_to)),
                "`{}` should get a copy, which it does only if the capture field holding the \
                 closure it is given was narrowed. The dump names: {:?}",
                relayed_to,
                specialized
            );
        }
    }

    /// The chain has to pass through a capture list to reach the end. `relay` never calls the closure
    /// it is given; the function that does call it is reached only from the lambda `relay` hands to
    /// `fold`, which captures the closure. Following the argument alone stops at `relay`, so the
    /// copies below exist only if the field of that lambda's capture list is narrowed to the closure
    /// it holds.
    #[test]
    pub fn test_the_chain_continues_through_a_capture_list() {
        let (_temp_dir, project_dir) = setup_test_env("relayed_closure");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", RELAYED_CLOSURE_OUTPUT);

        let narrowed = functions_named_with(&dump, CLOSURE_SPEC_SUFFIX)
            .into_iter()
            .filter(|name| name.contains(CLOSURE_LAM_SUFFIX))
            .collect::<Vec<_>>();
        assert!(
            !narrowed.is_empty(),
            "the lambda `relay` hands to `fold` should get a copy receiving the narrowed capture \
             list, but the dump names no copy of a lifted lambda. It names: {:?}",
            functions_named_with(&dump, "Main::")
        );

        let terminal = functions_named_with(&dump, "Main::terminal");
        assert!(
            terminal.iter().any(|name| name.contains(CLOSURE_SPEC_SUFFIX)),
            "the chain should reach `terminal` through that capture list and copy it, but the dump \
             names only: {:?}",
            terminal
        );
    }
}
