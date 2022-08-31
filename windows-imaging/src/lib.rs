//! FIXME: Assumes stride = width * bpp
//! TODO: test what happens if the factory gets dropped
//! TODO: use enums instead of GUIDs
#![allow(unsafe_code)]

use std::mem::MaybeUninit;

pub use windows::core::Result;
use windows::core::HSTRING;
use windows::Win32::Graphics::Imaging::D2D::IWICImagingFactory2;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, CLSID_WICImagingFactory2, WICBitmapDitherTypeNone,
    WICBitmapEncoderNoCache, WICBitmapPaletteTypeMedianCut, WICDecodeMetadataCacheOnDemand, IWICBitmapFrameDecode,
};
pub use windows::Win32::Graphics::Imaging::{
    IWICBitmap, IWICBitmapSource, IWICFormatConverter, IWICImagingFactory, WICRect,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::SystemServices::GENERIC_WRITE;

mod guid;
pub use guid::{Container, PixelFormat};
pub mod prelude {
    pub use super::Bitmap;
    pub use super::BitmapSource;
    pub use super::{
        IWICBitmap, IWICBitmapSource, IWICFormatConverter, IWICImagingFactory, WICRect,
    };
    pub use super::Container as WICContainer;
    pub use super::PixelFormat as WICPixelFormat;
}

pub fn wic() -> Result<IWICImagingFactory> {
    unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER) }
}

pub fn wic2() -> Result<IWICImagingFactory2> {
    unsafe { CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER) }
}

pub fn bitmap_from_memory<'wic, WIC> (
    factory: WIC,
    format: PixelFormat,
    width: u32,
    height: u32,
    stride: u32,
    data: &[u8],
) -> Result<IWICBitmap> where &'wic IWICImagingFactory: From<WIC> {
    let factory: &IWICImagingFactory = factory.into();

    unsafe {
        factory.CreateBitmapFromMemory(width as u32, height as u32, format.as_guid(), stride, data)
    }
}
pub fn container_from_memory<'wic, WIC>(
    factory: WIC,
    container: Container,
    data: &[u8],
) -> Result<IWICBitmapFrameDecode> where &'wic IWICImagingFactory: From<WIC> {
    let factory: &IWICImagingFactory = factory.into();

    let stream = unsafe {
        let stream = factory.CreateStream()?;
        stream.InitializeFromMemory(data)?;
        stream
    };

    let decoder = unsafe {
        let decoder = factory.CreateDecoder(container.as_guid(), std::ptr::null())?;
        decoder.Initialize(&stream, WICDecodeMetadataCacheOnDemand)?;
        decoder
    };

    unsafe { decoder.GetFrame(0) }
}

// pub fn bitmap_from_directxtex(&self, image: &DXImage, array_index: usize) -> Result<WICSource> {
//     const RGBA_BPP: u32 = 4;

//     let image = image.to_rgba()?;
//     let metadata = image.metadata()?;
//     let buf = image.image(array_index)?;

//     let bitmap = unsafe {
//         self.0.CreateBitmapFromMemory(
//             metadata.width as u32,
//             metadata.height as u32,
//             &GUID_WICPixelFormat32bppRGBA,
//             metadata.width as u32 * RGBA_BPP,
//             &buf,
//         )
//     }?;

//     Ok(WICSource {
//         wic:   self.0.clone(),
//         inner: bitmap.cast()?,
//     })
// }

pub trait Bitmap {
    fn as_wic_bitmap(&self) -> &IWICBitmap;
}

// compiler bug workaround
// impl<T> Bitmap for T
// where
//     for<'a> &'a IWICBitmap: From<&'a T>,
// {
//     fn as_wic_bitmap(&self) -> &IWICBitmap {
//         self.into()
//     }
// }

impl Bitmap for IWICBitmap
{
    fn as_wic_bitmap(&self) -> &IWICBitmap {
        self
    }
}

