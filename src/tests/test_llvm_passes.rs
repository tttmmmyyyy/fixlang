use crate::tests::test_util::fix_build_source_command;
use std::fs;
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
    let existing_pass = dir.path().join("existing_pass.txt");
    fs::write(&existing_pass, "default<O3>\n").expect("Failed to write the passes file");
    let missing_pass = dir.path().join("missing_pass.txt");
    fs::write(&missing_pass, "this-pass-does-not-exist\n")
        .expect("Failed to write the passes file");

    let first = fix_build_source_command(dir.path(), SOURCE, "basic")
        .arg("--llvm-passes-file")
        .arg(&existing_pass)
        .output()
        .expect("Failed to run the first build");
    assert!(
        first.status.success(),
        "the build with an existing pass should succeed, but it failed:\n{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = fix_build_source_command(dir.path(), SOURCE, "basic")
        .arg("--llvm-passes-file")
        .arg(&missing_pass)
        .output()
        .expect("Failed to run the second build");
    assert!(
        !second.status.success(),
        "the build naming a pass that does not exist should fail; that it succeeded means the \
         object files from the previous pass pipeline were reused"
    );
}
