use eframe::egui::style::{Margin, Spacing};
use eframe::egui::{vec2, Color32, FontId, RichText, Style, TextStyle, Vec2, Visuals, WidgetText};
use eframe::epaint::FontFamily;
use texturesforspiderman::prelude::*;

pub const SIDEBAR_WIDTH: f32 = 300.0;
pub const EXTRA_SPACING: f32 = 10.0;
pub const TEXT_HIGHLIGHT_COLOR: Color32 = Color32::LIGHT_GREEN;

pub const PREVIEW_FRAME_SIZE: f32 = 4.0;
pub const PREVIEW_FRAME_COLOR: Color32 = Color32::LIGHT_GRAY;
pub const PREVIEW_SIZE: f32 = 512.0;

pub const BUTTON_HEIGHT: f32 = 50.0;
pub const STATUS_HEIGHT: f32 = 250.0;

pub const BUTTON_FONT_SIZE: f32 = 30.0;

pub mod colors {
    use eframe::egui::Color32;

    pub const TEXT: Color32 = Color32::LIGHT_GRAY;
    pub const TEXT_WEAK: Color32 = Color32::GRAY;
    pub const TEXT_HIGHLIGHT: Color32 = Color32::WHITE;
    pub const TEXT_GOOD: Color32 = Color32::LIGHT_GREEN;
    pub const TEXT_WARNING: Color32 = Color32::GOLD;
    pub const TEXT_ERROR: Color32 = Color32::RED;
}

pub mod text {
    use eframe::egui::{RichText, WidgetText};

    pub fn normal(text: impl Into<String>) -> WidgetText {
        RichText::new(text).color(super::colors::TEXT).into()
    }

    pub fn weak(text: impl Into<String>) -> WidgetText {
        RichText::new(text).color(super::colors::TEXT_WEAK).into()
    }

    pub fn highlight(text: impl Into<String>) -> WidgetText {
        RichText::new(text)
            .color(super::colors::TEXT_HIGHLIGHT)
            .into()
    }

    pub fn good(text: impl Into<String>) -> WidgetText {
        RichText::new(text).color(super::colors::TEXT_GOOD).into()
    }

    pub fn warning(text: impl Into<String>) -> WidgetText {
        RichText::new(text)
            .color(super::colors::TEXT_WARNING)
            .into()
    }

    pub fn error(text: impl Into<String>) -> WidgetText {
        RichText::new(text).color(super::colors::TEXT_ERROR).into()
    }

    pub fn button(text: impl Into<String>) -> WidgetText {
        RichText::new(text)
            .color(super::colors::TEXT_HIGHLIGHT)
            .size(super::BUTTON_FONT_SIZE)
            .into()
    }
}

pub fn button_size() -> Vec2 { vec2(120.0, BUTTON_HEIGHT) }

pub fn nav_button_size() -> Vec2 { vec2(30.0, 30.0) }

pub fn visuals() -> Visuals { Visuals::dark() }

pub fn button_text(text: impl Into<String>) -> impl Into<WidgetText> {
    RichText::new(text).strong().size(20.0)
}

pub fn highlight_text(text: impl Into<String>) -> impl Into<WidgetText> {
    RichText::new(text).strong().color(TEXT_HIGHLIGHT_COLOR)
}

pub fn log_text(text: impl Into<String>, level: tracing::Level) -> impl Into<WidgetText> {
    let color = match level {
        // TODO: add other log levels
        level if level == ERROR => Color32::RED,
        level if level == WARN => Color32::GOLD,
        _ => Color32::LIGHT_GRAY,
    };

    RichText::new(text)
        .text_style(TextStyle::Monospace)
        .size(12.0)
        .color(color)
}

pub fn spacing() -> Spacing { Spacing::default() }

pub fn window_size() -> Vec2 {
    let spacing = spacing();

    let size = vec2(spacing.item_spacing.x.mul_add(2.0, SIDEBAR_WIDTH), 0.0);

    size + spacing.window_margin.sum() + preview_size_with_frame()
}

pub fn preview_size() -> Vec2 { Vec2::splat(PREVIEW_SIZE) }

pub fn preview_size_with_frame() -> Vec2 {
    Vec2::splat(PREVIEW_SIZE) + Vec2::splat(PREVIEW_FRAME_SIZE * 2.0)
}

pub fn style() -> Style {
    Style {
        override_font_id: Some(text_font()),
        ..Style::default()
    }
}

pub fn text_font() -> FontId { FontId::proportional(16.0) }
