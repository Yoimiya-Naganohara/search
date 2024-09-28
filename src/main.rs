mod search_engine;
use eframe::egui;
use handle::{Handle, Handler};
use search_engine::{Search, SearchEngine};
use std::env::args;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, sleep};
use std::time::Duration;
mod handle;
fn main() {
    if args().nth(1).is_some() {
        let mut handler = Handle::new();
        handler.welcome();
        loop {
            handler.input();
            handler.handler();
        }
    }
    let (send, recv) = channel();
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Search",
        native_options,
        Box::new(|cc| {
            let mut app = SearchApp::new(cc);
            app.init();
            app.set_sender(send);
            let mut engine = Search::new();
            let _thread = thread::spawn(move || loop {
                if let Ok(received) = recv.recv() {
                    engine.set_root_dir([received].iter().collect());
                    engine.generate_index();
                    engine.save_index();
                    engine.clear_index_files();
                }
            });
            let mut engine = Search::new();
            let _thread = thread::spawn(move || loop {
                sleep(Duration::from_secs(600));
                for path in 'A'..='Z' {
                    engine.set_root_dir([format!("{}:\\", path)].iter().collect());
                    engine.generate_index();
                    engine.save_index();
                    engine.clear_index_files();
                }
            });

            Ok(Box::new(app))
        }),
    );
}

struct SearchApp {
    command: String,
    file_list: Vec<(PathBuf, String)>,
    engine: Search,
    show_dialog: bool,
    root_dir: String,
    notice_message: Option<String>,
    sender: Option<Sender<String>>,
}

impl Default for SearchApp {
    fn default() -> Self {
        SearchApp {
            command: String::new(),
            file_list: Vec::new(),
            engine: Search::new(),
            show_dialog: false,
            root_dir: String::from("C:\\"),
            notice_message: None,
            sender: None,
        }
    }
}

impl SearchApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;
        Self::default()
    }
    fn set_sender(&mut self, send: Sender<String>) {
        self.sender = Some(send);
    }
    fn init(&mut self) {
        self.engine.load_index();
    }
    fn get(&mut self) {
        self.engine.reset_search_results();
        self.engine.search(&self.command);
        self.file_list = self.engine.get_results().clone();
    }
}

impl eframe::App for SearchApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _ = frame;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.command)
                                .hint_text("Search")
                                .desired_width(ui.available_width() - 40.0),
                        )
                        .changed()
                    {
                        self.get();
                    }
                    if ui.button("Set").clicked() {
                        self.show_dialog = true
                    }
                });

                if self.show_dialog {
                    egui::Window::new("Setting")
                        .open(&mut self.show_dialog)
                        .show(ctx, |ui| {
                            ui.label("Root Path");
                            ui.horizontal(|ui| {
                                if ui.text_edit_singleline(&mut self.root_dir).changed() {
                                    self.notice_message = None
                                };
                                if ui.button("Switch").clicked() {
                                    self.engine
                                        .set_root_dir([self.root_dir.clone()].iter().collect());
                                    self.engine.load_index();
                                    self.notice_message =
                                        Some("Root directory switched successfully".to_string());
                                }
                            });
                            if let Some(ref message) = self.notice_message {
                                ui.label(message);
                            }
                            ui.label("Update Index");
                            if ui.button("Update Index Immediately").clicked() {
                                if let Some(sender) = &self.sender {
                                    let _ = sender.send(self.root_dir.clone());
                                }
                            }
                        });
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for (_, (path, matched)) in self.file_list.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let file_name = path.file_name().unwrap().to_str().unwrap();
                            let file_name: Vec<&str> = file_name.split(matched).collect();
                            let file_name_left_part = *file_name.first().unwrap();
                            let file_name_right_part = *file_name.last().unwrap();
                            let default_visuals = ui.visuals().clone();
                            ui.visuals_mut().override_text_color =
                                Some(default_visuals.text_color());
                            if ui.link(file_name_left_part).clicked() {
                                if open::that(path).is_ok() {};
                            }
                            ui.visuals_mut().override_text_color =
                                Some(default_visuals.hyperlink_color);
                            if ui.link(matched).clicked() {
                                if open::that(path).is_ok() {}
                            }
                            ui.visuals_mut().override_text_color =
                                Some(default_visuals.text_color());
                            if ui.link(file_name_right_part).clicked() {
                                if open::that(path).is_ok() {};
                            }
                            if !self.command.is_empty() {
                                let e = ui.small_button("σ");
                                if e.contains_pointer() {
                                    ui.label(path.to_str().unwrap());
                                }
                                if e.clicked() {
                                    let path = path.parent().unwrap();
                                    if open::that(path).is_ok() {}
                                }
                            }
                        });
                    }
                });
            });
        });
    }
}
