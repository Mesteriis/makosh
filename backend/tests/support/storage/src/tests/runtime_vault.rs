use std::future::Future;
use std::sync::{Arc, Mutex};

use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_storage_control::{StorageFenceOutcomeV1, StorageVaultLeasePortV1};
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultCiphertextFrameV1, VaultResponseRecipientV1,
    VaultTransportBindingV1, VaultTransportCommandV1, VaultTransportDirectionV1,
    VaultTransportPublicKey, seal,
};

use crate::storage_runtime_vault::{
    StoragePlatformCredentialBootstrapV1, StoragePlatformCredentialPurposeV1,
    StoragePlatformCredentialStateV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
    StorageVaultRouteFailureV1, StorageVaultRoutePortV1, complete_immediately,
};

use super::fixtures::storage_role_binding;

#[path = "runtime_vault/unavailable_route.rs"]
mod unavailable_route;
use unavailable_route::UnavailableVaultRoute;

#[test]
fn storage_vault_adapter_revokes_only_its_fenced_runtime_audience() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let facts = Arc::new(Mutex::new(RouteFacts::default()));
    let route = ConfirmingVaultRoute::new(vault, Arc::clone(&facts));
    let mut adapter = StorageVaultLeaseAdapterV1::new(route, context);
    let binding = storage_role_binding("notes", "runtime_notes_1");

    let outcome = run(adapter.invalidate_lease(&binding));

    assert_eq!(outcome, StorageFenceOutcomeV1::Applied);
    let facts = facts.lock().expect("route facts");
    assert_eq!(facts.audience.as_deref(), Some("registration_notes"));
    assert_eq!(facts.grant_epoch, Some(1));
}

#[test]
fn storage_vault_adapter_fails_closed_when_vault_rejects_the_route() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let mut adapter = StorageVaultLeaseAdapterV1::new(RejectingVaultRoute, context);

    let outcome = run(adapter.invalidate_lease(&storage_role_binding("notes", "runtime_notes_1")));

    assert_eq!(outcome, StorageFenceOutcomeV1::Rejected);
}

#[test]
fn storage_vault_adapter_reports_unavailable_when_route_is_down() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let mut adapter = StorageVaultLeaseAdapterV1::new(UnavailableVaultRoute, context);

    let outcome = run(adapter.invalidate_lease(&storage_role_binding("notes", "runtime_notes_1")));

    assert_eq!(outcome, StorageFenceOutcomeV1::Unavailable);
}

#[test]
fn storage_vault_adapter_issues_and_resolves_only_its_platform_credential() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let mut adapter = StorageVaultLeaseAdapterV1::new(CredentialVaultRoute { vault }, context);
    let binding = storage_role_binding("notes", "runtime_notes_1");

    let lease_id = run(adapter.issue_runtime_credential(&binding)).expect("issue lease");
    let credential =
        run(adapter.resolve_runtime_credential(&binding, lease_id)).expect("resolve credential");

    assert_eq!(credential.as_slice(), b"leased-platform-credential");
}

#[test]
fn storage_vault_adapter_creates_only_a_fenced_platform_credential() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let mut adapter = StorageVaultLeaseAdapterV1::new(CredentialVaultRoute { vault }, context);
    let binding = storage_role_binding("notes", "runtime_notes_1");

    let lease_id = run(adapter.issue_platform_credential_create(&binding))
        .expect("issue platform credential create lease");
    run(adapter.store_platform_credential(&binding, lease_id, b"platform-credential"))
        .expect("store platform credential");
}
#[test]
fn storage_runtime_credential_is_generated_under_its_exact_runtime_principal() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let mut adapter = StorageVaultLeaseAdapterV1::new(
        MissingThenGeneratedVaultRoute {
            vault,
            step: 0,
            expected_purpose: "storage.runtime.credential",
            expected_configuration_instance: "runtime_notes_1",
        },
        context,
    );
    let binding = storage_role_binding("notes", "runtime_notes_1");

    assert_eq!(
        run(adapter.ensure_runtime_credential(&binding))
            .expect("generate and resolve fenced credential")
            .as_slice(),
        b"leased-platform-credential"
    );
}

