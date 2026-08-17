use std::os::unix::net::UnixStream;
use std::time::Duration;

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
        TELEGRAM_CLIENT_CONTRACT_REVISION, TELEGRAM_CLIENT_DESCRIPTOR_SET_V1,
        TELEGRAM_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1, TELEGRAM_OWNER_ID,
    },
    wire::{AuthorizationStatusResponse, TelegramOperationalProjectionChangedV1},
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::managed_control::with_blocking_control_channel_timeout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramClientRealtimeErrorV1 {
    InvalidEvent,
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
) -> Result<(), TelegramClientRealtimeErrorV1>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    if logical_owner_id.trim().is_empty()
        || runtime_generation == 0
        || status_revision == 0
        || occurred_at_unix_millis == 0
        || !valid_public_state(&status.state)
    {
        return Err(TelegramClientRealtimeErrorV1::InvalidEvent);
    }
    let event_id = authorization_event_id(
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
        .map_err(|_| TelegramClientRealtimeErrorV1::InvalidEvent)?;
    publish(channel, dispatcher, request, &cursor)
}

pub fn publish_operational_projection_changed_v1<D>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut D,
    logical_owner_id: &str,
    runtime_generation: u64,
    account_id: &str,
    latest_sequence: u64,
    occurred_at_unix_millis: u64,
) -> Result<(), TelegramClientRealtimeErrorV1>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    if logical_owner_id.trim().is_empty()
        || runtime_generation == 0
        || account_id.trim().is_empty()
        || account_id.len() > 256
        || latest_sequence == 0
        || occurred_at_unix_millis == 0
    {
        return Err(TelegramClientRealtimeErrorV1::InvalidEvent);
    }
    let account_digest = format!("{:x}", Sha256::digest(account_id.as_bytes()));
    let cursor = format!(
        "telegram-operational/{}/{runtime_generation}/{latest_sequence}",
        &account_digest[..32]
    );
    let request = ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(ContractReferenceV1 {
            owner: TELEGRAM_OWNER_ID.to_owned(),
            name: TELEGRAM_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1.to_owned(),
            major: TELEGRAM_CLIENT_CONTRACT_MAJOR,
            revision: TELEGRAM_CLIENT_CONTRACT_REVISION,
            schema_sha256: Sha256::digest(TELEGRAM_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
        }),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: operational_event_id(
            logical_owner_id,
            runtime_generation,
            account_id,
            latest_sequence,
        )
        .to_vec(),
        cursor: cursor.clone(),
        event_kind: TELEGRAM_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1.to_owned(),
        occurred_at_unix_millis,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload: TelegramOperationalProjectionChangedV1 {
            account_id: account_id.to_owned(),
            latest_sequence,
        }
        .encode_to_vec(),
    };
    validate_managed_client_realtime_publish_request_v1(&request)
        .map_err(|_| TelegramClientRealtimeErrorV1::InvalidEvent)?;
    publish(channel, dispatcher, request, &cursor)
}

fn publish<D>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut D,
    request: ManagedRuntimeClientRealtimePublishRequestV1,
    expected_cursor: &str,
) -> Result<(), TelegramClientRealtimeErrorV1>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    let response =
        with_blocking_control_channel_timeout(channel, Duration::from_millis(250), |channel| {
            channel.request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::PublishClientRealtime(request)),
                },
                dispatcher,
            )
        });
    let response = response
        .map_err(|_| TelegramClientRealtimeErrorV1::Unavailable)?
        .map_err(|_| TelegramClientRealtimeErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(TelegramClientRealtimeErrorV1::Unavailable);
    }
    let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
        return Err(TelegramClientRealtimeErrorV1::Unavailable);
    };
    if validate_managed_client_realtime_publish_response_v1(&response).is_err()
        || response.accepted_cursor != expected_cursor
    {
        return Err(TelegramClientRealtimeErrorV1::Unavailable);
    }
    Ok(())
}

