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

- Chunk ID / ID чанка: `068-source-backend-part-048`
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

### `backend/src/integrations/zoom/client/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/zoom/client/models.rs`
- Size bytes / Размер в байтах: `55426`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, TimeDelta, Utc};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::form_urlencoded;

use crate::platform::calls::{CallDirection, CallState, NewProviderCall};
use crate::platform::communications::{CommunicationProviderKind, ProviderAccount};

use super::ZoomError;
use super::validation::{validate_array, validate_non_empty, validate_object};

pub const ZOOM_PROVIDER_KIND: CommunicationProviderKind = CommunicationProviderKind::ZoomUser;
pub const ZOOM_PROVIDER_KIND_STR: &str = "zoom_user";
pub const ZOOM_RUNTIME_KIND: &str = "zoom_fixture_runtime";
pub const ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND: &str = "zoom_live_authorized_runtime";
pub const DEFAULT_ZOOM_AUTHORIZATION_ENDPOINT: &str = "https://zoom.us/oauth/authorize";
pub const DEFAULT_ZOOM_TOKEN_ENDPOINT: &str = "https://zoom.us/oauth/token";
pub const DEFAULT_ZOOM_API_BASE_URL: &str = "https://api.zoom.us/v2";
pub const ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS: i64 = 60;
pub const ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS: i64 = 60;
pub const ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS: i64 = 300;
pub const ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS: i64 = 86_400;
pub const ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER: &str = "zoom_token_rotation_required";
pub const ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE: usize = 30;
pub const ZOOM_PROVIDER_SYNC_MAX_PAGE_SIZE: usize = 100;
pub const ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS: usize = 100;
pub const ZOOM_PROVIDER_SYNC_MAX_MEETINGS: usize = 500;
pub const ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES: usize = 268_435_456;
pub const ZOOM_DEFAULT_WEBHOOK_SUBSCRIPTION_NAME: &str = "Макошь Zoom Runtime";
pub const ZOOM_DEFAULT_WEBHOOK_EVENT_TYPES: &[&str] = &[
    "meeting.started",
    "meeting.ended",
    "meeting.participant_joined",
    "meeting.participant_left",
    "recording.completed",
];

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoomAuthShape {
    Fixture,
    #[serde(rename = "oauth_user")]
    #[default]
    OAuthUser,
    ServerToServer,
}

