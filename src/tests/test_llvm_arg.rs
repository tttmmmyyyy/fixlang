//! The options `--llvm-arg` hands to LLVM, and whether they still reach it.
//!
//! LLVM takes an option it does not know without a word, and an option whose value it cannot read
//! with a message and nothing else, so a build goes on either way without the setting. An option
//! renamed between LLVM releases would therefore stop taking effect rather than stopping the
//! build, and a measurement taken with it would answer for a setting that was never made. These
//! tests read the effect out of the program the build produced rather than trusting the option.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_program;
    use std::fs;
    use std::process::Command;

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

    /// Builds `ONE_LOOP` with `build_args` on the build command, and answers the size of the
    /// program it produced together with what the program prints.
    ///
    /// The size is what says whether an option took effect. The machine code itself would say
    /// more, but reading it takes a tool that is not on every platform the compiler builds on, and
    /// the whole program does not compare byte for byte: the names a build mints carry a random
    /// part. Its size carries none, so two builds of one source come out at one size.
    fn build_and_run(build_args: &[&str]) -> (u64, String) {
        let (_temp_dir, program_path) =
            build_program(ONE_LOOP, "max", build_args, None, "a program of one loop");
        let size = fs::metadata(&program_path)
            .expect("Failed to read the program's size")
            .len();
        let output = Command::new(&program_path)
            .output()
            .expect("Failed to run the program");
        assert!(
            output.status.success(),
            "the program failed: {}",
            output.status
        );
        (size, String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// An option `--llvm-arg` hands to LLVM reaches it: asking for a boundary at the head of every
    /// basic block produces a larger program, since each boundary is reached by padding. The
    /// program answers the same either way, which is what says the option moved the code rather
    /// than the computation.
    ///
    /// This is what fails where the option LLVM offers is renamed. LLVM takes an unknown option
    /// without a word, so a build asking for a setting would otherwise go on not getting it, and
    /// every measurement taken with it would answer for a setting that was never made. The two
    /// builds without the option are what make the difference in size the option's doing.
    #[test]
    fn test_llvm_arg_reaches_llvm() {
        let (plain, plain_output) = build_and_run(&[]);
        let (plain_again, _) = build_and_run(&[]);
        assert_eq!(
            plain, plain_again,
            "two builds of one source should come out the same size"
        );

        let (aligned, aligned_output) = build_and_run(&[ALIGN_ALL_BLOCKS_TO_64]);
        assert!(
            aligned > plain,
            "asking for a 64-byte boundary at the head of every block should grow the program, \
             but it is {} bytes against {}",
            aligned,
            plain
        );
        assert_eq!(
            plain_output, aligned_output,
            "the program should answer the same with the boundary asked for as without it"
        );
    }

    /// An option LLVM does not know leaves the program as it was. That is the behavior the test
    /// above exists to catch, and it earns a test of its own because it is what a renamed option
    /// does: LLVM says nothing about it, and the build succeeds.
    #[test]
    fn test_an_option_llvm_does_not_know_leaves_the_program_alone() {
        let (plain, _) = build_and_run(&[]);
        let (with_unknown, _) = build_and_run(&[OPTION_LLVM_DOES_NOT_HAVE]);
        assert_eq!(
            plain, with_unknown,
            "an option LLVM does not know should leave the program as it was"
        );
    }
}
