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

- Chunk ID / ID чанка: `035-source-backend-part-015`
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

### `backend/src/application/review_inbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/review_inbox.rs`
- Size bytes / Размер в байтах: `50`
- Included characters / Включено символов: `50`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::review_inbox::*;
```

### `backend/src/application/review_promotion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/review_promotion.rs`
- Size bytes / Размер в байтах: `54`
- Included characters / Включено символов: `54`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::review_promotion::*;
```

### `backend/src/application/review_transitions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/review_transitions.rs`
- Size bytes / Размер в байтах: `10229`
- Included characters / Включено символов: `10229`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::decisions::{Decision, DecisionReviewState, DecisionStore, DecisionStoreError};
use crate::domains::obligations::{
    Obligation, ObligationReviewState, ObligationStore, ObligationStoreError,
};
use crate::domains::relationships::{
    Relationship, RelationshipReviewState, RelationshipStore, RelationshipStoreError,
};
use crate::domains::tasks::candidates::{
    StoredCandidateRow, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateReviewService, TaskCandidateReviewServiceError, TaskCandidateReviewState,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, sync_decision_review_state_with_observation,
    sync_obligation_review_state_with_observation, sync_relationship_review_state_with_observation,
    sync_task_candidate_review_state_in_transaction,
};

#[derive(Clone)]
pub struct DecisionReviewApplicationService {
    pool: PgPool,
}

impl DecisionReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "decision_id": decision_id,
                        "review_state": review_state.as_str(),
                        "operation": "decision_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("decision://{decision_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "decision_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let decision = DecisionStore::new(self.pool.clone())
            .set_review_state_with_observation(
                decision_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "decision_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        sync_decision_review_state_with_observation(
            &self.pool,
            &decision,
            &observation.observation_id,
        )
        .await?;

        Ok(decision)
    }
}

#[derive(Debug, Error)]
pub enum DecisionReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct ObligationReviewApplicationService {
    pool: PgPool,
}

impl ObligationReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "obligation_id": obligation_id,
                        "review_state": review_state.as_str(),
                        "operation": "obligation_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("obligation://{obligation_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "obligation_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let obligation = ObligationStore::new(self.pool.clone())
            .set_review_state_with_observation(
                obligation_id,
                review_state,
                Some(&observation.observation_id),
                None,
            )
            .await?;

        sync_obligation_review_state_with_observation(
            &self.pool,
            &obligation,
            &observation.observation_id,
        )
        .await?;

        Ok(obligation)
    }
}

#[derive(Debug, Error)]
pub enum ObligationReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct RelationshipReviewApplicationService {
    pool: PgPool,
}

impl RelationshipReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipReviewApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "relationship_id": relationship_id,
                        "review_state": review_state.as_str(),
                        "operation": "relationship_review",
                        "actor_id": "makosh-frontend",
                    }),
                    format!("relationship://{relationship_id}/review"),
                )
                .provenance(json!({
                    "captured_by": "relationship_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        let relationship = RelationshipStore::new(self.pool.clone())
            .set_review_state_with_observation(
                relationship_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "relationship_review_application.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        sync_relationship_review_state_with_observation(
            &self.pool,
            &relationship,
            &observation.observation_id,
        )
        .await?;

        Ok(relationship)
    }
}

#[derive(Debug, Error)]
pub enum RelationshipReviewApplicationError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}

#[derive(Clone)]
pub struct TaskCandidateReviewApplicationService {
    pool: PgPool,
}

impl TaskCandidateReviewApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateReviewApplicationError> {
        let result = TaskCandidateReviewService::new(self.pool.clone())
            .review_manual(command)
            .await?;

