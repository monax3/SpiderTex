// TESTS:
// characters_hero_hero_spiderman_tasm_textures_hero_spiderman_tasm_textile_red_01_g.texture

use std::borrow::Cow;
use std::collections::BTreeMap;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use windows::Win32::Graphics::Dxgi::Common::*;

use crate::headers::{read_texture, TextureFormatHeader, TEXTURE_HEADER_SIZE};

fn special_case(mut format: TextureFormat) -> TextureFormat {
    // Controller icons
    if format.sd_len() == 1296 {
        format.planes = ColorPlanes::Bgra;
    }

    format
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TextureFormat {
    pub example_file: String,
    pub format:       u32,
    pub standard:     Dimensions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highres:      Option<Dimensions>,
    pub planes:       ColorPlanes,
    pub array_size:   usize,
    pub raw_headers:  String,
}

impl TextureFormat {
    pub fn from_header(header: &TextureFormatHeader, file: &Utf8Path) -> Self {
        let format = header.format as u32;
        let standard = Dimensions {
            data_size: header.sd_len as usize,
            width:     header.sd_width as usize,
            height:    header.sd_height as usize,
            mipmaps:   header.sd_mipmaps,
        };

        let highres = (header.hd_width != header.sd_width || header.sd_height != header.hd_height)
            .then_some(Dimensions {
                data_size: header.hd_len as usize,
                width:     header.hd_width as usize,
                height:    header.hd_height as usize,
                mipmaps:   header.hd_mipmaps,
            });

        let example_file = file.file_name().unwrap_or_default().to_owned();

        let planes = match header.planes {
            18 => ColorPlanes::Luma,
            1 => {
                warn!("Format has the normal map color plane value");
                ColorPlanes::Rgba
            }
            0 => ColorPlanes::Rgba,
            value => {
                warn!("Format has an unidentified color plane value ({value})");
                ColorPlanes::Rgba
            }
        };

        special_case(Self {
            example_file,
            format,
            standard,
            highres,
            planes,
            array_size: header.array_size as usize,
            raw_headers: header.as_hexstring(),
        })
    }

    pub fn has_highres(&self) -> bool { self.highres.is_some() }

    pub fn key(&self) -> FormatKey { FormatKey(self.to_string()) }

    fn sd_file_len(&self) -> usize { self.standard.data_size + crate::headers::TEXTURE_HEADER_SIZE }

    #[allow(unused)]
    fn sd_len(&self) -> usize { self.standard.data_size }

    fn hd_len(&self) -> Option<usize> { self.highres.map(|dims| dims.data_size) }

    pub fn to_header(&self) -> TextureFormatHeader {
        // FIXME: This is actually a possible format.json error and shoudl be handled
        let bytes = hex::decode(&self.raw_headers).expect("Internal error");

        // FIXME: Ditto
        let header: &TextureFormatHeader =
            bytemuck::try_from_bytes(&bytes).expect("Internal error");

        header.to_owned()
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Dimensions {
    pub data_size: usize,
    pub width:     usize,
    pub height:    usize,
    pub mipmaps:   u8,
}

impl std::fmt::Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}/{}", self.width, self.height, self.mipmaps)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ColorPlanes {
    Rgb,
    Rgba,
    Bgra,
    Luma,
}

impl std::fmt::Display for TextureFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:?} {}",
            dxgi_format_identifier(self.format),
            self.planes,
            self.standard
        )?;

        if let Some(highres) = self.highres {
            write!(f, " {}", highres)?;
        }

        if self.array_size > 1 {
            write!(f, " x{}", self.array_size)?;
        }
        Ok(())
    }
}

fn dxgi_format_identifier(format: u32) -> Cow<'static, str> {
    match DXGI_FORMAT(format) {
        DXGI_FORMAT_BC1_UNORM => Cow::Borrowed("BC1"),
        DXGI_FORMAT_BC1_UNORM_SRGB => Cow::Borrowed("BC1 sRGB"),
        DXGI_FORMAT_BC2_UNORM => Cow::Borrowed("BC2"),
        DXGI_FORMAT_BC2_UNORM_SRGB => Cow::Borrowed("BC2 sRGB"),
        DXGI_FORMAT_BC3_UNORM => Cow::Borrowed("BC3"),
        DXGI_FORMAT_BC3_UNORM_SRGB => Cow::Borrowed("BC3 sRGB"),
        DXGI_FORMAT_BC4_UNORM => Cow::Borrowed("BC4"),
        DXGI_FORMAT_BC5_UNORM => Cow::Borrowed("BC5"),
        DXGI_FORMAT_BC7_UNORM => Cow::Borrowed("BC7"),
        DXGI_FORMAT_BC7_UNORM_SRGB => Cow::Borrowed("BC7 sRGB"),
        _ => Cow::Owned(format!("{}", format)),
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Clone)]
pub struct FormatKey(String);

