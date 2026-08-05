use crate::build::build_object_files::get_target_machine;
use crate::configuration::{Configuration, FixOptimizationLevel};
use crate::constants::MAX_SPLIT_SCALARS;
use crate::elaboration::elaborate_via_config;
use crate::error::panic_if_err;
use crate::generator::Generator;
use crate::misc::Map;
use crate::tests::test_util::{run_source_capture, test_source};
use inkwell::context::Context;
use std::sync::Arc;

// A program that carries unboxed structs every way the two representations differ: it builds a
// nested one, reads and modifies a field of a field, passes one across a function boundary and
// returns one, merges two at an `if`, and carries one around a loop. It reaches a boxed subobject
// (`Array`) inside an unboxed one as well, so the reference counting runs over a carried value.
const NESTED_STRUCT_PROGRAM: &str = r#"
module Main;

type Inner = unbox struct { a : I64, b : I64, flag : Bool };
type Middle = unbox struct { x : Inner, y : Inner, xs : Array I64 };
type Outer = unbox struct { p : Middle, q : Middle };

sum_inner : Inner -> I64;
sum_inner = |v| v.@a + v.@b + (if v.@flag { 1 } else { 0 });

sum_middle : Middle -> I64;
sum_middle = |v| sum_inner(v.@x) + sum_inner(v.@y) + v.@xs.to_iter.fold(0, |e, acc| acc + e);

bump : Outer -> Outer;
bump = |v| v.mod_p(|m| m.mod_x(|i| i.set_a(i.@a + 1)));

pick : Bool -> Outer -> Middle;
pick = |b, v| if b { v.@p } else { v.@q };

main : IO ();
main = (
    let inner = Inner { a : 1, b : 2, flag : true };
    let mid = Middle { x : inner, y : Inner { a : 10, b : 20, flag : false }, xs : [1, 2, 3] };
    let outer = Outer { p : mid, q : mid };
    let outer = loop((0, outer), |(i, o)|
        if i == 5 { break $ o } else { continue $ (i + 1, bump(o)) }
    );
    let total = sum_middle(pick(true, outer)) + sum_middle(pick(false, outer));
    println(total.to_string)
);
"#;

/// Verifies that a program computes the same answer whether its unboxed values are split into
/// their scalars or carried whole.
///
/// The limit that decides between the two sits above every type the test suite writes, so lowering
/// it is what reaches the carried-whole representation at all — with it at zero, every unboxed
/// struct in the program and in `Std` is carried whole.
#[test]
fn test_wide_value_carried_whole_computes_the_same() {
    let split_output = run_source_capture(NESTED_STRUCT_PROGRAM, Configuration::develop_mode());
    let mut whole_config = Configuration::develop_mode();
    whole_config.max_split_scalars = 0;
    let whole_output = run_source_capture(NESTED_STRUCT_PROGRAM, whole_config);

    assert_eq!(
        String::from_utf8_lossy(&split_output.stdout).trim(),
        "85",
        "the split representation computed the wrong answer"
    );
    assert_eq!(
        String::from_utf8_lossy(&whole_output.stdout),
        String::from_utf8_lossy(&split_output.stdout),
        "carrying the values whole changed the answer"
    );
}

/// Verifies that the limit is the widest a value may be and still be split: a type of exactly that
/// many scalars is split into them, and one scalar more is carried as a single part.
#[test]
fn test_split_limit_boundary() {
    let config = panic_if_err(Configuration::check_mode());
    let program = panic_if_err(elaborate_via_config(&config));
    let type_env = program.type_env().clone();
    let context = Context::create();
    let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
    let module = Generator::create_module("split_limit_test", &context, &target_machine);
    // The part counts below are read off the types alone, so this generator resolves no global and
    // is given none.
    let gc = Generator::new(
        &context,
        &module,
        target_machine.get_target_data(),
        config.clone(),
        type_env,
        Arc::new(Map::default()),
    );

    let limit = MAX_SPLIT_SCALARS;
    let scalar = context.i64_type().into();
    let at_limit = context.struct_type(&vec![scalar; limit], false);
    let past_limit = context.struct_type(&vec![scalar; limit + 1], false);

    assert_eq!(gc.part_count(at_limit.into()), limit);
    assert_eq!(gc.part_count(past_limit.into()), 1);

    // The limit counts the scalars a type holds through its nesting, so a struct of two structs of
    // `limit` scalars each is one part, though it has two fields.
    let nested = context.struct_type(&[at_limit.into(), at_limit.into()], false);
    assert_eq!(gc.part_count(nested.into()), 1);
}

// A value carried whole reaches a function's parameters, its return value and its phis, and the
// unit that defines a function and the unit that calls it derive that shape apart. One symbol per
// unit puts a boundary on every call, so the two sides agree only by reading the shape off the
// type. The shapes cover a value exactly at the limit beside one past it, a carried-whole value
// returned through the out-pointer, read a field at a time, merged at a branch, carried around a
// loop, held in an array, captured by a closure, and holding a boxed subobject.
const CROSS_UNIT_PROGRAM: &str = r#"
module Main;

