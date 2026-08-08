use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{Duration, Utc};
use makosh_backend_testkit::context::TestContext;
use serde_json::json;

use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::EventLogQuery;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::consumers::EventConsumerConfig;
use makosh_events_postgres::consumers::EventConsumerRunner;
use makosh_events_postgres::consumers::EventConsumerStore;
use makosh_events_postgres::consumers::EventDeadLetterReviewState;
use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::application::signal_hub_replay::SignalHubReplayService;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, consume_accepted_signal_event,
    project_accepted_signal_if_runtime_allows,
};
use makosh_hub_backend::domains::personas::core::roles::PERSONA_ROLE_ASSIGNED_EVENT_TYPE;
use makosh_hub_backend::domains::signal_hub::connections::SignalHubConnectionService;
use makosh_hub_backend::domains::signal_hub::controls::{
    SignalHubControlRequest, SignalHubControlService,
};
use makosh_hub_backend::domains::signal_hub::fixture_source::{
    SignalFixtureEmitRequest, SignalFixtureSourceService,
};
use makosh_hub_backend::domains::signal_hub::health::SignalHubHealthService;
use makosh_hub_backend::domains::signal_hub::policies::{
    SignalPolicyDecision, SignalPolicyEvaluator,
};
use makosh_hub_backend::domains::signal_hub::profiles::SignalHubProfileService;
use makosh_hub_backend::domains::signal_hub::replay_contracts::SignalReplayRequestCreate;
use makosh_hub_backend::domains::signal_hub::service::{
    SIGNAL_HUB_RAW_SIGNAL_CONSUMER, SignalHubSignalService, SignalProcessingOutcome,
    process_signal_hub_raw_event,
};
use makosh_hub_backend::domains::signal_hub::store::{
    SignalConnectionCreate, SignalHealthCheckRequest, SignalHubStore, SignalRuntimeStateUpdate,
};
use makosh_hub_backend::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use makosh_signal_hub_api::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use makosh_signal_hub_api::runtime_lifecycle::{RuntimeLifecyclePort, RuntimeLifecycleUpdate};
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;
use makosh_signal_hub_postgres::runtime_lifecycle::RuntimeLifecycleStore;

use makosh_hub_backend::platform::events::runtime::runtime_allows_processing;
use makosh_hub_backend::platform::settings::store::ApplicationSettingsStore;
use makosh_hub_backend::workflows::persona_derived_evidence::{
    PERSONA_DERIVED_EVIDENCE_CONSUMER, project_persona_derived_evidence_event,
};
use makosh_hub_backend::workflows::project_link_review_effects::{
    PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, PROJECT_LINK_REVIEW_EVENT_TYPE,
    project_link_review_effect_event,
};

#[tokio::test]
async fn signal_hub_restores_canonical_sources_idempotently() {
    let ctx = TestContext::new().await;
    let store = SignalHubStore::new(ctx.pool().clone());

    let first = store
        .restore_system_sources()
        .await
        .expect("first fixture restore");
    let second = store
        .restore_system_sources()
        .await
        .expect("second fixture restore");

    assert_eq!(first.sources_created, 16);
    assert_eq!(first.profiles_created, 4);
    assert_eq!(second.sources_created, 0);
    assert_eq!(second.sources_repaired, 0);
    assert_eq!(second.profiles_created, 0);
    assert_eq!(second.profiles_repaired, 0);

    let sources = store.list_sources().await.expect("list sources");
    let source_codes: Vec<_> = sources.iter().map(|source| source.code.as_str()).collect();

    assert_eq!(
        source_codes,
        vec![
            "ai",
            "browser",
            "calendar",
            "filesystem",
            "fixture",
            "github",
            "home_assistant",
            "mail",
            "rss",
            "system",
            "telegram",
            "voice",
            "whatsapp",
            "yandex_telemost",
            "zoom",
            "zulip",
        ]
    );

    let telegram = sources
        .iter()
        .find(|source| source.code == "telegram")
        .expect("telegram source exists");

    assert!(telegram.default_enabled);
    assert!(telegram.supports_connections);
    assert!(telegram.supports_runtime);
    assert!(telegram.supports_pause);
    assert!(telegram.supports_mute);

    let profiles = store.list_profiles().await.expect("list profiles");
    let profile_codes: Vec<_> = profiles
        .iter()
        .map(|profile| profile.code.as_str())
        .collect();
    assert_eq!(
        profile_codes,
        vec!["development", "maintenance", "production", "testing"]
    );
}

#[tokio::test]
async fn supervised_runtime_failure_updates_signal_hub_state_and_health() {
    let ctx = TestContext::new().await;
    let store = SignalHubStore::new(ctx.pool().clone());
    store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    let lifecycle = RuntimeLifecycleStore::new(ctx.pool().clone());

    lifecycle
        .record_runtime_lifecycle(&RuntimeLifecycleUpdate::running(
            "zulip",
            "zulip_event_ingest",
            "zulip_event_ingest",
            json!({"class": "background"}),
        ))
        .await
        .expect("record runtime start");
    lifecycle
        .record_runtime_lifecycle(&RuntimeLifecycleUpdate::degraded(
            "zulip",
            "zulip_event_ingest",
            "zulip_event_ingest",
            "provider_unavailable",
            json!({"state": "error"}),
        ))
        .await
        .expect("record runtime failure");

    let runtime = store
        .runtime_state("zulip", "zulip_event_ingest")
        .await
        .expect("load runtime state")
        .expect("runtime state");
    assert_eq!(runtime.state, "error");
    assert_eq!(
        runtime.last_error_code.as_deref(),
        Some("provider_unavailable")
    );

    let health = store
        .list_health()
        .await
        .expect("list health")
        .into_iter()
        .find(|health| health.source_code == "zulip" && health.connection_id.is_none())
        .expect("zulip health");
    assert_eq!(health.level, "degraded");
    assert_eq!(
        health
            .evidence
            .get("error_code")
            .and_then(serde_json::Value::as_str),
        Some("provider_unavailable")
    );
}

#[tokio::test]
async fn signal_hub_restore_repairs_existing_system_profile_by_uuid_id() {
    let ctx = TestContext::new().await;
    let store = SignalHubStore::new(ctx.pool().clone());

    store
        .restore_system_sources()
        .await
        .expect("initial fixture restore");

    sqlx::query(
        r#"
        UPDATE signal_profiles
        SET
            display_name = 'Broken Testing Profile',
            source_policies = '[]'::jsonb
        WHERE code = 'testing'
        "#,
    )
    .execute(ctx.pool())
    .await
    .expect("corrupt testing profile fixture");

    let report = store
        .restore_system_sources()
        .await
        .expect("repair fixture restore");

    assert_eq!(report.profiles_repaired, 1);

    let profiles = store.list_profiles().await.expect("list profiles");
    let testing = profiles
        .iter()
        .find(|profile| profile.code == "testing")
        .expect("testing profile");

    assert_eq!(testing.display_name, "Testing");
    assert_eq!(testing.source_policies.len(), 13);
}

#[test]
fn signal_policy_evaluator_applies_reject_pause_mute_allow_order() {
    let now = Utc::now();
    let policies = vec![
        SignalPolicy {
            scope: SignalPolicyScope::Global,
            source_code: None,
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Muted,
            reason: "maintenance window".to_owned(),
            expires_at: None,
        },
        SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Paused,
            reason: "review telegram backlog".to_owned(),
            expires_at: None,
        },
        SignalPolicy {
            scope: SignalPolicyScope::EventPattern,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: Some("signal.raw.telegram.message.observed".to_owned()),
            mode: SignalPolicyMode::Disabled,
            reason: "reject telegram messages".to_owned(),
            expires_at: None,
        },
        SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Disabled,
            reason: "expired source policy".to_owned(),
            expires_at: Some(now - Duration::minutes(5)),
        },
    ];

    let decision = SignalPolicyEvaluator::new(now).decide(
        "telegram",
        None,
        "signal.raw.telegram.message.observed",
        &policies,
    );

    assert_eq!(
        decision,
        SignalPolicyDecision::Rejected {
            reason: "reject telegram messages".to_owned()
        }
    );

    let decision = SignalPolicyEvaluator::new(now).decide(
        "telegram",
        None,
        "signal.raw.telegram.typing.observed",
        &policies,
    );

    assert_eq!(
        decision,
        SignalPolicyDecision::Paused {
            reason: "review telegram backlog".to_owned()
        }
    );

    let decision = SignalPolicyEvaluator::new(now).decide(
        "mail",
        None,
        "signal.raw.mail.message.observed",
        &policies,
    );

    assert_eq!(
        decision,
        SignalPolicyDecision::Muted {
            reason: "maintenance window".to_owned()
        }
    );
}

