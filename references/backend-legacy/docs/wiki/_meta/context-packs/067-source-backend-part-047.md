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

- Chunk ID / ID чанка: `067-source-backend-part-047`
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

### `backend/src/integrations/whatsapp/client/store/queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/store/queries.rs`
- Size bytes / Размер в байтах: `1292`
- Included characters / Включено символов: `1292`
- Truncated / Обрезано: `no`

```rust
use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::WhatsappWebMessage;
use crate::integrations::whatsapp::client::rows::provider_channel_message_to_whatsapp_web_message;
use crate::integrations::whatsapp::client::validation::validate_limit;

const WHATSAPP_WEB_CHANNEL_KINDS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];

impl WhatsappWebStore {
    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebMessage>, WhatsappWebError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        Ok(self
            .provider_channel_message_store()
            .recent_messages(
                account_id,
                provider_chat_id,
                WHATSAPP_WEB_CHANNEL_KINDS,
                limit,
            )
            .await?
            .into_iter()
            .map(provider_channel_message_to_whatsapp_web_message)
            .collect())
    }
}
```

### `backend/src/integrations/whatsapp/client/store/sessions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/store/sessions.rs`
- Size bytes / Размер в байтах: `7631`
- Included characters / Включено символов: `7631`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use super::WhatsappWebStore;
use super::evidence::link_whatsapp_entity_in_transaction;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{NewWhatsappWebSession, WhatsappWebSession};
use crate::integrations::whatsapp::client::rows::row_to_whatsapp_web_session;
use crate::integrations::whatsapp::client::validation::validate_limit;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

async fn capture_whatsapp_session_observation(
    transaction: &mut Transaction<'_, Postgres>,
    session: &WhatsappWebSession,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), WhatsappWebError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "WHATSAPP_WEB_SESSION",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "session_id": session.session_id,
                "account_id": session.account_id,
                "device_name": session.device_name,
                "companion_runtime": session.companion_runtime,
                "link_state": session.link_state,
                "local_state_path": session.local_state_path,
                "last_sync_at": session.last_sync_at,
                "metadata": session.metadata,
                "operation": relationship_kind,
            }),
            format!(
                "whatsapp-web-session://{}/{}",
                session.session_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_whatsapp_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "whatsapp_web_session",
        session.session_id.clone(),
        relationship_kind,
        json!({
            "account_id": session.account_id,
            "link_state": session.link_state,
            "companion_runtime": session.companion_runtime,
        }),
    )
    .await?;
    Ok(())
}

