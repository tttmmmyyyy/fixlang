use crate::tests::test_util::{emitted_llvm_ir, fix_build_source_command, EmittedIr};
use tempfile::TempDir;

/// A program that reaches an array's elements every way the compiler computes a pointer into one:
/// reading a slot, writing a slot in place, writing one of a shared array (which clones the
/// buffer), growing an array, and taking a range out of one.
const ARRAY_ACCESS_SOURCE: &str = r#"
    module Main;

    total : Array I64 -> I64;
    total = |arr| Iterator::range(0, arr.get_size).fold(0, |i, acc| acc + arr.@(i));

    main : IO ();
    main = (
        let arr = Array::from_map(8, |i| i);
        let shared = arr;
        let written = arr.set(3, 100);
        let grown = written.push_back(9);
        let taken = grown.get_sub(1, 5);
        println $ (total(shared) + total(written) + total(grown) + total(taken)).to_string
    );
"#;

/// Every pointer the compiler computes into an object is computed within that object's allocation,
/// and the generated code says so.
///
/// A `getelementptr` without `inbounds` is one LLVM has to assume may leave the allocation it
/// started in. It then keeps the address arithmetic it would otherwise fold into an addressing
/// mode, and it cannot bound an index that a loop's bounds check reads, which is what decides
/// whether that loop has a trip count it can unroll by.
#[test]
pub fn test_every_pointer_into_an_object_is_computed_inside_it() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    // The property is about what the compiler emits, so it is read before LLVM has run: an
    // optimized module also holds the pointer arithmetic LLVM itself introduced.
    let build = fix_build_source_command(dir, ARRAY_ACCESS_SOURCE, "none")
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
    // A `getelementptr` instruction is one the program computes an address with. The same syntax
    // also appears as a constant expression that walks off a null pointer to name the size of a
    // type, which is a number rather than an address and stays outside every allocation.
    let geps = ir
        .lines()
        .map(|line| line.trim())
        .filter(|line| {
            line.starts_with('%') && line.contains(" = getelementptr")
        })
        .collect::<Vec<_>>();
    assert!(
        !geps.is_empty(),
        "reading and writing an array should compute pointers into it"
    );
    let unbounded = geps
        .iter()
        .filter(|line| !line.contains("= getelementptr inbounds"))
        .collect::<Vec<_>>();
    assert!(
        unbounded.is_empty(),
        "every `getelementptr` instruction should be `inbounds`, but {} of {} are not:\n{}",
        unbounded.len(),
        geps.len(),
        unbounded
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}
