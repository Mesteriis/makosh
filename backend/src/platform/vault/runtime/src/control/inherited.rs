//! Vault-side handshake over the Kernel-provided inherited Unix stream.

use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

const MAX_FRAME_BYTES: usize = 512 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

#[allow(dead_code)] // Used by the inherited-channel composition harness.
pub fn open_and_describe(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    let stream = duplicate_inherited_stream()?;
    describe(stream, descriptor_bytes, settings_schema_bytes)
}

pub fn describe(
    mut stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    stream
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| "Vault inherited control channel is unavailable".to_owned())?;
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
            descriptor_bytes,
            settings_schema_bytes,
        })),
    };
    write_frame(&mut stream, &request.encode_to_vec())?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|_| "Vault inherited control response is invalid".to_owned())?;
    match response.result {
        Some(ControlResult::Describe(describe))
            if response.error_code.is_empty()
                && !describe.registration_id.is_empty()
                && describe.runtime_generation != 0
                && describe.grant_epoch != 0 =>
        {
            let ready = ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1 {
                    registration_id: describe.registration_id,
                    runtime_generation: describe.runtime_generation,
                    grant_epoch: describe.grant_epoch,
                })),
            };
            write_frame(&mut stream, &ready.encode_to_vec())?;
            stream
                .set_read_timeout(None)
                .and_then(|_| stream.set_write_timeout(None))
                .map_err(|_| "Vault inherited control channel is unavailable".to_owned())?;
            Ok(stream)
        }
        _ => Err("Vault managed-runtime descriptor was rejected".to_owned()),
    }
}

pub fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "Vault inherited control frame is invalid".to_owned())?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err("Vault inherited control frame is invalid".to_owned());
    }
    let mut bytes = vec![0_u8; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|_| "Vault inherited control channel is unavailable".to_owned())?;
    Ok(bytes)
}

pub fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err("Vault inherited control frame is invalid".to_owned());
    }
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "Vault inherited control frame is invalid".to_owned())?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| "Vault inherited control channel is unavailable".to_owned())
}

#[allow(dead_code)] // Used only by `open_and_describe` in the composition harness.
fn duplicate_inherited_stream() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Vault inherited control channel is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated as RawFd) };
    Ok(stream)
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|_| "Vault inherited control channel is unavailable".to_owned())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("Vault inherited control frame is invalid".to_owned())
}
