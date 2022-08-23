use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};

use camino::{Utf8Path, Utf8PathBuf};

use crate::formats::ImageFormat;
use crate::images::{DxImport, Image, Warnings};
use crate::prelude::*;

const GROUP_SEP: char = '#';

#[must_use]
#[cfg_attr(feature = "debug-inputs", instrument(level = "trace", ret))]
pub fn base_name(file: &Utf8Path) -> &str {
    let without_ext = file
        .as_str()
        .rfind('.')
        .map_or(file.as_str(), |pos| &file.as_str()[.. pos]);

    let without_seps = without_ext
        .rfind(|c| c == GROUP_SEP)
        .map_or(without_ext, |sep| &without_ext[.. sep]);

    let without_suffix = without_seps.strip_suffix("_hd").unwrap_or(without_seps);

    // let without_path = without_suffix
    //     .rfind(|c| std::path::is_separator(c))
    //     .map_or(without_suffix, |path_sep| &without_suffix[path_sep + 1 ..]);

    #[cfg(feature = "debug-inputs")]
    {
        tracing::Span::current().record("without_ext", without_ext);
        tracing::Span::current().record("without_seps", without_seps);
        tracing::Span::current().record("without_suffix", without_suffix);
    }

    without_suffix
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFormat {
    Ready(TextureFormat),
    UserOverride(FormatId),
    MetaOverride(TextureFormat),
    FromHeader(TextureFormat),
    FromSize(Vec<FormatId>),
    Unknown,
}

// #[derive(Debug)]
// pub struct FileGroup {
//     pub file_type: FileType,
//     pub inputs:    Vec<Utf8PathBuf>,
//     pub format:    FileFormat,
//     pub status:    FileStatus,
// }

#[derive(Debug, Clone)]
pub enum FileStatus {
    Unknown,
    Ok(Warnings, Vec<Utf8PathBuf>),
    Error(String),
}

impl FileStatus {
    pub fn from_result(func: impl FnOnce() -> Result<Self>) -> Self {
        match func() {
            Ok(status) => status,
            Err(error) => Self::Error(error.to_string()),
        }
    }
}

impl<F, T> From<F> for FileStatus
where
    T: Into<Self>,
    F: FnOnce() -> T,
{
    fn from(closure: F) -> Self { closure().into() }
}

impl<T, E> From<Result<T, E>> for FileStatus
where
    T: Into<Self>,
    E: ToString,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(inner) => inner.into(),
            Err(error) => Self::Error(error.to_string()),
        }
    }
}

impl From<(Warnings, Vec<Utf8PathBuf>)> for FileStatus {
    fn from((warnings, files): (Warnings, Vec<Utf8PathBuf>)) -> Self { Self::Ok(warnings, files) }
}

impl From<Vec<Utf8PathBuf>> for FileStatus {
    fn from(files: Vec<Utf8PathBuf>) -> Self { Self::Ok(Warnings::new(), files) }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum FileType {
    Texture,
    Image(ImageFormat),
}

impl Ord for FileType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Texture, Self::Texture) | (Self::Image(_), Self::Image(_)) => {
                std::cmp::Ordering::Equal
            }
            (_, Self::Image(_)) => std::cmp::Ordering::Less,
            (_, Self::Texture) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for FileType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

impl From<ImageFormat> for FileType {
    fn from(format: ImageFormat) -> Self { Self::Image(format) }
}
impl TryFrom<&Utf8Path> for FileType {
    type Error = Error;

    fn try_from(value: &Utf8Path) -> Result<Self, Self::Error> {
        match value.extension() {
            Some(ext) if is_texture_ext(ext) => Ok(Self::Texture),
            Some(ext) => {
                if let Some(format) = ImageFormat::from_extension(ext) {
                    Ok(format.into())
                } else {
                    error_message("Unsupported file type")
                }
            }
            None => error_message("File has no extennsion"),
        }
    }
}

