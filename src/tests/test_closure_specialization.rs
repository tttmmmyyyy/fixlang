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
    use crate::tests::test_util::{copy_dir_recursive, fix_command};
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

    /// What `wide_capture` prints: `relay` over four closures with `n = 2`, summing `terminal` on
    /// each and recursing four ways.
    const WIDE_CAPTURE_OUTPUT: &str = "18";

    /// What `deep_relay` prints: the cycle `a` -> `b` -> `c` entered with `n = 6`, wrapping the
    /// closure one level deeper on each turn of the cycle.
    const DEEP_RELAY_OUTPUT: &str = "569";

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
    /// * `opt_level` - pinned through `FIX_MAX_OPT_LEVEL`, so the level is the one this test asks
    ///   for whatever the level the suite is being run at.
    /// * `expected_output` - what the case prints on stdout.
    fn build_run_and_read_rc_ir(
        project_dir: &Path,
        opt_level: &str,
        expected_output: &str,
    ) -> String {
        let build = fix_command()
            .args(["build", "-O", opt_level, "--emit-rc-ir", "all"])
            .env("FIX_MAX_OPT_LEVEL", opt_level)
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
        let spec_segment = "#closure_spec_";
        let mut copies = functions_named_with(dump, spec_segment)
            .into_iter()
            .filter(|name| name.starts_with(func_prefix))
            .map(|name| {
                let spec_start = name.find(spec_segment).unwrap();
                let is_lambda_copy = name[..spec_start].contains("#closure_lam");
                let end = name[spec_start + 1..]
                    .find('#')
                    .map_or(name.len(), |offset| spec_start + 1 + offset);
                (name[..end].to_string(), is_lambda_copy)
            })
            .collect::<Vec<_>>();
        copies.sort();
        copies.dedup();
        copies
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

    /// A lambda passed to a global function is lifted to a global function of its own, and that
    /// global function is copied into a version specialized on it.
    #[test]
    pub fn test_a_lambda_passed_to_a_function_is_lifted_and_specialized_on() {
        let (_temp_dir, project_dir) = setup_test_env("specialized_fold");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", SPECIALIZED_FOLD_OUTPUT);

        let lifted = functions_named_with(&dump, "#closure_lam");
        assert!(
            !lifted.is_empty(),
            "the pass should lift the lambda to a global function, but the dump names none: {}",
            dump.lines().take(20).collect::<Vec<_>>().join("\n")
        );
        let specialized = functions_named_with(&dump, "#closure_spec");
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

        let lifted = functions_named_with(&dump, "#closure_lam");
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

        let specialized = functions_named_with(&dump, "#closure_spec")
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
        for opt_level in ["basic", "max", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, CLOSURE_SWAP_OUTPUT);
        }
    }

    /// One capture list carrying four closures whose identity is known, where each round narrows a
    /// different one of them.
    #[test]
    pub fn test_a_capture_list_carrying_four_known_closures() {
        let (_temp_dir, project_dir) = setup_test_env("wide_capture");
        for opt_level in ["basic", "max", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, WIDE_CAPTURE_OUTPUT);
        }
    }

    /// A cycle of three functions where one turn of the cycle wraps the closure a level deeper, so
    /// the chain of requests runs out on a cycle longer than the two the suite already covers.
    #[test]
    pub fn test_a_cycle_of_three_wrapping_one_level_per_turn_runs_out() {
        let (_temp_dir, project_dir) = setup_test_env("deep_relay");
        for opt_level in ["basic", "max", "experimental"] {
            build_run_and_read_rc_ir(&project_dir, opt_level, DEEP_RELAY_OUTPUT);
        }
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

        let specialized = functions_named_with(&dump, "#closure_spec");
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

        let narrowed = functions_named_with(&dump, "#closure_spec")
            .into_iter()
            .filter(|name| name.contains("#closure_lam"))
            .collect::<Vec<_>>();
        assert!(
            !narrowed.is_empty(),
            "the lambda `relay` hands to `fold` should get a copy receiving the narrowed capture \
             list, but the dump names no copy of a lifted lambda. It names: {:?}",
            functions_named_with(&dump, "Main::")
        );

        let terminal = functions_named_with(&dump, "Main::terminal");
        assert!(
            terminal.iter().any(|name| name.contains("#closure_spec")),
            "the chain should reach `terminal` through that capture list and copy it, but the dump \
             names only: {:?}",
            terminal
        );
    }
}
