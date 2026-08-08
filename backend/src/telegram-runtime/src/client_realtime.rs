use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ContractReferenceV1, ManagedRuntimeClientRealtimePublishRequestV1,
        ManagedRuntimeControlRequestV1, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::client_realtime::{
        validate_managed_client_realtime_publish_request_v1,
        validate_managed_client_realtime_publish_response_v1,
    },
};
use makosh_telegram_api::{
    TelegramAuthorizationStatus,
    client_contract::{
        TELEGRAM_AUTHORIZATION_STATUS_CHANGED_CONTRACT_NAME_V1, TELEGRAM_CLIENT_CONTRACT_MAJOR,
        TELEGRAM_CLIENT_CONTRACT_REVISION, TELEGRAM_CLIENT_DESCRIPTOR_SET_V1, TELEGRAM_OWNER_ID,
    },
    wire::AuthorizationStatusResponse,
};
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramAuthorizationRealtimeErrorV1 {
    InvalidStatus,
    Unavailable,
}

pub fn publish_authorization_status_changed_v1<D>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut D,
    logical_owner_id: &str,
    runtime_generation: u64,
    status_revision: u64,
    occurred_at_unix_millis: u64,
    status: &TelegramAuthorizationStatus,
) -> Result<(), TelegramAuthorizationRealtimeErrorV1>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    if logical_owner_id.trim().is_empty()
        || runtime_generation == 0
        || status_revision == 0
        || occurred_at_unix_millis == 0
        || !valid_public_state(&status.state)
    {
        return Err(TelegramAuthorizationRealtimeErrorV1::InvalidStatus);
    }
    let event_id = event_id(
        logical_owner_id,
        runtime_generation,
        status_revision,
        &status.state,
    );
    let contract = ContractReferenceV1 {
        owner: TELEGRAM_OWNER_ID.to_owned(),
        name: TELEGRAM_AUTHORIZATION_STATUS_CHANGED_CONTRACT_NAME_V1.to_owned(),
        major: TELEGRAM_CLIENT_CONTRACT_MAJOR,
        revision: TELEGRAM_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(TELEGRAM_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
    };
    let cursor = format!("telegram-authorization/{runtime_generation}/{status_revision}");
    let request = ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(contract),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id.to_vec(),
        cursor: cursor.clone(),
        event_kind: TELEGRAM_AUTHORIZATION_STATUS_CHANGED_CONTRACT_NAME_V1.to_owned(),
        occurred_at_unix_millis,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload: AuthorizationStatusResponse {
            state: status.state.clone(),
            qr_link: None,
            password_hint: None,
        }
        .encode_to_vec(),
    };
    validate_managed_client_realtime_publish_request_v1(&request)
        .map_err(|_| TelegramAuthorizationRealtimeErrorV1::InvalidStatus)?;
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::PublishClientRealtime(request)),
            },
            dispatcher,
        )
        .map_err(|_| TelegramAuthorizationRealtimeErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(TelegramAuthorizationRealtimeErrorV1::Unavailable);
    }
    let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
        return Err(TelegramAuthorizationRealtimeErrorV1::Unavailable);
    };
    if validate_managed_client_realtime_publish_response_v1(&response).is_err()
        || response.accepted_cursor != cursor
    {
        return Err(TelegramAuthorizationRealtimeErrorV1::Unavailable);
    }
    Ok(())
}

fn event_id(owner: &str, generation: u64, revision: u64, state: &str) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.telegram.authorization.client-realtime.v1\0");
    hash.update(owner.as_bytes());
    hash.update([0]);
    hash.update(generation.to_be_bytes());
    hash.update(revision.to_be_bytes());
    hash.update(state.as_bytes());
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn valid_public_state(state: &str) -> bool {
    matches!(
        state,
        "waiting_parameters"
            | "waiting_encryption_key"
            | "waiting_qr_scan"
            | "waiting_password"
            | "ready"
            | "closing"
            | "closed"
            | "error"
            | "other"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realtime_payload_excludes_qr_link_and_password_hint() {
        let status = TelegramAuthorizationStatus {
            state: "waiting_qr_scan".to_owned(),
            qr_link: Some("tg://private".to_owned()),
            password_hint: Some("private".to_owned()),
        };
        let bytes = AuthorizationStatusResponse {
            state: status.state,
            qr_link: None,
            password_hint: None,
        }
        .encode_to_vec();
        let decoded = AuthorizationStatusResponse::decode(bytes.as_slice()).expect("status");
        assert_eq!(decoded.state, "waiting_qr_scan");
        assert_eq!(decoded.qr_link, None);
        assert_eq!(decoded.password_hint, None);
    }
}
