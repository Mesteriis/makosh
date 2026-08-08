use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("review-note-candidate-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/review/note_candidate/v1/note_candidate.proto"],
            &["proto"],
        )
        .expect("Review note-candidate protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("review_note_candidate_schema.rs"),
        format!("pub const REVIEW_NOTE_CANDIDATE_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("Review note-candidate schema digest must be written");
}
