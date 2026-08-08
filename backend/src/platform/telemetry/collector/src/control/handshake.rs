//! One-shot descriptor handshake over Kernel's inherited control FD.

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use super::framing::{read_frame, write_frame};

#[allow(dead_code)] // Used by the Collector inherited-channel composition harness.
pub fn describe(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Telemetry inherited control is unavailable".to_owned());
    }
    let mut stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Telemetry inherited control is unavailable".to_owned())?;
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
            descriptor_bytes,
            settings_schema_bytes,
        })),
    };
    write_frame(&mut stream, &request.encode_to_vec())?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|_| "Telemetry inherited control is invalid".to_owned())?;
    match response.result {
        Some(ControlResult::Describe(value))
            if response.error_code.is_empty()
                && !value.registration_id.is_empty()
                && value.runtime_generation != 0 =>
        {
            let ready = ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1 {
                    registration_id: value.registration_id,
                    runtime_generation: value.runtime_generation,
                    grant_epoch: value.grant_epoch,
                })),
            };
            write_frame(&mut stream, &ready.encode_to_vec())?;
            stream
                .set_read_timeout(None)
                .and_then(|_| stream.set_write_timeout(None))
                .map_err(|_| "Telemetry inherited control is unavailable".to_owned())?;
            Ok(stream)
        }
        _ => Err("Telemetry managed-runtime descriptor was rejected".to_owned()),
    }
}
