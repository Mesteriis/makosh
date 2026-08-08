//! Live Mail credential binding, provider quiesce and Settings successor evidence.

use std::time::Duration;

use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSendMailRequestV1, MailSyncInboxRequestV1,
    account::{
        MailAccountReadinessV1, MailAccountStatusRequestV1, MailAccountStatusV1,
        MailBindCredentialRequestV1, MailCredentialBindingStateV1, MailCredentialPurposeV1,
    },
    account_lifecycle::{
        MailAccountLifecycleActionV1, MailAccountLifecycleCommandV1, MailAccountLifecycleReceiptV1,
        MailAccountLifecycleRetryV1, MailAccountLifecycleStateV1,
        MailAccountLifecycleStatusRequestV1, MailCredentialLifecycleStateV1,
    },
    client_contract::MailClientContractV1,
};
use makosh_mail_runtime::admission::MAIL_STORAGE_CAPABILITY_ID;
use prost::Message;

use crate::identity::device::signer::DeviceSigner;

use super::*;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Mail, IMAP, SMTP and NATS"]
fn managed_mail_credential_rotation_quiesces_until_settings_successor() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let smtp = MailSmtpFixture::start();
    let root = unique_target_root("makosh-managed-mail-account-credential");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let seeded = seed_mail_vault(&vault_dir);
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
    let admitted_mail = admit_mail_account_credential_runtime(&store);
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
    let predecessor = start_mail_delivery_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        imap.port(),
        smtp_settings(&smtp),
    );
    wait_for_mail_ready(&supervisor, &predecessor);
    let gmail_binding_runtime =
        tokio::runtime::Runtime::new().expect("Mail Gmail lifecycle binding runtime");
    gmail_binding_runtime.block_on(async {
        super::mail_event_flow::connect_postgres()
            .await
            .store_gmail_oauth_credential_binding(MAIL_ACCOUNT_ID, &seeded.binding(), 1)
            .await
            .expect("seed Gmail lifecycle credential binding");
    });

    let active = query_account_status(&store, &supervisor, &predecessor, 81);
    assert_eq!(active.readiness, MailAccountReadinessV1::Ready);
    assert!(active.bindings.iter().all(|binding| {
        binding.state == MailCredentialBindingStateV1::Active
            && binding.credential_revision == Some(1)
            && binding.applied_runtime_generation == Some(predecessor.runtime_generation)
    }));

    rotate_basic_mail_vault(&vault_dir, &seeded);
    for (request_id, purpose) in [
        (82, MailCredentialPurposeV1::ImapPassword),
        (83, MailCredentialPurposeV1::SmtpPassword),
    ] {
        let response = route_mail_client(
            &store,
            &supervisor,
            &predecessor,
            MailClientContractV1::AccountCredentialBind,
            request_id,
            &MailClientRequestV1::BindCredential(MailBindCredentialRequestV1 {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                purpose,
                expected_binding_revision: 1,
                credential_revision: 2,
            }),
        );
        let MailClientResponseV1::CredentialBinding(receipt) = response else {
            panic!("Mail credential bind returned the wrong response");
        };
        assert_eq!(receipt.binding_revision, 2);
        assert_eq!(receipt.state, MailCredentialBindingStateV1::PendingRestart);
    }
    let pending = query_account_status(&store, &supervisor, &predecessor, 84);
    assert_eq!(pending.readiness, MailAccountReadinessV1::PendingRestart);
    assert!(pending.bindings.iter().all(|binding| {
        binding.state == MailCredentialBindingStateV1::PendingRestart
            && binding.credential_revision == Some(2)
            && binding.applied_runtime_generation.is_none()
    }));

    let sync_error = route_mail_client_once(
        &store,
        &supervisor,
        &predecessor,
        MailClientContractV1::Sync,
        85,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "mail-sync-quiesced".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect_err("pending IMAP binding must quiesce provider sync");
    assert_eq!(sync_error, "Mail route runtime error");
    let delivery_error = route_mail_client_once(
        &store,
        &supervisor,
        &predecessor,
        MailClientContractV1::Delivery,
        86,
        &delivery_request("mail-delivery-quiesced"),
    )
    .expect_err("pending SMTP binding must quiesce provider delivery");
    assert_eq!(delivery_error, "Mail route runtime error");
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(imap.accepted_connections(), 0);
    assert_eq!(smtp.accepted_messages(), 0);

    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    let (client, owner_session) = open_owner_control_client(&owner_runtime_dir, &owner_signer);
    let revision_two = mail_delivery_settings_snapshot(
        &predecessor.registration_id,
        imap.port(),
        smtp_settings(&smtp),
        2,
    )
    .encode_to_vec();
    client
        .update_operator_settings(
            &owner_session,
            &predecessor.registration_id,
            1,
            revision_two,
        )
        .expect("commit Mail Settings revision two");
    let applied = client
        .apply_managed_integration_settings(
            &owner_session,
            &predecessor.registration_id,
            MAIL_STORAGE_CAPABILITY_ID,
            MAIL_ACCOUNT_ID,
            2,
            false,
        )
        .expect("apply Mail credential-rotation successor");
    assert_eq!(
        applied.runtime_generation,
        predecessor.runtime_generation + 1
    );
    let successor = current_mail_runtime(&store, &predecessor);
    wait_for_mail_ready(&supervisor, &successor);
    let ready = query_account_status(&store, &supervisor, &successor, 87);
    assert_eq!(ready.readiness, MailAccountReadinessV1::Ready);
    assert!(ready.bindings.iter().all(|binding| {
        binding.state == MailCredentialBindingStateV1::Active
            && binding.credential_revision == Some(2)
            && binding.applied_runtime_generation == Some(successor.runtime_generation)
    }));

    let sync = route_mail_client(
        &store,
        &supervisor,
        &successor,
        MailClientContractV1::Sync,
        88,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "mail-sync-successor".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    );
    assert!(matches!(
        sync,
        MailClientResponseV1::SyncInboxAccepted { .. }
    ));
    let accepted = route_mail_client(
        &store,
        &supervisor,
        &successor,
        MailClientContractV1::Delivery,
        89,
        &delivery_request("mail-delivery-successor"),
    );
    assert_eq!(
        accepted,
        MailClientResponseV1::MailAccepted {
            operation_id: "mail-delivery-successor".to_owned(),
        }
    );
    assert_delivery_completed(
        &store,
        &supervisor,
        &successor,
        "mail-delivery-successor",
        250,
    );
    assert_eq!(smtp.accepted_messages(), 1);

    supervisor
        .stop(vault_binding::VAULT_PROCESS_ID)
        .expect("stop Vault before ambiguous Mail retire");
    let retire_unknown = lifecycle_command(
        &store,
        &supervisor,
        &successor,
        MailClientContractV1::AccountRetire,
        90,
        MailClientRequestV1::RetireAccount(MailAccountLifecycleCommandV1 {
            operation_id: "mail-account-retire".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            expected_lifecycle_revision: 0,
        }),
    );
    assert_lifecycle_state(
        &retire_unknown,
        MailAccountLifecycleActionV1::Retire,
        1,
        4,
        MailAccountLifecycleStateV1::OutcomeUnknown,
        MailCredentialLifecycleStateV1::OutcomeUnknown,
    );
    let replayed_retire = lifecycle_command(
        &store,
        &supervisor,
        &successor,
        MailClientContractV1::AccountRetire,
        91,
        MailClientRequestV1::RetireAccount(MailAccountLifecycleCommandV1 {
            operation_id: "mail-account-retire".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            expected_lifecycle_revision: 0,
        }),
    );
    assert_eq!(replayed_retire, retire_unknown);
    assert_eq!(imap.accepted_connections(), 1);
    assert_eq!(smtp.accepted_messages(), 1);

    supervisor
        .stop("storage")
        .expect("stop Storage before rebinding the successor Vault generation");
    assert!(
        start_vault(&supervisor, &store, &data, release.kernel()) > 1,
        "Vault restart must advance its runtime generation"
    );
    assert!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory(),
        ) > 1,
        "Storage restart must bind the successor Vault generation"
    );
    let revision_three = mail_delivery_settings_snapshot(
        &successor.registration_id,
        imap.port(),
        smtp_settings(&smtp),
        3,
    )
    .encode_to_vec();
    client
        .update_operator_settings(
            &owner_session,
            &successor.registration_id,
            2,
            revision_three,
        )
        .expect("commit Mail Settings revision three");
    client
        .apply_managed_integration_settings(
            &owner_session,
            &successor.registration_id,
            MAIL_STORAGE_CAPABILITY_ID,
            MAIL_ACCOUNT_ID,
            3,
            false,
        )
        .expect("restart Mail for explicit lifecycle retry");
    let lifecycle_successor = current_mail_runtime(&store, &successor);
    wait_for_mail_ready(&supervisor, &lifecycle_successor);

    let retire = lifecycle_command(
        &store,
        &supervisor,
        &lifecycle_successor,
        MailClientContractV1::AccountLifecycleRetry,
        92,
        MailClientRequestV1::RetryAccountLifecycle(MailAccountLifecycleRetryV1 {
            operation_id: "mail-account-retire".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            expected_lifecycle_revision: 1,
        }),
    );
    assert_lifecycle_completed(&retire, MailAccountLifecycleActionV1::Retire, 1, 4);
    let retire_status = lifecycle_command(
        &store,
        &supervisor,
        &lifecycle_successor,
        MailClientContractV1::AccountLifecycleQuery,
        93,
        MailClientRequestV1::AccountLifecycleStatus(MailAccountLifecycleStatusRequestV1 {
            operation_id: "mail-account-retire".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    );
    assert_eq!(retire_status, retire);
    assert_eq!(
        query_account_status(&store, &supervisor, &lifecycle_successor, 94).readiness,
        MailAccountReadinessV1::Retired
    );
    assert!(
        route_mail_client_once(
            &store,
            &supervisor,
            &lifecycle_successor,
            MailClientContractV1::Sync,
            95,
            &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
                operation_id: "mail-sync-retired".to_owned(),
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
            }),
        )
        .is_err()
    );
    assert_eq!(imap.accepted_connections(), 1);
    assert_eq!(smtp.accepted_messages(), 1);

    let delete = lifecycle_command(
        &store,
        &supervisor,
        &lifecycle_successor,
        MailClientContractV1::AccountDelete,
        96,
        MailClientRequestV1::DeleteAccount(MailAccountLifecycleCommandV1 {
            operation_id: "mail-account-delete".to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            expected_lifecycle_revision: 1,
        }),
    );
    assert_lifecycle_completed(&delete, MailAccountLifecycleActionV1::Delete, 2, 4);
    assert_eq!(
        query_account_status(&store, &supervisor, &lifecycle_successor, 97).readiness,
        MailAccountReadinessV1::Deleted
    );
    let persistence = tokio::runtime::Runtime::new().expect("Mail tombstone query runtime");
    assert!(persistence.block_on(async {
        super::mail_event_flow::connect_postgres()
            .await
            .account_is_tombstoned(MAIL_ACCOUNT_ID)
            .await
            .expect("query Mail account tombstone")
    }));

    let revision_four = mail_delivery_settings_snapshot(
        &lifecycle_successor.registration_id,
        imap.port(),
        smtp_settings(&smtp),
        4,
    )
    .encode_to_vec();
    client
        .update_operator_settings(
            &owner_session,
            &lifecycle_successor.registration_id,
            3,
            revision_four,
        )
        .expect("commit Mail Settings revision four");
    client
        .apply_managed_integration_settings(
            &owner_session,
            &lifecycle_successor.registration_id,
            MAIL_STORAGE_CAPABILITY_ID,
            MAIL_ACCOUNT_ID,
            4,
            false,
        )
        .expect("restart deleted Mail account");
    let deleted_successor = current_mail_runtime(&store, &lifecycle_successor);
    wait_for_mail_ready(&supervisor, &deleted_successor);
    assert_eq!(
        query_account_status(&store, &supervisor, &deleted_successor, 98).readiness,
        MailAccountReadinessV1::Deleted
    );
    assert!(
        route_mail_client_once(
            &store,
            &supervisor,
            &deleted_successor,
            MailClientContractV1::AccountDelete,
            99,
            &MailClientRequestV1::DeleteAccount(MailAccountLifecycleCommandV1 {
                operation_id: "mail-account-delete-after-tombstone".to_owned(),
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                expected_lifecycle_revision: 2,
            }),
        )
        .is_err(),
        "a Mail account tombstone must reject a new lifecycle mutation"
    );
    assert_eq!(imap.accepted_connections(), 1);
    assert_eq!(smtp.accepted_messages(), 1);

    assert!(
        route_mail_client_once(
            &store,
            &supervisor,
            &predecessor,
            MailClientContractV1::AccountQuery,
            100,
            &MailClientRequestV1::AccountStatus(MailAccountStatusRequestV1 {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
            }),
        )
        .is_err(),
        "stale Mail generation must not retain its query route"
    );

    supervisor.shutdown().expect("stop managed processes");
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail credential fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn lifecycle_command(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request_id: u64,
    request: MailClientRequestV1,
) -> MailAccountLifecycleReceiptV1 {
    let response = route_mail_client(store, supervisor, mail, contract, request_id, &request);
    let MailClientResponseV1::AccountLifecycle(receipt) = response else {
        panic!("Mail lifecycle route returned the wrong response");
    };
    receipt
}

