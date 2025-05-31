use crate::gui::state::UiState;
use crate::gui::theme::{Layout, styled_panel, grouped_section};

pub fn render_about_tab(ui: &mut egui::Ui, _ui_state: &mut UiState) {
    ui.heading("About Copy DVD");
    ui.separator();
    
    // Application Information
    styled_panel(ui, |ui| {
        grouped_section(ui, "Application Information", |ui| {
            ui.label("Copy DVD - Simple DVD Copying Tool");
            ui.label("Version: 0.1.0");
            ui.label("Built with Rust and egui");
            
            ui.add_space(Layout::SPACING);
            
            ui.label("A simple, functional tool for copying DVDs to digital formats.");
            ui.label("Designed with simplicity and ease of use in mind.");
        });
    });
    
    ui.add_space(Layout::SPACING_LARGE);
    
    // System Information
    styled_panel(ui, |ui| {
        grouped_section(ui, "System Information", |ui| {
            ui.horizontal(|ui| {
                ui.label("Operating System:");
                ui.label(std::env::consts::OS);
            });
            
            ui.horizontal(|ui| {
                ui.label("Architecture:");
                ui.label(std::env::consts::ARCH);
            });
            
            ui.horizontal(|ui| {
                ui.label("GUI Framework:");
                ui.label("egui");
            });
        });
    });
    
    ui.add_space(Layout::SPACING_LARGE);
    
    // Features
    styled_panel(ui, |ui| {
        grouped_section(ui, "Features", |ui| {
            ui.label("• DVD detection and scanning");
            ui.label("• Title and chapter selection");
            ui.label("• Multiple output formats");
            ui.label("• HandBrake integration");
            ui.label("• Server upload support");
            ui.label("• Simple, clean interface");
        });
    });
    
    ui.add_space(Layout::SPACING_LARGE);
    
    // Support
    styled_panel(ui, |ui| {
        grouped_section(ui, "Support & Links", |ui| {
            if ui.link("HandBrake Official Website").clicked() {
                let _ = open::that("https://handbrake.fr/");
            }
            
            if ui.link("Rust Programming Language").clicked() {
                let _ = open::that("https://www.rust-lang.org/");
            }
            
            if ui.link("egui GUI Framework").clicked() {
                let _ = open::that("https://github.com/emilk/egui");
            }
        });
    });
}