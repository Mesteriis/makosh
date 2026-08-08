use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("retained-evidence-replay-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/events/replay/v1/retained_evidence_replay.proto"],
            &["proto"],
        )
        .expect("retained evidence replay protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor).expect("retained evidence replay descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("retained_evidence_replay_schema.rs"),
        format!("pub const RETAINED_EVIDENCE_REPLAY_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("retained evidence replay schema digest must be written");
}
