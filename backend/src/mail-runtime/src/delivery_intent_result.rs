//! Mail-owned terminal delivery-intent result envelopes.

use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1, ResultMetadataV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_mail_delivery_intent_contract::{
    mail_delivery_intent_rejected_contract_reference_v1,
    mail_delivery_intent_succeeded_contract_reference_v1,
    validate_mail_delivery_intent_rejected_v1, validate_mail_delivery_intent_succeeded_v1,
    wire::{
        MailDeliveryIntentRejectCodeV1, MailDeliveryIntentRejectedV1, MailDeliveryIntentSucceededV1,
    },
};
use makosh_mail_persistence::MailDeliveryIntentJobV1;
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

const SUCCEEDED_MESSAGE_DOMAIN: &[u8] = b"makosh.mail.delivery-intent.succeeded.v1";
const REJECTED_MESSAGE_DOMAIN: &[u8] = b"makosh.mail.delivery-intent.rejected.v1";
const MAIL_RUNTIME_MODULE_ID: &str = "makosh-mail-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDeliveryIntentResultContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub completed_at_unix_seconds: i64,
    pub completed_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryIntentResultErrorV1 {
    InvalidContext,
    InvalidEnvelope,
}

pub fn build_mail_delivery_intent_succeeded_outbox_v1(
    job: &MailDeliveryIntentJobV1,
    execution_attempt: u32,
    context: &MailDeliveryIntentResultContextV1,
) -> Result<OutboxRecordV1, MailDeliveryIntentResultErrorV1> {
    let payload = MailDeliveryIntentSucceededV1 {
        intent_id: job.intent_id.to_vec(),
        logical_owner_id: job.logical_owner_id.clone(),
        provider_operation_id: job.provider_operation_id.as_bytes().to_vec(),
    };
    validate_mail_delivery_intent_succeeded_v1(&payload)
        .map_err(|_| MailDeliveryIntentResultErrorV1::InvalidEnvelope)?;
    build_result_outbox_v1(
        job,
        execution_attempt,
        context,
        mail_delivery_intent_succeeded_contract_reference_v1(),
        SUCCEEDED_MESSAGE_DOMAIN,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
    )
}

pub fn build_mail_delivery_intent_rejected_outbox_v1(
    job: &MailDeliveryIntentJobV1,
    code: MailDeliveryIntentRejectCodeV1,
    execution_attempt: u32,
    context: &MailDeliveryIntentResultContextV1,
) -> Result<OutboxRecordV1, MailDeliveryIntentResultErrorV1> {
    let payload = MailDeliveryIntentRejectedV1 {
        intent_id: job.intent_id.to_vec(),
        logical_owner_id: job.logical_owner_id.clone(),
        code: code as i32,
    };
    validate_mail_delivery_intent_rejected_v1(&payload)
        .map_err(|_| MailDeliveryIntentResultErrorV1::InvalidEnvelope)?;
    build_result_outbox_v1(
        job,
        execution_attempt,
        context,
        mail_delivery_intent_rejected_contract_reference_v1(),
        REJECTED_MESSAGE_DOMAIN,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
    )
}

fn build_result_outbox_v1(
    job: &MailDeliveryIntentJobV1,
    execution_attempt: u32,
    context: &MailDeliveryIntentResultContextV1,
    contract: ContractReferenceV1,
    message_domain: &[u8],
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
) -> Result<OutboxRecordV1, MailDeliveryIntentResultErrorV1> {
    if context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 256
        || context.runtime_generation == 0
        || context.completed_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.completed_at_nanos)
        || execution_attempt == 0
    {
        return Err(MailDeliveryIntentResultErrorV1::InvalidContext);
    }
    let completed_at = Timestamp {
        seconds: context.completed_at_unix_seconds,
        nanos: context.completed_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: identifier(message_domain, job.intent_id).to_vec(),
        contract: Some(wire_contract(contract)),
        source: Some(SourceRefV1 {
            module_id: MAIL_RUNTIME_MODULE_ID.to_owned(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(completed_at),
        partition_key: job.intent_id.to_vec(),
        causation_message_id: job.command_message_id.to_vec(),
        correlation_id: job.intent_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_RUNTIME_MODULE_ID.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_RUNTIME_MODULE_ID.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: job.intent_id.to_vec(),
            command_message_id: job.command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(Timestamp {
                seconds: context.completed_at_unix_seconds,
                nanos: context.completed_at_nanos,
            }),
            execution_attempt,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailDeliveryIntentResultErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
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

fn outbox_error(_: OutboxRecordError) -> MailDeliveryIntentResultErrorV1 {
    MailDeliveryIntentResultErrorV1::InvalidEnvelope
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::{
        v1::{ResultOutcomeV1, durable_envelope_v1::Semantics},
        validation::envelope::decode_envelope_v1,
    };

    use super::*;

    #[test]
    fn terminal_results_are_causally_bound_and_provider_neutral() {
        let job = job();
        let context = context();
        let succeeded = build_mail_delivery_intent_succeeded_outbox_v1(&job, 2, &context)
            .expect("succeeded result");
        let envelope = decode_envelope_v1(succeeded.exact_bytes()).expect("valid envelope");
        assert_eq!(envelope.causation_message_id, job.command_message_id);
        assert_eq!(envelope.correlation_id, job.intent_id);
        let Semantics::Result(metadata) = envelope.semantics.expect("result semantics") else {
            panic!("expected result semantics");
        };
        assert_eq!(metadata.outcome, ResultOutcomeV1::Succeeded as i32);
        assert_eq!(metadata.execution_attempt, 2);

        let rejected = build_mail_delivery_intent_rejected_outbox_v1(
            &job,
            MailDeliveryIntentRejectCodeV1::MailDeliveryIntentRejectCodeProviderAmbiguous,
            3,
            &context,
        )
        .expect("rejected result");
        assert_ne!(succeeded.message_id(), rejected.message_id());
        assert!(
            !rejected
                .exact_bytes()
                .windows(job.recipient.len())
                .any(|window| window == job.recipient.as_bytes())
        );
    }

    fn context() -> MailDeliveryIntentResultContextV1 {
        MailDeliveryIntentResultContextV1 {
            runtime_instance_id: "mail-runtime-1".to_owned(),
            runtime_generation: 7,
            completed_at_unix_seconds: 1_700_000_000,
            completed_at_nanos: 0,
        }
    }

    fn job() -> MailDeliveryIntentJobV1 {
        MailDeliveryIntentJobV1 {
            intent_id: [1; 16],
            command_message_id: [2; 16],
            command_envelope_sha256: [3; 32],
            logical_owner_id: "owner-1".to_owned(),
            connection_id: "mail-1".to_owned(),
            provider_thread_id: "thread-1".to_owned(),
            reply_to_provider_message_id: Some("message-1".to_owned()),
            recipient: "private@example.test".to_owned(),
            subject: "Re: private".to_owned(),
            body_reference_id: [4; 16],
            body_declared_bytes: 4,
            body_sha256: [5; 32],
            custody_transfer_source_proof: vec![6],
            provider_operation_id: "mail-delivery-intent-01010101010101010101010101010101"
                .to_owned(),
        }
    }
}
