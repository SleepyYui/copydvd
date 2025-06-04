use crate::app::state::AppState;
use crate::config::Config;
use crate::gui::state::{Tab, UiState, UpdateStatus};
use crate::gui::tabs::*;
use crate::gui::theme::apply_theme;
use crate::gui::utils::updates::{auto_check_for_updates, UpdateCheckResult};
use crate::handbrake_manager::HandBrakeManager;

use egui::{Align, Layout, RichText};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

pub mod components;
pub mod notifications;
pub mod state;
pub mod tabs;
pub mod theme;
pub mod utils;

#[derive(Debug)]
pub enum HandBrakeStatus {
    Verifying,
    Verified(String), // Binary path
    Error(String),    // Error message
}

/// Main entry point for the simple GUI application
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([700.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Copy DVD",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            cc.egui_ctx.set_pixels_per_point(1.0);
            Ok(Box::new(CopyDvdApp::new(cc)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    egui::IconData {
        rgba: vec![0; 32 * 32 * 4],
        width: 32,
        height: 32,
    }
}

struct CopyDvdApp {
    app_state: Arc<Mutex<AppState>>,
    ui_state: UiState,
    config: Arc<Mutex<Config>>,
    first_frame: bool,
    update_receiver: Receiver<UpdateCheckResult>,
    handbrake_receiver: Receiver<HandBrakeStatus>,
    handbrake_status: Option<HandBrakeStatus>,
}

impl CopyDvdApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (_update_sender, update_receiver) = mpsc::channel();
        let (handbrake_sender, handbrake_receiver) = mpsc::channel();

        // Start HandBrake verification immediately
        tokio::spawn(async move {
            let _ = handbrake_sender.send(HandBrakeStatus::Verifying);

            let mut handbrake_manager = match HandBrakeManager::new() {
                Ok(manager) => manager,
                Err(e) => {
                    let _ = handbrake_sender.send(HandBrakeStatus::Error(format!(
                        "Failed to initialize HandBrake manager: {}",
                        e
                    )));
                    return;
                }
            };

            match handbrake_manager.verify_handbrake().await {
                Ok(binary_path) => {
                    let _ = handbrake_sender.send(HandBrakeStatus::Verified(binary_path));
                }
                Err(e) => {
                    let _ = handbrake_sender.send(HandBrakeStatus::Error(e.to_string()));
                }
            }
        });

        Self {
            app_state: Arc::new(Mutex::new(AppState::new(Config::default()))),
            ui_state: UiState::new(),
            config: Arc::new(Mutex::new(Config::default())),
            first_frame: true,
            update_receiver,
            handbrake_receiver,
            handbrake_status: None,
        }
    }

    fn update_status(&mut self, ctx: &egui::Context) {
        self.check_for_handbrake_status();

        if let Ok(_state) = self.app_state.try_lock() {
            // Note: AppState doesn't have an error field, so we'll skip this check
            // if let Some(error) = &state.error {
            //     self.ui_state.error_message = Some(error.to_string());
            // }
        }
        ctx.request_repaint();
    }

    fn handle_first_frame(&mut self) {
        if self.first_frame {
            // Start async update check using channel communication
            let (sender, receiver) = mpsc::channel();
            self.update_receiver = receiver;

            tokio::spawn(async move {
                let result = auto_check_for_updates().await;
                let _ = sender.send(result);
            });

            self.first_frame = false;
        }
    }

    fn check_for_handbrake_status(&mut self) {
        // Non-blocking check for HandBrake verification results
        if let Ok(status) = self.handbrake_receiver.try_recv() {
            self.handbrake_status = Some(status);
        }
    }

    fn check_for_update_results(&mut self) {
        // Non-blocking check for update results
        if let Ok(result) = self.update_receiver.try_recv() {
            match result {
                UpdateCheckResult::UpdateAvailable {
                    version,
                    download_url,
                    changelog: _,
                } => {
                    self.ui_state.update_status = UpdateStatus::UpdateAvailable {
                        version,
                        url: download_url,
                    };
                    self.ui_state.show_update_dialog = true;
                }
                UpdateCheckResult::UpToDate => {
                    self.ui_state.update_status = UpdateStatus::UpToDate;
                }
                UpdateCheckResult::Error(error) => {
                    self.ui_state.update_status = UpdateStatus::Error(error);
                }
            }
            self.ui_state.checking_updates = false;
        }
    }

    fn save_all_configs(&mut self) {
        self.update_config_from_ui();
        if let Ok(config) = self.config.try_lock() {
            if let Err(e) = config.save() {
                self.ui_state.error_message = format!("Failed to save config: {}", e);
            }
        }
    }

    fn update_config_from_ui(&mut self) {
        if let Ok(mut config) = self.config.try_lock() {
            // Update config from UI state
            config.output_dir = self.ui_state.output_path.clone().into();
            // Note: Config structure doesn't have direct dvd or server fields with these properties
            // These would need to be updated based on the actual Config structure
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Copy DVD").strong().size(16.0));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Ok(state) = self.app_state.try_lock() {
                    let status_text = match &state.status {
                        crate::app::state::AppStatus::Idle => "Ready",
                        crate::app::state::AppStatus::Scanning => "Scanning",
                        crate::app::state::AppStatus::Ripping { .. } => "Processing",
                        crate::app::state::AppStatus::Error(_) => "Error",
                        _ => "Active",
                    };

                    ui.label(status_text);

                    if !self.ui_state.titles.is_empty() {
                        ui.label(format!("{} titles", self.ui_state.titles.len()));
                    }
                }
            });
        });
        ui.separator();
    }

    fn render_navigation_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for tab in Tab::all() {
                let is_active = self.ui_state.active_tab == tab;

                if ui.selectable_label(is_active, tab.name()).clicked() {
                    self.ui_state.active_tab = tab;
                }
            }
        });
        ui.separator();
    }

    fn trigger_dvd_scan(&mut self) {
        use crate::gui::notifications::notify_info;

        if self.ui_state.input_path.is_empty() {
            notify_info("Please select a DVD input path first");
            return;
        }

        notify_info("Starting DVD scan...");

        // Clear previous titles
        self.ui_state.titles.clear();

        // In a real implementation, this would spawn an async task
        // For now, we'll simulate finding titles
        // TODO: Implement actual async DVD scanning with HandBrake integration
        let input_path = self.ui_state.input_path.clone();

        // Simulate async DVD scanning
        tokio::spawn(async move {
            // This would call the actual DVD scanning logic
            // let mut dvd = Dvd::new(PathBuf::from(input_path), config, handbrake_manager);
            // let result = dvd.scan_titles().await;

            // For now, just log that scanning would happen
            tracing::info!("Would scan DVD at: {}", input_path);
        });
    }
}

