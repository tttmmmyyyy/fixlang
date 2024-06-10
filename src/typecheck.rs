use std::sync::Arc;

use serde::{Deserialize, Serialize};

use self::ast::import;

use super::*;

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
    pub fn push(self: &mut Self, name: &str, ty: &T) {
        if !self.var.contains_key(name) {
            self.var.insert(String::from(name), Default::default());
        }
        self.var.get_mut(name).unwrap().local.push(ty.clone());
    }
    pub fn pop(self: &mut Self, name: &str) {
        self.var.get_mut(name).unwrap().local.pop();
    }
    pub fn local_names(&self) -> HashSet<Name> {
        let mut res: HashSet<Name> = Default::default();
        for (name, sv) in &self.var {
            if !sv.local.is_empty() {
                res.insert(name.clone());
            }
        }
        res
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
                "Duplicate definition for `{}.{}`",
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
    fn overloaded_candidates(
        &self,
        name: &FullName,
        import_stmts: &[ImportStatement],
    ) -> Vec<(NameSpace, T)> {
        if !self.var.contains_key(&name.name) {
            return vec![];
        }
        let sv = &self.var[&name.name];
        if name.is_local() && sv.local.len() > 0 {
            vec![(NameSpace::local(), sv.local.last().unwrap().clone())]
        } else {
            sv.global
                .iter()
                .filter(|(ns, _)| {
                    import::is_accessible(import_stmts, &FullName::new(ns, &name.name))
                })
                .filter(|(ns, _)| name.namespace.is_suffix_of(ns))
                .map(|(ns, v)| (ns.clone(), v.clone()))
                .collect()
        }
    }
}

// Type substitution. Name of type variable -> type.
// Managed so that the value (a type) of this HashMap doesn't contain a type variable that appears in keys. i.e.,
// when we want to COMPLETELY substitute type variables in a type by `substitution`, we only apply this mapy only ONCE.
#[derive(Clone, Serialize, Deserialize)]
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
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

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
            assert!(!self.data.contains_key(var));
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
            Type::TyVar(tyvar) => self.data.get(&tyvar.name).map_or(ty.clone(), |sub| {
                sub.set_source_if_none(ty.get_source().clone())
            }),
            Type::TyCon(_) => ty.clone(),
            Type::TyApp(fun, arg) => ty
                .set_tyapp_fun(self.substitute_type(fun))
                .set_tyapp_arg(self.substitute_type(arg)),
            Type::FunTy(src, dst) => ty
                .set_funty_src(self.substitute_type(&src))
                .set_funty_dst(self.substitute_type(&dst)),
            Type::AssocTy(_, args) => 
                ty.set_assocty_args(args.iter().map(|arg| self.substitute_type(arg)).collect()),
        }
    }

    pub fn substitute_scheme(&self, scm: &Arc<Scheme>) -> Arc<Scheme> {
        // Generalized variables cannot be replaced.
        for v in &scm.gen_vars {
            assert!(!self.data.contains_key(&v.name));
        }
        let mut preds = scm.predicates.clone();
        for p in &mut preds {
            self.substitute_predicate(p)
        }
        let mut eqs = scm.equalities.clone();
        for eq in &mut eqs {
            self.substitute_equality(eq)
        }
        Scheme::new_arc(
            scm.gen_vars.clone(),
            preds,
            eqs,
            self.substitute_type(&scm.ty),
        )
    }

    // Apply substitution to qualified type.
    pub fn substitute_qualtype(&self, qual_type: &mut QualType) {
        for pred in &mut qual_type.preds {
            self.substitute_predicate(pred);
        }
        for eq in &mut qual_type.eqs {
            self.substitute_equality(eq);
        }
        qual_type.ty = self.substitute_type(&qual_type.ty);
    }

    // Apply substitution to equality.
    pub fn substitute_equality(&self, eq: &mut Equality) {
        for arg in &mut eq.args {
            *arg = self.substitute_type(arg);
        }
        eq.value = self.substitute_type(&eq.value);
    }

    pub fn substitute_qualpred(&self, qual_pred: &mut QualPredicate) {
        for pred in &mut qual_pred.pred_constraints {
            self.substitute_predicate(pred);
        }
        for eq in &mut qual_pred.eq_constraints {
            self.substitute_equality(eq);
        }
        self.substitute_predicate(&mut qual_pred.predicate);
    }

    // Calculate minimum substitution s such that `s(ty1) = ty2`.
    // NOTE: This function only searches for syntactical substitution, i.e., does not resolve associated type.
    pub fn matching(
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
        fixed_tyvars: &HashSet<Name>,
        kind_env: &KindEnv
    ) -> Option<Self> {
        match &ty1.ty {
            Type::TyVar(v1) => {
                // We do not use `unify_tyvar` here:
                // `unify_tyvar` avoids adding circular substitution, but `matching` SHOULD not avoid it.
                // For example, consider `ty1 = t0 -> t0`, `ty2 = t1 -> t0`.
                // There is no substitution `s` such that `s(ty1) = ty2`, so we should return None.
                // If we use `unify_tyvar`, it returns `{t0 -> t1}`, because
                // - `unify_tyvar` returns `{t0 -> t1}` when trying to unify the domains of `ty1` and `ty2`.
                // - `unify_tyvar` returns `{}` (empty substitution) when trying to unify the codomains of `ty1` and `ty2`.
                // - `{t0 -> t1}` and `{}` can be merged to `{t0 -> t1}`.
                // And this implementation of mathcing is the same as one in "Typing Haskell in Haskell".
                if ty1.kind(kind_env) != ty2.kind(kind_env) {
                    return None;
                }
                if fixed_tyvars.contains(&v1.name) {
                    if ty1.to_string() == ty2.to_string() {
                        return Some(Self::default());
                    } else {
                        return None;
                    }
                }
                Some(Self::single(&v1.name, ty2.clone()))
            }
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
                    match Self::matching(fun1, fun2, fixed_tyvars, kind_env) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(arg1, arg2, fixed_tyvars, kind_env) {
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
                    match Self::matching(src1, src2, fixed_tyvars, kind_env) {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return None;
                            }
                        }
                        None => return None,
                    }
                    match Self::matching(dst1, dst2, fixed_tyvars, kind_env) {
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
            Type::AssocTy(assoc_ty1, args1) => {
                match &ty2.ty {
                    Type::AssocTy(assoc_ty2, args2) => {
                        if assoc_ty1 != assoc_ty2 {
                            return None;
                        }
                        let mut ret = Self::default();
                        for i in 0..args1.len() {
                            match Self::matching(&args1[i], &args2[i], fixed_tyvars, kind_env) {
                                Some(s) => {
                                    if !ret.merge_substitution(&s) {
                                        return None;
                                    }
                                },
                                None => return None,
                            }
                        }
                        Some(ret)
                    },
                    _ => None,
                }
            },
        }
    }
}

