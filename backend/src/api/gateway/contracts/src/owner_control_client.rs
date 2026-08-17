//! Typed client for the owner-private Kernel Unix control socket.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use prost::Message;

use crate::owner_control_proof::owner_control_proof_message_v1;
use crate::v1::{
    AdmitBundledStorageArtifactRequestV1, AdmitBundledStorageArtifactResponseV1,
    ApplyManagedIntegrationSettingsRequestV1, ApplyManagedIntegrationSettingsResponseV1,
    ApproveModuleRegistrationRequestV1, ApproveModuleRegistrationResponseV1,
    BeginBrowserPairingRequestV1, BeginManagedStorageBindingRevocationRequestV1,
    BeginManagedStorageBindingRevocationResponseV1, BeginOwnerControlSessionRequestV1,
    BeginOwnerControlSessionResponseV1, BindBundledManagedReleaseRequestV1,
    BindBundledManagedReleaseResponseV1, CompleteOwnerControlSessionRequestV1,
    GetManagedStorageBindingStatusRequestV1, GetManagedStorageBindingStatusResponseV1,
    GetModuleRegistrationStatusRequestV1, GetModuleRegistrationStatusResponseV1,
    IssueManagedStorageBindingRequestV1, IssueManagedStorageBindingResponseV1,
    OwnerControlRequestV1, OwnerControlResponseV1, ProposeBundledManagedArtifactRequestV1,
    ProposeBundledManagedArtifactResponseV1, ReserveBundledManagedRuntimeRequestV1,
    ReserveBundledManagedRuntimeResponseV1, StartReservedDomainRuntimeRequestV1,
    StartReservedDomainRuntimeResponseV1, StartReservedEngineRuntimeRequestV1,
    StartReservedEngineRuntimeResponseV1, StartReservedIntegrationRuntimeRequestV1,
    StartReservedIntegrationRuntimeResponseV1, StartReservedWorkflowRuntimeRequestV1,
    StartReservedWorkflowRuntimeResponseV1, TransitionModuleRegistrationRequestV1,
    TransitionModuleRegistrationResponseV1, UpdateOperatorSettingsRequestV1,
    UpdateOperatorSettingsResponseV1, UpgradeBundledManagedRegistrationRequestV1,
    UpgradeBundledManagedRegistrationResponseV1, owner_control_request_v1,
    owner_control_response_v1,
};

const IPC_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_FRAME_BYTES: usize = 64 * 1024;

pub trait OwnerControlProofSignerV1 {
    fn sign_owner_control_proof(&self, message: &[u8]) -> Result<[u8; 64], String>;
}

pub struct OwnerControlClientV1 {
    socket_path: PathBuf,
}

pub struct OwnerControlChallengeV1 {
    challenge_id: String,
    challenge_bytes: [u8; 32],
    kernel_instance_id: String,
    owner_id: String,
    device_id: String,
    control_store_generation: u64,
}

impl OwnerControlClientV1 {
    #[must_use]
    pub fn new(runtime_dir: &Path) -> Self {
        Self {
            socket_path: runtime_dir.join("owner.sock"),
        }
    }

    pub fn open_owner_session(
        &self,
        signer: &impl OwnerControlProofSignerV1,
    ) -> Result<String, String> {
        let response = self.request(owner_control_request_v1::Operation::BeginOwnerSession(
            BeginOwnerControlSessionRequestV1 {},
        ))?;
        let challenge = match response.result {
            Some(owner_control_response_v1::Result::BeginOwnerSession(value)) => {
                OwnerControlChallengeV1::from_response(value)?
            }
            _ => return Err("owner control session is unavailable".to_owned()),
        };
        let signature = signer.sign_owner_control_proof(&challenge.proof_message()?)?;
        let response = self.request(owner_control_request_v1::Operation::CompleteOwnerSession(
            CompleteOwnerControlSessionRequestV1 {
                challenge_id: challenge.challenge_id,
                signature_raw: signature.to_vec(),
            },
        ))?;
        match response.result {
            Some(owner_control_response_v1::Result::CompleteOwnerSession(value))
                if !value.owner_session_id.is_empty() =>
            {
                Ok(value.owner_session_id)
            }
            _ => Err("owner control session is unavailable".to_owned()),
        }
    }

    pub fn begin_browser_pairing(&self, owner_session_id: &str) -> Result<String, String> {
        let response = self.request(owner_control_request_v1::Operation::BeginBrowserPairing(
            BeginBrowserPairingRequestV1 {
                owner_session_id: owner_session_id.to_owned(),
            },
        ))?;
        match response.result {
            Some(owner_control_response_v1::Result::BeginBrowserPairing(value))
                if value.pairing_id.len() == 64 =>
            {
                Ok(value.pairing_id)
            }
            _ => Err("browser pairing is unavailable".to_owned()),
        }
    }

