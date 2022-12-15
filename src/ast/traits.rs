use super::*;

// Identifier to spacify trait.
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TraitId {
    pub name: NameSpacedName,
}

impl TraitId {
    pub fn new(ns: &[&str], name: &Name) -> TraitId {
        TraitId {
            name: NameSpacedName::from_strs(ns, &name),
        }
    }

    pub fn new_by_name(name: &Name) -> TraitId {
        TraitId {
            name: NameSpacedName::from_strs(&[], name),
        }
    }

    pub fn to_string(&self) -> String {
        self.namespaced_name().to_string()
    }

    pub fn namespaced_name(&self) -> NameSpacedName {
        self.name.clone()
    }

    pub fn set_namespace_of_traits(&mut self, trait_env: &TraitEnv, module_name: &Name) {
        self.name = trait_env.infer_namespace(&self.name, module_name);
    }
}

// Information on trait.
#[derive(Clone)]
pub struct TraitInfo {
    // Identifier of this trait (i.e. the name).
    pub id: TraitId,
    // Type variable used in trait definition.
    pub type_var: Arc<TyVar>,
    // Methods of this trait.
    // Here, for example, in case "trait a: Show { show: a -> String }",
    // the type of method "show" is "a -> String",
    // and not "a -> String for a : Show".
    pub methods: HashMap<Name, QualType>,
    pub instances: Vec<TraitInstance>,
    // Predicates at the trait declaration, e.g., "f: *->*" in "trait [f:*->*] f: Functor {}".
    pub kind_predicates: Vec<KindPredicate>,
}

impl TraitInfo {
    // Get type-scheme of a method.
    // Here, for example, in case "trait a: Show { show: a -> String }",
    // this function returns "a -> String for a: Show" as type of "show" method.
    pub fn method_scheme(&self, name: &Name) -> Arc<Scheme> {
        let mut ty = self.methods.get(name).unwrap().clone();
        let vars = ty.free_vars();
        if !vars.contains_key(&self.type_var.name) {
            error_exit("type of trait method must contain bounded type.");
            // TODO: check this in more early stage.
        }
        let mut preds = vec![Predicate {
            trait_id: self.id.clone(),
            ty: type_var_from_tyvar(self.type_var.clone()),
        }];
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
            error_exit("in trait declaration, only one constraint (specification of kind of the type variable trait is implemented to) is allowed.");
        }
        if self.kind_predicates.len() > 0 {
            if self.kind_predicates[0].name != self.type_var.name {
                error_exit(&format!(
                    "the type variable {} is not used in trait declaration.",
                    self.kind_predicates[0].name
                ));
            }
            self.type_var = self.type_var.set_kind(self.kind_predicates[0].kind.clone());
        }
    }

    fn set_kinds_to_methods(&mut self, trait_kind_map: &HashMap<TraitId, Arc<Kind>>) {
        let mut scope = KindPredicate::vec_to_map(self.kind_predicates.clone());
        for (_, method_ty) in &mut self.methods {
            method_ty.set_kinds(&mut scope, trait_kind_map)
        }
    }
}

// Trait instance.
#[derive(Clone)]
pub struct TraitInstance {
    // Statement such as "(a, b): Show for a: Show, b: Show".
    pub qual_pred: QualPredicate,
    // Method implementation.
    pub methods: HashMap<Name, Arc<ExprNode>>,
}

impl TraitInstance {
    // Export methods with information to register them as global symbols.
    // pub fn methods_as_global(&self) -> Vec<(NameSpacedName, Arc<ExprNode>, Arc<Scheme>)> {
    //     self.methods
    //         .iter()
    //         .map(|(name, expr, ty)| {
    //             let name = format!("{}%{}", name, self.id);
    //             let nsn = NameSpacedName::from_strs(&[&self.trait_id().name], &name);
    //             let scm =
    //                 Scheme::generalize(ty.free_vars(), self.qual_pred.context.clone(), ty.clone());
    //             (nsn, expr.clone(), scm)
    //         })
    //         .collect()
    // }

    // Get trait id.
    fn trait_id(&self) -> TraitId {
        self.qual_pred.predicate.trait_id.clone()
    }

