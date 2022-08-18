use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use eframe::egui::{vec2, Context, Layout, Response, Sense, Ui, Widget, Event};
use eframe::emath::Align;
use tracing::Level;
use tracing_subscriber::prelude::*;

use crate::theme;
use crate::util::MaybeReady;

pub static GLOBAL_LOG: Mutex<String> = Mutex::new(String::new());
pub static DEBUG_FLAG: AtomicBool = AtomicBool::new(cfg!(debug_assertions));

pub static UI_CONTEXT: MaybeReady<Context> = MaybeReady::new();

#[derive(Copy, Clone)]
pub struct Logger;

pub fn set_ui_context(ctx: &Context) { UI_CONTEXT.ready(ctx.clone()); }

pub fn init() {
    #[cfg(debug_assertions)]
    let level = Level::TRACE;
    #[cfg(not(debug_assertions))]
    let level = Level::INFO;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .without_time()
                .with_writer(|| Logger)
                .with_filter(DebugFilter),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::Targets::new().with_default(level))
        .init();
}

struct DebugFilter;

pub fn is_debug_toggle(event: &Event) -> bool {
    matches!(event, Event::Key { key: eframe::egui::Key::D, pressed: false, .. })
}

pub fn toggle_debug() {
    let flag = DEBUG_FLAG.load(Ordering::Acquire);

    DEBUG_FLAG.store(!flag, Ordering::Release);
}

pub fn debug_enabled() -> bool { DEBUG_FLAG.load(Ordering::Relaxed) }

impl<S> tracing_subscriber::layer::Filter<S> for DebugFilter {
    fn enabled(
        &self,
        meta: &tracing::Metadata<'_>,
        _cx: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        let debug = debug_enabled();
        let max_level = if debug { Level::TRACE } else { Level::INFO };

        meta.level() <= &max_level
    }
}

impl io::Write for Logger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let string =
            std::str::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        GLOBAL_LOG
            .lock()
            .map(|mut lock| {
                lock.push_str(string);

                buf.len()
            })
            .map_err(|_err| io::ErrorKind::Other.into())
            .map(|len| {
                if let Some(ctx) = UI_CONTEXT.get() {
                    ctx.request_repaint();
                }

                len
            })
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn parse_log_level(line: &str) -> Level {
    match &line[.. 5] {
        level if level == "ERROR" => Level::ERROR,
        level if level == " WARN" => Level::WARN,
        level if level == " INFO" => Level::INFO,
        level if level == "DEBUG" => Level::DEBUG,
        level if level == "TRACE" => Level::TRACE,
        _ => Level::INFO,
    }
}

impl Widget for Logger {
    fn ui(self, ui: &mut Ui) -> Response {
        let lock = GLOBAL_LOG.lock().unwrap();
        let lines = lock.lines().rev();

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            for line in lines {
                if ui.available_height() <= 0.0 {
                    break;
                }
                let level = parse_log_level(line);
                ui.label(theme::log_text(line.to_owned(), level));
            }
        });

        // eframe::egui::ScrollArea::vertical()
        //     .enable_scrolling(false)
        //     .stick_to_bottom()
        //     .show(ui, |ui| {
        //         for line in lines {
        //             let level = parse_log_level(line);
        //             ui.label(theme::log_text(line.to_owned(), level));
        //         }
        //     });

        ui.allocate_response(vec2(0.0, 0.0), Sense::hover())
    }
}
