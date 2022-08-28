#[cfg_attr(windows, path = "windows.rs")]
mod compat;

pub use compat::{Result, Error, HRESULT, HSTRING, GUID};
pub mod errors;