pub trait FileGroupInfo {
    fn file_type(&self) -> Option<&FileType> { None }
    fn iter_inputs<'a>(&'a self) -> Box<dyn Iterator<Item = Cow<'a, FileStatus>> + 'a> {
        Box::new(std::iter::empty())
    }
    fn iter_outputs<'a>(&'a self) -> Box<dyn Iterator<Item = Cow<'a, FileStatus>> + 'a> {
        Box::new(std::iter::empty())
    }

    fn input(&self) -> Cow<'_, FileStatus> { Cow::Owned(FileStatus::Unknown) }
    fn output(&self) -> Cow<'_, FileStatus> { Cow::Owned(FileStatus::Unknown) }
    fn output_format(&self) -> Option<&OutputFormat> { None }
}

impl<GROUP> std::ops::Deref for FileGroup<GROUP> {
    type Target = GROUP;

    fn deref(&self) -> &Self::Target { &self.0 }
}

// impl<GROUP> FileGroupInfo for FileGroupNg<GROUP> where GROUP: FileGroupInfo {
//     pub fn file_type(&self) -> Option<FileType>
//     where GROUP: FileGroupInfo {
//         self.0.file_type()
//     }

// }
// impl<GROUP: FileGroupInfo> FileGroupNg<GROUP> {
// }

impl FileGroupInfo for Uncategorized {
    fn iter_inputs<'a>(&'a self) -> Box<dyn Iterator<Item = Cow<'a, FileStatus>> + 'a> {
        make_filestatus_iter(&self.0)
    }

    fn input(&self) -> Cow<'_, FileStatus> {
        Cow::Owned(FileStatus::Ok(Warnings::new(), self.0.clone()))
    }
}

fn make_filestatus_iter<'a, FILE>(
    iter: impl IntoIterator<Item = FILE> + 'a,
) -> Box<dyn Iterator<Item = Cow<'a, FileStatus>> + 'a>
where FILE: Into<Utf8PathBuf> + 'a {
    Box::new(
        iter.into_iter()
            .map(|file| Cow::Owned(FileStatus::from(vec![file.into()]))),
    )
}

impl FileGroupInfo for Categorized {
    fn iter_inputs<'a>(&'a self) -> Box<dyn Iterator<Item = Cow<'a, FileStatus>> + 'a> {
        make_filestatus_iter(&self.files)
    }
}

impl FileGroupInfo for Scanned {
    fn file_type(&self) -> Option<&FileType> { Some(&self.file_type) }

    fn input(&self) -> Cow<'_, FileStatus> { Cow::Borrowed(&self.input) }

    fn output(&self) -> Cow<'_, FileStatus> {
        Cow::Owned(match self.output {
            OutputFormat::Exact { ref outputs, .. } => {
                FileStatus::Ok(Warnings::new(), outputs.clone())
            }
            OutputFormat::Candidates(_) => {
                FileStatus::Error("Multiple potential texture formats found".to_string())
            }
            OutputFormat::Unknown => {
                FileStatus::Error("No texture format could be identified".to_string())
            }
        })
    }

    fn output_format(&self) -> Option<&OutputFormat> { Some(&self.output) }
}

#[derive(Debug)]
pub struct FileGroup<GROUP>(pub GROUP);

#[derive(Debug)]
pub struct Uncategorized(pub Vec<Utf8PathBuf>);

#[derive(Debug)]
pub struct Categorized {
    pub file_type: FileType,
    pub files:     Vec<Utf8PathBuf>,
}

#[derive(Debug)]
pub struct Scanned {
    pub file_type: FileType,
    pub input:     FileStatus,
    pub output:    OutputFormat,
}

#[derive(Debug)]
pub enum OutputFormat {
    Exact {
        format:  TextureFormat,
        outputs: Vec<Utf8PathBuf>,
    },
    Candidates(Vec<TextureFormat>),
    Unknown,
}

impl FileGroup<Categorized> {
    #[must_use]
    pub fn scan(self) -> FileGroup<Scanned> {
        let Categorized { file_type, files } = self.0;

        match file_type {
            FileType::Texture => Self::scan_textures(files),
            FileType::Image(image_format) => Self::scan_images(image_format, files),
        }
    }

