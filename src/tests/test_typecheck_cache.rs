//! A run that hits the type-check cache must produce what a run that missed it produces. A hit
//! skips name resolution and type checking altogether, so whatever the entry gets wrong is served
//! without a word: a body the value never declared, a stale answer after an edit, or a report the
//! build owed the user.

use crate::{
    configuration::Configuration, constants::STD_NAME, fixstd::stdlib::make_std_mod,
    tests::test_util::test_source,
};

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

/// The C numeric types decide the Fix type the parser gives a `CInt` in an `FFI_CALL` signature,
/// and the implementations converting to a C type that the compiler builds. Neither is written in
/// any source, so the hash covers the types themselves: the width of each, and the sign of plain
/// `char`.
#[test]
fn test_the_module_dependency_hash_covers_the_c_numeric_types() {
    let config = Configuration::develop_mode();
    let std_name = STD_NAME.to_string();
    let program = make_std_mod(&config)
        .unwrap_or_else(|errs| panic!("Failed to build the `Std` module: {}", errs));
    let hash_under = |config: &Configuration| {
        program
            .module_dependency_hash(&std_name, config)
            .unwrap_or_else(|errs| panic!("Failed to hash the `Std` module: {}", errs))
    };

    let mut wider_int = config.clone();
    wider_int.c_numeric_types.int_bits = config.c_numeric_types.int_bits * 2;
    assert_ne!(
        hash_under(&config),
        hash_under(&wider_int),
        "a C `int` of another width gives the program other types, and the hash naming what the \
         program is checked from stayed where it was"
    );

    let mut other_char_sign = config.clone();
    other_char_sign.c_numeric_types.char_is_signed = !config.c_numeric_types.char_is_signed;
    assert_ne!(
        hash_under(&config),
        hash_under(&other_char_sign),
        "a plain `char` of the other sign gives the program other types, and the hash naming what \
         the program is checked from stayed where it was"
    );
}

#[cfg(test)]
mod integration_tests {
    use crate::constants::{DOT_FIXLANG, PARSED_MODULES_PATH, TYPED_MODULES_PATH};
    use crate::tests::test_util::{fix_command, run_fix};
    use std::fs::{self, File, Permissions};
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, SystemTime};
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
            let output = run_fix(temp.path(), &["run", "--file", "main.fix"]);
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
    fn dir_of_two_files_of_equal_content() -> TempDir {
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
        let (built, stderr) = try_build(dir, file, extra_args);
        assert!(built, "fix build failed in {}:\n{}", dir.display(), stderr);
        stderr
    }

    /// A deprecation warning is collected after type checking, out of the typed expression and the
    /// span it carries — which on the second build comes from the cache. Both builds owe the user
    /// the same warning, anchored in the file the use is written in.
    #[test]
    fn a_warm_cache_keeps_the_deprecation_warning() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(dir.join("main.fix"), DEPRECATED_USE).expect("Failed to write main.fix");

        let cold_stderr = build(dir, "main.fix", &[]);
        assert!(
            cold_stderr.contains("`Main::old_val` is deprecated"),
            "the first build must report the deprecated use.\nstderr: {}",
            cold_stderr
        );
        assert!(
            cold_stderr.contains("in \"main.fix\""),
            "the first build must attribute the use to the file it is written in.\nstderr: {}",
            cold_stderr
        );

        let warm_stderr = build(dir, "main.fix", &[]);
        assert!(
            warm_stderr.contains("`Main::old_val` is deprecated"),
            "the second build serves `Main::main` from the type-check cache and lost the \
             deprecation warning with it.\nstderr: {}",
            warm_stderr
        );
        assert!(
            warm_stderr.contains("in \"main.fix\""),
            "the second build attributed the deprecated use to another file.\nstderr: {}",
            warm_stderr
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
        let temp = dir_of_two_files_of_equal_content();
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

    /// Programs written one after another to a single file, each one a way a check can end: a
    /// program that passes, one the type checker rejects, one that leaves a hole, one that uses a
    /// deprecated value, and one that names a value nothing defines. Each uses a tuple of a size of
    /// its own, so each also asks `Std` for implementations the ones before it never asked for.
    const PROGRAMS_CHECKED_IN_TURN: [&str; 5] = [
        r#"module Main;
main : IO ();
main = println((1, 2).to_string);
"#,
        r#"module Main;
main : IO ();
main = println((1, 2, 3, 4).to_string + true);
"#,
        r#"module Main;
main : IO ();
main = println((1, 2, 3, 4, 5).to_string + _.to_string);
"#,
        r#"module Main;
DEPRECATED[old_val, "use `new_val` instead"];
old_val : I64;
old_val = 1;
new_val : I64;
new_val = 2;
main : IO ();
main = println((1, 2, 3, 4, 5, 6).to_string + (old_val + new_val).to_string);
"#,
        r#"module Main;
main : IO ();
main = println((1, 2, 3, 4, 5, 6, 7).to_string + no_such_value);
"#,
    ];

    /// Runs `fix check` in `dir` and returns everything it told the user.
    fn check(dir: &Path) -> String {
        let output = run_fix(dir, &["check"]);
        String::from_utf8_lossy(&output.stdout).to_string()
            + &String::from_utf8_lossy(&output.stderr)
    }

    /// The cache belongs to the working directory and a file is named by its path, so programs
    /// written one after another to a single file file their entries side by side. What the user is
    /// told about one of them must be what they are told when that program is the only one the
    /// directory has ever held.
    #[test]
    fn a_cache_filled_by_other_programs_at_one_path_keeps_every_report() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("fixproj.toml"),
            "[general]\nname = \"checked-in-turn\"\nversion = \"0.1.0\"\n[build]\nfiles = [\"main.fix\"]\n",
        )
        .expect("Failed to write fixproj.toml");

        let mut warm_reports = vec![];
        for program in PROGRAMS_CHECKED_IN_TURN {
            fs::write(dir.join("main.fix"), program).expect("Failed to write main.fix");
            warm_reports.push(check(dir));
        }

        for (program, warm) in PROGRAMS_CHECKED_IN_TURN.iter().zip(warm_reports) {
            fs::write(dir.join("main.fix"), program).expect("Failed to write main.fix");
            fs::remove_dir_all(dir.join(DOT_FIXLANG)).expect("Failed to empty the caches");
            assert_eq!(
                check(dir),
                warm,
                "checking this program told the user something else once the directory held the \
                 entries of the programs checked before it:\n{}",
                program
            );
        }
    }

    /// A cache entry is filed under a hash of everything the value is checked from, so a build that
    /// changed nothing finds every entry the build before it wrote. An input to that hash which
    /// varies from one run to the next — a path carrying a random component, a clock reading, the
    /// order a container is walked in — has each build file entries of its own and check the whole
    /// program again, while every test of what a build produces still passes.
    ///
    /// The program uses a tuple, whose implementations the compiler generates into a source of its
    /// own, and a value whose type the C numeric types decide, so the entries cover what the key reads
    /// beyond the program's own source.
    #[test]
    fn a_second_build_of_an_unedited_program_finds_the_entries_of_the_first() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("main.fix"),
            r#"module Main;

