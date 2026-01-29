use std::path::Path;
use std::io;

pub struct DeleteAction;

impl DeleteAction {
    pub fn delete_path(path: &Path) -> io::Result<()> {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }

    pub fn delete_paths(paths: &[&Path]) -> Vec<(String, io::Result<()>)> {
        paths
            .iter()
            .map(|path| {
                let result = Self::delete_path(path);
                (path.display().to_string(), result)
            })
            .collect()
    }

    pub fn can_delete(path: &Path) -> bool {
        // Check if we have write permission to parent directory
        if let Some(parent) = path.parent() {
            if let Ok(metadata) = parent.metadata() {
                return !metadata.permissions().readonly();
            }
        }
        false
    }
}
