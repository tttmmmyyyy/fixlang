// The concrete lambda sets of a specialized program, each kept once however it is reached.

use super::{
    partition::{canonical_classes, Child, ElemKey, NodeElems},
    solver::LamId,
};
use crate::{graph::Graph, misc::Map};

/// A concrete lambda set, as its index into `SetStore`.
pub type SetId = u32;

/// An element of a concrete lambda set.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Elem {
    /// A value only a closure built at run time stands for.
    Dynamic,
    /// The values the lambda builds in the copy of its symbol that the sets give its parameters
    /// (`Lambda::params`).
    Lam(LamId, Vec<SetId>),
}

/// A set a system handed to `SetStore::intern` refers to.
#[derive(Clone, Copy)]
pub enum Ref {
    /// A set the store already holds.
    Known(SetId),
    /// The set of an equation of the same system, by its position.
    Local(usize),
}

/// An element of a set a system handed to `SetStore::intern` defines.
pub enum PendingElem {
    /// `Elem::Dynamic`.
    Dynamic,
    /// `Elem::Lam`, with each set given as a `Ref`.
    Lam(LamId, Vec<Ref>),
}

/// The concrete lambda sets, each held once.
///
/// A set may contain itself through the parameters of a lambda in it, as the set of a function
/// whose lambda captures a function of that set does. Such a set is an infinite tree, held as a
/// cycle. Two sets are the same set when the trees they unfold to are the same, and the store holds
/// no two such: a set is held once however it is written, which is what lets a copy of a symbol be
/// keyed on the sets it is given.
pub struct SetStore {
    /// The elements of each set, sorted.
    sets: Vec<Vec<Elem>>,
    /// The set holding each list of elements.
    id_of: Map<Vec<Elem>, SetId>,
    /// Whether each set lies on a cycle: it reaches itself through the elements of the sets it holds.
    on_cycle: Vec<bool>,
    /// The sets lying on a cycle, by the classes of the sets each reaches (`CycleClasses`) and the
    /// class of the set itself among them.
    id_of_cycle: Map<CycleClasses, Map<usize, SetId>>,
}

/// A set read while a cyclic system is interned: a set of the system or one the store holds.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Node {
    /// The set of an equation of the system, by its position.
    Local(usize),
    /// A set the store holds that lies on a cycle.
    Known(SetId),
}

/// The sets a set lying on a cycle reaches, written so that two sets reaching the same trees are
/// written alike: the classes `canonical_classes` numbers them into, each by its sorted elements,
/// whose label is the lambda, `None` for `Elem::Dynamic`. The set itself is one of the classes, so
/// the classes and the class of the set are the tree it unfolds to.
type CycleClasses = Vec<Vec<ElemKey<Option<LamId>>>>;

impl SetStore {
    /// A store holding no set.
    pub fn new() -> Self {
        SetStore {
            sets: vec![],
            id_of: Map::default(),
            on_cycle: vec![],
            id_of_cycle: Map::default(),
        }
    }

    /// The elements of the set `id`, sorted.
    pub fn elems(&self, id: SetId) -> &[Elem] {
        &self.sets[id as usize]
    }

    /// The elements of the set `id` that name a lambda, each as the lambda and the sets it gives
    /// the lambda's parameters, in the order of `elems`.
    pub fn lam_elems(&self, id: SetId) -> impl Iterator<Item = (LamId, &[SetId])> {
        self.elems(id).iter().filter_map(|elem| match elem {
            Elem::Lam(lam, args) => Some((*lam, args.as_slice())),
            Elem::Dynamic => None,
        })
    }

    /// The number of sets the store holds: the sets are those numbered below it.
    pub fn len(&self) -> usize {
        self.sets.len()
    }

    /// The sets the equations of `system` define, held in the store. An equation lists the
    /// elements of its set, which may refer to sets of the store and to the sets of any equation
    /// of the system, its own included.
    pub fn intern(&mut self, system: Vec<Vec<PendingElem>>) -> Vec<SetId> {
        let n = system.len();
        let mut graph = Graph::new((0..n).collect::<Vec<usize>>());
        for (i, elems) in system.iter().enumerate() {
            for elem in elems {
                if let PendingElem::Lam(_, args) = elem {
                    for arg in args {
                        if let Ref::Local(j) = arg {
                            graph.connect_idx(i, *j);
                        }
                    }
                }
            }
        }
        let mut resolved: Vec<Option<SetId>> = vec![None; n];
        for component in graph.components_dependencies_first() {
            if !graph.is_cycle(&component) {
                let i = component[0];
                let elems = system[i]
                    .iter()
                    .map(|e| match e {
                        PendingElem::Dynamic => Elem::Dynamic,
                        PendingElem::Lam(lam, args) => Elem::Lam(
                            *lam,
                            args.iter()
                                .map(|a| match a {
                                    Ref::Known(id) => *id,
                                    Ref::Local(j) => resolved[*j].unwrap(),
                                })
                                .collect(),
                        ),
                    })
                    .collect();
                resolved[i] = Some(self.insert(elems, false));
            } else {
                self.intern_cycle(&component, &system, &mut resolved);
            }
        }
        resolved.into_iter().map(|id| id.unwrap()).collect()
    }