impl ZoomAuthShape {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::OAuthUser => "oauth_user",
            Self::ServerToServer => "server_to_server",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomAccountSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub account_email: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

impl ZoomAccountSetupRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub fn account_config(&self) -> Value {
        json!({
            "provider": "zoom",
            "provider_kind": ZOOM_PROVIDER_KIND_STR,
            "runtime_kind": ZOOM_RUNTIME_KIND,
            "auth_shape": ZoomAuthShape::Fixture.as_str(),
            "lifecycle_state": "fixture_ready",
            "account_email": trimmed_optional(&self.account_email),
            "metadata": &self.metadata,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomLiveAccountSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub account_email: Option<String>,
    #[serde(default)]
    pub auth_shape: ZoomAuthShape,
    pub client_id: String,
    pub token_secret_ref: Option<String>,
    pub client_secret_ref: Option<String>,
    pub webhook_secret_ref: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

impl ZoomLiveAccountSetupRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("client_id", &self.client_id)?;
        if self.auth_shape == ZoomAuthShape::Fixture {
            return Err(ZoomError::InvalidRequest(
                "auth_shape must be oauth_user or server_to_server for live account metadata"
                    .to_owned(),
            ));
        }
        validate_optional_ref("token_secret_ref", &self.token_secret_ref)?;
        validate_optional_ref("client_secret_ref", &self.client_secret_ref)?;
        validate_optional_ref("webhook_secret_ref", &self.webhook_secret_ref)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub fn provider_kind(&self) -> CommunicationProviderKind {
        match self.auth_shape {
            ZoomAuthShape::ServerToServer => CommunicationProviderKind::ZoomServerToServer,
            ZoomAuthShape::Fixture | ZoomAuthShape::OAuthUser => {
                CommunicationProviderKind::ZoomUser
            }
        }
    }

    pub fn account_config(&self) -> Value {
        let provider_kind = self.provider_kind();
        json!({
            "provider": "zoom",
            "provider_kind": provider_kind.as_str(),
            "runtime_kind": "zoom_live_blocked_runtime",
            "auth_shape": self.auth_shape.as_str(),
            "lifecycle_state": "blocked",
            "account_email": trimmed_optional(&self.account_email),
            "client_id": self.client_id.trim(),
            "credential_refs_bound": {
                "zoom_oauth_token": has_optional_ref(&self.token_secret_ref),
                "zoom_client_secret": has_optional_ref(&self.client_secret_ref),
                "zoom_webhook_secret": has_optional_ref(&self.webhook_secret_ref),
            },
            "runtime_blockers": ["zoom_live_authorization_required"],
            "metadata": &self.metadata,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAccount {
    pub account_id: String,
    pub provider_kind: String,
    pub display_name: String,
    pub external_account_id: String,
    pub auth_shape: String,
    pub lifecycle_state: String,
    pub runtime_kind: String,
    pub account_email: Option<String>,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProviderAccount> for ZoomAccount {
    fn from(account: ProviderAccount) -> Self {
        let auth_shape = account
            .config
            .get("auth_shape")
            .and_then(Value::as_str)
            .unwrap_or("fixture")
            .to_owned();
        let lifecycle_state = account
            .config
            .get("lifecycle_state")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let runtime_kind = account
            .config
            .get("runtime_kind")
            .and_then(Value::as_str)
            .unwrap_or(ZOOM_RUNTIME_KIND)
            .to_owned();
        let account_email = account
            .config
            .get("account_email")
            .and_then(Value::as_str)
            .map(str::to_owned);

        Self {
            account_id: account.account_id,
            provider_kind: account.provider_kind.as_str().to_owned(),
            display_name: account.display_name,
            external_account_id: account.external_account_id,
            auth_shape,
            lifecycle_state,
            runtime_kind,
            account_email,
            config: account.config,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAccountSetupResponse {
    pub account: ZoomAccount,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomAccountListResponse {
    pub items: Vec<ZoomAccount>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub status: String,
    pub healthy: bool,
    pub auth_shape: String,
    pub live_runtime_available: bool,
    pub recording_ingest_available: bool,
    pub transcript_ingest_available: bool,
    pub runtime_blockers: Vec<String>,
    pub last_error: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRuntimeStartRequest {
    pub account_id: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRuntimeStopRequest {
    pub account_id: String,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomRuntimeRemoveRequest {
    pub account_id: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomRuntimeRemoveResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub removed: bool,
    pub removed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZoomParticipantSnapshot {
    pub participant_id: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub joined_at: Option<DateTime<Utc>>,
    pub left_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZoomRecordingRef {
    pub recording_id: String,
    pub recording_type: Option<String>,
    pub download_ref: Option<String>,
    pub file_extension: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub recorded_at: Option<DateTime<Utc>>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomMeetingObservationRequest {
    pub observation_id: Option<String>,
    pub account_id: String,
    pub meeting_id: String,
    pub meeting_uuid: Option<String>,
    pub topic: Option<String>,
    pub host_email: Option<String>,
    pub join_url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    #[serde(default)]
    pub participants: Vec<ZoomParticipantSnapshot>,
    #[serde(default)]
    pub recording_refs: Vec<ZoomRecordingRef>,
    pub transcript_ref: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl ZoomMeetingObservationRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("meeting_id", &self.meeting_id)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub fn provider_chat_id(&self) -> String {
        format!("zoom:meeting:{}", self.meeting_id.trim())
    }

    pub fn event_subject_id(&self) -> String {
        self.meeting_uuid
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.meeting_id.trim())
            .to_owned()
    }

    pub fn into_call(&self, call_id: String, observed_at: DateTime<Utc>) -> NewProviderCall {
        let participants = sanitize_zoom_payload(json!(&self.participants));
        let recording_refs = sanitize_zoom_payload(json!(&self.recording_refs));
        let metadata = sanitize_zoom_payload(self.metadata.clone());
        NewProviderCall {
            call_id,
            account_id: self.account_id.trim().to_owned(),
            provider_call_id: self.meeting_id.trim().to_owned(),
            provider_chat_id: self.provider_chat_id(),
            direction: Call
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/zoom/client/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/zoom/client/store.rs`
- Size bytes / Размер в байтах: `148312`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::PgPool;
use url::form_urlencoded::byte_serialize;

use crate::platform::calls::{
    CallIntelligenceStore, NewCallTranscript, NewProviderCall, TranscriptStatus,
};
use crate::platform::communications::{
    DEFAULT_MAIL_SYNC_BLOB_ROOT, NewProviderAccountSecretBinding, ProviderAccountCommandPort,
    ProviderAccountSecretPurpose, ProviderSecretBindingCommandPort,
};
use crate::platform::events::bus::zoom_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use crate::platform::settings::ApplicationSettingsStore;
use crate::platform::storage::{
    ImportedAttachmentRecord, ImportedAttachmentStoragePort, ImportedAttachmentUpsert,
    SafetyScanRequest, delete_local_blob, new_attachment_import_id, put_local_blob,
    scan_attachment,
};
use crate::vault::{HostVault, SecretEntryContext};

use super::{
    MAX_TRANSCRIPT_FILE_TEXT_BYTES, ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS,
    ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND, ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES,
    ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS, ZOOM_PROVIDER_KIND, ZOOM_PROVIDER_KIND_STR,
    ZOOM_RUNTIME_KIND, ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS,
    ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS, ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER,
    ZoomAccount, ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventItem, ZoomAuditEventResponse, ZoomAuthShape, ZoomAuthorizationResult, ZoomError,
    ZoomLiveAccountSetupRequest, ZoomMeetingIngestResult, ZoomMeetingObservationRequest,
    ZoomOAuthPendingGrant, ZoomOAuthStartRequest, ZoomOAuthTokenBundle, ZoomOAuthTokenResponse,
    ZoomRecordingImportAuditItem, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingRef, ZoomRecordingSyncFailure,
    ZoomRecordingSyncRequest, ZoomRecordingSyncResult, ZoomRetentionCleanupItem,
    ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest,
    ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomServerToServerAuthorizeRequest, ZoomTokenMaintenanceItem, ZoomTokenMaintenanceRequest,
    ZoomTokenMaintenanceResult, ZoomTokenRefreshRequest, ZoomTokenRefreshResult,
    ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult,
    ZoomTranscriptObservationRequest, ZoomWebhookSubscription,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
    random_zoom_oauth_token, sanitize_zoom_payload, zoom_authorization_url, zoom_client_secret_ref,
    zoom_oauth_expires_at, zoom_oauth_token_secret_ref,
};

#[derive(Clone)]
pub struct ZoomStore {
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    imported_attachment_store: Arc<dyn ImportedAttachmentStoragePort>,
    call_store: CallIntelligenceStore,
    event_store: EventStore,
    event_bus: EventBus,
    http: reqwest::Client,
}

struct ZoomAuthorizedAccountUpdate<'a> {
    auth_shape: &'a str,
    token_secret_ref: &'a str,
    client_secret_ref: Option<&'a str>,
    expires_at: Option<DateTime<Utc>>,
    metadata: Value,
    authorized_at: DateTime<Utc>,
}

const ZOOM_RECORDING_IMPORT_RETENTION_DAYS_SETTING_KEY: &str =
    "privacy.zoom_recording_import_retention_days";
const ZOOM_TRANSCRIPT_RETENTION_DAYS_SETTING_KEY: &str = "privacy.zoom_transcript_retention_days";

impl ZoomStore {
    pub fn new(
        pool: PgPool,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        imported_attachment_store: Arc<dyn ImportedAttachmentStoragePort>,
        call_store: CallIntelligenceStore,
        event_store: EventStore,
        event_bus: EventBus,
    ) -> Self {
        Self {
            pool,
            provider_account_store,
            provider_secret_binding_store,
            imported_attachment_store,
            call_store,
            event_store,
            event_bus,
            http: zoom_http_client(),
        }
    }

    pub async fn setup_fixture_account(
        &self,
        request: &ZoomAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        request.validate()?;
        let account = self
            .provider_account_store
            .upsert_runtime_account(
                request.account_id.trim().to_owned(),
                ZOOM_PROVIDER_KIND.as_str().to_owned(),
                request.display_name.trim().to_owned(),
                request.external_account_id.trim().to_owned(),
                request.account_config(),
            )
            .await?;
        Ok(ZoomAccountSetupResponse {
            account: account.into(),
        })
    }

    pub async fn setup_live_blocked_account(
        &self,
        request: &ZoomLiveAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        request.validate()?;
        let provider_kind = request.provider_kind();
        let account = self
            .provider_account_store
            .upsert_runtime_account(
                request.account_id.trim().to_owned(),
                provider_kind.as_str().to_owned(),
                request.display_name.trim().to_owned(),
                request.external_account_id.trim().to_owned(),
                request.account_config(),
            )
            .await?;
        self.bind_live_secret_refs(request).await?;
        Ok(ZoomAccountSetupResponse {
            account: account.into(),
        })
    }

    pub async fn start_oauth(
        &self,
        request: &ZoomOAuthStartRequest,
    ) -> Result<ZoomOAuthPendingGrant, ZoomError> {
        request.validate()?;
        self.setup_live_blocked_account(&request.live_account_request())
            .await?;
        let setup_id = random_zoom_oauth_token()?;
        let state = random_zoom_oauth_token()?;
        let authorization_url = zoom_authorization_url(request, &state)?;
        Ok(ZoomOAuthPendingGrant {
            setup_id,
            account_id: request.account_id.trim().to_owned(),
            authorization_url,
            state,
            request: request.clone(),
        })
    }

    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<ZoomAccountListResponse, ZoomError> {
        let mut accounts = self
            .provider_account_store
            .list()
            .await?
            .into_iter()
            .filter(|account| account.provider_kind.is_zoom())
            .map(ZoomAccount::from)
            .filter(|account| include_removed || account.lifecycle_state != "removed")
            .collect::<Vec<_>>();
        accounts.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(ZoomAccountListResponse { items: accounts })
    }

    pub async fn runtime_status(&self, account_id: &str) -> Result<ZoomRuntimeStatus, ZoomError> {
        let account = self.zoom_account(account_id).await?;
        Ok(runtime_status_from_account(account))
    }

    pub async fn start_runtime(
        &self,
        request: &ZoomRuntimeStartRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        let account_id = validate_account_id(&request.account_id)?;
        let account = self.zoom_account(&account_id).await?;
        let mut config = account.config.clone();
        let live_blocked = account.auth_shape != "fixture";
        let live_authorized = live_blocked && zoom_account_is_authorized(&account);
        config["lifecycle_state"] = json!(if live_authorized {
            "running"
        } else if live_blocked {
            "blocked"
        } else {
            "running"
        });
        config["runtime_kind"] = json!(if live_authorized {
            ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND
        } else if live_blocked {
            "zoom_live_blocked_runtime"
        } else {
            ZOOM_RUNTIME_KIND
        });
        config["runtime_blockers"] = if live_authorized {
            let mut blockers = account
                .config
                .get("runtime_blockers")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            blockers.retain(|value| value.as_str() != Some("zoom_provider_workers_not_enabled"));
            json!(blockers)
        } else if live_blocked {
            json!(["zoom_live_authorization_required"])
        } else {
            json!([])
        };
        config["last_runtime_action"] = json!({
            "action": "start",
            "force": request.force,
            "observed_at": Utc::now(),
        });
        let updated = self.update_account_config(&account_id, config).await?;
        let status = runtime_status_from_account(updated);
        self.publish_runtime_status_event(&status, "zoom.runtime.start_requested")
            .await?;
        Ok(status)
    }

    pub async fn stop_runtime(
        &self,
        request: &ZoomRuntimeStopRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        let account_id = validate_account_id(&request.account_id)?;
        let account = self.zoom_account(&account_id).await?;
        let mut config = account.config.clone();
        if account.lifecycle_state != "removed" {
            config["lifecycle_state"] = json!("stopped");
        }
        config["last_runtime_action"] = json!({
            "action": "stop",
            "reason": &request.reason,
            "observed_at": Utc::now(),
        });
        let updated = self.update_account_config(&account_id, config).await?;
        let status = runtime_status_from_account(updated);
        self.publish_runtime_status_event(&status, "zoom.runtime.stop_requested")
            .await?;
        Ok(status)
    }

    pub async fn remove_runtime(
        &self,
        request: &ZoomRuntimeRemoveRequest,
    ) -> Result<ZoomRuntimeRemoveResponse, ZoomError> {
        let account_id = validate_account_id(&request.account_id)?;
        let account = self.zoom_account(&account_id).await?;
        let removed_at = Utc::now();
        let mut config = account.config.clone();
        config["lifecycle_state"] = json!("removed");
        config["removed_at"] = json!(removed_at);
        config["remove_reason"] = json!(&request.reason);
        let updated = self.update_account_config(&account_id, config).await?;
        let status = runtime_status_from_account(updated);
        self.publish_runtime_status_event(&status, "zoom.runtime.remove_requested")
            .await?;
        Ok(ZoomRuntimeRemoveResponse {
            account_id,
            provider_kind: status.provider_kind,
            removed: true,
            removed_at,
        })
    }

    pub async fn observe_meeting(
        &self,
        request: &ZoomMeetingObservationRequest,
    ) -> Result<ZoomMeetingIngestResult, ZoomError> {
        request.validate()?;
        self.ensure_zoom_account(&request.account_id).await?;
        let observed_at = request.started_at.unwrap_or_else(Utc::now);
        let call_id = stable_zoom_call_id(&request.account_id, &request.meeting_id);
        let call: NewProviderCall = request.into_call(call_id.clone(), observed_at);
        self.call_store.upsert_call(&call).await?;

        let eve
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/zoom/client/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/zoom/client/validation.rs`
- Size bytes / Размер в байтах: `848`
- Included characters / Включено символов: `848`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use super::ZoomError;

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<String, ZoomError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn validate_object(field: &'static str, value: &Value) -> Result<(), ZoomError> {
    if !value.is_object() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub(super) fn validate_array(field: &'static str, value: &Value) -> Result<(), ZoomError> {
    if !value.is_array() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(())
}
```

### `backend/src/integrations/zoom/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/zoom/mod.rs`
- Size bytes / Размер в байтах: `33`
- Included characters / Включено символов: `33`
- Truncated / Обрезано: `no`

```rust
pub mod client;
pub mod runtime;
```

### `backend/src/integrations/zoom/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/zoom/runtime.rs`
- Size bytes / Размер в байтах: `157`
- Included characters / Включено символов: `157`
- Truncated / Обрезано: `no`

```rust
pub use super::client::{
    ZoomRuntimeRemoveRequest, ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest,
    ZoomRuntimeStatus, ZoomRuntimeStopRequest,
};
```

### `backend/src/lib.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/lib.rs`
- Size bytes / Размер в байтах: `294`
- Included characters / Включено символов: `294`
- Truncated / Обрезано: `no`

```rust
#![allow(dead_code, unused_imports, unused_variables)]
pub mod ai;
pub mod app;
pub mod application;
pub mod contracts;
pub mod domains;
pub mod engines;
pub mod integrations;
pub mod platform;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod vault;
pub mod workflows;
```

### `backend/src/main.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/main.rs`
- Size bytes / Размер в байтах: `581`
- Included characters / Включено символов: `581`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::platform::config::AppConfig;
use tracing::Instrument;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    makosh_hub_backend::app::init_tracing();
    let flow_id = std::env::var("MAKOSH_FLOW_ID").unwrap_or_else(|_| "unknown".to_owned());
    let runtime_span = tracing::info_span!("makosh_runtime", flow_id = %flow_id);

    async move {
        let config = AppConfig::from_env()?;
        makosh_hub_backend::app::run(config).await?;
        Ok(())
    }
    .instrument(runtime_span)
    .await
}
```

### `backend/src/platform/ai_runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/ai_runtime.rs`
- Size bytes / Размер в байтах: `1294`
- Included characters / Включено символов: `1294`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct AiChatResult {
    pub model: String,
    pub content: String,
    pub total_duration_ns: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
    pub total_duration_ns: Option<u64>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{runtime} AI runtime request failed: {message}")]
pub struct AiRuntimePortError {
    pub runtime: String,
    pub message: String,
}

impl AiRuntimePortError {
    pub fn provider(runtime: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            runtime: runtime.into(),
            message: message.into(),
        }
    }
}

pub type SharedAiRuntimePort = Arc<dyn AiRuntimePort>;

pub trait AiRuntimePort: Send + Sync {
    fn runtime_name(&self) -> &'static str;

    fn chat<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiChatResult, AiRuntimePortError>> + Send + 'a>>;

    fn embed_with_model<'a>(
        &'a self,
        input: &'a str,
        model: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiEmbedResult, AiRuntimePortError>> + Send + 'a>>;
}
```

### `backend/src/platform/audit.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit.rs`
- Size bytes / Размер в байтах: `312`
- Included characters / Включено символов: `312`
- Truncated / Обрезано: `no`

```rust
mod communication;
mod constants;
mod documents;
mod errors;
mod events;
mod helpers;
mod models;
mod reviews;
mod settings;
mod store;
mod telegram;
mod telegram_dialogs;
mod telegram_participants;

pub use errors::ApiAuditError;
pub use models::{ApiAuditRecord, NewApiAuditRecord};
pub use store::ApiAuditLog;
```

### `backend/src/platform/audit/communication.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/communication.rs`
- Size bytes / Размер в байтах: `668`
- Included characters / Включено символов: `668`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn communication_email_send(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        recipient_count: usize,
    ) -> Self {
        Self::new(
            actor_id,
            "communication.email.send",
            "POST",
            "/api/v1/communications/send",
            "communication_provider_account",
            Some(account_id.into()),
            json!({
                "action_class": "provider_write",
                "transport": "smtp",
                "recipient_count": recipient_count,
            }),
        )
    }
}
```

### `backend/src/platform/audit/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/constants.rs`
- Size bytes / Размер в байтах: `113`
- Included characters / Включено символов: `113`
- Truncated / Обрезано: `no`

```rust
pub(super) const API_FRONTEND_ACTOR_KIND: &str = "frontend";
pub(super) const EVENT_TARGET_KIND: &str = "event";
```

### `backend/src/platform/audit/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/documents.rs`
- Size bytes / Размер в байтах: `499`
- Included characters / Включено символов: `499`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn document_processing_job_retry(
        actor_id: impl Into<String>,
        job_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "document_processing.job.retry",
            "POST",
            "/api/v1/document-processing/jobs/{job_id}/retry",
            "document_processing_job",
            Some(job_id.into()),
            json!({}),
        )
    }
}
```

### `backend/src/platform/audit/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/errors.rs`
- Size bytes / Размер в байтах: `131`
- Included characters / Включено символов: `131`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiAuditError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/platform/audit/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/events.rs`
- Size bytes / Размер в байтах: `1265`
- Included characters / Включено символов: `1265`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::constants::EVENT_TARGET_KIND;
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn event_append(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self::new(
            actor_id,
            "event.append",
            "POST",
            "/api/v1/events",
            EVENT_TARGET_KIND,
            Some(event_id.into()),
            json!({}),
        )
    }

    pub fn event_get(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self::new(
            actor_id,
            "event.get",
            "GET",
            "/api/v1/events/{event_id}",
            EVENT_TARGET_KIND,
            Some(event_id.into()),
            json!({}),
        )
    }

    pub fn event_list(
        actor_id: impl Into<String>,
        after_position: i64,
        limit: u32,
        wait_seconds: u64,
    ) -> Self {
        Self::new(
            actor_id,
            "event.list",
            "GET",
            "/api/v1/events",
            EVENT_TARGET_KIND,
            None,
            json!({
                "after_position": after_position,
                "limit": limit,
                "wait_seconds": wait_seconds,
            }),
        )
    }
}
```

### `backend/src/platform/audit/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/helpers.rs`
- Size bytes / Размер в байтах: `697`
- Included characters / Включено символов: `697`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

pub(super) fn insert_non_empty(
    metadata: &mut serde_json::Map<String, Value>,
    key: &'static str,
    value: String,
) {
    let value = value.trim();
    if !value.is_empty() {
        metadata.insert(key.to_owned(), json!(value));
    }
}

pub(super) fn insert_optional(
    metadata: &mut serde_json::Map<String, Value>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        insert_non_empty(metadata, key, value);
    }
}

pub(super) fn non_empty_optional(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
```

### `backend/src/platform/audit/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/models.rs`
- Size bytes / Размер в байтах: `1513`
- Included characters / Включено символов: `1513`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::constants::API_FRONTEND_ACTOR_KIND;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApiAuditRecord {
    pub audit_id: i64,
    pub recorded_at: DateTime<Utc>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub operation: String,
    pub method: String,
    pub path_template: String,
    pub target_kind: String,
    pub target_id: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewApiAuditRecord {
    pub(super) actor_kind: String,
    pub(super) actor_id: String,
    pub(super) operation: String,
    pub(super) method: String,
    pub(super) path_template: String,
    pub(super) target_kind: String,
    pub(super) target_id: Option<String>,
    pub(super) metadata: Value,
}

impl NewApiAuditRecord {
    pub(super) fn new(
        actor_id: impl Into<String>,
        operation: impl Into<String>,
        method: impl Into<String>,
        path_template: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: Option<String>,
        metadata: Value,
    ) -> Self {
        Self {
            actor_kind: API_FRONTEND_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: operation.into(),
            method: method.into(),
            path_template: path_template.into(),
            target_kind: target_kind.into(),
            target_id,
            metadata,
        }
    }
}
```

### `backend/src/platform/audit/reviews.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/reviews.rs`
- Size bytes / Размер в байтах: `3734`
- Included characters / Включено символов: `3734`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn project_link_review_set(
        actor_id: impl Into<String>,
        project_id: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        let project_id = project_id.into();
        let target_kind = target_kind.into();
        let target_id = target_id.into();

        Self::new(
            actor_id,
            "project.link_review.set",
            "PUT",
            "/api/v1/projects/{project_id}/link-reviews",
            "project_link",
            Some(format!("{project_id}:{target_kind}:{target_id}")),
            json!({
                "project_id": project_id,
                "target_kind": target_kind,
                "target_id": target_id,
            }),
        )
    }

    pub fn task_candidate_review_set(
        actor_id: impl Into<String>,
        task_candidate_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "task_candidate.review.set",
            "PUT",
            "/api/v1/task-candidates/{task_candidate_id}/review",
            "task_candidate",
            Some(task_candidate_id.into()),
            json!({}),
        )
    }

    pub fn obligation_review_set(
        actor_id: impl Into<String>,
        obligation_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "obligation.review.set",
            "PUT",
            "/api/v1/obligations/{obligation_id}/review",
            "obligation",
            Some(obligation_id.into()),
            json!({}),
        )
    }

    pub fn decision_review_set(
        actor_id: impl Into<String>,
        decision_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "decision.review.set",
            "PUT",
            "/api/v1/decisions/{decision_id}/review",
            "decision",
            Some(decision_id.into()),
            json!({}),
        )
    }

    pub fn relationship_review_set(
        actor_id: impl Into<String>,
        relationship_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "relationship.review.set",
            "PUT",
            "/api/v1/relationships/{relationship_id}/review",
            "relationship",
            Some(relationship_id.into()),
            json!({}),
        )
    }

    pub fn contradiction_review_set(
        actor_id: impl Into<String>,
        observation_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "contradiction.review.set",
            "PUT",
            "/api/v1/contradictions/{observation_id}/review",
            "contradiction_observation",
            Some(observation_id.into()),
            json!({}),
        )
    }

    pub fn message_workflow_state_set(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "message.workflow_state.set",
            "PUT",
            "/api/v1/communications/messages/{message_id}/workflow-state",
            "communication_message",
            Some(message_id.into()),
            json!({}),
        )
    }

    pub fn person_identity_review_set(
        actor_id: impl Into<String>,
        identity_candidate_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "person_identity.review.set",
            "PUT",
            "/api/v1/identity-candidates/{identity_candidate_id}/review",
            "person_identity_candidate",
            Some(identity_candidate_id.into()),
            json!({}),
        )
    }
}
```

### `backend/src/platform/audit/settings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/settings.rs`
- Size bytes / Размер в байтах: `475`
- Included characters / Включено символов: `475`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn application_setting_set(
        actor_id: impl Into<String>,
        setting_key: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "application_setting.set",
            "PUT",
            "/api/v1/settings/{setting_key}",
            "application_setting",
            Some(setting_key.into()),
            json!({}),
        )
    }
}
```

### `backend/src/platform/audit/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/store.rs`
- Size bytes / Размер в байтах: `3171`
- Included characters / Включено символов: `3171`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::constants::EVENT_TARGET_KIND;
use super::errors::ApiAuditError;
use super::models::{ApiAuditRecord, NewApiAuditRecord};

#[derive(Clone)]
pub struct ApiAuditLog {
    pool: PgPool,
}

impl ApiAuditLog {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(&self, record: &NewApiAuditRecord) -> Result<i64, ApiAuditError> {
        let audit_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO api_audit_log (
                actor_kind,
                actor_id,
                operation,
                method,
                path_template,
                target_kind,
                target_id,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING audit_id
            "#,
        )
        .bind(&record.actor_kind)
        .bind(&record.actor_id)
        .bind(&record.operation)
        .bind(&record.method)
        .bind(&record.path_template)
        .bind(&record.target_kind)
        .bind(&record.target_id)
        .bind(&record.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(audit_id)
    }

    pub async fn list_event_records(
        &self,
        target_id: Option<&str>,
        actor_id: Option<&str>,
        after_audit_id: i64,
        limit: u32,
    ) -> Result<Vec<ApiAuditRecord>, ApiAuditError> {
        let target_id = target_id.map(str::trim).filter(|value| !value.is_empty());
        let actor_id = actor_id.map(str::trim).filter(|value| !value.is_empty());
        let after_audit_id = after_audit_id.max(0);
        let limit = i64::from(limit.clamp(1, 500));

        let rows = sqlx::query(
            r#"
            SELECT
                audit_id,
                recorded_at,
                actor_kind,
                actor_id,
                operation,
                method,
                path_template,
                target_kind,
                target_id,
                metadata
            FROM api_audit_log
            WHERE target_kind = $1
              AND ($2::text IS NULL OR target_id = $2)
              AND ($3::text IS NULL OR actor_id = $3)
              AND audit_id > $4
            ORDER BY audit_id ASC
            LIMIT $5
            "#,
        )
        .bind(EVENT_TARGET_KIND)
        .bind(target_id)
        .bind(actor_id)
        .bind(after_audit_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_audit_record).collect()
    }
}

fn row_to_audit_record(row: sqlx::postgres::PgRow) -> Result<ApiAuditRecord, ApiAuditError> {
    Ok(ApiAuditRecord {
        audit_id: row.try_get("audit_id")?,
        recorded_at: row.try_get("recorded_at")?,
        actor_kind: row.try_get("actor_kind")?,
        actor_id: row.try_get("actor_id")?,
        operation: row.try_get("operation")?,
        method: row.try_get("method")?,
        path_template: row.try_get("path_template")?,
        target_kind: row.try_get("target_kind")?,
        target_id: row.try_get("target_id")?,
        metadata: row.try_get("metadata")?,
    })
}
```

### `backend/src/platform/audit/telegram.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/telegram.rs`
- Size bytes / Размер в байтах: `21468`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};

use super::helpers::{insert_non_empty, insert_optional, non_empty_optional};
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn automation_telegram_send_dry_run(
        actor_id: impl Into<String>,
        outbound_message_id: impl Into<String>,
        policy_id: impl Into<String>,
        template_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        rendered_preview_hash: impl Into<String>,
    ) -> Self {
        let outbound_message_id = outbound_message_id.into();
        let policy_id = policy_id.into();
        let template_id = template_id.into();
        let account_id = account_id.into();
        let provider_chat_id = provider_chat_id.into();
        let rendered_preview_hash = rendered_preview_hash.into();
        let decision =
            CapabilityDecision::scoped_automation_allowed("telegram.send", policy_id.clone());

        Self::automation_telegram_send_dry_run_decision(
            actor_id,
            TelegramSendDryRunAuditDecision {
                target_kind: "telegram_outbound_message",
                target_id: Some(outbound_message_id),
                policy_id,
                template_id: Some(template_id),
                account_id: Some(account_id),
                provider_chat_id,
                rendered_preview_hash: Some(rendered_preview_hash),
                decision: &decision,
            },
        )
    }

    pub fn automation_telegram_send_dry_run_rejected(
        actor_id: impl Into<String>,
        command_id: impl Into<String>,
        policy_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        decision: &CapabilityDecision,
    ) -> Self {
        Self::automation_telegram_send_dry_run_decision(
            actor_id,
            TelegramSendDryRunAuditDecision {
                target_kind: "telegram_send_request",
                target_id: non_empty_optional(command_id.into()),
                policy_id: policy_id.into(),
                template_id: None,
                account_id: None,
                provider_chat_id: provider_chat_id.into(),
                rendered_preview_hash: None,
                decision,
            },
        )
    }

    fn automation_telegram_send_dry_run_decision(
        actor_id: impl Into<String>,
        audit_decision: TelegramSendDryRunAuditDecision<'_>,
    ) -> Self {
        let mut metadata = audit_decision.decision.audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "policy_id", audit_decision.policy_id);
        insert_optional(metadata_object, "template_id", audit_decision.template_id);
        insert_optional(metadata_object, "account_id", audit_decision.account_id);
        insert_non_empty(
            metadata_object,
            "provider_chat_id",
            audit_decision.provider_chat_id,
        );
        insert_optional(
            metadata_object,
            "rendered_preview_hash",
            audit_decision.rendered_preview_hash,
        );

        Self::new(
            actor_id,
            "automation.telegram_send.dry_run",
            "POST",
            "/api/v1/policies/telegram-send/dry-run",
            audit_decision.target_kind,
            audit_decision.target_id,
            metadata,
        )
    }

    pub fn telegram_message_send(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        rendered_preview_hash: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.message.send",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(
            metadata_object,
            "rendered_preview_hash",
            rendered_preview_hash.into(),
        );

        Self::new(
            actor_id,
            "telegram.message.send",
            "POST",
            "/api/v1/integrations/telegram/provider-commands/messages/send",
            "telegram_message",
            Some(message_id.into()),
            metadata,
        )
    }

    pub fn telegram_media_upload(
        actor_id: impl Into<String>,
        command_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        attachment_id: Option<&str>,
        blob_id: Option<&str>,
        media_type: Option<&str>,
    ) -> Self {
        let command_id = command_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.media.upload",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_optional(
            metadata_object,
            "attachment_id",
            attachment_id.map(ToOwned::to_owned),
        );
        insert_optional(metadata_object, "blob_id", blob_id.map(ToOwned::to_owned));
        insert_optional(
            metadata_object,
            "media_type",
            media_type.map(ToOwned::to_owned),
        );

        Self::new(
            actor_id,
            "telegram.media.upload",
            "POST",
            "/api/v1/integrations/telegram/provider-media/upload",
            "telegram_media_upload_command",
            Some(command_id),
            metadata,
        )
    }

    pub fn telegram_account_logout(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        lifecycle_state: impl Into<String>,
    ) -> Self {
        Self::telegram_account_lifecycle(
            actor_id,
            TelegramAccountLifecycleAudit {
                operation: "telegram.account.logout",
                method: "POST",
                path_template: "/api/v1/integrations/telegram/accounts/{account_id}/logout",
                capability: "telegram.account.logout",
                account_id: account_id.into(),
                provider_kind: provider_kind.into(),
                lifecycle_state: lifecycle_state.into(),
            },
        )
    }

    pub fn telegram_account_remove(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        lifecycle_state: impl Into<String>,
    ) -> Self {
        Self::telegram_account_lifecycle(
            actor_id,
            TelegramAccountLifecycleAudit {
                operation: "telegram.account.remove",
                method: "DELETE",
                path_template: "/api/v1/integrations/telegram/accounts/{account_id}",
                capability: "telegram.account.remove",
                account_id: account_id.into(),
                provider_kind: provider_kind.into(),
                lifecycle_state: lifecycle_state.into(),
            },
        )
    }

    pub fn telegram_runtime_stop(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        runtime_kind: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        let account_id = account_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            "telegram.runtime.stop",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.clone());
        insert_non_empty(metadata_object, "provider_kind", provider_kind.into());
        insert_non_empty(metadata_object, "runtime_kind", runtime_kind.into());
        insert_non_empty(metadata_object, "status", status.into());

        Self::new(
            actor_id,
            "telegram.runtime.stop",
            "POST",
            "/api/v1/integrations/telegram/runtime/stop",
            "communication_provider_account",
            Some(account_id),
            metadata,
        )
    }

    pub fn telegram_runtime_restart(
        actor_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_kind: impl Into<String>,
        runtime_kind: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        let account_id = account_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            "telegram.runtime.restart",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.clone());
        insert_non_empty(metadata_object, "provider_kind", provider_kind.into());
        insert_non_empty(metadata_object, "runtime_kind", runtime_kind.into());
        insert_non_empty(metadata_object, "status", status.into());

        Self::new(
            actor_id,
            "telegram.runtime.restart",
            "POST",
            "/api/v1/integrations/telegram/runtime/restart",
            "communication_provider_account",
            Some(account_id),
            metadata,
        )
    }

    fn telegram_account_lifecycle(
        actor_id: impl Into<String>,
        audit: TelegramAccountLifecycleAudit,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::LocalWrite,
            audit.capability,
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", audit.account_id.clone());
        insert_non_empty(metadata_object, "provider_kind", audit.provider_kind);
        insert_non_empty(metadata_object, "lifecycle_state", audit.lifecycle_state);

        Self::new(
            actor_id,
            audit.operation,
            audit.method,
            audit.path_template,
            "communication_provider_account",
            Some(audit.account_id),
            metadata,
        )
    }
    pub fn telegram_message_edit(
        actor_id: impl Into<String>,
        message_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.message.edit",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", pro
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/platform/audit/telegram_dialogs.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/telegram_dialogs.rs`
- Size bytes / Размер в байтах: `10762`
- Included characters / Включено символов: `10762`
- Truncated / Обрезано: `no`

```rust
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};

use super::helpers::{insert_non_empty, insert_optional};
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn telegram_chat_action(
        actor_id: impl Into<String>,
        telegram_chat_id: Option<&str>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_message_id: Option<&str>,
        command_kind: &str,
    ) -> Self {
        let capability = match command_kind {
            "pin" | "unpin" => "telegram.dialog.pin",
            "archive" | "unarchive" => "telegram.dialog.archive",
            "mute" | "unmute" => "telegram.dialog.mute",
            "folder_add" => "telegram.dialog.folder_add",
            "folder_remove" => "telegram.dialog.folder_remove",
            "folder_reassign" => "telegram.dialog.folder_reassign",
            "mark_read" | "mark_unread" => "telegram.dialog.mark_read",
            "join" => "telegram.participants.join",
            "leave" => "telegram.participants.leave",
            _ => "telegram.dialog.action",
        };
        let path_template = match command_kind {
            "join" => "/api/v1/integrations/telegram/provider-commands/conversations/join",
            "leave" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/leave"
            }
            "pin" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/pin"
            }
            "unpin" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unpin"
            }
            "archive" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/archive"
            }
            "unarchive" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unarchive"
            }
            "mute" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/mute"
            }
            "unmute" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unmute"
            }
            "folder_add" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}"
            }
            "folder_remove" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove"
            }
            "folder_reassign" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/reassign"
            }
            "mark_read" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/read"
            }
            "mark_unread" => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/unread"
            }
            _ => {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/action"
            }
        };
        let operation = match command_kind {
            "pin" => "telegram.chat.pin",
            "unpin" => "telegram.chat.unpin",
            "archive" => "telegram.chat.archive",
            "unarchive" => "telegram.chat.unarchive",
            "mute" => "telegram.chat.mute",
            "unmute" => "telegram.chat.unmute",
            "folder_add" => "telegram.chat.folder_add",
            "folder_remove" => "telegram.chat.folder_remove",
            "folder_reassign" => "telegram.chat.folder_reassign",
            "mark_read" => "telegram.chat.mark_read",
            "mark_unread" => "telegram.chat.mark_unread",
            "join" => "telegram.chat.join",
            "leave" => "telegram.chat.leave",
            _ => "telegram.chat.action",
        };
        let action_class = match command_kind {
            "pin" | "unpin" | "archive" | "unarchive" | "mute" | "unmute" | "folder_add"
            | "folder_remove" | "folder_reassign" | "mark_read" | "mark_unread" | "join"
            | "leave" => CapabilityActionClass::ProviderWrite,
            _ => CapabilityActionClass::LocalWrite,
        };

        let provider_chat_id = provider_chat_id.into();
        let target_id = telegram_chat_id
            .map(ToOwned::to_owned)
            .or_else(|| Some(provider_chat_id.clone()));
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            action_class,
            capability,
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id);
        insert_non_empty(metadata_object, "command_kind", command_kind.to_owned());
        insert_optional(
            metadata_object,
            "provider_message_id",
            provider_message_id.map(ToOwned::to_owned),
        );

        Self::new(
            actor_id,
            operation,
            "POST",
            path_template,
            "telegram_chat",
            target_id,
            metadata,
        )
    }

    pub fn telegram_chat_folder_add(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_folder_id: i64,
    ) -> Self {
        Self::telegram_chat_folder_mutation(
            actor_id,
            telegram_chat_id,
            account_id,
            provider_chat_id,
            provider_folder_id,
            "folder_add",
            "telegram.dialog.folder_add",
            "telegram.chat.folder_add",
            "POST",
        )
    }

    pub fn telegram_chat_folder_remove(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_folder_id: i64,
    ) -> Self {
        Self::telegram_chat_folder_mutation(
            actor_id,
            telegram_chat_id,
            account_id,
            provider_chat_id,
            provider_folder_id,
            "folder_remove",
            "telegram.dialog.folder_remove",
            "telegram.chat.folder_remove",
            "POST",
        )
    }

    pub fn telegram_chat_folder_reassign(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        target_provider_folder_ids: &[i64],
        added_provider_folder_ids: &[i64],
        removed_provider_folder_ids: &[i64],
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            "telegram.dialog.folder_reassign",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(
            metadata_object,
            "command_kind",
            "folder_reassign".to_owned(),
        );
        metadata_object.insert(
            "target_provider_folder_ids".to_owned(),
            serde_json::Value::Array(
                target_provider_folder_ids
                    .iter()
                    .copied()
                    .map(|value| serde_json::Value::Number(value.into()))
                    .collect(),
            ),
        );
        metadata_object.insert(
            "added_provider_folder_ids".to_owned(),
            serde_json::Value::Array(
                added_provider_folder_ids
                    .iter()
                    .copied()
                    .map(|value| serde_json::Value::Number(value.into()))
                    .collect(),
            ),
        );
        metadata_object.insert(
            "removed_provider_folder_ids".to_owned(),
            serde_json::Value::Array(
                removed_provider_folder_ids
                    .iter()
                    .copied()
                    .map(|value| serde_json::Value::Number(value.into()))
                    .collect(),
            ),
        );

        Self::new(
            actor_id,
            "telegram.chat.folder_reassign",
            "POST",
            "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/reassign",
            "telegram_chat",
            Some(telegram_chat_id.to_owned()),
            metadata,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn telegram_chat_folder_mutation(
        actor_id: impl Into<String>,
        telegram_chat_id: &str,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        provider_folder_id: i64,
        command_kind: &str,
        capability: &str,
        operation: &str,
        method: &str,
    ) -> Self {
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::ProviderWrite,
            capability,
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(metadata_object, "command_kind", command_kind.to_owned());
        insert_non_empty(
            metadata_object,
            "provider_folder_id",
            provider_folder_id.to_string(),
        );

        Self::new(
            actor_id,
            operation,
            method,
            if command_kind == "folder_remove" {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove"
            } else {
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}"
            },
            "telegram_chat",
            Some(telegram_chat_id.to_owned()),
            metadata,
        )
    }
}
```

### `backend/src/platform/audit/telegram_participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/audit/telegram_participants.rs`
- Size bytes / Размер в байтах: `2661`
- Included characters / Включено символов: `2661`
- Truncated / Обрезано: `no`

```rust
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};

