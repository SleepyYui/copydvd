use crate::config::Config;
use crate::gui::components::*;
use crate::gui::state::UiState;
use crate::gui::theme::{glass_card, tech_section, ModernTheme, StyleConstants, neon_progress_bar};
use crate::handbrake_manager::HandBrakeManager;
use std::sync::{Arc, Mutex};

/// Render the HandBrake configuration tab
pub fn render_handbrake_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    ui.columns(2, |columns| {
        // Left column - HandBrake Management
        columns[0].vertical(|ui| {
            glass_card(ui, false, |ui| {
                tech_section(ui, "HandBrake Management", Some(ModernTheme::NEON_PURPLE), |ui| {
                    toggle_switch(
                        ui,
                        &mut ui_state.config_temp.auto_download,
                        "Auto-download HandBrake if not found",
                    );

                    ui.add_space(StyleConstants::SPACING_XS);
                    ui.colored_label(
                        ModernTheme::TEXT_MUTED,
                        "Automatically download HandBrake when needed",
                    );

                    ui.add_space(StyleConstants::SPACING_SM);

                    toggle_switch(
                        ui,
                        &mut ui_state.config_temp.prefer_system,
                        "Prefer system HandBrake over managed version",
                    );

                    ui.add_space(StyleConstants::SPACING_XS);
                    ui.colored_label(
                        ModernTheme::TEXT_MUTED,
                        "Use system-installed HandBrake if available",
                    );

                    ui.add_space(StyleConstants::SPACING_SM);

                    toggle_switch(
                        ui,
                        &mut ui_state.config_temp.verify_on_startup,
                        "Verify HandBrake on startup",
                    );

                    ui.add_space(StyleConstants::SPACING_XS);
                    ui.colored_label(
                        ModernTheme::TEXT_MUTED,
                        "Check HandBrake availability when app starts",
                    );
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            glass_card(ui, false, |ui| {
                tech_section(ui, "Cache Management", Some(ModernTheme::NEON_CYAN), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Max cache size (MB):");
                        ui.add(
                            text_input(&mut ui_state.config_temp.max_cache_size_mb)
                                .hint_text("0 = unlimited"),
                        );
                        ui.colored_label(ModernTheme::TEXT_MUTED, "(0 = unlimited)");
                    });

                    ui.add_space(StyleConstants::SPACING_MD);

                    // Cache information display
                    ui.label("Cache Information:");
                    ui.indent("cache_info", |ui| {
                        if let Some((cache_dir, cache_size)) = &ui_state.config_temp.cache_info {
                            ui.horizontal(|ui| {
                                ui.label("Directory:");
                                ui.colored_label(ModernTheme::TEXT_SECONDARY, cache_dir);
                            });

                            ui.horizontal(|ui| {
                                ui.label("Current size:");
                                ui.colored_label(ModernTheme::TEXT_SECONDARY, cache_size);
                            });
                        } else {
                            ui.colored_label(
                                ModernTheme::TEXT_MUTED,
                                "Cache information not available",
                            );
                        }
                    });

                    ui.add_space(StyleConstants::SPACING_MD);

                    button_group(ui, |ui| {
                        if ui.add(secondary_button("🔄 Refresh Info")).clicked() {
                            refresh_cache_info(ui_state);
                        }

                        if ui.add(danger_button("🗑️ Clear Cache")).clicked() {
                            ui_state.show_cache_clear_dialog = true;
                        }
                    });
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            card_container(ui, |ui| {
                section(ui, "⬬ Download Management", |ui| {
                    ui.label("HandBrake Status:");
                    ui.add_space(StyleConstants::SPACING_SM);

                    let handbrake_status = check_handbrake_status(); // TODO: Implement
                    match handbrake_status {
                        HandBrakeStatus::SystemAvailable(ref path) => {
                            success_display(ui, &format!("✅ System HandBrake found: {}", path));
                        }
                        HandBrakeStatus::ManagedAvailable(ref version) => {
                            success_display(
                                ui,
                                &format!("✅ Managed HandBrake available: {}", version),
                            );
                        }
                        HandBrakeStatus::NotFound => {
                            warning_display(ui, "⚠️ HandBrake not found");
                        }
                        HandBrakeStatus::Downloading(progress) => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Downloading HandBrake...");
                            });
                            styled_progress_bar(
                                ui,
                                progress,
                                Some(&format!("{:.1}%", progress * 100.0)),
                            );
                        }
                        HandBrakeStatus::Error(ref error) => {
                            ui.colored_label(ModernTheme::ERROR, "❌ Error:");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, error);
                        }
                    }

                    ui.add_space(StyleConstants::SPACING_MD);

                    button_group(ui, |ui| {
                        if ui.add(primary_button("🔍 Check Status")).clicked() {
                            check_handbrake_availability(ui_state);
                        }

                        if ui.add(secondary_button("⬬ Force Re-download")).clicked() {
                            force_redownload_handbrake(ui_state);
                        }
                    });
                });
            });
        });

        // Right column - Advanced Settings and Information
        columns[1].vertical(|ui| {
            card_container(ui, |ui| {
                section(ui, "⚙️ Advanced Settings", |ui| {
                    ui.label("HandBrake CLI Options:");
                    ui.indent("cli_options", |ui| {
                        let mut custom_args = String::new(); // TODO: Add to config
                        ui.horizontal(|ui| {
                            ui.label("Custom arguments:");
                            ui.add(
                                text_input(&mut custom_args)
                                    .hint_text("--preset 'Fast 1080p30' --encoder x264"),
                            );
                        });

                        ui.add_space(StyleConstants::SPACING_SM);
                        ui.colored_label(
                            ModernTheme::TEXT_MUTED,
                            "Additional command-line arguments for HandBrake",
                        );
                    });

                    ui.add_space(StyleConstants::SPACING_MD);

                    ui.label("Quality Presets:");
                    ui.indent("quality_presets", |ui| {
                        let mut quality_preset = "High Quality".to_string(); // TODO: Add to config
                        styled_combo_box(
                            ui,
                            "quality_preset",
                            &mut quality_preset,
                            &[
                                "Very Fast 1080p30",
                                "Fast 1080p30",
                                "High Quality",
                                "Super HQ 1080p30",
                                "Custom",
                            ],
                        );

                        ui.add_space(StyleConstants::SPACING_SM);
                        ui.colored_label(
                            ModernTheme::TEXT_MUTED,
                            "HandBrake quality preset to use for encoding",
                        );
                    });
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            glass_card(ui, true, |ui| {
                tech_section(ui, "System Information", Some(ModernTheme::NEON_GREEN), |ui| {
                    ui.label("Platform Support:");
                    ui.indent("platform_info", |ui| {
                        let platform = std::env::consts::OS;
                        let arch = std::env::consts::ARCH;

                        ui.horizontal(|ui| {
                            ui.label("Operating System:");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, platform);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Architecture:");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, arch);
                        });

                        let supported = matches!(
                            (platform, arch),
                            ("windows", "x86_64")
                                | ("windows", "aarch64")
                                | ("macos", _)
                                | ("linux", "x86_64")
                        );

                        ui.horizontal(|ui| {
                            ui.label("Auto-download supported:");
                            if supported {
                                ui.colored_label(ModernTheme::SUCCESS, "✅ Yes");
                            } else {
                                ui.colored_label(ModernTheme::WARNING, "⚠️ Limited");
                            }
                        });
                    });

                    ui.add_space(StyleConstants::SPACING_MD);

                    ui.label("Download Information:");
                    ui.indent("download_info", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Source:");
                            ui.colored_label(
                                ModernTheme::TEXT_SECONDARY,
                                "Official HandBrake releases",
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Latest version:");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "1.7.3");
                            // TODO: Get from API
                        });

                        ui.horizontal(|ui| {
                            ui.label("Size (approx):");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "~15-25 MB");
                        });
                    });
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            card_container(ui, |ui| {
                section(ui, "💾 Configuration", |ui| {
                    button_group(ui, |ui| {
                        if ui.add(success_button("💾 Save Settings")).clicked() {
                            save_handbrake_config(ui_state, config.clone());
                        }

                        if ui.add(secondary_button("🔄 Reset")).clicked() {
                            reset_handbrake_config(ui_state);
                        }
                    });

                    ui.add_space(StyleConstants::SPACING_SM);
                    ui.separator();
                    ui.add_space(StyleConstants::SPACING_SM);

                    ui.colored_label(
                        ModernTheme::TEXT_MUTED,
                        "Changes are applied immediately and saved automatically",
                    );
                });
            });
        });
    });

    // Cache clear confirmation dialog
    if ui_state.show_cache_clear_dialog {
        egui::Window::new("Clear Cache")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label("Are you sure you want to clear the HandBrake cache?");
                ui.add_space(StyleConstants::SPACING_SM);
                ui.colored_label(
                    ModernTheme::TEXT_MUTED,
                    "This will delete all downloaded HandBrake files.",
                );
                ui.add_space(StyleConstants::SPACING_MD);

                ui.horizontal(|ui| {
                    if ui.add(danger_button("Clear Cache")).clicked() {
                        clear_handbrake_cache(ui_state);
                        ui_state.show_cache_clear_dialog = false;
                    }

                    if ui.add(secondary_button("Cancel")).clicked() {
                        ui_state.show_cache_clear_dialog = false;
                    }
                });
            });
    }
}

#[derive(Debug, Clone)]
enum HandBrakeStatus {
    SystemAvailable(String),
    ManagedAvailable(String),
    NotFound,
    Downloading(f32),
    Error(String),
}

/// Check HandBrake status (placeholder)
fn check_handbrake_status() -> HandBrakeStatus {
    HandBrakeStatus::NotFound
}

/// Refresh cache information
fn refresh_cache_info(ui_state: &mut UiState) {
    // TODO: Implement cache info refresh
    ui_state.set_status("Refreshing cache information...".to_string());

    tokio::spawn(async move {
        // Cache info refresh logic here
    });
}

/// Check HandBrake availability
fn check_handbrake_availability(ui_state: &mut UiState) {
    ui_state.set_status("Checking HandBrake availability...".to_string());

    // TODO: Implement HandBrake availability check
    tokio::spawn(async move {
        // Availability check logic here
    });
}

/// Force re-download of HandBrake
fn force_redownload_handbrake(ui_state: &mut UiState) {
    ui_state.set_status("Starting HandBrake re-download...".to_string());

    // TODO: Implement forced re-download
    tokio::spawn(async move {
        // Re-download logic here
    });
}

/// Clear HandBrake cache
fn clear_handbrake_cache(ui_state: &mut UiState) {
    ui_state.set_status("Clearing HandBrake cache...".to_string());

    // TODO: Implement cache clearing
    tokio::spawn(async move {
        // Cache clearing logic here
    });
}

/// Save HandBrake configuration
fn save_handbrake_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(mut config) = config.try_lock() {
        config.handbrake_management.auto_download = ui_state.config_temp.auto_download;
        config.handbrake_management.prefer_system = ui_state.config_temp.prefer_system;
        config.handbrake_management.verify_on_startup = ui_state.config_temp.verify_on_startup;

        if let Ok(cache_size) = ui_state.config_temp.max_cache_size_mb.parse::<u64>() {
            config.handbrake_management.max_cache_size_mb = cache_size;
        }

        if let Err(e) = config.save() {
            ui_state.set_error(format!("Failed to save HandBrake configuration: {}", e));
        } else {
            ui_state.set_status("HandBrake configuration saved successfully".to_string());
        }
    }
}

/// Reset HandBrake configuration to defaults
fn reset_handbrake_config(ui_state: &mut UiState) {
    ui_state.config_temp.auto_download = true;
    ui_state.config_temp.prefer_system = true;
    ui_state.config_temp.verify_on_startup = true;
    ui_state.config_temp.max_cache_size_mb = "100".to_string();
    ui_state.set_status("HandBrake configuration reset to defaults".to_string());
}
