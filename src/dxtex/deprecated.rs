use windows::{Win32::Graphics::Dxgi::Common::DXGI_FORMAT, core::HRESULT};
use crate::Result;
use std::ptr::NonNull;
use crate::formats::DXGIFormat;

use super::TexMetadata;

extern "C" {
    fn CompressTexture(
        dst_format: DXGI_FORMAT,
        width: usize,
        height: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut *mut u8,
    ) -> isize;

    fn Decompress1D(
        format: DXGI_FORMAT,
        size: usize,
        array_size: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut *mut u8,
        dst_len: *mut usize,
        metadata: *mut TexMetadata,
    ) -> HRESULT;

    /// # Error
    /// `E_INVALIDARG`: The source data is the wrong length, dst_len will have
    /// the correct size if non-null
    fn DecompressTexture2(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        array_size: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut *mut u8,
        dst_len: *mut usize,
        metadata: *mut TexMetadata,
    ) -> HRESULT;

    fn DecompressTexture(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        array_size: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut *mut u8,
    ) -> isize;

    fn ExpectedSize(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        depth: usize,
        mipmaps: u8,
    ) -> usize;

    fn ExpectedSizeCube(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        depth: usize,
        mipmaps: u8,
    ) -> usize;

    fn ExpectedSizeArray(
        format: DXGI_FORMAT,
        width: usize,
        height: usize,
        depth: usize,
        mipmaps: u8,
    ) -> usize;

    fn Free(ptr: *const u8);

}

#[inline]
#[must_use]
pub fn expected_size3(
    format: DXGIFormat,
    width: u32,
    height: u32,
    depth: u32,
    mipmaps: u8,
) -> (usize, usize, usize) {
    unsafe {
        (
            ExpectedSize(
                format.into(),
                width as usize,
                height as usize,
                depth as usize,
                mipmaps,
            ),
            ExpectedSizeArray(
                format.into(),
                width as usize,
                height as usize,
                depth as usize,
                mipmaps,
            ),
            ExpectedSizeCube(
                format.into(),
                width as usize,
                height as usize,
                depth as usize,
                mipmaps,
            ),
        )
    }
}

