//! Live Gmail OAuth setup, rotation, ambiguity and fencing conformance.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use makosh_kernel_control_store::ModuleRegistrationState;
use makosh_mail_api::{
    GmailOAuthAuthorityV1, GmailOAuthCompleteRequestV1, GmailOAuthOutcomeV1,
    GmailOAuthRefreshRequestV1, GmailOAuthStartRequestV1, GmailOAuthStartedV1,
    GmailOAuthStatusRequestV1, MailClientRequestV1, MailClientResponseV1, MailCredentialPurpose,
    client_contract::MailClientContractV1,
};
use makosh_mail_persistence::{GmailOAuthCredentialBindingV1, MailDurablePersistence};
use makosh_mail_runtime::admission::MAIL_CREDENTIAL_LEASE_TTL_SECONDS;
use makosh_mail_runtime::client_port::{
    MailClientPortErrorV1, decode_module_response, encode_module_request,
};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::{SecretRecordId, SecretRecordScope, VaultStore};

use crate::{
    identity::device::signer::{DeviceSigner, FileDeviceSigner},
    modules::capability::router::{ManagedCapabilityRouteRequest, route_managed_client_request},
};

use super::*;

const SETUP_OPERATION_ID: &str = "managed-mail-gmail-oauth-start-1";
const COMPLETE_OPERATION_ID: &str = "managed-mail-gmail-oauth-complete-1";
const REFRESH_OPERATION_ID: &str = "managed-mail-gmail-oauth-refresh-1";
const MISSING_SCOPE_SETUP_OPERATION_ID: &str =
    "managed-mail-gmail-oauth-start-permanent-delete-missing-scope";
const MISSING_SCOPE_OPERATION_ID: &str = "managed-mail-gmail-oauth-permanent-delete-missing-scope";
const HTTP_UNKNOWN_SETUP_OPERATION_ID: &str = "managed-mail-gmail-oauth-start-http-unknown";
const HTTP_UNKNOWN_OPERATION_ID: &str = "managed-mail-gmail-oauth-http-unknown";
const VAULT_UNKNOWN_SETUP_OPERATION_ID: &str = "managed-mail-gmail-oauth-start-vault-unknown";
const VAULT_UNKNOWN_OPERATION_ID: &str = "managed-mail-gmail-oauth-vault-unknown";
const AUTHORIZATION_CODE_V1: &str = "managed-mail-gmail-authorization-code-v1";
const AUTHORIZATION_CODE_MISSING_SCOPE: &str =
    "managed-mail-gmail-authorization-code-permanent-delete-missing-scope";
const AUTHORIZATION_CODE_HTTP_UNKNOWN: &str = "managed-mail-gmail-authorization-code-http-unknown";
const AUTHORIZATION_CODE_VAULT_UNKNOWN: &str =
    "managed-mail-gmail-authorization-code-vault-unknown";

struct ManagedGmailOAuthTestContext {
    provider: MailGmailOAuthFixture,
    root: PathBuf,
    data: PathBuf,
    vault_dir: PathBuf,
    store: Arc<SqliteControlStore>,
    owner_signer: FileDeviceSigner,
    shutdown: Arc<AtomicBool>,
    supervisor: ManagedRuntimeSupervisor,
    mail: StartedMailRuntime,
    runtime: tokio::runtime::Runtime,
    durable: MailDurablePersistence,
}

fn start_managed_gmail_oauth_context() -> ManagedGmailOAuthTestContext {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let provider = MailGmailOAuthFixture::start_successful_rotation();
    let root = unique_target_root("makosh-managed-mail-gmail-oauth");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let release = installed_communications_mail_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim initial owner");
    let admitted_mail = admit_mail_gmail_oauth_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    configure_communications_jetstream(&store);
    let mail = start_mail_gmail_delivery_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        MailGmailFixtureSettingsV1 {
            port: provider.port(),
            ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
            oauth: Some(MailGmailOAuthFixtureSettingsV1 {
                host: provider.host().to_owned(),
                port: provider.port(),
                ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
            }),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);
    let runtime = tokio::runtime::Runtime::new().expect("Gmail OAuth persistence runtime");
    let durable = runtime.block_on(connect_postgres());
    runtime
        .block_on(durable.initialize())
        .expect("initialize Mail persistence");
    ManagedGmailOAuthTestContext {
        provider,
        root,
        data,
        vault_dir,
        store,
        owner_signer,
        shutdown,
        supervisor,
        mail,
        runtime,
        durable,
    }
}

