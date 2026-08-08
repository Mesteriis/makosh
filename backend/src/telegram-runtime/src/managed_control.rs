//! Telegram identity handshake over the platform-owned correlated control channel.

use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use makosh_runtime_protocol::v1::ManagedRuntimeReadyRequestV1;

const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn with_blocking_control_channel<T>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    request: impl FnOnce(&mut ManagedControlChannelV2<UnixStream>) -> T,
) -> Result<T, ()> {
    channel
        .inner_mut()
        .set_nonblocking(false)
        .and_then(|_| channel.inner_mut().set_read_timeout(Some(CONTROL_TIMEOUT)))
        .and_then(|_| channel.inner_mut().set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|_| ())?;
    let result = request(channel);
    let read_timeout_cleared = channel.inner_mut().set_read_timeout(None).is_ok();
    let write_timeout_cleared = channel.inner_mut().set_write_timeout(None).is_ok();
    let nonblocking_restored = channel.inner_mut().set_nonblocking(true).is_ok();
    if !read_timeout_cleared || !write_timeout_cleared || !nonblocking_restored {
        return Err(());
    }
    Ok(result)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramManagedRuntimeIdentity {
    registration_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl TelegramManagedRuntimeIdentity {
    pub fn open_inherited(
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        runtime_instance_id: impl Into<String>,
    ) -> Result<(Self, ManagedControlChannelV2<UnixStream>), String> {
        let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
        if duplicated < 0 {
            return Err("Telegram managed-runtime channel is unavailable".to_owned());
        }
        let channel = unsafe { UnixStream::from_raw_fd(duplicated as RawFd) };
        Self::authenticate(
            channel,
            descriptor_bytes,
            settings_schema_bytes,
            runtime_instance_id,
        )
    }

    pub fn authenticate(
        channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        runtime_instance_id: impl Into<String>,
    ) -> Result<(Self, ManagedControlChannelV2<UnixStream>), String> {
        if descriptor_bytes.is_empty() || settings_schema_bytes.is_empty() {
            return Err("Telegram managed-runtime descriptor is empty".to_owned());
        }
        let runtime_instance_id = runtime_instance_id.into();
        if runtime_instance_id.trim().is_empty() {
            return Err("Telegram managed-runtime instance id is empty".to_owned());
        }
        let mut channel = ManagedControlChannelV2::new(channel);
        channel
            .inner_mut()
            .set_read_timeout(Some(CONTROL_TIMEOUT))
            .and_then(|_| channel.inner_mut().set_write_timeout(Some(CONTROL_TIMEOUT)))
            .map_err(|_| "Telegram managed-runtime channel is unavailable".to_owned())?;
        let response = channel
            .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
            .map_err(|_| "Telegram managed-runtime descriptor was rejected".to_owned())?;
        let registration_id = response.registration_id;
        let runtime_generation = response.runtime_generation;
        let grant_epoch = response.grant_epoch;
        Ok((
            Self {
                registration_id,
                runtime_instance_id,
                runtime_generation,
                grant_epoch,
            },
            channel,
        ))
    }

    pub fn signal_ready(
        &self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
    ) -> Result<(), String> {
        channel
            .signal_ready(ManagedRuntimeReadyRequestV1 {
                registration_id: self.registration_id.clone(),
                runtime_generation: self.runtime_generation,
                grant_epoch: self.grant_epoch,
            })
            .map_err(|_| "Telegram managed-runtime readiness was rejected".to_owned())?;
        channel
            .inner_mut()
            .set_read_timeout(None)
            .and_then(|_| channel.inner_mut().set_write_timeout(None))
            .map_err(|_| "Telegram managed-runtime channel is unavailable".to_owned())?;
        Ok(())
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

#[cfg(test)]
mod tests {
    use std::io::{ErrorKind, Read, Write};
    use std::os::unix::net::UnixStream;
    use std::thread;

    use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
    use makosh_runtime_protocol::v1::{
        DescribeManagedRuntimeResponseV1, ManagedRuntimeControlAckV1,
        ManagedRuntimeControlResponseV1, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    };

    use super::{TelegramManagedRuntimeIdentity, with_blocking_control_channel};

    #[test]
    fn authenticates_with_the_exact_correlated_control_transport() {
        let (client, server) = UnixStream::pair().expect("control pair");
        let (release_server, wait_for_client) = std::sync::mpsc::sync_channel(0);
        let server = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(server);
            let (describe_id, describe) = channel.receive_request().expect("describe request");
            assert!(matches!(describe.operation, Some(Operation::Describe(_))));
            channel
                .write_response(
                    describe_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Describe(DescribeManagedRuntimeResponseV1 {
                            registration_id: "telegram".to_owned(),
                            runtime_generation: 7,
                            grant_epoch: 11,
                        })),
                        error_code: String::new(),
                    },
                )
                .expect("describe response");
            let (ready_id, ready) = channel.receive_request().expect("ready request");
            assert!(matches!(ready.operation, Some(Operation::Ready(_))));
            channel
                .write_response(
                    ready_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .expect("ready response");
            wait_for_client.recv().expect("client completed");
        });

        let (identity, mut channel): (
            TelegramManagedRuntimeIdentity,
            ManagedControlChannelV2<UnixStream>,
        ) = TelegramManagedRuntimeIdentity::authenticate(
            client,
            vec![1],
            vec![2],
            "telegram-runtime-7",
        )
        .expect("managed identity");
        assert_eq!(identity.registration_id(), "telegram");
        assert_eq!(identity.runtime_generation(), 7);
        assert_eq!(identity.grant_epoch(), 11);
        identity
            .signal_ready(&mut channel)
            .expect("managed runtime ready");
        release_server.send(()).expect("release server");
        server.join().expect("server join");
    }

    #[test]
    fn restores_nonblocking_mode_after_a_nested_control_request() {
        let (client, mut server) = UnixStream::pair().expect("control pair");
        client.set_nonblocking(true).expect("nonblocking client");
        let (release_server, wait_for_client) = std::sync::mpsc::sync_channel(0);
        let server = thread::spawn(move || {
            let mut request = [0_u8; 1];
            server.read_exact(&mut request).expect("request");
            assert_eq!(request, [7]);
            server.write_all(&[9]).expect("response");
            wait_for_client.recv().expect("client completed");
        });
        let mut channel = ManagedControlChannelV2::new(client);

        let response = with_blocking_control_channel(&mut channel, |channel| {
            channel.inner_mut().write_all(&[7]).expect("request");
            let mut response = [0_u8; 1];
            channel
                .inner_mut()
                .read_exact(&mut response)
                .expect("response");
            response
        })
        .expect("blocking request");

        assert_eq!(response, [9]);
        let mut pending = [0_u8; 1];
        assert_eq!(
            channel
                .inner_mut()
                .read(&mut pending)
                .expect_err("nonblocking stream")
                .kind(),
            ErrorKind::WouldBlock,
        );
        release_server.send(()).expect("release server");
        server.join().expect("server join");
    }
}
