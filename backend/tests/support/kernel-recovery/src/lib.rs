#![cfg_attr(test, allow(dead_code, unused_imports))]

// This crate is a test-only composition harness that path-includes selected
// private production slices. Those slices intentionally expose more members
// than each individual recovery scenario calls; semantic Clippy lints remain
// enabled for the harness and every included source file.

#[cfg(test)]
mod distribution {
    #[path = "../../../../../src/kernel/src/distribution/bundle_verifier.rs"]
    pub(crate) mod bundle_verifier;
    #[path = "../../../../../src/kernel/src/distribution/bundled_launch.rs"]
    pub(crate) mod bundled_launch;
    #[path = "../../../../../src/kernel/src/distribution/manifest_verifier.rs"]
    pub(crate) mod manifest_verifier;
    #[path = "../../../../../src/kernel/src/distribution/runtime_dependencies.rs"]
    pub(crate) mod runtime_dependencies;
    #[path = "../../../../../src/kernel/src/distribution/staged_artifact.rs"]
    pub(crate) mod staged_artifact;
    #[path = "../../../../../src/kernel/src/distribution/staged_contracts.rs"]
    pub(crate) mod staged_contracts;
    #[path = "../../../../../src/kernel/src/distribution/trust_root.rs"]
    pub(crate) mod trust_root;
}

#[cfg(test)]
mod identity {
    #[path = "../../../../../src/kernel/src/identity/browser_gateway.rs"]
    pub(crate) mod browser_gateway;
    pub(crate) mod device;
    pub(crate) mod owner;
    #[path = "../../../../../src/kernel/src/identity/owner_control/mod.rs"]
    pub(crate) mod owner_control;
}

#[cfg(test)]
mod infrastructure {
    #[path = "../../../../../src/kernel/src/infrastructure/filesystem.rs"]
    pub(crate) mod filesystem;
    #[path = "../../../../../src/kernel/src/infrastructure/paths.rs"]
    pub(crate) mod paths;
}

#[cfg(test)]
mod modules {
    pub(crate) mod capability;
    pub(crate) mod registration;
    #[path = "../../../../../src/kernel/src/modules/settings/mod.rs"]
    pub(crate) mod settings;
}

#[cfg(test)]
mod platform {
    pub(crate) mod blob;

    #[path = "../../../../../src/kernel/src/platform/client_realtime.rs"]
    pub(crate) mod client_realtime;

    #[path = "../../../../../src/kernel/src/platform/gateway.rs"]
    pub(crate) mod gateway;

    #[path = "../../../../../src/kernel/src/platform/integration_state.rs"]
    pub(crate) mod integration_state;

    pub(crate) mod managed;
    #[path = "../../../../../src/kernel/src/platform/scheduler/mod.rs"]
    pub(crate) mod scheduler;
    pub(crate) mod macos {
        #[path = "../../../../../../src/kernel/src/platform/macos/bundled_release.rs"]
        pub(crate) mod bundled_release;
        #[path = "../../../../../../src/kernel/src/platform/macos/code_signature.rs"]
        pub(crate) mod code_signature;
        #[path = "../../../../../../src/kernel/src/platform/macos/host_bridge_descriptor.rs"]
        pub(crate) mod host_bridge_descriptor;
        #[path = "../../../../../../src/kernel/src/platform/macos/managed_launch.rs"]
        pub(crate) mod managed_launch;
        #[path = "../../../../../../src/kernel/src/platform/macos/native_launch.rs"]
        pub(crate) mod native_launch;
        #[path = "../../../../../../src/kernel/src/platform/macos/release_resources.rs"]
        pub(crate) mod release_resources;
    }

    pub(crate) mod telemetry;

    pub(crate) mod events;
    pub(crate) mod storage;
    pub(crate) mod vault {
        #[path = "../../../../../../src/kernel/src/platform/vault/binding.rs"]
        pub(crate) mod binding;
        #[path = "../../../../../../src/kernel/src/platform/vault/ciphertext_route.rs"]
        pub(crate) mod ciphertext_route;
        #[path = "../../../../../../src/kernel/src/platform/vault/launch.rs"]
        pub(crate) mod launch;
        #[path = "../../../../../../src/kernel/src/platform/vault/managed_route.rs"]
        pub(crate) mod managed_route;
        #[path = "../../../../../../src/kernel/src/platform/vault/owner_derived_key.rs"]
        pub(crate) mod owner_derived_key;
        #[path = "../../../../../../src/kernel/src/platform/vault/owner_provisioning/mod.rs"]
        pub(crate) mod owner_provisioning;
        #[path = "../../../../../../src/kernel/src/platform/vault/provider_credential.rs"]
        pub(crate) mod provider_credential;
        #[path = "../../../../../../src/kernel/src/platform/vault/status.rs"]
        pub(crate) mod status;
    }
}

