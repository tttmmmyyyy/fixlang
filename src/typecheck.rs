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
struct Scope<T> {
    var: HashMap<String, ScopeValue<T>>,
}

#[derive(Clone)]
struct ScopeValue<T> {
    global: HashMap<NameSpace, T>,
    local: Vec<T>,
}

impl<T> Default for ScopeValue<T> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            local: Default::default(),
        }
    }
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        Self {
            var: Default::default(),
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
        self.var.get_mut(name).unwrap().local.push(ty.clone());
    }
    fn pop(self: &mut Self, name: &str) {
        self.var.get_mut(name).unwrap().local.pop();
        if self.var.get(name).unwrap().local.is_empty() {
            self.var.remove(name);
        }
    }
    fn get(self: &Self, name: &str) -> Option<&ScopeValue<T>> {
        self.var.get(name)
    }
    fn get_mut(self: &mut Self, name: &str) -> Option<&mut ScopeValue<T>> {
        self.var.get_mut(name)
    }
    fn add_global(&mut self, name: &str, namespace: &NameSpace, value: &T) {
        if !self.var.contains_key(name) {
            self.var.insert(String::from(name), Default::default());
        }
        if self.var[name].global.contains_key(namespace) {
            error_exit(&format!(
                "duplicate definition for `{}::{}`",
                namespace.to_string(),
                name
            ))
        }
        self.get_mut(name)
            .unwrap()
            .global
            .insert(namespace.clone(), value.clone());
    }

    // Get candidates list for overload resolution.
    // If `namespace` is unspecified (None) and a local variable `name` is found, then that local variable is returned.
    // If `namespace` is unspecified and no local variable `name` is found, then all global variables are returned.
    // If `namespace` is specified and non-empty, then returns all global variables whose namespaces have `namespace` as suffix.
    // If `namespace` is specified and empty, then returns local variable `name`.
    fn overloaded_candidates(
        &self,
        name: &str,
        namespace: &Option<NameSpace>,
    ) -> Vec<(NameSpace, T)> {
        if !self.var.contains_key(name) {
            return vec![];
        }
        let sv = &self.var[name];
        match namespace {
            None => {
                if sv.local.len() > 0 {
                    vec![(NameSpace::local(), sv.local.last().unwrap().clone())]
                } else {
                    sv.global
                        .iter()
                        .map(|(ns, v)| (ns.clone(), v.clone()))
                        .collect()
                }
            }
            Some(ns) => {
                if ns.is_local() {
                    if sv.local.len() > 0 {
                        vec![(NameSpace::local(), sv.local.last().unwrap().clone())]
                    } else {
                        vec![]
                    }
                } else {
                    sv.global
                        .iter()
                        .filter(|(ns2, v)| ns.is_suffix(ns2))
                        .map(|(ns2, v)| (ns2.clone(), v.clone()))
                        .collect()
                }
            }
        }
    }
}

// Type substitution. Name of type variable -> type.
// Managed so that the value (a type) of this HashMap doesn't contain a type variable that appears in keys. i.e.,
// when we want to COMPLETELY substitute type variables in a type by `substitution`, we only apply this mapy only ONCE.
#[derive(Clone)]
struct Substitution {
    data: HashMap<String, Arc<TypeNode>>,
}

impl Default for Substitution {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl Substitution {
    // Make single substitution.
    fn single(var: &str, ty: Arc<TypeNode>) -> Self {
        let mut data = HashMap::<String, Arc<TypeNode>>::default();
        data.insert(var.to_string(), ty);
        Self { data }
    }

    // Add substitution.
    fn add_substitution(&mut self, other: &Self) {
        for (_var, ty) in self.data.iter_mut() {
            let new_ty = other.substitute_type(&ty);
            *ty = new_ty;
        }
        for (var, ty) in &other.data {
            self.data.insert(var.to_string(), ty.clone());
        }
    }

