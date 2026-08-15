//! A module that no other module imports must not change the compiled program. Nothing imports
//! it, so no existing module's `module_dependency_hash` changes and every pre-existing global
//! value is served from the type-check cache on the second build — while the trait environment,
//! the type-constructor set and the set of global values have all grown underneath it.
//!
//! A tuple in such a module is the exception that decides how far the hash reaches. The compiler
//! implements the traits a tuple carries by generating a source for the sizes the program uses and
//! linking it into `Std`, so a module using a size nothing else uses rewrites a source every module
//! depends on.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{emitted_llvm_ir, fix_command, EmittedIr};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const FIXPROJ_WITHOUT_INTRUDER: &str = r#"[general]
name = "unrelated-module"
version = "0.1.0"
[build]
files = ["main.fix"]
"#;

    const FIXPROJ_WITH_INTRUDER: &str = r#"[general]
name = "unrelated-module"
version = "0.1.0"
[build]
files = ["main.fix", "intruder.fix"]
"#;

    const MAIN_FIX: &str = r#"module Main;

type MainData = struct { x : I64 };

impl MainData : ToString {
    to_string = |d| d.@x.to_string;
}

main : IO ();
main = println((1, 2, 3, 4, 5).to_string + MainData { x : 7 }.to_string);
"#;

    /// Declares a trait, a type and three instances, and defines a global value. No module
    /// imports it.
    const INTRUDER_FIX: &str = r#"module Intruder;

type IntruderT = struct { v : I64 };

trait a : IntruderTrait {
    itr : a -> I64;
}

impl IntruderT : IntruderTrait {
    itr = |x| x.@v;
}

impl IntruderT : ToString {
    to_string = |x| x.@v.to_string;
}

impl IntruderT : Eq {
    eq = |x, y| x.@v == y.@v;
}

intruder_value : I64;
intruder_value = IntruderT { v : 1 }.itr;
"#;

    /// Uses a tuple of a size no other module uses, and nothing else. No module imports it.
    const INTRUDER_WITH_A_TUPLE_FIX: &str = r#"module Intruder;

intruder_tuple : (I64, I64, I64, I64);
intruder_tuple = (1, 2, 3, 4);
"#;

    fn write_project(dir: &Path, fixproj: &str, intruder: Option<&str>) {
        fs::write(dir.join("fixproj.toml"), fixproj).unwrap();
        fs::write(dir.join("main.fix"), MAIN_FIX).unwrap();
        if let Some(intruder) = intruder {
            fs::write(dir.join("intruder.fix"), intruder).unwrap();
        }
    }

    /// `-g` so that source spans reach the emitted IR as debug info: a stale cached typed
    /// expression shows up there as a stale `DIFile` or line number.
    fn build(dir: &Path) {
        let out = fix_command()
            .args(["build", "-g", "--emit-llvm", "-o", "out"])
            .current_dir(dir)
            .output()
            .expect("failed to run fix build");
        assert!(
            out.status.success(),
            "fix build failed in {}:\n{}",
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn remove_emitted_ir(dir: &Path) {
        for entry in fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
            if entry.path().extension().is_some_and(|e| e == "ll") {
                fs::remove_file(entry.path()).unwrap();
            }
        }
    }

    /// The program compiled with `intruder` present from the first build, and the program compiled
    /// by adding `intruder` to a project whose type-check cache is already warm without it, as the
    /// LLVM IR of each.
    fn ir_of_a_cold_and_a_warm_build(intruder: &str) -> (String, String) {
        // Both builds run at the same absolute path: the module identifier the compiler stamps is
        // derived from the build path.
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("proj");

        // Cold: the intruder is present from the first build.
        fs::create_dir_all(&dir).unwrap();
        write_project(&dir, FIXPROJ_WITH_INTRUDER, Some(intruder));
        build(&dir);
        let cold_ir = emitted_llvm_ir(&dir, EmittedIr::All);

        // Warm: build without the intruder first, so every global value of `Main` and `Std` lands
        // in the type-check cache; then add the intruder and build again. Dropping the object
        // files forces code generation to run again while the type-check cache stays warm.
        fs::remove_dir_all(&dir).unwrap();
        fs::create_dir_all(&dir).unwrap();
        write_project(&dir, FIXPROJ_WITHOUT_INTRUDER, None);
        build(&dir);
        write_project(&dir, FIXPROJ_WITH_INTRUDER, Some(intruder));
        remove_emitted_ir(&dir);
        fs::remove_dir_all(dir.join(".fixlang/intermediate")).ok();
        build(&dir);
        let warm_ir = emitted_llvm_ir(&dir, EmittedIr::All);

        (cold_ir, warm_ir)
    }

    #[test]
    fn unrelated_module_does_not_change_the_compiled_program() {
        let (cold_ir, warm_ir) = ir_of_a_cold_and_a_warm_build(INTRUDER_FIX);
        assert_eq!(
            cold_ir, warm_ir,
            "a module that nothing imports changed the emitted program between a cold and a warm \
             type-check cache"
        );
    }

    /// The tuple in the intruder gives `Std` a source it did not have, holding the implementations
    /// that tuple carries. Every module depends on `Std`, so the entries the first build wrote
    /// belong to a `Std` made of other sources, and a build that serves them attributes the tuple
    /// implementations it shares to a source of the build before.
    #[test]
    fn unrelated_module_using_a_new_tuple_size_does_not_change_the_compiled_program() {
        let (cold_ir, warm_ir) = ir_of_a_cold_and_a_warm_build(INTRUDER_WITH_A_TUPLE_FIX);
        assert_eq!(
            cold_ir, warm_ir,
            "a module that nothing imports, using a tuple size nothing else uses, changed the \
             emitted program between a cold and a warm type-check cache"
        );
    }
}
