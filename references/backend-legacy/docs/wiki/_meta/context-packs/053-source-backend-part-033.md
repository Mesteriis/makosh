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

- Chunk ID / ID чанка: `053-source-backend-part-033`
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

### `backend/src/domains/signal_hub/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/service.rs`
- Size bytes / Размер в байтах: `9169`
- Included characters / Включено символов: `9169`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::policies::{SignalPolicyDecision, SignalPolicyEvaluator};
use super::store::{SignalHubError, SignalHubStore};
use crate::platform::events::{
    EventEnvelope, EventStore, EventStoreError, NewEventEnvelope, StoredEventEnvelope,
};

pub const SIGNAL_HUB_RAW_SIGNAL_CONSUMER: &str = "signal_hub_raw_signal_dispatcher";
const SIGNAL_HUB_RAW_SIGNAL_RUNTIME_SOURCE: &str = "system";

pub async fn process_signal_hub_raw_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    if !event.event.event_type.starts_with("signal.raw.") {
        return Ok(());
    }

    let service =
        SignalHubSignalService::new(SignalHubStore::new(pool.clone()), EventStore::new(pool));
    service
        .process_raw_signal(&event.event)
        .await
        .map(|_| ())
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn signal_hub_raw_dispatcher_allows_processing(
    signal_store: &SignalHubStore,
) -> Result<bool, SignalHubError> {
    signal_store.restore_system_sources().await?;
    Ok(crate::platform::events::runtime_allows_processing(
        signal_store.pool(),
        SIGNAL_HUB_RAW_SIGNAL_RUNTIME_SOURCE,
        SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
        &json!({
            "label": "Signal Hub raw signal dispatcher",
            "scope": "consumer",
        }),
    )
    .await?)
}