impl WhatsappWebStore {
    pub async fn upsert_session(
        &self,
        session: &NewWhatsappWebSession,
    ) -> Result<WhatsappWebSession, WhatsappWebError> {
        session.validate()?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO whatsapp_web_sessions (
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                device_name = EXCLUDED.device_name,
                companion_runtime = EXCLUDED.companion_runtime,
                link_state = EXCLUDED.link_state,
                local_state_path = EXCLUDED.local_state_path,
                last_sync_at = EXCLUDED.last_sync_at,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            "#,
        )
        .bind(session.session_id.trim())
        .bind(session.account_id.trim())
        .bind(session.device_name.trim())
        .bind(session.companion_runtime.as_str())
        .bind(session.link_state.as_str())
        .bind(session.local_state_path.trim())
        .bind(session.last_sync_at)
        .bind(&session.metadata)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_whatsapp_web_session(row)?;
        capture_whatsapp_session_observation(
            &mut transaction,
            &stored,
            "upsert",
            "whatsapp.client.store.upsert_session",
            stored.updated_at,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_sessions(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WhatsappWebSession>, WhatsappWebError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            FROM whatsapp_web_sessions
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY updated_at DESC, session_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_whatsapp_web_session).collect()
    }

    pub(in crate::integrations::whatsapp::client::store) async fn update_session_last_sync(
        &self,
        account_id: &str,
        last_sync_at: DateTime<Utc>,
    ) -> Result<(), WhatsappWebError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_web_sessions
            SET last_sync_at = GREATEST(COALESCE(last_sync_at, $2), $2),
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(last_sync_at)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let session = row_to_whatsapp_web_session(row)?;
            capture_whatsapp_session_observation(
                &mut transaction,
                &session,
                "sync_progress",
                "whatsapp.client.store.update_session_last_sync",
                session.updated_at,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub(in crate::integrations::whatsapp) async fn update_session_link_state(
        &self,
        account_id: &str,
        link_state: &str,
        actor: &str,
    ) -> Result<(), WhatsappWebError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE whatsapp_web_sessions
            SET link_state = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                session_id, account_id, device_name, companion_runtime,
                link_state, local_state_path, last_sync_at, metadata,
                created_at, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(link_state.trim())
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let session = row_to_whatsapp_web_session(row)?;
            capture_whatsapp_session_observation(
                &mut transaction,
                &session,
                "link_state_update",
                actor,
                session.updated_at,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
```

### `backend/src/integrations/whatsapp/client/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/validation.rs`
- Size bytes / Размер в байтах: `1245`
- Included characters / Включено символов: `1245`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::errors::WhatsappWebError;

pub(crate) fn validate_limit(limit: i64) -> Result<i64, WhatsappWebError> {
    if !(1..=100).contains(&limit) {
        return Err(WhatsappWebError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

pub(crate) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, WhatsappWebError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn validate_object(field: &'static str, value: &Value) -> Result<(), WhatsappWebError> {
    if !value.is_object() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub(crate) fn validate_string_array(
    field: &'static str,
    values: &[String],
) -> Result<(), WhatsappWebError> {
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must contain only non-empty strings"
        )));
    }
    Ok(())
}
```

### `backend/src/integrations/whatsapp/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/mod.rs`
- Size bytes / Размер в байтах: `33`
- Included characters / Включено символов: `33`
- Truncated / Обрезано: `no`

```rust
pub mod client;
pub mod runtime;
```

### `backend/src/integrations/whatsapp/runtime/business_cloud.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/runtime/business_cloud.rs`
- Size bytes / Размер в байтах: `38249`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;

use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;

use super::{
    ShapedWhatsAppProviderRuntime, WhatsAppProviderCommandExecutionError,
    WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderExecutableCommand,
    WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape, WhatsAppRuntimeHealth, WhatsappWebError,
    WhatsappWebStore,
};
use crate::platform::communications::{
    CommunicationProviderKind, ProviderAccount, ProviderAccountCommandPort,
    ProviderChannelMessageLookupPort, ProviderSecretBindingCommandPort,
};

pub(super) const BUSINESS_CLOUD_SMOKE_RUNTIME_KIND: &str = "business_cloud_smoke";
const BUSINESS_CLOUD_LIVE_SMOKE_OPT_IN_CONFIG_KEY: &str = "business_cloud_live_smoke_enabled";
const BUSINESS_CLOUD_GRAPH_API_VERSION_CONFIG_KEY: &str = "business_cloud_graph_api_version";
const BUSINESS_CLOUD_PHONE_NUMBER_ID_CONFIG_KEY: &str = "business_cloud_phone_number_id";
const BUSINESS_CLOUD_DEFAULT_GRAPH_API_VERSION: &str = "v24.0";
const BUSINESS_CLOUD_PUBLIC_AVAILABILITY_GATE: &str =
    "blocked_until_business_cloud_smoke_and_webhook_reconciliation";
const BUSINESS_CLOUD_VERIFIED_SUBMISSION_COMMANDS: &[&str] = &[
    "send_text",
    "send_template",
    "send_media",
    "send_voice_note",
];

pub(super) fn business_cloud_live_runtime_enabled() -> bool {
    false
}

pub(super) fn business_cloud_runtime_feature_blocker() -> &'static str {
    "whatsapp_business_cloud_runtime_feature_disabled"
}

pub(super) fn business_cloud_live_smoke_opted_in(config: &Value) -> bool {
    config
        .get(BUSINESS_CLOUD_LIVE_SMOKE_OPT_IN_CONFIG_KEY)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

#[derive(Clone)]
pub(super) struct BusinessCloudRuntimeManager {
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    http_client: reqwest::Client,
}

impl BusinessCloudRuntimeManager {
    pub(super) fn new(provider_account_store: Arc<dyn ProviderAccountCommandPort>) -> Self {
        Self {
            provider_account_store,
            http_client: reqwest::Client::new(),
        }
    }

    pub(super) async fn execute_live_provider_command(
        &self,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let account = self
            .lookup_account(&command.account_id)
            .await
            .map_err(|error| {
                WhatsAppProviderCommandExecutionError::new(
                    "business_cloud_account_lookup_failed",
                    error.to_string(),
                    Some(30),
                )
            })?;
        validate_business_cloud_account(&account)?;
        if account_runtime_kind(&account) != BUSINESS_CLOUD_SMOKE_RUNTIME_KIND
            || !business_cloud_live_smoke_opted_in(&account.config)
        {
            return Err(WhatsAppProviderCommandExecutionError::new(
                "whatsapp_business_cloud_live_smoke_opt_in_required",
                "Business Cloud live execution requires runtime=business_cloud_smoke and explicit account smoke opt-in",
                None,
            ));
        }
        if !business_cloud_submission_command_supported(&command.command_kind) {
            return Err(WhatsAppProviderCommandExecutionError::unsupported(
                &command.command_kind,
            ));
        }

        match command.command_kind.as_str() {
            "send_text" => self.execute_send_text(&account, command).await,
            "send_template" => self.execute_send_template(&account, command).await,
            "send_media" | "send_voice_note" => self.execute_send_media(&account, command).await,
            _ => Err(WhatsAppProviderCommandExecutionError::unsupported(
                &command.command_kind,
            )),
        }
    }

    pub(super) async fn decorate_runtime_health(
        &self,
        health: &mut WhatsAppRuntimeHealth,
        account_id: &str,
    ) {
        let manager_health = self.manager_health(account_id).await;
        health.checks["business_cloud_manager"] = manager_health.clone();
        health.checks["runtime"]["business_cloud_manager"] = manager_health;
    }

    async fn execute_send_text(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let access_token = command.api_access_token.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_access_token_unavailable",
                "Business Cloud command execution requires a host-vault access token binding",
                Some(30),
            )
        })?;
        let graph_api_version = business_cloud_graph_api_version(&account.config);
        let phone_number_id = business_cloud_phone_number_id(account)?;
        let recipient = required_command_value(&command.provider_chat_id, "provider_chat_id")?;
        let text = required_json_string(&command.payload, "text")?;
        let request_payload = json!({
            "messaging_product": "whatsapp",
            "to": recipient,
            "type": "text",
            "text": {
                "body": text,
                "preview_url": false,
            },
        });
        let response_payload = self
            .post_business_cloud_message(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                "send_text",
                &request_payload,
            )
            .await?;

        Ok(business_cloud_message_submitted_outcome(
            command,
            access_token.secret_ref(),
            &graph_api_version,
            &phone_number_id,
            &recipient,
            business_cloud_text_operation_metadata(&text),
            response_payload,
        ))
    }

    async fn execute_send_template(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let access_token = command.api_access_token.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_access_token_unavailable",
                "Business Cloud send_template requires a host-vault access token binding",
                Some(30),
            )
        })?;
        let graph_api_version = business_cloud_graph_api_version(&account.config);
        let phone_number_id = business_cloud_phone_number_id(account)?;
        let recipient = required_command_value(&command.provider_chat_id, "provider_chat_id")?;
        let template = business_cloud_template_payload(&command.payload)?;
        let request_payload = json!({
            "messaging_product": "whatsapp",
            "to": recipient,
            "type": "template",
            "template": template,
        });
        let response_payload = self
            .post_business_cloud_message(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                "send_template",
                &request_payload,
            )
            .await?;

        Ok(business_cloud_message_submitted_outcome(
            command,
            access_token.secret_ref(),
            &graph_api_version,
            &phone_number_id,
            &recipient,
            business_cloud_template_operation_metadata(&request_payload["template"]),
            response_payload,
        ))
    }

    async fn execute_send_media(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderExecutableCommand,
    ) -> Result<WhatsAppProviderCommandExecutionOutcome, WhatsAppProviderCommandExecutionError>
    {
        let access_token = command.api_access_token.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_access_token_unavailable",
                "Business Cloud media submission requires a host-vault access token binding",
                Some(30),
            )
        })?;
        let media_bytes = command.media_bytes.as_ref().ok_or_else(|| {
            WhatsAppProviderCommandExecutionError::new(
                "business_cloud_media_bytes_unavailable",
                "Business Cloud media submission requires redacted in-memory local blob bytes",
                Some(30),
            )
        })?;
        if media_bytes.is_empty() {
            return Err(WhatsAppProviderCommandExecutionError::new(
                "business_cloud_media_bytes_empty",
                "Business Cloud media upload received an empty local blob",
                None,
            ));
        }

        let graph_api_version = business_cloud_graph_api_version(&account.config);
        let phone_number_id = business_cloud_phone_number_id(account)?;
        let recipient = required_command_value(&command.provider_chat_id, "provider_chat_id")?;
        let content_type = required_json_string_for_command(
            command,
            "content_type",
            "Business Cloud media submission",
        )?;
        let media_type = business_cloud_media_type(command, &content_type)?;
        let media_filename = business_cloud_media_filename(command, &content_type);
        let uploaded_media_id = self
            .upload_business_cloud_media(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                &media_filename,
                &content_type,
                media_bytes.clone_bytes(),
            )
            .await?;
        let media_message_object =
            business_cloud_media_message_object(command, &media_type, &uploaded_media_id);
        let request_payload = json!({
            "messaging_product": "whatsapp",
            "to": recipient,
            "type": media_type.clone(),
            media_type.clone(): media_message_object,
        });
        let response_payload = self
            .post_business_cloud_message(
                access_token.expose_for_runtime(),
                &graph_api_version,
                &phone_number_id,
                &command.command_kind,
                &request_payload,
            )
            .await?;

        Ok(business_cloud_message_submitted_outcome(
            command,
            access_token.secret_ref(),
            &graph_api_version,
            &phone_number_id,
            &recipient,
            business_cloud_media_operation_metadata(
                command,
                &media_type,
                &content_type,
                media_bytes.len(),
                &media_filename,
                &uploaded_media_id,
            ),
            response_payload,
        ))
    }

    async fn post_business_cloud_message(
        &self,
        access_token: &str,
        graph_api_version: &str,
        phone_number_id: &str,
        operation: &str,
        request_payload: &Value,
    ) -> Result<Value, WhatsAppProviderCommandExecutionError> {
        let endpoint = business_cloud_messages_endpoint(graph_api_version, phone_number_id);
        let response = self
            .http_client
            .post(&endpoint)
            .bearer_auth(access_token)
            .json(request_payload)
            .send()
            .await
            .map_err(|error| {
                WhatsAppProviderCommandExecutionError::new(
                    "business_cloud_http_request_failed",
                    format!("Business Cloud {operation} request failed: {error}"),
                    Some(30),
                )
            })?;
        business_cloud_response_json(operation, response).await
    }

    async fn upload_business_cloud_media(
        &self,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/whatsapp/runtime/contracts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/runtime/contracts.rs`
- Size bytes / Размер в байтах: `34490`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::integrations::whatsapp::client::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebError, WhatsappWebMessage, WhatsappWebObservedCall,
    WhatsappWebObservedDialog, WhatsappWebObservedMedia, WhatsappWebObservedMessage,
    WhatsappWebObservedMessageDelete, WhatsappWebObservedMessageUpdate,
    WhatsappWebObservedParticipant, WhatsappWebObservedPresence, WhatsappWebObservedReaction,
    WhatsappWebObservedReceipt, WhatsappWebObservedRuntimeEvent, WhatsappWebObservedStatus,
    WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView, WhatsappWebSession,
};
use crate::platform::secrets::{SecretKind, SecretReferenceStore, SecretStoreKind};
use crate::vault::HostVault;

pub type WhatsAppProviderRuntimeFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, WhatsappWebError>> + Send + 'a>>;

pub type WhatsAppProviderCommandExecutionFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    WhatsAppProviderCommandExecutionOutcome,
                    WhatsAppProviderCommandExecutionError,
                >,
            > + Send
            + 'a,
    >,
>;

pub type WhatsAppRuntimeEventSinkFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), WhatsAppRuntimeEventSinkError>> + Send + 'a>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppProviderRuntimeShape {
    WebCompanion,
    NativeMultiDevice,
    BusinessCloud,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeBridgeDispatch {
    pub endpoint_path: &'static str,
    pub request_kind: &'static str,
    pub observed_source: &'static str,
}

impl WhatsAppRuntimeBridgeDispatch {
    pub fn new(
        endpoint_path: &'static str,
        request_kind: &'static str,
        observed_source: &'static str,
    ) -> Self {
        Self {
            endpoint_path,
            request_kind,
            observed_source,
        }
    }

    pub fn assert_runtime_bridge_contract(self) {
        debug_assert!(
            self.endpoint_path
                .starts_with("/api/v1/integrations/whatsapp/runtime-bridge/")
        );
        debug_assert!(!self.request_kind.trim().is_empty());
        debug_assert!(
            self.observed_source
                .starts_with("provider_observed.runtime_bridge_")
        );
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WhatsAppSanitizedRuntimeEventDto {
    pub account_id: String,
    pub provider_event_id: String,
    pub provider_shape: &'static str,
    pub runtime_driver: &'static str,
    pub provider_event_name: &'static str,
    pub event_family: &'static str,
    pub raw_record_kind: &'static str,
    pub raw_signal_event_kind: &'static str,
    pub accepted_event_kind: &'static str,
    pub source_fingerprint_seed: String,
    pub bridge_dispatch: WhatsAppRuntimeBridgeDispatch,
    pub metadata: Value,
}

impl WhatsAppSanitizedRuntimeEventDto {
    pub fn assert_event_spine_contract(&self) {
        debug_assert!(!self.account_id.trim().is_empty());
        debug_assert!(!self.provider_event_id.trim().is_empty());
        debug_assert_eq!(self.provider_shape, "whatsapp_native_md");
        debug_assert!(!self.runtime_driver.trim().is_empty());
        debug_assert!(!self.provider_event_name.trim().is_empty());
        debug_assert!(!self.event_family.trim().is_empty());
        debug_assert!(self.raw_record_kind.starts_with("whatsapp_web_"));
        debug_assert!(
            self.raw_signal_event_kind
                .starts_with("signal.raw.whatsapp.")
                && self.raw_signal_event_kind.ends_with(".observed")
        );
        debug_assert!(
            self.accepted_event_kind
                .starts_with("signal.accepted.whatsapp.")
        );
        debug_assert!(
            self.source_fingerprint_seed
                .starts_with("source_fingerprint:v5:")
        );
        self.bridge_dispatch.assert_runtime_bridge_contract();
        debug_assert_eq!(
            self.metadata.get("payload_policy").and_then(Value::as_str),
            Some("sanitized_metadata_only")
        );
        debug_assert_eq!(
            self.metadata
                .get("session_material")
                .and_then(Value::as_str),
            Some("excluded")
        );
        debug_assert_eq!(
            self.metadata
                .get("raw_provider_payload")
                .and_then(Value::as_str),
            Some("excluded")
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhatsAppRuntimeEventSinkError {
    pub code: &'static str,
}

impl WhatsAppRuntimeEventSinkError {
    pub fn new(code: &'static str) -> Self {
        Self { code }
    }
}

pub trait WhatsAppRuntimeEventSink: Send + Sync {
    fn accept<'a>(
        &'a self,
        dto: WhatsAppSanitizedRuntimeEventDto,
    ) -> WhatsAppRuntimeEventSinkFuture<'a>;
}

impl WhatsAppProviderRuntimeShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WebCompanion => "whatsapp_web_companion",
            Self::NativeMultiDevice => "whatsapp_native_md",
            Self::BusinessCloud => "whatsapp_business_cloud",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeStartRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeStopRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeRevokeRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeRelinkRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppRuntimeRemoveRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppQrLinkStartRequest {
    pub account_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppPairCodeStartRequest {
    pub account_id: String,
    pub phone_number: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppTextSendRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppReplyRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub reply_to_provider_message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppForwardRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub from_provider_chat_id: String,
    pub from_provider_message_id: String,
    pub text: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppEditRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppDeleteRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub confirmation_decision: Option<String>,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppReactionRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub reaction_emoji: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppConversationCommandRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub confirmation_decision: Option<String>,
    pub invite_link: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppStatusPublishRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppVoiceNoteSendRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub attachment_id: Option<String>,
    pub blob_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub scan_status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppMediaUploadRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub attachment_id: Option<String>,
    pub blob_id: String,
    pub media_type: String,
    pub caption: Option<String>,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub scan_status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsAppMediaDownloadRequest {
    pub command_id: Option<String>,
    pub idempotency_key: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_attachment_id: Option<String>,
    pub provider_media_id: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub status: String,
    pub fixture_runtime: bool,
    pub live_runtime_available: bool,
    pub live_send_available: bool,
    pub qr_pairing_available: bool,
    pub pair_code_available: bool,
    pub media_download_available: bool,
    pub media_upload_available: bool,
    pub session_restore_available: bool,
    pub session_secret_ref: Option<String>,
    pub runtime_blockers: Vec<String>,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeHealth {
    pub account_id: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub status: String,
    pub healthy: bool,
    pub checks: Value,
    pub checked_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppRuntimeRemoveResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub removed: bool,
    pub unbound_secret_refs: Vec<String>,
    pub removed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppQrLinkSession {
    pub account_id: String,
    pub provider_shape: String,
    pub runtime_kind: String,
    pub status: String,
    pub setup_id: String,
    pub qr_svg: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub runtime_blockers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsAppPairCodeSession {
    pub account_id: String,
    pub provider_shape: String,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/whatsapp/runtime/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/runtime/mod.rs`
- Size bytes / Размер в байтах: `239589`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
mod business_cloud;
mod contracts;
mod native_md;
mod web_companion;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use qrcode::QrCode;
use qrcode::render::svg;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use crate::integrations::whatsapp::client::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebError, WhatsappWebMessage, WhatsappWebObservedCall,
    WhatsappWebObservedDialog, WhatsappWebObservedMedia, WhatsappWebObservedMessage,
    WhatsappWebObservedMessageDelete, WhatsappWebObservedMessageUpdate,
    WhatsappWebObservedParticipant, WhatsappWebObservedPresence, WhatsappWebObservedReaction,
    WhatsappWebObservedReceipt, WhatsappWebObservedRuntimeEvent, WhatsappWebObservedStatus,
    WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView, WhatsappWebSession,
    WhatsappWebStore,
};
use crate::platform::communications::{
    CommunicationProviderKind, NewProviderAccountSecretBinding, ProviderAccount,
    ProviderAccountCommandPort, ProviderAccountSecretPurpose, ProviderChannelMessageLookupPort,
    ProviderSecretBindingCommandPort,
};
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretResolver, SecretStoreKind,
};
use crate::vault::{HostVault, SecretEntryContext};
pub use contracts::*;

