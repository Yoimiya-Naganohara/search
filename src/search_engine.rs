use crate::data::{Node, PathTree};
use std::{
    ffi::OsStr,
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
    /// * `section` - A `&char` representing to update
    fn generate_index(&mut self, root_dir: PathBuf, section: &char);
    /// Searches for a keyword in the index.
    ///
    /// # Arguments
    ///
    /// * `keyword` - A `String` representing the keyword to search for.
    ///
    /// # Returns
    ///
    /// * `Result<&Vec<PathBuf>, ()>` - A result containing a reference to a vector of `PathBuf` if the search is successful, or an error `()` if the keyword is not found.
    fn search(&self, keyword: &String) -> Result<Vec<PathBuf>, ()>;

    /// Saves the current index to disk.
    fn save_index(&mut self);

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

    fn indexed(&self) -> usize;
}

impl SearchEngine for Search {
    fn new() -> Self {
        Search {
            index: Node::new(),
            search_part: 'C',
        }
    }

    fn search(&self, keyword: &String) -> Result<Vec<PathBuf>, ()> {
        let mut search_results = Vec::new();
        if !keyword.ends_with('?') {
            let node = match self.index.get(&keyword) {
                Some(x) => x,
                None => {
                    return Err(());
                }
            };
            search_results.append(&mut node.val().clone());
            return Ok(search_results);
        }
        let mut keyword = keyword.clone();
        keyword.pop();
        let node = match self.index.get(&keyword) {
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

        traverse_node(node, &mut search_results);
        Ok(search_results)
    }

    fn save_index(&mut self) {
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
        self.index.clear();
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
    fn indexed(&self) -> usize {
        self.index.len()
    }

    fn generate_index(&mut self, root_dir: PathBuf, section: &char) {
        self.index.clear();
        fn traverse_directory(
            index: &mut Node,
            current_dir: &PathBuf,
            extension_node: &mut Node,
            section: &char,
        ) {
            if current_dir.metadata().is_err() || fs::read_dir(&current_dir).is_err() {
                return;
            }

            let entries = fs::read_dir(current_dir).expect("Failed to read directory");
            for entry in entries {
                let entry = entry.expect("Failed to get entry");
                if entry.file_type().unwrap().is_dir() {
                    traverse_directory(index, &entry.path(), extension_node, section);
                } else if entry.file_type().unwrap().is_file() {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_str().unwrap();
                    let path = entry.path();
                    if file_name_str.starts_with(section.clone()) || section == &'*' {
                        if !file_name_str.starts_with('.') {
                            index.insert(file_name_str, path);
                        }
                        let mut file_name = file_name_str.to_string();
                        file_name.remove(0);

                        // generate new node based on file extension

                        let path = entry.path();
                        let mut extension = path
                            .extension()
                            .unwrap_or(OsStr::new("None"))
                            .to_str()
                            .unwrap();
                        let path = entry.path();
                        if extension == "None" {
                            extension = &file_name;
                        }
                        extension_node.insert(extension, path);
                    }
                }
            }
        }
        let mut node = Node::new();
        traverse_directory(&mut self.index, &root_dir, &mut node, section);
        self.index.add_value('.', node);
    }
}

pub struct Search {
    index: Node,
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
        search.generate_index(test_dir, &'*');
        assert!(!search.index.is_empty());
    }

    #[test]
    fn test_search() {
        let mut search = Search::new();
        let test_dir = current_dir().unwrap();
        search.generate_index(test_dir, &'*');
        let keyword = String::from("main.rs");
        let result = search.search(&keyword);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_save_and_load_index() {
        let mut search = Search::new();
        let test_dir = current_dir().unwrap();
        search.generate_index(test_dir, &'*');
        search.save_index();

        let mut new_search = Search::new();
        new_search.load_index('m'); // Assuming 'a' is a valid section
        assert!(!new_search.index.is_empty());
    }
}
