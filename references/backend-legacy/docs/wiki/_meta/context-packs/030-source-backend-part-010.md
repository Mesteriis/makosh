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

- Chunk ID / ID чанка: `030-source-backend-part-010`
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

### `backend/src/app/handlers/decisions/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/decisions/mod.rs`
- Size bytes / Размер в байтах: `188`
- Included characters / Включено символов: `188`
- Truncated / Обрезано: `no`

```rust
mod handlers;
mod models;

pub(crate) use handlers::{get_v1_decisions, put_v1_decision_review};
pub(crate) use models::{DecisionListQuery, DecisionListResponse, DecisionReviewApiRequest};
```

### `backend/src/app/handlers/decisions/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/decisions/models.rs`
- Size bytes / Размер в байтах: `540`
- Included characters / Включено символов: `540`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use crate::domains::decisions::Decision;

#[derive(Debug, Deserialize)]
pub(crate) struct DecisionListQuery {
    pub(crate) entity_kind: Option<String>,
    pub(crate) entity_id: Option<String>,
    pub(crate) review_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DecisionReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DecisionListResponse {
    pub(crate) items: Vec<Decision>,
}
```

### `backend/src/app/handlers/documents/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/documents/mod.rs`
- Size bytes / Размер в байтах: `8216`
- Included characters / Включено символов: `8216`
- Truncated / Обрезано: `no`

```rust
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
use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};
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

