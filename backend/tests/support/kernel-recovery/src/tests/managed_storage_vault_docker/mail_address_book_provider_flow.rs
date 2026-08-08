//! Live Google People pagination through the signed managed Mail integration.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::*;

use crate::identity::device::signer::DeviceSigner;
use makosh_events_jetstream::DurableSubjectV1;
use makosh_mail_address_book_contract::{
    MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1, MailAddressBookContractV1,
    MailAddressBookEnvelopeContextV1, build_fetch_mail_address_book_page_command_v1,
    build_upsert_mail_address_book_entry_command_v1,
    wire::{
        FetchMailAddressBookPageCommandV1, MailAddressBookEntryObservedV1,
        MailAddressBookEntryUpsertRejectedV1, MailAddressBookPageCompletedV1,
        MailAddressBookRejectCodeV1, UpsertMailAddressBookEntryCommandV1,
    },
};
use makosh_mail_api::{
    account::{MailCredentialBindingStateV1, MailCredentialPurposeV1},
    account_lifecycle::{
        MailAccountLifecycleActionV1, MailAccountLifecycleCommandV1, MailAccountLifecycleStateV1,
        MailCredentialLifecycleStateV1,
    },
};
use makosh_mail_persistence::GmailOAuthCredentialBindingV1;

const OBSERVATION_SUBJECT_V1: &str = "makosh.observation.v1.mail.>";
const RESULT_SUBJECT_V1: &str = "makosh.result.v1.mail.>";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Mail and Google People TLS fixture"]
fn managed_mail_google_people_page_is_exact_restart_safe_and_private() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let provider = MailGmailFixture::start();
    let root = unique_target_root("makosh-managed-mail-google-people");
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
    let admitted = admit_mail_google_people_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Mail address-book Event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_mail_runtime(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let mail = start_mail_google_people_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted,
        MailGmailFixtureSettingsV1 {
            port: provider.port(),
            ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
            oauth: None,
        },
    );
    wait_for_mail_ready(&supervisor, &mail);

    let runtime = tokio::runtime::Runtime::new().expect("Google People conformance runtime");
    let _runtime_context = runtime.enter();
    let durable = runtime.block_on(connect_postgres());
    runtime
        .block_on(durable.initialize())
        .expect("initialize Mail persistence");
    let binding: GmailOAuthCredentialBindingV1 = seeded.contacts_binding();
    runtime
        .block_on(durable.store_gmail_oauth_credential_binding(MAIL_ACCOUNT_ID, &binding, 1))
        .expect("store contacts-authorized Gmail binding");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let (client, context, mut observations, mut results) = runtime.block_on(async {
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect address-book observer");
        let observations = client
            .subscribe(OBSERVATION_SUBJECT_V1)
            .await
            .expect("subscribe address-book observations");
        let results = client
            .subscribe(RESULT_SUBJECT_V1)
            .await
            .expect("subscribe address-book results");
        client
            .flush()
            .await
            .expect("activate address-book observers");
        let context = async_nats::jetstream::new(client.clone());
        (client, context, observations, results)
    });

    let first = fetch_command([0x61; 16], [0x71; 16], 1);
    runtime.block_on(publish(&context, &first));
    let (observed, completed) = runtime.block_on(async {
        (
            receive_contract(&mut observations, MailAddressBookContractV1::EntryObserved).await,
            receive_contract(&mut results, MailAddressBookContractV1::PageCompleted).await,
        )
    });
    let observed_payload = MailAddressBookEntryObservedV1::decode(observed.payload.as_slice())
        .expect("decode Google People observation");
    assert_eq!(
        observed_payload.provider_entry_id,
        "people/managed-contact-1"
    );
    assert_eq!(observed_payload.display_name, "Private Managed Contact");
    assert_eq!(
        observed_payload.email_addresses,
        ["private-managed-contact@example.test"]
    );
    let completed_payload = MailAddressBookPageCompletedV1::decode(completed.payload.as_slice())
        .expect("decode Google People page result");
    assert_eq!(completed_payload.observed_entries, 1);
    assert_eq!(completed_payload.next_continuation_cursor, None);
    assert_eq!(provider.accepted_people_reads(), 1);
    assert_terminal_privacy(&completed.encode_to_vec());

    runtime.block_on(publish(&context, &first));
    assert!(
        runtime
            .block_on(tokio::time::timeout(Duration::from_secs(1), results.next()))
            .is_err(),
        "completed redelivery must not repeat provider IO or terminal publication"
    );
    assert_eq!(provider.accepted_people_reads(), 1);

    let previous_generation = mail.runtime_generation;
    let mail = restart_mail_runtime_without_smtp(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        mail,
        provider.port(),
    );
    assert_eq!(mail.runtime_generation, previous_generation + 1);
    runtime
        .block_on(client.flush())
        .expect("flush after Mail restart");
    let successor = fetch_command([0x62; 16], [0x72; 16], 1);
    runtime.block_on(publish(&context, &successor));
    runtime.block_on(async {
        receive_contract(&mut observations, MailAddressBookContractV1::EntryObserved).await;
        receive_contract(&mut results, MailAddressBookContractV1::PageCompleted).await;
    });
    assert_eq!(provider.accepted_people_reads(), 2);

    runtime
        .block_on(durable.store_gmail_oauth_credential_binding(
            MAIL_ACCOUNT_ID,
            &seeded.binding(),
            2,
        ))
        .expect("remove Google Contacts write authority");
    let missing_scope = upsert_command([0x64; 16], [0x74; 16], 1);
    runtime.block_on(publish(&context, &missing_scope));
    let rejected = runtime.block_on(receive_contract(
        &mut results,
        MailAddressBookContractV1::EntryUpsertRejected,
    ));
    let rejected = MailAddressBookEntryUpsertRejectedV1::decode(rejected.payload.as_slice())
        .expect("decode missing-scope rejection");
    assert_eq!(
        rejected.code,
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeWriteScopeRequired as i32
    );
    assert!(!rejected.outcome_unknown);
    assert_eq!(provider.accepted_people_writes(), 0);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Google People fixture");
    std::fs::remove_dir_all(data).expect("remove Google People Kernel fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Mail and CardDAV TLS fixture"]
