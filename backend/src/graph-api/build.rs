use sha2::{Digest, Sha256};
fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    unsafe { std::env::set_var("PROTOC", protoc) }
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let descriptor = out.join("graph-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(&["proto/makosh/graph/v1/graph.proto"], &["proto"])
        .unwrap();
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&descriptor).unwrap()).into();
    std::fs::write(
        out.join("graph_schema.rs"),
        format!("pub const GRAPH_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .unwrap();
}