    #[must_use]
    pub fn scan_textures(files: Vec<Utf8PathBuf>) -> FileGroup<Scanned> {
        let mut formats = HashSet::new();

        let input = FileStatus::from(|| {
            for texture_file in &files {
                if let Some(texture_format) = ng_format_for_texture_file(texture_file) {
                    formats.insert(texture_format);
                }
            }
            if formats.is_empty() {
                let sizes: Vec<usize> = files
                    .iter()
                    .filter_map(|file| std::fs::metadata(file).map(|m| m.len() as usize).ok())
                    .collect();
                formats.extend(registry::formats_for_sizes(&sizes));
            }
            Ok::<_, Error>(files)
        });

        let output = if formats.len() > 1 {
            OutputFormat::Candidates(formats.into_iter().collect())
        } else if let (Some(format), FileStatus::Ok(_, inputs)) =
            (formats.into_iter().next(), &input)
        {
            let outputs = as_images(&format, inputs);
            OutputFormat::Exact { format, outputs }
        } else {
            OutputFormat::Unknown
        };

        FileGroup(Scanned {
            file_type: FileType::Texture,
            input,
            output,
        })
    }

    #[must_use]
    pub fn scan_images(image_format: ImageFormat, files: Vec<Utf8PathBuf>) -> FileGroup<Scanned> {
        let texture_formats: HashSet<TextureFormat> = files
            .iter()
            .filter_map(|image_file| ng_format_for_image_file(image_file))
            .collect();

        let exact_format = if texture_formats.len() == 1 {
            texture_formats.iter().next()
        } else {
            None
        };

        let input = FileStatus::from(|| {
            let warnings: Warnings = if let Some(texture_format) = exact_format {
                files
                    .iter()
                    .map(|image_file| DxImport::quick_check(texture_format, image_file))
                    .collect::<Result<_>>()?
            } else {
                Warnings::new()
            };
            Ok::<_, Error>((warnings, files))
        });

        let output =
            if let (Some(texture_format), FileStatus::Ok(_, inputs)) = (exact_format, &input) {
                let outputs = as_textures(texture_format, inputs);
                OutputFormat::Exact {
                    format: *texture_format,
                    outputs,
                }
            } else if texture_formats.len() > 1 {
                OutputFormat::Candidates(texture_formats.into_iter().collect())
            } else {
                OutputFormat::Unknown
            };

        FileGroup(Scanned {
            file_type: FileType::Image(image_format),
            input,
            output,
        })
    }
}

#[must_use]
#[cfg_attr(feature = "debug-inputs", instrument(ret))]
pub fn ng_format_for_image_file(image_file: &Utf8Path) -> Option<TextureFormat> {
    let file = Utf8PathBuf::from(base_name(image_file));

    try_read_meta(&file)
        .or_else(|| ng_format_for_texture_file(&file.with_extension("texture")))
        .or_else(|| ng_format_for_texture_file(&file.with_extension("raw")))
        .log_failure_with(|| format!("Failed to detect texture format of {image_file}"))
}

#[must_use]
#[cfg_attr(
    any(feature = "debug-inputs", feature = "debug-formats"),
    instrument(ret, skip(registry))
)]
pub fn ng_format_for_texture_file(texture_file: &Utf8Path) -> Option<TextureFormat> {
    registry().get_override(texture_file).or_else(|| {
        if texture_file.exists() {
            texture_file::read_header(texture_file)
                .log_failure_with(|| format!("Failed to read header of {texture_file}"))
                .ok()
                .and_then(|(header, _)| header.map(|header| header.to()))
        } else {
            None
        }
    })
}

#[derive(Debug, Clone)]
pub struct InputGroup {
    pub file_type: FileType,
    pub inputs:    BTreeSet<Utf8PathBuf>,
}