main : IO ();
main = println $ (1, 2, 3, 4).to_string + 65.c_int.i64.to_string;
"#,
        )
        .expect("Failed to write main.fix");
        let cache_entries = || {
            let mut names = fs::read_dir(dir.join(TYPED_MODULES_PATH))
                .expect("Failed to read the type-check cache directory")
                .map(|entry| entry.expect("Failed to read a cache entry").file_name())
                .collect::<Vec<_>>();
            names.sort();
            names
        };

        build(dir, "main.fix", &[]);
        let first_build_entries = cache_entries();
        assert!(
            !first_build_entries.is_empty(),
            "the first build fills the type-check cache"
        );

        build(dir, "main.fix", &[]);
        assert_eq!(
            first_build_entries,
            cache_entries(),
            "the second build filed entries of its own, so the hash naming what a value is checked \
             from does not name the same thing twice"
        );
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
        let temp = dir_of_two_files_of_equal_content();
        let dir = temp.path();

        build(dir, "a/main.fix", &[]);
        fs::write(dir.join("a").join("main.fix"), SHORTER_SOURCE)
            .expect("Failed to rewrite a/main.fix");
        build(dir, "b/main.fix", &["-g"]);

        let program_output = Command::new(dir.join("out"))
            .output()
            .expect("failed to run the built program");
        assert_eq!(
            String::from_utf8_lossy(&program_output.stdout).trim(),
            "3",
            "the program built from \"b/main.fix\" computes what its own source says"
        );
    }

    /// The files of the type-check cache under `dir`, by name, with when each was last written.
    fn cache_files(dir: &Path) -> Vec<(String, SystemTime)> {
        files_under(dir, TYPED_MODULES_PATH)
    }

    /// The name of each file in the directory `cache_path` of the project in `dir`, with when it was
    /// written, in order of the names.
    fn files_under(dir: &Path, cache_path: &str) -> Vec<(String, SystemTime)> {
        let mut files: Vec<_> = fs::read_dir(dir.join(cache_path))
            .expect("Failed to read a cache directory")
            .map(|entry| {
                let entry = entry.expect("Failed to read a cache file");
                let written = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .expect("Failed to read when a cache file was written");
                (entry.file_name().to_string_lossy().to_string(), written)
            })
            .collect();
        files.sort();
        files
    }

    /// Runs `fix build` on the Fix source `file`, named as the working directory `dir` reaches it,
    /// passing `extra_args` as well, and answers with whether it succeeded and what it wrote to
    /// stderr. The program is written to `out` in that directory.
    fn try_build(dir: &Path, file: &str, extra_args: &[&str]) -> (bool, String) {
        let output = run_fix(
            dir,
            &[&["build", "--file", file, "-o", "out"], extra_args].concat(),
        );
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    }

    /// What the program `out` in `dir` prints.
    fn run_out(dir: &Path) -> String {
        let output = Command::new(dir.join("out"))
            .output()
            .expect("failed to run the built program");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    /// The cache records the uses a body makes, and whether a use of a deprecated value is an error
    /// is decided by the settings of each build. A build denying deprecated uses after one that
    /// allowed them reads the body from the cache, and still stops at the use.
    #[test]
    fn a_build_denying_deprecated_uses_after_one_allowing_them_fails() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(dir.join("main.fix"), DEPRECATED_USE).expect("Failed to write main.fix");

        let (allowed_built, allowed_stderr) = try_build(dir, "main.fix", &["--allow-deprecated"]);
        assert!(allowed_built, "stderr: {}", allowed_stderr);
        assert!(
            !allowed_stderr.contains("is deprecated"),
            "a build allowing deprecated uses reported one.\nstderr: {}",
            allowed_stderr
        );

        let (denied_built, denied_stderr) = try_build(dir, "main.fix", &["--deny-deprecated"]);
        assert!(
            !denied_built,
            "a build denying deprecated uses succeeded on a program using one"
        );
        assert!(
            denied_stderr.contains("`Main::old_val` is deprecated"),
            "stderr: {}",
            denied_stderr
        );
    }

    /// `Std` carries the implementations of a tuple for each size the program uses, so two programs
    /// that give `Std` the same hash can need different bodies of it. A build
    /// using a larger tuple adds the bodies it checks to the file, and a build going back to the
    /// smaller tuple finds every body it needs there, and writes nothing.
    #[test]
    fn a_build_using_fewer_tuple_sizes_finds_every_body_in_the_cache() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let write_main = |tuple: &str| {
            fs::write(
                dir.join("main.fix"),
                format!(
                    "module Main;\n\nmain : IO ();\nmain = println $ {}.to_string;\n",
                    tuple
                ),
            )
            .expect("Failed to write main.fix");
        };
        let small = "(1, 2)";
        let large = "(1, 2, 3, 4, 5, 6, 7, 8, 9)";

        write_main(small);
        build(dir, "main.fix", &[]);
        write_main(large);
        build(dir, "main.fix", &[]);
        assert_eq!(run_out(dir), large);
        let after_large = cache_files(dir);

        write_main(large);
        build(dir, "main.fix", &[]);
        assert_eq!(
            cache_files(dir),
            after_large,
            "a build of an unedited program wrote the type-check cache"
        );

        write_main(small);
        build(dir, "main.fix", &[]);
        assert_eq!(run_out(dir), small);
        let std_files = |files: &[(String, SystemTime)]| -> Vec<(String, SystemTime)> {
            files
                .iter()
                .filter(|(name, _)| name.starts_with("Std_"))
                .cloned()
                .collect()
        };
        assert_eq!(
            std_files(&cache_files(dir)),
            std_files(&after_large),
            "the build going back to the smaller tuple checked bodies of `Std` again"
        );
    }

    /// Builds a program in `dir` that prints 3, and returns the path of the cache file of its module
    /// `Main`.
    fn build_three_and_find_the_cache_file_of_main(dir: &Path) -> PathBuf {
        fs::write(
            dir.join("main.fix"),
            "module Main;\n\nmain : IO ();\nmain = println $ (1 + 2).to_string;\n",
        )
        .expect("Failed to write main.fix");
        build(dir, "main.fix", &[]);
        let main_file = cache_files(dir)
            .into_iter()
            .map(|(name, _)| name)
            .find(|name| name.starts_with("Main_"))
            .expect("the build wrote no cache file of `Main`");
        dir.join(TYPED_MODULES_PATH).join(main_file)
    }

    /// A file of the type-check cache whose record, or the identity in front of a body, was damaged
    /// costs one build the check it could have skipped, and nothing else: that build writes what it
    /// checked back to the file, and the build after it checks nothing again and leaves the file as
    /// it is. Damaging the record makes the build check the module again; damaging the identity of
    /// a body the record points to makes instantiation check that body again.
    #[test]
    fn a_build_recovers_from_a_damaged_cache_file() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let main_file = build_three_and_find_the_cache_file_of_main(dir);
        let written = fs::read(&main_file).expect("Failed to read the cache file of `Main`");

        // The body comes first in the file, and its first bytes are the identity written in front
        // of it.
        let mut damaged_body = written.clone();
        for byte in damaged_body.iter_mut().take(8) {
            *byte = 0xff;
        }
        fs::write(&main_file, &damaged_body).expect("Failed to damage the cache file");
        let (built, stderr) = try_build(dir, "main.fix", &[]);
        assert!(built, "stderr: {}", stderr);
        assert!(
            stderr.contains("Failed to read cache file"),
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "3");
        let after_recovery = cache_files(dir);
        let (built, stderr) = try_build(dir, "main.fix", &[]);
        assert!(built, "stderr: {}", stderr);
        assert!(
            !stderr.contains("Failed to read cache file"),
            "the build after the one checking the damaged body again found it damaged still.\n\
             stderr: {}",
            stderr
        );
        assert_eq!(
            cache_files(dir),
            after_recovery,
            "the build after the one checking the damaged body again checked a body again"
        );

        // The footer ends the file.
        let mut damaged_footer = written.clone();
        let len = damaged_footer.len();
        damaged_footer[len - 1] ^= 0xff;
        fs::write(&main_file, &damaged_footer).expect("Failed to damage the cache file");
        let (built, stderr) = try_build(dir, "main.fix", &[]);
        assert!(built, "stderr: {}", stderr);
        assert!(
            stderr.contains("Failed to read cache file"),
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "3");
        assert_eq!(
            fs::read(&main_file).expect("Failed to read the cache file of `Main`"),
            written,
            "the build checking `Main` again wrote another file than the first build"
        );
    }

    /// Editing a module gives it a hash of its own, and a build writing the file of the new hash
    /// removes the file of the old one, so the cache holds one file for each module.
    #[test]
    fn a_build_after_an_edit_removes_the_file_of_the_old_hash() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let main_files = || {
            cache_files(dir)
                .into_iter()
                .filter(|(name, _)| name.starts_with("Main_"))
                .count()
        };
        for answer in ["1", "2"] {
            fs::write(
                dir.join("main.fix"),
                format!(
                    "module Main;\n\nmain : IO ();\nmain = println(\"{}\");\n",
                    answer
                ),
            )
            .expect("Failed to write main.fix");
            build(dir, "main.fix", &[]);
            assert_eq!(run_out(dir), answer);
            assert_eq!(
                main_files(),
                1,
                "the cache holds more than one file of `Main`"
            );
        }
    }

    /// The cache records the concrete type each opaque type of a body stands for. A build that
    /// reads such a body from the cache gets the types from the record, and builds the program the
    /// first build did.
    #[test]
    fn a_build_reading_a_body_with_an_opaque_type_from_the_cache_builds_the_program() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("main.fix"),
            r#"module Main;

