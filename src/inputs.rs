use std::collections::{BTreeMap, BTreeSet};

use camino::{Utf8Path, Utf8PathBuf};

use crate::files::{
    base_name,
    format_for_file,
    is_ignored_ext,
    is_image_ext,
    is_texture_ext,
    merge_file_formats,
    Categorized,
    FileFormat,
    FileGroup,
    FileType,
    InputGroup,
    Uncategorized,
};
use crate::prelude::*;
use crate::util::{open_files_dialog, WalkArgs};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Action {
    Import,
    Export,
    Ignore,
    Error,
}

#[cfg_attr(feature = "debug-inputs", instrument(level = "trace", ret))]
fn action(file: &Utf8Path) -> Action {
    file.extension()
        .and_then(|ext| {
            if is_texture_ext(ext) {
                Some(Action::Export)
            } else if is_image_ext(ext) {
                Some(Action::Import)
            } else if is_ignored_ext(ext) {
                Some(Action::Ignore)
            } else {
                None
            }
        })
        .unwrap_or(Action::Error)
}

#[derive(Debug)]
pub enum Job {
    Import(FileGroup<Categorized>),
    Export(FileGroup<Categorized>),
    Batch(Inputs),
    Nothing,
}

#[derive(Debug)]
pub struct Inputs {
    pub textures: Vec<Categorized>,
    pub images:   Vec<Categorized>,
}

impl Inputs {
    pub fn default_action(&self) -> Action {
        if self.textures.len() > self.images.len() {
            Action::Export
        } else {
            Action::Import
        }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool { self.textures.is_empty() && self.images.is_empty() }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize { self.textures.len() + self.images.len() }
}

pub struct InputsIter {
    textures: std::vec::IntoIter<Categorized>,
    images:   std::vec::IntoIter<Categorized>,
}

impl Iterator for InputsIter {
    type Item = Categorized;

    fn next(&mut self) -> Option<Self::Item> { self.textures.next().or_else(|| self.images.next()) }
}

impl From<Inputs> for InputsIter {
    fn from(inputs: Inputs) -> Self {
        let Inputs { textures, images } = inputs;
        InputsIter {
            textures: textures.into_iter(),
            images:   images.into_iter(),
        }
    }
}

impl IntoIterator for Inputs {
    type IntoIter = InputsIter;
    type Item = Categorized;

    fn into_iter(self) -> Self::IntoIter { InputsIter::from(self) }
}

#[must_use]
pub fn make_job(inputs: Inputs) -> Job {
    let Inputs {
        mut textures,
        mut images,
    } = inputs;

    if textures.len() + images.len() > 1 {
        Job::Batch(Inputs { textures, images })
    } else if images.len() == 1 {
        Job::Import(FileGroup(images.swap_remove(0)))
    } else if textures.len() == 1 {
        Job::Export(FileGroup(textures.swap_remove(0)))
    } else {
        Job::Nothing
    }
}

fn group_input_files(
    files: impl Iterator<Item = Utf8PathBuf>,
) -> BTreeMap<(FileType, String), InputGroup> {
    let mut groups = BTreeMap::new();

    for file in files {
        if let Ok(file_type) = FileType::try_from(file.as_ref()) {
            let base_name = base_name(&file);
            groups
                .entry((file_type, base_name.to_owned()))
                .or_insert_with(|| InputGroup {
                    file_type,
                    inputs: BTreeSet::new(),
                })
                .inputs
                .insert(file);
        }
    }
    groups
}

pub fn group(
    iter: impl Iterator<Item = Utf8PathBuf>,
) -> BTreeMap<(FileType, String), Vec<Utf8PathBuf>> {
    let mut grouped: BTreeMap<(FileType, String), Vec<Utf8PathBuf>> = BTreeMap::new();

    for (file_type, file) in iter.filter_map(|file| {
        Some((
            FileType::try_from(file.as_ref())
                .log_failure_with(|| format!("Determining file type of {file}"))
                .ok()?,
            file,
        ))
    }) {
        let key = (file_type, base_name(&file).to_string());
        grouped.entry(key).or_default().push(file);
    }

    grouped
}

pub fn categorize(grouped: BTreeMap<(FileType, String), Vec<Utf8PathBuf>>) -> Inputs {
    let (textures, images) = grouped
        .into_iter()
        .map(|((file_type, _), files)| Categorized { file_type, files })
        .partition(|Categorized { file_type, .. }| file_type == &FileType::Texture);

    Inputs { textures, images }
}

pub fn gather_from_args() -> Inputs {
    let args = std::env::args().skip(1).map(Utf8PathBuf::from);

    gather_iter(args)
}

pub fn gather(from: impl Into<Utf8PathBuf>) -> Inputs { gather_iter(std::iter::once(from.into())) }

pub fn gather_iter<'a>(iter: impl Iterator<Item = Utf8PathBuf> + 'a) -> Inputs {
    categorize(group(walk(iter)))
    // FIXME
    // if inputs.is_empty() {
    //     if let Some(selected) =
    //         open_files_dialog()?.log_failure_as("Failed to open Windows file
    // picker")     {
    //         inputs = selected;
    //     } else {
    //         return Ok(None);
    //     }
    // }
}

pub fn walk<'a>(
    iter: impl Iterator<Item = Utf8PathBuf> + 'a,
) -> impl Iterator<Item = Utf8PathBuf> + 'a {
    let mut walker = WalkArgs::new(iter);

    std::iter::from_fn(move || {
        while let Some(next) = walker.next() {
            if !next.as_str().contains(".custom.") {
                return Some(next);
            }
        }
        None
    })
}
