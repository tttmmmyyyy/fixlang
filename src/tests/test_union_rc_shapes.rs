//! Reference counting around unions that the RC IR rewrites: an operand a rewrite substitutes and
//! then nobody reads, an unboxed union nested inside unboxed aggregates, and a union built out of a
//! payload that holds its reference-counting units below the payload itself.

#[cfg(test)]
mod union_rc_shapes_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    /// Compile and run a source with Valgrind switched off, leaving the program's own assertions to
    /// decide the outcome.
    fn test_source_without_valgrind(source: &str) {
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        test_source(source, config);
    }

    /// Matching a union built on the spot lets the simplifier replace the match with the arm it
    /// knows is taken, substituting the payload the construction was given. Where the arm then
    /// ignores that payload — an arm binding nothing, a pattern binding a field the body never
    /// reads — the substituted operand is left with no reader, and its reference has to be released
    /// exactly once all the same. Run under memcheck.
    #[test]
    pub fn test_discarded_operand_released_once() {
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

    /// An unboxed union is one reference-counting unit, counted on the union itself, while the
    /// references it holds live inside its variants and differ in shape from one variant to the
    /// next. This nests such a union inside a struct and a tuple, then modifies an array of them
    /// while a second binding keeps that array shared. The modification copies an element, and the
    /// copy reaches those units two levels of unboxed aggregate down. Run under memcheck.
    #[test]
    pub fn test_nested_unboxed_union_shared_mod() {
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

            // Cycles through the three variants, so that the units beneath one union differ from
            // element to element.
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

    // An unboxed union is one reference-counting unit, counted on the union itself, and building one
    // lays the payload it is given in place. Where the payload's units sit below the payload itself
    // — a pair of boxed values, or an unboxed struct whose one boxed field sits a level down — the
    // union and those units are different objects, and the count of each has to be kept where its
    // own object is.
    //
    // Each union below is read through a call that stays out of line, and read once more after that
    // call returns, so that the simplifier leaves it standing as a value reference counting acts
    // on. The payload it was built from stays live beside it and is read back at the end. For the
    // shapes whose payload holds two units, freeing that payload early changes the answer. For the
    // shape whose unit sits one level down the answer stays right either way, and the assertion in
    // `unit_of` catches a key made for it. That payload arrives as a parameter, so the union is
    // resolved from a value whose own unit sits a level below it; it carries a second field so that
    // the struct around the boxed value survives to the RC IR.
    const UNION_PAYLOAD_UNITS_SOURCE: &str = r#"
module Main;

// A boxed value a payload carries, so that the payload holds a reference count.
type Guard = box struct { allowed : Array I64 };

// A payload whose one reference-counting unit sits one level down, in `held`. The second field
// keeps the struct from being unwrapped down to the boxed value it holds.
type Wrapped = unbox struct { held : Guard, tag : I64 };

// The `pair` payload holds two units, the `wrapped` payload holds one unit a level down, and
// `mark` holds none.
type Action = unbox union { pair : (Array I64, Array I64), wrapped : Wrapped, mark : I64 };

// Reads the union without consuming it. The recursion keeps the call out of line.
peek : Action -> I64 -> I64;
peek = |action, n| (
    if n == 0 { if action.is_pair { 1 } else { 0 } };
    peek(action, n - 1)
);

// Builds the union out of a pair that stays live beside it, then reads both back.
via_pair : (Array I64, Array I64) -> I64 -> I64;
via_pair = |payload, n| (
    if n == 0 { 0 };
    let action = Action::pair(payload);
    let seen_in_call = peek(action, 2);
    let seen_after_call = if action.is_pair { 1 } else { 0 };
    let (first, second) = payload;
    seen_in_call + seen_after_call
        + first.@size * 100 + first.@(0) + second.@size * 100 + second.@(0)
);

// The same for a payload whose unit sits one level down.
via_wrapped : Wrapped -> I64 -> I64;
via_wrapped = |payload, n| (
    if n == 0 { 0 };
    let action = Action::wrapped(payload);
    let seen_in_call = peek(action, 2);
    let seen_after_call = if action.is_wrapped { 1 } else { 0 };
    seen_in_call + seen_after_call + payload.@tag
        + payload.@held.@allowed.@size * 10 + payload.@held.@allowed.@(0)
);

// `Std::Option` is an unboxed union too, and a pair payload gives it the same shape.
peek_option : Option (Array I64, Array I64) -> I64 -> I64;
peek_option = |opt, n| (
    if n == 0 { if opt.is_some { 1 } else { 0 } };
    peek_option(opt, n - 1)
);

via_option : (Array I64, Array I64) -> I64 -> I64;
via_option = |payload, n| (
    if n == 0 { 0 };
    let opt = Option::some(payload);
    let seen_in_call = peek_option(opt, 2);
    let seen_after_call = if opt.is_some { 1 } else { 0 };
    let (first, second) = payload;
    seen_in_call + seen_after_call
        + first.@size * 100 + first.@(0) + second.@size * 100 + second.@(0)
);

// `Std::Result` likewise.
peek_result : Result String (Array I64, Array I64) -> I64 -> I64;
peek_result = |res, n| (
    if n == 0 { if res.is_ok { 1 } else { 0 } };
    peek_result(res, n - 1)
);

via_result : (Array I64, Array I64) -> I64 -> I64;
via_result = |payload, n| (
    if n == 0 { 0 };
    let res : Result String (Array I64, Array I64) = ok(payload);
    let seen_in_call = peek_result(res, 2);
    let seen_after_call = if res.is_ok { 1 } else { 0 };
    let (first, second) = payload;
    seen_in_call + seen_after_call
        + first.@size * 100 + first.@(0) + second.@size * 100 + second.@(0)
);

main : IO ();
main = (
    assert_eq(
        |_|"the pair is read back as it was built",
        via_pair((Array::fill(3, 7), Array::fill(4, 9)), 1), 718
    );;
    assert_eq(
        |_|"the boxed value one level below the payload is read back as it was built",
        via_wrapped(Wrapped { held : Guard { allowed : [5, 6] }, tag : 3 }, 1), 29
    );;
    assert_eq(
        |_|"the pair an option holds is read back as it was built",
        via_option((Array::fill(3, 7), Array::fill(4, 9)), 1), 718
    );;
    assert_eq(
        |_|"the pair a result holds is read back as it was built",
        via_result((Array::fill(3, 7), Array::fill(4, 9)), 1), 718
    );;
    pure()
);
"#;

    /// Every payload is read back as it was built. A pair freed while the union still holds it
    /// changes the answer, so this catches it without Valgrind.
    #[test]
    pub fn test_union_payload_units_correctness() {
        test_source_without_valgrind(UNION_PAYLOAD_UNITS_SOURCE);
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

    // Taking the payload out of one union value twice. Each extraction consumes a reference of the
    // union, so the union is retained once before the first one, and that single retain pays for
    // both. An unboxed union is one reference-counting unit, so the retain bumps every reference
    // the payload holds at once, while each extraction releases those references one field at a
    // time — the retain is un-bumped by a group of releases rather than by a single one. Cancelling
    // it against one release of that group leaves the rest standing, and the second extraction then
    // releases what the first already freed.
    //
    // The scalar the reads take is what keeps the extraction visible: reading a boxed field instead
    // consumes the payload in the read itself, leaving no release of the payload to pair with.
    //
    // The second shape builds the union out of a payload that holds one array in both of its
    // fields, so the union holds two references of one object. Counting them as one would let a
    // single release un-bump a retain that bumped both.
    const PAYLOAD_TAKEN_TWICE_SOURCE: &str = r#"
module Main;

// The `pair` payload holds two boxed values beside a scalar, and `mark` holds none.
type Action = unbox union { pair : (I64, Array I64, Array I64), mark : I64 };

// Takes the payload out of one union value twice, reading its scalar each time.
take_payload_twice : Action -> I64;
take_payload_twice = |action| (
    if action.as_pair.@0 == 0 { action.as_pair.@0 + 100 };
    -1
);

// The same, on a union built here out of a payload that holds one array in both of its fields.
take_payload_twice_of_shared : Array I64 -> I64;
take_payload_twice_of_shared = |arr| take_payload_twice(Action::pair((0, arr, arr)));

main : IO ();
main = (
    let actions = Array::from_map(4, |k| Action::pair(
        (0, Array::fill(k + 1, k), Array::fill(k + 2, k))
    ));
    assert_eq(|_|"the scalar is read twice", actions.to_iter.map(take_payload_twice).sum, 400);;
    assert_eq(
        |_|"the arrays the payload holds are read back as they were built",
        actions.to_iter.map(|a| a.as_pair.@1.@(0) + a.as_pair.@2.@(0)).sum, 12
    );;

    let shared = Array::fill(3, 5);
    assert_eq(
        |_|"the scalar of a payload holding one array twice is read twice",
        Iterator::range(0, 4).map(|_| take_payload_twice_of_shared(shared)).sum, 400
    );;
    assert_eq(|_|"the array both fields of that payload hold is read back", shared.@(0), 5);;
    pure()
);
"#;

    /// The arrays a payload taken out twice holds are read back as they were built. An array freed
    /// while the union still holds it changes the answer, so this catches it without Valgrind.
    #[test]
    pub fn test_payload_taken_twice_correctness() {
        test_source_without_valgrind(PAYLOAD_TAKEN_TWICE_SOURCE);
    }

    /// The arrays a payload taken out twice holds are freed exactly once and none of them leaks,
    /// checked under Valgrind MemCheck.
    #[test]
    pub fn test_payload_taken_twice_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        test_source(PAYLOAD_TAKEN_TWICE_SOURCE, Configuration::develop_mode());
    }

    // A union built out of a payload that holds one array in both of its fields, dropped whole
    // without its payload being taken, and the array read after that. An unboxed union is one
    // reference-counting unit, so its release un-bumps both of the references the payload holds at
    // once, while each retain that put the array there bumped one of them. A release reaching that
    // far closes no bracket: the retains it reaches past stay, and the array outlives the union.
    const UNION_DROPPED_WHOLE_SOURCE: &str = r#"
module Main;

// The `pair` payload holds two boxed values beside a scalar, and `mark` holds none.
type Action = unbox union { pair : (I64, Array I64, Array I64), mark : I64 };

// Builds a union out of a payload that holds one array in both of its fields, drops the union
// without taking its payload, and reads the array after that.
build_and_read : I64 -> I64;
build_and_read = |k| (
    let arr = Array::fill(k + 3, k);
    eval Action::pair((0, arr, arr));
    arr.@(0) + arr.@size
);

main : IO ();
main = (
    assert_eq(
        |_|"the array outlives the union that held it twice",
        Iterator::range(0, 4).map(build_and_read).sum, 24
    );;
    pure()
);
"#;

    /// The array a union held twice is read after the union is dropped whole. An array freed with
    /// the union changes the answer, so this catches it without Valgrind.
    #[test]
    pub fn test_union_dropped_whole_correctness() {
        test_source_without_valgrind(UNION_DROPPED_WHOLE_SOURCE);
    }

    /// The array a union held twice is freed exactly once and does not leak, checked under Valgrind
    /// MemCheck.
    #[test]
    pub fn test_union_dropped_whole_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        test_source(UNION_DROPPED_WHOLE_SOURCE, Configuration::develop_mode());
    }

    // A match whose catch-all arm has to dispose of the scrutinee. A sibling arm names the
    // scrutinee a second time, so the scrutinee is live at every arm's head, and the arm that does
    // not name it releases it there. In the catch-all arm that release meets a second one: the
    // payload the arm binds is the scrutinee itself, bound without a retain, and an arm body that
    // never reads the payload releases it too. The retain the match is entered with is what pays
    // for the second. One release short leaves the union alive; one too many frees it while the
    // caller still holds it.
    const CATCH_ALL_REREAD_SOURCE: &str = r#"
module Main;

type Choice = box union { arr : Array I64, num : I64 };
type Pair = unbox union { both : (Array I64, Array I64), none : I64 };

// The tagged arm reads the scrutinee a second time; the catch-all arm reads neither the scrutinee
// nor the payload it binds.
reread : Choice -> I64;
reread = |c| (
    match c {
        arr(a) => a.@(0) + (match c { arr(b) => b.@size, num(j) => j }),
        other => 100
    }
);

reread_unbox : Pair -> I64;
reread_unbox = |p| (
    match p {
        none(i) => i + (match p { both(t) => t.@0.@(0), none(j) => j }),
        other => 200
    }
);

// The catch-all arm reads the payload it binds, so the scrutinee flows into that arm as well.
reread_and_use : Choice -> I64;
reread_and_use = |c| (
    match c {
        num(i) => i + (match c { arr(b) => b.@size, num(j) => j }),
        other => (match other { arr(b) => b.@(0), num(j) => j * 3 })
    }
);

main : IO ();
main = (
    assert_eq(|_|"boxed union, tagged arm", reread(Choice::arr([11, 12, 13])), 14);;
    assert_eq(|_|"boxed union, catch-all arm", reread(Choice::num(7)), 100);;
    assert_eq(|_|"unboxed union, tagged arm", reread_unbox(Pair::none(5)), 10);;
    assert_eq(|_|"unboxed union, catch-all arm", reread_unbox(Pair::both(([1], [2]))), 200);;
    assert_eq(|_|"catch-all arm reads its payload", reread_and_use(Choice::arr([9, 8])), 9);;
    assert_eq(|_|"tagged arm rereads the scrutinee", reread_and_use(Choice::num(4)), 8);;
    pure()
);
"#;

    /// The unions are freed exactly once and none of them leaks, checked under Valgrind MemCheck,
    /// which is what a release too few shows up as.
    #[test]
    pub fn test_catch_all_arm_over_a_reread_scrutinee_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        test_source(CATCH_ALL_REREAD_SOURCE, Configuration::develop_mode());
    }

    // A union with a boxed payload sitting in a field of an unboxed struct, whose field is read
    // twice: once to test the tag, and once inside the arm that test chose. Each read takes a
    // reference of the union, so a retain pays for each one and a release disposes of each result.
    //
    // A retain and the release that un-bumps it are paired on the object they name, and the union
    // here is named two ways: as the struct's field where the retain acts, and as the value the read
    // produced where the release acts. Both have to resolve to one object. Resolving them to two
    // objects pairs the first retain with the release of the struct at the end instead, and
    // cancelling that pair leaves the read's own release to free the payload while the arm is still
    // reading it.
    //
    // The construction is written out of line from the reads, so that the two meet only where
    // inlining brings them together — a struct built and read in one function leaves the field with
    // a single reader. The union is built by an `if`, so that the field's object is one of two and
    // the resolution has to name the value that joins them.
    const FIELD_READ_TWICE_SOURCE: &str = r#"