// impl Unification {
//     pub fn is_empty(&self) -> bool {
//         self.substitution.data.is_empty() && self.equalities.is_empty()
//     }

//     pub fn single_substitution(var: &str, ty: Arc<TypeNode>) -> Unification {
//         Unification {
//             substitution: Substitution::single(var, ty),
//             equalities: vec![],
//         }
//     }

//     // pub fn from_equality(
//     //     eq: Equality,
//     //     kind_map: &HashMap<TyCon, Arc<Kind>>,
//     //     assoc_tys: &HashMap<FullName, Vec<EqualityScheme>>,
//     // ) -> Result<Unification, UnificationErr> {
//     //     let mut uni = Self::default();
//     //     uni.add_equality(eq, kind_map, assoc_tys)?;
//     //     Ok(uni)
//     // }

//     pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
//         self.substitution.substitute_type(ty)
//     }
// }

// impl TypeResolver {
//     // Set type environment.
//     pub fn set_type_env(&mut self, type_env: TypeEnv) {
//         self.kind_map = type_env.kinds();
//     }

//     // Apply substitution to type.
//     pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
//         self.unification.substitute_type(ty)
//     }

//     // Apply substitution to a predicate.
//     pub fn substitute_predicate(&self, p: &mut Predicate) {
//         self.unification.substitution.substitute_predicate(p)
//     }
// }

// In TypeCheckContext::instantiate_scheme, how constraints of type scheme is handled?
pub enum ConstraintInstantiationMode {
    // Require the constraints to be satisfied.
    Require,
    // Assume that the constraints are satisfied.
    Assume,
}

// Context under type-checking.
// Reference: https://uhideyuki.sakura.ne.jp/studs/index.cgi/ja/HindleyMilnerInHaskell#fn6
#[derive(Clone, Default)]
pub struct TypeCheckContext {
    // The identifier of type variables.
    tyvar_id: u32,
    // Scoped map of variable name -> scheme. (Assamptions of type inference.)
    pub scope: Scope<Arc<Scheme>>,
    // Substitution.
    pub substitution: Substitution,
    // Pending equalities.
    pub equalities: Vec<Equality>,
    // Collected predicates.
    pub predicates: Vec<Predicate>,
    // Trait environment.
    pub trait_env: Arc<TraitEnv>,
    // List of type constructors.
    pub type_env: TypeEnv,
    // Kind environment.
    pub kind_env: Arc<KindEnv>,
    // A map from a module to the import statements.
    // To decrease clone-cost, wrap it in reference counter.
    pub import_statements: Arc<HashMap<Name, Vec<ImportStatement>>>,
    // In which module is the current expression defined?
    // This is used as a state variable for typechecking.
    pub current_module: Option<Name>,
    // Equalities assumed.
    pub assumed_eqs: HashMap<TyAssoc, Vec<EqualityScheme>>,
    // Predicates assumed.
    pub assumed_preds: HashMap<TraitId, Vec<QualPredScheme>>,
    // Fixed type variables.
    // In unification, these type variables are not allowed to be replaced to another type.
    pub fixed_tyvars: HashSet<Name>,
}

