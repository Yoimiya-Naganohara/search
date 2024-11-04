#![windows_subsystem = "windows"]

mod ui;

use egui::{IconData, ViewportBuilder};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread::{self};
use std::time::Duration;
use ui::{SearchApp, SearchAppEngine};

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

    let shared_memory = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let shared_memory_clone = std::sync::Arc::clone(&shared_memory);
    thread::spawn(move || {
        search_engine::start_search_engine(recv, shared_memory_clone);
    });
    let _ = eframe::run_native(
        "Search",
        native_options,
        Box::new(|cc| {
            let mut app = SearchApp::new(cc);
            app.set_message_sender(send);
            app.set_message_receiver(shared_memory);
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

fn parse_update_time(update_time_s: &str, prev: u64) -> Duration {
    let update_time_s = update_time_s.parse::<u64>().unwrap_or(prev);
    Duration::from_secs(update_time_s)
}

fn read_update_time(path: &str) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    buf.trim().parse::<u64>().ok()
}
