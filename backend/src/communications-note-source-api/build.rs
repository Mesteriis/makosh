use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("communications-note-source-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/communications/note_source/v1/note_source.proto"],
            &["proto"],
        )
        .expect("Communications note source protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("communications_note_source_schema.rs"),
        format!("pub const COMMUNICATIONS_NOTE_SOURCE_SCHEMA_SHA256: [u8; 32] = {digest:?};\n"),
    )
    .expect("Communications note source schema digest must be written");
}
