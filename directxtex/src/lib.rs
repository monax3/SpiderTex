#![allow(unsafe_code)]

use std::{borrow::Cow, path::Path};
use std::mem::MaybeUninit;

use windows_compat::{Result, HSTRING, errors::E_INVALIDARG};
use dxgi_format::{DXGI_FORMAT, DxgiFormatExt};
// use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM;

use directxtex_sys::DXPtr;
#[allow(clippy::wildcard_imports)]
use directxtex_sys::*;
pub use directxtex_sys::{TexMetadata, TEX_DIMENSION, TEX_FILTER_FLAGS};

#[cfg(feature = "windows-imaging")]
pub use windows_imaging::Container;

#[must_use]
#[inline]
pub fn is_compressed(format: DXGI_FORMAT) -> bool {
    unsafe { IsCompressed(format) > 0 }
}

#[must_use]
#[inline]
pub fn is_srgb(format: DXGI_FORMAT) -> bool {
    unsafe { IsSRGB(format) > 0 }
}

#[repr(transparent)]
pub struct DXTImage(DXPtr);

impl Drop for DXTImage {
    fn drop(&mut self) {
        let _ignore = unsafe { V2_FreeImage(self.0) };
    }
}

impl DXTImage {
    #[inline]
    pub fn load(file_name: impl AsRef<Path>) -> Result<Self> { load(file_name) }

    #[inline]
    pub fn new(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        array_size: usize,
        mipmaps: u8,
        data: &[u8],
    ) -> Result<Self> {
        if height > 1 {
            Self::new_2d(format, width, height, array_size, mipmaps, data)
        } else {
            Self::new_1d(format, width, array_size, mipmaps, data)
        }
    }

