use camino::{Utf8Path, Utf8PathBuf};
use eframe::egui::{
    vec2,
    Button,
    CentralPanel,
    Context,
    Event,
    Layout,
    RadioButton,
    Rect,
    Response,
    RichText,
    Sense,
    SidePanel,
    TextStyle,
    Ui,
    Widget,
};
use eframe::emath::Align;
use eframe::{run_native, App, Frame as EFrame, NativeOptions};
use image::DynamicImage;
use texturesofspiderman::formats::TextureFormat;
use texturesofspiderman::prelude::*;

use super::preview::Preview;
use super::{theme, widgets};
use crate::log;

pub fn show<APP>(mut app: APP)
where APP: AppWindow + 'static {
    let options = NativeOptions {
        initial_window_size: Some(theme::window_size()),
        min_window_size: Some(theme::window_size()),
        drag_and_drop_support: false,
        resizable: true,
        ..Default::default()
    };

    run_native(
        crate::APP_TITLE,
        options,
        Box::new(move |cc| {
            // app.ctx(&cc.egui_ctx);

            cc.egui_ctx.set_visuals(theme::visuals());
            cc.egui_ctx.set_style(theme::style());

            log::set_ui_context(&cc.egui_ctx);

            Box::new(Window(app))
        }),
    );
}

pub trait AppWindow {
    fn update(&mut self);
    fn sidebar(&mut self, ui: &mut Ui);
    fn main(&mut self, ui: &mut Ui);
    fn can_close(&mut self) -> bool { true }
    fn constant_refresh(&mut self) -> bool { false }

    // #[allow(unused)]
    // fn ctx(&mut self, ctx: &Context) {}

    fn run(self)
    where Self: Sized + 'static {
        show(self);
    }
}

struct Window<APP: AppWindow>(APP);

impl<APP: AppWindow> App for Window<APP> {
    fn update(&mut self, ctx: &Context, _frame: &mut EFrame) {
        for event in &ctx.input().events {
            if log::is_debug_toggle(event) {
                log::toggle_debug();
            }
        }

        self.0.update();

        SidePanel::left("Status")
            .min_width(theme::SIDEBAR_WIDTH)
            .resizable(false)
            .show(ctx, |ui| self.0.sidebar(ui));

        CentralPanel::default().show(ctx, |ui| {
            self.0.main(ui);
        });

        if self.0.constant_refresh() {
            ctx.request_repaint();
        }
    }

    // fn on_exit(&mut self, _gl: &eframe::glow::Context) {}
}
