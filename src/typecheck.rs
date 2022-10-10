use core::panic;

use super::*;

// #[derive(Debug)]
// pub struct TypeError {}

fn error_exit_with_src(msg: &str, src: &Option<Span>) -> ! {
    let mut str = String::default();
    str += "error: ";
    str += msg;
    str += "\n";
    match src {
        None => todo!(),
        Some(v) => {
            str += &v.to_string();
        }
    };
    error_exit(&str)
}

#[derive(Clone)]
struct LocalTermVar {
    ty: Arc<TypeNode>,
}

#[derive(Clone)]
struct LocalTypeVar {
    ty: Arc<TypeNode>,
    /* field for type class */
}

struct Scope<T> {
    var: HashMap<String, Vec<T>>,
    // type_var: HashMap<String, Vec<LocalTypeVar>>,
}

impl<T> Scope<T> {
    fn empty() -> Self {
        Self {
            var: HashMap::new(),
        }
    }
}

impl<T> Scope<T>
where
    T: Clone,
{
    // TODO: throw TypeError when unwrap fails.
    fn push(self: &mut Self, name: &str, ty: &T) {
        if !self.var.contains_key(name) {
            self.var.insert(String::from(name), Default::default());
        }
        self.var.get_mut(name).unwrap().push(ty.clone());
    }
    fn pop(self: &mut Self, name: &str) {
        self.var.get_mut(name).unwrap().pop();
        if self.var.get(name).unwrap().is_empty() {
            self.var.remove(name);
        }
    }
    fn get(self: &Self, name: &str) -> Option<&T> {
        self.var.get(name).map(|v| v.last().unwrap())
    }
    fn get_mut(self: &mut Self, name: &str) -> Option<&mut T> {
        self.var.get_mut(name).map(|v| v.last_mut().unwrap())
    }

    // fn push_type(self: &mut Self, name: &str) {
    //     if !self.type_var.contains_key(name) {
    //         self.type_var.insert(String::from(name), Default::default());
    //     }
    //     self.type_var.get_mut(name).unwrap().push(LocalTypeVar {});
    // }
    // fn pop_type(self: &mut Self, name: &str) {
    //     self.type_var.get_mut(name).unwrap().pop();
    //     if self.type_var.get(name).unwrap().is_empty() {
    //         self.type_var.remove(name);
    //     }
    // }
    // fn get_type(self: &Self, name: &str) -> LocalTypeVar {
    //     self.type_var.get(name).unwrap().last().unwrap().clone()
    // }
}

pub fn check_type(ei: Arc<ExprInfo>) -> Arc<ExprInfo> {
    let mut scope = Scope::<LocalTermVar>::empty();
    deduce_expr(ei, &mut scope)
}

fn deduce_expr(ei: Arc<ExprInfo>, scope: &mut Scope<LocalTermVar>) -> Arc<ExprInfo> {
    match &*ei.expr {
        Expr::Var(v) => deduce_var(ei.clone(), v.clone(), scope),
        Expr::Lit(lit) => deduce_lit(ei.clone(), lit.clone(), scope),
        Expr::App(func, arg) => deduce_app(ei.clone(), func.clone(), arg.clone(), scope),
        Expr::Lam(arg, val) => deduce_lam(ei.clone(), arg.clone(), val.clone(), scope),
        Expr::Let(var, bound, val) => {
            deduce_let(ei.clone(), var.clone(), bound.clone(), val.clone(), scope)
        }
        Expr::If(cond, then_expr, else_expr) => deduce_if(
            ei.clone(),
            cond.clone(),
            then_expr.clone(),
            else_expr.clone(),
            scope,
        ),
        Expr::AppType(expr, ty) => deduce_apptype(ei.clone(), expr.clone(), ty.clone(), scope),
        Expr::ForAll(tyvar, expr) => deduce_forall(ei.clone(), tyvar.clone(), expr.clone(), scope),
    }
}

fn deduce_var(ei: Arc<ExprInfo>, var: Arc<Var>, scope: &mut Scope<LocalTermVar>) -> Arc<ExprInfo> {
    let src = ei.source.clone();
    let ty = scope.get(&var.name);
    let ty = ty
        .unwrap_or_else(|| {
            error_exit_with_src(&format!("unknown variable `{}`", var.name), &src);
        })
        .ty
        .clone();
    Arc::new(Expr::Var(var))
        .into_expr_info(ei.source.clone())
        .with_deduced_type(ty)
}

