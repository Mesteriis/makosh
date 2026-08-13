//! Couples a fresh inherited control FD to each bounded managed-child attempt.

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::runtime::lifecycle::control::{
    self as managed_runtime_control, ManagedRuntimeBlobCustodyReleaseHandler,
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeEventCredentialHandler,
    ManagedRuntimeExpectation, ManagedRuntimeOwnerDerivedKeyHandler,
    ManagedRuntimeProviderCredentialHandler,
};
use crate::runtime::managed::execution::{
    self as bounded_managed_child_execution, ManagedChildExecutionPolicy,
    ManagedChildExecutionResult,
};
use std::net::Shutdown;
use std::process::{Child, ExitStatus};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::time::{Duration, Instant};

use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlTransportMajorV1,
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlAckV1, ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;
use prost::Message;

pub fn run(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    expectation: &ManagedRuntimeExpectation,
    policy: &ManagedChildExecutionPolicy,
) -> Result<ManagedChildExecutionResult, String> {
    run_with_wait(
        staged_executable,
        arguments,
        expectation,
        policy,
        |child, _| {
            bounded_managed_child_execution::wait(child, policy.max_runtime())
                .map(ManagedChildAttemptOutcomeV1::Exited)
        },
    )
}

pub struct ManagedChildRunInput<'a> {
    pub staged_executable: &'a StagedNativeArtifact,
    pub arguments: &'a [String],
    pub expectation: &'a ManagedRuntimeExpectation,
    pub policy: &'a ManagedChildExecutionPolicy,
    pub control_transport: ManagedControlTransportMajorV1,
    pub shutdown_requested: &'a AtomicBool,
    pub stop_requested: &'a AtomicBool,
    pub relay_requests: &'a Receiver<managed_runtime_control::ManagedRuntimeRelayRequest>,
    pub control_handlers: managed_runtime_control::ManagedRuntimeControlHandlers<'a>,
    pub ready_sender: &'a SyncSender<Result<(), String>>,
    pub ready_state: &'a AtomicBool,
}

pub fn run_until_shutdown(
    input: ManagedChildRunInput<'_>,
) -> Result<ManagedChildExecutionResult, String> {
    match input.control_transport {
        ManagedControlTransportMajorV1::LegacyV1 => run_with_wait(
            input.staged_executable,
            input.arguments,
            input.expectation,
            input.policy,
            |child, channel| wait_until_shutdown_with_relay(child, channel, &input),
        ),
        ManagedControlTransportMajorV1::CorrelatedV2 => run_with_wait_v2(
            input.staged_executable,
            input.arguments,
            input.expectation,
            input.policy,
            |child, channel| wait_until_shutdown_with_correlated_relay(child, channel, &input),
        ),
    }
}

fn run_with_wait<F>(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    expectation: &ManagedRuntimeExpectation,
    policy: &ManagedChildExecutionPolicy,
    mut wait: F,
) -> Result<ManagedChildExecutionResult, String>
where
    F: FnMut(
        &mut Child,
        &mut std::os::unix::net::UnixStream,
    ) -> Result<ManagedChildAttemptOutcomeV1, String>,
{
    for attempt in 1..=policy.max_attempts() {
        let (kernel_end, child_stdin) = managed_runtime_control::create_inherited_channel()?;
        let mut child =
            bounded_managed_child_execution::spawn(staged_executable, arguments, child_stdin)?;
        let mut control_channel =
            match managed_runtime_control::establish_channel(kernel_end, expectation) {
                Ok(channel) => channel,
                Err(error) => {
                    let _ = bounded_managed_child_execution::terminate(&mut child);
                    if attempt == policy.max_attempts() {
                        return Err(error);
                    }
                    continue;
                }
            };
        match wait(&mut child, &mut control_channel)? {
            ManagedChildAttemptOutcomeV1::RequestedStop(status) => {
                return Ok(ManagedChildExecutionResult::succeeded(
                    attempt,
                    status.code().unwrap_or(0),
                ));
            }
            ManagedChildAttemptOutcomeV1::ControlFault(error) => {
                if attempt == policy.max_attempts() {
                    return Err(error);
                }
            }
            ManagedChildAttemptOutcomeV1::Exited(status) if status.success() => {
                return Ok(ManagedChildExecutionResult::succeeded(
                    attempt,
                    status.code().unwrap_or(0),
                ));
            }
            ManagedChildAttemptOutcomeV1::Exited(_) => {}
        }
    }
    Err("managed child exhausted its bounded restart attempts".to_owned())
}

