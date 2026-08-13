//! Actual-binary managed bootstrap for the dormant Mail-to-Person workflow.

use super::persons_managed_flow::{
    manual_create_command, persons_admin_pool_v1, persons_durable_counts_v1,
};
use super::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::identity::device::signer::DeviceSigner;
use futures_util::StreamExt;
use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::{
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MailAddressBookEnvelopeContextV1, MailAddressBookResultEnvelopeContextV1,
    build_mail_person_source_observed_v1, build_mail_person_source_page_completed_v1,
    build_mail_person_source_page_rejected_v1, build_mail_person_source_removed_v1,
    build_mail_person_source_updated_v1, mail_person_source_claims_digest_v1,
    mail_person_source_tombstone_digest_v1,
    wire_person_source::{
        FetchMailPersonSourcePageCommandV1, MailPersonSourceClaimsV1, MailPersonSourceIdentityV1,
        MailPersonSourceObservedV1, MailPersonSourcePageCompletedV1,
        MailPersonSourcePageRejectedV1, MailPersonSourceProvenanceV1, MailPersonSourceRejectCodeV1,
        MailPersonSourceRemovedV1, MailPersonSourceUpdatedV1,
    },
};
use makosh_mail_persons_sync_api::wire::{
    MailPersonsSyncRejectCodeV1, MailPersonsSyncRunOutcomeV1, MailPersonsSyncRunResultV1,
};
use makosh_mail_persons_sync_runtime::mail_persons_sync_module_descriptor_v1;
use makosh_scheduler_protocol::{
    SCHEDULER_JOB_DESCRIPTOR_SET_V1, SCHEDULER_RUNTIME_MODULE_ID_V1,
    v1::{
        JobKindV1, JobLeaseV1, JobRunOutcomeV1, JobRunReceiptV1, JobTriggerKindV1,
        ScheduledJobCommandV1,
    },
    validate_scheduled_job_command_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

const MAIL_PERSONS_SYNC_EVENT_HUB_MAX_PULL_WAITING_V1: i64 = 512;

struct MailPersonsSyncConsumerFixtureV1 {
    durable_name: String,
    subject: String,
    ack_wait: Duration,
    max_deliver: i64,
    max_ack_pending: i64,
}

struct ToggleableMailPersonsSyncEventCredentialHandlerV1 {
    deny: Arc<AtomicBool>,
    delegate: UnauthenticatedNatsCredentialHandler,
}

impl ManagedRuntimeEventCredentialHandler for ToggleableMailPersonsSyncEventCredentialHandlerV1 {
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String> {
        if self.deny.load(Ordering::SeqCst) {
            Err("Mail Persons Sync Event credential intentionally unavailable".to_owned())
        } else {
            self.delegate.issue_event_credential(expectation, request)
        }
    }
}

#[test]
#[ignore = "requires the real Mail Persons Sync assembly binary"]
fn managed_mail_persons_sync_assembly_cli_is_deterministic_and_cleans_failure() {
    let root = unique_target_root("makosh-mail-persons-sync-assembly-cli");
    std::fs::create_dir_all(&root).expect("assembly CLI root");
    let runtime = root.join("runtime");
    std::fs::write(&runtime, b"runtime-fixture").expect("assembly runtime fixture");
    let assembly = binary("MAKOSH_MAIL_PERSONS_SYNC_ASSEMBLY_BIN");
    for output in [root.join("first"), root.join("second")] {
        let result = std::process::Command::new(&assembly)
            .args([
                "--build-id",
                "task5b-assembly-live",
                "--runtime",
                runtime.to_str().expect("runtime UTF-8"),
                "--output",
                output.to_str().expect("output UTF-8"),
            ])
            .output()
            .expect("run assembly binary");
        assert!(
            result.status.success(),
            "assembly failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8(result.stdout).expect("assembly stdout"),
            "mail-persons-sync-release-assembly: ok\n"
        );
    }
    for name in [
        "mail_persons_sync.runtime.descriptor.pb",
        "mail_persons_sync.runtime.settings.pb",
        "mail_persons_sync.storage.bundle.pb",
        "mail_persons_sync.release-artifacts.json",
    ] {
        assert_eq!(
            std::fs::read(root.join("first").join(name)).expect("first artifact"),
            std::fs::read(root.join("second").join(name)).expect("second artifact"),
            "{name} must be deterministic"
        );
    }
    let empty_runtime = root.join("empty-runtime");
    std::fs::write(&empty_runtime, []).expect("empty runtime fixture");
    let failed = root.join("failed");
    let result = std::process::Command::new(&assembly)
        .args([
            "--build-id",
            "task5b-assembly-live",
            "--runtime",
            empty_runtime.to_str().expect("empty runtime UTF-8"),
            "--output",
            failed.to_str().expect("failed output UTF-8"),
        ])
        .output()
        .expect("run failing assembly binary");
    assert!(!result.status.success());
    assert!(!failed.exists(), "failed assembly must leave no output");
    std::fs::remove_dir_all(root).expect("remove assembly CLI fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Mail Persons Sync binaries"]
fn managed_mail_persons_sync_bootstrap_is_exact_and_control_responsive() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-mail-persons-sync-bootstrap");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_mail_persons_sync_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Mail Persons Sync logical owner");
    let admitted = admit_mail_persons_sync_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    let deny_event_credential = Arc::new(AtomicBool::new(true));
    supervisor
        .configure_event_credential_handler(Arc::new(
            ToggleableMailPersonsSyncEventCredentialHandlerV1 {
                deny: Arc::clone(&deny_event_credential),
                delegate: UnauthenticatedNatsCredentialHandler::new(Arc::clone(&store)),
            },
        ))
        .expect("configure toggleable Mail Persons Sync Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_mail_persons_sync_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);

    // With no Event credential handler, exact authority cannot reach ready.
    let started = launch_mail_persons_sync_runtime_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted,
        MailPersonsSyncBootstrapOverrideV1::None,
    );
    assert_never_ready_then_stop_v1(&supervisor, &started, "missing Event credential");
    deny_event_credential.store(false, Ordering::SeqCst);

    let started = reject_mail_persons_sync_extra_capability_successor_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
    );
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("extra-capability activity"),
        "undeclared capability must not launch a child"
    );

    let started = reject_mail_persons_sync_missing_storage_successor_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
    );
    assert!(
        !supervisor
            .is_active(&started.registration_id)
            .expect("missing-storage activity"),
        "Kernel-denied missing storage must not launch a child"
    );

    let started = launch_mail_persons_sync_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
        MailPersonsSyncBootstrapOverrideV1::StaleCredentialFence,
    );
    assert_never_ready_then_stop_v1(&supervisor, &started, "stale credential fence");

    let started = launch_mail_persons_sync_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
        MailPersonsSyncBootstrapOverrideV1::StopVaultAfterConfiguration,
    );
    assert_active_pre_ready_then_stop_v1(&supervisor, &started, "Vault bootstrap");
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

    let stalled_storage =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind stalled Storage endpoint");
    let stalled_storage_port = stalled_storage
        .local_addr()
        .expect("stalled Storage address")
        .port();
    let started = launch_mail_persons_sync_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
        MailPersonsSyncBootstrapOverrideV1::UnavailableStoragePort(stalled_storage_port),
    );
    assert_active_pre_ready_then_stop_v1(&supervisor, &started, "Storage bootstrap");
    drop(stalled_storage);

    let stalled_nats =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind stalled NATS endpoint");
    let stalled_nats_endpoint = format!(
        "nats://{}",
        stalled_nats.local_addr().expect("stalled NATS address")
    );
    let started = launch_mail_persons_sync_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
        MailPersonsSyncBootstrapOverrideV1::UnavailableEventEndpoint(stalled_nats_endpoint),
    );
    assert_active_pre_ready_then_stop_v1(&supervisor, &started, "NATS bootstrap");
    drop(stalled_nats);

    let runtime = tokio::runtime::Runtime::new().expect("Mail Persons Sync topology runtime");
    runtime.block_on(delete_mail_persons_sync_scheduler_consumer_v1(&store));
    let started = launch_mail_persons_sync_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
        MailPersonsSyncBootstrapOverrideV1::None,
    );
    assert_never_ready_then_stop_v1(&supervisor, &started, "missing consumer topology");

    configure_communications_jetstream(&store);
    runtime.block_on(install_drifted_mail_persons_sync_scheduler_consumer_v1(
        &store,
    ));
    let started = launch_mail_persons_sync_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
        MailPersonsSyncBootstrapOverrideV1::None,
    );
    assert_never_ready_then_stop_v1(&supervisor, &started, "headers-only consumer drift");

    runtime.block_on(delete_mail_persons_sync_scheduler_consumer_v1(&store));
    configure_communications_jetstream(&store);
    let started = launch_mail_persons_sync_successor_without_ready_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        started,
        MailPersonsSyncBootstrapOverrideV1::None,
    );
    supervisor
        .wait_until_ready(&started.registration_id)
        .expect("healthy Mail Persons Sync recovery");
    assert_prompt_stop_v1(&supervisor, &started, "post-ready idle/backpressure stop");

    supervisor
        .shutdown()
        .expect("stop Mail Persons Sync bootstrap dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail Persons Sync bootstrap fixture");
    std::fs::remove_dir_all(data).expect("remove Mail Persons Sync Kernel fixture");
}

