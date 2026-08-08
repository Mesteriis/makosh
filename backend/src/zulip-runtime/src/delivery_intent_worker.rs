//! Zulip-owned delivery-intent execution lifecycle.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use makosh_zulip_api::{ZulipCommandOperationOutcomeV1, ZulipCommandOperationStatusV1};
use makosh_zulip_delivery_intent_contract::wire::ZulipDeliveryIntentRejectCodeV1;
use makosh_zulip_persistence::{
    ClaimedZulipDeliveryIntentJobV1, ZULIP_DELIVERY_INTENT_MAX_ATTEMPTS_V1,
    ZulipDeliveryIntentJobStateV1, ZulipDeliveryIntentStoreV1, ZulipDurablePersistence,
};
use sha2::{Digest, Sha256};

use crate::{
    delivery_intent_execution::{
        ZulipDeliveryIntentExecutionErrorV1, enqueue_zulip_delivery_intent_v1,
        read_zulip_delivery_intent_body_v1, transfer_zulip_delivery_intent_body_v1,
    },
    delivery_intent_result::{
        ZulipDeliveryIntentResultContextV1, build_zulip_delivery_intent_rejected_outbox_v1,
        build_zulip_delivery_intent_succeeded_outbox_v1,
    },
};

const DELIVERY_INTENT_LEASE_SECONDS: i64 = 30;
const DELIVERY_INTENT_RETRY_SECONDS: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipDeliveryIntentWorkerErrorV1 {
    InvalidClock,
    InvalidRuntime,
    Persistence,
    ResultEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipDeliveryIntentWorkerContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
}

pub async fn process_next_zulip_delivery_intent_v1(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    durable: &ZulipDurablePersistence,
    context: &ZulipDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<bool, ZulipDeliveryIntentWorkerErrorV1> {
    let lease_expires_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_LEASE_SECONDS)
        .ok_or(ZulipDeliveryIntentWorkerErrorV1::InvalidClock)?;
    if now_unix_seconds <= 0 {
        return Err(ZulipDeliveryIntentWorkerErrorV1::InvalidClock);
    }
    if context.runtime_instance_id.trim().is_empty()
        || context.runtime_instance_id.len() > 256
        || context.runtime_generation == 0
    {
        return Err(ZulipDeliveryIntentWorkerErrorV1::InvalidRuntime);
    }
    let store = durable.delivery_intent_store();
    let worker_id = worker_id(&context.runtime_instance_id, context.runtime_generation);
    let Some(claimed) = store
        .claim_next_job(&worker_id, now_unix_seconds, lease_expires_at)
        .await
        .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence)?
    else {
        return Ok(false);
    };

    match claimed.state {
        ZulipDeliveryIntentJobStateV1::PendingCustody => {
            process_pending_custody(control_channel, &store, &claimed, context, now_unix_seconds)
                .await?
        }
        ZulipDeliveryIntentJobStateV1::BodyReady => {
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
        ZulipDeliveryIntentJobStateV1::DeliveryQueued => {
            process_delivery_queued(durable, &store, &claimed, context, now_unix_seconds).await?
        }
        ZulipDeliveryIntentJobStateV1::Succeeded
        | ZulipDeliveryIntentJobStateV1::Rejected
        | ZulipDeliveryIntentJobStateV1::OutcomeUnknown => {
            return Err(ZulipDeliveryIntentWorkerErrorV1::Persistence);
        }
    }
    Ok(true)
}

