mod maybe_ready;
pub use maybe_ready::MaybeReady;
mod panic;
pub use panic::{catch_panics, downcast_str};
mod walkdir;
use camino::Utf8PathBuf;
pub use walkdir::{walkdir, WalkArgs};
#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::{open_files_dialog, message_box_ok, message_box_error};

use crate::prelude::*;

#[must_use]
pub fn into_n_slices(buffer: &[u8], num_slices: usize) -> Option<impl Iterator<Item = &[u8]>> {
    (buffer.len() % num_slices == 0).then(|| {
        let slice_len = buffer.len() / num_slices;
        let mut index = 0;

        std::iter::from_fn(move || {
            let offset = index * slice_len;
            index += 1;

            (offset < buffer.len()).then(|| &buffer[offset .. offset + slice_len])
        })
    })
}

#[inline]
#[must_use]
pub fn exe_dir_utf8() -> Option<Utf8PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(ToOwned::to_owned))
        .and_then(|dir| Utf8PathBuf::from_path_buf(dir).ok())
}

#[inline]
#[must_use]
pub fn current_dir_utf8() -> Option<Utf8PathBuf> {
    std::env::current_dir()
        .ok()
        .and_then(|dir| Utf8PathBuf::from_path_buf(dir).ok())
}

pub fn log_for_tests(verbose: bool) {
    use tracing_subscriber::prelude::*;

    let level = if verbose { TRACE } else { WARN };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(false)
                .without_time(),
        )
        .with(tracing_subscriber::filter::Targets::new().with_default(level))
        .init();
}
