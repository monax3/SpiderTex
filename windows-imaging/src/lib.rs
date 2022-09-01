//! FIXME: Assumes stride = width * bpp
//! TODO: test what happens if the factory gets dropped
#![allow(unsafe_code)]

use std::mem::MaybeUninit;

pub use windows::core::{Error, Result};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{E_INVALIDARG, E_UNEXPECTED};
use windows::Win32::Graphics::Imaging::D2D::IWICImagingFactory2;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory,
    CLSID_WICImagingFactory2,
    WICBitmapDitherTypeNone,
    WICBitmapEncoderNoCache,
    WICBitmapPaletteTypeMedianCut,
    WICDecodeMetadataCacheOnDemand,
};
pub use windows::Win32::Graphics::Imaging::{
    IWICBitmap,
    IWICBitmapFrameDecode,
    IWICBitmapSource,
    IWICFormatConverter,
    IWICImagingFactory,
    WICRect,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::SystemServices::{GENERIC_READ, GENERIC_WRITE};

mod guid;
pub use guid::{Container, PixelFormat};
pub mod prelude {
    pub use super::{
        wic_factory,
        wic_factory2,
        Bitmap,
        BitmapSource,
        Container,
        IWICBitmap,
        IWICBitmapFrameDecode,
        IWICBitmapSource,
        IWICFormatConverter,
        IWICImagingFactory,
        PixelFormat,
        WICRect,
    };
}

pub(crate) fn invalid_arg() -> Error { Error::from(E_INVALIDARG) }
pub(crate) fn unexpected() -> Error { Error::from(E_UNEXPECTED) }

pub fn wic_factory() -> Result<IWICImagingFactory> {
    unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER) }
}

pub fn wic_factory2() -> Result<IWICImagingFactory2> {
    unsafe { CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER) }
}

pub fn load_bitmap_from_memory<'wic, WIC>(
    factory: WIC,
    format: PixelFormat,
    width: u32,
    height: u32,
    stride: u32,
    data: &[u8],
) -> Result<IWICBitmap>
where
    &'wic IWICImagingFactory: From<WIC>,
{
    let factory: &IWICImagingFactory = factory.into();

    unsafe {
        factory.CreateBitmapFromMemory(width as u32, height as u32, format.as_guid(), stride, data)
    }
}

pub fn load_container_from_memory<'wic, WIC>(
    factory: WIC,
    container: Container,
    data: &[u8],
) -> Result<IWICBitmapFrameDecode>
where
    &'wic IWICImagingFactory: From<WIC>,
{
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

// FIXME: work around the CreateDecoderFromFilename bug
pub fn load_container_from_file<F: WICFactory>(
    factory: &F,
    file_name: impl Into<HSTRING>,
) -> Result<IWICBitmapFrameDecode> {
    let factory = factory.as_wic_factory();
    let file_name: HSTRING = file_name.into();

    let decoder = unsafe {
        factory.CreateDecoderFromFilename(
            PCWSTR(file_name.as_ptr()),
            std::ptr::null(),
            GENERIC_READ,
            WICDecodeMetadataCacheOnDemand,
        )
    }?;

    unsafe { decoder.GetFrame(0) }
}

pub trait WICFactory {
    fn as_wic_factory(&self) -> &IWICImagingFactory;
}

impl<T> WICFactory for T
where for<'a> &'a IWICImagingFactory: From<&'a T>
{
    fn as_wic_factory(&self) -> &IWICImagingFactory { self.into() }
}

// pub fn bitmap_from_directxtex(&self, image: &DXImage, array_index: usize) ->
// Result<WICSource> {     const RGBA_BPP: u32 = 4;

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

impl Bitmap for IWICBitmap {
    fn as_wic_bitmap(&self) -> &IWICBitmap { self }
}

pub trait BitmapSource {
    fn as_wic_bitmap_source(&self) -> &IWICBitmapSource;

    fn rect(&self) -> Result<WICRect> {
        let mut width = 0;
        let mut height = 0;

        unsafe { self.as_wic_bitmap_source().GetSize(&mut width, &mut height) }?;

        // FIXME: better error code
        Ok(WICRect {
            X:      0,
            Y:      0,
            Width:  width.try_into().map_err(|_| unexpected())?,
            Height: height.try_into().map_err(|_| unexpected())?,
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

            eprintln!(
                "copy pixels with stride {stride}, buf_size {}",
                buffer.len()
            );

            self.as_wic_bitmap_source()
                .CopyPixels(&rect, stride, &mut buffer)
                .map(|_| {
                    buffer.set_len(size as usize);
                    buffer
                })
        }
    }

    fn pixel_format(&self) -> Result<PixelFormat> {
        unsafe { self.as_wic_bitmap_source().GetPixelFormat() }
            .map(|guid| PixelFormat::from_guid(&guid).unwrap_or_else(|| unimplemented!()))
    }

    fn convert_to_pixel_format<F: WICFactory>(
        &self,
        factory: &F,
        to: PixelFormat,
    ) -> Result<IWICFormatConverter> {
        let converter = unsafe { factory.as_wic_factory().CreateFormatConverter() }?;

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

    fn save<F: WICFactory>(
        &self,
        factory: &F,
        file_name: impl Into<HSTRING>,
        container: Container,
    ) -> Result<()> {
        let factory = factory.as_wic_factory();
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
impl<T> BitmapSource for T
where for<'a> &'a IWICBitmapSource: From<&'a T>
{
    fn as_wic_bitmap_source(&self) -> &IWICBitmapSource { self.into() }
}
