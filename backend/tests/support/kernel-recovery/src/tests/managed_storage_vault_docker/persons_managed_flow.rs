//! Live signed Persons command consumption over Vault, Storage and NATS.

use super::*;

use async_nats::jetstream::consumer::IntoConsumerConfig;
use futures_util::StreamExt;
use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::v1::DurableEnvelopeV1;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_persons_api::{
    persons_command_contract_reference_v1, persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1,
    wire::{
        ManualCreatePersonCommandV1, PersonCommandRejectedV1, PersonCommandSucceededV1,
        PersonProfileV1, PersonRevisionV1, PersonsCommandV1, TimestampV1,
        persons_command_v1::Command,
    },
};
use makosh_persons_runtime::{
    PERSONS_STORAGE_CAPABILITY_ID_V1,
    transport::{
        PersonsEnvelopeContextV1, build_persons_command_outbox_record_v1,
        build_persons_command_succeeded_outbox_record_v1,
    },
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

use crate::identity::device::signer::DeviceSigner;

const RESULT_SUBJECT_V1: &str = "makosh.result.v1.persons.>";
const EVENT_HUB_MAX_PULL_WAITING_V1: i64 = 512;

struct PersonsConsumerFixtureV1 {
    durable_name: String,
    subject: String,
    ack_wait: Duration,
    max_deliver: i64,
    max_ack_pending: i64,
}

#[derive(Clone, Copy)]
enum PersonsConsumerDriftV1 {
    DeliverPolicyNew,
    RetryBackoff,
    HeadersOnly,
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Persons binaries"]
fn managed_persons_bootstrap_is_control_responsive_and_requires_exact_consumer() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-persons-bootstrap");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_persons_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            PERSONS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Persons logical owner");
    let admitted = admit_persons_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Persons Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_persons_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let runtime = tokio::runtime::Runtime::new().expect("Persons bootstrap runtime");
    let unchanged = runtime.block_on(persons_durable_counts_v1());

    let persons = launch_persons_runtime_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted,
        PersonsBootstrapOverrideV1::StopVaultAfterConfiguration,
    );
    assert_pre_ready_stop_v1(&supervisor, &persons, "Vault bootstrap");
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    supervisor
        .stop("storage")
        .expect("stop Storage before Vault successor");
    assert_eq!(start_vault(&supervisor, &store, &data, release.kernel()), 2);
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory(),
        ),
        2
    );
    let stalled_listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind stalled Storage endpoint");
    let stalled_port = stalled_listener
        .local_addr()
        .expect("stalled Storage address")
        .port();
    let persons = launch_persons_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        persons,
        PersonsBootstrapOverrideV1::UnavailableStoragePort(stalled_port),
    );
    assert_pre_ready_stop_v1(&supervisor, &persons, "Storage bootstrap");
    drop(stalled_listener);
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    super::nats_outage_fixture::set_authenticated_nats_container_running(false);
    let persons = launch_persons_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        persons,
        PersonsBootstrapOverrideV1::None,
    );
    assert_pre_ready_stop_v1(&supervisor, &persons, "NATS bootstrap");
    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    runtime.block_on(delete_persons_command_consumer_v1(&store));
    let persons = launch_persons_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        persons,
        PersonsBootstrapOverrideV1::None,
    );
    assert_pre_ready_stop_v1(&supervisor, &persons, "missing consumer topology");
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    configure_communications_jetstream(&store);
    runtime.block_on(install_drifted_persons_command_consumer_v1(
        &store,
        PersonsConsumerDriftV1::DeliverPolicyNew,
    ));
    let persons = launch_persons_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        persons,
        PersonsBootstrapOverrideV1::None,
    );
    assert_topology_never_ready_before_stop_v1(
        &supervisor,
        &persons,
        "deliver-policy consumer drift",
    );
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    runtime.block_on(delete_persons_command_consumer_v1(&store));
    configure_communications_jetstream(&store);
    runtime.block_on(install_drifted_persons_command_consumer_v1(
        &store,
        PersonsConsumerDriftV1::RetryBackoff,
    ));
    let persons = launch_persons_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        persons,
        PersonsBootstrapOverrideV1::None,
    );
    assert_topology_never_ready_before_stop_v1(
        &supervisor,
        &persons,
        "retry-backoff consumer drift",
    );
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    runtime.block_on(delete_persons_command_consumer_v1(&store));
    configure_communications_jetstream(&store);
    runtime.block_on(install_drifted_persons_command_consumer_v1(
        &store,
        PersonsConsumerDriftV1::HeadersOnly,
    ));
    let persons = launch_persons_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        persons,
        PersonsBootstrapOverrideV1::None,
    );
    assert_topology_never_ready_before_stop_v1(
        &supervisor,
        &persons,
        "headers-only consumer drift",
    );
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    runtime.block_on(delete_persons_command_consumer_v1(&store));
    configure_communications_jetstream(&store);
    runtime.block_on(assert_persons_command_consumer_exact_v1(&store));
    let persons = restart_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), persons);
    assert!(
        supervisor
            .relay_port()
            .is_ready(&persons.registration_id)
            .expect("read recovered Persons readiness")
    );
    assert_pre_ready_or_ready_stop_v1(&supervisor, &persons, "recovered bootstrap");
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), unchanged);

    supervisor
        .shutdown()
        .expect("stop managed Persons bootstrap dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Persons bootstrap fixture");
    std::fs::remove_dir_all(data).expect("remove Persons bootstrap Kernel fixture");
}

