// How the values of each concrete lambda set are represented once the program is rewritten.

use super::{
    concrete::{Elem, SetId, SetStore},
    infer::Analysis,
    partition::{canonical_classes, elem_key, Child, ElemKey, NodeElems},
    solver::{LamId, SetVar},
    specialize::Specialization,
};
use crate::{
    constants::{LS_COPY_SUFFIX, MAX_IN_PLACE_CAPTURE_STRUCTS, MAX_UNION_VARIANTS},
    graph::Graph,
    hash::md5_hex,
    misc::{Map, Set},
};

/// How the values of a concrete set are represented.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Repr {
    /// As closures, the form a function value has without this pass.
    Closure,
    /// As the capture struct of the one class the set's elements make, whose values are called by
    /// calling the class's lambda directly. Holds that class, as its index into
    /// `Representation::classes`.
    Single(u32),
    /// As an unboxed union with a variant for each class the set's elements make, holding that
    /// class's capture struct, whose values are called by matching the variant and calling its
    /// class's lambda directly. Holds the union, as its index into `Representation::unions`.
    Union(u32),
}

/// The unboxed union standing for the values of the sets whose elements make the same two or more
/// classes.
pub struct UnionRepr {
    /// The class of each variant, as its index into `Representation::classes`, in the order of the
    /// variants: the order `canonical_classes` sorts the elements of the sets in, which depends only
    /// on the trees the sets unfold to.
    pub members: Vec<u32>,
    /// The hash of the text the union is written as. It is the same in every compilation of the
    /// program, and differs between two unions.
    pub hash: String,
}

/// The elements naming one lambda that the rewrite makes one global function of: they represent
/// alike every set at the lambda's interface (`Builder::interface`), so the global function and the
/// capture struct it takes have one type for all of them.
pub struct LamClass {
    /// The lambda.
    pub lam: LamId,
    /// How each set at the lambda's interface is represented, in the order `Builder::interface` lists
    /// them.
    pub interface_reprs: Vec<Repr>,
    /// The copy of the lambda's component that builds the values of one of the elements, as an
    /// index into `Specialization::instances`. The lambda's body is rewritten with the sets of this
    /// copy. The body shares only the sets at the interface with the copies that build and call the
    /// class's values, and every element of the class represents those alike.
    pub instance: usize,
    /// The hash of the text the class is written as, which is the same in every compilation of the
    /// program, and different for two classes.
    pub hash: String,
}

/// How the values of every set of a specialized program are represented.
pub struct Representation {
    /// How each set is represented, by its id.
    of_set: Vec<Repr>,
    /// The classes of the elements the rewrite makes global functions of: the elements of each set
    /// represented by a capture struct or a union, and each element of a set of closures naming a
    /// lambda that `Lambda::is_lifted_from_closures` holds of. The classes of the sets represented
    /// by a capture struct come first.
    pub classes: Vec<LamClass>,
    /// The unions standing for the sets represented by an unboxed union.
    pub unions: Vec<UnionRepr>,
    /// The position in `classes` of the class holding each element, by the lambda the element names
    /// and the sets it gives the lambda's parameters (`Lambda::params`).
    class_of_elem: Map<(LamId, Vec<SetId>), u32>,
    /// The number of classes of each lambda that has any: how many of `classes` hold the lambda.
    class_counts: Map<LamId, usize>,
}

impl Representation {
    /// How the values of `set` are represented.
    pub fn of(&self, set: SetId) -> Repr {
        self.of_set[set as usize]
    }

    /// The class of the element naming `lam` and giving its parameters the sets `args`, in the
    /// order of `Lambda::params`.
    pub fn class_of(&self, lam: LamId, args: &[SetId]) -> u32 {
        *self
            .class_of_elem
            .get(&(lam, args.to_vec()))
            .unwrap_or_else(|| {
                panic!(
                    "no class holds the element of the lambda {} given the sets {:?}",
                    lam, args
                )
            })
    }

