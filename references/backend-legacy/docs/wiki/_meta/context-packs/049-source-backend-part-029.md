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

- Chunk ID / ID чанка: `049-source-backend-part-029`
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

### `backend/src/domains/persons/identity/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/models.rs`
- Size bytes / Размер в байтах: `3800`
- Included characters / Включено символов: `3800`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::constants::PERSON_IDENTITY_ID_PREFIX;
use super::errors::PersonIdentityError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonIdentityCandidateKind {
    MergePersons,
    AttachEmailAddress,
    SplitPerson,
}

impl PersonIdentityCandidateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MergePersons => "merge_persons",
            Self::AttachEmailAddress => "attach_email_address",
            Self::SplitPerson => "split_person",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PersonIdentityReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl PersonIdentityReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub(super) fn parse(value: impl AsRef<str>) -> Result<Self, PersonIdentityError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(PersonIdentityError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonIdentityReviewCommand {
    pub command_id: String,
    pub identity_candidate_id: String,
    pub review_state: PersonIdentityReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonIdentityReviewCommandResult {
    pub identity_candidate_id: String,
    pub review_state: PersonIdentityReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonIdentityCandidate {
    pub identity_candidate_id: String,
    pub candidate_kind: String,
    pub left_person_id: String,
    pub right_person_id: Option<String>,
    pub email_address: Option<String>,
    pub evidence_summary: String,
    pub confidence: f64,
    pub review_state: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonIdentityDetail {
    pub items: Vec<PersonIdentityCandidate>,
}

#[derive(Debug)]
pub(crate) struct PersonIdentityCandidatePayload {
    pub(crate) candidate_kind: PersonIdentityCandidateKind,
    pub(crate) left_person_id: String,
    pub(crate) right_person_id: Option<String>,
    pub(crate) email_address: Option<String>,
    pub(crate) evidence_summary: String,
    pub(crate) confidence: f64,
}

impl PersonIdentityCandidatePayload {
    pub(crate) fn identity_candidate_id(&self) -> String {
        let left = self.left_person_id.clone();
        let right = self
            .right_person_id
            .clone()
            .unwrap_or_else(|| String::from("single"));

        match self.candidate_kind {
            PersonIdentityCandidateKind::MergePersons => {
                format!("{PERSON_IDENTITY_ID_PREFIX}merge_persons:{left}:{right}")
            }
            PersonIdentityCandidateKind::AttachEmailAddress => {
                let email = self
                    .email_address
                    .clone()
                    .unwrap_or_else(|| String::from("missing"));
                format!(
                    "{PERSON_IDENTITY_ID_PREFIX}attach_email_address:{left}:{}:{email}",
                    email.len()
                )
            }
            PersonIdentityCandidateKind::SplitPerson => {
                format!("{PERSON_IDENTITY_ID_PREFIX}split_person:{left}:{right}")
            }
        }
    }
}
```

### `backend/src/domains/persons/identity/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/rows.rs`
- Size bytes / Размер в байтах: `898`
- Included characters / Включено символов: `898`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonIdentityError;
use super::models::PersonIdentityCandidate;

pub(super) fn row_to_person_identity_candidate(
    row: PgRow,
) -> Result<PersonIdentityCandidate, PersonIdentityError> {
    Ok(PersonIdentityCandidate {
        identity_candidate_id: row.try_get("identity_candidate_id")?,
        candidate_kind: row.try_get("candidate_kind")?,
        left_person_id: row.try_get("left_person_id")?,
        right_person_id: row.try_get("right_person_id")?,
        email_address: row.try_get("email_address")?,
        evidence_summary: row.try_get("evidence_summary")?,
        confidence: row.try_get("confidence")?,
        review_state: row.try_get("review_state")?,
        generated_at: row.try_get("generated_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/persons/identity/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/store.rs`
- Size bytes / Размер в байтах: `374`
- Included characters / Включено символов: `374`
- Truncated / Обрезано: `no`

```rust
mod candidates;
mod name_merge_candidates;
mod queries;
mod review;
mod review_state;
mod split_candidates;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonIdentityStore {
    pool: PgPool,
}

impl PersonIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
```

### `backend/src/domains/persons/identity/store/candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/store/candidates.rs`
- Size bytes / Размер в байтах: `3070`
- Included characters / Включено символов: `3070`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::PersonIdentityError;
use super::super::models::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};
use super::super::upsert::upsert_candidate;
use super::super::validation::validate_limit;
use super::PersonIdentityStore;
use super::name_merge_candidates::refresh_name_merge_candidates;
use super::split_candidates::refresh_split_candidates;
use sqlx::Row;

impl PersonIdentityStore {
    pub async fn refresh_candidates(&self, limit: i64) -> Result<usize, PersonIdentityError> {
        let limit = validate_limit(limit)?;
        let merge_count = refresh_name_merge_candidates(self.pool(), limit).await?;
        let split_count = refresh_split_candidates(self.pool(), limit).await?;

        Ok(merge_count + split_count)
    }

    pub async fn suggest_attach_email_candidates(
        &self,
        display_name: &str,
        email_address: &str,
        evidence_summary: &str,
        confidence: f64,
        limit: i64,
    ) -> Result<usize, PersonIdentityError> {
        let limit = validate_limit(limit)?;
        let normalized_display_name = display_name.trim().to_ascii_lowercase();
        let normalized_email = email_address.trim().to_ascii_lowercase();
        if normalized_display_name.is_empty()
            || normalized_email.is_empty()
            || !normalized_email.contains('@')
        {
            return Ok(0);
        }

        let rows = sqlx::query(
            r#"
            SELECT person.person_id
            FROM persons person
            WHERE lower(trim(person.display_name)) = $1
              AND position('@' in lower(trim(person.display_name))) = 0
              AND NOT EXISTS (
                    SELECT 1
                    FROM person_identities identity_trace
                    WHERE identity_trace.person_id = person.person_id
                      AND identity_trace.identity_type = 'email'
                      AND lower(trim(identity_trace.identity_value)) = $2
                      AND identity_trace.status = 'active'
              )
            ORDER BY person.person_id ASC
            LIMIT $3
            "#,
        )
        .bind(&normalized_display_name)
        .bind(&normalized_email)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;

        let mut count = 0usize;
        for row in rows {
            let candidate = PersonIdentityCandidatePayload {
                candidate_kind: PersonIdentityCandidateKind::AttachEmailAddress,
                left_person_id: row.try_get("person_id")?,
                right_person_id: None,
                email_address: Some(normalized_email.clone()),
                evidence_summary: evidence_summary.trim().to_owned(),
                confidence,
            };
            upsert_candidate(
                self.pool(),
                &candidate,
                candidate.identity_candidate_id(),
                PersonIdentityReviewState::Suggested,
            )
            .await?;
            count += 1;
        }

        Ok(count)
    }
}
```

### `backend/src/domains/persons/identity/store/name_merge_candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/store/name_merge_candidates.rs`
- Size bytes / Размер в байтах: `1958`
- Included characters / Включено символов: `1958`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::super::errors::PersonIdentityError;
use super::super::models::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};
use super::super::upsert::upsert_candidate;

pub(super) async fn refresh_name_merge_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, PersonIdentityError> {
    let rows = sqlx::query(
        r#"
        SELECT
            c1.person_id AS left_person_id,
            c2.person_id AS right_person_id,
            lower(trim(c1.display_name)) AS normalized_display_name
        FROM persons c1
        JOIN persons c2
            ON c1.person_id < c2.person_id
           AND lower(trim(c1.display_name)) = lower(trim(c2.display_name))
        WHERE position('@' in lower(trim(c1.display_name))) = 0
          AND position('@' in lower(trim(c2.display_name))) = 0
        ORDER BY
            lower(trim(c1.display_name)),
            c1.person_id,
            c2.person_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let left = row.try_get::<String, _>("left_person_id")?;
        let right = row.try_get::<String, _>("right_person_id")?;
        let candidate = PersonIdentityCandidatePayload {
            candidate_kind: PersonIdentityCandidateKind::MergePersons,
            left_person_id: left,
            right_person_id: Some(right),
            email_address: None,
            evidence_summary: format!(
                "Same normalized display name: {}",
                row.try_get::<String, _>("normalized_display_name")?
            ),
            confidence: 0.72,
        };
        upsert_candidate(
            pool,
            &candidate,
            candidate.identity_candidate_id(),
            PersonIdentityReviewState::Suggested,
        )
        .await?;
        count += 1;
    }

    Ok(count)
}
```

### `backend/src/domains/persons/identity/store/queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/store/queries.rs`
- Size bytes / Размер в байтах: `3038`
- Included characters / Включено символов: `3038`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::PersonIdentityError;
use super::super::models::{PersonIdentityCandidate, PersonIdentityDetail};
use super::super::rows::row_to_person_identity_candidate;
use super::super::validation::{validate_non_empty, validate_optional_limit};
use super::PersonIdentityStore;

impl PersonIdentityStore {
    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<PersonIdentityCandidate>, PersonIdentityError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_person_id,
                right_person_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM person_identity_candidates
            ORDER BY updated_at DESC, identity_candidate_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(row_to_person_identity_candidate)
            .collect()
    }

    pub async fn person_identity(
        &self,
        person_id: &str,
    ) -> Result<PersonIdentityDetail, PersonIdentityError> {
        let person_id = validate_non_empty("person_id", person_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                identity_candidate_id,
                candidate_kind,
                left_person_id,
                right_person_id,
                email_address,
                evidence_summary,
                confidence,
                review_state,
                generated_at,
                reviewed_at,
                updated_at
            FROM person_identity_candidates merge
            WHERE (merge.left_person_id = $1 OR merge.right_person_id = $1)
              AND merge.candidate_kind = 'merge_persons'
              AND merge.review_state = 'user_confirmed'
              AND NOT EXISTS (
                  SELECT 1
                  FROM person_identity_candidates split
                  WHERE split.candidate_kind = 'split_person'
                    AND split.review_state = 'user_confirmed'
                    AND LEAST(split.left_person_id, split.right_person_id) =
                        LEAST(merge.left_person_id, merge.right_person_id)
                    AND GREATEST(split.left_person_id, split.right_person_id) =
                        GREATEST(merge.left_person_id, merge.right_person_id)
              )
            ORDER BY updated_at DESC, identity_candidate_id
            "#,
        )
        .bind(&person_id)
        .fetch_all(self.pool())
        .await?;

        let items = rows
            .into_iter()
            .map(row_to_person_identity_candidate)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PersonIdentityDetail { items })
    }
}
```

### `backend/src/domains/persons/identity/store/review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/store/review.rs`
- Size bytes / Размер в байтах: `4889`
- Included characters / Включено символов: `4889`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::Value;

use crate::platform::events::{EventEnvelope, EventStore};
use crate::platform::observations::materialize_review_transition_link_in_transaction;

use super::super::constants::{PERSON_IDENTITY_REVIEW_EVENT_TYPE, PERSON_IDENTITY_REVIEW_PREFIX};
use super::super::errors::PersonIdentityError;
use super::super::events::{ReviewCommandEvent, ReviewEvent};
use super::super::models::{
    PersonIdentityReviewCommand, PersonIdentityReviewCommandResult, PersonIdentityReviewState,
};
use super::super::validation::validate_non_empty;
use super::PersonIdentityStore;
use super::review_state::{apply_review_state_in_transaction, ensure_candidate_exists};
use super::split_candidates::materialize_split_candidate_for_confirmed_merge_in_transaction;

impl PersonIdentityStore {
    pub async fn set_review_state(
        &self,
        command: &PersonIdentityReviewCommand,
    ) -> Result<PersonIdentityReviewCommandResult, PersonIdentityError> {
        self.set_review_state_with_observation(command, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        command: &PersonIdentityReviewCommand,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<PersonIdentityReviewCommandResult, PersonIdentityError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let identity_candidate_id =
            validate_non_empty("identity_candidate_id", &command.identity_candidate_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool().begin().await?;
        ensure_candidate_exists(&mut transaction, &identity_candidate_id).await?;

        let event_id = format!("{PERSON_IDENTITY_REVIEW_PREFIX}{command_id}");
        let event = ReviewCommandEvent {
            command_id,
            identity_candidate_id: identity_candidate_id.clone(),
            review_state: command.review_state,
            actor_id: actor_id.clone(),
            event_id: event_id.clone(),
            occurred_at: Utc::now(),
        }
        .to_event()?;

        EventStore::append_in_transaction(&mut transaction, &event).await?;
        apply_review_state_in_transaction(
            &mut transaction,
            &identity_candidate_id,
            command.review_state,
            &event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;
        materialize_split_candidate_for_confirmed_merge_in_transaction(
            &mut transaction,
            &identity_candidate_id,
            command.review_state,
        )
        .await?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "persons",
            "identity_candidate",
            &identity_candidate_id,
            "review_state",
            command.review_state.as_str(),
            metadata
                .map(|extra| {
                    serde_json::json!({
                        "event_id": event_id,
                        "context": extra,
                    })
                })
                .or_else(|| {
                    Some(serde_json::json!({
                        "event_id": event_id,
                    }))
                }),
        )
        .await?;

        transaction.commit().await?;

        Ok(PersonIdentityReviewCommandResult {
            identity_candidate_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), PersonIdentityError> {
        if event.event_type != PERSON_IDENTITY_REVIEW_EVENT_TYPE {
            return Err(PersonIdentityError::InvalidEventType);
        }

        let parsed = ReviewEvent::from_payload(&event.payload)?;
        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(serde_json::Value::as_str)
            .ok_or(PersonIdentityError::MissingActorId)?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;
        let mut transaction = self.pool().begin().await?;
        ensure_candidate_exists(&mut transaction, &parsed.identity_candidate_id).await?;
        apply_review_state_in_transaction(
            &mut transaction,
            &parsed.identity_candidate_id,
            parsed.review_state,
            &event.event_id,
            &actor_id,
            event.occurred_at,
        )
        .await?;
        materialize_split_candidate_for_confirmed_merge_in_transaction(
            &mut transaction,
            &parsed.identity_candidate_id,
            parsed.review_state,
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }
}
```

### `backend/src/domains/persons/identity/store/review_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/store/review_state.rs`
- Size bytes / Размер в байтах: `2413`
- Included characters / Включено символов: `2413`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::super::errors::PersonIdentityError;
use super::super::models::PersonIdentityReviewState;

pub(super) async fn apply_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonIdentityReviewState,
    event_id: &str,
    actor_id: &str,
    reviewed_at: DateTime<Utc>,
) -> Result<(), PersonIdentityError> {
    match review_state {
        PersonIdentityReviewState::Suggested => {
            sqlx::query(
                r#"
                UPDATE person_identity_candidates
                SET
                    review_state = $1,
                    event_id = NULL,
                    actor_id = NULL,
                    reviewed_at = NULL,
                    updated_at = now()
                WHERE identity_candidate_id = $2
                "#,
            )
            .bind(review_state.as_str())
            .bind(identity_candidate_id)
            .execute(&mut **transaction)
            .await?;
        }
        PersonIdentityReviewState::UserConfirmed | PersonIdentityReviewState::UserRejected => {
            sqlx::query(
                r#"
                UPDATE person_identity_candidates
                SET
                    review_state = $1,
                    event_id = $2,
                    actor_id = $3,
                    reviewed_at = $4,
                    updated_at = now()
                WHERE identity_candidate_id = $5
                "#,
            )
            .bind(review_state.as_str())
            .bind(event_id)
            .bind(actor_id)
            .bind(reviewed_at)
            .bind(identity_candidate_id)
            .execute(&mut **transaction)
            .await?;
        }
    }

    Ok(())
}

