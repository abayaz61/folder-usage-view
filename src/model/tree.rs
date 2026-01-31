use slotmap::SlotMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::node::{FileCategory, NodeId, TreeNode};
use super::statistics::TreeStatistics;
use crate::app::SortMode;

pub struct FileTree {
    arena: SlotMap<NodeId, TreeNode>,
    root: Option<NodeId>,
    path_index: HashMap<PathBuf, NodeId>,
    pub statistics: TreeStatistics,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            arena: SlotMap::with_key(),
            root: None,
            path_index: HashMap::new(),
            statistics: TreeStatistics::new(),
        }
    }

    pub fn set_root(&mut self, path: &Path) -> NodeId {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let node = TreeNode::new_directory(name, None, 0);
        let id = self.arena.insert(node);
        self.root = Some(id);
        self.path_index.insert(path.to_path_buf(), id);
        self.statistics.add_directory();
        id
    }

    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn get(&self, id: NodeId) -> Option<&TreeNode> {
        self.arena.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut TreeNode> {
        self.arena.get_mut(id)
    }

    pub fn get_by_path(&self, path: &Path) -> Option<NodeId> {
        self.path_index.get(path).copied()
    }

    pub fn insert_entry(
        &mut self,
        path: &Path,
        parent_path: &Path,
        size: u64,
        is_dir: bool,
    ) -> Option<NodeId> {
        let parent_id = self.path_index.get(parent_path).copied()?;

        // Check if the entry already exists (e.g., from populate_children_from_fs)
        if let Some(&existing_id) = self.path_index.get(path) {
            // Update existing node with size info
            if !is_dir {
                if let Some(node) = self.arena.get_mut(existing_id) {
                    let old_size = node.size;
                    let size_diff = size as i64 - old_size as i64;
                    node.size = size;

                    // Update statistics
                    let category = node.category();
                    let name = node.name.clone();
                    self.statistics.add_file(category, size, existing_id, name);

                    // Propagate size difference up to ancestors
                    if size_diff > 0 {
                        self.propagate_size(parent_id, size_diff as u64);
                    }
                }
            }
            return Some(existing_id);
        }

        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let parent_depth = self.arena.get(parent_id).map(|n| n.depth).unwrap_or(0);
        let depth = parent_depth + 1;

        let node = if is_dir {
            self.statistics.add_directory();
            TreeNode::new_directory(name.clone(), Some(parent_id), depth)
        } else {
            let ext = path
                .extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let category = FileCategory::from_extension(&ext);
            let node = TreeNode::new_file(name.clone(), size, Some(parent_id), depth, category);

            let id = self.arena.insert(node);
            self.path_index.insert(path.to_path_buf(), id);

            // Add to parent
            if let Some(parent) = self.arena.get_mut(parent_id) {
                parent.children.push(id);
            }

            // Update statistics
            self.statistics.add_file(category, size, id, name);

            // Propagate size up to ancestors
            self.propagate_size(parent_id, size);

            return Some(id);
        };

        let id = self.arena.insert(node);
        self.path_index.insert(path.to_path_buf(), id);

        // Add to parent
        if let Some(parent) = self.arena.get_mut(parent_id) {
            parent.children.push(id);
        }

        Some(id)
    }

    fn propagate_size(&mut self, mut current: NodeId, size: u64) {
        loop {
            if let Some(node) = self.arena.get_mut(current) {
                node.size += size;
                node.item_count += 1;
                if let Some(parent) = node.parent {
                    current = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn get_children(&self, id: NodeId) -> Vec<(NodeId, &TreeNode)> {
        self.arena
            .get(id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|&child_id| self.arena.get(child_id).map(|n| (child_id, n)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_children_sorted_by_size(&self, id: NodeId) -> Vec<(NodeId, &TreeNode)> {
        let mut children = self.get_children(id);
        children.sort_by(|a, b| b.1.size.cmp(&a.1.size));
        children
    }

    pub fn get_children_sorted(&self, id: NodeId, sort_mode: SortMode) -> Vec<(NodeId, &TreeNode)> {
        let mut children = self.get_children(id);
        match sort_mode {
            SortMode::Size => {
                children.sort_by(|a, b| b.1.size.cmp(&a.1.size));
            }
            SortMode::Name => {
                children.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));
            }
            SortMode::Type => {
                children.sort_by(|a, b| {
                    // Directories first
                    let a_is_dir = a.1.is_dir();
                    let b_is_dir = b.1.is_dir();
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => {
                            // Same type - sort by extension then name
                            let a_ext = a.1.name.rsplit('.').next().unwrap_or("").to_lowercase();
                            let b_ext = b.1.name.rsplit('.').next().unwrap_or("").to_lowercase();
                            match a_ext.cmp(&b_ext) {
                                std::cmp::Ordering::Equal => {
                                    a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())
                                }
                                other => other,
                            }
                        }
                    }
                });
            }
            SortMode::Date => {
                children.sort_by(|a, b| {
                    let a_time = a.1.metadata.as_ref().and_then(|m| m.modified);
                    let b_time = b.1.metadata.as_ref().and_then(|m| m.modified);
                    match (a_time, b_time) {
                        (Some(a), Some(b)) => b.cmp(&a), // Newest first
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()),
                    }
                });
            }
        }
        children
    }

    pub fn toggle_selection(&mut self, id: NodeId) {
        if let Some(node) = self.arena.get_mut(id) {
            node.selected = !node.selected;
        }
    }

    pub fn get_selected(&self) -> Vec<NodeId> {
        self.arena
            .iter()
            .filter(|(_, node)| node.selected)
            .map(|(id, _)| id)
            .collect()
    }

    pub fn clear_all_selections(&mut self) {
        for (_, node) in self.arena.iter_mut() {
            node.selected = false;
        }
    }

    pub fn get_path(&self, id: NodeId) -> Option<PathBuf> {
        self.path_index
            .iter()
            .find(|(_, &node_id)| node_id == id)
            .map(|(path, _)| path.clone())
    }

    pub fn remove(&mut self, id: NodeId) -> bool {
        if let Some(node) = self.arena.remove(id) {
            // Remove from parent's children
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.arena.get_mut(parent_id) {
                    parent.children.retain(|child| *child != id);
                }
            }

            // Remove from path index
            self.path_index.retain(|_, &mut node_id| node_id != id);

            // Recursively remove children
            for child_id in node.children.iter() {
                self.remove(*child_id);
            }

            true
        } else {
            false
        }
    }

    pub fn total_nodes(&self) -> usize {
        self.arena.len()
    }

    /// Check if a node's children have been populated
    pub fn has_children_populated(&self, id: NodeId) -> bool {
        self.arena
            .get(id)
            .map(|node| !node.children.is_empty() || node.children_populated)
            .unwrap_or(false)
    }

    /// Mark a node as having its children populated (even if empty)
    pub fn mark_children_populated(&mut self, id: NodeId) {
        if let Some(node) = self.arena.get_mut(id) {
            node.children_populated = true;
        }
    }

    /// Populate a directory's children by reading the filesystem directly
    /// This is used when navigating into a directory during scanning
    pub fn populate_children_from_fs(&mut self, id: NodeId) -> bool {
        // Get the path for this node
        let path = match self.get_path(id) {
            Some(p) => p,
            None => return false,
        };

        // Check if already populated
        if self.has_children_populated(id) {
            return true;
        }

        // Read directory contents
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => {
                self.mark_children_populated(id);
                return false;
            }
        };

        // Get parent depth
        let parent_depth = self.arena.get(id).map(|n| n.depth).unwrap_or(0);

        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            let name = entry_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Skip if already exists
            if self.path_index.contains_key(&entry_path) {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let depth = parent_depth + 1;

            let node = if is_dir {
                TreeNode::new_directory(name.clone(), Some(id), depth)
            } else {
                // Get file size
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let ext = entry_path
                    .extension()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let category = FileCategory::from_extension(&ext);
                TreeNode::new_file(name.clone(), size, Some(id), depth, category)
            };

            let child_id = self.arena.insert(node);
            self.path_index.insert(entry_path, child_id);

            // Add to parent
            if let Some(parent) = self.arena.get_mut(id) {
                parent.children.push(child_id);
            }
        }

        self.mark_children_populated(id);
        true
    }
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}
