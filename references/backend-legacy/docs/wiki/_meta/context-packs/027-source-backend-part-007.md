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

- Chunk ID / ID чанка: `027-source-backend-part-007`
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

### `backend/src/app/handlers/calendar/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/health.rs`
- Size bytes / Размер в байтах: `1183`
- Included characters / Включено символов: `1075`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Calendar Watchtower ────────────────────────────────────────────────────

pub(crate) async fn get_calendar_watchtower(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let preparation = CalendarWatchtowerService::events_needing_preparation(&pool)
        .await
        .map_err(ApiError::from)?;
    let no_outcomes = CalendarWatchtowerService::events_without_outcomes(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        json!({"preparation": preparation, "without_outcomes": no_outcomes}),
    ))
}

pub(crate) async fn get_calendar_health(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let load = CalendarWatchtowerService::meeting_load_analysis(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(load))
}
```

### `backend/src/app/handlers/calendar/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/intelligence.rs`
- Size bytes / Размер в байтах: `4882`
- Included characters / Включено символов: `4772`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Event Intelligence ─────────────────────────────────────────────────────

pub(crate) async fn post_event_classify(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let participants = crate::app::api_support::app_store::<EventParticipantStore>(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let event_type = CalendarIntelligenceService::classify_event(
        &event.title,
        participants.len(),
        (event.end_at - event.start_at).num_minutes(),
    );
    let update = CalendarEventUpdate {
        event_type: Some(event_type.clone()),
        ..Default::default()
    };
    crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .update_runtime(
            &event_id,
            &update,
            "calendar_api.post_event_classify",
            "classify",
        )
        .await?;
    Ok(Json(json!({"event_type": event_type})))
}

pub(crate) async fn post_event_analyze(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let parts = crate::app::api_support::app_store::<EventParticipantStore>(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let has_agenda = crate::app::api_support::app_store::<EventAgendaStore>(pool.clone())
        .get(&event_id)
        .await
        .map(|a| a.is_some())
        .unwrap_or(false);
    let has_checklist = crate::app::api_support::app_store::<EventChecklistStore>(pool.clone())
        .get(&event_id)
        .await
        .map(|c| c.is_some())
        .unwrap_or(false);
    let has_relations = crate::app::api_support::app_store::<EventRelationStore>(pool.clone())
        .list(&event_id)
        .await
        .map(|r| !r.is_empty())
        .unwrap_or(false);

    let importance = CalendarIntelligenceService::calculate_importance(
        &event.title,
        parts.len(),
        has_relations,
        false,
    );
    let readiness = CalendarIntelligenceService::calculate_readiness(
        has_agenda,
        false,
        has_relations,
        has_checklist,
        !parts.is_empty(),
    );
    let risks = CalendarIntelligenceService::detect_risks(
        has_agenda,
        false,
        !parts.is_empty(),
        has_relations,
        event.start_at < Utc::now() + chrono::Duration::hours(24),
    );

    let update = CalendarEventUpdate {
        importance_score: Some(importance),
        readiness_score: Some(readiness),
        ..Default::default()
    };
    crate::app::api_support::app_store::<CalendarEventStore>(pool.clone())
        .update_runtime(
            &event_id,
            &update,
            "calendar_api.post_event_analyze",
            "analyze",
        )
        .await?;

    Ok(Json(
        json!({"importance": importance, "readiness": readiness, "risks": risks}),
    ))
}

pub(crate) async fn get_event_risks(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let parts = crate::app::api_support::app_store::<EventParticipantStore>(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let has_agenda = crate::app::api_support::app_store::<EventAgendaStore>(pool.clone())
        .get(&event_id)
        .await
        .map(|a| a.is_some())
        .unwrap_or(false);
    let has_relations = crate::app::api_support::app_store::<EventRelationStore>(pool.clone())
        .list(&event_id)
        .await
        .map(|r| !r.is_empty())
        .unwrap_or(false);
    let is_soon = event.start_at < Utc::now() + chrono::Duration::hours(24);
    let risks = CalendarIntelligenceService::detect_risks(
        has_agenda,
        false,
        !parts.is_empty(),
        has_relations,
        is_soon,
    );
    Ok(Json(json!({"risks": risks})))
}
```

### `backend/src/app/handlers/calendar/meetings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/meetings.rs`
- Size bytes / Размер в байтах: `6629`
- Included characters / Включено символов: `6281`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Meeting Notes ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct MeetingNotesResponse {
    items: Vec<crate::domains::calendar::meetings::MeetingNote>,
}

pub(crate) async fn get_meeting_notes(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<MeetingNotesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<MeetingNoteStore>(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(MeetingNotesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewNoteRequest {
    content: String,
    format: Option<String>,
    source: Option<String>,
}

pub(crate) async fn post_meeting_note(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewNoteRequest>,
) -> Result<Json<crate::domains::calendar::meetings::MeetingNote>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let note = CalendarCommandService::new(pool)
        .create_meeting_note_manual(
            &event_id,
            &req.content,
            req.format.as_deref(),
            requested_source,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(note))
}

// ── Meeting Outcomes ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct MeetingOutcomesResponse {
    items: Vec<crate::domains::calendar::meetings::MeetingOutcome>,
}

pub(crate) async fn get_meeting_outcomes(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<MeetingOutcomesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<MeetingOutcomeStore>(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(MeetingOutcomesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOutcomeRequest {
    outcome_type: String,
    title: String,
    description: Option<String>,
    owner_person_id: Option<String>,
    due_date: Option<DateTime<Utc>>,
}

pub(crate) async fn post_meeting_outcome(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewOutcomeRequest>,
) -> Result<Json<crate::domains::calendar::meetings::MeetingOutcome>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let outcome = CalendarMeetingOutcomeApplicationService::new(pool)
        .add_manual(
            &event_id,
            &req.outcome_type,
            &req.title,
            req.description.as_deref(),
            req.owner_person_id.as_deref(),
            req.due_date,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(outcome))
}

pub(crate) async fn post_event_follow_up(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<CalendarEventStore>(pool.clone())
        .set_status_manual(
            &event_id,
            "needs_follow_up",
            "calendar_api.post_event_follow_up",
        )
        .await?;
    Ok(Json(json!({"follow_up_created": true})))
}

pub(crate) async fn get_event_follow_up_status(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let status = crate::app::api_support::app_store::<MeetingOutcomeStore>(pool)
        .follow_up_status(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(status))
}

// ── Event Recordings ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRecordingsResponse {
    items: Vec<crate::domains::calendar::meetings::EventRecording>,
}

pub(crate) async fn get_event_recordings(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRecordingsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<EventRecordingStore>(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRecordingsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRecordingRequest {
    file_path: Option<String>,
    source: Option<String>,
    duration_seconds: Option<i32>,
}

pub(crate) async fn post_event_recording(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewRecordingRequest>,
) -> Result<Json<crate::domains::calendar::meetings::EventRecording>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rec = CalendarCommandService::new(pool)
        .add_event_recording_manual(
            &event_id,
            req.file_path.as_deref(),
            requested_source,
            req.duration_seconds,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rec))
}

pub(crate) async fn get_event_transcript(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let t = crate::app::api_support::app_store::<EventTranscriptStore>(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&t).unwrap_or_default()))
}
```

### `backend/src/app/handlers/calendar/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/mod.rs`
- Size bytes / Размер в байтах: `7308`
- Included characters / Включено символов: `7308`
- Truncated / Обрезано: `no`

```rust
// ADR-0073: calendar handlers are split by documented Calendar domain responsibilities.
mod accounts;
mod analytics;
mod brain;
mod events;
mod health;
mod intelligence;
mod meetings;
mod reminders;
mod rules;
mod scheduling;
mod search;
mod sync;

pub(crate) use accounts::*;
pub(crate) use analytics::*;
pub(crate) use brain::*;
pub(crate) use events::*;
pub(crate) use health::*;
pub(crate) use intelligence::*;
pub(crate) use meetings::*;
pub(crate) use reminders::*;
pub(crate) use rules::*;
pub(crate) use scheduling::*;
pub(crate) use search::*;
pub(crate) use sync::*;

use std::io;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;
use url::form_urlencoded;

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
use crate::domains::persons::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::persons::expertise::{PersonExpertiseError, PersonExpertiseStore};
use crate::domains::persons::export::{ExportError, ExportFormat, PersonExportService};
use crate::domains::persons::investigator::{InvestigatorError, PersonInvestigator};
use crate::engines::automation::{
    AutomationError, AutomationPolicy, AutomationStore, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::platform::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewTelegramCall, SpeechToTextProvider,
    TelegramCall, TranscriptStatus,
};
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};
use crate::platform::config::AppConfig;

use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};

use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore, PersonTrustError};

use crate::domains::persons::memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore, PersonMemoryError,
    PersonPreferenceStore, RelationshipEventStore,
};

use crate::domains::persons::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use crate::domains::persons::identity::{
    PersonIdentityCandidate, PersonIdentityDetail, PersonIdentityError,
    PersonIdentityReviewCommand, PersonIdentityReviewState, PersonIdentityStore,
};

use crate::application::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};
use crate::domains::calendar::brain::{CalendarBrainError, CalendarBrainService};
use crate::domains::calendar::core::{
    CalendarCoreError, ContextPackInput, EventAgendaStore, EventChecklistStore,
    EventContextPackStore, EventParticipantStore, EventRelationStore,
};
use crate::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarError, CalendarEventListQuery,
    CalendarEventStore, CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};
