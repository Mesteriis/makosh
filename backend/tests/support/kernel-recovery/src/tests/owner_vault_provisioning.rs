//! Live owner proof, opaque HPKE provisioning and restart-safe Vault receipt.

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, StatusCode};
use makosh_gateway_protocol::v1::{
    AuthorizeOwnerVaultProvisioningRequestV1, CommitOwnerVaultProvisioningRequestV1,
    OwnerVaultActionV1, OwnerVaultSecretClassV1, PrepareOwnerVaultProvisioningRequestV1,
};
use makosh_gateway_runtime::{
    GatewayApplicationRouter, InMemoryBrowserRealtimeSource, OWNER_VAULT_AUTHORIZE_PATH,
    OWNER_VAULT_COMMIT_PATH, OWNER_VAULT_PREPARE_PATH, OwnerVaultClientPrincipalV1,
    OwnerVaultProvisioningHandlerV1, OwnerVaultProvisioningRouteErrorV1,
};
use makosh_kernel_control_store::{
    ModuleDescriptorRegistrationRequestsV1, ModuleVaultPurposeRequestV1,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1, VaultCiphertextFrameV1,
    VaultProvisioningReceiptV1, VaultResponseRecipientV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportPublicKey, seal,
};

use super::common::*;
use crate::identity::browser_gateway::ControlStoreBrowserAuthority;
use crate::platform::managed::signed_bundle::{InstalledSignedBundle, SignedRuntimeArtifact};
use crate::platform::vault::owner_provisioning::KernelOwnerVaultProvisioningHandlerV1;
use crate::platform::vault::{binding as vault_binding, launch as vault_launch};
use crate::tests::platform_vault::live as vault_fixture;

const HUMAN_OWNER: &str = "owner-1";
const DEVICE: &str = "browser-1";
const REGISTRATION: &str = "mail-registration";
const CAPABILITY: &str = "mail.credentials.setup";
const PURPOSE: &str = "mail.imap.password";

#[test]
#[ignore = "builds and launches the real Vault runtime binary"]
fn owner_vault_provisioning_survives_vault_restart_with_durable_idempotency() {
    let root = unique_target_root("makosh-owner-vault-live");
    let data = vault_fixture::private_directory(root.join("kernel"));
    vault_fixture::initialize_vault(&data);
    let release = InstalledSignedBundle::install(
        &root,
        &[SignedRuntimeArtifact::new(
            "platform.vault",
            vault_fixture::vault_binary(),
            vault_fixture::vault_descriptor(),
        )],
    )
    .expect("install signed Vault release");
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "kernel-main", 1)
            .expect("create Control Store"),
    );
    let browser_signing_key = admit_owner_browser_and_mail(&store);
    let principal = OwnerVaultClientPrincipalV1::new(
        HUMAN_OWNER,
        DEVICE,
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .expect("browser principal");
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));
    vault_fixture::bind_and_start(&supervisor, &store, &data, release.kernel());
    let handler = KernelOwnerVaultProvisioningHandlerV1::new(
        Arc::clone(&store),
        &data,
        supervisor.relay_port(),
    );
    let operation_id = [11; 16];

    let first = provision_once(
        &handler,
        &principal,
        &browser_signing_key,
        operation_id,
        b"provider-secret",
    );
    assert_eq!(first.operation_id(), &operation_id);
    assert_eq!(first.action(), VaultActionV1::Create);
    assert_eq!(first.secret_revision(), 1);
    let stale_commit = pending_direct_commit(
        &handler,
        &principal,
        &browser_signing_key,
        [13; 16],
        b"stale-provider-secret",
    );

    supervisor
        .stop(vault_binding::VAULT_PROCESS_ID)
        .expect("stop Vault");
    assert_eq!(
        vault_launch::start_from_kernel(
            &supervisor,
            &store,
            &data,
            release.kernel(),
            &data.join("runtime"),
        )
        .expect("restart Vault"),
        2
    );
    assert_eq!(
        handler
            .commit(&principal, stale_commit.request)
            .expect_err("pre-restart provisioning session must be stale"),
        OwnerVaultProvisioningRouteErrorV1::Conflict
    );
    let replay = provision_once(
        &handler,
        &principal,
        &browser_signing_key,
        operation_id,
        b"provider-secret",
    );
    assert_eq!(replay, first);

    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("gateway-cert.der"),
        root.join("gateway-key.der"),
    )
    .expect("Gateway configuration");
    let router = crate::platform::gateway::gateway_service(
        Arc::clone(&store),
        &data,
        supervisor.clone(),
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(1_024)
            .expect("test realtime source"),
        &configuration,
        None,
    )
    .expect("compose owner Vault Gateway");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let cookie = super::browser_gateway_session::authenticate_gateway_router(&router, &runtime);
    let gateway_receipt = provision_once_through_gateway(
        &router,
        &runtime,
        &cookie,
        &browser_signing_key,
        [12; 16],
        b"second-provider-secret",
    );
    assert_eq!(gateway_receipt.operation_id(), &[12; 16]);
    assert_eq!(gateway_receipt.secret_revision(), 1);

    let conflict = provision_once_result(
        &handler,
        &principal,
        &browser_signing_key,
        operation_id,
        b"different-provider-secret",
    );
    assert_eq!(
        conflict.expect_err("operation ID intent mismatch must fail"),
        OwnerVaultProvisioningRouteErrorV1::Unavailable
    );

    supervisor.shutdown().expect("stop managed Vault");
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn provision_once(
    handler: &KernelOwnerVaultProvisioningHandlerV1,
    principal: &OwnerVaultClientPrincipalV1,
    browser_signing_key: &SigningKey,
    operation_id: [u8; 16],
    payload: &[u8],
) -> VaultProvisioningReceiptV1 {
    provision_once_result(
        handler,
        principal,
        browser_signing_key,
        operation_id,
        payload,
    )
    .expect("commit owner Vault provisioning")
}

