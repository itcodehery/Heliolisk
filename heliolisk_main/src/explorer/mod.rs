pub mod widget;

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileNodeType {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub node_type: FileNodeType,
    pub is_expanded: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let node_type = if path.is_dir() {
            FileNodeType::Directory
        } else {
            FileNodeType::File
        };

        Self {
            name,
            path,
            node_type,
            is_expanded: false,
            children: Vec::new(),
        }
    }

    pub fn read_children(&mut self) -> std::io::Result<()> {
        if self.node_type != FileNodeType::Directory {
            return Ok(());
        }

        let mut children = Vec::new();
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let child_path = entry.path();
            let child_name = entry.file_name().to_string_lossy().to_string();

            // Skip common hidden/ignored directories like .git or target
            if child_name == ".git" || child_name == "target" {
                continue;
            }

            children.push(FileNode::new(child_path));
        }

        // Sort: Directories first alphabetically, then files alphabetically
        children.sort_by(|a, b| match (&a.node_type, &b.node_type) {
            (FileNodeType::Directory, FileNodeType::File) => std::cmp::Ordering::Less,
            (FileNodeType::File, FileNodeType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.children = children;
        Ok(())
    }
}

/// State for the File Explorer Sidebar
#[derive(Debug, Clone)]
pub struct FileExplorer {
    pub root: FileNode,
    pub is_visible: bool,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub width: u16,
}

impl FileExplorer {
    pub fn new(root_dir: &Path, width: u16) -> Self {
        let mut root = FileNode::new(root_dir.to_path_buf());
        root.is_expanded = true;
        let _ = root.read_children();

        Self {
            root,
            is_visible: false,
            selected_index: 0,
            scroll_offset: 0,
            width,
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn refresh(&mut self) {
        let _ = self.root.read_children();
        self.clamp_selection();
    }

    /// Flatten visible nodes into an ordered list of (depth, node_reference)
    pub fn flatten_visible(&self) -> Vec<(usize, &FileNode)> {
        let mut result = Vec::new();
        Self::collect_visible(&self.root, 0, &mut result);
        result
    }

    fn collect_visible<'a>(node: &'a FileNode, depth: usize, out: &mut Vec<(usize, &'a FileNode)>) {
        for child in &node.children {
            out.push((depth, child));
            if child.is_expanded && child.node_type == FileNodeType::Directory {
                Self::collect_visible(child, depth + 1, out);
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let count = self.flatten_visible().len();
        if count > 0 && self.selected_index < count - 1 {
            self.selected_index += 1;
        }
    }

    pub fn selected_node_path(&self) -> Option<PathBuf> {
        let visible = self.flatten_visible();
        visible.get(self.selected_index).map(|(_, n)| n.path.clone())
    }

    pub fn toggle_or_open_selected(&mut self) -> Option<PathBuf> {
        let visible = self.flatten_visible();
        let target_path = visible.get(self.selected_index).map(|(_, n)| n.path.clone())?;

        // Find and toggle or return path for opening
        let mut file_to_open = None;
        Self::mutate_node(&mut self.root, &target_path, &mut |node| {
            if node.node_type == FileNodeType::Directory {
                node.is_expanded = !node.is_expanded;
                if node.is_expanded && node.children.is_empty() {
                    let _ = node.read_children();
                }
            } else {
                file_to_open = Some(node.path.clone());
            }
        });

        self.clamp_selection();
        file_to_open
    }

    pub fn collapse_selected(&mut self) {
        let visible = self.flatten_visible();
        if let Some((_, node)) = visible.get(self.selected_index) {
            let path = node.path.clone();
            if node.node_type == FileNodeType::Directory && node.is_expanded {
                Self::mutate_node(&mut self.root, &path, &mut |n| n.is_expanded = false);
            }
        }
        self.clamp_selection();
    }

    fn mutate_node<F>(current: &mut FileNode, target: &Path, f: &mut F) -> bool
    where
        F: FnMut(&mut FileNode),
    {
        if current.path == target {
            f(current);
            return true;
        }
        for child in &mut current.children {
            if Self::mutate_node(child, target, f) {
                return true;
            }
        }
        false
    }

    pub fn clamp_selection(&mut self) {
        let count = self.flatten_visible().len();
        if count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= count {
            self.selected_index = count - 1;
        }
    }

    pub fn update_viewport(&mut self, height: usize) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + height {
            self.scroll_offset = self.selected_index.saturating_sub(height) + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_explorer_tree_and_navigation() {
        let temp_dir = std::env::temp_dir().join("heliolisk_test_explorer");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let sub_dir = temp_dir.join("sub");
        std::fs::create_dir_all(&sub_dir).unwrap();

        let file_a = temp_dir.join("a.txt");
        let file_b = sub_dir.join("b.rs");

        let mut f1 = File::create(&file_a).unwrap();
        writeln!(f1, "hello a").unwrap();
        let mut f2 = File::create(&file_b).unwrap();
        writeln!(f2, "hello b").unwrap();

        let mut explorer = FileExplorer::new(&temp_dir, 30);
        let visible = explorer.flatten_visible();
        // Directories come first: "sub" directory, then "a.txt"
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].1.name, "sub");
        assert_eq!(visible[1].1.name, "a.txt");

        // Move down
        explorer.move_down();
        assert_eq!(explorer.selected_index, 1);
        assert_eq!(explorer.selected_node_path().unwrap(), file_a);

        // Move up
        explorer.move_up();
        assert_eq!(explorer.selected_index, 0);

        // Toggle expand sub dir
        let opened = explorer.toggle_or_open_selected();
        assert!(opened.is_none()); // it was a dir, so opened is None

        // Now sub/b.rs should be visible
        let visible_after = explorer.flatten_visible();
        assert_eq!(visible_after.len(), 3);
        assert_eq!(visible_after[1].1.name, "b.rs");

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

