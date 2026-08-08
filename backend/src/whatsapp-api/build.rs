fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    let output_directory =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR must be available"));

    let mut provider_config = prost_build::Config::new();
    provider_config.file_descriptor_set_path(output_directory.join("makosh.whatsapp.v1.bin"));
    provider_config
        .boxed(".makosh.whatsapp.v1.WhatsAppHostBridgeOperationV1.operation.observation");
    provider_config
        .compile_protos(&["proto/makosh/whatsapp/v1/client.proto"], &["proto"])
        .expect("WhatsApp client protocol must compile");

    let mut operational_config = prost_build::Config::new();
    operational_config
        .file_descriptor_set_path(output_directory.join("makosh.whatsapp.operational.v1.bin"));
    operational_config.extern_path(".makosh.whatsapp.v1", "crate::wire");
    operational_config
        .compile_protos(
            &["proto/makosh/whatsapp/operational/v1/client.proto"],
            &["proto"],
        )
        .expect("WhatsApp operational client protocol must compile");

    let mut realtime_config = prost_build::Config::new();
    realtime_config.file_descriptor_set_path(
        output_directory.join("makosh.whatsapp.operational.realtime.v1.bin"),
    );
    realtime_config.extern_path(".makosh.whatsapp.v1", "crate::wire");
    realtime_config
        .compile_protos(
            &["proto/makosh/whatsapp/operational/realtime/v1/client.proto"],
            &["proto"],
        )
        .expect("WhatsApp operational realtime client protocol must compile");
}
