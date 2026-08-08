//! Executes an owner-local Blob custody transfer for one admitted body.

use std::os::unix::net::UnixStream;

use hermes_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyTransferRequestV1,
    request_managed_blob_custody_transfer_v2,
};
use hermes_communications_api::CommunicationBodyBlobReferenceV1;
use hermes_communications_persistence::{
    CommunicationsBodyCustodyTransferErrorV1, CommunicationsDurablePersistence,
};
use hermes_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};

use crate::admission::COMMUNICATIONS_BLOB_CAPABILITY_ID;

const CUSTODY_TRANSFER_LEASE_SECONDS: i64 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsCustodyWorkerErrorV1 {
    StorageUnavailable,
    RetryPending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlobCustodyTransferFailureV1 {
    PolicyRejected,
    RetryPending,
}

pub async fn process_next_body_custody_transfer_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    persistence: &CommunicationsDurablePersistence,
    worker_id: &str,
    now_unix_seconds: i64,
) -> Result<bool, CommunicationsCustodyWorkerErrorV1> {
    let lease_expires_at_unix_seconds = now_unix_seconds
        .checked_add(CUSTODY_TRANSFER_LEASE_SECONDS)
        .ok_or(CommunicationsCustodyWorkerErrorV1::StorageUnavailable)?;
    let Some(claimed) = persistence
        .claim_next_body_custody_transfer(
            worker_id,
            now_unix_seconds,
            lease_expires_at_unix_seconds,
        )
        .await
        .map_err(|error| storage_error_at("claim", error))?
    else {
        return Ok(false);
    };
    if persistence
        .reuse_completed_body_custody_transfer(&claimed, now_unix_seconds)
        .await
        .map_err(|error| storage_error_at("reuse", error))?
    {
        return Ok(true);
    }

    control_channel
        .inner_mut()
        .set_nonblocking(false)
        .map_err(|_| unavailable_at("control_blocking"))?;
    let transfer = (|| {
        let session = request_managed_blob_custody_transfer_v2(
            control_channel,
            dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: COMMUNICATIONS_BLOB_CAPABILITY_ID,
                source_reference_id: &claimed.source_reference_id,
                declared_size: claimed.declared_bytes,
                receipt_sha256: &claimed.plaintext_sha256,
                custody_source_proof: &claimed.source_custody_proof,
                evidence_id: &claimed.evidence_id.bytes(),
                evidence_envelope_sha256: &claimed.envelope_sha256,
            },
        )?;
        let target_reference_id = session
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| BlobClientError::InvalidResponse)?;
        BlobDataClient::new(&session.data_socket_path)?
            .custody_transfer(session.grant, session.channel_binding)?;
        Ok::<[u8; 16], BlobClientError>(target_reference_id)
    })();
    control_channel
        .inner_mut()
        .set_nonblocking(true)
        .map_err(|_| unavailable_at("control_restore"))?;

    let target_reference_id = match transfer {
        Ok(reference_id) => reference_id,
        Err(error) => {
            if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_communications_custody_blob_error={error:?}");
            }
            match blob_transfer_failure(error) {
                BlobCustodyTransferFailureV1::PolicyRejected => {
                    persistence
                        .fail_body_custody_transfer(&claimed, now_unix_seconds)
                        .await
                        .map_err(|error| storage_error_at("fail", error))?;
                    return Ok(true);
                }
                BlobCustodyTransferFailureV1::RetryPending => {
                    persistence
                        .release_body_custody_transfer(&claimed)
                        .await
                        .map_err(|error| storage_error_at("release", error))?;
                    return Err(CommunicationsCustodyWorkerErrorV1::RetryPending);
                }
            }
        }
    };
    let blob_ref = format!(
        "blob-content:{}",
        target_reference_id
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    persistence
        .complete_body_custody_transfer(
            &claimed,
            CommunicationBodyBlobReferenceV1 {
                blob_ref,
                reference_id: target_reference_id,
                declared_bytes: claimed.declared_bytes,
                sha256: claimed.plaintext_sha256,
            },
            now_unix_seconds,
        )
        .await
        .map_err(|error| storage_error_at("complete", error))?;
    Ok(true)
}

fn storage_error(
    _: CommunicationsBodyCustodyTransferErrorV1,
) -> CommunicationsCustodyWorkerErrorV1 {
    CommunicationsCustodyWorkerErrorV1::StorageUnavailable
}

fn storage_error_at(
    stage: &str,
    error: CommunicationsBodyCustodyTransferErrorV1,
) -> CommunicationsCustodyWorkerErrorV1 {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_custody_storage_error stage={stage} error={error:?}");
    }
    storage_error(error)
}

fn unavailable_at(stage: &str) -> CommunicationsCustodyWorkerErrorV1 {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_communications_custody_storage_error stage={stage}");
    }
    CommunicationsCustodyWorkerErrorV1::StorageUnavailable
}

fn blob_transfer_failure(error: BlobClientError) -> BlobCustodyTransferFailureV1 {
    match error {
        BlobClientError::Rejected(_)
        | BlobClientError::InvalidCustodyDelegationRequest
        | BlobClientError::InvalidCustodyReleaseRequest => {
            BlobCustodyTransferFailureV1::PolicyRejected
        }
        BlobClientError::InvalidSocketPath
        | BlobClientError::InvalidTimeout
        | BlobClientError::Connect(_)
        | BlobClientError::Io(_)
        | BlobClientError::FrameTooLarge
        | BlobClientError::InvalidFrame
        | BlobClientError::InvalidResponse
        | BlobClientError::InvalidSessionRequest
        | BlobClientError::Unavailable => BlobCustodyTransferFailureV1::RetryPending,
    }
}

#[cfg(test)]
mod tests {
    use super::{BlobCustodyTransferFailureV1, blob_transfer_failure};
    use hermes_blob_client::BlobClientError;

    #[test]
    fn blob_unavailability_keeps_custody_transfer_pending() {
        assert_eq!(
            blob_transfer_failure(BlobClientError::Unavailable),
            BlobCustodyTransferFailureV1::RetryPending,
        );
    }

    #[test]
    fn rejected_custody_transfer_remains_terminal() {
        assert_eq!(
            blob_transfer_failure(BlobClientError::Rejected("denied".to_owned())),
            BlobCustodyTransferFailureV1::PolicyRejected,
        );
        assert_eq!(
            blob_transfer_failure(BlobClientError::InvalidCustodyReleaseRequest),
            BlobCustodyTransferFailureV1::PolicyRejected,
        );
    }
}
