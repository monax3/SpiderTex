
#[cfg(disabled)]
impl ImageInfo {
    pub fn from_file(format: &TextureFormat, file: &Utf8Path) -> Result<Self> {
        let ext = file.extension().ok_or(Error::Internal)?;

        match ext {
            ext if ext.eq_ignore_ascii_case("dds") => todo!(),
            ext if ext.eq_ignore_ascii_case("hdr") => todo!(),
            _ => Self::from_imagers(format, file),
        }
    }

    pub fn from_imagers(format: &TextureFormat, file: &Utf8Path) -> Result<Self> {
        let mut warnings = Vec::new();
        let mut image = image::open(file)?;

        let correct_format = match format.planes {
            texturesforspiderman::formats::ColorPlanes::Rgba => image::ColorType::Rgba8,
            texturesforspiderman::formats::ColorPlanes::Luma => image::ColorType::L8,
            texturesforspiderman::formats::ColorPlanes::Hdr => image::ColorType::Rgba32F,
        };

        let current_format = image.color();
        let original_dimensions = imagers_dimensions(&image);

        if current_format != correct_format {
            warnings.push(
                format!(
                    "Image has the wrong color information ({} instead of {}) and maybe not \
                     import correctly.",
                    imagers_color(current_format),
                    imagers_color(correct_format)
                )
                .into(),
            );
            image = imagers_convert(image, correct_format);
        }

        if !format.aspect_ratio_matches(original_dimensions) {
            warnings
                .push("Image has the wrong aspect ratio and will be distorted if imported.".into());
        }

        let (needs_resize, _is_lowres) = format.is_correct_size(original_dimensions);
        if needs_resize {
            warnings.push(
                format!(
                    "Image is the wrong size ({}x{} instead of {}x{}) and will be resized.",
                    original_dimensions.width,
                    original_dimensions.height,
                    format.preferred_width(),
                    format.preferred_height()
                )
                .into(),
            );

            image = image.resize_exact(
                format.preferred_width() as u32,
                format.preferred_height() as u32,
                texturesforspiderman::IMAGERS_RESIZE_FILTER,
            );
        }

        let dimensions = imagers_dimensions(&image);
        let data = image.into_bytes();

        Ok(Self {
            dimensions,
            data,
            warnings,
        })
    }
}

pub fn imagers_dimensions(image: &DynamicImage) -> Dimensions {
    Dimensions {
        data_size: image.as_bytes().len(),
        width:     image.width() as usize,
        height:    image.height() as usize,
        mipmaps:   1,
    }
}

pub fn imagers_convert(image: DynamicImage, color: image::ColorType) -> DynamicImage {
    match color {
        image::ColorType::L8 => image.into_luma8().into(),
        image::ColorType::La8 => image.into_luma_alpha8().into(),
        image::ColorType::Rgb8 => image.into_rgb8().into(),
        image::ColorType::Rgba8 => image.into_rgba8().into(),
        image::ColorType::L16 => image.into_luma16().into(),
        image::ColorType::La16 => image.into_luma_alpha16().into(),
        image::ColorType::Rgb16 => image.into_rgb16().into(),
        image::ColorType::Rgba16 => image.into_rgba16().into(),
        image::ColorType::Rgb32F => image.into_rgb32f().into(),
        image::ColorType::Rgba32F => image.into_rgba32f().into(),
        _ => unimplemented!(),
    }
}

pub fn imagers_color(color: image::ColorType) -> Cow<'static, str> {
    Cow::Borrowed(match color {
        image::ColorType::L8 => "monochome",
        image::ColorType::Rgb8 => "24-bit color RGB",
        image::ColorType::Rgba8 => "32-bit color RGBA",
        image::ColorType::Rgb32F => "96-bit HDR RGB",
        image::ColorType::Rgba32F => "128-bit HDR RGBA",
        _ => return Cow::Owned(format!("{color:?}")),
    })
}