    pub fn transition_module_registration(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        target_state: &str,
    ) -> Result<TransitionModuleRegistrationResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::TransitionModuleRegistration(
                TransitionModuleRegistrationRequestV1 {
                    owner_session_id: owner_session_id.to_owned(),
                    registration_id: registration_id.to_owned(),
                    target_state: target_state.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::TransitionModuleRegistration(value))
                if value.registration_id == registration_id
                    && value.registration_state == target_state
                    && value.grant_epoch > 0 =>
            {
                Ok(value)
            }
            _ => Err("module registration transition is unavailable".to_owned()),
        }
    }

    pub fn module_registration_status(
        &self,
        registration_id: &str,
    ) -> Result<GetModuleRegistrationStatusResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::GetModuleRegistrationStatus(
                GetModuleRegistrationStatusRequestV1 {
                    registration_id: registration_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::GetModuleRegistrationStatus(value))
                if value.registration_id == registration_id =>
            {
                Ok(value)
            }
            _ => Err("module registration status is unavailable".to_owned()),
        }
    }

    pub fn approve_module_registration(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        capability_ids: Vec<String>,
    ) -> Result<ApproveModuleRegistrationResponseV1, String> {
        let expected_capability_count = u32::try_from(capability_ids.len())
            .map_err(|_| "module capability set is invalid".to_owned())?;
        let response = self.request(
            owner_control_request_v1::Operation::ApproveModuleRegistration(
                ApproveModuleRegistrationRequestV1 {
                    registration_id: registration_id.to_owned(),
                    capability_id: capability_ids,
                    owner_session_id: owner_session_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::ApproveModuleRegistration(value))
                if value.registration_id == registration_id
                    && value.grant_epoch > 0
                    && value.effective_capability_count == expected_capability_count =>
            {
                Ok(value)
            }
            _ => Err("module registration approval is unavailable".to_owned()),
        }
    }

    pub fn propose_bundled_managed_artifact(
        &self,
        owner_session_id: &str,
        artifact_id: &str,
        expected_distribution_id: &str,
        expected_distribution_generation: u64,
        idempotency_key: [u8; 16],
    ) -> Result<ProposeBundledManagedArtifactResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::ProposeBundledManagedArtifact(
                ProposeBundledManagedArtifactRequestV1 {
                    owner_session_id: owner_session_id.to_owned(),
                    artifact_id: artifact_id.to_owned(),
                    expected_distribution_id: expected_distribution_id.to_owned(),
                    expected_distribution_generation,
                    idempotency_key: idempotency_key.to_vec(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::ProposeBundledManagedArtifact(value))
                if value.artifact_id == artifact_id
                    && value.distribution_id == expected_distribution_id
                    && value.distribution_generation == expected_distribution_generation
                    && !value.registration_id.is_empty()
                    && !value.module_id.is_empty()
                    && !value.owner_id.is_empty()
                    && value.descriptor_sha256.len() == 32 =>
            {
                Ok(value)
            }
            _ => Err("bundled managed artifact proposal is unavailable".to_owned()),
        }
    }

    pub fn admit_bundled_storage_artifact(
        &self,
        owner_session_id: &str,
        artifact_id: &str,
        expected_distribution_id: &str,
        expected_distribution_generation: u64,
    ) -> Result<AdmitBundledStorageArtifactResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::AdmitBundledStorageArtifact(
                AdmitBundledStorageArtifactRequestV1 {
                    owner_session_id: owner_session_id.to_owned(),
                    artifact_id: artifact_id.to_owned(),
                    expected_distribution_id: expected_distribution_id.to_owned(),
                    expected_distribution_generation,
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::AdmitBundledStorageArtifact(value))
                if value.artifact_id == artifact_id
                    && value.distribution_id == expected_distribution_id
                    && value.distribution_generation == expected_distribution_generation
                    && !value.owner_id.is_empty()
                    && value.storage_bundle_revision > 0
                    && value.storage_bundle_digest.len() == 32 =>
            {
                Ok(value)
            }
            _ => Err("bundled Storage artifact admission is unavailable".to_owned()),
        }
    }

    pub fn bind_bundled_managed_release(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        artifact_id: &str,
    ) -> Result<BindBundledManagedReleaseResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::BindBundledManagedRelease(
                BindBundledManagedReleaseRequestV1 {
                    registration_id: registration_id.to_owned(),
                    artifact_id: artifact_id.to_owned(),
                    owner_session_id: owner_session_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::BindBundledManagedRelease(value))
                if value.registration_id == registration_id
                    && value.artifact_id == artifact_id
                    && value.binding_revision > 0
                    && !value.distribution_id.is_empty() =>
            {
                Ok(value)
            }
            _ => Err("bundled managed release binding is unavailable".to_owned()),
        }
    }

    pub fn upgrade_bundled_managed_registration(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        artifact_id: &str,
        expected_distribution_id: &str,
        expected_distribution_generation: u64,
    ) -> Result<UpgradeBundledManagedRegistrationResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::UpgradeBundledManagedRegistration(
                UpgradeBundledManagedRegistrationRequestV1 {
                    owner_session_id: owner_session_id.to_owned(),
                    registration_id: registration_id.to_owned(),
                    artifact_id: artifact_id.to_owned(),
                    expected_distribution_id: expected_distribution_id.to_owned(),
                    expected_distribution_generation,
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::UpgradeBundledManagedRegistration(value))
                if value.registration_id == registration_id
                    && value.grant_epoch > 0
                    && value.descriptor_sha256.len() == 32
                    && value.effective_capability_count > 0 =>
            {
                Ok(value)
            }
            _ => Err("bundled managed registration upgrade is unavailable".to_owned()),
        }
    }

    pub fn reserve_bundled_managed_runtime(
        &self,
        owner_session_id: &str,
        registration_id: &str,
    ) -> Result<ReserveBundledManagedRuntimeResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::ReserveBundledManagedRuntime(
                ReserveBundledManagedRuntimeRequestV1 {
                    registration_id: registration_id.to_owned(),
                    owner_session_id: owner_session_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::ReserveBundledManagedRuntime(value))
                if value.registration_id == registration_id
                    && !value.runtime_instance_id.is_empty()
                    && value.runtime_generation > 0
                    && value.grant_epoch > 0 =>
            {
                Ok(value)
            }
            _ => Err("bundled managed runtime reservation is unavailable".to_owned()),
        }
    }

    pub fn begin_managed_storage_binding_revocation(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        capability_id: &str,
        binding_revision: u64,
    ) -> Result<BeginManagedStorageBindingRevocationResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::BeginManagedStorageBindingRevocation(
                BeginManagedStorageBindingRevocationRequestV1 {
                    owner_session_id: owner_session_id.to_owned(),
                    registration_id: registration_id.to_owned(),
                    capability_id: capability_id.to_owned(),
                    binding_revision,
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::BeginManagedStorageBindingRevocation(
                value,
            )) if value.registration_id == registration_id
                && value.capability_id == capability_id
                && value.binding_revision == binding_revision =>
            {
                Ok(value)
            }
            _ => Err("managed Storage binding revocation is unavailable".to_owned()),
        }
    }

    pub fn managed_storage_binding_status(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<GetManagedStorageBindingStatusResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::GetManagedStorageBindingStatus(
                GetManagedStorageBindingStatusRequestV1 {
                    owner_session_id: owner_session_id.to_owned(),
                    registration_id: registration_id.to_owned(),
                    capability_id: capability_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::GetManagedStorageBindingStatus(value))
                if value.registration_id == registration_id
                    && value.capability_id == capability_id
                    && value.binding_revision > 0
                    && value.role_epoch > 0
                    && value.credential_lease_revision > 0
                    && !value.runtime_instance_id.is_empty()
                    && value.runtime_generation > 0
                    && value.grant_epoch > 0
                    && value.storage_bundle_revision > 0
                    && value.storage_bundle_digest.len() == 32
                    && matches!(value.binding_state.as_str(), "active" | "revoking") =>
            {
                Ok(value)
            }
            _ => Err("managed Storage binding status is unavailable".to_owned()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn issue_managed_storage_binding(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        capability_id: &str,
        runtime_instance_id: &str,
        runtime_generation: u64,
        role_epoch: u64,
        credential_lease_revision: u64,
        storage_bundle_revision: u64,
        storage_bundle_digest: Vec<u8>,
    ) -> Result<IssueManagedStorageBindingResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::IssueManagedStorageBinding(
                IssueManagedStorageBindingRequestV1 {
                    owner_session_id: owner_session_id.to_owned(),
                    registration_id: registration_id.to_owned(),
                    capability_id: capability_id.to_owned(),
                    runtime_instance_id: runtime_instance_id.to_owned(),
                    runtime_generation,
                    role_epoch,
                    credential_lease_revision,
                    storage_bundle_revision,
                    storage_bundle_digest,
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::IssueManagedStorageBinding(value))
                if value.registration_id == registration_id
                    && value.capability_id == capability_id
                    && value.binding_revision > 0
                    && value.topology_revision > 0
                    && value.storage_generation > 0 =>
            {
                Ok(value)
            }
            _ => Err("managed Storage binding issuance is unavailable".to_owned()),
        }
    }

    pub fn start_reserved_domain_runtime(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        storage_capability_id: &str,
    ) -> Result<StartReservedDomainRuntimeResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::StartReservedDomainRuntime(
                StartReservedDomainRuntimeRequestV1 {
                    registration_id: registration_id.to_owned(),
                    storage_capability_id: storage_capability_id.to_owned(),
                    owner_session_id: owner_session_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::StartReservedDomainRuntime(value))
                if value.registration_id == registration_id
                    && value.runtime_generation > 0
                    && value.launch_state == "accepted" =>
            {
                Ok(value)
            }
            _ => Err("managed domain runtime start is unavailable".to_owned()),
        }
    }

    pub fn start_reserved_engine_runtime(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        storage_capability_id: &str,
    ) -> Result<StartReservedEngineRuntimeResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::StartReservedEngineRuntime(
                StartReservedEngineRuntimeRequestV1 {
                    registration_id: registration_id.to_owned(),
                    storage_capability_id: storage_capability_id.to_owned(),
                    owner_session_id: owner_session_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::StartReservedEngineRuntime(value))
                if value.registration_id == registration_id
                    && value.runtime_generation > 0
                    && value.launch_state == "accepted" =>
            {
                Ok(value)
            }
            _ => Err("managed engine runtime start is unavailable".to_owned()),
        }
    }

    pub fn start_reserved_workflow_runtime(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        storage_capability_id: &str,
    ) -> Result<StartReservedWorkflowRuntimeResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::StartReservedWorkflowRuntime(
                StartReservedWorkflowRuntimeRequestV1 {
                    registration_id: registration_id.to_owned(),
                    storage_capability_id: storage_capability_id.to_owned(),
                    owner_session_id: owner_session_id.to_owned(),
                    configuration_instance_id: String::new(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::StartReservedWorkflowRuntime(value))
                if value.registration_id == registration_id
                    && ((value.runtime_generation > 0 && value.launch_state == "accepted")
                        || (value.runtime_generation == 0
                            && value.launch_state == "unconfigured")) =>
            {
                Ok(value)
            }
            _ => Err("managed workflow runtime start is unavailable".to_owned()),
        }
    }

    pub fn start_reserved_workflow_configuration_runtime(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        storage_capability_id: &str,
        configuration_instance_id: &str,
    ) -> Result<StartReservedWorkflowRuntimeResponseV1, String> {
        if configuration_instance_id.is_empty() {
            return Err("managed workflow configuration instance is required".to_owned());
        }
        let response = self.request(
            owner_control_request_v1::Operation::StartReservedWorkflowRuntime(
                StartReservedWorkflowRuntimeRequestV1 {
                    registration_id: registration_id.to_owned(),
                    storage_capability_id: storage_capability_id.to_owned(),
                    owner_session_id: owner_session_id.to_owned(),
                    configuration_instance_id: configuration_instance_id.to_owned(),
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::StartReservedWorkflowRuntime(value))
                if value.registration_id == registration_id
                    && value.runtime_generation > 0
                    && value.launch_state == "accepted" =>
            {
                Ok(value)
            }
            _ => Err("managed workflow runtime start is unavailable".to_owned()),
        }
    }

    pub fn start_reserved_integration_runtime(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        storage_capability_id: &str,
        configuration_instance_id: &str,
        request_host_bridge: bool,
    ) -> Result<StartReservedIntegrationRuntimeResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::StartReservedIntegrationRuntime(
                StartReservedIntegrationRuntimeRequestV1 {
                    registration_id: registration_id.to_owned(),
                    storage_capability_id: storage_capability_id.to_owned(),
                    configuration_instance_id: configuration_instance_id.to_owned(),
                    owner_session_id: owner_session_id.to_owned(),
                    request_host_bridge,
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::StartReservedIntegrationRuntime(value))
                if value.registration_id == registration_id
                    && ((value.runtime_generation > 0 && value.launch_state == "accepted")
                        || (value.runtime_generation == 0
                            && value.launch_state == "unconfigured")) =>
            {
                Ok(value)
            }
            _ => Err("managed integration runtime start is unavailable".to_owned()),
        }
    }

    pub fn update_operator_settings(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        expected_revision: u64,
        snapshot_bytes: Vec<u8>,
    ) -> Result<UpdateOperatorSettingsResponseV1, String> {
        let response =
            self.request(owner_control_request_v1::Operation::UpdateOperatorSettings(
                UpdateOperatorSettingsRequestV1 {
                    registration_id: registration_id.to_owned(),
                    expected_revision,
                    snapshot_bytes,
                    owner_session_id: owner_session_id.to_owned(),
                },
            ))?;
        match response.result {
            Some(owner_control_response_v1::Result::UpdateOperatorSettings(value))
                if value.registration_id == registration_id
                    && value.desired_revision == expected_revision.saturating_add(1)
                    && value.apply_state == "pending_validation" =>
            {
                Ok(value)
            }
            _ => Err("operator settings update is unavailable".to_owned()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_managed_integration_settings(
        &self,
        owner_session_id: &str,
        registration_id: &str,
        storage_capability_id: &str,
        configuration_instance_id: &str,
        expected_desired_revision: u64,
        request_host_bridge: bool,
    ) -> Result<ApplyManagedIntegrationSettingsResponseV1, String> {
        let response = self.request(
            owner_control_request_v1::Operation::ApplyManagedIntegrationSettings(
                ApplyManagedIntegrationSettingsRequestV1 {
                    registration_id: registration_id.to_owned(),
                    storage_capability_id: storage_capability_id.to_owned(),
                    configuration_instance_id: configuration_instance_id.to_owned(),
                    expected_desired_revision,
                    owner_session_id: owner_session_id.to_owned(),
                    request_host_bridge,
                },
            ),
        )?;
        match response.result {
            Some(owner_control_response_v1::Result::ApplyManagedIntegrationSettings(value))
                if value.registration_id == registration_id
                    && value.effective_revision == expected_desired_revision
                    && value.runtime_generation > 0
                    && value.apply_state == "current" =>
            {
                Ok(value)
            }
            _ => Err("managed integration settings apply is unavailable".to_owned()),
        }
    }

    fn request(
        &self,
        operation: owner_control_request_v1::Operation,
    ) -> Result<OwnerControlResponseV1, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|_| "owner control socket is unavailable".to_owned())?;
        stream
            .set_read_timeout(Some(IPC_TIMEOUT))
            .and_then(|_| stream.set_write_timeout(Some(IPC_TIMEOUT)))
            .map_err(|_| "owner control socket is unavailable".to_owned())?;
        let request = OwnerControlRequestV1 {
            operation: Some(operation),
        };
        write_frame(&mut stream, &request.encode_to_vec())?;
        let response = OwnerControlResponseV1::decode(read_frame(&mut stream)?.as_slice())
            .map_err(|_| "owner control response is invalid".to_owned())?;
        response
            .error_code
            .is_empty()
            .then_some(response)
            .ok_or_else(|| "owner control operation was denied".to_owned())
    }
}