fn deduce_lit(
    ei: Arc<ExprInfo>,
    lit: Arc<Literal>,
    _scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let lit_ty = lit.ty.clone();
    Arc::new(Expr::Lit(lit))
        .into_expr_info(ei.source.clone())
        .with_deduced_type(lit_ty.clone())
}

fn deduce_app(
    ei: Arc<ExprInfo>,
    func: Arc<ExprInfo>,
    arg: Arc<ExprInfo>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let func = deduce_expr(func, scope);
    let arg = deduce_expr(arg, scope);
    let arg_ty = arg.deduced_type.clone().unwrap();
    let fun_ty = func.deduced_type.clone().unwrap();

    // If func_ty is for<...> x => y, then infer ... as long as possible by matching x to arg_ty.
    let fun_ty = defer_forall_of_fun(fun_ty);
    if fun_ty.is_none() {
        error_exit_with_src(
            &format!("an expression is not a function but applied\n",),
            &func.source,
        )
    }
    let fun_ty = fun_ty.unwrap();
    let (vars, fun_ty) = fun_ty.decompose_forall_reversed();
    let (param_ty, body_ty) = match &fun_ty.ty {
        Type::FunTy(x, y) => (x, y),
        _ => unreachable!(),
    };
    let mut vars_infer = HashSet::<String>::default();
    vars_infer.extend(vars.iter().map(|v| v.name.clone()));
    let inferred = match_type(param_ty, &arg_ty, vars_infer);
    if inferred.is_none() {
        error_exit_with_src(
            &format!(
                "type mismatch: expected {}, found {}",
                &param_ty.clone().to_string(),
                &arg_ty.clone().to_string(),
            ),
            &arg.source,
        )
    }
    let inferred = inferred.unwrap();
    let mut reduce_scope = Scope::<LocalTypeVar>::empty();
    for (var_name, ty) in inferred.iter() {
        reduce_scope.push(var_name, &LocalTypeVar { ty: ty.clone() });
    }
    let ty = reduce_type(body_ty.clone(), &mut reduce_scope);
    expr_app(func, arg, ei.source.clone()).with_deduced_type(ty)
}

// Transform a type of form "for<...> x => y" to "for<a1,...,an> x => for<...> y" as far as possible,
// where a1,...,an are type variables used in x.
// Returns None if given type isn't of form for<...> x => y.
fn defer_forall_of_fun(ty: Arc<TypeNode>) -> Option<Arc<TypeNode>> {
    let (vars, fun_ty) = ty.decompose_forall_reversed();
    let (x, mut y) = match &fun_ty.ty {
        Type::FunTy(x, y) => (x.clone(), y.clone()),
        _ => return None,
    };

    let used_in_x = x.calculate_free_vars().info.free_vars.clone().unwrap();
    let (outer_vars, inner_vars): (Vec<Arc<TyVar>>, Vec<Arc<TyVar>>) = vars
        .iter()
        .map(|var| var.clone())
        .partition(|var| used_in_x.contains(&var.name));

    for var in inner_vars.iter() {
        y = type_forall(var.clone(), y).calculate_free_vars();
    }

    let mut ret = type_func(x, y).calculate_free_vars();
    for var in outer_vars.iter() {
        ret = type_forall(var.clone(), ret).calculate_free_vars();
    }

    Some(ret)
}

fn deduce_lam(
    ei: Arc<ExprInfo>,
    param: Arc<Var>,
    val: Arc<ExprInfo>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let param_ty = param.type_annotation.clone().unwrap();
    scope.push(
        &param.name,
        &LocalTermVar {
            ty: param_ty.clone(),
        },
    );
    let val = deduce_expr(val, scope);
    scope.pop(&param.name);
    let val_ty = val.deduced_type.clone().unwrap();
    expr_abs(param, val, ei.source.clone()).with_deduced_type(type_func(param_ty, val_ty))
}

