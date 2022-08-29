use eframe::{
    egui::{Layout, Response, RichText, ScrollArea, Sense, TextStyle, Ui, Widget, WidgetText},
    emath::{Align, Vec2},
    epaint::{vec2, Color32},
};
pub use tracing::Level;
use tracing_messagevec::LogReader;

pub trait EguiColor {
    fn color32(&self) -> Color32;
}

impl EguiColor for Level {
    fn color32(&self) -> Color32 {
        match *self {
            level if level == Level::TRACE => Color32::LIGHT_BLUE,
            level if level == Level::DEBUG => Color32::DEBUG_COLOR,
            level if level == Level::INFO => Color32::LIGHT_GREEN,
            level if level == Level::WARN => Color32::GOLD,
            level if level == Level::ERROR => Color32::RED,
            _ => Color32::LIGHT_GRAY,
        }
    }
}

pub struct LogWidget<'a>(pub &'a LogReader<String>);

impl<'a> Widget for LogWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text_style = TextStyle::Monospace;
        let line_height = ui.text_style_height(&text_style);
        let mut response = ui.allocate_response(Vec2::default(), Sense::hover());

        ScrollArea::vertical().stick_to_bottom(true).show_rows(
            ui,
            line_height,
            self.0.len(),
            |ui, range| {
                for i in range {
                    let (level, message) = &self.0[i];
                    response |= ui
                        .horizontal_wrapped(|ui| {
                            ui.label(RichText::new(level.to_string()).text_style(text_style.clone()).color(level.color32()));
                            ui.label(RichText::new(message).text_style(text_style.clone()));
                        })
                        .response;
                }
            },
        );
        response
        // ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        //     for (level, message) in self.0.iter().rev() {
        //         if ui.available_height() <= 0.0 {
        //             break;
        //         }
        //         ui.horizontal_wrapped(|ui| {
        //             ui.label(log_level(*level));
        //             ui.label(log_text(message.clone()));
        //         });
        //     }
        // })
        // .response
    }
}

#[test]
fn test_log_colors() {
    use tracing::{event, Level};
    use tracing_subscriber::prelude::*;
    // tracing_subscriber::fmt().without_time().init();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(tracing::metadata::LevelFilter::TRACE)
        .init();

    event!(Level::TRACE, "trace");
    event!(Level::DEBUG, "debug");
    event!(Level::INFO, "info");
    event!(Level::WARN, "warn");
    event!(Level::ERROR, "error");
}