pub(crate) async fn get_document_processing(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<Json<DocumentProcessingRecord>, ApiError> {
    let _ = validate_non_empty_document_id(document_id.as_str())?;

    Ok(Json(
        document_processing_store(&state)?
            .document_processing(&document_id)
            .await?,
    ))
}

pub(crate) async fn get_document_processing_jobs(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<DocumentProcessingJobsResponse>, ApiError> {
    let query = parse_document_processing_jobs_query(raw_query.as_deref())?;
    let items = document_processing_store(&state)?
        .list_jobs(query.limit)
        .await?;

    Ok(Json(DocumentProcessingJobsResponse { items }))
}

pub(crate) async fn post_document_processing_job_retry(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<DocumentProcessingRetryApiRequest>,
) -> Result<Json<DocumentProcessingRetryApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(job_id, actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::document_processing_job_retry(
            &command.actor_id,
            &command.job_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = crate::domains::documents::processing::DocumentProcessingCommandService::new(pool)
        .retry_failed_job_manual(&command)
        .await?;
    Ok(Json(result.into()))
}
```

### `backend/src/app/handlers/events/handlers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/events/handlers.rs`
- Size bytes / Размер в байтах: `15449`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::Duration;

use axum::Json;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Sse};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{Instant, sleep};

use crate::app::api_support::{
    AppendEventRequest, AppendEventResponse, AuditEventsQuery, AuditEventsResponse, api_audit_log,
    event_store,
};
use crate::app::{ApiError, AppState};
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::bus::sanitize_event_payload;
use crate::platform::events::{EventEnvelope, EventTrace, StoredEventEnvelope};

pub(crate) async fn post_event(
    State(state): State<AppState>,
    Json(request): Json<AppendEventRequest>,
) -> Result<(StatusCode, Json<AppendEventResponse>), ApiError> {
    let actor_id = "makosh-frontend".to_string();

    let store = event_store(&state)?;
    let event = request.into_new_event()?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_append(
            actor_id,
            event.event_id.clone(),
        ))
        .await?;
    let position = store.append(&event).await?;

    Ok((
        StatusCode::CREATED,
        Json(AppendEventResponse {
            event_id: event.event_id,
            position,
        }),
    ))
}

pub(crate) async fn get_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventEnvelope>, ApiError> {
    let actor_id = "makosh-frontend".to_string();

    let store = event_store(&state)?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_get(actor_id, event_id.clone()))
        .await?;
    let Some(event) = store.get_by_id(&event_id).await? else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(event))
}

#[derive(Deserialize)]
pub(crate) struct EventTraceQuery {
    limit: Option<u32>,
}

pub(crate) async fn get_event_trace(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(query): Query<EventTraceQuery>,
) -> Result<Json<EventTrace>, ApiError> {
    let store = event_store(&state)?;
    let Some(trace) = store
        .trace_by_event_id(&event_id, query.limit.unwrap_or(1000))
        .await?
    else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(sanitize_trace_payloads(trace)))
}

pub(crate) async fn get_event_trace_by_correlation(
    State(state): State<AppState>,
    Path(correlation_id): Path<String>,
    Query(query): Query<EventTraceQuery>,
) -> Result<Json<EventTrace>, ApiError> {
    let store = event_store(&state)?;
    let trace = store
        .trace_by_correlation_id(&correlation_id, query.limit.unwrap_or(1000))
        .await?;

    Ok(Json(sanitize_trace_payloads(trace)))
}

pub(crate) async fn get_event_children(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(query): Query<EventTraceQuery>,
) -> Result<Json<Vec<StoredEventEnvelope>>, ApiError> {
    let store = event_store(&state)?;
    let children = store
        .list_children(&event_id, query.limit.unwrap_or(1000))
        .await?
        .into_iter()
        .map(sanitize_stored_event_payload)
        .collect();

    Ok(Json(children))
}

#[derive(Deserialize)]
pub(crate) struct EventListQuery {
    after_position: Option<i64>,
    limit: Option<u32>,
    wait_seconds: Option<u64>,
}

#[derive(Serialize)]
pub(crate) struct EventListResponse {
    items: Vec<StoredEventEnvelope>,
    next_after_position: i64,
    has_more: bool,
}

pub(crate) async fn get_events(
    State(state): State<AppState>,
    Query(query): Query<EventListQuery>,
) -> Result<Json<EventListResponse>, ApiError> {
    let after_position = query.after_position.unwrap_or(0);
    if after_position < 0 {
        return Err(ApiError::InvalidCommunicationQuery(
            "after_position must be non-negative",
        ));
    }
    let limit = query.limit.unwrap_or(100).clamp(1, 1000);
    let wait_seconds = query.wait_seconds.unwrap_or(0).clamp(0, 30);

    let store = event_store(&state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::event_list(
            "makosh-frontend",
            after_position,
            limit,
            wait_seconds,
        ))
        .await?;

    let deadline = Instant::now() + Duration::from_secs(wait_seconds);
    loop {
        let fetch_limit = limit.saturating_add(1).min(1000);
        let events = store
            .list_after_position(after_position, fetch_limit)
            .await?;
        if !events.is_empty() || Instant::now() >= deadline {
            return Ok(Json(event_list_response(after_position, limit, events)));
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        sleep(remaining.min(Duration::from_millis(500))).await;
    }
}

#[derive(Deserialize)]
pub(crate) struct EventStreamQuery {
    after_position: Option<i64>,
    batch_size: Option<u32>,
    heartbeat_seconds: Option<u64>,
}

pub(crate) async fn get_events_stream(
    State(state): State<AppState>,
    Query(query): Query<EventStreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let after_position = query.after_position.unwrap_or(0);
    if after_position < 0 {
        return Err(ApiError::InvalidCommunicationQuery(
            "after_position must be non-negative",
        ));
    }

    let store = event_store(&state)?;
    let stream_state = EventStreamState {
        store,
        after_position,
        batch_size: query.batch_size.unwrap_or(100).clamp(1, 1000),
        heartbeat: Duration::from_secs(query.heartbeat_seconds.unwrap_or(15).clamp(1, 60)),
        pending: VecDeque::new(),
    };

    let stream = futures::stream::unfold(stream_state, |mut state| async move {
        loop {
            if let Some(envelope) = state.pending.pop_front() {
                state.after_position = envelope.position;
                return Some((Ok(stored_event_to_sse(envelope)), state));
            }

            match state
                .store
                .list_after_position(state.after_position, state.batch_size)
                .await
            {
                Ok(events) if !events.is_empty() => {
                    state.pending = events.into();
                }
                Ok(_) => {
                    sleep(state.heartbeat).await;
                    return Some((Ok(heartbeat_event(state.after_position)), state));
                }
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        after_position = state.after_position,
                        "event SSE replay polling failed"
                    );
                    sleep(state.heartbeat).await;
                    return Some((Ok(stream_error_event()), state));
                }
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub(crate) async fn get_events_websocket(
    State(state): State<AppState>,
    Query(query): Query<EventStreamQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    let after_position = query.after_position.unwrap_or(0);
    if after_position < 0 {
        return Err(ApiError::InvalidCommunicationQuery(
            "after_position must be non-negative",
        ));
    }

    let store = event_store(&state)?;
    let stream_state = EventStreamState {
        store,
        after_position,
        batch_size: query.batch_size.unwrap_or(100).clamp(1, 1000),
        heartbeat: Duration::from_secs(query.heartbeat_seconds.unwrap_or(15).clamp(1, 60)),
        pending: VecDeque::new(),
    };

    Ok(ws.on_upgrade(move |socket| event_websocket_loop(socket, stream_state)))
}

pub(crate) async fn get_audit_events(
    State(state): State<AppState>,
    Query(query): Query<AuditEventsQuery>,
) -> Result<Json<AuditEventsResponse>, ApiError> {
    let audit_log = api_audit_log(&state)?;
    let items = audit_log
        .list_event_records(
            query.target_id.as_deref(),
            query.actor_id.as_deref(),
            query.after_audit_id.unwrap_or(0),
            query.limit.unwrap_or(100),
        )
        .await?;

    Ok(Json(AuditEventsResponse { items }))
}

struct EventStreamState {
    store: crate::platform::events::EventStore,
    after_position: i64,
    batch_size: u32,
    heartbeat: Duration,
    pending: VecDeque<StoredEventEnvelope>,
}

fn stored_event_to_sse(envelope: StoredEventEnvelope) -> Event {
    let position = envelope.position;
    match serde_json::to_string(&sanitize_stored_event_payload(envelope)) {
        Ok(data) => Event::default()
            .id(position.to_string())
            .event("event")
            .data(data),
        Err(error) => {
            tracing::warn!(error = %error, position, "event SSE serialization failed");
            stream_error_event()
        }
    }
}

async fn event_websocket_loop(mut socket: WebSocket, mut state: EventStreamState) {
    loop {
        if let Some(envelope) = state.pending.pop_front() {
            state.after_position = envelope.position;
            if send_ws_json(
                &mut socket,
                "event",
                serde_json::to_value(sanitize_stored_event_payload(envelope)),
            )
            .await
            {
                continue;
            }
            return;
        }

        match state
            .store
            .list_after_position(state.after_position, state.batch_size)
            .await
        {
            Ok(events) if !events.is_empty() => {
                state.pending = events.into();
            }
            Ok(_) => {
                sleep(state.heartbeat).await;
                if !send_ws_json(
                    &mut socket,
                    "heartbeat",
                    Ok(json!({ "after_position": state.after_position })),
                )
                .await
                {
                    return;
                }
            }
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    after_position = state.after_position,
                    "event WebSocket replay polling failed"
                );
                sleep(state.heartbeat).await;
                if !send_ws_json(
                    &mut socket,
                    "error",
                    Ok(json!({ "error": "event_stream_unavailable" })),
                )
                .await
                {
                    return;
                }
            }
        }
    }
}

async fn send_ws_json(
    socket: &mut WebSocket,
    message_type: &str,
    data: Result<serde_json::Value, serde_json::Error>,
) -> bool {
    let Ok(data) = data else {
        tracing::warn!(message_type, "event WebSocket serialization failed");
        return false;
    };
    let payload = json!({ "type": message_type, "data": data });
    socket
        .send(Message::Text(payload.to_string().into()))
        .await
        .is_ok()
}

fn event_list_response(
    after_position: i64,
    limit: u32,
    mut events: Vec<StoredEventEnvelope>,
) -> EventListResponse {
    let has_more = events.len() > limit as usize;
    events.truncate(limit as usize);
    let next_after_position = events
        .last()
        .map(|event| event.position)
        .unwrap_or(after_position);

    EventListResponse {
        items: events,
        next_after_position,
        has_more,
    }
}

fn heartbeat_event(after_position: i64) -> Event {
    Event::default()
        .event("heartbeat")
        .data(json!({ "after_position": after_position }).to_string())
}

fn stream_error_event() -> Event {
    Event::default()
        .event("error
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/handlers/events/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/events/mod.rs`
- Size bytes / Размер в байтах: `43`
- Included characters / Включено символов: `43`
- Truncated / Обрезано: `no`

```rust
mod handlers;

pub(crate) use handlers::*;
```

### `backend/src/app/handlers/graph/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/graph/mod.rs`
- Size bytes / Размер в байтах: `8464`
- Included characters / Включено символов: `8464`
- Truncated / Обрезано: `no`

```rust
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

pub(crate) async fn get_graph_summary(
    State(state): State<AppState>,
) -> Result<Json<crate::domains::graph::core::GraphSummary>, ApiError> {
    Ok(Json(graph_store(&state)?.summary().await?))
}

pub(crate) async fn get_graph_nodes(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<crate::domains::graph::core::GraphNode>>, ApiError> {
    let query = parse_graph_nodes_query(raw_query.as_deref())?;
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(
        graph_store(&state)?.list_nodes_for_picker(limit).await?,
    ))
}

pub(crate) async fn get_graph_neighborhood(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<crate::domains::graph::core::GraphNeighborhood>, ApiError> {
    let query = parse_graph_neighborhood_query(raw_query.as_deref())?;
    if query.depth.unwrap_or(1) != 1 {
        return Err(ApiError::InvalidGraphQuery("depth supports only 1"));
    }
    let Some(node_id) = query
        .node_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Err(ApiError::GraphNotFound);
    };
    let Some(neighborhood) = graph_store(&state)?.neighborhood(node_id).await? else {
        return Err(ApiError::GraphNotFound);
    };
    Ok(Json(neighborhood))
}

pub(crate) async fn get_graph_search(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<crate::domains::graph::core::GraphNode>>, ApiError> {
    let query = parse_graph_search_query(raw_query.as_deref())?;
    let search = query.q.as_deref().unwrap_or_default().trim();
    if search.is_empty() {
        return Err(ApiError::InvalidGraphQuery("q must not be empty"));
    }
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(
        graph_store(&state)?.search_nodes(search, limit).await?,
    ))
}
```

### `backend/src/app/handlers/obligations/handlers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/obligations/handlers.rs`
- Size bytes / Размер в байтах: `4306`
- Included characters / Включено символов: `4306`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use super::models::{ObligationListQuery, ObligationListResponse, ObligationReviewApiRequest};
use crate::app::{ApiError, AppState};
use crate::application::ObligationReviewApplicationService;
use crate::domains::obligations::{
    Obligation, ObligationEntityKind, ObligationReviewState, ObligationStore,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};

const OBLIGATION_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_OBLIGATION_LIMIT: i64 = 50;
const MIN_OBLIGATION_LIMIT: i64 = 1;
const MAX_OBLIGATION_LIMIT: i64 = 100;

pub(crate) async fn get_v1_obligations(
    State(state): State<AppState>,
    Query(query): Query<ObligationListQuery>,
) -> Result<Json<ObligationListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let store = obligation_store(&state)?;
    let items = match (
        query.review_state.as_deref(),
        query.entity_kind.as_deref(),
        query.entity_id.as_deref(),
    ) {
        (Some(review_state), None, None) => {
            let review_state = parse_review_state(review_state)?;
            store.list_by_review_state(review_state, limit).await?
        }
        (None, Some(entity_kind), Some(entity_id)) => {
            let entity_kind = parse_required_entity_kind(Some(entity_kind))?;
            let entity_id = validate_required_query_value(Some(entity_id))?;
            store
                .list_for_entity(entity_kind, &entity_id, limit)
                .await?
        }
        (Some(_), _, _) => {
            return Err(ApiError::InvalidObligationQuery(
                "review_state cannot be combined with entity filters",
            ));
        }
        (None, _, _) => {
            return Err(ApiError::InvalidObligationQuery(
                "missing required obligation query field",
            ));
        }
    };

    Ok(Json(ObligationListResponse { items }))
}

pub(crate) async fn put_v1_obligation_review(
    State(state): State<AppState>,
    Path(obligation_id): Path<String>,
    Json(request): Json<ObligationReviewApiRequest>,
) -> Result<Json<Obligation>, ApiError> {
    let obligation_id = validate_required_query_value(Some(&obligation_id))?;
    let review_state = parse_review_state(&request.review_state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::obligation_review_set(
            OBLIGATION_API_ACTOR_ID,
            &obligation_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let obligation = ObligationReviewApplicationService::new(pool)
        .review_manual(&obligation_id, review_state)
        .await?;

    Ok(Json(obligation))
}

fn obligation_store(state: &AppState) -> Result<ObligationStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::app_store::<ObligationStore>(
        pool.clone(),
    ))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_required_entity_kind(value: Option<&str>) -> Result<ObligationEntityKind, ApiError> {
    let value = validate_required_query_value(value)?;
    ObligationEntityKind::parse(&value).map_err(ApiError::from)
}

fn parse_review_state(value: &str) -> Result<ObligationReviewState, ApiError> {
    ObligationReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidObligationQuery(
            "missing required obligation query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_OBLIGATION_LIMIT);
    if !(MIN_OBLIGATION_LIMIT..=MAX_OBLIGATION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidObligationQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
```

### `backend/src/app/handlers/obligations/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/obligations/mod.rs`
- Size bytes / Размер в байтах: `198`
- Included characters / Включено символов: `198`
- Truncated / Обрезано: `no`

```rust
mod handlers;
mod models;

pub(crate) use handlers::{get_v1_obligations, put_v1_obligation_review};
pub(crate) use models::{ObligationListQuery, ObligationListResponse, ObligationReviewApiRequest};
```

### `backend/src/app/handlers/obligations/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/obligations/models.rs`
- Size bytes / Размер в байтах: `552`
- Included characters / Включено символов: `552`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use crate::domains::obligations::Obligation;

#[derive(Debug, Deserialize)]
pub(crate) struct ObligationListQuery {
    pub(crate) entity_kind: Option<String>,
    pub(crate) entity_id: Option<String>,
    pub(crate) review_state: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ObligationReviewApiRequest {
    pub(crate) review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ObligationListResponse {
    pub(crate) items: Vec<Obligation>,
}
```

### `backend/src/app/handlers/organizations/core_records.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/core_records.rs`
- Size bytes / Размер в байтах: `6452`
- Included characters / Включено символов: `6452`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::application::OrganizationContactLinkApplicationService;
use crate::domains::organizations::core::{
    OrgAliasStore, OrgContactLink, OrgContactLinkStore, OrgDepartment, OrgDepartmentStore,
    OrgDomainStore, OrgIdentityStore, OrganizationAlias, OrganizationDomain, OrganizationIdentity,
    RelatedOrgStore, RelatedOrganization,
};
use crate::domains::organizations::service::OrganizationCommandService;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgIdentitiesResponse {
    items: Vec<OrganizationIdentity>,
}

pub(crate) async fn get_org_identities(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgIdentitiesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgIdentityStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgIdentitiesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgIdentityRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

pub(crate) async fn post_org_identity(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgIdentityRequest>,
) -> Result<Json<OrganizationIdentity>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = database_pool(&state)?;
    let identity = OrganizationCommandService::new(pool)
        .add_identity_manual(
            &org_id,
            &req.identity_type,
            &req.identity_value,
            requested_source,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(identity))
}

#[derive(Serialize)]
pub(crate) struct OrgAliasesResponse {
    items: Vec<OrganizationAlias>,
}

pub(crate) async fn get_org_aliases(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgAliasesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgAliasStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgAliasesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgAliasRequest {
    name: String,
    alias_type: String,
    source: Option<String>,
}

pub(crate) async fn post_org_alias(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgAliasRequest>,
) -> Result<Json<OrganizationAlias>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = database_pool(&state)?;
    let alias = OrganizationCommandService::new(pool)
        .add_alias_manual(&org_id, &req.name, &req.alias_type, requested_source)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(alias))
}

#[derive(Serialize)]
pub(crate) struct OrgDomainsResponse {
    items: Vec<OrganizationDomain>,
}

pub(crate) async fn get_org_domains(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgDomainsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgDomainStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgDomainsResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgDepartmentsResponse {
    items: Vec<OrgDepartment>,
}

pub(crate) async fn get_org_departments(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgDepartmentsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgDepartmentStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgDepartmentsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrgDepartmentRequest {
    name: String,
    description: Option<String>,
    parent_id: Option<String>,
}

pub(crate) async fn post_org_department(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<NewOrgDepartmentRequest>,
) -> Result<Json<OrgDepartment>, ApiError> {
    let pool = database_pool(&state)?;
    let dept = OrganizationCommandService::new(pool)
        .add_department_manual(
            &org_id,
            &req.name,
            req.description.as_deref(),
            req.parent_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dept))
}

#[derive(Serialize)]
pub(crate) struct OrgContactsResponse {
    items: Vec<OrgContactLink>,
}

pub(crate) async fn get_org_contacts(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgContactsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgContactLinkStore>(pool)
        .list_by_org(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgContactsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct LinkOrgContactRequest {
    person_id: String,
    role: Option<String>,
    department: Option<String>,
    source: Option<String>,
}

pub(crate) async fn post_org_contact_link(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(req): Json<LinkOrgContactRequest>,
) -> Result<Json<OrgContactLink>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = database_pool(&state)?;
    let link = OrganizationContactLinkApplicationService::new(pool)
        .link_contact_manual(
            &org_id,
            &req.person_id,
            req.role.as_deref(),
            req.department.as_deref(),
            requested_source,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(link))
}

#[derive(Serialize)]
pub(crate) struct OrgRelatedResponse {
    items: Vec<RelatedOrganization>,
}

pub(crate) async fn get_org_related(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgRelatedResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<RelatedOrgStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRelatedResponse { items }))
}
```

### `backend/src/app/handlers/organizations/directory.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/directory.rs`
- Size bytes / Размер в байтах: `3688`
- Included characters / Включено символов: `3688`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::api::{Organization, OrganizationStore, OrganizationUpdate};
use crate::domains::organizations::service::OrganizationCommandService;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrganizationListResponse {
    items: Vec<Organization>,
}

#[derive(Deserialize)]
pub(crate) struct OrganizationListQuery {
    org_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_organizations(
    State(state): State<AppState>,
    Query(query): Query<OrganizationListQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrganizationStore>(pool)
        .list(query.org_type.as_deref(), query.limit.unwrap_or(50))
        .await?;
    Ok(Json(OrganizationListResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOrganizationRequest {
    display_name: String,
    org_type: Option<String>,
}

pub(crate) async fn post_organization(
    State(state): State<AppState>,
    Json(req): Json<NewOrganizationRequest>,
) -> Result<Json<Organization>, ApiError> {
    let pool = database_pool(&state)?;
    let org = OrganizationCommandService::new(pool)
        .create_organization_manual(&req.display_name, req.org_type.as_deref())
        .await?;
    Ok(Json(org))
}

pub(crate) async fn get_organization(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Organization>, ApiError> {
    let pool = database_pool(&state)?;
    crate::app::api_support::app_store::<OrganizationStore>(pool)
        .get(&org_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_organization(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(update): Json<OrganizationUpdate>,
) -> Result<Json<Organization>, ApiError> {
    let pool = database_pool(&state)?;
    let org = OrganizationCommandService::new(pool)
        .update_organization_manual(&org_id, &update)
        .await?;
    Ok(Json(org))
}

#[derive(Deserialize)]
pub(crate) struct OrganizationSearchQuery {
    q: String,
    limit: Option<i64>,
}

pub(crate) async fn get_organization_search(
    State(state): State<AppState>,
    Query(query): Query<OrganizationSearchQuery>,
) -> Result<Json<OrganizationListResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let store = crate::app::api_support::app_store::<OrganizationStore>(pool);
    let all = store.list(None, 200).await?;
    let q = query.q.trim().to_lowercase();
    let items: Vec<_> = all
        .into_iter()
        .filter(|o| {
            o.display_name.to_lowercase().contains(&q)
                || o.legal_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
                || o.website
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
        })
        .take(query.limit.unwrap_or(20).clamp(1, 100) as usize)
        .collect();
    Ok(Json(OrganizationListResponse { items }))
}

pub(crate) async fn post_organization_archive(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    OrganizationCommandService::new(pool)
        .archive_organization_manual(&org_id)
        .await?;
    Ok(Json(json!({"archived": true})))
}
```

### `backend/src/app/handlers/organizations/enrichment.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/enrichment.rs`
- Size bytes / Размер в байтах: `1195`
- Included characters / Включено символов: `1195`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::enrichment::{OrgEnrichmentResult, OrgEnrichmentStore};
use crate::domains::organizations::service::OrganizationCommandService;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgEnrichmentResponse {
    items: Vec<OrgEnrichmentResult>,
}

pub(crate) async fn get_org_enrichment(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgEnrichmentResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgEnrichmentStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgEnrichmentResponse { items }))
}

pub(crate) async fn post_org_enrich_apply(
    State(state): State<AppState>,
    Path((org_id, rid)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    OrganizationCommandService::new(pool)
        .apply_enrichment_manual(&org_id, &rid)
        .await?;
    Ok(Json(json!({"applied": true})))
}
```

### `backend/src/app/handlers/organizations/finance.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/finance.rs`
- Size bytes / Размер в байтах: `2718`
- Included characters / Включено символов: `2718`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::Value;

use crate::app::{ApiError, AppState};
use crate::domains::organizations::finance::{
    OrgCompliance, OrgComplianceStore, OrgContract, OrgContractStore, OrgFinancialStore,
    OrgProduct, OrgProductStore, OrgService, OrgServiceStore,
};

use super::support::database_pool;

pub(crate) async fn get_org_financial(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let info = crate::app::api_support::app_store::<OrgFinancialStore>(pool)
        .get(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&info).unwrap_or_default()))
}

#[derive(Serialize)]
pub(crate) struct OrgContractsResponse {
    items: Vec<OrgContract>,
}

pub(crate) async fn get_org_contracts(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgContractsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgContractStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgContractsResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgComplianceResponse {
    items: Vec<OrgCompliance>,
}

pub(crate) async fn get_org_compliance(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgComplianceResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgComplianceStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgComplianceResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgServicesResponse {
    items: Vec<OrgService>,
}

pub(crate) async fn get_org_services(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgServicesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgServiceStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgServicesResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgProductsResponse {
    items: Vec<OrgProduct>,
}

pub(crate) async fn get_org_products(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgProductsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgProductStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgProductsResponse { items }))
}
```

### `backend/src/app/handlers/organizations/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/health.rs`
- Size bytes / Размер в байтах: `1535`
- Included characters / Включено символов: `1535`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::health::{OrgHealthStore, OrgRisk, OrgRiskStore};
use crate::domains::organizations::service::OrganizationCommandService;

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgRisksResponse {
    items: Vec<OrgRisk>,
}

pub(crate) async fn get_org_risks(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgRisksResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgRiskStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgRisksResponse { items }))
}

pub(crate) async fn get_org_health(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let health = crate::app::api_support::app_store::<OrgHealthStore>(pool)
        .get(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&health).unwrap_or_default()))
}

