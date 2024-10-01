#![windows_subsystem = "windows"]
mod search_engine;

use egui::{IconData, ViewportBuilder};
use search_engine::{Search, SearchEngine};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread::{self, sleep};
use std::time::Duration;
mod ui_handle;
use ui_handle::{SearchApp, SearchAppEngine};

fn main() {
    run_gui_mode();
}

fn run_gui_mode() {
    let (send, recv) = channel();
    let mut icon_data = IconData::default();
    if let Ok(image_data) = image::ImageReader::open("ico.ico") {
        if let Ok(e) = image_data.decode() {
            let rgba = e.as_bytes();
            icon_data = IconData {
                rgba: rgba.to_vec(),
                width: e.width(),
                height: e.height(),
            };
        };
    };
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
    let mut update_time = 600;
    if let Ok(mut file) = File::open("updateTime.ini") {
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        update_time = buf.parse::<u64>().unwrap_or(600);
    };
    let mut engine = Search::new();
    thread::spawn(move || loop {
        sleep(Duration::from_secs(update_time));
        for path in 'A'..='Z' {
            engine.set_root_dir([format!("{}:\\", path)].iter().collect());
            engine.generate_index();
            engine.save_index();
            engine.clear_index_files();
        }
    });
}
