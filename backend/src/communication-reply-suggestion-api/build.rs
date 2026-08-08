use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communication-reply-suggestion-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communication_reply_suggestion/v1/reply_suggestion.proto"],
            &["proto"],
        )
        .expect("Communication reply suggestion protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("communication_reply_suggestion_schema.rs"),
        format!("pub const COMMUNICATION_REPLY_SUGGESTION_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("Communication reply suggestion schema digest must be written");
}
