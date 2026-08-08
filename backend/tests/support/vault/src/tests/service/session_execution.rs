use std::os::unix::fs::PermissionsExt;

use makosh_runtime_protocol::v1::{VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1, VaultPurposeRequestV1,
    VaultTransportBindingV1, VaultTransportCommandV1, VaultTransportDirectionV1, seal,
};
use makosh_vault_store_sqlcipher::{SecretRecordScope, VaultStore};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use prost::Message;
use tempfile::TempDir;

use crate::service::runtime::VaultService;
use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::route::execute_route;
use crate::transport::session::VaultTransportReplayGuard;

mod session_requests;

use session_requests::{audience, lease_request, lease_request_at};

#[test]
fn authenticated_session_executes_a_lease_scoped_resolve_without_record_id() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = resolve_purpose();
    let scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("secret scope");
    store
        .store_secret(&scope, b"session-command-marker")
        .expect("store secret");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let mut service = VaultService::new(store, 3).expect("Vault service");
    let keys = VaultTransportKeyPair::generate();
    let response_recipient = VaultTransportKeyPair::generate();
    assert_inherited_route(&mut service, &keys, &response_recipient, audience);
}

#[test]
fn authenticated_session_stores_then_resolves_a_lease_scoped_secret() {
    let temporary = private_temporary_directory();
    let store = initialize_store(&temporary);
    let audience = audience();
    let mut service = VaultService::new(store, 3).expect("Vault service");
    let keys = VaultTransportKeyPair::generate();
    let response_recipient = VaultTransportKeyPair::generate();
    let mut guard = VaultTransportReplayGuard::new(3);

    let store_command = VaultTransportCommandV1::StoreLease {
        lease_id: service
            .issue_lease(lease_request(store_purpose(), audience.clone()), 200)
            .expect("store lease")
            .lease_id()
            .clone(),
        secret_class: SecretClassV1::ProviderCredential,
        payload: b"new-session-command-marker".to_vec(),
    };
    let stored = execute_command(
        &mut service,
        &keys,
        &response_recipient,
        &mut guard,
        &audience,
        [9; 16],
        &store_command,
    );
    assert_eq!(
        stored.len(),
        16,
        "store response contains only record identity"
    );

    let resolve_command = VaultTransportCommandV1::ResolveLease {
        lease_id: service
            .issue_lease(lease_request(resolve_purpose(), audience.clone()), 202)
            .expect("resolve lease")
            .lease_id()
            .clone(),
        secret_class: SecretClassV1::ProviderCredential,
    };
    let resolved = execute_command(
        &mut service,
        &keys,
        &response_recipient,
        &mut guard,
        &audience,
        [10; 16],
        &resolve_command,
    );
    assert_eq!(resolved.as_slice(), b"new-session-command-marker");
}

#[test]
fn storage_delegated_route_rejects_secret_mutation_commands() {
    let temporary = private_temporary_directory();
    let store = initialize_store(&temporary);
    let audience = audience();
    let mut service = VaultService::new(store, 3).expect("Vault service");
    let keys = VaultTransportKeyPair::generate();
    let response_recipient = VaultTransportKeyPair::generate();
    let command = store_command(&mut service, &audience);
    let binding = VaultTransportBindingV1::new(
        3,
        audience.clone(),
        [14; 16],
        command.operation_digest(),
        VaultTransportDirectionV1::ToVault,
        *response_recipient.public_key().as_bytes(),
    )
    .expect("route binding");
    let frame = seal(keys.public_key(), &binding, &command.encode()).expect("seal route command");
    let signing_key = SigningKey::from_bytes((&[22_u8; 32]).into()).expect("test signing key");
    let authorization_key = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("SEC1 key");
    let mut route = route_from_binding(
        &audience,
        &binding,
        &frame,
        &response_recipient,
        &signing_key,
    );
    route.storage_role_epoch = 1;
    route.storage_credential_lease_revision = 1;
    route.storage_runtime_principal = "storage-runtime-principal".to_owned();
    route.storage_owner_id = "owner-1".to_owned();
    sign_route(&mut route, &signing_key);

    assert!(
        execute_route(
            &mut service,
            &keys,
            &mut VaultTransportReplayGuard::new(3),
            authorization_key,
            route,
            201,
        )
        .is_err()
    );
}

