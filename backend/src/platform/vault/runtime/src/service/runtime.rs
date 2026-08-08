//! Vault lease and secret-resolution core, independent from IPC transport.

use getrandom::fill;
use makosh_vault_protocol::{
    CredentialLeaseV1, LeaseAudienceV1, LeaseIdV1, VaultActionV1, VaultLeaseIssueRequestV1,
    VaultTransportCommandV1,
};
use makosh_vault_store_sqlcipher::{
    SecretRecordId, SecretRecordScope, VaultProvisioningMutationV1, VaultStore,
};
use zeroize::Zeroizing;

use crate::service::leases::{LeaseError, LeaseManager};

pub struct VaultService {
    store: VaultStore,
    leases: LeaseManager,
}

pub struct VaultSecretReplaceRequestV1<'a> {
    pub lease_id: &'a LeaseIdV1,
    pub audience: &'a LeaseAudienceV1,
    pub prior_record_id: &'a SecretRecordId,
    pub prior_scope: &'a SecretRecordScope,
    pub next_scope: &'a SecretRecordScope,
    pub payload: &'a [u8],
    pub now_unix_seconds: u64,
}

struct VaultProvisioningRequestV1<'a> {
    lease_id: &'a LeaseIdV1,
    operation_id: [u8; 16],
    action: VaultActionV1,
    secret_class: makosh_vault_protocol::SecretClassV1,
    payload: &'a [u8],
}

impl VaultService {
    pub fn new(store: VaultStore, runtime_generation: u64) -> Result<Self, VaultServiceError> {
        let leases = LeaseManager::new(store.instance_id().to_owned(), runtime_generation)
            .map_err(VaultServiceError::Lease)?;
        Ok(Self { store, leases })
    }

    pub fn issue_lease(
        &mut self,
        request: VaultLeaseIssueRequestV1,
        now_unix_seconds: u64,
    ) -> Result<CredentialLeaseV1, VaultServiceError> {
        if request
            .purpose()
            .actions()
            .iter()
            .any(|action| !supported_action(*action))
        {
            return Err(VaultServiceError::UnsupportedLeaseAction);
        }
        self.leases
            .issue(request, now_unix_seconds)
            .map_err(VaultServiceError::Lease)
    }

