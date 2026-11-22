// The copies of the components a program needs, each given by the concrete lambda sets of its
// parameters.

use super::{
    concrete::{PendingElem, Ref, SetId, SetStore},
    infer::{Analysis, Component, Lambda},
    solver::{Member, Owner, SetVar, Solver},
};
use crate::{
    ast::name::FullName,
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
    /// The components collapsed past their copy budget (`specialize`).
    pub collapse: Collapse,
    /// The sets the references to a collapsed component give to its `Component::body_params`. Its
    /// copies are given `Collapse::dynamic` there instead, so the values these sets stand for reach
    /// them as closures, and these sets are represented by closures too.
    pub collapsed_args: Vec<SetId>,
}

impl Specialization {
    /// The copy that a reference giving the parameters of the component `component` the sets
    /// `args` reaches, as an index into `instances`.
    pub fn copy_reached(
        &self,
        analysis: &Analysis,
        component: usize,
        mut args: Vec<SetId>,
    ) -> usize {
        self.collapse.replace_args(analysis, component, &mut args);
        self.instance_of[&(component, args)]
    }
}

/// The components collapsed past their copy budget, and what their copies are given instead of the
/// sets a reference gives their `Component::body_params`.
pub struct Collapse {
    /// The collapsed components, as indices into `Analysis::components`.
    pub components: Set<usize>,
    /// The set holding `Elem::Dynamic` alone, which a collapsed component is given at its
    /// `Component::body_params`; `None` where no component is collapsed.
    pub dynamic: Option<SetId>,
}

impl Collapse {
    /// Replaces by `dynamic` the sets `args` gives the `Component::body_params` of `component`
    /// where it is collapsed, and returns the sets replaced.
    fn replace_args(
        &self,
        analysis: &Analysis,
        component: usize,
        args: &mut [SetId],
    ) -> Vec<SetId> {
        if !self.components.contains(&component) {
            return vec![];
        }
        analysis.components[component]
            .body_params
            .iter()
            .map(|p| std::mem::replace(&mut args[*p], self.dynamic.unwrap()))
            .collect()
    }
}

/// The copies `analysis` asks for, starting from the component of the entry: each copy asks for a
/// copy of every component it refers to, given by the sets its reference gives the parameters
/// there. The copies of a component are as many as the distinct ways it is asked for. The
/// components refer to one another without a cycle, so the walk ends.
///
/// A component asked for in more ways than its budget (`copy_budgets`) is collapsed: its copies are
/// given a set holding `Elem::Dynamic` alone at `Component::body_params`, and so are the lambdas of
/// the component wherever a set names them, so that they share one body. The walk starts again
/// each time it meets a component going past its budget, with that component collapsed, until no
/// component goes past. A walk stops at the first copy past a budget, so each walk lists at most
/// the budgets' worth of copies.
///
/// Each copy is given the set of every variable of its component, which is what types its nodes.
pub fn specialize(analysis: &mut Analysis) -> Specialization {
    let budgets = copy_budgets(analysis);
    let mut collapsed: Set<usize> = Set::default();
    loop {
        match list_copies(analysis, &budgets, &collapsed) {
            Ok(spec) => return spec,
            Err(component) => {
                collapsed.insert(component);
            }
        }
    }
}

/// The copies `analysis` asks for with the components `collapsed` collapsed, as `specialize`
/// describes; `Err` with the first component that goes past its budget among the others.
fn list_copies(
    analysis: &mut Analysis,
    budgets: &[usize],
    collapsed: &Set<usize>,
) -> Result<Specialization, usize> {
    let mut store = SetStore::new();
    // Interned only where a component is collapsed, so that a program none goes past keeps the
    // numbering of its sets.
    let collapse = Collapse {
        components: collapsed.clone(),
        dynamic: (!collapsed.is_empty()).then(|| store.intern(vec![vec![PendingElem::Dynamic]])[0]),
    };
    let mut copies = Copies {
        instances: vec![],
        instance_of: Map::default(),
        queue: VecDeque::new(),
        counts: vec![0; analysis.components.len()],
    };
    copies.request(analysis.entry_component(), vec![]);
    let param_indices: Vec<Map<SetVar, usize>> = analysis
        .components
        .iter()
        .map(|c| c.params.iter().enumerate().map(|(i, p)| (*p, i)).collect())
        .collect();
    let mut collapsed_args = vec![];
    while let Some(k) = copies.queue.pop_front() {
        let component = copies.instances[k].component;
        let args = copies.instances[k].args.clone();
        let mut equations = CopyEquations {
            solver: &mut analysis.solver,
            lambdas: &analysis.lambdas,
            components: &analysis.components,
            component_of: &analysis.component_of,
            collapse: &collapse,
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
            let mut callee_args: Vec<SetId> = refs.iter().map(resolve).collect();
            collapsed_args.extend(collapse.replace_args(analysis, callee, &mut callee_args));
            let count = copies.request(callee, callee_args);
            if count > budgets[callee] && !collapsed.contains(&callee) {
                return Err(callee);
            }
        }
    }
    collapsed_args.sort();
    collapsed_args.dedup();
    Ok(Specialization {
        store,
        instances: copies.instances,
        instance_of: copies.instance_of,
        collapse,
        collapsed_args,
    })
}

/// The number of copies each component can be given before it is collapsed: `LS_COPY_BUDGET_BASE`
/// plus one for each reference on the way to it from the entry (`count_reaching_references`).
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
    /// How many copies of each component have been asked for.
    counts: Vec<usize>,
}

impl Copies {
    /// Asks for the copy of `component` given by `args`, adding it the first time it is asked for.
    /// Returns how many copies of `component` have been asked for.
    fn request(&mut self, component: usize, args: Vec<SetId>) -> usize {
        let key = (component, args);
        if self.instance_of.contains_key(&key) {
            return self.counts[component];
        }
        self.counts[component] += 1;
        let id = self.instances.len();
        self.instance_of.insert(key.clone(), id);
        self.instances.push(Instance {
            component: key.0,
            args: key.1,
            var_sets: vec![],
        });
        self.queue.push_back(id);
        self.counts[component]
    }
}

/// The equations of the sets of the variables of one copy of a component.
struct CopyEquations<'a> {
    /// The variables of the program, read for the members of each one the copy refers to.
    solver: &'a mut Solver,
    /// The lambdas of the program.
    lambdas: &'a [Lambda],
    /// The components of the program.
    components: &'a [Component],
    /// The component of each symbol.
    component_of: &'a Map<FullName, usize>,
    /// The collapsed components (`specialize`).
    collapse: &'a Collapse,
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
                } => {
                    let lambdas = self.lambdas;
                    let lambda = &lambdas[*lam as usize];
                    let component = self.component_of[&lambda.symbol];
                    let body_params = &self.components[component].body_params;
                    let collapsed = self.collapse.components.contains(&component);
                    let args = lambda
                        .param_deps
                        .iter()
                        .zip(owner_args)
                        .map(|(p, a)| {
                            if collapsed && body_params.contains(p) {
                                Ref::Known(self.collapse.dynamic.unwrap())
                            } else {
                                self.reference(*a)
                            }
                        })
                        .collect();
                    PendingElem::Lam(*lam, args)
                }
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
