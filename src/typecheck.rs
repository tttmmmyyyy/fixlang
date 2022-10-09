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
    ty: Arc<TypeInfo>,
}

#[derive(Clone)]
struct LocalTypeVar {
    ty: Arc<TypeInfo>,
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

// Additional information on types.
#[derive(Default, Clone)]
pub struct TypeAdditionalInfo {
    free_vars: Option<HashSet<String>>,
}

// Node of type ast tree that we usually use.
pub type TypeInfo = TypeNode<TypeAdditionalInfo>;

impl TypeInfo {
    // Calculate free type variables.
    pub fn calculate_free_vars(self: &Arc<Self>) -> Arc<Self> {
        if self.info.free_vars.is_some() {
            return self.clone();
        }
        let mut free_vars = HashSet::<String>::default();
        let ty = match &self.ty {
            Type::TyVar(tv) => {
                free_vars.insert(tv.name.clone());
                self.ty.clone()
            }
            Type::LitTy(_) => self.ty.clone(),
            Type::AppTy(forallty, argty) => {
                let forallty = forallty.calculate_free_vars();
                let argty = argty.calculate_free_vars();
                free_vars.extend(forallty.info.free_vars.clone().unwrap());
                Type::AppTy(forallty, argty)
            }
            Type::TyConApp(tycon, args) => {
                let tycon = tycon.clone();
                let args: Vec<Arc<Self>> = args.iter().map(|ty| ty.calculate_free_vars()).collect();
                for arg in args.iter() {
                    free_vars.extend(arg.info.free_vars.clone().unwrap());
                }
                Type::TyConApp(tycon, args)
            }
            Type::FunTy(input, output) => {
                let input = input.calculate_free_vars();
                let output = output.calculate_free_vars();
                free_vars.extend(input.info.free_vars.clone().unwrap());
                free_vars.extend(output.info.free_vars.clone().unwrap());
                Type::FunTy(input, output)
            }
            Type::ForAllTy(var, body) => {
                let body = body.calculate_free_vars();
                free_vars.extend(body.info.free_vars.clone().unwrap());
                free_vars.remove(&var.name);
                Type::ForAllTy(var.clone(), body)
            }
        };
        self.set_ty(ty).set_free_vars(free_vars)
    }

    // Set free variables.
    pub fn set_free_vars(self: &Arc<Self>, free_vars: HashSet<String>) -> Arc<Self> {
        let mut info = (*self.info).clone();
        info.free_vars = Some(free_vars);
        self.clone().set_info(Arc::new(info))
    }
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
    let fun_ty = func.deduced_type.clone().unwrap();
    let ty = match &fun_ty.ty {
        Type::FunTy(param_ty, result_ty) => {
            if is_equivalent_type(param_ty.clone(), arg_ty.clone()) {
                result_ty.clone()
            } else {
                error_exit_with_src(
                    &format!(
                        "expected {}, found {}",
                        &param_ty.clone().to_string(),
                        &arg_ty.clone().to_string(),
                    ),
                    &arg.source,
                )
            }
        }
        _ => error_exit_with_src(
            &format!(
                "an expression of type {} is not a function but applied to something\n",
                &fun_ty.clone().to_string()
            ),
            &func.source,
        ),
    };
    expr_app(func, arg, ei.source.clone()).with_deduced_type(ty)
}

// Transform a type of form "for<...> x => y" to "for<a1,...,an> x => for<...> y" as far as possible,
// where a1,...,an are type variables used in x.
fn defer_forall_of_fun(ty: Arc<TypeInfo>) -> Arc<TypeInfo> {
    unimplemented!()
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
    let ty = if is_equivalent_type(then_ty.clone(), else_ty.clone()) {
        if is_equivalent_type(cond.deduced_type.clone().unwrap(), bool_lit_ty()) {
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
    arg_ty: Arc<TypeInfo>,
    scope: &mut Scope<LocalTermVar>,
) -> Arc<ExprInfo> {
    let expr = deduce_expr(expr, scope);
    let arg_ty = reduce_type(arg_ty, &mut Scope::<LocalTypeVar>::empty());
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

fn reduce_type(ty: Arc<TypeInfo>, scope: &mut Scope<LocalTypeVar>) -> Arc<TypeInfo> {
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
            Some(local_ty) => local_ty.ty,
            None => ty,
        },
        Type::LitTy(_) => ty,
        Type::TyConApp(tycon, arg_tys) => {
            let arg_tys: Vec<Arc<TypeInfo>> = arg_tys
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

pub fn is_equivalent_type(lhs: Arc<TypeInfo>, rhs: Arc<TypeInfo>) -> bool {
    let mut lhs_scope = Scope::<u32>::empty();
    let mut rhs_scope = Scope::<u32>::empty();
    let mut next_id: u32 = 0;
    is_equivalent_type_inner(lhs, rhs, &mut lhs_scope, &mut rhs_scope, &mut next_id)
}

// "for<a> a" is equivalent to "for<b> b".
fn is_equivalent_type_inner(
    lhs: Arc<TypeInfo>,
    rhs: Arc<TypeInfo>,
    lhs_scope: &mut Scope<u32>, // name of type variable -> identifier
    rhs_scope: &mut Scope<u32>, // name of type variable -> identifier
    next_id: &mut u32,
) -> bool {
    match &lhs.ty {
        Type::TyVar(lhs_var) => match &rhs.ty {
            Type::TyVar(rhs_var) => {
                let lhs = lhs_scope.get(&lhs_var.name);
                let rhs = rhs_scope.get(&rhs_var.name);
                if lhs.is_none() {
                    if rhs.is_some() {
                        return false;
                    }
                    *lhs_var == *rhs_var
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
        Type::LitTy(lhs_lit) => match &rhs.ty {
            Type::LitTy(rhs_lit) => lhs_lit.id == rhs_lit.id,
            _ => false,
        },
        Type::AppTy(lhs_func_ty, lhs_arg_ty) => match &rhs.ty {
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
        Type::TyConApp(lhs_tycon, lhs_args) => match &rhs.ty {
            Type::TyConApp(rhs_tycon, rhs_args) => {
                if *lhs_tycon != *rhs_tycon {
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
        Type::FunTy(lhs_param_ty, lhs_val_ty) => match &rhs.ty {
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
        Type::ForAllTy(lhs_tyvar, lhs_val_ty) => match &rhs.ty {
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
