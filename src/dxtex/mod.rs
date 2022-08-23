#![allow(unsafe_code)] // no FFI without unsafe

use std::borrow::Cow;
use std::mem::MaybeUninit;

use camino::Utf8Path;
use image::ImageFormat;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_FORMAT_R8G8B8A8_UNORM};

use crate::util::{initialize_com, to_wstring};

const WIC_FORMATS: &[ImageFormat] = &[ImageFormat::Bmp, ImageFormat::Gif, ImageFormat::Png];
mod ffi;
#[allow(clippy::wildcard_imports)] use ffi::*;
pub use ffi::{TexMetadata, TEX_DIMENSION, TEX_FILTER_FLAGS};

use crate::prelude::*;

#[must_use]
#[inline]
pub fn is_compressed(format: DXGI_FORMAT) -> bool { unsafe { IsCompressed(format) > 0 } }

#[must_use]
#[inline]
pub fn is_srgb(format: DXGI_FORMAT) -> bool { unsafe { IsSRGB(format) > 0 } }

#[repr(transparent)]
pub struct DXImage(DXPtr);

impl Drop for DXImage {
    fn drop(&mut self) { let _ignore = unsafe { V2_FreeImage(self.0) }; }
}

impl DXImage {
    #[inline]
    pub fn load(file: impl AsRef<Utf8Path>) -> Result<Self> { load(file) }

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
    pub fn with_dimensions(
        format: DXGI_FORMAT,
        dimensions: Dimensions,
        array_size: usize,
        data: &[u8],
    ) -> Result<Self> {
        if dimensions.height > 1 {
            Self::new_2d(
                format,
                dimensions.width,
                dimensions.height,
                array_size,
                dimensions.mipmaps,
                data,
            )
        } else {
            Self::new_1d(
                format,
                dimensions.width,
                array_size,
                dimensions.mipmaps,
                data,
            )
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
        let mut handle = MaybeUninit::uninit();

        Ok(unsafe {
            V2_New1D(
                format,
                size,
                array_size,
                mipmaps,
                data.as_ptr(),
                data.len(),
                handle.as_mut_ptr(),
            )
            .ok()?;
            handle.assume_init()
        })
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
        let mut handle = MaybeUninit::uninit();

        Ok(unsafe {
            V2_New2D(
                format,
                width,
                height,
                array_size,
                mipmaps,
                data.as_ptr(),
                data.len(),
                handle.as_mut_ptr(),
            )
            .ok()?;
            handle.assume_init()
        })
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
        if condition { func(&self) } else { Ok(self) }
    }

    #[inline]
    pub fn generate_mipmaps(&self, mipmaps: u8) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        #[cfg(not(feature = "disable-wic"))]
        let flags = TEX_FILTER_FLAGS::default();
        #[cfg(feature = "disable-wic")]
        let flags = TEX_FILTER_FLAGS::TEX_FILTER_FORCE_NON_WIC;

        Ok(unsafe {
            GenerateMipmaps(self.0, flags, usize::from(mipmaps), out.as_mut_ptr()).ok()?;
            out.assume_init()
        })
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

        Ok(unsafe {
            V2_Compress(self.0, to_format, out.as_mut_ptr()).ok()?;
            out.assume_init()
        })
    }