type W4 = unbox struct { a : I64, b : I64, c : I64, d : I64 };
type W16 = unbox struct { a : W4, b : W4, c : W4, d : W4 };
type W64 = unbox struct { a : W16, b : W16, c : W16, d : W16 };
// 128 scalars: exactly the limit, so a value of this type is still split into them.
type W128 = unbox struct { x : W64, y : W64 };
// 129 scalars: one past the limit, so a value of this type is carried whole.
type W129 = unbox struct { x : W64, y : W64, t : I64 };
// A carried-whole value holding a boxed subobject, so reference counting runs over one.
type Boxy = unbox struct { w : W129, xs : Array I64 };

mk4 : I64 -> W4;
mk4 = |n| W4 { a : n, b : n + 1, c : n + 2, d : n + 3 };
mk16 : I64 -> W16;
mk16 = |n| W16 { a : mk4(n), b : mk4(n + 4), c : mk4(n + 8), d : mk4(n + 12) };
mk64 : I64 -> W64;
mk64 = |n| W64 { a : mk16(n), b : mk16(n + 16), c : mk16(n + 32), d : mk16(n + 48) };
mk128 : I64 -> W128;
mk128 = |n| W128 { x : mk64(n), y : mk64(n + 64) };
mk129 : I64 -> W129;
mk129 = |n| W129 { x : mk64(n), y : mk64(n + 64), t : n + 128 };

sum4 : W4 -> I64;
sum4 = |v| v.@a + v.@b + v.@c + v.@d;
sum16 : W16 -> I64;
sum16 = |v| sum4(v.@a) + sum4(v.@b) + sum4(v.@c) + sum4(v.@d);
sum64 : W64 -> I64;
sum64 = |v| sum16(v.@a) + sum16(v.@b) + sum16(v.@c) + sum16(v.@d);
sum128 : W128 -> I64;
sum128 = |v| sum64(v.@x) + sum64(v.@y);
sum129 : W129 -> I64;
sum129 = |v| sum64(v.@x) + sum64(v.@y) + v.@t;

// Reads a split field out of a carried-whole value.
half : Bool -> W129 -> W64;
half = |b, v| if b { v.@x } else { v.@y };

// Modifies a field of a field, and returns the value whole.
bump129 : W129 -> W129;
bump129 = |v| v.mod_x(|s| s.mod_a(|s| s.mod_a(|s| s.set_a(s.@a + 1))));

// Merges two carried-whole values at a branch.
choose : Bool -> W129 -> W129 -> W129;
choose = |b, x, y| if b { x } else { y };

// Carries a carried-whole value around a loop.
bump_n : I64 -> W129 -> W129;
bump_n = |n, v| loop((0, v), |(i, w)|
    if i == n { break $ w } else { continue $ (i + 1, bump129(w)) }
);

mk_boxy : I64 -> Boxy;
mk_boxy = |n| Boxy { w : mk129(n), xs : [n, n + 1, n + 2] };
sum_boxy : Boxy -> I64;
sum_boxy = |v| sum129(v.@w) + v.@xs.to_iter.fold(0, |e, acc| acc + e);

// Captures a carried-whole value in a closure.
adder : W129 -> (I64 -> I64);
adder = |v| |n| n + sum129(v);

main : IO ();
main = (
    assert_eq(|_|"at the limit", sum128(mk128(0)), 8128);;
    assert_eq(|_|"past the limit", sum129(mk129(0)), 8256);;
    let w = bump_n(5, mk129(0));
    assert_eq(|_|"around a loop", sum129(w), 8261);;
    assert_eq(|_|"a field at a time", sum64(half(true, w)) + sum64(half(false, w)), 8133);;
    assert_eq(|_|"merged at a branch", sum129(choose(true, mk129(0), w)), 8256);;
    assert_eq(|_|"holding a boxed subobject", sum_boxy(mk_boxy(0)), 8259);;
    assert_eq(|_|"captured by a closure", adder(mk129(0))(7), 8263);;
    let arr = Array::from_map(3, |i| mk129(i));
    assert_eq(|_|"held in an array", arr.to_iter.map(sum129).fold(0, |e, acc| acc + e), 25155);;
    pure()
);
"#;

/// Verifies that a value carried whole reaches the same answers when the function defining it and
/// the function calling it are compiled as separate units.
///
/// Separate compilation, which `max_cu_size` divides, runs at `Basic` and below, so the level comes
/// down to it: at a higher one the whole program is one unit and no call crosses a boundary.
#[test]
fn test_wide_value_crosses_compilation_units() {
    let mut config = Configuration::develop_mode();
    config.set_fix_opt_level(FixOptimizationLevel::Basic);
    config.max_cu_size = 1;
    test_source(CROSS_UNIT_PROGRAM, config);
}
