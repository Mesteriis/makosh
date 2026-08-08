# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `083-test-backend-part-006`
- Group / Группа: `backend`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/backend-tests.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/tests/email_sync_pipeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/email_sync_pipeline.rs`
- Size bytes / Размер в байтах: `31207`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use base64::Engine as _;
use chrono::{TimeZone, Utc};
use serde_json::json;
use sqlx::Row;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::domains::communications::storage::LocalCommunicationBlobStore;
use makosh_hub_backend::integrations::mail::sync::{
    EmailSyncBatch, FetchedCommunicationSourceMessage,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::email_sync_pipeline::project_email_sync_batch_with_mail_blobs;

#[tokio::test]
async fn email_sync_pipeline_records_raw_blob_and_projects_message_persons_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_sync_pipeline_{suffix}");
    let provider_record_id = format!("sync-pipeline-message-{suffix}");
    let sender_domain = format!("acme-{suffix}.test");
    let recipient_domain = format!("client-{suffix}.test");
    let sender_email = format!("sender-{suffix}@{sender_domain}");
    let recipient_email = format!("recipient-{suffix}@{recipient_domain}");
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalCommunicationBlobStore::new(blob_root.path());

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Sync pipeline IMAP",
            format!("sync-pipeline-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");

    let raw_rfc822 = format!(
        "Subject: Sync Pipeline\r\n\
         From: Sender <{sender_email}>\r\n\
         To: Recipient <{recipient_email}>\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         Cached message body.\r\n"
    );
    let raw_rfc822_base64 = base64::engine::general_purpose::STANDARD.encode(raw_rfc822);
    let batch = EmailSyncBatch {
        provider_kind: EmailProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 88})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: provider_record_id.clone(),
            source_fingerprint: format!("sha256:sync-pipeline-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 88,
                "raw_rfc822_base64": raw_rfc822_base64
            }),
        }],
    };

    let report = project_email_sync_batch_with_mail_blobs(
        pool.clone(),
        &blob_store,
        &account_id,
        format!("sync-pipeline-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("project email sync batch");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.raw_blobs_upserted, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.attachment_blobs_upserted, 0);
    assert_eq!(report.attachments_extracted, 0);
    assert_eq!(report.attachments_not_scanned, 0);
    assert_eq!(report.upserted_persons, 2);
    assert_eq!(report.upserted_person_identities, 2);
    assert_eq!(report.upserted_message_participants, 2);
    assert_eq!(report.upserted_relationship_events, 2);
    assert_eq!(report.upserted_organizations, 2);
    assert_eq!(report.upserted_organization_contact_links, 2);

    let projected = sqlx::query(
        r#"
        SELECT message_id, observation_id, subject, sender, recipients, body_text
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected message");
    let message_id: String = projected.try_get("message_id").expect("message id");
    let observation_id: String = projected.try_get("observation_id").expect("observation id");
    let subject: String = projected.try_get("subject").expect("subject");
    let sender: String = projected.try_get("sender").expect("sender");
    let recipients: serde_json::Value = projected.try_get("recipients").expect("recipients");
    let body_text: String = projected.try_get("body_text").expect("body_text");
    assert_eq!(subject, "Sync Pipeline");
    assert_eq!(sender, format!("Sender <{sender_email}>"));
    assert_eq!(body_text, "Cached message body.");
    assert_eq!(
        recipients,
        json!([format!("Recipient <{recipient_email}>")])
    );

    let accepted_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.accepted.mail.message'
          AND source ->> 'account_id' = $1
          AND subject ->> 'provider_record_id' = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("accepted mail signal count");
    assert_eq!(accepted_signal_count, 1);

    let identity_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM person_identities
        WHERE identity_type = 'email'
          AND identity_value = ANY($1)
          AND source = 'email_sync'
          AND status = 'active'
        "#,
    )
    .bind(vec![sender_email.as_str(), recipient_email.as_str()])
    .fetch_one(&pool)
    .await
    .expect("person email identities");
    assert_eq!(identity_count, 2);

    let persona_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'persons'
          AND entity_kind = 'persona'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("persona observation links");
    assert_eq!(persona_observation_link_count, 2);

    let identity_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'persons'
          AND entity_kind = 'identity'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("identity observation links");
    assert_eq!(identity_observation_link_count, 2);

    let participant_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM communication_message_participants
        WHERE message_id = (
            SELECT message_id
            FROM communication_messages
            WHERE account_id = $1 AND provider_record_id = $2
        )
          AND email_address = ANY($3)
          AND role = ANY($4)
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .bind(vec![sender_email.as_str(), recipient_email.as_str()])
    .bind(vec!["sender", "recipient"])
    .fetch_one(&pool)
    .await
    .expect("message participants");
    assert_eq!(participant_count, 2);

    let participant_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'communications'
          AND entity_kind = 'message_participant'
          AND relationship_kind = 'email_sync_participant'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("message participant observation links");
    assert_eq!(participant_observation_link_count, 2);

    let relationship_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM relationship_events
        WHERE related_entity_kind = 'communication_message'
          AND related_entity_id = (
            SELECT message_id
            FROM communication_messages
            WHERE account_id = $1 AND provider_record_id = $2
          )
          AND event_type IN ('email_sent', 'email_received')
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("relationship events");
    assert_eq!(relationship_count, 2);

    let relationship_event_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'persons'
          AND entity_kind = 'relationship_event'
          AND relationship_kind = 'email_sync_relationship_event'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("relationship event observation links");
    assert_eq!(relationship_event_observation_link_count, 2);

    let organization_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM organization_contact_links link
        JOIN organization_domains domain ON domain.organization_id = link.organization_id
        JOIN person_identities identity ON identity.person_id = link.person_id
        WHERE domain.domain = ANY($1)
          AND identity.identity_value = ANY($2)
        "#,
    )
    .bind(vec![sender_domain.as_str(), recipient_domain.as_str()])
    .bind(vec![sender_email.as_str(), recipient_email.as_str()])
    .fetch_one(&pool)
    .await
    .expect("organization contact links");
    assert_eq!(organization_link_count, 2);

    let organization_relationship_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM relationships relationship
        JOIN relationship_evidence evidence
          ON evidence.relationship_id = relationship.relationship_id
        JOIN organization_domains domain
          ON domain.organization_id = relationship.target_entity_id
        JOIN person_identities identity
          ON identity.person_id = relationship.source_entity_id
        WHERE relationship.source_entity_kind = 'persona'
          AND relationship.target_entity_kind = 'organization'
          AND relationship.relationship_type = 'member_of'
          AND relationship.review_state = 'system_accepted'
          AND relationship.metadata->>'compatibility_table' = 'organization_contact_links'
          AND relationship.metadata->>'source' = 'email_sync'
          AND evidence.source_kind = 'communication'
          AND evidence.source_id = $1
          AND domain.domain = ANY($2)
          AND identity.identity_value = ANY($3)
        "#,
    )
    .bind(&message_id)
    .bind(vec![sender_domain.as_str(), recipient_domain.as_str()])
    .bind(vec![sender_email.as_str(), recipient_email.as_str()])
    .fetch_one(&pool)
    .await
    .expect("organization relationships");
    assert_eq!(organization_relationship_count, 2);

    let organization_projection_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'organizations'
          AND entity_kind = 'organization'
          AND relationship_kind = 'email_sync_projection'
        "#,
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("organization observation links");
    assert_eq!(organization_projection_observation_link_count, 2);

    let organization_domain_projection_observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'organizations'

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/enrichment_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/enrichment_engine.rs`
- Size bytes / Размер в байтах: `2563`
- Included characters / Включено символов: `2563`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::engines::enrichment::EnrichmentEngine;
use serde_json::json;

