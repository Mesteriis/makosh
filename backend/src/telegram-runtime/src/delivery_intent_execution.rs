use std::{os::unix::net::UnixStream, time::Duration};

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};
use makosh_telegram_api::{TelegramProviderCommand, TelegramSendMessage};
use makosh_telegram_delivery_intent_contract::TELEGRAM_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1;
use makosh_telegram_persistence::{
    ClaimedTelegramDeliveryIntentJobV1, TelegramDeliveryIntentJobStateV1,
    TelegramDeliveryIntentJobV1, TelegramDurablePersistence,
};
use sha2::{Digest, Sha256};

use crate::TelegramRuntime;
use makosh_telegram_tdlib::TdJsonTransport;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramDeliveryIntentExecutionErrorV1 {
    InvalidJob,
    InvalidBody,
    ControlChannel,
    CustodyDenied,
    BlobUnavailable,
    QueueUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelegramDeliveryIntentTargetBodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
}

pub fn transfer_telegram_delivery_intent_body_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
) -> Result<TelegramDeliveryIntentTargetBodyReceiptV1, TelegramDeliveryIntentExecutionErrorV1> {
    if claimed.state != TelegramDeliveryIntentJobStateV1::PendingCustody
        || claimed.target_body_reference_id.is_some()
        || claimed.target_body_receipt_sha256.is_some()
    {
        return Err(TelegramDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    with_blocking_control_channel(control_channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let transfer = request_managed_blob_custody_transfer_v2(
            channel,
            &mut dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: TELEGRAM_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
                source_reference_id: &claimed.job.body_reference_id,
                declared_size: claimed.job.body_declared_bytes,
                receipt_sha256: &claimed.job.body_sha256,
                custody_source_proof: &claimed.job.custody_transfer_source_proof,
                evidence_id: &claimed.job.command_message_id,
                evidence_envelope_sha256: &claimed.job.command_envelope_sha256,
            },
        )
        .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::CustodyDenied)?;
        let reference_id = transfer
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::CustodyDenied)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::BlobUnavailable)?;
        Ok(TelegramDeliveryIntentTargetBodyReceiptV1 {
            reference_id,
            receipt_sha256: claimed.job.body_sha256,
        })
    })
}

pub fn read_telegram_delivery_intent_body_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
) -> Result<Vec<u8>, TelegramDeliveryIntentExecutionErrorV1> {
    if claimed.state != TelegramDeliveryIntentJobStateV1::BodyReady {
        return Err(TelegramDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    let reference_id = claimed
        .target_body_reference_id
        .ok_or(TelegramDeliveryIntentExecutionErrorV1::InvalidJob)?;
    let receipt_sha256 = claimed
        .target_body_receipt_sha256
        .ok_or(TelegramDeliveryIntentExecutionErrorV1::InvalidJob)?;
    if receipt_sha256 != claimed.job.body_sha256 {
        return Err(TelegramDeliveryIntentExecutionErrorV1::InvalidBody);
    }
    let read_end = claimed.job.body_declared_bytes;
    let bytes = with_blocking_control_channel(control_channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: TELEGRAM_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &reference_id,
                declared_size: claimed.job.body_declared_bytes,
                backup_class: 1,
                receipt_sha256: Some(&receipt_sha256),
                custody_target: None,
            },
        )
        .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::CustodyDenied)?;
        BlobDataClient::new(session.data_socket_path)
            .and_then(|client| {
                client.read_range(session.grant, session.channel_binding, 0, read_end)
            })
            .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::BlobUnavailable)
    })?;
    materialize_telegram_delivery_intent_v1(&claimed.job, &bytes)?;
    Ok(bytes)
}

pub fn materialize_telegram_delivery_intent_v1(
    job: &TelegramDeliveryIntentJobV1,
    body: &[u8],
) -> Result<TelegramProviderCommand, TelegramDeliveryIntentExecutionErrorV1> {
    let body_len = u64::try_from(body.len())
        .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::InvalidBody)?;
    if job.intent_id.iter().all(|byte| *byte == 0)
        || job.command_message_id.iter().all(|byte| *byte == 0)
        || job.account_id.trim().is_empty()
        || job.provider_chat_id.trim().is_empty()
        || job
            .reply_to_provider_message_id
            .as_deref()
            .is_some_and(str::is_empty)
        || job.body_reference_id.iter().all(|byte| *byte == 0)
        || job.provider_operation_id.trim().is_empty()
    {
        return Err(TelegramDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    if body_len != job.body_declared_bytes || Sha256::digest(body).as_slice() != job.body_sha256 {
        return Err(TelegramDeliveryIntentExecutionErrorV1::InvalidBody);
    }
    let text = std::str::from_utf8(body)
        .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::InvalidBody)?
        .to_owned();
    Ok(
        if let Some(reply_to_provider_message_id) = &job.reply_to_provider_message_id {
            TelegramProviderCommand::Reply {
                operation_id: job.provider_operation_id.clone(),
                account_id: job.account_id.clone(),
                provider_chat_id: job.provider_chat_id.clone(),
                reply_to_provider_message_id: reply_to_provider_message_id.clone(),
                text,
            }
        } else {
            TelegramProviderCommand::SendText(TelegramSendMessage {
                operation_id: job.provider_operation_id.clone(),
                account_id: job.account_id.clone(),
                provider_chat_id: job.provider_chat_id.clone(),
                text,
            })
        },
    )
}