pub(super) async fn ensure_candidate_exists(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
) -> Result<(), PersonIdentityError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM person_identity_candidates
            WHERE identity_candidate_id = $1
        )
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&mut **transaction)
    .await?;

    if !exists {
        return Err(PersonIdentityError::IdentityCandidateNotFound);
    }

    Ok(())
}
```

### `backend/src/domains/persons/identity/store/split_candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/store/split_candidates.rs`
- Size bytes / Размер в байтах: `3364`
- Included characters / Включено символов: `3364`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};

use super::super::errors::PersonIdentityError;
use super::super::models::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};
use super::super::upsert::{upsert_candidate, upsert_candidate_in_transaction};

pub(super) async fn refresh_split_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, PersonIdentityError> {
    let rows = sqlx::query(
        r#"
        SELECT
            merge.left_person_id,
            merge.right_person_id
        FROM person_identity_candidates merge
        WHERE merge.candidate_kind = 'merge_persons'
          AND merge.review_state = 'user_confirmed'
          AND merge.right_person_id IS NOT NULL
          AND NOT EXISTS (
              SELECT 1
              FROM person_identity_candidates split
              WHERE split.candidate_kind = 'split_person'
                AND split.left_person_id = merge.left_person_id
                AND split.right_person_id = merge.right_person_id
          )
        ORDER BY merge.updated_at DESC, merge.identity_candidate_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let left = row.try_get::<String, _>("left_person_id")?;
        let right = row.try_get::<String, _>("right_person_id")?;
        let candidate = split_candidate_payload(left, right);
        upsert_candidate(
            pool,
            &candidate,
            candidate.identity_candidate_id(),
            PersonIdentityReviewState::Suggested,
        )
        .await?;
        count += 1;
    }

    Ok(count)
}

pub(super) async fn materialize_split_candidate_for_confirmed_merge_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    if review_state != PersonIdentityReviewState::UserConfirmed {
        return Ok(());
    }

    let row = sqlx::query(
        r#"
        SELECT candidate_kind, left_person_id, right_person_id
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&mut **transaction)
    .await?;

    let candidate_kind = row.try_get::<String, _>("candidate_kind")?;
    if candidate_kind != PersonIdentityCandidateKind::MergePersons.as_str() {
        return Ok(());
    }

    let left = row.try_get::<String, _>("left_person_id")?;
    let Some(right) = row.try_get::<Option<String>, _>("right_person_id")? else {
        return Ok(());
    };
    let candidate = split_candidate_payload(left, right);
    upsert_candidate_in_transaction(
        transaction,
        &candidate,
        candidate.identity_candidate_id(),
        PersonIdentityReviewState::Suggested,
    )
    .await
}

fn split_candidate_payload(left: String, right: String) -> PersonIdentityCandidatePayload {
    PersonIdentityCandidatePayload {
        candidate_kind: PersonIdentityCandidateKind::SplitPerson,
        left_person_id: left.clone(),
        right_person_id: Some(right.clone()),
        email_address: None,
        evidence_summary: format!("Previously confirmed merge can be split: {left} and {right}"),
        confidence: 1.0,
    }
}
```