impl OwnerControlChallengeV1 {
    fn from_response(response: BeginOwnerControlSessionResponseV1) -> Result<Self, String> {
        let challenge_bytes: [u8; 32] = response
            .challenge_bytes
            .try_into()
            .map_err(|_| "owner control challenge is invalid".to_owned())?;
        (valid_identifier(&response.kernel_instance_id)
            && valid_identifier(&response.owner_id)
            && valid_identifier(&response.device_id)
            && response.challenge_id.len() == 64
            && response
                .challenge_id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            && response.control_store_generation > 0
            && response.expires_at_unix_millis > 0)
            .then_some(Self {
                challenge_id: response.challenge_id,
                challenge_bytes,
                kernel_instance_id: response.kernel_instance_id,
                owner_id: response.owner_id,
                device_id: response.device_id,
                control_store_generation: response.control_store_generation,
            })
            .ok_or_else(|| "owner control challenge is invalid".to_owned())
    }

    fn proof_message(&self) -> Result<Vec<u8>, String> {
        owner_control_proof_message_v1(
            &self.kernel_instance_id,
            &self.owner_id,
            &self.device_id,
            self.control_store_generation,
            &self.challenge_bytes,
        )
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "owner control frame is too large".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("owner control frame is too large".to_owned());
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
    Err("owner control frame length is invalid".to_owned())
}

fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| "owner control request is too large".to_owned())?;
    let mut prefix = [0_u8; 5];
    let mut index = 0;
    while length >= 0x80 {
        prefix[index] = (length as u8) | 0x80;
        length >>= 7;
        index += 1;
    }
    prefix[index] = length as u8;
    stream
        .write_all(&prefix[..=index])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}
