// Tests for the array builder primitives `append`, `reserve`, `resize`, and `push_back`. Each
// clones the array if it is shared, so building on a shared array must leave the original intact.
// The memory-safety test checks the boxed-element paths under valgrind: `append` moves the elements
// out of a unique source (with no reference counting) and copies them out of a shared one,
// `reserve` reallocs a unique array's block and copies a shared one, and each must neither leak an
// element nor free one twice.

#[cfg(test)]
mod array_builder_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    /// Verifies the values `append`, `reserve`, `resize`, `push_back`, `get_sub` and
    /// `unsafe_set_bounds_unchecked` produce, including the empty-range edge cases, and that each
    /// clones before writing so a shared argument keeps the elements it held.
    #[test]
    pub fn test_builder_correctness() {
        let source = r#"
module Main;

main : IO () = (
    // `append` on unboxed / boxed arrays.
    assert_eq(|_|"append unboxed", [1, 2].append([3, 4]), [1, 2, 3, 4]);;
    assert_eq(|_|"append boxed", [[1], [2]].append([[3]]), [[1], [2], [3]]);;
    assert_eq(|_|"append empty src", [1, 2].append([]), [1, 2]);;
    assert_eq(|_|"append empty dst", ([] : Array I64).append([3, 4]), [3, 4]);;

    // `append` of a shared array leaves both arguments intact.
    let a = [1, 2];
    let b = [3, 4];
    let c = a.append(b);
    assert_eq(|_|"append shared src", b, [3, 4]);;
    assert_eq(|_|"append shared dst", a, [1, 2]);;
    assert_eq(|_|"append shared result", c, [1, 2, 3, 4]);;

    // `append` of a shared empty source leaves it and the destination intact.
    let e = ([] : Array (Array I64));
    let d = [[1], [2]];
    assert_eq(|_|"append shared empty src result", d.append(e), [[1], [2]]);;
    assert_eq(|_|"append shared empty src intact", e, []);;
    assert_eq(|_|"append shared empty src dst intact", d, [[1], [2]]);;

    // Writing into an array after copying a range out of it. The copy borrows the array it reads,
    // so the writes go into that same array, and the copy keeps the elements it took.
    let base = [[1], [2], [3], [4]];
    let head = base.get_sub(0, 2);
    let swapped = base.set(0, head.@(1)).set(1, head.@(0));
    assert_eq(|_|"write after copy", swapped, [[2], [1], [3], [4]]);;
    assert_eq(|_|"copy intact after write", head, [[1], [2]]);;

    // `reserve` grows the capacity while keeping the elements.
    let r = [1, 2, 3].reserve(16);
    assert_eq(|_|"reserve keeps elements", r, [1, 2, 3]);;
    assert_eq(|_|"reserve grows capacity", r.@capacity >= 16, true);;

    // `resize` grows with the fill value and truncates.
    assert_eq(|_|"resize grow", [1, 2].resize(4, 9), [1, 2, 9, 9]);;
    assert_eq(|_|"resize shrink", [1, 2, 3, 4].resize(2, 0), [1, 2]);;

    // `push_back` past the capacity reallocates.
    let p = Iterator::range(0, 100).fold(Array::empty(1), |i, arr| arr.push_back(i));
    assert_eq(|_|"push_back grow", p.@(99), 99);;
    assert_eq(|_|"push_back size", p.@size, 100);;

    // `unsafe_set_bounds_unchecked` writes an element in place, cloning a shared array.
    assert_eq(|_|"unsafe_set boxed", [[1], [2], [3]].unsafe_set_bounds_unchecked(1, [9]), [[1], [9], [3]]);;
    let sh = [[1], [2]];
    let s2 = sh.unsafe_set_bounds_unchecked(0, [9]);
    assert_eq(|_|"unsafe_set shared original", sh, [[1], [2]]);;
    assert_eq(|_|"unsafe_set shared result", s2, [[9], [2]]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies under valgrind that the array builder primitives leak no boxed element and free
    /// none twice, over both element paths they choose between: the move taken out of a uniquely
    /// owned source and the retain-per-element copy taken out of a shared one.
    #[test]
    pub fn test_builder_memory_safety() {
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
    // `append` moves a unique boxed source's elements out with no reference counting; each must
    // end up owned by the result exactly once.
    eval [[1], [2]].append([[3], [4]]);

    // `append` copies a shared boxed source's elements with a retain each; the source and the result
    // must be released independently.
    let src = [[3], [4]];
    let dst = [[1], [2]];
    let both = dst.append(src);
    assert_eq(|_|"append shared src intact", src, [[3], [4]]);;
    assert_eq(|_|"append shared dst intact", dst, [[1], [2]]);;
    assert_eq(|_|"append shared result", both, [[1], [2], [3], [4]]);;

    // `reserve` reallocates a unique boxed array's block; the elements survive the move.
    eval [[1], [2], [3]].reserve(64);

    // `resize` grows a boxed array with a shared fill value and shrinks another, releasing the
    // dropped elements.
    eval [[1], [2]].resize(4, [9]);
    eval [[1], [2], [3], [4]].resize(2, [0]);

    // Growing a boxed array by repeated `push_back` reallocates several times.
    eval Iterator::range(0, 50).fold(Array::empty(1), |i, arr| arr.push_back([i]));

    // `sort_stable_by` merges between the array and a working copy of it, writing each element of
    // one over an element of the other; on boxed elements every element the writes drop must be
    // released exactly once.
    assert_eq(|_|"sort_stable boxed",
        Iterator::range(0, 40).map(|i| [(i * 7) % 40]).to_array.sort_stable_by(|(a, b)| a.@(0) < b.@(0)),
        Iterator::range(0, 40).map(|i| [i]).to_array);;

    // An input already in order takes, at every merge, the copy of a whole range instead of the
    // element-by-element comparison. Keeping a second holder makes the first of those copies clone
    // the array it writes into.
    let ordered = Iterator::range(0, 40).map(|i| [i]).to_array;
    assert_eq(|_|"sort_stable boxed already ordered",
        ordered.sort_stable_by(|(a, b)| a.@(0) < b.@(0)),
        Iterator::range(0, 40).map(|i| [i]).to_array);;
    assert_eq(|_|"the ordered source is intact", ordered,
        Iterator::range(0, 40).map(|i| [i]).to_array);;

    // `get_sub` on a boxed array copies the range out; the source stays intact.
    let source = [[1], [2], [3], [4]];
    assert_eq(|_|"get_sub boxed", source.get_sub(1, 3), [[2], [3]]);;
    assert_eq(|_|"get_sub boxed src intact", source, [[1], [2], [3], [4]]);;

    // Writing into a boxed array after copying a range out of it. The copy retains each element it
    // takes and borrows the array itself, so the writes reach that array in place while the copy
    // keeps its own references to the elements.
    let base = [[1], [2], [3], [4]];
    let head = base.get_sub(0, 2);
    let swapped = base.set(0, head.@(1)).set(1, head.@(0));
    assert_eq(|_|"write after copy", swapped, [[2], [1], [3], [4]]);;
    assert_eq(|_|"copy intact after write", head, [[1], [2]]);;

    // `append` of a shared empty boxed source takes the copy path over an empty range.
    let empty_src = ([] : Array (Array I64));
    let empty_src_dst = [[1], [2]];
    assert_eq(|_|"append shared empty src result", empty_src_dst.append(empty_src), [[1], [2]]);;
    assert_eq(|_|"append shared empty src intact", empty_src, []);;

    // `unsafe_set_bounds_unchecked` on a boxed array releases the overwritten element and, on a
    // shared array, clones so the original keeps its element.
    eval [[1], [2], [3]].unsafe_set_bounds_unchecked(1, [9]);
    let shared_base = [[1], [2], [3]];
    let overwritten = shared_base.unsafe_set_bounds_unchecked(0, [9]);
    assert_eq(|_|"unsafe_set shared base intact", shared_base, [[1], [2], [3]]);;
    assert_eq(|_|"unsafe_set shared overwritten", overwritten, [[9], [2], [3]]);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }

    /// Verifies that the append primitives keep the reference-count state dispatch on a global
    /// array, whose whole graph its initializer marked global.
    ///
    /// Each reference count the primitives perform meets a global object here: the release of the
    /// source `append` consumes, the retain of each element either of them copies, and the
    /// retain-per-element clone of a shared destination. A global is the only value a locality
    /// annotation can meet outside the local state, so this is where the declarations of what the
    /// primitives count reach a runtime check: development mode aborts at a reference count
    /// inferred local that meets a non-local object.
    #[test]
    pub fn test_builder_global_state_dispatch() {
        let source = r#"
module Main;

g : Array (Array I64);
g = [[1], [2]];

// A global with room past its length, so that it can be a destination written into.
g_spare : Array (Array I64);
g_spare = [[1], [2]].reserve(8);

main : IO () = (
    // The standard library's own callers of the two primitives, over a global array.
    assert_eq(|_|"get_sub of a global", g.get_sub(0, 2), [[1], [2]]);;
    assert_eq(|_|"append a global", [[0]].append(g), [[0], [1], [2]]);;
    assert_eq(|_|"append onto a global", g.append([[3]]), [[1], [2], [3]]);;

    // The primitives directly, with a global source and a local destination.
    let dst = ([[0]] : Array (Array I64)).reserve(8);
    assert_eq(|_|"copy a global into a shared destination",
        dst._unsafe_copy_capacity_bounds_unchecked(g, 0, 2), [[0], [1], [2]]);;
    assert_eq(|_|"append a global into a shared destination",
        dst._unsafe_append_capacity_unchecked(g), [[0], [1], [2]]);;

    // A global destination, which both primitives clone before writing into it.
    assert_eq(|_|"copy a local into a global destination",
        g_spare._unsafe_copy_capacity_bounds_unchecked([[7]], 0, 1), [[1], [2], [7]]);;
    assert_eq(|_|"append a local into a global destination",
        g_spare._unsafe_append_capacity_unchecked([[8]]), [[1], [2], [8]]);;

    assert_eq(|_|"the global sources are intact", (g, g_spare), ([[1], [2]], [[1], [2]]));;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies that a copy taken out of a global array yields one that still reaches the global's
    /// elements, so that a later operation cloning it keeps the state dispatch on their retains.
    ///
    /// `get_sub` allocates the array it returns, so that array's own object is local; the elements
    /// it holds come from wherever it copied them, and here that is a global. Sharing the copy
    /// makes the next `reserve` clone it by retaining each of those elements.
    #[test]
    pub fn test_copy_of_a_global_still_reaches_the_global_elements() {
        let source = r#"
module Main;

g : Array (Array I64);
g = [[1], [2]];

main : IO () = (
    let sub = g.get_sub(0, 2);
    let grown = sub.reserve(8);
    let grown_more = sub.reserve(16);
    assert_eq(|_|"the copy", sub, [[1], [2]]);;
    assert_eq(|_|"the grown copies", (grown, grown_more), ([[1], [2]], [[1], [2]]));;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies that the append primitives answer correctly over a multi-threaded array, whose
    /// storage and every element are out of the local reference-counting state.
    ///
    /// The uniqueness checks the primitives make on their own — `append`'s check on the source it
    /// moves the elements out of, and the clone-when-shared check on the destination — carry a
    /// threaded arm in a threaded build, and that arm returns a uniquely held threaded array to the
    /// local state before the move.
    #[test]
    pub fn test_builder_threaded_state_dispatch() {
        let source = r#"
module Main;

main : IO () = (
    // A multi-threaded array: its storage and every element are out of the local state.
    let threaded = [[1], [2], [3], [4]].mark_threaded;

    // The borrowing copy retains each element it takes out of a threaded source.
    assert_eq(|_|"get_sub of a threaded array", threaded.get_sub(1, 3), [[2], [3]]);;

    // The owning append over a threaded source that is uniquely held (the move path).
    assert_eq(|_|"append a uniquely held threaded source",
        [[0]].append(threaded.get_sub(0, 2).mark_threaded), [[0], [1], [2]]);;

    // The owning append over a threaded source that is shared (the retain-per-element copy path).
    assert_eq(|_|"append a shared threaded source", [[0]].append(threaded), [[0], [1], [2], [3], [4]]);;

    // A threaded destination, which the primitive clones before writing.
    assert_eq(|_|"append onto a threaded destination", threaded.append([[5]]),
        [[1], [2], [3], [4], [5]]);;
    assert_eq(|_|"the threaded array is intact", threaded, [[1], [2], [3], [4]]);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_threaded();
        test_source(source, config);
    }

    /// Verifies that a range copy leaves the array it reads sharable by everyone who already held
    /// it: a global, a struct field, an unboxed-union payload, and a value the caller keeps. Each
    /// holder then writes, and the write must reach only the array that holder owns.
    #[test]
    pub fn test_copy_leaves_other_holders_intact() {
        let source = r#"
module Main;

table : Array (Array I64);
table = [[1], [2], [3], [4]];

type Holder = unbox struct { xs : Array (Array I64) };
type Slot = unbox union { full : Array (Array I64), empty : () };

// A copy out of a global, then a write. The global keeps its own reference, so the write clones.
from_global : () -> (Array (Array I64), Array (Array I64));
from_global = |_| (
    let head = table.get_sub(0, 2);
    (table.set(0, [99]), head)
);

// A copy out of a struct field, then a write through the field.
from_field : Holder -> (Array (Array I64), Array (Array I64));
from_field = |holder| (
    let head = holder.@xs.get_sub(0, 2);
    (holder.mod_xs(|xs| xs.set(0, [88])).@xs, head)
);

// A copy out of an unboxed-union payload carried through a loop, then writes.
from_union : Array (Array I64) -> Array (Array I64);
from_union = |arr| (
    let out = loop((0, Slot::full(arr)), |(i, slot)| (
        if i == 2 { break $ slot };
        let arr = slot.as_full;
        let head = arr.get_sub(0, 2);
        let arr = arr.set(0, head.@(1)).set(1, head.@(0));
        continue $ (i + 1, Slot::full(arr))
    ));
    out.as_full
);

main : IO () = (
    // The global's array is untouched, and the copy keeps the elements it took.
    let (written, head) = from_global();
    assert_eq(|_|"global written", written, [[99], [2], [3], [4]]);;
    assert_eq(|_|"global intact", table, [[1], [2], [3], [4]]);;
    assert_eq(|_|"global copy intact", head, [[1], [2]]);;

    let (written, head) = from_field(Holder { xs : [[1], [2], [3]] });
    assert_eq(|_|"field written", written, [[88], [2], [3]]);;
    assert_eq(|_|"field copy intact", head, [[1], [2]]);;

    // Two round trips through the union leave the array where it started.
    assert_eq(|_|"union twice", from_union([[1], [2], [3]]), [[1], [2], [3]]);;

    // A copy out of an array of arrays, then a write through the copy and a write through the
    // source. Each element belongs to both, so each write must clone the element it changes.
    let arr = [[1, 10], [2, 20], [3, 30]];
    let head = arr.get_sub(0, 2);
    let head = head.mod(0, |x| x.set(0, 999));
    let arr = arr.mod(1, |x| x.set(0, 777));
    assert_eq(|_|"source after both writes", arr, [[1, 10], [777, 20], [3, 30]]);;
    assert_eq(|_|"copy after both writes", head, [[999, 10], [2, 20]]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies `sort_stable_by` on boxed elements, over inputs long enough that the sort merges
    /// them instead of sorting them by insertion: the order is stable, and an array a second holder
    /// keeps is left as it was.
    #[test]
    pub fn test_sort_stable_by_with_a_second_holder() {
        let source = r#"
module Main;

type Rec = unbox struct { key : I64, tag : Array I64 };

keys : Array Rec -> Array I64;
keys = |arr| arr.map(|x| x.@key);

tags : Array Rec -> Array I64;
tags = |arr| arr.map(|x| x.@tag.@(0));

main : IO () = (
    // Sorting an array the caller keeps: the sort must not write into the caller's array.
    let arr = Array::from_map(40, |i| Rec { key : (i * 7) % 40, tag : [i] });
    let sorted = arr.sort_stable_by(|(a, b)| a.@key < b.@key);
    assert_eq(|_|"sorted keys", keys(sorted), Array::from_map(40, |i| i));;
    assert_eq(|_|"source keys intact", keys(arr), Array::from_map(40, |i| (i * 7) % 40));;

    // Sorting a uniquely owned array, which is where the writes go in place.
    let sorted = Array::from_map(33, |i| Rec { key : (i * 5) % 33, tag : [i] })
        .sort_stable_by(|(a, b)| a.@key < b.@key);
    assert_eq(|_|"unique sorted keys", keys(sorted), Array::from_map(33, |i| i));;

    // Equal keys keep their input order, which is what the merge's tie-breaking has to preserve.
    let dup = Array::from_map(24, |i| Rec { key : i % 2, tag : [i] });
    let sorted = dup.sort_stable_by(|(a, b)| a.@key < b.@key);
    assert_eq(|_|"stable tags", tags(sorted), Array::from_map(24, |i| if i < 12 { i * 2 } else { (i - 12) * 2 + 1 }));;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies `sort_stable_by` over a global array of boxed elements, whose storage and elements
    /// are the one thing outside the local state that a locality annotation can meet: the merge
    /// moves elements it must not treat as local, and the global itself is left as it was.
    #[test]
    pub fn test_sort_stable_of_a_global() {
        let source = r#"
module Main;

// Long enough that sorting merges the range rather than sorting it by insertion.
scrambled : Array (Array I64);
scrambled = Array::from_map(20, |i| [(i * 7) % 20]);

// Already in order, so that every merge copies its two runs instead of comparing them.
ordered : Array (Array I64);
ordered = Array::from_map(20, |i| [i]);

heads : Array (Array I64) -> Array I64;
heads = |arr| arr.map(|x| x.@(0));

main : IO () = (
    assert_eq(|_|"a global sorted", heads(scrambled.sort_stable_by(|(a, b)| a.@(0) < b.@(0))),
        Array::from_map(20, |i| i));;
    assert_eq(|_|"an ordered global sorted", heads(ordered.sort_stable_by(|(a, b)| a.@(0) < b.@(0))),
        Array::from_map(20, |i| i));;
    assert_eq(|_|"the globals are intact", (heads(scrambled), heads(ordered)),
        (Array::from_map(20, |i| (i * 7) % 20), Array::from_map(20, |i| i)));;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies `sort_stable_by` in a build where every object is out of the local state, so that
    /// the merge's reads and writes go through the multi-threaded reference counting.
    #[test]
    pub fn test_sort_stable_when_threaded() {
        let source = r#"
module Main;

heads : Array (Array I64) -> Array I64;
heads = |arr| arr.map(|x| x.@(0));

main : IO () = (
    // Long enough that sorting merges the range rather than sorting it by insertion.
    let scrambled = Array::from_map(20, |i| [(i * 7) % 20]).mark_threaded;
    assert_eq(|_|"a threaded array sorted", heads(scrambled.sort_stable_by(|(a, b)| a.@(0) < b.@(0))),
        Array::from_map(20, |i| i));;
    assert_eq(|_|"the threaded array is intact", heads(scrambled),
        Array::from_map(20, |i| (i * 7) % 20));;

    // Already in order, so that every merge copies its two runs instead of comparing them.
    let ordered = Array::from_map(20, |i| [i]).mark_threaded;
    assert_eq(|_|"an ordered threaded array sorted", heads(ordered.sort_stable_by(|(a, b)| a.@(0) < b.@(0))),
        Array::from_map(20, |i| i));;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_threaded();
        test_source(source, config);
    }

    /// Verifies the values `_unsafe_append_capacity_unchecked` and
    /// `_unsafe_copy_capacity_bounds_unchecked` produce when called directly, over the cases their
    /// callers in the standard library do not reach: the same array as source and destination, an
    /// empty range out of a non-empty source, and a destination that already holds elements.
    #[test]
    pub fn test_range_primitives_called_directly() {
        let source = r#"
module Main;

main : IO () = (
    // The same array as the source and the destination of a copy, and of an append.
    let a = ([[1], [2], [3], [4]] : Array (Array I64)).reserve(8);
    assert_eq(|_|"self copy", a._unsafe_copy_capacity_bounds_unchecked(a, 0, 4),
        [[1], [2], [3], [4], [1], [2], [3], [4]]);;
    assert_eq(|_|"self append", a.append(a),
        [[1], [2], [3], [4], [1], [2], [3], [4]]);;

    // A copy out of a source the caller keeps.
    let src = ([[1], [2], [3]] : Array (Array I64));
    let dst = (Array::empty(3) : Array (Array I64));
    assert_eq(|_|"copy from kept src", dst._unsafe_copy_capacity_bounds_unchecked(src, 0, 3),
        [[1], [2], [3]]);;
    assert_eq(|_|"copy src intact", src, [[1], [2], [3]]);;

    // An empty range out of a non-empty source.
    let dst = (Array::empty(3) : Array (Array I64));
    assert_eq(|_|"copy empty range", dst._unsafe_copy_capacity_bounds_unchecked(src, 2, 2).@size, 0);;

    // A copy onto a destination that already holds elements.
    let dst = ([[0]] : Array (Array I64)).reserve(4);
    assert_eq(|_|"copy onto tail", dst._unsafe_copy_capacity_bounds_unchecked(src, 1, 3),
        [[0], [2], [3]]);;

    // The owning primitive with a source the caller keeps, which takes the retain-per-element path.
    let dst = ([[0]] : Array (Array I64)).reserve(4);
    assert_eq(|_|"append kept src", dst._unsafe_append_capacity_unchecked(src), [[0], [1], [2], [3]]);;
    assert_eq(|_|"append src intact", src, [[1], [2], [3]]);;

    // The owning primitive with a uniquely owned source, which moves the elements.
    let dst = ([[0]] : Array (Array I64)).reserve(4);
    assert_eq(|_|"append unique src", dst._unsafe_append_capacity_unchecked([[7], [8]]),
        [[0], [7], [8]]);;

    // The owning primitive with an empty source.
    let dst = ([[0]] : Array (Array I64)).reserve(4);
    assert_eq(|_|"append empty src", dst._unsafe_append_capacity_unchecked([]), [[0]]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }
}
