//! Fresh-proof public owner application for Kernel-owned module Settings.

mod authorization;
mod export;
mod operation;
mod state;
mod target;
mod values;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_gateway_protocol::v1::{
    CommitOwnerModuleSettingsRequestV1, CommitOwnerModuleSettingsResponseV1,
    PrepareOwnerModuleSettingsRequestV1, PrepareOwnerModuleSettingsResponseV1,
    prepare_owner_module_settings_request_v1,
};
use makosh_gateway_runtime::{
    OwnerBrowserPrincipalV1, OwnerModuleSettingsHandlerV1, OwnerModuleSettingsRouteErrorV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use prost::Message;
use sha2::{Digest, Sha256};

use self::authorization::{authorize_target, map_proof_error};
use self::state::{OwnerSettingsChallengeStateV1, PendingOwnerSettingsChallengeV1};
use crate::platform::gateway::owner_device_proof;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const CHALLENGE_TTL_MILLIS: u64 = 120_000;

pub(crate) struct KernelOwnerModuleSettingsHandlerV1 {
    store: Arc<SqliteControlStore>,
    data_dir: PathBuf,
    runtime_dir: PathBuf,
    supervisor: ManagedRuntimeSupervisor,
    state: Mutex<OwnerSettingsChallengeStateV1>,
}

impl KernelOwnerModuleSettingsHandlerV1 {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        data_dir: &Path,
        runtime_dir: &Path,
        supervisor: ManagedRuntimeSupervisor,
    ) -> Self {
        Self {
            store,
            data_dir: data_dir.to_path_buf(),
            runtime_dir: runtime_dir.to_path_buf(),
            supervisor,
            state: Mutex::new(OwnerSettingsChallengeStateV1::default()),
        }
    }
}

impl OwnerModuleSettingsHandlerV1 for KernelOwnerModuleSettingsHandlerV1 {
    fn prepare(
        &self,
        principal: &OwnerBrowserPrincipalV1,
        request: PrepareOwnerModuleSettingsRequestV1,
    ) -> Result<PrepareOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
        let target = authorize_target(&self.store, principal, &request)?;
        let now = now_unix_millis()?;
        let expires_at_unix_millis = now
            .checked_add(CHALLENGE_TTL_MILLIS)
            .ok_or(OwnerModuleSettingsRouteErrorV1::Internal)?;
        let challenge_id = random_identifier("oms-challenge")?;
        let control_generation = self.store.snapshot().generation();
        let identity_epoch = self
            .store
            .current_identity_epoch()
            .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?;
        let challenge_bytes = challenge_digest(
            principal,
            &request,
            control_generation,
            identity_epoch,
            target.grant_epoch,
            &random_bytes::<32>()?,
        );
        self.state
            .lock()
            .map_err(|_| OwnerModuleSettingsRouteErrorV1::Unavailable)?
            .insert(
                challenge_id.clone(),
                PendingOwnerSettingsChallengeV1 {
                    principal_owner_id: principal.owner_id().to_owned(),
                    principal_device_id: principal.device_id().to_owned(),
                    principal_session_id: principal.session_id().to_owned(),
                    request,
                    challenge_bytes,
                    control_generation,
                    identity_epoch,
                    grant_epoch: target.grant_epoch,
                    expires_at_unix_millis,
                },
                now,
            )
            .map_err(|()| OwnerModuleSettingsRouteErrorV1::Unavailable)?;
        Ok(PrepareOwnerModuleSettingsResponseV1 {
            major: 1,
            challenge_id,
            challenge_bytes: challenge_bytes.to_vec(),
            expires_at_unix_millis,
        })
    }

    fn commit(
        &self,
        principal: &OwnerBrowserPrincipalV1,
        request: CommitOwnerModuleSettingsRequestV1,
    ) -> Result<CommitOwnerModuleSettingsResponseV1, OwnerModuleSettingsRouteErrorV1> {
        let now = now_unix_millis()?;
        let pending = self
            .state
            .lock()
            .map_err(|_| OwnerModuleSettingsRouteErrorV1::Unavailable)?
            .take(&request.challenge_id, now)
            .ok_or(OwnerModuleSettingsRouteErrorV1::NotFound)?;
        require_same_principal(principal, &pending)?;
        if self.store.snapshot().generation() != pending.control_generation
            || self
                .store
                .current_identity_epoch()
                .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)?
                != pending.identity_epoch
        {
            return Err(OwnerModuleSettingsRouteErrorV1::Conflict);
        }
        owner_device_proof::verify_fresh_proof(
            &self.store,
            principal,
            &pending.challenge_bytes,
            &request.device_signature_raw,
        )
        .map_err(map_proof_error)?;
        let target = authorize_target(&self.store, principal, &pending.request)?;
        if target.registration_id != operation_registration_id(&pending.request)?
            || target.grant_epoch != pending.grant_epoch
        {
            return Err(OwnerModuleSettingsRouteErrorV1::Conflict);
        }
        let operation_id = pending.request.operation_id;
        match pending.request.operation {
            Some(prepare_owner_module_settings_request_v1::Operation::UpdateDesired(update)) => {
                operation::update_desired(&self.store, operation_id, update)
            }
            Some(prepare_owner_module_settings_request_v1::Operation::ApplyManagedIntegration(
                apply,
            )) => operation::apply_managed_integration(
                &self.store,
                &self.data_dir,
                &self.runtime_dir,
                &self.supervisor,
                operation_id,
                apply,
            ),
            Some(prepare_owner_module_settings_request_v1::Operation::ApplyManagedWorkflow(
                apply,
            )) => operation::apply_managed_workflow(
                &self.store,
                &self.runtime_dir,
                &self.supervisor,
                principal.owner_id(),
                operation_id,
                apply,
            ),
            Some(prepare_owner_module_settings_request_v1::Operation::ExportEffective(export)) => {
                export::effective(&self.store, operation_id, export)
            }
            Some(
                prepare_owner_module_settings_request_v1::Operation::CreateConfigurationTarget(
                    create,
                ),
            ) => target::create(&self.store, operation_id, create),
            None => Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument),
        }
    }
}

