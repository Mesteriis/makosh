use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::{NewEventEnvelope, StoredEventEnvelope};
use makosh_events_postgres::consumers::EventConsumerConfig;
use makosh_events_postgres::consumers::EventConsumerRunner;
use makosh_events_postgres::consumers::EventDeadLetterReviewState;
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, consume_accepted_signal_event,
    project_provider_observation_event,
};
use makosh_hub_backend::domains::signal_hub::service::{
    SIGNAL_HUB_RAW_SIGNAL_CONSUMER, process_signal_hub_raw_event,
};
use makosh_hub_backend::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use makosh_hub_backend::integrations::telegram::client::models::chats::TelegramChatKind;
use makosh_hub_backend::integrations::telegram::client::models::messages::{
    NewTelegramMessage, TelegramDeliveryState, TelegramMessage,
};
use makosh_hub_backend::integrations::telegram::client::store::TelegramStore;

use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::provider_messages::ProviderMessageObservationEvent;
use makosh_hub_backend::platform::communications::{
    EventStoreProviderMessageObservationEventPort, ProviderMessageObservationEventPort,
};

async fn live_context(_test_name: &str) -> Option<(TestContext, EventStore)> {
    let test_context = TestContext::new().await;
    let store = EventStore::new(test_context.pool().clone());
    Some((test_context, store))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn consumer_config(name: String, max_attempts: i32) -> EventConsumerConfig {
    EventConsumerConfig {
        consumer_name: name,
        batch_size: 1,
        max_attempts,
        retry_base_seconds: 0,
    }
}

async fn append_test_event(store: &EventStore, suffix: u128, marker: &str) -> i64 {
    let event_id = format!("evt_consumer_{marker}_{suffix}");
    let event = NewEventEnvelope::builder(
        &event_id,
        "system.consumer_test_event",
        Utc::now(),
        json!({
            "kind": "test",
            "provider": "event-consumers",
            "source_id": event_id
        }),
        json!({"kind": "system", "entity_id": "event-consumer-test"}),
    )
    .payload(json!({"marker": marker}))
    .build()
    .expect("valid event");

    store.append(&event).await.expect("append test event")
}

#[tokio::test]
async fn consumer_cursor_does_not_advance_before_success_against_postgres() {
    let Some((context, store)) = live_context("event consumer cursor").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "cursor").await;
    let pool = context.pool().clone();
    let consumer_name = format!("consumer_cursor_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 3));
    let starting_cursor = position - 1;
    runner
        .store()
        .save_position(&consumer_name, starting_cursor)
        .await
        .expect("place cursor before test event");

    let failed = runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "transient failure".to_owned(),
            ))
        })
        .await
        .expect("run failed handler");

    assert_eq!(failed.failed, 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after failure"),
        starting_cursor
    );
    assert_eq!(
        runner
            .store()
            .failure_attempt_count(&consumer_name, position)
            .await
            .expect("failure attempt count"),
        Some(1)
    );

    let succeeded = runner
        .process_next_batch(|_| async { Ok(()) })
        .await
        .expect("run successful handler");

    assert_eq!(succeeded.processed, 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after success"),
        position
    );
    assert_eq!(
        runner
            .store()
            .failure_attempt_count(&consumer_name, position)
            .await
            .expect("failure removed"),
        None
    );
}

#[tokio::test]
async fn consumer_retries_then_dead_letters_after_max_attempts_against_postgres() {
    let Some((context, store)) = live_context("event consumer DLQ").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "dlq").await;
    let pool = context.pool().clone();
    let consumer_name = format!("consumer_dlq_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 2));
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place cursor before test event");

    let first = runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "first failure".to_owned(),
            ))
        })
        .await
        .expect("first failure");

    assert_eq!(first.failed, 1);
    assert_eq!(first.dead_lettered, 0);
    assert_eq!(
        runner
            .store()
            .failure_attempt_count(&consumer_name, position)
            .await
            .expect("first attempt count"),
        Some(1)
    );

    let second = runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "second failure".to_owned(),
            ))
        })
        .await
        .expect("second failure");

    assert_eq!(second.failed, 1);
    assert_eq!(second.dead_lettered, 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after DLQ"),
        position
    );

    let dead_letter = runner
        .store()
        .dead_letter_for_event(&consumer_name, position)
        .await
        .expect("load dead letter")
        .expect("dead letter exists");

    assert_eq!(dead_letter.attempts, 2);
    assert_eq!(dead_letter.review_state, EventDeadLetterReviewState::Open);
    assert_eq!(dead_letter.event.position, position);
}

