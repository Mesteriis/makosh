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
        |child, _| bounded_managed_child_execution::wait(child, policy.max_runtime()),
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
    F: FnMut(&mut Child, &mut std::os::unix::net::UnixStream) -> Result<ExitStatus, String>,
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
        let status = wait(&mut child, &mut control_channel)?;
        if status.success() {
            return Ok(ManagedChildExecutionResult::succeeded(
                attempt,
                status.code().unwrap_or(0),
            ));
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
    ) -> Result<ExitStatus, String>,
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
        let status = wait(&mut child, &mut control_channel)?;
        if status.success() {
            return Ok(ManagedChildExecutionResult::succeeded(
                attempt,
                status.code().unwrap_or(0),
            ));
        }
    }
    Err("managed child exhausted its bounded restart attempts".to_owned())
}

fn wait_until_shutdown_with_relay(
    child: &mut Child,
    channel: &mut std::os::unix::net::UnixStream,
    input: &ManagedChildRunInput<'_>,
) -> Result<ExitStatus, String> {
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Ok(status);
        }
        if input
            .shutdown_requested
            .load(std::sync::atomic::Ordering::Acquire)
            || input
                .stop_requested
                .load(std::sync::atomic::Ordering::Acquire)
        {
            bounded_managed_child_execution::terminate(child)?;
            return Err("managed child stopped by Kernel shutdown".to_owned());
        }
        if let Some(ready) = managed_runtime_control::inbound::try_receive_ready(channel)? {
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
            Err(_) => return terminal_status_after_control_close(child),
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
) -> Result<ExitStatus, String> {
    channel
        .inner_mut()
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Ok(status);
        }
        if input
            .shutdown_requested
            .load(std::sync::atomic::Ordering::Acquire)
            || input
                .stop_requested
                .load(std::sync::atomic::Ordering::Acquire)
        {
            bounded_managed_child_execution::terminate(child)?;
            return Err("managed child stopped by Kernel shutdown".to_owned());
        }
        match channel.try_receive_request() {
            Ok(Some((correlation_id, request))) => {
                dispatch_v2_typed_request(channel, correlation_id, request, input)?;
                continue;
            }
            Ok(None) => {}
            Err(_) => return terminal_status_after_control_close(child),
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
