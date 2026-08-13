use super::*;

use std::time::Instant;

use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};
use makosh_blob_client::BlobDataClient;
use makosh_communications_ai_source_api::{
    communication_explanation_source_prepare_contract_reference_v1,
    communication_explanation_source_prepared_contract_reference_v1,
    communication_explanation_source_rejected_contract_reference_v1,
    communication_reply_source_prepare_contract_reference_v1,
    communication_reply_source_prepared_contract_reference_v1,
    communication_reply_source_rejected_contract_reference_v1,
    communication_summary_source_prepare_contract_reference_v1,
    communication_summary_source_prepared_contract_reference_v1,
    communication_summary_source_rejected_contract_reference_v1,
    communication_translation_source_prepare_contract_reference_v1,
    communication_translation_source_prepared_contract_reference_v1,
    communication_translation_source_rejected_contract_reference_v1,
};
use makosh_communications_api::query_wire::{
    CommunicationsQueryRequestV1, CommunicationsQueryResponseV1, GetEvidenceRequestV1,
    GetMessageRequestV1, ListAccountsRequestV1, ListMessageEvidenceRequestV1,
    SearchCommunicationsRequestV1, communications_query_request_v1::Operation,
    communications_query_response_v1::Result as QueryResult,
};
use makosh_communications_attachment_contract::admission::{
    communication_attachment_anchor_recorded_contract_reference_v1,
    communication_attachment_blob_admission_observed_contract_reference_v1,
    communication_attachment_safety_state_changed_contract_reference_v1,
    communication_attachment_safety_verdict_observed_contract_reference_v1,
};
use makosh_communications_call_evidence_api::{
    CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1, CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
    CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1, CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1,
    CALL_EVIDENCE_QUERY_CONNECT_PATH_V1, CALL_EVIDENCE_QUERY_CONTRACT_NAME_V1,
    CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1,
};
use makosh_communications_call_evidence_ingress::call_evidence_observed_contract_reference_v1;
use makosh_communications_content_api::{
    COMMUNICATIONS_CONTENT_READ_SCHEMA_SHA256, COMMUNICATIONS_CONTENT_TICKET_SCHEMA_SHA256,
    CONTENT_CONTRACT_MAJOR_V1, CONTENT_CONTRACT_REVISION_V1, CONTENT_READ_BLOB_PATH_V1,
    CONTENT_READ_CONTRACT_NAME_V1, CONTENT_TICKET_CONNECT_PATH_V1, CONTENT_TICKET_CONTRACT_NAME_V1,
    MAX_MESSAGE_BODY_BYTES_V1,
};
use makosh_communications_cross_channel_forward_source_api::{
    cross_channel_forward_source_prepare_contract_reference_v1,
    cross_channel_forward_source_prepared_contract_reference_v1,
    cross_channel_forward_source_rejected_contract_reference_v1,
};
use makosh_communications_evidence_export_source_api::{
    evidence_export_prepare_contract_reference_v1, evidence_export_prepared_contract_reference_v1,
    evidence_export_rejected_contract_reference_v1,
};
use makosh_communications_export_api::{
    COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1, COMMUNICATIONS_EXPORT_COMMAND_CONNECT_PATH_V1,
    COMMUNICATIONS_EXPORT_COMMAND_CONTRACT_NAME_V1, COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1,
    COMMUNICATIONS_EXPORT_CONTRACT_REVISION_V1, COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1,
    COMMUNICATIONS_EXPORT_MODULE_ID_V1, COMMUNICATIONS_EXPORT_OWNER_V1,
    COMMUNICATIONS_EXPORT_QUERY_CONNECT_PATH_V1, COMMUNICATIONS_EXPORT_QUERY_CONTRACT_NAME_V1,
    COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1, COMMUNICATIONS_EXPORT_READ_CONTRACT_NAME_V1,
    COMMUNICATIONS_EXPORT_REALTIME_CONTRACT_NAME_V1, COMMUNICATIONS_EXPORT_SCHEMA_SHA256,
    COMMUNICATIONS_EXPORT_TICKET_CONNECT_PATH_V1, COMMUNICATIONS_EXPORT_TICKET_CONTRACT_NAME_V1,
};
use makosh_communications_export_persistence::schema::{
    COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3, communications_export_storage_bundle_v1,
};
use makosh_communications_export_runtime::admission::{
    COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1, COMMUNICATIONS_EXPORT_BLOB_CUSTODY_SCOPE_ID_V1,
    COMMUNICATIONS_EXPORT_BLOB_QUOTA_BYTES_V1, COMMUNICATIONS_EXPORT_EVENTS_CAPABILITY_ID_V1,
    COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1, communications_export_module_descriptor_v1,
    communications_export_settings_schema_bytes_v1,
};
use makosh_communications_note_source_api::{
    communication_note_source_prepare_contract_reference_v1,
    communication_note_source_prepared_contract_reference_v1,
    communication_note_source_rejected_contract_reference_v1,
};
use makosh_communications_recipient_source_api::{
    communication_recipient_source_prepare_contract_reference_v1,
    communication_recipient_source_prepared_contract_reference_v1,
    communication_recipient_source_rejected_contract_reference_v1,
};
use makosh_communications_retained_evidence_replay_contract::{
    communications_replay_command_contract_reference_v1,
    communications_replay_result_contract_reference_v1,
};
use makosh_communications_runtime::admission::{
    COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID, COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID,
    COMMUNICATIONS_ATTACHMENT_BLOB_ADMISSION_OBSERVE_CAPABILITY_ID,
    COMMUNICATIONS_ATTACHMENT_SAFETY_VERDICT_OBSERVE_CAPABILITY_ID,
    COMMUNICATIONS_BLOB_CAPABILITY_ID, COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
    COMMUNICATIONS_BLOB_QUOTA_BYTES, COMMUNICATIONS_CALL_EVIDENCE_OBSERVE_CAPABILITY_ID,
    COMMUNICATIONS_CONTENT_CAPABILITY_ID,
    COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID,
    COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID, COMMUNICATIONS_EVENTS_CAPABILITY_ID,
    COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID,
    COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID,
    COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID, COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID,
    COMMUNICATIONS_MODULE_ID, COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID,
    COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID, COMMUNICATIONS_OBSERVE_CAPABILITY_ID,
    COMMUNICATIONS_OWNER_ID, COMMUNICATIONS_QUERY_CAPABILITY_ID,
    COMMUNICATIONS_RECIPIENT_SOURCE_BLOB_CAPABILITY_ID,
    COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID,
    COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID,
    COMMUNICATIONS_SAVED_SEARCH_CAPABILITY_ID, COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID,
    COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION, COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS,
    COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID, COMMUNICATIONS_SENDER_INSIGHTS_CAPABILITY_ID,
    COMMUNICATIONS_STORAGE_CAPABILITY_ID, COMMUNICATIONS_SUMMARY_SOURCE_BLOB_CAPABILITY_ID,
    COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID, COMMUNICATIONS_TASK_SOURCE_BLOB_CAPABILITY_ID,
    COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID, COMMUNICATIONS_TRANSLATION_SOURCE_BLOB_CAPABILITY_ID,
    COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID,
    communication_evidence_recorded_contract_reference_v1, communications_module_descriptor_v1,
    communications_query_contract_reference_v1, communications_settings_schema_bytes_v1,
};
use makosh_communications_runtime::query_client_port::encode_module_query_request_v1;
use makosh_communications_runtime::storage_bundle::communications_runtime_storage_bundle_v1;
use makosh_communications_saved_query_api::{
    COMMUNICATIONS_SAVED_SEARCH_SCHEMA_SHA256, SAVED_SEARCH_CONNECT_PATH_V1,
    SAVED_SEARCH_CONTRACT_MAJOR_V1, SAVED_SEARCH_CONTRACT_NAME_V1,
    SAVED_SEARCH_CONTRACT_REVISION_V1,
};
use makosh_communications_sender_insights_api::{
    COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256, SENDER_INSIGHTS_CONNECT_PATH_V1,
    SENDER_INSIGHTS_CONTRACT_MAJOR_V1, SENDER_INSIGHTS_CONTRACT_NAME_V1,
    SENDER_INSIGHTS_CONTRACT_REVISION_V1,
};
use makosh_communications_task_source_api::{
    communication_task_source_prepare_contract_reference_v1,
    communication_task_source_prepared_contract_reference_v1,
    communication_task_source_rejected_contract_reference_v1,
};
use makosh_kernel_control_store::{
    ModuleBlobOperationV1, ModuleBlobQuotaRequestV1, ModuleClientBlobContractVersionV1,
    ModuleClientBlobRouteV1, ModuleClientBlobTransportV1, ModuleClientRealtimeContractVersionV1,
    ModuleClientRealtimeRouteV1, ModuleClientRpcContractVersionV1, ModuleClientRpcRouteV1,
    ModuleDescriptorRegistrationRequestsV1, ModuleRegistrationState, ModuleVaultPurposePolicyV1,
    ModuleVaultPurposeRequestV1, PlatformStorageBindingStateV1,
};
use makosh_runtime_protocol::v1::{
    BlobDataOperationV1, ManagedRuntimeBlobSessionRequestV1, ModuleClientResponseV1, VaultActionV1,
    VaultSecretClassV1, VaultTargetScopeV1,
};

pub(super) const COMMUNICATIONS_REGISTRATION: &str = "communications-runtime";
pub(super) const COMMUNICATIONS_EXPORT_REGISTRATION: &str = "communications-export-runtime";
const COMMUNICATIONS_RUNTIME_INSTANCE_ID: &str = "02020202020202020202020202020202";
const COMMUNICATIONS_EXPORT_RUNTIME_INSTANCE_ID: &str = "06060606060606060606060606060606";
const COMMUNICATIONS_RUNTIME_INSTANCE_ID_V2: &str = "05050505050505050505050505050505";
const FIXTURE_SOURCE_REGISTRATION: &str = "fixture-source-integration";
const FIXTURE_SOURCE_CAPABILITY_ID: &str = "fixture-source.blob.v1";
const FIXTURE_SOURCE_RUNTIME_INSTANCE_ID: &str = "03030303030303030303030303030303";
const FIXTURE_SOURCE_RUNTIME_INSTANCE_ID_V2: &str = "04040404040404040404040404040404";
const EVENT_HUB_MAX_PULL_WAITING_V1: i64 = 512;

pub(super) fn configured_communications_store(root: &Path, kernel: &Path) -> SqliteControlStore {
    let store = configured_store(root, kernel);
    crate::platform::blob::binding::bind_installed_release(&store, kernel)
        .expect("bind signed Blob release");
    let schema = communications_settings_schema_bytes_v1();
    let descriptor =
        communications_module_descriptor_v1("managed-communications-live").encode_to_vec();
    let grant_epoch = record_communications_registration(&store, &descriptor);
    record_communications_runtime_fixture(&store, &schema, &descriptor, grant_epoch);
    record_communications_export_runtime_fixture(&store);
    store
}

pub(super) fn issue_initial_communications_storage_binding(store: &SqliteControlStore) {
    let runtime_bundle =
        communications_runtime_storage_bundle_v1().expect("compose Communications Storage bundle");
    let bundle = store
        .platform_storage_bundle("communications", u64::from(runtime_bundle.revision))
        .expect("read Communications Storage bundle")
        .expect("Communications Storage bundle is present");
    let binding = issue_managed(
        store,
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_RUNTIME_INSTANCE_ID,
        1,
        COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(1, 1, u64::from(runtime_bundle.revision), *bundle.digest())
            .expect("initial Communications Storage issue"),
    )
    .expect("issue Communications Storage binding");
    assert_eq!(binding.runtime_generation(), 1);
}

pub(super) fn issue_initial_communications_export_storage_binding(store: &SqliteControlStore) {
    let bundle = store
        .platform_storage_bundle(
            COMMUNICATIONS_EXPORT_OWNER_V1,
            u64::from(COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3),
        )
        .expect("read Communications Export Storage bundle")
        .expect("Communications Export Storage bundle is present");
    issue_managed(
        store,
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_RUNTIME_INSTANCE_ID,
        1,
        COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3),
            *bundle.digest(),
        )
        .expect("initial Communications Export Storage issue"),
    )
    .expect("issue Communications Export Storage binding");
}

pub(super) fn communications_export_storage_binding(
    store: &SqliteControlStore,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            COMMUNICATIONS_EXPORT_REGISTRATION,
            COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Communications Export Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Communications Export Storage binding")
}

pub(super) fn communications_storage_binding(
    store: &SqliteControlStore,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        )
        .expect("read Communications Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Communications Storage binding")
}

pub(super) fn configure_communications_jetstream(store: &SqliteControlStore) {
    configure_communications_jetstream_with_limits(store, None);
}

pub(super) fn record_communications_event_hub_topology_v1(store: &SqliteControlStore) {
    store
        .record_platform_event_hub_topology(&communications_event_hub_topology())
        .expect("record Event Hub topology");
}