#[test]
fn enrichment_engine_builds_persona_favorite_preference_draft() {
    let draft =
        EnrichmentEngine::persona_favorite_preference("person:v1:email:alice@example.com", true)
            .expect("favorite state should create a preference draft");

    assert_eq!(draft.preference_type, "ui:favorite");
    assert_eq!(draft.value, "true");
    assert_eq!(
        draft.source,
        "persons.is_favorite:person:v1:email:alice@example.com"
    );
    assert_eq!(draft.confidence, 1.0);
}

#[test]
fn enrichment_engine_skips_persona_favorite_preference_when_disabled() {
    let draft =
        EnrichmentEngine::persona_favorite_preference("person:v1:email:alice@example.com", false);

    assert!(draft.is_none());
}

#[test]
fn enrichment_engine_builds_source_backed_persona_observation_candidate() {
    let draft = EnrichmentEngine::persona_observation_candidate(
        "person:v1:email:alice@example.com",
        "communication_messages:message-1",
        "prefers concise asynchronous summaries",
        json!({
            "field": "communication_style",
            "value": "concise asynchronous summaries"
        }),
        0.82,
    )
    .expect("source-backed candidate should be valid");

    assert_eq!(draft.entity_kind, "persona");
    assert_eq!(draft.entity_id, "person:v1:email:alice@example.com");
    assert_eq!(draft.source, "communication_messages:message-1");
    assert_eq!(
        draft.extracted_claim,
        "prefers concise asynchronous summaries"
    );
    assert_eq!(draft.confidence, 0.82);
    assert_eq!(draft.review_state, "pending");
    assert_eq!(draft.freshness, "current");
    assert!(!draft.conflict_marker);
    assert_eq!(draft.data["field"], "communication_style");
    assert_eq!(
        draft.data["_enrichment"]["affected_entity_id"],
        "person:v1:email:alice@example.com"
    );
    assert_eq!(
        draft.data["_enrichment"]["extracted_claim"],
        "prefers concise asynchronous summaries"
    );
}

#[test]
fn enrichment_engine_rejects_unsourced_persona_observation_candidate() {
    let error = EnrichmentEngine::persona_observation_candidate(
        "person:v1:email:alice@example.com",
        " ",
        "prefers concise asynchronous summaries",
        json!({"field": "communication_style"}),
        0.82,
    )
    .expect_err("candidate source should be required");

    assert_eq!(
        error.to_string(),
        "enrichment candidate source must not be empty"
    );
}
```

### `backend/tests/event_consumers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/event_consumers.rs`
- Size bytes / Размер в байтах: `28817`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionPort, CommunicationProviderAccountStore, CommunicationProviderKind,
    NewProviderAccount,
};
use makosh_hub_backend::domains::communications::messages::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, ProviderChannelMessageStore,
    consume_accepted_signal_event, project_provider_observation_event,
};
use makosh_hub_backend::domains::signal_hub::{
    SIGNAL_HUB_RAW_SIGNAL_CONSUMER, dispatch_telegram_raw_signal, process_signal_hub_raw_event,
};
use makosh_hub_backend::integrations::telegram::client::{
    NewTelegramMessage, TelegramChatKind, TelegramDeliveryState, TelegramStore,
};
use makosh_hub_backend::platform::communications::{
    EventStoreProviderMessageObservationEventPort, ProviderMessageObservationEvent,
    ProviderMessageObservationEventPort,
};
use makosh_hub_backend::platform::events::{
    EventConsumerConfig, EventConsumerRunner, EventDeadLetterReviewState, EventStore,
    EventStoreError, NewEventEnvelope,
};
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

async fn live_context(_test_name: &str) -> Option<(Database, EventStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());
    Some((database, store))
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
    let Some((database, store)) = live_context("event consumer cursor").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "cursor").await;
    let pool = database.pool().expect("configured pool").clone();
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
    let Some((database, store)) = live_context("event consumer DLQ").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "dlq").await;
    let pool = database.pool().expect("configured pool").clone();
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
    let Some((database, store)) = live_context("event consumer DLQ replay").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "replay").await;
    let pool = database.pool().expect("configured pool").clone();
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
    let Some((database, store)) = live_context("event consumer idempotency").await else {
        return;
    };
    let suffix = unique_suffix();
    let position = append_test_event(&store, suffix, "idempotent").await;
    let pool = database.pool().expect("configured pool").clone();
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
    .execute(database.pool().expect("configured pool"))
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

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/event_log.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/event_log.rs`
- Size bytes / Размер в байтах: `18168`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{DateTime, Utc};
use serde_json::json;

use makosh_hub_backend::platform::events::{
    EventConsumerStore, EventEnvelope, EventStore, NewEventEnvelope, ProjectionCursorStore,
    StoredEventEnvelope, TraceContext,
};
use makosh_hub_backend::platform::storage::Database;