pub const WHATSAPP_OUTBOX_WORKER_ID: &str = "whatsapp-outbox-worker";
const RETRY_BASE_DELAY_SECONDS: i64 = 30;
const RETRY_MAX_DELAY_SECONDS: i64 = 15 * 60;
const STALE_EXECUTION_LOCK_SECONDS: i64 = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WhatsAppProviderWriteCommand {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) command_kind: String,
    pub(crate) idempotency_key: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) capability_state: String,
    pub(crate) action_class: String,
    pub(crate) confirmation_decision: String,
    pub(crate) status: String,
    pub(crate) retry_count: i32,
    pub(crate) max_retries: i32,
    pub(crate) last_error: Option<String>,
    pub(crate) payload: Value,
    pub(crate) target_ref: Value,
    pub(crate) result_payload: Value,
    pub(crate) audit_metadata: Value,
    pub(crate) provider_state: Value,
    pub(crate) reconciliation_status: String,
    pub(crate) next_attempt_at: Option<DateTime<Utc>>,
    pub(crate) last_attempt_at: Option<DateTime<Utc>>,
    pub(crate) provider_observed_at: Option<DateTime<Utc>>,
    pub(crate) reconciled_at: Option<DateTime<Utc>>,
    pub(crate) dead_lettered_at: Option<DateTime<Utc>>,
    pub(crate) completed_at: Option<DateTime<Utc>>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

struct ProviderCommandInsert<'a> {
    command_id: String,
    account_id: &'a str,
    command_kind: &'a str,
    idempotency_key: String,
    provider_chat_id: &'a str,
    provider_message_id: Option<&'a str>,
    action_class: &'a str,
    confirmation_decision: &'a str,
    payload: Value,
    target_ref: Value,
    rendered_preview_hash: Option<String>,
    restored_session_secret_ref: Option<String>,
}

pub fn whatsapp_web_companion_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    web_companion::build_runtime(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
    )
}

struct ShapedWhatsAppProviderRuntime {
    provider_shape: WhatsAppProviderRuntimeShape,
    inner: Arc<dyn WhatsAppProviderRuntime>,
    native_md_manager: Option<native_md::NativeMdRuntimeManager>,
    business_cloud_manager: Option<business_cloud::BusinessCloudRuntimeManager>,
}

impl ShapedWhatsAppProviderRuntime {
    fn new(
        provider_shape: WhatsAppProviderRuntimeShape,
        inner: Arc<dyn WhatsAppProviderRuntime>,
    ) -> Self {
        Self {
            provider_shape,
            inner,
            native_md_manager: None,
            business_cloud_manager: None,
        }
    }

    fn with_native_md_manager(mut self, manager: native_md::NativeMdRuntimeManager) -> Self {
        self.native_md_manager = Some(manager);
        self
    }

    fn with_business_cloud_manager(
        mut self,
        manager: business_cloud::BusinessCloudRuntimeManager,
    ) -> Self {
        self.business_cloud_manager = Some(manager);
        self
    }
}

macro_rules! delegate_inner_secret_method {
    ($method:ident, $request_ty:ty, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            secret_store: &'a SecretReferenceStore,
            vault: &'a HostVault,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(secret_store, vault, request)
        }
    };
}

macro_rules! delegate_inner_accountless_secret_method {
    ($method:ident, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            secret_store: &'a SecretReferenceStore,
            vault: &'a HostVault,
            account_id: &'a str,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(secret_store, vault, account_id)
        }
    };
}

macro_rules! delegate_inner_request_method {
    ($method:ident, $request_ty:ty, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(request)
        }
    };
}

macro_rules! delegate_inner_fixture_method {
    ($method:ident, $request_ty:ty, $result_ty:ty) => {
        fn $method<'a>(
            &'a self,
            request: &'a $request_ty,
        ) -> WhatsAppProviderRuntimeFuture<'a, $result_ty> {
            self.inner.$method(request)
        }
    };
}

impl WhatsAppProviderRuntime for ShapedWhatsAppProviderRuntime {
    fn provider_shape(&self) -> WhatsAppProviderRuntimeShape {
        self.provider_shape
    }

    delegate_inner_accountless_secret_method!(runtime_status, WhatsAppRuntimeStatus);