fn deduce_let(
    ei: Arc<ExprInfo>,
    var: Arc<Var>,
    bound: Arc<ExprInfo>,
    val: Arc<ExprInfo>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let bound = deduce_expr(bound, scope);
    let bound_ty = bound.deduced_type.clone().unwrap();
    scope.push(
        &var.name,
        &LocalTermVar {
            ty: bound_ty.clone(),
        },
    );
    let val = deduce_expr(val, scope);
    scope.pop(&var.name);
    let val_ty = val.deduced_type.clone().unwrap();
    let ty = match var.type_annotation.clone() {
        Some(annotation) => {
            if is_eqv_type(&annotation, &bound_ty) {
                val_ty
            } else {
                panic!("Type mismatch on let");
            }
        }
        None => val_ty,
    };
    expr_let(var, bound, val, ei.source.clone()).with_deduced_type(ty)
}

fn deduce_if(
    ei: Arc<ExprInfo>,
    cond: Arc<ExprInfo>,
    then_expr: Arc<ExprInfo>,
    else_expr: Arc<ExprInfo>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let cond = deduce_expr(cond, scope);
    let then_expr = deduce_expr(then_expr, scope);
    let else_expr = deduce_expr(else_expr, scope);
    let then_ty = then_expr.deduced_type.clone().unwrap();
    let else_ty = else_expr.deduced_type.clone().unwrap();
    let ty = if is_eqv_type(&then_ty, &else_ty) {
        if is_eqv_type(&cond.deduced_type.clone().unwrap(), &bool_lit_ty()) {
            then_ty
        } else {
            error_exit_with_src(
                &format!(
                    "expected Bool, found {}",
                    cond.deduced_type.clone().unwrap().to_string()
                ),
                &cond.source,
            )
        }
    } else {
        error_exit_with_src(
            &format!(
                "type mismatch between then and else: expected {}, found {}",
                &then_ty.to_string(),
                &else_ty.to_string()
            ),
            &else_expr.source,
        )
    };
    expr_if(cond, then_expr, else_expr, ei.source.clone()).with_deduced_type(ty)
}

fn deduce_apptype(
    ei: Arc<ExprInfo>,
    expr: Arc<ExprInfo>,
    arg_ty: Arc<TypeNode>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let expr = deduce_expr(expr, scope);
    let arg_ty = reduce_type(arg_ty, &mut Scope::<LocalTypeVar>::empty()); // necessary?
    let ty = match &expr.deduced_type.clone().unwrap().ty {
        Type::ForAllTy(var, val_ty) => {
            let mut ty_scope = Scope::<LocalTypeVar>::empty();
            ty_scope.push(&var.name, &LocalTypeVar { ty: arg_ty.clone() });
            reduce_type(val_ty.clone(), &mut ty_scope)
        }
        _ => error_exit_with_src(
            &format!("type argument given to non-polymorphic expression"),
            &ei.source,
        ),
    };
    expr_appty(expr, arg_ty, ei.source.clone()).with_deduced_type(ty)
}

fn deduce_forall(
    ei: Arc<ExprInfo>,
    tyvar: Arc<TyVar>,
    expr: Arc<ExprInfo>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let expr = deduce_expr(expr, scope);
    let ty = type_forall(tyvar.clone(), expr.deduced_type.clone().unwrap());
    expr_forall(tyvar, expr, ei.source.clone()).with_deduced_type(ty)
}