#[tokio::test]
async fn event_store_queries_signal_events_by_type_source_subject_correlation_and_time() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let occurred_at = Utc::now();

    let telegram = NewEventEnvelope::builder(
        format!(
            "evt_signal_query_telegram_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "message-1"
        }),
    )
    .payload(json!({"summary": "metadata only"}))
    .correlation_id("corr-signal-query")
    .build()
    .expect("valid telegram signal");

    let mail = NewEventEnvelope::builder(
        format!(
            "evt_signal_query_mail_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.mail.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "mail",
            "source_id": "mail-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "mail",
            "entity_id": "message-2"
        }),
    )
    .payload(json!({"blob_ref": "mail/blob/ref"}))
    .correlation_id("corr-other-signal-query")
    .build()
    .expect("valid mail signal");

    store.append(&telegram).await.expect("append telegram");
    store.append(&mail).await.expect("append mail");

    let queried = store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.raw.telegram.message.observed")
                .source_code("telegram")
                .subject_kind("signal")
                .subject_entity_id("message-1")
                .correlation_id("corr-signal-query")
                .occurred_between(
                    occurred_at - Duration::seconds(1),
                    occurred_at + Duration::seconds(1),
                )
                .limit(10),
        )
        .await
        .expect("query signal events");

    assert_eq!(queried.len(), 1);
    assert_eq!(queried[0].event.event_id, telegram.event_id);
}

#[tokio::test]
async fn signal_hub_accepts_raw_signal_when_no_policy_blocks_it() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    let event_store = EventStore::new(ctx.pool().clone());
    let service = SignalHubSignalService::new(
        Arc::new(RawSignalStore::new(ctx.pool().clone())),
        event_store.clone(),
    );
    let occurred_at = Utc::now();
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_raw_accept_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-accepted"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-message-accepted"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-message-accepted"}))
    .correlation_id("corr-raw-accept")
    .build()
    .expect("valid raw signal");

    event_store
        .append_for_dispatch(&raw)
        .await
        .expect("append raw");
    let raw_event = event_store
        .get_by_id(&raw.event_id)
        .await
        .expect("load raw")
        .expect("raw exists");

    let outcome = service
        .process_raw_signal(&raw_event)
        .await
        .expect("process raw signal");

    assert!(matches!(outcome, SignalProcessingOutcome::Accepted { .. }));

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .source_code("telegram")
                .correlation_id("corr-raw-accept")
                .limit(10),
        )
        .await
        .expect("query accepted signal");

    assert_eq!(accepted.len(), 1);
    assert_eq!(
        accepted[0].event.causation_id.as_deref(),
        Some(raw.event_id.as_str())
    );
    assert_eq!(accepted[0].event.payload, raw.payload);
}

#[tokio::test]
async fn signal_hub_pause_policy_buffers_raw_signal_without_accepted_publication() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    signal_store
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Paused,
            reason: "manual pause".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create pause policy");

    let event_store = EventStore::new(ctx.pool().clone());
    let service = SignalHubSignalService::new(
        Arc::new(RawSignalStore::new(ctx.pool().clone())),
        event_store.clone(),
    );
    let occurred_at = Utc::now();
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_raw_pause_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-paused"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-message-paused"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-message-paused"}))
    .correlation_id("corr-raw-pause")
    .build()
    .expect("valid raw signal");

    event_store
        .append_for_dispatch(&raw)
        .await
        .expect("append raw");
    let raw_event = event_store
        .get_by_id(&raw.event_id)
        .await
        .expect("load raw")
        .expect("raw exists");

    let outcome = service
        .process_raw_signal(&raw_event)
        .await
        .expect("process raw signal");

    assert_eq!(
        outcome,
        SignalProcessingOutcome::Paused {
            reason: "manual pause".to_owned()
        }
    );

    assert_eq!(
        signal_store
            .paused_event_count("telegram")
            .await
            .expect("paused count"),
        1
    );

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .source_code("telegram")
                .correlation_id("corr-raw-pause")
                .limit(10),
        )
        .await
        .expect("query accepted signal");

    assert!(accepted.is_empty());
}

#[tokio::test]
async fn signal_hub_connection_scoped_pause_policy_matches_raw_event_account_binding() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let connection = signal_store
        .create_connection(&SignalConnectionCreate {
            source_code: "telegram".to_owned(),
            display_name: "Primary Telegram".to_owned(),
            status: "connected".to_owned(),
            profile: Some("default".to_owned()),
            settings: json!({"account_id": "telegram-account-primary"}),
            secret_ref: None,
        })
        .await
        .expect("create telegram connection");

    signal_store
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Connection,
            source_code: Some("telegram".to_owned()),
            connection_id: Some(connection.id.clone()),
            event_pattern: None,
            mode: SignalPolicyMode::Paused,
            reason: "pause only one telegram account".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create connection pause policy");

    let event_store = EventStore::new(ctx.pool().clone());
    let service = SignalHubSignalService::new(
        Arc::new(RawSignalStore::new(ctx.pool().clone())),
        event_store.clone(),
    );
    let occurred_at = Utc::now();
    let paused_raw = NewEventEnvelope::builder(
        format!(
            "evt_raw_pause_connection_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-paused-connection",
            "account_id": "telegram-account-primary"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-message-paused-connection",
            "account_id": "telegram-account-primary"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-message-paused-connection"}))
    .correlation_id("corr-raw-pause-connection")
    .build()
    .expect("valid paused connection raw signal");
    let allowed_raw = NewEventEnvelope::builder(
        format!(
            "evt_raw_pause_other_connection_{}",
            occurred_at.timestamp_nanos_opt().unwrap_or_default() + 1
        ),
        "signal.raw.telegram.message.observed",
        occurred_at + chrono::Duration::seconds(1),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-other-connection",
            "account_id": "telegram-account-secondary"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-message-other-connection",
            "account_id": "telegram-account-secondary"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-message-other-connection"}))
    .correlation_id("corr-raw-other-connection")
    .build()
    .expect("valid other connection raw signal");

    event_store
        .append_for_dispatch(&paused_raw)
        .await
        .expect("append paused raw");
    let paused_raw_event = event_store
        .get_by_id(&paused_raw.event_id)
        .await
        .expect("load paused raw")
        .expect("paused raw exists");
    let paused_outcome = service
        .process_raw_signal(&paused_raw_event)
        .await
        .expect("process paused connection raw");
    assert_eq!(
        paused_outcome,
        SignalProcessingOutcome::Paused {
            reason: "pause only one telegram account".to_owned()
        }
    );

    event_store
        .append_for_dispatch(&allowed_raw)
        .await
        .expect("append allowed raw");
    let allowed_raw_event = event_store
        .get_by_id(&allowed_raw.event_id)
        .await
        .expect("load allowed raw")
        .expect("allowed raw exists");
    let allowed_outcome = service
        .process_raw_signal(&allowed_raw_event)
        .await
        .expect("process allowed connection raw");
    assert!(matches!(
        allowed_outcome,
        SignalProcessingOutcome::Accepted { .. }
    ));

    let paused_request = signal_store
        .create_replay_request(&SignalReplayRequestCreate {
            source_code: Some("telegram".to_owned()),
            connection_id: Some(connection.id.clone()),
            event_pattern: Some("signal.raw.telegram.*".to_owned()),
            from_position: None,
            to_position: None,
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: None,
            requested_by: "test".to_owned(),
            metadata: json!({}),
        })
        .await
        .expect("create paused replay request");

    let paused_events = signal_store
        .list_paused_events_for_replay(&paused_request, 10)
        .await
        .expect("list paused events for connection replay");
    assert_eq!(paused_events.len(), 1);
    assert_eq!(paused_events[0].event_id, paused_raw.event_id);
}

#[tokio::test]
async fn signal_hub_replay_request_canonicalizes_legacy_person_projection_alias() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());

    let request = signal_store
        .create_replay_request(&SignalReplayRequestCreate {
            source_code: None,
            connection_id: None,
            event_pattern: Some(PERSONA_ROLE_ASSIGNED_EVENT_TYPE.to_owned()),
            from_position: Some(1),
            to_position: Some(1),
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: Some("person_derived_evidence".to_owned()),
            requested_by: "test".to_owned(),
            metadata: json!({}),
        })
        .await
        .expect("create replay request with legacy projection alias");

    assert_eq!(
        request.target_projection.as_deref(),
        Some("persona_derived_evidence")
    );
    assert_eq!(
        request.metadata["target_projection"],
        json!("persona_derived_evidence")
    );
}

