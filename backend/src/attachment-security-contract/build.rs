use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("attachment-security-scan-candidate-observed-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/attachment_security/v1/scan_candidate.proto"],
            &["proto"],
        )
        .expect("Attachment Security scan-candidate protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("scan-candidate descriptor must exist"))
            .into();
    std::fs::write(
        output.join("attachment_security_scan_candidate_schema.rs"),
        format!(
            "pub const ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"
        ),
    )
    .expect("scan-candidate schema digest must be written");
}
