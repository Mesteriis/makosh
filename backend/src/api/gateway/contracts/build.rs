fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    let mut config = prost_build::Config::new();
    config.extern_path(".makosh.runtime.v1", "::makosh_runtime_protocol::v1");
    config.enum_attribute(
        ".makosh.gateway.v1.ExternalRuntimeSessionRequestV1.operation",
        "#[allow(clippy::large_enum_variant)]",
    );
    config
        .compile_protos(
            &[
                "proto/makosh/gateway/v1/recovery.proto",
                "proto/makosh/gateway/v1/owner_control.proto",
                "proto/makosh/gateway/v1/module_registration.proto",
                "proto/makosh/gateway/v1/external_runtime_session.proto",
                "proto/makosh/gateway/v1/client_realtime.proto",
                "proto/makosh/gateway/v1/client_system_status_realtime.proto",
                "proto/makosh/gateway/v1/browser_session.proto",
                "proto/makosh/gateway/v1/client_bootstrap.proto",
                "proto/makosh/gateway/v1/owner_vault_provisioning.proto",
                "proto/makosh/gateway/v1/owner_module_settings.proto",
            ],
            &["proto", "../../../platform/runtime_protocol/proto"],
        )
        .expect("gateway protocol must compile");
}
