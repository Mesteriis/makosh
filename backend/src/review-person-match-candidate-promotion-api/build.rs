use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must exist");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let descriptor = out.join("review-person-match-candidate-promotion-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/review/person_match_candidate/promotion/v1/promotion.proto"],
            &["proto"],
        )
        .expect("Review person-match promotion protocol must compile");
    let digest: [u8; 32] = Sha256::digest(std::fs::read(&descriptor).expect("descriptor")).into();
    std::fs::write(
        out.join("review_person_match_candidate_promotion_schema.rs"),
        format!("pub const REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("schema digest");
}
