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
use makosh_zulip_api::{
    client_contract::{
        ZULIP_CLIENT_CONTRACT_MAJOR, ZULIP_CLIENT_CONTRACT_REVISION,
        ZULIP_CLIENT_DESCRIPTOR_SET_V1, ZULIP_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1,
        ZULIP_OWNER_ID,
    },
    wire::ZulipOperationalProjectionChangedV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ZulipClientRealtimeErrorV1 {
    InvalidEvent,
    Unavailable,
}

pub(crate) fn publish_operational_projection_changed_v1<D>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut D,
    logical_owner_id: &str,
    runtime_generation: u64,
    account_id: &str,
    revision: u64,
    occurred_at_unix_millis: u64,
) -> Result<(), ZulipClientRealtimeErrorV1>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    if logical_owner_id.trim().is_empty()
        || runtime_generation == 0
        || account_id.trim().is_empty()
        || account_id.len() > 512
        || revision == 0
        || occurred_at_unix_millis == 0
    {
        return Err(ZulipClientRealtimeErrorV1::InvalidEvent);
    }
    let account_digest = Sha256::digest(account_id.as_bytes())[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let cursor = format!("zulip-operational/{account_digest}/{runtime_generation}/{revision}");
    let request = ManagedRuntimeClientRealtimePublishRequestV1 {
        contract: Some(ContractReferenceV1 {
            owner: ZULIP_OWNER_ID.to_owned(),
            name: ZULIP_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1.to_owned(),
            major: ZULIP_CLIENT_CONTRACT_MAJOR,
            revision: ZULIP_CLIENT_CONTRACT_REVISION,
            schema_sha256: Sha256::digest(ZULIP_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
        }),
        logical_owner_id: logical_owner_id.to_owned(),
        event_id: event_id(logical_owner_id, runtime_generation, account_id, revision).to_vec(),
        cursor: cursor.clone(),
        event_kind: ZULIP_OPERATIONAL_PROJECTION_CHANGED_CONTRACT_NAME_V1.to_owned(),
        occurred_at_unix_millis,
        causation_id: String::new(),
        correlation_id: String::new(),
        trace_id: String::new(),
        payload: ZulipOperationalProjectionChangedV1 {
            account_id: account_id.to_owned(),
            revision,
        }
        .encode_to_vec(),
    };
    validate_managed_client_realtime_publish_request_v1(&request)
        .map_err(|_| ZulipClientRealtimeErrorV1::InvalidEvent)?;
    if set_blocking(channel).is_err() {
        let _ = restore_nonblocking(channel);
        return Err(ZulipClientRealtimeErrorV1::Unavailable);
    }
    let response = channel.request_next_with_dispatch(
        ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::PublishClientRealtime(request)),
        },
        dispatcher,
    );
    let restored = restore_nonblocking(channel);
    let response = response.map_err(|_| ZulipClientRealtimeErrorV1::Unavailable)?;
    restored.map_err(|_| ZulipClientRealtimeErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(ZulipClientRealtimeErrorV1::Unavailable);
    }
    let Some(ControlResult::ClientRealtimePublish(response)) = response.result else {
        return Err(ZulipClientRealtimeErrorV1::Unavailable);
    };
    if validate_managed_client_realtime_publish_response_v1(&response).is_err()
        || response.accepted_cursor != cursor
    {
        return Err(ZulipClientRealtimeErrorV1::Unavailable);
    }
    Ok(())
}

fn set_blocking(channel: &mut ManagedControlChannelV2<UnixStream>) -> std::io::Result<()> {
    channel.inner_mut().set_nonblocking(false)?;
    channel
        .inner_mut()
        .set_read_timeout(Some(Duration::from_millis(250)))?;
    channel
        .inner_mut()
        .set_write_timeout(Some(Duration::from_millis(250)))
}

fn restore_nonblocking(channel: &mut ManagedControlChannelV2<UnixStream>) -> std::io::Result<()> {
    channel.inner_mut().set_read_timeout(None)?;
    channel.inner_mut().set_write_timeout(None)?;
    channel.inner_mut().set_nonblocking(true)
}

fn event_id(owner: &str, generation: u64, account_id: &str, revision: u64) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.zulip.operational.client-realtime.v1\0");
    hash.update(owner.as_bytes());
    hash.update([0]);
    hash.update(generation.to_be_bytes());
    hash.update(account_id.as_bytes());
    hash.update([0]);
    hash.update(revision.to_be_bytes());
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
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
    use std::thread;

    #[test]
    fn projection_change_exposes_no_zulip_content_or_provider_identity() {
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
            let payload = ZulipOperationalProjectionChangedV1::decode(request.payload.as_slice())
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
            2,
            "local-account",
            3,
            1,
        )
        .expect("publish projection change");
        let (request, payload) = received.recv().expect("published request");
        assert_eq!(payload.account_id, "local-account");
        assert_eq!(payload.revision, 3);
        assert!(!request.cursor.contains("local-account"));
        assert_eq!(request.payload, payload.encode_to_vec());
        release_server.send(()).expect("release server");
        server.join().expect("server join");
    }
}
