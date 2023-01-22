use super::*;

// Convert global function func = \x -> \y -> (...)  to func@uncurry = \(x, y) -> (...), and convert `func x y` to `func@uncurry (x, y)`
// Before optimization, `func x y` creates lambda object `func x` on heap, capturing `x`.
// After optimization, if construction of (x, y) is implemented as a special code that avoids heap allocation, then `func@uncurry (x, y)` requires no heap allocation.

// Global functions that cannot be uncurried.
pub fn exclude(name: &FullName) -> bool {
    let fix_name = FullName::from_strs(&[STD_NAME], FIX_NAME);
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
    let mut next_var_id: u32 = 0;
    // First, define uncurried versions of global symbols.
    let syms = std::mem::replace(&mut fix_mod.instantiated_global_symbols, Default::default());
    for (sym_name, sym) in syms {
        let typechcker = sym.typechecker.as_ref().unwrap();

        fix_mod
            .instantiated_global_symbols
            .insert(sym_name.clone(), sym.clone());

        // Add uncurried function as long as possible.
        for arg_cnt in 2..(TUPLE_SIZE_MAX + 1) {
            let mut expr = uncurry_lambda(
                &sym.template_name,
                sym.expr.as_ref().unwrap(),
                fix_mod,
                typechcker,
                &mut next_var_id,
                arg_cnt as usize,
            );
            if expr.is_none() {
                break;
            }
            let expr = calculate_free_vars(expr.take().unwrap());
            let ty = expr.inferred_ty.clone().unwrap();
            let mut name = sym_name.clone();
            convert_to_uncurried_name(name.name_as_mut(), arg_cnt as usize);
            fix_mod.instantiated_global_symbols.insert(
                name.clone(),
                InstantiatedSymbol {
                    template_name: FullName::local(&format!(
                        "{} created by uncurry_optimization from {}",
                        &name.to_string(),
                        sym.template_name.to_string()
                    )),
                    ty,
                    expr: Some(expr.clone()),
                    typechecker: sym.typechecker.clone(),
                },
            );
        }
    }

    // Then replace expressions in the global symbols.
    let mut symbol_names: HashSet<FullName> = Default::default();
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

fn convert_to_uncurried_name(name: &mut Name, count: usize) {
    *name += &format!("@uncurry{}", count);
}

pub fn make_pair_name(size: usize) -> FullName {
    FullName::from_strs(&[STD_NAME], &make_tuple_name(size as u32))
}

pub fn make_tuple_ty(tys: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
    assert!(tys.len() <= TUPLE_SIZE_MAX as usize);
    let mut ty = type_tycon(&tycon(make_pair_name(tys.len())));
    for field_ty in tys {
        ty = type_tyapp(ty, field_ty);
    }
    ty
}

// Convert expression `\x -> \y -> z` to `\(x, y) -> z`.
// NOTE: applying this repeatedly, `\x -> \y -> \z -> w` is converted to `\((x, y), z) -> w`, not to `\(x, (y, z)) -> w`.
// if uncurry cannot be done, return None.
fn uncurry_lambda(
    template_name: &FullName,
    expr: &Arc<ExprNode>,
    fix_mod: &mut FixModule,
    typechcker: &TypeCheckContext, // for resolving types of expr
    next_var_id: &mut u32,
    vars_count: usize,
) -> Option<Arc<ExprNode>> {
    if exclude(template_name) {
        return None;
    }
    // Extract abstructions from expr.
    let expr = move_abs_front_let_all(expr);
    let (args, body) = collect_abs(&expr, vars_count);
    if args.len() != vars_count {
        return None;
    }

    // Collect types of argments.
    let expr_type = typechcker.substitute_type(expr.inferred_ty.as_ref().unwrap());
    let (arg_types, body_ty) = collect_app_src(&expr_type, vars_count);
    let tuple_ty = make_tuple_ty(arg_types.clone());
    assert_eq!(
        typechcker.substitute_type(body.inferred_ty.as_ref().unwrap()),
        body_ty
    );

    // Collect getter of fields of the tuple.
    let getter_types = (0..vars_count)
        .map(|i| type_fun(tuple_ty.clone(), arg_types[i].clone()))
        .collect::<Vec<_>>();
    let getters = (0..vars_count)
        .map(|i| {
            let name = FullName::new(
                &make_pair_name(vars_count).to_namespace(),
                &format!("{}_{}", STRUCT_GETTER_NAME, i),
            );
            let name = fix_mod.require_instantiated_symbol(&name, &getter_types[i]);
            expr_var(name, None).set_inferred_type(getter_types[i].clone())
        })
        .collect::<Vec<_>>();

    // Make argument (tuple) name.
    let tuple_arg_name = FullName::local(&format!("%uncurried_tuple{}", *next_var_id));
    *next_var_id += 1;

    // Construct body of resulting lambda.
    let mut lam_body = body.clone();
    for i in (0..vars_count).rev() {
        lam_body = expr_let(
            Pattern::var_pattern(args[i].clone()),
            expr_app(
                getters[i].clone(),
                expr_var(tuple_arg_name.clone(), None).set_inferred_type(tuple_ty.clone()),
                None,
            )
            .set_inferred_type(arg_types[i].clone()),
            lam_body,
            None,
        )
        .set_inferred_type(body_ty.clone())
    }

    // Construct uncurried lambda.
    let uncurried_lam = expr_abs(var_var(tuple_arg_name, None), lam_body, None)
        .set_inferred_type(type_fun(tuple_ty, body_ty));
    Some(uncurried_lam)
}

// Convert \x -> \y -> z to ([x, y], z).
fn collect_abs(expr: &Arc<ExprNode>, vars_limit: usize) -> (Vec<Arc<Var>>, Arc<ExprNode>) {
    fn collect_abs_inner(
        expr: &Arc<ExprNode>,
        vars: &mut Vec<Arc<Var>>,
        vars_limit: usize,
    ) -> Arc<ExprNode> {
        match &*expr.expr {
            Expr::Lam(var, val) => {
                vars.push(var.clone());
                if vars.len() >= vars_limit {
                    return val.clone();
                }
                return collect_abs_inner(val, vars, vars_limit);
            }
            _ => expr.clone(),
        }
    }

    let mut vars: Vec<Arc<Var>> = vec![];
    let val = collect_abs_inner(expr, &mut vars, vars_limit);
    (vars, val)
}

// Convert x y z to (x, [y, z]).
fn collect_app(expr: &Arc<ExprNode>) -> (Arc<ExprNode>, Vec<Arc<ExprNode>>) {
    match &*expr.expr {
        Expr::App(fun, arg) => {
            let (fun, mut args) = collect_app(fun);
            args.push(arg.clone());
            (fun, args)
        }
        _ => (expr.clone(), vec![]),
    }
}

// Convert A -> B -> C to ([A, B], C)
fn collect_app_src(ty: &Arc<TypeNode>, vars_limit: usize) -> (Vec<Arc<TypeNode>>, Arc<TypeNode>) {
    fn collect_app_src_inner(
        ty: &Arc<TypeNode>,
        vars: &mut Vec<Arc<TypeNode>>,
        vars_limit: usize,
    ) -> Arc<TypeNode> {
        match &ty.ty {
            Type::FunTy(var, val) => {
                vars.push(var.clone());
                if vars.len() >= vars_limit {
                    return val.clone();
                }
                return collect_app_src_inner(&val, vars, vars_limit);
            }
            _ => ty.clone(),
        }
    }

    let mut vars: Vec<Arc<TypeNode>> = vec![];
    let val = collect_app_src_inner(ty, &mut vars, vars_limit);
    (vars, val)
}

// Convert expression like `func x y` to `func@uncurry (x, y)` if possible.
fn uncurry_global_function_call(
    expr: &Arc<ExprNode>,
    symbols: &HashSet<FullName>,
) -> Arc<ExprNode> {
    let (fun, args) = collect_app(expr);
    match &*fun.expr {
        Expr::Var(v) => {
            if v.name.is_local() {
                // If fun is not global, do not apply uncurry.
                return expr.clone();
            }
            let mut f_uncurry = v.as_ref().clone();
            convert_to_uncurried_name(&mut f_uncurry.name.name, args.len());
            if !symbols.contains(&f_uncurry.name) {
                // If uncurried function is not defined, do not apply uncurry.
                return expr.clone();
            }
            let result_ty = expr.inferred_ty.clone().unwrap();
            let arg_tys = args
                .iter()
                .map(|arg| arg.inferred_ty.clone().unwrap())
                .collect::<Vec<_>>();
            let tuple_ty = make_tuple_ty(arg_tys);
            let f_uncurry = expr_var(f_uncurry.name, None)
                .set_inferred_type(type_fun(tuple_ty.clone(), result_ty.clone()));
            expr_app(
                f_uncurry,
                expr_make_tuple(args).set_inferred_type(tuple_ty),
                None,
            )
            .set_inferred_type(result_ty)
        }
        _ => expr.clone(),
    }
}

// Apply uncurry_global_function_call to all sub-expressions.
// NOTE: we need to convert sub-expression `func x y z` to `func@uncurry@uncurry ((x, y), z)`, not to `func@uncurry@uncurry (x, (y, z))`.
fn uncurry_global_function_call_subexprs(
    expr: &Arc<ExprNode>,
    symbols: &HashSet<FullName>,
) -> Arc<ExprNode> {
    let expr = uncurry_global_function_call(expr, symbols);
    match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::Lit(_) => expr.clone(),
        Expr::App(fun, arg) => expr
            .set_app_func(uncurry_global_function_call_subexprs(fun, symbols))
            .set_app_arg(uncurry_global_function_call_subexprs(arg, symbols)),
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
        Expr::MakeTuple(fields) => {
            let fields = fields.clone();
            let mut expr = expr;
            for (idx, field) in fields.iter().enumerate() {
                let field = uncurry_global_function_call_subexprs(field, symbols);
                expr = expr.set_make_tuple_field(field, idx);
            }
            expr
        }
    }
}

// Convert `let a = x in \b -> y` to `\b -> let a = x in y` if possible.
// NOTE: if name `b` is contained in x, then first we need to replace `b` to another name.
fn move_abs_front_let_one(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Let(let_var, let_bound, let_val) => {
            let let_val = move_abs_front_let_one(let_val);
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

// apply move_abs_front_let_one repeatedly at the head.
fn move_abs_front_let_all(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Lam(_, val) => {
            let val = move_abs_front_let_all(val);
            expr.set_lam_body(val)
        }
        Expr::Let(_, _, val) => {
            let val = move_abs_front_let_all(val);
            let expr = &expr.set_let_value(val);
            move_abs_front_let_one(&expr)
        }
        _ => expr.clone(),
    }
}

fn replace_free_var(expr: &Arc<ExprNode>, from: &FullName, to: &FullName) -> Arc<ExprNode> {
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
            let val = if v.vars().contains(from) {
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
        Expr::MakeTuple(fields) => {
            let mut expr = expr.clone();
            for (idx, field) in fields.iter().enumerate() {
                let field = replace_free_var(field, from, to);
                expr = expr.set_make_tuple_field(field, idx);
            }
            expr
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
