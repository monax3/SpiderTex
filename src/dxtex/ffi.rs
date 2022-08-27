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

extern "C" {
    pub fn V2_FreeImage(image: DXPtr) -> HRESULT;
    pub fn BufferSize(image: DXPtr) -> usize;
    pub fn Buffer(image: DXPtr, dst: *mut u8, dst_len: usize) -> HRESULT;
    pub fn ImageCount(image: DXPtr) -> usize;
    pub fn ImageSize(image: DXPtr, array_index: usize) -> usize;
    pub fn ImageData(image: DXPtr, array_index: usize, dst: *mut u8, dst_len: usize) -> HRESULT;
    pub fn V2_GetMetadata(image: DXPtr, metadata: *mut TexMetadata) -> HRESULT;
    pub fn V2_New1D(
        format: DXGI_FORMAT,
        size: usize,
        array_size: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut DXImage,
    ) -> HRESULT;
    pub fn V2_New2D(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        array_size: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut DXImage,
    ) -> HRESULT;
    pub fn GenerateMipmaps(
        image: DXPtr,
        flags: TEX_FILTER_FLAGS,
        mipmaps: usize,
        dst: *mut DXImage,
    ) -> HRESULT;
    pub fn Resize(
        image: DXPtr,
        width: usize,
        height: usize,
        flags: TEX_FILTER_FLAGS,
        dst: *mut DXImage,
    ) -> HRESULT;
    pub fn V2_Compress(image: DXPtr, to_format: DXGI_FORMAT, dst: *mut DXImage) -> HRESULT;
    pub fn V2_Decompress(image: DXPtr, dst: *mut DXImage) -> HRESULT;
    pub fn V2_Convert(
        image: DXPtr,
        to_format: DXGI_FORMAT,
        flags: TEX_FILTER_FLAGS,
        dst: *mut DXImage,
    ) -> HRESULT;
    pub fn V2_PremultiplyAlpha(image: DXPtr, reverse: bool, dst: *mut DXImage) -> HRESULT;
    pub fn V2_SaveDDS(image: DXPtr, file: *const u16, flags: u64) -> HRESULT;
    pub fn V2_SaveTGA(image: DXPtr, array_index: usize, file: *const u16, flags: u64) -> HRESULT;
    pub fn V2_SaveHDR(image: DXPtr, array_index: usize, file: *const u16) -> HRESULT;
    pub fn V2_SaveEXR(image: DXPtr, array_index: usize, file: *const u16) -> HRESULT;
    pub fn IsCompressed(format: DXGI_FORMAT) -> i32;
    pub fn IsSRGB(format: DXGI_FORMAT) -> i32;

    pub fn Clone(image: DXPtr, dst: *mut DXImage) -> HRESULT;

    pub fn ExpectedSize(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        depth: usize,
        mipmaps: u8,
    ) -> usize;

    pub fn ExpectedSizeArray(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        array_size: usize,
        mipmaps: u8,
    ) -> usize;

    pub fn LoadFromDDSFile(
        file: *const u16,
        dds_flags: DDS_FLAGS,
        metadata: *mut TexMetadata,
        out: *mut DXImage,
    ) -> HRESULT;
    pub fn LoadFromTGAFile(
        file: *const u16,
        tga_flags: TGA_FLAGS,
        metadata: *mut TexMetadata,
        out: *mut DXImage,
    ) -> HRESULT;
    pub fn LoadFromHDRFile(
        file: *const u16,
        metadata: *mut TexMetadata,
        out: *mut DXImage,
    ) -> HRESULT;
    pub fn LoadFromEXRFile(
        file: *const u16,
        metadata: *mut TexMetadata,
        out: *mut DXImage,
    ) -> HRESULT;
    pub fn GetMetadataFromDDSFile(
        file: *const u16,
        dds_flags: DDS_FLAGS,
        metadata: *mut TexMetadata,
    ) -> HRESULT;
    pub fn GetMetadataFromTGAFile(
        file: *const u16,
        tga_flags: TGA_FLAGS,
        metadata: *mut TexMetadata,
    ) -> HRESULT;
    pub fn GetMetadataFromHDRFile(file: *const u16, metadata: *mut TexMetadata) -> HRESULT;
    pub fn GetMetadataFromEXRFile(file: *const u16, metadata: *mut TexMetadata) -> HRESULT;
    pub fn OverrideFormat(image: DXPtr, format: DXGI_FORMAT) -> HRESULT;
}

#[cfg(not(feature = "disable-wic"))]
extern "C" {
    pub fn GetMetadataFromWICFile(
        file: *const u16,
        wic_flags: WIC_FLAGS,
        metadata: *mut TexMetadata,
    ) -> HRESULT;

    pub fn LoadFromWICFile(
        file: *const u16,
        wic_flags: WIC_FLAGS,
        metadata: *mut TexMetadata,
        out: *mut DXImage,
    ) -> HRESULT;

    pub fn SaveToWICFile(
        image: DXPtr,
        array_index: usize,
        wic_flags: WIC_FLAGS,
        container: *const GUID,
        file: *const u16,
        format: *const GUID,
    ) -> HRESULT;
}

#[test]
fn test_dxtex_sizes() {
    #[repr(C)]
    #[derive(Default)]
    struct DirectXTexFfi {
        TexMetadata: usize,
        TEX_DIMENSION: usize,
        DDS_FLAGS: usize,
        TGA_FLAGS: usize,
        DXGI_FORMAT: usize,
        TEX_FILTER_FLAGS: usize,
    }

    extern "C" {
        fn GetFfiSizes(dst_len: usize, dst: *mut DirectXTexFfi) -> usize;
    }

    let mut sizes = DirectXTexFfi::default();

    assert_eq!(0, unsafe {
        GetFfiSizes(std::mem::size_of::<DirectXTexFfi>(), &mut sizes)
    });
    assert_eq!(std::mem::size_of::<TEX_DIMENSION>(), sizes.TEX_DIMENSION);
    assert_eq!(std::mem::size_of::<DXGI_FORMAT>(), sizes.DXGI_FORMAT);
    assert_eq!(std::mem::size_of::<DDS_FLAGS>(), sizes.DDS_FLAGS);
    assert_eq!(std::mem::size_of::<TGA_FLAGS>(), sizes.TGA_FLAGS);
    assert_eq!(
        std::mem::size_of::<TEX_FILTER_FLAGS>(),
        sizes.TEX_FILTER_FLAGS
    );
    assert_eq!(std::mem::size_of::<TexMetadata>(), sizes.TexMetadata);
}
