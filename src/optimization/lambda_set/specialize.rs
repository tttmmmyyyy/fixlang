// The copies of the components a program needs, each given by the concrete lambda sets of its
// parameters.

use super::{
    concrete::{PendingElem, Ref, SetId, SetStore},
    infer::Analysis,
    solver::{Member, Owner, SetVar},
};
use crate::misc::Map;
use std::collections::VecDeque;

/// A copy of a component: the concrete lambda set given to each of its parameters.
pub struct Instance {
    pub component: usize,
    pub args: Vec<SetId>,
}

/// The copies a program needs, found from its entry and the values it exports.
pub struct Specialization {
    pub store: SetStore,
    pub instances: Vec<Instance>,
}

/// The copies `analysis` asks for, starting from the component of the entry: each copy asks for a
/// copy of every component it refers to, given by the sets its reference gives the parameters
/// there. The copies of a component are as many as the distinct ways it is asked for. The
/// components refer to one another without a cycle, so the walk ends.
pub fn specialize(analysis: &mut Analysis) -> Specialization {
    let mut store = SetStore::new();
    let mut instances: Vec<Instance> = vec![];
    let mut instance_of: Map<(usize, Vec<SetId>), usize> = Map::default();
    let mut queue: VecDeque<usize> = VecDeque::new();
    let root = (analysis.root(), vec![]);
    instance_of.insert(root.clone(), 0);
    instances.push(Instance {
        component: root.0,
        args: root.1,
    });
    queue.push_back(0);
    let param_indices: Vec<Map<SetVar, usize>> = analysis
        .components
        .iter()
        .map(|c| c.params.iter().enumerate().map(|(i, p)| (*p, i)).collect())
        .collect();
    while let Some(k) = queue.pop_front() {
        let component = instances[k].component;
        let args = instances[k].args.clone();
        let site_count = analysis.components[component].sites.len();
        let mut solve = Solve {
            analysis: &mut *analysis,
            param_index: &param_indices[component],
            args: &args,
            local_of: Map::default(),
            system: vec![],
        };
        let mut site_refs: Vec<(usize, Vec<Ref>)> = vec![];
        for s in 0..site_count {
            let site = &solve.analysis.components[component].sites[s];
            let callee = site.component;
            let site_args = site.args.clone();
            let refs = site_args.iter().map(|a| solve.reference(*a)).collect();
            site_refs.push((callee, refs));
        }
        let system = solve.finish();
        let ids = store.intern(system);
        for (callee, refs) in site_refs {
            let callee_args: Vec<SetId> = refs
                .iter()
                .map(|r| match r {
                    Ref::Known(id) => *id,
                    Ref::Local(i) => ids[*i],
                })
                .collect();
            let key = (callee, callee_args);
            if !instance_of.contains_key(&key) {
                let id = instances.len();
                instance_of.insert(key.clone(), id);
                instances.push(Instance {
                    component: key.0,
                    args: key.1,
                });
                queue.push_back(id);
            }
        }
    }
    Specialization { store, instances }
}

/// The equations of the sets of the variables of one copy of a component.
struct Solve<'a> {
    analysis: &'a mut Analysis,
    /// The position of each parameter of the component.
    param_index: &'a Map<SetVar, usize>,
    /// The sets the copy gives the parameters.
    args: &'a [SetId],
    /// The equation of each variable that is not a parameter, by its root.
    local_of: Map<SetVar, usize>,
    /// The equations, each `None` until the members of its variable are read.
    system: Vec<Option<Vec<PendingElem>>>,
}

impl<'a> Solve<'a> {
    /// The set of the variable `v` in the copy: the set given to it where it is a parameter, and
    /// its equation otherwise.
    fn reference(&mut self, v: SetVar) -> Ref {
        let root = self.analysis.solver.find(v);
        if let Some(p) = self.param_index.get(&root) {
            return Ref::Known(self.args[*p]);
        }
        if let Some(i) = self.local_of.get(&root) {
            return Ref::Local(*i);
        }
        let i = self.system.len();
        self.local_of.insert(root, i);
        self.system.push(None);
        let members = self.analysis.solver.members(root).to_vec();
        let elems = members
            .iter()
            .map(|m| match m {
                Member::Dynamic(_) => PendingElem::Dynamic,
                Member::Lam {
                    lam,
                    owner: Owner::Instance(owner_args),
                } => PendingElem::Lam(
                    *lam,
                    owner_args.iter().map(|a| self.reference(*a)).collect(),
                ),
                Member::Lam {
                    owner: Owner::Current,
                    ..
                } => unreachable!("every lambda has its owner given after inference"),
            })
            .collect();
        self.system[i] = Some(elems);
        Ref::Local(i)
    }

    /// The equations, once every variable referred to has been read.
    fn finish(self) -> Vec<Vec<PendingElem>> {
        self.system.into_iter().map(|e| e.unwrap()).collect()
    }
}
