// The concrete lambda sets of a specialized program, each kept once however it is reached.

use super::solver::LamId;
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
    index: Map<Vec<Elem>, SetId>,
    /// Whether each set lies on a cycle: it reaches itself through the elements of the sets it holds.
    on_cycle: Vec<bool>,
    /// The sets lying on a cycle, in increasing order: the ids whose `on_cycle` is true, kept apart
    /// so that interning a cycle reads them without a scan of the store.
    cyclic: Vec<SetId>,
}

/// A set compared while a cyclic system is interned: a set of the system or one the store holds.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Candidate {
    /// The set of an equation of the system, by its position.
    Local(usize),
    /// A set the store holds that lies on a cycle.
    Known(SetId),
}

/// What an element of a candidate refers to while a cyclic system is interned.
#[derive(Clone, Copy)]
enum Child {
    /// Another candidate, by its position.
    Candidate(usize),
    /// A set the store holds that lies on no cycle, which is equal to no candidate but itself.
    Fixed(SetId),
}

/// A child as the partition refinement compares it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ChildKey {
    /// A candidate, by the class of the partition it is in.
    Block(usize),
    /// A set the store holds that lies on no cycle.
    Fixed(SetId),
}

impl SetStore {
    /// A store holding no set.
    pub fn new() -> Self {
        SetStore {
            sets: vec![],
            index: Map::default(),
            on_cycle: vec![],
            cyclic: vec![],
        }
    }

    /// The elements of the set `id`, sorted.
    pub fn elems(&self, id: SetId) -> &[Elem] {
        &self.sets[id as usize]
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
        if let Some(id) = self.index.get(&elems) {
            assert!(
                !on_cycle,
                "a set on a cycle is inserted, though it is held already"
            );
            return *id;
        }
        let id = self.sets.len() as SetId;
        self.sets.push(elems.clone());
        self.index.insert(elems, id);
        self.on_cycle.push(on_cycle);
        if on_cycle {
            self.cyclic.push(id);
        }
        id
    }

    /// Resolves the sets of `component`, a strongly connected component of `system` with a cycle,
    /// whose references out of it are resolved already.
    ///
    /// A set of the component is equal to a set of the store only if both lie on a cycle, so the
    /// sets of the component and the cyclic sets of the store are partitioned into the classes of
    /// equal sets, by refining the partition by the elements until it is stable. A class holding a
    /// set of the store resolves its sets of the component to that one; the store holds no two
    /// equal sets, so there is at most one. Each other class becomes a new set.
    fn intern_cycle(
        &mut self,
        component: &[usize],
        system: &[Vec<PendingElem>],
        resolved: &mut [Option<SetId>],
    ) {
        let mut candidates: Vec<Candidate> = vec![];
        let mut position: Map<Candidate, usize> = Map::default();
        for i in component {
            position.insert(Candidate::Local(*i), candidates.len());
            candidates.push(Candidate::Local(*i));
        }
        for id in &self.cyclic {
            position.insert(Candidate::Known(*id), candidates.len());
            candidates.push(Candidate::Known(*id));
        }
        let known_child = |store: &SetStore, id: SetId| -> Child {
            if store.on_cycle[id as usize] {
                Child::Candidate(position[&Candidate::Known(id)])
            } else {
                Child::Fixed(id)
            }
        };
        let elems: Vec<Vec<(Option<LamId>, Vec<Child>)>> = candidates
            .iter()
            .map(|c| match c {
                Candidate::Local(i) => system[*i]
                    .iter()
                    .map(|e| match e {
                        PendingElem::Dynamic => (None, vec![]),
                        PendingElem::Lam(lam, args) => (
                            Some(*lam),
                            args.iter()
                                .map(|a| match a {
                                    Ref::Known(id) => known_child(self, *id),
                                    Ref::Local(j) => match position.get(&Candidate::Local(*j)) {
                                        Some(p) => Child::Candidate(*p),
                                        None => known_child(self, resolved[*j].unwrap()),
                                    },
                                })
                                .collect(),
                        ),
                    })
                    .collect(),
                Candidate::Known(id) => self.sets[*id as usize]
                    .iter()
                    .map(|e| match e {
                        Elem::Dynamic => (None, vec![]),
                        Elem::Lam(lam, args) => (
                            Some(*lam),
                            args.iter().map(|a| known_child(self, *a)).collect(),
                        ),
                    })
                    .collect(),
            })
            .collect();

        // Refine the partition until no class splits.
        let mut block = vec![0usize; candidates.len()];
        let mut block_count = 1;
        loop {
            let mut block_of_signature: Map<(usize, Vec<(Option<LamId>, Vec<ChildKey>)>), usize> =
                Map::default();
            let mut next_block = vec![0usize; candidates.len()];
            for (c, c_elems) in elems.iter().enumerate() {
                let mut signature: Vec<(Option<LamId>, Vec<ChildKey>)> = c_elems
                    .iter()
                    .map(|(lam, children)| {
                        (
                            *lam,
                            children
                                .iter()
                                .map(|ch| match ch {
                                    Child::Candidate(p) => ChildKey::Block(block[*p]),
                                    Child::Fixed(id) => ChildKey::Fixed(*id),
                                })
                                .collect(),
                        )
                    })
                    .collect();
                signature.sort();
                signature.dedup();
                let count = block_of_signature.len();
                next_block[c] = *block_of_signature
                    .entry((block[c], signature))
                    .or_insert(count);
            }
            let stable = block_of_signature.len() == block_count;
            block = next_block;
            block_count = block_of_signature.len();
            if stable {
                break;
            }
        }

        // The set of each class: the store's where the class holds one, a new one otherwise.
        let mut block_set: Vec<Option<SetId>> = vec![None; block_count];
        for (c, candidate) in candidates.iter().enumerate() {
            if let Candidate::Known(id) = candidate {
                assert!(
                    block_set[block[c]].is_none(),
                    "the store holds two equal sets"
                );
                block_set[block[c]] = Some(*id);
            }
        }
        let mut new_blocks: Vec<(usize, usize)> = vec![];
        for (c, candidate) in candidates.iter().enumerate() {
            if let Candidate::Local(_) = candidate {
                if block_set[block[c]].is_none() {
                    let id = (self.sets.len() + new_blocks.len()) as SetId;
                    block_set[block[c]] = Some(id);
                    new_blocks.push((block[c], c));
                }
            }
        }
        let new_sets: Vec<Vec<Elem>> = new_blocks
            .iter()
            .map(|(_, c)| {
                elems[*c]
                    .iter()
                    .map(|(lam, children)| match lam {
                        None => Elem::Dynamic,
                        Some(lam) => Elem::Lam(
                            *lam,
                            children
                                .iter()
                                .map(|ch| match ch {
                                    Child::Candidate(p) => block_set[block[*p]].unwrap(),
                                    Child::Fixed(id) => *id,
                                })
                                .collect(),
                        ),
                    })
                    .collect()
            })
            .collect();
        for elems in new_sets {
            let expected = self.sets.len() as SetId;
            let id = self.insert(elems, true);
            assert_eq!(id, expected);
        }
        for (c, candidate) in candidates.iter().enumerate() {
            if let Candidate::Local(i) = candidate {
                resolved[*i] = block_set[block[c]];
            }
        }
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
}