    pub fn resolve_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        record_id: &SecretRecordId,
        scope: &SecretRecordScope,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Resolve, now_unix_seconds)?;
        if !scope.matches_lease_request(lease.request()) {
            return Err(VaultServiceError::LeaseScopeMismatch);
        }
        self.store
            .resolve_scoped_secret(record_id, scope)
            .map_err(|_| VaultServiceError::SecretUnavailable)
    }

    pub fn replace_once(
        &mut self,
        request: VaultSecretReplaceRequestV1<'_>,
    ) -> Result<SecretRecordId, VaultServiceError> {
        let lease = self.consume_action(
            request.lease_id,
            request.audience,
            VaultActionV1::ReplaceCas,
            request.now_unix_seconds,
        )?;
        if !request.next_scope.matches_lease_request(lease.request()) {
            return Err(VaultServiceError::LeaseScopeMismatch);
        }
        self.store
            .replace_secret(
                request.prior_record_id,
                request.prior_scope,
                request.next_scope,
                request.payload,
            )
            .map_err(|_| VaultServiceError::SecretUnavailable)
    }

    pub fn execute_command_once(
        &mut self,
        command: &VaultTransportCommandV1,
        audience: &LeaseAudienceV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        log_developer_command(command);
        match command {
            VaultTransportCommandV1::RevokeAudience => {
                self.revoke_audience(audience);
                Ok(Zeroizing::new(vec![1]))
            }
            VaultTransportCommandV1::IssueLease { request } => self
                .issue_transport_lease(request.clone(), audience, now_unix_seconds)
                .map(|lease| Zeroizing::new(lease.lease_id().as_str().as_bytes().to_vec())),
            VaultTransportCommandV1::ResolveLease {
                lease_id,
                secret_class,
            } => self.resolve_current_once(lease_id, audience, *secret_class, now_unix_seconds),
            VaultTransportCommandV1::RetireLease {
                lease_id,
                secret_class,
            } => self.retire_current_once(lease_id, audience, *secret_class, now_unix_seconds),
            VaultTransportCommandV1::DeleteLease {
                lease_id,
                secret_class,
            } => self.delete_current_once(lease_id, audience, *secret_class, now_unix_seconds),
            VaultTransportCommandV1::StoreLease {
                lease_id,
                secret_class,
                payload,
            } => self.store_current_once(
                lease_id,
                audience,
                *secret_class,
                payload,
                now_unix_seconds,
            ),
            VaultTransportCommandV1::GenerateOpaqueToken {
                lease_id,
                secret_class,
            } => {
                self.generate_opaque_token_once(lease_id, audience, *secret_class, now_unix_seconds)
            }
            VaultTransportCommandV1::EnsureOwnerDerivedKey { lease_id } => {
                self.ensure_owner_derived_key_once(lease_id, audience, now_unix_seconds)
            }
            VaultTransportCommandV1::ReplaceLease {
                lease_id,
                secret_class,
                prior_record_id,
                payload,
            } => self.replace_current_once(
                lease_id,
                audience,
                *secret_class,
                prior_record_id,
                payload,
                now_unix_seconds,
            ),
            VaultTransportCommandV1::ProvisionLease {
                lease_id,
                operation_id,
                action,
                secret_class,
                payload,
            } => self.provision_current_once(
                &VaultProvisioningRequestV1 {
                    lease_id,
                    operation_id: *operation_id,
                    action: *action,
                    secret_class: *secret_class,
                    payload,
                },
                audience,
                now_unix_seconds,
            ),
        }
    }

    pub fn revoke_audience(&mut self, audience: &LeaseAudienceV1) {
        self.leases.invalidate_audience(audience);
    }

    fn issue_transport_lease(
        &mut self,
        request: VaultLeaseIssueRequestV1,
        audience: &LeaseAudienceV1,
        now_unix_seconds: u64,
    ) -> Result<CredentialLeaseV1, VaultServiceError> {
        if request.audience() != audience {
            return Err(VaultServiceError::LeaseScopeMismatch);
        }
        self.issue_lease(request, now_unix_seconds)
    }

    pub fn advance_runtime_generation(
        &mut self,
        next_generation: u64,
    ) -> Result<(), VaultServiceError> {
        self.leases
            .advance_generation(next_generation)
            .map_err(VaultServiceError::Lease)
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.leases.runtime_generation()
    }

    fn consume_action(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        action: VaultActionV1,
        now_unix_seconds: u64,
    ) -> Result<CredentialLeaseV1, VaultServiceError> {
        let lease = self
            .leases
            .consume_once(lease_id, audience, now_unix_seconds)
            .map_err(VaultServiceError::Lease)?;
        if lease.request().purpose().actions().contains(&action) {
            Ok(lease)
        } else {
            Err(VaultServiceError::LeaseActionDenied)
        }
    }

    fn resolve_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: makosh_vault_protocol::SecretClassV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Resolve, now_unix_seconds)?;
        let scope = SecretRecordScope::new(
            lease.request().logical_owner_id().to_owned(),
            lease.request().purpose(),
            secret_class,
            lease.request().secret_revision(),
        )
        .map_err(|_| VaultServiceError::LeaseScopeMismatch)?;
        match self.store.resolve_current_secret(&scope) {
            Ok(credential) => Ok(credential),
            Err(error) => {
                if error.is_malformed_record() {
                    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                        eprintln!("developer_vault_recover_malformed_record");
                    }
                    let _ = self.store.delete_secret(&scope, now_unix_seconds);
                } else if error.is_record_not_found()
                    && std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some()
                {
                    eprintln!("developer_vault_missing_record");
                }
                if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                    eprintln!("developer_vault_resolve_runtime_credential_error={error:?}");
                }
                Err(VaultServiceError::SecretUnavailable)
            }
        }
    }

    fn store_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: makosh_vault_protocol::SecretClassV1,
        payload: &[u8],
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Create, now_unix_seconds)?;
        let scope = SecretRecordScope::new(
            lease.request().logical_owner_id().to_owned(),
            lease.request().purpose(),
            secret_class,
            lease.request().secret_revision(),
        )
        .map_err(|_| VaultServiceError::LeaseScopeMismatch)?;
        let record_id = self
            .store
            .store_secret(&scope, payload)
            .map_err(|_| VaultServiceError::SecretUnavailable)?;
        Ok(Zeroizing::new(record_id.as_bytes().to_vec()))
    }

    fn retire_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: makosh_vault_protocol::SecretClassV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Retire, now_unix_seconds)?;
        let scope = scope_for_lease(&lease, secret_class, lease.request().secret_revision())?;
        self.store
            .retire_secret(&scope, now_unix_seconds)
            .map_err(|_| VaultServiceError::SecretUnavailable)?;
        Ok(Zeroizing::new(vec![1]))
    }

    fn delete_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: makosh_vault_protocol::SecretClassV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Delete, now_unix_seconds)?;
        let scope = scope_for_lease(&lease, secret_class, lease.request().secret_revision())?;
        self.store
            .delete_secret(&scope, now_unix_seconds)
            .map_err(|_| VaultServiceError::SecretUnavailable)?;
        Ok(Zeroizing::new(vec![1]))
    }

    fn generate_opaque_token_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: makosh_vault_protocol::SecretClassV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease = self.consume_action(
            lease_id,
            audience,
            generate_action(secret_class)?,
            now_unix_seconds,
        )?;
        let scope = scope_for_lease(&lease, secret_class, lease.request().secret_revision())?;
        let token = generate_secret_material(secret_class)?;
        let record_id = self
            .store
            .store_secret(&scope, token.as_slice())
            .map_err(|error| {
                if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                    eprintln!("developer_vault_store_runtime_credential_error={error:?}");
                }
                VaultServiceError::SecretUnavailable
            })?;
        Ok(Zeroizing::new(record_id.as_bytes().to_vec()))
    }

    fn replace_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: makosh_vault_protocol::SecretClassV1,
        prior_record_id: &[u8; 16],
        payload: &[u8],
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease = self.consume_action(
            lease_id,
            audience,
            VaultActionV1::ReplaceCas,
            now_unix_seconds,
        )?;
        let next_scope = scope_for_lease(&lease, secret_class, lease.request().secret_revision())?;
        let prior_revision = lease
            .request()
            .secret_revision()
            .checked_sub(1)
            .filter(|revision| *revision > 0)
            .ok_or(VaultServiceError::LeaseScopeMismatch)?;
        let prior_scope = scope_for_lease(&lease, secret_class, prior_revision)?;
        let record_id = self
            .store
            .replace_secret(
                &SecretRecordId::from_bytes(*prior_record_id),
                &prior_scope,
                &next_scope,
                payload,
            )
            .map_err(|_| VaultServiceError::SecretUnavailable)?;
        Ok(Zeroizing::new(record_id.as_bytes().to_vec()))
    }

    fn provision_current_once(
        &mut self,
        request: &VaultProvisioningRequestV1<'_>,
        audience: &LeaseAudienceV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(request.lease_id, audience, request.action, now_unix_seconds)?;
        let scope = scope_for_lease(
            &lease,
            request.secret_class,
            lease.request().secret_revision(),
        )?;
        let mutation = VaultProvisioningMutationV1::new(
            request.operation_id,
            request.action,
            scope,
            request.payload.to_vec(),
            now_unix_seconds,
        )
        .map_err(|_| VaultServiceError::LeaseScopeMismatch)?;
        self.store
            .provision(mutation)
            .map(|receipt| Zeroizing::new(receipt.encode()))
            .map_err(|_| VaultServiceError::SecretUnavailable)
    }

    fn ensure_owner_derived_key_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease = self.consume_action(
            lease_id,
            audience,
            VaultActionV1::IssueOwnerDerivedKey,
            now_unix_seconds,
        )?;
        let scope = scope_for_lease(
            &lease,
            makosh_vault_protocol::SecretClassV1::OwnerDerivedKey,
            lease.request().secret_revision(),
        )?;
        if let Ok(existing) = self.store.resolve_current_secret(&scope) {
            return Ok(existing);
        }
        let key = generate_secret_material(makosh_vault_protocol::SecretClassV1::OwnerDerivedKey)?;
        match self.store.store_secret(&scope, key.as_slice()) {
            Ok(_) => Ok(key),
            Err(_) => self
                .store
                .resolve_current_secret(&scope)
                .map_err(|_| VaultServiceError::SecretUnavailable),
        }
    }
}

