// Tests for the PunchedArray builtins `_punch` / `_plug` (which force-unique) and their
// `_uniqueness_unchecked` variants. `_punch` moves an element out of an array, leaving a hole;
// `_plug` writes an element back into the hole. A PunchedArray's release / clone skips the
// hole, so with boxed elements the moved-out element is neither leaked nor double-freed —
// checked here under valgrind.

#[cfg(test)]
mod punched_array_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    #[test]
    pub fn test_punch_plug_roundtrip() {
        let source = r#"
module Main;

main : IO () = (
    // Unboxed round-trip: punch, then plug a new element into the hole.
    let (parr, elm) = PunchedArray::_punch(1, [10, 20, 30]);
    assert_eq(|_|"elem", elm, 20);;
    assert_eq(|_|"plug", PunchedArray::_plug_uniqueness_unchecked(99, parr), [10, 99, 30]);;

    // Boxed round-trip.
    let (parr, elm) = PunchedArray::_punch(1, [[1],[2],[3]]);
    assert_eq(|_|"elem boxed", elm, [2]);;
    assert_eq(|_|"plug boxed", PunchedArray::_plug_uniqueness_unchecked([99], parr), [[1],[99],[3]]);;

    // Plug at the first and last index.
    let (parr, _) = PunchedArray::_punch(0, [1, 2, 3]);
    assert_eq(|_|"plug first", PunchedArray::_plug_uniqueness_unchecked(9, parr), [9, 2, 3]);;
    let (parr, _) = PunchedArray::_punch(2, [1, 2, 3]);
    assert_eq(|_|"plug last", PunchedArray::_plug_uniqueness_unchecked(9, parr), [1, 2, 9]);;

    // Multi-plug (shared punched array): the force-unique plug clones per call, so each
    // result is independent.
    let (parr, _) = PunchedArray::_punch(1, [[1],[2],[3]]);
    let a1 = PunchedArray::_plug([10], parr);
    let a2 = PunchedArray::_plug([20], parr);
    assert_eq(|_|"multiplug a1", a1, [[1],[10],[3]]);;
    assert_eq(|_|"multiplug a2", a2, [[1],[20],[3]]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_punch_plug_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let source = r#"
module Main;

main : IO () = (
    // Drop a punched array without plugging: the hole (a moved-out element) must be skipped
    // by the release, so its element is dropped exactly once.
    let (parr, elm) = PunchedArray::_punch(1, [[1],[2],[3]]);
    eval elm;    // drops the moved-out element
    eval parr;   // drops the punched array (skip-idx release of the survivors)

    // Drop the punched array while the moved-out element is still held; the held element
    // stays valid.
    let (parr, elm) = PunchedArray::_punch(0, [[7],[8],[9]]);
    eval parr;
    assert_eq(|_|"held elem after drop", elm, [7]);;

    // Multi-plug with boxed elements: the skip-idx clone retains the survivors and leaves the
    // hole for the plug, without touching the moved-out element.
    let (parr, elm) = PunchedArray::_punch(1, [[1],[2],[3]]);
    eval elm;
    let a1 = PunchedArray::_plug([10], parr);
    let a2 = PunchedArray::_plug([20], parr);
    assert_eq(|_|"boxed multiplug a1", a1, [[1],[10],[3]]);;
    assert_eq(|_|"boxed multiplug a2", a2, [[1],[20],[3]]);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }
}