pub(crate) async fn post_org_watchlist_toggle(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let on = OrganizationCommandService::new(pool)
        .toggle_watchlist_manual(&org_id)
        .await?;
    Ok(Json(json!({"watchlist": on})))
}
```

### `backend/src/app/handlers/organizations/investigator.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/investigator.rs`
- Size bytes / Размер в байтах: `1353`
- Included characters / Включено символов: `1353`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use serde_json::Value;

use crate::app::{ApiError, AppState};
use crate::domains::organizations::investigator::OrganizationInvestigator;

use super::support::database_pool;

pub(crate) async fn get_org_dossier(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let dossier = OrganizationInvestigator::new(pool)
        .dossier(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&dossier).unwrap_or_default()))
}

pub(crate) async fn get_org_brief(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let brief = OrganizationInvestigator::new(pool)
        .brief(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&brief).unwrap_or_default()))
}

pub(crate) async fn get_org_context_pack(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let pack = OrganizationInvestigator::new(pool)
        .context_pack(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}
```

### `backend/src/app/handlers/organizations/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/mod.rs`
- Size bytes / Размер в байтах: `326`
- Included characters / Включено символов: `326`
- Truncated / Обрезано: `no`

```rust
mod core_records;
mod directory;
mod enrichment;
mod finance;
mod health;
mod investigator;
mod support;
mod workflows;

pub(crate) use core_records::*;
pub(crate) use directory::*;
pub(crate) use enrichment::*;
pub(crate) use finance::*;
pub(crate) use health::*;
pub(crate) use investigator::*;
pub(crate) use workflows::*;
```

