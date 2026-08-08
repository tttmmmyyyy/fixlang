//! The two techniques of the closure specialization pass, read off the `--emit-rc-ir` dump: a
//! lambda is lifted to a global function, and the function it is passed to gets a copy that calls
//! that function by name. A recursion that hands the next round a closure built from the one it
//! was given could ask for one copy per round, and runs out instead.
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

    /// What `derived_closure` prints: `relay` sums `terminal(shifted, i)` over `0..n`, adds
    /// `shifted(n)`, and recurses on `n - 1`, with `shifted = |x| x * 3 + 1` and `n = 4`.
    const DERIVED_CLOSURE_OUTPUT: &str = "89";

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

    /// A closure a function builds from the one it was given becomes a capture list, and the lambda
    /// that carries it into `fold` holds it in a capture field. Specializing the function narrows
    /// the inner capture list, so what that field holds changes type — and a field that cannot
    /// follow has no closure to fall back on, since it is not one.
    #[test]
    pub fn test_a_capture_field_follows_the_value_it_holds() {
        let (_temp_dir, project_dir) = setup_test_env("derived_closure");
        build_run_and_read_rc_ir(&project_dir, "max", DERIVED_CLOSURE_OUTPUT);
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