### `backend/src/domains/persons/identity/upsert.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/upsert.rs`
- Size bytes / Размер в байтах: `7630`
- Included characters / Включено символов: `7630`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};
use uuid::Uuid;

use super::errors::PersonIdentityError;
use super::models::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};
use crate::platform::events::{EventStore, NewEventEnvelope};

const PERSON_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE: &str = "person_identity.candidate.detected";

pub(super) async fn upsert_candidate(
    pool: &PgPool,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    let mut transaction = pool.begin().await?;
    upsert_candidate_in_transaction(
        &mut transaction,
        payload,
        identity_candidate_id,
        review_state,
    )
    .await?;
    transaction.commit().await?;

    Ok(())
}

pub(crate) fn person_identity_candidate_detected_event_type() -> &'static str {
    PERSON_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE
}

pub(super) async fn upsert_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: String,
    review_state: PersonIdentityReviewState,
) -> Result<(), PersonIdentityError> {
    let stored_review_state: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_identity_candidates (
            identity_candidate_id,
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            evidence_summary,
            confidence,
            review_state,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL, NULL)
        ON CONFLICT (identity_candidate_id)
        DO UPDATE SET
            candidate_kind = EXCLUDED.candidate_kind,
            left_person_id = EXCLUDED.left_person_id,
            right_person_id = EXCLUDED.right_person_id,
            email_address = EXCLUDED.email_address,
            evidence_summary = EXCLUDED.evidence_summary,
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            event_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN person_identity_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN person_identity_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        RETURNING review_state
        "#,
    )
    .bind(&identity_candidate_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.left_person_id)
    .bind(&payload.right_person_id)
    .bind(&payload.email_address)
    .bind(&payload.evidence_summary)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .fetch_one(&mut **transaction)
    .await?;

    append_candidate_detected_event(
        transaction,
        payload,
        &identity_candidate_id,
        &stored_review_state,
    )
    .await?;

    Ok(())
}

