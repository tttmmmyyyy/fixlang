//! What `fix build --verbose` reports.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{fix_command_at_opt_level, run_in, single_source_project_dir};

    /// Verifies that `--verbose` reports how long the steps of the build took: the type check, the
    /// optimization and the object files, each as a line naming the step and its time in seconds.
    #[test]
    fn test_verbose_reports_the_time_of_each_step() {
        let dir = single_source_project_dir(
            "verbose",
            "module Main;\n\nmain : IO ();\nmain = println(\"hi\");\n",
        );
        let output = run_in(
            fix_command_at_opt_level("build", "basic").arg("--verbose"),
            dir.path(),
            "`fix build --verbose`",
        );
        for step in ["typecheck", "optimization::run", "build_object_files"] {
            let line_start = format!("\n{}: ", step);
            assert!(
                output.split(&line_start).nth(1).is_some_and(|rest| rest
                    .split_once('\n')
                    .is_some_and(|(time, _)| time.ends_with(" sec"))),
                "`--verbose` should report the time of `{}`.\n{}",
                step,
                output
            );
        }
    }
}
