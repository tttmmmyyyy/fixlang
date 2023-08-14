use serde::{Deserialize, Serialize};

use super::*;

// Identifier to spacify trait.
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TraitId {
    pub name: FullName,
}

impl TraitId {
    pub fn from_fullname(name: FullName) -> TraitId {
        TraitId { name }
    }

    pub fn to_string(&self) -> String {
        self.namespaced_name().to_string()
    }

    pub fn namespaced_name(&self) -> FullName {
        self.name.clone()
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), String> {
        self.name = ctx.resolve(&self.name, NameResolutionType::Trait)?;
        Ok(())
    }
}

// Traits.
#[derive(Clone)]
pub struct TraitInfo {
    // Identifier of this trait (i.e. the name).
    pub id: TraitId,
    // Type variable used in trait definition.
    pub type_var: Rc<TyVar>,
    // Methods of this trait.
    // Here, for example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String",
    // and not "a -> String for a : Show".
    pub methods: HashMap<Name, QualType>,
    // Predicates at the trait declaration, e.g., "f: *->*" in "trait [f:*->*] f: Functor {}".
    pub kind_predicates: Vec<KindPredicate>,
    // Source location of trait definition.
    pub source: Option<Span>,
}

impl TraitInfo {
    // Resolve namespace.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for (_name, qt) in &mut self.methods {
            qt.resolve_namespace(ctx);
        }
    }

    // Resolve type aliases
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for (_name, qt) in &mut self.methods {
            qt.resolve_type_aliases(type_env);
        }
    }

    // Get type-scheme of a method.
    // Here, for example, in case "trait a: Show { show: a -> String }",
    // this function returns "[a: Show] a -> String" as type of "show" method.
    pub fn method_scheme(&self, name: &Name) -> Rc<Scheme> {
        let mut ty = self.methods.get(name).unwrap().clone();
        let vars = ty.free_vars();
        let mut preds = vec![Predicate::make(
            self.id.clone(),
            type_var_from_tyvar(self.type_var.clone()),
        )];
        preds.append(&mut ty.preds);
        Scheme::generalize(vars, preds, ty.ty)
    }

    // Get the type of a method.
    // Here, for example, in case "trait a: Show { show: a -> String }",
    // this function returns "a -> String" as type of "show" method.
    pub fn method_ty(&self, name: &Name) -> QualType {
        self.methods.get(name).unwrap().clone()
    }

    // Validate kind_predicates and set it to self.type_var.
    pub fn set_trait_kind(&mut self) {
        if self.kind_predicates.len() >= 2 {
            let span = Span::unite_opt(
                &self.kind_predicates[0].source,
                &self.kind_predicates[1].source,
            );
            error_exit_with_src(
                "You can specify at most one condition of the form `{type-variable} : {kind}` as the assumption of trait definition.",
                &span,
            );
        }
        if self.kind_predicates.len() > 0 {
            if self.kind_predicates[0].name != self.type_var.name {
                error_exit_with_src(
                    &format!(
                        "The type variable used in the assumption of trait `{}` has to be `{}`.",
                        self.id.to_string(),
                        self.type_var.name,
                    ),
                    &self.kind_predicates[0].source,
                );
            }
            self.type_var = self.type_var.set_kind(self.kind_predicates[0].kind.clone());
        }
    }
}

// Trait instance.
#[derive(Clone)]
pub struct TraitInstance {
    // Statement such as "(a, b): Show for a: Show, b: Show".
    pub qual_pred: QualPredicate,
    // Method implementation.
    pub methods: HashMap<Name, Rc<ExprNode>>,
    // Module where this instance is defined.
    pub define_module: Name,
    // Source location where this module is defined.
    pub source: Option<Span>,
}

impl TraitInstance {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        self.qual_pred.resolve_namespace(ctx);

        // This function is called only by resolve_namespace_in_declaration, so we don't need to see into expression.

