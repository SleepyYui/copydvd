use crate::config::Config;
use crate::gui::state::UiState;
use crate::gui::theme::{BasicTheme, Layout, styled_panel, grouped_section, full_width_button};
use crate::gui::notifications::{notify_success, notify_error};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

pub fn render_config_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    ui.heading("Configuration");
    ui.separator();

    // Wrap content in scroll area to prevent overflow
    egui::ScrollArea::both().show(ui, |ui| {
        // DVD Settings
        render_dvd_settings(ui, ui_state);
        
        ui.add_space(Layout::SPACING_LARGE);

        // Output Settings
        render_output_settings(ui, ui_state);
        
        ui.add_space(Layout::SPACING_LARGE);

        // Quality Settings
        render_quality_settings(ui, ui_state);
        
        ui.add_space(Layout::SPACING_LARGE);

        // Save/Load Settings
        render_config_actions(ui, ui_state, config);
    });
}

fn render_dvd_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "DVD Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Input path:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.input_path));
                
                if ui.button("Browse").clicked() {
                    browse_for_input(ui_state);
                }
            });
            
            ui.checkbox(&mut ui_state.config_temp.auto_download, "Auto-detect DVD drives");
            ui.checkbox(&mut ui_state.main_feature_only, "Main feature only");
        });
    });
}

fn render_output_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Output Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.output_path));
                
                if ui.button("Browse").clicked() {
                    browse_for_output(ui_state);
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Output format:");
                egui::ComboBox::from_id_source("output_format_combo")
                    .selected_text(&ui_state.config_temp.encode_algo)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut ui_state.config_temp.encode_algo, "MP4".to_string(), "MP4");
                        ui.selectable_value(&mut ui_state.config_temp.encode_algo, "MKV".to_string(), "MKV");
                        ui.selectable_value(&mut ui_state.config_temp.encode_algo, "AVI".to_string(), "AVI");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Filename pattern:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.naming_pattern));
            });
            
            ui.checkbox(&mut ui_state.config_temp.organize_by_date, "Create subfolders for each DVD");
            ui.checkbox(&mut ui_state.config_temp.auto_cleanup, "Overwrite existing files");
        });
    });
}

fn render_quality_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Quality Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Video quality:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.custom_args));
            });
            
            ui.horizontal(|ui| {
                ui.label("Video codec:");
                egui::ComboBox::from_id_source("video_codec_combo")
                    .selected_text(&ui_state.config_temp.video_codec)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut ui_state.config_temp.video_codec, "H.264".to_string(), "H.264");
                        ui.selectable_value(&mut ui_state.config_temp.video_codec, "H.265".to_string(), "H.265");
                        ui.selectable_value(&mut ui_state.config_temp.video_codec, "VP9".to_string(), "VP9");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Thread count:");
                ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.thread_count));
            });
            
            ui.checkbox(&mut ui_state.config_temp.gpu_acceleration, "GPU acceleration");
            ui.checkbox(&mut ui_state.config_temp.two_pass_encoding, "Two-pass encoding");
        });
    });
}

fn render_config_actions(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Configuration", |ui| {
            if full_width_button(ui, "Save Settings").clicked() {
                save_config(ui_state, config.clone());
            }
            
            ui.add_space(Layout::SPACING_SMALL);
            
            if full_width_button(ui, "Load Settings").clicked() {
                load_config(ui_state, config.clone());
            }
            
            ui.add_space(Layout::SPACING_SMALL);
            
            if full_width_button(ui, "Reset to Defaults").clicked() {
                reset_to_defaults(ui_state);
            }
            
            ui.add_space(Layout::SPACING);
            
            if full_width_button(ui, "Export Config").clicked() {
                export_config(ui_state);
            }
            
            ui.add_space(Layout::SPACING_SMALL);
            
            if full_width_button(ui, "Import Config").clicked() {
                import_config(ui_state);
            }
        });
    });
}

fn browse_for_input(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select DVD Source")
        .pick_folder()
    {
        ui_state.input_path = path.to_string_lossy().to_string();
    }
}

fn browse_for_output(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select Output Directory")
        .pick_folder()
    {
        ui_state.output_path = path.to_string_lossy().to_string();
        
        // Set as default if empty
        if ui_state.output_path.is_empty() {
            ui_state.output_path = std::env::home_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
                .join("Movies")
                .to_string_lossy()
                .to_string();
        }
    }
}

