use crate::tests::test_util::{
    emitted_llvm_ir, fix_command_at_opt_level, llvm_function_bodies, EmittedIr,
};
use std::fs;
use tempfile::TempDir;

/// A program whose global `table` has an initializer long enough that an inlining decided by size
/// would leave the accessor a call, and whose `main` reads `table` from a loop, where the
/// accessor's flag test and load are worth lifting out.
const LONG_INITIALIZER_SOURCE: &str = r#"
    module Main;

    // The characters the table's initializer reads.
    alphabet : Array U8;
    alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".get_bytes;

    // A table built by a fold over another global.
    table : Array I64;
    table = Iterator::range(0, 256).fold(Array::fill(256, 3), |i, entries|
        entries.set(alphabet.@(i.bit_and(63)).to_I64, 3)
    );

    main : IO ();
    main = (
        let total = Iterator::range(0, 1000).fold(0, |i, total| total + table.@(i.bit_and(255)));
        println(total.to_string)
    );
"#;

/// The names the compiler gives the parts of `table`, as the emitted LLVM IR quotes them.
const TABLE_ACCESSOR: &str = "@\"Get#Main::table#";
const TABLE_INITIALIZER: &str = "@\"InitValue#Main::table#";
const TABLE_STORAGE: &str = "@\"GlobalVar#Main::table#";
const TABLE_FLAG: &str = "@\"InitFlag#Main::table#";

/// Build `LONG_INITIALIZER_SOURCE` with `--emit-llvm` in a directory of its own, and return that
/// directory.
///
/// The build works at `-O max` whatever level the suite runs at, which compiles the program as one
/// compilation unit: the emitted IR is one module, holding the accessor, the initializer, the loop
/// reading the global, and one numbering of the attribute groups.
fn build_emitting_llvm_ir() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    let source_path = dir.join("generated.fix");
    fs::write(&source_path, LONG_INITIALIZER_SOURCE)
        .expect("Failed to write the generated source file");
    let build = fix_command_at_opt_level("build", "max")
        .arg("--file")
        .arg(&source_path)
        .arg("--emit-llvm")
        .current_dir(dir)
        .output()
        .expect("Failed to execute fix build");
    assert!(
        build.status.success(),
        "the build should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );
    temp_dir
}

/// The single body of the function whose name starts with `name`, out of `ir`.
fn sole_body(ir: &str, name: &str) -> String {
    let bodies = llvm_function_bodies(ir, name);
    assert_eq!(bodies.len(), 1, "the build should emit one `{}`", name);
    bodies[0].clone()
}

/// The initializer of a global is a function of its own, which is left there.
///
/// Reading a global tests an initialization flag and loads the storage — four instructions on
/// x86-64. An accessor holding the initializer as well is as large as the initializer, and an
/// inlining decided by size leaves it a call: one per read, with the flag test and the load stuck
/// in the loop behind it. An accessor whose initializer is elsewhere is small enough to be placed
/// at every reader without being asked.
///
/// The initializer runs once in the program's life, so it belongs at no call site, and the accessor
/// is its only caller — an inliner reaching it would fold it straight back in.
#[test]
pub fn test_the_initializer_of_a_global_sits_outside_the_accessor() {
    let temp_dir = build_emitting_llvm_ir();
    let ir = emitted_llvm_ir(temp_dir.path(), EmittedIr::BeforeOptimization);

    let accessor = sole_body(&ir, TABLE_ACCESSOR);
    assert!(
        accessor.contains(TABLE_INITIALIZER),
        "the accessor should call the initializer, and its body is:\n{}",
        accessor
    );

    let signature = sole_body(&ir, TABLE_INITIALIZER)
        .lines()
        .next()
        .expect("a function body starts with its signature")
        .to_string();
    let group = signature
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .filter(|group| group.starts_with('#'))
        .unwrap_or_else(|| {
            panic!(
                "the initializer should carry attributes, and its signature is `{}`",
                signature
            )
        });
    let group_line = format!("attributes {} =", group);
    let attributes = ir
        .lines()
        .find(|line| line.starts_with(&group_line))
        .unwrap_or_else(|| panic!("the emitted IR should define `{}`", group_line));
    assert!(
        attributes.contains("noinline"),
        "the initializer should stay out of its caller, and its attributes are `{}`",
        attributes
    );
}

/// A reader of a global sees every write to the global's storage and flag.
///
/// This is what lets a reader lift its reads out of a loop. The reads are of two module-level
/// variables, and the call to the initializer sits between them in the reader's loop: a reader that
/// could not see the writes would have to assume that call performs them, and read the flag and the
/// storage again on every turn.
#[test]
pub fn test_a_reader_of_a_global_sees_every_write_to_it() {
    let temp_dir = build_emitting_llvm_ir();
    let ir = emitted_llvm_ir(temp_dir.path(), EmittedIr::BeforeOptimization);
    let accessor = sole_body(&ir, TABLE_ACCESSOR);

    for variable in [TABLE_STORAGE, TABLE_FLAG] {
        let writes = |text: &str| {
            text.lines()
                .filter(|line| line.trim_start().starts_with("store") && line.contains(variable))
                .count()
        };
        let in_module = writes(&ir);
        assert!(in_module > 0, "the program should write `{}`", variable);
        assert_eq!(
            writes(&accessor),
            in_module,
            "every write to `{}` should be in the accessor, and {} of the {} are:\n{}",
            variable,
            writes(&accessor),
            in_module,
            accessor
        );
    }
}

/// A global read inside a loop is read without a call.
///
/// A read of `table` goes through its accessor, twice per element: the bounds check reads the
/// length and the element read reads the pointer. The property is read off the emitted LLVM IR: it
/// is about the code the build emits, and a program cannot observe a call it does not make.
///
/// This is the requirement. `test_the_initializer_of_a_global_sits_outside_the_accessor` and
/// `test_a_reader_of_a_global_sees_every_write_to_it` pin the two properties this compiler reaches
/// it by, and another mechanism would keep this test green and turn those red.
#[test]
pub fn test_reading_a_global_in_a_loop_costs_no_call() {
    let temp_dir = build_emitting_llvm_ir();
    let dir = temp_dir.path();
    let calls_to_accessor = |ir: &str| {
        ir.lines()
            .filter(|line| line.contains("call") && line.contains(TABLE_ACCESSOR))
            .map(str::to_string)
            .collect::<Vec<_>>()
    };

    // The generated IR holds the calls the optimized IR is then checked for: a program reaching
    // `table` some other way would satisfy that check for free.
    let generated_ir = emitted_llvm_ir(dir, EmittedIr::BeforeOptimization);
    assert!(
        !calls_to_accessor(&generated_ir).is_empty(),
        "the program should read `table` through its accessor"
    );

    let remaining_calls = calls_to_accessor(&emitted_llvm_ir(dir, EmittedIr::AfterOptimization));
    assert!(
        remaining_calls.is_empty(),
        "reading `table` should cost no call, and the optimized IR holds {}:\n{}",
        remaining_calls.len(),
        remaining_calls.join("\n")
    );
}
