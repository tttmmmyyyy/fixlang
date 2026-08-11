//! Deducing the trait constraints a program requires.
//!
//! A constraint holds when an instance gives it and everything that instance's context asks for
//! holds in turn. These tests fix what happens when the asking comes back to where it started, and
//! when it goes on forever.

use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
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
pub fn test_unfounded_instance_context_via_equality() {
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
pub fn test_nested_instance_context_via_equality() {
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
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "so the deduction does not end",
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
    // started after a turn through both.
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
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Deducing it needs itself",
    );
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