impl std::fmt::Display for FormatKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FormatDb {
    pub formats: BTreeMap<FormatKey, TextureFormat>,
    pub lengths: BTreeMap<usize, FormatKey>,
}

#[cfg(debug_assertions)]
pub mod database {
    use super::*;

    const FORMAT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/formats.json");

    pub fn update_format(header: &TextureFormatHeader, file: &Utf8Path) -> Result<()> {
        let mut db = load_database()?;

        let format = TextureFormat::from_header(header, file);

        let key = format.key();

        if !db.formats.contains_key(&key) {
            check_clash(&db, &key, format.sd_file_len())?;
            if let Some(hd_len) = format.hd_len() {
                check_clash(&db, &key, hd_len)?;
                db.lengths.insert(hd_len, key.clone());
            }

            db.lengths.insert(format.sd_file_len(), key.clone());
            db.formats.insert(key.clone(), format);

            std::fs::write(FORMAT_FILE, &serde_json::to_vec_pretty(&db)?)?;

            eprintln!("Added format {key}");
        }
        Ok(())
    }

    fn check_clash(db: &FormatDb, key: &FormatKey, length: usize) -> Result<()> {
        if let Some(clash) = db.lengths.get(&length) {
            Err(eyre!(
                "CRITICAL! Formats {clash} and {key} both have length {length}"
            ))
        } else {
            Ok(())
        }
    }

    pub fn load_database() -> Result<FormatDb> {
        if std::path::Path::new(FORMAT_FILE).exists() {
            let format_str = std::fs::read_to_string(FORMAT_FILE)?;
            Ok(serde_json::from_str(&format_str)?)
        } else {
            Ok(FormatDb::default())
        }
    }
}

#[cfg(not(debug_assertions))]
pub mod database {
    use super::*;

    const DATABASE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/formats.json"));

    pub fn load_database() -> Result<FormatDb> { Ok(serde_json::from_str(DATABASE)?) }

    pub fn update_format(header: &TextureFormatHeader, file: &Utf8Path) -> Result<()> { Ok(()) }
}

pub fn guess_format(len: usize) -> Option<TextureFormat> {
    let db = database::load_database().ok()?;

    if let Some(key) = db.lengths.get(&len) {
        db.formats.get(key).map(Clone::clone)
    } else {
        None
    }
}

pub fn guess_dimensions(len: usize, format: &TextureFormat) -> Option<(Dimensions, bool)> {
    let len_without_header = len - TEXTURE_HEADER_SIZE;

    if format.standard.data_size == len {
        Some((format.standard, false))
    } else if format.standard.data_size == len_without_header {
        Some((format.standard, true))
    } else if let Some(highres) = format.highres {
        if highres.data_size == len {
            Some((highres, false))
        } else if highres.data_size == len_without_header {
            Some((highres, true))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn any_format() -> TextureFormat {
    let db = database::load_database().expect("Format database failed to load");

    db.formats
        .values()
        .next()
        .expect("Format database empty")
        .clone()
}

pub fn probe_textures(
    input_files: &[Utf8PathBuf],
) -> Result<(Option<TextureFormat>, &Utf8Path, Vec<u8>, FormatDb)> {
    let format_db = database::load_database()?;

    let (mut smallest, mut smallest_len): (Option<&Utf8Path>, usize) = (None, usize::MAX);
    let mut found_key: Option<&FormatKey> = None;

    for file in input_files {
        debug!("Probing {file}");

        let len = file.metadata()?.len() as usize;
        if len < smallest_len {
            smallest = Some(file.as_ref());
            smallest_len = len;
        }

        if let Some(key) = format_db.lengths.get(&len) {
            if found_key.is_some() {
                error!("Found conflicting format {}", key);
            } else {
                info!("Found matching format {}", key);
                found_key = Some(key);
            }
        }
    }

    let smallest = smallest.expect("probe_textures: No input files specified");

    let guessed_format = found_key
        .and_then(|key| format_db.formats.get(key))
        .map(ToOwned::to_owned);

    match read_texture(smallest, guessed_format.as_ref()) {
        Ok((probed_format, texture_data)) => {
            if probed_format.is_some() && probed_format != guessed_format {
                if guessed_format.is_none() {
                    warn!("New format discovered! Please submit this to the author");
                    // FIXME: add automatic format submission
                } else {
                    warn!("Probed and guessed format don't match!");
                }
            }
            Ok((
                probed_format.or(guessed_format),
                smallest,
                texture_data,
                format_db,
            ))
        }
        Err(error) => {
            let texture_data = std::fs::read(smallest)?;

            warn!("Failed to read header: {error}");

            Ok((guessed_format, smallest, texture_data, format_db))
        }
    }
}