fn assert_pre_ready_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    persons: &StartedPersonsRuntimeV1,
    phase: &str,
) {
    std::thread::sleep(Duration::from_millis(250));
    assert!(
        supervisor
            .is_active(&persons.registration_id)
            .unwrap_or_else(|error| panic!("{phase} activity: {error}")),
        "{phase} must remain active until the Kernel closes control"
    );
    assert!(
        !supervisor
            .relay_port()
            .is_ready(&persons.registration_id)
            .unwrap_or_else(|error| panic!("{phase} readiness: {error}")),
        "{phase} must not signal ready"
    );
    assert_pre_ready_or_ready_stop_v1(supervisor, persons, phase);
}

fn assert_topology_never_ready_before_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    persons: &StartedPersonsRuntimeV1,
    phase: &str,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        assert!(
            !supervisor
                .relay_port()
                .is_ready(&persons.registration_id)
                .unwrap_or_else(|error| panic!("{phase} readiness: {error}")),
            "{phase} must never signal ready"
        );
        if !supervisor
            .is_active(&persons.registration_id)
            .unwrap_or_else(|error| panic!("{phase} activity: {error}"))
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_pre_ready_or_ready_stop_v1(supervisor, persons, phase);
}

fn assert_pre_ready_or_ready_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    persons: &StartedPersonsRuntimeV1,
    phase: &str,
) {
    let started = std::time::Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(&persons.registration_id)
            .unwrap_or_else(|error| panic!("{phase} request stop: {error}"))
    );
    assert!(
        supervisor
            .stop_if_active(&persons.registration_id)
            .unwrap_or_else(|error| panic!("{phase} join stop: {error}"))
    );
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "{phase} control close must be terminal and prompt"
    );
    assert_eq!(
        supervisor
            .last_failure(&persons.registration_id)
            .unwrap_or_else(|error| panic!("{phase} failure state: {error}")),
        None,
        "{phase} requested stop must not start a replacement attempt"
    );
}

fn persons_consumer_fixture_v1(store: &SqliteControlStore) -> PersonsConsumerFixtureV1 {
    let configuration = store
        .platform_event_hub_topology()
        .expect("read Persons Event Hub topology")
        .expect("Persons Event Hub topology");
    let contracts = event_catalog::resolve_contracts(store).expect("resolve Persons contracts");
    let plan = event_topology::plan(&contracts, &configuration).expect("plan Persons topology");
    let expected = persons_command_contract_reference_v1();
    let consumer = plan
        .consumers()
        .iter()
        .find(|consumer| {
            consumer.contract().owner == expected.owner
                && consumer.contract().name == expected.name
                && consumer.contract().major == expected.major
                && consumer.contract().revision == expected.revision
        })
        .expect("exact Persons command consumer");
    PersonsConsumerFixtureV1 {
        durable_name: consumer.durable_name().to_owned(),
        subject: consumer.subject().as_str(),
        ack_wait: Duration::from_millis(consumer.delivery_policy().ack_wait_millis().into()),
        max_deliver: i64::from(consumer.delivery_policy().max_deliver()),
        max_ack_pending: i64::from(consumer.max_in_flight()),
    }
}