#[test]
fn new_event_envelope_rejects_empty_event_type() {
    let error = NewEventEnvelope::builder(
        "evt_test_empty_type",
        " ",
        Utc::now(),
        json!({"kind": "test", "source_id": "source-empty-type"}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect_err("empty event type must fail");

    assert_eq!(error.to_string(), "event_type must not be empty");
}

#[test]
fn new_event_envelope_rejects_non_object_source() {
    let error = NewEventEnvelope::builder(
        "evt_test_bad_source",
        "system_test_event",
        Utc::now(),
        json!("not-an-object"),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect_err("non-object source must fail");

    assert_eq!(error.to_string(), "source must be a JSON object");
}

#[test]
fn new_event_envelope_normalizes_missing_correlation_id_to_event_id() {
    let event = NewEventEnvelope::builder(
        " evt_test_missing_correlation ",
        "system_test_event",
        Utc::now(),
        json!({"kind": "test", "source_id": "source-missing-correlation"}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .correlation_id(" ")
    .build()
    .expect("valid event");

    assert_eq!(event.event_id, "evt_test_missing_correlation");
    assert_eq!(
        event.correlation_id.as_deref(),
        Some("evt_test_missing_correlation")
    );
}

#[test]
fn trace_context_builds_root_and_child_contexts() {
    let root = TraceContext::root("trace-root");
    assert_eq!(root.correlation_id, "trace-root");
    assert_eq!(root.causation_id, None);

    let parent = EventEnvelope {
        event_id: "evt_parent".to_owned(),
        event_type: "system_test_event".to_owned(),
        schema_version: 1,
        occurred_at: Utc::now(),
        recorded_at: Utc::now(),
        source: json!({"kind": "test"}),
        actor: None,
        subject: json!({"kind": "system", "entity_id": "backend"}),
        payload: json!({}),
        provenance: json!({}),
        causation_id: None,
        correlation_id: Some("trace-parent".to_owned()),
    };

    let child = TraceContext::child_of(&parent);
    assert_eq!(child.correlation_id, "trace-parent");
    assert_eq!(child.causation_id.as_deref(), Some("evt_parent"));
}

#[tokio::test]
async fn event_store_appends_and_loads_event_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_test_{suffix}");
    let occurred_at: DateTime<Utc> = Utc::now();

    let event = NewEventEnvelope::builder(
        &event_id,
        "system_test_event",
        occurred_at,
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": event_id,
            "import_batch_id": "event-log-test"
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .payload(json!({"test": true}))
    .provenance(json!({"confidence": 1.0}))
    .correlation_id("corr_event_log_test")
    .build()
    .expect("valid event");

    store.append(&event).await.expect("append event");

    let loaded = store
        .get_by_id(&event_id)
        .await
        .expect("load event")
        .expect("event exists");

    assert_eq!(
        loaded,
        EventEnvelope {
            event_id: event_id.clone(),
            event_type: "system_test_event".to_owned(),
            schema_version: 1,
            occurred_at,
            recorded_at: loaded.recorded_at,
            source: json!({
                "kind": "test",
                "provider": "integration",
                "source_id": loaded.event_id,
                "import_batch_id": "event-log-test"
            }),
            actor: None,
            subject: json!({"kind": "system", "entity_id": "backend"}),
            payload: json!({"test": true}),
            provenance: json!({"confidence": 1.0}),
            causation_id: None,
            correlation_id: Some("corr_event_log_test".to_owned()),
        }
    );

    let duplicate_source_event = NewEventEnvelope::builder(
        format!("{event_id}_duplicate"),
        "system_test_event",
        occurred_at,
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": loaded.event_id,
            "import_batch_id": "event-log-test"
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid duplicate source event");

    assert!(
        store.append(&duplicate_source_event).await.is_err(),
        "same event_type and source identity must be idempotent"
    );

    let mutation = sqlx::query("UPDATE event_log SET payload = '{}'::jsonb WHERE event_id = $1")
        .bind(&loaded.event_id)
        .execute(database.pool().expect("configured pool"))
        .await;

    assert!(mutation.is_err(), "event_log must be append-only");
}

#[tokio::test]
async fn event_store_replays_events_after_position_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let occurred_at = Utc::now();
    let first_id = format!("evt_replay_first_{suffix}");
    let second_id = format!("evt_replay_second_{suffix}");

    let first = NewEventEnvelope::builder(
        &first_id,
        "system_replay_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": first_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid first event");
    let first_position = store.append(&first).await.expect("append first event");

    let second = NewEventEnvelope::builder(
        &second_id,
        "system_replay_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": second_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid second event");
    let second_position = store.append(&second).await.expect("append second event");

    let replayed = store
        .list_after_position(first_position, 10)
        .await
        .expect("replay events");

    assert_eq!(
        replayed,
        vec![StoredEventEnvelope {
            position: second_position,
            event: EventEnvelope {
                event_id: second_id,
                event_type: "system_replay_test_event".to_owned(),
                schema_version: 1,
                occurred_at,
                recorded_at: replayed[0].event.recorded_at,
                source: json!({
                    "kind": "test",
                    "provider": "integration",
                    "source_id": replayed[0].event.event_id
                }),
                actor: None,
                subject: json!({"kind": "system", "entity_id": "backend"}),
                payload: json!({}),
                provenance: json!({}),
                causation_id: None,
                correlation_id: Some(replayed[0].event.event_id.clone()),
            },
        }]
    );
}

#[tokio::test]
async fn event_store_reconstructs_trace_edges_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let occurred_at = Utc::now();
    let trace_id = format!("trace_event_log_{suffix}");
    let root_id = format!("evt_trace_root_{suffix}");
    let child_id = format!("evt_trace_child_{suffix}");
    let grandchild_id = format!("evt_trace_grandchild_{suffix}");

    let root = NewEventEnvelope::builder(
        &root_id,
        "system_trace_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": root_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .correlation_id(&trace_id)
    .build()
    .expect("valid root event");
    store.append(&root).await.expect("append root event");

    let child = NewEventEnvelope::builder(
        &child_id,
        "system_trace_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": child_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .causation_id(&root_id)
    .correlation_id(&trace_id)
    .build()
    .expect("valid child event");
    store.append(&child).await.expect("append child event");

    let grandchild = NewEventEnvelope::builder(
        &grandchild_id,
        "system_trace_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": grandchild_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .causation_id(&child_id)
    .correlation_id(&trace_id)
    .build()
    .expect("valid grandchild event");
    store
        .append(&grandchild)
        .await
        .expect("append grandchild event");

    let trace = store
        .trace_by_event_id(&grandchild_id, 100)
        .await
        .expect("trace query")
        .expect("trace exists");

    assert_eq!(trace.correlation_id, trace_id);
    assert_eq!(trace.root_event_ids, vec![root_id.clone()]);
    assert_eq!(trace.events.len(), 3);
    assert_eq!(trace.missing_parent_ids, Vec::<String>::new());
    assert_eq!(trace.orphan_event_ids, Vec::<String>::new());
    assert_eq!(
        trace.edges,
        vec![
            makosh_hub_backend::platform::events::EventTraceEdge {
                parent_event_id: root_id.clone(),
                child_event_id: child_id.clone(),
            },
            makosh_hub_backend::platform::events::EventTraceEdge {
                parent_event_id: child_id.clone(),
                child_event_id: grandchild_id,
            },
        ]
    );

    let children = store
        .list_children(&root_id, 10)
        .await
        .expect("children query");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].event.event_id, child_id);
}

#[tokio::test]
async fn event_store_reports_missing_trace_parent_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let trace_id = format!("trace_missing_parent_{suffix}");
    let event_id = format!("evt_missing_parent_{suf
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/event_platform.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/event_platform.rs`
- Size bytes / Размер в байтах: `9948`
- Included characters / Включено символов: `9948`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use futures::StreamExt;
use serde_json::json;
use sqlx::Row;
use testkit::context::TestContext;
use tokio::time::{Duration, timeout};

use makosh_hub_backend::platform::events::{
    EventOutboxDispatcher, EventStore, InMemoryEventBus, NatsJetStreamEventBus, NewEventEnvelope,
};

#[tokio::test]
async fn append_for_dispatch_records_event_and_pending_outbox_subject() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!("evt_outbox_{}", occurred_at.timestamp_nanos_opt().unwrap()),
        "signal.accepted.telegram.message",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "accepted-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "accepted-message-1"
        }),
    )
    .build()
    .expect("valid event");

    let position = store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");

    assert!(position > 0);

    let outbox = store
        .pending_outbox_batch(10)
        .await
        .expect("load pending outbox");

    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_id, event.event_id);
    assert_eq!(outbox[0].subject, "signal.accepted.telegram.message");
    assert_eq!(outbox[0].status, "pending");
    assert_eq!(outbox[0].attempts, 0);
}

#[tokio::test]
async fn in_memory_event_bus_delivers_events_to_subscribers() {
    let bus = InMemoryEventBus::new();
    let mut subscriber = bus.subscribe();
    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_memory_bus_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.fixture.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "fixture",
            "source_id": "fixture-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "fixture",
            "entity_id": "fixture-message-1"
        }),
    )
    .build()
    .expect("valid event");

    assert_eq!(bus.broadcast(event.clone()), 1);

    let received = subscriber.recv().await.expect("receive event");
    assert_eq!(received.event_id, event.event_id);
    assert_eq!(received.event_type, event.event_type);
}

#[tokio::test]
async fn event_outbox_dispatcher_publishes_pending_events_to_nats() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let nats_server_url = ctx.nats_server_url().await;
    let bus = NatsJetStreamEventBus::connect(&nats_server_url)
        .await
        .expect("connect JetStream bus");
    let dispatcher = EventOutboxDispatcher::new(store.clone(), bus);
    let client = async_nats::connect(&nats_server_url)
        .await
        .expect("connect NATS client");
    let event_subject = format!(
        "signal.accepted.telegram.message.test.{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    );
    let mut subscriber = client
        .subscribe(event_subject.clone())
        .await
        .expect("subscribe to accepted telegram signal");

    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_dispatch_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        event_subject.clone(),
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "accepted-message-2"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "accepted-message-2"
        }),
    )
    .payload(json!({
        "message_id": "accepted-message-2"
    }))
    .build()
    .expect("valid event");

    store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");

    let report = dispatcher
        .dispatch_pending_once()
        .await
        .expect("dispatch outbox");

    assert_eq!(report.recovered, 0);
    assert_eq!(report.claimed, 1);
    assert_eq!(report.published, 1);
    assert_eq!(report.retried, 0);

    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("message receive timeout")
        .expect("subscription yields message");
    let published_event: makosh_hub_backend::platform::events::EventEnvelope =
        serde_json::from_slice(&message.payload).expect("decode published event");
    assert_eq!(published_event.event_id, event.event_id);
    assert_eq!(published_event.event_type, event.event_type);

    let row = sqlx::query(
        r#"
        SELECT status, attempts, published_at IS NOT NULL AS published
        FROM event_outbox
        WHERE event_id = $1
        "#,
    )
    .bind(&event.event_id)
    .fetch_one(ctx.pool())
    .await
    .expect("load event outbox row");

    let status: String = row.try_get("status").expect("status");
    let attempts: i32 = row.try_get("attempts").expect("attempts");
    let published: bool = row.try_get("published").expect("published flag");

    assert_eq!(status, "published");
    assert_eq!(attempts, 1);
    assert!(published);
}

#[tokio::test]
async fn event_outbox_dispatcher_broadcasts_published_events_to_realtime_bus() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let nats_server_url = ctx.nats_server_url().await;
    let jetstream_bus = NatsJetStreamEventBus::connect(&nats_server_url)
        .await
        .expect("connect JetStream bus");
    let realtime_bus = InMemoryEventBus::new();
    let mut subscriber = realtime_bus.subscribe();
    let dispatcher = EventOutboxDispatcher::new(store.clone(), jetstream_bus)
        .with_realtime_bus(realtime_bus.clone());

    let event_subject = format!(
        "signal.accepted.telegram.message.test.{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    );
    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_realtime_dispatch_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        event_subject,
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "accepted-message-realtime"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "accepted-message-realtime"
        }),
    )
    .payload(json!({
        "message_id": "accepted-message-realtime"
    }))
    .build()
    .expect("valid event");

    store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");

    let report = dispatcher
        .dispatch_pending_once()
        .await
        .expect("dispatch outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.published, 1);

    let received = timeout(Duration::from_secs(5), subscriber.recv())
        .await
        .expect("receive timeout")
        .expect("receive realtime event");
    assert_eq!(received.event_id, event.event_id);
    assert_eq!(received.event_type, event.event_type);
    assert_eq!(
        received.payload["message_id"],
        json!("accepted-message-realtime")
    );
}

#[tokio::test]
async fn event_outbox_dispatcher_recovers_stale_dispatching_items() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let nats_server_url = ctx.nats_server_url().await;
    let bus = NatsJetStreamEventBus::connect(&nats_server_url)
        .await
        .expect("connect JetStream bus");
    let dispatcher = EventOutboxDispatcher::new(store.clone(), bus);
    let client = async_nats::connect(&nats_server_url)
        .await
        .expect("connect NATS client");
    let mut subscriber = client
        .subscribe("signal.accepted.mail.message")
        .await
        .expect("subscribe to accepted mail signal");

    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!("evt_recover_{}", occurred_at.timestamp_nanos_opt().unwrap()),
        "signal.accepted.mail.message",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "mail",
            "source_id": "accepted-mail-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "mail",
            "entity_id": "accepted-mail-1"
        }),
    )
    .build()
    .expect("valid event");

    store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");
    let claimed = store
        .claim_pending_outbox_batch(10)
        .await
        .expect("claim event outbox item");
    assert_eq!(claimed.len(), 1);

    sqlx::query(
        r#"
        UPDATE event_outbox
        SET updated_at = now() - interval '5 minutes'
        WHERE event_id = $1
        "#,
    )
    .bind(&event.event_id)
    .execute(ctx.pool())
    .await
    .expect("mark event outbox item stale");

    let report = dispatcher
        .dispatch_pending_once()
        .await
        .expect("dispatch recovered outbox");

    assert_eq!(report.recovered, 1);
    assert_eq!(report.claimed, 1);
    assert_eq!(report.published, 1);
    assert_eq!(report.retried, 0);

    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("message receive timeout")
        .expect("subscription yields message");
    let published_event: makosh_hub_backend::platform::events::EventEnvelope =
        serde_json::from_slice(&message.payload).expect("decode published event");
    assert_eq!(published_event.event_id, event.event_id);
}
```

### `backend/tests/events_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/events_api.rs`
- Size bytes / Размер в байтах: `19900`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::{self, context::TestContext};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "events-api-test-token";

#[tokio::test]
async fn post_event_rejects_when_local_api_secret_is_not_configured() {
    let app = build_router(
        testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_pairs([("MAKOSH_DEV_MODE", "true")])
            .expect("app config"),
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_no_db",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_no_db"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_missing_token",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_missing_token"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_rejects_invalid_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_invalid_token",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_invalid_token"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            "wrong-token",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_accepts_secret_without_actor_header_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request_with_token_without_actor(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_missing_actor",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_missing_actor"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn get_event_ignores_actor_header_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/events/evt_api_invalid_actor",
            LOCAL_API_TOKEN,
            "invalid actor",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn get_event_rejects_missing_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/events/evt_api_missing_token"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn get_event_rejects_invalid_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/events/evt_api_invalid_token",
            "wrong-token",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn get_audit_events_rejects_missing_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request(
            "/api/v1/audit/events?target_id=evt_api_audit_missing_token",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_returns_service_unavailable_when_database_is_not_configured() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_no_db",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_no_db"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn post_event_rejects_invalid_envelope() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let app = app_with_database(&database_url).await;

    let response = app
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_invalid",
                "event_type": " ",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_invalid"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_event_envelope",
            "message": "event_type must not be empty"
        })
    );
}

#[tokio::test]
async fn post_then_get_event_round_trips_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let (app, pool) = app_and_pool_with_database(&database_url).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_round_trip_{suffix}");
    let occurred_at = Utc::now();

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": occurred_at,
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "payload": {"api": true},
                "provenance": {"confidence": 1.0},
                "correlation_id": "corr_events_api_test"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = json_body(create_response).await;
    assert_eq!(create_body["event_id"], event_id);
    assert!(create_body["position"].as_i64().expect("position") > 0);

    let get_response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{event_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("get response");

    assert_eq!(get_response.status(), StatusCode::OK);

    let get_body = json_body(get_response).await;
    assert_eq!(get_body["event_id"], event_id);
    assert_eq!(get_body["event_type"], "system_api_test_event");
    assert_eq!(get_body["payload"], json!({"api": true}));
    assert_eq!(get_body["provenance"], json!({"confidence": 1.0}));

    let audit_operations = audit_operations_for_target(&pool, &event_id).await;
    assert_eq!(
        audit_operations,
        vec!["event.append".to_owned(), "event.get".to_owned()]
    );

    let mutation =
        sqlx::query("UPDATE api_audit_log SET metadata = '{}'::jsonb WHERE target_id = $1")
            .bind(&event_id)
            .execute(&pool)
            .await;
    assert!(mutation.is_err(), "api_audit_log must be append-only");
}

#[tokio::test]
async fn get_event_returns_not_found_for_missing_event_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let (app, pool) = app_and_pool_with_database(&database_url).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_missing_{suffix}");

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{event_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let audit_operations = audit_operations_for_target(&pool, &event_id).await;
    assert_eq!(audit_operations, vec!["event.get".to_owned()]);
}

#[tokio::test]
async fn get_audit_events_returns_records_without_self_auditing_against_postgres() {
    let test_context = Tes
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/events_long_poll_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/events_long_poll_api.rs`
- Size bytes / Размер в байтах: `4939`
- Included characters / Включено символов: `4939`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "events-long-poll-test-token";

#[tokio::test]
async fn get_events_lists_replay_batch_and_audits_access_against_postgres() {
    let context = TestContext::new().await;
    let (app, pool) = app_and_pool_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_list_{suffix}");

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "payload": {"list": true}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body = json_body(create_response).await;
    let position = create_body["position"].as_i64().expect("position");

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/events?after_position={}&limit=10&wait_seconds=0",
                position - 1
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("list response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["position"], position);
    assert_eq!(items[0]["event"]["event_id"], event_id);
    assert_eq!(items[0]["event"]["payload"], json!({"list": true}));
    assert_eq!(body["next_after_position"], position);
    assert_eq!(body["has_more"], false);

    let audit = latest_event_list_audit_record(&pool).await;
    assert_eq!(audit["operation"], "event.list");
    assert_eq!(audit["method"], "GET");
    assert_eq!(audit["target_kind"], "event");
    assert!(audit["target_id"].is_null());
    assert_eq!(audit["metadata"]["after_position"], position - 1);
    assert_eq!(audit["metadata"]["limit"], 10);
    assert_eq!(audit["metadata"]["wait_seconds"], 0);
}

async fn app_and_pool_with_database(database_url: &str) -> (axum::Router, sqlx::PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    (
        build_router_with_database(config_with_api_token(), database),
        pool,
    )
}

fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn json_request_with_token(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");

    serde_json::from_slice(&body).expect("json body")
}

async fn latest_event_list_audit_record(pool: &sqlx::PgPool) -> Value {
    let row = sqlx::query(
        r#"
        SELECT operation, method, target_kind, target_id, metadata
        FROM api_audit_log
        WHERE operation = 'event.list'
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("event list audit record");

    json!({
        "operation": row.try_get::<String, _>("operation").expect("operation"),
        "method": row.try_get::<String, _>("method").expect("method"),
        "target_kind": row.try_get::<String, _>("target_kind").expect("target_kind"),
        "target_id": row.try_get::<Option<String>, _>("target_id").expect("target_id"),
        "metadata": row.try_get::<Value, _>("metadata").expect("metadata")
    })
}
```

### `backend/tests/events_stream_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/events_stream_api.rs`
- Size bytes / Размер в байтах: `7341`
- Included characters / Включено символов: `7341`
- Truncated / Обрезано: `no`

```rust
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use futures::StreamExt;
use serde_json::json;
use tokio::time::timeout;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "events-stream-test-token";

#[tokio::test]
async fn event_stream_replays_event_log_positions_as_sse_against_postgres() {
    let context = TestContext::new().await;
    let app = app_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_stream_{suffix}");

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body = json_body(create_response).await;
    let position = create_body["position"].as_i64().expect("position");

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/events/stream?after_position={}", position - 1),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("stream response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.starts_with("text/event-stream")),
        Some(true)
    );

    let mut stream = response.into_body().into_data_stream();
    let chunk = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("first SSE chunk timed out")
        .expect("first SSE chunk")
        .expect("first SSE chunk bytes");
    let text = std::str::from_utf8(&chunk).expect("SSE chunk is UTF-8");

    assert!(text.contains(&format!("id: {position}")), "{text}");
    assert!(text.contains("event: event"), "{text}");
    assert!(text.contains("system_api_test_event"), "{text}");
    assert!(text.contains("correlation_id"), "{text}");
    assert!(text.contains(&event_id), "{text}");
}

#[tokio::test]
async fn event_trace_api_returns_causal_edges_against_postgres() {
    let context = TestContext::new().await;
    let app = app_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let trace_id = format!("trace_api_{suffix}");
    let root_id = format!("evt_api_trace_root_{suffix}");
    let child_id = format!("evt_api_trace_child_{suffix}");

    let root_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": root_id,
                "event_type": "system_api_trace_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": root_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "correlation_id": trace_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("root create response");
    assert_eq!(root_response.status(), StatusCode::CREATED);

    let child_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": child_id,
                "event_type": "system_api_trace_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": child_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "causation_id": root_id,
                "correlation_id": trace_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("child create response");
    assert_eq!(child_response.status(), StatusCode::CREATED);

    let trace_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{child_id}/trace"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("trace response");
    assert_eq!(trace_response.status(), StatusCode::OK);
    let trace_body = json_body(trace_response).await;

    assert_eq!(trace_body["correlation_id"], json!(trace_id));
    assert_eq!(trace_body["root_event_ids"], json!([root_id]));
    assert_eq!(trace_body["events"].as_array().expect("events").len(), 2);
    assert_eq!(
        trace_body["edges"],
        json!([{
            "parent_event_id": root_id,
            "child_event_id": child_id
        }])
    );
    assert_eq!(trace_body["missing_parent_ids"], json!([]));
    assert_eq!(trace_body["orphan_event_ids"], json!([]));

    let children_response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{root_id}/children"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("children response");
    assert_eq!(children_response.status(), StatusCode::OK);
    let children_body = json_body(children_response).await;
    assert_eq!(children_body.as_array().expect("children").len(), 1);
    assert_eq!(children_body[0]["event"]["event_id"], json!(child_id));
}

async fn app_with_database(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(config_with_api_token(), database)
}

fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn json_request_with_token(uri: &str, value: serde_json::Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");

    serde_json::from_slice(&body).expect("json body")
}
```

### `backend/tests/events_websocket_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/events_websocket_api.rs`
- Size bytes / Размер в байтах: `3896`
- Included characters / Включено символов: `3896`
- Truncated / Обрезано: `no`

```rust
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "events-websocket-test-token";

#[tokio::test]
async fn event_websocket_accepts_protected_upgrade_against_postgres() {
    let context = TestContext::new().await;
    let app = app_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_ws_{suffix}");

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("test server");
    });

    let mut stream = TcpStream::connect(address).await.expect("websocket socket");
    stream
        .write_all(
            format!(
                "GET /api/events/ws?after_position=0&batch_size=10&heartbeat_seconds=1&makosh_secret={LOCAL_API_TOKEN} HTTP/1.1\r\n\
                 Host: {address}\r\n\
                 Connection: Upgrade\r\n\
                 Upgrade: websocket\r\n\
                 Sec-WebSocket-Version: 13\r\n\
                 Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
                 \r\n"
            )
            .as_bytes(),
        )
        .await
        .expect("websocket upgrade request");

    let mut buffer = [0_u8; 2048];
    let bytes_read = timeout(Duration::from_secs(2), stream.read(&mut buffer))
        .await
        .expect("websocket upgrade response timed out")
        .expect("websocket upgrade response");
    let response_text = std::str::from_utf8(&buffer[..bytes_read]).expect("utf-8 response");
    let normalized_response = response_text.to_ascii_lowercase();

    assert!(
        response_text.starts_with("HTTP/1.1 101 Switching Protocols"),
        "{response_text}"
    );
    assert!(
        normalized_response.contains("upgrade: websocket"),
        "{response_text}"
    );

    server.abort();
}

async fn app_with_database(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(config_with_api_token(), database)
}

fn config_with_api_token() -> AppConfig {
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn json_request_with_token(uri: &str, value: serde_json::Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}
```

### `backend/tests/gmail_outbox_delivery.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/gmail_outbox_delivery.rs`
- Size bytes / Размер в байтах: `12596`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxEmailSender, CommunicationOutboxStatus, CommunicationOutboxStore,
    EmailOutboxDeliveryWorker, NewCommunicationOutboxItem,
};
use makosh_hub_backend::integrations::mail::outbox::LiveGmailOutboxTransport;
use makosh_hub_backend::integrations::mail::send::LiveSmtpTransport;
use makosh_hub_backend::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{HostVault, HostVaultConfig, SecretEntryContext};
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "gmail-outbox-delivery-test-token";

#[tokio::test]
async fn outbox_delivery_worker_sends_gmail_items_through_gmail_api_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app).await;

    let gmail_api = MockGmailApiServer::start();
    let account_id = "gmail-outbox-enabled";
    let secret_ref = format!("secret:provider-account:{account_id}:oauth_token");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                EmailProviderKind::Gmail,
                "Gmail Outbox Enabled",
                "sender@gmail.com",
            )
            .config(json!({
                "auth": "oauth",
                "api": "gmail",
                "gmail_send_enabled": true,
                "gmail_api_base_url": gmail_api.base_url()
            })),
        )
        .await
        .expect("store gmail account");
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::HostVault,
            "Gmail outbox OAuth credential",
        ))
        .await
        .expect("store gmail OAuth secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &secret_ref,
        ))
        .await
        .expect("bind gmail OAuth secret");
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .store_secret(
            &secret_ref,
            &json!({
                "token_url": "http://127.0.0.1:1/token",
                "client_id": "desktop-client-id",
                "access_token": "gmail-access-token",
                "refresh_token": "gmail-refresh-token",
                "expires_at": "2999-01-01T00:00:00Z",
                "scope": "https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/gmail.send"
            })
            .to_string(),
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id,
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "Gmail OAuth credential",
                metadata: &json!({ "provider": "gmail", "account_id": account_id }),
            },
        )
        .expect("store gmail OAuth bundle");

    let outbox_store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let outbox_id = "outbox:gmail:scheduled";
    outbox_store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.to_owned(),
            account_id: account_id.to_owned(),
            draft_id: None,
            to_recipients: vec!["recipient@example.com".to_owned()],
            cc_recipients: vec!["copy@example.com".to_owned()],
            bcc_recipients: Vec::new(),
            subject: "Scheduled Gmail API send".to_owned(),
            body_text: "Queued Gmail outbox body.".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Scheduled,
            scheduled_send_at: Some(now - Duration::seconds(1)),
            undo_deadline_at: Some(now - Duration::seconds(1)),
            metadata: json!({}),
        })
        .await
        .expect("enqueue gmail outbox item");

    let worker = EmailOutboxDeliveryWorker::new(
        outbox_store.clone(),
        CommunicationOutboxEmailSender::new(
            pool.clone(),
            vault.clone(),
            LiveSmtpTransport,
            LiveGmailOutboxTransport::new(pool.clone(), vault),
        ),
    );
    let report = worker
        .deliver_due(now + Duration::seconds(1), 10)
        .await
        .expect("deliver gmail outbox");
    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.retried, 0);

    let sent_items = outbox_store
        .list(Some(account_id), Some(CommunicationOutboxStatus::Sent), 10)
        .await
        .expect("list sent outbox");
    assert_eq!(sent_items.len(), 1);
    assert_eq!(
        sent_items[0].provider_message_id.as_deref(),
        Some("gmail-api-message-id")
    );

    let requests = gmail_api.requests();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .request_line
            .starts_with("POST /gmail/v1/users/me/messages/send ")
    );
    assert_eq!(
        requests[0].header("authorization").as_deref(),
        Some("Bearer gmail-access-token")
    );
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn unlock_test_vault<S>(app: S)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let entropy_response = app
        .clone()
        .oneshot(post(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);

    let create_response = app
        .oneshot(post("/api/v1/vault/create", json!({})))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);
}

