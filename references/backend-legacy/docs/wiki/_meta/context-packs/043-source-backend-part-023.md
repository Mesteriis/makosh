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

- Chunk ID / ID чанка: `043-source-backend-part-023`
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

### `backend/src/domains/communications/storage/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/store.rs`
- Size bytes / Размер в байтах: `7557`
- Included characters / Включено символов: `7557`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use super::errors::CommunicationStorageError;
use super::ids::{mail_attachment_id, mail_blob_id};
use super::models::{
    NewCommunicationAttachment, NewCommunicationBlob, StoredCommunicationAttachment,
    StoredCommunicationAttachmentWithBlob, StoredCommunicationBlob,
};
use super::rows::{row_to_mail_attachment, row_to_mail_attachment_with_blob, row_to_mail_blob};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct CommunicationStorageStore {
    pub(crate) pool: PgPool,
}

impl CommunicationStorageStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_blob(
        &self,
        blob: &NewCommunicationBlob,
    ) -> Result<StoredCommunicationBlob, CommunicationStorageError> {
        let blob = blob.validate()?;
        let blob_id = mail_blob_id(&blob.sha256);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_mail_blobs (
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (sha256)
            DO UPDATE SET
                content_type = COALESCE(communication_mail_blobs.content_type, EXCLUDED.content_type)
            RETURNING
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type,
                created_at
            "#,
        )
        .bind(&blob_id)
        .bind(&blob.storage_kind)
        .bind(&blob.storage_path)
        .bind(&blob.sha256)
        .bind(blob.size_bytes)
        .bind(&blob.content_type)
        .fetch_one(&self.pool)
        .await?;

        row_to_mail_blob(row)
    }

    pub async fn upsert_attachment(
        &self,
        attachment: &NewCommunicationAttachment,
    ) -> Result<StoredCommunicationAttachment, CommunicationStorageError> {
        let attachment = attachment.validate()?;
        let attachment_id =
            mail_attachment_id(&attachment.message_id, &attachment.provider_attachment_id);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_attachments (
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
            ON CONFLICT (message_id, provider_attachment_id)
            DO UPDATE SET
                raw_record_id = EXCLUDED.raw_record_id,
                blob_id = EXCLUDED.blob_id,
                filename = EXCLUDED.filename,
                content_type = EXCLUDED.content_type,
                size_bytes = EXCLUDED.size_bytes,
                sha256 = EXCLUDED.sha256,
                disposition = EXCLUDED.disposition,
                scan_status = EXCLUDED.scan_status,
                scan_engine = EXCLUDED.scan_engine,
                scan_checked_at = EXCLUDED.scan_checked_at,
                scan_summary = EXCLUDED.scan_summary,
                scan_metadata = EXCLUDED.scan_metadata,
                updated_at = now()
            RETURNING
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&attachment_id)
        .bind(&attachment.message_id)
        .bind(&attachment.raw_record_id)
        .bind(&attachment.blob_id)
        .bind(&attachment.provider_attachment_id)
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.sha256)
        .bind(attachment.disposition.as_str())
        .bind(attachment.scan_report.status.as_str())
        .bind(&attachment.scan_report.engine)
        .bind(attachment.scan_report.checked_at)
        .bind(&attachment.scan_report.summary)
        .bind(&attachment.scan_report.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_mail_attachment(row)
    }

    pub async fn attachments_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<StoredCommunicationAttachmentWithBlob>, CommunicationStorageError> {
        let message_id = validate_non_empty("message_id", message_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
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
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.message_id = $1
            ORDER BY a.created_at ASC, a.attachment_id ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_mail_attachment_with_blob)
            .collect()
    }

    pub async fn attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<Option<StoredCommunicationAttachmentWithBlob>, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
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
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.attachment_id = $1
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_mail_attachment_with_blob).transpose()
    }
}
```

### `backend/src/domains/communications/storage/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/storage/validation.rs`
- Size bytes / Размер в байтах: `1994`
- Included characters / Включено символов: `1994`
- Truncated / Обрезано: `no`

```rust
use std::path::{Component, Path};

use super::constants::{LOCAL_FS_STORAGE_KIND, SHA256_PREFIX};
use super::errors::CommunicationStorageError;

pub(crate) fn validate_storage_kind(value: &str) -> Result<String, CommunicationStorageError> {
    let value = validate_non_empty("storage_kind", value)?;
    if value != LOCAL_FS_STORAGE_KIND {
        return Err(CommunicationStorageError::InvalidStorageKind(value));
    }
    Ok(value)
}

pub(crate) fn validate_storage_path(value: &str) -> Result<String, CommunicationStorageError> {
    let value = validate_non_empty("storage_path", value)?;
    let path = Path::new(value.as_str());
    if path.is_absolute() || value.contains('\\') {
        return Err(CommunicationStorageError::UnsafeStoragePath(value));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(CommunicationStorageError::UnsafeStoragePath(value)),
        }
    }

    Ok(value)
}

pub(crate) fn validate_sha256(value: &str) -> Result<String, CommunicationStorageError> {
    let value = validate_non_empty("sha256", value)?;
    let Some(hex) = value.strip_prefix(SHA256_PREFIX) else {
        return Err(CommunicationStorageError::InvalidSha256(value));
    };
    if hex.len() != 64 || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(CommunicationStorageError::InvalidSha256(value));
    }
    Ok(format!("{SHA256_PREFIX}{}", hex.to_ascii_lowercase()))
}

pub(crate) fn validate_size_bytes(value: i64) -> Result<i64, CommunicationStorageError> {
    if value < 0 {
        return Err(CommunicationStorageError::NegativeSizeBytes(value));
    }
    Ok(value)
}