#[test]
fn platform_bootstrap_generates_a_missing_credential_without_accepting_plaintext() {
    let vault = VaultResponseRecipientV1::generate();
    let audience = storage_audience();
    let context = route_context(&vault);
    let mut bootstrap = StoragePlatformCredentialBootstrapV1::new(
        MissingThenGeneratedVaultRoute {
            vault,
            step: 0,
            expected_purpose: "storage.control.pgbouncer.admin",
            expected_configuration_instance: "storage-main",
        },
        context,
        audience,
        StoragePlatformCredentialPurposeV1::PgBouncerAdmin,
        "storage-main".to_owned(),
        3,
    )
    .expect("valid platform bootstrap");

    assert_eq!(
        run(bootstrap.ensure()),
        Ok(StoragePlatformCredentialStateV1::Generated)
    );
}

#[test]
fn platform_bootstrap_scopes_postgres_admin_credential_to_its_own_purpose() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let mut bootstrap = StoragePlatformCredentialBootstrapV1::new(
        MissingThenGeneratedVaultRoute {
            vault,
            step: 0,
            expected_purpose: "storage.control.postgres.admin",
            expected_configuration_instance: "storage-main",
        },
        context,
        storage_audience(),
        StoragePlatformCredentialPurposeV1::PostgresAdmin,
        "storage-main".to_owned(),
        3,
    )
    .expect("valid platform bootstrap");

    assert_eq!(
        run(bootstrap.ensure()),
        Ok(StoragePlatformCredentialStateV1::Generated)
    );
}

#[test]
fn platform_bootstrap_returns_an_existing_credential_only_after_a_fresh_resolve_lease() {
    let vault = VaultResponseRecipientV1::generate();
    let context = route_context(&vault);
    let mut bootstrap = StoragePlatformCredentialBootstrapV1::new(
        CredentialVaultRoute { vault },
        context,
        storage_audience(),
        StoragePlatformCredentialPurposeV1::PgBouncerAdmin,
        "storage-main".to_owned(),
        3,
    )
    .expect("valid platform bootstrap");

    assert_eq!(
        run(bootstrap.ensure_and_resolve())
            .expect("resolved platform credential")
            .as_slice(),
        b"leased-platform-credential"
    );
}

#[test]
fn platform_bootstrap_refuses_a_route_that_cannot_complete_on_the_inherited_channel() {
    assert!(matches!(
        complete_immediately(std::future::pending::<()>()),
        Err(crate::storage_runtime_vault::StoragePlatformCredentialErrorV1::Unavailable)
    ));
}

fn route_context(vault: &VaultResponseRecipientV1) -> StorageVaultRouteContextV1 {
    StorageVaultRouteContextV1::new("vault-instance".into(), 7, *vault.public_key().as_bytes())
        .expect("valid Vault transport context")
}

fn storage_audience() -> LeaseAudienceV1 {
    LeaseAudienceV1::new(
        "storage-control".to_owned(),
        "runtime-storage-1".to_owned(),
        4,
        9,
    )
    .expect("Storage runtime audience")
}

fn run<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("test runtime")
        .block_on(future)
}

struct ConfirmingVaultRoute {
    vault: VaultResponseRecipientV1,
    facts: Arc<Mutex<RouteFacts>>,
}

impl ConfirmingVaultRoute {
    fn new(vault: VaultResponseRecipientV1, facts: Arc<Mutex<RouteFacts>>) -> Self {
        Self { vault, facts }
    }
}

impl StorageVaultRoutePortV1 for ConfirmingVaultRoute {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>> + Send
    {
        let response = confirm_revoke(&self.vault, &route);
        if let Ok(mut facts) = self.facts.lock() {
            facts.audience = Some(route.registration_id);
            facts.grant_epoch = Some(route.grant_epoch);
        }
        async move { response }
    }
}