fn vault_entropy_events(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "x": index % 997,
                "y": index % 577,
                "dx": (index % 11) as i64 - 5,
                "dy": (index % 13) as i64 - 6,
                "timestamp_ms": index * 5,
                "velocity": (index % 19) as f64 / 10.0,
                "acceleration": (index % 23) as f64 / 100.0,
                "interval_ms": 5
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct HttpRequest {
    request_line: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl HttpRequest {
    fn header(&self, name: &str) -> Option<String> {
        self.headers
            .iter()
            .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.clone())
    }
}

struct MockGmailApiServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockGmailApiServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind Gmail API server");
        let addr = listener.local_addr().expect("Gmail API server addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let request = read_http_request(&mut stream);
            if request.body.is_empty() {
                return;
            }
            requests_for_thread
                .lock()
                .expect("Gmail API requests lock")
                .push(request);
            write_http_response(
                &mut stream,
                &json!({
                    "id": "gmail-api-message-id",
                    "threadId": "gmail-api-thread-id",
                    "labelIds": ["SENT"]
                })
                .to_string(),
            );
        });

        Self {
            addr,
            requests,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> Vec<HttpRequest> {
        self.requests
            .lock()
            .expect("Gmail API requests lock")
            .clone()
    }
}

impl Drop for MockGmailApiServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("Gmail API server join");
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> HttpRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut content_length = 0usize;
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .expect("read request line");
    let request_line = request_line.trim_end().to_owned();
    let mut headers = Vec::new();

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read header line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_owned();
            let value = value.trim().to_owned();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().expect("content length");
            }
            headers.push((name, value));
        }
    }

    let mut body = vec![0_u8; content_length];
    use std::io::Read;
    reader.read_exa
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/gmail_send_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/gmail_send_api.rs`
- Size bytes / Размер в байтах: `12261`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use makosh_hub_backend::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{HostVault, HostVaultConfig, SecretEntryContext};
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "gmail-send-api-test-token";

#[tokio::test]
async fn gmail_send_api_queues_outbox_when_send_scope_enabled_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "MAKOSH_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let gmail_api = MockGmailApiServer::start();
    let account_id = "gmail-send-enabled";
    let secret_ref = format!("secret:provider-account:{account_id}:oauth_token");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                EmailProviderKind::Gmail,
                "Gmail Send Enabled",
                "sender@gmail.com",
            )
            .config(json!({
                "auth": "oauth",
                "api": "gmail",
                "gmail_send_enabled": true,
                "gmail_api_base_url": gmail_api.base_url()
            })),
        )
        .await
        .expect("store gmail account");
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::HostVault,
            "Gmail send OAuth credential",
        ))
        .await
        .expect("store gmail OAuth secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &secret_ref,
        ))
        .await
        .expect("bind gmail OAuth secret");
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .store_secret(
            &secret_ref,
            &json!({
                "token_url": "http://127.0.0.1:1/token",
                "client_id": "desktop-client-id",
                "access_token": "gmail-access-token",
                "refresh_token": "gmail-refresh-token",
                "expires_at": "2999-01-01T00:00:00Z",
                "scope": "https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/gmail.send"
            })
            .to_string(),
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id,
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "Gmail OAuth credential",
                metadata: &json!({ "provider": "gmail", "account_id": account_id }),
            },
        )
        .expect("store gmail OAuth bundle");

    let response = app
        .oneshot(post(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "cc": ["copy@example.com"],
                "subject": "Gmail API send",
                "body_text": "Message body through Gmail API.",
                "confirmed_provider_write": true
            }),
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["transport"], "outbox");
    assert_eq!(body["status"], "queued");
    assert_eq!(
        body["accepted_recipients"],
        json!(["recipient@example.com", "copy@example.com"])
    );
    let outbox_id = body["outbox_id"].as_str().expect("outbox id");
    assert_eq!(body["message_id"], json!(outbox_id));
    let outbox = sqlx::query(
        "SELECT status, to_participants, cc_participants, subject
         FROM communication_outbox
         WHERE outbox_id = $1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("gmail outbox item");
    let status: String = outbox.try_get("status").expect("outbox status");
    let subject: String = outbox.try_get("subject").expect("outbox subject");
    let to_participants: Value = outbox.try_get("to_participants").expect("to participants");
    let cc_participants: Value = outbox.try_get("cc_participants").expect("cc participants");
    assert_eq!(status, "queued");
    assert_eq!(subject, "Gmail API send");
    assert_eq!(to_participants, json!(["recipient@example.com"]));
    assert_eq!(cc_participants, json!(["copy@example.com"]));

    let link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("gmail outbox observation link");
    let link_metadata: Value = link.try_get("metadata").expect("link metadata");
    assert_eq!(link_metadata["operation"], "outbox_enqueue");
    assert_eq!(link_metadata["status"], "queued");

    let requests = gmail_api.requests();
    assert!(requests.is_empty());
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn unlock_test_vault<S>(app: S)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let entropy_response = app
        .clone()
        .oneshot(post(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);

    let create_response = app
        .oneshot(post("/api/v1/vault/create", json!({})))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);
}

fn vault_entropy_events(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "x": index % 997,
                "y": index % 577,
                "dx": (index % 11) as i64 - 5,
                "dy": (index % 13) as i64 - 6,
                "timestamp_ms": index * 5,
                "velocity": (index % 19) as f64 / 10.0,
                "acceleration": (index % 23) as f64 / 100.0,
                "interval_ms": 5
            })
        })
        .collect()
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

#[derive(Clone, Debug)]
struct HttpRequest {
    body: String,
}

struct MockGmailApiServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockGmailApiServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind Gmail API server");
        let addr = listener.local_addr().expect("Gmail API server addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let request = read_http_request(&mut stream);
            if request.body.is_empty() {
                return;
            }
            requests_for_thread
                .lock()
                .expect("Gmail API requests lock")
                .push(request);
            write_http_response(
                &mut stream,
                &json!({
                    "id": "gmail-api-message-id",
                    "threadId": "gmail-api-thread-id",
                    "labelIds": ["SENT"]
                })
                .to_string(),
            );
        });

        Self {
            addr,
            requests,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> Vec<HttpRequest> {
        self.requests
            .lock()
            .expect("Gmail API requests lock")
            .clone()
    }
}

impl Drop for MockGmailApiServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("Gmail API server join");
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> HttpRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut content_length = 0usize;
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .expect("read request line");

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read header line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_owned();
            let value = value.trim().to_owned();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().expect("content length");
            }
        }
    }

    let mut body = vec![0_u8; content_length];
    use std::io::Read;
    reader.read_exact(&mut body).expect("read request body");

    HttpRequest {
        body: String::from_utf8(body).expect("utf8 body"),
    }
}

fn write_http_response(stream: &mut TcpStream, body: &str) {
    let result = write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\nco
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/graph.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph.rs`
- Size bytes / Размер в байтах: `8781`
- Included characters / Включено символов: `8781`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use makosh_hub_backend::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, GraphStoreError,
    NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType, edge_id, evidence_id,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::Row;