#[tokio::test]
async fn signal_hub_connection_status_orchestrates_operator_policy_for_signal_flow() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let signal_store = SignalHubStore::new(pool.clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let event_store = EventStore::new(pool.clone());
    let connection_service =
        SignalHubConnectionService::new(signal_store.clone(), event_store.clone());
    let signal_service = SignalHubSignalService::new(
        Arc::new(RawSignalStore::new(pool.clone())),
        event_store.clone(),
    );

    let connection = connection_service
        .create_connection(&SignalConnectionCreate {
            source_code: "telegram".to_owned(),
            display_name: "Operator Controlled Telegram".to_owned(),
            status: "paused".to_owned(),
            profile: Some("default".to_owned()),
            settings: json!({"account_id": "telegram-operator-account"}),
            secret_ref: None,
        })
        .await
        .expect("create paused connection");
    assert_eq!(connection.status, "paused");

    let policies = RawSignalStore::new(pool.clone())
        .list_active_policies()
        .await
        .expect("list active policies after create");
    assert!(policies.iter().any(|policy| {
        policy.scope == SignalPolicyScope::Connection
            && policy.connection_id.as_deref() == Some(connection.id.as_str())
            && policy.mode == SignalPolicyMode::Paused
    }));

    let paused_raw = NewEventEnvelope::builder(
        "evt_connection_status_paused_raw",
        "signal.raw.telegram.message.observed",
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-operator-paused",
            "account_id": "telegram-operator-account"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-operator-paused",
            "account_id": "telegram-operator-account"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-operator-paused"}))
    .build()
    .expect("valid paused raw");
    event_store
        .append_for_dispatch(&paused_raw)
        .await
        .expect("append paused raw");
    let paused_outcome = signal_service
        .process_raw_signal(
            &event_store
                .get_by_id(&paused_raw.event_id)
                .await
                .expect("load paused raw")
                .expect("paused raw exists"),
        )
        .await
        .expect("process paused raw");
    assert!(matches!(
        paused_outcome,
        SignalProcessingOutcome::Paused { .. }
    ));

    let muted_connection = connection_service
        .update_connection(
            &makosh_hub_backend::domains::signal_hub::store::SignalConnectionUpdate {
                id: connection.id.clone(),
                display_name: None,
                status: Some("muted".to_owned()),
                profile: None,
                settings: None,
                secret_ref: None,
            },
        )
        .await
        .expect("update connection to muted");
    assert_eq!(muted_connection.status, "muted");

    let muted_raw = NewEventEnvelope::builder(
        "evt_connection_status_muted_raw",
        "signal.raw.telegram.message.observed",
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-operator-muted",
            "account_id": "telegram-operator-account"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-operator-muted",
            "account_id": "telegram-operator-account"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-operator-muted"}))
    .build()
    .expect("valid muted raw");
    event_store
        .append_for_dispatch(&muted_raw)
        .await
        .expect("append muted raw");
    let muted_outcome = signal_service
        .process_raw_signal(
            &event_store
                .get_by_id(&muted_raw.event_id)
                .await
                .expect("load muted raw")
                .expect("muted raw exists"),
        )
        .await
        .expect("process muted raw");
    assert!(matches!(
        muted_outcome,
        SignalProcessingOutcome::Muted { .. }
    ));

    let disabled_connection = connection_service
        .update_connection(
            &makosh_hub_backend::domains::signal_hub::store::SignalConnectionUpdate {
                id: connection.id.clone(),
                display_name: None,
                status: Some("disabled".to_owned()),
                profile: None,
                settings: None,
                secret_ref: None,
            },
        )
        .await
        .expect("update connection to disabled");
    assert_eq!(disabled_connection.status, "disabled");

    let disabled_raw = NewEventEnvelope::builder(
        "evt_connection_status_disabled_raw",
        "signal.raw.telegram.message.observed",
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-operator-disabled",
            "account_id": "telegram-operator-account"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-operator-disabled",
            "account_id": "telegram-operator-account"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-operator-disabled"}))
    .build()
    .expect("valid disabled raw");
    event_store
        .append_for_dispatch(&disabled_raw)
        .await
        .expect("append disabled raw");
    let disabled_outcome = signal_service
        .process_raw_signal(
            &event_store
                .get_by_id(&disabled_raw.event_id)
                .await
                .expect("load disabled raw")
                .expect("disabled raw exists"),
        )
        .await
        .expect("process disabled raw");
    assert!(matches!(
        disabled_outcome,
        SignalProcessingOutcome::Rejected { .. }
    ));

    let connected_connection = connection_service
        .update_connection(
            &makosh_hub_backend::domains::signal_hub::store::SignalConnectionUpdate {
                id: connection.id.clone(),
                display_name: None,
                status: Some("connected".to_owned()),
                profile: None,
                settings: None,
                secret_ref: None,
            },
        )
        .await
        .expect("update connection to connected");
    assert_eq!(connected_connection.status, "connected");

    let policies_after_connect = RawSignalStore::new(pool.clone())
        .list_active_policies()
        .await
        .expect("list active policies after connect");
    assert!(!policies_after_connect.iter().any(|policy| {
        policy.scope == SignalPolicyScope::Connection
            && policy.connection_id.as_deref() == Some(connection.id.as_str())
            && matches!(
                policy.mode,
                SignalPolicyMode::Paused | SignalPolicyMode::Muted | SignalPolicyMode::Disabled
            )
    }));

    let connected_raw = NewEventEnvelope::builder(
        "evt_connection_status_connected_raw",
        "signal.raw.telegram.message.observed",
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-operator-connected",
            "account_id": "telegram-operator-account"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-operator-connected",
            "account_id": "telegram-operator-account"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-operator-connected"}))
    .build()
    .expect("valid connected raw");
    event_store
        .append_for_dispatch(&connected_raw)
        .await
        .expect("append connected raw");
    let connected_outcome = signal_service
        .process_raw_signal(
            &event_store
                .get_by_id(&connected_raw.event_id)
                .await
                .expect("load connected raw")
                .expect("connected raw exists"),
        )
        .await
        .expect("process connected raw");
    assert!(matches!(
        connected_outcome,
        SignalProcessingOutcome::Accepted { .. }
    ));
}

#[tokio::test]
async fn signal_hub_replay_request_releases_paused_signal_into_accepted_flow() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    signal_store
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::Source,
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: None,
            mode: SignalPolicyMode::Paused,
            reason: "manual pause".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create pause policy");

    let event_store = EventStore::new(ctx.pool().clone());
    let signal_service = SignalHubSignalService::new(
        Arc::new(RawSignalStore::new(ctx.pool().clone())),
        event_store.clone(),
    );
    let replay_service = SignalHubReplayService::new(signal_store.clone(), event_store.clone());
    let occurred_at = Utc::now();
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_raw_replay_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-message-replay"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-message-replay"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-message-replay"}))
    .correlation_id("corr-raw-replay")
    .build()
    .expect("valid raw signal");

    event_store
        .append_for_dispatch(&raw)
        .await
        .expect("append raw");
    let raw_event = event_store
        .get_by_id(&raw.event_id)
        .await
        .expect("load raw")
        .expect("raw exists");

    let outcome = signal_service
        .process_raw_signal(&raw_event)
        .await
        .expect("pause raw signal");
    assert!(matches!(outcome, SignalProcessingOutcome::Paused { .. }));
    assert_eq!(
        signal_store
            .paused_event_count("telegram")
            .await
            .expect("paused count"),
        1
    );

    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: Some("signal.raw.telegram.*".to_owned()),
            from_position: None,
            to_position: None,
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: None,
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "signal_hub_test"}),
        })
        .await
        .expect("request replay");
    assert_eq!(request.status, "queued");

    let report = replay_service
        .process_next_request()
        .await
        .expect("process replay request")
        .expect("replay report");
    assert_eq!(report.replayed_count, 1);

    assert_eq!(
        signal_store
            .paused_event_count("telegram")
            .await
            .expect("paused count after replay"),
        0
    );

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .source_code("telegram")
                .correlation_id("corr-raw-replay")
                .limit(10),
        )
        .await
        .expect("query accepted replayed signal");
    assert_eq!(accepted.len(), 1);

    let replay_completed = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.replay.completed")
                .correlation_id(request.id.clone())
                .limit(10),
        )
        .await
        .expect("query replay completion events");
    assert_eq!(replay_completed.len(), 1);
}

