use super::check_holes::collect_hole_errors;
use super::typecheckcache::TypeCheckCache;
use crate::ast::import;
use crate::misc::{collect_results, grow_stack, insert_to_map_vec, shorten_for_report, Map, Set};
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
        qual_pred::{QualPred, QualPredScheme},
        qual_type::QualType,
        traits::{TraitEnv, TraitId},
        types::{
            is_type_wildcard_tyvar, kind_star, make_tyvar, type_from_tyvar, type_fun, type_tyapp,
            type_tycon, AssocType, Kind, OpaqueTyConResolution, Scheme, TyCon, TyConInfo,
            TyConVariant, TyVar, Type, TypeNode, MAX_TYPE_DEPTH,
        },
    },
    constants::{
        ERR_AMBIGUOUS_NAME, ERR_MISSING_STRUCT_FIELD, ERR_NO_VALUE_MATCH, ERR_UNKNOWN_NAME,
        WRAP_OPAQUE_TYVAR_PREFIX,
    },
    elaboration::name_resolution::NameResolutionContext,
    error::{Error, Errors},
    fixstd::builtin::{make_array_ty, make_bool_ty, make_iostate_ty, make_tuple_ty},
    parse::sourcefile::Span,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::mem;
use std::sync::Arc;

/// The names a value of type `T` can be written under: the local names bound around the expression
/// being checked, and the global names of the whole program.
#[derive(Clone)]
pub struct Scope<T> {
    /// The values bound to each local name, innermost last, so that a binding made inside another
    /// hides it until it is popped.
    local: Map<Name, Vec<T>>,
    /// Every global name with the value it stands for, shared by all the scopes cloned from this
    /// one.
    global: Arc<Vec<(FullName, T)>>,
}

impl<T> Default for Scope<T> {
    /// A scope in which no name is bound.
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
    /// Binds `name` locally to `v`, hiding what `name` was bound to until the binding is popped.
    pub fn push(&mut self, name: &Name, v: T) {
        insert_to_map_vec(&mut self.local, name, v);
    }

    /// Removes the innermost local binding of `name`, uncovering the one it hid. Panics unless
    /// `name` has been bound.
    pub fn pop(self: &mut Self, name: &Name) {
        self.local.get_mut(name).unwrap().pop();
    }

    /// Whether `name` is bound locally.
    pub fn has_value(&self, name: &Name) -> bool {
        self.local.contains_key(name) && !self.local[name].is_empty()
    }

    /// The value `name` is bound to by its innermost local binding.
    pub fn get_local(&self, name: &Name) -> Option<T> {
        if self.local.contains_key(name) && !self.local[name].is_empty() {
            Some(self.local[name].last().unwrap().clone())
        } else {
            None
        }
    }

    /// The names bound locally.
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

    /// Replaces the global names and their values, which every scope later cloned from this one
    /// shares.
    pub fn set_globals(&mut self, globals: Vec<(FullName, T)>) {
        self.global = Arc::new(globals);
    }

    /// The values `name` can stand for, each with the namespace of the name it was found under, for
    /// an overload resolution to choose between.
    ///
    /// A local `name` that is bound stands for its innermost binding alone. Otherwise every global
    /// name that `name` is a suffix of and that the imports make accessible is a candidate; a name
    /// written absolutely reaches the one global it spells out, and the namespace answered for it
    /// is absolute as well.
    ///
    /// # Arguments
    /// * `import_stmts` — the import statements in force where the name is written, which decide
    ///   which global names are accessible from there.
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

/// What each of a set of type variables is replaced by.
///
/// No type on the right hand side names a type variable the substitution replaces, so replacing
/// every such variable of a type takes one walk over that type.
#[derive(Clone, Serialize, Deserialize)]
pub struct Substitution {
    /// The type replacing each type variable, by the variable's name.
    pub data: Map<Name, Arc<TypeNode>>,
}

impl Default for Substitution {
    /// A substitution that replaces no type variable.
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl Substitution {
    /// Whether this substitution replaces no type variable, so that applying it changes nothing.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The substitution that replaces the type variable named `var` by `ty`, and nothing else.
    pub fn single(var: &str, ty: Arc<TypeNode>) -> Self {
        let mut data = Map::<String, Arc<TypeNode>>::default();
        data.insert(var.to_string(), ty);
        Self { data }
    }

    /// Extends this substitution to the one that applies `following` after it: every type it
    /// replaces a variable by has `following` applied to it, and the replacements of `following`
    /// are added.
    ///
    /// Panics where `following` replaces a variable this substitution already replaces.
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

    /// Adds to this substitution the replacements of `other`, which have to agree with it wherever
    /// both replace one type variable.
    ///
    /// # Returns
    /// Whether the two agreed. Where they disagree, the replacements taken from `other` before the
    /// disagreement stay, so a caller that carries on has to drop this substitution.
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

    /// Replaces the type variables this substitution binds in the type `p` constrains.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        p.ty = self.substitute_type(&p.ty);
    }

    /// Replaces each type variable of `ty` that this substitution binds.
    ///
    /// A type none of whose variables this substitution replaces is returned as
    /// it came: the common case of a substitution that says nothing about a type
    /// walks the type and hands back the same node.
    pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        match &ty.ty {
            Type::TyVar(tyvar) => self.data.get(&tyvar.name).map_or(ty.clone(), |sub| {
                sub.set_source_if_none(ty.get_source().clone())
            }),
            Type::TyCon(_) => ty.clone(),
            Type::TyApp(fun, arg) => {
                let new_fun = self.substitute_type(fun);
                let new_arg = self.substitute_type(arg);
                if Arc::ptr_eq(&new_fun, fun) && Arc::ptr_eq(&new_arg, arg) {
                    return ty.clone();
                }
                ty.set_tyapp_fun(new_fun).set_tyapp_arg(new_arg)
            }
            Type::AssocTy(_, args) => {
                let new_args = args
                    .iter()
                    .map(|arg| self.substitute_type(arg))
                    .collect::<Vec<_>>();
                if new_args
                    .iter()
                    .zip(args)
                    .all(|(new_arg, arg)| Arc::ptr_eq(new_arg, arg))
                {
                    return ty.clone();
                }
                ty.set_assocty_args(new_args)
            }
        }
    }

    /// Substitutes the types the error carries, so it is reported in terms of
    /// the current substitution.
    pub fn substitute_unification_error(&self, e: &mut UnificationErr) {
        match e {
            UnificationErr::Unsatisfiable(predicate) => {
                self.substitute_predicate(predicate);
            }
            UnificationErr::Circular(way) | UnificationErr::Endless(way) => {
                for predicate in way {
                    self.substitute_predicate(predicate);
                }
            }
            UnificationErr::Disjoint(ty1, ty2) => {
                *ty1 = self.substitute_type(ty1);
                *ty2 = self.substitute_type(ty2);
            }
        }
    }

    /// Replaces the type variables this substitution replaces throughout `scm`: in its type, its
    /// predicates and its equalities.
    ///
    /// Panics where this substitution replaces a variable the scheme generalizes, since such a
    /// variable stands for every type and no substitution may fix it to one.
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

    /// Replaces the type variables this substitution replaces throughout `qual_type`: in its type
    /// and in the predicates and equalities qualifying it.
    pub fn substitute_qualtype(&self, qual_type: &mut QualType) {
        for pred in &mut qual_type.preds {
            self.substitute_predicate(pred);
        }
        for eq in &mut qual_type.eqs {
            self.substitute_equality(eq);
        }
        qual_type.ty = self.substitute_type(&qual_type.ty);
    }

    /// Replaces the type variables this substitution replaces in the arguments of the associated
    /// type `eq` speaks about and in the type it is equated to.
    pub fn substitute_equality(&self, eq: &mut Equality) {
        for arg in &mut eq.args {
            *arg = self.substitute_type(arg);
        }
        eq.value = self.substitute_type(&eq.value);
    }

    /// Replaces the type variables this substitution replaces throughout `qual_pred`: in the
    /// predicate it states and in the constraints that predicate is qualified by.
    pub fn substitute_qualpred(&self, qual_pred: &mut QualPred) {
        for pred in &mut qual_pred.pred_constraints {
            self.substitute_predicate(pred);
        }
        for eq in &mut qual_pred.eq_constraints {
            self.substitute_equality(eq);
        }
        self.substitute_predicate(&mut qual_pred.predicate);
    }

    /// The smallest substitution `s` with `s(ty1) = ty2`, and `None` where no substitution sends
    /// `ty1` to `ty2`.
    ///
    /// The two types are compared as they are written, so an associated type matches an associated
    /// type of the same name and is never reduced to the type it stands for.
    ///
    /// # Arguments
    /// * `fixed_tyvars` — the type variables that stand for themselves; one of them matches a type
    ///   written the same way and nothing else.
    /// * `kind_env` — the kinds by which a type variable and the type it would take are compared,
    ///   so that a match giving a variable a type of another kind is refused.
    pub fn matching(
        ty1: &Arc<TypeNode>,
        ty2: &Arc<TypeNode>,
        fixed_tyvars: &[Arc<TyVar>],
        kind_env: &KindEnv,
    ) -> Result<Option<Self>, Errors> {
        Self::matching_internal(ty1, ty2, fixed_tyvars, Some(kind_env))
    }

    /// The smallest substitution `s` with `s(ty1) = ty2`, and `None` where no substitution sends
    /// `ty1` to `ty2`, decided by the shape of the two types alone.
    ///
    /// A type variable may take a type of another kind here, so this answers a substitution where a
    /// match that compares kinds answers `None`. Use it where the kinds of the program are out of
    /// reach.
    ///
    /// # Arguments
    /// * `fixed_tyvars` — the type variables that stand for themselves; one of them matches a type
    ///   written the same way and nothing else.
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