#[tokio::test]
async fn graph_store_upserts_node_idempotently_against_postgres() {
    let Some(store) = live_graph_store("node idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    let node = NewGraphNode::new(
        GraphNodeKind::Person,
        format!("person-{suffix}"),
        format!("Alex {suffix}"),
    )
    .properties(json!({"email_address": format!("alex-{suffix}@example.com")}));

    let first = store.upsert_node(&node).await.expect("first node upsert");
    let second = store.upsert_node(&node).await.expect("second node upsert");

    assert_eq!(first.node_id, second.node_id);
    assert_eq!(first.node_kind, GraphNodeKind::Person);
    assert_eq!(first.stable_key, format!("person-{suffix}"));
}

#[tokio::test]
async fn graph_store_upserts_edge_with_evidence_idempotently_against_postgres() {
    let Some((pool, store)) = live_graph_context("edge idempotence").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("person-{suffix}"),
            format!("Person {suffix}"),
        ))
        .await
        .expect("person node");
    let email = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("person-{suffix}@example.com"),
            format!("person-{suffix}@example.com"),
        ))
        .await
        .expect("email node");
    let edge = NewGraphEdge::new(
        person.node_id.clone(),
        email.node_id.clone(),
        RelationshipType::PersonHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );
    let evidence_source_id = format!("person-{suffix}");
    let first_evidence =
        NewGraphEvidence::new(GraphEvidenceSourceKind::Person, evidence_source_id.clone())
            .excerpt("initial person evidence")
            .metadata(json!({"projection": "first"}));
    let second_evidence =
        NewGraphEvidence::new(GraphEvidenceSourceKind::Person, evidence_source_id.clone())
            .excerpt("updated person evidence")
            .metadata(json!({"projection": "second", "source": "duplicate-upsert"}));

    let first = store
        .upsert_edge_with_evidence(&edge, std::slice::from_ref(&first_evidence))
        .await
        .expect("first edge");
    let second = store
        .upsert_edge_with_evidence(&edge, &[second_evidence])
        .await
        .expect("second edge");

    assert_eq!(first.edge_id, second.edge_id);
    assert_eq!(
        first.relationship_type,
        RelationshipType::PersonHasEmailAddress
    );
    assert_eq!(first.review_state, GraphReviewState::SystemAccepted);

    let evidence_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_evidence WHERE edge_id = $1")
            .bind(&first.edge_id)
            .fetch_one(&pool)
            .await
            .expect("evidence count");
    assert_eq!(evidence_count, 1);

    let evidence_row = sqlx::query(
        r#"
        SELECT excerpt, metadata
        FROM graph_evidence
        WHERE edge_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.edge_id)
    .bind(GraphEvidenceSourceKind::Person.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored evidence row");

    let excerpt: Option<String> = evidence_row.try_get("excerpt").expect("evidence excerpt");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(excerpt.as_deref(), Some("updated person evidence"));
    assert_eq!(
        metadata,
        json!({"projection": "second", "source": "duplicate-upsert"})
    );
}

