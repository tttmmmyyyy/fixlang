use super::*;
use core::panic;
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
    pub free_vars: Option<HashSet<FullName>>,
    pub source: Option<Span>,
    pub app_order: AppSourceCodeOrderType,
    pub inferred_ty: Option<Arc<TypeNode>>,
}

impl ExprNode {
    // Set free vars
    fn set_free_vars(&self, free_vars: HashSet<FullName>) -> Arc<Self> {
        let mut ret = self.clone();
        ret.free_vars = Some(free_vars);
        Arc::new(ret)
    }

    // Get free vars
    pub fn free_vars(self: &Self) -> &HashSet<FullName> {
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

    pub fn set_app_args(&self, args: Vec<Arc<ExprNode>>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::App(func, _) => {
                ret.expr = Arc::new(Expr::App(func.clone(), args));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    // destructure lambda expression to list of variables and body expression
    pub fn destructure_lam(&self) -> (Vec<Arc<Var>>, Arc<ExprNode>) {
        match &*self.expr {
            Expr::Lam(args, body) => (args.clone(), body.clone()),
            _ => panic!(""),
        }
    }

    #[allow(dead_code)]
    pub fn set_lam_params(&self, params: Vec<Arc<Var>>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Lam(_, body) => {
                ret.expr = Arc::new(Expr::Lam(params, body.clone()));
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
    pub fn set_let_pat(&self, pat: Arc<Pattern>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Let(_, bound, val) => {
                ret.expr = Arc::new(Expr::Let(pat, bound.clone(), val.clone()));
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

    pub fn set_make_struct_tycon(&self, tc: Arc<TyCon>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakeStruct(_, fields) => {
                ret.expr = Arc::new(Expr::MakeStruct(tc, fields.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_make_struct_field(&self, field_name: &Name, field_expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakeStruct(tc, fields) => {
                let mut fields = fields.clone();
                for (name, expr) in &mut fields {
                    if name == field_name {
                        *expr = field_expr.clone();
                    }
                }
                ret.expr = Arc::new(Expr::MakeStruct(tc.clone(), fields));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_make_struct_fields(&self, fields: Vec<(Name, Arc<ExprNode>)>) -> Arc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakeStruct(tc, _) => {
                ret.expr = Arc::new(Expr::MakeStruct(tc.clone(), fields));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_array_lit_elem(&self, elem: Arc<ExprNode>, idx: usize) -> Arc<ExprNode> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::ArrayLit(elems) => {
                let mut elems = elems.clone();
                elems[idx] = elem;
                ret.expr = Arc::new(Expr::ArrayLit(elems));
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
            Expr::App(fun, args) => {
                let args = args.iter().map(|arg| arg.resolve_namespace(ctx)).collect();
                self.clone()
                    .set_app_func(fun.resolve_namespace(ctx))
                    .set_app_args(args)
            }
            Expr::Lam(_, body) => self.clone().set_lam_body(body.resolve_namespace(ctx)),
            Expr::Let(pat, bound, value) => self
                .clone()
                .set_let_pat(pat.resolve_namespace(ctx))
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
            Expr::MakeStruct(tc, fields) => {
                let mut expr = self.clone();
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx);
                expr = expr.set_make_struct_tycon(Arc::new(tc));
                for (field_name, field_expr) in fields {
                    let field_expr = field_expr.resolve_namespace(ctx);
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let expr = self.clone();
                for (i, elem) in elems.iter().enumerate() {
                    expr.set_array_lit_elem(elem.resolve_namespace(ctx), i);
                }
                expr
            }
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Var(Arc<Var>),
    Lit(Arc<Literal>),
    // application of multiple arguments is generated by optimization.
    App(Arc<ExprNode>, Vec<Arc<ExprNode>>),
    // lambda of multiple arguments is generated by optimization.
    Lam(Vec<Arc<Var>>, Arc<ExprNode>),
    Let(Arc<Pattern>, Arc<ExprNode>, Arc<ExprNode>),
    If(Arc<ExprNode>, Arc<ExprNode>, Arc<ExprNode>),
    TyAnno(Arc<ExprNode>, Arc<TypeNode>),
    ArrayLit(Vec<Arc<ExprNode>>),
    // Expresison `(x, y)` is not parsed to `Tuple2.new x y`, but to `MakeStruct x y`.
    // `MakeStruct x y` is compiled to a more performant code than function call (currently).
    MakeStruct(Arc<TyCon>, Vec<(Name, Arc<ExprNode>)>),
}

#[derive(Clone)]
pub enum Pattern {
    Var(Arc<Var>, Option<Arc<TypeNode>>),
    Struct(Arc<TyCon>, Vec<(Name, Arc<Pattern>)>),
    Union(Arc<TyCon>, Name, Arc<Pattern>),
}

impl Pattern {
    // Make basic variable pattern.
    #[allow(dead_code)]
    pub fn var_pattern(var: Arc<Var>) -> Arc<Pattern> {
        Arc::new(Pattern::Var(var, None))
    }

    // Check if variables defined in this pattern is duplicated or not.
    // For example, pattern (x, y) is ok, but (x, x) is invalid.
    pub fn validate_duplicated_vars(&self) -> bool {
        (self.vars().len() as u32) < self.count_vars()
    }

    // Count if variables defined in this pattern.
    fn count_vars(&self) -> u32 {
        match self {
            Pattern::Var(_, _) => 1,
            Pattern::Struct(_, field_to_pat) => {
                let mut ret = 0;
                for (_, pat) in field_to_pat {
                    ret += pat.count_vars();
                }
                ret
            }
            Pattern::Union(_, _, pat) => pat.count_vars(),
        }
    }

    // Returns the type of whole pattern and each variable.
    pub fn get_type(
        &self,
        typechcker: &mut TypeCheckContext,
    ) -> (Arc<TypeNode>, HashMap<FullName, Arc<TypeNode>>) {
        match self {
            Pattern::Var(v, ty) => {
                let var_name = v.name.clone();
                let ty = if ty.is_none() {
                    type_tyvar_star(&typechcker.new_tyvar())
                } else {
                    ty.as_ref().unwrap().clone()
                };
                let mut var_to_ty = HashMap::default();
                var_to_ty.insert(var_name, ty.clone());
                (ty, var_to_ty)
            }
            Pattern::Struct(tc, field_to_pat) => {
                let ty = tc.get_struct_union_value_type(typechcker);
                let mut var_to_ty = HashMap::default();
                let field_tys = ty.field_types(&typechcker.type_env);
                let fields = &typechcker.type_env.tycons.get(tc).unwrap().fields;
                assert_eq!(fields.len(), field_tys.len());
                let field_name_to_ty = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let ty = field_tys[i].clone();
                        (field.name.clone(), ty)
                    })
                    .collect::<HashMap<_, _>>();
                for (field_name, pat) in field_to_pat {
                    let (pat_ty, var_ty) = pat.get_type(typechcker);
                    var_to_ty.extend(var_ty);
                    let ok = typechcker.unify(&pat_ty, field_name_to_ty.get(field_name).unwrap());
                    if !ok {
                        error_exit(&format!(
                            "inappropriate pattern `{}` for field `{}` of struct `{}`",
                            pat.to_string(),
                            field_name,
                            tc.to_string(),
                        ));
                    }
                }
                (ty, var_to_ty)
            }
            Pattern::Union(tc, field_name, pat) => {
                let ty = tc.get_struct_union_value_type(typechcker);
                let mut var_to_ty = HashMap::default();
                let fields = &typechcker.type_env.tycons.get(tc).unwrap().fields;
                let field_tys = ty.field_types(&typechcker.type_env);
                assert_eq!(fields.len(), field_tys.len());
                let field_idx = fields
                    .iter()
                    .enumerate()
                    .find_map(|(i, f)| if &f.name == field_name { Some(i) } else { None })
                    .unwrap();
                let field_ty = field_tys[field_idx].clone();
                let (pat_ty, var_ty) = pat.get_type(typechcker);
                var_to_ty.extend(var_ty);
                let ok = typechcker.unify(&pat_ty, &field_ty);
                if !ok {
                    error_exit(&format!(
                        "inappropriate pattern `{}` for field `{}` of union `{}`",
                        pat.to_string(),
                        field_name,
                        tc.to_string(),
                    ));
                }
                (ty, var_to_ty)
            }
        }
    }

    // Calculate the set of variables that appears in this pattern.
    pub fn vars(&self) -> HashSet<FullName> {
        match self {
            Pattern::Var(var, _) => HashSet::from([var.name.clone()]),
            Pattern::Struct(_, pats) => {
                let mut ret = HashSet::default();
                for (_, pat) in pats {
                    ret.extend(pat.vars());
                }
                ret
            }
            Pattern::Union(_, _, pat) => pat.vars(),
        }
    }

    pub fn set_var_tyanno(self: &Arc<Pattern>, tyanno: Option<Arc<TypeNode>>) -> Arc<Pattern> {
        match &**self {
            Pattern::Var(v, _) => Arc::new(Pattern::Var(v.clone(), tyanno)),
            _ => panic!(),
        }
    }

    pub fn set_struct_tycon(self: &Arc<Pattern>, tc: Arc<TyCon>) -> Arc<Pattern> {
        match &**self {
            Pattern::Struct(_, field_to_pat) => Arc::new(Pattern::Struct(tc, field_to_pat.clone())),
            _ => panic!(),
        }
    }

    pub fn set_struct_field_to_pat(
        self: &Arc<Pattern>,
        field_to_pat: Vec<(Name, Arc<Pattern>)>,
    ) -> Arc<Pattern> {
        match &**self {
            Pattern::Struct(tc, _) => Arc::new(Pattern::Struct(tc.clone(), field_to_pat)),
            _ => panic!(),
        }
    }

    pub fn set_union_tycon(self: &Arc<Pattern>, tc: Arc<TyCon>) -> Arc<Pattern> {
        match &**self {
            Pattern::Union(_, field_name, pat) => {
                Arc::new(Pattern::Union(tc, field_name.clone(), pat.clone()))
            }
            _ => panic!(),
        }
    }

    pub fn set_union_pat(self: &Arc<Pattern>, pat: Arc<Pattern>) -> Arc<Pattern> {
        match &**self {
            Pattern::Union(tc, field_name, _) => {
                Arc::new(Pattern::Union(tc.clone(), field_name.clone(), pat))
            }
            _ => panic!(),
        }
    }

    pub fn resolve_namespace(self: &Arc<Pattern>, ctx: &NameResolutionContext) -> Arc<Pattern> {
        match &**self {
            Pattern::Var(_, ty) => {
                self.set_var_tyanno(ty.as_ref().map(|ty| ty.resolve_namespace(ctx)))
            }
            Pattern::Struct(tc, field_to_pat) => {
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx);
                let field_to_pat = field_to_pat
                    .iter()
                    .map(|(field_name, pat)| (field_name.clone(), pat.resolve_namespace(ctx)))
                    .collect::<Vec<_>>();
                self.set_struct_tycon(Arc::new(tc))
                    .set_struct_field_to_pat(field_to_pat)
            }
            Pattern::Union(tc, _, pat) => {
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx);
                self.set_union_tycon(Arc::new(tc))
                    .set_union_pat(pat.resolve_namespace(ctx))
            }
        }
    }

    pub fn to_string(&self) -> String {
        let mut ret = "".to_string();
        match self {
            Pattern::Var(v, t) => {
                ret += &v.name.to_string();
                match t {
                    Some(t) => {
                        ret += ": ";
                        ret += &t.to_string();
                    }
                    None => {}
                }
                ret
            }
            Pattern::Struct(tc, fields) => {
                if tc.name.namespace == NameSpace::new_str(&[STD_NAME])
                    && tc.name.name.starts_with(TUPLE_NAME)
                {
                    let pats = fields
                        .iter()
                        .map(|(_, pat)| pat.to_string())
                        .collect::<Vec<_>>();
                    format!("({})", pats.join(", "))
                } else {
                    let pats = fields
                        .iter()
                        .map(|(name, pat)| format!("{}: {}", name, pat.to_string()))
                        .collect::<Vec<_>>();
                    format!("{} {{{}}}", tc.to_string(), pats.join(", "))
                }
            }
            Pattern::Union(tc, field, pat) => {
                format!("{}.{}({})", tc.to_string(), field, pat.to_string())
            }
        }
    }
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
            Expr::App(_, _) => {
                let (fun, args) = collect_app(&Arc::new(self.clone()).into_expr_info(None));
                let mut omit_brace_around_fun = false;
                match *(fun.expr) {
                    Expr::Var(_) => omit_brace_around_fun = true,
                    Expr::Lit(_) => omit_brace_around_fun = true,
                    Expr::App(_, _) => omit_brace_around_fun = true,
                    _ => {}
                }
                let fun_str = fun.expr.to_string();

                let args_str = args
                    .iter()
                    .map(|arg| arg.expr.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                if omit_brace_around_fun {
                    format!("{}({})", fun_str, args_str)
                } else {
                    format!("({})({})", fun_str, args_str)
                }
            }
            Expr::Lam(xs, fx) => {
                format!(
                    "|{}| {}",
                    xs.iter()
                        .map(|x| x.name.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    fx.expr.to_string()
                )
            }
            Expr::Let(p, b, v) => format!(
                "let {}={} in {}",
                p.to_string(),
                b.expr.to_string(),
                v.expr.to_string()
            ),
            Expr::If(c, t, e) => format!(
                "if {} then {} else {}",
                c.expr.to_string(),
                t.expr.to_string(),
                e.expr.to_string()
            ),
            Expr::TyAnno(e, t) => format!("{}: {}", e.expr.to_string(), t.to_string()),
            Expr::MakeStruct(tc, fields) => {
                format!(
                    "{} {{{}}}",
                    tc.to_string(),
                    fields
                        .iter()
                        .map(|(name, expr)| format!("{}: {}", name, expr.expr.to_string()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::ArrayLit(elems) => {
                format!(
                    "[{}]",
                    elems
                        .iter()
                        .map(|e| e.expr.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

pub type InlineLLVM = dyn Send
    + Sync
    + for<'c, 'm, 'b> Fn(
        &mut GenerationContext<'c, 'm>,
        &Arc<TypeNode>,     // type of this literal
        Option<Object<'c>>, // rvo
    ) -> Object<'c>;

#[derive(Clone)]
pub struct Literal {
    pub generator: Arc<InlineLLVM>,
    pub free_vars: Vec<FullName>, // e.g. "+" literal has two free variables.
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

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn module(&self) -> Name {
        self.names[0].clone()
    }

    pub fn append(&self, mut rhs: NameSpace) -> NameSpace {
        let mut names = self.names.clone();
        names.append(&mut rhs.names);
        NameSpace::new(names)
    }
}

#[derive(Clone)]
pub struct Var {
    pub name: FullName,
    pub source: Option<Span>,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct FullName {
    pub namespace: NameSpace,
    pub name: String,
}

impl FullName {
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

    pub fn is_suffix(&self, other: &FullName) -> bool {
        self.name == other.name && self.namespace.is_suffix(&other.namespace)
    }

    pub fn to_namespace(&self) -> NameSpace {
        let mut names = self.namespace.names.clone();
        names.push(self.name.clone());
        NameSpace { names }
    }

    #[allow(dead_code)]
    pub fn module(&self) -> Name {
        self.namespace.module()
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

    pub fn set_name(&self, nsn: FullName) -> Arc<Self> {
        let mut ret = self.clone();
        ret.name = nsn;
        Arc::new(ret)
    }
}

pub fn var_var(name: FullName, src: Option<Span>) -> Arc<Var> {
    Arc::new(Var { name, source: src })
}

pub fn var_local(var_name: &str, src: Option<Span>) -> Arc<Var> {
    var_var(FullName::local(var_name), src)
}

pub fn expr_lit(
    generator: Arc<InlineLLVM>,
    free_vars: Vec<FullName>,
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
    pat: Arc<Pattern>,
    bound: Arc<ExprNode>,
    expr: Arc<ExprNode>,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::Let(pat, bound, expr)).into_expr_info(src)
}

pub fn expr_abs(vars: Vec<Arc<Var>>, val: Arc<ExprNode>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::Lam(vars, val)).into_expr_info(src)
}

pub fn expr_app(lam: Arc<ExprNode>, args: Vec<Arc<ExprNode>>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::App(lam, args)).into_expr_info(src)
}

// Make variable expression.
pub fn expr_var(name: FullName, src: Option<Span>) -> Arc<ExprNode> {
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

pub fn expr_make_struct(tc: Arc<TyCon>, fields: Vec<(Name, Arc<ExprNode>)>) -> Arc<ExprNode> {
    Arc::new(Expr::MakeStruct(tc, fields)).into_expr_info(None)
}

pub fn expr_array_lit(elems: Vec<Arc<ExprNode>>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::ArrayLit(elems)).into_expr_info(src)
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
        Expr::App(func, args) => {
            let func = calculate_free_vars(func.clone());
            let args = args
                .iter()
                .map(|arg| calculate_free_vars(arg.clone()))
                .collect::<Vec<_>>();
            let mut free_vars = func.free_vars.clone().unwrap();
            for arg in &args {
                free_vars.extend(arg.free_vars.clone().unwrap());
            }
            ei.set_app_func(func)
                .set_app_args(args)
                .set_free_vars(free_vars)
        }
        Expr::Lam(args, body) => {
            let body = calculate_free_vars(body.clone());
            let mut free_vars = body.free_vars.clone().unwrap();
            for arg in args {
                free_vars.remove(&arg.name);
            }
            free_vars.remove(&FullName::local(SELF_NAME));
            ei.set_lam_body(body).set_free_vars(free_vars)
        }
        Expr::Let(pat, bound, val) => {
            // NOTE: Our Let is non-recursive let, i.e.,
            // "let x = f x in g x" is equal to "let y = f x in g y",
            // and x ∈ FreeVars("let x = f x in g x") = (FreeVars(g x) - {x}) + FreeVars(f x) != (FreeVars(g x) + FreeVars(f x)) - {x}.
            let bound = calculate_free_vars(bound.clone());
            let val = calculate_free_vars(val.clone());
            let mut free_vars = val.free_vars.clone().unwrap();
            for v in pat.vars() {
                free_vars.remove(&v);
            }
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
        Expr::MakeStruct(_, fields) => {
            let mut free_vars: HashSet<FullName> = Default::default();
            let mut ei = ei.clone();
            for (field_name, field_expr) in fields {
                let field_expr = calculate_free_vars(field_expr.clone());
                free_vars.extend(field_expr.free_vars.clone().unwrap());
                ei = ei.set_make_struct_field(field_name, field_expr);
            }
            ei.set_free_vars(free_vars)
        }
        Expr::ArrayLit(elems) => {
            let mut free_vars: HashSet<FullName> = Default::default();
            let mut ei = ei.clone();
            for (i, e) in elems.iter().enumerate() {
                let e = calculate_free_vars(e.clone());
                ei = ei.set_array_lit_elem(e.clone(), i);
                free_vars.extend(e.free_vars.clone().unwrap());
            }
            ei
        }
    }
}

// Convert f(y, z) to (f, [y, z]).
pub fn collect_app(expr: &Arc<ExprNode>) -> (Arc<ExprNode>, Vec<Arc<ExprNode>>) {
    match &*expr.expr {
        Expr::App(fun, arg) => {
            let (fun, mut args) = collect_app(fun);
            args.append(&mut arg.clone());
            (fun, args)
        }
        _ => (expr.clone(), vec![]),
    }
}
