// Memory-safety tests for a field getter reading a reference-bearing field out of a boxed
// container. Such a read borrows the container: the field is moved out and retained, so the value
// read out holds a reference of its own, and reference-count insertion releases the container at
// its last use rather than at the read. The container's fields cover each shape whose retain has
// somewhere to reach -- a boxed value, an unbox struct with two boxed leaves, a union with a boxed
// variant, and a closure -- beside one holding no reference at all. An unboxed container is the
// exception: its fields are its references, so a read takes it over, and the last arm below is
// that shape.

#[cfg(test)]
mod struct_get_borrow_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const STRUCT_GET_BORROW_SOURCE: &str = r#"
module Main;

// A boxed value, so that a field holding one holds a reference.
type Leaf = box struct { arr : Array I64 };

// An unbox struct with boxed leaves, so that retaining a field read out reaches every leaf.
type Pair = unbox struct { left : Leaf, right : Leaf };

// A union with a boxed variant, so that retaining a field read out dispatches on the tag.
type Choice = unbox union { held : Leaf, plain : I64 };

// The boxed container the reads below go through. Its fields cover each shape a field getter
// meets: one holding no reference, a boxed one, an unbox struct with boxed leaves, a union with a
// boxed variant, and a closure.
type Holder = box struct {
    n : I64,
    arr : Array I64,
    pair : Pair,
    choice : Choice,
    step : I64 -> I64
};

// The same read out of an unboxed container, which takes the container over instead.
type UnboxedHolder = unbox struct { arr : Array I64, n : I64 };

make_holder : I64 -> Holder;
make_holder = |base| Holder {
    n : base,
    arr : Array::from_map(4, |i| i + base),
    pair : Pair {
        left : Leaf { arr : Array::from_map(2, |i| i + base) },
        right : Leaf { arr : Array::from_map(2, |i| i + base + 10) }
    },
    choice : Choice::held(Leaf { arr : Array::from_map(2, |i| i + base + 20) }),
    step : |k| k + base
};

// Reads every reference-bearing field of the container once a round without consuming it, so each
// read has to leave the container alive for the rounds that follow.
read_each_round : I64 -> Holder -> I64;
read_each_round = |rounds, h| (
    Iterator::range(0, rounds).fold(0, |_, acc|
        acc + h.@arr.@(0) + h.@pair.@left.@arr.@(0) + h.@choice.as_held.@arr.@(0) + (h.@step $ 1)
    )
);

main : IO ();
main = (
    let h = make_holder(3);

    // A field read out of a boxed container is a reference of the reader's own, so writing to it
    // copies instead of writing through to the container's own.
    assert_eq(|_|"the field read out is written", h.@arr.set(0, 111).@(0), 111);;
    assert_eq(|_|"the container's own field is unchanged", h.@arr.@(0), 3);;
    assert_eq(|_|"the leaf of an unbox struct field is written",
        h.@pair.@left.@arr.set(0, 222).@(0), 222);;
    assert_eq(|_|"the container's own leaf is unchanged", h.@pair.@left.@arr.@(0), 3);;
    assert_eq(|_|"the payload of a union field is written",
        h.@choice.as_held.@arr.set(0, 333).@(0), 333);;
    assert_eq(|_|"the container's own payload is unchanged", h.@choice.as_held.@arr.@(0), 23);;

    // The reads leave the container usable, round after round.
    assert_eq(|_|"the container is read once a round", read_each_round(5, h), 165);;
    assert_eq(|_|"the container outlives the reads", h.@n, 3);;

    // The read is the container's last use, so whoever owns it releases it right after. The field
    // has to carry a reference of its own out, or it is read back after the container freed it.
    let taken = (
        let holders = Array::from_map(3, make_holder);
        holders.@(1).@arr
    );
    assert_eq(|_|"the field outlives the container it was read out of", taken.@(0), 1);;

    // An unboxed container's read takes the container over, and hands out a field just as usable.
    let p = UnboxedHolder { arr : Array::from_map(4, |i| i * 2), n : 9 };
    assert_eq(|_|"the field of an unboxed container is written", p.@arr.set(0, 444).@(0), 444);;
    assert_eq(|_|"the unboxed container's own field is unchanged", p.@arr.@(0), 0);;

    pure()
);
"#;

    /// A value read out of a boxed container is the reader's own, so writing to it leaves the
    /// container's own copy as it was. A field handed out without a reference of its own is written
    /// through instead, which these assertions catch without Valgrind.
    #[test]
    pub fn test_struct_get_borrow_correctness() {
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        test_source(STRUCT_GET_BORROW_SOURCE, config);
    }

    /// The container a read borrows is freed exactly once and leaks nothing, and a field read out
    /// at the container's last use outlives it, checked under Valgrind MemCheck.
    #[test]
    pub fn test_struct_get_borrow_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        test_source(STRUCT_GET_BORROW_SOURCE, Configuration::develop_mode());
    }
}
