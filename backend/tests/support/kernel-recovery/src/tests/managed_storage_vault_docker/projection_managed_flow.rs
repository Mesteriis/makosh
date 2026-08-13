//! Actual Search, Timeline and Graph projection, query, replay and restart contour.

use super::*;

use std::time::{Duration, Instant};

use makosh_consistency_api::{
    CONSISTENCY_CLIENT_CAPABILITY_ID_V1, CONSISTENCY_MODULE_ID_V1, CONSISTENCY_OWNER_ID_V1,
    consistency_contradictions_contract_reference_v1,
    wire::{ListConsistencyContradictionsRequestV1, ListConsistencyContradictionsResultV1},
};
use makosh_decisions_api::{
    DECISIONS_MODULE_ID_V1, DecisionsEnvelopeContextV1, build_decision_changed_outbox_record_v1,
    client_wire::{DecisionChangedV1, DecisionStateV1, TimestampV1 as DecisionTimestampV1},
};
use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::{delivery::OutboxRecordV1, v1::DurableEnvelopeV1};
use makosh_graph_api::{
    GRAPH_CLIENT_CAPABILITY_ID_V1, GRAPH_MODULE_ID_V1, GRAPH_OWNER_ID_V1,
    graph_neighbors_contract_reference_v1,
    wire::{GraphNeighborsRequestV1, GraphNeighborsResultV1, GraphNodeRefV1},
};
use makosh_memory_api::{
    MEMORY_CLIENT_CAPABILITY_ID_V1, MEMORY_MODULE_ID_V1, MEMORY_OWNER_ID_V1,
    memory_list_contract_reference_v1,
    wire::{ListMemoryRequestV1, ListMemoryResultV1},
};
use makosh_organizations_api::{
    ORGANIZATIONS_MODULE_ID_V1, OrganizationsEnvelopeContextV1,
    build_organization_changed_outbox_record_v1,
    client_wire::{OrganizationChangedV1, OrganizationStateV1, TimestampV1},
};
use makosh_relationships_api::{
    RelationshipsEnvelopeContextV1, build_relationship_changed_outbox_record_v1,
    client_wire::{
        RelationshipChangedV1, RelationshipParticipantKindV1, RelationshipParticipantV1,
        RelationshipStateV1, RelationshipTypeV1, TimestampV1 as RelationshipTimestampV1,
    },
};
use makosh_risk_api::{
    RISK_CLIENT_CAPABILITY_ID_V1, RISK_MODULE_ID_V1, RISK_OWNER_ID_V1,
    risk_list_contract_reference_v1,
    wire::{ListRiskRequestV1, ListRiskResultV1},
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use makosh_search_api::{
    SEARCH_CLIENT_CAPABILITY_ID_V1, SEARCH_MODULE_ID_V1, SEARCH_OWNER_ID_V1,
    search_query_contract_reference_v1,
    wire::{SearchQueryResultV1, SearchQueryV1},
};
use makosh_tasks_command_api::{
    TASKS_MODULE_ID_V1, TasksCommandEnvelopeContextV1, build_task_changed_outbox_record_v1,
    client_wire::{TaskChangedV1, TaskPriorityV1, TaskStateV1, TimestampV1 as TaskTimestampV1},
};
use makosh_timeline_api::{
    TIMELINE_CLIENT_CAPABILITY_ID_V1, TIMELINE_MODULE_ID_V1, TIMELINE_OWNER_ID_V1,
    timeline_list_contract_reference_v1,
    wire::{ListTimelineRequestV1, ListTimelineResultV1},
};

use crate::identity::device::signer::DeviceSigner;

const PRIVATE_MARKER_V1: &[u8] = b"projection-private-provider-marker";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and projection binaries"]
fn managed_search_timeline_graph_project_query_replay_and_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-projections");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_projection_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("projection owner signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            PROJECTION_HUMAN_OWNER_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim projection owner");
    let admitted = [
        ProjectionKindV1::Search,
        ProjectionKindV1::Timeline,
        ProjectionKindV1::Graph,
    ]
    .map(|kind| admit_projection_v1(&store, kind));
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure projection Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = admitted.map(|value| prepare_projection_v1(&supervisor, &store, value));
    record_communications_event_hub_topology_v1(&store);
    configure_communications_jetstream(&store);
    let [search, timeline, graph] = admitted
        .map(|value| start_projection_v1(&supervisor, &store, &root.join("runtime"), value));

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("projection clock")
        .as_secs() as i64
        - 1;
    let organization = build_organization_changed_outbox_record_v1(
        [0x10; 16],
        OrganizationChangedV1 {
            event_id: vec![0x11; 16],
            organization_id: vec![0x12; 16],
            logical_owner_id: PROJECTION_HUMAN_OWNER_V1.to_owned(),
            organization_revision: 1,
            state: OrganizationStateV1::OrganizationStateActive as i32,
            occurred_at: Some(TimestampV1 {
                unix_seconds: now,
                nanos: 0,
            }),
        },
        &OrganizationsEnvelopeContextV1 {
            module_id: ORGANIZATIONS_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "projection-organizations-source".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .expect("organization event");
    let relationship = build_relationship_changed_outbox_record_v1(
        [0x20; 16],
        RelationshipChangedV1 {
            event_id: vec![0x21; 16],
            relationship_id: vec![0x22; 16],
            logical_owner_id: PROJECTION_HUMAN_OWNER_V1.to_owned(),
            source: Some(RelationshipParticipantV1 {
                kind: RelationshipParticipantKindV1::RelationshipParticipantKindPerson as i32,
                public_id: vec![0x23; 16],
            }),
            target: Some(RelationshipParticipantV1 {
                kind: RelationshipParticipantKindV1::RelationshipParticipantKindOrganization as i32,
                public_id: vec![0x12; 16],
            }),
            relationship_type: RelationshipTypeV1::RelationshipTypeMemberOf as i32,
            state: RelationshipStateV1::RelationshipStateConfirmed as i32,
            valid_from: Some(RelationshipTimestampV1 {
                unix_seconds: now,
                nanos: 0,
            }),
            valid_until: None,
            relationship_revision: 1,
            occurred_at: Some(RelationshipTimestampV1 {
                unix_seconds: now,
                nanos: 0,
            }),
        },
        &RelationshipsEnvelopeContextV1 {
            runtime_instance_id: "projection-relationships-source".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .expect("relationship event");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read projection Event Hub")
        .expect("projection Event Hub")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("projection publish runtime")
        .block_on(async {
            let context = async_nats::jetstream::new(
                async_nats::connect(&endpoint)
                    .await
                    .expect("connect projection JetStream"),
            );
            publish_projection_record_v1(&context, &organization).await;
            publish_projection_record_v1(&context, &relationship).await;
        });
    wait_for_projection_counts_v1((2, 2, 1));

    let search_before: SearchQueryResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &search,
        SEARCH_MODULE_ID_V1,
        SEARCH_OWNER_ID_V1,
        SEARCH_CLIENT_CAPABILITY_ID_V1,
        1,
        search_query_contract_reference_v1(),
        SearchQueryV1 {
            logical_owner_id: String::new(),
            query: "organizations organization organization_state_active".to_owned(),
            after_cursor: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(search_before.hits.len(), 1);
    let timeline_before: ListTimelineResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &timeline,
        TIMELINE_MODULE_ID_V1,
        TIMELINE_OWNER_ID_V1,
        TIMELINE_CLIENT_CAPABILITY_ID_V1,
        2,
        timeline_list_contract_reference_v1(),
        ListTimelineRequestV1 {
            logical_owner_id: String::new(),
            after_cursor: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(timeline_before.entries.len(), 2);
    let graph_before: GraphNeighborsResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &graph,
        GRAPH_MODULE_ID_V1,
        GRAPH_OWNER_ID_V1,
        GRAPH_CLIENT_CAPABILITY_ID_V1,
        3,
        graph_neighbors_contract_reference_v1(),
        GraphNeighborsRequestV1 {
            logical_owner_id: String::new(),
            node: Some(GraphNodeRefV1 {
                node_owner: "persons".to_owned(),
                node_kind: "person".to_owned(),
                node_id: vec![0x23; 16],
            }),
            after_edge_id: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(graph_before.edges.len(), 1);

    let search = restart_projection_v1(&supervisor, &store, &root.join("runtime"), search);
    let timeline = restart_projection_v1(&supervisor, &store, &root.join("runtime"), timeline);
    let graph = restart_projection_v1(&supervisor, &store, &root.join("runtime"), graph);
    wait_for_projection_counts_v1((2, 2, 1));
    let search_after: SearchQueryResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &search,
        SEARCH_MODULE_ID_V1,
        SEARCH_OWNER_ID_V1,
        SEARCH_CLIENT_CAPABILITY_ID_V1,
        4,
        search_query_contract_reference_v1(),
        SearchQueryV1 {
            logical_owner_id: String::new(),
            query: "organizations organization organization_state_active".to_owned(),
            after_cursor: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(search_before, search_after);

    assert_projection_private_free_v1();
    let started = [search, timeline, graph];
    for runtime in &started {
        let before = Instant::now();
        assert!(
            supervisor
                .request_stop_if_active(&runtime.registration_id)
                .expect("request projection stop")
        );
        assert!(
            supervisor
                .stop_if_active(&runtime.registration_id)
                .expect("join projection stop")
        );
        assert!(before.elapsed() < Duration::from_secs(2));
        assert_eq!(
            supervisor
                .last_failure(&runtime.registration_id)
                .expect("projection failure state"),
            None
        );
    }
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and projection binaries"]
fn managed_memory_consistency_risk_project_query_replay_and_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-derived-engines");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_projection_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    let (owner_signer, _) = FileDeviceSigner::open_or_create_for_instance(&data).unwrap();
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            PROJECTION_HUMAN_OWNER_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .unwrap();
    let admitted = [
        ProjectionKindV1::Memory,
        ProjectionKindV1::Consistency,
        ProjectionKindV1::Risk,
    ]
    .map(|kind| admit_projection_v1(&store, kind));
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .unwrap();
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = admitted.map(|value| prepare_projection_v1(&supervisor, &store, value));
    record_communications_event_hub_topology_v1(&store);
    configure_communications_jetstream(&store);
    let [memory, consistency, risk] = admitted
        .map(|value| start_projection_v1(&supervisor, &store, &root.join("runtime"), value));

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - 1;
    let decision = build_decision_changed_outbox_record_v1(
        [0x40; 16],
        DecisionChangedV1 {
            event_id: vec![0x41; 16],
            decision_id: vec![0x42; 16],
            logical_owner_id: PROJECTION_HUMAN_OWNER_V1.into(),
            decision_revision: 1,
            state: DecisionStateV1::DecisionStateDecided as i32,
            occurred_at: Some(DecisionTimestampV1 {
                unix_seconds: now,
                nanos: 0,
            }),
        },
        &DecisionsEnvelopeContextV1 {
            module_id: DECISIONS_MODULE_ID_V1.into(),
            runtime_instance_id: "memory-decisions-source".into(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .unwrap();
    let task = build_task_changed_outbox_record_v1(
        [0x50; 16],
        TaskChangedV1 {
            event_id: vec![0x51; 16],
            task_id: vec![0x52; 16],
            logical_owner_id: PROJECTION_HUMAN_OWNER_V1.into(),
            task_revision: 1,
            state: TaskStateV1::TaskStateOpen as i32,
            priority: TaskPriorityV1::TaskPriorityNormal as i32,
            occurred_at: Some(TaskTimestampV1 {
                unix_seconds: now,
                nanos: 0,
            }),
        },
        &TasksCommandEnvelopeContextV1 {
            module_id: TASKS_MODULE_ID_V1.into(),
            runtime_instance_id: "risk-tasks-source".into(),
            runtime_generation: 1,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .unwrap();
    let relation = |event: u8, relation: u8, target: u8| {
        build_relationship_changed_outbox_record_v1(
            [event.wrapping_sub(1); 16],
            RelationshipChangedV1 {
                event_id: vec![event; 16],
                relationship_id: vec![relation; 16],
                logical_owner_id: PROJECTION_HUMAN_OWNER_V1.into(),
                source: Some(RelationshipParticipantV1 {
                    kind: RelationshipParticipantKindV1::RelationshipParticipantKindPerson as i32,
                    public_id: vec![0x60; 16],
                }),
                target: Some(RelationshipParticipantV1 {
                    kind: RelationshipParticipantKindV1::RelationshipParticipantKindOrganization
                        as i32,
                    public_id: vec![target; 16],
                }),
                relationship_type: RelationshipTypeV1::RelationshipTypeMemberOf as i32,
                state: RelationshipStateV1::RelationshipStateConfirmed as i32,
                valid_from: Some(RelationshipTimestampV1 {
                    unix_seconds: now,
                    nanos: 0,
                }),
                valid_until: None,
                relationship_revision: u64::from(relation),
                occurred_at: Some(RelationshipTimestampV1 {
                    unix_seconds: now,
                    nanos: 0,
                }),
            },
            &RelationshipsEnvelopeContextV1 {
                runtime_instance_id: "consistency-relationships-source".into(),
                runtime_generation: 1,
                recorded_at_unix_seconds: now,
                recorded_at_nanos: 0,
            },
        )
        .unwrap()
    };
    let first = relation(0x61, 1, 0x62);
    let second = relation(0x63, 2, 0x64);
    let endpoint = store
        .platform_event_hub_topology()
        .unwrap()
        .unwrap()
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let context = async_nats::jetstream::new(async_nats::connect(&endpoint).await.unwrap());
        for record in [&decision, &task, &first, &second] {
            publish_projection_record_v1(&context, record).await;
        }
    });
    wait_for_derived_projection_counts_v1((1, 2, 1));

    let memory_before: ListMemoryResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &memory,
        MEMORY_MODULE_ID_V1,
        MEMORY_OWNER_ID_V1,
        MEMORY_CLIENT_CAPABILITY_ID_V1,
        10,
        memory_list_contract_reference_v1(),
        ListMemoryRequestV1 {
            logical_owner_id: String::new(),
            after_cursor: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(memory_before.entries.len(), 1);
    let consistency_before: ListConsistencyContradictionsResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &consistency,
        CONSISTENCY_MODULE_ID_V1,
        CONSISTENCY_OWNER_ID_V1,
        CONSISTENCY_CLIENT_CAPABILITY_ID_V1,
        11,
        consistency_contradictions_contract_reference_v1(),
        ListConsistencyContradictionsRequestV1 {
            logical_owner_id: String::new(),
            after_first_claim_id: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(consistency_before.contradictions.len(), 1);
    let risk_before: ListRiskResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &risk,
        RISK_MODULE_ID_V1,
        RISK_OWNER_ID_V1,
        RISK_CLIENT_CAPABILITY_ID_V1,
        12,
        risk_list_contract_reference_v1(),
        ListRiskRequestV1 {
            logical_owner_id: String::new(),
            after_cursor: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(risk_before.entries.len(), 1);

    let memory = restart_projection_v1(&supervisor, &store, &root.join("runtime"), memory);
    let consistency =
        restart_projection_v1(&supervisor, &store, &root.join("runtime"), consistency);
    let risk = restart_projection_v1(&supervisor, &store, &root.join("runtime"), risk);
    wait_for_derived_projection_counts_v1((1, 2, 1));
    let memory_after: ListMemoryResultV1 = route_projection_v1(
        &store,
        &supervisor,
        &memory,
        MEMORY_MODULE_ID_V1,
        MEMORY_OWNER_ID_V1,
        MEMORY_CLIENT_CAPABILITY_ID_V1,
        13,
        memory_list_contract_reference_v1(),
        ListMemoryRequestV1 {
            logical_owner_id: String::new(),
            after_cursor: Vec::new(),
            limit: 10,
        }
        .encode_to_vec(),
    );
    assert_eq!(memory_before, memory_after);
    assert_derived_projection_private_free_v1();
    for runtime in [memory, consistency, risk] {
        let before = Instant::now();
        assert!(
            supervisor
                .request_stop_if_active(&runtime.registration_id)
                .unwrap()
        );
        assert!(supervisor.stop_if_active(&runtime.registration_id).unwrap());
        assert!(before.elapsed() < Duration::from_secs(2));
        assert_eq!(
            supervisor.last_failure(&runtime.registration_id).unwrap(),
            None
        );
    }
}

async fn publish_projection_record_v1(
    context: &async_nats::jetstream::Context,
    record: &OutboxRecordV1,
) {
    let envelope =
        DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode projection event");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive projection subject")
        .as_str();
    context
        .publish(subject, record.exact_bytes().to_vec().into())
        .await
        .expect("publish projection event")
        .await
        .expect("ack projection event");
}

#[allow(clippy::too_many_arguments)]
fn route_projection_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    runtime: &StartedProjectionV1,
    module_id: &str,
    owner_id: &str,
    capability_id: &str,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> T {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: module_id.to_owned(),
        owner_id: owner_id.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload: payload,
        logical_owner_id: PROJECTION_HUMAN_OWNER_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &runtime.registration_id,
        &runtime.runtime_instance_id,
        runtime.runtime_generation,
        runtime.grant_epoch,
        capability_id,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated projection request");
    let response = ModuleClientResponseV1::decode(bytes.as_slice()).expect("projection response");
    assert!(response.error_code.is_empty(), "{}", response.error_code);
    T::decode(response.response_payload.as_slice()).expect("decode projection response")
}

fn wait_for_projection_counts_v1(expected: (i64, i64, i64)) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let actual = tokio::runtime::Runtime::new()
            .expect("projection SQL runtime")
            .block_on(async {
                sqlx::query_as::<_, (i64, i64, i64)>(
                    "SELECT \
                     (SELECT count(*) FROM makosh_data.search_projection_inbox WHERE logical_owner_id='owner-1'), \
                     (SELECT count(*) FROM makosh_data.timeline_projection_inbox WHERE logical_owner_id='owner-1'), \
                     (SELECT count(*) FROM makosh_data.graph_projection_inbox WHERE logical_owner_id='owner-1')",
                )
                .fetch_one(&authenticated_storage_admin_pool_v1().await)
                .await
                .expect("projection durable counts")
            });
        if actual == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "projections did not converge: actual={actual:?} expected={expected:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn wait_for_derived_projection_counts_v1(expected: (i64, i64, i64)) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let actual = tokio::runtime::Runtime::new().unwrap().block_on(async {
            sqlx::query_as::<_, (i64, i64, i64)>(
                "SELECT \
                 (SELECT count(*) FROM makosh_data.memory_projection_inbox WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.consistency_projection_inbox WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.risk_projection_inbox WHERE logical_owner_id='owner-1')",
            )
            .fetch_one(&authenticated_storage_admin_pool_v1().await)
            .await
            .unwrap()
        });
        if actual == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "derived projections did not converge: actual={actual:?} expected={expected:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn assert_projection_private_free_v1() {
    tokio::runtime::Runtime::new()
        .expect("projection privacy runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            for query in [
                "SELECT envelope_bytes FROM makosh_data.search_projection_inbox",
                "SELECT envelope_bytes FROM makosh_data.timeline_projection_inbox",
                "SELECT envelope_bytes FROM makosh_data.graph_projection_inbox",
            ] {
                let rows: Vec<Vec<u8>> = sqlx::query_scalar(query)
                    .fetch_all(&pool)
                    .await
                    .expect("projection privacy rows");
                assert!(!rows.is_empty());
                for row in rows {
                    assert!(
                        !row.windows(PRIVATE_MARKER_V1.len())
                            .any(|value| value == PRIVATE_MARKER_V1)
                    );
                }
            }
        });
}

fn assert_derived_projection_private_free_v1() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let pool = authenticated_storage_admin_pool_v1().await;
        for query in [
            "SELECT envelope_bytes FROM makosh_data.memory_projection_inbox",
            "SELECT envelope_bytes FROM makosh_data.consistency_projection_inbox",
            "SELECT envelope_bytes FROM makosh_data.risk_projection_inbox",
        ] {
            let rows: Vec<Vec<u8>> = sqlx::query_scalar(query).fetch_all(&pool).await.unwrap();
            assert!(!rows.is_empty());
            for row in rows {
                assert!(
                    !row.windows(PRIVATE_MARKER_V1.len())
                        .any(|value| value == PRIVATE_MARKER_V1)
                );
            }
        }
    });
}
