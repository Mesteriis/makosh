use std::{env, fs, path::PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    let proto = "proto/makosh/documents/client/v1/documents.proto";
    println!("cargo:rerun-if-changed={proto}");
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let descriptor =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("documents-client-v1.bin");
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc);
    config.file_descriptor_set_path(&descriptor);
    config
        .compile_protos(&[proto], &["proto"])
        .expect("documents proto");
    let digest: [u8; 32] = Sha256::digest(fs::read(&descriptor).expect("descriptor bytes")).into();
    fs::write(
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("documents_client_schema.rs"),
        format!("pub const DOCUMENTS_CLIENT_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("schema digest");
}
