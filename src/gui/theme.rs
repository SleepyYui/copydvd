use egui::{Color32, Margin, Rounding, Stroke, Style, Vec2, Visuals};

/// Modern theme colors inspired by contemporary design systems
pub struct ModernTheme;

#[allow(dead_code, reason = "Future feature - theme system expansion")]
impl ModernTheme {
    // Background colors - rich dark theme with subtle warmth
    pub const BACKGROUND_PRIMARY: Color32 = Color32::from_rgb(16, 20, 24); // Deep blue-gray
    pub const BACKGROUND_SECONDARY: Color32 = Color32::from_rgb(24, 30, 36); // Card background
    pub const BACKGROUND_TERTIARY: Color32 = Color32::from_rgb(32, 40, 48); // Elevated surfaces

    // Accent colors - vibrant purple-blue palette
    pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(138, 92, 246); // Vibrant purple
    #[allow(dead_code, reason = "Future feature - theme system expansion")]
    pub const ACCENT_SECONDARY: Color32 = Color32::from_rgb(124, 78, 230); // Darker purple
    #[allow(dead_code, reason = "Future feature - theme system expansion")]
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(152, 106, 255); // Lighter purple

    // Status colors - vibrant and modern
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94); // Emerald green
    pub const WARNING: Color32 = Color32::from_rgb(251, 146, 60); // Warm orange
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68); // Bright red
    pub const INFO: Color32 = Color32::from_rgb(59, 130, 246); // Sky blue

    // Text colors - high contrast for accessibility
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255); // Pure white
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(174, 174, 178); // Light gray
    pub const TEXT_TERTIARY: Color32 = Color32::from_rgb(142, 142, 147); // Medium gray
    #[allow(dead_code, reason = "Future feature - theme system expansion")]
    pub const TEXT_DISABLED: Color32 = Color32::from_rgb(99, 99, 102); // Dark gray

    // Interactive element colors
    #[allow(dead_code, reason = "Future feature - theme system expansion")]
    pub const BUTTON_PRIMARY: Color32 = Color32::from_rgb(138, 92, 246); // Vibrant purple
    pub const BUTTON_SECONDARY: Color32 = Color32::from_rgb(40, 48, 56); // Warm dark gray
    pub const BUTTON_HOVER: Color32 = Color32::from_rgb(152, 106, 255); // Lighter purple
    pub const BUTTON_ACTIVE: Color32 = Color32::from_rgb(124, 78, 230); // Darker purple

    // Border and stroke colors
    pub const BORDER_PRIMARY: Color32 = Color32::from_rgb(64, 72, 80); // Subtle warm borders
    #[allow(dead_code, reason = "Future feature - theme system expansion")]
    pub const BORDER_SECONDARY: Color32 = Color32::from_rgb(48, 56, 64); // Lighter borders
    #[allow(dead_code, reason = "Future feature - theme system expansion")]
    pub const BORDER_ACCENT: Color32 = Color32::from_rgb(138, 92, 246); // Purple accent borders
}

/// Modern spacing system based on 8px grid
pub struct Spacing;

#[allow(dead_code, reason = "Future feature - spacing system expansion")]
impl Spacing {
    #[allow(dead_code, reason = "Future feature - spacing system expansion")]
    pub const NONE: f32 = 0.0;
    pub const XS: f32 = 4.0; // 0.5 units
    pub const SM: f32 = 8.0; // 1 unit
    pub const MD: f32 = 16.0; // 2 units
    pub const LG: f32 = 24.0; // 3 units
    pub const XL: f32 = 32.0; // 4 units
    #[allow(dead_code, reason = "Future feature - spacing system expansion")]
    pub const XXL: f32 = 48.0; // 6 units
    #[allow(dead_code, reason = "Future feature - spacing system expansion")]
    pub const XXXL: f32 = 64.0; // 8 units
}

/// Modern border radius system
pub struct BorderRadius;

#[allow(dead_code, reason = "Future feature - border radius system expansion")]
impl BorderRadius {
    #[allow(dead_code, reason = "Future feature - border radius system expansion")]
    pub const NONE: f32 = 0.0;
    pub const SM: f32 = 6.0; // Small radius
    pub const MD: f32 = 12.0; // Medium radius
    pub const LG: f32 = 16.0; // Large radius
    #[allow(dead_code, reason = "Future feature - border radius system expansion")]
    pub const XL: f32 = 24.0; // Extra large radius
    #[allow(dead_code, reason = "Future feature - border radius system expansion")]
    pub const FULL: f32 = 999.0; // Fully rounded
}

