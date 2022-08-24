use serde::{Deserialize, Serialize};

use super::Dimensions;
use super::DXGI_FORMAT;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct TextureFormat {
    #[serde(with = "super::dxgi::serde")]
    pub dxgi_format: DXGI_FORMAT,
    // pub stex_format:           (u8, u8),
    pub standard: Dimensions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highres: Option<Dimensions>,
    #[serde(default)]
    pub array_size: usize,
}

impl TextureFormat {
    #[inline]
    #[must_use]
    pub const fn all_dimensions(&self) -> (Dimensions, Option<Dimensions>) {
        if let Some(highres) = self.highres {
            (highres, Some(self.standard))
        } else {
            (self.standard, None)
        }
    }
}
