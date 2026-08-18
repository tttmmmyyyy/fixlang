use crate::env_vars::MAX_OPT_LEVEL_VAR;
use crate::tests::test_util::{
    emitted_llvm_ir, fix_build_source_command, llvm_function_bodies, EmittedIr,
};
use tempfile::TempDir;

/// A program with two globals the compiler decides differently about.
///
/// `table` is read from a loop, so its accessor is shared and its initializer is long enough that
/// an inlining decided by size would leave the accessor a call. `read_once` is read from one place.
const TWO_GLOBALS_SOURCE: &str = r#"
    module Main;

    // The characters the table's initializer reads.
    alphabet : Array U8;
    alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".get_bytes;

    // A table built by a fold over another global.
    table : Array I64;
    table = Iterator::range(0, 256).fold(Array::fill(256, 3), |i, entries|
        entries.set(alphabet.@(i.bit_and(63)).to_I64, 3)
    );

    read_once : I64;
    read_once = Iterator::range(0, 256).fold(0, |i, acc| acc + i * 3);

    main : IO ();
    main = (
        let total = Iterator::range(0, 1000).fold(0, |i, total| total + table.@(i.bit_and(255)));
        println((total + read_once).to_string)
    );
"#;

/// The names the compiler gives the parts of `table`, as the emitted LLVM IR quotes them.
const TABLE_ACCESSOR: &str = "@\"Get#Main::table#";
const TABLE_INITIALIZER: &str = "@\"InitValue#Main::table#";
const TABLE_STORAGE: &str = "@\"GlobalVar#Main::table#";
const TABLE_FLAG: &str = "@\"InitFlag#Main::table#";

/// The same, for `read_once`.
const READ_ONCE_ACCESSOR: &str = "@\"Get#Main::read_once#";
const READ_ONCE_INITIALIZER: &str = "@\"InitValue#Main::read_once#";

/// A program whose global is read by the C function an `FFI_EXPORT` statement builds.
///
/// `counter` is read by `main` and by the exported C function. The exported function is emitted
/// after every symbol of the program is, so its read arrives after the rest.
const EXPORTED_GLOBAL_SOURCE: &str = r#"
    module Main;

    counter : I64;
    counter = Iterator::range(0, 256).fold(0, |i, acc| acc + i * 3);
    FFI_EXPORT[counter, c_counter];

    main : IO ();
    main = println(counter.to_string);
"#;

/// The names the compiler gives the parts of `counter`, as the emitted LLVM IR quotes them.
const COUNTER_ACCESSOR: &str = "@\"Get#Main::counter#";
const COUNTER_INITIALIZER: &str = "@\"InitValue#Main::counter#";

/// Build `TWO_GLOBALS_SOURCE` with `--emit-llvm` in a directory of its own, and return that
/// directory.
///
/// The build works at `-O max` whatever level the suite runs at, which compiles the program as one
/// compilation unit: the emitted IR is one module, holding the accessors, the initializers, the
/// loop reading a global, and one numbering of the attribute groups.
fn build_emitting_llvm_ir(source: &str) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    let build = fix_build_source_command(dir, source, "max")
        .env(MAX_OPT_LEVEL_VAR, "max")
        .arg("--emit-llvm")
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

/// Whether the function whose name starts with `name` asks to be left out of its callers.
fn stays_out_of_its_callers(ir: &str, name: &str) -> bool {
    let signature = sole_body(ir, name)
        .lines()
        .next()
        .expect("a function body starts with its signature")
        .to_string();
    let Some(group) = signature
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .filter(|group| group.starts_with('#'))
    else {
        // A function carrying no attribute at all is printed without an attribute group, and asks
        // for nothing — `noinline` included.
        return false;
    };
    let group_line = format!("attributes {} =", group);
    let attributes = ir
        .lines()
        .find(|line| line.starts_with(&group_line))
        .unwrap_or_else(|| panic!("the emitted IR should define `{}`", group_line));
    attributes.contains("noinline")
}

/// How many times `ir` calls the function whose name starts with `name`.
fn calls_to(ir: &str, name: &str) -> usize {
    ir.lines()
        .filter(|line| line.contains("call") && line.contains(name))
        .count()
}