#[tokio::test]
async fn graph_store_rejects_system_edge_without_evidence_against_postgres() {
    let Some(store) = live_graph_store("evidence requirement").await else {
        return;
    };
    let suffix = unique_suffix();
    let left = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::Person,
            format!("left-{suffix}"),
            "Left",
        ))
        .await
        .expect("left node");
    let right = store
        .upsert_node(&NewGraphNode::new(
            GraphNodeKind::EmailAddress,
            format!("right-{suffix}@example.com"),
            "right@example.com",
        ))
        .await
        .expect("right node");
    let edge = NewGraphEdge::new(
        left.node_id,
        right.node_id,
        RelationshipType::PersonHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );

    let error = store
        .upsert_edge_with_evidence(&edge, &[])
        .await
        .expect_err("system edge without evidence must fail");

    assert!(matches!(error, GraphStoreError::SystemEdgeRequiresEvidence));
}

#[tokio::test]
async fn graph_store_rejects_suggested_edge_without_evidence_before_database_write() {
    let store = disconnected_graph_store();
    let edge = NewGraphEdge::new(
        "graph:node:v1:person:left".to_owned(),
        "graph:node:v1:email:right@example.com".to_owned(),
        RelationshipType::PersonHasEmailAddress,
        0.5,
        GraphReviewState::Suggested,
    );

    let error = store
        .upsert_edge_with_evidence(&edge, &[])
        .await
        .expect_err("suggested edge without evidence must fail");

    assert!(matches!(error, GraphStoreError::SystemEdgeRequiresEvidence));
}

