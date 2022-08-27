use camino::Utf8Path;
use spidertexlib::prelude::*;
use spidertexlib::util::log_for_tests;
use std::collections::BTreeSet;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

#[test]
#[cfg(windows)]
fn crashing_bug_check_files() -> Result<()> {
    const TEST_SRC: &str = r"C:\Users\PalateSwap\spider-modding\posm";
    const TEST_DST: &str = r"C:\Users\PalateSwap\spider-modding\posm - test";

    log_for_tests(true);
    spidertex_lib::util::initialize_com()?;
    registry::load()?;

    let test_src = Utf8Path::new(TEST_SRC);
    let test_dst = Utf8Path::new(TEST_DST);

    if test_dst.exists() {
        std::fs::remove_dir_all(test_dst)?;
    }
    std::fs::create_dir_all(test_dst)?;

    let mut inputs = Vec::new();

    for src_file in test_src
        .read_dir_utf8()?
        .filter_map(Result::ok)
        .filter(|file| {
            file.path()
                .extension()
                .map_or(false, |ext| ext == "texture" || ext == "raw")
        })
    {
        let src_file = src_file.path();

        if let Some(file_name) = src_file.file_name() {
            let dst_file = test_dst.with_file_name(file_name);
            event!(TRACE, "Copying {src_file} to {dst_file}");
            std::fs::copy(&src_file, &dst_file)?;
            inputs.push(dst_file);
        }
    }

    for texture_file in inputs {
        let (header, _): (Option<texture_file::FormatHeader>, _) =
            texture_file::read_header(&texture_file)?;
        if let Some(header) = header {
            let format: TextureFormat = header.to();
            header.check(Some(&format));
            event!(DEBUG, header.stex_format, header.planes, %format);
        }
    }
    Ok(())
}

#[test]
fn crashing_bug_check_formats() -> Result<()> {
    log_for_tests(true);
    registry::load()?;
    let registry = registry();
    let mut formats = BTreeSet::new();
    let mut changes = BTreeSet::<String>::new();

    for (&id, &format) in &registry.formats {
        let raw_headers = registry::raw_header(id).expect("No raw header");
        let header = texture_file::FormatHeader::from_hexstring(&raw_headers)?;
        let format_from_raw = header.to();

        // header.check(Some(&format));
        // check_values(&header, &mut changes);
        // formats.insert((format.dxgi_format.0, header.planes, header.stex_format, (header.unk1, header.unk2, header.unk3, header.unk4)));

        let mut flags_str: BTreeSet<&str> = BTreeSet::new();
        let mut flags = header.stex_format;
        let flags_prev = flags;
        if flags & (1 << 0) != 0 { flags_str.insert("sRGB"); flags &= !(1 << 0); }
        if flags & (1 << 2) != 0 { flags_str.insert("normal"); flags &= !(1 << 2); }
        if flags & (1 << 3) != 0 { flags_str.insert("gradient"); flags &= !(1 << 3); }
        if flags & (1 << 4) != 0 { flags_str.insert("RGB8"); flags &= !(1 << 4); }
        if flags & (1 << 5) != 0 { flags_str.insert("LUT"); flags &= !(1 << 5); }
        if flags & (1 << 6) != 0 { flags_str.insert("skybox"); flags &= !(1 << 6); }

        formats.insert(format!("{flags:08b} {flags_str:?} {flags_prev:02x} {:02x} {} {:?} {format}", header.planes, header.array_size, (header.unk1, header.unk2, header.unk3, header.unk4)));
        if flags & (1 << 2) != 0 {
            event!(DEBUG, flags = format!("{flags:08b}"), %format, planes = header.planes, unks = ?(header.unk1, header.unk2, header.unk3, header.unk4));
        }
        // event!(DEBUG, %format, stex = header.stex_format, planes = header.planes, unks = ?(header.unk1, header.unk2, header.unk3, header.unk4));
        assert_eq!(format, format_from_raw);
    }

    for format in formats.into_iter() {
        event!(INFO, %format);
    }

    // for (dxgi_format, planes, stex, unks) in formats.into_iter() {
    //     let dxgi_format = DXGI_FORMAT(dxgi_format);
    //     event!(INFO, dxgi_format = %dxgi_format.display(), stex, planes, ?unks);
    // }

    // for change in changes.into_iter() {
    //     event!(INFO, %change);
    // }
    Ok(())
}

fn check_values(header: &texture_file::FormatHeader, changes: &mut BTreeSet<String>) {
    if header.stex_format != 0 {
        changes.insert("STEX".to_string());
    }

    if header.planes != 0 {
        changes.insert("PLANES".to_string());
    }

    if header.unk1 != 0 { changes.insert("FMT_UNK1".to_string()); }
    if header.unk2 != 0 { changes.insert("FMT_UNK2".to_string()); }
    if header.unk3 != 0 { changes.insert("FMT_UNK3".to_string()); }
    if header.unk4 != 0 { changes.insert("FMT_UNK4".to_string()); }

    let zeroes = header.zeroes1.iter().chain(header.zeroes2.iter()).enumerate();

    for (i, &zero) in zeroes {
        if zero != 0 {
            changes.insert(format!("FMT_ZERO{}", i + 1));
        }
    }
}
