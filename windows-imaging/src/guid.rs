use windows::core::GUID;
use windows::Win32::Graphics::Imaging::{
    GUID_ContainerFormatPng, GUID_WICPixelFormat24bppBGR, GUID_WICPixelFormat32bppRGBA,
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum PixelFormat {
    RGBA32bpp,
    BGRA24bpp,
}

impl PixelFormat {
    pub fn bpp(self) -> u32 {
        match self {
            PixelFormat::RGBA32bpp => 4,
            PixelFormat::BGRA24bpp => 3,
        }
    }

    pub fn as_guid(self) -> &'static GUID {
        self.into()
    }

    pub fn from_guid(guid: &GUID) -> Option<Self> {
        match guid {
            guid if guid == &GUID_WICPixelFormat24bppBGR => Some(Self::BGRA24bpp),
            guid if guid == &GUID_WICPixelFormat32bppRGBA => Some(Self::RGBA32bpp),
            _ => None,
        }
    }
}

impl From<PixelFormat> for &'static GUID {
    fn from(format: PixelFormat) -> &'static GUID {
        match format {
            PixelFormat::RGBA32bpp => &GUID_WICPixelFormat32bppRGBA,
            PixelFormat::BGRA24bpp => &GUID_WICPixelFormat24bppBGR,
        }
    }
}

pub enum Container {
    Png,
}

impl Container {
    pub fn extension(self) -> &'static str {
        match self {
            Container::Png => "png",
        }
    }

    pub fn as_guid(self) -> &'static GUID {
        self.into()
    }

    pub fn from_guid(guid: &GUID) -> Option<Self> {
        match guid {
            guid if guid == &GUID_ContainerFormatPng => Some(Self::Png),
            _ => None,
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            ext if ext.eq_ignore_ascii_case("png") => Some(Self::Png),
            _ => None,
        }
    }
}

impl From<Container> for &'static GUID {
    fn from(container: Container) -> &'static GUID {
        match container {
            Container::Png => &GUID_ContainerFormatPng,
        }
    }
}