fn assert_active_pre_ready_then_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedMailPersonsSyncRuntimeV1,
    phase: &str,
) {
    std::thread::sleep(Duration::from_millis(250));
    assert!(
        supervisor
            .is_active(&started.registration_id)
            .unwrap_or_else(|error| panic!("{phase} activity: {error}")),
        "{phase} must stay supervised until control closes"
    );
    assert!(
        !supervisor
            .relay_port()
            .is_ready(&started.registration_id)
            .unwrap_or_else(|error| panic!("{phase} readiness: {error}")),
        "{phase} must not signal ready"
    );
    assert_prompt_stop_v1(supervisor, started, phase);
}

fn assert_never_ready_then_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedMailPersonsSyncRuntimeV1,
    phase: &str,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        assert!(
            !supervisor
                .relay_port()
                .is_ready(&started.registration_id)
                .unwrap_or_else(|error| panic!("{phase} readiness: {error}")),
            "{phase} must never signal ready"
        );
        if !supervisor
            .is_active(&started.registration_id)
            .unwrap_or_else(|error| panic!("{phase} activity: {error}"))
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_prompt_stop_v1(supervisor, started, phase);
}

fn assert_prompt_stop_v1(
    supervisor: &ManagedRuntimeSupervisor,
    started: &StartedMailPersonsSyncRuntimeV1,
    phase: &str,
) {
    let stop_started = std::time::Instant::now();
    assert!(
        supervisor
            .request_stop_if_active(&started.registration_id)
            .unwrap_or_else(|error| panic!("{phase} request stop: {error}"))
    );
    assert!(
        supervisor
            .stop_if_active(&started.registration_id)
            .unwrap_or_else(|error| panic!("{phase} stop: {error}"))
    );
    assert!(
        stop_started.elapsed() < Duration::from_secs(2),
        "{phase} control close must be prompt"
    );
}