fn run_with_wait_v2<F>(
    staged_executable: &StagedNativeArtifact,
    arguments: &[String],
    expectation: &ManagedRuntimeExpectation,
    policy: &ManagedChildExecutionPolicy,
    mut wait: F,
) -> Result<ManagedChildExecutionResult, String>
where
    F: FnMut(
        &mut Child,
        &mut ManagedControlChannelV2<std::os::unix::net::UnixStream>,
    ) -> Result<ManagedChildAttemptOutcomeV1, String>,
{
    for attempt in 1..=policy.max_attempts() {
        let (kernel_end, child_stdin) = managed_runtime_control::create_inherited_channel()?;
        let mut child =
            bounded_managed_child_execution::spawn(staged_executable, arguments, child_stdin)?;
        let mut control_channel =
            match managed_runtime_control::establish_correlated_channel(kernel_end, expectation) {
                Ok(channel) => channel,
                Err(error) => {
                    let _ = bounded_managed_child_execution::terminate(&mut child);
                    if attempt == policy.max_attempts() {
                        return Err(error);
                    }
                    continue;
                }
            };
        match wait(&mut child, &mut control_channel)? {
            ManagedChildAttemptOutcomeV1::RequestedStop(status) => {
                return Ok(ManagedChildExecutionResult::succeeded(
                    attempt,
                    status.code().unwrap_or(0),
                ));
            }
            ManagedChildAttemptOutcomeV1::ControlFault(error) => {
                if attempt == policy.max_attempts() {
                    return Err(error);
                }
            }
            ManagedChildAttemptOutcomeV1::Exited(status) if status.success() => {
                return Ok(ManagedChildExecutionResult::succeeded(
                    attempt,
                    status.code().unwrap_or(0),
                ));
            }
            ManagedChildAttemptOutcomeV1::Exited(_) => {}
        }
    }
    Err("managed child exhausted its bounded restart attempts".to_owned())
}

fn wait_until_shutdown_with_relay(
    child: &mut Child,
    channel: &mut std::os::unix::net::UnixStream,
    input: &ManagedChildRunInput<'_>,
) -> Result<ManagedChildAttemptOutcomeV1, String> {
    loop {
        if input
            .shutdown_requested
            .load(std::sync::atomic::Ordering::Acquire)
            || input
                .stop_requested
                .load(std::sync::atomic::Ordering::Acquire)
        {
            channel
                .shutdown(Shutdown::Both)
                .map_err(|error| error.to_string())?;
            return terminal_status_after_control_close(child)
                .map(ManagedChildAttemptOutcomeV1::RequestedStop);
        }
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Ok(ManagedChildAttemptOutcomeV1::ControlFault(
                "managed runtime exited without a Kernel stop request".to_owned(),
            ));
        }
        let ready = match managed_runtime_control::inbound::try_receive_ready(channel) {
            Ok(ready) => ready,
            Err(error) => return unexpected_control_fault(child, error),
        };
        if let Some(ready) = ready {
            if !input.expectation.matches_ready(&ready) {
                let _ = input
                    .ready_sender
                    .try_send(Err("managed runtime ready signal is stale".to_owned()));
                return Err("managed runtime ready signal is stale".to_owned());
            }
            input
                .ready_state
                .store(true, std::sync::atomic::Ordering::Release);
            let _ = input.ready_sender.try_send(Ok(()));
            continue;
        }
        match process_typed_requests(channel, input.expectation, input.control_handlers) {
            Ok(true) => continue,
            Ok(false) => {}
            Err(error) => {
                return unexpected_control_fault(child, error);
            }
        }
        match input.relay_requests.recv_timeout(Duration::from_millis(25)) {
            Ok(request) => request.dispatch(channel, input.expectation, input.control_handlers),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                bounded_managed_child_execution::terminate(child)?;
                return Err("managed runtime relay was disconnected".to_owned());
            }
        }
    }
}

