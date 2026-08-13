use makosh_consistency_core::{ConsistencyEdgeV1, ConsistencyNodeV1, contradictions_v1};
use makosh_consistency_persistence::{
    ApplyConsistencyMutationV1, CONSISTENCY_SCHEMA_V1, ConsistencyEnvelopeRecordV1,
    ConsistencyMutationV1, ConsistencyPersistenceErrorV1, ConsistencyPersistenceV1,
    ConsistencyReplayOutcomeV1,
};
use makosh_graph_core::{GraphEdgeV1, GraphNodeV1};
use makosh_graph_persistence::{
    ApplyGraphMutationV1, GRAPH_SCHEMA_V1, GraphEnvelopeRecordV1, GraphMutationV1,
    GraphPersistenceErrorV1, GraphPersistenceV1, GraphReplayOutcomeV1,
};
use makosh_memory_core::MemoryProjectionEntryV1;
use makosh_memory_persistence::{
    ApplyMemoryEntryV1, MEMORY_SCHEMA_V1, MemoryEnvelopeRecordV1, MemoryPersistenceErrorV1,
    MemoryPersistenceV1, MemoryReplayOutcomeV1,
};
use makosh_omniroute_core::OmniRouteRequestReceiptV1;
use makosh_omniroute_persistence::{
    OMNIROUTE_SCHEMA_V1, OmniRoutePersistenceErrorV1, OmniRoutePersistenceV1,
    OmniRouteReplayOutcomeV1,
};
use makosh_risk_core::RiskProjectionEntryV1;
use makosh_risk_persistence::{
    ApplyRiskEntryV1, RISK_SCHEMA_V1, RiskEnvelopeRecordV1, RiskPersistenceErrorV1,
    RiskPersistenceV1, RiskReplayOutcomeV1,
};
use makosh_search_core::{SearchProjectionDocumentV1, search_query_token_digests_v1};
use makosh_search_persistence::{
    ApplySearchDocumentV1, SEARCH_SCHEMA_V1, SearchEnvelopeRecordV1, SearchPersistenceErrorV1,
    SearchPersistenceV1, SearchReplayOutcomeV1,
};
use makosh_telemost_persistence::{
    TELEMOST_SCHEMA_V1, TelemostAccountRecordV1, TelemostObservationRecordV1,
    TelemostPersistenceErrorV1, TelemostPersistenceV1, TelemostReplayOutcomeV1,
};
use makosh_timeline_core::TimelineProjectionEntryV1;
use makosh_timeline_persistence::{
    ApplyTimelineEntryV1, TIMELINE_SCHEMA_V1, TimelineEnvelopeRecordV1, TimelinePersistenceErrorV1,
    TimelinePersistenceV1, TimelineReplayOutcomeV1,
};
use makosh_zoom_persistence::{
    ZOOM_SCHEMA_V1, ZoomAccountRecordV1, ZoomObservationRecordV1, ZoomPersistenceErrorV1,
    ZoomPersistenceV1, ZoomReplayOutcomeV1,
};
use sha2::{Digest, Sha256};
use sqlx::{Executor, PgPool, Row, postgres::PgPoolOptions};

const OWNER_A: &str = "owner-a";
const OWNER_B: &str = "owner-b";

fn envelope(seed: u8) -> ([u8; 16], [u8; 32], Vec<u8>) {
    let bytes = vec![seed, seed.wrapping_add(1), seed.wrapping_add(2)];
    ([seed; 16], Sha256::digest(&bytes).into(), bytes)
}

fn search_input(
    owner: &str,
    seed: u8,
    generation: u64,
    revision: u64,
    deleted: bool,
) -> ApplySearchDocumentV1 {
    let (message_id, envelope_sha256, envelope_bytes) = envelope(seed);
    ApplySearchDocumentV1 {
        input: SearchEnvelopeRecordV1 {
            message_id,
            envelope_sha256,
            envelope_bytes,
        },
        projection_generation: generation,
        document: SearchProjectionDocumentV1 {
            logical_owner_id: owner.to_owned(),
            source_owner: "persons".to_owned(),
            entity_kind: "person".to_owned(),
            entity_id: [7; 16],
            source_revision: revision,
            lifecycle_state: if deleted {
                String::new()
            } else {
                "active".to_owned()
            },
            occurred_at_unix_millis: 1_000 + i64::try_from(revision).unwrap(),
            deleted,
        },
        token_digests: if deleted {
            Vec::new()
        } else {
            search_query_token_digests_v1(&[9; 32], "public person").unwrap()
        },
        completed_at_unix_millis: 2_000 + i64::try_from(revision).unwrap(),
    }
}

