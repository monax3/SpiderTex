use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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
    FileGroups,
    FileType,
    Grouped,
    InputGroup,
};
use crate::prelude::*;
use crate::util::WalkArgs;

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

#[cfg(disabled)]
impl Inputs {
    #[must_use]
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

    pub fn add_pairs(&mut self) {
        for texture in &mut self.textures {
            texture.files.retain(|f| {
                if f.as_str().contains(".custom") {
                    event!(WARN, "Skipping {f}");
                    false
                } else {
                    true
                }
            });

            let set = BTreeSet::from_iter(&texture.files);
            let mut extras = Vec::new();

            if let Some((file, base_name)) = texture.files.first().map(|f| (f, base_name(f))) {
                let raw = Utf8Path::new(base_name).with_extension("raw");
                if !set.contains(&raw) && raw.exists() {
                    event!(DEBUG, "Adding {raw}");
                    extras.push(raw);
                }
                let hd = Utf8PathBuf::from(format!("{base_name}_hd.texture"));
                if !set.contains(&hd) && hd.exists() {
                    event!(DEBUG, "Adding {hd}");
                    extras.push(hd);
                }
                let hd2 = Utf8PathBuf::from(format!("{base_name}.hd.texture"));
                if !set.contains(&hd2) && hd2.exists() {
                    event!(DEBUG, "Adding {hd2}");
                    extras.push(hd2);
                }
                let texture = Utf8PathBuf::from(format!("{base_name}.texture"));
                if !set.contains(&texture) && texture.exists() {
                    event!(DEBUG, "Adding {texture}");
                    extras.push(texture);
                }
            }
            if !extras.is_empty() {
                texture.files.extend(extras);
            }
        }
    }
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

#[cfg(disabled)]
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

pub fn group(iter: impl Iterator<Item = PathBuf>) -> FileGroups<Grouped> {
    let mut grouped: BTreeMap<String, BTreeSet<PathBuf>> = BTreeMap::new();

    for (key, file) in iter.filter_map(|file| {
        file.file_name()
            .and_then(OsStr::to_str)
            .map(|name| base_name(name).to_string())
            .map(|base_name| (base_name, file))
    }) {
        grouped.entry(key).or_default().insert(file);
    }

    FileGroups::from_iter(grouped)
}

#[cfg(disabled)]
pub fn categorize(grouped: BTreeMap<String, Vec<PathBuf>>) -> Inputs {
    let (textures, images) = grouped
        .into_iter()
        .map(|((file_type, _), files)| Categorized { file_type, files })
        .partition(|Categorized { file_type, .. }| file_type == &FileType::Texture);

    Inputs { textures, images }
}

pub fn gather_from_args() -> FileGroups<Grouped> {
    let args = std::env::args_os().skip(1).map(PathBuf::from);

    gather_from_iter(args)
}

pub fn gather(from: impl Into<PathBuf>) -> FileGroups<Grouped> {
    gather_from_iter(std::iter::once(from.into()))
}

pub fn gather_from_iter<'a, FILE>(iter: impl Iterator<Item = FILE> + 'a) -> FileGroups<Grouped> where FILE: Into<PathBuf> {
    group(walk(iter))
}

pub fn walk<'a, FILE>(
    mut iter: impl Iterator<Item = FILE> + 'a,
) -> impl Iterator<Item = PathBuf> + 'a where FILE: Into<PathBuf> {
    let mut dirs: VecDeque<walkdir::IntoIter> = VecDeque::new();

    std::iter::from_fn(move || {
        loop {
            while let Some(walkdir) = dirs.front_mut() {
                for entry in walkdir {
                    match entry {
                        Ok(entry) => {
                            if entry.path().is_file() {
                                return Some(entry.path().to_owned());
                            }
                        }
                        Err(error) => {
                            event!(ERROR, "Error building file list: {error}");
                        }
                    }
                }
                dirs.pop_front();
            }
            if let Some(entry) = iter.next() {
                let entry: PathBuf = entry.into();
                if entry.is_dir() {
                    dirs.push_back(walkdir::WalkDir::new(entry).into_iter());
                    continue;
                } else if entry.is_file() {
                    return Some(entry);
                }
            }
            break None;
        }
    })
}

#[test]
fn test_walk_new() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .without_time()
        .with_line_number(true)
        .with_file(true)
        .init();

    // crate::util::log_for_tests(true);
    let paths = ["src", "tests"]
        .into_iter()
        .map(|item| Path::new(env!("CARGO_MANIFEST_DIR")).join(item));

    let span = span!(TRACE, "test span", hello = "world", "walking");
    let _entered = span.enter();

    for file in walk(paths) {
        event!(INFO, "{}", file.display());
    }
    event!(TRACE, "a trace");
}
