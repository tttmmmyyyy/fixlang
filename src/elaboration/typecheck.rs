use std::sync::Arc;
use crate::{
    ast::{
        equality::{Equality, EqualityScheme},
        expr::{AppSourceCodeOrderType, Expr, ExprNode},
        import::ImportStatement,
        kind_scope::KindEnv,
        name::{FullName, Name, NameSpace},
        pattern::{Pattern, PatternNode},
        predicate::Predicate,
        program::{ModuleInfo, TypeEnv},
        types::OpaqueTyConResolution,
        qual_pred::{QualPred, QualPredScheme},
        qual_type::QualType,
        traits::{TraitEnv, TraitId},
        types::{
            kind_star, make_tyvar, type_from_tyvar, type_fun, type_tyapp,
            type_tycon, AssocType, Kind, Scheme, TyCon, TyConInfo, TyConVariant, TyVar, Type,
            TypeNode,
        },
    },
    constants::{
        ERR_AMBIGUOUS_NAME, ERR_MISSING_STRUCT_FIELD, ERR_NO_VALUE_MATCH, ERR_UNKNOWN_NAME,
        WRAP_OPAQUE_TYVAR_PREFIX,
    },
    error::{Error, Errors},
    elaboration::name_resolution::NameResolutionContext,
    fixstd::builtin::{make_array_ty, make_bool_ty, make_iostate_ty, make_tuple_ty},
    parse::sourcefile::Span,
};
use crate::ast::import;
use crate::misc::{collect_results, insert_to_map_vec, make_map, Map, Set};
use serde::{Deserialize, Serialize};
use super::check_holes;
use super::typecheckcache::TypeCheckCache;

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
        insert_to_map_vec(&mut self.local, name, v);
    }

    // Pop a local value.
    pub fn pop(self: &mut Self, name: &Name) {
        self.local.get_mut(name).unwrap().pop();
    }

    // Check if a local value exists.
    pub fn has_value(&self, name: &Name) -> bool {
        self.local.contains_key(name) && !self.local[name].is_empty()
    }

    // Get a local value.
    pub fn get_local(&self, name: &Name) -> Option<T> {
        if self.local.contains_key(name) && !self.local[name].is_empty() {
            Some(self.local[name].last().unwrap().clone())
        } else {
            None
        }
    }

    // Get a set of local names.
    #[allow(dead_code)]
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
    //
    // If `name` is an absolute name, the returned (at most one) candidates will also be set as absolute names.
    fn overloaded_candidates(
        &self,
        name: &FullName,
        import_stmts: &[ImportStatement],
    ) -> Vec<(NameSpace, T)> {
        if name.is_local() && self.has_value(&name.name) {
            vec![(
                NameSpace::local(),
                self.local[&name.name].last().unwrap().clone(),
            )]
        } else {
            self.global
                .iter()
                .filter_map(|(full_name, v)| {
                    if name == full_name && name.is_absolute() {
                        // Inherit the absolute property.
                        let mut full_name = full_name.clone();
                        full_name.set_absolute();
                        return Some((full_name.namespace.clone(), v.clone()));
                    }
                    if name.is_suffix_of(full_name)
                        && import::is_accessible(import_stmts, full_name)
                    {
                        return Some((full_name.namespace.clone(), v.clone()));
                    }
                    return None;
                })
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

    // Compose substitution.
    pub fn compose(&mut self, following: &Self) {
        for (_var, ty) in self.data.iter_mut() {
            let new_ty = following.substitute_type(&ty);
            *ty = new_ty;
        }
        for (var, ty) in &following.data {
            assert!(!self.data.contains_key(var));
            self.data.insert(var.to_string(), ty.clone());
        }
    }

    // Merge substitution.
    // Returns true when merge succeeds.
    pub fn merge(&mut self, other: &Self) -> bool {
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
            Type::AssocTy(_, args) => {
                ty.set_assocty_args(args.iter().map(|arg| self.substitute_type(arg)).collect())
            }
        }
    }

    pub fn substitute_unification_error(&self, e: &mut UnificationErr) {
        match e {
            UnificationErr::Unsatisfiable(predicate) => {
                self.substitute_predicate(predicate);
            }
            UnificationErr::Disjoint(type_node, type_node1) => {
                *type_node = self.substitute_type(type_node);
                *type_node1 = self.substitute_type(type_node1);
            }
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
            scm.kind_signs.clone(),
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

    pub fn substitute_qualpred(&self, qual_pred: &mut QualPred) {
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
        kind_env: &KindEnv,
    ) -> Result<Option<Self>, Errors> {
        Self::matching_internal(ty1, ty2, fixed_tyvars, Some(kind_env))
    }

    pub fn matching_no_kind_check(
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
        fixed_tyvars: &[Arc<TyVar>],
    ) -> Option<Self> {
        // With kind_env=None, matching_internal never returns Err.
        match Self::matching_internal(ty1, ty2, fixed_tyvars, None) {
            Ok(result) => result,
            Err(_) => unreachable!("matching_internal without kind_env should not fail"),
        }
    }

    fn matching_internal(
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
        fixed_tyvars: &[Arc<TyVar>],
        kind_env: Option<&KindEnv>,
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
                if let Some(kind_env) = kind_env {
                    if ty1.kind(kind_env)? != ty2.kind(kind_env)? {
                        return Ok(None);
                    }
                }
                if fixed_tyvars.iter().any(|tv| tv.name == v1.name) {
                    if ty1.to_string() == ty2.to_string() {
                        return Ok(Some(Self::default()));
                    } else {
                        return Ok(None);
                    }
                }
                return Ok(Some(Self::single(&v1.name, ty2.clone())));
            }
            Type::TyCon(tc1) => match &ty2.ty {
                Type::TyCon(tc2) => {
                    if tc1 == tc2 {
                        return Ok(Some(Self::default()));
                    } else {
                        return Ok(None);
                    }
                }
                _ => return Ok(None),
            },
            Type::TyApp(fun1, arg1) => match &ty2.ty {
                Type::TyApp(fun2, arg2) => {
                    let mut ret = Self::default();
                    match Self::matching_internal(fun1, fun2, fixed_tyvars, kind_env)? {
                        Some(s) => {
                            if !ret.merge(&s) {
                                return Ok(None);
                            }
                        }
                        None => return Ok(None),
                    }
                    match Self::matching_internal(arg1, arg2, fixed_tyvars, kind_env)? {
                        Some(s) => {
                            if !ret.merge(&s) {
                                return Ok(None);
                            }
                        }
                        None => return Ok(None),
                    }
                    return Ok(Some(ret));
                }
                _ => return Ok(None),
            },
            Type::AssocTy(assoc_ty1, args1) => match &ty2.ty {
                Type::AssocTy(assoc_ty2, args2) => {
                    if assoc_ty1 != assoc_ty2 {
                        return Ok(None);
                    }
                    let mut ret = Self::default();
                    for i in 0..args1.len() {
                        match Self::matching_internal(&args1[i], &args2[i], fixed_tyvars, kind_env)? {
                            Some(s) => {
                                if !ret.merge(&s) {
                                    return Ok(None);
                                }
                            }
                            None => return Ok(None),
                        }
                    }
                    return Ok(Some(ret));
                }
                _ => return Ok(None),
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
    // The map from type variables to the source location of an expression whose type is the type variable.
    // This is used for generating error messages.
    pub tyvar_expr: Map<String, Span>,
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
    pub current_module: Option<ModuleInfo>,
    // Names that should be imported in the current module.
    pub import_required: Vec<FullName>,
    // Equalities assumed.
    pub assumed_eqs: Map<AssocType, Vec<EqualityScheme>>,
    // Predicates assumed.
    pub assumed_preds: Map<TraitId, Vec<QualPredScheme>>,
    // Fixed type variables.
    // In unification, these type variables are not allowed to be replaced to another type.
    // NOTE: We use `Vec` instead of `Set` because the expected size is small.
    pub fixed_tyvars: Vec<Arc<TyVar>>,
    // Locally assumed equalities.
    // For example, when type checking `extend : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e] c1 -> c2 -> c2`, we assume `Elem c1 = e` and `Elem c2 = e` locally.
    pub local_assumed_eqs: Vec<Equality>,
    // Type check cache.
    pub cache: Arc<dyn TypeCheckCache + Sync + Send>,
    // Number of worker threads.
    pub num_worker_threads: usize,
    // Records which fresh type variables were assigned to opaque-type gen_vars when instantiating #wrap_opaque functions.
    // Key: the gen_var name (e.g., "#Std::repeat::?it"), Value: the fresh TyVar generated for it.
    // After type-checking, these are resolved via substitution to find the concrete types.
    pub opaque_instantiations: Map<Name, Arc<TyVar>>,
    /// When true, errors raised from elaborating a sub-expression are
    /// swallowed: that sub-expression is replaced by a placeholder
    /// annotated with the expected type, and elaboration continues on
    /// its siblings, so types can still be inferred around an
    /// unrelated type error elsewhere in the body.
    pub error_tolerant: bool,
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
        println!("local_assumed_eqs size = {}", self.local_assumed_eqs.len());
        println!("import_required size = {}", self.import_required.len());
    }

    /// Builds a fresh `TypeCheckContext` seeded with the given
    /// trait/type environment and worker pool size.
    pub fn new(
        trait_env: TraitEnv,
        type_env: TypeEnv,
        kind_env: KindEnv,
        import_statements: Map<Name, Vec<ImportStatement>>,
        cache: Arc<dyn TypeCheckCache + Sync + Send>,
        num_worker_threads: usize,
        error_tolerant: bool,
    ) -> Self {
        let assumed_preds = trait_env.qualified_predicates();
        let assumed_eqs = trait_env.type_equalities();
        Self {
            tyvar_id: Default::default(),
            tyvar_expr: Default::default(),
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
            local_assumed_eqs: vec![],
            import_required: vec![],
            cache,
            num_worker_threads,
            opaque_instantiations: Map::default(),
            error_tolerant,
        }
    }

    // Register the source location of an expression whose type is a type variable.
    pub fn add_tyvar_source(&mut self, tyvar_name: Name, source: Option<Span>) {
        if let Some(source) = source {
            self.tyvar_expr.insert(tyvar_name, source);
        }
    }

    /// Fresh type variable wrapped as `TypeNode`, with `src` registered
    /// as its source span. Used by the `error_tolerant` fallback paths
    /// in `unify_type_of_expr_inner` to give each child of a failed
    /// elaboration an unconstrained-but-typed placeholder so the child
    /// can still be elaborated against `self`.
    pub fn fresh_ty_with_src(&mut self, src: &Option<Span>) -> Arc<TypeNode> {
        let tv = self.new_tyvar_star();
        self.add_tyvar_source(tv.name.clone(), src.clone());
        type_from_tyvar(tv)
    }

    /// In `error_tolerant` mode, swallow a soft `Err` so the caller can
    /// substitute a fallback value. Returns:
    ///
    /// - `Ok(Some(v))` — the original success;
    /// - `Ok(None)` — strict-mode behaviour would have returned `Err`,
    ///   but tolerant mode chooses to continue;
    /// - `Err(e)` — strict mode, re-raised for the caller's `?`.
    ///
    /// Use with `?.unwrap_or_else(|| <fallback>)` to keep the call site
    /// compact. The receiver is `&self` (only reads `error_tolerant`),
    /// so a `&mut self` fallback closure can still borrow `self` after
    /// the call.
    pub fn tolerate<T>(&self, res: Result<T, Errors>) -> Result<Option<T>, Errors> {
        match res {
            Ok(v) => Ok(Some(v)),
            Err(_) if self.error_tolerant => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Run `validate_pattern` then `get_typed` on `pat`, returning the
    /// typed pattern and its variable bindings. Used by both `Let`
    /// and `Match` arms — i.e. every site where a pattern introduces
    /// new binders into the surrounding scope — to combine the two
    /// fallible steps into one `Result` so the call site can choose
    /// between propagating the error (strict mode) and substituting
    /// a fresh-tyvar pattern (`error_tolerant` mode).
    fn elaborate_pattern_binding(
        &mut self,
        pat: &Arc<PatternNode>,
    ) -> Result<(Arc<PatternNode>, Map<FullName, Arc<TypeNode>>), Errors> {
        self.validate_pattern(pat)?;
        pat.get_typed(self)
    }

    /// Resolve the matched value's TyCon for a `Match` arm with a
    /// union pattern. Returns the `(TyCon, TyConInfo)` pair required
    /// by `Pattern::validate_variant_name`. Fails if `cond_ty` isn't
    /// resolvable to a concrete tycon yet, or if it resolves to a
    /// non-union type.
    fn resolve_match_cond_tycon(
        &mut self,
        cond: &Arc<ExprNode>,
        cond_ty: &Arc<TypeNode>,
        pat: &Arc<PatternNode>,
    ) -> Result<(Arc<TyCon>, TyConInfo), Errors> {
        let cond_ty = self.substitute_and_reduce_type(cond_ty)?;
        let Some(cond_tycon) = cond_ty.toplevel_tycon() else {
            return Err(Errors::from_msg_srcs(
                "The type of the matched value must be known at this point. Add type annotation to it."
                    .to_string(),
                &[&cond.source],
            ));
        };
        let cond_ti = self.type_env.tycons.get(&cond_tycon).unwrap().clone();
        if cond_ti.variant != TyConVariant::Union {
            return Err(Errors::from_msg_srcs(
                format!(
                    "The matched value has non-union type `{}`, but it is matched on a variant pattern `{}`.",
                    cond_ty.to_string_normalize(),
                    pat.pattern.to_string()
                ),
                &[&cond.source, &pat.info.source],
            ));
        }
        Ok((cond_tycon, cond_ti))
    }

    // Set the source locations of two unified type variables to the same one.
    pub fn unify_tyvar_source(&mut self, tv1: Name, tv2: Name) {
        let mut src = None;
        if let Some(tv1_src) = self.tyvar_expr.get(&tv1) {
            src = Some(tv1_src.clone());
        }
        if let Some(tv2_src) = self.tyvar_expr.get(&tv2) {
            src = Some(tv2_src.clone());
        }
        self.add_tyvar_source(tv1, src.clone());
        self.add_tyvar_source(tv2, src);
    }

    // Get modules imported by current module.
    pub fn imported_statements(&self) -> &Vec<ImportStatement> {
        self.import_statements
            .get(&self.current_module.as_ref().unwrap().name)
            .unwrap()
    }

    // Create a new type variable.
    pub fn new_tyvar_name(&mut self) -> String {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
        "#a".to_string() + &id.to_string()
    }

    // Create a new type variable.
    pub fn new_tyvar(&mut self, kind: Arc<Kind>) -> Arc<TyVar> {
        let name = self.new_tyvar_name();
        make_tyvar(&name, &kind)
    }

    // Create a new type variable of kind `*`.
    pub fn new_tyvar_star(&mut self) -> Arc<TyVar> {
        self.new_tyvar(kind_star())
    }

    // Create a new type variable by copying information from another type variable.
    pub fn new_tyvar_by(&mut self, tv: &Arc<TyVar>) -> Arc<TyVar> {
        tv.set_name(self.new_tyvar_name())
    }

    // Apply substitution to type.
    pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        self.substitution.substitute_type(ty)
    }

    // Apply substitution and then reduce associated types by equalities.
    pub fn substitute_and_reduce_type(
        &mut self,
        ty: &Arc<TypeNode>,
    ) -> Result<Arc<TypeNode>, Errors> {
        let ty = self.substitute_type(ty);
        self.reduce_type_by_equality(ty)
    }

    // Apply substitution to a predicate.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        self.substitution.substitute_predicate(p)
    }

    // Apply substitution to an equality.
    pub fn substitute_equality(&self, eq: &mut Equality) {
        self.substitution.substitute_equality(eq)
    }

    // Fill in the concrete rhs for opaque type resolutions from the current substitution.
    //
    // Example: if `#Std::repeat::?it` was instantiated to a fresh TyVar that unified to
    // `MapIterator (RangeIterator I64) a`, fills `rhs = Some(MapIterator (RangeIterator I64) a)`
    // into the corresponding `OpaqueTyConResolution` entries.
    pub fn fill_opaque_concrete_types(
        &mut self,
        opaque_types: &mut Map<FullName, Vec<OpaqueTyConResolution>>,
    ) {
        let instantiations = self.opaque_instantiations.clone();
        for (k, v) in instantiations {
            let fullname_str = k.strip_prefix(WRAP_OPAQUE_TYVAR_PREFIX).unwrap();
            let opaque_tycon_name = FullName::parse(fullname_str).unwrap();
            let rhs = self.substitute_and_reduce_type(&type_from_tyvar(v))
                .unwrap_or_else(|_| panic!("failed to reduce opaque type rhs"));
            if let Some(resolutions) = opaque_types.get_mut(&opaque_tycon_name) {
                for resolution in resolutions {
                    assert!(resolution.rhs.is_none(), "opaque type rhs already filled");
                    resolution.rhs = Some(rhs.clone());
                }
            }
        }
    }

    pub fn instantiate_type(&mut self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        let mut sub = Substitution::default();
        for tv in ty.free_vars_vec() {
            let new_tv = self.new_tyvar_by(&tv);
            let merge_ok = sub.merge(&Substitution::single(&tv.name, type_from_tyvar(new_tv)));
            assert!(merge_ok);
        }
        sub.substitute_type(ty)
    }

    // Instantiate a scheme.
    pub fn instantiate_scheme(
        &mut self,
        scheme: &Arc<Scheme>,
        constraint_mode: ConstraintInstantiationMode,
    ) -> Result<Arc<TypeNode>, UnifOrOtherErr> {
        let mut preds = vec![];
        for pred in &scheme.predicates {
            preds.append(&mut pred.resolve_trait_aliases(&self.trait_env.aliases)?);
        }
        let mut eqs = scheme.equalities.clone();
        match constraint_mode {
            ConstraintInstantiationMode::Require => {
                // Instantiate type variables.
                let mut sub = Substitution::default();
                for tv in &scheme.gen_vars {
                    let new_tv = self.new_tyvar_by(tv);
                    let merge_ok =
                        sub.merge(&Substitution::single(&tv.name, type_from_tyvar(new_tv.clone())));
                    assert!(merge_ok);
                    // Record opaque-type gen_vars (prefixed with WRAP_OPAQUE_TYVAR_PREFIX)
                    // so their concrete types can be extracted after type-checking.
                    if tv.name.starts_with(WRAP_OPAQUE_TYVAR_PREFIX) {
                        assert!(
                            !self.opaque_instantiations.contains_key(&tv.name),
                            "Duplicate opaque type variable name: {}",
                            tv.name
                        );
                        self.opaque_instantiations.insert(tv.name.clone(), new_tv.clone());
                    }
                }
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
            }
            ConstraintInstantiationMode::Assume => {
                for tv in &scheme.gen_vars {
                    self.fixed_tyvars.push(tv.clone());
                }
                for pred in preds {
                    let trait_id = pred.trait_id.clone();
                    let qual_pred_scm = QualPredScheme {
                        gen_vars: vec![],
                        qual_pred: QualPred {
                            pred_constraints: vec![],
                            eq_constraints: vec![],
                            kind_constraints: vec![],
                            predicate: pred,
                        },
                    };
                    insert_to_map_vec(&mut self.assumed_preds, &trait_id, qual_pred_scm);
                }
                for eq in eqs {
                    let assoc_ty = eq.assoc_type.clone();
                    let eq_scm = EqualityScheme {
                        gen_vars: vec![],
                        equality: eq.clone(),
                    };
                    insert_to_map_vec(&mut self.assumed_eqs, &assoc_ty, eq_scm);
                    self.local_assumed_eqs.push(eq);
                }
                return Ok(scheme.ty.clone());
            }
        }
    }

    pub fn validate_type_annotation(
        &mut self,
        ty: &Arc<TypeNode>,
    ) -> Result<Arc<TypeNode>, Errors> {
        // All type variables should be fixed by the TypeCheckContext, i.e., appear in the generalized variables of the current scheme.
        for tv in ty.free_vars_vec() {
            if !self
                .fixed_tyvars
                .iter()
                .any(|fixed_tv| fixed_tv.name == tv.name)
            {
                return Err(Errors::from_msg_srcs(
                    format!("Unknown type variable `{}`.", tv.name),
                    &[&ty.get_source()],
                ));
            }
        }

        // Set kinds of type variables in the type annotation.
        let sub = Substitution {
            data: make_map(
                self.fixed_tyvars
                    .iter()
                    .map(|tv| (tv.name.clone(), type_from_tyvar(tv.clone()))),
            ),
        };
        let ty = sub.substitute_type(ty);

        Ok(ty)
    }

    // Perform typechecking.
    // Update type substitution so that `ei` has type `ty`.
    // Returns given AST augmented with inferred information.
    pub fn unify_type_of_expr(
        &mut self,
        ei: &Arc<ExprNode>,
        ty: Arc<TypeNode>,
    ) -> Result<Arc<ExprNode>, Errors> {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            let ty_for_fallback = ty.clone();
            match self.unify_type_of_expr_inner(ei, ty) {
                Ok(e) => Ok(e),
                Err(errs) if self.error_tolerant => {
                    // Swallow the failure and substitute a placeholder
                    // annotated with the expected type, so enclosing
                    // elaboration can keep going on sibling nodes.
                    let _ = errs;
                    Ok(ei.set_type(ty_for_fallback))
                }
                Err(errs) => Err(errs),
            }
        })
    }

    fn unify_type_of_expr_inner(
        &mut self,
        ei: &Arc<ExprNode>,
        ty: Arc<TypeNode>,
    ) -> Result<Arc<ExprNode>, Errors> {
        let ei = ei.set_type(ty.clone());
        match &*ei.expr {
            Expr::Var(var) => {
                let mut candidates = self
                    .scope
                    .overloaded_candidates(&var.name, self.imported_statements());
                if ei.struct_act_func_in_index_syntax {
                    // If this variable `act_{field}` arises from struct index `obj[^field]`, filter candidates to struct accessor functions only.
                    candidates = candidates
                        .into_iter()
                        .filter(|(ns, _)| {
                            let full_name = FullName::new(ns, &var.name.name);
                            self.type_env.is_struct_act(&full_name).is_some()
                        })
                        .collect();
                }
                if candidates.is_empty() {
                    let src = ei
                        .source
                        .clone()
                        .or(self.current_module.as_ref().map(|m| m.source.clone()));
                    let mut err = Error::from_msg_srcs(
                        format!("Unknown name `{}`.", var.name.to_string()),
                        &[&src],
                    );
                    err.code = Some(ERR_UNKNOWN_NAME);
                    err.data = Some(serde_json::Value::String(var.name.to_string()));
                    return Err(Errors::from_err(err));
                }
                let mut candidates_check_res: Vec<
                    Result<
                        (TypeCheckContext, NameSpace),
                        (TypeCheckContext, FullName, Arc<Scheme>, UnificationErr),
                    >,
                > = vec![];
                for (ns, scm) in &candidates {
                    let fullname = FullName::new(ns, &var.name.name);
                    let mut tc = self.clone();
                    let var_ty = UnifOrOtherErr::extract_others(
                        tc.instantiate_scheme(&scm, ConstraintInstantiationMode::Require),
                    )?;
                    if let Err(e) = var_ty {
                        candidates_check_res.push(Err((tc, fullname, scm.clone(), e)))
                    } else if let Err(e) =
                        UnifOrOtherErr::extract_others(tc.unify(&var_ty.ok().unwrap(), &ty))?
                    {
                        candidates_check_res.push(Err((tc, fullname, scm.clone(), e)))
                    } else if let Err(e) = UnifOrOtherErr::extract_others(tc.reduce_predicates())? {
                        candidates_check_res.push(Err((tc, fullname, scm.clone(), e)))
                    } else {
                        candidates_check_res.push(Ok((tc, ns.clone())))
                    }
                }
                let ok_count = candidates_check_res
                    .iter()
                    .filter(|cand| cand.is_ok())
                    .count();
                if ok_count == 0 {
                    let mut extra_srcs = vec![];

                    let err_cnt = candidates_check_res
                        .iter()
                        .filter(|cand| cand.is_err())
                        .count();
                    let expected_type = self.substitute_type(&ty);
                    let msg = if err_cnt == 1 {
                        let (tc, fullname, scm, e) = candidates_check_res
                            .iter()
                            .find_map(|cand| cand.as_ref().err())
                            .unwrap();
                        let scm = tc.substitution.substitute_scheme(scm);
                        let msg = format!(
                            "`{}` of type `{}` does not match the expected type `{}` since `{}` cannot be deduced.",
                            fullname.to_string(),
                            scm.to_string(),
                            expected_type.to_string(),
                            e.to_constraint_string(),
                        );
                        let mut tvs = vec![];
                        scm.free_vars_to_vec(&mut tvs);
                        expected_type.free_vars_to_vec(&mut tvs);
                        e.free_vars_to_vec(&mut tvs);
                        extra_srcs.append(&mut self.create_tyvar_location_messages(&tvs, None));
                        msg
                    } else {
                        let mut msg = format!(
                            "Any of values named `{}` does not match the expected type `{}`.",
                            var.name.to_string(),
                            expected_type.to_string(),
                        );
                        extra_srcs.append(
                            &mut self.create_tyvar_location_messages(
                                &expected_type.free_vars_vec(),
                                None,
                            ),
                        );

                        let mut candidates_errors = vec![];
                        for (tc, fullname, scm, e) in candidates_check_res
                            .iter()
                            .filter_map(|cand| cand.as_ref().err())
                        {
                            let cnt = candidates_errors.len() + 1;
                            let scm = tc.substitution.substitute_scheme(scm);
                            let msg = format!(
                                "- ({}) `{}` of type `{}` does not match since `{}` cannot be deduced.",
                                cnt,
                                fullname.to_string(),
                                scm.to_string(),
                                e.to_constraint_string(),
                            );
                            candidates_errors.push(msg);
                            let mut tvs = vec![];
                            scm.free_vars_to_vec(&mut tvs);
                            e.free_vars_to_vec(&mut tvs);
                            extra_srcs
                                .append(&mut self.create_tyvar_location_messages(&tvs, Some(cnt)));
                        }
                        if candidates_errors.len() > 0 {
                            msg.push_str("\n");
                            msg.push_str(&candidates_errors.join("\n"));
                        }
                        msg
                    };
                    let mut error = Error::from_msg_srcs(msg, &[&ei.source]);
                    error.code = Some(ERR_NO_VALUE_MATCH);
                    error.data = Some(serde_json::Value::String(var.name.to_string()));
                    error.add_srcs(extra_srcs);
                    return Err(Errors::from_err(error));
                } else if ok_count >= 2 {
                    // FullName of candidates.
                    let candidates = candidates_check_res
                        .iter()
                        .filter_map(|cand| cand.as_ref().ok())
                        .map(|(_, ns)| FullName::new(&ns, &var.name.name))
                        .collect::<Vec<_>>();
                    let msg = NameResolutionContext::create_ambiguous_message(
                        &var.name.to_string(),
                        candidates.clone(),
                        true,
                    );
                    let mut err = Error::from_msg_srcs(msg, &[&ei.source]);
                    err.code = Some(ERR_AMBIGUOUS_NAME);
                    err.data = Some(serde_json::Value::Array(
                        candidates
                            .iter()
                            .map(|name| serde_json::Value::String(name.to_string()))
                            .collect(),
                    ));
                    return Err(Errors::from_err(err));
                } else {
                    // candidates.len() == 1
                    let (tc, ns) = candidates_check_res
                        .iter()
                        .find_map(|cand| cand.as_ref().ok())
                        .unwrap();
                    *self = tc.clone();
                    let ei = ei.set_var_namespace(ns.clone());
                    let name = &ei.get_var().name;
                    if name.is_global() && !name.is_absolute() {
                        self.import_required.push(name.clone());
                    }
                    Ok(ei)
                }
            }
            Expr::LLVM(lit) => {
                if let Err(e) = UnifOrOtherErr::extract_others(self.unify(&ty, &lit.generic_ty))? {
                    let err = self.create_type_mismatch_error(&ty, &lit.generic_ty, &e, &ei.source);
                    return Err(Errors::from_err(err));
                }
                Ok(ei.clone())
            }
            Expr::App(fun, args) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_tv = self.new_tyvar_star();
                self.add_tyvar_source(arg_tv.name.clone(), arg.source.clone());
                let arg_ty = type_from_tyvar(arg_tv);
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

                let arg_tv = self.new_tyvar_star();
                self.add_tyvar_source(arg_tv.name.clone(), ei.aux_src.clone());
                let arg_ty = type_from_tyvar(arg_tv);

                let body_tv = self.new_tyvar_star();
                self.add_tyvar_source(body_tv.name.clone(), body.source.clone());
                let body_ty = type_from_tyvar(body_tv);

                let fun_ty = type_fun(arg_ty.clone(), body_ty.clone());
                if let Err(e) = UnifOrOtherErr::extract_others(self.unify(&ty, &fun_ty))? {
                    if !self.error_tolerant {
                        let err = self.create_type_mismatch_error(&ty, &fun_ty, &e, &ei.source);
                        return Err(Errors::from_err(err));
                    }
                    // In error_tolerant mode, continue elaborating the
                    // body even when the lambda's function type cannot
                    // be reconciled with the expected type — `body_ty`
                    // is still a fresh tyvar, so the body gets a
                    // best-effort type.
                }
                assert!(arg.name.is_local());
                self.scope.push(&arg.name.name, Scheme::from_type(arg_ty));
                let body = self.unify_type_of_expr(body, body_ty)?;
                self.scope.pop(&arg.name.name);
                Ok(ei.set_lam_body(body))
            }
            Expr::Let(pat, val, body) => {
                // `validate_pattern` / `get_typed` may fail on a
                // malformed pattern (unknown struct field, duplicate
                // variable, sub-pattern type mismatch). In
                // `error_tolerant` mode we still want to elaborate
                // `val` and `body` so any nested cursor inside them
                // gets a useful type — fall back to a fresh-tyvar
                // pattern with no variable bindings.
                let elab = self.elaborate_pattern_binding(pat);
                let (pat, var_ty) = self.tolerate(elab)?.unwrap_or_else(|| {
                    let pat_ty = self.fresh_ty_with_src(&pat.info.source);
                    (pat.set_type(pat_ty), Map::default())
                });
                let val = self.unify_type_of_expr(val, pat.info.type_.as_ref().unwrap().clone())?;
                for (var_name, var_ty) in &var_ty {
                    assert!(var_name.is_local());
                    self.scope
                        .push(&var_name.name, Scheme::from_type(var_ty.clone()));
                }
                let body = self.unify_type_of_expr(body, ty)?;
                for (name, _) in var_ty {
                    self.scope.pop(&name.name);
                }
                Ok(ei.set_let_pat(pat).set_let_bound(val).set_let_value(body))
            }
            Expr::Match(cond, pat_vals) => {
                // First, perform type inference for the condition.
                let cond_tv = self.new_tyvar_star();
                self.add_tyvar_source(cond_tv.name.clone(), cond.source.clone());
                let cond_ty = type_from_tyvar(cond_tv);
                let cond = self.unify_type_of_expr(cond, cond_ty.clone())?;

                let mut cond_tc_info: Option<(Arc<TyCon>, TyConInfo)> = None;

                // Elaborate each arm. In `error_tolerant` mode every
                // per-arm validation (unreachable-after-otherwise,
                // pattern shape, variant name, pattern/cond type
                // mismatch) is swallowed so the typed `(pat, val)`
                // pair is still appended to `new_pat_vals` — the LSP
                // needs the value's typed subtree to drive dot
                // completion even when the surrounding match is
                // structurally broken.
                let mut new_pat_vals = vec![];
                let mut otherwise: Option<Arc<PatternNode>> = None;
                for (pat, val) in pat_vals {
                    if let Some(otherwise) = &otherwise {
                        if !self.error_tolerant {
                            return Err(Errors::from_msg_srcs(
                                format!(
                                    "Pattern after `{}` is unreachable.",
                                    otherwise.pattern.to_string()
                                ),
                                &[&pat.info.source],
                            ));
                        }
                    }

                    let pat = if pat.is_union() {
                        // Determine the cond's TyCon on the first
                        // union arm so `validate_variant_name` knows
                        // which union to check against. Failure is
                        // tolerated; we fall through to `get_typed`
                        // which can still type sub-patterns from the
                        // variant's signature.
                        let validated = if cond_tc_info.is_none() {
                            self.resolve_match_cond_tycon(&cond, &cond_ty, pat)
                                .and_then(|info| {
                                    cond_tc_info = Some(info);
                                    let (tycon, ti) = cond_tc_info.as_ref().unwrap();
                                    pat.validate_variant_name(tycon, ti)
                                })
                        } else {
                            let (tycon, ti) = cond_tc_info.as_ref().unwrap();
                            pat.validate_variant_name(tycon, ti)
                        };
                        match validated {
                            Ok(p) => p,
                            Err(e) => {
                                if !self.error_tolerant {
                                    return Err(e);
                                }
                                pat.clone()
                            }
                        }
                    } else {
                        // `pat` is not a union pattern, so we can use it as is.
                        otherwise = Some(pat.clone());
                        pat.clone()
                    };

                    // Type the pattern, then unify with cond.
                    // `get_typed` is itself tolerant of sub-pattern
                    // mismatches in `error_tolerant` mode; the only
                    // remaining failure path here is `validate_pattern`
                    // (struct field validity etc.).
                    let elab = self.elaborate_pattern_binding(&pat);
                    let (pat, var_ty) = self.tolerate(elab)?.unwrap_or_else(|| {
                        let pat_ty = self.fresh_ty_with_src(&pat.info.source);
                        (pat.set_type(pat_ty), Map::default())
                    });
                    let pat_ty = pat.info.type_.as_ref().unwrap().clone();
                    if let Err(e) = UnifOrOtherErr::extract_others(self.unify(&cond_ty, &pat_ty))? {
                        if !self.error_tolerant {
                            let err = self.create_type_mismatch_error(
                                &pat_ty,
                                &cond_ty,
                                &e,
                                &pat.info.source,
                            );
                            return Err(Errors::from_err(err));
                        }
                    }

                    // Check if the type of the value matches the whole type.
                    for (var_name, var_ty) in &var_ty {
                        assert!(var_name.is_local());
                        self.scope
                            .push(&var_name.name, Scheme::from_type(var_ty.clone()));
                    }
                    let val = self.unify_type_of_expr(val, ty.clone())?;
                    for (var_name, _) in var_ty {
                        self.scope.pop(&var_name.name);
                    }
                    new_pat_vals.push((pat, val));
                }

                // Build the typed Match before the exhaustiveness
                // check so the typed tree survives even when the
                // check is swallowed in `error_tolerant` mode.
                let typed = ei.set_match_cond(cond).set_match_pat_vals(new_pat_vals);

                // If there is at least one union pattern, check if the match cases are exhaustive.
                if let Some((cond_tycon, cond_ti)) = cond_tc_info {
                    let pats = match &*typed.expr {
                        Expr::Match(_, pvs) => {
                            pvs.iter().map(|(pat, _)| pat.clone()).collect::<Vec<_>>()
                        }
                        _ => unreachable!(),
                    };
                    let res = Pattern::validate_match_cases_exhaustiveness(
                        &cond_tycon,
                        &cond_ti,
                        &typed.source,
                        pats.into_iter(),
                    );
                    if !self.error_tolerant {
                        res?;
                    }
                }

                Ok(typed)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.unify_type_of_expr(cond, make_bool_ty())?;
                let then_expr = self.unify_type_of_expr(then_expr, ty.clone())?;
                let else_expr = self.unify_type_of_expr(else_expr, ty)?;
                Ok(ei
                    .set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr))
            }
            Expr::TyAnno(e, anno_ty) => {
                let anno_ty = self.validate_type_annotation(&anno_ty)?;
                let child_ty = if let Err(unif_err) =
                    UnifOrOtherErr::extract_others(self.unify(&ty, &anno_ty))?
                {
                    if !self.error_tolerant {
                        let err = self.create_type_mismatch_error(&ty, &anno_ty, &unif_err, &ei.source);
                        return Err(Errors::from_err(err));
                    }
                    // Annotation conflicts with the expected type.
                    // Honour the annotation when typing the child so
                    // the child's elaboration still benefits from it.
                    anno_ty.clone()
                } else {
                    ty.clone()
                };
                let e = self.unify_type_of_expr(e, child_ty)?;
                Ok(ei.set_tyanno_expr(e))
            }
            Expr::MakeStruct(tc, fields) => {
                let strict = !self.error_tolerant;

                // Strict-mode structural validation: in `error_tolerant`
                // mode every gate below is downgraded to "still type
                // each provided field expression, possibly against a
                // fresh tyvar". The user's saved struct literal may
                // be syntactically incomplete (missing/unknown fields,
                // wrong type name) while the cursor sits inside one of
                // its field expressions — the LSP needs that
                // expression typed to drive dot completion.
                let tycon_info = self.type_env.tycons.get(&tc);
                let tycon_info = match tycon_info {
                    Some(info) if info.variant == TyConVariant::Struct => Some(info.clone()),
                    Some(_) => {
                        if strict {
                            return Err(Errors::from_msg_srcs(
                                format!("Type `{}` is not a struct.", tc.to_string()),
                                &[&ei.source],
                            ));
                        }
                        None
                    }
                    None => {
                        if strict {
                            return Err(Errors::from_msg_srcs(
                                format!("Unknown type name `{}`.", tc.to_string()),
                                &[&ei.source],
                            ));
                        }
                        None
                    }
                };

                // Strict path: original elaboration (validate + reorder + type).
                if let Some(ti) = tycon_info.as_ref().filter(|_| strict) {
                    let field_names =
                        ti.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
                    let field_names_in_struct_defn: Set<Name> =
                        Set::from_iter(field_names.iter().cloned());
                    let field_names_in_expression: Set<Name> =
                        Set::from_iter(fields.iter().map(|(name, _, _)| name.clone()));
                    let missing: Vec<Name> = field_names
                        .iter()
                        .filter(|f| !field_names_in_expression.contains(*f))
                        .cloned()
                        .collect();
                    if !missing.is_empty() {
                        let msg = if missing.len() == 1 {
                            format!(
                                "Missing field `{}` of struct `{}`.",
                                missing[0],
                                tc.to_string()
                            )
                        } else {
                            let list = missing
                                .iter()
                                .map(|n| format!("`{}`", n))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("Missing fields {} of struct `{}`.", list, tc.to_string())
                        };
                        let mut err = Error::from_msg_srcs(msg, &[&ei.source]);
                        err.code = Some(ERR_MISSING_STRUCT_FIELD);
                        err.data = Some(serde_json::json!(missing));
                        return Err(Errors::from_err(err));
                    }
                    for f in &field_names_in_expression {
                        if !field_names_in_struct_defn.contains(f) {
                            return Err(Errors::from_msg_srcs(
                                format!("Unknown field `{}` for struct `{}`.", f, tc.to_string()),
                                &[&ei.source],
                            ));
                        }
                    }
                    let struct_ty = tc.get_struct_union_value_type(self);
                    if let Err(e) = UnifOrOtherErr::extract_others(self.unify(&ty, &struct_ty))? {
                        let err =
                            self.create_type_mismatch_error(&ty, &struct_ty, &e, &ei.source);
                        return Err(Errors::from_err(err));
                    }
                    let field_tys = struct_ty.field_types(&self.type_env);
                    assert_eq!(field_tys.len(), fields.len());

                    let fields_map: Map<Name, (Option<Span>, Arc<ExprNode>)> = Map::from_iter(
                        fields.iter().map(|(n, s, e)| (n.clone(), (s.clone(), e.clone()))),
                    );
                    let mut fields = field_names
                        .iter()
                        .map(|name| {
                            let (name_src, e) = fields_map[name].clone();
                            (name.clone(), name_src, e)
                        })
                        .collect::<Vec<_>>();
                    for (field_ty, (_, _, field_expr)) in field_tys.iter().zip(fields.iter_mut())
                    {
                        *field_expr = self.unify_type_of_expr(field_expr, field_ty.clone())?;
                    }
                    return Ok(ei.set_make_struct_fields(fields));
                }

                // Tolerant path: type each provided field expression
                // best-effort. Use the struct's declared type for known
                // fields, fresh tyvar otherwise. Field order, missing
                // and extra fields are not validated — the resulting
                // typed `MakeStruct` may be ill-formed structurally
                // but every field expression carries its inferred type.
                let known_field_tys: Option<Map<Name, Arc<TypeNode>>> = tycon_info.map(|_| {
                    let struct_ty = tc.get_struct_union_value_type(self);
                    let _ = UnifOrOtherErr::extract_others(self.unify(&ty, &struct_ty));
                    let field_tys = struct_ty.field_types(&self.type_env);
                    self.type_env
                        .tycons
                        .get(&tc)
                        .unwrap()
                        .fields
                        .iter()
                        .zip(field_tys.iter())
                        .map(|(f, ft)| (f.name.clone(), ft.clone()))
                        .collect()
                });
                let mut new_fields = fields.clone();
                for (name, _name_src, field_expr) in new_fields.iter_mut() {
                    let field_ty = known_field_tys
                        .as_ref()
                        .and_then(|m| m.get(name).cloned())
                        .unwrap_or_else(|| self.fresh_ty_with_src(&field_expr.source));
                    *field_expr = self.unify_type_of_expr(field_expr, field_ty)?;
                }
                Ok(ei.set_make_struct_fields(new_fields))
            }
            Expr::ArrayLit(elems) => {
                // Prepare type of element.
                let elem_src = if elems.len() > 0 {
                    elems[0].source.clone()
                } else {
                    ei.source.clone().map(|s| s.after_head_character())
                };
                let elem_tv = self.new_tyvar_star();
                self.add_tyvar_source(elem_tv.name.clone(), elem_src.clone());
                let elem_ty = type_from_tyvar(elem_tv);

                let array_ty = type_tyapp(make_array_ty(), elem_ty.clone());
                if let Err(e) = UnifOrOtherErr::extract_others(self.unify(&array_ty, &ty))? {
                    if !self.error_tolerant {
                        let err = self.create_type_mismatch_error(&ty, &array_ty, &e, &ei.source);
                        return Err(Errors::from_err(err));
                    }
                    // Expected type isn't an array; type each element
                    // against the fresh `elem_ty` anyway so subtrees
                    // still get their inferred type.
                }
                let mut ei = ei.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = self.unify_type_of_expr(e, elem_ty.clone())?;
                    ei = ei.set_array_lit_elem(e, i);
                }
                Ok(ei)
            }
            Expr::FFICall(_, ret_ty, param_tys, is_var_args, args, is_io) => {
                let ret_ty = type_tycon(ret_ty);
                let ret_ty = if *is_io {
                    make_tuple_ty(vec![make_iostate_ty(), ret_ty])
                } else {
                    ret_ty
                };
                if let Err(e) = UnifOrOtherErr::extract_others(self.unify(&ty, &ret_ty))? {
                    if !self.error_tolerant {
                        let err = self.create_type_mismatch_error(&ty, &ret_ty, &e, &ei.source);
                        return Err(Errors::from_err(err));
                    }
                    // Expected return type doesn't match the FFI
                    // signature; type each argument against the
                    // declared parameter type anyway so subtrees keep
                    // their inferred type.
                }
                let mut ei = ei.clone();
                for (i, e) in args.iter().enumerate() {
                    let param_ty = if i < param_tys.len() {
                        // The explicitly given parameter type.
                        type_tycon(&param_tys[i])
                    } else if i == args.len() - 1 && *is_io {
                        // The last parameter is iostate for IO FFI call.
                        make_iostate_ty()
                    } else {
                        // An implicitly given parameter type (for variadic arguments).
                        assert!(*is_var_args);
                        let tv = self.new_tyvar_star();
                        self.add_tyvar_source(tv.name.clone(), ei.source.clone());
                        type_from_tyvar(tv)
                    };
                    let e = self.unify_type_of_expr(e, param_ty)?;
                    ei = ei.set_ffi_call_arg(e, i);
                }
                Ok(ei)
            }
            Expr::Eval(side, main) => {
                let side_tv = self.new_tyvar_star();
                self.add_tyvar_source(side_tv.name.clone(), side.source.clone());
                let side = self.unify_type_of_expr(side, type_from_tyvar(side_tv))?;
                let main = self.unify_type_of_expr(main, ty)?;
                Ok(ei.set_eval_main(main).set_eval_side(side))
            }
        }
    }

    // Validate pattern and raise error if invalid,
    fn validate_pattern(&mut self, pat: &PatternNode) -> Result<(), Errors> {
        // In `error_tolerant` mode every gate below is downgraded to a
        // no-op so a single bad sub-check doesn't bail out of the whole
        // walk — siblings still get a chance to validate, and the
        // tolerant variants of `pattern.rs::get_typed` cope with any
        // structural slip that validation would otherwise have caught.
        let tolerate = self.error_tolerant;
        match &pat.pattern {
            Pattern::Var(_, opt_ty) => {
                if let Some(anno_ty) = opt_ty {
                    if let Err(e) = self.validate_type_annotation(anno_ty) {
                        if !tolerate {
                            return Err(e);
                        }
                    }
                }
            }
            Pattern::Struct(tc, pats) => {
                let ti = self.type_env.tycons.get(&tc).unwrap();
                let fields_str = ti.fields.iter().map(|f| f.name.clone()).collect::<Set<_>>();
                let fields_pat = pats
                    .iter()
                    .map(|(name, _, _)| name.clone())
                    .collect::<Set<_>>();
                if fields_pat.len() < pats.len() && !tolerate {
                    return Err(Errors::from_msg_srcs(
                        "Duplicate field in struct pattern.".to_string(),
                        &[&pat.info.source],
                    ));
                }
                for f in fields_pat {
                    if !fields_str.contains(&f) && !tolerate {
                        return Err(Errors::from_msg_srcs(
                            format!(
                                "Unknown field `{}` for struct `{}`.",
                                f,
                                tc.name.to_string()
                            ),
                            &[&pat.info.source],
                        ));
                    }
                }
                for (_, _, p) in pats {
                    if let Err(e) = self.validate_pattern(p) {
                        if !tolerate {
                            return Err(e);
                        }
                    }
                }
            }
            Pattern::Union(_, _, subpat) => {
                if let Err(e) = self.validate_pattern(subpat) {
                    if !tolerate {
                        return Err(e);
                    }
                }
            }
        }
        if pat.pattern.has_duplicate_vars() && !tolerate {
            return Err(Errors::from_msg_srcs(
                "Duplicate name defined by pattern.".to_string(),
                &[&pat.info.source],
            ));
        }
        Ok(())
    }

    pub fn create_tyvar_location_messages(
        &self,
        tvs: &[Arc<TyVar>],
        ref_no: Option<usize>,
    ) -> Vec<(String, Span)> {
        let mut tvs = tvs
            .into_iter()
            .map(|tv| tv.name.clone())
            .collect::<Vec<_>>();
        tvs.sort();
        tvs.dedup();
        let mut msg_srcs = vec![];
        for tv in tvs {
            if let Some(src) = self.tyvar_expr.get(&tv) {
                let prefix = if let Some(ref_no) = ref_no {
                    format!("`{}` in ({})", tv, ref_no)
                } else {
                    format!("`{}`", tv)
                };
                let msg = match short_span_snippet(src) {
                    Some(snippet) => format!("{} is the type for `{}`.", prefix, snippet),
                    // Snippet absent — the span is multi-line, too
                    // long, or zero-width (e.g. the position between
                    // `[` and `]` for an empty-array element). Fall
                    // back to a self-contained sentence rather than a
                    // dangling colon, since the source pointer
                    // attached separately may not visually flow as a
                    // continuation of the message text.
                    None => format!("{} is the type for this expression.", prefix),
                };
                msg_srcs.push((msg, src.clone()));
            }
        }
        msg_srcs
    }

    fn create_type_mismatch_error(
        &self,
        expected_ty: &Arc<TypeNode>,
        found_ty: &Arc<TypeNode>,
        unif_err: &UnificationErr,
        source: &Option<Span>,
    ) -> Error {
        let expected_ty = self.substitution.substitute_type(expected_ty);
        let found_ty = self.substitution.substitute_type(found_ty);
        let mut unif_err = unif_err.clone();
        self.substitution
            .substitute_unification_error(&mut unif_err);

        let mut tvs = vec![];
        expected_ty.free_vars_to_vec(&mut tvs);
        found_ty.free_vars_to_vec(&mut tvs);
        unif_err.free_vars_to_vec(&mut tvs);
        let tv_loc_msgs = self.create_tyvar_location_messages(&tvs, None);
        let mut err = Error::from_msg_srcs(
            format!(
                "Type mismatch. Expected `{}`, found `{}`. They do not match since `{}` cannot be deduced.",
                expected_ty.to_string(),
                found_ty.to_string(),
                unif_err.to_constraint_string(),
            ),
            &[&source],
        );
        err.add_srcs(tv_loc_msgs);
        err
    }

    // Check that the `TypeCheckContext` is "fresh", i.e., it state variables are default.
    pub fn assert_freshness(&self) {
        assert!(self.tyvar_id == 0);
        assert!(self.substitution.is_empty());
        assert!(self.predicates.is_empty());
        assert!(self.equalities.is_empty());
        assert!(self.local_assumed_eqs.is_empty());
        assert!(self.fixed_tyvars.is_empty());
        assert!(self.import_required.is_empty());
    }

    pub fn check_scheme_equivalent(
        self: &TypeCheckContext,
        lhs: &Arc<Scheme>,
        rhs: &Arc<Scheme>,
    ) -> Result<(), UnifOrOtherErr> {
        self.assert_freshness();
        {
            let mut tc = self.clone();
            tc.check_scheme_equivalent_one(lhs, rhs)?;
        }
        {
            let mut tc = self.clone();
            tc.check_scheme_equivalent_one(rhs, lhs)?;
        }

        Ok(())
    }

    fn check_scheme_equivalent_one(
        self: &mut TypeCheckContext,
        lhs: &Arc<Scheme>,
        rhs: &Arc<Scheme>,
    ) -> Result<(), UnifOrOtherErr> {
        let lhs = self.instantiate_scheme(lhs, ConstraintInstantiationMode::Assume)?;
        let rhs = self.instantiate_scheme(rhs, ConstraintInstantiationMode::Require)?;
        self.unify(&lhs, &rhs)?;
        self.reduce_predicates()?;
        if self.predicates.len() > 0 {
            let pred = &self.predicates[0];
            let e = UnificationErr::Unsatisfiable(pred.clone());
            return Err(UnifOrOtherErr::UnifErr(e));
        }
        if self.equalities.len() > 0 {
            let eq = &self.equalities[0];
            let e = UnificationErr::Disjoint(eq.lhs(), eq.value.clone());
            return Err(UnifOrOtherErr::UnifErr(e));
        }
        Ok(())
    }

    /// Check that `expr` matches `expect_scm` and return the
    /// expression annotated with inferred types on every subnode.
    ///
    /// # Returns
    /// * `Ok((expr, errors))` — substitution finished and `expr` is
    ///   the fully substituted typed expression. `errors` may still
    ///   contain tolerated diagnostics (holes, cannot-infer,
    ///   unsatisfiable predicates, disjoint equalities). Callers
    ///   should propagate `errors` but may also use `expr` (e.g. save
    ///   it so the LSP can hover on its sub-expressions).
    /// * `Err(errs)` — a hard failure before substitution completed
    ///   (type mismatch in `unify_type_of_expr`, failure of
    ///   `substitute_and_reduce_type` inside `fix_types`, or scheme
    ///   instantiation failure). No typed expression to return.
    pub fn check_type(
        &mut self,
        expr: Arc<ExprNode>,
        expect_scm: Arc<Scheme>,
    ) -> Result<(Arc<ExprNode>, Errors), Errors> {
        self.assert_freshness();

        fn make_error(
            tc: &TypeCheckContext,
            mut unif_err: UnificationErr,
            src: &Option<Span>,
        ) -> Error {
            tc.substitution.substitute_unification_error(&mut unif_err);
            let mut error = Error::from_msg_srcs(
                format!(
                    "`{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    unif_err.to_constraint_string()
                ),
                &[src],
            );
            let mut tvs = vec![];
            unif_err.free_vars_to_vec(&mut tvs);
            let tv_loc_msgs = tc.create_tyvar_location_messages(&tvs, None);
            error.add_srcs(tv_loc_msgs);
            error
        }

        let specified_ty = UnifOrOtherErr::extract_others(
            self.instantiate_scheme(&expect_scm, ConstraintInstantiationMode::Assume),
        )?;
        if let Err(e) = specified_ty {
            return Err(Errors::from_err(make_error(self, e, &expr.source)));
        }
        let specified_ty = specified_ty.ok().unwrap();

        // Hard step 1: unify. Failure here is a real type mismatch and
        // there is no useful typed expression to keep.
        let expr = self.unify_type_of_expr(&expr, specified_ty.clone())?;

        // Hard step 2: substitute every node's type. This walks the
        // tree but does not check that types are fully determined.
        // Failure means substitute_and_reduce_type itself failed (e.g.
        // an associated-type reduction blew up), which is rare and
        // again leaves no usable typed expression.
        let expr = self.fix_types(expr)?;

        // From here on we have a fully substituted typed expression.
        // Tolerated diagnostics are collected as a cascade — each
        // layer is reported only if every earlier layer was clean,
        // since later diagnostics are usually consequences of earlier
        // ones and showing both is just noise. We always return the
        // typed expression so callers can hand it to the LSP.
        //
        // Order (see also doc on `check_types_are_fixed`):
        //   hole > cannot-infer > predicate > equality
        //
        // - hole > cannot-infer: a hole introduces `Std::#hole : a`
        //   which is the most common source of indeterminate types.
        // - cannot-infer > predicate / equality: an unresolved type
        //   variable usually leaves predicates and equalities
        //   unsolved.
        // - predicate > equality: an unsatisfied trait constraint
        //   often leaves an associated type unable to be reduced,
        //   which then surfaces as a disjoint equality.

        // In `error_tolerant` mode (LSP completion's live-buffer
        // elaborate) every diagnostic layer below is skipped:
        //
        // - Holes are intentional in completion (the cursor itself
        //   resolves to `Std::#hole`), so reporting them is noise.
        // - Unresolved tyvars are expected: the tolerant cases in
        //   `unify_type_of_expr_inner` fall back to fresh tyvars
        //   when a child can't be constrained.
        // - Predicate / equality residue is likewise expected: the
        //   tolerant path may have accumulated inconsistent
        //   constraints from partially failed sub-expressions, and
        //   surfacing them as diagnostics confuses the LSP without
        //   helping the user.
        //
        // The tolerant path's only invariant is "every node has an
        // inferred type" (see Section 1 of the refactor plan);
        // `check_all_typed` walks the tree to catch regressions.
        if self.error_tolerant {
            self.check_all_typed(&expr)?;
            return Ok((expr, Errors::empty()));
        }

        // Pre-extract the source span so the error-construction
        // helpers below can borrow it independently of `expr` (which
        // each early-return consumes).
        let src = expr.source.clone();

        // Layer 1: holes.
        let hole_errors = check_holes::collect_hole_errors(&expr, self);
        if hole_errors.has_diagnostics() {
            return Ok((expr, hole_errors));
        }

        // Layer 2: cannot-infer.
        if let Err(e) = self.check_types_are_fixed(&expr) {
            return Ok((expr, e));
        }

        // Layer 3: predicates. `reduce_predicates` itself can fail
        // with a non-unification diagnostic; treat that as a hard
        // failure (return `Err` from `check_type`, not just a
        // tolerated error).
        if let Err(e) = UnifOrOtherErr::extract_others(self.reduce_predicates())? {
            return Ok((expr, Errors::from_err(make_error(self, e, &src))));
        }
        if self.predicates.len() > 0 {
            let pred = &self.predicates[0];
            let e = UnificationErr::Unsatisfiable(pred.clone());
            return Ok((expr, Errors::from_err(make_error(self, e, &src))));
        }

        // Layer 4: equalities.
        if self.equalities.len() > 0 {
            let eq = &self.equalities[0];
            let e = UnificationErr::Disjoint(eq.lhs(), eq.value.clone());
            return Ok((expr, Errors::from_err(make_error(self, e, &src))));
        }

        Ok((expr, Errors::empty()))
    }

    fn add_substitution(&mut self, subst: &Substitution) -> Result<(), UnifOrOtherErr> {
        self.substitution.compose(subst);
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

        // Structural change-detection. `substitute_equality` and the
        // reductions below only ever touch `eq.args` / `eq.value`, so
        // a structural compare on those fields is sufficient and
        // avoids per-call type rendering.
        let args_before = eq.args.clone();
        let value_before = eq.value.clone();

        // If the equality can be simplified by substitution, call unify.
        self.substitute_equality(&mut eq);
        if eq.args != args_before || eq.value != value_before {
            return self.unify(&eq.lhs(), &eq.value);
        }

        // From here on `eq.args` is stable, so cache the lhs once.
        let lhs = eq.lhs();

        // If the lhs of the equality is reducible, call unify.
        let red_lhs = self.reduce_type_by_equality(lhs.clone())?;
        if red_lhs != lhs {
            return self.unify(&red_lhs, &eq.value);
        }

        // If the rhs of the equality is reducible, call unify.
        let rhs_before = eq.value.clone();
        eq.value = self.reduce_type_by_equality(eq.value.clone())?;
        if eq.value != rhs_before {
            return self.unify(&lhs, &eq.value);
        }

        // Avoid adding trivial equality.
        if lhs == eq.value {
            return Ok(());
        }

        self.equalities.push(eq);
        Ok(())
    }

    // Reduce a type by replacing associated type to its value.
    fn reduce_type_by_equality(&mut self, ty: Arc<TypeNode>) -> Result<Arc<TypeNode>, Errors> {
        match &ty.ty {
            Type::TyVar(_) => Ok(ty),
            Type::TyCon(_) => Ok(ty),
            Type::TyApp(tyfun, tyarg) => {
                let tyfun = self.reduce_type_by_equality(tyfun.clone())?;
                let tyarg = self.reduce_type_by_equality(tyarg.clone())?;
                Ok(ty.set_tyapp_fun(tyfun).set_tyapp_arg(tyarg))
            }
            Type::AssocTy(assoc_ty, args) => {
                // Reduce each arguments.
                let args = collect_results(
                    args.iter()
                        .map(|arg| self.reduce_type_by_equality(arg.clone())),
                )?;

                // The first argument should implement the trait of the associated type.
                let pred = Predicate {
                    trait_id: assoc_ty.trait_id(),
                    ty: args[0].clone(),
                    src: None,
                    trait_src: None,
                };
                self.predicates.push(pred);

                let ty = ty.set_assocty_args(args);

                // Try matching to assumed equality.
                for assumed_eq in &self.assumed_eqs.get(assoc_ty).cloned().unwrap_or(vec![]) {
                    // Instantiate `assumed_eq`.
                    let mut subst = Substitution::default();
                    for tv in &assumed_eq.gen_vars {
                        let new_tv = type_from_tyvar(self.new_tyvar_by(tv));
                        let merge_ok = subst.merge(&Substitution::single(&tv.name, new_tv));
                        assert!(merge_ok);
                    }
                    let mut equality = assumed_eq.equality.clone();
                    subst.substitute_equality(&mut equality);

                    // Try to match lhs of `equality` to `ty`.
                    let subst: Option<Substitution> = Substitution::matching(
                        &equality.lhs(),
                        &ty,
                        &self.fixed_tyvars,
                        &self.kind_env,
                    )?;
                    if subst.is_none() {
                        continue;
                    }
                    let subst: Substitution = subst.unwrap();
                    let rhs = subst.substitute_type(&equality.value);
                    return self.reduce_type_by_equality(rhs);
                }
                Ok(ty)
            }
        }
    }

    // Unify two types.
    pub fn unify(
        &mut self,
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
    ) -> Result<(), UnifOrOtherErr> {
        let mut ty1 = self.substitute_and_reduce_type(ty1)?;
        let mut ty2 = self.substitute_and_reduce_type(ty2)?;

        // `TypeNode::PartialEq` is structural and ignores source spans.
        // The `Arc::ptr_eq` fast path catches the common case where
        // `substitute_and_reduce_type` returned the same Arc unchanged.
        if Arc::ptr_eq(&ty1, &ty2) || ty1 == ty2 {
            return Ok(());
        }

        // Case: Either is a type variable.
        for _ in 0..2 {
            match &ty1.ty {
                Type::TyVar(var1) => {
                    if !self
                        .fixed_tyvars
                        .iter()
                        .any(|fixed_tv| fixed_tv.name == var1.name)
                    {
                        return self.unify_tyvar(var1.clone(), ty2.clone());
                    }
                }
                _ => {}
            }
            std::mem::swap(&mut ty1, &mut ty2);
        }

        // Case: Either is usage of associated type.
        for _ in 0..2 {
            if let Type::AssocTy(assoc_ty, args) = &ty1.ty {
                let eq = Equality {
                    assoc_type: assoc_ty.clone(),
                    args: args.clone(),
                    value: ty2.clone(),
                    src: None,
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
            }
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
        assert!(!self
            .fixed_tyvars
            .iter()
            .any(|fixed_tv| fixed_tv.name == tyvar1.name));

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

        // If `ty2` is also a type variable, unify source locations of them.
        if let Type::TyVar(tv2) = &ty2.ty {
            self.unify_tyvar_source(tyvar1.name.clone(), tv2.name.clone());
        }

        self.add_substitution(&Substitution::single(&tyvar1.name, ty2.clone()))?;
        Ok(())
    }

    /// Reduces predicates stored in `self.predicates` as long as possible.
    /// If a predicate is unsatisfiable, returns `Err`.
    pub(crate) fn reduce_predicates(&mut self) -> Result<(), UnifOrOtherErr> {
        let mut irr_preds = vec![];
        let mut skip: Set<String> = Set::default();
        while let Some(pred) = self.predicates.pop() {
            self.reduce_predicate(pred, &mut irr_preds, &mut skip)?;
        }
        self.predicates = irr_preds;
        Ok(())
    }

    // Reduce a predicate and add reduced predicates to `irr_preds`.
    fn reduce_predicate(
        &mut self,
        pred: Predicate,
        irr_preds: &mut Vec<Predicate>,
        skip: &mut Set<String>,
    ) -> Result<(), UnifOrOtherErr> {
        for pred in pred.resolve_trait_aliases(&self.trait_env.aliases)? {
            self.reduce_predicate_noalias(pred, irr_preds, skip)?;
        }
        Ok(())
    }

    // Add a predicate after reducing it.
    // Trait in `pred` should not be a trait alias.
    fn reduce_predicate_noalias(
        &mut self,
        mut pred: Predicate,
        irr_preds: &mut Vec<Predicate>,
        skip: &mut Set<String>,
    ) -> Result<(), UnifOrOtherErr> {
        self.substitute_predicate(&mut pred);
        let pred_str = pred.to_string();
        if skip.contains(&pred_str) {
            return Ok(());
        }
        skip.insert(pred_str);
        pred.ty = self.substitute_and_reduce_type(&pred.ty)?;
        let mut unifiable = false;
        for qual_pred_scm in &self
            .assumed_preds
            .get(&pred.trait_id)
            .unwrap_or(&vec![])
            .clone()
        {
            // Instantiate qualified predicate.
            let mut subst = Substitution::default();
            for tv in &qual_pred_scm.gen_vars {
                let new_tv = type_from_tyvar(self.new_tyvar_by(tv));
                let merge_ok = subst.merge(&Substitution::single(&tv.name, new_tv));
                assert!(merge_ok);
            }
            let mut qual_pred = qual_pred_scm.qual_pred.clone();
            subst.substitute_qualpred(&mut qual_pred);

            // Try to match head of `qual_pred` to `pred`.
            if let Some(subst) = Substitution::matching(
                &qual_pred.predicate.ty,
                &pred.ty,
                &self.fixed_tyvars,
                &self.kind_env,
            )? {
                for mut eq in qual_pred.eq_constraints {
                    subst.substitute_equality(&mut eq);
                    self.add_equality(eq)?;
                }
                for mut pred in qual_pred.pred_constraints {
                    subst.substitute_predicate(&mut pred);
                    self.reduce_predicate(pred, irr_preds, skip)?;
                }
                return Ok(());
            } else {
                // If match fails, then we cannot reduce the predicate at now.
                // But we may be able to reduce it after the predicate is substituted further.
                // To see if there is possibility for further reduction, we check here the unifiability.
                let mut tc = self.clone();
                if UnifOrOtherErr::extract_others(tc.unify(&qual_pred.predicate.ty, &pred.ty))?
                    .is_ok()
                {
                    unifiable = true;
                }
            }
        }
        if !unifiable {
            return Err(UnificationErr::Unsatisfiable(pred).into());
        }
        irr_preds.push(pred);
        return Ok(());
    }

    pub fn fix_types_for_pattern(
        &mut self,
        pat: Arc<PatternNode>,
    ) -> Result<Arc<PatternNode>, Errors> {
        let raw_ty = pat
            .info
            .type_
            .as_ref()
            .expect("fix_types_for_pattern: every pattern should be typed");
        // Same rationale as in `fix_types`.
        let reduced = self.substitute_and_reduce_type(raw_ty);
        let ty = self.tolerate(reduced)?.unwrap_or_else(|| raw_ty.clone());
        let pat = pat.set_type(ty);
        Ok(match &pat.pattern {
            Pattern::Var(_var, _anno_ty) => {
                // Currently, type annotation is not used in the following processes, so there is no need to finish type annotation.
                pat
            }
            Pattern::Union(_, _, subpat) => {
                let subpat = self.fix_types_for_pattern(subpat.clone())?;
                pat.set_union_pat(subpat)
            }
            Pattern::Struct(_, fied_to_pat) => {
                let mut field_to_pat = fied_to_pat.clone();
                for (_field_name, _, subpat) in field_to_pat.iter_mut() {
                    let new_subpat = self.fix_types_for_pattern(subpat.clone())?;
                    *subpat = new_subpat;
                }
                pat.set_struct_field_to_pat(field_to_pat)
            }
        })
    }

    fn check_is_type_fixed(
        &self,
        src_type: &str,
        src: &Option<Span>,
        ty: &Arc<TypeNode>,
    ) -> Option<Errors> {
        let mut errs = None;
        let mut fvs = ty
            .free_vars()
            .into_iter()
            .filter(|(k, _v)| !self.fixed_tyvars.iter().any(|tv| tv.name == *k));
        if let Some((fv_name, fv)) = fvs.next() {
            // Must stay in sync with the same message in program.rs (instantiate_expr).
            let mut err = Error::from_msg_srcs(
                format!(
                    "Cannot infer the type of this {0}: inferred as `{1}`, but the type variable `{2}` is unresolved.\nHint: add a type annotation to this {0}.",
                    src_type,
                    ty.to_string(),
                    fv_name,
                ),
                &[src],
            );
            let tv_loc_msgs = self.create_tyvar_location_messages(&[fv], None);
            err.add_srcs(tv_loc_msgs);
            errs = Some(Errors::from_err(err));
        }
        errs
    }

    /// Apply the type substitution to every node's `type_` field and to
    /// every pattern type. Does not check whether the resulting types
    /// are fixed (free of unsolved type variables); see
    /// `check_types_are_fixed` for that. Substitution and the
    /// fixed-type check are kept separate so other passes (e.g. hole
    /// detection) can run on the substituted AST in between.
    pub fn fix_types(&mut self, expr: Arc<ExprNode>) -> Result<Arc<ExprNode>, Errors> {
        let raw_ty = expr
            .type_
            .as_ref()
            .expect("fix_types: every node should be typed by unify_type_of_expr");
        // Associated-type reduction can fail when the accumulated
        // substitution / equality state is inconsistent — a normal
        // consequence of the tolerant elaborator stitching together
        // partially failed sub-expressions. Fall back to the
        // un-reduced type so downstream consumers (LSP dot completion,
        // hover) can still read whatever type info survived.
        let reduced = self.substitute_and_reduce_type(raw_ty);
        let ty = self.tolerate(reduced)?.unwrap_or_else(|| raw_ty.clone());
        let expr = expr.set_type(ty);
        Ok(match &*expr.expr {
            Expr::Var(_) => expr,
            Expr::LLVM(_) => expr,
            Expr::App(fun, args) => {
                let args = collect_results(args.iter().map(|arg| self.fix_types(arg.clone())))?;
                let fun = self.fix_types(fun.clone())?;
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_args, body) => {
                let body = self.fix_types(body.clone())?;
                expr.set_lam_body(body)
            }
            Expr::Let(pat, val, body) => {
                let pat = self.fix_types_for_pattern(pat.clone())?;
                let val = self.fix_types(val.clone())?;
                let body = self.fix_types(body.clone())?;
                expr.set_let_pat(pat).set_let_bound(val).set_let_value(body)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.fix_types(cond.clone())?;
                let then_expr = self.fix_types(then_expr.clone())?;
                let else_expr = self.fix_types(else_expr.clone())?;
                expr.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::Match(cond, pat_vals) => {
                let cond = self.fix_types(cond.clone())?;
                let mut new_pat_vals = vec![];
                for (pat, val) in pat_vals {
                    let pat = self.fix_types_for_pattern(pat.clone())?;
                    let val = self.fix_types(val.clone())?;
                    new_pat_vals.push((pat, val));
                }
                expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
            }
            Expr::TyAnno(e, _) => expr.set_tyanno_expr(self.fix_types(e.clone())?),
            Expr::MakeStruct(_tc, fields) => {
                let mut new_fields = fields.clone();
                for (_, _, e) in new_fields.iter_mut() {
                    *e = self.fix_types(e.clone())?;
                }
                expr.set_make_struct_fields(new_fields)
            }
            Expr::ArrayLit(elems) => {
                let elems = collect_results(elems.iter().map(|e| self.fix_types(e.clone())))?;
                expr.set_array_lit_elems(elems)
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                let args = collect_results(args.iter().map(|arg| self.fix_types(arg.clone())))?;
                expr.set_ffi_call_args(args)
            }
            Expr::Eval(side, main) => {
                let side = self.fix_types(side.clone())?;
                let main = self.fix_types(main.clone())?;
                expr.set_eval_side(side).set_eval_main(main)
            }
        })
    }

    /// Verify that every node and pattern in `expr` has a type with no
    /// unsolved free type variables. Walks depth-first and surfaces
    /// the innermost failure (errors from inner subtrees take
    /// precedence over the failure at the root).
    pub fn check_types_are_fixed(&self, expr: &Arc<ExprNode>) -> Result<(), Errors> {
        match &*expr.expr {
            Expr::Var(_) | Expr::LLVM(_) => {}
            Expr::App(fun, args) => {
                for arg in args {
                    self.check_types_are_fixed(arg)?;
                }
                self.check_types_are_fixed(fun)?;
            }
            Expr::Lam(_, body) => self.check_types_are_fixed(body)?,
            Expr::Let(pat, val, body) => {
                self.check_pattern_types_are_fixed(pat)?;
                self.check_types_are_fixed(val)?;
                self.check_types_are_fixed(body)?;
            }
            Expr::If(cond, then_e, else_e) => {
                self.check_types_are_fixed(cond)?;
                self.check_types_are_fixed(then_e)?;
                self.check_types_are_fixed(else_e)?;
            }
            Expr::Match(cond, arms) => {
                self.check_types_are_fixed(cond)?;
                for (pat, val) in arms {
                    self.check_pattern_types_are_fixed(pat)?;
                    self.check_types_are_fixed(val)?;
                }
            }
            Expr::TyAnno(e, _) => self.check_types_are_fixed(e)?,
            Expr::MakeStruct(_, fields) => {
                for (_, _, fe) in fields {
                    self.check_types_are_fixed(fe)?;
                }
            }
            Expr::ArrayLit(elems) => {
                for e in elems {
                    self.check_types_are_fixed(e)?;
                }
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                for a in args {
                    self.check_types_are_fixed(a)?;
                }
            }
            Expr::Eval(side, main) => {
                self.check_types_are_fixed(side)?;
                self.check_types_are_fixed(main)?;
            }
        }
        if let Some(errs) =
            self.check_is_type_fixed("expression", &expr.source, expr.type_.as_ref().unwrap())
        {
            return Err(errs);
        }
        Ok(())
    }

    /// Pattern-tree counterpart of `check_types_are_fixed`. Recurses
    /// into sub-patterns, then validates the type of `pat` itself.
    fn check_pattern_types_are_fixed(&self, pat: &Arc<PatternNode>) -> Result<(), Errors> {
        match &pat.pattern {
            Pattern::Var(_, _) => {}
            Pattern::Union(_, _, subpat) => self.check_pattern_types_are_fixed(subpat)?,
            Pattern::Struct(_, fields) => {
                for (_, _, subpat) in fields {
                    self.check_pattern_types_are_fixed(subpat)?;
                }
            }
        }
        if let Some(errs) = self.check_is_type_fixed(
            "pattern",
            &pat.info.source,
            pat.info.type_.as_ref().unwrap(),
        ) {
            return Err(errs);
        }
        Ok(())
    }

    /// `error_tolerant`-mode counterpart of `check_types_are_fixed`:
    /// verify the weaker invariant that every node and pattern in
    /// `expr` carries an inferred `type_` (it may still contain
    /// unresolved tyvars). Used in `check_type`'s tolerant path to
    /// surface elaborator bugs early — a `None` here would have
    /// panicked in `fix_types` under the pre-refactor codepath.
    fn check_all_typed(&self, expr: &Arc<ExprNode>) -> Result<(), Errors> {
        if expr.type_.is_none() {
            return Err(Errors::from_msg_srcs(
                "Internal error: error_tolerant elaborate left an expression node without an inferred type."
                    .to_string(),
                &[&expr.source],
            ));
        }
        match &*expr.expr {
            Expr::Var(_) | Expr::LLVM(_) => {}
            Expr::App(fun, args) => {
                for arg in args {
                    self.check_all_typed(arg)?;
                }
                self.check_all_typed(fun)?;
            }
            Expr::Lam(_, body) => self.check_all_typed(body)?,
            Expr::Let(pat, val, body) => {
                self.check_all_pattern_typed(pat)?;
                self.check_all_typed(val)?;
                self.check_all_typed(body)?;
            }
            Expr::If(cond, then_e, else_e) => {
                self.check_all_typed(cond)?;
                self.check_all_typed(then_e)?;
                self.check_all_typed(else_e)?;
            }
            Expr::Match(cond, arms) => {
                self.check_all_typed(cond)?;
                for (pat, val) in arms {
                    self.check_all_pattern_typed(pat)?;
                    self.check_all_typed(val)?;
                }
            }
            Expr::TyAnno(e, _) => self.check_all_typed(e)?,
            Expr::MakeStruct(_, fields) => {
                for (_, _, fe) in fields {
                    self.check_all_typed(fe)?;
                }
            }
            Expr::ArrayLit(elems) => {
                for e in elems {
                    self.check_all_typed(e)?;
                }
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                for a in args {
                    self.check_all_typed(a)?;
                }
            }
            Expr::Eval(side, main) => {
                self.check_all_typed(side)?;
                self.check_all_typed(main)?;
            }
        }
        Ok(())
    }

    /// Pattern-tree counterpart of `check_all_typed`.
    fn check_all_pattern_typed(&self, pat: &Arc<PatternNode>) -> Result<(), Errors> {
        if pat.info.type_.is_none() {
            return Err(Errors::from_msg_srcs(
                "Internal error: error_tolerant elaborate left a pattern node without an inferred type."
                    .to_string(),
                &[&pat.info.source],
            ));
        }
        match &pat.pattern {
            Pattern::Var(_, _) => {}
            Pattern::Union(_, _, subpat) => self.check_all_pattern_typed(subpat)?,
            Pattern::Struct(_, fields) => {
                for (_, _, subpat) in fields {
                    self.check_all_pattern_typed(subpat)?;
                }
            }
        }
        Ok(())
    }
}

/// Returns the trimmed source text covered by `span` if it fits on a single line and within a small character budget, suitable for inlining into a diagnostic message.
fn short_span_snippet(span: &Span) -> Option<String> {
    const MAX_CHARS: usize = 30;
    let source = span.input.string().ok()?;
    let snippet = source.get(span.start..span.end)?;
    let trimmed = snippet.trim();
    if trimmed.is_empty() || trimmed.contains('\n') {
        return None;
    }
    if trimmed.chars().count() > MAX_CHARS {
        return None;
    }
    Some(trimmed.to_string())
}

#[derive(Clone)]
pub enum UnificationErr {
    Unsatisfiable(Predicate),
    Disjoint(Arc<TypeNode>, Arc<TypeNode>),
}

impl UnificationErr {
    pub fn to_constraint_string(&self) -> String {
        match self {
            UnificationErr::Unsatisfiable(p) => p.to_string(),
            UnificationErr::Disjoint(ty1, ty2) => {
                format!("{} = {}", ty1.to_string(), ty2.to_string())
            }
        }
    }

    // Append free type variables to a buffer of type Vec.
    pub fn free_vars_to_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        match self {
            UnificationErr::Unsatisfiable(p) => p.free_vars_to_vec(buf),
            UnificationErr::Disjoint(ty1, ty2) => {
                ty1.free_vars_to_vec(buf);
                ty2.free_vars_to_vec(buf);
            }
        }
    }
}

pub enum UnifOrOtherErr {
    UnifErr(UnificationErr),
    Others(Errors),
}

impl UnifOrOtherErr {
    pub fn extract_others<T>(
        res: Result<T, UnifOrOtherErr>,
    ) -> Result<Result<T, UnificationErr>, Errors> {
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
