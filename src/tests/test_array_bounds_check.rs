use crate::{
    configuration::{Configuration, FixOptimizationLevel},
    fixstd::runtime::{RUNTIME_ARRAY_SIZE_OVERFLOW, RUNTIME_MALLOC},
    tests::test_util::{
        emitted_llvm_ir, fix_build_source_command, llvm_function_bodies, test_source,
        test_source_fail, EmittedIr,
    },
};
use tempfile::TempDir;

/// A configuration whose runtime checks are on, which is what makes the checks under test part of
/// the emitted program.
fn runtime_checked_config() -> Configuration {
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    config
}

/// A configuration whose runtime checks are off, as `--no-runtime-check` leaves them.
fn runtime_unchecked_config() -> Configuration {
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = true;
    config
}

/// `@` bounds-checks the index it is given, and the message reports both the index and the size.
#[test]
pub fn test_get() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].@(3);
                pure()
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(&source, config, "Index out of range: index=3, size=3");
}

/// `set` bounds-checks the index it is given.
#[test]
pub fn test_set() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].set(3, 42);
                pure()
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(&source, config, "Index out of range");
}

/// `mod` bounds-checks the index it is given.
#[test]
pub fn test_mod() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].mod(3, |_| 42);
                pure()
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(&source, config, "Index out of range");
}

/// `act` bounds-checks the index it is given.
#[test]
pub fn test_act() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].act(3, |_| some(42));
                pure()
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(&source, config, "Index out of range");
}

/// The `arr[idx]` index syntax bounds-checks the index as `@` does.
#[test]
pub fn test_index_syntax() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2][3].iget;
                pure()
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(&source, config, "Index out of range");
}

/// A negative capacity given to `Array::empty` aborts the program, and the message reports the
/// capacity.
#[test]
pub fn test_empty_negative_capacity() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval Array::empty(-1) : Array I64; 
                pure()
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(&source, config, "Negative array size or capacity: -1");
}

/// A negative size given to `Array::fill` aborts the program.
#[test]
pub fn test_fill_negative_size() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                eval Array::fill(-1, 42);
                pure()
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(&source, config, "Negative array size or capacity");
}

// The bytes an array's elements need must fit in the address space. A capacity whose byte count
// exceeds it wraps around to a small number, which `malloc` supplies, leaving an array whose block
// has no room for the capacity it claims; the first write past the block corrupts the heap.

/// A capacity of 2^61 elements of 8 bytes comes to exactly 2^64 bytes, which wraps to zero.
#[test]
pub fn test_empty_capacity_byte_count_wraps() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(2305843009213693952);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 2305843009213693952",
    );
}

/// A capacity of 2^61-3 elements of 8 bytes comes to 24 bytes short of 2^64: the elements fit, while
/// the header and the padding that puts the element buffer on its alignment do not.
#[test]
pub fn test_empty_capacity_byte_count_leaves_no_room_for_the_header() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(2305843009213693949);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 2305843009213693949",
    );
}

/// A capacity the compiler cannot see the value of is checked where the program allocates, rather
/// than by folding the check away at the capacity a literal gives.
#[test]
pub fn test_empty_capacity_byte_count_is_checked_at_run_time() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let args = *get_args;
                // A program run with no argument gets one element in `args`, so this is 2^61-3.
                let arr : Array I64 = Array::empty(2305843009213693950 - args.@size);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 2305843009213693949",
    );
}

/// Growing an array's capacity reallocates its storage, and the byte count of the new capacity is
/// checked as the one an array is created with is.
#[test]
pub fn test_reserve_capacity_byte_count_wraps() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = [1, 2, 3];
                println(arr.reserve(2305843009213693952).@(0).to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 2305843009213693952",
    );
}