pub(crate) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, CommunicationStorageError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(CommunicationStorageError::EmptyField(field_name));
    }
    Ok(value)
}
```

### `backend/src/domains/communications/subscriptions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/subscriptions.rs`
- Size bytes / Размер в байтах: `5030`
- Included characters / Включено символов: `5030`
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
pub struct SubscriptionSource {
    pub sender: String,
    pub message_count: i64,
    pub first_seen: String,
    pub last_seen: String,
    pub is_newsletter: bool,
    pub has_unsubscribe: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubscriptionSourceListPage {
    pub items: Vec<SubscriptionSource>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone)]
pub struct SubscriptionStore {
    pool: PgPool,
}

impl SubscriptionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn detect_subscriptions(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SubscriptionSource>, SubscriptionError> {
        Ok(self
            .detect_subscriptions_page(account_id, limit, None)
            .await?
            .items)
    }

    pub async fn detect_subscriptions_page(
        &self,
        account_id: Option<&str>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<SubscriptionSourceListPage, SubscriptionError> {
        let limit = limit.clamp(1, 100);
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_subscription_cursor)
            .transpose()?;
        let rows = sqlx::query(
            r#"WITH subscription_sources AS (
                SELECT sender, count(*)::BIGINT AS message_count,
                    min(occurred_at)::TEXT AS first_seen, max(occurred_at)::TEXT AS last_seen,
                    bool_or(lower(body_text) LIKE '%unsubscribe%' OR lower(body_text) LIKE '%opt out%' OR lower(body_text) LIKE '%manage preferences%') AS has_unsubscribe,
                    bool_or(lower(subject) LIKE '%newsletter%' OR lower(subject) LIKE '%digest%' OR lower(body_text) LIKE '%newsletter%') AS is_newsletter
                FROM communication_messages
                WHERE ($1::text IS NULL OR account_id = $1)
                  AND channel_kind = 'email'
                  AND local_state = 'active'
                GROUP BY sender
                HAVING count(*) > 1
            )
            SELECT sender, message_count, first_seen, last_seen, has_unsubscribe, is_newsletter
            FROM subscription_sources
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

        let mut subs = Vec::new();
        for row in rows {
            subs.push(SubscriptionSource {
                sender: row.try_get("sender")?,
                message_count: row.try_get("message_count")?,
                first_seen: row.try_get("first_seen")?,
                last_seen: row.try_get("last_seen")?,
                is_newsletter: row.try_get("is_newsletter")?,
                has_unsubscribe: row.try_get("has_unsubscribe")?,
            });
        }
        let has_more = subs.len() > limit as usize;
        if has_more {
            subs.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            subs.last().map(encode_subscription_cursor).transpose()?
        } else {
            None
        };
        Ok(SubscriptionSourceListPage {
            items: subs,
            next_cursor,
            has_more,
        })
    }
}

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid subscription cursor")]
    InvalidCursor,
}

#[derive(Debug, Deserialize, Serialize)]
struct SubscriptionCursor {
    message_count: i64,
    sender: String,
}

fn encode_subscription_cursor(source: &SubscriptionSource) -> Result<String, SubscriptionError> {
    let cursor = SubscriptionCursor {
        message_count: source.message_count,
        sender: source.sender.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| SubscriptionError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_subscription_cursor(cursor: &str) -> Result<SubscriptionCursor, SubscriptionError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| SubscriptionError::InvalidCursor)?;
    let cursor: SubscriptionCursor =
        serde_json::from_slice(&bytes).map_err(|_| SubscriptionError::InvalidCursor)?;
    if cursor.message_count < 0 || cursor.sender.trim().is_empty() {
        return Err(SubscriptionError::InvalidCursor);
    }
    Ok(cursor)
}
```

