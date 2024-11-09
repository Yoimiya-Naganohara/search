use std::{
    fs::File,
    io::{Read, Write},
    ops::AddAssign,
    path::PathBuf,
    process::Command,
    sync::{mpsc::Sender, Arc, Mutex},
    time::{Duration, SystemTime},
};

use clipboard::ClipboardProvider;
use egui::{FontDefinitions, FontFamily};
use search_engine::{determine_search_mode, SearchMode};

/// Represents the main application structure for the search functionality.
pub struct SearchApp {
    search_command: String,
    display_dialog: bool,
    root_directory: String,
    notification_message: Option<String>,
    message_sender: Option<Sender<String>>,
    message_receiver: Option<Arc<Mutex<Vec<(PathBuf, String)>>>>,
    loading_status: bool,
    last_active_time: SystemTime,
    current_active_time: SystemTime,
    avg_suspend_duration: Duration,
}

impl Default for SearchApp {
    fn default() -> Self {
        let update_interval = File::open("updateTime.ini")
            .and_then(|mut file| {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                buffer.parse::<u64>().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid update interval")
                })
            })
            .unwrap_or(600);

        SearchApp {
            search_command: String::new(),
            display_dialog: false,
            root_directory: String::from("C:\\"),
            notification_message: None,
            message_sender: None,
            message_receiver: None,
            loading_status: false,
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
    fn set_message_receiver(&mut self, receiver: Arc<Mutex<Vec<(PathBuf, String)>>>);
    fn new(cc: &eframe::CreationContext<'_>) -> Self;
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
        if let Some(msg_sender) = &self.message_sender {
            let search_command = self.search_command.trim_start_matches(':');
            let message = if self.search_command.starts_with(':') {
                format!("SearchRegex:{}", search_command)
            } else {
                format!("Search:{}", self.search_command)
            };
            msg_sender.send(message).unwrap();
        }
    }

    fn update_interface(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.ui_contains_pointer() {}
            ui.vertical(|ui| {
                self.render_search_input(ui);
                if self.display_dialog {
                    self.render_settings_window(ctx, ui);
                }
                if self.loading_status {
                    self.render_loading_status(ui);
                }
                if !self.search_command.is_empty() {
                    self.render_results_list(ui);
                }
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

    fn render_settings_window(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::Window::new("Setting")
            .open(&mut self.display_dialog)
            .show(ctx, |ui| {
                ui.heading("Root Path");
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut self.root_directory).changed() {
                        self.notification_message = None;
                    }
                    if ui.button("Switch").clicked() {
                        if let Some(sender) = &self.message_sender {
                            let _ = sender.send(format!("SetRootDir:{}", self.root_directory));
                        }
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
                    self.avg_suspend_duration.as_secs()
                ));
                if ui.button("Update Index Immediately").clicked() {
                    if let Some(sender) = &self.message_sender {
                        let _ = sender.send("UpdateIndex".to_string());
                    }
                }
            });
    }

    fn render_results_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(receiver) = &self.message_receiver {
                if let Ok(msg) = receiver.lock() {
                    for (path, matched) in msg.iter() {
                        if matched.is_empty() {
                            continue;
                        }
                        ui.horizontal(|ui| {
                            let binding = self.search_command.clone();
                            let mut query = binding.as_str();
                            let mode = determine_search_mode(&mut query);

                            let file_name = match mode {
                                SearchMode::FILE => path.file_name().unwrap().to_str().unwrap(),
                                SearchMode::DIR => path.to_str().unwrap_or_default(),
                            };
                            let file_name = format!("-{} ", file_name);
                            let default_visuals = ui.visuals().clone();
                            let file_name_parts: Vec<&str> = file_name.split(matched).collect();
                            let file_path = path.to_str().unwrap();
                            for part in file_name_parts {
                                {
                                    let label = ui.label(part);
                                    if label.clicked_by(egui::PointerButton::Secondary) {
                                        let clipboard = clipboard::ClipboardContext::new();
                                        if let Ok(mut clipboard) = clipboard {
                                            clipboard.set_contents(file_name.clone()).unwrap();
                                        }
                                    }
                                    if label.clicked() && open::that_detached(file_path).is_ok() {}
                                    label
                                        .clone()
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                                    ui.add_space(-8.5);
                                    label.on_hover_text(format!(
                                        "{}\nfile size: {} B",
                                        file_path,
                                        path.metadata().map_or(0, |info| info.len())
                                    ));
                                    if !part.ends_with(' ') {
                                        let matched_label = ui.strong(matched);
                                        if matched_label.clicked()
                                            && open::that_detached(file_path).is_ok()
                                        {
                                        }
                                        if matched_label.clicked_by(egui::PointerButton::Secondary)
                                        {
                                            let clipboard = clipboard::ClipboardContext::new();
                                            if let Ok(mut clipboard) = clipboard {
                                                clipboard.set_contents(file_name.clone()).unwrap();
                                            }
                                        }
                                        matched_label
                                            .clone()
                                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                                        matched_label.on_hover_text(format!(
                                            "{}\nfile size: {} B",
                                            file_path,
                                            {
                                                if let Ok(info) = path.metadata() {
                                                    info.len()
                                                } else {
                                                    0
                                                }
                                            }
                                        ));
                                        ui.add_space(-8.5);
                                    }
                                }
                            }
                            ui.visuals_mut().override_text_color =
                                Some(default_visuals.hyperlink_color);
                            if !self.search_command.is_empty() {
                                ui.add_space(1.0);
                                let explorer_button = ui
                                    .label("σ")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                                if explorer_button.clicked_by(egui::PointerButton::Secondary) {
                                    let clipboard = clipboard::ClipboardContext::new();
                                    if let Ok(mut clipboard) = clipboard {
                                        clipboard
                                            .set_contents(
                                                path.to_str().unwrap_or_default().to_string(),
                                            )
                                            .unwrap();
                                    }
                                }
                                if explorer_button.clicked() {
                                    let _ =
                                        Command::new("explorer").arg("/select,").arg(path).spawn();
                                }
                            }
                        });
                    }
                }
            }
        });
    }

    fn render_loading_status(&mut self, ui: &mut egui::Ui) {
        ui.heading("Loading...");
    }

    fn update_avg_suspend_duration(&mut self) {
        self.current_active_time = SystemTime::now();
        if let Ok(suspend_duration) = self
            .current_active_time
            .duration_since(self.last_active_time)
        {
            if suspend_duration.as_secs() >= self.avg_suspend_duration.as_secs() {
                if let Some(sender) = &self.message_sender {
                    sender.send("UpdateIndex".to_string()).unwrap();
                }
            }
            if suspend_duration.as_secs() >= 300 {
                self.last_active_time = self.current_active_time;
                self.avg_suspend_duration.add_assign(suspend_duration);
                self.avg_suspend_duration = self.avg_suspend_duration.div_f32(2.0);
                if let Some(sender) = &self.message_sender {
                    let _ = sender.send(format!(
                        ":{}",
                        self.avg_suspend_duration.as_secs()
                    ));
                }
            }
        };
    }

    fn set_message_receiver(&mut self, receiver: Arc<Mutex<Vec<(PathBuf, String)>>>) {
        self.message_receiver = Some(receiver);
    }
}

impl eframe::App for SearchApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _ = frame;
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(receiver) = &self.message_receiver {
                if let Ok(msg) = receiver.lock() {
                    if let Some((path, _)) = msg.first() {
                        let _ = open::that_detached(path);
                    }
                }
            }
        }
        setup_custom_fonts(ctx);
        self.update_interface(ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Ok(mut file) = File::create("updateTime.ini") {
            file.write(self.avg_suspend_duration.as_secs().to_string().as_bytes())
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
