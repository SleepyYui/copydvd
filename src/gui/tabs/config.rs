use crate::config::Config;
use crate::gui::notifications::{notify_error, notify_success};
use crate::gui::state::UiState;
use crate::gui::theme::{full_width_button, grouped_section, styled_panel, Layout};
use directories::UserDirs;
use num_cpus;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub fn render_config_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    ui.heading("Configuration");
    ui.separator();

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
}

fn render_dvd_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "DVD Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Input path:");
                ui.add_sized(
                    [200.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.input_path),
                );

                if ui.button("Browse").clicked() {
                    browse_for_input(ui_state);
                };
            });

            ui.checkbox(
                &mut ui_state.config_temp.auto_download,
                "Auto-detect DVD drives",
            );
            ui.checkbox(&mut ui_state.main_feature_only, "Main feature only");
        });
    });
}

fn render_output_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Output Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                ui.add_sized(
                    [200.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.output_path),
                );

                if ui.button("Browse").clicked() {
                    browse_for_output(ui_state);
                };
            });

            ui.horizontal(|ui| {
                ui.label("Output format:");
                egui::ComboBox::from_id_source("output_format_combo")
                    .selected_text(
                        egui::RichText::new(&ui_state.config_temp.encode_algo)
                            .color(crate::gui::theme::ModernTheme::TEXT_PRIMARY),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut ui_state.config_temp.encode_algo,
                            "MP4".to_string(),
                            "MP4",
                        );
                        ui.selectable_value(
                            &mut ui_state.config_temp.encode_algo,
                            "MKV".to_string(),
                            "MKV",
                        );
                        ui.selectable_value(
                            &mut ui_state.config_temp.encode_algo,
                            "AVI".to_string(),
                            "AVI",
                        );
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Filename pattern:");
                ui.add_sized(
                    [200.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.config_temp.naming_pattern),
                );
            });

            ui.checkbox(
                &mut ui_state.config_temp.organize_by_date,
                "Create subfolders for each DVD",
            );
            ui.checkbox(
                &mut ui_state.config_temp.auto_cleanup,
                "Overwrite existing files",
            );
        });
    });
}

fn render_quality_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Quality Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Video quality:");
                ui.add_sized(
                    [200.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.config_temp.custom_args),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Video codec:");
                egui::ComboBox::from_id_source("video_codec_combo")
                    .selected_text(
                        egui::RichText::new(&ui_state.config_temp.video_codec)
                            .color(crate::gui::theme::ModernTheme::TEXT_PRIMARY),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut ui_state.config_temp.video_codec,
                            "H.264".to_string(),
                            "H.264",
                        );
                        ui.selectable_value(
                            &mut ui_state.config_temp.video_codec,
                            "H.265".to_string(),
                            "H.265",
                        );
                        ui.selectable_value(
                            &mut ui_state.config_temp.video_codec,
                            "VP9".to_string(),
                            "VP9",
                        );
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Thread count:");
                ui.add_sized(
                    [100.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.config_temp.thread_count),
                );
            });

            ui.checkbox(
                &mut ui_state.config_temp.gpu_acceleration,
                "GPU acceleration",
            );
            ui.checkbox(
                &mut ui_state.config_temp.two_pass_encoding,
                "Two-pass encoding",
            );
        });
    });
}

