use crate::data::{Node, PathTree};
use std::{
    fs::{self, create_dir, exists, File},
    path::PathBuf,
};

/// A trait that defines the basic operations for a search engine.
pub(crate) trait SearchEngine {
    /// Creates a new instance of the search engine.
    fn new() -> Self;

    /// Generates an index from the given root directory.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - A `PathBuf` representing the root directory to index.
    fn generate_index(&mut self, root_dir: PathBuf);

    /// Searches for a keyword in the index.
    ///
    /// # Arguments
    ///
    /// * `keyword` - A `String` representing the keyword to search for.
    ///
    /// # Returns
    ///
    /// * `Result<&Vec<PathBuf>, ()>` - A result containing a reference to a vector of `PathBuf` if the search is successful, or an error `()` if the keyword is not found.
    fn search(&mut self, keyword: &String) -> Result<&Vec<PathBuf>, ()>;

    /// Saves the current index to disk.
    fn save_index(&self);

    /// Loads an index from disk for a given section.
    ///
    /// # Arguments
    ///
    /// * `section` - A `char` representing the section to load.
    fn load_index(&mut self, section: char);

    /// Loads an index from disk for a given section.
    ///
    /// # Arguments
    ///
    /// * `section` - A `char`
    /// Sets the part of the index to be used.    ///
    /// # Arguments
    ///
    /// * `part` - A `char` representing the part of the index to set.
    fn set_part(&mut self, part: char);
}

impl SearchEngine for Search {
    fn new() -> Self {
        Search {
            index: Node::new(),
            search_results: Vec::new(),
            search_part: 'C',
        }
    }

    fn generate_index(&mut self, root_dir: PathBuf) {
        self.index.clear();

        fn traverse_directory(index: &mut Node, current_dir: &PathBuf) {
            if current_dir.metadata().is_err() || fs::read_dir(&current_dir).is_err() {
                return;
            }

            let entries = fs::read_dir(current_dir).expect("Failed to read directory");
            for entry in entries {
                let entry = entry.expect("Failed to get entry");
                if entry.file_type().unwrap().is_dir() {
                    traverse_directory(index, &entry.path());
                } else if entry.file_type().unwrap().is_file() {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_str().unwrap();
                    let path = entry.path();
                    index.insert(file_name_str, path);
                }
            }
        }

        traverse_directory(&mut self.index, &root_dir);
    }

    fn search(&mut self, keyword: &String) -> Result<&Vec<PathBuf>, ()> {
        self.search_results.clear();
        let node = match self.index.get(keyword) {
            Some(x) => x,
            None => {
                return Err(());
            }
        };

        fn traverse_node(node: &Node, search_results: &mut Vec<PathBuf>) {
            let mut path = node.val().clone();
            search_results.append(&mut path);
            if node.groups().len() == 0 {
                return;
            }
            for (_, sub_node) in node.groups() {
                traverse_node(sub_node, search_results);
            }
        }

        traverse_node(node, &mut self.search_results);
        Ok(&self.search_results)
    }

    fn save_index(&self) {
        if self.index.is_empty() {
            return;
        }
        if !exists(format!("index{}", self.search_part)).unwrap_or(false) {
            if let Err(e) = create_dir(format!("index{}", self.search_part)) {
                eprintln!("Failed to create directory: {}", e);
            }
        }
        for (ch, node) in self.index.groups() {
            let file = File::create(format!(
                "index{}/data-{}{}",
                self.search_part,
                ch,
                ch.is_uppercase()
            ))
            .expect("Failed to create file");
            let mut writer = std::io::BufWriter::new(file);
            bincode::serialize_into(&mut writer, node).expect("Failed to serialize data");
        }
    }

    fn load_index(&mut self, section: char) {
        let file = match File::open(format!(
            "index{}/data-{}{}",
            self.search_part,
            section,
            section.is_uppercase()
        )) {
            Ok(x) => x,
            Err(_) => {
                self.index = Node::new();
                return;
            }
        };
        let mut reader = std::io::BufReader::new(file);
        self.index = bincode::deserialize_from(&mut reader).expect("Failed to deserialize data");
    }
    fn set_part(&mut self, part: char) {
        self.search_part = part;
    }
}

pub struct Search {
    index: Node,
    search_results: Vec<PathBuf>,
    search_part: char,
}
#[cfg(test)]
mod tests {
    use std::env;

    use env::current_dir;

    use super::*;

    #[test]
    fn test_generate_index() {
        let mut search = Search::new();
        let test_dir = current_dir().unwrap();
        search.generate_index(test_dir);
        assert!(!search.index.is_empty());
    }

    #[test]
    fn test_search() {
        let mut search = Search::new();
        let test_dir = current_dir().unwrap();
        search.generate_index(test_dir);
        let keyword = String::from("main.rs");
        let result = search.search(&keyword);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_save_and_load_index() {
        let mut search = Search::new();
        let test_dir = current_dir().unwrap();
        search.generate_index(test_dir);
        search.save_index();

        let mut new_search = Search::new();
        new_search.load_index('m'); // Assuming 'a' is a valid section
        assert!(!new_search.index.is_empty());
    }
}
