//! Owner-authorized issuance of durable bindings for managed module runtimes.

use makosh_kernel_control_store::{
    PlatformStorageBindingInputV1, PlatformStorageBindingStateV1, PlatformStorageBindingV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use sha2::{Digest, Sha256};

use super::authorization::{
    StorageBindingAuthorizationV1, authorize_binding, authorize_managed_binding,
};
use super::topology;

pub(crate) struct StorageBindingIssueV1 {
    role_epoch: u64,
    credential_lease_revision: u64,
    storage_bundle_revision: u64,
    storage_bundle_digest: [u8; 32],
}

impl StorageBindingIssueV1 {
    pub(crate) fn new(
        role_epoch: u64,
        credential_lease_revision: u64,
        storage_bundle_revision: u64,
        storage_bundle_digest: [u8; 32],
    ) -> Result<Self, String> {
        (role_epoch > 0
            && credential_lease_revision > 0
            && storage_bundle_revision > 0
            && storage_bundle_digest != [0; 32])
            .then_some(Self {
                role_epoch,
                credential_lease_revision,
                storage_bundle_revision,
                storage_bundle_digest,
            })
            .ok_or_else(|| "Storage binding fences are invalid".to_owned())
    }

    pub(crate) const fn storage_bundle_revision(&self) -> u64 {
        self.storage_bundle_revision
    }

    pub(crate) const fn role_epoch(&self) -> u64 {
        self.role_epoch
    }

    pub(crate) const fn credential_lease_revision(&self) -> u64 {
        self.credential_lease_revision
    }

    pub(crate) const fn storage_bundle_digest(&self) -> &[u8; 32] {
        &self.storage_bundle_digest
    }
}

pub(crate) fn issue_managed(
    store: &SqliteControlStore,
    registration_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    capability_id: &str,
    issue: StorageBindingIssueV1,
) -> Result<PlatformStorageBindingV1, String> {
    let authorization = authorize_managed_binding(
        store,
        registration_id,
        runtime_instance_id,
        runtime_generation,
        capability_id,
    )?;
    issue_authorized(store, registration_id, capability_id, authorization, issue)
}

pub(crate) fn issue_external(
    store: &SqliteControlStore,
    registration_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    capability_id: &str,
    issue: StorageBindingIssueV1,
) -> Result<PlatformStorageBindingV1, String> {
    let authorization = authorize_binding(
        store,
        registration_id,
        runtime_instance_id,
        runtime_generation,
        capability_id,
    )?;
    issue_authorized(store, registration_id, capability_id, authorization, issue)
}

fn issue_authorized(
    store: &SqliteControlStore,
    registration_id: &str,
    capability_id: &str,
    authorization: StorageBindingAuthorizationV1,
    issue: StorageBindingIssueV1,
) -> Result<PlatformStorageBindingV1, String> {
    let topology = topology::current(store)?;
    verify_admitted_bundle(store, authorization.owner_id(), &issue)?;
    let previous = store
        .platform_storage_binding(registration_id, capability_id)
        .map_err(|_| "Storage binding is unavailable".to_owned())?;
    if previous
        .as_ref()
        .is_some_and(|binding| exact_retry(binding, &authorization, &topology, &issue))
    {
        return Ok(previous.expect("checked exact Storage binding retry"));
    }
    validate_successor(previous.as_ref(), &issue)?;
    let binding = bind(authorization, &topology, previous.as_ref(), issue)?;
    store
        .record_platform_storage_binding(&binding)
        .map_err(|_| "Storage binding cannot be recorded".to_owned())?;
    Ok(binding)
}

fn exact_retry(
    binding: &PlatformStorageBindingV1,
    authorization: &StorageBindingAuthorizationV1,
    topology: &makosh_kernel_control_store::PlatformStorageTopology,
    issue: &StorageBindingIssueV1,
) -> bool {
    binding.state() == PlatformStorageBindingStateV1::Active
        && binding.registration_id() == authorization.registration_id()
        && binding.capability_id() == authorization.capability_id()
        && binding.owner_id() == authorization.owner_id()
        && binding.topology_revision() == topology.revision()
        && binding.storage_generation() == topology.storage_generation()
        && binding.runtime_instance_id() == authorization.runtime_id()
        && binding.runtime_generation() == authorization.runtime_generation()
        && binding.grant_epoch() == authorization.grant_epoch()
        && binding.connection_budget() == authorization.connection_budget()
        && binding.statement_timeout_millis() == authorization.statement_timeout_millis()
        && binding.role_epoch() == issue.role_epoch()
        && binding.credential_lease_revision() == issue.credential_lease_revision()
        && binding.storage_bundle_revision() == issue.storage_bundle_revision()
        && binding.storage_bundle_digest() == issue.storage_bundle_digest()
}

fn verify_admitted_bundle(
    store: &SqliteControlStore,
    owner_id: &str,
    issue: &StorageBindingIssueV1,
) -> Result<(), String> {
    let bundle = store
        .platform_storage_bundle(owner_id, issue.storage_bundle_revision())
        .map_err(|_| "Storage bundle is unavailable".to_owned())?
        .ok_or_else(|| "Storage bundle is unavailable".to_owned())?;
    (bundle.digest() == issue.storage_bundle_digest())
        .then_some(())
        .ok_or_else(|| "Storage bundle digest is unavailable".to_owned())
}

fn validate_successor(
    previous: Option<&PlatformStorageBindingV1>,
    issue: &StorageBindingIssueV1,
) -> Result<(), String> {
    match previous {
        None if issue.role_epoch == 1 && issue.credential_lease_revision == 1 => Ok(()),
        Some(previous)
            if previous.state() == PlatformStorageBindingStateV1::Revoking
                && issue.role_epoch == increment(previous.role_epoch())?
                && issue.credential_lease_revision
                    == increment(previous.credential_lease_revision())?
                && issue.storage_bundle_revision >= previous.storage_bundle_revision() =>
        {
            Ok(())
        }
        _ => Err("Storage binding fences are not a valid successor".to_owned()),
    }
}

fn bind(
    authorization: StorageBindingAuthorizationV1,
    topology: &makosh_kernel_control_store::PlatformStorageTopology,
    previous: Option<&PlatformStorageBindingV1>,
    issue: StorageBindingIssueV1,
) -> Result<PlatformStorageBindingV1, String> {
    PlatformStorageBindingV1::new(PlatformStorageBindingInputV1 {
        registration_id: authorization.registration_id().to_owned(),
        capability_id: authorization.capability_id().to_owned(),
        owner_id: authorization.owner_id().to_owned(),
        binding_revision: next_revision(previous)?,
        topology_revision: topology.revision(),
        storage_generation: topology.storage_generation(),
        runtime_instance_id: authorization.runtime_id().to_owned(),
        runtime_generation: authorization.runtime_generation(),
        grant_epoch: authorization.grant_epoch(),
        role_epoch: issue.role_epoch,
        runtime_principal: runtime_principal(
            topology.storage_instance_id(),
            authorization.registration_id(),
            authorization.capability_id(),
            issue.role_epoch,
        ),
        connection_budget: authorization.connection_budget(),
        statement_timeout_millis: authorization.statement_timeout_millis(),
        credential_lease_revision: issue.credential_lease_revision,
        storage_bundle_revision: issue.storage_bundle_revision,
        storage_bundle_digest: issue.storage_bundle_digest,
    })
    .map_err(|_| "Storage binding is invalid".to_owned())
}

fn next_revision(previous: Option<&PlatformStorageBindingV1>) -> Result<u64, String> {
    previous.map_or(Ok(1), |binding| increment(binding.binding_revision()))
}

fn increment(value: u64) -> Result<u64, String> {
    value
        .checked_add(1)
        .ok_or_else(|| "Storage binding fence overflowed".to_owned())
}

fn runtime_principal(
    storage_instance_id: &str,
    registration_id: &str,
    capability_id: &str,
    role_epoch: u64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(storage_instance_id.as_bytes());
    hasher.update([0]);
    hasher.update(registration_id.as_bytes());
    hasher.update([0]);
    hasher.update(capability_id.as_bytes());
    let digest = hasher.finalize();
    format!(
        "storage_{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}_{role_epoch}",
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    )
}
