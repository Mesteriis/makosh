use sha2::{Digest, Sha256};
fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    unsafe { std::env::set_var("PROTOC", protoc) }
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let descriptor = out.join("consistency-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/consistency/v1/consistency.proto"],
            &["proto"],
        )
        .unwrap();
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&descriptor).unwrap()).into();
    std::fs::write(
        out.join("consistency_schema.rs"),
        format!("pub const CONSISTENCY_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .unwrap();
}
