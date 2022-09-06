use super::*;

// #[derive(Debug)]
// pub struct TypeError {}

#[derive(Clone)]
struct LocalTermVar {
    ty: Arc<Type>,
}

#[derive(Clone)]
struct LocalTypeVar {/* field for type class */}

#[derive(Default)]
struct Scope {
    term_var: HashMap<String, Vec<LocalTermVar>>,
    type_var: HashMap<String, Vec<LocalTypeVar>>,
}

impl Scope {
    // TODO: throw TypeError when unwrap fails.
    fn push_term(self: &mut Self, name: &str, ty: &LocalTermVar) {
        if !self.term_var.contains_key(name) {
            self.term_var.insert(String::from(name), Default::default());
        }
        self.term_var.get_mut(name).unwrap().push(ty.clone());
    }
    fn pop_term(self: &mut Self, name: &str) {
        self.term_var.get_mut(name).unwrap().pop();
        if self.term_var.get(name).unwrap().is_empty() {
            self.term_var.remove(name);
        }
    }
    fn get_term(self: &Self, name: &str) -> LocalTermVar {
        self.term_var.get(name).unwrap().last().unwrap().clone()
    }

    fn push_type(self: &mut Self, name: &str) {
        if !self.type_var.contains_key(name) {
            self.type_var.insert(String::from(name), Default::default());
        }
        self.type_var.get_mut(name).unwrap().push(LocalTypeVar {});
    }
    fn pop_type(self: &mut Self, name: &str) {
        self.type_var.get_mut(name).unwrap().pop();
        if self.type_var.get(name).unwrap().is_empty() {
            self.type_var.remove(name);
        }
    }
    fn get_type(self: &Self, name: &str) -> LocalTypeVar {
        self.type_var.get(name).unwrap().last().unwrap().clone()
    }
}

pub fn check_type(ei: Arc<ExprInfo>) -> Arc<ExprInfo> {
    let mut scope: Scope = Default::default();
    check_expr(ei, &mut scope)
}

fn check_expr(ei: Arc<ExprInfo>, scope: &mut Scope) -> Arc<ExprInfo> {
    match &*ei.expr {
        Expr::Var(v) => check_var(v.clone(), scope),
        Expr::Lit(lit) => check_lit(lit.clone(), scope),
        Expr::App(func, arg) => check_app(func.clone(), arg.clone(), scope),
        Expr::Lam(arg, val) => check_lam(arg.clone(), val.clone(), scope),
        Expr::Let(var, bound, val) => check_let(var.clone(), bound.clone(), val.clone(), scope),
        Expr::If(_, _, _) => todo!(),
        Expr::AppType(_, _) => todo!(),
        Expr::ForAll(_, _) => todo!(),
    }
}

fn check_var(var: Arc<Var>, scope: &mut Scope) -> Arc<ExprInfo> {
    let ty = scope.get_term(&var.name).ty;
    let ty = match &var.type_annotation {
        None => ty,
        Some(ty_anno) => {
            if ty == *ty_anno {
                ty
            } else {
                panic!("Type mismatch at {}.", var.name)
            }
        }
    };
    Arc::new(Expr::Var(var))
        .into_expr_info()
        .with_deduced_type(ty)
}

fn check_lit(lit: Arc<Literal>, _scope: &mut Scope) -> Arc<ExprInfo> {
    let lit_ty = lit.ty.clone();
    Arc::new(Expr::Lit(lit))
        .into_expr_info()
        .with_deduced_type(lit_ty.clone())
}

fn check_app(func: Arc<ExprInfo>, arg: Arc<ExprInfo>, scope: &mut Scope) -> Arc<ExprInfo> {
    let func = check_expr(func, scope);
    let arg = check_expr(arg, scope);
    let ty = match &*func.deduced_type.clone().unwrap() {
        Type::FunTy(arg_ty, result_ty) => {
            if *arg_ty != *arg.deduced_type.as_ref().unwrap() {
                panic!("Type mismatch at {}.", arg.expr.to_string())
            }
            result_ty.clone()
        }
        _ => panic!("Function required at {}.", func.expr.to_string()),
    };
    app(func, arg).with_deduced_type(ty)
}

fn check_lam(arg: Arc<Var>, val: Arc<ExprInfo>, scope: &mut Scope) -> Arc<ExprInfo> {
    let arg_ty = arg.type_annotation.clone().unwrap();
    scope.push_term(&arg.name, &LocalTermVar { ty: arg_ty.clone() });
    let val = check_expr(val, scope);
    scope.pop_term(&arg.name);
    let val_ty = val.deduced_type.clone().unwrap();
    lam(arg, val).with_deduced_type(lam_ty(arg_ty, val_ty))
}

fn check_let(
    var: Arc<Var>,
    bound: Arc<ExprInfo>,
    val: Arc<ExprInfo>,
    scope: &mut Scope,
) -> Arc<ExprInfo> {
    let bound = check_expr(bound, scope);
    let bound_ty = bound.deduced_type.clone().unwrap();
    if var.type_annotation.is_some() && var.type_annotation.clone().unwrap() != bound_ty {
        panic!(
            "Type mismatch on let {} = {}; ...",
            var.name,
            bound.expr.to_string()
        );
    }
    scope.push_term(&var.name, &LocalTermVar { ty: bound_ty });
    let val = check_expr(val, scope);
    scope.pop_term(&var.name);
    let_in(var, bound, val)
}
