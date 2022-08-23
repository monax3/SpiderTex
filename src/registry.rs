use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{Debug, Display};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::formats::{Source, TextureFormat};
use crate::prelude::*;
use crate::util::{current_dir_utf8, exe_dir_utf8, MaybeReady};

static REGISTRY: MaybeReady<Registry> = MaybeReady::new();

pub fn load() -> Result<()> {
    if !REGISTRY.is_ready() {
        let registry = Registry::load()?;

        REGISTRY.ready(registry);
    }

    Ok(())
}

#[inline]
#[must_use]
pub fn registry() -> &'static Registry { REGISTRY.get() }

pub trait FormatRef: AsRef<FormatId> + Debug {}
impl<T> FormatRef for T where T: AsRef<FormatId> + Debug {}

#[inline]
#[must_use]
pub fn get(id: impl FormatRef) -> &'static TextureFormat { registry().get(id) }

#[inline]
#[must_use]
pub fn get_all(ids: &[impl FormatRef]) -> Vec<&'static TextureFormat> { registry().get_all(ids) }

#[inline]
#[must_use]
pub fn raw_header(id: impl FormatRef) -> Option<String> { registry().raw_header(id) }

#[must_use]
pub fn formats_for_size(size: usize) -> Vec<&'static TextureFormat> {
    if let Some(formats) = registry().lengths.get(&size) {
        formats.iter().map(get).collect()
    } else {
        Vec::new()
    }
}

#[must_use]
pub fn formats_for_sizes(sizes: &[usize]) -> Vec<&'static TextureFormat> {
    let registry = registry();

    let formats: HashSet<&TextureFormat> = sizes
        .iter()
        .filter_map(|size| registry.lengths.get(size))
        .flatten()
        .map(get)
        .collect();

    formats.into_iter().collect()
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Registry {
    #[serde(default)]
    pub formats:     BTreeMap<FormatId, TextureFormat>,
    #[serde(default)]
    pub lengths:     BTreeMap<usize, BTreeSet<FormatId>>,
    #[serde(default)]
    pub overrides:   Vec<(String, FormatId)>,
    #[serde(default)]
    pub raw_headers: BTreeMap<FormatId, String>,
    #[serde(default)]
    pub examples:    BTreeMap<FormatId, String>,
}

impl Registry {
    #[cfg(not(feature = "rebuild-registry"))]
    const EMBEDDED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/formats.json"));
    const REGISTRY_FILE: &str = "formats.json";

    #[inline]
    #[must_use]
    pub fn formats_with_size_iter(
        &self,
        size: usize,
    ) -> Option<impl IntoIterator<Item = &FormatId>> {
        self.lengths.get(&size)
    }

    #[inline]
    #[must_use]
    pub fn formats_with_size(&self, size: usize) -> Vec<FormatId> {
        self.lengths
            .get(&size)
            .map(|fmts| fmts.iter().copied().collect())
            .unwrap_or_default()
    }

    #[inline]
    #[must_use]
    pub fn known(&self, id: impl FormatRef) -> bool { self.formats.contains_key(id.as_ref()) }

    #[inline]
    #[must_use]
    pub fn get(&self, id: impl FormatRef) -> &TextureFormat {
        self.formats
            .get(id.as_ref())
            .unwrap_or_else(|| panic!("Failed to resolve format {id:?}"))
    }

    #[inline]
    #[must_use]
    pub fn try_get(&self, id: FormatId) -> Option<&TextureFormat> { self.formats.get(&id) }

    #[inline]
    #[must_use]
    pub fn raw_header(&self, id: impl FormatRef) -> Option<String> {
        self.raw_headers.get(id.as_ref()).map(Clone::clone)
    }