module Main;

// A boxed value a union carries, whose scalar the reads take.
type Held = box struct { size : I64 };

// A struct whose first field is a union holding that boxed value.
type Slot = struct { held : Option Held, tag : I64 };

// The same shape with `Result`, whose payload sits in the other variant.
type Outcome = struct { done : Result I64 Held, tag : I64 };

make_slot : I64 -> Slot;
make_slot = |k| Slot {
    held : if k % 3 == 0 { Option::none() } else { Option::some(Held { size : k }) },
    tag : k
};

make_outcome : I64 -> Outcome;
make_outcome = |k| Outcome {
    done : if k % 3 == 0 { Result::err(k) } else { Result::ok(Held { size : k }) },
    tag : k
};

// Reads the field twice: once to test the tag, once inside the arm that test chose.
read_slot : Slot -> I64;
read_slot = |s| (if s.@held.is_none { 0 } else { s.@held.as_some.@size }) + s.@tag;

read_outcome : Outcome -> I64;
read_outcome = |o| (if o.@done.is_err { 0 } else { o.@done.as_ok.@size }) + o.@tag;

// A third read, one arm deeper than the second.
read_slot_nested : Slot -> I64;
read_slot_nested = |s| (
    if s.@held.is_none { 0 } else {
        if s.@held.as_some.@size == 0 { 0 } else { s.@held.as_some.@size * 10 }
    }
) + s.@tag;

main : IO ();
main = (
    assert_eq(
        |_|"the option in a struct field is read twice",
        Iterator::range(0, 6).map(|k| read_slot(make_slot(k))).sum, 27
    );;
    assert_eq(
        |_|"the result in a struct field is read twice",
        Iterator::range(0, 6).map(|k| read_outcome(make_outcome(k))).sum, 27
    );;
    assert_eq(
        |_|"the option in a struct field is read three times, each read an arm deeper",
        Iterator::range(0, 6).map(|k| read_slot_nested(make_slot(k))).sum, 135
    );;
    pure()
);
"#;

    /// Each read of the field gets the payload as it was built. A payload freed while the struct
    /// still holds it changes the scalar the next read takes, so this catches it without Valgrind.
    #[test]
    pub fn test_field_read_twice_correctness() {
        test_source_without_valgrind(FIELD_READ_TWICE_SOURCE);
    }

    /// The payloads are freed exactly once and none of them leaks, checked under Valgrind MemCheck,
    /// which is what the read of a freed payload and the second free of it show up as.
    #[test]
    pub fn test_field_read_twice_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        test_source(FIELD_READ_TWICE_SOURCE, Configuration::develop_mode());
    }
}
