use std::time::{SystemTime, UNIX_EPOCH};

use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_scheduler_protocol::{
    OwnerJobLeaseV1, SCHEDULER_JOB_DESCRIPTOR_SET_V1, build_owner_job_command_v1,
    v1::OwnerJobTriggerKindV1,
};
use makosh_telegram_calls_core::{
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1,
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1,
    TELEGRAM_CALLS_REALTIME_BACKFILL_MAX_BATCHES_PER_BOOT_V1,
    TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1, telegram_calls_realtime_backfill_idempotency_key_v1,
    telegram_calls_realtime_backfill_job_kind_v1, telegram_calls_realtime_backfill_lease_expiry_v1,
    telegram_calls_realtime_backfill_message_id_v1, telegram_calls_realtime_backfill_run_id_v1,
    telegram_calls_realtime_backfill_scope_v1,
};
use makosh_telegram_calls_persistence::{
    TelegramCallsBackfillErrorV1, TelegramCallsBackfillStateV1, TelegramCallsPersistence,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::managed_control::TelegramManagedRuntimeIdentity;

const TELEGRAM_RUNTIME_MODULE_ID: &str = "makosh-telegram-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TelegramCallsBackfillRuntimeErrorV1 {
    Clock,
    InvalidEnvelope,
    Persistence(TelegramCallsBackfillErrorV1),
    ExecutionPolicyExhausted,
}

pub(crate) async fn complete_calls_realtime_backfill_v1(
    persistence: &TelegramCallsPersistence,
    identity: &TelegramManagedRuntimeIdentity,
) -> Result<(), TelegramCallsBackfillRuntimeErrorV1> {
    if persistence
        .calls_realtime_backfill_execution_v1()
        .await
        .map_err(TelegramCallsBackfillRuntimeErrorV1::Persistence)?
        .is_none()
    {
        let accepted_at = current_unix_millis()?;
        let envelope = build_calls_realtime_backfill_envelope_v1(identity, accepted_at)?;
        persistence
            .accept_calls_realtime_backfill_v1(&envelope)
            .await
            .map_err(TelegramCallsBackfillRuntimeErrorV1::Persistence)?;
    }
    let mut execution = persistence
        .claim_calls_realtime_backfill_v1(identity.runtime_generation(), current_unix_millis()?)
        .await
        .map_err(TelegramCallsBackfillRuntimeErrorV1::Persistence)?;
    if execution.state == TelegramCallsBackfillStateV1::Succeeded {
        return Ok(());
    }
    for _ in 0..TELEGRAM_CALLS_REALTIME_BACKFILL_MAX_BATCHES_PER_BOOT_V1 {
        let batch = persistence
            .execute_calls_realtime_backfill_batch_v1(
                identity.runtime_generation(),
                execution.lease_epoch,
                current_unix_millis()?,
            )
            .await
            .map_err(TelegramCallsBackfillRuntimeErrorV1::Persistence)?;
        execution = batch.execution;
        if execution.state == TelegramCallsBackfillStateV1::Succeeded {
            return Ok(());
        }
    }
    Err(TelegramCallsBackfillRuntimeErrorV1::ExecutionPolicyExhausted)
}

pub(crate) fn build_calls_realtime_backfill_envelope_v1(
    identity: &TelegramManagedRuntimeIdentity,
    accepted_at_unix_millis: i64,
) -> Result<Vec<u8>, TelegramCallsBackfillRuntimeErrorV1> {
    let lease_expires_at =
        telegram_calls_realtime_backfill_lease_expiry_v1(accepted_at_unix_millis)
            .ok_or(TelegramCallsBackfillRuntimeErrorV1::InvalidEnvelope)?;
    let run_id = telegram_calls_realtime_backfill_run_id_v1();
    let lease = OwnerJobLeaseV1::new(run_id, 1, UtcMillisV1::new(lease_expires_at))
        .map_err(|_| TelegramCallsBackfillRuntimeErrorV1::InvalidEnvelope)?;
    let payload = build_owner_job_command_v1(
        &telegram_calls_realtime_backfill_job_kind_v1(),
        &telegram_calls_realtime_backfill_scope_v1(),
        OwnerJobTriggerKindV1::UpgradeReconciliation,
        UtcMillisV1::new(accepted_at_unix_millis),
        lease,
    )
    .map_err(|_| TelegramCallsBackfillRuntimeErrorV1::InvalidEnvelope)?
    .encode_to_vec();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: telegram_calls_realtime_backfill_message_id_v1().to_vec(),
        contract: Some(ContractRefV1 {
            owner: TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1.to_owned(),
            name: TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1.to_owned(),
            major: u32::from(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1),
            revision: 1,
            schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: TELEGRAM_RUNTIME_MODULE_ID.to_owned(),
            runtime_instance_id: runtime_source_reference(identity.runtime_instance_id()).to_vec(),
            runtime_generation: identity.runtime_generation(),
        }),
        recorded_at: Some(timestamp(accepted_at_unix_millis)?),
        partition_key: TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1
            .as_bytes()
            .to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: run_id.bytes().to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System as i32,
            actor_id: TELEGRAM_RUNTIME_MODULE_ID.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: TELEGRAM_RUNTIME_MODULE_ID.as_bytes().to_vec(),
            epoch: identity.runtime_generation(),
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: run_id.bytes().to_vec(),
            target_capability: "job_execute".to_owned(),
            idempotency_key: telegram_calls_realtime_backfill_idempotency_key_v1().to_vec(),
            deadline: Some(timestamp(lease_expires_at)?),
            logical_attempt: 1,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| TelegramCallsBackfillRuntimeErrorV1::InvalidEnvelope)?;
    Ok(envelope.encode_to_vec())
}

fn timestamp(unix_millis: i64) -> Result<Timestamp, TelegramCallsBackfillRuntimeErrorV1> {
    if unix_millis <= 0 {
        return Err(TelegramCallsBackfillRuntimeErrorV1::InvalidEnvelope);
    }
    Ok(Timestamp {
        seconds: unix_millis.div_euclid(1_000),
        nanos: i32::try_from(unix_millis.rem_euclid(1_000) * 1_000_000)
            .map_err(|_| TelegramCallsBackfillRuntimeErrorV1::InvalidEnvelope)?,
    })
}

fn current_unix_millis() -> Result<i64, TelegramCallsBackfillRuntimeErrorV1> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TelegramCallsBackfillRuntimeErrorV1::Clock)?
        .as_millis();
    i64::try_from(millis).map_err(|_| TelegramCallsBackfillRuntimeErrorV1::Clock)
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.runtime.source-reference.v1\0");
    hasher.update(runtime_instance_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}