#[derive(Clone)]
pub struct SignalHubSignalService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalHubSignalService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn process_raw_signal(
        &self,
        raw_event: &EventEnvelope,
    ) -> Result<SignalProcessingOutcome, SignalHubError> {
        let parsed = ParsedRawSignal::parse(raw_event)?;
        let connection_id = self
            .signal_store
            .resolve_connection_id_for_event(&parsed.source_code, raw_event)
            .await?;
        let policies = self.signal_store.list_active_policies().await?;
        let decision = SignalPolicyEvaluator::new(Utc::now()).decide(
            &parsed.source_code,
            connection_id.as_deref(),
            &raw_event.event_type,
            &policies,
        );

        match decision {
            SignalPolicyDecision::Allow => {
                let accepted = build_derived_event(
                    raw_event,
                    &format!(
                        "signal.accepted.{}.{}",
                        parsed.source_code, parsed.event_kind
                    ),
                    signal_decision_payload("accepted", None, connection_id.as_deref()),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&accepted)
                    .await?;
                Ok(SignalProcessingOutcome::Accepted {
                    event_id: accepted.event_id,
                })
            }
            SignalPolicyDecision::Rejected { reason } => {
                let rejected = build_derived_event(
                    raw_event,
                    &format!(
                        "signal.rejected.{}.{}",
                        parsed.source_code, parsed.event_kind
                    ),
                    signal_decision_payload(
                        "rejected",
                        Some(reason.as_str()),
                        connection_id.as_deref(),
                    ),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&rejected)
                    .await?;
                Ok(SignalProcessingOutcome::Rejected { reason })
            }
            SignalPolicyDecision::Muted { reason } => {
                let muted = build_derived_event(
                    raw_event,
                    &format!("signal.muted.{}.{}", parsed.source_code, parsed.event_kind),
                    signal_decision_payload(
                        "muted",
                        Some(reason.as_str()),
                        connection_id.as_deref(),
                    ),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&muted)
                    .await?;
                Ok(SignalProcessingOutcome::Muted { reason })
            }
            SignalPolicyDecision::Paused { reason } => {
                self.signal_store
                    .record_paused_event(
                        raw_event,
                        &parsed.source_code,
                        connection_id.as_deref(),
                        &reason,
                    )
                    .await?;
                let paused = build_derived_event(
                    raw_event,
                    &format!("signal.paused.{}.{}", parsed.source_code, parsed.event_kind),
                    signal_decision_payload(
                        "paused",
                        Some(reason.as_str()),
                        connection_id.as_deref(),
                    ),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&paused)
                    .await?;
                Ok(SignalProcessingOutcome::Paused { reason })
            }
        }
    }

    pub async fn replay_raw_signal(
        &self,
        raw_event: &EventEnvelope,
    ) -> Result<SignalProcessingOutcome, SignalHubError> {
        let parsed = ParsedRawSignal::parse(raw_event)?;
        let connection_id = self
            .signal_store
            .resolve_connection_id_for_event(&parsed.source_code, raw_event)
            .await?;
        let accepted = build_derived_event(
            raw_event,
            &format!(
                "signal.accepted.{}.{}",
                parsed.source_code, parsed.event_kind
            ),
            signal_decision_payload("replayed", None, connection_id.as_deref()),
        )?;
        self.event_store
            .append_for_dispatch_idempotent(&accepted)
            .await?;
        Ok(SignalProcessingOutcome::Accepted {
            event_id: accepted.event_id,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalProcessingOutcome {
    Accepted { event_id: String },
    Rejected { reason: String },
    Muted { reason: String },
    Paused { reason: String },
}

struct ParsedRawSignal {
    source_code: String,
    event_kind: String,
}

impl ParsedRawSignal {
    fn parse(raw_event: &EventEnvelope) -> Result<Self, SignalHubError> {
        let parts: Vec<&str> = raw_event.event_type.split('.').collect();
        if parts.len() < 5 || parts[0] != "signal" || parts[1] != "raw" {
            return Err(SignalHubError::InvalidRawSignalEventType(
                raw_event.event_type.clone(),
            ));
        }
        if parts.last() != Some(&"observed") {
            return Err(SignalHubError::InvalidRawSignalEventType(
                raw_event.event_type.clone(),
            ));
        }

        let source_code = source_code_from_value(&raw_event.source)
            .or_else(|| parts.get(2).map(|value| (*value).to_owned()))
            .ok_or(SignalHubError::MissingSourceCode)?;
        let event_kind = parts[3..parts.len() - 1].join(".");
        if event_kind.trim().is_empty() {
            return Err(SignalHubError::InvalidRawSignalEventType(
                raw_event.event_type.clone(),
            ));
        }

        Ok(Self {
            source_code,
            event_kind,
        })
    }
}

fn source_code_from_value(value: &Value) -> Option<String> {
    value
        .get("source_code")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|source_code| !source_code.is_empty())
        .map(ToOwned::to_owned)
}

fn signal_decision_payload(
    decision: &str,
    reason: Option<&str>,
    connection_id: Option<&str>,
) -> Value {
    let mut payload = json!({
        "decision": decision,
    });

    if let Some(reason) = reason {
        payload["reason"] = json!(reason);
    }

    if let Some(connection_id) = connection_id {
        payload["connection_id"] = json!(connection_id);
    }

    payload
}

fn build_derived_event(
    raw_event: &EventEnvelope,
    event_type: &str,
    decision: Value,
) -> Result<NewEventEnvelope, SignalHubError> {
    let event_id = format!("{}_{}", event_type.replace('.', "_"), raw_event.event_id);
    let event = NewEventEnvelope::builder(
        event_id,
        event_type,
        Utc::now(),
        raw_event.source.clone(),
        raw_event.subject.clone(),
    )
    .payload(raw_event.payload.clone())
    .provenance(json!({
        "signal_hub": decision,
        "raw_event_id": raw_event.event_id,
        "raw_event_provenance": raw_event.provenance,
    }))
    .causation_id(raw_event.event_id.clone());

    let event = match &raw_event.correlation_id {
        Some(correlation_id) => event.correlation_id(correlation_id.clone()),
        None => event,
    };

    Ok(event.build()?)
}
```

### `backend/src/domains/signal_hub/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/store.rs`
- Size bytes / Размер в байтах: `95236`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;
use uuid::Uuid;

use super::fixtures::{
    SystemProfileFixture, SystemSourceFixture, system_profile_fixtures, system_source_fixtures,
};
use super::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use crate::platform::events::{EventEnvelope, EventEnvelopeError, EventStoreError};
use crate::platform::settings::SettingsError;

#[derive(Debug, Error)]
pub enum SignalHubError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Envelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Settings(#[from] SettingsError),

    #[error("invalid raw signal event type: {0}")]
    InvalidRawSignalEventType(String),

    #[error("signal source_code is missing")]
    MissingSourceCode,

    #[error("invalid signal policy scope: {0}")]
    InvalidPolicyScope(String),

    #[error("invalid signal policy mode: {0}")]
    InvalidPolicyMode(String),

    #[error("invalid signal connection id: {0}")]
    InvalidConnectionId(String),

    #[error("invalid signal connection status: {0}")]
    InvalidConnectionStatus(String),

    #[error("signal source not found: {0}")]
    SourceNotFound(String),

    #[error("signal source does not support connections: {0}")]
    SourceDoesNotSupportConnections(String),

    #[error("signal connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("invalid signal runtime state: {0}")]
    InvalidRuntimeState(String),

    #[error("invalid signal runtime id: {0}")]
    InvalidRuntimeId(String),

    #[error("invalid signal health id: {0}")]
    InvalidHealthId(String),

    #[error("invalid signal replay request: {0}")]
    InvalidReplayRequest(String),

    #[error("signal fixture not found: {0}")]
    FixtureNotFound(String),

    #[error("invalid signal fixture catalog: {0}")]
    InvalidFixtureCatalog(String),

    #[error("signal profile not found: {0}")]
    ProfileNotFound(String),

    #[error("invalid signal profile definition: {0}")]
    InvalidProfileDefinition(String),

    #[error("system signal profile is immutable: {0}")]
    SystemProfileImmutable(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl SignalHubError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::InvalidRawSignalEventType(_)
                | Self::MissingSourceCode
                | Self::InvalidPolicyScope(_)
                | Self::InvalidPolicyMode(_)
                | Self::InvalidConnectionId(_)
                | Self::InvalidConnectionStatus(_)
                | Self::InvalidRuntimeState(_)
                | Self::InvalidRuntimeId(_)
                | Self::InvalidHealthId(_)
                | Self::InvalidReplayRequest(_)
                | Self::InvalidFixtureCatalog(_)
                | Self::InvalidProfileDefinition(_)
                | Self::SystemProfileImmutable(_)
                | Self::EmptyField(_)
        )
    }

    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::SourceNotFound(_)
                | Self::ConnectionNotFound(_)
                | Self::FixtureNotFound(_)
                | Self::ProfileNotFound(_)
        )
    }

    pub fn is_failed_precondition(&self) -> bool {
        matches!(self, Self::SourceDoesNotSupportConnections(_))
    }
}

#[derive(Clone)]
pub struct SignalHubStore {
    pool: PgPool,
}

impl SignalHubStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn restore_system_sources(&self) -> Result<FixtureRestoreReport, SignalHubError> {
        let mut report = FixtureRestoreReport::default();

        for fixture in system_source_fixtures() {
            let existing = self.source_by_code(fixture.code).await?;
            match existing {
                Some(source) if source_matches_fixture(&source, fixture) => {}
                Some(source) => {
                    sqlx::query(
                        r#"
                        UPDATE signal_sources
                        SET
                            display_name = $2,
                            category = $3,
                            source_kind = $4,
                            default_enabled = $5,
                            supports_connections = $6,
                            supports_runtime = $7,
                            supports_replay = $8,
                            supports_pause = $9,
                            supports_mute = $10,
                            capability_schema_version = 1,
                            updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(source.id)
                    .bind(fixture.display_name)
                    .bind(fixture.category)
                    .bind(fixture.source_kind)
                    .bind(fixture.default_enabled)
                    .bind(fixture.supports_connections)
                    .bind(fixture.supports_runtime)
                    .bind(fixture.supports_replay)
                    .bind(fixture.supports_pause)
                    .bind(fixture.supports_mute)
                    .execute(&self.pool)
                    .await?;
                    report.sources_repaired += 1;
                }
                None => {
                    match sqlx::query(
                        r#"
                        INSERT INTO signal_sources (
                            id,
                            code,
                            display_name,
                            category,
                            source_kind,
                            default_enabled,
                            supports_connections,
                            supports_runtime,
                            supports_replay,
                            supports_pause,
                            supports_mute,
                            capability_schema_version
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1)
                        "#,
                    )
                    .bind(Uuid::now_v7())
                    .bind(fixture.code)
                    .bind(fixture.display_name)
                    .bind(fixture.category)
                    .bind(fixture.source_kind)
                    .bind(fixture.default_enabled)
                    .bind(fixture.supports_connections)
                    .bind(fixture.supports_runtime)
                    .bind(fixture.supports_replay)
                    .bind(fixture.supports_pause)
                    .bind(fixture.supports_mute)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => {
                            report.sources_created += 1;
                        }
                        Err(error) if is_unique_violation(&error) => {
                            if let Some(source) = self.source_by_code(fixture.code).await?
                                && !source_matches_fixture(&source, fixture)
                            {
                                sqlx::query(
                                    r#"
                                        UPDATE signal_sources
                                        SET
                                            display_name = $2,
                                            category = $3,
                                            source_kind = $4,
                                            default_enabled = $5,
                                            supports_connections = $6,
                                            supports_runtime = $7,
                                            supports_replay = $8,
                                            supports_pause = $9,
                                            supports_mute = $10,
                                            capability_schema_version = 1,
                                            updated_at = now()
                                        WHERE id = $1
                                        "#,
                                )
                                .bind(source.id)
                                .bind(fixture.display_name)
                                .bind(fixture.category)
                                .bind(fixture.source_kind)
                                .bind(fixture.default_enabled)
                                .bind(fixture.supports_connections)
                                .bind(fixture.supports_runtime)
                                .bind(fixture.supports_replay)
                                .bind(fixture.supports_pause)
                                .bind(fixture.supports_mute)
                                .execute(&self.pool)
                                .await?;
                                report.sources_repaired += 1;
                            }
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        }

        for fixture in system_profile_fixtures() {
            let existing = self.profile_by_code(fixture.code).await?;
            match existing {
                Some(profile) if profile_matches_fixture(&profile, fixture) => {}
                Some(profile) => {
                    sqlx::query(
                        r#"
                        UPDATE signal_profiles
                        SET
                            display_name = $2,
                            description = $3,
                            source_policies = $4,
                            is_system = $5,
                            updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(profile.id)
                    .bind(fixture.display_name)
                    .bind(fixture.description)
                    .bind(profile_policies_json(fixture)?)
                    .bind(fixture.is_system)
                    .execute(&self.pool)
                    .await?;
                    report.profiles_repaired += 1;
                }
                None => {
                    match sqlx::query(
                        r#"
                        INSERT INTO signal_profiles (
                            id,
                            code,
                            display_name,
                            description,
                            source_policies,
                            is_system
                        )
                        VALUES ($1, $2, $3, $4, $5, $6)
                        "#,
                    )
                    .bind(Uuid::now_v7())
                    .bind(fixture.code)
                    .bind(fixture.display_name)
                    .bind(fixture.description)
                    .bind(profile_policies_json(fixture)?)
                    .bind(fixture.is_system)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => {
                            report.profiles_created += 1;
                        }
                        Err(error) if is_unique_violation(&error) => {
                            if let Some(profile) = self.profile_by_code(fixture.code).await?
                                && !profile_matches_fixture(&profile, fixture)
                            {
                                sqlx::query(
                                    r#"

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/signal_hub/telegram.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/telegram.rs`
- Size bytes / Размер в байтах: `3694`
- Included characters / Включено символов: `3694`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;

use super::service::signal_hub_raw_dispatcher_allows_processing;
use super::{SignalHubError, SignalHubSignalService, SignalHubStore, SignalProcessingOutcome};
use crate::platform::communications::StoredRawCommunicationRecord;
use crate::platform::events::{EventEnvelope, EventStore, NewEventEnvelope};
use crate::platform::observations::observation_captured_event_id;

pub async fn dispatch_telegram_raw_signal(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_telegram_raw_signal(raw_record)?;
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store
        .get_by_id(&raw_signal.event_id)
        .await?
        .ok_or_else(|| SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()))?;
    let accepted_event_id = accepted_telegram_signal_event_id(&raw_event.event_id);
    if let Some(accepted_event) = event_store.get_by_id(&accepted_event_id).await? {
        return Ok(Some(accepted_event));
    }

    let signal_store = SignalHubStore::new(pool);
    if !signal_hub_raw_dispatcher_allows_processing(&signal_store).await? {
        return Ok(None);
    }

    let service = SignalHubSignalService::new(signal_store, event_store.clone());
    match service.process_raw_signal(&raw_event).await? {
        SignalProcessingOutcome::Accepted { event_id } => {
            Ok(event_store.get_by_id(&event_id).await?)
        }
        SignalProcessingOutcome::Rejected { .. }
        | SignalProcessingOutcome::Muted { .. }
        | SignalProcessingOutcome::Paused { .. } => Ok(None),
    }
}

fn build_telegram_raw_signal(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewEventEnvelope, SignalHubError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let source = json!({
        "kind": "signal_source",
        "source_code": "telegram",
        "source_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
    });
    let subject = json!({
        "kind": "communication_raw_record",
        "source_code": "telegram",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
    });
    let provenance = json!({
        "source": "communications_raw_record",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
        "import_batch_id": raw_record.import_batch_id,
        "raw_record_provenance": raw_record.provenance,
    });

    Ok(NewEventEnvelope::builder(
        telegram_raw_signal_event_id(&raw_record.raw_record_id),
        "signal.raw.telegram.message.observed",
        occurred_at,
        source,
        subject,
    )
    .payload(raw_record.payload.clone())
    .provenance(provenance)
    .causation_id(observation_captured_event_id(&raw_record.observation_id))
    .correlation_id(raw_record.observation_id.clone())
    .build()?)
}

fn telegram_raw_signal_event_id(raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!("evt_signal_raw_telegram_{:x}", hasher.finalize())
}

fn accepted_telegram_signal_event_id(raw_event_id: &str) -> String {
    format!("signal_accepted_telegram_message_{raw_event_id}")
}
```

### `backend/src/domains/signal_hub/whatsapp.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/signal_hub/whatsapp.rs`
- Size bytes / Размер в байтах: `4102`
- Included characters / Включено символов: `4102`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;

use super::service::signal_hub_raw_dispatcher_allows_processing;
use super::{SignalHubError, SignalHubSignalService, SignalHubStore, SignalProcessingOutcome};
use crate::platform::communications::StoredRawCommunicationRecord;
use crate::platform::events::{EventEnvelope, EventStore, NewEventEnvelope};
use crate::platform::observations::observation_captured_event_id;

pub async fn dispatch_whatsapp_raw_signal(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let raw_signal = build_whatsapp_raw_signal(raw_record)?;
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store
        .get_by_id(&raw_signal.event_id)
        .await?
        .ok_or_else(|| SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()))?;

    let signal_store = SignalHubStore::new(pool);
    if !signal_hub_raw_dispatcher_allows_processing(&signal_store).await? {
        return Ok(None);
    }

    let service = SignalHubSignalService::new(signal_store, event_store.clone());
    match service.process_raw_signal(&raw_event).await? {
        SignalProcessingOutcome::Accepted { event_id } => {
            Ok(event_store.get_by_id(&event_id).await?)
        }
        SignalProcessingOutcome::Rejected { .. }
        | SignalProcessingOutcome::Muted { .. }
        | SignalProcessingOutcome::Paused { .. } => Ok(None),
    }
}

fn build_whatsapp_raw_signal(
    raw_record: &StoredRawCommunicationRecord,
) -> Result<NewEventEnvelope, SignalHubError> {
    let occurred_at = raw_record.occurred_at.unwrap_or(raw_record.captured_at);
    let source = json!({
        "kind": "signal_source",
        "source_code": "whatsapp",
        "source_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
    });
    let subject = json!({
        "kind": "communication_raw_record",
        "source_code": "whatsapp",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
    });
    let provenance = json!({
        "source": "communications_raw_record",
        "raw_record_id": raw_record.raw_record_id,
        "account_id": raw_record.account_id,
        "provider_record_id": raw_record.provider_record_id,
        "record_kind": raw_record.record_kind,
        "import_batch_id": raw_record.import_batch_id,
        "raw_record_provenance": raw_record.provenance,
    });

    let event_kind = match raw_record.record_kind.as_str() {
        "whatsapp_web_reaction" => "reaction",
        "whatsapp_web_media" => "media",
        "whatsapp_web_status" => "status",
        "whatsapp_web_status_view" => "status_view",
        "whatsapp_web_status_delete" => "status_delete",
        "whatsapp_web_presence" => "presence",
        "whatsapp_web_call" => "call_metadata",
        "whatsapp_web_runtime_event" => "runtime_event",
        "whatsapp_web_dialog" => "dialog",
        "whatsapp_web_participant" => "participant",
        "whatsapp_web_message_update" => "message_update",
        "whatsapp_web_message_delete" => "message_delete",
        "whatsapp_web_receipt" => "receipt",
        _ => "message",
    };

    Ok(NewEventEnvelope::builder(
        whatsapp_raw_signal_event_id(&raw_record.raw_record_id),
        format!("signal.raw.whatsapp.{event_kind}.observed"),
        occurred_at,
        source,
        subject,
    )
    .payload(raw_record.payload.clone())
    .provenance(provenance)
    .causation_id(observation_captured_event_id(&raw_record.observation_id))
    .correlation_id(raw_record.observation_id.clone())
    .build()?)
}

fn whatsapp_raw_signal_event_id(raw_record_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_record_id.as_bytes());
    format!("evt_signal_raw_whatsapp_{:x}", hasher.finalize())
}
```

### `backend/src/domains/tasks/api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/api.rs`
- Size bytes / Размер в байтах: `20728`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use super::core::{TaskCoreError, materialize_task_observation_link_in_transaction};
use super::service::TaskCommandService;
use crate::platform::observations::ObservationStoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub task_candidate_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub provenance_kind: String,
    pub provenance_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub source_type: String,
    pub project_id: Option<String>,
    pub status: String,
    pub makosh_status: String,
    pub priority_score: Option<f64>,
    pub risk_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub area: Option<String>,
    pub why: Option<String>,
    pub outcome: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub waiting_reason: Option<String>,
    pub energy_type: Option<String>,
    pub confidentiality: String,
    pub tags: Value,
    pub task_metadata: Value,
    pub linked_person_id: Option<String>,
    pub linked_organization_id: Option<String>,
    pub created_from_event_id: Option<String>,
    pub created_by_actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type TaskCommandPort = TaskStore;

#[derive(Clone)]
pub struct TaskStore {
    pool: PgPool,
}

impl TaskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &NewTask) -> Result<Task, TaskError> {
        TaskCommandService::new(self.pool.clone())
            .create_task_manual(req)
            .await
            .map_err(|error| match error {
                super::service::TaskCommandServiceError::ObservationCapture { source, .. } => {
                    TaskError::from(source)
                }
                super::service::TaskCommandServiceError::Task(inner) => inner,
                super::service::TaskCommandServiceError::Core(inner) => TaskError::from(inner),
                super::service::TaskCommandServiceError::Sqlx(inner) => TaskError::from(inner),
                super::service::TaskCommandServiceError::ObservationStore(inner) => {
                    TaskError::from(inner)
                }
                super::service::TaskCommandServiceError::MissingEvidenceSourceId => {
                    TaskError::MissingSourceIdentifier
                }
            })
    }

    pub(crate) async fn create_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        req: &NewTask,
    ) -> Result<Task, TaskError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let task_id = format!("task:v1:{ts:x}");
        let tags = req.tags.clone().unwrap_or_else(|| json!([]));
        let row = sqlx::query(
            "INSERT INTO tasks (task_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, makosh_status, priority_score, area, why, due_at, energy_type, confidentiality, tags, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
             RETURNING task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        ).bind(&task_id).bind(&req.title).bind(req.description.as_deref())
         .bind(req.provenance_kind.as_deref().ok_or(TaskError::MissingProvenance)?)
         .bind(req.provenance_id.as_deref().ok_or(TaskError::MissingProvenance)?)
         .bind(req.source_kind.as_deref().unwrap_or("manual")).bind(req.source_id.as_deref().unwrap_or("manual"))
         .bind(req.source_type.as_deref().unwrap_or("manual")).bind(req.project_id.as_deref())
         .bind(req.makosh_status.as_deref().unwrap_or("new")).bind(req.priority_score)
         .bind(req.area.as_deref()).bind(req.why.as_deref()).bind(req.due_at)
         .bind(req.energy_type.as_deref()).bind(req.confidentiality.as_deref().unwrap_or("private_local"))
         .bind(&tags).bind(req.linked_person_id.as_deref()).bind(req.linked_organization_id.as_deref())
         .bind(req.created_from_event_id.as_deref()).bind(req.created_by_actor_id.as_deref())
         .fetch_one(&mut **transaction).await?;
        let task = row_to_task(row)?;

        if let Some(observation_id) = req
            .source_id
            .as_deref()
            .filter(|_| req.source_kind.as_deref() == Some("observation"))
        {
            materialize_task_observation_link_in_transaction(
                transaction,
                Some(observation_id),
                Some("task_create"),
                &task.task_id,
                Some(json!({
                    "source_kind": req.source_kind,
                    "source_type": req.source_type,
                    "provenance_kind": req.provenance_kind,
                    "provenance_id": req.provenance_id,
                })),
            )
            .await?;
        }

        Ok(task)
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<Task>, TaskError> {
        let row = sqlx::query("SELECT task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE task_id=$1")
            .bind(task_id).fetch_optional(&self.pool).await?;
        row.map(|r| row_to_task(r).map_err(TaskError::from))
            .transpose()
    }

    pub async fn list(&self, query: &TaskListQuery) -> Result<Vec<Task>, TaskError> {
        let limit = query.limit.unwrap_or(100).clamp(1, 500);
        let rows = sqlx::query(
            "SELECT task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE ($1::text IS NULL OR makosh_status=$1) AND ($2::text IS NULL OR project_id=$2) AND ($3::text IS NULL OR source_type=$3) ORDER BY COALESCE(priority_score,0) DESC, due_at ASC NULLS LAST, created_at DESC LIMIT $4"
        ).bind(query.status.as_deref()).bind(query.project_id.as_deref()).bind(query.source_type.as_deref()).bind(limit)
         .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(TaskError::from)
    }

    pub async fn update(&self, task_id: &str, update: &TaskUpdate) -> Result<Task, TaskError> {
        self.update_internal(task_id, update, None, None, None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        task_id: &str,
        update: &TaskUpdate,
        observation_id: &str,
        relationship_kind: &str,
        metadata: Value,
    ) -> Result<Task, TaskError> {
        self.update_internal(
            task_id,
            update,
            Some(observation_id),
            Some(relationship_kind),
            Some(metadata),
        )
        .await
    }

    async fn update_internal(
        &self,
        task_id: &str,
        update: &TaskUpdate,
        observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Task, TaskError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE tasks SET title=COALESCE($2,title), description=COALESCE($3,description), makosh_status=COALESCE($4,makosh_status), priority_score=COALESCE($5,priority_score), risk_score=COALESCE($6,risk_score), readiness_score=COALESCE($7,readiness_score), area=COALESCE($8,area), why=COALESCE($9,why), outcome=COALESCE($10,outcome), due_at=COALESCE($11,due_at), waiting_reason=COALESCE($12,waiting_reason), energy_type=COALESCE($13,energy_type), confidentiality=COALESCE($14,confidentiality), tags=COALESCE($15,tags), task_metadata=COALESCE($16,task_metadata), linked_person_id=COALESCE($17,linked_person_id), linked_organization_id=COALESCE($18,linked_organization_id), completed_at=COALESCE($19,completed_at), updated_at=now() WHERE task_id=$1 RETURNING task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        ).bind(task_id).bind(update.title.as_deref()).bind(update.description.as_deref())
         .bind(update.makosh_status.as_deref()).bind(update.priority_score).bind(update.risk_score).bind(update.readiness_score)
         .bind(update.area.as_deref()).bind(update.why.as_deref()).bind(update.outcome.as_deref())
         .bind(update.due_at).bind(update.waiting_reason.as_deref()).bind(update.energy_type.as_deref())
         .bind(update.confidentiality.as_deref()).bind(update.tags.as_ref()).bind(update.task_metadata.as_ref())
         .bind(update.linked_person_id.as_deref()).bind(update.linked_organization_id.as_deref())
         .bind(update.completed_at)
         .fetch_one(&mut *transaction).await?;
        let task = row_to_task(row)?;

        materialize_task_observation_link_in_transaction(
            &mut transaction,
            observation_id,
            relationship_kind,
            task_id,
            metadata,
        )
        .await?;
        transaction.commit().await?;

        Ok(task)
    }

    pub async fn set_status(&self, task_id: &str, status: &str) -> Result<Task, TaskError> {
        self.set_status_internal(task_id, status, None, None, None)
            .await
    }

    pub async fn set_status_with_observation(
        &self,
        task_id: &str,
        status: &str,
        observation_id: &str,
        relationship_kind: &str,
        metadata: Value,
    ) -> Result<Task, TaskError> {
        self.set_status_internal(
            task_id,
            status,
            Some(observation_id),
            Some(relationship_kind),
            Some(metadata),
        )
        .await
    }

    async fn set_status_internal(
        &self,
        task_id: &str,
        status: &str,
        observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Task, T
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/tasks/brain.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/brain.rs`
- Size bytes / Размер в байтах: `4716`
- Included characters / Включено символов: `4716`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::tasks::core::TaskContextPackStore;
use crate::domains::tasks::core::TaskCoreError;

pub struct TaskBrainService;

impl TaskBrainService {
    pub async fn explain_task(pool: &PgPool, task_id: &str) -> Result<Value, TaskBrainError> {
        let task = sqlx::query("SELECT task_id, title, description, source_type, makosh_status, why, outcome, due_at FROM tasks WHERE task_id=$1")
            .bind(task_id).fetch_optional(pool).await?;
        let task = task.ok_or(TaskBrainError::NotFound)?;

        let ctx = TaskContextPackStore::new(pool.clone()).get(task_id).await?;

        let evidence =
            sqlx::query("SELECT source_type, quote FROM task_evidence WHERE task_id=$1 LIMIT 5")
                .bind(task_id)
                .fetch_all(pool)
                .await?;

        Ok(json!({
            "task_id": task.try_get::<String,_>("task_id").unwrap_or_default(),
            "title": task.try_get::<String,_>("title").unwrap_or_default(),
            "description": task.try_get::<Option<String>,_>("description").unwrap_or(None),
            "what": format!("Task: {}", task.try_get::<String,_>("title").unwrap_or_default()),
            "why": task.try_get::<Option<String>,_>("why").unwrap_or(Some("No reason recorded".into())),
            "status": task.try_get::<String,_>("makosh_status").unwrap_or_default(),
            "source": task.try_get::<String,_>("source_type").unwrap_or_default(),
            "context": ctx.map(|r| json!({
                "summary": r.summary,
                "blockers": r.blockers,
                "risks": r.risks,
                "next_action": r.suggested_next_action,
            })),
            "evidence": evidence.iter().map(|r| json!({
                "source": r.try_get::<String,_>("source_type").unwrap_or_default(),
                "quote": r.try_get::<Option<String>,_>("quote").unwrap_or(None),
            })).collect::<Vec<_>>(),
        }))
    }

    pub async fn search_tasks(pool: &PgPool, query: &str) -> Result<Value, TaskBrainError> {
        let pattern = format!("%{query}%");
        let rows = sqlx::query("SELECT task_id, title, makosh_status, priority_score, due_at FROM tasks WHERE title ILIKE $1 OR description ILIKE $1 ORDER BY COALESCE(priority_score,0) DESC LIMIT 20")
            .bind(&pattern).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                    "title": r.try_get::<String,_>("title").unwrap_or_default(),
                    "status": r.try_get::<String,_>("makosh_status").unwrap_or_default(),
                    "priority": r.try_get::<Option<f64>,_>("priority_score").unwrap_or(None),
                    "due_at": r.try_get::<Option<DateTime<Utc>>,_>("due_at").unwrap_or(None),
                })
            })
            .collect();
        Ok(json!({"query": query, "results": items}))
    }

    pub async fn daily_brief(pool: &PgPool) -> Result<Value, TaskBrainError> {
        let now = Utc::now();
        let _today_end = now
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .map(|d| DateTime::from_naive_utc_and_offset(d, Utc))
            .unwrap_or(now);

        let active = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE makosh_status IN ('new','triaged','ready','in_progress','waiting','blocked','review')")
            .fetch_one(pool).await?;
        let overdue = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE due_at < $1 AND makosh_status NOT IN ('done','cancelled','archived')")
            .bind(now).fetch_one(pool).await?;
        let high_risk = sqlx::query("SELECT task_id, title FROM tasks WHERE risk_score > 0.7 AND makosh_status NOT IN ('done','cancelled','archived') ORDER BY risk_score DESC LIMIT 5")
            .fetch_all(pool).await?;

        Ok(json!({
            "active_tasks": active.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
            "overdue": overdue.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
            "high_risk": high_risk.iter().map(|r| json!({
                "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                "title": r.try_get::<String,_>("title").unwrap_or_default(),
            })).collect::<Vec<_>>(),
        }))
    }
}

#[derive(Debug, Error)]
pub enum TaskBrainError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    TaskCore(#[from] TaskCoreError),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/tasks/candidates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates.rs`
- Size bytes / Размер в байтах: `604`
- Included characters / Включено символов: `604`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod errors;
mod events;
mod extraction;
mod ids;
mod models;
mod persistence;
mod service;
mod store;
mod validation;

pub use errors::TaskCandidateError;
pub(crate) use ids::task_id_from_candidate;
pub(crate) use models::StoredCandidateRow;
pub use models::{
    TaskCandidate, TaskCandidateKind, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateReviewState, TaskCandidateSourceKind,
};
pub use service::{TaskCandidateReviewService, TaskCandidateReviewServiceError};
pub use store::TaskCandidateStore;
pub use store::TaskCandidateStore as TaskCandidatePort;
```

### `backend/src/domains/tasks/candidates/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/constants.rs`
- Size bytes / Размер в байтах: `1166`
- Included characters / Включено символов: `1166`
- Truncated / Обрезано: `no`

```rust
pub(crate) const TASK_CANDIDATE_REVIEW_EVENT_TYPE: &str = "task_candidate.review_state_changed";
pub(crate) const TASK_CANDIDATE_REVIEW_SOURCE_KIND: &str = "task_candidate_review";
pub(crate) const TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER: &str = "local_api";
pub(crate) const TASK_CANDIDATE_ID_PREFIX: &str = "task_candidate:v1:";
pub(crate) const TASK_ID_PREFIX: &str = "task:v1:";
pub(crate) const TASK_CANDIDATE_EVENT_PREFIX: &str = "task_candidate_review:";
pub(crate) const TASK_CANDIDATE_KIND_TASK: &str = "task";
pub(crate) const TASK_CANDIDATE_KIND_OBLIGATION_TASK: &str = "obligation_task";
pub(crate) const OBLIGATION_TASK_LINK_KIND: &str = "fulfillment_task";
pub(crate) const OBLIGATION_CANDIDATE_METADATA_KEY: &str = "obligation_candidate";
pub(crate) const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
pub(crate) const FNV_PRIME: u64 = 0x100000001b3;
pub(crate) const DEFAULT_LIMIT: i64 = 50;
pub(crate) const MAX_LIMIT: i64 = 100;
pub(crate) const MIN_LIMIT: i64 = 1;
pub(crate) const REVIEW_TEXT_SNIPPET_CHARS: usize = 180;
pub(crate) const TITLE_PREVIEW_CHARS: usize = 120;
pub(crate) const OWNER_PERSONA_EXTRACTION_CONTEXT_ID: &str = "persona:owner";
```

### `backend/src/domains/tasks/candidates/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/errors.rs`
- Size bytes / Размер в байтах: `1832`
- Included characters / Включено символов: `1832`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::obligations::ObligationStoreError;
use crate::domains::tasks::core::TaskCoreError;
use crate::engines::obligation::ObligationEngineError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum TaskCandidateError {
    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("task candidate was not found")]
    TaskCandidateNotFound,

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("payload must be an object")]
    InvalidPayload(String),

    #[error("payload field was missing: {0}")]
    MissingPayloadField(String),

    #[error("candidate metadata is missing or invalid: {0}")]
    InvalidCandidateMetadata(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error("invalid task candidate source kind: {0}")]
    InvalidSourceKind(String),

    #[error("task candidate requires observation evidence: {0}")]
    ObservationRequired(String),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    ObligationEngine(#[from] ObligationEngineError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    ObligationStore(#[from] ObligationStoreError),

    #[error(transparent)]
    TaskCore(#[from] TaskCoreError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/domains/tasks/candidates/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/events.rs`
- Size bytes / Размер в байтах: `2784`
- Included characters / Включено символов: `2784`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use crate::platform::events::NewEventEnvelope;

use super::constants::{
    TASK_CANDIDATE_REVIEW_EVENT_TYPE, TASK_CANDIDATE_REVIEW_SOURCE_KIND,
    TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER,
};
use super::errors::TaskCandidateError;
use super::models::TaskCandidateReviewState;
use super::validation::validate_non_empty;

pub(crate) struct ReviewCommandEvent {
    pub(crate) command_id: String,
    pub(crate) task_candidate_id: String,
    pub(crate) review_state: TaskCandidateReviewState,
    pub(crate) actor_id: String,
    pub(crate) event_id: String,
    pub(crate) occurred_at: DateTime<Utc>,
}

impl ReviewCommandEvent {
    pub(crate) fn into_event(self) -> Result<NewEventEnvelope, TaskCandidateError> {
        let event_id = self.event_id.clone();
        Ok(NewEventEnvelope::builder(
            event_id,
            TASK_CANDIDATE_REVIEW_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": TASK_CANDIDATE_REVIEW_SOURCE_KIND,
                "provider": TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "task_candidate_review",
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "task_candidate_id": self.task_candidate_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct ReviewEventPayload {
    pub(crate) task_candidate_id: String,
    pub(crate) review_state: TaskCandidateReviewState,
}

impl ReviewEventPayload {
    pub(crate) fn from_payload(payload: &Value) -> Result<Self, TaskCandidateError> {
        let payload = as_object(payload)?;
        Ok(Self {
            task_candidate_id: required_payload_string(payload, "task_candidate_id")?,
            review_state: TaskCandidateReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, TaskCandidateError> {
    value
        .as_object()
        .ok_or_else(|| TaskCandidateError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, TaskCandidateError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| TaskCandidateError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| TaskCandidateError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}
```

### `backend/src/domains/tasks/candidates/extraction.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/extraction.rs`
- Size bytes / Размер в байтах: `3128`
- Included characters / Включено символов: `3128`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::engines::obligation::ObligationCandidate;

use super::constants::{
    OBLIGATION_CANDIDATE_METADATA_KEY, REVIEW_TEXT_SNIPPET_CHARS, TITLE_PREVIEW_CHARS,
};
use super::errors::TaskCandidateError;
use super::models::{CandidatePayload, TaskCandidateKind, TaskCandidateSourceKind};
use super::validation::text_preview;

pub(crate) struct CandidateFragment {
    pub(crate) text: String,
    pub(crate) due_text: Option<String>,
    pub(crate) assignee_label: Option<String>,
}

pub(crate) fn extract_candidate_fragment(text: &str) -> Option<CandidateFragment> {
    let text_lower = text.to_lowercase();
    if !(text_lower.contains("action:")
        || text_lower.contains("please ")
        || text_lower.contains("follow up")
        || text_lower.contains("next step"))
    {
        return None;
    }

    let selected = text
        .lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_lowercase();
            lower.contains("action:")
                || lower.contains("please ")
                || lower.contains("follow up")
                || lower.contains("next step")
        })
        .unwrap_or(text);

    let due_text = text.lines().find_map(parse_due_text);
    let assignee_label = text.lines().find_map(parse_assignee_label);

    Some(CandidateFragment {
        text: selected.to_owned(),
        due_text,
        assignee_label,
    })
}

pub(crate) fn parse_due_text(line: &str) -> Option<String> {
    let normalized = line.trim().to_lowercase();
    if !normalized.starts_with("due") {
        return None;
    }

    normalized.split_once(':').and_then(|(_, right)| {
        let due = right.trim();
        (!due.is_empty()).then_some(due.to_owned())
    })
}

pub(crate) fn parse_assignee_label(line: &str) -> Option<String> {
    let normalized = line.to_lowercase();
    if !normalized.starts_with("assignee") {
        return None;
    }

    normalized.split_once(':').and_then(|(_, right)| {
        let assignee = right.trim();
        (!assignee.is_empty()).then_some(assignee.to_owned())
    })
}

pub(crate) fn title_from_fragment(value: &str) -> String {
    text_preview(value, TITLE_PREVIEW_CHARS)
}

pub(crate) fn evidence_excerpt(value: &str) -> String {
    text_preview(value, REVIEW_TEXT_SNIPPET_CHARS)
}

pub(crate) fn task_candidate_payload_from_obligation(
    observation_id: String,
    candidate: &ObligationCandidate,
) -> CandidatePayload {
    CandidatePayload {
        source_kind: TaskCandidateSourceKind::Observation,
        source_id: observation_id.clone(),
        observation_id: Some(observation_id),
        candidate_kind: TaskCandidateKind::ObligationTask,
        candidate_metadata: json!({
            "engine": "obligation",
            OBLIGATION_CANDIDATE_METADATA_KEY: candidate,
        }),
        project_id: None,
        title: title_from_fragment(&candidate.statement),
        due_text: candidate.due_text.clone(),
        assignee_label: None,
        confidence: (candidate.confidence - 0.08).max(0.0),
        evidence_excerpt: evidence_excerpt(&candidate.quote),
    }
}
```

### `backend/src/domains/tasks/candidates/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/ids.rs`
- Size bytes / Размер в байтах: `710`
- Included characters / Включено символов: `710`
- Truncated / Обрезано: `no`

```rust
use super::constants::{FNV_OFFSET_BASIS, FNV_PRIME, TASK_CANDIDATE_ID_PREFIX, TASK_ID_PREFIX};

pub(crate) fn task_candidate_id_from_source(
    source_kind: &str,
    source_id: &str,
    title: &str,
) -> String {
    let title_hash = fnv1a64_hex(title);
    format!("{TASK_CANDIDATE_ID_PREFIX}{source_kind}:{source_id}:{title_hash}")
}

pub(crate) fn task_id_from_candidate(task_candidate_id: &str) -> String {
    format!("{TASK_ID_PREFIX}{}", fnv1a64_hex(task_candidate_id))
}

fn fnv1a64_hex(value: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{hash:016x}")
}
```

### `backend/src/domains/tasks/candidates/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/models.rs`
- Size bytes / Размер в байтах: `3938`
- Included characters / Включено символов: `3938`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::constants::{TASK_CANDIDATE_KIND_OBLIGATION_TASK, TASK_CANDIDATE_KIND_TASK};
use super::errors::TaskCandidateError;
use super::ids::task_candidate_id_from_source;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateSourceKind {
    Observation,
}

impl TaskCandidateSourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observation => "observation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateKind {
    Task,
    ObligationTask,
}

impl TaskCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => TASK_CANDIDATE_KIND_TASK,
            Self::ObligationTask => TASK_CANDIDATE_KIND_OBLIGATION_TASK,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskCandidateReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl TaskCandidateReviewState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub(crate) fn parse(value: impl AsRef<str>) -> Result<Self, TaskCandidateError> {
        match value.as_ref() {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(TaskCandidateError::InvalidReviewState(
                value.as_ref().to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCandidateReviewCommand {
    pub command_id: String,
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskCandidateReviewCommandResult {
    pub task_candidate_id: String,
    pub review_state: TaskCandidateReviewState,
    pub event_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TaskCandidate {
    pub task_candidate_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub project_id: Option<String>,
    pub title: String,
    pub due_text: Option<String>,
    pub assignee_label: Option<String>,
    pub confidence: f64,
    pub review_state: String,
    pub evidence_excerpt: String,
    pub generated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct CandidatePayload {
    pub(crate) source_kind: TaskCandidateSourceKind,
    pub(crate) source_id: String,
    pub(crate) observation_id: Option<String>,
    pub(crate) candidate_kind: TaskCandidateKind,
    pub(crate) candidate_metadata: Value,
    pub(crate) project_id: Option<String>,
    pub(crate) title: String,
    pub(crate) due_text: Option<String>,
    pub(crate) assignee_label: Option<String>,
    pub(crate) confidence: f64,
    pub(crate) evidence_excerpt: String,
}

impl CandidatePayload {
    pub(crate) fn task_candidate_id(&self) -> String {
        let source_id = self
            .observation_id
            .as_deref()
            .unwrap_or(self.source_id.as_str());
        task_candidate_id_from_source(self.source_kind.as_str(), source_id, &self.title)
    }
}

#[derive(Debug)]
pub(crate) struct StoredCandidateRow {
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    pub(crate) observation_id: Option<String>,
    pub(crate) candidate_kind: String,
    pub(crate) candidate_metadata: Value,
    pub(crate) project_id: Option<String>,
    pub(crate) title: String,
    pub(crate) due_text: Option<String>,
    pub(crate) assignee_label: Option<String>,
    pub(crate) confidence: f64,
    pub(crate) evidence_excerpt: String,
}
```

### `backend/src/domains/tasks/candidates/persistence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/persistence.rs`
- Size bytes / Размер в байтах: `7316`
- Included characters / Включено символов: `7316`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::{PgPool, PgRow, Postgres};
use sqlx::{Row, Transaction};

use super::errors::TaskCandidateError;
use super::models::{
    CandidatePayload, StoredCandidateRow, TaskCandidate, TaskCandidateReviewState,
};

pub(crate) async fn upsert_task_candidate(
    pool: &PgPool,
    payload: &CandidatePayload,
    task_candidate_id: String,
    review_state: TaskCandidateReviewState,
) -> Result<(), TaskCandidateError> {
    let update_result = sqlx::query(
        r#"
        UPDATE task_candidates
        SET
            source_kind = $2,
            source_id = $3,
            observation_id = $4,
            candidate_kind = $5,
            candidate_metadata = $6,
            project_id = COALESCE($7, project_id),
            title = $8,
            due_text = COALESCE($9, due_text),
            assignee_label = COALESCE($10, assignee_label),
            confidence = $11,
            review_state = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN review_state
                ELSE $12
            END,
            evidence_excerpt = $13,
            event_id = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        WHERE task_candidate_id = $1
           OR (source_kind = $2 AND source_id = $3 AND lower(title) = lower($8))
        "#,
    )
    .bind(&task_candidate_id)
    .bind(payload.source_kind.as_str())
    .bind(&payload.source_id)
    .bind(payload.observation_id.as_deref())
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.candidate_metadata)
    .bind(&payload.project_id)
    .bind(&payload.title)
    .bind(&payload.due_text)
    .bind(&payload.assignee_label)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .bind(&payload.evidence_excerpt)
    .execute(pool)
    .await?;

    if update_result.rows_affected() > 0 {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO task_candidates (
            task_candidate_id,
            source_kind,
            source_id,
            observation_id,
            candidate_kind,
            candidate_metadata,
            project_id,
            title,
            due_text,
            assignee_label,
            confidence,
            review_state,
            evidence_excerpt,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NULL, NULL, NULL)
        ON CONFLICT (source_kind, source_id, lower(title))
        DO UPDATE SET
            source_kind = EXCLUDED.source_kind,
            source_id = EXCLUDED.source_id,
            observation_id = EXCLUDED.observation_id,
            candidate_kind = EXCLUDED.candidate_kind,
            candidate_metadata = EXCLUDED.candidate_metadata,
            project_id = COALESCE(EXCLUDED.project_id, task_candidates.project_id),
            title = EXCLUDED.title,
            due_text = COALESCE(EXCLUDED.due_text, task_candidates.due_text),
            assignee_label = COALESCE(EXCLUDED.assignee_label, task_candidates.assignee_label),
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            evidence_excerpt = EXCLUDED.evidence_excerpt,
            event_id = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.actor_id
                ELSE NULL
                END,
            reviewed_at = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(task_candidate_id)
    .bind(payload.source_kind.as_str())
    .bind(&payload.source_id)
    .bind(payload.observation_id.as_deref())
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.candidate_metadata)
    .bind(&payload.project_id)
    .bind(&payload.title)
    .bind(&payload.due_text)
    .bind(&payload.assignee_label)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .bind(&payload.evidence_excerpt)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn row_task_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
) -> Result<StoredCandidateRow, TaskCandidateError> {
    let row = sqlx::query(
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
            confidence,
            evidence_excerpt
        FROM task_candidates
        WHERE task_candidate_id = $1
        FOR UPDATE
        "#,
    )
    .bind(task_candidate_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(TaskCandidateError::TaskCandidateNotFound)?;

    Ok(StoredCandidateRow {
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        observation_id: row.try_get("observation_id")?,
        candidate_kind: row.try_get("candidate_kind")?,
        candidate_metadata: row.try_get("candidate_metadata")?,
        project_id: row.try_get("project_id")?,
        title: row.try_get("title")?,
        due_text: row.try_get("due_text")?,
        assignee_label: row.try_get("assignee_label")?,
        confidence: row.try_get("confidence")?,
        evidence_excerpt: row.try_get("evidence_excerpt")?,
    })
}

pub(crate) fn row_to_task_candidate(row: PgRow) -> Result<TaskCandidate, TaskCandidateError> {
    Ok(TaskCandidate {
        task_candidate_id: row.try_get("task_candidate_id")?,
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        observation_id: row.try_get("observation_id")?,
        project_id: row.try_get("project_id")?,
        title: row.try_get("title")?,
        due_text: row.try_get("due_text")?,
        assignee_label: row.try_get("assignee_label")?,
        confidence: row.try_get("confidence")?,
        review_state: row.try_get::<String, _>("review_state")?,
        evidence_excerpt: row.try_get("evidence_excerpt")?,
        generated_at: row.try_get("generated_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/tasks/candidates/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/service.rs`
- Size bytes / Размер в байтах: `2349`
- Included characters / Включено символов: `2349`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{
    TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateStore,
};

#[derive(Clone)]
pub struct TaskCandidateReviewService {
    pool: PgPool,
}

impl TaskCandidateReviewService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateReviewServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "task_candidate_id": command.task_candidate_id,
                        "command_id": command.command_id,
                        "review_state": command.review_state.as_str(),
                        "actor_id": command.actor_id,
                        "operation": "task_candidate_review",
                    }),
                    format!(
                        "task-candidate://{}/review/{}",
                        command.task_candidate_id, command.command_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "tasks.candidates_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(TaskCandidateStore::new(self.pool.clone())
            .set_review_state_with_observation(
                command,
                &observation.observation_id,
                json!({
                    "captured_by": "tasks.candidates_service.review_manual",
                    "operation": "review_manual",
                }),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum TaskCandidateReviewServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),
}
```

### `backend/src/domains/tasks/candidates/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/store.rs`
- Size bytes / Размер в байтах: `1958`
- Included characters / Включено символов: `1958`
- Truncated / Обрезано: `no`

```rust
mod list;
mod refresh;
mod review;
mod task_activation;

use sqlx::postgres::PgPool;

use super::errors::TaskCandidateError;
use super::models::{TaskCandidate, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult};
use crate::platform::events::EventEnvelope;
use serde_json::Value;

#[derive(Clone)]
pub struct TaskCandidateStore {
    pool: PgPool,
}

impl TaskCandidateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_deterministic_candidates(
        &self,
        limit: i64,
    ) -> Result<usize, TaskCandidateError> {
        refresh::refresh_deterministic_candidates(&self.pool, limit).await
    }

    pub async fn set_review_state(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
        review::set_review_state(&self.pool, command).await
    }

    pub async fn set_review_state_with_observation(
        &self,
        command: &TaskCandidateReviewCommand,
        observation_id: &str,
        metadata: Value,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
        review::set_review_state_with_observation(
            &self.pool,
            command,
            Some(observation_id),
            Some(metadata),
        )
        .await
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), TaskCandidateError> {
        review::apply_review_event(&self.pool, event).await
    }

    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<TaskCandidate>, TaskCandidateError> {
        list::list_candidates(&self.pool, limit).await
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, TaskCandidateError> {
        refresh::refresh_message_candidates_for_ids(&self.pool, message_ids).await
    }
}
```

### `backend/src/domains/tasks/candidates/store/list.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/store/list.rs`
- Size bytes / Размер в байтах: `1047`
- Included characters / Включено символов: `1047`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::super::errors::TaskCandidateError;
use super::super::models::TaskCandidate;
use super::super::persistence::row_to_task_candidate;
use super::super::validation::validate_optional_limit;

pub(super) async fn list_candidates(
    pool: &PgPool,
    limit: Option<i64>,
) -> Result<Vec<TaskCandidate>, TaskCandidateError> {
    let limit = validate_optional_limit(limit)?;

    let rows = sqlx::query(
        r#"
        SELECT
            task_candidate_id,
            source_kind,
            source_id,
            observation_id,
            project_id,
            title,
            due_text,
            assignee_label,
            confidence,
            review_state,
            evidence_excerpt,
            generated_at,
            reviewed_at,
            updated_at
        FROM task_candidates
        ORDER BY updated_at DESC, task_candidate_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_task_candidate).collect()
}
```

### `backend/src/domains/tasks/candidates/store/refresh.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/store/refresh.rs`
- Size bytes / Размер в байтах: `7666`
- Included characters / Включено символов: `7666`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::engines::obligation::{
    ObligationEngine, ObligationEntityKind, ObligationExtractionInput,
};

use super::super::constants::OWNER_PERSONA_EXTRACTION_CONTEXT_ID;
use super::super::errors::TaskCandidateError;
use super::super::extraction::{
    evidence_excerpt, extract_candidate_fragment, task_candidate_payload_from_obligation,
    title_from_fragment,
};
use super::super::models::{
    CandidatePayload, TaskCandidateKind, TaskCandidateReviewState, TaskCandidateSourceKind,
};
use super::super::persistence::upsert_task_candidate;
use super::super::validation::validate_limit;

pub(super) async fn refresh_deterministic_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, TaskCandidateError> {
    let limit = validate_limit(limit)?;

    let message_count = refresh_message_candidates(pool, limit).await?;
    let document_count = refresh_document_candidates(pool, limit).await?;

    Ok(message_count + document_count)
}

async fn refresh_message_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, TaskCandidateError> {
    let rows = sqlx::query(
        r#"
        SELECT
            message_id,
            observation_id,
            subject,
            body_text
        FROM communication_messages
        ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let source_id = row.try_get::<String, _>("message_id")?;
        let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
        let source_text = format!(
            "{}\n{}",
            row.try_get::<String, _>("subject")?,
            row.try_get::<String, _>("body_text")?,
        );

        count +=
            refresh_message_candidate_from_text(pool, &source_id, &observation_id, &source_text)
                .await?;
    }

    Ok(count)
}

pub(super) async fn refresh_message_candidates_for_ids(
    pool: &PgPool,
    message_ids: &[String],
) -> Result<usize, TaskCandidateError> {
    if message_ids.is_empty() {
        return Ok(0);
    }

    let rows = sqlx::query(
        r#"
        SELECT
            message_id,
            observation_id,
            subject,
            body_text
        FROM communication_messages
        WHERE message_id = ANY($1)
        ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
        "#,
    )
    .bind(message_ids.to_vec())
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let source_id = row.try_get::<String, _>("message_id")?;
        let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
        let source_text = format!(
            "{}\n{}",
            row.try_get::<String, _>("subject")?,
            row.try_get::<String, _>("body_text")?,
        );
        count +=
            refresh_message_candidate_from_text(pool, &source_id, &observation_id, &source_text)
                .await?;
    }

    Ok(count)
}

async fn refresh_message_candidate_from_text(
    pool: &PgPool,
    source_id: &str,
    observation_id: &Option<String>,
    source_text: &str,
) -> Result<usize, TaskCandidateError> {
    let observation_id = observation_id
        .clone()
        .ok_or_else(|| TaskCandidateError::ObservationRequired(source_id.to_owned()))?;

    if let Some(fragment) = extract_candidate_fragment(source_text) {
        let payload = CandidatePayload {
            source_kind: TaskCandidateSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id.clone()),
            candidate_kind: TaskCandidateKind::Task,
            candidate_metadata: json!({}),
            project_id: None,
            title: title_from_fragment(&fragment.text),
            due_text: fragment.due_text,
            assignee_label: fragment.assignee_label,
            confidence: 0.8,
            evidence_excerpt: evidence_excerpt(&fragment.text),
        };
        upsert_suggested_candidate(pool, &payload).await?;
        return Ok(1);
    }

    let input = ObligationExtractionInput::communication(
        source_id,
        source_text,
        ObligationEntityKind::Persona,
        OWNER_PERSONA_EXTRACTION_CONTEXT_ID,
    );
    let extraction = ObligationEngine::detect_candidates(&input)?;

    let mut count = 0usize;
    for obligation_candidate in extraction.obligations {
        let payload =
            task_candidate_payload_from_obligation(observation_id.clone(), &obligation_candidate);
        upsert_suggested_candidate(pool, &payload).await?;
        count += 1;
    }

    Ok(count)
}

async fn refresh_document_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, TaskCandidateError> {
    let rows = sqlx::query(
        r#"
            SELECT
                document_id,
                observation_id,
                title,
                extracted_text
            FROM documents
            ORDER BY imported_at DESC, document_id
            LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let source_id = row.try_get::<String, _>("document_id")?;
        let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
        let source_text = format!(
            "{}\n{}",
            row.try_get::<String, _>("title")?,
            row.try_get::<String, _>("extracted_text")?,
        );

        count +=
            refresh_document_candidate_from_text(pool, &source_id, observation_id, &source_text)
                .await?;
    }

    Ok(count)
}

async fn refresh_document_candidate_from_text(
    pool: &PgPool,
    source_id: &str,
    observation_id: Option<String>,
    source_text: &str,
) -> Result<usize, TaskCandidateError> {
    let observation_id = observation_id
        .ok_or_else(|| TaskCandidateError::ObservationRequired(source_id.to_owned()))?;

    if let Some(fragment) = extract_candidate_fragment(source_text) {
        let payload = CandidatePayload {
            source_kind: TaskCandidateSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id.clone()),
            candidate_kind: TaskCandidateKind::Task,
            candidate_metadata: json!({}),
            project_id: None,
            title: title_from_fragment(&fragment.text),
            due_text: fragment.due_text,
            assignee_label: fragment.assignee_label,
            confidence: 0.7,
            evidence_excerpt: evidence_excerpt(&fragment.text),
        };
        upsert_suggested_candidate(pool, &payload).await?;
        return Ok(1);
    }

    let input = ObligationExtractionInput::document(
        source_id,
        source_text,
        ObligationEntityKind::Persona,
        OWNER_PERSONA_EXTRACTION_CONTEXT_ID,
    );
    let extraction = ObligationEngine::detect_candidates(&input)?;

    let mut count = 0usize;
    for obligation_candidate in extraction.obligations {
        let payload =
            task_candidate_payload_from_obligation(observation_id.clone(), &obligation_candidate);
        upsert_suggested_candidate(pool, &payload).await?;
        count += 1;
    }

    Ok(count)
}

async fn upsert_suggested_candidate(
    pool: &PgPool,
    payload: &CandidatePayload,
) -> Result<(), TaskCandidateError> {
    upsert_task_candidate(
        pool,
        payload,
        payload.task_candidate_id(),
        TaskCandidateReviewState::Suggested,
    )
    .await
}
```

### `backend/src/domains/tasks/candidates/store/review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/store/review.rs`
- Size bytes / Размер в байтах: `12858`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationReviewPort,
    ObligationReviewState,
};
use crate::domains::tasks::core::ObligationTaskLinkPort;
use crate::engines::obligation::ObligationCandidate;
use crate::platform::events::{EventEnvelope, EventStore};
use crate::platform::observations::materialize_review_transition_link_in_transaction;

use super::super::constants::{
    OBLIGATION_CANDIDATE_METADATA_KEY, TASK_CANDIDATE_EVENT_PREFIX,
    TASK_CANDIDATE_KIND_OBLIGATION_TASK, TASK_CANDIDATE_REVIEW_EVENT_TYPE,
};
use super::super::errors::TaskCandidateError;
use super::super::events::{ReviewCommandEvent, ReviewEventPayload};
use super::super::ids::task_id_from_candidate;
use super::super::models::{
    StoredCandidateRow, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateReviewState,
};
use super::super::persistence::row_task_candidate;
use super::super::validation::validate_non_empty;
use super::task_activation::upsert_task_in_transaction;

pub(super) async fn set_review_state(
    pool: &PgPool,
    command: &TaskCandidateReviewCommand,
) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
    set_review_state_with_observation(pool, command, None, None).await
}

pub(super) async fn set_review_state_with_observation(
    pool: &PgPool,
    command: &TaskCandidateReviewCommand,
    observation_id: Option<&str>,
    metadata: Option<Value>,
) -> Result<TaskCandidateReviewCommandResult, TaskCandidateError> {
    let command_id = validate_non_empty("command_id", &command.command_id)?;
    let task_candidate_id = validate_non_empty("task_candidate_id", &command.task_candidate_id)?;
    let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

    let mut transaction = pool.begin().await?;
    let event_id = format!("{TASK_CANDIDATE_EVENT_PREFIX}{command_id}");
    let event = ReviewCommandEvent {
        command_id,
        task_candidate_id: task_candidate_id.clone(),
        review_state: command.review_state,
        actor_id: actor_id.clone(),
        event_id: event_id.clone(),
        occurred_at: Utc::now(),
    }
    .into_event()?;

    EventStore::append_in_transaction(&mut transaction, &event).await?;
    apply_review_state_in_transaction(
        &mut transaction,
        &task_candidate_id,
        command.review_state,
        &event_id,
        &actor_id,
        event.occurred_at,
    )
    .await?;
    let metadata = match metadata {
        Some(extra) => Some(json!({
            "event_id": event_id,
            "context": extra,
        })),
        None => Some(json!({
            "event_id": event_id,
        })),
    };
    materialize_review_transition_link_in_transaction(
        &mut transaction,
        observation_id,
        "tasks",
        "task_candidate",
        &task_candidate_id,
        "review_state",
        command.review_state.as_str(),
        metadata,
    )
    .await?;

    transaction.commit().await?;

    Ok(TaskCandidateReviewCommandResult {
        task_candidate_id,
        review_state: command.review_state,
        event_id,
    })
}

pub(super) async fn apply_review_event(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), TaskCandidateError> {
    if event.event_type != TASK_CANDIDATE_REVIEW_EVENT_TYPE {
        return Err(TaskCandidateError::InvalidEventType);
    }

    let payload = ReviewEventPayload::from_payload(&event.payload)?;
    let actor_id = event
        .actor
        .as_ref()
        .and_then(|value| value.get("actor_id"))
        .and_then(Value::as_str)
        .ok_or(TaskCandidateError::MissingActorId)?;
    let actor_id = validate_non_empty("actor_id", actor_id)?;

    let mut transaction = pool.begin().await?;
    apply_review_state_in_transaction(
        &mut transaction,
        &payload.task_candidate_id,
        payload.review_state,
        &event.event_id,
        &actor_id,
        event.occurred_at,
    )
    .await?;

    transaction.commit().await?;
    Ok(())
}

async fn apply_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    event_id: &str,
    actor_id: &str,
    reviewed_at: DateTime<Utc>,
) -> Result<(), TaskCandidateError> {
    let candidate = row_task_candidate(transaction, task_candidate_id).await?;

    match review_state {
        TaskCandidateReviewState::UserConfirmed => {
            upsert_task_in_transaction(
                transaction,
                task_candidate_id,
                &candidate,
                event_id,
                actor_id,
            )
            .await?;
            update_candidate_review_state(
                transaction,
                task_candidate_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
            )
            .await?;
            sync_obligation_candidate_in_transaction(
                transaction,
                task_candidate_id,
                &candidate,
                TaskCandidateReviewState::UserConfirmed,
            )
            .await?;
        }
        TaskCandidateReviewState::Suggested | TaskCandidateReviewState::UserRejected => {
            sync_obligation_candidate_in_transaction(
                transaction,
                task_candidate_id,
                &candidate,
                review_state,
            )
            .await?;
            update_candidate_review_state(
                transaction,
                task_candidate_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
            )
            .await?;
            delete_task_for_candidate(transaction, task_candidate_id).await?;
        }
    }

    Ok(())
}

async fn sync_obligation_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    review_state: TaskCandidateReviewState,
) -> Result<(), TaskCandidateError> {
    if candidate.candidate_kind != TASK_CANDIDATE_KIND_OBLIGATION_TASK {
        return Ok(());
    }

    match review_state {
        TaskCandidateReviewState::UserConfirmed => {
            let observation_id = candidate.observation_id.as_deref().ok_or_else(|| {
                TaskCandidateError::ObservationRequired(task_candidate_id.to_owned())
            })?;
            let obligation_candidate =
                obligation_candidate_from_metadata(&candidate.candidate_metadata)?;
            let mut obligation = NewObligation::new(
                map_obligation_entity_kind(obligation_candidate.obligated_entity_kind),
                obligation_candidate.obligated_entity_id.clone(),
                obligation_candidate.statement.clone(),
                obligation_candidate.confidence,
                ObligationReviewState::UserConfirmed,
            )
            .metadata(json!({
                "task_candidate_id": task_candidate_id,
                "candidate_kind": TASK_CANDIDATE_KIND_OBLIGATION_TASK,
            }));
            if let (Some(kind), Some(entity_id)) = (
                obligation_candidate.beneficiary_entity_kind,
                obligation_candidate.beneficiary_entity_id.as_deref(),
            ) {
                obligation = obligation.beneficiary(map_obligation_entity_kind(kind), entity_id);
            }
            if let Some(condition) = obligation_candidate.condition.as_deref() {
                obligation = obligation.condition(condition);
            }

            let evidence = [NewObligationEvidence::observation(observation_id)
                .quote(obligation_candidate.quote.clone())
                .confidence(obligation_candidate.confidence)
                .metadata(json!({
                    "task_candidate_id": task_candidate_id,
                }))];
            let stored = ObligationReviewPort::upsert_with_evidence_in_transaction(
                transaction,
                &obligation,
                &evidence,
            )
            .await?;
            ObligationTaskLinkPort::link_fulfillment_task_in_transaction(
                transaction,
                &stored.obligation_id,
                &task_id_from_candidate(task_candidate_id),
            )
            .await?;
        }
        TaskCandidateReviewState::Suggested | TaskCandidateReviewState::UserRejected => {
            let linked_obligation_ids = sqlx::query_scalar::<_, String>(
                r#"
                SELECT link.obligation_id
                FROM obligation_task_links link
                JOIN tasks task
                  ON task.task_id = link.task_id
                WHERE task.task_candidate_id = $1
                  AND link.link_kind = 'fulfillment_task'
                ORDER BY link.obligation_id
                "#,
            )
            .bind(task_candidate_id)
            .fetch_all(&mut **transaction)
            .await?;
            let obligation_review_state = match review_state {
                TaskCandidateReviewState::Suggested => ObligationReviewState::Suggested,
                TaskCandidateReviewState::UserRejected => ObligationReviewState::UserRejected,
                TaskCandidateReviewState::UserConfirmed => unreachable!(),
            };
            for obligation_id in linked_obligation_ids {
                ObligationReviewPort::set_review_state_in_transaction(
                    transaction,
                    &obligation_id,
                    obligation_review_state,
                    candidate.observation_id.as_deref(),
                    Some(json!({
                        "task_candidate_id": task_candidate_id,
                        "review_state": review_state.as_str(),
                    })),
                )
                .await?;
            }
        }
    }

    Ok(())
}

fn obligation_candidate_from_metadata(
    metadata: &Value,
) -> Result<ObligationCandidate, TaskCandidateError> {
    let candidate = metadata
        .get(OBLIGATION_CANDIDATE_METADATA_KEY)
        .cloned()
        .ok_or_else(|| {
            TaskCandidateError::InvalidCandidateMetadata(
                OBLIGATION_CANDIDATE_METADATA_KEY.to_owned(),
            )
        })?;
    Ok(serde_json::from_value(candidate)?)
}

fn map_obligation_entity_kind(
    value: crate::engines::obligation::ObligationEntityKind,
) -> ObligationEntityKind {
    match value {
        crate::engines::obligation::ObligationEntityKind::Persona => ObligationEntityKind::Persona,
        crate::engines::obligation::ObligationEntityKind::Organization => {
            ObligationEntityKind::Organization
        }
        crate::engines::obligation::ObligationEntityKind::Project => ObligationEntityKind::Project,
        crate::engines::obligation::ObligationEntityKind::Communication => {
            ObligationEntityKind::Communication
        }
        crate::engines::obligation::ObligationEntityKind::Document => {
            ObligationEntityKind::Document
        }
        crate::engines::obligation::ObligationEntityKind::Task => ObligationEntityKind::Task,
        crate::engines::obligation::ObligationEntityKind::Event => ObligationEntityKind::Event,
        crate::engines::obligation::ObligationEntityKind::Decision => {
            ObligationEntityKind::Decision
        }
        crate::engines::obligation::ObligationEntityKind::Obligation => {
            ObligationEntityKind::Obligation
        }
        crate::engines::obligation::ObligationEntityKind::Knowledge => {
            ObligationEntityKind::Knowledge
        }
    }
}

async fn update_candidate_review_state(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    event_id: &str,
    actor_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/tasks/candidates/store/task_activation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/store/task_activation.rs`
- Size bytes / Размер в байтах: `2450`
- Included characters / Включено символов: `2450`
- Truncated / Обрезано: `no`

```rust
use sqlx::Postgres;
use sqlx::Transaction;

use crate::domains::tasks::candidates::errors::TaskCandidateError;
use crate::domains::tasks::candidates::ids::task_id_from_candidate;
use crate::domains::tasks::candidates::models::StoredCandidateRow;

pub(super) async fn upsert_task_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    event_id: &str,
    actor_id: &str,
) -> Result<(), TaskCandidateError> {
    let observation_id = candidate
        .observation_id
        .clone()
        .or_else(|| {
            (candidate.source_kind == "observation").then(|| candidate.source_id.clone())
        })
        .ok_or_else(|| {
            TaskCandidateError::ObservationRequired(format!(
                "task candidate {task_candidate_id} has source_kind={} and no canonical observation_id",
                candidate.source_kind
            ))
        })?;

    sqlx::query(
        r#"
        INSERT INTO tasks (
            task_id,
            task_candidate_id,
            title,
            provenance_kind,
            provenance_id,
            source_kind,
            source_id,
            source_type,
            project_id,
            status,
            makosh_status,
            created_from_event_id,
            created_by_actor_id
        )
        VALUES ($1, $2, $3, 'observation', $4, $5, $6, $7, $8, 'active', 'ready', $9, $10)
        ON CONFLICT (task_candidate_id)
        DO UPDATE SET
            title = EXCLUDED.title,
            provenance_kind = EXCLUDED.provenance_kind,
            provenance_id = EXCLUDED.provenance_id,
            source_kind = EXCLUDED.source_kind,
            source_id = EXCLUDED.source_id,
            source_type = EXCLUDED.source_type,
            project_id = EXCLUDED.project_id,
            status = EXCLUDED.status,
            makosh_status = EXCLUDED.makosh_status,
            created_from_event_id = EXCLUDED.created_from_event_id,
            created_by_actor_id = EXCLUDED.created_by_actor_id,
            updated_at = now()
        "#,
    )
    .bind(task_id_from_candidate(task_candidate_id))
    .bind(task_candidate_id)
    .bind(&candidate.title)
    .bind(&observation_id)
    .bind("observation")
    .bind(&observation_id)
    .bind("observation")
    .bind(&candidate.project_id)
    .bind(event_id)
    .bind(actor_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
```

### `backend/src/domains/tasks/candidates/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/candidates/validation.rs`
- Size bytes / Размер в байтах: `982`
- Included characters / Включено символов: `982`
- Truncated / Обрезано: `no`

```rust
use super::constants::{DEFAULT_LIMIT, MAX_LIMIT, MIN_LIMIT};
use super::errors::TaskCandidateError;

pub(crate) fn validate_non_empty(field: &str, value: &str) -> Result<String, TaskCandidateError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TaskCandidateError::EmptyField(field.to_owned()));
    }

    Ok(value.to_owned())
}

pub(crate) fn validate_limit(limit: i64) -> Result<i64, TaskCandidateError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(TaskCandidateError::InvalidLimit);
    }

    Ok(limit)
}

pub(crate) fn validate_optional_limit(limit: Option<i64>) -> Result<i64, TaskCandidateError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIMIT))
}

pub(crate) fn text_preview(value: &str, max_chars: usize) -> String {
    let preview = value.trim().chars().take(max_chars).collect::<String>();
    if value.trim().chars().count() > max_chars {
        format!("{preview}...")
    } else {
        preview
    }
}
```

### `backend/src/domains/tasks/command_service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/command_service.rs`
- Size bytes / Размер в байтах: `26037`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::api::{NewTask, Task, TaskError, TaskStore, TaskUpdate};
use super::core::{
    TaskChecklist, TaskChecklistStore, TaskCoreError, TaskEvidence, TaskEvidenceStore,
    TaskRelation, TaskRelationStore, TaskSubtask, TaskSubtaskStore,
};
use super::intelligence::TaskIntelligenceService;

#[derive(Clone)]
pub struct TaskCommandService {
    pool: PgPool,
}

impl TaskCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_task_manual(&self, req: &NewTask) -> Result<Task, TaskCommandServiceError> {
        let resolved = self.resolve_provenance(req).await?;
        let mut transaction = self.pool.begin().await?;
        let task = TaskStore::new(self.pool.clone())
            .create_in_transaction(&mut transaction, &resolved)
            .await?;
        transaction.commit().await?;
        Ok(task)
    }

    pub async fn update_task_manual(
        &self,
        task_id: &str,
        update: &TaskUpdate,
    ) -> Result<Task, TaskCommandServiceError> {
        let task_store = TaskStore::new(self.pool.clone());
        task_store.get(task_id).await?.ok_or(TaskError::NotFound)?;

        let observation = self
            .capture_observation(
                "task update",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "title": update.title.as_deref(),
                    "description": update.description.as_deref(),
                    "makosh_status": update.makosh_status.as_deref(),
                    "priority_score": update.priority_score,
                    "risk_score": update.risk_score,
                    "readiness_score": update.readiness_score,
                    "area": update.area.as_deref(),
                    "why": update.why.as_deref(),
                    "outcome": update.outcome.as_deref(),
                    "due_at": update.due_at,
                    "waiting_reason": update.waiting_reason.as_deref(),
                    "energy_type": update.energy_type.as_deref(),
                    "confidentiality": update.confidentiality.as_deref(),
                    "tags": update.tags.clone(),
                    "task_metadata": update.task_metadata.clone(),
                    "linked_person_id": update.linked_person_id.as_deref(),
                    "linked_organization_id": update.linked_organization_id.as_deref(),
                    "completed_at": update.completed_at,
                }),
                format!("task://{task_id}/update"),
                json!({
                    "captured_by": "tasks_service.update_task_manual",
                    "operation": "update_task_manual",
                }),
            )
            .await?;

        Ok(task_store
            .update_with_observation(
                task_id,
                update,
                &observation.observation_id,
                "task_update",
                json!({}),
            )
            .await?)
    }

    pub async fn set_status_manual(
        &self,
        task_id: &str,
        status: &str,
    ) -> Result<Task, TaskCommandServiceError> {
        let task_store = TaskStore::new(self.pool.clone());
        task_store.get(task_id).await?.ok_or(TaskError::NotFound)?;

        let observation = self
            .capture_observation(
                "task status",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "status": status,
                    "operation": "set_status",
                }),
                format!("task://{task_id}/status"),
                json!({
                    "captured_by": "tasks_service.set_status_manual",
                    "operation": "set_status_manual",
                }),
            )
            .await?;

        Ok(task_store
            .set_status_with_observation(
                task_id,
                status,
                &observation.observation_id,
                "status_update",
                json!({
                    "status": status,
                }),
            )
            .await?)
    }

    pub async fn archive_manual(&self, task_id: &str) -> Result<Task, TaskCommandServiceError> {
        let task_store = TaskStore::new(self.pool.clone());
        task_store.get(task_id).await?.ok_or(TaskError::NotFound)?;

        let observation = self
            .capture_observation(
                "task archive",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "status": "archived",
                    "operation": "archive",
                }),
                format!("task://{task_id}/archive"),
                json!({
                    "captured_by": "tasks_service.archive_manual",
                    "operation": "archive_manual",
                }),
            )
            .await?;

        Ok(task_store
            .archive_with_observation(
                task_id,
                &observation.observation_id,
                "status_update",
                json!({
                    "status": "archived",
                }),
            )
            .await?)
    }

    pub async fn analyze_runtime(
        &self,
        task_id: &str,
    ) -> Result<TaskAnalysisResult, TaskCommandServiceError> {
        let task = TaskStore::new(self.pool.clone())
            .get(task_id)
            .await?
            .ok_or(TaskError::NotFound)?;
        let has_ctx = super::core::TaskContextPackStore::new(self.pool.clone())
            .get(task_id)
            .await
            .map(|c| c.is_some())
            .unwrap_or(false);
        let _has_relations = TaskRelationStore::new(self.pool.clone())
            .list(task_id)
            .await
            .map(|r| !r.is_empty())
            .unwrap_or(false);
        let is_legal = task.area.as_deref() == Some("legal") || task.area.as_deref() == Some("tax");
        let is_tax = task.area.as_deref() == Some("tax");
        let has_contact = task.linked_person_id.is_some();
        let has_org = task.linked_organization_id.is_some();
        let priority = TaskIntelligenceService::calculate_priority(
            task.due_at,
            has_contact,
            has_org,
            task.project_id.is_some(),
            is_legal,
            is_tax,
            false,
        );
        let risk = TaskIntelligenceService::calculate_risk(
            task.due_at
                .map(|d| (d - Utc::now()).num_hours() < 24)
                .unwrap_or(false),
            false,
            false,
            false,
            is_legal,
            &task.title,
        );
        let readiness = TaskIntelligenceService::calculate_readiness(
            task.description.is_some(),
            has_ctx,
            false,
            task.due_at.is_some(),
            true,
            has_contact,
        );
        let missing_context = TaskIntelligenceService::detect_missing_context(
            task.description.is_some(),
            has_ctx,
            task.due_at.is_some(),
            has_contact,
            task.project_id.is_some(),
        );
        let next_action = TaskIntelligenceService::suggest_next_action(
            &task.makosh_status,
            false,
            false,
            task.waiting_reason.as_deref(),
        );
        let update = TaskUpdate {
            priority_score: Some(priority),
            risk_score: Some(risk),
            readiness_score: Some(readiness),
            ..Default::default()
        };

        let observation = self
            .capture_observation(
                "task analyze",
                ObservationOriginKind::LocalRuntime,
                json!({
                    "task_id": task_id,
                    "priority_score": priority,
                    "risk_score": risk,
                    "readiness_score": readiness,
                    "missing_context": missing_context,
                    "next_action": next_action,
                }),
                format!("task://{task_id}/analyze"),
                json!({
                    "captured_by": "tasks_service.analyze_runtime",
                    "operation": "analyze_runtime",
                    "engine": "task_intelligence",
                }),
            )
            .await?;

        TaskStore::new(self.pool.clone())
            .update_with_observation(
                task_id,
                &update,
                &observation.observation_id,
                "analysis_update",
                json!({}),
            )
            .await?;

        Ok(TaskAnalysisResult {
            priority,
            risk,
            readiness,
            missing_context,
            next_action,
        })
    }

    pub async fn add_evidence(
        &self,
        task_id: &str,
        requested_source_type: Option<&str>,
        requested_source_id: Option<&str>,
        quote: Option<String>,
        confidence: Option<f64>,
    ) -> Result<TaskEvidence, TaskCommandServiceError> {
        let requested_source_type = requested_source_type.unwrap_or("manual").trim();
        let requested_source_id = requested_source_id.map(str::trim);

        let (source_type, source_id) =
            if requested_source_type.is_empty() || requested_source_type == "manual" {
                let observation = self
                    .capture_observation(
                        "task evidence",
                        ObservationOriginKind::Manual,
                        json!({
                            "task_id": task_id,
                            "quote": quote,
                            "confidence": confidence,
                        }),
                        format!("task://{task_id}/evidence"),
                        json!({
                            "captured_by": "tasks_service.add_evidence",
                            "operation": "add_evidence",
                        }),
                    )
                    .await?;
                ("observation".to_owned(), observation.observation_id)
            } else {
                let source_id =
                    requested_source_id.ok_or(TaskCommandServiceError::MissingEvidenceSourceId)?;
                if source_id.is_empty() {
                    return Err(TaskCommandServiceError::MissingEvidenceSourceId);
                }
                (requested_source_type.to_owned(), source_id.to_owned())
            };

        Ok(TaskEvidenceStore::new(self.pool.clone())
            .add(
                task_id,
                &source_type,
                &source_id,
                quote.as_deref(),
                confidence,
            )
            .await?)
    }

    pub async fn add_relation_manual(
        &self,
        task_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
    ) -> Result<TaskRelation, TaskCommandServiceError> {
        let observation = self
            .capture_observation(
                "task relation",
                ObservationOriginKind::Manual,
                json!({
                    "task_id": task_id,
                    "entity_type": entity_type,
                    "entity_id": entity_id,
                    "relation_type": relation_type,
                }),
                format!("task://{task_id}/relation"),
                json!({
                    "captured_by": "tasks_service.add_relation_manual",
                    "operation": "add_relation_manual",
                }),
            )
            .await?;

        Ok(TaskRelationStore::new(self.pool.clone())
            .link
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/tasks/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core.rs`
- Size bytes / Размер в байтах: `935`
- Included characters / Включено символов: `935`
- Truncated / Обрезано: `no`

```rust
mod checklists;
mod context_packs;
mod errors;
mod evidence;
mod external_identities;
mod obligation_links;
mod observation_links;
mod provider_store;
mod providers;
mod relations;
mod subtasks;

pub use checklists::{TaskChecklist, TaskChecklistStore};
pub use context_packs::{TaskContextPack, TaskContextPackStore};
pub use errors::TaskCoreError;
pub use evidence::{TaskEvidence, TaskEvidenceStore};
pub use external_identities::{ExternalTaskIdentity, ExternalTaskIdentityStore};
pub use obligation_links::ObligationTaskLinkStore;
pub use obligation_links::ObligationTaskLinkStore as ObligationTaskLinkPort;
pub(crate) use observation_links::{
    materialize_task_entity_link_in_transaction, materialize_task_observation_link_in_transaction,
};
pub use provider_store::TaskProviderStore;
pub use providers::TaskProviderAccount;
pub use relations::{TaskRelation, TaskRelationStore};
pub use subtasks::{TaskSubtask, TaskSubtaskStore};
```

### `backend/src/domains/tasks/core/checklists.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/checklists.rs`
- Size bytes / Размер в байтах: `3319`
- Included characters / Включено символов: `3319`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::{TaskCoreError, materialize_task_entity_link_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskChecklist {
    pub id: String,
    pub task_id: String,
    pub items: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskChecklistStore {
    pool: PgPool,
}

impl TaskChecklistStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<TaskChecklist>, TaskCoreError> {
        let row = sqlx::query(
            r#"
            SELECT id::text, task_id, items, source, created_at, updated_at
            FROM task_checklists
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(TaskChecklist {
                id: row.try_get("id")?,
                task_id: row.try_get("task_id")?,
                items: row.try_get("items")?,
                source: row.try_get("source")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    pub async fn set(
        &self,
        task_id: &str,
        items: Value,
        source: &str,
    ) -> Result<TaskChecklist, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let checklist = Self::set_in_transaction(&mut transaction, task_id, items, source).await?;

        if let Some(observation_id) = checklist
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        {
            materialize_task_entity_link_in_transaction(
                &mut transaction,
                Some(observation_id),
                "task_checklist",
                &checklist.id,
                None,
                None,
                Some(json!({
                    "task_id": task_id,
                })),
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(checklist)
    }

    async fn set_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        task_id: &str,
        items: Value,
        source: &str,
    ) -> Result<TaskChecklist, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_checklists (task_id, items, source)
            VALUES ($1, $2, $3)
            RETURNING id::text, task_id, items, source, created_at, updated_at
            "#,
        )
        .bind(task_id)
        .bind(&items)
        .bind(source)
        .fetch_one(&mut **transaction)
        .await?;

        Ok(TaskChecklist {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            items: row.try_get("items")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
```

### `backend/src/domains/tasks/core/context_packs.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/context_packs.rs`
- Size bytes / Размер в байтах: `3523`
- Included characters / Включено символов: `3523`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;
use crate::engines::context_packs::{
    ContextPack, ContextPackKind, ContextPackSourceKind, ContextPackStore, NewContextPack,
    NewContextPackSource,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskContextPack {
    pub id: String,
    pub task_id: String,
    pub summary: Option<String>,
    pub source_summary: Option<String>,
    pub open_questions: Value,
    pub blockers: Value,
    pub risks: Value,
    pub suggested_next_action: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskContextPackStore {
    pool: PgPool,
}

impl TaskContextPackStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<TaskContextPack>, TaskCoreError> {
        ContextPackStore::new(self.pool.clone())
            .get(ContextPackKind::Task, task_id)
            .await?
            .map(|pack| task_context_pack_from_engine(pack, task_id))
            .transpose()
    }

    pub async fn upsert(
        &self,
        task_id: &str,
        summary: Option<&str>,
        questions: Value,
        blockers: Value,
        risks: Value,
        next_action: Option<&str>,
    ) -> Result<TaskContextPack, TaskCoreError> {
        let stored = ContextPackStore::new(self.pool.clone())
            .upsert_with_sources(
                &NewContextPack::new(
                    ContextPackKind::Task,
                    task_id,
                    json!({
                        "summary": summary,
                        "source_summary": summary,
                        "open_questions": questions,
                        "blockers": blockers,
                        "risks": risks,
                        "suggested_next_action": next_action,
                    }),
                )
                .metadata(json!({
                    "owner": "domains.tasks.core.context_packs",
                })),
                &[NewContextPackSource::new(ContextPackSourceKind::Task, task_id).role("subject")],
            )
            .await?;
        task_context_pack_from_engine(stored, task_id)
    }
}

fn task_context_pack_from_engine(
    pack: ContextPack,
    task_id: &str,
) -> Result<TaskContextPack, TaskCoreError> {
    let content = &pack.content;
    Ok(TaskContextPack {
        id: pack.context_pack_id,
        task_id: task_id.to_owned(),
        summary: optional_string(content, "summary"),
        source_summary: optional_string(content, "source_summary"),
        open_questions: content
            .get("open_questions")
            .cloned()
            .unwrap_or_else(|| json!([])),
        blockers: content
            .get("blockers")
            .cloned()
            .unwrap_or_else(|| json!([])),
        risks: content.get("risks").cloned().unwrap_or_else(|| json!([])),
        suggested_next_action: optional_string(content, "suggested_next_action"),
        generated_at: pack.built_at,
        model: optional_string(&pack.metadata, "model"),
        created_at: pack.built_at,
        updated_at: pack.updated_at,
    })
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}
```
