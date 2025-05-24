use std::sync::Arc;
use tokio::sync::Mutex;
use eframe::egui;
use tracing::{info, warn};

use crate::app::{AppState, AppStatus};
use crate::error::{Result, AppError};
use crate::dvd::types::Title;

/// Run the GUI application using egui
pub fn run(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    info!("Starting egui GUI");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("DVD Ripper"),
        ..Default::default()
    };

    let app = DvdRipperApp::new(app_state);
    
    eframe::run_native(
        "DVD Ripper",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ).map_err(|e| AppError::GuiError(format!("Failed to run GUI: {}", e)))?;
    
    Ok(())
}

struct DvdRipperApp {
    app_state: Arc<Mutex<AppState>>,
    input_path: String,
    output_path: String,
    selected_titles: Vec<bool>,
    main_feature_only: bool,
    chapter_split: bool,
    upload_to_server: bool,
    status_message: String,
    error_message: String,
    titles: Vec<Title>,
    show_config_panel: bool,
    show_server_config: bool,
    
    // Configuration fields
    handbrake_path: String,
    encode_algo: String,
    thread_count: String,
    eject_after_rip: bool,
    
    // Server configuration
    server_host: String,
    server_username: String,
    server_password: String,
    server_path: String,
    
    // Runtime state - removed to avoid nested runtime issues
}

impl DvdRipperApp {
    fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        // Get initial configuration without blocking
        let (config, status) = {
            if let Ok(state) = app_state.try_lock() {
                (state.config.clone(), state.status.clone())
            } else {
                (crate::config::Config::default(), AppStatus::Idle)
            }
        };
        
