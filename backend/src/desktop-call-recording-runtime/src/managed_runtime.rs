use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::prelude::{FileTypeExt, PermissionsExt};

use makosh_desktop_call_recording_api::OWNER_ID_V1;
use makosh_desktop_call_recording_persistence::{
    DesktopCallRecordingRepositoryV1, PersistenceErrorV1,
};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    request_managed_runtime_event_access_v2,
};
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    },
    v1::{
        ManagedIntegrationHostBridgeConfigurationV1, ManagedRuntimeClientDeliveryResponseV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::{
        integration_host_bridge::validate_managed_integration_host_bridge_configuration,
        managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES,
        module_client::{validate_module_client_request_v1, validate_module_client_response_v1},
    },
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};

use crate::{
    blob::{RecordingBlobErrorV1, RecordingBlobReceiptV1, write_recording_blob_v1},
    client_port::dispatch_client_request_v1,
    client_realtime::publish_pending_realtime_v1,
    outbox::relay_outbox_once_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopRecordingRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopRecordingManagedRuntimeErrorV1 {
    Admission,
    Persistence(PersistenceErrorV1),
    InvalidDelivery,
    HostBridge,
    Unavailable,
}

pub struct DesktopRecordingManagedRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: DesktopCallRecordingRepositoryV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    logical_human_owner_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    host_bridge_socket_path: String,
    host_bridge_route_binding: [u8; 32],
}

struct NestedClientDispatcherV1<'a> {
    persistence: &'a DesktopCallRecordingRepositoryV1,
}

impl ManagedControlRequestDispatcherV2<UnixStream> for NestedClientDispatcherV1<'_> {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    let response = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(dispatch_client_request_v1(
                            self.persistence,
                            &request,
                            now_unix_ms().unwrap_or_default(),
                        ))
                    });
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(response),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                _ => control_error("managed_runtime_control_invalid_client_delivery"),
            },
            _ => control_error("managed_runtime_control_unexpected_request"),
        };
        channel.write_response(correlation_id, response)
    }
}

impl DesktopRecordingManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &DesktopRecordingRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        host_bridge_configuration: ManagedIntegrationHostBridgeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, DesktopRecordingManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_hub_endpoint.is_empty() || event_credential_revision == 0 {
            return Err(DesktopRecordingManagedRuntimeErrorV1::Admission);
        }
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let (host_bridge_socket_path, host_bridge_route_binding) =
            host_bridge_route(&host_bridge_configuration, admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            storage_configuration
                .vault_hpke_public_key_x25519
                .as_slice()
                .try_into()
                .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?,
        )
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
        let persistence = DesktopCallRecordingRepositoryV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(DesktopRecordingManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(DesktopRecordingManagedRuntimeErrorV1::Persistence)?;
        let mut control_channel = leases.into_route_port().into_channel();
        let event_access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &admission.module_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_credential_revision,
        )
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
        if !event_access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?
            .is_empty()
        {
            return Err(DesktopRecordingManagedRuntimeErrorV1::Admission);
        }
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            RuntimeNatsIdentity::new(
                admission.runtime_instance_id.clone(),
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            logical_human_owner_id: admission.logical_human_owner_id.clone(),
            runtime_instance_id: admission.runtime_instance_id.clone(),
            runtime_generation: admission.runtime_generation,
            host_bridge_socket_path,
            host_bridge_route_binding,
        })
    }

    pub fn bind_host_bridge_listener(
        &self,
    ) -> Result<UnixListener, DesktopRecordingManagedRuntimeErrorV1> {
        let listener = UnixListener::bind(&self.host_bridge_socket_path)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::HostBridge)?;
        std::fs::set_permissions(
            &self.host_bridge_socket_path,
            std::fs::Permissions::from_mode(0o600),
        )
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::HostBridge)?;
        Ok(listener)
    }

    pub async fn try_handle_client_delivery(
        &mut self,
    ) -> Result<bool, DesktopRecordingManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.control_channel
                .write_response(
                    correlation_id,
                    control_error("managed_runtime_control_unexpected_request"),
                )
                .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
            return Ok(true);
        };
        let request = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
            .ok_or(DesktopRecordingManagedRuntimeErrorV1::InvalidDelivery)?;
        let response =
            dispatch_client_request_v1(&self.persistence, &request, now_unix_ms()?).await;
        validate_module_client_response_v1(&response)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::InvalidDelivery)?;
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ClientDelivery(
                        ManagedRuntimeClientDeliveryResponseV1 {
                            response: Some(response),
                        },
                    )),
                    error_code: String::new(),
                },
            )
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn relay_outbox(&self) -> Result<usize, DesktopRecordingManagedRuntimeErrorV1> {
        relay_outbox_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.event_publish_permit,
            now_unix_ms()?,
        )
        .await
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)
    }

    pub async fn publish_realtime(
        &mut self,
    ) -> Result<bool, DesktopRecordingManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = NestedClientDispatcherV1 {
            persistence: &self.persistence,
        };
        let result = publish_pending_realtime_v1(
            &self.persistence,
            &mut self.control_channel,
            &mut dispatcher,
            now_unix_ms()?,
        )
        .await;
        if let Err(error) = result
            && std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some()
        {
            eprintln!("developer_desktop_recording_realtime_error={error:?}");
        }
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
        result.map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)
    }

    pub fn try_serve_host_bridge_once(
        &mut self,
        listener: &UnixListener,
        handle: &tokio::runtime::Handle,
    ) -> Result<bool, DesktopRecordingManagedRuntimeErrorV1> {
        let (stream, _) = match listener.accept() {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
            Err(_) => return Err(DesktopRecordingManagedRuntimeErrorV1::HostBridge),
        };
        // A native-client protocol, authorization, or disconnected-stream
        // failure belongs to this accepted connection. The private route
        // remains available for the next explicitly authenticated host
        // operation; untrusted input must not restart the managed runtime.
        match crate::host_transport::serve_one_operation_v1(stream, self, handle, now_unix_ms()?) {
            Err(error) if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() => {
                eprintln!("developer_desktop_recording_host_transport_error={error:?}");
            }
            _ => {}
        }
        Ok(true)
    }

    #[must_use]
    pub fn accepts_host_route(&self, route_binding_sha256: &[u8]) -> bool {
        route_binding_sha256 == self.host_bridge_route_binding
    }

    #[must_use]
    pub fn logical_human_owner_id(&self) -> &str {
        &self.logical_human_owner_id
    }

    #[must_use]
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    pub(crate) fn persistence(&self) -> &DesktopCallRecordingRepositoryV1 {
        &self.persistence
    }

    pub(crate) fn claim_sha256(&self, host_claim_id: [u8; 16]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(b"makosh.desktop-call-recording.host-claim.v1\0");
        hash.update(self.host_bridge_route_binding);
        hash.update(host_claim_id);
        hash.finalize().into()
    }

    pub(crate) fn write_recording_blob(
        &mut self,
        recording_evidence_id: [u8; 16],
        bytes: Vec<u8>,
        sha256: [u8; 32],
    ) -> Result<RecordingBlobReceiptV1, RecordingBlobErrorV1> {
        let mut dispatcher = NestedClientDispatcherV1 {
            persistence: &self.persistence,
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| RecordingBlobErrorV1::Unavailable)?;
        let result = write_recording_blob_v1(
            &mut self.control_channel,
            &mut dispatcher,
            recording_evidence_id,
            bytes,
            sha256,
        );
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| RecordingBlobErrorV1::Unavailable)?;
        result
    }
}

