//! Native-host fixture for the one-operation, private desktop recording bridge.

use std::{
    io::{Read, Write},
    os::unix::{fs::FileTypeExt, net::UnixStream},
    time::Instant,
};

use super::*;

use makosh_desktop_call_recording_api::{
    HOST_PROTOCOL_MAJOR_V1, HOST_PROTOCOL_REVISION_V1,
    wire::{
        DesktopRecordingHostCommandClaimV1, DesktopRecordingHostCommandLeaseV1,
        DesktopRecordingHostCommandV1, DesktopRecordingHostHandshakeAcceptedV1,
        DesktopRecordingHostHandshakeV1, DesktopRecordingHostObservationAcceptedV1,
        DesktopRecordingHostObservationV1, DesktopRecordingHostOperationV1,
        desktop_recording_host_operation_v1::Operation,
    },
};

pub(super) fn claim_desktop_recording_commands_v1(
    runtime: &StartedDesktopRecordingRuntimeV1,
    host_claim_id: [u8; 16],
) -> Vec<DesktopRecordingHostCommandV1> {
    let response = desktop_recording_host_round_trip_v1(
        runtime,
        DesktopRecordingHostOperationV1 {
            operation: Some(Operation::ClaimCommands(
                DesktopRecordingHostCommandClaimV1 {
                    host_claim_id: host_claim_id.to_vec(),
                    lease_seconds: 30,
                    limit: 8,
                },
            )),
        },
    );
    DesktopRecordingHostCommandLeaseV1::decode(response.as_slice())
        .expect("decode desktop recording host command lease")
        .commands
}

pub(super) fn submit_desktop_recording_observation_v1(
    runtime: &StartedDesktopRecordingRuntimeV1,
    observation: DesktopRecordingHostObservationV1,
) -> DesktopRecordingHostObservationAcceptedV1 {
    let response = desktop_recording_host_round_trip_v1(
        runtime,
        DesktopRecordingHostOperationV1 {
            operation: Some(Operation::Observation(observation)),
        },
    );
    DesktopRecordingHostObservationAcceptedV1::decode(response.as_slice())
        .expect("decode accepted desktop recording host observation")
}

pub(super) fn assert_desktop_recording_observation_rejected_v1(
    runtime: &StartedDesktopRecordingRuntimeV1,
    observation: DesktopRecordingHostObservationV1,
) {
    wait_for_desktop_recording_host_socket_v1(runtime);
    let mut stream = authenticated_host_stream_v1(runtime);
    write_frame(
        &mut stream,
        &DesktopRecordingHostOperationV1 {
            operation: Some(Operation::Observation(observation)),
        }
        .encode_to_vec(),
    );
    let mut byte = [0_u8; 1];
    assert_eq!(
        stream
            .read(&mut byte)
            .expect("read rejected host operation"),
        0,
        "rejected native observation must close its one-operation connection",
    );
}

pub(super) fn assert_stale_desktop_recording_host_route_rejected_v1(
    stale_runtime: &StartedDesktopRecordingRuntimeV1,
    current_runtime: &StartedDesktopRecordingRuntimeV1,
) {
    wait_for_desktop_recording_host_socket_v1(current_runtime);
    let mut stream = UnixStream::connect(&current_runtime.host_bridge_socket_path)
        .expect("connect current desktop recording host socket with stale binding");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(3))))
        .expect("bound stale host route deadlines");
    write_frame(
        &mut stream,
        &DesktopRecordingHostHandshakeV1 {
            protocol_major: HOST_PROTOCOL_MAJOR_V1,
            protocol_revision: HOST_PROTOCOL_REVISION_V1,
            route_binding_sha256: stale_runtime.route_binding_sha256.to_vec(),
        }
        .encode_to_vec(),
    );
    let mut byte = [0_u8; 1];
    assert_eq!(
        stream.read(&mut byte).expect("read stale host handshake"),
        0,
        "stale runtime generation must not authenticate to the successor host route",
    );
}

pub(super) fn canonical_recording_wav_v1(sample_count: u32) -> Vec<u8> {
    assert!(sample_count > 0, "recording WAV needs samples");
    let data_bytes = sample_count.checked_mul(2).expect("bounded WAV data");
    let riff_size = 36_u32.checked_add(data_bytes).expect("bounded RIFF size");
    let mut wav = Vec::with_capacity(44 + usize::try_from(data_bytes).expect("bounded WAV data"));
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_size.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&16_000_u32.to_le_bytes());
    wav.extend_from_slice(&32_000_u32.to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in 0..sample_count {
        let value = i16::try_from(sample % 257).expect("bounded sample") - 128;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

fn desktop_recording_host_round_trip_v1(
    runtime: &StartedDesktopRecordingRuntimeV1,
    operation: DesktopRecordingHostOperationV1,
) -> Vec<u8> {
    wait_for_desktop_recording_host_socket_v1(runtime);
    let mut stream = authenticated_host_stream_v1(runtime);
    write_frame(&mut stream, &operation.encode_to_vec());
    read_frame(&mut stream)
}

fn authenticated_host_stream_v1(runtime: &StartedDesktopRecordingRuntimeV1) -> UnixStream {
    let mut stream = UnixStream::connect(&runtime.host_bridge_socket_path)
        .expect("connect exact desktop recording host route");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(3))))
        .expect("bound desktop recording host route deadlines");
    write_frame(
        &mut stream,
        &DesktopRecordingHostHandshakeV1 {
            protocol_major: HOST_PROTOCOL_MAJOR_V1,
            protocol_revision: HOST_PROTOCOL_REVISION_V1,
            route_binding_sha256: runtime.route_binding_sha256.to_vec(),
        }
        .encode_to_vec(),
    );
    let accepted =
        DesktopRecordingHostHandshakeAcceptedV1::decode(read_frame(&mut stream).as_slice())
            .expect("decode desktop recording host handshake acceptance");
    assert_eq!(accepted.protocol_major, HOST_PROTOCOL_MAJOR_V1);
    assert_eq!(accepted.protocol_revision, HOST_PROTOCOL_REVISION_V1);
    stream
}

fn wait_for_desktop_recording_host_socket_v1(runtime: &StartedDesktopRecordingRuntimeV1) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(metadata) = std::fs::symlink_metadata(&runtime.host_bridge_socket_path)
            && metadata.file_type().is_socket()
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "managed desktop recording host route did not become a Unix socket",
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) {
    let mut length = u32::try_from(bytes.len()).expect("bounded desktop recording host frame");
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
        .and_then(|_| stream.flush())
        .expect("write desktop recording host frame");
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .expect("read desktop recording host frame length");
        length |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            let mut bytes = vec![
                0_u8;
                usize::try_from(length)
                    .expect("bounded desktop recording frame length")
            ];
            stream
                .read_exact(&mut bytes)
                .expect("read desktop recording host frame");
            return bytes;
        }
    }
    panic!("desktop recording host frame length is invalid");
}

#[test]
fn canonical_recording_fixture_is_mono_pcm_16khz() {
    let wav = canonical_recording_wav_v1(16_000);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 16_000);
    assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16);
}
