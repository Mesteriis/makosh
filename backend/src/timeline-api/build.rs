use sha2::{Digest, Sha256};
fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let descriptor = out.join("timeline-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(&["proto/makosh/timeline/v1/timeline.proto"], &["proto"])
        .expect("timeline proto");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&descriptor).expect("descriptor")).into();
    std::fs::write(
        out.join("timeline_schema.rs"),
        format!("pub const TIMELINE_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("digest");
}
