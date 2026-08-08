use makosh_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1};
use makosh_telegram_calls_api::{
    TELEGRAM_CALLS_CONTRACT_MAJOR, TELEGRAM_CALLS_CONTRACT_REVISION,
    TELEGRAM_CALLS_DESCRIPTOR_SET_V1, TELEGRAM_CALLS_MODULE_ID, TELEGRAM_CALLS_OWNER_ID,
    TelegramCallsContractV1,
    wire::{
        CallDirectionV1, CallDiscardReasonV1, CallFailureCategoryV1, CallFrameV1, CallListV1,
        CallOperationKindV1, CallOperationListV1, CallOperationStateV1, CallOperationV1,
        CallStateV1, CallsCommandRequestV1, CallsCommandResponseV1, CallsFailureV1,
        CallsQueryRequestV1, CallsQueryResponseV1, CallsReplayRequestV1, CallsReplayResponseV1,
        EmptyV1, TelegramCallV1, call_frame_v1, calls_command_request_v1,
        calls_command_response_v1, calls_failure_v1::Code as CallsFailureCodeV1,
        calls_query_request_v1, calls_query_response_v1,
    },
};
use makosh_telegram_calls_core::{
    TelegramCallCommand, TelegramCallDirection, TelegramCallDiscardReason,
    TelegramCallFailureCategory, TelegramCallMediaState, TelegramCallOperation,
    TelegramCallOperationKind, TelegramCallOperationState, TelegramCallSession,
    TelegramProviderCallState,
};
use makosh_telegram_calls_persistence::{
    TelegramCallRealtimeEvent, TelegramCallRealtimePayload, TelegramCallsPersistence,
    TelegramCallsPersistenceError,
};
use prost::Message;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::client_port::{
    MODULE_CLIENT_PROTOCOL_MAJOR, TelegramClientPortError, encode_module_response_payload,
};

const MAX_LIST_LIMIT: u32 = 200;
const MAX_ID_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallsRoute {
    Command,
    Query,
    Realtime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TelegramCallsCommandRuntimeError;

pub trait TelegramCallsCommandRuntime {
    fn calls_media_available(&self) -> bool;
    fn calls_fence(&self) -> Option<(u64, u64)>;
    fn owns_calls_account(&self, account_id: &str) -> bool;
    fn resolve_call_owner_provider_identity(
        &mut self,
        correlation_id: &str,
    ) -> Result<String, TelegramCallsCommandRuntimeError>;
}

pub fn calls_route(bytes: &[u8]) -> Result<Option<TelegramCallsRoute>, TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let Some(contract) = envelope.contract.as_ref() else {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client contract is missing".to_owned(),
        ));
    };
    let Some(contract_kind) = TelegramCallsContractV1::from_contract_name(&contract.name) else {
        return Ok(None);
    };
    let route = match contract_kind {
        TelegramCallsContractV1::Query => TelegramCallsRoute::Query,
        TelegramCallsContractV1::Realtime => TelegramCallsRoute::Realtime,
        TelegramCallsContractV1::Command => TelegramCallsRoute::Command,
    };
    validate_calls_envelope(&envelope, contract, contract_kind)?;
    Ok(Some(route))
}

pub async fn handle_calls_module_request<R: TelegramCallsCommandRuntime>(
    bytes: &[u8],
    runtime: &mut R,
    persistence: &TelegramCallsPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let contract = envelope.contract.as_ref().ok_or_else(|| {
        TelegramClientPortError::Protocol("Telegram client contract is missing".to_owned())
    })?;
    let contract_kind =
        TelegramCallsContractV1::from_contract_name(&contract.name).ok_or_else(|| {
            TelegramClientPortError::Protocol("Telegram Calls route is not admitted".to_owned())
        })?;
    let response_payload = match contract_kind {
        TelegramCallsContractV1::Query => {
            validate_calls_envelope(&envelope, contract, contract_kind)?;
            handle_query(&envelope.request_payload, persistence).await?
        }
        TelegramCallsContractV1::Realtime => {
            validate_calls_envelope(&envelope, contract, contract_kind)?;
            handle_replay(&envelope.request_payload, persistence).await?
        }
        TelegramCallsContractV1::Command => {
            validate_calls_envelope(&envelope, contract, contract_kind)?;
            handle_command(&envelope.request_payload, runtime, persistence).await?
        }
    };
    encode_module_response_payload(envelope.request_id, response_payload)
}