fn reduce_type(ty: Arc<TypeNode>, scope: &mut Scope<LocalTypeVar>) -> Arc<TypeNode> {
    match &ty.ty {
        Type::AppTy(fun_ty, arg_ty) => {
            let arg_ty = reduce_type(arg_ty.clone(), scope);
            match &fun_ty.ty {
                Type::ForAllTy(param_ty, val_ty) => {
                    scope.push(param_ty.name.as_str(), &LocalTypeVar { ty: arg_ty });
                    let val_ty = reduce_type(val_ty.clone(), scope);
                    scope.pop(param_ty.name.as_str());
                    val_ty
                }
                _ => panic!("Applying type requires forall."),
            }
        }
        Type::TyVar(var) => match scope.get(var.name.as_str()) {
            Some(local_ty) => local_ty.ty.clone(),
            None => ty,
        },
        Type::LitTy(_) => ty,
        Type::TyConApp(tycon, arg_tys) => {
            let arg_tys: Vec<Arc<TypeNode>> = arg_tys
                .iter()
                .map(|ty| reduce_type(ty.clone(), scope))
                .collect();
            if tycon.arity != arg_tys.len() as u32 {
                panic!(
                    "Type constructor {} requires {} argments.",
                    tycon.name, tycon.arity
                );
            }
            tycon_app(tycon.clone(), arg_tys)
        }
        Type::FunTy(param_ty, val_ty) => type_fun(
            reduce_type(param_ty.clone(), scope),
            reduce_type(val_ty.clone(), scope),
        ),
        Type::ForAllTy(var, val_ty) => {
            scope.push(
                &var.name,
                &LocalTypeVar {
                    ty: type_var_from_tyvar(var.clone()),
                },
            );
            let val_ty = reduce_type(val_ty.clone(), scope);
            scope.pop(&var.name);
            type_forall(var.clone(), val_ty)
        }
    }
}

// Info of local variable in type matching
#[derive(Clone)]
enum LocalTyVarInfo {
    Free,                            // Free variable defined outer.
    ForAll(u32), // local variable introduced in for<...> with identifier number.
    Inferred(Option<Arc<TypeNode>>), // local variable inferred (or waiting to be inferred when None).
}

// Check two types are equivalent.
// Equivalence is checked except naming of type variables introduced by for<...>.
// Free type variables must coincide.
// For example, "for<a> a => x" is equivalent to "for<b> b => x", but not to "for<b> b => y".
pub fn is_eqv_type(lhs: &Arc<TypeNode>, rhs: &Arc<TypeNode>) -> bool {
    match_type(lhs, rhs, HashSet::<String>::default()).is_some()
}

// Match a type to another type under equivalence.
pub fn match_type(
    lhs: &Arc<TypeNode>,
    rhs: &Arc<TypeNode>,
    lhs_vars_infer: HashSet<String>,
) -> Option<HashMap<String, Arc<TypeNode>>> {
    let lhs = lhs.calculate_free_vars();
    let rhs = rhs.calculate_free_vars();
    let lhs_free_vars = lhs.info.free_vars.as_ref().unwrap();
    let rhs_free_vars = rhs.info.free_vars.as_ref().unwrap();

    // Check if lhs_free_vars = rhs_free_vars + lhs_vars_infer.
    if lhs_free_vars.len() != rhs_free_vars.len() + lhs_vars_infer.len() {
        return None;
    }
    let mut merged = HashSet::<String>::default();
    merged.extend(rhs_free_vars.to_owned());
    merged.extend(lhs_vars_infer.to_owned());
    if *lhs_free_vars != merged {
        return None;
    }

    // Set up scopes.
    let mut lhs_scope = Scope::<LocalTyVarInfo>::empty();
    for var_name in lhs_free_vars {
        if lhs_vars_infer.contains(var_name) {
            lhs_scope.push(&var_name, &LocalTyVarInfo::Inferred(None));
        } else {
            lhs_scope.push(&var_name, &LocalTyVarInfo::Free);
        }
    }
    let mut rhs_scope = Scope::<LocalTyVarInfo>::empty();
    for var_name in rhs_free_vars {
        rhs_scope.push(&var_name, &LocalTyVarInfo::Free);
    }

    // Match and return result.
    let mut next_id: u32 = 0;
    let ok = match_type_core(&lhs, &rhs, &mut lhs_scope, &mut rhs_scope, &mut next_id);
    if !ok {
        return None;
    }

    let mut ret = HashMap::<String, Arc<TypeNode>>::default();
    for var_name in lhs_vars_infer {
        let inferred = lhs_scope.get(&var_name).unwrap();
        match inferred {
            LocalTyVarInfo::Inferred(inferred) => match inferred {
                Some(inferred) => {
                    ret.insert(var_name, inferred.clone());
                }
                None => {
                    return None;
                }
            },
            _ => unreachable!(),
        }
    }
    Some(ret)
}

