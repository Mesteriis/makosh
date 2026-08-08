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

- Chunk ID / ID чанка: `050-source-backend-part-030`
- Group / Группа: `backend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/backend.md`

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

### `backend/src/domains/persons/memory/relationship_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/memory/relationship_events.rs`
- Size bytes / Размер в байтах: `7181`
- Included characters / Включено символов: `7181`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;
use crate::domains::persons::core::{link_persons_entity, link_persons_entity_in_transaction};
use crate::engines::timeline::{TimelineEngine, TimelineEventDraft};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub id: String,
    pub person_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RelationshipEventStore {
    pool: PgPool,
}

impl RelationshipEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn timeline(
        &self,
        person_id: &str,
        limit: i64,
    ) -> Result<Vec<RelationshipEvent>, PersonMemoryError> {
        let limit = TimelineEngine::bounded_entity_limit(limit);
        let rows = sqlx::query(
            "SELECT id::text, person_id, event_type, title, description, occurred_at, source,
             related_entity_id, related_entity_kind, confidence::float8 AS confidence, metadata, created_at
             FROM relationship_events WHERE person_id = $1
             ORDER BY occurred_at DESC LIMIT $2",
        )
        .bind(person_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_event).collect()
    }

    pub async fn add(
        &self,
        event: &NewRelationshipEvent,
    ) -> Result<RelationshipEvent, PersonMemoryError> {
        TimelineEngine::validate_event(&TimelineEventDraft {
            entity_kind: "persona",
            entity_id: &event.person_id,
            event_type: &event.event_type,
            title: &event.title,
            occurred_at: event.occurred_at,
            source: &event.source,
        })?;

        let row = sqlx::query(
            "INSERT INTO relationship_events (person_id, event_type, title, description,
             occurred_at, source, related_entity_id, related_entity_kind)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id::text, person_id, event_type, title, description, occurred_at, source,
                       related_entity_id, related_entity_kind, confidence::float8 AS confidence, metadata, created_at",
        )
        .bind(&event.person_id)
        .bind(&event.event_type)
        .bind(&event.title)
        .bind(&event.description)
        .bind(event.occurred_at)
        .bind(&event.source)
        .bind(&event.related_entity_id)
        .bind(&event.related_entity_kind)
        .fetch_one(&self.pool)
        .await?;
        row_to_event(row)
    }

    pub async fn add_with_observation(
        &self,
        event: &NewRelationshipEvent,
        observation_id: &str,
    ) -> Result<RelationshipEvent, PersonMemoryError> {
        let event_record = self.add(event).await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "relationship_event",
            event_record.id.clone(),
            None,
            Some(json!({
                "person_id": event_record.person_id,
                "event_type": event_record.event_type,
            })),
        )
        .await?;
        Ok(event_record)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_email_message_event(
        &self,
        observation_id: &str,
        message_id: &str,
        occurred_at: DateTime<Utc>,
        person_id: &str,
        event_type: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<bool, PersonMemoryError> {
        TimelineEngine::validate_event(&TimelineEventDraft {
            entity_kind: "persona",
            entity_id: person_id,
            event_type,
            title,
            occurred_at,
            source: "email_sync",
        })?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO relationship_events (
                person_id,
                event_type,
                title,
                description,
                occurred_at,
                source,
                related_entity_id,
                related_entity_kind,
                metadata
            )
            SELECT
                $1,
                $2,
                $3,
                $4,
                $5,
                'email_sync',
                $6,
                'communication_message',
                '{}'::jsonb
            WHERE NOT EXISTS (
                SELECT 1
                FROM relationship_events
                WHERE person_id = $1
                  AND event_type = $2
                  AND related_entity_id = $6
                  AND related_entity_kind = 'communication_message'
            )
            RETURNING id::text AS event_id
            "#,
        )
        .bind(person_id)
        .bind(event_type)
        .bind(title)
        .bind(description)
        .bind(occurred_at)
        .bind(message_id)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            transaction.commit().await?;
            return Ok(false);
        };
        let event_id: String = row.try_get("event_id")?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "relationship_event",
            event_id,
            Some("email_sync_relationship_event"),
            Some(json!({
                "person_id": person_id,
                "event_type": event_type,
                "related_entity_id": message_id,
                "related_entity_kind": "communication_message",
                "source": "email_sync",
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(true)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewRelationshipEvent {
    pub person_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
}

fn row_to_event(row: PgRow) -> Result<RelationshipEvent, PersonMemoryError> {
    Ok(RelationshipEvent {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        event_type: row.try_get("event_type")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        occurred_at: row.try_get("occurred_at")?,
        source: row.try_get("source")?,
        related_entity_id: row.try_get("related_entity_id")?,
        related_entity_kind: row.try_get("related_entity_kind")?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}
```

### `backend/src/domains/persons/memory/snapshots.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/memory/snapshots.rs`
- Size bytes / Размер в байтах: `4746`
- Included characters / Включено символов: `4746`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonSnapshot {
    pub id: String,
    pub person_id: String,
    pub snapshot_date: DateTime<Utc>,
    pub data: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonSnapshotStore {
    pool: PgPool,
}

impl PersonSnapshotStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonSnapshot>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, snapshot_date, data, source, created_at
             FROM person_snapshots WHERE person_id = $1 ORDER BY snapshot_date DESC LIMIT 20",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_snapshot).collect()
    }

    pub async fn create(
        &self,
        person_id: &str,
        data: Value,
        source: &str,
    ) -> Result<PersonSnapshot, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_snapshots (person_id, data, source)
             VALUES ($1, $2, $3)
             RETURNING id::text, person_id, snapshot_date, data, source, created_at",
        )
        .bind(person_id)
        .bind(&data)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_snapshot(row)
    }

    pub async fn history_diff(
        &self,
        person_id: &str,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<HistoryDiff, PersonMemoryError> {
        let from = sqlx::query(
            "SELECT id::text, person_id, snapshot_date, data, source, created_at
             FROM person_snapshots WHERE person_id = $1 AND snapshot_date <= $2
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .bind(person_id)
        .bind(from_date)
        .fetch_optional(&self.pool)
        .await?;

        let to = sqlx::query(
            "SELECT id::text, person_id, snapshot_date, data, source, created_at
             FROM person_snapshots WHERE person_id = $1 AND snapshot_date <= $2
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .bind(person_id)
        .bind(to_date)
        .fetch_optional(&self.pool)
        .await?;

        let changes = snapshot_changes(&from, &to);

        Ok(HistoryDiff {
            person_id: person_id.to_string(),
            from_date: from.map(|r| r.try_get("snapshot_date").unwrap_or(from_date)),
            to_date: to.map(|r| r.try_get("snapshot_date").unwrap_or(to_date)),
            changes,
        })
    }
}

fn row_to_snapshot(row: PgRow) -> Result<PersonSnapshot, PersonMemoryError> {
    Ok(PersonSnapshot {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        snapshot_date: row.try_get("snapshot_date")?,
        data: row.try_get("data")?,
        source: row.try_get("source")?,
        created_at: row.try_get("created_at")?,
    })
}

fn snapshot_changes(from: &Option<PgRow>, to: &Option<PgRow>) -> Vec<FieldChange> {
    let mut changes: Vec<FieldChange> = Vec::new();
    if let (Some(from_row), Some(to_row)) = (from, to) {
        let from_data: Value = from_row.try_get("data").unwrap_or_default();
        let to_data: Value = to_row.try_get("data").unwrap_or_default();
        if let (Some(from_obj), Some(to_obj)) = (from_data.as_object(), to_data.as_object()) {
            for (key, to_val) in to_obj {
                let from_val = from_obj.get(key);
                if from_val != Some(to_val) {
                    changes.push(FieldChange {
                        field: key.clone(),
                        old_value: from_val.cloned(),
                        new_value: Some(to_val.clone()),
                    });
                }
            }
            for key in from_obj.keys() {
                if !to_obj.contains_key(key) {
                    changes.push(FieldChange {
                        field: key.clone(),
                        old_value: from_obj.get(key).cloned(),
                        new_value: None,
                    });
                }
            }
        }
    }
    changes
}

#[derive(Clone, Debug, Serialize)]
pub struct HistoryDiff {
    pub person_id: String,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub changes: Vec<FieldChange>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}
```

### `backend/src/domains/persons/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/mod.rs`
- Size bytes / Размер в байтах: `290`
- Included characters / Включено символов: `290`
- Truncated / Обрезано: `no`

```rust
pub mod analytics;
pub mod api;
mod command_service;
pub mod core;
pub mod enrichment;
pub mod enrichment_engine;
pub mod expertise;
pub mod export;
pub mod health;
pub mod identity;
pub mod intelligence;
pub mod investigator;
pub mod memory;
pub mod ports;
pub mod service;
pub mod trust;
```

### `backend/src/domains/persons/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/ports.rs`
- Size bytes / Размер в байтах: `139`
- Included characters / Включено символов: `139`
- Truncated / Обрезано: `no`

```rust
pub use super::api::PersonProjectionStore as PersonProjectionPort;
pub use super::memory::RelationshipEventStore as RelationshipEventPort;
```

### `backend/src/domains/persons/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/service.rs`
- Size bytes / Размер в байтах: `35`
- Included characters / Включено символов: `35`
- Truncated / Обрезано: `no`

```rust
pub use super::command_service::*;
```

### `backend/src/domains/persons/trust.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/trust.rs`
- Size bytes / Размер в байтах: `269`
- Included characters / Включено символов: `269`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod health_projection;
mod models;
mod promises;
mod risks;
mod rows;

pub use errors::PersonTrustError;
pub use models::{PersonPromise, PersonRisk};
pub use promises::{PERSON_PROMISE_CREATED_EVENT_TYPE, PersonPromiseStore};
pub use risks::PersonRiskStore;
```

### `backend/src/domains/persons/trust/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/trust/errors.rs`
- Size bytes / Размер в байтах: `440`
- Included characters / Включено символов: `440`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::EventStoreError;

#[derive(Debug, Error)]
pub enum PersonTrustError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    RiskEngine(#[from] crate::engines::risk::RiskEngineError),

    #[error(transparent)]
    Observation(#[from] crate::platform::observations::ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),
}
```

### `backend/src/domains/persons/trust/health_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/trust/health_projection.rs`
- Size bytes / Размер в байтах: `1179`
- Included characters / Включено символов: `1179`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Row, Transaction};

use crate::engines::risk::{RiskEngine, RiskSeverity, RiskSignal};

use super::errors::PersonTrustError;

pub(super) async fn sync_person_health_status_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
) -> Result<(), PersonTrustError> {
    let rows = sqlx::query(
        r#"
        SELECT severity
        FROM person_risks
        WHERE person_id = $1
          AND resolved_at IS NULL
        "#,
    )
    .bind(person_id)
    .fetch_all(&mut **transaction)
    .await?;
    let risks = rows
        .into_iter()
        .map(|row| {
            let severity: String = row.try_get("severity")?;
            Ok(RiskSignal::unresolved(RiskSeverity::parse(&severity)?))
        })
        .collect::<Result<Vec<_>, PersonTrustError>>()?;
    let health_status = RiskEngine::derive_attention_status(&risks).as_persona_health_status();

    sqlx::query(
        "UPDATE persons
         SET health_status = $2, last_health_check = now(), updated_at = now()
         WHERE person_id = $1",
    )
    .bind(person_id)
    .bind(health_status)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
```

### `backend/src/domains/persons/trust/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/trust/models.rs`
- Size bytes / Размер в байтах: `845`
- Included characters / Включено символов: `845`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPromise {
    pub id: String,
    pub person_id: String,
    pub description: String,
    pub source_message_id: Option<String>,
    pub promised_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonRisk {
    pub id: String,
    pub person_id: String,
    pub risk_type: String,
    pub description: String,
    pub severity: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}
```

### `backend/src/domains/persons/trust/promises.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/trust/promises.rs`
- Size bytes / Размер в байтах: `3565`
- Included characters / Включено символов: `3565`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::Postgres;
use sqlx::postgres::PgPool;

use crate::platform::events::{EventStore, EventStoreError, NewEventEnvelope};

use super::errors::PersonTrustError;
use super::models::PersonPromise;
use super::rows::row_to_promise;

pub const PERSON_PROMISE_CREATED_EVENT_TYPE: &str = "person.promise.created";

#[derive(Clone)]
pub struct PersonPromiseStore {
    pool: PgPool,
}

impl PersonPromiseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonPromise>, PersonTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, description, source_message_id, promised_at,
             due_at, fulfilled_at, status, created_at, updated_at
             FROM person_promises WHERE person_id = $1 ORDER BY promised_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_promise).collect()
    }

    pub async fn create(
        &self,
        person_id: &str,
        description: &str,
        due_at: Option<DateTime<Utc>>,
    ) -> Result<PersonPromise, PersonTrustError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO person_promises (person_id, description, due_at)
             VALUES ($1, $2, $3)
             RETURNING id::text, person_id, description, source_message_id, promised_at,
                       due_at, fulfilled_at, status, created_at, updated_at",
        )
        .bind(person_id)
        .bind(description)
        .bind(due_at)
        .fetch_one(&mut *transaction)
        .await?;
        let promise = row_to_promise(row)?;
        append_promise_created_event(&mut transaction, &promise).await?;
        transaction.commit().await?;

        Ok(promise)
    }

    pub async fn fulfill(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query(
            "UPDATE person_promises
             SET status = 'fulfilled', fulfilled_at = now(), updated_at = now()
             WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_broken(&self, id: &str) -> Result<(), PersonTrustError> {
        sqlx::query(
            "UPDATE person_promises SET status = 'broken', updated_at = now() WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

async fn append_promise_created_event(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    promise: &PersonPromise,
) -> Result<(), PersonTrustError> {
    let event = NewEventEnvelope::builder(
        format!("person_promise_created:{}", promise.id),
        PERSON_PROMISE_CREATED_EVENT_TYPE,
        promise.promised_at,
        json!({
            "kind": "person_promise",
            "provider": "makosh",
            "source_id": promise.id,
        }),
        json!({
            "kind": "persona",
            "person_id": &promise.person_id,
        }),
    )
    .payload(json!({
        "promise_id": &promise.id,
        "person_id": &promise.person_id,
        "description": &promise.description,
        "due_at": promise.due_at,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    match EventStore::append_in_transaction(transaction, &event).await {
        Ok(_) => Ok(()),
        Err(error) if error.is_unique_violation() => Ok(()),
        Err(error) => Err(error.into()),
    }
}
```

### `backend/src/domains/persons/trust/risks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/trust/risks.rs`
- Size bytes / Размер в байтах: `2874`
- Included characters / Включено символов: `2874`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::engines::risk::RiskEngine;

use super::errors::PersonTrustError;
use super::health_projection::sync_person_health_status_in_transaction;
use super::models::PersonRisk;
use super::rows::row_to_risk;

#[derive(Clone)]
pub struct PersonRiskStore {
    pool: PgPool,
}

impl PersonRiskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonRisk>, PersonTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, risk_type, description, severity, source, confidence::float8 AS confidence,
             created_at, resolved_at, resolution
             FROM person_risks WHERE person_id = $1 ORDER BY created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_risk).collect()
    }

    pub async fn report(
        &self,
        person_id: &str,
        risk_type: &str,
        description: &str,
        severity: &str,
        source: &str,
    ) -> Result<PersonRisk, PersonTrustError> {
        let observation =
            RiskEngine::persona_observation(person_id, risk_type, description, severity, source)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO person_risks (person_id, risk_type, description, severity, source, confidence)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id::text, person_id, risk_type, description, severity, source, confidence::float8 AS confidence,
                       created_at, resolved_at, resolution",
        )
        .bind(&observation.affected_entity_id)
        .bind(&observation.risk_type)
        .bind(&observation.evidence)
        .bind(observation.severity.as_str())
        .bind(&observation.source)
        .bind(observation.confidence)
        .fetch_one(&mut *transaction)
        .await?;
        let risk = row_to_risk(row)?;
        sync_person_health_status_in_transaction(&mut transaction, person_id).await?;
        transaction.commit().await?;
        Ok(risk)
    }

    pub async fn resolve(&self, id: &str, resolution: &str) -> Result<(), PersonTrustError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE person_risks
             SET resolved_at = now(), resolution = $2
             WHERE id::text = $1
             RETURNING person_id",
        )
        .bind(id)
        .bind(resolution)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let person_id: String = row.try_get("person_id")?;
            sync_person_health_status_in_transaction(&mut transaction, &person_id).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
```

### `backend/src/domains/persons/trust/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/trust/rows.rs`
- Size bytes / Размер в байтах: `1277`
- Included characters / Включено символов: `1277`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonTrustError;
use super::models::{PersonPromise, PersonRisk};

pub(super) fn row_to_promise(row: PgRow) -> Result<PersonPromise, PersonTrustError> {
    Ok(PersonPromise {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        description: row.try_get("description")?,
        source_message_id: row.try_get("source_message_id")?,
        promised_at: row.try_get("promised_at")?,
        due_at: row.try_get("due_at")?,
        fulfilled_at: row.try_get("fulfilled_at")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_risk(row: PgRow) -> Result<PersonRisk, PersonTrustError> {
    Ok(PersonRisk {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        risk_type: row.try_get("risk_type")?,
        description: row.try_get("description")?,
        severity: row.try_get("severity")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        created_at: row.try_get("created_at")?,
        resolved_at: row.try_get("resolved_at")?,
        resolution: row.try_get("resolution")?,
    })
}
```

### `backend/src/domains/projects/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core.rs`
- Size bytes / Размер в байтах: `634`
- Included characters / Включено символов: `634`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod errors;
mod ids;
mod models;
mod projection;
mod read_model;
mod rows;
mod store;
mod validation;

pub use errors::ProjectStoreError;
pub use errors::ProjectStoreError as ProjectCommandPortError;
pub use ids::project_graph_node_id;
pub use models::{
    NewProject, Project, ProjectDetail, ProjectDocumentSummary, ProjectListResponse,
    ProjectMessageSummary, ProjectPersonSummary, ProjectStats, ProjectSummary, ProjectTimelineItem,
};
pub(crate) use models::{ProjectMatchedDocument, ProjectMatchedMessage, ProjectProjectionSource};
pub use store::ProjectStore;
pub use store::ProjectStore as ProjectCommandPort;
```

### `backend/src/domains/projects/core/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/constants.rs`
- Size bytes / Размер в байтах: `150`
- Included characters / Включено символов: `150`
- Truncated / Обрезано: `no`

```rust
pub(super) const DEFAULT_PROJECT_LIMIT: i64 = 25;
pub(super) const MAX_PROJECT_LIMIT: i64 = 100;
pub(super) const PROJECT_DETAIL_ITEM_LIMIT: i64 = 8;
```

### `backend/src/domains/projects/core/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/errors.rs`
- Size bytes / Размер в байтах: `662`
- Included characters / Включено символов: `662`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("project progress_percent must be between 0 and 100: {0}")]
    InvalidProgress(i32),

    #[error("project must have at least one keyword")]
    NoKeywords,

    #[error(transparent)]
    ProjectLinkReview(#[from] crate::domains::projects::link_reviews::ProjectLinkReviewError),

    #[error("project limit must be positive")]
    InvalidLimit,

    #[error("project message recipients must be a JSON array of strings")]
    InvalidRecipients,
}
```

### `backend/src/domains/projects/core/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/ids.rs`
- Size bytes / Размер в байтах: `111`
- Included characters / Включено символов: `111`
- Truncated / Обрезано: `no`

```rust
pub fn project_graph_node_id(project_id: &str) -> String {
    format!("graph:node:v1:project:{project_id}")
}
```

### `backend/src/domains/projects/core/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/models.rs`
- Size bytes / Размер в байтах: `6437`
- Included characters / Включено символов: `6437`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeSet;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::domains::projects::link_reviews::ProjectLinkReviewState;

use super::errors::ProjectStoreError;
use super::validation::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProject {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub description: String,
    pub owner_display_name: String,
    pub progress_percent: i32,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub keywords: Vec<String>,
}

impl NewProject {
    pub fn active(
        project_id: impl Into<String>,
        name: impl Into<String>,
        kind: impl Into<String>,
        description: impl Into<String>,
        owner_display_name: impl Into<String>,
        keywords: Vec<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            name: name.into(),
            kind: kind.into(),
            status: "active".to_owned(),
            description: description.into(),
            owner_display_name: owner_display_name.into(),
            progress_percent: 0,
            start_date: None,
            target_date: None,
            keywords,
        }
    }

    pub fn progress(mut self, progress_percent: i32) -> Self {
        self.progress_percent = progress_percent;
        self
    }

    pub(super) fn validate(&self) -> Result<ValidatedProject, ProjectStoreError> {
        let project_id = validate_non_empty("project_id", &self.project_id)?;
        let name = validate_non_empty("name", &self.name)?;
        let kind = validate_non_empty("kind", &self.kind)?;
        let status = validate_non_empty("status", &self.status)?;
        let description = validate_non_empty("description", &self.description)?;
        let owner_display_name =
            validate_non_empty("owner_display_name", &self.owner_display_name)?;
        if !(0..=100).contains(&self.progress_percent) {
            return Err(ProjectStoreError::InvalidProgress(self.progress_percent));
        }

        let mut seen = BTreeSet::new();
        let mut keywords = Vec::new();
        for keyword in &self.keywords {
            let keyword = validate_non_empty("keyword", keyword)?;
            if seen.insert(keyword.to_ascii_lowercase()) {
                keywords.push(keyword);
            }
        }
        if keywords.is_empty() {
            return Err(ProjectStoreError::NoKeywords);
        }

        Ok(ValidatedProject {
            project_id,
            name,
            kind,
            status,
            description,
            owner_display_name,
            progress_percent: self.progress_percent,
            start_date: self.start_date,
            target_date: self.target_date,
            keywords,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ValidatedProject {
    pub(super) project_id: String,
    pub(super) name: String,
    pub(super) kind: String,
    pub(super) status: String,
    pub(super) description: String,
    pub(super) owner_display_name: String,
    pub(super) progress_percent: i32,
    pub(super) start_date: Option<NaiveDate>,
    pub(super) target_date: Option<NaiveDate>,
    pub(super) keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub description: String,
    pub owner_display_name: String,
    pub progress_percent: i32,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectStats {
    pub message_count: i64,
    pub document_count: i64,
    pub people_count: i64,
    pub graph_connection_count: i64,
    pub latest_activity_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectSummary {
    pub project: Project,
    pub stats: ProjectStats,
    pub graph_node_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectDetail {
    pub project: Project,
    pub stats: ProjectStats,
    pub graph_node_id: String,
    pub timeline: Vec<ProjectTimelineItem>,
    pub key_people: Vec<ProjectPersonSummary>,
    pub recent_messages: Vec<ProjectMessageSummary>,
    pub documents: Vec<ProjectDocumentSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectTimelineItem {
    pub item_kind: String,
    pub item_id: String,
    pub title: String,
    pub subtitle: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectPersonSummary {
    pub display_name: String,
    pub email_address: String,
    pub interaction_count: i64,
    pub last_interaction_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectMessageSummary {
    pub message_id: String,
    pub subject: String,
    pub sender: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectDocumentSummary {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub observation_id: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectListResponse {
    pub items: Vec<ProjectSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectProjectionSource {
    pub project: Project,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMatchedMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub observation_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub review_state: ProjectLinkReviewState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMatchedDocument {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub observation_id: String,
    pub source_fingerprint: String,
    pub imported_at: DateTime<Utc>,
    pub review_state: ProjectLinkReviewState,
}
```

### `backend/src/domains/projects/core/projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/projection.rs`
- Size bytes / Размер в байтах: `4540`
- Included characters / Включено символов: `4540`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashMap;

use crate::domains::projects::link_reviews::{ProjectLinkReviewState, ProjectReviewedTarget};

use super::errors::ProjectStoreError;
use super::models::{ProjectMatchedDocument, ProjectMatchedMessage, ProjectProjectionSource};
use super::rows::{row_to_matched_document, row_to_matched_message};
use super::store::ProjectStore;

impl ProjectStore {
    pub(crate) async fn graph_projection_projects(
        &self,
    ) -> Result<Vec<ProjectProjectionSource>, ProjectStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date,
                created_at,
                updated_at
            FROM projects
            ORDER BY project_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut projects = Vec::new();
        for row in rows {
            let project = super::rows::row_to_project(row)?;
            projects.push(ProjectProjectionSource {
                keywords: self.project_keywords(&project.project_id).await?,
                project,
            });
        }

        Ok(projects)
    }

    pub(crate) async fn matching_project_messages(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMatchedMessage>, ProjectStoreError> {
        let reviewed = self.active_project_messages(project_id).await?;
        if reviewed.is_empty() {
            return Ok(Vec::new());
        }
        let (message_ids, reviewed_by_id) = reviewed_targets_and_map(reviewed);

        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                occurred_at,
                projected_at
            FROM communication_messages message
            WHERE message_id = ANY($1)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            "#,
        )
        .bind(&message_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            let mut message = row_to_matched_message(row)?;
            message.review_state = reviewed_by_id
                .get(&message.message_id)
                .copied()
                .unwrap_or(ProjectLinkReviewState::Suggested);
            messages.push(message);
        }

        Ok(messages)
    }

    pub(crate) async fn matching_project_documents(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMatchedDocument>, ProjectStoreError> {
        let reviewed = self.active_project_documents(project_id).await?;
        if reviewed.is_empty() {
            return Ok(Vec::new());
        }
        let (document_ids, reviewed_by_id) = reviewed_targets_and_map(reviewed);

        let rows = sqlx::query(
            r#"
            SELECT document_id, document_kind, title, observation_id, source_fingerprint, imported_at
            FROM documents document
            WHERE document_id = ANY($1)
            ORDER BY imported_at DESC, document_id
            "#,
        )
        .bind(&document_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::with_capacity(rows.len());
        for row in rows {
            let mut document = row_to_matched_document(row)?;
            document.review_state = reviewed_by_id
                .get(&document.document_id)
                .copied()
                .unwrap_or(ProjectLinkReviewState::Suggested);
            documents.push(document);
        }

        Ok(documents)
    }
}

pub(super) fn reviewed_targets_and_map(
    targets: Vec<ProjectReviewedTarget>,
) -> (Vec<String>, HashMap<String, ProjectLinkReviewState>) {
    let mut ids = Vec::with_capacity(targets.len());
    let mut map = HashMap::with_capacity(targets.len());
    for target in targets {
        map.insert(target.target_id.clone(), target.review_state);
        ids.push(target.target_id);
    }

    (ids, map)
}

pub(super) fn reviewed_target_ids(targets: &[ProjectReviewedTarget]) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.target_id.clone())
        .collect()
}
```

### `backend/src/domains/projects/core/read_model.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model.rs`
- Size bytes / Размер в байтах: `116`
- Included characters / Включено символов: `116`
- Truncated / Обрезано: `no`

```rust
mod documents;
mod keywords;
mod messages;
mod people;
mod projects;
mod reviewed_targets;
mod stats;
mod timeline;
```

### `backend/src/domains/projects/core/read_model/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/documents.rs`
- Size bytes / Размер в байтах: `1110`
- Included characters / Включено символов: `1110`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::ProjectStoreError;
use super::super::models::ProjectDocumentSummary;
use super::super::projection::reviewed_target_ids;
use super::super::rows::row_to_project_document;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_documents(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectDocumentSummary>, ProjectStoreError> {
        let document_ids = reviewed_target_ids(&self.active_project_documents(project_id).await?);
        if document_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT document_id, document_kind, title, observation_id, imported_at
            FROM documents document
            WHERE document_id = ANY($1)
            ORDER BY imported_at DESC, document_id
            LIMIT $2
            "#,
        )
        .bind(&document_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_project_document).collect()
    }
}
```

### `backend/src/domains/projects/core/read_model/keywords.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/keywords.rs`
- Size bytes / Размер в байтах: `584`
- Included characters / Включено символов: `584`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::ProjectStoreError;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_keywords(
        &self,
        project_id: &str,
    ) -> Result<Vec<String>, ProjectStoreError> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT keyword
            FROM project_keywords
            WHERE project_id = $1
            ORDER BY keyword
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
```

### `backend/src/domains/projects/core/read_model/messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/messages.rs`
- Size bytes / Размер в байтах: `1192`
- Included characters / Включено символов: `1192`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::ProjectStoreError;
use super::super::models::ProjectMessageSummary;
use super::super::projection::reviewed_target_ids;
use super::super::rows::row_to_project_message;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_messages(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectMessageSummary>, ProjectStoreError> {
        let message_ids = reviewed_target_ids(&self.active_project_messages(project_id).await?);
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                subject,
                sender,
                COALESCE(occurred_at, projected_at) AS occurred_at
            FROM communication_messages message
            WHERE message_id = ANY($1)
            ORDER BY occurred_at DESC, message_id
            LIMIT $2
            "#,
        )
        .bind(&message_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_project_message).collect()
    }
}
```

### `backend/src/domains/projects/core/read_model/people.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/people.rs`
- Size bytes / Размер в байтах: `2147`
- Included characters / Включено символов: `2147`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::ProjectStoreError;
use super::super::models::ProjectPersonSummary;
use super::super::projection::reviewed_target_ids;
use super::super::rows::row_to_project_person;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_people(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<ProjectPersonSummary>, ProjectStoreError> {
        let message_ids = reviewed_target_ids(&self.active_project_messages(project_id).await?);
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            WITH project_messages AS (
                SELECT sender, recipients, COALESCE(occurred_at, projected_at) AS occurred_at
                FROM communication_messages message
                WHERE message_id = ANY($1)
            ),
            participants AS (
                SELECT lower(trim(sender)) AS email_address, occurred_at
                FROM project_messages
                UNION ALL
                SELECT lower(trim(recipient.value)) AS email_address, message.occurred_at
                FROM project_messages message,
                     jsonb_array_elements_text(message.recipients) AS recipient(value)
            )
            SELECT
                COALESCE(person.display_name, participants.email_address) AS display_name,
                participants.email_address,
                count(*)::BIGINT AS interaction_count,
                max(participants.occurred_at) AS last_interaction_at
            FROM participants
            LEFT JOIN persons person ON person.email_address = participants.email_address
            WHERE participants.email_address <> ''
            GROUP BY participants.email_address, person.display_name
            ORDER BY interaction_count DESC, last_interaction_at DESC NULLS LAST, display_name
            LIMIT $2
            "#,
        )
        .bind(&message_ids)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_project_person).collect()
    }
}
```

### `backend/src/domains/projects/core/read_model/projects.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/projects.rs`
- Size bytes / Размер в байтах: `933`
- Included characters / Включено символов: `933`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::ProjectStoreError;
use super::super::models::Project;
use super::super::rows::row_to_project;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn project_by_id(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date,
                created_at,
                updated_at
            FROM projects
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_project).transpose()
    }
}
```

### `backend/src/domains/projects/core/read_model/reviewed_targets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/projects/core/read_model/reviewed_targets.rs`
- Size bytes / Размер в байтах: `960`
- Included characters / Включено символов: `960`
- Truncated / Обрезано: `no`

```rust
use crate::domains::projects::link_reviews::{ProjectLinkReviewStore, ProjectReviewedTarget};

use super::super::errors::ProjectStoreError;
use super::super::store::ProjectStore;

impl ProjectStore {
    pub(in crate::domains::projects::core) async fn active_project_messages(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_message_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }

    pub(in crate::domains::projects::core) async fn active_project_documents(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectStoreError> {
        ProjectLinkReviewStore::new(self.pool.clone())
            .active_document_ids_for_project(project_id)
            .await
            .map_err(ProjectStoreError::ProjectLinkReview)
    }
}
```
