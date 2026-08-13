//! Empty-start provider reconstruction through the admitted Persons contour.

use super::mail_persons_sync_managed_flow::{publish_exact_v1, scheduler_due_v1};
use super::persons_managed_flow::{persons_admin_pool_v1, persons_durable_counts_v1};
use super::*;
use crate::identity::device::signer::DeviceSigner;
use makosh_mail_address_book_contract::{
    MailPersonSourceContractV1, wire_person_source::MailPersonSourceAccountReadyV1,
};
use makosh_mail_api::account_lifecycle::{
    MailAccountLifecycleActionV1, MailAccountLifecycleCommandV1, MailAccountLifecycleStateV1,
    MailCredentialLifecycleStateV1,
};
use makosh_persons_api::{
    PERSONS_CLIENT_CAPABILITY_ID_V1, PERSONS_MODULE_ID_V1, PERSONS_OWNER_ID_V1,
    persons_client_get_profile_contract_reference_v1,
    persons_client_list_directory_contract_reference_v1,
    persons_client_list_source_links_contract_reference_v1,
    wire::{
        PersonCommandRejectedV1, PersonDirectoryResultV1, PersonProfileResultV1,
        PersonSourceLinksResultV1, ReadPersonDirectoryRequestV1, ReadPersonProfileRequestV1,
        ReadPersonSourceLinksRequestV1,
    },
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Scheduler, Mail, Mail Persons Sync and Persons binaries"]
fn empty_start_provider_resync_without_contacts_import() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let provider = MailCardDavFixture::start();
    let root = unique_target_root("makosh-persons-admission-empty-start");
    assert_captured_runtime_diagnostic_privacy_v1();
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_mail_vault(&vault_dir);
    let release = installed_persons_admission_release_v1(&root);
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

    let admitted_mail = admit_mail_carddav_runtime(&store);
    let admitted_persons = admit_persons_runtime_v1(&store);
    let admitted_workflow = admit_mail_persons_sync_runtime_v1(&store);
    record_scheduler_runtime_for_mail_persons_sync(&store);

    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure admission Event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );

    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    let admitted_persons = prepare_persons_runtime_v1(&supervisor, &store, admitted_persons);
    let admitted_workflow =
        prepare_mail_persons_sync_runtime_v1(&supervisor, &store, admitted_workflow);
    issue_initial_scheduler_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &scheduler_binding(&store),
    )
    .expect("provision Scheduler Storage binding");
    configure_communications_jetstream(&store);

    let runtime = tokio::runtime::Runtime::new().expect("Persons admission event harness");
    let _runtime_context = runtime.enter();
    runtime.block_on(assert_legacy_schema_absent_v1());
    assert_eq!(runtime.block_on(persons_durable_counts_v1()).0, 0);
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let (client, mut account_ready) = runtime.block_on(async {
        let client = async_nats::connect(&endpoint)
            .await
            .expect("connect admission observer");
        let account_ready = client
            .subscribe("makosh.observation.v1.mail.mail_person_source_account_ready.v1")
            .await
            .expect("subscribe sanitized account lifecycle");
        client.flush().await.expect("activate admission observer");
        (client, account_ready)
    });

    let persons =
        start_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_persons);
    let workflow = start_mail_persons_sync_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_workflow,
    );
    let scheduler_reservation = managed_launch::load(&supervisor, &store, SCHEDULER_REGISTRATION)
        .expect("load Scheduler reservation");
    assert_eq!(
        scheduler_launch::start_from_reservation(
            &supervisor,
            &store,
            release.kernel(),
            &root.join("runtime"),
            scheduler_reservation,
            &scheduler_binding(&store),
        )
        .expect("start Scheduler for provider reconstruction"),
        1
    );
    let mut mail = start_mail_carddav_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        MailCardDavFixtureSettingsV1 {
            imap_port: 19_993,
            carddav_port: provider.port(),
            ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);

    let account_public_id = runtime.block_on(async {
        let message = tokio::time::timeout(Duration::from_secs(15), account_ready.next())
            .await
            .expect("sanitized account lifecycle timeout")
            .expect("sanitized account lifecycle delivery");
        let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
            .expect("decode sanitized account lifecycle envelope");
        let ready = MailPersonSourceAccountReadyV1::decode(envelope.payload.as_slice())
            .expect("decode sanitized account lifecycle payload");
        assert_eq!(ready.logical_owner_id, "owner-1");
        ready
            .account_public_id
            .try_into()
            .expect("public account identity")
    });
    if let Err(detail) = runtime.block_on(wait_for_provider_reconstruction_v1(&provider, false, 1))
    {
        panic!(
            "{detail}; Mail active={} last_failure={:?}",
            supervisor
                .is_active(&mail.registration_id)
                .expect("read Mail activity"),
            supervisor
                .last_failure(&mail.registration_id)
                .expect("read Mail failure"),
        );
    }
    runtime.block_on(wait_for_workflow_account_idle_v1(account_public_id));
    runtime.block_on(wait_for_scheduler_idle_v1());
    assert_eq!(runtime.block_on(persons_durable_counts_v1()).0, 1);
    let (profile_person_id, profile_revision, source_link_count) =
        route_persons_snapshot_v1(&store, &supervisor, &persons);
    assert_eq!(source_link_count, 1);

    let replay_command = runtime.block_on(latest_mail_fetch_command_v1());
    let replay_state = runtime.block_on(mail_fetch_consumer_state_v1(&store));
    let replay_identity = runtime.block_on(persons_reconstruction_identity_v1());
    let reports_before_replay = provider.reports();
    runtime.block_on(publish_exact_v1(&client, &replay_command));
    runtime.block_on(wait_for_mail_fetch_ack_v1(&store, replay_state.0));
    assert_eq!(
        provider.reports(),
        reports_before_replay,
        "exact durable fetch replay must not call the provider"
    );
    assert_eq!(
        runtime.block_on(persons_reconstruction_identity_v1()),
        replay_identity
    );

    // A provider-unchanged run is a real reconstruction replay: it may add
    // workflow receipts, but it must not create another Person/source or
    // mutate the canonical identity. Its exact durable counts survive restart.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_millis() as i64;
    runtime.block_on(publish_exact_v1(
        &client,
        &scheduler_due_v1(now, [0x91; 16], account_public_id),
    ));
    if let Err(detail) = runtime.block_on(wait_for_provider_reconstruction_v1(&provider, false, 2))
    {
        panic!("{detail}");
    }
    runtime.block_on(wait_for_workflow_account_idle_v1(account_public_id));
    runtime.block_on(wait_for_scheduler_idle_v1());
    let unchanged_durable = runtime.block_on(persons_reconstruction_identity_v1());
    let persons = restart_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), persons);
    assert_eq!(persons.runtime_generation, 2);
    assert_eq!(
        runtime.block_on(persons_reconstruction_identity_v1()),
        unchanged_durable
    );
    let unchanged_snapshot = route_persons_snapshot_v1(&store, &supervisor, &persons);
    assert_eq!(unchanged_snapshot, (profile_person_id, profile_revision, 1));

    provider.remove_source();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_millis() as i64;
    runtime.block_on(publish_exact_v1(
        &client,
        &scheduler_due_v1(now, [0x92; 16], account_public_id),
    ));
    assert!(
        runtime.block_on(wait_for_workflow_run_started_v1([0x92; 16])),
        "second scheduled run was not consumed: active={} ready={} last_failure={:?}",
        supervisor
            .is_active(&workflow.registration_id)
            .expect("read workflow activity"),
        supervisor
            .relay_port()
            .is_ready(&workflow.registration_id)
            .expect("read workflow readiness"),
        supervisor
            .last_failure(&workflow.registration_id)
            .expect("read workflow failure"),
    );
    assert_ne!(
        runtime.block_on(workflow_run_state_v1([0x92; 16])),
        Some(4),
        "second scheduled run must not be bounded as AccountBusy after both workflow and Scheduler are idle",
    );
    if let Err(detail) = runtime.block_on(wait_for_provider_reconstruction_v1(&provider, true, 3)) {
        panic!(
            "{detail}; Mail active={} last_failure={:?}",
            supervisor
                .is_active(&mail.registration_id)
                .expect("read Mail activity"),
            supervisor
                .last_failure(&mail.registration_id)
                .expect("read Mail failure"),
        );
    }
    assert_eq!(runtime.block_on(persons_durable_counts_v1()).0, 1);

    let retained = runtime.block_on(persons_retained_after_source_removal_v1());
    assert_eq!(retained, (1, 1));
    let durable_before_restart = runtime.block_on(persons_reconstruction_identity_v1());
    let persons = restart_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), persons);
    assert_eq!(persons.runtime_generation, 3);
    assert!(
        supervisor
            .is_active(&workflow.registration_id)
            .expect("read workflow state after Persons successor")
    );
    assert_eq!(
        runtime.block_on(persons_retained_after_source_removal_v1()),
        retained
    );
    assert_eq!(
        runtime.block_on(persons_reconstruction_identity_v1()),
        durable_before_restart
    );
    let (restarted_person_id, restarted_revision, restarted_source_count) =
        route_persons_snapshot_v1(&store, &supervisor, &persons);
    assert_eq!(restarted_person_id, profile_person_id);
    assert!(restarted_revision >= profile_revision);
    assert_eq!(
        restarted_source_count, 0,
        "removed links stay durable but leave the active client view"
    );

    // Crash boundary: the canonical Mail lifecycle commit is durable, but the
    // separate sanitized Retired outbox has not happened yet. A successor must
    // reconcile that exact completed receipt before ready, drive Cancel, and
    // replay without duplicate public lifecycle/control records.
    assert!(
        supervisor
            .stop_if_active(&mail.registration_id)
            .expect("stop Mail at lifecycle crash boundary")
    );
    runtime.block_on(async {
        let durable = super::mail_event_flow::connect_postgres().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("lifecycle clock")
            .as_secs() as i64;
        let begin = durable
            .begin_account_lifecycle(
                &MailAccountLifecycleCommandV1 {
                    operation_id: "task6-crash-retire".to_owned(),
                    connection_id: MAIL_ACCOUNT_ID.to_owned(),
                    expected_lifecycle_revision: 0,
                },
                MailAccountLifecycleActionV1::Retire,
                MAIL_ACCOUNT_ID,
                now,
            )
            .await
            .expect("commit canonical lifecycle before simulated crash");
        let mut receipt = begin.receipt;
        for credential in receipt.credentials.clone() {
            receipt = durable
                .record_account_lifecycle_progress(
                    MAIL_ACCOUNT_ID,
                    "task6-crash-retire",
                    credential.purpose,
                    MailCredentialLifecycleStateV1::Completed,
                    now,
                )
                .await
                .expect("complete lifecycle credential before simulated crash");
        }
        assert_eq!(receipt.state, MailAccountLifecycleStateV1::Completed);
        let pool = persons_admin_pool_v1().await;
        let retired: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_lifecycle_outbox \
             WHERE logical_owner_id=$1 AND semantic_kind=2",
        )
        .bind("owner-1")
        .fetch_one(&pool)
        .await
        .expect("read pre-reconciliation Retired outbox");
        pool.close().await;
        assert_eq!(retired, 0, "simulated crash gap must be observable");
    });
    mail = restart_mail_carddav_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        mail,
        MailCardDavFixtureSettingsV1 {
            imap_port: 19_993,
            carddav_port: provider.port(),
            ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);
    let retired_cancel = runtime
        .block_on(wait_for_retired_cancel_v1())
        .unwrap_or_else(|state| {
            panic!(
                "Retired reconciliation did not reach Cancel: state={state:?} Mail active={} failure={:?} workflow active={} failure={:?}",
                supervisor
                    .is_active(&mail.registration_id)
                    .expect("read Mail activity after reconciliation"),
                supervisor
                    .last_failure(&mail.registration_id)
                    .expect("read Mail reconciliation failure"),
                supervisor
                    .is_active(&workflow.registration_id)
                    .expect("read workflow activity after reconciliation"),
                supervisor
                    .last_failure(&workflow.registration_id)
                    .expect("read workflow reconciliation failure"),
            )
        });
    assert_eq!(retired_cancel, [1, 1, 1, 1, 1]);
    mail = restart_mail_carddav_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        mail,
        MailCardDavFixtureSettingsV1 {
            imap_port: 19_993,
            carddav_port: provider.port(),
            ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
        },
    );
    wait_for_mail_ready(&supervisor, &mail);
    assert_eq!(
        runtime
            .block_on(wait_for_retired_cancel_v1())
            .expect("replayed Retired reaches exact Cancel state"),
        retired_cancel,
        "completed lifecycle reconciliation must replay without duplicate Retired or Cancel",
    );
    runtime.block_on(assert_private_sentinels_absent_from_public_surfaces_v1());
    for registration_id in [
        &mail.registration_id,
        &workflow.registration_id,
        &persons.registration_id,
    ] {
        let failure = supervisor
            .last_failure(registration_id)
            .expect("read managed runtime failure state");
        let diagnostic = format!("{failure:?}");
        assert_private_sentinels_absent_v1(diagnostic.as_bytes(), "managed runtime error");
        assert!(
            failure.is_none(),
            "admitted runtime ended with {diagnostic}"
        );
    }

    supervisor
        .shutdown()
        .expect("stop Persons admission contour");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove admission fixture");
    std::fs::remove_dir_all(data).expect("remove admission Kernel fixture");
}

