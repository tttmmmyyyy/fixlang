//! A run that hits the type-check cache must produce what a run that missed it produces. A hit
//! skips name resolution and type checking altogether, so whatever the entry gets wrong is served
//! without a word: a body the value never declared, a stale answer after an edit, or a report the
//! build owed the user.

use crate::{configuration::Configuration, tests::test_util::test_source};

/// Declaring a struct gives its field `b` an accessor named `@b`, so a value named `_b` in the
/// same namespace differs from the accessor only in a character no file name can carry. Both are
/// read here, and each must answer with its own body.
#[test]
fn test_an_accessor_and_a_value_differing_only_in_punctuation_keep_their_own_bodies() {
    let source = r#"
        module Main;

        type S = unbox struct { b : I64 };

        namespace S {
            _b : S -> I64;
            _b = |_s| 999;
        }

        main : IO ();
        main = (
            let s = S { b : 1 };
            assert_eq(|_| "the field accessor", s.@b, 1);;
            assert_eq(|_| "the value named like the accessor", s._b, 999);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_command;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    /// The version hash of a cache entry comes from the sources of the module the value's name
    /// begins with, so a value has to belong to the module whose file it is written in. Opening a
    /// namespace named after another module nests it under the module that opens it, and an edit
    /// to the value's body reaches its key.
    #[test]
    fn a_value_under_a_namespace_named_after_another_module_is_rechecked_after_an_edit() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let source_path = temp.path().join("main.fix");
        let source = |answer: &str| {
            format!(
                r#"
                module Main;

                namespace Std {{
                    answer : I64;
                    answer = {};
                }}

                main : IO ();
                main = println $ Std::answer.to_string;
            "#,
                answer
            )
        };
        let run = || {
            let output = fix_command()
                .args(["run", "--file", "main.fix"])
                .current_dir(temp.path())
                .output()
                .expect("Failed to execute fix run");
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        fs::write(&source_path, source("1")).expect("Failed to write main.fix");
        assert_eq!(run(), "1");

        fs::write(&source_path, source("2")).expect("Failed to write main.fix");
        assert_eq!(
            run(),
            "2",
            "the edited body is what the second run compiles"
        );
    }

    /// Uses a deprecated value from a value that is not itself deprecated, so building the program
    /// owes the user one deprecation warning.
    const DEPRECATED_USE: &str = r#"module Main;

DEPRECATED[old_val, "use `new_val` instead"];

old_val : I64;
old_val = 1;

new_val : I64;
new_val = 2;

main : IO ();
main = println((old_val + new_val).to_string);
"#;

    /// The content of a file a cache entry points into, as a later build finds it: short enough
    /// that the spans of `DEPRECATED_USE` fall past its end.
    const SHORTER_SOURCE: &str = r#"module Main;

main : IO ();
main = println("s");
"#;

    /// A temporary directory holding `a/main.fix` and `b/main.fix`, two files whose content is
    /// `DEPRECATED_USE`. Built from that directory, the two share the cache it holds.
    fn two_files_of_equal_content() -> TempDir {
        let temp = TempDir::new().expect("Failed to create temp directory");
        for sub_dir in ["a", "b"] {
            let sub_dir = temp.path().join(sub_dir);
            fs::create_dir(&sub_dir).expect("Failed to create the directory of a file");
            fs::write(sub_dir.join("main.fix"), DEPRECATED_USE).expect("Failed to write main.fix");
        }
        temp
    }

    /// Builds the Fix source `file`, named as the working directory `dir` reaches it, passing
    /// `extra_args` to `fix build` as well, and returns what the compiler wrote to stderr. The
    /// program is written to `out` in that directory.
    fn build(dir: &Path, file: &str, extra_args: &[&str]) -> String {
        let out = fix_command()
            .args(["build", "--file", file, "-o", "out"])
            .args(extra_args)
            .current_dir(dir)
            .output()
            .expect("failed to run fix build");
        assert!(
            out.status.success(),
            "fix build failed in {}:\n{}",
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stderr).to_string()
    }

    /// A deprecation warning is collected after type checking, out of the typed expression and the
    /// span it carries — which on the second build comes from the cache. Both builds owe the user
    /// the same warning, anchored in the file the use is written in.
    #[test]
    fn a_warm_cache_keeps_the_deprecation_warning() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(dir.join("main.fix"), DEPRECATED_USE).expect("Failed to write main.fix");

        let cold = build(dir, "main.fix", &[]);
        assert!(
            cold.contains("`Main::old_val` is deprecated"),
            "the first build must report the deprecated use.\nstderr: {}",
            cold
        );
        assert!(
            cold.contains("in \"main.fix\""),
            "the first build must attribute the use to the file it is written in.\nstderr: {}",
            cold
        );

        let warm = build(dir, "main.fix", &[]);
        assert!(
            warm.contains("`Main::old_val` is deprecated"),
            "the second build serves `Main::main` from the type-check cache and lost the \
             deprecation warning with it.\nstderr: {}",
            warm
        );
        assert!(
            warm.contains("in \"main.fix\""),
            "the second build attributed the deprecated use to another file.\nstderr: {}",
            warm
        );
    }

    /// Two files of equal content are two files still, and each build owes the user a warning
    /// anchored in the file it was given. The cache belongs to the working directory, so building
    /// both from one place has them share it, and the entry carries the span of the use. An entry
    /// that a file of equal content may claim answers the second build with a span pointing into
    /// the first build's file, and the warning is then dropped as belonging to a file this build
    /// never read.
    #[test]
    fn two_files_of_equal_content_are_each_reported_in_their_own_file() {
        let temp = two_files_of_equal_content();
        let dir = temp.path();

        for sub_dir in ["a", "b"] {
            let file = format!("{}/main.fix", sub_dir);
            let stderr = build(dir, &file, &[]);
            assert!(
                stderr.contains("`Main::old_val` is deprecated"),
                "building \"{}\" must report the deprecated use.\nstderr: {}",
                file,
                stderr
            );
            assert!(
                stderr.contains(&format!("in \"{}\"", file)),
                "building \"{}\" attributed the deprecated use to another file.\nstderr: {}",
                file,
                stderr
            );
        }
    }

    /// A build with debug information turns the span of every expression into a line and a column,
    /// which it reads out of the file the span points into. An entry a file of equal content may
    /// claim hands the second build spans that point into the first build's file, and that file is
    /// free to have been edited since: the offsets fall past its end, and the compiler aborts with
    /// `called 'Option::unwrap()' on a 'None' value`, naming no file and reporting no diagnostic.
    ///
    /// The first build carries no debug information, so the second one shares none of its object
    /// files and has to generate code — which is where the spans are read.
    #[test]
    fn a_build_with_debug_information_reads_the_spans_out_of_its_own_file() {
        let temp = two_files_of_equal_content();
        let dir = temp.path();

        build(dir, "a/main.fix", &[]);
        fs::write(dir.join("a").join("main.fix"), SHORTER_SOURCE)
            .expect("Failed to rewrite a/main.fix");
        build(dir, "b/main.fix", &["-g"]);

        let run = Command::new(dir.join("out"))
            .output()
            .expect("failed to run the built program");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout).trim(),
            "3",
            "the program built from \"b/main.fix\" computes what its own source says"
        );
    }
}
