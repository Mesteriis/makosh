//! Descriptor-bound identity handshake on the inherited Kernel FD.

use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use super::framing::{read_frame, write_frame};

pub(crate) struct EventsAuthorityRuntimeIdentityV1 {
    registration_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

pub(crate) fn authenticate(
    stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, EventsAuthorityRuntimeIdentityV1), String> {
    let mut stream = stream;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Events authority inherited channel is unavailable".to_owned())?;
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
            descriptor_bytes,
            settings_schema_bytes,
        })),
    };
    write_frame(&mut stream, &request.encode_to_vec())?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|_| "Events authority inherited response is invalid".to_owned())?;
    match response.result {
        Some(ControlResult::Describe(value))
            if response.error_code.is_empty()
                && valid_id(&value.registration_id)
                && value.runtime_generation > 0
                && value.grant_epoch > 0 =>
        {
            Ok((
                stream,
                EventsAuthorityRuntimeIdentityV1 {
                    registration_id: value.registration_id,
                    runtime_generation: value.runtime_generation,
                    grant_epoch: value.grant_epoch,
                },
            ))
        }
        _ => Err("Events authority managed-runtime descriptor was rejected".to_owned()),
    }
}

impl EventsAuthorityRuntimeIdentityV1 {
    pub(crate) fn registration_id(&self) -> &str {
        &self.registration_id
    }
    pub(crate) const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
    pub(crate) const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