    // Get type-scheme of a method implementation.
    // Here, for example, in case "impl (a, b): Show for a: Show, b: Show",
    // this function returns "a -> String for a: Show, b: Show" as the type of "show".
    // Give type of this method, e.g., "a -> String".
    pub fn method_scheme(&self, method_name: &Name, trait_info: &TraitInfo) -> Arc<Scheme> {
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
    pub fn method_expr(&self, name: &Name) -> Arc<ExprNode> {
        self.methods.get(name).unwrap().clone()
    }
}

impl TraitInfo {
    // Add a instance to a trait.
    pub fn add_instance(&mut self, inst: TraitInstance, type_env: &TypeEnv) {
        // Check trait id.
        assert!(self.id == inst.qual_pred.predicate.trait_id);

        // Check overlapping instance.
        for i in &self.instances {
            if Substitution::unify(
                type_env,
                &inst.qual_pred.predicate.ty,
                &i.qual_pred.predicate.ty,
            )
            .is_some()
            {
                error_exit("overlapping instance.");
            }
        }

        // TODO: check validity of method implementation, etc.

        self.instances.push(inst)
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
    pub fn extend_kind_scope(
        scope: &mut HashMap<Name, Arc<Kind>>,
        preds: &Vec<Predicate>,
        kind_preds: &Vec<KindPredicate>,
        trait_kind_map: &HashMap<TraitId, Arc<Kind>>,
    ) {
        let mut new_kind_bounds: HashMap<Name, Arc<Kind>> = Default::default();
        for p in preds {
            let tyvar = match &p.ty.ty {
                Type::TyVar(tv) => tv.name.clone(),
                _ => {
                    error_exit("currently, trait bound in the context has to be a form `type-variable : trait`.");
                }
            };
            let trait_id = &p.trait_id;
            if !trait_kind_map.contains_key(trait_id) {
                error_exit(&format!("unknown kind: {}", trait_id.to_string()));
            }
            let kind = trait_kind_map[trait_id].clone();
            new_kind_bounds.insert(tyvar, kind);
        }
        for kp in kind_preds {
            let tyvar = kp.name.clone();
            let kind = kp.kind.clone();
            new_kind_bounds.insert(tyvar, kind);
        }
        for (tyvar, kind) in new_kind_bounds {
            if scope.contains_key(&tyvar) {
                if scope[&tyvar] != kind {
                    error_exit(&format!("kind mismatch on {}", tyvar));
                }
            } else {
                scope.insert(tyvar, kind);
            }
        }
    }

    pub fn set_kinds(
        &mut self,
        scope: &mut HashMap<Name, Arc<Kind>>,
        trait_kind_map: &HashMap<TraitId, Arc<Kind>>,
    ) {
        Self::extend_kind_scope(scope, &self.context, &self.kind_preds, trait_kind_map);
        for ctx in &mut self.context {
            ctx.set_kinds(scope);
        }
        self.predicate.set_kinds(scope);
    }
}

#[derive(Clone)]
pub struct QualType {
    pub preds: Vec<Predicate>,
    pub kind_preds: Vec<KindPredicate>,
    pub ty: Arc<TypeNode>,
}

impl QualType {
    // Calculate free type variables.
    pub fn free_vars(&self) -> HashMap<Name, Arc<Kind>> {
        self.ty.free_vars()
    }

    pub fn set_kinds(
        &mut self,
        scope: &mut HashMap<Name, Arc<Kind>>,
        trait_kind_map: &HashMap<TraitId, Arc<Kind>>,
    ) {
        QualPredicate::extend_kind_scope(scope, &self.preds, &self.kind_preds, trait_kind_map);
        for p in &mut self.preds {
            p.set_kinds(scope);
        }
        self.ty = self.ty.set_kinds(scope);
    }
}

// Statement such as "String: Show" or "a: Eq".
#[derive(Clone)]
pub struct Predicate {
    pub trait_id: TraitId,
    pub ty: Arc<TypeNode>,
}

impl Predicate {
    pub fn to_string(&self) -> String {
        format!("{} : {}", self.ty.to_string(), self.trait_id.to_string())
    }

    pub fn set_kinds(&mut self, scope: &HashMap<Name, Arc<Kind>>) {
        self.ty.set_kinds(scope);
    }

    pub fn check_kinds(&self, type_env: &TypeEnv, trait_kind_map: &HashMap<TraitId, Arc<Kind>>) {
        let expected = &trait_kind_map[&self.trait_id];
        let found = self.ty.kind(type_env);
        if *expected != found {
            error_exit(&format!(
                "kind mismatch. Expect: {}, found: {}",
                expected.to_string(),
                found.to_string()
            ))
        }
    }

