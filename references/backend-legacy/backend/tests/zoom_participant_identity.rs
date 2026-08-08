use std::sync::Arc;

use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_events_api::EventLogQuery;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::integrations::zoom::client::{
    models::{ZoomAccountSetupRequest, ZoomMeetingObservationRequest, ZoomParticipantSnapshot},
    store::ZoomStore,
};
use makosh_hub_backend::platform::calls::store::CallIntelligenceStore;
use makosh_hub_backend::platform::events::bus::InMemoryEventBus;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::review_inbox::project_persona_identity_review_event;
use makosh_hub_backend::workflows::zoom_participant_identity::project_zoom_participant_identity;
use serde_json::json;
use sqlx::Row;

#[tokio::test]
async fn zoom_participant_identity_candidates_flow_into_review_inbox() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());

    let person_store = PersonaProjectionStore::new(pool.clone());
    let matched_person = person_store
        .upsert_email_persona(&format!("existing-zoom-person-{suffix}@example.com"))
        .await
        .expect("upsert existing person");
    let display_name = format!("Zoom Person {suffix}");
    sqlx::query("UPDATE personas SET display_name = $1 WHERE persona_id = $2")
        .bind(&display_name)
        .bind(&matched_person.persona_id)
        .execute(&pool)
        .await
        .expect("seed display name");

    let event_bus = InMemoryEventBus::new();
    let zoom_store = ZoomStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(
            makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore::new(
                pool.clone(),
            ),
        ),
        CallIntelligenceStore::new(pool.clone()),
        EventStore::new(pool.clone()),
        event_bus,
    );
    let account_id = format!("zoom-participant-identity-{suffix}");
    zoom_store
        .setup_fixture_account(&ZoomAccountSetupRequest {
            account_id: account_id.clone(),
            display_name: "Zoom Participant Identity Fixture".to_owned(),
            external_account_id: format!("zoom-participant-identity-external-{suffix}"),
            account_email: None,
            metadata: json!({}),
        })
        .await
        .expect("fixture account");

    let participant_email = format!("zoom.person.{suffix}@example.com").to_ascii_lowercase();
    let correlation_id = format!("zoom-participant-identity-correlation-{suffix}");
    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-participant-identity-observation-{suffix}")),
            account_id: account_id.clone(),
            meeting_id: format!("participant-identity-{suffix}"),
            meeting_uuid: Some(format!("participant-identity-uuid-{suffix}")),
            topic: Some("Identity candidate meeting".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(format!("https://example.zoom.us/j/{suffix}")),
            started_at: Some(Utc::now()),
            ended_at: None,
            duration_seconds: None,
            participants: vec![ZoomParticipantSnapshot {
                participant_id: Some(format!("participant-{suffix}")),
                display_name: Some(display_name.clone()),
                email: Some(participant_email.clone()),
                joined_at: None,
                left_at: None,
                metadata: json!({ "source": "zoom_participant_identity_test" }),
            }],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({}),
            causation_id: None,
            correlation_id: Some(correlation_id.clone()),
        })
        .await
        .expect("observe zoom meeting");

    let stored_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("stored zoom events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(observed.call_id))
        .expect("matched stored zoom event");

    project_zoom_participant_identity(&pool, &stored_event.event)
        .await
        .expect("zoom participant identity projection");

    let expected_candidate_id = format!(
        "identity_candidate:v1:attach_email_address:{}:{}:{}",
        matched_person.persona_id,
        participant_email.len(),
        participant_email
    );
    let candidate = sqlx::query(
        r#"
        SELECT
            identity_candidate_id,
            candidate_kind,
            left_persona_id,
            right_persona_id,
            email_address,
            review_state,
            evidence_summary
        FROM persona_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&expected_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("zoom participant identity candidate");
    assert_eq!(
        candidate
            .try_get::<String, _>("candidate_kind")
            .expect("candidate kind"),
        "attach_email_address"
    );
    assert_eq!(
        candidate
            .try_get::<String, _>("left_persona_id")
            .expect("left person id"),
        matched_person.persona_id
    );
    assert_eq!(
        candidate
            .try_get::<Option<String>, _>("right_persona_id")
            .expect("right person id"),
        None
    );
    assert_eq!(
        candidate
            .try_get::<Option<String>, _>("email_address")
            .expect("email address"),
        Some(participant_email.clone())
    );
    assert_eq!(
        candidate
            .try_get::<String, _>("review_state")
            .expect("review state"),
        "suggested"
    );
    assert!(
        candidate
            .try_get::<String, _>("evidence_summary")
            .expect("evidence summary")
            .contains(&participant_email)
    );

    let candidate_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("persona_identity.candidate.detected".to_owned()),
            limit: Some(20),
            ..Default::default()
        })
        .await
        .expect("persona identity candidate events")
        .into_iter()
        .find(|event| event.event.payload["identity_candidate_id"] == json!(expected_candidate_id))
        .expect("candidate detected event");

    project_persona_identity_review_event(pool.clone(), candidate_event)
        .await
        .expect("person identity review inbox projection");

    let review_item = sqlx::query(
        r#"
        SELECT review_item_id, item_kind, metadata
        FROM review_items
        WHERE item_kind = 'identity_candidate'
          AND metadata->>'identity_candidate_id' = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&expected_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("review item");
    assert_eq!(
        review_item
            .try_get::<String, _>("item_kind")
            .expect("item kind"),
        "identity_candidate"
    );
    let metadata: serde_json::Value = review_item.try_get("metadata").expect("metadata");
    assert_eq!(metadata["candidate_kind"], json!("attach_email_address"));
    assert_eq!(
        metadata["left_persona_id"],
        json!(matched_person.persona_id)
    );
    assert_eq!(metadata["email_address"], json!(participant_email));
}
