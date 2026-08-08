//! Mail-owned delivery-intent execution lifecycle.

use makosh_mail_api::MailDeliveryOutcomeV1;
use makosh_mail_delivery_intent_contract::wire::MailDeliveryIntentRejectCodeV1;
use makosh_mail_persistence::{
    ClaimedMailDeliveryIntentJobV1, MAIL_DELIVERY_INTENT_MAX_ATTEMPTS_V1,
    MailDeliveryIntentJobStateV1, MailDeliveryIntentStoreV1,
};
use sha2::{Digest, Sha256};

use crate::{
    delivery_intent_execution::{
        MailDeliveryIntentExecutionErrorV1, enqueue_mail_delivery_intent_v1,
        read_mail_delivery_intent_body_v1, transfer_mail_delivery_intent_body_v1,
    },
    delivery_intent_result::{
        MailDeliveryIntentResultContextV1, build_mail_delivery_intent_rejected_outbox_v1,
        build_mail_delivery_intent_succeeded_outbox_v1,
    },
    managed::MailAdmittedRuntime,
};

const DELIVERY_INTENT_LEASE_SECONDS: i64 = 30;
const DELIVERY_INTENT_RETRY_SECONDS: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryIntentWorkerErrorV1 {
    InvalidClock,
    Persistence,
    Runtime,
    ResultEnvelope,
}

pub async fn process_next_mail_delivery_intent_v1(
    runtime: &mut MailAdmittedRuntime,
    now_unix_seconds: i64,
) -> Result<bool, MailDeliveryIntentWorkerErrorV1> {
    let lease_expires_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_LEASE_SECONDS)
        .ok_or(MailDeliveryIntentWorkerErrorV1::InvalidClock)?;
    if now_unix_seconds <= 0 {
        return Err(MailDeliveryIntentWorkerErrorV1::InvalidClock);
    }
    let store = runtime.durable.delivery_intent_store();
    let worker_id = worker_id(&runtime.runtime_instance_id, runtime.runtime_generation);
    let Some(claimed) = store
        .claim_next_job(&worker_id, now_unix_seconds, lease_expires_at)
        .await
        .map_err(|_| MailDeliveryIntentWorkerErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    runtime
        .select_account(&claimed.job.connection_id)
        .map_err(|_| MailDeliveryIntentWorkerErrorV1::Runtime)?;

    match claimed.state {
        MailDeliveryIntentJobStateV1::PendingCustody => {
            process_pending_custody(runtime, &store, &claimed, now_unix_seconds).await?
        }
        MailDeliveryIntentJobStateV1::BodyReady => {
            process_body_ready(runtime, &store, &claimed, now_unix_seconds).await?
        }
        MailDeliveryIntentJobStateV1::DeliveryQueued => {
            process_delivery_queued(runtime, &store, &claimed, now_unix_seconds).await?
        }
        MailDeliveryIntentJobStateV1::Succeeded
        | MailDeliveryIntentJobStateV1::Rejected
        | MailDeliveryIntentJobStateV1::OutcomeUnknown => {
            return Err(MailDeliveryIntentWorkerErrorV1::Persistence);
        }
    }
    Ok(true)
}