async fn delete_persons_command_consumer_v1(store: &SqliteControlStore) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Persons Event Hub topology")
        .expect("Persons Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let consumer = persons_consumer_fixture_v1(store);
    async_nats::jetstream::new(
        async_nats::connect(endpoint)
            .await
            .expect("connect Persons topology fixture"),
    )
    .delete_consumer_from_stream(consumer.durable_name, "MAKOSH_COMMAND_V1")
    .await
    .expect("delete exact Persons command consumer");
}

async fn install_drifted_persons_command_consumer_v1(
    store: &SqliteControlStore,
    drift: PersonsConsumerDriftV1,
) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Persons Event Hub topology")
        .expect("Persons Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let consumer = persons_consumer_fixture_v1(store);
    let context = async_nats::jetstream::new(
        async_nats::connect(endpoint)
            .await
            .expect("connect Persons topology fixture"),
    );
    context
        .delete_consumer_from_stream(&consumer.durable_name, "MAKOSH_COMMAND_V1")
        .await
        .expect("delete exact Persons command consumer before drift");
    let mut backoff = persons_consumer_retry_backoff_v1(consumer.ack_wait, consumer.max_deliver);
    if matches!(drift, PersonsConsumerDriftV1::RetryBackoff) {
        backoff[0] = backoff[0].saturating_add(Duration::from_millis(1));
    }
    context
        .create_consumer_on_stream(
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(consumer.durable_name.clone()),
                name: Some(consumer.durable_name),
                deliver_policy: if matches!(drift, PersonsConsumerDriftV1::DeliverPolicyNew) {
                    async_nats::jetstream::consumer::DeliverPolicy::New
                } else {
                    async_nats::jetstream::consumer::DeliverPolicy::All
                },
                filter_subject: consumer.subject,
                ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                ack_wait: consumer.ack_wait,
                max_deliver: consumer.max_deliver,
                max_waiting: EVENT_HUB_MAX_PULL_WAITING_V1,
                max_ack_pending: consumer.max_ack_pending,
                headers_only: matches!(drift, PersonsConsumerDriftV1::HeadersOnly),
                max_batch: consumer.max_ack_pending,
                max_expires: consumer.ack_wait,
                inactive_threshold: Duration::ZERO,
                num_replicas: 1,
                replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
                backoff,
                ..Default::default()
            },
            "MAKOSH_COMMAND_V1",
        )
        .await
        .expect("install drifted Persons command consumer");
}

async fn assert_persons_command_consumer_exact_v1(store: &SqliteControlStore) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Persons Event Hub topology")
        .expect("Persons Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let consumer = persons_consumer_fixture_v1(store);
    let context = async_nats::jetstream::new(
        async_nats::connect(endpoint)
            .await
            .expect("connect Persons exact topology fixture"),
    );
    let actual = context
        .get_consumer_from_stream::<async_nats::jetstream::consumer::pull::Config, _, _>(
            &consumer.durable_name,
            "MAKOSH_COMMAND_V1",
        )
        .await
        .expect("read exact Persons command consumer");
    let expected = exact_persons_consumer_config_v1(&consumer).into_consumer_config();
    assert_eq!(actual.cached_info().config, expected);
}

fn exact_persons_consumer_config_v1(
    consumer: &PersonsConsumerFixtureV1,
) -> async_nats::jetstream::consumer::pull::Config {
    async_nats::jetstream::consumer::pull::Config {
        durable_name: Some(consumer.durable_name.clone()),
        name: Some(consumer.durable_name.clone()),
        deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
        filter_subject: consumer.subject.clone(),
        ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
        ack_wait: consumer.ack_wait,
        max_deliver: consumer.max_deliver,
        max_waiting: EVENT_HUB_MAX_PULL_WAITING_V1,
        max_ack_pending: consumer.max_ack_pending,
        max_batch: consumer.max_ack_pending,
        max_expires: consumer.ack_wait,
        inactive_threshold: Duration::ZERO,
        num_replicas: 1,
        replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
        backoff: persons_consumer_retry_backoff_v1(consumer.ack_wait, consumer.max_deliver),
        ..Default::default()
    }
}