use crate::domains::calendar::health::{CalendarHealthError, CalendarWatchtowerService};
use crate::domains::calendar::intelligence::CalendarIntelligenceService;
use crate::domains::calendar::meetings::{
    EventRecordingStore, EventTranscriptStore, MeetingNoteStore, MeetingOutcomeStore, MeetingsError,
};
use crate::domains::calendar::reminders::{CalendarReminderStore, ReminderError};
use crate::domains::calendar::rules::{CalendarRuleError, CalendarRuleStore, RuleUpdate};
use crate::domains::calendar::scheduling::{
    DeadlineStore, FocusBlockStore, SchedulingError, SmartSchedulingService,
};
use crate::domains::calendar::service::{CalendarCommandService, CalendarCommandServiceError};
use crate::domains::calendar::sync::{export_event_ics, export_event_md};
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::domains::organizations::api::{
    OrganizationError, OrganizationStore, OrganizationUpdate,
};
use crate::domains::projects::core::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::domains::tasks::api::{NewTask, TaskError, TaskListQuery, TaskStore, TaskUpdate};
use crate::domains::tasks::brain::{TaskBrainError, TaskBrainService};
use crate::domains::tasks::candidates::{
    TaskCandidate, TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewState,
    TaskCandidateStore,
};
use crate::domains::tasks::core::{
    ExternalTaskIdentityStore, TaskChecklistStore, TaskContextPackStore, TaskCoreError,
    TaskEvidenceStore, TaskProviderStore, TaskRelationStore, TaskSubtaskStore,
};
use crate::domains::tasks::health::{TaskHealthError, TaskWatchtowerService};
use crate::domains::tasks::intelligence::TaskIntelligenceService;
use crate::domains::tasks::rules::{TaskRuleError, TaskRuleStore, TaskTemplateStore};
use crate::domains::tasks::sync::{export_task_json, export_task_md};
use crate::integrations::mail::accounts::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};
use crate::application::CalendarMeetingOutcomeApplicationService;
```

### `backend/src/app/handlers/calendar/reminders.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/reminders.rs`
- Size bytes / Размер в байтах: `2265`
- Included characters / Включено символов: `2149`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Event Reminders ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRemindersResponse {
    items: Vec<crate::domains::calendar::reminders::CalendarReminder>,
}

pub(crate) async fn get_event_reminders(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRemindersResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<CalendarReminderStore>(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRemindersResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewReminderRequest {
    reminder_type: String,
    minutes_before: Option<i32>,
    message: Option<String>,
}

pub(crate) async fn post_event_reminder(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewReminderRequest>,
) -> Result<Json<crate::domains::calendar::reminders::CalendarReminder>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let r = CalendarCommandService::new(pool)
        .create_event_reminder_manual(
            &event_id,
            &req.reminder_type,
            req.minutes_before,
            req.message.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(r))
}

#[derive(Deserialize)]
pub(crate) struct ToggleReminderRequest {
    active: bool,
}

pub(crate) async fn post_event_reminder_toggle(
    State(state): State<AppState>,
    Path((event_id, reminder_id)): Path<(String, String)>,
    Json(req): Json<ToggleReminderRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarCommandService::new(pool)
        .toggle_event_reminder_manual(&event_id, &reminder_id, req.active)
        .await?;
    Ok(Json(json!({"toggled": true, "active": req.active})))
}
```

### `backend/src/app/handlers/calendar/rules.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/rules.rs`
- Size bytes / Размер в байтах: `2485`
- Included characters / Включено символов: `2367`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Calendar Rules ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarRulesResponse {
    items: Vec<crate::domains::calendar::rules::CalendarRule>,
}

pub(crate) async fn get_calendar_rules(
    State(state): State<AppState>,
) -> Result<Json<CalendarRulesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<CalendarRuleStore>(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(CalendarRulesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRuleRequest {
    name: String,
    description: Option<String>,
    dsl: Value,
    approval_mode: Option<String>,
}

pub(crate) async fn post_calendar_rule(
    State(state): State<AppState>,
    Json(req): Json<NewRuleRequest>,
) -> Result<Json<crate::domains::calendar::rules::CalendarRule>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = CalendarCommandService::new(pool)
        .create_calendar_rule_manual(
            &req.name,
            req.description.as_deref(),
            req.dsl,
            req.approval_mode.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn put_calendar_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(update): Json<RuleUpdate>,
) -> Result<Json<crate::domains::calendar::rules::CalendarRule>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = CalendarCommandService::new(pool)
        .update_calendar_rule_manual(&rule_id, &update)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn delete_calendar_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarCommandService::new(pool)
        .delete_calendar_rule_manual(&rule_id)
        .await?;
    Ok(Json(json!({"deleted": true})))
}
```

### `backend/src/app/handlers/calendar/scheduling.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/scheduling.rs`
- Size bytes / Размер в байтах: `4931`
- Included characters / Включено символов: `4563`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Deadlines ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct DeadlinesResponse {
    items: Vec<crate::domains::calendar::scheduling::DeadlineEvent>,
}

#[derive(Deserialize)]
pub(crate) struct DeadlineQuery {
    status: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_deadlines(
    State(state): State<AppState>,
    Query(query): Query<DeadlineQuery>,
) -> Result<Json<DeadlinesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<DeadlineStore>(pool)
        .list(query.status.as_deref(), query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(DeadlinesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewDeadlineRequest {
    title: String,
    due_at: DateTime<Utc>,
    severity: Option<String>,
    source_entity_type: Option<String>,
    source_entity_id: Option<String>,
}

pub(crate) async fn post_deadline(
    State(state): State<AppState>,
    Json(req): Json<NewDeadlineRequest>,
) -> Result<Json<crate::domains::calendar::scheduling::DeadlineEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let d = CalendarCommandService::new(pool)
        .create_deadline_manual(
            &req.title,
            req.due_at,
            req.severity.as_deref(),
            req.source_entity_type.as_deref(),
            req.source_entity_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(d))
}

// ── Focus Blocks ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct FocusBlocksResponse {
    items: Vec<crate::domains::calendar::scheduling::FocusBlock>,
}

#[derive(Deserialize)]
pub(crate) struct FocusBlockQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: Option<i64>,
}

pub(crate) async fn get_focus_blocks(
    State(state): State<AppState>,
    Query(query): Query<FocusBlockQuery>,
) -> Result<Json<FocusBlocksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<FocusBlockStore>(pool)
        .list(query.from, query.to, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(FocusBlocksResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewFocusBlockRequest {
    title: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    purpose: Option<String>,
    linked_project_id: Option<String>,
    protection_level: Option<String>,
}

pub(crate) async fn post_focus_block(
    State(state): State<AppState>,
    Json(req): Json<NewFocusBlockRequest>,
) -> Result<Json<crate::domains::calendar::scheduling::FocusBlock>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let fb = CalendarCommandService::new(pool)
        .create_focus_block_manual(
            &req.title,
            req.start_at,
            req.end_at,
            req.purpose.as_deref(),
            req.linked_project_id.as_deref(),
            req.protection_level.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fb))
}

// ── Smart Schedule ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct SmartScheduleRequest {
    duration_minutes: Option<i64>,
    lookahead_hours: Option<i64>,
}

pub(crate) async fn post_smart_schedule(
    State(state): State<AppState>,
    Json(req): Json<SmartScheduleRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let events = crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .list(&CalendarEventListQuery {
            limit: Some(200),
            ..Default::default()
        })
        .await?;
    let pairs: Vec<(DateTime<Utc>, DateTime<Utc>)> =
        events.iter().map(|e| (e.start_at, e.end_at)).collect();
    let slots = SmartSchedulingService::find_slots(
        &pairs,
        req.duration_minutes.unwrap_or(30),
        req.lookahead_hours.unwrap_or(48),
    );
    Ok(Json(json!({"slots": slots})))
}
```

### `backend/src/app/handlers/calendar/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/search.rs`
- Size bytes / Размер в байтах: `718`
- Included characters / Включено символов: `602`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Calendar Search ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarSearchQuery {
    q: String,
}

pub(crate) async fn get_calendar_search(
    State(state): State<AppState>,
    Query(query): Query<CalendarSearchQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let results = CalendarBrainService::search_events(&pool, &query.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(results))
}
```

### `backend/src/app/handlers/calendar/sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/sync.rs`
- Size bytes / Размер в байтах: `4272`
- Included characters / Включено символов: `4170`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Calendar Import/Export ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarImportRequest {
    ics_data: Option<String>,
    events: Option<Value>,
}

pub(crate) async fn post_calendar_import(
    State(state): State<AppState>,
    Json(req): Json<CalendarImportRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let ics_data_received = req
        .ics_data
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let mut imported = 0;
    if let Some(events) = req.events
        && let Some(arr) = events.as_array()
    {
        for evt in arr {
            let title = evt
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Imported Event");
            let start = evt
                .get("start_at")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or(Utc::now());
            let end = evt
                .get("end_at")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or(start);
            let source_event_id = evt
                .get("source_event_id")
                .and_then(|v| v.as_str())
                .map(ToOwned::to_owned);
            let _ = crate::app::api_support::app_store::<CalendarEventStore>(pool.clone())
                .create_file_import(
                    &NewCalendarEvent {
                        source_event_id,
                        title: title.to_string(),
                        start_at: start,
                        end_at: end,
                        ..Default::default()
                    },
                    &format!("calendar-import://event/{imported}"),
                )
                .await;
            imported += 1;
        }
    }
    Ok(Json(
        json!({"imported": imported, "ics_data_received": ics_data_received}),
    ))
}

pub(crate) async fn post_calendar_sync(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarCommandService::new(pool)
        .trigger_calendar_sync_manual(&account_id)
        .await?;
    Ok(Json(
        json!({"sync_triggered": true, "note": "Provider sync is deferred to future implementation"}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct EventExportQuery {
    format: Option<String>,
}

pub(crate) async fn get_event_export(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(query): Query<EventExportQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let fmt = query.format.as_deref().unwrap_or("json");
    match fmt {
        "ics" => {
            let ics = export_event_ics(
                &event.title,
                event.description.as_deref(),
                event.location.as_deref(),
                &event.start_at.format("%Y%m%dT%H%M%S").to_string(),
                &event.end_at.format("%Y%m%dT%H%M%S").to_string(),
                event.timezone.as_deref(),
            );
            Ok(Json(json!({"format": "ics", "content": ics})))
        }
        "md" => {
            let md = export_event_md(
                &event.title,
                event.description.as_deref(),
                event.location.as_deref(),
                &event.start_at.to_rfc3339(),
                &event.end_at.to_rfc3339(),
                &[],
            );
            Ok(Json(json!({"format": "markdown", "content": md})))
        }
        _ => Ok(Json(serde_json::to_value(&event).unwrap_or_default())),
    }
}
```

### `backend/src/app/handlers/calls/handlers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calls/handlers.rs`
- Size bytes / Размер в байтах: `2663`
- Included characters / Включено символов: `2663`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use crate::app::api_support::{
    CallApiRequest, CallListResponse, CallTranscriptFixtureApiRequest, CallTranscriptResponse,
    TelegramListQuery, call_intelligence_store,
};
use crate::app::{ApiError, AppState};
use crate::platform::calls::{
    CallTranscript, FixtureSpeechToTextProvider, NewCallTranscript, ProviderCall,
    SpeechToTextProvider, TranscriptStatus,
};

pub(crate) async fn post_call(
    State(state): State<AppState>,
    Json(request): Json<CallApiRequest>,
) -> Result<Json<ProviderCall>, ApiError> {
    Ok(Json(
        call_intelligence_store(&state)?
            .upsert_call(&request.into_call())
            .await?,
    ))
}

pub(crate) async fn get_calls(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<CallListResponse>, ApiError> {
    let items = call_intelligence_store(&state)?
        .list_calls(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.provider.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(CallListResponse { items }))
}

pub(crate) async fn post_call_transcript_fixture(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
    Json(request): Json<CallTranscriptFixtureApiRequest>,
) -> Result<Json<CallTranscript>, ApiError> {
    let stt = FixtureSpeechToTextProvider;
    let fixture = stt.transcribe_fixture(&request.source_audio_ref)?;
    let transcript = NewCallTranscript {
        transcript_id: request.transcript_id,
        call_id,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        transcript_status: TranscriptStatus::Succeeded,
        stt_provider: stt.provider_name().to_owned(),
        source_audio_ref: Some(request.source_audio_ref),
        language_code: request.language_code,
        transcript_text: fixture.text,
        segments: fixture.segments,
        provenance: json!({
            "runtime": "fixture",
            "source": "local_call_audio",
            "always_on_policy": request.always_on_policy,
        }),
    };

    Ok(Json(
        call_intelligence_store(&state)?
            .upsert_transcript(&transcript)
            .await?,
    ))
}

pub(crate) async fn get_call_transcript(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<CallTranscriptResponse>, ApiError> {
    let transcript = call_intelligence_store(&state)?
        .transcript_for_call(&call_id)
        .await?;

    Ok(Json(CallTranscriptResponse { transcript }))
}
```

### `backend/src/app/handlers/calls/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calls/mod.rs`
- Size bytes / Размер в байтах: `43`
- Included characters / Включено символов: `43`
- Truncated / Обрезано: `no`

```rust
mod handlers;

pub(crate) use handlers::*;
```

### `backend/src/app/handlers/communications/account_management.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_management.rs`
- Size bytes / Размер в байтах: `9438`
- Included characters / Включено символов: `9438`
- Truncated / Обрезано: `no`

```rust
use super::*;
use crate::app::signal_hub_support::{
    remove_provider_account_signal_connection, sync_provider_account_signal_connection,
    sync_provider_account_signal_connection_with_status,
};

pub(crate) async fn get_v1_email_accounts(
    State(state): State<AppState>,
) -> Result<Json<EmailAccountListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let accounts = crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool)
        .list()
        .await?
        .into_iter()
        .filter(|account| account.provider_kind.is_email())
        .map(email_account_view)
        .collect();

    Ok(Json(EmailAccountListResponse { items: accounts }))
}

pub(crate) async fn get_v1_email_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountView>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    Ok(Json(email_account_view(account)))
}