/// The bound scales with the element stride: 2^60 elements of 32 bytes overflow where the same
/// capacity of 8-byte elements is far below the bound.
#[test]
pub fn test_capacity_byte_count_wraps_for_a_wide_element() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array (I64, I64, I64, I64) = Array::empty(1152921504606846976);
                println(arr.push_back((1, 2, 3, 4)).@size.to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 1152921504606846976",
    );
}

/// An element type of no size gives a buffer of no bytes, whatever the capacity, so any capacity is
/// allocatable and the check has no bound to compare against.
#[test]
pub fn test_capacity_of_a_zero_sized_element_is_allocatable() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array () = Array::fill(3, ());
                println(arr.@size.to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source(&source, config);
}

/// A capacity reaches the check through the `_unsafe_` primitives as well, which take one the caller
/// promises is in range. Read as the byte count it asks for, a negative capacity of an eight-byte
/// element exceeds the address space.
#[test]
pub fn test_unsafe_empty_capacity_unchecked_checks_the_byte_count() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::_unsafe_empty_capacity_unchecked(-1);
                println(arr.@capacity.to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: -1",
    );
}

/// The bound is exact at its top end: one element above the widest capacity that fits, the byte
/// count plus the header and the alignment padding comes to 2^64 + 7, which wraps to 7 bytes that
/// `malloc` supplies.
#[test]
pub fn test_empty_capacity_one_element_past_the_bound_is_rejected() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(2305843009213693948);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 2305843009213693948",
    );
}

/// Growing a shared array allocates a fresh storage instead of reallocating in place, and the byte
/// count of the new capacity is checked on that branch as it is on the in-place one.
#[test]
pub fn test_reserve_capacity_byte_count_wraps_for_a_shared_array() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::fill(3, 7);
                // Two live references to one storage send `reserve` down the shared branch.
                let pair = (arr, arr);
                let big = pair.@0.reserve(2305843009213693952);
                println((big.@(0) + pair.@1.@(0)).to_string)
            );
        "#;
    let config = runtime_checked_config();
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 2305843009213693952",
    );
}

/// The runtime function the check calls is declared in every compilation unit and defined in one, so
/// a program split into one unit per symbol links and aborts where a program of one unit does.
#[test]
pub fn test_capacity_byte_count_under_separate_compilation() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let args = *get_args;
                // A program run with no argument gets one element in `args`, so this is 2^61-3.
                let arr : Array I64 = Array::empty(2305843009213693950 - args.@size);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let mut config = runtime_checked_config();
    config.set_fix_opt_level(FixOptimizationLevel::Basic);
    config.cu_size = 1;
    test_source_fail(
        &source,
        config,
        "Array size or capacity exceeds the address space: 2305843009213693949",
    );
}

/// `--no-runtime-check` drops the byte-count check with the bounds checks, so the same capacity that
/// aborts with checks on runs to completion with them off.
#[test]
pub fn test_capacity_byte_count_respects_no_runtime_check() {
    // The array is never written to, so the run with the check off stays memory-safe: the wrapped
    // byte count reaches `malloc`, and the array it returns is only asked for its capacity.
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(2305843009213693952);
                println(arr.@capacity.to_string)
            );
        "#;
    let checked = runtime_checked_config();
    test_source_fail(
        &source,
        checked,
        "Array size or capacity exceeds the address space: 2305843009213693952",
    );
    let unchecked = runtime_unchecked_config();
    test_source(&source, unchecked);
}

// `--no-runtime-check` disables array bounds checks (documented in the CLI help), so `set`
// and `swap` must honor it like `@` / `mod` / `act`. An index within the array's capacity but
// past its size stays inside the allocated buffer, so the access itself is memory-safe; only
// the bounds check decides whether it aborts. Such an access therefore runs to completion
// exactly when the check is gone, and that is what the tests below read off.

