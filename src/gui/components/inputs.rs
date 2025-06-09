use crate::gui::theme::{ModernTheme, StyleConstants};
use egui::{Color32, Rounding, Stroke, TextEdit, Vec2};

/// Create a styled text input field
pub fn text_input<'a>(text: &'a mut String) -> TextEdit<'a> {
    TextEdit::singleline(text)
        .desired_width(f32::INFINITY)
        .margin(Vec2::new(
            StyleConstants::SPACING_SM,
            StyleConstants::SPACING_SM,
        ))
}

/// Create a password input field
pub fn password_input<'a>(text: &'a mut String) -> TextEdit<'a> {
    TextEdit::singleline(text)
        .password(true)
        .desired_width(f32::INFINITY)
        .margin(Vec2::new(
            StyleConstants::SPACING_SM,
            StyleConstants::SPACING_SM,
        ))
}

/// Create a multiline text input
pub fn multiline_input<'a>(text: &'a mut String, rows: usize) -> TextEdit<'a> {
    TextEdit::multiline(text)
        .desired_rows(rows)
        .desired_width(f32::INFINITY)
        .margin(Vec2::new(
            StyleConstants::SPACING_SM,
            StyleConstants::SPACING_SM,
        ))
}

/// Create a number input field
pub fn number_input(ui: &mut egui::Ui, value: &mut String, label: &str) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(text_input(value).hint_text("Enter number"))
    })
    .inner
}

/// Create a path input with browse button
pub fn path_input(
    ui: &mut egui::Ui,
    path: &mut String,
    label: &str,
    browse_action: impl FnOnce(),
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(label);
        let text_response = ui.add(text_input(path).hint_text("Enter path"));
        if ui
            .add(crate::gui::components::buttons::browse_button())
            .clicked()
        {
            browse_action();
        }
        text_response
    })
    .inner
}

/// Create a labeled input field
pub fn labeled_input(
    ui: &mut egui::Ui,
    label: &str,
    text: &mut String,
    hint: Option<&str>,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut input = text_input(text);
        if let Some(hint) = hint {
            input = input.hint_text(hint);
        }
        ui.add(input)
    })
    .inner
}

/// Create a combo box with modern styling
pub fn styled_combo_box<'a>(
    ui: &mut egui::Ui,
    id: &str,
    selected: &'a mut String,
    options: &[&str],
) -> egui::Response {
    egui::ComboBox::from_id_source(id)
        .selected_text(egui::RichText::new(selected.as_str()).color(crate::gui::theme::ModernTheme::TEXT_PRIMARY))
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(selected, option.to_string(), *option);
            }
        })
        .response
}

/// Create a slider with modern styling
pub fn styled_slider<Num: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    value: &mut Num,
    range: std::ops::RangeInclusive<Num>,
    text: &str,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(text);
        ui.add(egui::Slider::new(value, range))
    })
    .inner
}

/// Create an input group with consistent styling
pub fn input_group<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_inputs: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());

        // Group title
        ui.heading(title);
        ui.add_space(StyleConstants::SPACING_SM);

        let response = add_inputs(ui);

        ui.add_space(StyleConstants::SPACING_SM);
        response
    })
    .inner
}

/// Create a form field with validation styling
pub fn validated_input(
    ui: &mut egui::Ui,
    label: &str,
    text: &mut String,
    is_valid: bool,
    error_msg: Option<&str>,
) -> egui::Response {
    ui.vertical(|ui| {
        let response = labeled_input(ui, label, text, None);

        if !is_valid {
            if let Some(error) = error_msg {
                ui.colored_label(ModernTheme::ERROR, error);
            }
        }

        response
    })
    .inner
}

/// Create a toggle switch with modern styling
pub fn toggle_switch(ui: &mut egui::Ui, value: &mut bool, text: &str) -> egui::Response {
    ui.horizontal(|ui| {
        let response = ui.checkbox(value, "");
        ui.label(text);
        response
    })
    .inner
}

/// Create a search input field
pub fn search_input<'a>(text: &'a mut String) -> TextEdit<'a> {
    text_input(text).hint_text("Search...")
}
