#![allow(clippy::cognitive_complexity, clippy::too_many_lines, clippy::match_same_arms)]

use windows::core::{Error, GUID};
use windows::Win32::Graphics::Imaging::{
    GUID_ContainerFormatBmp,
    GUID_ContainerFormatDds,
    GUID_ContainerFormatGif,
    GUID_ContainerFormatHeif,
    GUID_ContainerFormatIco,
    GUID_ContainerFormatJpeg,
    GUID_ContainerFormatPng,
    GUID_ContainerFormatTiff,
    GUID_ContainerFormatWebp,
    GUID_ContainerFormatWmp,
    GUID_WICPixelFormat112bpp6ChannelsAlpha,
    GUID_WICPixelFormat112bpp7Channels,
    GUID_WICPixelFormat128bpp7ChannelsAlpha,
    GUID_WICPixelFormat128bpp8Channels,
    GUID_WICPixelFormat128bppPRGBAFloat,
    GUID_WICPixelFormat128bppRGBAFixedPoint,
    GUID_WICPixelFormat128bppRGBAFloat,
    GUID_WICPixelFormat128bppRGBFixedPoint,
    GUID_WICPixelFormat128bppRGBFloat,
    GUID_WICPixelFormat144bpp8ChannelsAlpha,
    GUID_WICPixelFormat16bppBGR555,
    GUID_WICPixelFormat16bppBGR565,
    GUID_WICPixelFormat16bppBGRA5551,
    GUID_WICPixelFormat16bppCbCr,
    GUID_WICPixelFormat16bppCbQuantizedDctCoefficients,
    GUID_WICPixelFormat16bppCrQuantizedDctCoefficients,
    GUID_WICPixelFormat16bppGray,
    GUID_WICPixelFormat16bppGrayFixedPoint,
    GUID_WICPixelFormat16bppGrayHalf,
    GUID_WICPixelFormat16bppYQuantizedDctCoefficients,
    GUID_WICPixelFormat1bppIndexed,
    GUID_WICPixelFormat24bpp3Channels,
    GUID_WICPixelFormat24bppBGR,
    GUID_WICPixelFormat24bppRGB,
    GUID_WICPixelFormat2bppGray,
    GUID_WICPixelFormat2bppIndexed,
    GUID_WICPixelFormat32bpp3ChannelsAlpha,
    GUID_WICPixelFormat32bpp4Channels,
    GUID_WICPixelFormat32bppBGR,
    GUID_WICPixelFormat32bppBGR101010,
    GUID_WICPixelFormat32bppBGRA,
    GUID_WICPixelFormat32bppCMYK,
    GUID_WICPixelFormat32bppGrayFixedPoint,
    GUID_WICPixelFormat32bppGrayFloat,
    GUID_WICPixelFormat32bppPBGRA,
    GUID_WICPixelFormat32bppPRGBA,
    GUID_WICPixelFormat32bppR10G10B10A2,
    GUID_WICPixelFormat32bppR10G10B10A2HDR10,
    GUID_WICPixelFormat32bppRGB,
    GUID_WICPixelFormat32bppRGBA,
    GUID_WICPixelFormat32bppRGBA1010102,
    GUID_WICPixelFormat32bppRGBA1010102XR,
    GUID_WICPixelFormat32bppRGBE,
    GUID_WICPixelFormat40bpp4ChannelsAlpha,
    GUID_WICPixelFormat40bpp5Channels,
    GUID_WICPixelFormat40bppCMYKAlpha,
    GUID_WICPixelFormat48bpp3Channels,
    GUID_WICPixelFormat48bpp5ChannelsAlpha,
    GUID_WICPixelFormat48bpp6Channels,
    GUID_WICPixelFormat48bppBGR,
    GUID_WICPixelFormat48bppBGRFixedPoint,
    GUID_WICPixelFormat48bppRGB,
    GUID_WICPixelFormat48bppRGBFixedPoint,
    GUID_WICPixelFormat48bppRGBHalf,
    GUID_WICPixelFormat4bppGray,
    GUID_WICPixelFormat4bppIndexed,
    GUID_WICPixelFormat56bpp6ChannelsAlpha,
    GUID_WICPixelFormat56bpp7Channels,
    GUID_WICPixelFormat64bpp3ChannelsAlpha,
    GUID_WICPixelFormat64bpp4Channels,
    GUID_WICPixelFormat64bpp7ChannelsAlpha,
    GUID_WICPixelFormat64bpp8Channels,
    GUID_WICPixelFormat64bppBGRA,
    GUID_WICPixelFormat64bppBGRAFixedPoint,
    GUID_WICPixelFormat64bppCMYK,
    GUID_WICPixelFormat64bppPBGRA,
    GUID_WICPixelFormat64bppPRGBA,
    GUID_WICPixelFormat64bppPRGBAHalf,
    GUID_WICPixelFormat64bppRGB,
    GUID_WICPixelFormat64bppRGBA,
    GUID_WICPixelFormat64bppRGBAFixedPoint,
    GUID_WICPixelFormat64bppRGBAHalf,
    GUID_WICPixelFormat64bppRGBFixedPoint,
    GUID_WICPixelFormat64bppRGBHalf,
    GUID_WICPixelFormat72bpp8ChannelsAlpha,
    GUID_WICPixelFormat80bpp4ChannelsAlpha,
    GUID_WICPixelFormat80bpp5Channels,
    GUID_WICPixelFormat80bppCMYKAlpha,
    GUID_WICPixelFormat8bppAlpha,
    GUID_WICPixelFormat8bppCb,
    GUID_WICPixelFormat8bppCr,
    GUID_WICPixelFormat8bppGray,
    GUID_WICPixelFormat8bppIndexed,
    GUID_WICPixelFormat8bppY,
    GUID_WICPixelFormat96bpp5ChannelsAlpha,
    GUID_WICPixelFormat96bpp6Channels,
    GUID_WICPixelFormat96bppRGBFixedPoint,
    GUID_WICPixelFormat96bppRGBFloat,
    GUID_WICPixelFormatBlackWhite,
    GUID_WICPixelFormatDontCare,
};

