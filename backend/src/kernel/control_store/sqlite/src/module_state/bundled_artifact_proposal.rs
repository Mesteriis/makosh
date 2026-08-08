//! Atomic proposal of a verified bundled module artifact and its pending registration.

use makosh_kernel_control_store::{
    BundledManagedArtifactProposalInputV1, BundledManagedArtifactProposalReceiptV1,
    ModuleDescriptorRegistrationRequestsV1, ModuleRegistration,
};
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_identity_token};

use super::{
    blob_request::{insert_blob_quota_requests, validate_blob_quota_requests},
    client_blob_route::{insert_client_blob_routes, validate_client_blob_routes},
    client_realtime_route::{insert_client_realtime_routes, validate_client_realtime_routes},
    client_rpc_route::{insert_client_rpc_routes, validate_client_rpc_routes},
    event_request::{insert_event_route_requests, validate_event_route_requests},
    registry::{
        insert_pending_registration, read_required_registration, validate_pending_registration,
    },
    scheduler_request::{insert_scheduler_job_requests, validate_scheduler_job_requests},
    storage_request::{insert_storage_requests, validate_storage_requests},
    vault_purpose_request::{insert_vault_purpose_requests, validate_vault_purpose_requests},
};

impl SqliteControlStore {
    pub fn propose_bundled_managed_artifact(
        &self,
        proposal: &BundledManagedArtifactProposalInputV1,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
        requests: ModuleDescriptorRegistrationRequestsV1<'_>,
    ) -> Result<BundledManagedArtifactProposalReceiptV1, StoreError> {
        validate_proposal(proposal, registration, requested_capability_ids, &requests)?;
        let proposal = proposal.clone();
        let registration = registration.clone();
        let capabilities = requested_capability_ids.to_vec();
        let storage_requests = requests.storage.to_vec();
        let event_requests = requests.events.to_vec();
        let blob_requests = requests.blobs.to_vec();
        let scheduler_requests = requests.scheduler.to_vec();
        let vault_purpose_requests = requests.vault_purposes.to_vec();
        let client_rpc_routes = requests.client_rpc_routes.to_vec();
        let client_blob_routes = requests.client_blob_routes.to_vec();
        let client_realtime_routes = requests.client_realtime_routes.to_vec();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            if let Some((request_digest, registration_id)) = transaction
                .query_row(
                    "SELECT request_digest, registration_id
                     FROM makosh_kernel_bundled_artifact_proposal
                     WHERE operation_id = ?1",
                    [proposal.operation_id().as_bytes().as_slice()],
                    |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?
            {
                let request_digest: [u8; 32] = request_digest
                    .try_into()
                    .map_err(|_| StoreError::InvalidBundledManagedArtifactProposal)?;
                if &request_digest != proposal.request_digest() {
                    return Err(StoreError::OperationRequestDigestConflict);
                }
                let registration = read_required_registration(&transaction, &registration_id)?;
                transaction.commit()?;
                return Ok(BundledManagedArtifactProposalReceiptV1::new(
                    registration,
                    true,
                ));
            }

            insert_pending_registration(&transaction, &registration, &capabilities)?;
            insert_storage_requests(&transaction, &storage_requests)?;
            insert_event_route_requests(&transaction, &event_requests)?;
            insert_blob_quota_requests(&transaction, &blob_requests)?;
            insert_scheduler_job_requests(&transaction, &scheduler_requests)?;
            insert_vault_purpose_requests(&transaction, &vault_purpose_requests)?;
            insert_client_rpc_routes(&transaction, &client_rpc_routes)?;
            insert_client_blob_routes(&transaction, &client_blob_routes)?;
            insert_client_realtime_routes(&transaction, &client_realtime_routes)?;
            transaction.execute(
                "INSERT INTO makosh_kernel_bundled_artifact_proposal
                 (operation_id, request_digest, registration_id, distribution_id,
                  distribution_generation, artifact_id, descriptor_sha256)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    proposal.operation_id().as_bytes().as_slice(),
                    proposal.request_digest().as_slice(),
                    registration.registration_id(),
                    proposal.distribution_id(),
                    as_sql(proposal.distribution_generation())?,
                    proposal.artifact_id(),
                    registration.descriptor_sha256().as_slice(),
                ],
            )?;
            transaction.commit()?;
            Ok(BundledManagedArtifactProposalReceiptV1::new(
                registration,
                false,
            ))
        })
    }
}

fn validate_proposal(
    proposal: &BundledManagedArtifactProposalInputV1,
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &ModuleDescriptorRegistrationRequestsV1<'_>,
) -> Result<(), StoreError> {
    if proposal
        .operation_id()
        .as_bytes()
        .iter()
        .all(|byte| *byte == 0)
        || proposal.distribution_generation() == 0
        || !valid_identity_token(proposal.distribution_id())
        || !valid_identity_token(proposal.artifact_id())
    {
        return Err(StoreError::InvalidBundledManagedArtifactProposal);
    }
    validate_pending_registration(registration, capabilities)?;
    validate_storage_requests(registration, capabilities, requests.storage)?;
    validate_event_route_requests(registration, capabilities, requests.events)?;
    validate_blob_quota_requests(registration, capabilities, requests.blobs)?;
    validate_scheduler_job_requests(registration, capabilities, requests.scheduler)?;
    validate_vault_purpose_requests(registration, capabilities, requests.vault_purposes)?;
    validate_client_rpc_routes(registration, capabilities, requests.client_rpc_routes)?;
    validate_client_blob_routes(
        registration,
        capabilities,
        requests.blobs,
        requests.client_blob_routes,
    )?;
    validate_client_realtime_routes(registration, capabilities, requests.client_realtime_routes)
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::RecoveryFenceOverflow)
}
