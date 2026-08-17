use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnionFind {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self {
            parent: Vec::new(),
            size: Vec::new(),
        }
    }

    pub fn add(&mut self) -> usize {
        let index = self.parent.len();
        self.parent.push(index);
        self.size.push(1);
        index
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            let root = self.find(self.parent[x]);

            self.parent[x] = root;
        }

        self.parent[x]
    }

    pub fn unite(&mut self, x: usize, y: usize) -> bool {
        let mut x = self.find(x);
        let mut y = self.find(y);

        if x == y {
            return false;
        }

        if self.size[x] < self.size[y] {
            std::mem::swap(&mut x, &mut y);
        }

        self.parent[y] = x;
        self.size[x] += self.size[y];

        true
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn size(&mut self, x: usize) -> usize {
        let root = self.find(x);

        self.size[root]
    }

    pub fn reset(&mut self) {
        for i in 0..self.parent.len() {
            self.parent[i] = i;
            self.size[i] = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_returns_sequential_indices() {
        let mut uf = UnionFind::new();

        let a = uf.add();
        let b = uf.add();
        let c = uf.add();

        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(c, 2);
    }

    #[test]
    fn unite_makes_same_true() {
        let mut uf = UnionFind::new();

        let a = uf.add();
        let b = uf.add();
        let c = uf.add();

        assert!(!uf.same(a, b));
        assert!(!uf.same(a, c));

        uf.unite(a, b);

        assert!(uf.same(a, b));
        assert!(!uf.same(a, c));
    }

    #[test]
    fn size_reflects_set_size_after_unite() {
        let mut uf = UnionFind::new();

        let a = uf.add();
        let b = uf.add();
        let c = uf.add();

        assert_eq!(uf.size(a), 1);

        uf.unite(a, b);

        assert_eq!(uf.size(a), 2);
        assert_eq!(uf.size(b), 2);
        assert_eq!(uf.size(c), 1);
    }
}