pub(crate) async fn get_v1_email_account_export(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountExportResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let sanitized_account = ProviderAccount {
        config: sanitize_account_config(&account.config),
        ..account
    };

    Ok(Json(EmailAccountExportResponse {
        exported_at: Utc::now(),
        capabilities: email_account_capabilities(&sanitized_account),
        account: sanitized_account,
        sync_settings: settings,
    }))
}

pub(crate) async fn post_v1_email_account_import(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<EmailAccountLogoutResponse>, ApiError> {
    if contains_secret_material(&payload) {
        return Err(ApiError::InvalidCommunicationQuery(
            "account import payload must not contain secrets or secret references",
        ));
    }

    let request: EmailAccountImportRequest = serde_json::from_value(payload)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid account import payload"))?;
    let provider_kind = EmailProviderKind::try_from(request.account.provider_kind.as_str())
        .map_err(|_| ApiError::InvalidCommunicationQuery("unsupported email provider kind"))?;
    if !provider_kind.is_email() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider kind is not an email provider",
        ));
    }
    let config = if request.account.config.is_null() {
        json!({})
    } else {
        request.account.config
    };
    if !config.is_object() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account config must be an object",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account = crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool)
        .upsert(
            &crate::domains::communications::core::NewProviderAccount::new(
                request.account.account_id,
                provider_kind,
                request.account.display_name,
                request.account.external_account_id,
            )
            .config(config),
        )
        .await?;
    sync_provider_account_signal_connection(&state, &account, None).await?;

    let current_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let settings_update = request.sync_settings.map_or(
        MailSyncSettingsUpdate {
            sync_enabled: current_settings.sync_enabled,
            batch_size: current_settings.batch_size,
            poll_interval_seconds: current_settings.poll_interval_seconds,
        },
        |settings| MailSyncSettingsUpdate {
            sync_enabled: settings
                .sync_enabled
                .unwrap_or(current_settings.sync_enabled),
            batch_size: settings.batch_size.unwrap_or(current_settings.batch_size),
            poll_interval_seconds: settings
                .poll_interval_seconds
                .unwrap_or(current_settings.poll_interval_seconds),
        },
    );
    let sync_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .update_settings(&account.account_id, settings_update)
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(EmailAccountLogoutResponse {
        capabilities: email_account_capabilities(&account),
        account,
        sync_settings,
    }))
}

pub(crate) async fn post_v1_email_account_logout(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountLogoutResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated_account =
        crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool)
            .mark_logged_out(&account.account_id)
            .await?
            .ok_or(ApiError::NotFound)?;
    sync_provider_account_signal_connection_with_status(
        &state,
        &updated_account,
        "disconnected",
        None,
    )
    .await?;
    let current_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .settings_for_account(&account.account_id)
        .await
        .map_err(mail_sync_api_error)?;
    let sync_settings = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .update_settings(
            &account.account_id,
            MailSyncSettingsUpdate {
                sync_enabled: false,
                batch_size: current_settings.batch_size,
                poll_interval_seconds: current_settings.poll_interval_seconds,
            },
        )
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(EmailAccountLogoutResponse {
        capabilities: email_account_capabilities(&updated_account),
        account: updated_account,
        sync_settings,
    }))
}

pub(crate) async fn delete_v1_email_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<EmailAccountDeleteResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool);
    let usage = store.usage(&account.account_id).await?;
    if usage.has_retained_evidence() {
        return Err(ApiError::EmailAccountDeleteConflict);
    }

    let deleted = store.delete_metadata(&account.account_id).await?;
    remove_provider_account_signal_connection(&state, &account).await?;

    Ok(Json(EmailAccountDeleteResponse {
        account_id: account.account_id,
        deleted: deleted.account.is_some(),
        unbound_secret_refs: deleted.unbound_secret_refs,
    }))
}

pub(crate) async fn get_v1_email_account_sync_status(
    State(state): State<AppState>,
) -> Result<Json<MailSyncStatusListResponse>, ApiError> {
    let statuses = mail_sync_store(&state)
        .map_err(mail_sync_api_error)?
        .sync_statuses()
        .await
        .map_err(mail_sync_api_error)?;

    Ok(Json(MailSyncStatusListResponse { items: statuses }))
}

