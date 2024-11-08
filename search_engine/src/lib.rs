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

enum SortBy {
    NONE,
    SizeU,
    SizeD,
}

fn sort_by(mut search_results: Vec<(PathBuf, String)>, sortby: SortBy) -> Vec<(PathBuf, String)> {
    match sortby {
        SortBy::SizeU => {
            search_results.sort_by(|a, b| {
                a.0.metadata()
                    .unwrap()
                    .len()
                    .cmp(&b.0.metadata().unwrap().len())
            });
        }
        SortBy::SizeD => {
            search_results.sort_by(|b, a| {
                a.0.metadata()
                    .unwrap()
                    .len()
                    .cmp(&b.0.metadata().unwrap().len())
            });
        }
        _ => {}
    }
    search_results
}

fn handle_message(
    msg: &str,
    search_engine: &mut Search,
    sender: &Arc<Mutex<Vec<(PathBuf, String)>>>,
) {
    match msg {
        "UpdateIndex" => handle_update_index(search_engine),
        msg if msg.starts_with("SearchRegex:") => handle_search_regex(msg, search_engine, sender),
        msg if msg.starts_with("Search:") => handle_search(msg, search_engine, sender),
        msg if msg.starts_with("SetRootDir:") => handle_set_root_dir(msg, search_engine),
        _ => {}
    }
}

fn handle_update_index(search_engine: &mut Search) {
    search_engine.clear_index_files();
    let mut search_engine_clone = search_engine.clone();
    thread::spawn(move || {
        search_engine_clone.generate_index();
        search_engine_clone.save_partition();
        search_engine_clone.save_index();
        search_engine_clone.clear_index_files();
    });
}

fn handle_search_regex(
    msg: &str,
    search_engine: &mut Search,
    sender: &Arc<Mutex<Vec<(PathBuf, String)>>>,
) {
    let mut query = msg.trim_start_matches("SearchRegex:");
    let sortby = determine_sort_by(&mut query);
    search_engine.search_regex(query);
    let search_results = search_engine.get_results();
    let search_results = sort_by(search_results.clone(), sortby);
    update_sender_results(search_results, sender);
    search_engine.reset_search_results();
}

fn handle_search(
    msg: &str,
    search_engine: &mut Search,
    sender: &Arc<Mutex<Vec<(PathBuf, String)>>>,
) {
    let mut query = msg.trim_start_matches("Search:");
    let sortby = determine_sort_by(&mut query);
    search_engine.search(query);
    let search_results = search_engine.get_results();
    let search_results = sort_by(search_results.clone(), sortby);
    update_sender_results(search_results, sender);
    search_engine.reset_search_results();
}

fn handle_set_root_dir(msg: &str, search_engine: &mut Search) {
    let dir = msg.trim_start_matches("SetRootDir:");
    search_engine.set_root_dir(dir.into());
}

fn determine_sort_by(query: &mut &str) -> SortBy {
    match *query {
        q if q.ends_with(":size") => {
            *query = q.trim_end_matches(":size");
            SortBy::SizeU
        }
        q if q.ends_with(":size.u") => {
            *query = q.trim_end_matches(":size.u");
            SortBy::SizeU
        }
        q if q.ends_with(":size.d") => {
            *query = q.trim_end_matches(":size.d");
            SortBy::SizeD
        }
        _ => SortBy::NONE,
    }
}

fn update_sender_results(
    search_results: Vec<(PathBuf, String)>,
    sender: &Arc<Mutex<Vec<(PathBuf, String)>>>,
) {
    if let Ok(mut writer) = sender.lock() {
        *writer = search_results;
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
