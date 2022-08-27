//! TODO: allow use without the windows import and operate on a u32

use std::fmt::Display;

#[allow(clippy::wildcard_imports)] use windows::Win32::Graphics::Dxgi::Common::*;
pub use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

// FIXME: update these
pub const HDR_FORMATS: &[DXGI_FORMAT] = &[
    DXGI_FORMAT_R16G16B16A16_FLOAT,
    DXGI_FORMAT_R32G32B32A32_FLOAT,
];
// FIXME: add all the R8G8_ formats maybe
pub const LUMA_FORMATS: &[DXGI_FORMAT] = &[
    DXGI_FORMAT_R8_TYPELESS,
    DXGI_FORMAT_R8_UNORM,
    DXGI_FORMAT_R8_UINT,
    DXGI_FORMAT_R8_SNORM,
    DXGI_FORMAT_R8_UINT,
    DXGI_FORMAT_A8_UNORM,
];
pub const RGBA_FORMATS: &[DXGI_FORMAT] =
    &[DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB];
pub const BC1_FORMATS: &[DXGI_FORMAT] = &[
    DXGI_FORMAT_BC1_TYPELESS,
    DXGI_FORMAT_BC1_UNORM,
    DXGI_FORMAT_BC1_UNORM_SRGB,
];
pub const BC4_FORMATS: &[DXGI_FORMAT] = &[
    DXGI_FORMAT_BC4_TYPELESS,
    DXGI_FORMAT_BC4_SNORM,
    DXGI_FORMAT_BC4_UNORM,
];
pub const BC6_FORMATS: &[DXGI_FORMAT] = &[
    DXGI_FORMAT_BC6H_TYPELESS,
    DXGI_FORMAT_BC6H_UF16,
    DXGI_FORMAT_BC6H_SF16,
];
pub const BC7_FORMATS: &[DXGI_FORMAT] = &[
    DXGI_FORMAT_BC7_TYPELESS,
    DXGI_FORMAT_BC7_UNORM,
    DXGI_FORMAT_BC7_UNORM_SRGB,
];

#[allow(clippy::wrong_self_convention)]
pub trait DxgiFormatExt {
    #[must_use]
    fn display(self) -> DxgiFormatDisplay;
    #[must_use]
    fn compressed_format(self) -> Self;
    #[must_use]
    fn uncompressed_format(self) -> Self;
    #[must_use]
    fn is_bc1(self) -> bool;
    #[must_use]
    fn is_bc4(self) -> bool;
    #[must_use]
    fn is_bc6(self) -> bool;
    #[must_use]
    fn is_bc7(self) -> bool;
    #[must_use]
    fn is_rgb(self) -> bool;
    #[must_use]
    fn is_rgba(self) -> bool;
    #[must_use]
    fn is_luma(self) -> bool;
    #[must_use]
    fn is_hdr(self) -> bool;
    #[must_use]
    fn planes(self) -> ColorPlanes;
    #[must_use]
    fn is_compressed(self) -> bool;
    #[must_use]
    fn is_srgb(self) -> bool;
}

impl DxgiFormatExt for DXGI_FORMAT {
    #[inline]
    fn display(self) -> DxgiFormatDisplay { DxgiFormatDisplay(self) }

    #[inline]
    fn compressed_format(self) -> Self {
        #[allow(clippy::match_same_arms)]
        match self {
            DXGI_FORMAT_R8_UNORM => DXGI_FORMAT_BC4_UNORM,
            DXGI_FORMAT_R8G8B8A8_UNORM => DXGI_FORMAT_BC7_UNORM,
            DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => DXGI_FORMAT_BC7_UNORM_SRGB,
            DXGI_FORMAT_R32G32B32A32_FLOAT | DXGI_FORMAT_R16G16B16A16_FLOAT => {
                DXGI_FORMAT_BC6H_UF16
            }
            _ => self,
        }
    }