use crate::invalid_arg;

#[cfg_attr(test, derive(strum::EnumIter, strum::AsRefStr))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    PF112bpp6ChannelsAlpha,
    PF112bpp7Channels,
    PF128bpp7ChannelsAlpha,
    PF128bpp8Channels,
    PF128bppPRGBAFloat,
    PF128bppRGBAFixedPoint,
    PF128bppRGBAFloat,
    PF128bppRGBFixedPoint,
    PF128bppRGBFloat,
    PF144bpp8ChannelsAlpha,
    PF16bppBGR555,
    PF16bppBGR565,
    PF16bppBGRA5551,
    PF16bppCbCr,
    PF16bppCbQuantizedDctCoefficients,
    PF16bppCrQuantizedDctCoefficients,
    PF16bppGray,
    PF16bppGrayFixedPoint,
    PF16bppGrayHalf,
    PF16bppYQuantizedDctCoefficients,
    PF1bppIndexed,
    PF24bpp3Channels,
    PF24bppBGR,
    PF24bppRGB,
    PF2bppGray,
    PF2bppIndexed,
    PF32bpp3ChannelsAlpha,
    PF32bpp4Channels,
    PF32bppBGR,
    PF32bppBGR101010,
    PF32bppBGRA,
    PF32bppCMYK,
    PF32bppGrayFixedPoint,
    PF32bppGrayFloat,
    PF32bppPBGRA,
    PF32bppPRGBA,
    PF32bppR10G10B10A2,
    PF32bppR10G10B10A2HDR10,
    PF32bppRGB,
    PF32bppRGBA,
    PF32bppRGBA1010102,
    PF32bppRGBA1010102XR,
    PF32bppRGBE,
    PF40bpp4ChannelsAlpha,
    PF40bpp5Channels,
    PF40bppCMYKAlpha,
    PF48bpp3Channels,
    PF48bpp5ChannelsAlpha,
    PF48bpp6Channels,
    PF48bppBGR,
    PF48bppBGRFixedPoint,
    PF48bppRGB,
    PF48bppRGBFixedPoint,
    PF48bppRGBHalf,
    PF4bppGray,
    PF4bppIndexed,
    PF56bpp6ChannelsAlpha,
    PF56bpp7Channels,
    PF64bpp3ChannelsAlpha,
    PF64bpp4Channels,
    PF64bpp7ChannelsAlpha,
    PF64bpp8Channels,
    PF64bppBGRA,
    PF64bppBGRAFixedPoint,
    PF64bppCMYK,
    PF64bppPBGRA,
    PF64bppPRGBA,
    PF64bppPRGBAHalf,
    PF64bppRGB,
    PF64bppRGBA,
    PF64bppRGBAFixedPoint,
    PF64bppRGBAHalf,
    PF64bppRGBFixedPoint,
    PF64bppRGBHalf,
    PF72bpp8ChannelsAlpha,
    PF80bpp4ChannelsAlpha,
    PF80bpp5Channels,
    PF80bppCMYKAlpha,
    PF8bppAlpha,
    PF8bppCb,
    PF8bppCr,
    PF8bppGray,
    PF8bppIndexed,
    PF8bppY,
    PF96bpp5ChannelsAlpha,
    PF96bpp6Channels,
    PF96bppRGBFixedPoint,
    PF96bppRGBFloat,
    PFBlackWhite,
    PFDontCare,
}

