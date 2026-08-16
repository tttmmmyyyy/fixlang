use crate::tests::test_util::{
    emitted_llvm_ir, fix_build_source_command, llvm_function_bodies, EmittedIr,
};
use tempfile::TempDir;

/// A program holding a global whose initializer is long enough that a size-based inlining would
/// leave the accessor a call, read from a loop where the accessor's flag test and load want to be
/// carried out.
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

/// The accessor that reads a global asks to be placed wherever the global is read.
///
/// The accessor tests an initialization flag and loads the stored value, and it also holds the
/// initializer. A reader that has the test and the load in front of it also has every write to the
/// flag and the storage in front of it, which is what lets those leave a loop over the global; left
/// as a call, they stay in the loop and each read costs a call as well. The size an ordinary
/// inlining decides by is the initializer's, and the initializer is on the path a single access of
/// the run takes, so it is not the size to decide this by — hence the request, whatever that size.
///
/// The request is what this test reads, rather than whether the placement happened: the placement
/// is LLVM's to make, and it makes it differently as the surrounding code changes, where the
/// request is what the compiler decides.
#[test]
pub fn test_the_accessor_of_a_global_asks_to_be_placed_at_its_readers() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    let build = fix_build_source_command(dir, LONG_INITIALIZER_SOURCE, "max")
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
    let accessors = llvm_function_bodies(&ir, "@\"Get#Main::table#");
    assert_eq!(
        accessors.len(),
        1,
        "the build should emit one accessor for `table`"
    );
    let signature = accessors[0]
        .lines()
        .next()
        .expect("a function body starts with its signature");
    let group = signature
        .rsplit_once('#')
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .map(|group| format!("attributes #{} =", group))
        .expect("the accessor's signature should name an attribute group");
    let attributes = ir
        .lines()
        .find(|line| line.starts_with(&group))
        .unwrap_or_else(|| panic!("the emitted IR should define `{}`", group));
    assert!(
        attributes.contains("alwaysinline"),
        "the accessor should ask to be placed at its readers, and its attributes are `{}`",
        attributes
    );
}
