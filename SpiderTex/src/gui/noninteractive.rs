use eframe::{
    egui::{vec2, CentralPanel, ProgressBar, Style, TopBottomPanel, Visuals, Layout},
    epaint::FontId,
    run_native, App, NativeOptions,
};
use std::sync::mpsc;
use tracing::{event, Level};
use tracing_egui_repaint::oncecell::RepaintHandle;
use tracing_messagevec::{LogReader, LogWriter};

use super::widgets::log::LogWidget;
use crate::APP_TITLE;

#[cfg(disabled)]
pub fn test_log() {
    use tracing_subscriber::prelude::*;

    let (log_reader, log_writer) = tracing_messagevec::new::<String>();
    let (repaint_layer, repaint_handle) =
        tracing_oncecell::OnceCellLayer::<tracing_egui_repaint::Repaint>::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false),
        )
        .with(repaint_layer)
        .with(log_writer)
        .init();

    event!(Level::DEBUG, "debug message");
    event!(Level::TRACE, "trace message");
    event!(Level::WARN, "a warning");
    event!(Level::ERROR, "this is an error");

    eprintln!("printing log:");
    for (level, message) in log_reader.iter() {
        eprintln!("{level:>5} {message}");
    }
}

pub fn show(
    repaint_handle: RepaintHandle,
    progress_rx: mpsc::Receiver<f32>,
    log_reader: LogReader<String>,
) {
    let window_size = vec2(500.0, 600.0);

    run_native(
        APP_TITLE,
        NativeOptions {
            initial_window_size: Some(window_size),
            min_window_size: Some(window_size),
            drag_and_drop_support: false,
            resizable: true,
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
                state: State::Waiting,
                rx: progress_rx,
                log_reader,
            })
        }),
    );
}

enum State {
    Waiting,
    Progress(f32),
    Complete,
}

impl State {
    fn as_f32(&self) -> f32 {
        match self {
            State::Waiting => 0.0,
            State::Progress(progress) => *progress,
            State::Complete => 1.0,
        }
    }

    fn is_complete(&self) -> bool {
        matches!(self, State::Complete)
    }
}

impl From<f32> for State {
    fn from(progress: f32) -> Self {
        if progress <= 0.0 {
            State::Waiting
        } else if progress >= 1.0 {
            State::Complete
        } else {
            State::Progress(progress)
        }
    }
}

pub struct Noninteractive {
    state: State,
    rx: mpsc::Receiver<f32>,
    log_reader: LogReader<String>,
}

impl App for Noninteractive {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Some(progress) = self.rx.try_iter().last() {
            self.state = progress.into();
        }

        // TODO: turn this into a close button when complete
        TopBottomPanel::bottom("Progress").show(ctx, |ui| {
            let line_height = ui.text_style_height(&eframe::egui::TextStyle::Monospace);

            ui.with_layout(Layout::centered_and_justified(eframe::egui::Direction::TopDown), |ui| {

            ui.set_width(ui.available_width());
            ui.set_height(line_height * 2.0);

            if self.state.is_complete() {
                ui.button("Close");
            } else {
                ui.add(ProgressBar::new(self.state.as_f32()).animate(true));
            }

            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add(LogWidget(&self.log_reader));
        });
    }

    fn on_close_event(&mut self) -> bool {
        self.state.is_complete()
    }
}
