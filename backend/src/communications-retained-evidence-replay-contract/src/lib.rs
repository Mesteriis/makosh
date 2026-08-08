#![forbid(unsafe_code)]

use std::collections::HashSet;

mod envelope;

pub use envelope::{
    CommunicationsReplayCommandEnvelopeContextV1, CommunicationsReplayCommandEnvelopeErrorV1,
    CommunicationsReplayResultEnvelopeContextV1, CommunicationsReplayResultEnvelopeErrorV1,
    build_communications_replay_command_outbox_v1, build_communications_replay_result_outbox_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.replay.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/communications_replay_schema.rs"));

pub const COMMUNICATIONS_REPLAY_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/communications-replay-v1.bin"));

pub const PACKAGE: &str = "makosh-communications-retained-evidence-replay-contract";
pub const COMMUNICATIONS_REPLAY_OWNER_ID_V1: &str = "communications";
pub const COMMUNICATIONS_REPLAY_SOURCE_MODULE_ID_V1: &str =
    "makosh-attachment-preview-evidence-replay-runtime";
pub const COMMUNICATIONS_REPLAY_TARGET_MODULE_ID_V1: &str = "makosh-communications-runtime";
pub const COMMUNICATIONS_REPLAY_CAPABILITY_ID_V1: &str =
    "communications.retained-evidence-replay.v1";
pub const COMMUNICATIONS_REPLAY_COMMAND_CONTRACT_NAME_V1: &str =
    "communications_retained_evidence_replay_command";
pub const COMMUNICATIONS_REPLAY_RESULT_CONTRACT_NAME_V1: &str =
    "communications_retained_evidence_replay_result";
pub const COMMUNICATIONS_REPLAY_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATIONS_REPLAY_CONTRACT_REVISION_V1: u32 = 2;
pub const COMMUNICATIONS_REPLAY_MAX_IN_FLIGHT_V1: u32 = 8;
pub const COMMUNICATIONS_REPLAY_MAX_MESSAGES_V1: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsReplayValidationErrorV1 {
    InvalidOperation,
    InvalidOwner,
    InvalidActor,
    InvalidAttachmentAnchor,
    InvalidMessageSelection,
    InvalidResult,
}

pub fn validate_communications_replay_command_v1(
    command: &wire::ReplayCommunicationsEvidenceCommandV1,
) -> Result<(), CommunicationsReplayValidationErrorV1> {
    validate_id16(&command.operation_id)
        .map_err(|_| CommunicationsReplayValidationErrorV1::InvalidOperation)?;
    if !valid_identity(&command.logical_owner_id) {
        return Err(CommunicationsReplayValidationErrorV1::InvalidOwner);
    }
    if !valid_sha256(&command.owner_device_actor_sha256) {
        return Err(CommunicationsReplayValidationErrorV1::InvalidActor);
    }
    validate_id16(&command.attachment_anchor_id)
        .map_err(|_| CommunicationsReplayValidationErrorV1::InvalidAttachmentAnchor)
}

pub fn validate_communications_replay_result_v1(
    result: &wire::ReplayCommunicationsEvidenceResultV1,
) -> Result<(), CommunicationsReplayValidationErrorV1> {
    validate_id16(&result.operation_id)
        .map_err(|_| CommunicationsReplayValidationErrorV1::InvalidOperation)?;
    use wire::{
        ReplayCommunicationsEvidenceFailureV1 as Failure,
        ReplayCommunicationsEvidenceOutcomeV1 as Outcome,
    };
    let outcome = Outcome::try_from(result.outcome)
        .map_err(|_| CommunicationsReplayValidationErrorV1::InvalidResult)?;
    let failure = Failure::try_from(result.failure)
        .map_err(|_| CommunicationsReplayValidationErrorV1::InvalidResult)?;
    match outcome {
        Outcome::Published | Outcome::AlreadyPublished if failure == Failure::Unspecified => {
            validate_message_ids(&result.original_message_ids)
        }
        Outcome::Rejected | Outcome::Unavailable if failure != Failure::Unspecified => {
            validate_optional_message_ids(&result.original_message_ids)
        }
        _ => Err(CommunicationsReplayValidationErrorV1::InvalidResult),
    }
}

fn validate_message_ids(ids: &[Vec<u8>]) -> Result<(), CommunicationsReplayValidationErrorV1> {
    if ids.is_empty() {
        return Err(CommunicationsReplayValidationErrorV1::InvalidMessageSelection);
    }
    validate_optional_message_ids(ids)
}

fn validate_optional_message_ids(
    ids: &[Vec<u8>],
) -> Result<(), CommunicationsReplayValidationErrorV1> {
    if ids.len() > COMMUNICATIONS_REPLAY_MAX_MESSAGES_V1 {
        return Err(CommunicationsReplayValidationErrorV1::InvalidMessageSelection);
    }
    let mut unique = HashSet::with_capacity(ids.len());
    for id in ids {
        if validate_id16(id).is_err() || !unique.insert(id.as_slice()) {
            return Err(CommunicationsReplayValidationErrorV1::InvalidMessageSelection);
        }
    }
    Ok(())
}

fn validate_id16(value: &[u8]) -> Result<(), ()> {
    (value.len() == 16 && value.iter().any(|byte| *byte != 0))
        .then_some(())
        .ok_or(())
}

fn valid_sha256(value: &[u8]) -> bool {
    value.len() == 32 && value.iter().any(|byte| *byte != 0)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

#[must_use]
pub fn communications_replay_command_contract_reference_v1() -> ContractReferenceV1 {
    contract(COMMUNICATIONS_REPLAY_COMMAND_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communications_replay_result_contract_reference_v1() -> ContractReferenceV1 {
    contract(COMMUNICATIONS_REPLAY_RESULT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communications_replay_command_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Command,
        communications_replay_command_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communications_replay_command_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Command,
        communications_replay_command_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communications_replay_result_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        communications_replay_result_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communications_replay_result_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        communications_replay_result_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_REPLAY_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATIONS_REPLAY_CONTRACT_MAJOR_V1,
        revision: COMMUNICATIONS_REPLAY_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_REPLAY_SCHEMA_SHA256_V1.to_vec(),
    }
}

fn route(
    kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: COMMUNICATIONS_REPLAY_MAX_IN_FLIGHT_V1,
            subscription_requirement: requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_and_result_are_exact_communications_routes() {
        let Some(Request::EventRoute(command)) =
            communications_replay_command_consume_request_v1().request
        else {
            panic!("command route");
        };
        let Some(Request::EventRoute(result)) =
            communications_replay_result_publish_request_v1().request
        else {
            panic!("result route");
        };
        assert_eq!(command.envelope_kind, DurableEnvelopeKindV1::Command as i32);
        assert_eq!(result.envelope_kind, DurableEnvelopeKindV1::Result as i32);
        assert_eq!(command.contract.expect("contract").owner, "communications");
        assert_eq!(result.contract.expect("contract").owner, "communications");
    }
}