#[tokio::test]
async fn graph_store_rejects_invalid_confidence_before_database_write() {
    let store = disconnected_graph_store();
    let edge = NewGraphEdge::new(
        "graph:node:v1:person:left".to_owned(),
        "graph:node:v1:email:right@example.com".to_owned(),
        RelationshipType::PersonHasEmailAddress,
        1.1,
        GraphReviewState::SystemAccepted,
    );
    let evidence = NewGraphEvidence::new(GraphEvidenceSourceKind::Person, "person-id");

    let error = store
        .upsert_edge_with_evidence(&edge, &[evidence])
        .await
        .expect_err("invalid confidence must fail");

    assert!(matches!(error, GraphStoreError::InvalidConfidence(_)));
}

#[tokio::test]
async fn graph_store_rejects_closed_temporal_edge_before_database_write() {
    let store = disconnected_graph_store();
    let mut edge = NewGraphEdge::new(
        "graph:node:v1:person:left".to_owned(),
        "graph:node:v1:email:right@example.com".to_owned(),
        RelationshipType::PersonHasEmailAddress,
        1.0,
        GraphReviewState::SystemAccepted,
    );
    edge.valid_to = Some(Utc::now());
    let evidence = NewGraphEvidence::new(GraphEvidenceSourceKind::Person, "person-id");

    let error = store
        .upsert_edge_with_evidence(&edge, &[evidence])
        .await
        .expect_err("closed temporal edge must fail");

    assert!(matches!(error, GraphStoreError::TemporalEdgesUnsupported));
}