    /// The smallest substitution `s` with `s(ty1) = ty2`, and `None` where no substitution sends
    /// `ty1` to `ty2`.
    ///
    /// # Arguments
    /// * `fixed_tyvars` — the type variables that stand for themselves; one of them matches a type
    ///   written the same way and nothing else.
    /// * `kind_env` — the kinds by which a type variable and the type it would take are compared.
    ///   Where it is absent the kinds go uncompared, and the answer is then always `Ok`.
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
                        match Self::matching_internal(&args1[i], &args2[i], fixed_tyvars, kind_env)?
                        {
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

/// What becomes of the constraints of a type scheme when the scheme is instantiated.
pub enum ConstraintInstantiationMode {
    /// Each constraint becomes one the type checking has to deduce.
    Require,
    /// Each constraint becomes one that later deductions may use.
    Assume,
}

/// The state of type checking an expression: the assumptions it is checked under, what the
/// inference has settled so far, and the environment of the program it belongs to.
///
/// Reference: https://uhideyuki.sakura.ne.jp/studs/index.cgi/ja/HindleyMilnerInHaskell#fn6
#[derive(Clone)]
pub struct TypeCheckContext {
    /// The number the next type variable created here is named after.
    tyvar_id: u32,
    /// The source location of the expression whose type each type variable is, by the variable's
    /// name, so that a diagnostic naming a type variable can point at where it arose.
    pub tyvar_expr: Map<String, Span>,
    /// The type scheme each name in scope stands for. These are the assumptions of the inference.
    pub scope: Scope<Arc<Scheme>>,
    /// What the inference has settled about the type variables so far.
    pub substitution: Substitution,
    /// The equalities on associated types the inference has still to settle.
    pub equalities: Vec<Equality>,
    /// The trait constraints the inference requires and has still to deduce.
    pub predicates: Vec<Predicate>,
    /// The traits of the program, their aliases and their instances.
    pub trait_env: Arc<TraitEnv>,
    /// The type constructors of the program and what each declares.
    pub type_env: TypeEnv,
    /// The kind of each type constructor, associated type and trait of the program.
    pub kind_env: Arc<KindEnv>,
    /// The import statements of each module, by module name. Shared, so that cloning a context
    /// copies no statement.
    pub import_statements: Arc<Map<Name, Vec<ImportStatement>>>,
    /// The module the expression being checked is defined in, which decides the names accessible
    /// to it.
    pub current_module: Option<ModuleInfo>,
    /// The global names the expression reached without writing them absolutely, which the module
    /// it belongs to therefore has to import.
    pub import_required: Vec<FullName>,
    /// The equalities on associated types that may be used without deducing them, by the
    /// associated type each speaks about. Shared, so that a context cloned for a speculative check
    /// copies none of them.
    pub assumed_eqs: Arc<Map<AssocType, Vec<EqualityScheme>>>,
    /// The trait constraints that may be used without deducing them, by the trait each speaks
    /// about: the program's instances, and the constraints the checked value's own signature
    /// states. Shared, so that a context cloned for a speculative check copies none of them.
    pub assumed_preds: Arc<Map<TraitId, Vec<QualPredScheme>>>,
    /// The type variables that stand for themselves, which unification may not replace by another
    /// type: the ones generalized by the type the expression is checked against.
    ///
    /// Their number is small, so a lookup searches the whole list.
    pub fixed_tyvars: Vec<Arc<TyVar>>,
    /// The equalities the checked value's own signature states, such as `Elem c1 = e` and
    /// `Elem c2 = e` while checking
    /// `extend : [c1 : Collects, c2 : Collects, Elem c1 = e, Elem c2 = e] c1 -> c2 -> c2`.
    pub local_assumed_eqs: Vec<Equality>,
    /// Where the type inferred for a global value is looked up and stored, so that a value whose
    /// source is unchanged is not checked again.
    pub cache: Arc<dyn TypeCheckCache + Sync + Send>,
    /// How many threads check the program's global values at once.
    pub num_worker_threads: usize,
    /// The type variable created for each generalized variable that stands for an opaque type,
    /// such as `#Std::repeat::?it`, by that variable's name. Substituting them once the inference
    /// is done gives the types the opaque ones stand for.
    pub opaque_instantiations: Map<Name, Arc<TyVar>>,
    /// When true, errors raised from elaborating a sub-expression are
    /// swallowed: that sub-expression keeps the expected type at its
    /// root and gets fresh type variables below (`set_fallback_types`),
    /// and elaboration continues on its siblings, so types can still be
    /// inferred around an unrelated type error elsewhere in the body.
    pub error_tolerant: bool,
}

impl TypeCheckContext {
    /// Print the entry count of each of the context's collections; a
    /// debugging aid for inspecting the context's growth.
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
            assumed_preds: Arc::new(assumed_preds),
            assumed_eqs: Arc::new(assumed_eqs),
            fixed_tyvars: vec![],
            local_assumed_eqs: vec![],
            import_required: vec![],
            cache,
            num_worker_threads,
            opaque_instantiations: Map::default(),
            error_tolerant,
        }
    }

    /// Records that the type of the expression at `source` is the type variable named `tyvar_name`,
    /// so that a diagnostic naming that variable can point at the expression. An expression with no
    /// source location leaves the record as it stands.
    pub fn add_tyvar_source(&mut self, tyvar_name: Name, source: Option<Span>) {
        if let Some(source) = source {
            self.tyvar_expr.insert(tyvar_name, source);
        }
    }

    /// Fresh type variable wrapped as `TypeNode`, with `src` registered
    /// as its source span. Useful as an unconstrained-but-typed
    /// placeholder when a child of an elaboration cannot be given a
    /// more specific type.
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

    /// Unify `expected` and `found`, surfacing a type-mismatch error
    /// pointed at `source` on failure. In `error_tolerant` mode a
    /// unification mismatch is swallowed (`Ok(())`) so the caller can
    /// keep elaborating siblings; non-unification errors (e.g. an
    /// associated-type reduction failure) always propagate.
    fn unify_or_tolerated_mismatch(
        &mut self,
        expected: &Arc<TypeNode>,
        found: &Arc<TypeNode>,
        source: &Option<Span>,
    ) -> Result<(), Errors> {
        if let Err(e) = UnifOrOtherErr::extract_others(self.unify(expected, found))? {
            if !self.error_tolerant {
                let err = self.create_type_mismatch_error(expected, found, &e, source);
                return Err(Errors::from_err(err));
            }
        }
        Ok(())
    }

    /// Validate `pat` and infer its type, returning the typed pattern
    /// and the variable bindings it introduces. Combines the two
    /// fallible steps into one `Result` so the caller can choose
    /// between propagating the error (strict mode) and substituting
    /// a fresh-tyvar pattern (`error_tolerant` mode).
    fn elaborate_pattern_binding(
        &mut self,
        pat: &Arc<PatternNode>,
    ) -> Result<(Arc<PatternNode>, Map<FullName, Arc<TypeNode>>), Errors> {
        self.validate_pattern(pat)?;
        pat.get_typed(self)
    }

    /// Combine `tolerate` with the canonical fresh-tyvar fallback for a
    /// failed pattern elaboration: on `Err` in `error_tolerant` mode,
    /// substitute a typed-throughout, empty-binding pattern so the
    /// surrounding walk can continue.
    pub fn tolerate_pattern_typed(
        &mut self,
        res: Result<(Arc<PatternNode>, Map<FullName, Arc<TypeNode>>), Errors>,
        pat: &Arc<PatternNode>,
    ) -> Result<(Arc<PatternNode>, Map<FullName, Arc<TypeNode>>), Errors> {
        Ok(self
            .tolerate(res)?
            .unwrap_or_else(|| (self.set_fallback_types_for_pattern(pat), Map::default())))
    }

    /// Assign a type to every node of `ei`: `ty` at the root and a fresh type
    /// variable at every descendant expression and pattern. The tolerant
    /// fallbacks substitute an unelaborated subtree for one whose elaboration
    /// failed, and every later tree walk relies on each node carrying a type
    /// (see `fix_types`), so the substitute is typed throughout.
    fn set_fallback_types(&mut self, ei: &Arc<ExprNode>, ty: Arc<TypeNode>) -> Arc<ExprNode> {
        let expr = self
            .map_types(
                ei,
                &mut |tc, e| Ok(tc.fresh_ty_with_src(&e.source)),
                &mut |tc, p| Ok(tc.fresh_ty_with_src(&p.info.source)),
            )
            .unwrap_or_else(|_| unreachable!("fresh-type callbacks cannot fail"));
        expr.set_type(ty)
    }

    /// Pattern counterpart of `set_fallback_types`: a fresh type variable at
    /// the pattern and at each of its sub-patterns.
    fn set_fallback_types_for_pattern(&mut self, pat: &Arc<PatternNode>) -> Arc<PatternNode> {
        self.map_types_for_pattern(pat, &mut |tc, p| Ok(tc.fresh_ty_with_src(&p.info.source)))
            .unwrap_or_else(|_| unreachable!("fresh-type callbacks cannot fail"))
    }

    /// Resolve `tc`, the head of a struct literal or of a struct
    /// pattern, to its struct definition. In strict mode, an unknown
    /// or non-struct tycon is an error; in tolerant mode it degrades
    /// to `None`, letting the caller fall back to fresh type
    /// variables for the fields.
    pub fn resolve_struct_tycon(
        &self,
        tc: &Arc<TyCon>,
        source: &Option<Span>,
        strict: bool,
    ) -> Result<Option<&TyConInfo>, Errors> {
        match self.type_env.tycons().get(tc) {
            Some(ti) if ti.variant == TyConVariant::Struct => Ok(Some(ti)),
            Some(_) if strict => Err(Errors::from_msg_srcs(
                format!("Type `{}` is not a struct.", tc.to_string()),
                &[source],
            )),
            None if strict => Err(Errors::from_msg_srcs(
                format!("Unknown type name `{}`.", tc.to_string()),
                &[source],
            )),
            _ => Ok(None),
        }
    }

    /// Unify the outer expected type with the constructed struct
    /// type, then return a `name -> field type` map the caller can
    /// look each provided field expression up in. Returns an empty
    /// map when `tycon_info` is `None` (the tolerant-mode degrade
    /// path), so the caller falls back to fresh tyvars for every
    /// field.
    fn compute_make_struct_field_tys(
        &mut self,
        tc: &Arc<TyCon>,
        tycon_info: Option<&TyConInfo>,
        expected_ty: &Arc<TypeNode>,
        source: &Option<Span>,
    ) -> Result<Map<Name, Arc<TypeNode>>, Errors> {
        let Some(ti) = tycon_info else {
            return Ok(Map::default());
        };
        let struct_ty = tc.get_struct_union_value_type(self);
        self.unify_or_tolerated_mismatch(expected_ty, &struct_ty, source)?;
        let field_tys = struct_ty.field_types(&self.type_env);
        Ok(ti
            .fields
            .iter()
            .zip(field_tys.iter())
            .map(|(f, ft)| (f.name.clone(), ft.clone()))
            .collect())
    }

    /// Type `expr` against `ty` with `var_ty`'s bindings pushed onto
    /// the scope for the duration of that call, then pop the
    /// bindings whether or not elaboration succeeded.
    fn unify_type_of_expr_with_scope(
        &mut self,
        expr: &Arc<ExprNode>,
        ty: Arc<TypeNode>,
        var_ty: &Map<FullName, Arc<TypeNode>>,
    ) -> Result<Arc<ExprNode>, Errors> {
        for (var_name, vt) in var_ty {
            assert!(var_name.is_local());
            self.scope
                .push(&var_name.name, Scheme::from_type(vt.clone()));
        }
        let result = self.unify_type_of_expr(expr, ty);
        for (var_name, _) in var_ty {
            self.scope.pop(&var_name.name);
        }
        result
    }

