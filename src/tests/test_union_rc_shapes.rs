// Reference counting around unions that the RC IR rewrites: an operand a rewrite substitutes and
// then nobody reads, an unboxed union nested inside unboxed aggregates, and a union built out of a
// payload whose root is not one reference-counting unit.

#[cfg(test)]
mod union_rc_shapes_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    #[test]
    pub fn test_discarded_operand_released_once() {
        // Matching a union built on the spot lets the simplifier replace the match with the arm it
        // knows is taken, substituting the payload the construction was given. Where the arm then
        // ignores that payload — an arm binding nothing, a pattern binding a field the body never
        // reads — the substituted operand is left with no reader, and its reference has to be
        // released exactly once all the same. Run under memcheck.
        let source = r#"
            module Main;

            type Payload = box union { arr : Array I64, txt : String };
            type Fields = struct { a : Array I64, b : String, c : I64 };

            main : IO ();
            main = (
                // The arm ignores the payload the construction was given.
                let discarded = match Payload::arr([1, 2, 3]) {
                    arr(_) => 10,
                    txt(_) => -1
                };
                assert_eq(|_|"payload discarded", discarded, 10);;

                // The arm reads the payload twice.
                let read_twice = match Payload::arr([1, 2, 3, 4]) {
                    arr(x) => x.@size + x.@(0),
                    txt(_) => -1
                };
                assert_eq(|_|"payload read twice", read_twice, 5);;

                // A catch-all binds the union built on the spot, and matches it a second time.
                let rematched = match Payload::txt("abcd") {
                    arr(x) => x.@size,
                    other => (
                        match other {
                            arr(_) => -1,
                            txt(t) => t.@size
                        }
                    )
                };
                assert_eq(|_|"catch-all over a known constructor", rematched, 4);;

                // A struct literal destructured with a boxed field the body never reads.
                let Fields { a : _unread, b : text, c : count } = Fields {
                    a : [7, 8, 9], b : "xy", c : 5
                };
                assert_eq(|_|"field discarded", text.@size + count, 7);;

                // A tuple built and destructured in one step.
                let (numbers, letters) = ([1, 2], "zzz");
                assert_eq(|_|"tuple destructured", numbers.@size + letters.@size, 5);;

                pure()
            );
        "#;
        test_source(&source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_nested_unboxed_union_shared_mod() {
        // An unboxed union is one reference-counting unit whose root is where its count is kept,
        // while the references it holds live inside its variants and differ in shape from one
        // variant to the next. Nesting such a union inside a struct and a tuple, and then modifying
        // an array of them while a second binding keeps the array shared, makes the copy the
        // modification takes reach those units through two levels of unboxed aggregate. Run under
        // memcheck.
        let source = r#"
            module Main;

            type Payload = union { arr : Array I64, txt : String, num : I64 };
            type Slot = struct { p : Payload, n : I64 };
            type Pair = struct { left : Slot, right : (Payload, Payload) };

            size_of : Payload -> I64;
            size_of = |p| match p {
                arr(a) => a.@size,
                txt(t) => t.@size,
                num(i) => i
            };

            // Cycles through the three variants, so that the units beneath one union's root differ
            // from element to element.
            mk : I64 -> Payload;
            mk = |k| (
                if k % 3 == 0 { Payload::arr(Array::fill(k + 1, k)) };
                if k % 3 == 1 { Payload::txt(k.to_string) };
                Payload::num(k)
            );

            total : Pair -> I64;
            total = |t| (
                let Pair { right : r, left : l } = t;
                let Slot { p : lp, n : ln } = l;
                let (r0, r1) = r;
                size_of(lp) + ln + size_of(r0) + size_of(r1)
            );

            main : IO ();
            main = (
                let pairs = Array::from_map(9, |k| Pair {
                    left : Slot { p : mk(k), n : k },
                    right : (mk(k + 1), mk(k + 2))
                });
                assert_eq(
                    |_|"nested unions summed",
                    pairs.to_iter.fold(0, |t, acc| acc + total(t)), 145
                );;

                // A second binding keeps the array shared, so the modification copies rather than
                // writing in place.
                let kept = pairs;
                let modified = pairs.mod(0, |t| t.set_left(Slot { p : Payload::num(100), n : 1 }));
                assert_eq(|_|"mutated copy", total(modified.@(0)), 104);;
                assert_eq(|_|"original element untouched", total(kept.@(0)), 4);;
                assert_eq(
                    |_|"original array untouched",
                    kept.to_iter.fold(0, |t, acc| acc + total(t)), 145
                );;
                pure()
            );
        "#;
        test_source(&source, Configuration::develop_mode());
    }

    // An unboxed union is one reference-counting unit, kept at its root, and building one lays the
    // payload it is given in place. Where that payload's own root is not a single unit — a pair of
    // boxed values, or an unboxed struct whose one boxed field sits a level down — the union's root
    // and the payload's units are different objects, and the count of each has to be kept where its
    // own object is.
    //
    // Each union below is read through a call that stays out of line, and read once more after that
    // call returns, so that it reaches reference counting instead of being folded into the
    // constructor that built it. The payload it was built from stays live beside it and is read
    // back at the end. Freeing that payload early changes the answer for the pair; for the shape
    // whose unit lies below its root the answer stays right either way, and what catches a key made
    // for it is the assertion in `unit_of`, which the development-mode build these tests run under
    // has in place.
    const UNION_PAYLOAD_UNITS_SOURCE: &str = r#"
module Main;

// A boxed value a payload carries, so that the payload holds a reference count.
type Guard = box struct { allowed : Array I64 };

// A payload whose reference-counting unit lies below its root.
type One = unbox struct { only : Guard };

// The `pair` payload holds two units, the `one` payload holds a unit below its root, and `mark`
// holds none.
type Action = unbox union { pair : (Array I64, Array I64), one : One, mark : I64 };

// Reads the union without consuming it. The recursion keeps the call out of line.
peek : Action -> I64 -> I64;
peek = |action, n| (
    if n == 0 { if action.is_pair { 1 } else { 0 } };
    peek(action, n - 1)
);

// Builds the union out of a pair that stays live beside it, then reads both back.
via_pair : (Array I64, Array I64) -> I64 -> I64;
via_pair = |both, n| (
    if n == 0 { 0 };
    let action = Action::pair(both);
    let seen = peek(action, 2);
    let tagged = if action.is_pair { 1 } else { 0 };
    let (first, second) = both;
    seen + tagged + first.@size * 100 + first.@(0) + second.@size * 100 + second.@(0)
);

// The same for a payload whose unit lies below its root.
via_one : Guard -> I64 -> I64;
via_one = |guard, n| (
    if n == 0 { 0 };
    let action = Action::one(One { only : guard });
    let seen = peek(action, 2);
    let tagged = if action.is_one { 1 } else { 0 };
    seen + tagged + guard.@allowed.@size * 10 + guard.@allowed.@(0)
);

main : IO ();
main = (
    assert_eq(
        |_|"the pair is read back as it was built",
        via_pair((Array::fill(3, 7), Array::fill(4, 9)), 1), 718
    );;
    assert_eq(
        |_|"the boxed value is read back as it was built",
        via_one(Guard { allowed : [5, 6] }, 1), 26
    );;
    pure()
);
"#;

    /// Both payloads are read back as they were built. A pair freed while the union still holds it
    /// changes the answer, so this catches it without Valgrind.
    #[test]
    pub fn test_union_payload_units_correctness() {
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        test_source(UNION_PAYLOAD_UNITS_SOURCE, config);
    }

    /// The boxed values the payloads carry are freed exactly once and none of them leaks, checked
    /// under Valgrind MemCheck.
    #[test]
    pub fn test_union_payload_units_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        test_source(UNION_PAYLOAD_UNITS_SOURCE, Configuration::develop_mode());
    }
}