#[tokio::test]
async fn dead_letter_replay_marks_event_replayed_against_postgres() {
    let Some((context, store)) = live_context("event consumer DLQ replay").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "replay").await;
    let pool = context.pool().clone();
    let consumer_name = format!("consumer_replay_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 1));
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place cursor before test event");

    runner
        .process_next_batch(|_| async {
            Err(EventStoreError::ConsumerHandlerFailed(
                "poison event".to_owned(),
            ))
        })
        .await
        .expect("dead letter event");

    let dead_letter = runner
        .store()
        .dead_letter_for_event(&consumer_name, position)
        .await
        .expect("load dead letter")
        .expect("dead letter exists");
    runner
        .store()
        .request_dead_letter_replay(&dead_letter.dead_letter_id)
        .await
        .expect("request replay");

    runner
        .replay_dead_letter(&dead_letter.dead_letter_id, |event| async move {
            assert_eq!(event.position, position);
            Ok(())
        })
        .await
        .expect("replay dead letter");

    let replayed = runner
        .store()
        .dead_letter_by_id(&dead_letter.dead_letter_id)
        .await
        .expect("load replayed dead letter");
    assert_eq!(replayed.review_state, EventDeadLetterReviewState::Replayed);
}

#[tokio::test]
async fn duplicate_consumer_event_delivery_is_idempotent_against_postgres() {
    let Some((context, store)) = live_context("event consumer idempotency").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "idempotent").await;
    let pool = context.pool().clone();
    let consumer_name = format!("consumer_idempotent_{suffix}");
    let runner = EventConsumerRunner::new(pool, consumer_config(consumer_name.clone(), 3));
    runner
        .store()
        .save_position(&consumer_name, position - 1)
        .await
        .expect("place cursor before test event");
    let call_count = Arc::new(AtomicUsize::new(0));

    let first_count = Arc::clone(&call_count);
    runner
        .process_next_batch(move |_| {
            let first_count = Arc::clone(&first_count);
            async move {
                first_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("first processing");

    let second_count = Arc::clone(&call_count);
    runner
        .process_next_batch(move |_| {
            let second_count = Arc::clone(&second_count);
            async move {
                second_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("second processing");

    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        runner
            .store()
            .last_processed_position(&consumer_name)
            .await
            .expect("cursor after idempotent processing"),
        position
    );

    assert_eq!(
        runner
            .store()
            .processed_event_count(&consumer_name, position)
            .await
            .expect("processed marker count"),
        1
    );

    sqlx::query(
        r#"
        UPDATE event_consumers
        SET last_processed_position = $2, updated_at = now()
        WHERE consumer_name = $1
        "#,
    )
    .bind(&consumer_name)
    .bind(position - 1)
    .execute(context.pool())
    .await
    .expect("rewind consumer cursor");

    let duplicate_count = Arc::clone(&call_count);
    let duplicate = runner
        .process_next_batch(move |_| {
            let duplicate_count = Arc::clone(&duplicate_count);
            async move {
                duplicate_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await
        .expect("duplicate delivery");

    assert_eq!(duplicate.skipped_duplicates, 1);
    assert_eq!(duplicate.processed, 0);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        runner
            .store()
            .processed_event_count(&consumer_name, position)
            .await
            .expect("processed marker still single"),
        1
    );
}

#[tokio::test]
async fn processed_event_identity_is_idempotent_across_distinct_positions_against_postgres() {
    let Some((context, store)) = live_context("event consumer event identity").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "identity").await;
    let consumer_name = format!("consumer_event_identity_{suffix}");
    let consumer_store = EventConsumerRunner::new(
        context.pool().clone(),
        consumer_config(consumer_name.clone(), 3),
    )
    .store()
    .clone();
    let stored = store
        .list_after_position(position - 1, 1)
        .await
        .expect("stored test event")
        .into_iter()
        .next()
        .expect("event exists");

    assert!(
        consumer_store
            .record_processed(&consumer_name, &stored)
            .await
            .expect("first processed marker")
    );
    let duplicate = StoredEventEnvelope {
        position: stored.position + 1_000_000,
        event: stored.event.clone(),
    };
    assert!(
        !consumer_store
            .record_processed(&consumer_name, &duplicate)
            .await
            .expect("event-id duplicate must be ignored")
    );
    assert!(
        consumer_store
            .has_processed_event_id(&consumer_name, &stored.event.event_id)
            .await
            .expect("processed event ID")
    );
    assert_eq!(
        consumer_store
            .processed_event_count(&consumer_name, stored.position)
            .await
            .expect("processed marker count"),
        1
    );
}

#[tokio::test]
async fn provider_observation_events_are_emitted_with_required_telegram_event_types_against_postgres()
 {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let message = create_projected_telegram_message(&pool, "event-types").await;
    let event_port = EventStoreProviderMessageObservationEventPort::new(pool.clone());
    let observed_at = Utc::now();

    let observations = [
        (
            "content_observed",
            None,
            json!({
                "body_text": "event type content",
                "message_metadata": {"event_type_test": "content"},
                "observed_at": observed_at,
            }),
        ),
        (
            "metadata_observed",
            None,
            json!({"message_metadata": {"event_type_test": "metadata"}}),
        ),
        (
            "delivery_state_observed",
            None,
            json!({"delivery_state": "read", "observed_at": observed_at}),
        ),
        (
            "pinned_state_observed",
            None,
            json!({"is_pinned": true, "observed_at": observed_at}),
        ),
        (
            "attachment_download_state_observed",
            None,
            json!({
                "provider_attachment_id": "att-event-types",
                "provider_file_id": 42,
                "download_state": "downloaded",
                "local_path": "docker/data/telegram/att-event-types.bin",
                "size_bytes": 12,
                "content_type": "application/octet-stream",
                "filename": "att.bin",
                "observed_at": observed_at,
            }),
        ),
    ];

    for (event_kind, external_event_id, payload) in observations {
        append_provider_observation(
            &event_port,
            &message,
            event_kind,
            external_event_id,
            observed_at,
            &payload,
        )
        .await
        .expect("append provider observation");
    }

    let event_types = sqlx::query_scalar::<_, String>(
        r#"
        SELECT event_type
        FROM event_log
        WHERE source->>'kind' = 'provider_observation'
          AND source->>'account_id' = $1
        ORDER BY event_type ASC
        "#,
    )
    .bind(&message.account_id)
    .fetch_all(&pool)
    .await
    .expect("provider observation event types");

    assert!(event_types.contains(&"signal.raw.telegram.message.content.observed".to_owned()));
    assert!(event_types.contains(&"signal.raw.telegram.message.metadata.observed".to_owned()));
    assert!(
        event_types.contains(&"signal.raw.telegram.message.delivery_state.observed".to_owned())
    );
    assert!(event_types.contains(&"signal.raw.telegram.message.pinned_state.observed".to_owned()));
    assert!(
        event_types.contains(&"signal.raw.telegram.attachment.download_state.observed".to_owned())
    );

    let outbox_event_types = sqlx::query_scalar::<_, String>(
        r#"
        SELECT event_log.event_type
        FROM event_outbox
        JOIN event_log ON event_log.event_id = event_outbox.event_id
        WHERE event_log.source->>'kind' = 'provider_observation'
          AND event_log.source->>'account_id' = $1
        ORDER BY event_log.event_type ASC
        "#,
    )
    .bind(&message.account_id)
    .fetch_all(&pool)
    .await
    .expect("provider observation outbox event types");

    assert_eq!(outbox_event_types, event_types);
}

#[tokio::test]
async fn communication_provider_observation_projection_is_idempotent_against_postgres() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let message = create_projected_telegram_message(&pool, "projection-idempotent").await;
    let event_port = EventStoreProviderMessageObservationEventPort::new(pool.clone());
    let observed_at = Utc::now();
    let payload = json!({"message_metadata": {"projection_marker": "external-event"}});

    let first_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        Some("provider-event-1"),
        observed_at,
        &payload,
    )
    .await
    .expect("first provider observation")
    .expect("first append position");
    let duplicate_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        Some("provider-event-1"),
        observed_at,
        &payload,
    )
    .await
    .expect("duplicate provider observation");
    assert_eq!(duplicate_position, None);

    run_signal_hub_raw_consumer(pool.clone(), first_position - 1).await;
    let accepted_position = accepted_position_for_raw_event(&pool, first_position).await;

    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig {
            consumer_name: COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER.to_owned(),
            batch_size: 10,
            max_attempts: 3,
            retry_base_seconds: 0,
        },
    );
    runner
        .store()
        .save_position(
            COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
            accepted_position - 1,
        )
        .await
        .expect("place consumer before accepted provider event");

    let first_run = runner
        .process_next_batch(|event| project_provider_observation_event(pool.clone(), event))
        .await
        .expect("project provider observation");
    assert_eq!(first_run.processed, 1);

    let projected = ProviderChannelMessageStore::new(pool.clone())
        .message_by_id(&message.message_id, &["telegram_user", "telegram_bot"])
        .await
        .expect("load projected message")
        .expect("projected message exists");
    assert_eq!(
        projected.message_metadata["projection_marker"],
        json!("external-event")
    );

    sqlx::query(
        r#"
        UPDATE event_consumers
        SET last_processed_position = $2, updated_at = now()
        WHERE consumer_name = $1
        "#,
    )
    .bind(COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER)
    .bind(first_position - 1)
    .execute(&pool)
    .await
    .expect("rewind projection consumer cursor");

    let replay = runner
        .process_next_batch(|event| project_provider_observation_event(pool.clone(), event))
        .await
        .expect("replay provider observation");
    assert_eq!(replay.skipped_duplicates, 1);

    let communication_update_events = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'communication.message.updated'
          AND causation_id = (
              SELECT event_id
              FROM event_log
              WHERE position = $1
          )
        "#,
    )
    .bind(accepted_position)
    .fetch_one(&pool)
    .await
    .expect("communication update event count");
    assert_eq!(communication_update_events, 1);
}

