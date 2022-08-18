use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use camino::Utf8Path;
use tracing::debug;

use color_eyre::eyre::{ensure, eyre};
use color_eyre::Result;
use tracing::warn;
use zerocopy::{AsBytes, FromBytes, LayoutVerified};
use bytemuck::{Pod, Zeroable};

use crate::TextureInfo;
use crate::formats::TextureFormat;

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

#[derive(Debug, Copy, Clone, AsBytes, FromBytes, Pod, Zeroable, Default)]
#[repr(C)]
pub struct TextureFileHeader {
    magic:      [u8; 4],
    header_len: u32,
    data_len_1: u32,
    unk1:       [u32; 2],
    data_len_2: u32,
    unk2:       [u32; 3],
}

impl TextureFileHeader {
    const MAGIC: [u8; 4] = [0xb9, 0x80, 0x45, 0x5c];

    pub fn with_length(data_len: usize) -> Self {
        let data_len: u32 = data_len.try_into().expect("Internal error");

        Self {
            magic: Self::MAGIC,
            header_len: TextureHeader::LEN.try_into().expect("Internal error"),
            data_len_1: data_len,
            data_len_2: data_len,
            unk1: Default::default(),
            unk2: Default::default(),
        }
    }

    pub fn has_magic(&self) -> bool { self.magic == Self::MAGIC }

    pub fn check(&self, format: Option<&TextureFormat>) {
        expected("FILE_MAGIC", self.magic, Self::MAGIC, fmt_array_hex);
        expected("FILE_HDRLEN", self.header_len as usize, TextureHeader::LEN, fmt_generic);

        if let Some(format) = format {
            expected("FILE_DATALEN1", self.data_len_1 as usize, format.standard.data_size, fmt_generic);
            expected("FILE_DATALEN2", self.data_len_2 as usize, format.standard.data_size, fmt_generic);
        }

        let unks = self.unk1.into_iter().chain(self.unk2).enumerate();
        for (i, unk) in unks {
            expected(&format!("FILE_UNK{i}"), unk, 0, fmt_generic);
        }
    }
}

#[derive(Debug, Copy, Clone, AsBytes, FromBytes, Pod, Zeroable, Default)]
#[repr(C)]
pub struct TextureHeader {
    magic_1:    [u8; 4],
    magic_2:    [u8; 4],
    header_len: u32,
    version:    u32,
    magic_3:    [u8; 4],
    len_plus_4:    u32,
    format_len: u32,
}

fn expected<T: PartialEq>(name: &str, value: T, expected: T, formatter: impl Fn(T) -> String) {
    if value != expected {
        warn!("Header value '{name}' has an unexpected value: {} insetad of {}", formatter(value), formatter(expected));
    }
}

fn fmt_array_hex<const LEN: usize>(array: [u8; LEN]) -> String {
    hex::encode(array)
}

fn fmt_array_string<const LEN: usize>(array: [u8; LEN]) -> String {
    String::from_utf8_lossy(&array).to_string()
}

fn fmt_generic<T: std::fmt::Display>(value: T) -> String {
    value.to_string()
}

impl TextureHeader {
    const LEN: usize = 0x5c;
    const VERSION: u32 = 1;
    const MAGIC1: [u8;4] = [0x31, 0x54, 0x41, 0x44];
    const MAGIC2: [u8;4] = [0xb9, 0x80, 0x45, 0x5c];
    const MAGIC3: [u8;4] = [0x93, 0x35, 0xde, 0x4e];

    pub fn new() -> Self {
        Self {
            header_len: Self::LEN.try_into().expect("Internal error"),
            magic_1: Self::MAGIC1,
            magic_2: Self::MAGIC2,
            magic_3: Self::MAGIC3,
            version: 1,
            len_plus_4: (TextureFormatHeader::LEN + 4).try_into().expect("Internal error"),
            format_len: TextureFormatHeader::LEN.try_into().expect("Internal error"),
        }
    }

    pub fn check(&self) {
        expected("TEX_MAGIC1", self.magic_1, Self::MAGIC1, fmt_array_hex);
        expected("TEX_MAGIC2", self.magic_2, Self::MAGIC2, fmt_array_hex);
        expected("TEX_HDR_LEN", self.header_len as usize, Self::LEN, fmt_generic);
        expected("TEX_HDR_VERSION", self.version, Self::VERSION, fmt_generic);
        expected("TEX_LEN4", self.len_plus_4 as usize, TextureFormatHeader::LEN + 4, fmt_generic);
    }
}

#[derive(Debug, Copy, Clone, AsBytes, FromBytes, Pod, Zeroable)]
#[repr(C)]
pub struct TextureFormatHeader {
    pub sd_len:          u32,
    pub hd_len:            u32,
    pub hd_width:          u16,
    pub hd_height:         u16,
    pub sd_width:             u16,
    pub sd_height:            u16,
    pub array_size:              u16,
    unk1: u8,
    pub planes: u8,
    pub format: u16,
    unk2:              [u8; 8],
    pub sd_mipmaps:           u8,
    unk3:              u8,
    pub hd_mipmaps:        u8,
    unk4:              [u8; 11],
}

impl TextureFormatHeader {
    const LEN: usize = 0x2c;