#[tokio::test]
async fn signal_hub_replay_request_can_replay_event_log_range_into_accepted_flow() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let event_store = EventStore::new(ctx.pool().clone());
    let replay_service = SignalHubReplayService::new(signal_store.clone(), event_store.clone());
    let occurred_at = Utc::now();

    let first = NewEventEnvelope::builder(
        format!(
            "evt_raw_range_first_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-range-first"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-range-first"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-range-first"}))
    .correlation_id("corr-range-first")
    .build()
    .expect("valid raw first");
    let second = NewEventEnvelope::builder(
        format!(
            "evt_raw_range_second_{}",
            occurred_at.timestamp_nanos_opt().unwrap_or_default() + 1
        ),
        "signal.raw.telegram.message.observed",
        occurred_at + chrono::Duration::seconds(1),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-range-second"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-range-second"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-range-second"}))
    .correlation_id("corr-range-second")
    .build()
    .expect("valid raw second");

    let first_position = event_store
        .append_for_dispatch(&first)
        .await
        .expect("append first raw");
    let second_position = event_store
        .append_for_dispatch(&second)
        .await
        .expect("append second raw");

    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: Some("signal.raw.telegram.*".to_owned()),
            from_position: Some(first_position),
            to_position: Some(second_position),
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: None,
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "event_log_range"}),
        })
        .await
        .expect("request event-log replay");

    let report = replay_service
        .process_next_request()
        .await
        .expect("process event-log replay request")
        .expect("event-log replay report");
    assert_eq!(report.replayed_count, 2);

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .source_code("telegram")
                .limit(10),
        )
        .await
        .expect("query accepted event-log replayed signals");
    assert!(
        accepted
            .iter()
            .any(|event| event.event.correlation_id.as_deref() == Some("corr-range-first"))
    );
    assert!(
        accepted
            .iter()
            .any(|event| event.event.correlation_id.as_deref() == Some("corr-range-second"))
    );

    let replay_completed = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.replay.completed")
                .correlation_id(request.id.clone())
                .limit(10),
        )
        .await
        .expect("query range replay completion events");
    assert_eq!(replay_completed.len(), 1);
    assert_eq!(replay_completed[0].event.payload["replayed_count"], 2);
}

#[tokio::test]
async fn signal_hub_replay_request_can_filter_event_log_range_by_connection_binding() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let connection = signal_store
        .create_connection(&SignalConnectionCreate {
            source_code: "telegram".to_owned(),
            display_name: "Primary Telegram".to_owned(),
            status: "connected".to_owned(),
            profile: Some("default".to_owned()),
            settings: json!({"account_id": "telegram-account-primary"}),
            secret_ref: None,
        })
        .await
        .expect("create telegram connection");

    let event_store = EventStore::new(ctx.pool().clone());
    let replay_service = SignalHubReplayService::new(signal_store.clone(), event_store.clone());
    let occurred_at = Utc::now();

    let first = NewEventEnvelope::builder(
        format!(
            "evt_raw_connection_range_first_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-connection-range-first",
            "account_id": "telegram-account-primary"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-connection-range-first",
            "account_id": "telegram-account-primary"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-connection-range-first"}))
    .correlation_id("corr-connection-range-first")
    .build()
    .expect("valid connection scoped raw first");
    let second = NewEventEnvelope::builder(
        format!(
            "evt_raw_connection_range_second_{}",
            occurred_at.timestamp_nanos_opt().unwrap_or_default() + 1
        ),
        "signal.raw.telegram.message.observed",
        occurred_at + chrono::Duration::seconds(1),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-connection-range-second",
            "account_id": "telegram-account-secondary"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-connection-range-second",
            "account_id": "telegram-account-secondary"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-connection-range-second"}))
    .correlation_id("corr-connection-range-second")
    .build()
    .expect("valid connection scoped raw second");

    let first_position = event_store
        .append_for_dispatch(&first)
        .await
        .expect("append first connection raw");
    let second_position = event_store
        .append_for_dispatch(&second)
        .await
        .expect("append second connection raw");

    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: Some("telegram".to_owned()),
            connection_id: Some(connection.id.clone()),
            event_pattern: Some("signal.raw.telegram.*".to_owned()),
            from_position: Some(first_position),
            to_position: Some(second_position),
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: None,
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "event_log_connection_range"}),
        })
        .await
        .expect("request connection-scoped event-log replay");

    let report = replay_service
        .process_next_request()
        .await
        .expect("process connection-scoped event-log replay request")
        .expect("connection-scoped event-log replay report");
    assert_eq!(report.replayed_count, 1);

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .source_code("telegram")
                .limit(10),
        )
        .await
        .expect("query accepted connection replayed signals");
    assert!(
        accepted
            .iter()
            .any(|event| event.event.correlation_id.as_deref()
                == Some("corr-connection-range-first"))
    );
    assert!(
        accepted
            .iter()
            .all(|event| event.event.correlation_id.as_deref()
                != Some("corr-connection-range-second"))
    );

    let replay_completed = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.replay.completed")
                .correlation_id(request.id.clone())
                .limit(10),
        )
        .await
        .expect("query connection range replay completion events");
    assert_eq!(replay_completed.len(), 1);
    assert_eq!(replay_completed[0].event.payload["replayed_count"], 1);
}

#[tokio::test]
async fn signal_hub_target_consumer_replay_rewinds_consumer_cursor_and_skips_duplicates() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let event_store = EventStore::new(ctx.pool().clone());
    let replay_service = SignalHubReplayService::new(signal_store.clone(), event_store.clone());
    let occurred_at = Utc::now();
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_target_consumer_replay_raw_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-target-consumer-replay"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-target-consumer-replay"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-target-consumer-replay"}))
    .correlation_id("corr-target-consumer-replay")
    .build()
    .expect("valid raw replay signal");
    let position = event_store
        .append_for_dispatch(&raw)
        .await
        .expect("append target consumer raw");

    let consumer_name = format!("signal_hub_consumer_replay_{}", position);
    let runner = EventConsumerRunner::new(
        ctx.pool().clone(),
        EventConsumerConfig {
            consumer_name: consumer_name.clone(),
            batch_size: 1,
            max_attempts: 1,
            retry_base_seconds: 0,
        },
    );
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place consumer before replay event");

    let call_count = Arc::new(AtomicUsize::new(0));
    let first_counter = Arc::clone(&call_count);
    let first = runner
        .process_next_batch(move |_| {
            let first_counter = Arc::clone(&first_counter);
            async move {
                first_counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("process first batch");
    assert_eq!(first.processed, 1);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: Some("signal.raw.telegram.*".to_owned()),
            from_position: Some(position),
            to_position: Some(position),
            from_time: None,
            to_time: None,
            target_consumer: Some(consumer_name.clone()),
            target_projection: None,
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "target_consumer_duplicate"}),
        })
        .await
        .expect("request targeted consumer replay");
    assert_eq!(
        request.target_consumer.as_deref(),
        Some(consumer_name.as_str())
    );

    let report = replay_service
        .process_next_request()
        .await
        .expect("process targeted consumer replay request")
        .expect("targeted replay report");
    assert_eq!(report.replayed_count, 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("rewound cursor"),
        position - 1
    );

    let second_counter = Arc::clone(&call_count);
    let replayed = runner
        .process_next_batch(move |_| {
            let second_counter = Arc::clone(&second_counter);
            async move {
                second_counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("process replayed batch");
    assert_eq!(replayed.processed, 0);
    assert_eq!(replayed.skipped_duplicates, 1);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn signal_hub_target_consumer_replay_reopens_dead_lettered_event_for_normal_runner() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let event_store = EventStore::new(ctx.pool().clone());
    let replay_service = SignalHubReplayService::new(signal_store.clone(), event_store.clone());
    let occurred_at = Utc::now();
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_target_consumer_dead_letter_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-target-consumer-dead-letter"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-target-consumer-dead-letter"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-target-consumer-dead-letter"}))
    .correlation_id("corr-target-consumer-dead-letter")
    .build()
    .expect("valid dead-letter replay signal");
    let position = event_store
        .append_for_dispatch(&raw)
        .await
        .expect("append dead-letter replay raw");

    let consumer_name = format!("signal_hub_dead_letter_replay_{}", position);
    let runner = EventConsumerRunner::new(
        ctx.pool().clone(),
        EventConsumerConfig {
            consumer_name: consumer_name.clone(),
            batch_size: 1,
            max_attempts: 1,
            retry_base_seconds: 0,
        },
    );
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place consumer before dead-letter event");

    let failed = runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "poison event".to_owned(),
            ))
        })
        .await
        .expect("dead-letter batch");
    assert_eq!(failed.dead_lettered, 1);
    let dead_letter = runner
        .store()
        .dead_letter_for_event(&consumer_name, position)
        .await
        .expect("load dead letter")
        .expect("dead letter exists");
    assert_eq!(dead_letter.review_state, EventDeadLetterReviewState::Open);

    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: Some("signal.raw.telegram.*".to_owned()),
            from_position: Some(position),
            to_position: Some(position),
            from_time: None,
            to_time: None,
            target_consumer: Some(consumer_name.clone()),
            target_projection: None,
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "target_consumer_dead_letter"}),
        })
        .await
        .expect("request dead-letter targeted replay");

    let report = replay_service
        .process_next_request()
        .await
        .expect("process dead-letter targeted replay")
        .expect("dead-letter targeted replay report");
    assert_eq!(report.replayed_count, 1);
    assert_eq!(
        request.target_consumer.as_deref(),
        Some(consumer_name.as_str())
    );
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("rewound cursor after targeted replay"),
        position - 1
    );

    let recovered = runner
        .process_next_batch(|_| async { Ok(()) })
        .await
        .expect("recovered batch");
    assert_eq!(recovered.processed, 1);

    let dead_letter_after = runner
        .store()
        .dead_letter_for_event(&consumer_name, position)
        .await
        .expect("load replayed dead letter")
        .expect("dead letter still exists for audit");
    assert_eq!(
        dead_letter_after.review_state,
        EventDeadLetterReviewState::Replayed
    );
}

