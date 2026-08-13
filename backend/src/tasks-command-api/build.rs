use sha2::{Digest, Sha256};

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe { std::env::set_var("PROTOC", protoc) };
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set"));
    let descriptor = output.join("tasks-command-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(
            &["proto/makosh/tasks/command/v1/tasks_command.proto"],
            &["proto"],
        )
        .expect("Tasks command protocol must compile");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&descriptor).expect("descriptor must exist")).into();
    std::fs::write(
        output.join("tasks_command_schema.rs"),
        format!("pub const TASKS_COMMAND_SCHEMA_SHA256_V1: [u8; 32] = {digest:?};\n"),
    )
    .expect("Tasks command schema digest must be written");

    let client_descriptor = output.join("tasks-client-v1.bin");
    prost_build::Config::new()
        .file_descriptor_set_path(&client_descriptor)
        .compile_protos(&["proto/makosh/tasks/client/v1/tasks.proto"], &["proto"])
        .expect("Tasks client protocol must compile");
    let client_digest: [u8; 32] =
        Sha256::digest(std::fs::read(&client_descriptor).expect("client descriptor must exist"))
            .into();
    std::fs::write(
        output.join("tasks_client_schema.rs"),
        format!("pub const TASKS_CLIENT_SCHEMA_SHA256_V1: [u8; 32] = {client_digest:?};\n"),
    )
    .expect("Tasks client schema digest must be written");
}