    /// The set holding `elems`, inserted if the store holds none. `on_cycle` says whether a set
    /// inserted here lies on a cycle.
    fn insert(&mut self, mut elems: Vec<Elem>, on_cycle: bool) -> SetId {
        elems.sort();
        elems.dedup();
        if let Some(id) = self.id_of.get(&elems) {
            assert!(
                !on_cycle,
                "a set on a cycle is inserted, though it is held already"
            );
            return *id;
        }
        let id = self.sets.len() as SetId;
        self.sets.push(elems.clone());
        self.id_of.insert(elems, id);
        self.on_cycle.push(on_cycle);
        id
    }

    /// Resolves the sets of `component`, a strongly connected component of `system` with a cycle,
    /// whose references out of it are resolved already.
    ///
    /// A set of the component is equal to a set of the store only if both lie on a cycle. The sets
    /// of the component reach one another, so they reach the same sets of the store: the nodes are
    /// those sets and the sets of the component. `canonical_classes` partitions the nodes into the
    /// classes of equal sets, and numbers the classes by the trees the sets unfold to, so the
    /// classes and one of them name the tree of a set (`CycleClasses`). A class naming a set of the
    /// store resolves its sets of the component to that one; each other class becomes a new set.
    fn intern_cycle(
        &mut self,
        component: &[usize],
        system: &[Vec<PendingElem>],
        resolved: &mut [Option<SetId>],
    ) {
        let mut nodes: Vec<Node> = component.iter().map(|i| Node::Local(*i)).collect();
        let mut position: Map<Node, usize> =
            nodes.iter().enumerate().map(|(p, n)| (*n, p)).collect();
        // The elements of each node, in the order of `nodes`, which grows as the walk meets the sets
        // of the store the nodes reach.
        let mut elems: Vec<NodeElems<Option<LamId>>> = vec![];
        while elems.len() < nodes.len() {
            let node_elems = match nodes[elems.len()] {
                Node::Local(i) => system[i]
                    .iter()
                    .map(|e| match e {
                        PendingElem::Dynamic => (None, vec![]),
                        PendingElem::Lam(lam, args) => {
                            let children = args
                                .iter()
                                .map(|a| match a {
                                    Ref::Local(j) => match position.get(&Node::Local(*j)) {
                                        Some(q) => Child::Node(*q),
                                        None => self.child(
                                            resolved[*j].unwrap(),
                                            &mut nodes,
                                            &mut position,
                                        ),
                                    },
                                    Ref::Known(id) => self.child(*id, &mut nodes, &mut position),
                                })
                                .collect();
                            (Some(*lam), children)
                        }
                    })
                    .collect(),
                Node::Known(id) => self.sets[id as usize]
                    .iter()
                    .map(|e| match e {
                        Elem::Dynamic => (None, vec![]),
                        Elem::Lam(lam, args) => {
                            let children = args
                                .iter()
                                .map(|a| self.child(*a, &mut nodes, &mut position))
                                .collect();
                            (Some(*lam), children)
                        }
                    })
                    .collect(),
            };
            elems.push(node_elems);
        }

        let (class, class_elems) = canonical_classes(vec![0; elems.len()], &elems);
        let held = self.id_of_cycle.get(&class_elems);
        let mut class_set: Vec<Option<SetId>> = vec![None; class_elems.len()];
        for (p, node) in nodes.iter().enumerate() {
            if let Node::Known(id) = node {
                assert!(
                    class_set[class[p]].is_none(),
                    "the store holds two equal sets"
                );
                class_set[class[p]] = Some(*id);
            }
        }
        let mut new_classes: Vec<(usize, usize)> = vec![];
        for p in 0..component.len() {
            let c = class[p];
            if class_set[c].is_some() {
                continue;
            }
            class_set[c] = Some(match held.and_then(|held| held.get(&c)) {
                Some(id) => *id,
                None => {
                    new_classes.push((c, p));
                    (self.sets.len() + new_classes.len() - 1) as SetId
                }
            });
        }
        let mut made: Vec<(usize, SetId)> = vec![];
        for (c, p) in new_classes {
            let new_elems = elems[p]
                .iter()
                .map(|(lam, children)| match lam {
                    None => Elem::Dynamic,
                    Some(lam) => Elem::Lam(
                        *lam,
                        children
                            .iter()
                            .map(|ch| match ch {
                                Child::Node(q) => class_set[class[*q]].unwrap(),
                                Child::Fixed(id) => *id,
                            })
                            .collect(),
                    ),
                })
                .collect();
            let id = self.insert(new_elems, true);
            assert_eq!(Some(id), class_set[c]);
            made.push((c, id));
        }
        if !made.is_empty() {
            self.id_of_cycle
                .entry(class_elems)
                .or_default()
                .extend(made);
        }
        for (p, i) in component.iter().enumerate() {
            resolved[*i] = class_set[class[p]];
        }
    }

