// Debug-information end-to-end test.
//
// Builds a small Fix program with `-g` and drives a source-level debugger to confirm that DWARF
// line information is emitted correctly: a source breakpoint resolves to `main.fix:<line>`,
// execution stops there, and the backtrace carries per-frame line info up the Fix call chain.
// Assertions are mangle-name-independent (they check `file:line`, not the mangled/closure frame
// names), so they stay valid across name-mangling changes.
//
// Each scenario runs under whichever debugger the host provides: gdb on Linux and lldb on macOS
// (gdb has no working Apple-Silicon support), with the lldb variants also running on a Linux host
// that has lldb installed. A scenario skips when its debugger is absent. The AST and RC IR back
// ends must emit identical debug information; these tests guard that it stays correct under both.

#[cfg(test)]
mod debug_info_tests {
    use crate::tests::test_util::fix_command;
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
    };
    use tempfile::TempDir;

    #[derive(Clone, Copy)]
    enum Debugger {
        Gdb,
        Lldb,
    }

    impl Debugger {
        fn program(self) -> &'static str {
            match self {
                Debugger::Gdb => "gdb",
                Debugger::Lldb => "lldb",
            }
        }

        // Whether the debugger can be launched on this host. A scenario skips when its debugger is
        // absent: macOS ships no working gdb, and a Linux host without lldb installed skips the lldb
        // variants.
        fn is_available(self) -> bool {
            Command::new(self.program())
                .arg("--version")
                .output()
                .is_ok()
        }

        // The line a debugger prints when it stops at a breakpoint.
        fn stopped_marker(self) -> &'static str {
            match self {
                Debugger::Gdb => "Breakpoint 1, ",
                Debugger::Lldb => "stop reason = breakpoint",
            }
        }
    }

    // Build `sample` with debug information into a fresh temp directory and return it. `-g` also
    // forces `-O none`, so the locals are not optimized away.
    fn build_debuggee(sample: PathBuf) -> TempDir {
        let temp = TempDir::new().expect("Failed to create temp directory");
        fs::copy(sample, temp.path().join("main.fix")).expect("Failed to copy main.fix");
        let build = fix_command()
            .args(["build", "-g", "-f", "main.fix", "-o", "prog"])
            .current_dir(temp.path())
            .output()
            .expect("Failed to execute `fix build`");
        assert!(
            build.status.success(),
            "`fix build -g` failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            temp.path().join("prog").exists(),
            "output binary `prog` was not produced by `fix build -g`"
        );
        temp
    }

    // Drive `debugger` over `./prog` in `dir`, issuing the given native `commands` in order, and
    // return its combined stdout+stderr.
    fn drive(debugger: Debugger, dir: &Path, commands: &[String]) -> String {
        let mut cmd = Command::new(debugger.program());
        match debugger {
            Debugger::Gdb => {
                cmd.args(["-batch", "-iex", "set debuginfod enabled off"]);
                for c in commands {
                    cmd.arg("-ex").arg(c);
                }
            }
            Debugger::Lldb => {
                cmd.args(["--batch", "--no-lldbinit"]);
                for c in commands {
                    cmd.arg("-o").arg(c);
                }
            }
        }
        cmd.arg("./prog");
        let out = cmd
            .current_dir(dir)
            .output()
            .expect("Failed to execute the debugger");
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }

    fn assert_contains(out: &str, needle: &str, what: &str) {
        assert!(
            out.contains(needle),
            "debugger did not show {} (expected `{}`).\ndebugger output:\n{}",
            what,
            needle,
            out
        );
    }

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

    // A source breakpoint resolves to `main.fix:<line>`, execution stops there, and the backtrace
    // carries per-frame line info up the Fix call chain (wrap's call site and main's call site),
    // independent of frame names.
    fn baseline_impl(debugger: Debugger) {
        let temp = build_debuggee(sample_main_fix());
        let commands = match debugger {
            Debugger::Gdb => vec![
                format!("break main.fix:{}", LINE_COMPUTE_BODY),
                "run".to_string(),
                "backtrace".to_string(),
                "continue".to_string(),
            ],
            Debugger::Lldb => vec![
                format!("breakpoint set --file main.fix --line {}", LINE_COMPUTE_BODY),
                "run".to_string(),
                "thread backtrace".to_string(),
                "continue".to_string(),
            ],
        };
        let out = drive(debugger, temp.path(), &commands);

        assert!(
            out.contains(debugger.stopped_marker())
                && out.contains(&format!("main.fix:{}", LINE_COMPUTE_BODY)),
            "execution did not stop at main.fix:{}.\ndebugger output:\n{}",
            LINE_COMPUTE_BODY,
            out
        );
        for line in [LINE_WRAP_DEF, LINE_MAIN_CALL] {
            assert!(
                out.contains(&format!("main.fix:{}", line)),
                "backtrace is missing frame line info main.fix:{}.\ndebugger output:\n{}",
                line,
                out
            );
        }
    }

    #[test]
    fn test_debug_info_baseline_gdb() {
        if !Debugger::Gdb.is_available() {
            eprintln!("skipping test_debug_info_baseline_gdb: gdb is not available");
            return;
        }
        baseline_impl(Debugger::Gdb);
    }

    #[test]
    fn test_debug_info_baseline_lldb() {
        if !Debugger::Lldb.is_available() {
            eprintln!("skipping test_debug_info_baseline_lldb: lldb is not available");
            return;
        }
        baseline_impl(Debugger::Lldb);
    }

    fn sample_debug_vars() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("src/tests/test_debug_info/cases/debug_vars/main.fix");
        p
    }

    // Line in cases/debug_vars/main.fix where all locals (i, bt, bf, arr, s) are live.
    const LINE_VARS_BREAK: u32 = 10; // "    eval i;"

    // Debug info drives correct variable inspection at a breakpoint. Unboxed scalars print their
    // value — an `I64` as its number, a `Bool` as `true` / `false` (i.e. `Bool`'s debug type is
    // `DW_ATE_boolean`, not a union struct). An `Array` / `String` local carries its Fix type name
    // (`Std::Array Std::I64`, `Std::String`), and an `Array` value also exposes its size directly.
    fn variable_values_impl(debugger: Debugger) {
        let temp = build_debuggee(sample_debug_vars());
        let commands: Vec<String> = match debugger {
            Debugger::Gdb => [
                format!("break main.fix:{}", LINE_VARS_BREAK).as_str(),
                "run",
                "print i",
                "print bt",
                "print bf",
                "whatis arr",
                "print arr",
                "whatis s",
                // A String's characters are the bytes of its `_data` array. After the flip those
                // elements live in the `#ArrayStorage` behind `_data._storage`, beginning right
                // after its 8-byte control block. The debug info cannot bound the flexible element
                // array, so read them as a C string from that offset.
                "x/s (char*)s._data._storage + 8",
                "continue",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            Debugger::Lldb => [
                format!("breakpoint set --file main.fix --line {}", LINE_VARS_BREAK).as_str(),
                "run",
                "frame variable i",
                "frame variable bt",
                "frame variable bf",
                "frame variable arr",
                "frame variable s",
                "x/s (char *)s._data._storage + 8",
                "continue",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        };
        let out = drive(debugger, temp.path(), &commands);

        // gdb prints the string read by `x/s` quoted; lldb's format differs, so match the bytes
        // without the surrounding quotes.
        let hello = match debugger {
            Debugger::Gdb => "\"hello\"",
            Debugger::Lldb => "hello",
        };
        for (needle, what) in [
            ("= 42", "I64 value"),
            ("= true", "Bool `true`"),
            ("= false", "Bool `false`"),
            ("Std::Array Std::I64", "Array type"),
            ("<array size> = 3", "Array size"),
            ("Std::String", "String type"),
            (hello, "String contents (raw bytes)"),
        ] {
            assert_contains(&out, needle, what);
        }
    }

    #[test]
    fn test_debug_info_variable_values_gdb() {
        if !Debugger::Gdb.is_available() {
            eprintln!("skipping test_debug_info_variable_values_gdb: gdb is not available");
            return;
        }
        variable_values_impl(Debugger::Gdb);
    }

    #[test]
    fn test_debug_info_variable_values_lldb() {
        if !Debugger::Lldb.is_available() {
            eprintln!("skipping test_debug_info_variable_values_lldb: lldb is not available");
            return;
        }
        variable_values_impl(Debugger::Lldb);
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
    fn destructure_impl(debugger: Debugger) {
        let temp = build_debuggee(sample_debug_destructure());
        let commands: Vec<String> = match debugger {
            Debugger::Gdb => [
                format!("break main.fix:{}", LINE_DESTRUCTURE_BREAK).as_str(),
                "run",
                "print a",
                "print n",
                "whatis arr",
                "print arr",
                "whatis str",
                "x/s (char*)str._data._storage + 8",
                "continue",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            Debugger::Lldb => [
                format!(
                    "breakpoint set --file main.fix --line {}",
                    LINE_DESTRUCTURE_BREAK
                )
                .as_str(),
                "run",
                "frame variable a",
                "frame variable n",
                "frame variable arr",
                "frame variable str",
                "x/s (char *)str._data._storage + 8",
                "continue",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        };
        let out = drive(debugger, temp.path(), &commands);

        let hello = match debugger {
            Debugger::Gdb => "\"hello\"",
            Debugger::Lldb => "hello",
        };
        for (needle, what) in [
            ("= 7", "destructured I64 field `a`"),
            ("= 5", "destructured I64 field `n`"),
            ("Std::Array Std::I64", "destructured Array field `arr` type"),
            ("<array size> = 3", "destructured Array field `arr` size"),
            ("Std::String", "destructured String field `str` type"),
            (hello, "destructured String field `str` contents"),
        ] {
            assert_contains(&out, needle, what);
        }
    }

    #[test]
    fn test_debug_info_destructure_gdb() {
        if !Debugger::Gdb.is_available() {
            eprintln!("skipping test_debug_info_destructure_gdb: gdb is not available");
            return;
        }
        destructure_impl(Debugger::Gdb);
    }

    #[test]
    fn test_debug_info_destructure_lldb() {
        if !Debugger::Lldb.is_available() {
            eprintln!("skipping test_debug_info_destructure_lldb: lldb is not available");
            return;
        }
        destructure_impl(Debugger::Lldb);
    }

    // Line number in cases/debug_array/main.fix. If that file changes, update this.
    const LINE_ARRAY_BREAK: u32 = 8; // "    let sum = arr3.@(0) + arr150.@(0);"

    // Checks that a debugger displays the elements of `Array` / `String` values. The debug info
    // claims a fixed number of elements (`DEBUG_ARRAY_ASSUMED_LEN`, 100) with byte sizes covering
    // all of them, so the debugger shows 100 elements whose first `<array size>` ones are the valid
    // values, without "access outside bounds" errors.
    fn array_elements_impl(debugger: Debugger) {
        let temp = build_debuggee(array_main_fix());
        // Break while the arrays are still alive (they are used after the breakpoint line; Fix
        // releases locals at their last use), then print them. A flipped `Array` value prints its
        // size directly, but its elements live in the `#ArrayStorage` behind `_storage`, so the
        // size and the elements come from two separate prints.
        let commands: Vec<String> = match debugger {
            Debugger::Gdb => [
                "set print elements unlimited",
                format!("break main.fix:{}", LINE_ARRAY_BREAK).as_str(),
                "run",
                "print arr3",
                "print *arr3._storage",
                "print arr150",
                "print *arr150._storage",
                "print *msg._data._storage",
                "continue",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            Debugger::Lldb => [
                "settings set target.max-children-count 10000",
                format!("breakpoint set --file main.fix --line {}", LINE_ARRAY_BREAK).as_str(),
                "run",
                "print arr3",
                "print *arr3._storage",
                "print arr150",
                "print *arr150._storage",
                "print *msg._data._storage",
                "continue",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        };
        let out = drive(debugger, temp.path(), &commands);

        // Execution stopped at the breakpoint.
        assert!(
            out.contains(debugger.stopped_marker())
                && out.contains(&format!("main.fix:{}", LINE_ARRAY_BREAK)),
            "execution did not stop at main.fix:{}.\ndebugger output:\n{}",
            LINE_ARRAY_BREAK,
            out
        );
        // A 3-element array prints its size and its valid elements first, and a 150-element array
        // its size and the first 100 elements (the 100th displayed value is 1000). Elements past
        // the valid ones are unspecified memory, so only the prefix is asserted. gdb renders the
        // synthetic members exactly; lldb's aggregate formatting differs, so match it loosely.
        match debugger {
            Debugger::Gdb => {
                assert!(
                    out.contains("<array size> = 3")
                        && out.contains("<array elements> = {10, 20, 30"),
                    "3-element array was not printed with its valid elements first.\ndebugger output:\n{}",
                    out
                );
                assert!(
                    out.contains("<array size> = 150") && out.contains("980, 990, 1000}"),
                    "150-element array was not printed up to its 100th element.\ndebugger output:\n{}",
                    out
                );
                assert_contains(&out, "<array elements> = \"hello", "string bytes as \"hello...\"");
            }
            Debugger::Lldb => {
                assert_contains(&out, "<array size> = 3", "3-element array size");
                assert_contains(&out, "10, 20, 30", "3-element array valid elements");
                assert_contains(&out, "<array size> = 150", "150-element array size");
                assert_contains(&out, "980, 990, 1000", "150-element array up to its 100th element");
                assert_contains(&out, "hello", "string bytes");
            }
        }
        // No out-of-bounds read errors anywhere in the output.
        for err in ["access outside bounds of object", "error reading variable"] {
            assert!(
                !out.contains(err),
                "debugger reported `{}`.\ndebugger output:\n{}",
                err,
                out
            );
        }
    }

    #[test]
    fn test_debug_info_array_elements_gdb() {
        if !Debugger::Gdb.is_available() {
            eprintln!("skipping test_debug_info_array_elements_gdb: gdb is not available");
            return;
        }
        array_elements_impl(Debugger::Gdb);
    }

    #[test]
    fn test_debug_info_array_elements_lldb() {
        if !Debugger::Lldb.is_available() {
            eprintln!("skipping test_debug_info_array_elements_lldb: lldb is not available");
            return;
        }
        array_elements_impl(Debugger::Lldb);
    }
}
