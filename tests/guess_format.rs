use camino::Utf8Path;
use spidertexlib::dxtex::{self, DXImage, TexMetadata, TEX_DIMENSION, TEX_FILTER_FLAGS};
use spidertexlib::formats::{
    guess_dimensions_2, probe_textures_2, ColorPlanes, ImageFormat, TextureFormat,
};
use spidertexlib::prelude::*;
use spidertexlib::registry::Registry;

const TESTDATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata");

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

fn get_block_size(mut size: usize) -> Option<(usize, usize)> {
    if size.is_power_of_two() {
        let width = 1 << (size.trailing_zeros() / 2);
        return Some((size, width));
    }
    let mut pow2 = size.next_power_of_two() >> 1;
    // event!(TRACE, pow2, size);
    let orig_pow2 = pow2;
    let mut rem = size;

    while pow2 >= 5 && rem >= pow2 {
        // event!(DEBUG, pow2, rem);
        if rem == 0 || rem == pow2 {
            return Some((orig_pow2, 1 << (orig_pow2.trailing_zeros() / 2)));
        }
        rem -= pow2;
        pow2 >>= 2;
    }
    None
}

fn get_block_size_ext(size: usize) -> Option<(usize, usize)> {
    let sqrt = (size as f32).sqrt() as usize;
    if size == sqrt * sqrt {
        return Some((size, sqrt));
    }
    let pow = (size.next_power_of_two().trailing_zeros() / 2);
    if pow < 4 {
        return None;
    } // biggest side is less than 16
    let mut div = 1 << pow;
    let min_width = 1 << (pow - 1);
    event!(
        TRACE,
        size,
        width_start = div,
        width_min = min_width,
        size_mod_div = size % div,
        pow,
        pow1 = 1 << pow
    );
    if size.is_power_of_two() {
        panic!();
    }
    let mut div_10 = div / 10 * 10;
    while div_10 >= min_width {
        // event!(
        //     TRACE,
        //     size,
        //     div_10,
        //     size_div = size / div_10,
        //     size_mod = size % div_10
        // );
        if size % div_10 == 0 && size / div_10 % 2 == 0 {
            return Some((size, div_10));
        }
        div_10 -= 10;
    }
    while div >= min_width {
        // event!(
        //     TRACE,
        //     size,
        //     div,
        //     size_div = size / div,
        //     size_mod = size % div
        // );
        if size % div == 0 && size / div % 2 == 0 {
            return Some((size, div));
        }
        div -= 2;
    }
    // event!(ERROR, size, pow, "failed");
    None
}

#[test]
fn test() -> Result<()> {
    spidertexlib::util::log_for_tests(true);

    let testdir = Utf8Path::new(TESTDATA);
    let mut registry = Registry::load()?;

    for file in spidertexlib::util::walkdir(testdir) {
        let span = tracing::error_span!("", file = file.file_name().unwrap_or_default());
        let _entered = span.enter();

        if file
            .extension()
            .map_or(false, |ext| ext == "texture" || ext == "raw")
        {
            // if !(file.as_str().contains(".raw") || file.as_str().contains("_hd")) {
            // continue; }

            let mut len = std::fs::metadata(&file)?.len() as usize;
            let (data_block, width) = get_block_size(len)
                .or_else(|| get_block_size(len - 0x80))
                .or_else(|| get_block_size(len / 3))
                .or_else(|| get_block_size((len - 0x80) / 3))
                .or_else(|| get_block_size_ext(len))
                .or_else(|| get_block_size_ext(len - 0x80))
                .or_else(|| get_block_size_ext(len / 3))
                .or_else(|| get_block_size_ext((len - 0x80) / 3))
                .unwrap();
            let height = data_block / width;

            let (format, _) = texture_file::read_header(&file)?;
            let dims = (std::cmp::max(width, height), std::cmp::min(width, height));
            let dims_bc16 = (
                std::cmp::max(dims.0, dims.1 * 2),
                std::cmp::min(dims.0, dims.1 * 2),
            );
            if let Some(format) = format {
                let format = format.to();

                let mut f = Vec::new();
                for f_dims in format.dimensions_iter().map(|f| {
                    (
                        std::cmp::max(f.width, f.height),
                        std::cmp::min(f.width, f.height),
                    )
                }) {
                    if dims == f_dims || dims_bc16 == f_dims {
                        continue;
                    }
                    f.push(f_dims);
                }
                event!(TRACE, orig = len, data = data_block, ?dims, f_dims = ?f);
            } else {
                event!(TRACE, orig = len, data = data_block, ?dims, f_dims = "");
            }
            // event!(
            //     TRACE,
            //     "orig = {len}, data = {data_block}, width = {width}, height =
            // {}",     data_block / width
            // );

            // if file.as_str().ends_with(".texture") { len -= 0x80; }
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
            //             spidertexlib::formats::ColorPlanes::Hdr => continue,
            //             spidertexlib::formats::ColorPlanes::Rgba => {
            //                 let img: image::ImageBuffer<image::Rgba<u8>, _> =
            //                     image::ImageBuffer::from_raw(
            //                         metadata.width as u32,
            //                         metadata.height as u32,
            //                         out,
            //                     )
            //                     .expect("PNG creation failed");
            //                 image::DynamicImage::from(img)
            //             }
            //             spidertexlib::formats::ColorPlanes::Luma => {
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