#[tokio::test]
async fn signal_hub_timeline_projection_replay_rewinds_projection_cursor_and_emits_update_event() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let event_store = EventStore::new(ctx.pool().clone());
    let replay_service = SignalHubReplayService::new(signal_store.clone(), event_store.clone());
    let occurred_at = Utc::now();
    let first = NewEventEnvelope::builder(
        format!(
            "evt_timeline_projection_replay_message_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "message_received",
        occurred_at,
        json!({
            "kind": "communication_messages",
            "source_id": "timeline-projection-replay-message"
        }),
        json!({
            "kind": "persona",
            "entity_id": "persona:v1:human:alice"
        }),
    )
    .payload(json!({"title": "Message from Alice"}))
    .build()
    .expect("valid timeline message event");
    let second = NewEventEnvelope::builder(
        format!(
            "evt_timeline_projection_replay_decision_{}",
            occurred_at.timestamp_nanos_opt().unwrap_or_default() + 1
        ),
        "decision_recorded",
        occurred_at + Duration::seconds(1),
        json!({
            "kind": "decisions",
            "source_id": "timeline-projection-replay-decision"
        }),
        json!({
            "kind": "project",
            "entity_id": "project:makosh"
        }),
    )
    .payload(json!({"title": "Decision accepted"}))
    .build()
    .expect("valid timeline decision event");

    let first_position = event_store
        .append_for_dispatch(&first)
        .await
        .expect("append first timeline event");
    let second_position = event_store
        .append_for_dispatch(&second)
        .await
        .expect("append second timeline event");

    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: None,
            connection_id: None,
            event_pattern: None,
            from_position: Some(first_position),
            to_position: Some(second_position),
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: Some("timeline_event_log".to_owned()),
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "timeline_projection_rebuild"}),
        })
        .await
        .expect("request timeline projection replay");
    assert_eq!(
        request.target_projection.as_deref(),
        Some("timeline_event_log")
    );

    let report = replay_service
        .process_next_request()
        .await
        .expect("process timeline projection replay")
        .expect("timeline projection report");
    assert_eq!(report.replayed_count, 2);

    let cursor_store = ProjectionCursorStore::new(ctx.pool().clone());
    assert_eq!(
        cursor_store
            .last_processed_position("signal_hub.timeline_event_log")
            .await
            .expect("timeline projection cursor"),
        second_position
    );

    let projection_events = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("timeline.projection.updated")
                .correlation_id(request.id.clone())
                .limit(10),
        )
        .await
        .expect("query timeline projection update events");
    assert_eq!(projection_events.len(), 1);
    assert_eq!(
        projection_events[0].event.payload["target_projection"],
        "timeline_event_log"
    );
    assert_eq!(projection_events[0].event.payload["replayed_count"], 2);
    assert_eq!(projection_events[0].event.payload["entries_count"], 2);
}

#[tokio::test]
async fn signal_hub_communication_messages_projection_replay_clears_processed_markers_and_rebuilds_messages()
 {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let signal_store = SignalHubStore::new(pool.clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "telegram-replay-account",
            CommunicationProviderKind::TelegramUser,
            "Telegram Replay",
            "telegram-replay-account",
        ))
        .await
        .expect("provider account");

    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw_signal_hub_projection_replay_telegram",
                "telegram-replay-account",
                "telegram_message",
                "telegram-replay-message-1",
                "sha256:signal-hub:projection-replay:telegram",
                "signal-hub-projection-replay",
                json!({
                    "provider_chat_id": "telegram-replay-chat",
                    "chat_title": "Telegram Replay Chat",
                    "sender_id": "telegram-replay-sender",
                    "sender_display_name": "Replay Sender",
                    "text": "rebuild this message",
                    "delivery_state": "received"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": "telegram-replay-account",
                "provider_chat_id": "telegram-replay-chat",
            })),
        )
        .await
        .expect("raw telegram record");

    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch telegram raw signal")
        .expect("accepted telegram signal");
    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted telegram signal")
        .expect("projected telegram message");

    let event_store = EventStore::new(pool.clone());
    let accepted_stored = event_store
        .list_matching(
            EventLogQuery::default()
                .correlation_id(accepted_event.correlation_id.clone().unwrap_or_default())
                .event_type("signal.accepted.telegram.message")
                .limit(1),
        )
        .await
        .expect("load accepted stored event")
        .into_iter()
        .next()
        .expect("accepted stored event exists");

    let consumer_store = EventConsumerStore::new(pool.clone());
    consumer_store
        .record_processed(
            COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
            &accepted_stored,
        )
        .await
        .expect("record processed accepted event");
    consumer_store
        .save_position(
            COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
            accepted_stored.position,
        )
        .await
        .expect("save consumer position");

    sqlx::query("DELETE FROM communication_messages WHERE message_id = $1")
        .bind(&projected.message_id)
        .execute(&pool)
        .await
        .expect("delete projected message");

    let replay_service = SignalHubReplayService::new(signal_store, event_store.clone());
    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: Some("telegram".to_owned()),
            connection_id: None,
            event_pattern: Some("signal.accepted.telegram.message".to_owned()),
            from_position: Some(accepted_stored.position),
            to_position: Some(accepted_stored.position),
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: Some("communication_messages".to_owned()),
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "communications_projection_rebuild"}),
        })
        .await
        .expect("request communications projection replay");
    assert_eq!(
        request.target_projection.as_deref(),
        Some("communication_messages")
    );

    let report = replay_service
        .process_next_request()
        .await
        .expect("process communications projection replay")
        .expect("communications projection report");
    assert_eq!(report.replayed_count, 1);

    let rebuilt_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM communication_messages WHERE message_id = $1",
    )
    .bind(&projected.message_id)
    .fetch_one(&pool)
    .await
    .expect("rebuilt message count");
    assert_eq!(rebuilt_count, 1);
    assert!(
        consumer_store
            .has_processed_event(
                COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                accepted_stored.position
            )
            .await
            .expect("processed marker after replay")
    );

    let projection_events = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("communications.projection.updated")
                .correlation_id(request.id.clone())
                .limit(10),
        )
        .await
        .expect("query communications projection update events");
    assert_eq!(projection_events.len(), 1);
    assert_eq!(
        projection_events[0].event.payload["target_projection"],
        "communication_messages"
    );
    assert_eq!(projection_events[0].event.payload["replayed_count"], 1);
}

#[tokio::test]
async fn signal_hub_persona_derived_evidence_projection_replay_rebuilds_relationships() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let signal_store = SignalHubStore::new(pool.clone());
    let event_store = EventStore::new(pool.clone());

    let position = event_store
        .append_for_dispatch(
            &NewEventEnvelope::builder(
                "evt_person_role_assigned_replay",
                PERSONA_ROLE_ASSIGNED_EVENT_TYPE,
                Utc::now(),
                json!({
                    "kind": "person",
                    "persona_id": "person-replay-1",
                }),
                json!({
                    "kind": "person_role",
                    "persona_id": "person-replay-1",
                    "role": "friend",
                }),
            )
            .payload(json!({
                "persona_id": "person-replay-1",
                "role": "friend",
                "assigned_by": "owner-replay",
            }))
            .build()
            .expect("valid person role event"),
        )
        .await
        .expect("append person role event");

    let stored_event = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type(PERSONA_ROLE_ASSIGNED_EVENT_TYPE)
                .position_between(position, position)
                .limit(1),
        )
        .await
        .expect("load stored person role event")
        .into_iter()
        .next()
        .expect("stored person role event exists");

    project_persona_derived_evidence_event(pool.clone(), stored_event.clone())
        .await
        .expect("project persona derived evidence once");

    let relationship_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = 'person-replay-1'
          AND relationship_type = 'has_role'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("role relationship count after first projection");
    assert_eq!(relationship_count, 1);

    let consumer_store = EventConsumerStore::new(pool.clone());
    consumer_store
        .record_processed(PERSONA_DERIVED_EVIDENCE_CONSUMER, &stored_event)
        .await
        .expect("record processed persona derived evidence event");
    consumer_store
        .save_position(PERSONA_DERIVED_EVIDENCE_CONSUMER, stored_event.position)
        .await
        .expect("save persona derived evidence consumer position");

    sqlx::query(
        r#"
        DELETE FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = 'person-replay-1'
          AND relationship_type = 'has_role'
        "#,
    )
    .execute(&pool)
    .await
    .expect("delete role relationship");

    let replay_service = SignalHubReplayService::new(signal_store, event_store.clone());
    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: None,
            connection_id: None,
            event_pattern: Some(PERSONA_ROLE_ASSIGNED_EVENT_TYPE.to_owned()),
            from_position: Some(stored_event.position),
            to_position: Some(stored_event.position),
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: Some("persona_derived_evidence".to_owned()),
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "persona_derived_projection_rebuild"}),
        })
        .await
        .expect("request persona derived evidence replay");
    assert_eq!(
        request.target_projection.as_deref(),
        Some("persona_derived_evidence")
    );

    let report = replay_service
        .process_next_request()
        .await
        .expect("process persona derived evidence replay")
        .expect("persona derived evidence report");
    assert_eq!(report.replayed_count, 1);

    let rebuilt_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = 'person-replay-1'
          AND relationship_type = 'has_role'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("rebuilt role relationship count");
    assert_eq!(rebuilt_count, 1);
    assert!(
        consumer_store
            .has_processed_event(PERSONA_DERIVED_EVIDENCE_CONSUMER, stored_event.position)
            .await
            .expect("processed marker after person derived replay")
    );

    let projection_events = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("personas.derived_evidence.updated")
                .correlation_id(request.id.clone())
                .limit(10),
        )
        .await
        .expect("query person derived projection update events");
    assert_eq!(projection_events.len(), 1);
    assert_eq!(
        projection_events[0].event.payload["target_projection"],
        "persona_derived_evidence"
    );
    assert_eq!(projection_events[0].event.payload["replayed_count"], 1);
}