evens : [?it : Iterator, Item ?it = I64] I64 -> ?it;
evens = |n| Iterator::range(0, n).filter(|i| i % 2 == 0);

main : IO ();
main = println $ evens(10).fold(0, |i, s| s + i).to_string;
"#,
        )
        .expect("Failed to write main.fix");

        build(dir, "main.fix", &[]);
        assert_eq!(run_out(dir), "20");
        let after_first = cache_files(dir);

        build(dir, "main.fix", &[]);
        assert_eq!(
            cache_files(dir),
            after_first,
            "the second build checked a body again instead of reading it from the cache"
        );
        assert_eq!(run_out(dir), "20");
    }

    /// The warning on a struct pattern that leaves out fields without `_` is derived from what the
    /// cache records of the body, so a build reading the body from the cache reports it as the
    /// build that checked the body did.
    #[test]
    fn a_warm_cache_keeps_the_missing_pattern_field_warning() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("main.fix"),
            r#"module Main;

type S = struct { x : I64, y : I64 };

main : IO ();
main = (
    let S { x } = S { x : 1, y : 2 };
    println(x.to_string)
);
"#,
        )
        .expect("Failed to write main.fix");

        for build_name in ["the build checking the body", "the build reading the cache"] {
            let stderr = build(dir, "main.fix", &[]);
            assert!(
                stderr.contains("This pattern leaves out the"),
                "{} reported no missing field.\nstderr: {}",
                build_name,
                stderr
            );
        }
    }

    /// A build writing the file of one module removes the files of that module's other hashes, and
    /// keeps the files of every other module, whatever their names begin with.
    #[test]
    fn a_build_after_an_edit_keeps_the_files_of_the_other_modules() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("sub.fix"),
            "module Main.Sub;\n\nvalue : I64;\nvalue = 1;\n",
        )
        .expect("Failed to write sub.fix");
        let write_main = |answer: &str| {
            fs::write(
                dir.join("main.fix"),
                format!(
                    "module Main;\nimport Main.Sub;\n\nmain : IO ();\nmain = println $ (Main.Sub::value + {}).to_string;\n",
                    answer
                ),
            )
            .expect("Failed to write main.fix");
        };
        let both_files = ["--file", "sub.fix"];

        // The second build finds every body in the cache, so the cache then holds the bodies of
        // both modules, and the build after the edit writes the file of `Main` alone.
        write_main("1");
        build(dir, "main.fix", &both_files);
        build(dir, "main.fix", &both_files);
        write_main("2");
        build(dir, "main.fix", &both_files);
        assert_eq!(run_out(dir), "3");
        let after_edit = cache_files(dir);

        build(dir, "main.fix", &both_files);
        assert_eq!(
            cache_files(dir),
            after_edit,
            "the build after the edit removed a file of another module"
        );
    }

    /// A build that cannot write the file of a module still builds the program from the bodies it
    /// checked, and reports the file it could not write.
    #[test]
    fn a_build_that_cannot_write_the_cache_builds_the_program() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("main.fix"),
            r#"module Main;

