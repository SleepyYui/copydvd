use crate::config::Config;
use crate::gui::icons::svg_icon;
use crate::gui::notifications::{notify_error, notify_info, notify_success};
use crate::gui::state::UiState;
use crate::gui::theme::{
    modern_button, modern_card, modern_input, section_header, status_indicator, ButtonVariant,
    ModernTheme, Spacing, StatusType,
};
use std::sync::{Arc, Mutex};

pub fn render_server_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    // Header
    ui.heading(
        egui::RichText::new("Server Configuration")
            .size(24.0)
            .color(ModernTheme::TEXT_PRIMARY)
            .strong(),
    );

    ui.add_space(Spacing::XL);

    // Connection Settings
    render_connection_settings(ui, ui_state);

    ui.add_space(Spacing::XL);

    // Upload Settings
    render_upload_settings(ui, ui_state);

    ui.add_space(Spacing::XL);

    // Transfer Options
    render_transfer_options(ui, ui_state);

    ui.add_space(Spacing::XL);

    // Configuration Actions
    render_server_actions(ui, ui_state, config);
}

async fn test_connection(
    host: &str,
    port: u16,
    _username: &str,
    _password: &str,
    upload_method: &str,
) -> Result<(), String> {
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::time::timeout;

    tracing::info!(
        "Testing connection to {}:{} using {}",
        host,
        port,
        upload_method
    );

    // Basic TCP connectivity test
    let tcp_result = timeout(
        Duration::from_secs(10),
        TcpStream::connect(format!("{}:{}", host, port)),
    )
    .await;

    match tcp_result {
        Ok(Ok(stream)) => {
            drop(stream);
            tracing::info!("TCP connection successful to {}:{}", host, port);

            // For different upload methods, we could add specific protocol tests here
            match upload_method.to_lowercase().as_str() {
                "ssh" | "scp" | "sftp" => {
                    // In a full implementation, we would:
                    // 1. Use ssh2 crate to attempt SSH authentication
                    // 2. Test SFTP subsystem if needed
                    // 3. Verify permissions on target directory
                    tracing::info!("SSH/SFTP connection would be tested here");
                    Ok(())
                }
                "rsync" => {
                    // In a full implementation, we would:
                    // 1. Test rsync daemon connectivity if using rsync://
                    // 2. Test SSH connectivity if using rsync over SSH
                    tracing::info!("Rsync connection would be tested here");
                    Ok(())
                }
                _ => Err(format!("Unsupported upload method: {}", upload_method)),
            }
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to connect to {}:{}: {}", host, port, e);
            tracing::warn!("{}", error_msg);
            Err(error_msg)
        }
        Err(_) => {
            let error_msg = format!("Connection timeout to {}:{}", host, port);
            tracing::warn!("{}", error_msg);
            Err(error_msg)
        }
    }
}

fn render_connection_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    modern_card(ui, |ui| {
        section_header(ui, "Connection Settings", None::<fn(&mut egui::Ui)>);

        // Server host input
        ui.label(egui::RichText::new("Server host").color(ModernTheme::TEXT_SECONDARY));
        ui.add_space(Spacing::XS);
        modern_input(ui, &mut ui_state.config_temp.server_host, "example.com");
        ui.add_space(Spacing::MD);

        // Username input
        ui.label(egui::RichText::new("Username").color(ModernTheme::TEXT_SECONDARY));
        ui.add_space(Spacing::XS);
        modern_input(
            ui,
            &mut ui_state.config_temp.server_username,
            "your-username",
        );
        ui.add_space(Spacing::MD);

        // Password input
        ui.label(egui::RichText::new("Password").color(ModernTheme::TEXT_SECONDARY));
        ui.add_space(Spacing::XS);
        let mut password = ui_state.config_temp.server_password.clone();
        let password_edit = egui::TextEdit::singleline(&mut password)
            .password(true)
            .hint_text(egui::RichText::new("your-password").color(ModernTheme::TEXT_TERTIARY))
            .desired_width(ui.available_width())
            .margin(egui::Vec2::new(Spacing::MD, Spacing::SM));
        ui.add(password_edit);
        ui_state.config_temp.server_password = password;
        ui.add_space(Spacing::MD);

        // Remote path input
        ui.label(egui::RichText::new("Remote path").color(ModernTheme::TEXT_SECONDARY));
        ui.add_space(Spacing::XS);
        modern_input(
            ui,
            &mut ui_state.config_temp.server_path,
            "/path/to/destination",
        );
        ui.add_space(Spacing::LG);

        // Test connection button
        if modern_button(ui, "Test Connection", ButtonVariant::Primary).clicked() {
            test_server_connection(ui_state);
        }
    });
}