pub(super) fn configure_communications_jetstream_for_retained_replay_test(
    store: &SqliteControlStore,
) {
    configure_communications_jetstream_with_limits(
        store,
        Some((Duration::ZERO, Duration::from_millis(100))),
    );
}

fn configure_communications_jetstream_with_limits(
    store: &SqliteControlStore,
    replay_limits: Option<(Duration, Duration)>,
) {
    let configuration = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let contracts = event_catalog::resolve_contracts(store).expect("resolve Event Hub contracts");
    let plan = event_topology::plan(&contracts, &configuration).expect("plan Event Hub topology");
    let endpoint = configuration.nats_endpoint().to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(&endpoint)
                    .await
                    .expect("connect JetStream"),
            );
            for stream in plan.streams() {
                let (name, subject) = communications_stream_details(stream.kind());
                let (max_age, duplicate_window) = replay_limits.unwrap_or_default();
                context
                    .create_stream(async_nats::jetstream::stream::Config {
                        name: name.to_owned(),
                        subjects: vec![subject.to_owned()],
                        max_age,
                        duplicate_window,
                        ..Default::default()
                    })
                    .await
                    .expect("create Communications Event stream");
            }
            for consumer in plan.consumers() {
                let subject = consumer.subject().as_str();
                let stream_name = communications_stream_for_subject(&subject);
                let ack_wait =
                    Duration::from_millis(consumer.delivery_policy().ack_wait_millis().into());
                let max_deliver = i64::from(consumer.delivery_policy().max_deliver());
                let max_ack_pending = i64::from(consumer.max_in_flight());
                context
                    .create_consumer_on_stream(
                        async_nats::jetstream::consumer::pull::Config {
                            durable_name: Some(consumer.durable_name().to_owned()),
                            name: Some(consumer.durable_name().to_owned()),
                            deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::All,
                            filter_subject: subject,
                            ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                            ack_wait,
                            max_deliver,
                            max_waiting: EVENT_HUB_MAX_PULL_WAITING_V1,
                            max_ack_pending,
                            max_batch: max_ack_pending,
                            max_expires: ack_wait,
                            inactive_threshold: Duration::ZERO,
                            num_replicas: 1,
                            memory_storage: false,
                            replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
                            backoff: (0..max_deliver)
                                .scan(ack_wait, |delay, _| {
                                    let current = *delay;
                                    *delay = delay.saturating_mul(2).min(Duration::from_secs(600));
                                    Some(current)
                                })
                                .collect(),
                            ..Default::default()
                        },
                        stream_name,
                    )
                    .await
                    .expect("create Communications Event consumer");
            }
        });
}

pub(super) fn wait_for_communications_jetstream_subject_expiry(
    store: &SqliteControlStore,
    subject: &str,
) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let stream_name = communications_stream_for_subject(subject).to_owned();
    let subject = subject.to_owned();
    tokio::runtime::Runtime::new()
        .expect("retained evidence expiry runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect retained evidence expiry observer"),
            );
            let stream = context
                .get_stream(stream_name)
                .await
                .expect("read retained evidence stream");
            let mut expiring = stream.cached_info().config.clone();
            expiring.max_age = Duration::from_millis(250);
            context
                .update_stream(expiring.clone())
                .await
                .expect("apply retained evidence expiry window");
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if stream
                    .get_last_raw_message_by_subject(&subject)
                    .await
                    .is_err()
                {
                    expiring.max_age = Duration::ZERO;
                    context
                        .update_stream(expiring)
                        .await
                        .expect("restore retained evidence stream retention");
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "retained evidence subject did not expire from JetStream"
                );
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        });
}

pub(super) fn start_communications_domain(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> u64 {
    let reservation = managed_launch::load(supervisor, store, COMMUNICATIONS_REGISTRATION)
        .expect("load Communications reservation");
    let binding = communications_storage_binding(store);
    start_reserved_communications_domain(supervisor, store, runtime_dir, reservation, binding)
}

pub(super) fn start_communications_export_workflow(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> u64 {
    let reservation = managed_launch::load(supervisor, store, COMMUNICATIONS_EXPORT_REGISTRATION)
        .expect("load Communications Export reservation");
    let binding = communications_export_storage_binding(store);
    start_reserved_communications_export_workflow(
        supervisor,
        store,
        runtime_dir,
        reservation,
        binding,
    )
}

pub(super) fn restart_communications_export_workflow(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> u64 {
    let predecessor = communications_export_storage_binding(store);
    let issue = storage_successor::issue_after(&predecessor)
        .expect("derive Communications Export successor fences");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve successor Communications Export launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Communications Export Storage binding");
    start_reserved_communications_export_workflow(
        supervisor,
        store,
        runtime_dir,
        reservation,
        binding,
    )
}

fn start_reserved_communications_export_workflow(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    binding: makosh_kernel_control_store::PlatformStorageBindingV1,
) -> u64 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Communications Export Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let generation = managed_launch::start_reserved_workflow(
        supervisor,
        runtime_dir,
        reservation,
        makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: "owner-1".to_owned(),
            registration_id: COMMUNICATIONS_EXPORT_REGISTRATION.to_owned(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            runtime_artifacts: Vec::new(),
            configuration_instance_id: String::new(),
            settings_revision: 0,
            configuration_instances: Vec::new(),
        },
        &[],
    )
    .expect("start Communications Export workflow");
    supervisor
        .wait_until_ready(COMMUNICATIONS_EXPORT_REGISTRATION)
        .expect("wait for Communications Export readiness");
    generation
}

pub(super) fn restart_communications_domain(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> u64 {
    let predecessor = communications_storage_binding(store);
    let issue = storage_successor::issue_after(&predecessor)
        .expect("derive Communications successor fences");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor Communications launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Communications Storage binding");
    start_reserved_communications_domain(supervisor, store, runtime_dir, reservation, binding)
}

fn start_reserved_communications_domain(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    binding: makosh_kernel_control_store::PlatformStorageBindingV1,
) -> u64 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Communications Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let generation = managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: COMMUNICATIONS_OWNER_ID.to_owned(),
            registration_id: COMMUNICATIONS_REGISTRATION.to_owned(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            logical_human_owner_id: "owner-1".to_owned(),
        },
    )
    .expect("start Communications domain");
    if let Err(error) = supervisor.wait_until_ready(COMMUNICATIONS_REGISTRATION) {
        panic!(
            "wait for Communications readiness: {error}; last_failure={:?}",
            supervisor.last_failure(COMMUNICATIONS_REGISTRATION),
        );
    }
    generation
}

pub(super) fn assert_communications_query_delivery(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) -> Vec<u8> {
    let payload = CommunicationsQueryRequestV1 {
        protocol_major: 1,
        operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
            limit: 16,
            cursor: Vec::new(),
        })),
    }
    .encode_to_vec();
    let query = route_communications_query(store, supervisor, 1, &payload);
    let Some(QueryResult::ListAccounts(accounts)) = query.result else {
        panic!("Communications accounts query result");
    };
    let evidence_id = accounts
        .accounts
        .first()
        .expect("Communications account projection")
        .last_evidence_id
        .clone();
    let payload = CommunicationsQueryRequestV1 {
        protocol_major: 1,
        operation: Some(Operation::GetEvidence(GetEvidenceRequestV1 { evidence_id })),
    }
    .encode_to_vec();
    let evidence = route_communications_query(store, supervisor, 17, &payload);
    let Some(QueryResult::GetEvidence(response)) = evidence.result else {
        panic!("Communications evidence query result");
    };
    let evidence = response.evidence.expect("Communications evidence metadata");
    assert_eq!(evidence.evidence_id.len(), 16);
    assert_eq!(evidence.correlation_id.len(), 16);
    assert!(evidence.causation_message_id.is_empty());
    assert!(evidence.recorded_at_unix_seconds > 0);
    assert!((0..1_000_000_000).contains(&evidence.recorded_at_nanos));
    evidence.evidence_id
}

pub(super) fn assert_communications_module_query_delivery(supervisor: &ManagedRuntimeSupervisor) {
    let payload = CommunicationsQueryRequestV1 {
        protocol_major: 1,
        operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
            limit: 16,
            cursor: Vec::new(),
        })),
    }
    .encode_to_vec();
    let request_id = vec![19; 16];
    let bytes = supervisor
        .relay(
            COMMUNICATIONS_REGISTRATION,
            makosh_runtime_protocol::v1::ManagedRuntimeControlRequestV1 {
                operation: Some(
                    makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::DeliverModuleQuery(
                        makosh_runtime_protocol::v1::ManagedRuntimeModuleQueryDeliveryV1 {
                            request_id: request_id.clone(),
                            logical_owner_id: "owner_local".to_owned(),
                            contract: Some(communications_query_contract_reference_v1()),
                            request_payload: payload,
                        },
                    ),
                ),
            }
            .encode_to_vec(),
        )
        .expect("deliver live Communications module query");
    let response =
        makosh_runtime_protocol::v1::ManagedRuntimeControlResponseV1::decode(bytes.as_slice())
            .expect("decode module query delivery response");
    let Some(
        makosh_runtime_protocol::v1::managed_runtime_control_response_v1::Result::ModuleQueryDelivery(
            response,
        ),
    ) = response.result
    else {
        panic!("Communications module query delivery result");
    };
    assert_eq!(response.request_id, request_id);
    assert!(response.error_code.is_empty());
    let query = CommunicationsQueryResponseV1::decode(response.response_payload.as_slice())
        .expect("decode Communications module query response");
    assert!(matches!(query.result, Some(QueryResult::ListAccounts(_))));
}

pub(super) fn assert_communications_canonical_read_v2_pagination(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let first_page = route_communications_query(
        store,
        supervisor,
        20,
        &CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                limit: 1,
                cursor: Vec::new(),
            })),
        }
        .encode_to_vec(),
    );
    let Some(QueryResult::ListAccounts(first_page)) = first_page.result else {
        panic!("Communications first canonical account page");
    };
    assert_eq!(first_page.accounts.len(), 1);
    assert!(
        !first_page.next_cursor.is_empty(),
        "multiple canonical accounts must expose an opaque continuation",
    );

    let second_page = route_communications_query(
        store,
        supervisor,
        21,
        &CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                limit: 1,
                cursor: first_page.next_cursor,
            })),
        }
        .encode_to_vec(),
    );
    let Some(QueryResult::ListAccounts(second_page)) = second_page.result else {
        panic!("Communications second canonical account page");
    };
    assert_eq!(second_page.accounts.len(), 1);
    assert_ne!(
        first_page.accounts[0].account_id, second_page.accounts[0].account_id,
        "keyset continuation must not repeat the boundary row",
    );
}

