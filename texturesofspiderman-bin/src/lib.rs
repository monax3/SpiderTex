
pub const APP_TITLE: &str = concat!("Spider-Man Texture Converter v", env!("CARGO_PKG_VERSION"));

use tracing_egui_repaint::oncecell::RepaintHandle;
use tracing_messagevec::LogReader;

#[cfg(windows)]
pub mod win32;
pub mod gui;

#[must_use]
pub fn setup_logging() -> (LogReader<String>, RepaintHandle) {
    use tracing_subscriber::prelude::*;
    use tracing_egui_repaint::oncecell::repaint_once_cell;

    let (log_reader, log_writer) = tracing_messagevec::new::<String>();
    let (repaint_layer, repaint_handle) =
        repaint_once_cell();

    let subscriber = tracing_subscriber::registry()
        .with(repaint_layer)
        .with(log_writer);

    // Turn off console logging in Windows release builds
    #[cfg(any(not(windows), debug_assertions))]
    let subscriber = subscriber.with(
        tracing_subscriber::fmt::layer()
            .without_time()
            .with_target(false),
    );

    subscriber.init();

    (log_reader, repaint_handle)
}

