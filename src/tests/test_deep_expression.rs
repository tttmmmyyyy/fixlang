// The compiler handles a deeply nested expression without overflowing its own
// stack. Type checking and code generation recurse to the expression's nesting
// depth, and so does reading such an expression back out of the type-check
// cache; all of them run on worker threads sized for that recursion via
// `COMPILER_THREAD_STACK_SIZE`, and the whole compilation runs on a
// thread of the same size, so a deep `let`/`;;` chain compiles rather than
// aborting with a stack overflow.

#[cfg(test)]
mod integration_tests {
    use crate::constants::TYPE_CHECK_CACHE_PATH;
    use crate::misc::Map;
    use crate::tests::test_util::{fix_build_source_command, fix_command_at_opt_level};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;
    use tempfile::TempDir;

    /// 300 exceeds the few-hundred nesting depth at which the compiler overflows a
    /// worker thread of the default stack size, so this build fails unless the
    /// compiler's threads are sized for deep recursion; it stays shallow enough to
    /// keep the `-O max` build time modest.
    const DEPTH: usize = 300;

    /// Verifies that a module whose expressions nest `DEPTH` levels deep compiles
    /// successfully at `-O max`.
    #[test]
    fn test_deeply_nested_expression_compiles_without_stack_overflow() {
        // A single global whose body is a `DEPTH`-deep chain of monadic binds.
        let mut source = String::from("module Main;\nmain : IO ();\nmain = (\n");
        for i in 0..DEPTH {
            source.push_str(&format!("    println(\"{}\");;\n", i));
        }
        source.push_str("    pure()\n);\n");

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output = fix_build_source_command(temp_dir.path(), &source, "max")
            .output()
            .expect("Failed to execute fix build");

        assert!(
            output.status.success(),
            "compiling a {}-deep expression failed (stack overflow?):\nstdout: {}\nstderr: {}",
            DEPTH,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    /// The nesting depth of the expression that is written to the type-check cache and read back.
    /// It is deeper than `DEPTH` because the read is the cheapest of the three recursions to reach:
    /// `-O none` keeps both compilations to a few seconds at this depth, where `-O max` would not.
    const CACHE_DEPTH: usize = 2000;

    /// When each entry of the type-check cache under `dir` was last written.
    ///
    /// A run that reads an entry returns what it holds and writes nothing, so an entry whose file
    /// is untouched by a second run is one that second run read.
    fn cache_entry_times(dir: &Path) -> Map<PathBuf, SystemTime> {
        fs::read_dir(dir.join(TYPE_CHECK_CACHE_PATH))
            .expect("the type-check cache directory should exist after a compilation")
            .map(|entry| {
                let entry = entry.expect("a cache entry should be readable");
                let written = entry
                    .metadata()
                    .expect("a cache entry should have metadata")
                    .modified()
                    .expect("a cache entry should carry a modification time");
                (entry.path(), written)
            })
            .collect()
    }

    /// Verifies that a `CACHE_DEPTH`-deep expression stored in the type-check cache is read back
    /// and gives the answer the run that stored it gave.
    ///
    /// The entry holds the typed expression as a tree, and reading it walks that tree to its full
    /// depth, so an expression this deep exercises a recursion no single build performs: the first
    /// compilation writes the entry, and only the second one reads it.
    ///
    /// An entry the second run fails to read costs it nothing but the check it could have skipped,
    /// so the value alone would say the same whether the entry was read or ignored. What tells the
    /// two apart is that a run which reads an entry leaves its file alone.
    #[test]
    fn test_deeply_nested_expression_survives_the_type_check_cache() {
        let mut source = String::from("module Main;\n\ndeep : I64;\ndeep = (\n    let x0 = 1;\n");
        let mut expected: i64 = 1;
        for i in 1..CACHE_DEPTH {
            let step = (i % 7) as i64;
            expected += step;
            source.push_str(&format!("    let x{} = x{} + {};\n", i, i - 1, step));
        }
        source.push_str(&format!(
            "    x{}\n);\n\nmain : IO ();\nmain = println(deep.to_string);\n",
            CACHE_DEPTH - 1
        ));

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let source_path = temp_dir.path().join("generated.fix");
        fs::write(&source_path, &source).expect("Failed to write the generated source file");

        // The compiler keeps its cache under the directory it runs in, so running twice there is
        // what makes the second run read what the first one wrote.
        let run = |pass: &str| {
            let output = fix_command_at_opt_level("run", "none")
                .arg("--file")
                .arg(&source_path)
                .current_dir(temp_dir.path())
                .output()
                .expect("Failed to execute fix run");
            assert!(
                output.status.success(),
                "the run that {} a {}-deep expression failed (stack overflow?):\nstdout: {}\nstderr: {}",
                pass,
                CACHE_DEPTH,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).trim(),
                expected.to_string(),
                "the run that {} a {}-deep expression printed the wrong value",
                pass,
                CACHE_DEPTH,
            );
        };

        run("stores the expression");
        let stored = cache_entry_times(temp_dir.path());
        assert!(
            !stored.is_empty(),
            "the run that stores a {}-deep expression left the type-check cache empty",
            CACHE_DEPTH,
        );
        run("reads the expression back");
        assert_eq!(
            cache_entry_times(temp_dir.path()),
            stored,
            "the second run rewrote cache entries a {}-deep expression should have let it read",
            CACHE_DEPTH,
        );
    }
}
