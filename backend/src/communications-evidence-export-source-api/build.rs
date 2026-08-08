use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-evidence-export-source-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communications/evidence_export_source/v1/evidence_export_source.proto"],
            &["proto"],
        )
        .expect("Communications evidence-export source protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor)
            .expect("Communications evidence-export source descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_evidence_export_source_schema.rs"),
        format!(
            "pub const COMMUNICATIONS_EVIDENCE_EXPORT_SOURCE_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"
        ),
    )
    .expect("Communications evidence-export source schema digest must be written");
}
