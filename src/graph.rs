use std::{collections::VecDeque, hash::Hash};

use crate::misc::{Map, Set};

/// A directed graph whose nodes are the indices `0..len()`, each holding an element of type `T`.
pub struct Graph<T> {
    /// The element held by each node, indexed by node.
    elems: Vec<T>,
    /// The outgoing edges: `edges[i]` lists every node `j` with an edge `i -> j`.
    edges: Vec<Vec<usize>>,
}

impl<T> Graph<T> {
    /// Creates a graph with one node per element of `elems` and no edges.
    pub fn new(elems: Vec<T>) -> Self {
        let len = elems.len();
        Graph {
            elems,
            edges: vec![vec![]; len],
        }
    }

    /// Creates a graph with one node per element of `elems` and the outgoing edges `edges`, which
    /// has one entry per node.
    pub fn new_with_edges(elems: Vec<T>, edges: Vec<Vec<usize>>) -> Self {
        let len = elems.len();
        assert_eq!(len, edges.len());
        Graph { elems, edges }
    }

    /// The number of nodes.
    pub fn len(&self) -> usize {
        self.elems.len()
    }

    /// Creates a graph with one node per element of `elems` and no edges, together with the map
    /// from each element to its node.
    pub fn from_set(elems: Set<T>) -> (Self, Map<T, usize>)
    where
        T: Clone + Eq + Hash,
    {
        let elems = elems.into_iter().collect::<Vec<_>>();
        let inv_map = elems
            .iter()
            .enumerate()
            .map(|(idx, elem)| (elem.clone(), idx))
            .collect::<Map<_, _>>();
        (Graph::new(elems), inv_map)
    }

    /// The element held by `node`.
    pub fn get(&self, node: usize) -> &T {
        self.elems.get(node).unwrap()
    }

    /// Adds the edge `from -> to`.
    pub fn connect_idx(&mut self, from: usize, to: usize) {
        self.edges.get_mut(from).unwrap().push(to);
    }

    /// Adds to `visited` every node reachable from `from`, `from` included, skipping the nodes
    /// already in `visited`.
    fn reachable_nodes_inner(&self, from: usize, visited: &mut Set<usize>) {
        if visited.contains(&from) {
            return;
        }
        visited.insert(from);
        for to in &self.edges[from] {
            self.reachable_nodes_inner(*to, visited)
        }
    }

    /// The nodes reachable from `from`, `from` included.
    pub fn reachable_nodes(&self, from: usize) -> Set<usize> {
        let mut nodes: Set<usize> = Default::default();
        self.reachable_nodes_inner(from, &mut nodes);
        nodes
    }

    /// Finds a loop. For a loop `a(1) -> a(2) -> ... -> a(n) -> a(1)`, returns
    /// `vec![a(1), a(2), ..., a(n)]`; when the graph has no loop, returns an empty `Vec`.
    #[allow(dead_code)]
    pub fn find_loop(&self) -> Vec<usize> {
        /// Searches depth-first from `pos` for a loop, with `path_set` and `path_vec` holding the
        /// path from the search's root to `pos`, and `verified` the nodes from which no loop is
        /// reachable. Returns the loop found, or an empty `Vec` when none is reachable from `pos`.
        fn visit<T>(
            pos: usize,
            path_set: &mut Set<usize>,
            path_vec: &mut Vec<usize>,
            verified: &mut Set<usize>,
            graph: &Graph<T>,
        ) -> Vec<usize> {
            if path_set.contains(&pos) {
                // Loop found.
                for i in 0..path_vec.len() {
                    if path_vec[i] == pos {
                        return Vec::from(path_vec.split_at(i).1);
                    }
                }
                unreachable!()
            }
            if verified.contains(&pos) {
                return vec![];
            }
            path_set.insert(pos);
            path_vec.push(pos);
            for next in &graph.edges[pos] {
                let maybe_loop = visit(*next, path_set, path_vec, verified, graph);
                if !maybe_loop.is_empty() {
                    return maybe_loop;
                }
            }
            path_set.remove(&pos);
            path_vec.pop();
            verified.insert(pos);
            return vec![];
        }

        let mut path_set: Set<usize> = Default::default();
        let mut path_vec: Vec<usize> = Default::default();
        let mut visited: Set<usize> = Default::default();
        path_set.reserve(self.elems.len());
        visited.reserve(self.elems.len());

        for pos in 0..self.elems.len() {
            let maybe_loop = visit(pos, &mut path_set, &mut path_vec, &mut visited, self);
            if !maybe_loop.is_empty() {
                return maybe_loop;
            }
        }
        return vec![];
    }