    /// The number of classes whose lambda is `lam`, a lambda that has any.
    pub fn class_count(&self, lam: LamId) -> usize {
        *self
            .class_counts
            .get(&lam)
            .unwrap_or_else(|| panic!("no class holds an element of the lambda {}", lam))
    }

    /// The variant of the union `u` whose payload is the capture struct of the class `c`, a member
    /// of the union.
    pub fn variant_of(&self, u: u32, c: u32) -> usize {
        self.unions[u as usize]
            .members
            .iter()
            .position(|m| *m == c)
            .unwrap_or_else(|| panic!("the class {} is no member of the union {}", c, u))
    }

    /// The suffix of a copy made for the representations `reprs`: `LS_COPY_SUFFIX` followed by
    /// the hash of their texts.
    pub fn copy_suffix(&self, reprs: &[Repr]) -> String {
        let text: Vec<&str> = reprs.iter().map(|r| self.hash_text(*r)).collect();
        format!("{}{}", LS_COPY_SUFFIX, md5_hex(&text.join(",")))
    }

    /// The text a `Repr` is written as in the hashes that name copies.
    fn hash_text(&self, repr: Repr) -> &str {
        match repr {
            Repr::Closure => "c",
            Repr::Single(c) => &self.classes[c as usize].hash,
            Repr::Union(u) => &self.unions[u as usize].hash,
        }
    }
}

/// The copy of a lambda's component that builds the values of an element naming the lambda, and
/// the sets at the element's interface.
struct Builder {
    /// The copy, as an index into `Specialization::instances`: the first copy of the lambda's
    /// component, in the order of the copies, that gives the lambda's parameters the element's
    /// sets.
    instance: usize,
    /// The sets at the element's interface: those the copy gives the variables of
    /// `Lambda::interface`. For the lambda that is the body of its symbol, they are the sets the
    /// element gives the lambda's parameters, as those are the parameters its body depends on.
    interface: Vec<SetId>,
    /// The sets the copy gives the variables of `Lambda::held_in_place`: those whose values the
    /// element's capture struct holds in place.
    held_in_place: Vec<SetId>,
}

/// The `Builder` of each element naming a lambda, by the lambda and the sets the element gives the
/// lambda's parameters (`Lambda::params`).
struct Builders(Map<(LamId, Vec<SetId>), Builder>);

impl Builders {
    /// The builders of the elements the copies of `spec` build.
    fn new(analysis: &Analysis, spec: &Specialization) -> Self {
        let mut lambdas_of_component: Vec<Vec<LamId>> = vec![vec![]; analysis.components.len()];
        for (lam, lambda) in analysis.lambdas.iter().enumerate() {
            lambdas_of_component[analysis.component_of[&lambda.symbol]].push(lam as LamId);
        }
        let mut builders: Map<(LamId, Vec<SetId>), Builder> = Map::default();
        for (k, instance) in spec.instances.iter().enumerate() {
            let component = &analysis.components[instance.component];
            for lam in &lambdas_of_component[instance.component] {
                let lambda = &analysis.lambdas[*lam as usize];
                let args: Vec<SetId> = lambda.params.iter().map(|p| instance.args[*p]).collect();
                let set_of = |v: &SetVar| instance.var_sets[component.var_index[v]];
                let interface: Vec<SetId> = if lambda.is_body() {
                    args.clone()
                } else {
                    lambda.interface.iter().map(set_of).collect()
                };
                let held_in_place: Vec<SetId> = lambda.held_in_place.iter().map(set_of).collect();
                // A held set decides the layout of the element's capture struct, so it is one the
                // class reads at its interface, or one holding `Dynamic`, which the layout never
                // holds in place.
                for (v, held) in lambda.held_in_place.iter().zip(&held_in_place) {
                    assert!(
                        lambda.interface.contains(v)
                            || spec.store.elems(*held).contains(&Elem::Dynamic),
                        "{} holds in place a set outside its interface",
                        lambda.to_string()
                    );
                }
                match builders.get(&(*lam, args.clone())) {
                    Some(known) => assert_eq!(
                        known.interface,
                        interface,
                        "two copies giving the parameters of {} the same sets give its interface \
                         different ones",
                        lambda.to_string()
                    ),
                    None => {
                        builders.insert(
                            (*lam, args),
                            Builder {
                                instance: k,
                                interface,
                                held_in_place,
                            },
                        );
                    }
                }
            }
        }
        Builders(builders)
    }