async fn append_candidate_detected_event(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonIdentityCandidatePayload,
    identity_candidate_id: &str,
    review_state: &str,
) -> Result<(), PersonIdentityError> {
    let event_instance_id = Uuid::now_v7();
    let event = NewEventEnvelope::builder(
        format!(
            "person_identity_candidate_detected:{identity_candidate_id}:{}",
            event_instance_id
        ),
        PERSON_IDENTITY_CANDIDATE_DETECTED_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "person_identity",
            "provider": "makosh",
            "source_id": format!("{identity_candidate_id}:{event_instance_id}"),
        }),
        json!({
            "kind": "person_identity_candidate",
            "identity_candidate_id": identity_candidate_id,
        }),
    )
    .payload(json!({
        "identity_candidate_id": identity_candidate_id,
        "candidate_kind": payload.candidate_kind.as_str(),
        "left_person_id": &payload.left_person_id,
        "right_person_id": &payload.right_person_id,
        "email_address": &payload.email_address,
        "evidence_summary": &payload.evidence_summary,
        "confidence": payload.confidence,
        "review_state": review_state,
    }))
    .build()?;

    match EventStore::append_in_transaction(transaction, &event).await {
        Ok(_) => Ok(()),
        Err(error) if error.is_unique_violation() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) async fn load_identity_candidate_payload(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
) -> Result<PersonIdentityCandidatePayload, PersonIdentityError> {
    let row = sqlx::query(
        r#"
        SELECT
            candidate_kind,
            left_person_id,
            right_person_id,
            email_address,
            evidence_summary,
            confidence::float8 AS confidence
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_optional(&mut **transaction)
    .await?;

    let row = row.ok_or(PersonIdentityError::IdentityCandidateNotFound)?;
    let candidate_kind = match row.try_get::<String, _>("candidate_kind")?.as_str() {
        "merge_persons" => PersonIdentityCandidateKind::MergePersons,
        "attach_email_address" => PersonIdentityCandidateKind::AttachEmailAddress,
        "split_person" => PersonIdentityCandidateKind::SplitPerson,
        other => return Err(PersonIdentityError::InvalidCandidateKind(other.to_owned())),
    };

    Ok(PersonIdentityCandidatePayload {
        candidate_kind,
        left_person_id: row.try_get("left_person_id")?,
        right_person_id: row.try_get("right_person_id")?,
        email_address: row.try_get("email_address")?,
        evidence_summary: row.try_get("evidence_summary")?,
        confidence: row.try_get("confidence")?,
    })
}

pub(crate) fn parse_person_identity_candidate_kind(
    value: &str,
) -> Result<PersonIdentityCandidateKind, PersonIdentityError> {
    match value {
        "merge_persons" => Ok(PersonIdentityCandidateKind::MergePersons),
        "attach_email_address" => Ok(PersonIdentityCandidateKind::AttachEmailAddress),
        "split_person" => Ok(PersonIdentityCandidateKind::SplitPerson),
        other => Err(PersonIdentityError::InvalidCandidateKind(other.to_owned())),
    }
}

pub(crate) fn parse_person_identity_review_state(
    value: &str,
) -> Result<PersonIdentityReviewState, PersonIdentityError> {
    match value {
        "suggested" => Ok(PersonIdentityReviewState::Suggested),
        "user_confirmed" => Ok(PersonIdentityReviewState::UserConfirmed),
        "user_rejected" => Ok(PersonIdentityReviewState::UserRejected),
        other => Err(PersonIdentityError::InvalidReviewState(other.to_owned())),
    }
}
```

### `backend/src/domains/persons/identity/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/validation.rs`
- Size bytes / Размер в байтах: `1419`
- Included characters / Включено символов: `1419`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::constants::{DEFAULT_LIMIT, MAX_LIMIT, MIN_LIMIT};
use super::errors::PersonIdentityError;

pub(super) fn as_object(
    value: &Value,
) -> Result<&serde_json::Map<String, Value>, PersonIdentityError> {
    value
        .as_object()
        .ok_or_else(|| PersonIdentityError::InvalidPayload("payload".to_owned()))
}

pub(super) fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, PersonIdentityError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| PersonIdentityError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| PersonIdentityError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}

pub(super) fn validate_non_empty(field: &str, value: &str) -> Result<String, PersonIdentityError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(PersonIdentityError::EmptyField(field.to_owned()));
    }

    Ok(normalized.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, PersonIdentityError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(PersonIdentityError::InvalidLimit);
    }

    Ok(limit)
}

pub(super) fn validate_optional_limit(limit: Option<i64>) -> Result<i64, PersonIdentityError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIMIT))
}
```

### `backend/src/domains/persons/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/intelligence.rs`
- Size bytes / Размер в байтах: `8520`
- Included characters / Включено символов: `8510`
- Truncated / Обрезано: `no`

````rust
use crate::platform::ai_runtime::{AiRuntimePortError, SharedAiRuntimePort};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationFingerprint {
    pub avg_message_length: Option<usize>,
    pub avg_response_hours: Option<f64>,
    pub frequent_topics: Vec<String>,
    pub typical_tone: Option<String>,
    pub detected_language: Option<String>,
    pub writing_style: Option<String>,
    pub preferred_time_of_day: Option<String>,
    pub trust_score: Option<i16>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonInsight {
    pub person_id: String,
    pub fingerprint: CommunicationFingerprint,
    pub suggested_actions: Vec<String>,
}

#[derive(Clone)]
pub struct PersonIntelligenceService {
    runtime: Option<SharedAiRuntimePort>,
}

impl PersonIntelligenceService {
    pub fn new(runtime: Option<SharedAiRuntimePort>) -> Self {
        Self { runtime }
    }

    pub fn heuristic_fingerprint(messages: &[PersonMessage]) -> CommunicationFingerprint {
        if messages.is_empty() {
            return CommunicationFingerprint {
                avg_message_length: None,
                avg_response_hours: None,
                frequent_topics: vec![],
                typical_tone: None,
                detected_language: None,
                writing_style: None,
                preferred_time_of_day: None,
                trust_score: None,
            };
        }

        let total_len: usize = messages.iter().map(|m| m.body_text.len()).sum();
        let avg_len = total_len / messages.len();

        let mut topics = Vec::new();
        let combined_text: String = messages
            .iter()
            .map(|m| &m.body_text as &str)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        for (topic, keywords) in [
            ("finance", &["invoice", "payment", "amount", "tax"][..]),
            ("legal", &["contract", "nda", "agreement", "legal"][..]),
            (
                "project",
                &["project", "deadline", "milestone", "deliverable"][..],
            ),
            ("support", &["help", "issue", "problem", "bug"][..]),
        ] {
            if keywords.iter().any(|k| combined_text.contains(k)) {
                topics.push(topic.into());
            }
        }

        let tone = if combined_text.contains("urgent") || combined_text.contains("asap") {
            Some("urgent".into())
        } else if combined_text.contains("thanks") || combined_text.contains("appreciate") {
            Some("friendly".into())
        } else if combined_text.contains("please") && combined_text.contains("would") {
            Some("polite".into())
        } else {
            Some("neutral".into())
        };

        let detected_language = detect_language(&combined_text);

        let trust = 50i16
            .saturating_add((messages.len() as i16 * 2).min(30))
            .saturating_add(if !topics.is_empty() { 10 } else { 0 });

        CommunicationFingerprint {
            avg_message_length: Some(avg_len),
            avg_response_hours: None,
            frequent_topics: topics,
            typical_tone: tone,
            detected_language: Some(detected_language),
            writing_style: if avg_len > 500 {
                Some("verbose".into())
            } else if avg_len < 100 {
                Some("concise".into())
            } else {
                Some("balanced".into())
            },
            preferred_time_of_day: None,
            trust_score: Some(trust.clamp(0, 100)),
        }
    }

    pub async fn llm_fingerprint(
        &self,
        messages: &[PersonMessage],
    ) -> Result<Option<CommunicationFingerprint>, PersonIntelligenceError> {
        let Some(ref runtime) = self.runtime else {
            return Ok(None);
        };
        let sample: String = messages
            .iter()
            .take(5)
            .map(|m| format!("Subject: {}\nBody: {}\n", m.subject, m.body_text))
            .collect::<Vec<_>>()
            .join("\n---\n");
        let prompt = format!(
            "Analyze communication patterns from these email samples. Return JSON with: frequent_topics (array of strings), typical_tone (one word), detected_language (code), writing_style (verbose/concise/balanced), preferred_time_of_day (morning/afternoon/evening or null).\n\nSamples:\n{sample}"
        );
        let result = runtime.chat(&prompt).await?;
        let content = result
            .content
            .trim()
            .strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .map(str::trim)
            .unwrap_or(result.content.trim());
        Ok(serde_json::from_str(content).ok())
    }

    pub fn suggested_actions(fingerprint: &CommunicationFingerprint) -> Vec<String> {
        let mut actions = Vec::new();
        if let Some(ref tone) = fingerprint.typical_tone {
            actions.push(format!("Person tends to be {tone} — match tone in replies"));
        }
        if let Some(ref lang) = fingerprint.detected_language
            && lang != "en"
        {
            actions.push(format!(
                "Person writes in {lang} — consider translating replies"
            ));
        }
        if let Some(ref style) = fingerprint.writing_style {
            actions.push(format!("Person style: {style}"));
        }
        if let Some(score) = fingerprint.trust_score
            && score < 30
        {
            actions.push("Low trust score — verify claims".into());
        }
        actions
    }
}

fn detect_language(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        return "unknown".to_owned();
    }

    let lower = text.to_lowercase();
    if text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)) {
        if lower.contains('ї') || lower.contains('є') {
            return "uk".to_owned();
        }
        return "ru".to_owned();
    }
    if text.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)) {
        return "zh".to_owned();
    }
    if lower.contains('ñ')
        || [
            "hola",
            "gracias",
            "para",
            "como",
            "que",
            "por favor",
            "saludos",
            "adjunto",
        ]
        .iter()
        .any(|word| lower.contains(word))
    {
        return "es".to_owned();
    }
    if ["privet", "spasibo", "pozhaluysta"]
        .iter()
        .any(|word| lower.contains(word))
    {
        return "ru".to_owned();
    }
    if [
        "mit", "und", "der", "die", "das", "ist", "von", "für", "danke", "bitte",
    ]
    .iter()
    .any(|word| lower.contains(word))
    {
        return "de".to_owned();
    }
    if text.chars().any(|c| c.is_ascii_alphabetic()) {
        return "en".to_owned();
    }

    "unknown".to_owned()
}

