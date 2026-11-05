// How the values of each concrete lambda set are represented once the program is rewritten.

use super::{
    concrete::{Elem, SetId},
    infer::Analysis,
    partition::{canonical_classes, Child, NodeElems},
    solver::{LamId, SetVar},
    specialize::Specialization,
};
use crate::{
    constants::LS_COPY_SUFFIX,
    graph::Graph,
    hash::md5_hex,
    misc::{Map, Set},
};

/// How the values of a concrete set are represented.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Repr {
    /// As closures, the form a function value has without this pass.
    Closure,
    /// As the capture struct of the one lambda of the set, whose values are called by calling that
    /// lambda directly. Holds the class of the set's element, as its index into
    /// `Representation::classes`.
    Single(u32),
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
    /// copy; another element of the class represents the sets at the interface alike, which is
    /// all the body shares with the copies building and calling the class's values.
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
    /// represented by a capture struct, and each element of a set of closures naming a lambda that
    /// `Lambda::is_lifted_from_closures` holds of. The classes of the sets represented by a capture
    /// struct come first.
    pub classes: Vec<LamClass>,
    /// The class of each element of `classes`, by the lambda it names and the sets it gives the
    /// lambda's parameters (`Lambda::params`).
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
        }
    }
}

/// The variables of the sets at the interface of each lambda that is not the body of its symbol,
/// by the lambda: those of its parameter and result types, then those of the types of the values it
/// captures, in the order `Lambda::captures` lists them. These are the sets the copy building a
/// value of the lambda and the global function the lambda is lifted to both read. The lambda's
/// own set is left out.
fn interface_vars(analysis: &Analysis) -> Vec<Vec<SetVar>> {
    analysis
        .lambdas
        .iter()
        .map(|lambda| {
            if lambda.is_body() {
                return vec![];
            }
            let component = &analysis.components[analysis.component_of[&lambda.symbol]];
            let pos = component
                .symbols
                .iter()
                .position(|s| *s == lambda.symbol)
                .unwrap();
            let own_type = &component.node_types[pos][lambda.nodes.start].ty;
            own_type.slots[1..]
                .iter()
                .chain(lambda.captures.iter().flat_map(|(_, ty)| ty.slots.iter()))
                .copied()
                .collect()
        })
        .collect()
}

/// The copy of a lambda's component that builds the values of an element naming the lambda, and
/// the sets at the element's interface.
struct Builder {
    /// The copy, as an index into `Specialization::instances`: the first copy of the lambda's
    /// component, in the order of the copies, that gives the lambda's parameters the element's
    /// sets.
    instance: usize,
    /// The sets at the element's interface: those the copy gives the variables `interface_vars`
    /// lists. The lambda that is the body of its symbol depends on the parameters its body does,
    /// so the sets of those are its interface.
    interface: Vec<SetId>,
}

/// The `Builder` of each element naming a lambda, by the lambda and the sets the element gives the
/// lambda's parameters (`Lambda::params`).
struct Builders(Map<(LamId, Vec<SetId>), Builder>);

