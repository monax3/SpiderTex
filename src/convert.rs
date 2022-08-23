use std::fs::File;
use std::time::{Duration, Instant};

use camino::{Utf8Path, Utf8PathBuf};
use image::ImageBuffer;
use windows::Win32::Graphics::Dxgi::Common::*;

use crate::dxtex::DXImage;
use crate::files::{FileFormat, FileGroup, FileStatus, FileType, Categorized};
use crate::formats::ColorPlanes;
use crate::images::{DxImport, ImageRs};
use crate::prelude::*;

pub enum TaskResult {
    Started(usize),
    Result(FileGroup<Categorized>, Duration),
    Complete(Duration),
}

pub fn convert(group: FileGroup<Categorized>) -> Option<TaskResult> {
    todo!()

    // let outputs = match &group.outputs {
    //     Ok(outputs) => outputs,
    //     Err(error) => return None,
    // };
    // {
    //     let status = group.status.lock();
    //     if !matches!(&*status, FileStatus::Ok | FileStatus::Warnings(_)) {
    //         return None;
    //     }
    // }
    // if group.file_type == FileType::Texture {
    //     Some(export_texture(group))
    // } else {
    //     Some(import_images(group))
    // }
}

pub fn task_error(group: FileGroup<Categorized>, error: Error, duration: Duration) -> TaskResult {
    todo!()
    //TaskResult::Result(group, duration)
}

pub fn export_texture(group: FileGroup<Categorized>) -> TaskResult {
    // let start = Instant::now();

    // let format = match &group.format {
    //     FileFormat::Ready(format) => format.clone(),
    //     _ => return task_error(group, Error::Internal, start.elapsed()),
    // };

    // let (texture_file, mut dimensions) = pick_best_texture(&format, &group.inputs);
    // let texture_data = match std::fs::read(&texture_file) {
    //     Err(error) => return task_error(group, error.into(), start.elapsed()),
    //     Ok(data) => data,
    // };
    // let data = format.without_header(&texture_data);

    // if let Some(highres) = format.highres {
    //     let expected_size =
    //         dxtex::expected_size_array(format.dxgi_format, dimensions, format.array_size);
    //     if expected_size != data.len() {
    //         let expected_size =
    //             dxtex::expected_size_array(format.dxgi_format, highres, format.array_size);
    //         if expected_size == data.len() {
    //             dimensions = highres;
    //             event!(WARN, "Matched with high-res dimensions");
    //         } else {
    //             event!(
    //                 WARN,
    //                 "Dimensions don't match, conversion will probably fail"
    //             );
    //         }
    //     }
    // }

    // let dx = match if format.is_1d() {
    //     DXImage::new_1d(
    //         format.dxgi_format,
    //         dimensions.width,
    //         format.array_size,
    //         dimensions.mipmaps,
    //         data,
    //     )
    //     .log_failure_as("new_1d")
    // } else {
    //     DXImage::new_2d(
    //         format.dxgi_format,
    //         dimensions.width,
    //         dimensions.height,
    //         format.array_size,
    //         dimensions.mipmaps,
    //         data,
    //     )
    //     .log_failure_as("new_2d")
    // } {
    //     Err(error) => return task_error(group, error, start.elapsed()),
    //     Ok(dx) => dx,
    // };

    // let dx = match dx.map_if(format.dxgi_format.is_compressed(), |dx| {
    //     dx.decompress()
    //         .log_failure_with(|| format!("{texture_file}: Decompression failed"))
    // }) {
    //     Err(error) => return task_error(group, error, start.elapsed()),
    //     Ok(dx) => dx,
    // };

    todo!()
    // for (array_index, output) in
    // group.outputs.as_ref().unwrap().into_iter().enumerate() {     if let
    // Err(error) = save_image(&dx, &format, &output, array_index) {
    //         return task_error(group, error, start.elapsed());
    //     }
    // }

    // TaskResult::Result(group, start.elapsed())
}