fn validate_calls_envelope(
    envelope: &ModuleClientRequestV1,
    contract: &ContractReferenceV1,
    route: TelegramCallsContractV1,
) -> Result<(), TelegramClientPortError> {
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != TELEGRAM_CALLS_MODULE_ID
        || envelope.owner_id != TELEGRAM_CALLS_OWNER_ID
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
        || contract.owner != TELEGRAM_CALLS_OWNER_ID
        || contract.name != route.contract_name()
        || contract.major != TELEGRAM_CALLS_CONTRACT_MAJOR
        || contract.revision != TELEGRAM_CALLS_CONTRACT_REVISION
        || contract.schema_sha256 != Sha256::digest(TELEGRAM_CALLS_DESCRIPTOR_SET_V1).as_slice()
    {
        return Err(TelegramClientPortError::Protocol(
            "Telegram Calls client routing metadata is not admitted".to_owned(),
        ));
    }
    Ok(())
}

async fn handle_query(
    payload: &[u8],
    persistence: &TelegramCallsPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let request = CallsQueryRequestV1::decode(payload)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let response = match request.request {
        Some(calls_query_request_v1::Request::ListCalls(query)) => {
            if invalid_id(&query.account_id) || invalid_cursor(&query.after_call_session_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "account_id")
            } else if let Some(limit) = validated_limit(query.limit) {
                match persistence
                    .list_calls(&query.account_id, &query.after_call_session_id, limit)
                    .await
                {
                    Ok(calls) => {
                        let next_call_session_id = if calls.len() == limit as usize {
                            calls
                                .last()
                                .map(|call| call.call_session_id.clone())
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        match calls_wire(persistence, &calls).await {
                            Ok(calls) => query_response(
                                calls_query_response_v1::Response::CallList(CallListV1 {
                                    calls,
                                    next_call_session_id,
                                }),
                            ),
                            Err(error) => query_persistence_failure(error),
                        }
                    }
                    Err(error) => query_persistence_failure(error),
                }
            } else {
                query_failure(CallsFailureCodeV1::InvalidRequest, "limit")
            }
        }
        Some(calls_query_request_v1::Request::GetCall(query)) => {
            if invalid_id(&query.account_id) || invalid_id(&query.call_session_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "call_session_id")
            } else {
                match persistence
                    .call(&query.account_id, &query.call_session_id)
                    .await
                {
                    Ok(Some(call)) => match persisted_call_wire(persistence, &call).await {
                        Ok(call) => query_response(calls_query_response_v1::Response::Call(call)),
                        Err(error) => query_persistence_failure(error),
                    },
                    Ok(None) => query_failure(CallsFailureCodeV1::NotFound, "call_session_id"),
                    Err(error) => query_persistence_failure(error),
                }
            }
        }
        Some(calls_query_request_v1::Request::GetActiveCall(query)) => {
            if invalid_id(&query.account_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "account_id")
            } else {
                match persistence.active_call(&query.account_id).await {
                    Ok(Some(call)) => match persisted_call_wire(persistence, &call).await {
                        Ok(call) => query_response(calls_query_response_v1::Response::Call(call)),
                        Err(error) => query_persistence_failure(error),
                    },
                    Ok(None) => {
                        query_response(calls_query_response_v1::Response::NoActiveCall(EmptyV1 {}))
                    }
                    Err(error) => query_persistence_failure(error),
                }
            }
        }
        Some(calls_query_request_v1::Request::ListCallOperations(query)) => {
            if invalid_id(&query.account_id) || invalid_cursor(&query.after_operation_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "account_id")
            } else if let Some(limit) = validated_limit(query.limit) {
                match persistence
                    .list_call_operations(&query.account_id, &query.after_operation_id, limit)
                    .await
                {
                    Ok(operations) => {
                        let next_operation_id = if operations.len() == limit as usize {
                            operations
                                .last()
                                .map(|operation| operation.operation_id.clone())
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        query_response(calls_query_response_v1::Response::OperationList(
                            CallOperationListV1 {
                                operations: operations.iter().map(operation_wire).collect(),
                                next_operation_id,
                            },
                        ))
                    }
                    Err(error) => query_persistence_failure(error),
                }
            } else {
                query_failure(CallsFailureCodeV1::InvalidRequest, "limit")
            }
        }
        Some(calls_query_request_v1::Request::GetCallOperation(query)) => {
            if invalid_id(&query.account_id) || invalid_id(&query.operation_id) {
                query_failure(CallsFailureCodeV1::InvalidRequest, "operation_id")
            } else {
                match persistence
                    .call_operation(&query.account_id, &query.operation_id)
                    .await
                {
                    Ok(Some(operation)) => query_response(
                        calls_query_response_v1::Response::Operation(operation_wire(&operation)),
                    ),
                    Ok(None) => query_failure(CallsFailureCodeV1::NotFound, "operation_id"),
                    Err(error) => query_persistence_failure(error),
                }
            }
        }
        None => query_failure(CallsFailureCodeV1::InvalidRequest, "request"),
    };
    Ok(response.encode_to_vec())
}