pub(crate) async fn get_v1_email_account_sync_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncSettings>, ApiError> {
    Ok(Json(
        mail_sync_store(&state)
            .map_err(mail_sync_api_error)?
            .settings_for_account(&account_id)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn put_v1_email_account_sync_settings(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<MailSyncSettingsUpdate>,
) -> Result<Json<MailSyncSettings>, ApiError> {
    Ok(Json(
        mail_sync_store(&state)
            .map_err(mail_sync_api_error)?
            .update_settings(&account_id, request)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn post_v1_email_account_sync_now(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncRunResponse>, ApiError> {
    Ok(Json(
        mail_sync_service(&state)
            .map_err(mail_sync_api_error)?
            .run_account(&account_id, MailSyncTrigger::Manual)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}

pub(crate) async fn post_v1_email_account_sync_full_resync(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailSyncRunResponse>, ApiError> {
    Ok(Json(
        mail_sync_service(&state)
            .map_err(mail_sync_api_error)?
            .run_account_full_resync(&account_id)
            .await
            .map_err(mail_sync_api_error)?,
    ))
}
```

### `backend/src/app/handlers/communications/account_setup.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_setup.rs`
- Size bytes / Размер в байтах: `271`
- Included characters / Включено символов: `271`
- Truncated / Обрезано: `no`

```rust
mod calendar;
mod gmail_callback;
mod gmail_oauth;
mod helpers;
mod imap;
mod models;

pub(crate) use gmail_callback::get_gmail_oauth_callback;
pub(crate) use gmail_oauth::{post_gmail_oauth_complete, post_gmail_oauth_start};
pub(crate) use imap::post_imap_account_setup;
```

### `backend/src/app/handlers/communications/account_setup/calendar.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_setup/calendar.rs`
- Size bytes / Размер в байтах: `1271`
- Included characters / Включено символов: `1271`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(super) async fn upsert_google_workspace_calendar_account(
    state: &AppState,
    mail_account_id: &str,
    display_name: &str,
    external_account_id: &str,
    secret_ref: &str,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<crate::domains::calendar::events::CalendarAccountStore>(
        pool,
    )
    .upsert_google_workspace_account(
        mail_account_id,
        display_name,
        Some(external_account_id),
        secret_ref,
    )
    .await?;
    Ok(())
}

pub(super) async fn upsert_apple_icloud_calendar_account(
    state: &AppState,
    mail_account_id: &str,
    display_name: &str,
    external_account_id: &str,
    secret_ref: &str,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<crate::domains::calendar::events::CalendarAccountStore>(
        pool,
    )
    .upsert_apple_icloud_account(
        mail_account_id,
        display_name,
        Some(external_account_id),
        secret_ref,
    )
    .await?;
    Ok(())
}
```

### `backend/src/app/handlers/communications/account_setup/gmail_callback.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_setup/gmail_callback.rs`
- Size bytes / Размер в байтах: `10118`
- Included characters / Включено символов: `10118`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::calendar::upsert_google_workspace_calendar_account;
use super::helpers::{gmail_pending_external_account_id, trimmed_optional};
use super::models::GmailOAuthCallbackQuery;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};

pub(crate) async fn get_gmail_oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<GmailOAuthCallbackQuery>,
) -> (StatusCode, Html<String>) {
    let GmailOAuthCallbackQuery {
        code,
        state: oauth_state,
        error,
        error_description: _,
    } = query;
    if trimmed_optional(error).is_some() {
        return gmail_oauth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Google authorization failed. Start the mail connection again.",
        );
    }
    let Some(code) = trimmed_optional(code) else {
        return gmail_oauth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Missing authorization code. Start the mail connection again.",
        );
    };
    let Some(oauth_state) = trimmed_optional(oauth_state) else {
        return gmail_oauth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Missing OAuth state. Start the mail connection again.",
        );
    };

    let pending = match remove_pending_gmail_oauth_by_state(&state, &oauth_state) {
        Ok(Some(pending)) => pending,
        Ok(None) => {
            return gmail_oauth_callback_error_page(
                StatusCode::BAD_REQUEST,
                "OAuth grant expired or was already used. Start the mail connection again.",
            );
        }
        Err(_error) => {
            tracing::error!("Gmail OAuth callback state lookup failed");
            return gmail_oauth_callback_error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Account setup state is unavailable. Start the mail connection again.",
            );
        }
    };

    let service = match account_setup_service(&state) {
        Ok(service) => service,
        Err(_error) => {
            tracing::error!("Gmail OAuth callback setup service failed");
            return gmail_oauth_callback_error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Account setup is unavailable. Check local backend and vault status.",
            );
        }
    };
    let app_return_url = pending.request.app_return_url.clone();
    let mail_account_id = pending.account_id.clone();
    let display_name = pending.request.display_name.clone();
    let external_account_id = gmail_pending_external_account_id(&pending);
    match service.complete_gmail_oauth(pending, &code).await {
        Ok(result) => {
            let account = match provider_account_or_not_found(&state, &result.account_id).await {
                Ok(account) => account,
                Err(error) => {
                    tracing::error!("Gmail OAuth callback account lookup failed");
                    return gmail_oauth_callback_error_page(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Signal Hub sync failed. Check local backend status.",
                    );
                }
            };
            if let Err(error) =
                sync_provider_account_signal_connection(&state, &account, Some(&result.secret_ref))
                    .await
            {
                tracing::error!("Gmail OAuth callback signal hub sync failed");
                return gmail_oauth_callback_error_page(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Signal Hub sync failed. Check local backend status.",
                );
            }
            if let Err(error) = upsert_google_workspace_calendar_account(
                &state,
                &mail_account_id,
                &display_name,
                &external_account_id,
                &result.secret_ref,
            )
            .await
            {
                tracing::error!("Gmail OAuth callback calendar account setup failed");
                return gmail_oauth_callback_error_page(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    gmail_oauth_callback_api_error_message(&error),
                );
            }
            gmail_oauth_callback_success_page(&result.account_id, app_return_url.as_deref())
        }
        Err(error) => {
            let status = if matches!(
                error,
                EmailAccountSetupError::InvalidRequest { .. }
                    | EmailAccountSetupError::MissingProviderField { .. }
            ) {
                StatusCode::BAD_REQUEST
            } else {
                tracing::error!(error = %error, "Gmail OAuth callback completion failed");
                StatusCode::INTERNAL_SERVER_ERROR
            };
            gmail_oauth_callback_error_page(status, gmail_oauth_callback_error_message(&error))
        }
    }
}

fn remove_pending_gmail_oauth_by_state(
    state: &AppState,
    oauth_state: &str,
) -> Result<Option<GmailOAuthPendingGrant>, ApiError> {
    let mut pending_map = state
        .account_setup
        .pending_gmail_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    let setup_id = pending_map
        .iter()
        .find_map(|(setup_id, pending)| (pending.state == oauth_state).then(|| setup_id.clone()));
    Ok(setup_id.and_then(|setup_id| pending_map.remove(&setup_id)))
}

fn gmail_oauth_callback_success_page(
    account_id: &str,
    app_return_url: Option<&str>,
) -> (StatusCode, Html<String>) {
    let account_id = html_escape(account_id);
    let return_url_json = app_return_url
        .map(|url| serde_json::to_string(url).expect("serialize OAuth return URL"))
        .unwrap_or_else(|| "null".to_owned());
    let return_link = app_return_url
        .map(|url| {
            format!(
                r#"<p><a href="{}">Return to Макошь settings</a></p>"#,
                html_escape(url)
            )
        })
        .unwrap_or_default();
    (
        StatusCode::OK,
        Html(format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Макошь OAuth</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    p {{ line-height: 1.5; }}
    a {{ color: #0f766e; font-weight: 700; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
  <script>
    window.setTimeout(function () {{
      try {{
        if (window.opener && !window.opener.closed) {{
          window.opener.postMessage({{ type: 'makosh:gmail-oauth-connected' }}, '*');
        }}
      }} catch (_error) {{}}
      try {{
        window.close();
      }} catch (_error) {{}}
    }}, 250);
    window.setTimeout(function () {{
      var returnUrl = {return_url_json};
      if (returnUrl) {{
        window.location.replace(returnUrl);
      }}
    }}, 1400);
  </script>
</head>
<body>
  <main>
    <h1>Google mail connected</h1>
    <p>Макошь saved the Google mail account and encrypted OAuth credential locally.</p>
    <p>Account</p>
    <code>{account_id}</code>
    <p>This tab will close automatically. If it stays open, return to Макошь settings.</p>
    {return_link}
  </main>
</body>
</html>"#
        )),
    )
}

fn gmail_oauth_callback_error_page(
    status: StatusCode,
    message: &str,
) -> (StatusCode, Html<String>) {
    let message = html_escape(message);
    (
        status,
        Html(format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Макошь OAuth</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
</head>
<body>
  <main>
    <h1>Google mail connection failed</h1>
    <p>{message}</p>
    <p>Return to Макошь and start Google mail connection again.</p>
  </main>
</body>
</html>"#
        )),
    )
}

fn gmail_oauth_callback_error_message(error: &EmailAccountSetupError) -> &'static str {
    match error {
        EmailAccountSetupError::HostVault(HostVaultError::Locked) => {
            "Макошь Secure Vault is locked. Unlock the vault in Макошь, then start Google mail connection again."
        }
        EmailAccountSetupError::HostVault(HostVaultError::Uninitialized) => {
            "Макошь Secure Vault is not initialized. Create the vault in Макошь, then start Google mail connection again."
        }
        EmailAccountSetupError::InvalidRequest { field, .. } if *field == "authorization_code" => {
            "Missing authorization code. Start the mail connection again."
        }
        EmailAccountSetupError::MissingProviderField { field } if *field == "refresh_token" => {
            "Google did not return a refresh token. Start the connection again and approve offline access."
        }
        EmailAccountSetupError::InvalidRequest { .. }
        | EmailAccountSetupError::MissingProviderField { .. } => {
            "Google mail authorization response was incomplete. Start the connection again."
        }
        _ => "Google mail account setup failed. Check local backend and vault status.",
    }
}

fn gmail_oauth_callback_api_error_message(error: &ApiError) -> &'static str {
    match error {
        ApiError::DatabaseNotConfigured => {
            "Google mail connected, but calendar account setup could not write to the local database."
        }
        _ => {
            "Google mail connected, but linked calendar account setup failed. Check local backend status."
        }
    }
}
```

### `backend/src/app/handlers/communications/account_setup/gmail_oauth.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_setup/gmail_oauth.rs`
- Size bytes / Размер в байтах: `2845`
- Included characters / Включено символов: `2845`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::calendar::upsert_google_workspace_calendar_account;
use super::helpers::{gmail_pending_external_account_id, trimmed_optional};
use super::models::{
    EmailAccountSetupApiResponse, GmailOAuthCompleteApiRequest, GmailOAuthStartApiRequest,
    GmailOAuthStartApiResponse,
};
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};

pub(crate) async fn post_gmail_oauth_start(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthStartApiRequest>,
) -> Result<Json<GmailOAuthStartApiResponse>, ApiError> {
    require_unlocked_host_vault(&state)?;
    let service = account_setup_service(&state)?;
    let pending = service.start_gmail_oauth(request.into_setup_request(&state.config)?)?;
    let response = GmailOAuthStartApiResponse {
        setup_id: pending.setup_id.clone(),
        authorization_url: pending.authorization_url.clone(),
        state: pending.state.clone(),
        redirect_uri: pending.request.redirect_uri.clone(),
    };
    let mut pending_map = state
        .account_setup
        .pending_gmail_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    pending_map.insert(pending.setup_id.clone(), pending);

    Ok(Json(response))
}

pub(crate) async fn post_gmail_oauth_complete(
    State(state): State<AppState>,
    Json(request): Json<GmailOAuthCompleteApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let mut pending = {
        let mut pending_map = state
            .account_setup
            .pending_gmail_oauth
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending_map
            .remove(&request.setup_id)
            .ok_or(ApiError::AccountSetupPendingGrantNotFound)?
    };
    if pending.state != request.state {
        return Err(ApiError::AccountSetupStateMismatch);
    }
    if let Some(external_account_id) = trimmed_optional(request.external_account_id) {
        pending.request = pending.request.external_account_id(external_account_id);
    }
    let mail_account_id = pending.account_id.clone();
    let display_name = pending.request.display_name.clone();
    let external_account_id = gmail_pending_external_account_id(&pending);

    let service = account_setup_service(&state)?;
    let result = service
        .complete_gmail_oauth(pending, &request.authorization_code)
        .await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.secret_ref)).await?;
    upsert_google_workspace_calendar_account(
        &state,
        &mail_account_id,
        &display_name,
        &external_account_id,
        &result.secret_ref,
    )
    .await?;

    Ok(Json(result.into()))
}
```

### `backend/src/app/handlers/communications/account_setup/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_setup/helpers.rs`
- Size bytes / Размер в байтах: `421`
- Included characters / Включено символов: `421`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(super) fn gmail_pending_external_account_id(pending: &GmailOAuthPendingGrant) -> String {
    trimmed_optional(Some(pending.request.external_account_id.clone()))
        .unwrap_or_else(|| pending.account_id.clone())
}

pub(super) fn trimmed_optional(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_owned())
        .filter(|trimmed| !trimmed.is_empty())
}
```

### `backend/src/app/handlers/communications/account_setup/imap.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_setup/imap.rs`
- Size bytes / Размер в байтах: `1527`
- Included characters / Включено символов: `1527`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::calendar::upsert_apple_icloud_calendar_account;
use super::models::{EmailAccountSetupApiResponse, ImapAccountSetupApiRequest};
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};

pub(crate) async fn post_imap_account_setup(
    State(state): State<AppState>,
    Json(request): Json<ImapAccountSetupApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let setup_request = request.into_setup_request()?;
    let service = account_setup_service(&state)?;
    require_unlocked_host_vault(&state)?;
    let icloud_calendar_account =
        (setup_request.provider_kind == EmailProviderKind::Icloud).then(|| {
            (
                setup_request.account_id.clone(),
                setup_request.display_name.clone(),
                setup_request.external_account_id.clone(),
            )
        });
    let result = service.setup_imap_account(setup_request).await?;
    let account = provider_account_or_not_found(&state, &result.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&result.secret_ref)).await?;
    if let Some((mail_account_id, display_name, external_account_id)) = icloud_calendar_account {
        upsert_apple_icloud_calendar_account(
            &state,
            &mail_account_id,
            &display_name,
            &external_account_id,
            &result.secret_ref,
        )
        .await?;
    }

    Ok(Json(result.into()))
}
```

