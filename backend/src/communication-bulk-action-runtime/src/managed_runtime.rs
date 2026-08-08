use std::os::unix::net::UnixStream;

use makosh_communication_bulk_action_api::{
    COMMUNICATION_BULK_ACTION_MODULE_ID_V1, COMMUNICATION_BULK_ACTION_OWNER_V1,
};
use makosh_communication_bulk_action_persistence::{
    BulkDeliveryPersistenceErrorV1, CommunicationBulkActionPersistenceV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::{
        ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1, ModuleClientRequestV1,
        ModuleClientResponseV1, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_client::{
        validate_module_client_request_v1, validate_module_client_response_v1,
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
    client_port::{get_status_payload_v1, start_bulk_delivery_payload_v1},
    client_realtime::{BulkDeliveryClientRealtimeErrorV1, BulkDeliveryClientRealtimePublisherV1},
    contracts::{bulk_command_contract_v1, bulk_query_contract_v1},
    managed_delivery_port::ManagedDeliveryIntentRequestPortV1,
    worker::{BulkDeliveryWorkerErrorV1, process_next_target_v1},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkDeliveryRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkDeliveryManagedRuntimeErrorV1 {
    Admission,
    Persistence(BulkDeliveryPersistenceErrorV1),
    InvalidTransition,
    Unavailable,
}

pub struct BulkDeliveryManagedRuntimeV1 {
    logical_owner_id: String,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: CommunicationBulkActionPersistenceV1,
    client_realtime: BulkDeliveryClientRealtimePublisherV1,
}

impl BulkDeliveryManagedRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &BulkDeliveryRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
    ) -> Result<Self, BulkDeliveryManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?;
        let persistence = CommunicationBulkActionPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(BulkDeliveryManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(BulkDeliveryManagedRuntimeErrorV1::Persistence)?;
        let mut control_channel = leases.into_route_port().into_channel();
        let mut client_realtime = BulkDeliveryClientRealtimePublisherV1::default();
        let mut dispatcher = RejectManagedControlRequestsV2;
        client_realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_owner_id,
            )
            .await
            .map_err(realtime_error)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            logical_owner_id: admission.logical_owner_id.clone(),
            control_channel,
            persistence,
            client_realtime,
        })
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_seconds: i64,
    ) -> Result<bool, BulkDeliveryManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            self.control_channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: None,
                        error_code: "managed_runtime_control_unexpected_request".to_owned(),
                    },
                )
                .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
            return Ok(true);
        };
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            self.control_channel
                .write_response(
                    correlation_id,
                    ManagedRuntimeControlResponseV1 {
                        result: None,
                        error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                    },
                )
                .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
            return Ok(true);
        };
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let response = dispatch_client(
            &self.persistence,
            &self.logical_owner_id,
            request,
            now_unix_seconds,
        )
        .await;
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(BulkDeliveryManagedRuntimeErrorV1::Unavailable);
        }
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
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn process_next_target(
        &mut self,
        worker_id: &str,
        now_unix_seconds: i64,
    ) -> Result<bool, BulkDeliveryManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let mut port = ManagedDeliveryIntentRequestPortV1 {
            channel: &mut self.control_channel,
            dispatcher: &mut dispatcher,
        };
        let result = process_next_target_v1(
            &self.persistence,
            &mut port,
            &self.logical_owner_id,
            worker_id,
            now_unix_seconds,
        )
        .await
        .map(|outcome| outcome.is_some())
        .map_err(worker_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        result
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, BulkDeliveryManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = self
            .client_realtime
            .publish_pending(
                &self.persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &self.logical_owner_id,
            )
            .await
            .map_err(realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
        result
    }
}

async fn dispatch_client(
    persistence: &CommunicationBulkActionPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> ModuleClientResponseV1 {
    let valid_identity = request.protocol_major == 1
        && request.module_id == COMMUNICATION_BULK_ACTION_MODULE_ID_V1
        && request.owner_id == COMMUNICATION_BULK_ACTION_OWNER_V1;
    let (payload, accepted_route) = if valid_identity {
        if request.contract.as_ref() == Some(&bulk_command_contract_v1()) {
            (
                start_bulk_delivery_payload_v1(
                    persistence,
                    logical_owner_id,
                    &request.request_payload,
                    now_unix_seconds,
                )
                .await,
                true,
            )
        } else if request.contract.as_ref() == Some(&bulk_query_contract_v1()) {
            (
                get_status_payload_v1(persistence, logical_owner_id, &request.request_payload)
                    .await,
                true,
            )
        } else {
            (Vec::new(), false)
        }
    } else {
        (Vec::new(), false)
    };
    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id: request.request_id,
        response_payload: payload,
        error_code: if accepted_route {
            String::new()
        } else {
            "REJECTED".to_owned()
        },
    }
}

fn validate_admission(
    admission: &BulkDeliveryRuntimeAdmissionV1,
) -> Result<(), BulkDeliveryManagedRuntimeErrorV1> {
    if admission.logical_owner_id.is_empty()
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(BulkDeliveryManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &BulkDeliveryRuntimeAdmissionV1,
) -> Result<(), BulkDeliveryManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(BulkDeliveryManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &BulkDeliveryRuntimeAdmissionV1,
) -> Result<(), BulkDeliveryManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, BulkDeliveryManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(BulkDeliveryManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &BulkDeliveryRuntimeAdmissionV1,
) -> Result<StorageBindingV1, BulkDeliveryManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != configuration.owner
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(BulkDeliveryManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| BulkDeliveryManagedRuntimeErrorV1::Admission)
}

const fn worker_error(error: BulkDeliveryWorkerErrorV1) -> BulkDeliveryManagedRuntimeErrorV1 {
    match error {
        BulkDeliveryWorkerErrorV1::InvalidInput => BulkDeliveryManagedRuntimeErrorV1::Admission,
        BulkDeliveryWorkerErrorV1::Persistence(error) => {
            BulkDeliveryManagedRuntimeErrorV1::Persistence(error)
        }
    }
}

const fn realtime_error(
    error: BulkDeliveryClientRealtimeErrorV1,
) -> BulkDeliveryManagedRuntimeErrorV1 {
    match error {
        BulkDeliveryClientRealtimeErrorV1::InvalidTransition => {
            BulkDeliveryManagedRuntimeErrorV1::InvalidTransition
        }
        BulkDeliveryClientRealtimeErrorV1::Persistence(error) => {
            BulkDeliveryManagedRuntimeErrorV1::Persistence(error)
        }
        BulkDeliveryClientRealtimeErrorV1::Unavailable => {
            BulkDeliveryManagedRuntimeErrorV1::Unavailable
        }
    }
}