    /// The builder of the element naming `lam` and giving its parameters the sets `args`.
    fn of(&self, lam: LamId, args: &[SetId]) -> &Builder {
        self.0.get(&(lam, args.to_vec())).unwrap_or_else(|| {
            panic!(
                "no copy builds the values of the lambda {} given the sets {:?}",
                lam, args
            )
        })
    }
}

/// Whether a capture struct, or a union of capture structs, can stand for the values of each set of
/// `spec`: the set is not empty and holds lambdas alone, and its layout does not hold itself, as
/// `represent` describes.
fn unboxable_sets(analysis: &Analysis, spec: &Specialization, builders: &Builders) -> Vec<bool> {
    let store = &spec.store;
    let set_count = store.len();
    let mut unboxable: Vec<bool> = (0..set_count)
        .map(|id| {
            let elems = store.elems(id as SetId);
            !elems.is_empty() && elems.iter().all(|elem| matches!(elem, Elem::Lam(..)))
        })
        .collect();
    let mut holds = Graph::new((0..set_count).collect::<Vec<usize>>());
    for id in 0..set_count {
        if !unboxable[id] {
            continue;
        }
        for (lam, args) in store.lam_elems(id as SetId) {
            let lambda = &analysis.lambdas[lam as usize];
            assert!(
                lambda.is_body() || lambda.is_lifted_from_closures(),
                "the set {} of lambdas alone holds {}, which needs the closure object holding it",
                id,
                lambda.to_string()
            );
            for held in &builders.of(lam, args).held_in_place {
                if unboxable[*held as usize] {
                    holds.connect_idx(id, *held as usize);
                }
            }
        }
    }
    for component in holds.components_dependencies_first() {
        if holds.is_cycle(&component) {
            for id in component {
                unboxable[id] = false;
            }
        }
    }
    unboxable
}

/// The elements of the set `id` one per class of the partition `block`, each with its key under the
/// partition, sorted by the key. `elems` holds the elements each set is partitioned by
/// (`partition_sets`).
fn class_elems<'s>(
    store: &'s SetStore,
    elems: &[NodeElems<LamId>],
    block: &[usize],
    id: usize,
) -> Vec<(ElemKey<LamId>, LamId, &'s [SetId])> {
    let mut classes: Vec<(ElemKey<LamId>, LamId, &[SetId])> = store
        .lam_elems(id as SetId)
        .zip(&elems[id])
        .map(|((lam, args), (label, children))| (elem_key(label, children, block), lam, args))
        .collect();
    classes.sort_by(|a, b| a.0.cmp(&b.0));
    classes.dedup_by(|a, b| a.0 == b.0);
    classes
}