### `backend/src/app/handlers/organizations/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/support.rs`
- Size bytes / Размер в байтах: `503`
- Included characters / Включено символов: `503`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::app::{ApiError, AppState};
use crate::platform::observations::ObservationStore;

pub(super) fn database_pool(state: &AppState) -> Result<PgPool, ApiError> {
    state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)
        .cloned()
}

pub(super) fn observation_store(state: &AppState) -> Result<ObservationStore, ApiError> {
    Ok(crate::app::api_support::app_store::<ObservationStore>(
        database_pool(state)?,
    ))
}
```

### `backend/src/app/handlers/organizations/workflows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/organizations/workflows.rs`
- Size bytes / Размер в байтах: `2983`
- Included characters / Включено символов: `2983`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::app::{ApiError, AppState};
use crate::domains::organizations::workflows::{
    OrgPlaybook, OrgPlaybookStore, OrgPortal, OrgPortalStore, OrgProcedure, OrgProcedureStore,
    OrgTemplate, OrgTemplateStore, OrgTimelineEvent, OrgTimelineStore,
};

use super::support::database_pool;

#[derive(Serialize)]
pub(crate) struct OrgTimelineResponse {
    items: Vec<OrgTimelineEvent>,
}

#[derive(Deserialize)]
pub(crate) struct OrgTimelineQuery {
    limit: Option<i64>,
}

pub(crate) async fn get_org_timeline(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Query(query): Query<OrgTimelineQuery>,
) -> Result<Json<OrgTimelineResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgTimelineStore>(pool)
        .list(&org_id, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgTimelineResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgPortalsResponse {
    items: Vec<OrgPortal>,
}

pub(crate) async fn get_org_portals(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgPortalsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgPortalStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgPortalsResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgProceduresResponse {
    items: Vec<OrgProcedure>,
}

pub(crate) async fn get_org_procedures(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgProceduresResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgProcedureStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgProceduresResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgPlaybooksResponse {
    items: Vec<OrgPlaybook>,
}

pub(crate) async fn get_org_playbooks(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgPlaybooksResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgPlaybookStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgPlaybooksResponse { items }))
}

#[derive(Serialize)]
pub(crate) struct OrgTemplatesResponse {
    items: Vec<OrgTemplate>,
}

pub(crate) async fn get_org_templates(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgTemplatesResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = crate::app::api_support::app_store::<OrgTemplateStore>(pool)
        .list(&org_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OrgTemplatesResponse { items }))
}
```