### `backend/src/app/handlers/communications/account_setup/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_setup/models.rs`
- Size bytes / Размер в байтах: `6875`
- Included characters / Включено символов: `6875`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use super::helpers::trimmed_optional;

#[derive(Deserialize)]
pub(crate) struct GmailOAuthStartApiRequest {
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) external_account_id: Option<String>,
    pub(super) client_id: Option<String>,
    pub(super) client_secret: Option<String>,
    pub(super) redirect_uri: String,
    pub(super) app_return_url: Option<String>,
    pub(super) scopes: Option<Vec<String>>,
    pub(super) authorization_endpoint: Option<String>,
    pub(super) token_endpoint: Option<String>,
}

impl GmailOAuthStartApiRequest {
    pub(super) fn into_setup_request(
        self,
        config: &crate::platform::config::AppConfig,
    ) -> Result<GmailOAuthSetupRequest, EmailAccountSetupError> {
        let client_id = trimmed_optional(self.client_id)
            .or_else(|| config.google_oauth_client_id().map(str::to_owned))
            .ok_or(EmailAccountSetupError::InvalidRequest {
                field: "client_id",
                message: "must be configured as request client_id or MAKOSH_GOOGLE_OAUTH_CLIENT_ID",
            })?;
        let mut request = GmailOAuthSetupRequest::new(
            self.account_id,
            self.display_name,
            trimmed_optional(self.external_account_id).unwrap_or_default(),
            client_id,
            self.redirect_uri,
        );
        if let Some(app_return_url) = trimmed_optional(self.app_return_url) {
            request = request.app_return_url(app_return_url);
        }
        if let Some(client) = config.google_oauth_client() {
            request = request
                .authorization_endpoint(client.authorization_endpoint().to_owned())
                .token_endpoint(client.token_endpoint().to_owned());
        }
        if let Some(client_secret) = trimmed_optional(self.client_secret).or_else(|| {
            config
                .google_oauth_client_secret()
                .map(|secret| secret.expose_for_runtime().to_owned())
        }) {
            request = request.client_secret(client_secret);
        }
        if let Some(scopes) = self.scopes {
            request = request.scopes(scopes);
        }
        if let Some(authorization_endpoint) = self.authorization_endpoint {
            request = request.authorization_endpoint(authorization_endpoint);
        }
        if let Some(token_endpoint) = self.token_endpoint {
            request = request.token_endpoint(token_endpoint);
        }

        Ok(request)
    }
}

#[derive(Serialize)]
pub(crate) struct GmailOAuthStartApiResponse {
    pub(super) setup_id: String,
    pub(super) authorization_url: String,
    pub(super) state: String,
    pub(super) redirect_uri: String,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCompleteApiRequest {
    pub(super) setup_id: String,
    pub(super) state: String,
    pub(super) authorization_code: String,
    pub(super) external_account_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCallbackQuery {
    pub(super) code: Option<String>,
    pub(super) state: Option<String>,
    pub(super) error: Option<String>,
    pub(super) error_description: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ImapAccountSetupApiRequest {
    pub(super) account_id: String,
    pub(super) provider_kind: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) mailbox: String,
    pub(super) username: String,
    pub(super) password: String,
    pub(super) secret_kind: Option<String>,
    pub(super) smtp_host: Option<String>,
    pub(super) smtp_port: Option<u16>,
    pub(super) smtp_tls: Option<bool>,
    pub(super) smtp_starttls: Option<bool>,
    pub(super) smtp_username: Option<String>,
}

impl ImapAccountSetupApiRequest {
    pub(super) fn into_setup_request(self) -> Result<ImapAccountSetupRequest, ApiError> {
        let Self {
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            host,
            port,
            tls,
            mailbox,
            username,
            password,
            secret_kind,
            smtp_host,
            smtp_port,
            smtp_tls,
            smtp_starttls,
            smtp_username,
        } = self;
        let provider_kind = match provider_kind.trim() {
            "icloud" => EmailProviderKind::Icloud,
            "imap" => EmailProviderKind::Imap,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "provider_kind",
                    message: "must be icloud or imap",
                }
                .into());
            }
        };
        let secret_kind = match secret_kind.as_deref().unwrap_or("password").trim() {
            "app_password" => SecretKind::AppPassword,
            "password" => SecretKind::Password,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "secret_kind",
                    message: "must be app_password or password",
                }
                .into());
            }
        };

        let mut request = ImapAccountSetupRequest::new(
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            host,
            port,
            tls,
            mailbox,
            username,
            password,
        )
        .secret_kind(secret_kind);
        if let Some(smtp_host) = trimmed_optional(smtp_host) {
            request = request.smtp_host(smtp_host);
        }
        if let Some(smtp_port) = smtp_port {
            request = request.smtp_port(smtp_port);
        }
        if let Some(smtp_tls) = smtp_tls {
            request = request.smtp_tls(smtp_tls);
        }
        if let Some(smtp_starttls) = smtp_starttls {
            request = request.smtp_starttls(smtp_starttls);
        }
        if let Some(smtp_username) = trimmed_optional(smtp_username) {
            request = request.smtp_username(smtp_username);
        }

        Ok(request)
    }
}

