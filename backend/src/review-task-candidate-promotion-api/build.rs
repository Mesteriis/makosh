use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("review-task-candidate-promotion-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/review/task_candidate/promotion/v1/promotion.proto"],
            &["proto"],
        )
        .expect("Review task-candidate promotion protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("review_task_candidate_promotion_schema.rs"),
        format!(
            "pub const REVIEW_TASK_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"
        ),
    )
    .expect("Review task-candidate promotion schema digest must be written");
}