    #[inline]
    #[must_use]
    pub fn get_all<'r, ID>(&'r self, ids: impl IntoIterator<Item = ID>) -> Vec<&'r TextureFormat>
    where ID: FormatRef {
        ids.into_iter().map(|id| self.get(id)).collect()
    }

    #[cfg(feature = "rebuild-registry")]
    #[inline]
    pub fn load() -> Result<Self> { Ok(Self::default()) }

    #[must_use]
    pub fn get_override(&self, file: &Utf8Path) -> Option<TextureFormat> {
        let size = std::fs::metadata(file).ok()?.len() as usize;
        let file_stem = file.file_stem()?;

        if file_stem.ends_with("_g") {
            event!(DEBUG, ?file_stem, ?size);
        }

        let format = if file_stem.ends_with("_g") && size == 327_680 {
            Some(registry::get(FormatId(0x800_aa62)))
        } else {
            None
        };

        let format = *format?;
        Some(TextureFormat {
            source: Source::FromFilename,
            ..format
        })
    }

    pub fn make_ref(&mut self, format: TextureFormat) -> &TextureFormat {
        let id = format.id();
        if !self.known(id) {
            self.update_format(format, None::<Utf8PathBuf>);
        }
        self.get(id)
    }

    #[cfg(not(feature = "rebuild-registry"))]
    pub fn load() -> Result<Self> {
        let mut registry: Self = serde_json::from_str(Self::EMBEDDED).log_failure()?;

        if let Some(dir) = current_dir_utf8() {
            registry.try_extend_from_dir(&dir);
        }
        if let Some(dir) = exe_dir_utf8() {
            registry.try_extend_from_dir(&dir);
        }

        Ok(registry)
    }

    fn try_extend_from_dir(&mut self, dir: &Utf8Path) {
        if let Some(Ok(extra)) = try_format_file_name(dir).map(load_format_file) {
            self.extend(extra);
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.formats.extend(other.formats);

        for (len, ids) in other.lengths {
            self.lengths.entry(len).or_default().extend(ids);
        }
    }

    pub fn update_length(&mut self, length: usize, id: FormatId) {
        self.lengths.entry(length).or_default().insert(id);
    }

    pub fn update_header(&mut self, header: &texture_file::FormatHeader) {
        let id = TextureFormat::from(header).id();
        self.raw_headers.insert(id, header.as_hexstring());
    }

    pub fn replace_format(&mut self, format: impl Into<TextureFormat>) {
        let format: TextureFormat = format.into();
        let id = format.id();
        self.formats.insert(id, format);
    }

    pub fn update_format(
        &mut self,
        format: impl Into<TextureFormat>,
        example_file: Option<impl Into<Utf8PathBuf>>,
    ) -> FormatId {
        let mut format: TextureFormat = format.into();

        texture_file::texture_format_overrides(&mut format);

        let id = format.id();

        self.update_length(format.sd_file_len(), id);
        if let Some(hd_len) = format.hd_len() {
            self.update_length(hd_len, id);
        }

        if self.known(id) {
            #[cfg(feature = "debug-imports")]
            event!(TRACE, ?id, ?format, "Format already known");
        } else {
            #[cfg(feature = "debug-imports")]
            event!(TRACE, ?id, ?format, "Inserting format");
            self.formats.insert(id, format);
            if let Some(example_file) =
                example_file.and_then(|f| f.into().file_name().map(ToOwned::to_owned))
            {
                self.examples.insert(id, example_file);
            }
        }

        id
    }

    // TODO: add a variant to save specific formats only
    pub fn save(&mut self) -> Result<()> {
        let file = Utf8Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/formats.json"));
        let json =
            serde_json::to_string_pretty(self).log_failure_as("Failed to serialize registry")?;
        std::fs::write(file, &json).log_failure_as("Failed to save registry")?;

        Ok(())
    }
}

fn load_format_file(file: impl AsRef<Utf8Path>) -> Result<Registry> {
    let fmt = std::fs::read(file.as_ref())?;
    Ok(serde_json::from_slice(&fmt).log_failure()?)
}

fn try_format_file_name(dir: impl AsRef<Utf8Path>) -> Option<Utf8PathBuf> {
    let file = dir.as_ref().join(Registry::REGISTRY_FILE);

    if file.exists() {
        Some(file)
    } else {
        event!(INFO, "Optional registry {file} not found");
        None
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
pub struct FormatId(u64);

impl AsRef<Self> for FormatId {
    fn as_ref(&self) -> &Self { self }
}

impl Display for FormatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FormatId({:016X})", self.0)
    }
}

impl Serialize for FormatId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&format!("{:09x}", self.0))
    }
}

impl<'de> Deserialize<'de> for FormatId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        <&str>::deserialize(deserializer).and_then(|s| {
            u64::from_str_radix(s, 16)
                .map_err(serde::de::Error::custom)
                .map(FormatId)
        })
    }
}

impl From<&TextureFormat> for FormatId {
    #[cfg_attr(feature = "debug-formats", instrument(ret))]
    fn from(format: &TextureFormat) -> Self {
        const FORMAT_BITS: u32 = 7;
        const DIM_BITS: u32 = 14;
        const SIZE_BITS: u32 = 40;

        /* 0.. 6 */
        let mut hash: u64 = u64::from(format.dxgi_format.0);
        /* 7..20 */
        hash |= (format.standard.width as u64) << FORMAT_BITS;
        /* 21..34 */
        hash |= (format.standard.height as u64) << (FORMAT_BITS + DIM_BITS);
        /* 35..64 */
        hash |= (format.standard.data_size as u64).reverse_bits() >> SIZE_BITS;

        Self(hash)

        // Self(
        //     (u64::from(format.dxgi_format.0)) << u32::BITS
        //         | u64::from(crc32fast::hash(format.raw_headers.as_ref())),
        // )
    }
}