#[derive(Clone, Debug)]
pub struct PersonMessage {
    pub subject: String,
    pub body_text: String,
    pub occurred_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Error)]
pub enum PersonIntelligenceError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimePortError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_messages() -> Vec<PersonMessage> {
        vec![
            PersonMessage {
                subject: "Invoice".into(),
                body_text: "Please pay invoice #123 for $500".into(),
                occurred_at: None,
            },
            PersonMessage {
                subject: "Thanks".into(),
                body_text: "Thank you for your help with the project".into(),
                occurred_at: None,
            },
        ]
    }

    #[test]
    fn fingerprint_detects_topics() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&sample_messages());
        assert!(fp.frequent_topics.contains(&"finance".into()));
    }
    #[test]
    fn fingerprint_sets_trust() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&sample_messages());
        assert!(fp.trust_score.unwrap() >= 50);
    }
    #[test]
    fn fingerprint_detects_tone() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&sample_messages());
        assert!(fp.typical_tone.is_some());
    }
    #[test]
    fn empty_messages_returns_none() {
        let fp = PersonIntelligenceService::heuristic_fingerprint(&[]);
        assert!(fp.trust_score.is_none());
    }
}
````

### `backend/src/domains/persons/investigator.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator.rs`
- Size bytes / Размер в байтах: `281`
- Included characters / Включено символов: `281`
- Truncated / Обрезано: `no`

```rust
mod assembly;
mod errors;
mod meeting_prep;
mod models;
mod sections;
mod service;
mod snapshots;

pub use errors::InvestigatorError;
pub use models::{
    DossierReviewState, DossierSectionItem, DossierSnapshot, MeetingPrep, PersonDossier,
};
pub use service::PersonInvestigator;
```

### `backend/src/domains/persons/investigator/assembly.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator/assembly.rs`
- Size bytes / Размер в байтах: `3225`
- Included characters / Включено символов: `3225`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use sqlx::postgres::PgPool;

use super::errors::InvestigatorError;
use super::models::PersonDossier;
use super::sections::{
    ai_observation_section, communication_pattern_section, dossier_source_refs, expertise_section,
    fact_section,
};
use crate::domains::persons::enrichment::PersonEnrichmentStore;
use crate::domains::persons::expertise::PersonExpertiseStore;
use crate::domains::persons::memory::{
    PersonFactStore, PersonMemoryCardStore, PersonPreferenceStore, RelationshipEventStore,
};

pub(super) async fn assemble_dossier(
    pool: &PgPool,
    person_id: &str,
) -> Result<PersonDossier, InvestigatorError> {
    let enrichment = PersonEnrichmentStore::new(pool.clone());
    let facts = PersonFactStore::new(pool.clone());
    let cards = PersonMemoryCardStore::new(pool.clone());
    let preferences = PersonPreferenceStore::new(pool.clone());
    let timeline = RelationshipEventStore::new(pool.clone());
    let expertise = PersonExpertiseStore::new(pool.clone());

    let person = enrichment
        .get_enriched(person_id)
        .await?
        .ok_or(InvestigatorError::PersonNotFound)?;

    let facts_list = facts.list(person_id).await.unwrap_or_default();
    let cards_list = cards.list(person_id).await.unwrap_or_default();
    let preferences_list = preferences.list(person_id).await.unwrap_or_default();
    let timeline_list = timeline.timeline(person_id, 50).await.unwrap_or_default();
    let expertise_list = expertise.list(person_id).await.unwrap_or_default();

    let mut summary_parts: Vec<String> = Vec::new();
    if let Some(tone) = &person.tone {
        summary_parts.push(format!("Tone: {tone}"));
    }
    if let Some(lang) = &person.language {
        summary_parts.push(format!("Language: {lang}"));
    }
    if person.interaction_count > 0 {
        summary_parts.push(format!("{} interactions", person.interaction_count));
    }
    if !person.frequent_topics.is_empty() {
        summary_parts.push(format!("Topics: {}", person.frequent_topics.join(", ")));
    }
    for card in &cards_list {
        if card.importance >= 7 {
            summary_parts.push(format!("Key: {}", card.title));
        }
    }

    let interests = fact_section(&facts_list, "interest");
    let projects = fact_section(&facts_list, "project");
    let organizations = fact_section(&facts_list, "organization");
    let skills = expertise_section(&expertise_list);
    let communication_patterns = communication_pattern_section(&person, &preferences_list);
    let ai_observations = ai_observation_section(&cards_list);
    let source_refs = dossier_source_refs(
        &facts_list,
        &cards_list,
        &preferences_list,
        &timeline_list,
        &expertise_list,
    );

    Ok(PersonDossier {
        person,
        facts: facts_list,
        memory_cards: cards_list,
        timeline: timeline_list,
        identities: vec![],
        expertise: vec![],
        promises: vec![],
        risks: vec![],
        summary: summary_parts.join(" | "),
        interests,
        projects,
        organizations,
        skills,
        communication_patterns,
        ai_observations,
        source_refs,
        generated_at: Utc::now(),
    })
}
```

### `backend/src/domains/persons/investigator/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator/errors.rs`
- Size bytes / Размер в байтах: `2077`
- Included characters / Включено символов: `2077`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::persons::enrichment::PersonEnrichmentError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::platform::events::EventStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum InvestigatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Memory(#[from] crate::engines::memory::MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
    #[error(transparent)]
    Trust(#[from] crate::engines::trust::TrustEngineError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Event(#[from] EventStoreError),
    #[error("person not found")]
    PersonNotFound,
    #[error("dossier snapshot not found")]
    DossierSnapshotNotFound,
    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidDossierReviewState,
}

impl From<PersonEnrichmentError> for InvestigatorError {
    fn from(error: PersonEnrichmentError) -> Self {
        match error {
            PersonEnrichmentError::NotFound => Self::PersonNotFound,
            PersonEnrichmentError::Sqlx(error) => Self::Sqlx(error),
            PersonEnrichmentError::Trust(error) => Self::Trust(error),
            PersonEnrichmentError::Observation(error) => Self::Observation(error),
            PersonEnrichmentError::Event(error) => Self::Event(error),
        }
    }
}

impl From<PersonMemoryError> for InvestigatorError {
    fn from(error: PersonMemoryError) -> Self {
        match error {
            PersonMemoryError::NotFound => Self::PersonNotFound,
            PersonMemoryError::Sqlx(error) => Self::Sqlx(error),
            PersonMemoryError::Memory(error) => Self::Memory(error),
            PersonMemoryError::Timeline(error) => Self::Timeline(error),
            PersonMemoryError::ObservationStore(error) => Self::Observation(error),
        }
    }
}
```

