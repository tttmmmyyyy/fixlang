// Tests for the array literal with no element.
//
// A literal with no element takes one module-level `#ArrayStorage` instead of allocating, so every
// such literal of an element type shares a block. The block is in the global reference-count state:
// it is never retained, released or freed. Sharing it stays invisible because a capacity-zero array
// holds no element an alias could reach, so such an array is still reported unique and raising its
// capacity gives it a block of its own. The tests below exercise the paths where the sharing could
// show through — mutation, uniqueness, reference counting, a global value's initializer, and boxed
// elements.

#[cfg(test)]
mod empty_array_tests {
    use crate::{
        configuration::{Configuration, FixOptimizationLevel},
        constants::REFCNT_STATE_GLOBAL,
        misc::function_name,
        tests::test_util::{emit_llvm_ir, test_source},
    };

    #[test]
    pub fn test_empty_arrays_are_independent() {
        // Empty arrays that share one block stay independent: growing one leaves the others empty,
        // and pushing into one empty value twice gives two results of one element each.
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
        // An array filled with empty rows: pushing into one row leaves the other rows empty, as a
        // fill of separately allocated rows would.
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
        // What uniqueness reports for a capacity-zero array — one from a literal, one from
        // `Array::empty(0)`, and one held under two names — against what it reports for an array
        // with reserved capacity, where a second name does make it shared.
        let source = r#"
module Main;

main : IO ();
main = (
    // A capacity-zero array has no element an alias could reach, so it is unique whether it sits on
    // the shared block or on one of its own.
    let e = ([] : Array I64).assert_unique_array(|_|"empty literal");
    let (literal_unique, e) = e._unsafe_is_storage_unique;
    assert_eq(|_|"literal is unique", literal_unique, true);;
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

    // Two names for one capacity-zero block: both still read as unique, and growing one leaves the
    // other empty.
    let aliased = Array::empty(0) : Array I64;
    let other_alias = aliased;
    let (aliased_unique, aliased) = aliased._unsafe_is_storage_unique;
    assert_eq(|_|"aliased empty is unique", aliased_unique, true);;
    assert_eq(|_|"grown alias", aliased.push_back(4), [4]);;
    assert_eq(|_|"other alias untouched", other_alias, []);;

    // Reserved capacity is what makes an array shareable, so an array with room and no element is
    // reported shared once a second name holds it.
    let reserved = Array::empty(4) : Array I64;
    let reserved_alias = reserved;
    let (reserved_unique, reserved) = reserved._unsafe_is_storage_unique;
    assert_eq(|_|"reserved and aliased is shared", reserved_unique, false);;
    assert_eq(|_|"reserved grows", reserved.push_back(5), [5]);;
    assert_eq(|_|"reserved alias untouched", reserved_alias, []);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_empty_array_of_boxed_elements() {
        // Reference counting around the shared block: an empty array of boxed elements releases no
        // element and takes ownership of what is pushed into it, and the block survives arbitrarily
        // many empty arrays created and dropped. A nested literal and an element type of size zero
        // are covered here too.
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

    // An element type of no size at all: the block's buffer field is an empty aggregate.
    let units = [] : Array ();
    assert_eq(|_|"unit size", units.@size, 0);;
    assert_eq(|_|"unit grown", units.push_back(()).@size, 1);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_empty_array_in_a_global_value() {
        // A global value holding an empty array: initializing it marks its whole value graph global,
        // which reaches the shared block, and the value stays readable and growable afterwards.
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

    /// The Fix program the empty-array behavior is checked with, used at more than one build
    /// setting.
    const EMPTY_ARRAY_BEHAVIOR: &str = r#"
module Main;

type Boxed = box struct { value : I64 };

empty_global : Array I64;
empty_global = [];

rows_global : Array (Array I64);
rows_global = Array::fill(3, []);

main : IO ();
main = (
    let a = [] : Array I64;
    let b = [] : Array I64;
    assert_eq(|_|"grown", a.push_back(1).push_back(2), [1, 2]);;
    assert_eq(|_|"untouched", b, []);;
    assert_eq(|_|"reserved", ([] : Array I64).reserve(4).push_back(7), [7]);;
    assert_eq(|_|"appended", ([] : Array I64) + [1, 2], [1, 2]);;
    assert_eq(|_|"appended empty", ([] : Array I64) + [], []);;
    assert_eq(|_|"sorted", ([] : Array I64).sort, []);;
    assert_eq(|_|"resized", ([] : Array I64).resize(2, 3), [3, 3]);;
    assert_eq(|_|"collected", ([] : Array I64).to_iter.to_array, []);;

    let rows = Array::fill(4, [] : Array I64);
    let rows = rows.mod(1, push_back(11));
    assert_eq(|_|"row 0", rows.@(0), []);;
    assert_eq(|_|"row 1", rows.@(1), [11]);;

    let e = [] : Array Boxed;
    assert_eq(|_|"boxed pushed", e.push_back(Boxed { value : 42 }).@(0).@value, 42);;
    let total = Iterator::range(0, 100).fold(0, |_, acc|
        acc + ([] : Array Boxed).push_back(Boxed { value : 1 }).@(0).@value
    );
    assert_eq(|_|"boxed total", total, 100);;

    assert_eq(|_|"global empty", empty_global, []);;
    assert_eq(|_|"global grown", empty_global.push_back(5), [5]);;
    assert_eq(|_|"global row grown", rows_global.@(1).push_back(9), [9]);;
    pure()
);
"#;

    #[test]
    pub fn test_empty_array_at_every_optimization_level() {
        // The shared block is taken at every optimization level, while the check that clones a
        // shared array before writing into it is dropped only from `max` up. Both sides of that have
        // to behave the same.
        for level in [
            FixOptimizationLevel::None,
            FixOptimizationLevel::Basic,
            FixOptimizationLevel::Max,
            FixOptimizationLevel::Experimental,
        ] {
            let mut config = Configuration::develop_mode();
            config.set_fix_opt_level(level);
            test_source(EMPTY_ARRAY_BEHAVIOR, config);
        }
    }

    #[test]
    pub fn test_empty_array_survives_publication() {
        // Publishing a value graph to other threads walks it and writes the threaded state into
        // every object it reaches. The shared block lives in read-only memory, so the walk has to
        // leave it as it is, and the arrays on it stay readable and growable afterwards.
        let source = r#"
module Main;

type Rec = box struct { rows : Array (Array I64), tag : Array U8 };

global_empty : Array I64;
global_empty = [];

main : IO ();
main = (
    let v = (Rec { rows : Array::fill(4, []), tag : [] }).mark_threaded;
    assert_eq(|_|"row after publish", v.@rows.@(1), []);;
    assert_eq(|_|"row grown after publish", v.@rows.@(1).push_back(1), [1]);;
    assert_eq(|_|"tag after publish", v.@tag.@size, 0);;

    let e = ([] : Array I64).mark_threaded;
    assert_eq(|_|"published empty grown", e.push_back(2), [2]);;
    let g = global_empty.mark_threaded;
    assert_eq(|_|"published global grown", g.push_back(3), [3]);;
    let twice = (([] : Array I64).mark_threaded).mark_threaded;
    assert_eq(|_|"published twice", twice.push_back(4), [4]);;
    let heap = (Array::empty(0) : Array I64).mark_threaded;
    assert_eq(|_|"published capacity-zero heap block", heap.push_back(5), [5]);;

    let n = Iterator::range(0, 200).fold(0, |_, acc|
        acc + (([] : Array I64).mark_threaded).push_back(1).@size
    );
    assert_eq(|_|"loop", n, 200);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_threaded();
        test_source(source, config);
    }

    #[test]
    pub fn test_capacity_zero_array_off_the_shared_block_grows() {
        // A capacity-zero array that is a heap block of its own — no literal with no element appears
        // in this program — takes the same path as one on the shared block when its capacity is
        // raised: a fresh block is allocated and the old one is dropped. Its elements have to reach
        // the new block, and the old block has to be freed exactly once.
        let source = r#"
module Main;

type Boxed = box struct { v : I64 };

main : IO ();
main = (
    let a = (Array::empty(0) : Array I64).push_back(1).push_back(2).push_back(3);
    assert_eq(|_|"grown from empty(0)", a, [1, 2, 3]);;

    let b = (Array::empty(0) : Array Boxed).push_back(Boxed { v : 5 });
    assert_eq(|_|"boxed grown", b.@(0).@v, 5);;

    // Many capacity-zero blocks grown, and many dropped without growing: a block that is neither
    // carried over nor freed shows up as a leak.
    let grown = Iterator::range(0, 200).fold(0, |i, acc|
        acc + (Array::empty(0) : Array I64).push_back(i).@(0)
    );
    assert_eq(|_|"grown total", grown, 199 * 200 / 2);;
    let dropped = Iterator::range(0, 200).fold(0, |_, acc|
        acc + (Array::empty(0) : Array Boxed).@size
    );
    assert_eq(|_|"dropped", dropped, 0);;

    // Capacity-zero arrays that other functions produce.
    let sub = [1, 2, 3].get_sub(1, 1);
    assert_eq(|_|"empty sub grown", sub.push_back(9), [9]);;
    let filled = Array::fill(0, Boxed { v : 1 });
    assert_eq(|_|"fill 0 grown", filled.push_back(Boxed { v : 2 }).@(0).@v, 2);;

    // A capacity-zero array held under two names: growing one leaves the other empty.
    let x = Array::empty(0) : Array I64;
    let y = x;
    assert_eq(|_|"x grown", x.push_back(11), [11]);;
    assert_eq(|_|"y untouched", y.@size, 0);;
    assert_eq(|_|"y grown", y.push_back(22), [22]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_capacity_shrunk_to_zero_and_grown_again() {
        // An array whose capacity is lowered to zero joins the capacity-zero case from the other
        // side: it sits on a block of its own, not on the shared one, and a second name for it makes
        // that block genuinely shared. Growing it again has to leave the other name empty.
        let source = r#"
module Main;

main : IO ();
main = (
    // Lowering the capacity to zero is in contract while the array holds no element.
    let a = (Array::empty(8) : Array I64)._unsafe_set_capacity_bounds_unchecked(0);
    assert_eq(|_|"lowered capacity", a.@capacity, 0);;
    assert_eq(|_|"lowered size", a.@size, 0);;

    // A second name for that block, then growth through the first.
    let alias = a;
    assert_eq(|_|"grown", a.push_back(1).push_back(2), [1, 2]);;
    assert_eq(|_|"alias untouched", alias, []);;

    // The same under the `true` arm of the uniqueness flag, which is where the optimizer reads the
    // array as uniquely owned.
    let b = (Array::empty(8) : Array I64)._unsafe_set_capacity_bounds_unchecked(0);
    let b_alias = b;
    let (unique, b) = b._unsafe_is_storage_unique;
    let grown = if unique { b.reserve(4).push_back(10).push_back(20) } else { [-1] };
    assert_eq(|_|"grown under the flag", grown, [10, 20]);;
    assert_eq(|_|"alias untouched under the flag", b_alias, []);;

    // Lowering the capacity of an array a second name already holds.
    let c = Array::empty(8) : Array I64;
    let c_alias = c;
    let c = c._unsafe_set_capacity_bounds_unchecked(0);
    assert_eq(|_|"shared lowered", c.push_back(3), [3]);;
    assert_eq(|_|"shared alias untouched", c_alias, []);;

    // Lowering and raising in a loop, with boxed elements to release each round.
    let looped = Iterator::range(0, 200).fold([] : Array String, |i, arr|
        arr.push_back(i.to_string).truncate(0)._unsafe_set_capacity_bounds_unchecked(0)
    );
    assert_eq(|_|"looped", looped.@size, 0);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_update_driven_by_the_uniqueness_answer() {
        // A program that branches on `_unsafe_is_storage_unique` and writes through the unchecked
        // primitive on `true` — the shape `Array::_unsafe_act_bounds_unchecked` uses. An array that
        // starts empty and is then grown and aliased must still take the checked path.
        let source = r#"
module Main;

my_set : I64 -> I64 -> Array I64 -> Array I64;
my_set = |idx, v, arr| (
    let (unique, arr) = arr._unsafe_is_storage_unique;
    if unique {
        arr.unsafe_set_bounds_unchecked(idx, v)
    } else {
        arr.set(idx, v)
    }
);

main : IO ();
main = (
    // Empty, then grown, then held under a second name: the write must not reach the second name.
    let empty = [] : Array I64;
    let empty_alias = empty;
    let grown = empty.push_back(1).push_back(2).push_back(3);
    let grown_alias = grown;
    assert_eq(|_|"written", grown.my_set(1, 99), [1, 99, 3]);;
    assert_eq(|_|"grown alias untouched", grown_alias, [1, 2, 3]);;
    assert_eq(|_|"empty alias untouched", empty_alias, []);;

    // The same starting from `Array::empty(0)`.
    let allocated = Array::empty(0) : Array I64;
    let allocated_alias = allocated;
    let filled = allocated.push_back(4).push_back(5);
    let filled_alias = filled;
    assert_eq(|_|"allocated written", filled.my_set(0, 88), [88, 5]);;
    assert_eq(|_|"allocated alias untouched", filled_alias, [4, 5]);;
    assert_eq(|_|"allocated empty alias untouched", allocated_alias, []);;

    // A row that started empty inside a larger array, updated through the same path.
    let rows = Array::fill(3, [] : Array I64).mod(0, push_back(7)).mod(1, push_back(8));
    let rows_alias = rows;
    let rows = rows.mod(0, my_set(0, 77));
    assert_eq(|_|"row written", rows.@(0), [77]);;
    assert_eq(|_|"rows alias untouched", rows_alias.@(0), [7]);;
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
        let ir = emit_llvm_ir(source, function_name!(), "max");

        // Both `Array I64` literals reach the same block: the module defines a block per element
        // type, so a second definition for one name would mean each literal took a block of its own.
        // Definitions of other element types are the standard library's and are none of this test's
        // business.
        let definitions = ir
            .lines()
            .filter(|line| line.starts_with("@\"EmptyArrayStorage#"))
            .collect::<Vec<_>>();
        let names = definitions
            .iter()
            .map(|line| line.split_whitespace().next().unwrap())
            .collect::<Vec<_>>();
        // A second block for one element type would be added under the name the first already holds,
        // and LLVM keeps that unique by appending a counter — the one thing a hexadecimal hash never
        // contains is a dot.
        let renamed = names.iter().find(|name| name.contains('.'));
        assert!(
            renamed.is_none(),
            "one element type wants one shared block, and the IR renamed a second `{}`:\n{}",
            renamed.unwrap_or(&""),
            ir
        );
        assert!(
            !names.is_empty(),
            "the empty literals want a shared block, and the IR defines none:\n{}",
            ir
        );

        // The block is a constant, and it carries a reference count of one and the global state tag,
        // so that retain and release skip it.
        let definition = definitions[0];
        for expected in [
            "= internal constant".to_string(),
            "i32 1".to_string(),
            format!("i8 {}", REFCNT_STATE_GLOBAL),
        ] {
            assert!(
                definition.contains(&expected),
                "the shared empty storage lacks `{}`:\n{}",
                expected,
                definition
            );
        }
    }
}