fn provision_once_result(
    handler: &KernelOwnerVaultProvisioningHandlerV1,
    principal: &OwnerVaultClientPrincipalV1,
    browser_signing_key: &SigningKey,
    operation_id: [u8; 16],
    payload: &[u8],
) -> Result<VaultProvisioningReceiptV1, OwnerVaultProvisioningRouteErrorV1> {
    let pending = pending_direct_commit(
        handler,
        principal,
        browser_signing_key,
        operation_id,
        payload,
    );
    let committed = handler.commit(principal, pending.request)?;
    Ok(open_committed_receipt(
        pending.recipient,
        pending.command_binding,
        committed,
    ))
}

fn pending_direct_commit(
    handler: &KernelOwnerVaultProvisioningHandlerV1,
    principal: &OwnerVaultClientPrincipalV1,
    browser_signing_key: &SigningKey,
    operation_id: [u8; 16],
    payload: &[u8],
) -> PendingOwnerVaultCommitV1 {
    let recipient = VaultResponseRecipientV1::generate();
    let prepared = prepare(
        handler,
        principal,
        operation_id,
        recipient.public_key().as_bytes(),
    );
    let signature: Signature = browser_signing_key.sign(&prepared.challenge_bytes);
    let authorized = handler
        .authorize(
            principal,
            AuthorizeOwnerVaultProvisioningRequestV1 {
                challenge_id: prepared.challenge_id,
                device_signature_raw: signature.to_bytes().to_vec(),
            },
        )
        .expect("authorize owner Vault provisioning");
    seal_commit_payload(operation_id, payload, recipient, authorized)
}

fn prepare(
    handler: &KernelOwnerVaultProvisioningHandlerV1,
    principal: &OwnerVaultClientPrincipalV1,
    operation_id: [u8; 16],
    response_public_key: &[u8; 32],
) -> makosh_gateway_protocol::v1::PrepareOwnerVaultProvisioningResponseV1 {
    handler
        .prepare(
            principal,
            PrepareOwnerVaultProvisioningRequestV1 {
                operation_id: operation_id.to_vec(),
                target_registration_id: REGISTRATION.to_owned(),
                capability_id: CAPABILITY.to_owned(),
                configuration_instance_id: "mail-account-1".to_owned(),
                purpose_id: PURPOSE.to_owned(),
                secret_class: OwnerVaultSecretClassV1::ProviderCredential as i32,
                action: OwnerVaultActionV1::Create as i32,
                secret_revision: 1,
                response_recipient_hpke_public_key_x25519: response_public_key.to_vec(),
            },
        )
        .expect("prepare owner Vault provisioning")
}

struct PendingOwnerVaultCommitV1 {
    recipient: VaultResponseRecipientV1,
    command_binding: VaultTransportBindingV1,
    request: CommitOwnerVaultProvisioningRequestV1,
}

