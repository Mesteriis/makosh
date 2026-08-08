fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    let descriptor_set =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR must be available"))
            .join("makosh.telegram.automation.v1.bin");
    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(descriptor_set);
    config
        .compile_protos(
            &["proto/makosh/telegram/automation/v1/automation.proto"],
            &["proto"],
        )
        .expect("Telegram automation protocol must compile");
}
