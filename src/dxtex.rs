use std::ptr::NonNull;

use color_eyre::Result;

extern "C" {
    fn CompressTexture(
        dst_format: u32,
        width: usize,
        height: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut *mut u8,
    ) -> isize;

    fn DecompressTexture(
        format: u32,
        width: usize,
        height: usize,
        array_size: usize,
        mipmaps: u8,
        src: *const u8,
        src_len: usize,
        dst: *mut *mut u8,
    ) -> isize;

    fn ExpectedSize(
        format: u32,
        width: usize,
        height: usize,
        depth: usize,
        mipmaps: u8,
    ) -> usize;

    fn ExpectedSizeCube(
        format: u32,
        width: usize,
        height: usize,
        depth: usize,
        mipmaps: u8,
    ) -> usize;

    fn ExpectedSizeArray(
        format: u32,
        width: usize,
        height: usize,
        depth: usize,
        mipmaps: u8,
    ) -> usize;

    fn Free(ptr: *const u8);
}

pub fn expected_size(
    format: u32,
    width: u32,
    height: u32,
    depth: u32,
    mipmaps: u8,
) -> usize {
    unsafe {
        ExpectedSize(
            format,
            width as usize,
            height as usize,
            depth as usize,
            mipmaps,
        )
    }
}

pub fn expected_size3(
    format: u32,
    width: u32,
    height: u32,
    depth: u32,
    mipmaps: u8,
) -> (usize, usize, usize) {
    unsafe {
        (ExpectedSize(
            format,
            width as usize,
            height as usize,
            depth as usize,
            mipmaps,
        ),
        ExpectedSizeArray(
            format,
            width as usize,
            height as usize,
            depth as usize,
            mipmaps,
        ),
        ExpectedSizeCube(
            format,
            width as usize,
            height as usize,
            depth as usize,
            mipmaps,
        ))
    }
}

pub fn compress_texture(
    format: u32,
    width: usize,
    height: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<DXBuf, isize> {
    let mut ptr = std::ptr::null_mut();
    let len = unsafe {
        CompressTexture(
            format,
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

pub fn decompress_texture(
    format: u32,
    width: usize,
    height: usize,
    array_size: usize,
    mipmaps: u8,
    data: &[u8],
) -> Result<DXBuf, isize> {
    let mut ptr = std::ptr::null_mut();
    let len = unsafe {
        DecompressTexture(
            format,
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

        (0 .. array_size).map(|i|
            unsafe { std::slice::from_raw_parts(self.0.as_ptr().add(i * array_len), array_len) }
        ).collect()
    }
}

impl Drop for DXBuf {
    fn drop(&mut self) { unsafe { Free(self.0.as_ptr()) } }
}
