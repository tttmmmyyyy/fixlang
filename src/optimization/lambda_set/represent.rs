// How the values of each concrete lambda set are represented once the program is rewritten.

use super::{
    concrete::{Elem, SetId},
    infer::Analysis,
    solver::LamId,
    specialize::Specialization,
};
use crate::{graph::Graph, hash::md5_hex, misc::Map};

/// How the values of a concrete set are represented.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Repr {
    /// As closures, the form a function value has without this pass.
    Closure,
    /// As the capture struct of the one lambda of the set, whose values are called by calling that
    /// lambda directly. Holds the class of the sets represented alike, as its index into
    /// `Representation::classes`.
    Single(u32),
}

/// The sets whose values are represented by the capture struct of one lambda, and represented
/// alike: their elements name one lambda, and give its parameters sets represented alike.
pub struct SingleClass {
    /// The lambda.
    pub lam: LamId,
    /// How each set the element gives the parameters of the lambda is represented, in the order of
    /// `Lambda::params`.
    pub args: Vec<Repr>,
    /// The copy of the lambda's component that builds the values of one of the sets, as an index
    /// into `Specialization::instances`. Every set of the class gives the lambda's body the same
    /// types, so the sets of this copy are the ones the lambda's body is rewritten with.
    pub instance: usize,
    /// The hash of the text the class is written as, which is the same in every compilation of the
    /// program, and different for two classes.
    pub hash: String,
}

/// How the values of every set of a specialized program are represented.
pub struct Representation {
    /// How each set is represented, by its id.
    of_set: Vec<Repr>,
    /// The classes of the sets represented by a capture struct.
    pub classes: Vec<SingleClass>,
    /// The number of classes of each lambda that has any.
    class_counts: Map<LamId, usize>,
}

impl Representation {
    /// How the values of `set` are represented.
    pub fn of(&self, set: SetId) -> Repr {
        self.of_set[set as usize]
    }

    /// The number of classes whose lambda is `lam`.
    pub fn class_count(&self, lam: LamId) -> usize {
        self.class_counts.get(&lam).copied().unwrap_or(0)
    }

    /// The text a `Repr` is written as in the hashes that name copies.
    pub fn hash_text(&self, repr: Repr) -> &str {
        match repr {
            Repr::Closure => "c",
            Repr::Single(c) => &self.classes[c as usize].hash,
        }
    }
}

