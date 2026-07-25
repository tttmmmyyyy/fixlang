// Reproduction test for a pre-existing bug, kept RED as a reminder to fix it. It asserts the
// behavior that should hold and currently fails on `main`; it turns green once the bug is fixed.

use crate::tests::test_util::fix_command;
use std::fs;
use tempfile::TempDir;

// Building a program that uses a recursive type with `-g` must succeed. It currently overflows the
// compiler's stack while emitting debug information: generating the `DIType` for the recursive type
// recurses without a cycle guard, so `fix build -g` aborts. `-g` is needed to reach the debug-info
// path; without it the same program builds.
#[test]
pub fn test_build_g_recursive_type_succeeds() {
    let source = r#"
        module Main;

        type Tree = box union { leaf : (), node : (Tree, Tree) };

        size : Tree -> I64;
        size = |t| match t {
            leaf(_) => 1,
            node(lr) => size(lr.@0) + size(lr.@1)
        };

        main : IO ();
        main = println(size(Tree::node $ (Tree::leaf(), Tree::leaf())).to_string);
    "#;
    let temp = TempDir::new().expect("Failed to create temp directory");
    fs::write(temp.path().join("main.fix"), source).expect("Failed to write main.fix");
    let build = fix_command()
        .args(["build", "-g", "-f", "main.fix", "-o", "prog"])
        .current_dir(temp.path())
        .output()
        .expect("Failed to execute `fix build`");
    assert!(
        build.status.success(),
        "`fix build -g` on a recursive type failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );
}