    /// Decomposes the graph into strongly connected components (SCCs) and returns, for each node,
    /// the id of the SCC it belongs to.
    ///
    /// The returned `scc_ids` satisfies:
    /// - `scc_ids[i] == scc_ids[j]` if and only if nodes `i` and `j` are in the same SCC.
    /// - If there is a path from `i` to `j`, then `scc_ids[i] <= scc_ids[j]`.
    pub fn compute_sccs(&self) -> Vec<usize> {
        let num_nodes = self.elems.len();
        if num_nodes == 0 {
            return Vec::new();
        }

        // 1. First DFS (DFS1): Push nodes onto the stack in order of their finish times
        let mut visited_dfs1 = vec![false; num_nodes];
        let mut finish_order = VecDeque::new(); // stack for finish order

        for i in 0..num_nodes {
            if !visited_dfs1[i] {
                self.scc_dfs1(i, &mut visited_dfs1, &mut finish_order);
            }
        }

        // 2. Construct the transposed graph
        let mut reversed_edges: Vec<Vec<usize>> = vec![vec![]; num_nodes];
        for u in 0..num_nodes {
            for &v in &self.edges[u] {
                reversed_edges[v].push(u); // Reverse the edge u -> v into v -> u.
            }
        }

        // 3. Second DFS (DFS2): Perform DFS on the transposed graph in the order of decreasing finish times
        let mut scc_ids = vec![0; num_nodes];
        let mut visited_dfs2 = vec![false; num_nodes];
        let mut current_scc_id = 0;

        while let Some(node) = finish_order.pop_back() {
            if !visited_dfs2[node] {
                // A new SCC is found
                self.scc_dfs2(
                    node,
                    current_scc_id,
                    &reversed_edges,
                    &mut visited_dfs2,
                    &mut scc_ids,
                );
                current_scc_id += 1;
            }
        }

        scc_ids
    }

    /// The strongly connected components, each listing its nodes in increasing order, and each
    /// placed after every other component it has a path to.
    pub fn components_dependencies_first(&self) -> Vec<Vec<usize>> {
        let scc_ids = self.compute_sccs();
        let count = scc_ids.iter().max().map_or(0, |m| m + 1);
        let mut components = vec![vec![]; count];
        for (node, id) in scc_ids.iter().enumerate() {
            components[*id].push(node);
        }
        // `compute_sccs` gives a component an id no larger than that of any component it reaches.
        components.reverse();
        components
    }

    /// Whether the strongly connected component `component` holds a cycle: it has more than one
    /// node, or its one node has an edge to itself.
    pub fn is_cycle(&self, component: &[usize]) -> bool {
        component.len() > 1 || self.edges[component[0]].contains(&component[0])
    }

    /// The first depth-first search of `compute_sccs`: marks in `visited` every node reachable
    /// from `u`, and pushes each newly visited node onto `finish_order` once its search finishes.
    fn scc_dfs1(&self, u: usize, visited: &mut Vec<bool>, finish_order: &mut VecDeque<usize>) {
        visited[u] = true;
        for &v in &self.edges[u] {
            if !visited[v] {
                self.scc_dfs1(v, visited, finish_order);
            }
        }
        // The search from `u` has finished, so push `u` onto the stack.
        finish_order.push_back(u);
    }

    /// The second depth-first search of `compute_sccs`: assigns `current_scc_id` to every node
    /// reachable from `u` along `reversed_edges` that is not yet marked in `visited`.
    fn scc_dfs2(
        &self,
        u: usize,
        current_scc_id: usize,
        reversed_edges: &Vec<Vec<usize>>,
        visited: &mut Vec<bool>,
        scc_ids: &mut Vec<usize>,
    ) {
        visited[u] = true;
        scc_ids[u] = current_scc_id;

        for &v in &reversed_edges[u] {
            if !visited[v] {
                self.scc_dfs2(v, current_scc_id, reversed_edges, visited, scc_ids);
            }
        }
    }
}

/// Tests of `Graph`.
#[cfg(test)]
mod tests {
    use super::*;

    /// `compute_sccs` of the empty graph is empty.
    #[test]
    fn test_scc_empty_graph() {
        let graph: Graph<char> = Graph::new_with_edges(vec![], vec![]);
        let scc_ids = graph.compute_sccs();
        assert_eq!(scc_ids, Vec::<usize>::new());
    }

    /// `compute_sccs` puts the one node of a single-node graph in SCC 0.
    #[test]
    fn test_scc_single_node() {
        let graph = Graph::new_with_edges(vec!['A'], vec![vec![]]);
        let scc_ids = graph.compute_sccs();
        assert_eq!(scc_ids, vec![0]);
    }