        let mut transaction = self.pool.begin().await?;
        let candidate_row = sqlx::query(
            r#"
            SELECT
                source_kind,
                source_id,
                observation_id,
                candidate_kind,
                candidate_metadata,
                project_id,
                title,
                due_text,
                assignee_label,
                confidence::float8 AS confidence,
                evidence_excerpt
            FROM task_candidates
            WHERE task_candidate_id = $1
            FOR UPDATE
            "#,
        )
        .bind(&command.task_candidate_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(TaskCandidateReviewApplicationError::TaskCandidateNotFound)?;
        let candidate = StoredCandidateRow {
            source_kind: sqlx::Row::try_get(&candidate_row, "source_kind")?,
            source_id: sqlx::Row::try_get(&candidate_row, "source_id")?,
            observation_id: sqlx::Row::try_get(&candidate_row, "observation_id")?,
            candidate_kind: sqlx::Row::try_get(&candidate_row, "candidate_kind")?,
            candidate_metadata: sqlx::Row::try_get(&candidate_row, "candidate_metadata")?,
            project_id: sqlx::Row::try_get(&candidate_row, "project_id")?,
            title: sqlx::Row::try_get(&candidate_row, "title")?,
            due_text: sqlx::Row::try_get(&candidate_row, "due_text")?,
            assignee_label: sqlx::Row::try_get(&candidate_row, "assignee_label")?,
            confidence: sqlx::Row::try_get(&candidate_row, "confidence")?,
            evidence_excerpt: sqlx::Row::try_get(&candidate_row, "evidence_excerpt")?,
        };
        sync_task_candidate_review_state_in_transaction(
            &mut transaction,
            &command.task_candidate_id,
            &candidate,
            command.review_state,
        )
        .await?;
        transaction.commit().await?;

        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum TaskCandidateReviewApplicationError {
    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateReviewServiceError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error("task candidate was not found")]
    TaskCandidateNotFound,
}
```

### `backend/src/application/signal_hub_replay.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/signal_hub_replay.rs`
- Size bytes / Размер в байтах: `39311`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::domains::communications::messages::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, replay_accepted_signal_event,
    supports_communication_projection_signal_event,
};
use crate::domains::persons::core::{
    PERSON_ROLE_ASSIGNED_EVENT_TYPE, PERSON_ROLE_REMOVED_EVENT_TYPE,
};
use crate::domains::persons::enrichment::PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::persons::trust::PERSON_PROMISE_CREATED_EVENT_TYPE;
use crate::domains::signal_hub::{
    SignalHubError, SignalHubSignalService, SignalHubStore, SignalReplayRequest,
    SignalReplayRequestCreate,
};
use crate::engines::timeline::TimelineEngine;
use crate::platform::events::{
    EventConsumerStore, EventLogQuery, EventStore, NewEventEnvelope, ProjectionCursorStore,
    StoredEventEnvelope,
};
use crate::workflows::project_link_review_effects::PROJECT_LINK_REVIEW_EVENT_TYPE;
use crate::workflows::realtime_conversation_transcript_projection::REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER;

use super::{
    PERSON_DERIVED_EVIDENCE_CONSUMER, PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
    ZOOM_CALENDAR_MATCHING_CONSUMER, project_link_review_effect_event,
    project_person_derived_evidence_event, project_realtime_conversation_transcript_event,
    project_yandex_telemost_calendar_matching_event, project_zoom_calendar_matching_event,
};

const DEFAULT_REPLAY_BATCH_SIZE: u32 = 500;
const COMMUNICATION_MESSAGES_PROJECTION: &str = "communication_messages";
const PERSON_DERIVED_EVIDENCE_PROJECTION: &str = "person_derived_evidence";
const PROJECT_LINK_REVIEW_EFFECTS_PROJECTION: &str = "project_link_review_effects";
const REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION: &str =
    "realtime_conversation_transcript_projection";
const TIMELINE_EVENT_LOG_PROJECTION: &str = "timeline_event_log";
const YANDEX_TELEMOST_CALENDAR_MATCHING_PROJECTION: &str = "yandex_telemost_calendar_matching";
const ZOOM_CALENDAR_MATCHING_PROJECTION: &str = "zoom_calendar_matching";
const TIMELINE_EVENT_LOG_CURSOR: &str = "signal_hub.timeline_event_log";

#[derive(Clone)]
pub struct SignalHubReplayService {
    signal_store: SignalHubStore,
    signal_service: SignalHubSignalService,
    event_store: EventStore,
}

impl SignalHubReplayService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        let signal_service = SignalHubSignalService::new(signal_store.clone(), event_store.clone());
        Self {
            signal_store,
            signal_service,
            event_store,
        }
    }

    pub async fn request_replay(
        &self,
        request: &SignalReplayRequestCreate,
    ) -> Result<crate::domains::signal_hub::SignalReplayRequest, SignalHubError> {
        let replay_request = self.signal_store.create_replay_request(request).await?;
        self.append_replay_lifecycle_event(
            "signal.replay.requested",
            &replay_request.id,
            json!({
                "status": replay_request.status,
                "source_code": replay_request.source_code,
                "connection_id": replay_request.connection_id,
                "event_pattern": replay_request.event_pattern,
                "target_consumer": replay_request.target_consumer,
                "target_projection": replay_request.target_projection,
                "requested_by": replay_request.requested_by,
                "requested_at": replay_request.requested_at,
                "metadata": replay_request.metadata,
            }),
        )
        .await?;
        Ok(replay_request)
    }

    pub async fn process_next_request(
        &self,
    ) -> Result<Option<SignalReplayRunReport>, SignalHubError> {
        let Some(request) = self.signal_store.claim_next_replay_request().await? else {
            return Ok(None);
        };

        match self.process_claimed_request(&request).await {
            Ok(report) => Ok(Some(report)),
            Err(error) => {
                self.signal_store
                    .mark_replay_request_failed(&request.id, &error.to_string())
                    .await?;
                self.append_replay_lifecycle_event(
                    "signal.replay.failed",
                    &request.id,
                    json!({
                        "status": "failed",
                        "error": error.to_string(),
                    }),
                )
                .await?;
                Err(error)
            }
        }
    }

    async fn process_claimed_request(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<SignalReplayRunReport, SignalHubError> {
        let mut replayed_count: u32 = 0;
        if let Some(target_projection) = request.target_projection.as_deref() {
            replayed_count = self.rebuild_projection(target_projection, request).await?;
        } else if let Some(target_consumer) = request.target_consumer.as_deref() {
            let replay_events = self.list_consumer_replay_events(request).await?;
            self.prepare_consumer_replay(target_consumer, &replay_events)
                .await?;
            replayed_count = u32::try_from(replay_events.len()).unwrap_or(u32::MAX);
        } else if uses_event_log_replay(request) {
            let replay_events = self.list_event_log_events_for_replay(request).await?;
            for replay_event in replay_events {
                self.signal_service
                    .replay_raw_signal(&replay_event.event)
                    .await?;
                replayed_count = replayed_count.saturating_add(1);
            }
        } else {
            let paused_events = self
                .signal_store
                .list_paused_events_for_replay(request, DEFAULT_REPLAY_BATCH_SIZE)
                .await?;

            for paused_event in paused_events {
                self.signal_service
                    .replay_raw_signal(&paused_event.event)
                    .await?;
                self.signal_store
                    .release_paused_event(&paused_event.event_id)
                    .await?;
                replayed_count = replayed_count.saturating_add(1);
            }
        }

        self.signal_store
            .mark_replay_request_completed(
                &request.id,
                i32::try_from(replayed_count).unwrap_or(i32::MAX),
            )
            .await?;
        self.append_replay_lifecycle_event(
            "signal.replay.completed",
            &request.id,
            json!({
                "status": "completed",
                "replayed_count": replayed_count,
                "source_code": request.source_code,
                "connection_id": request.connection_id,
                "event_pattern": request.event_pattern,
                "target_consumer": request.target_consumer,
                "target_projection": request.target_projection,
                "from_position": request.from_position,
                "to_position": request.to_position,
                "from_time": request.from_time.map(|value| value.to_rfc3339()),
                "to_time": request.to_time.map(|value| value.to_rfc3339()),
            }),
        )
        .await?;

        Ok(SignalReplayRunReport {
            request_id: request.id.clone(),
            replayed_count,
        })
    }

    async fn rebuild_projection(
        &self,
        target_projection: &str,
        request: &SignalReplayRequest,
    ) -> Result<u32, SignalHubError> {
        match target_projection {
            COMMUNICATION_MESSAGES_PROJECTION => {
                self.rebuild_communication_messages_projection(request)
                    .await
            }
            PERSON_DERIVED_EVIDENCE_PROJECTION => {
                self.rebuild_person_derived_evidence_projection(request)
                    .await
            }
            PROJECT_LINK_REVIEW_EFFECTS_PROJECTION => {
                self.rebuild_project_link_review_effects_projection(request)
                    .await
            }
            REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION => {
                self.rebuild_realtime_conversation_transcript_projection(request)
                    .await
            }
            YANDEX_TELEMOST_CALENDAR_MATCHING_PROJECTION => {
                self.rebuild_yandex_telemost_calendar_matching_projection(request)
                    .await
            }
            ZOOM_CALENDAR_MATCHING_PROJECTION => {
                self.rebuild_zoom_calendar_matching_projection(request)
                    .await
            }
            TIMELINE_EVENT_LOG_PROJECTION => self.rebuild_timeline_projection(request).await,
            other => Err(SignalHubError::InvalidReplayRequest(format!(
                "unsupported target_projection: {other}"
            ))),
        }
    }

    async fn append_replay_lifecycle_event(
        &self,
        event_type: &str,
        replay_request_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!("evt_{}_{}", event_type.replace('.', "_"), replay_request_id),
            event_type,
            chrono::Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": "system",
                "source_id": replay_request_id,
            }),
            json!({
                "kind": "signal_replay_request",
                "entity_id": replay_request_id,
            }),
        )
        .payload(payload)
        .correlation_id(replay_request_id)
        .build()?;

        self.event_store
            .append_for_dispatch_idempotent(&event)
            .await?;
        Ok(())
    }

    async fn list_event_log_events_for_replay(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<Vec<StoredEventEnvelope>, SignalHubError> {
        let events = self.list_matching_signal_events(request).await?;
        Ok(events
            .into_iter()
            .filter(|event| event.event.event_type.starts_with("signal.raw."))
            .collect())
    }

    async fn list_consumer_replay_events(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<Vec<StoredEventEnvelope>, SignalHubError> {
        self.list_matching_signal_events(request).await
    }

    async fn list_matching_signal_events(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<Vec<StoredEventEnvelope>, SignalHubError> {
        let mut query = EventLogQuery::default().limit(DEFAULT_REPLAY_BATCH_SIZE);
        if let Some(source_code) = request.source_code.as_deref() {
            query = query.source_code(source_code);
        }
        if let (Some(from_position), Some(to_position)) =
            (request.from_position, request.to_position)
        {
            query = query.position_between(from_position, to_position);
        } else {
            if let Some(from_position) = request.from_position {
                query = query.position_after(from_position);
            }
            if let Some(to_position) = request.to_position {
                query = query.position_before(to_position);
            }
        }
        if let (Some(from_time), Some(to_time)) = (request.from_time, request.to_time) {
            query = query.occurred_between(from_time, to_time);
        } else {
            query.occurred_after = request.from_time;
            query.occurred_before = request.to_time;
        }

        let events = self.event_store.list_matching(query).await?;
        let mut filtered_events = Vec::new();
        for event in events.into_iter().filter(|event| {
            event.event.event_type.starts_with("signal.")
                && request.event_pattern.as_deref().is_none_or(|pattern| {
                    crate::domains::signal_hub::event_type_pattern_matches(
                        pattern,
                        &event.event.event_type,
                    )
                })
        }) {
            if let
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/task_creation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/task_creation.rs`
- Size bytes / Размер в байтах: `51`
- Included characters / Включено символов: `51`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::task_creation::*;
```

### `backend/src/application/telegram_runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/telegram_runtime.rs`
- Size bytes / Размер в байтах: `9426`
- Included characters / Включено символов: `9426`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use crate::integrations::telegram::client::models::messages::{
    TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest,
};
use crate::integrations::telegram::client::{TelegramChatMember, TelegramError, TelegramStore};
use crate::integrations::telegram::runtime::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse, TelegramMediaDownloadContext, TelegramMediaDownloadRequest,
    TelegramMediaDownloadResponse, TelegramMemberSyncContext, TelegramProviderSearchRequest,
    TelegramRuntimeEventBridgeContext, TelegramRuntimeManager, TelegramRuntimeOperationDeps,
    TelegramRuntimeRestartRequest, TelegramRuntimeStartContext, TelegramRuntimeStartRequest,
    TelegramRuntimeStatus, TelegramRuntimeStopRequest,
};
use crate::platform::config::AppConfig;
use crate::platform::events::EventBus;
use crate::platform::secrets::SecretReferenceStore;
use crate::vault::HostVault;

pub(crate) struct TelegramRuntimeUseCaseContext<'a> {
    pub(crate) provider_account_store: CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    pub(crate) telegram_store: TelegramStore,
    pub(crate) secret_store: SecretReferenceStore,
    pub(crate) secret_resolver: &'a HostVault,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bus: &'a EventBus,
    pub(crate) runtime: &'a TelegramRuntimeManager,
}

pub(crate) struct TelegramRuntimeUseCaseStores {
    pub(crate) provider_account_store: CommunicationProviderAccountStore,
    pub(crate) provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    pub(crate) telegram_store: TelegramStore,
    pub(crate) secret_store: SecretReferenceStore,
}

pub(crate) struct TelegramRuntimeUseCaseRuntime<'a> {
    pub(crate) secret_resolver: &'a HostVault,
    pub(crate) config: &'a AppConfig,
    pub(crate) event_bus: &'a EventBus,
    pub(crate) runtime: &'a TelegramRuntimeManager,
}

impl<'a> TelegramRuntimeUseCaseContext<'a> {
    pub(crate) fn new(
        stores: TelegramRuntimeUseCaseStores,
        runtime: TelegramRuntimeUseCaseRuntime<'a>,
    ) -> Self {
        Self {
            provider_account_store: stores.provider_account_store,
            provider_secret_binding_store: stores.provider_secret_binding_store,
            telegram_store: stores.telegram_store,
            secret_store: stores.secret_store,
            secret_resolver: runtime.secret_resolver,
            config: runtime.config,
            event_bus: runtime.event_bus,
            runtime: runtime.runtime,
        }
    }

    fn event_bridge_context(&self) -> TelegramRuntimeEventBridgeContext {
        TelegramRuntimeEventBridgeContext::new(
            Some(self.telegram_store.clone()),
            self.event_bus.clone(),
        )
    }

    fn operation_deps(&self) -> TelegramRuntimeOperationDeps<'_, HostVault> {
        TelegramRuntimeOperationDeps {
            provider_account_store: &self.provider_account_store,
            provider_secret_binding_store: &self.provider_secret_binding_store,
            telegram_store: &self.telegram_store,
            secret_store: &self.secret_store,
            secret_resolver: self.secret_resolver,
            config: self.config,
            event_bridge: Some(self.event_bridge_context()),
        }
    }
}

pub(crate) async fn runtime_status(
    context: &TelegramRuntimeUseCaseContext<'_>,
    account_id: &str,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    context
        .runtime
        .status_for_account(&context.provider_account_store, context.config, account_id)
        .await
}

pub(crate) async fn start_runtime(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramRuntimeStartRequest,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    let start_context = TelegramRuntimeStartContext {
        provider_account_store: &context.provider_account_store,
        provider_secret_binding_store: &context.provider_secret_binding_store,
        telegram_store: &context.telegram_store,
        secret_store: &context.secret_store,
        secret_resolver: context.secret_resolver,
        config: context.config,
        event_bus: context.event_bus,
    };
    context.runtime.start_account(&start_context, request).await
}

pub(crate) async fn stop_runtime(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramRuntimeStopRequest,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    context
        .runtime
        .stop_account_runtime(&context.provider_account_store, context.config, request)
        .await
}

pub(crate) async fn restart_runtime(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramRuntimeRestartRequest,
) -> Result<TelegramRuntimeStatus, TelegramError> {
    let start_context = TelegramRuntimeStartContext {
        provider_account_store: &context.provider_account_store,
        provider_secret_binding_store: &context.provider_secret_binding_store,
        telegram_store: &context.telegram_store,
        secret_store: &context.secret_store,
        secret_resolver: context.secret_resolver,
        config: context.config,
        event_bus: context.event_bus,
    };
    context
        .runtime
        .restart_account_runtime(&start_context, request)
        .await
}

pub(crate) async fn sync_chat_members(
    context: &TelegramRuntimeUseCaseContext<'_>,
    telegram_chat_id: &str,
) -> Result<Vec<TelegramChatMember>, TelegramError> {
    context
        .runtime
        .sync_chat_members(
            TelegramMemberSyncContext {
                provider_account_store: &context.provider_account_store,
                provider_secret_binding_store: &context.provider_secret_binding_store,
                telegram_store: &context.telegram_store,
                secret_store: &context.secret_store,
                secret_resolver: context.secret_resolver,
                config: context.config,
                event_bridge: Some(context.event_bridge_context()),
            },
            telegram_chat_id,
        )
        .await
}

pub(crate) async fn sync_chats(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramChatSyncRequest,
) -> Result<TelegramChatSyncResponse, TelegramError> {
    context
        .runtime
        .sync_chats_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn sync_history(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramHistorySyncRequest,
) -> Result<TelegramHistorySyncResponse, TelegramError> {
    context
        .runtime
        .sync_history_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn send_manual_message(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramManualSendRequest,
) -> Result<TelegramManualSendResponse, TelegramError> {
    context
        .runtime
        .send_manual_message_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn send_reply_message(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramReplyRequest,
) -> Result<TelegramManualSendResponse, TelegramError> {
    context
        .runtime
        .send_reply_message_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn send_forward_message(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramForwardRequest,
) -> Result<TelegramManualSendResponse, TelegramError> {
    context
        .runtime
        .send_forward_message_with_deps(context.operation_deps(), request)
        .await
}

pub(crate) async fn refresh_provider_search(
    context: &TelegramRuntimeUseCaseContext<'_>,
    account_id: String,
    provider_chat_id: Option<String>,
    query: String,
    limit: i32,
) -> Result<(), TelegramError> {
    context
        .runtime
        .search_provider_messages_with_deps(
            context.operation_deps(),
            &TelegramProviderSearchRequest {
                account_id,
                provider_chat_id,
                query,
                limit,
            },
        )
        .await
        .map(|_| ())
}

pub(crate) async fn refresh_forum_topics(
    context: &TelegramRuntimeUseCaseContext<'_>,
    telegram_chat_id: &str,
) -> Result<(), TelegramError> {
    context
        .runtime
        .sync_forum_topics_with_deps(context.operation_deps(), telegram_chat_id)
        .await
        .map(|_| ())
}

pub(crate) async fn download_media(
    context: &TelegramRuntimeUseCaseContext<'_>,
    request: &TelegramMediaDownloadRequest,
) -> Result<TelegramMediaDownloadResponse, TelegramError> {
    context
        .runtime
        .download_media(
            TelegramMediaDownloadContext {
                provider_account_store: &context.provider_account_store,
                provider_secret_binding_store: &context.provider_secret_binding_store,
                telegram_store: &context.telegram_store,
                secret_store: &context.secret_store,
                secret_resolver: context.secret_resolver,
                config: context.config,
                event_bridge: Some(context.event_bridge_context()),
            },
            request,
        )
        .await
}
```

### `backend/src/application/whatsapp_command_executor.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/whatsapp_command_executor.rs`
- Size bytes / Размер в байтах: `74387`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domains::communications::storage::{
    CommunicationStorageStore, LocalCommunicationBlobStore,
};
use crate::integrations::whatsapp::client::{
    NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage, NewWhatsappWebMessageDelete,
    NewWhatsappWebMessageUpdate, NewWhatsappWebReaction, NewWhatsappWebStatus,
};
use crate::integrations::whatsapp::runtime::{
    WhatsAppProviderApiAccessToken, WhatsAppProviderCommandExecutionError,
    WhatsAppProviderExecutableCommand, WhatsAppProviderInMemoryMediaBytes,
    WhatsAppProviderMediaDownloadRef, WhatsAppProviderWriteCommand,
    claim_due_business_cloud_commands_for_execution, claim_due_commands_for_execution,
    claim_due_native_md_commands_for_execution, dead_letter_failed_command,
    import_canonical_provider_commands, record_live_provider_command_submitted,
    recover_stale_fixture_executing_commands, recover_stale_live_executing_commands,
    reschedule_failed_command, whatsapp_business_cloud_access_token_secret_ref,
    whatsapp_native_md_media_download_secret_ref,
};
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::events::bus::whatsapp_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};
use crate::vault::{HostVault, HostVaultError};