#[test]
fn authenticated_session_replaces_a_credential_at_the_next_revision() {
    let temporary = private_temporary_directory();
    let store = initialize_store(&temporary);
    let audience = audience();
    let mut service = VaultService::new(store, 3).expect("Vault service");
    let keys = VaultTransportKeyPair::generate();
    let response_recipient = VaultTransportKeyPair::generate();
    let mut guard = VaultTransportReplayGuard::new(3);
    let initial_command = store_command(&mut service, &audience);
    let initial = execute_command(
        &mut service,
        &keys,
        &response_recipient,
        &mut guard,
        &audience,
        [11; 16],
        &initial_command,
    );
    let prior_record_id = initial
        .as_slice()
        .try_into()
        .expect("record identity has fixed length");
    let replacement = VaultTransportCommandV1::ReplaceLease {
        lease_id: service
            .issue_lease(
                lease_request_at(replace_purpose(), audience.clone(), 2),
                202,
            )
            .expect("replace lease")
            .lease_id()
            .clone(),
        secret_class: SecretClassV1::ProviderCredential,
        prior_record_id,
        payload: b"replacement-session-command-marker".to_vec(),
    };
    let replacement_id = execute_command(
        &mut service,
        &keys,
        &response_recipient,
        &mut guard,
        &audience,
        [12; 16],
        &replacement,
    );
    assert_eq!(replacement_id.len(), 16);
    let resolve_command = resolve_command(&mut service, &audience, 2);
    let resolved = execute_command(
        &mut service,
        &keys,
        &response_recipient,
        &mut guard,
        &audience,
        [13; 16],
        &resolve_command,
    );
    assert_eq!(resolved.as_slice(), b"replacement-session-command-marker");
}

fn assert_inherited_route(
    service: &mut VaultService,
    keys: &VaultTransportKeyPair,
    response_recipient: &VaultTransportKeyPair,
    audience: LeaseAudienceV1,
) {
    let command = VaultTransportCommandV1::ResolveLease {
        lease_id: service
            .issue_lease(lease_request(resolve_purpose(), audience.clone()), 200)
            .expect("second resolve lease")
            .lease_id()
            .clone(),
        secret_class: SecretClassV1::ProviderCredential,
    };
    let binding = VaultTransportBindingV1::new(
        3,
        audience.clone(),
        [8; 16],
        command.operation_digest(),
        VaultTransportDirectionV1::ToVault,
        *response_recipient.public_key().as_bytes(),
    )
    .expect("route binding");
    let frame = seal(keys.public_key(), &binding, &command.encode()).expect("seal route command");
    let signing_key = SigningKey::from_bytes((&[21_u8; 32]).into()).expect("test signing key");
    let authorization_key = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("SEC1 key");
    let route = route_from_binding(
        &audience,
        &binding,
        &frame,
        response_recipient,
        &signing_key,
    );
    let mut tampered = route.clone();
    tampered.kernel_instance_id = "other-instance".to_owned();
    assert!(
        crate::transport::route::verify_kernel_authorization(&tampered, authorization_key).is_err()
    );
    assert!(crate::transport::route::verify_kernel_authorization(&route, [4; 65]).is_err());
    let response = execute_route(
        service,
        keys,
        &mut VaultTransportReplayGuard::new(3),
        authorization_key,
        route,
        201,
    )
    .expect("execute inherited route");
    assert_response(response_recipient, audience, &command, response);
}

fn execute_command(
    service: &mut VaultService,
    keys: &VaultTransportKeyPair,
    response_recipient: &VaultTransportKeyPair,
    guard: &mut VaultTransportReplayGuard,
    audience: &LeaseAudienceV1,
    request_id: [u8; 16],
    command: &VaultTransportCommandV1,
) -> Vec<u8> {
    let binding = VaultTransportBindingV1::new(
        3,
        audience.clone(),
        request_id,
        command.operation_digest(),
        VaultTransportDirectionV1::ToVault,
        *response_recipient.public_key().as_bytes(),
    )
    .expect("route binding");
    let frame = seal(keys.public_key(), &binding, &command.encode()).expect("seal route command");
    let session = makosh_vault_protocol::VaultTransportSessionV1::new(binding, frame);
    let response = crate::transport::session::execute_session(guard, keys, service, &session, 201)
        .expect("execute session command");
    let response_binding = VaultTransportBindingV1::new(
        3,
        audience.clone(),
        request_id,
        command.operation_digest(),
        VaultTransportDirectionV1::FromVault,
        *response_recipient.public_key().as_bytes(),
    )
    .expect("response binding");
    let response =
        crate::transport::response::encrypt_result(session.binding(), response.as_slice())
            .expect("encrypt response");
    response_recipient
        .open(&response_binding, &response_frame(response))
        .expect("decrypt response")
        .to_vec()
}