pub trait BitmapSource {
    fn as_wic_bitmap_source(&self) -> &IWICBitmapSource;

    fn rect(&self) -> Result<WICRect> {
        let mut width = 0;
        let mut height = 0;

        unsafe { self.as_wic_bitmap_source().GetSize(&mut width, &mut height) }?;

        // FIXME: better error code
        Ok(WICRect {
            X: 0,
            Y: 0,
            Width: width.try_into().unwrap_or(i32::MAX),
            Height: width.try_into().unwrap_or(i32::MAX),
        })
    }

    fn pixels(&self) -> Result<Vec<u8>> {
        // FIXME: deal with stride, bpp
        // FIXME: stride can be done with IWICPixelFormatInfo
        // FIXME: don't load icon 1, make transparency work

        let rect = self.rect()?;
        let bpp = self.pixel_format()?.bpp() as u32;

        let stride = (rect.Width as u32) * bpp;
        let size = (rect.Height as u32) * stride;

        unsafe {
            // is there a better solutionhere than a blank buffer?
            // let mut buffer = Vec::with_capacity(size as usize);
            let mut buffer = vec![0; size as usize];

            eprintln!("copy pixels with stride {stride}, buf_size {}",buffer.len());

            self.as_wic_bitmap_source().CopyPixels(&rect, stride, &mut buffer).map(|_| {
                buffer.set_len(size as usize);
                buffer
            })
        }
    }

    fn pixel_format(&self) -> Result<PixelFormat> {
        unsafe { self.as_wic_bitmap_source().GetPixelFormat() }
            .map(|guid| PixelFormat::from_guid(&guid).unwrap_or_else(|| unimplemented!()))
    }

    fn convert_to_pixel_format<'wic, WIC>(
        &self,
        factory: WIC,
        to: PixelFormat,
    ) -> Result<IWICFormatConverter> where &'wic IWICImagingFactory: From<WIC> {
        let factory: &IWICImagingFactory = factory.into();
        let converter = unsafe { factory.CreateFormatConverter() }?;

        // FIXME: options?
        unsafe {
            converter.Initialize(
                self.as_wic_bitmap_source(),
                to.as_guid(),
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )
        }?;

        Ok(converter)
    }

    fn save<'wic, WIC>(
        &self,
        factory: WIC,
        file_name: impl Into<HSTRING>,
        container: Container,
    ) -> Result<()>  where &'wic IWICImagingFactory: From<WIC> {
        let factory: &IWICImagingFactory = factory.into();
        let file_name = file_name.into();

        let stream = unsafe {
            let stream = factory.CreateStream()?;
            stream.InitializeFromFilename(&file_name, GENERIC_WRITE)?;
            stream
        };

        let encoder = unsafe {
            let encoder = factory.CreateEncoder(container.as_guid(), std::ptr::null())?;
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

        let rect = self.rect()?;

        unsafe {
            frame.WriteSource(self.as_wic_bitmap_source(), &rect)?;
            frame.Commit()?;
            encoder.Commit()?;
        }
        Ok(())
    }
}

// compiler bug workaround
// impl<T> BitmapSource for T
// where
//     for<'a> &'a IWICBitmapSource: From<&'a T>,
// {
//     fn as_wic_bitmap_source(&self) -> &IWICBitmapSource {
//         self.into()
//     }
// }

impl BitmapSource for IWICBitmap {
    fn as_wic_bitmap_source(&self) -> &IWICBitmapSource {
        self.into()
    }
}

impl BitmapSource for IWICBitmapSource {
    fn as_wic_bitmap_source(&self) -> &IWICBitmapSource {
        self.into()
    }
}

impl BitmapSource for IWICFormatConverter {
    fn as_wic_bitmap_source(&self) -> &IWICBitmapSource {
        self.into()
    }
}

impl BitmapSource for IWICBitmapFrameDecode {
    fn as_wic_bitmap_source(&self) -> &IWICBitmapSource {
        self.into()
    }
}

#[cfg(disabled)]
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