/// Whether each set represented by a capture struct or a union under the partition `block` is too
/// large for it: its elements make more classes than a union has variants (`MAX_UNION_VARIANTS`),
/// or its values would lay out more capture structs in place than `MAX_IN_PLACE_CAPTURE_STRUCTS`.
///
/// A value lays out in place one capture struct per class of its set's elements, and with each the
/// capture structs of the values the class's lambda holds in place. The sets are judged from the
/// inside out: a held set too large counts as a set of closures, which lays out none, so of a chain
/// of growing sets only the first past the bound is too large.
fn too_large_sets(
    store: &SetStore,
    builders: &Builders,
    unboxable: &[bool],
    elems: &[NodeElems<LamId>],
    block: &[usize],
) -> Vec<bool> {
    /// The partition being judged, and the judgment of each block met so far (`in_place_of_block`):
    /// how many capture structs a value of its sets lays out in place, or `None` where its sets are
    /// too large.
    struct Judge<'j> {
        store: &'j SetStore,
        builders: &'j Builders,
        unboxable: &'j [bool],
        elems: &'j [NodeElems<LamId>],
        block: &'j [usize],
        in_place_of_block: Map<usize, Option<u64>>,
    }
    impl Judge<'_> {
        /// How many capture structs a value of the set `id` lays out in place, counting none for a
        /// set of closures and for one too large.
        fn in_place_count(&mut self, id: usize) -> u64 {
            if !self.unboxable[id] {
                return 0;
            }
            if let Some(judged) = self.in_place_of_block.get(&self.block[id]) {
                return judged.unwrap_or(0);
            }
            let classes = class_elems(self.store, self.elems, self.block, id);
            let judged = if classes.len() > MAX_UNION_VARIANTS {
                None
            } else {
                let mut n: u64 = 0;
                for (_, lam, args) in &classes {
                    n += 1;
                    for held in &self.builders.of(*lam, args).held_in_place {
                        n += self.in_place_count(*held as usize);
                    }
                }
                (n <= MAX_IN_PLACE_CAPTURE_STRUCTS).then_some(n)
            };
            self.in_place_of_block.insert(self.block[id], judged);
            judged.unwrap_or(0)
        }

        /// Whether the set `id` is represented by a capture struct or a union and too large for it.
        fn is_too_large(&mut self, id: usize) -> bool {
            self.in_place_count(id);
            self.unboxable[id] && self.in_place_of_block[&self.block[id]].is_none()
        }
    }
    let mut judge = Judge {
        store,
        builders,
        unboxable,
        elems,
        block,
        in_place_of_block: Map::default(),
    };
    (0..store.len()).map(|id| judge.is_too_large(id)).collect()
}

/// The partition of the sets of `store` by the code their values make (`canonical_classes`), where
/// `unboxable` holds the sets a capture struct or a union can stand for: the elements each set is
/// partitioned by, in the order `SetStore::lam_elems` lists them, the class of each set, and the
/// sorted elements of each class. A set no capture struct or union stands for is given no element.
fn partition_sets(
    store: &SetStore,
    builders: &Builders,
    unboxable: &[bool],
) -> (Vec<NodeElems<LamId>>, Vec<usize>, Vec<Vec<ElemKey<LamId>>>) {
    let elems: Vec<NodeElems<LamId>> = (0..store.len())
        .map(|id| {
            if !unboxable[id] {
                return vec![];
            }
            store
                .lam_elems(id as SetId)
                .map(|(lam, args)| {
                    let children = builders
                        .of(lam, args)
                        .interface
                        .iter()
                        .map(|s| Child::Node(*s as usize))
                        .collect();
                    (lam, children)
                })
                .collect()
        })
        .collect();
    let (block, block_elems) = canonical_classes(vec![0; store.len()], &elems);
    (elems, block, block_elems)
}

