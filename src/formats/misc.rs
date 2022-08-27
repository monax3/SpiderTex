use std::fmt::Display;

use serde::{Deserialize, Serialize};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

use super::{dxgi, DxgiFormatExt};
use crate::texture_file;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq)]
pub struct Dimensions {
    pub data_size: usize,
    pub width: usize,
    pub height: usize,
    pub mipmaps: u8,
}

impl PartialEq for Dimensions {
    fn eq(&self, other: &Self) -> bool {
        self.wh() == other.wh()
    }
}

impl Dimensions {
    #[inline]
    #[must_use]
    pub const fn wh(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    #[inline]
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    #[inline]
    #[must_use]
    pub fn aspect_ratio_matches(self, other: Self) -> bool {
        const ACCURACY: f32 = 0.001;

        (self.aspect_ratio() - other.aspect_ratio().abs()) < ACCURACY
    }

    #[inline]
    #[must_use]
    pub fn mip_levels(self, is_highres: bool) -> u8 {
        if !(self.width.is_power_of_two() && self.height.is_power_of_two()) {
            1
        } else if is_highres {
            2
        } else {
            let pow2 = std::cmp::min(self.width, self.height).trailing_zeros();
            if pow2 <= 5 {
                1
            } else {
                (pow2 - 2) as u8
            }
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_for_file_size(self, size: usize) -> bool {
        size == self.data_size || (size == self.data_size + texture_file::TEXTURE_HEADER_SIZE)
    }
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}/{}", self.width, self.height, self.mipmaps)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum ColorPlanes {
    Rgb,
    #[default]
    Rgba,
    Luma,
    Hdr,
}

impl ColorPlanes {
    #[inline]
    #[must_use]
    pub const fn bpp(self) -> usize {
        match self {
            Self::Rgb => 3,
            Self::Rgba => 4,
            Self::Luma => 1,
            Self::Hdr => 16,
        }
    }

    #[inline]
    #[must_use]
    pub const fn bpp_dxgi(self) -> usize {
        match self {
            Self::Rgb | Self::Rgba => 4,
            Self::Luma => 1,
            Self::Hdr => 16,
        }
    }

    #[inline]
    #[must_use]
    pub const fn expected_formats(self) -> &'static [DXGI_FORMAT] {
        match self {
            Self::Rgb | Self::Rgba => dxgi::RGBA_FORMATS,
            Self::Hdr => dxgi::HDR_FORMATS,
            Self::Luma => dxgi::LUMA_FORMATS,
        }
    }

    //FIXME
    #[inline]
    #[must_use]
    pub fn is_expected_format(self, format: DXGI_FORMAT) -> bool {
        match self {
            Self::Rgba | Self::Rgb => format.is_rgb() || format.is_rgba(),
            Self::Hdr => format.is_hdr(),
            Self::Luma => format.is_luma(),
        }
    }
}

pub trait ImageFormatExt {
    #[must_use]
    fn can_save_array(self) -> bool;
}

impl ImageFormatExt for image::ImageFormat {
    #[must_use]
    fn can_save_array(self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Self::Dds => true,
            _ => false,
        }
    }
}