fn persons_consumer_retry_backoff_v1(ack_wait: Duration, max_deliver: i64) -> Vec<Duration> {
    (0..max_deliver)
        .scan(ack_wait, |delay, _| {
            let current = *delay;
            *delay = delay.saturating_mul(2).min(Duration::from_secs(600));
            Some(current)
        })
        .collect()
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Persons binaries"]
fn managed_persons_command_is_atomic_replayable_restart_and_control_close_safe() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-persons");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_persons_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            PERSONS_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Persons logical owner");
    let admitted = admit_persons_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Persons Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_persons_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let persons = start_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);
    assert_eq!(persons.runtime_generation, 1);
    assert!(persons.grant_epoch > 0);

    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Persons Event Hub topology")
        .expect("Persons Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let runtime = tokio::runtime::Runtime::new().expect("Persons conformance runtime");
    let created = runtime.block_on(async {
        let client = async_nats::connect(&endpoint)
            .await
            .expect("connect Persons event observer");
        let mut results = client
            .subscribe(RESULT_SUBJECT_V1)
            .await
            .expect("subscribe Persons results");
        let context = async_nats::jetstream::new(client);
        let now = wall_seconds();
        let created = manual_create_command_with_deadline([0x31; 16], [0x41; 16], now, now + 3);
        publish(&context, &created).await;
        let created_result = match tokio::time::timeout(Duration::from_secs(15), results.next())
            .await
        {
            Ok(Some(message)) => message,
            outcome => panic!(
                "Persons first terminal failed outcome={outcome:?}; supervisor={:?}; durable={:?}",
                supervisor.last_failure(&persons.registration_id),
                persons_durable_counts_v1().await
            ),
        };
        let created_terminal_bytes = created_result.payload.to_vec();
        let created_result = DurableEnvelopeV1::decode(created_terminal_bytes.as_slice())
            .expect("decode first Persons terminal result");
        assert_eq!(
            created_result
                .contract
                .as_ref()
                .map(|value| value.name.as_str()),
            Some(
                persons_command_succeeded_contract_reference_v1()
                    .name
                    .as_str()
            )
        );
        let payload = PersonCommandSucceededV1::decode(created_result.payload.as_slice())
            .expect("decode Persons success");
        assert_eq!(payload.command_id, vec![0x31; 16]);
        assert_eq!(payload.affected_person_ids, vec![vec![0x41; 16]]);
        assert_eq!(payload.resulting_person_revisions.len(), 1);
        assert_eq!(payload.resulting_person_revisions[0].person_revision, 1);
        assert_private_fields_absent(&created_result.encode_to_vec());

        let advanced = manual_create_command([0x32; 16], [0x42; 16], now + 1);
        publish(&context, &advanced).await;
        let _ = receive_result(&mut results).await;
        tokio::time::sleep(Duration::from_secs(4)).await;
        let before_replay = persons_durable_counts_v1().await;
        let before_delivery = persons_command_consumer_state_v1(&store).await;
        publish(&context, &created).await;
        let replay_delivery = wait_for_persons_command_ack_v1(&store, before_delivery.0).await;
        assert_eq!(replay_delivery.1, 0, "exact expired replay must be ACKed");
        assert_eq!(
            replay_delivery.2, before_delivery.2,
            "exact expired replay must not redeliver"
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(1), results.next())
                .await
                .is_err(),
            "exact replay after aggregate advance must not duplicate terminal outbox"
        );
        assert_eq!(persons_durable_counts_v1().await, before_replay);
        assert_eq!(
            inbox_terminal_bytes_v1(created.message_id()).await,
            created_terminal_bytes,
            "post-expiry exact replay preserves the original completed terminal bytes"
        );

        let expired_now = wall_seconds();
        let unseen_expired = manual_create_command_with_deadline(
            [0x33; 16],
            [0x43; 16],
            expired_now,
            expired_now + 1,
        );
        tokio::time::sleep(Duration::from_secs(2)).await;
        let persons_before_expired = persons_durable_counts_v1().await.0;
        publish(&context, &unseen_expired).await;
        let rejected = receive_result(&mut results).await;
        assert_eq!(
            rejected.contract.as_ref().map(|value| value.name.as_str()),
            Some(
                persons_command_rejected_contract_reference_v1()
                    .name
                    .as_str()
            )
        );
        let rejected_payload = PersonCommandRejectedV1::decode(rejected.payload.as_slice())
            .expect("decode unseen expired Persons rejection");
        assert_eq!(rejected_payload.command_id, vec![0x33; 16]);
        assert_eq!(persons_durable_counts_v1().await.0, persons_before_expired);
        wait_for_pending_outbox_v1(0).await;
        created
    });

    let (outage_client, mut outage_results, pending) = runtime.block_on(async {
        let client = async_nats::connect(&endpoint)
            .await
            .expect("connect Persons outage observer");
        let results = client
            .subscribe(RESULT_SUBJECT_V1)
            .await
            .expect("subscribe Persons outage results");
        let pending = synthetic_pending_result_v1(wall_seconds());
        (client, results, pending)
    });
    super::nats_outage_fixture::set_authenticated_nats_container_running(false);
    runtime.block_on(async {
        insert_pending_outbox_v1(&pending).await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        assert_eq!(
            pending_outbox_exact_v1(pending.message_id()).await,
            pending.exact_bytes()
        );
        assert_eq!(persons_durable_counts_v1().await.3, 1);
        assert!(
            supervisor
                .is_active(&persons.registration_id)
                .expect("observe Persons during NATS outage"),
            "outbox publication failure must not stop the Persons runtime"
        );
    });
    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    super::nats_outage_fixture::wait_for_authenticated_nats_reconnect(
        &runtime,
        &outage_client,
        "Persons outage observer",
    );
    runtime.block_on(async {
        let context = async_nats::jetstream::new(outage_client.clone());
        let now = wall_seconds();
        for offset in 0..8_u8 {
            publish(
                &context,
                &manual_create_command([0x80 + offset; 16], [0x90 + offset; 16], now),
            )
            .await;
        }
        let mut saw_retained = false;
        let mut burst_commands = std::collections::BTreeSet::new();
        for _ in 0..9 {
            let relayed = tokio::time::timeout(Duration::from_secs(15), outage_results.next())
                .await
                .expect("Persons fair relay timeout")
                .expect("Persons fair relay stream");
            if relayed.payload.as_ref() == pending.exact_bytes() {
                saw_retained = true;
                continue;
            }
            let envelope = DurableEnvelopeV1::decode(relayed.payload.as_ref())
                .expect("decode fair Persons terminal");
            let result = PersonCommandSucceededV1::decode(envelope.payload.as_slice())
                .expect("decode fair Persons success");
            burst_commands.insert(result.command_id);
        }
        assert!(
            saw_retained,
            "preexisting outbox must progress under ready commands"
        );
        assert_eq!(burst_commands.len(), 8);
        wait_for_pending_outbox_v1(0).await;
    });
    runtime.block_on(async move {
        drop(outage_results);
        drop(outage_client);
    });

    let previous_generation = persons.runtime_generation;
    let persons = restart_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), persons);
    assert_eq!(persons.runtime_generation, previous_generation + 1);
    runtime.block_on(async {
        let client = async_nats::connect(&endpoint)
            .await
            .expect("connect restarted Persons observer");
        let mut results = client
            .subscribe(RESULT_SUBJECT_V1)
            .await
            .expect("subscribe restarted Persons results");
        let context = async_nats::jetstream::new(client);
        let before_replay = persons_durable_counts_v1().await;
        publish(&context, &created).await;
        assert!(
            tokio::time::timeout(Duration::from_secs(1), results.next())
                .await
                .is_err(),
            "restart replay must not duplicate terminal outbox"
        );
        assert_eq!(persons_durable_counts_v1().await, before_replay);
    });

    let before_close = runtime.block_on(persons_durable_counts_v1());
    let started = std::time::Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(&persons.registration_id)
            .expect("close Persons control peer")
    );
    assert!(
        supervisor
            .stop_if_active(&persons.registration_id)
            .expect("join control-closed Persons runtime")
    );
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "control closure must stop idle runtime promptly"
    );
    assert_eq!(
        supervisor
            .last_failure(&persons.registration_id)
            .expect("read Persons failure"),
        None,
        "the actual Persons binary must exit cleanly when the control peer closes"
    );
    assert_eq!(runtime.block_on(persons_durable_counts_v1()), before_close);

    assert_eq!(
        store
            .platform_storage_binding(&persons.registration_id, PERSONS_STORAGE_CAPABILITY_ID_V1)
            .expect("read Persons Storage binding")
            .expect("Persons Storage binding")
            .state(),
        PlatformStorageBindingStateV1::Active
    );
    supervisor
        .shutdown()
        .expect("stop managed Persons dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Persons fixture");
    std::fs::remove_dir_all(data).expect("remove Persons Kernel fixture");
}

