# TDLib Runtime Resources

This directory is packaged into the Tauri bundle as `$RESOURCES/tdlib/`.

Release builds for macOS must place `libtdjson.dylib` in one of these generated
directories before `tauri build`:

- `macos-arm64/libtdjson.dylib`
- `macos-x64/libtdjson.dylib`
- `macos-universal/libtdjson.dylib`

Use `make tdlib-macos-resource` to populate the matching directory from
`MAKOSH_TDJSON_SOURCE`, `MAKOSH_TDJSON_PATH`, or an installed Homebrew `tdlib`.
For release CI, `MAKOSH_TDLIB_BUILD_FROM_SOURCE=1 make tdlib-macos-resource`
can build TDLib from source before copying the generated dynamic library.
Generated dynamic libraries are ignored by Git.

Linux is development-container-only for this project and is not packaged as a
desktop TDLib resource.