### `backend/src/domains/persons/investigator/meeting_prep.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator/meeting_prep.rs`
- Size bytes / Размер в байтах: `1839`
- Included characters / Включено символов: `1839`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::errors::InvestigatorError;
use super::models::MeetingPrep;
use crate::domains::persons::enrichment::PersonEnrichmentStore;
use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore};

pub(super) async fn meeting_prep(
    pool: &PgPool,
    person_id: &str,
) -> Result<MeetingPrep, InvestigatorError> {
    let enrichment = PersonEnrichmentStore::new(pool.clone());
    let person = enrichment
        .get_enriched(person_id)
        .await?
        .ok_or(InvestigatorError::PersonNotFound)?;

    let last_interaction_days = person
        .last_interaction_at
        .map(|dt| (chrono::Utc::now() - dt).num_days());

    let promises = PersonPromiseStore::new(pool.clone());
    let risks = PersonRiskStore::new(pool.clone());
    let open_promises = promises
        .list(person_id)
        .await
        .unwrap_or_default()
        .iter()
        .filter(|promise| promise.status == "pending")
        .count() as i64;
    let open_risks = risks
        .list(person_id)
        .await
        .unwrap_or_default()
        .iter()
        .filter(|risk| risk.resolved_at.is_none())
        .count() as i64;

    let mut tips = person
        .frequent_topics
        .iter()
        .map(|topic| format!("Discuss topic: {topic}"))
        .collect::<Vec<_>>();
    if let Some(tone) = &person.tone {
        tips.push(format!("Match tone: {tone}"));
    }
    if let Some(style) = &person.writing_style {
        tips.push(format!("Style: {style}"));
    }

    Ok(MeetingPrep {
        person_id: person.person_id,
        display_name: person.display_name,
        last_interaction_days,
        open_promises,
        open_risks,
        recent_topics: person.frequent_topics,
        communication_tips: tips,
        shared_projects: person.linked_projects,
    })
}
```

### `backend/src/domains/persons/investigator/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator/models.rs`
- Size bytes / Размер в байтах: `2722`
- Included characters / Включено символов: `2722`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::InvestigatorError;
use crate::domains::persons::enrichment::EnrichedPerson;
use crate::domains::persons::memory::{PersonFact, PersonMemoryCard, RelationshipEvent};

#[derive(Clone, Debug, Serialize)]
pub struct DossierSectionItem {
    pub label: String,
    pub value: String,
    pub source_refs: Vec<String>,
    pub confidence: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonDossier {
    pub person: EnrichedPerson,
    pub facts: Vec<PersonFact>,
    pub memory_cards: Vec<PersonMemoryCard>,
    pub timeline: Vec<RelationshipEvent>,
    pub identities: Vec<Value>,
    pub expertise: Vec<Value>,
    pub promises: Vec<Value>,
    pub risks: Vec<Value>,
    pub summary: String,
    pub interests: Vec<DossierSectionItem>,
    pub projects: Vec<DossierSectionItem>,
    pub organizations: Vec<DossierSectionItem>,
    pub skills: Vec<DossierSectionItem>,
    pub communication_patterns: Vec<DossierSectionItem>,
    pub ai_observations: Vec<DossierSectionItem>,
    pub source_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DossierReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl DossierReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: &str) -> Result<Self, InvestigatorError> {
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(InvestigatorError::InvalidDossierReviewState),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DossierSnapshot {
    pub dossier_snapshot_id: String,
    pub persona_id: String,
    pub dossier: Value,
    pub source_refs: Value,
    pub review_state: DossierReviewState,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub generated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeetingPrep {
    pub person_id: String,
    pub display_name: String,
    pub last_interaction_days: Option<i64>,
    pub open_promises: i64,
    pub open_risks: i64,
    pub recent_topics: Vec<String>,
    pub communication_tips: Vec<String>,
    pub shared_projects: Vec<String>,
}
```

### `backend/src/domains/persons/investigator/sections.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator/sections.rs`
- Size bytes / Размер в байтах: `4099`
- Included characters / Включено символов: `4099`
- Truncated / Обрезано: `no`

```rust
use std::collections::BTreeSet;

use super::models::DossierSectionItem;
use crate::domains::persons::enrichment::EnrichedPerson;
use crate::domains::persons::expertise::PersonExpertise;
use crate::domains::persons::memory::{
    PersonFact, PersonMemoryCard, PersonPreference, RelationshipEvent,
};

pub(super) fn fact_section(facts: &[PersonFact], fact_type: &str) -> Vec<DossierSectionItem> {
    facts
        .iter()
        .filter(|fact| fact.is_active && fact.fact_type == fact_type)
        .map(|fact| DossierSectionItem {
            label: fact.fact_type.clone(),
            value: fact.value.clone(),
            source_refs: vec![fact.source.clone()],
            confidence: Some(fact.confidence),
        })
        .collect()
}

pub(super) fn expertise_section(expertise: &[PersonExpertise]) -> Vec<DossierSectionItem> {
    expertise
        .iter()
        .map(|item| DossierSectionItem {
            label: item.domain.clone().unwrap_or_else(|| "skill".to_owned()),
            value: item.skill.clone(),
            source_refs: vec![item.source.clone()],
            confidence: Some(item.confidence),
        })
        .collect()
}

pub(super) fn communication_pattern_section(
    person: &EnrichedPerson,
    preferences: &[PersonPreference],
) -> Vec<DossierSectionItem> {
    let mut items = Vec::new();
    let root_source = format!("persons:{}", person.person_id);

    if let Some(language) = &person.language {
        items.push(DossierSectionItem {
            label: "language".to_owned(),
            value: language.clone(),
            source_refs: vec![root_source.clone()],
            confidence: None,
        });
    }
    if let Some(tone) = &person.tone {
        items.push(DossierSectionItem {
            label: "tone".to_owned(),
            value: tone.clone(),
            source_refs: vec![root_source.clone()],
            confidence: None,
        });
    }
    if let Some(writing_style) = &person.writing_style {
        items.push(DossierSectionItem {
            label: "writing_style".to_owned(),
            value: writing_style.clone(),
            source_refs: vec![root_source.clone()],
            confidence: None,
        });
    }

    for preference in preferences {
        if preference.preference_type.starts_with("communication:")
            || preference
                .preference_type
                .starts_with("interaction_context:")
        {
            items.push(DossierSectionItem {
                label: preference.preference_type.clone(),
                value: preference.value.clone(),
                source_refs: vec![preference.source.clone()],
                confidence: Some(preference.confidence),
            });
        }
    }

    items
}

pub(super) fn ai_observation_section(cards: &[PersonMemoryCard]) -> Vec<DossierSectionItem> {
    cards
        .iter()
        .filter(|card| card.source.contains("ai") || card.title.to_lowercase().contains("ai"))
        .map(|card| DossierSectionItem {
            label: card.title.clone(),
            value: card.description.clone(),
            source_refs: vec![card.source.clone()],
            confidence: Some(card.confidence),
        })
        .collect()
}

pub(super) fn dossier_source_refs(
    facts: &[PersonFact],
    cards: &[PersonMemoryCard],
    preferences: &[PersonPreference],
    timeline: &[RelationshipEvent],
    expertise: &[PersonExpertise],
) -> Vec<String> {
    let mut refs = BTreeSet::new();
    for fact in facts {
        add_source_ref(&mut refs, &fact.source);
    }
    for card in cards {
        add_source_ref(&mut refs, &card.source);
    }
    for preference in preferences {
        add_source_ref(&mut refs, &preference.source);
    }
    for event in timeline {
        add_source_ref(&mut refs, &event.source);
    }
    for item in expertise {
        add_source_ref(&mut refs, &item.source);
    }
    refs.into_iter().collect()
}

fn add_source_ref(refs: &mut BTreeSet<String>, source: &str) {
    let source = source.trim();
    if !source.is_empty() {
        refs.insert(source.to_owned());
    }
}
```

