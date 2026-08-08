//! Native host-only client for the private, Kernel-fenced WhatsApp bridge.

use std::{
    io::{Read, Write},
    os::unix::{fs::FileTypeExt, net::UnixStream},
    time::Instant,
};

use super::*;

use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_whatsapp_api::{
    WhatsAppProviderCommand,
    client_contract::{WHATSAPP_DESCRIPTOR_SET_V1, WHATSAPP_MODULE_ID, WHATSAPP_OWNER_ID},
    host_bridge::{
        HOST_BRIDGE_CONTRACT_MAJOR, HOST_BRIDGE_CONTRACT_NAME, HOST_BRIDGE_CONTRACT_REVISION,
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostBridgeEnvelopeV1,
        WhatsAppHostBridgeHandshakeV1, WhatsAppHostCommandClaimV1,
        decode_host_bridge_command_lease, decode_host_bridge_handshake_accepted,
        decode_host_bridge_observation_accepted, encode_host_bridge_handshake,
        encode_host_bridge_payload, encode_host_command_claim,
    },
};
use sha2::{Digest, Sha256};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

pub(super) struct WhatsAppHostBridgeTestClient {
    stream: UnixStream,
    next_request_id: u64,
}

impl WhatsAppHostBridgeTestClient {
    pub(super) fn connect(runtime: &StartedWhatsAppRuntime) -> Self {
        wait_for_host_socket(runtime);
        let mut stream = UnixStream::connect(&runtime.host_bridge_socket_path)
            .expect("connect exact WhatsApp host route");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(2))))
            .expect("bound WhatsApp host route deadlines");
        let handshake = encode_host_bridge_handshake(&WhatsAppHostBridgeHandshakeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            route_binding_sha256: runtime.route_binding_sha256,
        })
        .expect("encode exact WhatsApp host route handshake");
        write_frame(&mut stream, &handshake);
        decode_host_bridge_handshake_accepted(&read_frame(&mut stream))
            .expect("exact WhatsApp host route handshake");
        Self {
            stream,
            next_request_id: 1,
        }
    }

    pub(super) fn claim_commands(
        &mut self,
        account_id: &str,
        host_claim_id: &str,
    ) -> Vec<WhatsAppProviderCommand> {
        let payload = encode_host_command_claim(&WhatsAppHostCommandClaimV1 {
            account_id: account_id.to_owned(),
            host_claim_id: host_claim_id.to_owned(),
            lease_seconds: 30,
            limit: 8,
        })
        .expect("encode WhatsApp host command claim");
        decode_host_bridge_command_lease(&self.round_trip(payload))
            .expect("decode WhatsApp host command lease")
    }

    pub(super) fn submit_observation(&mut self, envelope: &WhatsAppHostBridgeEnvelopeV1) -> String {
        let payload =
            encode_host_bridge_payload(envelope).expect("encode WhatsApp host observation");
        decode_host_bridge_observation_accepted(&self.round_trip(payload))
            .expect("decode accepted WhatsApp host observation")
    }

    fn round_trip(&mut self, request_payload: Vec<u8>) -> Vec<u8> {
        let request_id = self.next_request_id;
        self.next_request_id = request_id.checked_add(1).expect("bounded host request id");
        let request = ModuleClientRequestV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            module_id: WHATSAPP_MODULE_ID.to_owned(),
            owner_id: WHATSAPP_OWNER_ID.to_owned(),
            contract: Some(host_bridge_contract()),
            request_id,
            request_payload,
            logical_owner_id: String::new(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
        };
        write_frame(&mut self.stream, &request.encode_to_vec());
        let response = ModuleClientResponseV1::decode(read_frame(&mut self.stream).as_slice())
            .expect("decode WhatsApp host response");
        assert_eq!(response.protocol_major, MODULE_CLIENT_PROTOCOL_MAJOR);
        assert_eq!(response.request_id, request_id);
        assert!(
            response.error_code.is_empty(),
            "WhatsApp host response must not expose an error payload",
        );
        response.response_payload
    }
}

fn wait_for_host_socket(runtime: &StartedWhatsAppRuntime) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(metadata) = std::fs::symlink_metadata(&runtime.host_bridge_socket_path)
            && metadata.file_type().is_socket()
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "managed WhatsApp host route did not become a Unix socket",
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn host_bridge_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: WHATSAPP_OWNER_ID.to_owned(),
        name: HOST_BRIDGE_CONTRACT_NAME.to_owned(),
        major: HOST_BRIDGE_CONTRACT_MAJOR,
        revision: HOST_BRIDGE_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(WHATSAPP_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) {
    let mut length = u32::try_from(bytes.len()).expect("bounded WhatsApp host frame");
    let mut prefix = [0_u8; 5];
    let mut index = 0;
    while length >= 0x80 {
        prefix[index] = (length as u8 & 0x7f) | 0x80;
        length >>= 7;
        index += 1;
    }
    prefix[index] = length as u8;
    stream
        .write_all(&prefix[..=index])
        .and_then(|_| stream.write_all(bytes))
        .expect("write WhatsApp host frame");
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .expect("read WhatsApp host frame length");
        length |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            let mut bytes = vec![0_u8; usize::try_from(length).expect("bounded host frame length")];
            stream
                .read_exact(&mut bytes)
                .expect("read WhatsApp host frame");
            return bytes;
        }
    }
    panic!("WhatsApp host frame length is invalid");
}
