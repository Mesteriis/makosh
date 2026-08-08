use makosh_attachment_preview_evidence_replay_api::wire::{
    AttachmentPreviewEvidenceReplayErrorV1, AttachmentPreviewEvidenceReplayStateV1,
};
use makosh_attachment_preview_evidence_replay_core::{
    AuthenticatedReplayOperationRequestV1, ReplayFailureV1, ReplayProducerOutcomeV1,
    ReplayProducerResultV1, ReplayProducerV1,
};
use makosh_communications_retained_evidence_replay_contract::{
    COMMUNICATIONS_REPLAY_CAPABILITY_ID_V1, COMMUNICATIONS_REPLAY_SOURCE_MODULE_ID_V1,
    COMMUNICATIONS_REPLAY_TARGET_MODULE_ID_V1, communications_replay_command_contract_reference_v1,
    communications_replay_result_contract_reference_v1, validate_communications_replay_command_v1,
    validate_communications_replay_result_v1,
    wire::{
        ReplayCommunicationsEvidenceCommandV1, ReplayCommunicationsEvidenceFailureV1,
        ReplayCommunicationsEvidenceOutcomeV1, ReplayCommunicationsEvidenceResultV1,
    },
};
use makosh_events_protocol::{
    v1::{ActorKindV1, ContractRefV1, DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_retained_evidence_replay_contract::{
    MAIL_REPLAY_CAPABILITY_ID_V1, MAIL_REPLAY_SOURCE_MODULE_ID_V1, MAIL_REPLAY_TARGET_MODULE_ID_V1,
    mail_replay_command_contract_reference_v1, mail_replay_result_contract_reference_v1,
    validate_mail_replay_command_v1, validate_mail_replay_result_v1,
    wire::{
        ReplayMailEvidenceCommandV1, ReplayMailEvidenceFailureV1, ReplayMailEvidenceOutcomeV1,
        ReplayMailEvidenceResultV1,
    },
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::ReplayPersistenceErrorV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedReplayOperationV1 {
    pub request: AuthenticatedReplayOperationRequestV1,
    pub state: AttachmentPreviewEvidenceReplayStateV1,
    pub error: AttachmentPreviewEvidenceReplayErrorV1,
    pub state_revision: u64,
    pub accepted_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
}

pub(crate) fn request_fingerprint_v1(request: &AuthenticatedReplayOperationRequestV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.attachment-preview-evidence-replay.request.v1\0");
    hash.update(request.attachment_anchor_id);
    update_text(&mut hash, &request.logical_owner_id);
    hash.update(request.owner_device_actor_sha256);
    hash.finalize().into()
}

pub(crate) fn decode_command_v1(
    producer: ReplayProducerV1,
    exact_bytes: &[u8],
    request: &AuthenticatedReplayOperationRequestV1,
) -> Result<[u8; 16], ReplayPersistenceErrorV1> {
    let envelope = decode_envelope_v1(exact_bytes).map_err(|_| wrong_contract())?;
    let (expected_contract, source_module, capability) = match producer {
        ReplayProducerV1::Communications => (
            communications_replay_command_contract_reference_v1(),
            COMMUNICATIONS_REPLAY_SOURCE_MODULE_ID_V1,
            COMMUNICATIONS_REPLAY_CAPABILITY_ID_V1,
        ),
        ReplayProducerV1::Mail => (
            mail_replay_command_contract_reference_v1(),
            MAIL_REPLAY_SOURCE_MODULE_ID_V1,
            MAIL_REPLAY_CAPABILITY_ID_V1,
        ),
    };
    validate_command_envelope(
        &envelope,
        request,
        &expected_contract,
        source_module,
        capability,
    )?;
    match producer {
        ReplayProducerV1::Communications => {
            let command =
                ReplayCommunicationsEvidenceCommandV1::decode(envelope.payload.as_slice())
                    .map_err(|_| wrong_contract())?;
            validate_communications_replay_command_v1(&command).map_err(|_| wrong_contract())?;
            exact_command_payload(
                &command.operation_id,
                &command.logical_owner_id,
                &command.owner_device_actor_sha256,
                &command.attachment_anchor_id,
                request,
            )?;
        }
        ReplayProducerV1::Mail => {
            let command = ReplayMailEvidenceCommandV1::decode(envelope.payload.as_slice())
                .map_err(|_| wrong_contract())?;
            validate_mail_replay_command_v1(&command).map_err(|_| wrong_contract())?;
            exact_command_payload(
                &command.operation_id,
                &command.logical_owner_id,
                &command.owner_device_actor_sha256,
                &command.attachment_anchor_id,
                request,
            )?;
        }
    }
    id16(&envelope.message_id)
}

pub(crate) fn decode_result_v1(
    producer: ReplayProducerV1,
    exact_bytes: &[u8],
    expected_operation_id: [u8; 16],
    expected_command_message_id: [u8; 16],
) -> Result<ReplayProducerResultV1, ReplayPersistenceErrorV1> {
    let envelope = decode_envelope_v1(exact_bytes).map_err(|_| wrong_contract())?;
    let (expected_contract, source_module) = match producer {
        ReplayProducerV1::Communications => (
            communications_replay_result_contract_reference_v1(),
            COMMUNICATIONS_REPLAY_TARGET_MODULE_ID_V1,
        ),
        ReplayProducerV1::Mail => (
            mail_replay_result_contract_reference_v1(),
            MAIL_REPLAY_TARGET_MODULE_ID_V1,
        ),
    };
    validate_result_envelope(
        &envelope,
        expected_operation_id,
        expected_command_message_id,
        &expected_contract,
        source_module,
    )?;
    match producer {
        ReplayProducerV1::Communications => {
            let result = ReplayCommunicationsEvidenceResultV1::decode(envelope.payload.as_slice())
                .map_err(|_| wrong_contract())?;
            validate_communications_replay_result_v1(&result).map_err(|_| wrong_contract())?;
            if id16(&result.operation_id)? != expected_operation_id {
                return Err(wrong_contract());
            }
            Ok(ReplayProducerResultV1 {
                producer,
                original_message_ids: ids16(&result.original_message_ids)?,
                outcome: communications_outcome(result.outcome)?,
                failure: communications_failure(result.failure)?,
            })
        }
        ReplayProducerV1::Mail => {
            let result = ReplayMailEvidenceResultV1::decode(envelope.payload.as_slice())
                .map_err(|_| wrong_contract())?;
            validate_mail_replay_result_v1(&result).map_err(|_| wrong_contract())?;
            if id16(&result.operation_id)? != expected_operation_id {
                return Err(wrong_contract());
            }
            Ok(ReplayProducerResultV1 {
                producer,
                original_message_ids: ids16(&result.original_message_ids)?,
                outcome: mail_outcome(result.outcome)?,
                failure: mail_failure(result.failure)?,
            })
        }
    }
}

fn validate_command_envelope(
    envelope: &DurableEnvelopeV1,
    request: &AuthenticatedReplayOperationRequestV1,
    expected_contract: &ContractReferenceV1,
    source_module: &str,
    capability: &str,
) -> Result<(), ReplayPersistenceErrorV1> {
    let exact = contract_matches(envelope.contract.as_ref(), expected_contract)
        && envelope
            .source
            .as_ref()
            .is_some_and(|source| source.module_id == source_module)
        && envelope.partition_key == request.operation_id
        && envelope.correlation_id == request.operation_id
        && envelope.causation_message_id.is_empty()
        && envelope.actor.as_ref().is_some_and(|actor| {
            actor.kind == ActorKindV1::OwnerDevice as i32
                && actor.actor_id == request.owner_device_actor_sha256
        })
        && matches!(
            envelope.semantics.as_ref(),
            Some(Semantics::Command(command))
                if command.command_id == request.operation_id
                    && command.target_capability == capability
        );
    exact.then_some(()).ok_or_else(wrong_contract)
}

fn validate_result_envelope(
    envelope: &DurableEnvelopeV1,
    operation_id: [u8; 16],
    command_message_id: [u8; 16],
    expected_contract: &ContractReferenceV1,
    source_module: &str,
) -> Result<(), ReplayPersistenceErrorV1> {
    let exact = contract_matches(envelope.contract.as_ref(), expected_contract)
        && envelope
            .source
            .as_ref()
            .is_some_and(|source| source.module_id == source_module)
        && envelope.partition_key == operation_id
        && envelope.correlation_id == operation_id
        && envelope.causation_message_id == command_message_id
        && envelope.actor.as_ref().is_some_and(|actor| {
            actor.kind == ActorKindV1::Module as i32 && actor.actor_id == source_module.as_bytes()
        })
        && matches!(
            envelope.semantics.as_ref(),
            Some(Semantics::Result(result))
                if result.command_id == operation_id
                    && result.command_message_id == command_message_id
        );
    exact.then_some(()).ok_or_else(wrong_contract)
}

fn exact_command_payload(
    operation_id: &[u8],
    logical_owner_id: &str,
    owner_device_actor_sha256: &[u8],
    attachment_anchor_id: &[u8],
    request: &AuthenticatedReplayOperationRequestV1,
) -> Result<(), ReplayPersistenceErrorV1> {
    let exact = operation_id == request.operation_id
        && logical_owner_id == request.logical_owner_id
        && owner_device_actor_sha256 == request.owner_device_actor_sha256
        && attachment_anchor_id == request.attachment_anchor_id;
    exact.then_some(()).ok_or_else(wrong_contract)
}

fn contract_matches(actual: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner
            && actual.name == expected.name
            && actual.major == expected.major
            && actual.revision == expected.revision
            && actual.schema_sha256 == expected.schema_sha256
    })
}

fn update_text(hash: &mut Sha256, value: &str) {
    hash.update((value.len() as u64).to_be_bytes());
    hash.update(value.as_bytes());
}

pub(crate) fn id16(value: &[u8]) -> Result<[u8; 16], ReplayPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReplayPersistenceErrorV1::InvalidRow)
}