fn seal_commit_payload(
    operation_id: [u8; 16],
    payload: &[u8],
    recipient: VaultResponseRecipientV1,
    authorized: makosh_gateway_protocol::v1::AuthorizeOwnerVaultProvisioningResponseV1,
) -> PendingOwnerVaultCommitV1 {
    let audience = audience(&authorized);
    let lease_binding = binding(
        &authorized,
        audience.clone(),
        array(&authorized.lease_request_id),
        array(&authorized.lease_operation_digest_sha256),
        VaultTransportDirectionV1::FromVault,
        recipient.public_key().as_bytes(),
    );
    let lease_frame = VaultCiphertextFrameV1::from_parts(
        authorized.lease_response_hpke_encapped_key.clone(),
        authorized.lease_response_ciphertext.clone(),
        authorized.lease_response_hpke_authentication_tag.clone(),
    )
    .expect("lease response frame");
    let lease_id = recipient
        .open(&lease_binding, &lease_frame)
        .expect("open lease ID");
    let lease_id = LeaseIdV1::new(String::from_utf8(lease_id.to_vec()).expect("lease ID UTF-8"))
        .expect("lease ID");
    let command = VaultTransportCommandV1::ProvisionLease {
        lease_id,
        operation_id,
        action: VaultActionV1::Create,
        secret_class: SecretClassV1::ProviderCredential,
        payload: payload.to_vec(),
    };
    let command_binding = binding(
        &authorized,
        audience,
        array(&authorized.command_request_id),
        command.operation_digest(),
        VaultTransportDirectionV1::ToVault,
        recipient.public_key().as_bytes(),
    );
    let vault_key =
        VaultTransportPublicKey::from_bytes(array(&authorized.vault_hpke_public_key_x25519))
            .expect("Vault public key");
    let command_frame =
        seal(&vault_key, &command_binding, &command.encode()).expect("seal provisioning command");
    PendingOwnerVaultCommitV1 {
        recipient,
        command_binding,
        request: CommitOwnerVaultProvisioningRequestV1 {
            provisioning_session_id: authorized.provisioning_session_id,
            operation_digest_sha256: command.operation_digest().to_vec(),
            hpke_encapped_key: command_frame.encapped_key().to_vec(),
            ciphertext: command_frame.ciphertext().to_vec(),
            hpke_authentication_tag: command_frame.tag().to_vec(),
        },
    }
}

fn open_committed_receipt(
    recipient: VaultResponseRecipientV1,
    command_binding: VaultTransportBindingV1,
    committed: makosh_gateway_protocol::v1::CommitOwnerVaultProvisioningResponseV1,
) -> VaultProvisioningReceiptV1 {
    let receipt_binding = VaultTransportBindingV1::new(
        committed.vault_runtime_generation,
        command_binding.audience().clone(),
        array(&committed.command_request_id),
        array(&committed.operation_digest_sha256),
        VaultTransportDirectionV1::FromVault,
        *recipient.public_key().as_bytes(),
    )
    .expect("receipt binding");
    let receipt_frame = VaultCiphertextFrameV1::from_parts(
        committed.receipt_hpke_encapped_key,
        committed.receipt_ciphertext,
        committed.receipt_hpke_authentication_tag,
    )
    .expect("receipt frame");
    VaultProvisioningReceiptV1::decode(
        &recipient
            .open(&receipt_binding, &receipt_frame)
            .expect("open receipt"),
    )
    .expect("decode provisioning receipt")
}

