use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use disk_usage_analyzer::report::{
    build_duplicate_files_report, write_duplicate_files_report, DuplicateFilesReport,
};
use disk_usage_analyzer::scanner::{find_duplicate_files, DuplicateGroup, IgnoreMatcher, IgnorePreset};

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-dup-test-{}-{}", name, unique))
}

fn temp_output_path(extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-dup-report-{}.{}", unique, extension))
}

#[test]
fn finds_duplicates_by_content_hash() {
    let root = temp_dir("hash");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("a.txt"), b"same-content").unwrap();
    fs::write(root.join("b.txt"), b"same-content").unwrap();
    fs::write(root.join("c.txt"), b"same-contenu").unwrap();

    let groups = find_duplicate_files(&root, &IgnoreMatcher::default(), 1).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].files.len(), 2);
    assert_eq!(groups[0].size, 12);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn duplicate_finder_respects_ignore_rules() {
    let root = temp_dir("ignore");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("node_modules")).unwrap();
    fs::write(root.join("src").join("keep1.txt"), b"same-content").unwrap();
    fs::write(root.join("src").join("keep2.txt"), b"same-content").unwrap();
    fs::write(root.join("node_modules").join("skip.txt"), b"same-content").unwrap();

    let matcher = IgnoreMatcher::from_inputs(&[], &[IgnorePreset::Dependencies]);
    let groups = find_duplicate_files(&root, &matcher, 1).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].files.len(), 2);
    assert!(groups[0].files.iter().all(|path| !path.contains("node_modules")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn builds_duplicate_report_summary() {
    let groups = vec![DuplicateGroup {
        size: 100,
        wasted_bytes: 200,
        files: vec![
            "E:/repo/a.bin".to_string(),
            "E:/repo/b.bin".to_string(),
            "E:/repo/c.bin".to_string(),
        ],
    }];

    let report = build_duplicate_files_report("E:/repo", groups.clone());

    assert_eq!(report.scanned_path, "E:/repo");
    assert_eq!(report.duplicate_group_count, 1);
    assert_eq!(report.duplicate_file_count, 3);
    assert_eq!(report.reclaimable_bytes, 200);
    assert_eq!(report.groups, groups);
}

#[test]
fn writes_duplicate_report_markdown() {
    let output_path = temp_output_path("md");
    let report = DuplicateFilesReport {
        scanned_path: "E:/repo".to_string(),
        generated_at: "2026-05-21T13:00:00Z".to_string(),
        duplicate_group_count: 1,
        duplicate_file_count: 2,
        reclaimable_bytes: 500,
        groups: vec![DuplicateGroup {
            size: 500,
            wasted_bytes: 500,
            files: vec!["E:/repo/a.bin".to_string(), "E:/repo/b.bin".to_string()],
        }],
    };

    write_duplicate_files_report(&output_path, &report).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Duplicate Files Report"));
    assert!(content.contains("## Summary"));
    assert!(content.contains("## Duplicate Groups"));

    let _ = fs::remove_file(output_path);
}
