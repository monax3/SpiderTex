use camino::Utf8Path;
use spidertexlib::files::Categorized;
use spidertexlib::inputs::Inputs;
use spidertexlib::prelude::*;

#[test]
fn clean_testdata() -> Result<()> {
    const TESTDATA_IMAGES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/import");
    const TESTDATA_TEXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/export");
    assert_ne!(TESTDATA_IMAGES, TESTDATA_TEXTURES); // better late than never

    let testdata_textures = Utf8Path::new(TESTDATA_TEXTURES);
    let testdata_images = Utf8Path::new(TESTDATA_IMAGES);

    spidertexlib::util::log_for_tests(true);

    let Inputs { textures, .. } = spidertexlib::inputs::gather(testdata_images);
    for Categorized { files, .. } in textures {
        for file in files {
            if file.as_str().contains(".custom.") {
                event!(TRACE, "Removing old import {file}");
                std::fs::remove_file(file)?;
            }
        }
    }

    let Inputs { images, textures } = spidertexlib::inputs::gather(testdata_textures);
    for Categorized { files, .. } in images {
        for file in files {
            event!(TRACE, "Removing old export {file}");
            std::fs::rename(&file, testdata_images.join(file.file_name().unwrap()))?;
        }
    }
    for Categorized { files, .. } in textures {
        for file in files {
            let in_images = testdata_images.join(file.file_name().unwrap());
            if !in_images.exists() {

            }
            event!(TRACE, "Copying {file} to {testdata_images}");
            std::fs::copy(&file, &in_images)?;
        }
    }

    Ok(())
}
