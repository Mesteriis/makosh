#![forbid(unsafe_code)]

pub use makosh_communications_api::{
    CommunicationConversationIdV1, CommunicationConversationSummaryV1, CommunicationMessageIdV1,
    CommunicationMessageLifecycleStateV1, CommunicationMessageSummaryV1,
    CommunicationProviderProvenanceV1, CommunicationSourceCursorV1,
};

pub const PACKAGE: &str = "makosh-communication-delivery-intent-core";
pub const MAX_DELIVERY_BODY_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, PartialEq, Eq)]
pub struct DeliveryIntentDraftV1 {
    pub operation_id: [u8; 16],
    pub conversation_id: CommunicationConversationIdV1,
    pub reply_to_message_id: Option<CommunicationMessageIdV1>,
    pub body_utf8: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ValidatedDeliveryBodyV1(Vec<u8>);

impl ValidatedDeliveryBodyV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<Vec<u8>> for ValidatedDeliveryBodyV1 {
    type Error = DeliveryIntentPlanErrorV1;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() > MAX_DELIVERY_BODY_BYTES_V1 {
            return Err(DeliveryIntentPlanErrorV1::BodyLimitExceeded);
        }
        let body =
            std::str::from_utf8(&value).map_err(|_| DeliveryIntentPlanErrorV1::InvalidBody)?;
        if body.trim().is_empty() {
            return Err(DeliveryIntentPlanErrorV1::InvalidBody);
        }
        Ok(Self(value))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CommunicationDeliveryRouteV1 {
    pub provider: CommunicationProviderProvenanceV1,
    pub account_cursor: CommunicationSourceCursorV1,
    pub conversation_cursor: CommunicationSourceCursorV1,
    pub reply_to_source_cursor: Option<CommunicationSourceCursorV1>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct PlannedDeliveryIntentV1 {
    pub intent_id: [u8; 16],
    pub canonical_conversation_id: CommunicationConversationIdV1,
    pub canonical_reply_to_message_id: Option<CommunicationMessageIdV1>,
    pub route: CommunicationDeliveryRouteV1,
    pub body: ValidatedDeliveryBodyV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryIntentPlanErrorV1 {
    InvalidOperationId,
    InvalidBody,
    BodyLimitExceeded,
    ConversationMismatch,
    ReplyMessageRequired,
    ReplyConversationMismatch,
    ReplyMessageUnavailable,
    InvalidRoute,
}

pub fn plan_delivery_intent_v1(
    draft: DeliveryIntentDraftV1,
    conversation: &CommunicationConversationSummaryV1,
    reply_message: Option<&CommunicationMessageSummaryV1>,
) -> Result<PlannedDeliveryIntentV1, DeliveryIntentPlanErrorV1> {
    if draft.operation_id.iter().all(|byte| *byte == 0) {
        return Err(DeliveryIntentPlanErrorV1::InvalidOperationId);
    }
    let body = ValidatedDeliveryBodyV1::try_from(draft.body_utf8)?;
    if draft.conversation_id != conversation.conversation_id {
        return Err(DeliveryIntentPlanErrorV1::ConversationMismatch);
    }
    if !valid_cursor(conversation.account_cursor) || !valid_cursor(conversation.conversation_cursor)
    {
        return Err(DeliveryIntentPlanErrorV1::InvalidRoute);
    }

    let reply_to_source_cursor = match (draft.reply_to_message_id, reply_message) {
        (None, None) => None,
        (Some(_), None) | (None, Some(_)) => {
            return Err(DeliveryIntentPlanErrorV1::ReplyMessageRequired);
        }
        (Some(expected_id), Some(message)) => {
            if expected_id != message.message_id {
                return Err(DeliveryIntentPlanErrorV1::ReplyMessageRequired);
            }
            if message.conversation_id != conversation.conversation_id {
                return Err(DeliveryIntentPlanErrorV1::ReplyConversationMismatch);
            }
            if message.lifecycle_state != CommunicationMessageLifecycleStateV1::Active {
                return Err(DeliveryIntentPlanErrorV1::ReplyMessageUnavailable);
            }
            if !valid_cursor(message.source_cursor) {
                return Err(DeliveryIntentPlanErrorV1::InvalidRoute);
            }
            Some(message.source_cursor)
        }
    };

    Ok(PlannedDeliveryIntentV1 {
        intent_id: draft.operation_id,
        canonical_conversation_id: draft.conversation_id,
        canonical_reply_to_message_id: draft.reply_to_message_id,
        route: CommunicationDeliveryRouteV1 {
            provider: conversation.provider,
            account_cursor: conversation.account_cursor,
            conversation_cursor: conversation.conversation_cursor,
            reply_to_source_cursor,
        },
        body,
    })
}

fn valid_cursor(cursor: CommunicationSourceCursorV1) -> bool {
    cursor.bytes().iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communications_api::{
        CommunicationBodyStateV1, CommunicationDirectionV1, CommunicationObservationIdV1,
    };

    fn conversation() -> CommunicationConversationSummaryV1 {
        CommunicationConversationSummaryV1 {
            conversation_id: CommunicationConversationIdV1::new([2; 16]),
            account_cursor: CommunicationSourceCursorV1::new([3; 32]),
            conversation_cursor: CommunicationSourceCursorV1::new([4; 32]),
            provider: CommunicationProviderProvenanceV1::Telegram,
            first_observed_at_unix_seconds: 1,
            last_observed_at_unix_seconds: 2,
            last_evidence_id: CommunicationObservationIdV1::new([5; 16]),
        }
    }

    fn reply() -> CommunicationMessageSummaryV1 {
        CommunicationMessageSummaryV1 {
            message_id: CommunicationMessageIdV1::new([6; 16]),
            conversation_id: CommunicationConversationIdV1::new([2; 16]),
            source_cursor: CommunicationSourceCursorV1::new([7; 32]),
            body: CommunicationBodyStateV1::MetadataOnly,
            direction: CommunicationDirectionV1::Incoming,
            lifecycle_state: CommunicationMessageLifecycleStateV1::Active,
            first_observed_at_unix_seconds: 1,
            last_observed_at_unix_seconds: 2,
            last_evidence_id: CommunicationObservationIdV1::new([8; 16]),
        }
    }

    #[test]
    fn plan_preserves_canonical_identity_and_opaque_route() {
        let draft = DeliveryIntentDraftV1 {
            operation_id: [1; 16],
            conversation_id: CommunicationConversationIdV1::new([2; 16]),
            reply_to_message_id: Some(CommunicationMessageIdV1::new([6; 16])),
            body_utf8: b"private reply".to_vec(),
        };
        let plan = plan_delivery_intent_v1(draft, &conversation(), Some(&reply()))
            .expect("valid canonical reply must plan");

        assert_eq!(plan.intent_id, [1; 16]);
        assert_eq!(
            plan.route.provider,
            CommunicationProviderProvenanceV1::Telegram
        );
        assert_eq!(plan.route.account_cursor.bytes(), [3; 32]);
        assert_eq!(plan.route.conversation_cursor.bytes(), [4; 32]);
        assert_eq!(
            plan.route
                .reply_to_source_cursor
                .expect("reply cursor")
                .bytes(),
            [7; 32]
        );
        assert_eq!(plan.body.as_bytes(), b"private reply");
    }

    #[test]
    fn plan_rejects_invalid_content_and_cross_conversation_reply() {
        let base = DeliveryIntentDraftV1 {
            operation_id: [1; 16],
            conversation_id: CommunicationConversationIdV1::new([2; 16]),
            reply_to_message_id: None,
            body_utf8: b"  ".to_vec(),
        };
        assert_eq!(
            plan_delivery_intent_v1(base, &conversation(), None).err(),
            Some(DeliveryIntentPlanErrorV1::InvalidBody)
        );

        let mut foreign_reply = reply();
        foreign_reply.conversation_id = CommunicationConversationIdV1::new([9; 16]);
        let draft = DeliveryIntentDraftV1 {
            operation_id: [1; 16],
            conversation_id: CommunicationConversationIdV1::new([2; 16]),
            reply_to_message_id: Some(CommunicationMessageIdV1::new([6; 16])),
            body_utf8: b"reply".to_vec(),
        };
        assert_eq!(
            plan_delivery_intent_v1(draft, &conversation(), Some(&foreign_reply)).err(),
            Some(DeliveryIntentPlanErrorV1::ReplyConversationMismatch)
        );
    }
}