impl Drop for DesktopRecordingManagedRuntimeV1 {
    fn drop(&mut self) {
        let path = std::path::Path::new(&self.host_bridge_socket_path);
        if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_socket()) {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn validate_admission(
    admission: &DesktopRecordingRuntimeAdmissionV1,
) -> Result<(), DesktopRecordingManagedRuntimeErrorV1> {
    if admission.module_owner_id != OWNER_ID_V1
        || !valid_token(&admission.registration_id)
        || !valid_token(&admission.logical_human_owner_id)
        || !valid_token(&admission.runtime_instance_id)
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(DesktopRecordingManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &DesktopRecordingRuntimeAdmissionV1,
) -> Result<(), DesktopRecordingManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
    let identity = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
    if identity.registration_id != admission.registration_id
        || identity.runtime_generation != admission.runtime_generation
        || identity.grant_epoch != admission.grant_epoch
    {
        return Err(DesktopRecordingManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &DesktopRecordingRuntimeAdmissionV1,
) -> Result<(), DesktopRecordingManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, DesktopRecordingManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(DesktopRecordingManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &DesktopRecordingRuntimeAdmissionV1,
) -> Result<StorageBindingV1, DesktopRecordingManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != admission.module_owner_id
        || configuration.owner != OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(DesktopRecordingManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)
}

fn host_bridge_route(
    configuration: &ManagedIntegrationHostBridgeConfigurationV1,
    admission: &DesktopRecordingRuntimeAdmissionV1,
) -> Result<(String, [u8; 32]), DesktopRecordingManagedRuntimeErrorV1> {
    validate_managed_integration_host_bridge_configuration(configuration)
        .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?;
    if configuration.owner_id != admission.module_owner_id
        || configuration.registration_id != admission.registration_id
        || configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.runtime_generation != admission.runtime_generation
        || configuration.grant_epoch != admission.grant_epoch
    {
        return Err(DesktopRecordingManagedRuntimeErrorV1::Admission);
    }
    Ok((
        configuration.socket_path.clone(),
        configuration
            .route_binding_sha256
            .as_slice()
            .try_into()
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Admission)?,
    ))
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn now_unix_ms() -> Result<i64, DesktopRecordingManagedRuntimeErrorV1> {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)?
            .as_millis(),
    )
    .map_err(|_| DesktopRecordingManagedRuntimeErrorV1::Unavailable)
}

fn control_error(code: &str) -> ManagedRuntimeControlResponseV1 {
    ManagedRuntimeControlResponseV1 {
        result: None,
        error_code: code.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_is_exact_and_generation_fenced() {
        let admission = DesktopRecordingRuntimeAdmissionV1 {
            module_owner_id: OWNER_ID_V1.to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            registration_id: "registration-1".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        assert_eq!(validate_admission(&admission), Ok(()));
        let mut invalid = admission;
        invalid.runtime_generation = 0;
        assert_eq!(
            validate_admission(&invalid),
            Err(DesktopRecordingManagedRuntimeErrorV1::Admission)
        );
    }
}