impl PixelFormat {
    #[inline]
    #[must_use]
    pub const fn bpp(self) -> u32 {
        match self {
        Self::PF112bpp6ChannelsAlpha => 112,
        Self::PF112bpp7Channels => 112,
        Self::PF128bpp7ChannelsAlpha => 128,
        Self::PF128bpp8Channels => 128,
        Self::PF128bppPRGBAFloat => 128,
        Self::PF128bppRGBAFixedPoint => 128,
        Self::PF128bppRGBAFloat => 128,
        Self::PF128bppRGBFixedPoint => 128,
        Self::PF128bppRGBFloat => 128,
        Self::PF144bpp8ChannelsAlpha => 144,
        Self::PF16bppBGR555 => 16,
        Self::PF16bppBGR565 => 16,
        Self::PF16bppBGRA5551 => 16,
        Self::PF16bppCbCr => 16,
        Self::PF16bppCbQuantizedDctCoefficients => unimplemented!(),
        Self::PF16bppCrQuantizedDctCoefficients => unimplemented!(),
        Self::PF16bppGray => 16,
        Self::PF16bppGrayFixedPoint => 16,
        Self::PF16bppGrayHalf => 16,
        Self::PF16bppYQuantizedDctCoefficients => unimplemented!(),
        Self::PF1bppIndexed => 1,
        Self::PF24bpp3Channels => 24,
        Self::PF24bppBGR => 24,
        Self::PF24bppRGB => 24,
        Self::PF2bppGray => 2,
        Self::PF2bppIndexed => 2,
        Self::PF32bpp3ChannelsAlpha => 32,
        Self::PF32bpp4Channels => 32,
        Self::PF32bppBGR => 32,
        Self::PF32bppBGR101010 => 32,
        Self::PF32bppBGRA => 32,
        Self::PF32bppCMYK => 32,
        Self::PF32bppGrayFixedPoint => 32,
        Self::PF32bppGrayFloat => 32,
        Self::PF32bppPBGRA => 32,
        Self::PF32bppPRGBA => 32,
        Self::PF32bppR10G10B10A2 => unimplemented!(),
        Self::PF32bppR10G10B10A2HDR10 => 32,
        Self::PF32bppRGB => 32,
        Self::PF32bppRGBA => 32,
        Self::PF32bppRGBA1010102 => 32,
        Self::PF32bppRGBA1010102XR => 32,
        Self::PF32bppRGBE => 32,
        Self::PF40bpp4ChannelsAlpha => 40,
        Self::PF40bpp5Channels => 40,
        Self::PF40bppCMYKAlpha => 40,
        Self::PF48bpp3Channels => 48,
        Self::PF48bpp5ChannelsAlpha => 48,
        Self::PF48bpp6Channels => 48,
        Self::PF48bppBGR => 48,
        Self::PF48bppBGRFixedPoint => 48,
        Self::PF48bppRGB => 48,
        Self::PF48bppRGBFixedPoint => 48,
        Self::PF48bppRGBHalf => 48,
        Self::PF4bppGray => 4,
        Self::PF4bppIndexed => 4,
        Self::PF56bpp6ChannelsAlpha => 56,
        Self::PF56bpp7Channels => 56,
        Self::PF64bpp3ChannelsAlpha => 64,
        Self::PF64bpp4Channels => 64,
        Self::PF64bpp7ChannelsAlpha => 64,
        Self::PF64bpp8Channels => 64,
        Self::PF64bppBGRA => 64,
        Self::PF64bppBGRAFixedPoint => 64,
        Self::PF64bppCMYK => 64,
        Self::PF64bppPBGRA => 64,
        Self::PF64bppPRGBA => 64,
        Self::PF64bppPRGBAHalf => 64,
        Self::PF64bppRGB => 64,
        Self::PF64bppRGBA => 64,
        Self::PF64bppRGBAFixedPoint => 64,
        Self::PF64bppRGBAHalf => 64,
        Self::PF64bppRGBFixedPoint => 64,
        Self::PF64bppRGBHalf => 64,
        Self::PF72bpp8ChannelsAlpha => 72,
        Self::PF80bpp4ChannelsAlpha => 80,
        Self::PF80bpp5Channels => 80,
        Self::PF80bppCMYKAlpha => 80,
        Self::PF8bppAlpha => 8,
        Self::PF8bppCb => 8,
        Self::PF8bppCr => 8,
        Self::PF8bppGray => 8,
        Self::PF8bppIndexed => 8,
        Self::PF8bppY => 8,
        Self::PF96bpp5ChannelsAlpha => 96,
        Self::PF96bpp6Channels => 96,
        Self::PF96bppRGBFixedPoint => 96,
        Self::PF96bppRGBFloat => 96,
        Self::PFBlackWhite => 1,
        Self::PFDontCare => unimplemented!(),
    }
        }

