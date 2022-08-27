use camino::Utf8Path;
use windows::core::PCSTR;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

use crate::Result;
static COM_INITIALIZED: AtomicBool = AtomicBool::new(false);
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxA,
    MB_ICONERROR,
    MB_TASKMODAL,
    MESSAGEBOX_STYLE,
};

mod open_files;
pub use open_files::open_files_dialog;

pub fn initialize_com() -> Result<()> {
    if !COM_INITIALIZED.load(Ordering::Acquire) {
        #[allow(unsafe_code)]
        unsafe {
            CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED)?;
        }
        COM_INITIALIZED.store(true, Ordering::Release);
    }
    Ok(())
}

#[inline]
#[must_use]
pub fn to_wstring(path: impl AsRef<Utf8Path>) -> Vec<u16> {
    path.as_ref()
        .as_str()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}

pub fn message_box_ok(text: impl Into<String>, caption: &str) {
    message_box(text, caption, MB_TASKMODAL);
}

pub fn message_box_error(text: impl Into<String>, caption: &str) {
    message_box(text, caption, MB_TASKMODAL | MB_ICONERROR);
}

pub fn message_box(text: impl Into<String>, caption: &str, flags: MESSAGEBOX_STYLE) {
    let caption = to_cstring(caption);
    let text = to_cstring(text);

    #[allow(unsafe_code)]
    unsafe {
        MessageBoxA(
            None,
            PCSTR(text.as_ptr().cast::<u8>()),
            PCSTR(caption.as_ptr().cast::<u8>()),
            flags,
        );
    }
}

fn to_cstring(text: impl Into<String>) -> CString {
    let string: String = text.into();
    let mut string = string.replace('\n', "\r\n");

    if let Some(nul) = string.bytes().position(|b| b == b'\0') {
        string.truncate(nul);
    }

    // Safety: We've removed interior nuls so this should meet all invariants
    #[allow(unsafe_code)]
    unsafe {
        CString::from_vec_unchecked(string.into_bytes())
    }
}