use super::helpers::insert_non_empty;
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn telegram_participants_sync(
        actor_id: impl Into<String>,
        telegram_chat_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        synced_count: i64,
    ) -> Self {
        let telegram_chat_id = telegram_chat_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::Read,
            "telegram.participants.sync",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(
            metadata_object,
            "synced_count",
            synced_count.max(0).to_string(),
        );

        Self::new(
            actor_id,
            "telegram.participants.sync",
            "POST",
            "/api/v1/integrations/telegram/provider-sync/conversations/{chat_id}/members",
            "telegram_chat",
            Some(telegram_chat_id),
            metadata,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::platform::audit::NewApiAuditRecord;

    #[test]
    fn telegram_participants_sync_audit_preserves_capability_metadata() {
        let record = NewApiAuditRecord::telegram_participants_sync(
            "makosh-frontend",
            "telegram-chat-1",
            "account-1",
            "provider-chat-1",
            42,
        );

        assert_eq!(record.operation, "telegram.participants.sync");
        assert_eq!(record.method, "POST");
        assert_eq!(
            record.path_template,
            "/api/v1/integrations/telegram/provider-sync/conversations/{chat_id}/members"
        );
        assert_eq!(record.target_kind, "telegram_chat");
        assert_eq!(record.target_id.as_deref(), Some("telegram-chat-1"));
        assert_eq!(record.metadata["capability"], "telegram.participants.sync");
        assert_eq!(record.metadata["decision"], "allowed");
        assert_eq!(record.metadata["reason"], "explicit_user_confirmation");
        assert_eq!(record.metadata["account_id"], "account-1");
        assert_eq!(record.metadata["provider_chat_id"], "provider-chat-1");
        assert_eq!(record.metadata["synced_count"], "42");
    }
}
```

### `backend/src/platform/calls.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/calls.rs`
- Size bytes / Размер в байтах: `391`
- Included characters / Включено символов: `391`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod rows;
mod store;
mod stt;
mod validation;

pub use errors::CallError;
pub use models::{
    CallDirection, CallState, CallTranscript, NewCallTranscript, NewProviderCall, NewTelegramCall,
    ProviderCall, TelegramCall, TranscriptStatus,
};
pub use store::CallIntelligenceStore;
pub use stt::{FixtureSpeechToTextProvider, FixtureTranscript, SpeechToTextProvider};
```

### `backend/src/platform/calls/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/calls/errors.rs`
- Size bytes / Размер в байтах: `211`
- Included characters / Включено символов: `211`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CallError {
    #[error("invalid call intelligence request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/platform/calls/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/platform/calls/models.rs`
- Size bytes / Размер в байтах: `4389`
- Included characters / Включено символов: `4389`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::CallError;
use super::validation::{validate_array, validate_non_empty, validate_object};

#[derive(Clone, Debug, PartialEq)]
pub struct NewTelegramCall {
    pub call_id: String,
    pub account_id: String,
    pub provider_call_id: String,
    pub provider_chat_id: String,
    pub direction: CallDirection,
    pub call_state: CallState,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub transcription_policy_id: Option<String>,
    pub metadata: Value,
}

impl NewTelegramCall {
    pub(super) fn validate(&self) -> Result<(), CallError> {
        validate_non_empty("call_id", &self.call_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_call_id", &self.provider_call_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TelegramCall {
    pub call_id: String,
    pub account_id: String,
    pub provider_call_id: String,
    pub provider_chat_id: String,
    pub direction: String,
    pub call_state: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub transcription_policy_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type NewProviderCall = NewTelegramCall;
pub type ProviderCall = TelegramCall;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallDirection {
    Incoming,
    Outgoing,
}

impl CallDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incoming => "incoming",
            Self::Outgoing => "outgoing",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallState {
    Ringing,
    Active,
    Ended,
    Missed,
    Declined,
    Failed,
}

impl CallState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ringing => "ringing",
            Self::Active => "active",
            Self::Ended => "ended",
            Self::Missed => "missed",
            Self::Declined => "declined",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCallTranscript {
    pub transcript_id: String,
    pub call_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub transcript_status: TranscriptStatus,
    pub stt_provider: String,
    pub source_audio_ref: Option<String>,
    pub language_code: Option<String>,
    pub transcript_text: String,
    pub segments: Value,
    pub provenance: Value,
}

impl NewCallTranscript {
    pub(super) fn validate(&self) -> Result<(), CallError> {
        validate_non_empty("transcript_id", &self.transcript_id)?;
        validate_non_empty("call_id", &self.call_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("stt_provider", &self.stt_provider)?;
        validate_array("segments", &self.segments)?;
        validate_object("provenance", &self.provenance)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CallTranscript {
    pub transcript_id: String,
    pub call_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub transcript_status: String,
    pub stt_provider: String,
    pub source_audio_ref: Option<String>,
    pub language_code: Option<String>,
    pub transcript_text: String,
    pub segments: Value,
    pub provenance: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl TranscriptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}
```
