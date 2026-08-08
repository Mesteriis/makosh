use std::os::unix::net::UnixStream;

use makosh_communication_task_candidate_core::{
    CommunicationTaskCandidateRejectionCodeV1, CommunicationTaskCandidateStateV1,
    CommunicationTaskCandidateTransitionV1,
};
use makosh_communication_task_candidate_persistence::{
    CommunicationTaskCandidateInboxResultV1, CommunicationTaskCandidatePersistenceErrorV1,
    CommunicationTaskCandidatePersistenceV1, CommunicationTaskCandidateSourceResultV1,
    PersistedCommunicationTaskCandidateRunV1,
};
use makosh_communications_task_source_api::{
    communication_task_source_prepared_contract_reference_v1,
    communication_task_source_rejected_contract_reference_v1,
    wire::{
        CommunicationTaskSourceContentReceiptV1, CommunicationTaskSourcePreparedV1,
        CommunicationTaskSourceRejectedV1,
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
    CommunicationTaskCandidateBlobErrorV1,
    blob_materialization::{
        CommunicationTaskCandidateSourceBlobReceiptV1, materialize_task_source_v1,
        read_task_source_v1, release_task_source_v1,
    },
    extraction::{
        CommunicationTaskCandidateExtractionErrorV1,
        complete_communication_task_candidate_extraction_v1,
    },
    review_submission::CommunicationTaskCandidateReviewSubmissionContextV1,
};

const COMMUNICATIONS_RUNTIME_MODULE_ID_V1: &str = "makosh-communications-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateSourceResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(CommunicationTaskCandidateBlobErrorV1),
    Extraction(CommunicationTaskCandidateExtractionErrorV1),
    Persistence(CommunicationTaskCandidatePersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct CommunicationTaskCandidateSourcePreparedContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub module_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub consumed_at_unix_millis: i64,
}

impl<'a> CommunicationTaskCandidateSourcePreparedContextV1<'a> {
    fn review_submission_context(
        &'a self,
    ) -> CommunicationTaskCandidateReviewSubmissionContextV1<'a> {
        CommunicationTaskCandidateReviewSubmissionContextV1 {
            module_id: self.module_id,
            runtime_instance_id: self.runtime_instance_id,
            runtime_generation: self.runtime_generation,
            now_unix_millis: self.consumed_at_unix_millis,
        }
    }
}

