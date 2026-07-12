// P0 baseline: debug-information end-to-end test.
//
// Builds a small Fix program with `-g` and drives `gdb -batch` to confirm that
// DWARF line information is emitted correctly: a source breakpoint resolves to
// `main.fix:<line>`, execution stops there, and the backtrace carries per-frame
// line info up the Fix call chain. Assertions are mangle-name-independent (they
// check `file:line`, not the mangled/closure frame names), so they stay valid
// across name-mangling changes.
//
// This is the comparison baseline for the RC-IR codegen swap (P1), whose gate
// requires debug information to be unchanged.

#[cfg(test)]
mod debug_info_tests {
    use crate::tests::test_util::install_fix;
    use std::{fs, path::PathBuf, process::Command};
    use tempfile::TempDir;

    fn sample_main_fix() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/tests/test_debug_info/cases/debug_baseline/main.fix");
        p
    }

    // Line numbers in cases/debug_baseline/main.fix. If that file changes, update these.
    const LINE_COMPUTE_BODY: u32 = 5; // "    let y = x + 1;"           (inside `compute`)
    const LINE_WRAP_DEF: u32 = 10; //    "wrap = |x| compute(x + 10);"  (call site of `compute`)
    const LINE_MAIN_CALL: u32 = 14; //   "    let r = wrap(5);"         (call site of `wrap`)

    #[test]
    fn test_debug_info_baseline() {
        install_fix();

        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::copy(sample_main_fix(), dir.join("main.fix")).expect("Failed to copy main.fix");

        // Build with debug information (`-g` also forces `-O none`).
        let build = Command::new("fix")
            .args(["build", "-g", "-f", "main.fix", "-o", "prog"])
            .current_dir(dir)
            .output()
            .expect("Failed to execute `fix build`");
        assert!(
            build.status.success(),
            "`fix build -g` failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            dir.join("prog").exists(),
            "output binary `prog` was not produced by `fix build -g`"
        );

        // Drive gdb: break inside `compute`, run to the breakpoint, print a backtrace.
        let breakpoint = format!("break main.fix:{}", LINE_COMPUTE_BODY);
        let gdb = Command::new("gdb")
            .args([
                "-batch",
                "-iex",
                "set debuginfod enabled off",
                "-ex",
                &breakpoint,
                "-ex",
                "run",
                "-ex",
                "backtrace",
                "-ex",
                "continue",
                "./prog",
            ])
            .current_dir(dir)
            .output()
            .expect("Failed to execute `gdb` (is /usr/bin/gdb installed?)");
        let out = format!(
            "{}{}",
            String::from_utf8_lossy(&gdb.stdout),
            String::from_utf8_lossy(&gdb.stderr)
        );

        // (1) The source breakpoint resolves to `main.fix` at the requested line.
        assert!(
            out.contains(&format!("file main.fix, line {}", LINE_COMPUTE_BODY)),
            "breakpoint did not resolve to main.fix:{}.\ngdb output:\n{}",
            LINE_COMPUTE_BODY,
            out
        );

        // (2) Execution actually stopped at that breakpoint.
        assert!(
            out.contains("Breakpoint 1, ")
                && out.contains(&format!("main.fix:{}", LINE_COMPUTE_BODY)),
            "execution did not stop at main.fix:{}.\ngdb output:\n{}",
            LINE_COMPUTE_BODY,
            out
        );

        // (3) The backtrace carries per-frame line info up the Fix call chain
        //     (wrap's call site and main's call site), independent of frame names.
        for line in [LINE_WRAP_DEF, LINE_MAIN_CALL] {
            assert!(
                out.contains(&format!("main.fix:{}", line)),
                "backtrace is missing frame line info main.fix:{}.\ngdb output:\n{}",
                line,
                out
            );
        }
    }

    fn sample_debug_vars() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/tests/test_debug_info/cases/debug_vars/main.fix");
        p
    }

    // Line in cases/debug_vars/main.fix where all locals (i, bt, bf, arr, s) are live.
    const LINE_VARS_BREAK: u32 = 10; // "    eval i;"

    // Debug info drives correct variable inspection at a breakpoint. Unboxed values print their
    // value — an `I64` as its number, a `Bool` as `true` / `false` (i.e. `Bool`'s debug type is
    // `DW_ATE_boolean`, not a union struct). Boxed containers carry their Fix type (`Std::Array
    // Std::I64`, `Std::String`), and an `Array` also exposes its size. `-g` forces `-O none`, so
    // the locals are not optimized away.
    #[test]
    fn test_debug_info_variable_values() {
        install_fix();

        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::copy(sample_debug_vars(), dir.join("main.fix")).expect("Failed to copy main.fix");

        let build = Command::new("fix")
            .args(["build", "-g", "-f", "main.fix", "-o", "prog"])
            .current_dir(dir)
            .output()
            .expect("Failed to execute `fix build`");
        assert!(
            build.status.success(),
            "`fix build -g` failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let breakpoint = format!("break main.fix:{}", LINE_VARS_BREAK);
        let gdb = Command::new("gdb")
            .args([
                "-batch",
                "-iex",
                "set debuginfod enabled off",
                "-ex",
                &breakpoint,
                "-ex",
                "run",
                "-ex",
                "print i",
                "-ex",
                "print bt",
                "-ex",
                "print bf",
                "-ex",
                "whatis arr",
                "-ex",
                "print *arr",
                "-ex",
                "whatis s",
                // A String's characters are the bytes of its `_data` array, whose elements begin
                // after the 24-byte array header (control block + size + capacity on a 64-bit
                // target). The debug info cannot bound the flexible element array, so read them as a
                // C string from that offset.
                "-ex",
                "x/s (char*)s._data + 24",
                "-ex",
                "continue",
                "./prog",
            ])
            .current_dir(dir)
            .output()
            .expect("Failed to execute `gdb`");
        let out = format!(
            "{}{}",
            String::from_utf8_lossy(&gdb.stdout),
            String::from_utf8_lossy(&gdb.stderr)
        );

        for (needle, what) in [
            ("= 42", "I64 value"),
            ("= true", "Bool `true`"),
            ("= false", "Bool `false`"),
            ("Std::Array Std::I64", "Array type"),
            ("<array size> = 3", "Array size"),
            ("Std::String", "String type"),
            ("\"hello\"", "String contents (raw bytes)"),
        ] {
            assert!(
                out.contains(needle),
                "gdb did not show {} (expected `{}`).\ngdb output:\n{}",
                what,
                needle,
                out
            );
        }
    }

    fn sample_debug_destructure() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/tests/test_debug_info/cases/debug_destructure/main.fix");
        p
    }

    // Line in cases/debug_destructure/main.fix where the destructure-bound locals (a, arr, n, str)
    // are live.
    const LINE_DESTRUCTURE_BREAK: u32 = 9; // "    eval a;"

    // A `let`-pattern that destructures a tuple binds each field to a source variable; debug info
    // must let a debugger inspect every one by its source name. `a` and `n` are the unboxed `I64`
    // fields, `arr` and `str` the boxed `Array`/`String` fields, each extracted from its tuple.
    #[test]
    fn test_debug_info_destructure() {
        install_fix();

        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::copy(sample_debug_destructure(), dir.join("main.fix"))
            .expect("Failed to copy main.fix");

        let build = Command::new("fix")
            .args(["build", "-g", "-f", "main.fix", "-o", "prog"])
            .current_dir(dir)
            .output()
            .expect("Failed to execute `fix build`");
        assert!(
            build.status.success(),
            "`fix build -g` failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let breakpoint = format!("break main.fix:{}", LINE_DESTRUCTURE_BREAK);
        let gdb = Command::new("gdb")
            .args([
                "-batch",
                "-iex",
                "set debuginfod enabled off",
                "-ex",
                &breakpoint,
                "-ex",
                "run",
                "-ex",
                "print a",
                "-ex",
                "print n",
                "-ex",
                "whatis arr",
                "-ex",
                "print *arr",
                "-ex",
                "whatis str",
                "-ex",
                "x/s (char*)str._data + 24",
                "-ex",
                "continue",
                "./prog",
            ])
            .current_dir(dir)
            .output()
            .expect("Failed to execute `gdb`");
        let out = format!(
            "{}{}",
            String::from_utf8_lossy(&gdb.stdout),
            String::from_utf8_lossy(&gdb.stderr)
        );

        for (needle, what) in [
            ("= 7", "destructured I64 field `a`"),
            ("= 5", "destructured I64 field `n`"),
            ("Std::Array Std::I64", "destructured Array field `arr` type"),
            ("<array size> = 3", "destructured Array field `arr` size"),
            ("Std::String", "destructured String field `str` type"),
            ("\"hello\"", "destructured String field `str` contents"),
        ] {
            assert!(
                out.contains(needle),
                "gdb did not show {} (expected `{}`).\ngdb output:\n{}",
                what,
                needle,
                out
            );
        }
    }
}
