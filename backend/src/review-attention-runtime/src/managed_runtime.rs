use std::os::unix::net::UnixStream;

use makosh_review_attention_api::{REVIEW_ATTENTION_MODULE_ID_V1, REVIEW_ATTENTION_OWNER_V1};
use makosh_review_attention_persistence::{
    ReviewAttentionPersistenceErrorV1, ReviewAttentionPersistenceV1,
};
use makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
        RejectManagedControlRequestsV2,
    },
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
    client_port::{command_payload_v1, query_payload_v1},
    contracts::{review_attention_command_contract_v1, review_attention_query_contract_v1},
    realtime::{ReviewAttentionRealtimeErrorV1, ReviewAttentionRealtimePublisherV1},
};

const MAX_NESTED_REALTIME_PASSES_V1: u8 = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAttentionRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewAttentionManagedRuntimeErrorV1 {
    Admission,
    Persistence(ReviewAttentionPersistenceErrorV1),
    InvalidTransition,
    Unavailable,
}

pub struct ReviewAttentionManagedRuntimeV1 {
    admission: ReviewAttentionRuntimeAdmissionV1,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: ReviewAttentionPersistenceV1,
    realtime: ReviewAttentionRealtimePublisherV1,
}

struct ReviewAttentionNestedRequestDispatcherV1<'a> {
    persistence: &'a ReviewAttentionPersistenceV1,
    admission: &'a ReviewAttentionRuntimeAdmissionV1,
    publish_realtime_requested: bool,
}

impl ManagedControlRequestDispatcherV2<UnixStream>
    for ReviewAttentionNestedRequestDispatcherV1<'_>
{
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: makosh_runtime_protocol::v1::ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    let (response, publish_realtime) = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(dispatch_client(
                            self.persistence,
                            self.admission,
                            request,
                        ))
                    });
                    self.publish_realtime_requested |= publish_realtime;
                    client_delivery_response(response)
                }
                Some(request) => client_delivery_response(ModuleClientResponseV1 {
                    protocol_major: 1,
                    request_id: request.request_id,
                    response_payload: Vec::new(),
                    error_code: "REJECTED".to_owned(),
                }),
                None => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

impl ReviewAttentionManagedRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &ReviewAttentionRuntimeAdmissionV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
    ) -> Result<Self, ReviewAttentionManagedRuntimeErrorV1> {
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
            .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?;
        let persistence = ReviewAttentionPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(|error| persistence_error_at("connect", error))?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(|error| persistence_error_at("readiness", error))?;
        let mut control_channel = leases.into_route_port().into_channel();
        let mut realtime = ReviewAttentionRealtimePublisherV1::default();
        let mut dispatcher = RejectManagedControlRequestsV2;
        realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_human_owner_id,
            )
            .await
            .map_err(|error| realtime_error_at("replay", error))?;
        signal_ready(&mut control_channel, admission)?;
        Ok(Self {
            admission: admission.clone(),
            control_channel,
            persistence,
            realtime,
        })
    }

    pub async fn pump_control_once(
        &mut self,
    ) -> Result<bool, ReviewAttentionManagedRuntimeErrorV1> {
        let (correlation_id, request) = self
            .control_channel
            .receive_request()
            .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Unavailable)?;
        let (response, publish_realtime) = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    let (response, publish_realtime) =
                        dispatch_client(&self.persistence, &self.admission, request).await;
                    (
                        ManagedRuntimeControlResponseV1 {
                            result: Some(ControlResult::ClientDelivery(
                                ManagedRuntimeClientDeliveryResponseV1 {
                                    response: Some(response),
                                },
                            )),
                            error_code: String::new(),
                        },
                        publish_realtime,
                    )
                }
                _ => (
                    ManagedRuntimeControlResponseV1 {
                        result: None,
                        error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                    },
                    false,
                ),
            },
            _ => (
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_unexpected_request".to_owned(),
                },
                false,
            ),
        };
        self.control_channel
            .write_response(correlation_id, response)
            .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Unavailable)?;
        if publish_realtime {
            for pass in 1..=MAX_NESTED_REALTIME_PASSES_V1 {
                let mut dispatcher = ReviewAttentionNestedRequestDispatcherV1 {
                    persistence: &self.persistence,
                    admission: &self.admission,
                    publish_realtime_requested: false,
                };
                self.realtime
                    .publish_pending(
                        &self.persistence,
                        &mut self.control_channel,
                        &mut dispatcher,
                        &self.admission.logical_human_owner_id,
                    )
                    .await
                    .map_err(realtime_error)?;
                if !dispatcher.publish_realtime_requested {
                    return Ok(true);
                }
                if pass == MAX_NESTED_REALTIME_PASSES_V1 {
                    return Err(ReviewAttentionManagedRuntimeErrorV1::Unavailable);
                }
            }
        }
        Ok(true)
    }
}

fn client_delivery_response(response: ModuleClientResponseV1) -> ManagedRuntimeControlResponseV1 {
    ManagedRuntimeControlResponseV1 {
        result: Some(ControlResult::ClientDelivery(
            ManagedRuntimeClientDeliveryResponseV1 {
                response: Some(response),
            },
        )),
        error_code: String::new(),
    }
}

