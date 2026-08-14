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

    /// Builds `main.fix` in `dir` and returns whether the build succeeded together with what the
    /// compiler wrote to stderr.
    fn try_build(dir: &Path) -> (bool, String) {
        let out = fix_command()
            .args(["build", "--file", "main.fix", "-o", "out"])
            .current_dir(dir)
            .output()
            .expect("failed to run fix build");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )
    }

    /// Builds `main.fix` in `dir`, requiring the build to succeed, and returns what the compiler
    /// wrote to stderr.
    fn build(dir: &Path) -> String {
        let (succeeded, stderr) = try_build(dir);
        assert!(
            succeeded,
            "fix build failed in {}:\n{}",
            dir.display(),
            stderr
        );
        stderr
    }

    /// One trait method implemented for two types, by a member whose declared type fixes the
    /// trait's type variable through a constraint alone. Each implementation is an entry of its
    /// own, told apart by the type it is for; sharing one entry would let the second build serve
    /// the sound implementation's typed expression for the broken one and report nothing.
    const BROKEN_IMPLEMENTATION: &str = r#"module Main;

trait c : Make {
    make : [?it : Iterator, Item ?it = c] I64 -> ?it;
}

impl I64 : Make {
    make = |n| Iterator::range(0, n);
}

impl Bool : Make {
    make = |n| n;
}

main : IO ();
main = (
    let is : Array I64 = Make::make(3).to_array;
    println(is.to_string)
);
"#;

    /// Every build of a program the type checker rejects has to reject it, whether or not a cache
    /// of an earlier build is there to read.
    #[test]
    fn a_warm_cache_rejects_what_a_cold_cache_rejected() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(dir.join("main.fix"), BROKEN_IMPLEMENTATION).expect("Failed to write main.fix");

        let (cold_succeeded, cold) = try_build(dir);
        assert!(
            !cold_succeeded && cold.contains("`Std::I64 : Std::Iterator` cannot be deduced"),
            "the first build must reject the implementation whose body is not an iterator.\nstderr: {}",
            cold
        );

        let (warm_succeeded, warm) = try_build(dir);
        assert!(
            !warm_succeeded && warm.contains("`Std::I64 : Std::Iterator` cannot be deduced"),
            "the second build read the cache and accepted the program the first build rejected.\nstderr: {}",
            warm
        );
    }

    /// Two implementations of one trait method whose declared type fixes the trait's type variable
    /// through a constraint alone, reached both directly and through a caller generic in that
    /// variable. Each implementation hides an iterator of its own.
    const TWO_IMPLEMENTATIONS: &str = r#"module Main;

trait c : Make {
    make : [?it : Iterator, Item ?it = c] I64 -> ?it;
}

impl I64 : Make {
    make = |n| Iterator::range(0, n);
}

impl Bool : Make {
    make = |n| Iterator::range(0, n).map(|x| x % 2 == 0);
}

collect : [c : Make] I64 -> Array c;
collect = |n| Make::make(n).to_array;

main : IO ();
main = (
    let is : Array I64 = Make::make(3).to_array;
    let bs : Array Bool = Make::make(3).to_array;
    let ci : Array I64 = collect(2);
    let cb : Array Bool = collect(2);
    println(is.to_string + " / " + bs.to_string + " / " + ci.to_string + " / " + cb.to_string)
);
"#;

    /// A run that reads the cache serves each implementation the typed expression of that
    /// implementation, so the program prints what the run that filled the cache printed.
    #[test]
    fn a_warm_cache_serves_each_implementation_its_own_body() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(dir.join("main.fix"), TWO_IMPLEMENTATIONS).expect("Failed to write main.fix");

        let run = || {
            let out = fix_command()
                .args(["run", "--file", "main.fix"])
                .current_dir(dir)
                .output()
                .expect("failed to run fix run");
            assert!(
                out.status.success(),
                "fix run failed in {}:\n{}",
                dir.display(),
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };

        let expected = "[0, 1, 2] / [true, false, true] / [0, 1] / [true, false]";
        assert_eq!(run(), expected, "the run that fills the cache");
        assert_eq!(run(), expected, "the run that reads the cache");
    }

    /// A deprecation warning is collected after type checking, out of the typed expression and the
    /// span it carries — which on the second build comes from the cache. Both builds owe the user
    /// the same warning, anchored in the file the use is written in.
    #[test]
    fn a_warm_cache_keeps_the_deprecation_warning() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(dir.join("main.fix"), DEPRECATED_USE).expect("Failed to write main.fix");

        let cold = build(dir);
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

        let warm = build(dir);
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
}
