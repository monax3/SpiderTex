use std::{collections::HashMap, num::TryFromIntError};

use crate::formats::TextureFormat;
use formats::Dimensions;
use formats::DXGI_FORMAT;
use packed_struct::prelude::*;

mod formats;

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb")]
pub struct Format {
    #[packed_field(bits = "0")]
    hd: bool,
    #[packed_field(bits = "01..=14")]
    width: u16,
    #[packed_field(bits = "15..=28")]
    height: u16,
    #[packed_field(bits = "29..=32")]
    mipmaps_sd: u8,
    #[packed_field(bits = "33")]
    mipmaps_hd: u8,
    #[packed_field(bits = "34..=41")]
    dxgi_format: u8,
    #[packed_field(bits = "42..=47")]
    array_size: u8,
}

impl TryFrom<TextureFormat> for Format {
    type Error = TryFromIntError;

    fn try_from(value: TextureFormat) -> Result<Self, Self::Error> {
        let (dimensions, d2) = value.all_dimensions();

        let (mipmaps_sd, mipmaps_hd) = if let Some(d2) = d2 {
            (d2.mipmaps - 1, dimensions.mipmaps - 1)
        } else {
            (dimensions.mipmaps - 1, 0)
        };

        Ok(Format {
            hd: d2.is_some(),
            width: u16::try_from(dimensions.width)?,
            height: u16::try_from(dimensions.height)?,
            mipmaps_sd,
            mipmaps_hd,
            dxgi_format: u8::try_from(value.dxgi_format.0)?,
            array_size: u8::try_from(value.array_size)?,
        })
    }
}

impl From<Format> for TextureFormat {
    fn from(value: Format) -> Self {
        let mut dim = Dimensions {
            data_size: 0,
            width: value.width.into(),
            height: value.height.into(),
            mipmaps: value.mipmaps_sd + 1,
        };

        let dxgi_format = DXGI_FORMAT(u32::from(value.dxgi_format));

        let (mut standard, highres) = if value.hd {
            dim.mipmaps = value.mipmaps_hd + 1;
            dim.data_size = dim.data_size(dxgi_format, usize::from(value.array_size));

            let standard = Dimensions {
                data_size: 0,
                width: dim.width >> dim.mipmaps,
                height: dim.height >> dim.mipmaps,
                mipmaps: value.mipmaps_sd + 1,
            };
            (standard, Some(dim))
        } else {
            (dim, None)
        };
        standard.data_size = standard.data_size(dxgi_format, value.array_size.into());

        TextureFormat {
            dxgi_format: formats::DXGI_FORMAT(u32::from(value.dxgi_format)),
            standard,
            highres,
            array_size: usize::from(value.array_size),
        }
    }
}

fn main() {
    let mut packed_formats: Vec<[u8; 6]> = Vec::new();
    let formats: HashMap<String, TextureFormat> =
        serde_json::from_str(include_str!("../formats.json")).unwrap();

    let orig_formats: Vec<TextureFormat> = formats.values().copied().collect();

    let packed = pack(orig_formats.iter().copied());
    let unpacked = unpack(packed);

    for (orig, new) in orig_formats.iter().zip(unpacked) {
        if *orig != new {
            eprintln!("> {orig:?}");
            eprintln!("  {new:?}");
        }
    }
}

fn pack<'a>(
    mut formats: impl Iterator<Item = TextureFormat> + 'a,
) -> impl Iterator<Item = [u8; 6]> + 'a {
    std::iter::from_fn(move || {
        formats.next().map(|format| {
            let temp = Format::try_from(format).unwrap();
            temp.pack().unwrap()
        })
    })
}

fn unpack<'a>(
    mut formats: impl Iterator<Item = [u8; 6]> + 'a,
) -> impl Iterator<Item = TextureFormat> + 'a {
    std::iter::from_fn(move || {
        formats.next().map(|packed| {
            let temp = Format::unpack(&packed).unwrap();
            TextureFormat::from(temp)
        })
    })
}

const BASE_32: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz";

fn to_base32(bytes: impl Iterator<Item = u8>) -> impl Iterator<Item = u8> {
    const BITS: u8 = 5;

    let mut rem: u8 = 0;
    let mut rem_bits: u8 = 0;

    std::iter::from_fn(move || {
        if let Some(next) = bytes.next() {
            let i = next & ((1 << bits) - 1);
        }
    })
}
