use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::*;

use camino::{Utf8Path, Utf8PathBuf};
use image::{DynamicImage, ImageFormat};

pub mod dxtex;
mod error;
pub mod formats;
pub mod registry;
pub mod util;
use dxtex::{compress_texture, decompress_texture};
pub mod convert;
pub mod files;
pub mod images;
pub mod inputs;
pub mod rgb;
pub mod texture_file;

pub const APP_TITLE: &str = concat!("Spider-Man Texture Converter v", env!("CARGO_PKG_VERSION"));

#[cfg(doc)]
#[doc(inline)]
pub use std;

pub const SUPPORTED_TEXTURE_EXTENSIONS: &[&str] = &["texture", "raw"];
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &["png", "tga", "dds", "hdr", "exr"];
pub const META_EXTENSION: &str = "json";
pub const IMAGERS_RESIZE_FILTER: image::imageops::FilterType = image::imageops::Lanczos3;
pub const DEFAULT_IMAGE_FORMAT: ImageFormat = ImageFormat::Png;

pub mod prelude {
    pub use tracing::{event, instrument, span};
    pub const ERROR: tracing::Level = tracing::Level::ERROR;
    pub const WARN: tracing::Level = tracing::Level::WARN;
    pub const INFO: tracing::Level = tracing::Level::INFO;
    pub const DEBUG: tracing::Level = tracing::Level::DEBUG;
    pub const TRACE: tracing::Level = tracing::Level::TRACE;

    pub use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

    pub use crate::error::{error_message, Error, LogFailure, Result};
    pub use crate::formats::{Dimensions, DxgiFormatExt, ImageFormatExt, TextureFormat};
    pub use crate::registry::{self, registry, FormatId, Registry};
    pub use crate::texture_file::{self, TEXTURE_HEADER_SIZE};
    pub use crate::{dxtex, SUPPORTED_IMAGE_EXTENSIONS, SUPPORTED_TEXTURE_EXTENSIONS};
}

use prelude::*;

use crate::dxtex::{expected_size, expected_size_array};

fn read_metadata_from_json(input_file: &Utf8Path) -> Result<TextureFormat> {
    let metadata_file = input_file.with_extension("metadata.json");
    let metadata_str = std::fs::read_to_string(&metadata_file)?;
    let metadata = serde_json::from_str(&metadata_str)?;

    Ok(metadata)
}

fn read_metadata_from_texture(input_file: &Utf8Path) -> Result<Option<TextureFormat>> {
    let texture_file = input_file.with_extension("texture");
    let (metadata, _) = texture_file::read_header(&texture_file)?;

    Ok(metadata.map(Into::into))
}

fn read_metadata(input_file: &Utf8Path) -> Result<TextureFormat> {
    if let Ok(metadata) = read_metadata_from_json(input_file) {
        Ok(metadata)
    } else if let Ok(Some(metadata)) = read_metadata_from_texture(input_file) {
        Ok(metadata)
    } else {
        Err(Error::message(
            "Converting a PNG to texture needs either the original texture or the .metadata.json \
             file from a previous conversion",
        ))
    }
}

fn get_sd_texture(metadata: &TextureFormat, image: &DynamicImage) -> Result<Vec<u8>> {
    let (width, height) = (
        metadata.standard.width as u32,
        metadata.standard.height as u32,
    );

    if metadata.has_highres() {
        let sd_image = image.resize(width, height, image::imageops::FilterType::CatmullRom);
        compress_texture(
            metadata.dxgi_format,
            width as usize,
            height as usize,
            metadata.array_size,
            metadata.standard.mipmaps,
            sd_image.as_bytes(),
        )
        .map_err(|code| Error::message(format!("Texture conversion failed with code {code}")))
    } else {
        if image.width() != width || image.height() != height {
            return Err(Error::message(format!(
                "Incorrect image dimensions, expected {}x{} got {}x{}",
                width,
                height,
                image.width(),
                image.height()
            )));
        }

        compress_texture(
            metadata.dxgi_format,
            width as usize,
            height as usize,
            metadata.array_size,
            metadata.standard.mipmaps,
            image.as_bytes(),
        )
        .map_err(|code| Error::message(format!("Texture conversion failed with code {code}")))
    }
}

fn get_hd_texture(metadata: &TextureFormat, image: &DynamicImage) -> Result<Vec<u8>> {
    let highres = metadata.highres.unwrap();

    let (width, height) = (highres.width as u32, highres.height as u32);

    if image.width() != width || image.height() != height {
        return Err(Error::message(format!(
            "Incorrect image dimensions, expected {}x{} got {}x{}",
            width,
            height,
            image.width(),
            image.height()
        )));
    }

    compress_texture(
        metadata.dxgi_format,
        width as usize,
        height as usize,
        metadata.array_size,
        highres.mipmaps,
        image.as_bytes(),
    )
    .map_err(|code| Error::message(format!("Texture conversion failed with code {code}")))
}

