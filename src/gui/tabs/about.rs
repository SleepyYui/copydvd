use crate::gui::components::*;
use crate::gui::state::{UiState, UpdateStatus};
use crate::gui::theme::{
    animated_glass_card, get_device_type, glass_card, responsive_container, responsive_two_column,
    tech_section, ModernTheme, StyleConstants,
};
use std::sync::{Arc, Mutex};

/// Render the about tab with app information and update checking
pub fn render_about_tab(ui: &mut egui::Ui, ui_state: &mut UiState) {
    let screen_width = ui.available_width();
    let device_type = get_device_type(screen_width);

    responsive_container(ui, |ui| {
        responsive_two_column(
            ui,
            // Left column - App Information
            |ui| {
                animated_glass_card(ui, egui::Id::new("app_info"), true, |ui| {
                    tech_section(
                        ui,
                        "Application Information",
                        Some(ModernTheme::NEON_CYAN),
                        |ui| {
                            ui.vertical_centered(|ui| {
                                // App icon/logo placeholder
                                ui.add_space(StyleConstants::responsive_spacing(
                                    screen_width,
                                    StyleConstants::SPACING_MD,
                                ));
                                ui.label("🎬");
                                ui.add_space(StyleConstants::responsive_spacing(
                                    screen_width,
                                    StyleConstants::SPACING_SM,
                                ));

                                ui.heading("DVD Ripper");
                                ui.colored_label(
                                    ModernTheme::TEXT_SECONDARY,
                                    "Modern and fast DVD Ripping Solution",
                                );
                                ui.add_space(StyleConstants::responsive_spacing(
                                    screen_width,
                                    StyleConstants::SPACING_MD,
                                ));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Version:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.colored_label(
                                            ModernTheme::TEXT_SECONDARY,
                                            env!("CARGO_PKG_VERSION"),
                                        );
                                    },
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Build Date:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.colored_label(ModernTheme::TEXT_SECONDARY, "2025-05-24");
                                        // TODO: Get from build
                                    },
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Platform:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.colored_label(
                                            ModernTheme::TEXT_SECONDARY,
                                            format!(
                                                "{} ({})",
                                                std::env::consts::OS,
                                                std::env::consts::ARCH
                                            ),
                                        );
                                    },
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("GUI Framework:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.colored_label(
                                            ModernTheme::TEXT_SECONDARY,
                                            "egui + eframe",
                                        );
                                    },
                                );
                            });
                        },
                    );
                });

                ui.add_space(StyleConstants::responsive_spacing(
                    screen_width,
                    StyleConstants::SPACING_MD,
                ));

                animated_glass_card(ui, egui::Id::new("credits"), false, |ui| {
                    tech_section(ui, "Credits", Some(ModernTheme::NEON_PURPLE), |ui| {
                        ui.label("Developed by:");
                        ui.indent("developers", |ui| {
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "• SleepyYui");
                            //ui.colored_label(ModernTheme::TEXT_SECONDARY, "• Community Contributors");
                        });

                        ui.add_space(StyleConstants::responsive_spacing(
                            screen_width,
                            StyleConstants::SPACING_SM,
                        ));

                        ui.label("Built with:");
                        ui.indent("technologies", |ui| {
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "• Rust");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "• HandBrake");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "• egui UI Framework");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "• Tokio Async Runtime");
                        });

                        ui.add_space(StyleConstants::responsive_spacing(
                            screen_width,
                            StyleConstants::SPACING_SM,
                        ));

                        ui.label("Special Thanks:");
                        ui.indent("thanks", |ui| {
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "• HandBrake Project");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "• Rust Community");
                            /*ui.colored_label(
                                ModernTheme::TEXT_SECONDARY,
                                "• Open Source Contributors",
                            );*/
                        });
                    });
                });

                ui.add_space(StyleConstants::responsive_spacing(
                    screen_width,
                    StyleConstants::SPACING_MD,
                ));

                animated_glass_card(ui, egui::Id::new("license"), false, |ui| {
                    tech_section(ui, "License & Legal", Some(ModernTheme::NEON_PINK), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("License:");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.colored_label(ModernTheme::TEXT_SECONDARY, "MIT License");
                                },
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Copyright:");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.colored_label(
                                        ModernTheme::TEXT_SECONDARY,
                                        "© 2024 SleepyYui",
                                    );
                                },
                            );
                        });

                        ui.add_space(StyleConstants::responsive_spacing(
                            screen_width,
                            StyleConstants::SPACING_SM,
                        ));

                        if ui.add(small_button("📄 View License")).clicked() {
                            open_license();
                        }
                    });
                });
            },
            // Right column - Updates and System Info
            |ui| {
                animated_glass_card(ui, egui::Id::new("updates"), true, |ui| {
                    tech_section(
                        ui,
                        "Software Updates",
                        Some(ModernTheme::NEON_GREEN),
                        |ui| {
                            // Update status display
                            match &ui_state.update_status {
                                UpdateStatus::Unknown => {
                                    info_display(ui, "Update status unknown");
                                }
                                UpdateStatus::UpToDate => {
                                    success_display(ui, "✅ You have the latest version");
                                }
                                UpdateStatus::UpdateAvailable { version, url: _ } => {
                                    ui.colored_label(ModernTheme::SUCCESS, "🎉 Update available!");
                                    ui.colored_label(
                                        ModernTheme::TEXT_SECONDARY,
                                        format!("New version: {}", version),
                                    );

                                    ui.add_space(StyleConstants::responsive_spacing(
                                        screen_width,
                                        StyleConstants::SPACING_SM,
                                    ));

                                    button_group(ui, |ui| {
                                        if ui.add(primary_button("📥 Download Update")).clicked()
                                        {
                                            download_update(ui_state);
                                        }

                                        if ui.add(secondary_button("📋 View Changelog")).clicked()
                                        {
                                            view_changelog(ui_state);
                                        }
                                    });
                                }
                                UpdateStatus::Error(ref error) => {
                                    ui.colored_label(ModernTheme::ERROR, "❌ Update check failed:");
                                    ui.colored_label(ModernTheme::TEXT_SECONDARY, error);
                                }
                            }

                            ui.add_space(StyleConstants::responsive_spacing(
                                screen_width,
                                StyleConstants::SPACING_MD,
                            ));

                            button_group(ui, |ui| {
                                ui.add_enabled_ui(!ui_state.checking_updates, |ui| {
                                    if ui.add(secondary_button("🔍 Check for Updates")).clicked()
                                    {
                                        check_for_updates(ui_state);
                                    }
                                });

                                if ui_state.checking_updates {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label("Checking...");
                                    });
                                }
                            });

                            ui.add_space(StyleConstants::responsive_spacing(
                                screen_width,
                                StyleConstants::SPACING_SM,
                            ));
                            ui.separator();
                            ui.add_space(StyleConstants::responsive_spacing(
                                screen_width,
                                StyleConstants::SPACING_SM,
                            ));

                            ui.colored_label(
                                ModernTheme::TEXT_MUTED,
                                "Automatic update checking can be configured in settings",
                            );
                        },
                    );
                });

                ui.add_space(StyleConstants::responsive_spacing(
                    screen_width,
                    StyleConstants::SPACING_MD,
                ));

                animated_glass_card(ui, egui::Id::new("system_info"), false, |ui| {
                    tech_section(
                        ui,
                        "System Information",
                        Some(ModernTheme::NEON_BLUE),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Operating System:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.colored_label(
                                            ModernTheme::TEXT_SECONDARY,
                                            format!(
                                                "{} ({})",
                                                std::env::consts::OS,
                                                std::env::consts::ARCH
                                            ),
                                        );
                                    },
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("CPU Cores:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.colored_label(
                                            ModernTheme::TEXT_SECONDARY,
                                            num_cpus::get().to_string(),
                                        );
                                    },
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("GUI Backend:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.colored_label(
                                            ModernTheme::TEXT_SECONDARY,
                                            "egui + eframe",
                                        );
                                    },
                                );
                            });

                            ui.add_space(StyleConstants::responsive_spacing(
                                screen_width,
                                StyleConstants::SPACING_SM,
                            ));
                        },
                    );
                });

                ui.add_space(StyleConstants::responsive_spacing(
                    screen_width,
                    StyleConstants::SPACING_MD,
                ));

                animated_glass_card(ui, egui::Id::new("links"), false, |ui| {
                    tech_section(ui, "Links & Support", Some(ModernTheme::NEON_PINK), |ui| {
                        button_group(ui, |ui| {
                            if ui.add(secondary_button("🌐 GitHub Repository")).clicked() {
                                open_url("https://github.com/sleepyyui/copydvd");
                            }
                        });

                        ui.add_space(StyleConstants::responsive_spacing(
                            screen_width,
                            StyleConstants::SPACING_SM,
                        ));

                        button_group(ui, |ui| {
                            if ui.add(secondary_button("🐛 Report Bug")).clicked() {
                                open_url("https://github.com/sleepyyui/copydvd/issues/new");
                            }

                            if ui.add(secondary_button("💡 Feature Request")).clicked() {
                                open_url("https://github.com/sleepyyui/copydvd/issues/new");
                            }
                        });
                    });
                });
            },
        );
    });

    // Update dialog
    if ui_state.show_update_dialog {
        egui::Window::new("Software Update")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label("A new version of DVD Ripper is available!");

                if let UpdateStatus::UpdateAvailable { version, url: _ } = &ui_state.update_status {
                    ui.add_space(StyleConstants::SPACING_SM);
                    ui.colored_label(
                        ModernTheme::TEXT_SECONDARY,
                        format!("New version: {}", version),
                    );
                    ui.colored_label(
                        ModernTheme::TEXT_SECONDARY,
                        format!("Current version: {}", env!("CARGO_PKG_VERSION")),
                    );
                }

                ui.add_space(StyleConstants::SPACING_MD);

                // TODO: Implement automatic changelog getting and parsing
                ui.label("What's new:");
                ui.indent("changelog", |ui| {
                    ui.colored_label(
                        ModernTheme::TEXT_SECONDARY,
                        "• Improved performance and stability",
                    );
                    ui.colored_label(ModernTheme::TEXT_SECONDARY, "• New encoding options");
                    ui.colored_label(
                        ModernTheme::TEXT_SECONDARY,
                        "• Bug fixes and UI improvements",
                    );
                });

                ui.add_space(StyleConstants::SPACING_MD);

                ui.horizontal(|ui| {
                    if ui.add(primary_button("📥 Download Update")).clicked() {
                        download_update(ui_state);
                        ui_state.show_update_dialog = false;
                    }

                    if ui.add(secondary_button("📋 View Full Changelog")).clicked() {
                        view_changelog(ui_state);
                    }

                    if ui.add(secondary_button("Later")).clicked() {
                        ui_state.show_update_dialog = false;
                    }
                });
            });
    }
}