/// How the values of each set of `spec` are represented.
///
/// The representation is decided in three steps.
///
/// 1. A set whose values neither a capture struct nor a union of capture structs can stand for is
///    represented by closures: an empty set, a set holding `Elem::Dynamic`, and a set whose layout
///    holds itself, where the capture structs of its elements hold in place, through the unboxed
///    fields of the values they capture, a set that holds in place this one again. Such a layout
///    would have no size, as an unboxed type reaching itself has none.
///    So is a set step 2 finds too large for a capture struct or a union (`too_large_sets`): its
///    elements make more classes than a union has variants, or its values would lay out more
///    capture structs in place than `MAX_IN_PLACE_CAPTURE_STRUCTS`. Step 2 is then taken again with
///    it among these.
/// 2. The other sets and their elements are partitioned by the code they make: two elements are
///    equal when they name the same lambda and the sets at its interface (`Builder::interface`) are
///    equal, and two sets are equal when their elements are, counted up to that equality. The sets
///    of step 1 are all equal to one another and to no other set, as a closure is the same whatever
///    it holds.
/// 3. A set whose elements make one class of the partition is represented by the capture struct of
///    that class, and a set whose elements make two or more by an unboxed union holding the capture
///    struct of each of those classes in a variant of its own. Equal sets share the capture struct
///    or the union.
///
/// The elements of the sets represented by closures that name a lambda fall into classes too,
/// where `Lambda::is_lifted_from_closures` holds of the lambda: the rewrite lifts such a lambda to a
/// global function, and the closure is that function applied to the capture struct. Elements
/// representing the sets at the lambda's interface alike are in one class, so they share the
/// global function and the capture struct.
pub fn represent(analysis: &Analysis, spec: &Specialization) -> Representation {
    let store = &spec.store;
    let set_count = store.len();
    let builders = Builders::new(analysis, spec);

    // The sets a capture struct or a union of capture structs can stand for.
    let mut unboxable = unboxable_sets(analysis, spec, &builders);

    // The partition of the sets by the code their values make. The sets of closures hold no
    // element the partition reads, so they make one class of their own. A set too large for a
    // capture struct or a union is represented by closures as well, and the partition is made again
    // with it among them. Once is enough: the partition made first, with the blocks of those sets
    // merged into the block of the sets of closures, is still stable, so the one made again is as
    // coarse or coarser, and no set it holds makes more classes, or lays out more capture structs,
    // than `too_large_sets` counted for it.
    let (mut elems, mut block, mut block_elems) = partition_sets(store, &builders, &unboxable);
    let too_large = too_large_sets(store, &builders, &unboxable, &elems, &block);
    if too_large.contains(&true) {
        for (id, large) in too_large.into_iter().enumerate() {
            if large {
                unboxable[id] = false;
            }
        }
        (elems, block, block_elems) = partition_sets(store, &builders, &unboxable);
        assert!(
            !too_large_sets(store, &builders, &unboxable, &elems, &block).contains(&true),
            "a set is too large for a capture struct or a union once the partition is made again"
        );
    }

    // A set whose elements make one class is represented by that class's capture struct, and one
    // whose elements make more by a union of their capture structs. The sets of one block share the
    // capture struct or the union, which are numbered in the order of the sets: the first set of a
    // block stands for it.
    let mut of_set: Vec<Repr> = vec![Repr::Closure; set_count];
    let mut repr_of_block: Map<usize, Repr> = Map::default();
    let mut first_struct_sets: Vec<usize> = vec![];
    let mut first_union_sets: Vec<usize> = vec![];
    for id in 0..set_count {
        if !unboxable[id] {
            continue;
        }
        let repr = *repr_of_block.entry(block[id]).or_insert_with(|| {
            if block_elems[block[id]].len() == 1 {
                first_struct_sets.push(id);
                Repr::Single((first_struct_sets.len() - 1) as u32)
            } else {
                first_union_sets.push(id);
                Repr::Union((first_union_sets.len() - 1) as u32)
            }
        });
        of_set[id] = repr;
    }

    // The classes of the sets represented by a capture struct come first, numbered as `of_set`
    // names them, then the classes the members of the unions add, then those the elements of the
    // sets of closures add.
    let mut classes: Vec<LamClass> = vec![];
    let mut class_of_reprs: Map<(LamId, Vec<Repr>), u32> = Map::default();
    let mut class_of_elem: Map<(LamId, Vec<SetId>), u32> = Map::default();
    let mut add_class = |lam: LamId, args: &[SetId]| -> u32 {
        let builder = builders.of(lam, args);
        let reprs: Vec<Repr> = builder
            .interface
            .iter()
            .map(|s| of_set[*s as usize])
            .collect();
        let class = *class_of_reprs
            .entry((lam, reprs.clone()))
            .or_insert_with(|| {
                classes.push(LamClass {
                    lam,
                    interface_reprs: reprs,
                    instance: builder.instance,
                    hash: String::new(),
                });
                (classes.len() - 1) as u32
            });
        class_of_elem.insert((lam, args.to_vec()), class);
        class
    };
    for (class, id) in first_struct_sets.iter().enumerate() {
        for (lam, args) in store.lam_elems(*id as SetId) {
            assert_eq!(
                add_class(lam, args),
                class as u32,
                "an element of a set represented by a capture struct represents its interface \
                 otherwise than the set's class"
            );
        }
    }
    // The members of a union are the classes of its first set's elements, in the order its block's
    // elements are sorted in. Two elements are in one class exactly when they are equal in the
    // partition, as the representation of a set is decided by its block.
    let mut unions: Vec<UnionRepr> = vec![];
    for id in &first_union_sets {
        let members: Vec<(ElemKey<LamId>, u32)> = class_elems(store, &elems, &block, *id)
            .into_iter()
            .map(|(key, lam, args)| (key, add_class(lam, args)))
            .collect();
        assert!(
            members
                .iter()
                .map(|(key, _)| key)
                .eq(block_elems[block[*id]].iter()),
            "the elements of a set represented by a union differ from those of its block"
        );
        let member_classes: Vec<u32> = members.into_iter().map(|(_, class)| class).collect();
        let distinct: Set<u32> = member_classes.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            member_classes.len(),
            "two variants of a union hold the capture struct of one class"
        );
        unions.push(UnionRepr {
            members: member_classes,
            hash: String::new(),
        });
    }
    for id in 0..set_count {
        match of_set[id] {
            Repr::Single(class) => {
                for (lam, args) in store.lam_elems(id as SetId) {
                    assert_eq!(
                        add_class(lam, args),
                        class,
                        "two elements of a set represented by a capture struct are in different \
                         classes"
                    );
                }
            }
            Repr::Union(u) => {
                for (lam, args) in store.lam_elems(id as SetId) {
                    let class = add_class(lam, args);
                    assert!(
                        unions[u as usize].members.contains(&class),
                        "an element of a set represented by a union is in no class of the union"
                    );
                }
            }
            Repr::Closure => {
                for (lam, args) in store.lam_elems(id as SetId) {
                    if analysis.lambdas[lam as usize].is_lifted_from_closures() {
                        add_class(lam, args);
                    }
                }
            }
        }
    }
    name_classes_and_unions(analysis, &mut classes, &mut unions);

    let mut class_counts: Map<LamId, usize> = Map::default();
    for class in &classes {
        *class_counts.entry(class.lam).or_insert(0) += 1;
    }
    Representation {
        of_set,
        classes,
        unions,
        class_of_elem,
        class_counts,
    }
}