    #[inline]
    pub fn resize(&self, width: usize, height: usize) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        Ok(unsafe {
            Resize(self.0, width, height, TEX_FILTER_FLAGS(0), out.as_mut_ptr()).ok()?;
            out.assume_init()
        })
    }

    #[inline]
    pub fn decompress(&self) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        Ok(unsafe {
            V2_Decompress(self.0, out.as_mut_ptr()).ok()?;
            out.assume_init()
        })
    }

    #[inline]
    pub fn convert(&self, to_format: DXGI_FORMAT, flags: TEX_FILTER_FLAGS) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        Ok(unsafe {
            V2_Convert(self.0, to_format, flags, out.as_mut_ptr()).ok()?;
            out.assume_init()
        })
    }

    #[inline]
    pub fn premultiply_alpha(&self, reverse: bool) -> Result<Self> {
        let mut out = MaybeUninit::uninit();

        Ok(unsafe {
            V2_PremultiplyAlpha(self.0, reverse, out.as_mut_ptr()).ok()?;
            out.assume_init()
        })
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
    pub fn len(&self) -> usize { unsafe { BufferSize(self.0) } }

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

    pub fn num_images(&self) -> Result<usize> { Ok(unsafe { ImageCount(self.0) }) }

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
        image_format: ImageFormat,
        file: impl AsRef<Utf8Path>,
    ) -> Result<()> {
        // FIXME: handle aray_index

        let file: &Utf8Path = file.as_ref();

        match image_format {
            ImageFormat::Dds => self.save_dds(file),
            ImageFormat::Tga => self.save_tga(array_index, file),
            ImageFormat::Hdr => self.save_hdr(array_index, file),
            ImageFormat::OpenExr => self.save_exr(array_index, file),
            #[cfg(not(feature = "disable-wic"))]
            image_format if is_wic_format(image_format) => {
                self.save_wic(array_index, image_format, file)
            }
            _ => error_message("DirectXTex tried to save an unsupported file format"),
        }
    }

    pub fn save_dds(&self, file: impl AsRef<Utf8Path>) -> Result<()> {
        let file_utf16: Vec<u16> = file
            .as_ref()
            .as_str()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            V2_SaveDDS(self.0, file_utf16.as_ptr(), 0).ok()?;
        }
        Ok(())
    }

    pub fn save_tga(&self, array_index: usize, file: impl AsRef<Utf8Path>) -> Result<()> {
        let file_utf16: Vec<u16> = file
            .as_ref()
            .as_str()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            V2_SaveTGA(self.0, array_index, file_utf16.as_ptr(), 0).ok()?;
        }
        Ok(())
    }

    pub fn save_hdr(&self, array_index: usize, file: impl AsRef<Utf8Path>) -> Result<()> {
        let file_utf16 = to_wstring(file);

        unsafe {
            V2_SaveHDR(self.0, array_index, file_utf16.as_ptr()).ok()?;
        }
        Ok(())
    }

    pub fn save_exr(&self, array_index: usize, file: impl AsRef<Utf8Path>) -> Result<()> {
        let file_utf16 = to_wstring(file);

        unsafe {
            V2_SaveEXR(self.0, array_index, file_utf16.as_ptr()).ok()?;
        }
        Ok(())
    }

    // TODO: Maybe pass ImageFormat
    #[cfg(not(feature = "disable-wic"))]
    pub fn save_wic(
        &self,
        array_index: usize,
        image_format: image::ImageFormat,
        file: impl AsRef<Utf8Path>,
    ) -> Result<()> {
        use windows::Win32::Graphics::Imaging::{
            GUID_ContainerFormatBmp,
            GUID_ContainerFormatGif,
            GUID_ContainerFormatPng,
        };

        let container = match image_format {
            ImageFormat::Bmp => &GUID_ContainerFormatBmp,
            ImageFormat::Gif => &GUID_ContainerFormatGif,
            ImageFormat::Png => &GUID_ContainerFormatPng,
            _ => return error_message("Extension not supported by WIC"),
        };

        let file_utf16 = to_wstring(file);

        initialize_com()?;
        unsafe {
            SaveToWICFile(
                self.0,
                array_index,
                0,
                container,
                file_utf16.as_ptr(),
                std::ptr::null(),
            )
            .ok()?;
        }
        Ok(())
    }

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

impl Clone for DXImage {
    fn clone(&self) -> Self {
        let mut handle = MaybeUninit::uninit();

        if let Err(error) = unsafe { Clone(self.0, handle.as_mut_ptr()) }.ok() {
            panic!("DXImage::Clone failed: {error}");
        } else {
            unsafe { handle.assume_init() }
        }
    }
}

#[inline]
#[must_use]
pub fn expected_size(format: DXGI_FORMAT, dimensions: Dimensions, depth: usize) -> usize {
    unsafe {
        ExpectedSize(
            format,
            dimensions.width,
            dimensions.height,
            depth,
            dimensions.mipmaps,
        )
    }
}

#[inline]
#[must_use]
pub fn expected_size_array(
    format: DXGI_FORMAT,
    dimensions: Dimensions,
    array_size: usize,
) -> usize {
    unsafe {
        ExpectedSizeArray(
            format,
            dimensions.width,
            dimensions.height,
            array_size,
            dimensions.mipmaps,
        )
    }
}

