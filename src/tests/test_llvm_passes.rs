use crate::configuration::{Configuration, FixOptimizationLevel};
use crate::tests::test_util::{emitted_llvm_ir, fix_build_source_command, fix_command, EmittedIr};
use std::fs;
use std::path::Path;
use std::process::Output;
use tempfile::TempDir;

/// A program small enough that the whole build is dominated by the pipeline under test, and whose
/// one global is called from `main` so that a broken build fails rather than skipping it.
const SOURCE: &str = r#"
module Main;

import Std::{I64, IO, Debug::assert_eq, Monad::pure};

double : I64 -> I64;
double = |x| x + x;

main : IO ();
main = (
    assert_eq(|_|"double", double(21), 42);;
    pure()
);
"#;

/// The pass name a build is given when the test wants the pipeline to fail on sight.
const MISSING_PASS: &str = "this-pass-does-not-exist";

/// Builds `SOURCE` in `dir` with `passes` as the LLVM pass pipeline, written to a file named
/// `passes_file_name`, and returns the output of the build. `extra_args` are appended to the
/// command line.
fn build_with_passes(
    dir: &Path,
    passes_file_name: &str,
    passes: &str,
    extra_args: &[&str],
) -> Output {
    let passes_file = dir.join(passes_file_name);
    fs::write(&passes_file, passes).expect("Failed to write the passes file");
    fix_build_source_command(dir, SOURCE, "basic")
        .arg("--llvm-passes-file")
        .arg(&passes_file)
        .args(extra_args)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run the build with \"{}\": {}", passes.trim(), e))
}

/// A build whose LLVM pass pipeline differs from the previous build's must generate its object
/// files again, rather than reuse the ones compiled under the previous pipeline.
///
/// The second build below names a pass that does not exist, which makes the pipeline fail as soon
/// as it runs. Reused objects skip the pipeline, so a second build that succeeds is one that
/// compiled nothing — and every comparison of two pipelines in the same directory would then be
/// reporting whichever one compiled first. The failure message is checked as well, so that a build
/// failing for an unrelated reason does not pass for a regenerated one.
#[test]
fn test_pass_pipeline_change_regenerates_objects() {
    let dir = TempDir::new().expect("Failed to create a temporary directory");

    let first = build_with_passes(dir.path(), "existing_pass.txt", "default<O3>\n", &[]);
    assert!(
        first.status.success(),
        "the build with an existing pass should succeed, but it failed:\n{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = build_with_passes(
        dir.path(),
        "missing_pass.txt",
        &format!("{}\n", MISSING_PASS),
        &[],
    );
    assert!(
        !second.status.success(),
        "the build naming a pass that does not exist should fail; that it succeeded means the \
         object files from the previous pass pipeline were reused"
    );
    let message = String::from_utf8_lossy(&second.stderr);
    assert!(
        message.contains(MISSING_PASS),
        "the build should have failed on the missing pass, but it reported:\n{}",
        message
    );
}