async fn handle_replay(
    payload: &[u8],
    persistence: &TelegramCallsPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let request = CallsReplayRequestV1::decode(payload)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let response = if invalid_id(&request.account_id) {
        replay_failure(CallsFailureCodeV1::InvalidRequest, "account_id")
    } else if let Some(limit) = validated_limit(request.limit) {
        match persistence
            .realtime_after(&request.account_id, request.after_sequence, limit)
            .await
        {
            Ok(frames) => {
                let next_sequence = frames
                    .last()
                    .map(|frame| frame.sequence)
                    .unwrap_or(request.after_sequence);
                CallsReplayResponseV1 {
                    earliest_available_sequence: frames.first().map(|frame| frame.sequence),
                    latest_available_sequence: frames.last().map(|frame| frame.sequence),
                    frames: frames.iter().map(frame_wire).collect(),
                    next_sequence,
                    reset_required: false,
                    failure: None,
                }
            }
            Err(error) => replay_persistence_failure(error),
        }
    } else {
        replay_failure(CallsFailureCodeV1::InvalidRequest, "limit")
    };
    Ok(response.encode_to_vec())
}

async fn handle_command<R: TelegramCallsCommandRuntime>(
    payload: &[u8],
    runtime: &mut R,
    persistence: &TelegramCallsPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let request = CallsCommandRequestV1::decode(payload)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    if !runtime.calls_media_available() {
        return Ok(command_failure(CallsFailureCodeV1::Unavailable, "call_media").encode_to_vec());
    }
    let Some((runtime_generation, grant_epoch)) = runtime.calls_fence() else {
        return Ok(
            command_failure(CallsFailureCodeV1::Unauthorized, "runtime_fence").encode_to_vec(),
        );
    };
    let command = match request.request {
        Some(calls_command_request_v1::Request::InitiateAudioCall(request)) => {
            let call_session_id = generated_call_session_id().map_err(|_| {
                TelegramClientPortError::Protocol(
                    "Telegram call session identity is unavailable".to_owned(),
                )
            })?;
            TelegramCallCommand::InitiateAudio {
                operation_id: request.operation_id,
                account_id: request.account_id,
                call_session_id,
                provider_user_id: request.provider_user_id,
            }
        }
        Some(calls_command_request_v1::Request::AcceptAudioCall(request)) => {
            TelegramCallCommand::AcceptAudio {
                operation_id: request.operation_id,
                account_id: request.account_id,
                call_session_id: request.call_session_id,
            }
        }
        Some(calls_command_request_v1::Request::DeclineCall(request)) => {
            TelegramCallCommand::Decline {
                operation_id: request.operation_id,
                account_id: request.account_id,
                call_session_id: request.call_session_id,
            }
        }
        Some(calls_command_request_v1::Request::EndCall(request)) => TelegramCallCommand::End {
            operation_id: request.operation_id,
            account_id: request.account_id,
            call_session_id: request.call_session_id,
        },
        Some(calls_command_request_v1::Request::SetLocalMute(request)) => {
            TelegramCallCommand::SetLocalMute {
                operation_id: request.operation_id,
                account_id: request.account_id,
                call_session_id: request.call_session_id,
                muted: request.muted,
            }
        }
        None => {
            return Ok(
                command_failure(CallsFailureCodeV1::InvalidRequest, "request").encode_to_vec(),
            );
        }
    };
    if !runtime.owns_calls_account(command.account_id()) {
        return Ok(command_failure(CallsFailureCodeV1::Unauthorized, "account_id").encode_to_vec());
    }
    let own_provider_user_id = if matches!(command, TelegramCallCommand::InitiateAudio { .. }) {
        match runtime
            .resolve_call_owner_provider_identity(&format!("{}:get-me", command.operation_id()))
        {
            Ok(value) => Some(value),
            Err(_) => {
                return Ok(
                    command_failure(CallsFailureCodeV1::Unavailable, "provider_identity")
                        .encode_to_vec(),
                );
            }
        }
    } else {
        None
    };
    let now_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            TelegramClientPortError::Protocol("Telegram runtime clock is unavailable".to_owned())
        })?
        .as_secs();
    let response = match persistence
        .accept_call_command(
            &command,
            own_provider_user_id.as_deref(),
            runtime_generation,
            grant_epoch,
            now_unix_seconds,
        )
        .await
    {
        Ok(persisted) => command_response(calls_command_response_v1::Response::Accepted(
            operation_wire(&persisted.operation),
        )),
        Err(error) => command_persistence_failure(error),
    };
    Ok(response.encode_to_vec())
}

