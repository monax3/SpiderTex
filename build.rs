use std::env;

fn main() {
    const LCID_EN_US: u16 = 0x0409;

    let _library = vcpkg::find_package("directxtex").unwrap();

    println!(r"cargo:rustc-link-search={}", env!("CARGO_MANIFEST_DIR"));
    println!(r"cargo:rustc-link-lib=static=DXTexWrapper");

    let mut res = winres::WindowsResource::new();

    res.set("CompanyName", &env::var("CARGO_PKG_AUTHORS").unwrap())
        .set("LegalCopyright", &env::var("CARGO_PKG_LICENSE").unwrap())
        .set_language(LCID_EN_US);

    res.set_manifest(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/app_manifest.xml"
    )));

    res.set_icon("spidertex32.ico");

    res.compile().expect("Failed to compile resources");
}