#[derive(Serialize)]
pub(crate) struct EmailAccountSetupApiResponse {
    pub(super) account_id: String,
    pub(super) secret_ref: String,
    pub(super) secret_kind: SecretKind,
    pub(super) store_kind: crate::platform::secrets::SecretStoreKind,
}

impl From<crate::integrations::mail::accounts::EmailAccountSetupResult>
    for EmailAccountSetupApiResponse
{
    fn from(result: crate::integrations::mail::accounts::EmailAccountSetupResult) -> Self {
        Self {
            account_id: result.account_id,
            secret_ref: result.secret_ref,
            secret_kind: result.secret_kind,
            store_kind: result.store_kind,
        }
    }
}
```

### `backend/src/app/handlers/communications/account_support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/account_support.rs`
- Size bytes / Размер в байтах: `8838`
- Included characters / Включено символов: `8838`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Serialize)]
pub(crate) struct MailSyncStatusListResponse {
    pub(super) items: Vec<MailSyncStatus>,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountListResponse {
    pub(super) items: Vec<EmailAccountView>,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountView {
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountCapabilities {
    pub(super) read: bool,
    pub(super) sync: bool,
    pub(super) send: bool,
    pub(super) oauth: bool,
    pub(super) imap: bool,
    pub(super) smtp: bool,
    pub(super) mutate_flags: bool,
    pub(super) mutate_mailboxes: bool,
    pub(super) server_delete: bool,
    pub(super) provider_folders: bool,
    pub(super) local_trash: bool,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountExportResponse {
    pub(super) exported_at: DateTime<Utc>,
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
    pub(super) sync_settings: MailSyncSettings,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountLogoutResponse {
    pub(super) account: ProviderAccount,
    pub(super) capabilities: EmailAccountCapabilities,
    pub(super) sync_settings: MailSyncSettings,
}

#[derive(Serialize)]
pub(crate) struct EmailAccountDeleteResponse {
    pub(super) account_id: String,
    pub(super) deleted: bool,
    pub(super) unbound_secret_refs: Vec<String>,
}

#[derive(Deserialize)]
pub(super) struct EmailAccountImportRequest {
    pub(super) account: EmailAccountImportAccount,
    pub(super) sync_settings: Option<EmailAccountImportSyncSettings>,
}

#[derive(Deserialize)]
pub(super) struct EmailAccountImportAccount {
    pub(super) account_id: String,
    pub(super) provider_kind: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    #[serde(default)]
    pub(super) config: Value,
}

#[derive(Deserialize)]
pub(super) struct EmailAccountImportSyncSettings {
    pub(super) sync_enabled: Option<bool>,
    pub(super) batch_size: Option<i32>,
    pub(super) poll_interval_seconds: Option<i32>,
}
pub(super) async fn email_account_or_not_found(
    state: &AppState,
    account_id: &str,
) -> Result<ProviderAccount, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let Some(account) =
        crate::app::api_support::app_store::<CommunicationProviderAccountStore>(pool)
            .get(account_id)
            .await?
    else {
        return Err(ApiError::NotFound);
    };
    if !account.provider_kind.is_email() {
        return Err(ApiError::NotFound);
    }

    Ok(account)
}

pub(super) fn email_account_view(account: ProviderAccount) -> EmailAccountView {
    EmailAccountView {
        capabilities: email_account_capabilities(&account),
        account,
    }
}

pub(super) fn email_account_capabilities(account: &ProviderAccount) -> EmailAccountCapabilities {
    let logged_out = account
        .config
        .get("auth_state")
        .and_then(Value::as_str)
        .is_some_and(|state| state == "logged_out");
    let smtp = smtp_configured(&account.config);
    let imap = matches!(
        account.provider_kind,
        EmailProviderKind::Icloud | EmailProviderKind::Imap
    );
    let oauth = matches!(account.provider_kind, EmailProviderKind::Gmail)
        || account
            .config
            .get("auth")
            .and_then(Value::as_str)
            .is_some_and(|auth| auth == "oauth");
    let gmail_send_enabled = account
        .config
        .get("gmail_send_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    EmailAccountCapabilities {
        read: !logged_out,
        sync: !logged_out,
        send: !logged_out && (smtp || gmail_send_enabled),
        oauth,
        imap,
        smtp,
        mutate_flags: !logged_out && imap,
        mutate_mailboxes: false,
        server_delete: false,
        provider_folders: false,
        local_trash: true,
    }
}

pub(super) fn smtp_configured(config: &Value) -> bool {
    let Some(object) = config.as_object() else {
        return false;
    };
    object
        .get("smtp_host")
        .and_then(Value::as_str)
        .is_some_and(|host| !host.trim().is_empty())
        && object.get("smtp_port").and_then(Value::as_i64).is_some()
}

pub(super) fn sanitize_account_config(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter_map(|(key, value)| {
                    if is_secret_config_key(key) {
                        None
                    } else {
                        Some((key.clone(), sanitize_account_config(value)))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(sanitize_account_config).collect()),
        other => other.clone(),
    }
}

pub(super) fn contains_secret_material(value: &Value) -> bool {
    match value {
        Value::Object(object) => object
            .iter()
            .any(|(key, value)| is_secret_config_key(key) || contains_secret_material(value)),
        Value::Array(items) => items.iter().any(contains_secret_material),
        _ => false,
    }
}

pub(super) fn is_secret_config_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "password",
        "secret",
        "secret_ref",
        "token",
        "credential",
        "api_key",
        "private_key",
        "client_secret",
        "refresh_token",
        "access_token",
    ]
    .iter()
    .any(|marker| key.contains(marker))
}

pub(super) fn require_unlocked_host_vault(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(ApiError::HostVault(HostVaultError::Locked)),
        VaultMode::Uninitialized => Err(ApiError::HostVault(HostVaultError::Uninitialized)),
    }
}

pub(super) fn mail_sync_store(state: &AppState) -> Result<MailSyncStore, MailSyncError> {
    let Some(pool) = state.database.pool() else {
        return Err(MailSyncError::InvalidSetting {
            field: "database",
            message: "DATABASE_URL is not configured",
        });
    };

    Ok(crate::app::api_support::app_store::<MailSyncStore>(
        pool.clone(),
    ))
}

pub(super) fn mail_sync_service(
    state: &AppState,
) -> Result<MailBackgroundSyncService, MailSyncError> {
    let Some(pool) = state.database.pool() else {
        return Err(MailSyncError::InvalidSetting {
            field: "database",
            message: "DATABASE_URL is not configured",
        });
    };

    Ok(MailBackgroundSyncService::new(
        pool.clone(),
        state.vault.clone(),
        DEFAULT_MAIL_SYNC_BLOB_ROOT,
        std::sync::Arc::new(
            crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort::new(
                pool.clone(),
                state.vault.clone(),
                std::sync::Arc::new(crate::app::api_support::app_store::<
                    crate::domains::communications::core::CommunicationProviderSecretBindingStore,
                >(pool.clone())),
                crate::application::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
            ),
        ),
    ))
}

pub(super) fn mail_sync_api_error(error: MailSyncError) -> ApiError {
    match error {
        MailSyncError::AccountNotFound => ApiError::NotFound,
        MailSyncError::RunAlreadyActive | MailSyncError::RunNotFound => {
            ApiError::InvalidCommunicationQuery("mail sync run is already active")
        }
        MailSyncError::InvalidSetting {
            field: "database", ..
        } => ApiError::DatabaseNotConfigured,
        MailSyncError::InvalidSetting { .. } => {
            ApiError::InvalidCommunicationQuery("invalid mail sync settings")
        }
        MailSyncError::Sqlx(error) => {
            tracing::error!(error = %error, "mail sync database operation failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
        MailSyncError::Communication(error) => {
            tracing::error!(error = %error, "mail sync communication store failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
        MailSyncError::EventEnvelope(error) => ApiError::InvalidEnvelope(error),
        MailSyncError::EventLogPort(error) => ApiError::Store(error),
        MailSyncError::ObservationPort(error) => {
            tracing::error!(error = %error, "mail sync observation store failed");
            ApiError::InvalidCommunicationQuery("mail sync operation failed")
        }
    }
}
```

### `backend/src/app/handlers/communications/communication_messages.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_messages.rs`
- Size bytes / Размер в байтах: `5268`
- Included characters / Включено символов: `5268`
- Truncated / Обрезано: `no`

