use makosh_backend_testkit::context::TestContext;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_observations_api::models::{
    NewObservation, NewObservationIngestionRun, NewObservationLink, ObservationIngestionRunStatus,
    ObservationOriginKind,
};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

#[tokio::test]
async fn manual_capture_creates_observation_without_vault_source_against_postgres() {
    let Some((_test_context, pool, store)) =
        live_observation_context("manual capture without vault").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observed_at = Utc.with_ymd_and_hms(2026, 6, 18, 8, 30, 0).unwrap();

    let stored = store
        .capture(
            &NewObservation::new(
                "VOICE_RECORDING",
                ObservationOriginKind::Manual,
                observed_at,
                json!({
                    "transcript": "Record the Макошь evidence architecture decision.",
                    "duration_seconds": 42
                }),
                format!("manual://voice-memo/{suffix}"),
            )
            .confidence(0.96)
            .provenance(json!({
                "captured_by": "manual_voice_memo",
                "device": "local_desktop"
            })),
        )
        .await
        .expect("capture manual observation");

    assert_eq!(stored.kind_code, "VOICE_RECORDING");
    assert_eq!(stored.origin_kind, ObservationOriginKind::Manual);
    assert_eq!(stored.vault_source_id, None);
    assert!(stored.content_hash.starts_with("sha256:"));

    let row = sqlx::query(
        r#"
        SELECT payload, provenance
        FROM observations
        WHERE observation_id = $1
          AND vault_source_id IS NULL
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("stored observation row");
    let payload: Value = row.try_get("payload").expect("payload");
    let provenance: Value = row.try_get("provenance").expect("provenance");

    assert_eq!(
        payload["transcript"],
        json!("Record the Макошь evidence architecture decision.")
    );
    assert_eq!(provenance["captured_by"], json!("manual_voice_memo"));

    let event_row = sqlx::query(
        r#"
        SELECT event_id, correlation_id, causation_id, subject
        FROM event_log
        WHERE event_type = 'observation.captured.v1'
          AND subject ->> 'observation_id' = $1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("observation captured event row");
    let event_id: String = event_row.try_get("event_id").expect("event_id");
    let correlation_id: Option<String> =
        event_row.try_get("correlation_id").expect("correlation_id");
    let causation_id: Option<String> = event_row.try_get("causation_id").expect("causation_id");
    let subject: Value = event_row.try_get("subject").expect("subject");

    assert_eq!(
        event_id,
        format!("event:v1:observation-captured:{}", stored.observation_id)
    );
    assert_eq!(
        correlation_id.as_deref(),
        Some(stored.observation_id.as_str())
    );
    assert_eq!(causation_id, None);
    assert_eq!(subject["kind"], json!("observation"));
    assert_eq!(subject["entity_id"], json!(stored.observation_id));
    assert_eq!(subject["observation_id"], json!(stored.observation_id));
    assert_eq!(subject["observation_kind"], json!("VOICE_RECORDING"));
}

#[tokio::test]
async fn manual_note_creates_observation_without_vault_source_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("manual note without vault").await
    else {
        return;
    };
    let suffix = unique_suffix();

    let stored = store
        .capture(
            &NewObservation::new(
                "DOCUMENT",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 8, 45, 0).unwrap(),
                json!({
                    "title": format!("Manual note {suffix}"),
                    "body": "Create note should also land in canonical evidence."
                }),
                format!("manual://note/{suffix}"),
            )
            .confidence(0.91)
            .provenance(json!({
                "captured_by": "manual_note"
            })),
        )
        .await
        .expect("capture manual note observation");

    assert_eq!(stored.kind_code, "DOCUMENT");
    assert_eq!(stored.origin_kind, ObservationOriginKind::Manual);
    assert_eq!(stored.vault_source_id, None);
}

#[tokio::test]
async fn observations_are_append_only_and_survive_provider_deletion_against_postgres() {
    let Some((_test_context, pool, store)) = live_observation_context("append-only deletion").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source_ref = format!("gmail://account/{suffix}/message/provider-{suffix}");
    let observed_at = Utc.with_ymd_and_hms(2026, 6, 18, 9, 0, 0).unwrap();

    let imported = store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::VaultSource,
                observed_at,
                json!({
                    "provider": "gmail",
                    "message_id": format!("provider-{suffix}"),
                    "subject": "Evidence store"
                }),
                source_ref.clone(),
            )
            .vault_source_id(format!("vault_source:gmail:{suffix}"))
            .confidence(0.99),
        )
        .await
        .expect("capture provider message");

    let update_error =
        sqlx::query("UPDATE observations SET payload = $1 WHERE observation_id = $2")
            .bind(json!({"mutated": true}))
            .bind(&imported.observation_id)
            .execute(&pool)
            .await
            .expect_err("observation update must be blocked");
    assert!(
        update_error.to_string().contains("append-only"),
        "unexpected update error: {update_error}"
    );

    let delete_error = sqlx::query("DELETE FROM observations WHERE observation_id = $1")
        .bind(&imported.observation_id)
        .execute(&pool)
        .await
        .expect_err("observation delete must be blocked");
    assert!(
        delete_error.to_string().contains("append-only"),
        "unexpected delete error: {delete_error}"
    );

    let deletion = store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE_DELETED",
                ObservationOriginKind::VaultSource,
                observed_at,
                json!({
                    "provider": "gmail",
                    "message_id": format!("provider-{suffix}"),
                    "deletion_observed": true
                }),
                source_ref.clone(),
            )
            .vault_source_id(format!("vault_source:gmail:{suffix}"))
            .confidence(0.93),
        )
        .await
        .expect("capture provider deletion observation");

    assert_ne!(imported.observation_id, deletion.observation_id);

    let rows = sqlx::query(
        r#"
        SELECT kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.source_ref = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&source_ref)
    .fetch_all(&pool)
    .await
    .expect("observations for source ref");
    let codes: Vec<String> = rows
        .into_iter()
        .map(|row| row.try_get("code").expect("kind code"))
        .collect();

    assert_eq!(
        codes,
        vec!["COMMUNICATION_MESSAGE", "COMMUNICATION_MESSAGE_DELETED"]
    );
}

