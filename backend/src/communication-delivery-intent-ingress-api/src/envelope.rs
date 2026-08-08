use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATION_DELIVERY_INTENT_INGRESS_COMMAND_CAPABILITY_ID_V1,
    COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_REVISION_V1,
    COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_BYTES_V1,
    COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_PROOF_BYTES_V1,
    COMMUNICATION_DELIVERY_INTENT_INGRESS_OWNER_V1,
    COMMUNICATION_DELIVERY_INTENT_INGRESS_SCHEMA_SHA256,
    COMMUNICATION_DELIVERY_INTENT_REJECTED_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_SUBMIT_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_SUBMITTED_CONTRACT_NAME_V1,
    wire::{
        CommunicationDeliveryIntentIngressRejectCodeV1, CommunicationDeliveryIntentRejectedV1,
        CommunicationDeliveryIntentSubmittedV1, DeliveryIntentBodySourceReceiptV1,
        SubmitCommunicationDeliveryIntentCommandV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationDeliveryIntentIngressEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_communication_delivery_intent_submit_outbox_record_v1(
    intent_id: [u8; 16],
    target_conversation_id: [u8; 16],
    target_reply_to_message_id: Option<[u8; 16]>,
    body_source: DeliveryIntentBodySourceReceiptV1,
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CommunicationDeliveryIntentIngressEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    validate_context(context)?;
    validate_common_payload(&intent_id, logical_owner_id)?;
    if !valid_id(&target_conversation_id)
        || target_reply_to_message_id
            .as_ref()
            .is_some_and(|reply_id| !valid_id(reply_id))
        || !valid_body_source(&body_source)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = SubmitCommunicationDeliveryIntentCommandV1 {
        intent_id: intent_id.to_vec(),
        target_conversation_id: target_conversation_id.to_vec(),
        target_reply_to_message_id: target_reply_to_message_id
            .map_or_else(Vec::new, |reply_id| reply_id.to_vec()),
        body_source: Some(body_source),
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        communication_delivery_intent_submit_message_id_v1(&intent_id),
        &intent_id,
        &[],
        COMMUNICATION_DELIVERY_INTENT_SUBMIT_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: intent_id.to_vec(),
            target_capability: COMMUNICATION_DELIVERY_INTENT_INGRESS_COMMAND_CAPABILITY_ID_V1
                .to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"communication-delivery-intent-event-ingress-v1".as_slice(),
                    &intent_id,
                ]
                .concat(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        }),
        payload,
        context,
    )
}

#[must_use]
pub fn communication_delivery_intent_submit_message_id_v1(intent_id: &[u8; 16]) -> [u8; 16] {
    let digest = Sha256::digest(
        [
            b"communication-delivery-intent-submit-message-v1".as_slice(),
            intent_id,
        ]
        .concat(),
    );
    digest[..16]
        .try_into()
        .expect("SHA-256 prefix is exactly 16 bytes")
}

pub fn build_communication_delivery_intent_submitted_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationDeliveryIntentSubmittedV1,
    context: &CommunicationDeliveryIntentIngressEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let intent_id = validate_result_payload(&payload.intent_id, &payload.logical_owner_id)?;
    if command_message_id != communication_delivery_intent_submit_message_id_v1(&intent_id) {
        return Err(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        communication_delivery_intent_submitted_message_id_v1(&intent_id),
        &intent_id,
        &command_message_id,
        COMMUNICATION_DELIVERY_INTENT_SUBMITTED_CONTRACT_NAME_V1,
        result_semantics(
            &intent_id,
            &command_message_id,
            ResultOutcomeV1::Succeeded,
            context,
        ),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_delivery_intent_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationDeliveryIntentRejectedV1,
    context: &CommunicationDeliveryIntentIngressEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let intent_id = validate_result_payload(&payload.intent_id, &payload.logical_owner_id)?;
    let reject_code = CommunicationDeliveryIntentIngressRejectCodeV1::try_from(payload.code);
    if command_message_id != communication_delivery_intent_submit_message_id_v1(&intent_id)
        || !matches!(
            reject_code,
            Ok(code)
                if code
                    != CommunicationDeliveryIntentIngressRejectCodeV1::
                        CommunicationDeliveryIntentIngressRejectCodeUnspecified
        )
    {
        return Err(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        communication_delivery_intent_rejected_message_id_v1(&intent_id),
        &intent_id,
        &command_message_id,
        COMMUNICATION_DELIVERY_INTENT_REJECTED_CONTRACT_NAME_V1,
        result_semantics(
            &intent_id,
            &command_message_id,
            ResultOutcomeV1::Rejected,
            context,
        ),
        payload.encode_to_vec(),
        context,
    )
}

fn build_envelope(
    message_id: [u8; 16],
    partition_key: &[u8; 16],
    causation_message_id: &[u8],
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &CommunicationDeliveryIntentIngressEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: COMMUNICATION_DELIVERY_INTENT_INGRESS_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_DELIVERY_INTENT_INGRESS_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: partition_key.to_vec(),
        causation_message_id: causation_message_id.to_vec(),
        correlation_id: partition_key.to_vec(),
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
        semantics: Some(semantics),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn result_semantics(
    command_id: &[u8; 16],
    command_message_id: &[u8; 16],
    outcome: ResultOutcomeV1,
    context: &CommunicationDeliveryIntentIngressEnvelopeContextV1,
) -> Semantics {
    Semantics::Result(ResultMetadataV1 {
        command_id: command_id.to_vec(),
        command_message_id: command_message_id.to_vec(),
        outcome: outcome as i32,
        completed_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        execution_attempt: 1,
    })
}

fn validate_context(
    context: &CommunicationDeliveryIntentIngressEnvelopeContextV1,
) -> Result<(), CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    if context.module_id.is_empty()
        || context.module_id.len() > 128
        || !context.module_id.is_ascii()
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 256
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_result_payload(
    intent_id: &[u8],
    logical_owner_id: &str,
) -> Result<[u8; 16], CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    let intent_id = id16(intent_id)?;
    validate_common_payload(&intent_id, logical_owner_id)?;
    Ok(intent_id)
}

fn validate_common_payload(
    intent_id: &[u8; 16],
    logical_owner_id: &str,
) -> Result<(), CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    if !valid_id(intent_id) || !valid_logical_owner_id(logical_owner_id) {
        return Err(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(())
}

fn valid_body_source(receipt: &DeliveryIntentBodySourceReceiptV1) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_BYTES_V1)
            .contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_PROOF_BYTES_V1
}

fn valid_id(id: &[u8; 16]) -> bool {
    id.iter().any(|byte| *byte != 0)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidPayload)
}