### `backend/src/app/handlers/persons/compatibility.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/compatibility.rs`
- Size bytes / Размер в байтах: `4288`
- Included characters / Включено символов: `4046`
- Truncated / Обрезано: `no`

```rust
use super::support::*;
// ── Person Roles ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonRolesResponse {
    items: Vec<PersonRole>,
}

pub(crate) async fn get_person_roles(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonRolesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonRoleStore>(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonRolesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonRoleRequest {
    role: String,
}

pub(crate) async fn post_person_role(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonRoleRequest>,
) -> Result<Json<PersonRole>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .assign_role_manual(&person_id, &req.role)
            .await?,
    ))
}

pub(crate) async fn delete_person_role(
    State(state): State<AppState>,
    Path((person_id, role)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = crate::domains::persons::service::PersonCommandService::new(pool)
        .remove_role_manual(&person_id, &role)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Person Personas ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPersonasResponse {
    items: Vec<PersonPersona>,
}

pub(crate) async fn get_person_personas(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPersonasResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonPersonaStore>(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPersonasResponse { items }))
}

pub(crate) async fn post_person_persona(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPersonaRequest>,
) -> Result<Json<PersonPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_persona_manual(&NewPersonPersona {
                person_id,
                persona_id: req.persona_id,
                name: req.name,
                context: req.context,
                default_tone: req.default_tone,
                default_language: req.default_language,
                preferred_channel: req.preferred_channel,
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonPersonaRequest {
    persona_id: String,
    name: String,
    context: Option<String>,
    default_tone: Option<String>,
    default_language: Option<String>,
    preferred_channel: Option<String>,
}

pub(crate) async fn delete_person_persona(
    State(state): State<AppState>,
    Path((_person_id, persona_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = crate::domains::persons::service::PersonCommandService::new(pool)
        .delete_person_persona_manual(&_person_id, &persona_id)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}
```