#[tokio::test]
async fn observation_platform_persists_links_and_ingestion_runs_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("observation links and runs").await
    else {
        return;
    };
    let suffix = unique_suffix();

    let observation = store
        .capture(
            &NewObservation::new(
                "MEETING_TRANSCRIPT",
                ObservationOriginKind::FileImport,
                Utc.with_ymd_and_hms(2026, 6, 18, 9, 30, 0).unwrap(),
                json!({
                    "meeting_id": format!("meeting:v1:{suffix}"),
                    "transcript": "Action: prepare NAS purchase context."
                }),
                format!("import://meeting-transcript/{suffix}"),
            )
            .confidence(0.92),
        )
        .await
        .expect("capture imported transcript observation");

    let link = store
        .upsert_link(
            &NewObservationLink::new(
                observation.observation_id.clone(),
                "meetings",
                "meeting",
                format!("meeting:v1:{suffix}"),
            )
            .relationship_kind("evidence_for")
            .confidence(0.88)
            .metadata(json!({
                "linked_by": "ingestion_test"
            })),
        )
        .await
        .expect("upsert observation link");
    assert_eq!(link.domain, "meetings");

    let links = store
        .list_links(&observation.observation_id)
        .await
        .expect("list observation links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].entity_kind, "meeting");

    let started = store
        .start_ingestion_run(&NewObservationIngestionRun::new(
            format!("ingestion-run:v1:{suffix}"),
            observation.observation_id.clone(),
            "meetings/transcript-ingestion",
        ))
        .await
        .expect("start ingestion run");
    assert_eq!(started.status, ObservationIngestionRunStatus::Running);

    let finished = store
        .finish_ingestion_run(
            &started.ingestion_run_id,
            ObservationIngestionRunStatus::Succeeded,
            &json!({
                "produced": ["meeting", "task_candidate", "knowledge_candidate"]
            }),
            None,
        )
        .await
        .expect("finish ingestion run");
    assert_eq!(finished.status, ObservationIngestionRunStatus::Succeeded);
    assert!(finished.finished_at.is_some());

    let runs = store
        .list_ingestion_runs(&observation.observation_id)
        .await
        .expect("list ingestion runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].pipeline, "meetings/transcript-ingestion");
    assert_eq!(runs[0].output["produced"][0], json!("meeting"));
}

#[tokio::test]
async fn canonical_observation_kind_definitions_are_seeded_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("canonical kind definitions").await
    else {
        return;
    };

    let definitions = store
        .list_kind_definitions()
        .await
        .expect("list observation kind definitions");
    let codes: HashSet<&str> = definitions
        .iter()
        .map(|definition| definition.code.as_str())
        .collect();

    for required in [
        "COMMUNICATION_MESSAGE",
        "COMMUNICATION_DRAFT",
        "COMMUNICATION_FOLDER",
        "COMMUNICATION_SAVED_SEARCH",
        "COMMUNICATION_OUTBOX",
        "COMMUNICATION_DELIVERY_STATUS",
        "COMMUNICATION_READ_RECEIPT",
        "CONTRADICTION_OBSERVATION",
        "COMMUNICATION_MESSAGE_DELETED",
        "COMMUNICATION_ATTACHMENT",
        "MEETING",
        "MEETING_RECORDING",
        "MEETING_TRANSCRIPT",
        "DOCUMENT",
        "VOICE_RECORDING",
        "BROWSER_CAPTURE",
        "PERSONA_RECORD",
        "CALENDAR_EVENT",
        "CALENDAR_EVENT_DELETED",
    ] {
        assert!(
            codes.contains(required),
            "missing kind definition {required}"
        );
    }
}

