use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communication-explanation-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communication_explanation/v1/explanation.proto"],
            &["proto"],
        )
        .expect("Communication explanation protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("communication_explanation_schema.rs"),
        format!("pub const COMMUNICATION_EXPLANATION_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("Communication explanation schema digest must be written");
}
