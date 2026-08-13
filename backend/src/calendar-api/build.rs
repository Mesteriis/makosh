use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("calendar-client-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/calendar/client/v1/calendar.proto"],
            &["proto"],
        )
        .expect("Calendar client protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("calendar_client_schema.rs"),
        format!("pub const CALENDAR_CLIENT_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("Calendar client schema digest must be written");
}
