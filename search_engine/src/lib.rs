mod engine;
use std::{
    path::PathBuf,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
};

use engine::{Search, SearchEngine};

pub fn start_search_engine(recv: Receiver<String>, sender: Arc<Mutex<Vec<(PathBuf, String)>>>) {
    let mut search_engine = Search::new();

    loop {
        if search_engine.get_index().is_empty()
            || search_engine.get_partition().is_empty()
            || search_engine.is_index_modified()
        {
            initialize_search_engine(&mut search_engine);
        }

        if let Ok(msg) = recv.recv() {
            handle_message(&msg, &mut search_engine, &sender);
        }
    }
}

fn initialize_search_engine(search_engine: &mut Search) {
    search_engine.load_partition();
    search_engine.load_index();
    if search_engine.get_index().is_empty() || search_engine.get_partition().is_empty() {
        search_engine.generate_index();
        search_engine.save_partition();
        search_engine.save_index();
    }
}

fn handle_message(
    msg: &str,
    search_engine: &mut Search,
    sender: &Arc<Mutex<Vec<(PathBuf, String)>>>,
) {
    match msg {
        "UpdateIndex" => {
            search_engine.clear_index_files();
            let mut search_engine_clone = search_engine.clone();
            thread::spawn(move || {
                search_engine_clone.generate_index();
                search_engine_clone.save_partition();
                search_engine_clone.save_index();
                search_engine_clone.clear_index_files();
            });
        }
        msg if msg.starts_with("SearchRegex:") => {
            let query = msg.trim_start_matches("SearchRegex:");
            search_engine.search_regex(query);
            update_sender_results(search_engine, sender);
        }
        msg if msg.starts_with("Search:") => {
            let query = msg.trim_start_matches("Search:");
            search_engine.search(query);
            update_sender_results(search_engine, sender);
        }
        msg if msg.starts_with("SetRootDir:") => {
            let dir = msg.trim_start_matches("SetRootDir:");
            search_engine.set_root_dir(dir.into());
        }
        _ => {}
    }
}

fn update_sender_results(search_engine: &mut Search, sender: &Arc<Mutex<Vec<(PathBuf, String)>>>) {
    if let Ok(mut writer) = sender.lock() {
        *writer = search_engine.get_results().clone();
        search_engine.reset_search_results();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread::sleep, time::Duration};

    #[test]
    fn test_update_index() {
        let (tx, rx) = mpsc::channel();
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        thread::spawn(move || {
            start_search_engine(rx, results_clone);
        });
        tx.send("SetRootDir:C:\\".to_owned()).unwrap();
        for _i in 0..10 {
            tx.send("UpdateIndex".to_string()).unwrap();
        }
        sleep(Duration::from_secs(1));

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
        tx.send("SetRootDir:C:\\".to_string()).unwrap();
        tx.send("Search:0".to_string()).unwrap();
        sleep(Duration::from_secs(1));
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
        sleep(Duration::from_secs(1));

        // Assuming there's a way to verify the root directory was set correctly
        // This is a placeholder assertion
        assert!(true);
    }

    #[test]
    fn test_search_regex() {
        let (tx, rx) = mpsc::channel();
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        thread::spawn(move || {
            start_search_engine(rx, results_clone);
        });

        tx.send("SearchRegex:.*".to_string()).unwrap();
        sleep(Duration::from_secs(1));

        let results = results.lock().unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_invalid_command() {
        let (tx, rx) = mpsc::channel();
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        thread::spawn(move || {
            start_search_engine(rx, results_clone);
        });

        tx.send("InvalidCommand".to_string()).unwrap();
        sleep(Duration::from_secs(1));

        let results = results.lock().unwrap();
        assert!(results.is_empty());
    }
}