fn managed_mail_carddav_page_uses_separate_credential_and_read_only_provider() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let provider = MailCardDavFixture::start();
    let root = unique_target_root("makosh-managed-mail-carddav");
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
    let admitted = admit_mail_carddav_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure CardDAV Event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_mail_runtime(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let mail = start_mail_carddav_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted,
        MailCardDavFixtureSettingsV1 {
            imap_port: 19_993,
            carddav_port: provider.port(),
            ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);

    let runtime = tokio::runtime::Runtime::new().expect("CardDAV conformance runtime");
    let _runtime_context = runtime.enter();
    let durable = runtime.block_on(connect_postgres());
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read CardDAV Event Hub topology")
        .expect("CardDAV Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let (context, mut observations, mut results) = runtime.block_on(async {
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect CardDAV observer");
        let observations = client
            .subscribe(OBSERVATION_SUBJECT_V1)
            .await
            .expect("subscribe CardDAV observations");
        let results = client
            .subscribe(RESULT_SUBJECT_V1)
            .await
            .expect("subscribe CardDAV results");
        client.flush().await.expect("activate CardDAV observers");
        (async_nats::jetstream::new(client), observations, results)
    });
    let command = fetch_command([0x63; 16], [0x73; 16], 1);
    runtime.block_on(publish(&context, &command));
    let (observed, completed) = runtime.block_on(async {
        (
            receive_contract(&mut observations, MailAddressBookContractV1::EntryObserved).await,
            receive_contract(&mut results, MailAddressBookContractV1::PageCompleted).await,
        )
    });
    let observed = MailAddressBookEntryObservedV1::decode(observed.payload.as_slice())
        .expect("decode CardDAV observation");
    assert_eq!(observed.provider_entry_id, "/contacts/book/managed-1.vcf");
    assert_eq!(observed.display_name, "Private CardDAV Contact");
    let completed = MailAddressBookPageCompletedV1::decode(completed.payload.as_slice())
        .expect("decode CardDAV page result");
    assert_eq!(completed.observed_entries, 1);
    assert_eq!(provider.reports(), 1);

    runtime.block_on(publish(&context, &command));
    assert!(
        runtime
            .block_on(tokio::time::timeout(Duration::from_secs(1), results.next()))
            .is_err()
    );
    assert_eq!(provider.reports(), 1);

    let read_only = upsert_command([0x65; 16], [0x75; 16], 1);
    runtime.block_on(publish(&context, &read_only));
    let rejected = runtime.block_on(receive_contract(
        &mut results,
        MailAddressBookContractV1::EntryUpsertRejected,
    ));
    let rejected = MailAddressBookEntryUpsertRejectedV1::decode(rejected.payload.as_slice())
        .expect("decode iCloud read-only rejection");
    assert_eq!(
        rejected.code,
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeReadOnlyProvider as i32
    );
    assert!(!rejected.outcome_unknown);
    assert_eq!(
        provider.reports(),
        1,
        "iCloud rejection must not perform provider IO"
    );

    let lifecycle = runtime
        .block_on(durable.begin_account_lifecycle(
            &MailAccountLifecycleCommandV1 {
                operation_id: "managed-carddav-retire".to_owned(),
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                expected_lifecycle_revision: 0,
            },
            MailAccountLifecycleActionV1::Retire,
            MAIL_ACCOUNT_ID,
            10,
        ))
        .expect("begin CardDAV credential lifecycle");
    assert!(lifecycle.created);
    assert!(lifecycle.receipt.credentials.iter().any(|credential| {
        credential.purpose == MailCredentialPurposeV1::IcloudCardDavPassword
    }));
    let mut terminal = lifecycle.receipt;
    for (offset, credential) in terminal.credentials.clone().into_iter().enumerate() {
        terminal = runtime
            .block_on(durable.record_account_lifecycle_progress(
                MAIL_ACCOUNT_ID,
                "managed-carddav-retire",
                credential.purpose,
                MailCredentialLifecycleStateV1::Completed,
                11 + i64::try_from(offset).expect("bounded lifecycle offset"),
            ))
            .expect("complete CardDAV credential lifecycle");
    }
    assert_eq!(terminal.state, MailAccountLifecycleStateV1::Completed);
    let carddav_binding = runtime
        .block_on(durable.account_credential_binding(
            MAIL_ACCOUNT_ID,
            MailCredentialPurposeV1::IcloudCardDavPassword,
        ))
        .expect("read retired CardDAV binding")
        .expect("retired CardDAV binding");
    assert_eq!(carddav_binding.state, MailCredentialBindingStateV1::Retired);

    supervisor
        .shutdown()
        .expect("stop managed CardDAV processes");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove CardDAV fixture");
    std::fs::remove_dir_all(data).expect("remove CardDAV Kernel fixture");
}

