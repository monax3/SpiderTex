use eframe::egui::{Response, WidgetText, Ui, Layout, Align};
use crate::{theme, log};

pub fn log_with_heading(ui: &mut Ui, heading: impl Into<WidgetText>) -> Response {
    ui.set_max_height(ui.available_height() - theme::BUTTON_HEIGHT);

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.label("Working");
    });
    ui.separator();

    ui.add(log::Logger)
}