#[tokio::test]
async fn communication_provider_observation_projection_consumes_accepted_telegram_message_against_postgres()
 {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let unique = unique_suffix();
    let suffix = format!("accepted-base-{unique}");
    let account_id = format!("acct-{suffix}");
    let account = NewProviderAccount::new(
        account_id.clone(),
        CommunicationProviderKind::TelegramUser,
        format!("Telegram {suffix}"),
        format!("telegram:{suffix}"),
    )
    .config(json!({"runtime": "fixture"}));
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&account)
        .await
        .expect("provider account");

    let store = TelegramStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(
            makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(
            makosh_communications_postgres::store::CommunicationIngestionStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(EventStoreProviderMessageObservationEventPort::new(pool.clone())),
    );
    let provider_chat_id = format!("-100{suffix}");
    let provider_message_id = format!("{provider_chat_id}:1");
    let observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: format!("Chat {suffix}"),
            sender_id: "user:1".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "accepted base message".to_owned(),
            import_batch_id: format!("batch-{suffix}"),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture");
    let stored_raw = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(&observed.raw)
        .await
        .expect("store raw record");
    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &stored_raw)
        .await
        .expect("dispatch raw telegram signal")
        .expect("accepted telegram signal");
    let accepted_position = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT position
        FROM event_log
        WHERE event_id = $1
        "#,
    )
    .bind(&accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("accepted telegram event position");

    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig {
            consumer_name: COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER.to_owned(),
            batch_size: 10,
            max_attempts: 3,
            retry_base_seconds: 0,
        },
    );
    runner
        .store()
        .save_position(
            COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
            accepted_position - 1,
        )
        .await
        .expect("place consumer before accepted telegram event");

    let report = runner
        .process_next_batch(|event| project_provider_observation_event(pool.clone(), event))
        .await
        .expect("project accepted telegram base event");
    assert_eq!(report.processed, 1);

    let projected_message_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT message_id
        FROM communication_messages
        WHERE raw_record_id = $1
        "#,
    )
    .bind(&stored_raw.raw_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected telegram message id");
    let projected = store
        .message_by_id(&projected_message_id)
        .await
        .expect("load projected telegram message")
        .expect("projected telegram message exists");
    assert_eq!(projected.provider_message_id, provider_message_id);

    let communication_recorded_events = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'communication.message.recorded'
          AND causation_id = $1
        "#,
    )
    .bind(&accepted_event.event_id)
    .fetch_one(&pool)
    .await
    .expect("communication recorded event count");
    assert_eq!(communication_recorded_events, 1);
}

