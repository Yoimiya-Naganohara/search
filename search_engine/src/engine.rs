use std::{
    collections::HashMap,
    fs::{self, read_dir, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::SystemTime,
};

use regex::Regex;

#[derive(Clone)]
pub(crate) struct Search {
    indexed_files: Vec<PathBuf>,
    search_results: Vec<(PathBuf, String)>,
    root_dir: PathBuf,
    last_modify_time: SystemTime,
    search_results_limit: usize,
    partition: HashMap<char, usize>,
}

#[allow(dead_code)]
pub trait SearchEngine {
    fn clear_index_files(&mut self);
    fn generate_index(&mut self);
    fn get_index(&self) -> &Vec<PathBuf>;
    fn get_results(&self) -> &Vec<(PathBuf, String)>;
    fn get_root_dir(&self) -> &PathBuf;
    fn get_partition(&self) -> &HashMap<char, usize>;
    fn is_index_modified(&mut self) -> bool;
    fn len(&self) -> usize;
    fn load_index(&mut self);
    fn new() -> Self;
    fn reset_search_results(&mut self);
    fn save_index(&self);
    fn save_partition(&self);
    fn load_partition(&mut self);
    fn search(&mut self, key: &str);
    fn search_regex(&mut self, key: &str);
    fn set_root_dir(&mut self, root_dir: PathBuf);
    fn set_search_results_limit(&mut self, limit: usize);
}

impl SearchEngine for Search {
    fn new() -> Self {
        Search {
            indexed_files: Vec::new(),
            root_dir: PathBuf::from("C:\\"),
            search_results: Vec::new(),
            last_modify_time: SystemTime::now(),
            search_results_limit: 200,
            partition: HashMap::new(),
        }
    }

    fn generate_index(&mut self) {
        fn traverse_index(current_path: &PathBuf, indexed: &mut Vec<PathBuf>) {
            if current_path.metadata().is_err() {
                return;
            }

            if let Ok(entries) = read_dir(current_path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        traverse_index(&entry.path(), indexed);
                    } else if entry.path().is_file() {
                        indexed.push(entry.path());
                    }
                }
            }
        }

        let mut indexed_files = Vec::new();
        traverse_index(&self.root_dir, &mut indexed_files);
        indexed_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for (i, path) in indexed_files.iter().enumerate() {
            let k = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .chars()
                .nth(0)
                .unwrap_or_default();
            self.partition.entry(k).or_insert(i);
        }

        self.partition.shrink_to_fit();
        indexed_files.shrink_to_fit();
        self.indexed_files = indexed_files;
    }

    fn save_index(&self) {
        if self.indexed_files.is_empty() {
            return;
        }

        let file_path = format!(
            "index {}",
            self.root_dir
                .to_str()
                .unwrap_or_default()
                .replace("\\", "")
                .replace(":", "")
        );

        let file = File::create(file_path).expect("Fail to create file");
        let writer = BufWriter::new(file);

        if let Err(e) = bincode::serialize_into(writer, &self.indexed_files) {
            eprintln!("Failed to serialize index: {}", e);
        }
    }

    fn load_index(&mut self) {
        let file_path = format!(
            "index {}",
            self.root_dir
                .to_str()
                .unwrap_or_default()
                .replace("\\", "")
                .replace(":", "")
        );

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                self.indexed_files = Vec::new();
                return;
            }
        };

        let reader = BufReader::new(file);
        self.indexed_files = bincode::deserialize_from(reader).unwrap_or_default();
    }

    fn len(&self) -> usize {
        self.indexed_files.len()
    }

    fn get_index(&self) -> &Vec<PathBuf> {
        &self.indexed_files
    }

    fn set_root_dir(&mut self, root_dir: PathBuf) {
        self.root_dir = root_dir;
    }

    fn is_index_modified(&mut self) -> bool {
        let file_path = format!(
            "index {}",
            self.root_dir
                .to_str()
                .unwrap_or_default()
                .replace("\\", "")
                .replace(":", "")
        );

        if let Ok(info) = fs::metadata(file_path) {
            if let Ok(time) = info.modified() {
                if self.last_modify_time != time {
                    self.last_modify_time = time;
                    return true;
                }
            }
        }
        false
    }

    fn get_root_dir(&self) -> &PathBuf {
        &self.root_dir
    }

    fn search_regex(&mut self, key: &str) {
        let regex = Regex::new(key).unwrap_or_else(|_| Regex::new("None").unwrap());
        let mut searched = 0usize;

        for file in &self.indexed_files {
            if searched >= self.search_results_limit {
                break;
            }

            let file_name = file.file_name().unwrap().to_str().unwrap();
            if regex.is_match(file_name) {
                if let Some(re) = regex.find(file_name) {
                    self.search_results.push((file.clone(), re.as_str().to_string()));
                    searched += 1;
                }
            }
        }
    }

    fn get_results(&self) -> &Vec<(PathBuf, String)> {
        &self.search_results
    }

    fn reset_search_results(&mut self) {
        self.search_results.clear();
    }

    fn set_search_results_limit(&mut self, limit: usize) {
        self.search_results_limit = limit;
    }

    fn clear_index_files(&mut self) {
        self.indexed_files.clear();
    }

    fn search(&mut self, key: &str) {
        let mut searched = 0usize;
        let start = self.partition.get(&key.chars().nth(0).unwrap_or_default()).cloned().unwrap_or(0);
        let end = self.indexed_files.len();

        for i in start..end {
            let file = &self.indexed_files[i];
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if searched >= self.search_results_limit || !file_name.starts_with(key.chars().nth(0).unwrap_or_default()) {
                break;
            }

            if file_name.starts_with(key) {
                self.search_results.push((file.clone(), key.to_string()));
                searched += 1;
            }
        }
    }

    fn save_partition(&self) {
        if self.partition.is_empty() {
            return;
        }

        let file_path = format!(
            "partition {}",
            self.root_dir
                .to_str()
                .unwrap_or_default()
                .replace("\\", "")
                .replace(":", "")
        );

        let file = File::create(file_path).expect("Fail to create file");
        let writer = BufWriter::new(file);

        if let Err(e) = bincode::serialize_into(writer, &self.partition) {
            eprintln!("Failed to serialize partition: {}", e);
        }
    }

    fn load_partition(&mut self) {
        let file_path = format!(
            "partition {}",
            self.root_dir
                .to_str()
                .unwrap_or_default()
                .replace("\\", "")
                .replace(":", "")
        );

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                self.partition = HashMap::new();
                return;
            }
        };

        let reader = BufReader::new(file);
        self.partition = bincode::deserialize_from(reader).unwrap_or_default();
    }

    fn get_partition(&self) -> &HashMap<char, usize> {
        &self.partition
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
