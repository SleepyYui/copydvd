use crate::gui::theme::{ModernTheme, StyleConstants};
use egui::{Button, Rounding, Vec2};

/// Create a modern primary button with glassmorphism effects
pub fn primary_button(text: &str) -> Button<'_> {
    Button::new(text)
        .fill(ModernTheme::PRIMARY)
        .stroke(egui::Stroke::new(1.5, ModernTheme::PRIMARY_BRIGHT))
        .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
        .min_size(Vec2::new(0.0, StyleConstants::BUTTON_HEIGHT_MD))
}

/// Create a secondary button with subtle styling
pub fn secondary_button(text: &str) -> Button<'_> {
    Button::new(text)
        .fill(ModernTheme::GLASS_SURFACE)
        .stroke(egui::Stroke::new(1.0, ModernTheme::BORDER_NORMAL))
        .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
        .min_size(Vec2::new(0.0, StyleConstants::BUTTON_HEIGHT_MD))
}

/// Create a success button with green accent
pub fn success_button(text: &str) -> Button<'_> {
    Button::new(text)
        .fill(ModernTheme::SUCCESS)
        .stroke(egui::Stroke::new(1.0, ModernTheme::SUCCESS_GLOW))
        .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
        .min_size(Vec2::new(0.0, StyleConstants::BUTTON_HEIGHT_MD))
}

/// Create a danger button with red accent
pub fn danger_button(text: &str) -> Button<'_> {
    Button::new(text)
        .fill(ModernTheme::ERROR)
        .stroke(egui::Stroke::new(1.0, ModernTheme::ERROR_GLOW))
        .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
        .min_size(Vec2::new(0.0, StyleConstants::BUTTON_HEIGHT_MD))
}

/// Create a small button for secondary actions
pub fn small_button(text: &str) -> Button<'_> {
    Button::new(text)
        .fill(ModernTheme::INTERACTIVE_IDLE)
        .stroke(egui::Stroke::new(1.0, ModernTheme::BORDER_NORMAL))
        .rounding(Rounding::same(StyleConstants::ROUNDING_SM))
        .min_size(Vec2::new(0.0, StyleConstants::BUTTON_HEIGHT_SM))
}

/// Create an icon button with minimal styling
pub fn icon_button(icon: &str) -> Button<'_> {
    Button::new(icon)
        .fill(ModernTheme::INTERACTIVE_IDLE)
        .stroke(egui::Stroke::new(1.0, ModernTheme::BORDER_NORMAL))
        .rounding(Rounding::same(StyleConstants::ROUNDING_FULL))
}

/// Create a browse button for file dialogs
pub fn browse_button() -> Button<'static> {
    Button::new("Browse")
        .fill(ModernTheme::NEON_BLUE)
        .stroke(egui::Stroke::new(1.5, ModernTheme::NEON_BLUE))
        .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
}

/// Create a refresh button
pub fn refresh_button() -> Button<'static> {
    Button::new("🔄")
        .fill(ModernTheme::NEON_CYAN)
        .rounding(Rounding::same(StyleConstants::ROUNDING_FULL))
}

/// Create a clear button
pub fn clear_button() -> Button<'static> {
    Button::new("Clear")
        .fill(ModernTheme::WARNING)
        .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
}

/// Render button with proper spacing
pub fn render_button_with_spacing(ui: &mut egui::Ui, button: Button) -> egui::Response {
    ui.add_space(StyleConstants::SPACING_SM);
    let response = ui.add(button);
    ui.add_space(StyleConstants::SPACING_SM);
    response
}

/// Create a button group container
pub fn button_group<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = StyleConstants::SPACING_SM;
        add_contents(ui)
    }).inner
}