fn wait_until_shutdown_with_correlated_relay(
    child: &mut Child,
    channel: &mut ManagedControlChannelV2<std::os::unix::net::UnixStream>,
    input: &ManagedChildRunInput<'_>,
) -> Result<ManagedChildAttemptOutcomeV1, String> {
    channel
        .inner_mut()
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    loop {
        if input
            .shutdown_requested
            .load(std::sync::atomic::Ordering::Acquire)
            || input
                .stop_requested
                .load(std::sync::atomic::Ordering::Acquire)
        {
            channel
                .inner_mut()
                .shutdown(Shutdown::Both)
                .map_err(|error| error.to_string())?;
            return terminal_status_after_control_close(child)
                .map(ManagedChildAttemptOutcomeV1::RequestedStop);
        }
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Ok(ManagedChildAttemptOutcomeV1::ControlFault(
                "managed runtime exited without a Kernel stop request".to_owned(),
            ));
        }
        match channel.try_receive_request() {
            Ok(Some((correlation_id, request))) => {
                dispatch_v2_typed_request(channel, correlation_id, request, input)?;
                continue;
            }
            Ok(None) => {}
            Err(_) => {
                return unexpected_control_fault(
                    child,
                    "managed runtime correlated control channel failed".to_owned(),
                );
            }
        }
        match input.relay_requests.recv_timeout(Duration::from_millis(25)) {
            Ok(request) => dispatch_correlated_relay(channel, request, input),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                bounded_managed_child_execution::terminate(child)?;
                return Err("managed runtime relay was disconnected".to_owned());
            }
        }
    }
}

fn dispatch_correlated_relay(
    channel: &mut ManagedControlChannelV2<std::os::unix::net::UnixStream>,
    relay: managed_runtime_control::ManagedRuntimeRelayRequest,
    input: &ManagedChildRunInput<'_>,
) {
    let (payload, response_sender) = relay.into_parts();
    let response = (|| {
        let request = ManagedRuntimeControlRequestV1::decode(payload.as_slice())
            .map_err(|_| "managed runtime V2 relay request is invalid".to_owned())?;
        let response_kind = CorrelatedRelayResponseKindV1::from_request(&request)?;
        channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|error| error.to_string())?;
        let response = channel.request_next(request, |channel, correlation_id, request| {
            dispatch_v2_typed_request(channel, correlation_id, request, input).map_err(|_| {
                makosh_runtime_protocol::managed_control::ManagedControlTransportErrorV2::InvalidFrame
            })
        });
        let restore = channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|error| error.to_string());
        restore?;
        let response =
            response.map_err(|_| "managed runtime V2 relay response is invalid".to_owned())?;
        response_kind
            .matches(&response)
            .then_some(response.encode_to_vec())
            .ok_or_else(|| "managed runtime V2 relay response kind is invalid".to_owned())
    })();
    let _ = response_sender.send(response);
}

#[derive(Clone, Copy)]
enum CorrelatedRelayResponseKindV1 {
    Client,
    ModuleQuery,
    ModuleRequest,
}

impl CorrelatedRelayResponseKindV1 {
    fn from_request(request: &ManagedRuntimeControlRequestV1) -> Result<Self, String> {
        use makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation;

        match request.operation.as_ref() {
            Some(Operation::ClientDelivery(_)) => Ok(Self::Client),
            Some(Operation::DeliverModuleQuery(_)) => Ok(Self::ModuleQuery),
            Some(Operation::DeliverModuleRequest(_)) => Ok(Self::ModuleRequest),
            _ => Err("managed runtime V2 relay operation is prohibited".to_owned()),
        }
    }

    fn matches(self, response: &ManagedRuntimeControlResponseV1) -> bool {
        matches!(
            (self, response.result.as_ref()),
            (Self::Client, Some(ControlResult::ClientDelivery(_)))
                | (
                    Self::ModuleQuery,
                    Some(ControlResult::ModuleQueryDelivery(_))
                )
                | (
                    Self::ModuleRequest,
                    Some(ControlResult::ModuleRequestDelivery(_))
                )
        )
    }
}

fn dispatch_v2_typed_request(
    channel: &mut ManagedControlChannelV2<std::os::unix::net::UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    request: ManagedRuntimeControlRequestV1,
    input: &ManagedChildRunInput<'_>,
) -> Result<(), String> {
    let request = managed_runtime_control::inbound::decode_typed_request(request)?;
    match request {
        managed_runtime_control::inbound::ManagedRuntimeInboundRequestV1::Ready(ready) => {
            if !input.expectation.matches_ready(&ready) {
                let _ = input
                    .ready_sender
                    .try_send(Err("managed runtime ready signal is stale".to_owned()));
                return Err("managed runtime ready signal is stale".to_owned());
            }
            channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .map_err(|_| "managed runtime correlated control response is invalid".to_owned())?;
            input
                .ready_state
                .store(true, std::sync::atomic::Ordering::Release);
            let _ = input.ready_sender.try_send(Ok(()));
            Ok(())
        }
        request => {
            let response = managed_runtime_control::dispatch_typed_request(
                request,
                input.expectation,
                input.control_handlers,
            )?;
            channel
                .write_response(correlation_id, response)
                .map_err(|_| "managed runtime correlated control response is invalid".to_owned())
        }
    }
}

