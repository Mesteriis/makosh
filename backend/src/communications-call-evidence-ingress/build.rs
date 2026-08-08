use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-call-evidence-ingress-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communications/call_evidence/v1/call_evidence.proto"],
            &["proto"],
        )
        .expect("Communications call evidence ingress protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor)
            .expect("Communications call evidence ingress descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_call_evidence_ingress_schema.rs"),
        format!(
            "pub const COMMUNICATIONS_CALL_EVIDENCE_INGRESS_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"
        ),
    )
    .expect("Communications call evidence ingress schema digest must be written");
}
