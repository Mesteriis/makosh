//! Exact event-only adapter from integration call observations to Communications state.

use makosh_communications_call_evidence_core::{
    RecordCallEvidenceV1, decode_call_evidence_observation_v1,
};
use makosh_communications_call_evidence_ingress::{
    call_evidence_observed_contract_reference_v1, wire::CallEvidenceObservedV1,
};
use makosh_communications_call_evidence_persistence::{
    CallEvidenceConsumeOutcomeV1, CallEvidencePersistenceErrorV1,
    CommunicationsCallEvidencePersistenceV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

use crate::consumer::{CommunicationsDeliveryErrorV1, CommunicationsEventConsumeErrorV1};

pub async fn consume_next_call_evidence_observation_v1(
    persistence: &CommunicationsCallEvidencePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    logical_owner_id: &str,
    consumed_at_unix_seconds: i64,
) -> Result<CallEvidenceConsumeOutcomeV1, CommunicationsDeliveryErrorV1> {
    let expected = call_evidence_observed_contract_reference_v1();
    if !exact_contract(permit.contract(), &expected) {
        return Err(CommunicationsDeliveryErrorV1::Consume(
            CommunicationsEventConsumeErrorV1::WrongContract,
        ));
    }
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsDeliveryErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsDeliveryErrorV1::InvalidEnvelope)?;
    let decoded = decode_call_evidence_envelope_v1(&envelope, &expected)
        .map_err(CommunicationsDeliveryErrorV1::Consume)?;
    if decoded.evidence.logical_owner_id != logical_owner_id {
        return Err(CommunicationsDeliveryErrorV1::Consume(
            CommunicationsEventConsumeErrorV1::WrongContract,
        ));
    }
    let message_id = id16(&envelope.message_id).map_err(CommunicationsDeliveryErrorV1::Consume)?;
    let outcome = persistence
        .consume(
            logical_owner_id,
            message_id,
            *record.envelope_sha256(),
            decoded.evidence,
            decoded.observed_at_unix_seconds,
            consumed_at_unix_seconds,
        )
        .await
        .map_err(persistence_error)?;
    delivery.acknowledge().await.map_err(delivery_error)?;
    Ok(outcome)
}

struct DecodedCallEvidenceV1 {
    evidence: RecordCallEvidenceV1,
    observed_at_unix_seconds: i64,
}

fn decode_call_evidence_envelope_v1(
    envelope: &DurableEnvelopeV1,
    expected: &ContractReferenceV1,
) -> Result<DecodedCallEvidenceV1, CommunicationsEventConsumeErrorV1> {
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(CommunicationsEventConsumeErrorV1::WrongContract);
    };
    let source = envelope
        .source
        .as_ref()
        .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?;
    let source_fence = envelope
        .source_fence
        .as_ref()
        .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?;
    if !exact_envelope_contract(envelope, expected)
        || metadata.observation_id != envelope.message_id
        || source.module_id.is_empty()
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || source_fence.kind != FenceKindV1::RuntimeLease as i32
        || source_fence.scope_id != source.module_id.as_bytes()
        || source_fence.epoch != source.runtime_generation
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != source.module_id.as_bytes()
    {
        return Err(CommunicationsEventConsumeErrorV1::WrongContract);
    }
    let payload = CallEvidenceObservedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)?;
    if metadata.source_cursor_sha256 != payload.source_call_cursor_sha256
        || metadata.source_sequence != Some(payload.source_revision)
        || envelope.partition_key != payload.call_evidence_id
        || envelope.correlation_id != payload.call_evidence_id
    {
        return Err(CommunicationsEventConsumeErrorV1::WrongContract);
    }
    let observed_at = metadata
        .occurred_at
        .as_ref()
        .ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?;
    if observed_at.nanos != 0 || !(-62_135_596_800..=253_402_300_799).contains(&observed_at.seconds)
    {
        return Err(CommunicationsEventConsumeErrorV1::InvalidPayload);
    }
    let evidence = decode_call_evidence_observation_v1(&payload, &envelope.payload)
        .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    Ok(DecodedCallEvidenceV1 {
        evidence,
        observed_at_unix_seconds: observed_at.seconds,
    })
}

