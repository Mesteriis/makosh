use makosh_communication_delivery_intent_api::COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1;
use makosh_communication_delivery_intent_core::{
    CommunicationConversationIdV1, CommunicationMessageIdV1, DeliveryIntentDraftV1,
    DeliveryIntentPlanErrorV1, plan_delivery_intent_v1,
};
use makosh_communication_delivery_intent_ingress_api::{
    COMMUNICATION_DELIVERY_INTENT_INGRESS_COMMAND_CAPABILITY_ID_V1,
    CommunicationDeliveryIntentIngressEnvelopeContextV1,
    build_communication_delivery_intent_rejected_outbox_record_v1,
    build_communication_delivery_intent_submitted_outbox_record_v1,
    communication_delivery_intent_submit_contract_reference_v1,
    communication_delivery_intent_submit_message_id_v1,
    wire::{
        CommunicationDeliveryIntentIngressRejectCodeV1, CommunicationDeliveryIntentRejectedV1,
        CommunicationDeliveryIntentSubmittedV1, SubmitCommunicationDeliveryIntentCommandV1,
    },
};
use makosh_communication_delivery_intent_persistence::{
    CreateDeliveryIntentV1, DeliveryIntentIngressBlobReceiptV1, DeliveryIntentIngressDispositionV1,
    DeliveryIntentIngressEventV1,
};
use makosh_events_jetstream::{
    RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1, receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, ContractRefV1, FenceKindV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::{
    managed_control::RejectManagedControlRequestsV2, v1::ContractReferenceV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    body_materializer::ManagedDeliveryIntentBodyMaterializerV1,
    communications_query_client::ManagedCommunicationsQueryClientV1,
    coordinator::{DeliveryIntentCoordinatorErrorV1, prepare_create_delivery_intent_v1},
    ingress_body::{DeliveryIntentIngressBodyErrorV1, read_delivery_intent_ingress_body_v1},
    runtime::{DeliveryIntentManagedRuntimeV1, DeliveryIntentRuntimeErrorV1},
};

const CROSS_CHANNEL_FORWARD_RUNTIME_MODULE_ID_V1: &str =
    "makosh-communication-cross-channel-forward-runtime";

#[derive(Clone, Debug)]
struct DecodedDeliveryIntentIngressV1 {
    event: DeliveryIntentIngressEventV1,
    target_conversation_id: [u8; 16],
    target_reply_to_message_id: Option<[u8; 16]>,
    body_source: DeliveryIntentIngressBlobReceiptV1,
    deadline_expired: bool,
}

enum PreparedDeliveryIntentIngressV1 {
    Admit(Box<CreateDeliveryIntentV1>),
    Reject(CommunicationDeliveryIntentIngressRejectCodeV1),
}

pub(crate) fn bind_delivery_intent_ingress_subscription(
    permits: &mut Vec<RuntimeSubscribePermitV1>,
) -> Result<RuntimeSubscribePermitV1, DeliveryIntentRuntimeErrorV1> {
    let expected = communication_delivery_intent_submit_contract_reference_v1();
    let matching = permits
        .iter()
        .enumerate()
        .filter(|(_, permit)| {
            permit
                .contract()
                .is_some_and(|actual| exact_permit_contract(actual, &expected))
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(DeliveryIntentRuntimeErrorV1::Admission);
    }
    Ok(permits.remove(matching[0]))
}

impl DeliveryIntentManagedRuntimeV1 {
    pub async fn consume_next_event_ingress_v1(
        &mut self,
        now_unix_seconds: i64,
    ) -> Result<bool, DeliveryIntentRuntimeErrorV1> {
        if now_unix_seconds <= 0 {
            return Err(DeliveryIntentRuntimeErrorV1::InvalidRequest);
        }
        let delivery =
            receive_runtime_pull_delivery(&self.event_connection, &self.event_ingress_subscription)
                .await
                .map_err(event_error)?;
        let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
            .map_err(|_| DeliveryIntentRuntimeErrorV1::EventContract)?;
        let mut decoded = decode_event_ingress(&record, &self.logical_owner_id, now_unix_seconds)?;
        match self
            .persistence
            .inspect_event_ingress(&decoded.event)
            .await
            .map_err(DeliveryIntentRuntimeErrorV1::Persistence)?
        {
            DeliveryIntentIngressDispositionV1::ExactDuplicate => {
                delivery.acknowledge().await.map_err(event_error)?;
                return Ok(true);
            }
            DeliveryIntentIngressDispositionV1::New => {}
        }

        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let prepared = self.prepare_event_ingress(&mut decoded);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DeliveryIntentRuntimeErrorV1::Unavailable)?;
        let prepared = prepared?;
        let result_context = CommunicationDeliveryIntentIngressEnvelopeContextV1 {
            module_id: COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1.to_owned(),
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            recorded_at_unix_seconds: now_unix_seconds,
            recorded_at_nanos: 0,
        };
        match prepared {
            PreparedDeliveryIntentIngressV1::Admit(command) => {
                let result = build_communication_delivery_intent_submitted_outbox_record_v1(
                    decoded.event.command_message_id,
                    CommunicationDeliveryIntentSubmittedV1 {
                        intent_id: decoded.event.intent_id.to_vec(),
                        logical_owner_id: decoded.event.logical_owner_id.clone(),
                    },
                    &result_context,
                )
                .map_err(|_| DeliveryIntentRuntimeErrorV1::EventContract)?;
                self.persistence
                    .admit_event_ingress(&decoded.event, &command, &result)
                    .await
                    .map_err(DeliveryIntentRuntimeErrorV1::Persistence)?;
            }
            PreparedDeliveryIntentIngressV1::Reject(code) => {
                let result = build_communication_delivery_intent_rejected_outbox_record_v1(
                    decoded.event.command_message_id,
                    CommunicationDeliveryIntentRejectedV1 {
                        intent_id: decoded.event.intent_id.to_vec(),
                        code: code as i32,
                        logical_owner_id: decoded.event.logical_owner_id.clone(),
                    },
                    &result_context,
                )
                .map_err(|_| DeliveryIntentRuntimeErrorV1::EventContract)?;
                self.persistence
                    .reject_event_ingress(&decoded.event, &result)
                    .await
                    .map_err(DeliveryIntentRuntimeErrorV1::Persistence)?;
            }
        }
        delivery.acknowledge().await.map_err(event_error)?;
        Ok(true)
    }

    fn prepare_event_ingress(
        &mut self,
        decoded: &mut DecodedDeliveryIntentIngressV1,
    ) -> Result<PreparedDeliveryIntentIngressV1, DeliveryIntentRuntimeErrorV1> {
        if decoded.deadline_expired {
            return Ok(PreparedDeliveryIntentIngressV1::Reject(
                CommunicationDeliveryIntentIngressRejectCodeV1::
                    CommunicationDeliveryIntentIngressRejectCodePolicy,
            ));
        }
        let mut dispatcher = RejectManagedControlRequestsV2;
        let (body, body_receipt) = match read_delivery_intent_ingress_body_v1(
            &mut self.control_channel,
            &mut dispatcher,
            &decoded.body_source,
            &decoded.event.command_message_id,
            &decoded.event.envelope_sha256,
        ) {
            Ok(body) => body,
            Err(DeliveryIntentIngressBodyErrorV1::InvalidReceipt) => {
                return Ok(PreparedDeliveryIntentIngressV1::Reject(
                    CommunicationDeliveryIntentIngressRejectCodeV1::
                        CommunicationDeliveryIntentIngressRejectCodeCustodyInvalid,
                ));
            }
            Err(DeliveryIntentIngressBodyErrorV1::Unavailable) => {
                return Err(DeliveryIntentRuntimeErrorV1::Unavailable);
            }
        };
        decoded.event.body_receipt = body_receipt;
        let conversation_id = CommunicationConversationIdV1::new(decoded.target_conversation_id);
        let reply_to_message_id = decoded
            .target_reply_to_message_id
            .map(CommunicationMessageIdV1::new);
        let (conversation, reply) = {
            let mut query_client = ManagedCommunicationsQueryClientV1 {
                control_channel: &mut self.control_channel,
                dispatcher: &mut dispatcher,
            };
            query_client
                .resolve_route_sources(
                    decoded.event.intent_id,
                    conversation_id,
                    reply_to_message_id,
                )
                .map_err(|_| DeliveryIntentRuntimeErrorV1::RouteUnavailable)?
        };
        let planned = match plan_delivery_intent_v1(
            DeliveryIntentDraftV1 {
                operation_id: decoded.event.intent_id,
                conversation_id,
                reply_to_message_id,
                body_utf8: body.to_vec(),
            },
            &conversation,
            reply.as_ref(),
        ) {
            Ok(planned) => planned,
            Err(DeliveryIntentPlanErrorV1::InvalidBody)
            | Err(DeliveryIntentPlanErrorV1::BodyLimitExceeded)
            | Err(DeliveryIntentPlanErrorV1::InvalidOperationId) => {
                return Ok(PreparedDeliveryIntentIngressV1::Reject(
                    CommunicationDeliveryIntentIngressRejectCodeV1::
                        CommunicationDeliveryIntentIngressRejectCodeInvalidRequest,
                ));
            }
            Err(_) => {
                return Ok(PreparedDeliveryIntentIngressV1::Reject(
                    CommunicationDeliveryIntentIngressRejectCodeV1::
                        CommunicationDeliveryIntentIngressRejectCodeCanonicalTargetUnavailable,
                ));
            }
        };
        let command = {
            let mut materializer = ManagedDeliveryIntentBodyMaterializerV1 {
                control_channel: &mut self.control_channel,
                dispatcher: &mut dispatcher,
            };
            prepare_create_delivery_intent_v1(
                decoded.event.logical_owner_id.clone(),
                planned,
                decoded.event.consumed_at_unix_seconds,
                &mut materializer,
            )
        };
        match command {
            Ok(command) => Ok(PreparedDeliveryIntentIngressV1::Admit(Box::new(command))),
            Err(DeliveryIntentCoordinatorErrorV1::InvalidInput) => {
                Ok(PreparedDeliveryIntentIngressV1::Reject(
                    CommunicationDeliveryIntentIngressRejectCodeV1::
                        CommunicationDeliveryIntentIngressRejectCodeInvalidRequest,
                ))
            }
            Err(DeliveryIntentCoordinatorErrorV1::BlobUnavailable) => {
                Err(DeliveryIntentRuntimeErrorV1::Unavailable)
            }
        }
    }
}

fn decode_event_ingress(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_seconds: i64,
) -> Result<DecodedDeliveryIntentIngressV1, DeliveryIntentRuntimeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| DeliveryIntentRuntimeErrorV1::EventContract)?;
    let expected_contract = communication_delivery_intent_submit_contract_reference_v1();
    let source = envelope
        .source
        .as_ref()
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)?;
    let recorded_at = envelope
        .recorded_at
        .as_ref()
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)?;
    let Some(Semantics::Command(command)) = envelope.semantics.as_ref() else {
        return Err(DeliveryIntentRuntimeErrorV1::EventContract);
    };
    let intent_id = id16(&command.command_id)?;
    let expected_idempotency = Sha256::digest(
        [
            b"communication-delivery-intent-event-ingress-v1".as_slice(),
            &intent_id,
        ]
        .concat(),
    );
    let deadline = command
        .deadline
        .as_ref()
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)?;
    if !exact_contract(envelope.contract.as_ref(), &expected_contract)
        || source.module_id != CROSS_CHANNEL_FORWARD_RUNTIME_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_instance_id.iter().all(|byte| *byte == 0)
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id.as_slice() != CROSS_CHANNEL_FORWARD_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id.as_slice() != CROSS_CHANNEL_FORWARD_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || recorded_at.seconds <= 0
        || !(0..1_000_000_000).contains(&recorded_at.nanos)
        || command.target_capability
            != COMMUNICATION_DELIVERY_INTENT_INGRESS_COMMAND_CAPABILITY_ID_V1
        || command.idempotency_key.as_slice() != expected_idempotency.as_slice()
        || command.logical_attempt != 1
        || deadline.nanos != 0
        || deadline.seconds <= recorded_at.seconds
        || envelope.message_id.as_slice()
            != communication_delivery_intent_submit_message_id_v1(&intent_id)
        || envelope.partition_key.as_slice() != intent_id
        || envelope.correlation_id.as_slice() != intent_id
        || !envelope.causation_message_id.is_empty()
    {
        return Err(DeliveryIntentRuntimeErrorV1::EventContract);
    }
    let payload = SubmitCommunicationDeliveryIntentCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| DeliveryIntentRuntimeErrorV1::EventContract)?;
    if payload.intent_id.as_slice() != intent_id
        || payload.logical_owner_id != expected_logical_owner_id
    {
        return Err(DeliveryIntentRuntimeErrorV1::EventContract);
    }
    let target_conversation_id = id16(&payload.target_conversation_id)?;
    let target_reply_to_message_id = if payload.target_reply_to_message_id.is_empty() {
        None
    } else {
        Some(id16(&payload.target_reply_to_message_id)?)
    };
    let body_source = payload
        .body_source
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)?;
    let body_source = DeliveryIntentIngressBlobReceiptV1 {
        reference_id: id16(&body_source.reference_id)?,
        declared_bytes: body_source.declared_bytes,
        sha256: id32(&body_source.sha256)?,
        custody_source_proof: body_source.custody_transfer_source_proof,
    };
    Ok(DecodedDeliveryIntentIngressV1 {
        event: DeliveryIntentIngressEventV1 {
            command_message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            correlation_id: intent_id,
            logical_owner_id: payload.logical_owner_id,
            intent_id,
            body_receipt: body_source.clone(),
            consumed_at_unix_seconds,
        },
        target_conversation_id,
        target_reply_to_message_id,
        body_source,
        deadline_expired: deadline.seconds <= consumed_at_unix_seconds,
    })
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

