use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct Graph<T> {
    // Data stored in nodes;
    elems: Vec<T>,
    // Edges
    edges: Vec<Vec<usize>>,
}

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
    pub fn from_set(elems: HashSet<T>) -> (Self, HashMap<T, usize>)
    where
        T: Clone + Eq + Hash,
    {
        let elems = elems.into_iter().collect::<Vec<_>>();
        let inv_map = elems
            .iter()
            .enumerate()
            .map(|(idx, elem)| (elem.clone(), idx))
            .collect::<HashMap<_, _>>();
        (Graph::new(elems), inv_map)
    }
    // Get an element
    pub fn get(&self, node: usize) -> &T {
        self.elems.get(node).unwrap()
    }
    // Connect two nodes.
    pub fn connect(&mut self, from: usize, to: usize) {
        self.edges.get_mut(from).unwrap().push(to);
    }
    // Collect nodes reachable from a set of nodes.
    pub fn reachable_nodes_from_set(&self, froms: &Vec<usize>) -> HashSet<usize> {
        let mut nodes: HashSet<usize> = Default::default();
        for from in froms {
            self.reachable_nodes_inner(*from, &mut nodes);
        }
        nodes
    }
    // Collect nodes reachable from a node.
    fn reachable_nodes_inner(&self, from: usize, visited: &mut HashSet<usize>) {
        if visited.contains(&from) {
            return;
        }
        visited.insert(from);
        for to in &self.edges[from] {
            self.reachable_nodes_inner(*to, visited)
        }
    }
}
