//! Descriptor-bound handshake over the Kernel-provided inherited FD.

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_vault_protocol::LeaseAudienceV1;
use prost::Message;

use super::framing::{read_frame, write_frame};

// Test composition exercises the descriptor handshake through the same
// inherited-channel path as the managed runtime. Production uses `authenticate`.
#[allow(dead_code)]
pub fn describe(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Storage inherited control channel is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
        .map(|(channel, _)| channel)
}

#[allow(dead_code)]
pub fn describe_on_channel(
    stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
        .map(|(channel, _)| channel)
}

pub(super) fn authenticate(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, ManagedStorageRuntimeIdentityV1), String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Storage inherited control channel is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
}

pub(super) fn authenticate_on_channel(
    mut stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, ManagedStorageRuntimeIdentityV1), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Storage inherited control channel is unavailable".to_owned())?;
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
            descriptor_bytes,
            settings_schema_bytes,
        })),
    };
    write_frame(&mut stream, &request.encode_to_vec())?;
    let response = ManagedRuntimeControlResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|_| "Storage inherited control response is invalid".to_owned())?;
    match response.result {
        Some(ControlResult::Describe(value))
            if response.error_code.is_empty()
                && !value.registration_id.is_empty()
                && value.runtime_generation != 0
                && value.grant_epoch != 0 =>
        {
            let audience = LeaseAudienceV1::new(
                value.registration_id,
                "storage-runtime".to_owned(),
                value.runtime_generation,
                value.grant_epoch,
            )
            .map_err(|_| "Storage managed-runtime descriptor was rejected".to_owned())?;
            Ok((stream, ManagedStorageRuntimeIdentityV1 { audience }))
        }
        _ => Err("Storage managed-runtime descriptor was rejected".to_owned()),
    }
}

#[derive(Clone)]
pub(super) struct ManagedStorageRuntimeIdentityV1 {
    audience: LeaseAudienceV1,
}

impl ManagedStorageRuntimeIdentityV1 {
    #[must_use]
    pub(super) const fn runtime_generation(&self) -> u64 {
        self.audience.runtime_generation()
    }

    #[must_use]
    pub(super) fn audience(&self) -> LeaseAudienceV1 {
        self.audience.clone()
    }
}
