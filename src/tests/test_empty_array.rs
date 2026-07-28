// Tests for the array literal with no element.
//
// Every empty array shares one module-level `#ArrayStorage`, so an empty literal allocates nothing.
// The block is in the global reference-count state: it is never retained, released or freed. Sharing
// it stays invisible because a capacity-zero array holds no element an alias could reach, so such an
// array is still reported unique and raising its capacity gives it a block of its own. The tests
// below exercise the paths where the sharing could show through — mutation, uniqueness, reference
// counting, a global value's initializer, and boxed elements.

#[cfg(test)]
mod empty_array_tests {
    use crate::{
        configuration::Configuration,
        constants::COMPILER_TEST_WORKING_PATH,
        misc::function_name,
        tests::test_util::{fix_command, test_source},
    };
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };

    #[test]
    pub fn test_empty_arrays_are_independent() {
        let source = r#"
module Main;

main : IO ();
main = (
    // Two empty arrays share one storage, so growing one must leave the other empty.
    let a = [] : Array I64;
    let b = [] : Array I64;
    let a = a.push_back(1).push_back(2);
    assert_eq(|_|"grown", a, [1, 2]);;
    assert_eq(|_|"untouched", b, []);;
    assert_eq(|_|"untouched size", b.@size, 0);;

    // The same array value used twice: the second push must not see the first.
    let e = [] : Array I64;
    assert_eq(|_|"first", e.push_back(10), [10]);;
    assert_eq(|_|"second", e.push_back(20), [20]);;

    // Reserving capacity on an empty array moves it off the shared storage.
    let r = ([] : Array I64).reserve(4).push_back(7);
    assert_eq(|_|"reserved", r, [7]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_empty_array_filled_into_a_larger_array() {
        let source = r#"
module Main;

main : IO ();
main = (
    // The idiom that makes empty arrays common: a row per vertex, then edges pushed into rows.
    let rows = Array::fill(4, [] : Array I64);
    let rows = rows.mod(1, push_back(11));
    let rows = rows.mod(1, push_back(12));
    let rows = rows.mod(3, push_back(33));
    assert_eq(|_|"row 0", rows.@(0), []);;
    assert_eq(|_|"row 1", rows.@(1), [11, 12]);;
    assert_eq(|_|"row 2", rows.@(2), []);;
    assert_eq(|_|"row 3", rows.@(3), [33]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_empty_array_is_unique() {
        let source = r#"
module Main;

main : IO ();
main = (
    // A capacity-zero array has no element an alias could reach, so it is unique whether it sits on
    // the shared block or on one of its own.
    let e = ([] : Array I64).assert_unique_array(|_|"empty literal");
    let (shared_unique, e) = e._unsafe_is_storage_unique;
    assert_eq(|_|"literal is unique", shared_unique, true);;
    assert_eq(|_|"still empty", e, []);;

    let a = (Array::empty(0) : Array I64).assert_unique_array(|_|"empty capacity");
    let (allocated_unique, a) = a._unsafe_is_storage_unique;
    assert_eq(|_|"allocated is unique", allocated_unique, true);;

    // Raising the capacity of either gives it a block of its own, and the elements written into it
    // are the ones read back.
    let e = e.reserve(2).push_back(1).push_back(2);
    assert_eq(|_|"grown literal", e, [1, 2]);;
    let a = a.push_back(3);
    assert_eq(|_|"grown allocated", a, [3]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_empty_array_of_boxed_elements() {
        let source = r#"
module Main;

type Boxed = box struct { value : I64 };

main : IO ();
main = (
    // An empty array of boxed elements: releasing it must free no element, and pushing into it
    // must take ownership of the pushed one.
    let e = [] : Array Boxed;
    assert_eq(|_|"empty size", e.@size, 0);;
    let e = e.push_back(Boxed { value : 42 });
    assert_eq(|_|"pushed", e.@(0).@value, 42);;

    // Many empty arrays created and dropped: the shared storage must survive all of them.
    let total = Iterator::range(0, 100).fold(0, |_, acc|
        acc + ([] : Array Boxed).push_back(Boxed { value : 1 }).@(0).@value
    );
    assert_eq(|_|"total", total, 100);;

    // An empty array nested in a literal, and one carried through a structure.
    let nested = [[] : Array I64, [1]];
    assert_eq(|_|"nested 0", nested.@(0), []);;
    assert_eq(|_|"nested 1", nested.@(1), [1]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_empty_array_in_a_global_value() {
        let source = r#"
module Main;

// Initializing a global marks its whole value graph as global, which reaches the shared storage.
empty_global : Array I64;
empty_global = [];

rows_global : Array (Array I64);
rows_global = Array::fill(3, []);

main : IO ();
main = (
    assert_eq(|_|"global empty", empty_global, []);;
    assert_eq(|_|"global grown", empty_global.push_back(5), [5]);;
    assert_eq(|_|"global still empty", empty_global.@size, 0);;
    assert_eq(|_|"global row", rows_global.@(1), []);;
    assert_eq(|_|"global row grown", rows_global.@(1).push_back(9), [9]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_empty_array_literal_shares_one_storage() {
        // The sharing is what removes the allocation, and it is invisible from a Fix program: an
        // empty array behaves exactly as a freshly allocated one. The emitted IR is where it shows.
        let source = r#"
        module Main;

        rows : Array (Array I64);
        rows = Array::fill(2, []);

        main : IO ();
        main = (
            let a = [] : Array I64;
            println((a.@size + rows.@size).to_string)
        );
        "#;
        let work_dir = PathBuf::from(format!(
            "{}/{}",
            COMPILER_TEST_WORKING_PATH,
            function_name!()
        ));
        let _ = fs::remove_dir_all(&work_dir);
        fs::create_dir_all(&work_dir).unwrap();
        File::create(work_dir.join("main.fix"))
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();

        let output = fix_command()
            .args([
                "build",
                "-O",
                "none",
                "--emit-llvm",
                "--file",
                "main.fix",
                "--output",
                "prog",
            ])
            .current_dir(&work_dir)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(0),
            "`fix build --emit-llvm` failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let ir = fs::read_dir(&work_dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                let name = p.file_name().unwrap().to_string_lossy();
                name.ends_with(".ll") && !name.ends_with("_optimized.ll")
            })
            .map(|p| fs::read_to_string(p).unwrap())
            .collect::<Vec<_>>()
            .join("\n");

        // The block carries a reference count of one and the global state tag, so that retain and
        // release skip it.
        let expected = format!(
            "= internal global {{ {{ i32, i8 }}, {{ i64 }} }} \
             {{ {{ i32, i8 }} {{ i32 1, i8 {} }}, {{ i64 }} zeroinitializer }}",
            crate::constants::REFCNT_STATE_GLOBAL
        );
        assert!(
            ir.contains(&expected),
            "emitted IR lacks the shared empty storage `{}`:\n{}",
            expected,
            ir
        );
    }
}