#[tokio::test]
async fn signal_hub_project_link_review_effects_projection_replay_rebuilds_relationships_and_decisions()
 {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let signal_store = SignalHubStore::new(pool.clone());
    let event_store = EventStore::new(pool.clone());

    let position = event_store
        .append_for_dispatch(
            &NewEventEnvelope::builder(
                "evt_project_link_review_replay",
                PROJECT_LINK_REVIEW_EVENT_TYPE,
                Utc::now(),
                json!({
                    "kind": "project",
                    "project_id": "project-replay-1",
                }),
                json!({
                    "kind": "project_link_review",
                    "project_id": "project-replay-1",
                    "target_kind": "message",
                    "target_id": "message-replay-1",
                }),
            )
            .payload(json!({
                "project_id": "project-replay-1",
                "target_kind": "message",
                "target_id": "message-replay-1",
                "review_state": "user_confirmed",
            }))
            .build()
            .expect("valid project link review event"),
        )
        .await
        .expect("append project link review event");

    let stored_event = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type(PROJECT_LINK_REVIEW_EVENT_TYPE)
                .position_between(position, position)
                .limit(1),
        )
        .await
        .expect("load stored project link review event")
        .into_iter()
        .next()
        .expect("stored project link review event exists");

    project_link_review_effect_event(pool.clone(), stored_event.clone())
        .await
        .expect("project link review effects once");

    let relationship_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM relationships
        WHERE source_entity_kind = 'project'
          AND source_entity_id = 'project-replay-1'
          AND relationship_type = 'project_has_message'
          AND target_entity_id = 'message-replay-1'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("project relationship count after first projection");
    assert_eq!(relationship_count, 1);

    let decision_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM decisions
        WHERE metadata ->> 'project_id' = 'project-replay-1'
          AND metadata ->> 'target_id' = 'message-replay-1'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("project decision count after first projection");
    assert_eq!(decision_count, 1);

    let consumer_store = EventConsumerStore::new(pool.clone());
    consumer_store
        .record_processed(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, &stored_event)
        .await
        .expect("record processed project link review event");
    consumer_store
        .save_position(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, stored_event.position)
        .await
        .expect("save project link review consumer position");

    sqlx::query(
        r#"
        DELETE FROM relationships
        WHERE source_entity_kind = 'project'
          AND source_entity_id = 'project-replay-1'
          AND relationship_type = 'project_has_message'
          AND target_entity_id = 'message-replay-1'
        "#,
    )
    .execute(&pool)
    .await
    .expect("delete project relationship");
    sqlx::query(
        r#"
        DELETE FROM decisions
        WHERE metadata ->> 'project_id' = 'project-replay-1'
          AND metadata ->> 'target_id' = 'message-replay-1'
        "#,
    )
    .execute(&pool)
    .await
    .expect("delete project decision");

    let replay_service = SignalHubReplayService::new(signal_store, event_store.clone());
    let request = replay_service
        .request_replay(&SignalReplayRequestCreate {
            source_code: None,
            connection_id: None,
            event_pattern: Some(PROJECT_LINK_REVIEW_EVENT_TYPE.to_owned()),
            from_position: Some(stored_event.position),
            to_position: Some(stored_event.position),
            from_time: None,
            to_time: None,
            target_consumer: None,
            target_projection: Some("project_link_review_effects".to_owned()),
            requested_by: "test".to_owned(),
            metadata: json!({"requested_from": "project_link_review_projection_rebuild"}),
        })
        .await
        .expect("request project link review replay");
    assert_eq!(
        request.target_projection.as_deref(),
        Some("project_link_review_effects")
    );

    let report = replay_service
        .process_next_request()
        .await
        .expect("process project link review replay")
        .expect("project link review report");
    assert_eq!(report.replayed_count, 1);

    let rebuilt_relationship_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM relationships
        WHERE source_entity_kind = 'project'
          AND source_entity_id = 'project-replay-1'
          AND relationship_type = 'project_has_message'
          AND target_entity_id = 'message-replay-1'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("rebuilt project relationship count");
    assert_eq!(rebuilt_relationship_count, 1);

    let rebuilt_decision_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM decisions
        WHERE metadata ->> 'project_id' = 'project-replay-1'
          AND metadata ->> 'target_id' = 'message-replay-1'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("rebuilt project decision count");
    assert_eq!(rebuilt_decision_count, 1);
    assert!(
        consumer_store
            .has_processed_event(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, stored_event.position)
            .await
            .expect("processed marker after project link review replay")
    );

    let projection_events = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("projects.link_review_effects.updated")
                .correlation_id(request.id.clone())
                .limit(10),
        )
        .await
        .expect("query project link review projection update events");
    assert_eq!(projection_events.len(), 1);
    assert_eq!(
        projection_events[0].event.payload["target_projection"],
        "project_link_review_effects"
    );
    assert_eq!(projection_events[0].event.payload["replayed_count"], 1);
}

#[tokio::test]
async fn signal_hub_fixture_source_emits_deterministic_raw_signal_through_normal_consumer() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let event_store = EventStore::new(ctx.pool().clone());
    let fixture_service =
        SignalFixtureSourceService::new(signal_store.clone(), event_store.clone());

    let first = fixture_service
        .emit_fixture(&SignalFixtureEmitRequest {
            fixture_id: "fixture_basic_message".to_owned(),
        })
        .await
        .expect("emit fixture signal");
    let second = fixture_service
        .emit_fixture(&SignalFixtureEmitRequest {
            fixture_id: "fixture_basic_message".to_owned(),
        })
        .await
        .expect("emit fixture signal idempotently");

    assert_eq!(first, second);

    run_signal_hub_raw_consumer(ctx.pool().clone()).await;

    let raw = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.raw.fixture.message.observed")
                .source_code("fixture")
                .correlation_id("fixture-basic-message")
                .limit(10),
        )
        .await
        .expect("query raw fixture signal");
    assert_eq!(raw.len(), 1);
    assert_eq!(raw[0].event.event_id, first.raw_event_id);
    assert_eq!(
        raw[0].event.provenance["fixture_id"],
        "fixture_basic_message"
    );

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.fixture.message")
                .source_code("fixture")
                .correlation_id("fixture-basic-message")
                .limit(10),
        )
        .await
        .expect("query accepted fixture signal");
    assert_eq!(accepted.len(), 1);
    assert_eq!(
        accepted[0].event.causation_id.as_deref(),
        Some(first.raw_event_id.as_str())
    );
}

