//! Exact durable result consumers for the Communications-owned source port.

use makosh_communications_evidence_export_source_api::{
    evidence_export_prepared_contract_reference_v1, evidence_export_rejected_contract_reference_v1,
    wire::{
        EvidenceExportBodyStateV1, EvidenceExportDirectionV1, EvidenceExportPreparedV1,
        EvidenceExportRejectedV1,
    },
};
use makosh_communications_export_core::EvidenceExportDirectionV1 as CoreDirectionV1;
use makosh_communications_export_persistence::{
    CommunicationsExportPersistenceErrorV1, CommunicationsExportPersistenceV1,
    CommunicationsExportPreparedItemV1, CommunicationsExportSourceReceiptV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExportEventConsumerErrorV1 {
    Unavailable,
    InvalidEnvelope,
    InvalidPayload,
    Persistence,
}

pub async fn consume_next_prepared_result_v1(
    persistence: &CommunicationsExportPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    consumed_at_unix_seconds: i64,
) -> Result<(), CommunicationsExportEventConsumerErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsExportEventConsumerErrorV1::InvalidEnvelope)?;
    let envelope =
        exact_result_envelope(&record, &evidence_export_prepared_contract_reference_v1())?;
    let payload = EvidenceExportPreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsExportEventConsumerErrorV1::InvalidPayload)?;
    if !valid_logical_owner_id(&payload.logical_owner_id) {
        return Err(CommunicationsExportEventConsumerErrorV1::InvalidPayload);
    }
    let export_id = result_export_id(&envelope, &payload.export_id)?;
    let items = payload
        .items
        .into_iter()
        .map(prepared_item)
        .collect::<Result<Vec<_>, _>>()?;
    persistence
        .record_prepared_result(
            *record.message_id(),
            *record.envelope_sha256(),
            export_id,
            &payload.logical_owner_id,
            &items,
            consumed_at_unix_seconds,
        )
        .await
        .map_err(persistence_error)?;
    delivery.acknowledge().await.map_err(delivery_error)
}

pub async fn consume_next_rejected_result_v1(
    persistence: &CommunicationsExportPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    consumed_at_unix_seconds: i64,
) -> Result<(), CommunicationsExportEventConsumerErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsExportEventConsumerErrorV1::InvalidEnvelope)?;
    let envelope =
        exact_result_envelope(&record, &evidence_export_rejected_contract_reference_v1())?;
    let payload = EvidenceExportRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsExportEventConsumerErrorV1::InvalidPayload)?;
    if !valid_logical_owner_id(&payload.logical_owner_id) {
        return Err(CommunicationsExportEventConsumerErrorV1::InvalidPayload);
    }
    let export_id = result_export_id(&envelope, &payload.export_id)?;
    let rejection_code = u16::try_from(payload.code)
        .ok()
        .filter(|code| (1..=5).contains(code))
        .ok_or(CommunicationsExportEventConsumerErrorV1::InvalidPayload)?;
    persistence
        .record_rejected_result(
            *record.message_id(),
            *record.envelope_sha256(),
            export_id,
            &payload.logical_owner_id,
            rejection_code,
            consumed_at_unix_seconds,
        )
        .await
        .map_err(persistence_error)?;
    delivery.acknowledge().await.map_err(delivery_error)
}