use super::communication_fixture_ingest::CommunicationFixtureIngestError;
use super::communication_fixture_ingest::WhatsappFixtureIngestApplicationService;
use super::provider_runtime_contracts::WhatsAppProviderRuntimeRef;

const WHATSAPP_COMMAND_EXECUTOR_RUNTIME: &str = "whatsapp_command_executor";
static WHATSAPP_COMMAND_EXECUTOR_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(crate) async fn execute_due_fixture_commands(
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    event_store: EventStore,
    event_bus: EventBus,
    limit: i64,
) {
    let now = Utc::now();
    match import_canonical_provider_commands(&pool, now, limit).await {
        Ok(commands) => {
            for command in commands {
                let _ = publish_command_event(
                    &event_store,
                    &event_bus,
                    whatsapp_event_types::COMMAND_STATUS_CHANGED,
                    &command,
                    json!({"source": "canonical_provider_command_import"}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp command executor: canonical import failed");
        }
    }

    match recover_stale_fixture_executing_commands(&pool, now).await {
        Ok(commands) => {
            for command in commands {
                let _ = publish_command_event(
                    &event_store,
                    &event_bus,
                    whatsapp_event_types::COMMAND_STATUS_CHANGED,
                    &command,
                    json!({"source": "stale_recovery"}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp command executor: stale recovery failed");
        }
    }

    let commands = match claim_due_commands_for_execution(&pool, now, limit).await {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp command executor: failed to claim commands");
            return;
        }
    };
    if commands.is_empty() {
        return;
    }

    let fixture_ingest = WhatsappFixtureIngestApplicationService::new(
        pool.clone(),
        runtime,
        event_store.clone(),
        event_bus.clone(),
    );

    for command in commands {
        let _ = publish_command_event(
            &event_store,
            &event_bus,
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            &command,
            json!({"source": "command_executor", "phase": "claimed"}),
        )
        .await;
        let _ = publish_media_execution_started_event(&fixture_ingest, &command).await;
        if let Err(error) = execute_claimed_command(&fixture_ingest, &command).await {
            tracing::warn!(
                error = %error,
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp command executor: command execution failed"
            );
            let _ = publish_media_execution_failed_event(&fixture_ingest, &command, &error).await;
            match reschedule_failed_command(
                &pool,
                &command.command_id,
                Utc::now(),
                &error.to_string(),
                None,
                None,
            )
            .await
            {
                Ok(Some(updated)) => {
                    let _ = publish_command_event(
                        &event_store,
                        &event_bus,
                        whatsapp_event_types::COMMAND_STATUS_CHANGED,
                        &updated,
                        json!({"source": "command_executor", "error": error.to_string()}),
                    )
                    .await;
                }
                Ok(None) => {}
                Err(update_error) => {
                    tracing::warn!(
                        error = %update_error,
                        command_id = %command.command_id,
                        "whatsapp command executor: failed to reschedule failed command"
                    );
                }
            }
        }
    }
}

pub(crate) async fn execute_due_live_native_md_commands(
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    vault: HostVault,
    event_store: EventStore,
    event_bus: EventBus,
    limit: i64,
) {
    let now = Utc::now();
    match recover_stale_live_executing_commands(&pool, now, None).await {
        Ok(commands) => {
            for command in commands {
                let _ = publish_command_event(
                    &event_store,
                    &event_bus,
                    whatsapp_event_types::COMMAND_STATUS_CHANGED,
                    &command,
                    json!({"source": "stale_live_recovery"}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "whatsapp native command executor: stale recovery failed");
        }
    }

    let commands = match claim_due_native_md_commands_for_execution(&pool, now, limit).await {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp native command executor: failed to claim commands"
            );
            return;
        }
    };
    if commands.is_empty() {
        return;
    }

    let media_event_ingest = WhatsappFixtureIngestApplicationService::new(
        pool.clone(),
        runtime.clone(),
        event_store.clone(),
        event_bus.clone(),
    );

    for command in commands {
        let _ = publish_command_event(
            &event_store,
            &event_bus,
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            &command,
            json!({"source": "native_md_command_executor", "phase": "claimed"}),
        )
        .await;
        let _ = publish_media_execution_started_event(&media_event_ingest, &command).await;
        let mut executable = WhatsAppProviderExecutableCommand::from(&command);
        if let Err(error) =
            prepare_live_native_md_media_upload(&pool, &command, &mut executable).await
        {
            tracing::warn!(
                error_code = error.error_code.as_deref().unwrap_or("unknown"),
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp native command executor: media upload preparation failed"
            );
            record_live_native_md_command_failure(
                &pool,
                &event_store,
                &event_bus,
                &media_event_ingest,
                &command,
                &error,
            )
            .await;
            continue;
        }
        if let Err(error) =
            prepare_live_native_md_media_download(&vault, &command, &mut executable).await
        {
            tracing::warn!(
                error_code = error.error_code.as_deref().unwrap_or("unknown"),
                command_id = %command.command_id,
                command_kind = %command.command_kind,
                "whatsapp native command executor: media download preparation failed"
            );
            record_live_native_md_command_failure(
                &pool,
                &event_store,
                &event_bus,
                &media_event_ingest,
                &command,
                &error,
            )
            .await;
            continue;
        }
        match runtime.execute_live_provider_command(&executable).await {
            Ok(outcome) => {
                if command.command_kind == "download_media" {
                    if let Err(error) = persist_live_native_md_media_download(
                        &media_event_ingest,
                        &command,
                        &outcome,
                    )
                    .await
                    {
                        tracing::warn!(
                            error_code = error.error_code.as_deref().unwrap_or("unknown"),
                            command_id = %command.command_id,
                            command_kind = %command.command_kind,
                            "whatsapp native command executor: media download persistence failed"
                        );
                        record_live_native_md_command_failure(
                            &pool,
                            &event_store,
                            &event_bus,
                            &media_event_ingest,
                            &command,
                            &error,
                        )
                        .await;
                    }
                    continue;
                }
                match record_live_provider_command_submitted(&pool, Utc::now(), &outcome).await {
                    Ok(Some(updated)) => {
                        let _ = publish_media_execution_progress(
                            &media_event_ingest,
                            &command,
                            "submitted_to_provider_awaiting_observed_evidence",
                            95,
                            None,
                            None,
                            None,
                        )
                        .await;
                        let _ = publish_command_event(
                            &event_store,
                            &event_bus,
                            whatsapp_event_types::COMMAND_STATUS_CHANGED,
                            &updated,
                            json!({
                                "source": "native_md_command_executor",
                                "phase": "submitted_to_provider",
                                "provider_request_id": outcome.provider_request_id,
                                "completion_rule": "provider_observed_event_reconciliation_required",
                                "payload_policy": "sanitized_metadata_only",
                            }),
                        )
                        .await;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            command_id = %command.command_id,
                            "whatsapp native command executor: failed to re
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/whatsapp_provider_observation_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/whatsapp_provider_observation_reconciliation.rs`
- Size bytes / Размер в байтах: `25763`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::application::provider_runtime_contracts::{
    NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage, NewWhatsappWebMessageDelete,
    NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant, NewWhatsappWebReaction,
    NewWhatsappWebReceipt, NewWhatsappWebStatus, WhatsAppProviderCommand,
};
use crate::domains::communications::core::CommunicationProviderAccountStore;
use crate::integrations::whatsapp::client::WhatsappWebDeliveryState;
use crate::platform::communications::StoredRawCommunicationRecord;
use crate::platform::events::bus::whatsapp_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope, StoredEventEnvelope};

pub(crate) const WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_CONSUMER: &str =
    "whatsapp_provider_observation_reconciliation";

pub(crate) async fn reconcile_whatsapp_provider_observation_event(
    pool: PgPool,
    event_bus: EventBus,
    event: StoredEventEnvelope,
) -> Result<(), String> {
    if !supports_whatsapp_provider_reconciliation_event(&event.event.event_type) {
        return Ok(());
    }

    let account_store = CommunicationProviderAccountStore::new(pool.clone());
    let raw_record_id = required_subject_str(&event.event.subject, "raw_record_id")?;
    let raw_record =
        crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone())
            .raw_record(raw_record_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("WhatsApp raw record `{raw_record_id}` not found"))?;

    let Some(account) = account_store
        .get(&raw_record.account_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };
    if !account.provider_kind.is_whatsapp() {
        return Ok(());
    }

    let runtime = crate::application::whatsapp_provider_runtime(pool.clone());
    let commands = match event.event.event_type.as_str() {
        "signal.accepted.whatsapp.message" => {
            runtime
                .reconcile_fixture_message_commands(&raw_record_to_whatsapp_message(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.reaction" => {
            runtime
                .reconcile_fixture_reaction_commands(&raw_record_to_whatsapp_reaction(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.receipt" => {
            runtime
                .reconcile_fixture_receipt_commands(&raw_record_to_whatsapp_receipt(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.media" => {
            runtime
                .reconcile_fixture_media_commands(&raw_record_to_whatsapp_media(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.status" => {
            runtime
                .reconcile_fixture_status_commands(&raw_record_to_whatsapp_status(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.dialog" => {
            runtime
                .reconcile_fixture_dialog_commands(&raw_record_to_whatsapp_dialog(&raw_record)?)
                .await
        }
        "signal.accepted.whatsapp.participant" => {
            runtime
                .reconcile_fixture_participant_commands(&raw_record_to_whatsapp_participant(
                    &raw_record,
                )?)
                .await
        }
        "signal.accepted.whatsapp.message_update" => {
            runtime
                .reconcile_fixture_message_update_commands(&raw_record_to_whatsapp_message_update(
                    &raw_record,
                )?)
                .await
        }
        "signal.accepted.whatsapp.message_delete" => {
            runtime
                .reconcile_fixture_message_delete_commands(&raw_record_to_whatsapp_message_delete(
                    &raw_record,
                )?)
                .await
        }
        _ => return Ok(()),
    }
    .map_err(|error| error.to_string())?;

    let event_store = EventStore::new(pool);
    for command in commands {
        publish_whatsapp_command_events(
            &event_store,
            &event_bus,
            &command,
            "provider_observation_consumer",
        )
        .await?;
    }
    Ok(())
}

fn supports_whatsapp_provider_reconciliation_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.whatsapp.message"
            | "signal.accepted.whatsapp.reaction"
            | "signal.accepted.whatsapp.receipt"
            | "signal.accepted.whatsapp.media"
            | "signal.accepted.whatsapp.status"
            | "signal.accepted.whatsapp.dialog"
            | "signal.accepted.whatsapp.participant"
            | "signal.accepted.whatsapp.message_update"
            | "signal.accepted.whatsapp.message_delete"
    )
}

async fn publish_whatsapp_command_events(
    event_store: &EventStore,
    event_bus: &EventBus,
    command: &WhatsAppProviderCommand,
    source: &str,
) -> Result<(), String> {
    let payload = json!({
        "account_id": command.account_id,
        "command_id": command.command_id,
        "idempotency_key": command.idempotency_key,
        "command_kind": command.command_kind,
        "action": command.command_kind,
        "provider_chat_id": command.provider_chat_id,
        "provider_message_id": command.provider_message_id,
        "capability_state": command.capability_state,
        "action_class": command.action_class,
        "confirmation_decision": command.confirmation_decision,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "audit_metadata": command.audit_metadata,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "source": source,
    });
    publish_whatsapp_command_event(
        event_store,
        event_bus,
        whatsapp_event_types::COMMAND_STATUS_CHANGED,
        command,
        payload.clone(),
    )
    .await?;
    publish_whatsapp_command_event(
        event_store,
        event_bus,
        whatsapp_event_types::COMMAND_RECONCILED,
        command,
        payload,
    )
    .await?;
    Ok(())
}

async fn publish_whatsapp_command_event(
    event_store: &EventStore,
    event_bus: &EventBus,
    event_type: &str,
    command: &WhatsAppProviderCommand,
    payload: Value,
) -> Result<(), String> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}:{}",
        command.command_id,
        command.command_kind,
        command.status,
        event_type,
        now.timestamp_micros()
    );
    let event = NewEventEnvelope::builder(
        format!(
            "evt_whatsapp_command_{}_{}_{}",
            event_type.replace('.', "_"),
            command.command_id,
            Uuid::now_v7()
        ),
        event_type.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": command.account_id,
            "actor_id": "makosh-frontend",
            "kind": "whatsapp_provider_commands",
            "source_id": source_id,
        }),
        json!({
            "id": command.command_id,
            "entity_id": command.command_id,
            "kind": "whatsapp_provider_command",
        }),
    )
    .payload(payload)
    .build()
    .map_err(|error| error.to_string())?;
    event_store
        .append(&event)
        .await
        .map_err(|error| error.to_string())?;
    let _ = event_bus.broadcast(event);
    Ok(())
}

fn raw_record_to_whatsapp_message(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebMessage, String> {
    Ok(NewWhatsappWebMessage {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: raw_record.provider_record_id.clone(),
        chat_title: required_payload_str(&raw_record.payload, "chat_title")?.to_owned(),
        sender_id: required_payload_str(&raw_record.payload, "sender_id")?.to_owned(),
        sender_display_name: required_payload_str(&raw_record.payload, "sender_display_name")?
            .to_owned(),
        text: required_payload_str(&raw_record.payload, "text")?.to_owned(),
        reply_to_provider_message_id: optional_payload_str(
            &raw_record.payload,
            "reply_to_provider_message_id",
        )
        .map(str::to_owned),
        forward_origin_chat_id: optional_payload_str(&raw_record.payload, "forward_origin_chat_id")
            .map(str::to_owned),
        forward_origin_message_id: optional_payload_str(
            &raw_record.payload,
            "forward_origin_message_id",
        )
        .map(str::to_owned),
        forward_origin_sender_id: optional_payload_str(
            &raw_record.payload,
            "forward_origin_sender_id",
        )
        .map(str::to_owned),
        forward_origin_sender_name: optional_payload_str(
            &raw_record.payload,
            "forward_origin_sender_name",
        )
        .map(str::to_owned),
        forwarded_at: optional_payload_datetime(&raw_record.payload, "forwarded_at")?,
        message_metadata: raw_record
            .payload
            .get("message_metadata")
            .cloned()
            .unwrap_or_else(|| json!({})),
        import_batch_id: raw_record.import_batch_id.clone(),
        occurred_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
        delivery_state: parse_delivery_state(required_payload_str(
            &raw_record.payload,
            "delivery_state",
        )?)?,
    })
}

fn raw_record_to_whatsapp_reaction(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebReaction, String> {
    Ok(NewWhatsappWebReaction {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: required_payload_str(&raw_record.payload, "provider_message_id")?
            .to_owned(),
        provider_actor_id: required_payload_str(&raw_record.payload, "provider_actor_id")?
            .to_owned(),
        sender_display_name: required_payload_str(&raw_record.payload, "sender_display_name")?
            .to_owned(),
        reaction: required_payload_str(&raw_record.payload, "reaction")?.to_owned(),
        is_active: required_payload_bool(&raw_record.payload, "is_active")?,
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: raw_record.occurred_at.unwrap_or(raw_record.captured_at),
    })
}

fn raw_record_to_whatsapp_receipt(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebReceipt, String> {
    Ok(NewWhatsappWebReceipt {
        account_id: raw_record.account_id.clone(),
        provider_chat_id: required_payload_str(&raw_record.payload, "provider_chat_id")?.to_owned(),
        provider_message_id: raw_record.provider_record_id.clone(),
        delivery_state: required_delivery_state(&raw_record.payload)?,
        import_batch_id: raw_record.import_batch_id.clone(),
        observed_at: required_datetime(
            optional_payload_datetime(&raw_record.payload, "observed_at")?,
            "observed_at",
        )?,
    })
}

fn raw_record_to_whatsapp_media(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewWhatsappWebMedia, String> {
    Ok(NewWhatsappWeb
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/whatsapp_runtime_event_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/whatsapp_runtime_event_projection.rs`
- Size bytes / Размер в байтах: `17243`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Map, Value, json};
use sqlx::postgres::PgPool;

use crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus;
use crate::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderAccountStore,
    CommunicationProviderSecretBindingStore, ProviderAccount, ProviderAccountSecretPurpose,
};
use crate::domains::signal_hub::{SignalHubConnectionService, SignalHubError, SignalHubStore};
use crate::platform::events::EventStore;
use crate::platform::events::StoredEventEnvelope;
use crate::platform::secrets::SecretReferenceStore;
use crate::vault::{HostVault, HostVaultError};

pub(crate) const WHATSAPP_RUNTIME_EVENT_CONSUMER: &str = "whatsapp_runtime_event_projection";

pub(crate) async fn sync_whatsapp_runtime_signal_connection_for_pool(
    pool: &PgPool,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
    secret_ref: Option<String>,
) -> Result<(), SignalHubError> {
    let signal_store = SignalHubStore::new(pool.clone());
    let connection_service =
        SignalHubConnectionService::new(signal_store.clone(), EventStore::new(pool.clone()));
    signal_store.restore_system_sources().await?;
    let settings = merged_whatsapp_runtime_connection_settings(
        signal_store
            .find_connection_by_account("whatsapp", &account.account_id)
            .await?
            .as_ref()
            .map(|connection| &connection.settings),
        account,
        status,
    );
    connection_service
        .upsert_account_connection(
            "whatsapp",
            &account.account_id,
            &account.display_name,
            whatsapp_runtime_signal_status(status),
            settings,
            secret_ref,
        )
        .await?;
    Ok(())
}

pub(crate) async fn project_whatsapp_runtime_event(
    pool: PgPool,
    vault: HostVault,
    event: StoredEventEnvelope,
) -> Result<(), String> {
    if event.event.event_type != "signal.accepted.whatsapp.runtime_event" {
        return Ok(());
    }

    let raw_record_id = required_subject_str(&event.event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("WhatsApp runtime event raw record `{raw_record_id}` not found"))?;

    let decision = reconcile_decision_from_payload(&raw_record.payload);
    let Some(decision) = decision else {
        return Ok(());
    };

    let account_store = CommunicationProviderAccountStore::new(pool.clone());
    let binding_store = CommunicationProviderSecretBindingStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool.clone());
    let runtime = crate::application::whatsapp_provider_runtime(pool.clone());

    let Some(current_account) = account_store
        .get(&raw_record.account_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };
    if !current_account.provider_kind.is_whatsapp() {
        return Ok(());
    }

    if decision.clear_restorable_session {
        clear_whatsapp_restorable_session(
            &binding_store,
            &secret_store,
            &vault,
            &raw_record.account_id,
        )
        .await?;
    }

    if let Some(account_lifecycle_state) = decision.account_lifecycle_state {
        update_whatsapp_account_lifecycle_state(
            &account_store,
            &current_account,
            account_lifecycle_state,
            raw_record.occurred_at.unwrap_or(raw_record.captured_at),
        )
        .await?;
    }

    if let Some(session_link_state) = decision.session_link_state {
        update_whatsapp_session_projection_state(
            &pool,
            &raw_record.account_id,
            session_link_state,
            raw_record.occurred_at.unwrap_or(raw_record.captured_at),
        )
        .await?;
    }

    let status = runtime
        .runtime_status(&secret_store, &vault, &raw_record.account_id)
        .await
        .map_err(|error| error.to_string())?;

    if status.status == "removed" {
        remove_whatsapp_signal_connection(&pool, &raw_record.account_id).await?;
    } else {
        let account = account_store
            .get(&raw_record.account_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| {
                format!(
                    "WhatsApp account `{}` disappeared during runtime-event reconciliation",
                    raw_record.account_id
                )
            })?;
        sync_whatsapp_runtime_signal_connection_for_pool(
            &pool,
            &account,
            &status,
            status.session_secret_ref.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuntimeEventReconcileDecision {
    account_lifecycle_state: Option<&'static str>,
    session_link_state: Option<&'static str>,
    clear_restorable_session: bool,
}

fn reconcile_decision_from_payload(payload: &Value) -> Option<RuntimeEventReconcileDecision> {
    let lifecycle_state = payload.get("lifecycle_state").and_then(Value::as_str);
    let runtime_status = payload.get("runtime_status").and_then(Value::as_str);
    let effective_state = lifecycle_state.or(runtime_status)?.trim();
    if effective_state.is_empty() {
        return None;
    }
    reconcile_decision_from_effective_state(effective_state)
}

fn reconcile_decision_from_effective_state(
    effective_state: &str,
) -> Option<RuntimeEventReconcileDecision> {
    match effective_state {
        "qr_pending" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("qr_pending"),
            session_link_state: Some("qr_pending"),
            clear_restorable_session: false,
        }),
        "pair_code_pending" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("pair_code_pending"),
            session_link_state: Some("pair_code_pending"),
            clear_restorable_session: false,
        }),
        "linked" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("linked"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "available" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("available"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "syncing" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("syncing"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "degraded" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("degraded"),
            session_link_state: Some("linked"),
            clear_restorable_session: false,
        }),
        "link_required" | "created" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("created"),
            session_link_state: Some("link_required"),
            clear_restorable_session: true,
        }),
        "revoked" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("revoked"),
            session_link_state: Some("revoked"),
            clear_restorable_session: true,
        }),
        "removed" => Some(RuntimeEventReconcileDecision {
            account_lifecycle_state: Some("removed"),
            session_link_state: Some("removed"),
            clear_restorable_session: true,
        }),
        _ => None,
    }
}

async fn update_whatsapp_account_lifecycle_state(
    account_store: &CommunicationProviderAccountStore,
    account: &crate::platform::communications::ProviderAccount,
    lifecycle_state: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), String> {
    let mut config = account.config.clone();
    let config_object = config
        .as_object_mut()
        .ok_or_else(|| "WhatsApp provider account config must be an object".to_owned())?;
    config_object.insert("lifecycle_state".to_owned(), json!(lifecycle_state));
    match lifecycle_state {
        "created" => {
            config_object.remove("revoked_at");
            config_object.remove("removed_at");
        }
        "revoked" => {
            config_object.insert("revoked_at".to_owned(), json!(observed_at));
            config_object.remove("removed_at");
        }
        "removed" => {
            config_object.insert("removed_at".to_owned(), json!(observed_at));
        }
        _ => {
            config_object.remove("revoked_at");
            config_object.remove("removed_at");
        }
    }
    account_store
        .update_config_with_origin(
            &account.account_id,
            &config,
            crate::platform::observations::ObservationOriginKind::LocalRuntime,
            "application.whatsapp_runtime_event_projection",
            "runtime_event_reconcile",
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn update_whatsapp_session_projection_state(
    pool: &PgPool,
    account_id: &str,
    link_state: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE whatsapp_web_sessions
        SET link_state = $2,
            last_sync_at = COALESCE($3, last_sync_at),
            updated_at = now()
        WHERE account_id = $1
        "#,
    )
    .bind(account_id)
    .bind(link_state)
    .bind(observed_at)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn clear_whatsapp_restorable_session(
    binding_store: &CommunicationProviderSecretBindingStore,
    secret_store: &SecretReferenceStore,
    vault: &HostVault,
    account_id: &str,
) -> Result<(), String> {
    let Some(binding) = binding_store
        .get_for_account(
            account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
        )
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };

    binding_store
        .unbind_for_account(
            account_id,
            ProviderAccountSecretPurpose::WhatsappWebSessionKey,
        )
        .await
        .map_err(|error| error.to_string())?;
    secret_store
        .delete_secret_reference(&binding.secret_ref)
        .await
        .map_err(|error| error.to_string())?;
    match vault.delete_secret(&binding.secret_ref) {
        Ok(_) => {}
        Err(HostVaultError::MissingSecret { .. }) => {}
        Err(error) => return Err(error.to_string()),
    }
    Ok(())
}

async fn remove_whatsapp_signal_connection(pool: &PgPool, account_id: &str) -> Result<(), String> {
    let signal_store = crate::domains::signal_hub::SignalHubStore::new(pool.clone());
    let connection_service = crate::domains::signal_hub::SignalHubConnectionService::new(
        signal_store,
        crate::platform::events::EventStore::new(pool.clone()),
    );
    connection_service
        .remove_account_connection("whatsapp", account_id)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn whatsapp_runtime_signal_status(status: &WhatsAppRuntimeStatus) -> &'static str {
    match status.status.as_str() {
        "removed" => "removed",
        "revoked" | "link_required" | "created" | "blocked" => "awaiting_user_action",
        "qr_pending" | "pair_code_pending" | "syncing" | "degraded" => "connecting",
        "available" | "linked" => "connected",
        _ => {
            if status.session_restore_available {
                "connected"
            } else {
                "awaiting_user_action"
            }
        }
    }
}

fn merged_whatsapp_runtime_connection_settings(
    current: Option<&Value>,
    account: &ProviderAccount,
    status: &
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/whatsapp_runtime_signal_ingest.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/whatsapp_runtime_signal_ingest.rs`
- Size bytes / Размер в байтах: `17998`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionPort,
};
use crate::domains::signal_hub::{SignalHubError, dispatch_whatsapp_raw_signal};
use crate::integrations::whatsapp::runtime::{
    WhatsAppRuntimeEventSink, WhatsAppRuntimeEventSinkError, WhatsAppRuntimeEventSinkFuture,
    WhatsAppSanitizedRuntimeEventDto,
};
use crate::platform::communications::NewRawCommunicationRecord;

#[derive(Clone)]
pub(crate) struct WhatsappRuntimeSignalIngestService {
    pool: PgPool,
}

impl WhatsappRuntimeSignalIngestService {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn ingest_sanitized_runtime_event(
        &self,
        dto: WhatsAppSanitizedRuntimeEventDto,
    ) -> Result<WhatsappRuntimeSignalIngestResult, WhatsappRuntimeSignalIngestError> {
        dto.assert_event_spine_contract();
        let raw = native_runtime_raw_record(&dto);
        let stored_raw = CommunicationIngestionPort::new(self.pool.clone())
            .record_raw_source(&raw)
            .await?;
        let Some(accepted_event) =
            dispatch_whatsapp_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(WhatsappRuntimeSignalIngestError::SignalControlBlocked);
        };
        if accepted_event.event_type != dto.accepted_event_kind {
            return Err(
                WhatsappRuntimeSignalIngestError::AcceptedEventKindMismatch {
                    expected: dto.accepted_event_kind,
                    actual: accepted_event.event_type,
                },
            );
        }
        Ok(WhatsappRuntimeSignalIngestResult {
            raw_record_id: stored_raw.raw_record_id,
            accepted_event_id: accepted_event.event_id,
        })
    }
}

impl WhatsAppRuntimeEventSink for WhatsappRuntimeSignalIngestService {
    fn accept<'a>(
        &'a self,
        dto: WhatsAppSanitizedRuntimeEventDto,
    ) -> WhatsAppRuntimeEventSinkFuture<'a> {
        Box::pin(async move {
            self.ingest_sanitized_runtime_event(dto)
                .await
                .map(|_| ())
                .map_err(WhatsappRuntimeSignalIngestError::into_sink_error)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WhatsappRuntimeSignalIngestResult {
    pub(crate) raw_record_id: String,
    pub(crate) accepted_event_id: String,
}

#[derive(Debug, Error)]
pub(crate) enum WhatsappRuntimeSignalIngestError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error("whatsapp native runtime signal was blocked by Signal Hub controls")]
    SignalControlBlocked,

    #[error("accepted event kind mismatch: expected {expected}, got {actual}")]
    AcceptedEventKindMismatch {
        expected: &'static str,
        actual: String,
    },
}

impl WhatsappRuntimeSignalIngestError {
    fn into_sink_error(self) -> WhatsAppRuntimeEventSinkError {
        match self {
            Self::Communication(_) => {
                WhatsAppRuntimeEventSinkError::new("native_md_raw_record_append_failed")
            }
            Self::SignalHub(_) => {
                WhatsAppRuntimeEventSinkError::new("native_md_signal_hub_dispatch_failed")
            }
            Self::SignalControlBlocked => {
                WhatsAppRuntimeEventSinkError::new("native_md_signal_hub_control_blocked")
            }
            Self::AcceptedEventKindMismatch { .. } => {
                WhatsAppRuntimeEventSinkError::new("native_md_signal_hub_event_kind_mismatch")
            }
        }
    }
}

fn native_runtime_raw_record(dto: &WhatsAppSanitizedRuntimeEventDto) -> NewRawCommunicationRecord {
    let payload = native_runtime_raw_payload(dto);
    NewRawCommunicationRecord::new(
        native_runtime_raw_record_id(dto),
        dto.account_id.clone(),
        dto.raw_record_kind,
        dto.provider_event_id.clone(),
        native_runtime_source_fingerprint(dto),
        "whatsapp_native_md_live",
        payload,
    )
    .provenance(json!({
        "provider": dto.provider_shape,
        "provider_shape": dto.provider_shape,
        "runtime_driver": dto.runtime_driver,
        "account_id": dto.account_id,
        "provider_event_name": dto.provider_event_name,
        "event_family": dto.event_family,
        "observed_source": dto.bridge_dispatch.observed_source,
        "runtime_bridge": {
            "endpoint_path": dto.bridge_dispatch.endpoint_path,
            "request_kind": dto.bridge_dispatch.request_kind,
            "observed_source": dto.bridge_dispatch.observed_source,
        },
        "raw_signal_event_kind": dto.raw_signal_event_kind,
        "accepted_event_kind": dto.accepted_event_kind,
        "payload_policy": "sanitized_metadata_only",
        "captured_by": "application.whatsapp_runtime_signal_ingest",
    }))
}

fn native_runtime_raw_payload(dto: &WhatsAppSanitizedRuntimeEventDto) -> Value {
    let mut payload = json!({
        "provider_event_id": dto.provider_event_id,
        "provider_shape": dto.provider_shape,
        "runtime_driver": dto.runtime_driver,
        "provider_event_name": dto.provider_event_name,
        "event_family": dto.event_family,
        "runtime_event_kind": format!("native_md.{}", dto.event_family),
        "metadata": redact_secret_like_metadata(dto.metadata.clone()),
    });

    if dto.raw_record_kind == "whatsapp_web_runtime_event" {
        let (runtime_status, lifecycle_state, severity) = runtime_event_state(dto);
        payload["runtime_status"] = json!(runtime_status);
        payload["lifecycle_state"] = json!(lifecycle_state);
        payload["severity"] = json!(severity);
    }

    payload
}

fn runtime_event_state(
    dto: &WhatsAppSanitizedRuntimeEventDto,
) -> (&'static str, &'static str, &'static str) {
    if let Some(override_state) = sanitized_runtime_state_override(dto) {
        return override_state;
    }
    match dto.provider_event_name {
        "PairingQrCode" => ("qr_pending", "qr_pending", "info"),
        "PairingCode" => ("pair_code_pending", "pair_code_pending", "info"),
        "PairSuccess" => ("linked", "linked", "info"),
        "Connected" => ("available", "available", "info"),
        "HistorySync" | "OfflineSyncPreview" | "OfflineSyncCompleted" => {
            ("syncing", "syncing", "info")
        }
        "LoggedOut" => ("revoked", "revoked", "warning"),
        "Disconnected" | "StreamReplaced" | "TemporaryBan" | "ConnectFailure" | "StreamError" => {
            ("degraded", "degraded", "warning")
        }
        _ => ("degraded", "degraded", "warning"),
    }
}

fn sanitized_runtime_state_override(
    dto: &WhatsAppSanitizedRuntimeEventDto,
) -> Option<(&'static str, &'static str, &'static str)> {
    let runtime_status = dto.metadata.get("runtime_status").and_then(Value::as_str)?;
    let lifecycle_state = dto
        .metadata
        .get("lifecycle_state")
        .and_then(Value::as_str)?;
    let severity = dto.metadata.get("severity").and_then(Value::as_str)?;
    Some((
        allowed_runtime_status(runtime_status)?,
        allowed_lifecycle_state(lifecycle_state)?,
        allowed_runtime_severity(severity)?,
    ))
}

fn allowed_runtime_status(value: &str) -> Option<&'static str> {
    match value {
        "available" => Some("available"),
        "degraded" => Some("degraded"),
        "revoked" => Some("revoked"),
        "stopped" => Some("stopped"),
        "syncing" => Some("syncing"),
        "linked" => Some("linked"),
        "qr_pending" => Some("qr_pending"),
        "pair_code_pending" => Some("pair_code_pending"),
        _ => None,
    }
}

fn allowed_lifecycle_state(value: &str) -> Option<&'static str> {
    match value {
        "available" => Some("available"),
        "degraded" => Some("degraded"),
        "recovering" => Some("recovering"),
        "revoked" => Some("revoked"),
        "stopped" => Some("stopped"),
        "syncing" => Some("syncing"),
        "linked" => Some("linked"),
        "qr_pending" => Some("qr_pending"),
        "pair_code_pending" => Some("pair_code_pending"),
        _ => None,
    }
}

fn allowed_runtime_severity(value: &str) -> Option<&'static str> {
    match value {
        "info" => Some("info"),
        "warning" => Some("warning"),
        "blocked" => Some("blocked"),
        _ => None,
    }
}

fn native_runtime_raw_record_id(dto: &WhatsAppSanitizedRuntimeEventDto) -> String {
    stable_whatsapp_native_id(
        "raw:v5:whatsapp_native_md",
        &[
            dto.account_id.as_str(),
            dto.raw_record_kind,
            dto.provider_event_id.as_str(),
        ],
    )
}

fn native_runtime_source_fingerprint(dto: &WhatsAppSanitizedRuntimeEventDto) -> String {
    stable_whatsapp_native_id("sha256", &[dto.source_fingerprint_seed.as_str()])
}

fn stable_whatsapp_native_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.trim().as_bytes());
        hasher.update(b"\0");
    }
    format!("{prefix}:{:x}", hasher.finalize())
}

fn redact_secret_like_metadata(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    if is_secret_like_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_like_metadata(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_like_metadata).collect())
        }
        other => other,
    }
}

fn is_secret_like_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "access_token"
            | "refresh_token"
            | "session_key"
            | "session_material"
            | "authorization"
            | "cookie"
            | "token"
            | "secret"
            | "secret_key"
            | "media_key"
            | "direct_path"
            | "static_url"
            | "url"
            | "password"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use testkit::context::TestContext;

    use super::*;
    use crate::domains::communications::core::{
        CommunicationProviderAccountStore, CommunicationProviderKind, NewProviderAccount,
    };
    use crate::integrations::whatsapp::runtime::WhatsAppRuntimeBridgeDispatch;
    use crate::platform::storage::Database;

    #[tokio::test]
    async fn sanitized_native_runtime_event_enters_raw_evidence_and_signal_hub_idempotently() {
        let test_context = TestContext::new().await;
        let database_url = test_context.connection_string();
        let database = Database::connect(Some(&database_url))
            .await
            .expect("database connection");
        let pool = database.pool().expect("configured pool").clone();
        let account_id = "whatsapp-native-sink-account";
        let provider_event_id = "native-md-provider-event-1";

        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(
                &NewProviderAccount::new(
                    account_id,
                    CommunicationProviderKind::WhatsappWeb,
                    "WhatsApp Native Sink",
                    "wa-native-sink",
                )
                .config(json!({
                    "provider_shape": "whatsapp_native_md",
                    "runtime_kind": "native_md",
                })),
            )
            .await
            .expect("provider account");

        let service = WhatsappRuntimeSignalIngestService::new(pool.clone());
        let dto = WhatsAppSanitizedRuntim
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/workflow_action_person_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/workflow_action_person_projection.rs`
- Size bytes / Размер в байтах: `71`
- Included characters / Включено символов: `71`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::workflow_action_person_projection::*;
```

### `backend/src/application/yandex_telemost_calendar_matching.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/yandex_telemost_calendar_matching.rs`
- Size bytes / Размер в байтах: `71`
- Included characters / Включено символов: `71`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::yandex_telemost_calendar_matching::*;
```

### `backend/src/application/zoom_calendar_matching.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/zoom_calendar_matching.rs`
- Size bytes / Размер в байтах: `60`
- Included characters / Включено символов: `60`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::zoom_calendar_matching::*;
```

### `backend/src/application/zoom_participant_identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/zoom_participant_identity.rs`
- Size bytes / Размер в байтах: `63`
- Included characters / Включено символов: `63`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::zoom_participant_identity::*;
```

### `backend/src/application/zoom_signal_detection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/zoom_signal_detection.rs`
- Size bytes / Размер в байтах: `59`
- Included characters / Включено символов: `59`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::zoom_signal_detection::*;
```

### `backend/src/bin/makosh_document_process.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_document_process.rs`
- Size bytes / Размер в байтах: `1761`
- Included characters / Включено символов: `1761`
- Truncated / Обрезано: `no`

```rust
use std::env;

use makosh_hub_backend::domains::documents::processing::{
    DocumentProcessingRunReport, DocumentProcessingStore,
};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use serde::Serialize;
use thiserror::Error;

const DEFAULT_LIMIT: i64 = 25;

#[derive(Debug, Serialize)]
struct DocumentProcessCommandReport {
    runner: DocumentProcessingRunReport,
    requested_limit: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    makosh_hub_backend::app::init_tracing();

    let config = AppConfig::from_env()?;
    let database_url = config
        .database_url()
        .ok_or(DocumentProcessCommandError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(DocumentProcessCommandError::MissingDatabaseUrl)?
        .clone();
    let store = DocumentProcessingStore::new(pool);

    let requested_limit = env::args()
        .nth(1)
        .as_deref()
        .map_or(Ok(DEFAULT_LIMIT), |value| {
            value.parse::<i64>().map_err(|_| {
                DocumentProcessCommandError::InvalidLimit(format!(
                    "limit argument must be integer: {value}"
                ))
            })
        })?;

    let runner = store.run_queued_jobs(requested_limit).await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&DocumentProcessCommandReport {
            runner,
            requested_limit,
        })?
    );

    Ok(())
}

#[derive(Debug, Error)]
enum DocumentProcessCommandError {
    #[error("DATABASE_URL is required for document processing")]
    MissingDatabaseUrl,

    #[error("{0}")]
    InvalidLimit(String),
}
```

### `backend/src/bin/makosh_email_fixture_dev.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_fixture_dev.rs`
- Size bytes / Размер в байтах: `5154`
- Included characters / Включено символов: `5154`
- Truncated / Обрезано: `no`

```rust
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use makosh_hub_backend::domains::communications::core::EmailProviderKind;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::email_fixture_pipeline::{
    EmailFixturePipelineRequest, import_fixture_email_messages_for_dev,
    project_fixture_email_messages,
};
use thiserror::Error;

const DEFAULT_FIXTURE_PATH: &str = "tmp/email-fixtures/icloud-inbox-redacted.json";
const DEFAULT_ACCOUNT_ID: &str = "dev-icloud-fixture";
const DEFAULT_DISPLAY_NAME: &str = "iCloud Redacted Fixture";
const DEFAULT_EXTERNAL_ACCOUNT_ID: &str = "redacted-icloud@example.invalid";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    makosh_hub_backend::app::init_tracing();

    let config = EmailFixtureDevConfig::from_env()?;
    let app_config = AppConfig::from_env()?;
    let database_url = app_config
        .database_url()
        .ok_or(EmailFixtureDevConfigError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(EmailFixtureDevConfigError::MissingDatabaseUrl)?
        .clone();
    let fixture_json = fs::read_to_string(&config.fixture_path).map_err(|source| {
        EmailFixtureDevConfigError::FixtureRead {
            path: config.fixture_path.clone(),
            source,
        }
    })?;
    let request = EmailFixturePipelineRequest::new(
        config.account_id,
        config.display_name,
        config.external_account_id,
        config.provider_kind,
        config.import_batch_id,
        fixture_json,
    );

    match config.mode {
        EmailFixtureDevMode::Import => {
            let report = import_fixture_email_messages_for_dev(pool, &request).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        EmailFixtureDevMode::Project => {
            let report = project_fixture_email_messages(pool, &request).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}

struct EmailFixtureDevConfig {
    mode: EmailFixtureDevMode,
    fixture_path: PathBuf,
    account_id: String,
    display_name: String,
    external_account_id: String,
    provider_kind: EmailProviderKind,
    import_batch_id: String,
}

impl EmailFixtureDevConfig {
    fn from_env() -> Result<Self, EmailFixtureDevConfigError> {
        let mode = parse_mode(
            optional_env("MAKOSH_EMAIL_FIXTURE_MODE")
                .unwrap_or_else(|| "project".to_owned())
                .as_str(),
        )?;
        let provider_kind = parse_provider_kind(
            optional_env("MAKOSH_EMAIL_FIXTURE_PROVIDER")
                .unwrap_or_else(|| "icloud".to_owned())
                .as_str(),
        )?;

        Ok(Self {
            mode,
            fixture_path: PathBuf::from(
                optional_env("MAKOSH_EMAIL_FIXTURE_PATH")
                    .unwrap_or_else(|| DEFAULT_FIXTURE_PATH.to_owned()),
            ),
            account_id: optional_env("MAKOSH_EMAIL_FIXTURE_ACCOUNT_ID")
                .unwrap_or_else(|| DEFAULT_ACCOUNT_ID.to_owned()),
            display_name: optional_env("MAKOSH_EMAIL_FIXTURE_DISPLAY_NAME")
                .unwrap_or_else(|| DEFAULT_DISPLAY_NAME.to_owned()),
            external_account_id: optional_env("MAKOSH_EMAIL_FIXTURE_EXTERNAL_ACCOUNT_ID")
                .unwrap_or_else(|| DEFAULT_EXTERNAL_ACCOUNT_ID.to_owned()),
            provider_kind,
            import_batch_id: optional_env("MAKOSH_EMAIL_FIXTURE_IMPORT_BATCH_ID")
                .unwrap_or_else(|| format!("fixture-dev-{}", Utc::now().timestamp())),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmailFixtureDevMode {
    Import,
    Project,
}

fn parse_mode(value: &str) -> Result<EmailFixtureDevMode, EmailFixtureDevConfigError> {
    match value.trim() {
        "import" => Ok(EmailFixtureDevMode::Import),
        "project" => Ok(EmailFixtureDevMode::Project),
        other => Err(EmailFixtureDevConfigError::InvalidMode(other.to_owned())),
    }
}

fn parse_provider_kind(value: &str) -> Result<EmailProviderKind, EmailFixtureDevConfigError> {
    EmailProviderKind::try_from(value.trim())
        .map_err(|_| EmailFixtureDevConfigError::InvalidProviderKind(value.to_owned()))
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Error)]
enum EmailFixtureDevConfigError {
    #[error("DATABASE_URL is required for email fixture dev commands")]
    MissingDatabaseUrl,

    #[error("failed to read fixture file `{path}`: {source}")]
    FixtureRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid MAKOSH_EMAIL_FIXTURE_MODE `{0}`; expected `import` or `project`")]
    InvalidMode(String),

    #[error("invalid MAKOSH_EMAIL_FIXTURE_PROVIDER `{0}`; expected `gmail`, `icloud` or `imap`")]
    InvalidProviderKind(String),
}
```

### `backend/src/bin/makosh_email_fixture_export.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_fixture_export.rs`
- Size bytes / Размер в байтах: `7138`
- Included characters / Включено символов: `7138`
- Truncated / Обрезано: `no`

```rust
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use makosh_hub_backend::domains::communications::core::EmailProviderKind;
use makosh_hub_backend::domains::communications::fixtures::export::{
    EmailFixtureExportOptions, export_fixture_messages_from_sync_batch,
};
use makosh_hub_backend::integrations::mail::gmail::client::{ImapFetchOptions, ImapNetworkClient};
use makosh_hub_backend::platform::secrets::ResolvedSecret;
use serde::Serialize;
use thiserror::Error;

const DEFAULT_ICLOUD_IMAP_HOST: &str = "imap.mail.me.com";
const DEFAULT_IMAP_PORT: u16 = 993;
const DEFAULT_MAILBOX: &str = "INBOX";
const DEFAULT_MAX_MESSAGES: usize = 10;
const DEFAULT_OUTPUT_PATH: &str = "tmp/email-fixtures/icloud-inbox-redacted.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    makosh_hub_backend::app::init_tracing();

    let config = LiveImapFixtureConfig::from_env()?;
    ensure_parent_dir(&config.output_path)?;

    let password = ResolvedSecret::new(config.password)?;
    let mut options = ImapFetchOptions::new(
        config.host,
        config.port,
        config.tls,
        config.mailbox.clone(),
        config.username,
    )
    .provider_kind(EmailProviderKind::Icloud)
    .max_messages(config.max_messages)
    .latest_messages();

    if let Some(last_seen_uid) = config.last_seen_uid {
        options = options.last_seen_uid(last_seen_uid);
    }

    let batch = ImapNetworkClient::new()
        .fetch_raw_messages(&password, &options)
        .await?;
    let checkpoint = batch.checkpoint.clone();
    let fixtures =
        export_fixture_messages_from_sync_batch(&batch, EmailFixtureExportOptions::default())?;
    let exported_messages = fixtures.len();

    fs::write(
        &config.output_path,
        serde_json::to_string_pretty(&fixtures)?,
    )?;

    println!(
        "{}",
        serde_json::to_string_pretty(&LiveImapFixtureExportReport {
            provider: "icloud",
            mailbox: &config.mailbox,
            exported_messages,
            output_path: config.output_path.display().to_string(),
            redaction: "redacted",
            checkpoint,
        })?
    );

    Ok(())
}

struct LiveImapFixtureConfig {
    username: String,
    password: String,
    host: String,
    port: u16,
    tls: bool,
    mailbox: String,
    max_messages: usize,
    last_seen_uid: Option<u32>,
    output_path: PathBuf,
}

impl LiveImapFixtureConfig {
    fn from_env() -> Result<Self, LiveImapFixtureConfigError> {
        Ok(Self {
            username: first_env(["MAKOSH_IMAP_FIXTURE_USERNAME", "ICLOUD_LOGIN"])?,
            password: first_env(["MAKOSH_IMAP_FIXTURE_PASSWORD", "ICLOUD_2FA"])?,
            host: optional_env("MAKOSH_IMAP_FIXTURE_HOST")
                .unwrap_or_else(|| DEFAULT_ICLOUD_IMAP_HOST.to_owned()),
            port: optional_env("MAKOSH_IMAP_FIXTURE_PORT")
                .map(|value| parse_port("MAKOSH_IMAP_FIXTURE_PORT", &value))
                .transpose()?
                .unwrap_or(DEFAULT_IMAP_PORT),
            tls: optional_env("MAKOSH_IMAP_FIXTURE_TLS")
                .map(|value| parse_bool("MAKOSH_IMAP_FIXTURE_TLS", &value))
                .transpose()?
                .unwrap_or(true),
            mailbox: optional_env("MAKOSH_IMAP_FIXTURE_MAILBOX")
                .unwrap_or_else(|| DEFAULT_MAILBOX.to_owned()),
            max_messages: optional_env("MAKOSH_IMAP_FIXTURE_MAX_MESSAGES")
                .map(|value| parse_usize("MAKOSH_IMAP_FIXTURE_MAX_MESSAGES", &value))
                .transpose()?
                .unwrap_or(DEFAULT_MAX_MESSAGES),
            last_seen_uid: optional_env("MAKOSH_IMAP_FIXTURE_LAST_SEEN_UID")
                .map(|value| parse_u32("MAKOSH_IMAP_FIXTURE_LAST_SEEN_UID", &value))
                .transpose()?,
            output_path: PathBuf::from(
                optional_env("MAKOSH_IMAP_FIXTURE_OUTPUT")
                    .unwrap_or_else(|| DEFAULT_OUTPUT_PATH.to_owned()),
            ),
        })
    }
}

#[derive(Serialize)]
struct LiveImapFixtureExportReport<'a> {
    provider: &'a str,
    mailbox: &'a str,
    exported_messages: usize,
    output_path: String,
    redaction: &'a str,
    checkpoint: Option<serde_json::Value>,
}

fn first_env<const N: usize>(
    names: [&'static str; N],
) -> Result<String, LiveImapFixtureConfigError> {
    for name in names.iter().copied() {
        if let Some(value) = optional_env(name) {
            return Ok(value);
        }
    }

    Err(LiveImapFixtureConfigError::MissingEnv(names.join(" or ")))
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse_port(name: &'static str, value: &str) -> Result<u16, LiveImapFixtureConfigError> {
    let port = parse_u16(name, value)?;
    if port == 0 {
        return Err(LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(port)
}

fn parse_bool(name: &'static str, value: &str) -> Result<bool, LiveImapFixtureConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected one of true/false/yes/no/1/0",
        }),
    }
}

fn parse_usize(name: &'static str, value: &str) -> Result<usize, LiveImapFixtureConfigError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected positive integer",
        })?;
    if parsed == 0 {
        return Err(LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(parsed)
}

fn parse_u32(name: &'static str, value: &str) -> Result<u32, LiveImapFixtureConfigError> {
    value
        .parse::<u32>()
        .map_err(|_| LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected u32 integer",
        })
}

fn parse_u16(name: &'static str, value: &str) -> Result<u16, LiveImapFixtureConfigError> {
    value
        .parse::<u16>()
        .map_err(|_| LiveImapFixtureConfigError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected u16 integer",
        })
}

fn ensure_parent_dir(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[derive(Debug, Error)]
enum LiveImapFixtureConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("invalid {name} value `{value}`: {message}")]
    InvalidEnv {
        name: &'static str,
        value: String,
        message: &'static str,
    },
}
```

### `backend/src/bin/makosh_email_reproject_dev.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_reproject_dev.rs`
- Size bytes / Размер в байтах: `7076`
- Included characters / Включено символов: `7070`
- Truncated / Обрезано: `no`

```rust
use std::env;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use makosh_hub_backend::app::init_tracing;
use makosh_hub_backend::domains::communications::core::StoredRawCommunicationRecord;
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, parse_raw_email_message_from_blob, project_parsed_raw_email_message,
};
use makosh_hub_backend::domains::communications::storage::LocalCommunicationBlobStore;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = ReprojectDevConfig::from_env()?;
    let app_config = AppConfig::from_env()?;
    let database_url = app_config
        .database_url()
        .ok_or(ReprojectDevError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(ReprojectDevError::MissingDatabaseUrl)?
        .clone();
    let blob_store = LocalCommunicationBlobStore::new(&config.blob_root);
    let message_store = MessageProjectionStore::new(pool.clone());
    let raw_records = email_blob_records_for_reprojection(
        &pool,
        config.account_id.as_deref(),
        config.only_corrupt,
    )
    .await?;

    let mut reprojected_messages = 0usize;
    let mut failed_records = 0usize;
    for raw_record in &raw_records {
        match parse_raw_email_message_from_blob(&blob_store, raw_record).await {
            Ok(parsed) => {
                match project_parsed_raw_email_message(&message_store, raw_record, &parsed).await {
                    Ok(_) => reprojected_messages += 1,
                    Err(error) => {
                        failed_records += 1;
                        eprintln!(
                            "mail reproject failed raw_record_id={} error={}",
                            raw_record.raw_record_id, error
                        );
                    }
                }
            }
            Err(error) => {
                failed_records += 1;
                eprintln!(
                    "mail raw parse failed raw_record_id={} error={}",
                    raw_record.raw_record_id, error
                );
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&ReprojectDevReport {
            account_id: config.account_id.as_deref(),
            only_corrupt: config.only_corrupt,
            blob_root: config.blob_root.display().to_string(),
            selected_records: raw_records.len(),
            reprojected_messages,
            failed_records,
        })?
    );

    if failed_records > 0 {
        return Err(ReprojectDevError::FailedRecords {
            count: failed_records,
        }
        .into());
    }

    Ok(())
}

async fn email_blob_records_for_reprojection(
    pool: &sqlx::PgPool,
    account_id: Option<&str>,
    only_corrupt: bool,
) -> Result<Vec<StoredRawCommunicationRecord>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            r.raw_record_id,
            r.observation_id,
            r.account_id,
            r.record_kind,
            r.provider_record_id,
            r.source_fingerprint,
            r.import_batch_id,
            r.occurred_at,
            r.captured_at,
            r.payload,
            r.provenance
        FROM communication_raw_records r
        JOIN communication_messages m ON m.raw_record_id = r.raw_record_id
        WHERE r.record_kind = 'email_message'
          AND r.payload->>'raw_blob_storage_kind' = 'local_fs'
          AND r.payload ? 'raw_blob_storage_path'
          AND ($1::text IS NULL OR r.account_id = $1)
          AND (
              $2::bool = false
              OR m.subject LIKE '%�%'
              OR m.sender LIKE '%�%'
              OR m.body_text LIKE '%�%'
          )
        ORDER BY r.captured_at ASC, r.raw_record_id ASC
        "#,
    )
    .bind(account_id)
    .bind(only_corrupt)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(StoredRawCommunicationRecord {
                raw_record_id: row.try_get("raw_record_id")?,
                observation_id: row.try_get("observation_id")?,
                account_id: row.try_get("account_id")?,
                record_kind: row.try_get("record_kind")?,
                provider_record_id: row.try_get("provider_record_id")?,
                source_fingerprint: row.try_get("source_fingerprint")?,
                import_batch_id: row.try_get("import_batch_id")?,
                occurred_at: row.try_get::<Option<DateTime<Utc>>, _>("occurred_at")?,
                captured_at: row.try_get("captured_at")?,
                payload: row.try_get::<Value, _>("payload")?,
                provenance: row.try_get::<Value, _>("provenance")?,
            })
        })
        .collect()
}

struct ReprojectDevConfig {
    account_id: Option<String>,
    only_corrupt: bool,
    blob_root: PathBuf,
}

impl ReprojectDevConfig {
    fn from_env() -> Result<Self, ReprojectDevError> {
        Ok(Self {
            account_id: optional_env("MAKOSH_EMAIL_REPROJECT_ACCOUNT_ID"),
            only_corrupt: optional_env("MAKOSH_EMAIL_REPROJECT_ONLY_CORRUPT")
                .map(|value| parse_bool("MAKOSH_EMAIL_REPROJECT_ONLY_CORRUPT", &value))
                .transpose()?
                .unwrap_or(true),
            blob_root: PathBuf::from(
                optional_env("MAKOSH_EMAIL_REPROJECT_BLOB_ROOT")
                    .unwrap_or_else(|| DEFAULT_MAIL_SYNC_BLOB_ROOT.to_owned()),
            ),
        })
    }
}

#[derive(Serialize)]
struct ReprojectDevReport<'a> {
    account_id: Option<&'a str>,
    only_corrupt: bool,
    blob_root: String,
    selected_records: usize,
    reprojected_messages: usize,
    failed_records: usize,
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse_bool(name: &'static str, value: &str) -> Result<bool, ReprojectDevError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(ReprojectDevError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected one of true/false/yes/no/1/0",
        }),
    }
}