fn log_developer_command(command: &VaultTransportCommandV1) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_none() {
        return;
    }
    let name = match command {
        VaultTransportCommandV1::RevokeAudience => "revoke_audience",
        VaultTransportCommandV1::IssueLease { .. } => "issue_lease",
        VaultTransportCommandV1::ResolveLease { .. } => "resolve_lease",
        VaultTransportCommandV1::RetireLease { .. } => "retire_lease",
        VaultTransportCommandV1::DeleteLease { .. } => "delete_lease",
        VaultTransportCommandV1::StoreLease { .. } => "store_lease",
        VaultTransportCommandV1::GenerateOpaqueToken { .. } => "generate_opaque_token",
        VaultTransportCommandV1::EnsureOwnerDerivedKey { .. } => "ensure_owner_derived_key",
        VaultTransportCommandV1::ReplaceLease { .. } => "replace_lease",
        VaultTransportCommandV1::ProvisionLease { .. } => "provision_lease",
    };
    eprintln!("developer_vault_command={name}");
}

fn scope_for_lease(
    lease: &CredentialLeaseV1,
    secret_class: makosh_vault_protocol::SecretClassV1,
    revision: u64,
) -> Result<SecretRecordScope, VaultServiceError> {
    SecretRecordScope::new(
        lease.request().logical_owner_id().to_owned(),
        lease.request().purpose(),
        secret_class,
        revision,
    )
    .map_err(|_| VaultServiceError::LeaseScopeMismatch)
}

