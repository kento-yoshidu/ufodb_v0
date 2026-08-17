use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod union_find;
mod graph;
pub mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct Ufdb {
    keys: HashMap<String, usize>,
    uf: union_find::UnionFind,
    graph: graph::Graph,
}

impl Ufdb {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            uf: union_find::UnionFind::new(),
            graph: graph::Graph::new(),
        }
    }

    pub fn make_set(&mut self, key: &str) -> bool {
        if !self.keys.contains_key(key) {
            let index = self.uf.add();

            self.keys.insert(key.to_string(), index);
            self.graph.add_node(key);

            true
        } else {
            false
        }
    }

    pub fn unite(&mut self, key_a: &str, key_b: &str) -> bool {
        self.make_set(key_a);
        self.make_set(key_b);

        let a_index = self.keys[key_a];
        let b_index = self.keys[key_b];

        if self.uf.unite(a_index, b_index) {
            self.graph.add_edge(key_a, key_b);
            true
        } else {
            false
        }
    }

    pub fn same(&mut self, key_a: &str, key_b: &str) -> bool {
        let (Some(a_index), Some(b_index)) = (self.keys.get(key_a), self.keys.get(key_b)) else {
            return false;
        };

        self.uf.same(*a_index, *b_index)
    }

    pub fn groups(&mut self) -> HashMap<usize, Vec<&String>> {
        let mut groups = HashMap::new();

        for (key, index) in self.keys.iter() {
            let root = self.uf.find(*index);

            groups.entry(root).or_insert_with(Vec::new).push(key);
        }

        groups
    }

    pub fn size(&mut self, key: &str) -> Option<usize> {
        let Some(index) = self.keys.get(key) else {
            return None;
        };

        Some(self.uf.size(*index))
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    // graphが正（実際にMERGEされた辺の記録）で、ufはそこから毎回作り直せる派生データ。
    // 1) 対象の辺をgraphから削除 → 2) ufを初期状態（全員バラバラ）に戻す →
    // 3) 残ったgraphの辺をkeysでインデックスに変換しながらufに再生する
    pub fn unmerge(&mut self, key_a: &str, key_b: &str) {
        self.uf.reset();
        self.graph.remove_edge(key_a, key_b);

        for (key, index) in self.keys.iter() {
            if let Some(neighbors) = self.graph.neighbors(key) {
                for neighbor in neighbors {
                    let neighbor_index = self.keys[neighbor];

                    self.uf.unite(*index, neighbor_index);
                }
            }
        }
    }

    pub fn neighbors(&self, key: &str) -> Option<&Vec<String>> {
        self.graph.neighbors(key)
    }

    pub fn seed(&mut self) {
        let unions = [
            ("apple", "banana"),
            ("banana", "cherry"),
            ("date", "elderberry"),
            ("grape", "honeydew"),
            ("honeydew", "kiwi"),
            ("kiwi", "lemon"),
        ];

        for (a, b) in unions {
            self.unite(a, b);
        }

        self.make_set("fig");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unite_auto_registers_unknown_keys_and_connects_them() {
        let mut ufdb = Ufdb::new();

        let merged = ufdb.unite("a", "b");

        assert!(merged);
        assert_eq!(ufdb.keys.len(), 2);

        let a_index = ufdb.keys["a"];
        let b_index = ufdb.keys["b"];

        assert!(ufdb.uf.same(a_index, b_index));
    }

    #[test]
    fn make_set_registers_isolated_node_in_graph() {
        let mut ufdb = Ufdb::new();

        ufdb.make_set("a");

        assert_eq!(ufdb.graph.neighbors("a"), Some(&vec![]));
    }

    #[test]
    fn unite_registers_edge_in_graph() {
        let mut ufdb = Ufdb::new();

        ufdb.unite("a", "b");

        assert_eq!(ufdb.graph.neighbors("a"), Some(&vec!["b".to_string()]));
        assert_eq!(ufdb.graph.neighbors("b"), Some(&vec!["a".to_string()]));
    }

    #[test]
    fn unite_returns_false_when_already_same_set() {
        let mut ufdb = Ufdb::new();

        assert!(ufdb.unite("a", "b"));
        assert!(!ufdb.unite("a", "b"));
    }

    #[test]
    fn same_returns_true_after_unite() {
        let mut ufdb = Ufdb::new();

        ufdb.unite("a", "b");

        assert!(ufdb.same("a", "b"));
    }

    #[test]
    fn same_returns_false_when_not_united() {
        let mut ufdb = Ufdb::new();

        ufdb.make_set("a");
        ufdb.make_set("b");

        assert!(!ufdb.same("a", "b"));
    }

    #[test]
    fn same_returns_false_for_unregistered_keys_without_registering_them() {
        let mut ufdb = Ufdb::new();

        assert!(!ufdb.same("a", "b"));
        assert_eq!(ufdb.keys.len(), 0);
    }

    #[test]
    fn size_returns_none_for_unregistered_key() {
        let mut ufdb = Ufdb::new();

        assert_eq!(ufdb.size("a"), None);
    }

    #[test]
    fn size_returns_one_for_freshly_registered_key() {
        let mut ufdb = Ufdb::new();

        ufdb.make_set("a");

        assert_eq!(ufdb.size("a"), Some(1));
    }

    #[test]
    fn size_increases_after_unite() {
        let mut ufdb = Ufdb::new();

        ufdb.unite("a", "b");

        assert_eq!(ufdb.size("a"), Some(2));
        assert_eq!(ufdb.size("b"), Some(2));
    }

    #[test]
    fn unmerge_splits_group_by_removing_edge() {
        let mut ufdb = Ufdb::new();

        ufdb.unite("a", "b");
        ufdb.unite("b", "c");

        assert!(ufdb.same("a", "c"));

        ufdb.unmerge("b", "c");

        assert!(ufdb.same("a", "b"));
        assert!(!ufdb.same("b", "c"));
        assert!(!ufdb.same("a", "c"));
    }
}
