use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("attachment-archive-inspection-ingress-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/attachment_archive_inspection/ingress/v1/custody_delegation.proto"],
            &["proto"],
        )
        .expect("Attachment Archive Inspection ingress protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor)
            .expect("Attachment Archive Inspection ingress descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("attachment_archive_inspection_ingress_schema.rs"),
        format!(
            "pub const ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"
        ),
    )
    .expect("Attachment Archive Inspection ingress schema digest must be written");
}
