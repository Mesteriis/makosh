use makosh_communication_bulk_action_api::{
    COMMUNICATION_BULK_ACTION_CONTRACT_MAJOR_V1, COMMUNICATION_BULK_ACTION_MAX_STATUS_PAGE_V1,
    wire::{
        BulkDeliveryBatchStateV1 as WireBatchState, BulkDeliveryErrorCodeV1,
        BulkDeliveryTargetStateV1 as WireTargetState, BulkDeliveryTargetStatusV1,
        GetBulkDeliveryStatusRequestV1, GetBulkDeliveryStatusResponseV1,
        StartBulkDeliveryRequestV1, StartBulkDeliveryResponseV1,
    },
};
use makosh_communication_bulk_action_core::{BulkDeliveryDraftV1, BulkDeliveryTargetDraftV1};
use makosh_communication_bulk_action_persistence::{
    BulkDeliveryBatchStateV1, BulkDeliveryPersistenceErrorV1, BulkDeliveryTargetStateV1,
    CommunicationBulkActionPersistenceV1, CreateBulkDeliveryOutcomeV1, CreateBulkDeliveryV1,
};
use prost::Message;

pub async fn start_bulk_delivery_payload_v1(
    persistence: &CommunicationBulkActionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
    now_unix_seconds: i64,
) -> Vec<u8> {
    let Ok(request) = StartBulkDeliveryRequestV1::decode(payload) else {
        return start_error(Vec::new());
    };
    let response_batch_id = request.batch_operation_id.clone();
    let Some(draft) = start_draft(request) else {
        return start_error(response_batch_id);
    };
    let target_count = draft.targets.len() as u32;
    let batch_id = draft.batch_id;
    match persistence
        .create_batch(CreateBulkDeliveryV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            created_at_unix_seconds: now_unix_seconds,
        })
        .await
    {
        Ok(CreateBulkDeliveryOutcomeV1::Created { .. })
        | Ok(CreateBulkDeliveryOutcomeV1::Existing { .. }) => StartBulkDeliveryResponseV1 {
            batch_id: batch_id.to_vec(),
            state: WireBatchState::BulkDeliveryBatchStateAccepted as i32,
            target_count,
            error: BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(_) => start_error(response_batch_id),
    }
}

