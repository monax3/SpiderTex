//! .texture file format:
//! 00 .. 24 [`FileHeader`]
//! 24 .. 40 [`TextureHeader`]
//! 40 .. 54 [`TEXTURE_TAG`]
//! 54 .. 80 [`TextureFormatHeader`]
//! 80 ..    Raw image data

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};

use bytemuck::{Pod, Zeroable};
use camino::Utf8Path;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

use crate::formats::{ColorPlanes, ImageFormat};
use crate::prelude::*;

pub const TEXTURE_HEADER_SIZE: usize = Header::SIZE;
pub const TEXTURE_TAG: &[u8; 20] = b"Texture Built File\0\0";

pub fn texture_format_overrides(format: &mut TextureFormat) {
    #[cfg(feature = "debug-formats")]
    event!(TRACE, ?format, "crc={:08x}", header_crc);

    let expected =
        dxtex::expected_size_array(format.dxgi_format, format.standard, format.array_size);

    #[cfg(windows)]
    if format.standard.data_size != expected {
        if format.standard.data_size % expected == 0 {
            format.array_size = format.standard.data_size / expected;
            event!(
                TRACE,
                "{format}: Overriding array size ({}) to match data size",
                format.array_size
            );
        } else {
            event!(
                ERROR,
                "{format}: Data size ({}) doesn't match expected size ({expected})",
                format.standard.data_size
            );
        }
    }
}

#[derive(Pod, Copy, Clone, Zeroable)]
#[repr(C)]
pub struct Header(FileHeader, TextureHeader, [u8; 20], FormatHeader);

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct FileHeader {
    magic: [u8; 4],
    header_len: u32,
    data_len_1: u32,
    unk1: [u32; 2],
    data_len_2: u32,
    unk2: [u32; 3],
}

#[derive(Debug, Copy, Clone, Pod, Zeroable, Default)]
#[repr(C)]
pub struct TextureHeader {
    magic_1: [u8; 4],
    magic_2: [u8; 4],
    header_len: u32,
    version: u32,
    magic_3: [u8; 4],
    len_plus_4: u32,
    format_len: u32,
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct FormatHeader {
    pub sd_len: u32,
    pub hd_len: u32,
    pub hd_width: u16,
    pub hd_height: u16,
    pub sd_width: u16,
    pub sd_height: u16,
    pub array_size: u16,
    pub stex_format: u8,
    pub planes: u8,
    pub format: u16,
    pub zeroes1: [u8; 5],
    pub unk1: u8,
    pub unk2: u8,
    pub sd_mipmaps_high: u8,
    pub sd_mipmaps: u8,
    pub hd_mipmaps_high: u8,
    pub hd_mipmaps: u8,
    pub unk3: u8,
    pub unk4: u8,
    pub zeroes2: [u8; 9],
}

impl FileHeader {
    // Header size is for testing struct sizes are correct
    const MAGIC: [u8; 4] = [0xb9, 0x80, 0x45, 0x5c];
    #[allow(unused)]
    const SIZE: usize = 0x24;

