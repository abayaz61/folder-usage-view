use std::path::PathBuf;

use disk_usage_analyzer::model::FileTree;

#[test]
fn file_tree_returns_paths_for_inserted_nodes() {
    let root = PathBuf::from("E:/repo");
    let src = root.join("src");
    let main = src.join("main.rs");

    let mut tree = FileTree::new();
    tree.set_root(&root);
    let src_id = tree.insert_entry(&src, &root, 0, true).unwrap();
    let main_id = tree.insert_entry(&main, &src, 42, false).unwrap();

    assert_eq!(tree.get_path(src_id), Some(src));
    assert_eq!(tree.get_path(main_id), Some(main));
}

#[test]
fn file_tree_removes_path_indexes_for_deleted_subtrees() {
    let root = PathBuf::from("E:/repo");
    let src = root.join("src");
    let main = src.join("main.rs");

    let mut tree = FileTree::new();
    tree.set_root(&root);
    let src_id = tree.insert_entry(&src, &root, 0, true).unwrap();
    let main_id = tree.insert_entry(&main, &src, 42, false).unwrap();

    assert!(tree.remove(src_id));
    assert_eq!(tree.get_path(src_id), None);
    assert_eq!(tree.get_path(main_id), None);
    assert_eq!(tree.get_by_path(&src), None);
    assert_eq!(tree.get_by_path(&main), None);
}