// Match two types.
// Only lhs_scope can contain LocalTyVarInfo::Inferred.
fn match_type_core(
    lhs: &Arc<TypeNode>,
    rhs: &Arc<TypeNode>,
    lhs_scope: &mut Scope<LocalTyVarInfo>,
    rhs_scope: &mut Scope<LocalTyVarInfo>,
    next_id: &mut u32,
) -> bool {
    match &lhs.ty {
        Type::TyVar(lhs_var) => {
            let lhs = lhs_scope.get(&lhs_var.name).unwrap().clone();
            match lhs {
                LocalTyVarInfo::ForAll(lhs_id) => {
                    // Lhs was introduced at forall.
                    match &rhs.ty {
                        Type::TyVar(rhs_var) => {
                            let rhs = rhs_scope.get(&rhs_var.name).unwrap();
                            match rhs {
                                LocalTyVarInfo::ForAll(rhs_id) => {
                                    return lhs_id == *rhs_id;
                                }
                                _ => {
                                    return false;
                                }
                            }
                        }
                        _ => {
                            // Lhs is a type variable but rhs is not.
                            return false;
                        }
                    }
                }
                LocalTyVarInfo::Inferred(infer) => match infer {
                    Some(inferred) => {
                        // Lhs was already inferred.
                        return match_type_core(&inferred, rhs, lhs_scope, rhs_scope, next_id);
                    }
                    None => {
                        // Lhs is waiting to be inferred.
                        *lhs_scope.get_mut(&lhs_var.name).unwrap() =
                            LocalTyVarInfo::Inferred(Some(rhs.clone()));
                        return true;
                    }
                },
                LocalTyVarInfo::Free => {
                    // Lhs is free variable defined outer.
                    match &rhs.ty {
                        Type::TyVar(rhs_var) => {
                            return lhs_var.name == rhs_var.name;
                        }
                        _ => {
                            // Lhs is a free type variable but rhs is not.
                            return false;
                        }
                    }
                }
            }
        }
        Type::LitTy(lhs_lit) => match &rhs.ty {
            Type::LitTy(rhs_lit) => lhs_lit.id == rhs_lit.id,
            _ => false,
        },
        Type::AppTy(lhs_func_ty, lhs_arg_ty) => match &rhs.ty {
            Type::AppTy(rhs_func_ty, rhs_arg_ty) => {
                match_type_core(lhs_func_ty, rhs_func_ty, lhs_scope, rhs_scope, next_id)
                    && match_type_core(lhs_arg_ty, rhs_arg_ty, lhs_scope, rhs_scope, next_id)
            }
            _ => false,
        },
        Type::TyConApp(lhs_tycon, lhs_args) => match &rhs.ty {
            Type::TyConApp(rhs_tycon, rhs_args) => {
                if *lhs_tycon != *rhs_tycon {
                    return false;
                }
                if lhs_args.len() != rhs_args.len() {
                    return false;
                }
                for i in 0..lhs_args.len() {
                    if !match_type_core(&lhs_args[i], &rhs_args[i], lhs_scope, rhs_scope, next_id) {
                        return false;
                    }
                }
                return true;
            }
            _ => false,
        },
        Type::FunTy(lhs_param_ty, lhs_val_ty) => match &rhs.ty {
            Type::FunTy(rhs_param_ty, rhs_val_ty) => {
                match_type_core(lhs_param_ty, rhs_param_ty, lhs_scope, rhs_scope, next_id)
                    && match_type_core(lhs_val_ty, rhs_val_ty, lhs_scope, rhs_scope, next_id)
            }
            _ => false,
        },
        Type::ForAllTy(lhs_tyvar, lhs_val_ty) => match &rhs.ty {
            Type::ForAllTy(rhs_tyvar, rhs_val_ty) => {
                lhs_scope.push(&lhs_tyvar.name, &LocalTyVarInfo::ForAll(*next_id));
                rhs_scope.push(&rhs_tyvar.name, &LocalTyVarInfo::ForAll(*next_id));
                *next_id += 1;
                let ret = match_type_core(lhs_val_ty, rhs_val_ty, lhs_scope, rhs_scope, next_id);
                rhs_scope.pop(&rhs_tyvar.name);
                lhs_scope.pop(&lhs_tyvar.name);
                ret
            }
            _ => false,
        },
    }
}
