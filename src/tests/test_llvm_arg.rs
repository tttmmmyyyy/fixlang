//! The options `--llvm-arg` hands to LLVM, and whether they still reach it.
//!
//! LLVM takes an option it does not know without a word, so an option renamed between LLVM
//! releases stops taking effect rather than stopping the build. A measurement taken with one would
//! then report the effect of a setting that was never made. These tests read the effect out of the
//! program the build produced rather than trusting the option.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::fix_command_at_opt_level;
    use std::fs;
    use std::path::Path;
    use std::process::Stdio;
    use tempfile::TempDir;

    /// The option that asks LLVM for a 64-byte boundary (`2^6`) at the head of every innermost
    /// loop. A loop that starts on one is fed faster by the front end, and where it otherwise
    /// starts is decided by the sizes of everything placed before it, so this is what a
    /// measurement uses to hold that still.
    const ALIGN_INNERMOST_LOOPS_TO_64: &str =
        "--llvm-arg=--x86-experimental-pref-innermost-loop-alignment=6";

    /// How many loops the program holds. Each one asked for a wider boundary takes up to the extra
    /// boundary in padding, so the program has to hold enough of them for the padding to outweigh
    /// what a build varies by, which is nothing: two builds of one source come out the same size.
    const LOOPS: usize = 120;

    /// A program of `LOOPS` loops, each written so that it survives to the machine code: the count
    /// comes from the arguments, so no round of a loop can be folded away, and every result is
    /// added into what is printed, so no loop is dead.
    fn program_of_many_loops() -> String {
        let mut source = String::from("module Main;\n\n");
        for i in 0..LOOPS {
            source += &format!(
                "f{i} : I64 -> I64;\n\
                 f{i} = |n| Iterator::range(0, n).fold({i}, |j, acc| acc + j * j % {m});\n\n",
                i = i,
                m = i + 3
            );
        }
        let sum = (0..LOOPS)
            .map(|i| format!("f{}(n)", i))
            .collect::<Vec<_>>()
            .join(" + ");
        source += &format!(
            "main : IO ();\n\
             main = (\n\
             \x20   let n = (*IO::get_args).@size * 10;\n\
             \x20   println $ ({}).to_string\n\
             );\n",
            sum
        );
        source
    }

    /// Builds `source` with `build_args` on the build command, and answers the size of the program
    /// it produced together with what the program prints.
    fn build_and_run(dir: &Path, source: &str, name: &str, build_args: &[&str]) -> (u64, String) {
        let source_path = dir.join("main.fix");
        fs::write(&source_path, source).expect("Failed to write the source");
        let program_path = dir.join(name);

        // Each build starts from the sources alone, since the point is what this compiler and
        // these options make of them rather than what another build left behind.
        let _ = fs::remove_dir_all(dir.join(".fixlang"));
        let status = fix_command_at_opt_level("build", "max")
            .arg("--file")
            .arg(&source_path)
            .args(build_args)
            .arg("-o")
            .arg(&program_path)
            .current_dir(dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to run the build");
        assert!(status.success(), "the build failed: {}", status);

        let size = fs::metadata(&program_path)
            .expect("Failed to read the program's size")
            .len();
        let output = std::process::Command::new(&program_path)
            .output()
            .expect("Failed to run the program");
        assert!(
            output.status.success(),
            "the program failed: {}",
            output.status
        );
        (size, String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// An option `--llvm-arg` hands to LLVM reaches it: asking for a wider boundary at the head of
    /// every loop produces a larger program, since each boundary is reached by padding.
    ///
    /// This is what fails where the option LLVM offers is renamed. LLVM takes an unknown option
    /// without a word, so a build asking for the boundary would otherwise go on not getting it,
    /// and every measurement taken with it would answer for a setting that was never made.
    ///
    /// Two builds of one source come out the same size, which is what makes a difference in size
    /// the option's doing; `test_two_builds_of_one_source_come_out_the_same_size` holds that.
    #[test]
    fn test_llvm_arg_reaches_llvm() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let source = program_of_many_loops();
        let (plain, plain_output) = build_and_run(temp_dir.path(), &source, "plain", &[]);
        let (aligned, aligned_output) = build_and_run(
            temp_dir.path(),
            &source,
            "aligned",
            &[ALIGN_INNERMOST_LOOPS_TO_64],
        );

        assert!(
            aligned > plain,
            "asking for a 64-byte boundary at the head of {} loops should grow the program, but \
             it is {} bytes against {}",
            LOOPS,
            aligned,
            plain
        );
        assert_eq!(
            plain_output, aligned_output,
            "the program should answer the same with the boundary asked for as without it"
        );
    }

    /// Two builds of one source produce programs of one size, which is what lets a difference in
    /// size be read as an option's doing.
    #[test]
    fn test_two_builds_of_one_source_come_out_the_same_size() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let source = program_of_many_loops();
        let (first, _) = build_and_run(temp_dir.path(), &source, "first", &[]);
        let (second, _) = build_and_run(temp_dir.path(), &source, "second", &[]);
        assert_eq!(
            first, second,
            "two builds of one source should come out the same size"
        );
    }

    /// An option LLVM does not know leaves the program as it was, which is the behavior the test
    /// above exists to catch: LLVM says nothing about it, and the build carries on.
    #[test]
    fn test_an_option_llvm_does_not_know_leaves_the_program_alone() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let source = program_of_many_loops();
        let (plain, _) = build_and_run(temp_dir.path(), &source, "plain", &[]);
        let (with_unknown, _) = build_and_run(
            temp_dir.path(),
            &source,
            "unknown",
            &["--llvm-arg=--fixlang-test-option-llvm-does-not-have=1"],
        );
        assert_eq!(
            plain, with_unknown,
            "an option LLVM does not know should leave the program as it was"
        );
    }
}