fn exact_contract(left: Option<&ContractReferenceV1>, right: &ContractReferenceV1) -> bool {
    left.is_some_and(|left| {
        left.owner == right.owner
            && left.name == right.name
            && left.major == right.major
            && left.revision == right.revision
            && left.schema_sha256 == right.schema_sha256
    })
}

fn exact_envelope_contract(envelope: &DurableEnvelopeV1, expected: &ContractReferenceV1) -> bool {
    envelope.contract.as_ref().is_some_and(|contract| {
        contract.owner == expected.owner
            && contract.name == expected.name
            && contract.major == expected.major
            && contract.revision == expected.revision
            && contract.schema_sha256 == expected.schema_sha256
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsEventConsumeErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsEventConsumeErrorV1::WrongContract)
}

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsDeliveryErrorV1 {
    CommunicationsDeliveryErrorV1::Unavailable
}

fn persistence_error(error: CallEvidencePersistenceErrorV1) -> CommunicationsDeliveryErrorV1 {
    match error {
        CallEvidencePersistenceErrorV1::InvalidInput
        | CallEvidencePersistenceErrorV1::InvalidRow
        | CallEvidencePersistenceErrorV1::InboxHashConflict => {
            CommunicationsDeliveryErrorV1::Consume(
                CommunicationsEventConsumeErrorV1::PersistenceRejected,
            )
        }
        CallEvidencePersistenceErrorV1::StorageUnavailable => {
            CommunicationsDeliveryErrorV1::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use makosh_communications_call_evidence_ingress::{
        CallDirectionV1, CallEvidenceEnvelopeContextV1, CallEvidenceObservationDraftV1,
        CallLifecycleStateV1, CallMediaKindV1, CallProviderProvenanceV1,
        build_call_evidence_observed_outbox_record_v1,
    };

    use super::*;

    fn record() -> OutboxRecordV1 {
        build_call_evidence_observed_outbox_record_v1(
            &CallEvidenceObservationDraftV1 {
                observation_id: "telegram-call-1-revision-1".to_owned(),
                logical_owner_id: "owner-1".to_owned(),
                provider: CallProviderProvenanceV1::Telegram,
                external_account_id: "account-1".to_owned(),
                external_call_id: "call-1".to_owned(),
                external_conversation_id: Some("conversation-1".to_owned()),
                external_participant_id: Some("participant-1".to_owned()),
                direction: CallDirectionV1::Incoming,
                media_kind: CallMediaKindV1::OneToOneAudio,
                state: CallLifecycleStateV1::Ringing,
                terminal_disposition: None,
                source_revision: 1,
                observed_at_unix_seconds: 1_700_000_000,
                started_at_unix_seconds: Some(1_700_000_000),
                connected_at_unix_seconds: None,
                ended_at_unix_seconds: None,
                duration_seconds: None,
                participant_display_label: Some("Alice".to_owned()),
            },
            &CallEvidenceEnvelopeContextV1 {
                module_id: "telegram".to_owned(),
                runtime_instance_id: "telegram-runtime-1".to_owned(),
                runtime_generation: 7,
                recorded_at_unix_seconds: 1_700_000_001,
                recorded_at_nanos: 0,
            },
        )
        .expect("record")
    }

    #[test]
    fn exact_event_decodes_without_provider_locator_leakage() {
        let record = record();
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let decoded = decode_call_evidence_envelope_v1(
            &envelope,
            &call_evidence_observed_contract_reference_v1(),
        )
        .expect("decoded");
        assert_eq!(decoded.evidence.source_revision, 1);
        assert_eq!(decoded.observed_at_unix_seconds, 1_700_000_000);
        assert!(!envelope.payload.windows(6).any(|bytes| bytes == b"call-1"));
    }

    #[test]
    fn mismatched_source_sequence_fails_closed() {
        let record = record();
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Observation(metadata)) = envelope.semantics.as_mut() else {
            panic!("observation");
        };
        metadata.source_sequence = Some(2);
        assert_eq!(
            decode_call_evidence_envelope_v1(
                &envelope,
                &call_evidence_observed_contract_reference_v1(),
            )
            .map(|_| ()),
            Err(CommunicationsEventConsumeErrorV1::WrongContract)
        );
    }
}