fn call_wire(
    call: &TelegramCallSession,
    local_muted: bool,
    media_state: Option<TelegramCallMediaState>,
) -> TelegramCallV1 {
    TelegramCallV1 {
        call_session_id: call.call_session_id.clone(),
        account_id: call.account_id.clone(),
        provider_call_unique_id: call.provider_call_unique_id,
        provider_user_id: call.provider_user_id.clone(),
        direction: match call.direction {
            TelegramCallDirection::Incoming => CallDirectionV1::Incoming as i32,
            TelegramCallDirection::Outgoing => CallDirectionV1::Outgoing as i32,
        },
        state: match (call.state, media_state) {
            (TelegramProviderCallState::MediaReady, Some(TelegramCallMediaState::Connecting))
            | (TelegramProviderCallState::MediaReady, Some(TelegramCallMediaState::Reconnecting)) => {
                CallStateV1::Connecting as i32
            }
            (TelegramProviderCallState::MediaReady, Some(TelegramCallMediaState::Active)) => {
                CallStateV1::Active as i32
            }
            (TelegramProviderCallState::MediaReady, Some(TelegramCallMediaState::Failed)) => {
                CallStateV1::Failed as i32
            }
            (TelegramProviderCallState::Pending, _) => CallStateV1::Pending as i32,
            (TelegramProviderCallState::ExchangingKeys, _) => CallStateV1::ExchangingKeys as i32,
            (TelegramProviderCallState::MediaReady, None) => CallStateV1::MediaReady as i32,
            (TelegramProviderCallState::HangingUp, _) => CallStateV1::HangingUp as i32,
            (TelegramProviderCallState::Discarded, _) => CallStateV1::Ended as i32,
            (TelegramProviderCallState::Error, _) => CallStateV1::Failed as i32,
        },
        pending_created: call.pending_created,
        pending_received: call.pending_received,
        discard_reason: call.discard_reason.map(|reason| match reason {
            TelegramCallDiscardReason::Empty => CallDiscardReasonV1::Empty as i32,
            TelegramCallDiscardReason::Missed => CallDiscardReasonV1::Missed as i32,
            TelegramCallDiscardReason::Declined => CallDiscardReasonV1::Declined as i32,
            TelegramCallDiscardReason::Disconnected => CallDiscardReasonV1::Disconnected as i32,
            TelegramCallDiscardReason::HungUp => CallDiscardReasonV1::HungUp as i32,
        }),
        failure_category: call.failure_category.map(failure_category_wire),
        revision: call.revision,
        created_at_unix_seconds: call.created_at_unix_seconds,
        updated_at_unix_seconds: call.updated_at_unix_seconds,
        ended_at_unix_seconds: call.ended_at_unix_seconds,
        local_muted,
    }
}

fn operation_wire(operation: &TelegramCallOperation) -> CallOperationV1 {
    CallOperationV1 {
        operation_id: operation.operation_id.clone(),
        call_session_id: operation.call_session_id.clone(),
        account_id: operation.account_id.clone(),
        operation_kind: operation.kind.storage_name().to_owned(),
        operation_state: operation.state.storage_name().to_owned(),
        accepted_at_unix_seconds: operation.accepted_at_unix_seconds,
        completed_at_unix_seconds: operation.completed_at_unix_seconds,
        kind: match operation.kind {
            TelegramCallOperationKind::InitiateAudio => CallOperationKindV1::InitiateAudio as i32,
            TelegramCallOperationKind::AcceptAudio => CallOperationKindV1::AcceptAudio as i32,
            TelegramCallOperationKind::Decline => CallOperationKindV1::Decline as i32,
            TelegramCallOperationKind::End => CallOperationKindV1::End as i32,
            TelegramCallOperationKind::SetLocalMute => CallOperationKindV1::SetLocalMute as i32,
        },
        state: match operation.state {
            TelegramCallOperationState::Accepted => CallOperationStateV1::Accepted as i32,
            TelegramCallOperationState::Dispatching => CallOperationStateV1::Dispatching as i32,
            TelegramCallOperationState::AwaitingProvider => {
                CallOperationStateV1::AwaitingProvider as i32
            }
            TelegramCallOperationState::Completed => CallOperationStateV1::Completed as i32,
            TelegramCallOperationState::Failed => CallOperationStateV1::Failed as i32,
        },
        failure_category: operation.failure_category.map(failure_category_wire),
        revision: operation.revision,
    }
}

