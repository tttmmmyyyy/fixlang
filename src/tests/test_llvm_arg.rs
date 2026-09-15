//! The options `--llvm-arg` hands to LLVM, and whether they still reach it.
//!
//! LLVM ignores an option it does not know. An option whose value LLVM cannot read gets a message
//! on the error stream and nothing more. The build succeeds either way, with the setting unmade, so
//! an option renamed between LLVM releases would stop taking effect while the build went on
//! succeeding, and a measurement taken with it would answer for a setting that was never made.
//! These tests read the effect out of the program the build produced.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{
        build_program, fix_build_source_command, fix_command_at_opt_level,
    };
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    /// An option that asks LLVM to start every basic block on a 64-byte boundary (`2^6`). It is
    /// read by the block placement every target shares, so what it does here is what it does
    /// wherever the compiler runs, and it reaches that placement through the same parser as the
    /// options a measurement uses.
    const ALIGN_ALL_BLOCKS_TO_64: &str = "--llvm-arg=--align-all-blocks=6";

    /// An option no LLVM has, which is what a renamed one looks like from here.
    const OPTION_LLVM_DOES_NOT_HAVE: &str = "--llvm-arg=--fixlang-test-option-llvm-does-not-have=1";

    /// A program of one loop, written so that the loop survives to the machine code: the count
    /// comes from the arguments, so no round of it can be folded away, and the answer is printed,
    /// so none of it is dead.
    const ONE_LOOP: &str = r#"
        module Main;

        main : IO ();
        main = (
            let n = (*IO::get_args).@size * 1000;
            let total = Iterator::range(0, n).fold(0, |i, acc| acc + i * i % 7);
            println $ total.to_string
        );
    "#;

    /// The bytes of object code a build wrote, taken from the object files it left under
    /// `dir`. That total is what says whether an option took effect.
    ///
    /// The linked program says nothing: the compiler writes the runtime's C source under a name
    /// carrying a number it mints per build, the name reaches the program's string table, and its
    /// length reaches the program's size. Two builds of one source therefore come out at two sizes
    /// often enough to measure -- four of ten at one size and six at another, for one program
    /// measured here. The object files the Fix code compiles to carry no such name and come out at
    /// one size over ten builds.
    fn object_code_size(dir: &Path) -> u64 {
        let units = dir.join(".fixlang").join("intermediate").join("units");
        let mut total = 0;
        for entry in fs::read_dir(&units).expect("Failed to read the directory of object files") {
            let entry = entry.expect("Failed to read an object file");
            if entry.path().extension().is_some_and(|ext| ext == "o") {
                total += entry
                    .metadata()
                    .expect("Failed to read an object file")
                    .len();
            }
        }
        assert!(
            total > 0,
            "the build wrote no object file under {:?}",
            units
        );
        total
    }

    /// Builds `ONE_LOOP` with `build_args` on the build command, and answers the bytes of object
    /// code it wrote together with what the program prints.
    fn build_and_run(build_args: &[&str]) -> (u64, String) {
        let (temp_dir, program_path) =
            build_program(ONE_LOOP, "max", build_args, None, "a program of one loop");
        let size = object_code_size(temp_dir.path());
        let output = Command::new(&program_path)
            .output()
            .expect("Failed to run the program");
        assert!(
            output.status.success(),
            "the program failed: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        (size, String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Builds `ONE_LOOP` in `dir` with `build_args` written between the source and `-o`, and
    /// answers the bytes of object code it wrote together with what it put on the error stream.
    ///
    /// The options go before `-o`, so a `--llvm-arg` among them is followed by an option of the
    /// compiler's. Finding the program at the path after `-o` is what says that option was read as
    /// one: an argument taking several values at once would take `-o` and the path as two more of
    /// them and write the program elsewhere.
    fn build_in(dir: &Path, program_name: &str, build_args: &[&str]) -> (u64, String) {
        let program_path = dir.join(program_name);
        // Each build compiles the sources again, since what is being compared is what this set of
        // options makes of them rather than what the build before it left.
        let _ = fs::remove_dir_all(dir.join(".fixlang").join("intermediate"));
        let output = fix_build_source_command(dir, ONE_LOOP, "max")
            .args(build_args)
            .arg("-o")
            .arg(&program_path)
            .output()
            .expect("Failed to execute fix build");
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        assert!(
            output.status.success(),
            "the build failed: {}\n{}",
            output.status,
            stderr
        );
        fs::metadata(&program_path)
            .expect("the build should write its program at the path after `-o`");
        (object_code_size(dir), stderr)
    }

    /// An option `--llvm-arg` hands to LLVM reaches it: asking for a boundary at the head of every
    /// basic block produces more object code, since each boundary is reached by padding. The
    /// program answers the same either way, which is what says the option moved the code rather
    /// than the computation.
    ///
    /// This is what fails where the option LLVM offers is renamed. LLVM ignores an unknown option,
    /// so a build asking for a setting would otherwise go on without it, and every measurement
    /// taken with it would answer for a setting that was never made. The two builds without the
    /// option are what make the difference in size the option's doing.
    #[test]
    fn test_llvm_arg_reaches_llvm() {
        let (plain, plain_output) = build_and_run(&[]);
        let (plain_again, _) = build_and_run(&[]);
        assert_eq!(
            plain, plain_again,
            "two builds of one source should compile to the same bytes of object code"
        );

        let (aligned, aligned_output) = build_and_run(&[ALIGN_ALL_BLOCKS_TO_64]);
        assert!(
            aligned > plain,
            "asking for a 64-byte boundary at the head of every block should grow the object \
             code, but it is {} bytes against {}",
            aligned,
            plain
        );
        assert_eq!(
            plain_output, aligned_output,
            "the program should answer the same with the boundary asked for as without it"
        );
    }

    /// An option LLVM does not know leaves the program as it was. That is the behavior
    /// `test_llvm_arg_reaches_llvm` exists to catch, and it earns a test of its own because it is
    /// what a renamed option does: LLVM says nothing about it, and the build succeeds.
    #[test]
    fn test_an_option_llvm_does_not_know_leaves_the_program_alone() {
        let (plain, _) = build_and_run(&[]);
        let (with_unknown, _) = build_and_run(&[OPTION_LLVM_DOES_NOT_HAVE]);
        assert_eq!(
            plain, with_unknown,
            "an option LLVM does not know should leave the object code as it was"
        );
    }

    /// `--llvm-arg` takes one value, so an option written after it is read as an option, and the
    /// value may be written after `=` or after a space.
    ///
    /// An option of LLVM's opens with a hyphen, which is why the argument allows a hyphenated
    /// value. An argument taking several values at once -- the shape of every other repeated option
    /// beside it -- would read `-o` and the path after it as two more values.
    #[test]
    fn test_the_option_takes_one_value_so_an_option_after_it_is_still_read() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();

        let (plain, _) = build_in(dir, "plain", &[]);
        let (after_equals, _) = build_in(dir, "after-equals", &[ALIGN_ALL_BLOCKS_TO_64]);
        let (after_space, _) =
            build_in(dir, "after-space", &["--llvm-arg", "--align-all-blocks=6"]);

        assert!(
            after_equals > plain,
            "the option should reach LLVM, but the object code is {} bytes against {}",
            after_equals,
            plain
        );
        assert_eq!(
            after_space, after_equals,
            "a value written after a space should name the option a value written after `=` names"
        );
    }

    /// `--llvm-arg` may be written more than once, and every occurrence reaches LLVM. The
    /// occurrence beside the one under test carries an option LLVM ignores, so the size of the
    /// program answers for the other occurrence alone.
    #[test]
    fn test_every_occurrence_of_the_option_reaches_llvm() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();

        let (plain, _) = build_in(dir, "plain", &[]);
        let (alone, _) = build_in(dir, "alone", &[ALIGN_ALL_BLOCKS_TO_64]);
        assert!(
            alone > plain,
            "the option should reach LLVM on its own, but the object code is {} bytes against {}",
            alone,
            plain
        );

        let (second, _) = build_in(
            dir,
            "second",
            &[OPTION_LLVM_DOES_NOT_HAVE, ALIGN_ALL_BLOCKS_TO_64],
        );
        assert_eq!(
            second, alone,
            "the second of two occurrences should reach LLVM"
        );

        let (first, _) = build_in(
            dir,
            "first",
            &[ALIGN_ALL_BLOCKS_TO_64, OPTION_LLVM_DOES_NOT_HAVE],
        );
        assert_eq!(
            first, alone,
            "the first of two occurrences should reach LLVM"
        );
    }

    /// An option whose value LLVM cannot read leaves the setting unmade and lets the build run to
    /// the end: LLVM reports it on the error stream, the build succeeds, and the program comes out
    /// as it would have without the option. That is why the help of `--llvm-arg` tells a user to
    /// compare the programs.
    ///
    /// The report opens with `fix --llvm-arg`, the name `set_llvm_options` hands LLVM for itself,
    /// which is what marks the message as LLVM's.
    #[test]
    fn test_an_option_whose_value_llvm_cannot_read_is_reported_and_the_build_goes_on() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();

        let (plain, _) = build_in(dir, "plain", &[]);
        let (with_bad_value, stderr) =
            build_in(dir, "bad-value", &["--llvm-arg=--align-all-blocks=six"]);

        assert_eq!(
            with_bad_value, plain,
            "a value LLVM cannot read should leave the object code as it was"
        );
        assert!(
            stderr.contains("fix --llvm-arg"),
            "LLVM's report should name the option the value came from, but the build said: {}",
            stderr
        );
    }

    /// An option `--llvm-arg` hands LLVM reaches the code a `fix run` generates, and not only the
    /// command line it is accepted on. The option is on the subcommands that build a program and
    /// then run it as well as on `build`, which is where a measurement through `fix run` needs it.
    #[test]
    fn test_the_option_reaches_the_code_a_run_generates() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();
        let source_path = dir.join("generated.fix");
        fs::write(&source_path, ONE_LOOP).expect("Failed to write the generated source file");

        let run = |build_args: &[&str]| {
            let _ = fs::remove_dir_all(dir.join(".fixlang").join("intermediate"));
            let output = fix_command_at_opt_level("run", "max")
                .arg("--file")
                .arg(&source_path)
                .args(build_args)
                .current_dir(dir)
                .output()
                .expect("Failed to execute fix run");
            assert!(
                output.status.success(),
                "`fix run` failed: {}\n{}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            (
                object_code_size(dir),
                String::from_utf8_lossy(&output.stdout).to_string(),
            )
        };

        let (plain, plain_output) = run(&[]);
        let (aligned, aligned_output) = run(&[ALIGN_ALL_BLOCKS_TO_64]);
        assert!(
            aligned > plain,
            "asking a run for a 64-byte boundary at the head of every block should grow the object \
             code it generates, but it is {} bytes against {}",
            aligned,
            plain
        );
        assert_eq!(
            plain_output, aligned_output,
            "`fix run` should answer the same with the option as without it"
        );
    }
}
