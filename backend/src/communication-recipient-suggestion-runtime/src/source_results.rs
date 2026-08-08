use std::os::unix::net::UnixStream;

use makosh_communication_recipient_suggestion_core::{
    CommunicationRecipientSuggestionRejectionCodeV1, CommunicationRecipientSuggestionStateV1,
    CommunicationRecipientSuggestionTransitionV1,
};
use makosh_communication_recipient_suggestion_persistence::{
    CommunicationRecipientSuggestionInboxResultV1,
    CommunicationRecipientSuggestionPersistenceErrorV1,
    CommunicationRecipientSuggestionPersistenceV1, CommunicationRecipientSuggestionSourceResultV1,
    PersistedCommunicationRecipientSuggestionRunV1,
};
use makosh_communications_recipient_source_api::{
    communication_recipient_source_prepared_contract_reference_v1,
    communication_recipient_source_rejected_contract_reference_v1,
    wire::{
        CommunicationRecipientBodySourceReceiptV1, CommunicationRecipientSourcePreparedV1,
        CommunicationRecipientSourceRejectedV1,
    },
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::ContractReferenceV1,
};
use prost::Message;

use crate::{
    CommunicationRecipientSuggestionBlobErrorV1, CommunicationRecipientSuggestionEvaluationErrorV1,
    blob_materialization::{
        CommunicationRecipientSuggestionSourceBlobReceiptV1, materialize_recipient_source_v1,
        read_recipient_source_v1, release_recipient_source_v1,
    },
    complete_communication_recipient_suggestion_evaluation_v1,
};

const COMMUNICATIONS_RUNTIME_MODULE_ID_V1: &str = "makosh-communications-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionSourceResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(CommunicationRecipientSuggestionBlobErrorV1),
    Evaluation(CommunicationRecipientSuggestionEvaluationErrorV1),
    Persistence(CommunicationRecipientSuggestionPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_recipient_source_prepared_once_v1(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CommunicationRecipientSuggestionSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let prepared = decode_prepared(&record, expected_logical_owner_id)?;
    let run = persistence
        .load_run(expected_logical_owner_id, &prepared.run_id)
        .await
        .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Persistence)?;
    if run.draft.source_message_id != prepared.source_message_id
        || run.draft.expected_source_revision != prepared.expected_source_revision
    {
        return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload);
    }
    let terminal = matches!(
        run.status.state,
        CommunicationRecipientSuggestionStateV1::Ready
            | CommunicationRecipientSuggestionStateV1::Rejected
    );
    if terminal {
        if run.cleanup_completed_at_unix_millis.is_none() {
            let cleanup = run
                .source_cleanup
                .as_ref()
                .ok_or(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
            release_recipient_source_v1(
                channel,
                dispatcher,
                run.draft.run_id,
                cleanup,
                run.status.state == CommunicationRecipientSuggestionStateV1::Ready,
            )
            .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Blob)?;
            persistence
                .complete_blob_cleanup(
                    expected_logical_owner_id,
                    &run.draft.run_id,
                    cleanup,
                    consumed_at_unix_millis,
                )
                .await
                .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Persistence)?;
        }
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }

    let run = if run.status.state == CommunicationRecipientSuggestionStateV1::PreparingSource {
        let source = prepared
            .body_source
            .as_ref()
            .ok_or(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
        let materialized = materialize_recipient_source_v1(
            channel,
            dispatcher,
            &source_blob_receipt(&record, source)?,
        )
        .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Blob)?;
        let persisted = persistence
            .persist_source_result(CommunicationRecipientSuggestionSourceResultV1 {
                result_message_id: *record.message_id(),
                envelope_sha256: *record.envelope_sha256(),
                logical_owner_id: expected_logical_owner_id.to_owned(),
                run_id: prepared.run_id,
                transition: CommunicationRecipientSuggestionTransitionV1::SourcePrepared {
                    source_evidence_id: prepared.source_evidence_id,
                    source_evidence_revision: prepared.source_evidence_revision,
                    source_sha256: sha256(&source.sha256)?,
                },
                evaluation_receipt_bytes: Some(source.encode_to_vec()),
                source_cleanup: Some(materialized.source_cleanup.clone()),
                occurred_at_unix_millis: consumed_at_unix_millis,
            })
            .await
            .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Persistence)?;
        let persisted = match persisted {
            CommunicationRecipientSuggestionInboxResultV1::Applied(value)
            | CommunicationRecipientSuggestionInboxResultV1::Duplicate(value) => value,
        };
        finish_evaluation(
            persistence,
            channel,
            dispatcher,
            &persisted,
            materialized.body_utf8.as_slice(),
            consumed_at_unix_millis,
        )
        .await?
    } else if run.status.state == CommunicationRecipientSuggestionStateV1::Evaluating {
        let cleanup = run
            .source_cleanup
            .as_ref()
            .ok_or(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
        let body = read_recipient_source_v1(channel, dispatcher, cleanup)
            .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Blob)?;
        finish_evaluation(
            persistence,
            channel,
            dispatcher,
            &run,
            body.as_slice(),
            consumed_at_unix_millis,
        )
        .await?
    } else {
        run
    };
    if !matches!(
        run.status.state,
        CommunicationRecipientSuggestionStateV1::Ready
            | CommunicationRecipientSuggestionStateV1::Rejected
    ) {
        return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload);
    }
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

