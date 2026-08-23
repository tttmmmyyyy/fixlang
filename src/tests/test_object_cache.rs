//! What a build may reuse from a previous build in the same directory, and what it may not.
//!
//! A build records the object files it produced under a hash of everything they were generated
//! from, and a later build whose hash matches links them instead of generating anything. Every test
//! here therefore builds at least twice in one directory: the first build fills the cache and the
//! second is the one under test. A test that built once, as a test given a directory of its own
//! does, would leave the reuse untouched.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_command_at_opt_level;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::TempDir;

    /// A program reaching enough of `Std` for its build to be divided into several compilation
    /// units.
    const SOURCE: &str = r#"module Main;

main : IO ();
main = println $ Iterator::range(0, 10).map(|x| x * x).fold(0, Add::add).to_string;
"#;

    /// What `SOURCE` prints: the squares of `0..9` summed.
    const OUTPUT: &str = "285";

    /// A directory holding `SOURCE` as the whole of a project, ready to be built in.
    fn project_dir() -> TempDir {
        let dir = TempDir::new().expect("Failed to create temp directory");
        fs::write(dir.path().join("main.fix"), SOURCE).expect("Failed to write the source");
        fs::write(
            dir.path().join("fixproj.toml"),
            "[general]\nname = \"objcache\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"main.fix\"]\n",
        )
        .expect("Failed to write the project file");
        dir
    }

    /// Runs `command` in `dir` and returns what it wrote to its two streams, failing the test if the
    /// command does not succeed.
    fn run_in(command: &mut Command, dir: &Path, what: &str) -> String {
        let output = command
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute {}: {}", what, e));
        let streams = format!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success(), "{} should succeed.\n{}", what, streams);
        streams
    }

    /// The files directly under `dir` whose name ends in `suffix`.
    fn files_ending_in(dir: &Path, suffix: &str) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dir.display(), e))
            .map(|entry| entry.expect("failed to read a directory entry").path())
            .filter(|path| path.to_string_lossy().ends_with(suffix))
            .collect();
        paths.sort();
        paths
    }

    /// The RC IR, the LLVM IR and the symbols are written as the code they describe is generated, so
    /// a build that answered from the cache would write none of them. A build asked for a dump
    /// therefore generates the code again, and what it writes is of the build that was asked.
    #[test]
    fn test_a_build_asked_for_a_dump_writes_it_however_much_is_cached() {
        let dir = project_dir();
        let dir = dir.path();
        let dumps = dir.join(".fixlang");

        // A dump of one optimization level, so that a dump of another can be told from it.
        run_in(
            fix_command_at_opt_level("build", "basic").args(["--emit-rc-ir", "all"]),
            dir,
            "the build at -O basic",
        );
        let dump_at_basic = fs::read_to_string(dumps.join("rc_ir.post.txt"))
            .expect("the build at -O basic should write the RC IR");

        // A build of another level that asks for nothing, which is what fills the cache the build
        // under test answers from.
        run_in(
            &mut fix_command_at_opt_level("build", "none"),
            dir,
            "the build at -O none",
        );

        // The build under test: its object files are all cached, and it is asked for every dump.
        let asked_for_dumps = |dir: &Path, what: &str| {
            run_in(
                fix_command_at_opt_level("build", "none")
                    .args(["--emit-rc-ir", "all"])
                    .arg("--emit-llvm")
                    .arg("--emit-symbols"),
                dir,
                what,
            );
        };
        asked_for_dumps(dir, "the build at -O none asked for the dumps");
        let dump_at_none = fs::read_to_string(dumps.join("rc_ir.post.txt"))
            .expect("the build asked for the RC IR should write it");
        assert_ne!(
            dump_at_basic, dump_at_none,
            "the RC IR should be the one of the level the build was asked at"
        );
        assert!(
            !files_ending_in(dir, ".ll").is_empty(),
            "the build asked for the LLVM IR should write it"
        );
        assert!(
            !files_ending_in(&dumps, ".symbols.fix").is_empty(),
            "the build asked for the symbols should write them"
        );

        // The same build again, with what it wrote taken away: the cache now holds its own object
        // files, and the dumps are written all the same.
        fs::remove_file(dumps.join("rc_ir.pre.txt")).expect("failed to remove the RC IR dump");
        fs::remove_file(dumps.join("rc_ir.post.txt")).expect("failed to remove the RC IR dump");
        for path in files_ending_in(dir, ".ll") {
            fs::remove_file(path).expect("failed to remove an LLVM IR dump");
        }
        for path in files_ending_in(&dumps, ".symbols.fix") {
            fs::remove_file(path).expect("failed to remove a symbols dump");
        }
        asked_for_dumps(dir, "the build repeating the one asked for the dumps");
        assert!(
            dumps.join("rc_ir.post.txt").exists(),
            "a repeated build asked for the RC IR should write it again"
        );
        assert!(
            !files_ending_in(dir, ".ll").is_empty(),
            "a repeated build asked for the LLVM IR should write it again"
        );
        assert!(
            !files_ending_in(&dumps, ".symbols.fix").is_empty(),
            "a repeated build asked for the symbols should write them again"
        );
    }

    /// `fix build` and `fix run` compile the same program into the same code — the entry point of a
    /// test build is the one that differs — so each reuses the object files of the other.
    #[test]
    fn test_fix_run_and_fix_build_share_their_object_files() {
        let dir = project_dir();
        let dir = dir.path();

        let run = run_in(
            &mut fix_command_at_opt_level("run", "none"),
            dir,
            "the run of the program",
        );
        assert!(
            run.contains(OUTPUT),
            "the program should print {}.\n{}",
            OUTPUT,
            run
        );

        let build = run_in(
            fix_command_at_opt_level("build", "none").arg("--verbose"),
            dir,
            "the build after the run",
        );
        assert!(
            build.contains("Using cached object files."),
            "the build after the run should take the object files the run generated.\n{}",
            build
        );
    }

    /// The most symbols a compilation unit holds decides how many object files a build produces, so
    /// a build asking for a different division makes them rather than taking the ones a previous
    /// build left.
    #[test]
    fn test_a_build_dividing_itself_differently_generates_its_own_object_files() {
        let dir = project_dir();
        let dir = dir.path();

        let generated = |output: &str| output.matches("Generating object file for").count();

        let first = run_in(
            fix_command_at_opt_level("build", "basic").arg("--verbose"),
            dir,
            "the first build",
        );
        let units = generated(&first);
        assert!(units > 0, "the first build should generate its units.\n{}", first);

        let repeated = run_in(
            fix_command_at_opt_level("build", "basic").arg("--verbose"),
            dir,
            "the repeated build",
        );
        assert!(
            repeated.contains("Using cached object files."),
            "a build repeating another takes its object files.\n{}",
            repeated
        );

        let divided = run_in(
            fix_command_at_opt_level("build", "basic")
                .args(["--max-cu-size", "1"])
                .arg("--verbose"),
            dir,
            "the build dividing itself into units of one symbol",
        );
        assert!(
            !divided.contains("Using cached object files."),
            "a build dividing itself differently has object files of its own.\n{}",
            divided
        );
        assert!(
            generated(&divided) > units,
            "a build holding one symbol per unit generates more units than the {} of a build \
             holding the default.\n{}",
            units,
            divided
        );
    }

}
