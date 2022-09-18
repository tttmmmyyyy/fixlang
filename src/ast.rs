use super::*;

// Haskell Core:
//
// data Expr
//   = Var Var
//   | Lit Literal
//   | App Expr Expr
//   | Lam Var Expr -- Both term and type lambda
//   | Let Bind Expr
//   | Case Expr Var Type [(AltCon, [Var], Expr)]
//   | Type Type -- Used for type application

// data Var = Id Name Type -- Term variable
//   | TyVar Name Kind -- Type variable

// data Type = TyVarTy Var
//   | LitTy TyLit
//   | AppTy Type Type
//   | TyConApp TyCon [Type]
//   | FunTy Type Type
//   | ForAllTy Var Type

pub struct ExprInfo {
    pub expr: Arc<Expr>,
    pub free_vars: HashSet<String>,
    pub deduced_type: Option<Arc<Type>>,
}

impl ExprInfo {
    fn with_free_vars(self: &Arc<Self>, free_vars: HashSet<String>) -> Arc<Self> {
        Arc::new(ExprInfo {
            expr: self.expr.clone(),
            free_vars,
            deduced_type: self.deduced_type.clone(),
        })
    }
    pub fn with_deduced_type(self: &Arc<Self>, ty: Arc<Type>) -> Arc<Self> {
        Arc::new(ExprInfo {
            expr: self.expr.clone(),
            free_vars: self.free_vars.clone(),
            deduced_type: Some(ty),
        })
    }
}

#[derive(Clone)]
pub enum Expr {
    Var(Arc<Var>),
    Lit(Arc<Literal>),
    App(Arc<ExprInfo>, Arc<ExprInfo>),
    Lam(Arc<Var>, Arc<ExprInfo>),
    Let(Arc<Var>, Arc<ExprInfo>, Arc<ExprInfo>),
    If(Arc<ExprInfo>, Arc<ExprInfo>, Arc<ExprInfo>), // TODO: Implement case
    AppType(Arc<ExprInfo>, Arc<Type>),
    ForAll(Arc<TyVar>, Arc<ExprInfo>),
}

impl Expr {
    pub fn into_expr_info(self: &Arc<Self>) -> Arc<ExprInfo> {
        Arc::new(ExprInfo {
            expr: self.clone(),
            free_vars: Default::default(),
            deduced_type: None,
        })
    }
    pub fn to_string(&self) -> String {
        match self {
            Expr::Var(v) => v.name.clone(),
            Expr::Lit(l) => l.name.clone(),
            Expr::App(f, a) => format!("({}) ({})", f.expr.to_string(), a.expr.to_string()),
            Expr::Lam(x, fx) => format!("\\{}->({})", x.name, fx.expr.to_string()),
            Expr::Let(x, b, v) => format!(
                "let {}={} in ({})",
                x.name,
                b.expr.to_string(),
                v.expr.to_string()
            ),
            Expr::If(c, t, e) => format!(
                "if {} then {} else ({})",
                c.expr.to_string(),
                t.expr.to_string(),
                e.expr.to_string()
            ),
            Expr::AppType(expr, ty) => {
                format!("({})<{}>", expr.expr.to_string(), ty.to_string())
            }
            Expr::ForAll(tyvar, expr) => {
                format!("for<{}> ({})", tyvar.name, expr.expr.to_string())
            }
        }
    }
}

pub type LiteralGenerator =
    dyn Send + Sync + for<'c, 'm, 'b> Fn(&mut GenerationContext<'c, 'm>) -> PointerValue<'c>;

pub struct Literal {
    pub generator: Arc<LiteralGenerator>,
    pub free_vars: Vec<String>, // e.g. "+" literal has two free variables.
    name: String,
    pub ty: Arc<Type>,
}

#[derive(Eq, PartialEq)]
pub struct Var {
    pub name: String,
    pub type_annotation: Option<Arc<Type>>,
}

#[derive(Eq, PartialEq)]
pub struct TyVar {
    pub name: String,
}

// impl Var {
//     pub fn name(self: &Self) -> &String {
//         match self {
//             Var::TermVar {
//                 name,
//                 type_annotation: _,
//             } => name,
//             Var::TyVar { name } => name,
//         }
//     }
// }

#[derive(Eq, PartialEq)]
pub enum Kind {
    Star,
    Arrow(Arc<Kind>, Arc<Kind>),
}

#[derive(Eq, PartialEq)]
pub struct TyLit {
    pub id: u32,
    pub name: String,
}

#[derive(Eq, PartialEq)]
pub enum Type {
    TyVar(Arc<TyVar>),
    LitTy(Arc<TyLit>),
    AppTy(Arc<Type>, Arc<Type>),
    TyConApp(Arc<TyCon>, Vec<Arc<Type>>),
    FunTy(Arc<Type>, Arc<Type>),
    ForAllTy(Arc<TyVar>, Arc<Type>),
}

impl Type {
    fn to_string(&self) -> String {
        todo!()
    }
}

#[derive(Eq, PartialEq)]
pub struct TyCon {
    pub name: String,
    id: u32,
    pub arity: u32, // kind: Arc<Kind>,
}

pub fn star_kind() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

pub fn arrow_kind(src: Arc<Kind>, dst: Arc<Kind>) -> Arc<Kind> {
    Arc::new(Kind::Arrow(src, dst))
}

pub fn lam_ty(src: Arc<Type>, dst: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::FunTy(src, dst))
}

pub fn tyvar_var(var_name: &str) -> Arc<TyVar> {
    Arc::new(TyVar {
        name: String::from(var_name),
    })
}

