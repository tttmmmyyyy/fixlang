//! Deducing the trait constraints a program requires.
//!
//! A constraint holds when an instance gives it and everything that instance's context asks for
//! holds in turn. These tests fix what happens when the asking comes back to where it started, and
//! when it goes on forever.

use crate::{
    configuration::Configuration,
    tests::test_util::{run_source_assert_failed, test_source, test_source_fail},
};

#[test]
pub fn test_circular_instance_context_via_equality() {
    // `Wrap (Wrap I64) : Show` is given by the instance below, whose context asks for
    // `Held (Wrap I64) : Show`. The associated type sends that back to `Wrap (Wrap I64) : Show`,
    // which is what is being deduced, so nothing gives `Show` here.
    let source = r##"
module Main;

trait c : Holder {
    type Held c;
}
trait a : Show {
    show : a -> String;
}

type Wrap c = unbox struct { data : c };

impl Wrap c : Holder {
    type Held (Wrap c) = Wrap (Wrap c);
}

impl [c : Holder, Held c = e, e : Show] Wrap c : Show {
    show = |_| "Wrap(" + (undefined("no value") : Held c).show + ")";
}

main : IO ();
main = println(Wrap { data : Wrap { data : 42 } }.show);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Deducing it needs itself",
    );
}

#[test]
pub fn test_unsatisfiable_instance_context_via_equality() {
    // The same program with an associated type that leads out of the instance instead of back into
    // it. What the context asks for is then a constraint of its own, and no instance gives it.
    let source = r##"
module Main;

trait c : Holder {
    type Held c;
}
trait a : Show {
    show : a -> String;
}

type Wrap c = unbox struct { data : c };

impl Wrap c : Holder {
    type Held (Wrap c) = I64;
}

impl [c : Holder, Held c = e, e : Show] Wrap c : Show {
    show = |_| "Wrap(" + (undefined("no value") : Held c).show + ")";
}

main : IO ();
main = println(Wrap { data : Wrap { data : 42 } }.show);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "`Std::I64 : Main::Show` cannot be deduced",
    );
}

