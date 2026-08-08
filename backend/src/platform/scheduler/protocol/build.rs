fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR must be set"))
            .join("makosh.scheduler.v1.bin"),
    );
    config
        .compile_protos(&["proto/makosh/scheduler/v1/job_command.proto"], &["proto"])
        .expect("scheduler job protocol must compile");
}