fn remove_managed_gmail_oauth_context(root: PathBuf, data: PathBuf) {
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Gmail OAuth fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Mail, Gmail OAuth TLS and NATS"]
fn managed_mail_gmail_oauth_rotates_credentials_once_and_fails_closed() {
    let ManagedGmailOAuthTestContext {
        provider,
        root,
        data,
        vault_dir,
        store,
        owner_signer: _,
        shutdown: _,
        supervisor,
        mail,
        runtime,
        durable,
    } = start_managed_gmail_oauth_context();
    developer_test_stage("runtime_ready");

    let started = start_oauth(&store, &supervisor, &mail, 1, SETUP_OPERATION_ID);
    let authorization = provider.authorization_material(&started.authorization_url);
    assert_client_text_is_sanitized(
        &started.authorization_url,
        &[
            AUTHORIZATION_CODE_V1,
            "managed-mail-gmail-access-v1",
            "managed-mail-gmail-refresh-v1",
        ],
    );
    complete_oauth(
        &store,
        &supervisor,
        &mail,
        2,
        COMPLETE_OPERATION_ID,
        &started,
        &authorization.state,
        AUTHORIZATION_CODE_V1,
    );
    provider.wait_for_request_count(1);
    let pending = wait_for_oauth_outcome(
        &store,
        &supervisor,
        &mail,
        3,
        COMPLETE_OPERATION_ID,
        GmailOAuthOutcomeV1::Pending,
    );
    assert!(pending.completed_at_unix_seconds.is_none());
    provider.wait_for_response_count(1);
    std::thread::sleep(Duration::from_secs(2));
    let completed = wait_for_oauth_outcome(
        &store,
        &supervisor,
        &mail,
        30,
        COMPLETE_OPERATION_ID,
        GmailOAuthOutcomeV1::Completed,
    );
    assert!(completed.completed_at_unix_seconds.is_some());
    provider.assert_authorization_code_exchange(
        0,
        AUTHORIZATION_CODE_V1,
        &authorization.code_challenge,
    );
    assert_eq!(provider.request_count(), 1);
    developer_test_stage("setup_completed");
    let initial_binding = current_binding(&runtime, &durable);
    assert_binding_shape(&initial_binding, 1);

    let conflicting_complete =
        MailClientRequestV1::GmailOAuthComplete(GmailOAuthCompleteRequestV1 {
            operation_id: "managed-mail-gmail-oauth-conflict".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            setup_id: started.setup_id.clone(),
            state: authorization.state.clone(),
            authorization_code: "managed-mail-gmail-conflicting-code".to_owned(),
        });
    assert!(
        route_mail_client_once(
            &store,
            &supervisor,
            &mail,
            MailClientContractV1::GmailOAuthComplete,
            4,
            &conflicting_complete,
        )
        .is_err(),
        "a consumed Gmail OAuth setup accepted a conflicting replay"
    );
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(provider.request_count(), 1);

    let refresh = MailClientRequestV1::GmailOAuthRefresh(GmailOAuthRefreshRequestV1 {
        operation_id: REFRESH_OPERATION_ID.to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
    });
    let response = route_mail_client(
        &store,
        &supervisor,
        &mail,
        MailClientContractV1::GmailOAuthRefresh,
        5,
        &refresh,
    );
    assert_oauth_accepted(response, REFRESH_OPERATION_ID);
    provider.wait_for_response_count(2);
    std::thread::sleep(Duration::from_secs(2));
    wait_for_oauth_outcome(
        &store,
        &supervisor,
        &mail,
        6,
        REFRESH_OPERATION_ID,
        GmailOAuthOutcomeV1::Completed,
    );
    provider.assert_refresh_exchange(1);
    assert_eq!(provider.request_count(), 2);
    let rotated_binding = current_binding(&runtime, &durable);
    assert_ne!(
        rotated_binding.access_token_record_id,
        initial_binding.access_token_record_id
    );
    assert_ne!(
        rotated_binding.refresh_credential_record_id,
        initial_binding.refresh_credential_record_id
    );
    assert_binding_shape(&rotated_binding, 2);

    let missing_scope_started = start_oauth_with_authority(
        &store,
        &supervisor,
        &mail,
        700,
        MISSING_SCOPE_SETUP_OPERATION_ID,
        GmailOAuthAuthorityV1::PermanentDelete,
    );
    let missing_scope_authorization =
        provider.permanent_delete_authorization_material(&missing_scope_started.authorization_url);
    complete_oauth(
        &store,
        &supervisor,
        &mail,
        701,
        MISSING_SCOPE_OPERATION_ID,
        &missing_scope_started,
        &missing_scope_authorization.state,
        AUTHORIZATION_CODE_MISSING_SCOPE,
    );
    provider.wait_for_request_count(3);
    provider.wait_for_response_count(3);
    wait_for_oauth_outcome(
        &store,
        &supervisor,
        &mail,
        702,
        MISSING_SCOPE_OPERATION_ID,
        GmailOAuthOutcomeV1::Rejected,
    );
    provider.assert_authorization_code_exchange(
        2,
        AUTHORIZATION_CODE_MISSING_SCOPE,
        &missing_scope_authorization.code_challenge,
    );
    assert_eq!(
        current_binding(&runtime, &durable),
        rotated_binding,
        "an under-scoped permanent-delete consent must not rotate the credential binding"
    );
    assert_stale_client_fences(&store, &supervisor, &mail);
    developer_test_stage("refresh_completed");

    let http_unknown_started = start_oauth(
        &store,
        &supervisor,
        &mail,
        7,
        HTTP_UNKNOWN_SETUP_OPERATION_ID,
    );
    let http_unknown_authorization =
        provider.authorization_material(&http_unknown_started.authorization_url);
    complete_oauth(
        &store,
        &supervisor,
        &mail,
        8,
        HTTP_UNKNOWN_OPERATION_ID,
        &http_unknown_started,
        &http_unknown_authorization.state,
        AUTHORIZATION_CODE_HTTP_UNKNOWN,
    );
    provider.wait_for_request_count(4);
    std::thread::sleep(Duration::from_secs(1));
    wait_for_oauth_outcome(
        &store,
        &supervisor,
        &mail,
        9,
        HTTP_UNKNOWN_OPERATION_ID,
        GmailOAuthOutcomeV1::OutcomeUnknown,
    );
    provider.assert_authorization_code_exchange(
        3,
        AUTHORIZATION_CODE_HTTP_UNKNOWN,
        &http_unknown_authorization.code_challenge,
    );
    std::thread::sleep(Duration::from_millis(1_200));
    assert_eq!(
        provider.request_count(),
        4,
        "ambiguous Gmail OAuth HTTP exchange was retried without a new command"
    );
    assert_eq!(current_binding(&runtime, &durable), rotated_binding);
    developer_test_stage("http_unknown_completed");

    let vault_unknown_started = start_oauth(
        &store,
        &supervisor,
        &mail,
        10,
        VAULT_UNKNOWN_SETUP_OPERATION_ID,
    );
    let vault_unknown_authorization =
        provider.authorization_material(&vault_unknown_started.authorization_url);
    supervisor
        .stop(vault_binding::VAULT_PROCESS_ID)
        .expect("stop Vault before ambiguous credential mutation");
    complete_oauth(
        &store,
        &supervisor,
        &mail,
        11,
        VAULT_UNKNOWN_OPERATION_ID,
        &vault_unknown_started,
        &vault_unknown_authorization.state,
        AUTHORIZATION_CODE_VAULT_UNKNOWN,
    );
    provider.wait_for_request_count(5);
    provider.wait_for_response_count(4);
    std::thread::sleep(Duration::from_secs(2));
    wait_for_oauth_outcome(
        &store,
        &supervisor,
        &mail,
        12,
        VAULT_UNKNOWN_OPERATION_ID,
        GmailOAuthOutcomeV1::OutcomeUnknown,
    );
    provider.assert_authorization_code_exchange(
        4,
        AUTHORIZATION_CODE_VAULT_UNKNOWN,
        &vault_unknown_authorization.code_challenge,
    );
    std::thread::sleep(Duration::from_millis(1_200));
    assert_eq!(
        provider.request_count(),
        5,
        "ambiguous Vault outcome caused a hidden Gmail OAuth retry"
    );
    assert_eq!(current_binding(&runtime, &durable), rotated_binding);
    assert!(
        runtime
            .block_on(durable.pending_communications_outbox(4))
            .expect("read Mail Communications outbox after OAuth")
            .is_empty(),
        "Gmail OAuth credential lifecycle leaked into Communications events"
    );
    assert_runtime_diagnostics_are_sanitized(&supervisor, &mail);
    developer_test_stage("vault_unknown_completed");

    supervisor.shutdown().expect("stop managed processes");
    assert_vault_rotation_state(&vault_dir, &initial_binding, &rotated_binding);
    remove_managed_gmail_oauth_context(root, data);
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Mail and NATS"]
fn managed_mail_gmail_oauth_route_is_fenced_by_owner_revoke() {
    let context = start_managed_gmail_oauth_context();
    developer_test_stage("revoke_runtime_ready");
    let started = start_oauth(
        &context.store,
        &context.supervisor,
        &context.mail,
        1,
        "managed-mail-gmail-oauth-revoke-start",
    );
    assert_client_text_is_sanitized(
        &started.authorization_url,
        &[
            AUTHORIZATION_CODE_V1,
            "managed-mail-gmail-access-v1",
            "managed-mail-gmail-refresh-v1",
        ],
    );
    let (owner_runtime_dir, owner_control) = start_owner_control(
        &context.data,
        &context.store,
        &context.shutdown,
        &context.supervisor,
    );
    let revoked = transition_registration(
        &owner_runtime_dir,
        &context.owner_signer,
        &context.mail.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(revoked.grant_epoch > context.mail.grant_epoch);
    assert_eq!(
        context
            .store
            .module_registration(&context.mail.registration_id)
            .expect("read revoked Mail registration")
            .expect("revoked Mail registration")
            .state(),
        ModuleRegistrationState::Revoked
    );
    assert_revoked_oauth_route_is_rejected(&context.store, &context.supervisor, &context.mail);
    developer_test_stage("runtime_revoked");
    context
        .supervisor
        .shutdown()
        .expect("stop managed processes");
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server result");
    let ManagedGmailOAuthTestContext { root, data, .. } = context;
    remove_managed_gmail_oauth_context(root, data);
}

fn developer_test_stage(stage: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_gmail_oauth_test_stage={stage}");
    }
}

fn start_oauth(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) -> GmailOAuthStartedV1 {
    start_oauth_with_authority(
        store,
        supervisor,
        mail,
        request_id,
        operation_id,
        GmailOAuthAuthorityV1::Operational,
    )
}

fn start_oauth_with_authority(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
    authority: GmailOAuthAuthorityV1,
) -> GmailOAuthStartedV1 {
    let response = route_mail_client(
        store,
        supervisor,
        mail,
        MailClientContractV1::GmailOAuthStart,
        request_id,
        &MailClientRequestV1::GmailOAuthStart(GmailOAuthStartRequestV1 {
            operation_id: operation_id.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            authority,
        }),
    );
    match response {
        MailClientResponseV1::GmailOAuthStarted(value) => value,
        _ => panic!("unexpected Gmail OAuth start response"),
    }
}

#[allow(clippy::too_many_arguments)]
fn complete_oauth(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
    started: &GmailOAuthStartedV1,
    state: &str,
    authorization_code: &str,
) {
    let response = route_mail_client(
        store,
        supervisor,
        mail,
        MailClientContractV1::GmailOAuthComplete,
        request_id,
        &MailClientRequestV1::GmailOAuthComplete(GmailOAuthCompleteRequestV1 {
            operation_id: operation_id.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            setup_id: started.setup_id.clone(),
            state: state.to_owned(),
            authorization_code: authorization_code.to_owned(),
        }),
    );
    assert_oauth_accepted(response, operation_id);
}

fn assert_oauth_accepted(response: MailClientResponseV1, operation_id: &str) {
    match response {
        MailClientResponseV1::GmailOAuthAccepted {
            operation_id: accepted,
        } => assert_eq!(accepted, operation_id),
        _ => panic!("unexpected Gmail OAuth accepted response"),
    }
}

fn wait_for_oauth_outcome(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
    expected: GmailOAuthOutcomeV1,
) -> makosh_mail_api::GmailOAuthOperationStatusV1 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_mail_client(
            store,
            supervisor,
            mail,
            MailClientContractV1::GmailOAuthQuery,
            request_id,
            &MailClientRequestV1::GmailOAuthStatus(GmailOAuthStatusRequestV1 {
                operation_id: operation_id.to_owned(),
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
            }),
        );
        let MailClientResponseV1::GmailOAuthStatus(Some(status)) = response else {
            panic!("unexpected Gmail OAuth query response");
        };
        if status.outcome == expected {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "Gmail OAuth operation did not reach its expected terminal outcome"
        );
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn current_binding(
    runtime: &tokio::runtime::Runtime,
    durable: &MailDurablePersistence,
) -> GmailOAuthCredentialBindingV1 {
    runtime
        .block_on(durable.gmail_oauth_credential_binding(MAIL_ACCOUNT_ID))
        .expect("read Gmail OAuth credential binding")
        .expect("Gmail OAuth credential binding")
}

fn assert_binding_shape(binding: &GmailOAuthCredentialBindingV1, revision: u64) {
    assert!(
        binding.access_token_record_id.iter().any(|byte| *byte != 0)
            && binding
                .refresh_credential_record_id
                .iter()
                .any(|byte| *byte != 0),
        "Mail binding did not retain opaque Vault record IDs"
    );
    assert_ne!(
        binding.access_token_record_id,
        binding.refresh_credential_record_id
    );
    assert_eq!(binding.access_token_revision, revision);
    assert_eq!(binding.refresh_credential_revision, revision);
    assert!(binding.access_token_expires_at_unix_seconds > 0);
    assert!(binding.scope_sha256.iter().any(|byte| *byte != 0));
    assert!(!binding.permanent_delete_authorized);
}

fn route_mail_client_once(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request_id: u64,
    request: &MailClientRequestV1,
) -> Result<MailClientResponseV1, String> {
    let encoded = encode_module_request(request_id, request)
        .map_err(|_| "Mail client request encoding failed".to_owned())?;
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        contract.capability_id(),
        &encoded,
    );
    let response = route_managed_client_request(store, &supervisor.relay_port(), &route)?;
    let (response_id, response) =
        decode_module_response(contract, &response).map_err(|error| match error {
            MailClientPortErrorV1::Protocol => "Mail client protocol failed".to_owned(),
            MailClientPortErrorV1::Runtime => "Mail client runtime failed".to_owned(),
        })?;
    (response_id == request_id)
        .then_some(response)
        .ok_or_else(|| "Mail client response identity differs".to_owned())
}

fn assert_stale_client_fences(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        20,
        &MailClientRequestV1::GmailOAuthStart(GmailOAuthStartRequestV1 {
            operation_id: "stale-mail-gmail-oauth".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            authority: GmailOAuthAuthorityV1::Operational,
        }),
    )
    .expect("encode stale Gmail OAuth request");
    for (runtime_generation, grant_epoch) in [
        (mail.runtime_generation + 1, mail.grant_epoch),
        (mail.runtime_generation, mail.grant_epoch + 1),
    ] {
        let route = ManagedCapabilityRouteRequest::new(
            &mail.registration_id,
            &mail.runtime_instance_id,
            runtime_generation,
            grant_epoch,
            MailClientContractV1::GmailOAuthStart.capability_id(),
            &request,
        );
        assert!(
            route_managed_client_request(store, &supervisor.relay_port(), &route).is_err(),
            "stale Gmail OAuth runtime fence reached Mail"
        );
    }
}

