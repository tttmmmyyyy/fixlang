// The call graph of the symbols of a program.

use crate::{
    ast::{name::FullName, program::Symbol},
    graph::Graph,
    misc::Map,
};

/// The call graph of `symbols` — an edge from A to B means A's body refers to B — with a node for
/// each of `names`, numbered in that order, together with the node each name is held at.
///
/// `names` lists every key of `symbols`. The edges of a node run in the order of the nodes they
/// reach. A name a body refers to that `names` does not list carries no edge.
pub fn call_graph_of(
    symbols: &Map<FullName, Symbol>,
    names: Vec<FullName>,
) -> (Graph<FullName>, Map<FullName, usize>) {
    assert_eq!(names.len(), symbols.len());
    let name_to_idx = names
        .iter()
        .enumerate()
        .map(|(idx, name)| (name.clone(), idx))
        .collect::<Map<FullName, usize>>();
    let mut graph = Graph::new(names);
    for caller_idx in 0..graph.len() {
        let caller = graph.get(caller_idx);
        let mut callees: Vec<usize> = symbols[caller]
            .expr
            .as_ref()
            .unwrap()
            .free_vars()
            .iter()
            .filter_map(|callee| name_to_idx.get(callee).copied())
            .collect();
        callees.sort();
        for callee_idx in callees {
            graph.connect_idx(caller_idx, callee_idx);
        }
    }
    (graph, name_to_idx)
}
