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

- Chunk ID / ID чанка: `077-source-backend-part-057`
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

### `backend/src/workflows/telegram_media_storage.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/telegram_media_storage.rs`
- Size bytes / Размер в байтах: `13428`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::path::Path;

use serde_json::{Value, json};
use sqlx::PgPool;
use thiserror::Error;

use crate::domains::communications::storage::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanStatus, AttachmentSafetyScanner,
    CommunicationAttachmentDisposition, CommunicationBlobMetadataPort,
    ImportedCommunicationAttachment, LocalCommunicationBlobPort, NewCommunicationAttachment,
    NewCommunicationBlob, NoopAttachmentSafetyScanner,
};
use crate::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramMediaDownloadData {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) tdlib_file_id: i64,
    pub(crate) provider_attachment_id: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
}

impl TelegramMediaDownloadData {
    pub(crate) fn provider_attachment_id(&self) -> String {
        self.provider_attachment_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("tdlib-file:{}", self.tdlib_file_id))
    }

    pub(crate) fn content_type(&self) -> String {
        self.content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "application/octet-stream".to_owned())
    }

    pub(crate) fn filename(&self) -> Option<String> {
        self.filename
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramDownloadedFileData {
    pub(crate) file_id: i64,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) expected_size_bytes: Option<i64>,
    pub(crate) local_path: Option<String>,
    pub(crate) is_downloading_active: bool,
    pub(crate) is_downloading_completed: bool,
    pub(crate) downloaded_size_bytes: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramAttachmentAnchor {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramMediaDownloadProjection {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) tdlib_file_id: i64,
    pub(crate) local_path: Option<String>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) expected_size_bytes: Option<i64>,
    pub(crate) downloaded_size_bytes: Option<i64>,
    pub(crate) is_downloading_active: bool,
    pub(crate) is_downloading_completed: bool,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) scan_status: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramProviderMediaCommand {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) command_kind: String,
    pub(crate) provider_chat_id: String,
    pub(crate) payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramPreparedMediaSendRequest {
    pub(crate) command_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) media_type: String,
    pub(crate) local_path: String,
    pub(crate) caption: Option<String>,
    pub(crate) filename: Option<String>,
}

#[derive(Debug, Error)]
pub(crate) enum TelegramMediaStorageError {
    #[error("invalid Telegram media request: {0}")]
    InvalidRequest(String),

    #[error("Telegram media runtime error: {0}")]
    Runtime(String),

    #[error("Telegram media storage error: {0}")]
    Storage(String),
}