fn authorization_event_id(owner: &str, generation: u64, revision: u64, state: &str) -> [u8; 16] {
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

fn operational_event_id(owner: &str, generation: u64, account_id: &str, sequence: u64) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.telegram.operational.client-realtime.v1\0");
    hash.update(owner.as_bytes());
    hash.update([0]);
    hash.update(generation.to_be_bytes());
    hash.update(account_id.as_bytes());
    hash.update([0]);
    hash.update(sequence.to_be_bytes());
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
    use makosh_runtime_protocol::managed_control::{
        ManagedControlChannelV2, RejectManagedControlRequestsV2,
    };
    use makosh_runtime_protocol::v1::{
        ManagedRuntimeClientRealtimePublishResponseV1, ManagedRuntimeControlResponseV1,
    };
    use std::{os::unix::net::UnixStream, thread, time::Duration};

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

    #[test]
    fn operational_realtime_payload_contains_only_account_and_sequence() {
        let (client, server) = UnixStream::pair().expect("control pair");
        client.set_nonblocking(true).expect("nonblocking client");
        let (published, received) = std::sync::mpsc::sync_channel(1);
        let (release_server, wait_for_client) = std::sync::mpsc::sync_channel(0);
        let server = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(server);
            let (correlation_id, request) = channel.receive_request().expect("realtime request");
            let Some(Operation::PublishClientRealtime(request)) = request.operation else {
                panic!("expected realtime publish request");
            };
            let payload =
                TelegramOperationalProjectionChangedV1::decode(request.payload.as_slice())
                    .expect("projection changed payload");
            published
                .send((request.clone(), payload))
                .expect("capture request");
            channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientRealtimePublish(
                            ManagedRuntimeClientRealtimePublishResponseV1 {
                                accepted_cursor: request.cursor,
                            },
                        )),
                        error_code: String::new(),
                    },
                )
                .expect("realtime response");
            wait_for_client.recv().expect("client completed");
        });
        let mut channel = ManagedControlChannelV2::new(client);
        let mut dispatcher = RejectManagedControlRequestsV2;

        publish_operational_projection_changed_v1(
            &mut channel,
            &mut dispatcher,
            "owner-1",
            7,
            "local-account",
            42,
            1,
        )
        .expect("publish operational realtime");
        let (request, payload) = received.recv().expect("published request");
        assert_eq!(
            request.event_kind,
            TELEGRAM_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1
        );
        assert_eq!(payload.account_id, "local-account");
        assert_eq!(payload.latest_sequence, 42);
        assert!(!request.cursor.contains("local-account"));
        assert_eq!(request.payload, payload.encode_to_vec());
        release_server.send(()).expect("release server");
        server.join().expect("server join");
    }

    #[test]
    fn publishes_realtime_from_the_nonblocking_provider_loop() {
        let (client, server) = UnixStream::pair().expect("control pair");
        client.set_nonblocking(true).expect("nonblocking client");
        let (release_server, wait_for_client) = std::sync::mpsc::sync_channel(0);
        let server = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(server);
            let (correlation_id, request) = channel.receive_request().expect("realtime request");
            let Some(Operation::PublishClientRealtime(request)) = request.operation else {
                panic!("expected realtime publish request");
            };
            thread::sleep(Duration::from_millis(20));
            channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientRealtimePublish(
                            ManagedRuntimeClientRealtimePublishResponseV1 {
                                accepted_cursor: request.cursor,
                            },
                        )),
                        error_code: String::new(),
                    },
                )
                .expect("realtime response");
            wait_for_client.recv().expect("client completed");
        });
        let mut channel = ManagedControlChannelV2::new(client);
        let mut dispatcher = RejectManagedControlRequestsV2;

        publish_authorization_status_changed_v1(
            &mut channel,
            &mut dispatcher,
            "owner-1",
            1,
            1,
            1,
            &TelegramAuthorizationStatus {
                state: "ready".to_owned(),
                qr_link: None,
                password_hint: None,
            },
        )
        .expect("publish realtime from nonblocking provider loop");
        release_server.send(()).expect("release server");
        server.join().expect("server join");
    }
}