fn process_typed_requests(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handlers: managed_runtime_control::ManagedRuntimeControlHandlers<'_>,
) -> Result<bool, String> {
    if let Some(route) = managed_runtime_control::inbound::try_receive_vault_route(channel)? {
        let result = handlers
            .vault_route
            .ok_or_else(|| "managed runtime Vault route is not available".to_owned())?
            .route_vault_ciphertext(expectation, route);
        managed_runtime_control::inbound::respond_vault_route(channel, result)?;
        return Ok(true);
    }
    if let Some(request) = managed_runtime_control::inbound::try_receive_event_credential(channel)?
    {
        dispatch_event_credential(channel, expectation, handlers.event_credential, request)?;
        return Ok(true);
    }
    if let Some(request) =
        managed_runtime_control::inbound::try_receive_provider_credential(channel)?
    {
        dispatch_provider_credential(channel, expectation, handlers.provider_credential, request)?;
        return Ok(true);
    }
    if let Some(request) = managed_runtime_control::inbound::try_receive_owner_derived_key(channel)?
    {
        dispatch_owner_derived_key(channel, expectation, handlers.owner_derived_key, request)?;
        return Ok(true);
    }
    if let Some(request) = managed_runtime_control::inbound::try_receive_blob_session(channel)? {
        dispatch_blob_session(channel, expectation, handlers.blob_session, request)?;
        return Ok(true);
    }
    if let Some(request) =
        managed_runtime_control::inbound::try_receive_blob_custody_delegation(channel)?
    {
        dispatch_blob_custody_delegation(channel, expectation, handlers.blob_session, request)?;
        return Ok(true);
    }
    if let Some(request) =
        managed_runtime_control::inbound::try_receive_blob_custody_release(channel)?
    {
        dispatch_blob_custody_release(
            channel,
            expectation,
            handlers.blob_custody_release,
            request,
        )?;
        return Ok(true);
    }
    Ok(false)
}

fn dispatch_blob_custody_delegation(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handler: Option<&dyn ManagedRuntimeBlobSessionHandler>,
    request: makosh_runtime_protocol::v1::ManagedRuntimeBlobCustodyDelegationRequestV1,
) -> Result<(), String> {
    let result = handler
        .ok_or_else(|| "managed runtime Blob custody delegation route is not available".to_owned())?
        .delegate_blob_custody(expectation, request);
    managed_runtime_control::inbound::respond_blob_custody_delegation(channel, result)
}

fn dispatch_blob_custody_release(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handler: Option<&dyn ManagedRuntimeBlobCustodyReleaseHandler>,
    request: makosh_runtime_protocol::v1::ManagedRuntimeBlobCustodyReleaseRequestV1,
) -> Result<(), String> {
    let result = handler
        .ok_or_else(|| "managed runtime Blob custody release route is not available".to_owned())?
        .release_blob_custody(expectation, request);
    managed_runtime_control::inbound::respond_blob_custody_release(channel, result)
}

fn dispatch_blob_session(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handler: Option<&dyn ManagedRuntimeBlobSessionHandler>,
    request: makosh_runtime_protocol::v1::ManagedRuntimeBlobSessionRequestV1,
) -> Result<(), String> {
    let result = handler
        .ok_or_else(|| "managed runtime Blob session route is not available".to_owned())?
        .issue_blob_session(expectation, request);
    managed_runtime_control::inbound::respond_blob_session(channel, result)
}

fn dispatch_provider_credential(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handler: Option<&dyn ManagedRuntimeProviderCredentialHandler>,
    request: makosh_runtime_protocol::v1::ManagedRuntimeProviderCredentialRequestV1,
) -> Result<(), String> {
    let result = handler
        .ok_or_else(|| "managed runtime provider credential route is not available".to_owned())?
        .issue_provider_credential(expectation, request);
    managed_runtime_control::inbound::respond_provider_credential(channel, result)
}