pub(crate) async fn persist_downloaded_media(
    pool: PgPool,
    request: &TelegramMediaDownloadData,
    file: &TelegramDownloadedFileData,
    anchor: Option<TelegramAttachmentAnchor>,
    blob_root: &Path,
) -> Result<TelegramMediaDownloadProjection, TelegramMediaStorageError> {
    let mut response = TelegramMediaDownloadProjection {
        account_id: request.account_id.trim().to_owned(),
        provider_chat_id: request.provider_chat_id.trim().to_owned(),
        provider_message_id: request.provider_message_id.trim().to_owned(),
        runtime_kind: "tdlib_qr_authorized".to_owned(),
        status: if file.is_downloading_completed {
            "downloaded".to_owned()
        } else if file.is_downloading_active {
            "downloading".to_owned()
        } else {
            "remote".to_owned()
        },
        tdlib_file_id: file.file_id,
        local_path: file.local_path.clone(),
        size_bytes: file.size_bytes,
        expected_size_bytes: file.expected_size_bytes,
        downloaded_size_bytes: file.downloaded_size_bytes,
        is_downloading_active: file.is_downloading_active,
        is_downloading_completed: file.is_downloading_completed,
        attachment_id: None,
        blob_id: None,
        scan_status: None,
    };

    if !file.is_downloading_completed {
        return Ok(response);
    }

    let local_path = file.local_path.as_deref().ok_or_else(|| {
        TelegramMediaStorageError::Runtime(
            "TDLib reported a completed download without a local file path".to_owned(),
        )
    })?;
    let bytes = tokio::fs::read(local_path).await.map_err(|error| {
        TelegramMediaStorageError::Runtime(format!(
            "failed to read downloaded Telegram file `{local_path}`: {error}"
        ))
    })?;
    let blob_store = LocalCommunicationBlobPort::new(blob_root);
    let local_blob = blob_store.put_blob(&bytes).await.map_err(|error| {
        TelegramMediaStorageError::Storage(format!("failed to store Telegram media blob: {error}"))
    })?;
    let mail_store = CommunicationBlobMetadataPort::new(pool);
    let content_type = request.content_type();
    let stored_blob = mail_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type(content_type.clone()),
        )
        .await
        .map_err(|error| {
            TelegramMediaStorageError::Storage(format!(
                "failed to record Telegram media blob: {error}"
            ))
        })?;
    let scanner = NoopAttachmentSafetyScanner;
    let provider_attachment_id = request.provider_attachment_id();
    let filename = request.filename();
    let scan_report = scanner
        .scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: &provider_attachment_id,
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &local_blob.sha256,
            storage_kind: &local_blob.storage_kind,
            storage_path: &local_blob.storage_path,
            bytes: &bytes,
        })
        .map_err(|error| {
            TelegramMediaStorageError::Storage(format!("Telegram media scan failed: {error}"))
        })?;
    let anchor = anchor.ok_or_else(|| {
        TelegramMediaStorageError::InvalidRequest(
            "completed Telegram media download requires a communication message anchor".to_owned(),
        )
    })?;
    let mut attachment = NewCommunicationAttachment::new(
        anchor.message_id,
        anchor.raw_record_id,
        stored_blob.blob_id.clone(),
        provider_attachment_id,
        content_type,
        local_blob.size_bytes,
        local_blob.sha256.clone(),
    )
    .disposition(CommunicationAttachmentDisposition::Attachment)
    .scan_report(scan_report);
    if let Some(filename) = filename {
        attachment = attachment.filename(filename);
    }
    let stored_attachment = mail_store
        .upsert_attachment(&attachment)
        .await
        .map_err(|error| {
            TelegramMediaStorageError::Storage(format!(
                "failed to record Telegram media attachment: {error}"
            ))
        })?;

    response.attachment_id = Some(stored_attachment.attachment_id);
    response.blob_id = Some(stored_blob.blob_id);
    response.scan_status = Some(stored_attachment.scan_status.as_str().to_owned());
    Ok(response)
}