pub fn save_image(
    image: &DXImage,
    format: &TextureFormat,
    file: &Utf8Path,
    array_index: usize,
) -> Result<()> {
    event!(INFO, "Saving {file}");

    let metadata = image.metadata()?;

    let ext = file.extension().unwrap();
    match ext {
        ext if ext.eq_ignore_ascii_case("dds") => image.save_dds(file),
        ext if ext.eq_ignore_ascii_case("hdr") => image.save_hdr(array_index, file),
        ext if ext.eq_ignore_ascii_case("exr") => image.save_exr(array_index, file),
        ext if ext.eq_ignore_ascii_case("tga") => image.save_tga(array_index, file),
        ext if ext.eq_ignore_ascii_case("png") => {
            let data = image.image(array_index)?;

            match metadata.format {
                DXGI_FORMAT_R8_UNORM => {
                    let img: ImageBuffer<image::Luma<u8>, _> =
                        ImageBuffer::from_raw(metadata.width as u32, metadata.height as u32, data)
                            .log_failure_as("Failed to open image data")
                            .ok_or(Error::Internal)?;
                    Ok(img
                        .save(file)
                        .log_failure_with(|| format!("Failed to save {file}"))?)
                }
                DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => {
                    let img: ImageBuffer<image::Rgba<u8>, _> =
                        ImageBuffer::from_raw(metadata.width as u32, metadata.height as u32, data)
                            .log_failure_as("Failed to open image data")
                            .ok_or(Error::Internal)?;
                    Ok(img
                        .save(file)
                        .log_failure_with(|| format!("Failed to save {file}"))?)
                }
                format => error_message(format!(
                    "Trying to save an unsupported format {}",
                    format.display()
                )),
            }
        }
        _ => error_message("Trying to save with unsupported extension"),
    }
}

fn pipeline_png<FILE: AsRef<Utf8Path> + std::fmt::Display>(
    format: &TextureFormat,
    mut inputs: impl Iterator<Item = FILE>,
    (dim_primary, dim_secondary): (Dimensions, Option<Dimensions>),
) -> impl Iterator<Item = Result<(Vec<u8>, Option<Vec<u8>>)>> {
    std::iter::from_fn(move || {
        inputs.next().and_then(|input| {
            let span = span!(TRACE, "pipeline_png", %input);
            let _entered = span.enter();

            match image::open(input.as_ref()) {
                Err(error) => Some(Err(error.into())),
                Ok(image) => {
                    event!(
                        TRACE,
                        "Resizing primary to {}x{}",
                        dim_primary.width,
                        dim_primary.height
                    );
                    let primary = if image.width() as usize != dim_primary.width
                        || image.height() as usize != dim_primary.height
                    {
                        image
                            .resize_exact(
                                dim_primary.width as u32,
                                dim_primary.height as u32,
                                crate::IMAGERS_RESIZE_FILTER,
                            )
                            .into_bytes()
                    } else {
                        event!(TRACE, "It's already correct");
                        image.clone().into_bytes()
                    };
                    let secondary = dim_secondary.map(|dim| {
                        event!(TRACE, "Resizing secondary to {}x{}", dim.width, dim.height);
                        if image.width() as usize != dim.width
                            || image.height() as usize != dim.height
                        {
                            image
                                .resize_exact(
                                    dim.width as u32,
                                    dim.height as u32,
                                    crate::IMAGERS_RESIZE_FILTER,
                                )
                                .into_bytes()
                        } else {
                            event!(TRACE, "It's already correct");
                            image.clone().into_bytes()
                        }
                    });
                    Some(Ok((primary, secondary)))
                }
            }
        })
    })
}

fn pipeline_dx<FILE: AsRef<Utf8Path> + std::fmt::Display>(
    format: &TextureFormat,
    mut inputs: impl Iterator<Item = FILE>,
    (dim_primary, dim_secondary): (Dimensions, Option<Dimensions>),
) -> impl Iterator<Item = Result<(Vec<u8>, Option<Vec<u8>>)>> {
    std::iter::from_fn(move || {
        inputs.next().and_then(|input| {
            let span = span!(TRACE, "pipeline_dx", %input);
            let _entered = span.enter();

            match DXImage::load(input.as_ref()) {
                Err(error) => Some(Err(error.into())),
                Ok(image) => {
                    let metadata = image.metadata().unwrap();

                    event!(
                        TRACE,
                        "Resizing primary to {}x{}",
                        dim_primary.width,
                        dim_primary.height
                    );
                    let primary = if metadata.width != dim_primary.width
                        || metadata.width as usize != dim_primary.height
                    {
                        image
                            .resize(dim_primary.width, dim_primary.height)
                            .unwrap()
                            .pixels()
                            .unwrap()
                    } else {
                        event!(TRACE, "It's already correct");
                        image.pixels().unwrap()
                    };
                    let secondary = dim_secondary.map(|dim| {
                        event!(TRACE, "Resizing secondary to {}x{}", dim.width, dim.height);
                        if metadata.width != dim.width || metadata.height != dim.height {
                            image
                                .resize(dim.width, dim.height)
                                .unwrap()
                                .pixels()
                                .unwrap()
                        } else {
                            event!(TRACE, "It's already correct");
                            image.pixels().unwrap()
                        }
                    });
                    Some(Ok((primary, secondary)))
                }
            }
        })
    })
}

