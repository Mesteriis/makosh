//! Live signed Contacts command consumption over Vault, Storage and NATS.

use super::*;

use futures_util::StreamExt;
use makosh_contacts_command_api::{
    ContactsCommandEnvelopeContextV1, build_upsert_contact_command_outbox_record_v1,
    contact_upsert_rejected_contract_reference_v1, contact_upserted_contract_reference_v1,
    wire::{
        ContactUpsertFromMailAddressBookEntryRejectedV1, ContactUpsertedFromMailAddressBookEntryV1,
        MailAddressBookProviderKindV1, UpsertContactFromMailAddressBookEntryCommandV1,
    },
};
use makosh_contacts_runtime::CONTACTS_STORAGE_CAPABILITY_ID_V1;
use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::v1::DurableEnvelopeV1;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use prost_types::Timestamp;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

use crate::identity::device::signer::DeviceSigner;

const RESULT_SUBJECT_V1: &str = "makosh.result.v1.contacts.>";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Contacts binaries"]
fn managed_contacts_command_is_atomic_replayable_and_restart_safe() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-contacts");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_contacts_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            CONTACTS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Contacts logical owner");
    let admitted = admit_contacts_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Contacts Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_contacts_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let contacts = start_contacts_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);
    assert_eq!(contacts.runtime_generation, 1);
    assert!(contacts.grant_epoch > 0);

    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Contacts Event Hub topology")
        .expect("Contacts Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let runtime = tokio::runtime::Runtime::new().expect("Contacts conformance runtime");
    runtime.block_on(async {
        let client = async_nats::connect(&endpoint)
            .await
            .expect("connect Contacts event observer");
        let mut results = client
            .subscribe(RESULT_SUBJECT_V1)
            .await
            .expect("subscribe Contacts results");
        let context = async_nats::jetstream::new(client);
        let now = wall_seconds();

        let created = contact_command([0x31; 16], [0x41; 32], 1, now, now + 300);
        publish(&context, &created).await;
        let created_result = receive_result(&mut results).await;
        assert_eq!(
            created_result
                .contract
                .as_ref()
                .map(|value| value.name.as_str()),
            Some(contact_upserted_contract_reference_v1().name.as_str())
        );
        let created_payload =
            ContactUpsertedFromMailAddressBookEntryV1::decode(created_result.payload.as_slice())
                .expect("decode Contacts success");
        assert_eq!(created_payload.command_id, vec![0x31; 16]);
        assert_eq!(created_payload.contact_revision, 1);
        assert_private_fields_absent(&created_result.encode_to_vec());

        publish(&context, &created).await;
        assert!(
            tokio::time::timeout(Duration::from_secs(1), results.next())
                .await
                .is_err(),
            "exact duplicate must replay persistence without a second terminal event"
        );

        let expired = contact_command([0x32; 16], [0x42; 32], 1, now - 10, now - 5);
        publish(&context, &expired).await;
        let rejected_result = receive_result(&mut results).await;
        assert_eq!(
            rejected_result
                .contract
                .as_ref()
                .map(|value| value.name.as_str()),
            Some(
                contact_upsert_rejected_contract_reference_v1()
                    .name
                    .as_str()
            )
        );
        let rejected = ContactUpsertFromMailAddressBookEntryRejectedV1::decode(
            rejected_result.payload.as_slice(),
        )
        .expect("decode Contacts rejection");
        assert_eq!(rejected.command_id, vec![0x32; 16]);
        assert_ne!(rejected.code, 0);
        assert_private_fields_absent(&rejected_result.encode_to_vec());

        let pool = contacts_admin_pool_v1().await;
        let contacts_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM makosh_data.contacts_state WHERE logical_owner_id=$1",
        )
        .bind(CONTACTS_LOGICAL_HUMAN_OWNER_ID_V1)
        .fetch_one(&pool)
        .await
        .expect("count Contacts state");
        let completed_inbox: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM makosh_data.contacts_mail_entry_inbox
             WHERE logical_owner_id=$1 AND completed=TRUE",
        )
        .bind(CONTACTS_LOGICAL_HUMAN_OWNER_ID_V1)
        .fetch_one(&pool)
        .await
        .expect("count completed Contacts inbox");
        assert_eq!(contacts_count, 1);
        assert_eq!(completed_inbox, 2);
        pool.close().await;
    });

    let previous_generation = contacts.runtime_generation;
    let contacts =
        restart_contacts_runtime_v1(&supervisor, &store, &root.join("runtime"), contacts);
    assert_eq!(contacts.runtime_generation, previous_generation + 1);
    runtime.block_on(async {
        let client = async_nats::connect(&endpoint)
            .await
            .expect("connect restarted Contacts observer");
        let mut results = client
            .subscribe(RESULT_SUBJECT_V1)
            .await
            .expect("subscribe restarted Contacts results");
        let context = async_nats::jetstream::new(client);
        let now = wall_seconds();
        publish(
            &context,
            &contact_command([0x33; 16], [0x43; 32], 2, now, now + 300),
        )
        .await;
        let updated = receive_result(&mut results).await;
        let payload = ContactUpsertedFromMailAddressBookEntryV1::decode(updated.payload.as_slice())
            .expect("decode restarted Contacts result");
        assert_eq!(payload.command_id, vec![0x33; 16]);
        assert_eq!(payload.contact_revision, 2);
        assert_private_fields_absent(&updated.encode_to_vec());
    });

    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    let revoked = transition_registration(
        &owner_runtime_dir,
        &owner_signer,
        &contacts.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(revoked.grant_epoch > contacts.grant_epoch);
    assert_eq!(
        store
            .module_registration(&contacts.registration_id)
            .expect("read revoked Contacts registration")
            .expect("revoked Contacts registration")
            .state(),
        ModuleRegistrationState::Revoked
    );
    assert_eq!(
        store
            .platform_storage_binding(&contacts.registration_id, CONTACTS_STORAGE_CAPABILITY_ID_V1,)
            .expect("read revoked Contacts Storage binding")
            .expect("revoked Contacts Storage binding")
            .state(),
        PlatformStorageBindingStateV1::Revoking
    );
    assert!(
        !supervisor
            .stop_if_active(&contacts.registration_id)
            .expect("observe stopped Contacts runtime"),
        "owner revoke must already stop the Contacts process"
    );

    supervisor
        .shutdown()
        .expect("stop managed Contacts processes");
    shutdown.store(true, Ordering::SeqCst);
    owner_control
        .join()
        .expect("join Contacts owner control server")
        .expect("Contacts owner control server");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Contacts fixture");
    std::fs::remove_dir_all(data).expect("remove Contacts Kernel fixture");
}

