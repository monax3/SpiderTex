use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

use super::{dxgi, ColorPlanes, Dimensions, ImageFormat};
use crate::prelude::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Source {
    #[default]
    FromHeader,
    FromSize,
    FromFilename,
    UserOverride,
    MetaOverride,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq)]
pub struct TextureFormat {
    #[serde(with = "dxgi::serde")]
    pub dxgi_format: DXGI_FORMAT,
    // pub stex_format:           (u8, u8),
    pub standard:    Dimensions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highres:     Option<Dimensions>,
    #[serde(
        default = "default_array_size",
        skip_serializing_if = "is_default_array_size"
    )]
    pub array_size:  usize,
    #[serde(default, skip)]
    pub source:      Source,
}

impl Hash for TextureFormat {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher {
        self.id().hash(state);
    }
}

impl TextureFormat {
    #[inline]
    #[must_use]
    pub fn id(&self) -> FormatId { FormatId::from(self) }

    #[inline]
    #[must_use]
    pub const fn is_1d(&self) -> bool { self.standard.height == 1 }

    #[inline]
    #[must_use]
    pub const fn has_highres(&self) -> bool { self.highres.is_some() }

    #[inline]
    #[must_use]
    pub fn without_header<'buf>(&self, buffer: &'buf [u8]) -> &'buf [u8] {
        if buffer.len() == self.standard.data_size + TEXTURE_HEADER_SIZE {
            &buffer[TEXTURE_HEADER_SIZE ..]
        } else {
            buffer
        }
    }

    // FIXME: look into this
    #[inline]
    #[must_use]
    pub fn default_image_format(&self) -> ImageFormat {
        if self.dxgi_format.is_hdr() {
            ImageFormat::OpenExr
        } else if self.array_size >= 6 {
            ImageFormat::Dds
        } else {
            ImageFormat::Png
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_lut(&self) -> bool { self.is_1d_lut() || self.is_2d_lut() }

    #[inline]
    #[must_use]
    pub const fn is_1d_lut(&self) -> bool { self.standard.width == 16 && self.standard.height == 1 }

    #[inline]
    #[must_use]
    pub const fn is_2d_lut(&self) -> bool {
        self.array_size > 1 && self.standard.width == 32 && self.standard.height == 32
    }

    #[inline]
    #[must_use]
    pub const fn num_images(&self) -> usize { self.array_size }

    #[inline]
    #[must_use]
    pub const fn num_textures(&self) -> usize { if self.has_highres() { 2 } else { 1 } }

    #[inline]
    #[must_use]
    pub const fn sd_file_len(&self) -> usize { self.standard.data_size + TEXTURE_HEADER_SIZE }

    #[inline]
    #[must_use]
    #[allow(unused)]
    pub const fn sd_len(&self) -> usize { self.standard.data_size }

    #[inline]
    #[must_use]
    pub fn hd_len(&self) -> Option<usize> { self.highres.map(|dims| dims.data_size) }

    #[inline]
    pub fn to_header(&self) -> Result<texture_file::FormatHeader> {
        // TODO: create a fake header if necessary
        let header_str = registry::raw_header(self.id())
            .ok_or_else(|| Error::message("Raw header not found"))?;
        texture_file::FormatHeader::from_hexstring(&header_str)
    }

    #[inline]
    #[must_use]
    pub fn expected_standard_buffer_size(&self) -> usize {
        self.standard.width
            * self.standard.height
            * self.array_size
            * self.dxgi_format.planes().bpp_dxgi()
    }

    #[inline]
    #[must_use]
    pub fn expected_highres_buffer_size(&self) -> Option<usize> {
        self.highres.map(|highres| {
            highres.width * highres.height * self.array_size * self.dxgi_format.planes().bpp_dxgi()
        })
    }

    #[inline]
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 { self.standard.aspect_ratio() }

    #[inline]
    #[must_use]
    pub fn aspect_ratio_matches(&self, other: Dimensions) -> bool {
        self.standard.aspect_ratio_matches(other)
    }

    #[inline]
    #[must_use]
    pub const fn preferred_width(&self) -> usize { self.dimensions().width }

    #[inline]
    #[must_use]
    pub const fn preferred_height(&self) -> usize { self.dimensions().height }

    #[inline]
    #[must_use]
    pub fn is_correct_size(&self, other: Dimensions) -> (bool, bool) {
        let mut primary = true;

        for dimension in self.dimensions_iter() {
            if dimension == other {
                return (true, primary);
            }
            primary = false;
        }
        (false, false)
    }

    #[inline]
    #[must_use]
    pub fn best_texture<FILE>(&self, files: impl IntoIterator<Item = FILE>) -> Option<(Dimensions, FILE)> where FILE: AsRef<Path> + std::fmt::Display {
        let (best, other) = self.all_dimensions();
        let mut fallback = None;
        for file in files {
            event!(TRACE, name="best_texture", %file);
            if let Ok(size) = std::fs::metadata(file.as_ref()).map(|file| file.len() as usize) {
                event!(TRACE, size, ?best, ?other);
                if size == best.data_size || size == best.data_size + texture_file::TEXTURE_HEADER_SIZE {return Some((best, file)); }
                if let Some(other) = other {
                    if size == other.data_size || size == other.data_size  + texture_file::TEXTURE_HEADER_SIZE{
                        fallback = Some((other, file));
                    }
                }
            }
        }
        fallback
    }

    #[inline]
    #[must_use]
    pub fn dimensions_for_file(&self, file: impl AsRef<Path>) -> Option<Dimensions> {
        let size = std::fs::metadata(file.as_ref()).ok()?.len() as usize;

        self.dimensions_for_size(size)
    }

    #[inline]
    #[must_use]
    pub fn dimensions_for_size(&self, size: usize) -> Option<Dimensions> {
        self.dimensions_iter()
            .find(|dimensions| dimensions.is_for_file_size(size))
    }

    #[inline]
    #[must_use]
    pub const fn dimensions(&self) -> Dimensions {
        if let Some(highres) = self.highres {
            highres
        } else {
            self.standard
        }
    }

    #[inline]
    pub fn dimensions_iter(&self) -> impl Iterator<Item = Dimensions> + 'static {
        let (primary, secondary) = self.all_dimensions();

        Some(primary).into_iter().chain(secondary)
    }

    #[inline]
    #[must_use]
    pub const fn all_dimensions(&self) -> (Dimensions, Option<Dimensions>) {
        if let Some(highres) = self.highres {
            (highres, Some(self.standard))
        } else {
            (self.standard, None)
        }
    }

    #[inline]
    #[must_use]
    pub fn planes(&self) -> ColorPlanes { self.dxgi_format.planes() }
}

