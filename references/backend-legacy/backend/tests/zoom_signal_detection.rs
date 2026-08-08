use std::sync::Arc;

use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_events_api::EventLogQuery;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::domains::signal_hub::profiles::SignalHubProfileService;
use makosh_hub_backend::domains::signal_hub::store::SignalHubStore;
use makosh_hub_backend::integrations::zoom::client::{
    models::{ZoomAccountSetupRequest, ZoomMeetingObservationRequest},
    store::ZoomStore,
};
use makosh_hub_backend::platform::calls::store::CallIntelligenceStore;
use makosh_hub_backend::platform::events::bus::InMemoryEventBus;
use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::zoom_signal_detection::project_zoom_signal_detection;
use serde_json::json;

#[tokio::test]
async fn zoom_meeting_events_flow_into_signal_hub_detection_events() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    SignalHubStore::new(pool.clone())
        .restore_system_sources()
        .await
        .expect("restore signal hub sources");

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
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let account_id = format!("zoom-signal-detection-{suffix}");
    let correlation_id = format!("zoom-signal-detection-correlation-{suffix}");
    zoom_store
        .setup_fixture_account(&ZoomAccountSetupRequest {
            account_id: account_id.clone(),
            display_name: "Zoom Signal Detection Fixture".to_owned(),
            external_account_id: format!("zoom-signal-detection-external-{suffix}"),
            account_email: None,
            metadata: json!({}),
        })
        .await
        .expect("fixture account");

    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-signal-detection-observation-{suffix}")),
            account_id: account_id.clone(),
            meeting_id: format!("meeting-{suffix}"),
            meeting_uuid: Some(format!("meeting-uuid-{suffix}")),
            topic: Some("Signal detection meeting".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(format!("https://example.zoom.us/j/{suffix}")),
            started_at: Some(Utc::now()),
            ended_at: None,
            duration_seconds: None,
            participants: vec![],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({
                "source": "zoom_signal_detection_test",
            }),
            causation_id: None,
            correlation_id: Some(correlation_id.clone()),
        })
        .await
        .expect("observe meeting");

    let zoom_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id.clone()),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom meeting events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(observed.call_id))
        .expect("stored zoom event");

    project_zoom_signal_detection(&pool, &zoom_event.event)
        .await
        .expect("project zoom signal detection");

    let signal_store = EventStore::new(pool.clone());
    let raw_signal = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.raw.zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id.clone()),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom raw signal events")
        .into_iter()
        .next()
        .expect("raw signal");
    assert_eq!(raw_signal.event.subject["source_code"], json!("zoom"));
    assert_eq!(raw_signal.event.subject["account_id"], json!(account_id));
    assert_eq!(
        raw_signal.event.subject["entity_id"],
        json!(observed.call_id)
    );
    assert_eq!(
        raw_signal.event.subject["meeting_id"],
        json!(format!("meeting-{suffix}"))
    );
    assert_eq!(
        raw_signal.event.payload["meeting_id"],
        json!(format!("meeting-{suffix}"))
    );
    assert_eq!(
        raw_signal.event.provenance["source"],
        json!("zoom_signal_detection")
    );

    let accepted_signal = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.accepted.zoom.meeting".to_owned()),
            correlation_id: Some(correlation_id),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom accepted signal events")
        .into_iter()
        .next()
        .expect("accepted signal");
    assert_eq!(
        accepted_signal.event.subject["entity_id"],
        json!(observed.call_id)
    );
    assert_eq!(
        accepted_signal.event.provenance["signal_hub"]["decision"],
        json!("accepted")
    );
    assert_eq!(
        accepted_signal.event.provenance["raw_event_id"],
        json!(raw_signal.event.event_id)
    );
}

#[tokio::test]
async fn zoom_meeting_signal_detection_respects_testing_profile_muting() {
    let context = TestContext::new().await;
    let database = Database::connect(Some(&context.connection_string()))
        .await
        .expect("database connection");
    let pool = database.pool().expect("pool").clone();
    let signal_store = SignalHubStore::new(pool.clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore signal hub sources");
    SignalHubProfileService::new(
        signal_store.clone(),
        ApplicationSettingsStore::new(pool.clone()),
        EventStore::new(pool.clone()),
    )
    .apply_profile("testing")
    .await
    .expect("apply testing profile");

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
    let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or_default());
    let account_id = format!("zoom-signal-muted-{suffix}");
    let correlation_id = format!("zoom-signal-muted-correlation-{suffix}");
    zoom_store
        .setup_fixture_account(&ZoomAccountSetupRequest {
            account_id: account_id.clone(),
            display_name: "Zoom Signal Muted Fixture".to_owned(),
            external_account_id: format!("zoom-signal-muted-external-{suffix}"),
            account_email: None,
            metadata: json!({}),
        })
        .await
        .expect("fixture account");

    let observed = zoom_store
        .observe_meeting(&ZoomMeetingObservationRequest {
            observation_id: Some(format!("zoom-signal-muted-observation-{suffix}")),
            account_id,
            meeting_id: format!("muted-meeting-{suffix}"),
            meeting_uuid: Some(format!("muted-meeting-uuid-{suffix}")),
            topic: Some("Muted meeting".to_owned()),
            host_email: Some("owner@example.test".to_owned()),
            join_url: Some(format!("https://example.zoom.us/j/muted-{suffix}")),
            started_at: Some(Utc::now()),
            ended_at: None,
            duration_seconds: None,
            participants: vec![],
            recording_refs: vec![],
            transcript_ref: None,
            metadata: json!({}),
            causation_id: None,
            correlation_id: Some(correlation_id.clone()),
        })
        .await
        .expect("observe meeting");

    let zoom_event = EventStore::new(pool.clone())
        .list_matching(EventLogQuery {
            event_type: Some("zoom.meeting.observed".to_owned()),
            correlation_id: Some(correlation_id.clone()),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom meeting events")
        .into_iter()
        .find(|event| event.event.subject["call_id"] == json!(observed.call_id))
        .expect("stored zoom event");

    project_zoom_signal_detection(&pool, &zoom_event.event)
        .await
        .expect("project zoom signal detection");

    let signal_store = EventStore::new(pool);
    let muted_signal = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.muted.zoom.meeting".to_owned()),
            correlation_id: Some(correlation_id),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("zoom muted signal events")
        .into_iter()
        .next()
        .expect("muted signal");
    assert_eq!(
        muted_signal.event.subject["entity_id"],
        json!(observed.call_id)
    );
    assert_eq!(
        muted_signal.event.provenance["signal_hub"]["decision"],
        json!("muted")
    );
    assert_eq!(
        muted_signal.event.provenance["signal_hub"]["reason"],
        json!("testing profile mutes Zoom signals")
    );

    let accepted = signal_store
        .list_matching(EventLogQuery {
            event_type: Some("signal.accepted.zoom.meeting".to_owned()),
            correlation_id: Some(format!("zoom-signal-muted-correlation-{suffix}")),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("accepted signal query");
    assert!(accepted.is_empty());
}