### `backend/src/domains/persons/investigator/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator/service.rs`
- Size bytes / Размер в байтах: `4275`
- Included characters / Включено символов: `4275`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::assembly;
use super::errors::InvestigatorError;
use super::meeting_prep;
use super::models::{DossierReviewState, DossierSnapshot, MeetingPrep, PersonDossier};
use super::snapshots;
use crate::domains::persons::core::link_persons_entity;
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, materialize_review_transition_link,
};

#[derive(Clone)]
pub struct PersonInvestigator {
    pool: PgPool,
}

impl PersonInvestigator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn assemble_dossier(
        &self,
        person_id: &str,
    ) -> Result<PersonDossier, InvestigatorError> {
        assembly::assemble_dossier(&self.pool, person_id).await
    }

    pub async fn assemble_and_cache_dossier(
        &self,
        person_id: &str,
    ) -> Result<(PersonDossier, DossierSnapshot), InvestigatorError> {
        let dossier = self.assemble_dossier(person_id).await?;
        let snapshot = self.cache_dossier_snapshot(&dossier).await?;
        Ok((dossier, snapshot))
    }

    pub async fn assemble_cache_and_record_refresh(
        &self,
        person_id: &str,
        operation: &str,
        captured_by: &str,
        endpoint: &str,
        source_ref: String,
    ) -> Result<(PersonDossier, DossierSnapshot), InvestigatorError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "PERSON_MUTATION",
                    ObservationOriginKind::Manual,
                    chrono::Utc::now(),
                    json!({
                        "person_id": person_id,
                        "operation": operation,
                    }),
                    source_ref,
                )
                .provenance(json!({
                    "captured_by": captured_by,
                    "endpoint": endpoint,
                })),
            )
            .await?;
        let (dossier, snapshot) = self.assemble_and_cache_dossier(person_id).await?;
        link_persons_entity(
            &self.pool,
            &observation.observation_id,
            "dossier_snapshot",
            snapshot.dossier_snapshot_id.clone(),
            Some("dossier_refresh"),
            Some(json!({
                "person_id": person_id,
                "trigger": endpoint,
            })),
        )
        .await?;
        Ok((dossier, snapshot))
    }

    pub async fn cache_dossier_snapshot(
        &self,
        dossier: &PersonDossier,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        snapshots::cache_dossier_snapshot(&self.pool, dossier).await
    }

    pub async fn review_dossier_snapshot(
        &self,
        person_id: &str,
        review_state: DossierReviewState,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        self.review_dossier_snapshot_with_observation(person_id, review_state, None, None)
            .await
    }

    pub async fn review_dossier_snapshot_with_observation(
        &self,
        person_id: &str,
        review_state: DossierReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        let snapshot =
            snapshots::review_dossier_snapshot(&self.pool, person_id, review_state).await?;
        materialize_review_transition_link(
            &self.pool,
            observation_id,
            "persons",
            "dossier_snapshot",
            &snapshot.dossier_snapshot_id,
            "review_state",
            snapshot.review_state.as_str(),
            metadata
                .map(|extra| {
                    json!({
                        "person_id": person_id,
                        "context": extra,
                    })
                })
                .or_else(|| {
                    Some(json!({
                        "person_id": person_id,
                    }))
                }),
        )
        .await?;
        Ok(snapshot)
    }

    pub async fn meeting_prep(&self, person_id: &str) -> Result<MeetingPrep, InvestigatorError> {
        meeting_prep::meeting_prep(&self.pool, person_id).await
    }
}
```

### `backend/src/domains/persons/investigator/snapshots.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/investigator/snapshots.rs`
- Size bytes / Размер в байтах: `3320`
- Included characters / Включено символов: `3320`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::InvestigatorError;
use super::models::{DossierReviewState, DossierSnapshot, PersonDossier};

pub(super) async fn cache_dossier_snapshot(
    pool: &PgPool,
    dossier: &PersonDossier,
) -> Result<DossierSnapshot, InvestigatorError> {
    let dossier_value = serde_json::to_value(dossier)?;
    let source_refs = serde_json::to_value(&dossier.source_refs)?;
    let snapshot_id = dossier_snapshot_id(&dossier.person.person_id);
    let row = sqlx::query(
        r#"
        INSERT INTO persona_dossier_snapshots (
            dossier_snapshot_id,
            persona_id,
            dossier,
            source_refs,
            review_state,
            generated_at
        )
        VALUES ($1, $2, $3, $4, 'suggested', $5)
        ON CONFLICT (persona_id)
        DO UPDATE SET
            dossier = EXCLUDED.dossier,
            source_refs = EXCLUDED.source_refs,
            generated_at = EXCLUDED.generated_at,
            updated_at = now()
        RETURNING
            dossier_snapshot_id,
            persona_id,
            dossier,
            source_refs,
            review_state,
            reviewed_by,
            reviewed_at,
            metadata,
            generated_at,
            created_at,
            updated_at
        "#,
    )
    .bind(&snapshot_id)
    .bind(&dossier.person.person_id)
    .bind(dossier_value)
    .bind(source_refs)
    .bind(dossier.generated_at)
    .fetch_one(pool)
    .await?;

    row_to_dossier_snapshot(row)
}

pub(super) async fn review_dossier_snapshot(
    pool: &PgPool,
    person_id: &str,
    review_state: DossierReviewState,
) -> Result<DossierSnapshot, InvestigatorError> {
    let row = sqlx::query(
        r#"
        UPDATE persona_dossier_snapshots
        SET
            review_state = $2,
            reviewed_by = 'owner_persona',
            reviewed_at = now(),
            updated_at = now()
        WHERE persona_id = $1
        RETURNING
            dossier_snapshot_id,
            persona_id,
            dossier,
            source_refs,
            review_state,
            reviewed_by,
            reviewed_at,
            metadata,
            generated_at,
            created_at,
            updated_at
        "#,
    )
    .bind(person_id)
    .bind(review_state.as_str())
    .fetch_optional(pool)
    .await?
    .ok_or(InvestigatorError::DossierSnapshotNotFound)?;

    row_to_dossier_snapshot(row)
}

fn dossier_snapshot_id(person_id: &str) -> String {
    format!("persona_dossier:v1:{person_id}")
}

fn row_to_dossier_snapshot(row: PgRow) -> Result<DossierSnapshot, InvestigatorError> {
    Ok(DossierSnapshot {
        dossier_snapshot_id: row.try_get("dossier_snapshot_id")?,
        persona_id: row.try_get("persona_id")?,
        dossier: row.try_get("dossier")?,
        source_refs: row.try_get("source_refs")?,
        review_state: DossierReviewState::parse(
            row.try_get::<String, _>("review_state")?.as_str(),
        )?,
        reviewed_by: row.try_get("reviewed_by")?,
        reviewed_at: row.try_get("reviewed_at")?,
        metadata: row.try_get("metadata")?,
        generated_at: row.try_get("generated_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/persons/memory.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/memory.rs`
