//! One-shot validation over the inherited private managed-runtime control FD.

use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::Stdio;
use std::sync::mpsc::SyncSender;
use std::time::{Duration, Instant};

use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleRegistration,
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlTransportErrorV2,
};
use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, ManagedRuntimeBlobCustodyDelegationDeliveryV1,
    ManagedRuntimeBlobCustodyDelegationRequestV1, ManagedRuntimeBlobCustodyReleaseDeliveryV1,
    ManagedRuntimeBlobCustodyReleaseRequestV1, ManagedRuntimeBlobSessionDeliveryV1,
    ManagedRuntimeBlobSessionRequestV1, ManagedRuntimeClientRealtimePublishRequestV1,
    ManagedRuntimeClientRealtimePublishResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeEventCredentialDeliveryV1,
    ManagedRuntimeEventCredentialRequestV1, ManagedRuntimeModuleQueryRequestV1,
    ManagedRuntimeModuleQueryResponseV1, ManagedRuntimeModuleRequestRequestV1,
    ManagedRuntimeModuleRequestResponseV1, ManagedRuntimeOwnerDerivedKeyDeliveryV1,
    ManagedRuntimeOwnerDerivedKeyRequestV1, ManagedRuntimeProviderCredentialDeliveryV1,
    ManagedRuntimeProviderCredentialRequestV1, VaultCiphertextResponseV1, VaultCiphertextRouteV1,
};
use makosh_runtime_protocol::validation::descriptor::{
    decode_descriptor_v1, decode_settings_schema_v1,
};
use makosh_runtime_protocol::validation::vault::validate_vault_ciphertext_route_v1;
use prost::Message;
use sha2::{Digest, Sha256};

const MAX_FRAME_BYTES: usize = 512 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);
const CORRELATED_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(60);

#[path = "control/inbound.rs"]
pub(crate) mod inbound;

pub trait ManagedRuntimeVaultRouteHandler: Send + Sync {
    fn route_vault_ciphertext(
        &self,
        expectation: &ManagedRuntimeExpectation,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, String>;
}

pub trait ManagedRuntimeEventCredentialHandler: Send + Sync {
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String>;
}

pub trait ManagedRuntimeProviderCredentialHandler: Send + Sync {
    fn issue_provider_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeProviderCredentialRequestV1,
    ) -> Result<ManagedRuntimeProviderCredentialDeliveryV1, String>;
}

pub trait ManagedRuntimeOwnerDerivedKeyHandler: Send + Sync {
    fn issue_owner_derived_key(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeOwnerDerivedKeyRequestV1,
    ) -> Result<ManagedRuntimeOwnerDerivedKeyDeliveryV1, String>;
}

pub trait ManagedRuntimeBlobSessionHandler: Send + Sync {
    fn issue_blob_session(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobSessionRequestV1,
    ) -> Result<ManagedRuntimeBlobSessionDeliveryV1, String>;

    fn delegate_blob_custody(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobCustodyDelegationRequestV1,
    ) -> Result<ManagedRuntimeBlobCustodyDelegationDeliveryV1, String>;
}

pub trait ManagedRuntimeBlobCustodyReleaseHandler: Send + Sync {
    fn release_blob_custody(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobCustodyReleaseRequestV1,
    ) -> Result<ManagedRuntimeBlobCustodyReleaseDeliveryV1, String>;
}

pub trait ManagedRuntimeModuleQueryHandler: Send + Sync {
    fn route_module_query(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeModuleQueryRequestV1,
    ) -> Result<ManagedRuntimeModuleQueryResponseV1, String>;
}

pub trait ManagedRuntimeModuleRequestHandler: Send + Sync {
    fn route_module_request(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeModuleRequestRequestV1,
    ) -> Result<ManagedRuntimeModuleRequestResponseV1, String>;
}