fn fetch_command(
    command_id: [u8; 16],
    run_id: [u8; 16],
    page_sequence: u64,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    let now = wall_seconds();
    build_fetch_mail_address_book_page_command_v1(
        FetchMailAddressBookPageCommandV1 {
            command_id: command_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: "owner-1".to_owned(),
            account_id: MAIL_ACCOUNT_ID.to_owned(),
            page_sequence,
            continuation_cursor: None,
            page_size: 10,
        },
        now + 300,
        &MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "mail-contacts-sync-conformance".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .expect("build address-book fetch command")
}

fn upsert_command(
    command_id: [u8; 16],
    run_id: [u8; 16],
    expected_contact_revision: u64,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    let now = wall_seconds();
    build_upsert_mail_address_book_entry_command_v1(
        UpsertMailAddressBookEntryCommandV1 {
            command_id: command_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: "owner-1".to_owned(),
            account_id: MAIL_ACCOUNT_ID.to_owned(),
            contact_snapshot_reference_id: vec![0x91; 16],
            contact_snapshot_sha256: vec![0x92; 32],
            expected_contact_revision,
            contact_snapshot_declared_bytes: 1,
            contact_snapshot_custody_source_proof: vec![0x93],
        },
        now + 300,
        &MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "mail-contacts-sync-conformance".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .expect("build address-book upsert command")
}

async fn publish(
    context: &async_nats::jetstream::Context,
    record: &makosh_events_protocol::delivery::OutboxRecordV1,
) {
    let envelope =
        DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode address-book fetch command");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive address-book command subject")
        .as_str();
    context
        .publish(subject, record.exact_bytes().to_vec().into())
        .await
        .expect("publish address-book fetch command")
        .await
        .expect("acknowledge address-book fetch command");
}

async fn receive_contract(
    subscriber: &mut async_nats::Subscriber,
    expected: MailAddressBookContractV1,
) -> DurableEnvelopeV1 {
    let message = tokio::time::timeout(Duration::from_secs(15), subscriber.next())
        .await
        .expect("address-book event timeout")
        .expect("address-book event stream");
    let envelope =
        DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode address-book event");
    let contract = envelope.contract.as_ref().expect("address-book contract");
    let expected = expected.reference();
    assert_eq!(contract.owner, expected.owner);
    assert_eq!(contract.name, expected.name);
    assert_eq!(contract.major, expected.major);
    assert_eq!(contract.revision, expected.revision);
    assert_eq!(contract.schema_sha256, expected.schema_sha256);
    envelope
}

fn assert_terminal_privacy(bytes: &[u8]) {
    for private in [
        "Private Managed Contact",
        "private-managed-contact@example.test",
        "managed-mail-gmail-access-token",
        "people/managed-contact-1",
    ] {
        assert!(
            !bytes
                .windows(private.len())
                .any(|window| window == private.as_bytes()),
            "terminal page result exposed provider-private bytes"
        );
    }
}

fn wall_seconds() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("wall clock")
            .as_secs(),
    )
    .expect("wall clock fits i64")
}
