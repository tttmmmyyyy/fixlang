use std::sync::Arc;

use crate::error::Errors;
use ast::{import::ImportStatement, name::{FullName, NameSpace}};
use misc::{collect_results, make_map, Map, Set};
use serde::{Deserialize, Serialize};
use typecheckcache::TypeCheckCache;

use self::ast::import;

use super::*;

#[derive(Clone)]
pub struct Scope<T> {
    // Map from variable name to its value stacks.
    local: Map<Name, Vec<T>>,
    // List of pairs of global names and its values.
    // Arc for sharing the list among multiple scopes.
    global: Arc<Vec<(FullName, T)>>,
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        Self {
            local: Default::default(),
            global: Arc::new(Default::default()),
        }
    }
}

impl<T> Scope<T>
where
    T: Clone,
{
    // Push a local value.
    pub fn push(&mut self, name: &Name, v: T) {
        misc::insert_to_map_vec(&mut self.local, name, v);
    }

    // Pop a local value.
    pub fn pop(self: &mut Self, name: &Name) {
        self.local.get_mut(name).unwrap().pop();
    }
    
    // Check if a local value exists.
    fn has_value(&self, name: &Name) -> bool {
        self.local.contains_key(name) && !self.local[name].is_empty()
    }

    // Get a set of local names.
    pub fn local_names(&self) -> Set<Name> {
        let mut res: Set<Name> = Default::default();
        for (name, stack) in &self.local {
            if !stack.is_empty() {
                res.insert(name.clone());
            }
        }
        res
    }

    // Set global values.
    pub fn set_globals(&mut self, globals: Vec<(FullName, T)>) {
        self.global = Arc::new(globals);
    }

    // Get candidates list for overload resolution.
    fn overloaded_candidates(
        &self,
        name: &FullName,
        import_stmts: &[ImportStatement],
    ) -> Vec<(NameSpace, T)> {
        if name.is_local() && self.has_value(&name.name) {
            vec![(NameSpace::local(), self.local[&name.name].last().unwrap().clone())]
        } else {
            self.global
                .iter()
                .filter(|(full_name, _)| {
                    full_name.name == name.name && name.namespace.is_suffix_of(&full_name.namespace) && 
                    import::is_accessible(import_stmts, full_name)
                })
                .map(|(full_name, v)| (full_name.namespace.clone(), v.clone()))
                .collect()
        }
    }
}

// Type substitution. Name of type variable -> type.
// Managed so that the value (a type) of this HashMap doesn't contain a type variable that appears in keys. i.e.,
// when we want to COMPLETELY substitute type variables in a type by `substitution`, we only apply this mapy only ONCE.
#[derive(Clone, Serialize, Deserialize)]
pub struct Substitution {
    pub data: Map<Name, Arc<TypeNode>>,
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
        let mut data = Map::<String, Arc<TypeNode>>::default();
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
    pub fn merge_substitution(&mut self, other: &Self) -> bool {
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
    // Returns None if such substitution does not exist.
    // NOTE: This function only searches for syntactical substitution, i.e., does not resolve associated type.
    pub fn matching(
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
        fixed_tyvars: &[Arc<TyVar>],
        kind_env: &KindEnv
    ) -> Result<Option<Self>, Errors> {
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
                if ty1.kind(kind_env)? != ty2.kind(kind_env)? {
                    return Ok(None);
                }
                if fixed_tyvars.iter().any(|tv| tv.name == v1.name) {
                    if ty1.to_string() == ty2.to_string() {
                        return Ok(Some(Self::default()));
                    } else {
                        return Ok(None);
                    }
                }
                return Ok(Some(Self::single(&v1.name, ty2.clone())))
            }
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1 == tc2 {
                        return Ok(Some(Self::default()))
                    } else {
                        return Ok(None)
                    }
                }
                _ => return Ok(None),
            },
            Type::TyApp(fun1, arg1) => match &ty2.ty {
                Type::TyApp(fun2, arg2) => {
                    let mut ret = Self::default();
                    match Self::matching(fun1, fun2, fixed_tyvars, kind_env)? {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return Ok(None);
                            }
                        }
                        None => return Ok(None),
                    }
                    match Self::matching(arg1, arg2, fixed_tyvars, kind_env)? {
                        Some(s) => {
                            if !ret.merge_substitution(&s) {
                                return Ok(None);
                            }
                        }
                        None => return Ok(None),
                    }
                    return Ok(Some(ret))
                }
                _ => return Ok(None),
            },
            Type::AssocTy(assoc_ty1, args1) => {
                match &ty2.ty {
                    Type::AssocTy(assoc_ty2, args2) => {
                        if assoc_ty1 != assoc_ty2 {
                            return Ok(None);
                        }
                        let mut ret = Self::default();
                        for i in 0..args1.len() {
                            match Self::matching(&args1[i], &args2[i], fixed_tyvars, kind_env)? {
                                Some(s) => {
                                    if !ret.merge_substitution(&s) {
                                        return Ok(None);
                                    }
                                },
                                None => return Ok(None),
                            }
                        }
                        return Ok(Some(ret))
                    },
                    _ => return Ok(None),
                }
            },
        }
    }
}

