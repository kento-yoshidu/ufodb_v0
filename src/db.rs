use std::collections::HashMap;

use crate::Ufdb;

#[derive(Debug)]
pub struct Db {
    current_db: String,
    db: HashMap<String, Ufdb>,
}

#[derive(Debug)]
pub enum UseDbResult {
    Switched,
    NotFound,
    Corrupted(std::io::Error),
}

impl Db {
    pub fn new() -> Self {
        let ufdb = match crate::storage::load("ufdb") {
            Ok(Some(ufdb)) => ufdb,
            Ok(None) => Ufdb::new(),
            Err(e) => panic!("起動時のデータロードに失敗しました: {e}"),
        };

        Self {
            current_db: "ufdb".to_string(),
            db: HashMap::from([("ufdb".to_string(), ufdb)]),
        }
    }

    pub fn current(&mut self) -> &mut Ufdb {
        self.db.get_mut(&self.current_db).unwrap()
    }

    pub fn current_name(&self) -> &str {
        &self.current_db
    }

    pub fn create_db(&mut self, name: &str) -> bool {
        if !self.db.contains_key(name) {
            self.db.insert(name.to_string(), Ufdb::new());
            self.current_db = name.to_string();
            true
        } else {
            self.current_db = name.to_string();
            false
        }
    }

    pub fn use_db(&mut self, name: &str) -> UseDbResult {
        // メモリー上にデータがあるか
        if self.db.contains_key(name) {
            self.current_db = name.to_string();

            return UseDbResult::Switched;
        }

        match crate::storage::load(name) {
            Ok(Some(ufdb)) => {
                self.db.insert(name.to_string(), ufdb);
                self.current_db = name.to_string();
                UseDbResult::Switched
            },
            Ok(None) => UseDbResult::NotFound,
            Err(e) => UseDbResult::Corrupted(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_on_default_db() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let mut db = Db::new();

        assert_eq!(db.current_db, "ufdb");
        assert!(db.current().is_empty());
    }

    #[test]
    fn create_db_new_returns_true_and_switches() {
        let mut db = Db::new();

        assert!(db.create_db("foo"));
        assert_eq!(db.current_db, "foo");
    }

    #[test]
    fn create_db_existing_returns_false_but_still_switches() {
        let mut db = Db::new();

        db.create_db("foo");
        db.use_db("ufdb");

        assert!(!db.create_db("foo"));
        assert_eq!(db.current_db, "foo");
    }

    #[test]
    fn use_db_existing_switches_and_returns_true() {
        let mut db = Db::new();

        db.create_db("foo");
        db.use_db("ufdb");

        assert!(matches!(db.use_db("foo"), UseDbResult::Switched));
        assert_eq!(db.current_db, "foo");
    }

    #[test]
    fn use_db_missing_returns_false_without_switching() {
        let mut db = Db::new();

        assert!(matches!(db.use_db("nope"), UseDbResult::NotFound));
        assert_eq!(db.current_db, "ufdb");
    }

    #[test]
    fn databases_are_independent() {
        let mut db = Db::new();

        db.current().make_set("a");

        db.create_db("other");
        assert!(db.current().is_empty());
        assert_eq!(db.current().size("a"), None);

        db.use_db("ufdb");
        assert_eq!(db.current().size("a"), Some(1));
    }
}