/// Helper function placeholder (keeping for compatibility)
#[allow(dead_code)]
pub fn svg_icon_fn(_name: &'static str, _size: f32, _color: Color32) -> impl Fn(&mut egui::Ui) {
    move |_ui: &mut egui::Ui| {
        // Icons removed from UI
    }
}

/// Apply modern theme to egui context
pub fn apply_modern_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::dark();

    // Background colors
    visuals.panel_fill = ModernTheme::BACKGROUND_PRIMARY;
    visuals.window_fill = ModernTheme::BACKGROUND_SECONDARY;
    visuals.extreme_bg_color = ModernTheme::BACKGROUND_TERTIARY;

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = ModernTheme::BACKGROUND_SECONDARY;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, ModernTheme::TEXT_SECONDARY);

    visuals.widgets.inactive.bg_fill = ModernTheme::BUTTON_SECONDARY;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, ModernTheme::TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::same(BorderRadius::MD);

    visuals.widgets.hovered.bg_fill = ModernTheme::BUTTON_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, ModernTheme::TEXT_PRIMARY);
    visuals.widgets.hovered.rounding = Rounding::same(BorderRadius::MD);

    visuals.widgets.active.bg_fill = ModernTheme::BUTTON_ACTIVE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, ModernTheme::TEXT_PRIMARY);
    visuals.widgets.active.rounding = Rounding::same(BorderRadius::MD);

    visuals.widgets.open.bg_fill = ModernTheme::ACCENT_PRIMARY;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, ModernTheme::TEXT_PRIMARY);
    visuals.widgets.open.rounding = Rounding::same(BorderRadius::MD);

    // Selection colors
    visuals.selection.bg_fill = ModernTheme::ACCENT_PRIMARY;
    visuals.selection.stroke = Stroke::new(1.0, ModernTheme::ACCENT_PRIMARY);

    // Text colors
    visuals.widgets.noninteractive.fg_stroke.color = ModernTheme::TEXT_PRIMARY;
    visuals.widgets.inactive.fg_stroke.color = ModernTheme::TEXT_PRIMARY;
    visuals.widgets.hovered.fg_stroke.color = ModernTheme::TEXT_PRIMARY;
    visuals.widgets.active.fg_stroke.color = ModernTheme::TEXT_PRIMARY;
    visuals.widgets.open.fg_stroke.color = ModernTheme::TEXT_PRIMARY;
    visuals.hyperlink_color = ModernTheme::ACCENT_PRIMARY;
    visuals.warn_fg_color = ModernTheme::WARNING;
    visuals.error_fg_color = ModernTheme::ERROR;

    // Spacing
    style.spacing.button_padding = Vec2::new(Spacing::MD, Spacing::SM);
    style.spacing.item_spacing = Vec2::new(Spacing::SM, Spacing::SM);
    style.spacing.window_margin = Margin::same(Spacing::LG);
    style.spacing.menu_margin = Margin::same(Spacing::SM);

    // Apply the style
    style.visuals = visuals;
    ctx.set_style(style);
}

/// Modern card component with subtle shadow effect
pub fn modern_card<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let frame = egui::Frame {
        fill: ModernTheme::BACKGROUND_SECONDARY,
        stroke: Stroke::new(1.0, ModernTheme::BORDER_PRIMARY),
        rounding: Rounding::same(BorderRadius::LG),
        inner_margin: Margin::same(Spacing::LG),
        outer_margin: Margin::same(Spacing::SM),
        shadow: egui::epaint::Shadow {
            color: Color32::from_black_alpha(60),
            offset: Vec2::new(0.0, 4.0),
            blur: 12.0,
            spread: 0.0,
        },
    };

    frame.show(ui, add_contents)
}

