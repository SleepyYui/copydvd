use crate::config::Config;
use crate::gui::components::*;
use crate::gui::state::UiState;
use crate::gui::theme::{glass_card, tech_section, ModernTheme, StyleConstants};
use std::sync::{Arc, Mutex};

/// Render the configuration tab
pub fn render_config_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    ui.columns(2, |columns| {
        // Left column - General Settings
        columns[0].vertical(|ui| {
            glass_card(ui, false, |ui| {
                tech_section(ui, "General Settings", Some(ModernTheme::NEON_CYAN), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("HandBrake Path:");
                        ui.add(
                            text_input(&mut ui_state.config_temp.handbrake_path)
                                .hint_text("Leave empty for auto-detection"),
                        );
                        if ui.add(browse_button()).clicked() {
                            browse_for_handbrake(ui_state);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Output Directory:");
                        ui.add(
                            text_input(&mut ui_state.output_path)
                                .hint_text("Default output location"),
                        );
                        if ui.add(browse_button()).clicked() {
                            browse_for_output_dir(ui_state);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Encoding Algorithm:");
                        styled_combo_box(
                            ui,
                            "encode_algo",
                            &mut ui_state.config_temp.encode_algo,
                            &["x264", "x265", "VP9", "AV1"],
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Thread Count:");
                        ui.add(
                            text_input(&mut ui_state.config_temp.thread_count)
                                .hint_text("Number of CPU threads to use"),
                        );
                        ui.colored_label(ModernTheme::TEXT_MUTED, "(0 = auto)");
                    });
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            card_container(ui, |ui| {
                section(ui, "🎛️ Behavior Settings", |ui| {
                    toggle_switch(
                        ui,
                        &mut ui_state.config_temp.eject_after_rip,
                        "Eject disc after ripping",
                    );

                    ui.add_space(StyleConstants::SPACING_SM);
                    ui.colored_label(
                        ModernTheme::TEXT_MUTED,
                        "Additional options will be added in future updates",
                    );
                });
            });
        });

        // Right column - Advanced Settings
        columns[1].vertical(|ui| {
            card_container(ui, |ui| {
                section(ui, "🔧 Advanced Settings", |ui| {
                    ui.label("Quality Settings:");
                    ui.indent("quality", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Video Quality:");
                            styled_combo_box(
                                ui,
                                "video_quality",
                                &mut "High".to_string(), // TODO: Add to config
                                &["Low", "Medium", "High", "Very High"],
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Audio Quality:");
                            styled_combo_box(
                                ui,
                                "audio_quality",
                                &mut "High".to_string(), // TODO: Add to config
                                &["Medium", "High", "Very High"],
                            );
                        });
                    });

                    ui.add_space(StyleConstants::SPACING_MD);

                    ui.label("Processing Options:");
                    ui.indent("processing", |ui| {
                        let mut use_gpu = false; // TODO: Add to config
                        toggle_switch(ui, &mut use_gpu, "Use GPU acceleration (if available)");

                        let mut fast_start = true; // TODO: Add to config
                        toggle_switch(ui, &mut fast_start, "Optimize for fast start");

                        let mut preserve_metadata = true; // TODO: Add to config
                        toggle_switch(ui, &mut preserve_metadata, "Preserve original metadata");
                    });
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            glass_card(ui, true, |ui| {
                tech_section(ui, "Configuration Management", Some(ModernTheme::NEON_GREEN), |ui| {
                    button_group(ui, |ui| {
                        if ui.add(success_button("💾 Save Settings")).clicked() {
                            save_config(ui_state, config.clone());
                        }

                        if ui.add(danger_button("Reset to Defaults")).clicked() {
                            reset_to_defaults(ui_state);
                        }
                    });

                    ui.add_space(StyleConstants::SPACING_SM);

                    ui.horizontal(|ui| {
                        if ui.add(small_button("📤 Export Config")).clicked() {
                            export_config(config.clone());
                        }

                        if ui.add(small_button("📥 Import Config")).clicked() {
                            import_config(ui_state, config.clone());
                        }
                    });

                    ui.add_space(StyleConstants::SPACING_SM);
                    ui.separator();
                    ui.add_space(StyleConstants::SPACING_SM);

                    ui.colored_label(
                        ModernTheme::TEXT_MUTED,
                        "Configuration is automatically saved when changed",
                    );
                });
            });
        });
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
