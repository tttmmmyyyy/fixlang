// The call graph of the symbols of a program.

use crate::{
    ast::{name::FullName, program::Symbol},
    graph::Graph,
    misc::Map,
};

/// The call graph of the symbols `names` of `symbols` — an edge from A to B means A's body refers
/// to B — with a node for each of `names`, numbered in that order.
///
/// `names` must list every symbol that the bodies of the listed symbols refer to. The edges of a
/// node run in the order of the nodes they reach.
pub fn call_graph_of(symbols: &Map<FullName, Symbol>, names: Vec<FullName>) -> Graph<FullName> {
    let name_to_idx = names
        .iter()
        .enumerate()
        .map(|(idx, name)| (name.clone(), idx))
        .collect::<Map<FullName, usize>>();
    assert_eq!(name_to_idx.len(), names.len(), "`names` lists a symbol twice");
    let mut graph = Graph::new(names);
    for caller_idx in 0..graph.len() {
        let caller = graph.get(caller_idx);
        let mut callees: Vec<usize> = symbols[caller]
            .expr
            .as_ref()
            .unwrap()
            .free_vars()
            .iter()
            .map(|callee| {
                *name_to_idx.get(callee).unwrap_or_else(|| {
                    panic!(
                        "`{}` refers to `{}`, which `names` does not list",
                        caller.to_string(),
                        callee.to_string()
                    )
                })
            })
            .collect();
        callees.sort();
        for callee_idx in callees {
            graph.connect_idx(caller_idx, callee_idx);
        }
    }
    graph
}