fn supported_action(action: VaultActionV1) -> bool {
    matches!(
        action,
        VaultActionV1::Resolve
            | VaultActionV1::Create
            | VaultActionV1::ReplaceCas
            | VaultActionV1::Retire
            | VaultActionV1::Delete
            | VaultActionV1::IssueOwnerDerivedKey
    )
}

fn generate_action(
    secret_class: makosh_vault_protocol::SecretClassV1,
) -> Result<VaultActionV1, VaultServiceError> {
    match secret_class {
        makosh_vault_protocol::SecretClassV1::PlatformCredential => Ok(VaultActionV1::Create),
        makosh_vault_protocol::SecretClassV1::OwnerDerivedKey => {
            Ok(VaultActionV1::IssueOwnerDerivedKey)
        }
        _ => Err(VaultServiceError::LeaseScopeMismatch),
    }
}

fn generate_secret_material(
    secret_class: makosh_vault_protocol::SecretClassV1,
) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
    match secret_class {
        makosh_vault_protocol::SecretClassV1::PlatformCredential => generate_opaque_token(),
        makosh_vault_protocol::SecretClassV1::OwnerDerivedKey => {
            let mut key = Zeroizing::new(vec![0_u8; 32]);
            fill(key.as_mut_slice()).map_err(|_| VaultServiceError::EntropyUnavailable)?;
            Ok(key)
        }
        _ => Err(VaultServiceError::LeaseScopeMismatch),
    }
}

fn generate_opaque_token() -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
    let mut entropy = Zeroizing::new([0_u8; 32]);
    fill(entropy.as_mut()).map_err(|_| VaultServiceError::EntropyUnavailable)?;
    let mut token = Zeroizing::new(Vec::with_capacity(entropy.len() * 2));
    for byte in entropy.iter().copied() {
        token.push(hex_digit(byte >> 4));
        token.push(hex_digit(byte & 0x0f));
    }
    Ok(token)
}

const fn hex_digit(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        _ => b'a' + (value - 10),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum VaultServiceError {
    Lease(LeaseError),
    UnsupportedLeaseAction,
    LeaseActionDenied,
    LeaseScopeMismatch,
    SecretUnavailable,
    EntropyUnavailable,
}
