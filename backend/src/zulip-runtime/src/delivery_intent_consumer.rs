//! Exact Zulip delivery-intent durable command decoder.
//!
//! This module owns only the Zulip integration boundary. It deliberately has no
//! provider discriminator and imports no workflow or Communications domain
//! implementation.

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1, ResultMetadataV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::{decode_envelope_v1, validate_envelope_v1},
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use makosh_zulip_delivery_intent_contract::{
    ZULIP_DELIVERY_INTENT_SOURCE_MODULE_ID_V1, ZULIP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
    validate_zulip_delivery_intent_execute_v1, validate_zulip_delivery_intent_rejected_v1,
    wire::{
        ExecuteZulipDeliveryIntentCommandV1, ZulipDeliveryIntentRejectCodeV1,
        ZulipDeliveryIntentRejectedV1,
    },
    zulip_delivery_intent_execute_contract_reference_v1,
    zulip_delivery_intent_rejected_contract_reference_v1,
};
use makosh_zulip_persistence::{
    ZulipDeliveryIntentAdmissionV1, ZulipDeliveryIntentInboxOutcomeV1, ZulipDeliveryIntentStoreV1,
    ZulipDurablePersistenceError,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

const MESSAGE_DOMAIN: &[u8] = b"makosh.zulip.delivery-intent.execute.v1";
const REJECTED_MESSAGE_DOMAIN: &[u8] = b"makosh.zulip.delivery-intent.rejected.v1";
const ZULIP_RUNTIME_MODULE_ID: &str = "makosh-zulip-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedZulipDeliveryIntentV1 {
    pub command_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub intent_id: [u8; 16],
    pub logical_owner_id: String,
    pub account_source_cursor: [u8; 32],
    pub conversation_source_cursor: [u8; 32],
    pub reply_to_source_cursor: Option<[u8; 32]>,
    pub body_reference_id: [u8; 16],
    pub body_declared_bytes: u64,
    pub body_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipDeliveryIntentDecodeErrorV1 {
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongAudience,
    InvalidPayload,
    OwnerMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipDeliveryIntentResultContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub completed_at_unix_seconds: i64,
    pub completed_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipDeliveryIntentConsumeErrorV1 {
    Unavailable,
    Decode(ZulipDeliveryIntentDecodeErrorV1),
    InvalidResultContext,
    InvalidResultEnvelope,
    Persistence,
}

pub async fn consume_next_zulip_delivery_intent_v1(
    store: &ZulipDeliveryIntentStoreV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    result_context: &ZulipDeliveryIntentResultContextV1,
) -> Result<ZulipDeliveryIntentInboxOutcomeV1, ZulipDeliveryIntentConsumeErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| ZulipDeliveryIntentConsumeErrorV1::Unavailable)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec()).map_err(|_| {
        ZulipDeliveryIntentConsumeErrorV1::Decode(ZulipDeliveryIntentDecodeErrorV1::InvalidEnvelope)
    })?;
    let outcome =
        accept_zulip_delivery_intent_v1(store, &record, expected_logical_owner_id, result_context)
            .await?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| ZulipDeliveryIntentConsumeErrorV1::Unavailable)?;
    Ok(outcome)
}

pub async fn accept_zulip_delivery_intent_v1(
    store: &ZulipDeliveryIntentStoreV1,
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
    result_context: &ZulipDeliveryIntentResultContextV1,
) -> Result<ZulipDeliveryIntentInboxOutcomeV1, ZulipDeliveryIntentConsumeErrorV1> {
    let decoded = decode_zulip_delivery_intent_v1(record, expected_logical_owner_id)
        .map_err(ZulipDeliveryIntentConsumeErrorV1::Decode)?;
    let route_not_found = build_rejected_outbox_v1(
        &decoded,
        ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeRouteNotFound,
        result_context,
    )?;
    store
        .accept_command(
            &ZulipDeliveryIntentAdmissionV1 {
                command_message_id: decoded.command_message_id,
                envelope_sha256: decoded.envelope_sha256,
                intent_id: decoded.intent_id,
                logical_owner_id: decoded.logical_owner_id,
                account_source_cursor: decoded.account_source_cursor,
                conversation_source_cursor: decoded.conversation_source_cursor,
                reply_to_source_cursor: decoded.reply_to_source_cursor,
                body_reference_id: decoded.body_reference_id,
                body_declared_bytes: decoded.body_declared_bytes,
                body_sha256: decoded.body_sha256,
                custody_transfer_source_proof: decoded.custody_transfer_source_proof,
            },
            &route_not_found,
            result_context.completed_at_unix_seconds,
        )
        .await
        .map_err(persistence_error)
}

