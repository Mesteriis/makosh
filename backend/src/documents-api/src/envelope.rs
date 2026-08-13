use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    DOCUMENTS_MODULE_ID_V1, client_wire::DocumentChangedV1,
    documents_lifecycle_event_contract_reference_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentsEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentsEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_document_changed_outbox_record_v1(
    operation_id: [u8; 16],
    payload: DocumentChangedV1,
    context: &DocumentsEnvelopeContextV1,
) -> Result<OutboxRecordV1, DocumentsEnvelopeBuildErrorV1> {
    if context.module_id != DOCUMENTS_MODULE_ID_V1
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(DocumentsEnvelopeBuildErrorV1::InvalidContext);
    }
    let event_id = id16(&payload.event_id)?;
    let document_id = id16(&payload.document_id)?;
    let occurred_at = payload
        .occurred_at
        .as_ref()
        .filter(|value| value.unix_seconds > 0 && (0..1_000_000_000).contains(&value.nanos))
        .ok_or(DocumentsEnvelopeBuildErrorV1::InvalidPayload)?;
    if !nonzero(&operation_id)
        || payload.document_revision == 0
        || payload.state == 0
        || payload.custody_state == 0
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(DocumentsEnvelopeBuildErrorV1::InvalidPayload);
    }
    let contract = documents_lifecycle_event_contract_reference_v1();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: event_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest(
                b"documents-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"source",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        partition_key: document_id.to_vec(),
        causation_message_id: operation_id.to_vec(),
        correlation_id: document_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(Timestamp {
                seconds: occurred_at.unix_seconds,
                nanos: occurred_at.nanos,
            }),
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope).map_err(|_| DocumentsEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], DocumentsEnvelopeBuildErrorV1> {
    let value: [u8; 16] = bytes
        .try_into()
        .map_err(|_| DocumentsEnvelopeBuildErrorV1::InvalidPayload)?;
    nonzero(&value)
        .then_some(value)
        .ok_or(DocumentsEnvelopeBuildErrorV1::InvalidPayload)
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn digest(domain: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((left.len() as u64).to_be_bytes());
    hash.update(left);
    hash.update((right.len() as u64).to_be_bytes());
    hash.update(right);
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn outbox_error(_: OutboxRecordError) -> DocumentsEnvelopeBuildErrorV1 {
    DocumentsEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_wire::{
        DocumentChangedV1, DocumentCustodyStateV1, DocumentStateV1, TimestampV1,
    };

    #[test]
    fn lifecycle_event_excludes_profile_blob_and_source_details() {
        let record = build_document_changed_outbox_record_v1(
            [1; 16],
            DocumentChangedV1 {
                event_id: vec![2; 16],
                document_id: vec![3; 16],
                logical_owner_id: "owner-1".to_owned(),
                document_revision: 2,
                state: DocumentStateV1::DocumentStateActive as i32,
                custody_state: DocumentCustodyStateV1::DocumentCustodyStateBound as i32,
                occurred_at: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
            },
            &DocumentsEnvelopeContextV1 {
                module_id: DOCUMENTS_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "documents-runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("event");
        for private in [
            b"private-title".as_slice(),
            b"private-file-name".as_slice(),
            b"blob-reference".as_slice(),
        ] {
            assert!(
                !record
                    .exact_bytes()
                    .windows(private.len())
                    .any(|window| window == private)
            );
        }
    }
}
