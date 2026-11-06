// The classes of equal nodes of a graph whose nodes hold labelled elements, numbered by the trees
// the nodes unfold to.

use super::concrete::SetId;

/// What an element of a node refers to.
#[derive(Clone, Copy)]
pub enum Child {
    /// Another node, by its position.
    Node(usize),
    /// A set held outside the graph, which is equal to no node and to no set but itself.
    Fixed(SetId),
}

/// A child as the refinement compares it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChildKey {
    /// A node, by the class it is in.
    Class(usize),
    /// A set held outside the graph.
    Fixed(SetId),
}

/// The elements of a node: each a label and the children it refers to.
pub type NodeElems<L> = Vec<(L, Vec<Child>)>;

/// An element as the refinement compares it: its label and the classes of its children.
pub type ElemKey<L> = (L, Vec<ChildKey>);

/// The classes of equal nodes among nodes whose elements are `elems`, each node by its position,
/// and the elements of each class, sorted and written with the classes of the children.
///
/// Two nodes are equal when they start in one class and their elements, read through the classes
/// of their children, are the same set; the partition is refined until no class splits. Each round
/// numbers the classes in the order of their signatures, which read only the labels and the classes
/// of the round before, so the number a class gets says which tree its nodes unfold to: nodes
/// reaching equal sets of trees get the same numbers, however they are positioned.
///
/// # Arguments
/// * `initial` - the class each node starts in, numbered from 0 with no number skipped, by what the
///   nodes are rather than where they are positioned.
pub fn canonical_classes<L: Ord + Clone>(
    initial: Vec<usize>,
    elems: &[NodeElems<L>],
) -> (Vec<usize>, Vec<Vec<ElemKey<L>>>) {
    assert_eq!(initial.len(), elems.len());
    let mut class = initial;
    let mut class_count = class.iter().max().map_or(0, |c| c + 1);
    loop {
        let signatures: Vec<(usize, Vec<ElemKey<L>>)> = elems
            .iter()
            .enumerate()
            .map(|(p, node_elems)| {
                let mut keys: Vec<ElemKey<L>> = node_elems
                    .iter()
                    .map(|(label, children)| {
                        let children = children
                            .iter()
                            .map(|ch| match ch {
                                Child::Node(q) => ChildKey::Class(class[*q]),
                                Child::Fixed(id) => ChildKey::Fixed(*id),
                            })
                            .collect();
                        (label.clone(), children)
                    })
                    .collect();
                keys.sort();
                keys.dedup();
                (class[p], keys)
            })
            .collect();
        let mut distinct = signatures.clone();
        distinct.sort();
        distinct.dedup();
        if distinct.len() == class_count {
            // No class split, so the classes keep their numbers: each signature carries the class it
            // refines first, and the classes sort in that order.
            assert!(
                distinct
                    .iter()
                    .enumerate()
                    .all(|(c, (refined, _))| *refined == c),
                "the classes a partition starts with skip a number"
            );
            let class_elems = distinct.into_iter().map(|(_, keys)| keys).collect();
            return (class, class_elems);
        }
        class_count = distinct.len();
        class = signatures
            .iter()
            .map(|sig| distinct.binary_search(sig).unwrap())
            .collect();
    }
}
