use crate::build::build_object_files::get_target_machine;
use crate::configuration::{Configuration, FixOptimizationLevel};
use crate::constants::MAX_SPLIT_SCALARS;
use crate::elaboration::elaborate_via_config;
use crate::error::panic_if_err;
use crate::generator::Generator;
use crate::misc::Map;
use crate::tests::test_util::{run_source_capture, test_source};
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
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
        Default::default(),
        Default::default(),
        Default::default(),
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
/// A `cu_size` of 1 puts each of the program's entries in a compilation unit of its own, so every
/// call the program makes crosses a unit boundary.
#[test]
fn test_wide_value_crosses_compilation_units() {
    let mut config = Configuration::develop_mode();
    config.set_fix_opt_level(FixOptimizationLevel::Basic);
    config.cu_size = 1;
    test_source(CROSS_UNIT_PROGRAM, config);
}

// A type wider than the split limit, built by nesting an unbox struct of two fields eight levels
// deep, so `L8` holds 256 scalars under the default limit. The program carries it every way the two
// representations differ: it reads a field of a field, modifies a deep one, passes one across a
// function boundary and returns it, merges two at an `if`, carries one around a loop, stores one in
// an array, wraps one in a union variant, captures one in a closure, and reaches one through a
// boxed owner.
const WIDE_STRUCT_PROGRAM: &str = r#"
module Main;

type L0 = unbox struct { v : I64 };
type L1 = unbox struct { a : L0, b : L0 };
type L2 = unbox struct { a : L1, b : L1 };
type L3 = unbox struct { a : L2, b : L2 };
type L4 = unbox struct { a : L3, b : L3 };
type L5 = unbox struct { a : L4, b : L4 };
type L6 = unbox struct { a : L5, b : L5 };
type L7 = unbox struct { a : L6, b : L6 };
type L8 = unbox struct { a : L7, b : L7 };

type Wrapped = unbox union { none : (), some : L8 };
type Owner = box struct { w : L8 };

sum0 : L0 -> I64;
sum0 = |x| x.@v;
sum1 : L1 -> I64;
sum1 = |x| sum0(x.@a) + sum0(x.@b);
sum2 : L2 -> I64;
sum2 = |x| sum1(x.@a) + sum1(x.@b);
sum3 : L3 -> I64;
sum3 = |x| sum2(x.@a) + sum2(x.@b);
sum4 : L4 -> I64;
sum4 = |x| sum3(x.@a) + sum3(x.@b);
sum5 : L5 -> I64;
sum5 = |x| sum4(x.@a) + sum4(x.@b);
sum6 : L6 -> I64;
sum6 = |x| sum5(x.@a) + sum5(x.@b);
sum7 : L7 -> I64;
sum7 = |x| sum6(x.@a) + sum6(x.@b);
sum8 : L8 -> I64;
sum8 = |x| sum7(x.@a) + sum7(x.@b);

mk0 : I64 -> L0;
mk0 = |n| L0 { v : n };
mk1 : I64 -> L1;
mk1 = |n| L1 { a : mk0(n), b : mk0(n + 1) };
mk2 : I64 -> L2;
mk2 = |n| L2 { a : mk1(n), b : mk1(n + 2) };
mk3 : I64 -> L3;
mk3 = |n| L3 { a : mk2(n), b : mk2(n + 4) };
mk4 : I64 -> L4;
mk4 = |n| L4 { a : mk3(n), b : mk3(n + 8) };
mk5 : I64 -> L5;
mk5 = |n| L5 { a : mk4(n), b : mk4(n + 16) };
mk6 : I64 -> L6;
mk6 = |n| L6 { a : mk5(n), b : mk5(n + 32) };
mk7 : I64 -> L7;
mk7 = |n| L7 { a : mk6(n), b : mk6(n + 64) };
mk8 : I64 -> L8;
mk8 = |n| L8 { a : mk7(n), b : mk7(n + 128) };

// Add one to the leftmost leaf, reaching it through eight nested modifications.
bump : L8 -> L8;
bump = |x| x.mod_a(|y| y.mod_a(|y| y.mod_a(|y| y.mod_a(|y| y.mod_a(
    |y| y.mod_a(|y| y.mod_a(|y| y.mod_a(|z| z.set_v(z.@v + 1)))))))));

pick : Bool -> L8 -> L8 -> L8;
pick = |c, x, y| if c { x } else { y };

unwrapped : Wrapped -> I64;
unwrapped = |w| if w.is_none { -1 } else { sum8(w.as_some) };

captured : L8 -> (I64 -> I64);
captured = |x| |k| sum8(x) + k;

main : IO ();
main = (
    let x = mk8(0);
    let y = loop((0, x), |(i, v)|
        if i == 5 { break $ v } else { continue $ (i + 1, bump(v)) }
    );
    let owner = Owner { w : y };
    let arr = Array::fill(3, x).set(1, y);
    let total = sum8(x)
        + sum8(y)
        + sum8(pick(true, x, y))
        + sum8(pick(false, x, y))
        + arr.to_iter.map(sum8).fold(0, |e, acc| acc + e)
        + unwrapped(Wrapped::some(y))
        + unwrapped(Wrapped::none())
        + captured(y)(1000)
        + sum8(owner.@w)
        + y.@a.@a.@a.@a.@a.@a.@a.@a.@v
        + y.@b.@b.@b.@b.@b.@b.@b.@b.@v;
    println(total.to_string)
);
"#;

