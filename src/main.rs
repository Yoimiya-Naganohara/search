#![windows_subsystem = "windows"]

mod search_engine;
mod ui_handle;

use egui::{IconData, ViewportBuilder};
use search_engine::{Search, SearchEngine};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver};
use std::thread::{self, sleep};
use std::time::Duration;
use ui_handle::{SearchApp, SearchAppEngine};

fn main() {
    run_gui_mode();
}

fn run_gui_mode() {
    let (send, recv) = channel();
    let icon_data = load_icon_data("ico.ico").unwrap_or_default();
    let viewport = ViewportBuilder::default();
    let native_options = eframe::NativeOptions {
        viewport: viewport.with_icon(icon_data),
        ..Default::default()
    };

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

fn load_icon_data(path: &str) -> Option<IconData> {
    let image_data = image::ImageReader::open(path).ok()?;
    let image = image_data.decode().ok()?;
    let rgba = image.as_bytes().to_vec();
    Some(IconData {
        rgba,
        width: image.width(),
        height: image.height(),
    })
}

fn start_background_threads(recv: Receiver<String>) {
    start_search_thread(recv);
    start_update_thread();
}

fn start_search_thread(recv: Receiver<String>) {
    let mut engine = Search::new();
    thread::spawn(move || loop {
        if let Ok(received) = recv.recv() {
            engine.set_root_dir([received].iter().collect());
            engine.generate_index();
            engine.save_index();
            engine.clear_index_files();
        }
    });
}

fn start_update_thread() {
    let mut update_time = read_update_time("updateTime.ini").unwrap_or(600);
    let mut engine = Search::new();
    thread::spawn(move || loop {
        sleep(Duration::from_secs(update_time));
        update_time = read_update_time("updateTime.ini").unwrap_or(600);
        for path in 'A'..='Z' {
            engine.set_root_dir([format!("{}:\\", path)].iter().collect());
            engine.generate_index();
            engine.save_index();
            engine.clear_index_files();
        }
    });
}

fn read_update_time(path: &str) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    buf.trim().parse::<u64>().ok()
}