#[test]
pub fn test_shrinking_instance_context_via_equality() {
    // An instance of the same shape, with everything its context asks for given: deducing
    // `Wrap (Wrap I64) : Show` asks for `Wrap I64 : Show`, which asks for `I64 : Show`, which an
    // instance gives outright. A deduction that reaches the trait it started from, at a smaller
    // type each time, is what a program writes, and stays accepted.
    let source = r##"
module Main;

trait c : Holder {
    type Held c;
    to_held : c -> Held c;
}
trait a : Show {
    show : a -> String;
}

type Wrap c = unbox struct { data : c };

impl Wrap c : Holder {
    type Held (Wrap c) = Wrap c;
    to_held = |w| w;
}

impl I64 : Holder {
    type Held I64 = I64;
    to_held = |x| x;
}

impl I64 : Show {
    show = |x| x.to_string;
}

impl [c : Holder, Held c = e, e : Show] Wrap c : Show {
    show = |w| "Wrap(" + w.@data.to_held.show + ")";
}

main : IO ();
main = (
    assert_eq(|_|"", Wrap { data : Wrap { data : 42 } }.show, "Wrap(Wrap(42))");;
    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_growing_instance_context_via_equality() {
    // The associated type sends the deduction to a type one `Wrap` larger every time, so it never
    // asks for the same constraint twice. The bound on how deep a type the deduction asks about is
    // what ends it.
    let source = r##"
module Main;

trait c : Holder {
    type Held c;
}
trait a : Show {
    show : a -> String;
}

type Wrap c = unbox struct { data : c };

impl Wrap c : Holder {
    type Held (Wrap c) = Wrap (Wrap (Wrap c));
}

impl [c : Holder, Held c = e, e : Show] Wrap c : Show {
    show = |_| "Wrap(" + (undefined("no value") : Held c).show + ")";
}

main : IO ();
main = println(Wrap { data : Wrap { data : 42 } }.show);
    "##;
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    assert!(
        errmsg.contains("so the deduction does not end"),
        "the deduction that does not end went unreported:\n{}",
        errmsg
    );
    // The deduction asks about hundreds of constraints, the last of them on a type of thousands of
    // characters. What the report shows of that is a few steps, each cut short.
    assert!(
        errmsg.len() < 4000,
        "the report is {} characters long, which is more of the way than a reader can read:\n{}",
        errmsg.len(),
        errmsg
    );
}

#[test]
pub fn test_circular_instance_context_of_trait_without_members() {
    // A trait with no members: the instance says that `Foo a : Marker` holds when `Foo a : Marker`
    // holds, which gives `Foo I64 : Marker` no more than declaring no instance would.
    let source = r##"
module Main;

trait a : Marker {}

type Foo a = unbox struct { x : a };

impl [Foo a : Marker] Foo a : Marker {}

need_marker : [a : Marker] a -> I64;
need_marker = |_| 0;

main : IO ();
main = println(need_marker(Foo { x : 42 }).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Deducing it needs itself",
    );
}

#[test]
pub fn test_mutually_circular_instance_contexts() {
    // Two instances that each ask for what the other gives. The deduction comes back to where it
    // started after a turn through both, and the report names the constraint it turned through,
    // which is what leads the reader to the second instance.
    let source = r##"
module Main;

trait a : A {}
trait a : B {}

type Foo a = unbox struct { x : a };

impl [Foo a : B] Foo a : A {}
impl [Foo a : A] Foo a : B {}

need_a : [a : A] a -> I64;
need_a = |_| 0;

main : IO ();
main = println(need_a(Foo { x : 42 }).to_string);
    "##;
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    for named in [
        "Deducing it needs itself",
        "`Main::Foo Std::I64 : Main::A`",
        "`Main::Foo Std::I64 : Main::B`",
    ] {
        assert!(
            errmsg.contains(named),
            "`{}` is missing from the report:\n{}",
            named,
            errmsg
        );
    }
}

/// A constraint on a type just under the depth bound is deduced, and one past it is reported by the
/// deduction: the same bound holds for the layout of a type, and the layout is where the report
/// would come from if the deduction did not carry the bound of its own.
#[test]
pub fn test_a_predicate_at_the_depth_bound_is_deduced_and_one_past_it_is_not() {
    fn source_nesting(levels: usize) -> String {
        let mut ty = "I64".to_string();
        for _ in 0..levels {
            ty = format!("W ({})", ty);
        }
        format!(
            r##"
module Main;

type W a = unbox struct {{ x : a }};

trait a : Marker {{}}

impl I64 : Marker {{}}

impl [a : Marker] W a : Marker {{}}

need_marker : [a : Marker] a -> I64;
need_marker = |_| 7;

deep : {} -> I64;
deep = |w| need_marker(w);

main : IO ();
main = (
    let argc = *IO::get_arg_count;
    let n = if argc < 0 {{ deep(undefined("no value")) }} else {{ 7 }};
    assert_eq(|_|"", n, 7);;
    pure()
);
            "##,
            ty
        )
    }
    test_source(&source_nesting(490), Configuration::develop_mode());
    test_source_fail(
        &source_nesting(510),
        Configuration::develop_mode(),
        "so the deduction does not end",
    );
}

/// A circle is reported the same way when the constraint settles only after the whole definition
/// has been checked, which is a report the type checker builds in another place.
#[test]
pub fn test_circular_instance_context_settled_at_the_end_of_a_definition() {
    let source = r##"
module Main;

trait a : Marker {}

type Foo a = unbox struct { x : a };

impl [Foo a : Marker] Foo a : Marker {}

need_marker : [a : Marker] a -> I64;
need_marker = |_| 0;

g : I64 -> I64;
g = |_| need_marker(Foo { x : 42 });

main : IO ();
main = println(g(0).to_string);
    "##;
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    for named in [
        "`Main::Foo Std::I64 : Main::Marker`",
        "Deducing it needs itself",
    ] {
        assert!(
            errmsg.contains(named),
            "`{}` is missing from the report:\n{}",
            named,
            errmsg
        );
    }
}

#[test]
pub fn test_circular_instance_context_of_associated_type() {
    // A trait whose only member is an associated type. The instance gives the associated type
    // outright, but the constraint that lets the program name it is deduced from itself.
    let source = r##"
module Main;

trait a : Holder {
    type Held a;
}

type Foo a = unbox struct { x : a };

impl [Foo a : Holder] Foo a : Holder {
    type Held (Foo a) = I64;
}

pick : [a : Holder, Held a = h] a -> h -> h;
pick = |_, h| h;

main : IO ();
main = println(pick(Foo { x : 1 }, 5).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Deducing it needs itself",
    );
}
