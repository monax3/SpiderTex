use eframe::{
    egui::{Layout, Response, Sense, Ui, Widget, WidgetText, RichText, TextStyle},
    emath::Align,
    epaint::{vec2, Color32},
};
pub use tracing::Level;

use tracing_messagevec::LogArc;
pub use tracing_messagevec::MessageVec;


fn log_text(text: impl Into<String>, level: Level) -> impl Into<WidgetText> {
    let color = match level {
        // TODO: add other log levels
        level if level == Level::ERROR => Color32::RED,
        level if level == Level::WARN => Color32::GOLD,
        _ => Color32::LIGHT_GRAY,
    };

    RichText::new(text)
        .text_style(TextStyle::Monospace)
        .size(12.0)
        .color(color)
}

pub struct LogWidget(pub LogArc<String>);

impl Widget for LogWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            for (level, message) in self.0.iter().rev() {
                if ui.available_height() <= 0.0 {
                    break;
                }
                ui.label(log_text(message.clone(), *level));
            }
        })
        .response
    }
}
