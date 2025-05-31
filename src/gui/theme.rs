use egui::{Color32, Rounding, Stroke, Vec2, Shadow};

/// Simple, clean theme for a functional UI similar to TeamSpeak
pub struct BasicTheme;

impl BasicTheme {
    // Background colors
    pub const BACKGROUND: Color32 = Color32::from_rgb(45, 45, 45);
    pub const PANEL: Color32 = Color32::from_rgb(55, 55, 55);
    pub const SURFACE: Color32 = Color32::from_rgb(65, 65, 65);
    
    // Text colors
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(220, 220, 220);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 180);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 140, 140);
    
    // Accent colors
    pub const ACCENT: Color32 = Color32::from_rgb(70, 130, 180);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(90, 150, 200);
    pub const SUCCESS: Color32 = Color32::from_rgb(60, 150, 60);
    pub const WARNING: Color32 = Color32::from_rgb(200, 150, 60);
    pub const ERROR: Color32 = Color32::from_rgb(180, 60, 60);
    
    // Border colors
    pub const BORDER: Color32 = Color32::from_rgb(80, 80, 80);
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(100, 100, 100);
    
    // Interactive states
    pub const BUTTON_ACTIVE: Color32 = Color32::from_rgb(60, 120, 170);
    pub const BUTTON_HOVER: Color32 = Color32::from_rgb(75, 75, 75);
}

pub struct Layout;

impl Layout {
    pub const SPACING: f32 = 8.0;
    pub const SPACING_SMALL: f32 = 4.0;
    pub const SPACING_LARGE: f32 = 16.0;
    pub const BUTTON_HEIGHT: f32 = 28.0;
    pub const INPUT_HEIGHT: f32 = 24.0;
    pub const ROUNDING: f32 = 4.0;
    pub const BORDER_WIDTH: f32 = 1.0;
}

/// Apply the theme to egui context
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Window styling
    style.visuals.window_fill = BasicTheme::BACKGROUND;
    style.visuals.panel_fill = BasicTheme::PANEL;
    style.visuals.window_stroke = Stroke::new(Layout::BORDER_WIDTH, BasicTheme::BORDER);
    style.visuals.window_rounding = Rounding::same(Layout::ROUNDING);
    
    // Text colors
    style.visuals.override_text_color = Some(BasicTheme::TEXT_PRIMARY);
    
    // Button styling
    style.visuals.widgets.noninteractive.bg_fill = BasicTheme::SURFACE;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(Layout::BORDER_WIDTH, BasicTheme::BORDER);
    style.visuals.widgets.noninteractive.rounding = Rounding::same(Layout::ROUNDING);
    
    style.visuals.widgets.inactive.bg_fill = BasicTheme::SURFACE;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(Layout::BORDER_WIDTH, BasicTheme::BORDER);
    style.visuals.widgets.inactive.rounding = Rounding::same(Layout::ROUNDING);
    
    style.visuals.widgets.hovered.bg_fill = BasicTheme::BUTTON_HOVER;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(Layout::BORDER_WIDTH, BasicTheme::BORDER_LIGHT);
    style.visuals.widgets.hovered.rounding = Rounding::same(Layout::ROUNDING);
    
    style.visuals.widgets.active.bg_fill = BasicTheme::BUTTON_ACTIVE;
    style.visuals.widgets.active.bg_stroke = Stroke::new(Layout::BORDER_WIDTH, BasicTheme::ACCENT);
    style.visuals.widgets.active.rounding = Rounding::same(Layout::ROUNDING);
    
    // Selection and hyperlink colors
    style.visuals.selection.bg_fill = BasicTheme::ACCENT;
    style.visuals.hyperlink_color = BasicTheme::ACCENT;
    
    // Remove shadows and effects
    style.visuals.window_shadow = Shadow::NONE;
    style.visuals.popup_shadow = Shadow::NONE;
    
    // Spacing
    style.spacing.button_padding = Vec2::new(Layout::SPACING, Layout::SPACING_SMALL);
    style.spacing.item_spacing = Vec2::new(Layout::SPACING, Layout::SPACING);
    style.spacing.window_margin = egui::Margin::same(Layout::SPACING);
    style.spacing.menu_margin = egui::Margin::same(Layout::SPACING_SMALL);
    
    ctx.set_style(style);
}

/// Create a styled panel
pub fn styled_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(BasicTheme::PANEL)
        .stroke(Stroke::new(Layout::BORDER_WIDTH, BasicTheme::BORDER))
        .rounding(Rounding::same(Layout::ROUNDING))
        .inner_margin(Layout::SPACING)
        .show(ui, add_contents);
}

/// Create a grouped section with heading
pub fn grouped_section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.group(|ui| {
        ui.label(egui::RichText::new(title).strong().color(BasicTheme::TEXT_PRIMARY));
        ui.separator();
        ui.add_space(Layout::SPACING_SMALL);
        add_contents(ui);
    });
}

/// Create a full-width button
pub fn full_width_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add_sized([ui.available_width(), Layout::BUTTON_HEIGHT], egui::Button::new(text))
}

/// Create a status indicator
pub fn status_indicator(ui: &mut egui::Ui, text: &str, status_type: StatusType) {
    let color = match status_type {
        StatusType::Success => BasicTheme::SUCCESS,
        StatusType::Warning => BasicTheme::WARNING,
        StatusType::Error => BasicTheme::ERROR,
        StatusType::Info => BasicTheme::ACCENT,
    };
    
    ui.horizontal(|ui| {
        ui.colored_label(color, "●");
        ui.label(text);
    });
}

pub enum StatusType {
    Success,
    Warning,
    Error,
    Info,
}