    #[must_use]
    pub fn with_length(data_len: usize) -> Self {
        let data_len: u32 = data_len as u32;

        Self {
            magic: Self::MAGIC,
            header_len: Header::MAIN_HEADER_SIZE as u32,
            data_len_1: data_len,
            data_len_2: data_len,
            unk1: Default::default(),
            unk2: Default::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn has_magic(&self) -> bool {
        self.magic == Self::MAGIC
    }

    pub fn check(&self, format: Option<&TextureFormat>) {
        expected("FILE_MAGIC", self.magic, Self::MAGIC, fmt_array_hex);
        expected(
            "FILE_HDR_LEN",
            self.header_len as usize,
            Header::MAIN_HEADER_SIZE,
            fmt_generic,
        );

        if let Some(format) = format {
            expected(
                "FILE_DATA_LEN1",
                self.data_len_1 as usize,
                format.standard.data_size,
                fmt_generic,
            );
            expected(
                "FILE_DATA_LEN2",
                self.data_len_2 as usize,
                format.standard.data_size,
                fmt_generic,
            );
        }

        let unks = self.unk1.into_iter().chain(self.unk2).enumerate();
        for (i, unk) in unks {
            expected(&format!("FILE_UNK{i}"), unk, 0, fmt_generic);
        }
    }
}

fn expected<T: PartialEq>(name: &str, value: T, expected: T, formatter: impl Fn(T) -> String) {
    if value != expected {
        event!(
            DEBUG,
            "Header value '{name}' has an unexpected value: {} insetad of {}",
            formatter(value),
            formatter(expected)
        );
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

fn fmt_debug<T: std::fmt::Debug>(value: T) -> String {
    format!("{value:?}")
}

impl TextureHeader {
    const MAGIC: [[u8; 4]; 3] = [
        [0x31, 0x54, 0x41, 0x44],
        [0xb9, 0x80, 0x45, 0x5c],
        [0x93, 0x35, 0xde, 0x4e],
    ];
    #[allow(unused)]
    const SIZE: usize = 0x1c;
    const VERSION: u32 = 1;

    #[must_use]
    pub const fn new() -> Self {
        Self {
            header_len: Header::MAIN_HEADER_SIZE as u32,
            magic_1: Self::MAGIC[0],
            magic_2: Self::MAGIC[1],
            magic_3: Self::MAGIC[2],
            version: 1,
            len_plus_4: (FormatHeader::SIZE + 4) as u32,
            format_len: FormatHeader::SIZE as u32,
        }
    }

    pub fn check(&self) {
        expected("TEX_MAGIC1", self.magic_1, Self::MAGIC[0], fmt_array_hex);
        expected("TEX_MAGIC2", self.magic_2, Self::MAGIC[1], fmt_array_hex);
        expected("TEX_MAGIC3", self.magic_3, Self::MAGIC[2], fmt_array_hex);
        expected(
            "TEX_HDR_LEN",
            self.header_len as usize,
            Header::MAIN_HEADER_SIZE,
            fmt_generic,
        );
        expected("TEX_HDR_VERSION", self.version, Self::VERSION, fmt_generic);
        expected(
            "TEX_LEN4",
            self.len_plus_4 as usize,
            FormatHeader::SIZE + 4,
            fmt_generic,
        );
    }
}

impl FormatHeader {
    const SIZE: usize = 0x2c;

    #[must_use]
    pub fn to(&self) -> TextureFormat {
        let mut format = self.into();
        texture_format_overrides(&mut format);
        format
    }

    pub fn check(&self, format: Option<&TextureFormat>) {
        if let Some(format) = format {
            expected(
                "FMT_SD_LEN",
                self.sd_len as usize,
                format.standard.data_size,
                fmt_generic,
            );
            expected(
                "FMT_SD_WIDTH",
                self.sd_width as usize,
                format.standard.width,
                fmt_generic,
            );
            expected(
                "FMT_SD_HEIGHT",
                self.sd_height as usize,
                format.standard.height,
                fmt_generic,
            );
            expected(
                "FMT_SD_MIPMAPS",
                self.sd_mipmaps,
                format.standard.mipmaps,
                fmt_generic,
            );

            if self.sd_width != self.hd_width {
                expected(
                    "FMT_HD_LEN",
                    self.hd_len as usize,
                    format.highres.map_or(0, |h| h.data_size),
                    fmt_generic,
                );
                expected(
                    "FMT_HD_WIDTH",
                    self.hd_width as usize,
                    format.highres.map_or(0, |h| h.width),
                    fmt_generic,
                );
                expected(
                    "FMT_HD_HEIGHT",
                    self.hd_height as usize,
                    format.highres.map_or(0, |h| h.height),
                    fmt_generic,
                );
                expected(
                    "FMT_HD_MIPMAPS",
                    self.hd_mipmaps,
                    format.highres.map_or(0, |h| h.mipmaps),
                    fmt_generic,
                );
            }

            expected(
                "FMT_ARRAY",
                self.array_size as usize,
                format.array_size,
                fmt_generic,
            );
            expected(
                "FMT_DXFORMAT",
                DXGI_FORMAT(self.format.into()),
                format.dxgi_format,
                fmt_debug,
            );
            expected(
                "FMT_SD_MM_HIGH",
                self.sd_mipmaps_high,
                0,
                fmt_debug,
            );
            expected(
                "FMT_HD_MM_HIGH",
                self.hd_mipmaps_high,
                0,
                fmt_debug,
            );
        }

        let zeroes = self
            .zeroes1
            .into_iter()
            .chain(self.zeroes2)
            .enumerate();

        for (i, zero) in zeroes {
            if zero != 0 {
            expected(&format!("FMT_ZERO{}", i + 1), zero, 0, fmt_generic);
            }
        }
    }

    #[inline]
    pub fn from_hexstring(hex: &str) -> Result<Self> {
        let mut bytes = hex::decode(hex)?;
        bytes.extend(std::iter::repeat(0).take(Self::SIZE - bytes.len()));
        Ok(*bytemuck::from_bytes(&bytes))
    }

    #[inline]
    #[must_use]
    pub fn as_hexstring(&self) -> String {
        let bytes = bytemuck::bytes_of(self);
        let pos = bytes.iter().rposition(|b| *b != 0).unwrap_or_default();
        hex::encode(&bytes[..pos])
    }
}

impl Header {
    const MAIN_HEADER_SIZE: usize = Self::SIZE - FileHeader::SIZE;
    const SIZE: usize = 0x80;

    pub const fn file(&self) -> &FileHeader {
        &self.0
    }

    pub const fn hdr(&self) -> &TextureHeader {
        &self.1
    }

    pub const fn tag(&self) -> &[u8; 20] {
        &self.2
    }

    pub const fn fmt(&self) -> &FormatHeader {
        &self.3
    }

    pub fn has_magic(&self) -> bool {
        self.file().has_magic()
    }

    fn check(&self, format: Option<&TextureFormat>) {
        expected("TAG", *self.tag(), *TEXTURE_TAG, fmt_array_string);

        self.file().check(format);
        self.hdr().check();
        self.fmt().check(format);
    }
}

pub fn read_header(texture_file: &Utf8Path) -> Result<(Option<FormatHeader>, impl Read)> {
    let mut reader = BufReader::new(File::open(texture_file)?);
    let mut header_buffer = [0_u8; TEXTURE_HEADER_SIZE];
    reader.read_exact(&mut header_buffer)?;

    let header: &Header =
        bytemuck::try_from_bytes(&header_buffer).expect("read_textures has the wrong buffer size");
    if !header.has_magic() {
        reader.seek(SeekFrom::Start(0))?;
        return Ok((None, reader));
    }
    #[cfg(feature = "debug-formats")]
    header.check(None);

    Ok((Some(*header.fmt()), reader))
}

pub fn read_texture(texture_file: &Utf8Path) -> Result<(Option<FormatHeader>, Vec<u8>)> {
    let (format, mut reader) = read_header(texture_file)?;
    let data_size = format.as_ref().map_or(0, |f| f.sd_len as usize);

    let mut buf = Vec::with_capacity(data_size);
    reader.read_to_end(&mut buf)?;

    Ok((format, buf))
}

#[test]
fn test_header_sizes() {
    assert_eq!(std::mem::size_of::<FileHeader>(), FileHeader::SIZE);
    assert_eq!(std::mem::size_of::<TextureHeader>(), TextureHeader::SIZE);
    assert_eq!(std::mem::size_of::<FormatHeader>(), FormatHeader::SIZE);
    assert_eq!(
        Header::SIZE,
        FileHeader::SIZE + TextureHeader::SIZE + FormatHeader::SIZE + TEXTURE_TAG.len()
    );
}

impl TryFrom<&TextureFormat> for FormatHeader {
    type Error = Error;

    fn try_from(format: &TextureFormat) -> Result<Self> {
        todo!()
    }
}