// In TypeCheckContext::instantiate_scheme, how constraints of type scheme should be handled?
pub enum ConstraintInstantiationMode {
    // Require the constraints to be satisfied.
    Require,
    // Assume that the constraints are satisfied.
    Assume,
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
    pub import_statements: Arc<Map<Name, Vec<ImportStatement>>>,
    // In which module is the current expression defined?
    // This is used as a state variable for typechecking.
    pub current_module: Option<Name>,
    // Equalities assumed.
    pub assumed_eqs: Map<TyAssoc, Vec<EqualityScheme>>,
    // Predicates assumed.
    pub assumed_preds: Map<Trait, Vec<QualPredScheme>>,
    // Fixed type variables.
    // In unification, these type variables are not allowed to be replaced to another type.
    // NOTE: We use `Vec` instead of `Set` because the expected size is small.
    pub fixed_tyvars: Vec<Arc<TyVar>>,
    // Type check cache.
    pub cache: Arc<dyn TypeCheckCache + Sync + Send>,
    // Number of worker threads.
    pub num_worker_threads: usize,
}

impl TypeCheckContext {
    #[allow(dead_code)]
    pub fn show_sizes(&self) {
        println!("scope size = {}", self.scope.local.len());
        println!("substitution size = {}", self.substitution.data.len());
        println!("equalities size = {}", self.equalities.len());
        println!("predicates size = {}", self.predicates.len());
        println!("assumed_eqs size = {}", self.assumed_eqs.len());
        println!("assumed_preds size = {}", self.assumed_preds.len());
        println!("fixed_tyvars size = {}", self.fixed_tyvars.len());
    }

