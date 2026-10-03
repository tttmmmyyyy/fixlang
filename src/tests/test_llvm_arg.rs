//! The options the compiler and `--llvm-arg` hand to LLVM, and whether they still reach it.
//!
//! LLVM ignores an option it does not know. An option whose value LLVM cannot read gets a message
//! on the error stream and nothing more. The build succeeds either way, with the setting unmade, so
//! an option renamed between LLVM releases would stop taking effect while the build went on
//! succeeding, and a measurement taken with it would answer for a setting that was never made.
//! These tests read the effect out of the program the build produced.

#[cfg(test)]
mod tests {
    use crate::build::build_object_files::get_target_machine;
    use crate::configuration::Configuration;
    use crate::misc::{Map, Set};
    use crate::tests::test_util::{
        build_program, fix_build_source_command, fix_command_at_opt_level,
    };
    use inkwell::context::Context;
    use inkwell::memory_buffer::MemoryBuffer;
    use inkwell::targets::FileType;
    use inkwell::OptimizationLevel;
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

    /// Builds `source` with `build_args` on the build command, and answers the bytes of object
    /// code it wrote together with what the program prints. `description` names the program in a
    /// failure.
    fn build_and_run(source: &str, description: &str, build_args: &[&str]) -> (u64, String) {
        let (temp_dir, program_path) = build_program(source, "max", build_args, None, description);
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
        let (plain, plain_output) = build_and_run(ONE_LOOP, "a program of one loop", &[]);
        let (plain_again, _) = build_and_run(ONE_LOOP, "a program of one loop", &[]);
        assert_eq!(
            plain, plain_again,
            "two builds of one source should compile to the same bytes of object code"
        );

        let (aligned, aligned_output) = build_and_run(ONE_LOOP, "a program of one loop", &[ALIGN_ALL_BLOCKS_TO_64]);
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
        let (plain, _) = build_and_run(ONE_LOOP, "a program of one loop", &[]);
        let (with_unknown, _) = build_and_run(ONE_LOOP, "a program of one loop", &[OPTION_LLVM_DOES_NOT_HAVE]);
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

    /// A loop that packs three bits of each input byte into its output and leaves with the bits
    /// still pending, which gives the loop values that leave it through its exit block.
    const PACK_BITS: &str = r#"
        module Main;

        encode : Array U8 -> Array U8;
        encode = |input| (
            loop((0, 0, 0, Array::empty(0)), |(i, bits, nbits, out)|
                if i >= input.@size { break $ if nbits > 0 { out.push_back(bits.u8) } else { out } };
                let bits = bits.shift_left(3).bit_or(input.@(i).i64.bit_and(7));
                let nbits = nbits + 3;
                if nbits >= 8 {
                    continue $ (i + 1, bits.shift_right(nbits - 8), nbits - 8, out.push_back(bits.shift_right(nbits - 8).u8))
                };
                continue $ (i + 1, bits, nbits, out)
            )
        );

        main : IO ();
        main = (
            let n = (*IO::get_args).@size * 100000;
            let input = Array::from_map(n, |i| (i * 31 % 251).u8);
            println $ encode(input).@size.to_string
        );
    "#;

    /// An option `--llvm-arg` names is handed to LLVM after the ones every build hands it, so it
    /// can set again what those set: turning `-no-phi-elim-live-out-early-exit` back off changes
    /// the object code of a loop it acts on, and the program answers the same.
    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_llvm_arg_overrides_an_option_every_build_hands_llvm() {
        let build =
            |build_args: &[&str]| build_and_run(PACK_BITS, "a loop packing bits", build_args);
        let (default, default_output) = build(&[]);
        let (overridden, overridden_output) =
            build(&["--llvm-arg=-no-phi-elim-live-out-early-exit=false"]);
        assert_ne!(
            default, overridden,
            "turning the option back off with `--llvm-arg` should change the object code"
        );
        assert_eq!(
            default_output, overridden_output,
            "the program should answer the same with the option turned off as with it on"
        );
    }

    /// A loop whose length leaves it through a block holding nothing but a branch. The code
    /// generator deletes that block, so the copy of the length the exit needs has to find another
    /// place: on the edge out of the loop, or at the end of the loop's last block, where it runs
    /// on every iteration.
    const VALUE_LEAVING_A_LOOP: &str = r#"
        define i64 @fill(i1 %flag) {
        entry:
          br i1 %flag, label %head, label %exit

        head:
          %len = phi i64 [ %len.next, %latch ], [ 0, %entry ]
          %buf = phi ptr [ %buf.next, %latch ], [ null, %entry ]
          br i1 %flag, label %append, label %latch

        append:
          %len.grown = or i64 %len, 1
          %grows = icmp slt i64 0, %len.grown
          br i1 %grows, label %reset, label %appended

        reset:
          br label %appended

        appended:
          %buf.appended = phi ptr [ null, %reset ], [ %buf, %append ]
          %slot = getelementptr i8, ptr null, i64 %len
          store i8 0, ptr %slot, align 1
          br label %latch

        latch:
          %buf.next = phi ptr [ %buf.appended, %appended ], [ %buf, %head ]
          %len.next = phi i64 [ %len.grown, %appended ], [ 0, %head ]
          br i1 %flag, label %exit, label %head

        exit:
          %result = phi i64 [ 0, %entry ], [ %len.next, %latch ]
          ret i64 %result
        }
    "#;

    /// The register-to-register `mov` instructions of `assembly` that lie inside a loop, counting
    /// as a loop the instructions from the label of a backward branch to the branch.
    fn copies_inside_loops(assembly: &str) -> usize {
        let mut labels = Map::default();
        let mut instructions = vec![];
        for line in assembly.lines() {
            if let Some(label) = line.strip_suffix(':').filter(|l| l.starts_with(".LBB")) {
                labels.insert(label.to_string(), instructions.len());
            } else if line.starts_with('\t') && !line.starts_with("\t.") {
                instructions.push(line.trim().split('#').next().unwrap().trim().to_string());
            }
        }
        let mut inside = Set::default();
        for (index, instruction) in instructions.iter().enumerate() {
            if !instruction.starts_with('j') {
                continue;
            }
            let target = instruction.split_whitespace().nth(1);
            if let Some(&start) = target.and_then(|target| labels.get(target)) {
                if start <= index {
                    inside.extend(start..=index);
                }
            }
        }
        inside
            .into_iter()
            .filter(|&index| {
                let mut parts = instructions[index].splitn(2, char::is_whitespace);
                let opcode = parts.next().unwrap();
                let operands: Vec<&str> = parts
                    .next()
                    .unwrap_or("")
                    .split(',')
                    .map(str::trim)
                    .collect();
                opcode.starts_with("mov")
                    && operands.len() == 2
                    && operands.iter().all(|operand| operand.starts_with('%'))
            })
            .count()
    }

    /// The copy of a value leaving a loop is made on the way out of the loop rather than on every
    /// iteration: the loop of `VALUE_LEAVING_A_LOOP` keeps the two copies its `or` needs and no
    /// third. This is what `LLVM_DEFAULT_OPTIONS` asks of LLVM, so it fails where LLVM renames the
    /// option or drops it, which LLVM would otherwise do without a word.
    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_a_value_leaving_a_loop_is_copied_outside_it() {
        let config = Configuration::develop_mode();
        let target_machine = get_target_machine(OptimizationLevel::Aggressive, &config);
        let context = Context::create();
        let buffer = MemoryBuffer::create_from_memory_range_copy(
            VALUE_LEAVING_A_LOOP.as_bytes(),
            "value_leaving_a_loop",
        );
        let module = context
            .create_module_from_ir(buffer)
            .expect("the test's IR should parse");
        module.set_triple(&target_machine.get_triple());
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        let assembly = target_machine
            .write_to_memory_buffer(&module, FileType::Assembly)
            .expect("LLVM should compile the test's IR");
        let assembly = String::from_utf8_lossy(assembly.as_slice()).to_string();
        assert_eq!(
            copies_inside_loops(&assembly),
            2,
            "the loop should copy only what its `or` needs, and leave the copy of the value it \
             returns to the edge out of it:\n{}",
            assembly
        );
    }
}
