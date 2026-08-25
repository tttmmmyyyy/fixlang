//! A change to a type declaration reaches the code generated for every symbol that lays that type
//! out, while leaving those symbols' names, types and expressions as they were. The digest naming a
//! compilation unit's object file reads the declarations of the types the unit's code is laid out
//! by (`divide_program::type_declarations_reached`), which is what invalidates those symbols'
//! object files.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_command_at_opt_level;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    const FIXPROJ: &str = r#"[general]
name = "object-cache-layout-change"
version = "0.1.0"
[build]
files = ["main.fix", "mid.fix", "lib.fix"]
"#;

    /// `Lib` before the change: `T`'s second field is a scalar, so `T` is laid out as two integers.
    const LIB_BEFORE: &str = r#"module Lib;

type T = struct { a : I64, b : I64 };

make_t : I64 -> T;
make_t = |x| T { a : x, b : x + 1 };
"#;

    /// `Lib` after the change: `T`'s second field is a boxed array, so `T` is laid out as an
    /// integer and a pointer. `make_t` is the only global whose expression differs between the two,
    /// which leaves the field accessor `T::@a` — the global `Mid::show` calls — with the name, the
    /// type and the expression it had, and the layout it lays out changed underneath it.
    const LIB_AFTER: &str = r#"module Lib;

type T = struct { a : I64, b : Array I64 };

make_t : I64 -> T;
make_t = |x| T { a : x, b : [x + 1] };
"#;

    const MID_FIX: &str = r#"module Mid;

import Lib;

show : I64 -> String;
show = |n| Lib::make_t(n).@a.to_string;
"#;

    const MAIN_FIX: &str = r#"module Main;

import Mid;

main : IO ();
main = println(Mid::show(3));
"#;

    fn write_project(dir: &Path, lib: &str) {
        fs::write(dir.join("fixproj.toml"), FIXPROJ).unwrap();
        fs::write(dir.join("lib.fix"), lib).unwrap();
        fs::write(dir.join("mid.fix"), MID_FIX).unwrap();
        fs::write(dir.join("main.fix"), MAIN_FIX).unwrap();
    }

    fn build(dir: &Path) {
        let out = fix_command_at_opt_level("build", "basic")
            .args(["-o", "prog"])
            .current_dir(dir)
            .output()
            .expect("failed to run fix build");
        assert!(
            out.status.success(),
            "fix build failed in {}:\nstdout:\n{}\nstderr:\n{}",
            dir.display(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn run(dir: &Path) -> String {
        let out = Command::new(dir.join("prog"))
            .current_dir(dir)
            .output()
            .expect("failed to run the built program");
        assert!(
            out.status.success(),
            "the program built in {} exited with {}:\nstdout:\n{}\nstderr:\n{}",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    #[test]
    fn a_changed_type_layout_invalidates_the_object_files_that_lay_it_out() {
        let temp = TempDir::new().unwrap();

        // The build that keeps its object files: it compiles the type one way and then the other.
        let warm = temp.path().join("warm");
        fs::create_dir_all(&warm).unwrap();
        write_project(&warm, LIB_BEFORE);
        build(&warm);
        write_project(&warm, LIB_AFTER);
        build(&warm);

        // The build that has none: the same sources compiled from nothing.
        let cold = temp.path().join("cold");
        fs::create_dir_all(&cold).unwrap();
        write_project(&cold, LIB_AFTER);
        build(&cold);

        assert_eq!(
            run(&warm),
            run(&cold),
            "the build that reused its object files produced a different program from the one that \
             compiled the same sources from nothing"
        );
        assert_eq!(run(&cold), "3\n", "the program prints the field it reads");
    }
}
