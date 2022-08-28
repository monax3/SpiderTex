#![allow(unsafe_code)]

use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED, CoUninitialize};

use windows::core::Result;
static COM_INITIALIZED: AtomicBool = AtomicBool::new(false);
use std::sync::atomic::{AtomicBool, Ordering};

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

pub fn com_initialized() -> Result<COMInitialized> {
    initialize_com().map(|_| COMInitialized)
}

pub struct COMInitialized;

impl Drop for COMInitialized {
    fn drop(&mut self) {
        unsafe { CoUninitialize(); }
    }
}