        Self {
            app_state,
            input_path: String::new(),
            output_path: config.output_dir.to_string_lossy().to_string(),
            selected_titles: Vec::new(),
            main_feature_only: false,
            chapter_split: config.chapter_split,
            upload_to_server: false,
            status_message: format!("{:?}", status),
            error_message: String::new(),
            titles: Vec::new(),
            show_config_panel: false,
            show_server_config: false,
            
            handbrake_path: config.handbrake_path.as_ref().map_or(String::new(), |p| p.to_string_lossy().to_string()),
            encode_algo: config.encode_algo,
            thread_count: config.thread_count.to_string(),
            eject_after_rip: config.eject_after_rip,
            
            server_host: config.server.as_ref().map(|s| s.host.clone()).unwrap_or_default(),
            server_username: config.server.as_ref().map(|s| s.username.clone()).unwrap_or_default(),
            server_password: config.server.as_ref().and_then(|s| s.password.clone()).unwrap_or_default(),
            server_path: config.server.as_ref().map(|s| s.path.clone()).unwrap_or_default(),
        }
    }
    
    fn update_status(&mut self) {
        let status = if let Ok(state) = self.app_state.try_lock() {
            state.status.clone()
        } else {
            return; // Skip update if can't get lock
        };
        
        self.status_message = match status {
            AppStatus::Idle => "Ready".to_string(),
            AppStatus::Ready => "Ready to scan DVD".to_string(),
            AppStatus::Scanning => "Scanning DVD...".to_string(),
            AppStatus::ScanComplete(count) => format!("Scan complete - {} titles found", count),
            AppStatus::Ripping { completed, total } => format!("Ripping: {}/{} completed", completed, total),
            AppStatus::RipComplete => "Ripping complete".to_string(),
            AppStatus::Uploading { progress } => format!("Uploading: {:.1}%", progress * 100.0),
            AppStatus::UploadComplete => "Upload complete".to_string(),
            AppStatus::Completed => "All operations completed".to_string(),
            AppStatus::Error(ref msg) => {
                self.error_message = msg.clone();
                format!("Error: {}", msg)
            }
        };
    }
    
    fn scan_dvd(&mut self) {
        if self.input_path.is_empty() {
            self.error_message = "Please select a DVD path first".to_string();
            return;
        }
        
        let input_path = std::path::PathBuf::from(&self.input_path);
        let app_state: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
        
        tokio::spawn(async move {
            let mut state = app_state.lock().await;
            state.status = AppStatus::Scanning;
            
            // Create DVD object and scan
            match crate::dvd::types::Dvd::new(input_path, state.config.clone()).await {
                Ok(mut dvd) => {
                    match dvd.scan_titles().await {
                        Ok(_) => {
                            let title_count = dvd.titles.len();
                            state.status = AppStatus::ScanComplete(title_count);
                            state.dvd = Some(Arc::new(Mutex::new(dvd)));
                        }
                        Err(e) => {
                            state.status = AppStatus::Error(format!("Scan failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    state.status = AppStatus::Error(format!("Failed to create DVD object: {}", e));
                }
            }
        });
    }
    
    fn start_ripping(&mut self) {
        if self.titles.is_empty() {
            self.error_message = "Please scan DVD first".to_string();
            return;
        }
        
        let app_state: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
        let output_path = std::path::PathBuf::from(&self.output_path);
        let selected_titles: Vec<usize> = self.selected_titles
            .iter()
            .enumerate()
            .filter_map(|(i, &selected)| if selected { Some(i + 1) } else { None })
            .collect();
        let main_feature = self.main_feature_only;
        let chapter_split = self.chapter_split;
        
        tokio::spawn(async move {
            let dvd_arc = {
                let state = app_state.lock().await;
                if let Some(dvd_arc) = &state.dvd {
                    Arc::clone(dvd_arc)
                } else {
                    let mut state = app_state.lock().await;
                    state.status = AppStatus::Error("No DVD scanned".to_string());
                    return;
                }
            };
            
            let tasks = {
                let dvd = dvd_arc.lock().await;
                
                let final_titles = if main_feature {
                    dvd.find_main_feature().map(|title| vec![title.number])
                } else if !selected_titles.is_empty() {
                    Some(selected_titles)
                } else {
                    None
                };
                
                dvd.create_rip_tasks(output_path, final_titles, chapter_split)
            };
            
            let task_count = tasks.len();
            if task_count > 0 {
                {
                    let mut state = app_state.lock().await;
                    state.status = AppStatus::Ripping { completed: 0, total: task_count };
                    state.rip_tasks = tasks.clone();
                }
                
                // Start ripping tasks
                for (i, task) in tasks.iter().enumerate() {
                    let result = {
                        let dvd = dvd_arc.lock().await;
                        dvd.rip_title(task).await
                    };
                    
                    let mut state = app_state.lock().await;
                    match result {
                        Ok(_) => {
                            if let AppStatus::Ripping { completed, total } = &mut state.status {
                                *completed += 1;
                                if *completed >= *total {
                                    state.status = AppStatus::RipComplete;
                                }
                            }
                        }
                        Err(e) => {
                            state.status = AppStatus::Error(format!("Rip task {} failed: {}", i + 1, e));
                            break;
                        }
                    }
                }
            } else {
                let mut state = app_state.lock().await;
                state.status = AppStatus::Error("No tasks created".to_string());
            }
        });
    }
    
    fn browse_for_folder(&mut self, target: &str) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            match target {
                "input" => self.input_path = path.to_string_lossy().to_string(),
                "output" => self.output_path = path.to_string_lossy().to_string(),
                _ => {}
            }
        }
    }
    
    fn save_config(&mut self) {
        let app_state: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
        let handbrake_path = std::path::PathBuf::from(&self.handbrake_path);
        let output_dir = std::path::PathBuf::from(&self.output_path);
        let encode_algo = self.encode_algo.clone();
        let thread_count = self.thread_count.parse().unwrap_or(4);
        let eject_after_rip = self.eject_after_rip;
        let chapter_split = self.chapter_split;
        
        let server_config = if !self.server_host.is_empty() {
            Some(crate::config::ServerConfig {
                host: self.server_host.clone(),
                username: self.server_username.clone(),
                password: Some(self.server_password.clone()),
                path: self.server_path.clone(),
            })
        } else {
            None
        };
        
        tokio::spawn(async move {
            let mut state = app_state.lock().await;
            state.config.handbrake_path = Some(handbrake_path);
            state.config.output_dir = output_dir;
            state.config.encode_algo = encode_algo;
            state.config.thread_count = thread_count;
            state.config.eject_after_rip = eject_after_rip;
            state.config.chapter_split = chapter_split;
            state.config.server = server_config;
            
            // Save to file
            if let Err(e) = state.config.save() {
                warn!("Failed to save configuration: {}", e);
            }
        });
    }
}

impl eframe::App for DvdRipperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update status from background tasks
        self.update_status();
        
        // Update titles if available
        if let Ok(state) = self.app_state.try_lock() {
            if let Some(dvd_arc) = &state.dvd {
                if let Ok(dvd) = dvd_arc.try_lock() {
                    if self.titles.len() != dvd.titles.len() {
                        self.titles = dvd.titles.clone();
                        self.selected_titles = vec![false; self.titles.len()];
                    }
                }
            }
        }
        
        // Request repaint for status updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DVD Ripper");
            
            // Error message display
            if !self.error_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.error_message);
                if ui.button("Dismiss").clicked() {
                    self.error_message.clear();
                }
                ui.separator();
            }
            
            // Status display
            ui.label(format!("Status: {}", self.status_message));
            ui.separator();
            
            // Input/Output paths
            ui.horizontal(|ui| {
                ui.label("DVD Path:");
                ui.text_edit_singleline(&mut self.input_path);
                if ui.button("Browse").clicked() {
                    self.browse_for_folder("input");
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Output Path:");
                ui.text_edit_singleline(&mut self.output_path);
                if ui.button("Browse").clicked() {
                    self.browse_for_folder("output");
                }
            });
            
            ui.separator();
            
            // Scan and action buttons
            ui.horizontal(|ui| {
                if ui.button("Scan DVD").clicked() {
                    self.scan_dvd();
                }
                
                if ui.button("Start Ripping").clicked() {
                    self.start_ripping();
                }
            });
            
            ui.separator();
            
            // Options
            ui.checkbox(&mut self.main_feature_only, "Main feature only");
            ui.checkbox(&mut self.chapter_split, "Split chapters");
            ui.checkbox(&mut self.upload_to_server, "Upload to server");
            
            ui.separator();
            
            // Configuration panels
            ui.horizontal(|ui| {
                if ui.button("Configuration").clicked() {
                    self.show_config_panel = !self.show_config_panel;
                }
                
                if ui.button("Server Settings").clicked() {
                    self.show_server_config = !self.show_server_config;
                }
            });
            
            // Configuration panel
            if self.show_config_panel {
                ui.separator();
                ui.heading("Configuration");
                
                ui.horizontal(|ui| {
                    ui.label("HandBrake Path:");
                    ui.text_edit_singleline(&mut self.handbrake_path);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Encode Algorithm:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.encode_algo)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.encode_algo, "x264".to_string(), "x264");
                            ui.selectable_value(&mut self.encode_algo, "x265".to_string(), "x265");
                        });
                });
                
                ui.horizontal(|ui| {
                    ui.label("Thread Count:");
                    ui.text_edit_singleline(&mut self.thread_count);
                });
                
                ui.checkbox(&mut self.eject_after_rip, "Eject after ripping");
                
                if ui.button("Save Configuration").clicked() {
                    self.save_config();
                    self.show_config_panel = false;
                }
            }
            
            // Server configuration panel
            if self.show_server_config {
                ui.separator();
                ui.heading("Server Configuration");
                
                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut self.server_host);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.server_username);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.server_password).password(true));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Remote Path:");
                    ui.text_edit_singleline(&mut self.server_path);
                });
                
                if ui.button("Save Server Config").clicked() {
                    self.save_config();
                    self.show_server_config = false;
                }
            }
            
            // Title selection
            if !self.titles.is_empty() {
                ui.separator();
                ui.heading("DVD Titles");
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, title) in self.titles.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if i < self.selected_titles.len() {
                                ui.checkbox(&mut self.selected_titles[i], "");
                            }
                            ui.label(format!(
                                "Title {}: {} ({} chapters, {:.1} min)",
                                title.number,
                                title.description.as_ref().unwrap_or(&"Unknown".to_string()),
                                title.chapters.len(),
                                title.duration.as_secs() as f64 / 60.0
                            ));
                        });
                    }
                });
            }
        });
    }
}