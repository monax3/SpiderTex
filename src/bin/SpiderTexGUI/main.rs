//! TODO: use bytemuck
//! TODO: warn on wrong aspect ratio
//! TODO: test many more textures including all controller buttons
//! TODO: Catch panics in release mode
//! TODO: Is it srgb? is it premultiplied alpha?

#![allow(non_snake_case)]

use spidertexlib::ui;

use color_eyre::{Result, eyre::eyre};
use camino::{Utf8PathBuf, Utf8Path};

pub mod import;
pub mod export;
pub mod theme;
pub mod preview;
pub mod log;
pub mod util;
pub mod widgets;

const APP_TITLE: &str = "Spider-Man Texture Converter";
const ARRAY_SEP: char = '#';

#[derive(Copy, Clone, PartialEq, Eq)]
enum FileTypes {
    None,
    Texture,
    Image,
    Mixed,
}

fn base_name(path: &Utf8Path) -> Option<&str> {
    let file_name = path.file_stem()?;

    Some(if let Some(sep) = file_name.rfind(|c| c == ARRAY_SEP) {
        &file_name[ .. sep]
    } else if let Some(sep) = file_name.rfind("_hd") {
        &file_name[ .. sep]
    } else {
        file_name
    })
}

fn common_name(paths: &[Utf8PathBuf]) -> Option<&str> {
    let mut iter = paths.iter();
    let base = iter.next().and_then(|f| base_name(f));

    for path in iter {
        if base_name(path) != base {
            return None;
        }
    }

    base
}

fn file_types(paths: &[Utf8PathBuf]) -> FileTypes {
    let mut file_types = FileTypes::None;

    for path in paths {
        let next = match path.extension() {
            Some(ext) if ext.eq_ignore_ascii_case("png") || ext.eq_ignore_ascii_case("tga") => FileTypes::Image,
            Some(ext) if ext.eq_ignore_ascii_case("raw") || ext.eq_ignore_ascii_case("texture") => FileTypes::Texture,
            Some(_) | None => FileTypes::None,
        };

        match (file_types, next) {
            (FileTypes::None | FileTypes::Texture, FileTypes::Texture) |
            (FileTypes::None | FileTypes::Image, FileTypes::Image) => { file_types = next },
            _ => { return FileTypes::Mixed }
        }
    }

    file_types
}

fn run() -> Result<()> {
    const INVALID_FILE: &str = "Please see the documentation for how to use this program.";

    log::init();

    // std::thread::spawn(|| {
    //     let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/headers.rs"));

    //     for (i, line) in src.lines().enumerate() {
    //         std::thread::sleep(std::time::Duration::from_millis(100));

    //         tracing::trace!("{:<4} {line}", i+1);
    //     }
    // });

    let input_files: Vec<Utf8PathBuf> = std::env::args().skip(1).map(Utf8PathBuf::from).collect();

    // FIXME
    // let common_name = base_name(input_files.first().unwrap()).unwrap().to_string();
    let common_name = match common_name(&input_files) {
        Some(n) => n.to_string(),
        None => return Err(eyre!(INVALID_FILE)),
    };

    match file_types(&input_files) {
        FileTypes::None => Err(eyre!(INVALID_FILE)),
        FileTypes::Texture => export::export_ui(input_files, common_name),
        FileTypes::Image => import::import_ui(input_files, common_name),
        FileTypes::Mixed => Err(eyre!(INVALID_FILE)),
    }
}

fn main() {
    let _ignore = color_eyre::install();

    match run() {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            let error = format!("{error}");
            ui::message_box_error(APP_TITLE, &error);
            std::process::exit(1);
        }
    }
}