pub async fn get_status_payload_v1(
    persistence: &CommunicationBulkActionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetBulkDeliveryStatusRequestV1::decode(payload) else {
        return status_error(
            Vec::new(),
            BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeInvalidRequest,
        );
    };
    let response_batch_id = request.batch_id.clone();
    let Ok(batch_id) = id16(&request.batch_id) else {
        return status_error(
            response_batch_id,
            BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeInvalidRequest,
        );
    };
    let cursor = decode_cursor(&request.cursor);
    if request.protocol_major != COMMUNICATION_BULK_ACTION_CONTRACT_MAJOR_V1
        || request.limit == 0
        || request.limit > COMMUNICATION_BULK_ACTION_MAX_STATUS_PAGE_V1
        || cursor.is_err()
    {
        return status_error(
            response_batch_id,
            BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeInvalidRequest,
        );
    }
    match persistence
        .status_page(
            logical_owner_id,
            batch_id,
            request.limit as u16,
            cursor.ok().flatten(),
        )
        .await
    {
        Ok(page) => GetBulkDeliveryStatusResponseV1 {
            batch_id: page.batch_id.to_vec(),
            state: batch_state(page.state) as i32,
            state_revision: page.state_revision,
            targets: page
                .targets
                .into_iter()
                .map(|target| BulkDeliveryTargetStatusV1 {
                    target_operation_id: target.target_operation_id.to_vec(),
                    state: target_state(target.state) as i32,
                    delivery_intent_id: target.delivery_intent_id.map(|id| id.to_vec()),
                    error: target.error_code.map_or(0, i32::from),
                })
                .collect(),
            next_cursor: page
                .next_cursor
                .map_or_else(Vec::new, |cursor| cursor.to_be_bytes().to_vec()),
            error: BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(BulkDeliveryPersistenceErrorV1::NotFound) => status_error(
            response_batch_id,
            BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeNotFound,
        ),
        Err(_) => status_error(
            response_batch_id,
            BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeUnavailable,
        ),
    }
}

fn start_draft(request: StartBulkDeliveryRequestV1) -> Option<BulkDeliveryDraftV1> {
    if request.protocol_major != COMMUNICATION_BULK_ACTION_CONTRACT_MAJOR_V1 {
        return None;
    }
    Some(BulkDeliveryDraftV1 {
        batch_id: id16(&request.batch_operation_id).ok()?,
        targets: request
            .targets
            .into_iter()
            .map(|target| {
                Some(BulkDeliveryTargetDraftV1 {
                    operation_id: id16(&target.target_operation_id).ok()?,
                    conversation_id: id16(&target.conversation_id).ok()?,
                    reply_to_message_id: target
                        .reply_to_message_id
                        .as_deref()
                        .map(id16)
                        .transpose()
                        .ok()?,
                    body_utf8: target.body_utf8,
                })
            })
            .collect::<Option<Vec<_>>>()?,
    })
}

fn decode_cursor(value: &[u8]) -> Result<Option<u16>, ()> {
    if value.is_empty() {
        return Ok(None);
    }
    let bytes: [u8; 2] = value.try_into().map_err(|_| ())?;
    Ok(Some(u16::from_be_bytes(bytes)))
}

fn id16(value: &[u8]) -> Result<[u8; 16], ()> {
    let id: [u8; 16] = value.try_into().map_err(|_| ())?;
    id.iter().any(|byte| *byte != 0).then_some(id).ok_or(())
}

const fn batch_state(state: BulkDeliveryBatchStateV1) -> WireBatchState {
    match state {
        BulkDeliveryBatchStateV1::Accepted => WireBatchState::BulkDeliveryBatchStateAccepted,
        BulkDeliveryBatchStateV1::Completed => WireBatchState::BulkDeliveryBatchStateCompleted,
        BulkDeliveryBatchStateV1::CompletedWithErrors => {
            WireBatchState::BulkDeliveryBatchStateCompletedWithErrors
        }
        BulkDeliveryBatchStateV1::Rejected => WireBatchState::BulkDeliveryBatchStateRejected,
    }
}

const fn target_state(state: BulkDeliveryTargetStateV1) -> WireTargetState {
    match state {
        BulkDeliveryTargetStateV1::Pending => WireTargetState::BulkDeliveryTargetStatePending,
        BulkDeliveryTargetStateV1::Dispatching => {
            WireTargetState::BulkDeliveryTargetStateDispatching
        }
        BulkDeliveryTargetStateV1::Accepted => WireTargetState::BulkDeliveryTargetStateAccepted,
        BulkDeliveryTargetStateV1::Retryable => WireTargetState::BulkDeliveryTargetStateRetryable,
        BulkDeliveryTargetStateV1::Rejected => WireTargetState::BulkDeliveryTargetStateRejected,
    }
}

fn start_error(batch_id: Vec<u8>) -> Vec<u8> {
    StartBulkDeliveryResponseV1 {
        batch_id,
        state: WireBatchState::BulkDeliveryBatchStateUnspecified as i32,
        target_count: 0,
        error: BulkDeliveryErrorCodeV1::BulkDeliveryErrorCodeInvalidRequest as i32,
    }
    .encode_to_vec()
}

fn status_error(batch_id: Vec<u8>, error: BulkDeliveryErrorCodeV1) -> Vec<u8> {
    GetBulkDeliveryStatusResponseV1 {
        batch_id,
        state: WireBatchState::BulkDeliveryBatchStateUnspecified as i32,
        state_revision: 0,
        targets: Vec::new(),
        next_cursor: Vec::new(),
        error: error as i32,
    }
    .encode_to_vec()
}
