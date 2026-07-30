use crate::tests::test_util::fix_build_source_command;
use std::fs;
use std::path::Path;
use std::process::Output;
use tempfile::TempDir;

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

/// Builds `SOURCE` in `dir` with `passes` as the LLVM pass pipeline, written to a file named
/// `passes_file_name`, and returns the output of the build.
fn build_with_passes(dir: &Path, passes_file_name: &str, passes: &str) -> Output {
    let passes_file = dir.join(passes_file_name);
    fs::write(&passes_file, passes).expect("Failed to write the passes file");
    fix_build_source_command(dir, SOURCE, "basic")
        .arg("--llvm-passes-file")
        .arg(&passes_file)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run the build with \"{}\": {}", passes.trim(), e))
}

/// A build whose LLVM pass pipeline differs from the previous build's must generate its object
/// files again, rather than reuse the ones compiled under the previous pipeline.
///
/// The second build below names a pass that does not exist, which makes the pipeline fail as soon
/// as it runs. Reused objects skip the pipeline, so a second build that succeeds is one that
/// compiled nothing — and every comparison of two pipelines in the same directory would then be
/// reporting whichever one compiled first.
#[test]
fn test_pass_pipeline_change_regenerates_objects() {
    let dir = TempDir::new().expect("Failed to create a temporary directory");

    let first = build_with_passes(dir.path(), "existing_pass.txt", "default<O3>\n");
    assert!(
        first.status.success(),
        "the build with an existing pass should succeed, but it failed:\n{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = build_with_passes(dir.path(), "missing_pass.txt", "this-pass-does-not-exist\n");
    assert!(
        !second.status.success(),
        "the build naming a pass that does not exist should fail; that it succeeded means the \
         object files from the previous pass pipeline were reused"
    );
}
