//! Kernel-fenced Blob redelegation and exact Text Extraction result materialization.

use std::{os::unix::net::UnixStream, time::Duration};

use makosh_attachment_security_persistence::{
    AttachmentSecurityTextDelegationWorkV1, ClaimedAttachmentSecurityTextDelegationV1,
};
use makosh_attachment_text_extraction_ingress::{
    ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_MODULE_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_OWNER_ID_V1, AttachmentTextCustodyEnvelopeContextV1,
    build_attachment_text_custody_delegated_outbox_record_v1,
    build_attachment_text_custody_delegation_rejected_outbox_record_v1,
    wire::{
        AttachmentTextCustodyDelegatedV1, AttachmentTextCustodyDelegationRejectCodeV1,
        AttachmentTextCustodyDelegationRejectedV1, RequestAttachmentTextCustodyDelegationV1,
    },
};
use makosh_blob_client::{
    BlobClientError, ManagedBlobCustodyDelegationRequestV1, ManagedBlobCustodyTargetV1,
    request_managed_blob_custody_delegation_v2,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, RejectManagedControlRequestsV2,
};

use crate::admission::{ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID, ATTACHMENT_SECURITY_MODULE_ID};

pub fn materialize_text_delegation_result_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedAttachmentSecurityTextDelegationV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> Result<OutboxRecordV1, AttachmentSecurityTextDelegationErrorV1> {
    let context = context(
        runtime_instance_id,
        runtime_generation,
        recorded_at_unix_seconds,
        recorded_at_nanos,
    );
    match &claimed.work {
        AttachmentSecurityTextDelegationWorkV1::Reject { request, code } => {
            build_rejected(claimed.command_message_id, request, *code, &context)
        }
        AttachmentSecurityTextDelegationWorkV1::Delegate {
            request,
            current_reference_id,
            current_receipt_sha256,
            declared_size,
            predecessor_custody_source_proof,
        } => {
            prepare_blocking_control_channel(control_channel)?;
            let mut dispatcher = RejectManagedControlRequestsV2;
            let request_id = id16(&request.request_id)?;
            let candidate_message_id = id16(&request.candidate_message_id)?;
            let candidate_envelope_sha256 = id32(&request.candidate_envelope_sha256)?;
            let result = request_managed_blob_custody_delegation_v2(
                control_channel,
                &mut dispatcher,
                ManagedBlobCustodyDelegationRequestV1 {
                    request_id: &request_id,
                    capability_id: ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID,
                    current_reference_id,
                    predecessor_custody_source_proof,
                    predecessor_evidence_id: &candidate_message_id,
                    predecessor_evidence_envelope_sha256: &candidate_envelope_sha256,
                    target: ManagedBlobCustodyTargetV1 {
                        owner_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_OWNER_ID_V1,
                        module_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_MODULE_ID_V1,
                        capability_id: ATTACHMENT_TEXT_EXTRACTION_BLOB_TARGET_CAPABILITY_ID_V1,
                    },
                },
            )
            .map_err(classify_blob_error);
            let restored = restore_nonblocking_control_channel(control_channel);
            let delegation = match (result, restored) {
                (Ok(value), Ok(())) => value,
                (Err(error), _) | (Ok(_), Err(error)) => return Err(error),
            };
            build_attachment_text_custody_delegated_outbox_record_v1(
                claimed.command_message_id,
                AttachmentTextCustodyDelegatedV1 {
                    request_id: request.request_id.clone(),
                    extraction_run_id: request.extraction_run_id.clone(),
                    attachment_anchor_id: request.attachment_anchor_id.clone(),
                    candidate_message_id: request.candidate_message_id.clone(),
                    safety_message_id: request.safety_message_id.clone(),
                    source_reference_id: current_reference_id.to_vec(),
                    declared_size: *declared_size,
                    receipt_sha256: current_receipt_sha256.to_vec(),
                    custody_transfer_source_proof: delegation.custody_transfer_source_proof,
                    logical_owner_id: request.logical_owner_id.clone(),
                },
                &context,
            )
            .map_err(|_| AttachmentSecurityTextDelegationErrorV1::InvalidEvidence)
        }
    }
}

pub fn materialize_text_delegation_exhausted_v1(
    claimed: &ClaimedAttachmentSecurityTextDelegationV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> Result<OutboxRecordV1, AttachmentSecurityTextDelegationErrorV1> {
    let request = match &claimed.work {
        AttachmentSecurityTextDelegationWorkV1::Delegate { request, .. }
        | AttachmentSecurityTextDelegationWorkV1::Reject { request, .. } => request,
    };
    build_rejected(
        claimed.command_message_id,
        request,
        AttachmentTextCustodyDelegationRejectCodeV1::AttachmentTextExtractionCustodyDelegationRejectCodeV1CustodyUnavailable,
        &context(
            runtime_instance_id,
            runtime_generation,
            recorded_at_unix_seconds,
            recorded_at_nanos,
        ),
    )
}

fn context(
    runtime_instance_id: &str,
    runtime_generation: u64,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> AttachmentTextCustodyEnvelopeContextV1 {
    AttachmentTextCustodyEnvelopeContextV1 {
        module_id: ATTACHMENT_SECURITY_MODULE_ID.to_owned(),
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        recorded_at_unix_seconds,
        recorded_at_nanos,
    }
}

fn build_rejected(
    command_message_id: [u8; 16],
    request: &RequestAttachmentTextCustodyDelegationV1,
    code: AttachmentTextCustodyDelegationRejectCodeV1,
    context: &AttachmentTextCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentSecurityTextDelegationErrorV1> {
    build_attachment_text_custody_delegation_rejected_outbox_record_v1(
        command_message_id,
        AttachmentTextCustodyDelegationRejectedV1 {
            request_id: request.request_id.clone(),
            extraction_run_id: request.extraction_run_id.clone(),
            attachment_anchor_id: request.attachment_anchor_id.clone(),
            code: code as i32,
            logical_owner_id: request.logical_owner_id.clone(),
        },
        context,
    )
    .map_err(|_| AttachmentSecurityTextDelegationErrorV1::InvalidEvidence)
}

fn prepare_blocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), AttachmentSecurityTextDelegationErrorV1> {
    channel
        .inner_mut()
        .set_nonblocking(false)
        .and_then(|_| {
            channel
                .inner_mut()
                .set_read_timeout(Some(Duration::from_secs(5)))
        })
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| AttachmentSecurityTextDelegationErrorV1::Unavailable)
}

fn restore_nonblocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), AttachmentSecurityTextDelegationErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| AttachmentSecurityTextDelegationErrorV1::Unavailable)
}

fn classify_blob_error(error: BlobClientError) -> AttachmentSecurityTextDelegationErrorV1 {
    match error {
        BlobClientError::Unavailable => AttachmentSecurityTextDelegationErrorV1::Unavailable,
        BlobClientError::Rejected(_) => AttachmentSecurityTextDelegationErrorV1::Rejected,
        _ => AttachmentSecurityTextDelegationErrorV1::InvalidEvidence,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentSecurityTextDelegationErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentSecurityTextDelegationErrorV1::InvalidEvidence)
}

fn id32(value: &[u8]) -> Result<[u8; 32], AttachmentSecurityTextDelegationErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentSecurityTextDelegationErrorV1::InvalidEvidence)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityTextDelegationErrorV1 {
    InvalidEvidence,
    Rejected,
    Unavailable,
}