    #[inline]
    #[must_use]
    #[rustfmt::skip]
    pub const fn as_guid(self) -> &'static GUID {
        match self {
            Self::PF112bpp6ChannelsAlpha => &GUID_WICPixelFormat112bpp6ChannelsAlpha,
            Self::PF112bpp7Channels => &GUID_WICPixelFormat112bpp7Channels,
            Self::PF128bpp7ChannelsAlpha => &GUID_WICPixelFormat128bpp7ChannelsAlpha,
            Self::PF128bpp8Channels => &GUID_WICPixelFormat128bpp8Channels,
            Self::PF128bppPRGBAFloat => &GUID_WICPixelFormat128bppPRGBAFloat,
            Self::PF128bppRGBAFixedPoint => &GUID_WICPixelFormat128bppRGBAFixedPoint,
            Self::PF128bppRGBAFloat => &GUID_WICPixelFormat128bppRGBAFloat,
            Self::PF128bppRGBFixedPoint => &GUID_WICPixelFormat128bppRGBFixedPoint,
            Self::PF128bppRGBFloat => &GUID_WICPixelFormat128bppRGBFloat,
            Self::PF144bpp8ChannelsAlpha => &GUID_WICPixelFormat144bpp8ChannelsAlpha,
            Self::PF16bppBGR555 => &GUID_WICPixelFormat16bppBGR555,
            Self::PF16bppBGR565 => &GUID_WICPixelFormat16bppBGR565,
            Self::PF16bppBGRA5551 => &GUID_WICPixelFormat16bppBGRA5551,
            Self::PF16bppCbCr => &GUID_WICPixelFormat16bppCbCr,
            Self::PF16bppCbQuantizedDctCoefficients => &GUID_WICPixelFormat16bppCbQuantizedDctCoefficients,
            Self::PF16bppCrQuantizedDctCoefficients => &GUID_WICPixelFormat16bppCrQuantizedDctCoefficients,
            Self::PF16bppGray => &GUID_WICPixelFormat16bppGray,
            Self::PF16bppGrayFixedPoint => &GUID_WICPixelFormat16bppGrayFixedPoint,
            Self::PF16bppGrayHalf => &GUID_WICPixelFormat16bppGrayHalf,
            Self::PF16bppYQuantizedDctCoefficients => &GUID_WICPixelFormat16bppYQuantizedDctCoefficients,
            Self::PF1bppIndexed => &GUID_WICPixelFormat1bppIndexed,
            Self::PF24bpp3Channels => &GUID_WICPixelFormat24bpp3Channels,
            Self::PF24bppBGR => &GUID_WICPixelFormat24bppBGR,
            Self::PF24bppRGB => &GUID_WICPixelFormat24bppRGB,
            Self::PF2bppGray => &GUID_WICPixelFormat2bppGray,
            Self::PF2bppIndexed => &GUID_WICPixelFormat2bppIndexed,
            Self::PF32bpp3ChannelsAlpha => &GUID_WICPixelFormat32bpp3ChannelsAlpha,
            Self::PF32bpp4Channels => &GUID_WICPixelFormat32bpp4Channels,
            Self::PF32bppBGR => &GUID_WICPixelFormat32bppBGR,
            Self::PF32bppBGR101010 => &GUID_WICPixelFormat32bppBGR101010,
            Self::PF32bppBGRA => &GUID_WICPixelFormat32bppBGRA,
            Self::PF32bppCMYK => &GUID_WICPixelFormat32bppCMYK,
            Self::PF32bppGrayFixedPoint => &GUID_WICPixelFormat32bppGrayFixedPoint,
            Self::PF32bppGrayFloat => &GUID_WICPixelFormat32bppGrayFloat,
            Self::PF32bppPBGRA => &GUID_WICPixelFormat32bppPBGRA,
            Self::PF32bppPRGBA => &GUID_WICPixelFormat32bppPRGBA,
            Self::PF32bppR10G10B10A2 => &GUID_WICPixelFormat32bppR10G10B10A2,
            Self::PF32bppR10G10B10A2HDR10 => &GUID_WICPixelFormat32bppR10G10B10A2HDR10,
            Self::PF32bppRGB => &GUID_WICPixelFormat32bppRGB,
            Self::PF32bppRGBA => &GUID_WICPixelFormat32bppRGBA,
            Self::PF32bppRGBA1010102 => &GUID_WICPixelFormat32bppRGBA1010102,
            Self::PF32bppRGBA1010102XR => &GUID_WICPixelFormat32bppRGBA1010102XR,
            Self::PF32bppRGBE => &GUID_WICPixelFormat32bppRGBE,
            Self::PF40bpp4ChannelsAlpha => &GUID_WICPixelFormat40bpp4ChannelsAlpha,
            Self::PF40bpp5Channels => &GUID_WICPixelFormat40bpp5Channels,
            Self::PF40bppCMYKAlpha => &GUID_WICPixelFormat40bppCMYKAlpha,
            Self::PF48bpp3Channels => &GUID_WICPixelFormat48bpp3Channels,
            Self::PF48bpp5ChannelsAlpha => &GUID_WICPixelFormat48bpp5ChannelsAlpha,
            Self::PF48bpp6Channels => &GUID_WICPixelFormat48bpp6Channels,
            Self::PF48bppBGR => &GUID_WICPixelFormat48bppBGR,
            Self::PF48bppBGRFixedPoint => &GUID_WICPixelFormat48bppBGRFixedPoint,
            Self::PF48bppRGB => &GUID_WICPixelFormat48bppRGB,
            Self::PF48bppRGBFixedPoint => &GUID_WICPixelFormat48bppRGBFixedPoint,
            Self::PF48bppRGBHalf => &GUID_WICPixelFormat48bppRGBHalf,
            Self::PF4bppGray => &GUID_WICPixelFormat4bppGray,
            Self::PF4bppIndexed => &GUID_WICPixelFormat4bppIndexed,
            Self::PF56bpp6ChannelsAlpha => &GUID_WICPixelFormat56bpp6ChannelsAlpha,
            Self::PF56bpp7Channels => &GUID_WICPixelFormat56bpp7Channels,
            Self::PF64bpp3ChannelsAlpha => &GUID_WICPixelFormat64bpp3ChannelsAlpha,
            Self::PF64bpp4Channels => &GUID_WICPixelFormat64bpp4Channels,
            Self::PF64bpp7ChannelsAlpha => &GUID_WICPixelFormat64bpp7ChannelsAlpha,
            Self::PF64bpp8Channels => &GUID_WICPixelFormat64bpp8Channels,
            Self::PF64bppBGRA => &GUID_WICPixelFormat64bppBGRA,
            Self::PF64bppBGRAFixedPoint => &GUID_WICPixelFormat64bppBGRAFixedPoint,
            Self::PF64bppCMYK => &GUID_WICPixelFormat64bppCMYK,
            Self::PF64bppPBGRA => &GUID_WICPixelFormat64bppPBGRA,
            Self::PF64bppPRGBA => &GUID_WICPixelFormat64bppPRGBA,
            Self::PF64bppPRGBAHalf => &GUID_WICPixelFormat64bppPRGBAHalf,
            Self::PF64bppRGB => &GUID_WICPixelFormat64bppRGB,
            Self::PF64bppRGBA => &GUID_WICPixelFormat64bppRGBA,
            Self::PF64bppRGBAFixedPoint => &GUID_WICPixelFormat64bppRGBAFixedPoint,
            Self::PF64bppRGBAHalf => &GUID_WICPixelFormat64bppRGBAHalf,
            Self::PF64bppRGBFixedPoint => &GUID_WICPixelFormat64bppRGBFixedPoint,
            Self::PF64bppRGBHalf => &GUID_WICPixelFormat64bppRGBHalf,
            Self::PF72bpp8ChannelsAlpha => &GUID_WICPixelFormat72bpp8ChannelsAlpha,
            Self::PF80bpp4ChannelsAlpha => &GUID_WICPixelFormat80bpp4ChannelsAlpha,
            Self::PF80bpp5Channels => &GUID_WICPixelFormat80bpp5Channels,
            Self::PF80bppCMYKAlpha => &GUID_WICPixelFormat80bppCMYKAlpha,
            Self::PF8bppAlpha => &GUID_WICPixelFormat8bppAlpha,
            Self::PF8bppCb => &GUID_WICPixelFormat8bppCb,
            Self::PF8bppCr => &GUID_WICPixelFormat8bppCr,
            Self::PF8bppGray => &GUID_WICPixelFormat8bppGray,
            Self::PF8bppIndexed => &GUID_WICPixelFormat8bppIndexed,
            Self::PF8bppY => &GUID_WICPixelFormat8bppY,
            Self::PF96bpp5ChannelsAlpha => &GUID_WICPixelFormat96bpp5ChannelsAlpha,
            Self::PF96bpp6Channels => &GUID_WICPixelFormat96bpp6Channels,
            Self::PF96bppRGBFixedPoint => &GUID_WICPixelFormat96bppRGBFixedPoint,
            Self::PF96bppRGBFloat => &GUID_WICPixelFormat96bppRGBFloat,
            Self::PFBlackWhite => &GUID_WICPixelFormatBlackWhite,
            Self::PFDontCare => &GUID_WICPixelFormatDontCare,
        }
    }

    #[inline]
    #[must_use]
    #[rustfmt::skip]
    pub fn from_guid(guid: &GUID) -> Option<Self> {
        match guid {
            guid if guid == &GUID_WICPixelFormat112bpp6ChannelsAlpha => Some(Self::PF112bpp6ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat112bpp7Channels => Some(Self::PF112bpp7Channels),
            guid if guid == &GUID_WICPixelFormat128bpp7ChannelsAlpha => Some(Self::PF128bpp7ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat128bpp8Channels => Some(Self::PF128bpp8Channels),
            guid if guid == &GUID_WICPixelFormat128bppPRGBAFloat => Some(Self::PF128bppPRGBAFloat),
            guid if guid == &GUID_WICPixelFormat128bppRGBAFixedPoint => Some(Self::PF128bppRGBAFixedPoint),
            guid if guid == &GUID_WICPixelFormat128bppRGBAFloat => Some(Self::PF128bppRGBAFloat),
            guid if guid == &GUID_WICPixelFormat128bppRGBFixedPoint => Some(Self::PF128bppRGBFixedPoint),
            guid if guid == &GUID_WICPixelFormat128bppRGBFloat => Some(Self::PF128bppRGBFloat),
            guid if guid == &GUID_WICPixelFormat144bpp8ChannelsAlpha => Some(Self::PF144bpp8ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat16bppBGR555 => Some(Self::PF16bppBGR555),
            guid if guid == &GUID_WICPixelFormat16bppBGR565 => Some(Self::PF16bppBGR565),
            guid if guid == &GUID_WICPixelFormat16bppBGRA5551 => Some(Self::PF16bppBGRA5551),
            guid if guid == &GUID_WICPixelFormat16bppCbCr => Some(Self::PF16bppCbCr),
            guid if guid == &GUID_WICPixelFormat16bppCbQuantizedDctCoefficients => Some(Self::PF16bppCbQuantizedDctCoefficients),
            guid if guid == &GUID_WICPixelFormat16bppCrQuantizedDctCoefficients => Some(Self::PF16bppCrQuantizedDctCoefficients),
            guid if guid == &GUID_WICPixelFormat16bppGray => Some(Self::PF16bppGray),
            guid if guid == &GUID_WICPixelFormat16bppGrayFixedPoint => Some(Self::PF16bppGrayFixedPoint),
            guid if guid == &GUID_WICPixelFormat16bppGrayHalf => Some(Self::PF16bppGrayHalf),
            guid if guid == &GUID_WICPixelFormat16bppYQuantizedDctCoefficients => Some(Self::PF16bppYQuantizedDctCoefficients),
            guid if guid == &GUID_WICPixelFormat1bppIndexed => Some(Self::PF1bppIndexed),
            guid if guid == &GUID_WICPixelFormat24bpp3Channels => Some(Self::PF24bpp3Channels),
            guid if guid == &GUID_WICPixelFormat24bppBGR => Some(Self::PF24bppBGR),
            guid if guid == &GUID_WICPixelFormat24bppRGB => Some(Self::PF24bppRGB),
            guid if guid == &GUID_WICPixelFormat2bppGray => Some(Self::PF2bppGray),
            guid if guid == &GUID_WICPixelFormat2bppIndexed => Some(Self::PF2bppIndexed),
            guid if guid == &GUID_WICPixelFormat32bpp3ChannelsAlpha => Some(Self::PF32bpp3ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat32bpp4Channels => Some(Self::PF32bpp4Channels),
            guid if guid == &GUID_WICPixelFormat32bppBGR => Some(Self::PF32bppBGR),
            guid if guid == &GUID_WICPixelFormat32bppBGR101010 => Some(Self::PF32bppBGR101010),
            guid if guid == &GUID_WICPixelFormat32bppBGRA => Some(Self::PF32bppBGRA),
            guid if guid == &GUID_WICPixelFormat32bppCMYK => Some(Self::PF32bppCMYK),
            guid if guid == &GUID_WICPixelFormat32bppGrayFixedPoint => Some(Self::PF32bppGrayFixedPoint),
            guid if guid == &GUID_WICPixelFormat32bppGrayFloat => Some(Self::PF32bppGrayFloat),
            guid if guid == &GUID_WICPixelFormat32bppPBGRA => Some(Self::PF32bppPBGRA),
            guid if guid == &GUID_WICPixelFormat32bppPRGBA => Some(Self::PF32bppPRGBA),
            guid if guid == &GUID_WICPixelFormat32bppR10G10B10A2 => Some(Self::PF32bppR10G10B10A2),
            guid if guid == &GUID_WICPixelFormat32bppR10G10B10A2HDR10 => Some(Self::PF32bppR10G10B10A2HDR10),
            guid if guid == &GUID_WICPixelFormat32bppRGB => Some(Self::PF32bppRGB),
            guid if guid == &GUID_WICPixelFormat32bppRGBA => Some(Self::PF32bppRGBA),
            guid if guid == &GUID_WICPixelFormat32bppRGBA1010102 => Some(Self::PF32bppRGBA1010102),
            guid if guid == &GUID_WICPixelFormat32bppRGBA1010102XR => Some(Self::PF32bppRGBA1010102XR),
            guid if guid == &GUID_WICPixelFormat32bppRGBE => Some(Self::PF32bppRGBE),
            guid if guid == &GUID_WICPixelFormat40bpp4ChannelsAlpha => Some(Self::PF40bpp4ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat40bpp5Channels => Some(Self::PF40bpp5Channels),
            guid if guid == &GUID_WICPixelFormat40bppCMYKAlpha => Some(Self::PF40bppCMYKAlpha),
            guid if guid == &GUID_WICPixelFormat48bpp3Channels => Some(Self::PF48bpp3Channels),
            guid if guid == &GUID_WICPixelFormat48bpp5ChannelsAlpha => Some(Self::PF48bpp5ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat48bpp6Channels => Some(Self::PF48bpp6Channels),
            guid if guid == &GUID_WICPixelFormat48bppBGR => Some(Self::PF48bppBGR),
            guid if guid == &GUID_WICPixelFormat48bppBGRFixedPoint => Some(Self::PF48bppBGRFixedPoint),
            guid if guid == &GUID_WICPixelFormat48bppRGB => Some(Self::PF48bppRGB),
            guid if guid == &GUID_WICPixelFormat48bppRGBFixedPoint => Some(Self::PF48bppRGBFixedPoint),
            guid if guid == &GUID_WICPixelFormat48bppRGBHalf => Some(Self::PF48bppRGBHalf),
            guid if guid == &GUID_WICPixelFormat4bppGray => Some(Self::PF4bppGray),
            guid if guid == &GUID_WICPixelFormat4bppIndexed => Some(Self::PF4bppIndexed),
            guid if guid == &GUID_WICPixelFormat56bpp6ChannelsAlpha => Some(Self::PF56bpp6ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat56bpp7Channels => Some(Self::PF56bpp7Channels),
            guid if guid == &GUID_WICPixelFormat64bpp3ChannelsAlpha => Some(Self::PF64bpp3ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat64bpp4Channels => Some(Self::PF64bpp4Channels),
            guid if guid == &GUID_WICPixelFormat64bpp7ChannelsAlpha => Some(Self::PF64bpp7ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat64bpp8Channels => Some(Self::PF64bpp8Channels),
            guid if guid == &GUID_WICPixelFormat64bppBGRA => Some(Self::PF64bppBGRA),
            guid if guid == &GUID_WICPixelFormat64bppBGRAFixedPoint => Some(Self::PF64bppBGRAFixedPoint),
            guid if guid == &GUID_WICPixelFormat64bppCMYK => Some(Self::PF64bppCMYK),
            guid if guid == &GUID_WICPixelFormat64bppPBGRA => Some(Self::PF64bppPBGRA),
            guid if guid == &GUID_WICPixelFormat64bppPRGBA => Some(Self::PF64bppPRGBA),
            guid if guid == &GUID_WICPixelFormat64bppPRGBAHalf => Some(Self::PF64bppPRGBAHalf),
            guid if guid == &GUID_WICPixelFormat64bppRGB => Some(Self::PF64bppRGB),
            guid if guid == &GUID_WICPixelFormat64bppRGBA => Some(Self::PF64bppRGBA),
            guid if guid == &GUID_WICPixelFormat64bppRGBAFixedPoint => Some(Self::PF64bppRGBAFixedPoint),
            guid if guid == &GUID_WICPixelFormat64bppRGBAHalf => Some(Self::PF64bppRGBAHalf),
            guid if guid == &GUID_WICPixelFormat64bppRGBFixedPoint => Some(Self::PF64bppRGBFixedPoint),
            guid if guid == &GUID_WICPixelFormat64bppRGBHalf => Some(Self::PF64bppRGBHalf),
            guid if guid == &GUID_WICPixelFormat72bpp8ChannelsAlpha => Some(Self::PF72bpp8ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat80bpp4ChannelsAlpha => Some(Self::PF80bpp4ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat80bpp5Channels => Some(Self::PF80bpp5Channels),
            guid if guid == &GUID_WICPixelFormat80bppCMYKAlpha => Some(Self::PF80bppCMYKAlpha),
            guid if guid == &GUID_WICPixelFormat8bppAlpha => Some(Self::PF8bppAlpha),
            guid if guid == &GUID_WICPixelFormat8bppCb => Some(Self::PF8bppCb),
            guid if guid == &GUID_WICPixelFormat8bppCr => Some(Self::PF8bppCr),
            guid if guid == &GUID_WICPixelFormat8bppGray => Some(Self::PF8bppGray),
            guid if guid == &GUID_WICPixelFormat8bppIndexed => Some(Self::PF8bppIndexed),
            guid if guid == &GUID_WICPixelFormat8bppY => Some(Self::PF8bppY),
            guid if guid == &GUID_WICPixelFormat96bpp5ChannelsAlpha => Some(Self::PF96bpp5ChannelsAlpha),
            guid if guid == &GUID_WICPixelFormat96bpp6Channels => Some(Self::PF96bpp6Channels),
            guid if guid == &GUID_WICPixelFormat96bppRGBFixedPoint => Some(Self::PF96bppRGBFixedPoint),
            guid if guid == &GUID_WICPixelFormat96bppRGBFloat => Some(Self::PF96bppRGBFloat),
            guid if guid == &GUID_WICPixelFormatBlackWhite => Some(Self::PFBlackWhite),
            guid if guid == &GUID_WICPixelFormatDontCare => Some(Self::PFDontCare),
            _ => None,
        }
    }
}

impl TryFrom<&GUID> for PixelFormat {
    type Error = Error;

    fn try_from(guid: &GUID) -> Result<Self, Self::Error> {
        Self::from_guid(guid).ok_or_else(invalid_arg)
    }
}

impl From<PixelFormat> for &'static GUID {
    fn from(format: PixelFormat) -> &'static GUID { format.as_guid() }
}

#[cfg_attr(test, derive(strum::EnumIter))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Container {
    Bmp,
    Dds,
    Gif,
    Heif, /* experimental */
    Ico,
    Jpeg,
    JpegXr,
    Png,
    Tiff,
    Webp, /* experimental */
}

impl Container {
    #[inline]
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Bmp => "bmp",
            Self::Dds => "dds",
            Self::Gif => "gif",
            Self::Heif => "heif",
            Self::Ico => "ico",
            Self::Jpeg => "jpg",
            Self::JpegXr => "jxr",
            Self::Png => "png",
            Self::Tiff => "tif",
            Self::Webp => "webp",
        }
    }

    #[inline]
    #[must_use]
    pub const fn as_guid(self) -> &'static GUID {
        match self {
            Self::Bmp => &GUID_ContainerFormatBmp,
            Self::Dds => &GUID_ContainerFormatDds,
            Self::Gif => &GUID_ContainerFormatGif,
            Self::Heif => &GUID_ContainerFormatHeif,
            Self::JpegXr => &GUID_ContainerFormatWmp,
            Self::Ico => &GUID_ContainerFormatIco,
            Self::Jpeg => &GUID_ContainerFormatJpeg,
            Self::Png => &GUID_ContainerFormatPng,
            Self::Tiff => &GUID_ContainerFormatTiff,
            Self::Webp => &GUID_ContainerFormatWebp,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_guid(guid: &GUID) -> Option<Self> {
        match guid {
            guid if guid == &GUID_ContainerFormatBmp => Some(Self::Bmp),
            guid if guid == &GUID_ContainerFormatDds => Some(Self::Dds),
            guid if guid == &GUID_ContainerFormatGif => Some(Self::Gif),
            guid if guid == &GUID_ContainerFormatWmp => Some(Self::JpegXr),
            guid if guid == &GUID_ContainerFormatHeif => Some(Self::Heif),
            guid if guid == &GUID_ContainerFormatIco => Some(Self::Ico),
            guid if guid == &GUID_ContainerFormatJpeg => Some(Self::Jpeg),
            guid if guid == &GUID_ContainerFormatPng => Some(Self::Png),
            guid if guid == &GUID_ContainerFormatTiff => Some(Self::Tiff),
            guid if guid == &GUID_ContainerFormatWebp => Some(Self::Webp),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            ext if ext.eq_ignore_ascii_case("avci") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("avcs") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("avif") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("avifs") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("bmp") => Some(Self::Bmp),
            ext if ext.eq_ignore_ascii_case("dds") => Some(Self::Dds),
            ext if ext.eq_ignore_ascii_case("dib") => Some(Self::Bmp),
            ext if ext.eq_ignore_ascii_case("gif") => Some(Self::Gif),
            ext if ext.eq_ignore_ascii_case("heic") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("heics") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("heif") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("heifs") => Some(Self::Heif),
            ext if ext.eq_ignore_ascii_case("ico") => Some(Self::Ico),
            ext if ext.eq_ignore_ascii_case("jfif") => Some(Self::Jpeg),
            ext if ext.eq_ignore_ascii_case("jpe") => Some(Self::Jpeg),
            ext if ext.eq_ignore_ascii_case("jpeg") => Some(Self::Jpeg),
            ext if ext.eq_ignore_ascii_case("jpg") => Some(Self::Jpeg),
            ext if ext.eq_ignore_ascii_case("jxr") => Some(Self::JpegXr),
            ext if ext.eq_ignore_ascii_case("png") => Some(Self::Png),
            ext if ext.eq_ignore_ascii_case("tif") => Some(Self::Tiff),
            ext if ext.eq_ignore_ascii_case("tiff") => Some(Self::Tiff),
            ext if ext.eq_ignore_ascii_case("wdp") => Some(Self::JpegXr),
            ext if ext.eq_ignore_ascii_case("webp") => Some(Self::Webp),
            _ => None,
        }
    }
}

impl TryFrom<&GUID> for Container {
    type Error = Error;

    fn try_from(guid: &GUID) -> Result<Self, Self::Error> {
        Self::from_guid(guid).ok_or_else(invalid_arg)
    }
}

impl From<Container> for &'static GUID {
    fn from(container: Container) -> &'static GUID { container.as_guid() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container() {
        use strum::IntoEnumIterator;

        for container in Container::iter() {
            let ext = container.extension();
            let guid = container.as_guid();
            assert_eq!(Some(container), Container::from_extension(ext));
            assert_eq!(Some(container), Container::from_guid(guid));
            assert_eq!(Ok(container), Container::try_from(guid));
            assert!(PixelFormat::try_from(guid).is_err());
        }
    }

    #[test]
    fn test_pixelformat() {
        use strum::IntoEnumIterator;

        for format in PixelFormat::iter() {
            let guid = format.as_guid();
            assert_eq!(Some(format), PixelFormat::from_guid(guid));
            assert_eq!(Ok(format), PixelFormat::try_from(guid));
            assert!(PixelFormat::try_from(&GUID::default()).is_err());
            assert!(Container::try_from(guid).is_err());
        }
    }

    #[test]
    fn look_for_new_pixelformats() {
        use windows::core::Interface;

        let _com = initialize_com::com_initialized().unwrap();
        let wic = crate::wic_factory().unwrap();
        let mut new_formats_found = false;

        unsafe {
            let enumerator = wic
                .CreateComponentEnumerator(
                    windows::Win32::Graphics::Imaging::WICPixelFormat.0 as u32,
                    windows::Win32::Graphics::Imaging::WICComponentEnumerateDefault.0 as u32,
                )
                .unwrap();

            loop {
                let mut next = [None];
                let mut count = 0;

                enumerator.Next(&mut next, &mut count).unwrap();
                if count == 0 {
                    break;
                }

                if let [Some(iunk)] = next {
                    let info: windows::Win32::Graphics::Imaging::IWICPixelFormatInfo =
                        iunk.cast().unwrap();
                    let guid = info.GetFormatGUID().unwrap();
                    if PixelFormat::from_guid(&guid).is_some() {
                        continue;
                    }

                    let mut name = [0_u16; 100];
                    let mut name_len = 0;
                    info.GetFriendlyName(&mut name, &mut name_len).unwrap();
                    let name_str = windows::core::HSTRING::from_wide(&name[.. name_len as usize]);
                    let bpp = info.GetBitsPerPixel().unwrap();
                    eprintln!("New format: {name_str}: {bpp} bpp");
                    new_formats_found = true;
                }
            }
        }
        assert!(!new_formats_found);
    }

    #[ignore]
    #[test]
    fn generate_bpp_table() {
        use strum::IntoEnumIterator;
        use windows::core::Interface;

        let _com = initialize_com::com_initialized().unwrap();
        let wic = crate::wic_factory().unwrap();

        for format in PixelFormat::iter() {
            let info: windows::Win32::Graphics::Imaging::IWICPixelFormatInfo = unsafe {
                let info = wic.CreateComponentInfo(format.as_guid());
                if info.is_err() {
                    println!("{} => unimplemented!(),", format.as_ref());
                    continue;
                }
                info.unwrap().cast().unwrap()
            };
            let bpp = unsafe { info.GetBitsPerPixel() }.unwrap();
            println!("{} => {bpp},", format.as_ref());
        }
    }
}
