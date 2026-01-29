use slotmap::SlotMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::node::{FileCategory, NodeId, TreeNode};
use super::statistics::TreeStatistics;

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
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}
