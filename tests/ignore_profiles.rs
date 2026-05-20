use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use disk_usage_analyzer::app::Config;
use disk_usage_analyzer::model::FileTree;
use crossbeam_channel::unbounded;
use disk_usage_analyzer::scanner::{IgnoreMatcher, IgnorePreset, ParallelScanner, ScanMessage};

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-ignore-test-{}-{}", name, unique))
}

#[test]
fn preset_matches_common_build_outputs() {
    let matcher = IgnoreMatcher::from_inputs(&[], &[IgnorePreset::Build]);

    assert!(matcher.matches(PathBuf::from("E:/repo/target/debug/app.exe").as_path()));
    assert!(matcher.matches(PathBuf::from("E:/repo/.next/server/app.js").as_path()));
    assert!(!matcher.matches(PathBuf::from("E:/repo/src/main.rs").as_path()));
}

#[test]
fn explicit_ignore_patterns_match_segment_and_nested_path() {
    let matcher = IgnoreMatcher::from_inputs(
        &["node_modules".to_string(), "dist/assets".to_string()],
        &[],
    );

    assert!(matcher.matches(PathBuf::from("E:/repo/node_modules/react/index.js").as_path()));
    assert!(matcher.matches(PathBuf::from("E:/repo/dist/assets/logo.svg").as_path()));
    assert!(!matcher.matches(PathBuf::from("E:/repo/dist/scripts/app.js").as_path()));
}

#[test]
fn scanner_skips_entries_from_ignore_matcher() {
    let root = temp_dir("scan");
    let keep_dir = root.join("src");
    let ignored_dir = root.join("target");

    fs::create_dir_all(&keep_dir).unwrap();
    fs::create_dir_all(&ignored_dir).unwrap();
    fs::write(keep_dir.join("keep.txt"), "keep").unwrap();
    fs::write(ignored_dir.join("ignored.bin"), "ignored").unwrap();

    let matcher = IgnoreMatcher::from_inputs(&[], &[IgnorePreset::Build]);
    let scanner = ParallelScanner::new().with_ignore_matcher(matcher);

    let (tx, rx) = unbounded();
    scanner
        .scan(root.clone(), tx, Arc::new(AtomicBool::new(false)))
        .unwrap();

    let mut seen_paths = Vec::new();
    while let Ok(message) = rx.try_recv() {
        if let ScanMessage::Entry(entry) = message {
            seen_paths.push(entry.path);
        }
    }

    assert!(seen_paths.iter().any(|path| path.ends_with("keep.txt")));
    assert!(!seen_paths.iter().any(|path| path.ends_with("ignored.bin")));
    assert!(!seen_paths.iter().any(|path| path.ends_with("target")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn config_builds_ignore_matcher_from_patterns_and_presets() {
    let config = Config::new(PathBuf::from("E:/repo"))
        .with_ignore_patterns(vec!["custom-cache".to_string()])
        .with_ignore_presets(vec![IgnorePreset::Dependencies]);

    let matcher = config.ignore_matcher();

    assert!(matcher.matches(PathBuf::from("E:/repo/node_modules/react").as_path()));
    assert!(matcher.matches(PathBuf::from("E:/repo/custom-cache/index.bin").as_path()));
    assert!(!matcher.matches(PathBuf::from("E:/repo/src/lib.rs").as_path()));
}

#[test]
fn file_tree_population_respects_ignore_matcher() {
    let root = temp_dir("tree");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("node_modules")).unwrap();
    fs::write(root.join("src").join("keep.rs"), "fn main() {}").unwrap();
    fs::write(root.join("node_modules").join("ignored.js"), "module.exports = {}").unwrap();

    let matcher = IgnoreMatcher::from_inputs(&[], &[IgnorePreset::Dependencies]);
    let mut tree = FileTree::new();
    let root_id = tree.set_root(&root);

    assert!(tree.populate_children_from_fs_with_filter(root_id, &matcher));

    let child_paths: Vec<_> = tree
        .get_children(root_id)
        .into_iter()
        .filter_map(|(id, _)| tree.get_path(id))
        .collect();

    assert!(child_paths.iter().any(|path| path.ends_with("src")));
    assert!(!child_paths.iter().any(|path| path.ends_with("node_modules")));

    let _ = fs::remove_dir_all(root);
}