fn mail_persons_sync_scheduler_consumer_fixture_v1(
    store: &SqliteControlStore,
) -> MailPersonsSyncConsumerFixtureV1 {
    let topology = store
        .platform_event_hub_topology()
        .expect("read Mail Persons Sync Event Hub topology")
        .expect("Mail Persons Sync Event Hub topology");
    let contracts =
        event_catalog::resolve_contracts(store).expect("resolve Mail Persons Sync Event contracts");
    let plan =
        event_topology::plan(&contracts, &topology).expect("plan Mail Persons Sync Event topology");
    let descriptor = mail_persons_sync_module_descriptor_v1("topology-fixture");
    let expected = descriptor
        .capabilities
        .iter()
        .find(|capability| capability.capability_id == "mail_persons_sync.scheduler.v1")
        .and_then(|capability| capability.provides.first())
        .and_then(|surface| surface.contract.clone())
        .expect("Mail Persons Sync Scheduler contract");
    let consumer = plan
        .consumers()
        .iter()
        .find(|consumer| {
            let contract = consumer.contract();
            contract.owner == expected.owner
                && contract.name == expected.name
                && contract.major == expected.major
                && contract.revision == expected.revision
        })
        .expect("exact Mail Persons Sync Scheduler consumer");
    MailPersonsSyncConsumerFixtureV1 {
        durable_name: consumer.durable_name().to_owned(),
        subject: consumer.subject().as_str(),
        ack_wait: Duration::from_millis(consumer.delivery_policy().ack_wait_millis().into()),
        max_deliver: i64::from(consumer.delivery_policy().max_deliver()),
        max_ack_pending: i64::from(consumer.max_in_flight()),
    }
}

async fn delete_mail_persons_sync_scheduler_consumer_v1(store: &SqliteControlStore) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Mail Persons Sync Event Hub topology")
        .expect("Mail Persons Sync Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let consumer = mail_persons_sync_scheduler_consumer_fixture_v1(store);
    async_nats::jetstream::new(
        async_nats::connect(endpoint)
            .await
            .expect("connect Mail Persons Sync topology fixture"),
    )
    .delete_consumer_from_stream(consumer.durable_name, "MAKOSH_COMMAND_V1")
    .await
    .expect("delete Mail Persons Sync Scheduler consumer");
}

async fn install_drifted_mail_persons_sync_scheduler_consumer_v1(store: &SqliteControlStore) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Mail Persons Sync Event Hub topology")
        .expect("Mail Persons Sync Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let consumer = mail_persons_sync_scheduler_consumer_fixture_v1(store);
    let context = async_nats::jetstream::new(
        async_nats::connect(endpoint)
            .await
            .expect("connect Mail Persons Sync drift fixture"),
    );
    context
        .delete_consumer_from_stream(&consumer.durable_name, "MAKOSH_COMMAND_V1")
        .await
        .expect("delete exact Mail Persons Sync Scheduler consumer before drift");
    context
        .create_consumer_on_stream(
            async_nats::jetstream::consumer::pull::Config {
                headers_only: true,
                ..exact_mail_persons_sync_consumer_config_v1(&consumer)
            },
            "MAKOSH_COMMAND_V1",
        )
        .await
        .expect("install drifted Mail Persons Sync Scheduler consumer");
}

fn exact_mail_persons_sync_consumer_config_v1(
    consumer: &MailPersonsSyncConsumerFixtureV1,
) -> async_nats::jetstream::consumer::pull::Config {
    async_nats::jetstream::consumer::pull::Config {
        durable_name: Some(consumer.durable_name.clone()),
        name: Some(consumer.durable_name.clone()),
        deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
        filter_subject: consumer.subject.clone(),
        ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
        ack_wait: consumer.ack_wait,
        max_deliver: consumer.max_deliver,
        max_waiting: MAIL_PERSONS_SYNC_EVENT_HUB_MAX_PULL_WAITING_V1,
        max_ack_pending: consumer.max_ack_pending,
        max_batch: consumer.max_ack_pending,
        max_expires: consumer.ack_wait,
        inactive_threshold: Duration::ZERO,
        num_replicas: 1,
        replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
        backoff: mail_persons_sync_consumer_retry_backoff_v1(
            consumer.ack_wait,
            consumer.max_deliver,
        ),
        ..Default::default()
    }
}

fn mail_persons_sync_consumer_retry_backoff_v1(
    ack_wait: Duration,
    max_deliver: i64,
) -> Vec<Duration> {
    (0..max_deliver)
        .scan(ack_wait, |delay, _| {
            let current = *delay;
            *delay = delay.saturating_mul(2).min(Duration::from_secs(600));
            Some(current)
        })
        .collect()
}

