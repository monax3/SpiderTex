//! FIXME: use panic handling code elsewhere
#![allow(unsafe_code)] // no FFI without unsafe

use std::ffi::CString;
use std::fmt::Write;

use backtrace::Backtrace;
use camino::Utf8PathBuf;
use tracing::{event, Level};
use windows::core::{w, HSTRING, PCSTR, PCWSTR};
use windows::Win32::Foundation::ERROR_CANCELLED;
use windows::Win32::System::Com::{
    CoCreateInstance,
    CoInitializeEx,
    CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use windows::Win32::UI::Shell::{
    FileOpenDialog,
    IFileOpenDialog,
    IShellItem,
    IShellItemArray,
    SHCreateItemFromParsingName,
    FILEOPENDIALOGOPTIONS,
    FOS_ALLOWMULTISELECT,
    FOS_FORCEFILESYSTEM,
    SIGDN_FILESYSPATH,
};

use crate::prelude::*;
use crate::util::downcast_str;

fn is_std_panic(symbol: &backtrace::BacktraceSymbol) -> bool {
    symbol
        .name()
        .and_then(|name| name.as_str())
        .map_or(false, |name| {
            name.ends_with("::panicking::panic") || name.ends_with("::panicking::panic_fmt")
            // (name.starts_with("core::") || name.starts_with("std::")) &&
            // name.contains("panicking")
        })
}

/// XXX: This is a real hacky function but this is a Windows app so it should
/// work
fn is_absolute_path(path: &std::path::Path) -> bool {
    let path = path.display().to_string();
    println!("{path}");

    let mut iter = path.chars();

    if let Some(start) = iter.next() {
        if std::path::is_separator(start) {
            return true;
        }
    }

    if let Some(drive_sep) = iter.next() {
        if drive_sep == ':' {
            return true;
        }
    }

    false
}

#[allow(clippy::needless_pass_by_value)]
fn backtrace_to_string(backtrace: Backtrace) -> String {
    let mut out = String::new();

    let frame_iter = backtrace.frames().iter().rev();

    for frame in frame_iter {
        if frame.symbols().iter().any(is_std_panic) {
            break;
        }

        for symbol in frame.symbols() {
            match (symbol.filename(), symbol.lineno(), symbol.name()) {
                (Some(filename), Some(lineno), Some(name)) /* if is_absolute_path(filename) */ => {
                    let filename = filename.to_string_lossy();
                    let _ignored = writeln!(out, "- {name} ({filename}:{lineno})");
                }
                (_, _, Some(name)) => {
                    let _ignored = writeln!(out, "- {name}");
                }
                _ => {
                    let _ignored = writeln!(out, "- {symbol:?}");
                }
            }
        }
    }

    out
}

fn message_box_on_panic(panic: &std::panic::PanicInfo<'_>) {
    let ctx = downcast_str(panic.payload());

    let backtrace = backtrace_to_string(Backtrace::new());

    ctx.map_or_else(
        || event!(ERROR, "Panic, dumping backtrace:\n{backtrace}"),
        |ctx| event!(ERROR, "{ctx}, dumping backtrace:\n{backtrace}"),
    );

    // FIXME: import fix this
    // let log_message = match log::save() {
    //     Ok(log_file) => format!("A debug log has been saved to {log_file}."),
    //     Err(error) => format!("The debug log failed to save: {error}."),
    // };

    // let mut message = format!("An unrecoverable error occurred. {log_message}");
    let mut message = "An unrecoverable error occurred".to_string();

    if let Some(ctx) = downcast_str(panic.payload()) {
        let _ignored = write!(message, "\n\nError: {ctx}");
    }

    super::message_box_error(message, crate::APP_TITLE);
}

pub fn init() {
    std::panic::set_hook(Box::new(message_box_on_panic));

    unsafe {
        CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED).expect("Initializing COM failed");
    }
}

fn make_filter_spec(exts: &[&str]) -> String {
    let iter = exts
        .windows(2)
        .map(|exts| (exts[0], Some(exts[1])))
        .chain(exts.last().map(|a| (*a, None)));

    iter.map(|(a, b)| {
        if b.is_some() {
            format!("*.{a};")
        } else {
            format!("*.{a}")
        }
    })
    .collect()
}

#[test]
fn test_filter_spec() {
    println!("{}", make_filter_spec(SUPPORTED_IMAGE_EXTENSIONS));
    println!("{}", make_filter_spec(SUPPORTED_IMAGE_EXTENSIONS));
}

#[allow(clippy::string_lit_as_bytes)]
pub fn open_files_dialog() -> Result<Option<Vec<Utf8PathBuf>>> {
    let image_spec = HSTRING::from(make_filter_spec(SUPPORTED_IMAGE_EXTENSIONS));
    let texture_spec = HSTRING::from(make_filter_spec(SUPPORTED_TEXTURE_EXTENSIONS));

    let filters = [
        COMDLG_FILTERSPEC {
            pszName: w!("Textures").into(),
            pszSpec: PCWSTR::from(&texture_spec),
        },
        COMDLG_FILTERSPEC {
            pszName: w!("Images").into(),
            pszSpec: PCWSTR::from(&image_spec),
        },
        COMDLG_FILTERSPEC {
            pszName: w!("All files").into(),
            pszSpec: w!("*.*").into(),
        },
    ];

    unsafe {
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(FILEOPENDIALOGOPTIONS(
            FOS_FORCEFILESYSTEM.0 | FOS_ALLOWMULTISELECT.0,
        ))?;
        dialog.SetFileTypes(&filters)?;
        dialog.SetTitle(w!("Select the files to process"))?;

        if let Some(exe_dir) = std::env::current_exe().ok().and_then(|exe| {
            exe.parent()
                .map(|dir| HSTRING::from(dir.display().to_string()))
        }) {
            let shi: IShellItem = SHCreateItemFromParsingName(&exe_dir, None)?;

            dialog.SetDefaultFolder(&shi)?;
        }

        match dialog.Show(None) {
            Ok(()) => Ok(Some(shell_items_to_paths(&dialog.GetResults()?)?)),
            Err(error) if error == ERROR_CANCELLED.into() => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

unsafe fn shell_items_to_paths(items: &IShellItemArray) -> Result<Vec<Utf8PathBuf>> {
    Ok((0 .. items.GetCount()?)
        .into_iter()
        .filter_map(|i| {
            items
                .GetItemAt(i)
                .and_then(|item| item.GetDisplayName(SIGDN_FILESYSPATH).log_failure())
                .map(|name| name.to_string().ok().map(Utf8PathBuf::from))
                .transpose()
        })
        .collect::<Result<_, _>>()?)
}