    /// `compute_sccs` puts each node of a path in its own SCC, numbered along the path.
    #[test]
    fn test_scc_linear_graph() {
        // A -> B -> C
        let graph = Graph::new_with_edges(
            vec!['A', 'B', 'C'],
            vec![
                vec![1], // A -> B
                vec![2], // B -> C
                vec![],  // C
            ],
        );
        let scc_ids = graph.compute_sccs();
        assert_eq!(scc_ids, vec![0, 1, 2]);
    }

    /// `compute_sccs` puts the two nodes of a two-node cycle in one SCC.
    #[test]
    fn test_scc_simple_cycle() {
        // A <-> B
        let graph = Graph::new_with_edges(
            vec!['A', 'B'],
            vec![
                vec![1], // A -> B
                vec![0], // B -> A
            ],
        );
        let scc_ids = graph.compute_sccs();
        assert_eq!(scc_ids, vec![0, 0]);
    }

    /// `compute_sccs` on a graph of four SCCs, some of which reach others.
    #[test]
    fn test_scc_complex_graph() {
        let graph = Graph::new_with_edges(
            vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'], // 0-7
            vec![
                vec![1],       // 0 -> 1
                vec![2, 4, 5], // 1 -> 2, 4, 5
                vec![3, 6],    // 2 -> 3, 6
                vec![2, 7],    // 3 -> 2, 7
                vec![0, 5],    // 4 -> 0, 5
                vec![6],       // 5 -> 6
                vec![5, 7],    // 6 -> 5, 7
                vec![7],       // 7 -> 7
            ],
        );

        let scc_ids = graph.compute_sccs();
        assert_eq!(scc_ids, [0, 0, 1, 1, 0, 2, 2, 3]);
    }

    /// `components_dependencies_first` places each component after those it reaches, and
    /// `is_cycle` tells the components holding a cycle.
    #[test]
    fn test_components_dependencies_first() {
        // 0 -> 1 <-> 2 -> 3, 4 -> 4, 5 -> 3
        let graph = Graph::new_with_edges(
            vec!['a', 'b', 'c', 'd', 'e', 'f'],
            vec![vec![1], vec![2], vec![1, 3], vec![], vec![4], vec![3]],
        );
        let components = graph.components_dependencies_first();
        let mut position = vec![0; graph.len()];
        for (i, component) in components.iter().enumerate() {
            for node in component {
                position[*node] = i;
            }
        }
        let mut sorted = components.clone();
        sorted.sort();
        assert_eq!(sorted, vec![vec![0], vec![1, 2], vec![3], vec![4], vec![5]]);
        assert!(position[3] < position[1]);
        assert!(position[1] < position[0]);
        assert!(position[3] < position[5]);
        let cycles: Vec<bool> = sorted.iter().map(|c| graph.is_cycle(c)).collect();
        assert_eq!(cycles, vec![false, true, false, true, false]);

        let graph: Graph<char> = Graph::new_with_edges(vec![], vec![]);
        assert!(graph.components_dependencies_first().is_empty());
    }

    /// `find_loop` finds a loop exactly when the graph has one.
    #[test]
    pub fn test_find_loop() {
        let g = Graph::new((0..3).collect());
        assert_eq!(g.find_loop(), vec![] as Vec<usize>);

        let mut g = Graph::new((0..3).collect());
        g.connect_idx(0, 1);
        g.connect_idx(1, 2);
        assert_eq!(g.find_loop(), vec![] as Vec<usize>);

        let mut g = Graph::new((0..3).collect());
        g.connect_idx(0, 0);
        assert_eq!(g.find_loop(), vec![0 as usize]);

        let mut g = Graph::new((0..3).collect());
        g.connect_idx(1, 1);
        assert_eq!(g.find_loop(), vec![1 as usize]);

        let mut g = Graph::new((0..3).collect());
        g.connect_idx(0, 1);
        g.connect_idx(2, 2);
        assert_eq!(g.find_loop(), vec![2 as usize]);

        let mut g = Graph::new((0..3).collect());
        g.connect_idx(1, 2);
        g.connect_idx(2, 1);
        assert_eq!(g.find_loop(), vec![1 as usize, 2 as usize]);

        let mut g = Graph::new((0..4).collect());
        g.connect_idx(0, 1);
        g.connect_idx(1, 2);
        g.connect_idx(1, 3);
        g.connect_idx(2, 3);
        assert_eq!(g.find_loop(), vec![] as Vec<usize>);

        let mut g = Graph::new((0..5).collect());
        g.connect_idx(0, 1);
        g.connect_idx(1, 2);
        g.connect_idx(1, 3);
        g.connect_idx(3, 4);
        g.connect_idx(4, 1);
        assert_eq!(
            g.find_loop(),
            vec![1 as usize, 3 as usize, 4 as usize] as Vec<usize>
        );
    }
}