#[derive(Debug, Error)]
enum ReprojectDevError {
    #[error("DATABASE_URL is required for email reproject dev command")]
    MissingDatabaseUrl,

    #[error("invalid {name} value `{value}`: {message}")]
    InvalidEnv {
        name: &'static str,
        value: String,
        message: &'static str,
    },

    #[error("mail reproject failed for {count} raw records")]
    FailedRecords { count: usize },
}
```

### `backend/src/bin/makosh_email_sync_dev.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_sync_dev.rs`
- Size bytes / Размер в байтах: `895`
- Included characters / Включено символов: `895`
- Truncated / Обрезано: `no`

```rust
#[path = "makosh_email_sync_dev/account.rs"]
mod account;
#[path = "makosh_email_sync_dev/checkpoint.rs"]
mod checkpoint;
#[path = "makosh_email_sync_dev/config.rs"]
mod config;
#[path = "makosh_email_sync_dev/env.rs"]
mod env;
#[path = "makosh_email_sync_dev/errors.rs"]
mod errors;
#[path = "makosh_email_sync_dev/fetch.rs"]
mod fetch;
#[path = "makosh_email_sync_dev/provider.rs"]
mod provider;
#[path = "makosh_email_sync_dev/report.rs"]
mod report;
#[path = "makosh_email_sync_dev/runner.rs"]
mod runner;

