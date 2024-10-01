use std::{
    fs::File,
    io::{Read, Write},
    ops::AddAssign,
    path::PathBuf,
    sync::mpsc::Sender,
    time::{Duration, SystemTime},
};

use crate::search_engine::{Search, SearchEngine};
use egui::{FontDefinitions, FontFamily, Key};

/// Represents the main application structure for the search functionality.
pub struct SearchApp {
    command: String,
    file_list: Vec<(PathBuf, String)>,
    engine: Search,
    show_dialog: bool,
    root_dir: String,
    notice_message: Option<String>,
    sender: Option<Sender<String>>,
    is_loading: bool,
    is_updating: bool,
    last_usage_time: SystemTime,
    current_time: SystemTime,
    average_suspend_duration: Duration,
}

impl Default for SearchApp {
    fn default() -> Self {
        let mut update_time = 600;
        if let Ok(mut file) = File::open("updateTime.ini") {
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            update_time = buf.parse::<u64>().unwrap_or(600);
        }
        SearchApp {
            command: String::new(),
            file_list: Vec::new(),
            engine: Search::new(),
            show_dialog: false,
            root_dir: String::from("C:\\"),
            notice_message: None,
            sender: None,
            is_loading: false,
            is_updating: false,
            last_usage_time: SystemTime::now(),
            current_time: SystemTime::now(),
            average_suspend_duration: Duration::from_secs(update_time),
        }
    }
}

/// A trait that defines the core functionalities for a search application engine.
pub(crate) trait SearchAppEngine {
    fn render_file_list(&mut self, ui: &mut egui::Ui);
    fn render_settings_dialog(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);
    fn render_search_bar(&mut self, ui: &mut egui::Ui);
    fn render_loading(&mut self, ui: &mut egui::Ui);
    fn update_ui(&mut self, ctx: &egui::Context);
    fn get(&mut self);
    fn set_sender(&mut self, send: Sender<String>);
    fn new(cc: &eframe::CreationContext<'_>) -> Self;
    fn update_index(&self);
    fn verify_index(&mut self);
    fn update_average_time_suspend(&mut self);
}

impl SearchAppEngine for SearchApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;
        Self::default()
    }

    fn set_sender(&mut self, send: Sender<String>) {
        self.sender = Some(send);
    }

    fn get(&mut self) {
        self.engine.reset_search_results();
        self.engine.search(&self.command);
        self.file_list = self.engine.get_results().clone();
    }

    fn update_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.render_search_bar(ui);
                if self.show_dialog {
                    self.render_settings_dialog(ctx, ui);
                }
                if self.is_loading {
                    self.render_loading(ui);
                }
                self.render_file_list(ui);
            });
        });
    }

    fn render_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let search_bar = ui.add(
                egui::TextEdit::singleline(&mut self.command)
                    .hint_text("Search")
                    .desired_width(ui.available_width() - 40.0),
            );
            if !self.show_dialog {
                search_bar.request_focus();
            }
            if search_bar.changed() {
                self.get();
            }
            if ui.button("Set").clicked() {
                self.show_dialog = true;
            }
        });
    }

    fn render_settings_dialog(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let _ = ui;
        egui::Window::new("Setting")
            .open(&mut self.show_dialog)
            .show(ctx, |ui| {
                ui.heading("Root Path");
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut self.root_dir).changed() {
                        self.notice_message = None;
                    }
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
                ui.heading("Update Index");
                ui.label(format!(
                    "Automatic index update interval: {} seconds",
                    self.average_suspend_duration.as_secs().to_string()
                ));
                if ui.button("Update Index Immediately").clicked() {
                    if let Some(sender) = &self.sender {
                        let _ = sender.send(self.root_dir.clone());
                    }
                }
            });
    }

    fn render_file_list(&mut self, ui: &mut egui::Ui) {
        let file_list = self.file_list.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (path, matched) in &file_list {
                if matched.is_empty() {
                    continue;
                }
                ui.horizontal(|ui| {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let file_name = format!("-{} ", file_name);
                    let default_visuals = ui.visuals().clone();
                    let file_name_parts: Vec<&str> = file_name.split(matched).collect();
                    let file_path = path.to_str().unwrap();
                    for i in file_name_parts {
                        {
                            let label = ui.label(i);
                            if label.contains_pointer() {
                                self.update_average_time_suspend();
                            }
                            if label.clicked() && open::that(file_path).is_ok() {}
                            label
                                .clone()
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            ui.add_space(-8.5);
                            label.on_hover_text(file_path);
                            if !i.ends_with(' ') {
                                let matched_label = ui.strong(matched);
                                if matched_label.clicked() && open::that(file_path).is_ok() {}
                                matched_label
                                    .clone()
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                                matched_label.on_hover_text(file_path);
                                ui.add_space(-8.5);
                            }
                        }
                    }
                    ui.visuals_mut().override_text_color = Some(default_visuals.hyperlink_color);
                    if !self.command.is_empty() {
                        ui.add_space(1.0);
                        let e = ui
                            .label("σ")
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        let path = path.parent().unwrap();
                        if e.clicked() && open::that(path).is_ok() {}
                    }
                });
            }
        });
    }

    fn update_index(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(self.root_dir.clone());
        }
    }

    fn verify_index(&mut self) {
        if self.engine.len() == 0 {
            if self.is_loading && !self.is_updating {
                self.is_updating = true;
                self.update_index();
            }
            self.engine.load_index();
            self.is_loading = true
        } else {
            self.is_loading = false;
            self.is_updating = false;
        }
    }

    fn render_loading(&mut self, ui: &mut egui::Ui) {
        ui.heading("Loading...");
    }

    fn update_average_time_suspend(&mut self) {
        self.current_time = SystemTime::now();
        if let Ok(suspend_time) = self.current_time.duration_since(self.last_usage_time) {
            if suspend_time.as_secs() >= 300 {
                self.last_usage_time = self.current_time;
                self.average_suspend_duration.add_assign(suspend_time);
                self.average_suspend_duration = self.average_suspend_duration.div_f32(2.0);
            }
        };
    }
}

impl eframe::App for SearchApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _ = frame;
        setup_custom_fonts(ctx);
        self.verify_index();
        self.update_ui(ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Ok(mut file) = File::create("updateTime.ini") {
            file.write(
                self.average_suspend_duration
                    .as_secs()
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Load a font that supports Chinese characters
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("./font/NotoSerifCJKsc-Regular.otf")),
    );

    // Insert the font into the font family
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    ctx.set_fonts(fonts);
}