    // Create instance.
    pub fn new(
        trait_env: TraitEnv,
        type_env: TypeEnv,
        kind_env: KindEnv,
        import_statements: Map<Name, Vec<ImportStatement>>,
        cache: Arc<dyn TypeCheckCache + Sync + Send>,
        num_worker_threads: usize,
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
            fixed_tyvars: vec![],
            cache,
            num_worker_threads,
        }
    }

    // Get modules imported by current module.
    pub fn imported_statements(&self) -> &Vec<ImportStatement> {
        self.import_statements
            .get(self.current_module.as_ref().unwrap())
            .unwrap()
    }

    // Generate new type variable.
    pub fn new_tyvar(&mut self, kind: Option<Arc<Kind>>, src: Option<Span>) -> Arc<TypeNode> {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
         // To avlid confliction with user-defined type variable, we use prefix #a.
        let tv_name = "#a".to_string() + &id.to_string();
        let kind = kind.unwrap_or(kind_star());
        type_tyvar(&tv_name, &kind).set_source(src)
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
    ) -> Result<Arc<TypeNode>, UnifOrOtherErr> {
        let mut preds = vec![];
        for pred in &scheme.predicates {
            preds.append(&mut pred.resolve_trait_aliases(&self.trait_env)?);
        }
        let mut eqs = scheme.equalities.clone();
        match constraint_mode {
            ConstraintInstantiationMode::Require => {
                // Instantiate type variables.
                let mut sub = Substitution::default();
                for tv in &scheme.gen_vars {
                    let new_var = self.new_tyvar(Some(tv.kind.clone()), scheme.get_first_tv_source(&tv.name));
                    sub.add_substitution(&Substitution::single(&tv.name, new_var));
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
                    self.fixed_tyvars.push(tv.clone());
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
                    misc::insert_to_map_vec(&mut self.assumed_preds, &trait_id, qual_pred_scm);
                }
                for eq in eqs {
                    let assoc_ty = eq.assoc_type.clone();
                    let eq_scm = EqualityScheme {
                        gen_vars: vec![],
                        equality: eq,
                    };
                    misc::insert_to_map_vec(&mut self.assumed_eqs, &assoc_ty, eq_scm);
                }
                return Ok(scheme.ty.clone());
            },
        }
    }

    pub fn validate_type_annotation(&mut self, ty: &Arc<TypeNode>) -> Result<Arc<TypeNode>, Errors> {
        // All type variables should be fixed by the TypeCheckContext, i.e., appear in the generalized variables of the current scheme.
        for tv in ty.free_vars_vec() {
            if !self.fixed_tyvars.iter().any(|fixed_tv| fixed_tv.name == tv.name) {
                return Err(Errors::from_msg_srcs(
                    format!("Unknown type variable `{}`.", tv.name),
                    &[&ty.get_source()],
                ));
            }
        }

        // Set kinds of type variables in the type annotation.
        let sub = Substitution {
            data : make_map(self.fixed_tyvars.iter().map(|tv| (tv.name.clone(), type_from_tyvar(tv.clone()))))
        };
        let ty = sub.substitute_type(ty);

        // Add predicates required by associated type usages in `anno_ty`.
        let mut req_preds = ty.predicates_from_associated_types();
        for req_pred in &mut req_preds {
            self.substitute_predicate(req_pred);
        }
        self.predicates.append(&mut req_preds.clone());

        Ok(ty)
    }

    // Perform typechecking.
    // Update type substitution so that `ei` has type `ty`.
    // Returns given AST augmented with inferred information.
    pub fn unify_type_of_expr(&mut self, ei: &Arc<ExprNode>, ty: Arc<TypeNode>) -> Result<Arc<ExprNode>, Errors> {
        let ei = ei.set_inferred_type(ty.clone());
        match &*ei.expr {
            Expr::Var(var) => {
                let candidates = self
                    .scope
                    .overloaded_candidates(&var.name, self.imported_statements());
                if candidates.is_empty() {
                    return Err(Errors::from_msg_srcs(
                        format!("No value `{}` is found.", var.name.to_string()),
                        &[&ei.source],
                    ));
                }
                let mut overload_res: Vec<Result<_, _>> = vec![];
                for (ns, scm) in &candidates {
                    let fullname = FullName::new(ns, &var.name.name);
                        let mut tc = self.clone();
                        let var_ty = UnifOrOtherErr::extract_others(tc.instantiate_scheme(&scm, ConstraintInstantiationMode::Require))?;
                        if let Err(e) = var_ty {
                            let msg = format!("- `{}` of type `{}` does not match since the constraint {} cannot be deduced.", 
                                fullname.to_string(), 
                                self.substitution.substitute_scheme(scm).to_string_normalize(), 
                                e.to_constraint_string()
                            );
                            overload_res.push(Err(msg))
                        } else if let Err(e) = UnifOrOtherErr::extract_others(tc.unify(&var_ty.ok().unwrap(), &ty))? {
                            let msg = format!(
                                "- `{}` of type `{}` does not match the expected type since the constraint {} cannot be deduced.",
                                fullname.to_string(),
                                self.substitution.substitute_scheme(scm).to_string_normalize(),
                                e.to_constraint_string()
                            );
                            overload_res.push(Err(msg))
                        } else if let Err(e) = UnifOrOtherErr::extract_others(tc.reduce_predicates())? {
                            let msg = format!(
                                "- `{}` of type `{}` does not match the expected type since the constraint `{}` cannot be deduced.",
                                fullname.to_string(),
                                self.substitution.substitute_scheme(scm).to_string_normalize(),
                                e.to_constraint_string()
                            );
                            overload_res.push(Err(msg))
                        } else {
                            overload_res.push(Ok((tc, ns.clone())))
                        }
                }
                let ok_count = overload_res.iter().filter(|cand| cand.is_ok()).count();
                if ok_count == 0 {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "No value named `{}` matches the expected type `{}`.\n{}",
                            var.name.to_string(),
                            &self.substitute_type(&ty).to_string_normalize(),
                            overload_res
                                .iter()
                                .map(|cand| cand.as_ref().err().unwrap().clone())
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                        &[&ei.source],
                    ));
                } else if ok_count >= 2 {
                    // FullName of candidates.
                    let candidates = overload_res
                        .iter()
                        .filter_map(|cand| cand.as_ref().ok())
                        .map(|(_, ns)| {
                            FullName::new(&ns, &var.name.name)
                        })
                        .collect::<Vec<_>>();
                    let msg = NameResolutionContext::create_ambiguous_message(&var.name.to_string(), candidates,true);                  
                    return Err(Errors::from_msg_srcs(msg,&[&ei.source]));
                } else {
                    // candidates.len() == 1
                    let (tc, ns) = overload_res
                        .iter()
                        .find_map(|cand| cand.as_ref().ok())
                        .unwrap();
                    *self = tc.clone();
                    Ok(ei.set_var_namespace(ns.clone()))
                }
            }
            Expr::LLVM(lit) => {
                if let Err(_) = UnifOrOtherErr::extract_others(self.unify(&lit.ty, &ty))? {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &lit.ty.to_string_normalize(),
                        ),
                        &[&ei.source],
                    ));
                }
                Ok(ei.clone())
            }
            Expr::App(fun, args) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_ty = self.new_tyvar(None, arg.source.clone());
                if ei.app_order == AppSourceCodeOrderType::XDotF {
                    let arg = self.unify_type_of_expr(&arg, arg_ty.clone())?;
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty))?;
                    Ok(ei.set_app_args(vec![arg]).set_app_func(fun))
                } else {
                    let fun = self.unify_type_of_expr(fun, type_fun(arg_ty.clone(), ty))?;
                    let arg = self.unify_type_of_expr(&arg, arg_ty.clone())?;
                    Ok(ei.set_app_args(vec![arg]).set_app_func(fun))
                }
            }
            Expr::Lam(args, body) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_ty = self.new_tyvar(None, body.get_first_var_source(&arg.name));
                let body_ty = self.new_tyvar(None, body.source.clone());
                let fun_ty = type_fun(arg_ty.clone(), body_ty.clone());
                if let Err(_) = UnifOrOtherErr::extract_others(self.unify(&fun_ty, &ty))? {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Type mismatch. Expected `{}`, found `{}`",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&fun_ty).to_string_normalize(),
                        ),
                        &[&ei.source],
                    ));
                }
                assert!(arg.name.is_local());
                self.scope.push(&arg.name.name, Scheme::from_type(arg_ty));
                let body = self.unify_type_of_expr(body, body_ty)?;
                self.scope.pop(&arg.name.name);
                Ok(ei.set_lam_body(body))
            }
            Expr::Let(pat, val, body) => {
                pat.validate(&self.type_env)?;
                let (pat, var_ty) = pat.get_typed(self)?;
                let val = self.unify_type_of_expr(val, pat.info.inferred_ty.as_ref().unwrap().clone())?;
                for (var_name, var_ty) in &var_ty {
                    assert!(var_name.is_local());
                    self.scope.push(&var_name.name, Scheme::from_type(var_ty.clone()));
                }
                let body = self.unify_type_of_expr(body, ty)?;
                for (name, _) in var_ty {
                    self.scope.pop(&name.name);
                }
                Ok(ei.set_let_pat(pat).set_let_bound(val).set_let_value(body))
            }
            Expr::Match(cond, pat_vals) => {
                // First, perform type inference for the condition.
                let cond_ty = self.new_tyvar(None, cond.source.clone());
                let cond = self.unify_type_of_expr(cond, cond_ty.clone())?;

                let mut cond_tc_info: Option<(Arc<TyCon>, TyConInfo)> = None;

                // Validate each cases.
                let mut new_pat_vals = vec![];
                for (pat, val) in pat_vals {
                    pat.validate(&self.type_env)?;
                    let pat = if pat.is_union() {
                        // Check if the union variant name is valid.

                        // Find the type constructor of the union variant.
                        if cond_tc_info.is_none() {
                            let cond_ty = self.substitute_type(&cond_ty);
                            let cond_ty = self.reduce_type_by_equality(cond_ty)?;
                            let cond_tycon = cond_ty.toplevel_tycon();
                            if cond_tycon.is_none() {
                                return Err(Errors::from_msg_srcs(
                                    "The condition of `match` is unknown at this point.\
                                        Please give a type annotation to the condition.".to_string(),
                                    &[&cond.source],
                                ));
                            }
                            let cond_tycon = cond_tycon.unwrap();
                            let cond_ti = self.type_env.tycons.get(&cond_tycon).unwrap().clone();
                            if cond_ti.variant != TyConVariant::Union {
                                return Err(Errors::from_msg_srcs(
                                    format!("The condition of `match` has type `{}` which is not union, but matched on a variant pattern `{}`.", cond_ty.to_string_normalize(), pat.pattern.to_string()),
                                    &[&cond.source, &pat.info.source],
                                ));
                            }
                            cond_tc_info = Some((cond_tycon.clone(), cond_ti.clone()));
                        }
                        let (cond_tycon, cond_ti) = cond_tc_info.as_ref().unwrap();
                        pat.validate_variant_name(cond_tycon, cond_ti)?
                    } else {
                        pat.clone()
                    };

                    // Check if the type of the pattern matches the type of the condition.
                    let (pat, var_ty) = pat.get_typed(self)?;
                    let pat_ty = pat.info.inferred_ty.as_ref().unwrap().clone();
                    if let Err(_) = UnifOrOtherErr::extract_others(self.unify(&cond_ty, &pat_ty))? {
                        let type_strs = TypeNode::to_string_normalize_many(&vec![cond_ty.clone(), pat_ty.clone()]);
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "Type mismatch. Expected `{}`, found `{}`.",
                                &type_strs[0],
                                &type_strs[1],
                            ),
                            &[&ei.source],
                        ));
                    }

                    // Check if the type of the value matches the whole type.
                    for (var_name, var_ty) in &var_ty {
                        assert!(var_name.is_local());
                        self.scope.push(&var_name.name, Scheme::from_type(var_ty.clone()));
                    }
                    let val = self.unify_type_of_expr(val, ty.clone())?;
                    for (var_name, _) in var_ty {
                        self.scope.pop(&var_name.name);
                    }
                    new_pat_vals.push((pat, val));
                }

                // If there is at least one union pattern, check if the match cases are exhaustive.
                if let Some((cond_tycon, cond_ti)) = cond_tc_info {
                    let pats = new_pat_vals.iter().map(|(pat, _)| pat.clone());
                    Pattern::validate_match_cases_exhaustiveness(&cond_tycon, &cond_ti, &ei.source, pats)?;
                }

                Ok(ei.set_match_cond(cond).set_match_pat_vals(new_pat_vals))
            },
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.unify_type_of_expr(cond, make_bool_ty())?;
                let then_expr = self.unify_type_of_expr(then_expr, ty.clone())?;
                let else_expr = self.unify_type_of_expr(else_expr, ty)?;
                Ok(ei.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr))
            }
            Expr::TyAnno(e, anno_ty) => {
                let anno_ty = self.validate_type_annotation(&anno_ty)?;
                if let Err(_) = UnifOrOtherErr::extract_others(self.unify(&ty, &anno_ty))? {
                    let ty_strs = TypeNode::to_string_normalize_many(&vec![self.substitute_type(&ty), self.substitute_type(&anno_ty)]);
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &ty_strs[0],
                            &ty_strs[1],
                        ),
                        &[&ei.source],
                    ));
                }
                let e = self.unify_type_of_expr(e, ty.clone())?;
                Ok(ei.set_tyanno_expr(e))
            }
            Expr::MakeStruct(tc, fields) => {
                // `tc` should be a struct.
                let tycon_info = self.type_env.tycons.get(&tc);
                if tycon_info.is_none() {
                    return Err(Errors::from_msg_srcs(
                        format!("Unknown type name `{}`.", tc.to_string()),
                        &[&ei.source],
                    ));
                }
                let tycon_info = tycon_info.unwrap();
                if tycon_info.variant != TyConVariant::Struct {
                    return Err(Errors::from_msg_srcs(
                        format!("Type `{}` is not a struct.", tc.to_string()),
                        &[&ei.source],
                    ));
                }

                // Get list of field names.
                let ti = self.type_env.tycons.get(tc);
                if ti.is_none() {
                    return Err(Errors::from_msg_srcs(
                        format!("Unknown type name `{}`.", tc.to_string()),
                        &[&ei.source],
                    ));
                }
                let ti = ti.unwrap();
                let field_names = ti.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();

                // Validate fields.
                let field_names_in_struct_defn: Set<Name> =
                    Set::from_iter(field_names.iter().cloned());
                let field_names_in_expression: Set<Name> =
                    Set::from_iter(fields.iter().map(|(name, _)| name.clone()));
                for f in &field_names_in_struct_defn {
                    if !field_names_in_expression.contains(f) {
                        return Err(Errors::from_msg_srcs(
                            format!("Missing field `{}` of struct `{}`.", f, tc.to_string()),
                            &[&ei.source],
                        ));
                    }
                }
                for f in &field_names_in_expression {
                    if !field_names_in_struct_defn.contains(f) {
                        return Err(Errors::from_msg_srcs(
                            format!("Unknown field `{}` for struct `{}`.", f, tc.to_string()),
                            &[&ei.source],
                        ));
                    }
                }

                // Get field types.
                let struct_ty = tc.get_struct_union_value_type(self);
                if let Err(_) = UnifOrOtherErr::extract_others(self.unify(&struct_ty, &ty))? {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&struct_ty).to_string_normalize(),
                        ),
                        &[&ei.source],
                    ));
                }
                let field_tys = struct_ty.field_types(&self.type_env);
                assert_eq!(field_tys.len(), fields.len());

                // Reorder fields as ordering of fields in struct definition.
                let fields: Map<Name, Arc<ExprNode>> =
                    Map::from_iter(fields.iter().cloned());
                let mut fields = field_names
                    .iter()
                    .map(|name| (name.clone(), fields[name].clone()))
                    .collect::<Vec<_>>();

                for (field_ty, (_, field_expr)) in field_tys.iter().zip(fields.iter_mut()) {
                    *field_expr = self.unify_type_of_expr(field_expr, field_ty.clone())?;
                }
                Ok(ei.set_make_struct_fields(fields))
            }
            Expr::ArrayLit(elems) => {
                // Prepare type of element.
                let elem_src = if elems.len() > 0 {
                    elems[0].source.clone()
                } else {
                    ei.source.as_ref().map(|src|src.to_next_head_character())
                };
                let elem_ty = self.new_tyvar(None, elem_src);
                let array_ty = type_tyapp(make_array_ty(), elem_ty.clone());
                if let Err(_) = UnifOrOtherErr::extract_others(self.unify(&array_ty, &ty))? {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Type mismatch. Expected `{}`, found an array.",
                            &self.substitute_type(&ty).to_string_normalize(),
                        ),
                        &[&ei.source],
                    ));
                }
                let mut ei = ei.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.unify_type_of_expr(e, elem_ty.clone())?;
                    ei = ei.set_array_lit_elem(e, i);
                }
                Ok(ei)
            }
            Expr::FFICall(_, ret_ty, param_tys, args, is_io) => {
                let ret_ty = type_tycon(ret_ty);
                let ret_ty = if *is_io {
                    make_tuple_ty(vec![make_iostate_ty(), ret_ty])
                } else {
                    ret_ty
                };
                if let Err(_) = UnifOrOtherErr::extract_others(self.unify(&ty, &ret_ty))? {
                    return Err(Errors::from_msg_srcs(
                        format!(
                            "Type mismatch. Expected `{}`, found `{}`.",
                            &self.substitute_type(&ty).to_string_normalize(),
                            &self.substitute_type(&ret_ty).to_string_normalize()
                        ),
                        &[&ei.source],
                    ));
                }
                let mut param_tys = param_tys
                    .iter()
                    .map(|tc| type_tycon(tc))
                    .collect::<Vec<_>>();
                if *is_io {
                    param_tys.push(make_iostate_ty());
                }
                let mut ei = ei.clone();
                for (i, e) in args.iter().enumerate() {
                    assert!(i < param_tys.len());
                    let e = self.unify_type_of_expr(e, param_tys[i].clone())?;
                    ei = ei.set_ffi_call_arg(e, i);
                }
                Ok(ei)
            }
        }
    }

    // Check if expr has type scm.
    // Returns given AST set with inferred type.
    pub fn check_type(&mut self, expr: Arc<ExprNode>, expect_scm: Arc<Scheme>) -> Result<Arc<ExprNode>, Errors> {
        // This function should be called when TypeCheckContext is "fresh".
        assert!(self.substitution.is_empty());
        assert!(self.predicates.is_empty());
        assert!(self.equalities.is_empty());

        let specified_ty = UnifOrOtherErr::extract_others(self.instantiate_scheme(&expect_scm, ConstraintInstantiationMode::Assume))?;
        if let Err(e) = specified_ty {
            return Err(Errors::from_msg_srcs(
                format!(
                    "`{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    e.to_constraint_string()
                ), &[&expr.source]
            ));
        }
        let specified_ty = specified_ty.ok().unwrap();
        let expr = self.unify_type_of_expr(&expr, specified_ty.clone())?;
        let reduction_res = UnifOrOtherErr::extract_others(self.reduce_predicates())?;
        if let Err(e) = reduction_res {
            return Err(Errors::from_msg_srcs(
                format!(
                    "`{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    e.to_constraint_string()
                ), &[&expr.source]
            ));
        }
        if self.predicates.len() > 0 {
            let pred = &self.predicates[0];
            return Err(Errors::from_msg_srcs(
                format!(
                    "Condition `{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    pred.to_string()
                ), &[&expr.source]
            ));
        }
        if self.equalities.len() > 0 {
            let eq = &self.equalities[0];
            return Err(Errors::from_msg_srcs(
                format!(
                    "Condition `{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    eq.to_string()
                ), &[&expr.source]
            ));
        }

        Ok(expr)
    }

    fn add_substitution(&mut self, subst: &Substitution) -> Result<(), UnifOrOtherErr> {
        self.substitution.add_substitution(subst);
        let eqs = std::mem::replace(&mut self.equalities, vec![]);
        for eq in eqs {
            self.add_equality(eq)?;
        }
        Ok(())
    }

    fn add_equality(&mut self, mut eq: Equality) -> Result<(), UnifOrOtherErr> {
        // We add only equalities that are not trivial, and cannot be simplified further. 
        // If the equation can be simplified in some way, then unify lhs and rhs of the equation, instead of adding it to `equalities`.
        // `unify` may be recursively call this function again.
        // To avoid infinite loop, we use `unify` only when the equality can be simplified.

        let eq_org = eq.to_string();
        let lhs_org = eq.lhs().to_string();
        let rhs_org = eq.value.to_string();

        // If the equality can be simplified by substitution, call unify.
        self.substitute_equality(&mut eq);
        if eq.to_string() != eq_org {
            self.unify(&eq.lhs(), &eq.value)?;
            return Ok(());
        }

        // If the lhs of the equality is reducible, call unify.
        let red_lhs = self.reduce_type_by_equality(eq.lhs())?;
        if red_lhs.to_string() != lhs_org {
            self.unify(&red_lhs, &eq.value)?;
            return Ok(());
        }
        
        // If the rhs of the equality is reducible, call unify.
        eq.value = self.reduce_type_by_equality(eq.value.clone())?;
        if eq.value.to_string() != rhs_org {
            self.unify(&eq.lhs(), &eq.value)?;
            return Ok(());
        }
        
        // Avoid adding trivial equality.
        if eq.lhs().to_string() == eq.value.to_string() {
            return Ok(());
        }
        
        self.equalities.push(eq);
        Ok(())
    }

    // Reduce a type by replacing associated type to its value.
    fn reduce_type_by_equality(&mut self, ty: Arc<TypeNode>) -> Result<Arc<TypeNode>, Errors> {
        Ok(match &ty.ty {
            Type::TyVar(_) => ty,
            Type::TyCon(_) => ty,
            Type::TyApp(tyfun, tyarg) => {
                let tyfun = self.reduce_type_by_equality(tyfun.clone())?;
                let tyarg = self.reduce_type_by_equality(tyarg.clone())?;
                ty.set_tyapp_fun(tyfun).set_tyapp_arg(tyarg)
            },
            Type::AssocTy(assoc_ty, args) => {
                // Reduce each arguments. 
                let args = collect_results(args.iter().map(|arg| self.reduce_type_by_equality(arg.clone())))?;
                let ty = ty.set_assocty_args(args);

                // Try matching to assumed equality.
                for assumed_eq in &self.assumed_eqs.get(assoc_ty).unwrap().clone() {
                    // Instantiate `assumed_eq`.
                    let mut subst = Substitution::default();
                    for tv in &assumed_eq.gen_vars {
                        let new_tv = self.new_tyvar(Some(tv.kind.clone()), assumed_eq.get_first_tv_source(&tv.name));
                        subst.add_substitution(&Substitution::single(&tv.name, new_tv));
                    }
                    let mut equality = assumed_eq.equality.clone();
                    subst.substitute_equality(&mut equality);

                    // Try to match lhs of `equality` to `ty`.
                    let subst: Option<Substitution> = Substitution::matching(&equality.lhs(), &ty, &self.fixed_tyvars, &self.kind_env)?;
                    if subst.is_none() {
                        continue;
                    }
                    let subst: Substitution = subst.unwrap();
                    let rhs = subst.substitute_type(&equality.value);
                    return self.reduce_type_by_equality(rhs);
                }
                ty
            },
        })
    }

    // Unify two types.
    pub fn unify(
        &mut self,
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
    ) -> Result<(), UnifOrOtherErr> {
        let ty1 = self.substitute_type(ty1);
        let mut ty1 = self.reduce_type_by_equality(ty1)?;
        let ty2 = self.substitute_type(ty2);
        let mut ty2 = self.reduce_type_by_equality(ty2)?;

        if ty1.to_string() == ty2.to_string() {
            return Ok(());
        }

        // Case: Either is a type variable.
        for _ in 0..2 {
            match &ty1.ty {
                Type::TyVar(var1) => {
                    if !self.fixed_tyvars.iter().any(|fixed_tv| fixed_tv.name == var1.name) {
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
                return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()).into());
            },
            Type::AssocTy(_, _) => unreachable!(),
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1 == tc2 {
                        return Ok(());
                    } else {
                        return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()).into());
                    }
                }
                _ => {
                    return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()).into());
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
                    return Err(UnificationErr::Disjoint(ty1.clone(), ty2.clone()).into());
                }
            },
        }
    }

    // Subroutine of unify().
    fn unify_tyvar(
        &mut self,
        tyvar1: Arc<TyVar>,
        ty2: Arc<TypeNode>,
    ) -> Result<(), UnifOrOtherErr> {
        assert!(!self.fixed_tyvars.iter().any(|fixed_tv| fixed_tv.name == tyvar1.name));

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
            return Err(UnificationErr::Disjoint(type_from_tyvar(tyvar1), ty2).into());
        }
        if tyvar1.kind != ty2.kind(&self.kind_env)? {
            return Err(UnificationErr::Disjoint(type_from_tyvar(tyvar1), ty2).into());
        }
        self.add_substitution(&Substitution::single(&tyvar1.name, ty2.clone()))?;
        Ok(())
    }

    // Reduce predicates as long as possible.
    // If predicates are unsatisfiable, return Err.
    fn reduce_predicates(&mut self) -> Result<(), UnifOrOtherErr> {
        let preds = std::mem::replace(&mut self.predicates, vec![]);
        let mut already_added : Set<String> = Set::default();
        for pred in preds {
            self.add_predicate_reducing(pred, &mut already_added)?;
        }
        Ok(())
    }

    // Add a predicate after reducing it.
    fn add_predicate_reducing(&mut self, pred : Predicate, already_added: &mut Set<Name>) -> Result<(), UnifOrOtherErr> {
        for pred in pred.resolve_trait_aliases(&self.trait_env)? {
            self.add_predicate_reducing_noalias(pred, already_added)?;
        }
        Ok(())
    }

    // Add a predicate after reducing it.
    // Trait in `pred` should not be a trait alias.
    fn add_predicate_reducing_noalias(&mut self, mut pred : Predicate, already_added: &mut Set<Name>) -> Result<(), UnifOrOtherErr> {
        self.substitute_predicate(&mut pred);
        pred.ty = self.reduce_type_by_equality(pred.ty)?;
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
                let new_tyvar = self.new_tyvar(Some(tv.kind.clone()), qual_pred_scm.get_first_tv_source(&tv.name));
                subst.add_substitution(&Substitution::single(&tv.name, new_tyvar));
            }
            let mut qual_pred = qual_pred_scm.qual_pred.clone();
            subst.substitute_qualpred(&mut qual_pred);

            // Try to match head of `qual_pred` to `pred`.
            if let Some(subst) = Substitution::matching(&qual_pred.predicate.ty, &pred.ty, &self.fixed_tyvars, &self.kind_env)? {
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
                if UnifOrOtherErr::extract_others(tc.unify(&qual_pred.predicate.ty, &pred.ty))?.is_ok() {
                    unifiable = true;
                }
            }
        }
        if !unifiable {
            return Err(UnificationErr::Unsatisfiable(pred).into());
        }
        self.predicates.push(pred);
        return Ok(());
    }

    pub fn finish_inferred_types(&mut self, expr: Arc<ExprNode>) -> Result<Arc<ExprNode>, Errors> {
        let ty = self.substitute_type(expr.ty.as_ref().unwrap());
        let ty = self.reduce_type_by_equality(ty)?;
        let expr = expr.set_inferred_type(ty);
        Ok(match &*expr.expr {
            Expr::Var(_) => expr,
            Expr::LLVM(_) => expr,
            Expr::App(fun, args) => {
                let args = collect_results(args.iter().map(|arg| self.finish_inferred_types(arg.clone())))?;
                let fun = self.finish_inferred_types(fun.clone())?;
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_args, body) => {
                let body = self.finish_inferred_types(body.clone())?;
                expr.set_lam_body(body)
            }
            Expr::Let(_pat, val, body) => {
                let val = self.finish_inferred_types(val.clone())?;
                let body = self.finish_inferred_types(body.clone())?;
                expr.set_let_bound(val).set_let_value(body)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.finish_inferred_types(cond.clone())?;
                let then_expr = self.finish_inferred_types(then_expr.clone())?;
                let else_expr = self.finish_inferred_types(else_expr.clone())?;
                expr.set_if_cond(cond).set_if_then(then_expr).set_if_else(else_expr)
            }
            Expr::Match(cond, pat_vals) => {
                let cond = self.finish_inferred_types(cond.clone())?;
                let mut new_pat_vals = vec![];
                for (pat, val) in pat_vals {
                    let val = self.finish_inferred_types(val.clone())?;
                    new_pat_vals.push((pat.clone(), val));
                }
                expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
            },
            Expr::TyAnno(e, _) => expr.set_tyanno_expr(self.finish_inferred_types(e.clone())?),
            Expr::MakeStruct(_tc, fields) => {
                let mut fields_res = vec![];
                for (name, e) in fields {
                    let e = self.finish_inferred_types(e.clone())?;
                    fields_res.push((name.clone(), e));
                }
                expr.set_make_struct_fields(fields_res)
            }
            Expr::ArrayLit(elems) => {
                let elems = collect_results(elems.iter().map(|e| self.finish_inferred_types(e.clone())))?;
                expr.set_array_lit_elems(elems)
            }
            Expr::FFICall(_, _, _, args,_) => {
                let args = collect_results(args.iter().map(|arg| self.finish_inferred_types(arg.clone())))?;
                expr.set_ffi_call_args(args)
            }
        })
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

pub enum UnifOrOtherErr {
    UnifErr(UnificationErr),
    Others(Errors),
}

impl UnifOrOtherErr {
    pub fn extract_others<T>(res: Result<T, UnifOrOtherErr>) -> Result<Result<T, UnificationErr>, Errors> {
        match res {
            Ok(v) => Ok(Ok(v)),
            Err(UnifOrOtherErr::UnifErr(ue)) => Ok(Err(ue)),
            Err(UnifOrOtherErr::Others(es)) => Err(es),
        }
    }
}

impl From<Errors> for UnifOrOtherErr {
    fn from(e: Errors) -> Self {
        UnifOrOtherErr::Others(e)
    }
}

impl From<UnificationErr> for UnifOrOtherErr {
    fn from(e: UnificationErr) -> Self {
        UnifOrOtherErr::UnifErr(e)
    }
}