pub(crate) fn id32(value: &[u8]) -> Result<[u8; 32], ReplayPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReplayPersistenceErrorV1::InvalidRow)
}

fn ids16(values: &[Vec<u8>]) -> Result<Vec<[u8; 16]>, ReplayPersistenceErrorV1> {
    values.iter().map(|value| id16(value)).collect()
}

fn communications_outcome(value: i32) -> Result<ReplayProducerOutcomeV1, ReplayPersistenceErrorV1> {
    match ReplayCommunicationsEvidenceOutcomeV1::try_from(value).map_err(|_| wrong_contract())? {
        ReplayCommunicationsEvidenceOutcomeV1::Published => Ok(ReplayProducerOutcomeV1::Published),
        ReplayCommunicationsEvidenceOutcomeV1::AlreadyPublished => {
            Ok(ReplayProducerOutcomeV1::AlreadyPublished)
        }
        ReplayCommunicationsEvidenceOutcomeV1::Rejected => Ok(ReplayProducerOutcomeV1::Rejected),
        ReplayCommunicationsEvidenceOutcomeV1::Unavailable => {
            Ok(ReplayProducerOutcomeV1::Unavailable)
        }
        ReplayCommunicationsEvidenceOutcomeV1::Unspecified => Err(wrong_contract()),
    }
}

