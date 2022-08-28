pub fn log() {
    use tracing_subscriber::EnvFilter;
    use tracing::metadata::LevelFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy())
        .without_time()
        .with_line_number(true)
        .with_file(true)
        .init();
}
