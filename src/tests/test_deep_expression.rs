// The compiler compiles a deeply nested expression without overflowing its own
// stack. Type checking and code generation recurse to the expression's nesting
// depth; they run on worker threads sized for that recursion via
// `COMPILER_THREAD_STACK_SIZE`, and the whole compilation runs on a
// thread of the same size, so a deep `let`/`;;` chain compiles rather than
// aborting with a stack overflow.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_build_source_command;
    use tempfile::TempDir;

    // 300 exceeds the few-hundred nesting depth at which the compiler overflows a
    // worker thread without the enlarged stack, so this build fails if the fix
    // regresses; it stays shallow enough to keep the `-O max` build time modest.
    const DEPTH: usize = 300;

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
}