fn timeline_input(
    owner: &str,
    seed: u8,
    generation: u64,
    revision: u64,
    tombstone: bool,
) -> ApplyTimelineEntryV1 {
    let (message_id, envelope_sha256, envelope_bytes) = envelope(seed);
    ApplyTimelineEntryV1 {
        input: TimelineEnvelopeRecordV1 {
            message_id,
            envelope_sha256,
            envelope_bytes,
        },
        projection_generation: generation,
        entry: TimelineProjectionEntryV1 {
            event_id: message_id,
            logical_owner_id: owner.to_owned(),
            source_owner: "persons".to_owned(),
            entity_kind: "person".to_owned(),
            entity_id: [7; 16],
            source_revision: revision,
            lifecycle_state: if tombstone {
                String::new()
            } else {
                "active".to_owned()
            },
            occurred_at_unix_millis: 1_000 + i64::try_from(revision).unwrap(),
            tombstone,
        },
        completed_at_unix_millis: 2_000 + i64::try_from(revision).unwrap(),
    }
}

fn graph_input(
    owner: &str,
    seed: u8,
    generation: u64,
    revision: u64,
    deleted: bool,
) -> ApplyGraphMutationV1 {
    let (message_id, envelope_sha256, envelope_bytes) = envelope(seed);
    let source = GraphNodeV1 {
        owner: "persons".to_owned(),
        kind: "person".to_owned(),
        id: [7; 16],
    };
    let mutation = if deleted {
        GraphMutationV1::UpsertNode {
            node: source,
            source_revision: revision,
            deleted: true,
        }
    } else {
        GraphMutationV1::UpsertEdge(GraphEdgeV1 {
            edge_id: [8; 16],
            logical_owner_id: owner.to_owned(),
            source,
            target: GraphNodeV1 {
                owner: "persons".to_owned(),
                kind: "person".to_owned(),
                id: [9; 16],
            },
            edge_kind: "confirmed_relationship".to_owned(),
            source_revision: revision,
            occurred_at_unix_millis: 1_000 + i64::try_from(revision).unwrap(),
            deleted: false,
        })
    };
    ApplyGraphMutationV1 {
        input: GraphEnvelopeRecordV1 {
            message_id,
            envelope_sha256,
            envelope_bytes,
        },
        projection_generation: generation,
        logical_owner_id: owner.to_owned(),
        source_owner: "relationships".to_owned(),
        source_revision: revision,
        mutation,
        completed_at_unix_millis: 2_000 + i64::try_from(revision).unwrap(),
    }
}

fn memory_input(
    owner: &str,
    seed: u8,
    generation: u64,
    revision: u64,
    tombstone: bool,
) -> ApplyMemoryEntryV1 {
    let (message_id, envelope_sha256, envelope_bytes) = envelope(seed);
    ApplyMemoryEntryV1 {
        input: MemoryEnvelopeRecordV1 {
            message_id,
            envelope_sha256,
            envelope_bytes,
        },
        projection_generation: generation,
        entry: MemoryProjectionEntryV1 {
            event_id: message_id,
            logical_owner_id: owner.into(),
            source_owner: "knowledge".into(),
            entity_kind: "knowledge_item".into(),
            entity_id: [71; 16],
            source_revision: revision,
            memory_kind: if tombstone {
                String::new()
            } else {
                "verified_knowledge".into()
            },
            occurred_at_unix_millis: 10_000 + i64::try_from(revision).unwrap(),
            tombstone,
        },
        completed_at_unix_millis: 11_000 + i64::try_from(revision).unwrap(),
    }
}

fn risk_input(
    owner: &str,
    seed: u8,
    generation: u64,
    revision: u64,
    cleared: bool,
) -> ApplyRiskEntryV1 {
    let (message_id, envelope_sha256, envelope_bytes) = envelope(seed);
    ApplyRiskEntryV1 {
        input: RiskEnvelopeRecordV1 {
            message_id,
            envelope_sha256,
            envelope_bytes,
        },
        projection_generation: generation,
        entry: RiskProjectionEntryV1 {
            event_id: message_id,
            logical_owner_id: owner.into(),
            source_owner: "obligations".into(),
            entity_kind: "obligation".into(),
            entity_id: [72; 16],
            source_revision: revision,
            reason_code: if cleared {
                String::new()
            } else {
                "breached_obligation".into()
            },
            severity: if cleared { 0 } else { 5 },
            occurred_at_unix_millis: 10_000 + i64::try_from(revision).unwrap(),
            expires_at_unix_millis: if cleared {
                0
            } else {
                20_000 + i64::try_from(revision).unwrap()
            },
            cleared,
        },
        completed_at_unix_millis: 11_000 + i64::try_from(revision).unwrap(),
    }
}

