#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{Result, Error, HRESULT, HSTRING, GUID};

#[cfg(not(windows))]
mod compat;
#[cfg(not(windows))]
pub use compat::{Result, Error, HRESULT, HSTRING, GUID};

pub mod errors;
