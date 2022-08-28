fn main() {
    // #[cfg(feature = "vcpkg")]
    let _library = vcpkg::find_package("directxtex").unwrap();

    println!(r"cargo:rustc-link-search={}", env!("CARGO_MANIFEST_DIR"));
    println!(r"cargo:rustc-link-lib=static=DXTexWrapper");
    println!("cargo:rerun-if-changed=DXTexWrapper.lib");
}
