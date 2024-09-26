use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
/// A trait representing a tree structure where each node is associated with a path.
pub(crate) trait PathTree {
    /// Creates a new instance of the tree.
    fn new() -> Self;

    /// Retrieves the node associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to search for.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the node if found, or `None` if not found.
    fn get(&self, key: &str) -> Option<&Node>;

    /// Inserts a key-value pair into the tree.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key.
    /// * `value` - A `PathBuf` that holds the value to be inserted.
    fn insert(&mut self, key: &str, value: PathBuf);

    /// Deletes the value associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to delete.
    fn delete(&mut self, key: &str);

    /// Displays the tree structure.
    fn show_tree(&self);

    /// Returns the number of nodes in the tree.
    ///
    /// # Returns
    ///
    /// A `usize` representing the number of nodes.
    fn len(&self) -> usize;

    /// Returns a reference to the children of the current node.
    ///
    /// # Returns
    ///
    /// A reference to a `HashMap` containing the children nodes.
    fn groups(&self) -> &HashMap<char, Node>;

    /// Returns a reference to the paths stored in the current node.
    ///
    /// # Returns
    ///
    /// A reference to a `Vec` containing the paths.
    fn val(&self) -> &Vec<PathBuf>;

    /// Clears all nodes and paths in the tree.
    fn clear(&mut self);

    /// Checks if the tree is empty.
    ///
    /// # Returns
    ///
    /// A `bool` indicating whether the tree is empty.
    fn is_empty(&self) -> bool;
}

impl PathTree for Node {
    fn new() -> Self {
        Node {
            children: HashMap::new(),
            paths: Vec::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&Node> {
        let mut current_node = self;
        for character in key.chars() {
            match current_node.children.get(&character) {
                Some(node) => current_node = node,
                None => return None,
            }
        }
        Some(current_node)
    }

    fn insert(&mut self, key: &str, value: PathBuf) {
        let mut current_node = self;
        for character in key.chars() {
            current_node = current_node
                .children
                .entry(character)
                .or_insert_with(Node::new);
        }
        current_node.paths.push(value);
    }

    fn delete(&mut self, key: &str) {
        fn delete_recursive(node: &mut Node, key: &str, depth: usize) -> bool {
            if depth == key.len() {
                node.paths.clear();
                return node.children.is_empty();
            }
            let character = key.chars().nth(depth).unwrap();
            if let Some(child_node) = node.children.get_mut(&character) {
                if delete_recursive(child_node, key, depth + 1) {
                    node.children.remove(&character);
                    return node.paths.is_empty() && node.children.is_empty();
                }
            }
            false
        }
        delete_recursive(self, key, 0);
    }

    fn show_tree(&self) {
        println!("{:?}", self.children);
    }

    fn len(&self) -> usize {
        fn count_nodes(node: &Node) -> usize {
            let mut count = 1; // Count the current node
            for child_node in node.children.values() {
                count += count_nodes(child_node);
            }
            count
        }
        count_nodes(self)
    }

    fn groups(&self) -> &HashMap<char, Node> {
        &self.children
    }

    fn val(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    fn clear(&mut self) {
        self.children.clear();
        self.paths.clear();
    }

    fn is_empty(&self) -> bool {
        self.children.is_empty() && self.paths.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    children: HashMap<char, Node>,
    paths: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let node = Node::new();
        assert!(node.children.is_empty());
        assert!(node.paths.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let mut root_node = Node::new();
        let path = PathBuf::from("/some/path");
        root_node.insert("key", path.clone());

        let retrieved_node = root_node.get("key").unwrap();
        assert_eq!(retrieved_node.paths[0], path);
    }

    #[test]
    fn test_delete() {
        let mut root_node = Node::new();
        let path = PathBuf::from("/some/path");
        root_node.insert("key", path.clone());

        root_node.delete("key");
        assert!(root_node.get("key").is_none());
    }

    #[test]
    fn test_len() {
        let mut root_node = Node::new();
        root_node.insert("key1", PathBuf::from("/some/path1"));
        root_node.insert("key2", PathBuf::from("/some/path2"));

        dbg!(root_node.clone());
        assert_eq!(root_node.len(), 6); // root node + 2 key nodes
    }

    #[test]
    fn test_clear() {
        let mut root_node = Node::new();
        root_node.insert("key", PathBuf::from("/some/path"));

        root_node.clear();
        assert!(root_node.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let root_node = Node::new();
        assert!(root_node.is_empty());

        let mut root_node = Node::new();
        root_node.insert("key", PathBuf::from("/some/path"));
        assert!(!root_node.is_empty());
    }

    #[test]
    fn test_show_tree() {
        let mut root_node = Node::new();
        root_node.insert("key", PathBuf::from("/some/path"));
        root_node.show_tree(); // This will print the tree structure
    }

    #[test]
    fn test_groups() {
        let mut root_node = Node::new();
        root_node.insert("key", PathBuf::from("/some/path"));

        let groups = root_node.groups();
        assert!(groups.contains_key(&'k'));
    }

    #[test]
    fn test_val() {
        let mut root_node = Node::new();
        let path = PathBuf::from("/some/path");
        root_node.insert("key", path.clone());

        let val = dbg!(match root_node.get("key") {
            Some(node) => node.val(),
            None => return,
        });
        assert_eq!(val[0], path);
    }
}
