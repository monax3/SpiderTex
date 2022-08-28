use camino::Utf8Path;
use directxtex::{self, DXTImage, TexMetadata, TEX_DIMENSION, TEX_FILTER_FLAGS};
use texturesofspiderman::formats::{
    guess_dimensions_2,
    probe_textures_2,
    ColorPlanes,
    ImageFormat,
    TextureFormat,
};
use texturesofspiderman::prelude::*;
use texturesofspiderman::registry::Registry;
use texturesofspiderman::texture_file::read_texture;

const TESTDATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata");

#[test]
fn test() -> Result<()> {
    texturesofspiderman::util::log_for_tests(true);

    let testdir = Utf8Path::new(TESTDATA);
    let mut registry = Registry::load()?;

    for format in registry.formats.values() {
        if format.dxgi_format.is_bc1() && format.planes() != ColorPlanes::Rgb {
            event!(ERROR, ?format, "Non-RGB BC1");
        }
        if format.dxgi_format.is_bc4() && format.planes() != ColorPlanes::Luma {
            event!(ERROR, ?format, "Non-luma BC4");
        } else if format.dxgi_format.is_bc6() && format.planes() != ColorPlanes::Hdr {
            event!(ERROR, ?format, "Non-hdr BC6");
        } else if (format.dxgi_format.is_bc7()) && format.planes() != ColorPlanes::Rgba {
            event!(ERROR, ?format, "Non-RGB BC7");
        }
        if format.standard.mipmaps != format.standard.mip_levels(false) {
            event!(
                ERROR,
                "Algo doesn't match for ST {:?}: {}",
                format.standard,
                format.standard.mip_levels(false)
            );
            if let Some(highres) = format.highres {
                if highres.mipmaps != highres.mip_levels(true) {
                    event!(
                        ERROR,
                        "Algo doesn't match for HR {highres:?}: {}",
                        highres.mip_levels(true)
                    );
                }
            }
        }
    }

    for ids in registry.lengths.values() {
        if ids.len() <= 1 {
            continue;
        }

        for id in ids {
            let format = registry.get(id);
            event!(TRACE, "Duplicate format {format}");
        }
    }

    return Ok(());

    for file in texturesofspiderman::util::walkdir(testdir) {
        let span = tracing::error_span!("", file = file.file_name().unwrap_or_default());
        let _entered = span.enter();

        if file
            .extension()
            .map_or(false, |ext| ext == "texture" || ext == "raw")
        {
            let (format, data) = read_texture(&file)?;
            if let Some(format) = format {
                let format = TextureFormat::from(format);
                let dx = DXTImage::new(
                    format.dxgi_format,
                    format.standard.width,
                    format.standard.height,
                    format.array_size,
                    format.standard.mipmaps,
                    &data,
                )
                .log_failure_as("new")?;
                let metadata = dx.metadata().log_failure_as("metadata 1")?;
                // event!(DEBUG, before=?DXGIFormat::from(metadata.format));
                let dx = if directxtex::is_compressed(metadata.format) {
                    dx.decompress().log_failure_as("compress")?
                } else {
                    dx
                };
                let metadata = dx.metadata().log_failure_as("metadata 2")?;
                // event!(DEBUG, after=?DXGIFormat::from(metadata.format));
            }
            // find_data_len(len);
            // let nearest_pow = (0 .. usize::BITS).find(|i| len >> i ==
            // 1).unwrap(); let len_strip = (len >
            // TEXTURE_HEADER_SIZE).then_some(len - TEXTURE_HEADER_SIZE);
            // if len % 4 == 0 {
            //     fn find_mip_levels(len: usize) -> usize {
            //         let mut mips = 0;
            //         let mut rem = len;
            //         while rem > 0 {
            //             let nearest_pow = (0 .. usize::BITS).find(|i| rem >>
            // i == 1).unwrap_or_default();             rem -= 1 <<
            // nearest_pow;             mips += 1;
            //         }
            //         mips
            //     }
            //     let nearest_pow = (0 .. usize::BITS).find(|i| len >> i ==
            // 1).unwrap();     let rem = len - (1 << nearest_pow);
            //     let sqrt = ((1 << nearest_pow) as f32).sqrt() as usize;
            //     let sqrt_rem = (rem as f32).sqrt() as usize;
            //     let mips = find_mip_levels(len);
            //     let test = (len as f32).log2();
            //     event!(TRACE, "rem = {}", len - (1 << nearest_pow));
            //     event!(TRACE, "log2 = {}", 8192.0_f32.log2());
            //     event!(TRACE, "len / 4 = {}, len % 256 = {}, len % (256*256) = {}, rem = {}, sqrt = {}, sqrt_rem = {}, mips = {}, log2 = {}", len / 4, len % (256 * 256), len % 256, rem, sqrt, sqrt_rem, mips, test);
            // } else if let Some(len) = len_strip {
            //     if len % 4 == 0 {
            //         event!(TRACE, "len(strip) / 4 = {}", len / 4);
            //     }
            // }

            // let (detected_formats, _smallest_file, mut image_buffer) =
            //     probe_textures_2(&mut registry, &[file.to_owned()])?;
            // if !detected_formats.is_empty() {
            //     if let Some((dimensions, strip_header)) =
            //         guess_dimensions_2(image_buffer.len(), &detected_formats)
            //     {
            //         // FIXME
            //         let format = detected_formats.first().unwrap();
            //         let data = if strip_header {
            //             &image_buffer[TEXTURE_HEADER_SIZE ..]
            //         } else {
            //             image_buffer.as_slice()
            //         };
            //         let dx = DXImage::with_dimensions(
            //             format.dxgi_format,
            //             dimensions,
            //             format.array_size,
            //             data,
            //         )?;

            //         let mut dxgi_format = format.dxgi_format;
            //         let mut mipmaps = dimensions.mipmaps;

            //         let out_dds = Utf8Path::new("out")
            //             .join(file.file_name().unwrap())
            //             .with_extension("dds");
            //         let out_tga = out_dds.with_extension("tga");
            //         let out_png = out_dds.with_extension("png");
            //         let out_exr = out_dds.with_extension("exr");

            //         let dx = dx
            //             .map_if(format.dxgi_format.is_compressed(),
            // DXImage::decompress)?             .inspect(|dx| {
            //                 let metadata = dx.metadata()?;
            //                 let new_format =
            // DXGIFormat::from(metadata.format);

            //                 if format.dxgi_format != new_format {
            //                     event!(INFO, "{:?} => {:?}",
            // format.dxgi_format, new_format);                 }
            //                 event!(TRACE, "{metadata:?}");
            //                 test_metadata(file, &metadata, format);
            //                 Ok(())
            //             })?
            //             .inspect(|dx| {
            //                 let metadata = dx.metadata()?;
            //                 let new_format =
            // DXGIFormat::from(metadata.format);
            // event!(INFO, "Saving with format {:?}", new_format);
            //                 Ok(())
            //             })?
            //             .inspect(|dx|
            // dx.save_dds(out_dds).log_failure_as("DDS save"))?
            //             .inspect(|dx| {
            //                 if format.planes != ColorPlanes::Hdr {
            //                     dx.save_tga(0,
            // out_tga).log_failure_as("Saving TGA")
            // } else {                     Ok(())
            //                 }
            //             })?;

            //         let (out, metadata) = (dx.pixels()?, dx.metadata()?);

            //         if format.default_output_format == ImageFormat::Exr {
            //             let metadata = dx.metadata()?;
            //             event!(ERROR, ?metadata);
            //             dx.save_exr(0, out_exr).log_failure_as("Save EXR")?;
            //         }

            //         let img = match format.planes {
            //             texturesofspiderman::formats::ColorPlanes::Hdr => continue,
            //             texturesofspiderman::formats::ColorPlanes::Rgba => {
            //                 let img: image::ImageBuffer<image::Rgba<u8>, _> =
            //                     image::ImageBuffer::from_raw(
            //                         metadata.width as u32,
            //                         metadata.height as u32,
            //                         out,
            //                     )
            //                     .expect("PNG creation failed");
            //                 image::DynamicImage::from(img)
            //             }
            //             texturesofspiderman::formats::ColorPlanes::Luma => {
            //                 let img: image::ImageBuffer<image::Luma<u8>, _> =
            //                     image::ImageBuffer::from_raw(
            //                         metadata.width as u32,
            //                         metadata.height as u32,
            //                         out,
            //                     )
            //                     .expect("PNG creation failed");
            //                 image::DynamicImage::from(img)
            //             }
            //         };
            //         match img.save(&out_png) {
            //             Ok(()) => {
            //                 let metadata =
            // dxtex::metadata_from_wic(&out_png)?;
            // event!(INFO, "Exported to {out_png} {metadata:?}");
            //             }
            //             Err(error) => event!(ERROR, "{error}"),
            //         }
            //     } else {
            //         event!(ERROR, "{file}: Couldn't match
            // dimensions");     }
            // } else {
            //     event!(ERROR, "{file}: No usable format found");
            // }
        }
    }

    Ok(())
}
