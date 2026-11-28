/*
# Act Optimization

## Overview

Act-family functions refer to `Std::Array::act(i)` and struct field accessors `act_{field}`.
These functions have the type `[f : Functor] (T -> f T) -> U -> f U`.

This optimization replaces the implementation of act-family functions instantiated with specific functors with more efficient implementations:
- `Std::Identity`
- `Std::Const T`
- `Std::Tuple2 X`

## Notes

This optimization operates on the definitions of global symbols.
Therefore, to achieve maximum effect, it must be applied before global symbol inlining.

*/

use std::{collections::HashMap, mem::replace, sync::Arc};

use crate::{
    ast::{
        expr::expr_var_typed,
        name::{FullName, Name},
        program::{Program, Symbol},
        types::{tycon, type_tyapp, type_tycon, TyCon, Type::TyApp, TypeNode},
    },
    configuration::Configuration,
    constants::{
        ARRAY_NAME, BUILTIN_ACT_NAME, CONST_NAME, IDENTITY_NAME, STD_NAME, STRUCT_ACT_SYMBOL,
    },
    elaboration::typecheck::TypeCheckContext,
    error::Errors,
    fixstd::builtin::make_tuple_name_abs,
    misc::info_msg,
};

pub fn run(prg: &mut Program, config: &Configuration) {
    match run_internal(prg, config) {
        Ok(_) => {}
        Err(errs) => {
            let msg = format!(
                "Errors occurred during \"act optimization\". Please consider submitting an issue to the fixlang's repository. Details:\n{}",
                errs.to_string()
            );
            if config.develop_mode {
                panic!("{}", msg);
            } else {
                info_msg(&msg);
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
    if is_builtin_array_act(&sym.generic_name) {
        return run_on_array_act(sym, prg, tc);
    } else if let Some((str, field)) = prg.type_env.is_struct_act(&sym.generic_name) {
        return run_on_struct_field_act(&str, &field, sym, prg, tc);
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

    let opt_func_name = if is_functor_identity(&lens_ty) {
        Some(FullName::from_strs(
            &[STD_NAME, ARRAY_NAME],
            &format!("{}_identity", BUILTIN_ACT_NAME),
        ))
    } else if is_functor_const(&lens_ty) {
        Some(FullName::from_strs(
            &[STD_NAME, ARRAY_NAME],
            &format!("{}_const", BUILTIN_ACT_NAME),
        ))
    } else if is_functor_tuple2(&lens_ty) {
        Some(FullName::from_strs(
            &[STD_NAME, ARRAY_NAME],
            &format!("{}_tuple2", BUILTIN_ACT_NAME),
        ))
    } else {
        None
    };

    if let Some(opt_func_name) = opt_func_name {
        let inst_name = prg.require_instantiation(&opt_func_name, &act_ty)?;
        prg.instantiate_symbols(tc)?;
        let optimized_val = expr_var_typed(inst_name, act_ty.clone());
        sym.expr = Some(optimized_val);
    }

    Ok(())
}

fn is_builtin_array_act(generic_name: &FullName) -> bool {
    generic_name.is(&[STD_NAME, ARRAY_NAME], BUILTIN_ACT_NAME)
}

fn run_on_struct_field_act(
    str: &TyCon,
    field: &Name,
    sym: &mut Symbol,
    prg: &mut Program,
    tc: &TypeCheckContext,
) -> Result<(), Errors> {
    // Get the type `T`
    let act_ty = sym.ty.clone(); // (T -> f T) -> U -> f U

    let str_namespace = str.name.to_namespace();
    let opt_func_name = if is_functor_identity(&act_ty.clone()) {
        Some(FullName::new(
            &str_namespace,
            &format!("_{}{}_identity", STRUCT_ACT_SYMBOL, field),
        ))
    } else if is_functor_const(&act_ty.clone()) {
        Some(FullName::new(
            &str_namespace,
            &format!("_{}{}_const", STRUCT_ACT_SYMBOL, field),
        ))
    } else if is_functor_tuple2(&act_ty.clone()) {
        Some(FullName::new(
            &str_namespace,
            &format!("_{}{}_tuple2", STRUCT_ACT_SYMBOL, field),
        ))
    } else {
        None
    };

    if let Some(opt_func_name) = opt_func_name {
        let inst_name = prg.require_instantiation(&opt_func_name, &act_ty)?;
        prg.instantiate_symbols(tc)?;
        let optimized_val = expr_var_typed(inst_name, act_ty.clone());
        sym.expr = Some(optimized_val);
    }

    Ok(())
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

/// Whether `f` in the act type `lens_ty`, `(T -> f T) -> U -> f U`, is `Std::Identity`.
fn is_functor_identity(lens_ty: &Arc<TypeNode>) -> bool {
    let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(lens_ty);

    let id_ty = type_tycon(&tycon(FullName::from_strs(&[STD_NAME], IDENTITY_NAME)));
    ft_ty == type_tyapp(id_ty.clone(), t_ty) && fu_ty == type_tyapp(id_ty, u_ty)
}

/// Whether `f` in the act type `lens_ty`, `(T -> f T) -> U -> f U`, is `Std::Const C` for some
/// type `C`.
fn is_functor_const(lens_ty: &Arc<TypeNode>) -> bool {
    let const_ty = type_tycon(&tycon(FullName::from_strs(&[STD_NAME], CONST_NAME)));
    is_functor_applied_once(lens_ty, &const_ty)
}

/// Whether `f` in the act type `lens_ty`, `(T -> f T) -> U -> f U`, is `Std::Tuple2 X` for some
/// type `X`.
fn is_functor_tuple2(lens_ty: &Arc<TypeNode>) -> bool {
    let tuple2_ty = type_tycon(&tycon(make_tuple_name_abs(2)));
    is_functor_applied_once(lens_ty, &tuple2_ty)
}

/// Whether `f` in the act type `lens_ty`, `(T -> f T) -> U -> f U`, is `g C` for some type `C`.
/// `f T` and `f U` have to carry the same `C`.
///
/// # Arguments
/// * `g` — a type constructor that takes two arguments, such as `Std::Const`.
fn is_functor_applied_once(lens_ty: &Arc<TypeNode>, g: &Arc<TypeNode>) -> bool {
    let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(lens_ty);
    match (
        functor_argument(&ft_ty, &t_ty, g),
        functor_argument(&fu_ty, &u_ty, g),
    ) {
        (Some(ft_c), Some(fu_c)) => ft_c == fu_c,
        _ => false,
    }
}

/// The type `C` if `ty` is `g C arg_ty`.
///
/// # Examples
/// With `g` = `Std::Const`, `ty` = `Const I64 Bool` and `arg_ty` = `Bool`, returns `I64`.
fn functor_argument<'t>(
    ty: &'t Arc<TypeNode>,
    arg_ty: &Arc<TypeNode>,
    g: &Arc<TypeNode>,
) -> Option<&'t Arc<TypeNode>> {
    let TyApp(g_c, arg) = &ty.ty else {
        return None;
    };
    let TyApp(g_base, c) = &g_c.ty else {
        return None;
    };
    (arg == arg_ty && g_base == g).then_some(c)
}