pub(super) fn manual_create_command(
    command_id: [u8; 16],
    person_id: [u8; 16],
    recorded_at: i64,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    manual_create_command_with_deadline(command_id, person_id, recorded_at, recorded_at + 300)
}

fn manual_create_command_with_deadline(
    command_id: [u8; 16],
    person_id: [u8; 16],
    recorded_at: i64,
    deadline: i64,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    build_persons_command_outbox_record_v1(
        PersonsCommandV1 {
            command: Some(Command::ManualCreate(ManualCreatePersonCommandV1 {
                command_id: command_id.to_vec(),
                person_id: person_id.to_vec(),
                logical_owner_id: PERSONS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
                owner_profile: Some(PersonProfileV1 {
                    display_name: Some("Private Person Name".to_owned()),
                    given_name: Some("Private".to_owned()),
                    family_name: Some("Person".to_owned()),
                    normalized_emails: vec!["private-person@example.test".to_owned()],
                    normalized_phones: vec!["+12025550123".to_owned()],
                }),
                created_at: Some(TimestampV1 {
                    unix_seconds: recorded_at,
                    nanos: 0,
                }),
            })),
        },
        deadline,
        &PersonsEnvelopeContextV1 {
            module_id: "persons-conformance-producer".to_owned(),
            runtime_instance_id: "persons-conformance-producer-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: recorded_at,
            recorded_at_nanos: 0,
        },
    )
    .expect("build exact Persons command")
}

