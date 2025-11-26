/*
# Act Optimization

## Overview

Act-family functions refer to `Std::Array::act(i)` and struct field accessors `act_{field}`.
These functions have the type `[f : Functor] (T -> f T) -> U -> f U`.

This optimization performs the following transformations:

(1) Replace the implementation of act-family functions instantiated with `f = Std::Identity` with the implementation of mod-family functions.

(2) (Not implemented yet) Replace the implementation of act-family functions instantiated with `f = Std::Const T` with the composition of the projection from `U` to `T` (get-family functions) and the argument function.

## Notes

This optimization operates on the definitions of global symbols.
Therefore, to achieve maximum effect, it must be applied before global symbol inlining.

*/

use std::sync::Arc;

use crate::{
    ast::{
        expr::{expr_abs_many, expr_abs_typed},
        name::FullName,
        program::{Program, Symbol},
        types::{tycon, type_tyapp, type_tycon, TypeNode},
    },
    constants::{ARRAY_ACT_NAME, ARRAY_NAME, IDENTITY_NAME, STD_NAME},
};

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym);
    }
}

fn run_on_symbol(sym: &mut Symbol) {
    if is_array_act(&sym.generic_name) {
        run_on_array_act(sym);
    }
}

fn is_array_act(generic_name: &FullName) -> bool {
    generic_name == &FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_ACT_NAME)
}

fn run_on_array_act(sym: &mut Symbol) {
    // Get the type `T`
    let ty = sym.ty.clone(); // I64 -> (T -> f T) -> U -> f U
    let act_ty = ty.get_lambda_dst(); // (T -> f T) -> U -> f U

    if is_functor_identity(&act_ty) {
        // Replace the implementation with that of the `mod` function.
        todo!("")
        // expr_abs_typed(var, var_ty, val)
    }
}

// Destructure the act type `(T -> f T) -> U -> f U` into `(T, U, f T, f U)`.
fn destructure_act_ty(
    act_ty: &Arc<TypeNode>,
) -> (Arc<TypeNode>, Arc<TypeNode>, Arc<TypeNode>, Arc<TypeNode>) {
    // act_ty: (T -> f T) -> U -> f U
    let act_dst_ty = act_ty.get_lambda_dst(); // U -> f U
    let u_ty = act_dst_ty.get_lambda_srcs()[0].clone(); // U
    let fu_ty = act_dst_ty.get_lambda_dst(); // f U
    let act_src_ty = act_ty.get_lambda_srcs()[0].clone(); // T -> f T
    let t_ty = act_src_ty.get_lambda_srcs()[0].clone(); // T
    let ft_ty = act_src_ty.get_lambda_dst(); // f T
    (t_ty, u_ty, ft_ty, fu_ty)
}

// Taking type `(T -> f T) -> U -> f U`, check if `f` is `Std::Identity`.
fn is_functor_identity(act_ty: &Arc<TypeNode>) -> bool {
    let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(&act_ty);

    let id_ty = type_tycon(&tycon(FullName::from_strs(&[STD_NAME], IDENTITY_NAME)));
    let id_t_ty = type_tyapp(id_ty.clone(), t_ty);
    let id_u_ty = type_tyapp(id_ty, u_ty);

    ft_ty.to_string() == id_t_ty.to_string() && fu_ty.to_string() == id_u_ty.to_string()
}
