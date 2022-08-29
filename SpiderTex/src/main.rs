// FIXME: make this tool work again too, with panic handler

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use camino::{Utf8Path, Utf8PathBuf};
use directxtex::{DXTImage, TEX_FILTER_FLAGS};
use texturesofspiderman::files::as_textures;
use texturesofspiderman::files::{as_images, Categorized, FileGroup, FileStatus, OutputFormat, Scanned};
use texturesofspiderman::formats::ColorPlanes;
use texturesofspiderman::images::Warnings;
use texturesofspiderman::inputs::Inputs;
use texturesofspiderman::prelude::*;
use texturesofspiderman::util::{log_for_tests};
use texturesofspiderman::{inputs, APP_TITLE};
use std::sync::mpsc;

#[cfg(windows)]
use texturesofspiderman::util::{message_box_error, message_box_ok};

pub mod gui  {
    pub mod noninteractive;
    pub mod widgets {
        pub mod log;
    }
}

#[cfg(not(windows))]
fn message_box_error(text: impl Into<String>, _caption: &str) {
    event!(ERROR, "{}", text.into())
}

#[cfg(not(windows))]
fn message_box_ok(text: impl Into<String>, _caption: &str) {
    event!(INFO, "{}", text.into())
}

#[cfg(windows)]
fn run(mut inputs: Inputs) -> Result<(String, Warnings)> {
    inputs.add_pairs();

    registry::load()?;

    if inputs.is_empty() {
        error_message("No input files selected, drag textures or images to this exe to use it.")
    } else if !inputs.textures.is_empty() && !inputs.images.is_empty() {
        error_message("Both textures and images selected, please pick only one type.")
    } else if !inputs.textures.is_empty() {
        export_textures(inputs.textures)
    } else {
        import_images(inputs.images)
    }
}

fn main() {
    use tracing_subscriber::prelude::*;
    use tracing_egui_repaint::oncecell::repaint_once_cell;

    let (log_reader, log_writer) = tracing_messagevec::new::<String>();
    let (repaint_layer, repaint_handle) =
        repaint_once_cell();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false),
        )
        .with(repaint_layer)
        .with(log_writer)
        .init();

    let (tx, rx) = mpsc::channel();

    gui::noninteractive::show(repaint_handle, rx, log_reader.clone());
}

