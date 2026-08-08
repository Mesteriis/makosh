use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-saved-search-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communications/saved_search/v1/saved_search.proto"],
            &["proto"],
        )
        .expect("Communications saved-search protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor).expect("Communications saved-search descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_saved_search_schema.rs"),
        format!("pub const COMMUNICATIONS_SAVED_SEARCH_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("Communications saved-search schema digest must be written");
}
