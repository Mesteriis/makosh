//! One-shot Scheduler NATS credential acquisition on its inherited Kernel FD.

use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::Duration,
};

use makosh_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingInputV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialDeliveryV1, NatsRuntimeCredentialRecipientV1, RuntimeNatsJwtCredentialV1,
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeEventCredentialRequestV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::vault_request_id::next_vault_transport_request_id_v1;
use prost::Message;

const MAX_FRAME_BYTES: usize = 512 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Obtains a short-lived NATS JWT only from the authenticated Kernel relay.
pub fn request_runtime_credential(
    channel: &mut UnixStream,
    logical_owner_id: &str,
    registration_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    credential_revision: u64,
) -> Result<RuntimeNatsJwtCredentialV1, SchedulerNatsCredentialErrorV1> {
    let request_id = request_id()?;
    let recipient = NatsRuntimeCredentialRecipientV1::generate();
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(Operation::IssueEventCredential(
            ManagedRuntimeEventCredentialRequestV1 {
                request_id: request_id.to_vec(),
                credential_revision,
                ttl_seconds: 300,
                recipient_public_key_x25519: recipient.public_key().as_bytes().to_vec(),
            },
        )),
    };
    channel
        .set_read_timeout(Some(REQUEST_TIMEOUT))
        .and_then(|_| channel.set_write_timeout(Some(REQUEST_TIMEOUT)))
        .map_err(|_| SchedulerNatsCredentialErrorV1::Unavailable)?;
    write_frame(channel, &request.encode_to_vec())?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(channel)?.as_slice())
        .map_err(|_| SchedulerNatsCredentialErrorV1::Rejected)?;
    let delivery = delivery(response)?;
    let binding =
        NatsRuntimeCredentialDeliveryBindingV1::new(NatsRuntimeCredentialDeliveryBindingInputV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            registration_id: registration_id.to_owned(),
            runtime_instance_id: runtime_instance_id.to_owned(),
            runtime_generation,
            grant_epoch,
            credential_revision,
            request_id,
            recipient_public_key: recipient.public_key().clone(),
        })
        .map_err(|_| SchedulerNatsCredentialErrorV1::Rejected)?;
    recipient
        .open(&binding, &delivery)
        .map_err(|_| SchedulerNatsCredentialErrorV1::Rejected)
}

fn delivery(
    response: ManagedRuntimeControlResponseV1,
) -> Result<NatsRuntimeCredentialDeliveryV1, SchedulerNatsCredentialErrorV1> {
    let Some(ResponseResult::EventCredentialDelivery(value)) = response.result else {
        return Err(SchedulerNatsCredentialErrorV1::Rejected);
    };
    if !response.error_code.is_empty() {
        return Err(SchedulerNatsCredentialErrorV1::Rejected);
    }
    NatsRuntimeCredentialDeliveryV1::from_parts(value.encapped_key, value.ciphertext, value.tag)
        .map_err(|_| SchedulerNatsCredentialErrorV1::Rejected)
}

fn request_id() -> Result<[u8; 16], SchedulerNatsCredentialErrorV1> {
    next_vault_transport_request_id_v1().ok_or(SchedulerNatsCredentialErrorV1::Unavailable)
}

fn write_frame(
    channel: &mut UnixStream,
    bytes: &[u8],
) -> Result<(), SchedulerNatsCredentialErrorV1> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(SchedulerNatsCredentialErrorV1::Rejected);
    }
    let mut frame = Vec::with_capacity(bytes.len() + 5);
    encode_length(bytes.len(), &mut frame);
    frame.extend_from_slice(bytes);
    channel
        .write_all(&frame)
        .map_err(|_| SchedulerNatsCredentialErrorV1::Unavailable)
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, SchedulerNatsCredentialErrorV1> {
    let length = read_length(channel)?;
    let mut bytes = vec![0_u8; length];
    channel
        .read_exact(&mut bytes)
        .map_err(|_| SchedulerNatsCredentialErrorV1::Unavailable)?;
    Ok(bytes)
}

fn read_length(channel: &mut UnixStream) -> Result<usize, SchedulerNatsCredentialErrorV1> {
    let mut value = 0_usize;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel
            .read_exact(&mut byte)
            .map_err(|_| SchedulerNatsCredentialErrorV1::Unavailable)?;
        value |= usize::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 && (1..=MAX_FRAME_BYTES).contains(&value) {
            return Ok(value);
        }
    }
    Err(SchedulerNatsCredentialErrorV1::Rejected)
}

fn encode_length(mut value: usize, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerNatsCredentialErrorV1 {
    Rejected,
    Unavailable,
}