pub(super) fn assert_communications_search_query_delivery(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let missing_payload = CommunicationsQueryRequestV1 {
        protocol_major: 1,
        operation: Some(Operation::SearchCommunications(
            SearchCommunicationsRequestV1 {
                query: "known-missing-token".to_owned(),
                limit: 16,
                cursor: Vec::new(),
            },
        )),
    }
    .encode_to_vec();
    let query = route_communications_query(store, supervisor, 2, &missing_payload);
    assert!(
        matches!(query.result, Some(QueryResult::SearchCommunications(hits)) if hits.hits.is_empty())
    );

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let payload = CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::SearchCommunications(
                SearchCommunicationsRequestV1 {
                    query: "fixture".to_owned(),
                    limit: 16,
                    cursor: Vec::new(),
                },
            )),
        }
        .encode_to_vec();
        let query = route_communications_query(store, supervisor, 16, &payload);
        let Some(QueryResult::SearchCommunications(hits)) = query.result else {
            panic!("Communications search query result");
        };
        if !hits.hits.is_empty() {
            assert!(hits.hits.iter().all(|hit| {
                hit.evidence_id.len() == 16
                    && hit.message_id.len() == 16
                    && hit.conversation_id.len() == 16
                    && hit.matched_token_count > 0
            }));
            let hit = hits.hits.first().expect("canonical search hit").clone();
            let detail = route_communications_query(
                store,
                supervisor,
                22,
                &CommunicationsQueryRequestV1 {
                    protocol_major: 1,
                    operation: Some(Operation::GetMessage(GetMessageRequestV1 {
                        message_id: hit.message_id.clone(),
                    })),
                }
                .encode_to_vec(),
            );
            let Some(QueryResult::GetMessage(detail)) = detail.result else {
                panic!("Communications exact message detail result");
            };
            let message = detail.message.expect("canonical search message detail");
            assert_eq!(message.message_id, hit.message_id);
            assert_eq!(message.conversation_id, hit.conversation_id);

            let history = route_communications_query(
                store,
                supervisor,
                23,
                &CommunicationsQueryRequestV1 {
                    protocol_major: 1,
                    operation: Some(Operation::ListMessageEvidence(
                        ListMessageEvidenceRequestV1 {
                            message_id: message.message_id,
                            limit: 1,
                            cursor: Vec::new(),
                        },
                    )),
                }
                .encode_to_vec(),
            );
            let Some(QueryResult::ListMessageEvidence(history)) = history.result else {
                panic!("Communications exact message evidence history result");
            };
            assert_eq!(history.evidence.len(), 1);
            assert_eq!(history.evidence[0].evidence_id.len(), 16);
            let public_payload = CommunicationsQueryResponseV1 {
                result: Some(QueryResult::SearchCommunications(hits)),
                error_code: String::new(),
            }
            .encode_to_vec();
            for private_value in [
                "fixture source body for custody transfer",
                "blob://fixture-source/admitted-body-1",
            ] {
                assert!(
                    !public_payload
                        .windows(private_value.len())
                        .any(|window| window == private_value.as_bytes()),
                    "public Communications search must not reveal private body or Blob locator",
                );
            }
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "transferred Communications body was not indexed",
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn assert_fenced_communications_target_cannot_issue_blob_custody_grant(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    kernel_data: &Path,
) {
    let launch = store
        .effective_managed_launch_record(COMMUNICATIONS_REGISTRATION)
        .expect("read Communications launch")
        .expect("Communications launch is active");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_RUNTIME_INSTANCE_ID_V2,
            1,
            1,
            launch.runtime_generation() + 1,
            launch.grant_epoch(),
        ))
        .expect("record Communications successor launch");
    let request = ManagedRuntimeBlobSessionRequestV1 {
        request_id: vec![7; 16],
        capability_id: COMMUNICATIONS_BLOB_CAPABILITY_ID.to_owned(),
        operation: BlobDataOperationV1::BlobDataOperationCustodyTransferV1 as u32,
        channel_binding_sha256: vec![8; 32],
        reference_id: vec![9; 16],
        declared_size: 1,
        backup_class: 1,
        ttl_seconds: 30,
        receipt_sha256: vec![10; 32],
        custody_source_proof: vec![11],
        evidence_id: vec![12; 16],
        evidence_envelope_sha256: vec![13; 32],
        custody_target_owner_id: String::new(),
        custody_target_module_id: String::new(),
        custody_target_capability_id: String::new(),
    };
    let handler = BlobSessionHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
        kernel_data.to_path_buf(),
    );
    let result = handler.issue_blob_session(
        &ManagedRuntimeExpectation::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_RUNTIME_INSTANCE_ID,
            COMMUNICATIONS_MODULE_ID,
            launch.runtime_generation(),
            launch.grant_epoch(),
            [2; 32],
            None,
        ),
        request.clone(),
    );
    assert!(
        result.is_err(),
        "stale Communications target must not receive a Blob custody grant",
    );
    let retained_capabilities = store
        .module_grant_snapshot(COMMUNICATIONS_REGISTRATION)
        .expect("read Communications grant snapshot")
        .expect("Communications grant snapshot")
        .effective_grants()
        .expect("approved Communications grants")
        .capability_ids()
        .iter()
        .filter(|capability| capability.as_str() != COMMUNICATIONS_BLOB_CAPABILITY_ID)
        .cloned()
        .collect::<Vec<_>>();
    let suspended = store
        .transition_module_registration(
            COMMUNICATIONS_REGISTRATION,
            ModuleRegistrationState::Suspended,
        )
        .expect("suspend Communications target before grant replacement");
    let reapproved = store
        .approve_module_registration(COMMUNICATIONS_REGISTRATION, &retained_capabilities)
        .expect("reapprove Communications without Blob capability");
    assert!(
        reapproved.grant_epoch() > suspended.grant_epoch(),
        "grant replacement must advance the Communications grant epoch",
    );
    const GRANT_REVOKED_RUNTIME_INSTANCE_ID: &str =
        "communications-runtime-instance-blob-grant-revoked";
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            COMMUNICATIONS_REGISTRATION,
            GRANT_REVOKED_RUNTIME_INSTANCE_ID,
            1,
            1,
            launch.runtime_generation() + 2,
            reapproved.grant_epoch(),
        ))
        .expect("record Communications launch without Blob capability");
    let grant_revoked = handler.issue_blob_session(
        &ManagedRuntimeExpectation::new(
            COMMUNICATIONS_REGISTRATION,
            GRANT_REVOKED_RUNTIME_INSTANCE_ID,
            COMMUNICATIONS_MODULE_ID,
            launch.runtime_generation() + 2,
            reapproved.grant_epoch(),
            [2; 32],
            None,
        ),
        request.clone(),
    );
    assert!(
        grant_revoked.is_err(),
        "current Communications target without Blob capability must not receive a custody grant",
    );
    store
        .transition_module_registration(
            COMMUNICATIONS_REGISTRATION,
            ModuleRegistrationState::Revoked,
        )
        .expect("revoke current Communications target");
    let revoked = handler.issue_blob_session(
        &ManagedRuntimeExpectation::new(
            COMMUNICATIONS_REGISTRATION,
            GRANT_REVOKED_RUNTIME_INSTANCE_ID,
            COMMUNICATIONS_MODULE_ID,
            launch.runtime_generation() + 2,
            reapproved.grant_epoch(),
            [2; 32],
            None,
        ),
        request,
    );
    assert!(
        revoked.is_err(),
        "revoked Communications target must not receive a Blob custody grant",
    );
}

pub(super) fn assert_communications_ingress_delivery(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let draft = makosh_mail_core::draft_ingress_observation_with_sender_subject_body(
        "managed-ingress-observation-1",
        makosh_communications_ingress::ProviderProvenanceV1::MailImap,
        "integration-private-account-1",
        "integration-private-record-1",
        Some("Fixture Sender <sender@example.test>".to_owned()),
        Some("Managed ingress subject".to_owned()),
        makosh_communications_ingress::BodyAvailabilityV1::MetadataOnly,
    )
    .expect("build typed Mail ingress draft");
    let record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "mail-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "makosh-mail-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build exact typed integration envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            use futures_util::StreamExt as _;

            let client = async_nats::connect(endpoint)
                .await
                .expect("connect disposable JetStream");
            let mut canonical_events = client
                .subscribe("makosh.event.v1.communications.communication_evidence_recorded.v1")
                .await
                .expect("subscribe to exact canonical event subject");
            let context = async_nats::jetstream::new(client);
            context
                .publish(
                    "makosh.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish exact typed integration envelope")
                .await
                .expect("acknowledge exact typed integration envelope");
            let canonical =
                tokio::time::timeout(std::time::Duration::from_secs(5), canonical_events.next())
                    .await
                    .unwrap_or_else(|_| {
                        panic!(
                            "canonical Communications event timeout: active={:?}, failure={:?}",
                            supervisor.is_active(COMMUNICATIONS_REGISTRATION),
                            supervisor.last_failure(COMMUNICATIONS_REGISTRATION),
                        )
                    })
                    .expect("canonical Communications event missing");
            let envelope = makosh_events_protocol::validation::envelope::decode_envelope_v1(
                canonical.payload.as_ref(),
            )
            .expect("canonical Communications envelope");
            let ingress = makosh_events_protocol::validation::envelope::decode_envelope_v1(
                record.exact_bytes(),
            )
            .expect("typed integration envelope");
            assert!(matches!(
                envelope.contract.as_ref(),
                Some(contract)
                    if contract.owner == "communications"
                        && contract.name == "communication_evidence_recorded"
                        && contract.major == 1
                        && contract.revision == 2
            ));
            assert_eq!(envelope.causation_message_id, record.message_id().to_vec());
            assert_eq!(envelope.correlation_id, ingress.correlation_id);
            let payload = makosh_communications_api::wire::CommunicationEvidenceRecordedV1::decode(
                envelope.payload.as_slice(),
            )
            .expect("canonical Communications payload");
            assert_eq!(
                payload.message_subject.as_deref(),
                Some("Managed ingress subject")
            );
            context
                .publish(
                    "makosh.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("republish exact typed integration envelope")
                .await
                .expect("acknowledge republished exact typed integration envelope");
            assert!(
                tokio::time::timeout(std::time::Duration::from_secs(1), canonical_events.next(),)
                    .await
                    .is_err(),
                "duplicate ingress must not produce a second canonical event"
            );
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let payload = CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                limit: 16,
                cursor: Vec::new(),
            })),
        }
        .encode_to_vec();
        let query = route_communications_query(store, supervisor, 3, &payload);
        if matches!(query.result, Some(QueryResult::ListAccounts(accounts)) if !accounts.accounts.is_empty())
        {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "typed integration ingress was not committed to Communications"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn managed_mail_target_conversation(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) -> Vec<u8> {
    let account_cursor = fixture_account_cursor(
        makosh_communications_ingress::ProviderProvenanceV1::MailImap,
        "integration-private-account-1",
    );
    let accounts = route_communications_query(
        store,
        supervisor,
        72,
        &CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                limit: 16,
                cursor: Vec::new(),
            })),
        }
        .encode_to_vec(),
    );
    let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
        panic!("managed Mail accounts query result");
    };
    let account = accounts
        .accounts
        .into_iter()
        .find(|account| account.account_cursor_sha256 == account_cursor)
        .expect("managed Mail account");
    let conversations = route_communications_query(
        store,
        supervisor,
        73,
        &CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::ListConversations(
                makosh_communications_api::query_wire::ListConversationsRequestV1 {
                    account_cursor_sha256: account.account_cursor_sha256,
                    limit: 16,
                    cursor: Vec::new(),
                },
            )),
        }
        .encode_to_vec(),
    );
    let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
        panic!("managed Mail conversations query result");
    };
    conversations
        .conversations
        .first()
        .expect("managed Mail conversation")
        .conversation_id
        .clone()
}

pub(super) fn assert_communications_transferred_body_projection(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    kernel_data: &Path,
    kernel: &Path,
    runtime_dir: &Path,
    exercise_recovery_failures: bool,
) -> Vec<u8> {
    assert_communications_transferred_body_projection_with_plaintext(
        store,
        supervisor,
        kernel_data,
        kernel,
        runtime_dir,
        b"fixture source body for custody transfer",
        exercise_recovery_failures,
    )
}