async fn dispatch_client(
    persistence: &ReviewAttentionPersistenceV1,
    admission: &ReviewAttentionRuntimeAdmissionV1,
    request: ModuleClientRequestV1,
) -> (ModuleClientResponseV1, bool) {
    let (payload, accepted, publish_realtime) = if !valid_client_identity(&request, admission) {
        (Vec::new(), false, false)
    } else if request.contract.as_ref() == Some(&review_attention_command_contract_v1()) {
        (
            command_payload_v1(
                persistence,
                &admission.logical_human_owner_id,
                &request.request_payload,
            )
            .await,
            true,
            true,
        )
    } else if request.contract.as_ref() == Some(&review_attention_query_contract_v1()) {
        (
            query_payload_v1(
                persistence,
                &admission.logical_human_owner_id,
                &request.request_payload,
            )
            .await,
            true,
            false,
        )
    } else {
        (Vec::new(), false, false)
    };
    let response = ModuleClientResponseV1 {
        protocol_major: 1,
        request_id: request.request_id,
        response_payload: payload,
        error_code: if accepted {
            String::new()
        } else {
            "REJECTED".to_owned()
        },
    };
    debug_assert!(validate_module_client_response_v1(&response).is_ok());
    (response, publish_realtime)
}

fn valid_client_identity(
    request: &ModuleClientRequestV1,
    admission: &ReviewAttentionRuntimeAdmissionV1,
) -> bool {
    request.protocol_major == 1
        && request.module_id == REVIEW_ATTENTION_MODULE_ID_V1
        && request.owner_id == REVIEW_ATTENTION_OWNER_V1
        && request.logical_owner_id == admission.logical_human_owner_id
}

fn validate_admission(
    admission: &ReviewAttentionRuntimeAdmissionV1,
) -> Result<(), ReviewAttentionManagedRuntimeErrorV1> {
    if admission.logical_owner_id != REVIEW_ATTENTION_OWNER_V1
        || admission.logical_human_owner_id.is_empty()
        || admission.logical_human_owner_id == admission.logical_owner_id
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(ReviewAttentionManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &ReviewAttentionRuntimeAdmissionV1,
) -> Result<(), ReviewAttentionManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(ReviewAttentionManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &ReviewAttentionRuntimeAdmissionV1,
) -> Result<(), ReviewAttentionManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ReviewAttentionManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(ReviewAttentionManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &ReviewAttentionRuntimeAdmissionV1,
) -> Result<StorageBindingV1, ReviewAttentionManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != admission.logical_owner_id
        || configuration.owner != REVIEW_ATTENTION_OWNER_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(ReviewAttentionManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| ReviewAttentionManagedRuntimeErrorV1::Admission)
}

fn realtime_error(error: ReviewAttentionRealtimeErrorV1) -> ReviewAttentionManagedRuntimeErrorV1 {
    match error {
        ReviewAttentionRealtimeErrorV1::InvalidTransition => {
            ReviewAttentionManagedRuntimeErrorV1::InvalidTransition
        }
        ReviewAttentionRealtimeErrorV1::Persistence(error) => {
            ReviewAttentionManagedRuntimeErrorV1::Persistence(error)
        }
        ReviewAttentionRealtimeErrorV1::Unavailable => {
            ReviewAttentionManagedRuntimeErrorV1::Unavailable
        }
    }
}

fn realtime_error_at(
    stage: &str,
    error: ReviewAttentionRealtimeErrorV1,
) -> ReviewAttentionManagedRuntimeErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_review_attention_runtime_error stage={stage} kind={error:?}");
    }
    realtime_error(error)
}

fn persistence_error_at(
    stage: &str,
    error: ReviewAttentionPersistenceErrorV1,
) -> ReviewAttentionManagedRuntimeErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_review_attention_runtime_error stage={stage} kind={error:?}");
    }
    ReviewAttentionManagedRuntimeErrorV1::Persistence(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_separates_module_and_human_owner() {
        let valid = ReviewAttentionRuntimeAdmissionV1 {
            logical_owner_id: "review".to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            registration_id: "review-registration".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        assert_eq!(validate_admission(&valid), Ok(()));
        let mut invalid = valid;
        invalid.logical_human_owner_id = "review".to_owned();
        assert_eq!(
            validate_admission(&invalid),
            Err(ReviewAttentionManagedRuntimeErrorV1::Admission)
        );
    }

    #[test]
    fn client_identity_rejects_a_different_human_owner() {
        let admission = ReviewAttentionRuntimeAdmissionV1 {
            logical_owner_id: REVIEW_ATTENTION_OWNER_V1.to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            registration_id: "review-registration".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        let request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: REVIEW_ATTENTION_MODULE_ID_V1.to_owned(),
            owner_id: REVIEW_ATTENTION_OWNER_V1.to_owned(),
            logical_owner_id: "owner-2".to_owned(),
            ..Default::default()
        };
        assert!(!valid_client_identity(&request, &admission));
    }
}
