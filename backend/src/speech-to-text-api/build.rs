use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("speech-to-text-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/speech_to_text/v1/speech_to_text.proto"],
            &["proto"],
        )
        .expect("Speech-to-Text protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("speech_to_text_schema.rs"),
        format!("pub const SPEECH_TO_TEXT_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("Speech-to-Text schema digest must be written");
}
