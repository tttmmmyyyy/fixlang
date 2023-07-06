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

// Information on trait.
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
                "Currently, exactly one condition is allowed as the assumption of trait definition.",
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
        for (_name, expr) in &mut self.methods {
            *expr = expr.resolve_namespace(ctx);
        }
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

#[derive(Clone, Serialize, Deserialize)]
pub struct PredicateInfo {
    pub source: Option<Span>,
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
            error_exit(&format!(
                "kind mismatch. Expect: {}, found: {}.",
                expected.to_string(),
                found.to_string()
            ))
        }
    }
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
}

impl TraitEnv {
    pub fn validate(&mut self, kind_map: &HashMap<TyCon, Rc<Kind>>) {
        // Validate trait instances.
        for (trait_id, insts) in &mut self.instances {
            for inst in insts.iter_mut() {
                *inst.trait_id_mut() = trait_id.clone();

                // Check instance is not head-normal-form.
                let implemented_ty = &inst.qual_pred.predicate.ty;
                if implemented_ty.is_hnf() {
                    error_exit_with_src(
                        &format!("Implementing trait for type `{}` is not allowed (by type inference algorithm used in Fix). The head (in this case, `{}`) of a type for which trait is implemented should be a type constructor and cannot be a type variable.", implemented_ty.to_string(), implemented_ty.get_head_string()),&implemented_ty.get_source()
                    );
                }

                // Check context is head-normal-form.
                // NOTE: we are currently require more string condition: `tv : SomeTrait`.
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

    // Add traits.
    pub fn add(&mut self, trait_infos: Vec<TraitInfo>, trait_impls: Vec<TraitInstance>) {
        for trait_info in trait_infos {
            self.add_trait(trait_info);
        }
        for trait_impl in trait_impls {
            self.add_instance(trait_impl);
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

    // Reduce predicate using trait instances.
    // Returns None when p cannot be reduced more.
    pub fn reduce_to_instance_contexts_one(
        &self,
        p: &Predicate,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Option<Vec<Predicate>> {
        let insntances = self.instances.get(&p.trait_id);
        if let Some(instances) = insntances {
            for inst in instances {
                match Substitution::matching(kind_map, &inst.qual_pred.predicate.ty, &p.ty) {
                    Some(s) => {
                        let ret = inst
                            .qual_pred
                            .context
                            .iter()
                            .map(|c| {
                                let mut c = c.clone();
                                s.substitute_predicate(&mut c);
                                c
                            })
                            .collect();
                        return Some(ret);
                    }
                    None => {}
                }
            }
        }
        return None;
    }

    // Entailment.
    pub fn entail(
        &self,
        ps: &Vec<Predicate>,
        p: &Predicate,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> bool {
        // If p in contained in ps, then ok.
        for q in ps {
            if q.trait_id == p.trait_id {
                if Substitution::matching(kind_map, &q.ty, &p.ty).is_some() {
                    return true;
                }
            }
        }
        // Try reducing by instances.
        match self.reduce_to_instance_contexts_one(p, kind_map) {
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
    pub fn reduce_to_hnf(
        &self,
        p: &Predicate,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Result<Vec<Predicate>, Predicate> {
        if p.ty.is_hnf() {
            return Ok(vec![p.clone()]);
        }
        match self.reduce_to_instance_contexts_one(p, kind_map) {
            Some(ps) => self.reduce_to_hnfs(&ps, kind_map),
            None => Err(p.clone()),
        }
    }

    // Reduce predicates to head normal form.
    // Returns Err(p) if reduction failed due to predicate p.
    pub fn reduce_to_hnfs(
        &self,
        ps: &Vec<Predicate>,
        kind_map: &HashMap<TyCon, Rc<Kind>>,
    ) -> Result<Vec<Predicate>, Predicate> {
        let mut ret: Vec<Predicate> = Default::default();
        for p in ps {
            ret.append(&mut self.reduce_to_hnf(p, kind_map)?)
        }
        Ok(ret)
    }

    // Simplify a set of predicates by entail.
    pub fn simplify_predicates(
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
        // TODO: Improve performance. See scEntail in Typing Haskell in Haskell.
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
        let ret = self.reduce_to_hnfs(ps, kind_map)?;
        let ret = self.simplify_predicates(&ret, kind_map);
        // Every predicate has to be hnf.
        assert!(ret.iter().all(|p| p.ty.is_hnf()));
        Ok(ret)
    }

    // Set each TraitInfo's kind.
    pub fn set_kinds(&mut self) {
        for (_id, ti) in &mut self.traits {
            ti.set_trait_kind();
        }
    }

    pub fn trait_kind_map(&self) -> HashMap<TraitId, Rc<Kind>> {
        let mut res: HashMap<TraitId, Rc<Kind>> = HashMap::default();
        for (id, ti) in &self.traits {
            res.insert(id.clone(), ti.type_var.kind.clone());
        }
        res
    }

    pub fn import(&mut self, other: TraitEnv) {
        for (_, ti) in other.traits {
            self.add_trait(ti);
        }
        for (_, insts) in other.instances {
            for inst in insts {
                self.add_instance(inst)
            }
        }
    }
}
