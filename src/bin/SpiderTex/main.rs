#![allow(non_snake_case)]

use spidertexlib::ui;
use spidertexlib::{convert_png_to_texture, convert_texture_to_png};

use color_eyre::{Result, eyre::eyre};
use std::path::Path;

const APP_TITLE: &str = "Spider-Man Texture Converter";

fn run() -> Result<()> {
    let input_file_str = std::env::args_os()
        .nth(1)
        .ok_or_else(|| eyre!("No input file, drag a .png or .texture to this exe to use it"))?;
    let input_file = Path::new(&input_file_str);

    if input_file
        .file_stem()
        .map_or(false, |stem| stem.to_string_lossy().ends_with("_hd"))
    {
        return Err(eyre!(
            "To convert a high-resolution texture, please drag the original texture onto this \
             program. If no low-resolution version exists, please other methods for now."
        ));
    }

    match input_file.extension() {
        Some(ext) if ext == "png" => convert_png_to_texture(input_file),
        Some(ext) if ext == "texture" => convert_texture_to_png(input_file),
        Some(_) | None => Err(eyre!(
            "Unrecognized extension, input file must be .png or .texture"
        )),
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
