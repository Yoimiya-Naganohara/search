use std::{
    fs::{read_dir, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use regex::Regex;

pub(crate) struct Search {
    indexed_files: Vec<PathBuf>,
    root_dir: PathBuf,
}
#[allow(dead_code)]
pub trait SearchEngine {
    fn new() -> Self;
    fn generate_index(&mut self);
    fn save_index(&self);
    fn load_index(&mut self);
    fn get_index(&self) -> &Vec<PathBuf>;
    fn set_root_dir(&mut self, root_dir: PathBuf);
    fn get_root_dir(&self) -> &PathBuf;
    fn search(&self, key: &String) -> Vec<PathBuf>;
}
impl SearchEngine for Search {
    fn generate_index(&mut self) {
        // clear before new index added to indexed_files
        self.indexed_files.clear();

        fn traverse_index(current_path: &PathBuf, indexed: &mut Vec<PathBuf>) {
            if current_path.metadata().is_err() {
                return;
            }

            match read_dir(current_path) {
                Ok(entries) => {
                    for entry in entries {
                        let entry = match entry {
                            Ok(x) => x,
                            Err(_) => {
                                return;
                            }
                        };

                        // is dir
                        if entry.path().is_dir() {
                            traverse_index(&entry.path(), indexed);
                        } else if entry.path().is_file() {
                            indexed.push(entry.path());
                        }
                    }
                }
                Err(_) => {
                    return;
                }
            }
        }

        traverse_index(&self.root_dir, &mut self.indexed_files);
    }

    fn new() -> Self {
        Search {
            indexed_files: Vec::new(),
            root_dir: PathBuf::from("C:\\"),
        }
    }

    fn save_index(&self) {
        let file = File::create(format!(
            "index {}",
            self.root_dir
                .to_str()
                .unwrap_or_default()
                .replace("\\", "")
                .replace(":", "")
        ))
        .expect("Fail to create file");

        let writer = BufWriter::new(file);
        if let Err(e) = bincode::serialize_into(writer, &self.indexed_files) {
            eprintln!("Failed to serialize index: {}", e);
        }
    }

    fn load_index(&mut self) {
        let file = match File::open(format!(
            "index {}",
            self.root_dir
                .to_str()
                .unwrap_or_default()
                .replace("\\", "")
                .replace(":", "")
        )) {
            Ok(x) => x,
            Err(_) => {
                self.indexed_files = Vec::new();
                return;
            }
        };
        let reader = BufReader::new(file);
        self.indexed_files = match bincode::deserialize_from(reader) {
            Ok(x) => x,
            Err(_) => Vec::new(),
        };
    }

    fn get_index(&self) -> &Vec<PathBuf> {
        &self.indexed_files
    }

    fn set_root_dir(&mut self, root_dir: PathBuf) {
        self.root_dir = root_dir;
    }

    fn get_root_dir(&self) -> &PathBuf {
        &self.root_dir
    }

    fn search(&self, key: &String) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let regex = Regex::new(&key).unwrap_or(Regex::new("None").unwrap());

        for file in &self.indexed_files {
            if regex.is_match(file.file_name().unwrap().to_str().unwrap()) {
                found.push(file.clone());
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let search = Search::new();
        assert!(search.indexed_files.is_empty());
        assert_eq!(search.root_dir, PathBuf::from("C:\\"));
    }

    #[test]
    fn test_set_root_dir() {
        let mut search = Search::new();
        let new_root = PathBuf::from("D:\\");
        search.set_root_dir(new_root.clone());
        assert_eq!(search.root_dir, new_root);
    }

    #[test]
    fn test_generate_index() {
        let mut search = Search::new();
        search.set_root_dir(PathBuf::from("."));
        search.generate_index();
        assert!(!search.indexed_files.is_empty());
    }

    #[test]
    fn test_save_and_load_index() {
        let mut search = Search::new();
        search.set_root_dir(PathBuf::from("."));
        search.generate_index();
        search.save_index();

        let mut new_search = Search::new();
        new_search.set_root_dir(PathBuf::from("."));
        new_search.load_index();
        assert_eq!(search.indexed_files, new_search.indexed_files);
    }

    #[test]
    fn test_get_index() {
        let mut search = Search::new();
        search.set_root_dir(PathBuf::from("."));
        search.generate_index();
        let index = search.get_index();
        assert_eq!(index, &search.indexed_files);
    }
}