    fn start_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                manager
                    .start_runtime(self.inner.as_ref(), secret_store, vault, request)
                    .await
            });
        }
        self.inner.start_runtime(secret_store, vault, request)
    }

    fn stop_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeStopRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner.stop_runtime(secret_store, vault, request).await
            });
        }
        self.inner.stop_runtime(secret_store, vault, request)
    }

    fn revoke_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRevokeRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner
                    .revoke_runtime(secret_store, vault, request)
                    .await
            });
        }
        self.inner.revoke_runtime(secret_store, vault, request)
    }

    fn relink_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRelinkRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeStatus> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner
                    .relink_runtime(secret_store, vault, request)
                    .await
            });
        }
        self.inner.relink_runtime(secret_store, vault, request)
    }

    fn remove_runtime<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppRuntimeRemoveRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeRemoveResponse> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let _ = manager.stop_account(&request.account_id).await;
                self.inner
                    .remove_runtime(secret_store, vault, request)
                    .await
            });
        }
        self.inner.remove_runtime(secret_store, vault, request)
    }

    fn runtime_health<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        account_id: &'a str,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppRuntimeHealth> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                let mut health = self
                    .inner
                    .runtime_health(secret_store, vault, account_id)
                    .await?;
                manager
                    .decorate_runtime_health(&mut health, account_id)
                    .await;
                Ok(health)
            });
        }
        if let Some(manager) = self.business_cloud_manager.as_ref() {
            return Box::pin(async move {
                let mut health = self
                    .inner
                    .runtime_health(secret_store, vault, account_id)
                    .await?;
                manager
                    .decorate_runtime_health(&mut health, account_id)
                    .await;
                Ok(health)
            });
        }
        self.inner.runtime_health(secret_store, vault, account_id)
    }
    fn start_qr_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppQrLinkStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppQrLinkSession> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                manager
                    .start_qr_link(self.inner.as_ref(), secret_store, vault, request)
                    .await
            });
        }
        self.inner.start_qr_link(secret_store, vault, request)
    }

    fn start_pair_code_link<'a>(
        &'a self,
        secret_store: &'a SecretReferenceStore,
        vault: &'a HostVault,
        request: &'a WhatsAppPairCodeStartRequest,
    ) -> WhatsAppProviderRuntimeFuture<'a, WhatsAppPairCodeSession> {
        if let Some(manager) = self.native_md_manager.as_ref() {
            return Box::pin(async move {
                manager
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/whatsapp/runtime/native_md.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/runtime/native_md.rs`
- Size bytes / Размер в байтах: `205501`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "whatsapp-native-md-runtime")]
use base64::Engine as _;
#[cfg(feature = "whatsapp-native-md-runtime")]
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
#[cfg(feature = "whatsapp-native-md-runtime")]
use std::collections::BTreeMap;
#[cfg(feature = "whatsapp-native-md-runtime")]
use std::str::FromStr;

use super::{
    ShapedWhatsAppProviderRuntime, WhatsAppPairCodeSession, WhatsAppPairCodeStartRequest,
    WhatsAppProviderCommandExecutionError, WhatsAppProviderCommandExecutionOutcome,
    WhatsAppProviderExecutableCommand, WhatsAppProviderInMemoryMediaBytes,
    WhatsAppProviderMediaDownloadRef, WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape,
    WhatsAppQrLinkSession, WhatsAppQrLinkStartRequest, WhatsAppRuntimeBridgeDispatch,
    WhatsAppRuntimeEventSink, WhatsAppRuntimeEventSinkError, WhatsAppRuntimeHealth,
    WhatsAppRuntimeStartRequest, WhatsAppRuntimeStatus, WhatsAppSanitizedRuntimeEventDto,
    WhatsappWebError, WhatsappWebStore,
};
use crate::platform::communications::{
    NewProviderAccountSecretBinding, ProviderAccount, ProviderAccountCommandPort,
    ProviderAccountSecretPurpose, ProviderChannelMessageLookupPort,
    ProviderSecretBindingCommandPort,
};
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use crate::vault::HostVault;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdRuntimeCommandChannel {
    DurableOutbox,
}

impl NativeMdRuntimeCommandChannel {
    fn as_str(self) -> &'static str {
        match self {
            Self::DurableOutbox => "durable_provider_command_outbox",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdRuntimeEventSink {
    SignalHubRawEvidence,
}

impl NativeMdRuntimeEventSink {
    fn as_str(self) -> &'static str {
        match self {
            Self::SignalHubRawEvidence => "signal_hub_raw_evidence",
        }
    }
}

const NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_WAIT_SECONDS: u64 = 10;
const NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_DEFAULT_TTL_SECONDS: i64 = 180;
const NATIVE_MD_RECONNECT_BASE_DELAY_SECONDS: i64 = 5;
const NATIVE_MD_RECONNECT_MAX_DELAY_SECONDS: i64 = 300;
const NATIVE_MD_RECONNECT_MAX_ATTEMPTS: u32 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeMdTransientAuthArtifactKind {
    QrCode,
    PairCode,
}

#[derive(Clone, Debug, PartialEq)]
struct NativeMdTransientAuthArtifact {
    kind: NativeMdTransientAuthArtifactKind,
    value: String,
    expires_at: DateTime<Utc>,
}

impl NativeMdTransientAuthArtifact {
    fn new(kind: NativeMdTransientAuthArtifactKind, value: String, timeout: Duration) -> Self {
        Self {
            kind,
            value,
            expires_at: native_md_auth_artifact_expires_at(timeout),
        }
    }
}

fn native_md_auth_artifact_expires_at(timeout: Duration) -> DateTime<Utc> {
    let effective_timeout = if timeout.as_secs() == 0 {
        Duration::from_secs(NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_DEFAULT_TTL_SECONDS as u64)
    } else {
        timeout
    };
    let ttl = chrono::Duration::from_std(effective_timeout).unwrap_or_else(|_| {
        chrono::Duration::seconds(NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_DEFAULT_TTL_SECONDS)
    });
    Utc::now() + ttl
}

#[cfg(feature = "whatsapp-native-md-runtime")]
#[derive(Clone, Default)]
struct NativeMdTransientAuthArtifacts {
    state: Arc<tokio::sync::Mutex<HashMap<String, NativeMdTransientAuthArtifact>>>,
}

#[cfg(feature = "whatsapp-native-md-runtime")]
impl NativeMdTransientAuthArtifacts {
    fn new() -> Self {
        Self::default()
    }

    async fn clear(&self, account_id: &str) {
        self.state.lock().await.remove(account_id);
    }

    async fn record_event(&self, account_id: &str, event: &wa_rs::types::events::Event) {
        use wa_rs::types::events::Event;

        let artifact = match event {
            Event::PairingQrCode { code, timeout } => Some(NativeMdTransientAuthArtifact::new(
                NativeMdTransientAuthArtifactKind::QrCode,
                code.clone(),
                *timeout,
            )),
            Event::PairingCode { code, timeout } => Some(NativeMdTransientAuthArtifact::new(
                NativeMdTransientAuthArtifactKind::PairCode,
                code.clone(),
                *timeout,
            )),
            _ => None,
        };

        if let Some(artifact) = artifact {
            self.state
                .lock()
                .await
                .insert(account_id.to_owned(), artifact);
        }
    }

    async fn wait_for(
        &self,
        account_id: &str,
        kind: NativeMdTransientAuthArtifactKind,
    ) -> Option<NativeMdTransientAuthArtifact> {
        let deadline = tokio::time::Instant::now()
            + Duration::from_secs(NATIVE_MD_TRANSIENT_AUTH_ARTIFACT_WAIT_SECONDS);
        loop {
            if let Some(artifact) = self.take_current(account_id, kind).await {
                return Some(artifact);
            }
            if tokio::time::Instant::now() >= deadline {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    async fn take_current(
        &self,
        account_id: &str,
        kind: NativeMdTransientAuthArtifactKind,
    ) -> Option<NativeMdTransientAuthArtifact> {
        let mut state = self.state.lock().await;
        let artifact = state.get(account_id).cloned()?;
        if artifact.expires_at <= Utc::now() {
            state.remove(account_id);
            return None;
        }
        if artifact.kind != kind {
            return None;
        }
        state.remove(account_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeMdRuntimeLifecycleSnapshot {
    lifecycle_state: &'static str,
    runtime_status: &'static str,
    severity: &'static str,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    next_reconnect_at: Option<DateTime<Utc>>,
    last_error_code: Option<String>,
    last_event_kind: &'static str,
    updated_at: DateTime<Utc>,
}

impl NativeMdRuntimeLifecycleSnapshot {
    #[allow(clippy::too_many_arguments)]
    fn new(
        lifecycle_state: &'static str,
        runtime_status: &'static str,
        severity: &'static str,
        reconnect_attempts: u32,
        next_reconnect_at: Option<DateTime<Utc>>,
        last_error_code: Option<String>,
        last_event_kind: &'static str,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            lifecycle_state,
            runtime_status,
            severity,
            reconnect_attempts,
            max_reconnect_attempts: NATIVE_MD_RECONNECT_MAX_ATTEMPTS,
            next_reconnect_at,
            last_error_code,
            last_event_kind,
            updated_at,
        }
    }

    fn stopped(now: DateTime<Utc>) -> Self {
        Self::new(
            "stopped",
            "stopped",
            "info",
            0,
            None,
            None,
            "runtime.actor.stopped",
            now,
        )
    }

    fn to_health_json(&self, now: DateTime<Utc>) -> Value {
        json!({
            "lifecycle_state": self.lifecycle_state,
            "runtime_status": self.runtime_status,
            "severity": self.severity,
            "reconnect_attempts": self.reconnect_attempts,
            "max_reconnect_attempts": self.max_reconnect_attempts,
            "next_reconnect_at": self.next_reconnect_at.map(|value| value.to_rfc3339()),
            "reconnect_due": self.next_reconnect_at.is_some_and(|deadline| deadline <= now),
            "last_error_code": self.last_error_code,
            "last_event_kind": self.last_event_kind,
            "updated_at": self.updated_at.to_rfc3339(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct NativeMdRuntimeLifecycleEvent {
    provider_event_name: &'static str,
    event_kind: &'static str,
    runtime_status: &'static str,
    lifecycle_state: &'static str,
    severity: &'static str,
    metadata: Value,
    observed_at: DateTime<Utc>,
}

impl NativeMdRuntimeLifecycleEvent {
    fn new(
        provider_event_name: &'static str,
        event_kind: &'static str,
        runtime_status: &'static str,
        lifecycle_state: &'static str,
        severity: &'static str,
        metadata: Value,
        observed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            provider_event_name,
            event_kind,
            runtime_status,
            lifecycle_state,
            severity,
            metadata,
            observed_at,
        }
    }

    fn to_dto(&self, account_id: &str) -> WhatsAppSanitizedRuntimeEventDto {
        native_md_synthetic_runtime_lifecycle_dto(account_id, self)
    }
}

#[derive(Clone, Default)]
struct NativeMdRuntimeLifecycleRegistry {
    state: Arc<tokio::sync::Mutex<HashMap<String, NativeMdRuntimeLifecycleSnapshot>>>,
}

impl NativeMdRuntimeLifecycleRegistry {
    fn new() -> Self {
        Self::default()
    }

    async fn record_start_requested(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        self.record_fixed_state(
            account_id,
            "NativeMdRuntimeActorStartRequested",
            "runtime.actor.start_requested",
            "degraded",
            "recovering",
            "info",
            None,
            None,
        )
        .await
    }

    async fn record_start_succeeded(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        self.record_fixed_state(
            account_id,
            "NativeMdRuntimeActorStarted",
            "runtime.actor.started",
            "degraded",
            "recovering",
            "info",
            None,
            None,
        )
        .await
    }

    async fn record_stopped(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        let now = Utc::now();
        self.state.lock().await.insert(
            account_id.to_owned(),
            NativeMdRuntimeLifecycleSnapshot::stopped(now),
        );
        NativeMdRuntimeLifecycleEvent::new(
            "NativeMdRuntimeActorStopped",
            "runtime.actor.stopped",
            "stopped",
            "stopped",
            "info",
            native_md_lifecycle_metadata(
                "runtime.actor.stopped",
                "stopped",
                "stopped",
                "info",
                json!({
                    "reconnect_policy": "disabled_after_explicit_stop",
                }),
            ),
            now,
        )
    }

    async fn record_reconnect_started(&self, account_id: &str) -> NativeMdRuntimeLifecycleEvent {
        self.record_fixed_state(
            account_id,
            "NativeMdRuntimeReconnectStarted",
            "connection.reconnect.started",
            "degraded",
            "recovering",
            "info",
            None,
            Some(json!({
                "reconnect_policy": "tick_driven_restart_from_vault_session",
            })),
        )
        .await
    }

    async fn record_start_failed(
        &self,
        account_id: &str,
        error_code: &'static str,
    ) -> NativeMdRuntimeLifecycleEvent {
        self.record_degraded(
            account_id,
            "NativeMdRuntimeActorStartFailed",
            "runtime.actor.start_failed",
            Some(error_code.to_owned()),
            true,
        )
        .await
    }

    async fn record_reconnect_failed(
        &self,
        account_id: &str,
        error_code: &'static str,
    ) -> NativeMdRuntimeLifecycleEvent {
        self.record_degraded(
            account_id,
            "NativeMdRuntimeReconnectFailed",
            "connection.reconnect.failed",
            Some(error_code.to_owned()),
            true,
        )
        .await
    }

    async fn reconnect_due(&self, account_id
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/whatsapp/runtime/web_companion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/runtime/web_companion.rs`
- Size bytes / Размер в байтах: `8242`
- Included characters / Включено символов: `8242`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::{
    ShapedWhatsAppProviderRuntime, WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape,
    WhatsappWebStore,
};
use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderChannelMessageLookupPort, ProviderSecretBindingCommandPort,
};

pub(crate) fn build_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    Arc::new(ShapedWhatsAppProviderRuntime::new(
        WhatsAppProviderRuntimeShape::WebCompanion,
        Arc::new(WhatsappWebStore::new(
            pool,
            provider_account_store,
            provider_secret_binding_store,
            provider_channel_message_store,
        )),
    ))
}

pub(super) fn web_companion_bridge_contract_health_check() -> Value {
    json!({
        "driver_id": "webview_companion_bridge",
        "readiness": "visible_desktop_producer_shell_with_runtime_event_dispatch_smoke_pending",
        "public_availability": false,
        "runtime_kind": "webview_companion",
        "provider_shape": WhatsAppProviderRuntimeShape::WebCompanion.as_str(),
        "desktop_producer": {
            "artifact": "frontend/src-tauri/src/whatsapp_companion.rs",
            "commands": [
                "open_whatsapp_web_companion",
                "whatsapp_web_companion_manifest",
                "whatsapp_web_companion_relay_observation"
            ],
            "driver_id": "tauri_visible_webview_companion",
            "window_label_prefix": "whatsapp-companion",
            "target_url": "https://web.whatsapp.com/",
            "owner_visible": true,
            "hidden_headless_mode": "forbidden",
            "tauri_ipc_available_to_companion_window": true,
            "tauri_ipc_scope": "allowlisted_runtime_event_relay_only",
            "event_extractor": "contract_injected_relay_dispatch_available",
            "public_availability": false
        },
        "event_extractor": {
            "state": "contract_injected_relay_dispatch_available",
            "artifact": "frontend/src-tauri/src/whatsapp_companion.rs",
            "capability_artifact": "frontend/src-tauri/capabilities/whatsapp-companion-relay.json",
            "initialization_script": "installed_on_visible_companion_window",
            "script_scope": "main_frame_only",
            "origin_guard": "https://web.whatsapp.com",
            "navigation_guard": "https://web.whatsapp.com_only",
            "relay_channel": "tauri_allowlisted_companion_runtime_bridge_dispatch",
            "runtime_bridge_dispatch": "runtime_events_bridge_wired_smoke_pending",
            "runtime_bridge_dispatch_path": "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            "dispatch_payload": "NewWhatsappWebRuntimeEvent",
            "relay_command": "whatsapp_web_companion_relay_observation",
            "relay_command_policy": {
                "remote_capability_url": "https://web.whatsapp.com",
                "window_label_pattern": "whatsapp-companion-*",
                "caller_window_label_must_match_account": true,
                "metadata_sanitizer": "secret_and_private_content_key_drop",
                "backend_auth": "X-Макошь-Secret_from_tauri_process_env_only",
                "backend_target": "loopback_http_runtime_bridge_only",
                "typed_projection": "not_attempted_until_richer_typed_payload",
                "domain_mutation": "forbidden",
                "command_completion": "forbidden"
            },
            "allowed_observations": [
                "runtime_lifecycle_metadata",
                "sync_lifecycle_metadata",
                "message_identity_metadata",
                "receipt_metadata",
                "reaction_metadata",
                "dialog_metadata",
                "participant_metadata",
                "presence_metadata",
                "call_metadata",
                "status_metadata",
                "media_metadata_without_bytes"
            ],
            "forbidden_reads": [
                "cookies",
                "web_storage",
                "indexed_db",
                "browser_profile_secrets",
                "session_material",
                "message_bodies",
                "media_bytes"
            ],
            "next_gate": "manual_live_smoke_before_public_availability"
        },
        "owner_visibility": {
            "visible_runtime_required": true,
            "hidden_headless_mode": "forbidden",
            "owner_controls_required": [
                "link",
                "stop",
                "revoke",
                "relink",
                "remove",
                "health",
                "command_audit"
            ]
        },
        "session_storage": {
            "binding_store": "host_vault",
            "binding_purpose": "whatsapp_web_session_key",
            "postgres_policy": "metadata_bindings_only_no_session_cookie_or_local_profile_secret"
        },
        "event_sink": {
            "kind": "protected_runtime_bridge_routes",
            "raw_evidence_policy": "append_only_sanitized_metadata",
            "accepted_event_policy": "signal_hub_acceptance_before_projection",
            "runtime_to_domain_calls": "forbidden",
            "required_event_families": [
                "runtime_lifecycle",
                "sync_lifecycle",
                "messages",
                "message_updates",
                "message_deletes",
                "receipts",
                "reactions",
                "dialogs",
                "participants",
                "presence",
                "calls_metadata",
                "statuses",
                "status_views",
                "status_deletes",
                "media_metadata",
                "media_lifecycle",
                "unsupported_evidence"
            ],
            "endpoint_paths": [
                "/api/v1/integrations/whatsapp/runtime-bridge/messages",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
                "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
                "/api/v1/integrations/whatsapp/runtime-bridge/reactions",
                "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
                "/api/v1/integrations/whatsapp/runtime-bridge/participants",
                "/api/v1/integrations/whatsapp/runtime-bridge/presence",
                "/api/v1/integrations/whatsapp/runtime-bridge/calls",
                "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
                "/api/v1/integrations/whatsapp/runtime-bridge/status-views",
                "/api/v1/integrations/whatsapp/runtime-bridge/status-deletes",
                "/api/v1/integrations/whatsapp/runtime-bridge/media",
                "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
                "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
                "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle"
            ]
        },
        "command_channel": {
            "kind": "durable_outbox",
            "claim_path": "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            "failure_path": "/api/v1/integrations/whatsapp/runtime-bridge/commands/{command_id}/failed",
            "completion_rule": "provider_observed_event_reconciliation_required"
        },
        "redaction_policy": {
            "session_material": "excluded",
            "cookies": "excluded",
            "browser_profile_secrets": "excluded",
            "qr_pair_code_artifacts": "transient_memory_only",
            "message_bodies_in_health": "excluded",
            "media_bytes": "excluded"
        },
        "blockers": [
            "whatsapp_visible_runtime_required",
            "whatsapp_webview_runtime_panel_action_not_implemented",
            "manual_live_smoke_required"
        ]
    })
}
```

### `backend/src/integrations/yandex_telemost/client/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/client/errors.rs`
- Size bytes / Размер в байтах: `1556`
- Included characters / Включено символов: `1556`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use std::io;

use crate::domains::review::ReviewInboxError;
use crate::platform::communications::{ProviderAccountPortError, ProviderSecretBindingPortError};
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;
use crate::platform::secrets::{SecretReferenceError, SecretResolutionError};
use crate::platform::settings::SettingsError;
use crate::vault::HostVaultError;

#[derive(Debug, Error)]
pub enum YandexTelemostError {
    #[error("invalid Yandex Telemost request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    ProviderAccountStore(#[from] ProviderAccountPortError),

    #[error(transparent)]
    ProviderSecretBindingStore(#[from] ProviderSecretBindingPortError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Settings(#[from] SettingsError),
}
```

### `backend/src/integrations/yandex_telemost/client/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/client/mod.rs`
- Size bytes / Размер в байтах: `400`
- Included characters / Включено символов: `400`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod store;
mod validation;

pub use errors::YandexTelemostError;
pub use models::*;
pub use store::{YandexTelemostHttpClient, YandexTelemostStore};
pub use validation::{
    sanitize_yandex_telemost_payload, validate_telemost_join_url,
    yandex_telemost_oauth_token_secret_ref,
};
pub(crate) use validation::{validate_api_base_url, validate_json_object, validate_required};
```

### `backend/src/integrations/yandex_telemost/client/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/client/models.rs`
- Size bytes / Размер в байтах: `18470`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::integrations::yandex_telemost::client::YandexTelemostError;
use crate::platform::communications::{CommunicationProviderKind, ProviderAccount};

pub const YANDEX_TELEMOST_PROVIDER_KIND_STR: &str = "yandex_telemost_user";
pub const YANDEX_TELEMOST_RUNTIME_KIND: &str = "yandex_telemost_webview_runtime";
pub const YANDEX_TELEMOST_LIVE_RUNTIME_KIND: &str = "yandex_telemost_live_authorized_runtime";
pub const YANDEX_TELEMOST_API_BASE_URL: &str = "https://cloud-api.yandex.net/v1/telemost-api";
pub const YANDEX_TELEMOST_WEB_ORIGIN: &str = "https://telemost.yandex.ru";

pub const YANDEX_TELEMOST_CAP_CONFERENCE_CREATE: &str = "telemost.conferences.create";
pub const YANDEX_TELEMOST_CAP_CONFERENCE_READ: &str = "telemost.conferences.read";
pub const YANDEX_TELEMOST_CAP_CONFERENCE_UPDATE: &str = "telemost.conferences.update";
pub const YANDEX_TELEMOST_CAP_COHOSTS_READ: &str = "telemost.cohosts.read";
pub const YANDEX_TELEMOST_CAP_WEBVIEW_OPEN: &str = "telemost.webview.open";
pub const YANDEX_TELEMOST_CAP_LOCAL_RECORDING: &str = "telemost.local_recording.mp3";
pub const YANDEX_TELEMOST_CAP_SPEAKER_TIMELINE_HINTS: &str =
    "telemost.speaker_timeline.webview_hints";

fn default_json_object() -> Value {
    json!({})
}

fn default_json_array() -> Value {
    json!([])
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostAccount {
    pub account_id: String,
    pub provider_kind: String,
    pub display_name: String,
    pub external_account_id: String,
    pub lifecycle_state: String,
    pub runtime_kind: String,
    pub api_base_url: String,
    pub token_secret_ref: Option<String>,
    pub join_webview_available: bool,
    pub local_recorder_available: bool,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProviderAccount> for YandexTelemostAccount {
    fn from(account: ProviderAccount) -> Self {
        let config = account.config.clone();
        Self {
            account_id: account.account_id,
            provider_kind: account.provider_kind.as_str().to_owned(),
            display_name: account.display_name,
            external_account_id: account.external_account_id,
            lifecycle_state: config
                .get("lifecycle_state")
                .and_then(Value::as_str)
                .unwrap_or("blocked")
                .to_owned(),
            runtime_kind: config
                .get("runtime_kind")
                .and_then(Value::as_str)
                .unwrap_or(YANDEX_TELEMOST_RUNTIME_KIND)
                .to_owned(),
            api_base_url: config
                .get("api_base_url")
                .and_then(Value::as_str)
                .unwrap_or(YANDEX_TELEMOST_API_BASE_URL)
                .to_owned(),
            token_secret_ref: config
                .get("token_secret_ref")
                .and_then(Value::as_str)
                .map(str::to_owned),
            join_webview_available: config
                .get("join_webview_available")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            local_recorder_available: config
                .get("local_recorder_available")
                .and_then(Value::as_bool)
                .unwrap_or(true),
            config,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostAccountSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    #[serde(default)]
    pub oauth_token: Option<String>,
    #[serde(default)]
    pub oauth_token_ref: Option<String>,
    #[serde(default)]
    pub api_base_url: Option<String>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostAccountSetupResponse {
    pub account: YandexTelemostAccount,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostAccountListResponse {
    pub items: Vec<YandexTelemostAccount>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub lifecycle_state: String,
    pub runtime_kind: String,
    pub checked_at: DateTime<Utc>,
    pub api_base_url: String,
    pub authorized: bool,
    pub blockers: Vec<String>,
    pub capabilities: Vec<YandexTelemostCapabilityState>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostCapabilityState {
    pub capability: String,
    pub status: String,
    pub source: String,
    pub confidence: f32,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TelemostCohost {
    pub email: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TelemostLiveStreamRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TelemostLiveStreamResponse {
    #[serde(default)]
    pub watch_url: Option<String>,
    #[serde(default)]
    pub access_level: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConferenceRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_room_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_stream: Option<TelemostLiveStreamRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cohosts: Vec<TelemostCohost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_summarization_enabled: Option<bool>,
    #[serde(default = "default_json_object", skip_serializing)]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConferencePatchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_room_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_stream: Option<TelemostLiveStreamRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cohosts: Vec<TelemostCohost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auto_summarization_enabled: Option<bool>,
    #[serde(default = "default_json_object", skip_serializing)]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConference {
    pub id: String,
    pub join_url: String,
    #[serde(default)]
    pub access_level: Option<String>,
    #[serde(default)]
    pub waiting_room_level: Option<String>,
    #[serde(default)]
    pub live_stream: Option<TelemostLiveStreamResponse>,
    #[serde(default)]
    pub sip_uri_meeting: Option<String>,
    #[serde(default)]
    pub sip_uri_telemost: Option<String>,
    #[serde(default)]
    pub sip_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostCreateConferenceCommand {
    pub account_id: String,
    pub body: YandexTelemostConferenceRequest,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostCohostPage {
    #[serde(default)]
    pub cohosts: Vec<TelemostCohost>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostConferenceOpenRequest {
    pub account_id: String,
    #[serde(default)]
    pub conference_id: Option<String>,
    pub join_url: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostConferenceWebviewManifest {
    pub account_id: String,
    pub conference_id: Option<String>,
    pub join_url: String,
    pub target_origin: &'static str,
    pub provider_shape: &'static str,
    pub runtime_kind: &'static str,
    pub window_label: String,
    pub opened_window: bool,
    pub focused_existing_window: bool,
    pub owner_visible: bool,
    pub hidden_headless_mode: &'static str,
    pub local_recording: YandexTelemostLocalRecordingManifest,
    pub speaker_timeline: YandexTelemostSpeakerTimelinePolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostLocalRecordingManifest {
    pub state: &'static str,
    pub audio_format: &'static str,
    pub recorder_boundary: &'static str,
    pub consent_required: bool,
    pub default_output_policy: &'static str,
    pub audio_device_policy: YandexTelemostLocalRecordingPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostLocalRecordingPolicy {
    pub macos: &'static str,
    pub linux: &'static str,
    pub windows: &'static str,
    pub ffmpeg_path_env: &'static str,
    pub ffmpeg_input_env: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct YandexTelemostSpeakerTimelinePolicy {
    pub state: &'static str,
    pub source: &'static str,
    pub reliability: &'static str,
    pub output_files: Vec<&'static str>,
    pub role_in_transcription: &'static str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostRecordingBridgeRequest {
    pub account_id: String,
    #[serde(default)]
    pub conference_id: Option<String>,
    pub join_url: String,
    pub recording_session_id: String,
    pub output_dir: String,
    pub audio_path: String,
    pub speaker_jsonl_path: String,
    pub speaker_txt_path: String,
    pub started_at_epoch_ms: u128,
    pub stopped_at_epoch_ms: u128,
    pub consent_attested: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostRecordingBridgeResponse {
    pub account_id: String,
    pub conference_id: Option<String>,
    pub recording_session_id: String,
    pub bundle_id: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub follow_up_events: Vec<String>,
    pub radar_signal_kinds: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct YandexTelemostTranscriptBridgeRequest {
    pub account_id: String,
    #[serde(default)]
    pub conference_id: Option<String>,
    pub bundle_id: String,
    pub bundle_root: String,
    pub transcript_text: String,
    #[serde(default = "default_json_array")]
    pub segments: Value,
    #[serde(default)]
    pub language_code: Option<String>,
    pub stt_provider: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default = "default_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct YandexTelemostTranscriptBridgeResponse {
    pub account_id: String,
    pub conference_id: Option<String>,
    pub bundle_id: String,
    pub manifest_path: String,
    pub transcript_json_path: String,
    pub transcript_markdown_path: String,
    pub summary_markdown_path: Option<String>,
    pub follow_up_events: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct YandexTelemostRetentionCleanupRequest {
    #[serde(default = "default_true")]
    pub remove_audio: bool,
    #[serde(default = "default_true")]
    pub remove_speaker_hints: bool,
    #[serde(default = "default_yandex_telemost_retention_cleanup_limit")]
    pub limit: i64,
}

impl YandexTelemostRetentionCleanupRequest {
    pub fn validate(&self) -> Result<(), YandexTelemostError> {
        if !self.remove_audio
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/yandex_telemost/client/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/client/store.rs`
- Size bytes / Размер в байтах: `42019`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::platform::communications::{
    NewProviderAccountSecretBinding, ProviderAccount, ProviderAccountCommandPort,
    ProviderAccountSecretPurpose, ProviderSecretBindingCommandPort,
};
use crate::platform::events::bus::yandex_telemost_event_types;
use crate::platform::events::{EventBus, EventLogQuery, EventStore, NewEventEnvelope};
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use crate::platform::settings::ApplicationSettingsStore;
use crate::vault::{HostVault, SecretEntryContext};

use super::{
    TelemostCohost, YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_LIVE_RUNTIME_KIND,
    YANDEX_TELEMOST_PROVIDER_KIND_STR, YANDEX_TELEMOST_RUNTIME_KIND, YandexTelemostAccount,
    YandexTelemostAccountListResponse, YandexTelemostAccountSetupRequest,
    YandexTelemostAccountSetupResponse, YandexTelemostCapabilityState, YandexTelemostCohostPage,
    YandexTelemostConference, YandexTelemostConferencePatchRequest,
    YandexTelemostConferenceRequest, YandexTelemostError, YandexTelemostRetentionCleanupItem,
    YandexTelemostRetentionCleanupRequest, YandexTelemostRetentionCleanupResponse,
    YandexTelemostRuntimeStatus, sanitize_yandex_telemost_payload, telemost_provider_kind,
    validate_api_base_url, validate_json_object, validate_required, yandex_telemost_capabilities,
    yandex_telemost_default_config, yandex_telemost_oauth_token_secret_ref,
};

const YANDEX_TELEMOST_RECORDING_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.yandex_telemost_recording_retention_days";
const YANDEX_TELEMOST_SPEAKER_TIMELINE_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.yandex_telemost_speaker_timeline_retention_days";

#[derive(Clone)]
pub struct YandexTelemostHttpClient {
    http: reqwest::Client,
    base_url: String,
}

impl YandexTelemostHttpClient {
    pub fn new(base_url: Option<&str>) -> Result<Self, YandexTelemostError> {
        Ok(Self {
            http: reqwest::Client::new(),
            base_url: validate_api_base_url(base_url)?,
        })
    }

    pub async fn create_conference(
        &self,
        oauth_token: &str,
        request: &YandexTelemostConferenceRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        validate_conference_request(request)?;
        let payload = provider_payload(request)?;
        let response = self
            .http
            .post(format!("{}/conferences", self.base_url))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn get_conference(
        &self,
        oauth_token: &str,
        conference_id: &str,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        let response = self
            .http
            .get(format!("{}/conferences/{}", self.base_url, conference_id))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn update_conference(
        &self,
        oauth_token: &str,
        conference_id: &str,
        request: &YandexTelemostConferencePatchRequest,
    ) -> Result<YandexTelemostConference, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        validate_conference_patch_request(request)?;
        let payload = provider_payload(request)?;
        let response = self
            .http
            .patch(format!("{}/conferences/{}", self.base_url, conference_id))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json::<YandexTelemostConference>().await?)
    }

    pub async fn list_cohosts(
        &self,
        oauth_token: &str,
        conference_id: &str,
        offset: Option<u32>,
        limit: Option<u16>,
    ) -> Result<YandexTelemostCohostPage, YandexTelemostError> {
        let conference_id = validate_required("conference_id", conference_id)?;
        let mut request = self
            .http
            .get(format!(
                "{}/conferences/{}/cohosts",
                self.base_url, conference_id
            ))
            .header(AUTHORIZATION, format!("OAuth {}", oauth_token.trim()))
            .header(ACCEPT, "application/json");
        if let Some(offset) = offset {
            request = request.query(&[("offset", offset)]);
        }
        if let Some(limit) = limit {
            request = request.query(&[("limit", limit)]);
        }
        let response = request.send().await?.error_for_status()?;
        Ok(response.json::<YandexTelemostCohostPage>().await?)
    }
}

#[derive(Clone)]
pub struct YandexTelemostStore {
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    event_store: EventStore,
    event_bus: EventBus,
}

impl YandexTelemostStore {
    pub fn new(
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        event_store: EventStore,
        event_bus: EventBus,
    ) -> Self {
        Self {
            provider_account_store,
            provider_secret_binding_store,
            event_store,
            event_bus,
        }
    }

    pub async fn setup_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &YandexTelemostAccountSetupRequest,
    ) -> Result<YandexTelemostAccountSetupResponse, YandexTelemostError> {
        validate_account_setup_request(request)?;
        let account_id = validate_required("account_id", &request.account_id)?;
        let display_name = validate_required("display_name", &request.display_name)?;
        let external_account_id =
            validate_required("external_account_id", &request.external_account_id)?;
        let api_base_url = validate_api_base_url(request.api_base_url.as_deref())?;
        let token_secret_ref = self
            .store_or_register_oauth_token(secret_store, vault, &account_id, request)
            .await?;
        let config = merge_metadata(
            yandex_telemost_default_config(Some(&token_secret_ref), &api_base_url),
            &request.metadata,
        );
        let account = self
            .provider_account_store
            .upsert_runtime_account(
                account_id.clone(),
                telemost_provider_kind().as_str().to_owned(),
                display_name,
                external_account_id,
                config,
            )
            .await?;
        self.provider_secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
                &token_secret_ref,
            ))
            .await?;
        self.publish_account_configured_event(&account, &token_secret_ref)
            .await?;
        self.publish_authorization_event(&account, &token_secret_ref)
            .await?;
        Ok(YandexTelemostAccountSetupResponse {
            account: account.into(),
        })
    }

    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<YandexTelemostAccountListResponse, YandexTelemostError> {
        let mut items = self
            .provider_account_store
            .list()
            .await?
            .into_iter()
            .filter(|account| account.provider_kind.is_yandex_telemost())
            .map(YandexTelemostAccount::from)
            .filter(|account| include_removed || account.lifecycle_state != "removed")
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(YandexTelemostAccountListResponse { items })
    }

    pub async fn runtime_status(
        &self,
        account_id: &str,
    ) -> Result<YandexTelemostRuntimeStatus, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        let authorized = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::YandexTelemostOauthToken,
            )
            .await?
            .is_some();
        let status = runtime_status_from_account(account.into(), authorized);
        self.publish_runtime_status_event(&status).await?;
        Ok(status)
    }

    pub async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &YandexTelemostRetentionCleanupRequest,
    ) -> Result<YandexTelemostRetentionCleanupResponse, YandexTelemostError> {
        let account = self.telemost_account(account_id).await?;
        request.validate()?;
        let checked_at = Utc::now();
        let mut response = YandexTelemostRetentionCleanupResponse {
            account_id: account.account_id.clone(),
            checked_at,
            audio_files_removed: 0,
            speaker_hint_files_removed: 0,
            bundles_cleaned: 0,
            items: Vec::new(),
        };
        let events = self
            .event_store
            .list_matching(
                EventLogQuery::default()
                    .event_type(yandex_telemost_event_types::LOCAL_RECORDING_COMPLETED)
                    .limit(1000),
            )
            .await?;

        for event in events {
            if response.items.len() >= request.limit() as usize {
                break;
            }
            let Some(candidate) = retention_cleanup_candidate_from_event(
                &event.event.payload,
                event.event.occurred_at,
            ) else {
                continue;
            };
            if candidate.account_id != account.account_id {
                continue;
            }
            let policy = self
                .resolved_local_recording_retention_policy(
                    &candidate.manifest_path,
                    event.event.occurred_at,
                )
                .await?;
            let now = Utc::now();
            let audio_expired = request.remove_audio
                && policy
                    .audio_expires_at
                    .is_some_and(|expires| expires <= now);
            let speaker_expired = request.remove_speaker_hints
                && policy
                    .speaker_hints_expires_at
                    .is_some_and(|expires| expires <= now);
            if !audio_expired && !speaker_expired {
                continue;
            }

            let removed_at = Utc::now();
            let audio_removed = if audio_expired {
                remove_local_file_if_exists(&candidate.audio_path)?
            } else {
                false
            };
            let speaker_jsonl_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.speaker_jsonl_path)?
            } else {
                false
            };
            let speaker_txt_removed = if speaker_expired {
                remove_local_file_if_exists(&candidate.speaker_txt_path)?
            }
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/yandex_telemost/client/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/client/validation.rs`
- Size bytes / Размер в байтах: `4397`
- Included characters / Включено символов: `4397`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::{YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_PROVIDER_KIND_STR, YandexTelemostError};

pub(crate) fn validate_required(
    field: &'static str,
    value: &str,
) -> Result<String, YandexTelemostError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value.to_owned())
}

pub(crate) fn validate_json_object(
    field: &'static str,
    value: &Value,
) -> Result<(), YandexTelemostError> {
    if !value.is_object() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub(crate) fn validate_api_base_url(value: Option<&str>) -> Result<String, YandexTelemostError> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(YANDEX_TELEMOST_API_BASE_URL);
    if !(value.starts_with("https://")
        || value.starts_with("http://127.0.0.1")
        || value.starts_with("http://localhost"))
    {
        return Err(YandexTelemostError::InvalidRequest(
            "Yandex Telemost API base URL must be HTTPS, localhost, or 127.0.0.1".to_owned(),
        ));
    }
    Ok(value.trim_end_matches('/').to_owned())
}

pub fn validate_telemost_join_url(value: &str) -> Result<String, YandexTelemostError> {
    let value = validate_required("join_url", value)?;
    if !value.starts_with("https://") {
        return Err(YandexTelemostError::InvalidRequest(
            "Yandex Telemost join URL must be HTTPS".to_owned(),
        ));
    }
    let host = value
        .strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default();
    match host {
        "telemost.yandex.ru" | "telemost.yandex.com" => Ok(value),
        _ => Err(YandexTelemostError::InvalidRequest(format!(
            "unsupported Yandex Telemost join URL host `{host}`"
        ))),
    }
}

pub fn yandex_telemost_oauth_token_secret_ref(account_id: &str) -> String {
    let stable = account_id
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!("provider/{YANDEX_TELEMOST_PROVIDER_KIND_STR}/{stable}/oauth-token")
}

pub fn sanitize_yandex_telemost_payload(payload: Value) -> Value {
    match payload {
        Value::Object(mut object) => {
            for key in [
                "access_token",
                "authorization",
                "oauth_token",
                "token",
                "refresh_token",
                "cookie",
                "cookies",
                "password",
                "secret",
                "audio_bytes",
                "raw_audio",
                "mp3_bytes",
            ] {
                object.remove(key);
            }
            Value::Object(
                object
                    .into_iter()
                    .map(|(key, value)| (key, sanitize_yandex_telemost_payload(value)))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(sanitize_yandex_telemost_payload)
                .collect(),
        ),
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn join_url_rejects_non_telemost_hosts() {
        let error = validate_telemost_join_url("https://evil.example/room").unwrap_err();
        assert!(error.to_string().contains("unsupported Yandex Telemost"));
    }

    #[test]
    fn sanitizer_removes_secret_and_audio_material_recursively() {
        let sanitized = sanitize_yandex_telemost_payload(json!({
            "id": "c1",
            "oauth_token": "secret",
            "nested": { "mp3_bytes": "base64", "speaker": "Alice" }
        }));
        assert_eq!(sanitized["id"], "c1");
        assert!(sanitized.get("oauth_token").is_none());
        assert!(sanitized["nested"].get("mp3_bytes").is_none());
    }
}
```

