// The copies of the components a program needs, each given by the concrete lambda sets of its
// parameters.

use super::{
    concrete::{PendingElem, Ref, SetId, SetStore},
    infer::Analysis,
    solver::{Member, Owner, SetVar, Solver},
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
    let mut copies = Copies {
        instances: vec![],
        instance_of: Map::default(),
        queue: VecDeque::new(),
    };
    copies.request(analysis.root(), vec![]);
    let param_indices: Vec<Map<SetVar, usize>> = analysis
        .components
        .iter()
        .map(|c| c.params.iter().enumerate().map(|(i, p)| (*p, i)).collect())
        .collect();
    while let Some(k) = copies.queue.pop_front() {
        let component = copies.instances[k].component;
        let args = copies.instances[k].args.clone();
        let mut solve = Solve {
            solver: &mut analysis.solver,
            param_index: &param_indices[component],
            args: &args,
            local_of: Map::default(),
            system: vec![],
        };
        let site_refs: Vec<(usize, Vec<Ref>)> = analysis.components[component]
            .sites
            .iter()
            .map(|site| {
                let refs = site.args.iter().map(|a| solve.reference(*a)).collect();
                (site.component, refs)
            })
            .collect();
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
            copies.request(callee, callee_args);
        }
    }
    Specialization {
        store,
        instances: copies.instances,
    }
}

/// The copies found so far, and those of them whose references are still to be followed.
struct Copies {
    instances: Vec<Instance>,
    /// The index into `instances` of each copy, by its component and the sets of its parameters.
    instance_of: Map<(usize, Vec<SetId>), usize>,
    /// The copies whose references are still to be followed, as indices into `instances`.
    queue: VecDeque<usize>,
}

impl Copies {
    /// Asks for the copy of `component` given by `args`, adding it the first time it is asked for.
    fn request(&mut self, component: usize, args: Vec<SetId>) {
        let key = (component, args);
        if self.instance_of.contains_key(&key) {
            return;
        }
        let id = self.instances.len();
        self.instance_of.insert(key.clone(), id);
        self.instances.push(Instance {
            component: key.0,
            args: key.1,
        });
        self.queue.push_back(id);
    }
}

/// The equations of the sets of the variables of one copy of a component.
struct Solve<'a> {
    solver: &'a mut Solver,
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
        let root = self.solver.find(v);
        if let Some(p) = self.param_index.get(&root) {
            return Ref::Known(self.args[*p]);
        }
        if let Some(i) = self.local_of.get(&root) {
            return Ref::Local(*i);
        }
        let i = self.system.len();
        self.local_of.insert(root, i);
        self.system.push(None);
        let members = self.solver.members(root).to_vec();
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
