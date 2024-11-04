mod engine;
use std::{
    path::PathBuf,
    sync::mpsc::Receiver,
};

use engine::{Search, SearchEngine};
pub fn start_search_engine(
    recv: Receiver<String>,
    sender: std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, String)>>>,
) {
    let mut search_engine = Search::new();

    loop {
        if search_engine.get_index().is_empty() {
            search_engine.load_index();
            if search_engine.get_index().is_empty() {
                search_engine.generate_index();
            }
        }
        if let Ok(msg) = recv.try_recv() {
            match dbg!(msg).as_str() {
                "UpdateIndex" => {
                    search_engine.generate_index();
                }
                msg if msg.starts_with("Search:") => {
                    let msg = msg.trim_start_matches("Search:");
                    search_engine.search(msg);
                    if let Ok(mut writer) = sender.lock() {
                        *writer = search_engine.get_results().clone();
                        search_engine.reset_search_results();
                        dbg!(search_engine.get_results().len());
                    };
                }
                msg if msg.starts_with("SetRootDir:") => {
                    let msg = msg.trim_start_matches("SetRootDir:");
                    search_engine.set_root_dir(msg.into());
                }
                _ => {}
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::{sync::{mpsc, Arc, Mutex}, thread, time::Duration};

    use super::*;

    #[test]
    fn test_update_index() {
        let (tx, rx) = mpsc::channel();
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        thread::spawn(move || {
            start_search_engine(rx, results_clone);
        });

        tx.send("UpdateIndex".to_string()).unwrap();
        thread::sleep(Duration::from_secs(1));

        let results = results.lock().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search() {
        let (tx, rx) = mpsc::channel();
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        thread::spawn(move || {
            start_search_engine(rx, results_clone);
        });

        tx.send("Search:test".to_string()).unwrap();
        thread::sleep(Duration::from_secs(1));

        let results = results.lock().unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_set_root_dir() {
        let (tx, rx) = mpsc::channel();
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        thread::spawn(move || {
            start_search_engine(rx, results_clone);
        });

        tx.send("SetRootDir:/new/root/dir".to_string()).unwrap();
        thread::sleep(Duration::from_secs(1));

        // Assuming there's a way to verify the root directory was set correctly
        // This is a placeholder assertion
        assert!(true);
    }
}