pub fn decode_zulip_delivery_intent_v1(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<DecodedZulipDeliveryIntentV1, ZulipDeliveryIntentDecodeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ZulipDeliveryIntentDecodeErrorV1::InvalidEnvelope)?;
    let expected_contract = zulip_delivery_intent_execute_contract_reference_v1();
    if !exact_contract(envelope.contract.as_ref(), &expected_contract) {
        return Err(ZulipDeliveryIntentDecodeErrorV1::WrongContract);
    }
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(ZulipDeliveryIntentDecodeErrorV1::WrongContract);
    };
    if metadata.target_capability != ZULIP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1 {
        return Err(ZulipDeliveryIntentDecodeErrorV1::WrongAudience);
    }
    let source = envelope
        .source
        .as_ref()
        .ok_or(ZulipDeliveryIntentDecodeErrorV1::WrongSource)?;
    if source.module_id != ZULIP_DELIVERY_INTENT_SOURCE_MODULE_ID_V1
        || source.runtime_generation == 0
        || envelope.source_fence.as_ref().is_none_or(|fence| {
            fence.scope_id != ZULIP_DELIVERY_INTENT_SOURCE_MODULE_ID_V1.as_bytes()
                || fence.epoch != source.runtime_generation
        })
    {
        return Err(ZulipDeliveryIntentDecodeErrorV1::WrongSource);
    }

    let command = ExecuteZulipDeliveryIntentCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| ZulipDeliveryIntentDecodeErrorV1::InvalidPayload)?;
    validate_zulip_delivery_intent_execute_v1(&command)
        .map_err(|_| ZulipDeliveryIntentDecodeErrorV1::InvalidPayload)?;
    if command.logical_owner_id != expected_logical_owner_id {
        return Err(ZulipDeliveryIntentDecodeErrorV1::OwnerMismatch);
    }

    let intent_id = id16(&command.intent_id)?;
    let command_message_id = id16(&envelope.message_id)?;
    if metadata.command_id.as_slice() != intent_id
        || envelope.partition_key.as_slice() != intent_id
        || envelope.correlation_id.as_slice() != intent_id
        || command_message_id != message_id(intent_id)
    {
        return Err(ZulipDeliveryIntentDecodeErrorV1::WrongContract);
    }
    let body = command
        .body_source
        .ok_or(ZulipDeliveryIntentDecodeErrorV1::InvalidPayload)?;
    Ok(DecodedZulipDeliveryIntentV1 {
        command_message_id,
        envelope_sha256: *record.envelope_sha256(),
        intent_id,
        logical_owner_id: command.logical_owner_id,
        account_source_cursor: id32(&command.account_source_cursor)?,
        conversation_source_cursor: id32(&command.conversation_source_cursor)?,
        reply_to_source_cursor: command
            .reply_to_source_cursor
            .as_deref()
            .map(id32)
            .transpose()?,
        body_reference_id: id16(&body.reference_id)?,
        body_declared_bytes: body.declared_bytes,
        body_sha256: id32(&body.sha256)?,
        custody_transfer_source_proof: body.custody_transfer_source_proof,
    })
}

fn exact_contract(
    value: Option<&makosh_events_protocol::v1::ContractRefV1>,
    expected: &ContractReferenceV1,
) -> bool {
    value.is_some_and(|value| {
        value.owner == expected.owner
            && value.name == expected.name
            && value.major == expected.major
            && value.revision == expected.revision
            && value.schema_sha256.as_slice() == expected.schema_sha256
    })
}

fn message_id(intent_id: [u8; 16]) -> [u8; 16] {
    identifier(MESSAGE_DOMAIN, intent_id)
}

