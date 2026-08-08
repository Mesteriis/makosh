use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let descriptor = output.join("call-transcription-ingress-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/call_transcription/ingress/v1/recording.proto"],
            &["proto"],
        )
        .expect("call transcription ingress protocol");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&descriptor).expect("descriptor")).into();
    std::fs::write(
        output.join("call_transcription_ingress_schema.rs"),
        format!("pub const CALL_TRANSCRIPTION_INGRESS_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("schema digest");
}