async fn process_pending_custody(
    runtime: &mut MailAdmittedRuntime,
    store: &MailDeliveryIntentStoreV1,
    claimed: &ClaimedMailDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentWorkerErrorV1> {
    match transfer_mail_delivery_intent_body_v1(&mut runtime.control_channel, claimed) {
        Ok(receipt) => store
            .record_target_body_receipt(
                claimed.job.intent_id,
                &claimed.worker_id,
                receipt.reference_id,
                receipt.receipt_sha256,
                now_unix_seconds,
            )
            .await
            .map_err(|_| MailDeliveryIntentWorkerErrorV1::Persistence),
        Err(MailDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                runtime,
                store,
                claimed,
                MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeInvalidRequest,
                MailDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(MailDeliveryIntentExecutionErrorV1::CustodyDenied) => {
            complete_rejected(
                runtime,
                store,
                claimed,
                MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeCustodyRejected,
                MailDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(runtime, store, claimed, now_unix_seconds).await,
    }
}

async fn process_body_ready(
    runtime: &mut MailAdmittedRuntime,
    store: &MailDeliveryIntentStoreV1,
    claimed: &ClaimedMailDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentWorkerErrorV1> {
    let body = match read_mail_delivery_intent_body_v1(&mut runtime.control_channel, claimed) {
        Ok(body) => body,
        Err(
            MailDeliveryIntentExecutionErrorV1::InvalidJob
            | MailDeliveryIntentExecutionErrorV1::InvalidBody,
        ) => {
            return complete_rejected(
                runtime,
                store,
                claimed,
                MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeInvalidRequest,
                MailDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await;
        }
        Err(_) => {
            return retry_or_complete_unavailable(runtime, store, claimed, now_unix_seconds).await;
        }
    };
    match enqueue_mail_delivery_intent_v1(runtime, &claimed.job, &body, now_unix_seconds).await {
        Ok(()) => store
            .mark_delivery_queued(claimed.job.intent_id, &claimed.worker_id, now_unix_seconds)
            .await
            .map_err(|_| MailDeliveryIntentWorkerErrorV1::Persistence),
        Err(MailDeliveryIntentExecutionErrorV1::InvalidJob) => {
            complete_rejected(
                runtime,
                store,
                claimed,
                MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeInvalidRequest,
                MailDeliveryIntentJobStateV1::Rejected,
                now_unix_seconds,
            )
            .await
        }
        Err(_) => retry_or_complete_unavailable(runtime, store, claimed, now_unix_seconds).await,
    }
}

async fn process_delivery_queued(
    runtime: &MailAdmittedRuntime,
    store: &MailDeliveryIntentStoreV1,
    claimed: &ClaimedMailDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentWorkerErrorV1> {
    let status = runtime
        .delivery_operation_status(&claimed.job.provider_operation_id)
        .await
        .map_err(|_| MailDeliveryIntentWorkerErrorV1::Runtime)?;
    let Some(status) = status else {
        return reschedule(store, claimed, now_unix_seconds).await;
    };
    match status.outcome {
        MailDeliveryOutcomeV1::Pending => reschedule(store, claimed, now_unix_seconds).await,
        MailDeliveryOutcomeV1::Accepted => {
            let context =
                result_context(runtime, status.completed_at_unix_seconds, now_unix_seconds);
            let result = build_mail_delivery_intent_succeeded_outbox_v1(
                &claimed.job,
                claimed.attempt_count.max(1),
                &context,
            )
            .map_err(|_| MailDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
            store
                .complete_claimed_job(
                    claimed.job.intent_id,
                    &claimed.worker_id,
                    MailDeliveryIntentJobStateV1::Succeeded,
                    &result,
                    context.completed_at_unix_seconds,
                )
                .await
                .map_err(|_| MailDeliveryIntentWorkerErrorV1::Persistence)
        }
        MailDeliveryOutcomeV1::Rejected => {
            complete_rejected(
                runtime,
                store,
                claimed,
                MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeProviderRejected,
                MailDeliveryIntentJobStateV1::Rejected,
                status.completed_at_unix_seconds.unwrap_or(now_unix_seconds),
            )
            .await
        }
        MailDeliveryOutcomeV1::OutcomeUnknown => {
            complete_rejected(
                runtime,
                store,
                claimed,
                MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeProviderAmbiguous,
                MailDeliveryIntentJobStateV1::OutcomeUnknown,
                status.completed_at_unix_seconds.unwrap_or(now_unix_seconds),
            )
            .await
        }
    }
}

async fn complete_rejected(
    runtime: &MailAdmittedRuntime,
    store: &MailDeliveryIntentStoreV1,
    claimed: &ClaimedMailDeliveryIntentJobV1,
    code: MailDeliveryIntentRejectCodeV1,
    terminal_state: MailDeliveryIntentJobStateV1,
    completed_at_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentWorkerErrorV1> {
    let context = result_context(
        runtime,
        Some(completed_at_unix_seconds),
        completed_at_unix_seconds,
    );
    let result = build_mail_delivery_intent_rejected_outbox_v1(
        &claimed.job,
        code,
        claimed.attempt_count.max(1),
        &context,
    )
    .map_err(|_| MailDeliveryIntentWorkerErrorV1::ResultEnvelope)?;
    store
        .complete_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            terminal_state,
            &result,
            completed_at_unix_seconds,
        )
        .await
        .map_err(|_| MailDeliveryIntentWorkerErrorV1::Persistence)
}

async fn reschedule(
    store: &MailDeliveryIntentStoreV1,
    claimed: &ClaimedMailDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentWorkerErrorV1> {
    let next_attempt_at = now_unix_seconds
        .checked_add(DELIVERY_INTENT_RETRY_SECONDS)
        .ok_or(MailDeliveryIntentWorkerErrorV1::InvalidClock)?;
    store
        .reschedule_claimed_job(
            claimed.job.intent_id,
            &claimed.worker_id,
            claimed.state,
            next_attempt_at,
        )
        .await
        .map_err(|_| MailDeliveryIntentWorkerErrorV1::Persistence)
}

async fn retry_or_complete_unavailable(
    runtime: &MailAdmittedRuntime,
    store: &MailDeliveryIntentStoreV1,
    claimed: &ClaimedMailDeliveryIntentJobV1,
    now_unix_seconds: i64,
) -> Result<(), MailDeliveryIntentWorkerErrorV1> {
    if claimed.attempt_count
        >= u32::try_from(MAIL_DELIVERY_INTENT_MAX_ATTEMPTS_V1)
            .expect("positive Mail delivery-intent attempt limit")
    {
        complete_rejected(
            runtime,
            store,
            claimed,
            MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeUnavailable,
            MailDeliveryIntentJobStateV1::Rejected,
            now_unix_seconds,
        )
        .await
    } else {
        reschedule(store, claimed, now_unix_seconds).await
    }
}

fn result_context(
    runtime: &MailAdmittedRuntime,
    completed_at_unix_seconds: Option<i64>,
    fallback_unix_seconds: i64,
) -> MailDeliveryIntentResultContextV1 {
    MailDeliveryIntentResultContextV1 {
        runtime_instance_id: runtime.runtime_instance_id.clone(),
        runtime_generation: runtime.runtime_generation,
        completed_at_unix_seconds: completed_at_unix_seconds.unwrap_or(fallback_unix_seconds),
        completed_at_nanos: 0,
    }
}

fn worker_id(runtime_instance_id: &str, runtime_generation: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.mail.delivery-intent.worker.v1");
    hasher.update(runtime_instance_id.as_bytes());
    hasher.update(runtime_generation.to_be_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    let suffix = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("mail-delivery-intent-{suffix}")
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
