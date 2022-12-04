use std::fmt::format;

use super::*;

// Identifier to spacify trait.
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TraitId {
    name: String,
    // TODO: add namespace.
}

impl TraitId {
    pub fn to_string(&self) -> String {
        self.name.clone()
    }
}

// Information on trait.
#[derive(Clone)]
pub struct TraitInfo {
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
#[derive(Clone)]
pub struct QualPredicate {
    context: Vec<Predicate>,
    predicate: Predicate,
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
}

// Trait environments.
#[derive(Clone, Default)]
pub struct TraitEnv {
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

    // Reduce predicate using trait instances.
    // Returns None when p cannot be satisfied.
    pub fn reduce_to_instance_contexts_one(
        &self,
        p: &Predicate,
        tycons: &HashMap<String, Arc<Kind>>,
    ) -> Option<Vec<Predicate>> {
        for qual in &self.traits[&p.trait_id].instances {
            match Substitution::matching(tycons, &qual.predicate.ty, &p.ty) {
                Some(s) => {
                    let ret = qual
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
    pub fn reduce_to_instance_contexts_alap(
        &self,
        p: &Predicate,
        tycons: &HashMap<String, Arc<Kind>>,
    ) -> Option<Vec<Predicate>> {
        self.reduce_to_instance_contexts_one(p, tycons)
            .map(|qs| {
                qs.iter()
                    .map(|q| self.reduce_to_instance_contexts_alap(&q, tycons))
                    .collect::<Option<Vec<_>>>()
            })
            .flatten()
            .map(|vs| vs.concat())
    }

    // Entailment.
    pub fn entail(
        &self,
        ps: &Vec<Predicate>,
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
        // Try reducing by instances.
        match self.reduce_to_instance_contexts_one(p, tycons) {
            Some(ctxs) => {
                let mut all_ok = true;
                for ctx in ctxs {
                    if !self.entail(ps, &ctx, tycons) {
                        all_ok = false;
                        break;
                    }
                }
                all_ok
            }
            None => false,
        }
    }

    // // Reduce a predicate to head normal form.
    // pub fn reduce_to_hnf(
    //     &self,
    //     p: &Predicate,
    //     tycons: &HashMap<String, Arc<Kind>>,
    // ) -> Option<Vec<Predicate>> {
    //     if p.ty.isHnf() {
    //         return Some(vec![p.clone()]);
    //     }
    //     self.reduce_to_instance_contexts_one(p, tycons)
    //         .map(|ctxs| self.reduce_to_hnfs(&ctxs, tycons))
    //         .flatten()
    // }

    // // Reduce predicates to head normal form.
    // pub fn reduce_to_hnfs(
    //     &self,
    //     ps: &Vec<Predicate>,
    //     tycons: &HashMap<String, Arc<Kind>>,
    // ) -> Option<Vec<Predicate>> {
    //     let mut ret: Vec<Predicate> = Default::default();
    //     for p in ps {
    //         match self.reduce_to_hnf(p, tycons) {
    //             Some(mut v) => ret.append(&mut v),
    //             None => return None,
    //         }
    //     }
    //     Some(ret)
    // }

    // Simplify a set of predicates by entail.
    pub fn simplify_predicates(
        &self,
        ps: &Vec<Predicate>,
        tycons: &HashMap<String, Arc<Kind>>,
    ) -> Vec<Predicate> {
        let mut ps = ps.clone();
        let mut i = 0 as usize;
        while i < ps.len() {
            let qs: Vec<Predicate> = ps
                .iter()
                .enumerate()
                .filter_map(|(j, p)| if i == j { None } else { Some(p.clone()) })
                .collect();
            if self.entail(&qs, &ps[i], tycons) {
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
    pub fn reduce(
        &self,
        ps: &Vec<Predicate>,
        tycons: &HashMap<String, Arc<Kind>>,
    ) -> Option<Vec<Predicate>> {
        let ret = ps
            .iter()
            .map(|p| self.reduce_to_instance_contexts_alap(p, tycons))
            .collect::<Option<Vec<_>>>()
            .map(|vs| vs.concat())
            .map(|ps| self.simplify_predicates(&ps, tycons));

        // Every predicate has to be hnf.
        assert!(ret.is_none() || ret.as_ref().unwrap().iter().all(|p| p.ty.is_hnf()));
        ret
    }
}