fn consistency_input(
    owner: &str,
    seed: u8,
    generation: u64,
    revision: u64,
    target: u8,
) -> ApplyConsistencyMutationV1 {
    let (message_id, envelope_sha256, envelope_bytes) = envelope(seed);
    ApplyConsistencyMutationV1 {
        input: ConsistencyEnvelopeRecordV1 {
            message_id,
            envelope_sha256,
            envelope_bytes,
        },
        projection_generation: generation,
        logical_owner_id: owner.into(),
        source_owner: "relationships".into(),
        source_revision: revision,
        mutation: ConsistencyMutationV1::UpsertEdge(ConsistencyEdgeV1 {
            edge_id: [seed; 16],
            logical_owner_id: owner.into(),
            source: ConsistencyNodeV1 {
                owner: "persons".into(),
                kind: "person".into(),
                id: [73; 16],
            },
            target: ConsistencyNodeV1 {
                owner: "persons".into(),
                kind: "person".into(),
                id: [target; 16],
            },
            edge_kind: "confirmed_relationship".into(),
            source_revision: revision,
            occurred_at_unix_millis: 10_000 + i64::try_from(revision).unwrap(),
            deleted: false,
        }),
        completed_at_unix_millis: 11_000 + i64::try_from(revision).unwrap(),
    }
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn projection_rebuild_replay_deletion_restart_and_rls_are_durable() {
    let url = std::env::var("MAKOSH_PROJECTION_POSTGRES_URL").expect("managed disposable URL");
    let pool = PgPoolOptions::new()
        .max_connections(12)
        .connect(&url)
        .await
        .expect("admin pool");
    pool.execute("CREATE SCHEMA IF NOT EXISTS makosh_data")
        .await
        .expect("schema namespace");
    sqlx::raw_sql(std::str::from_utf8(SEARCH_SCHEMA_V1).unwrap())
        .execute(&pool)
        .await
        .expect("search schema");
    sqlx::raw_sql(std::str::from_utf8(TIMELINE_SCHEMA_V1).unwrap())
        .execute(&pool)
        .await
        .expect("timeline schema");
    sqlx::raw_sql(std::str::from_utf8(GRAPH_SCHEMA_V1).unwrap())
        .execute(&pool)
        .await
        .expect("graph schema");

    let search = SearchPersistenceV1::new(pool.clone());
    let timeline = TimelinePersistenceV1::new(pool.clone());
    let graph = GraphPersistenceV1::new(pool.clone());
    assert_eq!(
        search.ensure_live_generation(OWNER_A, 900).await.unwrap(),
        1
    );
    assert_eq!(
        timeline.ensure_live_generation(OWNER_A, 900).await.unwrap(),
        1
    );
    assert_eq!(graph.ensure_live_generation(OWNER_A, 900).await.unwrap(), 1);

    let search_live = search_input(OWNER_A, 1, 1, 1, false);
    let timeline_live = timeline_input(OWNER_A, 2, 1, 1, false);
    let graph_live = graph_input(OWNER_A, 3, 1, 1, false);
    assert_eq!(
        search.apply_document_once(&search_live).await.unwrap(),
        SearchReplayOutcomeV1::Applied
    );
    assert_eq!(
        timeline.apply_entry_once(&timeline_live).await.unwrap(),
        TimelineReplayOutcomeV1::Applied
    );
    assert_eq!(
        graph.apply_once(&graph_live).await.unwrap(),
        GraphReplayOutcomeV1::Applied
    );
    assert_eq!(
        search.apply_document_once(&search_live).await.unwrap(),
        SearchReplayOutcomeV1::Replayed
    );
    assert_eq!(
        timeline.apply_entry_once(&timeline_live).await.unwrap(),
        TimelineReplayOutcomeV1::Replayed
    );
    assert_eq!(
        graph.apply_once(&graph_live).await.unwrap(),
        GraphReplayOutcomeV1::Replayed
    );

    let mut changed = search_live.clone();
    changed.input.envelope_bytes.push(99);
    changed.input.envelope_sha256 = Sha256::digest(&changed.input.envelope_bytes).into();
    assert_eq!(
        search.apply_document_once(&changed).await,
        Err(SearchPersistenceErrorV1::Conflict)
    );
    let mut changed = timeline_live.clone();
    changed.input.envelope_bytes.push(99);
    changed.input.envelope_sha256 = Sha256::digest(&changed.input.envelope_bytes).into();
    assert_eq!(
        timeline.apply_entry_once(&changed).await,
        Err(TimelinePersistenceErrorV1::Conflict)
    );
    let mut changed = graph_live.clone();
    changed.input.envelope_bytes.push(99);
    changed.input.envelope_sha256 = Sha256::digest(&changed.input.envelope_bytes).into();
    assert_eq!(
        graph.apply_once(&changed).await,
        Err(GraphPersistenceErrorV1::Conflict)
    );

    let search_generation = search.start_rebuild(OWNER_A, 1, 3_000).await.unwrap();
    let timeline_generation = timeline.start_rebuild(OWNER_A, 1, 3_000).await.unwrap();
    let graph_generation = graph.start_rebuild(OWNER_A, 1, 3_000).await.unwrap();
    assert_eq!(
        (search_generation, timeline_generation, graph_generation),
        (2, 2, 2)
    );
    search
        .apply_document_once(&search_input(OWNER_A, 11, 2, 1, false))
        .await
        .unwrap();
    timeline
        .apply_entry_once(&timeline_input(OWNER_A, 12, 2, 1, false))
        .await
        .unwrap();
    graph
        .apply_once(&graph_input(OWNER_A, 13, 2, 1, false))
        .await
        .unwrap();
    assert_eq!(
        search
            .projection_status(OWNER_A)
            .await
            .unwrap()
            .active_generation,
        1
    );
    assert_eq!(timeline.status(OWNER_A).await.unwrap().active_generation, 1);
    assert_eq!(graph.status(OWNER_A).await.unwrap().active_generation, 1);

    search.complete_rebuild(OWNER_A, 2, 4_000).await.unwrap();
    timeline.complete_rebuild(OWNER_A, 2, 4_000).await.unwrap();
    graph.complete_rebuild(OWNER_A, 2, 4_000).await.unwrap();
    assert_eq!(
        search
            .projection_status(OWNER_A)
            .await
            .unwrap()
            .active_generation,
        2
    );
    assert_eq!(timeline.status(OWNER_A).await.unwrap().active_generation, 2);
    assert_eq!(graph.status(OWNER_A).await.unwrap().active_generation, 2);

    let search_partial = search.start_rebuild(OWNER_A, 2, 5_000).await.unwrap();
    let timeline_partial = timeline.start_rebuild(OWNER_A, 2, 5_000).await.unwrap();
    let graph_partial = graph.start_rebuild(OWNER_A, 2, 5_000).await.unwrap();
    search
        .apply_document_once(&search_input(OWNER_A, 21, search_partial, 1, false))
        .await
        .unwrap();
    timeline
        .apply_entry_once(&timeline_input(OWNER_A, 22, timeline_partial, 1, false))
        .await
        .unwrap();
    graph
        .apply_once(&graph_input(OWNER_A, 23, graph_partial, 1, false))
        .await
        .unwrap();
    assert_eq!(
        search
            .complete_rebuild(OWNER_A, search_partial, 6_000)
            .await,
        Err(SearchPersistenceErrorV1::Conflict)
    );
    assert_eq!(
        timeline
            .complete_rebuild(OWNER_A, timeline_partial, 6_000)
            .await,
        Err(TimelinePersistenceErrorV1::Conflict)
    );
    assert_eq!(
        graph.complete_rebuild(OWNER_A, graph_partial, 6_000).await,
        Err(GraphPersistenceErrorV1::Conflict)
    );
    assert_eq!(
        search
            .projection_status(OWNER_A)
            .await
            .unwrap()
            .active_generation,
        2
    );
    assert_eq!(timeline.status(OWNER_A).await.unwrap().active_generation, 2);
    assert_eq!(graph.status(OWNER_A).await.unwrap().active_generation, 2);

    drop(search);
    drop(timeline);
    drop(graph);
    let search = SearchPersistenceV1::new(pool.clone());
    let timeline = TimelinePersistenceV1::new(pool.clone());
    let graph = GraphPersistenceV1::new(pool.clone());
    search
        .apply_document_once(&search_input(OWNER_A, 31, 2, 2, true))
        .await
        .unwrap();
    timeline
        .apply_entry_once(&timeline_input(OWNER_A, 32, 2, 2, true))
        .await
        .unwrap();
    graph
        .apply_once(&graph_input(OWNER_A, 33, 2, 2, true))
        .await
        .unwrap();
    assert!(
        search
            .query_active(
                OWNER_A,
                &search_query_token_digests_v1(&[9; 32], "public person").unwrap(),
                None,
                10
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        timeline
            .list_active(OWNER_A, None, 10)
            .await
            .unwrap()
            .first()
            .unwrap()
            .tombstone
    );
    assert!(graph.load_active_edges(OWNER_A).await.unwrap().is_empty());

    seed_other_owner(&search, &timeline, &graph).await;
    assert_owner_rls(&pool).await;
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn memory_consistency_risk_rebuild_replay_expiry_restart_and_rls_are_durable() {
    let url = std::env::var("MAKOSH_PROJECTION_POSTGRES_URL").expect("managed disposable URL");
    let pool = PgPoolOptions::new()
        .max_connections(12)
        .connect(&url)
        .await
        .expect("admin pool");
    pool.execute("CREATE SCHEMA IF NOT EXISTS makosh_data")
        .await
        .unwrap();
    for schema in [MEMORY_SCHEMA_V1, CONSISTENCY_SCHEMA_V1, RISK_SCHEMA_V1] {
        sqlx::raw_sql(std::str::from_utf8(schema).unwrap())
            .execute(&pool)
            .await
            .unwrap();
    }

    let memory = MemoryPersistenceV1::new(pool.clone());
    let consistency = ConsistencyPersistenceV1::new(pool.clone());
    let risk = RiskPersistenceV1::new(pool.clone());
    assert_eq!(
        memory.ensure_live_generation(OWNER_A, 900).await.unwrap(),
        1
    );
    assert_eq!(
        consistency
            .ensure_live_generation(OWNER_A, 900)
            .await
            .unwrap(),
        1
    );
    assert_eq!(risk.ensure_live_generation(OWNER_A, 900).await.unwrap(), 1);

    let memory_live = memory_input(OWNER_A, 51, 1, 1, false);
    let consistency_first = consistency_input(OWNER_A, 52, 1, 1, 74);
    let consistency_second = consistency_input(OWNER_A, 53, 1, 2, 75);
    let risk_live = risk_input(OWNER_A, 54, 1, 1, false);
    assert_eq!(
        memory.apply_entry_once(&memory_live).await.unwrap(),
        MemoryReplayOutcomeV1::Applied
    );
    assert_eq!(
        consistency.apply_once(&consistency_first).await.unwrap(),
        ConsistencyReplayOutcomeV1::Applied
    );
    assert_eq!(
        consistency.apply_once(&consistency_second).await.unwrap(),
        ConsistencyReplayOutcomeV1::Applied
    );
    assert_eq!(
        risk.apply_entry_once(&risk_live).await.unwrap(),
        RiskReplayOutcomeV1::Applied
    );
    assert_eq!(
        memory.apply_entry_once(&memory_live).await.unwrap(),
        MemoryReplayOutcomeV1::Replayed
    );
    assert_eq!(
        consistency.apply_once(&consistency_first).await.unwrap(),
        ConsistencyReplayOutcomeV1::Replayed
    );
    assert_eq!(
        risk.apply_entry_once(&risk_live).await.unwrap(),
        RiskReplayOutcomeV1::Replayed
    );
    assert_eq!(
        contradictions_v1(&consistency.load_active_edges(OWNER_A).await.unwrap())
            .unwrap()
            .len(),
        1
    );

    let mut changed = memory_live.clone();
    changed.input.envelope_bytes.push(99);
    changed.input.envelope_sha256 = Sha256::digest(&changed.input.envelope_bytes).into();
    assert_eq!(
        memory.apply_entry_once(&changed).await,
        Err(MemoryPersistenceErrorV1::Conflict)
    );
    let mut changed = consistency_first.clone();
    changed.input.envelope_bytes.push(99);
    changed.input.envelope_sha256 = Sha256::digest(&changed.input.envelope_bytes).into();
    assert_eq!(
        consistency.apply_once(&changed).await,
        Err(ConsistencyPersistenceErrorV1::Conflict)
    );
    let mut changed = risk_live.clone();
    changed.input.envelope_bytes.push(99);
    changed.input.envelope_sha256 = Sha256::digest(&changed.input.envelope_bytes).into();
    assert_eq!(
        risk.apply_entry_once(&changed).await,
        Err(RiskPersistenceErrorV1::Conflict)
    );

    drop(memory);
    drop(consistency);
    drop(risk);
    let memory = MemoryPersistenceV1::new(pool.clone());
    let consistency = ConsistencyPersistenceV1::new(pool.clone());
    let risk = RiskPersistenceV1::new(pool.clone());
    memory
        .apply_entry_once(&memory_input(OWNER_A, 61, 1, 2, true))
        .await
        .unwrap();
    risk.apply_entry_once(&risk_input(OWNER_A, 62, 1, 2, true))
        .await
        .unwrap();
    assert!(memory.list_active(OWNER_A, None, 10).await.unwrap()[0].tombstone);
    assert!(risk.list_active(OWNER_A, None, 10).await.unwrap()[0].cleared);
    assert_eq!(consistency.status(OWNER_A).await.unwrap().source_events, 2);

    seed_memory_consistency_risk_other_owner(&memory, &consistency, &risk).await;
    assert_memory_consistency_risk_rls(&pool).await;
}

#[tokio::test]
#[ignore = "requires managed disposable PostgreSQL"]
async fn zoom_telemost_omniroute_replay_restart_and_rls_are_durable() {
    let url = std::env::var("MAKOSH_PROJECTION_POSTGRES_URL").expect("managed disposable URL");
    let pool = PgPoolOptions::new()
        .max_connections(12)
        .connect(&url)
        .await
        .expect("admin pool");
    pool.execute("CREATE SCHEMA IF NOT EXISTS makosh_data")
        .await
        .unwrap();
    for schema in [ZOOM_SCHEMA_V1, TELEMOST_SCHEMA_V1, OMNIROUTE_SCHEMA_V1] {
        sqlx::raw_sql(std::str::from_utf8(schema).unwrap())
            .execute(&pool)
            .await
            .unwrap();
    }

    let zoom = ZoomPersistenceV1::new(pool.clone());
    let telemost = TelemostPersistenceV1::new(pool.clone());
    let omniroute = OmniRoutePersistenceV1::new(pool.clone());
    zoom.upsert_account(&ZoomAccountRecordV1 {
        logical_owner_id: OWNER_A.into(),
        account_cursor_sha256: [1; 32],
        mapping_revision: 1,
        lifecycle_state: 1,
        updated_at_unix_millis: 1_000,
    })
    .await
    .unwrap();
    telemost
        .upsert_account(&TelemostAccountRecordV1 {
            logical_owner_id: OWNER_A.into(),
            account_cursor_sha256: [2; 32],
            mapping_revision: 1,
            lifecycle_state: 1,
            updated_at_unix_millis: 1_000,
        })
        .await
        .unwrap();
    let zoom_input = ZoomObservationRecordV1 {
        logical_owner_id: OWNER_A.into(),
        message_id: [3; 16],
        exact_envelope_bytes: vec![3, 4, 5],
        account_cursor_sha256: [1; 32],
        source_revision: 1,
        call_evidence_message_id: [4; 16],
        call_evidence_bytes: vec![6, 7, 8],
        completed_at_unix_millis: 2_000,
    };
    let telemost_input = TelemostObservationRecordV1 {
        logical_owner_id: OWNER_A.into(),
        message_id: [5; 16],
        exact_envelope_bytes: vec![9, 10, 11],
        account_cursor_sha256: [2; 32],
        source_revision: 1,
        call_evidence_message_id: [6; 16],
        call_evidence_bytes: vec![12, 13, 14],
        completed_at_unix_millis: 2_000,
    };
    assert_eq!(
        zoom.record_observation_once(&zoom_input).await.unwrap(),
        ZoomReplayOutcomeV1::Applied
    );
    assert_eq!(
        telemost
            .record_observation_once(&telemost_input)
            .await
            .unwrap(),
        TelemostReplayOutcomeV1::Applied
    );
    assert_eq!(
        zoom.record_observation_once(&zoom_input).await.unwrap(),
        ZoomReplayOutcomeV1::Replayed
    );
    assert_eq!(
        telemost
            .record_observation_once(&telemost_input)
            .await
            .unwrap(),
        TelemostReplayOutcomeV1::Replayed
    );
    let omni = OmniRouteRequestReceiptV1 {
        request_id: [7; 16],
        logical_owner_id: OWNER_A.into(),
        contract_name: "ai_provider_reply_generation".into(),
        request_sha256: [8; 32],
        model: "route/model".into(),
        settings_revision: 1,
        accepted_at_unix_millis: 1_000,
    };
    assert_eq!(
        omniroute.accept_once(&omni, [9; 32]).await.unwrap(),
        OmniRouteReplayOutcomeV1::Accepted
    );
    assert_eq!(
        omniroute.accept_once(&omni, [9; 32]).await.unwrap(),
        OmniRouteReplayOutcomeV1::Replayed
    );
    omniroute
        .complete(OWNER_A, omni.request_id, &[10, 11, 12], 2_000)
        .await
        .unwrap();

    let mut changed = zoom_input.clone();
    changed.exact_envelope_bytes.push(99);
    assert_eq!(
        zoom.record_observation_once(&changed).await,
        Err(ZoomPersistenceErrorV1::Conflict)
    );
    let mut changed = telemost_input.clone();
    changed.exact_envelope_bytes.push(99);
    assert_eq!(
        telemost.record_observation_once(&changed).await,
        Err(TelemostPersistenceErrorV1::Conflict)
    );
    let mut changed = omni.clone();
    changed.request_sha256 = [99; 32];
    assert_eq!(
        omniroute.accept_once(&changed, [9; 32]).await,
        Err(OmniRoutePersistenceErrorV1::Conflict)
    );

    drop((zoom, telemost, omniroute));
    let zoom = ZoomPersistenceV1::new(pool.clone());
    let telemost = TelemostPersistenceV1::new(pool.clone());
    assert_eq!(zoom.counts(OWNER_A).await.unwrap(), (1, 1, 1));
    assert_eq!(telemost.counts(OWNER_A).await.unwrap(), (1, 1, 1));
    assert_task26_rls(&pool).await;
}

async fn assert_task26_rls(pool: &PgPool) {
    pool.execute("CREATE ROLE task26_provider_rls NOLOGIN NOSUPERUSER NOBYPASSRLS")
        .await
        .unwrap();
    pool.execute("GRANT USAGE ON SCHEMA makosh_data TO task26_provider_rls")
        .await
        .unwrap();
    pool.execute("GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA makosh_data TO task26_provider_rls")
        .await
        .unwrap();
    pool.execute(
        "GRANT USAGE,SELECT ON ALL SEQUENCES IN SCHEMA makosh_data TO task26_provider_rls",
    )
    .await
    .unwrap();
    for table in [
        "zoom_accounts",
        "zoom_observation_inbox",
        "zoom_call_evidence_outbox",
        "telemost_accounts",
        "telemost_observation_inbox",
        "telemost_call_evidence_outbox",
        "omniroute_runs",
    ] {
        let mut tx = pool.begin().await.unwrap();
        tx.execute("SET LOCAL ROLE task26_provider_rls")
            .await
            .unwrap();
        sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
            .bind(OWNER_A)
            .execute(&mut *tx)
            .await
            .unwrap();
        let statement =
            format!("SELECT COUNT(*) count FROM makosh_data.{table} WHERE logical_owner_id=$1");
        let count: i64 = sqlx::query(sqlx::AssertSqlSafe(statement))
            .bind(OWNER_B)
            .fetch_one(&mut *tx)
            .await
            .unwrap()
            .try_get("count")
            .unwrap();
        assert_eq!(count, 0, "cross-owner SELECT {table}");
        tx.rollback().await.unwrap();
    }
    let mut tx = pool.begin().await.unwrap();
    tx.execute("SET LOCAL ROLE task26_provider_rls")
        .await
        .unwrap();
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(OWNER_A)
        .execute(&mut *tx)
        .await
        .unwrap();
    let error = sqlx::query("INSERT INTO makosh_data.zoom_accounts(logical_owner_id,account_cursor_sha256,mapping_revision,lifecycle_state,updated_at_unix_millis) VALUES($1,$2,1,1,1)")
        .bind(OWNER_B)
        .bind([77_u8; 32].as_slice())
        .execute(&mut *tx)
        .await
        .expect_err("cross-owner INSERT blocked");
    assert_eq!(
        error
            .as_database_error()
            .and_then(|value| value.code())
            .as_deref(),
        Some("42501")
    );
    tx.rollback().await.unwrap();
}

async fn seed_memory_consistency_risk_other_owner(
    memory: &MemoryPersistenceV1,
    consistency: &ConsistencyPersistenceV1,
    risk: &RiskPersistenceV1,
) {
    memory.ensure_live_generation(OWNER_B, 900).await.unwrap();
    consistency
        .ensure_live_generation(OWNER_B, 900)
        .await
        .unwrap();
    risk.ensure_live_generation(OWNER_B, 900).await.unwrap();
    memory
        .apply_entry_once(&memory_input(OWNER_B, 71, 1, 1, false))
        .await
        .unwrap();
    consistency
        .apply_once(&consistency_input(OWNER_B, 72, 1, 1, 76))
        .await
        .unwrap();
    risk.apply_entry_once(&risk_input(OWNER_B, 73, 1, 1, false))
        .await
        .unwrap();
}

async fn assert_memory_consistency_risk_rls(pool: &PgPool) {
    pool.execute("CREATE ROLE task25_projection_rls NOLOGIN NOSUPERUSER NOBYPASSRLS")
        .await
        .unwrap();
    pool.execute("GRANT USAGE ON SCHEMA makosh_data TO task25_projection_rls")
        .await
        .unwrap();
    pool.execute("GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA makosh_data TO task25_projection_rls").await.unwrap();
    let tables = [
        "memory_projection_control",
        "memory_projection_entries",
        "memory_projection_inbox",
        "memory_projection_rebuilds",
        "consistency_projection_control",
        "consistency_projection_nodes",
        "consistency_projection_edges",
        "consistency_projection_inbox",
        "consistency_projection_rebuilds",
        "risk_projection_control",
        "risk_projection_entries",
        "risk_projection_inbox",
        "risk_projection_rebuilds",
    ];
    for table in tables {
        let mut tx = pool.begin().await.unwrap();
        tx.execute("SET LOCAL ROLE task25_projection_rls")
            .await
            .unwrap();
        sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
            .bind(OWNER_A)
            .execute(&mut *tx)
            .await
            .unwrap();
        let select =
            format!("SELECT COUNT(*) count FROM makosh_data.{table} WHERE logical_owner_id=$1");
        let count: i64 = sqlx::query(sqlx::AssertSqlSafe(select))
            .bind(OWNER_B)
            .fetch_one(&mut *tx)
            .await
            .unwrap()
            .try_get("count")
            .unwrap();
        assert_eq!(count, 0, "cross-owner SELECT {table}");
        let update = format!(
            "UPDATE makosh_data.{table} SET logical_owner_id=logical_owner_id WHERE logical_owner_id=$1"
        );
        assert_eq!(
            sqlx::query(sqlx::AssertSqlSafe(update))
                .bind(OWNER_B)
                .execute(&mut *tx)
                .await
                .unwrap()
                .rows_affected(),
            0,
            "cross-owner UPDATE {table}"
        );
        let delete = format!("DELETE FROM makosh_data.{table} WHERE logical_owner_id=$1");
        assert_eq!(
            sqlx::query(sqlx::AssertSqlSafe(delete))
                .bind(OWNER_B)
                .execute(&mut *tx)
                .await
                .unwrap()
                .rows_affected(),
            0,
            "cross-owner DELETE {table}"
        );
        tx.rollback().await.unwrap();
    }
    let mut tx = pool.begin().await.unwrap();
    tx.execute("SET LOCAL ROLE task25_projection_rls")
        .await
        .unwrap();
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(OWNER_A)
        .execute(&mut *tx)
        .await
        .unwrap();
    let error = sqlx::query("INSERT INTO makosh_data.memory_projection_control(logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES($1,1,2,1)").bind(OWNER_B).execute(&mut *tx).await.expect_err("cross-owner INSERT blocked");
    assert_eq!(
        error
            .as_database_error()
            .and_then(|value| value.code())
            .as_deref(),
        Some("42501")
    );
    tx.rollback().await.unwrap();
}

async fn seed_other_owner(
    search: &SearchPersistenceV1,
    timeline: &TimelinePersistenceV1,
    graph: &GraphPersistenceV1,
) {
    search.ensure_live_generation(OWNER_B, 900).await.unwrap();
    timeline.ensure_live_generation(OWNER_B, 900).await.unwrap();
    graph.ensure_live_generation(OWNER_B, 900).await.unwrap();
    search
        .apply_document_once(&search_input(OWNER_B, 41, 1, 1, false))
        .await
        .unwrap();
    timeline
        .apply_entry_once(&timeline_input(OWNER_B, 42, 1, 1, false))
        .await
        .unwrap();
    graph
        .apply_once(&graph_input(OWNER_B, 43, 1, 1, false))
        .await
        .unwrap();
}

async fn assert_owner_rls(pool: &PgPool) {
    pool.execute("CREATE ROLE projection_rls_runtime NOLOGIN NOSUPERUSER NOBYPASSRLS")
        .await
        .expect("role");
    pool.execute("GRANT USAGE ON SCHEMA makosh_data TO projection_rls_runtime")
        .await
        .unwrap();
    pool.execute("GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA makosh_data TO projection_rls_runtime").await.unwrap();
    let tables = [
        "search_projection_control",
        "search_projection_documents",
        "search_projection_tokens",
        "search_projection_inbox",
        "search_projection_rebuilds",
        "timeline_projection_control",
        "timeline_projection_entries",
        "timeline_projection_inbox",
        "timeline_projection_rebuilds",
        "graph_projection_control",
        "graph_projection_nodes",
        "graph_projection_edges",
        "graph_projection_inbox",
        "graph_projection_rebuilds",
    ];
    for table in tables {
        let mut tx = pool.begin().await.unwrap();
        tx.execute("SET LOCAL ROLE projection_rls_runtime")
            .await
            .unwrap();
        sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
            .bind(OWNER_A)
            .execute(&mut *tx)
            .await
            .unwrap();
        let select =
            format!("SELECT COUNT(*) count FROM makosh_data.{table} WHERE logical_owner_id=$1");
        let count: i64 = sqlx::query(sqlx::AssertSqlSafe(select))
            .bind(OWNER_B)
            .fetch_one(&mut *tx)
            .await
            .unwrap()
            .try_get("count")
            .unwrap();
        assert_eq!(count, 0, "cross-owner SELECT {table}");
        let update = format!(
            "UPDATE makosh_data.{table} SET logical_owner_id=logical_owner_id WHERE logical_owner_id=$1"
        );
        assert_eq!(
            sqlx::query(sqlx::AssertSqlSafe(update))
                .bind(OWNER_B)
                .execute(&mut *tx)
                .await
                .unwrap()
                .rows_affected(),
            0,
            "cross-owner UPDATE {table}"
        );
        let delete = format!("DELETE FROM makosh_data.{table} WHERE logical_owner_id=$1");
        assert_eq!(
            sqlx::query(sqlx::AssertSqlSafe(delete))
                .bind(OWNER_B)
                .execute(&mut *tx)
                .await
                .unwrap()
                .rows_affected(),
            0,
            "cross-owner DELETE {table}"
        );
        tx.rollback().await.unwrap();
    }
    let mut tx = pool.begin().await.unwrap();
    tx.execute("SET LOCAL ROLE projection_rls_runtime")
        .await
        .unwrap();
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(OWNER_A)
        .execute(&mut *tx)
        .await
        .unwrap();
    let error = sqlx::query("INSERT INTO makosh_data.search_projection_control(logical_owner_id,active_projection_generation,next_projection_generation,rebuilt_at_unix_millis) VALUES($1,1,2,1)").bind(OWNER_B).execute(&mut *tx).await.expect_err("cross-owner INSERT blocked");
    assert_eq!(
        error
            .as_database_error()
            .and_then(|value| value.code())
            .as_deref(),
        Some("42501")
    );
    tx.rollback().await.unwrap();
}
