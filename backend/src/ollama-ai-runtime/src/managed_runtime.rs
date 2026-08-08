use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    ai_provider_explanation_contract_reference_v1,
    ai_provider_reply_generation_contract_reference_v1,
    ai_provider_summary_generation_contract_reference_v1,
    ai_provider_translation_contract_reference_v1,
};
use makosh_ollama_ai_api::{OLLAMA_OWNER_ID_V1, OllamaAiRuntimeSettingsV1};
use makosh_ollama_ai_persistence::{OllamaAiPersistenceErrorV1, OllamaAiPersistenceV1};
use makosh_runtime_protocol::{
    managed_control::ManagedControlChannelV2,
    v1::{
        ManagedRuntimeControlResponseV1, ManagedRuntimeModuleRequestResponseV1,
        ManagedRuntimeReadyRequestV1, ManagedStorageRuntimeConfigurationV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        validate_module_request_delivery_v1, validate_module_request_response_v1,
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
    explanation_worker::execute_explanation_payload_v1,
    summary_worker::execute_summary_payload_v1,
    translation_worker::execute_translation_payload_v1,
    worker::{LocalOllamaAiExecutionPortV1, OllamaAiWorkerErrorV1, execute_payload_v1},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaAiRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub configuration_instance_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiManagedRuntimeErrorV1 {
    Admission,
    Persistence(OllamaAiPersistenceErrorV1),
    Unavailable,
}

pub struct OllamaAiManagedRuntimeV1 {
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: OllamaAiPersistenceV1,
    settings: OllamaAiRuntimeSettingsV1,
    logical_human_owner_id: String,
}

impl OllamaAiManagedRuntimeV1 {
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &OllamaAiRuntimeAdmissionV1,
        settings: OllamaAiRuntimeSettingsV1,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
    ) -> Result<Self, OllamaAiManagedRuntimeErrorV1> {
        validate_admission_v1(admission)?;
        if settings.settings_revision == 0 {
            return Err(OllamaAiManagedRuntimeErrorV1::Admission);
        }
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate_v1(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let binding = storage_binding_v1(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential_v1(&mut leases, &binding).await?;
        let password =
            std::str::from_utf8(&password).map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?;
        let persistence = OllamaAiPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(OllamaAiManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(OllamaAiManagedRuntimeErrorV1::Persistence)?;
        let mut control_channel = leases.into_route_port().into_channel();
        signal_ready_v1(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| OllamaAiManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            control_channel,
            persistence,
            settings,
            logical_human_owner_id: admission.logical_human_owner_id.clone(),
        })
    }

    pub async fn pump_control_once(&mut self) -> Result<bool, OllamaAiManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| OllamaAiManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let response = match request.operation {
            Some(Operation::DeliverModuleRequest(delivery)) => {
                let request_id = delivery.request_id.clone();
                let contract = delivery.contract.as_ref();
                let is_explanation =
                    contract == Some(&ai_provider_explanation_contract_reference_v1());
                let is_reply =
                    contract == Some(&ai_provider_reply_generation_contract_reference_v1());
                let is_summary =
                    contract == Some(&ai_provider_summary_generation_contract_reference_v1());
                let is_translation =
                    contract == Some(&ai_provider_translation_contract_reference_v1());
                let response = if validate_module_request_delivery_v1(&delivery).is_err()
                    || (!is_explanation && !is_reply && !is_summary && !is_translation)
                    || delivery.logical_owner_id != self.logical_human_owner_id
                {
                    rejected_v1(request_id)
                } else {
                    let mut port = LocalOllamaAiExecutionPortV1;
                    let executed = if is_explanation {
                        execute_explanation_payload_v1(
                            &self.persistence,
                            &mut port,
                            &self.logical_human_owner_id,
                            &self.settings,
                            &delivery.request_payload,
                        )
                        .await
                    } else if is_translation {
                        execute_translation_payload_v1(
                            &self.persistence,
                            &mut port,
                            &self.logical_human_owner_id,
                            &self.settings,
                            &delivery.request_payload,
                        )
                        .await
                    } else if is_summary {
                        execute_summary_payload_v1(
                            &self.persistence,
                            &mut port,
                            &self.logical_human_owner_id,
                            &self.settings,
                            &delivery.request_payload,
                        )
                        .await
                    } else {
                        execute_payload_v1(
                            &self.persistence,
                            &mut port,
                            &self.logical_human_owner_id,
                            &self.settings,
                            &delivery.request_payload,
                        )
                        .await
                    };
                    match executed {
                        Ok(payload) => ManagedRuntimeModuleRequestResponseV1 {
                            request_id,
                            response_payload: payload,
                            error_code: String::new(),
                        },
                        Err(
                            OllamaAiWorkerErrorV1::Unavailable | OllamaAiWorkerErrorV1::Uncertain,
                        ) => unavailable_v1(request_id),
                        Err(
                            OllamaAiWorkerErrorV1::InvalidRequest | OllamaAiWorkerErrorV1::Conflict,
                        ) => rejected_v1(request_id),
                    }
                };
                if validate_module_request_response_v1(&response).is_err() {
                    return Err(OllamaAiManagedRuntimeErrorV1::Unavailable);
                }
                ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ModuleRequestDelivery(response)),
                    error_code: String::new(),
                }
            }
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        self.control_channel
            .write_response(correlation_id, response)
            .map_err(|_| OllamaAiManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }
}

fn rejected_v1(request_id: Vec<u8>) -> ManagedRuntimeModuleRequestResponseV1 {
    ManagedRuntimeModuleRequestResponseV1 {
        request_id,
        response_payload: Vec::new(),
        error_code: "REJECTED".to_owned(),
    }
}

fn unavailable_v1(request_id: Vec<u8>) -> ManagedRuntimeModuleRequestResponseV1 {
    ManagedRuntimeModuleRequestResponseV1 {
        request_id,
        response_payload: Vec::new(),
        error_code: "UNAVAILABLE".to_owned(),
    }
}

fn validate_admission_v1(
    admission: &OllamaAiRuntimeAdmissionV1,
) -> Result<(), OllamaAiManagedRuntimeErrorV1> {
    if admission.module_owner_id != OLLAMA_OWNER_ID_V1
        || !valid_owner_id_v1(&admission.logical_human_owner_id)
        || admission.configuration_instance_id.is_empty()
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
    {
        return Err(OllamaAiManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn valid_owner_id_v1(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn authenticate_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &OllamaAiRuntimeAdmissionV1,
) -> Result<(), OllamaAiManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| OllamaAiManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| OllamaAiManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(OllamaAiManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &OllamaAiRuntimeAdmissionV1,
) -> Result<(), OllamaAiManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| OllamaAiManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| OllamaAiManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential_v1(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, OllamaAiManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(OllamaAiManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding_v1(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &OllamaAiRuntimeAdmissionV1,
) -> Result<StorageBindingV1, OllamaAiManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != admission.module_owner_id
        || configuration.owner != OLLAMA_OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(OllamaAiManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| OllamaAiManagedRuntimeErrorV1::Admission)
}

#[cfg(test)]
mod tests {
    use makosh_ollama_ai_api::OLLAMA_AI_MODULE_ID_V1;

    use super::*;

    #[test]
    fn admission_is_exactly_ollama_owned_and_generation_fenced() {
        let valid = OllamaAiRuntimeAdmissionV1 {
            module_owner_id: OLLAMA_OWNER_ID_V1.to_owned(),
            logical_human_owner_id: "owner-1".to_owned(),
            configuration_instance_id: "ollama-local".to_owned(),
            registration_id: OLLAMA_AI_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        assert_eq!(validate_admission_v1(&valid), Ok(()));
        let mut invalid = valid;
        invalid.module_owner_id = "communications".to_owned();
        assert_eq!(
            validate_admission_v1(&invalid),
            Err(OllamaAiManagedRuntimeErrorV1::Admission)
        );
        invalid.module_owner_id = OLLAMA_OWNER_ID_V1.to_owned();
        invalid.logical_human_owner_id.clear();
        assert_eq!(
            validate_admission_v1(&invalid),
            Err(OllamaAiManagedRuntimeErrorV1::Admission)
        );
    }
}