fn contact_command(
    command_id: [u8; 16],
    entry_digest: [u8; 32],
    source_revision: u64,
    recorded_at: i64,
    deadline: i64,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    build_upsert_contact_command_outbox_record_v1(
        UpsertContactFromMailAddressBookEntryCommandV1 {
            command_id: command_id.to_vec(),
            logical_owner_id: CONTACTS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            source_account_id: "mail-account-1".to_owned(),
            provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindGmail as i32,
            provider_entry_id: "people/contact-1".to_owned(),
            provider_etag: Some(format!("etag-{source_revision}")),
            display_name: "Private Contact Name".to_owned(),
            email_addresses: vec!["private-contact@example.test".to_owned()],
            phone_numbers: vec!["+12025550123".to_owned()],
            observed_at: Some(Timestamp {
                seconds: recorded_at,
                nanos: 0,
            }),
            source_revision,
            entry_digest: entry_digest.to_vec(),
        },
        deadline,
        &ContactsCommandEnvelopeContextV1 {
            module_id: "mail-contacts-sync".to_owned(),
            runtime_instance_id: "mail-contacts-sync-test-producer".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: recorded_at,
            recorded_at_nanos: 0,
        },
    )
    .expect("build exact Contacts command")
}

async fn publish(
    context: &async_nats::jetstream::Context,
    record: &makosh_events_protocol::delivery::OutboxRecordV1,
) {
    let envelope =
        DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode exact Contacts command");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive Contacts command subject")
        .as_str();
    context
        .publish(subject, record.exact_bytes().to_vec().into())
        .await
        .expect("publish Contacts command")
        .await
        .expect("acknowledge Contacts command");
}

async fn receive_result(subscriber: &mut async_nats::Subscriber) -> DurableEnvelopeV1 {
    let message = tokio::time::timeout(Duration::from_secs(15), subscriber.next())
        .await
        .expect("Contacts terminal result timeout")
        .expect("Contacts terminal result stream");
    DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode Contacts terminal result")
}

fn assert_private_fields_absent(bytes: &[u8]) {
    for private in [
        b"Private Contact Name".as_slice(),
        b"private-contact@example.test".as_slice(),
        b"+12025550123".as_slice(),
        b"people/contact-1".as_slice(),
        b"mail-account-1".as_slice(),
    ] {
        assert!(
            !bytes.windows(private.len()).any(|window| window == private),
            "Contacts terminal event must not expose private provider fields"
        );
    }
}

fn wall_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Contacts wall clock")
        .as_secs()
        .try_into()
        .expect("Contacts wall clock range")
}

async fn contacts_admin_pool_v1() -> sqlx::PgPool {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let options = PgConnectOptions::new()
        .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
        .port(
            required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                .parse()
                .expect("valid PostgreSQL port"),
        )
        .username("makosh_postgres_admin")
        .password(password.as_str())
        .database("makosh_storage_authenticated")
        .ssl_mode(PgSslMode::Disable);
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect Contacts conformance database")
}