use config::DevEmailSyncConfig;
use errors::DevEmailSyncError;
use runner::run_dev_email_sync;

#[tokio::main]
async fn main() -> Result<(), DevEmailSyncError> {
    makosh_hub_backend::app::init_tracing();

    let config = DevEmailSyncConfig::from_env()?;
    let report = run_dev_email_sync(config).await?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
```

### `backend/src/bin/makosh_email_sync_dev/account.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_sync_dev/account.rs`
- Size bytes / Размер в байтах: `804`
- Included characters / Включено символов: `804`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::communications::core::CommunicationProviderAccountStore;
use makosh_hub_backend::domains::communications::core::NewProviderAccount;
use serde_json::json;

use crate::config::DevEmailSyncConfig;
use crate::errors::DevEmailSyncError;

pub(super) async fn upsert_dev_provider_account(
    store: &CommunicationProviderAccountStore,
    config: &DevEmailSyncConfig,
) -> Result<(), DevEmailSyncError> {
    let account = NewProviderAccount::new(
        &config.account_id,
        config.provider_kind,
        &config.display_name,
        &config.external_account_id,
    )
    .config(json!({
        "host": config.host,
        "port": config.port,
        "tls": config.tls,
        "mailbox": config.mailbox
    }));

    store.upsert(&account).await?;

    Ok(())
}
```

### `backend/src/bin/makosh_email_sync_dev/checkpoint.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_sync_dev/checkpoint.rs`
- Size bytes / Размер в байтах: `528`
- Included characters / Включено символов: `528`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::communications::core::CommunicationIngestionStore;

use crate::errors::DevEmailSyncError;

pub(super) async fn last_seen_uid(
    store: &CommunicationIngestionStore,
    account_id: &str,
    stream_id: &str,
) -> Result<Option<u32>, DevEmailSyncError> {
    let checkpoint = store.checkpoint(account_id, stream_id).await?;
    Ok(checkpoint
        .and_then(|checkpoint| checkpoint.checkpoint["last_seen_uid"].as_u64())
        .and_then(|last_seen_uid| u32::try_from(last_seen_uid).ok()))
}
```

### `backend/src/bin/makosh_email_sync_dev/config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_sync_dev/config.rs`
- Size bytes / Размер в байтах: `3461`
- Included characters / Включено символов: `3461`
- Truncated / Обрезано: `no`

```rust
use std::path::PathBuf;

use makosh_hub_backend::domains::communications::core::EmailProviderKind;
use makosh_hub_backend::platform::secrets::ResolvedSecret;

use crate::env::{first_env, optional_env, parse_bool, parse_port, parse_usize};
use crate::errors::DevEmailSyncError;
use crate::provider::{default_host, parse_provider_kind};

const DEFAULT_MAILBOX: &str = "INBOX";
const DEFAULT_MAX_MESSAGES: usize = 25;
const DEFAULT_BLOB_ROOT: &str = "docker/data/mail";

pub(super) struct DevEmailSyncConfig {
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) provider_kind: EmailProviderKind,
    pub(super) username: String,
    pub(super) password: ResolvedSecret,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) mailbox: String,
    pub(super) max_messages: usize,
    pub(super) blob_root: PathBuf,
    pub(super) import_batch_id: String,
}

