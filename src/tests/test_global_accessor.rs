use crate::tests::test_util::{
    emitted_llvm_ir, fix_command_at_opt_level, llvm_function_bodies, EmittedIr,
};
use std::fs;
use tempfile::TempDir;

/// How many times `widely_read` is read, which is more readers than an accessor is placed at (see
/// `READERS_TO_PLACE_THE_ACCESSOR_AT`).
const WIDE_READ_COUNT: usize = 70;

/// A program with two globals whose accessors the compiler decides differently about.
///
/// `table` has an initializer long enough that an inlining decided by size would leave the accessor
/// a call, and `main` reads it from a loop, where the accessor's flag test and load are worth
/// lifting out. `widely_read` is read from more places than an accessor is placed at.
fn program_source() -> String {
    let wide_reads = vec!["widely_read"; WIDE_READ_COUNT].join(" + ");
    format!(
        r#"
        module Main;

        // The characters the table's initializer reads.
        alphabet : Array U8;
        alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".get_bytes;

        // A table built by a fold over another global.
        table : Array I64;
        table = Iterator::range(0, 256).fold(Array::fill(256, 3), |i, entries|
            entries.set(alphabet.@(i.bit_and(63)).to_I64, 3)
        );

        widely_read : I64;
        widely_read = Iterator::range(0, 256).fold(0, |i, acc| acc + i * 3);

        sum_widely_read : I64;
        sum_widely_read = {wide_reads};

        main : IO ();
        main = (
            let total = Iterator::range(0, 1000).fold(0, |i, total| total + table.@(i.bit_and(255)));
            println((total + sum_widely_read).to_string)
        );
    "#
    )
}

/// The name `global_accessor_name` gives the accessor of `table`, as the emitted LLVM IR quotes it.
const TABLE_ACCESSOR: &str = "@\"Get#Main::table#";

/// The same, for `widely_read`.
const WIDELY_READ_ACCESSOR: &str = "@\"Get#Main::widely_read#";

/// Build `program_source()` with `--emit-llvm` in a directory of its own, and return that directory.
///
/// The build works at `-O max` whatever level the suite runs at, which compiles the program as one
/// compilation unit: the emitted IR is one module, holding the accessors, the loop reading one of
/// them, and one numbering of the attribute groups.
fn build_emitting_llvm_ir() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    let source_path = dir.join("generated.fix");
    fs::write(&source_path, program_source()).expect("Failed to write the generated source file");
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

/// Whether the accessor named by `accessor` asks to be placed at its readers, read off `ir`.
fn asks_to_be_placed_at_its_readers(ir: &str, accessor: &str) -> bool {
    let bodies = llvm_function_bodies(ir, accessor);
    assert_eq!(bodies.len(), 1, "the build should emit one `{}`", accessor);
    let signature = bodies[0]
        .lines()
        .next()
        .expect("a function body starts with its signature");
    let Some(group) = signature
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .filter(|group| group.starts_with('#'))
    else {
        return false;
    };
    let group_line = format!("attributes {} =", group);
    let attributes = ir
        .lines()
        .find(|line| line.starts_with(&group_line))
        .unwrap_or_else(|| panic!("the emitted IR should define `{}`", group_line));
    attributes.contains("alwaysinline")
}

/// The accessor that reads a global asks to be inlined into every reader of the global.
///
/// Reading a global tests an initialization flag and loads the stored value, and the accessor holds
/// the initializer as well. A reader with the test and the load in front of it lifts them out of a
/// loop over the global; behind a call they stay in the loop, and every read costs a call as well.
/// An ordinary inlining decides by the accessor's size, which the initializer dominates, so the
/// accessor asks for the inlining whatever that size is.
///
/// This test reads the request, which the compiler decides.
/// `test_reading_a_global_in_a_loop_costs_no_call` reads what LLVM makes of the request, which the
/// surrounding code has a say in as well.
#[test]
pub fn test_the_accessor_of_a_global_asks_to_be_placed_at_its_readers() {
    let temp_dir = build_emitting_llvm_ir();
    let ir = emitted_llvm_ir(temp_dir.path(), EmittedIr::BeforeOptimization);
    assert!(
        asks_to_be_placed_at_its_readers(&ir, TABLE_ACCESSOR),
        "the accessor of `table` should ask to be placed at its readers"
    );
}

/// A global read from many places keeps its accessor to itself.
///
/// The request copies the accessor, and the initializer it holds, into every reader, so it is made
/// only where the readers are few. Without the bound a global read from all over a program carries
/// its initializer to all of those places, which costs object size and compilation time for reads
/// that gain little: what the request buys is the lifting of a flag test and a load out of a loop,
/// and a loop is one reader.
#[test]
pub fn test_a_global_with_many_readers_keeps_its_accessor_to_itself() {
    let temp_dir = build_emitting_llvm_ir();
    let ir = emitted_llvm_ir(temp_dir.path(), EmittedIr::BeforeOptimization);

    // The reads the bound is counting: a program reaching `widely_read` some other way, or one
    // whose reads were folded together, would satisfy the check below for free.
    let reads = ir
        .lines()
        .filter(|line| line.contains("call") && line.contains(WIDELY_READ_ACCESSOR))
        .count();
    assert_eq!(
        reads, WIDE_READ_COUNT,
        "the program should read `widely_read` {} times",
        WIDE_READ_COUNT
    );

    assert!(
        !asks_to_be_placed_at_its_readers(&ir, WIDELY_READ_ACCESSOR),
        "the accessor of `widely_read` should not ask to be placed at {} readers",
        reads
    );
}

/// A global read inside a loop is read without a call.
///
/// A read of `table` goes through its accessor, twice per element: the bounds check reads the
/// length and the element read reads the pointer. The accessor's size is its initializer's, which
/// is large enough for an ordinary inlining to leave the accessor a call — one per read, and the
/// flag test and the load stay in the loop with it. The property is read off the emitted LLVM IR:
/// it is about the code the build emits, and a program cannot observe a call it does not make.
///
/// This is the requirement. `test_the_accessor_of_a_global_asks_to_be_placed_at_its_readers` pins
/// the mechanism this compiler reaches it by, and another mechanism would keep this test green and
/// turn that one red.
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
