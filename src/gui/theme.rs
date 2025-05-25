use egui::{Color32, Rounding, Shadow, Stroke, Style, Visuals, FontId, TextStyle};
use std::collections::BTreeMap;

/// Ultra-modern high-tech theme with advanced visual effects
pub struct ModernTheme;

impl ModernTheme {
    // Advanced gradient backgrounds
    pub const BACKGROUND_DARK: Color32 = Color32::from_rgb(8, 10, 16);
    pub const BACKGROUND_MAIN: Color32 = Color32::from_rgb(12, 15, 23);
    pub const BACKGROUND_LIGHT: Color32 = Color32::from_rgb(16, 20, 30);
    
    // Glassmorphism surfaces
    pub const GLASS_SURFACE: Color32 = Color32::from_rgba_premultiplied(30, 35, 45, 180);
    pub const GLASS_ELEVATED: Color32 = Color32::from_rgba_premultiplied(40, 45, 60, 200);
    pub const GLASS_OVERLAY: Color32 = Color32::from_rgba_premultiplied(50, 55, 70, 220);
    
    // Neon accent colors with glow effects
    pub const NEON_BLUE: Color32 = Color32::from_rgb(0, 150, 255);
    pub const NEON_CYAN: Color32 = Color32::from_rgb(0, 255, 255);
    pub const NEON_PURPLE: Color32 = Color32::from_rgb(138, 43, 226);
    pub const NEON_PINK: Color32 = Color32::from_rgb(255, 20, 147);
    pub const NEON_GREEN: Color32 = Color32::from_rgb(57, 255, 20);
    
    // Primary brand colors with gradients
    pub const PRIMARY: Color32 = Color32::from_rgb(0, 150, 255);
    pub const PRIMARY_BRIGHT: Color32 = Color32::from_rgb(30, 170, 255);
    pub const PRIMARY_GLOW: Color32 = Color32::from_rgba_premultiplied(0, 150, 255, 100);
    pub const PRIMARY_DARK: Color32 = Color32::from_rgb(0, 100, 180);
    
    // Status colors with enhanced vibrancy
    pub const SUCCESS: Color32 = Color32::from_rgb(0, 255, 100);
    pub const SUCCESS_GLOW: Color32 = Color32::from_rgba_premultiplied(0, 255, 100, 80);
    pub const WARNING: Color32 = Color32::from_rgb(255, 170, 0);
    pub const WARNING_GLOW: Color32 = Color32::from_rgba_premultiplied(255, 170, 0, 80);
    pub const ERROR: Color32 = Color32::from_rgb(255, 50, 80);
    pub const ERROR_GLOW: Color32 = Color32::from_rgba_premultiplied(255, 50, 80, 80);
    pub const INFO: Color32 = Color32::from_rgb(100, 200, 255);
    pub const INFO_GLOW: Color32 = Color32::from_rgba_premultiplied(100, 200, 255, 80);
    
    // Advanced text hierarchy
    pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(255, 255, 255);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 245, 250);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 190, 200);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(130, 140, 150);
    pub const TEXT_SUBTLE: Color32 = Color32::from_rgb(80, 90, 100);
    
    // Sophisticated borders and lines
    pub const BORDER_BRIGHT: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 60);
    pub const BORDER_NORMAL: Color32 = Color32::from_rgba_premultiplied(200, 210, 220, 40);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(150, 160, 170, 20);
    pub const BORDER_GLOW: Color32 = Color32::from_rgba_premultiplied(0, 150, 255, 120);
    
    // Interactive states with advanced effects
    pub const INTERACTIVE_IDLE: Color32 = Color32::from_rgba_premultiplied(60, 70, 85, 150);
    pub const INTERACTIVE_HOVER: Color32 = Color32::from_rgba_premultiplied(80, 90, 110, 180);
    pub const INTERACTIVE_ACTIVE: Color32 = Color32::from_rgba_premultiplied(100, 110, 130, 200);
    pub const INTERACTIVE_GLOW: Color32 = Color32::from_rgba_premultiplied(0, 150, 255, 60);
    
    // Advanced card styles
    pub const CARD_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(25, 30, 40, 200);
    pub const CARD_ELEVATED: Color32 = Color32::from_rgba_premultiplied(35, 40, 55, 220);
    pub const CARD_GLOW: Color32 = Color32::from_rgba_premultiplied(0, 150, 255, 30);
}

/// Advanced styling constants for modern design
pub struct StyleConstants;

impl StyleConstants {
    // Enhanced spacing system
    pub const SPACING_XS: f32 = 2.0;
    pub const SPACING_SM: f32 = 6.0;
    pub const SPACING_MD: f32 = 12.0;
    pub const SPACING_LG: f32 = 20.0;
    pub const SPACING_XL: f32 = 32.0;
    pub const SPACING_XXL: f32 = 48.0;
    pub const SPACING_XXXL: f32 = 64.0;
    