impl DevEmailSyncConfig {
    pub(super) fn from_env() -> Result<Self, DevEmailSyncError> {
        let provider_kind = parse_provider_kind(
            optional_env("MAKOSH_EMAIL_SYNC_PROVIDER")
                .unwrap_or_else(|| "icloud".to_owned())
                .as_str(),
        )?;
        let username = first_env([
            "MAKOSH_EMAIL_SYNC_USERNAME",
            "MAKOSH_IMAP_FIXTURE_USERNAME",
            "ICLOUD_LOGIN",
        ])?;
        let external_account_id = optional_env("MAKOSH_EMAIL_SYNC_EXTERNAL_ACCOUNT_ID")
            .unwrap_or_else(|| username.clone());
        let password = ResolvedSecret::new(first_env([
            "MAKOSH_EMAIL_SYNC_PASSWORD",
            "MAKOSH_IMAP_FIXTURE_PASSWORD",
            "ICLOUD_2FA",
        ])?)?;

        Ok(Self {
            account_id: optional_env("MAKOSH_EMAIL_SYNC_ACCOUNT_ID")
                .unwrap_or_else(|| format!("dev-{}-mail-cache", provider_kind.as_str())),
            display_name: optional_env("MAKOSH_EMAIL_SYNC_DISPLAY_NAME")
                .unwrap_or_else(|| "Dev Mail Cache".to_owned()),
            external_account_id,
            provider_kind,
            username,
            password,
            host: optional_env("MAKOSH_EMAIL_SYNC_HOST")
                .unwrap_or_else(|| default_host(provider_kind).to_owned()),
            port: optional_env("MAKOSH_EMAIL_SYNC_PORT")
                .map(|value| parse_port("MAKOSH_EMAIL_SYNC_PORT", &value))
                .transpose()?
                .unwrap_or(crate::provider::DEFAULT_IMAP_PORT),
            tls: optional_env("MAKOSH_EMAIL_SYNC_TLS")
                .map(|value| parse_bool("MAKOSH_EMAIL_SYNC_TLS", &value))
                .transpose()?
                .unwrap_or(true),
            mailbox: optional_env("MAKOSH_EMAIL_SYNC_MAILBOX")
                .unwrap_or_else(|| DEFAULT_MAILBOX.to_owned()),
            max_messages: optional_env("MAKOSH_EMAIL_SYNC_MAX_MESSAGES")
                .map(|value| parse_usize("MAKOSH_EMAIL_SYNC_MAX_MESSAGES", &value))
                .transpose()?
                .unwrap_or(DEFAULT_MAX_MESSAGES),
            blob_root: PathBuf::from(
                optional_env("MAKOSH_EMAIL_SYNC_BLOB_ROOT")
                    .unwrap_or_else(|| DEFAULT_BLOB_ROOT.to_owned()),
            ),
            import_batch_id: optional_env("MAKOSH_EMAIL_SYNC_IMPORT_BATCH_ID")
                .unwrap_or_else(|| format!("email-sync-dev-{}", chrono::Utc::now().timestamp())),
        })
    }
}
```

### `backend/src/bin/makosh_email_sync_dev/env.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_sync_dev/env.rs`
- Size bytes / Размер в байтах: `2117`
- Included characters / Включено символов: `2117`
- Truncated / Обрезано: `no`

```rust
use std::env;

