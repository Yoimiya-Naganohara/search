#![windows_subsystem = "windows"]
mod handle;
mod search_engine;

use handle::{Handle, Handler};
use search_engine::{Search, SearchEngine};
use std::env::args;
use std::sync::mpsc::channel;
use std::thread::{self, sleep};
use std::time::Duration;
mod ui_handle;
use ui_handle::{SearchApp, SearchAppEngine};

fn main() {
    if args().nth(1).is_some() {
        run_cli_mode();
    } else {
        run_gui_mode();
    }
}

fn run_cli_mode() {
    let mut handler = Handle::new();
    handler.welcome();
    loop {
        handler.input();
        handler.handler();
    }
}

fn run_gui_mode() {
    let (send, recv) = channel();
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Search",
        native_options,
        Box::new(|cc| {
            let mut app = SearchApp::new(cc);
            app.set_sender(send);
            start_background_threads(recv);
            Ok(Box::new(app))
        }),
    );
}

fn start_background_threads(recv: std::sync::mpsc::Receiver<String>) {
    let mut engine = Search::new();
    thread::spawn(move || loop {
        if let Ok(received) = recv.recv() {
            engine.set_root_dir([received].iter().collect());
            engine.generate_index();
            engine.save_index();
            engine.clear_index_files();
        }
    });

    let mut engine = Search::new();
    thread::spawn(move || loop {
        sleep(Duration::from_secs(600));
        for path in 'A'..='Z' {
            engine.set_root_dir([format!("{}:\\", path)].iter().collect());
            engine.generate_index();
            engine.save_index();
            engine.clear_index_files();
        }
    });
}