pub(crate) async fn consume_task_source_prepared_once_v1(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    context: &CommunicationTaskCandidateSourcePreparedContextV1<'_>,
) -> Result<bool, CommunicationTaskCandidateSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidEnvelope)?;
    let prepared = decode_prepared(&record, context.logical_owner_id)?;
    let run = persistence
        .load_run(context.logical_owner_id, &prepared.run_id)
        .await
        .map_err(CommunicationTaskCandidateSourceResultErrorV1::Persistence)?;
    if run.draft.source_message_id != prepared.source_message_id
        || run.draft.expected_source_revision != prepared.expected_source_revision
    {
        return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload);
    }
    let terminal = matches!(
        run.status.state,
        CommunicationTaskCandidateStateV1::Ready | CommunicationTaskCandidateStateV1::Rejected
    );
    if terminal {
        if run.cleanup_completed_at_unix_millis.is_none() {
            let cleanup = run
                .source_cleanup
                .as_ref()
                .ok_or(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
            release_task_source_v1(
                channel,
                dispatcher,
                run.draft.run_id,
                cleanup,
                run.status.state == CommunicationTaskCandidateStateV1::Ready,
            )
            .map_err(CommunicationTaskCandidateSourceResultErrorV1::Blob)?;
            persistence
                .complete_blob_cleanup(
                    context.logical_owner_id,
                    &run.draft.run_id,
                    cleanup,
                    context.consumed_at_unix_millis,
                )
                .await
                .map_err(CommunicationTaskCandidateSourceResultErrorV1::Persistence)?;
        }
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }

    let run = if run.status.state == CommunicationTaskCandidateStateV1::PreparingSource {
        let source = prepared
            .source_content
            .as_ref()
            .ok_or(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
        let materialized =
            materialize_task_source_v1(channel, dispatcher, &source_blob_receipt(&record, source)?)
                .map_err(CommunicationTaskCandidateSourceResultErrorV1::Blob)?;
        let persisted = persistence
            .persist_source_result(CommunicationTaskCandidateSourceResultV1 {
                result_message_id: *record.message_id(),
                envelope_sha256: *record.envelope_sha256(),
                logical_owner_id: context.logical_owner_id.to_owned(),
                run_id: prepared.run_id,
                transition: CommunicationTaskCandidateTransitionV1::SourcePrepared {
                    source_evidence_id: prepared.source_evidence_id,
                    source_evidence_revision: prepared.source_evidence_revision,
                    source_sha256: sha256(&source.sha256)?,
                },
                source_read_receipt_bytes: Some(source.encode_to_vec()),
                source_cleanup: Some(materialized.source_cleanup.clone()),
                occurred_at_unix_millis: context.consumed_at_unix_millis,
            })
            .await
            .map_err(CommunicationTaskCandidateSourceResultErrorV1::Persistence)?;
        let persisted = match persisted {
            CommunicationTaskCandidateInboxResultV1::Applied(value)
            | CommunicationTaskCandidateInboxResultV1::Duplicate(value) => value,
        };
        finish_extraction(
            persistence,
            channel,
            dispatcher,
            &persisted,
            materialized.body_utf8.as_slice(),
            &context.review_submission_context(),
            context.consumed_at_unix_millis,
        )
        .await?
    } else if run.status.state == CommunicationTaskCandidateStateV1::Extracting {
        let cleanup = run
            .source_cleanup
            .as_ref()
            .ok_or(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
        let body = read_task_source_v1(channel, dispatcher, cleanup)
            .map_err(CommunicationTaskCandidateSourceResultErrorV1::Blob)?;
        finish_extraction(
            persistence,
            channel,
            dispatcher,
            &run,
            body.as_slice(),
            &context.review_submission_context(),
            context.consumed_at_unix_millis,
        )
        .await?
    } else {
        run
    };
    if !matches!(
        run.status.state,
        CommunicationTaskCandidateStateV1::Ready | CommunicationTaskCandidateStateV1::Rejected
    ) {
        return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload);
    }
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

async fn finish_extraction(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCommunicationTaskCandidateRunV1,
    body_utf8: &[u8],
    submission_context: &CommunicationTaskCandidateReviewSubmissionContextV1<'_>,
    occurred_at_unix_millis: i64,
) -> Result<PersistedCommunicationTaskCandidateRunV1, CommunicationTaskCandidateSourceResultErrorV1>
{
    let cleanup = run
        .source_cleanup
        .as_ref()
        .ok_or(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
    let terminal = complete_communication_task_candidate_extraction_v1(
        persistence,
        channel,
        dispatcher,
        run,
        body_utf8,
        submission_context,
        occurred_at_unix_millis,
    )
    .await
    .map_err(CommunicationTaskCandidateSourceResultErrorV1::Extraction)?;
    release_task_source_v1(
        channel,
        dispatcher,
        run.draft.run_id,
        cleanup,
        terminal.status.state == CommunicationTaskCandidateStateV1::Ready,
    )
    .map_err(CommunicationTaskCandidateSourceResultErrorV1::Blob)?;
    persistence
        .complete_blob_cleanup(
            &run.logical_owner_id,
            &run.draft.run_id,
            cleanup,
            occurred_at_unix_millis,
        )
        .await
        .map_err(CommunicationTaskCandidateSourceResultErrorV1::Persistence)
}

pub async fn consume_task_source_rejected_once_v1(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CommunicationTaskCandidateSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidEnvelope)?;
    let rejected = decode_rejected(&record, expected_logical_owner_id)?;
    persistence
        .persist_source_result(CommunicationTaskCandidateSourceResultV1 {
            result_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            logical_owner_id: expected_logical_owner_id.to_owned(),
            run_id: rejected.run_id,
            transition: CommunicationTaskCandidateTransitionV1::Reject(rejected.rejection),
            source_read_receipt_bytes: None,
            source_cleanup: None,
            occurred_at_unix_millis: consumed_at_unix_millis,
        })
        .await
        .map_err(CommunicationTaskCandidateSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct PreparedSourceV1 {
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
    source_content: Option<CommunicationTaskSourceContentReceiptV1>,
}

struct RejectedSourceV1 {
    run_id: [u8; 16],
    rejection: CommunicationTaskCandidateRejectionCodeV1,
}

fn decode_prepared(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<PreparedSourceV1, CommunicationTaskCandidateSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_task_source_prepared_contract_reference_v1(),
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
    let payload = CommunicationTaskSourcePreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload);
    }
    Ok(PreparedSourceV1 {
        run_id,
        source_message_id: id16(&payload.source_message_id)?,
        expected_source_revision: positive(payload.expected_source_revision)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        source_evidence_revision: positive(payload.source_evidence_revision)?,
        source_content: payload.source_content,
    })
}

fn decode_rejected(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<RejectedSourceV1, CommunicationTaskCandidateSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidEnvelope)?;
    let run_id = validate_result_envelope(
        &envelope.contract,
        &communication_task_source_rejected_contract_reference_v1(),
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
    let payload = CommunicationTaskSourceRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id || payload.run_id.as_slice() != run_id
    {
        return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload);
    }
    let rejection = match payload.code {
        1 => CommunicationTaskCandidateRejectionCodeV1::InvalidRequest,
        6 => CommunicationTaskCandidateRejectionCodeV1::Policy,
        2..=5 => CommunicationTaskCandidateRejectionCodeV1::SourceRejected,
        _ => return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload),
    };
    Ok(RejectedSourceV1 { run_id, rejection })
}

fn source_blob_receipt(
    record: &OutboxRecordV1,
    source: &CommunicationTaskSourceContentReceiptV1,
) -> Result<
    CommunicationTaskCandidateSourceBlobReceiptV1,
    CommunicationTaskCandidateSourceResultErrorV1,
> {
    Ok(CommunicationTaskCandidateSourceBlobReceiptV1 {
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
) -> Result<[u8; 16], CommunicationTaskCandidateSourceResultErrorV1> {
    if !exact_contract(actual_contract.as_ref(), expected_contract)
        || source_module_id != Some(COMMUNICATIONS_RUNTIME_MODULE_ID_V1)
        || source_runtime_generation == 0
    {
        return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Result(result)) = semantics else {
        return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidEnvelope);
    };
    if result.command_id.len() != 16
        || result.command_message_id.as_slice() != result.command_id
        || result.outcome != expected_outcome as i32
        || result.execution_attempt == 0
    {
        return Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidEnvelope);
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

fn positive(value: u64) -> Result<u64, CommunicationTaskCandidateSourceResultErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationTaskCandidateSourceResultErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], CommunicationTaskCandidateSourceResultErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> CommunicationTaskCandidateSourceResultErrorV1 {
    CommunicationTaskCandidateSourceResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_rejection_mapping_is_bounded() {
        assert_eq!(positive(1), Ok(1));
        assert_eq!(
            positive(0),
            Err(CommunicationTaskCandidateSourceResultErrorV1::InvalidPayload)
        );
    }
}