    // Advanced component sizing
    pub const BUTTON_HEIGHT_SM: f32 = 32.0;
    pub const BUTTON_HEIGHT_MD: f32 = 40.0;
    pub const BUTTON_HEIGHT_LG: f32 = 48.0;
    pub const BUTTON_HEIGHT_XL: f32 = 56.0;
    
    pub const INPUT_HEIGHT: f32 = 40.0;
    pub const TAB_HEIGHT: f32 = 48.0;
    pub const HEADER_HEIGHT: f32 = 64.0;
    
    // Sophisticated border radius
    pub const ROUNDING_XS: f32 = 2.0;
    pub const ROUNDING_SM: f32 = 6.0;
    pub const ROUNDING_MD: f32 = 12.0;
    pub const ROUNDING_LG: f32 = 18.0;
    pub const ROUNDING_XL: f32 = 24.0;
    pub const ROUNDING_FULL: f32 = 1000.0;
    
    // Advanced shadow system
    pub const SHADOW_SUBTLE: Shadow = Shadow {
        offset: egui::Vec2::new(0.0, 1.0),
        blur: 3.0,
        spread: 0.0,
        color: Color32::from_black_alpha(20),
    };
    
    pub const SHADOW_SOFT: Shadow = Shadow {
        offset: egui::Vec2::new(0.0, 2.0),
        blur: 8.0,
        spread: 0.0,
        color: Color32::from_black_alpha(40),
    };
    
    pub const SHADOW_MEDIUM: Shadow = Shadow {
        offset: egui::Vec2::new(0.0, 4.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_black_alpha(60),
    };
    
    pub const SHADOW_LARGE: Shadow = Shadow {
        offset: egui::Vec2::new(0.0, 8.0),
        blur: 24.0,
        spread: 0.0,
        color: Color32::from_black_alpha(80),
    };
    
    pub const SHADOW_GLOW: Shadow = Shadow {
        offset: egui::Vec2::new(0.0, 0.0),
        blur: 20.0,
        spread: 2.0,
        color: Color32::from_rgba_premultiplied(0, 150, 255, 60),
    };
    
    // Animation timing
    pub const ANIMATION_FAST: f32 = 0.15;
    pub const ANIMATION_NORMAL: f32 = 0.25;
    pub const ANIMATION_SLOW: f32 = 0.4;
}

/// Apply stunning modern theme with advanced visual effects
pub fn apply_modern_theme(ctx: &egui::Context) {
    // Setup custom fonts for better typography
    setup_custom_fonts(ctx);
    
    let mut style = Style::default();
    
    // Advanced dark visuals with glassmorphism
    style.visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(ModernTheme::TEXT_PRIMARY),
        
        // Sophisticated widget styling
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: ModernTheme::GLASS_SURFACE,
                weak_bg_fill: ModernTheme::BACKGROUND_MAIN,
                bg_stroke: Stroke::new(1.0, ModernTheme::BORDER_SUBTLE),
                rounding: Rounding::same(StyleConstants::ROUNDING_MD),
                fg_stroke: Stroke::new(1.0, ModernTheme::TEXT_PRIMARY),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: ModernTheme::INTERACTIVE_IDLE,
                weak_bg_fill: ModernTheme::GLASS_SURFACE,
                bg_stroke: Stroke::new(1.0, ModernTheme::BORDER_NORMAL),
                rounding: Rounding::same(StyleConstants::ROUNDING_MD),
                fg_stroke: Stroke::new(1.0, ModernTheme::TEXT_PRIMARY),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: ModernTheme::INTERACTIVE_HOVER,
                weak_bg_fill: ModernTheme::GLASS_ELEVATED,
                bg_stroke: Stroke::new(1.5, ModernTheme::BORDER_BRIGHT),
                rounding: Rounding::same(StyleConstants::ROUNDING_MD),
                fg_stroke: Stroke::new(1.0, ModernTheme::TEXT_BRIGHT),
                expansion: 2.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: ModernTheme::INTERACTIVE_ACTIVE,
                weak_bg_fill: ModernTheme::GLASS_OVERLAY,
                bg_stroke: Stroke::new(2.0, ModernTheme::PRIMARY_BRIGHT),
                rounding: Rounding::same(StyleConstants::ROUNDING_MD),
                fg_stroke: Stroke::new(1.0, ModernTheme::TEXT_BRIGHT),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: ModernTheme::GLASS_OVERLAY,
                weak_bg_fill: ModernTheme::GLASS_SURFACE,
                bg_stroke: Stroke::new(2.0, ModernTheme::PRIMARY),
                rounding: Rounding::same(StyleConstants::ROUNDING_MD),
                fg_stroke: Stroke::new(1.0, ModernTheme::TEXT_BRIGHT),
                expansion: 0.0,
            },
        },
        
        // Enhanced selection styling
        selection: egui::style::Selection {
            bg_fill: ModernTheme::PRIMARY_GLOW,
            stroke: Stroke::new(2.0, ModernTheme::PRIMARY_BRIGHT),
        },
        
        // Modern color scheme
        hyperlink_color: ModernTheme::NEON_CYAN,
        faint_bg_color: ModernTheme::GLASS_SURFACE,
        extreme_bg_color: ModernTheme::BACKGROUND_DARK,
        code_bg_color: ModernTheme::GLASS_ELEVATED,
        warn_fg_color: ModernTheme::WARNING,
        error_fg_color: ModernTheme::ERROR,
        
        // Advanced window styling
        window_rounding: Rounding::same(StyleConstants::ROUNDING_LG),
        window_shadow: StyleConstants::SHADOW_LARGE,
        window_fill: ModernTheme::BACKGROUND_MAIN,
        window_stroke: Stroke::new(1.0, ModernTheme::BORDER_BRIGHT),
        
        // Sophisticated UI elements
        menu_rounding: Rounding::same(StyleConstants::ROUNDING_MD),
        panel_fill: ModernTheme::GLASS_SURFACE,
        popup_shadow: StyleConstants::SHADOW_MEDIUM,
        
        resize_corner_size: 16.0,
        clip_rect_margin: 4.0,
        button_frame: true,
        collapsing_header_frame: true,
        
        ..Default::default()
    };
    
    // Enhanced spacing configuration
    style.spacing = egui::style::Spacing {
        item_spacing: egui::vec2(StyleConstants::SPACING_MD, StyleConstants::SPACING_SM),
        window_margin: egui::Margin::same(StyleConstants::SPACING_LG),
        button_padding: egui::vec2(StyleConstants::SPACING_LG, StyleConstants::SPACING_MD),
        menu_margin: egui::Margin::same(StyleConstants::SPACING_MD),
        indent: StyleConstants::SPACING_XL,
        icon_width: 16.0,
        icon_spacing: StyleConstants::SPACING_MD,
        tooltip_width: 800.0,
        combo_height: StyleConstants::INPUT_HEIGHT,
        
        ..Default::default()
    };
    
    ctx.set_style(style);
}