async fn run_signal_hub_raw_consumer(pool: sqlx::PgPool, cursor: i64) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(SIGNAL_HUB_RAW_SIGNAL_CONSUMER),
    );
    runner
        .store()
        .save_position(SIGNAL_HUB_RAW_SIGNAL_CONSUMER, cursor)
        .await
        .expect("place signal hub consumer before raw event");

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

async fn accepted_position_for_raw_event(pool: &sqlx::PgPool, raw_position: i64) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT position
        FROM event_log
        WHERE causation_id = (
            SELECT event_id
            FROM event_log
            WHERE position = $1
        )
          AND event_type LIKE 'signal.accepted.telegram.%'
        ORDER BY position ASC
        LIMIT 1
        "#,
    )
    .bind(raw_position)
    .fetch_one(pool)
    .await
    .expect("accepted telegram event position")
}

#[tokio::test]
async fn provider_observation_fallback_idempotency_uses_payload_hash_against_postgres() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let message = create_projected_telegram_message(&pool, "fallback-idempotent").await;
    let event_port = EventStoreProviderMessageObservationEventPort::new(pool);
    let observed_at = Utc::now();
    let payload = json!({"message_metadata": {"projection_marker": "fallback"}});

    let first_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        None,
        observed_at,
        &payload,
    )
    .await
    .expect("first fallback observation");
    let duplicate_position = append_provider_observation(
        &event_port,
        &message,
        "metadata_observed",
        None,
        observed_at,
        &payload,
    )
    .await
    .expect("duplicate fallback observation");

    assert!(first_position.is_some());
    assert_eq!(duplicate_position, None);
}