#[tokio::test]
async fn signal_hub_profile_application_sets_active_profile_and_managed_policies() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system fixture");
    let profile_service = SignalHubProfileService::new(
        signal_store.clone(),
        ApplicationSettingsStore::new(ctx.pool().clone()),
        EventStore::new(ctx.pool().clone()),
    );

    let applied = profile_service
        .apply_profile("testing")
        .await
        .expect("apply testing profile");
    assert_eq!(applied.code, "testing");
    assert!(applied.is_active);
    assert!(applied.policy_count >= 1);

    let profiles = profile_service
        .list_profiles()
        .await
        .expect("list profiles");
    assert_eq!(
        profiles.iter().filter(|profile| profile.is_active).count(),
        1
    );
    assert!(
        profiles
            .iter()
            .find(|profile| profile.code == "testing")
            .is_some_and(|profile| profile.is_active)
    );

    let active_policies = RawSignalStore::new(ctx.pool().clone())
        .list_active_policies()
        .await
        .expect("list active policies");
    assert!(active_policies.iter().any(|policy| {
        policy.source_code.as_deref() == Some("telegram")
            && matches!(policy.mode, SignalPolicyMode::Muted)
    }));

    let event_store = EventStore::new(ctx.pool().clone());
    let signal_service = SignalHubSignalService::new(
        Arc::new(RawSignalStore::new(ctx.pool().clone())),
        event_store.clone(),
    );
    let raw = NewEventEnvelope::builder(
        format!(
            "evt_profile_testing_{}",
            Utc::now().timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.telegram.message.observed",
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "telegram-profile-testing"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "telegram-profile-testing"
        }),
    )
    .payload(json!({"provider_message_id": "telegram-profile-testing"}))
    .correlation_id("corr-profile-testing")
    .build()
    .expect("valid raw signal");
    event_store
        .append_for_dispatch(&raw)
        .await
        .expect("append raw");
    let raw_event = event_store
        .get_by_id(&raw.event_id)
        .await
        .expect("load raw")
        .expect("raw exists");

    let outcome = signal_service
        .process_raw_signal(&raw_event)
        .await
        .expect("process raw under testing profile");
    assert_eq!(
        outcome,
        SignalProcessingOutcome::Muted {
            reason: "testing profile mutes Telegram signals".to_owned()
        }
    );
}

#[tokio::test]
async fn raw_signal_consumer_applies_global_pause_to_system_signals() {
    let ctx = TestContext::new().await;
    let signal_store = SignalHubStore::new(ctx.pool().clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    let event_store = EventStore::new(ctx.pool().clone());

    signal_store
        .create_policy(&SignalPolicy {
            scope: SignalPolicyScope::EventPattern,
            source_code: None,
            connection_id: None,
            event_pattern: Some("signal.raw.*".to_owned()),
            mode: SignalPolicyMode::Paused,
            reason: "global signal maintenance window".to_owned(),
            expires_at: None,
        })
        .await
        .expect("create global pause policy");

    let raw = NewEventEnvelope::builder(
        format!(
            "evt_signal_raw_system_{}",
            Utc::now().timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.system.health.observed",
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "system",
            "source_id": "system-health"
        }),
        json!({
            "source_code": "system",
            "entity_id": "system-health"
        }),
    )
    .payload(json!({"health": "ok"}))
    .build()
    .expect("valid system signal");
    event_store
        .append(&raw)
        .await
        .expect("append raw system signal");

    run_signal_hub_raw_consumer(ctx.pool().clone()).await;

    assert_eq!(
        signal_store
            .paused_event_count("system")
            .await
            .expect("paused system count"),
        1
    );

    let accepted = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.system.health")
                .source_code("system"),
        )
        .await
        .expect("query accepted system events");
    assert!(accepted.is_empty());
}

#[tokio::test]
async fn signal_hub_runtime_states_can_control_system_consumers() {
    let ctx = TestContext::new().await;
    let store = SignalHubStore::new(ctx.pool().clone());
    store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let initial = store
        .ensure_runtime_state(
            "system",
            "signal_hub_raw_signal_dispatcher",
            "running",
            json!({"scope": "consumer"}),
        )
        .await
        .expect("ensure runtime state");
    assert_eq!(initial.state, "running");

    let updated = store
        .set_runtime_state(&SignalRuntimeStateUpdate {
            source_code: "system".to_owned(),
            runtime_kind: "signal_hub_raw_signal_dispatcher".to_owned(),
            state: "paused".to_owned(),
            metadata: json!({"scope": "consumer", "requested_by": "test"}),
        })
        .await
        .expect("pause runtime state");
    assert_eq!(updated.state, "paused");
    assert_eq!(updated.metadata["requested_by"], "test");

    let listed = store
        .list_runtime_states()
        .await
        .expect("list runtime states");
    let runtime = listed
        .iter()
        .find(|item| item.runtime_kind == "signal_hub_raw_signal_dispatcher")
        .expect("stored runtime state");
    assert_eq!(runtime.source_code, "system");
    assert_eq!(runtime.state, "paused");

    let replay_runtime = store
        .set_runtime_state(&SignalRuntimeStateUpdate {
            source_code: "system".to_owned(),
            runtime_kind: "signal_replay_dispatcher".to_owned(),
            state: "paused".to_owned(),
            metadata: json!({"scope": "dispatcher", "requested_by": "test"}),
        })
        .await
        .expect("pause replay dispatcher runtime state");
    assert_eq!(replay_runtime.state, "paused");

    let listed = store
        .list_runtime_states()
        .await
        .expect("list runtime states after replay update");
    let replay_runtime = listed
        .iter()
        .find(|item| item.runtime_kind == "signal_replay_dispatcher")
        .expect("stored replay runtime state");
    assert_eq!(replay_runtime.source_code, "system");
    assert_eq!(replay_runtime.state, "paused");
}

#[tokio::test]
async fn paused_signal_hub_raw_dispatcher_blocks_sync_raw_signal_acceptance() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let signal_store = SignalHubStore::new(pool.clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "telegram-runtime-paused-account",
            CommunicationProviderKind::TelegramUser,
            "Telegram Runtime Paused",
            "telegram-runtime-paused-account",
        ))
        .await
        .expect("provider account");

    signal_store
        .set_runtime_state(&SignalRuntimeStateUpdate {
            source_code: "system".to_owned(),
            runtime_kind: SIGNAL_HUB_RAW_SIGNAL_CONSUMER.to_owned(),
            state: "paused".to_owned(),
            metadata: json!({"scope": "consumer", "requested_by": "test"}),
        })
        .await
        .expect("pause signal hub raw dispatcher");

    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw_signal_hub_runtime_paused_telegram",
                "telegram-runtime-paused-account",
                "telegram_message",
                "telegram-runtime-paused-message-1",
                "sha256:signal-hub:runtime-paused:telegram",
                "signal-hub-runtime-paused",
                json!({
                    "provider_chat_id": "telegram-runtime-paused-chat",
                    "chat_title": "Telegram Runtime Paused Chat",
                    "sender_id": "telegram-runtime-paused-sender",
                    "sender_display_name": "Paused Sender",
                    "text": "hold this signal",
                    "delivery_state": "received"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": "telegram-runtime-paused-account",
                "provider_chat_id": "telegram-runtime-paused-chat",
            })),
        )
        .await
        .expect("raw telegram record");

    let accepted = dispatch_telegram_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch raw telegram signal while paused");
    assert!(accepted.is_none());

    let event_store = EventStore::new(pool.clone());
    let raw_events = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.raw.telegram.message.observed")
                .correlation_id(raw_record.observation_id.clone())
                .limit(10),
        )
        .await
        .expect("query raw telegram signal events");
    assert_eq!(raw_events.len(), 1);

    let accepted_events = event_store
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .correlation_id(raw_record.observation_id.clone())
                .limit(10),
        )
        .await
        .expect("query accepted telegram signal events");
    assert!(accepted_events.is_empty());
}

#[tokio::test]
async fn paused_accepted_signal_consumer_blocks_sync_projection_of_accepted_signal() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let signal_store = SignalHubStore::new(pool.clone());
    signal_store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            "telegram-accepted-paused-account",
            CommunicationProviderKind::TelegramUser,
            "Telegram Accepted Paused",
            "telegram-accepted-paused-account",
        ))
        .await
        .expect("provider account");

    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                "raw_signal_hub_accepted_paused_telegram",
                "telegram-accepted-paused-account",
                "telegram_message",
                "telegram-accepted-paused-message-1",
                "sha256:signal-hub:accepted-paused:telegram",
                "signal-hub-accepted-paused",
                json!({
                    "provider_chat_id": "telegram-accepted-paused-chat",
                    "chat_title": "Telegram Accepted Paused Chat",
                    "sender_id": "telegram-accepted-paused-sender",
                    "sender_display_name": "Paused Accepted Sender",
                    "text": "hold accepted projection",
                    "delivery_state": "received"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": "telegram-accepted-paused-account",
                "provider_chat_id": "telegram-accepted-paused-chat",
            })),
        )
        .await
        .expect("raw telegram record");

    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch raw telegram signal")
        .expect("accepted telegram signal");

    signal_store
        .set_runtime_state(&SignalRuntimeStateUpdate {
            source_code: "system".to_owned(),
            runtime_kind: COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER.to_owned(),
            state: "paused".to_owned(),
            metadata: json!({"scope": "consumer", "requested_by": "test"}),
        })
        .await
        .expect("pause accepted-signal consumer");

    let projected = project_accepted_signal_if_runtime_allows(pool.clone(), &accepted_event)
        .await
        .expect("project accepted signal with runtime gate");
    assert!(projected.is_none());

    let projected_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM communication_messages WHERE raw_record_id = $1")
            .bind(&raw_record.raw_record_id)
            .fetch_one(&pool)
            .await
            .expect("projected message count");
    assert_eq!(projected_count, 0);

    let accepted_events = EventStore::new(pool.clone())
        .list_matching(
            EventLogQuery::default()
                .event_type("signal.accepted.telegram.message")
                .correlation_id(raw_record.observation_id.clone())
                .limit(10),
        )
        .await
        .expect("query accepted telegram signal events");
    assert_eq!(accepted_events.len(), 1);
}