/// Verifies that a value of a type wider than the split limit computes the same answers as the
/// narrow types the rest of the suite uses.
///
/// The limit sits above every type the suite writes, so this is the only test that reaches the
/// carried-whole representation as a released compiler selects it; the tests above reach it by
/// lowering the limit instead.
#[test]
fn test_wide_value_under_the_default_limit() {
    // The leaves of `mk8(0)` are `0` to `255`, so each unmodified value sums to 32640, and each
    // value the loop bumped five times sums to 32645. `total` adds four unmodified values (`x`,
    // `pick(true, ..)`, and two array elements) and six bumped ones (`y`, `pick(false, ..)`, one
    // array element, the union payload, the closure's capture and the owner's field), then the
    // closure's `1000`, the `-1` of the empty variant, and the two extreme leaves of `y`, which
    // are `5` and `255`.
    let expected = 4 * 32640 + 6 * 32645 + 1000 - 1 + 5 + 255;
    let output = run_source_capture(WIDE_STRUCT_PROGRAM, Configuration::develop_mode());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected.to_string(),
        "a value wider than the split limit computed the wrong answer"
    );
}

// A program that merges values of one unbox struct across the arms of an `if` whose result the
// continuation reads, so the arms meet at a phi rather than at a return, and that reads fields of
// the merged value afterwards. The struct holds a boxed subobject as well, so reference counting
// runs over the merged value.
const BRANCH_MERGE_PROGRAM: &str = r#"
module Main;

type Pair = unbox struct { a : I64, b : I64 };
type Quad = unbox struct { p : Pair, q : Pair, xs : Array I64 };

pick : Bool -> Quad -> Quad -> Quad;
pick = |c, x, y| (
    let v = if c { x } else { y };
    v.mod_p(|p| p.set_a(p.@a + 100))
);

total : Quad -> I64;
total = |v| v.@p.@a + v.@p.@b * 3 + v.@q.@a * 5 + v.@q.@b * 7
    + v.@xs.to_iter.fold(0, |e, acc| acc * 11 + e);

main : IO ();
main = (
    let x = Quad { p : Pair { a : 1, b : 2 }, q : Pair { a : 3, b : 4 }, xs : [5, 6] };
    let y = Quad { p : Pair { a : 10, b : 20 }, q : Pair { a : 30, b : 40 }, xs : [50, 60] };
    println((total(pick(true, x, y)) * 13 + total(pick(false, x, y))).to_string)
);
"#;

/// Verifies that a value merged across the arms of a branch computes the same answer whether it is
/// carried whole -- where the merge is one phi of the aggregate -- or split, where it is one phi per
/// scalar.
#[test]
fn test_carried_whole_value_merged_at_a_branch() {
    let split_output = run_source_capture(BRANCH_MERGE_PROGRAM, Configuration::develop_mode());
    let mut whole_config = Configuration::develop_mode();
    whole_config.max_split_scalars = 0;
    let whole_output = run_source_capture(BRANCH_MERGE_PROGRAM, whole_config);

    assert_eq!(
        String::from_utf8_lossy(&split_output.stdout).trim(),
        "3953",
        "the split representation computed the wrong answer"
    );
    assert_eq!(
        String::from_utf8_lossy(&whole_output.stdout),
        String::from_utf8_lossy(&split_output.stdout),
        "carrying the merged value whole changed the answer"
    );
}

/// Verifies that the parts of a struct are exactly its fields' parts laid end to end: `part_count`
/// is the length of `type_parts`, and the ranges `field_part_range` gives the fields tile that list
/// in order, without gap or overlap. Checked on both sides of the limit, so it covers the claim the
/// descent rests on -- a field of a type within the limit is within it too, so a split struct has
/// no field carried whole.
#[test]
fn test_field_part_ranges_tile_the_part_list() {
    let config = panic_if_err(Configuration::check_mode());
    let program = panic_if_err(elaborate_via_config(&config));
    let type_env = program.type_env().clone();
    let context = Context::create();
    let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
    let module = Generator::create_module("part_range_test", &context, &target_machine);
    // The part lists below are read off the types alone, so this generator resolves no global and
    // is given none.
    let gc = Generator::new(
        &context,
        &module,
        target_machine.get_target_data(),
        config.clone(),
        type_env,
        Arc::new(Map::default()),
        Default::default(),
        Default::default(),
        Default::default(),
    );

    // A struct of `n` scalars of alternating class, led by a zero-sized member that yields no part.
    let mixed = |n: usize| {
        let mut fields: Vec<BasicTypeEnum> = vec![context.i8_type().array_type(0).into()];
        for i in 0..n {
            fields.push(if i % 2 == 0 {
                context.i64_type().into()
            } else {
                context.f64_type().into()
            });
        }
        context.struct_type(&fields, false)
    };

    let limit = MAX_SPLIT_SCALARS;
    // `outer` holds `2 * n` scalars, so `n` around half the limit puts it on either side of it.
    for n in [
        0,
        1,
        2,
        (limit / 2).saturating_sub(1),
        limit / 2,
        limit / 2 + 1,
        limit,
    ] {
        let outer = context.struct_type(&[mixed(n).into(), mixed(n).into()], false);
        let parts = gc.type_parts(outer.into());
        assert_eq!(gc.part_count(outer.into()), parts.len());
        if gc.is_carried_whole(outer.into()) {
            assert_eq!(parts.len(), 1);
            continue;
        }
        let mut offset = 0;
        for i in 0..outer.count_fields() {
            let field_ty = outer.get_field_type_at_index(i).unwrap();
            assert!(!gc.is_carried_whole(field_ty));
            let (off, cnt) = gc.field_part_range(outer, i);
            assert_eq!(off, offset);
            assert_eq!(cnt, gc.part_count(field_ty));
            assert_eq!(&parts[off..off + cnt], gc.type_parts(field_ty).as_slice());
            offset += cnt;
        }
        assert_eq!(offset, parts.len());
    }
}
