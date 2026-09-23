use std::{collections::HashMap, path::Path};

use crate::Ufdb;

#[derive(Debug)]
pub struct Db {
    db: HashMap<String, Ufdb>,
}

#[derive(Debug)]
pub enum LoadError {
    NotFound,
    Corrupted(std::io::Error),
}

impl Db {
    pub fn new(db_name: &str, dir: &Path) -> Result<Self, LoadError> {
        let ufdb = match crate::storage::load(db_name, dir) {
            Ok(Some(ufdb)) => ufdb,
            Ok(None) if db_name == "ufdb" => Ufdb::new(),
            Ok(None) => return Err(LoadError::NotFound),
            Err(e) => return Err(LoadError::Corrupted(e)),
        };

        Ok(Self {
            db: HashMap::from([(db_name.to_string(), ufdb)]),
        })
    }

    pub fn create_db(&mut self, name: &str) -> bool {
        if !self.db.contains_key(name) {
            self.db.insert(name.to_string(), Ufdb::new());
            true
        } else {
            false
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Ufdb> {
        self.db.get_mut(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_default_db_when_missing() {
        let temp_dir = tempfile::tempdir().unwrap();

        let mut db = Db::new("ufdb", temp_dir.path()).unwrap();

        assert!(db.get_mut("ufdb").unwrap().is_empty());
    }

    #[test]
    fn new_rejects_missing_non_default_db() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = Db::new("foo", temp_dir.path());

        assert!(matches!(result, Err(LoadError::NotFound)));
    }

    #[test]
    fn create_db_new_returns_true() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db = Db::new("ufdb", temp_dir.path()).unwrap();

        assert!(db.create_db("foo"));
        assert!(db.get_mut("foo").unwrap().is_empty());
    }

    #[test]
    fn create_db_existing_returns_false() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db = Db::new("ufdb", temp_dir.path()).unwrap();

        db.create_db("foo");

        assert!(!db.create_db("foo"));
    }

    #[test]
    fn databases_are_independent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db = Db::new("ufdb", temp_dir.path()).unwrap();

        db.get_mut("ufdb").unwrap().make_set("a");

        db.create_db("other");

        assert!(db.get_mut("other").unwrap().is_empty());
        assert_eq!(db.get_mut("other").unwrap().size("a"), None);
        assert_eq!(db.get_mut("ufdb").unwrap().size("a"), Some(1));
    }
}