fn assert_revoked_oauth_route_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        21,
        &MailClientRequestV1::GmailOAuthStatus(GmailOAuthStatusRequestV1 {
            operation_id: COMPLETE_OPERATION_ID.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("encode revoked Gmail OAuth query");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        MailClientContractV1::GmailOAuthQuery.capability_id(),
        &request,
    );
    assert!(
        route_managed_client_request(store, &supervisor.relay_port(), &route).is_err(),
        "revoked Gmail OAuth route reached Mail"
    );
}

fn assert_runtime_diagnostics_are_sanitized(
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let diagnostic = format!(
        "{:?}",
        supervisor
            .last_failure(&mail.registration_id)
            .expect("read Mail runtime diagnostic")
    );
    assert_client_text_is_sanitized(
        &diagnostic,
        &[
            AUTHORIZATION_CODE_V1,
            AUTHORIZATION_CODE_HTTP_UNKNOWN,
            AUTHORIZATION_CODE_VAULT_UNKNOWN,
            "managed-mail-gmail-access-v1",
            "managed-mail-gmail-access-v2",
            "managed-mail-gmail-access-v3",
            "managed-mail-gmail-refresh-v1",
            "managed-mail-gmail-refresh-v2",
            "managed-mail-gmail-refresh-v3",
        ],
    );
}