pub fn convert_png_to_texture(png_file: &Utf8Path) -> Result<()> {
    let metadata = read_metadata(png_file)?;

    let image: DynamicImage = image::open(png_file)?.into_rgba8().into();

    let texture_file = png_file.with_extension("custom.texture");

    let hd_file_name = {
        let mut hd_file_name = png_file.file_stem().expect("Internal error").to_owned();
        hd_file_name.push_str("_hd.custom.texture");
        hd_file_name
    };

    let hd_file = texture_file.with_file_name(hd_file_name);

    if metadata.has_highres() {
        let hd_data = get_hd_texture(&metadata, &image)?;

        std::fs::write(&hd_file, hd_data.as_slice())?;
    }

    let texture_data = get_sd_texture(&metadata, &image)?;

    let mut writer = File::create(&texture_file)?;

    let headers: Vec<u8> = unimplemented!(); // hex::decode(&metadata.raw_headers)?;

    writer.write_all(&headers)?;
    writer.write_all(texture_data.as_slice())?;

    let pos = writer.stream_position()?;

    let message = if metadata.has_highres() {
        format!(
            "{}\r\n\r\nconverted to\r\n\r\n{}\r\n\r\nand\r\n\r\n{}",
            png_file, texture_file, hd_file
        )
    } else {
        format!("{}\r\n\r\nconverted to\r\n\r\n{}", png_file, texture_file)
    };

    let correct_size = pos == (TEXTURE_HEADER_SIZE + metadata.standard.data_size) as u64
        && metadata.highres.map_or(true, |highres| {
            hd_file.metadata().map_or(0, |m| m.len() as usize) == highres.data_size
        });

    if correct_size {
        event!(INFO, message);
    } else {
        let message = format!(
            "{}\r\n\r\nThe file is the wrong size, which means something failed along the way.",
            message
        );
        event!(INFO, message);
    }

    Ok(())
}

pub fn convert_texture_to_png(texture_file: &Utf8Path) -> Result<()> {
    let (texture_info, mut reader) = texture_file::read_header(texture_file)?;
    let texture_info: TextureFormat = texture_info.unwrap().into();

    let hd_file_name = {
        let mut hd_file_name = texture_file.file_stem().expect("Internal error").to_owned();
        hd_file_name.push_str("_hd.texture");
        hd_file_name
    };

    let hd_file = texture_file.with_file_name(hd_file_name);

    if texture_info.has_highres() && !hd_file.is_file() {
        return Err(Error::message(format!(
            "This texture is a high-resolution texture. To extract it, you need the original \
             texture, which you have:\r\n\r\n{}\r\n\r\nBut you also need the high-resolution \
             texture file from the 14, 15 or 16 archive, renamed as follows:\r\n\r\n{}",
            texture_file, hd_file
        )));
    }

    let metadata_file = texture_file.with_extension("metadata.json");
    let metadata = serde_json::to_string_pretty(&texture_info)?;
    std::fs::write(&metadata_file, &metadata)?;

    let (width, height, mipmaps, texture_data) = if let Some(highres) = texture_info.highres {
        (
            highres.width,
            highres.height,
            highres.mipmaps,
            std::fs::read(hd_file)?,
        )
    } else {
        let mut buf = Vec::with_capacity(texture_info.standard.data_size);
        reader.read_to_end(&mut buf)?;
        (
            texture_info.standard.width,
            texture_info.standard.height,
            texture_info.standard.mipmaps,
            buf,
        )
    };

    let png_file = texture_file.with_extension("png");

    println!(
        "expected SD sizes are {:?}, I have {}",
        (
            expected_size(texture_info.dxgi_format, texture_info.standard, 1),
            expected_size_array(
                texture_info.dxgi_format,
                texture_info.standard,
                texture_info.array_size
            )
        ),
        texture_info.standard.data_size
    );
    if let Some(highres) = texture_info.highres {
        println!(
            "expected HD size are {:?}, I have {}",
            (
                expected_size(texture_info.dxgi_format, highres, 1),
                expected_size_array(texture_info.dxgi_format, highres, texture_info.array_size)
            ),
            texture_data.len()
        );
    }

    find_size(
        &texture_info,
        texture_info.standard.data_size,
        texture_data.len(),
    );

    let decompressed = decompress_texture(
        texture_info.dxgi_format,
        width as usize,
        height as usize,
        texture_info.array_size as usize,
        mipmaps,
        &texture_data,
    )
    .map_err(|code| Error::message(format!("Decompressing texture failed with code {code}")))?;

    eprintln!(
        "Uncompressed texture data is {} bytes, expected {}",
        decompressed.len(),
        (width as u32) * (height as u32) * 4
    );

    #[allow(clippy::unwrap_used)]
    if texture_info.array_size > 1 {
        let slice_len = decompressed.len() / texture_info.array_size;
        assert_eq!(decompressed.len() % texture_info.array_size, 0);

        let mut start = 0;
        let slices: Vec<&[u8]> = (0 .. texture_info.array_size)
            .map(|_| {
                let slice = &decompressed[start .. start + slice_len];
                start += slice_len;
                slice
            })
            .collect();

        for (i, data) in slices.iter().enumerate() {
            let mut file_name = png_file.file_stem().unwrap().to_owned();
            file_name.push_str(&format!("_image{i}.png"));
            let png_file = png_file.with_file_name(file_name);

            let img: image::ImageBuffer<image::Rgba<u8>, _> =
                image::ImageBuffer::from_raw(width as u32, height as u32, *data)
                    .ok_or_else(|| Error::message(format!("PNG creation failed")))?;

            img.save(&png_file)?;
        }
    } else {
        // let img: image::ImageBuffer<image::Luma<u8>, _> =
        //     image::ImageBuffer::from_raw(width as u32, height as u32,
        // decompressed.as_slice())         .ok_or_else(|| eyre!("PNG creation
        // failed"))?;

        let buf = decompressed.as_slice().to_vec();

        let mut img: image::ImageBuffer<image::Rgba<u8>, _> =
            image::ImageBuffer::from_raw(width as u32, height as u32, buf)
                .ok_or_else(|| Error::message(format!("PNG creation failed")))?;

        // FIXME: swapping colors
        for pixel in img.pixels_mut() {
            let r = pixel[0];
            pixel[0] = pixel[2];
            pixel[2] = r;
        }

        img.save(&png_file)?;
    }

    let message = format!("{}\r\n\r\nconverted to\r\n\r\n{}", texture_file, png_file);
    event!(INFO, message);

    Ok(())
}

