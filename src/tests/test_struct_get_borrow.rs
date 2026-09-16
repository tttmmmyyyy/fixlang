// Memory-safety tests for a field getter reading a reference-bearing field out of a boxed
// container. Such a read borrows the container: the field is moved out and retained, so the value
// read out holds a reference of its own, and reference-count insertion releases the container at
// its last use. The container's fields cover each shape whose retain has somewhere to reach -- a
// boxed value, an unbox struct with two boxed leaves, a union with a boxed variant, and a closure
// -- alongside one holding no reference at all. An unboxed container is the exception: its fields
// are its references, so a read takes it over, and the last arm below is that shape.
//
// A parameter a function only field-reads is borrowable for that reason alone, so such a function
// gets a borrowing version and calls are routed to it. The second source below is where a container
// reaches borrow-ification and cancellation in that shape: it takes a boxed container through a
// routed call, a tail call passing an owned argument, a match whose arms join, a recursive walk, a
// closure capture, and an indirect call.

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

    const BORROWED_CONTAINER_SOURCE: &str = r#"
module Main;

type Leaf = box struct { v : I64 };
type Pair = struct { left : Leaf, right : Leaf };
type Choice = union { held : Leaf, plain : I64 };

// A boxed container whose fields cover every shape a read of one meets.
type Node = box struct {
    arr : Array I64,
    leaf : Leaf,
    pair : Pair,
    choice : Choice,
    step : I64 -> I64,
    tag : I64
};

type Chain = box struct { head : Node, rest : Option Chain };

make_node : I64 -> Node;
make_node = |base| Node {
    arr : Array::from_map(4, |i| i + base),
    leaf : Leaf { v : base },
    pair : Pair { left : Leaf { v : base }, right : Leaf { v : base + 10 } },
    choice : Choice::held(Leaf { v : base + 20 }),
    step : |k| k + base,
    tag : base
};

// `b` is only field-read, so borrow inference can make it `Borrow` only under the rule that a read
// out of a boxed container borrows it. `w` is what the caller uses after the call, so routing this
// call to the borrow version saves a retain.
read_only : I64 -> Node -> Node -> I64;
read_only = |n, b, w| (
    if n > 0 { read_only(n - 1, b, w) };
    b.@arr.@(0) + b.@leaf.@v + b.@pair.@left.@v + b.@choice.as_held.@v + (b.@step $ 1) + b.@tag
);

// The read's result is handed to a call at an owning position, out of a borrowed container.
hand_over : Node -> Array I64;
hand_over = |b| b.@arr.append([100]);

// One arm reads the container and the other returns a fresh value, so the two join.
pick : Bool -> Node -> Leaf;
pick = |take, b| if take { b.@leaf } else { Leaf { v : 0 } };

// A recursive walk over a boxed list by field reads alone: the parameter is borrowable only under
// the new rule, and the recursion is what the ownership fixed point has to settle.
walk : Chain -> I64;
walk = |c| (
    let here = c.@head.@tag;
    match c.@rest {
        Option::none(_) => here,
        Option::some(next) => here + walk(next)
    }
);

// Mutual recursion, so the fixed point has to settle two functions at once.
even_sum : I64 -> Node -> I64;
even_sum = |n, b| if n == 0 { b.@tag } else { odd_sum(n - 1, b) };

odd_sum : I64 -> Node -> I64;
odd_sum = |n, b| if n == 0 { b.@leaf.@v } else { even_sum(n - 1, b) };

// A tail call passing an owned argument: routing it to a borrow version would put a release after a
// tail call, so the routing test has to refuse it.
tail_owned : Node -> Node -> I64;
tail_owned = |b, owned| read_only(0, b, owned);

// An indirect call: the callee is decided at run time, so it keeps the all-owning ABI.
via_closure : (Node -> I64) -> Node -> I64;
via_closure = |f, b| f(b);

main : IO ();
main = (
    let b = make_node(1);
    let w = make_node(2);

    // `w` is used after the call, so the call is worth routing to the borrow version.
    let r = read_only(0, b, w);
    assert_eq(|_|"read_only", r, 1 + 1 + 1 + 21 + 2 + 1);;
    assert_eq(|_|"w outlives the call", w.@tag, 2);;

    assert_eq(|_|"hand_over", hand_over(b).@(4), 100);;
    assert_eq(|_|"the container keeps its own field", b.@arr.@size, 4);;

    assert_eq(|_|"pick takes", pick(true, b).@v, 1);;
    assert_eq(|_|"pick drops", pick(false, b).@v, 0);;

    let chain = Chain {
        head : make_node(10),
        rest : Option::some(Chain { head : make_node(20), rest : Option::none() })
    };
    assert_eq(|_|"walk", walk(chain), 30);;

    assert_eq(|_|"even_sum", even_sum(4, b), 1);;
    assert_eq(|_|"odd_sum", odd_sum(4, b), 1);;

    assert_eq(|_|"tail_owned", tail_owned(b, make_node(5)), 27);;

    // A closure capturing the container reads its fields from the capture.
    let from_capture = |k| b.@leaf.@v + b.@tag + k;
    assert_eq(|_|"read out of a capture", from_capture(1), 3);;

    let args = *IO::get_args;
    let chosen : Node -> I64 = if args.@size > 100 { |x| x.@tag } else { |x| x.@leaf.@v };
    assert_eq(|_|"via_closure", via_closure(chosen, b), 1);;

    // The read is the container's last use, so the field has to carry a reference of its own out.
    let taken = (let ns = Array::from_map(3, make_node); ns.@(1).@leaf);
    assert_eq(|_|"the field outlives its container", taken.@v, 1);;

    // A value read out of a boxed container is the reader's own, so writing to it leaves the
    // container's own as it was.
    assert_eq(|_|"the field read out is written", b.@arr.set(0, 111).@(0), 111);;
    assert_eq(|_|"the container's own field is unchanged", b.@arr.@(0), 1);;

    assert_eq(|_|"the container survives every read", b.@tag, 1);;
    pure()
);
"#;

    /// A function whose boxed container a call routes to its borrowing version computes the same
    /// answers as the owning one. The borrowing version drops the release of that container, so a
    /// field read that failed to carry a reference out would read a freed value here.
    #[test]
    pub fn test_borrowed_container_correctness() {
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        test_source(BORROWED_CONTAINER_SOURCE, config);
    }

    /// The same source under Valgrind MemCheck and the develop-mode checks, which is where
    /// borrow-ification and cancellation state their own invariants: the pending-retain walk, the
    /// merge of match arms, the un-bump answer, and the levelling of ownership.
    #[test]
    pub fn test_borrowed_container_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        test_source(BORROWED_CONTAINER_SOURCE, Configuration::develop_mode());
    }
}