#[derive(Default)]
struct RouteFacts {
    audience: Option<String>,
    grant_epoch: Option<u64>,
}

struct RejectingVaultRoute;

struct CredentialVaultRoute {
    vault: VaultResponseRecipientV1,
}

struct MissingThenGeneratedVaultRoute {
    vault: VaultResponseRecipientV1,
    step: u8,
    expected_purpose: &'static str,
    expected_configuration_instance: &'static str,
}

impl StorageVaultRoutePortV1 for MissingThenGeneratedVaultRoute {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>> + Send
    {
        let response = self.respond(&route);
        async move { response }
    }
}

impl MissingThenGeneratedVaultRoute {
    fn respond(
        &mut self,
        route: &VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
        let (audience, command) = command_from_route(&self.vault, route)?;
        let response = match (self.step, command) {
            (0, VaultTransportCommandV1::IssueLease { request })
                if request.purpose().actions()
                    == [makosh_vault_protocol::VaultActionV1::Resolve]
                    && request.purpose().purpose_id() == self.expected_purpose
                    && request.purpose().configuration_instance_id()
                        == self.expected_configuration_instance =>
            {
                response_for(route, audience, b"0123456789abcdef0123456789abcdef")
            }
            (1, VaultTransportCommandV1::ResolveLease { .. }) => {
                Err(StorageVaultRouteFailureV1::Rejected)
            }
            (2, VaultTransportCommandV1::IssueLease { request })
                if request.purpose().actions()
                    == [makosh_vault_protocol::VaultActionV1::Create]
                    && request.purpose().purpose_id() == self.expected_purpose
                    && request.purpose().configuration_instance_id()
                        == self.expected_configuration_instance =>
            {
                response_for(route, audience, b"fedcba9876543210fedcba9876543210")
            }
            (3, VaultTransportCommandV1::GenerateOpaqueToken { .. }) => {
                response_for(route, audience, &[7; 16])
            }
            (4, VaultTransportCommandV1::IssueLease { request })
                if request.purpose().actions()
                    == [makosh_vault_protocol::VaultActionV1::Resolve]
                    && request.purpose().purpose_id() == self.expected_purpose
                    && request.purpose().configuration_instance_id()
                        == self.expected_configuration_instance =>
            {
                response_for(route, audience, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            }
            (5, VaultTransportCommandV1::ResolveLease { .. }) => {
                response_for(route, audience, b"leased-platform-credential")
            }
            _ => Err(StorageVaultRouteFailureV1::Rejected),
        };
        if response.is_ok() || self.step == 1 {
            self.step = self.step.saturating_add(1);
        }
        response
    }
}

impl StorageVaultRoutePortV1 for CredentialVaultRoute {
    #[allow(clippy::manual_async_fn)] // The route port requires a Send future.
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>> + Send
    {
        let response = confirm_credential(&self.vault, &route);
        async move { response }
    }
}

impl StorageVaultRoutePortV1 for RejectingVaultRoute {
    #[allow(clippy::manual_async_fn)] // The route port requires a Send future.
    fn route_vault_ciphertext(
        &mut self,
        _: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>> + Send
    {
        async { Err(StorageVaultRouteFailureV1::Rejected) }
    }
}

fn confirm_revoke(
    vault: &VaultResponseRecipientV1,
    route: &VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
    let (audience, command) = command_from_route(vault, route)?;
    if command != VaultTransportCommandV1::RevokeAudience {
        return Err(StorageVaultRouteFailureV1::Rejected);
    }
    response_for(route, audience, &[1])
}

fn confirm_credential(
    vault: &VaultResponseRecipientV1,
    route: &VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
    let (audience, command) = command_from_route(vault, route)?;
    match command {
        VaultTransportCommandV1::IssueLease { request }
            if request.vault_instance_id() == "vault-instance"
                && request.audience() == &audience
                && request.purpose().allowed_secret_classes()
                    == [SecretClassV1::PlatformCredential]
                && request.purpose().actions()
                    == [makosh_vault_protocol::VaultActionV1::Resolve] =>
        {
            response_for(route, audience, b"0123456789abcdef0123456789abcdef")
        }
        VaultTransportCommandV1::IssueLease { request }
            if request.vault_instance_id() == "vault-instance"
                && request.audience() == &audience
                && request.purpose().allowed_secret_classes()
                    == [SecretClassV1::PlatformCredential]
                && request.purpose().actions()
                    == [makosh_vault_protocol::VaultActionV1::Create] =>
        {
            response_for(route, audience, b"fedcba9876543210fedcba9876543210")
        }
        VaultTransportCommandV1::StoreLease {
            lease_id,
            secret_class: SecretClassV1::PlatformCredential,
            payload,
        } if lease_id.as_str() == "fedcba9876543210fedcba9876543210"
            && payload == b"platform-credential" =>
        {
            response_for(route, audience, &[7; 16])
        }
        VaultTransportCommandV1::ResolveLease {
            lease_id,
            secret_class: SecretClassV1::PlatformCredential,
        } if lease_id.as_str() == "0123456789abcdef0123456789abcdef" => {
            response_for(route, audience, b"leased-platform-credential")
        }
        _ => Err(StorageVaultRouteFailureV1::Rejected),
    }
}

fn command_from_route(
    vault: &VaultResponseRecipientV1,
    route: &VaultCiphertextRouteV1,
) -> Result<(LeaseAudienceV1, VaultTransportCommandV1), StorageVaultRouteFailureV1> {
    let audience = route_audience(route)?;
    let request_binding =
        transport_binding(route, audience.clone(), VaultTransportDirectionV1::ToVault)?;
    let request_frame = VaultCiphertextFrameV1::from_parts(
        route.hpke_encapped_key.clone(),
        route.ciphertext.clone(),
        route.hpke_authentication_tag.clone(),
    )
    .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    let plaintext = vault
        .open(&request_binding, &request_frame)
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    let command = VaultTransportCommandV1::decode(plaintext.as_slice())
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    Ok((audience, command))
}

fn route_audience(
    route: &VaultCiphertextRouteV1,
) -> Result<LeaseAudienceV1, StorageVaultRouteFailureV1> {
    LeaseAudienceV1::new(
        route.registration_id.clone(),
        route.runtime_instance_id.clone(),
        route.caller_runtime_generation,
        route.grant_epoch,
    )
    .map_err(|_| StorageVaultRouteFailureV1::Rejected)
}

fn response_for(
    route: &VaultCiphertextRouteV1,
    audience: LeaseAudienceV1,
    plaintext: &[u8],
) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
    let binding = transport_binding(route, audience, VaultTransportDirectionV1::FromVault)?;
    let key = route
        .response_recipient_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    let key = VaultTransportPublicKey::from_bytes(key)
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    let frame =
        seal(&key, &binding, plaintext).map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    Ok(VaultCiphertextResponseV1 {
        major: 1,
        vault_runtime_generation: route.vault_runtime_generation,
        request_id: route.request_id.clone(),
        operation_digest_sha256: route.operation_digest_sha256.clone(),
        direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        caller_runtime_generation: route.caller_runtime_generation,
    })
}

fn transport_binding(
    route: &VaultCiphertextRouteV1,
    audience: LeaseAudienceV1,
    direction: VaultTransportDirectionV1,
) -> Result<VaultTransportBindingV1, StorageVaultRouteFailureV1> {
    let request_id = route
        .request_id
        .as_slice()
        .try_into()
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    let digest = route
        .operation_digest_sha256
        .as_slice()
        .try_into()
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    let response_key = route
        .response_recipient_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    VaultTransportBindingV1::new(
        route.vault_runtime_generation,
        audience,
        request_id,
        digest,
        direction,
        response_key,
    )
    .map_err(|_| StorageVaultRouteFailureV1::Rejected)
}