fn store_command(
    service: &mut VaultService,
    audience: &LeaseAudienceV1,
) -> VaultTransportCommandV1 {
    VaultTransportCommandV1::StoreLease {
        lease_id: service
            .issue_lease(lease_request(store_purpose(), audience.clone()), 200)
            .expect("store lease")
            .lease_id()
            .clone(),
        secret_class: SecretClassV1::ProviderCredential,
        payload: b"new-session-command-marker".to_vec(),
    }
}

fn resolve_command(
    service: &mut VaultService,
    audience: &LeaseAudienceV1,
    revision: u64,
) -> VaultTransportCommandV1 {
    VaultTransportCommandV1::ResolveLease {
        lease_id: service
            .issue_lease(
                lease_request_at(resolve_purpose(), audience.clone(), revision),
                202,
            )
            .expect("resolve lease")
            .lease_id()
            .clone(),
        secret_class: SecretClassV1::ProviderCredential,
    }
}

fn assert_response(
    response_recipient: &VaultTransportKeyPair,
    audience: LeaseAudienceV1,
    command: &VaultTransportCommandV1,
    response: makosh_runtime_protocol::v1::VaultCiphertextResponseV1,
) {
    let response_binding = VaultTransportBindingV1::new(
        3,
        audience,
        [8; 16],
        command.operation_digest(),
        VaultTransportDirectionV1::FromVault,
        *response_recipient.public_key().as_bytes(),
    )
    .expect("response binding");
    let response_frame = response_frame(response);
    assert_eq!(
        response_recipient
            .open(&response_binding, &response_frame)
            .expect("decrypt inherited response")
            .as_slice(),
        b"session-command-marker"
    );
}

fn route_from_binding(
    audience: &LeaseAudienceV1,
    binding: &VaultTransportBindingV1,
    frame: &makosh_vault_protocol::VaultCiphertextFrameV1,
    response_recipient: &VaultTransportKeyPair,
    signing_key: &SigningKey,
) -> VaultCiphertextRouteV1 {
    let mut route = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: audience.module_registration_id().to_owned(),
        runtime_instance_id: audience.runtime_instance_id().to_owned(),
        caller_runtime_generation: audience.runtime_generation(),
        vault_runtime_generation: 3,
        grant_epoch: audience.grant_epoch(),
        request_id: binding.request_id().to_vec(),
        operation_digest_sha256: binding.operation_digest().to_vec(),
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        response_recipient_hpke_public_key_x25519: response_recipient
            .public_key()
            .as_bytes()
            .to_vec(),
        kernel_instance_id: "instance-1".to_owned(),
        kernel_authorization_signature_raw: Vec::new(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    };
    sign_route(&mut route, signing_key);
    route
}

fn sign_route(route: &mut VaultCiphertextRouteV1, signing_key: &SigningKey) {
    route.kernel_authorization_signature_raw.clear();
    let mut message = b"makosh.vault-route-authorization.v1\0".to_vec();
    message.extend_from_slice(&route.encode_to_vec());
    let signature: Signature = signing_key.sign(&message);
    route.kernel_authorization_signature_raw = signature.to_bytes().to_vec();
}

fn response_frame(
    response: makosh_runtime_protocol::v1::VaultCiphertextResponseV1,
) -> makosh_vault_protocol::VaultCiphertextFrameV1 {
    makosh_vault_protocol::VaultCiphertextFrameV1::from_parts(
        response.hpke_encapped_key,
        response.ciphertext,
        response.hpke_authentication_tag,
    )
    .expect("response frame")
}

fn initialize_store(temporary: &TempDir) -> VaultStore {
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let key = provider.load_or_create().expect("file wrapping key");
    VaultStore::initialize(
        &temporary.path().join("vault.db"),
        &temporary.path().join("vault.anchor"),
        "vault-instance",
        &key,
    )
    .expect("Vault store")
}

fn private_temporary_directory() -> TempDir {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    temporary
}

fn resolve_purpose() -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Resolve],
        60,
    )
    .expect("typed purpose")
}

fn store_purpose() -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Create],
        60,
    )
    .expect("typed purpose")
}

fn replace_purpose() -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::ReplaceCas],
        60,
    )
    .expect("typed purpose")
}