#[test]
fn graph_deterministic_ids_distinguish_delimiter_bearing_components() {
    let relationship_type = RelationshipType::PersonHasEmailAddress;

    assert_ne!(
        edge_id("a:b", relationship_type, "c"),
        edge_id("a", relationship_type, "b:c")
    );
    assert_ne!(
        evidence_id("edge:a:b", GraphEvidenceSourceKind::Person, "c"),
        evidence_id("edge:a", GraphEvidenceSourceKind::Person, "b:c")
    );
}

async fn live_graph_store(test_name: &str) -> Option<GraphStore> {
    live_graph_context(test_name)
        .await
        .map(|(_pool, store)| store)
}

async fn live_graph_context(_test_name: &str) -> Option<(sqlx::postgres::PgPool, GraphStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    Some((pool.clone(), GraphStore::new(pool)))
}

fn disconnected_graph_store() -> GraphStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    GraphStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
```

### `backend/tests/graph_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_api.rs`
- Size bytes / Размер в байтах: `186`
- Included characters / Включено символов: `186`
- Truncated / Обрезано: `no`

```rust
#[path = "graph_api/auth.rs"]
mod auth;
#[path = "graph_api/neighborhood.rs"]
mod neighborhood;
#[path = "graph_api/search.rs"]
mod search;
#[path = "graph_api/support.rs"]
mod support;
```

### `backend/tests/graph_api/auth.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/graph_api/auth.rs`
- Size bytes / Размер в байтах: `2842`
- Included characters / Включено символов: `2842`
- Truncated / Обрезано: `no`

```rust
use crate::support::*;

#[tokio::test]
async fn graph_summary_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/graph/summary"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn graph_summary_accepts_secret_without_actor_header() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_without_actor(
            "/api/v1/graph/summary",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn graph_search_rejects_missing_local_api_secret_before_missing_query_validation() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/graph/search"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn graph_nodes_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/graph/nodes"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn graph_neighborhood_rejects_missing_local_api_secret_before_malformed_query_validation() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request(
            "/api/v1/graph/neighborhood?node_id=graph:node:v1:person:alex&depth=not-a-number",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}
```
