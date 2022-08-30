use eframe::{
    egui::{
        text::LayoutSection, Label, Layout, Response, RichText, ScrollArea, Sense, TextFormat,
        TextStyle, Ui, Widget, WidgetText,
    },
    emath::{Align, Vec2},
    epaint::{
        text::{LayoutJob, TextWrapping},
        vec2, Color32, FontId,
    },
};
pub use tracing::Level;
use tracing_messagevec::LogReader;

pub trait EguiColor {
    fn color32(&self) -> Color32;
}

impl EguiColor for Level {
    fn color32(&self) -> Color32 {
        match *self {
            level if level == Self::TRACE => Color32::LIGHT_BLUE,
            level if level == Self::DEBUG => Color32::DEBUG_COLOR,
            level if level == Self::INFO => Color32::LIGHT_GREEN,
            level if level == Self::WARN => Color32::GOLD,
            level if level == Self::ERROR => Color32::RED,
            _ => Color32::LIGHT_GRAY,
        }
    }
}

fn layout_job(font_id: FontId, level: Level, message: &str, max_width: f32) -> LayoutJob {
    let message = format!("{level:>5} {message}");
    let len = message.len();

    LayoutJob {
        text: message,
        sections: vec![
            LayoutSection {
                leading_space: 0.0,
                byte_range: 0..5,
                format: TextFormat {
                    font_id: font_id.clone(),
                    color: level.color32(),
                    ..Default::default()
                },
            },
            LayoutSection {
                leading_space: 0.0,
                byte_range: 5..len,
                format: TextFormat {
                    font_id,
                    color: Color32::WHITE,
                    ..Default::default()
                },
            },
        ],
        wrap: TextWrapping {
            max_width,
            max_rows: 0,
            break_anywhere: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub struct LogWidget<'a>(pub &'a LogReader<String>);

impl<'a> Widget for LogWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text_style = TextStyle::Monospace;
        let line_height = ui.text_style_height(&text_style);
        let mut response = ui.allocate_response(Vec2::default(), Sense::hover());
        let font_id = text_style.resolve(ui.style());
        ui.style_mut().override_text_style = Some(text_style);
        // ui.set_width(ui.available_width());
        // ui.set_height(ui.available_height());

        // ui.with_layout(Layout::centered_and_justified(eframe::egui::Direction::BottomUp), |ui| {
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .min_scrolled_height(ui.available_height())
            .min_scrolled_width(ui.available_width())
            .show_rows(ui, line_height, self.0.len(), |ui, range| {
                //     ui.set_width(ui.available_width());
                // ui.set_height(ui.available_height());

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    let level_width = ui.fonts().glyph_width(&font_id, 'X') * 6.0;

                    // ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    // ui.set_width(ui.available_width());
                    // ui.set_height(ui.available_height());

                    // for (level, message) in self.0.iter().rev() {
                    for i in range.rev() {
                        let (level, message) = &self.0[i];
                        response |= ui
                            .horizontal_wrapped(|ui| {
                                ui.colored_label(level.color32(), level.to_string());
                                ui.label(message);
                            })
                            .response;
                    }
                });
            });
        response
    }
}
