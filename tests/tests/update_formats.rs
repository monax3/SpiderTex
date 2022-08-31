use camino::Utf8Path;
use directxtex::DXTImage;
use texturesofspiderman::formats::ColorPlanes;
use texturesofspiderman::prelude::*;
use texturesofspiderman::registry::Registry;

const TESTDATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/export");

fn guess_array_size(format: &TextureFormat, data_size: usize) -> Option<usize> {
    if format.array_size > 1 {
        return None;
    }
    let expected = directxtex::expected_size(format.dxgi_format, format.standard.width, format.standard.height, 1, format.standard.mipmaps);
    if expected == data_size {
        return None;
    }
    if data_size % expected == 0 {
        let array_size = data_size / expected;
        let expected = directxtex::expected_size_array(format.dxgi_format, format.standard.width, format.standard.height, array_size, format.standard.mipmaps);
        if expected == data_size {
            event!(INFO, "Array size {array_size} looks good!");
            return Some(array_size);
        }
    }
    event!(ERROR, "Failed to find an array size that works");
    None
}

#[derive(serde::Deserialize)]
struct Override {
    pattern: String,
    header:  String,
}

#[cfg(disabled)]
fn load_overrides(registry: &mut Registry) -> Result<()> {
    const OVERRIDES: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/overrides.json"));
    let overrides: Vec<Override> = serde_json::from_str(OVERRIDES)?;

    for Override { pattern, header } in overrides {
        let header_buf = hex::decode(&header)?;
        let header_ref: &texture_file::FormatHeader = bytemuck::from_bytes(&header_buf);
        let id = TextureFormat::from(header_ref).id();

        registry.overrides.push((pattern, id));
    }

    Ok(())
}

fn main() -> Result<()> {
    texturesofspiderman::util::log_for_tests(true);

    let mut registry = Registry::load()?;
    // load_overrides(&mut registry)?;

    let testdir = Utf8Path::new(TESTDATA);

    for file in texturesofspiderman::util::walkdir(testdir) {
        let span = tracing::error_span!("", file = file.file_name().unwrap_or_default());
        let _entered = span.enter();

        if file
            .extension()
            .map_or(false, |ext| ext == "texture" || ext == "raw")
            && !file.file_name().unwrap().contains("_hd.")
        {
            let len = std::fs::metadata(&file).map(|m| m.len())? as usize;

            if file.extension().unwrap() == "raw" && file.with_extension("texture").exists() { continue; }
            // FIXME: this should be read_header except we're now gonna do luma detection
            match texture_file::read_texture(&file) {
                Err(error) => event!(ERROR, %error),
                Ok((None, _)) => {
                    let formats = registry.formats_with_size(len);
                    if let Some(format) = formats.first() {
                        let format = registry.get(format);
                        event!(INFO, "Format is probably {format}");
                    } else {
                        event!(WARN, "Unrecognized file with no header");
                    }
                }
                Ok((Some(header), data)) => {
                    let mut format = header.to();
                    if registry.known(format.id()) {
                        event!(INFO, "Known {format}");
                        registry.replace_format(format);
                    } else {
                        event!(INFO, "Added {format}");

                        let data = format.without_header(&data);

                        if let Some(array_size) = guess_array_size(&format, data.len()) {
                            format.array_size = array_size;
                        }

                        let expected = directxtex::expected_size_array(
                            format.dxgi_format,
                            format.standard.width, format.standard.height,
                            format.array_size,
                            format.standard.mipmaps
                        );
                        if data.len() != expected {
                            event!(ERROR, "INPUT {} != {expected}", data.len());
                        }

                        let dx = DXTImage::new(
                            format.dxgi_format,
                            format.standard.width,
                            format.standard.height,
                            format.array_size,
                            format.standard.mipmaps,
                            data,
                        )
                        .log_failure_as("DXTImage::new")?
                        .map_if(format.dxgi_format.is_compressed(), DXTImage::decompress)
                        .log_failure_as("DXImage::decompress")?;
                        let image_size = dx.len();

                        registry.update_header(&header);
                        let id = registry.update_format(format, Some(file));
                        let format = registry.get(id);

                        let expected_size = format.expected_standard_buffer_size_with_mips();
                        if image_size != expected_size {
                            event!(ERROR, "OUTPUT {image_size} != {expected_size}");
                            // if image_size == expected_size * 4 {
                            //     format.planes = ColorPlanes::Luma;
                            // }
                        }
                    }
                }
            }
        }
    }

    registry.save()?;

    Ok(())
}
