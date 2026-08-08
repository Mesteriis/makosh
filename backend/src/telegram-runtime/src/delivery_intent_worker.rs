//! Telegram-owned delivery-intent execution lifecycle.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use makosh_telegram_api::TelegramOperationState;
use makosh_telegram_delivery_intent_contract::wire::TelegramDeliveryIntentRejectCodeV1;
use makosh_telegram_persistence::{
    ClaimedTelegramDeliveryIntentJobV1, TELEGRAM_DELIVERY_INTENT_MAX_ATTEMPTS_V1,
    TelegramDeliveryIntentJobStateV1, TelegramDeliveryIntentStoreV1, TelegramDurablePersistence,
};
use makosh_telegram_tdlib::TdJsonTransport;
use sha2::{Digest, Sha256};

use crate::{
    TelegramRuntime,
    delivery_intent_execution::{
        TelegramDeliveryIntentExecutionErrorV1, enqueue_telegram_delivery_intent_v1,
        read_telegram_delivery_intent_body_v1, transfer_telegram_delivery_intent_body_v1,
    },
    delivery_intent_result::{
        TelegramDeliveryIntentResultContextV1, build_telegram_delivery_intent_rejected_outbox_v1,
        build_telegram_delivery_intent_succeeded_outbox_v1,
    },
};

const DELIVERY_INTENT_LEASE_SECONDS: i64 = 30;
const DELIVERY_INTENT_RETRY_SECONDS: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramDeliveryIntentWorkerErrorV1 {
    InvalidClock,
    InvalidRuntime,
    Persistence,
    ResultEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramDeliveryIntentWorkerContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
}

pub async fn process_next_telegram_delivery_intent_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    runtime: &mut TelegramRuntime<TdJsonTransport>,
    durable: &TelegramDurablePersistence,
    context: &TelegramDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<bool, TelegramDeliveryIntentWorkerErrorV1> {
    let lease_expires_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_LEASE_SECONDS)
        .ok_or(TelegramDeliveryIntentWorkerErrorV1::InvalidClock)?;
    if now_unix_seconds <= 0 {
        return Err(TelegramDeliveryIntentWorkerErrorV1::InvalidClock);
    }
    if context.runtime_instance_id.trim().is_empty()
        || context.runtime_instance_id.len() > 256
        || context.runtime_generation == 0
    {
        return Err(TelegramDeliveryIntentWorkerErrorV1::InvalidRuntime);
    }
    let store = durable.delivery_intent_store();
    let worker_id = worker_id(&context.runtime_instance_id, context.runtime_generation);
    let Some(claimed) = store
        .claim_next_job(&worker_id, now_unix_seconds, lease_expires_at)
        .await
        .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::Persistence)?
    else {
        return Ok(false);
    };

    match claimed.state {
        TelegramDeliveryIntentJobStateV1::PendingCustody => {
            process_pending_custody(control_channel, &store, &claimed, context, now_unix_seconds)
                .await?
        }
        TelegramDeliveryIntentJobStateV1::BodyReady => {
            process_body_ready(
                control_channel,
                runtime,
                durable,
                &store,
                &claimed,
                context,
                now_unix_seconds,
            )
            .await?
        }
        TelegramDeliveryIntentJobStateV1::DeliveryQueued => {
            process_delivery_queued(durable, &store, &claimed, context, now_unix_seconds).await?
        }
        TelegramDeliveryIntentJobStateV1::Succeeded
        | TelegramDeliveryIntentJobStateV1::Rejected
        | TelegramDeliveryIntentJobStateV1::OutcomeUnknown => {
            return Err(TelegramDeliveryIntentWorkerErrorV1::Persistence);
        }
    }
    Ok(true)
}

