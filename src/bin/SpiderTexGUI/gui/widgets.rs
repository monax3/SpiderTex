use camino::Utf8PathBuf;
use eframe::egui::{Align, Color32, Layout, Response, RichText, Ui, WidgetText};
use eframe::epaint::Rect;

use super::theme;
use crate::log;

pub fn log_with_heading(ui: &mut Ui, heading: impl Into<WidgetText>) -> Response {
    ui.set_max_height(ui.available_height() - theme::BUTTON_HEIGHT);

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.label(heading);
    });
    ui.separator();

    ui.add(log::Logger)
}

pub fn file_name_label(
    ui: &mut Ui,
    file: impl Into<Utf8PathBuf>,
    label_func: impl Fn(String) -> WidgetText,
    tooltip_func: impl Fn(String) -> WidgetText,
) -> Response {
    let file: Utf8PathBuf = file.into();

    let file_name = file.file_name().unwrap_or(file.as_str()).to_string();
    let full_name = String::from(file);

    ui.label(label_func(file_name))
        .on_hover_text(tooltip_func(full_name))
}

pub fn file_name_labels<P>(
    ui: &mut Ui,
    files: impl IntoIterator<Item = P>,
    label_func: impl Fn(String) -> WidgetText + Copy,
    tooltip_func: impl Fn(String) -> WidgetText + Copy,
) where
    P: Into<Utf8PathBuf>,
{
    for file in files.into_iter() {
        file_name_label(ui, file, label_func, tooltip_func);
    }
}

pub fn debug_notification(ui: &mut Ui, mut rect: Rect) {
    const DEBUG_SIZE: f32 = 18.0;
    const DEBUG_COLOR: Color32 = Color32::RED;

    rect.min.y = rect.max.y - DEBUG_SIZE;

    let painter = ui.painter();

    let fill = ui.visuals().widgets.noninteractive.bg_fill;

    painter.rect(
        rect,
        eframe::egui::Rounding::same(DEBUG_SIZE / 2.0),
        fill,
        eframe::egui::Stroke::new(2.00, DEBUG_COLOR),
    );
    painter.text(
        rect.center(),
        eframe::egui::Align2::CENTER_CENTER,
        "DEBUG MODE",
        eframe::egui::FontId::monospace(DEBUG_SIZE),
        DEBUG_COLOR,
    );
}
