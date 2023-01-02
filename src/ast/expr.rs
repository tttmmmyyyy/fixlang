use super::*;
use std::{collections::HashSet, sync::Arc};

pub type Name = String;

#[derive(Clone, PartialEq)]
pub enum AppSourceCodeOrderType {
    FunctionIsFormer,
    ArgumentIsFormer,
}

#[derive(Clone)]
pub struct ExprNode {
    pub expr: Arc<Expr>,
    pub free_vars: Option<HashSet<NameSpacedName>>,
    pub source: Option<Span>,
    pub app_order: AppSourceCodeOrderType,
    pub inferred_ty: Option<Arc<TypeNode>>,
}

impl ExprNode {
    // Set free vars
    fn set_free_vars(&self, free_vars: HashSet<NameSpacedName>) -> Arc<Self> {
        let mut ret = self.clone();
        ret.free_vars = Some(free_vars);
        Arc::new(ret)
    }

    // Get free vars
    pub fn free_vars(self: &Self) -> &HashSet<NameSpacedName> {
        self.free_vars.as_ref().unwrap()
    }

    // Set source
    pub fn set_source(&self, src: Option<Span>) -> Arc<Self> {
        let mut ret = self.clone();
        ret.source = src;
        Arc::new(ret)
    }

    // Set app order
    pub fn set_app_order(&self, app_order: AppSourceCodeOrderType) -> Arc<Self> {
        let mut ret = self.clone();
        ret.app_order = app_order;
        Arc::new(ret)
    }

    // Set inferred type.
    pub fn set_inferred_type(&self, ty: Arc<TypeNode>) -> Arc<Self> {
        let mut ret = self.clone();
        ret.inferred_ty = Some(ty);
        Arc::new(ret)
    }

