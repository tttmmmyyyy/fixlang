use super::*;

// Convert global function func = \x -> \y -> (...)  to func@uncurry = \(x, y) -> (...), and convert `func x y` to `func@uncurry (x, y)`
// Before optimization, `func x y` creates lambda object `func x` on heap, capturing `x`.
// After optimization, if construction of (x, y) is implemented as a special code that avoids heap allocation, then `func@uncurry (x, y)` requires no heap allocation.

// Global functions that cannot be uncurried.
pub fn exclude(name: &NameSpacedName) -> bool {
    let fix_name = NameSpacedName::from_strs(&[STD_NAME], FIX_NAME);
    if *name == fix_name
        || (name.to_string() + INSTANCIATED_NAME_SEPARATOR).starts_with(&fix_name.to_string())
    {
        // fix@uncurry will be corrupted since it uses SELF.
        // If uncurried, the type of SELF is changed but of course the implementation will not change.
        return true;
    }
    return false;
}

pub fn uncurry_optimization(fix_mod: &mut FixModule) {
    // First, define uncurried versions of global symbols.
    let syms = std::mem::replace(&mut fix_mod.instantiated_global_symbols, Default::default());
    for (sym_name, sym) in syms {
        let typechcker = sym.typechecker.as_ref().unwrap();

        fix_mod
            .instantiated_global_symbols
            .insert(sym_name.clone(), sym.clone());

        // Add uncurried function as long as possible.
        let mut expr = uncurry_lambda(
            &sym.template_name,
            sym.expr.as_ref().unwrap(),
            fix_mod,
            typechcker,
        );
        let mut name = sym_name.clone();
        while expr.is_some() {
            let new_expr = calculate_free_vars(expr.take().unwrap());
            convert_to_uncurried_name(name.name_as_mut());
            let new_ty = new_expr.inferred_ty.clone().unwrap();
            fix_mod.instantiated_global_symbols.insert(
                name.clone(),
                InstantiatedSymbol {
                    template_name: NameSpacedName::local(&format!(
                        "{} created by uncurry_optimization from {}",
                        &name.to_string(),
                        sym.template_name.to_string()
                    )),
                    ty: new_ty.clone(),
                    expr: Some(new_expr.clone()),
                    typechecker: sym.typechecker.clone(),
                },
            );
            expr = uncurry_lambda(&sym.template_name, &new_expr, fix_mod, typechcker);
        }
    }

    // Then replace expressions in the global symbols.
    let mut symbol_names: HashSet<NameSpacedName> = Default::default();
    for (name, _sym) in &fix_mod.instantiated_global_symbols {
        symbol_names.insert(name.clone());
    }
    for (_name, sym) in &mut fix_mod.instantiated_global_symbols {
        let expr = uncurry_global_function_call_subexprs(sym.expr.as_ref().unwrap(), &symbol_names);
        let expr = calculate_free_vars(expr);
        sym.expr = Some(expr);
    }

    // In the above process, there is possibility that constructor / getter of pairs is required to be instanciated.
    fix_mod.instantiate_symbols();
}

fn convert_to_uncurried_name(name: &mut Name) {
    *name += "@uncurry";
}

fn make_pair_name() -> NameSpacedName {
    NameSpacedName::from_strs(&[STD_NAME], &make_tuple_name(2))
}

pub fn make_pair_ty(ty0: &Arc<TypeNode>, ty1: &Arc<TypeNode>) -> Arc<TypeNode> {
    type_tyapp(
        type_tyapp(type_tycon(&tycon(make_pair_name())), ty0.clone()),
        ty1.clone(),
    )
}