/// Modern button with improved styling
pub fn modern_button(ui: &mut egui::Ui, text: &str, variant: ButtonVariant) -> egui::Response {
    let (bg_color, text_color, border_color) = match variant {
        ButtonVariant::Primary => (
            ModernTheme::ACCENT_PRIMARY,
            ModernTheme::TEXT_PRIMARY,
            ModernTheme::ACCENT_PRIMARY,
        ),
        ButtonVariant::Secondary => (
            ModernTheme::BUTTON_SECONDARY,
            ModernTheme::TEXT_PRIMARY,
            ModernTheme::BORDER_PRIMARY,
        ),
        ButtonVariant::Success => (
            ModernTheme::SUCCESS,
            ModernTheme::TEXT_PRIMARY,
            ModernTheme::SUCCESS,
        ),
        ButtonVariant::Warning => (
            ModernTheme::WARNING,
            ModernTheme::BACKGROUND_PRIMARY,
            ModernTheme::WARNING,
        ),
        ButtonVariant::Error => (
            ModernTheme::ERROR,
            ModernTheme::TEXT_PRIMARY,
            ModernTheme::ERROR,
        ),
    };

    let button = egui::Button::new(egui::RichText::new(text).color(text_color))
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .rounding(Rounding::same(BorderRadius::MD));

    ui.add_sized([ui.available_width(), 32.0], button)
}

/// Button style variants
#[derive(Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    #[allow(dead_code, reason = "Future feature - button variant expansion")]
    Error,
}

/// Modern section header with icon support
pub fn section_header<F>(ui: &mut egui::Ui, title: &str, icon_fn: Option<F>)
where
    F: Fn(&mut egui::Ui),
{
    ui.horizontal(|ui| {
        if let Some(icon_func) = icon_fn {
            icon_func(ui);
            ui.add_space(Spacing::SM);
        }

        ui.label(
            egui::RichText::new(title)
                .size(18.0)
                .color(ModernTheme::TEXT_PRIMARY)
                .strong(),
        );
    });

    ui.add_space(Spacing::SM);

    // Add subtle divider
    let rect = ui.allocate_space(Vec2::new(ui.available_width(), 1.0)).1;
    ui.painter()
        .rect_filled(rect, Rounding::ZERO, ModernTheme::BORDER_PRIMARY);

    ui.add_space(Spacing::MD);
}

/// Modern content wrapper for main content areas
#[allow(dead_code, reason = "Future feature for content area styling")]
pub fn content_wrapper<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let frame = egui::Frame {
        fill: Color32::TRANSPARENT,
        stroke: egui::Stroke::NONE,
        rounding: egui::Rounding::ZERO,
        inner_margin: egui::Margin::symmetric(Spacing::LG, 0.0),
        outer_margin: egui::Margin::ZERO,
        shadow: egui::epaint::Shadow::NONE,
    };

    frame.show(ui, add_contents)
}

/// Modern input field with improved styling
pub fn modern_input(ui: &mut egui::Ui, text: &mut String, placeholder: &str) -> egui::Response {
    let text_edit = egui::TextEdit::singleline(text)
        .hint_text(egui::RichText::new(placeholder).color(ModernTheme::TEXT_TERTIARY))
        .desired_width(ui.available_width())
        .margin(Vec2::new(Spacing::MD, Spacing::SM));

    ui.add(text_edit)
}

/// Icon with text component for better integration
#[allow(dead_code, reason = "Icons removed from UI")]
pub fn icon_with_text(
    ui: &mut egui::Ui,
    icon_fn: impl Fn(&mut egui::Ui),
    text: &str,
    color: Color32,
) {
    ui.horizontal(|ui| {
        icon_fn(ui);
        ui.add_space(Spacing::SM);
        ui.label(egui::RichText::new(text).color(color));
    });
}

/// Modern navigation item for sidebar
pub fn nav_item(ui: &mut egui::Ui, text: &str, is_active: bool) -> egui::Response {
    let (bg_color, text_color, _border_color) = if is_active {
        (
            Color32::from_rgba_unmultiplied(138, 92, 246, 25), // Subtle accent background
            ModernTheme::ACCENT_PRIMARY,
            ModernTheme::ACCENT_PRIMARY,
        )
    } else {
        (
            Color32::TRANSPARENT,
            ModernTheme::TEXT_SECONDARY,
            Color32::TRANSPARENT,
        )
    };

    let desired_size = egui::Vec2::new(ui.available_width(), 48.0);
    let response = ui.allocate_response(desired_size, egui::Sense::click());

    // Hover effect
    let final_bg_color = if response.hovered() && !is_active {
        Color32::from_rgba_unmultiplied(255, 255, 255, 8)
    } else {
        bg_color
    };

    if ui.is_rect_visible(response.rect) {
        // Background
        ui.painter().rect_filled(
            response.rect,
            egui::Rounding::same(BorderRadius::LG),
            final_bg_color,
        );

        // Left border for active state
        if is_active {
            let border_rect = egui::Rect::from_min_size(
                response.rect.min,
                egui::Vec2::new(3.0, response.rect.height()),
            );
            ui.painter().rect_filled(
                border_rect,
                egui::Rounding::same(BorderRadius::SM),
                ModernTheme::ACCENT_PRIMARY,
            );
        }

        // Content
        let content_rect = response.rect.shrink2(egui::Vec2::new(Spacing::MD, 0.0));
        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(Spacing::LG);
                ui.label(
                    egui::RichText::new(text)
                        .color(text_color)
                        .size(14.0)
                        .strong(),
                );
            });
        });
    }

    response
}

