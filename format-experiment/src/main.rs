use std::{collections::HashMap, num::TryFromIntError};

use crate::formats::TextureFormat;
use formats::Dimensions;
use packed_struct::prelude::*;
use formats::DXGI_FORMAT;

mod formats;

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", endian = "lsb")]
pub struct Format {
    #[packed_field(bits = "0")]
    hd: bool,
    #[packed_field(bits="01..=15")]
    width: u16,
    #[packed_field(bits = "16..=30")]
    height: u16,
    #[packed_field(bits = "31..=34")]
    mip_levels: u8, // might not even be needed, test
    #[packed_field(bits = "35..=42")]
    dxgi_format: u8,
    #[packed_field(bits = "43..=46")]
    array_size: u8,
    #[packed_field(bits = "47")]
    _reserved: ReservedZero<packed_bits::Bits<1>>,
}

impl TryFrom<TextureFormat> for Format {
    type Error = TryFromIntError;

    fn try_from(value: TextureFormat) -> Result<Self, Self::Error> {
        let (dimensions, d2) = value.all_dimensions();

        Ok(Format {
            hd: d2.is_some(),
            width: u16::try_from(dimensions.width)?,
            height: u16::try_from(dimensions.height)?,
            mip_levels: dimensions.mipmaps.into(),
            dxgi_format: u8::try_from(value.dxgi_format.0)?.into(),
            array_size: u8::try_from(value.array_size)?.into(),
            _reserved: Default::default()
        })
    }
}

impl From<Format> for TextureFormat {
    fn from(value: Format) -> Self {
        let mut dim = Dimensions {
            data_size: 0,
            width: value.width.into(),
            height: value.height.into(),
            mipmaps: value.mip_levels,
        };

        let dxgi_format = DXGI_FORMAT(u32::from(value.dxgi_format));

        let (mut standard, highres) = if value.hd {
            let (sm, hm) = if dim.mipmaps > 2 {
                (dim.mipmaps - 2, 2)
            } else {
                (1, 1)
            };
            let mut standard = Dimensions {
                data_size: 0,
                width: dim.width >> 2,
                height: dim.width >> 2,
                mipmaps: sm
            };
            dim.mipmaps = hm;
            dim.data_size = dim.data_size(dxgi_format, usize::from(value.array_size));
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
    let mut packed_formats: Vec<[u8;6]> = Vec::new();
    let formats: HashMap<String, TextureFormat> =
        serde_json::from_str(include_str!("../formats.json")).unwrap();

    let packed = pack(formats.values().copied());
    let unpacked = unpack(packed);

    for format in formats.values() {
        let format = Format::try_from(*format).unwrap();
        let packed = format.pack().unwrap();
        packed_formats.push(packed);
    }
    // eprintln!("{formats:?}");
}

fn pack<'a>(mut formats: impl Iterator<Item = TextureFormat> + 'a) -> impl Iterator<Item = [u8; 6]> + 'a {
    std::iter::from_fn(move || {
        formats.next().map(|format| {
            let temp= Format::try_from(format).unwrap();
            temp.pack().unwrap()
        })
    })
}

fn unpack<'a>(mut formats: impl Iterator<Item = [u8; 6]> + 'a) -> impl Iterator<Item = TextureFormat> + 'a {
    std::iter::from_fn(move || {
        formats.next().map(|packed| {
            let temp = Format::unpack(&packed).unwrap();
            TextureFormat::from(temp)
        })
    })
}