    #[inline]
    #[allow(clippy::match_same_arms)]
    fn uncompressed_format(self) -> Self {
        match self {
            DXGI_FORMAT_BC1_TYPELESS | DXGI_FORMAT_BC1_UNORM => DXGI_FORMAT_R8G8B8A8_UNORM,
            DXGI_FORMAT_BC1_UNORM_SRGB => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
            DXGI_FORMAT_BC4_TYPELESS | DXGI_FORMAT_BC4_UNORM | DXGI_FORMAT_BC4_SNORM => {
                DXGI_FORMAT_R8_UNORM
            }
            DXGI_FORMAT_BC6H_TYPELESS | DXGI_FORMAT_BC6H_UF16 | DXGI_FORMAT_BC6H_SF16 => {
                DXGI_FORMAT_R32G32B32A32_FLOAT
            }
            DXGI_FORMAT_BC7_TYPELESS | DXGI_FORMAT_BC7_UNORM => DXGI_FORMAT_R8G8B8A8_UNORM,
            DXGI_FORMAT_BC7_UNORM_SRGB => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
            _ => self,
        }
    }

    #[inline]
    #[must_use]
    fn is_bc1(self) -> bool { BC1_FORMATS.contains(&self) }

    #[inline]
    #[must_use]
    fn is_bc4(self) -> bool { BC4_FORMATS.contains(&self) }

    #[inline]
    #[must_use]
    fn is_bc6(self) -> bool { BC6_FORMATS.contains(&self) }

    #[inline]
    #[must_use]
    fn is_bc7(self) -> bool { BC7_FORMATS.contains(&self) }

    #[inline]
    #[must_use]
    fn is_rgb(self) -> bool { self.is_bc1() }

    #[inline]
    #[must_use]
    fn is_rgba(self) -> bool { self.is_bc7() || RGBA_FORMATS.contains(&self) }

    #[inline]
    #[must_use]
    fn is_luma(self) -> bool { self.is_bc4() || LUMA_FORMATS.contains(&self) }

    #[inline]
    #[must_use]
    fn is_hdr(self) -> bool { self.is_bc6() || HDR_FORMATS.contains(&self) }

    #[inline]
    #[must_use]
    fn planes(self) -> ColorPlanes {
        if self.is_rgb() {
            ColorPlanes::Rgb
        } else if self.is_luma() {
            ColorPlanes::Luma
        } else if self.is_hdr() {
            ColorPlanes::Hdr
        } else {
            ColorPlanes::Rgba
        }
    }

    #[inline]
    #[must_use]
    fn is_compressed(self) -> bool {
        // FIXME
        self.is_bc1() || self.is_bc4() || self.is_bc6() || self.is_bc7()
    }

    #[inline]
    #[must_use]
    fn is_srgb(self) -> bool { unimplemented!() } // FIXME
}

pub struct DxgiFormatDisplay(DXGI_FORMAT);

impl Display for DxgiFormatDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.0 {
            DXGI_FORMAT_BC1_UNORM => "BC1",
            DXGI_FORMAT_BC1_UNORM_SRGB => "BC1 sRGB",
            DXGI_FORMAT_BC2_UNORM => "BC2",
            DXGI_FORMAT_BC2_UNORM_SRGB => "BC2 sRGB",
            DXGI_FORMAT_BC3_UNORM => "BC3",
            DXGI_FORMAT_BC3_UNORM_SRGB => "BC3 sRGB",
            DXGI_FORMAT_BC4_UNORM => "BC4",
            DXGI_FORMAT_BC5_UNORM => "BC5",
            DXGI_FORMAT_BC6H_UF16 => "BC6",
            DXGI_FORMAT_BC7_UNORM => "BC7",
            DXGI_FORMAT_BC7_UNORM_SRGB => "BC7 sRGB",
            DXGI_FORMAT_R8G8B8A8_UNORM => "RGBA8",
            DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => "RGBA8 sRGB",
            DXGI_FORMAT_R8_UNORM => "Luma",
            DXGI_FORMAT_R32G32B32A32_FLOAT => "HDR 32f",
            DXGI_FORMAT_R16G16B16A16_FLOAT => "HDR 16f",
            _ => return write!(f, "{:?}", self.0),
        })
    }
}

pub mod serde {
    use serde::Deserialize;
    use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: serde::Serializer>(
        format: &DXGI_FORMAT,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(format.0)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DXGI_FORMAT, D::Error> {
        Ok(DXGI_FORMAT(u32::deserialize(deserializer)?))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ColorPlanes {
    Rgb,
    Rgba,
    Luma,
    Hdr,
}
