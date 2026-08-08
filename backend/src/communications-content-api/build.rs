use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));

    compile_contract(
        &output,
        "proto/makosh/communications/content/ticket/v1/ticket.proto",
        "communications-content-ticket-v1.bin",
        "COMMUNICATIONS_CONTENT_TICKET_SCHEMA_SHA256",
        "communications_content_ticket_schema.rs",
    );
    compile_contract(
        &output,
        "proto/makosh/communications/content/read/v1/read.proto",
        "communications-content-read-v1.bin",
        "COMMUNICATIONS_CONTENT_READ_SCHEMA_SHA256",
        "communications_content_read_schema.rs",
    );
}

fn compile_contract(
    output: &std::path::Path,
    proto: &str,
    descriptor_name: &str,
    digest_name: &str,
    digest_file: &str,
) {
    let descriptor = output.join(descriptor_name);
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(&[proto], &["proto"])
        .expect("Communications content protocol must compile");
    let digest: [u8; 32] = Sha256::digest(
        std::fs::read(&descriptor).expect("Communications content descriptor must exist"),
    )
    .into();
    std::fs::write(
        output.join(digest_file),
        format!("pub const {digest_name}: [u8; 32] = {digest:?};\n"),
    )
    .expect("Communications content schema digest must be written");
}