fn assert_captured_runtime_diagnostic_privacy_v1() {
    for executable in [
        binary("MAKOSH_MAIL_RUNTIME_BIN"),
        binary("MAKOSH_MAIL_PERSONS_SYNC_RUNTIME_BIN"),
    ] {
        let output = std::process::Command::new(&executable)
            .arg("managed-1.vcf")
            .output()
            .expect("capture actual runtime diagnostic");
        assert!(
            !output.status.success(),
            "private provider sentinel must exercise a rejected actual-binary contour"
        );
        assert!(
            !output.stderr.is_empty(),
            "rejected actual runtime must emit a captured bounded diagnostic"
        );
        assert_private_sentinels_absent_v1(&output.stderr, "captured runtime stderr");
        assert_private_sentinels_absent_v1(&output.stdout, "captured runtime stdout");
    }
}

fn route_persons_snapshot_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    persons: &StartedPersonsRuntimeV1,
) -> ([u8; 16], u64, usize) {
    let route = |request_id, contract, payload: Vec<u8>| {
        let request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: PERSONS_MODULE_ID_V1.to_owned(),
            owner_id: PERSONS_OWNER_ID_V1.to_owned(),
            contract: Some(contract),
            request_id,
            request_payload: payload,
            logical_owner_id: "owner-1".to_owned(),
            authenticated_device_id: "desktop-1".to_owned(),
            authenticated_client_session_id: "session-1".to_owned(),
        }
        .encode_to_vec();
        let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
            &persons.registration_id,
            &persons.runtime_instance_id,
            persons.runtime_generation,
            persons.grant_epoch,
            PERSONS_CLIENT_CAPABILITY_ID_V1,
            &request,
        );
        let bytes = crate::modules::capability::router::route_managed_client_request(
            store,
            &supervisor.relay_port(),
            &route,
        )
        .expect("route authenticated Persons client request");
        let response = ModuleClientResponseV1::decode(bytes.as_slice()).expect("Persons response");
        assert!(response.error_code.is_empty(), "{}", response.error_code);
        assert_private_sentinels_absent_v1(&response.response_payload, "Persons client payload");
        response.response_payload
    };
    let directory = PersonDirectoryResultV1::decode(
        route(
            1,
            persons_client_list_directory_contract_reference_v1(),
            ReadPersonDirectoryRequestV1 {
                logical_owner_id: String::new(),
                after_person_id: Vec::new(),
                limit: 1,
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("directory");
    assert_eq!(directory.persons.len(), 1);
    assert!(directory.next_after_person_id.is_empty());
    let person_id: [u8; 16] = directory.persons[0]
        .person_id
        .clone()
        .try_into()
        .expect("person id");
    let profile = PersonProfileResultV1::decode(
        route(
            2,
            persons_client_get_profile_contract_reference_v1(),
            ReadPersonProfileRequestV1 {
                logical_owner_id: String::new(),
                person_id: person_id.to_vec(),
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("profile");
    let links = PersonSourceLinksResultV1::decode(
        route(
            3,
            persons_client_list_source_links_contract_reference_v1(),
            ReadPersonSourceLinksRequestV1 {
                logical_owner_id: String::new(),
                person_id: person_id.to_vec(),
                after_source_link_id: Vec::new(),
                limit: 1,
            }
            .encode_to_vec(),
        )
        .as_slice(),
    )
    .expect("source links");
    (person_id, profile.person_revision, links.source_links.len())
}

fn installed_persons_admission_release_v1(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(mail_release_artifact());
    artifacts.push(persons_release_artifact_v1());
    artifacts.push(mail_persons_sync_release_artifact_v1());
    artifacts.push(scheduler_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Persons admission release")
}

async fn workflow_run_state_v1(run_id: [u8; 16]) -> Option<i16> {
    let pool = persons_admin_pool_v1().await;
    let state = sqlx::query_scalar(
        "SELECT state FROM makosh_data.mail_persons_sync_runs \
         WHERE logical_owner_id=$1 AND run_id=$2",
    )
    .bind("owner-1")
    .bind(run_id.as_slice())
    .fetch_optional(&pool)
    .await
    .expect("read provider reconstruction run state");
    pool.close().await;
    state
}

async fn latest_mail_fetch_command_v1() -> Vec<u8> {
    let pool = persons_admin_pool_v1().await;
    let bytes = sqlx::query_scalar(
        "SELECT envelope_bytes FROM makosh_data.mail_address_book_person_source_fetch_inbox \
         WHERE logical_owner_id=$1 ORDER BY processed_at_unix_millis DESC,command_id DESC LIMIT 1",
    )
    .bind("owner-1")
    .fetch_one(&pool)
    .await
    .expect("read exact stored Mail fetch command");
    pool.close().await;
    bytes
}

fn mail_fetch_consumer_name_v1(store: &SqliteControlStore) -> String {
    let topology = store
        .platform_event_hub_topology()
        .expect("read Mail Event Hub topology")
        .expect("Mail Event Hub topology");
    let contracts = event_catalog::resolve_contracts(store).expect("resolve Event contracts");
    let plan = event_topology::plan(&contracts, &topology).expect("plan Event topology");
    let expected = MailPersonSourceContractV1::FetchPageCommand.reference();
    plan.consumers()
        .iter()
        .find(|consumer| consumer.contract() == &expected)
        .expect("Mail fetch consumer")
        .durable_name()
        .to_owned()
}

async fn mail_fetch_consumer_state_v1(store: &SqliteControlStore) -> (u64, usize) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Mail Event Hub topology")
        .expect("Mail Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let context = async_nats::jetstream::new(
        async_nats::connect(endpoint)
            .await
            .expect("connect Mail fetch observer"),
    );
    let stream = context
        .get_stream("MAKOSH_COMMAND_V1")
        .await
        .expect("read command stream");
    let mut consumer: async_nats::jetstream::consumer::PullConsumer = stream
        .get_consumer(&mail_fetch_consumer_name_v1(store))
        .await
        .expect("read Mail fetch consumer");
    let info = consumer.info().await.expect("refresh Mail fetch consumer");
    (info.ack_floor.consumer_sequence, info.num_ack_pending)
}

async fn wait_for_mail_fetch_ack_v1(store: &SqliteControlStore, previous_ack_floor: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let state = mail_fetch_consumer_state_v1(store).await;
        if state.0 > previous_ack_floor && state.1 == 0 {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "exact Mail fetch replay was not acknowledged: state={state:?}",
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_scheduler_idle_v1() {
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        let pool = persons_admin_pool_v1().await;
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_platform.scheduler_runs \
             WHERE state IN ('pending_dispatch','dispatched','running','retry_wait')",
        )
        .fetch_one(&pool)
        .await
        .expect("count active Scheduler runs");
        pool.close().await;
        if active == 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Scheduler did not terminalize provider reconstruction: active={active}",
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_retired_cancel_v1() -> Result<[i64; 5], [i64; 5]> {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        let pool = persons_admin_pool_v1().await;
        let state = [
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_lifecycle_outbox \
                 WHERE logical_owner_id=$1 AND semantic_kind=2",
            )
            .bind("owner-1")
            .fetch_one(&pool)
            .await
            .expect("count durable Retired lifecycle"),
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_account_inbox \
                 WHERE logical_owner_id=$1 AND semantic_kind=2",
            )
            .bind("owner-1")
            .fetch_one(&pool)
            .await
            .expect("count consumed Retired lifecycle"),
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_schedule_control_outbox \
                 WHERE logical_owner_id=$1 AND semantic_kind=2",
            )
            .bind("owner-1")
            .fetch_one(&pool)
            .await
            .expect("count exact Cancel controls"),
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_account_bindings \
                 WHERE logical_owner_id=$1 AND state=2",
            )
            .bind("owner-1")
            .fetch_one(&pool)
            .await
            .expect("count retired workflow account binding"),
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_lifecycle_outbox \
                 WHERE logical_owner_id=$1 AND semantic_kind=2 AND published_at_unix_millis IS NOT NULL",
            )
            .bind("owner-1")
            .fetch_one(&pool)
            .await
            .expect("count published Retired lifecycle"),
        ];
        pool.close().await;
        if state == [1, 1, 1, 1, 1] {
            return Ok(state);
        }
        if std::time::Instant::now() >= deadline {
            return Err(state);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_workflow_run_started_v1(run_id: [u8; 16]) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let pool = persons_admin_pool_v1().await;
        let present: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM makosh_data.mail_persons_sync_runs \
             WHERE logical_owner_id=$1 AND run_id=$2)",
        )
        .bind("owner-1")
        .bind(run_id.as_slice())
        .fetch_one(&pool)
        .await
        .expect("read scheduled provider reconstruction run");
        pool.close().await;
        if present {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_workflow_account_idle_v1(account_public_id: [u8; 16]) {
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        let pool = persons_admin_pool_v1().await;
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_runs \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND state IN (1,2)",
        )
        .bind("owner-1")
        .bind(account_public_id.as_slice())
        .fetch_one(&pool)
        .await
        .expect("count active provider reconstruction runs");
        pool.close().await;
        if active == 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "provider reconstruction run did not terminalize: active={active}",
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_provider_reconstruction_v1(
    provider: &MailCardDavFixture,
    removed: bool,
    expected_reports: usize,
) -> Result<(), String> {
    let reports_deadline = std::time::Instant::now() + Duration::from_secs(60);
    while provider.reports() < expected_reports {
        if std::time::Instant::now() >= reports_deadline {
            return Err(format!(
                "provider REPORT did not reach {expected_reports}: actual={} stages={:?}",
                provider.reports(),
                provider_reconstruction_stages_v1().await,
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let expected_state = (1, i64::from(removed));
    let state_deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        let state = persons_retained_after_source_removal_v1().await;
        if state == expected_state {
            return Ok(());
        }
        if std::time::Instant::now() >= state_deadline {
            return Err(format!(
                "provider reconstruction did not reach the expected durable state: reports={} state={state:?} stages={:?} persons_reject_code={} second_fetch_deadline_delta_ms={}",
                provider.reports(),
                provider_reconstruction_stages_v1().await,
                persons_reject_code_v1().await,
                second_fetch_deadline_delta_millis_v1().await,
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn provider_reconstruction_stages_v1() -> [i64; 23] {
    let pool = persons_admin_pool_v1().await;
    let values = [
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_account_bindings")
            .fetch_one(&pool).await.expect("count workflow account bindings"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_schedule_control_outbox")
            .fetch_one(&pool).await.expect("count workflow schedule controls"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_schedule_control_outbox WHERE published_at_unix_millis IS NULL")
            .fetch_one(&pool).await.expect("count pending workflow schedule controls"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_platform.scheduler_schedule_control_inbox")
            .fetch_one(&pool).await.expect("count Scheduler controls"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_platform.scheduler_schedules")
            .fetch_one(&pool).await.expect("count Scheduler schedules"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_platform.scheduler_dispatches")
            .fetch_one(&pool).await.expect("count Scheduler dispatches"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_runs")
            .fetch_one(&pool).await.expect("count workflow runs"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_fetch_inbox")
            .fetch_one(&pool).await.expect("count Mail fetch inbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_fetch_outbox")
            .fetch_one(&pool).await.expect("count Mail source outbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_fetch_outbox WHERE published_at_unix_millis IS NULL")
            .fetch_one(&pool).await.expect("count pending Mail source outbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_pages")
            .fetch_one(&pool).await.expect("count workflow pages"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_sources")
            .fetch_one(&pool).await.expect("count workflow sources"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_inbox")
            .fetch_one(&pool).await.expect("count workflow inbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_outbox")
            .fetch_one(&pool).await.expect("count workflow outbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_outbox WHERE published_at_unix_millis IS NULL")
            .fetch_one(&pool).await.expect("count pending workflow outbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.persons_command_inbox")
            .fetch_one(&pool).await.expect("count Persons command inbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.persons_outbox")
            .fetch_one(&pool).await.expect("count Persons outbox"),
        sqlx::query_scalar("SELECT COALESCE(MAX(outcome),0)::bigint FROM makosh_data.mail_persons_sync_sources")
            .fetch_one(&pool).await.expect("read workflow source outcome"),
        sqlx::query_scalar("SELECT COALESCE(MAX(state),0)::bigint FROM makosh_data.mail_persons_sync_runs WHERE run_id=decode(repeat('92',16),'hex')")
            .fetch_one(&pool).await.expect("read second workflow run state"),
        sqlx::query_scalar("SELECT COALESCE(MAX(state),0)::bigint FROM makosh_data.mail_address_book_person_source_runs")
            .fetch_one(&pool).await.expect("read Mail source run state"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_runs WHERE terminal_snapshot_succeeded")
            .fetch_one(&pool).await.expect("count terminal Mail snapshots"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_seen")
            .fetch_one(&pool).await.expect("count Mail snapshot seen sources"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_address_book_person_sources WHERE active")
            .fetch_one(&pool).await.expect("count active Mail public sources"),
    ];
    pool.close().await;
    values
}

async fn second_fetch_deadline_delta_millis_v1() -> i64 {
    let pool = persons_admin_pool_v1().await;
    let bytes: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT envelope_bytes FROM makosh_data.mail_persons_sync_outbox \
         WHERE run_id=decode(repeat('92',16),'hex') AND semantic_kind=2 LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .expect("read second Mail fetch envelope");
    pool.close().await;
    let Some(bytes) = bytes else {
        return i64::MIN;
    };
    let envelope = DurableEnvelopeV1::decode(bytes.as_slice()).expect("decode second Mail fetch");
    let Some(makosh_events_protocol::v1::durable_envelope_v1::Semantics::Command(command)) =
        envelope.semantics
    else {
        return i64::MIN + 1;
    };
    let deadline = command.deadline.expect("second Mail fetch deadline");
    let deadline_millis = deadline.seconds * 1_000 + i64::from(deadline.nanos / 1_000_000);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_millis() as i64;
    deadline_millis - now
}

async fn persons_reject_code_v1() -> i32 {
    let pool = persons_admin_pool_v1().await;
    let terminal: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT terminal_envelope_bytes FROM makosh_data.persons_command_inbox \
         WHERE completed ORDER BY completed_at_unix_millis DESC LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .expect("read Persons terminal")
    .flatten();
    pool.close().await;
    let Some(terminal) = terminal else {
        return -1;
    };
    let envelope = DurableEnvelopeV1::decode(terminal.as_slice()).expect("decode Persons terminal");
    PersonCommandRejectedV1::decode(envelope.payload.as_slice())
        .map(|value| value.code)
        .unwrap_or(0)
}

async fn persons_retained_after_source_removal_v1() -> (i64, i64) {
    let pool = persons_admin_pool_v1().await;
    let persons = sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_data.persons_current WHERE logical_owner_id=$1",
    )
    .bind("owner-1")
    .fetch_one(&pool)
    .await
    .expect("count durable Persons");
    let removed_sources = sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_data.persons_sources WHERE logical_owner_id=$1 AND removed",
    )
    .bind("owner-1")
    .fetch_one(&pool)
    .await
    .expect("count removed public sources");
    pool.close().await;
    (persons, removed_sources)
}

async fn assert_legacy_schema_absent_v1() {
    let pool = persons_admin_pool_v1().await;
    let retired_table_scan = concat!(
        "SELECT COUNT(*) FROM information_schema.tables ",
        "WHERE table_schema='makosh_data' AND ",
        "(table_name LIKE 'contacts_%' OR table_name LIKE 'mail_",
        "contacts_sync_%')",
    );
    let legacy: i64 = sqlx::query_scalar(retired_table_scan)
        .fetch_one(&pool)
        .await
        .expect("scan installed schema for retired Contacts tables");
    pool.close().await;
    assert_eq!(legacy, 0, "retired Contacts schema must not be installed");
}

async fn persons_reconstruction_identity_v1() -> (Vec<u8>, Vec<u8>, i64, i64, i64, i64) {
    let pool = persons_admin_pool_v1().await;
    let account: Vec<u8> = sqlx::query_scalar(
        "SELECT account_public_id FROM makosh_data.mail_address_book_person_source_accounts \
         WHERE logical_owner_id=$1 ORDER BY account_public_id LIMIT 1",
    )
    .bind("owner-1")
    .fetch_one(&pool)
    .await
    .expect("read stable public account id");
    let person: Vec<u8> = sqlx::query_scalar(
        "SELECT person_id FROM makosh_data.persons_current WHERE logical_owner_id=$1 ORDER BY person_id LIMIT 1",
    )
    .bind("owner-1")
    .fetch_one(&pool)
    .await
    .expect("read stable Person id");
    let fetch_inbox = sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_fetch_inbox",
    )
    .fetch_one(&pool)
    .await
    .expect("count Mail fetch inbox");
    let fetch_outbox = sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_data.mail_address_book_person_source_fetch_outbox",
    )
    .fetch_one(&pool)
    .await
    .expect("count Mail fetch outbox");
    let persons_inbox =
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.persons_command_inbox")
            .fetch_one(&pool)
            .await
            .expect("count Persons inbox");
    let persons_outbox = sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.persons_outbox")
        .fetch_one(&pool)
        .await
        .expect("count Persons outbox");
    pool.close().await;
    (
        account,
        person,
        fetch_inbox,
        fetch_outbox,
        persons_inbox,
        persons_outbox,
    )
}

async fn assert_private_sentinels_absent_from_public_surfaces_v1() {
    let pool = persons_admin_pool_v1().await;
    let rows: Vec<Vec<u8>> = sqlx::query_scalar(
        "SELECT envelope_bytes FROM makosh_data.mail_address_book_person_source_lifecycle_outbox \
         UNION ALL SELECT envelope_bytes FROM makosh_data.mail_address_book_person_source_fetch_outbox \
         UNION ALL SELECT envelope_bytes FROM makosh_data.mail_persons_sync_outbox \
         UNION ALL SELECT terminal_envelope_bytes FROM makosh_data.persons_command_inbox \
           WHERE terminal_envelope_bytes IS NOT NULL",
    )
    .fetch_all(&pool)
    .await
    .expect("load public durable envelopes for privacy scan");
    pool.close().await;
    for bytes in rows {
        assert_private_sentinels_absent_v1(&bytes, "public durable envelope");
    }
}

fn assert_private_sentinels_absent_v1(bytes: &[u8], surface: &str) {
    for sentinel in [
        b"managed-1.vcf".as_slice(),
        b"carddav-etag-1".as_slice(),
        b"BEGIN:VCARD".as_slice(),
        b"managed-mail-carddav-password".as_slice(),
    ] {
        assert!(
            !bytes
                .windows(sentinel.len())
                .any(|window| window == sentinel),
            "private provider sentinel escaped into {surface}",
        );
    }
}
