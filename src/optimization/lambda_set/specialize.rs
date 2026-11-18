// The copies of the components a program needs, each given by the concrete lambda sets of its
// parameters.

use super::{
    concrete::{PendingElem, Ref, SetId, SetStore},
    infer::{Analysis, Component, Lambda},
    solver::{Member, Owner, SetVar, Solver},
};
use crate::{
    constants::{LS_COPIES_PER_REFERENCE, LS_COPY_LIMIT_BASE},
    misc::{Map, Set},
};
use std::collections::VecDeque;

/// A copy of a component: the concrete lambda set given to each of its parameters.
pub struct Instance {
    /// The component, as its index into `Analysis::components`.
    pub component: usize,
    /// The set given to each parameter, in the order of `Component::params`.
    pub args: Vec<SetId>,
    /// The set of each variable of the component in this copy, in the order of `Component::vars`.
    pub var_sets: Vec<SetId>,
}

impl Instance {
    /// The set this copy gives the variable `v` of `component`, the copy's component.
    pub fn set_of(&self, component: &Component, v: SetVar) -> SetId {
        self.var_sets[component.var_index[&v]]
    }

    /// The sets this copy gives the parameters `lambda` depends on (`Lambda::param_deps`), `lambda`
    /// a lambda of the copy's component.
    pub fn lambda_args(&self, lambda: &Lambda) -> Vec<SetId> {
        lambda.param_deps.iter().map(|p| self.args[*p]).collect()
    }
}

/// The copies a program needs, found from its entry and the values it exports.
pub struct Specialization {
    /// The concrete sets the copies name.
    pub store: SetStore,
    /// The copies, in the order they were first asked for: the copy of the entry's component first.
    pub instances: Vec<Instance>,
    /// The copy a call of a component goes to, by the component and the sets the call gives its
    /// parameters: the copy whose `component` and `args` these are, or, for a call widened past the
    /// limit on copies, the copy giving every parameter the set of `Elem::Dynamic` alone.
    pub instance_of: Map<(usize, Vec<SetId>), usize>,
    /// The sets passed at the calls widened past the limit on copies, which are represented by
    /// closures, as the copy the call goes to takes closures.
    pub widened_sets: Set<SetId>,
}

/// The copies `analysis` asks for, starting from the component of the entry: each copy asks for a
/// copy of every component it refers to, given by the sets its reference gives the parameters
/// there. The copies of a component are as many as the distinct ways it is asked for, up to
/// `LS_COPY_LIMIT_BASE` and `LS_COPIES_PER_REFERENCE` per reference to it. A way asked for past that
/// is widened: it goes to the copy giving every parameter the set of `Elem::Dynamic` alone. The
/// components refer to one another without a cycle, so the walk ends.
///
/// Each copy is given the set of every variable of its component, which is what types its nodes.
pub fn specialize(analysis: &mut Analysis) -> Specialization {
    let mut store = SetStore::new();
    let dynamic = store.intern(vec![vec![PendingElem::Dynamic]])[0];
    let mut references = vec![0usize; analysis.components.len()];
    for site in analysis.components.iter().flat_map(|c| &c.sites) {
        references[site.component] += 1;
    }
    let mut copies = Copies {
        instances: vec![],
        instance_of: Map::default(),
        queue: VecDeque::new(),
        limits: references
            .iter()
            .map(|n| LS_COPY_LIMIT_BASE + LS_COPIES_PER_REFERENCE * n)
            .collect(),
        counts: vec![0; analysis.components.len()],
        dynamic,
        widened_sets: Set::default(),
    };
    copies.request(analysis.entry_component(), vec![]);
    let param_indices: Vec<Map<SetVar, usize>> = analysis
        .components
        .iter()
        .map(|c| c.params.iter().enumerate().map(|(i, p)| (*p, i)).collect())
        .collect();
    while let Some(k) = copies.queue.pop_front() {
        let component = copies.instances[k].component;
        let args = copies.instances[k].args.clone();
        let mut equations = CopyEquations {
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
                let refs = site.args.iter().map(|a| equations.reference(*a)).collect();
                (site.component, refs)
            })
            .collect();
        let var_refs: Vec<Ref> = analysis.components[component]
            .vars
            .iter()
            .map(|v| {
                assert_eq!(
                    equations.solver.find(*v),
                    *v,
                    "a variable of a component is not the root it was recorded as"
                );
                equations.reference(*v)
            })
            .collect();
        let system = equations.finish();
        let ids = store.intern(system);
        let resolve = |r: &Ref| match r {
            Ref::Known(id) => *id,
            Ref::Local(i) => ids[*i],
        };
        copies.instances[k].var_sets = var_refs.iter().map(resolve).collect();
        for (callee, refs) in site_refs {
            copies.request(callee, refs.iter().map(resolve).collect());
        }
    }
    Specialization {
        store,
        instances: copies.instances,
        instance_of: copies.instance_of,
        widened_sets: copies.widened_sets,
    }
}

/// The copies found so far, and those of them whose references are still to be followed.
struct Copies {
    /// The copies, in the order they were first asked for.
    instances: Vec<Instance>,
    /// The copy each call goes to, as `Specialization::instance_of` holds it.
    instance_of: Map<(usize, Vec<SetId>), usize>,
    /// The copies whose references are still to be followed, as indices into `instances`.
    queue: VecDeque<usize>,
    /// How many copies of each component are made before a call of it is widened.
    limits: Vec<usize>,
    /// How many copies of each component are made.
    counts: Vec<usize>,
    /// The set of `Elem::Dynamic` alone, which a widened call gives every parameter.
    dynamic: SetId,
    /// The sets passed at the widened calls.
    widened_sets: Set<SetId>,
}

impl Copies {
    /// Asks for the copy of `component` given by `args`, adding it the first time it is asked for.
    /// Past the limit on the copies of `component`, the call is widened instead: it goes to the copy
    /// giving every parameter the set of `Elem::Dynamic` alone, and `args` are recorded as widened.
    fn request(&mut self, component: usize, args: Vec<SetId>) {
        if self.instance_of.contains_key(&(component, args.clone())) {
            return;
        }
        if self.counts[component] < self.limits[component] {
            self.add(component, args);
            return;
        }
        let widened = vec![self.dynamic; args.len()];
        let id = match self.instance_of.get(&(component, widened.clone())) {
            Some(id) => *id,
            None => self.add(component, widened),
        };
        self.widened_sets.extend(args.iter().copied());
        self.instance_of.insert((component, args), id);
    }

    /// Adds the copy of `component` given by `args`, to be followed, and returns its index.
    fn add(&mut self, component: usize, args: Vec<SetId>) -> usize {
        let id = self.instances.len();
        self.instance_of.insert((component, args.clone()), id);
        self.instances.push(Instance {
            component,
            args,
            var_sets: vec![],
        });
        self.counts[component] += 1;
        self.queue.push_back(id);
        id
    }
}

/// The equations of the sets of the variables of one copy of a component.
struct CopyEquations<'a> {
    /// The variables of the program, read for the members of each one the copy refers to.
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

impl<'a> CopyEquations<'a> {
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