### `backend/src/domains/communications/templates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/templates.rs`
- Size bytes / Размер в байтах: `19000`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationTemplate {
    pub template_id: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<String>,
    #[serde(default)]
    pub placeholder_variables: Vec<String>,
    #[serde(default)]
    pub undeclared_variables: Vec<String>,
    #[serde(default)]
    pub unused_variables: Vec<String>,
    #[serde(default)]
    pub malformed_placeholders: Vec<String>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CommunicationTemplateStore {
    pool: PgPool,
}

impl CommunicationTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        tpl: &NewCommunicationTemplate,
    ) -> Result<CommunicationTemplate, CommunicationTemplateError> {
        tpl.validate()?;
        let vars: Value = tpl
            .variables
            .iter()
            .map(|v| Value::String(v.clone()))
            .collect();
        let row = sqlx::query(
            r#"INSERT INTO communication_templates (template_id, name, subject_template, body_template, variables, language)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (template_id) DO UPDATE SET
                name = EXCLUDED.name, subject_template = EXCLUDED.subject_template,
                body_template = EXCLUDED.body_template, variables = EXCLUDED.variables,
                language = EXCLUDED.language, updated_at = now()
            RETURNING template_id, name, subject_template, body_template, variables, language, created_at, updated_at"#,
        )
        .bind(&tpl.template_id).bind(&tpl.name).bind(&tpl.subject_template).bind(&tpl.body_template)
        .bind(&vars).bind(tpl.language.as_deref())
        .fetch_one(&self.pool).await?;
        row_to_template(row)
    }

    pub async fn list(&self) -> Result<Vec<CommunicationTemplate>, CommunicationTemplateError> {
        let rows = sqlx::query(
            r#"SELECT template_id, name, subject_template, body_template, variables, language, created_at, updated_at
            FROM communication_templates ORDER BY name"#,
        ).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_template).collect()
    }

    pub async fn get(
        &self,
        template_id: &str,
    ) -> Result<Option<CommunicationTemplate>, CommunicationTemplateError> {
        let row = sqlx::query(
            r#"SELECT template_id, name, subject_template, body_template, variables, language, created_at, updated_at
            FROM communication_templates WHERE template_id = $1"#,
        ).bind(template_id).fetch_optional(&self.pool).await?;
        row.map(row_to_template).transpose()
    }

    pub async fn delete(&self, template_id: &str) -> Result<bool, CommunicationTemplateError> {
        let result = sqlx::query("DELETE FROM communication_templates WHERE template_id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Render a template with variables.
    pub fn render(
        &self,
        template: &CommunicationTemplate,
        vars: &HashMap<String, String>,
    ) -> Result<RenderedTemplate, CommunicationTemplateError> {
        let missing_variables = template
            .variables
            .iter()
            .filter(|variable| {
                vars.get(variable.as_str())
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        let subject = render_template_text(&template.subject_template, vars);
        let body = render_template_text(&template.body_template, vars);
        let unresolved_variables = unique_strings(
            subject
                .unresolved_variables
                .iter()
                .chain(body.unresolved_variables.iter()),
        );
        let malformed_placeholders = unique_strings(
            subject
                .malformed_placeholders
                .iter()
                .chain(body.malformed_placeholders.iter()),
        );
        Ok(RenderedTemplate {
            subject: subject.text,
            body: body.text,
            missing_variables,
            unresolved_variables,
            malformed_placeholders,
        })
    }

    pub fn render_mail_merge_preview(
        &self,
        template: &CommunicationTemplate,
        rows: Vec<CommunicationMergePreviewRow>,
    ) -> Result<CommunicationMergePreview, CommunicationTemplateError> {
        let template_has_blocking_diagnostics = !template.undeclared_variables.is_empty()
            || !template.malformed_placeholders.is_empty();
        let items = rows
            .into_iter()
            .map(|row| {
                let rendered = self.render(template, &row.variables)?;
                let ready = !template_has_blocking_diagnostics
                    && rendered.missing_variables.is_empty()
                    && rendered.unresolved_variables.is_empty()
                    && rendered.malformed_placeholders.is_empty();
                Ok(CommunicationMergePreviewItem {
                    row_id: row.row_id,
                    ready,
                    rendered,
                })
            })
            .collect::<Result<Vec<_>, CommunicationTemplateError>>()?;
        let ready_count = items.iter().filter(|item| item.ready).count();
        let row_count = items.len();
        Ok(CommunicationMergePreview {
            template_id: template.template_id.clone(),
            row_count,
            ready_count,
            blocked_count: row_count.saturating_sub(ready_count),
            items,
        })
    }
}

#[derive(Clone, Debug)]
pub struct NewCommunicationTemplate {
    pub template_id: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub language: Option<String>,
}

impl NewCommunicationTemplate {
    fn validate(&self) -> Result<(), CommunicationTemplateError> {
        if self.template_id.trim().is_empty() {
            return Err(CommunicationTemplateError::InvalidTemplate(
                "template_id empty",
            ));
        }
        if self.name.trim().is_empty() {
            return Err(CommunicationTemplateError::InvalidTemplate("name empty"));
        }
        if self.subject_template.trim().is_empty() {
            return Err(CommunicationTemplateError::InvalidTemplate(
                "subject_template empty",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RenderedTemplate {
    pub subject: String,
    pub body: String,
    pub missing_variables: Vec<String>,
    pub unresolved_variables: Vec<String>,
    pub malformed_placeholders: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CommunicationMergePreviewRow {
    pub row_id: String,
    pub variables: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationMergePreview {
    pub template_id: String,
    pub row_count: usize,
    pub ready_count: usize,
    pub blocked_count: usize,
    pub items: Vec<CommunicationMergePreviewItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationMergePreviewItem {
    pub row_id: String,
    pub ready: bool,
    pub rendered: RenderedTemplate,
}

struct RenderedTemplateText {
    text: String,
    unresolved_variables: Vec<String>,
    malformed_placeholders: Vec<String>,
}

struct TemplateValidation {
    placeholder_variables: Vec<String>,
    undeclared_variables: Vec<String>,
    unused_variables: Vec<String>,
    malformed_placeholders: Vec<String>,
}

struct CommunicationTemplateMetadataInput {
    template_id: String,
    name: String,
    subject_template: String,
    body_template: String,
    variables: Vec<String>,
    language: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn row_to_template(row: PgRow) -> Result<CommunicationTemplate, CommunicationTemplateError> {
    let vars: Value = row.try_get("variables")?;
    let variables: Vec<String> = vars
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Ok(email_template_with_metadata(
        CommunicationTemplateMetadataInput {
            template_id: row.try_get("template_id")?,
            name: row.try_get("name")?,
            subject_template: row.try_get("subject_template")?,
            body_template: row.try_get("body_template")?,
            variables,
            language: row.try_get("language")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        },
    ))
}

fn email_template_with_metadata(
    input: CommunicationTemplateMetadataInput,
) -> CommunicationTemplate {
    let validation = validate_template_content(
        &input.subject_template,
        &input.body_template,
        &input.variables,
    );
    CommunicationTemplate {
        template_id: input.template_id,
        name: input.name,
        subject_template: input.subject_template,
        body_template: input.body_template,
        variables: input.variables,
        placeholder_variables: validation.placeholder_variables,
        undeclared_variables: validation.undeclared_variables,
        unused_variables: validation.unused_variables,
        malformed_placeholders: validation.malformed_placeholders,
        language: input.language,
        created_at: input.created_at,
        updated_at: input.updated_at,
    }
}

fn validate_template_content(
    subject_template: &str,
    body_template: &str,
    variables: &[String],
) -> TemplateValidation {
    let empty_vars = HashMap::new();
    let subject = render_template_text(subject_template, &empty_vars);
    let body = render_template_text(body_template, &empty_vars);
    let placeholder_variables = unique_strings(
        subject
            .unresolved_variables
            .iter()
            .chain(body.unresolved_variables.iter()),
    );
    let malformed_placeholders = unique_strings(
        subject
            .malformed_placeholders
            .iter()
            .chain(body.malformed_placeholders.iter()),
    );
    let undeclared_variables = strings_not_in(&placeholder_variables, variables);
    let unused_variables = strings_not_in(variables, &placeholder_variables);

    TemplateValidation {
        placeholder_variables,
        undeclared_variables,
        unused_variables,
        malformed_placeholders,
    }
}

fn strings_not_in(source: &[String], excluded: &[String]) -> Vec<String> {
    source
        .iter()
        .filter(|value| {
            !excluded
                .iter()
                .any(|excluded_value| excluded_value.as_str() == value.as_str())
        })
        .cloned()
        .collect()
}

fn render_template_text(template: &str, vars: &HashMap<String, String>) -> RenderedTemplateText {
    let mut rendered = String::with_capacity(template.len());
    let mut unresolved_variables = Vec::new();
    let mut malformed_placeholders = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        rendered.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        let Some(end) = after_open.find("}}") else {
            let malformed = &rest[start..];
            rendered.push_str(malformed);
            if !malformed_placeholders
                .iter()
                .any(|existing: &String| existing.as_str() == malformed)
            {

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/communications/threads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/communications/threads.rs`
- Size bytes / Размер в байтах: `15363`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
#[allow(unused_imports)]
use sqlx::postgres::PgRow;
use thiserror::Error;

/// Normalize an email subject for thread grouping.
/// Strips Re:, Fwd:, AW:, WG: prefixes and whitespace.
pub fn normalize_subject_for_thread(subject: &str) -> String {
    let mut s = subject.trim().to_owned();
    loop {
        let lower = s.to_lowercase();
        let prefix_len = if lower.starts_with("re:") {
            "re:".len()
        } else if lower.starts_with("aw:") {
            "aw:".len()
        } else if lower.starts_with("wg:") {
            "wg:".len()
        } else if lower.starts_with("fwd:") {
            "fwd:".len()
        } else if lower.starts_with("fw:") {
            "fw:".len()
        } else {
            break;
        };
        s = s[prefix_len..].trim().to_owned();
    }
    s
}

/// Deterministic thread ID from account + normalized subject.
pub fn thread_id(account_id: &str, subject: &str) -> String {
    let normalized = normalize_subject_for_thread(subject);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&account_id, &mut hasher);
    std::hash::Hash::hash(&normalized.to_lowercase(), &mut hasher);
    format!("thread:{:016x}", std::hash::Hasher::finish(&hasher))
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationThread {
    pub thread_id: String,
    pub account_id: String,
    pub subject: String,
    pub message_count: i64,
    pub participant_count: i64,
    pub first_message_at: Option<DateTime<Utc>>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub has_open_action: bool,
    pub has_attachments: bool,
    pub dominant_workflow_state: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationThreadListPage {
    pub items: Vec<CommunicationThread>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone)]
pub struct CommunicationThreadStore {
    pool: PgPool,
}

impl CommunicationThreadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List threads for an account, ordered by most recent activity.
    pub async fn list_threads(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CommunicationThread>, CommunicationThreadError> {
        Ok(self.list_threads_page(account_id, None, limit).await?.items)
    }

    pub async fn list_threads_page(
        &self,
        account_id: Option<&str>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<CommunicationThreadListPage, CommunicationThreadError> {
        let limit = if (1..=100).contains(&limit) {
            limit
        } else {
            50
        };
        let cursor = cursor.map(decode_thread_list_cursor).transpose()?;

        let rows = sqlx::query(
            r#"
            WITH grouped_threads AS (
                SELECT
                    COALESCE(m.conversation_id, md5(m.account_id || ':' || lower(regexp_replace(regexp_replace(regexp_replace(m.subject, '^re:\s*', '', 'i'), '^fwd:\s*', '', 'i'), '^aw:\s*', '', 'i')))) AS thread_id,
                    m.account_id,
                    regexp_replace(regexp_replace(regexp_replace(m.subject, '^re:\s*', '', 'i'), '^fwd:\s*', '', 'i'), '^aw:\s*', '', 'i') AS normalized_subject,
                    count(*)::BIGINT AS message_count,
                    count(DISTINCT m.sender)::BIGINT AS participant_count,
                    min(m.occurred_at) AS first_message_at,
                    max(m.occurred_at) AS last_message_at,
                    max(COALESCE(m.occurred_at, m.projected_at)) AS last_activity_at,
                    bool_or(m.workflow_state IN ('needs_action', 'new')) AS has_open_action,
                    bool_or(EXISTS(SELECT 1 FROM communication_attachments a WHERE a.message_id = m.message_id)) AS has_attachments,
                    mode() WITHIN GROUP (ORDER BY m.workflow_state) AS dominant_workflow_state
                FROM communication_messages m
                WHERE ($1::text IS NULL OR m.account_id = $1)
                  AND m.channel_kind = 'email'
                  AND m.local_state = 'active'
                GROUP BY thread_id, m.account_id, normalized_subject
            )
            SELECT
                thread_id,
                account_id,
                normalized_subject,
                message_count,
                participant_count,
                first_message_at,
                last_message_at,
                last_activity_at,
                has_open_action,
                has_attachments,
                dominant_workflow_state
            FROM grouped_threads
            WHERE (
                $2::timestamptz IS NULL
                OR last_activity_at < $2
                OR (last_activity_at = $2 AND thread_id > $3)
            )
            ORDER BY last_activity_at DESC, thread_id ASC
            LIMIT $4
            "#,
        )
        .bind(account_id)
        .bind(cursor.as_ref().map(|cursor| cursor.last_activity_at))
        .bind(cursor.as_ref().map(|cursor| cursor.thread_id.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;

        let mut threads = Vec::new();
        for row in rows {
            threads.push(CommunicationThread {
                thread_id: row.try_get("thread_id")?,
                account_id: row.try_get("account_id")?,
                subject: row.try_get("normalized_subject")?,
                message_count: row.try_get("message_count")?,
                participant_count: row.try_get("participant_count")?,
                first_message_at: row.try_get("first_message_at")?,
                last_message_at: row.try_get("last_message_at")?,
                last_activity_at: row.try_get("last_activity_at")?,
                has_open_action: row.try_get("has_open_action")?,
                has_attachments: row.try_get("has_attachments")?,
                dominant_workflow_state: row.try_get::<String, _>("dominant_workflow_state")?,
            });
        }

        let has_more = threads.len() > limit as usize;
        if has_more {
            threads.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            threads.last().map(encode_thread_list_cursor).transpose()?
        } else {
            None
        };

        Ok(CommunicationThreadListPage {
            items: threads,
            next_cursor,
            has_more,
        })
    }

    /// Get messages belonging to a thread, identified by normalized subject + account_id.
    pub async fn thread_messages(
        &self,
        account_id: &str,
        normalized_subject: &str,
        limit: i64,
    ) -> Result<Vec<ThreadMessage>, CommunicationThreadError> {
        let limit = if (1..=100).contains(&limit) {
            limit
        } else {
            50
        };

        let rows = sqlx::query(
            r#"
            SELECT
                m.message_id,
                m.provider_record_id,
                m.account_id,
                m.subject,
                m.sender,
                m.sender_display_name,
                m.body_text,
                m.occurred_at,
                m.projected_at,
                m.workflow_state,
                m.importance_score,
                m.ai_category,
                m.ai_summary,
                m.delivery_state,
                count(a.attachment_id)::BIGINT AS attachment_count,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'attachment_id', a.attachment_id,
                            'message_id', a.message_id,
                            'raw_record_id', a.raw_record_id,
                            'blob_id', a.blob_id,
                            'provider_attachment_id', a.provider_attachment_id,
                            'filename', a.filename,
                            'content_type', a.content_type,
                            'size_bytes', a.size_bytes,
                            'sha256', a.sha256,
                            'disposition', a.disposition,
                            'scan_status', a.scan_status,
                            'scan_engine', a.scan_engine,
                            'scan_checked_at', a.scan_checked_at,
                            'scan_summary', a.scan_summary,
                            'scan_metadata', a.scan_metadata,
                            'storage_kind', b.storage_kind,
                            'storage_path', b.storage_path,
                            'created_at', a.created_at,
                            'updated_at', a.updated_at
                        )
                        ORDER BY a.created_at ASC
                    ) FILTER (WHERE a.attachment_id IS NOT NULL),
                    '[]'::jsonb
                ) AS attachments
            FROM communication_messages m
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            LEFT JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE m.account_id = $1
              AND m.channel_kind = 'email'
              AND m.local_state = 'active'
              AND regexp_replace(regexp_replace(regexp_replace(m.subject, '^re:\s*', '', 'i'), '^fwd:\s*', '', 'i'), '^aw:\s*', '', 'i') = $2
            GROUP BY m.message_id, m.provider_record_id, m.account_id, m.subject, m.sender, m.sender_display_name,
                     m.body_text, m.occurred_at, m.projected_at, m.workflow_state,
                     m.importance_score, m.ai_category, m.ai_summary, m.delivery_state
            ORDER BY COALESCE(m.occurred_at, m.projected_at) ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(normalized_subject)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(ThreadMessage {
                message_id: row.try_get("message_id")?,
                provider_record_id: row.try_get("provider_record_id")?,
                account_id: row.try_get("account_id")?,
                subject: row.try_get("subject")?,
                sender: row.try_get("sender")?,
                sender_display_name: row.try_get("sender_display_name")?,
                body_text: row.try_get("body_text")?,
                occurred_at: row.try_get("occurred_at")?,
                projected_at: row.try_get("projected_at")?,
                workflow_state: row.try_get("workflow_state")?,
                importance_score: row.try_get("importance_score")?,
                ai_category: row.try_get("ai_category")?,
                ai_summary: row.try_get("ai_summary")?,
                delivery_state: row.try_get("delivery_state")?,
                attachment_count: row.try_get("attachment_count")?,
                attachments: serde_json::from_value(row.try_get::<Value, _>("attachments")?)
                    .map_err(CommunicationThreadError::Serde)?,
            });
        }

        Ok(messages)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ThreadMessage {
    pub message_id: String,
    pub provider_record_id: String,
    pub account_id: String,
    pub subject: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub workflow_state: String,
    pub importance_score: Option<i16>,
    pub ai_category: Option<String>,
    pub ai_summary: Option<String>,
    pub delivery_state: String,
    pub attachment_count: i64,
    pub attachments: Vec<ThreadMessageAttachment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadMessageAttachmen
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/decisions/candidate_refresh.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/candidate_refresh.rs`
- Size bytes / Размер в байтах: `5538`
- Included characters / Включено символов: `5538`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;

use crate::domains::decisions::{
    DecisionEngine, DecisionExtractionInput, DecisionExtractionResult,
};

use super::errors::DecisionStoreError;
use super::models::DecisionEntityKind;
use super::store::DecisionStore;
use super::validation::{preserve_existing_review_state, validate_refresh_limit};

impl DecisionStore {
    pub async fn refresh_deterministic_candidates(
        &self,
        limit: i64,
    ) -> Result<usize, DecisionStoreError> {
        let limit = validate_refresh_limit(limit)?;
        let message_count = self.refresh_message_candidates(limit).await?;
        let document_count = self.refresh_document_candidates(limit).await?;

        Ok(message_count + document_count)
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, DecisionStoreError> {
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
        .fetch_all(&self.pool)
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
            count += self
                .refresh_communication_decision_candidates(&source_id, observation_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    async fn refresh_message_candidates(&self, limit: i64) -> Result<usize, DecisionStoreError> {
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
        .fetch_all(&self.pool)
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
            count += self
                .refresh_communication_decision_candidates(&source_id, observation_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    async fn refresh_communication_decision_candidates(
        &self,
        source_id: &str,
        observation_id: Option<String>,
        source_text: &str,
    ) -> Result<usize, DecisionStoreError> {
        let input = DecisionExtractionInput::communication(
            source_id,
            source_text,
            DecisionEntityKind::Communication,
            source_id,
        )
        .with_observation_id(observation_id);
        let extraction = DecisionEngine::detect_candidates(&input)?;
        self.persist_decision_extraction(extraction).await
    }

    async fn refresh_document_candidates(&self, limit: i64) -> Result<usize, DecisionStoreError> {
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
        .fetch_all(&self.pool)
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
            let input = DecisionExtractionInput::document(
                &source_id,
                &source_text,
                DecisionEntityKind::Document,
                &source_id,
            )
            .with_observation_id(observation_id);
            let extraction = DecisionEngine::detect_candidates(&input)?;
            count += self.persist_decision_extraction(extraction).await?;
        }

        Ok(count)
    }

    async fn persist_decision_extraction(
        &self,
        extraction: DecisionExtractionResult,
    ) -> Result<usize, DecisionStoreError> {
        let mut count = 0usize;
        for candidate in extraction.decisions {
            let (mut decision, evidence, impacted_entities) = candidate.to_decision_draft();
            preserve_existing_review_state(&self.pool, &mut decision).await?;
            self.upsert_with_evidence(&decision, &[evidence], &impacted_entities)
                .await?;
            count += 1;
        }

        Ok(count)
    }
}
```

### `backend/src/domains/decisions/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/constants.rs`
- Size bytes / Размер в байтах: `92`
- Included characters / Включено символов: `92`
- Truncated / Обрезано: `no`

```rust
pub(super) const MAX_REFRESH_LIMIT: i64 = 100;
pub(super) const MIN_REFRESH_LIMIT: i64 = 1;
```

### `backend/src/domains/decisions/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/errors.rs`
- Size bytes / Размер в байтах: `1651`
- Included characters / Включено символов: `1651`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::decisions::DecisionEngineError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum DecisionStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be a JSON array")]
    InvalidJsonArray(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("decision evidence is required")]
    MissingEvidence,

    #[error("observation decision evidence must use the same source_id and observation_id")]
    InvalidObservationEvidenceSource,

    #[error("decision evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("decision was not found")]
    DecisionNotFound,

    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("decided_by entity kind and id must be provided together")]
    PartialDecider,

    #[error("unknown decision entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown decision evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown decision status stored in database: {0}")]
    UnknownStatus(String),

    #[error("unknown decision review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error(transparent)]
    DecisionEngine(#[from] DecisionEngineError),
}
```

### `backend/src/domains/decisions/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/evidence.rs`
- Size bytes / Размер в байтах: `1314`
- Included characters / Включено символов: `1314`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{
    ObservationStoreError, link_domain_entity_in_transaction,
    materialize_review_transition_link_in_transaction,
};

use super::models::DecisionReviewState;

pub(crate) async fn link_decision_support_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    decision_id: impl Into<String>,
    confidence: f64,
    metadata: Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "decisions",
        "decision",
        decision_id.into(),
        Some("supports"),
        Some(confidence),
        Some(metadata),
    )
    .await
}

pub(crate) async fn link_decision_review_transition_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    decision_id: &str,
    review_state: DecisionReviewState,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    materialize_review_transition_link_in_transaction(
        transaction,
        observation_id,
        "decisions",
        "decision",
        decision_id,
        "review_state",
        review_state.as_str(),
        metadata,
    )
    .await
}
```

### `backend/src/domains/decisions/extraction.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/extraction.rs`
- Size bytes / Размер в байтах: `276`
- Included characters / Включено символов: `276`
- Truncated / Обрезано: `no`

```rust
mod detection;
mod engine;
mod errors;
mod models;

pub use engine::DecisionEngine;
pub use errors::DecisionEngineError;
pub use models::{
    DecisionCandidate, DecisionCandidateKind, DecisionExtractionInput, DecisionExtractionResult,
    DecisionImpactedEntityCandidate,
};
```

### `backend/src/domains/decisions/extraction/detection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/extraction/detection.rs`
- Size bytes / Размер в байтах: `3079`
- Included characters / Включено символов: `3079`
- Truncated / Обрезано: `no`

```rust
use super::models::{
    DecisionCandidate, DecisionCandidateKind, DecisionExtractionInput,
    DecisionImpactedEntityCandidate,
};
use crate::domains::decisions::DecisionReviewState;

pub fn detect_decision(
    input: &DecisionExtractionInput,
    sentence: &str,
) -> Option<DecisionCandidate> {
    let normalized_sentence = sentence.trim();
    let lower = normalized_sentence.to_lowercase();

    let (kind, body_start, confidence) = if lower.starts_with("decision:") {
        (
            DecisionCandidateKind::ExplicitDecision,
            "decision:".len(),
            0.83,
        )
    } else if lower.starts_with("we decided to ") {
        (
            DecisionCandidateKind::ExplicitDecision,
            "we decided to ".len(),
            0.78,
        )
    } else if lower.starts_with("approved:") {
        (DecisionCandidateKind::Approval, "approved:".len(), 0.74)
    } else if lower.starts_with("confirmed:") {
        (
            DecisionCandidateKind::Confirmation,
            "confirmed:".len(),
            0.72,
        )
    } else {
        return None;
    };

    let body = normalized_sentence[body_start..]
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .trim();
    if body.is_empty() {
        return None;
    }

    let (title, rationale) = split_rationale(body);
    if title.len() < 3 || rationale.len() < 3 {
        return None;
    }

    Some(DecisionCandidate {
        kind,
        title,
        rationale,
        quote: ensure_sentence_terminator(normalized_sentence),
        confidence,
        review_state: DecisionReviewState::Suggested,
        evidence_source_kind: input.source_kind,
        evidence_source_id: input.source_id.clone(),
        evidence_observation_id: input.observation_id.clone(),
        decided_by_entity_kind: input.decided_by_entity_kind,
        decided_by_entity_id: input.decided_by_entity_id.clone(),
        impacted_entities: vec![DecisionImpactedEntityCandidate {
            entity_kind: input.impacted_entity_kind,
            entity_id: input.impacted_entity_id.clone(),
            impact_type: "decision_context".to_owned(),
        }],
    })
}

fn split_rationale(value: &str) -> (String, String) {
    let lower = value.to_lowercase();
    for marker in [" because ", " so that "] {
        if let Some(index) = lower.find(marker) {
            let title = value[..index].trim().to_owned();
            let rationale = value[index + marker.len()..].trim().to_owned();
            if !title.is_empty() && !rationale.is_empty() {
                return (title, rationale);
            }
        }
    }

    (value.trim().to_owned(), value.trim().to_owned())
}

pub fn sentences(text: &str) -> Vec<&str> {
    text.split_terminator(['\n', '.', '!', '?'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

pub fn ensure_sentence_terminator(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with(['.', '!', '?']) {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.")
    }
}
```

### `backend/src/domains/decisions/extraction/engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/extraction/engine.rs`
- Size bytes / Размер в байтах: `664`
- Included characters / Включено символов: `664`
- Truncated / Обрезано: `no`

```rust
use super::detection::{detect_decision, sentences};
use super::errors::DecisionEngineError;
use super::models::{DecisionExtractionInput, DecisionExtractionResult};

pub struct DecisionEngine;

impl DecisionEngine {
    pub fn detect_candidates(
        input: &DecisionExtractionInput,
    ) -> Result<DecisionExtractionResult, DecisionEngineError> {
        input.validate()?;

        let mut result = DecisionExtractionResult::default();
        for sentence in sentences(&input.text) {
            if let Some(candidate) = detect_decision(input, sentence) {
                result.decisions.push(candidate);
            }
        }

        Ok(result)
    }
}
```

### `backend/src/domains/decisions/extraction/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/extraction/errors.rs`
- Size bytes / Размер в байтах: `241`
- Included characters / Включено символов: `241`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecisionEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("decided_by entity kind and id must be provided together")]
    PartialDecider,
}
```

### `backend/src/domains/decisions/extraction/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/extraction/models.rs`
- Size bytes / Размер в байтах: `6376`
- Included characters / Включено символов: `6376`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::errors::DecisionEngineError;
use crate::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, NewDecision,
    NewDecisionEvidence, NewDecisionImpactedEntity,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionExtractionInput {
    pub source_kind: DecisionEvidenceSourceKind,
    pub source_id: String,
    pub text: String,
    pub observation_id: Option<String>,
    pub impacted_entity_kind: DecisionEntityKind,
    pub impacted_entity_id: String,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
}

impl DecisionExtractionInput {
    pub fn communication(
        source_id: impl Into<String>,
        text: impl Into<String>,
        impacted_entity_kind: DecisionEntityKind,
        impacted_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: DecisionEvidenceSourceKind::Communication,
            source_id: source_id.into(),
            text: text.into(),
            observation_id: None,
            impacted_entity_kind,
            impacted_entity_id: impacted_entity_id.into(),
            decided_by_entity_kind: None,
            decided_by_entity_id: None,
        }
    }

    pub fn document(
        source_id: impl Into<String>,
        text: impl Into<String>,
        impacted_entity_kind: DecisionEntityKind,
        impacted_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: DecisionEvidenceSourceKind::Document,
            source_id: source_id.into(),
            text: text.into(),
            observation_id: None,
            impacted_entity_kind,
            impacted_entity_id: impacted_entity_id.into(),
            decided_by_entity_kind: None,
            decided_by_entity_id: None,
        }
    }

    pub fn decided_by(
        mut self,
        decided_by_entity_kind: DecisionEntityKind,
        decided_by_entity_id: impl Into<String>,
    ) -> Self {
        self.decided_by_entity_kind = Some(decided_by_entity_kind);
        self.decided_by_entity_id = Some(decided_by_entity_id.into());
        self
    }

    pub fn with_observation_id(mut self, observation_id: Option<String>) -> Self {
        self.observation_id = observation_id;
        self
    }

    pub fn validate(&self) -> Result<(), DecisionEngineError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("impacted_entity_id", &self.impacted_entity_id)?;
        if let Some(observation_id) = &self.observation_id {
            validate_non_empty("observation_id", observation_id)?;
        }
        match (
            self.decided_by_entity_kind,
            self.decided_by_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(decided_by_entity_id)) => {
                validate_non_empty("decided_by_entity_id", decided_by_entity_id)?;
            }
            _ => return Err(DecisionEngineError::PartialDecider),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecisionExtractionResult {
    pub decisions: Vec<DecisionCandidate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionCandidateKind {
    ExplicitDecision,
    Approval,
    Confirmation,
}

impl DecisionCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitDecision => "explicit_decision",
            Self::Approval => "approval",
            Self::Confirmation => "confirmation",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecisionCandidate {
    pub kind: DecisionCandidateKind,
    pub title: String,
    pub rationale: String,
    pub quote: String,
    pub confidence: f64,
    pub review_state: DecisionReviewState,
    pub evidence_source_kind: DecisionEvidenceSourceKind,
    pub evidence_source_id: String,
    pub evidence_observation_id: Option<String>,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub impacted_entities: Vec<DecisionImpactedEntityCandidate>,
}

impl DecisionCandidate {
    pub fn to_decision_draft(
        &self,
    ) -> (
        NewDecision,
        NewDecisionEvidence,
        Vec<NewDecisionImpactedEntity>,
    ) {
        let mut decision = NewDecision::new(
            self.title.clone(),
            self.rationale.clone(),
            self.confidence,
            self.review_state,
        )
        .metadata(json!({
            "engine": "decision",
            "candidate_kind": self.kind.as_str(),
        }));

        if let (Some(kind), Some(id)) = (self.decided_by_entity_kind, &self.decided_by_entity_id) {
            decision = decision.decided_by(kind, id.clone());
        }

        let evidence =
            NewDecisionEvidence::new(self.evidence_source_kind, self.evidence_source_id.clone())
                .with_observation_id(self.evidence_observation_id.clone())
                .quote(self.quote.clone())
                .confidence(self.confidence)
                .metadata(json!({
                    "engine": "decision",
                    "candidate_kind": self.kind.as_str(),
                }));

        let impacted_entities = self
            .impacted_entities
            .iter()
            .map(|entity| {
                NewDecisionImpactedEntity::new(entity.entity_kind, entity.entity_id.clone())
                    .impact_type(entity.impact_type.clone())
                    .metadata(json!({
                        "engine": "decision",
                        "candidate_kind": self.kind.as_str(),
                    }))
            })
            .collect();

        (decision, evidence, impacted_entities)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecisionImpactedEntityCandidate {
    pub entity_kind: DecisionEntityKind,
    pub entity_id: String,
    pub impact_type: String,
}

pub fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), DecisionEngineError> {
    if value.trim().is_empty() {
        return Err(DecisionEngineError::EmptyField(field_name));
    }

    Ok(())
}
```

### `backend/src/domains/decisions/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/ids.rs`
- Size bytes / Размер в байтах: `1217`
- Included characters / Включено символов: `1217`
- Truncated / Обрезано: `no`

```rust
use super::models::{DecisionEntityKind, DecisionEvidenceSourceKind, NewDecision};

pub fn decision_id(decision: &NewDecision) -> String {
    let title = normalize_text(&decision.title);
    let decider_kind = decision
        .decided_by_entity_kind
        .map(DecisionEntityKind::as_str)
        .unwrap_or("");
    let decider_id = decision.decided_by_entity_id.as_deref().unwrap_or("");
    let decided_at = decision
        .decided_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_default();

    format!(
        "decision:v1:{}:{}:{}:{}:{}:{}:{}:{}",
        title.len(),
        title,
        decider_kind.len(),
        decider_kind,
        decider_id.len(),
        decider_id,
        decided_at.len(),
        decided_at
    )
}

pub fn evidence_id(
    decision_id: &str,
    source_kind: DecisionEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "decision:evidence:v1:{}:{}:{}:{}:{}:{}",
        decision_id.len(),
        decision_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
```

### `backend/src/domains/decisions/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/mod.rs`
- Size bytes / Размер в байтах: `849`
- Included characters / Включено символов: `849`
- Truncated / Обрезано: `no`

```rust
mod candidate_refresh;
mod constants;
mod errors;
mod evidence;
mod extraction;
mod ids;
mod models;
pub mod ports;
mod row_mapping;
mod service;
mod store;
mod validation;

pub use errors::DecisionStoreError;
pub use errors::DecisionStoreError as DecisionReviewPortError;
pub use extraction::{
    DecisionCandidate, DecisionCandidateKind, DecisionEngine, DecisionEngineError,
    DecisionExtractionInput, DecisionExtractionResult, DecisionImpactedEntityCandidate,
};
pub use ids::{decision_id, evidence_id};
pub use models::{
    Decision, DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, DecisionStatus,
    NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};
pub use service::{DecisionCommandService, DecisionCommandServiceError};
pub use store::DecisionStore;
pub use store::DecisionStore as DecisionReviewPort;
```

### `backend/src/domains/decisions/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/models.rs`
- Size bytes / Размер в байтах: `375`
- Included characters / Включено символов: `375`
- Truncated / Обрезано: `no`

```rust
mod decision;
mod entity_kind;
mod evidence;
mod impacted_entity;
mod source_kind;
mod states;

pub use decision::{Decision, NewDecision};
pub use entity_kind::DecisionEntityKind;
pub use evidence::NewDecisionEvidence;
pub use impacted_entity::NewDecisionImpactedEntity;
pub use source_kind::DecisionEvidenceSourceKind;
pub use states::{DecisionReviewState, DecisionStatus};
```

### `backend/src/domains/decisions/models/decision.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/models/decision.rs`
- Size bytes / Размер в байтах: `3483`
- Included characters / Включено символов: `3483`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::super::errors::DecisionStoreError;
use super::super::validation::{
    validate_json_array, validate_json_object, validate_non_empty, validate_score,
};
use super::entity_kind::DecisionEntityKind;
use super::states::{DecisionReviewState, DecisionStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecision {
    pub title: String,
    pub status: DecisionStatus,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: DecisionReviewState,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewDecision {
    pub fn new(
        title: impl Into<String>,
        rationale: impl Into<String>,
        confidence: f64,
        review_state: DecisionReviewState,
    ) -> Self {
        Self {
            title: title.into(),
            status: DecisionStatus::Active,
            rationale: rationale.into(),
            alternatives: json!([]),
            decided_by_entity_kind: None,
            decided_by_entity_id: None,
            decided_at: None,
            review_state,
            confidence,
            metadata: json!({}),
        }
    }

    pub fn status(mut self, status: DecisionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn alternatives(mut self, alternatives: Value) -> Self {
        self.alternatives = alternatives;
        self
    }

    pub fn decided_by(
        mut self,
        decided_by_entity_kind: DecisionEntityKind,
        decided_by_entity_id: impl Into<String>,
    ) -> Self {
        self.decided_by_entity_kind = Some(decided_by_entity_kind);
        self.decided_by_entity_id = Some(decided_by_entity_id.into());
        self
    }

    pub fn decided_at(mut self, decided_at: DateTime<Utc>) -> Self {
        self.decided_at = Some(decided_at);
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(in crate::domains::decisions) fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("title", &self.title)?;
        validate_non_empty("rationale", &self.rationale)?;
        validate_score("confidence", self.confidence)?;
        validate_json_array("alternatives", &self.alternatives)?;
        validate_json_object("decision metadata", &self.metadata)?;

        match (
            self.decided_by_entity_kind,
            self.decided_by_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(decided_by_entity_id)) => {
                validate_non_empty("decided_by_entity_id", decided_by_entity_id)?;
            }
            _ => return Err(DecisionStoreError::PartialDecider),
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Decision {
    pub decision_id: String,
    pub title: String,
    pub status: DecisionStatus,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: DecisionReviewState,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### `backend/src/domains/decisions/models/entity_kind.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/models/entity_kind.rs`
- Size bytes / Размер в байтах: `1605`
- Included characters / Включено символов: `1605`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use super::super::errors::DecisionStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionEntityKind {
    Persona,
    Organization,
    Project,
    Communication,
    Document,
    Task,
    Event,
    Decision,
    Obligation,
    Knowledge,
}

impl DecisionEntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Persona => "persona",
            Self::Organization => "organization",
            Self::Project => "project",
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Task => "task",
            Self::Event => "event",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Knowledge => "knowledge",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, DecisionStoreError> {
        let value = value.as_ref().trim();
        match value {
            "persona" => Ok(Self::Persona),
            "organization" => Ok(Self::Organization),
            "project" => Ok(Self::Project),
            "communication" => Ok(Self::Communication),
            "document" => Ok(Self::Document),
            "task" => Ok(Self::Task),
            "event" => Ok(Self::Event),
            "decision" => Ok(Self::Decision),
            "obligation" => Ok(Self::Obligation),
            "knowledge" => Ok(Self::Knowledge),
            _ => Err(DecisionStoreError::UnknownEntityKind(value.to_owned())),
        }
    }
}
```

### `backend/src/domains/decisions/models/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/models/evidence.rs`
- Size bytes / Размер в байтах: `2559`
- Included characters / Включено символов: `2559`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::errors::DecisionStoreError;
use super::super::validation::{validate_json_object, validate_non_empty, validate_score};
use super::source_kind::DecisionEvidenceSourceKind;

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecisionEvidence {
    pub source_kind: DecisionEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub quote: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewDecisionEvidence {
    pub fn new(source_kind: DecisionEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            observation_id: None,
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn observation(observation_id: impl Into<String>) -> Self {
        let observation_id = observation_id.into();
        Self {
            source_kind: DecisionEvidenceSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id),
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_observation_id<T: Into<String>>(mut self, observation_id: Option<T>) -> Self {
        self.observation_id = observation_id.map(Into::into);
        self
    }

    pub(in crate::domains::decisions) fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        if let Some(observation_id) = &self.observation_id {
            validate_non_empty("observation_id", observation_id)?;
        }
        if self.source_kind == DecisionEvidenceSourceKind::Observation
            && self.observation_id.as_deref() != Some(self.source_id.as_str())
        {
            return Err(DecisionStoreError::InvalidObservationEvidenceSource);
        }
        validate_score("evidence confidence", self.confidence)?;
        validate_json_object("evidence metadata", &self.metadata)?;
        if let Some(quote) = &self.quote {
            validate_non_empty("quote", quote)?;
        }

        Ok(())
    }
}
```

### `backend/src/domains/decisions/models/impacted_entity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/models/impacted_entity.rs`
- Size bytes / Размер в байтах: `1235`
- Included characters / Включено символов: `1235`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::errors::DecisionStoreError;
use super::super::validation::{validate_json_object, validate_non_empty};
use super::entity_kind::DecisionEntityKind;

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecisionImpactedEntity {
    pub entity_kind: DecisionEntityKind,
    pub entity_id: String,
    pub impact_type: String,
    pub metadata: Value,
}

impl NewDecisionImpactedEntity {
    pub fn new(entity_kind: DecisionEntityKind, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind,
            entity_id: entity_id.into(),
            impact_type: "related".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn impact_type(mut self, impact_type: impl Into<String>) -> Self {
        self.impact_type = impact_type.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(in crate::domains::decisions) fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("entity_id", &self.entity_id)?;
        validate_non_empty("impact_type", &self.impact_type)?;
        validate_json_object("impact metadata", &self.metadata)
    }
}
```

### `backend/src/domains/decisions/models/source_kind.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/models/source_kind.rs`
- Size bytes / Размер в байтах: `1051`
- Included characters / Включено символов: `1051`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionEvidenceSourceKind {
    Observation,
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Relationship,
    Project,
    Organization,
    Persona,
}

impl DecisionEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Event => "event",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Task => "task",
            Self::Relationship => "relationship",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Persona => "persona",
        }
    }
}
```

### `backend/src/domains/decisions/models/states.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/models/states.rs`
- Size bytes / Размер в байтах: `1401`
- Included characters / Включено символов: `1401`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

use super::super::errors::DecisionStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Active,
    Superseded,
    Reversed,
    Deprecated,
}

impl DecisionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Superseded => "superseded",
            Self::Reversed => "reversed",
            Self::Deprecated => "deprecated",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl DecisionReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, DecisionStoreError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(DecisionStoreError::UnknownReviewState(value.to_owned())),
        }
    }
}
```

### `backend/src/domains/decisions/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/ports.rs`
- Size bytes / Размер в байтах: `59`
- Included characters / Включено символов: `59`
- Truncated / Обрезано: `no`

```rust
pub use super::store::DecisionStore as DecisionReviewPort;
```

### `backend/src/domains/decisions/row_mapping.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/decisions/row_mapping.rs`
- Size bytes / Размер в байтах: `1736`
- Included characters / Включено символов: `1736`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::DecisionStoreError;
use super::models::{Decision, DecisionEntityKind, DecisionReviewState, DecisionStatus};

pub(super) fn row_to_decision(row: PgRow) -> Result<Decision, DecisionStoreError> {
    let decided_by_entity_kind = row
        .try_get::<Option<String>, _>("decided_by_entity_kind")?
        .map(parse_entity_kind)
        .transpose()?;

    Ok(Decision {
        decision_id: row.try_get("decision_id")?,
        title: row.try_get("title")?,
        status: parse_status(row.try_get("status")?)?,
        rationale: row.try_get("rationale")?,
        alternatives: row.try_get("alternatives")?,
        decided_by_entity_kind,
        decided_by_entity_id: row.try_get("decided_by_entity_id")?,
        decided_at: row.try_get("decided_at")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<DecisionEntityKind, DecisionStoreError> {
    DecisionEntityKind::parse(value)
}

fn parse_status(value: String) -> Result<DecisionStatus, DecisionStoreError> {
    match value.as_str() {
        "active" => Ok(DecisionStatus::Active),
        "superseded" => Ok(DecisionStatus::Superseded),
        "reversed" => Ok(DecisionStatus::Reversed),
        "deprecated" => Ok(DecisionStatus::Deprecated),
        _ => Err(DecisionStoreError::UnknownStatus(value)),
    }
}

fn parse_review_state(value: String) -> Result<DecisionReviewState, DecisionStoreError> {
    DecisionReviewState::parse(value)
}
```
