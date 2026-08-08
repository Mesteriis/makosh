#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-cross-channel-forward-core";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardDraftV1 {
    pub forward_operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub target_conversation_id: [u8; 16],
    pub target_reply_to_message_id: Option<[u8; 16]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardValidationErrorV1 {
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidTargetConversationId,
    InvalidTargetReplyId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardStateV1 {
    Accepted,
    PreparingSource,
    Dispatching,
    DeliveryAccepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardStatusV1 {
    pub state: CrossChannelForwardStateV1,
    pub state_revision: u64,
    pub delivery_intent_id: Option<[u8; 16]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardTransitionV1 {
    BeginSourcePreparation,
    BeginDispatch { delivery_intent_id: [u8; 16] },
    AcceptDelivery,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardTransitionErrorV1 {
    InvalidTransition,
    InvalidDeliveryIntentId,
    RevisionExhausted,
}

pub fn validate_cross_channel_forward_v1(
    draft: CrossChannelForwardDraftV1,
) -> Result<CrossChannelForwardDraftV1, CrossChannelForwardValidationErrorV1> {
    if zero_id(&draft.forward_operation_id) {
        return Err(CrossChannelForwardValidationErrorV1::InvalidOperationId);
    }
    if zero_id(&draft.source_message_id) {
        return Err(CrossChannelForwardValidationErrorV1::InvalidSourceMessageId);
    }
    if zero_id(&draft.target_conversation_id) {
        return Err(CrossChannelForwardValidationErrorV1::InvalidTargetConversationId);
    }
    if draft
        .target_reply_to_message_id
        .as_ref()
        .is_some_and(zero_id)
    {
        return Err(CrossChannelForwardValidationErrorV1::InvalidTargetReplyId);
    }
    Ok(draft)
}

pub fn transition_cross_channel_forward_v1(
    current: CrossChannelForwardStatusV1,
    transition: CrossChannelForwardTransitionV1,
) -> Result<CrossChannelForwardStatusV1, CrossChannelForwardTransitionErrorV1> {
    let (state, delivery_intent_id) = match (current.state, transition) {
        (
            CrossChannelForwardStateV1::Accepted,
            CrossChannelForwardTransitionV1::BeginSourcePreparation,
        ) => (CrossChannelForwardStateV1::PreparingSource, None),
        (
            CrossChannelForwardStateV1::PreparingSource,
            CrossChannelForwardTransitionV1::BeginDispatch { delivery_intent_id },
        ) if !zero_id(&delivery_intent_id) => (
            CrossChannelForwardStateV1::Dispatching,
            Some(delivery_intent_id),
        ),
        (
            CrossChannelForwardStateV1::PreparingSource,
            CrossChannelForwardTransitionV1::BeginDispatch { .. },
        ) => {
            return Err(CrossChannelForwardTransitionErrorV1::InvalidDeliveryIntentId);
        }
        (
            CrossChannelForwardStateV1::Dispatching,
            CrossChannelForwardTransitionV1::AcceptDelivery,
        ) => (
            CrossChannelForwardStateV1::DeliveryAccepted,
            current.delivery_intent_id,
        ),
        (
            CrossChannelForwardStateV1::Accepted
            | CrossChannelForwardStateV1::PreparingSource
            | CrossChannelForwardStateV1::Dispatching,
            CrossChannelForwardTransitionV1::Reject,
        ) => (
            CrossChannelForwardStateV1::Rejected,
            current.delivery_intent_id,
        ),
        _ => {
            return Err(CrossChannelForwardTransitionErrorV1::InvalidTransition);
        }
    };
    let state_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CrossChannelForwardTransitionErrorV1::RevisionExhausted)?;
    Ok(CrossChannelForwardStatusV1 {
        state,
        state_revision,
        delivery_intent_id,
    })
}

fn zero_id(value: &[u8; 16]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CrossChannelForwardDraftV1 {
        CrossChannelForwardDraftV1 {
            forward_operation_id: [1; 16],
            source_message_id: [2; 16],
            target_conversation_id: [3; 16],
            target_reply_to_message_id: Some([4; 16]),
        }
    }

    fn accepted() -> CrossChannelForwardStatusV1 {
        CrossChannelForwardStatusV1 {
            state: CrossChannelForwardStateV1::Accepted,
            state_revision: 1,
            delivery_intent_id: None,
        }
    }

    #[test]
    fn accepts_only_non_zero_canonical_identities() {
        assert_eq!(validate_cross_channel_forward_v1(draft()), Ok(draft()));
        let mut invalid = draft();
        invalid.source_message_id = [0; 16];
        assert_eq!(
            validate_cross_channel_forward_v1(invalid),
            Err(CrossChannelForwardValidationErrorV1::InvalidSourceMessageId)
        );
    }

    #[test]
    fn advances_only_through_the_bounded_forward_state_machine() {
        let preparing = transition_cross_channel_forward_v1(
            accepted(),
            CrossChannelForwardTransitionV1::BeginSourcePreparation,
        )
        .expect("accepted operation can prepare source");
        let dispatching = transition_cross_channel_forward_v1(
            preparing,
            CrossChannelForwardTransitionV1::BeginDispatch {
                delivery_intent_id: [8; 16],
            },
        )
        .expect("prepared source can dispatch");
        let completed = transition_cross_channel_forward_v1(
            dispatching,
            CrossChannelForwardTransitionV1::AcceptDelivery,
        )
        .expect("dispatched operation can accept delivery");
        assert_eq!(
            completed,
            CrossChannelForwardStatusV1 {
                state: CrossChannelForwardStateV1::DeliveryAccepted,
                state_revision: 4,
                delivery_intent_id: Some([8; 16]),
            }
        );
        assert_eq!(
            transition_cross_channel_forward_v1(completed, CrossChannelForwardTransitionV1::Reject,),
            Err(CrossChannelForwardTransitionErrorV1::InvalidTransition)
        );
    }

    #[test]
    fn rejects_zero_downstream_identity_and_revision_overflow() {
        let preparing = CrossChannelForwardStatusV1 {
            state: CrossChannelForwardStateV1::PreparingSource,
            state_revision: 2,
            delivery_intent_id: None,
        };
        assert_eq!(
            transition_cross_channel_forward_v1(
                preparing,
                CrossChannelForwardTransitionV1::BeginDispatch {
                    delivery_intent_id: [0; 16],
                },
            ),
            Err(CrossChannelForwardTransitionErrorV1::InvalidDeliveryIntentId)
        );
        assert_eq!(
            transition_cross_channel_forward_v1(
                CrossChannelForwardStatusV1 {
                    state: CrossChannelForwardStateV1::Accepted,
                    state_revision: u64::MAX,
                    delivery_intent_id: None,
                },
                CrossChannelForwardTransitionV1::BeginSourcePreparation,
            ),
            Err(CrossChannelForwardTransitionErrorV1::RevisionExhausted)
        );
    }
}
