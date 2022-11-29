use super::*;

// Identifier to spacify trait.
#[derive(Hash, Eq, PartialEq, Clone)]
struct TraitId {
    name: String,
    // TODO: add namespace.
}

impl TraitId {
    pub fn to_string(&self) -> String {
        self.name.clone()
    }
}

// Information on trait.
struct TraitInfo {
    id: TraitId,
    instances: Vec<QualPredicate>,
}

impl TraitInfo {
    // Add a instance to a trait.
    pub fn add_instance(&mut self, inst: QualPredicate, tycons: &HashMap<String, Arc<Kind>>) {
        // Check trait id.
        assert!(self.id == inst.predicate.trait_id);

        // Check overlapping instance.
        for i in &self.instances {
            if Substitution::unify(tycons, &inst.predicate.ty, &i.predicate.ty).is_some() {
                error_exit("overlapping instance.");
            }
        }

        self.instances.push(inst)
    }
}

// Qualified predicate. Statement such as "impl Array a : Eq for a : Eq {}".
struct QualPredicate {
    context: Vec<Predicate>,
    predicate: Predicate,
}

// Statement such as "String: Show" or "a: Eq".
pub struct Predicate {
    trait_id: TraitId,
    pub ty: Arc<TypeNode>,
}

// Trait environments.
struct TraitEnv {
    traits: HashMap<TraitId, TraitInfo>,
}

impl TraitEnv {
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
    pub fn add_instance(&mut self, inst: QualPredicate, tycons: &HashMap<String, Arc<Kind>>) {
        let trait_id = &inst.predicate.trait_id;

        // Check existaice of trait.
        if !self.traits.contains_key(trait_id) {
            error_exit(&format!("no trait `{}` defined", trait_id.to_string()));
        }

        self.traits
            .get_mut(trait_id)
            .unwrap()
            .add_instance(inst, tycons)
    }

    // Entailment
    pub fn entail(
        &self,
        ps: &[&Predicate],
        p: &Predicate,
        tycons: &HashMap<String, Arc<Kind>>,
    ) -> bool {
        // If p in contained in ps, then ok.
        for q in ps {
            if q.trait_id == p.trait_id {
                if Substitution::matching(tycons, &q.ty, &p.ty).is_some() {
                    return true;
                }
            }
        }
        // Try instances.
        for qual in &self.traits[&p.trait_id].instances {
            match Substitution::matching(tycons, &qual.predicate.ty, &p.ty) {
                Some(s) => {
                    let mut all_context_ok = true;
                    for c in &qual.context {
                        if !self.entail(ps, &c, tycons) {
                            all_context_ok = false;
                            break;
                        }
                    }
                    if all_context_ok {
                        return true;
                    }
                }
                None => {}
            }
        }
        return false;
    }
}