```rust
use super::*;
use crate::domains::communications::messages::ProjectedMessagePageQuery;

pub(crate) async fn get_v1_communication_messages(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<CommunicationMessagesResponse>, ApiError> {
    let query = parse_communication_messages_query(raw_query.as_deref())?;
    let limit = query.limit.unwrap_or(5000).clamp(1, 5000);
    let workflow_state = query
        .workflow_state
        .as_deref()
        .map(str::parse::<WorkflowState>)
        .transpose()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid workflow state value"))?;
    let local_state = query
        .local_state
        .as_deref()
        .unwrap_or("active")
        .parse::<LocalMessageState>()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid local_state value"))?;
    let page = message_store(&state)?
        .list_messages_page(ProjectedMessagePageQuery {
            account_id: query.account_id.as_deref(),
            workflow_state,
            channel_kind: query.channel_kind.as_deref(),
            conversation_id: query.conversation_id.as_deref(),
            query: query.q.as_deref(),
            match_mode: query.match_mode,
            search: query.search.clone(),
            local_state,
            cursor: query.cursor.as_deref(),
            limit,
        })
        .await?;
    let items = page
        .items
        .into_iter()
        .map(CommunicationMessageSummaryResponse::from)
        .collect();

    Ok(Json(CommunicationMessagesResponse {
        items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn get_v1_communication_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<CommunicationMessageDetailResponse>, ApiError> {
    let Some(message) = message_store(&state)?.message(&message_id).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let rich_detail = rich_email_message_detail_for_message(&state, &message).await?;
    let message_metadata = message_metadata_with_raw_headers(
        &message.message_metadata,
        rich_detail.headers.as_slice(),
    );
    let attachments = communication_storage_store(&state)?
        .attachments_for_message(&message.message_id)
        .await?
        .into_iter()
        .map(CommunicationAttachmentResponse::from)
        .collect();

    Ok(Json(CommunicationMessageDetailResponse {
        message: CommunicationMessageDetailItem::from_message_with_metadata(
            message,
            rich_detail.body_html,
            message_metadata,
        ),
        attachments,
    }))
}

pub(super) async fn rich_body_html_for_message(
    state: &AppState,
    message: &ProjectedMessage,
) -> Result<Option<String>, ApiError> {
    Ok(rich_email_message_detail_for_message(state, message)
        .await?
        .body_html)
}

#[derive(Default)]
pub(super) struct RichCommunicationMessageDetail {
    pub(super) body_html: Option<String>,
    pub(super) headers: Vec<(String, String)>,
}

async fn rich_email_message_detail_for_message(
    state: &AppState,
    message: &ProjectedMessage,
) -> Result<RichCommunicationMessageDetail, ApiError> {
    let Some(raw) = communication_ingestion_store(state)?
        .raw_record(&message.raw_record_id)
        .await?
    else {
        return Ok(RichCommunicationMessageDetail::default());
    };
    if raw.record_kind != "email_message" {
        return Ok(RichCommunicationMessageDetail::default());
    }
    if raw
        .payload
        .get("raw_blob_storage_kind")
        .and_then(Value::as_str)
        != Some("local_fs")
    {
        return Ok(RichCommunicationMessageDetail::default());
    }
    if raw
        .payload
        .get("raw_blob_storage_path")
        .and_then(Value::as_str)
        .is_none()
    {
        return Ok(RichCommunicationMessageDetail::default());
    }

    let blob_store = crate::app::api_support::communication_blob_store();
    match parse_raw_email_message_from_blob(&blob_store, &raw).await {
        Ok(parsed) => Ok(RichCommunicationMessageDetail {
            body_html: parsed.body_html.filter(|value| !value.trim().is_empty()),
            headers: parsed.headers,
        }),
        Err(error) => {
            tracing::warn!(
                error = %error,
                message_id = %message.message_id,
                raw_record_id = %message.raw_record_id,
                "mail detail rich html extraction failed; falling back to projected body_text"
            );
            Ok(RichCommunicationMessageDetail::default())
        }
    }
}

fn message_metadata_with_raw_headers(
    message_metadata: &Value,
    headers: &[(String, String)],
) -> Value {
    let mut metadata = message_metadata.as_object().cloned().unwrap_or_default();
    if !headers.is_empty() && !metadata.contains_key("headers") {
        metadata.insert(
            "headers".to_owned(),
            Value::Array(
                headers
                    .iter()
                    .map(|(name, value)| json!({ "name": name, "value": value }))
                    .collect(),
            ),
        );
    }
    Value::Object(metadata)
}
```

### `backend/src/app/handlers/communications/communication_queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries.rs`
- Size bytes / Размер в байтах: `431`
- Included characters / Включено символов: `431`
- Truncated / Обрезано: `no`

```rust
mod attachments;
mod drafts;
mod folders;
mod imports;
mod outbox;
mod personas;
mod read_receipts;
mod saved_searches;
mod search;
mod threads;

pub(crate) use attachments::*;
pub(crate) use drafts::*;
pub(crate) use folders::*;
pub(crate) use imports::*;
pub(crate) use outbox::*;
pub(crate) use personas::*;
pub(crate) use read_receipts::*;
pub(crate) use saved_searches::*;
pub(crate) use search::*;
pub(crate) use threads::*;
```

### `backend/src/app/handlers/communications/communication_queries/attachments.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/attachments.rs`
- Size bytes / Размер в байтах: `18539`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::super::*;
use crate::domains::communications::archive_inspection::{
    ArchiveInspectionLimits, ArchiveInspectionReport, inspect_zip_bytes,
};
use crate::domains::communications::attachment_search::{
    AttachmentSearchQuery, AttachmentSearchStore,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

const MAX_TEXT_PREVIEW_BYTES: usize = 64 * 1024;
const MAX_IMAGE_PREVIEW_BYTES: usize = 5 * 1024 * 1024;
const MAX_AUDIO_PREVIEW_BYTES: usize = 24 * 1024 * 1024;
const MAX_VIDEO_PREVIEW_BYTES: usize = 32 * 1024 * 1024;
const MAX_PDF_PREVIEW_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize)]
pub(crate) struct AttachmentSearchRequest {
    pub(crate) account_id: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) scan_status: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) async fn get_v1_attachment_search(
    State(state): State<AppState>,
    Query(query): Query<AttachmentSearchRequest>,
) -> Result<Json<crate::domains::communications::attachment_search::AttachmentSearchPage>, ApiError>
{
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let store = crate::app::api_support::app_store::<AttachmentSearchStore>(pool);
    Ok(Json(
        store
            .search(AttachmentSearchQuery {
                account_id: query.account_id.as_deref(),
                query: query.q.as_deref(),
                content_type: query.content_type.as_deref(),
                scan_status: query.scan_status.as_deref(),
                cursor: query.cursor.as_deref(),
                limit: query.limit.unwrap_or(100),
            })
            .await?,
    ))
}

#[derive(Serialize)]
pub(crate) struct AttachmentArchiveInspectionResponse {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) scan_status: String,
    pub(crate) report: ArchiveInspectionReport,
}

#[derive(Serialize)]
pub(crate) struct AttachmentPreviewResponse {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) scan_status: String,
    pub(crate) preview_kind: &'static str,
    pub(crate) text: String,
    pub(crate) data_url: Option<String>,
    pub(crate) truncated: bool,
    pub(crate) byte_count: usize,
    pub(crate) max_preview_bytes: usize,
}

pub(crate) async fn get_v1_attachment_preview(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    let attachment = communication_storage_store(&state)?
        .attachment_by_id(&attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if attachment.storage_kind != "local_fs" {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment preview requires a local blob",
        ));
    }
    if !is_preview_allowed_by_scan_status(&attachment) {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment preview is blocked by attachment scan status",
        ));
    }
    let preview_kind =
        attachment_preview_kind(&attachment).ok_or(ApiError::InvalidCommunicationQuery(
            "attachment preview supports text, image, audio, video and pdf attachments only",
        ))?;

    let bytes = crate::app::api_support::communication_blob_store()
        .read_blob(&attachment.storage_path)
        .await?;
    let byte_count = bytes.len();

    match preview_kind {
        AttachmentPreviewKind::Text => text_attachment_preview(attachment, bytes, byte_count),
        AttachmentPreviewKind::Image => image_attachment_preview(attachment, bytes, byte_count),
        AttachmentPreviewKind::Audio => audio_attachment_preview(attachment, bytes, byte_count),
        AttachmentPreviewKind::Video => video_attachment_preview(attachment, bytes, byte_count),
        AttachmentPreviewKind::Pdf => pdf_attachment_preview(attachment, bytes, byte_count),
    }
}

fn text_attachment_preview(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    let truncated = byte_count > MAX_TEXT_PREVIEW_BYTES;
    let preview_bytes = if truncated {
        &bytes[..MAX_TEXT_PREVIEW_BYTES]
    } else {
        &bytes
    };
    let text = String::from_utf8_lossy(preview_bytes).into_owned();

    Ok(Json(AttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "text",
        text,
        data_url: None,
        truncated,
        byte_count,
        max_preview_bytes: MAX_TEXT_PREVIEW_BYTES,
    }))
}

fn image_attachment_preview(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    if byte_count > MAX_IMAGE_PREVIEW_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment image preview exceeds size limit",
        ));
    }
    let content_type = preview_image_content_type(&attachment).unwrap_or("image/png");
    let data_url = format!(
        "data:{content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    );

    Ok(Json(AttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "image",
        text: String::new(),
        data_url: Some(data_url),
        truncated: false,
        byte_count,
        max_preview_bytes: MAX_IMAGE_PREVIEW_BYTES,
    }))
}

