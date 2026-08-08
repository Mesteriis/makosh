//! Live managed Mail launch through Kernel-owned admission and platform leases.

use super::mail_composition_flow::{
    assert_mail_composition, assert_mail_composition_survives_restart,
};
use super::*;

use makosh_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use makosh_mail_api::{
    MailClientRequestV1, MailSendMailRequestV1, MailSyncInboxRequestV1,
    client_contract::MailClientContractV1,
};
use makosh_mail_runtime::admission::MAIL_STORAGE_CAPABILITY_ID;
use makosh_mail_runtime::client_port::encode_module_request;

use crate::identity::device::signer::DeviceSigner;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Mail and NATS binaries"]
fn managed_mail_runtime_uses_kernel_leases_and_route_specific_admission() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let root = unique_target_root("makosh-managed-mail-runtime");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_mail_vault(&vault_dir);
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
    let admitted_mail = admit_mail_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));

    let mut mail = start_mail_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        imap.port(),
    );
    assert_mail_event_only_communications_handoff(&store, &supervisor, &mail);
    assert_mail_attachment_lifecycle(&store, &supervisor, &mail);
    assert_mail_operational_read(&store, &supervisor, &mail);
    assert_mail_message_flags(&store, &supervisor, &mail, &imap);
    assert_mail_composition(&store, &supervisor, &mail);
    let accepted_connections = imap.accepted_connections();
    assert_mail_sync_replay_and_health(
        &store,
        &supervisor,
        &mail,
        "managed-mail-operational-cursor-stale",
        1,
        90,
    );
    assert_eq!(
        imap.accepted_connections(),
        accepted_connections,
        "an exact replayed IMAP sync operation must not reach the provider twice"
    );
    let streaming_operation_id = "managed-mail-streaming-pages";
    imap.enable_streaming_sync_pages();
    sync_mail(&store, &supervisor, &mail, 120, streaming_operation_id);
    let second_page_deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !imap.second_page_requested() {
        assert!(
            std::time::Instant::now() < second_page_deadline,
            "managed IMAP worker did not request the second bounded page"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_sync_run_running(&store, &supervisor, &mail, streaming_operation_id, 121);
    assert_eq!(
        mail_operational_message_count(&store, &supervisor, &mail, 122),
        2,
        "the first IMAP page must be queryable before the second provider page completes"
    );
    imap.release_second_page();
    wait_for_successful_sync_run(&store, &supervisor, &mail, streaming_operation_id, 2, 123);
    let accepted_streaming_connections = imap.accepted_connections();
    sync_mail(&store, &supervisor, &mail, 150, streaming_operation_id);
    assert_eq!(
        imap.accepted_connections(),
        accepted_streaming_connections,
        "replaying the streamed IMAP operation must not reopen the provider"
    );
    mail = restart_mail_runtime_without_smtp(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        mail,
        imap.port(),
    );
    assert_mail_composition_survives_restart(&store, &supervisor, &mail);
    assert_ungranted_delivery_is_rejected(&store, &supervisor, &mail);
    assert_stale_sync_generation_is_rejected(&store, &supervisor, &mail);
    assert!(
        imap.accepted_connections() > 0,
        "managed Mail runtime must reach the live loopback IMAP fixture"
    );
    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    revoke_mail_runtime(
        &owner_runtime_dir,
        &owner_signer,
        &store,
        &supervisor,
        &mail,
    );

    supervisor.shutdown().expect("stop managed processes");
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn revoke_mail_runtime(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let revoked =
        transition_registration(owner_runtime_dir, signer, &mail.registration_id, "revoked");
    assert_eq!(revoked.state, "revoked");
    assert!(
        revoked.grant_epoch > mail.grant_epoch,
        "revoke advances the durable grant epoch before process stop"
    );
    let registration = store
        .module_registration(&mail.registration_id)
        .expect("read revoked Mail registration")
        .expect("revoked Mail registration");
    assert_eq!(registration.state(), ModuleRegistrationState::Revoked);
    let binding = store
        .platform_storage_binding(&mail.registration_id, MAIL_STORAGE_CAPABILITY_ID)
        .expect("read revoked Mail Storage binding")
        .expect("revoked Mail Storage binding");
    assert_eq!(
        binding.state(),
        PlatformStorageBindingStateV1::Revoking,
        "owner transition durably reserves the exact Mail Storage fence"
    );
    assert!(
        !supervisor
            .stop_if_active(&mail.registration_id)
            .expect("observe stopped Mail worker"),
        "owner transition already stopped the exact Mail worker"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("observe Communications worker"),
        "Mail revoke must not stop Communications"
    );
    assert_revoked_sync_route_is_rejected(store, supervisor, mail);
}

fn assert_revoked_sync_route_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        3,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "revoked-mail-sync".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("encode revoked Mail sync module request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        MailClientContractV1::Sync.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("revoked Mail sync route"),
        "module registration is not approved"
    );
}

fn assert_ungranted_delivery_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        1,
        &MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: "ungranted-mail-delivery".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            provider_conversation_id: "conversation-1".to_owned(),
            recipients: vec!["recipient@example.test".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "must not be delivered".to_owned(),
            text_body: "Kernel rejects this route before Mail receives it".to_owned(),
            attachment_anchor_ids: Vec::new(),
        }),
    )
    .expect("encode exact Mail delivery module request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        MailClientContractV1::Delivery.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("ungranted Mail delivery route"),
        "capability is not granted to this registration"
    );
}

fn assert_stale_sync_generation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        2,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "stale-mail-sync".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("encode exact Mail sync module request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation + 1,
        mail.grant_epoch,
        MailClientContractV1::Sync.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("stale Mail sync generation"),
        "managed runtime fence is stale"
    );
}
