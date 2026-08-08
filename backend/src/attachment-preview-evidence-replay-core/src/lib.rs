#![forbid(unsafe_code)]

use std::collections::HashSet;

use makosh_attachment_preview_evidence_replay_api::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAX_MESSAGES_PER_PRODUCER_V1,
    wire::{AttachmentPreviewEvidenceReplayErrorV1, AttachmentPreviewEvidenceReplayStateV1},
};

pub const PACKAGE: &str = "makosh-attachment-preview-evidence-replay-core";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum ReplayProducerV1 {
    Communications = 1,
    Mail = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedReplayOperationRequestV1 {
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub logical_owner_id: String,
    pub owner_device_actor_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayProducerCommandIntentV1 {
    pub operation_id: [u8; 16],
    pub logical_owner_id: String,
    pub owner_device_actor_sha256: [u8; 32],
    pub producer: ReplayProducerV1,
    pub attachment_anchor_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayOperationPlanV1 {
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub commands: [ReplayProducerCommandIntentV1; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayProducerOutcomeV1 {
    Published,
    AlreadyPublished,
    Rejected,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayFailureV1 {
    None,
    NotFound,
    HashMismatch,
    WrongContract,
    StaleRuntimeFence,
    StaleGrantFence,
    OwnerMismatch,
    PublishUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayProducerResultV1 {
    pub producer: ReplayProducerV1,
    pub original_message_ids: Vec<[u8; 16]>,
    pub outcome: ReplayProducerOutcomeV1,
    pub failure: ReplayFailureV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayOperationStateV1 {
    pub request: AuthenticatedReplayOperationRequestV1,
    pub communications_result: Option<ReplayProducerResultV1>,
    pub mail_result: Option<ReplayProducerResultV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayCoreErrorV1 {
    InvalidRequest,
    InvalidResult,
    ResultConflict,
}

pub fn plan_replay_operation_v1(
    request: AuthenticatedReplayOperationRequestV1,
) -> Result<ReplayOperationPlanV1, ReplayCoreErrorV1> {
    validate_request(&request)?;
    Ok(ReplayOperationPlanV1 {
        operation_id: request.operation_id,
        attachment_anchor_id: request.attachment_anchor_id,
        commands: [
            command_intent(&request, ReplayProducerV1::Communications),
            command_intent(&request, ReplayProducerV1::Mail),
        ],
    })
}

pub fn accepted_replay_operation_v1(
    request: AuthenticatedReplayOperationRequestV1,
) -> Result<ReplayOperationStateV1, ReplayCoreErrorV1> {
    validate_request(&request)?;
    Ok(ReplayOperationStateV1 {
        request,
        communications_result: None,
        mail_result: None,
    })
}

pub fn observe_producer_result_v1(
    state: &mut ReplayOperationStateV1,
    result: ReplayProducerResultV1,
) -> Result<(), ReplayCoreErrorV1> {
    validate_result(state, &result)?;
    let slot = match result.producer {
        ReplayProducerV1::Communications => &mut state.communications_result,
        ReplayProducerV1::Mail => &mut state.mail_result,
    };
    match slot {
        Some(existing) if existing != &result => Err(ReplayCoreErrorV1::ResultConflict),
        Some(_) => Ok(()),
        None => {
            *slot = Some(result);
            Ok(())
        }
    }
}

#[must_use]
pub fn replay_operation_status_v1(
    state: &ReplayOperationStateV1,
) -> (
    AttachmentPreviewEvidenceReplayStateV1,
    AttachmentPreviewEvidenceReplayErrorV1,
) {
    let results = [
        state.communications_result.as_ref(),
        state.mail_result.as_ref(),
    ];
    if results.iter().flatten().any(|result| {
        result.outcome == ReplayProducerOutcomeV1::Rejected
            && matches!(
                result.failure,
                ReplayFailureV1::StaleRuntimeFence
                    | ReplayFailureV1::StaleGrantFence
                    | ReplayFailureV1::OwnerMismatch
            )
    }) {
        return (
            AttachmentPreviewEvidenceReplayStateV1::Rejected,
            AttachmentPreviewEvidenceReplayErrorV1::StaleProducerFence,
        );
    }
    if results.iter().flatten().any(|result| {
        matches!(
            result.outcome,
            ReplayProducerOutcomeV1::Rejected | ReplayProducerOutcomeV1::Unavailable
        )
    }) {
        return (
            AttachmentPreviewEvidenceReplayStateV1::Unavailable,
            AttachmentPreviewEvidenceReplayErrorV1::ProducerUnavailable,
        );
    }
    if results.iter().all(Option::is_some) {
        return (
            AttachmentPreviewEvidenceReplayStateV1::Completed,
            AttachmentPreviewEvidenceReplayErrorV1::Unspecified,
        );
    }
    (
        AttachmentPreviewEvidenceReplayStateV1::AwaitingProducers,
        AttachmentPreviewEvidenceReplayErrorV1::Unspecified,
    )
}

fn command_intent(
    request: &AuthenticatedReplayOperationRequestV1,
    producer: ReplayProducerV1,
) -> ReplayProducerCommandIntentV1 {
    ReplayProducerCommandIntentV1 {
        operation_id: request.operation_id,
        logical_owner_id: request.logical_owner_id.clone(),
        owner_device_actor_sha256: request.owner_device_actor_sha256,
        producer,
        attachment_anchor_id: request.attachment_anchor_id,
    }
}

fn validate_request(
    request: &AuthenticatedReplayOperationRequestV1,
) -> Result<(), ReplayCoreErrorV1> {
    if zero(&request.operation_id)
        || zero(&request.attachment_anchor_id)
        || zero(&request.owner_device_actor_sha256)
        || !valid_identity(&request.logical_owner_id)
    {
        return Err(ReplayCoreErrorV1::InvalidRequest);
    }
    Ok(())
}

fn validate_result(
    _state: &ReplayOperationStateV1,
    result: &ReplayProducerResultV1,
) -> Result<(), ReplayCoreErrorV1> {
    let ids_valid = result.original_message_ids.len()
        <= ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MAX_MESSAGES_PER_PRODUCER_V1
        && result.original_message_ids.iter().all(|id| !zero(id))
        && result
            .original_message_ids
            .iter()
            .collect::<HashSet<_>>()
            .len()
            == result.original_message_ids.len();
    let outcome_valid = match result.outcome {
        ReplayProducerOutcomeV1::Published | ReplayProducerOutcomeV1::AlreadyPublished => {
            result.failure == ReplayFailureV1::None && !result.original_message_ids.is_empty()
        }
        ReplayProducerOutcomeV1::Rejected | ReplayProducerOutcomeV1::Unavailable => {
            result.failure != ReplayFailureV1::None
        }
    };
    if !ids_valid || !outcome_valid {
        return Err(ReplayCoreErrorV1::InvalidResult);
    }
    Ok(())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn zero(value: &[u8]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_two_isolated_producer_commands_without_storage_or_subjects() {
        let request = request();
        let plan = plan_replay_operation_v1(request.clone()).expect("plan");
        assert_eq!(plan.commands[0].producer, ReplayProducerV1::Communications);
        assert_eq!(plan.commands[1].producer, ReplayProducerV1::Mail);
        assert_eq!(plan.commands[0].logical_owner_id, request.logical_owner_id);
        assert_eq!(
            plan.commands[0].attachment_anchor_id,
            request.attachment_anchor_id
        );
        assert_eq!(
            plan.commands[1].attachment_anchor_id,
            request.attachment_anchor_id
        );
    }

    #[test]
    fn waits_for_both_results_and_is_order_independent() {
        let mut state = accepted_replay_operation_v1(request()).expect("state");
        observe_producer_result_v1(&mut state, published(ReplayProducerV1::Mail)).expect("mail");
        assert_eq!(
            replay_operation_status_v1(&state).0,
            AttachmentPreviewEvidenceReplayStateV1::AwaitingProducers
        );
        observe_producer_result_v1(&mut state, published(ReplayProducerV1::Communications))
            .expect("communications");
        assert_eq!(
            replay_operation_status_v1(&state),
            (
                AttachmentPreviewEvidenceReplayStateV1::Completed,
                AttachmentPreviewEvidenceReplayErrorV1::Unspecified
            )
        );
    }

    #[test]
    fn stale_fence_is_terminal_and_duplicate_conflict_fails_closed() {
        let mut state = accepted_replay_operation_v1(request()).expect("state");
        let stale = ReplayProducerResultV1 {
            producer: ReplayProducerV1::Communications,
            original_message_ids: vec![[3; 16]],
            outcome: ReplayProducerOutcomeV1::Rejected,
            failure: ReplayFailureV1::StaleRuntimeFence,
        };
        observe_producer_result_v1(&mut state, stale.clone()).expect("stale");
        assert_eq!(
            replay_operation_status_v1(&state),
            (
                AttachmentPreviewEvidenceReplayStateV1::Rejected,
                AttachmentPreviewEvidenceReplayErrorV1::StaleProducerFence
            )
        );
        let mut collision = stale;
        collision.failure = ReplayFailureV1::StaleGrantFence;
        assert_eq!(
            observe_producer_result_v1(&mut state, collision),
            Err(ReplayCoreErrorV1::ResultConflict)
        );
    }

    fn request() -> AuthenticatedReplayOperationRequestV1 {
        AuthenticatedReplayOperationRequestV1 {
            operation_id: [1; 16],
            attachment_anchor_id: [2; 16],
            logical_owner_id: "owner-1".to_owned(),
            owner_device_actor_sha256: [9; 32],
        }
    }

    fn published(producer: ReplayProducerV1) -> ReplayProducerResultV1 {
        ReplayProducerResultV1 {
            producer,
            original_message_ids: vec![match producer {
                ReplayProducerV1::Communications => [3; 16],
                ReplayProducerV1::Mail => [4; 16],
            }],
            outcome: ReplayProducerOutcomeV1::Published,
            failure: ReplayFailureV1::None,
        }
    }
}