async fn finish_evaluation(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCommunicationRecipientSuggestionRunV1,
    body_utf8: &[u8],
    occurred_at_unix_millis: i64,
) -> Result<
    PersistedCommunicationRecipientSuggestionRunV1,
    CommunicationRecipientSuggestionSourceResultErrorV1,
> {
    let cleanup = run
        .source_cleanup
        .as_ref()
        .ok_or(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
    let terminal = complete_communication_recipient_suggestion_evaluation_v1(
        persistence,
        run,
        body_utf8,
        occurred_at_unix_millis,
    )
    .await
    .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Evaluation)?;
    release_recipient_source_v1(
        channel,
        dispatcher,
        run.draft.run_id,
        cleanup,
        terminal.status.state == CommunicationRecipientSuggestionStateV1::Ready,
    )
    .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Blob)?;
    persistence
        .complete_blob_cleanup(
            &run.logical_owner_id,
            &run.draft.run_id,
            cleanup,
            occurred_at_unix_millis,
        )
        .await
        .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Persistence)
}

pub async fn consume_recipient_source_rejected_once_v1(
    persistence: &CommunicationRecipientSuggestionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CommunicationRecipientSuggestionSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let rejected = decode_rejected(&record, expected_logical_owner_id)?;
    persistence
        .persist_source_result(CommunicationRecipientSuggestionSourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: rejected.run_id,
            transition: CommunicationRecipientSuggestionTransitionV1::Reject(rejected.rejection),
            evaluation_receipt_bytes: None,
            source_cleanup: None,
            occurred_at_unix_millis: consumed_at_unix_millis,
        })
        .await
        .map_err(CommunicationRecipientSuggestionSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct PreparedSourceV1 {
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
    body_source: Option<CommunicationRecipientBodySourceReceiptV1>,
}

struct RejectedSourceV1 {
    run_id: [u8; 16],
    rejection: CommunicationRecipientSuggestionRejectionCodeV1,
}

fn decode_prepared(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<PreparedSourceV1, CommunicationRecipientSuggestionSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_recipient_source_prepared_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = CommunicationRecipientSourcePreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload);
    }
    Ok(PreparedSourceV1 {
        run_id,
        source_message_id: id16(&payload.source_message_id)?,
        expected_source_revision: positive(payload.expected_source_revision)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        source_evidence_revision: positive(payload.source_evidence_revision)?,
        body_source: payload.body_source,
    })
}

fn decode_rejected(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<RejectedSourceV1, CommunicationRecipientSuggestionSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_recipient_source_rejected_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Rejected,
    )?;
    let payload = CommunicationRecipientSourceRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload);
    }
    let rejection = match payload.code {
        1 => CommunicationRecipientSuggestionRejectionCodeV1::InvalidRequest,
        6 => CommunicationRecipientSuggestionRejectionCodeV1::Policy,
        2..=5 => CommunicationRecipientSuggestionRejectionCodeV1::SourceRejected,
        _ => return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload),
    };
    Ok(RejectedSourceV1 { run_id, rejection })
}

fn source_blob_receipt(
    record: &OutboxRecordV1,
    source: &CommunicationRecipientBodySourceReceiptV1,
) -> Result<
    CommunicationRecipientSuggestionSourceBlobReceiptV1,
    CommunicationRecipientSuggestionSourceResultErrorV1,
> {
    Ok(CommunicationRecipientSuggestionSourceBlobReceiptV1 {
        result_message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        reference_id: id16(&source.reference_id)?,
        declared_bytes: source.declared_bytes,
        sha256: sha256(&source.sha256)?,
        custody_proof: source.custody_transfer_source_proof.clone(),
    })
}

fn validate_result_envelope(
    actual_contract: &Option<ContractRefV1>,
    expected_contract: &ContractReferenceV1,
    source_module_id: Option<&str>,
    source_runtime_generation: u64,
    semantics: Option<&Semantics>,
    expected_outcome: ResultOutcomeV1,
) -> Result<[u8; 16], CommunicationRecipientSuggestionSourceResultErrorV1> {
    if !exact_contract(actual_contract.as_ref(), expected_contract)
        || source_module_id != Some(COMMUNICATIONS_RUNTIME_MODULE_ID_V1)
        || source_runtime_generation == 0
    {
        return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Result(result)) = semantics else {
        return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidEnvelope);
    };
    if result.command_id.len() != 16
        || result.command_message_id.as_slice() != result.command_id
        || result.outcome != expected_outcome as i32
        || result.execution_attempt == 0
    {
        return Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidEnvelope);
    }
    id16(&result.command_id)
}

fn exact_contract(actual: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner
            && actual.name == expected.name
            && actual.major == expected.major
            && actual.revision == expected.revision
            && actual.schema_sha256 == expected.schema_sha256
    })
}

fn positive(value: u64) -> Result<u64, CommunicationRecipientSuggestionSourceResultErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationRecipientSuggestionSourceResultErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], CommunicationRecipientSuggestionSourceResultErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)
}

fn event_error(
    _: RuntimePullDeliveryErrorV1,
) -> CommunicationRecipientSuggestionSourceResultErrorV1 {
    CommunicationRecipientSuggestionSourceResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_rejection_mapping_is_bounded() {
        assert_eq!(positive(1), Ok(1));
        assert_eq!(
            positive(0),
            Err(CommunicationRecipientSuggestionSourceResultErrorV1::InvalidPayload)
        );
    }
}
