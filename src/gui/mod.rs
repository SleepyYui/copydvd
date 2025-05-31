use crate::app::state::AppState;
use crate::config::Config;
use crate::gui::theme::apply_theme;
use crate::gui::state::{UiState, Tab, UpdateStatus};
use crate::gui::tabs::*;
use crate::gui::utils::updates::{auto_check_for_updates, UpdateCheckResult};

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use egui::{Vec2, RichText, Align, Layout};

pub mod theme;
pub mod components;
pub mod tabs;
pub mod utils;
pub mod state;
pub mod notifications;

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
        })
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
}

impl CopyDvdApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (update_sender, update_receiver) = mpsc::channel();
        
        Self {
            app_state: Arc::new(Mutex::new(AppState::new(Config::default()))),
            ui_state: UiState::new(),
            config: Arc::new(Mutex::new(Config::default())),
            first_frame: true,
            update_receiver,
        }
    }

    fn update_status(&mut self, ctx: &egui::Context) {
        if let Ok(state) = self.app_state.try_lock() {
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

    fn check_for_update_results(&mut self) {
        // Non-blocking check for update results
        if let Ok(result) = self.update_receiver.try_recv() {
            match result {
                UpdateCheckResult::UpdateAvailable { version, download_url, changelog: _ } => {
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
                .show(ui, |ui| {
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
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            if self.ui_state.active_tab == Tab::Main {
                // TODO: Trigger DVD scan
            }
        }
        
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_all_configs();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_all_configs();
    }
}