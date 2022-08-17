use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use color_eyre::eyre::{ensure, eyre};
use color_eyre::Result;
use zerocopy::{AsBytes, FromBytes, LayoutVerified};

use crate::TextureInfo;

const TEXTURE_TAG: &[u8; 20] = b"Texture Built File\0\0";

#[allow(clippy::unwrap_used)]
fn encode_raw_headers(
    file_header: &TextureFileHeader,
    texture_header: &TextureHeader,
    format_header: &TextureFormatHeader,
) -> String {
    let mut buffer = Vec::with_capacity(TEXTURE_HEADER_SIZE);

    buffer.write_all(file_header.as_bytes()).unwrap();
    buffer.write_all(texture_header.as_bytes()).unwrap();
    buffer.write_all(TEXTURE_TAG).unwrap();
    buffer.write_all(format_header.as_bytes()).unwrap();

    base64::encode(buffer)
}

#[derive(Debug, Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct TextureFileHeader {
    magic:      [u8; 4],
    header_len: u32,
    data_len_1: u32,
    unk1:       u32,
    unk2:       u32,
    data_len_2: u32,
    unk3:       u32,
    unk4:       u32,
    unk5:       u32,
}

impl TextureFileHeader {
    pub fn is_texture(&self) -> bool { self.magic == TEXTURE_MAGIC }

    pub fn validate(&self) -> bool {
        eprintln!(
            "File magic is {:02x}{:02x}{:02x}{:02x}",
            self.magic[0], self.magic[1], self.magic[2], self.magic[3]
        );

        eprintln!(
            "Data lengths are {:08x} ({}) and {:08x} {}",
            self.data_len_1, self.data_len_1, self.data_len_2, self.data_len_2
        );

        if self.header_len != 0x5c {
            eprintln!(
                "Header length is {:02x} ({})",
                self.header_len, self.header_len
            );
        }

        if self.unk1 != 0 {
            eprintln!("unk1 is {:x} ({})", self.unk1, self.unk1);
        }
        if self.unk2 != 0 {
            eprintln!("unk2 is {:x} ({})", self.unk2, self.unk2);
        }
        if self.unk3 != 0 {
            eprintln!("unk3 is {:x} ({})", self.unk3, self.unk3);
        }
        if self.unk4 != 0 {
            eprintln!("unk4 is {:x} ({})", self.unk4, self.unk4);
        }

        self.data_len_1 == self.data_len_2 && self.header_len == 0x5c
    }
}

#[derive(Debug, Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct TextureHeader {
    magic_1:    u32,
    magic_2:    u32,
    header_len: u32,
    version:    u32,
    magic_3:    u32,
    unk_len:    u32,
    format_len: u32,
}

impl TextureHeader {
    pub fn validate(&self) -> bool {
        eprintln!(
            "Header magics are {:08x} ({}) {:08x} ({}) {:08x} ({})",
            self.magic_1, self.magic_1, self.magic_2, self.magic_2, self.magic_3, self.magic_3
        );
        eprintln!(
            "Header length is {:02x} ({}){}",
            self.header_len,
            self.header_len,
            if self.header_len == 0x5c {
                ""
            } else {
                "Should be 0x5c"
            }
        );
        eprintln!("Header version is {:02x} ({})", self.version, self.version);
        eprintln!("Unknown length is {:02x} ({})", self.unk_len, self.unk_len);
        eprintln!(
            "Format length is {:02x} ({})",
            self.format_len, self.format_len
        );

        self.version == 1 && self.format_len == 0x2c && self.unk_len == self.format_len + 4
    }
}

#[derive(Debug, Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct TextureFormatHeader {
    data_len:            u32,
    hd_len:              u32,
    hd_width:            u16,
    hd_height:           u16,
    width:               u16,
    height:              u16,
    unk1:                u16,
    uncompressed_format: u16,
    compressed_format:   u16,
    unk3:                [u8; 8],
    mipmaps:             u8,
    unk5:                u8,
    hd_mipmaps:          u8,
    unk4:                [u8; 11],
}

