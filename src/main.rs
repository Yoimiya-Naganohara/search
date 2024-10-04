#![windows_subsystem = "windows"]

mod search_engine;
mod ui_handle;

use egui::{IconData, ViewportBuilder};
use search_engine::{Search, SearchEngine};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, Sender};
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
            app.set_message_sender(send);
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
    let (sender, receiver) = channel();
    start_search_thread(recv, sender);
    start_update_thread(receiver);
}

fn start_search_thread(recv: Receiver<String>, sender: Sender<String>) {
    let mut engine = Search::new();
    thread::spawn(move || {
        while let Ok(mut received) = recv.recv() {
            if received.starts_with(':') {
                received.remove(0);
                let _ = sender.send(received);
                continue;
            }
            process_search_request(&mut engine, &received);
        }
    });
}

fn process_search_request(engine: &mut Search, received: &str) {
    engine.set_root_dir([received.to_string()].iter().collect());
    engine.generate_index();
    engine.save_index();
    engine.clear_index_files();
}

fn start_update_thread(recv: Receiver<String>) {
    let update_time = read_update_time("updateTime.ini").unwrap_or(600);
    let mut update_time = Duration::from_secs(update_time);

    let mut engine = Search::new();
    thread::spawn(move || loop {
        sleep(update_time.div_f64(1.25));
        if let Ok(update_time_s) = recv.recv_timeout(update_time.div_f64(5.0)) {
            update_time = parse_update_time(&update_time_s, update_time.as_secs());
            if update_time_s.is_empty() {
                update_time = update_time.mul_f64(2.0);
            }
        }
        update_all_drives(&mut engine);
    });
}

fn parse_update_time(update_time_s: &str, prev: u64) -> Duration {
    let update_time_s = update_time_s.parse::<u64>().unwrap_or(prev);
    Duration::from_secs(update_time_s)
}

fn update_all_drives(engine: &mut Search) {
    for path in 'A'..='Z' {
        let drive_path = format!("{}:\\", path);
        engine.set_root_dir([drive_path].iter().collect());
        engine.generate_index();
        engine.save_index();
        engine.clear_index_files();
    }
}

fn read_update_time(path: &str) -> Option<u64> {
    let mut file = File::open(path).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    buf.trim().parse::<u64>().ok()
}
