use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("knowledge-command-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/knowledge/command/v1/knowledge_command.proto"],
            &["proto"],
        )
        .expect("Knowledge command protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("knowledge_command_schema.rs"),
        format!("pub const KNOWLEDGE_COMMAND_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("Knowledge command schema digest must be written");

    let client_descriptor = output.join("knowledge-client-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&client_descriptor)
        .compile_protos(
            &["proto/makosh/knowledge/client/v1/knowledge.proto"],
            &["proto"],
        )
        .expect("Knowledge client protocol must compile");
    let client_digest: [u8; 32] =
        Sha256::digest(std::fs::read(&client_descriptor).expect("client descriptor must exist"))
            .into();
    std::fs::write(
        output.join("knowledge_client_schema.rs"),
        format!("pub const KNOWLEDGE_CLIENT_SCHEMA_SHA256_V1: [u8; 32] = {client_digest:?};\n"),
    )
    .expect("Knowledge client schema digest must be written");
}