#[test]
fn mail_persons_sync_scheduler_fixture_is_an_exact_durable_command() {
    let bytes = scheduler_due_v1(1_700_000_000_000, [0x51; 16], [0x31; 16]);
    let envelope = DurableEnvelopeV1::decode(bytes.as_slice()).expect("scheduler envelope");
    validate_envelope_v1(&envelope).expect("valid durable scheduler envelope");
    let command =
        ScheduledJobCommandV1::decode(envelope.payload.as_slice()).expect("scheduler payload");
    validate_scheduled_job_command_v1(&command).expect("valid scheduler payload");
    let deadline = match envelope.semantics.expect("scheduler semantics") {
        Semantics::Command(metadata) => metadata.deadline.expect("scheduler deadline"),
        _ => panic!("scheduler command semantics"),
    };
    let lease = command.lease.expect("scheduler lease");
    assert_eq!(
        deadline.seconds * 1_000 + i64::from(deadline.nanos / 1_000_000),
        lease.expires_at_unix_millis,
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Mail Persons Sync binary"]
fn managed_mail_persons_sync_actual_binary_bootstraps_exact_private_contour() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-mail-persons-sync");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_mail_persons_sync_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Mail Persons Sync logical owner");
    let admitted_persons = admit_persons_runtime_v1(&store);
    let admitted = admit_mail_persons_sync_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Mail Persons Sync Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted_persons = prepare_persons_runtime_v1(&supervisor, &store, admitted_persons);
    let admitted = prepare_mail_persons_sync_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let persons =
        start_persons_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_persons);
    let mut started =
        start_mail_persons_sync_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);
    assert_eq!(persons.runtime_generation, 1);
    assert_eq!(started.runtime_generation, 1);
    assert!(
        supervisor
            .relay_port()
            .is_ready(&started.registration_id)
            .expect("read Mail Persons Sync readiness")
    );
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read event topology")
        .expect("event topology")
        .nats_endpoint()
        .to_owned();
    let runtime = tokio::runtime::Runtime::new().expect("Mail Persons Sync event harness");
    let _runtime_context = runtime.enter();
    let outage_client = runtime.block_on(exercise_source_lifecycle_flow_v1(&endpoint));
    super::nats_outage_fixture::set_authenticated_nats_container_running(false);
    let retained = runtime.block_on(retain_exact_run_result_for_outage_v1([0x71; 16]));
    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(250)).await;
        assert_eq!(pending_exact_outbox_v1(retained.0).await, retained.1);
    });
    assert!(
        supervisor
            .is_active(&started.registration_id)
            .expect("observe Mail Persons Sync during NATS outage"),
        "publish outage must not stop the workflow",
    );
    supervisor
        .stop(&started.registration_id)
        .expect("stop Mail Persons Sync with pending outbox");
    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    super::nats_outage_fixture::wait_for_authenticated_nats_reconnect(
        &runtime,
        &outage_client,
        "Mail Persons Sync outage observer",
    );
    let mut replay = runtime.block_on(async {
        let envelope = DurableEnvelopeV1::decode(retained.1.as_slice())
            .expect("decode retained workflow result");
        let subject = DurableSubjectV1::from_envelope(&envelope)
            .expect("retained workflow result subject")
            .as_str();
        let subscriber = outage_client
            .subscribe(subject)
            .await
            .expect("subscribe retained workflow result");
        outage_client
            .flush()
            .await
            .expect("activate retained result subscriber");
        subscriber
    });
    started =
        restart_mail_persons_sync_runtime_v1(&supervisor, &store, &root.join("runtime"), started);
    assert_eq!(started.runtime_generation, 2);
    runtime.block_on(async {
        let message = tokio::time::timeout(Duration::from_secs(15), replay.next())
            .await
            .expect("retained workflow relay timeout")
            .expect("retained workflow relay delivery");
        assert_eq!(message.payload.as_ref(), retained.1.as_slice());
        assert_eq!(pending_exact_outbox_v1(retained.0).await, Vec::<u8>::new());
    });
    supervisor
        .shutdown()
        .expect("stop managed Mail Persons Sync dependencies");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail Persons Sync fixture");
    std::fs::remove_dir_all(data).expect("remove Mail Persons Sync Kernel fixture");
}