    /// The child an element referring to the set `id` of the store has: the node of `id` where it
    /// lies on a cycle, added to `nodes` the first time it is met, and `id` itself otherwise.
    fn child(&self, id: SetId, nodes: &mut Vec<Node>, position: &mut Map<Node, usize>) -> Child {
        if !self.on_cycle[id as usize] {
            return Child::Fixed(id);
        }
        let node = Node::Known(id);
        let p = *position.entry(node).or_insert_with(|| {
            nodes.push(node);
            nodes.len() - 1
        });
        Child::Node(p)
    }
}

#[cfg(test)]
mod tests {
    use super::{PendingElem, Ref, SetStore};

    /// A set containing itself, `s = {lam0, lam1[s]}`, is the same set however far it is unfolded:
    /// within one system, two equations unfolding to the same tree get one set, and a later system
    /// writing it unfolded once gets the set the store holds. A set whose tree differs, `{lam1[t]}`
    /// with `t` itself, is another set.
    #[test]
    fn test_a_set_containing_itself_is_held_once_however_far_it_is_unfolded() {
        let mut store = SetStore::new();
        let folded = store.intern(vec![vec![
            PendingElem::Lam(0, vec![]),
            PendingElem::Lam(1, vec![Ref::Local(0)]),
        ]]);
        assert_eq!(folded.len(), 1);
        let s = folded[0];

        let unfolded = store.intern(vec![
            vec![
                PendingElem::Lam(0, vec![]),
                PendingElem::Lam(1, vec![Ref::Local(1)]),
            ],
            vec![
                PendingElem::Lam(0, vec![]),
                PendingElem::Lam(1, vec![Ref::Local(0)]),
            ],
        ]);
        assert_eq!(unfolded, vec![s, s]);

        let through_known = store.intern(vec![vec![
            PendingElem::Lam(0, vec![]),
            PendingElem::Lam(1, vec![Ref::Known(s)]),
        ]]);
        assert_eq!(through_known, vec![s]);

        let other = store.intern(vec![vec![PendingElem::Lam(1, vec![Ref::Local(0)])]]);
        assert_ne!(other[0], s);
    }

    /// A cycle of the system equal to a set of the store that it reaches gets that set:
    /// `x = {lam1[x], lam2[y]}` with `y = {lam1[y], lam2[y]}` held already unfolds to `y`'s tree.
    #[test]
    fn test_a_cycle_equal_to_a_set_it_reaches_gets_that_set() {
        let mut store = SetStore::new();
        let y = store.intern(vec![vec![
            PendingElem::Lam(1, vec![Ref::Local(0)]),
            PendingElem::Lam(2, vec![Ref::Local(0)]),
        ]])[0];
        let x = store.intern(vec![vec![
            PendingElem::Lam(1, vec![Ref::Local(0)]),
            PendingElem::Lam(2, vec![Ref::Known(y)]),
        ]]);
        assert_eq!(x, vec![y]);
        assert_eq!(store.len(), 1);
    }

    /// A cycle of the system gets the sets the store holds for it whatever order the system lists
    /// its equations in: `a = {lam1[b]}` with `b = {lam2[a]}`, interned again with `b` written
    /// first, gets the two sets held already.
    #[test]
    fn test_a_cycle_written_in_another_order_gets_the_sets_held() {
        let mut store = SetStore::new();
        let ab = store.intern(vec![
            vec![PendingElem::Lam(1, vec![Ref::Local(1)])],
            vec![PendingElem::Lam(2, vec![Ref::Local(0)])],
        ]);
        let ba = store.intern(vec![
            vec![PendingElem::Lam(2, vec![Ref::Local(1)])],
            vec![PendingElem::Lam(1, vec![Ref::Local(0)])],
        ]);
        assert_eq!(ba, vec![ab[1], ab[0]]);
        assert_eq!(store.len(), 2);
    }
}