fn frame_wire(frame: &TelegramCallRealtimeEvent) -> CallFrameV1 {
    CallFrameV1 {
        sequence: frame.sequence,
        event: Some(match &frame.payload {
            TelegramCallRealtimePayload::Call {
                session,
                local_muted,
            } => call_frame_v1::Event::Call(call_wire(session, *local_muted, None)),
            TelegramCallRealtimePayload::Operation(operation) => {
                call_frame_v1::Event::Operation(operation_wire(operation))
            }
        }),
    }
}

async fn calls_wire(
    persistence: &TelegramCallsPersistence,
    calls: &[TelegramCallSession],
) -> Result<Vec<TelegramCallV1>, TelegramCallsPersistenceError> {
    let mut wire = Vec::with_capacity(calls.len());
    for call in calls {
        wire.push(persisted_call_wire(persistence, call).await?);
    }
    Ok(wire)
}

async fn persisted_call_wire(
    persistence: &TelegramCallsPersistence,
    call: &TelegramCallSession,
) -> Result<TelegramCallV1, TelegramCallsPersistenceError> {
    let local_muted = persistence
        .local_mute(&call.account_id, &call.call_session_id)
        .await?;
    let media_state = persistence
        .media_projection(&call.account_id, &call.call_session_id)
        .await?
        .map(|projection| projection.state);
    Ok(call_wire(call, local_muted, media_state))
}

fn failure_category_wire(category: TelegramCallFailureCategory) -> i32 {
    match category {
        TelegramCallFailureCategory::Network => CallFailureCategoryV1::Network as i32,
        TelegramCallFailureCategory::NotAvailable => CallFailureCategoryV1::NotAvailable as i32,
        TelegramCallFailureCategory::Permission => CallFailureCategoryV1::Permission as i32,
        TelegramCallFailureCategory::Protocol => CallFailureCategoryV1::Protocol as i32,
        TelegramCallFailureCategory::Unknown => CallFailureCategoryV1::Unknown as i32,
    }
}

fn query_response(response: calls_query_response_v1::Response) -> CallsQueryResponseV1 {
    CallsQueryResponseV1 {
        response: Some(response),
    }
}

fn command_response(response: calls_command_response_v1::Response) -> CallsCommandResponseV1 {
    CallsCommandResponseV1 {
        response: Some(response),
    }
}

fn command_failure(code: CallsFailureCodeV1, field: &str) -> CallsCommandResponseV1 {
    command_response(calls_command_response_v1::Response::Failure(failure(
        code, field,
    )))
}

fn command_persistence_failure(error: TelegramCallsPersistenceError) -> CallsCommandResponseV1 {
    let (code, field) = persistence_failure(error);
    command_failure(code, field)
}

fn query_failure(code: CallsFailureCodeV1, field: &str) -> CallsQueryResponseV1 {
    query_response(calls_query_response_v1::Response::Failure(failure(
        code, field,
    )))
}

fn query_persistence_failure(error: TelegramCallsPersistenceError) -> CallsQueryResponseV1 {
    let (code, field) = persistence_failure(error);
    query_failure(code, field)
}

fn replay_failure(code: CallsFailureCodeV1, field: &str) -> CallsReplayResponseV1 {
    CallsReplayResponseV1 {
        frames: Vec::new(),
        next_sequence: 0,
        reset_required: false,
        earliest_available_sequence: None,
        latest_available_sequence: None,
        failure: Some(failure(code, field)),
    }
}

fn replay_persistence_failure(error: TelegramCallsPersistenceError) -> CallsReplayResponseV1 {
    let (code, field) = persistence_failure(error);
    replay_failure(code, field)
}

