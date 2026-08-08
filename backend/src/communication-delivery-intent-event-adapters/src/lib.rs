pub mod mail;
pub mod telegram;
pub mod whatsapp;
pub mod zulip;

use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-communication-delivery-intent-event-adapters";
pub const DELIVERY_INTENT_RUNTIME_MODULE_ID_V1: &str =
    "makosh-communication-delivery-intent-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentCommandContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
    pub deadline_unix_seconds: i64,
    pub causation_message_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentBodySourceV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryIntentTerminalOutcomeV1 {
    Succeeded { provider_operation_id: Vec<u8> },
    Rejected { rejection_code: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedDeliveryIntentTerminalV1 {
    pub envelope_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub command_message_id: [u8; 16],
    pub intent_id: [u8; 16],
    pub logical_owner_id: String,
    pub outcome: DeliveryIntentTerminalOutcomeV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentEventAdapterErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongResult,
}

pub(crate) fn build_command_outbox_v1(
    intent_id: [u8; 16],
    logical_owner_id: &str,
    contract: ContractReferenceV1,
    target_capability: &str,
    message_domain: &[u8],
    payload: Vec<u8>,
    context: &DeliveryIntentCommandContextV1,
) -> Result<OutboxRecordV1, DeliveryIntentEventAdapterErrorV1> {
    validate_command_context(context)?;
    if intent_id.iter().all(|byte| *byte == 0)
        || logical_owner_id.is_empty()
        || target_capability.is_empty()
    {
        return Err(DeliveryIntentEventAdapterErrorV1::InvalidPayload);
    }
    let message_id = identifier(message_domain, &intent_id);
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(wire_contract(contract)),
        source: Some(SourceRefV1 {
            module_id: DELIVERY_INTENT_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: intent_id.to_vec(),
        causation_message_id: context.causation_message_id.to_vec(),
        correlation_id: intent_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: DELIVERY_INTENT_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: DELIVERY_INTENT_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: intent_id.to_vec(),
            target_capability: target_capability.to_owned(),
            idempotency_key: idempotency_key(message_domain, logical_owner_id, &intent_id),
            deadline: Some(Timestamp {
                seconds: context.deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub(crate) fn decode_result_envelope_v1(
    exact_bytes: &[u8],
    expected_contract: &ContractReferenceV1,
    expected_source_module: &str,
    expected_outcome: ResultOutcomeV1,
) -> Result<(DurableEnvelopeV1, [u8; 16], [u8; 32]), DeliveryIntentEventAdapterErrorV1> {
    let envelope = DurableEnvelopeV1::decode(exact_bytes)
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidEnvelope)?;
    validate_envelope_v1(&envelope)
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidEnvelope)?;
    if !exact_contract(envelope.contract.as_ref(), expected_contract) {
        return Err(DeliveryIntentEventAdapterErrorV1::WrongContract);
    }
    if envelope
        .source
        .as_ref()
        .map(|source| source.module_id.as_str())
        != Some(expected_source_module)
    {
        return Err(DeliveryIntentEventAdapterErrorV1::WrongSource);
    }
    let result = match envelope.semantics.as_ref() {
        Some(Semantics::Result(result)) if result.outcome == expected_outcome as i32 => result,
        _ => return Err(DeliveryIntentEventAdapterErrorV1::WrongResult),
    };
    let command_message_id = id16(&result.command_message_id)?;
    if envelope.causation_message_id.as_slice() != command_message_id.as_slice()
        || result.execution_attempt == 0
    {
        return Err(DeliveryIntentEventAdapterErrorV1::WrongResult);
    }
    id16(&envelope.message_id)?;
    let envelope_sha256 = Sha256::digest(exact_bytes).into();
    Ok((envelope, command_message_id, envelope_sha256))
}

pub(crate) fn validate_result_identity_v1(
    envelope: &DurableEnvelopeV1,
    intent_id: &[u8],
) -> Result<[u8; 16], DeliveryIntentEventAdapterErrorV1> {
    let intent_id = id16(intent_id)?;
    let result = match envelope.semantics.as_ref() {
        Some(Semantics::Result(result)) => result,
        _ => return Err(DeliveryIntentEventAdapterErrorV1::WrongResult),
    };
    if result.command_id.as_slice() != intent_id.as_slice()
        || envelope.partition_key.as_slice() != intent_id.as_slice()
        || envelope.correlation_id.as_slice() != intent_id.as_slice()
    {
        return Err(DeliveryIntentEventAdapterErrorV1::WrongResult);
    }
    Ok(intent_id)
}

fn validate_command_context(
    context: &DeliveryIntentCommandContextV1,
) -> Result<(), DeliveryIntentEventAdapterErrorV1> {
    if context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
        || context.deadline_unix_seconds <= context.recorded_at_unix_seconds
        || context.causation_message_id.iter().all(|byte| *byte == 0)
    {
        return Err(DeliveryIntentEventAdapterErrorV1::InvalidContext);
    }
    Ok(())
}

fn exact_contract(value: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    value.is_some_and(|value| {
        value.owner == expected.owner
            && value.name == expected.name
            && value.major == expected.major
            && value.revision == expected.revision
            && value.schema_sha256 == expected.schema_sha256
    })
}

fn wire_contract(reference: ContractReferenceV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: reference.owner,
        name: reference.name,
        major: reference.major,
        revision: reference.revision,
        schema_sha256: reference.schema_sha256,
    }
}

fn identifier(domain: &[u8], intent_id: &[u8; 16]) -> [u8; 16] {
    let digest = Sha256::digest([domain, intent_id.as_slice()].concat());
    digest[..16].try_into().expect("SHA-256 prefix")
}

fn idempotency_key(domain: &[u8], logical_owner_id: &str, intent_id: &[u8; 16]) -> Vec<u8> {
    Sha256::digest(
        [
            domain,
            b"\0",
            logical_owner_id.as_bytes(),
            b"\0",
            intent_id.as_slice(),
        ]
        .concat(),
    )
    .to_vec()
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let digest = Sha256::digest(runtime_instance_id.as_bytes());
    digest[..16].try_into().expect("SHA-256 prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], DeliveryIntentEventAdapterErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::WrongResult)?;
    if value.iter().all(|byte| *byte == 0) {
        return Err(DeliveryIntentEventAdapterErrorV1::WrongResult);
    }
    Ok(value)
}

fn outbox_error(_: OutboxRecordError) -> DeliveryIntentEventAdapterErrorV1 {
    DeliveryIntentEventAdapterErrorV1::InvalidEnvelope
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::v1::{
        ResultMetadataV1, ResultOutcomeV1, durable_envelope_v1::Semantics,
    };
    use makosh_mail_delivery_intent_contract::{
        MAIL_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1,
        mail_delivery_intent_succeeded_contract_reference_v1, wire::MailDeliveryIntentSucceededV1,
    };
    use makosh_telegram_delivery_intent_contract::TELEGRAM_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1;
    use makosh_whatsapp_delivery_intent_contract::WHATSAPP_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1;
    use makosh_zulip_delivery_intent_contract::ZULIP_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1;

    use super::*;

    fn context() -> DeliveryIntentCommandContextV1 {
        DeliveryIntentCommandContextV1 {
            runtime_instance_id: "delivery-runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 17,
            deadline_unix_seconds: 1_800_000_300,
            causation_message_id: [8; 16],
        }
    }

    fn body_source() -> DeliveryIntentBodySourceV1 {
        DeliveryIntentBodySourceV1 {
            reference_id: [9; 16],
            declared_bytes: 42,
            sha256: [10; 32],
            custody_transfer_source_proof: vec![11; 96],
        }
    }

    #[test]
    fn four_exact_builders_emit_distinct_provider_contracts_without_plaintext() {
        let intent_id = [1; 16];
        let owner = "owner-1";
        let account = [2; 32];
        let conversation = [3; 32];
        let reply = Some([4; 32]);
        let source = body_source();
        let context = context();
        let records = [
            mail::build_execute_outbox_v1(
                intent_id,
                owner,
                account,
                conversation,
                reply,
                &source,
                &context,
            )
            .expect("mail command"),
            telegram::build_execute_outbox_v1(
                intent_id,
                owner,
                account,
                conversation,
                reply,
                &source,
                &context,
            )
            .expect("telegram command"),
            whatsapp::build_execute_outbox_v1(
                intent_id,
                owner,
                account,
                conversation,
                reply,
                &source,
                &context,
            )
            .expect("whatsapp command"),
            zulip::build_execute_outbox_v1(
                intent_id,
                owner,
                account,
                conversation,
                reply,
                &source,
                &context,
            )
            .expect("zulip command"),
        ];
        let expected = [
            MAIL_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1,
            TELEGRAM_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1,
            WHATSAPP_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1,
            ZULIP_DELIVERY_INTENT_EXECUTE_CONTRACT_NAME_V1,
        ];
        for (record, expected_name) in records.iter().zip(expected) {
            let envelope =
                DurableEnvelopeV1::decode(record.exact_bytes()).expect("durable envelope");
            assert_eq!(
                envelope.contract.as_ref().map(|value| value.name.as_str()),
                Some(expected_name)
            );
            assert_eq!(envelope.partition_key, intent_id);
            assert_eq!(envelope.correlation_id, intent_id);
            assert!(
                !record
                    .exact_bytes()
                    .windows(b"private plaintext".len())
                    .any(|window| window == b"private plaintext")
            );
        }
        assert!(
            records
                .windows(2)
                .all(|pair| pair[0].message_id() != pair[1].message_id())
        );
    }

    #[test]
    fn exact_result_decoder_binds_source_contract_and_command_causation() {
        let intent_id = [1; 16];
        let command_message_id = [2; 16];
        let payload = MailDeliveryIntentSucceededV1 {
            intent_id: intent_id.to_vec(),
            logical_owner_id: "owner-1".to_owned(),
            provider_operation_id: vec![3; 24],
        }
        .encode_to_vec();
        let bytes = result_envelope(
            mail_delivery_intent_succeeded_contract_reference_v1(),
            "makosh-mail-runtime",
            intent_id,
            command_message_id,
            ResultOutcomeV1::Succeeded,
            payload,
        );
        let decoded = mail::decode_succeeded_v1(&bytes).expect("mail result");
        assert_eq!(decoded.intent_id, intent_id);
        assert_eq!(decoded.command_message_id, command_message_id);
        assert_eq!(
            decoded.outcome,
            DeliveryIntentTerminalOutcomeV1::Succeeded {
                provider_operation_id: vec![3; 24]
            }
        );

        let wrong_source = result_envelope(
            mail_delivery_intent_succeeded_contract_reference_v1(),
            "makosh-telegram-runtime",
            intent_id,
            command_message_id,
            ResultOutcomeV1::Succeeded,
            MailDeliveryIntentSucceededV1 {
                intent_id: intent_id.to_vec(),
                logical_owner_id: "owner-1".to_owned(),
                provider_operation_id: vec![3; 24],
            }
            .encode_to_vec(),
        );
        assert_eq!(
            mail::decode_succeeded_v1(&wrong_source),
            Err(DeliveryIntentEventAdapterErrorV1::WrongSource)
        );
    }

    fn result_envelope(
        contract: ContractReferenceV1,
        source_module: &str,
        intent_id: [u8; 16],
        command_message_id: [u8; 16],
        outcome: ResultOutcomeV1,
        payload: Vec<u8>,
    ) -> Vec<u8> {
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: [12; 16].to_vec(),
            contract: Some(wire_contract(contract)),
            source: Some(SourceRefV1 {
                module_id: source_module.to_owned(),
                runtime_instance_id: [13; 16].to_vec(),
                runtime_generation: 2,
            }),
            recorded_at: Some(Timestamp {
                seconds: 1_800_000_010,
                nanos: 0,
            }),
            partition_key: intent_id.to_vec(),
            causation_message_id: command_message_id.to_vec(),
            correlation_id: intent_id.to_vec(),
            actor: Some(ActorRefV1 {
                kind: ActorKindV1::Module as i32,
                actor_id: source_module.as_bytes().to_vec(),
            }),
            trace: None,
            source_fence: Some(SourceFenceV1 {
                kind: FenceKindV1::RuntimeLease as i32,
                scope_id: source_module.as_bytes().to_vec(),
                epoch: 2,
            }),
            semantics: Some(Semantics::Result(ResultMetadataV1 {
                command_id: intent_id.to_vec(),
                command_message_id: command_message_id.to_vec(),
                outcome: outcome as i32,
                completed_at: Some(Timestamp {
                    seconds: 1_800_000_010,
                    nanos: 0,
                }),
                execution_attempt: 1,
            })),
            payload,
        };
        validate_envelope_v1(&envelope).expect("valid result envelope");
        envelope.encode_to_vec()
    }
}
