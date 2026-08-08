fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    let output_directory =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR must be available"));
    let mut provider_config = prost_build::Config::new();
    provider_config.file_descriptor_set_path(output_directory.join("makosh.zulip.v1.bin"));
    provider_config
        .compile_protos(&["proto/makosh/zulip/v1/client.proto"], &["proto"])
        .expect("Zulip client protocol must compile");

    let mut operational_config = prost_build::Config::new();
    operational_config
        .file_descriptor_set_path(output_directory.join("makosh.zulip.operational.v1.bin"));
    operational_config.extern_path(".makosh.zulip.account.v1", "crate::account_wire_generated");
    operational_config
        .compile_protos(
            &["proto/makosh/zulip/operational/v1/client.proto"],
            &["proto"],
        )
        .expect("Zulip operational client protocol must compile");

    let mut realtime_config = prost_build::Config::new();
    realtime_config.file_descriptor_set_path(
        output_directory.join("makosh.zulip.operational.realtime.v1.bin"),
    );
    realtime_config.extern_path(
        ".makosh.zulip.operational.v1",
        "crate::operational_wire_generated",
    );
    realtime_config
        .compile_protos(
            &["proto/makosh/zulip/operational/realtime/v1/client.proto"],
            &["proto"],
        )
        .expect("Zulip operational realtime client protocol must compile");

    let mut account_config = prost_build::Config::new();
    account_config.file_descriptor_set_path(output_directory.join("makosh.zulip.account.v1.bin"));
    account_config
        .compile_protos(&["proto/makosh/zulip/account/v1/client.proto"], &["proto"])
        .expect("Zulip account lifecycle client protocol must compile");
}