pub(super) async fn publish(
    context: &async_nats::jetstream::Context,
    record: &makosh_events_protocol::delivery::OutboxRecordV1,
) {
    let envelope =
        DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode exact Persons command");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive Persons command subject")
        .as_str();
    context
        .publish(subject, record.exact_bytes().to_vec().into())
        .await
        .expect("publish Persons command")
        .await
        .expect("acknowledge Persons command");
}

async fn receive_result(subscriber: &mut async_nats::Subscriber) -> DurableEnvelopeV1 {
    let message = tokio::time::timeout(Duration::from_secs(15), subscriber.next())
        .await
        .expect("Persons terminal result timeout")
        .expect("Persons terminal result stream");
    DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode Persons terminal result")
}

fn assert_private_fields_absent(bytes: &[u8]) {
    for private in [
        b"Private Person Name".as_slice(),
        b"private-person@example.test".as_slice(),
        b"+12025550123".as_slice(),
    ] {
        assert!(
            !bytes.windows(private.len()).any(|window| window == private),
            "Persons terminal event must not expose private profile fields"
        );
    }
}

fn wall_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Persons wall clock")
        .as_secs()
        .try_into()
        .expect("Persons wall clock range")
}

pub(super) async fn persons_durable_counts_v1() -> (i64, i64, i64, i64) {
    let pool = persons_admin_pool_v1().await;
    let persons: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM makosh_data.persons_current WHERE logical_owner_id=$1",
    )
    .bind(PERSONS_LOGICAL_HUMAN_OWNER_ID_V1)
    .fetch_one(&pool)
    .await
    .expect("count Persons state");
    let inbox: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM makosh_data.persons_command_inbox WHERE logical_owner_id=$1 AND completed=TRUE",
    )
    .bind(PERSONS_LOGICAL_HUMAN_OWNER_ID_V1)
    .fetch_one(&pool)
    .await
    .expect("count Persons inbox");
    let outbox: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM makosh_data.persons_outbox WHERE logical_owner_id=$1",
    )
    .bind(PERSONS_LOGICAL_HUMAN_OWNER_ID_V1)
    .fetch_one(&pool)
    .await
    .expect("count Persons outbox");
    let pending: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM makosh_data.persons_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL",
    )
    .bind(PERSONS_LOGICAL_HUMAN_OWNER_ID_V1)
    .fetch_one(&pool)
    .await
    .expect("count pending Persons outbox");
    pool.close().await;
    (persons, inbox, outbox, pending)
}