/// Exportable configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExportableConfig {
    pub input_path: String,
    pub output_path: String,
    pub encode_algo: String,
    pub video_codec: String,
    pub thread_count: String,
    pub custom_args: String,
    pub naming_pattern: String,
    pub main_feature_only: bool,
    pub auto_download: bool,
    pub organize_by_date: bool,
    pub auto_cleanup: bool,
    pub gpu_acceleration: bool,
    pub two_pass_encoding: bool,
    pub version: String,
}

/// Create exportable config from UI state
fn create_export_config(ui_state: &UiState) -> ExportableConfig {
    ExportableConfig {
        input_path: ui_state.input_path.clone(),
        output_path: ui_state.output_path.clone(),
        encode_algo: ui_state.config_temp.encode_algo.clone(),
        video_codec: ui_state.config_temp.video_codec.clone(),
        thread_count: ui_state.config_temp.thread_count.clone(),
        custom_args: ui_state.config_temp.custom_args.clone(),
        naming_pattern: ui_state.config_temp.naming_pattern.clone(),
        main_feature_only: ui_state.main_feature_only,
        auto_download: ui_state.config_temp.auto_download,
        organize_by_date: ui_state.config_temp.organize_by_date,
        auto_cleanup: ui_state.config_temp.auto_cleanup,
        gpu_acceleration: ui_state.config_temp.gpu_acceleration,
        two_pass_encoding: ui_state.config_temp.two_pass_encoding,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Apply imported config to UI state
fn apply_imported_config(ui_state: &mut UiState, config: ExportableConfig) {
    ui_state.input_path = config.input_path;
    ui_state.output_path = config.output_path;
    ui_state.config_temp.encode_algo = config.encode_algo;
    ui_state.config_temp.video_codec = config.video_codec;
    ui_state.config_temp.thread_count = config.thread_count;
    ui_state.config_temp.custom_args = config.custom_args;
    ui_state.config_temp.naming_pattern = config.naming_pattern;
    ui_state.main_feature_only = config.main_feature_only;
    ui_state.config_temp.auto_download = config.auto_download;
    ui_state.config_temp.organize_by_date = config.organize_by_date;
    ui_state.config_temp.auto_cleanup = config.auto_cleanup;
    ui_state.config_temp.gpu_acceleration = config.gpu_acceleration;
    ui_state.config_temp.two_pass_encoding = config.two_pass_encoding;
}

fn save_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(mut config) = config.try_lock() {
        // Update config from UI state
        config.output_dir = ui_state.output_path.clone().into();
        config.encode_algo = ui_state.config_temp.encode_algo.clone();
        config.video_codec = ui_state.config_temp.video_codec.clone();
        config.thread_count = ui_state.config_temp.thread_count.parse().unwrap_or(0);
        
        if let Err(e) = config.save() {
            notify_error(&format!("Failed to save config: {}", e));
        } else {
            notify_success("Configuration saved successfully");
        }
    } else {
        notify_error("Failed to access configuration");
    }
}

fn load_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(config) = config.try_lock() {
        // Load config into UI state
        ui_state.load_config_temp(&config);
        notify_success("Configuration loaded successfully");
    } else {
        notify_error("Failed to load configuration");
    }
}

fn reset_to_defaults(ui_state: &mut UiState) {
    ui_state.input_path.clear();
    ui_state.output_path = std::env::home_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        .join("Movies")
        .to_string_lossy()
        .to_string();
    ui_state.config_temp = crate::gui::state::ConfigTemp::default();
    ui_state.main_feature_only = false;
    
    notify_success("Settings reset to defaults");
}

fn export_config(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Export Configuration")
        .add_filter("JSON", &["json"])
        .save_file()
    {
        // Create exportable config from UI state
        let export_config = create_export_config(ui_state);
        
        match serde_json::to_string_pretty(&export_config) {
            Ok(json_content) => {
                match std::fs::write(&path, json_content) {
                    Ok(_) => notify_success(&format!("Configuration exported to {}", path.display())),
                    Err(e) => notify_error(&format!("Failed to write config file: {}", e)),
                }
            }
            Err(e) => notify_error(&format!("Failed to serialize configuration: {}", e)),
        }
    }
}

fn import_config(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Import Configuration")
        .add_filter("JSON", &["json"])
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<ExportableConfig>(&content) {
                    Ok(imported_config) => {
                        apply_imported_config(ui_state, imported_config);
                        notify_success(&format!("Configuration imported from {}", path.display()));
                    }
                    Err(e) => notify_error(&format!("Failed to parse config file: {}", e)),
                }
            }
            Err(e) => notify_error(&format!("Failed to read config file: {}", e)),
        }
    }
}