pub fn compress_texture(
    format: DXGI_FORMAT,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<Vec<u8>> {
    let uncompressed = DXImage::new_2d(format, width, height, array_size, 1, data)?;
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
    let compressed = DXImage::new_2d(format, width, height, array_size, mipmaps, data)?;
    let decompressed = compressed.decompress()?;

    decompressed.pixels()
}

#[must_use]
#[inline]
pub fn is_supported_format(image_format: ImageFormat) -> bool {
    match image_format {
        ImageFormat::Dds | ImageFormat::Tga | ImageFormat::Hdr | ImageFormat::OpenExr => true,
        _ => is_wic_format(image_format),
    }
}

#[inline]
#[must_use]
pub fn is_wic_format(image_format: ImageFormat) -> bool {
    #[cfg(feature = "disable-wic")]
    return bool;
    #[cfg(not(feature = "disable-wic"))]
    WIC_FORMATS.contains(&image_format)
}

pub fn load(file: impl AsRef<Utf8Path>) -> Result<DXImage> {
    let file: &Utf8Path = file.as_ref();

    let ext = file
        .extension()
        .ok_or_else(|| Error::message("File has no extension"))?;

    match ImageFormat::from_extension(ext) {
        Some(format) if format == ImageFormat::Dds => load_dds(file),
        Some(format) if format == ImageFormat::Tga => load_tga(file),
        Some(format) if format == ImageFormat::Hdr => load_hdr(file),
        Some(format) if format == ImageFormat::OpenExr => load_exr(file),
        #[cfg(not(feature = "disable-wic"))]
        Some(format) if is_wic_format(format) => load_wic(file),
        _ => error_message("DirectXTex tried to open an unsupported file format"),
    }
}

pub fn load_dds(file: impl AsRef<Utf8Path>) -> Result<DXImage> {
    let file = to_wstring(file);
    let mut handle = MaybeUninit::uninit();

    Ok(unsafe {
        LoadFromDDSFile(file.as_ptr(), 0, std::ptr::null_mut(), handle.as_mut_ptr()).ok()?;
        handle.assume_init()
    })
}

pub fn load_tga(file: impl AsRef<Utf8Path>) -> Result<DXImage> {
    let file = to_wstring(file);
    let mut handle = MaybeUninit::uninit();

    Ok(unsafe {
        LoadFromTGAFile(file.as_ptr(), 0, std::ptr::null_mut(), handle.as_mut_ptr()).ok()?;
        handle.assume_init()
    })
}

pub fn load_hdr(file: impl AsRef<Utf8Path>) -> Result<DXImage> {
    let file = to_wstring(file);
    let mut handle = MaybeUninit::uninit();

    Ok(unsafe {
        LoadFromHDRFile(file.as_ptr(), std::ptr::null_mut(), handle.as_mut_ptr()).ok()?;
        handle.assume_init()
    })
}

pub fn load_exr(file: impl AsRef<Utf8Path>) -> Result<DXImage> {
    let file = to_wstring(file);
    let mut handle = MaybeUninit::uninit();

    Ok(unsafe {
        LoadFromEXRFile(file.as_ptr(), std::ptr::null_mut(), handle.as_mut_ptr()).ok()?;
        handle.assume_init()
    })
}

#[cfg(not(feature = "disable-wic"))]
pub fn load_wic(file: impl AsRef<Utf8Path>) -> Result<DXImage> {
    let file = to_wstring(file);
    let mut handle = MaybeUninit::uninit();

    initialize_com()?;

    Ok(unsafe {
        LoadFromWICFile(file.as_ptr(), 0, std::ptr::null_mut(), handle.as_mut_ptr()).ok()?;
        handle.assume_init()
    })
}

pub fn metadata(file: impl AsRef<Utf8Path>) -> Result<TexMetadata> {
    let file: &Utf8Path = file.as_ref();

    let ext = file
        .extension()
        .ok_or_else(|| Error::message("File has no extension"))?;

    match ImageFormat::from_extension(ext) {
        Some(format) if format == ImageFormat::Dds => metadata_from_dds(file),
        Some(format) if format == ImageFormat::Tga => metadata_from_tga(file),
        Some(format) if format == ImageFormat::Hdr => metadata_from_hdr(file),
        Some(format) if format == ImageFormat::OpenExr => metadata_from_exr(file),
        #[cfg(not(feature = "disable-wic"))]
        Some(format) if is_wic_format(format) => metadata_from_wic(file),
        format => error_message(format!(
            "DirectXTex tried to open an unsupported file format: {format:?} {file}"
        )),
    }
}

pub fn metadata_from_dds(file: impl AsRef<Utf8Path>) -> Result<TexMetadata> {
    let file = to_wstring(file);
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromDDSFile(file.as_ptr(), 0, &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_tga(file: impl AsRef<Utf8Path>) -> Result<TexMetadata> {
    let file = to_wstring(file);
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromTGAFile(file.as_ptr(), 0, &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_hdr(file: impl AsRef<Utf8Path>) -> Result<TexMetadata> {
    let file = to_wstring(file);
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromHDRFile(file.as_ptr(), &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_exr(file: impl AsRef<Utf8Path>) -> Result<TexMetadata> {
    let file = to_wstring(file);
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromEXRFile(file.as_ptr(), &mut metadata) }.ok()?;
    Ok(metadata)
}

pub fn metadata_from_wic(file: impl AsRef<Utf8Path>) -> Result<TexMetadata> {
    initialize_com()?;

    let file = to_wstring(file);
    let mut metadata = TexMetadata::default();
    unsafe { GetMetadataFromWICFile(file.as_ptr(), 0, &mut metadata) }.ok()?;
    Ok(metadata)
}
