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

    fn array_main_fix() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/tests/test_debug_info/cases/debug_array/main.fix");
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

    // Line number in cases/debug_array/main.fix. If that file changes, update this.
    const LINE_ARRAY_BREAK: u32 = 8; // "    let sum = arr3.@(0) + arr150.@(0);"

    // Checks that gdb displays the elements of `Array` / `String` values.
    // The debug info claims a fixed number of elements (`DEBUG_ARRAY_ASSUMED_LEN`,
    // 100) with byte sizes covering all of them, so gdb shows 100 elements whose
    // first `<array size>` ones are the valid values, without "access outside
    // bounds" errors.
    #[test]
    fn test_debug_info_array_elements() {
        install_fix();

        let temp = TempDir::new().expect("Failed to create temp directory");
        let dir = temp.path();
        fs::copy(array_main_fix(), dir.join("main.fix")).expect("Failed to copy main.fix");

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

        // Drive gdb: break while the arrays are still alive (they are used after the
        // breakpoint line; Fix releases locals at their last use), then print them.
        let breakpoint = format!("break main.fix:{}", LINE_ARRAY_BREAK);
        let gdb = Command::new("gdb")
            .args([
                "-batch",
                "-iex",
                "set debuginfod enabled off",
                "-ex",
                "set print elements unlimited",
                "-ex",
                &breakpoint,
                "-ex",
                "run",
                "-ex",
                "print *arr3",
                "-ex",
                "print *arr150",
                "-ex",
                "print *msg._data",
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

        // (1) Execution stopped at the breakpoint.
        assert!(
            out.contains("Breakpoint 1, ")
                && out.contains(&format!("main.fix:{}", LINE_ARRAY_BREAK)),
            "execution did not stop at main.fix:{}.\ngdb output:\n{}",
            LINE_ARRAY_BREAK,
            out
        );

        // (2) A 3-element array prints its size and its valid elements first.
        //     (Elements after the valid ones are unspecified memory, so only the
        //     prefix is asserted.)
        assert!(
            out.contains("<array size> = 3") && out.contains("<array elements> = {10, 20, 30"),
            "3-element array was not printed with its valid elements first.\ngdb output:\n{}",
            out
        );

        // (3) A 150-element array prints its size and the first 100 elements; the
        //     100th displayed element (value 1000) is directly followed by the closing
        //     brace.
        assert!(
            out.contains("<array size> = 150") && out.contains("980, 990, 1000}"),
            "150-element array was not printed up to its 100th element.\ngdb output:\n{}",
            out
        );

        // (4) A string prints its bytes as a readable string. (Bytes after the
        //     terminating NUL are unspecified memory, so only the prefix is asserted.)
        assert!(
            out.contains("<array elements> = \"hello"),
            "string bytes were not printed as \"hello...\".\ngdb output:\n{}",
            out
        );

        // (5) No out-of-bounds read errors anywhere in the output.
        for err in ["access outside bounds of object", "error reading variable"] {
            assert!(
                !out.contains(err),
                "gdb reported `{}`.\ngdb output:\n{}",
                err,
                out
            );
        }
    }
}