impl TypeCheckContext {
    // Create instance.
    pub fn new(
        trait_env: TraitEnv,
        type_env: TypeEnv,
        kind_env: KindEnv,
        import_statements: HashMap<Name, Vec<ImportStatement>>,
    ) -> Self {
        let assumed_preds = trait_env.qualified_predicates();
        let assumed_eqs = trait_env.type_equalities();
        Self {
            tyvar_id: Default::default(),
            scope: Default::default(),
            type_env,
            trait_env: Arc::new(trait_env),
            kind_env: Arc::new(kind_env),
            import_statements: Arc::new(import_statements),
            current_module: None,
            substitution: Substitution::default(),
            predicates: vec![],
            equalities: vec![],
            assumed_preds,
            assumed_eqs,
            fixed_tyvars: HashSet::default(),
        }
    }

    // Get modules imported by current module.
    pub fn imported_statements(&self) -> &Vec<ImportStatement> {
        self.import_statements
            .get(self.current_module.as_ref().unwrap())
            .unwrap()
    }

    // Generate new type variable.
    pub fn new_tyvar(&mut self) -> String {
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

    // Apply substitution to an equality.
    pub fn substitute_equality(&self, eq: &mut Equality) {
        self.substitution.substitute_equality(eq)
    }

    // Instantiate a scheme.
    pub fn instantiate_scheme(
        &mut self,
        scheme: &Arc<Scheme>,
        constraint_mode: ConstraintInstantiationMode,
    ) -> Result<Arc<TypeNode>, UnificationErr> {
        let mut preds = scheme.predicates.iter().map(|pred| pred.resolve_trait_aliases(&self.trait_env).into_iter()).flatten().collect::<Vec<_>>();
        let mut eqs = scheme.equalities.clone();
        match constraint_mode {
            ConstraintInstantiationMode::Require => {
                // Instantiate type variables.
                let mut sub = Substitution::default();
                let mut new_tyvars = vec![];
                for tv in &scheme.gen_vars {
                    let new_var_name = self.new_tyvar();
                    new_tyvars.push(new_var_name.clone());
                    sub.add_substitution(&Substitution::single(&tv.name, type_tyvar(&new_var_name, &tv.kind)));
                }
                // Apply substitution to type, predicates and equalities.
                let ty = sub.substitute_type(&scheme.ty);
                for eq in &mut eqs {
                    sub.substitute_equality(eq);
                }
                // Add constraints to the TypeCheckerContext.
                for pred in &mut preds {
                    sub.substitute_predicate(pred);
                }
                self.predicates.append(&mut preds);
                for eq in eqs {
                    self.add_equality(eq)?;
                }
                return Ok(ty);
            },
            ConstraintInstantiationMode::Assume => {
                for tv in &scheme.gen_vars {
                    self.fixed_tyvars.insert(tv.name.clone());
                }
                for pred in preds {
                    let trait_id = pred.trait_id.clone();
                    let qual_pred_scm = QualPredScheme { 
                        gen_vars: vec![],
                        qual_pred: QualPredicate { 
                            pred_constraints: vec![],
                            eq_constraints: vec![],
                            kind_constraints: vec![],
                            predicate: pred
                        }
                    };
                    misc::insert_to_hashmap_vec(&mut self.assumed_preds, &trait_id, qual_pred_scm);
                }
                for eq in eqs {
                    let assoc_ty = eq.assoc_type.clone();
                    let eq_scm = EqualityScheme {
                        gen_vars: vec![],
                        equality: eq,
                    };
                    misc::insert_to_hashmap_vec(&mut self.assumed_eqs, &assoc_ty, eq_scm);
                }
                return Ok(scheme.ty.clone());
            },
        }
    }

    // Perform typechecking.
    // Update type substitution so that `ei` has type `ty`.
    // Returns given AST augmented with inferred information.
    pub fn unify_type_of_expr(&mut self, ei: &Arc<ExprNode>, ty: Arc<TypeNode>) -> Arc<ExprNode> {
        let ei = ei.set_inferred_type(ty.clone());
        match &*ei.expr {
            Expr::Var(var) => {
                let candidates = self
                    .scope
                    .overloaded_candidates(&var.name, self.imported_statements());
                if candidates.is_empty() {
                    error_exit_with_src(
                        &format!("No value `{}` is found.", var.name.to_string()),
                        &ei.source,
                    );
                }
                let candidates: Vec<_> = candidates
                    .iter()
                    .map(|(ns, scm)| {
                        let fullname = FullName::new(ns, &var.name.name);
                        let mut tc = self.clone();
                        let var_ty = tc.instantiate_scheme(&scm, ConstraintInstantiationMode::Require);
                        if let Err(e) = var_ty {
                            let msg = format!("- `{}` of type `{}` does not match since the constraint {} cannot be deduced.", 
                                fullname.to_string(), 
                                self.substitution.substitute_scheme(scm).to_string(), 
                                e.to_constraint_string()
                            );
                            Err(msg)
                        } else if let Err(e) = tc.unify(&var_ty.ok().unwrap(), &ty) {
                            let msg = format!(
                                "- `{}` of type `{}` does not match the expected type since the constraint {} cannot be deduced.",
                                fullname.to_string(),
                                self.substitution.substitute_scheme(scm).to_string(),
                                e.to_constraint_string()
                            );
                            Err(msg)
                        } else if let Err(e) = tc.reduce_predicates() {
                            let msg = format!(
                                "- `{}` of type `{}` does not match the expected type since the constraint `{}` cannot be deduced.",
                                fullname.to_string(),
                                self.substitution.substitute_scheme(scm).to_string(),
                                e.to_constraint_string()
                            );
                            Err(msg)
                        } else {
                            Ok((tc, ns.clone()))
                        }
                    })
                    .collect();
                let ok_count = candidates.iter().filter(|cand| cand.is_ok()).count();
                if ok_count == 0 {
                    error_exit_with_src(
                        &format!(
                            "No value named `{}` matches the expected type `{}`.\n{}",
                            var.name.to_string(),
                            &self.substitute_type(&ty).to_string_normalize(),
                            candidates
                                .iter()
                                .map(|cand| cand.as_ref().err().unwrap().clone())
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                        &ei.source,
                    );
                } else if ok_count >= 2 {
                    let candidates_str = candidates
                        .iter()
                        .filter_map(|cand| cand.as_ref().ok())
                        .map(|(_, ns)| {
                            let fullname = FullName::new(&ns, &var.name.name);
                            "`".to_string() + &fullname.to_string() + "`"
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    error_exit_with_src(
                        &format!(
                            "Name `{}` is ambiguous: there are {}. Maybe you need to write (suffix of) its namespace or type annotation to help overloading resolution.",
                            var.name.to_string(),
                            candidates_str
                        ),
                        &ei.source,
                    );
                } else {
                    // candidates.len() == 1
                    let (tc, ns) = candidates
                        .iter()
                        .find_map(|cand| cand.as_ref().ok())
                        .unwrap();
                    *self = tc.clone();
                    ei.set_var_namespace(ns.clone())
                }
            }
            Expr::LLVM(lit) => {
                if let Err(_) = self.unify_rollback_if_err(&lit.ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &lit.ty.to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                ei.clone()
            }
            Expr::App(fun, args) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                if ei.app_order == AppSourceCodeOrderType::ArgumentIsFormer {
                    let arg = self.unify_type_of_expr(&arg, arg_ty.clone());
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty));
                    ei.set_app_args(vec![arg]).set_app_func(fun)
                } else {
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty));
                    let arg = self.unify_type_of_expr(&arg, arg_ty.clone());
                    ei.set_app_args(vec![arg]).set_app_func(fun)
                }
            }
            Expr::Lam(args, body) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_ty = type_tyvar_star(&self.new_tyvar());
                let body_ty = type_tyvar_star(&self.new_tyvar());
                let fun_ty = type_fun(arg_ty.clone(), body_ty.clone());
                if let Err(_) = self.unify_rollback_if_err(&fun_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&fun_ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                assert!(arg.name.is_local());
                self.scope.push(&arg.name.name, &Scheme::from_type(arg_ty));
                let body = self.unify_type_of_expr(body, body_ty);
                self.scope.pop(&arg.name.name);
                ei.set_lam_body(body)
            }
            Expr::Let(pat, val, body) => {
                pat.error_if_invalid(&self.type_env);
                let (pat_ty, var_ty) = pat.pattern.get_type(self);
                let val = self.unify_type_of_expr(val, pat_ty.clone());
                let var_scm = var_ty.iter().map(|(name, ty)| {
                    (
                        name.clone(),
                        Scheme::from_type(ty.clone()),
                    )
                });
                for (name, scm) in var_scm.clone() {
                    assert!(name.is_local());
                    self.scope.push(&name.name, &scm);
                }
                let body = self.unify_type_of_expr(body, ty);
                for (name, _) in var_scm {
                    self.scope.pop(&name.name);
                }
                ei.set_let_bound(val).set_let_value(body)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.unify_type_of_expr(cond, make_bool_ty());
                let then_expr = self.unify_type_of_expr(then_expr, ty.clone());
                let else_expr = self.unify_type_of_expr(else_expr, ty);
                ei.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::TyAnno(e, anno_ty) => {
                for tv in anno_ty.free_vars_vec() {
                    if !self.fixed_tyvars.contains(&tv.name) {
                        error_exit_with_src(
                            &format!("Unknown type variable `{}`.", tv.name),
                            &ei.source,
                        )
                    }
                }
                // Add predicates required by associated type usages in `anno_ty`.
                let mut req_preds = anno_ty.predicates_from_associated_types();
                for req_pred in &mut req_preds {
                    self.substitute_predicate(req_pred);
                }
                self.predicates.append(&mut req_preds.clone());
                if let Err(_) = self.unify_rollback_if_err(&ty, anno_ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&anno_ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                let e = self.unify_type_of_expr(e, ty.clone());
                ei.set_tyanno_expr(e)
            }
            Expr::MakeStruct(tc, fields) => {
                // Get list of field names.
                let ti = self.type_env.tycons.get(tc);
                if ti.is_none() {
                    error_exit_with_src(
                        &format!("Unknown type name `{}`.", tc.to_string()),
                        &ei.source,
                    );
                }
                let ti = ti.unwrap();
                let field_names = ti.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();

                // Validate fields.
                let field_names_in_struct_defn: HashSet<Name> =
                    HashSet::from_iter(field_names.iter().cloned());
                let field_names_in_expression: HashSet<Name> =
                    HashSet::from_iter(fields.iter().map(|(name, _)| name.clone()));
                for f in &field_names_in_struct_defn {
                    if !field_names_in_expression.contains(f) {
                        error_exit_with_src(
                            &format!("Missing field `{}` of struct `{}`.", f, tc.to_string()),
                            &ei.source,
                        )
                    }
                }
                for f in &field_names_in_expression {
                    if !field_names_in_struct_defn.contains(f) {
                        error_exit_with_src(
                            &format!("Unknown field `{}` for struct `{}`.", f, tc.to_string()),
                            &ei.source,
                        )
                    }
                }

                // Get field types.
                let struct_ty = tc.get_struct_union_value_type(self);
                if let Err(_) = self.unify_rollback_if_err(&struct_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&struct_ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                let field_tys = struct_ty.field_types(&self.type_env);
                assert_eq!(field_tys.len(), fields.len());

                // Reorder fields as ordering of fields in struct definition.
                let fields: HashMap<Name, Arc<ExprNode>> =
                    HashMap::from_iter(fields.iter().cloned());
                let mut fields = field_names
                    .iter()
                    .map(|name| (name.clone(), fields[name].clone()))
                    .collect::<Vec<_>>();

                for (field_ty, (_, field_expr)) in field_tys.iter().zip(fields.iter_mut()) {
                    *field_expr = self.unify_type_of_expr(field_expr, field_ty.clone());
                }
                ei.set_make_struct_fields(fields)
            }
            Expr::ArrayLit(elems) => {
                // Prepare type of element.
                let elem_ty = type_tyvar_star(&self.new_tyvar());
                let array_ty = type_tyapp(make_array_ty(), elem_ty.clone());
                if let Err(_) = self.unify_rollback_if_err(&array_ty, &ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found an array.",
                            &self.substitute_type(&ty).to_string_normalize(),
                        ),
                        &ei.source,
                    );
                }
                let mut ei = ei.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.unify_type_of_expr(e, elem_ty.clone());
                    ei = ei.set_array_lit_elem(e, i);
                }
                ei
            }
            Expr::CallC(_, ret_ty, param_tys, is_va_args, args) => {
                let ret_ty = type_tycon(ret_ty);
                if let Err(_) = self.unify_rollback_if_err(&ty, &ret_ty) {
                    error_exit_with_src(
                        &format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&ret_ty).to_string_normalize()
                        ),
                        &ei.source,
                    );
                }
                let param_tys = param_tys
                    .iter()
                    .map(|tc| type_tycon(tc))
                    .collect::<Vec<_>>();
                let mut ei = ei.clone();
                for (i, e) in args.iter().enumerate() {
                    let expect_ty = if i < param_tys.len() {
                        param_tys[i].clone()
                    } else {
                        assert!(is_va_args);
                        type_tyvar_star(&self.new_tyvar())
                    };
                    let e = self.unify_type_of_expr(e, expect_ty);
                    ei = ei.set_call_c_arg(e, i);
                }
                ei
            }
        }
    }

    // Check if expr has type scm.
    // Returns given AST set with inferred type.
    pub fn check_type(&mut self, expr: Arc<ExprNode>, expect_scm: Arc<Scheme>) -> Arc<ExprNode> {
        // This function should be called when TypeCheckContext is "fresh".
        assert!(self.substitution.is_empty());
        assert!(self.predicates.is_empty());
        assert!(self.equalities.is_empty());

        let specified_ty = self.instantiate_scheme(&expect_scm, ConstraintInstantiationMode::Assume);
        if let Err(e) = specified_ty {
            error_exit_with_src(
                &format!(
                    "`{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    e.to_constraint_string()
                ), &expr.source
            );
        }
        let specified_ty = specified_ty.ok().unwrap();
        let expr = self.unify_type_of_expr(&expr, specified_ty.clone());
        let reduction_res = self.reduce_predicates();
        if let Err(e) = reduction_res {
            error_exit_with_src(
                &format!(
                    "`{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    e.to_constraint_string()
                ), &expr.source
            );
        }
        if self.predicates.len() > 0 {
            let pred = &self.predicates[0];
            error_exit_with_src(
                &format!(
                    "Condition `{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    pred.to_string()
                ),
                &expr.source,
            );
        }
        if self.equalities.len() > 0 {
            let eq = &self.equalities[0];
            error_exit_with_src(
                &format!(
                    "Condition `{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    eq.to_string()
                ),
                &expr.source,
            );
        }

        expr
    }

    fn add_substitution(&mut self, subst: &Substitution) -> Result<(), UnificationErr> {
        self.substitution.add_substitution(subst);
        let eqs = std::mem::replace(&mut self.equalities, vec![]);
        for eq in eqs {
            self.add_equality(eq)?;
        }
        Ok(())
    }

    fn add_equality(&mut self, mut eq: Equality) -> Result<(), UnificationErr> {
        self.substitute_equality(&mut eq);
        let red_lhs = self.reduce_type_by_equality(eq.lhs());
        if red_lhs.to_string() != eq.lhs().to_string() {
            self.unify(&red_lhs, &eq.value)?;
        } else {
            eq.value = self.reduce_type_by_equality(eq.value.clone());
            if eq.lhs().to_string() != eq.value.to_string() {
                self.equalities.push(eq);
            }
        }
        Ok(())
    }

    // Reduce a type by replacing associated type to its value.
    fn reduce_type_by_equality(&mut self, ty: Arc<TypeNode>) -> Arc<TypeNode> {
        match &ty.ty {
            Type::TyVar(_) => ty,
            Type::TyCon(_) => ty,
            Type::TyApp(tyfun, tyarg) => {
                let tyfun = self.reduce_type_by_equality(tyfun.clone());
                let tyarg = self.reduce_type_by_equality(tyarg.clone());
                ty.set_tyapp_fun(tyfun).set_tyapp_arg(tyarg)
            },
            Type::FunTy(tysrc, tydst) => {
                let tysrc = self.reduce_type_by_equality(tysrc.clone());
                let tydst = self.reduce_type_by_equality(tydst.clone());
                ty.set_funty_src(tysrc).set_funty_dst(tydst)
            },
            Type::AssocTy(assoc_ty, args) => {
                // Reduce each arguments. 
                let args = args.iter().map(|arg| self.reduce_type_by_equality(arg.clone())).collect::<Vec<_>>();
                let ty = ty.set_assocty_args(args);

                // Try matching to assumed equality.
                for assumed_eq in &self.assumed_eqs.get(assoc_ty).unwrap().clone() {
                    // Instantiate `assumed_eq`.
                    let mut subst = Substitution::default();
                    for tv in &assumed_eq.gen_vars {
                        subst.add_substitution(&Substitution::single(&tv.name, type_tyvar(&self.new_tyvar(), &tv.kind)));
                    }
                    let mut equality = assumed_eq.equality.clone();
                    subst.substitute_equality(&mut equality);

                    // Try to match lhs of `equality` to `ty`.
                    let equality = &assumed_eq.equality;
                    let subst: Option<Substitution> = Substitution::matching(&equality.lhs(), &ty, &self.fixed_tyvars, &self.kind_env);
                    if subst.is_none() {
                        continue;
                    }
                    let subst: Substitution = subst.unwrap();
                    let rhs = subst.substitute_type(&equality.value);
                    return self.reduce_type_by_equality(rhs);
                }
                return ty;
            },
        }
    }

    // Update unification to unify two types.
    // When unification fails, it has no side effect to self.
    pub fn unify_rollback_if_err(
        &mut self,
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
    ) -> Result<(), UnificationErr> {
        let mut cloned_self = self.clone();
        match cloned_self.unify(ty1, ty2) {
            Ok(_) => {
                *self = cloned_self;
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Unify two types.
    pub fn unify(
        &mut self,
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
    ) -> Result<(), UnificationErr> {
        let mut ty1 = &self.substitute_type(ty1);
        let mut ty2 = &self.substitute_type(ty2);
        if ty1.to_string() == ty2.to_string() {
            return Ok(());
        }
        // Case: Either is a type variable.
        for _ in 0..2 {
            match &ty1.ty {
                Type::TyVar(var1) => {
                    if !self.fixed_tyvars.contains(&var1.name) {
                        return self.unify_tyvar(var1.clone(), ty2.clone());
                    }
                }
                _ => {}
            }
            std::mem::swap(&mut ty1, &mut ty2);
        }
        // Case: Either is usage of associated type.
        for _ in 0..2 {
            if let Type::AssocTy(assoc_ty, args) = &ty1.ty
            {
                let eq = Equality {
                    assoc_type: assoc_ty.clone(),
                    args:args.clone(),
                    value: ty2.clone(),
                    source: None,
                };
                self.add_equality(eq)?;
                return Ok(());
            }
            std::mem::swap(&mut ty1, &mut ty2);
        }

        // Other case.
        match &ty1.ty {
            Type::TyVar(_) => {
                // If the code reaches here, `ty1` is a fixed type variable, and `ty1` is not equal to `ty2`.
                return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()));
            },
            Type::AssocTy(_, _) => unreachable!(),
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1 == tc2 {
                        return Ok(());
                    } else {
                        return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()));
                    }
                }
                _ => {
                    return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()));
                }
            },
            Type::TyApp(fun1, arg1) => match &ty2.ty {
                Type::TyApp(fun2, arg2) => {
                    self.unify(&fun1, &fun2)?;
                    let arg1 = self.substitute_type(arg1);
                    let arg2 = self.substitute_type(arg2);
                    self.unify(&arg1, &arg2)?;
                    return Ok(());
                }
                _ => {
                    return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()));
                }
            },
            Type::FunTy(arg_ty1, ret_ty1) => match &ty2.ty {
                Type::FunTy(arg_ty2, ret_ty2) => {
                    self.unify(&arg_ty1, &arg_ty2)?;
                    let ret_ty1 = self.substitute_type(ret_ty1);
                    let ret_ty2 = self.substitute_type(ret_ty2);
                    self.unify(&ret_ty1, &ret_ty2)?;
                    return Ok(());
                }
                _ => {
                    return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()));
                }
            },
        }
    }

    // Subroutine of unify().
    fn unify_tyvar(
        &mut self,
        tyvar1: Arc<TyVar>,
        ty2: Arc<TypeNode>,
    ) -> Result<(), UnificationErr> {
        assert!(!self.fixed_tyvars.contains(&tyvar1.name));
        match &ty2.ty {
            Type::TyVar(tyvar2) => {
                if tyvar1.name == tyvar2.name {
                    // Avoid adding circular subsitution.
                    return Ok(());
                }
            }
            _ => {}
        };
        if ty2.free_vars().contains_key(&tyvar1.name) {
            // For example, this error occurs when
            // the user is making `f c` in the implementation of
            // `map: [f: Functor] (a -> b) -> f a -> f b; map = |f, c| (...)`;
            return Err(UnificationErr::Disjoint(type_from_tyvar(tyvar1), ty2));
        }
        if tyvar1.kind != ty2.kind(&self.kind_env) {
            return Err(UnificationErr::Disjoint(type_from_tyvar(tyvar1), ty2));
        }
        self.add_substitution(&Substitution::single(&tyvar1.name, ty2.clone()))?;
        Ok(())
    }

    // Reduce predicates as long as possible.
    // If predicates are unsatisfiable, return Err.
    fn reduce_predicates(&mut self) -> Result<(), UnificationErr> {
        let preds = std::mem::replace(&mut self.predicates, vec![]);
        let mut already_added : HashSet<String> = HashSet::new();
        for pred in preds {
            self.add_predicate_reducing(pred, &mut already_added)?;
        }
        Ok(())
    }

    // Add a predicate after reducing it.
    fn add_predicate_reducing(&mut self, pred : Predicate, already_added: &mut HashSet<Name>) -> Result<(), UnificationErr> {
        for pred in pred.resolve_trait_aliases(&self.trait_env) {
            self.add_predicate_reducing_noalias(pred, already_added)?;
        }
        Ok(())
    }

    // Add a predicate after reducing it.
    // Trait in `pred` should not be a trait alias.
    fn add_predicate_reducing_noalias(&mut self, mut pred : Predicate, already_added: &mut HashSet<Name>) -> Result<(), UnificationErr> {
        self.substitute_predicate(&mut pred);
        pred.ty = self.reduce_type_by_equality(pred.ty);
        let pred_str = pred.to_string();
        if already_added.contains(&pred_str) {
            return Ok(());
        }
        already_added.insert(pred_str);
        let mut unifiable = false;
        for qual_pred_scm in &self.assumed_preds.get(&pred.trait_id).unwrap_or(&vec![]).clone() {
            // Instantiate qualified predicate.
            let mut subst = Substitution::default();
            for tv in &qual_pred_scm.gen_vars {
                subst.add_substitution(&Substitution::single(&tv.name, type_tyvar(&self.new_tyvar(), &tv.kind)));
            }
            let mut qual_pred = qual_pred_scm.qual_pred.clone();
            subst.substitute_qualpred(&mut qual_pred);

            // Try to match head of `qual_pred` to `pred`.
            if let Some(subst) = Substitution::matching(&qual_pred.predicate.ty, &pred.ty, &self.fixed_tyvars, &self.kind_env) {
                for mut eq in qual_pred.eq_constraints {
                    subst.substitute_equality(&mut eq);
                    self.add_equality(eq)?;
                }
                for mut pred in qual_pred.pred_constraints { 
                    subst.substitute_predicate(&mut pred);
                    self.add_predicate_reducing(pred, already_added)?;
                }
                return Ok(());
            } else {
                // If match fails, then we cannot reduce the predicate at now.
                // But we may be able to reduce it after the predicate is substituted further.
                // To see if there is possibility for further reduction, we check here the unifiability.
                let mut tc = self.clone();
                if tc.unify(&qual_pred.predicate.ty, &pred.ty).is_ok() {
                    unifiable = true;
                }
            }
        }
        if !unifiable {
            return Err(UnificationErr::Unsatisfiable(pred));
        }
        self.predicates.push(pred);
        return Ok(());
    }

    pub fn finish_inferred_types(&mut self, expr: Arc<ExprNode>) -> Arc<ExprNode> {
        let ty = self.substitute_type(expr.ty.as_ref().unwrap());
        let ty = self.reduce_type_by_equality(ty);
        let expr = expr.set_inferred_type(ty);
        match &*expr.expr {
            Expr::Var(_) => expr,
            Expr::LLVM(_) => expr,
            Expr::App(fun, args) => {
                let args = args.iter().map(|arg| self.finish_inferred_types(arg.clone())).collect::<Vec<_>>();
                let fun = self.finish_inferred_types(fun.clone());
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_args, body) => {
                let body = self.finish_inferred_types(body.clone());
                expr.set_lam_body(body)
            }
            Expr::Let(_pat, val, body) => {
                let val = self.finish_inferred_types(val.clone());
                let body = self.finish_inferred_types(body.clone());
                expr.set_let_bound(val).set_let_value(body)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.finish_inferred_types(cond.clone());
                let then_expr = self.finish_inferred_types(then_expr.clone());
                let else_expr = self.finish_inferred_types(else_expr.clone());
                expr.set_if_cond(cond).set_if_then(then_expr).set_if_else(else_expr)
            }
            Expr::TyAnno(e, _) => expr.set_tyanno_expr(self.finish_inferred_types(e.clone())),
            Expr::MakeStruct(_tc, fields) => {
                let fields = fields.iter().map(|(name, e)| (name.clone(), self.finish_inferred_types(e.clone()))).collect::<Vec<_>>();
                expr.set_make_struct_fields(fields)
            }
            Expr::ArrayLit(elems) => {
                let elems = elems.iter().map(|e| self.finish_inferred_types(e.clone())).collect::<Vec<_>>();
                expr.set_array_lit_elems(elems)
            }
            Expr::CallC(_, _, _, _, args) => {
                let args = args.iter().map(|arg| self.finish_inferred_types(arg.clone())).collect::<Vec<_>>();
                expr.set_call_c_args(args)
            }
        }
    }
}

