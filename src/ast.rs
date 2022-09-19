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
    pub source: Option<Span>,
}

impl ExprInfo {
    // Add free vars
    fn with_free_vars(self: &Arc<Self>, free_vars: HashSet<String>) -> Arc<Self> {
        Arc::new(ExprInfo {
            expr: self.expr.clone(),
            free_vars,
            deduced_type: self.deduced_type.clone(),
            source: self.source.clone(),
        })
    }

    // Add deduced type
    pub fn with_deduced_type(self: &Arc<Self>, ty: Arc<Type>) -> Arc<Self> {
        Arc::new(ExprInfo {
            expr: self.expr.clone(),
            free_vars: self.free_vars.clone(),
            deduced_type: Some(ty),
            source: self.source.clone(),
        })
    }

    // Add source
    pub fn with_source(self: &Arc<Self>, src: Option<Span>) -> Arc<Self> {
        Arc::new(ExprInfo {
            expr: self.expr.clone(),
            free_vars: self.free_vars.clone(),
            deduced_type: self.deduced_type.clone(),
            source: src,
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
    pub fn into_expr_info(self: &Arc<Self>, src: Option<Span>) -> Arc<ExprInfo> {
        Arc::new(ExprInfo {
            expr: self.clone(),
            free_vars: Default::default(),
            deduced_type: None,
            source: None,
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
                format!("({})<{}>", expr.expr.to_string(), ty.clone().to_string())
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

pub struct Var {
    pub name: String,
    pub type_annotation: Option<Arc<Type>>,
    pub source: Option<Span>,
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
    pub fn to_string(self: Arc<Type>) -> String {
        match &*self {
            Type::TyVar(v) => v.name.clone(),
            Type::LitTy(l) => l.name.clone(),
            Type::AppTy(_, _) => {
                let (ty, args) = self.decompose_appty();
                let ty = ty.to_string();
                let args: Vec<String> = args.iter().map(|a| a.clone().to_string()).collect();
                let mut res: String = Default::default();
                res += &ty;
                res += "<";
                res += &args.join(", ");
                res += ">";
                res
            }
            Type::TyConApp(tycon, args) => {
                let tycon = tycon.name.clone();
                let args: Vec<String> = args.iter().map(|a| a.clone().to_string()).collect();
                let mut res: String = Default::default();
                res += &tycon;
                res += "<";
                res += &args.join(", ");
                res += ">";
                res
            }
            Type::FunTy(src, dst) => {
                let src_brace_needed = match &**src {
                    Type::FunTy(_, _) => true,
                    Type::ForAllTy(_, _) => true,
                    _ => false,
                };
                let src = src.clone().to_string();
                let dst = dst.clone().to_string();
                let mut res: String = Default::default();
                if src_brace_needed {
                    res += "(";
                    res += &src;
                    res += ")";
                } else {
                    res += &src;
                }
                res += " => ";
                res += &dst;
                res
            }
            Type::ForAllTy(_, _) => {
                let (vars, ty) = self.decompose_forall();
                let vars: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
                let mut res: String = Default::default();
                res += "for<";
                res += &vars.join(", ");
                res += "> ";
                res += &ty.to_string();
                res
            }
        }
    }

    // Decompose AppTy as many as possible.
    // Example: a<b, c> --> (a, vec![b, c])
    fn decompose_appty(self: Arc<Type>) -> (Arc<Type>, Vec<Arc<Type>>) {
        match &*self {
            Type::AppTy(ty, arg) => {
                let (ty, mut args) = ty.clone().decompose_appty();
                args.push(arg.clone());
                (ty, args)
            }
            _ => (self.clone(), vec![]),
        }
    }

    // Decompose ForAllTy as many as possible.
    // Example: for<b, c> a --> (vec![b, c], a)
    fn decompose_forall(self: Arc<Type>) -> (Vec<Arc<TyVar>>, Arc<Type>) {
        let (mut vars, ty) = self.decompose_forall_inner();
        vars.reverse();
        (vars, ty)
    }

    // Decompose ForAllTy as many as possible. (vars reversed)
    fn decompose_forall_inner(self: Arc<Type>) -> (Vec<Arc<TyVar>>, Arc<Type>) {
        match &*self {
            Type::ForAllTy(var, ty) => {
                let (mut vars, ty) = ty.clone().decompose_forall();
                vars.push(var.clone());
                (vars, ty)
            }
            _ => (vec![], self.clone()),
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct TyCon {
    pub name: String,
    pub arity: u32, // kind: Arc<Kind>,
}

pub fn tycon(name: &str, arity: u32) -> Arc<TyCon> {
    Arc::new(TyCon {
        name: String::from(name),
        arity,
    })
}

pub fn star_kind() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

pub fn arrow_kind(src: Arc<Kind>, dst: Arc<Kind>) -> Arc<Kind> {
    Arc::new(Kind::Arrow(src, dst))
}

pub fn type_func(src: Arc<Type>, dst: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::FunTy(src, dst))
}

pub fn var_tyvar(var_name: &str) -> Arc<TyVar> {
    Arc::new(TyVar {
        name: String::from(var_name),
    })
}

pub fn type_tyvar(var_name: &str) -> Arc<Type> {
    Arc::new(Type::TyVar(var_tyvar(var_name)))
}

pub fn var_var(var_name: &str, type_annotation: Option<Arc<Type>>, src: Option<Span>) -> Arc<Var> {
    Arc::new(Var {
        name: String::from(var_name),
        type_annotation,
        source: src,
    })
}

pub fn expr_lit(
    generator: Arc<LiteralGenerator>,
    free_vars: Vec<String>,
    name: String,
    ty: Arc<Type>,
    src: Option<Span>,
) -> Arc<ExprInfo> {
    Arc::new(Expr::Lit(Arc::new(Literal {
        generator,
        free_vars,
        name,
        ty,
    })))
    .into_expr_info(src)
}

pub fn expr_let(
    var: Arc<Var>,
    bound: Arc<ExprInfo>,
    expr: Arc<ExprInfo>,
    src: Option<Span>,
) -> Arc<ExprInfo> {
    Arc::new(Expr::Let(var, bound, expr)).into_expr_info(src)
}

pub fn expr_abs(var: Arc<Var>, val: Arc<ExprInfo>, src: Option<Span>) -> Arc<ExprInfo> {
    Arc::new(Expr::Lam(var, val)).into_expr_info(src)
}

pub fn expr_app(lam: Arc<ExprInfo>, arg: Arc<ExprInfo>, src: Option<Span>) -> Arc<ExprInfo> {
    Arc::new(Expr::App(lam, arg)).into_expr_info(src)
}

// Make variable expression.
pub fn expr_var(var_name: &str, src: Option<Span>) -> Arc<ExprInfo> {
    Arc::new(Expr::Var(var_var(var_name, None, src.clone()))).into_expr_info(src)
}

pub fn expr_if(
    cond: Arc<ExprInfo>,
    then_expr: Arc<ExprInfo>,
    else_expr: Arc<ExprInfo>,
    src: Option<Span>,
) -> Arc<ExprInfo> {
    Arc::new(Expr::If(cond, then_expr, else_expr)).into_expr_info(src)
}

pub fn app_ty(expr: Arc<ExprInfo>, ty: Arc<Type>, src: Option<Span>) -> Arc<ExprInfo> {
    Arc::new(Expr::AppType(expr, ty)).into_expr_info(src)
}

pub fn forall(var: Arc<TyVar>, val: Arc<ExprInfo>, src: Option<Span>) -> Arc<ExprInfo> {
    Arc::new(Expr::ForAll(var, val)).into_expr_info(src)
}

pub fn lit_ty(id: u32, name: &str) -> Arc<Type> {
    Arc::new(Type::LitTy(Arc::new(TyLit {
        id,
        name: String::from(name),
    })))
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
            expr_app(func, arg, ei.source.clone()).with_free_vars(free_vars)
        }
        Expr::Lam(arg, val) => {
            let val = calculate_free_vars(val.clone());
            let mut free_vars = val.free_vars.clone();
            free_vars.remove(&arg.name);
            free_vars.remove(SELF_NAME);
            expr_abs(arg.clone(), val, ei.source.clone()).with_free_vars(free_vars)
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
            expr_let(var.clone(), bound, val, ei.source.clone()).with_free_vars(free_vars)
        }
        Expr::If(cond, then, else_expr) => {
            let cond = calculate_free_vars(cond.clone());
            let then = calculate_free_vars(then.clone());
            let else_expr = calculate_free_vars(else_expr.clone());
            let mut free_vars = cond.free_vars.clone();
            free_vars.extend(then.free_vars.clone());
            free_vars.extend(else_expr.free_vars.clone());
            expr_if(cond, then, else_expr, ei.source.clone()).with_free_vars(free_vars)
        }
        Expr::AppType(ei, ty) => {
            let ei = calculate_free_vars(ei.clone());
            app_ty(ei.clone(), ty.clone(), ei.source.clone()).with_free_vars(ei.free_vars.clone())
        }
        Expr::ForAll(tyvar, ei) => {
            let ei = calculate_free_vars(ei.clone());
            forall(tyvar.clone(), ei.clone(), ei.source.clone())
                .with_free_vars(ei.free_vars.clone())
        }
    }
}