### `backend/src/app/handlers/persons/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/errors.rs`
- Size bytes / Размер в байтах: `2177`
- Included characters / Включено символов: `2177`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

impl From<EnrichmentEngineError> for ApiError {
    fn from(error: EnrichmentEngineError) -> Self {
        tracing::error!(error = %error, "enrichment engine operation failed");
        ApiError::InvalidCommunicationQuery("enrichment engine operation failed")
    }
}

impl From<PersonExpertiseError> for ApiError {
    fn from(error: PersonExpertiseError) -> Self {
        tracing::error!(error = %error, "expertise operation failed");
        ApiError::InvalidCommunicationQuery("expertise operation failed")
    }
}

impl From<PersonTrustError> for ApiError {
    fn from(error: PersonTrustError) -> Self {
        tracing::error!(error = %error, "trust operation failed");
        ApiError::InvalidCommunicationQuery("trust operation failed")
    }
}

impl From<PersonHealthError> for ApiError {
    fn from(error: PersonHealthError) -> Self {
        tracing::error!(error = %error, "health operation failed");
        ApiError::InvalidCommunicationQuery("health operation failed")
    }
}

impl From<InvestigatorError> for ApiError {
    fn from(error: InvestigatorError) -> Self {
        match error {
            InvestigatorError::PersonNotFound | InvestigatorError::DossierSnapshotNotFound => {
                ApiError::PersonIdentityNotFound
            }
            InvestigatorError::InvalidDossierReviewState => ApiError::InvalidCommunicationQuery(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
            }
        }
    }
}

impl From<AnalyticsError> for ApiError {
    fn from(error: AnalyticsError) -> Self {
        tracing::error!(error = %error, "analytics operation failed");
        ApiError::InvalidCommunicationQuery("analytics operation failed")
    }
}

