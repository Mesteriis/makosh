//! Descriptor-bound authentication on the inherited Kernel FD.

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use super::framing::{read_frame, write_frame};

pub(super) struct SchedulerRuntimeIdentity {
    registration_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

pub(super) fn authenticate(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, SchedulerRuntimeIdentity), String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Scheduler inherited control channel is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
}

fn authenticate_on_channel(
    mut stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, SchedulerRuntimeIdentity), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Scheduler inherited control channel is unavailable".to_owned())?;
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
            descriptor_bytes,
            settings_schema_bytes,
        })),
    };
    write_frame(&mut stream, &request.encode_to_vec())?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|_| "Scheduler inherited control response is invalid".to_owned())?;
    match response.result {
        Some(ControlResult::Describe(value))
            if response.error_code.is_empty()
                && !value.registration_id.is_empty()
                && value.runtime_generation != 0
                && value.grant_epoch != 0 =>
        {
            Ok((
                stream,
                SchedulerRuntimeIdentity {
                    registration_id: value.registration_id,
                    runtime_generation: value.runtime_generation,
                    grant_epoch: value.grant_epoch,
                },
            ))
        }
        _ => Err("Scheduler managed-runtime descriptor was rejected".to_owned()),
    }
}

impl SchedulerRuntimeIdentity {
    #[must_use]
    pub(super) fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub(super) const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub(super) const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}