#[cfg(disabled)]
impl<FILE> FromIterator<FILE> for InputGroup
where FILE: Into<Utf8PathBuf>
{
    // fn from_iter<T: IntoIterator<Item = FILE>>(iter: T) -> Self { todo!() }

    // fn from_iter<T: IntoIterator<Item = (FileType, FILE)>>(iter: T) -> Self {
    //     let mut format = FileFormat::Unknown;
    //     let iter = iter.into_iter();

    //     let (size, _) = iter.size_hint();
    //     let mut inputs: Vec<Utf8PathBuf> = Vec::with_capacity(size);

    //     let outputs = if let Some(format) = format.exact() {
    //         output_files(format, self.file_type, &inputs, None)
    //     } else {
    //         event!(ERROR, ?inputs, ?format);
    //         error_message("An exact format couldn't be found for this file.")
    //     };

    //     Self {
    //         file_type,
    //         inputs,
    //     }
    // }
}

impl FileFormat {
    #[inline]
    #[must_use]
    pub fn exact(&self) -> Option<&TextureFormat> {
        match self {
            FileFormat::Ready(format)
            | FileFormat::FromHeader(format)
            | FileFormat::MetaOverride(format) => Some(format),
            FileFormat::UserOverride(id) => Some(registry::get(id)),
            FileFormat::FromSize(_) | FileFormat::Unknown => None,
        }
    }

    #[inline]
    #[must_use]
    #[cfg_attr(feature = "debug-formats", instrument(ret))]
    pub fn get_all(&self) -> Vec<&TextureFormat> {
        match self {
            FileFormat::Ready(ref format)
            | FileFormat::FromHeader(ref format)
            | FileFormat::MetaOverride(ref format) => vec![format],
            FileFormat::UserOverride(id) => vec![registry::get(id)],
            FileFormat::FromSize(formats) => registry::get_all(formats),
            FileFormat::Unknown => vec![],
        }
    }
}

pub fn output_files(
    format: &TextureFormat,
    file_type: FileType,
    files: &[Utf8PathBuf],
    output_format: Option<ImageFormat>,
) -> Result<Vec<Utf8PathBuf>> {
    match file_type {
        FileType::Image(image_format) => {
            let expected = format.num_images();
            if files.len() == expected {
                Ok(as_textures(format, files))
            } else if expected == 1 {
                event!(ERROR, %expected, len = %files.len(), format.array_size, "duuuupes {} {} {format} {image_format:?} {:?}", format.array_size, files.len(), format.default_image_format());
                error_message("Duplicate input files")
            } else {
                error_message(format!(
                    "{expected} image files are required for this array"
                ))
            }
        }
        FileType::Texture => Ok(as_images(format, files)),
    }
}

#[must_use]
pub fn as_images(texture_format: &TextureFormat, files: &[Utf8PathBuf]) -> Vec<Utf8PathBuf> {
    if let Some(first) = files.get(0).log_failure_as("as_images on an empty Vec") {
        let base = base_name(first);
        let image_format = texture_format.default_image_format();
        let num_images = texture_format.num_images();

        let ext = image_format
            .extensions_str()
            .first()
            .expect("Image format has no extensions");

        if num_images > 1 && !image_format.can_save_array() {
            let mut out = Vec::with_capacity(num_images);
            for i in 1 ..= num_images {
                let mut name = base.to_string();
                name.push_str(&format!("#{i:02}.{ext}"));
                out.push(Utf8PathBuf::from(name));
            }
            out
        } else {
            vec![Utf8PathBuf::from(base).with_extension(ext)]
        }
    } else {
        Vec::new()
    }
}

#[must_use]
pub fn as_textures(format: &TextureFormat, files: &[Utf8PathBuf]) -> Vec<Utf8PathBuf> {
    if let Some(first) = files.get(0).log_failure_as("as_textures on an empty Vec") {
        let mut base_name = base_name(first).to_string();
        base_name.push_str(".custom.texture");

        let texture = first.with_file_name(&base_name).with_extension("texture");
        // panic!("{files:?} ---- {texture} ---- {base_name}");

        if format.has_highres() {
            let raw = texture.with_extension("raw");
            vec![raw, texture]
        } else {
            vec![texture]
        }
    } else {
        Vec::new()
    }
}

