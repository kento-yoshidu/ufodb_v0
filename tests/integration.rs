//! ライブラリ API（`Db` / `Ufdb`）を外から呼ぶ形のシナリオテスト。
//! 個々のモジュールの単体テストは `src/*.rs` 側の `mod tests` にある。

use ufodb_v0::db::Db;

#[test]
fn merge_same_size_groups_end_to_end() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let mut db = Db::new(temp_dir.path());
    let ufdb = db.current();

    ufdb.make_set("apple");
    ufdb.unite("apple", "banana");
    ufdb.unite("banana", "cherry");
    ufdb.unite("date", "elderberry");

    assert!(ufdb.same("apple", "cherry"));
    assert!(!ufdb.same("apple", "date"));
    assert_eq!(ufdb.size("apple"), Some(3));
    assert_eq!(ufdb.size("date"), Some(2));

    assert_eq!(ufdb.groups().len(), 2);
}

#[test]
fn unmerge_rebuilds_connectivity_from_remaining_edges() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let mut db = Db::new(temp_dir.path());
    let ufdb = db.current();

    ufdb.unite("a", "b");
    ufdb.unite("b", "c");
    ufdb.unite("c", "d");

    ufdb.unmerge("b", "c");

    assert!(ufdb.same("a", "b"));
    assert!(ufdb.same("c", "d"));
    assert!(!ufdb.same("b", "c"));
    assert_eq!(ufdb.groups().len(), 2);
}

#[test]
fn databases_do_not_leak_across_use() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let mut db = Db::new(temp_dir.path());

    db.current().unite("a", "b");

    db.create_db("second");
    db.current().unite("x", "y");
    assert_eq!(db.current().size("a"), None);

    db.use_db("ufdb", temp_dir.path());
    assert!(db.current().same("a", "b"));
    assert_eq!(db.current().size("x"), None);
}