        // for (_name, expr) in &mut self.methods {
        //     *expr = expr.resolve_namespace(ctx);
        // }
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        self.qual_pred.resolve_type_aliases(type_env);
    }

    // Get trait id.
    fn trait_id(&self) -> TraitId {
        self.qual_pred.predicate.trait_id.clone()
    }

    // Get mutable trait id.
    fn trait_id_mut(&mut self) -> &mut TraitId {
        &mut self.qual_pred.predicate.trait_id
    }

    // Get type-scheme of a method implementation.
    // Here, for example, in case "impl [a: Show, b: Show] (a, b): Show",
    // this function returns "[a: Show, b: Show] (a, b) -> String" as the type of "show".
    pub fn method_scheme(&self, method_name: &Name, trait_info: &TraitInfo) -> Rc<Scheme> {
        let trait_tyvar = &trait_info.type_var.name;
        let impl_type = self.qual_pred.predicate.ty.clone();
        let s = Substitution::single(&trait_tyvar, impl_type);
        let mut method_qualty = trait_info.method_ty(method_name);
        s.substitute_qualtype(&mut method_qualty);

        let ty = method_qualty.ty.clone();
        let vars = ty.free_vars();
        let mut preds = self.qual_pred.context.clone();
        preds.append(&mut method_qualty.preds);
        Scheme::generalize(vars, preds, ty)
    }

    // Get expression that implements a method.
    pub fn method_expr(&self, name: &Name) -> Rc<ExprNode> {
        self.methods.get(name).unwrap().clone()
    }
}

// Trait Aliases
#[derive(Clone)]
pub struct TraitAlias {
    // Identifier of this trait (i.e., the name).
    pub id: TraitId,
    // Aliased traits.
    pub value: Vec<TraitId>,
    // Source location of alias definition.
    pub source: Option<Span>,
    // Kind of this trait alias.
    pub kind: Rc<Kind>,
}

impl TraitAlias {
    // Resolve namespace of trait names in value.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for trait_id in &mut self.value {
            let res = trait_id.resolve_namespace(ctx);
            if res.is_err() {
                error_exit_with_src(&res.unwrap_err(), &self.source);
            }
        }
    }
}

// Qualified predicate. Statement such as "Array a : Eq for a : Eq".
#[derive(Clone)]
pub struct QualPredicate {
    pub context: Vec<Predicate>,
    pub kind_preds: Vec<KindPredicate>,
    pub predicate: Predicate,
}

impl QualPredicate {
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for p in &mut self.context {
            p.resolve_namespace(ctx);
        }
        self.predicate.resolve_namespace(ctx);
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for p in &mut self.context {
            p.resolve_type_aliases(type_env);
        }
        self.predicate.resolve_type_aliases(type_env);
    }

    pub fn extend_kind_scope(
        scope: &mut HashMap<Name, Rc<Kind>>,
        preds: &Vec<Predicate>,
        kind_preds: &Vec<KindPredicate>,
        trait_kind_map: &HashMap<TraitId, Rc<Kind>>,
    ) -> Result<(), String> {
        fn insert(
            scope: &mut HashMap<Name, Rc<Kind>>,
            tyvar: String,
            kind: Rc<Kind>,
        ) -> Result<(), String> {
            if scope.contains_key(&tyvar) {
                if scope[&tyvar] != kind {
                    return Err("Kind mismatch on type variable.".to_string());
                }
            } else {
                scope.insert(tyvar, kind);
            }
            Ok(())
        }

        for p in preds {
            let tyvar = match &p.ty.ty {
                Type::TyVar(tv) => tv.name.clone(),
                _ => {
                    panic!("Currently, trait bound has to be of the form `tv : SomeTrait` for a type variable `tv`.")
                }
            };
            let trait_id = &p.trait_id;
            if !trait_kind_map.contains_key(trait_id) {
                panic!("Unknown trait: {}", trait_id.to_string());
            }
            let kind = trait_kind_map[trait_id].clone();
            insert(scope, tyvar, kind)?;
        }
        for kp in kind_preds {
            let tyvar = kp.name.clone();
            let kind = kp.kind.clone();
            insert(scope, tyvar, kind)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct QualType {
    pub preds: Vec<Predicate>,
    pub kind_preds: Vec<KindPredicate>,
    pub ty: Rc<TypeNode>,
}

impl QualType {
    // Resolve namespace.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        for pred in &mut self.preds {
            pred.resolve_namespace(ctx);
        }
        self.ty = self.ty.resolve_namespace(ctx);
    }

    // Resolve type aliases
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        for pred in &mut self.preds {
            pred.resolve_type_aliases(type_env);
        }
        self.ty = self.ty.resolve_aliases(type_env);
    }

    // Calculate free type variables.
    pub fn free_vars(&self) -> HashMap<Name, Rc<Kind>> {
        self.ty.free_vars()
    }
}

