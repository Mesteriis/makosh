use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use makosh_hub_backend::engines::context_packs::{
    errors::ContextPackStoreError,
    models::{ContextPackKind, ContextPackSourceKind, NewContextPack, NewContextPackSource},
    store::ContextPackStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::json;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[tokio::test]
async fn context_pack_store_persists_derived_pack_with_explicit_sources_against_postgres() {
    let Some((_pool, observation_store, context_pack_store)) =
        live_context_pack_context("context pack sources").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "MEETING_TRANSCRIPT",
                ObservationOriginKind::VaultSource,
                Utc.with_ymd_and_hms(2026, 6, 18, 12, 0, 0).unwrap(),
                json!({
                    "meeting_id": format!("meeting:v1:{suffix}"),
                    "transcript": "Decision: prepare the NAS purchase context pack."
                }),
                format!("zoom://meeting/{suffix}/transcript"),
            )
            .vault_source_id(format!("vault_source:zoom:{suffix}"))
            .confidence(0.94),
        )
        .await
        .expect("capture meeting transcript");

    let stored = context_pack_store
        .upsert_with_sources(
            &NewContextPack::new(
                ContextPackKind::Meeting,
                format!("meeting:v1:{suffix}"),
                json!({
                    "summary": "Meeting context for NAS purchase decision.",
                    "open_items": ["prepare storage requirements"]
                }),
            )
            .metadata(json!({"builder": "contract-test"})),
            &[
                NewContextPackSource::new(
                    ContextPackSourceKind::Observation,
                    observation.observation_id.clone(),
                )
                .role("primary_evidence"),
                NewContextPackSource::new(
                    ContextPackSourceKind::DomainEntity,
                    format!("meeting:v1:{suffix}"),
                )
                .role("meeting"),
                NewContextPackSource::new(
                    ContextPackSourceKind::Knowledge,
                    format!("knowledge:v1:{suffix}"),
                )
                .role("background"),
            ],
        )
        .await
        .expect("upsert context pack");

    assert_eq!(stored.kind, ContextPackKind::Meeting);
    assert_eq!(stored.subject_id, format!("meeting:v1:{suffix}"));
    assert!(stored.rebuildable);
    assert_eq!(
        stored.content["summary"],
        json!("Meeting context for NAS purchase decision.")
    );

    let sources = context_pack_store
        .list_sources(&stored.context_pack_id)
        .await
        .expect("list context pack sources");
    assert_eq!(sources.len(), 3);
    assert!(sources.iter().any(|source| {
        source.source_kind == ContextPackSourceKind::Observation
            && source.source_id == observation.observation_id
    }));
}

#[tokio::test]
async fn context_pack_store_rejects_pack_without_sources_before_database_write() {
    let store = disconnected_context_pack_store();
    let error = store
        .upsert_with_sources(
            &NewContextPack::new(
                ContextPackKind::Persona,
                "persona:missing-sources",
                json!({"summary": "source-less context is not acceptable"}),
            ),
            &[],
        )
        .await
        .expect_err("context pack without sources must fail before database write");

    assert!(matches!(error, ContextPackStoreError::MissingSources));
}

async fn live_context_pack_context(
    _test_name: &str,
) -> Option<(PgPool, ObservationStore, ContextPackStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((
        pool.clone(),
        ObservationStore::new(pool.clone()),
        ContextPackStore::new(pool),
    ))
}

fn disconnected_context_pack_store() -> ContextPackStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    ContextPackStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_nanos()
}