- Size bytes / Размер в байтах: `553`
- Included characters / Включено символов: `553`
- Truncated / Обрезано: `no`

```rust
mod cards;
mod errors;
mod facts;
mod preferences;
mod relationship_events;
mod snapshots;

pub use cards::{PersonMemoryCard, PersonMemoryCardStore};
pub use errors::PersonMemoryError;
pub use facts::{PersonFact, PersonFactStore};
pub use preferences::{PersonPreference, PersonPreferenceStore};
pub use relationship_events::RelationshipEventStore as RelationshipEventPort;
pub use relationship_events::{NewRelationshipEvent, RelationshipEvent, RelationshipEventStore};
pub use snapshots::{FieldChange, HistoryDiff, PersonSnapshot, PersonSnapshotStore};
```

### `backend/src/domains/persons/memory/cards.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/memory/cards.rs`
- Size bytes / Размер в байтах: `3338`
- Included characters / Включено символов: `3338`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;
use crate::domains::persons::core::link_persons_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonMemoryCard {
    pub id: String,
    pub person_id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct PersonMemoryCardStore {
    pool: PgPool,
}

impl PersonMemoryCardStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonMemoryCard>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, title, description, source, confidence::float8 AS confidence, importance,
             created_at, last_verified_at FROM person_memory_cards
             WHERE person_id = $1 ORDER BY importance DESC, created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_memory_card).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        title: &str,
        description: &str,
        source: &str,
        importance: i16,
    ) -> Result<PersonMemoryCard, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_memory_cards (person_id, title, description, source, importance)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, title, description, source, confidence::float8 AS confidence, importance,
                       created_at, last_verified_at",
        )
        .bind(person_id)
        .bind(title)
        .bind(description)
        .bind(source)
        .bind(importance)
        .fetch_one(&self.pool)
        .await?;
        row_to_memory_card(row)
    }

    pub async fn upsert_with_observation(
        &self,
        person_id: &str,
        title: &str,
        description: &str,
        source: &str,
        importance: i16,
        observation_id: &str,
    ) -> Result<PersonMemoryCard, PersonMemoryError> {
        let card = self
            .upsert(person_id, title, description, source, importance)
            .await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "memory_card",
            card.id.clone(),
            None,
            Some(json!({
                "person_id": person_id,
                "importance": card.importance,
            })),
        )
        .await?;
        Ok(card)
    }
}

fn row_to_memory_card(row: PgRow) -> Result<PersonMemoryCard, PersonMemoryError> {
    Ok(PersonMemoryCard {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        importance: row.try_get("importance")?,
        created_at: row.try_get("created_at")?,
        last_verified_at: row.try_get("last_verified_at")?,
    })
}
```

### `backend/src/domains/persons/memory/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/memory/errors.rs`
- Size bytes / Размер в байтах: `525`
- Included characters / Включено символов: `525`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::engines::memory::MemoryEngineError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonMemoryError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Memory(#[from] MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error("fact not found")]
    NotFound,
}
```

### `backend/src/domains/persons/memory/facts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/memory/facts.rs`
- Size bytes / Размер в байтах: `4546`
- Included characters / Включено символов: `4546`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;
use crate::domains::persons::core::link_persons_entity;
use crate::engines::memory::MemoryEngine;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonFact {
    pub id: String,
    pub person_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonFactStore {
    pool: PgPool,
}

impl PersonFactStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonFact>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, fact_type, value, source, confidence::float8 AS confidence, last_verified_at,
             valid_from, valid_to, is_active, created_at, updated_at
             FROM person_facts WHERE person_id = $1 ORDER BY created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_fact).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<PersonFact, PersonMemoryError> {
        let fact =
            MemoryEngine::persona_fact_memory(person_id, fact_type, value, source, confidence)?;
        let row = sqlx::query(
            "INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, fact_type, value, source, confidence::float8 AS confidence,
                       last_verified_at, valid_from, valid_to, is_active, created_at, updated_at",
        )
        .bind(&fact.affected_entity_id)
        .bind(&fact.fact_type)
        .bind(&fact.value)
        .bind(&fact.source)
        .bind(fact.confidence)
        .fetch_one(&self.pool)
        .await?;
        row_to_fact(row)
    }

    pub async fn upsert_with_observation(
        &self,
        person_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
        observation_id: &str,
    ) -> Result<PersonFact, PersonMemoryError> {
        let fact = self
            .upsert(person_id, fact_type, value, source, confidence)
            .await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "fact",
            fact.id.clone(),
            None,
            Some(json!({
                "person_id": person_id,
                "fact_type": fact.fact_type,
            })),
        )
        .await?;
        Ok(fact)
    }

    pub async fn update_confidence(
        &self,
        id: &str,
        confidence: f64,
    ) -> Result<(), PersonMemoryError> {
        sqlx::query("UPDATE person_facts SET confidence = $2, last_verified_at = now(), updated_at = now() WHERE id::text = $1")
            .bind(id).bind(confidence).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn decay_unverified(&self, threshold_days: i64) -> Result<u64, PersonMemoryError> {
        let result = sqlx::query(
            "UPDATE person_facts SET confidence = confidence * 0.5, updated_at = now()
             WHERE last_verified_at IS NULL
                OR last_verified_at < now() - ($1 || ' days')::interval",
        )
        .bind(threshold_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

fn row_to_fact(row: PgRow) -> Result<PersonFact, PersonMemoryError> {
    Ok(PersonFact {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        fact_type: row.try_get("fact_type")?,
        value: row.try_get("value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/persons/memory/preferences.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/memory/preferences.rs`
- Size bytes / Размер в байтах: `3292`
- Included characters / Включено символов: `3292`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;
use crate::domains::persons::core::link_persons_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPreference {
    pub id: String,
    pub person_id: String,
    pub preference_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPreferenceStore {
    pool: PgPool,
}

impl PersonPreferenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonPreference>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, preference_type, value, source, confidence::float8 AS confidence,
             last_verified_at, created_at, updated_at FROM person_preferences
             WHERE person_id = $1 ORDER BY preference_type",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_preference).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        preference_type: &str,
        value: &str,
        source: &str,
    ) -> Result<PersonPreference, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_preferences (person_id, preference_type, value, source)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (person_id, preference_type) DO UPDATE SET value = $3, source = $4, updated_at = now()
             RETURNING id::text, person_id, preference_type, value, source, confidence::float8 AS confidence,
                       last_verified_at, created_at, updated_at"
        ).bind(person_id).bind(preference_type).bind(value).bind(source).fetch_one(&self.pool).await?;
        row_to_preference(row)
    }

    pub async fn upsert_with_observation(
        &self,
        person_id: &str,
        preference_type: &str,
        value: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<PersonPreference, PersonMemoryError> {
        let pref = self
            .upsert(person_id, preference_type, value, source)
            .await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "preference",
            pref.id.clone(),
            None,
            Some(json!({
                "person_id": person_id,
                "preference_type": pref.preference_type,
            })),
        )
        .await?;
        Ok(pref)
    }
}

fn row_to_preference(row: PgRow) -> Result<PersonPreference, PersonMemoryError> {
    Ok(PersonPreference {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        preference_type: row.try_get("preference_type")?,
        value: row.try_get("value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```
