pub mod noninteractive;
pub mod widgets {
    mod log;
    pub use log::LogWidget;
}
pub use texturesofspiderman::prelude::*;

#[cfg(disabled)]
pub fn icon_wic() -> Result<IconData> {
    use windows_imaging::prelude::*;

    let icon = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/SpiderTex.ico"));

    let wic = windows_imaging::wic()?;
    let image = windows_imaging::container_from_memory(&wic, WICContainer::Ico, icon)?;
    let image = image.convert_to_pixel_format(&wic, WICPixelFormat::RGBA32bpp)?;
    let rect = image.rect()?;

    Ok(IconData {
        width: rect.Width as u32,
        height: rect.Height as u32,
        rgba: image.pixels().log_failure_as("pixels")?
    })
}

#[cfg(disabled)]
pub fn icon_directxtex() -> Result<IconData> {
    let icon = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/SpiderTex.png"));

    let image = directxtex::load_wic_from_memory(icon)?.into_format(directxtex::dxgi_format::RGBA8)?;
    let metadata = image.metadata()?;

    // FIXME: sanity checking for directxtex
    debug_assert_eq!(metadata.format, dxgi_format::RGBA8);

    Ok(IconData {
        width: metadata.width as u32,
        height: metadata.height as u32,
        rgba: image.image(0)?
    })
}