/// Gives each of `classes` and `unions` its hash: the MD5 digest of the text it is written as. The
/// text of a class names its lambda (`Lambda::to_string`), then writes the representation of each
/// set at the lambda's interface in parentheses: `c` for a set of closures, and for a set
/// represented by a capture struct or a union the text of that class or union. The text of a union
/// writes the texts of its members in braces, in the order of its variants. A class or a union met
/// again on the way, which one reaching itself meets, is written `^n`, `n` counting the classes and
/// unions written so far in that text; one outside the strongly connected component being written
/// is written `#` and its hash. The text reads only the lambdas, the representations and the order
/// of the variants, so it is the same however the sets were numbered.
///
/// # Examples
/// Two classes reaching each other, `A` of the lambda `f@lam1` representing its interface as
/// `[Closure, Single(B)]` and `B` of `g@lam0` representing it as `[Single(A)]`, are written
/// `f@lam1(c,g@lam0(^0))` and `g@lam0(f@lam1(c,^0))`. A union of the classes of `f@lam0()` and
/// `g@lam0()` is written `{f@lam0(),g@lam0()}`.
fn name_classes_and_unions(
    analysis: &Analysis,
    classes: &mut [LamClass],
    unions: &mut [UnionRepr],
) {
    let nodes = NameGraph {
        analysis,
        classes: &*classes,
        unions: &*unions,
    };
    let node_count = classes.len() + unions.len();
    let mut graph = Graph::new((0..node_count).collect::<Vec<usize>>());
    for node in 0..node_count {
        for child in nodes.children(node) {
            graph.connect_idx(node, child);
        }
    }
    let mut hashes: Vec<String> = vec![String::new(); node_count];
    for component in graph.components_dependencies_first() {
        let members: Set<usize> = component.iter().copied().collect();
        for node in &component {
            let mut text = String::new();
            let mut written: Map<usize, usize> = Map::default();
            nodes.write(&members, &hashes, *node, &mut written, &mut text);
            hashes[*node] = md5_hex(&text);
        }
    }
    let (class_hashes, union_hashes) = hashes.split_at(classes.len());
    for (class, hash) in classes.iter_mut().zip(class_hashes) {
        class.hash = hash.clone();
    }
    for (union, hash) in unions.iter_mut().zip(union_hashes) {
        union.hash = hash.clone();
    }
    let distinct: Set<&str> = class_hashes.iter().map(String::as_str).collect();
    assert_eq!(
        distinct.len(),
        classes.len(),
        "two classes are written as the same text"
    );
    let distinct: Set<&str> = union_hashes.iter().map(String::as_str).collect();
    assert_eq!(
        distinct.len(),
        unions.len(),
        "two unions are written as the same text"
    );
}