fn exact_permit_contract(actual: &ContractReferenceV1, expected: &ContractReferenceV1) -> bool {
    actual.owner == expected.owner
        && actual.name == expected.name
        && actual.major == expected.major
        && actual.revision == expected.revision
        && actual.schema_sha256 == expected.schema_sha256
}

fn id16(value: &[u8]) -> Result<[u8; 16], DeliveryIntentRuntimeErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)
}

fn id32(value: &[u8]) -> Result<[u8; 32], DeliveryIntentRuntimeErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 32]| id.iter().any(|byte| *byte != 0))
        .ok_or(DeliveryIntentRuntimeErrorV1::EventContract)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> DeliveryIntentRuntimeErrorV1 {
    DeliveryIntentRuntimeErrorV1::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communication_delivery_intent_ingress_api::{
        build_communication_delivery_intent_submit_outbox_record_v1,
        wire::DeliveryIntentBodySourceReceiptV1,
    };

    fn submit_record(module_id: &str) -> OutboxRecordV1 {
        build_communication_delivery_intent_submit_outbox_record_v1(
            [1; 16],
            [2; 16],
            None,
            DeliveryIntentBodySourceReceiptV1 {
                reference_id: vec![3; 16],
                declared_bytes: 42,
                sha256: vec![4; 32],
                custody_transfer_source_proof: vec![5; 64],
            },
            "owner-1",
            1_800_000_300,
            &CommunicationDeliveryIntentIngressEnvelopeContextV1 {
                module_id: module_id.to_owned(),
                runtime_instance_id: "cross-runtime-1".to_owned(),
                runtime_generation: 7,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("record")
    }

    #[test]
    fn ingress_requires_exact_cross_channel_source_and_owner() {
        let decoded = decode_event_ingress(
            &submit_record(CROSS_CHANNEL_FORWARD_RUNTIME_MODULE_ID_V1),
            "owner-1",
            1_800_000_001,
        )
        .expect("exact ingress");
        assert_eq!(decoded.event.intent_id, [1; 16]);
        assert_eq!(decoded.target_conversation_id, [2; 16]);
        assert!(!decoded.deadline_expired);
        assert!(
            decode_event_ingress(
                &submit_record(CROSS_CHANNEL_FORWARD_RUNTIME_MODULE_ID_V1),
                "owner-1",
                1_800_000_300
            )
            .expect("expired command remains rejectable")
            .deadline_expired
        );
        assert!(
            decode_event_ingress(
                &submit_record("makosh-communications-runtime"),
                "owner-1",
                1_800_000_001
            )
            .is_err()
        );
        assert!(
            decode_event_ingress(
                &submit_record(CROSS_CHANNEL_FORWARD_RUNTIME_MODULE_ID_V1),
                "owner-2",
                1_800_000_001
            )
            .is_err()
        );
    }
}