pub fn tyvar_ty(var_name: &str) -> Arc<Type> {
    Arc::new(Type::TyVar(tyvar_var(var_name)))
}

fn forall_ty(var_name: &str, ty: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::ForAllTy(tyvar_var(var_name), ty))
}

pub fn var_var(var_name: &str, type_annotation: Option<Arc<Type>>) -> Arc<Var> {
    Arc::new(Var {
        name: String::from(var_name),
        type_annotation,
    })
}

pub fn lit(
    generator: Arc<LiteralGenerator>,
    free_vars: Vec<String>,
    name: String,
    ty: Arc<Type>,
) -> Arc<ExprInfo> {
    Arc::new(Expr::Lit(Arc::new(Literal {
        generator,
        free_vars,
        name,
        ty,
    })))
    .into_expr_info()
}

pub fn let_in(var: Arc<Var>, bound: Arc<ExprInfo>, expr: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::Let(var, bound, expr)).into_expr_info()
}

pub fn lam(var: Arc<Var>, val: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::Lam(var, val)).into_expr_info()
}

pub fn app(lam: Arc<ExprInfo>, arg: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::App(lam, arg)).into_expr_info()
}

// Make variable expression.
pub fn var(var_name: &str) -> Arc<ExprInfo> {
    Arc::new(Expr::Var(var_var(var_name, None))).into_expr_info()
}

pub fn conditional(
    cond: Arc<ExprInfo>,
    then_expr: Arc<ExprInfo>,
    else_expr: Arc<ExprInfo>,
) -> Arc<ExprInfo> {
    Arc::new(Expr::If(cond, then_expr, else_expr)).into_expr_info()
}

pub fn app_ty(expr: Arc<ExprInfo>, ty: Arc<Type>) -> Arc<ExprInfo> {
    Arc::new(Expr::AppType(expr, ty)).into_expr_info()
}

pub fn forall(var: Arc<TyVar>, val: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::ForAll(var, val)).into_expr_info()
}

pub fn lit_ty(id: u32, name: &str) -> Arc<Type> {
    Arc::new(Type::LitTy(Arc::new(TyLit {
        id,
        name: String::from(name),
    })))
}

pub fn tycon(id: u32, name: &str, arity: u32) -> Arc<TyCon> {
    // let mut kind = star_kind();
    // for _ in 0..arity {
    //     kind = arrow_kind(star_kind(), kind);
    // }
    Arc::new(TyCon {
        id,
        name: String::from(name),
        arity,
    })
}

pub fn type_app(head: Arc<Type>, param: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::AppTy(head, param))
}

pub fn type_fun(src: Arc<Type>, dst: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::FunTy(src, dst))
}

pub fn type_forall(var: Arc<TyVar>, ty: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::ForAllTy(var, ty))
}

pub fn tycon_app(tycon: Arc<TyCon>, params: Vec<Arc<Type>>) -> Arc<Type> {
    Arc::new(Type::TyConApp(tycon, params))
}

// TODO: use persistent binary search tree as ExprAuxInfo to avoid O(n^2) complexity of calculate_aux_info.
pub fn calculate_free_vars(ei: Arc<ExprInfo>) -> Arc<ExprInfo> {
    match &*ei.expr {
        Expr::Var(var) => {
            let free_vars = vec![var.name.clone()].into_iter().collect();
            ei.with_free_vars(free_vars)
        }
        Expr::Lit(lit) => {
            let free_vars = lit.free_vars.clone().into_iter().collect();
            ei.with_free_vars(free_vars)
        }
        Expr::App(func, arg) => {
            let func = calculate_free_vars(func.clone());
            let arg = calculate_free_vars(arg.clone());
            let mut free_vars = func.free_vars.clone();
            free_vars.extend(arg.free_vars.clone());
            app(func, arg).with_free_vars(free_vars)
        }
        Expr::Lam(arg, val) => {
            let val = calculate_free_vars(val.clone());
            let mut free_vars = val.free_vars.clone();
            free_vars.remove(&arg.name);
            free_vars.remove(SELF_NAME);
            lam(arg.clone(), val).with_free_vars(free_vars)
        }
        Expr::Let(var, bound, val) => {
            // NOTE: Our Let is non-recursive let, i.e.,
            // "let x = f x in g x" is equal to "let y = f x in g y",
            // and x ∈ FreeVars("let x = f x in g x") = (FreeVars(g x) - {x}) + FreeVars(f x) != (FreeVars(g x) + FreeVars(f x)) - {x}.
            let bound = calculate_free_vars(bound.clone());
            let val = calculate_free_vars(val.clone());
            let mut free_vars = val.free_vars.clone();
            free_vars.remove(&var.name);
            free_vars.extend(bound.free_vars.clone());
            let_in(var.clone(), bound, val).with_free_vars(free_vars)
        }
        Expr::If(cond, then, else_expr) => {
            let cond = calculate_free_vars(cond.clone());
            let then = calculate_free_vars(then.clone());
            let else_expr = calculate_free_vars(else_expr.clone());
            let mut free_vars = cond.free_vars.clone();
            free_vars.extend(then.free_vars.clone());
            free_vars.extend(else_expr.free_vars.clone());
            conditional(cond, then, else_expr).with_free_vars(free_vars)
        }
        Expr::AppType(ei, ty) => {
            let ei = calculate_free_vars(ei.clone());
            app_ty(ei.clone(), ty.clone()).with_free_vars(ei.free_vars.clone())
        }
        Expr::ForAll(tyvar, ei) => {
            let ei = calculate_free_vars(ei.clone());
            forall(tyvar.clone(), ei.clone()).with_free_vars(ei.free_vars.clone())
        }
    }
}
