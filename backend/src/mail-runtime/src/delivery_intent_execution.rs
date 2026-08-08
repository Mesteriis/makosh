use std::{os::unix::net::UnixStream, time::Duration};

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_mail_api::MailSendMailRequestV1;
use makosh_mail_delivery_intent_contract::MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1;
use makosh_mail_persistence::{
    ClaimedMailDeliveryIntentJobV1, MailDeliveryIntentJobStateV1, MailDeliveryIntentJobV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};
use sha2::{Digest, Sha256};

use crate::managed::{MailAdmittedRuntime, MailBootstrapError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryIntentExecutionErrorV1 {
    InvalidJob,
    InvalidBody,
    ControlChannel,
    CustodyDenied,
    BlobUnavailable,
    QueueUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailDeliveryIntentTargetBodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
}

pub fn transfer_mail_delivery_intent_body_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedMailDeliveryIntentJobV1,
) -> Result<MailDeliveryIntentTargetBodyReceiptV1, MailDeliveryIntentExecutionErrorV1> {
    if claimed.state != MailDeliveryIntentJobStateV1::PendingCustody
        || claimed.target_body_reference_id.is_some()
        || claimed.target_body_receipt_sha256.is_some()
    {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    with_blocking_control_channel(control_channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let transfer = request_managed_blob_custody_transfer_v2(
            channel,
            &mut dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
                source_reference_id: &claimed.job.body_reference_id,
                declared_size: claimed.job.body_declared_bytes,
                receipt_sha256: &claimed.job.body_sha256,
                custody_source_proof: &claimed.job.custody_transfer_source_proof,
                evidence_id: &claimed.job.command_message_id,
                evidence_envelope_sha256: &claimed.job.command_envelope_sha256,
            },
        )
        .map_err(|_| MailDeliveryIntentExecutionErrorV1::CustodyDenied)?;
        let reference_id = transfer
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| MailDeliveryIntentExecutionErrorV1::CustodyDenied)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| MailDeliveryIntentExecutionErrorV1::BlobUnavailable)?;
        Ok(MailDeliveryIntentTargetBodyReceiptV1 {
            reference_id,
            receipt_sha256: claimed.job.body_sha256,
        })
    })
}

pub fn read_mail_delivery_intent_body_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedMailDeliveryIntentJobV1,
) -> Result<Vec<u8>, MailDeliveryIntentExecutionErrorV1> {
    if claimed.state != MailDeliveryIntentJobStateV1::BodyReady {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    let reference_id = claimed
        .target_body_reference_id
        .ok_or(MailDeliveryIntentExecutionErrorV1::InvalidJob)?;
    let receipt_sha256 = claimed
        .target_body_receipt_sha256
        .ok_or(MailDeliveryIntentExecutionErrorV1::InvalidJob)?;
    if receipt_sha256 != claimed.job.body_sha256 {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidBody);
    }
    let read_end = claimed.job.body_declared_bytes;
    let bytes = with_blocking_control_channel(control_channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: MAIL_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &reference_id,
                declared_size: claimed.job.body_declared_bytes,
                backup_class: 1,
                receipt_sha256: Some(&receipt_sha256),
                custody_target: None,
            },
        )
        .map_err(|_| MailDeliveryIntentExecutionErrorV1::CustodyDenied)?;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, read_end)
            })
            .map_err(|_| MailDeliveryIntentExecutionErrorV1::BlobUnavailable)
    })?;
    materialize_mail_delivery_intent_v1(&claimed.job, &bytes)?;
    Ok(bytes)
}

pub fn materialize_mail_delivery_intent_v1(
    job: &MailDeliveryIntentJobV1,
    body: &[u8],
) -> Result<MailSendMailRequestV1, MailDeliveryIntentExecutionErrorV1> {
    let body_len =
        u64::try_from(body.len()).map_err(|_| MailDeliveryIntentExecutionErrorV1::InvalidBody)?;
    if job.intent_id.iter().all(|byte| *byte == 0)
        || job.command_message_id.iter().all(|byte| *byte == 0)
        || job.connection_id.trim().is_empty()
        || job.provider_thread_id.trim().is_empty()
        || job.recipient.trim().is_empty()
        || job.subject.trim().is_empty()
        || job.body_reference_id.iter().all(|byte| *byte == 0)
        || job.provider_operation_id.trim().is_empty()
    {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    if body_len != job.body_declared_bytes || Sha256::digest(body).as_slice() != job.body_sha256 {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidBody);
    }
    let text_body = std::str::from_utf8(body)
        .map_err(|_| MailDeliveryIntentExecutionErrorV1::InvalidBody)?
        .to_owned();
    Ok(MailSendMailRequestV1 {
        operation_id: job.provider_operation_id.clone(),
        connection_id: job.connection_id.clone(),
        provider_conversation_id: job.provider_thread_id.clone(),
        recipients: vec![job.recipient.clone()],
        cc_recipients: Vec::new(),
        bcc_recipients: Vec::new(),
        subject: job.subject.clone(),
        text_body,
        attachment_anchor_ids: Vec::new(),
    })
}