pub trait ManagedRuntimeClientRealtimeHandler: Send + Sync {
    fn publish_client_realtime(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeClientRealtimePublishRequestV1,
    ) -> Result<ManagedRuntimeClientRealtimePublishResponseV1, String>;
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ManagedRuntimeControlHandlers<'a> {
    pub vault_route: Option<&'a dyn ManagedRuntimeVaultRouteHandler>,
    pub event_credential: Option<&'a dyn ManagedRuntimeEventCredentialHandler>,
    pub provider_credential: Option<&'a dyn ManagedRuntimeProviderCredentialHandler>,
    pub owner_derived_key: Option<&'a dyn ManagedRuntimeOwnerDerivedKeyHandler>,
    pub blob_session: Option<&'a dyn ManagedRuntimeBlobSessionHandler>,
    pub blob_custody_release: Option<&'a dyn ManagedRuntimeBlobCustodyReleaseHandler>,
    pub module_query: Option<&'a dyn ManagedRuntimeModuleQueryHandler>,
    pub module_request: Option<&'a dyn ManagedRuntimeModuleRequestHandler>,
    pub client_realtime: Option<&'a dyn ManagedRuntimeClientRealtimeHandler>,
}

pub(crate) fn dispatch_typed_request(
    request: inbound::ManagedRuntimeInboundRequestV1,
    expectation: &ManagedRuntimeExpectation,
    handlers: ManagedRuntimeControlHandlers<'_>,
) -> Result<ManagedRuntimeControlResponseV1, String> {
    match request {
        inbound::ManagedRuntimeInboundRequestV1::Ready(_) => {
            Err("managed runtime ready requires lifecycle dispatch".to_owned())
        }
        inbound::ManagedRuntimeInboundRequestV1::VaultRoute(route) => {
            Ok(inbound::vault_route_response(
                handlers
                    .vault_route
                    .ok_or_else(|| "managed runtime Vault route is not available".to_owned())
                    .and_then(|handler| handler.route_vault_ciphertext(expectation, route)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::EventCredential(request) => {
            Ok(inbound::event_credential_response(
                handlers
                    .event_credential
                    .ok_or_else(|| {
                        "managed runtime Event credential handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.issue_event_credential(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::ProviderCredential(request) => {
            Ok(inbound::provider_credential_response(
                handlers
                    .provider_credential
                    .ok_or_else(|| {
                        "managed runtime provider credential handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.issue_provider_credential(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::OwnerDerivedKey(request) => {
            Ok(inbound::owner_derived_key_response(
                handlers
                    .owner_derived_key
                    .ok_or_else(|| {
                        "managed runtime owner-derived key handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.issue_owner_derived_key(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::BlobSession(request) => {
            Ok(inbound::blob_session_response(
                handlers
                    .blob_session
                    .ok_or_else(|| {
                        "managed runtime Blob session handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.issue_blob_session(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::BlobCustodyDelegation(request) => {
            Ok(inbound::blob_custody_delegation_response(
                handlers
                    .blob_session
                    .ok_or_else(|| {
                        "managed runtime Blob custody delegation handler is not available"
                            .to_owned()
                    })
                    .and_then(|handler| handler.delegate_blob_custody(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::BlobCustodyRelease(request) => {
            Ok(inbound::blob_custody_release_response(
                handlers
                    .blob_custody_release
                    .ok_or_else(|| {
                        "managed runtime Blob custody release handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.release_blob_custody(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::ModuleQuery(request) => {
            Ok(inbound::module_query_response(
                handlers
                    .module_query
                    .ok_or_else(|| {
                        "managed runtime module query handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.route_module_query(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::ModuleRequest(request) => {
            Ok(inbound::module_request_response(
                handlers
                    .module_request
                    .ok_or_else(|| {
                        "managed runtime module request handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.route_module_request(expectation, request)),
            ))
        }
        inbound::ManagedRuntimeInboundRequestV1::ClientRealtime(request) => {
            Ok(inbound::client_realtime_response(
                handlers
                    .client_realtime
                    .ok_or_else(|| {
                        "managed runtime ClientRealtime handler is not available".to_owned()
                    })
                    .and_then(|handler| handler.publish_client_realtime(expectation, request)),
            ))
        }
    }
}

pub struct ManagedRuntimeRelayRequest {
    payload: Vec<u8>,
    response: SyncSender<Result<Vec<u8>, String>>,
}

impl ManagedRuntimeRelayRequest {
    pub fn new(payload: Vec<u8>, response: SyncSender<Result<Vec<u8>, String>>) -> Self {
        Self { payload, response }
    }

    pub(crate) fn into_parts(self) -> (Vec<u8>, SyncSender<Result<Vec<u8>, String>>) {
        (self.payload, self.response)
    }

    pub fn dispatch(
        self,
        channel: &mut UnixStream,
        expectation: &ManagedRuntimeExpectation,
        handlers: ManagedRuntimeControlHandlers<'_>,
    ) {
        let _ = self.response.send(relay_with_control_routes(
            channel,
            &self.payload,
            expectation,
            handlers,
        ));
    }
}

#[derive(Debug)]
pub struct ManagedRuntimeExpectation {
    registration_id: String,
    runtime_instance_id: String,
    module_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    descriptor_sha256: [u8; 32],
    settings_schema_sha256: Option<[u8; 32]>,
}

impl ManagedRuntimeExpectation {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        runtime_instance_id: impl Into<String>,
        module_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        descriptor_sha256: [u8; 32],
        settings_schema_sha256: Option<[u8; 32]>,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            runtime_instance_id: runtime_instance_id.into(),
            module_id: module_id.into(),
            runtime_generation,
            grant_epoch,
            descriptor_sha256,
            settings_schema_sha256,
        }
    }

    pub fn from_fenced_launch(
        registration: &ModuleRegistration,
        binding: &BundledManagedLaunchBinding,
        record: &ManagedLaunchRecord,
    ) -> Result<Self, String> {
        if registration.registration_id() != binding.registration_id()
            || registration.registration_id() != record.registration_id()
            || registration.descriptor_sha256() != binding.descriptor_sha256()
            || binding.binding_revision() != record.binding_revision()
            || registration.grant_epoch() != record.grant_epoch()
            || record.runtime_generation() == 0
        {
            return Err("managed launch fence does not match its approved registration".to_owned());
        }
        Ok(Self::new(
            registration.registration_id(),
            record.runtime_instance_id(),
            registration.module_id(),
            record.runtime_generation(),
            record.grant_epoch(),
            *binding.descriptor_sha256(),
            binding.settings_schema_sha256().copied(),
        ))
    }

    pub fn from_platform_fenced_launch(
        process_id: &str,
        module_id: &str,
        binding: &PlatformManagedProcessBinding,
        launch: &PlatformManagedProcessLaunch,
    ) -> Result<Self, String> {
        if process_id != binding.process_id()
            || process_id != launch.process_id()
            || binding.binding_revision() != launch.binding_revision()
            || launch.runtime_generation() == 0
        {
            return Err("platform managed launch fence does not match its binding".to_owned());
        }
        Ok(Self::new(
            process_id,
            process_id,
            module_id,
            launch.runtime_generation(),
            launch.grant_epoch(),
            *binding.descriptor_sha256(),
            binding.settings_schema_sha256().copied(),
        ))
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn module_id(&self) -> &str {
        &self.module_id
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

    #[must_use]
    pub fn matches_ready(
        &self,
        ready: &makosh_runtime_protocol::v1::ManagedRuntimeReadyRequestV1,
    ) -> bool {
        ready.registration_id == self.registration_id
            && ready.runtime_generation == self.runtime_generation
            && ready.grant_epoch == self.grant_epoch
    }
}

pub fn create_inherited_channel() -> Result<(UnixStream, Stdio), String> {
    let (kernel_end, child_end) = UnixStream::pair().map_err(|error| error.to_string())?;
    let child_fd: OwnedFd = child_end.into();
    Ok((kernel_end, Stdio::from(child_fd)))
}

pub fn establish_channel(
    mut stream: UnixStream,
    expectation: &ManagedRuntimeExpectation,
) -> Result<UnixStream, String> {
    stream
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|error| error.to_string())?;
    let result = read_frame(&mut stream)
        .and_then(|bytes| {
            ManagedRuntimeControlRequestV1::decode(bytes.as_slice())
                .map_err(|_| "managed runtime control request is invalid".to_owned())
        })
        .and_then(|request| validate_describe(request, expectation));
    let response = match result {
        Ok(()) => ManagedRuntimeControlResponseV1 {
            result: Some(
                makosh_runtime_protocol::v1::managed_runtime_control_response_v1::Result::Describe(
                    DescribeManagedRuntimeResponseV1 {
                        registration_id: expectation.registration_id.clone(),
                        runtime_generation: expectation.runtime_generation,
                        grant_epoch: expectation.grant_epoch,
                    },
                ),
            ),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_runtime_describe_rejected".to_owned(),
        },
    };
    write_frame(&mut stream, &response.encode_to_vec())?;
    if result.is_ok() {
        stream
            .set_read_timeout(None)
            .and_then(|_| stream.set_write_timeout(None))
            .map_err(|error| error.to_string())?;
    }
    result.map(|()| stream)
}

pub fn establish_correlated_channel(
    stream: UnixStream,
    expectation: &ManagedRuntimeExpectation,
) -> Result<ManagedControlChannelV2<UnixStream>, String> {
    stream
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|error| error.to_string())?;
    tracing::debug!(
        event = "managed_runtime.control.launch_binding",
        registration.id = %expectation.registration_id,
        module.id = %expectation.module_id,
        runtime.instance_id = %expectation.runtime_instance_id,
        runtime.generation = expectation.runtime_generation,
        grant.epoch = expectation.grant_epoch,
        binding.descriptor_sha256 = ?expectation.descriptor_sha256,
        binding.settings_schema_sha256 = ?expectation.settings_schema_sha256,
    );
    let mut channel = ManagedControlChannelV2::new(stream);
    let (correlation_id, request) = receive_correlated_request_until(
        &mut channel,
        Instant::now() + CORRELATED_HANDSHAKE_TIMEOUT,
    )
    .map_err(|error| {
        log_correlated_control_error("managed_runtime.control.describe.receive_failed", &error);
        "managed runtime correlated control request is invalid".to_owned()
    })?;
    log_sanitized_control_request(correlation_id, &request);
    validate_describe(request, expectation).map_err(|error| {
        tracing::error!(
            event = "managed_runtime.control.describe.validation_failed",
            error.class = "describe_validation",
            error.message = %error,
        );
        error
    })?;
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: Some(
                    makosh_runtime_protocol::v1::managed_runtime_control_response_v1::Result::Describe(
                        DescribeManagedRuntimeResponseV1 {
                            registration_id: expectation.registration_id.clone(),
                            runtime_generation: expectation.runtime_generation,
                            grant_epoch: expectation.grant_epoch,
                        },
                    ),
                ),
                error_code: String::new(),
            },
        )
        .map_err(|error| {
            log_correlated_control_error(
                "managed_runtime.control.describe.response_failed",
                &error,
            );
            "managed runtime correlated control response is invalid".to_owned()
        })?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|error| error.to_string())?;
    Ok(channel)
}

fn receive_correlated_request_until(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    deadline: Instant,
) -> Result<
    (
        [u8; makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES],
        ManagedRuntimeControlRequestV1,
    ),
    ManagedControlTransportErrorV2,
>{
    loop {
        match channel.receive_request() {
            Err(ManagedControlTransportErrorV2::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) && Instant::now() < deadline =>
            {
                tracing::debug!(
                    event = "managed_runtime.control.describe.waiting",
                    control.deadline_remaining_millis = u64::try_from(
                        deadline
                            .saturating_duration_since(Instant::now())
                            .as_millis(),
                    )
                    .unwrap_or(u64::MAX),
                );
            }
            result => return result,
        }
    }
}

fn log_sanitized_control_request(
    correlation_id: [u8; makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES],
    request: &ManagedRuntimeControlRequestV1,
) {
    use makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation;

    match request.operation.as_ref() {
        Some(Operation::Describe(describe)) => {
            let descriptor = decode_descriptor_v1(&describe.descriptor_bytes);
            let settings_schema = if describe.settings_schema_bytes.is_empty() {
                Ok(None)
            } else {
                decode_settings_schema_v1(&describe.settings_schema_bytes).map(Some)
            };
            tracing::debug!(
                event = "managed_runtime.control.request.received",
                control.operation = "describe",
                control.correlation_id = ?correlation_id,
                payload.descriptor_bytes = describe.descriptor_bytes.len(),
                payload.settings_schema_bytes = describe.settings_schema_bytes.len(),
                payload.descriptor = ?descriptor,
                payload.settings_schema = ?settings_schema,
            );
            if descriptor.is_err() || settings_schema.is_err() {
                tracing::debug!(
                    event = "managed_runtime.control.request.invalid_payload_bytes",
                    control.operation = "describe",
                    control.correlation_id = ?correlation_id,
                    payload.descriptor = ?describe.descriptor_bytes,
                    payload.settings_schema = ?describe.settings_schema_bytes,
                );
            }
        }
        Some(_) => tracing::debug!(
            event = "managed_runtime.control.request.received",
            control.operation = "unexpected",
            control.correlation_id = ?correlation_id,
        ),
        None => tracing::debug!(
            event = "managed_runtime.control.request.received",
            control.operation = "missing",
            control.correlation_id = ?correlation_id,
        ),
    }
}

fn log_correlated_control_error(event: &'static str, error: &ManagedControlTransportErrorV2) {
    if let ManagedControlTransportErrorV2::Io(io_error) = error {
        tracing::error!(
            event = event,
            error.class = managed_control_transport_error_class(error),
            error.io_kind = ?io_error.kind(),
            error.os_code = ?io_error.raw_os_error(),
            error.message = %io_error,
        );
    } else {
        tracing::error!(
            event = event,
            error.class = managed_control_transport_error_class(error),
        );
    }
}

const fn managed_control_transport_error_class(
    error: &ManagedControlTransportErrorV2,
) -> &'static str {
    match error {
        ManagedControlTransportErrorV2::InvalidTransportSelection => "invalid_transport_selection",
        ManagedControlTransportErrorV2::InvalidCorrelationId => "invalid_correlation_id",
        ManagedControlTransportErrorV2::InvalidFrame => "invalid_frame",
        ManagedControlTransportErrorV2::FrameTooLarge => "frame_too_large",
        ManagedControlTransportErrorV2::Io(_) => "io",
        ManagedControlTransportErrorV2::UnexpectedResponse => "unexpected_response",
        ManagedControlTransportErrorV2::UnexpectedRequest => "unexpected_request",
        ManagedControlTransportErrorV2::DuplicateCorrelationId => "duplicate_correlation_id",
        ManagedControlTransportErrorV2::PendingRequestLimit => "pending_request_limit",
        ManagedControlTransportErrorV2::PeerClosed => "peer_closed",
    }
}

pub fn relay(channel: &mut UnixStream, payload: &[u8]) -> Result<Vec<u8>, String> {
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err("managed runtime relay payload is invalid".to_owned());
    }
    write_frame(channel, payload)?;
    let response = read_frame(channel)?;
    if response.is_empty() {
        return Err("managed runtime relay response is invalid".to_owned());
    }
    Ok(response)
}

pub(crate) fn relay_with_vault_routes(
    channel: &mut UnixStream,
    payload: &[u8],
    expectation: &ManagedRuntimeExpectation,
    vault_route_handler: Option<&dyn ManagedRuntimeVaultRouteHandler>,
) -> Result<Vec<u8>, String> {
    relay_with_control_routes(
        channel,
        payload,
        expectation,
        ManagedRuntimeControlHandlers {
            vault_route: vault_route_handler,
            ..ManagedRuntimeControlHandlers::default()
        },
    )
}

pub(crate) fn relay_with_control_routes(
    channel: &mut UnixStream,
    payload: &[u8],
    expectation: &ManagedRuntimeExpectation,
    handlers: ManagedRuntimeControlHandlers<'_>,
) -> Result<Vec<u8>, String> {
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err("managed runtime relay payload is invalid".to_owned());
    }
    write_frame(channel, payload)?;
    loop {
        let frame = read_frame(channel)?;
        if let Some(route) = vault_route(&frame) {
            let result = handlers
                .vault_route
                .ok_or_else(|| "managed runtime Vault route is not available".to_owned())?
                .route_vault_ciphertext(expectation, route);
            inbound::respond_vault_route(channel, result)?;
            continue;
        }
        if let Ok(Some(request)) = inbound::event_credential_request(&frame) {
            let result = handlers
                .event_credential
                .ok_or_else(|| {
                    "managed runtime Event credential handler is not available".to_owned()
                })?
                .issue_event_credential(expectation, request);
            inbound::respond_event_credential(channel, result)?;
            continue;
        }
        if let Ok(Some(request)) = inbound::provider_credential_request(&frame) {
            let result = handlers
                .provider_credential
                .ok_or_else(|| {
                    "managed runtime provider credential handler is not available".to_owned()
                })?
                .issue_provider_credential(expectation, request);
            inbound::respond_provider_credential(channel, result)?;
            continue;
        }
        if let Ok(Some(request)) = inbound::owner_derived_key_request(&frame) {
            let result = handlers
                .owner_derived_key
                .ok_or_else(|| {
                    "managed runtime owner-derived key handler is not available".to_owned()
                })?
                .issue_owner_derived_key(expectation, request);
            inbound::respond_owner_derived_key(channel, result)?;
            continue;
        }
        if let Ok(Some(request)) = inbound::blob_session_request(&frame) {
            let result = handlers
                .blob_session
                .ok_or_else(|| "managed runtime Blob session handler is not available".to_owned())?
                .issue_blob_session(expectation, request);
            inbound::respond_blob_session(channel, result)?;
            continue;
        }
        if let Ok(Some(request)) = inbound::blob_custody_delegation_request(&frame) {
            let result = handlers
                .blob_session
                .ok_or_else(|| {
                    "managed runtime Blob custody delegation handler is not available".to_owned()
                })?
                .delegate_blob_custody(expectation, request);
            inbound::respond_blob_custody_delegation(channel, result)?;
            continue;
        }
        if let Ok(Some(request)) = inbound::blob_custody_release_request(&frame) {
            let result = handlers
                .blob_custody_release
                .ok_or_else(|| {
                    "managed runtime Blob custody release handler is not available".to_owned()
                })?
                .release_blob_custody(expectation, request);
            inbound::respond_blob_custody_release(channel, result)?;
            continue;
        }
        return Ok(frame);
    }
}

fn vault_route(frame: &[u8]) -> Option<VaultCiphertextRouteV1> {
    let route = ManagedRuntimeControlRequestV1::decode(frame)
        .ok()?
        .operation
        .and_then(|operation| match operation {
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::RouteVaultCiphertext(request) => request.route,
            _ => None,
        })?;
    validate_vault_ciphertext_route_v1(&route).ok()?;
    Some(route)
}

fn validate_describe(
    request: ManagedRuntimeControlRequestV1,
    expectation: &ManagedRuntimeExpectation,
) -> Result<(), String> {
    use makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation;

    let Some(Operation::Describe(describe)) = request.operation else {
        return Err("managed runtime control request is invalid".to_owned());
    };
    if Sha256::digest(&describe.descriptor_bytes).as_slice() != expectation.descriptor_sha256 {
        return Err("managed runtime descriptor digest does not match launch binding".to_owned());
    }
    let descriptor = decode_descriptor_v1(&describe.descriptor_bytes)
        .map_err(|_| "managed runtime descriptor is invalid".to_owned())?;
    if descriptor.module_id != expectation.module_id {
        return Err(
            "managed runtime descriptor module identity does not match launch binding".to_owned(),
        );
    }
    match expectation.settings_schema_sha256 {
        Some(expected_digest) => {
            if Sha256::digest(&describe.settings_schema_bytes).as_slice() != expected_digest {
                return Err(
                    "managed runtime settings schema digest does not match launch binding"
                        .to_owned(),
                );
            }
            decode_settings_schema_v1(&describe.settings_schema_bytes)
                .map_err(|_| "managed runtime settings schema is invalid".to_owned())?;
        }
        None if !describe.settings_schema_bytes.is_empty() => {
            return Err("managed runtime settings schema is not bound for this launch".to_owned());
        }
        None => {}
    }
    Ok(())
}

fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "managed runtime control frame is too large".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("managed runtime control frame is too large".to_owned());
    }
    let mut bytes = vec![0_u8; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("managed runtime control frame length is invalid".to_owned())
}

fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "managed runtime control response is too large".to_owned())?;
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .map_err(|error| error.to_string())?;
        length >>= 7;
    }
    stream
        .write_all(&[length as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod correlated_handshake_timeout_tests {
    use super::*;
    use makosh_runtime_protocol::v1::{
        DescribeManagedRuntimeRequestV1, managed_runtime_control_request_v1::Operation,
    };

    #[test]
    fn transient_read_timeout_does_not_abort_the_correlated_handshake() {
        let (kernel_stream, child_stream) = UnixStream::pair().expect("control pair");
        kernel_stream
            .set_read_timeout(Some(Duration::from_millis(5)))
            .expect("short read poll");
        let child = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            ManagedControlChannelV2::new(child_stream)
                .write_request(
                    [7_u8;
                        makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES],
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::Describe(
                            DescribeManagedRuntimeRequestV1::default(),
                        )),
                    },
                )
                .expect("delayed describe request");
        });
        let mut channel = ManagedControlChannelV2::new(kernel_stream);

        let (correlation_id, request) = receive_correlated_request_until(
            &mut channel,
            Instant::now() + Duration::from_millis(250),
        )
        .expect("correlated request after transient timeout");

        assert_eq!(correlation_id, [7_u8; 16]);
        assert!(matches!(request.operation, Some(Operation::Describe(_))));
        child.join().expect("delayed child");
    }
}
