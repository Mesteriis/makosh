use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-call-evidence-client-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communications/call_evidence/client/v1/client.proto"],
            &["proto"],
        )
        .expect("Communications call evidence client protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("communications_call_evidence_client_schema.rs"),
        format!("pub const CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("Communications call evidence client schema digest must be written");
}