    pub fn set_namespace_of_tycons_and_traits(
        &mut self,
        type_env: &TypeEnv,
        trait_env: &TraitEnv,
        module_name: &Name,
    ) {
        self.trait_id
            .set_namespace_of_traits(trait_env, module_name);
        self.ty = self.ty.set_namespace_of_tycons(type_env, module_name);
    }
}

// Statement such as "f: * -> *".
#[derive(Clone)]
pub struct KindPredicate {
    pub name: Name,
    pub kind: Arc<Kind>,
}

impl KindPredicate {
    pub fn vec_to_map(vec: Vec<KindPredicate>) -> HashMap<Name, Arc<Kind>> {
        let mut map: HashMap<Name, Arc<Kind>> = Default::default();
        for kp in vec {
            map.insert(kp.name, kp.kind);
        }
        map
    }
}

// Trait environments.
#[derive(Clone, Default)]
pub struct TraitEnv {
    pub traits: HashMap<TraitId, TraitInfo>,
}

impl TraitEnv {
    // TODO: fix duplication with TypeEnv::infer_namespace
    pub fn infer_namespace(&self, ns: &NameSpacedName, module_name: &Name) -> NameSpacedName {
        let candidates = self
            .traits
            .iter()
            .filter_map(|(id, _)| {
                if ns.is_suffix(&id.name) {
                    Some(id.name.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() == 0 {
            error_exit(&format!("unknown type: {}", ns.to_string()))
        } else if candidates.len() == 1 {
            candidates[0].clone()
        } else {
            // candidates.len() >= 2
            let candidates = candidates
                .iter()
                .filter(|name| name.namespace.len() >= 1 && name.namespace.module() == *module_name)
                .collect::<Vec<_>>();
            if candidates.len() == 1 {
                candidates[0].clone()
            } else {
                error_exit("Trait name `{}` is ambiguous.");
            }
        }
    }

    // Set traits.
    pub fn set(
        &mut self,
        trait_infos: Vec<TraitInfo>,
        trait_impls: Vec<TraitInstance>,
        type_env: &TypeEnv,
        module_name: &Name,
    ) {
        for trait_info in trait_infos {
            self.add_trait(trait_info);
        }
        for trait_impl in trait_impls {
            self.add_instance(trait_impl, type_env, module_name);
        }
    }

    // Add a trait to TraitEnv.
    pub fn add_trait(&mut self, info: TraitInfo) {
        // Check duplicate definition.
        if self.traits.contains_key(&info.id) {
            error_exit(&format!(
                "duplicate definition of trait {}.",
                info.id.to_string()
            ));
        }
        self.traits.insert(info.id.clone(), info);
    }

    // Add a instance.
    pub fn add_instance(
        &mut self,
        mut inst: TraitInstance,
        type_env: &TypeEnv,
        module_name: &Name,
    ) {
        let trait_id = &mut inst.qual_pred.predicate.trait_id;

        // Check existaice of trait.
        trait_id.name = self.infer_namespace(&trait_id.name, module_name).clone();

        // Check instance is not head-normal-form.
        if inst.qual_pred.predicate.ty.is_hnf() {
            error_exit("trait implementation cannot be a head-normal-form."); // TODO: better message?
        }

        // Check context is head-normal-form.
        for ctx in &inst.qual_pred.context {
            if !ctx.ty.is_hnf() {
                error_exit("trait implementation context must be a head-normal-form.");
                // TODO: better message?
            }
        }

        self.traits
            .get_mut(trait_id)
            .unwrap()
            .add_instance(inst, type_env)
    }

    // Reduce predicate using trait instances.
    // Returns None when p cannot be satisfied.
    pub fn reduce_to_instance_contexts_one(
        &self,
        p: &Predicate,
        type_env: &TypeEnv,
    ) -> Option<Vec<Predicate>> {
        for inst in &self.traits[&p.trait_id].instances {
            match Substitution::matching(type_env, &inst.qual_pred.predicate.ty, &p.ty) {
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
        return None;
    }

    // Reduce predicate using trait instances (as long as possible).
    // Returns None when p cannot be satisfied.
    // pub fn reduce_to_instance_contexts_alap(
    //     &self,
    //     p: &Predicate,
    //     tycons: &HashMap<String, Arc<Kind>>,
    // ) -> Option<Vec<Predicate>> {
    //     self.reduce_to_instance_contexts_one(p, tycons)
    //         .map(|qs| {
    //             qs.iter()
    //                 .map(|q| self.reduce_to_instance_contexts_alap(&q, tycons))
    //                 .collect::<Option<Vec<_>>>()
    //         })
    //         .flatten()
    //         .map(|vs| vs.concat())
    // }

    // Entailment.
    pub fn entail(&self, ps: &Vec<Predicate>, p: &Predicate, type_env: &TypeEnv) -> bool {
        // If p in contained in ps, then ok.
        for q in ps {
            if q.trait_id == p.trait_id {
                if Substitution::matching(type_env, &q.ty, &p.ty).is_some() {
                    return true;
                }
            }
        }
        // Try reducing by instances.
        match self.reduce_to_instance_contexts_one(p, type_env) {
            Some(ctxs) => {
                let mut all_ok = true;
                for ctx in ctxs {
                    if !self.entail(ps, &ctx, type_env) {
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
    pub fn reduce_to_hnf(&self, p: &Predicate, type_env: &TypeEnv) -> Option<Vec<Predicate>> {
        if p.ty.is_hnf() {
            return Some(vec![p.clone()]);
        }
        self.reduce_to_instance_contexts_one(p, type_env)
            .map(|ctxs| self.reduce_to_hnfs(&ctxs, type_env))
            .flatten()
    }

    // Reduce predicates to head normal form.
    pub fn reduce_to_hnfs(
        &self,
        ps: &Vec<Predicate>,
        type_env: &TypeEnv,
    ) -> Option<Vec<Predicate>> {
        let mut ret: Vec<Predicate> = Default::default();
        for p in ps {
            match self.reduce_to_hnf(p, type_env) {
                Some(mut v) => ret.append(&mut v),
                None => return None,
            }
        }
        Some(ret)
    }

    // Simplify a set of predicates by entail.
    pub fn simplify_predicates(&self, ps: &Vec<Predicate>, type_env: &TypeEnv) -> Vec<Predicate> {
        let mut ps = ps.clone();
        let mut i = 0 as usize;
        while i < ps.len() {
            let qs: Vec<Predicate> = ps
                .iter()
                .enumerate()
                .filter_map(|(j, p)| if i == j { None } else { Some(p.clone()) })
                .collect();
            if self.entail(&qs, &ps[i], type_env) {
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
    // Returns None when p cannot be satisfied.
    // pub fn reduce(
    //     &self,
    //     ps: &Vec<Predicate>,
    //     tycons: &HashMap<String, Arc<Kind>>,
    // ) -> Option<Vec<Predicate>> {
    //     let ret = ps
    //         .iter()
    //         .map(|p| self.reduce_to_instance_contexts_alap(p, tycons))
    //         .collect::<Option<Vec<_>>>()
    //         .map(|vs| vs.concat())
    //         .map(|ps| self.simplify_predicates(&ps, tycons));

    //     // Every predicate has to be hnf.
    //     assert!(ret.is_none() || ret.as_ref().unwrap().iter().all(|p| p.ty.is_hnf()));
    //     ret
    // }

    // Context reduction.
    // Returns qs when satisfaction of ps are reduced to qs.
    // In particular, returns empty when ps are satisfied.
    // Returns None when p cannot be satisfied.
    pub fn reduce(&self, ps: &Vec<Predicate>, type_env: &TypeEnv) -> Option<Vec<Predicate>> {
        let ret = self
            .reduce_to_hnfs(ps, type_env)
            .map(|ps| self.simplify_predicates(&ps, type_env));

        // Every predicate has to be hnf.
        assert!(ret.is_none() || ret.as_ref().unwrap().iter().all(|p| p.ty.is_hnf()));
        ret
    }

    // Set each TraitInfo's kind.
    pub fn set_kinds(&mut self) {
        for (_id, ti) in &mut self.traits {
            ti.set_trait_kind();
        }
    }

    pub fn trait_kind_map(&self) -> HashMap<TraitId, Arc<Kind>> {
        let mut res: HashMap<TraitId, Arc<Kind>> = HashMap::default();
        for (id, ti) in &self.traits {
            res.insert(id.clone(), ti.type_var.kind.clone());
        }
        res
    }

    pub fn check_kinds(&self, type_env: &TypeEnv, trait_kind_map: &HashMap<TraitId, Arc<Kind>>) {
        for (_, trait_info) in &self.traits {
            for (name, _) in &trait_info.methods {
                trait_info
                    .method_scheme(&name)
                    .check_kinds(type_env, trait_kind_map);
            }
        }
    }
}