    /// Run `Pattern::validate_match_cases_exhaustiveness` on the
    /// arms of a typed `Match` when at least one arm was a union
    /// variant (signalled by `cond_tc_info.is_some()`). In
    /// `error_tolerant` mode a non-exhaustive match is swallowed
    /// so the typed tree still surfaces to downstream LSP consumers.
    fn validate_match_exhaustiveness_if_needed(
        &self,
        typed: &Arc<ExprNode>,
        cond_tc_info: Option<(Arc<TyCon>, TyConInfo)>,
    ) -> Result<(), Errors> {
        let Some((cond_tycon, cond_ti)) = cond_tc_info else {
            return Ok(());
        };
        let pats = typed.get_match_pat_vals().into_iter().map(|(pat, _)| pat);
        let res = Pattern::validate_match_cases_exhaustiveness(
            &cond_tycon,
            &cond_ti,
            &typed.source,
            pats,
        );
        if self.error_tolerant {
            Ok(())
        } else {
            res
        }
    }

    /// Validate a union-variant pattern for a `Match` arm: on the
    /// first union arm of the match, resolve the cond's TyCon and
    /// populate `cond_tc_info` so later arms can skip that step.
    /// Then check that `pat`'s variant name belongs to that union.
    fn validate_union_arm(
        &mut self,
        cond: &Arc<ExprNode>,
        cond_ty: &Arc<TypeNode>,
        pat: &Arc<PatternNode>,
        cond_tc_info: &mut Option<(Arc<TyCon>, TyConInfo)>,
    ) -> Result<Arc<PatternNode>, Errors> {
        if cond_tc_info.is_none() {
            *cond_tc_info = Some(self.resolve_match_cond_tycon(cond, cond_ty, pat)?);
        }
        let (tycon, ti) = cond_tc_info.as_ref().unwrap();
        pat.validate_variant_name(tycon, ti)
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
        let cond_ti = self.type_env.tycons().get(&cond_tycon).unwrap().clone();
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

    /// Gives `tv1` and `tv2` one source location, the expression a diagnostic naming either of
    /// them points at. Where both already carry one, `tv2`'s is the one kept.
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

    /// The import statements of the module the expression being checked belongs to.
    pub fn imported_statements(&self) -> &Vec<ImportStatement> {
        self.import_statements
            .get(&self.current_module.as_ref().unwrap().name)
            .unwrap()
    }

    /// A name no type variable of this context has taken, such as `#a3`. The `#` keeps it apart
    /// from every name a program can write.
    pub fn new_tyvar_name(&mut self) -> String {
        let id = self.tyvar_id;
        self.tyvar_id += 1;
        "#a".to_string() + &id.to_string()
    }

    /// A type variable of the given kind that no type of this context names yet.
    pub fn new_tyvar(&mut self, kind: Arc<Kind>) -> Arc<TyVar> {
        let name = self.new_tyvar_name();
        make_tyvar(&name, &kind)
    }

    /// A type variable of kind `*` that no type of this context names yet.
    pub fn new_tyvar_star(&mut self) -> Arc<TyVar> {
        self.new_tyvar(kind_star())
    }

    /// A type variable of the same kind as `tv`, under a name that no type of this context names
    /// yet.
    pub fn new_tyvar_by(&mut self, tv: &Arc<TyVar>) -> Arc<TyVar> {
        tv.set_name(self.new_tyvar_name())
    }

    /// Replaces each type variable of `ty` that the inference has settled by what it settled on.
    pub fn substitute_type(&self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        self.substitution.substitute_type(ty)
    }

    /// Replaces each type variable of `ty` that the inference has settled, and then replaces each
    /// associated type in the result that the equalities in force decide the value of.
    pub fn substitute_and_reduce_type(
        &mut self,
        ty: &Arc<TypeNode>,
    ) -> Result<Arc<TypeNode>, Errors> {
        let ty = self.substitute_type(ty);
        self.reduce_type_by_equality(ty)
    }

    /// Replaces the type variables the inference has settled in the type `p` constrains.
    pub fn substitute_predicate(&self, p: &mut Predicate) {
        self.substitution.substitute_predicate(p)
    }

    /// Replaces the type variables the inference has settled throughout `eq`.
    pub fn substitute_equality(&self, eq: &mut Equality) {
        self.substitution.substitute_equality(eq)
    }

    /// Writes into each resolution of an opaque type the type the inference found for it.
    ///
    /// Where `#Std::repeat::?it` was instantiated to a type variable that unification sent to
    /// `MapIterator (RangeIterator I64) a`, every resolution recorded for `Std::repeat`'s opaque
    /// type constructor comes out holding that type.
    pub fn fill_opaque_concrete_types(
        &mut self,
        opaque_types: &mut Map<FullName, Vec<OpaqueTyConResolution>>,
    ) {
        let instantiations = self.opaque_instantiations.clone();
        for (k, v) in instantiations {
            let fullname_str = k.strip_prefix(WRAP_OPAQUE_TYVAR_PREFIX).unwrap();
            let opaque_tycon_name = FullName::parse(fullname_str).unwrap();
            let rhs = self
                .substitute_and_reduce_type(&type_from_tyvar(v))
                .unwrap_or_else(|_| panic!("failed to reduce opaque type rhs"));
            if let Some(resolutions) = opaque_types.get_mut(&opaque_tycon_name) {
                for resolution in resolutions {
                    assert!(resolution.rhs.is_none(), "opaque type rhs already filled");
                    resolution.rhs = Some(rhs.clone());
                }
            }
        }
    }

    /// The substitution that sends each of `tyvars` to a fresh type variable of the same kind.
    fn instantiate_tyvars(&mut self, tyvars: &[Arc<TyVar>]) -> Substitution {
        let mut sub = Substitution::default();
        for tv in tyvars {
            let new_tv = type_from_tyvar(self.new_tyvar_by(tv));
            let merge_ok = sub.merge(&Substitution::single(&tv.name, new_tv));
            assert!(merge_ok);
        }
        sub
    }

    /// Replaces every free type variable of `ty` by a fresh one of the same kind.
    pub fn instantiate_type(&mut self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        let sub = self.instantiate_tyvars(&ty.free_vars_vec());
        sub.substitute_type(ty)
    }

    /// The type of `scheme`, with the constraints it is qualified by recorded in this context.
    ///
    /// # Arguments
    /// * `constraint_mode` — `Require` replaces each variable the scheme generalizes by a fresh
    ///   type variable and leaves the constraints for the inference to deduce; `Assume` keeps the
    ///   generalized variables as the scheme writes them, fixes them against replacement, and
    ///   grants the constraints to later deductions.
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
                    let merge_ok = sub.merge(&Substitution::single(
                        &tv.name,
                        type_from_tyvar(new_tv.clone()),
                    ));
                    assert!(merge_ok);
                    // Record opaque-type gen_vars (prefixed with WRAP_OPAQUE_TYVAR_PREFIX)
                    // so their concrete types can be extracted after type-checking.
                    if tv.name.starts_with(WRAP_OPAQUE_TYVAR_PREFIX) {
                        assert!(
                            !self.opaque_instantiations.contains_key(&tv.name),
                            "Duplicate opaque type variable name: {}",
                            tv.name
                        );
                        self.opaque_instantiations
                            .insert(tv.name.clone(), new_tv.clone());
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
                    insert_to_map_vec(
                        Arc::make_mut(&mut self.assumed_preds),
                        &trait_id,
                        qual_pred_scm,
                    );
                }
                for eq in eqs {
                    let assoc_ty = eq.assoc_type.clone();
                    let eq_scm = EqualityScheme {
                        gen_vars: vec![],
                        equality: eq.clone(),
                    };
                    insert_to_map_vec(Arc::make_mut(&mut self.assumed_eqs), &assoc_ty, eq_scm);
                    self.local_assumed_eqs.push(eq);
                }
                return Ok(scheme.ty.clone());
            }
        }
    }

    /// The type an annotation written in the source stands for: every `_` wildcard replaced by a
    /// fresh type variable for the inference to settle, and every named type variable by the
    /// generalized variable of that name, which carries the kind its signature gives it.
    ///
    /// A named type variable has to be one the value being checked generalizes; any other name is
    /// an error.
    pub fn validate_type_annotation(
        &mut self,
        ty: &Arc<TypeNode>,
    ) -> Result<Arc<TypeNode>, Errors> {
        let mut sub = Substitution::default();
        for tv in ty.free_vars_vec() {
            let target = if is_type_wildcard_tyvar(&tv.name) {
                // A `_` type wildcard asks for this type to be inferred: replace it
                // with a fresh inference variable, keeping the wildcard's kind, so
                // that it unifies freely.
                self.new_tyvar_by(&tv)
            } else if let Some(fixed_tv) = self
                .fixed_tyvars
                .iter()
                .find(|fixed_tv| fixed_tv.name == tv.name)
            {
                // A named type variable must be one generalized by the current
                // scheme. Substitute the fixed variable to carry over its kind.
                fixed_tv.clone()
            } else {
                return Err(Errors::from_msg_srcs(
                    format!("Unknown type variable `{}`.", tv.name),
                    &[&ty.get_source()],
                ));
            };
            let merge_ok = sub.merge(&Substitution::single(&tv.name, type_from_tyvar(target)));
            assert!(merge_ok);
        }

        Ok(sub.substitute_type(ty))
    }

    /// Perform typechecking: update the type substitution so that `ei` has
    /// type `ty`, and return the given AST augmented with inferred
    /// information.
    pub fn unify_type_of_expr(
        &mut self,
        ei: &Arc<ExprNode>,
        ty: Arc<TypeNode>,
    ) -> Result<Arc<ExprNode>, Errors> {
        grow_stack(|| {
            let ty_for_fallback = ty.clone();
            match self.unify_type_of_expr_inner(ei, ty) {
                Ok(e) => Ok(e),
                Err(_) if self.error_tolerant => {
                    // Swallow the failure and substitute the original
                    // subtree — the expected type at its root, fresh
                    // type variables below — so enclosing elaboration
                    // can keep going on sibling nodes.
                    Ok(self.set_fallback_types(ei, ty_for_fallback))
                }
                Err(errs) => Err(errs),
            }
        })
    }

    /// Elaborate `ei` against the expected type `ty`, one arm per `Expr`
    /// variant, returning the expression annotated with its inferred type.
    /// Each arm tolerates what it can in `error_tolerant` mode; what it
    /// cannot is raised as an error.
    ///
    /// **An arm elaborates its sub-expressions in the order the source
    /// writes them.** Elaborating one sub-expression leaves the type
    /// checker changed — a type variable bound, a predicate or an
    /// equality added to the pending ones — so each sub-expression is
    /// checked knowing what the ones before it settled, and type
    /// information flows from earlier sub-expressions to later ones.
    ///
    /// Programmers write with that flow in mind: the sub-expression
    /// that settles a type goes first and the one that needs it
    /// follows. An annotation is one way to settle it (`f(y : T, y.g)`),
    /// and an expression whose type is already known is another
    /// (`f(g(y), y.h)`, where `g`'s parameter type settles `y`). Where
    /// the type so settled is what lets an overloaded name pick one of
    /// its candidates, the order decides whether the program compiles,
    /// so changing it rejects programs that compile today.
    ///
    /// `Expr::App` is the compiler's own use of the rule: it elaborates
    /// the function before the argument, and elaborates the argument
    /// first where the source wrote the call as `x.f`, so that the
    /// receiver's type reaches the resolution of `f`.
    ///
    /// An arm that holds its sub-expressions in another order for a
    /// later stage — as the `Expr::MakeStruct` arm holds them in the
    /// struct's declaration order for code generation — reorders them
    /// after the walk rather than before it.
    // PROOF: D/A (dev-docs/proof/rc_ir/borrow-cancel)
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
                    err.data = Some(Value::String(var.name.to_string()));
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

                    let err_count = candidates_check_res
                        .iter()
                        .filter(|cand| cand.is_err())
                        .count();
                    let expected_type = self.substitute_type(&ty);
                    let msg = if err_count == 1 {
                        let (tc, fullname, scm, e) = candidates_check_res
                            .iter()
                            .find_map(|cand| cand.as_ref().err())
                            .unwrap();
                        let scm = tc.substitution.substitute_scheme(scm);
                        let msg = e.message_with_note(format!(
                            "`{}` of type `{}` does not match the expected type `{}` since `{}` cannot be deduced.",
                            fullname.to_string(),
                            scm.to_string(),
                            expected_type.to_string(),
                            e.to_constraint_string(),
                        ));
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
                            let ref_no = candidates_errors.len() + 1;
                            let scm = tc.substitution.substitute_scheme(scm);
                            let msg = e.message_with_note(format!(
                                "- ({}) `{}` of type `{}` does not match since `{}` cannot be deduced.",
                                ref_no,
                                fullname.to_string(),
                                scm.to_string(),
                                e.to_constraint_string(),
                            ));
                            candidates_errors.push(msg);
                            let mut tvs = vec![];
                            scm.free_vars_to_vec(&mut tvs);
                            e.free_vars_to_vec(&mut tvs);
                            extra_srcs.append(
                                &mut self.create_tyvar_location_messages(&tvs, Some(ref_no)),
                            );
                        }
                        if candidates_errors.len() > 0 {
                            msg.push_str("\n");
                            msg.push_str(&candidates_errors.join("\n"));
                        }
                        msg
                    };
                    let mut error = Error::from_msg_srcs(msg, &[&ei.source]);
                    error.code = Some(ERR_NO_VALUE_MATCH);
                    error.data = Some(Value::String(var.name.to_string()));
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
                    err.data = Some(Value::Array(
                        candidates
                            .iter()
                            .map(|name| Value::String(name.to_string()))
                            .collect(),
                    ));
                    return Err(Errors::from_err(err));
                } else {
                    // candidates.len() == 1
                    let (tc, ns) = candidates_check_res
                        .into_iter()
                        .find_map(|cand| cand.ok())
                        .unwrap();
                    *self = tc;
                    let ei = ei.set_var_namespace(ns);
                    let name = &ei.get_var().name;
                    if name.is_global() && !name.is_absolute() {
                        self.import_required.push(name.clone());
                    }
                    Ok(ei)
                }
            }
            Expr::LLVM(lit) => {
                self.unify_or_tolerated_mismatch(&ty, &lit.generic_ty, &ei.source)?;
                Ok(ei.clone())
            }
            Expr::App(fun, args) => {
                assert_eq!(args.len(), 1); // lambda of multiple arguments generated in optimization.
                let arg = args[0].clone();
                let arg_tv = self.new_tyvar_star();
                self.add_tyvar_source(arg_tv.name.clone(), arg.source.clone());
                let arg_ty = type_from_tyvar(arg_tv);
                // The source's order, as this function's comment requires: a
                // call written `x.f` elaborates `x` first, so the receiver's
                // type is settled when `f` is resolved and an overloaded `f`
                // can pick the candidate that takes it.
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
                // In `error_tolerant` mode, swallowing the mismatch
                // lets us continue typing the body against `body_ty`
                // (a fresh tyvar), so the body still gets a
                // best-effort type.
                self.unify_or_tolerated_mismatch(&ty, &fun_ty, &ei.source)?;
                assert!(arg.name.is_local());
                self.scope.push(&arg.name.name, Scheme::from_type(arg_ty));
                let body = self.unify_type_of_expr(body, body_ty)?;
                self.scope.pop(&arg.name.name);
                Ok(ei.set_lam_body(body))
            }
            Expr::Let(pat, bound, val) => {
                // Pattern elaboration may fail on a malformed pattern
                // (unknown struct field, duplicate variable,
                // sub-pattern type mismatch). In
                // `error_tolerant` mode we still want to elaborate
                // `bound` and `val` so any nested cursor inside them
                // gets a useful type — fall back to a fresh-tyvar
                // pattern with no variable bindings.
                let elab = self.elaborate_pattern_binding(pat);
                let (pat, var_ty) = self.tolerate_pattern_typed(elab, pat)?;
                let bound =
                    self.unify_type_of_expr(bound, pat.info.type_.as_ref().unwrap().clone())?;
                let val = self.unify_type_of_expr_with_scope(val, ty, &var_ty)?;
                Ok(ei.set_let_pat(pat).set_let_bound(bound).set_let_value(val))
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
                        // In `error_tolerant` mode a failed variant
                        // check is swallowed and the arm keeps the
                        // unvalidated pattern; the pattern elaboration
                        // below can still type its sub-patterns from
                        // the variant's signature.
                        let validated =
                            self.validate_union_arm(&cond, &cond_ty, pat, &mut cond_tc_info);
                        self.tolerate(validated)?.unwrap_or_else(|| pat.clone())
                    } else {
                        // `pat` is not a union pattern, so we can use it as is.
                        otherwise = Some(pat.clone());
                        pat.clone()
                    };

                    // Type the pattern, then unify with cond. In
                    // `error_tolerant` mode sub-pattern type mismatches
                    // are already tolerated inside the elaboration
                    // itself; a pattern that still fails to elaborate
                    // (e.g. its shape cannot be validated) falls back
                    // to a fresh-tyvar pattern with no bindings.
                    let elab = self.elaborate_pattern_binding(&pat);
                    let (pat, var_ty) = self.tolerate_pattern_typed(elab, &pat)?;
                    let pat_ty = pat.info.type_.as_ref().unwrap().clone();
                    self.unify_or_tolerated_mismatch(&pat_ty, &cond_ty, &pat.info.source)?;

                    // Type the arm's value with the pattern's
                    // bindings in scope.
                    let val = self.unify_type_of_expr_with_scope(val, ty.clone(), &var_ty)?;
                    new_pat_vals.push((pat, val));
                }

                // Build the typed Match before the exhaustiveness
                // check so the typed tree survives even when the
                // check is swallowed in `error_tolerant` mode.
                let typed = ei.set_match_cond(cond).set_match_pat_vals(new_pat_vals);
                self.validate_match_exhaustiveness_if_needed(&typed, cond_tc_info)?;
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
                // In `error_tolerant` mode an ill-formed annotation
                // (e.g. one naming an unknown type variable) is
                // dropped, and the child is elaborated against the
                // contextual type.
                let validated = self.validate_type_annotation(&anno_ty);
                let anno_ty = self.tolerate(validated)?.unwrap_or_else(|| ty.clone());
                // After a successful unify, `ty` and `anno_ty` are
                // substitution-equivalent, so either could be the
                // child's expected type. After a tolerated mismatch
                // they diverge — honour `anno_ty` (the user's stated
                // intent) for the child either way.
                self.unify_or_tolerated_mismatch(&ty, &anno_ty, &ei.source)?;
                let e = self.unify_type_of_expr(e, anno_ty)?;
                Ok(ei.set_tyanno_expr(e))
            }
            Expr::MakeStruct(tc, fields) => {
                let strict = !self.error_tolerant;

                // 1. Resolve `tc` to its struct definition. Strict
                // mode errors out on unknown / non-struct names;
                // tolerant degrades to `None` so we can still type
                // each field expression against a fresh tyvar.
                // The definition is taken by value because the steps below
                // borrow the type checker mutably.
                let tycon_info = self.resolve_struct_tycon(tc, &ei.source, strict)?.cloned();

                // 2. Strict-only: pair each field with the declared
                // field it names, reporting a repeat, an omission and
                // an unknown name. The answer is the list in
                // declaration order, which step 5 takes; this call
                // takes only the errors, so that a field list the
                // struct cannot accept is reported before any field
                // expression is elaborated. Tolerant mode skips the
                // check so the user can keep typing inside a partially
                // written struct literal.
                if strict {
                    let ti = tycon_info
                        .as_ref()
                        .expect("strict mode resolves the head to a struct or reports an error");
                    make_struct_fields_in_declaration_order(ti, tc, fields, &ei.source)?;
                }

                // 3. Compute the `name -> expected field type` map
                // (after unifying the outer expected type with the
                // constructed struct type). An empty map when the
                // tycon didn't resolve, leaving every field
                // expression to be typed against a fresh tyvar.
                let known_field_tys =
                    self.compute_make_struct_field_tys(tc, tycon_info.as_ref(), &ty, &ei.source)?;

                // 4. Type each field expression, in the order the
                // literal writes them — see this function's own comment
                // for why the order is the source's. The expected type
                // comes from the field's name, so this walk needs no
                // help from the declaration's order.
                let mut typed_fields = fields.clone();
                for (name, _, field_expr) in typed_fields.iter_mut() {
                    let field_ty = match known_field_tys.get(name) {
                        Some(field_ty) => field_ty.clone(),
                        None => {
                            // No expected type to check the field expression against: the head
                            // names no struct, or the struct has no such field.
                            // `make_struct_fields_in_declaration_order` rejects both in strict
                            // mode and tolerates them in `error_tolerant` mode.
                            assert!(
                                self.error_tolerant,
                                "struct `{}` has no field `{}`",
                                tc.to_string(),
                                name
                            );
                            self.fresh_ty_with_src(&field_expr.source)
                        }
                    };
                    *field_expr = self.unify_type_of_expr(field_expr, field_ty)?;
                }

                // 5. Strict-only: put the typed fields in declaration
                // order, which is the order code generation reads the
                // values in. This is the walk step 2 ran, on the same
                // names, so it answers the same way; running it again
                // here is what lets step 4 walk the source's order.
                // Tolerant mode keeps the list as written — the
                // resulting typed tree may be structurally ill-formed
                // for codegen, but tolerant elaborates aren't fed to it.
                if strict {
                    let ti = tycon_info
                        .as_ref()
                        .expect("strict mode resolves the head to a struct or reports an error");
                    typed_fields =
                        make_struct_fields_in_declaration_order(ti, tc, &typed_fields, &ei.source)?;
                }

                Ok(ei.set_make_struct_fields(typed_fields))
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
                // In `error_tolerant` mode, swallowing the mismatch
                // lets us continue typing each element against the
                // fresh `elem_ty`, so subtrees still get an inferred
                // type even when the outer expected type isn't an
                // array.
                self.unify_or_tolerated_mismatch(&ty, &array_ty, &ei.source)?;
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
                // In `error_tolerant` mode, swallowing the mismatch
                // lets us continue typing each argument against the
                // declared parameter type, so subtrees keep their
                // inferred type even when the outer expected return
                // type doesn't match the FFI signature.
                self.unify_or_tolerated_mismatch(&ty, &ret_ty, &ei.source)?;
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

    /// Reject a pattern the elaboration cannot make sense of: a struct head
    /// that names no struct, an unknown or duplicated field name, an
    /// ill-formed type annotation, or a variable name bound twice. Recurses
    /// into the sub-patterns.
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
                // The head has to name a struct: the sub-patterns are matched against that
                // struct's fields, and the value is destructured in its field order.
                let tycon_info = self.resolve_struct_tycon(tc, &pat.info.source, !tolerate)?;
                if !tolerate {
                    // Every way the field list is wrong is reported together, as it is for a
                    // struct literal.
                    let mut errors = duplicate_field_errors(tc, pats);
                    if let Some(ti) = tycon_info {
                        let struct_field_names =
                            ti.fields.iter().map(|f| f.name.clone()).collect::<Set<_>>();
                        for (name, name_src, _) in pats {
                            if !struct_field_names.contains(name) {
                                errors.append(unknown_field_error(tc, name, name_src));
                            }
                        }
                    }
                    errors.to_result()?;
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

    /// Say where each of `tvs` came from: for every type variable whose source
    /// expression is known, a sentence naming it paired with that expression's
    /// span, to hang off an error as extra source pointers.
    ///
    /// # Arguments
    /// * `ref_no` — when the error text refers to several types by number, the
    ///   number of the one these variables belong to; it is printed alongside
    ///   each variable's name.
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

    /// Build the "Type mismatch" error pointed at `source`: `expected_ty` and
    /// `found_ty` with the current substitution applied, the constraint of
    /// `unif_err` that could not be deduced, and a source pointer for every
    /// type variable still free in them.
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
            unif_err.message_with_note(format!(
                "Type mismatch. Expected `{}`, found `{}`. They do not match since `{}` cannot be deduced.",
                expected_ty.to_string(),
                found_ty.to_string(),
                unif_err.to_constraint_string(),
            )),
            &[&source],
        );
        err.add_srcs(tv_loc_msgs);
        err
    }

    /// Panics unless no inference has run in this context yet: no type variable issued, an empty
    /// substitution, and no pending predicate, equality, fixed type variable or required import.
    pub fn assert_freshness(&self) {
        assert!(self.tyvar_id == 0);
        assert!(self.substitution.is_empty());
        assert!(self.predicates.is_empty());
        assert!(self.equalities.is_empty());
        assert!(self.local_assumed_eqs.is_empty());
        assert!(self.fixed_tyvars.is_empty());
        assert!(self.import_required.is_empty());
    }

    /// Checks that `lhs` and `rhs` describe the same values: each of the two, instantiated under
    /// the constraints it states, has the type of the other and meets what the other requires.
    ///
    /// The context has to be one in which no inference has run.
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

    /// Checks that `rhs` is at least as general as `lhs`: assuming what `lhs` states, the type of
    /// `rhs` unifies with the type of `lhs`, and every constraint `rhs` requires is deduced.
    ///
    /// The inference this performs stays in the context, so the caller gives it a context it uses
    /// for this check alone.
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

        /// Build the error reported when a constraint required by the
        /// inference cannot be deduced from the assumptions, attaching
        /// source-location notes for the type variables it mentions.
        fn make_error(
            tc: &TypeCheckContext,
            mut unif_err: UnificationErr,
            src: &Option<Span>,
        ) -> Error {
            tc.substitution.substitute_unification_error(&mut unif_err);
            let mut error = Error::from_msg_srcs(
                unif_err.message_with_note(format!(
                    "`{}` is required in the type inference of this expression but cannot be deduced from assumptions.",
                    unif_err.to_constraint_string()
                )),
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

        // The tolerant path's only invariant is "every node has an
        // inferred type". `check_all_typed` verifies it here, before
        // `fix_types` reads the types and aborts the process on a
        // missing one, so a fallback that leaves part of the tree
        // untyped surfaces as a failed request.
        if self.error_tolerant {
            self.check_all_typed(&expr)?;
        }

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
        if self.error_tolerant {
            return Ok((expr, Errors::empty()));
        }

        // Pre-extract the source span so the error-construction
        // helpers below can borrow it independently of `expr` (which
        // each early-return consumes).
        let src = expr.source.clone();

        // Layer 1: holes.
        let hole_errors = collect_hole_errors(&expr, self);
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

    /// Composes `subst` into the accumulated substitution, then re-examines the
    /// pending equalities, which the new bindings may let unify or reduce.
    fn add_substitution(&mut self, subst: &Substitution) -> Result<(), UnifOrOtherErr> {
        self.substitution.compose(subst);
        let eqs = mem::replace(&mut self.equalities, vec![]);
        for eq in eqs {
            self.add_equality(eq)?;
        }
        Ok(())
    }

    /// Records `eq` among the pending equalities, once neither of its sides can be simplified any
    /// further. Where the accumulated substitution or the known equalities do simplify a side, the
    /// two sides are unified instead, and an equality whose sides came out equal is dropped.
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

    /// Replaces each use of an associated type in `ty` by the value an assumed equality gives it,
    /// as deep as the assumed equalities reach, and leaves the rest of the type as it stands.
    ///
    /// An associated type met on the way also requires the trait that declares it of its first
    /// argument, so that predicate joins the pending ones.
    fn reduce_type_by_equality(&mut self, ty: Arc<TypeNode>) -> Result<Arc<TypeNode>, Errors> {
        self.reduce_type_by_equality_inner(ty, &mut TypeReduction::default())
    }

    /// The body of `reduce_type_by_equality`.
    ///
    /// # Arguments
    /// * `reduction` — the associated types whose reduction this one is inside. Meeting one of
    ///   them again is meeting a type whose reduction needs itself.
    fn reduce_type_by_equality_inner(
        &mut self,
        ty: Arc<TypeNode>,
        reduction: &mut TypeReduction,
    ) -> Result<Arc<TypeNode>, Errors> {
        match &ty.ty {
            Type::TyVar(_) => Ok(ty),
            Type::TyCon(_) => Ok(ty),
            Type::TyApp(tyfun, tyarg) => {
                let tyfun = self.reduce_type_by_equality_inner(tyfun.clone(), reduction)?;
                let tyarg = self.reduce_type_by_equality_inner(tyarg.clone(), reduction)?;
                Ok(ty.set_tyapp_fun(tyfun).set_tyapp_arg(tyarg))
            }
            Type::AssocTy(assoc_ty, args) => {
                // Reduce each arguments.
                let args = collect_results(
                    args.iter()
                        .map(|arg| self.reduce_type_by_equality_inner(arg.clone(), reduction)),
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

                // An equality gives a value that this reduction takes in the type's place, and that
                // value can name the type again. Reducing the value then asks for a reduction
                // that has begun and has yet to end, and the report names the equalities that lead
                // from the type back to itself. A reduction that instead keeps finding types it
                // has not met yet ends at `MAX_TYPE_DEPTH`.
                if reduction.on_path.contains(&ty) {
                    return Err(reduction.circular_error(&ty));
                }
                if ty.depth() > MAX_TYPE_DEPTH {
                    return Err(reduction.endless_error(&ty));
                }

                // Try matching to assumed equality.
                let assumed_eqs = self.assumed_eqs.clone();
                for assumed_eq in assumed_eqs.get(assoc_ty).map_or(&[][..], Vec::as_slice) {
                    // Instantiate `assumed_eq`.
                    let inst_subst = self.instantiate_tyvars(&assumed_eq.gen_vars);
                    let mut equality = assumed_eq.equality.clone();
                    inst_subst.substitute_equality(&mut equality);

                    // Try to match lhs of `equality` to `ty`.
                    let match_subst: Option<Substitution> = Substitution::matching(
                        &equality.lhs(),
                        &ty,
                        &self.fixed_tyvars,
                        &self.kind_env,
                    )?;
                    if match_subst.is_none() {
                        continue;
                    }
                    let match_subst: Substitution = match_subst.unwrap();
                    let rhs = match_subst.substitute_type(&equality.value);
                    reduction.enter(&ty, &equality.src);
                    let reduced = self.reduce_type_by_equality_inner(rhs, reduction);
                    reduction.leave();
                    return reduced;
                }
                Ok(ty)
            }
        }
    }

    /// Makes `ty1` and `ty2` one type, extending the accumulated substitution with the bindings
    /// that takes.
    ///
    /// A type variable free to be bound takes the other type as its value. A type variable held
    /// fixed stands for a type the caller may not choose, so it agrees with itself alone. A use of
    /// an associated type on either side becomes a pending equality, to be settled once enough is
    /// known about its arguments. Two types no substitution can make equal give
    /// `UnificationErr::Disjoint`.
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
            mem::swap(&mut ty1, &mut ty2);
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
            mem::swap(&mut ty1, &mut ty2);
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

    /// Whether unification of `ty1` and `ty2` succeeds under the current
    /// substitution, assumed equalities and fixed type variables.
    ///
    /// Both types are substituted here, and the unification then runs on an
    /// empty substitution of its own: a substitution maps each of its variables
    /// to a type free of them all, so it has nothing left to say about its own
    /// image. That keeps the query proportional to the two types while the
    /// inference state grows with the body being checked.
    pub fn are_unifiable(&self, ty1: &Arc<TypeNode>, ty2: &Arc<TypeNode>) -> Result<bool, Errors> {
        let ty1 = self.substitute_type(ty1);
        let ty2 = self.substitute_type(ty2);
        let mut tc = Self {
            // What unification reads.
            tyvar_id: self.tyvar_id,
            equalities: self.equalities.clone(),
            fixed_tyvars: self.fixed_tyvars.clone(),
            assumed_eqs: self.assumed_eqs.clone(),
            kind_env: self.kind_env.clone(),
            // What unification writes, and the answer discards.
            substitution: Substitution::default(),
            tyvar_expr: Map::default(),
            predicates: vec![],
            // What unification leaves alone: the large ones start empty, the
            // ones that cost a scalar or a reference count are carried.
            scope: Scope::default(),
            import_required: vec![],
            local_assumed_eqs: vec![],
            opaque_instantiations: Map::default(),
            assumed_preds: self.assumed_preds.clone(),
            trait_env: self.trait_env.clone(),
            type_env: self.type_env.clone(),
            import_statements: self.import_statements.clone(),
            current_module: self.current_module.clone(),
            cache: self.cache.clone(),
            num_worker_threads: self.num_worker_threads,
            error_tolerant: self.error_tolerant,
        };
        Ok(UnifOrOtherErr::extract_others(tc.unify(&ty1, &ty2))?.is_ok())
    }

    /// Binds the type variable `tyvar1` to `ty2` by extending the substitution,
    /// rejecting a binding that would be circular or kind-mismatched.
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
        let mut irreducible_preds = vec![];
        let mut deduction = PredicateDeduction::default();
        while let Some(pred) = self.predicates.pop() {
            self.reduce_predicate(pred, &mut irreducible_preds, &mut deduction)?;
        }
        self.predicates = irreducible_preds;
        Ok(())
    }

    /// Deduces `pred`, a constraint by a trait or by a trait alias, from the instances and the
    /// constraints assumed, collecting into `irreducible_preds` each constraint the deduction ends
    /// at: one on a type the inference has yet to settle, which no instance decides yet.
    ///
    /// # Arguments
    /// * `deduction` — the deductions this one is inside and the ones already ended, which tell a
    ///   deduction needing itself from one that holds, and keep a constraint several instances ask
    ///   for from being deduced once for each of them.
    fn reduce_predicate(
        &mut self,
        pred: Predicate,
        irreducible_preds: &mut Vec<Predicate>,
        deduction: &mut PredicateDeduction,
    ) -> Result<(), UnifOrOtherErr> {
        for pred in pred.resolve_trait_aliases(&self.trait_env.aliases)? {
            self.reduce_predicate_noalias(pred, irreducible_preds, deduction)?;
        }
        Ok(())
    }

    /// Deduces `pred`, whose trait has to be one the program declares, every alias having been
    /// resolved, from the instances and the constraints assumed. Collects into `irreducible_preds`
    /// each constraint the deduction ends at: one on a type the inference has yet to settle, which
    /// no instance decides yet.
    fn reduce_predicate_noalias(
        &mut self,
        mut pred: Predicate,
        irreducible_preds: &mut Vec<Predicate>,
        deduction: &mut PredicateDeduction,
    ) -> Result<(), UnifOrOtherErr> {
        self.substitute_predicate(&mut pred);
        // The constraint as it is asked for, before the associated types in it are reduced, is what
        // tells one deduction from another here. Two spellings of one constraint are then two
        // deductions, which costs a turn of the deduction before a circle through an associated
        // type closes on a spelling it has already met. The depth bound below is what ends a
        // deduction that keeps finding new spellings.
        let pred_str = pred.to_string();
        if deduction.settled.contains(&pred_str) {
            return Ok(());
        }
        if deduction.on_path.contains(&pred_str) {
            return Err(UnificationErr::Circular(deduction.way_round(&pred_str)).into());
        }
        pred.ty = self.substitute_and_reduce_type(&pred.ty)?;
        // An instance whose context asks for what it gives, on a larger type, never asks twice for
        // the same predicate, so the check above never meets one of its deductions a second time.
        // What such a deduction does at every step is ask about a deeper type, and this is where
        // that ends.
        if pred.ty.depth() > MAX_TYPE_DEPTH {
            return Err(UnificationErr::Endless(deduction.way_down(&pred)).into());
        }
        let mut unifiable = false;
        let assumed_preds = self.assumed_preds.clone();
        for qual_pred_scm in assumed_preds
            .get(&pred.trait_id)
            .map_or(&[][..], Vec::as_slice)
        {
            // Instantiate qualified predicate.
            let inst_subst = self.instantiate_tyvars(&qual_pred_scm.gen_vars);
            let mut qual_pred = qual_pred_scm.qual_pred.clone();
            inst_subst.substitute_qualpred(&mut qual_pred);

            // Try to match head of `qual_pred` to `pred`.
            if let Some(match_subst) = Substitution::matching(
                &qual_pred.predicate.ty,
                &pred.ty,
                &self.fixed_tyvars,
                &self.kind_env,
            )? {
                for mut eq in qual_pred.eq_constraints {
                    match_subst.substitute_equality(&mut eq);
                    self.add_equality(eq)?;
                }
                let context = qual_pred
                    .pred_constraints
                    .into_iter()
                    .map(|mut ctx_pred| {
                        match_subst.substitute_predicate(&mut ctx_pred);
                        ctx_pred
                    })
                    .collect();
                self.reduce_instance_context(
                    &pred,
                    &pred_str,
                    context,
                    irreducible_preds,
                    deduction,
                )?;
                deduction.settled.insert(pred_str);
                return Ok(());
            } else if !unifiable {
                // If match fails, then we cannot reduce the predicate at now.
                // But we may be able to reduce it after the predicate is substituted further.
                // To see if there is possibility for further reduction, we check here the unifiability.
                // One instance head it unifies with settles the question, so the rest go unasked.
                unifiable = self.are_unifiable(&qual_pred.predicate.ty, &pred.ty)?;
            }
        }
        if !unifiable {
            return Err(UnificationErr::Unsatisfiable(pred).into());
        }
        irreducible_preds.push(pred);
        deduction.settled.insert(pred_str);
        return Ok(());
    }

    /// Deduce the constraints the instance that gives `pred` asks for, with `pred` on the deduction
    /// while they are deduced, so that a deduction coming back to `pred` is seen for what it is.
    fn reduce_instance_context(
        &mut self,
        pred: &Predicate,
        pred_str: &str,
        context: Vec<Predicate>,
        irreducible_preds: &mut Vec<Predicate>,
        deduction: &mut PredicateDeduction,
    ) -> Result<(), UnifOrOtherErr> {
        deduction.enter(pred, pred_str);
        let deduced = context
            .into_iter()
            .try_for_each(|ctx_pred| self.reduce_predicate(ctx_pred, irreducible_preds, deduction));
        deduction.leave();
        deduced
    }

    /// Pattern half of `map_types`: rebuild `pat` with the type of the
    /// pattern and of each sub-pattern recomputed by `pat_ty`.
    fn map_types_for_pattern<G>(
        &mut self,
        pat: &Arc<PatternNode>,
        pat_ty: &mut G,
    ) -> Result<Arc<PatternNode>, Errors>
    where
        G: FnMut(&mut Self, &Arc<PatternNode>) -> Result<Arc<TypeNode>, Errors>,
    {
        let ty = pat_ty(self, pat)?;
        let pat = pat.set_type(ty);
        Ok(match &pat.pattern {
            Pattern::Var(_var, _anno_ty) => {
                // The annotation type inside the pattern is left as parsed;
                // nothing downstream reads it.
                pat
            }
            Pattern::Union(_, _, subpat) => {
                let subpat = self.map_types_for_pattern(subpat, pat_ty)?;
                pat.set_union_pat(subpat)
            }
            Pattern::Struct(_, field_to_pat) => {
                let mut field_to_pat = field_to_pat.clone();
                for (_field_name, _, subpat) in field_to_pat.iter_mut() {
                    *subpat = self.map_types_for_pattern(subpat, pat_ty)?;
                }
                pat.set_struct_field_to_pat(field_to_pat)
            }
        })
    }

    /// The error reported where `ty` still names a type variable that neither the inference has
    /// settled nor the checked value generalizes, so that the source at `src` has no one type.
    ///
    /// # Arguments
    /// * `src_type` — what the source at `src` is, as the message names it: "expression",
    ///   "pattern", and so on. The message asks for a type annotation on it.
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

    /// Substitute the accumulated type substitution into `ty` and reduce
    /// associated types. Associated-type reduction can fail when the
    /// substitution / equality state is inconsistent — a normal consequence
    /// of the tolerant elaborator stitching together partially failed
    /// sub-expressions. On a tolerated failure, keep the un-reduced type so
    /// downstream consumers (LSP dot completion, hover) can still read
    /// whatever type info survived.
    fn substitute_and_reduce_type_or_keep(
        &mut self,
        ty: &Arc<TypeNode>,
    ) -> Result<Arc<TypeNode>, Errors> {
        let reduced = self.substitute_and_reduce_type(ty);
        Ok(self.tolerate(reduced)?.unwrap_or_else(|| ty.clone()))
    }

    /// Apply the type substitution to every node's `type_` field and to
    /// every pattern type. Does not check whether the resulting types
    /// are fixed (free of unsolved type variables); see
    /// `check_types_are_fixed` for that. Substitution and the
    /// fixed-type check are kept separate so other passes (e.g. hole
    /// detection) can run on the substituted AST in between.
    pub fn fix_types(&mut self, expr: Arc<ExprNode>) -> Result<Arc<ExprNode>, Errors> {
        self.map_types(
            &expr,
            &mut |tc, e| {
                let raw_ty = e
                    .type_
                    .as_ref()
                    .expect("fix_types: every node should be typed by unify_type_of_expr");
                tc.substitute_and_reduce_type_or_keep(raw_ty)
            },
            &mut |tc, p| {
                let raw_ty = p
                    .info
                    .type_
                    .as_ref()
                    .expect("fix_types: every pattern should be typed");
                tc.substitute_and_reduce_type_or_keep(raw_ty)
            },
        )
    }

    /// Rebuild `expr` with the type of every expression node recomputed by
    /// `expr_ty` and the type of every pattern node by `pat_ty`, recursing
    /// through all children. The walk is top-down: a node's type is computed
    /// before its children are rebuilt.
    fn map_types<F, G>(
        &mut self,
        expr: &Arc<ExprNode>,
        expr_ty: &mut F,
        pat_ty: &mut G,
    ) -> Result<Arc<ExprNode>, Errors>
    where
        F: FnMut(&mut Self, &Arc<ExprNode>) -> Result<Arc<TypeNode>, Errors>,
        G: FnMut(&mut Self, &Arc<PatternNode>) -> Result<Arc<TypeNode>, Errors>,
    {
        let ty = expr_ty(self, expr)?;
        let expr = expr.set_type(ty);
        Ok(match &*expr.expr {
            Expr::Var(_) => expr,
            Expr::LLVM(_) => expr,
            Expr::App(fun, args) => {
                let args =
                    collect_results(args.iter().map(|arg| self.map_types(arg, expr_ty, pat_ty)))?;
                let fun = self.map_types(fun, expr_ty, pat_ty)?;
                expr.set_app_func(fun).set_app_args(args)
            }
            Expr::Lam(_args, body) => {
                let body = self.map_types(body, expr_ty, pat_ty)?;
                expr.set_lam_body(body)
            }
            Expr::Let(pat, bound, val) => {
                let pat = self.map_types_for_pattern(pat, pat_ty)?;
                let bound = self.map_types(bound, expr_ty, pat_ty)?;
                let val = self.map_types(val, expr_ty, pat_ty)?;
                expr.set_let_pat(pat)
                    .set_let_bound(bound)
                    .set_let_value(val)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.map_types(cond, expr_ty, pat_ty)?;
                let then_expr = self.map_types(then_expr, expr_ty, pat_ty)?;
                let else_expr = self.map_types(else_expr, expr_ty, pat_ty)?;
                expr.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
            }
            Expr::Match(cond, pat_vals) => {
                let cond = self.map_types(cond, expr_ty, pat_ty)?;
                let mut new_pat_vals = vec![];
                for (pat, val) in pat_vals {
                    let pat = self.map_types_for_pattern(pat, pat_ty)?;
                    let val = self.map_types(val, expr_ty, pat_ty)?;
                    new_pat_vals.push((pat, val));
                }
                expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
            }
            Expr::TyAnno(e, _) => {
                let e = self.map_types(e, expr_ty, pat_ty)?;
                expr.set_tyanno_expr(e)
            }
            Expr::MakeStruct(_tc, fields) => {
                let mut fields = fields.clone();
                for (_, _, field_expr) in fields.iter_mut() {
                    *field_expr = self.map_types(field_expr, expr_ty, pat_ty)?;
                }
                expr.set_make_struct_fields(fields)
            }
            Expr::ArrayLit(elems) => {
                let elems =
                    collect_results(elems.iter().map(|e| self.map_types(e, expr_ty, pat_ty)))?;
                expr.set_array_lit_elems(elems)
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                let args =
                    collect_results(args.iter().map(|arg| self.map_types(arg, expr_ty, pat_ty)))?;
                expr.set_ffi_call_args(args)
            }
            Expr::Eval(side, main) => {
                let side = self.map_types(side, expr_ty, pat_ty)?;
                let main = self.map_types(main, expr_ty, pat_ty)?;
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
            Expr::Let(pat, bound, val) => {
                self.check_pattern_types_are_fixed(pat)?;
                self.check_types_are_fixed(bound)?;
                self.check_types_are_fixed(val)?;
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
    /// unresolved tyvars). Surfaces elaborator bugs that would
    /// otherwise crash downstream consumers expecting every node to
    /// be typed.
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
            Expr::Let(pat, bound, val) => {
                self.check_all_pattern_typed(pat)?;
                self.check_all_typed(bound)?;
                self.check_all_typed(val)?;
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

    /// Verifies that `pat` and each of its sub-patterns carries an inferred type, reporting the
    /// innermost one that carries none.
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

/// The report for a field name a struct literal or a struct pattern for the
/// struct `tc` gives twice, located at the repeat and carrying the first
/// occurrence of that name as a related location.
fn duplicate_field_error(
    tc: &Arc<TyCon>,
    name: &Name,
    name_src: &Option<Span>,
    first_src: &Option<Span>,
) -> Error {
    let mut err = Error::from_msg_srcs(
        format!("Duplicate field `{}` of struct `{}`.", name, tc.to_string()),
        &[name_src],
    );
    if let Some(first_src) = first_src {
        err.add_src(
            "The field is given here first.".to_string(),
            first_src.clone(),
        );
    }
    err
}

/// The report for a name the struct `tc` does not declare, located at the name.
fn unknown_field_error(tc: &Arc<TyCon>, name: &Name, name_src: &Option<Span>) -> Errors {
    Errors::from_msg_srcs(
        format!("Unknown field `{}` for struct `{}`.", name, tc.to_string()),
        &[name_src],
    )
}

/// The report for declared fields a struct literal leaves out, located at the
/// whole literal, which is where the editor's quick fix inserts them.
fn missing_fields_error(tc: &Arc<TyCon>, missing: &[Name], source: &Option<Span>) -> Error {
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
    let mut err = Error::from_msg_srcs(msg, &[source]);
    err.code = Some(ERR_MISSING_STRUCT_FIELD);
    err.data = Some(json!(missing));
    err
}

/// Report each field name that a struct pattern for the struct `tc` matches
/// more than once: a field list names each field of the struct exactly once.
///
/// # Examples
/// For the sub-patterns of `S { a : x, b : y, a : z }` this returns one error,
/// at the second `a`; for `S { a : x, b : y }` it returns none.
fn duplicate_field_errors(
    tc: &Arc<TyCon>,
    fields: &[(Name, Option<Span>, Arc<PatternNode>)],
) -> Errors {
    let mut errors = Errors::empty();
    let mut first_srcs: Map<Name, Option<Span>> = Map::default();
    for (name, name_src, _) in fields {
        let Some(first_src) = first_srcs.get(name).cloned() else {
            first_srcs.insert(name.clone(), name_src.clone());
            continue;
        };
        errors.append(Errors::from_err(duplicate_field_error(
            tc, name, name_src, &first_src,
        )));
    }
    errors
}

/// Pair each field of an `Expr::MakeStruct` literal with the declared field of
/// `ti` it names, and answer with the list in declaration order.
///
/// Code generation reads a struct's field values by position
/// (`rc_ir/lower.rs::lower_make_struct` lowers the values in list order), so
/// the literal reaches it in the order the struct declares its fields even
/// where the user wrote them in another order.
///
/// A field list names each declared field exactly once, and every way it fails
/// to is reported: a name the struct doesn't declare, a name given twice, and a
/// declared field left out. All of them come back together, so one compilation
/// shows every way the list is wrong.
///
/// # Examples
/// For `S { b : 2, a : 1 }` on `type S = struct { a : I64, b : I64 };` this
/// answers with `a` before `b`; for `S { a : 1, a : 2 }` it reports the repeat
/// and the missing `b`.
fn make_struct_fields_in_declaration_order(
    ti: &TyConInfo,
    tc: &Arc<TyCon>,
    fields: &[(Name, Option<Span>, Arc<ExprNode>)],
    source: &Option<Span>,
) -> Result<Vec<(Name, Option<Span>, Arc<ExprNode>)>, Errors> {
    let mut errors = Errors::empty();
    let name_to_idx: Map<&Name, usize> = ti
        .fields
        .iter()
        .enumerate()
        .map(|(idx, f)| (&f.name, idx))
        .collect();
    let mut slots: Vec<Option<(Name, Option<Span>, Arc<ExprNode>)>> =
        (0..ti.fields.len()).map(|_| None).collect();
    for field in fields {
        let (name, name_src, _) = field;
        let Some(&idx) = name_to_idx.get(name) else {
            errors.append(unknown_field_error(tc, name, name_src));
            continue;
        };
        match &slots[idx] {
            Some((_, first_src, _)) => errors.append(Errors::from_err(duplicate_field_error(
                tc, name, name_src, first_src,
            ))),
            None => slots[idx] = Some(field.clone()),
        }
    }

    // Draining the slots yields the fields in declaration order and the names of
    // the declared fields no field claimed, so the answer is as long as the
    // declaration whenever nothing is missing.
    let mut ordered = Vec::with_capacity(ti.fields.len());
    let mut missing = Vec::new();
    for (declared, slot) in ti.fields.iter().zip(slots) {
        match slot {
            Some(field) => ordered.push(field),
            None => missing.push(declared.name.clone()),
        }
    }
    if !missing.is_empty() {
        errors.append(Errors::from_err(missing_fields_error(tc, &missing, source)));
    }

    errors.to_result()?;
    Ok(ordered)
}

/// Returns the trimmed source text covered by `span` if it fits on a single line and within a small character budget, suitable for inlining into a diagnostic message.
fn short_span_snippet(span: &Span) -> Option<String> {
    /// The longest snippet a message quotes. Long enough for a field name or a small expression,
    /// short enough to leave the message readable.
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

/// What the deduction of a predicate carries as it descends into the constraints that the instances
/// it uses ask for.
///
/// A predicate is deduced by finding the instance whose head it matches and deducing what that
/// instance's context asks for. Which of the two sets a predicate is in is what separates one that
/// holds from one whose deduction needs itself: `on_path` holds the predicates whose deduction has
/// begun and has yet to end, and `settled` the ones whose deduction ended.
#[derive(Default)]
struct PredicateDeduction {
    /// The predicates whose deduction the current one is inside, outermost first, each with its
    /// printed form. The printed form is what tells one predicate from another here, and it is kept
    /// so that the way out costs no more printing than the way in did.
    path: Vec<(Predicate, String)>,
    /// The printed form of each predicate on `path`. Meeting one of these again is meeting a
    /// predicate whose deduction needs itself.
    on_path: Set<String>,
    /// The printed form of each predicate whose deduction ended. Recorded on the way out, so that a
    /// predicate the deduction is still inside is not taken for one already deduced. It keeps a
    /// predicate that several instance contexts ask for from being deduced once for each of them.
    settled: Set<String>,
}

impl PredicateDeduction {
    /// Record that the deduction of `pred`, whose printed form is `pred_str`, has begun.
    fn enter(&mut self, pred: &Predicate, pred_str: &str) {
        self.on_path.insert(pred_str.to_string());
        self.path.push((pred.clone(), pred_str.to_string()));
    }

    /// Record that the deduction entered last has ended.
    fn leave(&mut self) {
        let (_pred, pred_str) = self
            .path
            .pop()
            .expect("a deduction ends only after it has begun");
        self.on_path.remove(&pred_str);
    }

    /// The way from the predicate printed as `pred_str` round to it again, that predicate at both
    /// ends.
    fn way_round(&self, pred_str: &str) -> Vec<Predicate> {
        let start = self
            .path
            .iter()
            .position(|(_ancestor, ancestor_str)| ancestor_str == pred_str)
            .expect("the caller found the predicate among the deductions it is inside");
        let (repeated, _repeated_str) = &self.path[start];
        self.way_from(start, repeated)
    }

    /// The way from the predicate the deduction started at down to `pred`, `pred` last.
    fn way_down(&self, pred: &Predicate) -> Vec<Predicate> {
        self.way_from(0, pred)
    }

    /// The way from the predicate at `start` of the path down to `last`, `last` last.
    fn way_from(&self, start: usize, last: &Predicate) -> Vec<Predicate> {
        self.path[start..]
            .iter()
            .map(|(ancestor, _ancestor_str)| ancestor.clone())
            .chain([last.clone()])
            .collect()
    }
}

/// The steps of a search that does not end, as a report shows them: each step quoted, the way
/// shortened in the middle where it is long, and the step the search comes back to last.
///
/// # Arguments
/// * `way` — the steps as they are printed, the one the search fails on last.
///
/// # Examples
/// A way of three steps reads ``` `A` -> `B` -> `A` ```.
fn way_string(way: &[String]) -> String {
    /// How many steps to show. Enough for one turn of a search that asks about ever larger types to
    /// be visible, and short enough that the reader can read what is printed.
    const SHOWN_STEPS: usize = 3;

    // The last step is where the search shows what is wrong with it: the step it came back to, or
    // the one on a type too deep to be one the program wrote.
    let (last, rest) = way
        .split_last()
        .expect("a search carries the step it started from");
    let step_string = |step: &String| format!("`{}`", shorten_for_report(step.clone()));
    let mut steps = rest
        .iter()
        .take(SHOWN_STEPS)
        .map(step_string)
        .collect::<Vec<_>>();
    if rest.len() > SHOWN_STEPS {
        steps.push("...".to_string());
    }
    steps.push(step_string(last));
    steps.join(" -> ")
}

/// What the reduction of a type carries as it descends into the values that the equalities it
/// applies give.
///
/// An associated type is reduced by applying the equality whose left side it matches and reducing
/// the value that equality gives in its place. `on_path` holds the associated types whose reduction
/// has begun and has yet to end, so that a value naming one of them is seen for what it is.
#[derive(Default)]
struct TypeReduction {
    /// The associated types whose reduction the current one is inside, outermost first, each with
    /// the source of the equality applied to it, which is a step of the way round.
    path: Vec<(Arc<TypeNode>, Option<Span>)>,
    /// Each associated type on `path`. A type carries the hash it answers with, so asking whether
    /// one is here costs no walk of the type after the first.
    on_path: Set<Arc<TypeNode>>,
}

impl TypeReduction {
    /// Record that the reduction of the associated type `ty` has begun by applying the equality
    /// written at `src`.
    fn enter(&mut self, ty: &Arc<TypeNode>, src: &Option<Span>) {
        let newly_begun = self.on_path.insert(ty.clone());
        assert!(
            newly_begun,
            "the reduction of `{}` begins a second time while the first has yet to end",
            ty.to_string()
        );
        self.path.push((ty.clone(), src.clone()));
    }

    /// Record that the reduction entered last has ended.
    fn leave(&mut self) {
        let (ty, _src) = self
            .path
            .pop()
            .expect("a reduction ends only after it has begun");
        self.on_path.remove(&ty);
    }

    /// The report that the associated type `ty` is circular, drawn at every equality constraint on
    /// the way round.
    fn circular_error(&self, ty: &Arc<TypeNode>) -> Errors {
        let start = self
            .path
            .iter()
            .position(|(ancestor, _src)| ancestor == ty)
            .expect("the caller found the type among the reductions it is inside");
        let round = &self.path[start..];
        let way: Vec<String> = round
            .iter()
            .map(|(ancestor, _src)| ancestor.to_string())
            .chain([ty.to_string()])
            .collect();
        let srcs: Vec<&Option<Span>> = round.iter().map(|(_ancestor, src)| src).collect();
        let sentence = if way.len() <= 2 {
            format!(
                "The type `{}` is circular.",
                shorten_for_report(ty.to_string()),
            )
        } else {
            format!(
                "The type `{}` is circular: {}.",
                shorten_for_report(ty.to_string()),
                way_string(&way),
            )
        };
        Errors::from_msg_srcs(sentence, &srcs)
    }

    /// The report that the reduction reached a type deeper than the compiler reduces, drawn at the
    /// equality constraint that grew it.
    fn endless_error(&self, ty: &Arc<TypeNode>) -> Errors {
        // A reduction that has yet to apply an equality reached the bound on the type it was asked
        // about, which says nothing about any equality, so the report is drawn where that type is
        // written; one that has applied equalities reached it by applying them, and the last is the
        // one to draw the report at.
        let Some((_ancestor_str, src)) = self.path.last() else {
            return Errors::from_msg_srcs(
                format!(
                    "The type `{}` is nested too deep.",
                    shorten_for_report(ty.to_string()),
                ),
                &[ty.get_source()],
            );
        };
        Errors::from_msg_srcs(
            format!(
                "The type `{}` grew too large.",
                shorten_for_report(ty.to_string()),
            ),
            &[src],
        )
    }
}

/// A constraint that type checking required and could not settle.
///
/// A report names the constraint, and adds what the deduction of it did where the deduction rather
/// than the constraint is what fails.
#[derive(Clone)]
pub enum UnificationErr {
    /// No instance the program declares gives the predicate.
    Unsatisfiable(Predicate),
    /// Deducing the predicate comes back to the predicate itself, so nothing gives it. Carries the
    /// way from the predicate round to it, the predicate at both ends.
    Circular(Vec<Predicate>),
    /// Deducing the predicate asks for one on a deeper type, and that one for a deeper one again, so
    /// the deduction does not end. Carries the way down, deepest last.
    Endless(Vec<Predicate>),
    /// Two types that are required to be equal and that unification could not make equal.
    Disjoint(Arc<TypeNode>, Arc<TypeNode>),
}

impl UnificationErr {
    /// The constraint the report names, printed: the predicate that cannot be deduced, or an
    /// equation between the two types that cannot be made equal.
    pub fn to_constraint_string(&self) -> String {
        match self {
            UnificationErr::Unsatisfiable(p) => p.to_string(),
            UnificationErr::Circular(way) | UnificationErr::Endless(way) => {
                Self::reported_predicate(way).to_string()
            }
            UnificationErr::Disjoint(ty1, ty2) => {
                format!("{} = {}", ty1.to_string(), ty2.to_string())
            }
        }
    }

    /// `sentence`, which names the constraint, followed by what the deduction of that constraint
    /// did where the deduction rather than the constraint is what fails.
    pub fn message_with_note(&self, sentence: String) -> String {
        match self.note() {
            Some(note) => format!("{} {}", sentence, note),
            None => sentence,
        }
    }

    /// What a report adds after naming the constraint, where what fails is the deduction of the
    /// constraint rather than the constraint itself.
    fn note(&self) -> Option<String> {
        match self {
            UnificationErr::Unsatisfiable(_) | UnificationErr::Disjoint(_, _) => None,
            // A deduction that comes straight back to where it began is the whole story; a longer
            // one is told by the constraints it passes through.
            UnificationErr::Circular(way) if way.len() <= 2 => {
                Some("The inference is circular.".to_string())
            }
            UnificationErr::Circular(way) => Some(format!(
                "The inference is circular: {}.",
                way_string(&Self::printed_steps(way))
            )),
            // A deduction that has yet to take a step reached the bound on the constraint it was
            // asked for, which says nothing about any instance.
            UnificationErr::Endless(way) if way.len() <= 1 => {
                Some("The type it names is nested too deep.".to_string())
            }
            UnificationErr::Endless(way) => Some(format!(
                "The inference is too long: {}.",
                way_string(&Self::printed_steps(way))
            )),
        }
    }

    /// Appends to `buf` the type variables free in the types this error carries.
    pub fn free_vars_to_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        match self {
            UnificationErr::Unsatisfiable(p) => p.free_vars_to_vec(buf),
            UnificationErr::Circular(way) | UnificationErr::Endless(way) => {
                for pred in way {
                    pred.free_vars_to_vec(buf);
                }
            }
            UnificationErr::Disjoint(ty1, ty2) => {
                ty1.free_vars_to_vec(buf);
                ty2.free_vars_to_vec(buf);
            }
        }
    }

    /// The predicate a failed deduction is about: the one the report names.
    fn reported_predicate(way: &[Predicate]) -> &Predicate {
        way.first()
            .expect("a deduction carries the predicate it started from")
    }

    /// The steps of a deduction, each printed.
    fn printed_steps(way: &[Predicate]) -> Vec<String> {
        way.iter().map(|pred| pred.to_string()).collect()
    }
}

/// What a step of type checking fails with: a constraint it could not settle, or errors raised for
/// any other reason.
pub enum UnifOrOtherErr {
    /// A constraint the step required and could not settle.
    UnifErr(UnificationErr),
    /// Errors raised for any other reason, carried as they are.
    Others(Errors),
}

impl UnifOrOtherErr {
    /// Splits the two failures `res` can carry, so that the caller propagates the errors raised for
    /// other reasons with `?` and is left holding the constraint that could not be settled, which
    /// it can answer for itself.
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
    /// Carries errors raised for a reason other than an unsettled constraint, so that a step
    /// reporting them fits where a failure of type checking is expected.
    fn from(e: Errors) -> Self {
        UnifOrOtherErr::Others(e)
    }
}

impl From<UnificationErr> for UnifOrOtherErr {
    /// Carries a constraint that could not be settled, so that a step raising one fits where a
    /// failure of type checking is expected.
    fn from(e: UnificationErr) -> Self {
        UnifOrOtherErr::UnifErr(e)
    }
}

#[cfg(test)]
mod tests {
    use super::TypeCheckContext;
    use crate::ast::expr::{
        expr_abs, expr_app, expr_array_lit, expr_eval, expr_ffi_call, expr_if, expr_let,
        expr_make_struct, expr_match, expr_tyanno, expr_var, var_var, ExprNode,
    };
    use crate::ast::kind_scope::KindEnv;
    use crate::ast::name::FullName;
    use crate::ast::pattern::PatternNode;
    use crate::ast::program::TypeEnv;
    use crate::ast::traits::TraitEnv;
    use crate::ast::types::TyCon;
    use crate::elaboration::typecheckcache::MemoryCache;
    use crate::fixstd::builtin::make_bool_ty;
    use crate::misc::Map;
    use std::sync::Arc;

    /// Context over empty environments: the fallback typing reads only the
    /// fresh-type-variable counter, so no program state is needed.
    fn empty_context() -> TypeCheckContext {
        TypeCheckContext::new(
            TraitEnv::default(),
            TypeEnv::default(),
            KindEnv::default(),
            Map::default(),
            Arc::new(MemoryCache::new()),
            1,
            true,
        )
    }

    /// Unelaborated variable expression, as the parser would produce it.
    fn local_var(name: &str) -> Arc<ExprNode> {
        expr_var(FullName::local(name), None)
    }

    /// Unelaborated variable pattern.
    fn var_pat(name: &str) -> Arc<PatternNode> {
        PatternNode::make_var(var_var(FullName::local(name)), None)
    }

    /// The tolerant fallbacks substitute an unelaborated subtree for one whose
    /// elaboration failed, and every later walk assumes each expression node
    /// and pattern carries a type (`fix_types` aborts the process on one that
    /// does not — the language server dies with it). Verifies that
    /// `set_fallback_types` establishes that invariant over a tree containing
    /// every child-bearing `Expr` variant and every `Pattern` variant, and
    /// puts the given type at the root.
    #[test]
    fn test_set_fallback_types_types_every_node() {
        let mut tc = empty_context();

        let tycon = Arc::new(TyCon::new(FullName::from_strs(&["Main"], "S")));
        let struct_pat = PatternNode::make_struct(
            tycon.clone(),
            vec![
                ("a".to_string(), var_pat("a")),
                (
                    "b".to_string(),
                    PatternNode::make_union_with_span(
                        FullName::from_strs(&["Main"], "some"),
                        None,
                        var_pat("c"),
                    ),
                ),
            ],
        );

        let match_expr = expr_match(local_var("m"), vec![(struct_pat, local_var("arm"))], None);
        let if_expr = expr_if(local_var("cond"), local_var("t"), local_var("e"), None);
        let app_expr = expr_app(local_var("f"), vec![local_var("x"), local_var("y")], None);
        let lam_expr = expr_abs(vec![var_var(FullName::local("p"))], app_expr, None);
        let struct_expr = expr_make_struct(
            tycon.clone(),
            vec![
                ("a".to_string(), local_var("fa")),
                ("b".to_string(), if_expr),
            ],
        );
        let anno_expr = expr_tyanno(struct_expr, make_bool_ty(), None);
        let ffi_expr = expr_ffi_call(
            "c_fun".to_string(),
            tycon,
            vec![],
            false,
            vec![local_var("z")],
            false,
            None,
        );
        let array_expr = expr_array_lit(vec![anno_expr, match_expr, ffi_expr], None);
        let eval_expr = expr_eval(lam_expr, array_expr, None);
        let root = expr_let(var_pat("v"), local_var("bound"), eval_expr, None);

        let root_ty = make_bool_ty();
        let typed = tc.set_fallback_types(&root, root_ty.clone());

        assert!(
            Arc::ptr_eq(typed.type_.as_ref().unwrap(), &root_ty),
            "the root should carry the given type"
        );
        assert!(
            tc.check_all_typed(&typed).is_ok(),
            "every node and pattern of the substitute should carry a type"
        );
    }
}