/// Check for software updates
fn check_for_updates(ui_state: &mut UiState) {
    ui_state.checking_updates = true;
    ui_state.set_status("Checking for updates...".to_string());

    // TODO: Implement actual update checking
    tokio::spawn(async move {
        // Update checking logic here
        // This should query GitHub releases API or similar
    });
}

/// Download software update
fn download_update(ui_state: &mut UiState) {
    ui_state.set_status("Preparing to download update...".to_string());

    // TODO: Implement update download
    if let UpdateStatus::UpdateAvailable { url, .. } = &ui_state.update_status {
        open_url(url);
    }
}

/// View changelog
fn view_changelog(ui_state: &UiState) {
    open_url("https://github.com/dvdripper/dvdripper/releases");
}

/// Open license file
fn open_license() {
    open_url("https://github.com/dvdripper/dvdripper/blob/main/LICENSE");
}

/// Open URL in default browser
fn open_url(url: &str) {
    if let Err(e) = webbrowser::open(url) {
        eprintln!("Failed to open URL {}: {}", url, e);
    }
}

/// Get OS information string
fn get_os_info() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Get memory information (placeholder)
fn get_memory_info() -> String {
    "N/A".to_string() // TODO: Implement memory detection
}

/// Get GUI backend information
fn get_gui_backend() -> String {
    #[cfg(feature = "wgpu")]
    return "wgpu".to_string();

    #[cfg(not(feature = "wgpu"))]
    return "glow".to_string();
}

/// Show detailed system information
fn show_detailed_system_info() {
    // TODO: Implement detailed system info dialog
}
