#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_code)] // no FFI without unsafe

use windows::core::{GUID, HRESULT};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

use super::DXImage;

pub type DXPtr = *mut DXOpaque;

#[repr(C)]
pub struct DXOpaque {
    _private: [u8; 0],
}

// FIXME
pub(super) type DDS_FLAGS = u32;
pub(super) type TGA_FLAGS = u32;
pub(super) type WIC_FLAGS = u32;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct TEX_FILTER_FLAGS(pub u32);

impl TEX_FILTER_FLAGS {
    #[cfg_attr(not(feature = "disable-wic"), allow(dead_code))]
    pub const TEX_FILTER_FORCE_NON_WIC: Self = Self(0x1000_0000);
    pub const TEX_FILTER_SRGB: Self =
        Self(Self::TEX_FILTER_SRGB_IN.0 | Self::TEX_FILTER_SRGB_OUT.0);
    pub const TEX_FILTER_SRGB_IN: Self = Self(0x100_0000);
    pub const TEX_FILTER_SRGB_OUT: Self = Self(0x200_0000);
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct TEX_DIMENSION(pub u32);

impl TEX_DIMENSION {
    pub const Texture1D: Self = Self(2);
    pub const Texture2D: Self = Self(3);
    pub const Texture3D: Self = Self(4);
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[repr(C)]
pub struct TexMetadata {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub arraySize: usize,
    pub mipLevels: usize,
    pub miscFlags: u32,
    pub miscFlags2: u32,
    pub format: DXGI_FORMAT,
    pub dimension: TEX_DIMENSION,
}
