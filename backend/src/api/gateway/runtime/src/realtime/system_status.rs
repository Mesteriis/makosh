use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, ClientSystemStatusChangedV1,
    client_realtime_frame_v1::Frame,
};
use makosh_gateway_session_contract::ClientSystemComponentStatusProjectionV1;
use prost::Message;

use crate::browser::system_status::wire_system_statuses;

pub(super) const SYSTEM_STATUS_CONTRACT_NAME: &str = "makosh.gateway.system-status";
pub(super) const SYSTEM_STATUS_EVENT_KIND: &str = "platform.system_status.changed";

pub(super) fn encoded_payload(
    statuses: &[ClientSystemComponentStatusProjectionV1],
    revision: u64,
) -> Vec<u8> {
    ClientSystemStatusChangedV1 {
        revision,
        statuses: wire_system_statuses(statuses),
    }
    .encode_to_vec()
}

pub(super) fn frame(
    revision: u64,
    occurred_at_unix_millis: u64,
    payload: Vec<u8>,
) -> ClientRealtimeFrameV1 {
    let identity = format!("gateway-system-status-{revision}");
    ClientRealtimeFrameV1 {
        frame: Some(Frame::Event(ClientRealtimeEventV1 {
            event_id: identity.as_bytes().to_vec(),
            cursor: identity,
            contract_name: SYSTEM_STATUS_CONTRACT_NAME.to_owned(),
            contract_version: 1,
            event_kind: SYSTEM_STATUS_EVENT_KIND.to_owned(),
            occurred_at_unix_millis,
            causation_id: String::new(),
            correlation_id: String::new(),
            trace_id: String::new(),
            payload,
        })),
    }
}
