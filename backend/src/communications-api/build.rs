use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-evidence-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communications/v1/evidence.proto"],
            &["proto"],
        )
        .expect("communications evidence protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor).expect("communications evidence descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_evidence_schema.rs"),
        format!("pub const COMMUNICATION_EVIDENCE_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("communications evidence schema digest must be written");

    let query_descriptor = output.join("communications-query-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&query_descriptor)
        .compile_protos(
            &["proto/makosh/communications/query/v1/query.proto"],
            &["proto"],
        )
        .expect("communications query protocol must compile");
    let query_digest: [u8; 32] = Sha256::digest(
        std::fs::read(&query_descriptor).expect("communications query descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join("communications_query_schema.rs"),
        format!("pub const COMMUNICATIONS_QUERY_SCHEMA_SHA256: [u8; 32] = {query_digest:?};\n"),
    )
    .expect("communications query schema digest must be written");
}
