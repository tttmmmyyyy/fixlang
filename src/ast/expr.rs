use super::super::*;
use std::{collections::HashSet, sync::Arc};

pub struct ExprInfo {
    pub expr: Arc<Expr>,
    pub free_vars: HashSet<String>,
    pub deduced_type: Option<Arc<TypeNode>>,
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
    pub fn with_deduced_type(self: &Arc<Self>, ty: Arc<TypeNode>) -> Arc<Self> {
        let ty = ty.calculate_free_vars();
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
    AppType(Arc<ExprInfo>, Arc<TypeNode>),
    ForAll(Arc<TyVar>, Arc<ExprInfo>),
}

impl Expr {
    pub fn into_expr_info(self: &Arc<Self>, src: Option<Span>) -> Arc<ExprInfo> {
        Arc::new(ExprInfo {
            expr: self.clone(),
            free_vars: Default::default(),
            deduced_type: None,
            source: src,
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
    pub ty: Arc<TypeNode>,
}

pub struct Var {
    pub name: String,
    pub type_annotation: Option<Arc<TypeNode>>,
    pub source: Option<Span>,
}

pub fn var_var(
    var_name: &str,
    type_annotation: Option<Arc<TypeNode>>,
    src: Option<Span>,
) -> Arc<Var> {
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
    ty: Arc<TypeNode>,
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

pub fn expr_appty(expr: Arc<ExprInfo>, ty: Arc<TypeNode>, src: Option<Span>) -> Arc<ExprInfo> {
    Arc::new(Expr::AppType(expr, ty)).into_expr_info(src)
}

pub fn expr_forall(var: Arc<TyVar>, val: Arc<ExprInfo>, src: Option<Span>) -> Arc<ExprInfo> {
    Arc::new(Expr::ForAll(var, val)).into_expr_info(src)
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
            expr_appty(ei.clone(), ty.clone(), ei.source.clone())
                .with_free_vars(ei.free_vars.clone())
        }
        Expr::ForAll(tyvar, ei) => {
            let ei = calculate_free_vars(ei.clone());
            expr_forall(tyvar.clone(), ei.clone(), ei.source.clone())
                .with_free_vars(ei.free_vars.clone())
        }
    }
}