// Statement such as "String: Show" or "a: Eq".
#[derive(Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub trait_id: TraitId,
    pub ty: Rc<TypeNode>,
    pub info: PredicateInfo,
}

impl Predicate {
    pub fn set_source(&mut self, source: Span) {
        self.info.source = Some(source);
    }

    pub fn make(trait_id: TraitId, ty: Rc<TypeNode>) -> Self {
        Predicate {
            trait_id,
            ty,
            info: PredicateInfo { source: None },
        }
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) {
        let resolve_result = self.trait_id.resolve_namespace(ctx);
        if resolve_result.is_err() {
            error_exit_with_src(&resolve_result.unwrap_err(), &self.info.source)
        }
        self.ty = self.ty.resolve_namespace(ctx);
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        self.ty = self.ty.resolve_aliases(type_env);
    }

    pub fn to_string_normalize(&self) -> String {
        format!(
            "{} : {}",
            self.ty.to_string_normalize(),
            self.trait_id.to_string()
        )
    }

    pub fn set_kinds(&mut self, scope: &HashMap<Name, Rc<Kind>>) {
        self.ty = self.ty.set_kinds(scope);
    }

    pub fn check_kinds(
        &self,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
        trait_kind_map: &HashMap<TraitId, Rc<Kind>>,
    ) {
        let expected = &trait_kind_map[&self.trait_id];
        let found = self.ty.kind(kind_map);
        if *expected != found {
            error_exit_with_src(
                &format!(
                    "Kind mismatch in `{}`. Expect: {}, found: {}.",
                    self.to_string_normalize(),
                    expected.to_string(),
                    found.to_string()
                ),
                &self.info.source,
            )
        }
    }