fn provision_once_through_gateway(
    router: &GatewayApplicationRouter<ControlStoreBrowserAuthority, InMemoryBrowserRealtimeSource>,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    browser_signing_key: &SigningKey,
    operation_id: [u8; 16],
    payload: &[u8],
) -> VaultProvisioningReceiptV1 {
    let recipient = VaultResponseRecipientV1::generate();
    let prepared: makosh_gateway_protocol::v1::PrepareOwnerVaultProvisioningResponseV1 =
        gateway_call(
            router,
            runtime,
            cookie,
            OWNER_VAULT_PREPARE_PATH,
            PrepareOwnerVaultProvisioningRequestV1 {
                operation_id: operation_id.to_vec(),
                target_registration_id: REGISTRATION.to_owned(),
                capability_id: CAPABILITY.to_owned(),
                configuration_instance_id: "mail-account-2".to_owned(),
                purpose_id: PURPOSE.to_owned(),
                secret_class: OwnerVaultSecretClassV1::ProviderCredential as i32,
                action: OwnerVaultActionV1::Create as i32,
                secret_revision: 1,
                response_recipient_hpke_public_key_x25519: recipient
                    .public_key()
                    .as_bytes()
                    .to_vec(),
            },
        );
    let signature: Signature = browser_signing_key.sign(&prepared.challenge_bytes);
    let authorized = gateway_call(
        router,
        runtime,
        cookie,
        OWNER_VAULT_AUTHORIZE_PATH,
        AuthorizeOwnerVaultProvisioningRequestV1 {
            challenge_id: prepared.challenge_id,
            device_signature_raw: signature.to_bytes().to_vec(),
        },
    );
    let pending = seal_commit_payload(operation_id, payload, recipient, authorized);
    let committed = gateway_call(
        router,
        runtime,
        cookie,
        OWNER_VAULT_COMMIT_PATH,
        pending.request,
    );
    open_committed_receipt(pending.recipient, pending.command_binding, committed)
}

fn gateway_call<RequestV1, ResponseV1>(
    router: &GatewayApplicationRouter<ControlStoreBrowserAuthority, InMemoryBrowserRealtimeSource>,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    request: RequestV1,
) -> ResponseV1
where
    RequestV1: Message,
    ResponseV1: Message + Default,
{
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("origin", "https://hub.local")
                .header("content-type", "application/connect+proto")
                .header("cookie", cookie)
                .body(Full::new(Bytes::from(request.encode_to_vec())))
                .expect("owner Vault Gateway request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = runtime
        .block_on(response.into_body().collect())
        .expect("owner Vault Gateway response")
        .to_bytes();
    ResponseV1::decode(bytes).expect("typed owner Vault Gateway response")
}

fn audience(
    authorized: &makosh_gateway_protocol::v1::AuthorizeOwnerVaultProvisioningResponseV1,
) -> LeaseAudienceV1 {
    LeaseAudienceV1::new(
        authorized.audience_registration_id.clone(),
        authorized.audience_runtime_instance_id.clone(),
        authorized.audience_runtime_generation,
        authorized.audience_grant_epoch,
    )
    .expect("owner Vault audience")
}

fn binding(
    authorized: &makosh_gateway_protocol::v1::AuthorizeOwnerVaultProvisioningResponseV1,
    audience: LeaseAudienceV1,
    request_id: [u8; 16],
    operation_digest: [u8; 32],
    direction: VaultTransportDirectionV1,
    recipient: &[u8; 32],
) -> VaultTransportBindingV1 {
    VaultTransportBindingV1::new(
        authorized.vault_runtime_generation,
        audience,
        request_id,
        operation_digest,
        direction,
        *recipient,
    )
    .expect("owner Vault transport binding")
}

fn array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    bytes.try_into().expect("fixed-size field")
}

fn admit_owner_browser_and_mail(store: &Arc<SqliteControlStore>) -> SigningKey {
    let browser_signing_key = super::browser_gateway_session::browser_signing_key();
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(
            HUMAN_OWNER,
            "owner-device",
            [4; 65],
        ))
        .expect("claim owner");
    super::browser_gateway_session::admit_browser_test_device(store, HUMAN_OWNER);
    let registration = ModuleRegistration::new(
        REGISTRATION,
        "integration.mail",
        "mail",
        [3; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    let purpose = ModuleVaultPurposeRequestV1::new(
        REGISTRATION,
        CAPABILITY,
        PURPOSE,
        120,
        VaultSecretClassV1::ProviderCredential as u8,
        makosh_runtime_protocol::v1::VaultActionV1::Create as u8,
        1,
    );
    store
        .create_pending_registration_with_all_descriptor_requests(
            &registration,
            &[CAPABILITY.to_owned()],
            ModuleDescriptorRegistrationRequestsV1 {
                storage: &[],
                events: &[],
                blobs: &[],
                scheduler: &[],
                vault_purposes: std::slice::from_ref(&purpose),
                client_rpc_routes: &[],
                client_blob_routes: &[],
                client_realtime_routes: &[],
                query_rpc_routes: &[],
                request_rpc_routes: &[],
                contract_dependencies: &[],
            },
        )
        .expect("record Mail registration");
    store
        .approve_module_registration(REGISTRATION, &[CAPABILITY.to_owned()])
        .expect("approve Mail capability");
    browser_signing_key
}
