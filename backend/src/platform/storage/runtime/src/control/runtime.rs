//! Storage runtime status service over the authenticated inherited Kernel channel.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeReadyRequestV1,
    managed_runtime_control_request_v1::Operation as ManagedOperation,
};
use makosh_storage_control::{StorageEndpointPreflightV1, preflight_storage_endpoints};
use makosh_storage_protocol::{
    v1::{
        GetStorageRuntimeStatusRequestV1, StorageRuntimeConfigurationV1,
        StorageRuntimeControlRequestV1, StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
        StorageRuntimeStatusV1, storage_runtime_control_request_v1::Operation,
        storage_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::{
        storage_binding_from_message, validate_storage_runtime_configuration,
        validate_storage_runtime_control_request,
    },
};
use prost::Message;

use super::admission::quarantine_invalid_desired_bindings;
use super::apply::{apply_active_binding, error_code as apply_error_code};
use super::framing::{read_frame, write_frame};
use super::handshake::{ManagedStorageRuntimeIdentityV1, authenticate, authenticate_on_channel};
use super::revocation::revoke_reserved_binding;
use super::vault_route::InheritedVaultRoutePortV1;
use crate::admin::{
    RuntimeRoleCredentialV1, apply_authorized_bindings, apply_authorized_migrations,
    reconcile_authorized_roles, verify_platform_admin, verify_platform_postgres,
};
use crate::vault::{
    StoragePlatformCredentialBootstrapV1, StoragePlatformCredentialErrorV1,
    StoragePlatformCredentialPurposeV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
    complete_immediately,
};

pub fn serve_inherited(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: StorageRuntimeConfigurationV1,
) -> Result<(), String> {
    validate_storage_runtime_configuration(&configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    let (channel, identity) = authenticate(descriptor_bytes, settings_schema_bytes)?;
    serve_bootstrapped(channel, identity, configuration)
}

#[allow(dead_code)]
pub fn serve_inherited_on_channel(
    channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: StorageRuntimeConfigurationV1,
) -> Result<(), String> {
    validate_storage_runtime_configuration(&configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    let (channel, identity) =
        authenticate_on_channel(channel, descriptor_bytes, settings_schema_bytes)?;
    serve_bootstrapped(channel, identity, configuration)
}

fn serve_bootstrapped(
    mut channel: UnixStream,
    identity: ManagedStorageRuntimeIdentityV1,
    configuration: StorageRuntimeConfigurationV1,
) -> Result<(), String> {
    let configuration = quarantine_invalid_desired_bindings(&configuration);
    let active_bindings = bootstrap_platform_services(&mut channel, &identity, &configuration)?;
    announce_ready(&mut channel, &identity)?;
    serve_authenticated(channel, identity, configuration, active_bindings)
}

fn announce_ready(
    channel: &mut UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
) -> Result<(), String> {
    let audience = identity.audience();
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Ready(ManagedRuntimeReadyRequestV1 {
            registration_id: audience.module_registration_id().to_owned(),
            runtime_generation: audience.runtime_generation(),
            grant_epoch: audience.grant_epoch(),
        })),
    };
    write_frame(channel, &request.encode_to_vec())
}

#[allow(dead_code)]
pub fn serve_on_channel(
    channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: StorageRuntimeConfigurationV1,
) -> Result<(), String> {
    validate_storage_runtime_configuration(&configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    let (channel, identity) =
        authenticate_on_channel(channel, descriptor_bytes, settings_schema_bytes)?;
    serve_authenticated(channel, identity, configuration, Vec::new())
}

#[allow(dead_code)]
pub fn serve_credential_bootstrapped_on_channel(
    channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: StorageRuntimeConfigurationV1,
) -> Result<(), String> {
    validate_storage_runtime_configuration(&configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    let (mut channel, identity) =
        authenticate_on_channel(channel, descriptor_bytes, settings_schema_bytes)?;
    bootstrap_pgbouncer_credential(&mut channel, &identity, &configuration)?;
    serve_authenticated(channel, identity, configuration, Vec::new())
}

fn serve_authenticated(
    mut channel: UnixStream,
    identity: ManagedStorageRuntimeIdentityV1,
    mut configuration: StorageRuntimeConfigurationV1,
    mut active_bindings: Vec<makosh_storage_protocol::v1::StorageBindingV1>,
) -> Result<(), String> {
    channel
        .set_read_timeout(None)
        .and_then(|_| channel.set_write_timeout(None))
        .map_err(|_| "Storage inherited control channel is unavailable".to_owned())?;
    loop {
        let response = read_frame(&mut channel)
            .and_then(|bytes| {
                StorageRuntimeControlRequestV1::decode(bytes.as_slice())
                    .map_err(|_| "Storage inherited control frame is invalid".to_owned())
            })
            .map(|request| {
                response_for(
                    &mut channel,
                    request,
                    &identity,
                    &mut configuration,
                    &mut active_bindings,
                )
            })
            .unwrap_or_else(|_| error_response("invalid_request"));
        write_frame(&mut channel, &response.encode_to_vec())?;
    }
}

fn bootstrap_platform_services(
    channel: &mut UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &StorageRuntimeConfigurationV1,
) -> Result<Vec<makosh_storage_protocol::v1::StorageBindingV1>, String> {
    let topology = configuration.topology.as_ref().expect("validated topology");
    let pgbouncer_credential = resolve_platform_credential(
        channel,
        identity,
        configuration,
        StoragePlatformCredentialPurposeV1::PgBouncerAdmin,
    )?;
    verify_platform_admin(topology, &pgbouncer_credential)?;
    let postgres_credential = resolve_platform_credential(
        channel,
        identity,
        configuration,
        StoragePlatformCredentialPurposeV1::PostgresAdmin,
    )?;
    let runtime_credentials = resolve_runtime_credentials(channel, configuration)?;
    verify_platform_postgres(topology, &postgres_credential)?;
    reconcile_authorized_roles(topology, &postgres_credential, &runtime_credentials)?;
    apply_authorized_migrations(
        topology,
        &postgres_credential,
        &configuration.desired_bindings,
        &configuration.desired_bundles,
    )?;
    apply_authorized_bindings(
        configuration,
        &pgbouncer_credential,
        &postgres_credential,
        &runtime_credentials,
    )?;
    Ok(configuration.desired_bindings.clone())
}

pub(super) fn resolve_runtime_credentials(
    channel: &UnixStream,
    configuration: &StorageRuntimeConfigurationV1,
) -> Result<Vec<RuntimeRoleCredentialV1>, String> {
    let context = vault_route_context(configuration)?;
    let route = InheritedVaultRoutePortV1::new(
        channel
            .try_clone()
            .map_err(|_| "Storage inherited control channel is unavailable".to_owned())?,
    );
    let mut vault = StorageVaultLeaseAdapterV1::new(route, context);
    configuration
        .desired_bindings
        .iter()
        .cloned()
        .map(|binding| resolve_runtime_credential(&mut vault, binding))
        .collect()
}

fn resolve_runtime_credential(
    vault: &mut StorageVaultLeaseAdapterV1<InheritedVaultRoutePortV1>,
    binding: makosh_storage_protocol::v1::StorageBindingV1,
) -> Result<RuntimeRoleCredentialV1, String> {
    let binding = storage_binding_from_message(&binding)
        .map_err(|_| "Storage binding is invalid".to_owned())?;
    let credential = match complete_immediately(vault.ensure_runtime_credential(&binding)) {
        Ok(Ok(credential)) => credential,
        Ok(Err(_)) | Err(_) => return Err("Storage runtime credential is unavailable".to_owned()),
    };
    RuntimeRoleCredentialV1::new(binding, credential)
}

fn bootstrap_pgbouncer_credential(
    channel: &mut UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &StorageRuntimeConfigurationV1,
) -> Result<(), String> {
    let topology = configuration.topology.as_ref().expect("validated topology");
    let credential = resolve_platform_credential(
        channel,
        identity,
        configuration,
        StoragePlatformCredentialPurposeV1::PgBouncerAdmin,
    )?;
    verify_platform_admin(topology, &credential)
}

pub(super) fn resolve_platform_credential(
    channel: &UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &StorageRuntimeConfigurationV1,
    purpose: StoragePlatformCredentialPurposeV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, String> {
    let topology = configuration.topology.as_ref().expect("validated topology");
    let context = vault_route_context(configuration)?;
    let route = InheritedVaultRoutePortV1::new(
        channel
            .try_clone()
            .map_err(|_| "Storage inherited control channel is unavailable".to_owned())?,
    );
    let mut bootstrap = StoragePlatformCredentialBootstrapV1::new(
        route,
        context,
        identity.audience(),
        purpose,
        topology.storage_instance_id.clone(),
        topology.storage_generation,
    )
    .map_err(|_| "Storage platform credential bootstrap is invalid".to_owned())?;
    match complete_immediately(bootstrap.ensure_and_resolve()) {
        Ok(Ok(credential)) => Ok(credential),
        Ok(Err(StoragePlatformCredentialErrorV1::Rejected)) => {
            Err("Storage platform credential bootstrap was rejected".to_owned())
        }
        Ok(Err(StoragePlatformCredentialErrorV1::Unavailable)) | Err(_) => {
            Err("Storage platform credential bootstrap is unavailable".to_owned())
        }
    }
}

fn vault_route_context(
    configuration: &StorageRuntimeConfigurationV1,
) -> Result<StorageVaultRouteContextV1, String> {
    let public_key = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| "Storage Vault route context is invalid".to_owned())?;
    StorageVaultRouteContextV1::new(
        configuration.vault_instance_id.clone(),
        configuration.vault_runtime_generation,
        public_key,
    )
    .map_err(|_| "Storage Vault route context is invalid".to_owned())
}

fn response_for(
    channel: &mut UnixStream,
    request: StorageRuntimeControlRequestV1,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &mut StorageRuntimeConfigurationV1,
    active_bindings: &mut Vec<makosh_storage_protocol::v1::StorageBindingV1>,
) -> StorageRuntimeControlResponseV1 {
    if validate_storage_runtime_control_request(&request).is_err() {
        return error_response("operation_not_available");
    }
    match request.operation {
        Some(Operation::GetStatus(GetStorageRuntimeStatusRequestV1 {})) => status_response(
            identity.runtime_generation(),
            &*configuration,
            active_bindings,
        ),
        Some(Operation::RevokeBinding(request)) => request.binding.map_or_else(
            || error_response("operation_not_available"),
            |binding| {
                revoke_reserved_binding(
                    channel,
                    identity,
                    &*configuration,
                    active_bindings,
                    binding,
                )
                .map(|binding| {
                    configuration.desired_bindings = active_bindings.clone();
                    revoked_response(binding)
                })
                .unwrap_or_else(|error| {
                    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                        eprintln!("developer_storage_binding_revocation_error={error}");
                    }
                    error_response(revocation_error_code(&error))
                })
            },
        ),
        Some(Operation::ApplyBinding(request)) => request.binding.zip(request.bundle).map_or_else(
            || error_response("operation_not_available"),
            |(binding, bundle)| {
                apply_active_binding(
                    channel,
                    identity,
                    configuration,
                    active_bindings,
                    binding,
                    bundle,
                )
                .map(active_response)
                .unwrap_or_else(|error| {
                    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                        eprintln!("developer_storage_binding_apply_error={error}");
                    }
                    error_response(apply_error_code(&error))
                })
            },
        ),
        None => error_response("operation_not_available"),
    }
}

fn revocation_error_code(error: &str) -> &'static str {
    match error {
        "Storage binding Vault lease revocation is incomplete" => "revocation_vault_incomplete",
        "Storage binding PgBouncer pause is incomplete" => "revocation_pool_pause_incomplete",
        "Storage binding PgBouncer disable is incomplete" => "revocation_pool_disable_incomplete",
        "Storage binding PgBouncer kill is incomplete" => "revocation_pool_kill_incomplete",
        "Storage binding PostgreSQL role fence is incomplete" => "revocation_postgres_incomplete",
        "Storage binding revocation lifecycle is invalid" => "revocation_lifecycle_invalid",
        "Storage platform credential bootstrap is unavailable" => {
            "revocation_platform_credential_unavailable"
        }
        "Storage platform credential bootstrap was rejected" => {
            "revocation_platform_credential_rejected"
        }
        "Storage platform credential bootstrap is invalid" => {
            "revocation_platform_credential_invalid"
        }
        "Storage Vault route context is invalid" => "revocation_vault_context_invalid",
        "Storage inherited control channel is unavailable" => "revocation_control_unavailable",
        "Storage PgBouncer admin endpoint is invalid" => "revocation_pool_endpoint_invalid",
        "Storage PgBouncer admin credential is invalid" => "revocation_pool_credential_invalid",
        "Storage PgBouncer admin authentication is unavailable" => {
            "revocation_pool_authentication_unavailable"
        }
        "Storage PostgreSQL admin authentication is unavailable" => {
            "revocation_postgres_authentication_unavailable"
        }
        "Storage binding is invalid" => "revocation_binding_invalid",
        _ => "revocation_incomplete",
    }
}

