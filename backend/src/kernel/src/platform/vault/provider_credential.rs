//! Authorizes one descriptor-declared provider credential lease without exposing plaintext.

use std::sync::Arc;

use makosh_kernel_control_store::{ModuleRegistrationState, ModuleVaultPurposeRequestV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    ManagedRuntimeProviderCredentialDeliveryV1, ManagedRuntimeProviderCredentialRequestV1,
    VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1, VaultPurposeRequestV1,
    VaultTransportBindingV1, VaultTransportCommandV1, VaultTransportDirectionV1,
    VaultTransportPublicKey, seal,
};

use crate::platform::vault::{managed_route::KernelManagedVaultRouteHandler, status};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeExpectation, ManagedRuntimeProviderCredentialHandler,
    ManagedRuntimeVaultRouteHandler,
};
use crate::runtime::lifecycle::fence::current_managed_runtime_matches;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

/// Kernel-only resolver for descriptor-declared, capability-approved provider secrets.
pub(crate) struct ProviderCredentialHandlerV1 {
    store: Arc<SqliteControlStore>,
    relay: ManagedRuntimeRelayPort,
    vault_route: Arc<KernelManagedVaultRouteHandler>,
}

impl ProviderCredentialHandlerV1 {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        relay: ManagedRuntimeRelayPort,
        vault_route: Arc<KernelManagedVaultRouteHandler>,
    ) -> Self {
        Self {
            store,
            relay,
            vault_route,
        }
    }
}

impl ManagedRuntimeProviderCredentialHandler for ProviderCredentialHandlerV1 {
    fn issue_provider_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeProviderCredentialRequestV1,
    ) -> Result<ManagedRuntimeProviderCredentialDeliveryV1, String> {
        if !current_managed_runtime_matches(
            &*self.store,
            expectation.registration_id(),
            expectation.runtime_instance_id(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "managed runtime provider credential request is unavailable".to_owned())?
        {
            return Err("managed runtime provider credential fence is stale".to_owned());
        }
        let secret_class = SecretClassV1::from_code(i64::from(request.secret_class))
            .ok_or_else(|| "managed runtime provider credential request is denied".to_owned())?;
        let action = VaultActionV1::from_code(i64::from(request.action))
            .ok_or_else(|| "managed runtime provider credential request is denied".to_owned())?;
        authorized_purpose(&self.store, expectation, &request)?;
        let vault = status::read_current(&self.store, &self.relay)?;
        let audience = LeaseAudienceV1::new(
            expectation.registration_id().to_owned(),
            expectation.runtime_instance_id().to_owned(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "managed runtime provider credential request is denied".to_owned())?;
        let purpose = VaultPurposeRequestV1::new(
            request.purpose_id.clone(),
            request.configuration_instance_id.clone(),
            vec![secret_class],
            vec![action],
            request.ttl_seconds,
        )
        .map_err(|_| "managed runtime provider credential request is denied".to_owned())?;
        let issue = VaultLeaseIssueRequestV1::new(
            self.store.snapshot().instance_id().to_owned(),
            vault.runtime_generation(),
            request.credential_revision,
            current_owner(&self.store, expectation)?,
            purpose,
            audience.clone(),
        )
        .map_err(|_| "managed runtime provider credential request is denied".to_owned())?;
        let command = VaultTransportCommandV1::IssueLease { request: issue };
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| "managed runtime provider credential request is denied".to_owned())?;
        let recipient_public_key: [u8; 32] = request
            .recipient_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| "managed runtime provider credential request is denied".to_owned())?;
        let binding = VaultTransportBindingV1::new(
            vault.runtime_generation(),
            audience,
            request_id,
            command.operation_digest(),
            VaultTransportDirectionV1::ToVault,
            recipient_public_key,
        )
        .map_err(|_| "managed runtime provider credential request is denied".to_owned())?;
        let vault_key = VaultTransportPublicKey::from_bytes(*vault.hpke_public_key_x25519())
            .map_err(|_| "managed runtime provider credential is unavailable".to_owned())?;
        let frame = seal(&vault_key, &binding, &command.encode())
            .map_err(|_| "managed runtime provider credential is unavailable".to_owned())?;
        let response = self.vault_route.route_vault_ciphertext(
            expectation,
            VaultCiphertextRouteV1 {
                major: 1,
                registration_id: expectation.registration_id().to_owned(),
                runtime_instance_id: expectation.runtime_instance_id().to_owned(),
                vault_runtime_generation: vault.runtime_generation(),
                grant_epoch: expectation.grant_epoch(),
                request_id: request_id.to_vec(),
                operation_digest_sha256: command.operation_digest().to_vec(),
                direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
                hpke_encapped_key: frame.encapped_key().to_vec(),
                ciphertext: frame.ciphertext().to_vec(),
                hpke_authentication_tag: frame.tag().to_vec(),
                response_recipient_hpke_public_key_x25519: recipient_public_key.to_vec(),
                kernel_instance_id: String::new(),
                kernel_authorization_signature_raw: Vec::new(),
                caller_runtime_generation: expectation.runtime_generation(),
                storage_role_epoch: 0,
                storage_credential_lease_revision: 0,
                storage_runtime_principal: String::new(),
                storage_owner_id: String::new(),
            },
        )?;
        Ok(ManagedRuntimeProviderCredentialDeliveryV1 {
            encapped_key: response.hpke_encapped_key,
            ciphertext: response.ciphertext,
            tag: response.hpke_authentication_tag,
        })
    }
}

