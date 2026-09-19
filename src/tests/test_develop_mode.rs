//! `--develop-mode` builds a program with the compiler's own consistency checks in it. The checks
//! cost run time and stop the program where one of them fails, and they are what the compiler's own
//! test suite builds under; the option is what builds a project of one's own the same way.
//!
//! A program that passes the checks answers what one built without them answers, so what is read
//! here is that each subcommand building a program takes the option, and that what it built runs.

use crate::tests::test_util::fix_command_at_opt_level;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

/// The body the program runs: a boxed value whose references are counted, an array written through,
/// and a string literal, whose bytes are exempt from counting.
const BODY: &str = r#"
    let boxed = Box::make([1, 2, 3]);
    let written = boxed.@value.set(0, 7);
    println $ "the answer is " + written.to_iter.sum.to_string
"#;

/// What the program prints, whichever subcommand built it.
const EXPECTED: &str = "the answer is 12";

/// A module named `module` whose `entry` runs `BODY`.
fn source_for(module: &str, entry: &str) -> String {
    format!(
        "module {module};\n\n{entry} : IO ();\n{entry} = ({body});\n",
        module = module,
        entry = entry,
        body = BODY,
    )
}

/// Runs `fix <subcommand> --develop-mode` over `source`, written into `dir`.
fn run_fix_on_source(subcommand: &str, source: &str, dir: &Path, extra_args: &[&str]) -> Output {
    let src_path = dir.join("generated.fix");
    fs::write(&src_path, source).expect("Failed to write the generated source file");
    fix_command_at_opt_level(subcommand, "max")
        .arg("--file")
        .arg(&src_path)
        .arg("--develop-mode")
        .args(extra_args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute fix")
}

/// Every subcommand that builds a program takes `--develop-mode`, and what it builds runs and
/// answers.
#[test]
pub fn test_every_building_subcommand_takes_develop_mode() {
    for (subcommand, module, entry) in [("run", "Main", "main"), ("test", "Test", "test")] {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output =
            run_fix_on_source(subcommand, &source_for(module, entry), temp_dir.path(), &[]);
        assert!(
            output.status.success(),
            "`fix {} --develop-mode` should succeed.\nstdout: {}\nstderr: {}",
            subcommand,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            EXPECTED,
            "`fix {} --develop-mode` should run the program the option built",
            subcommand,
        );
    }

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let program_path = temp_dir.path().join("program");
    let output = run_fix_on_source(
        "build",
        &source_for("Main", "main"),
        temp_dir.path(),
        &["-o", program_path.to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "`fix build --develop-mode` should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let run = Command::new(&program_path)
        .output()
        .expect("Failed to run the program the build produced");
    assert!(
        run.status.success(),
        "the program `fix build --develop-mode` produced should run.\nstderr: {}",
        String::from_utf8_lossy(&run.stderr),
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        EXPECTED,
        "the program `fix build --develop-mode` produced should answer",
    );
}