impl From<ExportError> for ApiError {
    fn from(error: ExportError) -> Self {
        tracing::error!(error = %error, "export operation failed");
        ApiError::InvalidCommunicationQuery("export operation failed")
    }
}
```

### `backend/src/app/handlers/persons/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/health.rs`
- Size bytes / Размер в байтах: `2220`
- Included characters / Включено символов: `2100`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

// ── Person Health ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonHealthResponse {
    items: Vec<crate::domains::persons::health::PersonHealth>,
}

pub(crate) async fn get_person_health(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<crate::domains::persons::health::PersonHealth>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<PersonHealthStore>(pool)
        .get(&person_id)
        .await
        .map_err(ApiError::from)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn get_persons_health(
    State(state): State<AppState>,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonHealthStore>(pool)
        .list_health()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonHealthResponse { items }))
}

pub(crate) async fn get_persons_watchlist(
    State(state): State<AppState>,
) -> Result<Json<PersonHealthResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonHealthStore>(pool)
        .list_watchlist()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonHealthResponse { items }))
}

pub(crate) async fn post_person_watchlist_toggle(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let on = crate::domains::persons::service::PersonCommandService::new(pool)
        .toggle_watchlist_manual(&person_id)
        .await?;
    Ok(Json(json!({"watchlist": on})))
}
```

### `backend/src/app/handlers/persons/history.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/history.rs`
- Size bytes / Размер в байтах: `5667`
- Included characters / Включено символов: `5343`
- Truncated / Обрезано: `no`

```rust
use super::support::*;
// ── Person Analytics ────────────────────────────────────────────────────────

pub(crate) async fn get_person_analytics(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let analytics = PersonAnalyticsService::new(pool)
        .compute(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&analytics).unwrap_or_default()))
}

// ── Person Export ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct PersonDownloadQuery {
    format: Option<String>,
}

pub(crate) async fn get_person_export_handler(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(query): Query<PersonDownloadQuery>,
) -> Result<(HeaderMap, String), ApiError> {
    let format = query
        .format
        .as_deref()
        .and_then(ExportFormat::parse)
        .unwrap_or(ExportFormat::Json);
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let content = PersonExportService::new(pool)
        .export(&person_id, format.clone())
        .await
        .map_err(ApiError::from)?;
    let mut headers_map = HeaderMap::new();
    headers_map.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(format.content_type())
            .unwrap_or(HeaderValue::from_static("application/json")),
    );
    headers_map.insert(
        HeaderName::from_static("content-disposition"),
        HeaderValue::from_str(&format!(
            "attachment; filename=person_{}.{}",
            person_id,
            format.extension()
        ))
        .unwrap(),
    );
    Ok((headers_map, content))
}

// ── Person Snapshots & History Diff ─────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonSnapshotsResponse {
    items: Vec<crate::domains::persons::memory::PersonSnapshot>,
}

pub(crate) async fn get_person_snapshots(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonSnapshotsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<
        crate::domains::persons::memory::PersonSnapshotStore,
    >(pool)
    .list(&person_id)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(PersonSnapshotsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct HistoryDiffQuery {
    from: String,
    to: String,
}

pub(crate) async fn get_person_history_diff(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(query): Query<HistoryDiffQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let from_date = DateTime::parse_from_rfc3339(&query.from)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid from date"))?
        .with_timezone(&Utc);
    let to_date = DateTime::parse_from_rfc3339(&query.to)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid to date"))?
        .with_timezone(&Utc);
    let diff = crate::app::api_support::app_store::<
        crate::domains::persons::memory::PersonSnapshotStore,
    >(pool)
    .history_diff(&person_id, from_date, to_date)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&diff).unwrap_or_default()))
}

pub(crate) async fn get_identity_candidates(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<PersonIdentityCandidateListResponse>, ApiError> {
    let query = parse_person_identity_candidates_query(raw_query.as_deref())?;
    let items = person_identity_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(PersonIdentityCandidateListResponse { items }))
}

pub(crate) async fn put_identity_candidate_review(
    State(state): State<AppState>,
    Path(identity_candidate_id): Path<String>,
    Json(request): Json<PersonIdentityReviewApiRequest>,
) -> Result<Json<PersonIdentityReviewApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(identity_candidate_id, actor_id)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::person_identity_review_set(
            &command.actor_id,
            &command.identity_candidate_id,
        ))
        .await?;

    let result = crate::domains::persons::service::PersonCommandService::new(pool)
        .review_identity_candidate_manual(&command)
        .await?;

    Ok(Json(result.into()))
}

pub(crate) async fn get_person_identity(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonIdentityDetail>, ApiError> {
    let _ = validate_non_empty_person_identity_field("person_id", &person_id)?;

    let detail = person_identity_store(&state)?
        .person_identity(&person_id)
        .await?;
    Ok(Json(detail))
}
```

### `backend/src/app/handlers/persons/identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/identity.rs`
- Size bytes / Размер в байтах: `4369`
- Included characters / Включено символов: `4369`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

#[derive(Serialize)]
pub(crate) struct PersonIdentitiesResponse {
    items: Vec<PersonIdentity>,
}

#[derive(Serialize)]
pub(crate) struct IdentityTracesResponse {
    items: Vec<PersonIdentity>,
}

