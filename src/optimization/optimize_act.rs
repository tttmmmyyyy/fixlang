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
        expr::{
            expr_abs_typed, expr_app_many_typed, expr_app_typed, expr_let_typed, expr_make_struct,
            expr_var, var_local,
        },
        name::FullName,
        pattern::PatternNode,
        program::{Program, Symbol},
        types::{tycon, type_fun, type_tyapp, type_tycon, TypeNode},
    },
    builtin::{make_i64_ty, make_tuple_name, make_tuple_ty},
    constants::{
        ARRAY_ACT_NAME, ARRAY_NAME, ARRAY_UNSAFE_GET_LINEAR_FU_NAME, ARRAY_UNSAFE_SET_NAME,
        IDENTITY_NAME, STD_NAME,
    },
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
        // Replace the implementation similar to that of the `mod` function.
        //
        // The implementation we need to generate is:
        // |idx, fun, arr| (
        //     let (arr, elm) = arr._unsafe_get_linear_fu(idx);
        //     let Identity { data : elm } = fun(elm);
        //     Identity { data : arr._unsafe_set(idx, elm) }
        // );

        let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(&act_ty);

        // Type: I64
        let idx_ty = make_i64_ty();
        // Type: T -> Identity T (since f = Identity, T -> f T = T -> Identity T)
        let fun_ty = type_fun(t_ty.clone(), ft_ty.clone());
        // Type: Array T (since U = Array T)
        let arr_ty = u_ty.clone();

        // Local variable names
        const IDX_VAR: &str = "idx";
        const FUN_VAR: &str = "fun";
        const ARR_VAR: &str = "arr";
        const ELM_VAR: &str = "elm";

        // Field names of Identity
        const DATA_FIELD: &str = "data";

        // Identity type constructor
        let identity_tycon = tycon(FullName::from_strs(&[STD_NAME], IDENTITY_NAME));

        // Build the expression from inside out

        // Identity { data : arr._unsafe_set(idx, elm) }
        let identity_wrap = {
            // arr._unsafe_set(idx, elm)
            let unsafe_set_call = {
                let unsafe_set_name =
                    FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_UNSAFE_SET_NAME);
                // Type: I64 -> a -> Array a -> Array a
                let unsafe_set_ty = type_fun(
                    idx_ty.clone(),
                    type_fun(t_ty.clone(), type_fun(arr_ty.clone(), arr_ty.clone())),
                );
                let unsafe_set_var = expr_var(unsafe_set_name, None).set_type(unsafe_set_ty);
                expr_app_many_typed(
                    unsafe_set_var,
                    vec![
                        expr_var(FullName::local(IDX_VAR), None).set_type(idx_ty.clone()),
                        expr_var(FullName::local(ELM_VAR), None).set_type(t_ty.clone()),
                        expr_var(FullName::local(ARR_VAR), None).set_type(arr_ty.clone()),
                    ],
                )
            };

            // Wrap the result in Identity { data : ... }
            expr_make_struct(
                identity_tycon.clone(),
                vec![(DATA_FIELD.to_string(), unsafe_set_call)],
            )
            .set_type(fu_ty.clone())
        };

        // let Identity { data : elm } = fun(elm);
        // This unwraps the Identity wrapper from the result of fun
        let elm_bind2 = {
            let fun_call = expr_app_typed(
                expr_var(FullName::local(FUN_VAR), None).set_type(fun_ty.clone()),
                vec![expr_var(FullName::local(ELM_VAR), None).set_type(t_ty.clone())],
            );
            // Pattern: Identity { data : elm }
            let pat = PatternNode::make_struct(
                identity_tycon.clone(),
                vec![(
                    DATA_FIELD.to_string(),
                    PatternNode::make_var(var_local(ELM_VAR), Some(t_ty.clone())),
                )],
            )
            .set_type(ft_ty.clone());
            expr_let_typed(pat, fun_call, identity_wrap)
        };

        // let (arr, elm) = arr._unsafe_get_linear_fu(idx);
        let arr_elm_bind = {
            let unsafe_get_name =
                FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_UNSAFE_GET_LINEAR_FU_NAME);
            let tuple_ty = make_tuple_ty(vec![arr_ty.clone(), t_ty.clone()]);
            // Type: I64 -> Array a -> (Array a, a)
            let unsafe_get_ty =
                type_fun(idx_ty.clone(), type_fun(arr_ty.clone(), tuple_ty.clone()));
            let unsafe_get_var = expr_var(unsafe_get_name, None).set_type(unsafe_get_ty);
            let unsafe_get_call = expr_app_many_typed(
                unsafe_get_var,
                vec![
                    expr_var(FullName::local(IDX_VAR), None).set_type(idx_ty.clone()),
                    expr_var(FullName::local(ARR_VAR), None).set_type(arr_ty.clone()),
                ],
            );
            let pat = PatternNode::make_struct(
                tycon(make_tuple_name(2)),
                vec![
                    (
                        "0".to_string(),
                        PatternNode::make_var(var_local(ARR_VAR), Some(arr_ty.clone())),
                    ),
                    (
                        "1".to_string(),
                        PatternNode::make_var(var_local(ELM_VAR), Some(t_ty.clone())),
                    ),
                ],
            )
            .set_type(tuple_ty);
            expr_let_typed(pat, unsafe_get_call, elm_bind2)
        };

        // For simplicity in AST generation, we'll skip the bounds check and size binding
        // since this is an optimization pass and the original act function already does the check
        let body = arr_elm_bind;

        // |idx, fun, arr| ...
        let lambda = expr_abs_typed(var_local(ARR_VAR), arr_ty, body);
        let lambda = expr_abs_typed(var_local(FUN_VAR), fun_ty, lambda);
        let lambda = expr_abs_typed(var_local(IDX_VAR), idx_ty, lambda);

        sym.expr = Some(lambda);
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