async fn inbox_terminal_bytes_v1(command_message_id: &[u8; 16]) -> Vec<u8> {
    let pool = persons_admin_pool_v1().await;
    let bytes: Vec<u8> = sqlx::query_scalar(
        "SELECT terminal_envelope_bytes FROM makosh_data.persons_command_inbox
         WHERE logical_owner_id=$1 AND command_message_id=$2 AND completed=TRUE",
    )
    .bind(PERSONS_LOGICAL_HUMAN_OWNER_ID_V1)
    .bind(command_message_id.as_slice())
    .fetch_one(&pool)
    .await
    .expect("read completed Persons terminal bytes");
    pool.close().await;
    bytes
}

async fn persons_command_consumer_state_v1(store: &SqliteControlStore) -> (u64, usize, usize) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Persons Event Hub topology")
        .expect("Persons Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let fixture = persons_consumer_fixture_v1(store);
    let context = async_nats::jetstream::new(
        async_nats::connect(endpoint)
            .await
            .expect("connect Persons consumer observer"),
    );
    let stream = context
        .get_stream("MAKOSH_COMMAND_V1")
        .await
        .expect("read Persons command stream");
    let mut consumer: async_nats::jetstream::consumer::PullConsumer = stream
        .get_consumer(&fixture.durable_name)
        .await
        .expect("read Persons command consumer");
    let info = consumer
        .info()
        .await
        .expect("refresh Persons consumer info");
    (
        info.ack_floor.consumer_sequence,
        info.num_ack_pending,
        info.num_redelivered,
    )
}

async fn wait_for_persons_command_ack_v1(
    store: &SqliteControlStore,
    previous_ack_floor: u64,
) -> (u64, usize, usize) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let state = persons_command_consumer_state_v1(store).await;
        if state.0 > previous_ack_floor && state.1 == 0 {
            return state;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "exact expired replay was not ACKed before the deadline: {state:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn synthetic_pending_result_v1(
    recorded_at: i64,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    build_persons_command_succeeded_outbox_record_v1(
        [0x71; 16],
        [0x72; 16],
        PersonCommandSucceededV1 {
            command_id: vec![0x73; 16],
            affected_person_ids: vec![vec![0x41; 16]],
            resulting_person_revisions: vec![PersonRevisionV1 {
                person_id: vec![0x41; 16],
                person_revision: 1,
            }],
            logical_owner_id: PERSONS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            resulting_owner_revision: 2,
            ..Default::default()
        },
        &PersonsEnvelopeContextV1 {
            module_id: "makosh-persons-runtime".to_owned(),
            runtime_instance_id: "persons-runtime-outage".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: recorded_at,
            recorded_at_nanos: 0,
        },
    )
    .expect("build retained Persons terminal result")
}

async fn insert_pending_outbox_v1(record: &makosh_events_protocol::delivery::OutboxRecordV1) {
    let pool = persons_admin_pool_v1().await;
    sqlx::query(
        "INSERT INTO makosh_data.persons_outbox
         (logical_owner_id, message_id, envelope_sha256, envelope_bytes, command_message_id,
          resulting_owner_revision, outbox_ordinal, semantic_order_key, created_at_unix_millis)
         VALUES ($1,$2,$3,$4,$2,2,0,decode('00','hex'),$5)",
    )
    .bind(PERSONS_LOGICAL_HUMAN_OWNER_ID_V1)
    .bind(record.message_id().as_slice())
    .bind(record.envelope_sha256().as_slice())
    .bind(record.exact_bytes())
    .bind(wall_seconds() * 1_000)
    .execute(&pool)
    .await
    .expect("insert exact pending Persons outbox");
    pool.close().await;
}

async fn pending_outbox_exact_v1(message_id: &[u8; 16]) -> Vec<u8> {
    let pool = persons_admin_pool_v1().await;
    let bytes: Vec<u8> = sqlx::query_scalar(
        "SELECT envelope_bytes FROM makosh_data.persons_outbox
         WHERE logical_owner_id=$1 AND message_id=$2 AND published_at_unix_millis IS NULL",
    )
    .bind(PERSONS_LOGICAL_HUMAN_OWNER_ID_V1)
    .bind(message_id.as_slice())
    .fetch_one(&pool)
    .await
    .expect("read exact pending Persons outbox");
    pool.close().await;
    bytes
}

async fn wait_for_pending_outbox_v1(expected: i64) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let pending = persons_durable_counts_v1().await.3;
        if pending == expected {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Persons pending outbox count {pending} did not reach {expected}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

pub(super) async fn persons_admin_pool_v1() -> sqlx::PgPool {
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
        .expect("connect Persons conformance database")
}