    fn check(&self, format: Option<&TextureFormat>) {
        if let Some(format) = format {
            expected("FMT_SD_LEN", self.sd_len as usize, format.standard.data_size, fmt_generic);
            expected("FMT_SD_WIDTH", self.sd_width as usize, format.standard.width, fmt_generic);
            expected("FMT_SD_HEIGHT", self.sd_height as usize, format.standard.height, fmt_generic);
            expected("FMT_SD_MIPMAPS", self.sd_mipmaps, format.standard.mipmaps, fmt_generic);

            if self.sd_width != self.hd_width {
            expected("FMT_HD_LEN", self.hd_len as usize, format.highres.map_or(0, |h| h.data_size), fmt_generic);
            expected("FMT_HD_WIDTH", self.hd_width as usize, format.highres.map_or(0, |h| h.width), fmt_generic);
            expected("FMT_HD_HEIGHT", self.hd_height as usize, format.highres.map_or(0, |h| h.height), fmt_generic);
            expected("FMT_HD_MIPMAPS", self.hd_mipmaps, format.highres.map_or(0, |h| h.mipmaps), fmt_generic);
            }

            expected("FMT_ARRAY", self.array_size as usize, format.array_size, fmt_generic);
            expected("FMT_DXFORMAT", self.format as u32, format.format, fmt_generic);
        }

        let unks = std::iter::once(self.unk1).chain(self.unk2).chain(std::iter::once(self.unk3)).chain(self.unk4).enumerate();

        for (i, unk) in unks {
            let exp = match i+1 {
                8 | 11 | 12 | 13 => 1,
                _ => 0,
            };

            expected(&format!("FMT_UNK{}", i+1), unk, exp, fmt_generic);
        }
    }

    pub fn as_hexstring(&self) -> String {
        hex::encode(self.as_bytes())
    }
}

pub const TEXTURE_HEADER_SIZE: usize = std::mem::size_of::<TextureFileHeader>()
    + std::mem::size_of::<TextureHeader>()
    + TEXTURE_TAG.len()
    + std::mem::size_of::<TextureFormatHeader>();

fn invalid_data() -> color_eyre::Report {
    eyre!("Unrecognized data in texture file, please contact this program's author")
}

#[derive(Pod, Copy, Clone, Zeroable)]
#[repr(C)]
struct Headers(TextureFileHeader, TextureHeader, [u8; 20], TextureFormatHeader);
impl Headers {
    fn file(&self) -> &TextureFileHeader { &self.0 }
    fn hdr(&self) -> &TextureHeader { &self.1 }
    fn tag(&self) -> &[u8; 20] { &self.2 }
    fn fmt(&self) -> &TextureFormatHeader { &self.3 }

    fn has_magic(&self) -> bool { self.file().has_magic() }

    fn check(&self, format: Option<&TextureFormat>) {
        expected("TAG", *self.tag(), *TEXTURE_TAG, fmt_array_string);

        self.file().check(format);
        self.hdr().check();
        self.fmt().check(format);
    }
}

pub fn read_texture_header_new(texture_file: &Utf8Path, guessed_format: Option<&TextureFormat>) -> Result<(Option<TextureFormat>, impl Read)> {
    let mut reader = BufReader::new(File::open(texture_file)?);
    let mut header_buffer = [0_u8; TEXTURE_HEADER_SIZE];
    reader.read_exact(&mut header_buffer)?;

    let headers: &Headers = bytemuck::try_from_bytes(&header_buffer).expect("read_textures has the wrong buffer size");
    if !headers.has_magic() {
        reader.seek(SeekFrom::Start(0))?;
        return Ok((None, reader));
    }
    headers.check(guessed_format);

    let format = TextureFormat::from_header(headers.fmt(), texture_file);
    Ok((Some(format), reader))
}

pub fn read_texture(texture_file: &Utf8Path, guessed_format: Option<&TextureFormat>) -> Result<(Option<TextureFormat>, Vec<u8>)> {
    let (format, mut reader) = read_texture_header_new(texture_file, guessed_format)?;
    let data_size = format.as_ref().map_or(0, |f| f.standard.data_size);

    let mut buf = Vec::with_capacity(data_size);
    reader.read_to_end(&mut buf)?;

    Ok((format, buf))
}

pub fn read_texture_header(texture_file: &Path) -> Result<(TextureInfo, impl Read)> {
    let mut reader = BufReader::new(File::open(texture_file)?);

    let mut header_buffer = [0_u8; TEXTURE_HEADER_SIZE];
    reader.read_exact(&mut header_buffer)?;

    let (file_header, rest): (LayoutVerified<_, TextureFileHeader>, _) =
        LayoutVerified::new_from_prefix(header_buffer.as_slice()).ok_or_else(invalid_data)?;
    let file_header: TextureFileHeader = *file_header.into_ref();

    if !file_header.has_magic() {
        dbg!(file_header.magic);
        return Err(eyre!(
            "This texture does not contain any metadata. It may be corrupt or it may be a \
             high-resolution texture. To convert high-resolution textures, you need both the \
             original (\"01\") .texture file and the high-resolution .texture of the same name \
             (usually from the 14, 15 or 16 archives), with the high-resolution file renamed to \
             have _hd after the file name."
        ));
    }

    file_header.check(None);

    let (texture_header, rest): (LayoutVerified<_, TextureHeader>, _) =
        LayoutVerified::new_from_prefix(rest).ok_or_else(invalid_data)?;
    let texture_header: TextureHeader = *texture_header.into_ref();
    texture_header.check();

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
    format_header.check(None);

    let TextureFormatHeader {
        sd_len,
        hd_len,
        sd_width,
        sd_height,
        hd_width,
        hd_height,
        array_size,
        format,
        sd_mipmaps,
        hd_mipmaps,
        ..
    } = format_header;

    let compressed_format = format as u32;
    let raw_headers = encode_raw_headers(&file_header, &texture_header, &format_header);

    #[allow(clippy::unwrap_used)]
    crate::formats::database::update_format(&format_header, camino::Utf8Path::from_path(texture_file).unwrap())?;

    Ok((
        TextureInfo {
            data_len: sd_len,
            hd_len,
            width: sd_width,
            height: sd_height,
            hd_width,
            hd_height,
            array_size,
            compressed_format,
            mipmaps: sd_mipmaps,
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
