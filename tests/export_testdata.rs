use camino::{Utf8Path, Utf8PathBuf};
use spidertexlib::dxtex::{self, DXImage, TexMetadata, TEX_DIMENSION, TEX_FILTER_FLAGS};
use spidertexlib::files::{as_images, ng_format_for_texture_file, Categorized, FileType};
use spidertexlib::formats::{
    guess_dimensions_2, probe_textures_2, ColorPlanes, ImageFormat, TextureFormat,
};
use spidertexlib::inputs::Inputs;
use spidertexlib::prelude::*;
use spidertexlib::registry::Registry;
use spidertexlib::rgb::{CONTAINER_PNG, PIXEL_FORMAT_BGR, WIC};
use spidertexlib::util::walkdir;

fn test_metadata(file: &Utf8Path, metadata: &TexMetadata, format: &TextureFormat) {
    let expected_formats = format.planes().expected_formats();

    if !expected_formats.contains(&metadata.format) {
        event!(
            WARN,
            "Output format is {}, container expects {:?}",
            metadata.format.display(),
            expected_formats
        );
    }

    if metadata.width != format.standard.width
        || metadata.height != format.standard.height
        || metadata.mipLevels != format.standard.mipmaps as usize
    {
        if let Some(highres) = format.highres {
            if metadata.width != highres.width
                || metadata.height != highres.height
                || metadata.mipLevels != highres.mipmaps as usize
            {
                event!(
                    WARN,
                    "Dimensions don't match (dxtex: {}x{}/{}, db: {}x{}/{})",
                    metadata.width,
                    metadata.height,
                    metadata.mipLevels,
                    format.standard.width,
                    format.standard.height,
                    format.standard.mipmaps
                );
            }
        } else {
            event!(
                WARN,
                "Dimensions don't match (dxtex: {}x{}, db: {}x{})",
                metadata.width,
                metadata.height,
                format.standard.width,
                format.standard.height
            );
        }
    }
    if metadata.depth > 1 {
        event!(WARN, "Depth is {}", metadata.depth);
    }
    if metadata.arraySize != format.array_size {
        event!(
            WARN,
            "Array size doesn't match (dxtex: {}, db: {})",
            metadata.arraySize,
            format.array_size
        );
    }
    if metadata.dimension != TEX_DIMENSION::Texture2D {
        event!(WARN, "Texture is a {:?}", metadata.dimension);
    }
    if metadata.miscFlags != 0 || metadata.miscFlags2 != 0 {
        event!(
            WARN,
            "Flags are {:08x} and {:08x}",
            metadata.miscFlags,
            metadata.miscFlags2
        );
    }
}

fn test_expected_size(dx: &DXImage, format: &TextureFormat, highres: bool) {
    let expected = if highres {
        format.expected_highres_buffer_size().unwrap()
    } else {
        format.expected_standard_buffer_size()
    };

    if expected != dx.len() {
        event!(
            ERROR,
            "Produced a {} byte buffer (expected: {})",
            dx.len(),
            expected
        );
    }
}

#[test]
fn test() -> Result<()> {
    const EXPORT_TESTDATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/export");

    spidertexlib::util::log_for_tests(true);

    registry::load()?;

    let Inputs { textures, images } = spidertexlib::inputs::gather(EXPORT_TESTDATA);

    for Categorized { files, .. } in images {
        for file in files {
            event!(TRACE, "Removing old export {file}");
            std::fs::remove_file(file)?;
        }
    }

    for Categorized { files, .. } in textures {
        let file = files
            .iter()
            .find(|f| f.as_str().ends_with(".texture") && !f.as_str().ends_with("_hd.texture"))
            .unwrap_or_else(|| files.first().unwrap());

        if let Some(_hd) = files.iter().find(|f| f.as_str().ends_with("_hd.texture")) {
            if files.len() == 1 {
                event!(ERROR, ?files, "Group has _hd alone");
            }
        }

        let span = tracing::error_span!("", file = file.file_name().unwrap_or_default());
        let _entered = span.enter();
        let format = if let Some(format) = ng_format_for_texture_file(file)
            .log_failure_as("format_for_texture_file")
            .or_else(|| {
                let size = std::fs::metadata(file).ok()?.len() as u64 as usize;
                event!(WARN, ?files, "Guessing format from file size {size}");
                registry::formats_for_size(size).first().map(|fmt| **fmt)
            }) {
            format
        } else {
            event!(ERROR, "Failed to find a texture format");
            continue;
        };

        let dimensions = match format
            .dimensions_for_file(file)
            .log_failure_as("dimensions_for_file")
        {
            Some(dimensions) => dimensions,
            None => continue,
        };
        let all_data = std::fs::read(file)?;
        let data = format.without_header(&all_data);

        let pixel_format = format.dxgi_format.uncompressed_format();

        let raw_image =
            DXImage::with_dimensions(format.dxgi_format, dimensions, format.array_size, data)
                .log_failure_as("DXImage::with_dimensions")?;
        let output_image = raw_image.to_format(pixel_format).log_failure_with(|| {
            format!(
                "DXImage::to_format failed to go from {} to {}",
                format.dxgi_format.display(),
                pixel_format.display()
            )
        })?;
        let outputs = as_images(&format, &files);
        let metadata = output_image.metadata()?;
        for (array_index, output) in outputs.into_iter().enumerate() {
            if format.planes() == ColorPlanes::Rgb {
                let wic = WIC::new().expect("WIC::new");
                let bitmap = wic
                    .bitmap_from_directxtex(&output_image, 0)
                    .expect("WIC::bitmap_from_directxtex");
                let from_pixel_format = bitmap.pixel_format().expect("WICSource::pixel_format");
                let rgb = bitmap
                    .to_pixel_format(&from_pixel_format, PIXEL_FORMAT_BGR)
                    .expect("WICSource::to_pixel_format");
                rgb.save(&output, CONTAINER_PNG).expect("WICBitmap::save");
            } else {
                output_image
                    .save(array_index, format.default_image_format(), &output)
                    .log_failure_with(|| {
                        format!(
                            "dx.save failed with {} => {} to {output}",
                            format.dxgi_format.display(),
                            metadata.format.display()
                        )
                    })?;
            }
            event!(INFO, name = "Saved", planes = ?format.planes(), texture_format = %format.dxgi_format.display(), final_format = %metadata.format.display(), file = %output);
        }
    }
    Ok(())
}