fn exact_result_envelope(
    record: &OutboxRecordV1,
    expected: &ContractReferenceV1,
) -> Result<makosh_events_protocol::v1::DurableEnvelopeV1, CommunicationsExportEventConsumerErrorV1>
{
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsExportEventConsumerErrorV1::InvalidEnvelope)?;
    if !exact_contract(envelope.contract.as_ref(), expected)
        || envelope.source.as_ref().is_none_or(|source| {
            source.module_id != "makosh-communications-runtime" || source.runtime_generation == 0
        })
        || !matches!(envelope.semantics, Some(Semantics::Result(_)))
    {
        return Err(CommunicationsExportEventConsumerErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn result_export_id(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    payload_export_id: &[u8],
) -> Result<[u8; 16], CommunicationsExportEventConsumerErrorV1> {
    let export_id = id16(payload_export_id)?;
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(CommunicationsExportEventConsumerErrorV1::InvalidEnvelope);
    };
    if result.command_id.as_slice() != export_id
        || result.command_message_id.as_slice() != export_id
        || envelope.correlation_id.as_slice() != export_id
    {
        return Err(CommunicationsExportEventConsumerErrorV1::InvalidEnvelope);
    }
    Ok(export_id)
}

fn prepared_item(
    item: makosh_communications_evidence_export_source_api::wire::EvidenceExportSourceItemV1,
) -> Result<CommunicationsExportPreparedItemV1, CommunicationsExportEventConsumerErrorV1> {
    let direction = match EvidenceExportDirectionV1::try_from(item.direction) {
        Ok(EvidenceExportDirectionV1::EvidenceExportDirectionIncoming) => CoreDirectionV1::Incoming,
        Ok(EvidenceExportDirectionV1::EvidenceExportDirectionOutgoing) => CoreDirectionV1::Outgoing,
        Ok(EvidenceExportDirectionV1::EvidenceExportDirectionUnknown) => CoreDirectionV1::Unknown,
        _ => return Err(CommunicationsExportEventConsumerErrorV1::InvalidPayload),
    };
    let body_source = match EvidenceExportBodyStateV1::try_from(item.body_state) {
        Ok(EvidenceExportBodyStateV1::EvidenceExportBodyStateAdmittedUtf8) => {
            let source = item
                .body_source
                .ok_or(CommunicationsExportEventConsumerErrorV1::InvalidPayload)?;
            Some(CommunicationsExportSourceReceiptV1 {
                reference_id: id16(&source.reference_id)?,
                declared_bytes: source.declared_bytes,
                sha256: id32(&source.sha256)?,
                custody_transfer_source_proof: source.custody_transfer_source_proof,
            })
        }
        Ok(EvidenceExportBodyStateV1::EvidenceExportBodyStateUnavailable)
            if item.body_source.is_none() =>
        {
            None
        }
        _ => return Err(CommunicationsExportEventConsumerErrorV1::InvalidPayload),
    };
    Ok(CommunicationsExportPreparedItemV1 {
        message_id: id16(&item.message_id)?,
        conversation_id: id16(&item.conversation_id)?,
        evidence_id: id16(&item.evidence_id)?,
        evidence_revision: item.evidence_revision,
        direction,
        occurred_at_unix_seconds: item.occurred_at_unix_seconds,
        observed_at_unix_seconds: item.observed_at_unix_seconds,
        participant_display_label: item.participant_display_label,
        body_source,
    })
}

fn exact_contract(contract: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    contract.is_some_and(|contract| {
        contract.owner == expected.owner
            && contract.name == expected.name
            && contract.major == expected.major
            && contract.revision == expected.revision
            && contract.schema_sha256 == expected.schema_sha256
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsExportEventConsumerErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsExportEventConsumerErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsExportEventConsumerErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|digest: &[u8; 32]| digest.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsExportEventConsumerErrorV1::InvalidPayload)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn persistence_error(
    error: CommunicationsExportPersistenceErrorV1,
) -> CommunicationsExportEventConsumerErrorV1 {
    match error {
        CommunicationsExportPersistenceErrorV1::StorageUnavailable => {
            CommunicationsExportEventConsumerErrorV1::Unavailable
        }
        _ => CommunicationsExportEventConsumerErrorV1::Persistence,
    }
}

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsExportEventConsumerErrorV1 {
    CommunicationsExportEventConsumerErrorV1::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communications_evidence_export_source_api::wire::{
        EvidenceExportBodySourceReceiptV1, EvidenceExportSourceItemV1,
    };

    #[test]
    fn body_state_and_receipt_must_match() {
        let item = EvidenceExportSourceItemV1 {
            message_id: vec![1; 16],
            conversation_id: vec![2; 16],
            evidence_id: vec![3; 16],
            evidence_revision: 1,
            direction: EvidenceExportDirectionV1::EvidenceExportDirectionIncoming as i32,
            occurred_at_unix_seconds: 1,
            observed_at_unix_seconds: 2,
            participant_display_label: None,
            body_state: EvidenceExportBodyStateV1::EvidenceExportBodyStateAdmittedUtf8 as i32,
            body_source: None,
        };
        assert!(prepared_item(item.clone()).is_err());
        assert!(
            prepared_item(EvidenceExportSourceItemV1 {
                body_source: Some(EvidenceExportBodySourceReceiptV1 {
                    reference_id: vec![4; 16],
                    declared_bytes: 1,
                    sha256: vec![5; 32],
                    custody_transfer_source_proof: vec![6],
                }),
                ..item
            })
            .is_ok()
        );
    }
}