#[cfg(disabled)] // dev
fn main() {
    log_for_tests(true);

    match run(inputs::gather_from_args()) {
        Ok((mut message, warnings)) => {
            if !warnings.is_empty() {
                message.push('\n');
            }
            for warning in warnings {
                message.push_str(&warning);
                message.push('\n');
            }
            let message = message.replace('\n', "\r\n");
            message_box_ok(&message, APP_TITLE);
            std::process::exit(0);
        }
        Err(error) => {
            let error = format!("{error}").replace('\n', "\r\n");
            message_box_error(&error, APP_TITLE);
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn save_rgb(image: &DXTImage, array_index: usize, file: &Utf8Path) -> Result<()> {
    use windows_imaging::prelude::*;

    // FIXME: create globally
    let wic = windows_imaging::wic()?;
    let metadata = image.metadata()?;

    let vec = image.image(array_index)?;
    let bitmap = windows_imaging::bitmap_from_memory(&wic, WICPixelFormat::RGBA32bpp, metadata.width as u32, metadata.height as u32, (metadata.width as u32) * WICPixelFormat::RGBA32bpp.bpp(), &vec)?;
    let converted = bitmap.convert_to_pixel_format(&wic, WICPixelFormat::BGR24bpp)?;
    Ok(converted.save(&wic, file.as_str(), WICContainer::Png)?)
}

// FIXME: to compile on non-windows, needs image-rs here
#[cfg(not(windows))]
fn save_rgb(image: &DXTImage, array_index: usize, file: &Utf8Path) -> Result<()> {
    unimplemented!()
    // image.save_wic(0, )
    // let wic = windows_imaging::wic()?;
    // // let bitmap = wic.bitmap_from_directxtex(image, 0)?;
    // // let from_pixel_format = bitmap.pixel_format()?;
    // // let rgb = bitmap.to_pixel_format(&from_pixel_format, PIXEL_FORMAT_BGR)?;
    // // rgb.save(&file, CONTAINER_PNG)
}

#[cfg(windows)]
fn export_texture(
    format: TextureFormat,
    inputs: &[Utf8PathBuf],
    outputs: &[Utf8PathBuf],
) -> Result<usize> {
    let mut output_count = 0;

    let (dimensions, texture_file) = format.best_texture(inputs).ok_or_else(|| {
        Error::message("Detected a format and it didn't match, the file may be corrupted.")
    })?;
    let all_data = std::fs::read(texture_file)?;
    let texture_data = format.without_header(&all_data);

    let pixel_format = format.dxgi_format.uncompressed_format();
    let raw_image = DXTImage::new(
        format.dxgi_format,
        dimensions.width, dimensions.height,
        format.array_size,
        dimensions.mipmaps,
        texture_data,
    )
    .map_err(|error| {
        Error::message(format!(
            "Loading the texture data from {texture_file} as {} failed: {error}",
            format.dxgi_format.display()
        ))
    })?;

    let output_image = raw_image.to_format(pixel_format).map_err(|error| {
        Error::message(format!(
            "Decompressing texture data from {texture_file} to pixel format {} failed: {error}",
            pixel_format.display()
        ))
    })?;

    let metadata = output_image.metadata()?;
    for (array_index, output_file) in outputs.iter().enumerate() {
        if format.planes() == ColorPlanes::Rgb {
            save_rgb(&output_image, array_index, output_file).map_err(|error| {
                Error::message(format!(
                    "Windows Imaging Component returned an error while saving PNG: {error}"
                ))
            })?;
        } else {
            output_image
                .save(array_index, &output_file)
                .map_err(|error| {
                    Error::message(format!(
                        "Failed to save the file as {output_file} from format {}: {error}",
                        metadata.format.display()
                    ))
                })?;
        }
        output_count += 1;
    }
    Ok(output_count)
}

#[cfg(windows)]
fn export_textures(groups: impl IntoIterator<Item = Categorized>) -> Result<(String, Warnings)> {
    let mut input_count: usize = 0;
    let mut output_count: usize = 0;
    let mut warnings = Warnings::new();

    for group in groups {
        let group = FileGroup(group);
        let orig_inputs = group.files.clone();
        if orig_inputs.is_empty() {
            continue;
        }
        let Scanned { input, output, .. } = group.scan().0;

        match (input, output) {
            (FileStatus::Unknown, _) => continue,
            (FileStatus::Ok(new_warnings, inputs), OutputFormat::Exact { format, outputs }) => {
                let first = orig_inputs.first().unwrap();
                for warning in new_warnings {
                    warnings.push(format!("{first}: {warning}"));
                }

                output_count += export_texture(format, &inputs, &outputs)?;
                input_count += 1;
            }
            (FileStatus::Ok(new_warnings, inputs), OutputFormat::Candidates(candidates))
                if !candidates.is_empty() =>
            {
                let format = *candidates.first().unwrap();
                let outputs = as_images(&format, &inputs);
                let first = orig_inputs.first().unwrap();
                warnings.push(format!(
                    "{first}: Guessed the file format based on file size"
                ));
                for warning in new_warnings {
                    warnings.push(format!("{first}: {warning}"));
                }
                output_count += export_texture(format, &inputs, &outputs)?;
                input_count += 1;
            }
            (FileStatus::Error(error), _) => {
                let mut message = String::new();
                for file in orig_inputs {
                    message.push_str(file.as_str());
                    message.push('\n');
                }
                message.push('\n');
                message.push_str(&error.to_string());

                let message = message.replace('\n', "\r\n");
                message_box_error(message, APP_TITLE);
            }
            (..) => {
                let first = orig_inputs.first().unwrap();
                warnings.push(format!("Failed to find the correct format for {first}"));
            }
        }
    }

    Ok((
        format!("{input_count} textures exported to {output_count} files"),
        warnings,
    ))
}

#[cfg(windows)]
fn import_images(groups: impl IntoIterator<Item = Categorized>) -> Result<(String, Warnings)> {
    let mut input_count: usize = 0;
    let mut output_count: usize = 0;
    let mut warnings = Warnings::new();

    for group in groups {
        let group = FileGroup(group);
        let orig_inputs = group.files.clone();
        let Scanned { input, output, .. } = group.scan().0;
        match (input, output) {
            (FileStatus::Unknown, _) => continue,
            (FileStatus::Ok(input_warnings, inputs), OutputFormat::Exact { format, outputs }) => {
                let first = orig_inputs
                    .first()
                    .and_then(|f| f.file_name())
                    .unwrap_or_default();
                let (new_outputs, output_warnings) = import_image(format, &inputs, &outputs)
                    .map_err(|error| {
                        Error::message(format!(
                            "Failed to import {inputs:?} to {}: {error}",
                            format.dxgi_format.display()
                        ))
                    })?;
                for warning in input_warnings
                    .into_iter()
                    .chain(output_warnings.into_iter())
                {
                    warnings.push(format!("{first}: {warning}"));
                }
                output_count += new_outputs;
                input_count += 1;
            }
            (FileStatus::Ok(input_warnings, inputs), OutputFormat::Candidates(candidates))
                if !candidates.is_empty() =>
            {
                let format = *candidates.first().unwrap();
                let outputs = as_textures(&format, &inputs);
                let first = orig_inputs
                    .first()
                    .and_then(|f| f.file_name())
                    .unwrap_or_default();
                let (new_outputs, output_warnings) = import_image(format, &inputs, &outputs)
                    .map_err(|error| {
                        Error::message(format!(
                            "Failed to import {inputs:?} to {}: {error}",
                            format.dxgi_format.display()
                        ))
                    })?;
                for warning in input_warnings
                    .into_iter()
                    .chain(output_warnings.into_iter()).chain(std::iter::once(Cow::Owned(format!(
                        "{first}: Guessed the file format based on file size"
                    ).into())))
                {
                    warnings.push(format!("{first}: {warning}"));
                }
                output_count += new_outputs;
                input_count += 1;
            }
            (FileStatus::Error(error), _) => {
                let mut message = String::new();
                for file in orig_inputs {
                    message.push_str(file.as_str());
                    message.push('\n');
                }
                message.push('\n');
                message.push_str(&error.to_string());

                let message = message.replace('\n', "\r\n");
                message_box_error(message, APP_TITLE);
            }
            (..) => {
                let first = orig_inputs.first().unwrap();
                warnings.push(format!("Failed to find the correct format for {first}"));
            }
        }
    }

    Ok((
        format!("{input_count} textures exported to {output_count} files"),
        warnings,
    ))
}

#[cfg(windows)]
fn bring_dx_to_format<'a>(
    image: &'a DXTImage,
    format: DXGI_FORMAT,
    dimensions: Dimensions,
) -> Result<(Cow<'a, DXTImage>, Warnings)> {
    let mut warnings = Warnings::new();
    let mut metadata = image.metadata()?;
    let image = if metadata.format == format
        && (metadata.width, metadata.height) == (dimensions.width, dimensions.height)
    {
        return Ok((Cow::Borrowed(image), warnings));
    } else if metadata.format.is_compressed() {
        Cow::Owned(image.decompress()?)
    } else {
        Cow::Borrowed(image)
    };

    metadata = image.metadata()?;
    let image = if metadata.format == format {
        image
    } else {
        Cow::Owned(image.convert(format, TEX_FILTER_FLAGS::default())?)
    };

    let metadata = image.metadata()?;
    if (metadata.width, metadata.height) == (dimensions.width, dimensions.height) {
        Ok((image, warnings))
    } else {
        warnings.push(format!(
            "Wrong dimensions ({}x{}), resized to {}x{}",
            metadata.width, metadata.height, dimensions.width, dimensions.height
        ));
        event!(
            WARN,
            "Resizing to {}x{} from {}x{}",
            dimensions.width,
            dimensions.height,
            metadata.width,
            metadata.height
        );
        Ok((
            Cow::Owned(image.resize(dimensions.width, dimensions.height)?),
            warnings,
        ))
    }
}

#[cfg(windows)]
fn load_image_array(
    array_size: usize,
    compressed_format: DXGI_FORMAT,
    pixel_format: DXGI_FORMAT,
    dimensions: Dimensions,
    images: &[Utf8PathBuf],
) -> Result<(DXTImage, Warnings)> {
    let mut warnings = Warnings::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(dimensions.data_size);

    for file in images {
        let dx = DXTImage::load(file).log_failure()?;
        let metadata = dx.metadata().log_failure()?;

        if images.len() == 1
            && metadata.format == compressed_format
            && dx.len() == dimensions.data_size
        {
            return Ok((dx, warnings));
        }

        if images.len() != array_size {
            return error_message(format!(
                "This texture contains {} images and only {} files were provided",
                array_size,
                images.len()
            ));
        }

        let (image, input_warnings) =
            bring_dx_to_format(&dx, pixel_format, dimensions).log_failure()?;
        warnings.extend(input_warnings);
        buffer.extend(image.image(0).log_failure()?);
    }

    Ok(DXTImage::new(
        pixel_format,
        dimensions.width, dimensions.height,
        images.len(),
        1,
        &buffer,
    )
    .log_failure()
    .map(|img| (img, warnings))?)
}

#[cfg(windows)]
fn import_image(
    format: TextureFormat,
    inputs: &[Utf8PathBuf],
    outputs: &[Utf8PathBuf],
) -> Result<(usize, Warnings)> {
    let mut output_count = 0;

    let span = span!(TRACE, "import_image", ?format);
    let _enter = span.enter();

    let dimensions = format.dimensions();
    let (image, warnings) = load_image_array(
        format.array_size,
        format.dxgi_format,
        format.dxgi_format.uncompressed_format(),
        dimensions,
        inputs,
    )
    .log_failure()
    .map_err(|error| Error::message(format!("Failed to load {inputs:?}: {error}")))?;

    for (dimensions, output_file) in format.dimensions_iter().zip(outputs.iter()) {
        let metadata = image.metadata().log_failure()?;
        let image = if (dimensions.width, dimensions.height) == (metadata.width, metadata.height) {
            Cow::Borrowed(&image)
        } else {
            Cow::Owned(image.resize(dimensions.width, dimensions.height)?)
        };

        let image = if metadata.format != format.dxgi_format
            && metadata.format != format.dxgi_format.uncompressed_format()
        {
            Cow::Owned(image.convert(
                format.dxgi_format.uncompressed_format(),
                TEX_FILTER_FLAGS::default(),
            )?)
        } else {
            image
        };

        let image = if dimensions.mipmaps > 1 {
            let metadata = image.metadata()?;
            // let expected = dxtex::expected_size_array(metadata.format, ,
            // format.array_size);
            let expected =
                directxtex::expected_size_array(metadata.format, dimensions.width, dimensions.height, format.array_size, dimensions.mipmaps);
            if image.len() == expected {
                image
            } else {
                // event!(DEBUG, "expected size is {expected}, image is {}", image.len());
                let data = image.pixels()?;
                let stripped = DXTImage::new(
                    metadata.format,
                    dimensions.width,
                    dimensions.height,
                    format.array_size,
                    1,
                    &data,
                )
                .log_failure()?;
                // event!(DEBUG, dimensions.mipmaps, len = stripped.len(), "generating mips");
                Cow::Owned(
                    stripped
                        .generate_mipmaps(dimensions.mipmaps)
                        .log_failure()?,
                )
            }
            // let image = image.pixels();
            // event!(TRACE, "stripping mips");
            // (0 .. format.array_size)
            // for images in 0 .. format.array_size
            // Cow::Owned(image.generate_mipmaps(dimensions.mipmaps)?)
        } else {
            image
        };
        // let metadata = image.metadata()?;
        // event!(DEBUG, len = image.len(), "before compress, mips: {}",
        // metadata.mipLevels);

        let metadata = image.metadata()?;
        let image = if metadata.format == format.dxgi_format {
            image
        } else {
            Cow::Owned(image.compress(format.dxgi_format)?)
        };
        // let metadata = image.metadata()?;
        // event!(DEBUG, "after compress, mips: {}", metadata.mipLevels);

        if image.len() != dimensions.data_size {
            let metadata = image.metadata()?;
            event!(ERROR, name="failed to hit target", ?metadata, ?dimensions, format = %format.dxgi_format.display(), len = %image.len());

            return error_message(format!(
                "Failed to hit the correct data size: expected {}, got {}",
                dimensions.data_size,
                image.len()
            ));
        }

        let mut writer = BufWriter::new(File::create(output_file)?);
        if output_file.as_str().ends_with(".texture") {
            let raw_headers = registry().raw_headers.get(&format.id()).ok_or_else(|| {
                Error::message(format!(
                    "Internal error: Missing the correct headers for format id {}",
                    format.id()
                ))
            })?;

            event!(TRACE, "Writing .texture headers to {output_file}");
            writer.write_all(bytemuck::bytes_of(&texture_file::FileHeader::with_length(
                format.standard.data_size,
            )))?;
            writer.write_all(bytemuck::bytes_of(&texture_file::TextureHeader::new()))?;
            writer.write_all(texture_file::TEXTURE_TAG)?;
            writer.write_all(bytemuck::bytes_of(
                &texture_file::FormatHeader::from_hexstring(raw_headers)?,
            ))?;
        }
        let pixels = image.pixels()?;
        writer.write_all(&pixels)?;

        event!(TRACE, "Saved {output_file}");
        output_count += 1;
    }

    Ok((output_count, warnings))
}

#[cfg(windows)]
#[test]
fn test_import() {
    log_for_tests(true);

    let mut inputs = inputs::gather(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/import"));
    inputs.textures.clear();

    let (string, warnings) = run(inputs).unwrap();

    for warning in warnings {
        event!(WARN, %warning);
    }
    event!(INFO, message = %string);
}

#[cfg(windows)]
#[test]
fn test_export() {
    log_for_tests(true);

    let mut inputs = inputs::gather(concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/export"));
    inputs.images.clear();

    let (string, warnings) = run(inputs).unwrap();

    for warning in warnings {
        event!(WARN, %warning);
    }
    event!(INFO, message = %string);
}