async fn process_pending_custody(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    store: &ZulipDeliveryIntentStoreV1,
    claimed: &ClaimedZulipDeliveryIntentJobV1,
    context: &ZulipDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), ZulipDeliveryIntentWorkerErrorV1> {
    match transfer_zulip_delivery_intent_body_v1(control_channel, claimed) {
        Ok(receipt) => store
            .record_target_body_receipt(
                claimed.job.intent_id,
                &claimed.worker_id,
                receipt.reference_id,
                receipt.receipt_sha256,
                now_unix_seconds,
            )
            .await
            .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence),
        Err(ZulipDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                store,
                claimed,
                context,
                ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeInvalidRequest,
                ZulipDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(ZulipDeliveryIntentExecutionErrorV1::CustodyDenied) => {
            complete_rejected(
                store,
                claimed,
                context,
                ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeCustodyRejected,
                ZulipDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await,
    }
}

async fn process_body_ready(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    durable: &ZulipDurablePersistence,
    store: &ZulipDeliveryIntentStoreV1,
    claimed: &ClaimedZulipDeliveryIntentJobV1,
    context: &ZulipDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), ZulipDeliveryIntentWorkerErrorV1> {
    let body = match read_zulip_delivery_intent_body_v1(control_channel, claimed) {
        Ok(body) => body,
        Err(
            ZulipDeliveryIntentExecutionErrorV1::InvalidJob
            | ZulipDeliveryIntentExecutionErrorV1::InvalidBody,
        ) => {
            return complete_rejected(
                store,
                claimed,
                context,
                ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeInvalidRequest,
                ZulipDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await;
        }
        Err(_) => {
            return retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await;
        }
    };
    match enqueue_zulip_delivery_intent_v1(durable, &claimed.job, &body, now_unix_seconds).await {
        Ok(()) => store
            .mark_delivery_queued(claimed.job.intent_id, &claimed.worker_id, now_unix_seconds)
            .await
            .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence),
        Err(ZulipDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                store,
                claimed,
                context,
                ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeInvalidRequest,
                ZulipDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(store, claimed, context, now_unix_seconds).await,
    }
}

async fn process_delivery_queued(
    durable: &ZulipDurablePersistence,
    store: &ZulipDeliveryIntentStoreV1,
    claimed: &ClaimedZulipDeliveryIntentJobV1,
    worker_context: &ZulipDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), ZulipDeliveryIntentWorkerErrorV1> {
    let status = durable
        .command_operation_status(&claimed.job.provider_operation_id)
        .await
        .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence)?;
    let Some(status) = status else {
        return reschedule(store, claimed, now_unix_seconds).await;
    };
    match status.outcome {
        ZulipCommandOperationOutcomeV1::Accepted { .. } => {
            let completed_at = operation_completed_at(&status, now_unix_seconds)?;
            let context = result_context(worker_context, completed_at);
            let result = build_zulip_delivery_intent_succeeded_outbox_v1(
                &claimed.job,
                claimed.attempt_count.max(1),
                &context,
            )
            .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
            store
                .complete_claimed_job(
                    claimed.job.intent_id,
                    &claimed.worker_id,
                    ZulipDeliveryIntentJobStateV1::Succeeded,
                    &result,
                    context.completed_at_unix_seconds,
                )
                .await
                .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence)
        }
        ZulipCommandOperationOutcomeV1::Rejected => {
            complete_rejected(
                store,
                claimed,
                worker_context,
                ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeProviderRejected,
                ZulipDeliveryIntentJobStateV1::Rejected,
                operation_completed_at(&status, now_unix_seconds)?,
            )
            .await
        }
        ZulipCommandOperationOutcomeV1::OutcomeUnknown => {
            match durable
                .command_operation_was_dispatched(&claimed.job.provider_operation_id)
                .await
                .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence)?
            {
                Some(true) => complete_rejected(
                    store,
                    claimed,
                    worker_context,
                    ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeProviderAmbiguous,
                    ZulipDeliveryIntentJobStateV1::OutcomeUnknown,
                    operation_completed_at(&status, now_unix_seconds)?,
                )
                .await,
                Some(false) => reschedule(store, claimed, now_unix_seconds).await,
                None => Err(ZulipDeliveryIntentWorkerErrorV1::Persistence),
            }
        }
    }
}

async fn complete_rejected(
    store: &ZulipDeliveryIntentStoreV1,
    claimed: &ClaimedZulipDeliveryIntentJobV1,
    worker_context: &ZulipDeliveryIntentWorkerContextV1,
    code: ZulipDeliveryIntentRejectCodeV1,
    terminal_state: ZulipDeliveryIntentJobStateV1,
    completed_at_unix_seconds: i64,
) -> Result<(), ZulipDeliveryIntentWorkerErrorV1> {
    let context = result_context(worker_context, completed_at_unix_seconds);
    let result = build_zulip_delivery_intent_rejected_outbox_v1(
        &claimed.job,
        code,
        claimed.attempt_count.max(1),
        &context,
    )
    .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
    store
        .complete_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            terminal_state,
            &result,
            completed_at_unix_seconds,
        )
        .await
        .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence)
}

async fn reschedule(
    store: &ZulipDeliveryIntentStoreV1,
    claimed: &ClaimedZulipDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), ZulipDeliveryIntentWorkerErrorV1> {
    let next_attempt_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_RETRY_SECONDS)
        .ok_or(ZulipDeliveryIntentWorkerErrorV1::InvalidClock)?;
    store
        .reschedule_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            claimed.state,
            next_attempt_at,
        )
        .await
        .map_err(|_| ZulipDeliveryIntentWorkerErrorV1::Persistence)
}

async fn retry_or_complete_unavailable(
    store: &ZulipDeliveryIntentStoreV1,
    claimed: &ClaimedZulipDeliveryIntentJobV1,
    context: &ZulipDeliveryIntentWorkerContextV1,
    now_unix_seconds: i64,
) -> Result<(), ZulipDeliveryIntentWorkerErrorV1> {
    if claimed.attempt_count
        >= u32::try_from(ZULIP_DELIVERY_INTENT_MAX_ATTEMPTS_V1)
            .expect("positive Zulip delivery-intent attempt limit")
    {
        complete_rejected(
            store,
            claimed,
            context,
            ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeUnavailable,
            ZulipDeliveryIntentJobStateV1::Rejected,
            now_unix_seconds,
        )
        .await
    } else {
        reschedule(store, claimed, now_unix_seconds).await
    }
}

fn result_context(
    context: &ZulipDeliveryIntentWorkerContextV1,
    completed_at_unix_seconds: i64,
) -> ZulipDeliveryIntentResultContextV1 {
    ZulipDeliveryIntentResultContextV1 {
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        completed_at_unix_seconds,
        completed_at_nanos: 0,
    }
}

fn operation_completed_at(
    operation: &ZulipCommandOperationStatusV1,
    fallback_unix_seconds: i64,
) -> Result<i64, ZulipDeliveryIntentWorkerErrorV1> {
    operation
        .completed_at_unix_seconds
        .map_or(Ok(fallback_unix_seconds), Ok)
}

fn worker_id(runtime_instance_id: &str, runtime_generation: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.zulip.delivery-intent.worker.v1");
    hasher.update(runtime_instance_id.as_bytes());
    hasher.update(runtime_generation.to_be_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    let suffix = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("zulip-delivery-intent-{suffix}")
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
