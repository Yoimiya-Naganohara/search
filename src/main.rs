#![windows_subsystem = "windows"]

mod ui;

use egui::{IconData, ViewportBuilder};
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
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

    let shared_memory = Arc::new(Mutex::new(Vec::new()));
    let shared_memory_clone = Arc::clone(&shared_memory);
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
