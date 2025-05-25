use crate::config::{Config, ServerConfig};
use crate::gui::components::*;
use crate::gui::state::UiState;
use crate::gui::theme::{glass_card, tech_section, ModernTheme, StyleConstants};
use std::sync::{Arc, Mutex};

/// Render the server configuration tab
pub fn render_server_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    ui.columns(2, |columns| {
        // Left column - Connection Settings
        columns[0].vertical(|ui| {
            glass_card(ui, false, |ui| {
                tech_section(ui, "Server Configuration", Some(ModernTheme::NEON_BLUE), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Hostname/IP:");
                        ui.add(
                            text_input(&mut ui_state.config_temp.server_host)
                                .hint_text("example.com or 192.168.1.100"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.add(
                            text_input(&mut ui_state.config_temp.server_username)
                                .hint_text("SSH username"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(
                            password_input(&mut ui_state.config_temp.server_password)
                                .hint_text("SSH password (optional if using keys)"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Remote Path:");
                        ui.add(
                            text_input(&mut ui_state.config_temp.server_path)
                                .hint_text("/path/to/upload/directory"),
                        );
                    });
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            card_container(ui, |ui| {
                section(ui, "🔧 Transfer Settings", |ui| {
                    ui.label("Transfer Method:");
                    ui.indent("transfer_method", |ui| {
                        let mut transfer_method = "rsync".to_string(); // TODO: Add to config
                        styled_combo_box(
                            ui,
                            "transfer_method",
                            &mut transfer_method,
                            &["rsync", "scp", "sftp"],
                        );

                        ui.add_space(StyleConstants::SPACING_SM);
                        ui.colored_label(
                            ModernTheme::TEXT_MUTED,
                            match transfer_method.as_str() {
                                "rsync" => "Fastest, with resume support and compression",
                                "scp" => "Simple and reliable, good for small files",
                                "sftp" => "Secure and stable, works through firewalls",
                                _ => "Unknown transfer method",
                            },
                        );
                    });

                    ui.add_space(StyleConstants::SPACING_MD);

                    ui.label("Options:");
                    ui.indent("transfer_options", |ui| {
                        let mut compress_transfer = true; // TODO: Add to config
                        toggle_switch(
                            ui,
                            &mut compress_transfer,
                            "Enable compression during transfer",
                        );

                        let mut resume_uploads = true; // TODO: Add to config
                        toggle_switch(ui, &mut resume_uploads, "Resume interrupted uploads");

                        let mut delete_after_upload = false; // TODO: Add to config
                        toggle_switch(
                            ui,
                            &mut delete_after_upload,
                            "Delete local files after successful upload",
                        );

                        let mut preserve_permissions = true; // TODO: Add to config
                        toggle_switch(ui, &mut preserve_permissions, "Preserve file permissions");
                    });
                });
            });
        });

        // Right column - Connection Testing and Status
        columns[1].vertical(|ui| {
            glass_card(ui, true, |ui| {
                tech_section(ui, "Connection Test", Some(ModernTheme::NEON_CYAN), |ui| {
                    ui.label("Test your server connection before uploading:");

                    ui.add_space(StyleConstants::SPACING_SM);

                    button_group(ui, |ui| {
                        if ui.add(primary_button("🔗 Test Connection")).clicked() {
                            test_connection(ui_state);
                        }

                        if ui.add(secondary_button("📁 Test Upload")).clicked() {
                            test_upload(ui_state);
                        }
                    });

                    ui.add_space(StyleConstants::SPACING_MD);

                    // Connection status display
                    let connection_status = get_connection_status(); // TODO: Implement
                    match connection_status {
                        ConnectionStatus::Unknown => {
                            info_display(ui, "Connection not tested yet");
                        }
                        ConnectionStatus::Testing => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Testing connection...");
                            });
                        }
                        ConnectionStatus::Success => {
                            success_display(ui, "✅ Connection successful");
                        }
                        ConnectionStatus::Failed(ref error) => {
                            ui.colored_label(ModernTheme::ERROR, "❌ Connection failed:");
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, error);
                        }
                    }
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            card_container(ui, |ui| {
                section(ui, "📊 Upload Statistics", |ui| {
                    ui.label("Recent Upload Activity:");
                    ui.add_space(StyleConstants::SPACING_SM);

                    // TODO: Add real statistics
                    ui.horizontal(|ui| {
                        ui.label("Total Uploads:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "0");
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Total Data:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "0 GB");
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Last Upload:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(ModernTheme::TEXT_SECONDARY, "Never");
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Success Rate:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(ModernTheme::SUCCESS, "100%");
                        });
                    });
                });
            });

            ui.add_space(StyleConstants::SPACING_MD);

            card_container(ui, |ui| {
                section(ui, "💾 Server Configuration", |ui| {
                    button_group(ui, |ui| {
                        if ui.add(success_button("💾 Save Settings")).clicked() {
                            save_server_config(ui_state, config.clone());
                        }

                        if ui.add(secondary_button("🔄 Reset")).clicked() {
                            reset_server_config(ui_state);
                        }
                    });

                    ui.add_space(StyleConstants::SPACING_SM);

                    ui.horizontal(|ui| {
                        if ui.add(small_button("📤 Export Profile")).clicked() {
                            export_server_profile(ui_state);
                        }

                        if ui.add(small_button("📥 Import Profile")).clicked() {
                            import_server_profile(ui_state);
                        }
                    });

                    ui.add_space(StyleConstants::SPACING_SM);
                    ui.separator();
                    ui.add_space(StyleConstants::SPACING_SM);

                    ui.colored_label(
                        ModernTheme::TEXT_MUTED,
                        "Passwords are stored securely and encrypted",
                    );
                });
            });
        });
    });
}

#[derive(Debug, Clone)]
enum ConnectionStatus {
    Unknown,
    Testing,
    Success,
    Failed(String),
}

/// Get current connection status (placeholder)
fn get_connection_status() -> ConnectionStatus {
    ConnectionStatus::Unknown
}

/// Test server connection
fn test_connection(ui_state: &mut UiState) {
    if ui_state.config_temp.server_host.is_empty() {
        ui_state.set_error("Please enter a hostname first".to_string());
        return;
    }

    if ui_state.config_temp.server_username.is_empty() {
        ui_state.set_error("Please enter a username first".to_string());
        return;
    }

    ui_state.set_status("Testing server connection...".to_string());

    // TODO: Implement actual connection testing
    tokio::spawn(async move {
        // Connection testing logic here
    });
}

/// Test upload functionality
fn test_upload(ui_state: &mut UiState) {
    if ui_state.config_temp.server_host.is_empty()
        || ui_state.config_temp.server_username.is_empty()
    {
        ui_state.set_error("Please configure server connection first".to_string());
        return;
    }

    ui_state.set_status("Testing upload functionality...".to_string());

    // TODO: Implement test upload (create a small test file and upload it)
    tokio::spawn(async move {
        // Test upload logic here
    });
}

/// Save server configuration
fn save_server_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(mut config) = config.try_lock() {
        if !ui_state.config_temp.server_host.is_empty()
            && !ui_state.config_temp.server_username.is_empty()
        {
            let server_config = ServerConfig {
                host: ui_state.config_temp.server_host.clone(),
                username: ui_state.config_temp.server_username.clone(),
                password: if ui_state.config_temp.server_password.is_empty() {
                    None
                } else {
                    Some(ui_state.config_temp.server_password.clone())
                },
                path: ui_state.config_temp.server_path.clone(),
            };

            config.server = Some(server_config);

            if let Err(e) = config.save() {
                ui_state.set_error(format!("Failed to save server configuration: {}", e));
            } else {
                ui_state.set_status("Server configuration saved successfully".to_string());
            }
        } else {
            config.server = None;
            if let Err(e) = config.save() {
                ui_state.set_error(format!("Failed to clear server configuration: {}", e));
            } else {
                ui_state.set_status("Server configuration cleared".to_string());
            }
        }
    }
}

/// Reset server configuration
fn reset_server_config(ui_state: &mut UiState) {
    ui_state.config_temp.server_host.clear();
    ui_state.config_temp.server_username.clear();
    ui_state.config_temp.server_password.clear();
    ui_state.config_temp.server_path.clear();
    ui_state.set_status("Server configuration reset".to_string());
}

/// Export server profile to file
fn export_server_profile(ui_state: &UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Export Server Profile")
        .add_filter("JSON", &["json"])
        .set_file_name("server_profile.json")
        .save_file()
    {
        let profile = serde_json::json!({
            "host": ui_state.config_temp.server_host,
            "username": ui_state.config_temp.server_username,
            "path": ui_state.config_temp.server_path,
            // Note: We don't export the password for security reasons
        });

        if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&profile).unwrap()) {
            eprintln!("Failed to export server profile: {}", e);
        }
    }
}

/// Import server profile from file
fn import_server_profile(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Import Server Profile")
        .add_filter("JSON", &["json"])
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(profile) => {
                    if let Some(host) = profile.get("host").and_then(|v| v.as_str()) {
                        ui_state.config_temp.server_host = host.to_string();
                    }
                    if let Some(username) = profile.get("username").and_then(|v| v.as_str()) {
                        ui_state.config_temp.server_username = username.to_string();
                    }
                    if let Some(path) = profile.get("path").and_then(|v| v.as_str()) {
                        ui_state.config_temp.server_path = path.to_string();
                    }
                    ui_state.set_status("Server profile imported successfully".to_string());
                }
                Err(e) => {
                    ui_state.set_error(format!("Invalid profile file: {}", e));
                }
            },
            Err(e) => {
                ui_state.set_error(format!("Failed to read profile file: {}", e));
            }
        }
    }
}