fn dispatch_owner_derived_key(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handler: Option<&dyn ManagedRuntimeOwnerDerivedKeyHandler>,
    request: makosh_runtime_protocol::v1::ManagedRuntimeOwnerDerivedKeyRequestV1,
) -> Result<(), String> {
    let result = handler
        .ok_or_else(|| "managed runtime owner-derived key route is not available".to_owned())?
        .issue_owner_derived_key(expectation, request);
    managed_runtime_control::inbound::respond_owner_derived_key(channel, result)
}

fn dispatch_event_credential(
    channel: &mut std::os::unix::net::UnixStream,
    expectation: &ManagedRuntimeExpectation,
    handler: Option<&dyn ManagedRuntimeEventCredentialHandler>,
    request: makosh_runtime_protocol::v1::ManagedRuntimeEventCredentialRequestV1,
) -> Result<(), String> {
    let result = handler
        .ok_or_else(|| "managed runtime Event credential route is not available".to_owned())?
        .issue_event_credential(expectation, request);
    managed_runtime_control::inbound::respond_event_credential(channel, result)
}

fn terminal_status_after_control_close(child: &mut Child) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(status) = child.try_wait().map_err(|failure| failure.to_string())? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            child.kill().map_err(|failure| failure.to_string())?;
            return child.wait().map_err(|failure| failure.to_string());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn unexpected_control_fault(
    child: &mut Child,
    error: String,
) -> Result<ManagedChildAttemptOutcomeV1, String> {
    bounded_managed_child_execution::terminate(child)?;
    Ok(ManagedChildAttemptOutcomeV1::ControlFault(error))
}

enum ManagedChildAttemptOutcomeV1 {
    Exited(ExitStatus),
    RequestedStop(ExitStatus),
    ControlFault(String),
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{atomic::AtomicBool, mpsc};