### `backend/src/integrations/yandex_telemost/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/mod.rs`
- Size bytes / Размер в байтах: `57`
- Included characters / Включено символов: `57`
- Truncated / Обрезано: `no`

```rust
pub mod client;
pub mod runtime;
pub mod runtime_bridge;
```

### `backend/src/integrations/yandex_telemost/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/runtime.rs`
- Size bytes / Размер в байтах: `288`
- Included characters / Включено символов: `288`
- Truncated / Обрезано: `no`

```rust
pub use super::client::{
    YandexTelemostConferenceOpenRequest, YandexTelemostConferenceWebviewManifest,
    YandexTelemostLocalRecordingManifest, YandexTelemostLocalRecordingPolicy,
    YandexTelemostRuntimeStatus, YandexTelemostSpeakerTimelinePolicy, webview_manifest_for_request,
};
```

### `backend/src/integrations/yandex_telemost/runtime_bridge.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/yandex_telemost/runtime_bridge.rs`
- Size bytes / Размер в байтах: `14264`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::integrations::yandex_telemost::client::{
    YandexTelemostError, YandexTelemostTranscriptBridgeRequest,
    YandexTelemostTranscriptBridgeResponse,
};
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};
use crate::platform::realtime_conversation::{
    CallBundleArtifact, CallBundleManifest, REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED,
};

