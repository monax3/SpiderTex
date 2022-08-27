use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

use camino::Utf8Path;
use image::{DynamicImage, ImageFormat};


use crate::files::FileStatus;
use crate::prelude::*;

pub trait Image: Sized {
    type Buffer: Sized;

    fn supports_format(image_format: ImageFormat) -> bool;
    fn quick_check(format: &TextureFormat, file: impl AsRef<Utf8Path>) -> Result<Warnings>;
    fn load(file: impl AsRef<Utf8Path>) -> Result<Self::Buffer>;

    fn as_supported_format(image_format: ImageFormat) -> Option<ImageFormat> {
        Self::supports_format(image_format).then_some(image_format)
    }
}

// TODO: this could impl deref
#[derive(Default, Debug, Clone)]
pub struct Warnings(Vec<Cow<'static, str>>);
impl Warnings {
    pub fn new() -> Self { Self::default() }

    pub fn push(&mut self, warning: impl Into<Cow<'static, str>>) { self.0.push(warning.into()); }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        let mut iter = self.0.iter();

        std::iter::from_fn(move || iter.next().map(AsRef::as_ref))
    }

    pub fn extend<W>(&mut self, iter: impl IntoIterator<Item = W>)
    where W: Into<Cow<'static, str>> {
        for warning in iter {
            self.push(warning);
        }
    }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

impl std::ops::DerefMut for Warnings {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl std::ops::Deref for Warnings {
    type Target = Vec<Cow<'static, str>>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl FromIterator<Warnings> for Warnings {
    fn from_iter<T: IntoIterator<Item = Warnings>>(iter: T) -> Self {
        let mut warnings = Warnings::new();
        for new in iter.into_iter() {
            warnings.extend(new);
        }
        warnings
    }
}

impl<A> FromIterator<A> for Warnings
where A: Into<Cow<'static, str>>
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Warnings(iter.into_iter().map(|w| w.into()).collect())
    }
}

impl IntoIterator for Warnings {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Cow<'static, str>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

pub struct DxImport;
impl Image for DxImport {
    type Buffer = directxtex::DXTImage;

    fn supports_format(image_format: ImageFormat) -> bool {
        // FIXME:
        true
        // directxtex::is_supported_format(image_format)
    }

    fn quick_check(format: &TextureFormat, file: impl AsRef<Utf8Path>) -> Result<Warnings> {
        #[cfg(feature = "debug-formats")]
        event!(TRACE, "Reading metadata for {}", file.as_ref());

        let metadata = directxtex::metadata(file.as_ref().as_str())?;
        let mut warnings = Warnings::new();

        let dimensions = Dimensions {
            data_size: 0,
            width:     metadata.width,
            height:    metadata.height,
            mipmaps:   metadata.mipLevels as u8,
        };

        let dxgi_format = metadata.format;
        // FIXME:
        // let expected_formats = format.planes.expected_formats();
        // if expected_formats.contains(&dxgi_format) {
        //     // event!(TRACE, file = %file.as_ref(), format =
        // %DXGIFormat::from(dxgi_format)); } else {
        //     status.warning(format!(
        //         "The file is in {} colors but the texture expects
        // {expected_formats:?}",         DXGIFormat::from(dxgi_format)
        //     ));
        // }

        if !format.aspect_ratio_matches(dimensions) {
            warnings.push("Image has the wrong aspect ratio and will be distorted if imported.");
        }

        let (correct_size, _is_lowres) = format.is_correct_size(dimensions);
        if !correct_size {
            warnings.push(format!(
                "Image is the wrong size ({}x{} instead of {}x{}) and will be resized.",
                dimensions.width,
                dimensions.height,
                format.preferred_width(),
                format.preferred_height()
            ));
        }

        Ok(warnings)
    }

    fn load(file: impl AsRef<Utf8Path>) -> Result<Self::Buffer> { todo!() }
}

pub struct ImageRs;

impl Image for ImageRs {
    type Buffer = DynamicImage;

    fn supports_format(image_format: ImageFormat) -> bool { true }

    // FIXME: image_rs will only read the dimensions
    // TODO: add header scanning for supported formats
    fn quick_check(format: &TextureFormat, file: impl AsRef<Utf8Path>) -> Result<Warnings> {
        let mut warnings = Warnings::new();

        let dimensions = Self::read_dimensions(file)?;

        if !format.aspect_ratio_matches(dimensions) {
            warnings.push("Image has the wrong aspect ratio and will be distorted if imported.");
        }
        let (needs_resize, _is_lowres) = format.is_correct_size(dimensions);
        if needs_resize {
            warnings.push(format!(
                "Image is the wrong size ({}x{} instead of {}x{}) and will be resized.",
                dimensions.width,
                dimensions.height,
                format.preferred_width(),
                format.preferred_height()
            ));
        }

        Ok(warnings)
    }

    fn load(file: impl AsRef<Utf8Path>) -> Result<DynamicImage> {
        let reader = image::io::Reader::open(file.as_ref())?.with_guessed_format()?;

        Ok(reader.decode()?)
    }
}

impl ImageRs {
    pub fn read_dimensions(file: impl AsRef<Utf8Path>) -> Result<Dimensions> {
        let reader = File::open(file.as_ref())?;
        let file_size = reader.metadata()?.len() as usize;

        let (width, height) = image::io::Reader::new(BufReader::new(reader))
            .with_guessed_format()?
            .into_dimensions()?;

        Ok(Dimensions {
            data_size: file_size,
            width:     width as usize,
            height:    height as usize,
            mipmaps:   1,
        })
    }
}