fn authorized_purpose(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
    request: &ManagedRuntimeProviderCredentialRequestV1,
) -> Result<ModuleVaultPurposeRequestV1, String> {
    let snapshot = store
        .module_grant_snapshot(expectation.registration_id())
        .map_err(|_| "managed runtime provider credential registration is unavailable".to_owned())?
        .ok_or_else(|| {
            "managed runtime provider credential registration is unavailable".to_owned()
        })?;
    let registration = snapshot.registration();
    let grants = snapshot.effective_grants().ok_or_else(|| {
        "managed runtime provider credential registration is unavailable".to_owned()
    })?;
    if registration.state() != ModuleRegistrationState::Approved
        || registration.grant_epoch() != expectation.grant_epoch()
    {
        return Err("managed runtime provider credential fence is stale".to_owned());
    }
    for capability_id in grants.capability_ids() {
        let purposes = store
            .module_vault_purpose_requests(registration.registration_id(), capability_id)
            .map_err(|_| {
                "managed runtime provider credential authorization is unavailable".to_owned()
            })?;
        if let Some(declared) = purposes
            .into_iter()
            .find(|declared| purpose_matches(declared, request))
        {
            return Ok(declared);
        }
    }
    Err("managed runtime provider credential request is denied".to_owned())
}

fn current_owner(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
) -> Result<String, String> {
    store
        .module_registration(expectation.registration_id())
        .map_err(|_| "managed runtime provider credential registration is unavailable".to_owned())?
        .filter(|registration| {
            registration.state() == ModuleRegistrationState::Approved
                && registration.grant_epoch() == expectation.grant_epoch()
        })
        .map(|registration| registration.owner_id().to_owned())
        .ok_or_else(|| "managed runtime provider credential fence is stale".to_owned())
}

fn purpose_matches(
    declared: &ModuleVaultPurposeRequestV1,
    request: &ManagedRuntimeProviderCredentialRequestV1,
) -> bool {
    declared.purpose_id() == request.purpose_id
        && declared.secret_class() == request.secret_class as u8
        && declared.action() == request.action as u8
        && declared.target_scope() == 1
        && request.ttl_seconds <= u32::from(declared.requested_lease_ttl_seconds())
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::ManagedRuntimeProviderCredentialRequestV1;
    use makosh_vault_protocol::VaultActionV1;

    use super::{ModuleVaultPurposeRequestV1, purpose_matches};

    fn request() -> ManagedRuntimeProviderCredentialRequestV1 {
        ManagedRuntimeProviderCredentialRequestV1 {
            request_id: vec![1; 16],
            purpose_id: "mail.imap.password".to_owned(),
            credential_revision: 1,
            ttl_seconds: 60,
            secret_class: 1,
            recipient_public_key_x25519: vec![2; 32],
            configuration_instance_id: "connection_1".to_owned(),
            action: VaultActionV1::Resolve.code() as u32,
        }
    }

    #[test]
    fn accepts_only_the_exact_resolve_purpose() {
        let declared = ModuleVaultPurposeRequestV1::new(
            "registration",
            "credentials",
            "mail.imap.password",
            120,
            1,
            VaultActionV1::Resolve.code() as u8,
            1,
        );
        assert!(purpose_matches(&declared, &request()));
        let mut wrong = request();
        wrong.secret_class = 2;
        assert!(!purpose_matches(&declared, &wrong));
        let mut wrong = request();
        wrong.action = VaultActionV1::ReplaceCas.code() as u32;
        assert!(!purpose_matches(&declared, &wrong));
    }
}