    // Apply substitution.
    fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        match &ty.ty {
            Type::TyVar(tyvar) => self
                .data
                .get(&tyvar.name)
                .map_or(ty.clone(), |sub| sub.clone()),
            Type::TyCon(tc) => ty.clone(),
            Type::TyApp(fun, arg) => {
                let fun = self.substitute_type(fun);
                let arg = self.substitute_type(arg);
                type_tyapp(fun, arg)
            }
            Type::FunTy(param, body) => {
                type_fun(self.substitute_type(&param), self.substitute_type(&body))
            }
        }
    }

    // Calculate minimum substitution to unify two types.
    fn unify(
        tycons: &HashMap<String, Arc<Kind>>,
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
    ) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(var1) => {
                return Self::unify_tyvar(tycons, &var1, ty2);
            }
            _ => {}
        }
        match &ty2.ty {
            Type::TyVar(var2) => {
                return Self::unify_tyvar(tycons, &var2, ty1);
            }
            _ => {}
        }
        match &ty1.ty {
            Type::TyVar(_) => unreachable!(),
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1.name == tc2.name {
                        return Some(Self::default());
                    } else {
                        return None;
                    }
                }
                _ => {
                    return None;
                }
            },
            Type::TyApp(fun1, arg1) => match &ty2.ty {
                Type::TyApp(fun2, arg2) => {
                    let mut ret = Self::default();
                    match Self::unify(tycons, &fun1, &fun2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let arg1 = ret.substitute_type(arg1);
                    let arg2 = ret.substitute_type(arg2);
                    match Self::unify(tycons, &arg1, &arg2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    return Some(ret);
                }
                _ => {
                    return None;
                }
            },
            Type::FunTy(arg_ty1, ret_ty1) => match &ty2.ty {
                Type::FunTy(arg_ty2, ret_ty2) => {
                    let mut ret = Self::default();
                    match Self::unify(tycons, &arg_ty1, &arg_ty2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    let ret_ty1 = ret.substitute_type(ret_ty1);
                    let ret_ty2 = ret.substitute_type(ret_ty2);
                    match Self::unify(tycons, &ret_ty1, &ret_ty2) {
                        Some(sub) => ret.add_substitution(&sub),
                        None => return None,
                    };
                    return Some(ret);
                }
                _ => {
                    return None;
                }
            },
        }
    }

    // Subroutine of unify().
    fn unify_tyvar(
        tycons: &HashMap<String, Arc<Kind>>,
        tyvar1: &Arc<TyVar>,
        ty2: &Arc<TypeNode>,
    ) -> Option<Self> {
        match &ty2.ty {
            Type::TyVar(tyvar2) => {
                if tyvar1.name == tyvar2.name {
                    // Avoid adding circular subsitution.
                    return Some(Self::default());
                }
            }
            _ => {}
        };
        if ty2.free_vars().contains_key(&tyvar1.name) {
            panic!("unify_tyvar is making circular substitution.")
        }
        if tyvar1.kind != ty2.kind(tycons) {
            error_exit_with_src("Kinds do not match.", &None);
        }
        Some(Self::single(&tyvar1.name, ty2.clone()))
    }
}

// Context under type-checking.
// Reference: https://uhideyuki.sakura.ne.jp/studs/index.cgi/ja/HindleyMilnerInHaskell#fn6
#[derive(Clone)]
pub struct TypeCheckContext {
    // The identifier of type variables.
    tyvar_id: u32,
    // Scoped map of variable name -> scheme. (Assamptions of type inference.)
    scope: Scope<Arc<Scheme>>,
    // Substitution.
    substitution: Substitution,
    // Set of TyCons associated with kinds
    tycons: HashMap<String, Arc<Kind>>,
}

impl Default for TypeCheckContext {
    fn default() -> Self {
        Self {
            tyvar_id: Default::default(),
            scope: Default::default(),
            substitution: Default::default(),
            tycons: bulitin_type_to_kind_map(),
        }
    }
}

