// The copies of the components a program needs, each given by the concrete lambda sets of its
// parameters.

use super::{
    concrete::{Elem, PendingElem, Ref, SetId, SetStore},
    infer::{Analysis, Component, Lambda},
    solver::{Member, Owner, SetVar, Solver},
};
use crate::{
    constants::LS_COPY_BUDGET_BASE,
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
    /// The index into `instances` of each copy, by its component and the sets of its parameters:
    /// the copy whose `component` and `args` these are.
    pub instance_of: Map<(usize, Vec<SetId>), usize>,
}

/// The copies `analysis` asks for, starting from the component of the entry: each copy asks for a
/// copy of every component it refers to, given by the sets its reference gives the parameters
/// there. The copies of a component are as many as the distinct ways it is asked for. The
/// components refer to one another without a cycle, so the walk ends.
///
/// Each copy is given the set of every variable of its component, which is what types its nodes.
/// The set of a variable holding `Member::Dynamic` is the set holding `Elem::Dynamic` alone: its
/// values are closures whatever lambdas build them, so the copies given them need no telling apart.
///
/// The walk counts the bodies each component is asked for: the distinct lists of sets its copies
/// give `Component::body_params`. It stops at the first component asked for more bodies than its
/// budget (`copy_budgets`) and returns `Err` with that component. A copy asks the same of the
/// components it refers to as every other copy of its body, so a walk lists, for each reference
/// of the program, at most one copy per body of the referring component.
///
/// A component inferred as past its copy budget (`Component::past_copy_budget`) is asked for one
/// body: every reference gives its `Component::body_params` the set holding `Elem::Dynamic` alone.
pub fn specialize(analysis: &mut Analysis) -> Result<Specialization, usize> {
    let budgets = copy_budgets(analysis);
    let mut store = SetStore::new();
    let mut copies = Copies {
        instances: vec![],
        instance_of: Map::default(),
        queue: VecDeque::new(),
        bodies: vec![Set::default(); analysis.components.len()],
    };
    copies.request(analysis, analysis.entry_component(), vec![]);
    let param_indices: Vec<Map<SetVar, usize>> = analysis
        .components
        .iter()
        .map(|c| c.params.iter().enumerate().map(|(i, p)| (*p, i)).collect())
        .collect();
    // What the first copy of each body asked for, which every copy of the body asks for too.
    let mut requests_of_body: Map<(usize, Vec<SetId>), Vec<(usize, Vec<SetId>)>> = Map::default();
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
        let requests: Vec<(usize, Vec<SetId>)> = site_refs
            .iter()
            .map(|(callee, refs)| (*callee, refs.iter().map(resolve).collect()))
            .collect();
        let body = analysis.components[component].body_of(&args);
        match requests_of_body.get(&(component, body.clone())) {
            Some(known) => assert_eq!(
                *known, requests,
                "two copies of one body of the component {} ask for different copies",
                component
            ),
            None => {
                requests_of_body.insert((component, body), requests.clone());
            }
        }
        for (callee, callee_args) in requests {
            let callee_component = &analysis.components[callee];
            if callee_component.past_copy_budget {
                for p in &callee_component.body_params {
                    assert!(
                        store.elems(callee_args[*p]) == [Elem::Dynamic],
                        "a reference gives the component {}, past its copy budget, a set of lambdas",
                        callee
                    );
                }
            }
            let bodies = copies.request(analysis, callee, callee_args);
            if bodies > budgets[callee] {
                return Err(callee);
            }
        }
    }
    Ok(Specialization {
        store,
        instances: copies.instances,
        instance_of: copies.instance_of,
    })
}

/// The number of bodies each component can be asked for: `LS_COPY_BUDGET_BASE` plus one for each
/// reference on the way to it from the entry (`count_reaching_references`).
fn copy_budgets(analysis: &Analysis) -> Vec<usize> {
    count_reaching_references(analysis)
        .into_iter()
        .map(|reaching| LS_COPY_BUDGET_BASE + reaching)
        .collect()
}

/// The copies found so far, and those of them whose references are still to be followed.
struct Copies {
    /// The copies, in the order they were first asked for.
    instances: Vec<Instance>,
    /// The index into `instances` of each copy, by its component and the sets of its parameters.
    instance_of: Map<(usize, Vec<SetId>), usize>,
    /// The copies whose references are still to be followed, as indices into `instances`.
    queue: VecDeque<usize>,
    /// The bodies each component has been asked for: the lists of sets its copies give
    /// `Component::body_params`.
    bodies: Vec<Set<Vec<SetId>>>,
}

impl Copies {
    /// Asks for the copy of `component` given by `args`, adding it the first time it is asked for.
    /// Returns how many bodies `component` has been asked for.
    fn request(&mut self, analysis: &Analysis, component: usize, args: Vec<SetId>) -> usize {
        let key = (component, args);
        if self.instance_of.contains_key(&key) {
            return self.bodies[component].len();
        }
        self.bodies[component].insert(analysis.components[component].body_of(&key.1));
        let id = self.instances.len();
        self.instance_of.insert(key.clone(), id);
        self.instances.push(Instance {
            component: key.0,
            args: key.1,
            var_sets: vec![],
        });
        self.queue.push_back(id);
        self.bodies[component].len()
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
    /// its equation otherwise. The equation of a variable holding `Member::Dynamic` holds
    /// `PendingElem::Dynamic` alone.
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
        let elems = if members.iter().any(|m| matches!(m, Member::Dynamic(_))) {
            vec![PendingElem::Dynamic]
        } else {
            members
                .iter()
                .map(|m| match m {
                    Member::Dynamic(_) => unreachable!("the variable holds no Dynamic"),
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
                .collect()
        };
        self.system[i] = Some(elems);
        Ref::Local(i)
    }

    /// The equations, once every variable referred to has been read.
    fn finish(self) -> Vec<Vec<PendingElem>> {
        self.system.into_iter().map(|e| e.unwrap()).collect()
    }
}

/// For each component, how many references (`Component::sites`) lie on a chain of references from
/// the entry's component to it.
fn count_reaching_references(analysis: &Analysis) -> Vec<usize> {
    let count = analysis.components.len();
    let mut reached_from: Vec<Set<usize>> = vec![Set::default(); count];
    for c in (0..count).rev() {
        let mut above = reached_from[c].clone();
        above.insert(c);
        for site in &analysis.components[c].sites {
            assert!(
                site.component < c,
                "the component {} refers to the component {}, which comes after it",
                c,
                site.component
            );
            reached_from[site.component].extend(above.iter().copied());
        }
    }
    reached_from
        .iter()
        .enumerate()
        .map(|(c, from)| {
            from.iter()
                .map(|a| {
                    analysis.components[*a]
                        .sites
                        .iter()
                        .filter(|site| site.component == c || from.contains(&site.component))
                        .count()
                })
                .sum()
        })
        .collect()
}
