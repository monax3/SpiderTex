use camino::{Utf8Path, Utf8PathBuf};
use spidertexlib::dxtex::{self, DXImage, TexMetadata, TEX_DIMENSION, TEX_FILTER_FLAGS};
use spidertexlib::files::{as_images, ng_format_for_texture_file, Categorized, FileType};
use spidertexlib::formats::{
    guess_dimensions_2,
    probe_textures_2,
    ColorPlanes,
    ImageFormat,
    TextureFormat,
};
use spidertexlib::inputs::Inputs;
use spidertexlib::prelude::*;
use spidertexlib::registry::Registry;
use spidertexlib::rgb::{CONTAINER_PNG, PIXEL_FORMAT_BGR, WIC};
use spidertexlib::util::walkdir;

#[test]
fn outputs() -> Result<()> {
    const IMPORT_TESTDATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/import");

    spidertexlib::util::log_for_tests(true);

    registry::load()?;

    let Inputs { textures, images } = spidertexlib::inputs::gather(IMPORT_TESTDATA);

    for Categorized { files, .. } in textures {
        for file in &files {
            let span = tracing::error_span!("", file = file.file_name().unwrap_or_default());
            let _entered = span.enter();
            let format = if let Some(format) = ng_format_for_texture_file(&file)
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

            event!(DEBUG, %file, ?format);
        }
        continue;

        // let dimensions = match format
        //     .dimensions_for_file(file)
        //     .log_failure_as("dimensions_for_file")
        // {
        //     Some(dimensions) => dimensions,
        //     None => continue,
        // };
        // let all_data = std::fs::read(file)?;
        // let data = format.without_header(&all_data);

        // let pixel_format = format.dxgi_format.uncompressed_format();

        // let raw_image =
        //     DXImage::with_dimensions(format.dxgi_format, dimensions, format.array_size, data)
        //         .log_failure_as("DXImage::with_dimensions")?;
        // let output_image = raw_image.to_format(pixel_format).log_failure_with(|| {
        //     format!(
        //         "DXImage::to_format failed to go from {} to {}",
        //         format.dxgi_format.display(),
        //         pixel_format.display()
        //     )
        // })?;
        // let outputs = as_images(&format, &files);
        // let metadata = output_image.metadata()?;
        // for (array_index, output) in outputs.into_iter().enumerate() {
        //     if format.planes() == ColorPlanes::Rgb {
        //         let wic = WIC::new().expect("WIC::new");
        //         let bitmap = wic
        //             .bitmap_from_directxtex(&output_image, 0)
        //             .expect("WIC::bitmap_from_directxtex");
        //         let from_pixel_format = bitmap.pixel_format().expect("WICSource::pixel_format");
        //         let rgb = bitmap
        //             .to_pixel_format(&from_pixel_format, PIXEL_FORMAT_BGR)
        //             .expect("WICSource::to_pixel_format");
        //         rgb.save(&output, CONTAINER_PNG).expect("WICBitmap::save");
        //     } else {
        //         output_image
        //             .save(array_index, format.default_image_format(), &output)
        //             .log_failure_with(|| {
        //                 format!(
        //                     "dx.save failed with {} => {} to {output}",
        //                     format.dxgi_format.display(),
        //                     metadata.format.display()
        //                 )
        //             })?;
        //     }
        //     event!(INFO, name = "Saved", planes = ?format.planes(), texture_format = %format.dxgi_format.display(), final_format = %metadata.format.display(), file = %output);
        // }
    }
    Ok(())
}