/// How the values of each set of `spec` are represented.
///
/// A set whose one element is a lambda is represented by that lambda's capture struct, unless its
/// layout holds itself: the capture struct holds in place, through the unboxed fields of the
/// values it captures, the capture struct of a set that holds in place this one again. Such a set
/// would have no size, as an unboxed type reaching itself has none, so its values stay closures,
/// which hold their captures behind a pointer. Every other set is represented by closures.
pub fn represent(analysis: &Analysis, spec: &Specialization) -> Representation {
    let store = &spec.store;
    let set_count = store.len();

    // The copy that builds the values of each element naming a lambda: the first copy of the
    // lambda's component, in the order of the copies, that gives the lambda's parameters the
    // element's sets.
    let mut lambdas_of_component: Vec<Vec<LamId>> = vec![vec![]; analysis.components.len()];
    for (lam, lambda) in analysis.lambdas.iter().enumerate() {
        lambdas_of_component[analysis.component_of[&lambda.symbol]].push(lam as LamId);
    }
    let mut builder: Map<(LamId, Vec<SetId>), usize> = Map::default();
    for (k, instance) in spec.instances.iter().enumerate() {
        for lam in &lambdas_of_component[instance.component] {
            let args = analysis.lambdas[*lam as usize]
                .params
                .iter()
                .map(|p| instance.args[*p])
                .collect();
            builder.entry((*lam, args)).or_insert(k);
        }
    }
    let builder_of = |lam: LamId, args: &[SetId]| -> usize {
        *builder.get(&(lam, args.to_vec())).unwrap_or_else(|| {
            panic!(
                "no copy builds the values of {} given the sets {:?}",
                analysis.lambdas[lam as usize].to_string(),
                args
            )
        })
    };

    // The sets of one lambda, and the capture structs each holds in place.
    let single: Vec<Option<(LamId, &[SetId])>> = (0..set_count)
        .map(|id| match store.elems(id as SetId) {
            [Elem::Lam(lam, args)] => Some((*lam, args.as_slice())),
            _ => None,
        })
        .collect();
    let mut holds = Graph::new((0..set_count).collect::<Vec<usize>>());
    for (id, elem) in single.iter().enumerate() {
        let Some((lam, args)) = elem else {
            continue;
        };
        let instance = &spec.instances[builder_of(*lam, args)];
        let component = &analysis.components[instance.component];
        for v in &analysis.lambdas[*lam as usize].held_in_place {
            let held = instance.sets[component.var_index[v]] as usize;
            if single[held].is_some() {
                holds.connect_idx(id, held);
            }
        }
    }
    let mut is_capture_struct: Vec<bool> = single.iter().map(|s| s.is_some()).collect();
    for component in holds.components_dependencies_first() {
        if holds.is_cycle(&component) {
            for id in component {
                is_capture_struct[id] = false;
            }
        }
    }

    // The classes: the coarsest partition of the sets represented by capture structs in which two
    // sets of one class name the same lambda and give its parameters sets of the same class, or
    // sets both represented by closures. Refine a partition by those until no class splits; the
    // closures make one block of their own, numbered 0.
    let mut block: Vec<usize> = (0..set_count)
        .map(|id| match single[id] {
            Some((lam, _)) if is_capture_struct[id] => 1 + lam as usize,
            _ => 0,
        })
        .collect();
    let mut block_count = usize::MAX;
    loop {
        let mut block_of_signature: Map<(usize, Vec<usize>), usize> = Map::default();
        block_of_signature.insert((0, vec![]), 0);
        let mut next = vec![0; set_count];
        for id in 0..set_count {
            if block[id] == 0 {
                continue;
            }
            let (_, args) = single[id].unwrap();
            let signature = (
                block[id],
                args.iter().map(|a| block[*a as usize]).collect(),
            );
            let count = block_of_signature.len();
            next[id] = *block_of_signature.entry(signature).or_insert(count);
        }
        block = next;
        if block_of_signature.len() == block_count {
            break;
        }
        block_count = block_of_signature.len();
    }

    // Number the classes in the order of the sets: the first set of a class stands for it.
    let mut class_of_block: Map<usize, u32> = Map::default();
    let mut first_set: Vec<usize> = vec![];
    let mut of_set: Vec<Repr> = vec![Repr::Closure; set_count];
    for id in 0..set_count {
        if block[id] == 0 {
            continue;
        }
        let class = *class_of_block.entry(block[id]).or_insert_with(|| {
            first_set.push(id);
            (first_set.len() - 1) as u32
        });
        of_set[id] = Repr::Single(class);
    }
    let mut classes: Vec<SingleClass> = first_set
        .iter()
        .map(|id| {
            let (lam, args) = single[*id].unwrap();
            SingleClass {
                lam,
                args: args.iter().map(|a| of_set[*a as usize]).collect(),
                instance: builder_of(lam, args),
                hash: String::new(),
            }
        })
        .collect();
    name_classes(analysis, &mut classes);

    let mut class_counts: Map<LamId, usize> = Map::default();
    for class in &classes {
        *class_counts.entry(class.lam).or_insert(0) += 1;
    }
    Representation {
        of_set,
        classes,
        class_counts,
    }
}

/// Gives each of `classes` its hash: the MD5 digest of the text it is written as. The text of a
/// class names its lambda (`Lambda::to_string`), then writes each argument in parentheses: `c` for
/// a set of closures, and for another class its text. A class met again on the way, which a class
/// reaching itself meets, is written `^n`, `n` counting the classes written so far in that text;
/// a class outside the strongly connected component being written is written by its hash. The
/// text reads only the lambdas and the arguments, so it is the same however the sets were
/// numbered.
fn name_classes(analysis: &Analysis, classes: &mut [SingleClass]) {
    let mut graph = Graph::new((0..classes.len()).collect::<Vec<usize>>());
    for (c, class) in classes.iter().enumerate() {
        for arg in &class.args {
            if let Repr::Single(a) = arg {
                graph.connect_idx(c, *a as usize);
            }
        }
    }
    for component in graph.components_dependencies_first() {
        let texts: Vec<(usize, String)> = component
            .iter()
            .map(|c| {
                let mut text = String::new();
                let mut written: Vec<usize> = vec![];
                write_class(analysis, classes, &component, *c, &mut written, &mut text);
                (*c, text)
            })
            .collect();
        for (c, text) in texts {
            classes[c].hash = md5_hex(&text);
        }
    }
}

/// Appends to `text` the text of the class `c`, a member of the strongly connected component
/// `component` of the classes, `written` holding the members written so far in this text.
fn write_class(
    analysis: &Analysis,
    classes: &[SingleClass],
    component: &[usize],
    c: usize,
    written: &mut Vec<usize>,
    text: &mut String,
) {
    if let Some(n) = written.iter().position(|w| *w == c) {
        text.push_str(&format!("^{}", n));
        return;
    }
    written.push(c);
    text.push_str(&analysis.lambdas[classes[c].lam as usize].to_string());
    text.push('(');
    for (i, arg) in classes[c].args.iter().enumerate() {
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