fn persistence_failure(error: TelegramCallsPersistenceError) -> (CallsFailureCodeV1, &'static str) {
    match error {
        TelegramCallsPersistenceError::InvalidRequest(field) => {
            (CallsFailureCodeV1::InvalidRequest, field)
        }
        TelegramCallsPersistenceError::IdentityConflict
        | TelegramCallsPersistenceError::StateRegression
        | TelegramCallsPersistenceError::TerminalConflict
        | TelegramCallsPersistenceError::IdempotencyConflict => {
            (CallsFailureCodeV1::Conflict, "call_state")
        }
        TelegramCallsPersistenceError::CommandConflict(field) => {
            (CallsFailureCodeV1::Conflict, field)
        }
        TelegramCallsPersistenceError::Database | TelegramCallsPersistenceError::InvalidRow => {
            (CallsFailureCodeV1::Unavailable, "persistence")
        }
    }
}

fn failure(code: CallsFailureCodeV1, field: &str) -> CallsFailureV1 {
    CallsFailureV1 {
        code: code as i32,
        field: field.to_owned(),
    }
}

fn validated_limit(limit: u32) -> Option<u32> {
    (1..=MAX_LIST_LIMIT).contains(&limit).then_some(limit)
}

fn invalid_id(value: &str) -> bool {
    value.trim().is_empty() || value.len() > MAX_ID_BYTES
}

fn invalid_cursor(value: &str) -> bool {
    value.len() > MAX_ID_BYTES
}

fn generated_call_session_id() -> Result<String, ()> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| ())?;
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(());
    }
    Ok(format!(
        "call-{}",
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ))
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1};

    use super::*;

    fn request_envelope(contract: TelegramCallsContractV1, payload: Vec<u8>) -> Vec<u8> {
        ModuleClientRequestV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            module_id: TELEGRAM_CALLS_MODULE_ID.to_owned(),
            owner_id: TELEGRAM_CALLS_OWNER_ID.to_owned(),
            contract: Some(ContractReferenceV1 {
                owner: TELEGRAM_CALLS_OWNER_ID.to_owned(),
                name: contract.contract_name().to_owned(),
                major: TELEGRAM_CALLS_CONTRACT_MAJOR,
                revision: TELEGRAM_CALLS_CONTRACT_REVISION,
                schema_sha256: Sha256::digest(TELEGRAM_CALLS_DESCRIPTOR_SET_V1).to_vec(),
            }),
            request_id: 1,
            request_payload: payload,
            logical_owner_id: String::new(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
        }
        .encode_to_vec()
    }

    #[test]
    fn calls_routes_are_exact_and_separate() {
        let query = request_envelope(
            TelegramCallsContractV1::Query,
            CallsQueryRequestV1 {
                request: Some(calls_query_request_v1::Request::ListCalls(
                    makosh_telegram_calls_api::wire::ListCallsRequestV1 {
                        account_id: "account-1".to_owned(),
                        after_call_session_id: String::new(),
                        limit: 10,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let realtime = request_envelope(
            TelegramCallsContractV1::Realtime,
            CallsReplayRequestV1 {
                account_id: "account-1".to_owned(),
                after_sequence: 0,
                limit: 10,
            }
            .encode_to_vec(),
        );
        let command = request_envelope(TelegramCallsContractV1::Command, vec![1]);

        assert!(matches!(
            calls_route(&query),
            Ok(Some(TelegramCallsRoute::Query))
        ));
        assert!(matches!(
            calls_route(&realtime),
            Ok(Some(TelegramCallsRoute::Realtime))
        ));
        assert!(matches!(
            calls_route(&command),
            Ok(Some(TelegramCallsRoute::Command))
        ));
    }

    #[test]
    fn call_wire_excludes_runtime_scoped_tdlib_identity() {
        let call = TelegramCallSession {
            call_session_id: "call-1".to_owned(),
            account_id: "account-1".to_owned(),
            runtime_generation: 9,
            tdlib_call_id: 77,
            provider_call_unique_id: Some(101),
            provider_user_id: "user-2".to_owned(),
            direction: TelegramCallDirection::Incoming,
            state: TelegramProviderCallState::Pending,
            pending_created: true,
            pending_received: false,
            discard_reason: None,
            failure_category: None,
            revision: 1,
            created_at_unix_seconds: 10,
            updated_at_unix_seconds: 10,
            ended_at_unix_seconds: None,
        };

        let bytes = call_wire(&call, false, None).encode_to_vec();
        assert!(!bytes.is_empty());
        assert!(!String::from_utf8_lossy(&bytes).contains("runtime_generation"));
        assert!(!String::from_utf8_lossy(&bytes).contains("tdlib_call_id"));
    }
}
