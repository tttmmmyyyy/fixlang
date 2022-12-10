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
pub struct Scope<T> {
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
    pub fn add_global(&mut self, name: Name, namespace: &NameSpace, value: &T) {
        if !self.var.contains_key(&name) {
            self.var.insert(name.clone(), Default::default());
        }
        if self.var[&name].global.contains_key(namespace) {
            error_exit(&format!(
                "duplicate definition for `{}.{}`",
                namespace.to_string(),
                name
            ))
        }
        self.get_mut(&name)
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
                        .filter(|(ns2, _)| ns.is_suffix(ns2))
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
pub struct Substitution {
    pub data: HashMap<Name, Arc<TypeNode>>,
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
    pub fn single(var: &str, ty: Arc<TypeNode>) -> Self {
        let mut data = HashMap::<String, Arc<TypeNode>>::default();
        data.insert(var.to_string(), ty);
        Self { data }
    }

    // Add (=compose) substitution.
    pub fn add_substitution(&mut self, other: &Self) {
        for (_var, ty) in self.data.iter_mut() {
            let new_ty = other.substitute_type(&ty);
            *ty = new_ty;
        }
        for (var, ty) in &other.data {
            self.data.insert(var.to_string(), ty.clone());
        }
    }

    // Merge substitution.
    // Returns true when merge succeeds.
    fn merge_substitution(&mut self, other: &Self) -> bool {
        for (var, ty) in &other.data {
            if self.data.contains_key(var) {
                if self.data[var] != *ty {
                    return false;
                }
            } else {
                self.data.insert(var.to_string(), ty.clone());
            }
        }
        return true;
    }

    // Apply substitution to predicate.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        p.ty = self.substitute_type(&p.ty);
    }

    // Apply substitution to type
    pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        match &ty.ty {
            Type::TyVar(tyvar) => self
                .data
                .get(&tyvar.name)
                .map_or(ty.clone(), |sub| sub.clone()),
            Type::TyCon(_) => ty.clone(),
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
    pub fn unify(
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
                    if tc1 == tc2 {
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

    // Calculate minimum substitution s such that `s(ty1) = ty2`.
    pub fn matching(
        tycons: &HashMap<String, Arc<Kind>>,
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
    ) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(v1) => Self::unify_tyvar(tycons, v1, ty2),
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1 == tc2 {
                        Some(Self::default())
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Type::TyApp(fun1, arg1) => match &ty2.ty {
                Type::TyApp(fun2, arg2) => {
                    let mut ret = Self::default();
                    match Self::matching(tycons, fun1, fun2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(tycons, arg1, arg2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    Some(ret)
                }
                _ => None,
            },
            Type::FunTy(src1, dst1) => match &ty2.ty {
                Type::FunTy(src2, dst2) => {
                    let mut ret = Self::default();
                    match Self::matching(tycons, src1, src2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(tycons, dst1, dst2) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    Some(ret)
                }
                _ => None,
            },
        }
    }
}

// Context under type-checking.
// Reference: https://uhideyuki.sakura.ne.jp/studs/index.cgi/ja/HindleyMilnerInHaskell#fn6
#[derive(Clone)]
pub struct TypeCheckContext {
    // The identifier of type variables.
    tyvar_id: u32,
    // Scoped map of variable name -> scheme. (Assamptions of type inference.)
    pub scope: Scope<Arc<Scheme>>,
    // Substitution.
    substitution: Substitution,
    // Predicates
    pub predicates: Vec<Predicate>,
    // Set of TyCons associated with kinds
    tycons: HashMap<String, Arc<Kind>>,
    // Trait environment.
    trait_env: TraitEnv,
}

impl Default for TypeCheckContext {
    fn default() -> Self {
        Self {
            tyvar_id: Default::default(),
            scope: Default::default(),
            substitution: Default::default(),
            predicates: Default::default(),
            tycons: bulitin_type_to_kind_map(),
            trait_env: Default::default(),
        }
    }
}

impl TypeCheckContext {
    // Generate new type variable.
    fn new_tyvar(&mut self) -> String {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
        "%a".to_string() + &id.to_string() // To avlid confliction with user-defined type variable, we add prefix #.
    }

    // Apply substitution to type.
    pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        self.substitution.substitute_type(ty)
    }

    // Apply substitution to a predicate.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        self.substitution.substitute_predicate(p)
    }

    // Apply substitution to scheme.
    pub fn substitute_scheme(&self, scm: &Arc<Scheme>) -> Arc<Scheme> {
        scm.substitute(&self.substitution)
    }

    // Instantiate a scheme.
    // Returns predicates if append_predicates = false or append them to self if append_predicates = true.
    pub fn instantiate_scheme(
        &mut self,
        scheme: &Arc<Scheme>,
        append_predicates: bool,
    ) -> (Vec<Predicate>, Arc<TypeNode>) {
        let mut sub = Substitution::default();
        for (var, kind) in &scheme.vars {
            let new_var_name = self.new_tyvar();
            sub.add_substitution(&Substitution::single(&var, type_tyvar(&new_var_name, kind)));
        }
        let mut preds = scheme.preds.clone();
        for p in &mut preds {
            sub.substitute_predicate(p);
        }
        if append_predicates {
            self.predicates.append(&mut preds);
        }
        (preds, sub.substitute_type(&scheme.ty))
    }

    // Make a scheme from a type by generalizing type variable that does not appear in fixed_vars.
    fn generalize_to_scheme(
        &mut self,
        ty: &Arc<TypeNode>,
        fixed_vars: &HashSet<Name>,
    ) -> Arc<Scheme> {
        // Get generalized type and predicates.
        let ty = self.substitute_type(ty);
        let mut preds = std::mem::replace(&mut self.predicates, vec![]);

        // Reduce predicates.
        for p in &mut preds {
            self.substitute_predicate(p);
        }
        let preds = match self.trait_env.reduce(&preds, &self.tycons) {
            Some(ps) => ps,
            None => self.error_exit_on_predicates(),
        };

        // Collect variables that appear in scope.
        // let mut vars_in_scope: HashSet<String> = Default::default();
        // for (_var, scp) in &self.scope.var {
        //     for scm in &scp.local {
        //         for (var_in_scope, _) in self.substitute_scheme(&scm).free_vars() {
        //             vars_in_scope.insert(var_in_scope);
        //         }
        //     }
        // }

        // Calculate genealized variables.
        let mut gen_vars = ty.free_vars();
        for v in fixed_vars {
            gen_vars.remove(v);
        }

        // Split predicates to generalized and deferred.
        let mut gen_preds: Vec<Predicate> = Default::default(); // Generalized predicates.
        let mut def_preds: Vec<Predicate> = Default::default(); // Deferred predicates.
        for p in preds {
            if p.ty.free_vars().iter().all(|(v, _)| fixed_vars.contains(v)) {
                // All free variables of p appears in fixed_vars.
                def_preds.push(p);
            } else if p
                .ty
                .free_vars()
                .iter()
                .any(|(v, _)| !fixed_vars.contains(v) && gen_vars.contains_key(v))
            {
                // A free variable of p appears neither in fixed_vars and generalized variables.
                error_exit(&format!("ambiguous type variable in `{}`", p.to_string()))
            } else {
                // A free variable of p appears in generalized variables.
                gen_preds.push(p);
            }
        }

        self.predicates = def_preds;
        Scheme::generalize(gen_vars, gen_preds, ty)
    }

    // Show an error message that predicates are unsatisfiable.
    pub fn error_exit_on_predicates(&self) -> ! {
        error_exit(&format!(
            "predicates are unsatisfiable: {}",
            self.predicates
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    // Update substitution to unify two types.
    pub fn unify(&mut self, ty1: &Arc<TypeNode>, ty2: &Arc<TypeNode>) -> bool {
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

    // Reduce predicates.
    // If predicates are unsatisfiable, do nothing and return false.
    pub fn reduce_predicates(&mut self) -> bool {
        let mut preds = std::mem::replace(&mut self.predicates, vec![]);
        for p in &mut preds {
            self.substitute_predicate(p);
        }
        self.predicates.append(&mut preds);
        match self.trait_env.reduce(&self.predicates, &self.tycons) {
            Some(ps) => {
                self.predicates = ps;
                return true;
            }
            None => return false,
        }
    }

    // Perform typechecking.
    // Update type substitution so that `ei` has type `ty`.
    // Returns given AST augmented with inferred information.
    pub fn unify_type_of_expr(&mut self, ei: &Arc<ExprNode>, ty: Arc<TypeNode>) -> Arc<ExprNode> {
        let ei = ei.set_inferred_type(ty.clone());
        match &*ei.expr {
            Expr::Var(var) => {
                let candidates = self.scope.overloaded_candidates(&var.name, &var.namespace);
                let candidates: Vec<(TypeCheckContext, NameSpace)> = candidates
                    .iter()
                    .filter_map(|(ns, scm)| {
                        let mut tc = self.clone();
                        let (_, var_ty) = tc.instantiate_scheme(&scm, true);
                        // if var_ty is unifiable to the required type and predicates are satisfiable, then thie candidate is ok.
                        if tc.unify(&var_ty, &ty) {
                            if tc.reduce_predicates() {
                                Some((tc, ns.clone()))
                            } else {
                                None
                            }
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
                    let arg = self.unify_type_of_expr(arg, arg_ty.clone());
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty));
                    ei.set_app_arg(arg).set_app_func(fun)
                } else {
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty));
                    let arg = self.unify_type_of_expr(arg, arg_ty.clone());
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
                self.scope.push(&arg.name, &Scheme::from_type(arg_ty));
                let body = self.unify_type_of_expr(body, body_ty);
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
                        self.instantiate_scheme(&scm, true).1
                    }
                    None => type_tyvar_star(&self.new_tyvar()),
                };
                let val = self.unify_type_of_expr(val, var_ty.clone());
                let var_scm = self.generalize_to_scheme(&var_ty, &HashSet::default());

                self.scope.push(&var.name, &var_scm);
                let body = self.unify_type_of_expr(body, ty);
                self.scope.pop(&var.name);
                ei.set_let_bound(val).set_let_value(body)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.unify_type_of_expr(cond, bool_lit_ty());
                let then_expr = self.unify_type_of_expr(then_expr, ty.clone());
                let else_expr = self.unify_type_of_expr(else_expr, ty);
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
                // TODO: Maybe this is wrong. We should check if "deduced type is more general than specified type".
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
                let e = self.unify_type_of_expr(e, ty.clone());
                ei.set_tyanno_expr(e)
            }
        }
    }

    // Check if expr has type scm.
    // Returns given AST augmented with inferred information.
    pub fn check_type_nofree(
        &mut self,
        expr: Arc<ExprNode>,
        expect_scm: Arc<Scheme>,
    ) -> Arc<ExprNode> {
        assert!(self.predicates.is_empty()); // This function is available only when predicates are empty.
        let (expr, deduced_scm) = self.deduce_scheme_nofree(expr);
        let (given_preds, specified_ty) = self.instantiate_scheme(&expect_scm, false);
        let (required_preds, most_general_ty) = self.instantiate_scheme(&deduced_scm, false);
        let s = Substitution::matching(&self.tycons, &most_general_ty, &specified_ty);
        if s.is_none() {
            error_exit(&format!(
                "type mismatch. Expected `{}`, found `{}`",
                specified_ty.to_string(),
                most_general_ty.to_string()
            ));
        }
        let s = s.unwrap();
        for p in required_preds {
            let mut p = p.clone();
            s.substitute_predicate(&mut p);
            if !self.trait_env.entail(&given_preds, &p, &self.tycons) {
                error_exit(&format!(
                    "condition `{}` is necessary for this expression but not assumed in the specified type.",
                    p.to_string()
                ));
            }
        }

        expr
    }

    // Deduce scheme of a expression. It may leave free variables and predicates.
    // Returns given AST augmented with inferred information.
    fn deduce_scheme(&mut self, expr: Arc<ExprNode>) -> (Arc<ExprNode>, Arc<Scheme>) {
        let ty = type_tyvar_star(&self.new_tyvar());
        let expr = self.unify_type_of_expr(&expr, ty.clone());
        let scm = self.generalize_to_scheme(&ty, &ty.free_vars_set());
        (expr, scm)
    }

    // Deduce scheme of a expression with no free variable and deferred predicates.
    // Returns given AST augmented with inferred information.
    fn deduce_scheme_nofree(&mut self, expr: Arc<ExprNode>) -> (Arc<ExprNode>, Arc<Scheme>) {
        assert!(self.predicates.is_empty()); // This function is available only when predicates are empty.
        let (expr, scm) = self.deduce_scheme(expr);

        // If free variables are left, raise an error.
        let free_vars = scm.free_vars();
        if !free_vars.is_empty() {
            let free_vars: Vec<Name> = free_vars
                .iter()
                .map(|(k, _)| "`".to_string() + k + "`")
                .collect();
            error_exit(&format!("unknown type variables {}", free_vars.join(", ")));
        }

        // If predicates are unsatisfiable or deferred, raise an error.
        if !self.reduce_predicates() || !self.predicates.is_empty() {
            self.error_exit_on_predicates();
        }

        (expr, scm)
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