#[cfg(disabled)]
pub fn compress_texture(
    format: DXGIFormat,
    width: usize,
    height: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<DXBuf> {
    let mut ptr = std::ptr::null_mut();
    let len = unsafe {
        CompressTexture(
            format.into(),
            width,
            height,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut ptr,
        )
    };
    if len <= 0 {
        Err(len)
    } else {
        unsafe { DXBuf::new(ptr, len as usize) }.ok_or(-69)
    }
}

#[cfg(disabled)]
pub fn decompress_texture(
    format: DXGIFormat,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<DXBuf, isize> {
    let mut ptr = std::ptr::null_mut();
    let len = unsafe {
        DecompressTexture(
            format.into(),
            width,
            height,
            array_size,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut ptr,
        )
    };
    if len <= 0 {
        Err(len)
    } else {
        unsafe { DXBuf::new(ptr, len as usize) }.ok_or(-69)
    }
}

#[cfg(disabled)]
pub fn decompress_texture2(
    format: DXGIFormat,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<(DXBuf, TexMetadata)> {
    let mut dst = std::ptr::null_mut();
    let mut dst_len: usize = 0;
    let mut metadata = TexMetadata::default();

    unsafe {
        DecompressTexture2(
            format.into(),
            width,
            height,
            array_size,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut dst,
            &mut dst_len,
            &mut metadata,
        )
    }
    .ok()?;

    Ok((
        unsafe { DXBuf::new(dst, dst_len) }.expect("Null pointer returned after success"),
        metadata,
    ))
}

pub fn decompress1d(
    format: DXGIFormat,
    size: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<(DXBuf, TexMetadata)> {
    let mut dst = std::ptr::null_mut();
    let mut dst_len: usize = 0;
    let mut metadata = TexMetadata::default();

    unsafe {
        Decompress1D(
            format.into(),
            size,
            array_size,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut dst,
            &mut dst_len,
            &mut metadata,
        )
    }
    .ok()?;

    Ok((
        unsafe { DXBuf::new(dst, dst_len) }.expect("Null pointer returned after success"),
        metadata,
    ))
}

#[cfg(disabled)]
#[tracing::instrument(skip(data))]
pub fn v2_savetga2d(
    file: &camino::Utf8Path,
    format: DXGIFormat,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<()> {
    let mut image: Option<ImageHandle> = None;

    warn!("Creating");

    let file_utf16: Vec<u16> = file
        .as_str()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        V2_New2D(
            format.into(),
            width,
            height,
            array_size,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut image,
        )
    }
    .ok()?;
    let image = image.expect("IE");

    warn!("Saving as TGA with format {format:?}");
    unsafe { V2_SaveTGA(image.0, 0, file_utf16.as_ptr(), 0) }.ok()?;

    Ok(())
}

#[cfg(disabled)]
#[tracing::instrument(skip(data))]
pub fn v2_decompress1d(
    format: DXGIFormat,
    size: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<(Vec<u8>, TexMetadata)> {
    let mut compressed: Option<ImageHandle> = None;

    warn!("Creating");

    unsafe {
        V2_New1D(
            format.into(),
            size,
            array_size,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut compressed,
        )
    }
    .ok()?;
    let compressed = compressed.expect("IE");

    let mut uncompressed: Option<ImageHandle> = None;
    warn!("Decompressing");
    unsafe { V2_Decompress(compressed.0, &mut uncompressed) }.ok()?;

    let uncompressed = uncompressed.expect("IE");
    warn!("Get image size");
    let image_len = unsafe { V2_GetImageSize(uncompressed.0) };

    let mut image_data = Vec::with_capacity(image_len);
    warn!("Get image");
    unsafe {
        V2_GetImage(
            uncompressed.0,
            image_data.as_mut_ptr(),
            image_data.capacity(),
        )
    }
    .ok()?;
    unsafe { image_data.set_len(image_data.capacity()) };

    let mut metadata = TexMetadata::default();
    warn!("Get metadata");
    unsafe { V2_GetMetadata(uncompressed.0, &mut metadata) }.ok()?;

    warn!("Returning");
    Ok((image_data, metadata))
}

#[cfg(disabled)]
#[tracing::instrument(skip(data))]
pub fn v2_convert2d(
    from_format: DXGIFormat,
    to_format: DXGIFormat,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<(Vec<u8>, TexMetadata)> {
    let mut before: Option<ImageHandle> = None;

    warn!("Creating");

    let expected = unsafe { ExpectedSize(from_format.into(), width, height, 1, mipmaps) };

    warn!("Expected size {expected} and we have {}", data.len());

    unsafe {
        V2_New2D(
            from_format.into(),
            width,
            height,
            array_size,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut before,
        )
    }
    .ok()?;
    let before = before.expect("IE");

    let mut after: Option<ImageHandle> = None;
    warn!("Converting");
    unsafe { V2_Convert(before.0, to_format.into(), 0, &mut after) }.ok()?;

    let after = after.expect("IE");
    warn!("Get image size");
    let image_len = unsafe { V2_GetImageSize(after.0) };

    let mut image_data = Vec::with_capacity(image_len);
    warn!("Get image");
    unsafe { V2_GetImage(after.0, image_data.as_mut_ptr(), image_data.capacity()) }.ok()?;
    unsafe { image_data.set_len(image_data.capacity()) };

    let mut metadata = TexMetadata::default();
    warn!("Get metadata");
    unsafe { V2_GetMetadata(after.0, &mut metadata) }.ok()?;

    warn!("Returning");
    Ok((image_data, metadata))
}

#[cfg(disabled)]
#[tracing::instrument(skip(data))]
pub fn v2_decompress2d(
    format: DXGIFormat,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<(Vec<u8>, TexMetadata)> {
    let mut compressed: Option<ImageHandle> = None;

    warn!("Creating");

    unsafe {
        V2_New2D(
            format.into(),
            width,
            height,
            array_size,
            mipmaps,
            data.as_ptr(),
            data.len(),
            &mut compressed,
        )
    }
    .ok()?;
    let compressed = compressed.expect("IE");

    let mut uncompressed: Option<ImageHandle> = None;
    warn!("Decompressing");
    unsafe { V2_Decompress(compressed.0, &mut uncompressed) }.ok()?;

    let uncompressed = uncompressed.expect("IE");
    warn!("Get image size");
    let image_len = unsafe { V2_GetImageSize(uncompressed.0) };

    let mut image_data = Vec::with_capacity(image_len);
    warn!("Get image");
    unsafe {
        V2_GetImage(
            uncompressed.0,
            image_data.as_mut_ptr(),
            image_data.capacity(),
        )
    }
    .ok()?;
    unsafe { image_data.set_len(image_data.capacity()) };

    let mut metadata = TexMetadata::default();
    warn!("Get metadata");
    unsafe { V2_GetMetadata(uncompressed.0, &mut metadata) }.ok()?;

    warn!("Returning");
    Ok((image_data, metadata))
}

pub struct DXBuf(NonNull<u8>, usize);
impl DXBuf {
    #[inline]
    unsafe fn new(ptr: *mut u8, len: usize) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Self(ptr, len))
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    #[inline]
    pub fn len(&self) -> usize { self.1 }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr(), self.1) }
    }

    #[inline]
    pub fn as_slices(&self, array_size: usize) -> Vec<&[u8]> {
        debug_assert!(self.1 % array_size == 0);
        let array_len = self.1 / array_size;

        (0 .. array_size)
            .map(|i| unsafe {
                std::slice::from_raw_parts(self.0.as_ptr().add(i * array_len), array_len)
            })
            .collect()
    }
}

impl Drop for DXBuf {
    fn drop(&mut self) { unsafe { Free(self.0.as_ptr()) } }
}
