use std::{
    fs::File,
    io::{Read, Write},
    ops::AddAssign,
    path::PathBuf,
    process::Command,
    sync::mpsc::Sender,
    time::{Duration, SystemTime},
};

use crate::search_engine::{Search, SearchEngine};
use egui::{FontDefinitions, FontFamily};
use image::ImageReader;

/// Represents the main application structure for the search functionality.
pub struct SearchApp {
    search_command: String,
    search_results: Vec<(PathBuf, String)>,
    search_engine: Search,
    display_dialog: bool,
    root_directory: String,
    notification_message: Option<String>,
    message_sender: Option<Sender<String>>,
    loading_status: bool,
    updating_status: bool,
    last_active_time: SystemTime,
    current_active_time: SystemTime,
    avg_suspend_duration: Duration,
}

impl Default for SearchApp {
    fn default() -> Self {
        let mut update_interval = 600;
        if let Ok(mut file) = File::open("updateTime.ini") {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap();
            update_interval = buffer.parse::<u64>().unwrap_or(600);
        }
        SearchApp {
            search_command: String::new(),
            search_results: Vec::new(),
            search_engine: Search::new(),
            display_dialog: false,
            root_directory: String::from("C:\\"),
            notification_message: None,
            message_sender: None,
            loading_status: false,
            updating_status: false,
            last_active_time: SystemTime::now(),
            current_active_time: SystemTime::now(),
            avg_suspend_duration: Duration::from_secs(update_interval),
        }
    }
}

/// A trait that defines the core functionalities for a search application engine.
pub(crate) trait SearchAppEngine {
    fn render_results_list(&mut self, ui: &mut egui::Ui);
    fn render_settings_window(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);
    fn render_search_input(&mut self, ui: &mut egui::Ui);
    fn render_loading_status(&mut self, ui: &mut egui::Ui);
    fn update_interface(&mut self, ctx: &egui::Context);
    fn execute_search(&mut self);
    fn set_message_sender(&mut self, sender: Sender<String>);
    fn new(cc: &eframe::CreationContext<'_>) -> Self;
    fn refresh_index(&self);
    fn validate_index(&mut self);
    fn update_avg_suspend_duration(&mut self);
}

impl SearchAppEngine for SearchApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = cc;
        Self::default()
    }

    fn set_message_sender(&mut self, sender: Sender<String>) {
        self.message_sender = Some(sender);
    }

    fn execute_search(&mut self) {
        self.search_engine.reset_search_results();
        self.search_engine.search(&self.search_command);
        self.search_results = self.search_engine.get_results().clone();
    }

    fn update_interface(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.ui_contains_pointer() {
                self.validate_index();
            }
            ui.vertical(|ui| {
                self.render_search_input(ui);
                if self.display_dialog {
                    self.render_settings_window(ctx, ui);
                }
                if self.loading_status {
                    self.render_loading_status(ui);
                }
                self.render_results_list(ui);
            });
        });
    }

    fn render_search_input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let search_input = ui.add(
                egui::TextEdit::singleline(&mut self.search_command)
                    .hint_text("Search")
                    .desired_width(ui.available_width() - 40.0),
            );
            if !self.display_dialog {
                search_input.request_focus();
            }
            if search_input.changed() {
                self.update_avg_suspend_duration();
                self.execute_search();
            }
            if ui.button("Set").clicked() {
                self.display_dialog = true;
            }
        });
    }

    fn render_settings_window(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let _ = ui;
        egui::Window::new("Setting")
            .open(&mut self.display_dialog)
            .show(ctx, |ui| {
                ui.heading("Root Path");
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut self.root_directory).changed() {
                        self.notification_message = None;
                    }
                    if ui.button("Switch").clicked() {
                        self.search_engine
                            .set_root_dir([self.root_directory.clone()].iter().collect());
                        self.search_engine.load_index();
                        self.notification_message =
                            Some("Root directory switched successfully".to_string());
                    }
                });
                if let Some(ref message) = self.notification_message {
                    ui.label(message);
                }
                ui.heading("Update Index");
                ui.label(format!(
                    "Automatic index update interval: {} seconds",
                    self.avg_suspend_duration.as_secs().to_string()
                ));
                if ui.button("Update Index Immediately").clicked() {
                    if let Some(sender) = &self.message_sender {
                        let _ = sender.send(self.root_directory.clone());
                    }
                }
            });
    }

    fn render_results_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (path, matched) in &self.search_results {
                if matched.is_empty() {
                    continue;
                }
                ui.horizontal(|ui| {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let file_name = format!("-{} ", file_name);
                    let default_visuals = ui.visuals().clone();
                    let file_name_parts: Vec<&str> = file_name.split(matched).collect();
                    let file_path = path.to_str().unwrap();
                    for part in file_name_parts {
                        {
                            let label = ui.label(part);
                            if label.clicked() && open::that(file_path).is_ok() {}
                            label
                                .clone()
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            ui.add_space(-8.5);
                            label.on_hover_text(file_path);
                            if !part.ends_with(' ') {
                                let matched_label = ui.strong(matched);
                                if matched_label.clicked() && open::that_detached(file_path).is_ok()
                                {
                                }
                                matched_label
                                    .clone()
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                                matched_label.on_hover_text(file_path);
                                ui.add_space(-8.5);
                            }
                        }
                    }
                    ui.visuals_mut().override_text_color = Some(default_visuals.hyperlink_color);
                    if !self.search_command.is_empty() {
                        ui.add_space(1.0);
                        let explorer_button = ui
                            .label("σ")
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if explorer_button.clicked() {
                            let _ = Command::new("explorer").arg("/select,").arg(path).spawn();
                        }
                    }
                });
            }
        });
    }

    fn refresh_index(&self) {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(self.root_directory.clone());
        }
    }

    fn validate_index(&mut self) {
        if self.search_engine.len() == 0 {
            if self.loading_status && !self.updating_status {
                self.updating_status = true;
                self.refresh_index();
            }
            self.search_engine.load_index();
            self.loading_status = true
        } else {
            self.loading_status = false;
            self.updating_status = false;
        }
    }

    fn render_loading_status(&mut self, ui: &mut egui::Ui) {
        ui.heading("Loading...");
    }

    fn update_avg_suspend_duration(&mut self) {
        self.current_active_time = SystemTime::now();
        if let Ok(suspend_duration) = self.current_active_time.duration_since(self.last_active_time) {
            if suspend_duration.as_secs() >= 300 {
                self.last_active_time = self.current_active_time;
                self.avg_suspend_duration.add_assign(suspend_duration);
                self.avg_suspend_duration = self.avg_suspend_duration.div_f32(2.0);
                if let Some(sender) = &self.message_sender {
                    let _ = sender.send(format!(
                        ":{}",
                        self.avg_suspend_duration.as_secs().to_string()
                    ));
                }
            }
        };
    }
}

impl eframe::App for SearchApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _ = frame;
        setup_custom_fonts(ctx);
        self.update_interface(ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Ok(mut file) = File::create("updateTime.ini") {
            file.write(
                self.avg_suspend_duration
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
