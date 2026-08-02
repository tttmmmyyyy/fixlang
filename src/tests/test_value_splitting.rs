use crate::build::build_object_files::get_target_machine;
use crate::configuration::Configuration;
use crate::constants::MAX_SPLIT_SCALARS;
use crate::elaboration::elaborate_via_config;
use crate::error::panic_if_err;
use crate::generator::Generator;
use crate::misc::Map;
use crate::tests::test_util::run_source_capture;
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