double : I64 -> I64;
double = |x| x * 2;

main : IO ();
main = println $ (double(21), 3).to_string;
"#,
        )
        .expect("Failed to write main.fix");
        let cache_dir = dir.join(TYPED_MODULES_PATH);
        fs::create_dir_all(&cache_dir).expect("Failed to create the type-check cache directory");
        fs::set_permissions(&cache_dir, Permissions::from_mode(0o555))
            .expect("Failed to make the type-check cache directory read-only");

        let (built, stderr) = try_build(dir, "main.fix", &[]);
        fs::set_permissions(&cache_dir, Permissions::from_mode(0o755))
            .expect("Failed to make the type-check cache directory writable");
        assert!(built, "stderr: {}", stderr);
        assert!(
            stderr.contains("Failed to write cache file"),
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "(42, 3)");
    }

    /// A build reads back the bodies it wrote to the files of their modules, as it reads those an
    /// earlier build wrote, and finds every one of them where it wrote it.
    #[test]
    fn a_build_reads_back_every_body_it_wrote() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("main.fix"),
            "module Main;\n\nmain : IO ();\nmain = println $ (1, 2, 3).to_string;\n",
        )
        .expect("Failed to write main.fix");

        let stderr = build(dir, "main.fix", &[]);
        assert!(
            !stderr.contains("Failed to read cache file"),
            "the build did not find a body where it wrote it.\nstderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "(1, 2, 3)");
    }

    /// Verifies that a program with more modules than a process may hold files open builds, and
    /// reads every body back, both from the files the build wrote and from those an earlier build
    /// wrote: a build closes each file once it has mapped it into memory.
    #[test]
    fn a_program_of_more_modules_than_the_open_file_limit_reads_back_every_body() {
        const MODULES: usize = 100;
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let mut files = vec![];
        let mut main = "module Main;\n\n".to_string();
        let mut terms = vec![];
        for i in 0..MODULES {
            let file = format!("m{}.fix", i);
            fs::write(
                dir.join(&file),
                format!("module M{};\n\nvalue : I64 = {};\n", i, i),
            )
            .expect("Failed to write a module");
            files.push(format!("\"{}\"", file));
            main += &format!("import M{};\n", i);
            terms.push(format!("M{}::value", i));
        }
        main += &format!(
            "\nmain : IO () = println(({}).to_string);\n",
            terms.join(" + ")
        );
        fs::write(dir.join("main.fix"), main).expect("Failed to write main.fix");
        files.push("\"main.fix\"".to_string());
        fs::write(
            dir.join("fixproj.toml"),
            format!(
                "[general]\nname = \"many\"\nversion = \"0.1.0\"\n[build]\nfiles = [{}]\n",
                files.join(", ")
            ),
        )
        .expect("Failed to write fixproj.toml");

        let fix = fix_command().get_program().to_os_string();
        for build in ["the first build", "the second build"] {
            // The soft limit is lowered below the number of modules.
            let output = Command::new("sh")
                .args(["-c", "ulimit -Sn 64 && exec \"$0\" build -o out"])
                .arg(&fix)
                .current_dir(dir)
                .output()
                .expect("Failed to run fix build");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(output.status.success(), "{} failed:\n{}", build, stderr);
            assert!(
                !stderr.contains("cache file"),
                "{} could not read or write the files of the modules.\nstderr: {}",
                build,
                stderr
            );
        }
        assert_eq!(run_out(dir), (0..MODULES).sum::<usize>().to_string());
    }

    /// A program of `count` functions, every one used from `main`, so that type checking it takes
    /// long enough for another process to act while it runs.
    fn source_with_many_functions(count: usize) -> String {
        let mut source = "module Main;\n\n".to_string();
        for i in 0..count {
            source += &format!("f{} : I64 -> I64;\nf{} = |x| x * {} + 1;\n\n", i, i, i);
        }
        let terms: Vec<String> = (0..count).map(|i| format!("f{}(1)", i)).collect();
        source += &format!(
            "main : IO ();\nmain = println $ ({}).to_string;\n",
            terms.join(" + ")
        );
        source
    }

    /// Two builds of one project at once each write the files of the modules, laying the bodies out
    /// in the order they happen to be checked, and the one renamed into place last wins. Each build
    /// still reads every body it needs from the file it opened, without reporting the file as
    /// unreadable.
    #[test]
    fn two_builds_at_once_read_back_every_body() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(dir.join("main.fix"), source_with_many_functions(300))
            .expect("Failed to write main.fix");

        let builds: Vec<_> = ["out1", "out2"]
            .iter()
            .map(|out| {
                fix_command()
                    .args(["build", "--file", "main.fix", "-o", out])
                    .current_dir(dir)
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to execute fix")
            })
            .collect();
        for build in builds {
            let output = build.wait_with_output().expect("Failed to wait for fix");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(output.status.success(), "stderr: {}", stderr);
            assert!(
                !stderr.contains("Failed to read cache file"),
                "a build did not find a body in the file it mapped.\nstderr: {}",
                stderr
            );
        }
    }

    /// A build that writes the file of a module removes the temporary files there that have gone
    /// unmodified for a day, and keeps the more recent ones, which a build still running may be
    /// writing.
    #[test]
    fn a_build_removes_the_temporary_files_left_a_day_ago() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("main.fix"),
            "module Main;\n\nmain : IO ();\nmain = println $ (1, 2, 3).to_string;\n",
        )
        .expect("Failed to write main.fix");
        let cache_dir = dir.join(TYPED_MODULES_PATH);
        fs::create_dir_all(&cache_dir).expect("Failed to create the type-check cache directory");
        let recent_path = cache_dir.join(".0123456789abcdef.tmp");
        fs::write(&recent_path, "").expect("Failed to create a temporary file");
        let left_path = cache_dir.join(".fedcba9876543210.tmp");
        File::create(&left_path)
            .and_then(|file| {
                file.set_modified(SystemTime::now() - Duration::from_secs(25 * 60 * 60))
            })
            .expect("Failed to create a temporary file a day old");

        build(dir, "main.fix", &[]);
        assert!(
            !left_path.exists(),
            "the build kept the temporary file left a day ago"
        );
        assert!(
            recent_path.exists(),
            "the build removed a temporary file a build still running may be writing"
        );
    }

    /// A build reads each body the file of its module records from that file as the build found it,
    /// so a file another process removes while the build runs still hands the build every body it
    /// recorded: the build reports no file as unreadable and checks none of those bodies again.
    ///
    /// `main.fix` is edited, so the build checks `Main` and writes its file; the file of
    /// `Std` is removed once that writing has begun, which is after the build read the record of
    /// every module and before instantiation reads a body of `Std`.
    #[test]
    fn a_build_reads_every_body_from_the_file_it_found_after_the_file_is_removed() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let cache_dir = dir.join(TYPED_MODULES_PATH);
        let names_in_cache = || -> Vec<String> {
            fs::read_dir(&cache_dir)
                .map(|entries| {
                    entries
                        .flatten()
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                        .collect()
                })
                .unwrap_or_default()
        };
        fs::write(dir.join("main.fix"), source_with_many_functions(300))
            .expect("Failed to write main.fix");
        build(dir, "main.fix", &[]);
        let std_file = names_in_cache()
            .into_iter()
            .find(|name| name.starts_with("Std_"))
            .expect("the build wrote no cache file of `Std`");

        fs::write(dir.join("main.fix"), source_with_many_functions(301))
            .expect("Failed to write main.fix");
        let mut running = fix_command()
            .args(["build", "--file", "main.fix", "-o", "out"])
            .current_dir(dir)
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to execute fix");
        while !names_in_cache().iter().any(|name| name.ends_with(".tmp")) {
            assert!(
                running
                    .try_wait()
                    .expect("Failed to wait for fix")
                    .is_none(),
                "the build ended before it began writing a file"
            );
            thread::sleep(Duration::from_millis(1));
        }
        fs::remove_file(cache_dir.join(&std_file)).expect("Failed to remove the file of `Std`");
        let output = running.wait_with_output().expect("Failed to wait for fix");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {}", stderr);
        assert!(
            !stderr.contains("Failed to read cache file"),
            "the build did not find a body in the file it mapped.\nstderr: {}",
            stderr
        );
        assert!(
            !names_in_cache().iter().any(|name| name.starts_with("Std_")),
            "the build checked a body of `Std` again"
        );
        let expected: i64 = (0..301).map(|i: i64| i + 1).sum();
        assert_eq!(run_out(dir), expected.to_string());
    }

    /// A file of a module shorter than its footer is a file the build cannot read the record of;
    /// the build reports it, checks the module again and writes the file anew, as for any damaged
    /// file.
    #[test]
    fn a_build_recovers_from_a_cache_file_shorter_than_its_footer() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let main_file = build_three_and_find_the_cache_file_of_main(dir);
        let written = fs::read(&main_file).expect("Failed to read the cache file of `Main`");

        for len in [0, 3] {
            fs::write(&main_file, &written[..len]).expect("Failed to truncate the cache file");
            let (built, stderr) = try_build(dir, "main.fix", &[]);
            assert!(built, "a file of {} bytes: stderr: {}", len, stderr);
            assert!(
                stderr.contains("Failed to read cache file"),
                "a file of {} bytes: stderr: {}",
                len,
                stderr
            );
            assert_eq!(run_out(dir), "3");
            assert_eq!(
                fs::read(&main_file).expect("Failed to read the cache file of `Main`"),
                written,
                "the build checking `Main` again wrote another file than the first build"
            );
        }
    }

    /// A module file the build cannot read is reported once, and the build writes the module's
    /// file anew in its place, so the build after it reads every body again and reports nothing.
    #[test]
    fn a_build_replaces_a_cache_file_it_cannot_read() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let main_file = build_three_and_find_the_cache_file_of_main(dir);
        fs::set_permissions(&main_file, Permissions::from_mode(0o000))
            .expect("Failed to make the cache file of `Main` unreadable");

        let stderr = build(dir, "main.fix", &[]);
        assert_eq!(
            stderr.matches("Failed to read cache file").count(),
            1,
            "stderr: {}",
            stderr
        );
        assert!(
            !stderr.contains("Failed to write cache file"),
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "3");
        let stderr = build(dir, "main.fix", &[]);
        assert!(!stderr.contains("cache file"), "stderr: {}", stderr);
        assert_eq!(run_out(dir), "3");
    }

    /// The names of the files of parsed bodies in `dir`, in order.
    fn parsed_files(dir: &Path) -> Vec<String> {
        files_under(dir, PARSED_MODULES_PATH)
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    }

    /// Writes into `dir` a module `Main` printing a value of the module `Lib`, and `Lib` defining
    /// the value as `value`, so that a build after an edit of `lib.fix` type-checks the body of
    /// `main` again from the source `main.fix` that the edit leaves as it was.
    fn write_main_and_lib(dir: &Path, value: &str) {
        fs::write(
            dir.join("main.fix"),
            "module Main;\nimport Lib;\n\nmain : IO ();\nmain = println $ (Lib::value : I64).to_string;\n",
        )
        .expect("Failed to write main.fix");
        fs::write(
            dir.join("lib.fix"),
            format!("module Lib;\n\nvalue : I64;\nvalue = {};\n", value),
        )
        .expect("Failed to write lib.fix");
    }

    /// The file of parsed bodies the builds in `dir` wrote for `main.fix`.
    fn parsed_file_of_main(dir: &Path) -> PathBuf {
        let name = parsed_files(dir)
            .into_iter()
            .find(|name| name.starts_with("main_fix_"))
            .expect("the build wrote no file of parsed bodies of main.fix");
        dir.join(PARSED_MODULES_PATH).join(name)
    }

    /// A body that fails to read back from its file of parsed bodies is taken from a parse of its
    /// source file made again. The file is reported once, and written anew from that parse, so the
    /// file is again what the first build wrote, and the build after it reads every body back and
    /// reports nothing.
    #[test]
    fn a_build_parses_a_source_again_when_its_file_of_parsed_bodies_is_damaged() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let lib = ["--file", "lib.fix"];
        write_main_and_lib(dir, "1");
        build(dir, "main.fix", &lib);
        let main_file = parsed_file_of_main(dir);
        let written = fs::read(&main_file).expect("Failed to read the file of parsed bodies");
        // The first body lies at the beginning of the file, and bytes all set do not decode.
        let mut damaged = written.clone();
        damaged[..8].fill(0xff);
        fs::write(&main_file, &damaged).expect("Failed to damage the file of parsed bodies");

        write_main_and_lib(dir, "2");
        let stderr = build(dir, "main.fix", &lib);
        assert_eq!(
            stderr.matches("Failed to read cache file").count(),
            1,
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "2");
        assert_eq!(
            fs::read(&main_file).expect("Failed to read the file of parsed bodies"),
            written,
            "the build parsing `main.fix` again wrote another file than the first build"
        );

        write_main_and_lib(dir, "3");
        let stderr = build(dir, "main.fix", &lib);
        assert!(!stderr.contains("cache file"), "stderr: {}", stderr);
        assert_eq!(run_out(dir), "3");
    }

    /// A file of parsed bodies whose index cannot be read is reported once, and written anew as the
    /// build parses its source file.
    #[test]
    fn a_build_replaces_a_file_of_parsed_bodies_whose_index_is_damaged() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let lib = ["--file", "lib.fix"];
        write_main_and_lib(dir, "1");
        build(dir, "main.fix", &lib);
        let main_file = parsed_file_of_main(dir);
        let written = fs::read(&main_file).expect("Failed to read the file of parsed bodies");
        fs::write(&main_file, &written[..written.len() - 1])
            .expect("Failed to damage the file of parsed bodies");

        let stderr = build(dir, "main.fix", &lib);
        assert_eq!(
            stderr.matches("Failed to read cache file").count(),
            1,
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "1");
        assert_eq!(
            fs::read(&main_file).expect("Failed to read the file of parsed bodies"),
            written,
            "the build parsing `main.fix` wrote another file than the first build"
        );
    }

    /// A build that cannot write the files of parsed bodies builds the program from the bodies it
    /// holds in memory, and reports the directory once, however many source files it parses.
    #[test]
    fn a_build_that_cannot_write_the_parsed_bodies_builds_the_program() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        write_main_and_lib(dir, "1");
        let parsed_dir = dir.join(PARSED_MODULES_PATH);
        fs::create_dir_all(&parsed_dir)
            .expect("Failed to create the directory of the files of parsed bodies");
        fs::set_permissions(&parsed_dir, Permissions::from_mode(0o555))
            .expect("Failed to make the directory of the files of parsed bodies read-only");

        let (built, stderr) = try_build(dir, "main.fix", &["--file", "lib.fix"]);
        fs::set_permissions(&parsed_dir, Permissions::from_mode(0o755))
            .expect("Failed to make the directory of the files of parsed bodies writable");
        assert!(built, "stderr: {}", stderr);
        assert_eq!(
            stderr.matches("Failed to write cache file").count(),
            1,
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "1");
    }

    /// A build after an edit of a source file replaces the file of parsed bodies of that source
    /// file, and leaves the files of the others as they were.
    #[test]
    fn a_build_after_an_edit_replaces_the_file_of_parsed_bodies_of_the_source_edited() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let lib = ["--file", "lib.fix"];
        write_main_and_lib(dir, "1");
        build(dir, "main.fix", &lib);
        let before = parsed_files(dir);

        write_main_and_lib(dir, "2");
        build(dir, "main.fix", &lib);
        assert_eq!(run_out(dir), "2");
        let after = parsed_files(dir);
        let of_lib = |files: &[String]| -> Vec<String> {
            files
                .iter()
                .filter(|name| name.starts_with("lib_fix_"))
                .cloned()
                .collect()
        };
        let others = |files: &[String]| -> Vec<String> {
            files
                .iter()
                .filter(|name| !name.starts_with("lib_fix_"))
                .cloned()
                .collect()
        };
        assert_eq!(of_lib(&after).len(), 1, "files: {:?}", after);
        assert_ne!(of_lib(&before), of_lib(&after), "files: {:?}", after);
        assert_eq!(others(&before), others(&after), "files: {:?}", after);
    }

    /// A source the compiler writes is named after its content, and its file of parsed bodies
    /// replaces the file of its other contents: a build using other sizes of tuples than the build
    /// before it leaves one file of the implementations `Std` carries for tuples.
    #[test]
    fn a_build_using_other_tuple_sizes_replaces_the_file_of_parsed_bodies_of_the_tuples() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let write_main = |tuple: &str| {
            fs::write(
                dir.join("main.fix"),
                format!(
                    "module Main;\n\nmain : IO ();\nmain = println $ {}.to_string;\n",
                    tuple
                ),
            )
            .expect("Failed to write main.fix");
        };
        let of_tuples = |dir: &Path| -> Vec<String> {
            parsed_files(dir)
                .into_iter()
                .filter(|name| name.starts_with("std_tuple_traits_"))
                .collect()
        };
        write_main("(1, 2)");
        build(dir, "main.fix", &[]);
        let before = of_tuples(dir);
        write_main("(1, 2, 3, 4, 5, 6, 7, 8, 9)");
        build(dir, "main.fix", &[]);
        assert_eq!(run_out(dir), "(1, 2, 3, 4, 5, 6, 7, 8, 9)");
        let after = of_tuples(dir);
        assert_eq!(after.len(), 1, "files: {:?}", parsed_files(dir));
        assert_ne!(before, after, "files: {:?}", parsed_files(dir));
    }

    /// A build that finds the file of parsed bodies of each source file it parses reads only the
    /// index, and leaves every file as the build before it wrote it.
    #[test]
    fn a_second_build_of_an_unedited_program_leaves_the_files_of_parsed_bodies_as_they_were() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let lib = ["--file", "lib.fix"];
        write_main_and_lib(dir, "1");
        build(dir, "main.fix", &lib);
        let written = files_under(dir, PARSED_MODULES_PATH);

        let stderr = build(dir, "main.fix", &lib);
        assert!(!stderr.contains("cache file"), "stderr: {}", stderr);
        assert_eq!(run_out(dir), "1");
        assert_eq!(
            files_under(dir, PARSED_MODULES_PATH),
            written,
            "the second build wrote a file of parsed bodies the first build had written"
        );
    }

    /// A file of parsed bodies whose index lacks a body of its source file is reported once, and
    /// written anew as the build parses its source file.
    #[test]
    fn a_build_replaces_a_file_of_parsed_bodies_that_lacks_a_body_of_its_source() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        let lib = ["--file", "lib.fix"];
        write_main_and_lib(dir, "1");
        build(dir, "main.fix", &lib);
        let main_file = parsed_file_of_main(dir);
        let written = fs::read(&main_file).expect("Failed to read the file of parsed bodies");
        let lib_file = parsed_files(dir)
            .into_iter()
            .find(|name| name.starts_with("lib_fix_"))
            .expect("the build wrote no file of parsed bodies of lib.fix");
        // The bodies of `lib.fix` are written at other places than the body of `main.fix`.
        fs::copy(dir.join(PARSED_MODULES_PATH).join(lib_file), &main_file)
            .expect("Failed to put the file of lib.fix in place of the file of main.fix");

        let stderr = build(dir, "main.fix", &lib);
        assert_eq!(
            stderr.matches("Failed to read cache file").count(),
            1,
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "1");
        assert_eq!(
            fs::read(&main_file).expect("Failed to read the file of parsed bodies"),
            written,
            "the build parsing `main.fix` wrote another file than the first build"
        );
    }

    /// A body whose place in its file of parsed bodies holds the expression of another body of the
    /// same source file is read as one that fails to read back: the build reports the file once,
    /// and takes the bodies from a parse of the source file made again, so each value keeps the
    /// body its source writes, and the file is again what the first build wrote.
    #[test]
    fn a_build_parses_a_source_again_when_its_file_of_parsed_bodies_holds_another_body_at_a_place()
    {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::write(
            dir.join("main.fix"),
            "module Main;\nimport Lib;\n\nmain : IO ();\nmain = println $ Lib::value.to_string + Lib::other.to_string;\n",
        )
        .expect("Failed to write main.fix");
        fs::write(
            dir.join("lib.fix"),
            "module Lib;\n\nvalue : I64;\nvalue = 1;\n\nother : I64;\nother = 2;\n",
        )
        .expect("Failed to write lib.fix");
        let lib = ["--file", "lib.fix"];
        build(dir, "main.fix", &lib);
        assert_eq!(run_out(dir), "12");

        let lib_file = dir.join(PARSED_MODULES_PATH).join(
            parsed_files(dir)
                .into_iter()
                .find(|name| name.starts_with("lib_fix_"))
                .expect("the build wrote no file of parsed bodies of lib.fix"),
        );
        let written = fs::read(&lib_file).expect("Failed to read the file of parsed bodies");
        // The footer begins with the offset of the index, and the index lists where each body
        // lies. The two bodies are written the same length, so swapping their bytes leaves each
        // place holding a whole expression, of the other body.
        let footer = written.len() - 24;
        let index_start = u64::from_le_bytes(written[footer..footer + 8].try_into().unwrap());
        let index: Vec<((usize, usize), std::ops::Range<u64>)> =
            postcard::from_bytes(&written[index_start as usize..footer])
                .expect("Failed to read the index of the file of parsed bodies");
        let [(_, first), (_, second)] = &index[..] else {
            panic!("the file of parsed bodies of lib.fix holds {} bodies", index.len())
        };
        let (first, second) = (
            first.start as usize..first.end as usize,
            second.start as usize..second.end as usize,
        );
        assert_eq!(first.len(), second.len(), "index: {:?}", index);
        let mut swapped = written.clone();
        swapped[first.clone()].copy_from_slice(&written[second.clone()]);
        swapped[second].copy_from_slice(&written[first]);
        fs::write(&lib_file, &swapped).expect("Failed to swap the bodies of lib.fix");
        // Without its record, `Lib` is type-checked again from the bodies in the file.
        for (name, _) in files_under(dir, TYPED_MODULES_PATH) {
            if name.starts_with("Lib_") {
                fs::remove_file(dir.join(TYPED_MODULES_PATH).join(name))
                    .expect("Failed to remove the file of `Lib`");
            }
        }

        let stderr = build(dir, "main.fix", &lib);
        assert_eq!(
            stderr.matches("Failed to read cache file").count(),
            1,
            "stderr: {}",
            stderr
        );
        assert_eq!(run_out(dir), "12");
        assert_eq!(
            fs::read(&lib_file).expect("Failed to read the file of parsed bodies"),
            written,
            "the build parsing `lib.fix` again wrote another file than the first build"
        );
    }
}