impl TypeCheckContext {
    // Generate new type variable.
    fn new_tyvar(&mut self) -> String {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
        "a".to_string() + &id.to_string()
    }

    // Apply substitution to type.
    fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        self.substitution.substitute_type(ty)
    }

    // Apply substitution to scheme.
    fn substitute_scheme(&self, scm: &Arc<Scheme>) -> Arc<Scheme> {
        Scheme::new_arc(scm.vars.clone(), self.substitute_type(&scm.ty))
    }

    // Instantiate a scheme.
    fn instantiate_scheme(&mut self, scheme: &Arc<Scheme>) -> Arc<TypeNode> {
        let mut sub = Substitution::default();
        for (var, kind) in &scheme.vars {
            let new_var_name = self.new_tyvar();
            sub.add_substitution(&Substitution::single(&var, type_tyvar(&new_var_name, kind)));
        }
        sub.substitute_type(&scheme.ty)
    }

    // Make a scheme from a type by abstracting type variable that does not appear in scope.
    fn abstract_to_scheme(&self, ty: &Arc<TypeNode>) -> Arc<Scheme> {
        let ty = self.substitute_type(ty);
        let mut vars = ty.free_vars();
        for (_var, scp) in &self.scope.var {
            for scm in &scp.local {
                for (var_in_scope, _) in self.substitute_scheme(&scm).free_vars() {
                    vars.remove(&var_in_scope);
                }
            }
        }
        Scheme::new_arc(vars, ty)
    }

    // Update substitution to unify two types.
    fn unify(&mut self, ty1: &Arc<TypeNode>, ty2: &Arc<TypeNode>) -> bool {
        let ty1 = &self.substitute_type(ty1);
        let ty2 = &self.substitute_type(ty2);
        match Substitution::unify(&self.tycons, ty1, ty2) {
            Some(sub) => {
                self.substitution.add_substitution(&sub);
                return true;
            }
            None => {
                return false;
            }
        }
    }

    // Perform typechecking.
    // Update type substitution so that `ei` has type `ty`.
    // Returns given AST augmented with inferred information.
    pub fn deduce_expr(&mut self, ei: &Arc<ExprNode>, ty: Arc<TypeNode>) -> Arc<ExprNode> {
        match &*ei.expr {
            Expr::Var(var) => {
                let candidates = self.scope.overloaded_candidates(&var.name, &var.namespace);
                let candidates: Vec<(TypeCheckContext, NameSpace)> = candidates
                    .iter()
                    .filter_map(|(ns, scm)| {
                        let mut tc = self.clone();
                        let var_ty = tc.instantiate_scheme(&scm);
                        let var_ty = tc.substitute_type(&var_ty);
                        if tc.unify(&var_ty, &ty) {
                            Some((tc, ns.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                if candidates.is_empty() {
                    error_exit_with_src(
                        &format!(
                            "name `{}` of required type `{}` is not found.",
                            var.name,
                            &self.substitute_type(&ty).to_string()
                        ),
                        &var.source,
                    );
                } else if candidates.len() >= 2 {
                    let candidates_str = candidates
                        .iter()
                        .map(|(_, ns)| {
                            "`".to_string() + &NameSpacedName::new(ns, &var.name).to_string() + "`"
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    error_exit_with_src(
                        &format!(
                            "name `{}` is ambiguous: there are {}.",
                            var.name, candidates_str
                        ),
                        &var.source,
                    );
                } else {
                    // candidates.len() == 1
                    let (tc, ns) = candidates[0].clone();
                    *self = tc;
                    ei.set_var_namespace(&ns)
                }
            }
            Expr::Lit(lit) => {
                if !self.unify(&lit.ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "type mismatch. Expected `{}`, Found `{}`",
                            &self.substitute_type(&ty).to_string(),
                            &lit.ty.to_string(),
                        ),
                        &ei.source,
                    );
                }
                ei.clone()
            }
            Expr::App(fun, arg) => {
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                if ei.app_order == AppSourceCodeOrderType::ArgumentIsFormer {
                    let arg = self.deduce_expr(arg, arg_ty.clone());
                    let fun = self.deduce_expr(fun, type_fun(arg_ty.clone(), ty));
                    ei.set_app_arg(arg).set_app_func(fun)
                } else {
                    let fun = self.deduce_expr(fun, type_fun(arg_ty.clone(), ty));
                    let arg = self.deduce_expr(arg, arg_ty.clone());
                    ei.set_app_arg(arg).set_app_func(fun)
                }
            }
            Expr::Lam(arg, body) => {
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                let body_ty = type_tyvar_star(&self.new_tyvar());
                let fun_ty = type_fun(arg_ty.clone(), body_ty.clone());
                if !self.unify(&fun_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "type mismatch. Expected `{}`, Found `{}`",
                            &self.substitute_type(&ty).to_string(),
                            &self.substitute_type(&fun_ty).to_string(),
                        ),
                        &ei.source,
                    );
                }
                self.scope
                    .push(&arg.name, &Scheme::new_arc(Default::default(), arg_ty));
                let body = self.deduce_expr(body, body_ty);
                self.scope.pop(&arg.name);
                ei.set_lam_body(body)
            }
            Expr::Let(var, val, body) => {
                let var_ty = match &var.type_annotation {
                    Some(scm) => {
                        let free_vars = scm.free_vars();
                        if !free_vars.is_empty() {
                            error_exit_with_src(
                                &format!(
                                    "unknown type variable `{}`",
                                    free_vars.iter().next().unwrap().0
                                ),
                                &var.source,
                            )
                        }
                        self.instantiate_scheme(&scm)
                    }
                    None => type_tyvar_star(&self.new_tyvar()),
                };
                let val = self.deduce_expr(val, var_ty.clone());
                let var_scm = self.abstract_to_scheme(&var_ty);

                let body = if var.namespace.as_ref().unwrap().is_local() {
                    self.scope.push(&var.name, &var_scm);
                    let body = self.deduce_expr(body, ty);
                    self.scope.pop(&var.name);
                    body
                } else {
                    // NOTE: currently, top-level definition is treated as let-binding.
                    self.scope
                        .add_global(&var.name, &var.namespace.as_ref().unwrap(), &var_scm);
                    self.deduce_expr(body, ty)
                };
                ei.set_let_bound(val).set_let_value(body)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.deduce_expr(cond, bool_lit_ty());
                let then_expr = self.deduce_expr(then_expr, ty.clone());
                let else_expr = self.deduce_expr(else_expr, ty);
                ei.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, anno_ty) => {
                if anno_ty.free_vars().len() > 0 {
                    error_exit(&format!(
                        "unknown type variable `{}`",
                        ty.free_vars().iter().next().unwrap().0
                    ))
                }
                if !self.unify(&ty, anno_ty) {
                    error_exit_with_src(
                        &format!(
                            "type mismatch. Expected `{}`, Found `{}`",
                            &self.substitute_type(&ty).to_string(),
                            &self.substitute_type(&anno_ty).to_string(),
                        ),
                        &ei.source,
                    );
                }
                let e = self.deduce_expr(e, ty.clone());
                ei.set_tyanno_expr(e)
            }
        }
    }

    // Read type declarations to verity it and extend type-to-kind mapping.
    pub fn add_tycons(&mut self, type_decls: &Vec<TypeDecl>) {
        for type_decl in type_decls {
            self.add_tycon(type_decl);
        }
    }

    // Read type declaration to verity it and extend type-to-kind mapping.
    fn add_tycon(&mut self, decl: &TypeDecl) {
        if self.tycons.contains_key(&decl.name) {
            error_exit_with_src(&format!("Type `{}` is already defined.", decl.name), &None);
        }
        self.tycons.insert(decl.name.clone(), kind_star());
    }
}
