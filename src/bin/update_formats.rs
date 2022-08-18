use color_eyre::Result;
use camino::Utf8Path;

const TESTDATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata");

fn main() -> Result<()> {
    use tracing_subscriber::prelude::*;

    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time().with_target(false))
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_default(tracing::Level::INFO)
                .with_target(env!("CARGO_CRATE_NAME"), tracing::Level::TRACE),
        )
    .init();

    let testdir = Utf8Path::new(TESTDATA);

    for file in testdir.read_dir_utf8()?.filter_map(Result::ok) {
        let file = file.path();
        if file.extension().map_or(false, |ext| ext == "texture") && !file.file_name().unwrap().contains("_hd.") {
            let len = file.metadata().unwrap().len() as usize;

            if let Some(fmt) = spidertexlib::formats::guess_format(len) {
                println!("{} => {}", file, fmt);
            } else {
                println!("{} => FAILED", file);
                if let Err(error) = spidertexlib::headers::read_texture_header(file.as_std_path()) {
                    println!("{file}: {error}");
                }
            }
        }
    }

    Ok(())
}