/// The initializer of a global read from many places is a function of its own, which is left there.
///
/// Reading a global tests an initialization flag and loads the storage — four instructions on
/// x86-64. An accessor holding the initializer as well is as large as the initializer, and an
/// inlining decided by size leaves it a call: one per read, with the flag test and the load stuck
/// in the loop behind it. An accessor whose initializer is elsewhere is small enough to be placed
/// at every reader without being asked.
///
/// The accessor is the initializer's only caller, so an inliner reaching the initializer would fold
/// it straight back in.
#[test]
pub fn test_the_initializer_of_a_shared_global_sits_outside_the_accessor() {
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
    let ir = emitted_llvm_ir(temp_dir.path(), EmittedIr::BeforeOptimization);

    // The readers the decision rests on.
    assert!(
        calls_to(&ir, TABLE_ACCESSOR) > 1,
        "the program should read `table` from more than one place"
    );

    let accessor = sole_body(&ir, TABLE_ACCESSOR);
    assert!(
        accessor.contains(TABLE_INITIALIZER),
        "the accessor should call the initializer, and its body is:\n{}",
        accessor
    );
    assert!(
        stays_out_of_its_callers(&ir, TABLE_INITIALIZER),
        "the initializer of `table` should stay out of the accessor"
    );
}

/// The initializer of a global read from one place is left in the accessor.
///
/// It has one place to be either way, and the place that costs nothing is the one the reader can
/// see: what the initializer knows — the length of an array, the shape of a structure — reaches the
/// code that reads the global, where it takes bounds checks out of loops.
#[test]
pub fn test_the_initializer_of_a_global_read_once_stays_where_its_reader_sees_it() {
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
    let ir = emitted_llvm_ir(temp_dir.path(), EmittedIr::BeforeOptimization);

    // The reader the decision rests on.
    assert_eq!(
        calls_to(&ir, READ_ONCE_ACCESSOR),
        1,
        "the program should read `read_once` from one place"
    );

    assert!(
        !stays_out_of_its_callers(&ir, READ_ONCE_INITIALIZER),
        "the initializer of `read_once` should be free to join the accessor"
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
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
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
/// This is the requirement. `test_the_initializer_of_a_shared_global_sits_outside_the_accessor` and
/// `test_a_reader_of_a_global_sees_every_write_to_it` pin the two properties this compiler reaches
/// it by, and another mechanism would keep this test green and turn those red.
#[test]
pub fn test_reading_a_global_in_a_loop_costs_no_call() {
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
    let dir = temp_dir.path();

    // The generated IR holds the calls the optimized IR is then checked for: a program reaching
    // `table` some other way would satisfy that check for free.
    let generated_ir = emitted_llvm_ir(dir, EmittedIr::BeforeOptimization);
    assert!(
        calls_to(&generated_ir, TABLE_ACCESSOR) > 0,
        "the program should read `table` through its accessor"
    );

    let optimized_ir = emitted_llvm_ir(dir, EmittedIr::AfterOptimization);
    let remaining_calls: Vec<_> = optimized_ir
        .lines()
        .filter(|line| line.contains("call") && line.contains(TABLE_ACCESSOR))
        .collect();
    assert!(
        remaining_calls.is_empty(),
        "reading `table` should cost no call, and the optimized IR holds {}:\n{}",
        remaining_calls.len(),
        remaining_calls.join("\n")
    );
}

/// A global read by an exported C function has that reader counted.
///
/// The C functions an `FFI_EXPORT` statement builds, and the program's entry point, are emitted
/// after the program's symbols are, and each reads the value it exports through that value's
/// accessor. Which accessors keep their initializer is decided once those readers are in the
/// module: decided before them, a global read from there is counted short, and every reader of it
/// pays a call.
#[test]
pub fn test_a_global_read_from_an_exported_c_function_has_that_reader_counted() {
    let temp_dir = build_emitting_llvm_ir(EXPORTED_GLOBAL_SOURCE);
    let ir = emitted_llvm_ir(temp_dir.path(), EmittedIr::BeforeOptimization);

    // The readers the decision rests on: `main`, and the exported C function.
    assert_eq!(
        calls_to(&ir, COUNTER_ACCESSOR),
        2,
        "the program should read `counter` from `main` and from the exported C function"
    );

    assert!(
        stays_out_of_its_callers(&ir, COUNTER_INITIALIZER),
        "the initializer of `counter` should stay out of the accessor"
    );
}