fn require_same_principal(
    principal: &OwnerBrowserPrincipalV1,
    pending: &PendingOwnerSettingsChallengeV1,
) -> Result<(), OwnerModuleSettingsRouteErrorV1> {
    if principal.owner_id() != pending.principal_owner_id
        || principal.device_id() != pending.principal_device_id
        || principal.session_id() != pending.principal_session_id
    {
        return Err(OwnerModuleSettingsRouteErrorV1::PermissionDenied);
    }
    Ok(())
}

fn operation_registration_id(
    request: &PrepareOwnerModuleSettingsRequestV1,
) -> Result<&str, OwnerModuleSettingsRouteErrorV1> {
    match request.operation.as_ref() {
        Some(prepare_owner_module_settings_request_v1::Operation::UpdateDesired(update)) => {
            Ok(&update.registration_id)
        }
        Some(prepare_owner_module_settings_request_v1::Operation::ApplyManagedIntegration(
            apply,
        )) => Ok(&apply.registration_id),
        Some(prepare_owner_module_settings_request_v1::Operation::ApplyManagedWorkflow(apply)) => {
            Ok(&apply.registration_id)
        }
        Some(prepare_owner_module_settings_request_v1::Operation::ExportEffective(export)) => {
            Ok(&export.registration_id)
        }
        Some(prepare_owner_module_settings_request_v1::Operation::CreateConfigurationTarget(
            create,
        )) => Ok(&create.registration_id),
        None => Err(OwnerModuleSettingsRouteErrorV1::InvalidArgument),
    }
}

fn challenge_digest(
    principal: &OwnerBrowserPrincipalV1,
    request: &PrepareOwnerModuleSettingsRequestV1,
    control_generation: u64,
    identity_epoch: u64,
    grant_epoch: u64,
    nonce: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest_field(&mut digest, b"makosh.owner_module_settings.challenge.v1");
    digest_field(&mut digest, principal.owner_id().as_bytes());
    digest_field(&mut digest, principal.device_id().as_bytes());
    digest_field(&mut digest, principal.session_id().as_bytes());
    digest_field(&mut digest, &request.encode_to_vec());
    digest_field(&mut digest, &control_generation.to_be_bytes());
    digest_field(&mut digest, &identity_epoch.to_be_bytes());
    digest_field(&mut digest, &grant_epoch.to_be_bytes());
    digest_field(&mut digest, nonce);
    digest.finalize().into()
}

fn digest_field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn random_identifier(prefix: &str) -> Result<String, OwnerModuleSettingsRouteErrorV1> {
    let bytes = random_bytes::<16>()?;
    Ok(format!(
        "{prefix}-{}",
        bytes
            .iter()
            .map(|value| format!("{value:02x}"))
            .collect::<String>()
    ))
}

fn random_bytes<const N: usize>() -> Result<[u8; N], OwnerModuleSettingsRouteErrorV1> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes).map_err(|_| OwnerModuleSettingsRouteErrorV1::Unavailable)?;
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(OwnerModuleSettingsRouteErrorV1::Unavailable);
    }
    Ok(bytes)
}

fn now_unix_millis() -> Result<u64, OwnerModuleSettingsRouteErrorV1> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)
        .and_then(|duration| {
            duration
                .as_millis()
                .try_into()
                .map_err(|_| OwnerModuleSettingsRouteErrorV1::Internal)
        })
}
