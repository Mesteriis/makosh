//! Module Registry mutations and read model shared by local control transports.

use makosh_kernel_control_store::{
    BundledArtifactProposalStore, BundledManagedArtifactProposalInputV1,
    BundledManagedArtifactProposalReceiptV1, ExternalRuntimeAttestation, ExternalRuntimeIdentity,
    GrantSet, HealthRecoveryStore, ModuleRegistration, ModuleRegistrationState,
    ModuleRegistryStore, OperationIdV1, OwnerIdentityStore, RuntimeTrustStore,
};
use makosh_kernel_control_store_sqlite::StoreError;

use crate::identity::owner::authorization::{authorize as authorize_file_owner, operation_digest};
use crate::infrastructure::filesystem::new_instance_id;
use crate::modules::capability::policy::permits_external_route;
use p256::ecdsa::VerifyingKey;

use super::descriptor::DescriptorRegistrationRequests;

pub struct ModuleRegistryStatus {
    registration: ModuleRegistration,
    effective_capability_count: usize,
    external_runtime_attestation: Option<ExternalRuntimeAttestation>,
}

pub struct BundledArtifactProposal {
    receipt: BundledManagedArtifactProposalReceiptV1,
    requested_capability_ids: Vec<String>,
}

pub struct BundledArtifactUpgrade {
    registration: ModuleRegistration,
    requested_capability_ids: Vec<String>,
}

impl BundledArtifactProposal {
    #[must_use]
    pub const fn receipt(&self) -> &BundledManagedArtifactProposalReceiptV1 {
        &self.receipt
    }

    #[must_use]
    pub fn requested_capability_ids(&self) -> &[String] {
        &self.requested_capability_ids
    }
}

impl BundledArtifactUpgrade {
    pub const fn registration(&self) -> &ModuleRegistration {
        &self.registration
    }

    pub fn requested_capability_ids(&self) -> &[String] {
        &self.requested_capability_ids
    }
}

impl ModuleRegistryStatus {
    #[must_use]
    pub fn registration(&self) -> &ModuleRegistration {
        &self.registration
    }
    #[must_use]
    pub fn effective_capability_count(&self) -> usize {
        self.effective_capability_count
    }
    #[must_use]
    pub fn external_runtime_attestation(&self) -> Option<&ExternalRuntimeAttestation> {
        self.external_runtime_attestation.as_ref()
    }
}

pub fn register<S>(store: &S, descriptor_bytes: &[u8]) -> Result<ModuleRegistration, String>
where
    S: ModuleRegistryStore<Error = StoreError> + OwnerIdentityStore<Error = StoreError>,
{
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err("module registration requires an enrolled initial owner".to_owned());
    }
    let requests = DescriptorRegistrationRequests::decode(descriptor_bytes)?;
    persist_registration(store, requests)
}

pub fn propose_bundled_artifact<S>(
    store: &S,
    descriptor_bytes: &[u8],
    operation_id: OperationIdV1,
    request_digest: [u8; 32],
    distribution_id: &str,
    distribution_generation: u64,
    artifact_id: &str,
) -> Result<BundledArtifactProposal, String>
where
    S: BundledArtifactProposalStore<Error = StoreError> + OwnerIdentityStore<Error = StoreError>,
{
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err("module registration requires an enrolled initial owner".to_owned());
    }
    let requests = DescriptorRegistrationRequests::decode(descriptor_bytes)?;
    let proposal = BundledManagedArtifactProposalInputV1::new(
        operation_id,
        request_digest,
        distribution_id,
        distribution_generation,
        artifact_id,
    );
    for _ in 0..16 {
        let registration = ModuleRegistration::new(
            new_instance_id()?,
            requests.module_id(),
            requests.owner_id(),
            requests.descriptor_sha256(),
            ModuleRegistrationState::Pending,
            1,
        );
        let bound = requests.bind(&registration);
        match store.propose_bundled_managed_artifact(
            &proposal,
            &registration,
            requests.capability_ids(),
            makosh_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                storage: &bound.storage,
                events: &bound.events,
                blobs: &bound.blobs,
                scheduler: &bound.scheduler,
                vault_purposes: &bound.vault_purposes,
                client_rpc_routes: &bound.client_rpc_routes,
                client_blob_routes: &bound.client_blob_routes,
                client_realtime_routes: &bound.client_realtime_routes,
                query_rpc_routes: &bound.query_rpc_routes,
                request_rpc_routes: &bound.request_rpc_routes,
                contract_dependencies: &bound.contract_dependencies,
            },
        ) {
            Ok(receipt) => {
                return Ok(BundledArtifactProposal {
                    receipt,
                    requested_capability_ids: requests.capability_ids().to_vec(),
                });
            }
            Err(StoreError::ModuleRegistrationAlreadyExists) => {}
            Err(error) => return Err(format!("{error:?}")),
        }
    }
    Err("unable to allocate a unique module registration ID".to_owned())
}