impl eframe::App for CopyDvdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_first_frame();
        self.check_for_update_results();
        self.update_status(ctx);

        // Main application layout
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            self.render_header(ui);

            // Simple navigation tabs
            self.render_navigation_tabs(ui);

            // Main content area
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .id_source("main_content_scroll")
                .show(ui, |ui| {
                    // HandBrake status display at top
                    if let Some(ref handbrake_status) = self.handbrake_status {
                        match handbrake_status {
                            HandBrakeStatus::Verifying => {
                                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "🔍 Verifying HandBrake installation...");
                                ui.separator();
                            }
                            HandBrakeStatus::Verified(path) => {
                                ui.colored_label(egui::Color32::from_rgb(0, 150, 0), format!("✅ HandBrake verified: {}", path));
                                ui.separator();
                            }
                            HandBrakeStatus::Error(error) => {
                                let error_clone = error.clone();

                                // Check if automatic fixes were attempted
                                let auto_fixes_attempted = error_clone.contains("Automatic security fixes were attempted");

                                if auto_fixes_attempted {
                                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "⚡ HandBrake Auto-Fix Attempted:");
                                    ui.label("The application automatically tried to resolve macOS security issues.");
                                } else {
                                    ui.colored_label(egui::Color32::from_rgb(200, 50, 50), "❌ HandBrake Error:");
                                }
                                ui.separator();

                                // Create a scrollable area for the error message
                                egui::ScrollArea::vertical()
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        let mut error_text = error.as_str();
                                        ui.add(egui::TextEdit::multiline(&mut error_text)
                                            .desired_width(f32::INFINITY)
                                            .font(egui::TextStyle::Monospace));
                                    });

                                ui.separator();

                                ui.horizontal(|ui| {
                                    // Add retry button with different text based on auto-fixes
                                    let button_text = if auto_fixes_attempted {
                                        "🔄 Retry After Auto-Fix"
                                    } else {
                                        "🔄 Retry HandBrake Verification"
                                    };

                                    if ui.button(button_text).clicked() {
                                        self.handbrake_status = None;
                                        let (handbrake_sender, handbrake_receiver) = mpsc::channel();
                                        self.handbrake_receiver = handbrake_receiver;

                                        tokio::spawn(async move {
                                            let _ = handbrake_sender.send(HandBrakeStatus::Verifying);

                                            let mut handbrake_manager = match HandBrakeManager::new() {
                                                Ok(manager) => manager,
                                                Err(e) => {
                                                    let _ = handbrake_sender.send(HandBrakeStatus::Error(format!("Failed to initialize HandBrake manager: {}", e)));
                                                    return;
                                                }
                                            };

                                            match handbrake_manager.verify_handbrake().await {
                                                Ok(binary_path) => {
                                                    let _ = handbrake_sender.send(HandBrakeStatus::Verified(binary_path));
                                                }
                                                Err(e) => {
                                                    let _ = handbrake_sender.send(HandBrakeStatus::Error(e.to_string()));
                                                }
                                            }
                                        });
                                    }

                                    // Add macOS-specific System Preferences button - always show on macOS
                                    #[cfg(target_os = "macos")]
                                    if ui.button("🔧 Open Security Settings").clicked() {
                                        tokio::spawn(async {
                                            // Try multiple methods to open Security preferences
                                            let methods = [
                                                ("open", vec!["-b", "com.apple.systempreferences", "/System/Library/PreferencePanes/Security.prefPane"]),
                                                ("open", vec!["/System/Library/PreferencePanes/Security.prefPane"]),
                                                ("open", vec!["-a", "System Preferences"]),
                                            ];

                                            for (cmd, args) in &methods {
                                                if std::process::Command::new(cmd).args(args).spawn().is_ok() {
                                                    break;
                                                }
                                            }
                                        });
                                    }
                                });

                                // Show helpful status message for auto-fixes
                                if auto_fixes_attempted {
                                    ui.separator();
                                    ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "💡 What happened:");
                                    ui.label("• Removed quarantine attributes automatically");
                                    ui.label("• Set executable permissions");
                                    ui.label("• Attempted to open Security preferences");
                                    ui.label("• Triggered macOS security dialog");
                                    ui.add_space(5.0);
                                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "➡️ Next steps:");
                                    ui.label("1. Click 'Retry After Auto-Fix' above");
                                    ui.label("2. If still blocked, use 'Open Security Settings'");
                                    ui.label("3. Look for 'Allow Anyway' button in Security settings");
                                }

                                ui.separator();
                            }
                        }
                    }

                    // Error display at top
                    if !self.ui_state.error_message.is_empty() {
                        ui.colored_label(egui::Color32::from_rgb(180, 60, 60), &self.ui_state.error_message);
                        ui.separator();
                    }

                    // Content
                    match self.ui_state.active_tab {
                        Tab::Main => {
                            render_main_tab(ui, &mut self.ui_state, self.app_state.clone());
                        }
                        Tab::Config => {
                            render_config_tab(ui, &mut self.ui_state, self.config.clone());
                        }
                        Tab::Server => {
                            render_server_tab(ui, &mut self.ui_state, self.config.clone());
                        }
                        Tab::HandBrake => {
                            render_handbrake_tab(ui, &mut self.ui_state, self.config.clone());
                        }
                        Tab::About => {
                            render_about_tab(ui, &mut self.ui_state);
                        }
                    }
                });
        });

        // Keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) && self.ui_state.active_tab == Tab::Main {
            self.trigger_dvd_scan();
        }

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_all_configs();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_all_configs();
    }
}
