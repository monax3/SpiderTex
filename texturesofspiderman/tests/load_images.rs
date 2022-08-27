use image::ImageFormat;
use texturesforspiderman::dxtex::DXImage;
use texturesforspiderman::prelude::*;

#[test]
fn load_images() -> Result<()> {
    const TEST_IN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/export/textures_gzumberge_gz_flag_gz_flag_rainbow_01_c.png"
    );
    const TEST_RGB: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/rgb.png");
    const TEST_LUMA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/luma.png");
    const TEST_LUMA_ALPHA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/luma_alpha.png");

    texturesforspiderman::util::log_for_tests(true);

    {
        let span = span!(TRACE, "rgb", file = TEST_RGB);
        let _entered = span.enter();

        image::open(TEST_IN)?.into_rgb8().save(TEST_RGB)?;
        let metadata = dxtex::metadata(TEST_RGB).log_failure_as("rgb metadata")?;
        event!(DEBUG, ?metadata, format_file = %metadata.format.display());
        let image = DXImage::load(TEST_RGB).log_failure_as("rgb load")?;
        let metadata = image.metadata()?;
        event!(DEBUG, ?metadata, format_image = %metadata.format.display(), size = image.len());

        let clone = image.clone();
        let metadata_clone = clone.metadata()?;
        assert_eq!(metadata, metadata_clone);

        event!(DEBUG, "Clone successful");

        image.save(
            0,
            ImageFormat::Png,
            concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/rgb.png1"),
        )?;
        std::mem::drop(image);
        clone.save(
            0,
            ImageFormat::Png,
            concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/rgb.png2"),
        )?;

        event!(DEBUG, "Clones saved");
    }

    {
        let span = span!(TRACE, "rgb", file = TEST_LUMA);
        let _entered = span.enter();

        image::open(TEST_IN)?.into_luma8().save(TEST_LUMA)?;
        let metadata = dxtex::metadata(TEST_LUMA).log_failure_as("luma metadata")?;
        event!(DEBUG, ?metadata, format_file = %metadata.format.display());
        let image = DXImage::load(TEST_LUMA).log_failure_as("luma load")?;
        let metadata = image.metadata()?;
        event!(DEBUG, ?metadata, format_image = %metadata.format.display(), size = image.len());
    }

    {
        let span = span!(TRACE, "rgb", file = TEST_LUMA_ALPHA);
        let _entered = span.enter();

        image::open(TEST_IN)?
            .into_luma_alpha8()
            .save(TEST_LUMA_ALPHA)?;
        let metadata = dxtex::metadata(TEST_LUMA_ALPHA).log_failure_as("luma alpha metadata")?;
        event!(DEBUG, ?metadata, format_file = %metadata.format.display());
        let image = DXImage::load(TEST_LUMA_ALPHA).log_failure_as("luma alpha load")?;
        let metadata = image.metadata()?;
        event!(DEBUG, ?metadata, format_image = %metadata.format.display(), size = image.len());
    }

    Ok(())
}