impl TextureFormatHeader {
    fn validate(&self) -> bool {
        eprintln!("Data length is {:04x} ({})", self.data_len, self.data_len,);
        eprintln!("HD length is {:04x} ({})", self.hd_len, self.hd_len,);
        eprintln!("Texture inline width is {}", self.width);
        eprintln!("Texture inline height is {}", self.height);
        eprintln!("Texture HD width is {}", self.hd_width);
        eprintln!("Texture HD height is {}", self.hd_height);
        eprintln!("Mipmap count is {}", self.mipmaps);
        eprintln!("HD mipmap count is {}", self.hd_mipmaps);
        eprintln!("Compressed format is {}", self.compressed_format);
        eprintln!("Uncompressed format is {}", self.uncompressed_format);
        eprintln!(
            "First unknown values are {:02x} ({}) {:02x} ({})",
            self.unk1, self.unk1, self.unk5, self.unk5
        );

        for (i, unk) in self.unk3.iter().enumerate() {
            if *unk == 0 {
                continue;
            }
            eprintln!("Unknown value {} is {:04x} ({})", i, unk, unk);
        }

        for (i, unk) in self.unk4.iter().enumerate() {
            if *unk == 0 {
                continue;
            }
            eprintln!(
                "Unknown value {} is {:04x} ({})",
                i + self.unk3.len(),
                unk,
                unk
            );
        }

        true
    }
}

const TEXTURE_MAGIC: [u8; 4] = [0xb9, 0x80, 0x45, 0x5c];

pub const TEXTURE_HEADER_SIZE: usize = std::mem::size_of::<TextureFileHeader>()
    + std::mem::size_of::<TextureHeader>()
    + TEXTURE_TAG.len()
    + std::mem::size_of::<TextureFormatHeader>();

fn invalid_data() -> color_eyre::Report {
    eyre!("Unrecognized data in texture file, please contact this program's author")
}

pub fn read_texture_header(texture_file: &Path) -> Result<(TextureInfo, impl Read)> {
    let mut reader = BufReader::new(File::open(texture_file)?);

    let mut header_buffer = [0_u8; TEXTURE_HEADER_SIZE];
    reader.read_exact(&mut header_buffer)?;

    let (file_header, rest): (LayoutVerified<_, TextureFileHeader>, _) =
        LayoutVerified::new_from_prefix(header_buffer.as_slice()).ok_or_else(invalid_data)?;
    let file_header: TextureFileHeader = *file_header.into_ref();

    if !file_header.is_texture() {
        dbg!(file_header.magic);
        return Err(eyre!(
            "This texture does not contain any metadata. It may be corrupt or it may be a \
             high-resolution texture. To convert high-resolution textures, you need both the \
             original (\"01\") .texture file and the high-resolution .texture of the same name \
             (usually from the 14, 15 or 16 archives), with the high-resolution file renamed to \
             have _hd after the file name."
        ));
    }

    ensure!(file_header.validate(), invalid_data());

    let (texture_header, rest): (LayoutVerified<_, TextureHeader>, _) =
        LayoutVerified::new_from_prefix(rest).ok_or_else(invalid_data)?;
    let texture_header: TextureHeader = *texture_header.into_ref();
    ensure!(texture_header.validate(), invalid_data());

    let (tag, rest) = LayoutVerified::new_slice_from_prefix(rest, 20).ok_or_else(invalid_data)?;
    #[allow(clippy::unwrap_used)]
    let tag: [u8; 20] = tag.into_slice().try_into().unwrap();
    ensure!(
        &tag == TEXTURE_TAG,
        eyre!("Unrecognized data in texture file, please contact this program's author")
    );

    let (format_header, _): (LayoutVerified<_, TextureFormatHeader>, _) =
        LayoutVerified::new_from_prefix(rest).ok_or_else(invalid_data)?;
    let format_header: TextureFormatHeader = *format_header.into_ref();
    ensure!(
        format_header.validate(),
        eyre!("Unrecognized data in texture file, please contact this program's author")
    );

    let TextureFormatHeader {
        data_len,
        hd_len,
        width,
        height,
        hd_width,
        hd_height,
        compressed_format,
        uncompressed_format,
        mipmaps,
        hd_mipmaps,
        ..
    } = format_header;

    let uncompressed_format = uncompressed_format as u32;
    let compressed_format = compressed_format as u32;
    // let format = PixelFormat::try_from(format)?;
    let raw_headers = encode_raw_headers(&file_header, &texture_header, &format_header);

    Ok((
        TextureInfo {
            data_len,
            hd_len,
            width,
            height,
            hd_width,
            hd_height,
            compressed_format,
            uncompressed_format,
            mipmaps,
            hd_mipmaps,
            raw_headers,
        },
        reader,
    ))
}

#[test]
fn test_header_sizes() {
    assert_eq!(std::mem::size_of::<TextureFileHeader>(), 0x24);
    assert_eq!(std::mem::size_of::<TextureHeader>(), 0x1c);
    assert_eq!(std::mem::size_of::<TextureFormatHeader>(), 0x2c);
    assert_eq!(TEXTURE_HEADER_SIZE, 0x80);
}
