fn main() {
    #[cfg(feature = "vcpkg")]
    let _library = vcpkg::find_package("directxtex").unwrap();
}