pub fn upgrade_bundled_artifact<S>(
    store: &S,
    registration_id: &str,
    descriptor_bytes: &[u8],
) -> Result<BundledArtifactUpgrade, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    let current = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed registration does not exist".to_owned())?;
    if current.state() != ModuleRegistrationState::Approved {
        return Err("managed registration is not approved".to_owned());
    }
    let requests = DescriptorRegistrationRequests::decode(descriptor_bytes)?;
    if current.module_id() != requests.module_id() || current.owner_id() != requests.owner_id() {
        return Err("managed registration identity cannot change during upgrade".to_owned());
    }
    if *current.descriptor_sha256() == requests.descriptor_sha256() {
        return Ok(BundledArtifactUpgrade {
            registration: current,
            requested_capability_ids: requests.capability_ids().to_vec(),
        });
    }
    let replacement = ModuleRegistration::new(
        registration_id,
        current.module_id(),
        current.owner_id(),
        requests.descriptor_sha256(),
        ModuleRegistrationState::Approved,
        current
            .grant_epoch()
            .checked_add(1)
            .ok_or_else(|| "managed registration grant epoch overflowed".to_owned())?,
    );
    let bound = requests.bind(&replacement);
    store
        .upgrade_approved_registration_with_all_descriptor_requests(
            &replacement,
            requests.capability_ids(),
            makosh_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                storage: &bound.storage,
                events: &bound.events,
                blobs: &bound.blobs,
                scheduler: &bound.scheduler,
                vault_purposes: &bound.vault_purposes,
                client_rpc_routes: &bound.client_rpc_routes,
                client_blob_routes: &bound.client_blob_routes,
                client_realtime_routes: &bound.client_realtime_routes,
                query_rpc_routes: &bound.query_rpc_routes,
                request_rpc_routes: &bound.request_rpc_routes,
                contract_dependencies: &bound.contract_dependencies,
            },
        )
        .map_err(|error| format!("{error:?}"))?;
    Ok(BundledArtifactUpgrade {
        registration: replacement,
        requested_capability_ids: requests.capability_ids().to_vec(),
    })
}

fn persist_registration<S>(
    store: &S,
    requests: DescriptorRegistrationRequests,
) -> Result<ModuleRegistration, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    for _ in 0..16 {
        let registration = ModuleRegistration::new(
            new_instance_id()?,
            requests.module_id(),
            requests.owner_id(),
            requests.descriptor_sha256(),
            ModuleRegistrationState::Pending,
            1,
        );
        let bound = requests.bind(&registration);
        match store.create_pending_registration_with_all_descriptor_requests(
            &registration,
            requests.capability_ids(),
            makosh_kernel_control_store::ModuleDescriptorRegistrationRequestsV1 {
                storage: &bound.storage,
                events: &bound.events,
                blobs: &bound.blobs,
                scheduler: &bound.scheduler,
                vault_purposes: &bound.vault_purposes,
                client_rpc_routes: &bound.client_rpc_routes,
                client_blob_routes: &bound.client_blob_routes,
                client_realtime_routes: &bound.client_realtime_routes,
                query_rpc_routes: &bound.query_rpc_routes,
                request_rpc_routes: &bound.request_rpc_routes,
                contract_dependencies: &bound.contract_dependencies,
            },
        ) {
            Ok(()) => return Ok(registration),
            Err(
                makosh_kernel_control_store_sqlite::StoreError::ModuleRegistrationAlreadyExists,
            ) => {}
            Err(error) => return Err(format!("{error:?}")),
        }
    }
    Err("unable to allocate a unique module registration ID".to_owned())
}

