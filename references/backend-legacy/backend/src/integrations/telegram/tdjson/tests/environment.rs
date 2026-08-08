use std::path::Path;

#[cfg(target_os = "macos")]
#[test]
fn macos_tdjson_candidates_prefer_bundled_tauri_resources() {
    let exe_dir = Path::new("/Applications/Макошь.app/Contents/MacOS");
    let cwd = Path::new("/workspace/makosh");
    let candidates =
        super::super::tdjson_library_candidates_with_context(None, Some(exe_dir), Some(cwd));
    let bundled_resource = Path::new("/Applications/Макошь.app/Contents/Resources")
        .join("tdlib")
        .join(super::super::tdjson_platform_dir())
        .join("libtdjson.dylib");
    let dev_resource = cwd
        .join("frontend/src-tauri/resources/tdlib")
        .join(super::super::tdjson_platform_dir())
        .join("libtdjson.dylib");

    assert_eq!(candidates.first(), Some(&bundled_resource));
    assert!(candidates.contains(&dev_resource));
    assert!(
        candidates
            .iter()
            .position(|candidate| candidate == &bundled_resource)
            < candidates
                .iter()
                .position(|candidate| candidate == Path::new("/opt/homebrew/lib/libtdjson.dylib"))
    );
}

#[test]
fn renders_tdlib_qr_link_as_svg() {
    let svg = super::super::render_qr_svg("tg://login?token=test-token").expect("QR SVG");

    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.len() > 100);
}