    #[inline]
    pub fn new_1d(
        format: DXGI_FORMAT,
        size: usize,
        array_size: usize,
        mipmaps: u8,
        data: &[u8],
    ) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        unsafe {
            V2_New1D(
                format,
                size,
                array_size,
                mipmaps,
                data.as_ptr(),
                data.len(),
                out.as_mut_ptr(),
            )
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    #[inline]
    pub fn new_2d(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        array_size: usize,
        mipmaps: u8,
        data: &[u8],
    ) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        unsafe {
            V2_New2D(
                format,
                width,
                height,
                array_size,
                mipmaps,
                data.as_ptr(),
                data.len(),
                out.as_mut_ptr(),
            )
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    // Helper for extracting intermediaries when chaining actions
    #[inline]
    pub fn inspect(self, func: impl FnOnce(&Self) -> Result<()>) -> Result<Self> {
        func(&self)?;
        Ok(self)
    }

    // Helper for chaining actions conditionally
    #[inline]
    pub fn map_if(self, condition: bool, func: impl FnOnce(&Self) -> Result<Self>) -> Result<Self> {
        if condition {
            func(&self)
        } else {
            Ok(self)
        }
    }

    #[inline]
    pub fn generate_mipmaps(&self, mipmaps: u8) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        #[cfg(not(feature = "disable-wic"))]
        let flags = TEX_FILTER_FLAGS::default();
        #[cfg(feature = "disable-wic")]
        let flags = TEX_FILTER_FLAGS::TEX_FILTER_FORCE_NON_WIC;

        unsafe {
            GenerateMipmaps(self.0, flags, usize::from(mipmaps), out.as_mut_ptr())
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    #[inline]
    pub fn override_format(&self, format: DXGI_FORMAT) -> Result<()> {
        unsafe {
            OverrideFormat(self.0, format).ok()?;
        }

        Ok(())
    }

    #[inline]
    pub fn compress(&self, to_format: DXGI_FORMAT) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        unsafe {
            V2_Compress(self.0, to_format, out.as_mut_ptr())
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    #[inline]
    pub fn resize(&self, width: usize, height: usize) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        unsafe {
            Resize(self.0, width, height, TEX_FILTER_FLAGS(0), out.as_mut_ptr())
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    #[inline]
    pub fn decompress(&self) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        unsafe {
            V2_Decompress(self.0, out.as_mut_ptr())
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    #[inline]
    pub fn convert(&self, to_format: DXGI_FORMAT, flags: TEX_FILTER_FLAGS) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        unsafe {
            V2_Convert(self.0, to_format, flags, out.as_mut_ptr())
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    #[inline]
    pub fn premultiply_alpha(&self, reverse: bool) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        unsafe {
            V2_PremultiplyAlpha(self.0, reverse, out.as_mut_ptr())
            .ok()
            .map(|_| DXTImage(out.assume_init()))
            }
    }

    #[inline]
    pub fn metadata(&self) -> Result<TexMetadata> {
        let mut metadata = TexMetadata::default();

        unsafe {
            V2_GetMetadata(self.0, &mut metadata).ok()?;
        }

        Ok(metadata)
    }

    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { BufferSize(self.0) }
    }

    // FIXME: rename to buffer
    pub fn pixels(&self) -> Result<Vec<u8>> {
        let len = unsafe { BufferSize(self.0) };

        let mut data = Vec::with_capacity(len);
        unsafe {
            Buffer(self.0, data.as_mut_ptr(), data.capacity()).ok()?;
            data.set_len(data.capacity());
        }

        Ok(data)
    }

    pub fn num_images(&self) -> Result<usize> {
        Ok(unsafe { ImageCount(self.0) })
    }

    pub fn image_len(&self, array_index: usize) -> Result<usize> {
        Ok(unsafe { ImageSize(self.0, array_index) })
    }

    pub fn image(&self, array_index: usize) -> Result<Vec<u8>> {
        let len = unsafe { ImageSize(self.0, array_index) };

        let mut data = Vec::with_capacity(len);

        unsafe {
            ImageData(self.0, array_index, data.as_mut_ptr(), data.capacity()).ok()?;
            data.set_len(data.capacity());
        }

        Ok(data)
    }

    pub fn save(
        &self,
        array_index: usize,
        file_name: impl AsRef<Path>,
    ) -> Result<()> {
        let file_name = file_name.as_ref();


        match file_name.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("dds") => self.save_dds(file_name.as_os_str()),
            Some(ext) if ext.eq_ignore_ascii_case("tga") => self.save_tga(array_index, file_name.as_os_str()),
            Some(ext) if ext.eq_ignore_ascii_case("hdr") => self.save_hdr(array_index, file_name.as_os_str()),
            Some(ext) if ext.eq_ignore_ascii_case("exr") => self.save_exr(array_index, file_name.as_os_str()),
            #[cfg(feature = "windows-imaging")]
            Some(ext) =>
                if let Some(container) = Container::from_extension(ext) { self.save_wic(array_index, container, file_name.as_os_str()) } else {
                    E_INVALIDARG.ok()
                }
            _ => E_INVALIDARG.ok()
        }
    }

    pub fn save_dds(&self, file_name: impl Into<HSTRING>) -> Result<()> {
        let file_name: HSTRING = file_name.into();

        unsafe {
            V2_SaveDDS(self.0, file_name.as_ptr(), 0).ok()?;
        }
        Ok(())
    }

    pub fn save_tga(&self, array_index: usize, file_name: impl Into<HSTRING>) -> Result<()> {
        let file_name: HSTRING = file_name.into();

        unsafe {
            V2_SaveTGA(self.0, array_index, file_name.as_ptr(), 0).ok()?;
        }
        Ok(())
    }

    pub fn save_hdr(&self, array_index: usize, file_name: impl Into<HSTRING>) -> Result<()> {
        let file_name: HSTRING = file_name.into();

        unsafe {
            V2_SaveHDR(self.0, array_index, file_name.as_ptr()).ok()?;
        }
        Ok(())
    }

    pub fn save_exr(&self, array_index: usize, file_name: impl Into<HSTRING>) -> Result<()> {
        let file_name: HSTRING = file_name.into();

        unsafe {
            V2_SaveEXR(self.0, array_index, file_name.as_ptr()).ok()?;
        }
        Ok(())
    }

    #[cfg(feature = "windows-imaging")]
    pub fn save_wic(
        &self,
        array_index: usize,
        container: Container,
        file_name: impl Into<HSTRING>,
    ) -> Result<()> {
        let file_name = file_name.into();

        unsafe {
            SaveToWICFile(
                self.0,
                array_index,
                0,
                container.as_guid(),
                file_name.as_ptr(),
                std::ptr::null(),
            )
            .ok()?;
        }
        Ok(())
    }

    #[cfg(disabled)]
    pub fn to_rgba<'image>(&'image self) -> Result<Cow<'image, Self>> {
        const RGBA: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;

        let mut metadata = self.metadata()?;
        let mut ret = Cow::Borrowed(self);

        if metadata.format.is_compressed() {
            ret = Cow::Owned(self.decompress()?);
            metadata = ret.metadata()?;
        }

        if !metadata.format.is_rgba() {
            ret = Cow::Owned(self.convert(RGBA, TEX_FILTER_FLAGS::default())?);
        }

        Ok(ret)
    }

    // FIXME: currently only works for uncompressed
    pub fn to_format<'image>(&'image self, format: DXGI_FORMAT) -> Result<Cow<'image, Self>> {
        let mut metadata = self.metadata()?;
        let mut ret = Cow::Borrowed(self);

        if metadata.format.is_compressed() {
            ret = Cow::Owned(self.decompress()?);
            metadata = ret.metadata()?;
        }

        if metadata.format != format {
            ret = Cow::Owned(self.convert(format, TEX_FILTER_FLAGS::default())?);
        }

        Ok(ret)
    }
}