fn find_size(metadata: &TextureFormat, have_2d: usize, have_3d: usize) {
    for i in 0 .. 10 {
        let sizes = (
            expected_size(metadata.dxgi_format, metadata.standard, 1),
            expected_size_array(metadata.dxgi_format, metadata.standard, i),
        );
        println!("SD {i}: {have_2d} {sizes:?}");
        if let Some(highres) = metadata.highres {
            let sizes = (
                expected_size(metadata.dxgi_format, highres, i),
                expected_size_array(metadata.dxgi_format, highres, i),
            );
            println!("HD {i}: {have_3d} {sizes:?}");
        }
    }
}

fn generic_failure<T>() -> Result<T> { Err(Error::message("Failed.")) }

pub fn convert_to_texture(
    format: &TextureFormat,
    images: &[DynamicImage],
    output_files: [Utf8PathBuf; 2],
) -> Result<()> {
    let [sd_file, hd_file] = output_files;

    if images.len() != format.array_size {
        event!(
            ERROR,
            "This format uses {} images and {} were supplied",
            format.array_size,
            images.len()
        );
        return generic_failure();
    }

    let sd_texture = convert_images_to_texture(format, images, format.standard)?;
    save_raw(&sd_file, &sd_texture)?;

    if let Some(highres) = format.highres {
        let hd_texture = convert_images_to_texture(format, images, highres)?;
        save_raw(&hd_file, &hd_texture)?;
    }

    event!(INFO, "Done");

    Ok(())
}

pub fn save_raw(file: &Utf8Path, data: &[u8]) -> Result<()> {
    std::fs::write(file, data)?;

    event!(INFO, "Saved raw texture to {file}");

    Ok(())
}

pub fn convert_images_to_texture(
    format: &TextureFormat,
    images: &[DynamicImage],
    dimensions: Dimensions,
) -> Result<Vec<u8>> {
    let mut vec = Vec::<u8>::with_capacity(dimensions.data_size);

    for (i, image) in images.iter().enumerate() {
        if images.len() > 1 {
            event!(INFO, "Converting image {} to {format}", i + 1);
        } else {
            event!(INFO, "Converting image to {format}");
        }

        event!(
            TRACE,
            "Resizing to {}x{}",
            dimensions.width,
            dimensions.height
        );
        let resized = resize_image(image, dimensions);

        event!(TRACE, "Calling DirectXTex");
        let buf = compress_texture(
            format.dxgi_format,
            dimensions.width,
            dimensions.height,
            format.array_size,
            dimensions.mipmaps,
            resized.as_bytes(),
        )
        .map_err(|code| Error::message(format!("Error code {code}")))?;
        vec.extend(buf.as_slice());
    }

    if vec.len() != dimensions.data_size {
        event!(
            WARN,
            "The converted data is the wrong size (expected {}, got {})",
            dimensions.data_size,
            vec.len()
        );
    } else {
        event!(INFO, "Successfully converted the texture");
    }
    Ok(vec)
}

pub fn resize_image(image: &DynamicImage, dimensions: Dimensions) -> Cow<'_, DynamicImage> {
    if image.width() as usize != dimensions.width || image.height() as usize != dimensions.height {
        Cow::Owned(image.resize_exact(
            dimensions.width as u32,
            dimensions.height as u32,
            image::imageops::Lanczos3,
        ))
    } else {
        Cow::Borrowed(image)
    }
}