pub(crate) async fn get_person_identities(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonIdentitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonsIdentityStore>(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonIdentitiesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct IdentityTracesQuery {
    status: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_identity_traces(
    State(state): State<AppState>,
    Query(query): Query<IdentityTracesQuery>,
) -> Result<Json<IdentityTracesResponse>, ApiError> {
    if query.status.as_deref().unwrap_or("unattached") != "unattached" {
        return Err(ApiError::InvalidCommunicationQuery(
            "identity trace status must be unattached",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonsIdentityStore>(pool);
    let items = store
        .list_unattached(query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(IdentityTracesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonIdentityRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct NewIdentityTraceRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

pub(crate) async fn post_identity_trace(
    State(state): State<AppState>,
    Json(req): Json<NewIdentityTraceRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .create_identity_trace_manual(&req.identity_type, &req.identity_value, requested_source)
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct IdentityTraceAssignmentRequest {
    person_id: String,
}

pub(crate) async fn put_identity_trace_assignment(
    State(state): State<AppState>,
    Path(identity_id): Path<String>,
    Json(req): Json<IdentityTraceAssignmentRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .assign_identity_trace_manual(&identity_id, &req.person_id)
            .await?,
    ))
}

pub(crate) async fn post_person_identity(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonIdentityRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_identity_manual(
                &person_id,
                &req.identity_type,
                &req.identity_value,
                requested_source,
            )
            .await?,
    ))
}

pub(crate) async fn delete_person_identity(
    State(state): State<AppState>,
    Path((person_id, identity_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = crate::domains::persons::service::PersonCommandService::new(pool)
        .delete_person_identity_manual(&person_id, &identity_id)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}
```

### `backend/src/app/handlers/persons/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/intelligence.rs`
- Size bytes / Размер в байтах: `4965`
- Included characters / Включено символов: `4499`
- Truncated / Обрезано: `no`

```rust
use super::support::*;
// ── Person Enrichment ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EnrichmentResultsResponse {
    items: Vec<crate::domains::persons::enrichment_engine::EnrichmentResult>,
}

pub(crate) async fn get_person_enrichment(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<EnrichmentResultsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<EnrichmentResultStore>(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EnrichmentResultsResponse { items }))
}

pub(crate) async fn post_person_enrichment_apply(
    State(state): State<AppState>,
    Path((person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::persons::service::PersonCommandService::new(pool)
        .apply_enrichment_manual(&person_id, &result_id)
        .await?;
    Ok(Json(json!({"applied": true})))
}

pub(crate) async fn post_person_enrichment_reject(
    State(state): State<AppState>,
    Path((person_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::persons::service::PersonCommandService::new(pool)
        .reject_enrichment_manual(&person_id, &result_id)
        .await?;
    Ok(Json(json!({"rejected": true})))
}

// ── Person Expertise ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonExpertiseResponse {
    items: Vec<crate::domains::persons::expertise::PersonExpertise>,
}

pub(crate) async fn get_person_expertise(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonExpertiseStore>(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonExpertiseResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct ExpertiseSearchQuery {
    skill: String,
    limit: Option<i64>,
}

pub(crate) async fn get_person_expertise_search(
    State(state): State<AppState>,
    Query(query): Query<ExpertiseSearchQuery>,
) -> Result<Json<PersonExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonExpertiseStore>(pool)
        .search_by_skill(&query.skill, query.limit.unwrap_or(20))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonExpertiseResponse { items }))
}

// ── Person Promises ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPromisesResponse {
    items: Vec<crate::domains::persons::trust::PersonPromise>,
}

pub(crate) async fn get_person_promises(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPromisesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonPromiseStore>(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPromisesResponse { items }))
}

// ── Person Risks ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonRisksResponse {
    items: Vec<crate::domains::persons::trust::PersonRisk>,
}

pub(crate) async fn get_person_risks(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonRisksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<PersonRiskStore>(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonRisksResponse { items }))
}
```

### `backend/src/app/handlers/persons/investigator.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/persons/investigator.rs`
- Size bytes / Размер в байтах: `3817`
- Included characters / Включено символов: `3709`
- Truncated / Обрезано: `no`

```rust
use super::support::*;
// ── Person Investigator ────────────────────────────────────────────────────

pub(crate) async fn post_person_investigate(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let (dossier, snapshot) = PersonInvestigator::new(pool)
        .assemble_cache_and_record_refresh(
            &person_id,
            "investigate",
            "persons_api.post_person_investigate",
            "post_person_investigate",
            format!("persona://{person_id}/investigate"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dossier_snapshot_response(&dossier, &snapshot)))
}

pub(crate) async fn get_person_dossier(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let (dossier, snapshot) = PersonInvestigator::new(pool)
        .assemble_cache_and_record_refresh(
            &person_id,
            "dossier_read_refresh",
            "persons_api.get_person_dossier",
            "get_person_dossier",
            format!("persona://{person_id}/dossier"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dossier_snapshot_response(&dossier, &snapshot)))
}

#[derive(Deserialize)]
pub(crate) struct DossierReviewRequest {
    review_state: String,
}

pub(crate) async fn put_person_dossier_review(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<DossierReviewRequest>,
) -> Result<Json<Value>, ApiError> {
    let review_state = DossierReviewState::parse(&req.review_state).map_err(ApiError::from)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let snapshot = crate::domains::persons::service::PersonCommandService::new(pool)
        .review_dossier_manual(&person_id, review_state)
        .await?;
    Ok(Json(dossier_snapshot_only_response(&snapshot)))
}

fn dossier_snapshot_response(dossier: &PersonDossier, snapshot: &DossierSnapshot) -> Value {
    let mut value = serde_json::to_value(dossier).unwrap_or_default();
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "dossier_snapshot_id".to_owned(),
            json!(snapshot.dossier_snapshot_id),
        );
        object.insert("review_state".to_owned(), json!(snapshot.review_state));
        object.insert("reviewed_by".to_owned(), json!(snapshot.reviewed_by));
        object.insert("reviewed_at".to_owned(), json!(snapshot.reviewed_at));
    }
    value
}

fn dossier_snapshot_only_response(snapshot: &DossierSnapshot) -> Value {
    json!({
        "dossier_snapshot_id": snapshot.dossier_snapshot_id,
        "persona_id": snapshot.persona_id,
        "review_state": snapshot.review_state,
        "reviewed_by": snapshot.reviewed_by,
        "reviewed_at": snapshot.reviewed_at,
        "generated_at": snapshot.generated_at,
        "updated_at": snapshot.updated_at
    })
}

pub(crate) async fn get_person_meeting_prep(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let prep = PersonInvestigator::new(pool)
        .meeting_prep(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&prep).unwrap_or_default()))
}
```