/// The classes and the unions `name_classes_and_unions` writes, as the nodes of one graph: the classes first,
/// by their positions in `classes`, then the unions, by their positions in `unions` after them.
struct NameGraph<'n> {
    /// The lambda sets inferred for the program, whose lambdas the texts name.
    analysis: &'n Analysis,
    /// The classes, as the nodes from 0 up to `classes.len()`.
    classes: &'n [LamClass],
    /// The unions, as the nodes from `classes.len()` on.
    unions: &'n [UnionRepr],
}

impl NameGraph<'_> {
    /// The node standing for the class or the union a representation names, if it names one.
    fn node_of(&self, repr: &Repr) -> Option<usize> {
        match repr {
            Repr::Closure => None,
            Repr::Single(c) => Some(*c as usize),
            Repr::Union(u) => Some(self.classes.len() + *u as usize),
        }
    }

    /// The nodes the text of `node` writes: for a class, the classes and the unions at its
    /// interface; for a union, its members.
    fn children(&self, node: usize) -> Vec<usize> {
        match node.checked_sub(self.classes.len()) {
            None => self.classes[node]
                .interface_reprs
                .iter()
                .filter_map(|r| self.node_of(r))
                .collect(),
            Some(u) => self.unions[u].members.iter().map(|c| *c as usize).collect(),
        }
    }

    /// Appends to `text` the text of `node`, a member of the strongly connected component
    /// `component`, `written` holding the members written so far in this text, each with the number
    /// of members written before it, and `hashes` the hash of each node outside the component.
    fn write(
        &self,
        component: &Set<usize>,
        hashes: &[String],
        node: usize,
        written: &mut Map<usize, usize>,
        text: &mut String,
    ) {
        if let Some(n) = written.get(&node) {
            text.push_str(&format!("^{}", n));
            return;
        }
        written.insert(node, written.len());
        let write_child = |child: usize, written: &mut Map<usize, usize>, text: &mut String| {
            if component.contains(&child) {
                self.write(component, hashes, child, written, text);
            } else {
                text.push('#');
                text.push_str(&hashes[child]);
            }
        };
        match node.checked_sub(self.classes.len()) {
            None => {
                let class = &self.classes[node];
                text.push_str(&self.analysis.lambdas[class.lam as usize].to_string());
                text.push('(');
                for (i, arg) in class.interface_reprs.iter().enumerate() {
                    if i > 0 {
                        text.push(',');
                    }
                    match self.node_of(arg) {
                        None => text.push('c'),
                        Some(child) => write_child(child, written, text),
                    }
                }
                text.push(')');
            }
            Some(u) => {
                text.push('{');
                for (i, c) in self.unions[u].members.iter().enumerate() {
                    if i > 0 {
                        text.push(',');
                    }
                    write_child(*c as usize, written, text);
                }
                text.push('}');
            }
        }
    }
}
