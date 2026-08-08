//! Communications-owned attachment safety transition use case.

use makosh_communications_api::{
    AttachmentSafetyTransitionCommandV1, AttachmentSafetyTransitionDecisionV1,
};
use makosh_communications_domain::decide_attachment_safety_transition;
use makosh_communications_persistence::CommunicationsDurablePersistence;

use crate::canonical_outbox::{
    CanonicalEventContextV1, build_attachment_safety_state_changed_outbox_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyTransitionApplyErrorV1 {
    InvalidTransition,
    Conflict,
    Unavailable,
}

pub async fn apply_attachment_safety_transition(
    persistence: &CommunicationsDurablePersistence,
    command: AttachmentSafetyTransitionCommandV1,
    causation_message_id: [u8; 16],
    correlation_id: [u8; 16],
    canonical_event_context: &CanonicalEventContextV1,
) -> Result<AttachmentSafetyTransitionDecisionV1, AttachmentSafetyTransitionApplyErrorV1> {
    let decision = decide_attachment_safety_transition(command)
        .map_err(|_| AttachmentSafetyTransitionApplyErrorV1::InvalidTransition)?;
    let canonical_outbox_record = build_attachment_safety_state_changed_outbox_v1(
        decision,
        causation_message_id,
        correlation_id,
        canonical_event_context,
    )
    .map_err(|_| AttachmentSafetyTransitionApplyErrorV1::InvalidTransition)?;
    let applied = persistence
        .compare_and_set_attachment_safety_state_with_outbox(
            decision,
            &canonical_outbox_record,
            canonical_event_context.recorded_at_unix_seconds,
        )
        .await
        .map_err(|_| AttachmentSafetyTransitionApplyErrorV1::Unavailable)?;
    applied
        .then_some(decision)
        .ok_or(AttachmentSafetyTransitionApplyErrorV1::Conflict)
}