async fn create_projected_telegram_message(pool: &sqlx::PgPool, suffix: &str) -> TelegramMessage {
    let unique = unique_suffix();
    let account_id = format!("acct-{suffix}-{unique}");
    let account = NewProviderAccount::new(
        account_id.clone(),
        CommunicationProviderKind::TelegramUser,
        format!("Telegram {suffix}"),
        format!("telegram:{suffix}:{unique}"),
    )
    .config(json!({"runtime": "fixture"}));
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&account)
        .await
        .expect("provider account");

    let store = TelegramStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(
            makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(
            makosh_communications_postgres::store::CommunicationIngestionStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(EventStoreProviderMessageObservationEventPort::new(pool.clone())),
    );
    let provider_chat_id = format!("-100{suffix}{unique}");
    let provider_message_id = format!("{provider_chat_id}:1");
    let observed = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.clone(),
            provider_chat_id: provider_chat_id.clone(),
            provider_message_id: provider_message_id.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: format!("Chat {suffix}"),
            sender_id: "user:1".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: format!("batch-{suffix}-{unique}"),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture");
    let stored_raw = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(&observed.raw)
        .await
        .expect("stored raw fixture");
    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &stored_raw)
        .await
        .expect("dispatch raw signal")
        .expect("accepted signal");
    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("accepted projection")
        .expect("projected message");
    store
        .message_by_id(&projected.message_id)
        .await
        .expect("load projected message")
        .expect("projected message exists")
}

async fn append_provider_observation(
    event_port: &EventStoreProviderMessageObservationEventPort,
    message: &TelegramMessage,
    event_kind: &str,
    external_event_id: Option<&str>,
    observed_at: chrono::DateTime<Utc>,
    payload: &serde_json::Value,
) -> Result<
    Option<i64>,
    makosh_hub_backend::platform::communications::errors::ProviderCommunicationMessagePortError,
> {
    event_port
        .append_provider_message_observation(ProviderMessageObservationEvent {
            provider: "telegram",
            account_id: &message.account_id,
            channel_kind: &message.channel_kind,
            message_id: &message.message_id,
            external_message_id: &message.provider_message_id,
            event_kind,
            observed_at,
            external_event_id,
            payload,
            causation_id: Some("event-consumer-test"),
            correlation_id: Some("event-consumer-test"),
        })
        .await
}
