use crate::ast::name::FullName;
use crate::ast::types::{tycon, type_tyapp, type_tycon, TypeNode};
use crate::build::build_object_files::get_target_machine;
use crate::configuration::Configuration;
use crate::constants::STD_NAME;
use crate::elaboration::elaborate_via_config;
use crate::error::panic_if_err;
use crate::fixstd::builtin::{
    make_bool_ty, make_i64_ty, make_ptr_ty, make_tuple_ty, make_u16_ty, make_u32_ty, make_u64_ty,
    make_u8_ty,
};
use crate::generator::Generator;
use crate::misc::Map;
use crate::tests::test_util::test_source;
use crate::object::ty_to_object_ty;
use inkwell::context::Context;
use std::sync::Arc;

/// `Option a = union { none : (), some : a }`, applied as `Option a`.
fn option_ty(elem: Arc<TypeNode>) -> Arc<TypeNode> {
    type_tyapp(
        type_tycon(&tycon(FullName::from_strs(&[STD_NAME], "Option"))),
        elem,
    )
}

/// `Result e o = union { ok : o, err : e }`, applied as `Result e o`.
fn result_ty(err: Arc<TypeNode>, ok: Arc<TypeNode>) -> Arc<TypeNode> {
    type_tyapp(
        type_tyapp(
            type_tycon(&tycon(FullName::from_strs(&[STD_NAME], "Result"))),
            err,
        ),
        ok,
    )
}

/// The (size, alignment) in bytes of a type's in-memory (embedded) representation.
fn layout<'c, 'm>(gc: &mut Generator<'c, 'm>, ty: Arc<TypeNode>) -> (u64, u64) {
    let object_ty = ty_to_object_ty(&ty, &vec![], gc.type_env());
    let embedded_ty = object_ty.to_embedded_type(gc);
    (gc.sizeof(&embedded_ty), gc.abi_alignment(&embedded_ty))
}

/// The size and alignment of a union are those of its payload buffer plus its tag, and the buffer
/// takes the size and ABI alignment of the largest payload. A union of small or empty payloads
/// therefore stays small: `Bool` is one byte, and `Option U8` two.
#[test]
fn test_union_memory_layout() {
    let config = panic_if_err(Configuration::check_mode());
    let program = panic_if_err(elaborate_via_config(&config));
    let type_env = program.type_env().clone();
    let context = Context::create();
    let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
    let module = Generator::create_module("union_layout_test", &context, &target_machine);
    // The layouts below are read off the types alone, so this generator resolves no global and is
    // given none.
    let mut gc = Generator::new(
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

    // A union's payload buffer takes the ABI alignment of its payloads, so a small or empty
    // payload does not pad the whole union up to 8 bytes.
    //
    // `Bool` = `union { _false : (), _true : () }` — empty payload, so tag only.
    assert_eq!(layout(&mut gc, make_bool_ty()), (1, 1), "Bool");

    // `Option a` = `union { none : (), some : a }` — the empty `none` variant must not inflate
    // the buffer; the payload's own size/alignment governs it.
    assert_eq!(
        layout(&mut gc, option_ty(make_u8_ty())),
        (2, 1),
        "Option U8"
    );
    assert_eq!(
        layout(&mut gc, option_ty(make_u16_ty())),
        (4, 2),
        "Option U16"
    );
    assert_eq!(
        layout(&mut gc, option_ty(make_u32_ty())),
        (8, 4),
        "Option U32"
    );
    assert_eq!(
        layout(&mut gc, option_ty(make_u64_ty())),
        (16, 8),
        "Option U64"
    );
    assert_eq!(
        layout(&mut gc, option_ty(make_i64_ty())),
        (16, 8),
        "Option I64"
    );
    assert_eq!(
        layout(&mut gc, option_ty(make_ptr_ty())),
        (16, 8),
        "Option Ptr"
    );

    // `Result e o` — the buffer is sized/aligned to the larger of the two payloads.
    assert_eq!(
        layout(&mut gc, result_ty(make_u8_ty(), make_u8_ty())),
        (2, 1),
        "Result U8 U8"
    );
    assert_eq!(
        layout(&mut gc, result_ty(make_u16_ty(), make_u8_ty())),
        (4, 2),
        "Result U16 U8"
    );
    assert_eq!(
        layout(&mut gc, result_ty(make_i64_ty(), make_u8_ty())),
        (16, 8),
        "Result I64 U8"
    );
    assert_eq!(
        layout(&mut gc, result_ty(make_u8_ty(), make_i64_ty())),
        (16, 8),
        "Result U8 I64"
    );

    // A payload the buffer's elements do not divide: `(U32, U32, U32)` is twelve bytes at
    // alignment four, and the `I64` beside it takes the elements to eight bytes, so covering the
    // twelve takes two of them rather than the one and a half they divide into.
    assert_eq!(
        layout(
            &mut gc,
            result_ty(
                make_tuple_ty(vec![make_u32_ty(), make_u32_ty(), make_u32_ty()]),
                make_i64_ty()
            )
        ),
        (24, 8),
        "Result (U32, U32, U32) I64"
    );
}

/// A payload the union's payload buffer's elements do not divide is read back whole. The buffer
/// covers the payload in elements as wide as the widest alignment among the variants, and a buffer
/// short of the payload leaves the bytes past its end out of the value.
#[test]
fn test_a_payload_the_buffer_elements_do_not_divide_round_trips() {
    let source = r#"
        module Main;

        type Uneven = unbox union { wide : (U32, U32, U32), aligned : I64 };

        main : IO ();
        main = (
            // The variant is chosen by a condition read from the program's arguments, which the
            // compiler cannot fold away, so the payload survives into the running program.
            let args = *IO::get_args;
            let u = if args.@size > 1000 { Uneven::aligned(0) } else {
                Uneven::wide((1_U32, 2_U32, 4000000000_U32))
            };
            assert_eq(|_|"the payload read back", u.as_wide, (1_U32, 2_U32, 4000000000_U32));;
            pure()
        );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A union is larger than its largest variant, whatever that variant's size: the payload buffer
/// covers the variant, and the tag needs a byte of its own beside it.
///
/// The payload here is one byte past 2^24, the largest integer a single-precision float counts to
/// exactly. Counting the buffer's elements through such a float rounds the count down from there,
/// so a buffer that covers every smaller payload can still fall short of this one.
#[test]
fn test_union_is_larger_than_a_payload_past_the_single_precision_significand() {
    let config = panic_if_err(Configuration::check_mode());
    let program = panic_if_err(elaborate_via_config(&config));
    let type_env = program.type_env().clone();
    let context = Context::create();
    let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
    let module = Generator::create_module("union_payload_test", &context, &target_machine);
    let mut gc = Generator::new(
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

    // A pair of a type is twice its size, so pairing `U8` twenty-four times over reaches 2^24
    // bytes; the `U8` beside it makes the payload one byte more.
    let mut payload = make_u8_ty();
    for _ in 0..24 {
        payload = make_tuple_ty(vec![payload.clone(), payload]);
    }
    let payload = make_tuple_ty(vec![payload, make_u8_ty()]);

    let (payload_size, _) = layout(&mut gc, payload.clone());
    let (union_size, _) = layout(&mut gc, option_ty(payload));
    assert_eq!(
        payload_size,
        (1 << 24) + 1,
        "the payload built for this test"
    );
    assert!(
        union_size > payload_size,
        "a union over a payload of {} bytes is {} bytes",
        payload_size,
        union_size,
    );
}
