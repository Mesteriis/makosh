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

- Chunk ID / ID чанка: `038-source-backend-part-018`
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

### `backend/src/domains/calendar/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/service.rs`
- Size bytes / Размер в байтах: `35`
- Included characters / Включено символов: `35`
- Truncated / Обрезано: `no`

```rust
pub use super::command_service::*;
```

### `backend/src/domains/calendar/sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/sync.rs`
- Size bytes / Размер в байтах: `1535`
- Included characters / Включено символов: `1535`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

/// Basic ICS export for a single event
pub fn export_event_ics(
    title: &str,
    description: Option<&str>,
    location: Option<&str>,
    start_at: &str,
    end_at: &str,
    timezone: Option<&str>,
) -> String {
    let tz = timezone.unwrap_or("Europe/Madrid");
    let desc = description.unwrap_or("");
    let loc = location.unwrap_or("");
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Макошь//Calendar//EN\r\nBEGIN:VEVENT\r\nDTSTART;TZID={tz}:{start_at}\r\nDTEND;TZID={tz}:{end_at}\r\nSUMMARY:{title}\r\nDESCRIPTION:{desc}\r\nLOCATION:{loc}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

/// Export event as markdown
pub fn export_event_md(
    title: &str,
    description: Option<&str>,
    location: Option<&str>,
    start_at: &str,
    end_at: &str,
    participants: &[String],
) -> String {
    let mut md = format!("# {title}\n\n**When:** {start_at} - {end_at}\n\n");
    if let Some(loc) = location
        && !loc.is_empty()
    {
        md.push_str(&format!("**Where:** {loc}\n\n"));
    }
    if let Some(desc) = description
        && !desc.is_empty()
    {
        md.push_str(&format!("{desc}\n\n"));
    }
    if !participants.is_empty() {
        md.push_str("## Participants\n\n");
        for p in participants {
            md.push_str(&format!("- {p}\n"));
        }
    }
    md
}

#[derive(Debug, Error)]
pub enum CalendarSyncError {
    #[error("sync failed: {0}")]
    SyncFailed(String),
    #[error("import failed: {0}")]
    ImportFailed(String),
}
```

### `backend/src/domains/communications/actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/actions.rs`
- Size bytes / Размер в байтах: `3539`
- Included characters / Включено символов: `3538`
- Truncated / Обрезано: `no`

```rust
// §4.2-4.4: Reply-all, Forward-EML, Send-later, Undo-send, Quoting
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplyConfig {
    pub quote_original: bool,
    pub include_attachments: bool,
    pub reply_all: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForwardConfig {
    pub as_eml: bool,
    pub attachments_only: bool,
    pub include_ai_summary: bool,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledSend {
    pub send_at: DateTime<Utc>,
    pub draft_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UndoSendWindow {
    pub window_seconds: i32,
    pub enabled: bool,
}

impl Default for ReplyConfig {
    fn default() -> Self {
        Self {
            quote_original: true,
            include_attachments: false,
            reply_all: false,
        }
    }
}

/// Build a reply body with optional quoting.
pub fn build_reply_body(
    original_sender: &str,
    original_date: &str,
    original_body: &str,
    reply_text: &str,
    quote: bool,
) -> String {
    if !quote {
        return reply_text.to_owned();
    }
    let quoted = original_body
        .lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{reply_text}\n\nOn {original_date}, {original_sender} wrote:\n{quoted}")
}

/// Build a forward body.
pub fn build_forward_body(
    original_sender: &str,
    original_date: &str,
    original_subject: &str,
    original_body: &str,
    note: Option<&str>,
) -> String {
    let header =
        format!("From: {original_sender}\nDate: {original_date}\nSubject: {original_subject}");
    let note_line = note.map(|n| format!("{n}\n\n")).unwrap_or_default();
    format!("{note_line}--- Forwarded message ---\n{header}\n\n{original_body}")
}

/// Build an EML representation of a forwarded message.
pub fn build_eml_forward(
    original_sender: &str,
    original_date: &str,
    original_subject: &str,
    original_body: &str,
    forward_to: &[String],
) -> String {
    let to = forward_to.join(", ");
    format!(
        "From: makosh@local\r\nTo: {to}\r\nSubject: Fwd: {original_subject}\r\nDate: {}\r\nContent-Type: message/rfc822\r\n\r\nFrom: {original_sender}\r\nDate: {original_date}\r\nSubject: {original_subject}\r\n\r\n{original_body}",
        Utc::now().to_rfc2822()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reply_with_quote() {
        let b = build_reply_body(
            "alice@ex.com",
            "Mon, 01 Jan 2026",
            "Hello\nHow are you?",
            "I'm fine",
            true,
        );
        assert!(b.contains("> Hello"));
        assert!(b.contains("I'm fine"));
    }
    #[test]
    fn reply_without_quote() {
        let b = build_reply_body("a@b.com", "d", "orig", "reply", false);
        assert_eq!(b, "reply");
    }
    #[test]
    fn forward_with_note() {
        let b = build_forward_body("s@e.com", "d", "subj", "body", Some("FYI"));
        assert!(b.contains("FYI"));
        assert!(b.contains("--- Forwarded"));
    }
    #[test]
    fn forward_eml_format() {
        let b = build_eml_forward("s@e.com", "d", "subj", "body", &["to@e.com".into()]);
        assert!(b.contains("Content-Type: message/rfc822"));
    }
    #[test]
    fn reply_config_defaults() {
        let c = ReplyConfig::default();
        assert!(c.quote_original);
        assert!(!c.reply_all);
    }
}
```

### `backend/src/domains/communications/ai_reply.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/ai_reply.rs`
- Size bytes / Размер в байтах: `3461`
- Included characters / Включено символов: `3461`
- Truncated / Обрезано: `no`

```rust
use crate::domains::communications::messages::ProjectedMessage;
use crate::platform::ai_runtime::{AiRuntimePortError, SharedAiRuntimePort};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiReplyDraft {
    pub subject: String,
    pub body: String,
    pub tone: String,
    pub language: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiReplyOptions {
    pub tone: Option<String>,
    pub language: Option<String>,
    pub context: Option<String>,
}

#[derive(Clone)]
pub struct AiReplyService {
    runtime: Option<SharedAiRuntimePort>,
}

impl AiReplyService {
    pub fn new(runtime: Option<SharedAiRuntimePort>) -> Self {
        Self { runtime }
    }

    pub async fn generate_reply(
        &self,
        message: &ProjectedMessage,
        options: &AiReplyOptions,
    ) -> Result<Option<AiReplyDraft>, AiReplyError> {
        let Some(ref runtime) = self.runtime else {
            return Ok(None);
        };
        let tone = options.tone.as_deref().unwrap_or("professional");
        let lang = options.language.as_deref().unwrap_or("auto-detect");
        let context = options.context.as_deref().unwrap_or("");

        let prompt = format!(
            "You are replying to an email.\n\nOriginal email:\nFrom: {}\nSubject: {}\nBody:\n{}\n\n{}\nGenerate a reply in {lang} with a {tone} tone. Return ONLY the reply body text, no subject line, no explanations.",
            message.sender,
            message.subject,
            truncate(&message.body_text, 2000),
            if context.is_empty() {
                "".into()
            } else {
                format!("Additional context: {context}")
            },
        );

        let result = runtime.chat(&prompt).await?;
        let body = result.content.trim().to_owned();

        let subject = if message.subject.to_lowercase().starts_with("re:") {
            message.subject.clone()
        } else {
            format!("Re: {}", message.subject)
        };

        Ok(Some(AiReplyDraft {
            subject,
            body,
            tone: tone.into(),
            language: lang.into(),
        }))
    }

    pub async fn generate_reply_variants(
        &self,
        message: &ProjectedMessage,
        languages: &[String],
        tones: &[String],
    ) -> Result<Vec<AiReplyDraft>, AiReplyError> {
        let mut variants = Vec::new();
        for lang in languages {
            for tone in tones {
                if let Some(draft) = self
                    .generate_reply(
                        message,
                        &AiReplyOptions {
                            language: Some(lang.clone()),
                            tone: Some(tone.clone()),
                            context: None,
                        },
                    )
                    .await?
                {
                    variants.push(draft);
                }
            }
        }
        Ok(variants)
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}

#[derive(Debug, Error)]
pub enum AiReplyError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimePortError),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn truncate_short() {
        assert_eq!(truncate("hi", 10), "hi");
    }
    #[test]
    fn truncate_long() {
        assert_eq!(truncate("hello world long text", 5), "hello");
    }
}
```

### `backend/src/domains/communications/ai_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/ai_state.rs`
- Size bytes / Размер в байтах: `10793`
- Included characters / Включено символов: `10793`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::ObservationStoreError;

const EVENT_TYPE_CHANGED: &str = "mail.ai_state.changed";
use super::evidence::link_mail_entity_in_transaction;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommunicationAiState {
    New,
    Processing,
    Processed,
    ReviewRequired,
    Failed,
    Archived,
}

impl CommunicationAiState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Processing => "PROCESSING",
            Self::Processed => "PROCESSED",
            Self::ReviewRequired => "REVIEW_REQUIRED",
            Self::Failed => "FAILED",
            Self::Archived => "ARCHIVED",
        }
    }
}

impl TryFrom<&str> for CommunicationAiState {
    type Error = CommunicationAiStateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "NEW" => Ok(Self::New),
            "PROCESSING" => Ok(Self::Processing),
            "PROCESSED" => Ok(Self::Processed),
            "REVIEW_REQUIRED" => Ok(Self::ReviewRequired),
            "FAILED" => Ok(Self::Failed),
            "ARCHIVED" => Ok(Self::Archived),
            _ => Err(CommunicationAiStateError::Invalid("ai_state")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationAiStateRecord {
    pub message_id: String,
    pub ai_state: CommunicationAiState,
    pub review_reason: Option<String>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommunicationAiStateTransitionRequest {
    pub ai_state: CommunicationAiState,
    pub review_reason: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct CommunicationAiStateStore {
    pool: PgPool,
}

impl CommunicationAiStateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn current(
        &self,
        message_id: &str,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        let message_id = normalize_required("message_id", message_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                m.message_id,
                COALESCE(s.ai_state, 'NEW') AS ai_state,
                s.review_reason,
                s.last_error,
                COALESCE(s.created_at, m.projected_at) AS created_at,
                COALESCE(s.updated_at, m.projected_at) AS updated_at
            FROM communication_messages m
            LEFT JOIN communication_ai_states s ON s.message_id = m.message_id
            WHERE m.message_id = $1
            "#,
        )
        .bind(&message_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_ai_state).transpose()
    }

    pub async fn transition(
        &self,
        message_id: &str,
        request: CommunicationAiStateTransitionRequest,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        self.transition_with_observation(message_id, request, None, "ai_state_transition", None)
            .await
    }

    pub async fn transition_with_observation(
        &self,
        message_id: &str,
        request: CommunicationAiStateTransitionRequest,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        let message_id = normalize_required("message_id", message_id)?;
        let update = NormalizedCommunicationAiStateTransition::from_request(request)?;
        let mut transaction = self.pool.begin().await?;

        let Some(previous) = select_current_ai_state(&mut transaction, &message_id).await? else {
            transaction.rollback().await?;
            return Ok(None);
        };

        let row = sqlx::query(
            r#"
            INSERT INTO communication_ai_states (message_id, ai_state, review_reason, last_error)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (message_id)
            DO UPDATE SET
                ai_state = EXCLUDED.ai_state,
                review_reason = EXCLUDED.review_reason,
                last_error = EXCLUDED.last_error,
                updated_at = now()
            RETURNING message_id, ai_state, review_reason, last_error, created_at, updated_at
            "#,
        )
        .bind(&message_id)
        .bind(update.ai_state.as_str())
        .bind(update.review_reason.as_deref())
        .bind(update.last_error.as_deref())
        .fetch_one(&mut *transaction)
        .await?;
        let record = row_to_ai_state(row)?;
        let event = ai_state_changed_event(&record, previous.ai_state)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                record.message_id.clone(),
                relationship_kind,
                json!({
                    "previous_ai_state": previous.ai_state.as_str(),
                    "ai_state": record.ai_state.as_str(),
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(Some(record))
    }
}

#[derive(Debug)]
struct NormalizedCommunicationAiStateTransition {
    ai_state: CommunicationAiState,
    review_reason: Option<String>,
    last_error: Option<String>,
}

impl NormalizedCommunicationAiStateTransition {
    fn from_request(
        request: CommunicationAiStateTransitionRequest,
    ) -> Result<Self, CommunicationAiStateError> {
        let review_reason = normalize_optional(request.review_reason)?;
        let last_error = normalize_optional(request.last_error)?;

        match request.ai_state {
            CommunicationAiState::ReviewRequired if review_reason.is_none() => {
                return Err(CommunicationAiStateError::Invalid("review_reason"));
            }
            CommunicationAiState::Failed if last_error.is_none() => {
                return Err(CommunicationAiStateError::Invalid("last_error"));
            }
            _ => {}
        }

        Ok(Self {
            ai_state: request.ai_state,
            review_reason: if request.ai_state == CommunicationAiState::ReviewRequired {
                review_reason
            } else {
                None
            },
            last_error: if request.ai_state == CommunicationAiState::Failed {
                last_error
            } else {
                None
            },
        })
    }
}

async fn select_current_ai_state(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: &str,
) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
    let row = sqlx::query(
        r#"
        SELECT
            m.message_id,
            COALESCE(s.ai_state, 'NEW') AS ai_state,
            s.review_reason,
            s.last_error,
            COALESCE(s.created_at, m.projected_at) AS created_at,
            COALESCE(s.updated_at, m.projected_at) AS updated_at
        FROM communication_messages m
        LEFT JOIN communication_ai_states s ON s.message_id = m.message_id
        WHERE m.message_id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(&mut **transaction)
    .await?;

    row.map(row_to_ai_state).transpose()
}

fn row_to_ai_state(row: PgRow) -> Result<CommunicationAiStateRecord, CommunicationAiStateError> {
    let ai_state: String = row.try_get("ai_state")?;
    Ok(CommunicationAiStateRecord {
        message_id: row.try_get("message_id")?,
        ai_state: CommunicationAiState::try_from(ai_state.as_str())?,
        review_reason: row.try_get("review_reason")?,
        last_error: row.try_get("last_error")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn ai_state_changed_event(
    record: &CommunicationAiStateRecord,
    previous_ai_state: CommunicationAiState,
) -> Result<NewEventEnvelope, CommunicationAiStateError> {
    Ok(NewEventEnvelope::builder(
        format!(
            "mail_ai_state_event:{}:{:x}",
            record.message_id,
            system_time_nanos()
        ),
        EVENT_TYPE_CHANGED,
        Utc::now(),
        json!({ "kind": "mail_ai_state_api" }),
        json!({
            "kind": "mail_ai_state",
            "id": record.message_id,
            "message_id": record.message_id,
        }),
    )
    .actor(json!({ "actor_id": "makosh-frontend" }))
    .payload(json!({
        "message_id": record.message_id,
        "ai_state": record.ai_state.as_str(),
        "previous_ai_state": previous_ai_state.as_str(),
        "review_required": record.review_reason.is_some(),
        "failed": record.last_error.is_some(),
    }))
    .provenance(json!({
        "source_kind": "local_api",
        "source_id": record.message_id,
    }))
    .correlation_id(record.message_id.clone())
    .build()?)
}

fn normalize_required(
    field: &'static str,
    value: &str,
) -> Result<String, CommunicationAiStateError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CommunicationAiStateError::Invalid(field));
    }
    Ok(value.to_owned())
}

fn normalize_optional(value: Option<String>) -> Result<Option<String>, CommunicationAiStateError> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn system_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[derive(Debug, Error)]
pub enum CommunicationAiStateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error(transparent)]
    EventStore(#[from] crate::platform::events::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),
    #[error("invalid mail AI state field: {0}")]
    Invalid(&'static str),
}
```

### `backend/src/domains/communications/analytics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/analytics.rs`
- Size bytes / Размер в байтах: `6975`
- Included characters / Включено символов: `6975`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct MailboxHealth {
    pub total_messages: i64,
    pub unread: i64,
    pub needs_action: i64,
    pub waiting: i64,
    pub done: i64,
    pub archived: i64,
    pub spam: i64,
    pub important: i64,
    pub with_attachments: i64,
    pub average_importance: f64,
    pub oldest_message_days: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SenderStats {
    pub sender: String,
    pub message_count: i64,
    pub avg_importance: f64,
    pub last_message_days: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SenderStatsListPage {
    pub items: Vec<SenderStats>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone)]
pub struct EmailAnalyticsStore {
    pool: PgPool,
}

impl EmailAnalyticsStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mailbox_health(
        &self,
        account_id: Option<&str>,
    ) -> Result<MailboxHealth, EmailAnalyticsError> {
        let row = sqlx::query(
            r#"SELECT
                count(*)::BIGINT AS total_messages,
                count(*) FILTER (WHERE workflow_state = 'new')::BIGINT AS unread,
                count(*) FILTER (WHERE workflow_state = 'needs_action')::BIGINT AS needs_action,
                count(*) FILTER (WHERE workflow_state = 'waiting')::BIGINT AS waiting,
                count(*) FILTER (WHERE workflow_state = 'done')::BIGINT AS done,
                count(*) FILTER (WHERE workflow_state = 'archived')::BIGINT AS archived,
                count(*) FILTER (WHERE workflow_state = 'spam')::BIGINT AS spam,
                count(*) FILTER (WHERE importance_score >= 75)::BIGINT AS important,
                count(*) FILTER (WHERE EXISTS(SELECT 1 FROM communication_attachments a WHERE a.message_id = communication_messages.message_id))::BIGINT AS with_attachments,
                COALESCE(avg(importance_score), 0)::DOUBLE PRECISION AS average_importance,
                EXTRACT(EPOCH FROM now() - min(occurred_at))::DOUBLE PRECISION / 86400.0::DOUBLE PRECISION AS oldest_message_days
            FROM communication_messages
            WHERE ($1::text IS NULL OR account_id = $1)
              AND channel_kind = 'email'
              AND local_state = 'active'"#,
        ).bind(account_id).fetch_one(&self.pool).await?;

        Ok(MailboxHealth {
            total_messages: row.try_get("total_messages")?,
            unread: row.try_get("unread")?,
            needs_action: row.try_get("needs_action")?,
            waiting: row.try_get("waiting")?,
            done: row.try_get("done")?,
            archived: row.try_get("archived")?,
            spam: row.try_get("spam")?,
            important: row.try_get("important")?,
            with_attachments: row.try_get("with_attachments")?,
            average_importance: row.try_get("average_importance")?,
            oldest_message_days: row.try_get("oldest_message_days")?,
        })
    }

    pub async fn top_senders(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SenderStats>, EmailAnalyticsError> {
        Ok(self.top_senders_page(account_id, limit, None).await?.items)
    }

    pub async fn top_senders_page(
        &self,
        account_id: Option<&str>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<SenderStatsListPage, EmailAnalyticsError> {
        let limit = limit.clamp(1, 50);
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_sender_stats_cursor)
            .transpose()?;
        let rows = sqlx::query(
            r#"WITH sender_stats AS (
                SELECT sender, count(*)::BIGINT AS message_count,
                    COALESCE(avg(importance_score), 0)::DOUBLE PRECISION AS avg_importance,
                    EXTRACT(EPOCH FROM now() - max(occurred_at))::DOUBLE PRECISION / 86400.0::DOUBLE PRECISION AS last_message_days
                FROM communication_messages
                WHERE ($1::text IS NULL OR account_id = $1)
                  AND channel_kind = 'email'
                  AND local_state = 'active'
                GROUP BY sender
            )
            SELECT sender, message_count, avg_importance, last_message_days
            FROM sender_stats
            WHERE (
                $2::BIGINT IS NULL
                OR message_count < $2
                OR (message_count = $2 AND sender > $3)
            )
            ORDER BY message_count DESC, sender ASC
            LIMIT $4"#,
        )
        .bind(account_id)
        .bind(cursor.as_ref().map(|value| value.message_count))
        .bind(cursor.as_ref().map(|value| value.sender.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(SenderStats {
                sender: row.try_get("sender")?,
                message_count: row.try_get("message_count")?,
                avg_importance: row.try_get("avg_importance")?,
                last_message_days: row.try_get("last_message_days")?,
            });
        }
        let has_more = stats.len() > limit as usize;
        if has_more {
            stats.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            stats.last().map(encode_sender_stats_cursor).transpose()?
        } else {
            None
        };
        Ok(SenderStatsListPage {
            items: stats,
            next_cursor,
            has_more,
        })
    }
}

#[derive(Debug, Error)]
pub enum EmailAnalyticsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid sender stats cursor")]
    InvalidCursor,
}

#[derive(Debug, Deserialize, Serialize)]
struct SenderStatsCursor {
    message_count: i64,
    sender: String,
}

fn encode_sender_stats_cursor(sender: &SenderStats) -> Result<String, EmailAnalyticsError> {
    let cursor = SenderStatsCursor {
        message_count: sender.message_count,
        sender: sender.sender.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| EmailAnalyticsError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_sender_stats_cursor(cursor: &str) -> Result<SenderStatsCursor, EmailAnalyticsError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| EmailAnalyticsError::InvalidCursor)?;
    let cursor: SenderStatsCursor =
        serde_json::from_slice(&bytes).map_err(|_| EmailAnalyticsError::InvalidCursor)?;
    if cursor.message_count < 0 || cursor.sender.trim().is_empty() {
        return Err(EmailAnalyticsError::InvalidCursor);
    }
    Ok(cursor)
}
```

### `backend/src/domains/communications/archive_inspection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/archive_inspection.rs`
- Size bytes / Размер в байтах: `5746`
- Included characters / Включено символов: `5746`
- Truncated / Обрезано: `no`

```rust
use std::io::Cursor;

use serde::Serialize;
use thiserror::Error;
use zip::ZipArchive;

const DEFAULT_MAX_ARCHIVE_BYTES: u64 = 100 * 1024 * 1024;
const DEFAULT_MAX_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;
const DEFAULT_MAX_ENTRIES: usize = 1_000;
const DEFAULT_MAX_DEPTH: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionLimits {
    pub max_archive_bytes: u64,
    pub max_uncompressed_bytes: u64,
    pub max_entries: usize,
    pub max_depth: usize,
}

impl Default for ArchiveInspectionLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: DEFAULT_MAX_ARCHIVE_BYTES,
            max_uncompressed_bytes: DEFAULT_MAX_UNCOMPRESSED_BYTES,
            max_entries: DEFAULT_MAX_ENTRIES,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArchiveInspectionReport {
    pub archive_kind: String,
    pub entry_count: usize,
    pub total_uncompressed_bytes: u64,
    pub has_nested_archive: bool,
    pub entries: Vec<ArchiveEntryInspection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArchiveEntryInspection {
    pub name: String,
    pub normalized_path: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub is_dir: bool,
    pub is_nested_archive: bool,
}

pub fn inspect_zip_bytes(
    bytes: &[u8],
    limits: ArchiveInspectionLimits,
) -> Result<ArchiveInspectionReport, ArchiveInspectionError> {
    if bytes.len() as u64 > limits.max_archive_bytes {
        return Err(ArchiveInspectionError::ArchiveSizeExceeded {
            size: bytes.len() as u64,
            limit: limits.max_archive_bytes,
        });
    }

    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let entry_count = archive.len();
    if entry_count > limits.max_entries {
        return Err(ArchiveInspectionError::EntryCountExceeded {
            count: entry_count,
            limit: limits.max_entries,
        });
    }

    let mut entries = Vec::with_capacity(entry_count);
    let mut total_uncompressed_bytes = 0_u64;
    let mut has_nested_archive = false;

    for index in 0..entry_count {
        let file = archive.by_index(index)?;
        let name = file.name().to_owned();
        let normalized_path = normalize_archive_entry_path(&name)?;
        let depth = path_depth(&normalized_path);
        if depth > limits.max_depth {
            return Err(ArchiveInspectionError::EntryDepthExceeded {
                entry_name: name,
                depth,
                limit: limits.max_depth,
            });
        }

        let uncompressed_size = file.size();
        total_uncompressed_bytes = total_uncompressed_bytes
            .checked_add(uncompressed_size)
            .ok_or(ArchiveInspectionError::UncompressedSizeExceeded {
                total: u64::MAX,
                limit: limits.max_uncompressed_bytes,
            })?;
        if total_uncompressed_bytes > limits.max_uncompressed_bytes {
            return Err(ArchiveInspectionError::UncompressedSizeExceeded {
                total: total_uncompressed_bytes,
                limit: limits.max_uncompressed_bytes,
            });
        }

        let is_nested_archive = is_archive_path(&normalized_path);
        has_nested_archive |= is_nested_archive;
        entries.push(ArchiveEntryInspection {
            name,
            normalized_path,
            compressed_size: file.compressed_size(),
            uncompressed_size,
            is_dir: file.is_dir(),
            is_nested_archive,
        });
    }

    Ok(ArchiveInspectionReport {
        archive_kind: "zip".to_owned(),
        entry_count,
        total_uncompressed_bytes,
        has_nested_archive,
        entries,
    })
}

fn normalize_archive_entry_path(name: &str) -> Result<String, ArchiveInspectionError> {
    let normalized = name.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.starts_with('/') {
        return Err(ArchiveInspectionError::UnsafeEntryPath {
            entry_name: name.to_owned(),
        });
    }

    let mut parts = Vec::new();
    for part in normalized.split('/') {
        let part = part.trim();
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(':') {
            return Err(ArchiveInspectionError::UnsafeEntryPath {
                entry_name: name.to_owned(),
            });
        }
        parts.push(part);
    }

    if parts.is_empty() {
        return Err(ArchiveInspectionError::UnsafeEntryPath {
            entry_name: name.to_owned(),
        });
    }

    Ok(parts.join("/"))
}

fn path_depth(path: &str) -> usize {
    path.split('/').filter(|part| !part.is_empty()).count()
}

fn is_archive_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".zip") || lower.ends_with(".rar") || lower.ends_with(".7z")
}

#[derive(Debug, Error)]
pub enum ArchiveInspectionError {
    #[error("archive size {size} exceeds limit {limit}")]
    ArchiveSizeExceeded { size: u64, limit: u64 },
    #[error("archive entry count {count} exceeds limit {limit}")]
    EntryCountExceeded { count: usize, limit: usize },
    #[error("archive uncompressed size {total} exceeds limit {limit}")]
    UncompressedSizeExceeded { total: u64, limit: u64 },
    #[error("archive entry {entry_name} depth {depth} exceeds limit {limit}")]
    EntryDepthExceeded {
        entry_name: String,
        depth: usize,
        limit: usize,
    },
    #[error("unsafe archive entry path: {entry_name}")]
    UnsafeEntryPath { entry_name: String },
    #[error("zip inspection failed: {0}")]
    Zip(#[from] zip::result::ZipError),
}
```

### `backend/src/domains/communications/attachment_dedup.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/attachment_dedup.rs`
- Size bytes / Размер в байтах: `3533`
- Included characters / Включено символов: `3533`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct DuplicateGroup {
    pub sha256: String,
    pub filenames: Vec<String>,
    pub message_ids: Vec<String>,
    pub count: i64,
}

#[derive(Clone)]
pub struct AttachmentDedupStore {
    pool: PgPool,
}

impl AttachmentDedupStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_duplicates(
        &self,
        limit: i64,
    ) -> Result<Vec<DuplicateGroup>, AttachmentDedupError> {
        let limit = limit.clamp(1, 50);
        let rows = sqlx::query(
            r#"SELECT sha256, array_agg(DISTINCT filename) AS filenames,
                array_agg(DISTINCT a.message_id) AS message_ids, count(*)::BIGINT AS cnt
            FROM communication_attachments a
            JOIN communication_messages m ON m.message_id = a.message_id
            WHERE m.local_state = 'active'
            GROUP BY sha256 HAVING count(*) > 1
            ORDER BY cnt DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut groups = Vec::new();
        for row in rows {
            let filenames: Vec<String> = row
                .try_get::<Vec<Option<String>>, _>("filenames")?
                .into_iter()
                .flatten()
                .collect();
            let message_ids: Vec<String> = row.try_get("message_ids")?;
            groups.push(DuplicateGroup {
                sha256: row.try_get("sha256")?,
                filenames,
                message_ids,
                count: row.try_get("cnt")?,
            });
        }
        Ok(groups)
    }

    pub async fn find_similar_filenames(
        &self,
        limit: i64,
    ) -> Result<Vec<DuplicateGroup>, AttachmentDedupError> {
        let limit = limit.clamp(1, 50);
        let rows = sqlx::query(
            r#"WITH normalized AS (
                SELECT lower(regexp_replace(regexp_replace(regexp_replace(regexp_replace(filename,
                    '_final', '', 'i'), '_v\d+', '', 'i'), '_copy', '', 'i'), '\s*\(\d+\)', '', 'i')) AS base_name,
                    filename, a.message_id, sha256
                FROM communication_attachments a
                JOIN communication_messages m ON m.message_id = a.message_id
                WHERE filename IS NOT NULL
                  AND m.local_state = 'active'
            )
            SELECT base_name, array_agg(DISTINCT filename) AS filenames,
                array_agg(DISTINCT message_id) AS message_ids, count(*)::BIGINT AS cnt
            FROM normalized
            GROUP BY base_name HAVING count(*) > 1
            ORDER BY cnt DESC LIMIT $1"#,
        ).bind(limit).fetch_all(&self.pool).await?;

        let mut groups = Vec::new();
        for row in rows {
            let base_name: String = row.try_get("base_name")?;
            let filenames: Vec<String> = row
                .try_get::<Vec<Option<String>>, _>("filenames")?
                .into_iter()
                .flatten()
                .collect();
            let message_ids: Vec<String> = row.try_get("message_ids")?;
            groups.push(DuplicateGroup {
                sha256: format!("name_group:{base_name}"),
                filenames,
                message_ids,
                count: row.try_get("cnt")?,
            });
        }
        Ok(groups)
    }
}

#[derive(Debug, Error)]
pub enum AttachmentDedupError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/domains/communications/attachment_search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/attachment_search.rs`
- Size bytes / Размер в байтах: `9973`
- Included characters / Включено символов: `9973`
- Truncated / Обрезано: `no`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::communications::storage::{
    AttachmentSafetyScanStatus, CommunicationAttachmentDisposition,
};

#[derive(Clone, Debug)]
pub struct AttachmentSearchQuery<'a> {
    pub account_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub scan_status: Option<&'a str>,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentSearchPage {
    pub items: Vec<AttachmentSearchResult>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentSearchResult {
    pub attachment_id: String,
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub message_subject: String,
    pub sender: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub blob_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub disposition: CommunicationAttachmentDispositionDto,
    pub scan_status: AttachmentSafetyScanStatusDto,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub storage_kind: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationAttachmentDispositionDto {
    Attachment,
    Inline,
    Unknown,
}

impl From<CommunicationAttachmentDisposition> for CommunicationAttachmentDispositionDto {
    fn from(value: CommunicationAttachmentDisposition) -> Self {
        match value {
            CommunicationAttachmentDisposition::Attachment => Self::Attachment,
            CommunicationAttachmentDisposition::Inline => Self::Inline,
            CommunicationAttachmentDisposition::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentSafetyScanStatusDto {
    NotScanned,
    Clean,
    Suspicious,
    Malicious,
    Failed,
}

impl From<AttachmentSafetyScanStatus> for AttachmentSafetyScanStatusDto {
    fn from(value: AttachmentSafetyScanStatus) -> Self {
        match value {
            AttachmentSafetyScanStatus::NotScanned => Self::NotScanned,
            AttachmentSafetyScanStatus::Clean => Self::Clean,
            AttachmentSafetyScanStatus::Suspicious => Self::Suspicious,
            AttachmentSafetyScanStatus::Malicious => Self::Malicious,
            AttachmentSafetyScanStatus::Failed => Self::Failed,
        }
    }
}

#[derive(Clone)]
pub struct AttachmentSearchStore {
    pool: PgPool,
}

impl AttachmentSearchStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search(
        &self,
        request: AttachmentSearchQuery<'_>,
    ) -> Result<AttachmentSearchPage, AttachmentSearchError> {
        let account_id = normalize_optional(request.account_id);
        let query = normalize_optional(request.query);
        let content_type = normalize_optional(request.content_type);
        let scan_status = normalize_optional(request.scan_status)
            .map(validate_scan_status)
            .transpose()?;
        let cursor = normalize_optional(request.cursor)
            .map(decode_attachment_search_cursor)
            .transpose()?;
        let cursor_created_at = cursor.as_ref().map(|cursor| cursor.created_at);
        let cursor_attachment_id = cursor.as_ref().map(|cursor| cursor.attachment_id.as_str());
        let limit = request.limit.clamp(1, 250);
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                m.account_id,
                m.subject AS message_subject,
                m.sender,
                m.occurred_at,
                a.blob_id,
                a.provider_attachment_id,
                a.filename,
                a.content_type,
                a.size_bytes,
                a.sha256,
                a.disposition,
                a.scan_status,
                a.scan_engine,
                a.scan_checked_at,
                a.scan_summary,
                b.storage_kind,
                b.storage_path,
                a.created_at,
                a.updated_at
            FROM communication_attachments a
            JOIN communication_messages m ON m.message_id = a.message_id
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE m.local_state = 'active'
              AND ($1::text IS NULL OR m.account_id = $1)
              AND ($2::text IS NULL OR a.content_type ILIKE '%' || $2 || '%')
              AND ($3::text IS NULL OR a.scan_status = $3)
              AND (
                $4::text IS NULL
                OR NOT EXISTS (
                  SELECT 1
                  FROM unnest(regexp_split_to_array(lower(trim($4)), '\s+')) AS term
                  WHERE term <> ''
                    AND lower(
                      concat_ws(
                        ' ',
                        a.filename,
                        a.content_type,
                        a.sha256,
                        a.provider_attachment_id,
                        m.subject,
                        m.sender
                      )
                    ) NOT LIKE '%' || term || '%'
                )
              )
              AND (
                $5::timestamptz IS NULL
                OR a.created_at < $5
                OR (a.created_at = $5 AND a.attachment_id > $6)
              )
            ORDER BY a.created_at DESC, a.attachment_id ASC
            LIMIT $7
            "#,
        )
        .bind(account_id)
        .bind(content_type)
        .bind(scan_status)
        .bind(query)
        .bind(cursor_created_at)
        .bind(cursor_attachment_id)
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;

        let has_more = rows.len() > limit as usize;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_attachment_search_result)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            items
                .last()
                .map(encode_attachment_search_cursor)
                .transpose()?
        } else {
            None
        };

        Ok(AttachmentSearchPage {
            items,
            next_cursor,
            has_more,
        })
    }
}

fn row_to_attachment_search_result(
    row: PgRow,
) -> Result<AttachmentSearchResult, AttachmentSearchError> {
    let disposition: String = row.try_get("disposition")?;
    let scan_status: String = row.try_get("scan_status")?;
    Ok(AttachmentSearchResult {
        attachment_id: row.try_get("attachment_id")?,
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        message_subject: row.try_get("message_subject")?,
        sender: row.try_get("sender")?,
        occurred_at: row.try_get("occurred_at")?,
        blob_id: row.try_get("blob_id")?,
        provider_attachment_id: row.try_get("provider_attachment_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        size_bytes: row.try_get("size_bytes")?,
        sha256: row.try_get("sha256")?,
        disposition: CommunicationAttachmentDisposition::try_from(disposition.as_str())?.into(),
        scan_status: AttachmentSafetyScanStatus::try_from(scan_status.as_str())?.into(),
        scan_engine: row.try_get("scan_engine")?,
        scan_checked_at: row.try_get("scan_checked_at")?,
        scan_summary: row.try_get("scan_summary")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Deserialize, Serialize)]
struct AttachmentSearchCursor {
    created_at: DateTime<Utc>,
    attachment_id: String,
}

fn encode_attachment_search_cursor(
    item: &AttachmentSearchResult,
) -> Result<String, AttachmentSearchError> {
    let cursor = AttachmentSearchCursor {
        created_at: item.created_at,
        attachment_id: item.attachment_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| AttachmentSearchError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_attachment_search_cursor(
    cursor: &str,
) -> Result<AttachmentSearchCursor, AttachmentSearchError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AttachmentSearchError::InvalidCursor)?;
    let cursor: AttachmentSearchCursor =
        serde_json::from_slice(&bytes).map_err(|_| AttachmentSearchError::InvalidCursor)?;
    if cursor.attachment_id.trim().is_empty() {
        return Err(AttachmentSearchError::InvalidCursor);
    }
    Ok(cursor)
}

fn normalize_optional(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn validate_scan_status(value: &str) -> Result<&str, AttachmentSearchError> {
    AttachmentSafetyScanStatus::try_from(value)?;
    Ok(value)
}

#[derive(Debug, Error)]
pub enum AttachmentSearchError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    CommunicationStorage(
        #[from] crate::domains::communications::storage::CommunicationStorageError,
    ),
    #[error("invalid attachment search cursor")]
    InvalidCursor,
}
```

### `backend/src/domains/communications/blockers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/blockers.rs`
- Size bytes / Размер в байтах: `5465`
- Included characters / Включено символов: `4688`
- Truncated / Обрезано: `no`

```rust
//! # Макошь Mail — Explicit Architecture Blockers
//!
//! Sections that are NOT implemented and WHY.
//! This file serves as authoritative documentation of known gaps.

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ArchitectureBlocker {
    pub section: String,
    pub feature: String,
    pub reason: String,
    pub resolution: String,
}

/// Returns all known blockers with explanations.
pub fn list_blockers() -> Vec<ArchitectureBlocker> {
    vec![
        ArchitectureBlocker {
            section: "§8".into(),
            feature: "Безопасность вложений (sandbox, антивирус)".into(),
            reason: "Conservative heuristic attachment scanning now flags obvious executable payloads, active-content extensions, macro-enabled office files and MIME/extension mismatch as suspicious or malicious during mail projection. Full clean/malware verdicts still require external tools: ClamAV, a containerized sandbox and an OLE macro parser.".into(),
            resolution: "Интегрировать ClamAV как sidecar-контейнер в docker-compose, добавить real attachment_scanner backend and keep heuristic scanning as a prefilter/fallback. Do not mark attachments clean without the real scanner backend.".into(),
        },
        ArchitectureBlocker {
            section: "§12 (крипто-проверка)".into(),
            feature: "Реальная криптографическая верификация подписей (S/MIME, PGP, CAdES, XAdES, ГОСТ)".into(),
            reason: "Требует OpenSSL, GPG, КриптоПро SDK. Это внешние нативные библиотеки, не Rust-крейты. Нужна отдельная интеграционная работа.".into(),
            resolution: "Добавить email_crypto модуль с привязкой к OpenSSL/GPG через FFI или CLI. Сертификаты из Keychain читать через macOS Security framework.".into(),
        },
        ArchitectureBlocker {
            section: "§16-17".into(),
            feature: "Outbox tracking (delivery status, read receipts, bounce detection) и Follow-up engine".into(),
            reason: "Durable outbox tracking, runtime scheduling, account-scoped SMTP sender wiring, Gmail OAuth send scopes, immediate and scheduled Gmail API send, retry/backoff handling, sanitized DSN delivery-status ingestion and MDN read-receipt ingestion exist. Production delivery/read receipt tracking still requires provider callback/webhook wiring and richer delivery UX.".into(),
            resolution: "Connect provider callback/runtime ingestion to the delivery-notification path, and surface delivery status in the user-facing outbox UX without exposing private content in logs or events.".into(),
        },
        ArchitectureBlocker {
            section: "§28-29".into(),
            feature: "Интеграции (Jira, YouTrack, Google Calendar, Apple Notes, Obsidian) и provider-side массовые действия".into(),
            reason: "Каждая интеграция — отдельный коннектор со своим API и аутентификацией. Local bounded bulk actions exist, but provider-side batch mutations, long-running jobs and progress events still require queues.".into(),
            resolution: "Реализовать интеграции как plugin-коннекторы по образцу Telegram/WhatsApp модулей. Provider-side массовые действия — через фоновые задачи projection runner with progress events.".into(),
        },
        ArchitectureBlocker {
            section: "§8.2".into(),
            feature: "Безопасная распаковка архивов (zip slip protection, вложенные архивы, password detection)".into(),
            reason: "Требует потоковой распаковки с защитой от zip bomb/path traversal. Нужна интеграция с zip/rar/7z крейтами и настройка лимитов.".into(),
            resolution: "Добавить email_archive_extractor с лимитами размера/глубины/количества файлов, использовать крейт zip + rar + sevenz-rs.".into(),
        },
        ArchitectureBlocker {
            section: "§9.3".into(),
            feature: "OCR (распознавание текста из PDF-сканов и изображений)".into(),
            reason: "Требует Tesseract OCR или облачного OCR-сервиса. Это тяжёлая зависимость (50+ MB trained data).".into(),
            resolution: "Опциональная фича: добавить tesseract-rs крейт под feature-флагом. Без неё — только текст из PDF/DOCX.".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_blockers_have_reason_and_resolution() {
        for b in list_blockers() {
            assert!(!b.reason.is_empty(), "{} has no reason", b.section);
            assert!(!b.resolution.is_empty(), "{} has no resolution", b.section);
        }
    }
    #[test]
    fn blocker_count_is_stable() {
        assert_eq!(
            list_blockers().len(),
            6,
            "Expected exactly 6 architectural blockers"
        );
    }
}
```

### `backend/src/domains/communications/bulk_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/bulk_actions.rs`
- Size bytes / Размер в байтах: `19974`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;

use crate::platform::events::{EventStore, NewEventEnvelope};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

use super::evidence::link_mail_entity_in_transaction;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BulkMessageAction {
    MarkRead,
    MarkUnread,
    Archive,
    Trash,
    Restore,
    Pin,
    Unpin,
    Important,
    NotImportant,
    AddLabel(String),
    RemoveLabel(String),
    Snooze(DateTime<Utc>),
}

impl BulkMessageAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MarkRead => "mark_read",
            Self::MarkUnread => "mark_unread",
            Self::Archive => "archive",
            Self::Trash => "trash",
            Self::Restore => "restore",
            Self::Pin => "pin",
            Self::Unpin => "unpin",
            Self::Important => "important",
            Self::NotImportant => "not_important",
            Self::AddLabel(_) => "add_label",
            Self::RemoveLabel(_) => "remove_label",
            Self::Snooze(_) => "snooze",
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            Self::MarkRead => "mail.message.read",
            Self::MarkUnread => "mail.message.unread",
            Self::Archive => "mail.message.archived",
            Self::Trash => "mail.message.deleted",
            Self::Restore => "mail.message.restored",
            Self::Pin => "mail.message.pinned",
            Self::Unpin => "mail.message.unpinned",
            Self::Important => "mail.message.important",
            Self::NotImportant => "mail.message.not_important",
            Self::AddLabel(_) => "mail.message.labeled",
            Self::RemoveLabel(_) => "mail.message.unlabeled",
            Self::Snooze(_) => "mail.message.snoozed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BulkMessageActionOutcome {
    pub action: String,
    pub requested_count: usize,
    pub matched_count: usize,
    pub updated_count: usize,
    pub not_found: Vec<String>,
}

pub struct BulkMessageActionStore {
    pool: PgPool,
}

impl BulkMessageActionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn apply(
        &self,
        message_ids: Vec<String>,
        action: BulkMessageAction,
    ) -> Result<BulkMessageActionOutcome, BulkMessageActionError> {
        let message_ids = normalize_message_ids(message_ids)?;
        let mut transaction = self.pool.begin().await?;
        let updated_ids = match &action {
            BulkMessageAction::MarkRead => {
                self.update_workflow_state(&mut transaction, &message_ids, "reviewed")
                    .await?
            }
            BulkMessageAction::MarkUnread => {
                self.update_workflow_state(&mut transaction, &message_ids, "new")
                    .await?
            }
            BulkMessageAction::Archive => {
                self.update_workflow_state(&mut transaction, &message_ids, "archived")
                    .await?
            }
            BulkMessageAction::Trash => self.move_to_trash(&mut transaction, &message_ids).await?,
            BulkMessageAction::Restore => {
                self.restore_from_trash(&mut transaction, &message_ids)
                    .await?
            }
            BulkMessageAction::Pin => {
                self.set_metadata_bool(&mut transaction, &message_ids, "pinned", true)
                    .await?
            }
            BulkMessageAction::Unpin => {
                self.set_metadata_bool(&mut transaction, &message_ids, "pinned", false)
                    .await?
            }
            BulkMessageAction::Important => {
                self.set_metadata_bool(&mut transaction, &message_ids, "important", true)
                    .await?
            }
            BulkMessageAction::NotImportant => {
                self.set_metadata_bool(&mut transaction, &message_ids, "important", false)
                    .await?
            }
            BulkMessageAction::AddLabel(label) => {
                self.add_label(&mut transaction, &message_ids, label)
                    .await?
            }
            BulkMessageAction::RemoveLabel(label) => {
                self.remove_label(&mut transaction, &message_ids, label)
                    .await?
            }
            BulkMessageAction::Snooze(until) => {
                self.snooze(&mut transaction, &message_ids, until).await?
            }
        };
        let outcome = outcome(action.as_str(), &message_ids, updated_ids.clone());

        if !updated_ids.is_empty() {
            self.capture_observation_trail(&mut transaction, &action, &outcome, &updated_ids)
                .await?;
            let event = bulk_message_action_event(&action, &outcome, &updated_ids)?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }
        transaction.commit().await?;

        Ok(outcome)
    }

    async fn update_workflow_state(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        workflow_state: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET workflow_state = $2, projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(workflow_state)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn move_to_trash(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET local_state = 'trash',
                local_state_changed_at = now(),
                local_state_reason = 'bulk_action',
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn restore_from_trash(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET local_state = 'active',
                local_state_changed_at = now(),
                local_state_reason = NULL,
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
        "#,
        )
        .bind(message_ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn set_metadata_bool(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        key: &str,
        value: bool,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        let path = vec![key.to_owned()];
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    $2,
                    to_jsonb($3::boolean),
                    true
                ),
                projected_at = now()
            WHERE message_id = ANY($1)
            RETURNING message_id
            "#,
        )
        .bind(message_ids)
        .bind(path)
        .bind(value)
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn add_label(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        label: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        validate_label(label)?;
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages AS m
            SET message_metadata = jsonb_set(
                    COALESCE(m.message_metadata, '{}'::jsonb),
                    '{labels}',
                    (
                        SELECT COALESCE(jsonb_agg(label_value ORDER BY label_value), '[]'::jsonb)
                        FROM (
                            SELECT DISTINCT label_value
                            FROM (
                                SELECT jsonb_array_elements_text(
                                    CASE
                                        WHEN jsonb_typeof(COALESCE(m.message_metadata, '{}'::jsonb)->'labels') = 'array'
                                        THEN m.message_metadata->'labels'
                                        ELSE '[]'::jsonb
                                    END
                                ) AS label_value
                                UNION ALL
                                SELECT $2::text AS label_value
                            ) labels
                            WHERE trim(label_value) <> ''
                        ) distinct_labels
                    ),
                    true
                ),
                projected_at = now()
            WHERE m.message_id = ANY($1)
            RETURNING m.message_id
        "#,
        )
        .bind(message_ids)
        .bind(label.trim())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn remove_label(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        label: &str,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        validate_label(label)?;
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages AS m
            SET message_metadata = jsonb_set(
                    COALESCE(m.message_metadata, '{}'::jsonb),
                    '{labels}',
                    (
                        SELECT COALESCE(jsonb_agg(label_value ORDER BY label_value), '[]'::jsonb)
                        FROM (
                            SELECT DISTINCT label_value
                            FROM jsonb_array_elements_text(
                                CASE
                                    WHEN jsonb_typeof(COALESCE(m.message_metadata, '{}'::jsonb)->'labels') = 'array'
                                    THEN m.message_metadata->'labels'
                                    ELSE '[]'::jsonb
                                END
                            ) AS label_value
                            WHERE label_value <> $2::text
                              AND trim(label_value) <> ''
                        ) remaining_labels
                    ),
                    true
                ),
                projected_at = now()
            WHERE m.message_id = ANY($1)
            RETURNING m.message_id
        "#,
        )
        .bind(message_ids)
        .bind(label.trim())
        .fetch_all(&mut **transaction)
        .await
        .map_err(BulkMessageActionError::Sqlx)
    }

    async fn snooze(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_ids: &[String],
        until: &DateTime<Utc>,
    ) -> Result<Vec<String>, BulkMessageActionError> {
        sqlx::query_scalar(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    COALESCE(message_metadata, '{}'::jsonb),
                    '{snooze_until}',
                    to_jsonb($2::text),
                    true
                ),
                proje
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/command_service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/command_service.rs`
- Size bytes / Размер в байтах: `49867`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::ai_state::{
    CommunicationAiStateRecord, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
use super::core::ProviderAccount;
use super::drafts::{
    CommunicationDraft, CommunicationDraftError, CommunicationDraftStore, DraftStatus,
    NewCommunicationDraft,
};
use super::flags::{MessageFlags, MessageFlagsError};
use super::folders::{
    CommunicationFolder, CommunicationFolderError, CommunicationFolderStore,
    FolderMessageActionResponse, NewCommunicationFolder, UpdateCommunicationFolder,
};
use super::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, WorkflowState,
};
use super::outbox::{
    CommunicationOutboxError, CommunicationOutboxItem, CommunicationOutboxStatus,
    CommunicationOutboxStore, NewCommunicationOutboxItem, ProviderSendStore,
    ProviderSendStoreError,
};
use super::saved_searches::{
    CommunicationSavedSearch, CommunicationSavedSearchError, CommunicationSavedSearchStore,
    NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use super::storage::{
    AttachmentSafetyScanError, AttachmentSafetyScanRequest, AttachmentSafetyScanner,
    CommunicationStorageError, CommunicationStorageStore, HeuristicAttachmentSafetyScanner,
    ImportedCommunicationAttachment, LocalCommunicationBlobStore, NewCommunicationAttachmentImport,
    NewCommunicationBlob, new_communication_attachment_import_id,
};
use crate::domains::communications::evidence::merge_metadata;
use crate::platform::communications::{DEFAULT_MAIL_SYNC_BLOB_ROOT, OutgoingEmail};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

const MAX_ATTACHMENT_IMPORT_BYTES: usize = 50 * 1024 * 1024;
const LOCAL_IMPORT_ACTOR_ID: &str = "makosh-frontend";

#[derive(Clone)]
pub struct CommunicationCommandService {
    pool: PgPool,
}

impl CommunicationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_draft(
        &self,
        command: CommunicationDraftUpsertCommand,
    ) -> Result<CommunicationDraft, CommunicationCommandServiceError> {
        let metadata = command.metadata.clone().unwrap_or_else(|| json!({}));
        let status = command
            .status
            .as_deref()
            .and_then(DraftStatus::parse)
            .unwrap_or(DraftStatus::Draft);
        let store = CommunicationDraftStore::new(self.pool.clone());
        let existing = store.get(&command.draft_id).await?;
        let operation = if existing.is_some() {
            "draft_update"
        } else {
            "draft_create"
        };
        let observation = self
            .capture_observation(
                "draft mutation",
                "COMMUNICATION_DRAFT",
                json!({
                    "draft_id": command.draft_id.clone(),
                    "account_id": command.account_id.clone(),
                    "persona_id": command.persona_id.clone(),
                    "to_recipient_count": command.to_recipients.len(),
                    "cc_recipient_count": command.cc_recipients.as_ref().map(|items| items.len()).unwrap_or(0),
                    "bcc_recipient_count": command.bcc_recipients.as_ref().map(|items| items.len()).unwrap_or(0),
                    "subject": command.subject.clone(),
                    "has_body_text": !command.body_text.trim().is_empty(),
                    "has_body_html": command.body_html.as_deref().is_some_and(|body| !body.trim().is_empty()),
                    "in_reply_to": command.in_reply_to.clone(),
                    "reference_count": command.references.as_ref().map(|items| items.len()).unwrap_or(0),
                    "status": status.as_str(),
                    "scheduled_send_at": command.scheduled_send_at,
                    "metadata": metadata,
                    "operation": operation,
                }),
                format!("draft://{}/{}", command.draft_id, if existing.is_some() { "update" } else { "create" }),
                json!({
                    "captured_by": "mail_service.upsert_draft",
                    "operation": operation,
                }),
            )
            .await?;

        Ok(store
            .upsert_with_observation(
                &NewCommunicationDraft {
                    draft_id: command.draft_id,
                    account_id: command.account_id,
                    persona_id: command.persona_id,
                    to_recipients: command.to_recipients,
                    cc_recipients: command.cc_recipients.unwrap_or_default(),
                    bcc_recipients: command.bcc_recipients.unwrap_or_default(),
                    subject: command.subject,
                    body_text: command.body_text,
                    body_html: command.body_html,
                    in_reply_to: command.in_reply_to,
                    references: command.references.unwrap_or_default(),
                    status,
                    scheduled_send_at: command.scheduled_send_at,
                    metadata: command.metadata.unwrap_or_else(|| json!({})),
                },
                Some(&observation.observation_id),
                "draft_upsert",
                None,
            )
            .await?)
    }

    pub async fn delete_draft(
        &self,
        draft_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let store = CommunicationDraftStore::new(self.pool.clone());
        let Some(existing_draft) = store.get(draft_id).await? else {
            return Ok(false);
        };
        let observation = self
            .capture_observation(
                "draft delete",
                "COMMUNICATION_DRAFT",
                json!({
                    "draft_id": existing_draft.draft_id,
                    "account_id": existing_draft.account_id,
                    "status": existing_draft.status.as_str(),
                    "scheduled_send_at": existing_draft.scheduled_send_at,
                    "operation": "draft_delete",
                }),
                format!("draft://{draft_id}/delete"),
                json!({
                    "captured_by": "mail_service.delete_draft",
                    "operation": "draft_delete",
                }),
            )
            .await?;

        Ok(store
            .delete_with_observation(
                draft_id,
                Some(&observation.observation_id),
                "draft_delete",
                Some(json!({
                    "status": existing_draft.status.as_str(),
                })),
            )
            .await?)
    }

    pub async fn create_folder(
        &self,
        request: NewCommunicationFolder,
    ) -> Result<CommunicationFolder, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder create",
                "COMMUNICATION_FOLDER",
                json!({
                    "account_id": request.account_id.clone(),
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "color": request.color.clone(),
                    "sort_order": request.sort_order,
                    "operation": "folder_create",
                }),
                "folder://create".to_owned(),
                json!({
                    "captured_by": "mail_service.create_folder",
                    "operation": "folder_create",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .create_with_observation(
                request,
                Some(&observation.observation_id),
                "folder_upsert",
                None,
            )
            .await?)
    }

    pub async fn update_folder(
        &self,
        folder_id: &str,
        request: UpdateCommunicationFolder,
    ) -> Result<Option<CommunicationFolder>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder update",
                "COMMUNICATION_FOLDER",
                json!({
                    "folder_id": folder_id,
                    "account_id": request.account_id.clone(),
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "color": request.color.clone(),
                    "sort_order": request.sort_order,
                    "operation": "folder_update",
                }),
                format!("folder://{folder_id}/update"),
                json!({
                    "captured_by": "mail_service.update_folder",
                    "operation": "folder_update",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .update_with_observation(
                folder_id,
                request,
                Some(&observation.observation_id),
                "folder_upsert",
                None,
            )
            .await?)
    }

    pub async fn delete_folder(
        &self,
        folder_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder delete",
                "COMMUNICATION_FOLDER",
                json!({
                    "folder_id": folder_id,
                    "operation": "folder_delete",
                }),
                format!("folder://{folder_id}/delete"),
                json!({
                    "captured_by": "mail_service.delete_folder",
                    "operation": "folder_delete",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .delete_with_observation(
                folder_id,
                Some(&observation.observation_id),
                "folder_delete",
                None,
            )
            .await?)
    }

    pub async fn copy_message_to_folder(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder message copy",
                "COMMUNICATION_MESSAGE",
                json!({
                    "folder_id": folder_id,
                    "message_id": message_id,
                    "operation": "folder_message_copy",
                }),
                format!("folder://{folder_id}/messages/{message_id}/copy"),
                json!({
                    "captured_by": "mail_service.copy_message_to_folder",
                    "operation": "folder_message_copy",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .copy_message_with_observation(
                folder_id,
                message_id,
                Some(&observation.observation_id),
                "folder_message_transition",
                None,
            )
            .await?)
    }

    pub async fn move_message_to_folder(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder message move",
                "COMMUNICATION_MESSAGE",
                json!({
                    "folder_id": folder_id,
                    "message_id": message_id,
                    "operation": "folder_message_move",
                }),
                format!("folder://{folder_id}/messages/{me
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core.rs`
- Size bytes / Размер в байтах: `979`
- Included characters / Включено символов: `979`
- Truncated / Обрезано: `no`

```rust
mod accounts;
mod checkpoints;
mod errors;
mod models;
mod provider_store;
mod raw_records;
mod rows;
mod secrets;
mod store;
mod validation;

pub use errors::CommunicationIngestionError;
pub use errors::ProviderCredentialError;
pub use models::{
    CommunicationProviderKind, DeletedProviderAccount, EmailProviderKind, IngestionCheckpoint,
    NewIngestionCheckpoint, NewProviderAccount, NewProviderAccountSecretBinding,
    NewRawCommunicationRecord, ProviderAccount, ProviderAccountSecretBinding,
    ProviderAccountSecretPurpose, ProviderAccountUsage, ProviderCredential,
    StoredRawCommunicationRecord,
};
pub use provider_store::CommunicationProviderAccountStore as CommunicationProviderAccountPort;
pub use provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
pub use secrets::ProviderCredentialReader;
pub use store::CommunicationIngestionStore;
pub use store::CommunicationIngestionStore as CommunicationIngestionPort;
```

### `backend/src/domains/communications/core/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/accounts.rs`
- Size bytes / Размер в байтах: `2025`
- Included characters / Включено символов: `2025`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;

use super::errors::CommunicationIngestionError;
use super::models::{
    DeletedProviderAccount, NewProviderAccount, ProviderAccount, ProviderAccountUsage,
};
use super::provider_store::CommunicationProviderAccountStore;
use super::store::CommunicationIngestionStore;
use serde_json::Value;

impl CommunicationIngestionStore {
    pub async fn upsert_provider_account(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .upsert(account)
            .await
    }

    pub async fn provider_account(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .get(account_id)
            .await
    }

    pub async fn list_provider_accounts(
        &self,
    ) -> Result<Vec<ProviderAccount>, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .list()
            .await
    }

    pub async fn update_provider_account_config(
        &self,
        account_id: &str,
        config: &Value,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .update_config(account_id, config)
            .await
    }

    pub async fn provider_account_usage(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccountUsage, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .usage(account_id)
            .await
    }

    pub async fn delete_provider_account_metadata(
        &self,
        account_id: &str,
    ) -> Result<DeletedProviderAccount, CommunicationIngestionError> {
        CommunicationProviderAccountStore::new(self.pool.clone())
            .delete_metadata(account_id)
            .await
    }
}
```

### `backend/src/domains/communications/core/checkpoints.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/checkpoints.rs`
- Size bytes / Размер в байтах: `2673`
- Included characters / Включено символов: `2673`
- Truncated / Обрезано: `no`

```rust
use super::errors::CommunicationIngestionError;
use super::models::{IngestionCheckpoint, NewIngestionCheckpoint};
use super::rows::row_to_checkpoint;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;

impl CommunicationIngestionStore {
    pub async fn save_checkpoint(
        &self,
        checkpoint: &NewIngestionCheckpoint,
    ) -> Result<IngestionCheckpoint, CommunicationIngestionError> {
        checkpoint.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_ingestion_checkpoints (
                account_id,
                stream_id,
                checkpoint,
                updated_at
            )
            VALUES ($1, $2, $3, now())
            ON CONFLICT (account_id, stream_id)
            DO UPDATE SET
                checkpoint = EXCLUDED.checkpoint,
                updated_at = now()
            RETURNING
                account_id,
                stream_id,
                checkpoint,
                updated_at
            "#,
        )
        .bind(checkpoint.account_id.trim())
        .bind(checkpoint.stream_id.trim())
        .bind(&checkpoint.checkpoint)
        .fetch_one(&self.pool)
        .await?;

        row_to_checkpoint(row)
    }

    pub async fn checkpoint(
        &self,
        account_id: &str,
        stream_id: &str,
    ) -> Result<Option<IngestionCheckpoint>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("stream_id", stream_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                stream_id,
                checkpoint,
                updated_at
            FROM communication_ingestion_checkpoints
            WHERE account_id = $1
              AND stream_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(stream_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_checkpoint).transpose()
    }

    pub async fn delete_checkpoint(
        &self,
        account_id: &str,
        stream_id: &str,
    ) -> Result<bool, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("stream_id", stream_id)?;

        let result = sqlx::query(
            r#"
            DELETE FROM communication_ingestion_checkpoints
            WHERE account_id = $1
              AND stream_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(stream_id.trim())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
```

### `backend/src/domains/communications/core/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/errors.rs`
- Size bytes / Размер в байтах: `1908`
- Included characters / Включено символов: `1908`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::communications::CommunicationContractError;
use crate::platform::observations::ObservationStoreError;
use crate::platform::secrets::{SecretKind, SecretReferenceError, SecretResolutionError};

use super::models::ProviderAccountSecretPurpose;

#[derive(Debug, Error)]
pub enum CommunicationIngestionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Contract(#[from] CommunicationContractError),

    #[error("unsupported communication provider kind: {0}")]
    UnsupportedProviderKind(String),

    #[error("unsupported provider account secret purpose: {0}")]
    UnsupportedSecretPurpose(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Debug, Error)]
pub enum ProviderCredentialError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(
        "provider account secret binding not found: account_id={account_id}, secret_purpose={secret_purpose:?}"
    )]
    MissingBinding {
        account_id: String,
        secret_purpose: ProviderAccountSecretPurpose,
    },

    #[error("provider account secret reference metadata was not found: {secret_ref}")]
    MissingSecretReference { secret_ref: String },

    #[error(
        "provider account secret kind is incompatible: secret_ref={secret_ref}, secret_purpose={secret_purpose:?}, secret_kind={secret_kind:?}"
    )]
    IncompatibleSecretKind {
        secret_ref: String,
        secret_purpose: ProviderAccountSecretPurpose,
        secret_kind: SecretKind,
    },
}
```

### `backend/src/domains/communications/core/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/models.rs`
- Size bytes / Размер в байтах: `510`
- Included characters / Включено символов: `510`
- Truncated / Обрезано: `no`

```rust
mod accounts;
mod checkpoints;
mod provider_kind;
mod raw_records;
mod secrets;

pub use crate::platform::communications::{
    CommunicationProviderKind, DeletedProviderAccount, EmailProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, NewRawCommunicationRecord, ProviderAccount,
    ProviderAccountSecretBinding, ProviderAccountSecretPurpose, ProviderAccountUsage,
    ProviderCredential, StoredRawCommunicationRecord,
};
pub use checkpoints::{IngestionCheckpoint, NewIngestionCheckpoint};
```

### `backend/src/domains/communications/core/models/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/models/accounts.rs`
- Size bytes / Размер в байтах: `2289`
- Included characters / Включено символов: `2289`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};

use super::super::errors::CommunicationIngestionError;
use super::super::validation::{validate_non_empty, validate_object};
use super::provider_kind::CommunicationProviderKind;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccountUsage {
    pub raw_record_count: i64,
    pub message_count: i64,
    pub checkpoint_count: i64,
}

impl ProviderAccountUsage {
    pub fn has_retained_evidence(&self) -> bool {
        self.raw_record_count > 0 || self.message_count > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeletedProviderAccount {
    pub account: Option<ProviderAccount>,
    pub unbound_secret_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccount {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub config: Value,
}

impl NewProviderAccount {
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: CommunicationProviderKind,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            provider_kind,
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            config: json!({}),
        }
    }

    pub fn config(mut self, config: Value) -> Self {
        self.config = config;
        self
    }

    pub(in crate::domains::communications::core) fn validate(
        &self,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_object("config", &self.config)
    }
}
```

### `backend/src/domains/communications/core/models/checkpoints.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/models/checkpoints.rs`
- Size bytes / Размер в байтах: `1181`
- Included characters / Включено символов: `1181`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::super::errors::CommunicationIngestionError;
use super::super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IngestionCheckpoint {
    pub account_id: String,
    pub stream_id: String,
    pub checkpoint: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewIngestionCheckpoint {
    pub account_id: String,
    pub stream_id: String,
    pub checkpoint: Value,
}

impl NewIngestionCheckpoint {
    pub fn new(
        account_id: impl Into<String>,
        stream_id: impl Into<String>,
        checkpoint: Value,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            stream_id: stream_id.into(),
            checkpoint,
        }
    }

    pub(in crate::domains::communications::core) fn validate(
        &self,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("stream_id", &self.stream_id)?;
        validate_object("checkpoint", &self.checkpoint)
    }
}
```

### `backend/src/domains/communications/core/models/provider_kind.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/models/provider_kind.rs`
- Size bytes / Размер в байтах: `2501`
- Included characters / Включено символов: `2501`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use super::super::errors::CommunicationIngestionError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationProviderKind {
    Gmail,
    Icloud,
    Imap,
    TelegramUser,
    TelegramBot,
    WhatsappWeb,
    WhatsappBusinessCloud,
    ZoomUser,
    ZoomServerToServer,
    YandexTelemostUser,
}

pub type EmailProviderKind = CommunicationProviderKind;

impl CommunicationProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Icloud => "icloud",
            Self::Imap => "imap",
            Self::TelegramUser => "telegram_user",
            Self::TelegramBot => "telegram_bot",
            Self::WhatsappWeb => "whatsapp_web",
            Self::WhatsappBusinessCloud => "whatsapp_business_cloud",
            Self::ZoomUser => "zoom_user",
            Self::ZoomServerToServer => "zoom_server_to_server",
            Self::YandexTelemostUser => "yandex_telemost_user",
        }
    }

    pub fn is_email(self) -> bool {
        matches!(self, Self::Gmail | Self::Icloud | Self::Imap)
    }

    pub fn is_telegram(self) -> bool {
        matches!(self, Self::TelegramUser | Self::TelegramBot)
    }

    pub fn is_whatsapp(self) -> bool {
        matches!(self, Self::WhatsappWeb | Self::WhatsappBusinessCloud)
    }

    pub fn is_zoom(self) -> bool {
        matches!(self, Self::ZoomUser | Self::ZoomServerToServer)
    }

    pub fn is_yandex_telemost(self) -> bool {
        matches!(self, Self::YandexTelemostUser)
    }
}

impl TryFrom<&str> for CommunicationProviderKind {
    type Error = CommunicationIngestionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "gmail" => Ok(Self::Gmail),
            "icloud" => Ok(Self::Icloud),
            "imap" => Ok(Self::Imap),
            "telegram_user" => Ok(Self::TelegramUser),
            "telegram_bot" => Ok(Self::TelegramBot),
            "whatsapp_web" => Ok(Self::WhatsappWeb),
            "whatsapp_business_cloud" => Ok(Self::WhatsappBusinessCloud),
            "zoom_user" => Ok(Self::ZoomUser),
            "zoom_server_to_server" => Ok(Self::ZoomServerToServer),
            "yandex_telemost_user" => Ok(Self::YandexTelemostUser),
            other => Err(CommunicationIngestionError::UnsupportedProviderKind(
                other.to_owned(),
            )),
        }
    }
}
```

### `backend/src/domains/communications/core/models/raw_records.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/models/raw_records.rs`
- Size bytes / Размер в байтах: `2719`
- Included characters / Включено символов: `2719`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};

use super::super::errors::CommunicationIngestionError;
use super::super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StoredRawCommunicationRecord {
    pub raw_record_id: String,
    pub observation_id: String,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub captured_at: DateTime<Utc>,
    pub payload: Value,
    pub provenance: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewRawCommunicationRecord {
    pub raw_record_id: String,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payload: Value,
    pub provenance: Value,
}

impl NewRawCommunicationRecord {
    pub fn new(
        raw_record_id: impl Into<String>,
        account_id: impl Into<String>,
        record_kind: impl Into<String>,
        provider_record_id: impl Into<String>,
        source_fingerprint: impl Into<String>,
        import_batch_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            raw_record_id: raw_record_id.into(),
            account_id: account_id.into(),
            record_kind: record_kind.into(),
            provider_record_id: provider_record_id.into(),
            source_fingerprint: source_fingerprint.into(),
            import_batch_id: import_batch_id.into(),
            occurred_at: None,
            payload,
            provenance: json!({}),
        }
    }

    pub fn occurred_at(mut self, occurred_at: DateTime<Utc>) -> Self {
        self.occurred_at = Some(occurred_at);
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    pub(in crate::domains::communications::core) fn validate(
        &self,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("raw_record_id", &self.raw_record_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("record_kind", &self.record_kind)?;
        validate_non_empty("provider_record_id", &self.provider_record_id)?;
        validate_non_empty("source_fingerprint", &self.source_fingerprint)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        validate_object("payload", &self.payload)?;
        validate_object("provenance", &self.provenance)
    }
}
```

### `backend/src/domains/communications/core/models/secrets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/models/secrets.rs`
- Size bytes / Размер в байтах: `5440`
- Included characters / Включено символов: `5440`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretReference};

use super::super::errors::CommunicationIngestionError;
use super::super::validation::validate_non_empty;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAccountSecretPurpose {
    OauthToken,
    ImapPassword,
    SmtpPassword,
    TelegramApiHash,
    TelegramSessionKey,
    TelegramBotToken,
    WhatsappWebSessionKey,
    WhatsappBusinessCloudAccessToken,
    WhatsappBusinessCloudAppSecret,
    WhatsappBusinessCloudWebhookVerifyToken,
    ZoomOauthToken,
    ZoomClientSecret,
    ZoomWebhookSecret,
    YandexTelemostOauthToken,
}

impl ProviderAccountSecretPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OauthToken => "oauth_token",
            Self::ImapPassword => "imap_password",
            Self::SmtpPassword => "smtp_password",
            Self::TelegramApiHash => "telegram_api_hash",
            Self::TelegramSessionKey => "telegram_session_key",
            Self::TelegramBotToken => "telegram_bot_token",
            Self::WhatsappWebSessionKey => "whatsapp_web_session_key",
            Self::WhatsappBusinessCloudAccessToken => "whatsapp_business_cloud_access_token",
            Self::WhatsappBusinessCloudAppSecret => "whatsapp_business_cloud_app_secret",
            Self::WhatsappBusinessCloudWebhookVerifyToken => {
                "whatsapp_business_cloud_webhook_verify_token"
            }
            Self::ZoomOauthToken => "zoom_oauth_token",
            Self::ZoomClientSecret => "zoom_client_secret",
            Self::ZoomWebhookSecret => "zoom_webhook_secret",
            Self::YandexTelemostOauthToken => "yandex_telemost_oauth_token",
        }
    }

    pub fn accepts_secret_kind(self, secret_kind: SecretKind) -> bool {
        match self {
            Self::OauthToken => secret_kind == SecretKind::OauthToken,
            Self::ImapPassword | Self::SmtpPassword => {
                matches!(secret_kind, SecretKind::AppPassword | SecretKind::Password)
            }
            Self::TelegramApiHash | Self::TelegramBotToken => secret_kind == SecretKind::ApiToken,
            Self::TelegramSessionKey | Self::WhatsappWebSessionKey => {
                matches!(secret_kind, SecretKind::PrivateKey | SecretKind::Other)
            }
            Self::WhatsappBusinessCloudAccessToken
            | Self::WhatsappBusinessCloudAppSecret
            | Self::WhatsappBusinessCloudWebhookVerifyToken
            | Self::ZoomClientSecret
            | Self::ZoomWebhookSecret => secret_kind == SecretKind::ApiToken,
            Self::ZoomOauthToken | Self::YandexTelemostOauthToken => {
                secret_kind == SecretKind::OauthToken
            }
        }
    }
}

impl TryFrom<&str> for ProviderAccountSecretPurpose {
    type Error = CommunicationIngestionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "oauth_token" => Ok(Self::OauthToken),
            "imap_password" => Ok(Self::ImapPassword),
            "smtp_password" => Ok(Self::SmtpPassword),
            "telegram_api_hash" => Ok(Self::TelegramApiHash),
            "telegram_session_key" => Ok(Self::TelegramSessionKey),
            "telegram_bot_token" => Ok(Self::TelegramBotToken),
            "whatsapp_web_session_key" => Ok(Self::WhatsappWebSessionKey),
            "whatsapp_business_cloud_access_token" => Ok(Self::WhatsappBusinessCloudAccessToken),
            "whatsapp_business_cloud_app_secret" => Ok(Self::WhatsappBusinessCloudAppSecret),
            "whatsapp_business_cloud_webhook_verify_token" => {
                Ok(Self::WhatsappBusinessCloudWebhookVerifyToken)
            }
            "zoom_oauth_token" => Ok(Self::ZoomOauthToken),
            "zoom_client_secret" => Ok(Self::ZoomClientSecret),
            "zoom_webhook_secret" => Ok(Self::ZoomWebhookSecret),
            other => Err(CommunicationIngestionError::UnsupportedSecretPurpose(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderAccountSecretBinding {
    pub account_id: String,
    pub secret_purpose: ProviderAccountSecretPurpose,
    pub secret_ref: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewProviderAccountSecretBinding {
    pub account_id: String,
    pub secret_purpose: ProviderAccountSecretPurpose,
    pub secret_ref: String,
}

impl NewProviderAccountSecretBinding {
    pub fn new(
        account_id: impl Into<String>,
        secret_purpose: ProviderAccountSecretPurpose,
        secret_ref: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            secret_purpose,
            secret_ref: secret_ref.into(),
        }
    }

    pub(in crate::domains::communications::core) fn validate(
        &self,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("secret_ref", &self.secret_ref)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredential {
    pub binding: ProviderAccountSecretBinding,
    pub reference: SecretReference,
    pub secret: ResolvedSecret,
}
```

### `backend/src/domains/communications/core/provider_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/provider_store.rs`
- Size bytes / Размер в байтах: `39197`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use super::errors::CommunicationIngestionError;
use super::models::{
    DeletedProviderAccount, NewProviderAccount, NewProviderAccountSecretBinding, ProviderAccount,
    ProviderAccountSecretBinding, ProviderAccountSecretPurpose, ProviderAccountUsage,
};
use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderAccountLookupPort, ProviderAccountPortError,
    ProviderSecretBindingCommandPort, ProviderSecretBindingLookupPort,
    ProviderSecretBindingPortError,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, link_domain_entity_in_transaction,
};

#[derive(Clone)]
pub struct CommunicationProviderAccountStore {
    pool: PgPool,
}

impl CommunicationProviderAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_runtime_account(
        &self,
        account_id: impl Into<String>,
        provider_kind: &str,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        config: serde_json::Value,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        let provider_kind =
            crate::domains::communications::core::CommunicationProviderKind::try_from(
                provider_kind,
            )?;
        self.upsert(
            &NewProviderAccount::new(account_id, provider_kind, display_name, external_account_id)
                .config(config),
        )
        .await
    }

    pub async fn upsert(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        self.upsert_with_origin(
            account,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_accounts.upsert",
        )
        .await
    }

    pub async fn restore(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        self.upsert_with_origin(
            account,
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_provider_account",
        )
        .await
    }

    pub async fn upsert_with_origin(
        &self,
        account: &NewProviderAccount,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        validate_provider_account(account)?;
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_ACCOUNT",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account.account_id.trim(),
                    "provider_kind": account.provider_kind.as_str(),
                    "display_name": account.display_name.trim(),
                    "external_account_id": account.external_account_id.trim(),
                    "config": account.config,
                    "action": "upsert_communication_provider_account",
                }),
                format!(
                    "communication-provider-account://{}",
                    account.account_id.trim()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "upsert_communication_provider_account",
                "provider_kind": account.provider_kind.as_str(),
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_accounts (
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = now()
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account.account_id.trim())
        .bind(account.provider_kind.as_str())
        .bind(account.display_name.trim())
        .bind(account.external_account_id.trim())
        .bind(&account.config)
        .fetch_one(&mut *transaction)
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            VaultOwnedEntityLink {
                observation_id: observation.observation_id.clone(),
                domain: "vault",
                entity_kind: "communication_provider_account",
                entity_id: account.account_id.trim().to_owned(),
                relationship_kind: "upsert",
                base_metadata: json!({
                    "provider_kind": account.provider_kind.as_str(),
                    "external_account_id": account.external_account_id.trim(),
                }),
                extra_metadata: None,
            },
        )
        .await?;
        transaction.commit().await?;

        row_to_provider_account(row)
    }

    pub async fn get(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_account).transpose()
    }

    pub async fn list(&self) -> Result<Vec<ProviderAccount>, CommunicationIngestionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            ORDER BY provider_kind ASC, display_name ASC, account_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_account).collect()
    }

    pub async fn update_config(
        &self,
        account_id: &str,
        config: &serde_json::Value,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        self.update_config_with_origin(
            account_id,
            config,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_accounts.update_config",
            "update_config",
        )
        .await
    }

    pub async fn update_whatsapp_lifecycle_state(
        &self,
        account_id: &str,
        lifecycle_state: &str,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        validate_non_empty_field("lifecycle_state", lifecycle_state)?;
        sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET config = jsonb_set(
                    COALESCE(config, '{}'::jsonb),
                    '{lifecycle_state}',
                    to_jsonb($2::text),
                    true
                ),
                updated_at = now()
            WHERE account_id = $1
              AND provider_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
            "#,
        )
        .bind(account_id)
        .bind(lifecycle_state)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_logged_out(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        let Some(current) = self.get(account_id).await? else {
            return Ok(None);
        };
        let mut config = current.config;
        let config_object = config
            .as_object_mut()
            .ok_or(CommunicationIngestionError::NonObjectJson("config"))?;
        config_object.insert("auth_state".to_owned(), json!("logged_out"));
        config_object.insert("logged_out_at".to_owned(), json!(chrono::Utc::now()));

        self.update_config_with_origin(
            account_id,
            &config,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_accounts.mark_logged_out",
            "logout",
        )
        .await
    }

    pub async fn update_config_with_origin(
        &self,
        account_id: &str,
        config: &serde_json::Value,
        origin_kind: ObservationOriginKind,
        actor: &str,
        action: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        if !config.is_object() {
            return Err(CommunicationIngestionError::NonObjectJson("config"));
        }

        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id.trim(),
                    "config": config,
                    "action": action,
                }),
                format!(
                    "communication-provider-account://{}/config",
                    account_id.trim()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": action,
            })),
        )
        .await?;

        let row = sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET config = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(config)
        .fetch_optional(&mut *transaction)
        .await?;

        if row.is_some() {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_account",
                    entity_id: account_id.trim().to_owned(),
                    relationship_kind: "config_update",
                    base_metadata: json!({
                        "account_id": account_id.trim(),
                        "action": action,
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
            transaction.commit().await?;
        } else {
            transaction.rollback().await?;
        }

        row.map(row_to_provider_account).transpose()
    }

    pub async fn usage(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccountUsage, CommunicationIngestionEr
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/core/raw_records.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/raw_records.rs`
- Size bytes / Размер в байтах: `6468`
- Included characters / Включено символов: `6468`
- Truncated / Обрезано: `no`

```rust
use super::errors::CommunicationIngestionError;
use super::models::{NewRawCommunicationRecord, StoredRawCommunicationRecord};
use super::rows::row_to_raw_record;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;
use crate::platform::communications::{
    CommunicationRawRecordCommandPort, CommunicationRawRecordPortFuture,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};
use chrono::Utc;
use serde_json::json;

impl CommunicationIngestionStore {
    pub async fn record_raw_source(
        &self,
        record: &NewRawCommunicationRecord,
    ) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
        record.validate()?;

        let mut transaction = self.pool.begin().await?;
        let existing = sqlx::query(raw_record_by_provider_identity_sql())
            .bind(record.account_id.trim())
            .bind(record.record_kind.trim())
            .bind(record.provider_record_id.trim())
            .fetch_optional(&mut *transaction)
            .await?;
        if let Some(row) = existing {
            let stored = row_to_raw_record(row)?;
            transaction.commit().await?;
            return Ok(stored);
        }

        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &raw_record_observation(record),
        )
        .await?;
        let inserted = sqlx::query(
            r#"
            INSERT INTO communication_raw_records (
                raw_record_id,
                observation_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                payload,
                provenance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (account_id, record_kind, provider_record_id)
            DO NOTHING
            RETURNING
                raw_record_id,
                observation_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            "#,
        )
        .bind(record.raw_record_id.trim())
        .bind(&observation.observation_id)
        .bind(record.account_id.trim())
        .bind(record.record_kind.trim())
        .bind(record.provider_record_id.trim())
        .bind(record.source_fingerprint.trim())
        .bind(record.import_batch_id.trim())
        .bind(record.occurred_at)
        .bind(&record.payload)
        .bind(&record.provenance)
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(row) = inserted {
            let stored = row_to_raw_record(row)?;
            transaction.commit().await?;
            return Ok(stored);
        }

        let row = sqlx::query(raw_record_by_provider_identity_sql())
            .bind(record.account_id.trim())
            .bind(record.record_kind.trim())
            .bind(record.provider_record_id.trim())
            .fetch_one(&mut *transaction)
            .await?;

        let stored = row_to_raw_record(row)?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn raw_record(
        &self,
        raw_record_id: &str,
    ) -> Result<Option<StoredRawCommunicationRecord>, CommunicationIngestionError> {
        validate_non_empty("raw_record_id", raw_record_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                raw_record_id,
                observation_id,
                account_id,
                record_kind,
                provider_record_id,
                source_fingerprint,
                import_batch_id,
                occurred_at,
                captured_at,
                payload,
                provenance
            FROM communication_raw_records
            WHERE raw_record_id = $1
            "#,
        )
        .bind(raw_record_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_raw_record).transpose()
    }
}

impl CommunicationRawRecordCommandPort for CommunicationIngestionStore {
    fn record_raw_source<'a>(
        &'a self,
        record: &'a NewRawCommunicationRecord,
    ) -> CommunicationRawRecordPortFuture<'a, StoredRawCommunicationRecord> {
        Box::pin(async move {
            CommunicationIngestionStore::record_raw_source(self, record)
                .await
                .map_err(|error| {
                    crate::platform::communications::ProviderCommunicationMessagePortError::InvalidRequest(
                        error.to_string(),
                    )
                })
        })
    }
}

fn raw_record_by_provider_identity_sql() -> &'static str {
    r#"
    SELECT
        raw_record_id,
        observation_id,
        account_id,
        record_kind,
        provider_record_id,
        source_fingerprint,
        import_batch_id,
        occurred_at,
        captured_at,
        payload,
        provenance
    FROM communication_raw_records
    WHERE account_id = $1
      AND record_kind = $2
      AND provider_record_id = $3
    "#
}