pub async fn enqueue_telegram_delivery_intent_v1(
    runtime: &mut TelegramRuntime<TdJsonTransport>,
    durable: &TelegramDurablePersistence,
    job: &TelegramDeliveryIntentJobV1,
    body: &[u8],
    requested_at_unix_seconds: i64,
) -> Result<(), TelegramDeliveryIntentExecutionErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(TelegramDeliveryIntentExecutionErrorV1::InvalidJob);
    }
    let command = materialize_telegram_delivery_intent_v1(job, body)?;
    let operation = runtime
        .execute_provider_command_durable(durable, command)
        .await
        .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::QueueUnavailable)?;
    if operation.operation_id != job.provider_operation_id {
        return Err(TelegramDeliveryIntentExecutionErrorV1::QueueUnavailable);
    }
    Ok(())
}

fn with_blocking_control_channel<T>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    operation: impl FnOnce(
        &mut ManagedControlChannelV2<UnixStream>,
    ) -> Result<T, TelegramDeliveryIntentExecutionErrorV1>,
) -> Result<T, TelegramDeliveryIntentExecutionErrorV1> {
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
) -> Result<(), TelegramDeliveryIntentExecutionErrorV1> {
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
        .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::ControlChannel)
}

fn restore_nonblocking_control_channel(
    channel: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<(), TelegramDeliveryIntentExecutionErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| TelegramDeliveryIntentExecutionErrorV1::ControlChannel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(body: &[u8]) -> TelegramDeliveryIntentJobV1 {
        TelegramDeliveryIntentJobV1 {
            intent_id: [1; 16],
            command_message_id: [2; 16],
            command_envelope_sha256: [5; 32],
            logical_owner_id: "owner-1".to_owned(),
            account_id: "telegram-account".to_owned(),
            provider_chat_id: "provider-chat".to_owned(),
            reply_to_provider_message_id: Some("provider-message".to_owned()),
            body_reference_id: [3; 16],
            body_declared_bytes: u64::try_from(body.len()).expect("test body length"),
            body_sha256: Sha256::digest(body).into(),
            custody_transfer_source_proof: vec![4; 32],
            provider_operation_id: "delivery-intent-01010101010101010101010101010101".to_owned(),
        }
    }

    #[test]
    fn materializes_exact_provider_route_into_existing_telegram_queue_contract() {
        let body = b"Reply body";
        let request = materialize_telegram_delivery_intent_v1(&job(body), body).expect("request");

        assert_eq!(
            request,
            TelegramProviderCommand::Reply {
                operation_id: "delivery-intent-01010101010101010101010101010101".to_owned(),
                account_id: "telegram-account".to_owned(),
                provider_chat_id: "provider-chat".to_owned(),
                reply_to_provider_message_id: "provider-message".to_owned(),
                text: "Reply body".to_owned(),
            }
        );
    }

    #[test]
    fn materializes_a_non_reply_as_the_existing_send_text_command() {
        let body = b"New message";
        let mut job = job(body);
        job.reply_to_provider_message_id = None;

        assert_eq!(
            materialize_telegram_delivery_intent_v1(&job, body),
            Ok(TelegramProviderCommand::SendText(TelegramSendMessage {
                operation_id: job.provider_operation_id,
                account_id: "telegram-account".to_owned(),
                provider_chat_id: "provider-chat".to_owned(),
                text: "New message".to_owned(),
            }))
        );
    }

    #[test]
    fn rejects_body_bytes_that_do_not_match_the_admitted_receipt() {
        let admitted = job(b"expected");

        assert_eq!(
            materialize_telegram_delivery_intent_v1(&admitted, b"different"),
            Err(TelegramDeliveryIntentExecutionErrorV1::InvalidBody),
        );
    }

    #[test]
    fn rejects_non_utf8_body_instead_of_reinterpreting_provider_content() {
        let body = [0xff, 0xfe];

        assert_eq!(
            materialize_telegram_delivery_intent_v1(&job(&body), &body),
            Err(TelegramDeliveryIntentExecutionErrorV1::InvalidBody),
        );
    }
}