#[tokio::test]
async fn signal_hub_source_disable_and_enable_orchestrate_source_runtime_states() {
    let ctx = TestContext::new().await;
    let store = SignalHubStore::new(ctx.pool().clone());
    store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    store
        .ensure_runtime_state(
            "system",
            "signal_hub_raw_signal_dispatcher",
            "running",
            json!({"scope": "consumer"}),
        )
        .await
        .expect("ensure system consumer runtime");
    store
        .ensure_runtime_state(
            "system",
            "event_outbox_dispatcher",
            "running",
            json!({"scope": "dispatcher"}),
        )
        .await
        .expect("ensure system dispatcher runtime");

    let controls = SignalHubControlService::new(store.clone(), EventStore::new(ctx.pool().clone()));

    controls
        .disable_source("system", Some("maintenance window"))
        .await
        .expect("disable system source");

    let disabled = store
        .list_runtime_states()
        .await
        .expect("list disabled runtimes");
    let system_states: Vec<_> = disabled
        .iter()
        .filter(|item| item.source_code == "system")
        .map(|item| item.state.as_str())
        .collect();
    assert!(system_states.iter().all(|state| *state == "stopped"));

    controls
        .enable_source("system", Some("resume maintenance"))
        .await
        .expect("enable system source");

    let reenabled = store
        .list_runtime_states()
        .await
        .expect("list reenabled runtimes");
    let system_states: Vec<_> = reenabled
        .iter()
        .filter(|item| item.source_code == "system")
        .map(|item| item.state.as_str())
        .collect();
    assert!(system_states.iter().all(|state| *state == "running"));
}

#[tokio::test]
async fn disabled_source_blocks_lazy_runtime_creation_for_new_runtime_kind() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = SignalHubStore::new(pool.clone());
    store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let controls = SignalHubControlService::new(store.clone(), EventStore::new(pool.clone()));
    controls
        .disable_source(
            "telegram",
            Some("operator disabled source before runtime boot"),
        )
        .await
        .expect("disable telegram source");

    let allows = runtime_allows_processing(
        &pool,
        "telegram",
        "telegram_runtime_event_bridge",
        &json!({
            "label": "Telegram realtime event bridge",
            "scope": "subscription",
        }),
    )
    .await
    .expect("runtime gate result");
    assert!(!allows);

    let runtime = store
        .runtime_state("telegram", "telegram_runtime_event_bridge")
        .await
        .expect("load lazy runtime row")
        .expect("lazy runtime row exists");
    assert_eq!(runtime.state, "stopped");
}

#[tokio::test]
async fn source_pause_and_mute_controls_reconcile_existing_and_lazy_runtime_rows_by_priority() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = SignalHubStore::new(pool.clone());
    store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let controls = SignalHubControlService::new(store.clone(), EventStore::new(pool.clone()));
    let request = SignalHubControlRequest {
        scope: SignalPolicyScope::Source,
        source_code: Some("telegram".to_owned()),
        connection_id: None,
        event_pattern: None,
        reason: "operator toggled telegram runtime".to_owned(),
    };

    controls
        .pause_signals(&request)
        .await
        .expect("pause telegram source");

    let paused_allows = runtime_allows_processing(
        &pool,
        "telegram",
        "telegram_runtime_event_bridge",
        &json!({
            "label": "Telegram realtime event bridge",
            "scope": "subscription",
        }),
    )
    .await
    .expect("paused runtime gate result");
    assert!(!paused_allows);

    let paused_runtime = store
        .runtime_state("telegram", "telegram_runtime_event_bridge")
        .await
        .expect("load paused runtime row")
        .expect("paused runtime row exists");
    assert_eq!(paused_runtime.state, "paused");

    controls
        .mute_signals(&request)
        .await
        .expect("mute telegram source");
    let still_paused = store
        .runtime_state("telegram", "telegram_runtime_event_bridge")
        .await
        .expect("load runtime after mute")
        .expect("runtime after mute exists");
    assert_eq!(still_paused.state, "paused");

    controls
        .resume_signals(&request)
        .await
        .expect("resume telegram source");
    let muted_runtime = store
        .runtime_state("telegram", "telegram_runtime_event_bridge")
        .await
        .expect("load runtime after resume")
        .expect("runtime after resume exists");
    assert_eq!(muted_runtime.state, "muted");

    controls
        .unmute_signals(&request)
        .await
        .expect("unmute telegram source");
    let running_runtime = store
        .runtime_state("telegram", "telegram_runtime_event_bridge")
        .await
        .expect("load runtime after unmute")
        .expect("runtime after unmute exists");
    assert_eq!(running_runtime.state, "running");
}

#[tokio::test]
async fn event_pattern_disable_controls_can_be_cleared_without_source_specific_api() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let store = SignalHubStore::new(pool.clone());
    store
        .restore_system_sources()
        .await
        .expect("restore system sources");

    let controls = SignalHubControlService::new(store.clone(), EventStore::new(pool.clone()));
    let request = SignalHubControlRequest {
        scope: SignalPolicyScope::EventPattern,
        source_code: None,
        connection_id: None,
        event_pattern: Some("signal.raw.telegram.*".to_owned()),
        reason: "operator disabled telegram raw pattern".to_owned(),
    };

    let disabled = controls
        .disable_signals(&request)
        .await
        .expect("disable event pattern");
    assert!(disabled.policy_id.is_some());

    let policies = RawSignalStore::new(ctx.pool().clone())
        .list_active_policies()
        .await
        .expect("list disabled policies");
    assert!(policies.iter().any(|policy| {
        policy.scope == SignalPolicyScope::EventPattern
            && policy.mode == SignalPolicyMode::Disabled
            && policy.event_pattern.as_deref() == Some("signal.raw.telegram.*")
    }));

    let enabled = controls
        .enable_signals(&request)
        .await
        .expect("enable event pattern");
    assert_eq!(enabled.cleared_count, 1);

    let remaining = RawSignalStore::new(ctx.pool().clone())
        .list_active_policies()
        .await
        .expect("list remaining policies");
    assert!(!remaining.iter().any(|policy| {
        policy.scope == SignalPolicyScope::EventPattern
            && policy.mode == SignalPolicyMode::Disabled
            && policy.event_pattern.as_deref() == Some("signal.raw.telegram.*")
    }));
}

#[tokio::test]
async fn signal_hub_health_check_updates_durable_health_state() {
    let ctx = TestContext::new().await;
    let store = SignalHubStore::new(ctx.pool().clone());
    store
        .restore_system_sources()
        .await
        .expect("restore system sources");
    store
        .ensure_runtime_state(
            "system",
            "signal_hub_raw_signal_dispatcher",
            "running",
            json!({"scope": "consumer"}),
        )
        .await
        .expect("ensure runtime state");

    let health_service =
        SignalHubHealthService::new(store.clone(), EventStore::new(ctx.pool().clone()));
    let health = health_service
        .run_health_check(&SignalHealthCheckRequest {
            source_code: "system".to_owned(),
            connection_id: None,
            runtime_kind: Some("signal_hub_raw_signal_dispatcher".to_owned()),
        })
        .await
        .expect("run health check");

    assert_eq!(health.source_code, "system");
    assert_eq!(health.level, "healthy");
    assert!(health.summary.contains("healthy"));

    let listed = store.list_health().await.expect("list health");
    assert!(
        listed
            .iter()
            .any(|item| item.id == health.id && item.level == "healthy")
    );
}

async fn run_signal_hub_raw_consumer(pool: sqlx::postgres::PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(SIGNAL_HUB_RAW_SIGNAL_CONSUMER),
    );

    for _ in 0..10 {
        let handler_pool = pool.clone();
        let report = runner
            .process_next_batch(|event| process_signal_hub_raw_event(handler_pool.clone(), event))
            .await
            .expect("signal hub raw consumer");
        if report.processed == 0 {
            break;
        }
    }
}
