// FIXME: file types etc can be found with IWICBitmapDecoderInfo

use windows::core::GUID;
use windows::Win32::Graphics::Imaging::{
    GUID_ContainerFormatPng, GUID_ContainerFormatIco, GUID_WICPixelFormat24bppBGR, GUID_WICPixelFormat32bppRGBA,
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum PixelFormat {
    RGBA32bpp,
    BGR24bpp,
}

impl PixelFormat {
    pub fn bpp(self) -> u32 {
        match self {
            PixelFormat::RGBA32bpp => 4,
            PixelFormat::BGR24bpp => 3,
        }
    }

    pub fn as_guid(self) -> &'static GUID {
        self.into()
    }

    pub fn from_guid(guid: &GUID) -> Option<Self> {
        match guid {
            guid if guid == &GUID_WICPixelFormat24bppBGR => Some(Self::BGR24bpp),
            guid if guid == &GUID_WICPixelFormat32bppRGBA => Some(Self::RGBA32bpp),
            _ => None,
        }
    }
}

impl From<PixelFormat> for &'static GUID {
    fn from(format: PixelFormat) -> &'static GUID {
        match format {
            PixelFormat::RGBA32bpp => &GUID_WICPixelFormat32bppRGBA,
            PixelFormat::BGR24bpp => &GUID_WICPixelFormat24bppBGR,
        }
    }
}

pub enum Container {
    Png,
    Ico,
}

impl Container {
    pub fn extension(self) -> &'static str {
        match self {
            Container::Png => "png",
            Container::Ico => "ico",
        }
    }

    pub fn as_guid(self) -> &'static GUID {
        self.into()
    }

    // FIXME: This should be a try_from
    pub fn from_guid(guid: &GUID) -> Option<Self> {
        match guid {
            guid if guid == &GUID_ContainerFormatPng => Some(Self::Png),
            guid if guid == &GUID_ContainerFormatIco => Some(Self::Ico),
            _ => None,
        }
    }

    // FIXME: This chould? be a try_from
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            ext if ext.eq_ignore_ascii_case("ico") => Some(Self::Ico),
            _ => None,
        }
    }
}

impl From<Container> for &'static GUID {
    fn from(container: Container) -> &'static GUID {
        match container {
            Container::Png => &GUID_ContainerFormatPng,
            Container::Ico => &GUID_ContainerFormatIco,
        }
    }
}
