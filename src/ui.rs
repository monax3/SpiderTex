use std::ffi::CString;

use windows::core::PCSTR;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxA,
    MB_ICONERROR,
    MB_TASKMODAL,
    MESSAGEBOX_STYLE,
};

pub fn message_box_ok(caption: &str, text: &str) { message_box(text, caption, MB_TASKMODAL); }

pub fn message_box_error(caption: &str, text: &str) {
    message_box(text, caption, MB_TASKMODAL | MB_ICONERROR);
}

pub fn message_box(caption: &str, text: &str, flags: MESSAGEBOX_STYLE) {
    let caption = to_cstring(caption);
    let text = to_cstring(text);

    unsafe {
        MessageBoxA(
            None,
            PCSTR(caption.as_ptr().cast::<u8>()),
            PCSTR(text.as_ptr().cast::<u8>()),
            flags,
        );
    }
}

fn to_cstring(text: impl Into<String>) -> CString {
    let mut string = text.into();

    if let Some(nul) = string.bytes().position(|b| b == b'\0') {
        string.truncate(nul);
    }

    // Safety: We've removed interior nuls so this should meet all invariants
    unsafe { CString::from_vec_unchecked(string.into_bytes()) }
}