fn assert_lifecycle_completed(
    receipt: &MailAccountLifecycleReceiptV1,
    action: MailAccountLifecycleActionV1,
    lifecycle_revision: u64,
    credential_count: usize,
) {
    assert_lifecycle_state(
        receipt,
        action,
        lifecycle_revision,
        credential_count,
        MailAccountLifecycleStateV1::Completed,
        MailCredentialLifecycleStateV1::Completed,
    );
}

fn assert_lifecycle_state(
    receipt: &MailAccountLifecycleReceiptV1,
    action: MailAccountLifecycleActionV1,
    lifecycle_revision: u64,
    credential_count: usize,
    lifecycle_state: MailAccountLifecycleStateV1,
    credential_state: MailCredentialLifecycleStateV1,
) {
    assert_eq!(receipt.action, action);
    assert_eq!(receipt.lifecycle_revision, lifecycle_revision);
    assert_eq!(receipt.state, lifecycle_state);
    assert_eq!(receipt.credentials.len(), credential_count);
    assert!(receipt.credentials.iter().all(|progress| {
        progress.state == credential_state
            && progress.credential_revision
                == if progress.purpose.bindable_by_client() {
                    2
                } else {
                    1
                }
    }));
}

fn query_account_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
) -> MailAccountStatusV1 {
    let response = route_mail_client(
        store,
        supervisor,
        mail,
        MailClientContractV1::AccountQuery,
        request_id,
        &MailClientRequestV1::AccountStatus(MailAccountStatusRequestV1 {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    );
    let MailClientResponseV1::AccountStatus(status) = response else {
        panic!("Mail account query returned the wrong response");
    };
    status
}

fn delivery_request(operation_id: &str) -> MailClientRequestV1 {
    MailClientRequestV1::SendMail(MailSendMailRequestV1 {
        operation_id: operation_id.to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        provider_conversation_id: "mail-credential-conversation".to_owned(),
        recipients: vec!["recipient@example.test".to_owned()],
        cc_recipients: Vec::new(),
        bcc_recipients: Vec::new(),
        subject: "credential rotation".to_owned(),
        text_body: "credential rotation body".to_owned(),
        attachment_anchor_ids: Vec::new(),
    })
}

fn smtp_settings(fixture: &MailSmtpFixture) -> MailSmtpFixtureSettingsV1 {
    MailSmtpFixtureSettingsV1 {
        port: fixture.port(),
        ca_certificate_pem: fixture.ca_certificate_pem().to_owned(),
    }
}
