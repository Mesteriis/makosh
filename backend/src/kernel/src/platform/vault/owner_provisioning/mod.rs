//! Kernel authority for fresh-proof, write-only owner Vault provisioning.

mod authorization;
mod routes;
mod state;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_gateway_protocol::v1::{
    AuthorizeOwnerVaultProvisioningRequestV1, AuthorizeOwnerVaultProvisioningResponseV1,
    CommitOwnerVaultProvisioningRequestV1, CommitOwnerVaultProvisioningResponseV1,
    PrepareOwnerVaultProvisioningRequestV1, PrepareOwnerVaultProvisioningResponseV1,
};
use makosh_gateway_runtime::{
    OwnerVaultClientPrincipalV1, OwnerVaultProvisioningHandlerV1,
    OwnerVaultProvisioningRouteErrorV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::VaultCiphertextResponseV1;
use makosh_runtime_protocol::vault_request_id::next_vault_transport_request_id_v1;
use makosh_vault_protocol::{
    LeaseAudienceV1, VaultCiphertextFrameV1, VaultLeaseIssueRequestV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportPublicKey, seal,
};
use sha2::{Digest, Sha256};

use crate::platform::vault::status;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

use self::authorization::{authorize_target, verify_fresh_proof};
use self::routes::OwnerVaultRouteInputV1;
use self::state::{
    AuthorizedProvisioningSessionV1, CachedCommitV1, OwnerProvisioningStateV1, PendingChallengeV1,
};

const CHALLENGE_TTL_MILLIS: u64 = 120_000;
const SESSION_TTL_MILLIS: u64 = 300_000;

pub(crate) struct KernelOwnerVaultProvisioningHandlerV1 {
    store: Arc<SqliteControlStore>,
    data_dir: PathBuf,
    relay: ManagedRuntimeRelayPort,
    state: Mutex<OwnerProvisioningStateV1>,
}

impl KernelOwnerVaultProvisioningHandlerV1 {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        data_dir: &Path,
        relay: ManagedRuntimeRelayPort,
    ) -> Self {
        Self {
            store,
            data_dir: data_dir.to_path_buf(),
            relay,
            state: Mutex::new(OwnerProvisioningStateV1::default()),
        }
    }
}

