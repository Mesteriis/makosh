//! WhatsApp-owned delivery-intent execution lifecycle.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use makosh_whatsapp_delivery_intent_contract::wire::WhatsAppDeliveryIntentRejectCodeV1;
use makosh_whatsapp_persistence::{
    ClaimedWhatsAppDeliveryIntentJobV1, WHATSAPP_DELIVERY_INTENT_MAX_ATTEMPTS_V1,
    WhatsAppDeliveryIntentJobStateV1, WhatsAppDeliveryIntentStoreV1, WhatsAppDurablePersistence,
    WhatsAppProviderCommandStateV1,
};
use sha2::{Digest, Sha256};

use crate::{
    delivery_intent_execution::{
        WhatsAppDeliveryIntentExecutionErrorV1, enqueue_whatsapp_delivery_intent_v1,
        read_whatsapp_delivery_intent_body_v1, transfer_whatsapp_delivery_intent_body_v1,
    },
    delivery_intent_result::{
        WhatsAppDeliveryIntentResultContextV1, build_whatsapp_delivery_intent_rejected_outbox_v1,
        build_whatsapp_delivery_intent_succeeded_outbox_v1,
    },
};

const DELIVERY_INTENT_LEASE_SECONDS: i64 = 30;
const DELIVERY_INTENT_RETRY_SECONDS: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppDeliveryIntentWorkerErrorV1 {
    InvalidClock,
    InvalidRuntime,
    Persistence,
    ResultEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppDeliveryIntentWorkerContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
}

pub async fn process_next_whatsapp_delivery_intent_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    durable: &WhatsAppDurablePersistence,
    context: &WhatsAppDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<bool, WhatsAppDeliveryIntentWorkerErrorV1> {
    let lease_expires_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_LEASE_SECONDS)
        .ok_or(WhatsAppDeliveryIntentWorkerErrorV1::InvalidClock)?;
    if now_unix_seconds <= 0 {
        return Err(WhatsAppDeliveryIntentWorkerErrorV1::InvalidClock);
    }
    if context.runtime_instance_id.trim().is_empty()
        || context.runtime_instance_id.len() > 256
        || context.runtime_generation == 0
    {
        return Err(WhatsAppDeliveryIntentWorkerErrorV1::InvalidRuntime);
    }
    let store = durable.delivery_intent_store();
    let worker_id = worker_id(&context.runtime_instance_id, context.runtime_generation);
    let Some(claimed) = store
        .claim_next_job(&worker_id, now_unix_seconds, lease_expires_at)
        .await
        .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::Persistence)?
    else {
        return Ok(false);
    };

    match claimed.state {
        WhatsAppDeliveryIntentJobStateV1::PendingCustody => {
            process_pending_custody(control_channel, &store, &claimed, context, now_unix_seconds)
                .await?
        }
        WhatsAppDeliveryIntentJobStateV1::BodyReady => {
            process_body_ready(
                control_channel,
                durable,
                &store,
                &claimed,
                context,
                now_unix_seconds,
            )
            .await?
        }
        WhatsAppDeliveryIntentJobStateV1::DeliveryQueued => {
            process_delivery_queued(durable, &store, &claimed, context, now_unix_seconds).await?
        }
        WhatsAppDeliveryIntentJobStateV1::Succeeded
        | WhatsAppDeliveryIntentJobStateV1::Rejected
        | WhatsAppDeliveryIntentJobStateV1::OutcomeUnknown => {
            return Err(WhatsAppDeliveryIntentWorkerErrorV1::Persistence);
        }
    }
    Ok(true)
}

