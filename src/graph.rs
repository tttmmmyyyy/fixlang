use std::{collections::VecDeque, hash::Hash};

use crate::misc::{Map, Set};

pub struct Graph<T> {
    // Data stored in nodes;
    elems: Vec<T>,
    // Edges
    // edges[i] = set of (indices of) nodes where an edge i -> j exists.
    edges: Vec<Vec<usize>>,
}

#[allow(dead_code)]
impl<T> Graph<T> {
    // Create a graph from a vector of elements.
    pub fn new(elems: Vec<T>) -> Self {
        let len = elems.len();
        Graph {
            elems,
            edges: vec![vec![]; len],
        }
    }

    pub fn new_with_edges(elems: Vec<T>, edges: Vec<Vec<usize>>) -> Self {
        let len = elems.len();
        assert_eq!(len, edges.len());
        Graph { elems, edges }
    }

    // Create a graph from a set of elements.
    // Returns a map from element to a node.
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

    // Get an element
    pub fn get(&self, node: usize) -> &T {
        self.elems.get(node).unwrap()
    }

    // Connect two nodes by indices.
    pub fn connect_idx(&mut self, from: usize, to: usize) {
        self.edges.get_mut(from).unwrap().push(to);
    }

    // Connect two nodes.
    pub fn connect(&mut self, from: &T, to: &T)
    where
        T: Eq,
    {
        let from = self.elem_to_idx(from).unwrap();
        let to = self.elem_to_idx(to).unwrap();
        self.connect_idx(from, to);
    }

    // Get the index of an element.
    pub fn elem_to_idx(&self, elem: &T) -> Option<usize>
    where
        T: Eq,
    {
        self.elems.iter().position(|e| e == elem)
    }

    // Collect nodes reachable from a node.
    fn reachable_nodes_inner(&self, from: usize, visited: &mut Set<usize>) {
        if visited.contains(&from) {
            return;
        }
        visited.insert(from);
        for to in &self.edges[from] {
            self.reachable_nodes_inner(*to, visited)
        }
    }

    // Collect nodes reachable from a node.
    pub fn reachable_nodes(&self, from: usize) -> Set<usize> {
        let mut nodes: Set<usize> = Default::default();
        self.reachable_nodes_inner(from, &mut nodes);
        nodes
    }

    // Find a loop.
    // If this function finds a loop a(1) -> a(2) -> ... -> a(n) -> a(1), it returns vec![a(1), a(2), ... , a(n)].
    // If there is no loop in the graph, this function returns an empty Vec.
    #[allow(dead_code)]
    pub fn find_loop(&self) -> Vec<usize> {
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

    // Performs a strongly connected component (SCC) decomposition and returns the ID of the SCC to which each vertex belongs.
    //
    // The output `scc_ids` is a vector where:
    // - `scc_ids[i] == scc_ids[j]` iff that vertices `i` and `j` are in the same SCC.
    // - If there is a path from `i` to `j` then `scc_ids[i] <= scc_ids[j]`.
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
                reversed_edges[v].push(u); // 辺 u -> v を v -> u に反転
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

    // Helper function for the first DFS (DFS1)
    fn scc_dfs1(&self, u: usize, visited: &mut Vec<bool>, finish_order: &mut VecDeque<usize>) {
        visited[u] = true;
        for &v in &self.edges[u] {
            if !visited[v] {
                self.scc_dfs1(v, visited, finish_order);
            }
        }
        // DFSが完了したノードをスタックに積む
        finish_order.push_back(u);
    }

    // Helper function for the second DFS (DFS2)
    fn scc_dfs2(
        &self,
        u: usize,
        current_scc_id: usize,
        reversed_edges: &Vec<Vec<usize>>,
        visited: &mut Vec<bool>,
        scc_ids: &mut Vec<usize>,
    ) {
        visited[u] = true;
        scc_ids[u] = current_scc_id; // 現在のSCCのIDを割り当てる

        for &v in &reversed_edges[u] {
            if !visited[v] {
                self.scc_dfs2(v, current_scc_id, reversed_edges, visited, scc_ids);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scc_empty_graph() {
        let graph: Graph<char> = Graph::new_with_edges(vec![], vec![]);
        let scc_ids = graph.compute_sccs();
        assert_eq!(scc_ids, Vec::<usize>::new());
    }

    #[test]
    fn test_scc_single_node() {
        let graph = Graph::new_with_edges(vec!['A'], vec![vec![]]);
        let scc_ids = graph.compute_sccs();
        assert_eq!(scc_ids, vec![0]);
    }

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

    #[test]
    pub fn test_find_loop() {
        // Test find_loop of graph.rs.

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
