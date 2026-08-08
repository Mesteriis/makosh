//! Owner and registration IPC setup for the external Storage process fixture.

use std::path::Path;

use makosh_gateway_protocol::v1::{
    AdmitStorageBundleRequestV1, ApproveModuleRegistrationRequestV1,
    BeginExternalStorageBindingRevocationRequestV1, BeginModuleRegistrationRequestV1,
    BeginOwnerControlSessionRequestV1, BindExternalRuntimeIdentityRequestV1,
    BindPlatformVaultReleaseRequestV1, CompleteOwnerControlSessionRequestV1,
    ConfigurePlatformStorageTopologyRequestV1, DescribeModuleRegistrationRequestV1,
    IssueExternalStorageBindingRequestV1, ModuleRegistrationRequestV1,
    ModuleRegistrationResponseV1, OwnerControlRequestV1, OwnerControlResponseV1,
    StartPlatformVaultRuntimeRequestV1,
    module_registration_request_v1::Operation as RegistrationOperation,
    module_registration_response_v1::Result as RegistrationResult,
    owner_control_request_v1::Operation as OwnerOperation,
    owner_control_response_v1::Result as OwnerResult,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ModuleDescriptorV1,
    ModuleKindV1, StorageNamespaceRequestV1, VaultActionV1, VaultPurposeRequestV1,
    VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use p256::ecdsa::SigningKey;
use prost::Message;
use sha2::{Digest, Sha256};

use super::kernel::RunningKernel;
use super::transport;
use crate::identity::device::signer::DeviceSigner;

const OWNER_ID: &str = "owner_storage";
pub(super) const RUNTIME_ID: &str = "runtime_storage";
pub(super) const RUNTIME_GENERATION: u64 = 1;

pub(super) fn configure(
    kernel: &RunningKernel,
    signing_key: &SigningKey,
) -> Result<String, String> {
    let registration_id = register(kernel)?;
    let session_id = owner_session(&kernel.owner_socket, &kernel.data_dir)?;
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::ApproveModuleRegistration(ApproveModuleRegistrationRequestV1 {
            registration_id: registration_id.clone(),
            capability_id: vec![
                "storage.access".to_owned(),
                "vault.lease.resolve".to_owned(),
            ],
            owner_session_id: session_id.clone(),
        }),
    )?;
    bind_external_identity(kernel, &registration_id, &session_id, signing_key)?;
    start_vault(kernel, &session_id)?;
    configure_topology(kernel, &session_id)?;
    admit_bundle(kernel, &session_id)?;
    Ok(registration_id)
}

pub(super) fn issue_binding(
    kernel: &RunningKernel,
    registration_id: &str,
    role_epoch: u64,
    credential_lease_revision: u64,
) -> Result<(), String> {
    let session_id = owner_session(&kernel.owner_socket, &kernel.data_dir)?;
    let bundle = storage_bundle();
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::IssueExternalStorageBinding(IssueExternalStorageBindingRequestV1 {
            owner_session_id: session_id,
            registration_id: registration_id.to_owned(),
            capability_id: "storage.access".to_owned(),
            runtime_instance_id: RUNTIME_ID.to_owned(),
            runtime_generation: RUNTIME_GENERATION,
            role_epoch,
            credential_lease_revision,
            storage_bundle_revision: 1,
            storage_bundle_digest: Sha256::digest(bundle.encode_to_vec()).to_vec(),
        }),
    )?;
    Ok(())
}

pub(super) fn rotate_binding(kernel: &RunningKernel, registration_id: &str) -> Result<(), String> {
    let session_id = owner_session(&kernel.owner_socket, &kernel.data_dir)?;
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::BeginExternalStorageBindingRevocation(
            BeginExternalStorageBindingRevocationRequestV1 {
                owner_session_id: session_id,
                registration_id: registration_id.to_owned(),
                capability_id: "storage.access".to_owned(),
                binding_revision: 1,
                runtime_instance_id: RUNTIME_ID.to_owned(),
                runtime_generation: RUNTIME_GENERATION,
            },
        ),
    )?;
    issue_binding(kernel, registration_id, 2, 2)
}

fn register(kernel: &RunningKernel) -> Result<String, String> {
    let begun = registration_call(
        &kernel.registration_socket,
        RegistrationOperation::Begin(BeginModuleRegistrationRequestV1 {}),
    )?;
    let RegistrationResult::Begin(begun) = begun else {
        return Err("module registration session is unavailable".to_owned());
    };
    let described = registration_call(
        &kernel.registration_socket,
        RegistrationOperation::Describe(DescribeModuleRegistrationRequestV1 {
            session_id: begun.session_id,
            descriptor_bytes: descriptor().encode_to_vec(),
        }),
    )?;
    match described {
        RegistrationResult::Describe(response) => Ok(response.registration_id),
        _ => Err("module registration is unavailable".to_owned()),
    }
}

fn bind_external_identity(
    kernel: &RunningKernel,
    registration_id: &str,
    session_id: &str,
    signing_key: &SigningKey,
) -> Result<(), String> {
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::BindExternalRuntimeIdentity(BindExternalRuntimeIdentityRequestV1 {
            registration_id: registration_id.to_owned(),
            public_key_sec1: signing_key
                .verifying_key()
                .to_sec1_point(false)
                .as_bytes()
                .to_vec(),
            owner_session_id: session_id.to_owned(),
        }),
    )?;
    Ok(())
}