/// Legacy tab button - keeping for compatibility
#[allow(dead_code, reason = "Legacy compatibility")]
pub fn tab_button(
    ui: &mut egui::Ui,
    text: &str,
    _icon_fn: impl Fn(&mut egui::Ui),
    is_active: bool,
) -> egui::Response {
    nav_item(ui, text, is_active)
}

/// Status indicator with modern styling
pub fn status_indicator(ui: &mut egui::Ui, message: &str, status_type: StatusType) {
    let (color, _icon_name) = match status_type {
        StatusType::Success => (ModernTheme::SUCCESS, "check"),
        StatusType::Warning => (ModernTheme::WARNING, "warning"),
        StatusType::Error => (ModernTheme::ERROR, "cross"),
        StatusType::Info => (ModernTheme::INFO, "info"),
    };

    let frame = egui::Frame {
        fill: Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 20),
        stroke: Stroke::new(1.0, color),
        rounding: Rounding::same(BorderRadius::SM),
        inner_margin: Margin::same(Spacing::SM),
        outer_margin: Margin::same(Spacing::XS),
        ..Default::default()
    };

    frame.show(ui, |ui| {
        ui.label(egui::RichText::new(message).color(color));
    });
}

#[derive(Clone, Copy)]
pub enum StatusType {
    Success,
    Warning,
    Error,
    Info,
}

/// Modern progress bar
#[allow(dead_code, reason = "Future feature - progress indicator system")]
pub fn progress_bar(ui: &mut egui::Ui, progress: f32, height: f32) {
    let desired_size = Vec2::new(ui.available_width(), height);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    // Background
    ui.painter().rect_filled(
        rect,
        Rounding::same(BorderRadius::FULL),
        ModernTheme::BACKGROUND_TERTIARY,
    );

    // Progress fill
    let fill_width = rect.width() * progress.clamp(0.0, 1.0);
    let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
    ui.painter().rect_filled(
        fill_rect,
        Rounding::same(BorderRadius::FULL),
        ModernTheme::ACCENT_PRIMARY,
    );
}

/// Legacy compatibility functions
pub fn styled_panel<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    modern_card(ui, add_contents)
}

pub fn grouped_section<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    section_header(ui, title, None::<fn(&mut egui::Ui)>);
    add_contents(ui)
}

pub fn full_width_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    modern_button(ui, text, ButtonVariant::Primary)
}

// Legacy theme compatibility
#[allow(dead_code, reason = "Legacy compatibility for future migration")]
pub struct BasicTheme;
#[allow(dead_code, reason = "Legacy compatibility for future migration")]
impl BasicTheme {
    #[allow(dead_code, reason = "Legacy compatibility for future migration")]
    pub const SUCCESS: Color32 = ModernTheme::SUCCESS;
    #[allow(dead_code, reason = "Legacy compatibility for future migration")]
    pub const WARNING: Color32 = ModernTheme::WARNING;
    #[allow(dead_code, reason = "Legacy compatibility for future migration")]
    pub const ERROR: Color32 = ModernTheme::ERROR;
    pub const ACCENT: Color32 = ModernTheme::ACCENT_PRIMARY;
    pub const SURFACE: Color32 = ModernTheme::BACKGROUND_SECONDARY;
}

pub struct Layout;
impl Layout {
    pub const SPACING: f32 = Spacing::MD;
    pub const SPACING_SMALL: f32 = Spacing::SM;
    pub const SPACING_LARGE: f32 = Spacing::LG;
    pub const ROUNDING: f32 = BorderRadius::MD;
}

// Deprecated - keeping for compatibility but redirecting to modern theme
#[allow(dead_code, reason = "Legacy compatibility for future migration")]
pub fn apply_theme(ctx: &egui::Context) {
    apply_modern_theme(ctx);
}