#[must_use]
pub fn communication_delivery_intent_submitted_message_id_v1(intent_id: &[u8; 16]) -> [u8; 16] {
    result_message_id(b"submitted", intent_id)
}

#[must_use]
pub fn communication_delivery_intent_rejected_message_id_v1(intent_id: &[u8; 16]) -> [u8; 16] {
    result_message_id(b"rejected", intent_id)
}

fn result_message_id(label: &[u8], intent_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communication-delivery-intent-event-ingress-result-v1");
    hasher.update(label);
    hasher.update(intent_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    Sha256::digest(runtime_instance_id.as_bytes())[..16]
        .try_into()
        .expect("digest prefix")
}

fn outbox_error(_: OutboxRecordError) -> CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1 {
    CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    fn context() -> CommunicationDeliveryIntentIngressEnvelopeContextV1 {
        CommunicationDeliveryIntentIngressEnvelopeContextV1 {
            module_id: "makosh-communication-cross-channel-forward-runtime".to_owned(),
            runtime_instance_id: "runtime-forward-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 12,
        }
    }

    fn body_source() -> DeliveryIntentBodySourceReceiptV1 {
        DeliveryIntentBodySourceReceiptV1 {
            reference_id: vec![5; 16],
            declared_bytes: 42,
            sha256: vec![6; 32],
            custody_transfer_source_proof: vec![7; 64],
        }
    }

    #[test]
    fn submit_command_is_bodyless_and_uses_exact_capability() {
        let record = build_communication_delivery_intent_submit_outbox_record_v1(
            [1; 16],
            [2; 16],
            Some([3; 16]),
            body_source(),
            "owner-1",
            1_800_000_030,
            &context(),
        )
        .expect("command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Command(command)) = envelope.semantics else {
            panic!("command semantics");
        };
        assert_eq!(
            command.target_capability,
            COMMUNICATION_DELIVERY_INTENT_INGRESS_COMMAND_CAPABILITY_ID_V1
        );
        assert_eq!(
            envelope.message_id,
            communication_delivery_intent_submit_message_id_v1(&[1; 16])
        );
        assert_ne!(envelope.message_id, vec![1; 16]);
        let payload =
            SubmitCommunicationDeliveryIntentCommandV1::decode(envelope.payload.as_slice())
                .expect("payload");
        assert_eq!(payload.target_conversation_id, vec![2; 16]);
        assert_eq!(payload.target_reply_to_message_id, vec![3; 16]);
        assert_eq!(payload.logical_owner_id, "owner-1");
        assert_eq!(
            payload.body_source.expect("receipt").reference_id,
            vec![5; 16]
        );
    }

    #[test]
    fn submit_requires_a_bounded_target_bound_receipt() {
        let mut invalid = body_source();
        invalid.declared_bytes = COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_BYTES_V1 + 1;
        assert_eq!(
            build_communication_delivery_intent_submit_outbox_record_v1(
                [1; 16],
                [2; 16],
                None,
                invalid,
                "owner-1",
                1_800_000_030,
                &context(),
            ),
            Err(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidPayload)
        );
    }

    #[test]
    fn results_are_correlated_and_rejection_codes_are_closed() {
        let submitted = build_communication_delivery_intent_submitted_outbox_record_v1(
            communication_delivery_intent_submit_message_id_v1(&[1; 16]),
            CommunicationDeliveryIntentSubmittedV1 {
                intent_id: vec![1; 16],
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("submitted");
        let envelope = DurableEnvelopeV1::decode(submitted.exact_bytes()).expect("envelope");
        let Some(Semantics::Result(result)) = envelope.semantics else {
            panic!("result semantics");
        };
        assert_eq!(result.command_id, vec![1; 16]);
        assert_eq!(
            result.command_message_id,
            communication_delivery_intent_submit_message_id_v1(&[1; 16])
        );

        let invalid = CommunicationDeliveryIntentRejectedV1 {
            intent_id: vec![1; 16],
            code: i32::MAX,
            logical_owner_id: "owner-1".to_owned(),
        };
        assert_eq!(
            build_communication_delivery_intent_rejected_outbox_record_v1(
                communication_delivery_intent_submit_message_id_v1(&[1; 16]),
                invalid,
                &context(),
            ),
            Err(CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1::InvalidPayload)
        );
    }
}
