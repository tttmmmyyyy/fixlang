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
        expr::expr_var,
        name::{FullName, Name},
        program::{Program, Symbol},
        types::{tycon, type_tyapp, type_tycon, TyCon, Type::TyApp, TypeNode},
    },
    builtin::make_tuple_name,
    configuration::Configuration,
    constants::{
        ARRAY_NAME, BUILTIN_ACT_NAME, CONST_NAME, IDENTITY_NAME, STD_NAME, STRUCT_ACT_SYMBOL,
    },
    error::Errors,
    misc::info_msg,
    typecheck::TypeCheckContext,
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
        let optimized_val = expr_var(inst_name, None).set_type(act_ty.clone());
        sym.expr = Some(optimized_val);
    }

    Ok(())
}

fn is_builtin_array_act(generic_name: &FullName) -> bool {
    generic_name == &FullName::from_strs(&[STD_NAME, ARRAY_NAME], BUILTIN_ACT_NAME)
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
        let optimized_val = expr_var(inst_name, None).set_type(act_ty.clone());
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

// Taking type `(T -> f T) -> U -> f U`, check if `f` is `Std::Identity`.
fn is_functor_identity(lens_ty: &Arc<TypeNode>) -> bool {
    let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(&lens_ty);

    let id_ty = type_tycon(&tycon(FullName::from_strs(&[STD_NAME], IDENTITY_NAME)));
    let id_t_ty = type_tyapp(id_ty.clone(), t_ty);
    let id_u_ty = type_tyapp(id_ty, u_ty);

    ft_ty.to_string() == id_t_ty.to_string() && fu_ty.to_string() == id_u_ty.to_string()
}

// Taking type `(T -> f T) -> U -> f U`, check if `f` is `Std::Const C` for some `C`.
fn is_functor_const(lens_ty: &Arc<TypeNode>) -> bool {
    let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(&lens_ty);

    let const_ty = type_tycon(&tycon(FullName::from_strs(&[STD_NAME], CONST_NAME)));

    // Try to extract C from ft_ty (which should be Const C T)
    // ft_ty has the form: (Const C) T = TyApp(TyApp(Const, C), T)
    let TyApp(ft_const_c, t) = &ft_ty.ty else {
        return false;
    };
    if t.to_string() != t_ty.to_string() {
        return false;
    }
    let TyApp(ft_const_base, ft_c) = &ft_const_c.ty else {
        return false;
    };
    if ft_const_base.to_string() != const_ty.to_string() {
        return false;
    }

    // Try to extract C from fu_ty (which should be Const C U)
    let TyApp(fu_const_c, u) = &fu_ty.ty else {
        return false;
    };
    if u.to_string() != u_ty.to_string() {
        return false;
    }
    let TyApp(fu_const_base, fu_c) = &fu_const_c.ty else {
        return false;
    };
    if fu_const_base.to_string() != const_ty.to_string() {
        return false;
    }

    // Check if both have the same C
    ft_c.to_string() == fu_c.to_string()
}

// Taking type `(T -> f T) -> U -> f U`, check if `f` is `Std::Tuple2 X` for some `X`.
fn is_functor_tuple2(lens_ty: &Arc<TypeNode>) -> bool {
    let (t_ty, u_ty, ft_ty, fu_ty) = destructure_act_ty(&lens_ty);

    let tuple2_ty = type_tycon(&tycon(make_tuple_name(2)));

    // Try to extract X from ft_ty (which should be Tuple2 X T)
    // ft_ty has the form: (Tuple2 X) T = TyApp(TyApp(Tuple2, X), T)
    let TyApp(ft_tuple2_x, t) = &ft_ty.ty else {
        return false;
    };
    if t.to_string() != t_ty.to_string() {
        return false;
    }
    let TyApp(ft_tuple2_base, ft_x) = &ft_tuple2_x.ty else {
        return false;
    };
    if ft_tuple2_base.to_string() != tuple2_ty.to_string() {
        return false;
    }

    // Try to extract X from fu_ty (which should be Tuple2 X U)
    let TyApp(fu_tuple2_x, u) = &fu_ty.ty else {
        return false;
    };
    if u.to_string() != u_ty.to_string() {
        return false;
    }
    let TyApp(fu_tuple2_base, fu_x) = &fu_tuple2_x.ty else {
        return false;
    };
    if fu_tuple2_base.to_string() != tuple2_ty.to_string() {
        return false;
    }

    // Check if both have the same X
    ft_x.to_string() == fu_x.to_string()
}