// Convert expression `\x -> \y -> z` to `\(x, y) -> z`.
// NOTE: applying this repeatedly, `\x -> \y -> \z -> w` is converted to `\((x, y), z) -> w`, not to `\(x, (y, z)) -> w`.
// if uncurry cannot be done, return None.
fn uncurry_lambda(
    template_name: &NameSpacedName,
    expr: &Arc<ExprNode>,
    fix_mod: &mut FixModule,
    typechcker: &TypeCheckContext, // for resolving types of expr
) -> Option<Arc<ExprNode>> {
    if exclude(template_name) {
        return None;
    }
    let lam_ty = typechcker.substitute_type(expr.inferred_ty.as_ref().unwrap());
    match &*expr.expr {
        Expr::Lam(arg0, body0) => {
            let arg0_ty = lam_ty.get_funty_src();
            let body0_ty = lam_ty.get_funty_dst();
            match &*body0.expr {
                Expr::Lam(arg1, body) => {
                    let arg1_ty = body0_ty.get_funty_src();
                    let arg_types = vec![arg0_ty.clone(), arg1_ty.clone()];
                    let pair_ty = make_pair_ty(&arg0_ty, &arg1_ty);
                    let getter_types = (0..2)
                        .map(|i| type_fun(pair_ty.clone(), arg_types[i].clone()))
                        .collect::<Vec<_>>();
                    let getters = (0..2)
                        .map(|i| {
                            let name = NameSpacedName::new(
                                &make_pair_name().to_namespace(),
                                &format!("{}_{}", STRUCT_GETTER_NAME, i),
                            );
                            let name = fix_mod.require_instantiated_symbol(&name, &getter_types[i]);
                            expr_var(name, None).set_inferred_type(getter_types[i].clone())
                        })
                        .collect::<Vec<_>>();
                    let pair_arg_name = NameSpacedName::local("%uncurried_pair");
                    let uncurried_body = expr_let(
                        arg0.clone(),
                        expr_app(
                            getters[0].clone(),
                            expr_var(pair_arg_name.clone(), None)
                                .set_inferred_type(pair_ty.clone()),
                            None,
                        )
                        .set_inferred_type(arg0_ty),
                        expr_let(
                            arg1.clone(),
                            expr_app(
                                getters[1].clone(),
                                expr_var(pair_arg_name.clone(), None)
                                    .set_inferred_type(pair_ty.clone()),
                                None,
                            )
                            .set_inferred_type(arg1_ty.clone()),
                            body.clone(),
                            None,
                        )
                        .set_inferred_type(body.inferred_ty.clone().unwrap()),
                        None,
                    )
                    .set_inferred_type(body.inferred_ty.clone().unwrap());
                    let uncurried_body = move_abs_front_let(&uncurried_body); // Prepare for following uncurry.
                    let uncurried_lam =
                        expr_abs(var_var(pair_arg_name, None), uncurried_body, None)
                            .set_inferred_type(type_fun(
                                pair_ty,
                                body.inferred_ty.clone().unwrap(),
                            ));
                    Some(calculate_free_vars(uncurried_lam))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

// Convert expression like `func x y` to `func@uncurry (x, y)` if possible.
fn uncurry_global_function_call(
    expr: &Arc<ExprNode>,
    symbols: &HashSet<NameSpacedName>,
) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::App(fun1, arg1) => match &*fun1.expr {
            Expr::App(fun0, arg0) => match &*fun0.expr {
                Expr::Var(v) => {
                    if v.name.is_local() {
                        // If fun0 is not global, do not apply uncurry.
                        return expr.clone();
                    }
                    let mut f_uncurry = v.as_ref().clone();
                    convert_to_uncurried_name(&mut f_uncurry.name.name);
                    if !symbols.contains(&f_uncurry.name) {
                        // If uncurried function is not defined, do not apply uncurry.
                        return expr.clone();
                    }
                    let result_ty = expr.inferred_ty.clone().unwrap();
                    let arg0_ty = arg0.inferred_ty.clone().unwrap();
                    let arg1_ty = arg1.inferred_ty.clone().unwrap();
                    let pair_ty = make_pair_ty(&arg0_ty, &arg1_ty);
                    let f_uncurry = expr_var(f_uncurry.name, None)
                        .set_inferred_type(type_fun(pair_ty.clone(), result_ty.clone()));
                    expr_app(
                        f_uncurry,
                        expr_make_pair(arg0.clone(), arg1.clone()).set_inferred_type(pair_ty),
                        None,
                    )
                    .set_inferred_type(result_ty)
                }
                _ => expr.clone(),
            },
            _ => expr.clone(),
        },
        _ => expr.clone(),
    }
}

// Apply uncurry_global_function_call to all sub-expressions.
// NOTE: we need to convert sub-expression `func x y z` to `func@uncurry@uncurry ((x, y), z)`, not to `func@uncurry@uncurry (x, (y, z))`.
fn uncurry_global_function_call_subexprs(
    expr: &Arc<ExprNode>,
    symbols: &HashSet<NameSpacedName>,
) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::Lit(_) => expr.clone(),
        Expr::App(fun, arg) => {
            let expr = expr
                .set_app_func(uncurry_global_function_call_subexprs(fun, symbols))
                .set_app_arg(uncurry_global_function_call_subexprs(arg, symbols));
            uncurry_global_function_call(&expr, symbols)
        }
        Expr::Lam(_, val) => expr.set_lam_body(uncurry_global_function_call_subexprs(val, symbols)),
        Expr::Let(_, bound, val) => expr
            .set_let_bound(uncurry_global_function_call_subexprs(bound, symbols))
            .set_let_value(uncurry_global_function_call_subexprs(val, symbols)),
        Expr::If(c, t, e) => expr
            .set_if_cond(uncurry_global_function_call_subexprs(c, symbols))
            .set_if_then(uncurry_global_function_call_subexprs(t, symbols))
            .set_if_else(uncurry_global_function_call_subexprs(e, symbols)),
        Expr::TyAnno(e, _) => {
            expr.set_tyanno_expr(uncurry_global_function_call_subexprs(e, symbols))
        }
        Expr::MakePair(lhs, rhs) => expr
            .set_make_pair_lhs(uncurry_global_function_call_subexprs(lhs, symbols))
            .set_make_pair_rhs(uncurry_global_function_call_subexprs(rhs, symbols)),
    }
}

// Convert `let a = x in \b -> y` to `\b -> let a = x in y` if possible.
// NOTE: if name `b` is contained in x, then first we need to replace `b` to another name.
fn move_abs_front_let(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Let(let_var, let_bound, let_val) => {
            let let_val = move_abs_front_let(let_val);
            match &*let_val.expr {
                Expr::Lam(lam_var, lam_val) => {
                    let ty = expr.inferred_ty.clone().unwrap();

                    // Replace lam_var and it's appearance in lam_val to avoid confliction with free variables in let_bound.
                    let let_bound = calculate_free_vars(let_bound.clone());
                    let let_bound_free_vars = let_bound.free_vars();
                    let original_name = lam_var.name.clone();
                    let mut lam_var_name = original_name.clone();
                    let mut counter = 0;
                    while let_bound_free_vars.contains(&lam_var_name) {
                        *lam_var_name.name_as_mut() = format!("{}@{}", original_name.name, counter);
                        counter += 1;
                    }
                    let (lam_var, lam_val) = if lam_var_name == lam_var.name {
                        // Replace is not needed.
                        (lam_var.clone(), lam_val.clone())
                    } else {
                        // Replace is needed.
                        let lam_var = lam_var.set_name(lam_var_name.clone());
                        let lam_val = replace_free_var(lam_val, &original_name, &lam_var_name);
                        (lam_var, lam_val)
                    };

                    // Construct the expression.
                    let expr = expr_let(let_var.clone(), let_bound.clone(), lam_val.clone(), None)
                        .set_inferred_type(lam_val.inferred_ty.clone().unwrap());
                    let expr = expr_abs(lam_var, expr, None).set_inferred_type(ty);
                    expr
                }
                _ => expr.clone(),
            }
        }
        _ => expr.clone(),
    }
}

