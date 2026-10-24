//! An entry of a project file's `objects` is the path of a file to link, whatever character it
//! starts with: the C compiler reads neither one starting with `-` as an option nor one starting
//! with `@` as a file of further options.

use crate::tests::test_util::{assert_succeeded, compile_c_object, run_fix};
use std::fs;
use tempfile::TempDir;

/// Builds and runs, in a fresh directory, a project whose `objects` lists the object file `name`,
/// which defines the C function the program calls, and checks that the program prints what that
/// function returns. `other_files` are written into the directory beside it, as (name, contents).
fn assert_object_named_links(name: &str, other_files: &[(&str, &str)]) {
    let dir = TempDir::new().unwrap();
    compile_c_object("int helper(void) { return 42; }\n", &dir.path().join(name));
    for (file, contents) in other_files {
        fs::write(dir.path().join(file), contents).unwrap();
    }
    fs::write(
        dir.path().join("fixproj.toml"),
        format!(
            "[general]\nname = \"objects\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"main.fix\"]\nobjects = [\"{}\"]\n",
            name
        ),
    )
    .unwrap();
    fs::write(
        dir.path().join("main.fix"),
        "module Main;\n\nmain : IO ();\nmain = println(FFI_CALL[CInt helper()].to_string);\n",
    )
    .unwrap();
    let output = run_fix(dir.path(), &["run", "-O", "none"]);
    assert_succeeded(
        &output,
        &format!("`fix run` should link the object \"{}\".", name),
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42\n");
}

#[test]
fn an_object_named_like_an_option_is_linked_as_a_file() {
    assert_object_named_links("-helper.o", &[]);
}

#[test]
fn an_object_named_like_an_options_file_is_linked_as_a_file() {
    // `gcc` reads `@helper.o` as the file `helper.o` of further options when that file exists.
    assert_object_named_links("@helper.o", &[("helper.o", "--no-such-option\n")]);
}