pub async fn enqueue_mail_delivery_intent_v1(
    runtime: &mut MailAdmittedRuntime,
    job: &MailDeliveryIntentJobV1,
    body: &[u8],
    requested_at_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentExecutionErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(MailDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    let request = materialize_mail_delivery_intent_v1(job, body)?;
    runtime
        .select_account(&job.connection_id)
        .map_err(map_queue_error)?;
    let operation_id = runtime
        .submit_delivery(&request, requested_at_unix_seconds)
        .await
        .map_err(map_queue_error)?;
    if operation_id != job.provider_operation_id {
        return Err(MailDeliveryIntentExecutionErrorV1::QueueUnavailable);
    }
    Ok(())
}

fn map_queue_error(_: MailBootstrapError) -> MailDeliveryIntentExecutionErrorV1 {
    MailDeliveryIntentExecutionErrorV1::QueueUnavailable
}

fn with_blocking_control_channel<T>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    operation: impl FnOnce(
        &mut ManagedControlChannelV2<UnixStream>,
    ) -> Result<T, MailDeliveryIntentExecutionErrorV1>,
) -> Result<T, MailDeliveryIntentExecutionErrorV1> {
    if let Err(error) = prepare_blocking_control_channel(channel) {
        let _ = restore_nonblocking_control_channel(channel);
        return Err(error);
    }
    let result = operation(channel);
    let restored = restore_nonblocking_control_channel(channel);
    match (result, restored) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
    }
}

fn prepare_blocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), MailDeliveryIntentExecutionErrorV1> {
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
        .map_err(|_| MailDeliveryIntentExecutionErrorV1::ControlChannel)
}

fn restore_nonblocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), MailDeliveryIntentExecutionErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| MailDeliveryIntentExecutionErrorV1::ControlChannel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(body: &[u8]) -> MailDeliveryIntentJobV1 {
        MailDeliveryIntentJobV1 {
            intent_id: [1; 16],
            command_message_id: [2; 16],
            command_envelope_sha256: [5; 32],
            logical_owner_id: "owner-1".to_owned(),
            connection_id: "mail-account".to_owned(),
            provider_thread_id: "provider-thread".to_owned(),
            reply_to_provider_message_id: Some("provider-message".to_owned()),
            recipient: "recipient@example.com".to_owned(),
            subject: "Re: Subject".to_owned(),
            body_reference_id: [3; 16],
            body_declared_bytes: u64::try_from(body.len()).expect("test body length"),
            body_sha256: Sha256::digest(body).into(),
            custody_transfer_source_proof: vec![4; 32],
            provider_operation_id: "delivery-intent-01010101010101010101010101010101".to_owned(),
        }
    }

    #[test]
    fn materializes_exact_provider_route_into_existing_mail_queue_contract() {
        let body = b"Reply body";
        let request = materialize_mail_delivery_intent_v1(&job(body), body).expect("request");

        assert_eq!(request.connection_id, "mail-account");
        assert_eq!(request.provider_conversation_id, "provider-thread");
        assert_eq!(request.recipients, ["recipient@example.com"]);
        assert_eq!(request.subject, "Re: Subject");
        assert_eq!(request.text_body, "Reply body");
        assert!(request.cc_recipients.is_empty());
        assert!(request.bcc_recipients.is_empty());
        assert!(request.attachment_anchor_ids.is_empty());
    }

    #[test]
    fn rejects_body_bytes_that_do_not_match_the_admitted_receipt() {
        let admitted = job(b"expected");

        assert_eq!(
            materialize_mail_delivery_intent_v1(&admitted, b"different"),
            Err(MailDeliveryIntentExecutionErrorV1::InvalidBody),
        );
    }

    #[test]
    fn rejects_non_utf8_body_instead_of_reinterpreting_provider_content() {
        let body = [0xff, 0xfe];

        assert_eq!(
            materialize_mail_delivery_intent_v1(&job(&body), &body),
            Err(MailDeliveryIntentExecutionErrorV1::InvalidBody),
        );
    }
}