pub(crate) struct MaterializedTelemostTranscriptBundle {
    pub(crate) bundle_root: PathBuf,
    pub(crate) manifest_path: PathBuf,
    pub(crate) manifest: CallBundleManifest,
    pub(crate) transcript_json_path: PathBuf,
    pub(crate) transcript_markdown_path: PathBuf,
    pub(crate) summary_markdown_path: Option<PathBuf>,
}

pub(crate) async fn complete_yandex_telemost_transcript_bridge(
    event_store: &EventStore,
    event_bus: Option<&EventBus>,
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<YandexTelemostTranscriptBridgeResponse, YandexTelemostError> {
    let materialized = materialize_yandex_telemost_transcript_artifacts(request)?;
    publish_realtime_conversation_transcript_completed_event(
        event_store,
        event_bus,
        request,
        &materialized,
    )
    .await?;
    Ok(YandexTelemostTranscriptBridgeResponse {
        account_id: request.account_id.clone(),
        conference_id: request.conference_id.clone(),
        bundle_id: materialized.manifest.bundle_id.clone(),
        manifest_path: materialized.manifest_path.to_string_lossy().into_owned(),
        transcript_json_path: materialized
            .transcript_json_path
            .to_string_lossy()
            .into_owned(),
        transcript_markdown_path: materialized
            .transcript_markdown_path
            .to_string_lossy()
            .into_owned(),
        summary_markdown_path: materialized
            .summary_markdown_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        follow_up_events: vec![REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED.to_owned()],
    })
}

pub(crate) fn materialize_yandex_telemost_transcript_artifacts(
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<MaterializedTelemostTranscriptBundle, YandexTelemostError> {
    validate_yandex_telemost_transcript_bridge_request(request)?;
    let bundle_root = canonical_existing_dir("bundle_root", &request.bundle_root)?;
    let manifest_path = canonical_existing_file(
        "manifest_path",
        &bundle_root.join("manifest.json").to_string_lossy(),
        &bundle_root,
    )?;
    let mut manifest: CallBundleManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    if manifest.bundle_id.trim() != request.bundle_id.trim() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "bundle_id `{}` does not match manifest bundle_id `{}`",
            request.bundle_id, manifest.bundle_id
        )));
    }
    if manifest.account_id.trim() != request.account_id.trim() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "account_id `{}` does not match manifest account_id `{}`",
            request.account_id, manifest.account_id
        )));
    }

    let transcript_json_path = bundle_root.join(&manifest.layout.transcript_json);
    let transcript_markdown_path = bundle_root.join(&manifest.layout.transcript_markdown);
    let summary_markdown_path = request
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|_| bundle_root.join(&manifest.layout.summary_markdown));
    let transcript_json_relative_path = manifest.layout.transcript_json.clone();
    let transcript_markdown_relative_path = manifest.layout.transcript_markdown.clone();
    let summary_markdown_relative_path = manifest.layout.summary_markdown.clone();
    let transcript_json = json!({
        "bundle_id": manifest.bundle_id,
        "provider_kind": manifest.provider_kind.as_str(),
        "conference_id": request.conference_id,
        "language_code": request.language_code,
        "stt_provider": request.stt_provider,
        "confidence": normalized_transcript_confidence(request),
        "summary": request.summary,
        "segments": request.segments,
        "metadata": request.metadata,
        "transcript_text": request.transcript_text,
    });
    fs::write(
        &transcript_json_path,
        serde_json::to_string_pretty(&transcript_json)?,
    )?;
    fs::write(
        &transcript_markdown_path,
        render_transcript_markdown(request),
    )?;
    if let Some(path) = &summary_markdown_path {
        fs::write(path, render_summary_markdown(request))?;
    }

    upsert_bundle_artifact(
        &mut manifest,
        CallBundleArtifact {
            kind: "transcript".to_owned(),
            relative_path: transcript_json_relative_path,
            source: request.stt_provider.trim().to_owned(),
            truth_status: "model_output".to_owned(),
            media_type: Some("application/json".to_owned()),
            description: Some("Structured transcript with evidence metadata".to_owned()),
        },
    );
    upsert_bundle_artifact(
        &mut manifest,
        CallBundleArtifact {
            kind: "transcript_markdown".to_owned(),
            relative_path: transcript_markdown_relative_path,
            source: request.stt_provider.trim().to_owned(),
            truth_status: "model_output".to_owned(),
            media_type: Some("text/markdown".to_owned()),
            description: Some("Owner-readable transcript markdown projection".to_owned()),
        },
    );
    if summary_markdown_path.is_some() {
        upsert_bundle_artifact(
            &mut manifest,
            CallBundleArtifact {
                kind: "summary_markdown".to_owned(),
                relative_path: summary_markdown_relative_path,
                source: request.stt_provider.trim().to_owned(),
                truth_status: "model_output".to_owned(),
                media_type: Some("text/markdown".to_owned()),
                description: Some("Owner-readable transcript summary".to_owned()),
            },
        );
    }
    manifest.pipeline_state.transcription = "completed".to_owned();
    manifest.pipeline_state.diarization =
        if transcript_segments_have_speaker_labels(&request.segments) {
            "completed_with_speaker_segments".to_owned()
        } else {
            "completed_without_speaker_labels".to_owned()
        };
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(MaterializedTelemostTranscriptBundle {
        bundle_root,
        manifest_path,
        manifest,
        transcript_json_path,
        transcript_markdown_path,
        summary_markdown_path,
    })
}

