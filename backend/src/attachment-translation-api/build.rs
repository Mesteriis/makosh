use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let control_descriptor = output.join("attachment-translation-control-v1.bin");
    let read_descriptor = output.join("attachment-translation-read-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&control_descriptor)
        .compile_protos(
            &["proto/makosh/attachment_translation/v1/translation.proto"],
            &["proto"],
        )
        .expect("Attachment Translation control protocol must compile");
    prost_build::Config::new()
        .file_descriptor_set_path(&read_descriptor)
        .compile_protos(
            &["proto/makosh/attachment_translation/read/v1/read.proto"],
            &["proto"],
        )
        .expect("Attachment Translation read protocol must compile");
    write_digest(
        &output,
        "attachment_translation_control_schema.rs",
        "ATTACHMENT_TRANSLATION_CONTROL_SCHEMA_SHA256",
        &control_descriptor,
    );
    write_digest(
        &output,
        "attachment_translation_read_schema.rs",
        "ATTACHMENT_TRANSLATION_READ_SCHEMA_SHA256",
        &read_descriptor,
    );
}

fn write_digest(
    output: &std::path::Path,
    file_name: &str,
    constant: &str,
    descriptor: &std::path::Path,
) {
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join(file_name),
        format!("pub const {constant}: [u8; 32] = {digest:?};\n"),
    )
    .expect("Attachment Translation schema digest must be written");
}