    pub fn set_var_namespace(&self, ns: NameSpace) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Var(var) => {
                let var = var.set_namsapce(ns);
                ret.expr = Arc::new(Expr::Var(var))
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_var_var(&self, v: Arc<Var>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Var(_) => ret.expr = Arc::new(Expr::Var(v)),
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_app_func(&self, func: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::App(_, arg) => {
                ret.expr = Arc::new(Expr::App(func, arg.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_app_arg(&self, arg: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::App(func, _) => {
                ret.expr = Arc::new(Expr::App(func.clone(), arg));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_lam_param(&self, param: Arc<Var>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Lam(_, body) => {
                ret.expr = Arc::new(Expr::Lam(param, body.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_lam_body(&self, body: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Lam(arg, _) => {
                ret.expr = Arc::new(Expr::Lam(arg.clone(), body));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_let_var(&self, var: Arc<Var>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Let(_, bound, val) => {
                ret.expr = Arc::new(Expr::Let(var, bound.clone(), val.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_let_bound(&self, bound: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Let(var, _, val) => {
                ret.expr = Arc::new(Expr::Let(var.clone(), bound, val.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_let_value(&self, value: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Let(var, bound, _) => {
                ret.expr = Arc::new(Expr::Let(var.clone(), bound.clone(), value));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_if_cond(&self, cond: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::If(_, then_expr, else_expr) => {
                ret.expr = Arc::new(Expr::If(cond, then_expr.clone(), else_expr.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_if_then(&self, then_expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::If(cond, _, else_expr) => {
                ret.expr = Arc::new(Expr::If(cond.clone(), then_expr, else_expr.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_if_else(&self, else_expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::If(cond, then_expr, _) => {
                ret.expr = Arc::new(Expr::If(cond.clone(), then_expr.clone(), else_expr));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_tyanno_expr(&self, expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::TyAnno(_, t) => {
                ret.expr = Arc::new(Expr::TyAnno(expr, t.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_tyanno_ty(&self, ty: Arc<TypeNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::TyAnno(e, _) => {
                ret.expr = Arc::new(Expr::TyAnno(e.clone(), ty));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_lit_lit(&self, lit: Arc<Literal>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Lit(_) => {
                ret.expr = Arc::new(Expr::Lit(lit));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_make_pair_lhs(&self, lhs: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakePair(_, rhs) => {
                ret.expr = Arc::new(Expr::MakePair(lhs, rhs.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_make_pair_rhs(&self, rhs: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakePair(lhs, _) => {
                ret.expr = Arc::new(Expr::MakePair(lhs.clone(), rhs));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn resolve_namespace(self: &Arc<ExprNode>, ctx: &NameResolutionContext) -> Arc<ExprNode> {
        match &*self.expr {
            Expr::Var(_) => {
                // Resolution of names of variables will be done in type checking phase.
                self.clone()
            }
            Expr::Lit(lit) => {
                let mut lit = lit.as_ref().clone();
                lit.ty = lit.ty.resolve_namespace(ctx);
                self.clone().set_lit_lit(Arc::new(lit))
            }
            Expr::App(fun, arg) => self
                .clone()
                .set_app_func(fun.resolve_namespace(ctx))
                .set_app_arg(arg.resolve_namespace(ctx)),
            Expr::Lam(_, body) => self.clone().set_lam_body(body.resolve_namespace(ctx)),
            Expr::Let(_, bound, value) => self
                .clone()
                .set_let_bound(bound.resolve_namespace(ctx))
                .set_let_value(value.resolve_namespace(ctx)),
            Expr::If(cond, then_expr, else_expr) => self
                .clone()
                .set_if_cond(cond.resolve_namespace(ctx))
                .set_if_then(then_expr.resolve_namespace(ctx))
                .set_if_else(else_expr.resolve_namespace(ctx)),
            Expr::TyAnno(expr, ty) => self
                .clone()
                .set_tyanno_expr(expr.resolve_namespace(ctx))
                .set_tyanno_ty(ty.resolve_namespace(ctx)),
            Expr::MakePair(lhs, rhs) => self
                .clone()
                .set_make_pair_lhs(lhs.resolve_namespace(ctx))
                .set_make_pair_rhs(rhs.resolve_namespace(ctx)),
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Var(Arc<Var>),
    Lit(Arc<Literal>),
    App(Arc<ExprNode>, Arc<ExprNode>),
    Lam(Arc<Var>, Arc<ExprNode>),
    Let(Arc<Var>, Arc<ExprNode>, Arc<ExprNode>),
    If(Arc<ExprNode>, Arc<ExprNode>, Arc<ExprNode>), // TODO: Implement case
    TyAnno(Arc<ExprNode>, Arc<TypeNode>),
    MakePair(Arc<ExprNode>, Arc<ExprNode>), // This node is generated in optimization:
                                            // expresison `(x, y)` is first interpreted as a naive function call `Tuple2.new x y`,
                                            // and it is converted to `MakePair x y` in uncurry optimization.
                                            // `MakePair x y` is compiled to a faster code than function call.
}

impl Expr {
    pub fn into_expr_info(self: &Arc<Self>, src: Option<Span>) -> Arc<ExprNode> {
        Arc::new(ExprNode {
            expr: self.clone(),
            free_vars: Default::default(),
            source: src,
            app_order: AppSourceCodeOrderType::FunctionIsFormer,
            inferred_ty: None,
        })
    }
    pub fn to_string(&self) -> String {
        match self {
            Expr::Var(v) => v.name.to_string(),
            Expr::Lit(l) => l.name.clone(),
            Expr::App(f, a) => format!("({}) ({})", f.expr.to_string(), a.expr.to_string()),
            Expr::Lam(x, fx) => format!("\\{}->({})", x.name.to_string(), fx.expr.to_string()),
            Expr::Let(x, b, v) => format!(
                "let {}={} in ({})",
                x.name.to_string(),
                b.expr.to_string(),
                v.expr.to_string()
            ),
            Expr::If(c, t, e) => format!(
                "if {} then {} else ({})",
                c.expr.to_string(),
                t.expr.to_string(),
                e.expr.to_string()
            ),
            Expr::TyAnno(e, t) => format!("({} : {})", e.expr.to_string(), t.to_string()),
            Expr::MakePair(lhs, rhs) => {
                format!("({}, {})", lhs.expr.to_string(), rhs.expr.to_string())
            }
        }
    }
}

pub type LiteralGenerator = dyn Send
    + Sync
    + for<'c, 'm, 'b> Fn(
        &mut GenerationContext<'c, 'm>,
        &Arc<TypeNode>,     // type of this literal
        Option<Object<'c>>, // rvo
    ) -> Object<'c>;

#[derive(Clone)]
pub struct Literal {
    pub generator: Arc<LiteralGenerator>,
    pub free_vars: Vec<NameSpacedName>, // e.g. "+" literal has two free variables.
    name: String,
    pub ty: Arc<TypeNode>,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct NameSpace {
    pub names: Vec<String>, // Empty implies it is local.
}

const NAMESPACE_SEPARATOR: &str = ".";

impl NameSpace {
    pub fn local() -> Self {
        Self { names: vec![] }
    }

    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    pub fn new_str(names: &[&str]) -> Self {
        Self::new(names.iter().map(|s| s.to_string()).collect())
    }

    pub fn is_local(&self) -> bool {
        self.names.len() == 0
    }

    pub fn to_string(&self) -> String {
        self.names.join(NAMESPACE_SEPARATOR)
    }

    pub fn is_suffix(&self, rhs: &NameSpace) -> bool {
        let n = self.names.len();
        let m = rhs.names.len();
        if n > m {
            return false;
        }
        for i in 0..n {
            if self.names[n - 1 - i] != rhs.names[m - i - 1] {
                return false;
            }
        }
        return true;
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn module(&self) -> Name {
        self.names[0].clone()
    }
}

#[derive(Clone)]
pub struct Var {
    pub name: NameSpacedName,
    pub source: Option<Span>,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct NameSpacedName {
    pub namespace: NameSpace,
    pub name: String,
}

impl NameSpacedName {
    pub fn new(ns: &NameSpace, name: &str) -> Self {
        Self {
            namespace: ns.clone(),
            name: name.to_string(),
        }
    }

    pub fn from_strs(ns: &[&str], name: &str) -> Self {
        Self::new(&NameSpace::new_str(ns), name)
    }

    pub fn local(name: &str) -> Self {
        Self::new(&NameSpace::local(), name)
    }

    pub fn is_local(&self) -> bool {
        return self.namespace.is_local();
    }

    pub fn to_string(&self) -> String {
        let ns = self.namespace.to_string();
        if ns.is_empty() {
            self.name.clone()
        } else {
            ns + NAMESPACE_SEPARATOR + &self.name
        }
    }

    pub fn is_suffix(&self, other: &NameSpacedName) -> bool {
        self.name == other.name && self.namespace.is_suffix(&other.namespace)
    }

    pub fn to_namespace(&self) -> NameSpace {
        let mut names = self.namespace.names.clone();
        names.push(self.name.clone());
        NameSpace { names }
    }

    pub fn name_as_mut(&mut self) -> &mut Name {
        &mut self.name
    }
}

impl Var {
    pub fn set_namsapce(&self, ns: NameSpace) -> Arc<Self> {
        let mut ret = self.clone();
        ret.name.namespace = ns;
        Arc::new(ret)
    }

    pub fn set_name(&self, nsn: NameSpacedName) -> Arc<Self> {
        let mut ret = self.clone();
        ret.name = nsn;
        Arc::new(ret)
    }
}

pub fn var_var(name: NameSpacedName, src: Option<Span>) -> Arc<Var> {
    Arc::new(Var { name, source: src })
}

pub fn var_local(var_name: &str, src: Option<Span>) -> Arc<Var> {
    var_var(NameSpacedName::local(var_name), src)
}

pub fn expr_lit(
    generator: Arc<LiteralGenerator>,
    free_vars: Vec<NameSpacedName>,
    name: String,
    ty: Arc<TypeNode>,
    src: Option<Span>,
) -> Arc<ExprNode> {
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
    bound: Arc<ExprNode>,
    expr: Arc<ExprNode>,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::Let(var, bound, expr)).into_expr_info(src)
}

pub fn expr_abs(var: Arc<Var>, val: Arc<ExprNode>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::Lam(var, val)).into_expr_info(src)
}

pub fn expr_app(lam: Arc<ExprNode>, arg: Arc<ExprNode>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::App(lam, arg)).into_expr_info(src)
}

// Make variable expression.
pub fn expr_var(name: NameSpacedName, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::Var(var_var(name, src.clone()))).into_expr_info(src)
}

pub fn expr_if(
    cond: Arc<ExprNode>,
    then_expr: Arc<ExprNode>,
    else_expr: Arc<ExprNode>,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::If(cond, then_expr, else_expr)).into_expr_info(src)
}

pub fn expr_tyanno(expr: Arc<ExprNode>, ty: Arc<TypeNode>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::TyAnno(expr, ty)).into_expr_info(src)
}

pub fn expr_make_pair(lhs: Arc<ExprNode>, rhs: Arc<ExprNode>) -> Arc<ExprNode> {
    Arc::new(Expr::MakePair(lhs, rhs)).into_expr_info(None)
}

// TODO: use persistent binary search tree as ExprAuxInfo to avoid O(n^2) complexity of calculate_free_vars.
pub fn calculate_free_vars(ei: Arc<ExprNode>) -> Arc<ExprNode> {
    match &*ei.expr {
        Expr::Var(var) => {
            let free_vars = vec![var.name.clone()].into_iter().collect();
            ei.set_free_vars(free_vars)
        }
        Expr::Lit(lit) => {
            let free_vars = lit.free_vars.clone().into_iter().collect();
            ei.set_free_vars(free_vars)
        }
        Expr::App(func, arg) => {
            let func = calculate_free_vars(func.clone());
            let arg = calculate_free_vars(arg.clone());
            let mut free_vars = func.free_vars.clone().unwrap();
            free_vars.extend(arg.free_vars.clone().unwrap());
            ei.set_app_func(func)
                .set_app_arg(arg)
                .set_free_vars(free_vars)
        }
        Expr::Lam(arg, body) => {
            let body = calculate_free_vars(body.clone());
            let mut free_vars = body.free_vars.clone().unwrap();
            free_vars.remove(&arg.name);
            free_vars.remove(&NameSpacedName::local(SELF_NAME));
            ei.set_lam_body(body).set_free_vars(free_vars)
        }
        Expr::Let(var, bound, val) => {
            // NOTE: Our Let is non-recursive let, i.e.,
            // "let x = f x in g x" is equal to "let y = f x in g y",
            // and x ∈ FreeVars("let x = f x in g x") = (FreeVars(g x) - {x}) + FreeVars(f x) != (FreeVars(g x) + FreeVars(f x)) - {x}.
            let bound = calculate_free_vars(bound.clone());
            let val = calculate_free_vars(val.clone());
            let mut free_vars = val.free_vars.clone().unwrap();
            free_vars.remove(&var.name);
            free_vars.extend(bound.free_vars.clone().unwrap());
            ei.set_let_bound(bound)
                .set_let_value(val)
                .set_free_vars(free_vars)
        }
        Expr::If(cond, then_expr, else_expr) => {
            let cond = calculate_free_vars(cond.clone());
            let then_expr = calculate_free_vars(then_expr.clone());
            let else_expr = calculate_free_vars(else_expr.clone());
            let mut free_vars = cond.free_vars.clone().unwrap();
            free_vars.extend(then_expr.free_vars.clone().unwrap());
            free_vars.extend(else_expr.free_vars.clone().unwrap());
            ei.set_if_cond(cond)
                .set_if_then(then_expr)
                .set_if_else(else_expr)
                .set_free_vars(free_vars)
        }
        Expr::TyAnno(e, _) => {
            let e = calculate_free_vars(e.clone());
            let free_vars = e.free_vars.clone().unwrap();
            ei.set_tyanno_expr(e).set_free_vars(free_vars)
        }
        Expr::MakePair(lhs, rhs) => {
            let lhs = calculate_free_vars(lhs.clone());
            let rhs = calculate_free_vars(rhs.clone());
            let mut free_vars = lhs.free_vars.clone().unwrap();
            free_vars.extend(rhs.free_vars.clone().unwrap());
            ei.set_make_pair_lhs(lhs)
                .set_make_pair_rhs(rhs)
                .set_free_vars(free_vars)
        }
    }
}