/// Setup custom fonts for enhanced typography
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Define modern text styles
    let mut text_styles = BTreeMap::new();
    text_styles.insert(
        TextStyle::Heading,
        FontId::new(28.0, egui::FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Name("subtitle".into()),
        FontId::new(20.0, egui::FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Body,
        FontId::new(14.0, egui::FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.0, egui::FontFamily::Monospace),
    );
    text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, egui::FontFamily::Proportional),
    );
    text_styles.insert(
        TextStyle::Small,
        FontId::new(11.0, egui::FontFamily::Proportional),
    );
    
    let mut style = egui::Style::default();
    style.text_styles = text_styles;
    ctx.set_style(style);
    
}

/// Create stunning glassmorphism card container
pub fn glass_card<R>(
    ui: &mut egui::Ui,
    glow: bool,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let fill = if glow {
        ModernTheme::CARD_ELEVATED
    } else {
        ModernTheme::CARD_BACKGROUND
    };
    
    let stroke = if glow {
        Stroke::new(1.5, ModernTheme::BORDER_BRIGHT)
    } else {
        Stroke::new(1.0, ModernTheme::BORDER_NORMAL)
    };
    
    let shadow = if glow {
        StyleConstants::SHADOW_GLOW
    } else {
        StyleConstants::SHADOW_MEDIUM
    };
    
    egui::Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(Rounding::same(StyleConstants::ROUNDING_LG))
        .shadow(shadow)
        .inner_margin(egui::Margin::same(StyleConstants::SPACING_LG))
        .show(ui, add_contents)
}

/// Create high-tech section with advanced styling
pub fn tech_section<R>(
    ui: &mut egui::Ui,
    title: &str,
    accent_color: Option<Color32>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let color = accent_color.unwrap_or(ModernTheme::PRIMARY);
    
    // Section header with glow effect
    ui.horizontal(|ui| {
        // Accent line
        let rect = ui.allocate_space(egui::vec2(4.0, 24.0)).1;
        ui.painter().rect_filled(
            rect,
            Rounding::same(2.0),
            color,
        );
        
        ui.add_space(StyleConstants::SPACING_MD);
        
        // Title with enhanced typography
        ui.colored_label(ModernTheme::TEXT_BRIGHT, 
            egui::RichText::new(title)
                .size(18.0)
                .strong()
        );
    });
    
    ui.add_space(StyleConstants::SPACING_MD);
    
    let response = add_contents(ui);
    ui.add_space(StyleConstants::SPACING_LG);
    
    response
}