fn assert_client_text_is_sanitized(value: &str, private_values: &[&str]) {
    assert!(
        private_values
            .iter()
            .all(|private| !value.contains(private)),
        "Gmail OAuth client or diagnostic surface exposed private credential material"
    );
}

fn assert_vault_rotation_state(
    vault_dir: &Path,
    previous: &GmailOAuthCredentialBindingV1,
    binding: &GmailOAuthCredentialBindingV1,
) {
    let key = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("open Vault wrapping key");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("open stopped Vault");
    assert_current_secret(
        &store,
        binding.access_token_record_id,
        MailCredentialPurpose::GmailAccessToken,
        SecretClassV1::ProviderCredential,
        binding.access_token_revision,
        b"managed-mail-gmail-access-v2",
    );
    assert_current_secret(
        &store,
        binding.refresh_credential_record_id,
        MailCredentialPurpose::GmailRefreshCredential,
        SecretClassV1::OAuthRefreshCredential,
        binding.refresh_credential_revision,
        b"managed-mail-gmail-refresh-v2",
    );
    assert_stale_secret_revision_rejected(
        &store,
        binding.access_token_record_id,
        MailCredentialPurpose::GmailAccessToken,
        SecretClassV1::ProviderCredential,
    );
    assert_stale_secret_revision_rejected(
        &store,
        binding.refresh_credential_record_id,
        MailCredentialPurpose::GmailRefreshCredential,
        SecretClassV1::OAuthRefreshCredential,
    );
    assert_stale_secret_revision_rejected(
        &store,
        previous.access_token_record_id,
        MailCredentialPurpose::GmailAccessToken,
        SecretClassV1::ProviderCredential,
    );
    assert_stale_secret_revision_rejected(
        &store,
        previous.refresh_credential_record_id,
        MailCredentialPurpose::GmailRefreshCredential,
        SecretClassV1::OAuthRefreshCredential,
    );
}