use crate::errors::DevEmailSyncError;

pub(super) fn first_env<const N: usize>(
    names: [&'static str; N],
) -> Result<String, DevEmailSyncError> {
    for name in names {
        if let Some(value) = optional_env(name) {
            return Ok(value);
        }
    }
    Err(DevEmailSyncError::MissingEnv(names.join(" or ")))
}

pub(super) fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub(super) fn parse_port(name: &'static str, value: &str) -> Result<u16, DevEmailSyncError> {
    let port = parse_u16(name, value)?;
    if port == 0 {
        return Err(DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(port)
}

pub(super) fn parse_bool(name: &'static str, value: &str) -> Result<bool, DevEmailSyncError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected one of true/false/yes/no/1/0",
        }),
    }
}

pub(super) fn parse_usize(name: &'static str, value: &str) -> Result<usize, DevEmailSyncError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected positive integer",
        })?;
    if parsed == 0 {
        return Err(DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "must be greater than zero",
        });
    }
    Ok(parsed)
}

fn parse_u16(name: &'static str, value: &str) -> Result<u16, DevEmailSyncError> {
    value
        .parse::<u16>()
        .map_err(|_| DevEmailSyncError::InvalidEnv {
            name,
            value: value.to_owned(),
            message: "expected u16 integer",
        })
}
```

### `backend/src/bin/makosh_email_sync_dev/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/bin/makosh_email_sync_dev/errors.rs`
- Size bytes / Размер в байтах: `1589`
- Included characters / Включено символов: `1589`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::communications::core::CommunicationIngestionError;
use makosh_hub_backend::integrations::mail::gmail::client::EmailProviderNetworkError;
use makosh_hub_backend::platform::config::ConfigError;
use makosh_hub_backend::platform::secrets::SecretResolutionError;
use makosh_hub_backend::platform::storage::StorageError;
use makosh_hub_backend::workflows::email_sync_pipeline::EmailSyncPipelineError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum DevEmailSyncError {
    #[error("DATABASE_URL is required for email sync dev command")]
    MissingDatabaseUrl,

    #[error("missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("invalid MAKOSH_EMAIL_SYNC_PROVIDER `{0}`; expected `icloud` or `imap`")]
    InvalidProviderKind(String),

    #[error("Gmail dev sync is not supported by this IMAP-only command")]
    UnsupportedProviderForDevSync,

    #[error("invalid {name} value `{value}`: {message}")]
    InvalidEnv {
        name: &'static str,
        value: String,
        message: &'static str,
    },

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(transparent)]
    ProviderNetwork(#[from] EmailProviderNetworkError),

    #[error(transparent)]
    Pipeline(#[from] EmailSyncPipelineError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```
