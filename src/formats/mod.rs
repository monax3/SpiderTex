use camino::{Utf8Path, Utf8PathBuf};
pub use image::ImageFormat;
use tracing::{debug, error, info, warn};

use crate::prelude::*;
use crate::registry::FormatId;
use crate::texture_file::{self, TEXTURE_HEADER_SIZE};
pub(crate) mod dxgi;
pub use dxgi::DxgiFormatExt;
mod texture;
pub use texture::{Source, TextureFormat};
mod misc;
pub use misc::{ColorPlanes, Dimensions, ImageFormatExt};

pub fn print_formats<'a>(iter: impl Iterator<Item = &'a FormatId>) {
    for id in iter {
        error!("{id}");
    }
}

#[must_use]
pub fn guess_dimensions(len: usize, formats: &[TextureFormat]) -> Option<(Dimensions, bool)> {
    let len_without_header = len - TEXTURE_HEADER_SIZE;

    // FIXME: improve this to scan all formats

    if let Some(format) = formats.first().log_failure() {
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
    } else {
        None
    }
}

#[must_use]
pub fn guess_dimensions_2(len: usize, formats: &[&TextureFormat]) -> Option<(Dimensions, bool)> {
    let len_without_header = if len <= TEXTURE_HEADER_SIZE {
        len
    } else {
        len - TEXTURE_HEADER_SIZE
    };

    // FIXME: improve this to scan all formats

    if let Some(format) = formats.first().log_failure() {
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
    } else {
        None
    }
}

// TODO: this baby doesn't work anymore it needs to be adapted for groups
pub fn probe_textures_2<'a, 'r>(
    registry: &'r mut crate::registry::Registry,
    input_files: &'a [Utf8PathBuf],
) -> Result<(Vec<&'r TextureFormat>, &'a Utf8Path, Vec<u8>)> {
    let (mut smallest, mut smallest_len): (Option<&Utf8Path>, usize) = (None, usize::MAX);
    let mut found_keys: Vec<FormatId> = Vec::new();

    for file in input_files {
        debug!("Probing {file}");

        let len = file.metadata()?.len() as usize;
        if len < smallest_len {
            smallest = Some(file.as_ref());
            smallest_len = len;
        }

        let matching = registry.formats_with_size(len);
        if !matching.is_empty() {
            if found_keys.is_empty() {
                info!("Found matching formats {matching:?}");
                found_keys.extend(matching);
            } else {
                error!("Found conflicting formats {matching:?}");
            }
        }
    }

    let smallest = smallest.expect("probe_textures: No input files specified");

    // let mut guessed_formats: Vec<&TextureFormat> = found_keys
    //     .iter()
    //     .filter_map(|key| registry.get(*key))
    //     .collect();

    match texture_file::read_texture(smallest) {
        Ok((Some(probed), texture_data)) => {
            let probed = TextureFormat::from(probed);
            let id = probed.id();
            if !registry.known(id) {
                //FIXME
                warn!("New format discovered! Please submit this to the author");
                registry.update_format(probed, Some(smallest));
            }
            let probed = registry.get(id);

            let known = registry.get(id);
            if probed != known {
                error!("Format exists but is somehow different");
            }
            Ok((vec![probed], smallest, texture_data))
        }
        Ok((None, texture_data)) => {
            let guessed = registry.get_all(registry.formats_with_size(texture_data.len()));
            Ok((guessed, smallest, texture_data))
        }
        Err(error) => {
            Err(error)
            // At this point, reading the file should fail
            // let texture_data = std::fs::read(smallest)?;
            // warn!("Failed to read header: {error}");
            // Ok((guessed_formats, smallest, texture_data))
        }
    }
}