pub enum UnificationErr {
    Unsatisfiable(Predicate),
    Disjoint(Arc<TypeNode>, Arc<TypeNode>),
}

impl UnificationErr {
    pub fn to_constraint_string(&self) -> String {
        match self {
            UnificationErr::Unsatisfiable(p) => {
                p.to_string()
            },
            UnificationErr::Disjoint(ty1, ty2) => {
                format!("`{}` = `{}`", ty1.to_string(), ty2.to_string())
            },
        }
    }
}

        // pub fn reduce_to_context_of_instance(
    //     &self,
    //     p: &Predicate,
    //     kind_map: &HashMap<TyCon, Arc<Kind>>,
    // ) -> Option<Vec<Predicate>> {
    //     let insntances = self.instances.get(&p.trait_id);
    //     if let Some(instances) = insntances {
    //         for inst in instances {
    //             match Substitution::matching(&inst.qual_pred.predicate.ty, &p.ty) {
    //                 Some(s) => {
    //                     let ps = inst.qual_pred.context.iter().map(|c| {
    //                         let mut c = c.clone();
    //                         s.substitute_predicate(&mut c);
    //                         c
    //                     });
    //                     let mut ret = vec![];
    //                     for p in ps {
    //                         ret.append(&mut p.resolve_trait_aliases(self));
    //                     }
    //                     return Some(ret);
    //                 }
    //                 None => {}
    //             }
    //         }
    //     }
    //     return None;
    // }


    // fn add_unification(&mut self, uni: &Unification) -> Result<(), UnificationErr> {
    //     self.add_substitution(&uni.substitution)?;
    //     for eq in &uni.equalities {
    //         self.add_equality(eq.clone())?;
    //     }
    //     Ok(())
    // }

    // // Try to reduce an equality into an unification. 
    // // If equality is unsatisfiable, return Err.
    // // If equality is satisfiable, returns true if it is reduced and added to unification.
    // fn reduce_equality(&mut self, eq: &Equality) -> Result<bool, UnificationErr> {
    //     // Try assumed equality.
    //     for assumed_eq in &self.assumed_eqs[&eq.assoc_type] {
    //         if eq.lhs().to_string() != assumed_eq.lhs().to_string() {
    //             continue;
    //         }
    //         self.unify(&eq.value, &assumed_eq.value)?;
    //         return Ok(true);
    //     }
    //     // If `impl_type` is head normal form, then it cannot be unified to trait instance.
    //     if eq.impl_type.is_hnf() {
    //         return Ok(false);
    //     }
    //     // Try matching to associated type instance.
    //     for assoc_type_inst in &self.assoc_tys[&eq.assoc_type] {
    //         let lhs1 = assoc_type_inst.equality.lhs();
    //         let lhs2 = eq.lhs();
    //         let s = Substitution::matching(&self.kind_map, &lhs1, &lhs2);
    //         if s.is_none() {
    //             continue;
    //         }
    //         let s = s.unwrap();
    //         let rhs1 = s.substitute_type(&assoc_type_inst.equality.value);
    //         let rhs2 = eq.value;
    //         self.unify(&rhs1, &rhs2)?;
    //         return Ok(true);
    //     }
    //     return Err(UnificationErr::Unsatisfiable(eq.predicate()));
    // }