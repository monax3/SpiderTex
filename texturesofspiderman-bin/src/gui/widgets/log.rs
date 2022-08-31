use eframe::egui::{Label, Layout, Response, ScrollArea, Sense, TextStyle, Ui, Widget};
use eframe::emath::{Align, Vec2};
use eframe::epaint::text::LayoutJob;
use eframe::epaint::{vec2, Color32, FontId, Rect};
use texturesofspiderman::prelude::LogFailure;
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

pub struct LogWidget<'a>(pub &'a LogReader<String>);

impl<'a> Widget for LogWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let text_style = TextStyle::Monospace;
        let line_height = ui.text_style_height(&text_style);
        let width = ui.available_width();

        let mut response = ui.allocate_response(Vec2::default(), Sense::hover());
        let font_id = text_style.resolve(ui.style());
        let text_color = ui.visuals().noninteractive().fg_stroke.color;
        ui.style_mut().override_text_style = Some(text_style);

        ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .min_scrolled_height(ui.available_height())
            .min_scrolled_width(ui.available_width())
            .show_rows(ui, line_height, self.0.len(), |ui, range| {
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    let level_width = text_width(ui, Level::ERROR.to_string(), font_id.clone())
                        + ui.style().spacing.item_spacing.x;

                    for i in range.rev() {
                        let (level, message) = &self.0[i];

                        let level_galley = ui.fonts().layout_job(LayoutJob::simple_singleline(
                            level.to_string(),
                            font_id.clone(),
                            level.color32(),
                        ));
                        let message_galley = ui.fonts().layout_job(LayoutJob::simple(
                            message.clone(),
                            font_id.clone(),
                            text_color,
                            width - level_width,
                        ));

                        let (rect, event_response) = ui.allocate_exact_size(
                            vec2(width, message_galley.rect.height()),
                            Sense::click(),
                        );

                        let mut context_menu = false;
                        let hovered = event_response.hovered();

                        response |= event_response.context_menu(|ui| {
                            if ui.button("Copy this line").clicked() {
                                arboard::Clipboard::new()
                                    .map(|mut clipboard| clipboard.set_text(message.clone()))
                                    .log_failure_as("Failed to copy text to clipboard").ignore();
                                ui.close_menu();
                            }

                            if ui.button("Save entire log").clicked() {
                                todo!()
                            }

                            context_menu = true;
                        });

                        if context_menu {
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                ui.visuals().widgets.active.bg_fill,
                            );
                        } else if hovered {
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                ui.visuals().widgets.hovered.bg_fill,
                            );
                        }

                        let level_rect = Rect::from_min_size(rect.min, level_galley.rect.size());
                        let message_rect = Rect::from_min_size(
                            rect.min + vec2(level_width, 0.0),
                            message_galley.rect.size(),
                        );

                        ui.put(level_rect, Label::new(level_galley));
                        ui.put(message_rect, Label::new(message_galley));
                    }
                });
            });
        response
    }
}

fn text_width(ui: &mut Ui, text: String, font_id: FontId) -> f32 {
    let galley = ui.fonts().layout_job(LayoutJob::simple_singleline(
        text,
        font_id,
        Color32::TEMPORARY_COLOR,
    ));

    galley.rect.width()
}