pub(super) fn assert_communications_transferred_body_projection_with_plaintext(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    kernel_data: &Path,
    kernel: &Path,
    runtime_dir: &Path,
    plaintext: &[u8],
    exercise_recovery_failures: bool,
) -> Vec<u8> {
    assert_communications_transferred_body_projection_with_plaintext_and_fixture_id(
        store,
        supervisor,
        kernel_data,
        kernel,
        runtime_dir,
        plaintext,
        exercise_recovery_failures,
        1,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn assert_communications_transferred_body_projection_with_plaintext_and_fixture_id(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    kernel_data: &Path,
    kernel: &Path,
    runtime_dir: &Path,
    plaintext: &[u8],
    exercise_recovery_failures: bool,
    fixture_id: u8,
) -> Vec<u8> {
    assert!(fixture_id > 0, "fixture id must be non-zero");
    let opaque_blob_reference = format!("blob://fixture-source/admitted-body-{fixture_id}");
    let external_account_id = format!("integration-private-body-account-{fixture_id}");
    let external_conversation_id = format!("integration-private-body-conversation-{fixture_id}");
    let external_participant_id = format!("integration-private-body-participant-{fixture_id}");
    let body_account_cursor = fixture_account_cursor(
        makosh_communications_ingress::ProviderProvenanceV1::Telegram,
        &external_account_id,
    );
    let source_grant_epoch = record_fixture_source_integration(store);
    let plaintext_sha256: [u8; 32] = Sha256::digest(plaintext).into();
    let reference_id = [fixture_id; 16];
    let channel_binding = vec![fixture_id.saturating_add(5); 32];
    let delivery = BlobSessionHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
        kernel_data.to_path_buf(),
    )
    .issue_blob_session(
        &ManagedRuntimeExpectation::new(
            FIXTURE_SOURCE_REGISTRATION,
            FIXTURE_SOURCE_RUNTIME_INSTANCE_ID,
            "integration.fixture-source",
            1,
            source_grant_epoch,
            [3; 32],
            None,
        ),
        ManagedRuntimeBlobSessionRequestV1 {
            request_id: vec![fixture_id.saturating_add(3); 16],
            capability_id: FIXTURE_SOURCE_CAPABILITY_ID.to_owned(),
            operation: BlobDataOperationV1::BlobDataOperationWriteV1 as u32,
            channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
            reference_id: reference_id.to_vec(),
            declared_size: u64::try_from(plaintext.len()).expect("fixture body size"),
            backup_class: 1,
            ttl_seconds: 30,
            receipt_sha256: plaintext_sha256.to_vec(),
            custody_source_proof: Vec::new(),
            evidence_id: Vec::new(),
            evidence_envelope_sha256: Vec::new(),
            custody_target_owner_id: String::new(),
            custody_target_module_id: String::new(),
            custody_target_capability_id: String::new(),
        },
    )
    .expect("issue source integration Blob write session");
    let source_proof = delivery.custody_transfer_source_proof;
    BlobDataClient::new(delivery.data_socket_path)
        .expect("open source Blob data client")
        .write(
            delivery.grant.expect("source Blob write grant"),
            channel_binding,
            plaintext.to_vec(),
        )
        .expect("write source integration Blob content");
    let rejected_draft = makosh_communications_ingress::new_scoped_communication_observation_draft(
        format!("managed-rejected-body-observation-{fixture_id}"),
        makosh_communications_ingress::SourceEnvelope {
            provider: makosh_communications_ingress::ProviderProvenanceV1::Telegram,
            external_record_id: format!("integration-private-body-record-rejected-{fixture_id}"),
            scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                external_account_id: external_account_id.clone(),
                external_conversation_id: Some(external_conversation_id.clone()),
                external_participant_id: Some(external_participant_id.clone()),
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        makosh_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
        makosh_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
        makosh_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_000),
    )
    .expect("build rejected admitted-body ingress draft");
    let rejected_draft = makosh_communications_ingress::with_admitted_body_blob(
        rejected_draft,
        makosh_communications_ingress::BodyBlobReceiptV1 {
            blob_ref: opaque_blob_reference.clone(),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
            sha256: [9; 32],
            media_type: "text/plain".to_owned(),
            custody_transfer_source_proof: source_proof.clone(),
        },
    )
    .expect("attach altered opaque Blob receipt");
    let rejected_record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &rejected_draft,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build altered admitted-body typed ingress envelope");
    let draft = makosh_communications_ingress::new_scoped_communication_observation_draft(
        format!("managed-admitted-body-observation-{fixture_id}"),
        makosh_communications_ingress::SourceEnvelope {
            provider: makosh_communications_ingress::ProviderProvenanceV1::Telegram,
            external_record_id: format!("integration-private-body-record-{fixture_id}"),
            scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                external_account_id,
                external_conversation_id: Some(external_conversation_id),
                external_participant_id: Some(external_participant_id),
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        makosh_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
        makosh_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
        makosh_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_001),
    )
    .expect("build admitted-body ingress draft");
    let draft = makosh_communications_ingress::with_participant_display_label(
        draft,
        Some("Alice Example <alice@example.test>".to_owned()),
    )
    .expect("attach admitted-body sender label");
    let draft = makosh_communications_ingress::with_message_subject(
        draft,
        Some("Quarterly update".to_owned()),
    )
    .expect("attach admitted-body subject");
    let draft = makosh_communications_ingress::with_admitted_body_blob(
        draft,
        makosh_communications_ingress::BodyBlobReceiptV1 {
            blob_ref: opaque_blob_reference.clone(),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
            sha256: plaintext_sha256,
            media_type: "text/plain".to_owned(),
            custody_transfer_source_proof: source_proof.clone(),
        },
    )
    .expect("attach admitted opaque Blob receipt");
    let record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_001,
            recorded_at_nanos: 0,
        },
    )
    .expect("build admitted-body typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    if !exercise_recovery_failures {
        tokio::runtime::Runtime::new()
            .expect("Tokio runtime")
            .block_on(async move {
                async_nats::jetstream::new(
                    async_nats::connect(endpoint)
                        .await
                        .expect("connect disposable JetStream"),
                )
                .publish(
                    "makosh.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish admitted-body typed ingress envelope")
                .await
                .expect("acknowledge admitted-body typed ingress envelope");
            });
        return wait_for_transferred_body_message(store, supervisor, &body_account_cursor);
    }
    supervisor
        .stop("vault")
        .expect("stop Vault for custody outage");
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect disposable JetStream"),
            );
            context
                .publish(
                    "makosh.observation.v1.communications.communication_observed.v1",
                    rejected_record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish altered admitted-body typed ingress envelope")
                .await
                .expect("acknowledge altered admitted-body typed ingress envelope");
            context
                .publish(
                    "makosh.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish admitted-body typed ingress envelope")
                .await
                .expect("acknowledge admitted-body typed ingress envelope");
        });

    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("read Communications process state during Vault outage"),
        "a transient Vault outage must not stop the Communications owner runtime",
    );
    assert!(
        supervisor
            .is_active("blob")
            .expect("read Blob process state during Vault outage"),
        "Vault unavailability must fail the Blob key route without stopping Blob",
    );
    supervisor
        .stop("blob")
        .expect("stop Blob before rebinding the successor Vault generation");
    supervisor
        .stop("storage")
        .expect("stop Storage before rebinding the successor Vault generation");
    assert_eq!(
        start_vault(supervisor, store, kernel_data, kernel),
        2,
        "Vault restart uses a successor managed runtime generation",
    );
    assert_eq!(
        start_storage(supervisor, store, kernel, &storage_runtime_directory(),),
        2,
        "Storage restart binds the successor Vault generation",
    );
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("read Communications process state during Blob outage"),
        "pending custody must not stop Communications while Blob remains unavailable",
    );
    assert_eq!(
        crate::platform::blob::launch::start_from_kernel(
            supervisor,
            store,
            kernel,
            kernel_data,
            runtime_dir,
        )
        .expect("restart signed Blob runtime after custody outage"),
        2,
        "Blob restart uses a successor managed runtime generation",
    );
    assert_eq!(
        restart_communications_domain(supervisor, store, runtime_dir),
        2,
        "Communications restart uses a successor managed runtime and Storage generation",
    );

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut stale_source_published = false;
    let mut revoked_source_published = false;
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            4,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                    limit: 16,
                    cursor: Vec::new(),
                })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        let Some(account) = accounts
            .accounts
            .iter()
            .find(|account| account.account_cursor_sha256 == body_account_cursor)
        else {
            assert!(
                std::time::Instant::now() < deadline,
                "admitted body account was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let conversations = route_communications_query(
            store,
            supervisor,
            5,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversations(
                    makosh_communications_api::query_wire::ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
            panic!("Communications conversations query result");
        };
        let Some(conversation) = conversations.conversations.first() else {
            assert!(
                std::time::Instant::now() < deadline,
                "admitted body conversation was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let messages = route_communications_query(
            store,
            supervisor,
            6,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationMessages(
                    makosh_communications_api::query_wire::ListConversationMessagesRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
            panic!("Communications messages query result");
        };
        let transferred = messages
            .messages
            .iter()
            .any(|message| message.body_state == 4);
        let rejected = messages
            .messages
            .iter()
            .filter(|message| message.body_state == 3)
            .count();
        if transferred && rejected >= 1 && !stale_source_published {
            store
                .record_managed_launch(&ManagedLaunchRecord::new(
                    FIXTURE_SOURCE_REGISTRATION,
                    FIXTURE_SOURCE_RUNTIME_INSTANCE_ID_V2,
                    1,
                    1,
                    2,
                    source_grant_epoch,
                ))
                .expect("record fixture source integration successor launch");
            let stale_draft =
                makosh_communications_ingress::new_scoped_communication_observation_draft(
                    "managed-stale-body-observation-1",
                    makosh_communications_ingress::SourceEnvelope {
                        provider: makosh_communications_ingress::ProviderProvenanceV1::Telegram,
                        external_record_id: "integration-private-body-record-stale-1".to_owned(),
                        scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                            external_account_id: "integration-private-body-account-1".to_owned(),
                            external_conversation_id: Some(
                                "integration-private-body-conversation-1".to_owned(),
                            ),
                            external_participant_id: None,
                            external_media_id: None,
                            external_reply_to_record_id: None,
                            external_forward_origin_record_id: None,
                        }),
                    },
                    makosh_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
                    makosh_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
                    makosh_communications_ingress::CommunicationDirectionV1::Incoming,
                    Some(1_783_024_002),
                )
                .expect("build stale admitted-body ingress draft");
            let stale_draft = makosh_communications_ingress::with_admitted_body_blob(
                stale_draft,
                makosh_communications_ingress::BodyBlobReceiptV1 {
                    blob_ref: opaque_blob_reference.clone(),
                    reference_id,
                    declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
                    sha256: plaintext_sha256,
                    media_type: "text/plain".to_owned(),
                    custody_transfer_source_proof: source_proof.clone(),
                },
            )
            .expect("attach stale source opaque Blob receipt");
            let stale_record = makosh_communications_ingress::build_observation_outbox_record_v1(
                &stale_draft,
                &makosh_communications_ingress::ObservationEnvelopeContextV1 {
                    runtime_instance_id: "integration-test-runtime-1".to_owned(),
                    runtime_generation: 1,
                    module_id: "integration-test-runtime".to_owned(),
                    recorded_at_unix_seconds: 1_783_024_002,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build stale source typed ingress envelope");
            let endpoint = store
                .platform_event_hub_topology()
                .expect("read Event Hub topology")
                .expect("Event Hub topology")
                .nats_endpoint()
                .to_owned();
            tokio::runtime::Runtime::new()
                .expect("Tokio runtime")
                .block_on(async move {
                    async_nats::jetstream::new(
                        async_nats::connect(endpoint)
                            .await
                            .expect("connect disposable JetStream"),
                    )
                    .publish(
                        "makosh.observation.v1.communications.communication_observed.v1",
                        stale_record.exact_bytes().to_vec().into(),
                    )
                    .await
                    .expect("publish stale source typed ingress envelope")
                    .await
                    .expect("acknowledge stale source typed ingress envelope");
                });
            stale_source_published = true;
            continue;
        }
        if transferred && rejected >= 2 && !revoked_source_published {
            let current_reference_id = [9; 16];
            let current_channel_binding = vec![7; 32];
            let current_delivery = BlobSessionHandlerV1::new(
                Arc::clone(store),
                supervisor.relay_port(),
                kernel_data.to_path_buf(),
            )
            .issue_blob_session(
                &ManagedRuntimeExpectation::new(
                    FIXTURE_SOURCE_REGISTRATION,
                    FIXTURE_SOURCE_RUNTIME_INSTANCE_ID_V2,
                    "integration.fixture-source",
                    2,
                    source_grant_epoch,
                    [3; 32],
                    None,
                ),
                ManagedRuntimeBlobSessionRequestV1 {
                    request_id: vec![5; 16],
                    capability_id: FIXTURE_SOURCE_CAPABILITY_ID.to_owned(),
                    operation: BlobDataOperationV1::BlobDataOperationWriteV1 as u32,
                    channel_binding_sha256: Sha256::digest(&current_channel_binding).to_vec(),
                    reference_id: current_reference_id.to_vec(),
                    declared_size: u64::try_from(plaintext.len()).expect("fixture body size"),
                    backup_class: 1,
                    ttl_seconds: 30,
                    receipt_sha256: plaintext_sha256.to_vec(),
                    custody_source_proof: Vec::new(),
                    evidence_id: Vec::new(),
                    evidence_envelope_sha256: Vec::new(),
                    custody_target_owner_id: String::new(),
                    custody_target_module_id: String::new(),
                    custody_target_capability_id: String::new(),
                },
            )
            .expect("issue successor source integration Blob write session");
            let current_source_proof = current_delivery.custody_transfer_source_proof;
            BlobDataClient::new(current_delivery.data_socket_path)
                .expect("open successor source Blob data client")
                .write(
                    current_delivery
                        .grant
                        .expect("successor source Blob write grant"),
                    current_channel_binding,
                    plaintext.to_vec(),
                )
                .expect("write successor source Blob content");
            store
                .transition_module_registration(
                    FIXTURE_SOURCE_REGISTRATION,
                    ModuleRegistrationState::Revoked,
                )
                .expect("revoke fixture source integration");
            let revoked_draft =
                makosh_communications_ingress::new_scoped_communication_observation_draft(
                    "managed-revoked-body-observation-1",
                    makosh_communications_ingress::SourceEnvelope {
                        provider: makosh_communications_ingress::ProviderProvenanceV1::Telegram,
                        external_record_id: "integration-private-body-record-revoked-1".to_owned(),
                        scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                            external_account_id: "integration-private-body-account-1".to_owned(),
                            external_conversation_id: Some(
                                "integration-private-body-conversation-1".to_owned(),
                            ),
                            external_participant_id: None,
                            external_media_id: None,
                            external_reply_to_record_id: None,
                            external_forward_origin_record_id: None,
                        }),
                    },
                    makosh_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
                    makosh_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
                    makosh_communications_ingress::CommunicationDirectionV1::Incoming,
                    Some(1_783_024_002),
                )
                .expect("build revoked admitted-body ingress draft");
            let revoked_draft = makosh_communications_ingress::with_admitted_body_blob(
                revoked_draft,
                makosh_communications_ingress::BodyBlobReceiptV1 {
                    blob_ref: opaque_blob_reference.clone(),
                    reference_id: current_reference_id,
                    declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
                    sha256: plaintext_sha256,
                    media_type: "text/plain".to_owned(),
                    custody_transfer_source_proof: current_source_proof,
                },
            )
            .expect("attach revoked source opaque Blob receipt");
            let revoked_record = makosh_communications_ingress::build_observation_outbox_record_v1(
                &revoked_draft,
                &makosh_communications_ingress::ObservationEnvelopeContextV1 {
                    runtime_instance_id: "integration-test-runtime-1".to_owned(),
                    runtime_generation: 1,
                    module_id: "integration-test-runtime".to_owned(),
                    recorded_at_unix_seconds: 1_783_024_002,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build revoked source typed ingress envelope");
            let endpoint = store
                .platform_event_hub_topology()
                .expect("read Event Hub topology")
                .expect("Event Hub topology")
                .nats_endpoint()
                .to_owned();
            tokio::runtime::Runtime::new()
                .expect("Tokio runtime")
                .block_on(async move {
                    async_nats::jetstream::new(
                        async_nats::connect(endpoint)
                            .await
                            .expect("connect disposable JetStream"),
                    )
                    .publish(
                        "makosh.observation.v1.communications.communication_observed.v1",
                        revoked_record.exact_bytes().to_vec().into(),
                    )
                    .await
                    .expect("publish revoked source typed ingress envelope")
                    .await
                    .expect("acknowledge revoked source typed ingress envelope");
                });
            revoked_source_published = true;
            continue;
        }
        if transferred && rejected >= 3 {
            let transferred_message_id = messages
                .messages
                .iter()
                .find(|message| message.body_state == 4)
                .expect("transferred message is present in the canonical projection")
                .message_id
                .clone();
            let public_payload = CommunicationsQueryResponseV1 {
                result: Some(QueryResult::ListConversationMessages(messages)),
                error_code: String::new(),
            }
            .encode_to_vec();
            assert!(
                !public_payload
                    .windows(opaque_blob_reference.len())
                    .any(|window| window == opaque_blob_reference.as_bytes()),
                "public Communications query must not reveal an owner-private Blob reference",
            );
            return transferred_message_id;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "custody transfer must retain a policy-rejected body without blocking a valid body"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn publish_and_wait_for_communications_message_deletion(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    message_id: &[u8],
) {
    let draft = makosh_communications_ingress::new_scoped_communication_observation_draft(
        "managed-deleted-body-observation-1",
        makosh_communications_ingress::SourceEnvelope {
            provider: makosh_communications_ingress::ProviderProvenanceV1::Telegram,
            external_record_id: "integration-private-body-record-1".to_owned(),
            scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-body-account-1".to_owned(),
                external_conversation_id: Some(
                    "integration-private-body-conversation-1".to_owned(),
                ),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        makosh_communications_ingress::CommunicationEvidenceKindV1::MessageDeleted,
        makosh_communications_ingress::BodyAvailabilityV1::Unavailable,
        makosh_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_010),
    )
    .expect("build deleted-message ingress draft");
    let record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_010,
            recorded_at_nanos: 0,
        },
    )
    .expect("build deleted-message typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect disposable JetStream"),
            )
            .publish(
                "makosh.observation.v1.communications.communication_observed.v1",
                record.exact_bytes().to_vec().into(),
            )
            .await
            .expect("publish deleted-message typed ingress envelope")
            .await
            .expect("acknowledge deleted-message typed ingress envelope");
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let detail = route_communications_query(
            store,
            supervisor,
            72,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::GetMessage(GetMessageRequestV1 {
                    message_id: message_id.to_vec(),
                })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::GetMessage(detail)) = detail.result else {
            panic!("Communications deleted-message detail result");
        };
        let message = detail
            .message
            .expect("deleted canonical message remains available as evidence metadata");
        if message.lifecycle_state == 2 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Communications delete observation did not advance canonical lifecycle; message={message:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn publish_and_wait_for_communications_message_edit(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    kernel_data: &Path,
    message_id: &[u8],
    plaintext: Vec<u8>,
    observed_at_unix_seconds: i64,
    fixture_marker: u8,
) -> Vec<u8> {
    let plaintext_sha256: [u8; 32] = Sha256::digest(&plaintext).into();
    let reference_id = [fixture_marker; 16];
    let channel_binding = vec![fixture_marker; 32];
    let launch = store
        .effective_managed_launch_record(FIXTURE_SOURCE_REGISTRATION)
        .expect("read fixture source integration launch")
        .expect("fixture source integration launch is active");
    let delivery = BlobSessionHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
        kernel_data.to_path_buf(),
    )
    .issue_blob_session(
        &ManagedRuntimeExpectation::new(
            FIXTURE_SOURCE_REGISTRATION,
            FIXTURE_SOURCE_RUNTIME_INSTANCE_ID,
            "integration.fixture-source",
            launch.runtime_generation(),
            launch.grant_epoch(),
            [3; 32],
            None,
        ),
        ManagedRuntimeBlobSessionRequestV1 {
            request_id: vec![fixture_marker; 16],
            capability_id: FIXTURE_SOURCE_CAPABILITY_ID.to_owned(),
            operation: BlobDataOperationV1::BlobDataOperationWriteV1 as u32,
            channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
            reference_id: reference_id.to_vec(),
            declared_size: u64::try_from(plaintext.len()).expect("edited fixture body size"),
            backup_class: 1,
            ttl_seconds: 30,
            receipt_sha256: plaintext_sha256.to_vec(),
            custody_source_proof: Vec::new(),
            evidence_id: Vec::new(),
            evidence_envelope_sha256: Vec::new(),
            custody_target_owner_id: String::new(),
            custody_target_module_id: String::new(),
            custody_target_capability_id: String::new(),
        },
    )
    .expect("issue edited source integration Blob write session");
    let source_proof = delivery.custody_transfer_source_proof;
    BlobDataClient::new(delivery.data_socket_path)
        .expect("open edited source Blob data client")
        .write(
            delivery.grant.expect("edited source Blob write grant"),
            channel_binding,
            plaintext.clone(),
        )
        .expect("write edited source integration Blob content");

    let draft = makosh_communications_ingress::new_scoped_communication_observation_draft(
        format!("managed-edited-body-observation-{fixture_marker}"),
        makosh_communications_ingress::SourceEnvelope {
            provider: makosh_communications_ingress::ProviderProvenanceV1::Telegram,
            external_record_id: "integration-private-body-record-1".to_owned(),
            scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-body-account-1".to_owned(),
                external_conversation_id: Some(
                    "integration-private-body-conversation-1".to_owned(),
                ),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        makosh_communications_ingress::CommunicationEvidenceKindV1::MessageEdited,
        makosh_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
        makosh_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(observed_at_unix_seconds),
    )
    .expect("build edited-message ingress draft");
    let draft = makosh_communications_ingress::with_admitted_body_blob(
        draft,
        makosh_communications_ingress::BodyBlobReceiptV1 {
            blob_ref: format!("blob://fixture-source/admitted-body-edited-{fixture_marker}"),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len()).expect("edited fixture body size"),
            sha256: plaintext_sha256,
            media_type: "text/plain".to_owned(),
            custody_transfer_source_proof: source_proof,
        },
    )
    .expect("attach edited-message admitted-body Blob receipt");
    let record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: observed_at_unix_seconds,
            recorded_at_nanos: 0,
        },
    )
    .expect("build edited-message typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect disposable JetStream"),
            )
            .publish(
                "makosh.observation.v1.communications.communication_observed.v1",
                record.exact_bytes().to_vec().into(),
            )
            .await
            .expect("publish edited-message typed ingress envelope")
            .await
            .expect("acknowledge edited-message typed ingress envelope");
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let detail = route_communications_query(
            store,
            supervisor,
            71,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::GetMessage(GetMessageRequestV1 {
                    message_id: message_id.to_vec(),
                })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::GetMessage(detail)) = detail.result else {
            panic!("Communications edited-message detail result");
        };
        let message = detail
            .message
            .expect("edited canonical message remains projected");
        if message.lifecycle_state == 1
            && message.body_state == 4
            && message.last_observed_at_unix_seconds == observed_at_unix_seconds
        {
            return plaintext;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Communications edit observation did not advance canonical revision; message={message:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn wait_for_transferred_body_message(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    account_cursor: &[u8],
) -> Vec<u8> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            41,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                    limit: 16,
                    cursor: Vec::new(),
                })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        let Some(account) = accounts
            .accounts
            .iter()
            .find(|account| account.account_cursor_sha256 == account_cursor)
        else {
            assert!(
                std::time::Instant::now() < deadline,
                "exportable admitted-body account was not projected; active={:?}; last_failure={:?}",
                supervisor.is_active(COMMUNICATIONS_REGISTRATION),
                supervisor.last_failure(COMMUNICATIONS_REGISTRATION),
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let conversations = route_communications_query(
            store,
            supervisor,
            42,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversations(
                    makosh_communications_api::query_wire::ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
            panic!("Communications conversations query result");
        };
        let Some(conversation) = conversations.conversations.first() else {
            assert!(
                std::time::Instant::now() < deadline,
                "exportable admitted-body conversation was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let messages = route_communications_query(
            store,
            supervisor,
            43,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationMessages(
                    makosh_communications_api::query_wire::ListConversationMessagesRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
            panic!("Communications messages query result");
        };
        if let Some(message) = messages
            .messages
            .iter()
            .find(|message| message.body_state == 4)
        {
            return message.message_id.clone();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "exportable admitted-body message was not projected"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn fixture_account_cursor(
    provider: makosh_communications_ingress::ProviderProvenanceV1,
    external_account_id: &str,
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.communications.account-cursor.v1\0");
    hasher.update(provider.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(external_account_id.as_bytes());
    hasher.finalize().to_vec()
}

pub(super) fn assert_communications_attachment_anchor_projection(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    const PROVIDER_MEDIA_LOCATOR: &str = "integration-private-media-1";
    let draft = makosh_communications_ingress::new_scoped_communication_observation_draft(
        "managed-attachment-observation-1",
        makosh_communications_ingress::SourceEnvelope {
            provider: makosh_communications_ingress::ProviderProvenanceV1::MailImap,
            // A media mutation updates the message established by the earlier Mail observation.
            external_record_id: "integration-private-record-1".to_owned(),
            scope: Some(makosh_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-account-1".to_owned(),
                external_conversation_id: Some("integration-private-record-1".to_owned()),
                external_participant_id: None,
                external_media_id: Some(PROVIDER_MEDIA_LOCATOR.to_owned()),
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        makosh_communications_ingress::CommunicationEvidenceKindV1::MediaChanged,
        makosh_communications_ingress::BodyAvailabilityV1::MetadataOnly,
        makosh_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_002),
    )
    .expect("build attachment ingress draft");
    let draft = makosh_communications_ingress::with_attachment_descriptor(
        draft,
        makosh_communications_ingress::AttachmentDescriptorV1 {
            filename: Some("evidence.txt".to_owned()),
            media_type: "text/plain".to_owned(),
            declared_bytes: 32,
            sha256: Some([10; 32]),
            disposition: makosh_communications_ingress::AttachmentDispositionV1::Attachment,
        },
    )
    .expect("attach typed attachment descriptor");
    let record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_002,
            recorded_at_nanos: 0,
        },
    )
    .expect("build attachment typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            use futures_util::StreamExt as _;
            use prost::Message as _;

            let client = async_nats::connect(endpoint)
                .await
                .expect("connect disposable JetStream");
            let mut anchor_events = client
                .subscribe("makosh.event.v1.communications.communication_attachment_anchor_recorded.v1")
                .await
                .expect("subscribe to exact attachment-anchor handoff subject");
            let mut safety_events = client
                .subscribe("makosh.event.v1.communications.communication_attachment_safety_state_changed.v1")
                .await
                .expect("subscribe to exact attachment lifecycle subject");
            let context = async_nats::jetstream::new(client);
            context
                .publish(
                    "makosh.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish attachment typed ingress envelope")
                .await
                .expect("acknowledge attachment typed ingress envelope");
            let anchor_event = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                anchor_events.next(),
            )
            .await
            .expect("attachment-anchor handoff timeout")
            .expect("attachment-anchor handoff missing");
            let envelope = makosh_events_protocol::validation::envelope::decode_envelope_v1(
                anchor_event.payload.as_ref(),
            )
            .expect("attachment-anchor handoff envelope");
            let ingress = makosh_events_protocol::validation::envelope::decode_envelope_v1(
                record.exact_bytes(),
            )
            .expect("attachment ingress envelope");
            assert!(matches!(
                envelope.contract.as_ref(),
                Some(contract)
                    if contract.owner == "communications"
                        && contract.name == "communication_attachment_anchor_recorded"
                        && contract.major == 1
                        && contract.revision == 1
            ));
            assert_eq!(envelope.causation_message_id, record.message_id().to_vec());
            assert_eq!(envelope.correlation_id, ingress.correlation_id);
            let payload = makosh_communications_attachment_contract::anchor_recorded_v1::AttachmentAnchorRecordedV1::decode(
                envelope.payload.as_slice(),
            )
            .expect("attachment-anchor handoff payload");
            assert_eq!(payload.source_observation_id, record.message_id().to_vec());
            assert_eq!(payload.media_cursor_sha256.len(), 32);
            assert_eq!(payload.initial_state, 1);
            let attachment_anchor_id: [u8; 16] = payload
                .attachment_anchor_id
                .as_slice()
                .try_into()
                .expect("attachment anchor identifier");
            let media_cursor_sha256: [u8; 32] = payload
                .media_cursor_sha256
                .as_slice()
                .try_into()
                .expect("attachment media cursor");
            let correlation_id: [u8; 16] = ingress
                .correlation_id
                .as_slice()
                .try_into()
                .expect("attachment correlation identifier");
            let requested = makosh_communications_attachment_contract::build_attachment_blob_admission_outbox_record_v1(
                &makosh_communications_attachment_contract::AttachmentBlobAdmissionFactV1 {
                    attachment_anchor_id,
                    source_observation_id: *record.message_id(),
                    correlation_id,
                    media_cursor_sha256,
                    expected_state: makosh_communications_attachment_contract::AttachmentBlobExpectedStateV1::DescriptorOnly,
                    transition: makosh_communications_attachment_contract::AttachmentBlobAdmissionTransitionV1::Requested,
                    observed_at_unix_seconds: 1_783_024_003,
                    blob_reference_binding_sha256: None,
                },
                &makosh_communications_attachment_contract::AttachmentObservationEnvelopeContextV1 {
                    runtime_instance_id: "attachment-integration-test-runtime-1".to_owned(),
                    runtime_generation: 1,
                    module_id: "attachment-integration-test-runtime".to_owned(),
                    recorded_at_unix_seconds: 1_783_024_003,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build requested attachment admission envelope");
            context
                .publish(
                    "makosh.observation.v1.communications.communication_attachment_blob_admission_observed.v1",
                    requested.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish requested attachment admission envelope")
                .await
                .expect("acknowledge requested attachment admission envelope");
            let admitted = makosh_communications_attachment_contract::build_attachment_blob_admission_outbox_record_v1(
                &makosh_communications_attachment_contract::AttachmentBlobAdmissionFactV1 {
                    attachment_anchor_id,
                    source_observation_id: *record.message_id(),
                    correlation_id,
                    media_cursor_sha256,
                    expected_state: makosh_communications_attachment_contract::AttachmentBlobExpectedStateV1::BlobPending,
                    transition: makosh_communications_attachment_contract::AttachmentBlobAdmissionTransitionV1::Admitted,
                    observed_at_unix_seconds: 1_783_024_004,
                    blob_reference_binding_sha256: Some([11; 32]),
                },
                &makosh_communications_attachment_contract::AttachmentObservationEnvelopeContextV1 {
                    runtime_instance_id: "attachment-integration-test-runtime-1".to_owned(),
                    runtime_generation: 1,
                    module_id: "attachment-integration-test-runtime".to_owned(),
                    recorded_at_unix_seconds: 1_783_024_004,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build admitted attachment admission envelope");
            context
                .publish(
                    "makosh.observation.v1.communications.communication_attachment_blob_admission_observed.v1",
                    admitted.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish admitted attachment admission envelope")
                .await
                .expect("acknowledge admitted attachment admission envelope");
            for (transition_index, causation_message_id) in
                [requested.message_id(), admitted.message_id()]
                    .into_iter()
                    .enumerate()
            {
                let state_event = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    safety_events.next(),
                )
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "attachment lifecycle event {transition_index} timeout: {error:?}",
                    )
                })
                .expect("attachment lifecycle event missing");
                let state_envelope = makosh_events_protocol::validation::envelope::decode_envelope_v1(
                    state_event.payload.as_ref(),
                )
                .expect("attachment lifecycle envelope");
                assert_eq!(state_envelope.causation_message_id, causation_message_id.to_vec());
                assert_eq!(state_envelope.correlation_id, ingress.correlation_id);
            }
            assert!(
                !anchor_event
                    .payload
                    .windows(PROVIDER_MEDIA_LOCATOR.len())
                    .any(|window| window == PROVIDER_MEDIA_LOCATOR.as_bytes()),
                "attachment-anchor handoff must not reveal a provider-local media locator",
            );
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            7,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                    limit: 16,
                    cursor: Vec::new(),
                })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        let Some(account) = accounts
            .accounts
            .iter()
            .find(|account| account.provider == 1)
        else {
            assert!(
                std::time::Instant::now() < deadline,
                "attachment account was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let conversations = route_communications_query(
            store,
            supervisor,
            8,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversations(
                    makosh_communications_api::query_wire::ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
            panic!("Communications conversations query result");
        };
        let Some(conversation) = conversations.conversations.first() else {
            assert!(
                std::time::Instant::now() < deadline,
                "attachment conversation was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let messages = route_communications_query(
            store,
            supervisor,
            9,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationMessages(
                    makosh_communications_api::query_wire::ListConversationMessagesRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
            panic!("Communications messages query result");
        };
        for message in messages.messages {
            let anchors = route_communications_query(
                store,
                supervisor,
                10,
                &CommunicationsQueryRequestV1 {
                    protocol_major: 1,
                    operation: Some(Operation::ListMessageAttachmentAnchors(
                        makosh_communications_api::query_wire::ListMessageAttachmentAnchorsRequestV1 {
                            message_id: message.message_id,
                            limit: 16,
                            cursor: Vec::new(),
                        },
                    )),
                }
                .encode_to_vec(),
            );
            let Some(QueryResult::ListMessageAttachmentAnchors(anchors)) = anchors.result else {
                panic!("Communications attachment anchors query result");
            };
            if let Some(anchor) = anchors.anchors.iter().find(|anchor| {
                anchor.has_descriptor
                    && anchor.filename == "evidence.txt"
                    && anchor.media_type == "text/plain"
                    && anchor.declared_bytes == 32
                    && anchor.sha256 == vec![10; 32]
                    && anchor.disposition == 1
                    && anchor.state == 3
            }) {
                let public_payload = CommunicationsQueryResponseV1 {
                    result: Some(QueryResult::ListMessageAttachmentAnchors(
                        makosh_communications_api::query_wire::ListMessageAttachmentAnchorsResponseV1 {
                            anchors: vec![anchor.clone()],
                            next_cursor: Vec::new(),
                        },
                    )),
                    error_code: String::new(),
                }
                .encode_to_vec();
                assert!(
                    !public_payload
                        .windows(PROVIDER_MEDIA_LOCATOR.len())
                        .any(|window| window == PROVIDER_MEDIA_LOCATOR.as_bytes()),
                    "public Communications anchor must not reveal a provider-local media locator",
                );
                return;
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "attachment anchor was not projected"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn assert_communications_relationship_projection(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    const PRIVATE_PARTICIPANT_ID: &str = "integration-private-participant-1";
    const PRIVATE_REPLY_RECORD_ID: &str = "integration-private-reply-1";
    const PRIVATE_FORWARD_RECORD_ID: &str = "integration-private-forward-1";
    let draft =
        makosh_telegram_core::observation_draft(makosh_telegram_api::TelegramMessageObservation {
            account_id: "integration-private-relationship-account-1".to_owned(),
            provider_chat_id: "integration-private-relationship-conversation-1".to_owned(),
            provider_message_id: "managed-relationship-observation-1".to_owned(),
            provider_topic_id: None,
            sender_id: PRIVATE_PARTICIPANT_ID.to_owned(),
            sender_display_name: None,
            is_outgoing: false,
            text: None,
            media: None,
            references: makosh_telegram_api::TelegramMessageReferences {
                reply_to: Some(makosh_telegram_api::TelegramReplyReference {
                    provider_chat_id: "integration-private-relationship-conversation-1".to_owned(),
                    provider_message_id: PRIVATE_REPLY_RECORD_ID.to_owned(),
                }),
                forward_origin: Some(makosh_telegram_api::TelegramForwardOrigin {
                    provider_chat_id: Some(
                        "integration-private-relationship-conversation-1".to_owned(),
                    ),
                    provider_message_id: Some(PRIVATE_FORWARD_RECORD_ID.to_owned()),
                    provider_sender_id: None,
                    sender_name: None,
                    observed_at_unix_seconds: None,
                }),
            },
            observed_at_unix_seconds: 1_783_024_003,
        })
        .expect("build typed Telegram relationship ingress draft");
    let record = makosh_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &makosh_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "telegram-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "makosh-telegram-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_003,
            recorded_at_nanos: 0,
        },
    )
    .expect("build relationship typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect disposable JetStream"),
            );
            context
                .publish(
                    "makosh.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish relationship typed ingress envelope")
                .await
                .expect("acknowledge relationship typed ingress envelope");
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            11,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                    limit: 16,
                    cursor: Vec::new(),
                })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        let Some(account) = accounts
            .accounts
            .iter()
            .find(|account| account.provider == 2)
        else {
            assert!(
                std::time::Instant::now() < deadline,
                "relationship account was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let conversations = route_communications_query(
            store,
            supervisor,
            12,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversations(
                    makosh_communications_api::query_wire::ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
            panic!("Communications conversations query result");
        };
        let Some(conversation) = conversations.conversations.first() else {
            assert!(
                std::time::Instant::now() < deadline,
                "relationship conversation was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let participants = route_communications_query(
            store,
            supervisor,
            13,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationParticipants(
                    makosh_communications_api::query_wire::ListConversationParticipantsRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationParticipants(participants)) = participants.result
        else {
            panic!("Communications participants query result");
        };
        let messages = route_communications_query(
            store,
            supervisor,
            14,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationMessages(
                    makosh_communications_api::query_wire::ListConversationMessagesRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
            panic!("Communications messages query result");
        };
        let Some(message) = messages.messages.first() else {
            assert!(
                std::time::Instant::now() < deadline,
                "relationship message was not projected"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let references = route_communications_query(
            store,
            supervisor,
            15,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListMessageReferences(
                    makosh_communications_api::query_wire::ListMessageReferencesRequestV1 {
                        message_id: message.message_id.clone(),
                        limit: 16,
                        cursor: Vec::new(),
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListMessageReferences(references)) = references.result else {
            panic!("Communications references query result");
        };
        if !participants.participants.is_empty()
            && references
                .references
                .iter()
                .any(|reference| reference.kind == 1)
            && references
                .references
                .iter()
                .any(|reference| reference.kind == 2)
        {
            let participant_payload = CommunicationsQueryResponseV1 {
                result: Some(QueryResult::ListConversationParticipants(participants)),
                error_code: String::new(),
            }
            .encode_to_vec();
            let reference_payload = CommunicationsQueryResponseV1 {
                result: Some(QueryResult::ListMessageReferences(references)),
                error_code: String::new(),
            }
            .encode_to_vec();
            for private_id in [
                PRIVATE_PARTICIPANT_ID,
                PRIVATE_REPLY_RECORD_ID,
                PRIVATE_FORWARD_RECORD_ID,
            ] {
                assert!(
                    !participant_payload
                        .windows(private_id.len())
                        .any(|window| window == private_id.as_bytes())
                        && !reference_payload
                            .windows(private_id.len())
                            .any(|window| window == private_id.as_bytes()),
                    "public Communications relationships must not reveal provider-local identifiers",
                );
            }
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "relationship projections were not committed"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn route_communications_query(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    request_id: u64,
    payload: &[u8],
) -> CommunicationsQueryResponseV1 {
    let request = encode_module_query_request_v1(request_id, payload)
        .expect("encode Communications query module request");
    let launch = store
        .effective_managed_launch_record(COMMUNICATIONS_REGISTRATION)
        .expect("read Communications launch")
        .expect("Communications launch is active");
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        COMMUNICATIONS_REGISTRATION,
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
        COMMUNICATIONS_QUERY_CAPABILITY_ID,
        &request,
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let bytes = match crate::modules::capability::router::route_managed_client_request(
            store,
            &supervisor.relay_port(),
            &route,
        ) {
            Ok(bytes) => bytes,
            Err(error)
                if error == "managed runtime V2 relay response is invalid"
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(std::time::Duration::from_millis(25));
                continue;
            }
            Err(error) => panic!("route exact Communications owner query: {error}"),
        };
        let response = ModuleClientResponseV1::decode(bytes.as_slice())
            .expect("decode Communications module response");
        assert_eq!(response.request_id, request_id);
        if response.error_code == "RUNTIME_UNAVAILABLE" && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        }
        assert!(
            response.error_code.is_empty(),
            "Communications query {request_id} failed: {}; last_failure={:?}",
            response.error_code,
            supervisor.last_failure(COMMUNICATIONS_REGISTRATION),
        );
        return CommunicationsQueryResponseV1::decode(response.response_payload.as_slice())
            .expect("decode Communications query response");
    }
}

fn record_communications_registration(store: &SqliteControlStore, descriptor: &[u8]) -> u64 {
    let registration = ModuleRegistration::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_MODULE_ID,
        COMMUNICATIONS_OWNER_ID,
        Sha256::digest(descriptor).into(),
        ModuleRegistrationState::Pending,
        1,
    );
    let capabilities = [
        COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_SUMMARY_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_TRANSLATION_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_ATTACHMENT_BLOB_ADMISSION_OBSERVE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_ATTACHMENT_SAFETY_VERDICT_OBSERVE_CAPABILITY_ID.to_owned(),
        "communications.blob.v1".to_owned(),
        CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1.to_owned(),
        COMMUNICATIONS_CALL_EVIDENCE_OBSERVE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_CONTENT_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_EVENTS_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_OBSERVE_CAPABILITY_ID.to_owned(),
        "communications.query.v1".to_owned(),
        COMMUNICATIONS_RECIPIENT_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_SAVED_SEARCH_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_SENDER_INSIGHTS_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_STORAGE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_TASK_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID.to_owned(),
    ];
    let storage = ModuleStorageRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        8,
        5_000,
    );
    let blob = makosh_kernel_control_store::ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![
            ModuleBlobOperationV1::ReadRange,
            ModuleBlobOperationV1::CustodyTransfer,
        ],
    );
    let content_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_CONTENT_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::ReadRange],
    );
    let export_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let cross_channel_forward_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let ai_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let summary_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_SUMMARY_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let task_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_TASK_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let note_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let translation_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_TRANSLATION_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let explanation_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let recipient_source_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_RECIPIENT_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    let vault_purpose = ModuleVaultPurposeRequestV1::new_with_key_schema_revision(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID,
        COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID,
        u16::try_from(COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS)
            .expect("Communications search key lease TTL fits u16"),
        ModuleVaultPurposePolicyV1 {
            secret_class: VaultSecretClassV1::OwnerDerivedKey as u8,
            action: VaultActionV1::IssueOwnerDerivedKey as u8,
            target_scope: VaultTargetScopeV1::OwnerDerivedProjectionKey as u8,
            key_schema_revision: COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION,
        },
    );
    let recorded = communication_evidence_recorded_contract_reference_v1();
    let attachment_anchor_recorded =
        communication_attachment_anchor_recorded_contract_reference_v1();
    let attachment_safety_state_changed =
        communication_attachment_safety_state_changed_contract_reference_v1();
    let observed =
        makosh_communications_ingress::admission::communication_observed_contract_reference_v1();
    let call_evidence_observed = call_evidence_observed_contract_reference_v1();
    let attachment_blob_admission =
        communication_attachment_blob_admission_observed_contract_reference_v1();
    let attachment_safety_verdict =
        communication_attachment_safety_verdict_observed_contract_reference_v1();
    let evidence_export_prepare = evidence_export_prepare_contract_reference_v1();
    let evidence_export_prepared = evidence_export_prepared_contract_reference_v1();
    let evidence_export_rejected = evidence_export_rejected_contract_reference_v1();
    let cross_channel_forward_source_prepare =
        cross_channel_forward_source_prepare_contract_reference_v1();
    let cross_channel_forward_source_prepared =
        cross_channel_forward_source_prepared_contract_reference_v1();
    let cross_channel_forward_source_rejected =
        cross_channel_forward_source_rejected_contract_reference_v1();
    let ai_source_prepare = communication_reply_source_prepare_contract_reference_v1();
    let ai_source_prepared = communication_reply_source_prepared_contract_reference_v1();
    let ai_source_rejected = communication_reply_source_rejected_contract_reference_v1();
    let summary_source_prepare = communication_summary_source_prepare_contract_reference_v1();
    let summary_source_prepared = communication_summary_source_prepared_contract_reference_v1();
    let summary_source_rejected = communication_summary_source_rejected_contract_reference_v1();
    let task_source_prepare = communication_task_source_prepare_contract_reference_v1();
    let task_source_prepared = communication_task_source_prepared_contract_reference_v1();
    let task_source_rejected = communication_task_source_rejected_contract_reference_v1();
    let note_source_prepare = communication_note_source_prepare_contract_reference_v1();
    let note_source_prepared = communication_note_source_prepared_contract_reference_v1();
    let note_source_rejected = communication_note_source_rejected_contract_reference_v1();
    let translation_source_prepare =
        communication_translation_source_prepare_contract_reference_v1();
    let translation_source_prepared =
        communication_translation_source_prepared_contract_reference_v1();
    let translation_source_rejected =
        communication_translation_source_rejected_contract_reference_v1();
    let explanation_source_prepare =
        communication_explanation_source_prepare_contract_reference_v1();
    let explanation_source_prepared =
        communication_explanation_source_prepared_contract_reference_v1();
    let explanation_source_rejected =
        communication_explanation_source_rejected_contract_reference_v1();
    let recipient_source_prepare = communication_recipient_source_prepare_contract_reference_v1();
    let recipient_source_prepared = communication_recipient_source_prepared_contract_reference_v1();
    let recipient_source_rejected = communication_recipient_source_rejected_contract_reference_v1();
    let replay_command = communications_replay_command_contract_reference_v1();
    let replay_result = communications_replay_result_contract_reference_v1();
    let routes = [
        communications_event_route(
            COMMUNICATIONS_ATTACHMENT_BLOB_ADMISSION_OBSERVE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Observation,
            &attachment_blob_admission,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_ATTACHMENT_SAFETY_VERDICT_OBSERVE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Observation,
            &attachment_safety_verdict,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_EVENTS_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Event,
            &recorded,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &evidence_export_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &evidence_export_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &evidence_export_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_EVENTS_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Event,
            &attachment_anchor_recorded,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_EVENTS_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Event,
            &attachment_safety_state_changed,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &note_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &note_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &note_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_OBSERVE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Observation,
            &observed,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_CALL_EVIDENCE_OBSERVE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Observation,
            &call_evidence_observed,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &cross_channel_forward_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &cross_channel_forward_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &cross_channel_forward_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &ai_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &ai_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &summary_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &summary_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &translation_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &translation_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &translation_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &explanation_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &explanation_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &explanation_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &recipient_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &recipient_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &recipient_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &summary_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &task_source_prepare,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &task_source_prepared,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &task_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &ai_source_rejected,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Command,
            &replay_command,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_event_route(
            COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Result,
            &replay_result,
            ModuleEventRouteDirectionV1::Publish,
        ),
    ];
    let client_rpc_routes = [
        ModuleClientRpcRouteV1::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_QUERY_CAPABILITY_ID,
            COMMUNICATIONS_OWNER_ID,
            "communications.query",
            ModuleClientRpcContractVersionV1 {
                major: 1,
                revision: 1,
            },
            makosh_communications_api::COMMUNICATIONS_QUERY_SCHEMA_SHA256,
            "/makosh.communications.query.v1.CommunicationsQueryService/Query",
        ),
        ModuleClientRpcRouteV1::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_CONTENT_CAPABILITY_ID,
            COMMUNICATIONS_OWNER_ID,
            CONTENT_TICKET_CONTRACT_NAME_V1,
            ModuleClientRpcContractVersionV1 {
                major: CONTENT_CONTRACT_MAJOR_V1,
                revision: CONTENT_CONTRACT_REVISION_V1,
            },
            COMMUNICATIONS_CONTENT_TICKET_SCHEMA_SHA256,
            CONTENT_TICKET_CONNECT_PATH_V1,
        ),
        ModuleClientRpcRouteV1::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_SAVED_SEARCH_CAPABILITY_ID,
            COMMUNICATIONS_OWNER_ID,
            SAVED_SEARCH_CONTRACT_NAME_V1,
            ModuleClientRpcContractVersionV1 {
                major: SAVED_SEARCH_CONTRACT_MAJOR_V1,
                revision: SAVED_SEARCH_CONTRACT_REVISION_V1,
            },
            COMMUNICATIONS_SAVED_SEARCH_SCHEMA_SHA256,
            SAVED_SEARCH_CONNECT_PATH_V1,
        ),
        ModuleClientRpcRouteV1::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_SENDER_INSIGHTS_CAPABILITY_ID,
            COMMUNICATIONS_OWNER_ID,
            SENDER_INSIGHTS_CONTRACT_NAME_V1,
            ModuleClientRpcContractVersionV1 {
                major: SENDER_INSIGHTS_CONTRACT_MAJOR_V1,
                revision: SENDER_INSIGHTS_CONTRACT_REVISION_V1,
            },
            COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256,
            SENDER_INSIGHTS_CONNECT_PATH_V1,
        ),
        ModuleClientRpcRouteV1::new(
            COMMUNICATIONS_REGISTRATION,
            CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1,
            COMMUNICATIONS_OWNER_ID,
            CALL_EVIDENCE_QUERY_CONTRACT_NAME_V1,
            ModuleClientRpcContractVersionV1 {
                major: CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
                revision: CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1,
            },
            CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1,
            CALL_EVIDENCE_QUERY_CONNECT_PATH_V1,
        ),
    ];
    let client_blob_route = ModuleClientBlobRouteV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_CONTENT_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        CONTENT_READ_CONTRACT_NAME_V1,
        ModuleClientBlobContractVersionV1 {
            major: CONTENT_CONTRACT_MAJOR_V1,
            revision: CONTENT_CONTRACT_REVISION_V1,
        },
        COMMUNICATIONS_CONTENT_READ_SCHEMA_SHA256,
        ModuleClientBlobTransportV1 {
            path: CONTENT_READ_BLOB_PATH_V1.to_owned(),
            max_response_bytes: MAX_MESSAGE_BODY_BYTES_V1,
        },
    );
    let query_rpc_route = makosh_kernel_control_store::ModuleQueryContractV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_QUERY_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        "communications.query",
        1,
        1,
        makosh_communications_api::COMMUNICATIONS_QUERY_SCHEMA_SHA256,
    );
    let call_evidence_query_rpc_route = makosh_kernel_control_store::ModuleQueryContractV1::new(
        COMMUNICATIONS_REGISTRATION,
        CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1,
        COMMUNICATIONS_OWNER_ID,
        CALL_EVIDENCE_QUERY_CONTRACT_NAME_V1,
        CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
        CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1,
        CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1,
    );
    let call_evidence_realtime_route = ModuleClientRealtimeRouteV1::new(
        COMMUNICATIONS_REGISTRATION,
        CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1,
        COMMUNICATIONS_OWNER_ID,
        CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1,
        ModuleClientRealtimeContractVersionV1 {
            major: CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
            revision: CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1,
        },
        CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1,
    );
    store
        .create_pending_registration_with_all_descriptor_requests(
            &registration,
            &capabilities,
            ModuleDescriptorRegistrationRequestsV1 {
                storage: std::slice::from_ref(&storage),
                events: &routes,
                blobs: &[
                    blob,
                    content_blob,
                    export_source_blob,
                    note_source_blob,
                    cross_channel_forward_source_blob,
                    ai_source_blob,
                    summary_source_blob,
                    task_source_blob,
                    translation_source_blob,
                    explanation_source_blob,
                    recipient_source_blob,
                ],
                scheduler: &[],
                vault_purposes: std::slice::from_ref(&vault_purpose),
                client_rpc_routes: &client_rpc_routes,
                client_blob_routes: std::slice::from_ref(&client_blob_route),
                client_realtime_routes: std::slice::from_ref(&call_evidence_realtime_route),
                query_rpc_routes: &[query_rpc_route, call_evidence_query_rpc_route],
                request_rpc_routes: &[],
                contract_dependencies: &[],
            },
        )
        .expect("record Communications registration");
    store
        .approve_module_registration(COMMUNICATIONS_REGISTRATION, &capabilities)
        .expect("approve Communications capabilities")
        .grant_epoch()
}

fn record_fixture_source_integration(store: &SqliteControlStore) -> u64 {
    if let Some(registration) = store
        .module_registration(FIXTURE_SOURCE_REGISTRATION)
        .expect("read fixture source integration registration")
    {
        assert_eq!(registration.state(), ModuleRegistrationState::Approved);
        return registration.grant_epoch();
    }
    let registration = ModuleRegistration::new(
        FIXTURE_SOURCE_REGISTRATION,
        "integration.fixture-source",
        COMMUNICATIONS_OWNER_ID,
        Sha256::digest(b"fixture-source-integration").into(),
        ModuleRegistrationState::Pending,
        1,
    );
    let capabilities = [FIXTURE_SOURCE_CAPABILITY_ID.to_owned()];
    let blob = ModuleBlobQuotaRequestV1::new(
        FIXTURE_SOURCE_REGISTRATION,
        FIXTURE_SOURCE_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
        COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID,
        vec![ModuleBlobOperationV1::Write],
    );
    store
        .create_pending_registration_with_requests(
            &registration,
            &capabilities,
            &[],
            &[],
            std::slice::from_ref(&blob),
        )
        .expect("record fixture source integration registration");
    let grant_epoch = store
        .approve_module_registration(FIXTURE_SOURCE_REGISTRATION, &capabilities)
        .expect("approve fixture source integration capability")
        .grant_epoch();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            FIXTURE_SOURCE_REGISTRATION,
            1,
            "fixture-source-distribution",
            "integration.fixture-source",
            Sha256::digest(b"fixture-source-integration-binary").into(),
            Sha256::digest(b"fixture-source-integration").into(),
            None,
        ))
        .expect("record fixture source integration release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            FIXTURE_SOURCE_REGISTRATION,
            FIXTURE_SOURCE_RUNTIME_INSTANCE_ID,
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record fixture source integration launch");
    grant_epoch
}

fn record_communications_runtime_fixture(
    store: &SqliteControlStore,
    schema: &[u8],
    descriptor: &[u8],
    grant_epoch: u64,
) {
    let canonical_bundle = communications_runtime_storage_bundle_v1()
        .expect("compose Communications runtime Storage bundle");
    let canonical_bundle_revision = canonical_bundle.revision;
    let canonical_bundle = canonical_bundle.encode_to_vec();
    let digest: [u8; 32] = Sha256::digest(&canonical_bundle).into();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                "communications",
                u64::from(canonical_bundle_revision),
                digest,
                canonical_bundle,
            )
            .expect("record Communications Storage bundle"),
        )
        .expect("persist Communications Storage bundle");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            COMMUNICATIONS_REGISTRATION,
            1,
            "makosh-managed-runtime-conformance",
            "domain.communications",
            Sha256::digest(
                std::fs::read(communications_binary()).expect("Communications binary bytes"),
            )
            .into(),
            Sha256::digest(descriptor).into(),
            Some(Sha256::digest(schema).into()),
        ))
        .expect("record Communications release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_RUNTIME_INSTANCE_ID,
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record Communications reservation");
    store
        .record_platform_event_hub_topology(&communications_event_hub_topology())
        .expect("record Event Hub topology");
}

fn record_communications_export_runtime_fixture(store: &SqliteControlStore) {
    let schema = communications_export_settings_schema_bytes_v1();
    let descriptor =
        communications_export_module_descriptor_v1("managed-communications-export-live")
            .encode_to_vec();
    let registration = ModuleRegistration::new(
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_MODULE_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        Sha256::digest(&descriptor).into(),
        ModuleRegistrationState::Pending,
        1,
    );
    let capabilities = [
        COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1.to_owned(),
        COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1.to_owned(),
        COMMUNICATIONS_EXPORT_EVENTS_CAPABILITY_ID_V1.to_owned(),
        COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1.to_owned(),
    ];
    let storage = ModuleStorageRequestV1::new(
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        4,
        5_000,
    );
    let client_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        COMMUNICATIONS_EXPORT_BLOB_QUOTA_BYTES_V1,
        COMMUNICATIONS_EXPORT_BLOB_CUSTODY_SCOPE_ID_V1,
        vec![ModuleBlobOperationV1::ReadRange],
    );
    let artifact_blob = ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        COMMUNICATIONS_EXPORT_BLOB_QUOTA_BYTES_V1,
        COMMUNICATIONS_EXPORT_BLOB_CUSTODY_SCOPE_ID_V1,
        vec![
            ModuleBlobOperationV1::Write,
            ModuleBlobOperationV1::ReadRange,
            ModuleBlobOperationV1::CustodyTransfer,
        ],
    );
    let prepare = evidence_export_prepare_contract_reference_v1();
    let prepared = evidence_export_prepared_contract_reference_v1();
    let rejected = evidence_export_rejected_contract_reference_v1();
    let routes = [
        communications_export_event_route(
            ModuleEventEnvelopeKindV1::Command,
            &prepare,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_export_event_route(
            ModuleEventEnvelopeKindV1::Result,
            &prepared,
            ModuleEventRouteDirectionV1::Consume,
        ),
        communications_export_event_route(
            ModuleEventEnvelopeKindV1::Result,
            &rejected,
            ModuleEventRouteDirectionV1::Consume,
        ),
    ];
    let client_rpc_routes = communications_export_client_rpc_routes();
    let client_blob_route = communications_export_client_blob_route();
    let client_realtime_route = communications_export_client_realtime_route();
    store
        .create_pending_registration_with_all_descriptor_requests(
            &registration,
            &capabilities,
            ModuleDescriptorRegistrationRequestsV1 {
                storage: std::slice::from_ref(&storage),
                events: &routes,
                blobs: &[client_blob, artifact_blob],
                scheduler: &[],
                vault_purposes: &[],
                client_rpc_routes: &client_rpc_routes,
                client_blob_routes: std::slice::from_ref(&client_blob_route),
                client_realtime_routes: std::slice::from_ref(&client_realtime_route),
                query_rpc_routes: &[],
                request_rpc_routes: &[],
                contract_dependencies: &[],
            },
        )
        .expect("record Communications Export registration");
    let grant_epoch = store
        .approve_module_registration(COMMUNICATIONS_EXPORT_REGISTRATION, &capabilities)
        .expect("approve Communications Export capabilities")
        .grant_epoch();
    let bundle = communications_export_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                COMMUNICATIONS_EXPORT_OWNER_V1,
                u64::from(COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record Communications Export Storage bundle"),
        )
        .expect("persist Communications Export Storage bundle");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            COMMUNICATIONS_EXPORT_REGISTRATION,
            1,
            "makosh-managed-runtime-conformance",
            "workflow.communications_export",
            Sha256::digest(
                std::fs::read(communications_export_binary())
                    .expect("Communications Export binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Communications Export release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            COMMUNICATIONS_EXPORT_REGISTRATION,
            COMMUNICATIONS_EXPORT_RUNTIME_INSTANCE_ID,
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record Communications Export reservation");
}

fn communications_export_event_route(
    kind: ModuleEventEnvelopeKindV1,
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    direction: ModuleEventRouteDirectionV1,
) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: COMMUNICATIONS_EXPORT_REGISTRATION.to_owned(),
            capability_id: COMMUNICATIONS_EXPORT_EVENTS_CAPABILITY_ID_V1.to_owned(),
            envelope_kind: kind,
            contract_owner: contract.owner.clone(),
            contract_name: contract.name.clone(),
            contract_major: contract.major,
            contract_revision: contract.revision,
            contract_schema_sha256: contract
                .schema_sha256
                .as_slice()
                .try_into()
                .expect("contract digest"),
            direction,
            max_in_flight: 16,
            delivery_policy: matches!(direction, ModuleEventRouteDirectionV1::Consume).then(|| {
                ModuleEventDeliveryPolicyV1::new(
                    ModuleEventSubscriptionRequirementV1::Required,
                    8,
                    30_000,
                )
            }),
        },
    )
}

fn communications_export_client_rpc_routes() -> [ModuleClientRpcRouteV1; 3] {
    [
        communications_export_client_rpc_route(
            COMMUNICATIONS_EXPORT_COMMAND_CONTRACT_NAME_V1,
            COMMUNICATIONS_EXPORT_COMMAND_CONNECT_PATH_V1,
        ),
        communications_export_client_rpc_route(
            COMMUNICATIONS_EXPORT_QUERY_CONTRACT_NAME_V1,
            COMMUNICATIONS_EXPORT_QUERY_CONNECT_PATH_V1,
        ),
        communications_export_client_rpc_route(
            COMMUNICATIONS_EXPORT_TICKET_CONTRACT_NAME_V1,
            COMMUNICATIONS_EXPORT_TICKET_CONNECT_PATH_V1,
        ),
    ]
}

fn communications_export_client_rpc_route(
    contract_name: &str,
    path: &str,
) -> ModuleClientRpcRouteV1 {
    ModuleClientRpcRouteV1::new(
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        contract_name,
        ModuleClientRpcContractVersionV1 {
            major: COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1,
            revision: COMMUNICATIONS_EXPORT_CONTRACT_REVISION_V1,
        },
        COMMUNICATIONS_EXPORT_SCHEMA_SHA256,
        path,
    )
}

fn communications_export_client_blob_route() -> ModuleClientBlobRouteV1 {
    ModuleClientBlobRouteV1::new(
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        COMMUNICATIONS_EXPORT_READ_CONTRACT_NAME_V1,
        ModuleClientBlobContractVersionV1 {
            major: COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1,
            revision: COMMUNICATIONS_EXPORT_CONTRACT_REVISION_V1,
        },
        COMMUNICATIONS_EXPORT_SCHEMA_SHA256,
        ModuleClientBlobTransportV1 {
            path: COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1.to_owned(),
            max_response_bytes: COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1,
        },
    )
}

fn communications_export_client_realtime_route() -> ModuleClientRealtimeRouteV1 {
    ModuleClientRealtimeRouteV1::new(
        COMMUNICATIONS_EXPORT_REGISTRATION,
        COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1,
        COMMUNICATIONS_EXPORT_OWNER_V1,
        COMMUNICATIONS_EXPORT_REALTIME_CONTRACT_NAME_V1,
        ModuleClientRealtimeContractVersionV1 {
            major: COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1,
            revision: COMMUNICATIONS_EXPORT_CONTRACT_REVISION_V1,
        },
        COMMUNICATIONS_EXPORT_SCHEMA_SHA256,
    )
}

fn communications_event_route(
    capability: &str,
    kind: ModuleEventEnvelopeKindV1,
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    direction: ModuleEventRouteDirectionV1,
) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: COMMUNICATIONS_REGISTRATION.to_owned(),
            capability_id: capability.to_owned(),
            envelope_kind: kind,
            contract_owner: contract.owner.clone(),
            contract_name: contract.name.clone(),
            contract_major: contract.major,
            contract_revision: contract.revision,
            contract_schema_sha256: contract
                .schema_sha256
                .as_slice()
                .try_into()
                .expect("contract digest"),
            direction,
            max_in_flight: 16,
            delivery_policy: matches!(direction, ModuleEventRouteDirectionV1::Consume).then(|| {
                ModuleEventDeliveryPolicyV1::new(
                    ModuleEventSubscriptionRequirementV1::Required,
                    8,
                    30_000,
                )
            }),
        },
    )
}

