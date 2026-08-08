use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-export-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communications_export/v1/export.proto"],
            &["proto"],
        )
        .expect("Communications export protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("export descriptor must exist")).into();
    std::fs::write(
        output.join("communications_export_schema.rs"),
        format!("pub const COMMUNICATIONS_EXPORT_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("Communications export schema digest must be written");
}
