use crate::gui::state::UiState;
use crate::gui::theme::{
    modern_card, section_header, ModernTheme, Spacing,
};

pub fn render_about_tab(ui: &mut egui::Ui, _ui_state: &mut UiState) {
    // Header
    ui.heading(
        egui::RichText::new("About CopyDVD")
            .size(24.0)
            .color(ModernTheme::TEXT_PRIMARY)
            .strong(),
    );

    ui.add_space(Spacing::XL);

    // Application Information
    modern_card(ui, |ui| {
        section_header(ui, "Application Information", None::<fn(&mut egui::Ui)>);

        ui.label(
            egui::RichText::new("CopyDVD - Simple DVD Copy Tool")
                .color(ModernTheme::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::SM);

        ui.label(
            egui::RichText::new(&format!("Version: {}", env!("CARGO_PKG_VERSION")))
                .color(ModernTheme::TEXT_PRIMARY),
        );
        ui.add_space(Spacing::SM);

        ui.label(
            egui::RichText::new("Built with Rust and egui")
                .color(ModernTheme::TEXT_PRIMARY),
        );

        ui.add_space(Spacing::MD);

        ui.label(
            egui::RichText::new("A simple, functional tool for copying DVDs to digital formats.")
                .color(ModernTheme::TEXT_SECONDARY),
        );
        ui.label(
            egui::RichText::new("Designed with simplicity and ease of use in mind.")
                .color(ModernTheme::TEXT_SECONDARY),
        );
    });

    ui.add_space(Spacing::XL);

    // System Information
    modern_card(ui, |ui| {
        section_header(ui, "System Information", None::<fn(&mut egui::Ui)>);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Operating System:").color(ModernTheme::TEXT_SECONDARY));
            ui.add_space(Spacing::MD);
            ui.label(
                egui::RichText::new(std::env::consts::OS)
                    .color(ModernTheme::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.add_space(Spacing::SM);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Architecture:").color(ModernTheme::TEXT_SECONDARY));
            ui.add_space(Spacing::MD);
            ui.label(
                egui::RichText::new(std::env::consts::ARCH)
                    .color(ModernTheme::TEXT_PRIMARY)
                    .strong(),
            );
        });
        ui.add_space(Spacing::SM);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("GUI Framework:").color(ModernTheme::TEXT_SECONDARY));
            ui.add_space(Spacing::MD);
            ui.label(
                egui::RichText::new("egui")
                    .color(ModernTheme::TEXT_PRIMARY)
                    .strong(),
            );
        });
    });

    ui.add_space(Spacing::XL);

    // Features
    modern_card(ui, |ui| {
        section_header(ui, "Features", None::<fn(&mut egui::Ui)>);

        let features = [
            ("search", "DVD detection and scanning"),
            ("menu", "Title and chapter selection"),
            ("save", "Multiple output formats"),
            ("wrench", "HandBrake integration"),
            ("upload", "Server upload support"),
            ("heart", "Simple, clean interface"),
        ];

        for (_icon, text) in &features {
            ui.label(
                egui::RichText::new(*text)
                    .color(ModernTheme::TEXT_PRIMARY),
            );
            ui.add_space(Spacing::SM);
        }
    });

    ui.add_space(Spacing::XL);

    // Support
    modern_card(ui, |ui| {
        section_header(ui, "Support & Links", None::<fn(&mut egui::Ui)>);

        if ui
            .link(
                egui::RichText::new("HandBrake Official Website")
                    .color(ModernTheme::ACCENT_PRIMARY),
            )
            .clicked()
        {
            let _ = open::that("https://handbrake.fr/");
        }
        ui.add_space(Spacing::SM);

        if ui
            .link(
                egui::RichText::new("Rust Programming Language")
                    .color(ModernTheme::ACCENT_PRIMARY),
            )
            .clicked()
        {
            let _ = open::that("https://www.rust-lang.org/");
        }
        ui.add_space(Spacing::SM);

        if ui
            .link(egui::RichText::new("egui GUI Framework").color(ModernTheme::ACCENT_PRIMARY))
            .clicked()
        {
            let _ = open::that("https://github.com/emilk/egui");
        }
    });
}
