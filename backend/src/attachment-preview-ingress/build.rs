use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("attachment-preview-ingress-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/attachment_preview/ingress/v1/custody_delegation.proto"],
            &["proto"],
        )
        .expect("Attachment Preview ingress protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("attachment_preview_ingress_schema.rs"),
        format!("pub const ATTACHMENT_PREVIEW_INGRESS_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("Attachment Preview ingress schema digest must be written");
}
