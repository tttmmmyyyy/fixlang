use std::hash::Hash;

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
}