#[cfg(test)]
mod control;

#[cfg(test)]
#[path = "../../../../src/kernel/src/control_store/lifecycle.rs"]
mod control_store_lifecycle;

#[cfg(test)]
mod control_store {
    pub(crate) use crate::control_store_lifecycle as lifecycle;
}

#[cfg(test)]
mod service;

#[cfg(test)]
mod transport;

#[cfg(test)]
#[path = "../../../../src/platform/storage/vault/src/lib.rs"]
pub(crate) mod vault;

#[cfg(test)]
mod storage_control;

#[cfg(test)]
mod recovery {
    #[path = "../../../../../src/kernel/src/recovery/capture_coordinator.rs"]
    pub(crate) mod capture_coordinator;
    #[path = "../../../../../src/kernel/src/recovery/control_store_media.rs"]
    pub(crate) mod control_store_media;
    #[path = "../../../../../src/kernel/src/recovery/fence.rs"]
    pub(crate) mod fence;
    #[path = "../../../../../src/kernel/src/recovery/media/mod.rs"]
    pub(crate) mod media;
    #[path = "../../../../../src/kernel/src/recovery/process_port.rs"]
    pub(crate) mod process_port;
    #[path = "../../../../../src/kernel/src/recovery/restore_coordinator.rs"]
    pub(crate) mod restore_coordinator;
}

#[cfg(test)]
mod runtime {
    pub(crate) mod external;
    pub(crate) mod lifecycle;
    pub(crate) mod managed;
}

#[cfg(test)]
mod tests {
    mod actor;
    mod blob_requests;
    mod blob_service;
    mod browser_device_identity;
    mod browser_gateway_session;
    mod bundled_artifact_proposal;
    mod capture_coordinator;
    mod client_blob_routes;
    mod client_realtime_routes;
    mod client_rpc_routes;
    mod common;
    mod control_plane_worker;
    mod control_store_media;
    mod deployment_contract;
    mod descriptor_basics;
    mod distribution_bundle_fixture;
    mod event_hub_topology_configuration;
    mod event_requests;
    mod event_topology;
    mod events_authority_account_jwt;
    mod events_authority_configuration;
    mod events_authority_launch;
    mod events_authority_managed_launch;
    mod events_authority_vault;
    mod external_storage_vault;
    mod external_storage_vault_process;
    mod gateway_http3;
    mod gateway_realtime_frames;
    mod gateway_runtime;
    mod managed_event_credential;
    mod managed_runtime_supervision;
    mod managed_storage_vault_docker;
    mod managed_vault_binary;
    mod managed_vault_route;
    mod module_grant_snapshot;
    mod module_query_routes;
    mod module_registration_upgrade;
    mod module_request_routes;
    mod operation_journal;
    mod owner_module_settings;
    mod owner_vault_provisioning;
    mod part_01;
    mod part_02;
    mod part_03;
    mod part_04;
    mod part_06;
    mod part_07;
    mod platform_vault;
    mod process_port;
    mod protocol_validation;
    mod recovery_fence;
    mod recovery_media;
    mod restore_coordinator;
    mod scheduler_lifecycle;
    mod scheduler_requests;
    mod secure_file;
    mod settings_contract;
    mod settings_schema_upgrade;
    mod storage_authorization;
    mod storage_launch;
    mod storage_requests;
    mod storage_status;
    mod storage_topology;
    mod storage_vault_composition;
    mod telemetry_launch;
    mod vault_route_contract;
    mod vault_status;
}
#[cfg(test)]
pub(crate) use makosh_kernel_control_store_sqlite::StoreError;

#[cfg(test)]
#[path = "../../../../src/kernel/control_store/sqlite/src/actor/handle.rs"]
mod control_store_handle;

#[cfg(test)]
#[path = "../../../../src/kernel/src/platform/control_plane/worker.rs"]
mod control_plane_worker;