async fn process_pending_custody(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    store: &WhatsAppDeliveryIntentStoreV1,
    claimed: &ClaimedWhatsAppDeliveryIntentJobV1,
    context: &WhatsAppDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), WhatsAppDeliveryIntentWorkerErrorV1> {
    match transfer_whatsapp_delivery_intent_body_v1(control_channel, claimed) {
        Ok(receipt) => store
            .record_target_body_receipt(
                claimed.job.intent_id,
                &claimed.worker_id,
                receipt.reference_id,
                receipt.receipt_sha256,
                now_unix_seconds,
            )
            .await
            .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::Persistence),
        Err(WhatsAppDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                store,
                claimed,
                context,
                WhatsAppDeliveryIntentRejectCodeV1::WhatsappDeliveryIntentRejectCodeInvalidRequest,
                WhatsAppDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(WhatsAppDeliveryIntentExecutionErrorV1::CustodyDenied) => {
            complete_rejected(
                store,
                claimed,
                context,
                WhatsAppDeliveryIntentRejectCodeV1::WhatsappDeliveryIntentRejectCodeCustodyRejected,
                WhatsAppDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await,
    }
}

async fn process_body_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    durable: &WhatsAppDurablePersistence,
    store: &WhatsAppDeliveryIntentStoreV1,
    claimed: &ClaimedWhatsAppDeliveryIntentJobV1,
    context: &WhatsAppDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), WhatsAppDeliveryIntentWorkerErrorV1> {
    let body = match read_whatsapp_delivery_intent_body_v1(control_channel, claimed) {
        Ok(body) => body,
        Err(
            WhatsAppDeliveryIntentExecutionErrorV1::InvalidJob
            | WhatsAppDeliveryIntentExecutionErrorV1::InvalidBody,
        ) => {
            return complete_rejected(
                store,
                claimed,
                context,
                WhatsAppDeliveryIntentRejectCodeV1::WhatsappDeliveryIntentRejectCodeInvalidRequest,
                WhatsAppDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await;
        }
        Err(_) => {
            return retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await;
        }
    };
    match enqueue_whatsapp_delivery_intent_v1(durable, &claimed.job, &body, now_unix_seconds).await
    {
        Ok(()) => store
            .mark_delivery_queued(claimed.job.intent_id, &claimed.worker_id, now_unix_seconds)
            .await
            .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::Persistence),
        Err(WhatsAppDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                store,
                claimed,
                context,
                WhatsAppDeliveryIntentRejectCodeV1::WhatsappDeliveryIntentRejectCodeInvalidRequest,
                WhatsAppDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await,
    }
}

async fn process_delivery_queued(
    durable: &WhatsAppDurablePersistence,
    store: &WhatsAppDeliveryIntentStoreV1,
    claimed: &ClaimedWhatsAppDeliveryIntentJobV1,
    worker_context: &WhatsAppDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), WhatsAppDeliveryIntentWorkerErrorV1> {
    let status = durable
        .provider_command_status(&claimed.job.provider_operation_id)
        .await
        .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::Persistence)?;
    let Some(status) = status else {
        return reschedule(store, claimed, now_unix_seconds).await;
    };
    match status.state {
        WhatsAppProviderCommandStateV1::Pending | WhatsAppProviderCommandStateV1::Claimed => {
            reschedule(store, claimed, now_unix_seconds).await
        }
        WhatsAppProviderCommandStateV1::Succeeded => {
            let completed_at = operation_completed_at(&status, now_unix_seconds)?;
            let context = result_context(worker_context, completed_at);
            let result = build_whatsapp_delivery_intent_succeeded_outbox_v1(
                &claimed.job,
                claimed.attempt_count.max(1),
                &context,
            )
            .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
            store
                .complete_claimed_job(
                    claimed.job.intent_id,
                    &claimed.worker_id,
                    WhatsAppDeliveryIntentJobStateV1::Succeeded,
                    &result,
                    context.completed_at_unix_seconds,
                )
                .await
                .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::Persistence)
        }
        WhatsAppProviderCommandStateV1::Failed => complete_rejected(
            store,
            claimed,
            worker_context,
            WhatsAppDeliveryIntentRejectCodeV1::WhatsappDeliveryIntentRejectCodeProviderAmbiguous,
            WhatsAppDeliveryIntentJobStateV1::OutcomeUnknown,
            operation_completed_at(&status, now_unix_seconds)?,
        )
        .await,
    }
}