fn audio_attachment_preview(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    if byte_count > MAX_AUDIO_PREVIEW_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment audio preview exceeds size limit",
        ));
    }
    let content_type = preview_audio_content_type(&attachment).unwrap_or("audio/mpeg");
    let data_url = format!(
        "data:{content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    );

    Ok(Json(AttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "audio",
        text: String::new(),
        data_url: Some(data_url),
        truncated: false,
        byte_count,
        max_preview_bytes: MAX_AUDIO_PREVIEW_BYTES,
    }))
}

fn video_attachment_preview(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    if byte_count > MAX_VIDEO_PREVIEW_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment video preview exceeds size limit",
        ));
    }
    let content_type = preview_video_content_type(&attachment).unwrap_or("video/mp4");
    let data_url = format!(
        "data:{content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    );

    Ok(Json(AttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "video",
        text: String::new(),
        data_url: Some(data_url),
        truncated: false,
        byte_count,
        max_preview_bytes: MAX_VIDEO_PREVIEW_BYTES,
    }))
}

fn pdf_attachment_preview(
    attachment: StoredCommunicationAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    if byte_count > MAX_PDF_PREVIEW_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment pdf preview exceeds size limit",
        ));
    }
    let content_type = preview_pdf_content_type(&attachment).unwrap_or("application/pdf");
    let data_url = format!(
        "data:{content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    );

    Ok(Json(AttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "pdf",
        text: String::new(),
        data_url: Some(data_url),
        truncated: false,
        byte_count,
        max_preview_bytes: MAX_PDF_PREVIEW_BYTES,
    }))
}

pub(crate) async fn get_v1_attachment_archive_inspection(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
) -> Result<Json<AttachmentArchiveInspectionResponse>, ApiError> {
    let attachment = communication_storage_store(&state)?
        .attachment_by_id(&attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if attachment.storage_kind != "local_fs" {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment archive inspection requires a local blob",
        ));
    }
    if !is_zip_attachment(&attachment) {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment archive inspection supports ZIP attachments only",
        ));
    }

    let bytes = crate::app::api_support::communication_blob_store()
        .read_blob(&attachment.storage_path)
        .await?;
    let report =
        inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).map_err(|error| {
            tracing::warn!(
                attachment_id = %attachment.attachment.attachment_id,
                error = %error,
                "attachment archive inspection rejected archive"
            );
            ApiError::InvalidCommunicationQuery("attachment archive inspection failed")
        })?;

    Ok(Json(AttachmentArchiveInspectionResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        report,
    }))
}

fn is_preview_allowed_by_scan_status(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    matches!(
        attachment.attachment.scan_status.as_str(),
        "not_scanned" | "clean"
    )
}

enum AttachmentPreviewKind {
    Text,
    Image,
    Audio,
    Video,
    Pdf,
}

fn attachment_preview_kind(
    attachment: &StoredCommunicationAttachmentWithBlob,
) -> Option<AttachmentPreviewKind> {
    if is_previewable_text_attachment(attachment) {
        return Some(AttachmentPreviewKind::Text);
    }
    if is_previewable_image_attachment(attachment) {
        return Some(AttachmentPreviewKind::Image);
    }
    if is_previewable_audio_attachment(attachment) {
        return Some(AttachmentPreviewKind::Audio);
    }
    if is_previewable_video_attachment(att
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/handlers/communications/communication_queries/drafts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/drafts.rs`
- Size bytes / Размер в байтах: `4184`
- Included characters / Включено символов: `4184`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::domains::communications::service::{
    CommunicationCommandService, CommunicationDraftUpsertCommand,
};

#[derive(Deserialize)]
pub(crate) struct DraftListQuery {
    pub(super) account_id: Option<String>,
    pub(super) status: Option<String>,
    pub(super) cursor: Option<String>,
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct DraftListResponse {
    pub(super) items: Vec<crate::domains::communications::drafts::CommunicationDraft>,
    pub(super) next_cursor: Option<String>,
    pub(super) has_more: bool,
}

#[derive(Deserialize)]
pub(crate) struct NewDraftRequest {
    pub(super) draft_id: String,
    pub(super) account_id: String,
    pub(super) persona_id: Option<String>,
    pub(super) to_recipients: Vec<String>,
    pub(super) cc_recipients: Option<Vec<String>>,
    pub(super) bcc_recipients: Option<Vec<String>>,
    pub(super) subject: String,
    pub(super) body_text: String,
    pub(super) body_html: Option<String>,
    pub(super) in_reply_to: Option<String>,
    pub(super) references: Option<Vec<String>>,
    pub(super) status: Option<String>,
    pub(super) scheduled_send_at: Option<DateTime<Utc>>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn get_v1_drafts(
    State(state): State<AppState>,
    Query(query): Query<DraftListQuery>,
) -> Result<Json<DraftListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::drafts::CommunicationDraftStore,
    >(pool);
    let status = query
        .status
        .as_deref()
        .and_then(crate::domains::communications::drafts::DraftStatus::parse);
    let page = store
        .list_page(
            query.account_id.as_deref(),
            status,
            query.cursor.as_deref(),
            query.limit.unwrap_or(100),
        )
        .await?;
    Ok(Json(DraftListResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn post_v1_draft(
    State(state): State<AppState>,
    Json(req): Json<NewDraftRequest>,
) -> Result<Json<crate::domains::communications::drafts::CommunicationDraft>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let draft = CommunicationCommandService::new(pool)
        .upsert_draft(CommunicationDraftUpsertCommand {
            draft_id: req.draft_id,
            account_id: req.account_id,
            persona_id: req.persona_id,
            to_recipients: req.to_recipients,
            cc_recipients: req.cc_recipients,
            bcc_recipients: req.bcc_recipients,
            subject: req.subject,
            body_text: req.body_text,
            body_html: req.body_html,
            in_reply_to: req.in_reply_to,
            references: req.references,
            status: req.status,
            scheduled_send_at: req.scheduled_send_at,
            metadata: req.metadata,
        })
        .await?;
    Ok(Json(draft))
}

pub(crate) async fn get_v1_draft(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Result<Json<crate::domains::communications::drafts::CommunicationDraft>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::drafts::CommunicationDraftStore,
    >(pool);
    store
        .get(&draft_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn delete_v1_draft(
    State(state): State<AppState>,
    Path(draft_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = CommunicationCommandService::new(pool)
        .delete_draft(&draft_id)
        .await?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}
```

### `backend/src/app/handlers/communications/communication_queries/folders.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/communication_queries/folders.rs`
- Size bytes / Размер в байтах: `4513`
- Included characters / Включено символов: `4513`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::domains::communications::folders::{
    CommunicationFolder, CommunicationFolderListPage, CommunicationFolderListQuery,
    CommunicationFolderStore, FolderMessageActionResponse, FolderMessageListQuery,
    FolderMessagePage, NewCommunicationFolder, UpdateCommunicationFolder,
};
use crate::domains::communications::service::CommunicationCommandService;

#[derive(Deserialize)]
pub(crate) struct FoldersQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct FolderMessagesQuery {
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct FolderDeleteResponse {
    pub(crate) deleted: bool,
}

pub(crate) async fn get_v1_mail_folders(
    State(state): State<AppState>,
    Query(query): Query<FoldersQuery>,
) -> Result<Json<CommunicationFolderListPage>, ApiError> {
    let page = folder_store(&state)?
        .list(CommunicationFolderListQuery {
            account_id: query.account_id.as_deref(),
            cursor: query.cursor.as_deref(),
            limit: query.limit.unwrap_or(500),
        })
        .await?;
    Ok(Json(page))
}

pub(crate) async fn post_v1_mail_folder(
    State(state): State<AppState>,
    Json(request): Json<NewCommunicationFolder>,
) -> Result<Json<CommunicationFolder>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let folder = CommunicationCommandService::new(pool)
        .create_folder(request)
        .await?;
    Ok(Json(folder))
}

pub(crate) async fn put_v1_mail_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    Json(request): Json<UpdateCommunicationFolder>,
) -> Result<Json<CommunicationFolder>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(folder) = CommunicationCommandService::new(pool)
        .update_folder(&folder_id, request)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(folder))
}

pub(crate) async fn delete_v1_mail_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
) -> Result<Json<FolderDeleteResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = CommunicationCommandService::new(pool)
        .delete_folder(&folder_id)
        .await?;
    Ok(Json(FolderDeleteResponse { deleted }))
}

pub(crate) async fn get_v1_mail_folder_messages(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    Query(query): Query<FolderMessagesQuery>,
) -> Result<Json<FolderMessagePage>, ApiError> {
    let page = folder_store(&state)?
        .list_messages(FolderMessageListQuery {
            folder_id: &folder_id,
            cursor: query.cursor.as_deref(),
            limit: query.limit.unwrap_or(250),
        })
        .await?;
    Ok(Json(page))
}

pub(crate) async fn post_v1_copy_message_to_folder(
    State(state): State<AppState>,
    Path((folder_id, message_id)): Path<(String, String)>,
) -> Result<Json<FolderMessageActionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(response) = CommunicationCommandService::new(pool)
        .copy_message_to_folder(&folder_id, &message_id)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(response))
}

pub(crate) async fn post_v1_move_message_to_folder(
    State(state): State<AppState>,
    Path((folder_id, message_id)): Path<(String, String)>,
) -> Result<Json<FolderMessageActionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(response) = CommunicationCommandService::new(pool)
        .move_message_to_folder(&folder_id, &message_id)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(response))
}

fn folder_store(state: &AppState) -> Result<CommunicationFolderStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::app_store::<CommunicationFolderStore>(pool))
}
```
