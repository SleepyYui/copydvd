use crate::config::Config;
use crate::gui::components::*;
use crate::gui::state::UiState;
use crate::gui::theme::{glass_card, tech_section, ModernTheme, StyleConstants, responsive_container, responsive_two_column, animated_glass_card, get_device_type};
use std::sync::{Arc, Mutex};

/// Render the configuration tab with responsive layout
pub fn render_config_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    let screen_width = ui.available_width();
    let device_type = get_device_type(screen_width);
    
    responsive_container(ui, |ui| {
        // Clone necessary state to avoid borrow conflicts
        let handbrake_path = ui_state.config_temp.handbrake_path.clone();
        let output_path = ui_state.output_path.clone();
        let encode_algo = ui_state.config_temp.encode_algo.clone();
        let thread_count = ui_state.config_temp.thread_count.clone();
        let eject_after_rip = ui_state.config_temp.eject_after_rip;
        
        responsive_two_column(ui, 
            // Left column - General Settings
            |ui| {
                animated_glass_card(ui, egui::Id::new("general_settings"), true, |ui| {
                    tech_section(ui, "General Settings", Some(ModernTheme::NEON_CYAN), |ui| {
                        ui.label("HandBrake Path:");
                        ui.colored_label(ModernTheme::TEXT_SECONDARY, &handbrake_path);
                        
                        ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_SM));
                        
                        ui.label("Output Directory:");
                        ui.colored_label(ModernTheme::TEXT_SECONDARY, &output_path);
                        
                        ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_SM));
                        
                        ui.label("Encoding Algorithm:");
                        ui.colored_label(ModernTheme::TEXT_SECONDARY, &encode_algo);
                        
                        ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_SM));
                        
                        ui.label("Thread Count:");
                        ui.colored_label(ModernTheme::TEXT_SECONDARY, &thread_count);
                    });
                });

                ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_MD));

                animated_glass_card(ui, egui::Id::new("behavior_settings"), false, |ui| {
                    tech_section(ui, "Behavior Settings", Some(ModernTheme::NEON_PURPLE), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Eject disc after ripping:");
                            ui.colored_label(
                                if eject_after_rip { ModernTheme::NEON_GREEN } else { ModernTheme::TEXT_MUTED },
                                if eject_after_rip { "Enabled" } else { "Disabled" }
                            );
                        });

                        ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_SM));
                        ui.colored_label(
                            ModernTheme::TEXT_MUTED,
                            "Additional options will be added in future updates",
                        );
                    });
                });
            },
            // Right column - Advanced Settings  
            |ui| {
                animated_glass_card(ui, egui::Id::new("advanced_settings"), true, |ui| {
                    tech_section(ui, "Advanced Settings", Some(ModernTheme::NEON_CYAN), |ui| {
                        ui.label("Quality Settings:");
                        ui.indent("quality", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Video Quality:");
                                ui.colored_label(ModernTheme::TEXT_SECONDARY, "High");
                            });

                            ui.horizontal(|ui| {
                                ui.label("Audio Quality:");
                                ui.colored_label(ModernTheme::TEXT_SECONDARY, "High");
                            });
                        });

                        ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_MD));

                        ui.label("Processing Options:");
                        ui.indent("processing", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("GPU acceleration:");
                                ui.colored_label(ModernTheme::TEXT_MUTED, "Available");
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Fast start:");
                                ui.colored_label(ModernTheme::NEON_GREEN, "Enabled");
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Preserve metadata:");
                                ui.colored_label(ModernTheme::NEON_GREEN, "Enabled");
                            });
                        });
                    });
                });

                ui.add_space(StyleConstants::SPACING_MD);

                animated_glass_card(ui, egui::Id::new("config_management"), true, |ui| {
                    tech_section(ui, "Configuration Management", Some(ModernTheme::NEON_GREEN), |ui| {
                        button_group(ui, |ui| {
                            if ui.add(success_button("💾 Save Settings")).clicked() {
                                // TODO: Implement save
                            }

                            if ui.add(danger_button("Reset to Defaults")).clicked() {
                                // TODO: Implement reset
                            }
                        });

                        ui.add_space(StyleConstants::SPACING_SM);

                        button_group(ui, |ui| {
                            if ui.add(secondary_button("📤 Export Config")).clicked() {
                                // TODO: Implement export
                            }

                            if ui.add(secondary_button("📥 Import Config")).clicked() {
                                // TODO: Implement import
                            }
                        });

                        ui.add_space(StyleConstants::SPACING_SM);
                        ui.separator();
                        ui.add_space(StyleConstants::SPACING_SM);

                        ui.colored_label(
                            ModernTheme::TEXT_MUTED,
                            "Configuration changes will take effect after restart",
                        );
                    });
                });
            }
        );
    });
}

/// Browse for HandBrake executable
fn browse_for_handbrake(ui_state: &mut UiState) {
    let file_dialog = rfd::FileDialog::new().set_title("Select HandBrake Executable");

    #[cfg(windows)]
    let file_dialog = file_dialog.add_filter("Executable", &["exe"]);

    if let Some(path) = file_dialog.pick_file() {
        ui_state.config_temp.handbrake_path = path.to_string_lossy().to_string();
    }
}

/// Browse for output directory
fn browse_for_output_dir(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select Default Output Directory")
        .pick_folder()
    {
        ui_state.output_path = path.to_string_lossy().to_string();
    }
}

/// Save configuration
fn save_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(mut config) = config.try_lock() {
        // Update config from UI state
        if !ui_state.config_temp.handbrake_path.is_empty() {
            config.handbrake_path = Some(ui_state.config_temp.handbrake_path.clone().into());
        } else {
            config.handbrake_path = None;
        }

        config.encode_algo = ui_state.config_temp.encode_algo.clone();
        config.eject_after_rip = ui_state.config_temp.eject_after_rip;

        if let Ok(thread_count) = ui_state.config_temp.thread_count.parse::<usize>() {
            config.thread_count = thread_count;
        }

        if !ui_state.output_path.is_empty() {
            config.output_dir = ui_state.output_path.clone().into();
        }

        // Save to disk
        if let Err(e) = config.save() {
            ui_state.set_error(format!("Failed to save configuration: {}", e));
        } else {
            ui_state.set_status("Configuration saved successfully".to_string());
        }
    }
}

/// Reset configuration to defaults
fn reset_to_defaults(ui_state: &mut UiState) {
    let default_config = Config::default();
    ui_state.load_config_temp(&default_config);
    ui_state.set_status("Configuration reset to defaults".to_string());
}

/// Export configuration to file
fn export_config(config: Arc<Mutex<Config>>) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Export Configuration")
        .add_filter("JSON", &["json"])
        .set_file_name("dvd_ripper_config.json")
        .save_file()
    {
        if let Ok(config) = config.try_lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*config) {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("Failed to export configuration: {}", e);
                }
            }
        }
    }
}

/// Import configuration from file
fn import_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Import Configuration")
        .add_filter("JSON", &["json"])
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<Config>(&json) {
                Ok(imported_config) => {
                    if let Ok(mut config) = config.try_lock() {
                        *config = imported_config;
                        ui_state.load_config_temp(&*config);
                        if let Err(e) = config.save() {
                            ui_state
                                .set_error(format!("Failed to save imported configuration: {}", e));
                        } else {
                            ui_state.set_status("Configuration imported successfully".to_string());
                        }
                    }
                }
                Err(e) => {
                    ui_state.set_error(format!("Invalid configuration file: {}", e));
                }
            },
            Err(e) => {
                ui_state.set_error(format!("Failed to read configuration file: {}", e));
            }
        }
    }
}
