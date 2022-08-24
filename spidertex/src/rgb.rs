//! FIXME: Assumes stride = width * bpp
#![allow(unsafe_code)]

use std::ffi::OsStr;
use std::mem::MaybeUninit;

use windows::core::{Interface, GUID, HSTRING};
use windows::Win32::Graphics::Imaging::D2D::IWICImagingFactory2;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory2,
    GUID_ContainerFormatPng,
    GUID_WICPixelFormat24bppBGR,
    GUID_WICPixelFormat32bppRGBA,
    IWICBitmap,
    IWICBitmapSource,
    WICBitmapDitherTypeNone,
    WICBitmapEncoderNoCache,
    WICBitmapPaletteTypeMedianCut,
    WICRect,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::SystemServices::GENERIC_WRITE;

use crate::dxtex::DXImage;
use crate::prelude::*;
use crate::util::initialize_com;

pub const PIXEL_FORMAT_BGR: &GUID = &GUID_WICPixelFormat24bppBGR;
pub const PIXEL_FORMAT_RGBA: &GUID = &GUID_WICPixelFormat32bppRGBA;
pub const CONTAINER_PNG: &GUID = &GUID_ContainerFormatPng;

pub unsafe fn initialize_wic() -> Result<IWICImagingFactory2> {
    initialize_com()?;

    let wic: IWICImagingFactory2 =
        CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER)?;

    Ok(wic)
}

pub struct WIC(IWICImagingFactory2);

impl WIC {
    pub fn new() -> Result<Self> {
        initialize_com()?;

        Ok(
            unsafe { CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER) }
                .map(Self)?,
        )
    }

    pub fn bitmap_from_directxtex(&self, image: &DXImage, array_index: usize) -> Result<WICSource> {
        const RGBA_BPP: u32 = 4;

        let image = image.to_rgba()?;
        let metadata = image.metadata()?;
        let buf = image.image(array_index)?;

        let bitmap = unsafe {
            self.0.CreateBitmapFromMemory(
                metadata.width as u32,
                metadata.height as u32,
                &GUID_WICPixelFormat32bppRGBA,
                metadata.width as u32 * RGBA_BPP,
                &buf,
            )
        }?;

        Ok(WICSource {
            wic:   self.0.clone(),
            inner: bitmap.cast()?,
        })
    }
}

pub struct WICSource {
    wic:   IWICImagingFactory2,
    inner: IWICBitmapSource,
}

impl WICSource {
    pub fn pixel_format(&self) -> Result<GUID> {
        let bitmap = self
            .inner
            .cast::<IWICBitmap>()
            .map_err(|_| Error::message("Pixel format called on a non-IWICBitmap source"))?;

        Ok(unsafe { bitmap.GetPixelFormat() }?)
    }

    pub fn to_pixel_format(&self, from_format: &GUID, to_format: &GUID) -> Result<Self> {
        let converter = unsafe { self.wic.CreateFormatConverter() }?;

        if !unsafe { converter.CanConvert(from_format, to_format) }?.as_bool() {
            return error_message("Can't convert pixel formats");
        }

        unsafe {
            converter.Initialize(
                &self.inner,
                to_format,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )
        }?;

        Ok(Self {
            wic:   self.wic.clone(),
            inner: converter.cast()?,
        })
    }

    pub fn rect(&self) -> Result<WICRect> {
        let mut width = 0;
        let mut height = 0;

        unsafe { self.inner.GetSize(&mut width, &mut height) }?;

        Ok(WICRect {
            X:      0,
            Y:      0,
            Width:  width.try_into().unwrap(),
            Height: height.try_into().unwrap(),
        })
    }

    pub fn save(&self, file: impl AsRef<OsStr>, container: &GUID) -> Result<()> {
        let file_name = HSTRING::from(file.as_ref());

        let stream = unsafe {
            let stream = self.wic.CreateStream()?;
            stream.InitializeFromFilename(&file_name, GENERIC_WRITE)?;
            stream
        };

        let encoder = unsafe {
            let encoder = self.wic.CreateEncoder(container, std::ptr::null())?;
            encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
            encoder
        };

        let frame = unsafe {
            let mut frame = MaybeUninit::uninit();
            encoder.CreateNewFrame(frame.as_mut_ptr(), std::ptr::null_mut())?;
            let frame = frame.assume_init().unwrap();
            frame.Initialize(None)?;
            frame
        };

        unsafe {
            frame.WriteSource(&self.inner, &self.rect()?)?;
            frame.Commit()?;
            encoder.Commit()?;
        }
        Ok(())
    }
}

#[test]
fn test_rgb() {
    const FROM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/rgb.png");
    const TO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/rgb_wic_test.png");

    crate::util::log_for_tests(true);

    let dx = DXImage::load(FROM).expect("DXImage::load");
    let wic = WIC::new().expect("WIC::new");
    let bitmap = wic
        .bitmap_from_directxtex(&dx, 0)
        .expect("WIC::bitmap_from_directxtex");
    let from_pixel_format = bitmap.pixel_format().expect("WICSource::pixel_format");

    let rgb = bitmap
        .to_pixel_format(&from_pixel_format, PIXEL_FORMAT_BGR)
        .expect("WICSource::to_pixel_format");

    rgb.save(TO, CONTAINER_PNG).expect("WICBitmap::save");
}
