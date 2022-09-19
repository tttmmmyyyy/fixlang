use core::panic;

use super::*;

// #[derive(Debug)]
// pub struct TypeError {}

#[derive(Clone)]
struct LocalTermVar {
    ty: Arc<Type>,
}

#[derive(Clone)]
struct LocalTypeVar {
    ty: Arc<Type>,
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
    fn get(self: &Self, name: &str) -> Option<T> {
        self.var.get(name).map(|v| v.last().unwrap().clone())
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
    let ty = scope.get(&var.name);
    let ty = ty
        .unwrap_or_else(|| {
            panic!("Unknown variable: {}", var.name);
        })
        .ty;
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
    let ty = match &*func.deduced_type.clone().unwrap() {
        Type::FunTy(param_ty, result_ty) => {
            if is_equivalent_type(param_ty.clone(), arg_ty) {
                result_ty.clone()
            } else {
                panic!("Type mismatch between parameter and argument!");
            }
        }
        _ => {
            panic!("In the expression \"a b\", \"a\" is expected to be a function.")
        }
    };
    app(func, arg, ei.source.clone()).with_deduced_type(ty)
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
    lam(param, val, ei.source.clone()).with_deduced_type(type_func(param_ty, val_ty))
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
            if is_equivalent_type(annotation, bound_ty) {
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
    let ty = if is_equivalent_type(then_ty.clone(), else_ty) {
        if is_equivalent_type(cond.deduced_type.clone().unwrap(), bool_lit_ty()) {
            then_ty
        } else {
            panic!("Type mismatch on if condtion");
        }
    } else {
        panic!("Type mismatch between then and else.")
    };
    conditional(cond, then_expr, else_expr, ei.source.clone()).with_deduced_type(ty)
}

fn deduce_apptype(
    ei: Arc<ExprInfo>,
    expr: Arc<ExprInfo>,
    arg_ty: Arc<Type>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let expr = deduce_expr(expr, scope);
    let arg_ty = reduce_type(arg_ty, &mut Scope::<LocalTypeVar>::empty());
    let ty = match &*expr.deduced_type.clone().unwrap() {
        Type::ForAllTy(var, val_ty) => {
            let mut ty_scope = Scope::<LocalTypeVar>::empty();
            ty_scope.push(&var.name, &LocalTypeVar { ty: arg_ty.clone() });
            reduce_type(val_ty.clone(), &mut ty_scope)
        }
        _ => {
            panic!("Applying type requires forall.")
        }
    };
    app_ty(expr, arg_ty, ei.source.clone()).with_deduced_type(ty)
}

fn deduce_forall(
    ei: Arc<ExprInfo>,
    tyvar: Arc<TyVar>,
    expr: Arc<ExprInfo>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let expr = deduce_expr(expr, scope);
    let ty = type_forall(tyvar.clone(), expr.deduced_type.clone().unwrap());
    forall(tyvar, expr, ei.source.clone()).with_deduced_type(ty)
}

fn reduce_type(ty: Arc<Type>, scope: &mut Scope<LocalTypeVar>) -> Arc<Type> {
    match &*ty {
        Type::AppTy(fun_ty, arg_ty) => {
            let arg_ty = reduce_type(arg_ty.clone(), scope);
            match &**fun_ty {
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
            Some(local_ty) => local_ty.ty,
            None => ty,
        },
        Type::LitTy(_) => ty,
        Type::TyConApp(tycon, arg_tys) => {
            let arg_tys: Vec<Arc<Type>> = arg_tys
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
                    ty: Arc::new(Type::TyVar(var.clone())),
                },
            );
            let val_ty = reduce_type(val_ty.clone(), scope);
            scope.pop(&var.name);
            type_forall(var.clone(), val_ty)
        }
    }
}

pub fn is_equivalent_type(lhs: Arc<Type>, rhs: Arc<Type>) -> bool {
    let mut lhs_scope = Scope::<u32>::empty();
    let mut rhs_scope = Scope::<u32>::empty();
    let mut next_id: u32 = 0;
    is_equivalent_type_inner(lhs, rhs, &mut lhs_scope, &mut rhs_scope, &mut next_id)
}

// "for<a> a" is equivalent to "for<b> b".
fn is_equivalent_type_inner(
    lhs: Arc<Type>,
    rhs: Arc<Type>,
    lhs_scope: &mut Scope<u32>, // name of type variable -> identifier
    rhs_scope: &mut Scope<u32>, // name of type variable -> identifier
    next_id: &mut u32,
) -> bool {
    match &*lhs {
        Type::TyVar(lhs_var) => match &*rhs {
            Type::TyVar(rhs_var) => {
                let lhs = lhs_scope.get(&lhs_var.name);
                let rhs = rhs_scope.get(&rhs_var.name);
                if lhs.is_none() {
                    if rhs.is_some() {
                        return false;
                    }
                    lhs_var == rhs_var
                } else {
                    if rhs.is_none() {
                        return false;
                    }
                    let lhs = lhs.unwrap();
                    let rhs = rhs.unwrap();
                    lhs == rhs
                }
            }
            _ => false,
        },
        Type::LitTy(lhs_lit) => match &*rhs {
            Type::LitTy(rhs_lit) => lhs_lit.id == rhs_lit.id,
            _ => false,
        },
        Type::AppTy(lhs_func_ty, lhs_arg_ty) => match &*rhs {
            Type::AppTy(rhs_func_ty, rhs_arg_ty) => {
                is_equivalent_type_inner(
                    lhs_func_ty.clone(),
                    rhs_func_ty.clone(),
                    lhs_scope,
                    rhs_scope,
                    next_id,
                ) && is_equivalent_type_inner(
                    lhs_arg_ty.clone(),
                    rhs_arg_ty.clone(),
                    lhs_scope,
                    rhs_scope,
                    next_id,
                )
            }
            _ => false,
        },
        Type::TyConApp(lhs_tycon, lhs_args) => match &*rhs {
            Type::TyConApp(rhs_tycon, rhs_args) => {
                if lhs_tycon != rhs_tycon {
                    return false;
                }
                if lhs_args.len() != rhs_args.len() {
                    return false;
                }
                for i in 0..lhs_args.len() {
                    if !is_equivalent_type_inner(
                        lhs_args[i].clone(),
                        rhs_args[i].clone(),
                        lhs_scope,
                        rhs_scope,
                        next_id,
                    ) {
                        return false;
                    }
                }
                return true;
            }
            _ => false,
        },
        Type::FunTy(lhs_param_ty, lhs_val_ty) => match &*rhs {
            Type::FunTy(rhs_param_ty, rhs_val_ty) => {
                is_equivalent_type_inner(
                    lhs_param_ty.clone(),
                    rhs_param_ty.clone(),
                    lhs_scope,
                    rhs_scope,
                    next_id,
                ) && is_equivalent_type_inner(
                    lhs_val_ty.clone(),
                    rhs_val_ty.clone(),
                    lhs_scope,
                    rhs_scope,
                    next_id,
                )
            }
            _ => false,
        },
        Type::ForAllTy(lhs_tyvar, lhs_val_ty) => match &*rhs {
            Type::ForAllTy(rhs_tyvar, rhs_val_ty) => {
                lhs_scope.push(&lhs_tyvar.name, next_id);
                rhs_scope.push(&rhs_tyvar.name, next_id);
                *next_id += 1;
                let ret = is_equivalent_type_inner(
                    lhs_val_ty.clone(),
                    rhs_val_ty.clone(),
                    lhs_scope,
                    rhs_scope,
                    next_id,
                );
                rhs_scope.pop(&rhs_tyvar.name);
                lhs_scope.pop(&lhs_tyvar.name);
                ret
            }
            _ => false,
        },
    }
}