fn mail_outcome(value: i32) -> Result<ReplayProducerOutcomeV1, ReplayPersistenceErrorV1> {
    match ReplayMailEvidenceOutcomeV1::try_from(value).map_err(|_| wrong_contract())? {
        ReplayMailEvidenceOutcomeV1::Published => Ok(ReplayProducerOutcomeV1::Published),
        ReplayMailEvidenceOutcomeV1::AlreadyPublished => {
            Ok(ReplayProducerOutcomeV1::AlreadyPublished)
        }
        ReplayMailEvidenceOutcomeV1::Rejected => Ok(ReplayProducerOutcomeV1::Rejected),
        ReplayMailEvidenceOutcomeV1::Unavailable => Ok(ReplayProducerOutcomeV1::Unavailable),
        ReplayMailEvidenceOutcomeV1::Unspecified => Err(wrong_contract()),
    }
}

fn communications_failure(value: i32) -> Result<ReplayFailureV1, ReplayPersistenceErrorV1> {
    let value =
        ReplayCommunicationsEvidenceFailureV1::try_from(value).map_err(|_| wrong_contract())?;
    Ok(match value {
        ReplayCommunicationsEvidenceFailureV1::Unspecified => ReplayFailureV1::None,
        ReplayCommunicationsEvidenceFailureV1::NotFound => ReplayFailureV1::NotFound,
        ReplayCommunicationsEvidenceFailureV1::HashMismatch => ReplayFailureV1::HashMismatch,
        ReplayCommunicationsEvidenceFailureV1::WrongContract => ReplayFailureV1::WrongContract,
        ReplayCommunicationsEvidenceFailureV1::StaleRuntimeFence => {
            ReplayFailureV1::StaleRuntimeFence
        }
        ReplayCommunicationsEvidenceFailureV1::StaleGrantFence => ReplayFailureV1::StaleGrantFence,
        ReplayCommunicationsEvidenceFailureV1::OwnerMismatch => ReplayFailureV1::OwnerMismatch,
        ReplayCommunicationsEvidenceFailureV1::PublishUnavailable => {
            ReplayFailureV1::PublishUnavailable
        }
    })
}