impl OwnerVaultProvisioningHandlerV1 for KernelOwnerVaultProvisioningHandlerV1 {
    fn prepare(
        &self,
        principal: &OwnerVaultClientPrincipalV1,
        request: PrepareOwnerVaultProvisioningRequestV1,
    ) -> Result<PrepareOwnerVaultProvisioningResponseV1, OwnerVaultProvisioningRouteErrorV1> {
        let target = authorize_target(&self.store, principal, &request)?;
        let now = now_unix_millis()?;
        let expires_at_unix_millis = now
            .checked_add(CHALLENGE_TTL_MILLIS)
            .ok_or(OwnerVaultProvisioningRouteErrorV1::Internal)?;
        let challenge_id = random_identifier("ovp-challenge")?;
        let control_generation = self.store.snapshot().generation();
        let identity_epoch = self
            .store
            .current_identity_epoch()
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?;
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
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?
            .insert_pending(
                challenge_id.clone(),
                PendingChallengeV1 {
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
            .map_err(|()| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        Ok(PrepareOwnerVaultProvisioningResponseV1 {
            major: 1,
            challenge_id,
            challenge_bytes: challenge_bytes.to_vec(),
            expires_at_unix_millis,
        })
    }

    fn authorize(
        &self,
        principal: &OwnerVaultClientPrincipalV1,
        request: AuthorizeOwnerVaultProvisioningRequestV1,
    ) -> Result<AuthorizeOwnerVaultProvisioningResponseV1, OwnerVaultProvisioningRouteErrorV1> {
        let now = now_unix_millis()?;
        let pending = self
            .state
            .lock()
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?
            .take_pending(&request.challenge_id, now)
            .ok_or(OwnerVaultProvisioningRouteErrorV1::NotFound)?;
        require_same_principal(principal, &pending)?;
        if self.store.snapshot().generation() != pending.control_generation
            || self
                .store
                .current_identity_epoch()
                .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?
                != pending.identity_epoch
        {
            return Err(OwnerVaultProvisioningRouteErrorV1::Conflict);
        }
        verify_fresh_proof(
            &self.store,
            principal,
            &pending.challenge_bytes,
            &request.device_signature_raw,
        )?;
        let target = authorize_target(&self.store, principal, &pending.request)?;
        if target.grant_epoch != pending.grant_epoch {
            return Err(OwnerVaultProvisioningRouteErrorV1::Conflict);
        }
        let vault = status::read_current(&self.store, &self.relay)
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let audience_runtime_instance_id = random_identifier("owner-vault")?;
        let audience_runtime_generation = self.store.snapshot().generation();
        let audience = LeaseAudienceV1::new(
            pending.request.target_registration_id.clone(),
            audience_runtime_instance_id.clone(),
            audience_runtime_generation,
            target.grant_epoch,
        )
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?;
        let lease_request = VaultLeaseIssueRequestV1::new(
            self.store.snapshot().instance_id().to_owned(),
            vault.runtime_generation(),
            pending.request.secret_revision,
            target.logical_owner_id,
            target.purpose,
            audience,
        )
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::InvalidArgument)?;
        let command = VaultTransportCommandV1::IssueLease {
            request: lease_request,
        };
        let lease_request_id = next_vault_transport_request_id_v1()
            .ok_or(OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let command_request_id = next_vault_transport_request_id_v1()
            .ok_or(OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let vault_key = VaultTransportPublicKey::from_bytes(*vault.hpke_public_key_x25519())
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let lease_binding = VaultTransportBindingV1::new(
            vault.runtime_generation(),
            LeaseAudienceV1::new(
                pending.request.target_registration_id.clone(),
                audience_runtime_instance_id.clone(),
                audience_runtime_generation,
                target.grant_epoch,
            )
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?,
            lease_request_id,
            command.operation_digest(),
            VaultTransportDirectionV1::ToVault,
            target.response_recipient_public_key,
        )
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?;
        let lease_frame = seal(&vault_key, &lease_binding, &command.encode())
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let lease_response = routes::relay(
            &self.store,
            &self.data_dir,
            &self.relay,
            OwnerVaultRouteInputV1 {
                registration_id: &pending.request.target_registration_id,
                runtime_instance_id: &audience_runtime_instance_id,
                runtime_generation: audience_runtime_generation,
                grant_epoch: target.grant_epoch,
                vault_runtime_generation: vault.runtime_generation(),
                request_id: lease_request_id,
                operation_digest_sha256: command.operation_digest(),
                response_recipient_public_key: target.response_recipient_public_key,
                frame: lease_frame,
            },
        )
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let expires_at_unix_millis = now
            .checked_add(SESSION_TTL_MILLIS)
            .ok_or(OwnerVaultProvisioningRouteErrorV1::Internal)?;
        let provisioning_session_id = random_identifier("ovp-session")?;
        self.state
            .lock()
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?
            .insert_authorized(
                provisioning_session_id.clone(),
                AuthorizedProvisioningSessionV1 {
                    principal_owner_id: principal.owner_id().to_owned(),
                    principal_device_id: principal.device_id().to_owned(),
                    principal_session_id: principal.session_id().to_owned(),
                    request: pending.request,
                    expires_at_unix_millis,
                    vault_runtime_generation: vault.runtime_generation(),
                    audience_registration_id: lease_binding
                        .audience()
                        .module_registration_id()
                        .to_owned(),
                    audience_runtime_instance_id,
                    audience_runtime_generation,
                    audience_grant_epoch: target.grant_epoch,
                    command_request_id,
                    cached_commit: None,
                },
                now,
            )
            .map_err(|()| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        Ok(authorize_response(
            provisioning_session_id,
            expires_at_unix_millis,
            vault.runtime_generation(),
            vault.hpke_public_key_x25519(),
            &lease_binding,
            command_request_id,
            lease_response,
        ))
    }

    fn commit(
        &self,
        principal: &OwnerVaultClientPrincipalV1,
        request: CommitOwnerVaultProvisioningRequestV1,
    ) -> Result<CommitOwnerVaultProvisioningResponseV1, OwnerVaultProvisioningRouteErrorV1> {
        let now = now_unix_millis()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let session = state
            .authorized(&request.provisioning_session_id, now)
            .ok_or(OwnerVaultProvisioningRouteErrorV1::NotFound)?;
        require_same_authorized_principal(principal, session)?;
        let operation_digest_sha256 = request
            .operation_digest_sha256
            .as_slice()
            .try_into()
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::InvalidArgument)?;
        let request_fingerprint_sha256 = commit_request_fingerprint(
            &operation_digest_sha256,
            &request.hpke_encapped_key,
            &request.ciphertext,
            &request.hpke_authentication_tag,
        );
        let target = authorize_target(&self.store, principal, &session.request)?;
        let vault = status::read_current(&self.store, &self.relay)
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        if vault.runtime_generation() != session.vault_runtime_generation
            || target.grant_epoch != session.audience_grant_epoch
            || self.store.snapshot().generation() != session.audience_runtime_generation
        {
            return Err(OwnerVaultProvisioningRouteErrorV1::Conflict);
        }
        if let Some(cached) = &session.cached_commit {
            return (cached.request_fingerprint_sha256 == request_fingerprint_sha256)
                .then(|| cached.response.clone())
                .ok_or(OwnerVaultProvisioningRouteErrorV1::Conflict);
        }
        let frame = VaultCiphertextFrameV1::from_parts(
            request.hpke_encapped_key,
            request.ciphertext,
            request.hpke_authentication_tag,
        )
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::InvalidArgument)?;
        let response = routes::relay(
            &self.store,
            &self.data_dir,
            &self.relay,
            OwnerVaultRouteInputV1 {
                registration_id: &session.audience_registration_id,
                runtime_instance_id: &session.audience_runtime_instance_id,
                runtime_generation: session.audience_runtime_generation,
                grant_epoch: session.audience_grant_epoch,
                vault_runtime_generation: session.vault_runtime_generation,
                request_id: session.command_request_id,
                operation_digest_sha256,
                response_recipient_public_key: target.response_recipient_public_key,
                frame,
            },
        )
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
        let response = commit_response(
            session.vault_runtime_generation,
            session.command_request_id,
            operation_digest_sha256,
            response,
        );
        session.cached_commit = Some(CachedCommitV1 {
            request_fingerprint_sha256,
            response: response.clone(),
        });
        Ok(response)
    }
}

fn authorize_response(
    provisioning_session_id: String,
    expires_at_unix_millis: u64,
    vault_runtime_generation: u64,
    vault_hpke_public_key_x25519: &[u8; 32],
    lease_binding: &VaultTransportBindingV1,
    command_request_id: [u8; 16],
    response: VaultCiphertextResponseV1,
) -> AuthorizeOwnerVaultProvisioningResponseV1 {
    AuthorizeOwnerVaultProvisioningResponseV1 {
        major: 1,
        provisioning_session_id,
        expires_at_unix_millis,
        vault_runtime_generation,
        vault_hpke_public_key_x25519: vault_hpke_public_key_x25519.to_vec(),
        audience_registration_id: lease_binding.audience().module_registration_id().to_owned(),
        audience_runtime_instance_id: lease_binding.audience().runtime_instance_id().to_owned(),
        audience_runtime_generation: lease_binding.audience().runtime_generation(),
        audience_grant_epoch: lease_binding.audience().grant_epoch(),
        lease_request_id: lease_binding.request_id().to_vec(),
        lease_operation_digest_sha256: lease_binding.operation_digest().to_vec(),
        command_request_id: command_request_id.to_vec(),
        lease_response_hpke_encapped_key: response.hpke_encapped_key,
        lease_response_ciphertext: response.ciphertext,
        lease_response_hpke_authentication_tag: response.hpke_authentication_tag,
    }
}

fn commit_response(
    vault_runtime_generation: u64,
    command_request_id: [u8; 16],
    operation_digest_sha256: [u8; 32],
    response: VaultCiphertextResponseV1,
) -> CommitOwnerVaultProvisioningResponseV1 {
    CommitOwnerVaultProvisioningResponseV1 {
        major: 1,
        vault_runtime_generation,
        command_request_id: command_request_id.to_vec(),
        operation_digest_sha256: operation_digest_sha256.to_vec(),
        receipt_hpke_encapped_key: response.hpke_encapped_key,
        receipt_ciphertext: response.ciphertext,
        receipt_hpke_authentication_tag: response.hpke_authentication_tag,
    }
}

fn require_same_principal(
    principal: &OwnerVaultClientPrincipalV1,
    pending: &PendingChallengeV1,
) -> Result<(), OwnerVaultProvisioningRouteErrorV1> {
    (pending.principal_owner_id == principal.owner_id()
        && pending.principal_device_id == principal.device_id()
        && pending.principal_session_id == principal.session_id())
    .then_some(())
    .ok_or(OwnerVaultProvisioningRouteErrorV1::PermissionDenied)
}

fn require_same_authorized_principal(
    principal: &OwnerVaultClientPrincipalV1,
    session: &AuthorizedProvisioningSessionV1,
) -> Result<(), OwnerVaultProvisioningRouteErrorV1> {
    (session.principal_owner_id == principal.owner_id()
        && session.principal_device_id == principal.device_id()
        && session.principal_session_id == principal.session_id())
    .then_some(())
    .ok_or(OwnerVaultProvisioningRouteErrorV1::PermissionDenied)
}

fn challenge_digest(
    principal: &OwnerVaultClientPrincipalV1,
    request: &PrepareOwnerVaultProvisioningRequestV1,
    control_generation: u64,
    identity_epoch: u64,
    grant_epoch: u64,
    nonce: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.owner-vault-provisioning.challenge.v1");
    digest_text(&mut digest, principal.owner_id());
    digest_text(&mut digest, principal.device_id());
    digest_text(&mut digest, principal.session_id());
    digest_field(&mut digest, &request.operation_id);
    digest_text(&mut digest, &request.target_registration_id);
    digest_text(&mut digest, &request.capability_id);
    digest_text(&mut digest, &request.configuration_instance_id);
    digest_text(&mut digest, &request.purpose_id);
    digest.update(request.secret_class.to_be_bytes());
    digest.update(request.action.to_be_bytes());
    digest.update(request.secret_revision.to_be_bytes());
    digest_field(
        &mut digest,
        &request.response_recipient_hpke_public_key_x25519,
    );
    digest.update(control_generation.to_be_bytes());
    digest.update(identity_epoch.to_be_bytes());
    digest.update(grant_epoch.to_be_bytes());
    digest.update(nonce);
    digest.finalize().into()
}

fn commit_request_fingerprint(
    operation_digest_sha256: &[u8; 32],
    hpke_encapped_key: &[u8],
    ciphertext: &[u8],
    hpke_authentication_tag: &[u8],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.owner-vault-provisioning.commit.v1");
    digest.update(operation_digest_sha256);
    digest_field(&mut digest, hpke_encapped_key);
    digest_field(&mut digest, ciphertext);
    digest_field(&mut digest, hpke_authentication_tag);
    digest.finalize().into()
}

fn digest_text(digest: &mut Sha256, value: &str) {
    digest_field(digest, value.as_bytes());
}

fn digest_field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn random_identifier(prefix: &str) -> Result<String, OwnerVaultProvisioningRouteErrorV1> {
    let bytes = random_bytes::<16>()?;
    let mut value = String::with_capacity(prefix.len() + 1 + bytes.len() * 2);
    value.push_str(prefix);
    value.push('-');
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut value, "{byte:02x}")
            .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?;
    }
    Ok(value)
}

fn random_bytes<const N: usize>() -> Result<[u8; N], OwnerVaultProvisioningRouteErrorV1> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes).map_err(|_| OwnerVaultProvisioningRouteErrorV1::Unavailable)?;
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(OwnerVaultProvisioningRouteErrorV1::Unavailable);
    }
    Ok(bytes)
}

fn now_unix_millis() -> Result<u64, OwnerVaultProvisioningRouteErrorV1> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?
        .as_millis()
        .try_into()
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)
}
