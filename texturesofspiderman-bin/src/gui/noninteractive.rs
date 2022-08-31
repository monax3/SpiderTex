use eframe::{
    egui::{vec2, CentralPanel, ProgressBar, Style, TopBottomPanel, Visuals, Layout},
    epaint::FontId,
    run_native, App, NativeOptions, IconData,
};
use std::sync::mpsc;
use tracing::{event, Level};
use tracing_egui_repaint::oncecell::RepaintHandle;
use tracing_messagevec::{LogReader, LogWriter};
use texturesofspiderman::prelude::*;
use crate::APP_TITLE;
use crate::gui::widgets;

pub fn show(
    repaint_handle: RepaintHandle,
    progress_rx: mpsc::Receiver<f32>,
    log_reader: LogReader<String>,
) -> Result<()> {
    let window_size = vec2(400.0, 400.0);

    #[cfg(windows)]
    let icon_data = Some(crate::win32::icon_data()?);
    #[cfg(not(windows))] // FIXME, maybe
    let icon_data = None;

    run_native(
        APP_TITLE,
        NativeOptions {
            initial_window_size: Some(window_size),
            min_window_size: Some(vec2(200.0, 200.0)),
            drag_and_drop_support: false,
            resizable: true,
            icon_data,
            ..Default::default()
        },
        Box::new(move |cc| {
            repaint_handle.set_context(cc.egui_ctx.clone());

            cc.egui_ctx.set_visuals(Visuals::dark());
            cc.egui_ctx.set_style(Style {
                override_font_id: Some(FontId::proportional(16.0)),
                ..Style::default()
            });

            Box::new(Noninteractive {
                progress: 0.0,
                rx: progress_rx,
                log_reader,
            })
        }),
    );

    Ok(())
}

pub struct Noninteractive {
    progress: f32,
    rx: mpsc::Receiver<f32>,
    log_reader: LogReader<String>,
}

impl Noninteractive {
    fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }
}

impl App for Noninteractive {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        if let Some(progress) = self.rx.try_iter().last() {
            self.progress = progress;
        }

        TopBottomPanel::bottom("Progress").show(ctx, |ui| {
            let line_height = ui.text_style_height(&eframe::egui::TextStyle::Monospace);

            ui.with_layout(Layout::centered_and_justified(eframe::egui::Direction::TopDown), |ui| {

            ui.set_width(ui.available_width());
            ui.set_height(line_height * 2.0);

            if self.is_complete() {
                if ui.button("Close").clicked() {
                    frame.close();
                }
            } else {
                ui.add(ProgressBar::new(self.progress).animate(true));
            }

            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add(widgets::LogWidget(&self.log_reader))
        });
    }

    fn on_close_event(&mut self) -> bool {
        self.is_complete()
    }
}