fn identifier(domain: &[u8], identity: [u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(identity);
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("SHA-256 prefix has fixed size")
}

fn build_rejected_outbox_v1(
    decoded: &DecodedZulipDeliveryIntentV1,
    code: ZulipDeliveryIntentRejectCodeV1,
    context: &ZulipDeliveryIntentResultContextV1,
) -> Result<OutboxRecordV1, ZulipDeliveryIntentConsumeErrorV1> {
    if context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 256
        || context.runtime_generation == 0
        || context.completed_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.completed_at_nanos)
    {
        return Err(ZulipDeliveryIntentConsumeErrorV1::InvalidResultContext);
    }
    let payload = ZulipDeliveryIntentRejectedV1 {
        intent_id: decoded.intent_id.to_vec(),
        logical_owner_id: decoded.logical_owner_id.clone(),
        code: code as i32,
    };
    validate_zulip_delivery_intent_rejected_v1(&payload)
        .map_err(|_| ZulipDeliveryIntentConsumeErrorV1::InvalidResultEnvelope)?;
    let completed_at = Timestamp {
        seconds: context.completed_at_unix_seconds,
        nanos: context.completed_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: identifier(REJECTED_MESSAGE_DOMAIN, decoded.intent_id).to_vec(),
        contract: Some(wire_contract(
            zulip_delivery_intent_rejected_contract_reference_v1(),
        )),
        source: Some(SourceRefV1 {
            module_id: ZULIP_RUNTIME_MODULE_ID.to_owned(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(completed_at),
        partition_key: decoded.intent_id.to_vec(),
        causation_message_id: decoded.command_message_id.to_vec(),
        correlation_id: decoded.intent_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: ZULIP_RUNTIME_MODULE_ID.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: ZULIP_RUNTIME_MODULE_ID.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: decoded.intent_id.to_vec(),
            command_message_id: decoded.command_message_id.to_vec(),
            outcome: ResultOutcomeV1::Rejected as i32,
            completed_at: Some(Timestamp {
                seconds: context.completed_at_unix_seconds,
                nanos: context.completed_at_nanos,
            }),
            execution_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ZulipDeliveryIntentConsumeErrorV1::InvalidResultEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let digest: [u8; 32] = Sha256::digest(runtime_instance_id.as_bytes()).into();
    digest[..16]
        .try_into()
        .expect("SHA-256 prefix has fixed size")
}

fn wire_contract(value: ContractReferenceV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: value.owner.to_owned(),
        name: value.name.to_owned(),
        major: value.major,
        revision: value.revision,
        schema_sha256: value.schema_sha256.to_vec(),
    }
}

fn persistence_error(_: ZulipDurablePersistenceError) -> ZulipDeliveryIntentConsumeErrorV1 {
    ZulipDeliveryIntentConsumeErrorV1::Persistence
}

fn outbox_error(_: OutboxRecordError) -> ZulipDeliveryIntentConsumeErrorV1 {
    ZulipDeliveryIntentConsumeErrorV1::InvalidResultEnvelope
}

fn id16(value: &[u8]) -> Result<[u8; 16], ZulipDeliveryIntentDecodeErrorV1> {
    value
        .try_into()
        .map_err(|_| ZulipDeliveryIntentDecodeErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], ZulipDeliveryIntentDecodeErrorV1> {
    value
        .try_into()
        .map_err(|_| ZulipDeliveryIntentDecodeErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::{
        delivery::OutboxRecordV1,
        v1::{
            ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1,
            FenceKindV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
        },
    };
    use makosh_runtime_protocol::v1::ContractReferenceV1;
    use makosh_zulip_delivery_intent_contract::{
        ZULIP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
        wire::ZulipDeliveryIntentBodySourceReceiptV1,
    };
    use prost_types::Timestamp;

    use super::*;

    #[test]
    fn decoder_accepts_only_exact_zulip_contract_source_audience_and_owner() {
        let record = command_record("owner-1");
        let decoded =
            decode_zulip_delivery_intent_v1(&record, "owner-1").expect("exact Zulip command");
        assert_eq!(decoded.intent_id, [1; 16]);
        assert_eq!(decoded.account_source_cursor, [2; 32]);
        assert_eq!(decoded.conversation_source_cursor, [3; 32]);
        assert_eq!(decoded.reply_to_source_cursor, Some([4; 32]));
        assert_eq!(decoded.body_reference_id, [5; 16]);
        assert_eq!(
            ZULIP_DELIVERY_INTENT_TARGET_BLOB_CAPABILITY_ID_V1,
            "zulip.blob.v1"
        );
        assert_eq!(
            decode_zulip_delivery_intent_v1(&record, "other-owner"),
            Err(ZulipDeliveryIntentDecodeErrorV1::OwnerMismatch)
        );
    }

    #[test]
    fn decoder_rejects_a_valid_envelope_with_the_wrong_target_capability() {
        let record = command_record_with("owner-1", "zulip.delivery-intent.execute.v1");
        assert_eq!(
            decode_zulip_delivery_intent_v1(&record, "owner-1"),
            Err(ZulipDeliveryIntentDecodeErrorV1::WrongAudience)
        );
    }

    #[test]
    fn route_failure_result_is_causally_bound_and_contains_no_free_text() {
        let decoded = decode_zulip_delivery_intent_v1(&command_record("owner-1"), "owner-1")
            .expect("exact Zulip command");
        let record = build_rejected_outbox_v1(
            &decoded,
            ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeRouteNotFound,
            &ZulipDeliveryIntentResultContextV1 {
                runtime_instance_id: "zulip-runtime-1".to_owned(),
                runtime_generation: 2,
                completed_at_unix_seconds: 1_700_000_001,
                completed_at_nanos: 0,
            },
        )
        .expect("rejected result");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("valid envelope");
        assert_eq!(
            envelope.causation_message_id,
            decoded.command_message_id.to_vec()
        );
        assert_eq!(envelope.correlation_id, decoded.intent_id.to_vec());
        let payload = ZulipDeliveryIntentRejectedV1::decode(envelope.payload.as_slice())
            .expect("typed rejection");
        assert_eq!(
            payload.code,
            ZulipDeliveryIntentRejectCodeV1::ZulipDeliveryIntentRejectCodeRouteNotFound as i32
        );
        assert!(
            !record
                .exact_bytes()
                .windows(b"route not found".len())
                .any(|window| window == b"route not found")
        );
    }

    fn command_record(owner: &str) -> OutboxRecordV1 {
        command_record_with(owner, ZULIP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1)
    }

    fn command_record_with(owner: &str, target_capability: &str) -> OutboxRecordV1 {
        let intent_id = [1; 16];
        let contract = zulip_delivery_intent_execute_contract_reference_v1();
        let payload = ExecuteZulipDeliveryIntentCommandV1 {
            intent_id: intent_id.to_vec(),
            logical_owner_id: owner.to_owned(),
            account_source_cursor: vec![2; 32],
            conversation_source_cursor: vec![3; 32],
            reply_to_source_cursor: Some(vec![4; 32]),
            body_source: Some(ZulipDeliveryIntentBodySourceReceiptV1 {
                reference_id: vec![5; 16],
                declared_bytes: 5,
                sha256: vec![6; 32],
                custody_transfer_source_proof: vec![7; 64],
            }),
        };
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: message_id(intent_id).to_vec(),
            contract: Some(wire_contract(contract)),
            source: Some(SourceRefV1 {
                module_id: ZULIP_DELIVERY_INTENT_SOURCE_MODULE_ID_V1.to_owned(),
                runtime_instance_id: vec![8; 16],
                runtime_generation: 1,
            }),
            recorded_at: Some(Timestamp {
                seconds: 1_700_000_000,
                nanos: 0,
            }),
            partition_key: intent_id.to_vec(),
            causation_message_id: vec![9; 16],
            correlation_id: intent_id.to_vec(),
            actor: Some(ActorRefV1 {
                kind: ActorKindV1::Module as i32,
                actor_id: ZULIP_DELIVERY_INTENT_SOURCE_MODULE_ID_V1
                    .as_bytes()
                    .to_vec(),
            }),
            trace: None,
            source_fence: Some(SourceFenceV1 {
                kind: FenceKindV1::RuntimeLease as i32,
                scope_id: ZULIP_DELIVERY_INTENT_SOURCE_MODULE_ID_V1
                    .as_bytes()
                    .to_vec(),
                epoch: 1,
            }),
            semantics: Some(Semantics::Command(CommandMetadataV1 {
                command_id: intent_id.to_vec(),
                target_capability: target_capability.to_owned(),
                idempotency_key: vec![10; 32],
                deadline: Some(Timestamp {
                    seconds: 1_700_000_600,
                    nanos: 0,
                }),
                logical_attempt: 1,
            })),
            payload: payload.encode_to_vec(),
        };
        OutboxRecordV1::accept(envelope.encode_to_vec()).expect("valid outbox record")
    }

    fn wire_contract(value: ContractReferenceV1) -> ContractRefV1 {
        ContractRefV1 {
            owner: value.owner.to_owned(),
            name: value.name.to_owned(),
            major: value.major,
            revision: value.revision,
            schema_sha256: value.schema_sha256.to_vec(),
        }
    }
}