    // If the trait used in this predicate is a trait alias, resolve it to a set of predicates that are not using trait aliases.
    fn resolve_trait_aliases(&self, trait_env: &TraitEnv) -> Vec<Predicate> {
        if !trait_env.is_alias(&self.trait_id) {
            return vec![self.clone()];
        }
        let trait_ids = trait_env.resolve_aliases(&self.trait_id);
        let mut res = vec![];
        for trait_id in trait_ids {
            let mut p = self.clone();
            p.trait_id = trait_id;
            res.push(p);
        }
        res
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PredicateInfo {
    pub source: Option<Span>,
}

// Statement such as "f: * -> *".
#[derive(Clone)]
pub struct KindPredicate {
    pub name: Name,
    pub kind: Rc<Kind>,
    pub source: Option<Span>,
}

// Trait environments.
#[derive(Clone, Default)]
pub struct TraitEnv {
    pub traits: HashMap<TraitId, TraitInfo>,
    pub instances: HashMap<TraitId, Vec<TraitInstance>>,
    pub aliases: HashMap<TraitId, TraitAlias>,
}

impl TraitEnv {
    // Get of list of trait names including aliases.
    pub fn trait_names(&self) -> HashSet<FullName> {
        let mut res: HashSet<FullName> = Default::default();
        for (k, _v) in &self.traits {
            res.insert(k.name.clone());
        }
        for (k, _v) in &self.aliases {
            res.insert(k.name.clone());
        }
        res
    }

    pub fn validate(&mut self, kind_map: &HashMap<TyCon, Rc<Kind>>) {
        // Check name confliction of traits and aliases.
        fn create_aconflicting_error(env: &TraitEnv, trait_id: &TraitId) {
            let this_src = &env.traits.get(trait_id).unwrap().source;
            let other_src = &env.aliases.get(trait_id).unwrap().source;
            error_exit_with_srcs(
                &format!("Duplicate definition for `{}`", trait_id.to_string()),
                &[&this_src, &other_src],
            );
        }
        for (trait_id, _) in &self.traits {
            if self.aliases.contains_key(trait_id) {
                create_aconflicting_error(self, trait_id);
            }
        }
        for (trait_id, _) in &self.aliases {
            if self.traits.contains_key(trait_id) {
                create_aconflicting_error(self, trait_id);
            }
        }

        // Check that values of trait aliases are defined.
        for (_, ta) in &self.aliases {
            for v in &ta.value {
                if !self.traits.contains_key(v) && !self.aliases.contains_key(v) {
                    error_exit_with_src(&format!("Unknown trait `{}`.", v.to_string()), &ta.source);
                }
            }
        }
        // Circular aliasing will be detected in `TraitEnv::resolve_aliases`.

        let aliases: HashSet<_> = self.aliases.keys().collect();
        // Validate trait instances.
        for (trait_id, insts) in &mut self.instances {
            for inst in insts.iter_mut() {
                // check implementation is given for trait, not for trait alias.
                if aliases.contains(trait_id) {
                    error_exit_with_src(
                        "You cannot implement a trait alias directly. Implement each aliased trait instead.",
                        &inst.qual_pred.predicate.info.source,
                    )
                }

                *inst.trait_id_mut() = trait_id.clone();

                // Check instance is not head-normal-form.
                let implemented_ty = &inst.qual_pred.predicate.ty;
                if implemented_ty.is_hnf() {
                    error_exit_with_src(
                        &format!("Implementing trait for type `{}` is not allowed (by type inference algorithm used in Fix). The head (in this case, `{}`) of a type for which trait is implemented should be a type constructor and cannot be a type variable.", implemented_ty.to_string(), implemented_ty.get_head_string()),&implemented_ty.get_source()
                    );
                }

                // Check context is head-normal-form.
                // NOTE: we are currently require more strong condition: `tv : SomeTrait`.
                // This is because we don't have "kind inference", so predicate of form `m SomeType` cannot be handled.
                for ctx in &inst.qual_pred.context {
                    match ctx.ty.ty {
                        Type::TyVar(_) => {}
                        _ => {
                            error_exit_with_src(&format!("Invalid trait bound `{}`. In current Fix, trait bound has to be of the form `tv : SomeTrait` for a type variable `tv`.", ctx.to_string_normalize()), &ctx.info.source);
                        }
                    }
                }

                // Check whether all trait methods are implemented.
                let trait_methods = &self.traits[trait_id]
                    .methods
                    .iter()
                    .map(|s| s.0)
                    .collect::<HashSet<_>>();
                let impl_methods = inst.methods.iter().map(|s| s.0).collect::<HashSet<_>>();
                for trait_method in trait_methods {
                    if !impl_methods.contains(trait_method) {
                        let pred = inst.qual_pred.predicate.to_string_normalize();
                        error_exit_with_src(
                            &format!(
                                "Lacking implementation of method `{}` for `{}`",
                                trait_method, pred,
                            ),
                            &inst.source,
                        )
                    }
                }
                for impl_method in impl_methods {
                    if !trait_methods.contains(impl_method) {
                        error_exit_with_src(
                            &format!(
                                "`{}` is not a method of trait `{}`.",
                                impl_method,
                                trait_id.to_string(),
                            ),
                            &inst
                                .methods
                                .get(impl_method)
                                .unwrap()
                                .source
                                .as_ref()
                                .map(|s| s.to_single_character()),
                        )
                    }
                }
            }

            // Check overlapping instance.
            for i in 0..insts.len() {
                for j in (i + 1)..insts.len() {
                    let inst_i = &insts[i];
                    let inst_j = &insts[j];
                    if Substitution::unify(
                        kind_map,
                        &inst_i.qual_pred.predicate.ty,
                        &inst_j.qual_pred.predicate.ty,
                    )
                    .is_some()
                    {
                        error_exit_with_srcs(
                            &format!(
                                "Two trait implementations for `{}` are overlapping.",
                                trait_id.to_string()
                            ),
                            &[
                                &inst_i.source.as_ref().map(|s| s.to_single_character()),
                                &inst_j.source.as_ref().map(|s| s.to_single_character()),
                            ],
                        );
                    }
                }
            }
        }
    }

    pub fn resolve_namespace(
        &mut self,
        ctx: &mut NameResolutionContext,
        imported_modules: &HashMap<Name, HashSet<Name>>,
    ) {
        // Resolve names in trait aliases.
        for (trait_id, alias_info) in &mut self.aliases {
            ctx.imported_modules = imported_modules[&trait_id.name.module()].clone();
            alias_info.resolve_namespace(ctx);
        }

        // Resolve names in trait definitions.
        for (trait_id, trait_info) in &mut self.traits {
            ctx.imported_modules = imported_modules[&trait_id.name.module()].clone();
            // Keys in self.traits should already be resolved.
            assert!(
                trait_id.name
                    == ctx
                        .resolve(&trait_id.name, NameResolutionType::Trait)
                        .unwrap()
            );
            trait_info.resolve_namespace(ctx);
        }

        // Resolve names in trait implementations.
        let insntaces = std::mem::replace(&mut self.instances, Default::default());
        let mut instances_resolved: HashMap<TraitId, Vec<TraitInstance>> = Default::default();
        for (trait_id, insts) in insntaces {
            for mut inst in insts {
                // Set up NameResolutionContext.
                ctx.imported_modules = imported_modules[&inst.define_module].clone();

                // Resolve trait_id's namespace.
                let mut trait_id = trait_id.clone();
                let resolve_result = trait_id.resolve_namespace(ctx);
                if resolve_result.is_err() {
                    let src = inst.qual_pred.predicate.info.source.clone();
                    error_exit_with_src(&resolve_result.unwrap_err(), &src)
                }

                // Resolve names in TrantInstance.
                inst.resolve_namespace(ctx);

                // Insert to instances_resolved
                if !instances_resolved.contains_key(&trait_id) {
                    instances_resolved.insert(trait_id.clone(), vec![]);
                }
                instances_resolved.get_mut(&trait_id).unwrap().push(inst);
            }
        }
        self.instances = instances_resolved;
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) {
        // Resolve aliases in trait definitions.
        for (_, trait_info) in &mut self.traits {
            trait_info.resolve_type_aliases(type_env);
        }

        // Resolve aliases in trait implementations.
        let insntaces = std::mem::replace(&mut self.instances, Default::default());
        let mut instances_resolved: HashMap<TraitId, Vec<TraitInstance>> = Default::default();
        for (trait_id, insts) in insntaces {
            for mut inst in insts {
                // Resolve names in TrantInstance.
                inst.resolve_type_aliases(type_env);

                // Insert to instances_resolved
                if !instances_resolved.contains_key(&trait_id) {
                    instances_resolved.insert(trait_id.clone(), vec![]);
                }
                instances_resolved.get_mut(&trait_id).unwrap().push(inst);
            }
        }
        self.instances = instances_resolved;
    }

    // Add traits.
    pub fn add(
        &mut self,
        trait_infos: Vec<TraitInfo>,
        trait_impls: Vec<TraitInstance>,
        trait_aliases: Vec<TraitAlias>,
    ) {
        for trait_info in trait_infos {
            self.add_trait(trait_info);
        }
        for trait_impl in trait_impls {
            self.add_instance(trait_impl);
        }
        for trait_alias in trait_aliases {
            self.add_alias(trait_alias);
        }
    }

    // Add a trait to TraitEnv.
    pub fn add_trait(&mut self, info: TraitInfo) {
        // Check duplicate definition.
        if self.traits.contains_key(&info.id) {
            let info1 = self.traits.get(&info.id).unwrap();
            error_exit_with_srcs(
                &format!("Duplicate definition for trait {}.", info.id.to_string()),
                &[&info1.source, &info.source],
            );
        }
        self.traits.insert(info.id.clone(), info);
    }

    // Add an instance.
    pub fn add_instance(&mut self, inst: TraitInstance) {
        let trait_id = inst.trait_id();
        if !self.instances.contains_key(&trait_id) {
            self.instances.insert(trait_id.clone(), vec![]);
        }
        self.instances.get_mut(&trait_id).unwrap().push(inst);
    }

    // Add an trait alias.
    fn add_alias(&mut self, alias: TraitAlias) {
        // Check duplicate definition.
        if self.aliases.contains_key(&alias.id) {
            let alias1 = self.aliases.get(&alias.id).unwrap();
            error_exit_with_srcs(
                &format!("Duplicate definition for trait {}.", alias.id.to_string()),
                &[&alias1.source, &alias.source],
            );
        }
        self.aliases.insert(alias.id.clone(), alias);
    }

    // Reduce a predicate p to a context of trait instance.
    // For example, reduce `Array a : Eq` to `a : Eq` using instance `impl [a : Eq] Array a : Eq`.
    // Returns None when p cannot be reduced more.
    fn reduce_to_context_of_instance(
        &self,
        p: &Predicate,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Option<Vec<Predicate>> {
        let insntances = self.instances.get(&p.trait_id);
        if let Some(instances) = insntances {
            for inst in instances {
                match Substitution::matching(kind_map, &inst.qual_pred.predicate.ty, &p.ty) {
                    Some(s) => {
                        let ps = inst.qual_pred.context.iter().map(|c| {
                            let mut c = c.clone();
                            s.substitute_predicate(&mut c);
                            c
                        });
                        let mut ret = vec![];
                        for p in ps {
                            ret.append(&mut p.resolve_trait_aliases(self));
                        }
                        return Some(ret);
                    }
                    None => {}
                }
            }
        }
        return None;
    }

    // Judge whether a predicate p is entailed by a set of predicates ps.
    pub fn entail(
        &self,
        ps: &Vec<Predicate>,
        p: &Predicate,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> bool {
        // Resolve trait aliases in ps.
        let mut resolved_ps = vec![];
        for p in ps {
            resolved_ps.append(&mut p.resolve_trait_aliases(self));
        }
        let ps = resolved_ps;

        // Resolve trait aliases in p.
        p.resolve_trait_aliases(self)
            .iter()
            .all(|p| self.entail_inner(&ps, p, kind_map))
    }

    // Judge whether a predicate p is entailed by a set of predicates ps.
    // p and ps cannot contain trait aliases.
    fn entail_inner(
        &self,
        ps: &Vec<Predicate>,
        p: &Predicate,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> bool {
        // If p is a special case of a predicate in ps, then ok.
        for q in ps {
            if q.trait_id == p.trait_id {
                if Substitution::matching(kind_map, &q.ty, &p.ty).is_some() {
                    return true;
                }
            }
        }
        // Try reducing p by instances.
        match self.reduce_to_context_of_instance(p, kind_map) {
            Some(ctxs) => {
                let mut all_ok = true;
                for ctx in ctxs {
                    if !self.entail(ps, &ctx, kind_map) {
                        all_ok = false;
                        break;
                    }
                }
                all_ok
            }
            None => false,
        }
    }

    // Reduce a predicate to head normal form.
    // Returns Err(p) if reduction failed due to predicate p.
    fn reduce_to_hnfs(
        &self,
        p: &Predicate,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Result<Vec<Predicate>, Predicate> {
        if p.ty.is_hnf() {
            return Ok(vec![p.clone()]);
        }
        match self.reduce_to_context_of_instance(p, kind_map) {
            Some(ps) => self.reduce_to_hnfs_many(&ps, kind_map),
            None => Err(p.clone()),
        }
    }

    // Reduce predicates to head normal form.
    // Returns Err(p) if reduction failed due to predicate p.
    fn reduce_to_hnfs_many(
        &self,
        ps: &Vec<Predicate>,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Result<Vec<Predicate>, Predicate> {
        let mut ret: Vec<Predicate> = Default::default();
        for p in ps {
            ret.append(&mut self.reduce_to_hnfs(p, kind_map)?)
        }
        Ok(ret)
    }

    // Simplify a set of predicates by entail.
    fn reduce_predicates_by_entail(
        &self,
        ps: &Vec<Predicate>,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Vec<Predicate> {
        let mut ps = ps.clone();
        let mut i = 0 as usize;
        while i < ps.len() {
            let qs: Vec<Predicate> = ps
                .iter()
                .enumerate()
                .filter_map(|(j, p)| if i == j { None } else { Some(p.clone()) })
                .collect();
            if self.entail(&qs, &ps[i], kind_map) {
                ps.remove(i);
            } else {
                i += 1;
            }
        }
        ps
        // TODO: Improve performance. See scEntail in "Typing Haskell in Haskell".
    }

    // Context reduction.
    // Returns qs when satisfaction of ps are reduced to qs.
    // In particular, returns empty when ps are satisfied.
    // Returns Err(p) if reduction failed due to predicate p.
    pub fn reduce(
        &self,
        ps: &Vec<Predicate>,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Result<Vec<Predicate>, Predicate> {
        // Resolve trait aliases in ps.
        let mut resolved_ps = vec![];
        for p in ps {
            resolved_ps.append(&mut p.resolve_trait_aliases(self));
        }
        let ps = resolved_ps;

        let ret = self.reduce_to_hnfs_many(&ps, kind_map)?;
        let ret = self.reduce_predicates_by_entail(&ret, kind_map);
        // Every predicate has to be hnf.
        assert!(ret.iter().all(|p| p.ty.is_hnf()));
        Ok(ret)
    }

    // Resolve trait aliases.
    fn resolve_aliases(&self, trait_id: &TraitId) -> Vec<TraitId> {
        fn resolve_aliases_inner(
            env: &TraitEnv,
            trait_id: &TraitId,
            res: &mut Vec<TraitId>,
            visited: &mut HashSet<TraitId>,
        ) {
            if visited.contains(trait_id) {
                error_exit(&format!(
                    "Circular aliasing detected in trait alias `{}`.",
                    trait_id.to_string()
                ));
            }
            visited.insert(trait_id.clone());
            if env.traits.contains_key(trait_id) {
                res.push(trait_id.clone());
                return;
            }
            for v in &env.aliases.get(trait_id).unwrap().value {
                resolve_aliases_inner(env, v, res, visited);
            }
        }

        let mut res = vec![];
        let mut visited = HashSet::new();
        resolve_aliases_inner(self, trait_id, &mut res, &mut visited);
        res
    }

    // Check if a trait name is an alias.
    pub fn is_alias(&self, trait_id: &TraitId) -> bool {
        self.aliases.contains_key(trait_id)
    }

    // Set kinds in TraitInfo and TraitAlias.
    pub fn set_kinds(&mut self) {
        for (_id, ti) in &mut self.traits {
            ti.set_trait_kind();
        }
        let mut resolved_aliases: HashMap<TraitId, Vec<TraitId>> = HashMap::new();
        for (id, _) in &self.aliases {
            resolved_aliases.insert(id.clone(), self.resolve_aliases(id));
        }
        for (id, ta) in &mut self.aliases {
            let mut kinds = resolved_aliases
                .get(id)
                .unwrap()
                .iter()
                .map(|id| self.traits.get(id).unwrap().type_var.kind.clone());
            let kind = kinds.next().unwrap();
            for k in kinds {
                if k != kind {
                    error_exit_with_src(
                        &format!(
                            "Kind mismatch in the definition of trait alias `{}`",
                            id.to_string()
                        ),
                        &ta.source,
                    )
                }
            }
            ta.kind = kind;
        }
    }

    pub fn trait_kind_map(&self) -> HashMap<TraitId, Rc<Kind>> {
        let mut res: HashMap<TraitId, Rc<Kind>> = HashMap::default();
        for (id, ti) in &self.traits {
            res.insert(id.clone(), ti.type_var.kind.clone());
        }
        for (id, ta) in &self.aliases {
            res.insert(id.clone(), ta.kind.clone());
        }
        res
    }

    pub fn import(&mut self, other: TraitEnv) {
        for (_, ti) in other.traits {
            self.add_trait(ti);
        }
        for (_, insts) in other.instances {
            for inst in insts {
                self.add_instance(inst);
            }
        }
        for (_, alias) in other.aliases {
            self.add_alias(alias);
        }
    }
}
