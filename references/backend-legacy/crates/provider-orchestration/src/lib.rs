//! Provider-neutral application workflows.
//!
//! This crate turns provider contracts into domain-port calls. It intentionally
//! contains no provider client, storage implementation, vault implementation,
//! scheduler, or runtime process management.

use chrono::{DateTime, Utc};
use makosh_communications_api::commands::{
    CommunicationProviderCommand, ProviderCommandQueuePort, ProviderCommandQueuePortError,
};
use makosh_communications_api::evidence::{
    CommunicationEvidencePort, CommunicationEvidencePortError, NewRawCommunicationRecord,
    StoredRawCommunicationRecord,
};
use makosh_provider_api::{
    CredentialLease, ProviderCommandEnvelope, ProviderCommandInput, ProviderCommandResult,
    ProviderContractError, ProviderId, ProviderObservationEnvelope, ProviderRuntimePort,
    ProviderRuntimePortError,
};
use makosh_signal_hub_api::raw_signals::{ProviderRawSignalInput, ProviderRawSignalPort};
use serde_json::Value;
use serde_json::json;
use thiserror::Error;

/// Adapts a provider-neutral observation into Communications canonical raw evidence.
pub fn observation_to_raw_communication_record(
    observation: ProviderObservationEnvelope,
) -> NewRawCommunicationRecord {
    NewRawCommunicationRecord::new(
        observation.observation_id,
        observation.account_id,
        observation.record_kind,
        observation.provider_record_id,
        observation.source_fingerprint,
        observation.import_batch_id,
        observation.payload,
    )
    .occurred_at(observation.occurred_at)
    .provenance(observation.provenance)
}

/// Converts a persisted Communications command into the provider-neutral
/// envelope consumed by an in-process adapter or connector client.
pub fn communication_command_to_provider_envelope(
    command: &makosh_communications_api::commands::CommunicationProviderCommand,
    provider_id: ProviderId,
    now: DateTime<Utc>,
    deadline: DateTime<Utc>,
    lease_epoch: u64,
) -> Result<ProviderCommandEnvelope, ProviderContractError> {
    ProviderCommandInput::new(
        command.command_id.clone(),
        command.idempotency_key.clone(),
        provider_id,
        command.account_id.clone(),
        now,
        deadline,
        command.retry_count.max(0) as u32,
        lease_epoch,
        json!({
            "command_kind": command.command_kind,
            "provider_message_id": command.provider_message_id,
            "payload": command.payload,
        }),
    )
    .with_causation_id(command.command_id.clone())
    .with_correlation_id(command.account_id.clone())
    .try_into()
}

/// Executes one queued command through the semantic runtime port. Queue state
/// mutation remains owned by the caller so retries and fencing policy are
/// explicit at the application boundary.
pub async fn execute_provider_command(
    runtime: &dyn ProviderRuntimePort,
    command: &ProviderCommandEnvelope,
    credential: CredentialLease,
) -> Result<ProviderCommandResult, ProviderRuntimePortError> {
    runtime.execute(command, credential).await
}

/// Persists a provider observation through the Communications evidence boundary.
pub async fn record_provider_observation(
    evidence: &dyn CommunicationEvidencePort,
    observation: ProviderObservationEnvelope,
) -> Result<StoredRawCommunicationRecord, ProviderObservationOrchestrationError> {
    let record = observation_to_raw_communication_record(observation);
    evidence
        .record_raw_source(&record)
        .await
        .map_err(ProviderObservationOrchestrationError::Evidence)
}

/// Persists an observation and submits the resulting canonical record to Signal Hub.
/// The provider implementation never needs to construct Signal Hub payloads itself.
pub async fn record_and_dispatch_provider_observation(
    evidence: &dyn CommunicationEvidencePort,
    signal_hub: &dyn ProviderRawSignalPort,
    observation: ProviderObservationEnvelope,
) -> Result<bool, ProviderObservationOrchestrationError> {
    let lease_epoch = observation.lease_epoch;
    let record = record_provider_observation(evidence, observation).await?;
    signal_hub
        .dispatch_provider_record(&ProviderRawSignalInput {
            raw_record_id: record.raw_record_id.clone(),
            observation_id: record.observation_id.clone(),
            account_id: record.account_id.clone(),
            record_kind: record.record_kind.clone(),
            provider_record_id: record.provider_record_id.clone(),
            source_fingerprint: record.source_fingerprint.clone(),
            import_batch_id: record.import_batch_id.clone(),
            occurred_at: record.occurred_at,
            captured_at: record.captured_at,
            payload: record.payload.clone(),
            provenance: record.provenance.clone(),
            lease_epoch,
        })
        .await
        .map(|event| event.is_some())
        .map_err(|error| ProviderObservationOrchestrationError::SignalHub(error.to_string()))
}

#[derive(Debug, Error)]
pub enum ProviderObservationOrchestrationError {
    #[error("provider observation evidence persistence failed: {0}")]
    Evidence(CommunicationEvidencePortError),
    #[error("provider observation Signal Hub dispatch failed: {0}")]
    SignalHub(String),
}

pub async fn reconcile_provider_command_observation(
    command_queue: &dyn ProviderCommandQueuePort,
    account_id: &str,
    channel_kind: &str,
    provider_message_id: &str,
    command_kinds: &[&str],
    observed_at: DateTime<Utc>,
    provider_state: Value,
) -> Result<Vec<CommunicationProviderCommand>, ProviderCommandObservationReconciliationError> {
    command_queue
        .mark_observed_by_provider_message(
            account_id,
            channel_kind,
            provider_message_id,
            command_kinds,
            observed_at,
            provider_state,
        )
        .await
        .map_err(ProviderCommandObservationReconciliationError::CommandQueue)
}

#[derive(Debug, Error)]
pub enum ProviderCommandObservationReconciliationError {
    #[error("provider command observation reconciliation failed: {0}")]
    CommandQueue(ProviderCommandQueuePortError),
}