fn render_config_actions(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Configuration", |ui| {
            if full_width_button(ui, "Calculate Best Settings").clicked() {
                calculate_optimal_settings(ui_state);
            }

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

            ui.add_space(Layout::SPACING);

            if full_width_button(ui, "Open App Cache Folder").clicked() {
                open_app_cache_folder();
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
            ui_state.output_path = UserDirs::new()
                .and_then(|user_dirs| user_dirs.video_dir().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
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
        config.thread_count = ui_state
            .config_temp
            .thread_count
            .parse()
            .unwrap_or(num_cpus::get().max(1));
        config.eject_after_rip = ui_state.config_temp.eject_after_rip;
        config.chapter_split = ui_state.chapter_split;

        // Update HandBrake path if provided
        if !ui_state.config_temp.handbrake_path.is_empty() {
            config.handbrake_path = Some(ui_state.config_temp.handbrake_path.clone().into());
        }

        // Update HandBrake management settings
        config.handbrake_management.auto_download = ui_state.config_temp.auto_download;
        config.handbrake_management.prefer_system = ui_state.config_temp.prefer_system;
        config.handbrake_management.max_cache_size_mb = ui_state
            .config_temp
            .max_cache_size_mb
            .parse()
            .unwrap_or(100);
        config.handbrake_management.verify_on_startup = ui_state.config_temp.verify_on_startup;

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

        // Also update UI-specific fields that aren't in config_temp
        ui_state.output_path = config.output_dir.to_string_lossy().to_string();
        ui_state.chapter_split = config.chapter_split;

        notify_success("Configuration loaded successfully");
    } else {
        notify_error("Failed to load configuration");
    }
}

fn reset_to_defaults(ui_state: &mut UiState) {
    ui_state.input_path.clear();
    ui_state.output_path = UserDirs::new()
        .and_then(|user_dirs| user_dirs.video_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
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
            Ok(json_content) => match std::fs::write(&path, json_content) {
                Ok(_) => notify_success(&format!("Configuration exported to {}", path.display())),
                Err(e) => notify_error(&format!("Failed to write config file: {}", e)),
            },
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
            Ok(content) => match serde_json::from_str::<ExportableConfig>(&content) {
                Ok(imported_config) => {
                    apply_imported_config(ui_state, imported_config);
                    notify_success(&format!("Configuration imported from {}", path.display()));
                }
                Err(e) => notify_error(&format!("Failed to parse config file: {}", e)),
            },
            Err(e) => notify_error(&format!("Failed to read config file: {}", e)),
        }
    }
}

fn open_app_cache_folder() {
    use directories::ProjectDirs;

    if let Some(project_dirs) = ProjectDirs::from("com", "sleepyyui", "copydvd") {
        let cache_dir = project_dirs.cache_dir();

        // Create cache directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(cache_dir) {
            notify_error(&format!("Failed to create cache directory: {}", e));
            return;
        }

        // Open the cache directory
        if let Err(e) = open::that(cache_dir) {
            notify_error(&format!("Failed to open cache folder: {}", e));
        } else {
            notify_success(&format!("Opened app cache folder: {}", cache_dir.display()));
        }
    } else {
        notify_error("Failed to determine app cache directory");
    }
}

fn calculate_optimal_settings(ui_state: &mut UiState) {
    use crate::gui::notifications::{notify_info, notify_success};

    notify_info("Calculating optimal settings based on system capabilities...");

    // Detect system capabilities
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    // Calculate optimal thread count (leave 1-2 cores for system)
    let optimal_threads = if cpu_count <= 2 {
        1
    } else if cpu_count <= 4 {
        cpu_count - 1
    } else {
        cpu_count - 2
    };

    // Set optimal thread count
    ui_state.config_temp.thread_count = optimal_threads.to_string();

    // Set optimal quality based on expected use case
    // RF 20-23 is generally good for archival quality
    ui_state.config_temp.quality = 22;

    // Default to H.264 for better compatibility
    ui_state.config_temp.video_codec = "H.264".to_string();

    // Default to MP4 for better compatibility
    ui_state.config_temp.encode_algo = "MP4".to_string();

    // Enable GPU acceleration if available (conservative default: off)
    ui_state.config_temp.gpu_acceleration = false;

    // Two-pass encoding for better quality (but slower)
    ui_state.config_temp.two_pass_encoding = false;

    // Set reasonable handbrake preset
    ui_state.config_temp.handbrake_preset = "Fast 1080p30".to_string();

    notify_success(&format!(
        "Optimal settings calculated: {} threads, RF{} quality, {} codec",
        optimal_threads, 22, "H.264"
    ));
}