fn communications_event_hub_topology() -> PlatformEventHubTopologyV1 {
    let budgets = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ]
    .into_iter()
    .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
    .collect();
    PlatformEventHubTopologyV1::new(
        1,
        required("MAKOSH_COMMUNICATIONS_LIVE_NATS_ENDPOINT"),
        COMMUNICATIONS_OWNER_ID,
        1,
        budgets,
    )
}

pub(super) fn installed_communications_release(root: &Path) -> InstalledSignedBundle {
    InstalledSignedBundle::install(root, &communications_release_artifacts())
        .expect("install signed Communications release")
}

pub(super) fn communications_release_artifacts() -> Vec<SignedRuntimeArtifact> {
    vec![
        SignedRuntimeArtifact::new(
            "platform.storage",
            storage_binary(),
            descriptor("storage").encode_to_vec(),
        ),
        SignedRuntimeArtifact::new(
            "platform.vault",
            vault_binary(),
            descriptor("vault").encode_to_vec(),
        ),
        blob_release_artifact(),
        SignedRuntimeArtifact::new(
            "domain.communications",
            communications_binary(),
            communications_module_descriptor_v1("managed-communications-live").encode_to_vec(),
        )
        .with_settings_schema(communications_settings_schema_bytes_v1()),
        SignedRuntimeArtifact::new(
            "workflow.communications_export",
            communications_export_binary(),
            communications_export_module_descriptor_v1("managed-communications-export-live")
                .encode_to_vec(),
        )
        .with_settings_schema(communications_export_settings_schema_bytes_v1()),
    ]
}