impl Clone for DXTImage {
    fn clone(&self) -> Self {
        let mut handle = MaybeUninit::uninit();

        if let Err(error) = unsafe { Clone(self.0, handle.as_mut_ptr()) }.ok() {
            panic!("DXTImage::Clone failed: {error}");
        } else {
            Self(unsafe { handle.assume_init() })
        }
    }
}

#[inline]
#[must_use]
pub fn expected_size(
    format: DXGI_FORMAT,
    width: usize,
    height: usize,
    depth: usize,
    mipmaps: u8,
) -> usize {
    unsafe { ExpectedSize(format, width, height, depth, mipmaps) }
}

#[inline]
#[must_use]
pub fn expected_size_array(
    format: DXGI_FORMAT,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
) -> usize {
    unsafe { ExpectedSizeArray(format, width, height, array_size, mipmaps) }
}

pub fn compress_texture(
    format: DXGI_FORMAT,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<Vec<u8>> {
    let uncompressed = DXTImage::new_2d(format, width, height, array_size, 1, data)?;
    let temp = if mipmaps > 1 {
        uncompressed.generate_mipmaps(mipmaps)?
    } else {
        uncompressed
    };

    let compressed = temp.compress(format)?;

    compressed.pixels()
}

pub fn decompress_texture(
    format: DXGI_FORMAT,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<Vec<u8>> {
    let compressed = DXTImage::new_2d(format, width, height, array_size, mipmaps, data)?;
    let decompressed = compressed.decompress()?;

    decompressed.pixels()
}

pub fn load(file_name: impl AsRef<Path>) -> Result<DXTImage> {
    let file_name = file_name.as_ref();

    match file_name.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("dds") => load_dds(file_name.as_os_str()),
        Some(ext) if ext.eq_ignore_ascii_case("tga") => load_tga(file_name.as_os_str()),
        Some(ext) if ext.eq_ignore_ascii_case("hdr") => load_hdr(file_name.as_os_str()),
        Some(ext) if ext.eq_ignore_ascii_case("exr") => load_exr(file_name.as_os_str()),
        #[cfg(feature = "windows-imaging")]
        Some(_) => load_wic(file_name.as_os_str()),
        _ => Err(E_INVALIDARG.into())
    }
}

pub fn load_dds(file_name: impl Into<HSTRING>) -> Result<DXTImage> {
    let file_name: HSTRING = file_name.into();
    let mut handle = MaybeUninit::uninit();

    unsafe {
        LoadFromDDSFile(
            file_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            handle.as_mut_ptr(),
        )
        .ok()
        .map(|_| DXTImage(handle.assume_init()))
    }
}

pub fn load_tga(file_name: impl Into<HSTRING>) -> Result<DXTImage> {
    let file_name: HSTRING = file_name.into();
    let mut handle = MaybeUninit::uninit();

    unsafe {
        LoadFromTGAFile(
            file_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            handle.as_mut_ptr(),
        )
        .ok()
        .map(|_| DXTImage(handle.assume_init()))
    }
}

pub fn load_hdr(file_name: impl Into<HSTRING>) -> Result<DXTImage> {
    let file_name: HSTRING = file_name.into();
    let mut handle = MaybeUninit::uninit();

    unsafe {
        LoadFromHDRFile(
            file_name.as_ptr(),
            std::ptr::null_mut(),
            handle.as_mut_ptr(),
        )
        .ok()
        .map(|_| DXTImage(handle.assume_init()))
    }
}

pub fn load_exr(file_name: impl Into<HSTRING>) -> Result<DXTImage> {
    let file_name: HSTRING = file_name.into();
    let mut handle = MaybeUninit::uninit();

    unsafe {
        LoadFromEXRFile(
            file_name.as_ptr(),
            std::ptr::null_mut(),
            handle.as_mut_ptr(),
        )
        .ok()
        .map(|_| DXTImage(handle.assume_init()))
    }
}

pub fn load_wic(file_name: impl Into<HSTRING>) -> Result<DXTImage> {
    let file_name: HSTRING = file_name.into();
    let mut handle = MaybeUninit::uninit();

    // FIXME initialize_com()?;

    unsafe {
        LoadFromWICFile(
            file_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            handle.as_mut_ptr(),
        )
        .ok()
        .map(|_| DXTImage(handle.assume_init()))
    }
}

pub fn metadata(file_name: impl AsRef<Path>) -> Result<TexMetadata> {
    let file_name = file_name.as_ref();

    match file_name.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("dds") => metadata_from_dds(file_name.as_os_str()),
        Some(ext) if ext.eq_ignore_ascii_case("tga") => metadata_from_tga(file_name.as_os_str()),
        Some(ext) if ext.eq_ignore_ascii_case("hdr") => metadata_from_hdr(file_name.as_os_str()),
        Some(ext) if ext.eq_ignore_ascii_case("exr") => metadata_from_exr(file_name.as_os_str()),
        #[cfg(feature = "windows-imaging")]
        Some(_) => metadata_from_wic(file_name.as_os_str()),
        _ => Err(E_INVALIDARG.into())
    }
}

pub fn metadata_from_dds(file_name: impl Into<HSTRING>) -> Result<TexMetadata> {
    let file_name: HSTRING = file_name.into();
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromDDSFile(file_name.as_ptr(), 0, &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_tga(file_name: impl Into<HSTRING>) -> Result<TexMetadata> {
    let file_name: HSTRING = file_name.into();
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromTGAFile(file_name.as_ptr(), 0, &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_hdr(file_name: impl Into<HSTRING>) -> Result<TexMetadata> {
    let file_name: HSTRING = file_name.into();
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromHDRFile(file_name.as_ptr(), &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_exr(file_name: impl Into<HSTRING>) -> Result<TexMetadata> {
    let file_name: HSTRING = file_name.into();
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromEXRFile(file_name.as_ptr(), &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_wic(file_name: impl Into<HSTRING>) -> Result<TexMetadata> {
    // FIXME initialize_com()?;
    let file_name: HSTRING = file_name.into();

    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromWICFile(file_name.as_ptr(), 0, &mut metadata) }.ok()?;
    Ok(metadata)
}