fn revoked_response(
    binding: makosh_storage_protocol::v1::StorageBindingV1,
) -> StorageRuntimeControlResponseV1 {
    StorageRuntimeControlResponseV1 {
        result: Some(ResponseResult::RevokedBinding(binding)),
        error_code: String::new(),
    }
}

fn active_response(
    binding: makosh_storage_protocol::v1::StorageBindingV1,
) -> StorageRuntimeControlResponseV1 {
    StorageRuntimeControlResponseV1 {
        result: Some(ResponseResult::ActiveBinding(binding)),
        error_code: String::new(),
    }
}

fn status_response(
    runtime_generation: u64,
    configuration: &StorageRuntimeConfigurationV1,
    active_bindings: &[makosh_storage_protocol::v1::StorageBindingV1],
) -> StorageRuntimeControlResponseV1 {
    let topology = configuration.topology.as_ref().expect("validated topology");
    let preflight = preflight_storage_endpoints(topology);
    let (state, blocker_code) = match preflight {
        StorageEndpointPreflightV1::Available => {
            let state = if active_bindings.is_empty() {
                StorageRuntimeStateV1::Reconciling
            } else {
                StorageRuntimeStateV1::Ready
            };
            (state, String::new())
        }
        StorageEndpointPreflightV1::Unavailable => (
            StorageRuntimeStateV1::Failed,
            "storage_endpoint_unavailable".to_owned(),
        ),
    };
    StorageRuntimeControlResponseV1 {
        result: Some(ResponseResult::Status(StorageRuntimeStatusV1 {
            state: state as i32,
            runtime_generation,
            storage_generation: topology.storage_generation,
            topology_revision: topology.topology_revision,
            vault_runtime_generation: configuration.vault_runtime_generation,
            active_bindings: active_bindings.to_vec(),
            blocker_code,
        })),
        error_code: String::new(),
    }
}

fn error_response(error_code: &str) -> StorageRuntimeControlResponseV1 {
    StorageRuntimeControlResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}