fn raw_record_observation(record: &NewRawCommunicationRecord) -> NewObservation {
    let kind_code = if record.record_kind.contains("attachment") {
        "COMMUNICATION_ATTACHMENT"
    } else {
        "COMMUNICATION_MESSAGE"
    };
    let observed_at = record.occurred_at.unwrap_or_else(Utc::now);
    NewObservation::new(
        kind_code,
        ObservationOriginKind::VaultSource,
        observed_at,
        record.payload.clone(),
        format!(
            "communication://{}/{}/{}",
            record.account_id.trim(),
            record.record_kind.trim(),
            record.provider_record_id.trim()
        ),
    )
    .confidence(1.0)
    .provenance(json!({
        "communication_raw_record": true,
        "raw_record_id": record.raw_record_id.trim(),
        "account_id": record.account_id.trim(),
        "record_kind": record.record_kind.trim(),
        "provider_record_id": record.provider_record_id.trim(),
        "import_batch_id": record.import_batch_id.trim(),
        "source_fingerprint": record.source_fingerprint.trim(),
        "raw_provenance": record.provenance
    }))
}
```

### `backend/src/domains/communications/core/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/core/rows.rs`
- Size bytes / Размер в байтах: `2499`
- Included characters / Включено символов: `2499`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::CommunicationIngestionError;
use super::models::{
    CommunicationProviderKind, IngestionCheckpoint, ProviderAccount, ProviderAccountSecretBinding,
    ProviderAccountSecretPurpose, StoredRawCommunicationRecord,
};

pub(super) fn row_to_provider_account(
    row: PgRow,
) -> Result<ProviderAccount, CommunicationIngestionError> {
    let provider_kind =
        CommunicationProviderKind::try_from(row.try_get::<String, _>("provider_kind")?.as_str())?;

    Ok(ProviderAccount {
        account_id: row.try_get("account_id")?,
        provider_kind,
        display_name: row.try_get("display_name")?,
        external_account_id: row.try_get("external_account_id")?,
        config: row.try_get("config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_raw_record(
    row: PgRow,
) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
    Ok(StoredRawCommunicationRecord {
        raw_record_id: row.try_get("raw_record_id")?,
        observation_id: row.try_get("observation_id")?,
        account_id: row.try_get("account_id")?,
        record_kind: row.try_get("record_kind")?,
        provider_record_id: row.try_get("provider_record_id")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        import_batch_id: row.try_get("import_batch_id")?,
        occurred_at: row.try_get("occurred_at")?,
        captured_at: row.try_get("captured_at")?,
        payload: row.try_get("payload")?,
        provenance: row.try_get("provenance")?,
    })
}

pub(super) fn row_to_checkpoint(
    row: PgRow,
) -> Result<IngestionCheckpoint, CommunicationIngestionError> {
    Ok(IngestionCheckpoint {
        account_id: row.try_get("account_id")?,
        stream_id: row.try_get("stream_id")?,
        checkpoint: row.try_get("checkpoint")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_secret_binding(
    row: PgRow,
) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
    let secret_purpose = ProviderAccountSecretPurpose::try_from(
        row.try_get::<String, _>("secret_purpose")?.as_str(),
    )?;

    Ok(ProviderAccountSecretBinding {
        account_id: row.try_get("account_id")?,
        secret_purpose,
        secret_ref: row.try_get("secret_ref")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```
