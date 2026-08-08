//! Authorizes one exact owner-derived projection-key lease without exposing key material.

use std::sync::Arc;

use makosh_kernel_control_store::{ModuleRegistrationState, ModuleVaultPurposeRequestV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    ManagedRuntimeOwnerDerivedKeyDeliveryV1, ManagedRuntimeOwnerDerivedKeyRequestV1,
    VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1, VaultPurposeRequestV1,
    VaultTransportBindingV1, VaultTransportCommandV1, VaultTransportDirectionV1,
    VaultTransportPublicKey, seal,
};

use crate::platform::vault::{managed_route::KernelManagedVaultRouteHandler, status};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeExpectation, ManagedRuntimeOwnerDerivedKeyHandler,
    ManagedRuntimeVaultRouteHandler,
};
use crate::runtime::lifecycle::fence::current_managed_runtime_matches;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

pub(crate) struct OwnerDerivedKeyHandlerV1 {
    store: Arc<SqliteControlStore>,
    relay: ManagedRuntimeRelayPort,
    vault_route: Arc<KernelManagedVaultRouteHandler>,
}

impl OwnerDerivedKeyHandlerV1 {
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

impl ManagedRuntimeOwnerDerivedKeyHandler for OwnerDerivedKeyHandlerV1 {
    fn issue_owner_derived_key(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeOwnerDerivedKeyRequestV1,
    ) -> Result<ManagedRuntimeOwnerDerivedKeyDeliveryV1, String> {
        if !current_managed_runtime_matches(
            &*self.store,
            expectation.registration_id(),
            expectation.runtime_instance_id(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "managed runtime owner-derived key request is unavailable".to_owned())?
        {
            return Err("managed runtime owner-derived key fence is stale".to_owned());
        }
        let (owner_id, capability_id) = authorized_purpose(&self.store, expectation, &request)?;
        let vault = status::read_current(&self.store, &self.relay)?;
        let audience = LeaseAudienceV1::new(
            expectation.registration_id().to_owned(),
            expectation.runtime_instance_id().to_owned(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "managed runtime owner-derived key request is denied".to_owned())?;
        // This is a Vault-internal deterministic scope derived from the approved
        // capability, not a module-supplied provider configuration instance.
        let purpose = VaultPurposeRequestV1::new(
            request.purpose_id.clone(),
            capability_id,
            vec![SecretClassV1::OwnerDerivedKey],
            vec![VaultActionV1::IssueOwnerDerivedKey],
            request.ttl_seconds,
        )
        .map_err(|_| "managed runtime owner-derived key request is denied".to_owned())?;
        let issue = VaultLeaseIssueRequestV1::new(
            self.store.snapshot().instance_id().to_owned(),
            vault.runtime_generation(),
            u64::from(request.key_schema_revision),
            owner_id,
            purpose,
            audience.clone(),
        )
        .map_err(|_| "managed runtime owner-derived key request is denied".to_owned())?;
        let command = VaultTransportCommandV1::IssueLease { request: issue };
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| "managed runtime owner-derived key request is denied".to_owned())?;
        let recipient_public_key: [u8; 32] = request
            .recipient_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| "managed runtime owner-derived key request is denied".to_owned())?;
        let binding = VaultTransportBindingV1::new(
            vault.runtime_generation(),
            audience,
            request_id,
            command.operation_digest(),
            VaultTransportDirectionV1::ToVault,
            recipient_public_key,
        )
        .map_err(|_| "managed runtime owner-derived key request is denied".to_owned())?;
        let vault_key = VaultTransportPublicKey::from_bytes(*vault.hpke_public_key_x25519())
            .map_err(|_| "managed runtime owner-derived key is unavailable".to_owned())?;
        let frame = seal(&vault_key, &binding, &command.encode())
            .map_err(|_| "managed runtime owner-derived key is unavailable".to_owned())?;
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
        Ok(ManagedRuntimeOwnerDerivedKeyDeliveryV1 {
            encapped_key: response.hpke_encapped_key,
            ciphertext: response.ciphertext,
            tag: response.hpke_authentication_tag,
        })
    }
}

fn authorized_purpose(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
    request: &ManagedRuntimeOwnerDerivedKeyRequestV1,
) -> Result<(String, String), String> {
    let snapshot = store
        .module_grant_snapshot(expectation.registration_id())
        .map_err(|_| "managed runtime owner-derived key registration is unavailable".to_owned())?
        .ok_or_else(|| {
            "managed runtime owner-derived key registration is unavailable".to_owned()
        })?;
    let registration = snapshot.registration();
    let grants = snapshot.effective_grants().ok_or_else(|| {
        "managed runtime owner-derived key registration is unavailable".to_owned()
    })?;
    if registration.state() != ModuleRegistrationState::Approved
        || registration.grant_epoch() != expectation.grant_epoch()
    {
        return Err("managed runtime owner-derived key fence is stale".to_owned());
    }
    if !grants
        .capability_ids()
        .iter()
        .any(|capability_id| capability_id == &request.capability_id)
    {
        return Err("managed runtime owner-derived key request is denied".to_owned());
    }
    let purposes = store
        .module_vault_purpose_requests(registration.registration_id(), &request.capability_id)
        .map_err(|_| "managed runtime owner-derived key authorization is unavailable".to_owned())?;
    purposes
        .into_iter()
        .any(|declared| purpose_matches(&declared, request))
        .then(|| {
            (
                registration.owner_id().to_owned(),
                request.capability_id.clone(),
            )
        })
        .ok_or_else(|| "managed runtime owner-derived key request is denied".to_owned())
}

fn purpose_matches(
    declared: &ModuleVaultPurposeRequestV1,
    request: &ManagedRuntimeOwnerDerivedKeyRequestV1,
) -> bool {
    declared.purpose_id() == request.purpose_id
        && declared.secret_class() == SecretClassV1::OwnerDerivedKey.code() as u8
        && declared.action() == VaultActionV1::IssueOwnerDerivedKey.code() as u8
        && declared.target_scope() == 2
        && declared.key_schema_revision() == request.key_schema_revision
        && request.ttl_seconds <= u32::from(declared.requested_lease_ttl_seconds())
}

#[cfg(test)]
mod tests {
    use makosh_kernel_control_store::ModuleVaultPurposeRequestV1;
    use makosh_runtime_protocol::v1::ManagedRuntimeOwnerDerivedKeyRequestV1;

    use super::purpose_matches;

    fn request() -> ManagedRuntimeOwnerDerivedKeyRequestV1 {
        ManagedRuntimeOwnerDerivedKeyRequestV1 {
            request_id: vec![1; 16],
            purpose_id: "communications.search.index".to_owned(),
            key_schema_revision: 1,
            ttl_seconds: 60,
            recipient_public_key_x25519: vec![2; 32],
            capability_id: "search".to_owned(),
        }
    }

    #[test]
    fn accepts_only_the_exact_owner_derived_key_purpose() {
        let declared = ModuleVaultPurposeRequestV1::new_with_key_schema_revision(
            "registration",
            "search",
            "communications.search.index",
            120,
            makosh_kernel_control_store::ModuleVaultPurposePolicyV1 {
                secret_class: 6,
                action: 7,
                target_scope: 2,
                key_schema_revision: 1,
            },
        );
        assert!(purpose_matches(&declared, &request()));
        let mut wrong = request();
        wrong.key_schema_revision = 2;
        assert!(!purpose_matches(&declared, &wrong));
    }
}