impl Builders {
    /// The builders of the elements the copies of `spec` build.
    fn new(analysis: &Analysis, spec: &Specialization) -> Self {
        let vars = interface_vars(analysis);
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
                let interface: Vec<SetId> = if lambda.is_body() {
                    args.clone()
                } else {
                    vars[*lam as usize]
                        .iter()
                        .map(|v| instance.var_sets[component.var_index[v]])
                        .collect()
                };
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

/// Whether a capture struct can stand for the values of each set of `spec`: the set holds lambdas
/// alone, and its layout does not hold itself (step 1 of `represent`).
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
        for (lam, args) in store.lambdas(id as SetId) {
            let lambda = &analysis.lambdas[lam as usize];
            assert!(
                lambda.is_body() || lambda.is_lifted_from_closures(),
                "the set {} of lambdas alone holds {}, which needs the closure object holding it",
                id,
                lambda.to_string()
            );
            let instance = &spec.instances[builders.of(lam, args).instance];
            let component = &analysis.components[instance.component];
            for v in &lambda.held_in_place {
                let held = instance.var_sets[component.var_index[v]] as usize;
                if unboxable[held] {
                    holds.connect_idx(id, held);
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

/// How the values of each set of `spec` are represented.
///
/// The representation is decided in three steps.
///
/// 1. A set whose values no capture struct can stand for is represented by closures: a set holding
///    `Elem::Dynamic`, and a set whose layout holds itself, where the capture structs of its
///    elements hold in place, through the unboxed fields of the values they capture, a set that
///    holds in place this one again. Such a layout would have no size, as an unboxed type reaching
///    itself has none.
/// 2. The other sets and their elements are partitioned by the code they make: two elements are
///    equal when they name the same lambda and the sets at its interface (`Builder::interface`) are
///    equal, and two sets are equal when their elements are, counted up to that equality. The sets
///    of the first step are all equal to one another and to no other set, as a closure is the same
///    whatever it holds.
/// 3. A set whose elements make one class of the partition is represented by the capture struct of
///    that class. Every other set is represented by closures. A capture struct's type reads a set
///    of closures as a closure whatever the set holds, so two sets share a capture struct when their
///    elements are equal with the sets represented by closures counted equal.
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

    // Step 1: the sets a capture struct can stand for.
    let unboxable = unboxable_sets(analysis, spec, &builders);

    // Step 2: the partition of the sets by the code their values make. The sets of closures hold no
    // element the partition reads, so they make one class of their own.
    let elems: Vec<NodeElems<LamId>> = (0..set_count)
        .map(|id| {
            if !unboxable[id] {
                return vec![];
            }
            store
                .lambdas(id as SetId)
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
    let (block, block_elems) = canonical_classes(vec![0; set_count], &elems);

    // Step 3: the sets whose elements make one class are represented by a capture struct. Two of
    // them share the capture struct when their elements are equal with every set represented by
    // closures counted equal to every other, as a closure has one type whatever it holds. The
    // classes are numbered in the order of the sets: the first set of a class stands for it.
    let is_capture_struct: Vec<bool> = (0..set_count)
        .map(|id| unboxable[id] && block_elems[block[id]].len() == 1)
        .collect();
    let capture_struct_elems: Vec<NodeElems<LamId>> = (0..set_count)
        .map(|id| match is_capture_struct[id] {
            true => elems[id].clone(),
            false => vec![],
        })
        .collect();
    let (struct_block, struct_block_elems) = canonical_classes(vec![0; set_count], &capture_struct_elems);
    let mut class_of_block: Map<usize, u32> = Map::default();
    let mut first_set: Vec<usize> = vec![];
    let mut of_set: Vec<Repr> = vec![Repr::Closure; set_count];
    for id in 0..set_count {
        if !is_capture_struct[id] {
            continue;
        }
        assert_eq!(
            struct_block_elems[struct_block[id]].len(),
            1,
            "the elements of a set of one class differ once the sets of closures are counted equal"
        );
        let class = *class_of_block.entry(struct_block[id]).or_insert_with(|| {
            first_set.push(id);
            (first_set.len() - 1) as u32
        });
        of_set[id] = Repr::Single(class);
    }
    // The classes of the sets represented by a capture struct come first, numbered as `of_set`
    // names them, then the classes the elements of the sets of closures add.
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
        let class = *class_of_reprs.entry((lam, reprs.clone())).or_insert_with(|| {
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
    for (class, id) in first_set.iter().enumerate() {
        for (lam, args) in store.lambdas(*id as SetId) {
            assert_eq!(
                add_class(lam, args),
                class as u32,
                "an element of a set represented by a capture struct represents its interface \
                 otherwise than the set's class"
            );
        }
    }
    for id in 0..set_count {
        match of_set[id] {
            Repr::Single(class) => {
                for (lam, args) in store.lambdas(id as SetId) {
                    assert_eq!(
                        add_class(lam, args),
                        class,
                        "two elements of a set represented by a capture struct are in different \
                         classes"
                    );
                }
            }
            Repr::Closure => {
                for (lam, args) in store.lambdas(id as SetId) {
                    if analysis.lambdas[lam as usize].is_lifted_from_closures() {
                        add_class(lam, args);
                    }
                }
            }
        }
    }
    name_classes(analysis, &mut classes);

    let mut class_counts: Map<LamId, usize> = Map::default();
    for class in &classes {
        *class_counts.entry(class.lam).or_insert(0) += 1;
    }
    Representation {
        of_set,
        classes,
        class_of_elem,
        class_counts,
    }
}

/// Gives each of `classes` its hash: the MD5 digest of the text it is written as. The text of a
/// class names its lambda (`Lambda::to_string`), then writes the representation of each set at the
/// lambda's interface in parentheses: `c` for a set of closures, and for a set represented by a
/// capture struct the text of its class. A class met again on the way, which a class reaching
/// itself meets, is written `^n`, `n` counting the classes written so far in that text; a class
/// outside the strongly connected component being written is written `#` and its hash. The text
/// reads only the lambdas and the representations, so it is the same however the sets were
/// numbered.
fn name_classes(analysis: &Analysis, classes: &mut [LamClass]) {
    let mut graph = Graph::new((0..classes.len()).collect::<Vec<usize>>());
    for (c, class) in classes.iter().enumerate() {
        for arg in &class.interface_reprs {
            if let Repr::Single(a) = arg {
                graph.connect_idx(c, *a as usize);
            }
        }
    }
    for component in graph.components_dependencies_first() {
        let members: Set<usize> = component.iter().copied().collect();
        let texts: Vec<(usize, String)> = component
            .iter()
            .map(|c| {
                let mut text = String::new();
                let mut written: Map<usize, usize> = Map::default();
                write_class(analysis, classes, &members, *c, &mut written, &mut text);
                (*c, text)
            })
            .collect();
        for (c, text) in texts {
            classes[c].hash = md5_hex(&text);
        }
    }
    let distinct: Set<&str> = classes.iter().map(|class| class.hash.as_str()).collect();
    assert_eq!(
        distinct.len(),
        classes.len(),
        "two classes are written as the same text"
    );
}

/// Appends to `text` the text of the class `c`, a member of the strongly connected component
/// `component` of the classes, `written` holding the members written so far in this text, each
/// with the number of members written before it.
fn write_class(
    analysis: &Analysis,
    classes: &[LamClass],
    component: &Set<usize>,
    c: usize,
    written: &mut Map<usize, usize>,
    text: &mut String,
) {
    if let Some(n) = written.get(&c) {
        text.push_str(&format!("^{}", n));
        return;
    }
    written.insert(c, written.len());
    text.push_str(&analysis.lambdas[classes[c].lam as usize].to_string());
    text.push('(');
    for (i, arg) in classes[c].interface_reprs.iter().enumerate() {
        if i > 0 {
            text.push(',');
        }
        match arg {
            Repr::Closure => text.push('c'),
            Repr::Single(a) if component.contains(&(*a as usize)) => {
                write_class(analysis, classes, component, *a as usize, written, text)
            }
            Repr::Single(a) => {
                text.push('#');
                text.push_str(&classes[*a as usize].hash);
            }
        }
    }
    text.push(')');
}
