//! A build that finds the caches an earlier build of the project left — the type-checked bodies
//! and the object files — must produce what a build from nothing produces. Each test here walks a
//! project through a sequence of versions, one edit apart, building every version on the caches of
//! the version before it. Each build is then repeated at the same path with the caches set aside,
//! and the two must agree on whether the build succeeds, on what it reports, and on every byte of
//! the program it writes.
//!
//! The edits are chosen by what a body's type check reads: the body itself and where it lies in
//! its file, the declarations of its own module, the declarations of each module it imports
//! directly or through another module, the names those imports make visible, the files the project
//! consists of, and the settings of the build.

#[cfg(test)]
mod integration_tests {
    use crate::constants::DOT_FIXLANG;
    use crate::tests::test_util::fix_command;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    /// What a build of one version of a project does.
    enum Outcome {
        /// The build succeeds without a report, and the program prints `stdout`.
        Prints(&'static str),
        /// The build succeeds with a report containing `report`, and the program prints `stdout`.
        WarnsAndPrints {
            report: &'static str,
            stdout: &'static str,
        },
        /// The build succeeds, and the program stops with a message containing `message`.
        Aborts(&'static str),
        /// The build fails with a report containing `report`.
        Rejects(&'static str),
    }

    /// One version of a project.
    struct Version {
        /// Every file of the version, by its path relative to the project directory, with its
        /// content. A file an earlier version has and this one lacks is removed.
        files: Vec<(&'static str, String)>,
        /// The options passed to `fix build`.
        build_args: Vec<&'static str>,
        /// Whether `fix check` runs on the version before it is built, so that the build finds the
        /// caches the check left.
        checked_first: bool,
        outcome: Outcome,
    }

    impl Version {
        /// A version of a project consisting of the Fix sources `sources`, built with debug
        /// information so that where each expression lies reaches the program.
        fn of(sources: &[(&'static str, String)], outcome: Outcome) -> Version {
            let mut files = sources.to_vec();
            files.push(("fixproj.toml", project_file(sources, "")));
            Version {
                files,
                build_args: vec!["-g"],
                checked_first: false,
                outcome,
            }
        }

        /// This version, checked by `fix check` before it is built.
        fn checked_first(self) -> Version {
            Version {
                checked_first: true,
                ..self
            }
        }

        /// This version, built with the options `build_args` in place of its own.
        fn built_with(self, build_args: &[&'static str]) -> Version {
            Version {
                build_args: build_args.to_vec(),
                ..self
            }
        }
    }

    /// The project file of a project whose build compiles the Fix sources `sources` that lie in the
    /// project directory, followed by `rest`.
    fn project_file(sources: &[(&'static str, String)], rest: &str) -> String {
        let files = sources
            .iter()
            .filter(|(path, _)| !path.starts_with("../"))
            .map(|(path, _)| format!("\"{}\"", path))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "[general]\nname = \"invalidation\"\nversion = \"0.1.0\"\n[build]\nfiles = [{}]\n{}",
            files, rest
        )
    }

    /// What one `fix build` produced.
    #[derive(PartialEq)]
    struct Build {
        succeeded: bool,
        stderr: String,
        /// The bytes of the program written, when the build succeeded.
        program: Option<Vec<u8>>,
    }

    /// The name of the program a build writes in the project directory.
    const PROGRAM: &str = "out";

    /// Runs `fix build` with `build_args` in `dir`, on whatever caches `dir` holds.
    fn build(dir: &Path, build_args: &[&str]) -> Build {
        let program = dir.join(PROGRAM);
        if program.exists() {
            fs::remove_file(&program).expect("Failed to remove the program of the last build");
        }
        let output = fix_command()
            .arg("build")
            .args(["-o", PROGRAM])
            .args(build_args)
            .current_dir(dir)
            .output()
            .expect("Failed to run fix build");
        let succeeded = output.status.success();
        Build {
            succeeded,
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            program: succeeded
                .then(|| fs::read(&program).expect("Failed to read the program the build wrote")),
        }
    }

    /// Runs `fix build` with `build_args` in `dir` as a build from nothing would, and leaves the
    /// caches of `dir` as it found them.
    fn build_without_caches(dir: &Path, build_args: &[&str]) -> Build {
        let caches = dir.join(DOT_FIXLANG);
        let set_aside = dir.join(".fixlang-set-aside");
        fs::rename(&caches, &set_aside).expect("Failed to set the caches aside");
        let build = build(dir, build_args);
        fs::remove_dir_all(&caches).expect("Failed to remove the caches of the build from nothing");
        fs::rename(&set_aside, &caches).expect("Failed to put the caches back");
        build
    }

    /// Asserts that the build `build` of the version `index` of a project in `dir` did what
    /// `outcome` says, running the program it wrote there.
    fn assert_outcome(dir: &Path, index: usize, build: &Build, outcome: &Outcome) {
        let report_of = |report: &str| {
            assert!(
                build.stderr.contains(report),
                "version {} should be reported with `{}`, and the build reported:\n{}",
                index,
                report,
                build.stderr
            )
        };
        let (report, rejects) = match outcome {
            Outcome::Prints(_) | Outcome::Aborts(_) => (None, false),
            Outcome::WarnsAndPrints { report, .. } => (Some(*report), false),
            Outcome::Rejects(report) => (Some(*report), true),
        };
        assert_eq!(
            build.succeeded,
            !rejects,
            "version {} should {}, and the build reported:\n{}",
            index,
            if rejects { "be rejected" } else { "build" },
            build.stderr
        );
        match report {
            Some(report) => report_of(report),
            None => assert_eq!(
                build.stderr, "",
                "version {} should build without a report",
                index
            ),
        }
        if rejects {
            return;
        }
        let run = Command::new(dir.join(PROGRAM))
            .current_dir(dir)
            .output()
            .expect("Failed to run the program");
        let stdout = String::from_utf8_lossy(&run.stdout);
        let stderr = String::from_utf8_lossy(&run.stderr);
        match outcome {
            Outcome::Prints(expected)
            | Outcome::WarnsAndPrints {
                stdout: expected, ..
            } => {
                assert!(
                    run.status.success(),
                    "the program of version {} stopped:\n{}",
                    index,
                    stderr
                );
                assert_eq!(
                    stdout.trim_end(),
                    *expected,
                    "the program of version {} printed something else",
                    index
                );
            }
            Outcome::Aborts(message) => {
                assert!(
                    !run.status.success() && stderr.contains(message),
                    "the program of version {} should stop with `{}`, and it printed:\n{}\n{}",
                    index,
                    message,
                    stdout,
                    stderr
                );
            }
            Outcome::Rejects(_) => unreachable!(),
        }
    }

    /// Builds each of `versions` in turn in one project directory, each on the caches the build of
    /// the version before it left, and asserts that each build agrees with a build of the same
    /// version from nothing, and does what the version says.
    ///
    /// Each version is also asserted to build something other than the version before it builds —
    /// another report or another program — since otherwise a build serving the version before it
    /// would agree all the same.
    fn assert_every_version_builds_as_from_nothing(versions: Vec<Version>) {
        let root = TempDir::new().expect("Failed to create temp directory");
        let dir = root.path().join("proj");
        fs::create_dir_all(&dir).expect("Failed to create the project directory");
        let mut written = BTreeSet::new();
        let mut previous: Option<Build> = None;
        for (index, version) in versions.iter().enumerate() {
            let paths = version
                .files
                .iter()
                .map(|(path, _)| *path)
                .collect::<BTreeSet<_>>();
            for stale in written.difference(&paths) {
                fs::remove_file(dir.join(stale)).expect("Failed to remove a file of the project");
            }
            for (path, content) in &version.files {
                let path = dir.join(path);
                fs::create_dir_all(path.parent().unwrap())
                    .expect("Failed to create a directory of the project");
                fs::write(&path, content).expect("Failed to write a file of the project");
            }
            written = paths;

            if version.checked_first {
                fix_command()
                    .arg("check")
                    .current_dir(&dir)
                    .output()
                    .expect("Failed to run fix check");
            }
            let warm = build(&dir, &version.build_args);
            if let Some(previous) = &previous {
                let cold = build_without_caches(&dir, &version.build_args);
                assert_eq!(
                    (warm.succeeded, &warm.stderr),
                    (cold.succeeded, &cold.stderr),
                    "version {} built on the caches of version {} reports otherwise than a build \
                     from nothing (left: on the caches, right: from nothing)",
                    index,
                    index - 1
                );
                assert!(
                    warm.program == cold.program,
                    "version {} built on the caches of version {} writes another program than a \
                     build from nothing",
                    index,
                    index - 1
                );
                assert!(
                    *previous != cold,
                    "version {} builds what version {} builds, so the test cannot tell whether a \
                     build follows the edit",
                    index,
                    index - 1
                );
            }
            assert_outcome(&dir, index, &warm, &version.outcome);
            previous = Some(warm);
        }
    }

    /// A project of the single module `Main`, written as `main`.
    fn main_only(main: String) -> Vec<(&'static str, String)> {
        vec![("main.fix", main)]
    }

    /// A project of `Main`, written as `main`, which imports `Lib`, written as `lib`.
    fn main_and_lib(main: &str, lib: String) -> Vec<(&'static str, String)> {
        vec![("main.fix", main.to_string()), ("lib.fix", lib)]
    }

    // The body and the declarations of the module it is defined in.

    /// Editing the body of a value, and then putting the earlier body back, which a build has
    /// written the file of the module for already.
    #[test]
    fn the_body_of_a_value_edited_and_restored() {
        let source = |answer: &str| {
            format!(
                "module Main;\n\nanswer : I64 = {};\n\nmain : IO () = println(answer.to_string);\n",
                answer
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("1")), Outcome::Prints("1")),
            Version::of(&main_only(source("2")), Outcome::Prints("2")),
            Version::of(&main_only(source("1")), Outcome::Prints("1")),
        ]);
    }

    /// Editing the type a value is declared with, leaving its body and the body using it as they
    /// are. The two bodies are written alike for both types, and compute in the type declared.
    #[test]
    fn the_type_of_a_value() {
        let source = |ty: &str| {
            format!(
                r#"module Main;

parsed : {};
parsed = "255".from_string.as_ok;

main : IO () = println((parsed + "1".from_string.as_ok).to_string);
"#,
                ty
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("I64")), Outcome::Prints("256")),
            Version::of(&main_only(source("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Inserting lines above a body, which moves it in its file and leaves its text as it is. The
    /// warning on the body's use of a deprecated value points at the line the use is on, and the
    /// debug information of the program at where each expression now lies.
    #[test]
    fn lines_inserted_above_a_body() {
        let source = |above: &str| {
            format!(
                r#"module Main;
{}
old : I64 = 1;
DEPRECATED[old, "use `new` instead"];

main : IO () = println(old.to_string);
"#,
                above
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(
                &main_only(source("")),
                Outcome::WarnsAndPrints {
                    report: "6:24-6:27 in \"main.fix\"",
                    stdout: "1",
                },
            ),
            Version::of(
                &main_only(source("\n\n")),
                Outcome::WarnsAndPrints {
                    report: "8:24-8:27 in \"main.fix\"",
                    stdout: "1",
                },
            ),
        ]);
    }

    /// Editing the type of a field of a struct, leaving the bodies constructing and reading it as
    /// they are.
    #[test]
    fn the_type_of_a_field_of_a_struct() {
        let source = |ty: &str| {
            format!(
                r#"module Main;

type P = struct {{ x : {} }};

sample : P = P {{ x : "255".from_string.as_ok }};

main : IO () = println((sample.@x + "1".from_string.as_ok).to_string);
"#,
                ty
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("I64")), Outcome::Prints("256")),
            Version::of(&main_only(source("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Editing the type a type alias stands for, leaving the declaration naming the alias as it is.
    #[test]
    fn the_type_a_type_alias_stands_for() {
        let source = |ty: &str| {
            format!(
                r#"module Main;

type Num = {};

parsed : Num = "255".from_string.as_ok;

main : IO () = println((parsed + "1".from_string.as_ok).to_string);
"#,
                ty
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("I64")), Outcome::Prints("256")),
            Version::of(&main_only(source("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Editing the body of a member of a trait implementation.
    #[test]
    fn the_body_of_a_member_of_a_trait_implementation() {
        let source = |prefix: &str| {
            format!(
                r#"module Main;

type P = struct {{ x : I64 }};

impl P : ToString {{
    to_string = |p| "{}" + p.@x.to_string;
}}

main : IO () = println(P {{ x : 1 }}.to_string);
"#,
                prefix
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("P")), Outcome::Prints("P1")),
            Version::of(&main_only(source("Q")), Outcome::Prints("Q1")),
        ]);
    }

    /// Editing the type of a member of a trait, leaving the body of its implementation and the
    /// body calling it as they are.
    #[test]
    fn the_type_of_a_member_of_a_trait() {
        let source = |ty: &str| {
            format!(
                r#"module Main;

trait a : Measured {{
    measure : a -> {};
}}

type P = struct {{ x : I64 }};

impl P : Measured {{
    measure = |_p| "255".from_string.as_ok;
}}

main : IO () = println((P {{ x : 0 }}.measure + "1".from_string.as_ok).to_string);
"#,
                ty
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("I64")), Outcome::Prints("256")),
            Version::of(&main_only(source("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Adding a trait implementation that overlaps the one a body uses, and removing it again.
    #[test]
    fn an_overlapping_trait_implementation_added_and_removed() {
        let source = |more: &str| {
            format!(
                r#"module Main;

type Holder a = struct {{ v : a }};

impl [a : ToString] Holder a : ToString {{
    to_string = |h| h.@v.to_string;
}}
{}
main : IO () = println(Holder {{ v : 1 }}.to_string);
"#,
                more
            )
        };
        let overlapping = "\nimpl Holder I64 : ToString {\n    to_string = |_h| \"I64\";\n}\n";
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("")), Outcome::Prints("1")),
            Version::of(
                &main_only(source(overlapping)),
                Outcome::Rejects("Two trait implementations for `Std::ToString` are overlapping."),
            ),
            Version::of(&main_only(source("")), Outcome::Prints("1")),
        ]);
    }

    /// Adding a value whose name makes the name a body uses ambiguous.
    #[test]
    fn a_value_making_a_name_ambiguous() {
        let source = |more: &str| {
            format!(
                r#"module Main;

namespace A {{
    v : I64 = 1;
}}
{}
main : IO () = println(v.to_string);
"#,
                more
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("")), Outcome::Prints("1")),
            Version::of(
                &main_only(source("\nnamespace B {\n    v : I64 = 2;\n}\n")),
                Outcome::Rejects("Name `v` is ambiguous"),
            ),
        ]);
    }

    /// Removing a value a body uses, and putting it back.
    #[test]
    fn a_used_value_removed_and_restored() {
        let source = |helper: &str| {
            format!(
                "module Main;\n{}\nmain : IO () = println(helper.to_string);\n",
                helper
            )
        };
        let helper = "\nhelper : I64 = 7;\n";
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source(helper)), Outcome::Prints("7")),
            Version::of(
                &main_only(source("")),
                Outcome::Rejects("Unknown name `helper`."),
            ),
            Version::of(&main_only(source(helper)), Outcome::Prints("7")),
        ]);
    }

    /// Editing the body of a polymorphic function, which instantiation reads at each type the
    /// program uses it at.
    #[test]
    fn the_body_of_a_polymorphic_function() {
        let source = |body: &str| {
            format!(
                r#"module Main;

scale : [a : Add] a -> a;
scale = |x| {};

main : IO () = println((scale(1), scale(10_U8)).to_string);
"#,
                body
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_only(source("x + x")), Outcome::Prints("(2, 20)")),
            Version::of(&main_only(source("x + x + x")), Outcome::Prints("(3, 30)")),
        ]);
    }

    // The declarations of a module the body's module imports.

    /// Editing the body of a value of an imported module.
    #[test]
    fn the_body_of_an_imported_value() {
        let main = "module Main;\n\nimport Lib;\n\nmain : IO () = println(answer.to_string);\n";
        let lib = |answer: &str| format!("module Lib;\n\nanswer : I64 = {};\n", answer);
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("1")), Outcome::Prints("1")),
            Version::of(&main_and_lib(main, lib("2")), Outcome::Prints("2")),
        ]);
    }

    /// Editing the type of a value of an imported module, leaving its body and the body using it
    /// as they are.
    #[test]
    fn the_type_of_an_imported_value() {
        let main = r#"module Main;

import Lib;

main : IO () = println((parsed + "1".from_string.as_ok).to_string);
"#;
        let lib = |ty: &str| {
            format!(
                "module Lib;\n\nparsed : {};\nparsed = \"255\".from_string.as_ok;\n",
                ty
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("I64")), Outcome::Prints("256")),
            Version::of(&main_and_lib(main, lib("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Editing the body of a polymorphic function of an imported module, which instantiation reads
    /// at the types the importing module uses it at.
    #[test]
    fn the_body_of_an_imported_polymorphic_function() {
        let main = r#"module Main;

import Lib;

main : IO () = println((scale(1), scale(10_U8)).to_string);
"#;
        let lib = |body: &str| {
            format!(
                "module Lib;\n\nscale : [a : Add] a -> a;\nscale = |x| {};\n",
                body
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(
                &main_and_lib(main, lib("x + x")),
                Outcome::Prints("(2, 20)"),
            ),
            Version::of(
                &main_and_lib(main, lib("x + x + x")),
                Outcome::Prints("(3, 30)"),
            ),
        ]);
    }

    /// Using an unedited polymorphic function of an imported module at another type, which the
    /// file of the imported module answers for whatever types it is used at.
    #[test]
    fn an_imported_polymorphic_function_used_at_another_type() {
        let main = |argument: &str| {
            format!(
                "module Main;\n\nimport Lib;\n\nmain : IO () = println(scale({}).to_string);\n",
                argument
            )
        };
        let lib = "module Lib;\n\nscale : [a : Add] a -> a;\nscale = |x| x + x;\n".to_string();
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(&main("1"), lib.clone()), Outcome::Prints("2")),
            Version::of(&main_and_lib(&main("200_U8"), lib), Outcome::Prints("144")),
        ]);
    }

    /// Editing the type of a field of a struct of an imported module, leaving the bodies
    /// constructing and reading it as they are.
    #[test]
    fn the_type_of_a_field_of_an_imported_struct() {
        let main = r#"module Main;

import Lib;

sample : P = P { x : "255".from_string.as_ok };

main : IO () = println((sample.@x + "1".from_string.as_ok).to_string);
"#;
        let lib = |ty: &str| format!("module Lib;\n\ntype P = struct {{ x : {} }};\n", ty);
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("I64")), Outcome::Prints("256")),
            Version::of(&main_and_lib(main, lib("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Editing the type an imported type alias stands for.
    #[test]
    fn the_type_an_imported_type_alias_stands_for() {
        let main = r#"module Main;

import Lib;

parsed : Num = "255".from_string.as_ok;

main : IO () = println((parsed + "1".from_string.as_ok).to_string);
"#;
        let lib = |ty: &str| format!("module Lib;\n\ntype Num = {};\n", ty);
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("I64")), Outcome::Prints("256")),
            Version::of(&main_and_lib(main, lib("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Editing the body of a member of a trait implementation in an imported module.
    #[test]
    fn the_body_of_a_member_of_an_imported_trait_implementation() {
        let main =
            "module Main;\n\nimport Lib;\n\nmain : IO () = println(P { x : 1 }.to_string);\n";
        let lib = |prefix: &str| {
            format!(
                r#"module Lib;

type P = struct {{ x : I64 }};

impl P : ToString {{
    to_string = |p| "{}" + p.@x.to_string;
}}
"#,
                prefix
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("P")), Outcome::Prints("P1")),
            Version::of(&main_and_lib(main, lib("Q")), Outcome::Prints("Q1")),
        ]);
    }

    /// Editing the type of a member of an imported trait, which the importing module implements.
    #[test]
    fn the_type_of_a_member_of_an_imported_trait() {
        let main = r#"module Main;

import Lib;

type P = struct { x : I64 };

impl P : Measured {
    measure = |_p| "255".from_string.as_ok;
}

main : IO () = println((P { x : 0 }.measure + "1".from_string.as_ok).to_string);
"#;
        let lib = |ty: &str| {
            format!(
                "module Lib;\n\ntrait a : Measured {{\n    measure : a -> {};\n}}\n",
                ty
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("I64")), Outcome::Prints("256")),
            Version::of(&main_and_lib(main, lib("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Editing the type an associated type of an imported implementation stands for.
    #[test]
    fn the_type_an_imported_associated_type_stands_for() {
        let main = r#"module Main;

import Lib;

main : IO () = println((Bag { s : "255" }.first + "1".from_string.as_ok).to_string);
"#;
        let lib = |ty: &str| {
            format!(
                r#"module Lib;

trait c : Container {{
    type Content c;
    first : c -> Content c;
}}

type Bag = struct {{ s : String }};

impl Bag : Container {{
    type Content Bag = {};
    first = |b| b.@s.from_string.as_ok;
}}
"#,
                ty
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("I64")), Outcome::Prints("256")),
            Version::of(&main_and_lib(main, lib("U8")), Outcome::Prints("0")),
        ]);
    }

    /// Editing the body of an imported function whose result type is opaque, which changes the
    /// concrete type the opaque type stands for.
    #[test]
    fn the_concrete_type_of_an_imported_opaque_type() {
        let main =
            "module Main;\n\nimport Lib;\n\nmain : IO () = println(nums(4).sum.to_string);\n";
        let lib = |body: &str| {
            format!(
                r#"module Lib;

nums : [?it : Iterator, Item ?it = I64] I64 -> ?it;
nums = |n| {};
"#,
                body
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(
                &main_and_lib(main, lib("Iterator::range(0, n)")),
                Outcome::Prints("6"),
            ),
            Version::of(
                &main_and_lib(main, lib("Iterator::range(0, n).map(|x| x * 10)")),
                Outcome::Prints("60"),
            ),
        ]);
    }

    /// Editing the traits an imported trait alias stands for, so that a type the importing module
    /// uses no longer satisfies it.
    #[test]
    fn the_traits_an_imported_trait_alias_stands_for() {
        let main = r#"module Main;

import Lib;

show : [a : Shown] a -> String = |x| x.to_string;

type P = struct { x : I64 };

impl P : ToString {
    to_string = |p| p.@x.to_string;
}

main : IO () = println(show(P { x : 1 }));
"#;
        let lib = |traits: &str| format!("module Lib;\n\ntrait Shown = {};\n", traits);
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("ToString")), Outcome::Prints("1")),
            Version::of(
                &main_and_lib(main, lib("ToString + Zero")),
                Outcome::Rejects("`Main::P : Std::Zero` is required"),
            ),
        ]);
    }

    /// Adding a value to an imported module whose name makes the name a body uses ambiguous.
    #[test]
    fn an_imported_value_making_a_name_ambiguous() {
        let main =
            "module Main;\n\nimport Lib;\nimport Other;\n\nmain : IO () = println(v.to_string);\n";
        let files = |lib: &str| {
            vec![
                ("main.fix", main.to_string()),
                ("lib.fix", format!("module Lib;\n\nw : I64 = 0;\n{}", lib)),
                ("other.fix", "module Other;\n\nv : I64 = 2;\n".to_string()),
            ]
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&files(""), Outcome::Prints("2")),
            Version::of(
                &files("\nv : I64 = 1;\n"),
                Outcome::Rejects("Name `v` is ambiguous"),
            ),
        ]);
    }

    /// Removing a value of an imported module a body uses.
    #[test]
    fn an_imported_value_removed() {
        let main = "module Main;\n\nimport Lib;\n\nmain : IO () = println(answer.to_string);\n";
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(
                &main_and_lib(main, "module Lib;\n\nanswer : I64 = 1;\n".to_string()),
                Outcome::Prints("1"),
            ),
            Version::of(
                &main_and_lib(main, "module Lib;\n\nother : I64 = 1;\n".to_string()),
                Outcome::Rejects("Unknown name `answer`."),
            ),
        ]);
    }

    /// Deprecating a value of an imported module a body uses.
    #[test]
    fn an_imported_value_deprecated() {
        let main = "module Main;\n\nimport Lib;\n\nmain : IO () = println(answer.to_string);\n";
        let lib = |pragma: &str| format!("module Lib;\n\nanswer : I64 = 1;\n{}", pragma);
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&main_and_lib(main, lib("")), Outcome::Prints("1")),
            Version::of(
                &main_and_lib(main, lib("DEPRECATED[answer, \"gone soon\"];\n")),
                Outcome::WarnsAndPrints {
                    report: "`Lib::answer` is deprecated: gone soon",
                    stdout: "1",
                },
            ),
        ]);
    }

    /// Moving which of two imported modules hides a name both define, which decides the value the
    /// name refers to.
    #[test]
    fn the_name_an_import_hides() {
        let main = |imports: &str| {
            format!(
                "module Main;\n\n{}\n\nmain : IO () = println(v.to_string);\n",
                imports
            )
        };
        let files = |imports: &str| {
            vec![
                ("main.fix", main(imports)),
                ("lib.fix", "module Lib;\n\nv : I64 = 1;\n".to_string()),
                ("other.fix", "module Other;\n\nv : I64 = 2;\n".to_string()),
            ]
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(
                &files("import Lib;\nimport Other hiding v;"),
                Outcome::Prints("1"),
            ),
            Version::of(
                &files("import Lib hiding v;\nimport Other;"),
                Outcome::Prints("2"),
            ),
        ]);
    }

    /// Editing a module imported through another module: the importing module and the module in
    /// between are unedited, and the importing module's body reads a type alias of the module at
    /// the end.
    #[test]
    fn a_module_imported_through_another_module() {
        let main = r#"module Main;

import Middle;

main : IO () = println((parsed + "1".from_string.as_ok).to_string);
"#;
        let middle =
            "module Middle;\n\nimport Base;\n\nparsed : Num = \"255\".from_string.as_ok;\n";
        let files = |ty: &str| {
            vec![
                ("main.fix", main.to_string()),
                ("middle.fix", middle.to_string()),
                ("base.fix", format!("module Base;\n\ntype Num = {};\n", ty)),
            ]
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&files("I64"), Outcome::Prints("256")),
            Version::of(&files("U8"), Outcome::Prints("0")),
        ]);
    }

    /// Checking each version with `fix check` before building it, which leaves the caches the
    /// build then reads: the type-checked bodies of the version, and none of its object files.
    #[test]
    fn a_check_before_each_build() {
        let main = "module Main;\n\nimport Lib;\n\nmain : IO () = println((scale(parsed), scale(1)).to_string);\n";
        let lib = |ty: &str, body: &str| {
            format!(
                "module Lib;\n\nparsed : {};\nparsed = \"200\".from_string.as_ok;\n\nscale : [a : Add] a -> a;\nscale = |x| {};\n",
                ty, body
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(
                &main_and_lib(main, lib("I64", "x + x")),
                Outcome::Prints("(400, 2)"),
            )
            .checked_first(),
            Version::of(
                &main_and_lib(main, lib("U8", "x + x")),
                Outcome::Prints("(144, 2)"),
            )
            .checked_first(),
            Version::of(
                &main_and_lib(main, lib("U8", "x + x + x")),
                Outcome::Prints("(88, 3)"),
            )
            .checked_first(),
        ]);
    }

    // The files the project consists of.

    /// Renaming the file of an imported module, leaving its content as it is. The debug
    /// information of the program names the file each expression lies in.
    #[test]
    fn the_file_of_a_module_renamed() {
        let main = "module Main;\n\nimport Lib;\n\nmain : IO () = println(answer.to_string);\n";
        let lib = "module Lib;\n\nanswer : I64 = 1;\n";
        let files = |lib_path: &'static str| {
            vec![("main.fix", main.to_string()), (lib_path, lib.to_string())]
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&files("lib.fix"), Outcome::Prints("1")),
            Version::of(&files("library.fix"), Outcome::Prints("1")),
        ]);
    }

    /// Renaming an imported module, and the import naming it.
    #[test]
    fn a_module_renamed() {
        let files = |module: &str| {
            main_and_lib(
                &format!(
                    "module Main;\n\nimport {};\n\nmain : IO () = println(answer.to_string);\n",
                    module
                ),
                format!("module {};\n\nanswer : I64 = 1;\n", module),
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&files("Lib"), Outcome::Prints("1")),
            Version::of(&files("Library"), Outcome::Prints("1")),
        ]);
    }

    /// Moving a value from an imported module into the module using it.
    #[test]
    fn a_value_moved_into_the_module_using_it() {
        let main = |own: &str| {
            format!(
                "module Main;\n\nimport Lib;\n{}\nmain : IO () = println(answer.to_string);\n",
                own
            )
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(
                &main_and_lib(&main(""), "module Lib;\n\nanswer : I64 = 1;\n".to_string()),
                Outcome::Prints("1"),
            ),
            Version::of(
                &main_and_lib(
                    &main("\nanswer : I64 = 2;\n"),
                    "module Lib;\n\nother : I64 = 1;\n".to_string(),
                ),
                Outcome::Prints("2"),
            ),
        ]);
    }

    /// Adding a module to the project and importing it, and then going back to the project
    /// without it.
    #[test]
    fn a_module_added_and_removed() {
        let alone = main_only("module Main;\n\nmain : IO () = println(\"alone\");\n".to_string());
        let with_lib = main_and_lib(
            "module Main;\n\nimport Lib;\n\nmain : IO () = println(answer.to_string);\n",
            "module Lib;\n\nanswer : I64 = 1;\n".to_string(),
        );
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&alone, Outcome::Prints("alone")),
            Version::of(&with_lib, Outcome::Prints("1")),
            Version::of(&alone, Outcome::Prints("alone")),
        ]);
    }

    /// Editing a module of a dependency the project names by its path.
    #[test]
    fn a_module_of_a_path_dependency() {
        let main = "module Main;\n\nimport Dep;\n\nmain : IO () = println((parsed + \"1\".from_string.as_ok).to_string);\n";
        let files = |ty: &str, outcome: Outcome| {
            let sources = vec![("main.fix", main.to_string())];
            let mut files = sources.clone();
            files.push((
                "fixproj.toml",
                project_file(
                    &sources,
                    "\n[[dependencies]]\nname = \"dep\"\nversion = \"*\"\npath = \"../dep\"\n",
                ),
            ));
            files.push((
                "../dep/fixproj.toml",
                "[general]\nname = \"dep\"\nversion = \"0.1.0\"\n[build]\nfiles = [\"lib.fix\"]\n"
                    .to_string(),
            ));
            files.push((
                "../dep/lib.fix",
                format!(
                    "module Dep;\n\nparsed : {};\nparsed = \"255\".from_string.as_ok;\n",
                    ty
                ),
            ));
            Version {
                files,
                build_args: vec!["-g"],
                checked_first: false,
                outcome,
            }
        };
        assert_every_version_builds_as_from_nothing(vec![
            // The first build resolves the dependencies and writes the lock file.
            files(
                "I64",
                Outcome::WarnsAndPrints {
                    report: "Dependencies resolved successfully.",
                    stdout: "256",
                },
            ),
            files("U8", Outcome::Prints("0")),
        ]);
    }

    // The settings of the build.

    /// Turning the evaluation of `eval` on and off.
    #[test]
    fn skipping_eval_turned_on_and_off() {
        let source = main_only(
            "module Main;\n\nmain : IO () = (\n    eval debug_println(\"evaluated\");\n    println(\"done\")\n);\n"
                .to_string(),
        );
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&source, Outcome::Prints("evaluated\ndone")),
            Version::of(&source, Outcome::Prints("done")).built_with(&["-g", "--skip-eval"]),
            Version::of(&source, Outcome::Prints("evaluated\ndone")),
        ]);
    }

    /// Turning the checks of integer operations on and off.
    #[test]
    fn checking_integer_operations_turned_on_and_off() {
        let source = main_only(
            "module Main;\n\nmain : IO () = println((I64::maximum + \"1\".from_string.as_ok).to_string);\n"
                .to_string(),
        );
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&source, Outcome::Prints("-9223372036854775808")),
            Version::of(&source, Outcome::Aborts("Signed integer overflow"))
                .built_with(&["-g", "--check-integer-operations"]),
            Version::of(&source, Outcome::Prints("-9223372036854775808")),
        ]);
    }

    /// Turning debug information on and off.
    #[test]
    fn debug_information_turned_on_and_off() {
        let source = main_only("module Main;\n\nmain : IO () = println(\"hi\");\n".to_string());
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&source, Outcome::Prints("hi")).built_with(&[]),
            Version::of(&source, Outcome::Prints("hi")),
            Version::of(&source, Outcome::Prints("hi")).built_with(&[]),
        ]);
    }

    /// Turning multi-threading on and off.
    #[test]
    fn threading_turned_on_and_off() {
        let source = main_only("module Main;\n\nmain : IO () = println(\"hi\");\n".to_string());
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&source, Outcome::Prints("hi")),
            Version::of(&source, Outcome::Prints("hi")).built_with(&["-g", "--threaded"]),
            Version::of(&source, Outcome::Prints("hi")),
        ]);
    }

    /// Turning the deprecation warnings off and on.
    #[test]
    fn deprecation_warnings_turned_off_and_on() {
        let source = main_only(
            "module Main;\n\nold : I64 = 1;\nDEPRECATED[old, \"use `new` instead\"];\n\nmain : IO () = println(old.to_string);\n"
                .to_string(),
        );
        let warned = || Outcome::WarnsAndPrints {
            report: "`Main::old` is deprecated",
            stdout: "1",
        };
        assert_every_version_builds_as_from_nothing(vec![
            Version::of(&source, warned()),
            Version::of(&source, Outcome::Prints("1")).built_with(&["-g", "--allow-deprecated"]),
            Version::of(&source, warned()),
        ]);
    }
}
