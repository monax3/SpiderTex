#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
#![deny(clippy::unwrap_used)]

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use camino::{Utf8PathBuf, Utf8Path};
use std::borrow::Cow;

use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Result;
use formats::{Dimensions, TextureFormat};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug, warn, trace};

pub mod dxtex;
pub mod headers;
pub mod ui;
pub mod formats;

use dxtex::{compress_texture, decompress_texture, DXBuf};
use headers::{read_texture_header, TEXTURE_HEADER_SIZE};

use crate::dxtex::{expected_size, expected_size3};

const APP_TITLE: &str = "Spider-Man Texture Converter";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureInfo {
    data_len:          u32,
    hd_len:            u32,
    width:             u16,
    height:            u16,
    hd_width:          u16,
    hd_height:         u16,
    array_size:        u16,
    compressed_format: u32,
    mipmaps:           u8,
    hd_mipmaps:        u8,
    raw_headers:       String,
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
            width as usize,
            height as usize,
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
            width as usize,
            height as usize,
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
        width as usize,
        height as usize,
        metadata.hd_mipmaps,
        image.as_bytes(),
    )
    .map_err(|code| eyre!("Texture conversion failed with code {code}"))
}

pub fn convert_png_to_texture(png_file: &Path) -> Result<()> {
    let metadata = read_metadata(png_file)?;

    let image: DynamicImage = image::open(png_file)
        .wrap_err_with(|| eyre!("Failed to open {}", png_file.display()))?
        .into_rgba8()
        .into();

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

pub fn convert_texture_to_png(texture_file: &Path) -> Result<()> {
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

    println!(
        "expected SD sizes are {:?}, I have {}",
        expected_size3(
            texture_info.compressed_format,
            texture_info.width as u32,
            texture_info.height as u32,
            texture_info.array_size as u32,
            texture_info.mipmaps
        ),
        texture_info.data_len
    );
    println!(
        "expected HD size are {:?}, I have {}",
        expected_size3(
            texture_info.compressed_format,
            width as u32,
            height as u32,
            texture_info.array_size as u32,
            texture_info.hd_mipmaps
        ),
        texture_data.len()
    );

    find_size(
        &texture_info,
        texture_info.data_len as usize,
        texture_data.len(),
    );

    let decompressed = decompress_texture(
        texture_info.compressed_format,
        width as usize,
        height as usize,
        texture_info.array_size as usize,
        mipmaps,
        &texture_data,
    )
    .map_err(|code| eyre!("Decompressing texture failed with code {code}"))?;

    eprintln!("Uncompressed texture data is {} bytes, expected {}", decompressed.len(), (width as u32) * (height as u32) * 4);

    #[allow(clippy::unwrap_used)]
    if texture_info.array_size > 1 {
        for (i, data) in decompressed
            .as_slices(texture_info.array_size as usize)
            .iter()
            .enumerate()
        {
            let mut file_name = png_file.file_stem().unwrap().to_owned();
            file_name.push(format!("_image{i}.png"));
            let png_file = png_file.with_file_name(file_name);

            let img: image::ImageBuffer<image::Rgba<u8>, _> =
            image::ImageBuffer::from_raw(width as u32, height as u32, *data)
                .ok_or_else(|| eyre!("PNG creation failed"))?;

        img.save(&png_file)?;
        }
    } else {
        // let img: image::ImageBuffer<image::Luma<u8>, _> =
        //     image::ImageBuffer::from_raw(width as u32, height as u32, decompressed.as_slice())
        //         .ok_or_else(|| eyre!("PNG creation failed"))?;

        let buf = decompressed.as_slice().to_vec();

        let mut img: image::ImageBuffer<image::Rgba<u8>, _> =
            image::ImageBuffer::from_raw(width as u32, height as u32, buf)
                .ok_or_else(|| eyre!("PNG creation failed"))?;

        // FIXME: swapping colors
        for pixel in img.pixels_mut() {
            let r = pixel[0];
            pixel[0] = pixel[2];
            pixel[2] = r;
        }

        img.save(&png_file)?;
    }

    let message = format!(
        "{}\r\n\r\nconverted to\r\n\r\n{}",
        texture_file.display(),
        png_file.display()
    );
    ui::message_box_ok(APP_TITLE, &message);

    Ok(())
}

fn find_size(metadata: &TextureInfo, have_2d: usize, have_3d: usize) {
    for i in 0 .. 10 {
        let sizes = expected_size3(
            metadata.compressed_format,
            metadata.width as _,
            metadata.height as _,
            metadata.array_size as _,
            i,
        );
        println!("SD {i}: {have_2d} {sizes:?}");
        let sizes = expected_size3(
            metadata.compressed_format,
            metadata.hd_width as _,
            metadata.hd_height as _,
            metadata.array_size as _,
            i,
        );
        println!("HD {i}: {have_3d} {sizes:?}");
    }
}

fn generic_failure<T>() -> Result<T> {
    Err(eyre!("Failed."))
}

pub fn convert_to_texture(format: &TextureFormat, images: &[DynamicImage], output_files: [Utf8PathBuf; 2]) -> Result<()> {
    let [sd_file, hd_file] = output_files;

    if images.len() != format.array_size {
        error!("This format uses {} images and {} were supplied", format.array_size, images.len());
        return generic_failure();
    }

    let sd_texture = convert_images_to_texture(format, images, format.standard)?;
    save_raw(&sd_file, &sd_texture)?;

    if let Some(highres) = format.highres {
        let hd_texture = convert_images_to_texture(format, images, highres)?;
        save_raw(&hd_file, &hd_texture)?;
    }

    info!("Done");

    Ok(())
}

pub fn save_with_header(format: &TextureFormat, file: &Utf8Path, data: &[u8]) -> Result<()> {
    use headers::{TextureFileHeader, TextureHeader, TextureFormatHeader};

    let headers = (TextureFileHeader::with_length(data.len()), TextureHeader::new(), format.to_header());

    todo!()
}

pub fn save_raw(file: &Utf8Path, data: &[u8]) -> Result<()> {
    std::fs::write(file, data)?;

    info!("Saved raw texture to {file}");

    Ok(())
}

pub fn convert_images_to_texture(format: &TextureFormat, images: &[DynamicImage], dimensions: Dimensions) -> Result<Vec<u8>> {
    let mut vec = Vec::<u8>::with_capacity(dimensions.data_size);

    for (i, image) in images.iter().enumerate() {
        if images.len() > 1 {
            info!("Converting image {} to {format}", i+1);
        } else {
            info!("Converting image to {format}");
        }

        trace!("Resizing to {}x{}", dimensions.width, dimensions.height);
        let resized = resize_image(image, dimensions);

        trace!("Calling DirectXTex");
        let buf = compress_texture(format.format, dimensions.width, dimensions.height, dimensions.mipmaps, resized.as_bytes()).map_err(|code| eyre!("Error code {code}"))?;
        vec.extend(buf.as_slice());
    }

    if vec.len() != dimensions.data_size {
        warn!("The converted data is the wrong size (expected {}, got {})", dimensions.data_size, vec.len());
    } else {
        info!("Successfully converted the texture");
    }
    Ok(vec)
}

pub fn resize_image(image: &DynamicImage, dimensions: Dimensions) -> Cow<'_, DynamicImage> {
    if image.width() as usize != dimensions.width || image.height() as usize != dimensions.height {
        Cow::Owned(image.resize_exact(dimensions.width as u32, dimensions.height as u32, image::imageops::Lanczos3))
    } else {
        Cow::Borrowed(image)
    }
}