pub(crate) async fn media_send_request(
    pool: &PgPool,
    command: &TelegramProviderMediaCommand,
) -> Result<TelegramPreparedMediaSendRequest, TelegramMediaStorageError> {
    let media_type = payload_string(command, "media_type")?;
    validate_media_type(&media_type)?;
    let attachment_id = payload_optional_string(command, "attachment_id");
    let blob_id = payload_optional_string(command, "blob_id");
    if attachment_id.is_none() && blob_id.is_none() {
        return Err(TelegramMediaStorageError::InvalidRequest(
            "send_media command requires attachment_id or blob_id".to_owned(),
        ));
    }

    let mail_store = CommunicationBlobMetadataPort::new(pool.clone());
    let imported = if let Some(attachment_id) = attachment_id.as_deref() {
        mail_store
            .imported_attachment_by_id(attachment_id)
            .await
            .map_err(|error| TelegramMediaStorageError::Storage(error.to_string()))?
            .ok_or_else(|| {
                TelegramMediaStorageError::InvalidRequest(format!(
                    "attachment import `{attachment_id}` was not found"
                ))
            })?
    } else {
        let blob_id = blob_id.as_deref().expect("blob_id checked above");
        if let Some(imported) = mail_store
            .imported_attachment_by_blob_id(blob_id)
            .await
            .map_err(|error| TelegramMediaStorageError::Storage(error.to_string()))?
        {
            imported
        } else {
            let blob = mail_store
                .blob_by_id(blob_id)
                .await
                .map_err(|error| TelegramMediaStorageError::Storage(error.to_string()))?
                .ok_or_else(|| {
                    TelegramMediaStorageError::InvalidRequest(format!(
                        "blob `{blob_id}` was not found"
                    ))
                })?;
            ImportedCommunicationAttachment {
                attachment_id: format!("blob:{blob_id}"),
                account_id: Some(command.account_id.clone()),
                channel_kind: Some("telegram".to_owned()),
                blob_id: blob.blob_id,
                filename: payload_optional_string(command, "filename"),
                content_type: blob
                    .content_type
                    .unwrap_or_else(|| "application/octet-stream".to_owned()),
                size_bytes: blob.size_bytes,
                sha256: blob.sha256,
                source_kind: "blob_reuse".to_owned(),
                imported_by: "telegram-outbox-worker".to_owned(),
                scan_status: AttachmentSafetyScanStatus::NotScanned,
                scan_engine: None,
                scan_checked_at: None,
                scan_summary: None,
                scan_metadata: json!({}),
                metadata: json!({}),
                storage_kind: blob.storage_kind,
                storage_path: blob.storage_path,
                created_at: blob.created_at,
                updated_at: blob.created_at,
            }
        }
    };

    if imported.storage_kind != "local_fs" {
        return Err(TelegramMediaStorageError::InvalidRequest(
            "send_media requires a local filesystem blob".to_owned(),
        ));
    }
    if imported.scan_status.as_str() == "malicious" {
        return Err(TelegramMediaStorageError::InvalidRequest(
            "send_media rejected a malicious attachment import".to_owned(),
        ));
    }
    let local_path = std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
        .join(&imported.storage_path)
        .to_string_lossy()
        .into_owned();

    Ok(TelegramPreparedMediaSendRequest {
        comm
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/workflows/workflow_action_person_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/workflow_action_person_projection.rs`
- Size bytes / Размер в байтах: `1936`
- Included characters / Включено символов: `1936`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::persons::api::{PersonProjectionError, PersonProjectionPort};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationPort};

pub(crate) async fn create_person_projection_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    email: &str,
    display_name: Option<&str>,
    message: Option<&ProjectedMessage>,
) -> Result<String, PersonProjectionError> {
    let (person, identity_id) =
        PersonProjectionPort::upsert_email_person_in_transaction(transaction, email).await?;
    let projection_observation_id = if let Some(message) = message {
        message.observation_id.clone()
    } else {
        ObservationPort::capture_in_transaction(
            transaction,
            &NewObservation::new(
                "PERSON_MUTATION",
                ObservationOriginKind::Manual,
                chrono::Utc::now(),
                serde_json::json!({
                    "command_id": command_id,
                    "event_id": event_id,
                    "email": email,
                    "display_name": display_name,
                    "operation": "workflow_action_create_person",
                }),
                format!("workflow-action://create-person/{command_id}"),
            )
            .provenance(serde_json::json!({
                "captured_by": "workflows.create_person_projection_in_transaction",
                "workflow_action": "create_person",
            })),
        )
        .await?
        .observation_id
    };
    PersonProjectionPort::link_email_person_projection_in_transaction(
        transaction,
        &projection_observation_id,
        &person,
        &identity_id,
        email,
        "workflow_action_projection",
    )
    .await?;
    Ok(person.person_id)
}
```

### `backend/src/workflows/yandex_telemost_calendar_matching.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/yandex_telemost_calendar_matching.rs`
- Size bytes / Размер в байтах: `8161`
- Included characters / Включено символов: `8161`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::core::{CalendarCoreError, EventParticipantPort, EventRelationPort};
use crate::domains::calendar::events::{CalendarError, CalendarEventQueryPort};
use crate::platform::events::bus::yandex_telemost_event_types;
use crate::platform::events::{EventEnvelope, EventStoreError, StoredEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationStoreError,
};

pub const YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER: &str = "yandex_telemost_calendar_matching";
pub const YANDEX_TELEMOST_CALENDAR_MATCHING_PROJECTION: &str = "yandex_telemost_calendar_matching";
pub const YANDEX_TELEMOST_CALENDAR_RELATION_TYPE: &str = "conference_call";
const YANDEX_TELEMOST_CALENDAR_PARTICIPANT_SOURCE: &str = "yandex_telemost_cohost_observed";

#[derive(Debug, Deserialize)]
struct TelemostCohostObservation {
    email: String,
}

#[derive(Debug, Error)]
pub enum YandexTelemostCalendarMatchingWorkflowError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    CalendarCore(#[from] CalendarCoreError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),
}

pub async fn project_yandex_telemost_calendar_matching_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_yandex_telemost_calendar_matching(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_yandex_telemost_calendar_matching(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), YandexTelemostCalendarMatchingWorkflowError> {
    if !supports_yandex_telemost_calendar_matching_event(&event.event_type) {
        return Ok(());
    }

    if event.event_type == yandex_telemost_event_types::COHOSTS_OBSERVED {
        return project_yandex_telemost_cohosts_into_calendar(pool, event).await;
    }

    let conference = event
        .payload
        .get("conference")
        .ok_or(YandexTelemostCalendarMatchingWorkflowError::MissingPayloadField("conference"))?;
    let conference_id = required_nested_string(conference, "id")?;
    let join_url = required_nested_string(conference, "join_url")?;

    let event_store = CalendarEventQueryPort::new(pool.clone());
    let Some(calendar_event) = event_store
        .find_yandex_telemost_conference_match(Some(join_url), conference_id)
        .await?
    else {
        return Ok(());
    };

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::LocalRuntime,
                event.occurred_at,
                json!({
                    "event_id": calendar_event.event_id,
                    "matched_entity_type": "telemost_conference",
                    "matched_entity_id": conference_id,
                    "conference_id": conference_id,
                    "join_url": join_url,
                    "source_event_id": event.event_id,
                    "match_strategy": "telemost_conference_url",
                }),
                format!(
                    "calendar-event://{}/matches/telemost-conference/{}",
                    calendar_event.event_id, conference_id
                ),
            )
            .provenance(json!({
                "captured_by": "yandex_telemost_calendar_matching",
                "event_id": event.event_id,
                "event_type": event.event_type,
            })),
        )
        .await?;

    EventRelationPort::new(pool.clone())
        .link_with_observation(
            &calendar_event.event_id,
            "telemost_conference",
            conference_id,
            YANDEX_TELEMOST_CALENDAR_RELATION_TYPE,
            event.event_type.as_str(),
            Some(&observation.observation_id),
        )
        .await?;

    Ok(())
}

async fn project_yandex_telemost_cohosts_into_calendar(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), YandexTelemostCalendarMatchingWorkflowError> {
    let conference_id = required_string(&event.payload, "conference_id")?;
    let cohosts = event
        .payload
        .get("cohosts")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let cohosts: Vec<TelemostCohostObservation> = serde_json::from_value(cohosts)?;

    if cohosts.is_empty() {
        return Ok(());
    }

    let event_store = CalendarEventQueryPort::new(pool.clone());
    let Some(calendar_event) = event_store
        .find_yandex_telemost_conference_match(None, conference_id)
        .await?
    else {
        return Ok(());
    };

    let participant_store = EventParticipantPort::new(pool.clone());
    let existing = participant_store.list(&calendar_event.event_id).await?;
    let mut known_emails = existing
        .into_iter()
        .map(|participant| participant.email.trim().to_ascii_lowercase())
        .filter(|email| !email.is_empty())
        .collect::<HashSet<_>>();

    let observation_store = ObservationPort::new(pool.clone());
    for cohost in cohosts {
        let email = cohost.email.trim().to_ascii_lowercase();
        if email.is_empty() || known_emails.contains(&email) {
            continue;
        }

        let observation = observation_store
            .capture(
                &NewObservation::new(
                    "CALENDAR_EVENT",
                    ObservationOriginKind::LocalRuntime,
                    event.occurred_at,
                    json!({
                        "event_id": calendar_event.event_id,
                        "conference_id": conference_id,
                        "participant_email": email,
                        "participant_role": "attendee",
                        "provider_role": "cohost",
                        "source_event_id": event.event_id,
                        "source_kind": "telemost_cohost",
                    }),
                    format!(
                        "calendar-event://{}/participants/telemost-cohost/{}",
                        calendar_event.event_id, email
                    ),
                )
                .provenance(json!({
                    "captured_by": "yandex_telemost_calendar_matching",
                    "event_id": event.event_id,
                    "event_type": event.event_type,
                })),
            )
            .await?;

        let _ = participant_store
            .add_with_observation(
                &calendar_event.event_id,
                &email,
                None,
                Some("attendee"),
                None,
                None,
                YANDEX_TELEMOST_CALENDAR_PARTICIPANT_SOURCE,
                Some(&observation.observation_id),
            )
            .await?;
        known_emails.insert(email);
    }

    Ok(())
}

pub fn supports_yandex_telemost_calendar_matching_event(event_type: &str) -> bool {
    matches!(
        event_type,
        yandex_telemost_event_types::CONFERENCE_CREATED
            | yandex_telemost_event_types::CONFERENCE_OBSERVED
            | yandex_telemost_event_types::CONFERENCE_UPDATED
            | yandex_telemost_event_types::COHOSTS_OBSERVED
    )
}

fn required_string<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, YandexTelemostCalendarMatchingWorkflowError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .ok_or(YandexTelemostCalendarMatchingWorkflowError::MissingPayloadField(field))
}

fn required_nested_string<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, YandexTelemostCalendarMatchingWorkflowError> {
    required_string(value, field)
}
```

### `backend/src/workflows/zoom_calendar_matching.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/zoom_calendar_matching.rs`
- Size bytes / Размер в байтах: `5041`
- Included characters / Включено символов: `5041`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::core::{CalendarCoreError, EventRelationPort};
use crate::domains::calendar::events::{CalendarError, CalendarEventQueryPort};
use crate::platform::events::bus::zoom_event_types;
use crate::platform::events::{EventEnvelope, EventStoreError, StoredEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationStoreError,
};

pub const ZOOM_CALENDAR_MATCHING_CONSUMER: &str = "zoom_calendar_matching";
pub const ZOOM_CALENDAR_MATCHING_PROJECTION: &str = "zoom_calendar_matching";
pub const ZOOM_CALENDAR_RELATION_TYPE: &str = "conference_call";

#[derive(Debug, Error)]
pub enum ZoomCalendarMatchingWorkflowError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    CalendarCore(#[from] CalendarCoreError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),
}

pub async fn project_zoom_calendar_matching_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_zoom_calendar_matching(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_zoom_calendar_matching(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomCalendarMatchingWorkflowError> {
    if event.event_type != zoom_event_types::MEETING_OBSERVED {
        return Ok(());
    }

    let call_id = required_subject_string(&event.subject, "call_id")?;
    let meeting_id = required_payload_string(&event.payload, "meeting_id")?;
    let join_url = optional_payload_string(&event.payload, "join_url");
    let started_at = optional_payload_datetime(&event.payload, "started_at");
    let ended_at = optional_payload_datetime(&event.payload, "ended_at");

    let event_store = CalendarEventQueryPort::new(pool.clone());
    let Some(calendar_event) = event_store
        .find_zoom_conference_match(join_url, meeting_id, started_at, ended_at)
        .await?
    else {
        return Ok(());
    };

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "CALENDAR_EVENT",
                ObservationOriginKind::LocalRuntime,
                event.occurred_at,
                json!({
                    "event_id": calendar_event.event_id,
                    "matched_entity_type": "call",
                    "matched_entity_id": call_id,
                    "meeting_id": meeting_id,
                    "join_url": join_url,
                    "source_event_id": event.event_id,
                    "match_strategy": "zoom_conference_url_and_time_overlap",
                }),
                format!(
                    "calendar-event://{}/matches/zoom-call/{}",
                    calendar_event.event_id, call_id
                ),
            )
            .provenance(json!({
                "captured_by": "zoom_calendar_matching",
                "event_id": event.event_id,
                "event_type": event.event_type,
            })),
        )
        .await?;

    EventRelationPort::new(pool.clone())
        .link_with_observation(
            &calendar_event.event_id,
            "call",
            call_id,
            ZOOM_CALENDAR_RELATION_TYPE,
            zoom_event_types::MEETING_OBSERVED,
            Some(&observation.observation_id),
        )
        .await?;

    Ok(())
}

fn required_subject_string<'a>(
    subject: &'a Value,
    field: &'static str,
) -> Result<&'a str, ZoomCalendarMatchingWorkflowError> {
    subject
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomCalendarMatchingWorkflowError::MissingPayloadField(
            field,
        ))
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, ZoomCalendarMatchingWorkflowError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomCalendarMatchingWorkflowError::MissingPayloadField(
            field,
        ))
}

fn optional_payload_string<'a>(payload: &'a Value, field: &'static str) -> Option<&'a str> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn optional_payload_datetime(payload: &Value, field: &'static str) -> Option<DateTime<Utc>> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
}
```

### `backend/src/workflows/zoom_participant_identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/zoom_participant_identity.rs`
- Size bytes / Размер в байтах: `3718`
- Included characters / Включено символов: `3718`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;
use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::persons::identity::{PersonIdentityError, PersonIdentityPort};
use crate::platform::events::bus::zoom_event_types;
use crate::platform::events::{EventEnvelope, EventStoreError, StoredEventEnvelope};

pub const ZOOM_PARTICIPANT_IDENTITY_CONSUMER: &str = "zoom_participant_identity";

const ATTACH_EMAIL_CANDIDATE_LIMIT_PER_PARTICIPANT: i64 = 10;
const ATTACH_EMAIL_CANDIDATE_CONFIDENCE: f64 = 0.68;

#[derive(Debug, Deserialize)]
struct ZoomParticipantObservation {
    display_name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Error)]