fn mail_failure(value: i32) -> Result<ReplayFailureV1, ReplayPersistenceErrorV1> {
    let value = ReplayMailEvidenceFailureV1::try_from(value).map_err(|_| wrong_contract())?;
    Ok(match value {
        ReplayMailEvidenceFailureV1::Unspecified => ReplayFailureV1::None,
        ReplayMailEvidenceFailureV1::NotFound => ReplayFailureV1::NotFound,
        ReplayMailEvidenceFailureV1::HashMismatch => ReplayFailureV1::HashMismatch,
        ReplayMailEvidenceFailureV1::WrongContract => ReplayFailureV1::WrongContract,
        ReplayMailEvidenceFailureV1::StaleRuntimeFence => ReplayFailureV1::StaleRuntimeFence,
        ReplayMailEvidenceFailureV1::StaleGrantFence => ReplayFailureV1::StaleGrantFence,
        ReplayMailEvidenceFailureV1::OwnerMismatch => ReplayFailureV1::OwnerMismatch,
        ReplayMailEvidenceFailureV1::PublishUnavailable => ReplayFailureV1::PublishUnavailable,
    })
}

fn wrong_contract() -> ReplayPersistenceErrorV1 {
    ReplayPersistenceErrorV1::WrongContract
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_covers_only_the_authenticated_provider_neutral_request() {
        let request = request();
        let baseline = request_fingerprint_v1(&request);
        for changed in [
            mutate(&request, |value| {
                value.logical_owner_id = "owner-2".to_owned()
            }),
            mutate(&request, |value| value.owner_device_actor_sha256 = [8; 32]),
            mutate(&request, |value| value.attachment_anchor_id = [7; 16]),
        ] {
            assert_ne!(baseline, request_fingerprint_v1(&changed));
        }
    }

    fn request() -> AuthenticatedReplayOperationRequestV1 {
        AuthenticatedReplayOperationRequestV1 {
            operation_id: [1; 16],
            attachment_anchor_id: [2; 16],
            logical_owner_id: "owner-1".to_owned(),
            owner_device_actor_sha256: [9; 32],
        }
    }

    fn mutate(
        request: &AuthenticatedReplayOperationRequestV1,
        apply: impl FnOnce(&mut AuthenticatedReplayOperationRequestV1),
    ) -> AuthenticatedReplayOperationRequestV1 {
        let mut request = request.clone();
        apply(&mut request);
        request
    }
}