/// Two builds with the same pipeline must reuse the object files of the first, so that keying the
/// cache on the pass pipeline narrows what counts as a hit without abolishing caching altogether —
/// which would satisfy `test_pass_pipeline_change_regenerates_objects` just as well.
#[test]
fn test_same_pass_pipeline_reuses_objects() {
    let dir = TempDir::new().expect("Failed to create a temporary directory");

    let first = build_with_passes(dir.path(), "passes.txt", "default<O3>\n", &["-v"]);
    assert!(
        first.status.success(),
        "the first build should succeed, but it failed:\n{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = build_with_passes(dir.path(), "passes.txt", "default<O3>\n", &["-v"]);
    assert!(
        second.status.success(),
        "the second build should succeed, but it failed:\n{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let message = String::from_utf8_lossy(&second.stderr);
    assert!(
        message.contains("Using cached object files"),
        "the second build should have reused the first build's object files, but it reported:\n{}",
        message
    );
}

/// A passes file the compiler cannot read is reported as an error, so that a mistyped path stops
/// the build with the path in the message.
#[test]
fn test_unreadable_passes_file_is_reported() {
    let dir = TempDir::new().expect("Failed to create a temporary directory");
    let missing_file = dir.path().join("no_such_file.txt");

    let output = fix_build_source_command(dir.path(), SOURCE, "basic")
        .arg("--llvm-passes-file")
        .arg(&missing_file)
        .output()
        .expect("Failed to run the build");
    assert!(
        !output.status.success(),
        "a build given a passes file that does not exist should fail"
    );
    let message = String::from_utf8_lossy(&output.stderr);
    assert!(
        message.contains("no_such_file.txt"),
        "the error should name the file it failed to read, but it reported:\n{}",
        message
    );
}

/// The pipeline a passes file supplies replaces the one the optimization level implies, so an empty
/// file leaves the module unoptimized even at `-O max`: its stack slots survive, where a
/// `default<O3>` run would have promoted every one of them to a register.
#[test]
fn test_passes_file_replaces_the_level_pipeline() {
    let dir = TempDir::new().expect("Failed to create a temporary directory");
    let empty_passes = dir.path().join("empty_passes.txt");
    fs::write(&empty_passes, "").expect("Failed to write the passes file");
    let source_path = dir.path().join("generated.fix");
    fs::write(&source_path, SOURCE).expect("Failed to write the generated source file");

    let output = fix_command()
        .arg("build")
        .arg("--file")
        .arg(&source_path)
        .arg("-O")
        .arg("max")
        .arg("--emit-llvm")
        .arg("--llvm-passes-file")
        .arg(&empty_passes)
        .current_dir(dir.path())
        .output()
        .expect("Failed to run the build");
    assert!(
        output.status.success(),
        "a build with an empty passes file should succeed, but it failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // `--emit-llvm` writes the module both before and after the pipeline runs; the post-pipeline
    // file is the one that shows what the pipeline did.
    let emitted_ir = emitted_llvm_ir(dir.path(), EmittedIr::AfterOptimization);
    assert!(
        emitted_ir.contains("alloca"),
        "an unoptimized module should still hold its values in stack slots; the emitted IR has \
         none, so the passes file did not replace the pipeline the optimization level implies"
    );
}

/// The optimization levels built for speed turn the tail self-calls a Fix loop is written as into
/// loops, run LLVM's pipeline three times and follow it with the passes measured to earn their
/// place, `-O basic` runs the pipeline once, and `-O none` runs nothing, so that a level change is
/// visible in the pipeline rather than only in the timings.
#[test]
fn test_pass_pipeline_per_optimization_level() {
    // `set_fix_opt_level` clamps to `FIX_MAX_OPT_LEVEL`, which the test suite's matrix varies, so
    // the expectation is read off the level the configuration ended up at.
    let expected = |level: FixOptimizationLevel| -> Vec<String> {
        let full_pipeline = "default<O3>".to_string();
        match level {
            FixOptimizationLevel::None => vec![],
            FixOptimizationLevel::Basic => vec![full_pipeline],
            FixOptimizationLevel::Max | FixOptimizationLevel::Experimental => {
                let mut passes = vec!["function(tailcallelim)".to_string()];
                passes.extend(vec![full_pipeline; 3]);
                passes.extend(
                    ["speculative-execution", "loop-vectorize", "pseudo-probe"]
                        .iter()
                        .map(|pass| pass.to_string()),
                );
                passes
            }
        }
    };
    for level in [
        FixOptimizationLevel::None,
        FixOptimizationLevel::Basic,
        FixOptimizationLevel::Max,
        FixOptimizationLevel::Experimental,
    ] {
        let mut config = Configuration::develop_mode();
        config.set_fix_opt_level(level);
        assert_eq!(
            config.llvm_passes(),
            expected(config.fix_opt_level()),
            "unexpected pipeline at -O {}",
            config.fix_opt_level()
        );
    }
}