#[tokio::test]
async fn browser_capture_creates_observation_without_vault_source_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("browser capture without vault").await
    else {
        return;
    };
    let suffix = unique_suffix();

    let stored = store
        .capture(
            &NewObservation::new(
                "BROWSER_CAPTURE",
                ObservationOriginKind::BrowserCapture,
                Utc.with_ymd_and_hms(2026, 6, 18, 10, 15, 0).unwrap(),
                json!({
                    "url": "https://example.com/makosh",
                    "selection": "Canonical evidence store",
                    "window_title": format!("Макошь Browser {suffix}")
                }),
                format!("browser://tab/{suffix}"),
            )
            .confidence(0.89)
            .provenance(json!({
                "captured_by": "browser_extension",
                "session": format!("browser-session-{suffix}")
            })),
        )
        .await
        .expect("capture browser observation");

    assert_eq!(stored.kind_code, "BROWSER_CAPTURE");
    assert_eq!(stored.origin_kind, ObservationOriginKind::BrowserCapture);
    assert!(stored.vault_source_id.is_none());
}

#[tokio::test]
async fn meeting_transcript_creates_observation_without_vault_source_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("meeting transcript without vault").await
    else {
        return;
    };
    let suffix = unique_suffix();

    let stored = store
        .capture(
            &NewObservation::new(
                "MEETING_TRANSCRIPT",
                ObservationOriginKind::FileImport,
                Utc.with_ymd_and_hms(2026, 6, 18, 10, 30, 0).unwrap(),
                json!({
                    "meeting_id": format!("meeting:v1:{suffix}"),
                    "transcript": "Action items: add NAS budget and owners."
                }),
                format!("meeting://transcript/{suffix}"),
            )
            .confidence(0.87)
            .provenance(json!({
                "captured_by": "meeting_transcript_import"
            })),
        )
        .await
        .expect("capture meeting transcript observation");

    assert_eq!(stored.kind_code, "MEETING_TRANSCRIPT");
    assert_eq!(stored.origin_kind, ObservationOriginKind::FileImport);
    assert!(stored.vault_source_id.is_none());
}

#[tokio::test]
async fn meeting_recording_creates_observation_without_vault_source_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("meeting recording without vault").await
    else {
        return;
    };
    let suffix = unique_suffix();

    let stored = store
        .capture(
            &NewObservation::new(
                "MEETING_RECORDING",
                ObservationOriginKind::FileImport,
                Utc.with_ymd_and_hms(2026, 6, 18, 10, 31, 0).unwrap(),
                json!({
                    "meeting_id": format!("meeting:v1:{suffix}"),
                    "recording_uri": format!("file:///tmp/meetings/{suffix}.m4a"),
                    "duration_seconds": 1800
                }),
                format!("meeting://recording/{suffix}"),
            )
            .confidence(0.85)
            .provenance(json!({
                "captured_by": "meeting_recording_import"
            })),
        )
        .await
        .expect("capture meeting recording observation");

    assert_eq!(stored.kind_code, "MEETING_RECORDING");
    assert_eq!(stored.origin_kind, ObservationOriginKind::FileImport);
    assert!(stored.vault_source_id.is_none());
}

#[tokio::test]
async fn calendar_event_creates_observation_without_vault_source_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("calendar event without vault").await
    else {
        return;
    };
    let suffix = unique_suffix();

    let stored = store
        .capture(
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::FileImport,
                Utc.with_ymd_and_hms(2026, 6, 18, 10, 32, 0).unwrap(),
                json!({
                    "calendar_event_id": format!("event:v1:{suffix}"),
                    "title": "Weekly planning",
                    "start_at": "2026-06-19T09:00:00Z",
                    "end_at": "2026-06-19T10:00:00Z"
                }),
                format!("calendar://events/{suffix}"),
            )
            .confidence(0.86)
            .provenance(json!({
                "captured_by": "calendar_import"
            })),
        )
        .await
        .expect("capture calendar event observation");

    assert_eq!(stored.kind_code, "CALENDAR_EVENT");
    assert_eq!(stored.origin_kind, ObservationOriginKind::FileImport);
    assert!(stored.vault_source_id.is_none());
}

#[tokio::test]
async fn persona_record_creates_observation_without_vault_source_against_postgres() {
    let Some((_test_context, _pool, store)) =
        live_observation_context("persona record without vault").await
    else {
        return;
    };
    let suffix = unique_suffix();

    let stored = store
        .capture(
            &NewObservation::new(
                "PERSONA_RECORD",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 10, 33, 0).unwrap(),
                json!({
                    "display_name": format!("Ada Example {suffix}"),
                    "channel": "manual_address_book_rollup",
                    "identity_tag": format!("persona:{suffix}")
                }),
                format!("persona://manual/{suffix}"),
            )
            .confidence(0.74)
            .provenance(json!({
                "captured_by": "address_book_import"
            })),
        )
        .await
        .expect("capture persona record observation");

    assert_eq!(stored.kind_code, "PERSONA_RECORD");
    assert_eq!(stored.origin_kind, ObservationOriginKind::Manual);
    assert!(stored.vault_source_id.is_none());
}

async fn live_observation_context(
    _test_name: &str,
) -> Option<(TestContext, PgPool, ObservationStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((test_context, pool.clone(), ObservationStore::new(pool)))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_nanos()
}