fn replace_free_var(
    expr: &Arc<ExprNode>,
    from: &NameSpacedName,
    to: &NameSpacedName,
) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Var(v) => {
            if v.name == *from {
                expr.clone().set_var_var(v.set_name(to.clone()))
            } else {
                expr.clone()
            }
        }
        Expr::Lit(_) => expr.clone(),
        Expr::App(func, arg) => {
            let func = replace_free_var(func, from, to);
            let arg = replace_free_var(arg, from, to);
            expr.set_app_func(func).set_app_arg(arg)
        }
        Expr::Lam(v, val) => {
            let val = if v.name == *from {
                // then, the from-name is shadowed in val, so we should not replace val.
                val.clone()
            } else {
                replace_free_var(val, from, to)
            };
            expr.set_lam_body(val)
        }
        Expr::Let(v, bound, val) => {
            let bound = replace_free_var(bound, from, to);
            let val = if v.name == *from {
                // then, the from-name is shadowed in val, so we should not replace val.
                val.clone()
            } else {
                replace_free_var(val, from, to)
            };
            expr.set_let_bound(bound).set_let_value(val)
        }
        Expr::If(c, t, e) => {
            let c = replace_free_var(c, from, to);
            let t = replace_free_var(t, from, to);
            let e = replace_free_var(e, from, to);
            expr.set_if_cond(c).set_if_then(t).set_if_else(e)
        }
        Expr::TyAnno(e, _) => {
            let e = replace_free_var(e, from, to);
            expr.set_tyanno_expr(e)
        }
        Expr::MakePair(l, r) => {
            let l = replace_free_var(l, from, to);
            let r = replace_free_var(r, from, to);
            expr.set_make_pair_lhs(l).set_make_pair_rhs(r)
        }
    }
}

// fn replace_travarsally(
//     expr: Arc<ExprNode>,
//     replace: &impl Fn(Arc<ExprNode>) -> Arc<ExprNode>,
// ) -> Arc<ExprNode> {
//     match &*expr.expr {
//         Expr::Var(_) => replace(expr.clone()),
//         Expr::Lit(_) => replace(expr.clone()),
//         Expr::App(fun, arg) => {
//             let expr = expr
//                 .set_app_func(replace_travarsally(fun.clone(), replace))
//                 .set_app_arg(replace_travarsally(arg.clone(), replace));
//             replace(expr)
//         }
//         Expr::Lam(_, val) => {
//             let expr = expr.set_lam_body(replace_travarsally(val.clone(), replace));
//             replace(expr)
//         }
//         Expr::Let(_, bound, val) => {
//             let expr = expr
//                 .set_let_bound(replace_travarsally(bound.clone(), replace))
//                 .set_let_value(replace_travarsally(val.clone(), replace));
//             replace(expr)
//         }
//         Expr::If(c, t, e) => {
//             let expr = expr
//                 .set_if_cond(replace_travarsally(c.clone(), replace))
//                 .set_if_then(replace_travarsally(t.clone(), replace))
//                 .set_if_else(replace_travarsally(e.clone(), replace));
//             replace(expr)
//         }
//         Expr::TyAnno(e, _) => {
//             let expr = expr.set_tyanno_expr(replace_travarsally(e.clone(), replace));
//             replace(expr)
//         }
//         Expr::MakePair(lhs, rhs) => {
//             let expr = expr
//                 .set_make_pair_lhs(replace_travarsally(lhs.clone(), replace))
//                 .set_make_pair_rhs(replace_travarsally(rhs.clone(), replace));
//             replace(expr)
//         }
//     }
// }
