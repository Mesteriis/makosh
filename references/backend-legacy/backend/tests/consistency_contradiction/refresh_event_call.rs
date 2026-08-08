use chrono::{Duration, Utc};
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::calendar::events::{
    event_store::CalendarEventStore, models::NewCalendarEvent,
};
use makosh_hub_backend::domains::calendar::meetings::notes::MeetingNoteStore;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::engines::consistency::{
    models::{ContradictionSeverity, ContradictionSourceKind},
    store::ContradictionObservationStore,
};
use makosh_hub_backend::platform::calls::{
    models::{CallDirection, CallState, NewCallTranscript, NewTelegramCall, TranscriptStatus},
    store::CallIntelligenceStore,
};

use serde_json::json;

use super::support::{live_consistency_pool, unique_suffix};

#[tokio::test]
async fn contradiction_refresh_detects_meeting_note_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction meeting note refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-meeting-{suffix}@example.com");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&email_address)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO persona_facts (persona_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let start_at = Utc::now();
    let event = CalendarEventStore::new(pool.clone())
        .create(&NewCalendarEvent {
            title: format!("Polygraph meeting {suffix}"),
            description: Some("Meeting evidence for consistency refresh".to_owned()),
            start_at,
            end_at: start_at + Duration::minutes(30),
            event_type: Some("meeting".to_owned()),
            ..NewCalendarEvent::default()
        })
        .await
        .expect("calendar event");
    sqlx::query(
        r#"
        INSERT INTO event_participants (event_id, email, display_name, role, persona_id)
        VALUES ($1, $2, 'Polygraph Participant', 'attendee', $3)
        "#,
    )
    .bind(&event.event_id)
    .bind(&email_address)
    .bind(&person.persona_id)
    .execute(&pool)
    .await
    .expect("event participant");
    let note = MeetingNoteStore::new(pool.clone())
        .create(
            &event.event_id,
            "Location: Madrid",
            Some("markdown"),
            Some("manual"),
        )
        .await
        .expect("meeting note");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == note.id)
        .expect("meeting note claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.new_source_kind, ContradictionSourceKind::Event);
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.persona_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "event"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM persona_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_call_transcript_claim_against_active_persona_fact_without_overwriting_memory()
 {
    let Some(pool) = live_consistency_pool("contradiction call transcript refresh").await else {
        return;
    };
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-call-{suffix}@example.com");
    let provider_chat_id = format!("telegram-chat-{suffix}");
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO persona_identities (persona_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'telegram', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.persona_id)
    .bind(&provider_chat_id)
    .execute(&pool)
    .await
    .expect("telegram identity");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO persona_facts (persona_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.persona_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let account_id = format!("acct_polygraph_call_{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Polygraph Call Compatibility Account",
            format!("polygraph-call-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");
    let call_store = CallIntelligenceStore::new(pool.clone());
    let call_id = format!("call:polygraph:{suffix}");
    call_store
        .upsert_call(&NewTelegramCall {
            call_id: call_id.clone(),
            account_id: account_id.clone(),
            provider_call_id: format!("provider-call-{suffix}"),
            provider_chat_id: provider_chat_id.clone(),
            direction: CallDirection::Incoming,
            call_state: CallState::Ended,
            started_at: Some(Utc::now()),
            ended_at: Some(Utc::now()),
            transcription_policy_id: None,
            metadata: json!({"source": "polygraph_test"}),
        })
        .await
        .expect("telegram call");
    let transcript_id = format!("call-transcript-polygraph-{suffix}");
    call_store
        .upsert_transcript(&NewCallTranscript {
            transcript_id: transcript_id.clone(),
            call_id,
            account_id,
            provider_chat_id,
            transcript_status: TranscriptStatus::Succeeded,
            stt_provider: "fixture-stt".to_owned(),
            source_audio_ref: Some(format!("audio-polygraph-{suffix}")),
            language_code: Some("en".to_owned()),
            transcript_text: "Location: Madrid".to_owned(),
            segments: json!([]),
            provenance: json!({"source": "polygraph_test"}),
        })
        .await
        .expect("call transcript");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == transcript_id)
        .expect("call transcript claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.persona_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM persona_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}