    use makosh_runtime_protocol::v1::{
        DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1, ModuleDescriptorV1,
        ModuleKindV1, managed_runtime_control_request_v1::Operation,
    };
    use prost::Message;
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn requested_stop_kills_unresponsive_child_once_without_replacement() {
        let root = std::env::temp_dir().join(format!(
            "makosh-managed-stop-no-restart-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("fixture root");
        let launches = root.join("launches");
        let (descriptor_bytes, framed_describe) = describe_fixture();
        let source = root.join("child.sh");
        std::fs::write(
            &source,
            format!(
                "#!/bin/sh\nprintf x >> \"$1\"\nprintf '{}' >&0\nwhile :; do sleep 1; done\n",
                shell_octal(&framed_describe)
            ),
        )
        .expect("child fixture");
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o500))
            .expect("fixture executable");
        let digest: [u8; 32] = Sha256::digest(std::fs::read(&source).expect("child bytes")).into();
        let staged = crate::distribution::staged_artifact::stage(
            &source,
            &root.join("staged"),
            "managed-child",
            &digest,
        )
        .expect("staged fixture");
        let expectation = ManagedRuntimeExpectation::new(
            "registration",
            "runtime",
            "persons",
            1,
            1,
            Sha256::digest(&descriptor_bytes).into(),
            None,
        );
        let policy = ManagedChildExecutionPolicy::new(3, Duration::from_secs(5)).expect("policy");
        let shutdown = AtomicBool::new(false);
        let stop = AtomicBool::new(true);
        let (_relay_sender, relay_receiver) = mpsc::sync_channel(1);
        let (ready_sender, _ready_receiver) = mpsc::sync_channel(1);
        let ready_state = AtomicBool::new(false);
        let started = Instant::now();
        let result = run_until_shutdown(ManagedChildRunInput {
            staged_executable: &staged,
            arguments: &[launches.display().to_string()],
            expectation: &expectation,
            policy: &policy,
            control_transport: ManagedControlTransportMajorV1::LegacyV1,
            shutdown_requested: &shutdown,
            stop_requested: &stop,
            relay_requests: &relay_receiver,
            control_handlers: managed_runtime_control::ManagedRuntimeControlHandlers::default(),
            ready_sender: &ready_sender,
            ready_state: &ready_state,
        })
        .expect("requested stop is terminal");
        assert_eq!(result.attempts(), 1);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(
            std::fs::read(&launches).expect("launch counter"),
            b"x",
            "killed child must not be replaced"
        );
        std::fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn malformed_correlated_control_frame_is_not_a_successful_requested_stop() {
        assert_unexpected_correlated_control_failure("printf '\\001\\377' >&0");
    }

    #[test]
    fn unexpected_correlated_peer_close_is_not_a_successful_requested_stop() {
        assert_unexpected_correlated_control_failure("exec 0>&-");
    }

    #[test]
    fn malformed_correlated_control_frame_then_exit_zero_is_still_a_fault() {
        assert_unexpected_correlated_control_failure("printf '\\001\\377' >&0; exit 0");
    }

    #[test]
    fn unexpected_correlated_peer_close_then_exit_zero_is_still_a_fault() {
        assert_unexpected_correlated_control_failure("exec 0>&-; exit 0");
    }

    fn assert_unexpected_correlated_control_failure(action: &str) {
        let root = std::env::temp_dir().join(format!(
            "makosh-managed-control-failure-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("fixture root");
        let launches = root.join("launches");
        let (descriptor_bytes, framed_describe) = correlated_describe_fixture();
        let source = root.join("child.sh");
        std::fs::write(
            &source,
            format!(
                "#!/bin/sh\nprintf x >> \"$1\"\nprintf '{}' >&0\nsleep 1\n{action}\nwhile :; do sleep 1; done\n",
                shell_octal(&framed_describe)
            ),
        )
        .expect("child fixture");
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o500))
            .expect("fixture executable");
        let digest: [u8; 32] = Sha256::digest(std::fs::read(&source).expect("child bytes")).into();
        let staged = crate::distribution::staged_artifact::stage(
            &source,
            &root.join("staged"),
            "managed-control-failure-child",
            &digest,
        )
        .expect("staged fixture");
        let expectation = ManagedRuntimeExpectation::new(
            "registration",
            "runtime",
            "persons",
            1,
            1,
            Sha256::digest(&descriptor_bytes).into(),
            None,
        );
        let policy = ManagedChildExecutionPolicy::new(2, Duration::from_secs(5)).expect("policy");
        let shutdown = AtomicBool::new(false);
        let stop = AtomicBool::new(false);
        let (_relay_sender, relay_receiver) = mpsc::sync_channel(1);
        let (ready_sender, _ready_receiver) = mpsc::sync_channel(1);
        let ready_state = AtomicBool::new(false);
        let result = run_until_shutdown(ManagedChildRunInput {
            staged_executable: &staged,
            arguments: &[launches.display().to_string()],
            expectation: &expectation,
            policy: &policy,
            control_transport: ManagedControlTransportMajorV1::CorrelatedV2,
            shutdown_requested: &shutdown,
            stop_requested: &stop,
            relay_requests: &relay_receiver,
            control_handlers: managed_runtime_control::ManagedRuntimeControlHandlers::default(),
            ready_sender: &ready_sender,
            ready_state: &ready_state,
        });
        assert!(
            result.is_err(),
            "unexpected control failure must not succeed"
        );
        assert_eq!(
            std::fs::read(&launches).expect("launch counter"),
            b"xx",
            "unexpected control failure follows bounded retry policy"
        );
        std::fs::remove_dir_all(root).expect("remove fixture");
    }

    fn describe_fixture() -> (Vec<u8>, Vec<u8>) {
        let descriptor_bytes = ModuleDescriptorV1 {
            descriptor_major: 1,
            descriptor_revision: 1,
            module_kind: ModuleKindV1::Domain as i32,
            module_id: "persons".to_owned(),
            owner_id: "persons".to_owned(),
            module_version: "1".to_owned(),
            build_id: "test".to_owned(),
            ..Default::default()
        }
        .encode_to_vec();
        let request = ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
                descriptor_bytes: descriptor_bytes.clone(),
                settings_schema_bytes: Vec::new(),
            })),
        }
        .encode_to_vec();
        assert!(request.len() < 128);
        let mut framed = vec![request.len() as u8];
        framed.extend_from_slice(&request);
        (descriptor_bytes, framed)
    }

    fn correlated_describe_fixture() -> (Vec<u8>, Vec<u8>) {
        let (descriptor_bytes, _) = describe_fixture();
        let request = ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
                descriptor_bytes: descriptor_bytes.clone(),
                settings_schema_bytes: Vec::new(),
            })),
        };
        let (writer, mut reader) = std::os::unix::net::UnixStream::pair().expect("control pair");
        let mut channel = ManagedControlChannelV2::new(writer);
        channel
            .write_request([1; MANAGED_CONTROL_CORRELATION_ID_BYTES], request)
            .expect("correlated describe");
        drop(channel);
        let mut framed = Vec::new();
        reader
            .read_to_end(&mut framed)
            .expect("read correlated describe frame");
        (descriptor_bytes, framed)
    }

    fn shell_octal(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("\\{byte:03o}")).collect()
    }
}