async fn process_pending_custody(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    store: &TelegramDeliveryIntentStoreV1,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
    context: &TelegramDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), TelegramDeliveryIntentWorkerErrorV1> {
    match transfer_telegram_delivery_intent_body_v1(control_channel, claimed) {
        Ok(receipt) => store
            .record_target_body_receipt(
                claimed.job.intent_id,
                &claimed.worker_id,
                receipt.reference_id,
                receipt.receipt_sha256,
                now_unix_seconds,
            )
            .await
            .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::Persistence),
        Err(TelegramDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                store,
                claimed,
                context,
                TelegramDeliveryIntentRejectCodeV1::TelegramDeliveryIntentRejectCodeInvalidRequest,
                TelegramDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(TelegramDeliveryIntentExecutionErrorV1::CustodyDenied) => {
            complete_rejected(
                store,
                claimed,
                context,
                TelegramDeliveryIntentRejectCodeV1::TelegramDeliveryIntentRejectCodeCustodyRejected,
                TelegramDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await,
    }
}

async fn process_body_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    runtime: &mut TelegramRuntime<TdJsonTransport>,
    durable: &TelegramDurablePersistence,
    store: &TelegramDeliveryIntentStoreV1,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
    context: &TelegramDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), TelegramDeliveryIntentWorkerErrorV1> {
    let body = match read_telegram_delivery_intent_body_v1(control_channel, claimed) {
        Ok(body) => body,
        Err(
            TelegramDeliveryIntentExecutionErrorV1::InvalidJob
            | TelegramDeliveryIntentExecutionErrorV1::InvalidBody,
        ) => {
            return complete_rejected(
                store,
                claimed,
                context,
                TelegramDeliveryIntentRejectCodeV1::TelegramDeliveryIntentRejectCodeInvalidRequest,
                TelegramDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await;
        }
        Err(_) => {
            return retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await;
        }
    };
    match enqueue_telegram_delivery_intent_v1(
        runtime,
        durable,
        &claimed.job,
        &body,
        now_unix_seconds,
    )
    .await
    {
        Ok(()) => store
            .mark_delivery_queued(claimed.job.intent_id, &claimed.worker_id, now_unix_seconds)
            .await
            .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::Persistence),
        Err(TelegramDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                store,
                claimed,
                context,
                TelegramDeliveryIntentRejectCodeV1::TelegramDeliveryIntentRejectCodeInvalidRequest,
                TelegramDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await,
    }
}

async fn process_delivery_queued(
    durable: &TelegramDurablePersistence,
    store: &TelegramDeliveryIntentStoreV1,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
    worker_context: &TelegramDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), TelegramDeliveryIntentWorkerErrorV1> {
    let status = durable
        .operation(&claimed.job.provider_operation_id)
        .await
        .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::Persistence)?;
    let Some(status) = status else {
        return reschedule(store, claimed, now_unix_seconds).await;
    };
    match status.state {
        TelegramOperationState::Accepted
        | TelegramOperationState::Running
        | TelegramOperationState::AwaitingProvider
        | TelegramOperationState::RetryScheduled => {
            reschedule(store, claimed, now_unix_seconds).await
        }
        TelegramOperationState::Completed => {
            let completed_at = operation_completed_at(&status, now_unix_seconds)?;
            let context = result_context(worker_context, completed_at);
            let result = build_telegram_delivery_intent_succeeded_outbox_v1(
                &claimed.job,
                claimed.attempt_count.max(1),
                &context,
            )
            .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
            store
                .complete_claimed_job(
                    claimed.job.intent_id,
                    &claimed.worker_id,
                    TelegramDeliveryIntentJobStateV1::Succeeded,
                    &result,
                    context.completed_at_unix_seconds,
                )
                .await
                .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::Persistence)
        }
        TelegramOperationState::Failed | TelegramOperationState::DeadLetter => complete_rejected(
            store,
            claimed,
            worker_context,
            TelegramDeliveryIntentRejectCodeV1::TelegramDeliveryIntentRejectCodeProviderAmbiguous,
            TelegramDeliveryIntentJobStateV1::OutcomeUnknown,
            operation_completed_at(&status, now_unix_seconds)?,
        )
        .await,
    }
}

async fn complete_rejected(
    store: &TelegramDeliveryIntentStoreV1,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
    worker_context: &TelegramDeliveryIntentWorkerContextV1,
    code: TelegramDeliveryIntentRejectCodeV1,
    terminal_state: TelegramDeliveryIntentJobStateV1,
    completed_at_unix_seconds: i64,
) -> Result<(), TelegramDeliveryIntentWorkerErrorV1> {
    let context = result_context(worker_context, completed_at_unix_seconds);
    let result = build_telegram_delivery_intent_rejected_outbox_v1(
        &claimed.job,
        code,
        claimed.attempt_count.max(1),
        &context,
    )
    .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
    store
        .complete_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            terminal_state,
            &result,
            completed_at_unix_seconds,
        )
        .await
        .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::Persistence)
}

async fn reschedule(
    store: &TelegramDeliveryIntentStoreV1,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), TelegramDeliveryIntentWorkerErrorV1> {
    let next_attempt_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_RETRY_SECONDS)
        .ok_or(TelegramDeliveryIntentWorkerErrorV1::InvalidClock)?;
    store
        .reschedule_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            claimed.state,
            next_attempt_at,
        )
        .await
        .map_err(|_| TelegramDeliveryIntentWorkerErrorV1::Persistence)
}

async fn retry_or_complete_unavailable(
    store: &TelegramDeliveryIntentStoreV1,
    claimed: &ClaimedTelegramDeliveryIntentJobV1,
    context: &TelegramDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), TelegramDeliveryIntentWorkerErrorV1> {
    if claimed.attempt_count
        >= u32::try_from(TELEGRAM_DELIVERY_INTENT_MAX_ATTEMPTS_V1)
            .expect("positive Telegram delivery-intent attempt limit")
    {
        complete_rejected(
            store,
            claimed,
            context,
            TelegramDeliveryIntentRejectCodeV1::TelegramDeliveryIntentRejectCodeUnavailable,
            TelegramDeliveryIntentJobStateV1::Rejected,
            now_unix_seconds,
        )
        .await
    } else {
        reschedule(store, claimed, now_unix_seconds).await
    }
}

fn result_context(
    context: &TelegramDeliveryIntentWorkerContextV1,
    completed_at_unix_seconds: i64,
) -> TelegramDeliveryIntentResultContextV1 {
    TelegramDeliveryIntentResultContextV1 {
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        completed_at_unix_seconds,
        completed_at_nanos: 0,
    }
}

fn operation_completed_at(
    operation: &makosh_telegram_api::TelegramOperation,
    fallback_unix_seconds: i64,
) -> Result<i64, TelegramDeliveryIntentWorkerErrorV1> {
    operation
        .reconciled_at_unix_seconds
        .or(operation.provider_observed_at_unix_seconds)
        .map_or(Ok(fallback_unix_seconds), |value| {
            i64::try_from(value).map_err(|_| TelegramDeliveryIntentWorkerErrorV1::InvalidClock)
        })
}

fn worker_id(runtime_instance_id: &str, runtime_generation: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.telegram.delivery-intent.worker.v1");
    hasher.update(runtime_instance_id.as_bytes());
    hasher.update(runtime_generation.to_be_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    let suffix = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("telegram-delivery-intent-{suffix}")
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