fn assert_current_secret(
    store: &VaultStore,
    record_id: [u8; 16],
    purpose: MailCredentialPurpose,
    secret_class: SecretClassV1,
    revision: u64,
    expected: &[u8],
) {
    let scope = credential_scope(purpose, secret_class, revision);
    let secret = store
        .resolve_scoped_secret(&SecretRecordId::from_bytes(record_id), &scope)
        .expect("resolve current Gmail OAuth credential");
    assert!(
        secret.as_slice() == expected,
        "Vault current credential differs from the exact provider rotation"
    );
}

fn assert_stale_secret_revision_rejected(
    store: &VaultStore,
    record_id: [u8; 16],
    purpose: MailCredentialPurpose,
    secret_class: SecretClassV1,
) {
    let stale = credential_scope(purpose, secret_class, 1);
    assert!(
        store
            .resolve_scoped_secret(&SecretRecordId::from_bytes(record_id), &stale)
            .is_err(),
        "Vault resolved a stale Gmail OAuth credential revision"
    );
}

fn credential_scope(
    purpose: MailCredentialPurpose,
    secret_class: SecretClassV1,
    revision: u64,
) -> SecretRecordScope {
    let request = VaultPurposeRequestV1::new(
        purpose.as_str().to_owned(),
        MAIL_ACCOUNT_ID.to_owned(),
        vec![secret_class],
        vec![VaultActionV1::Resolve],
        MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
    )
    .expect("Gmail OAuth credential purpose");
    SecretRecordScope::new(
        makosh_mail_api::client_contract::MAIL_OWNER_ID.to_owned(),
        &request,
        secret_class,
        revision,
    )
    .expect("Gmail OAuth credential scope")
}
