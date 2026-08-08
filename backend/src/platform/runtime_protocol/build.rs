fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc must be available");
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    let mut config = prost_build::Config::new();
    // These oneofs are generated wire representations. Boxing one variant
    // would be a source-breaking contract change, so scope the lint exception
    // to the two generated enums rather than the crate or consumer code.
    for operation in [
        ".makosh.runtime.v1.SchedulerRuntimeControlRequestV1.operation",
        ".makosh.runtime.v1.ManagedVaultRuntimeControlRequestV1.operation",
    ] {
        config.enum_attribute(operation, "#[allow(clippy::large_enum_variant)]");
    }
    config
        .compile_protos(
            &[
                "proto/makosh/runtime/v1/recovery.proto",
                "proto/makosh/runtime/v1/module_client.proto",
                "proto/makosh/runtime/v1/deployment.proto",
                "proto/makosh/runtime/v1/distribution.proto",
                "proto/makosh/runtime/v1/telemetry_runtime.proto",
                "proto/makosh/runtime/v1/blob_runtime.proto",
                "proto/makosh/runtime/v1/integration_host_bridge.proto",
                "proto/makosh/runtime/v1/events_authority_runtime.proto",
                "proto/makosh/runtime/v1/scheduler_runtime.proto",
                "proto/makosh/runtime/v1/managed_storage_runtime.proto",
                "proto/makosh/runtime/v1/managed_runtime_artifact.proto",
                "proto/makosh/runtime/v1/managed_domain_runtime.proto",
                "proto/makosh/runtime/v1/managed_engine_runtime.proto",
                "proto/makosh/runtime/v1/managed_workflow_runtime.proto",
                "proto/makosh/runtime/v1/managed_integration_runtime.proto",
                "proto/makosh/runtime/v1/vault_runtime.proto",
                "proto/makosh/runtime/v1/managed_runtime_control.proto",
            ],
            &["proto"],
        )
        .expect("runtime recovery protocol must compile");
}
