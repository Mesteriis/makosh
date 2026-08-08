//! Executes one owner-local derived-index job without exposing private content.

use makosh_communications_domain::CommunicationsSearchIndexJobV1;
use makosh_communications_persistence::{
    CommunicationsDerivedIndexFailureV1, CommunicationsDerivedIndexJobOperationV1,
    CommunicationsDurablePersistence, CommunicationsPersistenceError,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use std::os::unix::net::UnixStream;

use crate::{
    search_access::{CommunicationsSearchAccessErrorV1, CommunicationsSearchAccessV1},
    search_projection::assemble_search_projection_write_v1,
};

const INDEX_JOB_LEASE_SECONDS: i64 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchWorkerErrorV1 {
    StorageUnavailable,
}

pub async fn process_next_derived_index_job_v1(
    persistence: &CommunicationsDurablePersistence,
    access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    worker_id: &str,
    now_unix_seconds: i64,
) -> Result<bool, CommunicationsSearchWorkerErrorV1> {
    let lease_expires_at_unix_seconds = now_unix_seconds
        .checked_add(INDEX_JOB_LEASE_SECONDS)
        .ok_or(CommunicationsSearchWorkerErrorV1::StorageUnavailable)?;
    let Some(claimed) = persistence
        .claim_next_derived_index_job(worker_id, now_unix_seconds, lease_expires_at_unix_seconds)
        .await
        .map_err(storage_error)?
    else {
        return Ok(false);
    };
    let outcome = execute_claimed_job(
        persistence,
        access,
        control_channel,
        dispatcher,
        &claimed.job,
        now_unix_seconds,
    )
    .await;
    match outcome {
        Ok(()) => {
            persistence
                .complete_derived_index_job(
                    claimed.job.job_id,
                    &claimed.worker_id,
                    now_unix_seconds,
                )
                .await
                .map_err(storage_error)?;
        }
        Err(ExecutionErrorV1::Failure(failure)) => {
            persistence
                .fail_derived_index_job(
                    claimed.job.job_id,
                    &claimed.worker_id,
                    failure,
                    now_unix_seconds,
                )
                .await
                .map_err(storage_error)?;
        }
        Err(ExecutionErrorV1::RetryPending) => {
            persistence
                .release_derived_index_job(claimed.job.job_id, &claimed.worker_id)
                .await
                .map_err(storage_error)?;
            return Ok(false);
        }
        Err(ExecutionErrorV1::StorageUnavailable) => {
            return Err(CommunicationsSearchWorkerErrorV1::StorageUnavailable);
        }
    }
    Ok(true)
}

async fn execute_claimed_job(
    persistence: &CommunicationsDurablePersistence,
    access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    job: &makosh_communications_persistence::CommunicationsDerivedIndexJobV1,
    now_unix_seconds: i64,
) -> Result<(), ExecutionErrorV1> {
    match job.operation {
        CommunicationsDerivedIndexJobOperationV1::Remove => persistence
            .remove_search_projection(
                job.evidence_id,
                job.message_id,
                job.projection_revision,
                job.observed_at_unix_seconds,
            )
            .await
            .map(|_| ())
            .map_err(|_| ExecutionErrorV1::StorageUnavailable),
        CommunicationsDerivedIndexJobOperationV1::Index => {
            let search_job = CommunicationsSearchIndexJobV1 {
                evidence_id: job.evidence_id,
                message_id: job.message_id,
                conversation_id: job.conversation_id.ok_or(ExecutionErrorV1::Failure(
                    CommunicationsDerivedIndexFailureV1::InvalidContent,
                ))?,
                blob: job.blob.clone().ok_or(ExecutionErrorV1::Failure(
                    CommunicationsDerivedIndexFailureV1::InvalidContent,
                ))?,
                observed_at_unix_seconds: job.observed_at_unix_seconds,
                projection_revision: job.projection_revision,
            };
            let key = access
                .ensure_index_key(control_channel, dispatcher)
                .map_err(vault_execution_error)?;
            let body = access
                .read_admitted_body(control_channel, dispatcher, &search_job.blob)
                .map_err(blob_execution_error)?;
            let document = std::str::from_utf8(&body).map_err(|_| {
                ExecutionErrorV1::Failure(CommunicationsDerivedIndexFailureV1::InvalidContent)
            })?;
            let projection =
                assemble_search_projection_write_v1(&search_job, document, &key, now_unix_seconds)
                    .map_err(|_| {
                        ExecutionErrorV1::Failure(
                            CommunicationsDerivedIndexFailureV1::InvalidContent,
                        )
                    })?;
            persistence
                .replace_search_projection(&projection)
                .await
                .map(|_| ())
                .map_err(|_| ExecutionErrorV1::StorageUnavailable)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExecutionErrorV1 {
    Failure(CommunicationsDerivedIndexFailureV1),
    RetryPending,
    StorageUnavailable,
}

fn vault_execution_error(error: CommunicationsSearchAccessErrorV1) -> ExecutionErrorV1 {
    match error {
        CommunicationsSearchAccessErrorV1::Admission
        | CommunicationsSearchAccessErrorV1::Denied => {
            ExecutionErrorV1::Failure(CommunicationsDerivedIndexFailureV1::VaultDenied)
        }
        CommunicationsSearchAccessErrorV1::Unavailable => ExecutionErrorV1::RetryPending,
    }
}

fn blob_execution_error(error: CommunicationsSearchAccessErrorV1) -> ExecutionErrorV1 {
    match error {
        CommunicationsSearchAccessErrorV1::Admission
        | CommunicationsSearchAccessErrorV1::Denied => {
            ExecutionErrorV1::Failure(CommunicationsDerivedIndexFailureV1::BlobDenied)
        }
        CommunicationsSearchAccessErrorV1::Unavailable => ExecutionErrorV1::RetryPending,
    }
}

fn storage_error(_: CommunicationsPersistenceError) -> CommunicationsSearchWorkerErrorV1 {
    CommunicationsSearchWorkerErrorV1::StorageUnavailable
}