/// Create advanced progress bar with glow effects
pub fn neon_progress_bar(
    ui: &mut egui::Ui, 
    progress: f32, 
    color: Color32,
    height: f32,
    text: Option<&str>
) {
    let available_width = ui.available_width();
    let rect = ui.allocate_space(egui::vec2(available_width, height)).1;
    
    // Background
    ui.painter().rect_filled(
        rect,
        Rounding::same(height / 2.0),
        ModernTheme::GLASS_SURFACE,
    );
    
    // Progress fill with glow
    if progress > 0.0 {
        let fill_width = rect.width() * progress.clamp(0.0, 1.0);
        let fill_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(fill_width, rect.height()),
        );
        
        // Glow effect
        ui.painter().rect_filled(
            fill_rect.expand(2.0),
            Rounding::same(height / 2.0 + 2.0),
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 60),
        );
        
        // Main fill
        ui.painter().rect_filled(
            fill_rect,
            Rounding::same(height / 2.0),
            color,
        );
    }
    
    // Progress text
    if let Some(text) = text {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            FontId::proportional(12.0),
            ModernTheme::TEXT_BRIGHT,
        );
    }
}

/// Create stunning primary button with glow effects
pub fn neon_button(text: &str, color: Color32) -> egui::Button<'_> {
    egui::Button::new(
        egui::RichText::new(text)
            .size(14.0)
            .strong()
            .color(ModernTheme::TEXT_BRIGHT)
    )
    .fill(color)
    .stroke(Stroke::new(1.5, color.gamma_multiply(1.2)))
    .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
}

/// Create modern status indicator with glow
pub fn status_indicator(ui: &mut egui::Ui, status: &str, color: Color32, glow: bool) {
    ui.horizontal(|ui| {
        // Status dot with glow
        let dot_size = 8.0;
        let center = ui.next_widget_position() + egui::vec2(dot_size, dot_size);
        
        if glow {
            // Glow effect
            ui.painter().circle_filled(
                center,
                dot_size + 2.0,
                Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 80),
            );
        }
        
        ui.painter().circle_filled(center, dot_size / 2.0, color);
        ui.allocate_space(egui::vec2(dot_size * 2.0, dot_size * 2.0));
        
        ui.add_space(StyleConstants::SPACING_SM);
        ui.colored_label(ModernTheme::TEXT_PRIMARY, status);
    });
}

/// Create step indicator with advanced styling
pub fn step_indicator(
    ui: &mut egui::Ui,
    step: u8,
    completed: bool,
    active: bool,
    size: f32,
) {
    let center = ui.next_widget_position() + egui::vec2(size / 2.0, size / 2.0);
    
    let (bg_color, border_color, text_color) = if completed {
        (ModernTheme::SUCCESS, ModernTheme::SUCCESS, ModernTheme::BACKGROUND_DARK)
    } else if active {
        (ModernTheme::PRIMARY_GLOW, ModernTheme::PRIMARY_BRIGHT, ModernTheme::TEXT_BRIGHT)
    } else {
        (ModernTheme::INTERACTIVE_IDLE, ModernTheme::BORDER_NORMAL, ModernTheme::TEXT_MUTED)
    };
    
    // Glow effect for active/completed steps
    if active || completed {
        ui.painter().circle_filled(
            center,
            size / 2.0 + 4.0,
            Color32::from_rgba_premultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 40),
        );
    }
    
    // Main circle
    ui.painter().circle_filled(center, size / 2.0, bg_color);
    ui.painter().circle_stroke(center, size / 2.0, Stroke::new(2.0, border_color));
    
    if completed {
        // Checkmark
        let checkmark_points = [
            center + egui::vec2(-size * 0.25, 0.0),
            center + egui::vec2(-size * 0.1, size * 0.15),
            center + egui::vec2(size * 0.25, -size * 0.2),
        ];
        ui.painter().add(egui::Shape::line(
            checkmark_points[0..2].to_vec(),
            Stroke::new(2.5, text_color)
        ));
        ui.painter().add(egui::Shape::line(
            checkmark_points[1..3].to_vec(),
            Stroke::new(2.5, text_color)
        ));
    } else {
        // Step number
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            step.to_string(),
            FontId::proportional(size * 0.5),
            text_color,
        );
    }
    
    ui.allocate_rect(
        egui::Rect::from_center_size(center, egui::vec2(size, size)),
        egui::Sense::hover()
    );
}