async fn complete_rejected(
    store: &WhatsAppDeliveryIntentStoreV1,
    claimed: &ClaimedWhatsAppDeliveryIntentJobV1,
    worker_context: &WhatsAppDeliveryIntentWorkerContextV1,
    code: WhatsAppDeliveryIntentRejectCodeV1,
    terminal_state: WhatsAppDeliveryIntentJobStateV1,
    completed_at_unix_seconds: i64,
) -> Result<(), WhatsAppDeliveryIntentWorkerErrorV1> {
    let context = result_context(worker_context, completed_at_unix_seconds);
    let result = build_whatsapp_delivery_intent_rejected_outbox_v1(
        &claimed.job,
        code,
        claimed.attempt_count.max(1),
        &context,
    )
    .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
    store
        .complete_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            terminal_state,
            &result,
            completed_at_unix_seconds,
        )
        .await
        .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::Persistence)
}

async fn reschedule(
    store: &WhatsAppDeliveryIntentStoreV1,
    claimed: &ClaimedWhatsAppDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), WhatsAppDeliveryIntentWorkerErrorV1> {
    let next_attempt_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_RETRY_SECONDS)
        .ok_or(WhatsAppDeliveryIntentWorkerErrorV1::InvalidClock)?;
    store
        .reschedule_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            claimed.state,
            next_attempt_at,
        )
        .await
        .map_err(|_| WhatsAppDeliveryIntentWorkerErrorV1::Persistence)
}

async fn retry_or_complete_unavailable(
    store: &WhatsAppDeliveryIntentStoreV1,
    claimed: &ClaimedWhatsAppDeliveryIntentJobV1,
    context: &WhatsAppDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), WhatsAppDeliveryIntentWorkerErrorV1> {
    if claimed.attempt_count
        >= u32::try_from(WHATSAPP_DELIVERY_INTENT_MAX_ATTEMPTS_V1)
            .expect("positive WhatsApp delivery-intent attempt limit")
    {
        complete_rejected(
            store,
            claimed,
            context,
            WhatsAppDeliveryIntentRejectCodeV1::WhatsappDeliveryIntentRejectCodeUnavailable,
            WhatsAppDeliveryIntentJobStateV1::Rejected,
            now_unix_seconds,
        )
        .await
    } else {
        reschedule(store, claimed, now_unix_seconds).await
    }
}

fn result_context(
    context: &WhatsAppDeliveryIntentWorkerContextV1,
    completed_at_unix_seconds: i64,
) -> WhatsAppDeliveryIntentResultContextV1 {
    WhatsAppDeliveryIntentResultContextV1 {
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        completed_at_unix_seconds,
        completed_at_nanos: 0,
    }
}

fn operation_completed_at(
    operation: &makosh_whatsapp_persistence::WhatsAppProviderCommandStatusV1,
    fallback_unix_seconds: i64,
) -> Result<i64, WhatsAppDeliveryIntentWorkerErrorV1> {
    operation
        .completed_at_unix_seconds
        .map_or(Ok(fallback_unix_seconds), |value| {
            (value > 0)
                .then_some(value)
                .ok_or(WhatsAppDeliveryIntentWorkerErrorV1::InvalidClock)
        })
}

fn worker_id(runtime_instance_id: &str, runtime_generation: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.whatsapp.delivery-intent.worker.v1");
    hasher.update(runtime_instance_id.as_bytes());
    hasher.update(runtime_generation.to_be_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    let suffix = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("whatsapp-delivery-intent-{suffix}")
}

#[cfg(test)]
mod tests {
    use super::worker_id;

    #[test]
    fn worker_identity_is_bounded_and_generation_fenced() {
        let first = worker_id(&"runtime".repeat(100), 1);
        let second = worker_id(&"runtime".repeat(100), 2);
        assert!(first.len() <= 128);
        assert_ne!(first, second);
    }
}