async fn exercise_source_lifecycle_flow_v1(endpoint: &str) -> async_nats::Client {
    let client = async_nats::connect(endpoint)
        .await
        .expect("connect Mail Persons Sync harness");
    let mut fetches = client
        .subscribe("makosh.command.v1.mail.mail_person_source_fetch_page.v1")
        .await
        .expect("subscribe exact Mail fetch command");
    let mut results = client
        .subscribe("makosh.result.v1.mail_persons_sync.>")
        .await
        .expect("subscribe workflow results");
    let mut persons_results = client
        .subscribe("makosh.result.v1.persons.>")
        .await
        .expect("subscribe Persons terminals");
    let mut scheduler_results = client
        .subscribe("makosh.result.v1.scheduler.>")
        .await
        .expect("subscribe Scheduler receipts");
    client
        .flush()
        .await
        .expect("activate Mail Persons Sync harness subscriptions");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_millis() as i64;
    let run_id = [0x51; 16];
    let account_public_id = [0x31; 16];
    let unrelated = manual_create_command([0x21; 16], [0x22; 16], now / 1_000);
    publish_exact_v1(&client, unrelated.exact_bytes()).await;
    assert_persons_success_v1(&mut persons_results).await;
    let mail_rejected_run_id = [0x53; 16];
    let rejected_account_public_id = [0x32; 16];
    let (mail_rejected_fetch, mail_rejected_fetch_message_id) = begin_and_receive_fetch_v1(
        &client,
        &mut fetches,
        now,
        mail_rejected_run_id,
        rejected_account_public_id,
    )
    .await;
    let mail_rejected_command_id: [u8; 16] = mail_rejected_fetch
        .command_id
        .as_slice()
        .try_into()
        .expect("Mail rejected fetch command ID");
    let mail_rejected = build_mail_person_source_page_rejected_v1(
        mail_rejected_fetch_message_id,
        MailPersonSourcePageRejectedV1 {
            command_id: mail_rejected_command_id.to_vec(),
            run_id: mail_rejected_run_id.to_vec(),
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            account_public_id: rejected_account_public_id.to_vec(),
            page_sequence: 1,
            code: MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeSourceUnavailable as i32,
            retryable: true,
            rejected_at: Some(Timestamp {
                seconds: now / 1_000,
                nanos: ((now % 1_000) * 1_000_000) as i32,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "mail-person-source-harness".to_owned(),
            runtime_generation: 1,
            completed_at_unix_seconds: now / 1_000,
            completed_at_nanos: ((now % 1_000) * 1_000_000) as i32,
            execution_attempt: 1,
        },
    )
    .expect("build exact retryable Mail page rejection");
    publish_exact_v1(&client, mail_rejected.exact_bytes()).await;
    assert_workflow_rejected_code_v1(
        &mut results,
        mail_rejected_run_id,
        MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeSourceUnavailable,
    )
    .await;
    assert_scheduler_retryable_terminal_v1(&mut scheduler_results, mail_rejected_run_id).await;
    let (fetch, fetch_message_id) =
        begin_and_receive_fetch_v1(&client, &mut fetches, now, run_id, account_public_id).await;
    let concurrent_run_id = [0x52; 16];
    publish_exact_v1(
        &client,
        &scheduler_due_v1(now + 1, concurrent_run_id, account_public_id),
    )
    .await;
    assert_scheduler_retryable_terminal_v1(&mut scheduler_results, concurrent_run_id).await;
    let source = MailPersonSourceIdentityV1 {
        integration_public_id: vec![0x61; 16],
        account_public_id: account_public_id.to_vec(),
        provider_source_contact_public_id: vec![0x62; 16],
    };
    let claims = MailPersonSourceClaimsV1 {
        display_name: Some("Managed Public Person".to_owned()),
        normalized_emails: vec!["managed-public@example.test".to_owned()],
        normalized_phones: Vec::new(),
    };
    let claims_digest = mail_person_source_claims_digest_v1(&source, &claims)
        .expect("canonical public claims digest");
    let observed = build_mail_person_source_observed_v1(
        fetch_message_id,
        MailPersonSourceObservedV1 {
            observation_id: vec![0x63; 16],
            run_id: run_id.to_vec(),
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            claims: Some(claims),
            provenance: Some(MailPersonSourceProvenanceV1 {
                source_revision: 1,
                source_digest: claims_digest.to_vec(),
                observed_at: Some(Timestamp {
                    seconds: now / 1_000,
                    nanos: 0,
                }),
            }),
        },
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-runtime".to_owned(),
            runtime_instance_id: "mail-person-source-harness".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now / 1_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build sanitized source observation");
    let command_id: [u8; 16] = fetch
        .command_id
        .as_slice()
        .try_into()
        .expect("fetch command ID");
    let page_digest: [u8; 32] = Sha256::digest(b"mail-persons-sync-observed-page-v1").into();
    let page = build_mail_person_source_page_completed_v1(
        fetch_message_id,
        MailPersonSourcePageCompletedV1 {
            command_id: command_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            account_public_id: account_public_id.to_vec(),
            page_sequence: 1,
            observed_sources: 1,
            updated_sources: 0,
            removed_sources: 0,
            has_more: false,
            page_digest: page_digest.to_vec(),
            completed_at: Some(Timestamp {
                seconds: now / 1_000,
                nanos: 0,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "mail-person-source-harness".to_owned(),
            runtime_generation: 1,
            completed_at_unix_seconds: now / 1_000,
            completed_at_nanos: 0,
            execution_attempt: 1,
        },
    )
    .expect("build sanitized observed page");
    // Mail source observations and page terminals use independent streams.
    // Exercise the normal reverse arrival order: the terminal must remain
    // uncommitted and retryable without killing the managed runtime.
    publish_exact_v1(&client, page.exact_bytes()).await;
    assert_no_persons_terminal_v1(&mut persons_results).await;
    assert_eq!(mail_persons_sync_pending_outbox_v1(run_id).await, 0);
    publish_exact_v1(&client, observed.exact_bytes()).await;
    wait_for_staged_source_v1(run_id).await;
    assert_persons_success_v1(&mut persons_results).await;
    assert_workflow_results_v1(&mut results).await;
    assert_eq!(persons_durable_counts_v1().await.0, 2);

    let rejected_run_id = [0x65; 16];
    let (rejected_fetch, rejected_fetch_message_id) = begin_and_receive_fetch_v1(
        &client,
        &mut fetches,
        now,
        rejected_run_id,
        account_public_id,
    )
    .await;
    let changed_claims = MailPersonSourceClaimsV1 {
        display_name: Some("Conflicting Same Revision".to_owned()),
        normalized_emails: vec!["same-revision-conflict@example.test".to_owned()],
        normalized_phones: Vec::new(),
    };
    let changed_digest = mail_person_source_claims_digest_v1(&source, &changed_claims)
        .expect("changed same-revision claims digest");
    let updated = build_mail_person_source_updated_v1(
        rejected_fetch_message_id,
        MailPersonSourceUpdatedV1 {
            observation_id: vec![0x66; 16],
            run_id: rejected_run_id.to_vec(),
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            claims: Some(changed_claims),
            provenance: Some(MailPersonSourceProvenanceV1 {
                source_revision: 1,
                source_digest: changed_digest.to_vec(),
                observed_at: Some(Timestamp {
                    seconds: now / 1_000,
                    nanos: 0,
                }),
            }),
        },
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-runtime".to_owned(),
            runtime_instance_id: "mail-person-source-harness".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now / 1_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build same-revision conflicting update");
    publish_exact_v1(&client, updated.exact_bytes()).await;
    wait_for_staged_source_v1(rejected_run_id).await;
    let rejected_command_id: [u8; 16] = rejected_fetch
        .command_id
        .as_slice()
        .try_into()
        .expect("rejected fetch command ID");
    let rejected_page = build_mail_person_source_page_completed_v1(
        rejected_fetch_message_id,
        MailPersonSourcePageCompletedV1 {
            command_id: rejected_command_id.to_vec(),
            run_id: rejected_run_id.to_vec(),
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            account_public_id: account_public_id.to_vec(),
            page_sequence: 1,
            observed_sources: 0,
            updated_sources: 1,
            removed_sources: 0,
            has_more: false,
            page_digest: Sha256::digest(b"mail-persons-sync-rejected-page-v1").to_vec(),
            completed_at: Some(Timestamp {
                seconds: now / 1_000,
                nanos: 0,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "mail-person-source-harness".to_owned(),
            runtime_generation: 1,
            completed_at_unix_seconds: now / 1_000,
            completed_at_nanos: 0,
            execution_attempt: 1,
        },
    )
    .expect("build rejected page");
    publish_exact_v1(&client, rejected_page.exact_bytes()).await;
    assert_persons_rejected_v1(&mut persons_results).await;
    assert_workflow_rejected_v1(&mut results, rejected_run_id).await;
    assert_scheduler_retryable_terminal_v1(&mut scheduler_results, rejected_run_id).await;

    let remove_run_id = [0x71; 16];
    let (remove_fetch, remove_fetch_message_id) =
        begin_and_receive_fetch_v1(&client, &mut fetches, now, remove_run_id, account_public_id)
            .await;
    let tombstone_digest =
        mail_person_source_tombstone_digest_v1(&source).expect("canonical public tombstone digest");
    let removed = build_mail_person_source_removed_v1(
        remove_fetch_message_id,
        MailPersonSourceRemovedV1 {
            observation_id: vec![0x73; 16],
            run_id: remove_run_id.to_vec(),
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            page_sequence: 1,
            source: Some(source),
            provenance: Some(MailPersonSourceProvenanceV1 {
                source_revision: 2,
                source_digest: tombstone_digest.to_vec(),
                observed_at: Some(Timestamp {
                    seconds: now / 1_000,
                    nanos: 0,
                }),
            }),
        },
        &MailAddressBookEnvelopeContextV1 {
            module_id: "makosh-mail-runtime".to_owned(),
            runtime_instance_id: "mail-person-source-harness".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now / 1_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build sanitized source tombstone");
    publish_exact_v1(&client, removed.exact_bytes()).await;
    wait_for_staged_source_v1(remove_run_id).await;
    let remove_command_id: [u8; 16] = remove_fetch
        .command_id
        .as_slice()
        .try_into()
        .expect("remove fetch command ID");
    let remove_page = build_mail_person_source_page_completed_v1(
        remove_fetch_message_id,
        MailPersonSourcePageCompletedV1 {
            command_id: remove_command_id.to_vec(),
            run_id: remove_run_id.to_vec(),
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            account_public_id: account_public_id.to_vec(),
            page_sequence: 1,
            observed_sources: 0,
            updated_sources: 0,
            removed_sources: 1,
            has_more: false,
            page_digest: Sha256::digest(b"mail-persons-sync-removed-page-v1").to_vec(),
            completed_at: Some(Timestamp {
                seconds: now / 1_000,
                nanos: 0,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: "mail-person-source-harness".to_owned(),
            runtime_generation: 1,
            completed_at_unix_seconds: now / 1_000,
            completed_at_nanos: 0,
            execution_attempt: 1,
        },
    )
    .expect("build sanitized removed page");
    publish_exact_v1(&client, remove_page.exact_bytes()).await;
    assert_persons_success_v1(&mut persons_results).await;
    assert_workflow_results_v1(&mut results).await;
    assert_eq!(
        persons_durable_counts_v1().await.0,
        2,
        "source tombstone must retain Person"
    );

    client
}

async fn assert_scheduler_retryable_terminal_v1(
    results: &mut async_nats::Subscriber,
    run_id: [u8; 16],
) {
    for _ in 0..4 {
        let message = tokio::time::timeout(Duration::from_secs(15), results.next())
            .await
            .expect("Scheduler receipt timeout")
            .expect("Scheduler receipt delivery");
        let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
            .expect("decode Scheduler receipt envelope");
        let receipt = JobRunReceiptV1::decode(envelope.payload.as_slice())
            .expect("decode Scheduler receipt payload");
        if receipt.job_run_id == run_id
            && receipt.outcome == JobRunOutcomeV1::RetryableFailed as i32
        {
            return;
        }
    }
    panic!("concurrent Scheduler run did not receive retryable terminal");
}

async fn retain_exact_run_result_for_outage_v1(run_id: [u8; 16]) -> ([u8; 16], Vec<u8>) {
    let pool = persons_admin_pool_v1().await;
    let (message_id, bytes): (Vec<u8>, Vec<u8>) = sqlx::query_as(
        "SELECT message_id,envelope_bytes FROM makosh_data.mail_persons_sync_outbox \
         WHERE logical_owner_id=$1 AND run_id=$2 AND semantic_kind=6",
    )
    .bind(MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1)
    .bind(run_id.as_slice())
    .fetch_one(&pool)
    .await
    .expect("load exact completed run result");
    let message_id: [u8; 16] = message_id.try_into().expect("run result message ID");
    let updated = sqlx::query(
        "UPDATE makosh_data.mail_persons_sync_outbox SET published_at_unix_millis=NULL \
         WHERE logical_owner_id=$1 AND message_id=$2 AND published_at_unix_millis IS NOT NULL",
    )
    .bind(MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1)
    .bind(message_id.as_slice())
    .execute(&pool)
    .await
    .expect("simulate committed-before-publish workflow result");
    assert_eq!(updated.rows_affected(), 1);
    pool.close().await;
    (message_id, bytes)
}

async fn pending_exact_outbox_v1(message_id: [u8; 16]) -> Vec<u8> {
    let pool = persons_admin_pool_v1().await;
    let bytes = sqlx::query_scalar(
        "SELECT envelope_bytes FROM makosh_data.mail_persons_sync_outbox \
         WHERE logical_owner_id=$1 AND message_id=$2 AND published_at_unix_millis IS NULL",
    )
    .bind(MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1)
    .bind(message_id.as_slice())
    .fetch_optional(&pool)
    .await
    .expect("load exact pending workflow outbox")
    .unwrap_or_default();
    pool.close().await;
    bytes
}

async fn wait_for_staged_source_v1(run_id: [u8; 16]) {
    let pool = persons_admin_pool_v1().await;
    for _ in 0..200 {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_sources \
             WHERE logical_owner_id=$1 AND run_id=$2",
        )
        .bind(MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1)
        .bind(run_id.as_slice())
        .fetch_one(&pool)
        .await
        .expect("count staged Mail Persons Sync sources");
        if count == 1 {
            pool.close().await;
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    pool.close().await;
    panic!("Mail Persons Sync source was not durably staged");
}

async fn mail_persons_sync_pending_outbox_v1(run_id: [u8; 16]) -> i64 {
    let pool = persons_admin_pool_v1().await;
    let count = sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_outbox \
         WHERE logical_owner_id=$1 AND run_id=$2 AND published_at_unix_millis IS NULL",
    )
    .bind(MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1)
    .bind(run_id.as_slice())
    .fetch_one(&pool)
    .await
    .expect("count pending Mail Persons Sync outbox");
    pool.close().await;
    count
}

async fn begin_and_receive_fetch_v1(
    client: &async_nats::Client,
    fetches: &mut async_nats::Subscriber,
    now: i64,
    run_id: [u8; 16],
    account_public_id: [u8; 16],
) -> (FetchMailPersonSourcePageCommandV1, [u8; 16]) {
    publish_exact_v1(client, &scheduler_due_v1(now, run_id, account_public_id)).await;
    let message = match tokio::time::timeout(Duration::from_secs(15), fetches.next()).await {
        Ok(message) => message,
        Err(_) => {
            let pool = persons_admin_pool_v1().await;
            let state: Option<i16> = sqlx::query_scalar(
                "SELECT state FROM makosh_data.mail_persons_sync_runs WHERE logical_owner_id=$1 AND run_id=$2",
            )
            .bind(MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1)
            .bind(run_id.as_slice())
            .fetch_optional(&pool)
            .await
            .expect("fetch-timeout run diagnostic");
            let outbox: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_outbox WHERE logical_owner_id=$1 AND run_id=$2",
            )
            .bind(MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1)
            .bind(run_id.as_slice())
            .fetch_one(&pool)
            .await
            .expect("fetch-timeout outbox diagnostic");
            pool.close().await;
            panic!("fetch timeout for run {run_id:02x?}; state={state:?}; outbox={outbox}");
        }
    }
        .expect("fetch delivery");
    let envelope =
        DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode exact fetch envelope");
    let message_id = envelope
        .message_id
        .as_slice()
        .try_into()
        .expect("fetch message ID");
    let fetch = FetchMailPersonSourcePageCommandV1::decode(envelope.payload.as_slice())
        .expect("decode exact fetch payload");
    assert_eq!(
        fetch.logical_owner_id,
        MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1
    );
    assert_eq!(fetch.account_public_id, account_public_id);
    assert_eq!(fetch.run_id, run_id);
    assert_eq!(fetch.page_sequence, 1);
    (fetch, message_id)
}

async fn assert_persons_success_v1(results: &mut async_nats::Subscriber) {
    let message = match tokio::time::timeout(Duration::from_secs(15), results.next()).await {
        Ok(Some(message)) => message,
        Ok(None) => panic!("Persons terminal subscription closed"),
        Err(_) => panic!(
            "Persons terminal timeout; durable diagnostic={:?}",
            mail_persons_sync_diagnostic_v1().await,
        ),
    };
    let envelope =
        DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode Persons terminal");
    assert_eq!(
        envelope.contract.expect("Persons terminal contract").name,
        "persons_command_succeeded"
    );
}

async fn assert_persons_rejected_v1(results: &mut async_nats::Subscriber) {
    let message = tokio::time::timeout(Duration::from_secs(15), results.next())
        .await
        .expect("Persons rejection timeout")
        .expect("Persons rejection delivery");
    let envelope =
        DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode Persons rejection");
    assert_eq!(
        envelope.contract.expect("Persons rejection contract").name,
        "persons_command_rejected"
    );
}

async fn assert_workflow_rejected_v1(results: &mut async_nats::Subscriber, run_id: [u8; 16]) {
    for _ in 0..2 {
        let message = tokio::time::timeout(Duration::from_secs(15), results.next())
            .await
            .expect("workflow rejection timeout")
            .expect("workflow rejection delivery");
        let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
            .expect("decode workflow rejection envelope");
        if envelope
            .contract
            .as_ref()
            .map(|contract| contract.name.as_str())
            == Some("mail_persons_sync_run_result")
        {
            let result = MailPersonsSyncRunResultV1::decode(envelope.payload.as_slice())
                .expect("decode rejected RunResult");
            assert_eq!(result.run_id, run_id);
            assert_eq!(
                result.outcome,
                MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeRejected as i32,
            );
            return;
        }
    }
    panic!("rejected RunResult was not published");
}

async fn assert_workflow_rejected_code_v1(
    results: &mut async_nats::Subscriber,
    run_id: [u8; 16],
    expected_code: MailPersonsSyncRejectCodeV1,
) {
    for _ in 0..2 {
        let message = tokio::time::timeout(Duration::from_secs(15), results.next())
            .await
            .expect("workflow rejection timeout")
            .expect("workflow rejection delivery");
        let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
            .expect("decode workflow rejection envelope");
        if envelope
            .contract
            .as_ref()
            .map(|contract| contract.name.as_str())
            == Some("mail_persons_sync_run_result")
        {
            let result = MailPersonsSyncRunResultV1::decode(envelope.payload.as_slice())
                .expect("decode rejected RunResult");
            assert_eq!(result.run_id, run_id);
            assert_eq!(
                result.outcome,
                MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeRejected as i32,
            );
            assert_eq!(result.code, expected_code as i32);
            return;
        }
    }
    panic!("coded rejected RunResult was not published");
}

async fn mail_persons_sync_diagnostic_v1() -> (i64, i64, i64, i64, i64, i64) {
    let pool = persons_admin_pool_v1().await;
    let values = (
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_sources")
            .fetch_one(&pool).await.expect("diagnostic workflow sources"),
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_pages WHERE completed_message_id IS NOT NULL",
        ).fetch_one(&pool).await.expect("diagnostic completed workflow pages"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.mail_persons_sync_outbox")
            .fetch_one(&pool).await.expect("diagnostic workflow outbox"),
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_persons_sync_outbox WHERE published_at_unix_millis IS NULL",
        ).fetch_one(&pool).await.expect("diagnostic pending workflow outbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.persons_command_inbox")
            .fetch_one(&pool).await.expect("diagnostic Persons inbox"),
        sqlx::query_scalar("SELECT COUNT(*) FROM makosh_data.persons_outbox")
            .fetch_one(&pool).await.expect("diagnostic Persons outbox"),
    );
    pool.close().await;
    values
}

async fn assert_no_persons_terminal_v1(results: &mut async_nats::Subscriber) {
    assert!(
        tokio::time::timeout(Duration::from_millis(300), results.next())
            .await
            .is_err(),
        "PageCompleted before its source must not emit a Persons terminal",
    );
}

async fn assert_workflow_results_v1(results: &mut async_nats::Subscriber) {
    let mut names = Vec::new();
    while names.len() < 2 {
        let message = tokio::time::timeout(Duration::from_secs(15), results.next())
            .await
            .expect("workflow result timeout")
            .expect("workflow result delivery");
        let envelope =
            DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode workflow result");
        names.push(envelope.contract.expect("workflow result contract").name);
    }
    names.sort();
    assert_eq!(
        names,
        vec![
            "mail_persons_sync_page_receipt".to_owned(),
            "mail_persons_sync_run_result".to_owned(),
        ]
    );
}

pub(super) fn scheduler_due_v1(now: i64, run_id: [u8; 16], account_public_id: [u8; 16]) -> Vec<u8> {
    const LEASE_MILLIS: i64 = 300_000;
    let scope = account_public_id
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let payload = ScheduledJobCommandV1 {
        job_run_id: run_id.to_vec(),
        job_kind: Some(JobKindV1 {
            owner: "mail_persons_sync".to_owned(),
            name: "scheduled_sync".to_owned(),
            major: 1,
        }),
        schedule_id: Sha256::digest(
            [b"mail-persons-sync-schedule-v1".as_slice(), &run_id].concat(),
        )[..16]
            .to_vec(),
        schedule_revision: 1,
        scope_id: scope.clone(),
        trigger_kind: JobTriggerKindV1::Time as i32,
        scheduled_for_unix_millis: now,
        lease: Some(JobLeaseV1 {
            run_id: run_id.to_vec(),
            epoch: 1,
            expires_at_unix_millis: now + LEASE_MILLIS,
        }),
    };
    let message_id: [u8; 16] = Sha256::digest(
        [
            b"mail-persons-sync-scheduler-message-v1".as_slice(),
            &run_id,
        ]
        .concat(),
    )[..16]
        .try_into()
        .expect("scheduler message digest prefix");
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: "mail_persons_sync".to_owned(),
            name: "scheduled_sync".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: SCHEDULER_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: vec![0x42; 16],
            runtime_generation: 1,
        }),
        recorded_at: Some(Timestamp {
            seconds: now / 1_000,
            nanos: ((now % 1_000) * 1_000_000) as i32,
        }),
        partition_key: scope.as_bytes().to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System as i32,
            actor_id: SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: 1,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: run_id.to_vec(),
            target_capability: "job_execute".to_owned(),
            idempotency_key: vec![0x43; 32],
            deadline: Some(Timestamp {
                seconds: (now + LEASE_MILLIS) / 1_000,
                nanos: (((now + LEASE_MILLIS) % 1_000) * 1_000_000) as i32,
            }),
            logical_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    }
    .encode_to_vec()
}

pub(super) async fn publish_exact_v1(client: &async_nats::Client, bytes: &[u8]) {
    let envelope = DurableEnvelopeV1::decode(bytes).expect("decode publish envelope");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive exact subject")
        .as_str();
    async_nats::jetstream::new(client.clone())
        .publish(subject, bytes.to_vec().into())
        .await
        .expect("publish exact envelope")
        .await
        .expect("acknowledge exact durable publish");
    client.flush().await.expect("flush exact envelope");
}