pub fn approve<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<GrantSet, String>
where
    S: HealthRecoveryStore
        + ModuleRegistryStore<Error = StoreError>
        + OwnerIdentityStore<Error = StoreError>,
{
    let mut authorization_fields = Vec::with_capacity(capability_ids.len() + 1);
    authorization_fields.push(registration_id);
    authorization_fields.extend(capability_ids.iter().map(String::as_str));
    authorize_file_owner(
        data_dir,
        store,
        "module.approve.v1",
        operation_digest(&authorization_fields)?,
    )?;
    approve_after_owner_authorization(store, registration_id, capability_ids)
}

pub fn approve_after_owner_authorization<S>(
    store: &S,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<GrantSet, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    if capability_ids
        .iter()
        .any(|capability_id| !permits_external_route(capability_id))
    {
        return Err("capability grant is prohibited by Kernel policy".to_owned());
    }
    store
        .approve_module_registration(registration_id, capability_ids)
        .map_err(|error| format!("{error:?}"))
}

pub fn transition<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    next: ModuleRegistrationState,
) -> Result<ModuleRegistration, String>
where
    S: HealthRecoveryStore
        + ModuleRegistryStore<Error = StoreError>
        + OwnerIdentityStore<Error = StoreError>,
{
    authorize_file_owner(
        data_dir,
        store,
        "module.transition.v1",
        operation_digest(&[registration_id, next.as_str()])?,
    )?;
    transition_after_owner_authorization(store, registration_id, next)
}

pub fn transition_after_owner_authorization<S>(
    store: &S,
    registration_id: &str,
    next: ModuleRegistrationState,
) -> Result<ModuleRegistration, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    store
        .transition_module_registration(registration_id, next)
        .map_err(|error| format!("{error:?}"))
}

pub fn bind_external_runtime_identity<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    public_key_sec1: [u8; 65],
) -> Result<ModuleRegistration, String>
where
    S: HealthRecoveryStore
        + OwnerIdentityStore<Error = StoreError>
        + RuntimeTrustStore<Error = StoreError>,
{
    VerifyingKey::from_sec1_bytes(&public_key_sec1)
        .map_err(|_| "external runtime public key is invalid".to_owned())?;
    let public_key_hex = public_key_sec1
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    authorize_file_owner(
        data_dir,
        store,
        "module.bind_external_runtime_identity.v1",
        operation_digest(&[registration_id, &public_key_hex])?,
    )?;
    bind_external_runtime_identity_after_owner_authorization(
        store,
        registration_id,
        public_key_sec1,
    )
}

pub fn bind_external_runtime_identity_after_owner_authorization<S>(
    store: &S,
    registration_id: &str,
    public_key_sec1: [u8; 65],
) -> Result<ModuleRegistration, String>
where
    S: RuntimeTrustStore<Error = StoreError>,
{
    VerifyingKey::from_sec1_bytes(&public_key_sec1)
        .map_err(|_| "external runtime public key is invalid".to_owned())?;
    store
        .bind_external_runtime_identity(&ExternalRuntimeIdentity::new(
            registration_id,
            public_key_sec1,
        ))
        .map_err(|error| format!("{error:?}"))
}

pub fn status<S>(store: &S, registration_id: &str) -> Result<ModuleRegistryStatus, String>
where
    S: ModuleRegistryStore<Error = StoreError> + RuntimeTrustStore<Error = StoreError>,
{
    let snapshot = store
        .module_grant_snapshot(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "module registration does not exist".to_owned())?;
    let effective_capability_count = snapshot
        .effective_grants()
        .map_or(0, |grants| grants.capability_ids().len());
    let (registration, _) = snapshot.into_parts();
    let external_runtime_attestation = store
        .effective_external_runtime_attestation(registration_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(ModuleRegistryStatus {
        registration,
        effective_capability_count,
        external_runtime_attestation,
    })
}
