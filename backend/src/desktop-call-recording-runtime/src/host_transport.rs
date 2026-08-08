use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;

use makosh_desktop_call_recording_api::host_bridge::{
    decode_handshake_v1, encode_handshake_accepted_v1,
};

use crate::{
    host_port::handle_host_operation_v1, managed_runtime::DesktopRecordingManagedRuntimeV1,
};

const MAX_FRAME_BYTES_V1: usize = 64 * 1024 * 1024 + 32 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopRecordingHostTransportErrorV1 {
    Closed,
    Frame,
    Handshake,
    Io,
    Port,
}

pub fn serve_one_operation_v1(
    mut stream: UnixStream,
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    handle: &tokio::runtime::Handle,
    now_unix_ms: i64,
) -> Result<(), DesktopRecordingHostTransportErrorV1> {
    stream
        .set_nonblocking(false)
        .map_err(|_| DesktopRecordingHostTransportErrorV1::Io)?;
    let handshake = decode_handshake_v1(&read_frame(&mut stream)?)
        .map_err(|_| DesktopRecordingHostTransportErrorV1::Handshake)?;
    if !runtime.accepts_host_route(&handshake.route_binding_sha256) {
        return Err(DesktopRecordingHostTransportErrorV1::Handshake);
    }
    write_frame(&mut stream, &encode_handshake_accepted_v1())?;
    let operation = read_frame(&mut stream)?;
    let response = handle
        .block_on(handle_host_operation_v1(runtime, &operation, now_unix_ms))
        .map_err(|error| {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_desktop_recording_host_port_error={error:?}");
            }
            DesktopRecordingHostTransportErrorV1::Port
        })?;
    write_frame(&mut stream, &response)
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, DesktopRecordingHostTransportErrorV1> {
    let length = read_length(stream)?;
    if length == 0 || length > MAX_FRAME_BYTES_V1 {
        return Err(DesktopRecordingHostTransportErrorV1::Frame);
    }
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            DesktopRecordingHostTransportErrorV1::Closed
        } else {
            DesktopRecordingHostTransportErrorV1::Io
        }
    })?;
    Ok(bytes)
}

fn write_frame(
    stream: &mut UnixStream,
    bytes: &[u8],
) -> Result<(), DesktopRecordingHostTransportErrorV1> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES_V1 {
        return Err(DesktopRecordingHostTransportErrorV1::Frame);
    }
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| DesktopRecordingHostTransportErrorV1::Frame)?;
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
        .map_err(|_| DesktopRecordingHostTransportErrorV1::Io)
}

fn read_length(stream: &mut UnixStream) -> Result<usize, DesktopRecordingHostTransportErrorV1> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0; 1];
        match stream.read_exact(&mut byte) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
                return Err(DesktopRecordingHostTransportErrorV1::Closed);
            }
            Err(_) => return Err(DesktopRecordingHostTransportErrorV1::Io),
        }
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return usize::try_from(value).map_err(|_| DesktopRecordingHostTransportErrorV1::Frame);
        }
    }
    Err(DesktopRecordingHostTransportErrorV1::Frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_limit_covers_exact_audio_contract_only() {
        assert_eq!(MAX_FRAME_BYTES_V1, 64 * 1024 * 1024 + 32 * 1024);
    }
}