pub fn load_image(file: &Utf8Path, dimensions: Dimensions) -> Result<Vec<u8>> {
    event!(INFO, "Reading {file}");

    let ext = file.extension().unwrap();
    match ext {
        ext if ext.eq_ignore_ascii_case("dds") => dxtex::load_dds(file)
            .and_then(|dx| dx.resize(dimensions.width, dimensions.height))
            .and_then(|dx| dx.pixels()),
        ext if ext.eq_ignore_ascii_case("hdr") => dxtex::load_hdr(file)
            .and_then(|dx| dx.resize(dimensions.width, dimensions.height))
            .and_then(|dx| dx.pixels()),
        ext if ext.eq_ignore_ascii_case("exr") => dxtex::load_exr(file)
            .and_then(|dx| dx.resize(dimensions.width, dimensions.height))
            .and_then(|dx| dx.pixels()),
        ext if ext.eq_ignore_ascii_case("tga") => dxtex::load_tga(file)
            .and_then(|dx| dx.resize(dimensions.width, dimensions.height))
            .and_then(|dx| dx.pixels()),
        ext if ext.eq_ignore_ascii_case("png") => {
            let image = image::open(file)?;
            Ok(image
                .resize_exact(
                    dimensions.width as u32,
                    dimensions.height as u32,
                    crate::IMAGERS_RESIZE_FILTER,
                )
                .into_bytes())
        }
        _ => error_message("Trying to load an unsupported extension"),
    }
}

pub fn pick_best_texture<'a>(
    format: &TextureFormat,
    inputs: &'a [Utf8PathBuf],
) -> (&'a Utf8Path, Dimensions) {
    let mut best: Option<(&Utf8Path, Dimensions)> = None;

    for file in inputs {
        let size = std::fs::metadata(&file).unwrap().len() as usize;
        if let Some(highres) = format.highres {
            if highres.data_size == size {
                return (file, highres);
            }
        }
        if size == format.standard.data_size || size == format.sd_file_len() {
            best = Some((file, format.standard));
        }
    }

    match best {
        None => {
            event!(ERROR, ?format, ?inputs, "No 'best' file found for");
            (inputs.first().unwrap(), format.standard)
        }
        Some(best) => best,
    }
}

#[cfg(disabled)]
pub fn import_images(group: FileGroup) -> TaskResult {
    let start = Instant::now();

    let format = match &group.format {
        FileFormat::Final(format) => format.clone(),
        _ => return task_error(group, Error::Internal, start.elapsed()),
    };

    let dimensions = if let Some(highres) = format.highres {
        (highres, Some(format.standard))
    } else {
        (format.standard, None)
    };

    // let dimensions = format.highres.unwrap_or(format.standard);
    let mut texture_data = Vec::new();

    let pipeline: Vec<Result<(Vec<u8>, Option<Vec<u8>>)>> =
        match group.inputs.first().as_ref().unwrap().extension().unwrap() {
            ext if ext.eq_ignore_ascii_case("png") => {
                pipeline_png(&format, group.inputs.iter(), dimensions).collect()
            }
            _ => pipeline_dx(&format, group.inputs.iter(), dimensions).collect(),
        };

    for result in pipeline {
        texture_data.extend(match load_image(input, dimensions) {
            Err(error) => return task_error(group, error, start.elapsed()),
            Ok(data) => data,
        });
    }

    let expected_size =
        dxtex::expected_size_array(format.dxgi_format, dimensions, format.array_size);
    if expected_size != texture_data.len() {
        event!(ERROR, expected_size, have_size = %texture_data.len(), ?group.inputs);
    }

    let dx = match if format.is_1d() {
        DXImage::new_1d(
            format.dxgi_format,
            dimensions.width,
            format.array_size,
            1,
            &texture_data,
        )
        .log_failure_as("new_1d")
    } else {
        DXImage::new_2d(
            format.dxgi_format,
            dimensions.width,
            dimensions.height,
            format.array_size,
            1,
            &texture_data,
        )
        .log_failure_as("new_2d")
    } {
        Err(error) => return task_error(group, error, start.elapsed()),
        Ok(dx) => dx,
    };

    for file in group.outputs.as_ref().unwrap() {
        // FIXME: quick hack to git er dun
        if file.as_str().ends_with(".raw") {
            let pixels = dx.pixels().unwrap();
            std::fs::write(file, pixels)
                .log_failure_with(|| format!("Trying to save {file}"))
                .ignore();
        } else if file.as_str().ends_with(".texture") {
            use std::io::prelude::*;
            let mut file = File::create(file)
                .log_failure_with(|| format!("Trying to save {file}"))
                .unwrap();
            file.write_all(bytemuck::bytes_of(&texture_file::FileHeader::default()))
                .unwrap();
            file.write_all(bytemuck::bytes_of(&texture_file::TextureHeader::default()))
                .unwrap();
            file.write_all(texture_file::TEXTURE_TAG).unwrap();
            file.write_all(bytemuck::bytes_of(&format.to_header()))
                .unwrap();
            file.write_all(&texture_data).unwrap();
        }
    }

    TaskResult::Result(group, start.elapsed())
}