impl PartialEq for TextureFormat {
    fn eq(&self, other: &Self) -> bool { self.id() == other.id() }
}

impl From<texture_file::FormatHeader> for TextureFormat {
    fn from(header: texture_file::FormatHeader) -> Self { Self::from(&header) }
}

impl From<&texture_file::FormatHeader> for TextureFormat {
    fn from(header: &texture_file::FormatHeader) -> Self {
        let dxgi_format = DXGI_FORMAT(header.format.into());
        // let stex_format = (header.stex_format, header.planes);

        let standard = Dimensions {
            data_size: header.sd_len as usize,
            width:     header.sd_width as usize,
            height:    header.sd_height as usize,
            mipmaps:   header.sd_mipmaps,
        };

        let highres =
            (header.hd_len != header.sd_len && header.hd_len != 0).then_some(Dimensions {
                data_size: header.hd_len as usize,
                width:     header.hd_width as usize,
                height:    header.hd_height as usize,
                mipmaps:   header.hd_mipmaps,
            });

        Self {
            source: Source::FromHeader,
            dxgi_format,
            standard,
            highres,
            array_size: header.array_size as usize,
        }
    }
}

impl Display for TextureFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dxgi_format.display())?;

        for dimension in self.dimensions_iter() {
            write!(f, " {}", dimension)?;
        }

        if self.array_size > 1 {
            write!(f, " x{}", self.array_size)?;
        }
        Ok(())
    }
}

#[inline]
#[must_use]
pub const fn default_array_size() -> usize { 1 }

#[inline]
#[must_use]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub const fn is_default_array_size(array_size: &usize) -> bool {
    *array_size == default_array_size()
}