async fn publish_realtime_conversation_transcript_completed_event(
    event_store: &EventStore,
    event_bus: Option<&EventBus>,
    request: &YandexTelemostTranscriptBridgeRequest,
    materialized: &MaterializedTelemostTranscriptBundle,
) -> Result<(), YandexTelemostError> {
    let event = NewEventEnvelope::builder(
        format!(
            "realtime-conversation-transcript-completed-{}-{}",
            materialized.manifest.bundle_id,
            Uuid::new_v4()
        ),
        REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED,
        Utc::now(),
        json!({ "source_code": "workflow.realtime_conversation", "provider": "yandex_telemost" }),
        json!({ "kind": "call_bundle", "entity_id": materialized.manifest.bundle_id }),
    )
    .payload(json!({
        "bundle_id": materialized.manifest.bundle_id,
        "account_id": request.account_id,
        "conference_id": request.conference_id,
        "calendar_event_id": materialized.manifest.calendar_event_id,
        "manifest_path": materialized.manifest_path.to_string_lossy(),
        "bundle_root": materialized.bundle_root.to_string_lossy(),
        "transcript_json_path": materialized.transcript_json_path.to_string_lossy(),
        "transcript_markdown_path": materialized.transcript_markdown_path.to_string_lossy(),
        "summary_markdown_path": materialized.summary_markdown_path.as_ref().map(|path| path.to_string_lossy().into_owned()),
        "language_code": request.language_code,
        "stt_provider": request.stt_provider,
        "confidence": normalized_transcript_confidence(request),
        "summary": request.summary,
        "transcript_text": request.transcript_text,
        "segment_count": request.segments.as_array().map(|value| value.len()).unwrap_or(0),
        "metadata": request.metadata,
    }))
    .provenance(json!({ "origin": "telemost_transcript_runtime_bridge" }))
    .correlation_id(format!(
        "realtime-conversation:{}",
        materialized.manifest.bundle_id
    ))
    .build()?;
    if event_store
        .append_for_dispatch_idempotent(&event)
        .await?
        .is_some()
        && let Some(event_bus) = event_bus
    {
        event_bus.broadcast(event);
    }
    Ok(())
}

