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

use std::{collections::HashMap, mem::replace, sync::Arc};

use crate::{
    ast::{
        expr::expr_var,
        name::FullName,
        program::{Program, Symbol},
        types::{tycon, type_tyapp, type_tycon, TypeNode},
    },
    configuration::Configuration,
    constants::{ARRAY_ACT_NAME, ARRAY_NAME, IDENTITY_NAME, STD_NAME},
    error::Errors,
    misc::warn_msg,
    typecheck::TypeCheckContext,
};

pub fn run(prg: &mut Program, config: &Configuration) {
    match run_internal(prg, config) {
        Ok(_) => {}
        Err(errs) => {
            let msg = format!(
                "Errors occurred during \"act optimization\":\n{}",
                errs.to_string()
            );
            if config.develop_mode {
                panic!("{}", msg);
            } else {
                warn_msg(&msg);
            }
        }
    }
}

fn run_internal(prg: &mut Program, config: &Configuration) -> Result<(), Errors> {
    let tc = prg.create_typechecker(config);
    let mut sims = replace(&mut prg.symbols, HashMap::default());
    for (_name, sym) in &mut sims {
        run_on_symbol(sym, prg, &tc)?;
    }
    prg.symbols.extend(sims);
    Ok(())
}

fn run_on_symbol(sym: &mut Symbol, prg: &mut Program, tc: &TypeCheckContext) -> Result<(), Errors> {
    if is_array_act(&sym.generic_name) {
        run_on_array_act(sym, prg, tc)?;
    }
    Ok(())
}

fn run_on_array_act(
    sym: &mut Symbol,
    prg: &mut Program,
    tc: &TypeCheckContext,
) -> Result<(), Errors> {
    // Get the type `T`
    let act_ty = sym.ty.clone(); // I64 -> (T -> f T) -> U -> f U
    let lens_ty = act_ty.get_lambda_dst(); // (T -> f T) -> U -> f U

    if is_functor_identity(&lens_ty) {
        let act_for_id_gen_name =
            FullName::from_strs(&[STD_NAME, ARRAY_NAME], &format!("_act_identity"));
        let act_for_id_inst_name = prg.require_instantiation(&act_for_id_gen_name, &act_ty)?;
        prg.instantiate_symbols(tc)?;
        let act_for_id_val = expr_var(act_for_id_inst_name, None).set_type(act_ty.clone());
        sym.expr = Some(act_for_id_val);
    }

    Ok(())
}

fn is_array_act(generic_name: &FullName) -> bool {
    generic_name == &FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_ACT_NAME)
}

// Destructure the act type `(T -> f T) -> U -> f U` into `(T, U, f T, f U)`.
fn destructure_act_ty(
    lens_ty: &Arc<TypeNode>,
) -> (Arc<TypeNode>, Arc<TypeNode>, Arc<TypeNode>, Arc<TypeNode>) {
    // lens_ty: (T -> f T) -> U -> f U
    let lens_dst_ty = lens_ty.get_lambda_dst(); // U -> f U
    let u_ty = lens_dst_ty.get_lambda_srcs()[0].clone(); // U
    let fu_ty = lens_dst_ty.get_lambda_dst(); // f U
    let lens_src_ty = lens_ty.get_lambda_srcs()[0].clone(); // T -> f T
    let t_ty = lens_src_ty.get_lambda_srcs()[0].clone(); // T
    let ft_ty = lens_src_ty.get_lambda_dst(); // f T
    (t_ty, u_ty, ft_ty, fu_ty)
}

// Taking type `(T -> f T) -> U -> f U`, check if `f` is `Std::Identity`.
fn is_functor_identity(lens_ty: &Arc<TypeNode>) -> bool {
    let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(&lens_ty);

    let id_ty = type_tycon(&tycon(FullName::from_strs(&[STD_NAME], IDENTITY_NAME)));
    let id_t_ty = type_tyapp(id_ty.clone(), t_ty);
    let id_u_ty = type_tyapp(id_ty, u_ty);

    ft_ty.to_string() == id_t_ty.to_string() && fu_ty.to_string() == id_u_ty.to_string()
}
