use std::sync::Arc;

use inkwell::context::Context;

use crate::ast::name::FullName;
use crate::ast::types::{tycon, type_tyapp, type_tycon, TypeNode};
use crate::build::build_object_files::get_target_machine;
use crate::configuration::Configuration;
use crate::constants::STD_NAME;
use crate::elaboration::elaborate_via_config;
use crate::error::panic_if_err;
use crate::fixstd::builtin::{
    make_bool_ty, make_i64_ty, make_ptr_ty, make_u16_ty, make_u32_ty, make_u64_ty, make_u8_ty,
};
use crate::generator::Generator;
use crate::object::ty_to_object_ty;

fn option_ty(elem: Arc<TypeNode>) -> Arc<TypeNode> {
    type_tyapp(
        type_tycon(&tycon(FullName::from_strs(&[STD_NAME], "Option"))),
        elem,
    )
}

// `Result e o = union { ok : o, err : e }`, applied as `Result e o`.
fn result_ty(err: Arc<TypeNode>, ok: Arc<TypeNode>) -> Arc<TypeNode> {
    type_tyapp(
        type_tyapp(
            type_tycon(&tycon(FullName::from_strs(&[STD_NAME], "Result"))),
            err,
        ),
        ok,
    )
}

// The (size, alignment) in bytes of a type's in-memory (embedded) representation.
fn layout<'c, 'm>(gc: &mut Generator<'c, 'm>, ty: Arc<TypeNode>) -> (u64, u64) {
    let obj = ty_to_object_ty(&ty, &vec![], gc.type_env());
    let llvm = obj.to_embedded_type(gc, vec![]);
    (gc.sizeof(&llvm), gc.abi_alignment(&llvm))
}

#[test]
fn test_union_memory_layout() {
    let config = panic_if_err(Configuration::check_mode());
    let program = panic_if_err(elaborate_via_config(&config));
    let type_env = program.type_env().clone();
    let context = Context::create();
    let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
    let module = Generator::create_module("union_layout_test", &context, &target_machine);
    let mut gc = Generator::new(
        &context,
        &module,
        target_machine.get_target_data(),
        config.clone(),
        type_env,
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
}