fn validate_yandex_telemost_transcript_bridge_request(
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<(), YandexTelemostError> {
    crate::integrations::yandex_telemost::client::validate_required(
        "account_id",
        &request.account_id,
    )?;
    crate::integrations::yandex_telemost::client::validate_required(
        "bundle_id",
        &request.bundle_id,
    )?;
    crate::integrations::yandex_telemost::client::validate_required(
        "transcript_text",
        &request.transcript_text,
    )?;
    crate::integrations::yandex_telemost::client::validate_required(
        "stt_provider",
        &request.stt_provider,
    )?;
    if !request.segments.is_array() {
        return Err(YandexTelemostError::InvalidRequest(
            "segments must be a JSON array".to_owned(),
        ));
    }
    if !request.metadata.is_object() {
        return Err(YandexTelemostError::InvalidRequest(
            "metadata must be a JSON object".to_owned(),
        ));
    }
    if let Some(confidence) = request.confidence
        && !(0.0..=1.0).contains(&confidence)
    {
        return Err(YandexTelemostError::InvalidRequest(
            "confidence must be between 0.0 and 1.0".to_owned(),
        ));
    }
    Ok(())
}

fn upsert_bundle_artifact(manifest: &mut CallBundleManifest, artifact: CallBundleArtifact) {
    if let Some(existing) = manifest
        .artifacts
        .iter_mut()
        .find(|item| item.kind == artifact.kind)
    {
        *existing = artifact;
        return;
    }
    manifest.artifacts.push(artifact);
}

fn render_transcript_markdown(request: &YandexTelemostTranscriptBridgeRequest) -> String {
    let mut lines = vec![
        "# Transcript".to_owned(),
        String::new(),
        format!("- Bundle: `{}`", request.bundle_id),
        format!("- Account: `{}`", request.account_id),
        format!("- STT provider: `{}`", request.stt_provider.trim()),
    ];
    if let Some(conference_id) = request
        .conference_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        lines.push(format!("- Conference: `{conference_id}`"));
    }
    if let Some(language_code) = request
        .language_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        lines.push(format!("- Language: `{language_code}`"));
    }
    lines.push(String::new());
    lines.push("## Full text".to_owned());
    lines.push(String::new());
    lines.push(request.transcript_text.trim().to_owned());
    lines.join("\n")
}

fn render_summary_markdown(request: &YandexTelemostTranscriptBridgeRequest) -> String {
    let summary = request
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    format!("# Summary\n\n{summary}\n")
}

fn transcript_segments
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/zoom/client.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/zoom/client.rs`
- Size bytes / Размер в байтах: `124`
- Included characters / Включено символов: `124`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod store;
mod validation;

pub use errors::ZoomError;
pub use models::*;
pub use store::ZoomStore;
```

### `backend/src/integrations/zoom/client/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/zoom/client/errors.rs`
- Size bytes / Размер в байтах: `1531`
- Included characters / Включено символов: `1531`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::calls::CallError;
use crate::platform::communications::{ProviderAccountPortError, ProviderSecretBindingPortError};
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::secrets::{SecretReferenceError, SecretResolutionError};
use crate::platform::settings::SettingsError;
use crate::platform::storage::StorageError;
use crate::vault::HostVaultError;

#[derive(Debug, Error)]
pub enum ZoomError {
    #[error("invalid Zoom request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    ProviderAccountStore(#[from] ProviderAccountPortError),

    #[error(transparent)]
    ProviderSecretBindingStore(#[from] ProviderSecretBindingPortError),

    #[error(transparent)]
    Call(#[from] CallError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Settings(#[from] SettingsError),
}
```
