//! The two techniques of the closure specialization pass, read off the `--emit-rc-ir` dump: a
//! lambda is lifted to a global function, and the function it is passed to gets a copy that calls
//! that function by name. A recursion that hands the next round a closure built from the one it
//! was given gets no such copy.
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
    fn functions_named_with<'a>(dump: &'a str, name_part: &str) -> Vec<&'a str> {
        dump.lines()
            .filter_map(|line| line.strip_prefix("fn "))
            .map(|rest| rest.split('(').next().unwrap().trim())
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

    /// The pass specializes a function on a closure parameter only where the recursion passes that
    /// parameter on unchanged, so a recursion that wraps it on every round gets no specialized
    /// copy — neither where the recursion is a function's own nor where it goes around a cycle of
    /// two. The lifted lambdas are what show the pass looked at these functions at all.
    #[test]
    pub fn test_a_recursion_carrying_a_new_closure_each_round_is_not_specialized_on() {
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
            specialized.is_empty(),
            "neither recursion passes its closure parameter on unchanged, so the pass should mint \
             none of: {:?}",
            specialized
        );
    }
}
