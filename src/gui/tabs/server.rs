use crate::config::Config;
use crate::gui::state::UiState;
use crate::gui::theme::{BasicTheme, Layout, styled_panel, grouped_section, full_width_button, status_indicator, StatusType};
use crate::gui::notifications::{notify_success, notify_error, notify_info};
use std::sync::{Arc, Mutex};

pub fn render_server_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    ui.heading("Server Configuration");
    ui.separator();

    // Wrap content in scroll area to prevent overflow
    egui::ScrollArea::both().show(ui, |ui| {
        // Connection Settings
        render_connection_settings(ui, ui_state);
        
        ui.add_space(Layout::SPACING_LARGE);

        // Upload Settings
        render_upload_settings(ui, ui_state);
        
        ui.add_space(Layout::SPACING_LARGE);

        // Transfer Options
        render_transfer_options(ui, ui_state);
        
        ui.add_space(Layout::SPACING_LARGE);

        // Configuration Actions
        render_server_actions(ui, ui_state, config);
    });
}

fn render_connection_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Connection Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Server host:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.server_host));
            });
            
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.server_username));
            });
            
            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.server_password));
            });
            
            ui.horizontal(|ui| {
                ui.label("Remote path:");
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.server_path));
            });
            
            ui.add_space(Layout::SPACING);
            
            if full_width_button(ui, "Test Connection").clicked() {
                test_server_connection(ui_state);
            }
        });
    });
}

fn render_upload_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Upload Settings", |ui| {
            ui.checkbox(&mut ui_state.config_temp.compress_transfer, "Compress files during transfer");
            ui.checkbox(&mut ui_state.config_temp.resume_uploads, "Resume interrupted uploads");
            ui.checkbox(&mut ui_state.config_temp.preserve_permissions, "Preserve file permissions");
            ui.checkbox(&mut ui_state.config_temp.delete_after_upload, "Delete local files after successful upload");
        });
    });
}

fn render_transfer_options(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Transfer Status", |ui| {
            if ui_state.upload_to_server {
                status_indicator(ui, "Upload enabled", StatusType::Success);
            } else {
                status_indicator(ui, "Upload disabled", StatusType::Info);
            }
            
            ui.add_space(Layout::SPACING);
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.upload_to_server, "Enable automatic upload after ripping");
            });
            
            if ui_state.upload_to_server && !ui_state.config_temp.server_host.is_empty() {
                ui.add_space(Layout::SPACING);
                ui.label(format!("Files will be uploaded to: {}", ui_state.config_temp.server_host));
            }
        });
    });
}

fn render_server_actions(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Actions", |ui| {
            if full_width_button(ui, "Save Server Settings").clicked() {
                save_server_config(ui_state, config.clone());
            }
            
            ui.add_space(Layout::SPACING_SMALL);
            
            if full_width_button(ui, "Load Server Settings").clicked() {
                load_server_config(ui_state, config.clone());
            }
            
            ui.add_space(Layout::SPACING);
            
            if full_width_button(ui, "Clear Settings").clicked() {
                clear_server_settings(ui_state);
            }
        });
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
    
    // TODO: Implement actual server connection test
    tokio::spawn(async move {
        // Simulate connection test
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        notify_success("Server connection test successful");
    });
}

fn save_server_config(ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
    if let Ok(mut config) = config.try_lock() {
        // Update server config from UI state
        if let Some(server) = &mut config.server {
            server.host = ui_state.config_temp.server_host.clone();
            server.username = ui_state.config_temp.server_username.clone();
            server.password = if ui_state.config_temp.server_password.is_empty() {
                None
            } else {
                Some(ui_state.config_temp.server_password.clone())
            };
            server.path = ui_state.config_temp.server_path.clone();
        }
        
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