fn render_upload_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    modern_card(ui, |ui| {
        section_header(ui, "Upload Settings", None::<fn(&mut egui::Ui)>);

        // Compress files during transfer
        ui.checkbox(
            &mut ui_state.config_temp.compress_transfer,
            egui::RichText::new("Compress files during transfer").color(ModernTheme::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::SM);

        // Resume interrupted uploads
        ui.checkbox(
            &mut ui_state.config_temp.resume_uploads,
            egui::RichText::new("Resume interrupted uploads").color(ModernTheme::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::SM);

        // Preserve file permissions
        ui.checkbox(
            &mut ui_state.config_temp.preserve_permissions,
            egui::RichText::new("Preserve file permissions").color(ModernTheme::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::SM);

        // Delete local files after successful upload
        ui.checkbox(
            &mut ui_state.config_temp.delete_after_upload,
            egui::RichText::new("Delete local files after successful upload")
                .color(ModernTheme::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::SM);
    });
}

fn render_transfer_options(ui: &mut egui::Ui, ui_state: &mut UiState) {
    modern_card(ui, |ui| {
        section_header(ui, "Transfer Status", None::<fn(&mut egui::Ui)>);

        if ui_state.upload_to_server {
            status_indicator(ui, "Upload enabled", StatusType::Success);
        } else {
            status_indicator(ui, "Upload disabled", StatusType::Info);
        }

        ui.add_space(Spacing::MD);

        ui.checkbox(
            &mut ui_state.upload_to_server,
            egui::RichText::new("Enable automatic upload after ripping")
                .color(ModernTheme::TEXT_PRIMARY),
        );

        if ui_state.upload_to_server && !ui_state.config_temp.server_host.is_empty() {
            ui.add_space(Spacing::MD);
            ui.horizontal(|ui| {
                svg_icon(ui, "arrow-right", 14.0, ModernTheme::TEXT_SECONDARY);
                ui.label(
                    egui::RichText::new(format!(
                        "Files will be uploaded to: {}",
                        ui_state.config_temp.server_host
                    ))
                    .color(ModernTheme::TEXT_SECONDARY),
                );
            });
        }
    });
}

fn render_server_actions(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    modern_card(ui, |ui| {
        section_header(ui, "Actions", None::<fn(&mut egui::Ui)>);

        // Save settings button
        if modern_button(ui, "Save Server Settings", ButtonVariant::Success).clicked() {
            save_server_config(ui_state, config.clone());
        }

        ui.add_space(Spacing::MD);

        // Load settings button
        if modern_button(ui, "Load Server Settings", ButtonVariant::Secondary).clicked() {
            load_server_config(ui_state, config.clone());
        }

        ui.add_space(Spacing::MD);

        // Clear settings button
        if modern_button(ui, "Clear Settings", ButtonVariant::Warning).clicked() {
            clear_server_settings(ui_state);
        }
    });
}

fn test_server_connection(ui_state: &mut UiState) {
    if ui_state.config_temp.server_host.is_empty() {
        notify_error("Please enter a server host");
        return;
    }

    if ui_state.config_temp.server_username.is_empty() {
        notify_error("Please enter a username");
        return;
    }

    notify_info("Testing server connection...");

    // Clone data for async operation
    let host = ui_state.config_temp.server_host.clone();
    let username = ui_state.config_temp.server_username.clone();
    let password = ui_state.config_temp.server_password.clone();
    let port = 22; // Default SSH port
    let upload_method = "ssh".to_string(); // Default upload method

    tokio::spawn(async move {
        let result = test_connection(&host, port, &username, &password, &upload_method).await;

        match result {
            Ok(_) => notify_success("Server connection test successful"),
            Err(e) => notify_error(&format!("Connection test failed: {}", e)),
        }
    });
}

fn save_server_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(mut config) = config.try_lock() {
        // Create server config from UI state
        let server_config = crate::config::ServerConfig {
            host: ui_state.config_temp.server_host.clone(),
            username: ui_state.config_temp.server_username.clone(),
            password: if ui_state.config_temp.server_password.is_empty() {
                None
            } else {
                Some(ui_state.config_temp.server_password.clone())
            },
            path: ui_state.config_temp.server_path.clone(),
        };

        // Update server config
        config.server = Some(server_config);

        if let Err(e) = config.save() {
            notify_error(&format!("Failed to save server config: {}", e));
        } else {
            notify_success("Server configuration saved successfully");
        }
    } else {
        notify_error("Failed to access configuration");
    }
}

fn load_server_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(config) = config.try_lock() {
        // Load server config into UI state
        if let Some(server) = &config.server {
            ui_state.config_temp.server_host = server.host.clone();
            ui_state.config_temp.server_username = server.username.clone();
            ui_state.config_temp.server_password = server.password.clone().unwrap_or_default();
            ui_state.config_temp.server_path = server.path.clone();
        } else {
            // Clear UI if no server config exists
            ui_state.config_temp.server_host.clear();
            ui_state.config_temp.server_username.clear();
            ui_state.config_temp.server_password.clear();
            ui_state.config_temp.server_path.clear();
        }

        // Also load other config fields
        ui_state.load_config_temp(&config);
        notify_success("Server configuration loaded successfully");
    } else {
        notify_error("Failed to load server configuration");
    }
}

fn clear_server_settings(ui_state: &mut UiState) {
    ui_state.config_temp.server_host.clear();
    ui_state.config_temp.server_username.clear();
    ui_state.config_temp.server_password.clear();
    ui_state.config_temp.server_path.clear();
    ui_state.upload_to_server = false;

    notify_success("Server settings cleared");
}