pub enum ZoomParticipantIdentityWorkflowError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    PersonsIdentity(#[from] PersonIdentityError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),
}

pub async fn project_zoom_participant_identity_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_zoom_participant_identity(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_zoom_participant_identity(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomParticipantIdentityWorkflowError> {
    if event.event_type != zoom_event_types::MEETING_OBSERVED {
        return Ok(());
    }

    let meeting_id = required_payload_string(&event.payload, "meeting_id")?;
    let topic = optional_payload_string(&event.payload, "topic");
    let participants = event
        .payload
        .get("participants")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let participants: Vec<ZoomParticipantObservation> = serde_json::from_value(participants)?;

    let store = PersonIdentityPort::new(pool.clone());
    for participant in participants {
        let Some(display_name) = participant
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(email_address) = participant
            .email
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        let evidence_summary = if let Some(topic) = topic {
            format!(
                "Zoom participant {display_name} <{email_address}> observed in meeting {meeting_id} ({topic})"
            )
        } else {
            format!(
                "Zoom participant {display_name} <{email_address}> observed in meeting {meeting_id}"
            )
        };
        store
            .suggest_attach_email_candidates(
                display_name,
                email_address,
                &evidence_summary,
                ATTACH_EMAIL_CANDIDATE_CONFIDENCE,
                ATTACH_EMAIL_CANDIDATE_LIMIT_PER_PARTICIPANT,
            )
            .await?;
    }

    Ok(())
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, ZoomParticipantIdentityWorkflowError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomParticipantIdentityWorkflowError::MissingPayloadField(
            field,
        ))
}

fn optional_payload_string<'a>(payload: &'a Value, field: &'static str) -> Option<&'a str> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
```

### `backend/src/workflows/zoom_signal_detection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/zoom_signal_detection.rs`
- Size bytes / Размер в байтах: `6386`
- Included characters / Включено символов: `6386`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::signal_hub::{
    SignalHubError, SignalHubPort, SignalHubSignalService,
    signal_hub_raw_dispatcher_allows_processing,
};
use crate::platform::events::bus::zoom_event_types;
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventLogPort, EventLogPortError, NewEventEnvelope,
    StoredEventEnvelope,
};

pub const ZOOM_SIGNAL_DETECTION_CONSUMER: &str = "zoom_signal_detection";

#[derive(Debug, Error)]
pub enum ZoomSignalDetectionWorkflowError {
    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    EventLog(#[from] EventLogPortError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error("event `{0}` is missing required field `{1}`")]
    MissingField(&'static str, &'static str),
}

pub async fn project_zoom_signal_detection_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventLogPortError> {
    project_zoom_signal_detection(&pool, &event.event)
        .await
        .map_err(|error| EventLogPortError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_zoom_signal_detection(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ZoomSignalDetectionWorkflowError> {
    let Some(raw_signal) = build_zoom_raw_signal(event)? else {
        return Ok(());
    };

    let event_store = EventLogPort::new(pool.clone());
    let raw_signal_id = raw_signal.event_id.clone();
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store.get_by_id(&raw_signal_id).await?.ok_or(
        SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()),
    )?;

    let signal_store = SignalHubPort::new(pool.clone());
    if !signal_hub_raw_dispatcher_allows_processing(&signal_store).await? {
        return Ok(());
    }

    let service = SignalHubSignalService::new(signal_store, event_store);
    let _ = service.process_raw_signal(&raw_event).await?;
    Ok(())
}

fn build_zoom_raw_signal(
    event: &EventEnvelope,
) -> Result<Option<NewEventEnvelope>, ZoomSignalDetectionWorkflowError> {
    let Some(event_kind) = zoom_signal_event_kind(&event.event_type) else {
        return Ok(None);
    };

    let account_id = required_string(
        &event.source,
        "account_id",
        "zoom signal detection source.account_id",
    )?;
    let subject = zoom_raw_signal_subject(event, event_kind, account_id)?;
    let source = json!({
        "kind": "signal_source",
        "source_code": "zoom",
        "source_id": event.event_id,
        "account_id": account_id,
        "provider_kind": event.source.get("provider_kind").cloned(),
        "zoom_event_type": event.event_type,
    });
    let provenance = json!({
        "source": "zoom_signal_detection",
        "zoom_event_id": event.event_id,
        "zoom_event_type": event.event_type,
        "zoom_event_provenance": event.provenance,
    });

    let builder = NewEventEnvelope::builder(
        zoom_raw_signal_event_id(&event.event_id),
        format!("signal.raw.zoom.{event_kind}.observed"),
        event.occurred_at,
        source,
        subject,
    )
    .payload(event.payload.clone())
    .provenance(provenance)
    .causation_id(event.event_id.clone());

    let builder = match &event.correlation_id {
        Some(value) => builder.correlation_id(value.clone()),
        None => builder,
    };

    Ok(Some(builder.build()?))
}

fn zoom_signal_event_kind(event_type: &str) -> Option<&'static str> {
    match event_type {
        zoom_event_types::MEETING_OBSERVED => Some("meeting"),
        zoom_event_types::RECORDING_OBSERVED => Some("recording"),
        zoom_event_types::TRANSCRIPT_OBSERVED => Some("transcript"),
        _ => None,
    }
}

fn zoom_raw_signal_subject(
    event: &EventEnvelope,
    event_kind: &str,
    account_id: &str,
) -> Result<Value, ZoomSignalDetectionWorkflowError> {
    let mut subject = json!({
        "kind": "signal",
        "source_code": "zoom",
        "account_id": account_id,
        "zoom_event_id": event.event_id,
        "zoom_event_type": event.event_type,
    });

    match event_kind {
        "meeting" => {
            let call_id = required_string(&event.subject, "call_id", "zoom.meeting.observed")?;
            let meeting_id =
                required_string(&event.payload, "meeting_id", "zoom.meeting.observed")?;
            subject["entity_id"] = json!(call_id);
            subject["call_id"] = json!(call_id);
            subject["meeting_id"] = json!(meeting_id);
        }
        "recording" => {
            let recording_id =
                required_string(&event.subject, "recording_id", "zoom.recording.observed")?;
            let meeting_id =
                required_string(&event.subject, "meeting_id", "zoom.recording.observed")?;
            subject["entity_id"] = json!(recording_id);
            subject["recording_id"] = json!(recording_id);
            subject["meeting_id"] = json!(meeting_id);
        }
        "transcript" => {
            let transcript_id =
                required_string(&event.subject, "transcript_id", "zoom.transcript.observed")?;
            let call_id = required_string(&event.subject, "call_id", "zoom.transcript.observed")?;
            let meeting_id =
                required_string(&event.subject, "meeting_id", "zoom.transcript.observed")?;
            subject["entity_id"] = json!(transcript_id);
            subject["transcript_id"] = json!(transcript_id);
            subject["call_id"] = json!(call_id);
            subject["meeting_id"] = json!(meeting_id);
        }
        _ => {}
    }

    Ok(subject)
}

fn required_string<'a>(
    value: &'a Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<&'a str, ZoomSignalDetectionWorkflowError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ZoomSignalDetectionWorkflowError::MissingField(
            event_type, field,
        ))
}

fn zoom_raw_signal_event_id(zoom_event_id: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(zoom_event_id.as_bytes());
    format!("evt_signal_raw_zoom_{:x}", digest.finalize())
}
```
