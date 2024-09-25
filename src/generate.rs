use crate::data::{Node, PathTree};
use std::{
    fs::{self, create_dir, exists, File},
    path::PathBuf,
};
pub(crate) trait SearchEngine {
    fn new() -> Self;
    fn generate(&mut self, current_dir: PathBuf);
    fn find(&mut self, key: &String) -> Result<&Vec<PathBuf>, ()>;
    fn store(&self);
    fn read(&mut self, section: char);
}

impl SearchEngine for Search {
    fn new() -> Self {
        Search {
            datas: Node::new(),
            find_list: Vec::new(),
            section: ' ',
        }
    }
    fn generate(&mut self, current_dir: PathBuf) {
        self.datas.clear();
        fn dfs(datas: &mut Node, current_dir: &PathBuf) {
            if current_dir.metadata().is_err()
                || current_dir.metadata().unwrap().permissions().readonly()
                || fs::read_dir(&current_dir).is_err()
            {
                return;
            }
            let entries = fs::read_dir(current_dir).expect("Failed to read directory");
            for entry in entries {
                let entry = entry.expect("Failed to get entry");
                if entry.file_type().unwrap().is_dir() {
                    dfs(datas, &entry.path());
                } else if entry.file_type().unwrap().is_file() {
                    let mut path: Vec<String> = entry
                        .path()
                        .to_str()
                        .unwrap()
                        .to_string()
                        .split("\\")
                        .map(|s| s.to_string())
                        .collect();
                    path[0].push('\\');
                    let file_name = path.pop().unwrap();
                    let path: PathBuf = path.iter().collect();
                    datas.insert(&file_name, path);
                }
            }
        }
        dfs(&mut self.datas, &current_dir);
    }
    fn find(&mut self, key: &String) -> Result<&Vec<PathBuf>, ()> {
        self.find_list.clear();
        let node = match self.datas.get(key) {
            Some(x) => x,
            None => {
                return Err(());
            }
        };
        let file_name = String::from(format!("{}{}", self.section, key));
        fn dfs(node: &Node, find_list: &mut Vec<PathBuf>, file_name: &String) {
            let mut path = node.val().clone();
            for i in &mut path {
                i.push(file_name.clone());
            }
            find_list.append(&mut path);
            if node.groups().len() == 0 {
                return;
            }
            for (ch, sub_node) in node.groups() {
                dfs(sub_node, find_list, &format!("{}{}", file_name, ch));
            }
        }
        dfs(node, &mut self.find_list, &file_name);
        Ok(&self.find_list)
    }
    fn store(&self) {
        if !exists("datas").unwrap_or(false) {
            if let Err(e) = create_dir("datas") {
                eprintln!("Failed to create directory: {}", e);
            }
        }
        for (ch, node) in self.datas.groups() {
            let file = File::create(format!("datas/data-{}", ch)).expect("Failed to create file");
            let mut writer = std::io::BufWriter::new(file);
            bincode::serialize_into(&mut writer, node).expect("Failed to serialize data");
        }
    }

    fn read(&mut self, section: char) {
        self.section = section;
        let file = match File::open(format!("datas/data-{}", section)) {
            Ok(x) => x,
            Err(_) => {
                self.datas = Node::new();
                return;
            }
        };
        let mut reader = std::io::BufReader::new(file);
        self.datas = bincode::deserialize_from(&mut reader).expect("Failed to deserialize data");
    }
}
pub struct Search {
    datas: Node,
    find_list: Vec<PathBuf>,
    section: char,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let mut search = Search::new();
        let test_dir = PathBuf::from("C:\\");
        search.generate(test_dir);
        assert!(!search.datas.is_empty());
    }

    #[test]
    fn test_find() {
        let mut search = Search::new();
        let test_dir = PathBuf::from("C:\\");
        search.generate(test_dir);
        let key = String::from("cmd");
        let result = dbg!(search.find(&key));
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_store_and_read() {
        let mut search = Search::new();
        let test_dir = PathBuf::from("C:\\");
        search.generate(test_dir);
        search.store();

        let mut new_search = Search::new();
        new_search.read('a'); // Assuming 'a' is a valid section
        assert!(!new_search.datas.is_empty());
    }
}