pub fn merge_file_formats(current: FileFormat, next: FileFormat) -> FileFormat {
    #[cfg(feature = "debug-formats")]
    let span = span!(DEBUG, "merge_file_formats", ?current, ?next);
    #[cfg(feature = "debug-formats")]
    let _entered = span.enter();

    #[allow(clippy::match_same_arms)]
    let ret = match (&current, &next) {
        (FileFormat::Unknown, _) => next,
        (FileFormat::Ready(_), _) => current,
        (_, FileFormat::Ready(_)) => next,
        (FileFormat::UserOverride(_), _) => current,
        (_, FileFormat::UserOverride(_)) => next,
        (FileFormat::MetaOverride(_), _) => current,
        (_, FileFormat::MetaOverride(_)) => next,
        (FileFormat::FromHeader(current_id), FileFormat::FromHeader(next_id))
            if current_id != next_id =>
        {
            FileFormat::Unknown
        }
        (FileFormat::FromHeader(_), _) => current,
        (_, FileFormat::FromHeader(_)) => next,
        (FileFormat::FromSize(current_ids), FileFormat::FromSize(next_ids))
            if current_ids != next_ids =>
        {
            FileFormat::Unknown
        }
        (FileFormat::FromSize(_), _) => current,
    };

    #[cfg(feature = "debug-formats")]
    event!(DEBUG, matched = ?ret);
    ret
}

pub fn merge_file_format_iter(iter: impl Iterator<Item = FileFormat>) -> FileFormat {
    iter.fold(FileFormat::Unknown, merge_file_formats)
}

#[cfg_attr(
    any(feature = "debug-inputs", feature = "debug-formats"),
    instrument(ret, skip(registry))
)]
pub fn format_for_texture_file(file: &Utf8Path) -> FileFormat {
    let registry = registry();

    if let Some(format) = registry.get_override(file) {
        FileFormat::MetaOverride(format)
    } else if !file.exists() {
        FileFormat::Unknown
    } else if let Ok((Some(format), _)) = texture_file::read_header(file).log_failure() {
        FileFormat::FromHeader(format.into())
    } else if let Ok(len) = std::fs::metadata(file).map(|m| m.len() as usize) {
        FileFormat::FromSize(registry.formats_with_size(len))
    } else {
        FileFormat::Unknown
    }
}

#[cfg_attr(feature = "debug-inputs", instrument(ret))]
pub fn format_for_image_file(file: &Utf8Path) -> FileFormat {
    let file = Utf8PathBuf::from(base_name(file));

    try_read_meta(&file)
        .map(FileFormat::MetaOverride)
        .unwrap_or_else(|| format_for_texture_file(&file.with_extension("texture")))
}

#[cfg_attr(feature = "debug-inputs", instrument(ret))]
fn try_metafiles(file: &Utf8Path) -> Option<Utf8PathBuf> {
    let meta = file.with_extension("json");
    if meta.exists() {
        return Some(meta);
    }

    let meta = file.with_file_name(base_name(file)).with_extension("json");
    (meta.exists()).then_some(meta)
}

pub fn try_read_meta(file: &Utf8Path) -> Option<TextureFormat> {
    let metafile = try_metafiles(file)?;

    let meta = std::fs::read(file)
        .log_failure_with(|| format!("Failed to read meta file {metafile}"))
        .ok()?;
    let format: TextureFormat = serde_json::from_slice(&meta)
        .log_failure_with(|| format!("Failed to read meta file {metafile}"))
        .ok()?;

    Some(format)
}

pub fn format_for_file<'r>(file: &Utf8Path) -> FileFormat {
    if let Some(ext) = file.extension() {
        if is_texture_ext(ext) {
            format_for_texture_file(file)
        } else if is_image_ext(ext) {
            format_for_image_file(file)
        } else {
            unreachable!()
        }
    } else {
        FileFormat::Unknown
    }
}

#[inline]
#[must_use]
pub fn is_image_ext(ext: &str) -> bool { SUPPORTED_IMAGE_EXTENSIONS.contains(&ext) }

#[inline]
#[must_use]
pub fn is_ignored_ext(ext: &str) -> bool {
    ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("log")
}

#[inline]
#[must_use]
pub fn is_texture_ext(ext: &str) -> bool { SUPPORTED_TEXTURE_EXTENSIONS.contains(&ext) }