pub(super) fn blob_release_artifact() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new("platform.blob", blob_binary(), blob_descriptor())
        .with_settings_schema(blob_settings_schema())
}

fn communications_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATIONS_RUNTIME_BIN")
}

fn communications_export_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATIONS_EXPORT_RUNTIME_BIN")
}

fn blob_binary() -> PathBuf {
    binary("MAKOSH_BLOB_SERVICE_BIN")
}

fn blob_settings_schema() -> Vec<u8> {
    makosh_runtime_protocol::v1::SettingsSchemaV1 {
        major: 1,
        revision: 1,
        ..Default::default()
    }
    .encode_to_vec()
}

fn blob_descriptor() -> Vec<u8> {
    let schema = blob_settings_schema();
    makosh_runtime_protocol::v1::ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "blob".to_owned(),
        owner_id: "blob".to_owned(),
        module_kind: makosh_runtime_protocol::v1::ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "managed-communications-blob".to_owned(),
        settings_schema_ref: Some(makosh_runtime_protocol::v1::SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: schema.len() as u64,
            sha256: Sha256::digest(schema).to_vec(),
        }),
        ..Default::default()
    }
    .encode_to_vec()
}

fn communications_stream_details(
    kind: event_topology::subject::EventStreamKindV1,
) -> (&'static str, &'static str) {
    match kind {
        event_topology::subject::EventStreamKindV1::Command => {
            ("MAKOSH_COMMAND_V1", "makosh.command.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Event => {
            ("MAKOSH_EVENT_V1", "makosh.event.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Observation => {
            ("MAKOSH_OBSERVATION_V1", "makosh.observation.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Result => {
            ("MAKOSH_RESULT_V1", "makosh.result.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Ack => ("MAKOSH_ACK_V1", "makosh.ack.v1.>"),
    }
}

fn communications_stream_for_subject(subject: &str) -> &'static str {
    if subject.starts_with("makosh.command.") {
        "MAKOSH_COMMAND_V1"
    } else if subject.starts_with("makosh.event.") {
        "MAKOSH_EVENT_V1"
    } else if subject.starts_with("makosh.observation.") {
        "MAKOSH_OBSERVATION_V1"
    } else if subject.starts_with("makosh.result.") {
        "MAKOSH_RESULT_V1"
    } else {
        "MAKOSH_ACK_V1"
    }
}
