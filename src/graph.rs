// graph

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    edges: HashMap<String, Vec<String>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, key: &str) {
        self.edges.entry(key.to_string()).or_insert_with(Vec::new);
    }

    pub fn add_edge(&mut self, key_a: &str, key_b: &str) {
        self.edges.entry(key_a.to_string()).or_insert_with(Vec::new).push(key_b.to_string());
        self.edges.entry(key_b.to_string()).or_insert_with(Vec::new).push(key_a.to_string());
    }

    pub fn remove_edge(&mut self, key_a: &str, key_b: &str) {
        let Some(map) = self.edges.get_mut(key_a) else {
            return;
            // Todo:
        };

        map.retain(|key| key != key_b);

        let Some(map) = self.edges.get_mut(key_b) else {
            return;
            // Todo:
        };

        map.retain(|key| key != key_a);
    }

    pub fn neighbors(&self, key: &str) -> Option<&Vec<String>> {
        self.edges.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_graph_has_no_edges() {
        let graph = Graph::new();

        assert_eq!(graph.edges.get("a"), None);
    }

    #[test]
    fn add_edge_registers_both_directions() {
        let mut graph = Graph::new();

        graph.add_edge("a", "b");

        assert_eq!(graph.edges.get("a"), Some(&vec!["b".to_string()]));
        assert_eq!(graph.edges.get("b"), Some(&vec!["a".to_string()]));
    }

    #[test]
    fn add_edge_appends_to_existing_neighbors() {
        let mut graph = Graph::new();

        graph.add_edge("a", "b");
        graph.add_edge("a", "c");

        assert_eq!(graph.edges.get("a"), Some(&vec!["b".to_string(), "c".to_string()]));
    }
}
