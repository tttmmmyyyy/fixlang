use super::*;

// Convert global function func = \x -> \y -> (...)  to func@uncurry = \(x, y) -> (...), and convert `func x y` to `func@uncurry (x, y)`
// Before optimization, `func x y` creates lambda object `func x` on heap, capturing `x`.
// After optimization, if construction of (x, y) is implemented as a special code that avoids heap allocation, then `func@uncurry (x, y)` requires no heap allocation.

pub fn uncurry_optimization(fix_mod: &mut FixModule) {
    // First, define uncurried versions of global symbols.
    let syms = std::mem::replace(&mut fix_mod.instantiated_global_symbols, Default::default());
    for (sym_name, sym) in syms {
        fix_mod
            .instantiated_global_symbols
            .insert(sym_name.clone(), sym.clone());

        // Add uncurried function as long as possible.
        let mut expr = uncurry_lambda(sym.expr.as_ref().unwrap(), &sym.ty, fix_mod);
        let mut name = sym_name.clone();
        while expr.is_some() {
            let new_expr = expr.take().unwrap();
            convert_to_uncurried_name(name.name_as_mut());
            let new_ty = new_expr.inferred_ty.clone().unwrap();
            fix_mod.instantiated_global_symbols.insert(
                name.clone(),
                InstantiatedSymbol {
                    template_name: NameSpacedName::local("N/A; created by uncurry_optimization"),
                    ty: new_ty.clone(),
                    expr: Some(new_expr.clone()),
                    typechecker: sym.typechecker.clone(),
                },
            );
            expr = uncurry_lambda(&new_expr, &new_ty, fix_mod);
        }
    }

    // Then replace expressions in the global symbols.
    for (_name, sym) in &mut fix_mod.instantiated_global_symbols {
        sym.expr = Some(uncurry_global_function_call_subexprs(
            sym.expr.as_ref().unwrap(),
        ));
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
    expr: &Arc<ExprNode>,
    lam_ty: &Arc<TypeNode>,
    fix_mod: &mut FixModule,
) -> Option<Arc<ExprNode>> {
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
                                .set_inferred_type(getter_types[0].clone()),
                            None,
                        )
                        .set_inferred_type(arg0_ty),
                        expr_let(
                            arg1.clone(),
                            expr_app(
                                getters[1].clone(),
                                expr_var(pair_arg_name.clone(), None)
                                    .set_inferred_type(getter_types[1].clone()),
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
fn uncurry_global_function_call(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    let expr = match &*expr.expr {
        Expr::App(fun1, arg1) => match &*fun1.expr {
            Expr::App(fun0, arg0) => match &*fun0.expr {
                Expr::Var(v) => {
                    if v.name.is_local() {
                        // If fun0 is not global, do not apply uncurry.
                        return expr.clone();
                    }
                    let result_ty = expr.inferred_ty.clone().unwrap();
                    let arg0_ty = arg0.inferred_ty.clone().unwrap();
                    let arg1_ty = arg1.inferred_ty.clone().unwrap();
                    let pair_ty = make_pair_ty(&arg0_ty, &arg1_ty);
                    let mut f_uncurry = v.as_ref().clone();
                    convert_to_uncurried_name(&mut f_uncurry.name.name);
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
    };
    calculate_free_vars(expr)
}

// Apply uncurry_global_function_call to all sub-expressions.
// NOTE: we need to convert sub-expression `func x y z` to `func@uncurry@uncurry ((x, y), z)`, not to `func@uncurry@uncurry (x, (y, z))`.
fn uncurry_global_function_call_subexprs(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    let expr = match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::Lit(_) => expr.clone(),
        Expr::App(fun, arg) => {
            let expr = expr
                .set_app_func(uncurry_global_function_call_subexprs(fun))
                .set_app_arg(uncurry_global_function_call_subexprs(arg));
            uncurry_global_function_call(&expr)
        }
        Expr::Lam(_, val) => expr.set_lam_body(uncurry_global_function_call_subexprs(val)),
        Expr::Let(_, bound, val) => expr
            .set_let_bound(uncurry_global_function_call_subexprs(bound))
            .set_let_value(uncurry_global_function_call_subexprs(val)),
        Expr::If(c, t, e) => expr
            .set_if_cond(uncurry_global_function_call_subexprs(c))
            .set_if_then(uncurry_global_function_call_subexprs(t))
            .set_if_else(uncurry_global_function_call_subexprs(e)),
        Expr::TyAnno(e, _) => expr.set_tyanno_expr(uncurry_global_function_call_subexprs(e)),
        Expr::MakePair(lhs, rhs) => expr
            .set_make_pair_lhs(uncurry_global_function_call_subexprs(lhs))
            .set_make_pair_rhs(uncurry_global_function_call_subexprs(rhs)),
    };
    calculate_free_vars(expr)
}
