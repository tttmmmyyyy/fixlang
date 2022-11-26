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
    instances: Vec<TraitInstance>,
}

impl TraitInfo {
    // Add a instance to a trait.
    pub fn add_instance(&mut self, inst: TraitInstance) {
        // Check overlapping instance.
        for i in &self.instances {
            let subst = Substitution::default();
        }
    }
}

// Trait instance declaration such as "impl Array a : Eq for a : Eq {}".
struct TraitInstance {
    restriction: Vec<Predicate>,
    instance: Predicate,
}

// Statement such as "String: Show" or "a: Eq".
struct Predicate {
    trait_id: TraitId,
    ty: Arc<TypeNode>,
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

    // Add a instance to a trait.
}
