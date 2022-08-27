use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use camino::Utf8PathBuf;
use eframe::egui::{vec2, Context, Event, Layout, Response, Sense, Ui, Widget};
use eframe::emath::Align;
use parking_lot::Mutex;
use texturesforspiderman::prelude::*;
use texturesforspiderman::util::MaybeReady;
use tracing_subscriber::prelude::*;

use crate::gui::theme;

pub static GLOBAL_LOG: Mutex<String> = Mutex::new(String::new());
pub static DEBUG_FLAG: AtomicBool = AtomicBool::new(cfg!(debug_assertions));

pub static UI_CONTEXT: MaybeReady<Context> = MaybeReady::new();

#[derive(Copy, Clone)]
pub struct Logger;

pub fn set_ui_context(ctx: &Context) { UI_CONTEXT.ready(ctx.clone()); }

// FIXME: move the global UI stuff to widgets
pub fn request_repaint() {
    if let Some(ctx) = UI_CONTEXT.try_get() {
        ctx.request_repaint();
    }
}

pub fn init() {
    #[cfg(debug_assertions)]
    let level = TRACE;
    #[cfg(not(debug_assertions))]
    let level = INFO;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .without_time()
                .with_writer(|| Logger)
                .with_filter(DebugFilter),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(tracing_subscriber::filter::Targets::new().with_default(level))
        .init();
}

struct DebugFilter;

pub fn save() -> Result<Utf8PathBuf, std::io::Error> {
    let log_file = std::env::current_exe()
        .ok()
        .and_then(|exe| Utf8PathBuf::from_path_buf(exe).ok())
        .unwrap_or_else(|| Utf8PathBuf::from(env!("CARGO_BIN_NAME")))
        .with_extension("log");

    let lock = GLOBAL_LOG.lock();
    let log_contents: &str = lock.as_ref();
    std::fs::write(&log_file, log_contents)?;

    Ok(log_file)
}

#[must_use]
pub const fn is_debug_toggle(event: &Event) -> bool {
    matches!(event, Event::Key {
        key: eframe::egui::Key::D,
        pressed: false,
        ..
    })
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
        let max_level = if debug { TRACE } else { INFO };

        meta.level() <= &max_level
    }
}

impl io::Write for Logger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let string =
            std::str::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        {
            GLOBAL_LOG.lock().push_str(string);
        }

        if let Some(ctx) = UI_CONTEXT.try_get() {
            ctx.request_repaint();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn parse_log_level(line: &str) -> tracing::Level {
    match &line[.. 5] {
        level if level == "ERROR" => ERROR,
        level if level == " WARN" => WARN,
        level if level == " INFO" => INFO,
        level if level == "DEBUG" => DEBUG,
        level if level == "TRACE" => TRACE,
        _ => INFO,
    }
}

impl Widget for Logger {
    fn ui(self, ui: &mut Ui) -> Response {
        {
            let lock = GLOBAL_LOG.lock();
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
        }

        ui.allocate_response(vec2(0.0, 0.0), Sense::hover())
    }
}
