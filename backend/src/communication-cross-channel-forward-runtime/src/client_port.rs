use makosh_communication_cross_channel_forward_api::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_MAJOR_V1,
    wire::{
        CrossChannelForwardErrorCodeV1, CrossChannelForwardStateV1 as WireState,
        GetCrossChannelForwardStatusRequestV1, GetCrossChannelForwardStatusResponseV1,
        StartCrossChannelForwardRequestV1, StartCrossChannelForwardResponseV1,
    },
};
use makosh_communication_cross_channel_forward_core::{
    CrossChannelForwardDraftV1, CrossChannelForwardStateV1,
};
use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CreateCrossChannelForwardOutcomeV1,
    CreateCrossChannelForwardV1, CrossChannelForwardPersistenceErrorV1,
};
use prost::Message;

pub async fn start_cross_channel_forward_payload_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartCrossChannelForwardRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeInvalidRequest,
        );
    };
    let response_forward_id = request.forward_operation_id.clone();
    let Some(draft) = start_draft(request) else {
        return start_error(
            response_forward_id,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeInvalidRequest,
        );
    };
    let forward_id = draft.forward_operation_id;
    match persistence
        .create_forward(CreateCrossChannelForwardV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            created_at_unix_millis: now_unix_millis,
        })
        .await
    {
        Ok(CreateCrossChannelForwardOutcomeV1::Created { .. })
        | Ok(CreateCrossChannelForwardOutcomeV1::Existing { .. }) => {
            StartCrossChannelForwardResponseV1 {
                forward_id: forward_id.to_vec(),
                state: WireState::CrossChannelForwardStateAccepted as i32,
                error: CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnspecified
                    as i32,
            }
            .encode_to_vec()
        }
        Err(CrossChannelForwardPersistenceErrorV1::Conflict) => start_error(
            response_forward_id,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeInvalidRequest,
        ),
        Err(_) => start_error(
            response_forward_id,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnavailable,
        ),
    }
}

pub async fn get_cross_channel_forward_status_payload_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetCrossChannelForwardStatusRequestV1::decode(payload) else {
        return status_error(
            Vec::new(),
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeInvalidRequest,
        );
    };
    let response_forward_id = request.forward_id.clone();
    let Ok(forward_id) = id16(&request.forward_id) else {
        return status_error(
            response_forward_id,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_MAJOR_V1 {
        return status_error(
            response_forward_id,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeInvalidRequest,
        );
    }
    match persistence.status(logical_owner_id, &forward_id).await {
        Ok(status) => GetCrossChannelForwardStatusResponseV1 {
            forward_id: status.forward_id.to_vec(),
            source_message_id: status.source_message_id.to_vec(),
            target_conversation_id: status.target_conversation_id.to_vec(),
            target_reply_to_message_id: status
                .target_reply_to_message_id
                .map(|value| value.to_vec()),
            state: wire_state(status.state) as i32,
            state_revision: status.state_revision,
            delivery_intent_id: status.delivery_intent_id.map(|value| value.to_vec()),
            error: status.error_code.map_or(
                CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnspecified as i32,
                i32::from,
            ),
        }
        .encode_to_vec(),
        Err(CrossChannelForwardPersistenceErrorV1::NotFound) => status_error(
            response_forward_id,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeNotFound,
        ),
        Err(_) => status_error(
            response_forward_id,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnavailable,
        ),
    }
}

fn start_draft(request: StartCrossChannelForwardRequestV1) -> Option<CrossChannelForwardDraftV1> {
    if request.protocol_major != COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_MAJOR_V1 {
        return None;
    }
    Some(CrossChannelForwardDraftV1 {
        forward_operation_id: id16(&request.forward_operation_id).ok()?,
        source_message_id: id16(&request.source_message_id).ok()?,
        target_conversation_id: id16(&request.target_conversation_id).ok()?,
        target_reply_to_message_id: request
            .target_reply_to_message_id
            .as_deref()
            .map(id16)
            .transpose()
            .ok()?,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], ()> {
    let value: [u8; 16] = value.try_into().map_err(|_| ())?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(())
}

pub(crate) const fn wire_state(state: CrossChannelForwardStateV1) -> WireState {
    match state {
        CrossChannelForwardStateV1::Accepted => WireState::CrossChannelForwardStateAccepted,
        CrossChannelForwardStateV1::PreparingSource => {
            WireState::CrossChannelForwardStatePreparingSource
        }
        CrossChannelForwardStateV1::Dispatching => WireState::CrossChannelForwardStateDispatching,
        CrossChannelForwardStateV1::DeliveryAccepted => {
            WireState::CrossChannelForwardStateDeliveryAccepted
        }
        CrossChannelForwardStateV1::Rejected => WireState::CrossChannelForwardStateRejected,
    }
}

fn start_error(forward_id: Vec<u8>, error: CrossChannelForwardErrorCodeV1) -> Vec<u8> {
    StartCrossChannelForwardResponseV1 {
        forward_id,
        state: WireState::CrossChannelForwardStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn status_error(forward_id: Vec<u8>, error: CrossChannelForwardErrorCodeV1) -> Vec<u8> {
    GetCrossChannelForwardStatusResponseV1 {
        forward_id,
        source_message_id: Vec::new(),
        target_conversation_id: Vec::new(),
        target_reply_to_message_id: None,
        state: WireState::CrossChannelForwardStateUnspecified as i32,
        state_revision: 0,
        delivery_intent_id: None,
        error: error as i32,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_draft_accepts_only_exact_provider_neutral_identities() {
        let draft = start_draft(StartCrossChannelForwardRequestV1 {
            protocol_major: COMMUNICATION_CROSS_CHANNEL_FORWARD_CONTRACT_MAJOR_V1,
            forward_operation_id: vec![1; 16],
            source_message_id: vec![2; 16],
            target_conversation_id: vec![3; 16],
            target_reply_to_message_id: Some(vec![4; 16]),
        })
        .expect("valid draft");
        assert_eq!(draft.forward_operation_id, [1; 16]);
        assert_eq!(draft.source_message_id, [2; 16]);
        assert_eq!(draft.target_conversation_id, [3; 16]);
        assert_eq!(draft.target_reply_to_message_id, Some([4; 16]));

        assert!(
            start_draft(StartCrossChannelForwardRequestV1 {
                protocol_major: 2,
                forward_operation_id: vec![1; 16],
                source_message_id: vec![2; 16],
                target_conversation_id: vec![3; 16],
                target_reply_to_message_id: None,
            })
            .is_none()
        );
    }

    #[test]
    fn public_errors_are_typed_and_body_free() {
        let response = StartCrossChannelForwardResponseV1::decode(
            start_error(
                vec![9; 16],
                CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnavailable,
            )
            .as_slice(),
        )
        .expect("decode response");
        assert_eq!(response.forward_id, vec![9; 16]);
        assert_eq!(
            response.error,
            CrossChannelForwardErrorCodeV1::CrossChannelForwardErrorCodeUnavailable as i32
        );
    }
}
