#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
#![deny(clippy::unwrap_used)]

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Result;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

pub mod dxtex;
pub mod headers;
pub mod ui;

use dxtex::{compress_texture, decompress_texture, DXBuf};
use headers::{read_texture_header, TEXTURE_HEADER_SIZE};

const APP_TITLE: &str = "Spider-Man Texture Converter";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureInfo {
    data_len:            u32,
    hd_len:              u32,
    width:               u16,
    height:              u16,
    hd_width:            u16,
    hd_height:           u16,
    compressed_format:   u32,
    uncompressed_format: u32,
    mipmaps:             u8,
    hd_mipmaps:          u8,
    raw_headers:         String,
}

impl TextureInfo {
    fn has_hd(&self) -> bool { self.width != self.hd_width }
}

fn read_metadata_from_json(input_file: &Path) -> Result<TextureInfo> {
    let metadata_file = input_file.with_extension("metadata.json");
    let metadata_str = std::fs::read_to_string(&metadata_file)?;
    let metadata = serde_json::from_str(&metadata_str)?;

    Ok(metadata)
}

fn read_metadata_from_texture(input_file: &Path) -> Result<TextureInfo> {
    let texture_file = input_file.with_extension("texture");
    let (metadata, _) = read_texture_header(&texture_file)?;

    Ok(metadata)
}

fn read_metadata(input_file: &Path) -> Result<TextureInfo> {
    if let Ok(metadata) = read_metadata_from_json(input_file) {
        Ok(metadata)
    } else if let Ok(metadata) = read_metadata_from_texture(input_file) {
        Ok(metadata)
    } else {
        Err(eyre!(
            "Converting a PNG to texture needs either the original texture or the .metadata.json \
             file from a previous conversion"
        ))
    }
}

fn get_sd_texture(metadata: &TextureInfo, image: &DynamicImage) -> Result<DXBuf> {
    let (width, height) = (metadata.width as u32, metadata.height as u32);

    if metadata.has_hd() {
        let sd_image = image.resize(width, height, image::imageops::FilterType::CatmullRom);
        compress_texture(
            metadata.compressed_format,
            width,
            height,
            metadata.mipmaps,
            sd_image.as_bytes(),
        )
        .map_err(|code| eyre!("Texture conversion failed with code {code}"))
    } else {
        if image.width() != width || image.height() != height {
            return Err(eyre!(
                "Incorrect image dimensions, expected {}x{} got {}x{}",
                width,
                height,
                image.width(),
                image.height()
            ));
        }

        compress_texture(
            metadata.compressed_format,
            width,
            height,
            metadata.mipmaps,
            image.as_bytes(),
        )
        .map_err(|code| eyre!("Texture conversion failed with code {code}"))
    }
}

fn get_hd_texture(metadata: &TextureInfo, image: &DynamicImage) -> Result<DXBuf> {
    let (width, height) = (metadata.hd_width as u32, metadata.hd_height as u32);

    if image.width() != width || image.height() != height {
        return Err(eyre!(
            "Incorrect image dimensions, expected {}x{} got {}x{}",
            width,
            height,
            image.width(),
            image.height()
        ));
    }

    compress_texture(
        metadata.compressed_format,
        width,
        height,
        metadata.hd_mipmaps,
        image.as_bytes(),
    )
    .map_err(|code| eyre!("Texture conversion failed with code {code}"))
}

fn convert_png_to_texture(png_file: &Path) -> Result<()> {
    let metadata = read_metadata(png_file)?;

    let image: DynamicImage =
        image::open(png_file).wrap_err_with(|| eyre!("Failed to open {}", png_file.display()))?.into_rgba8().into();

    let texture_file = png_file.with_extension("custom.texture");

    let hd_file_name = {
        let mut hd_file_name = png_file.file_stem().expect("Internal error").to_owned();
        hd_file_name.push("_hd.custom.texture");
        hd_file_name
    };

    let hd_file = texture_file.with_file_name(hd_file_name);

    if metadata.has_hd() {
        let hd_data = get_hd_texture(&metadata, &image)?;

        std::fs::write(&hd_file, hd_data.as_slice())?;
    }

    let texture_data = get_sd_texture(&metadata, &image)?;

    let mut writer = File::create(&texture_file)?;

    let headers = base64::decode(&metadata.raw_headers)?;

    writer.write_all(&headers)?;
    writer.write_all(texture_data.as_slice())?;

    let pos = writer.stream_position()?;

    let message = if metadata.has_hd() {
        format!(
            "{}\r\n\r\nconverted to\r\n\r\n{}\r\n\r\nand\r\n\r\n{}",
            png_file.display(),
            texture_file.display(),
            hd_file.display()
        )
    } else {
        format!(
            "{}\r\n\r\nconverted to\r\n\r\n{}",
            png_file.display(),
            texture_file.display()
        )
    };

    let correct_size = pos == (TEXTURE_HEADER_SIZE + metadata.data_len as usize) as u64
        && (!metadata.has_hd() || hd_file.metadata()?.len() as u32 == metadata.hd_len);

    if correct_size {
        ui::message_box_ok(APP_TITLE, &message);
    } else {
        let message = format!(
            "{}\r\n\r\nThe file is the wrong size, which means something failed along the way.",
            message
        );
        ui::message_box_ok(APP_TITLE, &message);
    }

    Ok(())
}

fn convert_texture_to_png(texture_file: &Path) -> Result<()> {
    let (texture_info, mut reader) = read_texture_header(texture_file)?;

    let hd_file_name = {
        let mut hd_file_name = texture_file.file_stem().expect("Internal error").to_owned();
        hd_file_name.push("_hd.texture");
        hd_file_name
    };

    let hd_file = texture_file.with_file_name(hd_file_name);

    if texture_info.has_hd() && !hd_file.is_file() {
        return Err(eyre!(
            "This texture is a high-resolution texture. To extract it, you need the original \
             texture, which you have:\r\n\r\n{}\r\n\r\nBut you also need the high-resolution \
             texture file from the 14, 15 or 16 archive, renamed as follows:\r\n\r\n{}",
            texture_file.display(),
            hd_file.display()
        ));
    }

    let metadata_file = texture_file.with_extension("metadata.json");
    let metadata = serde_json::to_string_pretty(&texture_info)?;
    std::fs::write(&metadata_file, &metadata)?;

    let (width, height, mipmaps, texture_data) = if texture_info.has_hd() {
        (
            texture_info.hd_width,
            texture_info.hd_height,
            texture_info.hd_mipmaps,
            std::fs::read(hd_file)?,
        )
    } else {
        let mut buf = Vec::with_capacity(texture_info.data_len as usize);
        reader.read_to_end(&mut buf)?;
        (
            texture_info.width,
            texture_info.height,
            texture_info.mipmaps,
            buf,
        )
    };

    let png_file = texture_file.with_extension("png");

    let decompressed = decompress_texture(
        texture_info.compressed_format,
        width as u32,
        height as u32,
        mipmaps,
        &texture_data,
    )
    .map_err(|code| eyre!("Decompressing texture failed with code {code}"))?;

    eprintln!("Uncompressed texture data is {} bytes", decompressed.len());

    let img: image::ImageBuffer<image::Rgba<u8>, _> =
        image::ImageBuffer::from_raw(width as u32, height as u32, decompressed.as_slice())
            .ok_or_else(|| eyre!("PNG creation failed"))?;

    img.save(&png_file)?;

    let message = format!(
        "{}\r\n\r\nconverted to\r\n\r\n{}",
        texture_file.display(),
        png_file.display()
    );
    ui::message_box_ok(APP_TITLE, &message);

    Ok(())
}

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
