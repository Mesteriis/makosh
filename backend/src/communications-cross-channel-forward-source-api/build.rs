use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-cross-channel-forward-source-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &[
                "proto/makosh/communications/cross_channel_forward_source/v1/cross_channel_forward_source.proto",
            ],
            &["proto"],
        )
        .expect("Communications cross-channel forward source protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor)
            .expect("Communications cross-channel forward source descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_cross_channel_forward_source_schema.rs"),
        format!(
            "pub const COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"
        ),
    )
    .expect("Communications cross-channel forward source schema digest must be written");
}
