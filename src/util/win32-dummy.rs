use crate::Result;
use camino::{Utf8Path, Utf8PathBuf};

pub fn initialize_com() -> Result<()> {
    Ok(())
}
pub fn open_files_dialog() -> Result<Option<Vec<Utf8PathBuf>>> {
    unimplemented!()
}
pub fn to_wstring(path: impl AsRef<Utf8Path>) -> Vec<u16> {
    unimplemented!()
}
pub fn message_box_ok(text: impl Into<String>, caption: &str) {
    unimplemented!()
}
pub fn message_box_error(text: impl Into<String>, caption: &str) {
    unimplemented!()
}