/// `set` honors `--no-runtime-check`: one index, in capacity and past the size, aborts with the
/// checks on and completes with them off.
#[test]
pub fn test_set_bounds_check_respects_no_runtime_check() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(10); // size 0, capacity 10
                eval arr.set(5, 42);                    // in capacity, past size
                pure()
            );
        "#;
    // Checks on: the out-of-range index aborts.
    let checked = runtime_checked_config();
    test_source_fail(&source, checked, "Index out of range");
    // Checks off: no bounds check, so the (memory-safe) access completes.
    let unchecked = runtime_unchecked_config();
    test_source(&source, unchecked);
}

/// `swap` honors `--no-runtime-check` for both of the indices it checks.
#[test]
pub fn test_swap_bounds_check_respects_no_runtime_check() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(10); // size 0, capacity 10
                eval arr.swap(3, 5);                    // both in capacity, past size
                pure()
            );
        "#;
    let checked = runtime_checked_config();
    test_source_fail(&source, checked, "Index out of range");
    let unchecked = runtime_unchecked_config();
    test_source(&source, unchecked);
}

/// `unsafe_swap_bounds_unchecked` skips the bounds check even with the runtime checks on, so a pair
/// of indices in capacity and past the size runs to completion.
#[test]
pub fn test_unsafe_swap_bounds_unchecked_skips_check() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(10); // size 0, capacity 10
                eval arr.unsafe_swap_bounds_unchecked(3, 5);   // in capacity, past size
                pure()
            );
        "#;
    let checked = runtime_checked_config();
    test_source(&source, checked);
}

/// A program that writes into an array while a second handle to it is alive, so that the build emits
/// `Array::set` with both answers of its uniqueness check -- including the clone the shared answer
/// takes.
const SHARED_WRITE_SOURCE: &str = r#"
module Main;

main : IO ();
main = (
    let arr = Array::fill(4, 0);
    let shared = arr;
    let arr = arr.set(0, 7);
    println((arr.@(0) + shared.@(0)).to_string)
);
"#;

/// Writing into a shared array clones its storage with the capacity that array already holds, and
/// the clone allocates without checking that capacity: the allocation that gave the array its
/// capacity checked it.
///
/// The check is a comparison and a branch, and an array write is the innermost thing a loop over an
/// array does, so a check emitted here is what costs the enclosing loop its unrolling. The property
/// is read off the emitted LLVM IR: it is about the code the build emits, and a program cannot
/// observe a check that never fires.
#[test]
pub fn test_array_write_clones_without_rechecking_the_capacity() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    // At `-O none` the array write keeps its uniqueness check, so the clone is in the emitted code
    // whatever the program does at run time.
    let build = fix_build_source_command(dir, SHARED_WRITE_SOURCE, "none")
        .arg("--emit-llvm")
        .output()
        .expect("Failed to execute fix build");
    assert!(
        build.status.success(),
        "the build should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );

    let ir = emitted_llvm_ir(dir, EmittedIr::BeforeOptimization);
    let overflow_call = format!("call void @{}(", RUNTIME_ARRAY_SIZE_OVERFLOW);
    assert!(
        ir.contains(&overflow_call),
        "the build should check the capacity where the program chooses one"
    );

    let write_fn_bodies = llvm_function_bodies(&ir, "@\"Std::Array::set#");
    assert!(
        !write_fn_bodies.is_empty(),
        "the build should emit `Std::Array::set`"
    );
    let allocating_bodies = write_fn_bodies
        .iter()
        .filter(|body| body.contains(&format!("@{}(", RUNTIME_MALLOC)))
        .collect::<Vec<_>>();
    assert!(
        !allocating_bodies.is_empty(),
        "`Std::Array::set` should allocate, which is the clone the shared answer takes"
    );
    let rechecking_bodies = allocating_bodies
        .iter()
        .filter(|body| body.contains(&overflow_call))
        .collect::<Vec<_>>();
    assert!(
        rechecking_bodies.is_empty(),
        "the clone in `Std::Array::set` should allocate without checking the capacity again, \
         but {} of its functions do",
        rechecking_bodies.len()
    );
}