fn start_vault(kernel: &RunningKernel, session_id: &str) -> Result<(), String> {
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::BindPlatformVaultRelease(BindPlatformVaultReleaseRequestV1 {
            owner_session_id: session_id.to_owned(),
        }),
    )?;
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::StartPlatformVaultRuntime(StartPlatformVaultRuntimeRequestV1 {
            owner_session_id: session_id.to_owned(),
        }),
    )?;
    Ok(())
}

fn configure_topology(kernel: &RunningKernel, session_id: &str) -> Result<(), String> {
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::ConfigurePlatformStorageTopology(
            ConfigurePlatformStorageTopologyRequestV1 {
                owner_session_id: session_id.to_owned(),
                storage_generation: 1,
                storage_instance_id: "storage_main".to_owned(),
                database_id: "makosh".to_owned(),
                deployment_profile: 1,
                postgres_artifact_sha256: vec![3; 32],
                pgbouncer_artifact_sha256: vec![4; 32],
                postgres_host: "127.0.0.1".to_owned(),
                postgres_port: 5_432,
                pgbouncer_host: "127.0.0.1".to_owned(),
                pgbouncer_port: 6_432,
                pgbouncer_backend_host: "postgres".to_owned(),
                pgbouncer_backend_port: 5_432,
            },
        ),
    )?;
    Ok(())
}

fn admit_bundle(kernel: &RunningKernel, session_id: &str) -> Result<(), String> {
    owner_call(
        &kernel.owner_socket,
        OwnerOperation::AdmitStorageBundle(AdmitStorageBundleRequestV1 {
            owner_session_id: session_id.to_owned(),
            canonical_bundle: storage_bundle().encode_to_vec(),
        }),
    )?;
    Ok(())
}

fn owner_session(socket: &Path, data_dir: &Path) -> Result<String, String> {
    let begun = owner_call(
        socket,
        OwnerOperation::BeginOwnerSession(BeginOwnerControlSessionRequestV1 {}),
    )?;
    let OwnerResult::BeginOwnerSession(challenge) = begun else {
        return Err("owner challenge is unavailable".to_owned());
    };
    let signer = crate::identity::device::signer::FileDeviceSigner::open_for_instance(data_dir)?;
    let signature: [u8; 64] = signer.sign(&owner_proof(&challenge)?);
    let completed = owner_call(
        socket,
        OwnerOperation::CompleteOwnerSession(CompleteOwnerControlSessionRequestV1 {
            challenge_id: challenge.challenge_id,
            signature_raw: signature.to_vec(),
        }),
    )?;
    match completed {
        OwnerResult::CompleteOwnerSession(response) => Ok(response.owner_session_id),
        _ => Err("owner session is unavailable".to_owned()),
    }
}

fn owner_proof(
    challenge: &makosh_gateway_protocol::v1::BeginOwnerControlSessionResponseV1,
) -> Result<Vec<u8>, String> {
    let mut proof = b"makosh.owner-control-session.v1\0".to_vec();
    for value in [
        &challenge.kernel_instance_id,
        &challenge.owner_id,
        &challenge.device_id,
    ] {
        let length = u16::try_from(value.len()).map_err(|_| "owner proof is invalid".to_owned())?;
        proof.extend_from_slice(&length.to_be_bytes());
        proof.extend_from_slice(value.as_bytes());
    }
    proof.extend_from_slice(&challenge.control_store_generation.to_be_bytes());
    proof.extend_from_slice(&challenge.challenge_bytes);
    Ok(proof)
}

fn registration_call(
    socket: &Path,
    operation: RegistrationOperation,
) -> Result<RegistrationResult, String> {
    let response = transport::call::<_, ModuleRegistrationResponseV1>(
        socket,
        &ModuleRegistrationRequestV1 {
            operation: Some(operation),
        },
    )?;
    response.result.ok_or_else(|| {
        format!(
            "module registration request failed: {}",
            response.error_code
        )
    })
}

fn owner_call(socket: &Path, operation: OwnerOperation) -> Result<OwnerResult, String> {
    let response = transport::call::<_, OwnerControlResponseV1>(
        socket,
        &OwnerControlRequestV1 {
            operation: Some(operation),
        },
    )?;
    response
        .result
        .ok_or_else(|| format!("owner control request failed: {}", response.error_code))
}

fn descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "module_storage".to_owned(),
        owner_id: OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: "external-storage-process".to_owned(),
        capabilities: vec![storage_capability(), vault_capability()],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: "storage.access".to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: OWNER_ID.to_owned(),
                connection_budget: 4,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

fn vault_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: "vault.lease.resolve".to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
                purpose_id: "storage.runtime.credential".to_owned(),
                requested_lease_ttl_seconds: 60,
                allowed_secret_classes: vec![VaultSecretClassV1::PlatformCredential as i32],
                actions: vec![VaultActionV1::Resolve as i32, VaultActionV1::Create as i32],
                target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
                key_schema_revision: 0,
            })),
        }],
        ..Default::default()
    }
}

fn storage_bundle() -> StorageBundleV1 {
    let sql = b"CREATE TABLE makosh_data.owner_storage_probe (probe_id uuid);".to_vec();
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "owner_storage_bundle".to_owned(),
        owner_id: OWNER_ID.to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "create_probe".to_owned(),
            sha256: Sha256::digest(&sql).to_vec(),
            forward_sql_utf8: sql,
        }],
    }
}
