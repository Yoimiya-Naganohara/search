use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

pub(crate) trait PathTree {
    fn new() -> Self;
    fn get(&self, key: &str) -> Option<&Node>;
    fn insert(&mut self, key: &str, value: PathBuf);
    fn delete(&mut self, key: &str);
    fn show_tree(&self);
    fn len(&self) -> usize;
    fn groups(&self) -> &HashMap<char, Node>;
    fn val(&self) -> &Vec<PathBuf>;
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
}

impl PathTree for Node {
    fn new() -> Self {
        Node {
            layer: HashMap::new(),
            path: Vec::new(),
        }
    }

    /// Get value of given key
    fn get(&self, key: &str) -> Option<&Node> {
        let mut current_node = self;
        for ch in key.chars() {
            match current_node.layer.get(&ch) {
                Some(node) => current_node = node,
                None => return None,
            }
        }
        Some(current_node)
    }

    /// Insert { key, value }
    fn insert(&mut self, key: &str, value: PathBuf) {
        let mut current_node = self;
        for ch in key.chars() {
            current_node = current_node.layer.entry(ch).or_insert_with(Node::new);
        }
        current_node.path.push(value);
    }

    /// Delete the value associated with the given key
    fn delete(&mut self, key: &str) {
        fn delete_recursive(node: &mut Node, key: &str, depth: usize) -> bool {
            if depth == key.len() {
                node.path.clear();
                return node.layer.is_empty();
            }
            let ch = key.chars().nth(depth).unwrap();
            if let Some(child) = node.layer.get_mut(&ch) {
                if delete_recursive(child, key, depth + 1) {
                    node.layer.remove(&ch);
                    return node.path.is_empty() && node.layer.is_empty();
                }
            }
            false
        }
        delete_recursive(self, key, 0);
    }

    fn show_tree(&self) {
        println!("{:?}", self.layer);
    }

    fn len(&self) -> usize {
        fn count_nodes(node: &Node) -> usize {
            let mut count = 1; // Count the current node
            for child in node.layer.values() {
                count += count_nodes(child);
            }
            count
        }
        count_nodes(self)
    }

    fn groups(&self) -> &HashMap<char, Node> {
        &self.layer
    }

    fn val(&self) -> &Vec<PathBuf> {
        &self.path
    }

    fn clear(&mut self) {
        self.layer.clear();
        self.path.clear();
    }

    fn is_empty(&self) -> bool {
        self.layer.is_empty() && self.path.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    layer: HashMap<char, Node>,
    path: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let node = Node::new();
        assert!(node.layer.is_empty());
        assert!(node.path.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let mut node = Node::new();
        let path = PathBuf::from("/some/path");
        node.insert("key", path.clone());

        let retrieved_node = node.get("key").unwrap();
        assert_eq!(retrieved_node.path[0], path);
    }

    #[test]
    fn test_delete() {
        let mut node = Node::new();
        let path = PathBuf::from("/some/path");
        node.insert("key", path.clone());

        node.delete("key");
        assert!(node.get("key").is_none());
    }

    #[test]
    fn test_len() {
        let mut node = Node::new();
        node.insert("key1", PathBuf::from("/some/path1"));
        node.insert("key2", PathBuf::from("/some/path2"));

        dbg!(node.clone());
        assert_eq!(node.len(), 6); // root node + 2 key nodes
    }

    #[test]
    fn test_clear() {
        let mut node = Node::new();
        node.insert("key", PathBuf::from("/some/path"));

        node.clear();
        assert!(node.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let node = Node::new();
        assert!(node.is_empty());

        let mut node = Node::new();
        node.insert("key", PathBuf::from("/some/path"));
        assert!(!node.is_empty());
    }

    #[test]
    fn test_show_tree() {
        let mut node = Node::new();
        node.insert("key", PathBuf::from("/some/path"));
        node.show_tree(); // This will print the tree structure
    }

    #[test]
    fn test_groups() {
        let mut node = Node::new();
        node.insert("key", PathBuf::from("/some/path"));

        let groups = node.groups();
        assert!(groups.contains_key(&'k'));
    }

    #[test]
    fn test_val() {
        let mut node = Node::new();
        let path = PathBuf::from("/some/path");
        node.insert("key", path.clone());

        let val = dbg!(match node.get("key") {
            Some(x) => x.val(),
            None => return,
        });
        assert_eq!(val[0], path);
    }
}
