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

- Chunk ID / ID чанка: `033-source-backend-part-013`
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

### `backend/src/app/provider_runtime_handlers/whatsapp.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/whatsapp.rs`
- Size bytes / Размер в байтах: `230014`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app::api_support::{
    WhatsappCapabilitiesResponse, WhatsappWebListQuery, WhatsappWebMessageListResponse,
    WhatsappWebSessionListResponse, communication_provider_account_store,
    communication_provider_secret_binding_store, communication_storage_store,
    ensure_fixture_routes_enabled, event_store, message_store, whatsapp_fixture_ingest_service,
    whatsapp_provider_runtime_service, whatsapp_secret_reference_store,
};
use crate::app::signal_hub_support::{
    provider_account_or_not_found, remove_provider_account_signal_connection,
    sync_provider_account_signal_connection, sync_whatsapp_runtime_signal_connection,
};
use crate::app::{ApiError, AppState};
use crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService;
use crate::application::provider_runtime_contracts::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsAppAuthorizedSessionCredentialWrite,
    WhatsAppCommandDeadLetterRequest, WhatsAppConversationCommandRequest,
    WhatsAppCredentialBinding, WhatsAppDeleteRequest, WhatsAppEditRequest, WhatsAppForwardRequest,
    WhatsAppMediaDownloadRequest, WhatsAppMediaUploadRequest, WhatsAppPairCodeSession,
    WhatsAppPairCodeStartRequest, WhatsAppProviderCommand, WhatsAppProviderCommandListResponse,
    WhatsAppProviderCommandResponse, WhatsAppProviderRuntimeShape, WhatsAppQrLinkSession,
    WhatsAppQrLinkStartRequest, WhatsAppReactionRequest, WhatsAppReplyRequest,
    WhatsAppRuntimeHealth, WhatsAppRuntimeRelinkRequest, WhatsAppRuntimeRemoveRequest,
    WhatsAppRuntimeRemoveResponse, WhatsAppRuntimeRevokeRequest, WhatsAppRuntimeStartRequest,
    WhatsAppRuntimeStatus, WhatsAppRuntimeStopRequest, WhatsAppStatusPublishRequest,
    WhatsAppTextSendRequest, WhatsAppVoiceNoteSendRequest, WhatsappLiveAccountSetupRequest,
    WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse, WhatsappWebCallIngestResult,
    WhatsappWebDeliveryState, WhatsappWebDialogIngestResult, WhatsappWebError,
    WhatsappWebMediaIngestResult, WhatsappWebMessageDeleteIngestResult,
    WhatsappWebMessageIngestResult, WhatsappWebMessageUpdateIngestResult,
    WhatsappWebParticipantIngestResult, WhatsappWebPresenceIngestResult,
    WhatsappWebReactionIngestResult, WhatsappWebReceiptIngestResult,
    WhatsappWebRuntimeEventIngestResult, WhatsappWebStatusDeleteIngestResult,
    WhatsappWebStatusIngestResult, WhatsappWebStatusViewIngestResult,
    whatsapp_business_cloud_access_token_secret_ref, whatsapp_business_cloud_app_secret_ref,
    whatsapp_business_cloud_webhook_verify_token_ref,
};
use crate::domains::communications::messages::ProviderChannelMessageStore;
use crate::domains::communications::storage::AttachmentSafetyScanStatus;
use crate::platform::communications::{
    NewProviderAccountSecretBinding, ProviderAccount, ProviderAccountSecretPurpose,
};
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::{sanitize_event_payload, whatsapp_event_types};
use crate::platform::observations::ObservationOriginKind;
use crate::platform::secrets::{NewSecretReference, SecretKind, SecretStoreKind};
use crate::vault::SecretEntryContext;

const AUDIT_ACTOR_ID: &str = "makosh-frontend";
const BUSINESS_CLOUD_SIGNATURE_HEADER: &str = "x-hub-signature-256";
static WHATSAPP_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsappAccountSummary {
    pub(crate) account_id: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_shape: Option<String>,
    pub(crate) display_name: String,
    pub(crate) external_account_id: String,
    pub(crate) runtime: Option<String>,
    pub(crate) lifecycle_state: Option<String>,
    pub(crate) created_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsappAccountListResponse {
    pub(crate) items: Vec<WhatsappAccountSummary>,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappRuntimeAccountQuery {
    pub(crate) account_id: String,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

#[derive(Deserialize)]
pub(crate) struct WhatsAppCommandListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) command_kinds: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppChatSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppChatSyncItem {
    pub(crate) conversation_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: String,
    pub(crate) title: String,
    pub(crate) chat_kind: Option<String>,
    pub(crate) is_archived: bool,
    pub(crate) is_pinned: bool,
    pub(crate) is_muted: bool,
    pub(crate) is_unread: bool,
    pub(crate) unread_count: Option<i64>,
    pub(crate) participant_count: Option<i64>,
    pub(crate) community_parent_chat_id: Option<String>,
    pub(crate) community_parent_title: Option<String>,
    pub(crate) invite_link: Option<String>,
    pub(crate) is_community_root: bool,
    pub(crate) is_broadcast: bool,
    pub(crate) is_newsletter: bool,
    pub(crate) avatar_metadata: Value,
    pub(crate) provider_labels: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppChatSyncResponse {
    pub(crate) account_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) items: Vec<WhatsAppChatSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppHistorySyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppHistorySyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<crate::application::provider_runtime_contracts::WhatsappWebMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppMembersSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMembersSyncItem {
    pub(crate) participant_id: String,
    pub(crate) conversation_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_member_id: String,
    pub(crate) provider_identity_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) role: String,
    pub(crate) status: Option<String>,
    pub(crate) identity_kind: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) is_admin: bool,
    pub(crate) is_owner: bool,
    pub(crate) participant_metadata: Value,
    pub(crate) identity_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMembersSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppMembersSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppStatusSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppStatusSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<crate::application::provider_runtime_contracts::WhatsappWebMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppPresenceSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppPresenceSyncItem {
    pub(crate) identity_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_identity_id: String,
    pub(crate) identity_kind: String,
    pub(crate) display_name: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) presence_state: String,
    pub(crate) last_seen_at: Option<String>,
    pub(crate) observed_at: Option<String>,
    pub(crate) identity_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppPresenceSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppPresenceSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppCallsSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppCallsSyncItem {
    pub(crate) call_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_call_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) direction: String,
    pub(crate) call_state: String,
    pub(crate) started_at: Option<String>,
    pub(crate) ended_at: Option<String>,
    pub(crate) observed_at: Option<String>,
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppCallsSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppCallsSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppContactsSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppContactsSyncItem {
    pub(crate) identity_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_identity_id: String,
    pub(crate) identity_kind: String,
    pub(crate) display_name: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) push_name: Option<String>,
    pub(crate) business_profile: Value,
    pub(crate) profile_photo_ref: Value,
    pub(crate) display_name_history: Vec<String>,
    pub(crate) identity_metadata: Value,
    pub(crate) whatsapp_trace_metadata: Value,
    pub(crate)
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/provider_runtime_handlers/yandex_telemost.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/yandex_telemost.rs`
- Size bytes / Размер в байтах: `56512`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path as FsPath, PathBuf};
use uuid::Uuid;

use crate::app::api_support::{
    app_store, event_store, settings_store, yandex_telemost_provider_runtime_service,
    yandex_telemost_provider_runtime_store, yandex_telemost_secret_reference_store,
};
use crate::app::{ApiError, AppState};
use crate::domains::calendar::events::CalendarEventQueryPort;
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxStore, ReviewItemKind,
};
use crate::integrations::yandex_telemost::client::{
    TelemostCohost, YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_PROVIDER_KIND_STR,
    YANDEX_TELEMOST_WEB_ORIGIN, YandexTelemostAccountListResponse,
    YandexTelemostAccountSetupRequest, YandexTelemostAccountSetupResponse,
    YandexTelemostCapabilityState, YandexTelemostCohostPage, YandexTelemostConference,
    YandexTelemostConferenceOpenRequest, YandexTelemostConferencePatchRequest,
    YandexTelemostConferenceRequest, YandexTelemostConferenceWebviewManifest,
    YandexTelemostCreateConferenceCommand, YandexTelemostError,
    YandexTelemostLocalRecordingManifest, YandexTelemostLocalRecordingPolicy,
    YandexTelemostRecordingBridgeRequest, YandexTelemostRecordingBridgeResponse,
    YandexTelemostRetentionCleanupRequest, YandexTelemostRetentionCleanupResponse,
    YandexTelemostRuntimeStatus, YandexTelemostSpeakerTimelinePolicy,
    YandexTelemostTranscriptBridgeRequest, YandexTelemostTranscriptBridgeResponse,
    sanitize_yandex_telemost_payload, validate_telemost_join_url, webview_manifest_for_request,
    yandex_telemost_capabilities,
};
use crate::integrations::yandex_telemost::runtime_bridge::complete_yandex_telemost_transcript_bridge;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::yandex_telemost_event_types;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};
use crate::platform::realtime_conversation::{
    CallBundleManifest, REALTIME_CONVERSATION_AUDIO_CAPTURE_COMPLETED,
    REALTIME_CONVERSATION_CALL_BUNDLE_CREATED, REALTIME_CONVERSATION_RADAR_SIGNALS_DETECTED,
    REALTIME_CONVERSATION_SPEAKER_HINT_OBSERVED, REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED,
    RealtimeConversationProviderKind, SpeakerTimelineHint, build_call_bundle_manifest,
};
use crate::vault::VaultMode;
use crate::workflows::realtime_conversation_memory_pipeline::plan_memory_pipeline;
use crate::workflows::realtime_conversation_radar_projection::{
    RealtimeConversationRadarProjectionContext, call_bundle_radar_candidates,
};

const REALTIME_CONVERSATION_RADAR_SIGNAL_OBSERVATION_KIND: &str =
    "REALTIME_CONVERSATION_RADAR_SIGNAL";

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostRuntimeStatusQuery {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostCohostsQuery {
    #[serde(default)]
    pub(crate) offset: Option<u32>,
    #[serde(default)]
    pub(crate) limit: Option<u16>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostCapabilitiesResponse {
    pub(crate) provider_kind: &'static str,
    pub(crate) api_base_url: &'static str,
    pub(crate) web_origin: &'static str,
    pub(crate) capabilities: Vec<YandexTelemostCapabilityState>,
    pub(crate) recording_policy: YandexTelemostLocalRecordingManifest,
    pub(crate) speaker_timeline_policy: YandexTelemostSpeakerTimelinePolicy,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostConferenceOperationResponse {
    pub(crate) account_id: String,
    pub(crate) conference: YandexTelemostConference,
    pub(crate) status: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostRecordingIntentResponse {
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) consent_required: bool,
    pub(crate) source_of_truth: bool,
    pub(crate) local_recording: YandexTelemostLocalRecordingManifest,
    pub(crate) speaker_timeline: YandexTelemostSpeakerTimelinePolicy,
    pub(crate) tauri_commands: serde_json::Value,
}

pub(crate) async fn get_yandex_telemost_capabilities(
    State(state): State<AppState>,
) -> Result<Json<YandexTelemostCapabilitiesResponse>, ApiError> {
    let authorized = matches!(state.vault.status()?.state, VaultMode::Unlocked);
    Ok(Json(YandexTelemostCapabilitiesResponse {
        provider_kind: YANDEX_TELEMOST_PROVIDER_KIND_STR,
        api_base_url: YANDEX_TELEMOST_API_BASE_URL,
        web_origin: YANDEX_TELEMOST_WEB_ORIGIN,
        capabilities: yandex_telemost_capabilities(authorized),
        recording_policy: recording_policy_manifest(),
        speaker_timeline_policy: speaker_timeline_policy(),
    }))
}

pub(crate) async fn get_yandex_telemost_accounts(
    State(state): State<AppState>,
    Query(query): Query<YandexTelemostAccountsQuery>,
) -> Result<Json<YandexTelemostAccountListResponse>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_store(&state)?
            .list_accounts(query.include_removed)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_account(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostAccountSetupRequest>,
) -> Result<Json<YandexTelemostAccountSetupResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    Ok(Json(
        store
            .setup_account(&secret_store, &state.vault, &request)
            .await?,
    ))
}

pub(crate) async fn get_yandex_telemost_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<YandexTelemostRuntimeStatusQuery>,
) -> Result<Json<YandexTelemostRuntimeStatus>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_store(&state)?
            .runtime_status(&query.account_id)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_retention_cleanup(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<YandexTelemostRetentionCleanupRequest>,
) -> Result<Json<YandexTelemostRetentionCleanupResponse>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_service(&state)?
            .cleanup_retention(&account_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_conference(
    State(state): State<AppState>,
    Json(command): Json<YandexTelemostCreateConferenceCommand>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .create_conference(
            &secret_store,
            &state.vault,
            &command.account_id,
            &command.body,
        )
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id: command.account_id,
        conference,
        status: "created",
    }))
}

pub(crate) async fn get_yandex_telemost_conference(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .get_conference(&secret_store, &state.vault, &account_id, &conference_id)
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id,
        conference,
        status: "observed",
    }))
}

pub(crate) async fn patch_yandex_telemost_conference(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
    Json(request): Json<YandexTelemostConferencePatchRequest>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .update_conference(
            &secret_store,
            &state.vault,
            &account_id,
            &conference_id,
            &request,
        )
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id,
        conference,
        status: "updated",
    }))
}

pub(crate) async fn get_yandex_telemost_cohosts(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
    Query(query): Query<YandexTelemostCohostsQuery>,
) -> Result<Json<YandexTelemostCohostPage>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    Ok(Json(
        store
            .list_cohosts(
                &secret_store,
                &state.vault,
                &account_id,
                &conference_id,
                query.offset,
                query.limit,
            )
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_webview_manifest(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostConferenceOpenRequest>,
) -> Result<Json<YandexTelemostConferenceWebviewManifest>, ApiError> {
    validate_telemost_join_url(&request.join_url)?;
    let window_label = telemost_window_label(&request.account_id, request.conference_id.as_deref());
    publish_yandex_telemost_companion_event(
        &state,
        yandex_telemost_event_types::WEBVIEW_OPEN_REQUESTED,
        "webview_open_requested",
        &request,
        json!({ "window_label": window_label.clone(), "owner_visible": true, "hidden_headless_mode": "forbidden" }),
    )
    .await?;
    Ok(Json(webview_manifest_for_request(
        &request,
        window_label,
        false,
        false,
    )))
}

pub(crate) async fn post_yandex_telemost_recording_intent(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostConferenceOpenRequest>,
) -> Result<Json<YandexTelemostRecordingIntentResponse>, ApiError> {
    validate_telemost_join_url(&request.join_url)?;
    publish_yandex_telemost_companion_event(
        &state,
        yandex_telemost_event_types::LOCAL_RECORDING_REQUESTED,
        "local_recording_requested",
        &request,
        json!({
            "consent_required": true,
            "source_of_truth": false,
            "audio_format": "mp3",
            "speaker_timeline": "hint_not_truth"
        }),
    )
    .await?;
    Ok(Json(YandexTelemostRecordingIntentResponse {
        account_id: request.account_id,
        conference_id: request.conference_id,
        join_url: request.join_url,
        consent_required: true,
        source_of_truth: false,
        local_recording: recording_policy_manifest(),
        speaker_timeline: speaker_timeline_policy(),
        tauri_commands: json!({
            "open_webview": "open_yandex_telemost_companion",
            "prepare_audio_device": "yandex_telemost_prepare_audio_device",
            "start_recording": "yandex_telemost_recording_start",
            "stop_recording": "yandex_telemost_recording_stop",
            "append_speaker_hint": "yandex_telemost_speaker_timeline_append"
        }),
    }))
}

pub(crate) async fn post_yandex_telemost_runtime_bridge_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/provider_runtime_handlers/zoom.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/provider_runtime_handlers/zoom.rs`
- Size bytes / Размер в байтах: `52907`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;

use crate::app::api_support::{
    ensure_fixture_routes_enabled, settings_store, zoom_provider_runtime_service,
    zoom_secret_reference_store,
};
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomOAuthCompleteRequest,
    ZoomOAuthStartRequest, ZoomOAuthStartResponse, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingObservationRequest, ZoomRecordingSyncRequest,
    ZoomRecordingSyncResult, ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse,
    ZoomRuntimeRemoveRequest, ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest,
    ZoomRuntimeStatus, ZoomRuntimeStopRequest, ZoomServerToServerAuthorizeRequest,
    ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult, ZoomTokenRefreshRequest,
    ZoomTokenRefreshResult, ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult,
    ZoomTranscriptIngestResult, ZoomTranscriptObservationRequest,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
};
use crate::domains::communications::core::CommunicationProviderSecretBindingStore;
use crate::integrations::zoom::client::{ZoomError, ZoomRecordingRef};
use crate::platform::communications::ProviderAccountSecretPurpose;
use crate::vault::{HostVaultError, VaultMode};

const ZOOM_SIGNATURE_HEADER: &str = "x-zm-signature";
const ZOOM_TIMESTAMP_HEADER: &str = "x-zm-request-timestamp";
const ZOOM_WEBHOOK_SIGNATURE_TOLERANCE_SECONDS: i64 = 300;
const ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_ENABLED_SETTING_KEY: &str =
    "privacy.zoom_remote_transcript_download_enabled";
const ZOOM_REMOTE_RECORDING_DOWNLOAD_ENABLED_SETTING_KEY: &str =
    "privacy.zoom_remote_recording_download_enabled";
const ZOOM_REMOTE_RECORDING_DOWNLOAD_NOT_ENABLED: &str =
    "zoom_remote_recording_download_not_enabled";
const ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_NOT_ENABLED: &str =
    "zoom_remote_transcript_download_not_enabled";
type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub(crate) struct ZoomAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

#[derive(Deserialize)]
pub(crate) struct ZoomRuntimeStatusQuery {
    pub(crate) account_id: String,
}

#[derive(Deserialize)]
pub(crate) struct ZoomRecordingImportsQuery {
    #[serde(default = "default_zoom_recording_imports_limit")]
    pub(crate) limit: i64,
}

#[derive(Deserialize)]
pub(crate) struct ZoomAuditEventsQuery {
    #[serde(default = "default_zoom_audit_events_limit")]
    pub(crate) limit: i64,
}

#[derive(Deserialize)]
pub(crate) struct ZoomWebhookSubscriptionStatusQuery {
    pub(crate) account_id: String,
    pub(crate) api_base_url: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ZoomWebhookQuery {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug)]
struct ZoomWebhookTranscriptDownload {
    request: ZoomTranscriptFileImportRequest,
    download_url: String,
    download_token: Option<String>,
}

#[derive(Clone, Debug)]
struct ZoomWebhookRecordingMediaDownload {
    request: ZoomRecordingMediaDownloadRequest,
    download_token: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ZoomCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) capabilities: Vec<ZoomCapabilityStatus>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ZoomCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) category: &'static str,
    pub(crate) status: &'static str,
    pub(crate) action_class: &'static str,
    pub(crate) confirmation_required: bool,
    pub(crate) reason: &'static str,
}

pub(crate) async fn get_zoom_capabilities() -> Result<Json<ZoomCapabilitiesResponse>, ApiError> {
    Ok(Json(ZoomCapabilitiesResponse {
        version: "1.0",
        runtime_mode: "fixture_plus_authorized_live_workers",
        capabilities: vec![
            ZoomCapabilityStatus {
                capability: "accounts.fixture",
                category: "accounts",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Fixture Zoom accounts can be registered for local validation.",
            },
            ZoomCapabilityStatus {
                capability: "accounts.live_blocked",
                category: "accounts",
                status: "degraded",
                action_class: "local_write",
                confirmation_required: true,
                reason: "Live Zoom account metadata and secret references can be registered, but provider execution is blocked.",
            },
            ZoomCapabilityStatus {
                capability: "auth.oauth_user",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Zoom OAuth user grants can be exchanged and stored through host-vault secret references.",
            },
            ZoomCapabilityStatus {
                capability: "auth.server_to_server",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Zoom Server-to-Server OAuth account credentials can be exchanged and stored through host-vault secret references.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_refresh",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom OAuth and Server-to-Server credentials can be renewed through host-vault token bundle updates.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_maintenance",
                category: "authorization",
                status: "available",
                action_class: "external_provider_write",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can be scanned and expiring token bundles renewed through the same host-vault refresh boundary.",
            },
            ZoomCapabilityStatus {
                capability: "auth.token_rotation_policy",
                category: "authorization",
                status: "available",
                action_class: "read",
                confirmation_required: false,
                reason: "Runtime status exposes the Zoom token rotation policy, refresh due state and failure blocker without exposing raw token material.",
            },
            ZoomCapabilityStatus {
                capability: "token_maintenance.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "The local backend scheduler can invoke token maintenance behind Signal Hub and HostVault gates.",
            },
            ZoomCapabilityStatus {
                capability: "runtime.status",
                category: "runtime",
                status: "available",
                action_class: "read",
                confirmation_required: false,
                reason: "Runtime/account lifecycle state is exposed without reading provider secrets.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.meetings",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Meeting observations are stored as provider call evidence and emitted as Zoom events.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.recordings",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Recording observations are event-sourced and sanitized before dispatch.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.transcripts",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Transcript observations are linked to provider call evidence for AI/consistency workflows.",
            },
            ZoomCapabilityStatus {
                capability: "bridge.transcript_files",
                category: "ingest",
                status: "available",
                action_class: "local_write",
                confirmation_required: false,
                reason: "Zoom transcript files can be imported from VTT, SRT or plain text into provider call transcript evidence.",
            },
            ZoomCapabilityStatus {
                capability: "provider_sync.recordings",
                category: "ingest",
                status: "available",
                action_class: "external_provider_read",
                confirmation_required: true,
                reason: "Authorized Zoom accounts can manually synchronize cloud recording metadata; provider-side recording media and transcript-like file downloads require owner-visible privacy opt-in settings.",
            },
            ZoomCapabilityStatus {
                capability: "recording_imports.remove",
                category: "retention",
                status: "available",
                action_class: "local_delete",
                confirmation_required: true,
                reason: "Imported Zoom recording blobs can be explicitly removed per account through the local retention control surface, with follow-up audit events.",
            },
            ZoomCapabilityStatus {
                capability: "retention.cleanup",
                category: "retention",
                status: "available",
                action_class: "local_delete",
                confirmation_required: true,
                reason: "Expired Zoom recording imports and transcript evidence can be pruned through the owner-visible retention control surface using stamped expiry intent.",
            },
            ZoomCapabilityStatus {
                capability: "retention.cleanup.scheduler",
                category: "runtime",
                status: "available",
                action_class: "local_automation",
                confirmation_required: false,
                reason: "The local backend scheduler can periodically prune expired Zoom recording imports and transcript evidence through the same retention boundary.",
            },
            ZoomCapabilityStatus {
                capability: "provider_sync.recordings.scheduler",
                category: "runtime",
                status: "available",
                action_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/router.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router.rs`
- Size bytes / Размер в байтах: `6494`
- Included characters / Включено символов: `6494`
- Truncated / Обрезано: `no`

```rust
// ADR-0073: app router owns HTTP composition; route groups live in
// focused modules so endpoint registration remains auditable without a god file.
use std::io;

use axum::extract::State;
use axum::http::{HeaderName, Method, StatusCode, header};
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;

use crate::app::vault_reconciliation::spawn_host_vault_manifest_reconciliation;
use crate::app::{AccountSetupState, AppError, AppState};
use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::platform::config::AppConfig;
use crate::platform::events::EventBus;
use crate::platform::storage::{Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus};
use crate::vault::{HostVault, HostVaultConfig};

mod routes;

pub fn build_router(config: AppConfig) -> Router {
    build_router_with_database(config, Database::disabled())
}

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    let api_secret = config.local_api_secret().unwrap_or_default().to_owned();
    let nats_server_url = config.nats_server_url().map(ToOwned::to_owned);
    let vault = HostVault::new(HostVaultConfig {
        home: config.vault_home().to_path_buf(),
        dev_mode: config.dev_mode(),
        dev_key_path: config.dev_key_path().to_path_buf(),
    })
    .expect("host vault runtime must initialize");
    if let Err(error) = vault.unlock_existing() {
        tracing::warn!(error = %error, "host vault auto-unlock skipped");
    }
    let state = AppState {
        config,
        database,
        vault,
        account_setup: AccountSetupState::default(),
        telegram_runtime: TelegramRuntimeManager::default(),
        event_bus: EventBus::new(),
    };
    spawn_host_vault_manifest_reconciliation(&state);
    crate::application::bootstrap::start_background_services(
        crate::application::bootstrap::ApplicationBootstrapContext {
            pool: state.database.pool().cloned(),
            database_url: state.database.database_url().map(ToOwned::to_owned),
            nats_server_url,
            zoom_token_maintenance_scheduler_enabled: state
                .config
                .zoom_token_maintenance_scheduler_enabled(),
            zoom_recording_sync_scheduler_enabled: state
                .config
                .zoom_recording_sync_scheduler_enabled(),
            zoom_retention_cleanup_scheduler_enabled: state
                .config
                .zoom_retention_cleanup_scheduler_enabled(),
            vault: state.vault.clone(),
            telegram_runtime: state.telegram_runtime.clone(),
            event_bus: state.event_bus.clone(),
        },
    );

    let connect_routes = crate::app::connectrpc::protected_routes(
        state.database.pool().cloned(),
        state.config.clone(),
        api_secret.clone(),
    );

    Router::<AppState>::new()
        .merge(routes::public_routes())
        .merge(connect_routes)
        .merge(routes::protected_routes(api_secret))
        .with_state(state)
        .layer(local_frontend_cors_layer())
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    service: String,
}

#[derive(Serialize)]
pub(crate) struct ReadinessResponse {
    status: &'static str,
    service: String,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
pub(crate) struct ReadinessChecks {
    database: DatabaseReadiness,
    migrations: MigrationReadiness,
}

pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let http_addr = config.http_addr();
    let database = Database::connect(config.database_url()).await?;
    let listener = TcpListener::bind(http_addr).await?;

    tracing::info!(%http_addr, "starting Макошь backend");

    axum::serve(listener, build_router_with_database(config, database)).await?;

    Ok(())
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let log_format = std::env::var("MAKOSH_LOG_FORMAT").unwrap_or_else(|_| "plain".to_owned());

    if log_format.eq_ignore_ascii_case("json") {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_current_span(true)
            .with_span_list(false)
            .flatten_event(true)
            .try_init();
        return;
    }

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

pub(crate) fn local_frontend_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin
                .to_str()
                .map(is_allowed_local_frontend_origin)
                .unwrap_or(false)
        }))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static("x-makosh-secret"),
        ])
}

fn is_allowed_local_frontend_origin(origin: &str) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };

    matches!(
        (url.scheme(), url.host_str()),
        (
            "http" | "https",
            Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
        ) | ("http" | "https", Some("tauri.localhost"))
            | ("tauri", Some("localhost"))
    )
}

pub(crate) async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name().to_owned(),
    })
}

pub(crate) async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let database = state.database.readiness().await;
    let migrations = state.database.migration_readiness().await;
    let is_ready =
        database.status() == ReadinessStatus::Ok && migrations.status() == ReadinessStatus::Ok;

    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status: if is_ready { "ok" } else { "degraded" },
            service: state.config.service_name().to_owned(),
            checks: ReadinessChecks {
                database,
                migrations,
            },
        }),
    )
}
```

### `backend/src/app/router/routes/ai.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/ai.rs`
- Size bytes / Размер в байтах: `1930`
- Included characters / Включено символов: `1930`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/ai/status", get(get_ai_status))
        .route(
            "/api/v1/ai/settings/overview",
            get(get_ai_settings_overview),
        )
        .route(
            "/api/v1/ai/providers",
            get(get_ai_providers).post(post_ai_provider),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}",
            patch(patch_ai_provider),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/test",
            post(post_ai_provider_test),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/sync-models",
            post(post_ai_provider_sync_models),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/consent",
            post(post_ai_provider_consent),
        )
        .route("/api/v1/ai/models", get(get_ai_models))
        .route("/api/v1/ai/model-routes/{slot}", put(put_ai_model_route))
        .route(
            "/api/v1/ai/prompts",
            get(get_ai_prompts).post(post_ai_prompt),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/versions",
            post(post_ai_prompt_version),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/activate",
            post(post_ai_prompt_activate),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/test",
            post(post_ai_prompt_test),
        )
        .route("/api/v1/ai/agents", get(get_ai_agents))
        .route("/api/v1/ai/runs", get(get_ai_runs))
        .route("/api/v1/ai/runs/{run_id}", get(get_ai_run))
        .route("/api/v1/ai/answers", post(post_ai_answer))
        .route(
            "/api/v1/ai/task-candidates/refresh",
            post(post_ai_task_candidates_refresh),
        )
        .route("/api/v1/ai/meeting-prep", post(post_ai_meeting_prep))
}
```

### `backend/src/app/router/routes/audit_events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/audit_events.rs`
- Size bytes / Размер в байтах: `804`
- Included characters / Включено символов: `804`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/audit/events", get(get_audit_events))
        .route("/api/events/ws", get(get_events_websocket))
        .route("/api/events/realtime/ws", get(get_realtime_websocket))
        .route("/api/events/stream", get(get_events_stream))
        .route("/api/v1/events", get(get_events).post(post_event))
        .route("/api/v1/events/{event_id}", get(get_event))
        .route(
            "/api/v1/events/{event_id}/children",
            get(get_event_children),
        )
        .route("/api/v1/events/{event_id}/trace", get(get_event_trace))
        .route(
            "/api/v1/event-traces/{correlation_id}",
            get(get_event_trace_by_correlation),
        )
}
```

### `backend/src/app/router/routes/calendar.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/calendar.rs`
- Size bytes / Размер в байтах: `5398`
- Included characters / Включено символов: `5398`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/calendar/accounts",
            get(get_calendar_accounts).post(post_calendar_account),
        )
        .route(
            "/api/v1/calendar/accounts/{account_id}",
            get(get_calendar_account)
                .put(put_calendar_account)
                .delete(delete_calendar_account),
        )
        .route(
            "/api/v1/calendar/accounts/{account_id}/sources",
            get(get_calendar_sources).post(post_calendar_source),
        )
        .route(
            "/api/v1/calendar/events",
            get(get_calendar_events).post(post_calendar_event),
        )
        .route(
            "/api/v1/calendar/events/{event_id}",
            get(get_calendar_event)
                .put(put_calendar_event)
                .delete(delete_calendar_event),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/reschedule",
            post(post_calendar_event_reschedule),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/cancel",
            post(post_calendar_event_cancel),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/participants",
            get(get_event_participants).post(post_event_participant),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/relations",
            get(get_event_relations).post(post_event_relation),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/context-pack",
            get(get_event_context_pack).post(post_event_context_pack),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/agenda",
            get(get_event_agenda).post(post_event_agenda),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/checklist",
            get(get_event_checklist).post(post_event_checklist),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/classify",
            post(post_event_classify),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/analyze",
            post(post_event_analyze),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/risks",
            get(get_event_risks),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/notes",
            get(get_meeting_notes).post(post_meeting_note),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/outcomes",
            get(get_meeting_outcomes).post(post_meeting_outcome),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/recording",
            get(get_event_recordings).post(post_event_recording),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/transcript",
            get(get_event_transcript),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/follow-up",
            post(post_event_follow_up),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/follow-up-status",
            get(get_event_follow_up_status),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/brief",
            get(get_event_brief),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/generate-agenda",
            post(post_generate_agenda),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/export",
            get(get_event_export),
        )
        .route(
            "/api/v1/calendar/deadlines",
            get(get_deadlines).post(post_deadline),
        )
        .route(
            "/api/v1/calendar/focus-blocks",
            get(get_focus_blocks).post(post_focus_block),
        )
        .route("/api/v1/calendar/smart-schedule", post(post_smart_schedule))
        .route("/api/v1/calendar/watchtower", get(get_calendar_watchtower))
        .route("/api/v1/calendar/health", get(get_calendar_health))
        .route("/api/v1/calendar/weekly-brief", get(get_weekly_brief))
        .route("/api/v1/calendar/analytics", get(get_calendar_analytics))
        .route("/api/v1/calendar/brain", post(post_calendar_brain))
        .route("/api/v1/calendar/search", get(get_calendar_search))
        .route(
            "/api/v1/calendar/rules",
            get(get_calendar_rules).post(post_calendar_rule),
        )
        .route(
            "/api/v1/calendar/rules/{rule_id}",
            put(put_calendar_rule).delete(delete_calendar_rule),
        )
        .route("/api/v1/calendar/import", post(post_calendar_import))
        .route(
            "/api/v1/calendar/accounts/{account_id}/sync",
            post(post_calendar_sync),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/reminders",
            get(get_event_reminders).post(post_event_reminder),
        )
        .route(
            "/api/v1/calendar/events/{event_id}/reminders/{reminder_id}/toggle",
            post(post_event_reminder_toggle),
        )
        .route(
            "/api/v1/calendar/analytics/distribution",
            get(get_time_distribution),
        )
        .route(
            "/api/v1/calendar/analytics/focus-balance",
            get(get_focus_balance),
        )
        .route(
            "/api/v1/calendar/analytics/back-to-back",
            get(get_back_to_back),
        )
}
```

### `backend/src/app/router/routes/communications.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/communications.rs`
- Size bytes / Размер в байтах: `9711`
- Included characters / Включено символов: `9711`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/communications/messages",
            get(get_v1_communication_messages),
        )
        .route(
            "/api/v1/communications/messages/bulk-actions",
            post(post_v1_messages_bulk_action),
        )
        .route(
            "/api/v1/communications/messages/{message_id}",
            get(get_v1_communication_message)
                .patch(post_telegram_message_edit)
                .delete(post_telegram_message_delete),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/workflow-state",
            put(put_v1_message_workflow_state),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/ai-state",
            get(get_v1_message_ai_state).put(put_v1_message_ai_state),
        )
        .route(
            "/api/v1/communications/messages/states",
            get(get_v1_message_workflow_state_counts),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/analyze",
            post(post_v1_message_analyze),
        )
        .route("/api/v1/workflow-actions", post(post_v1_workflow_action))
        .route("/api/v1/communications/threads", get(get_v1_threads))
        .route(
            "/api/v1/communications/threads/translate",
            post(post_v1_translate_thread),
        )
        .route(
            "/api/v1/communications/threads/messages",
            get(get_v1_thread_messages),
        )
        .route("/api/v1/communications/search", get(get_v1_email_search))
        .route(
            "/api/v1/communications/personas",
            get(get_v1_personas).post(post_v1_persona),
        )
        .route(
            "/api/v1/communications/drafts",
            get(get_v1_drafts).post(post_v1_draft),
        )
        .route(
            "/api/v1/communications/drafts/{draft_id}",
            get(get_v1_draft).delete(delete_v1_draft),
        )
        .route("/api/v1/communications/outbox", get(get_v1_outbox))
        .route(
            "/api/v1/communications/outbox/{outbox_id}/undo",
            post(post_v1_outbox_undo),
        )
        .route(
            "/api/v1/communications/read-receipts",
            post(post_v1_read_receipt),
        )
        .route(
            "/api/v1/communications/delivery-notifications",
            post(post_v1_delivery_notification),
        )
        .route(
            "/api/v1/integrations/mail/provider-delivery-events",
            post(post_v1_provider_delivery_event),
        )
        .route(
            "/api/v1/communications/saved-searches",
            get(get_v1_saved_searches).post(post_v1_saved_search),
        )
        .route(
            "/api/v1/communications/saved-searches/{saved_search_id}",
            put(put_v1_saved_search).delete(delete_v1_saved_search),
        )
        .route(
            "/api/v1/communications/folders",
            get(get_v1_mail_folders).post(post_v1_mail_folder),
        )
        .route(
            "/api/v1/communications/folders/{folder_id}",
            put(put_v1_mail_folder).delete(delete_v1_mail_folder),
        )
        .route(
            "/api/v1/communications/folders/{folder_id}/messages",
            get(get_v1_mail_folder_messages),
        )
        .route(
            "/api/v1/communications/folders/{folder_id}/messages/{message_id}/copy",
            post(post_v1_copy_message_to_folder),
        )
        .route(
            "/api/v1/communications/folders/{folder_id}/messages/{message_id}/move",
            post(post_v1_move_message_to_folder),
        )
        .route(
            "/api/v1/communications/finance/invoices",
            get(get_v1_invoices).post(post_v1_invoice),
        )
        .route(
            "/api/v1/communications/analytics/health",
            get(get_v1_analytics_health),
        )
        .route(
            "/api/v1/communications/analytics/senders",
            get(get_v1_analytics_senders),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/explain",
            get(get_v1_message_explain),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/smart-cc",
            get(get_v1_message_smart_cc),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/pin",
            post(post_v1_message_pin),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/important",
            post(post_v1_message_important),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/snooze",
            post(post_v1_message_snooze),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/mute",
            post(post_v1_message_mute),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/labels",
            post(post_v1_message_label).delete(delete_v1_message_label),
        )
        .route(
            "/api/v1/communications/subscriptions",
            get(get_v1_subscriptions),
        )
        .route(
            "/api/v1/communications/attachments/search",
            get(get_v1_attachment_search),
        )
        .route(
            "/api/v1/communications/attachments/import",
            post(post_v1_attachment_import),
        )
        .route(
            "/api/v1/communications/attachments/{attachment_id}/translate",
            post(post_v1_translate_attachment),
        )
        .route(
            "/api/v1/communications/attachments/{attachment_id}/preview",
            get(get_v1_attachment_preview),
        )
        .route(
            "/api/v1/communications/attachments/{attachment_id}/archive-inspection",
            get(get_v1_attachment_archive_inspection),
        )
        .route(
            "/api/v1/communications/attachments/duplicates",
            get(get_v1_attachment_duplicates),
        )
        .route(
            "/api/v1/communications/legal",
            get(get_v1_legal_docs).post(post_v1_legal_doc),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/export",
            get(get_v1_message_export),
        )
        .route("/api/v1/communications/send", post(post_v1_send))
        .route(
            "/api/v1/communications/messages/{message_id}/reply",
            post(post_v1_reply),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/imap-mark-read",
            post(post_v1_imap_mark_read),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/imap-delete",
            post(post_v1_imap_delete),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/trash",
            post(post_v1_message_trash),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/restore",
            post(post_v1_message_restore),
        )
        .route(
            "/api/v1/communications/certificates",
            get(get_v1_certs).post(post_v1_cert),
        )
        .route(
            "/api/v1/communications/certificates/expiring",
            get(get_v1_certs_expiring),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/signature",
            get(get_v1_signature_check),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/forward",
            post(post_v1_forward),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/redirect",
            post(post_v1_redirect),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/detect-language",
            get(get_v1_detect_language),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/translate",
            post(post_v1_translate),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/ai-reply",
            post(post_v1_ai_reply),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/ai-reply-variants",
            post(post_v1_ai_reply_variants),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/bilingual-reply-flow",
            post(post_v1_bilingual_reply_flow),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/reply-all",
            post(post_v1_reply_all),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/forward-eml",
            post(post_v1_forward_eml),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/spf-dkim",
            get(get_v1_spf_dkim),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/extract-tasks",
            post(post_v1_extract_tasks),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/extract-notes",
            post(post_v1_extract_notes),
        )
        .route(
            "/api/v1/communications/templates/rich",
            get(get_v1_rich_templates).post(post_v1_rich_template),
        )
        .route(
            "/api/v1/communications/templates/rich/mail-merge-preview",
            post(post_v1_rich_template_mail_merge_preview),
        )
        .route(
            "/api/v1/communications/templates/rich/{template_id}",
            delete(delete_v1_rich_template),
        )
        .route(
            "/api/v1/communications/templates/rich/render",
            post(post_v1_render_template),
        )
        .route("/api/v1/communications/blockers", get(get_v1_blockers))
}
```

### `backend/src/app/router/routes/email_accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/email_accounts.rs`
- Size bytes / Размер в байтах: `1852`
- Included characters / Включено символов: `1852`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            post(post_gmail_oauth_start),
        )
        .route(
            "/api/v1/integrations/mail/accounts/gmail/oauth/complete",
            post(post_gmail_oauth_complete),
        )
        .route(
            "/api/v1/integrations/mail/accounts",
            get(get_v1_email_accounts),
        )
        .route(
            "/api/v1/integrations/mail/accounts/import",
            post(post_v1_email_account_import),
        )
        .route(
            "/api/v1/integrations/mail/accounts/imap",
            post(post_imap_account_setup),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}",
            get(get_v1_email_account).delete(delete_v1_email_account),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/export",
            get(get_v1_email_account_export),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/logout",
            post(post_v1_email_account_logout),
        )
        .route(
            "/api/v1/integrations/mail/accounts/sync-status",
            get(get_v1_email_account_sync_status),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sync-settings",
            get(get_v1_email_account_sync_settings).put(put_v1_email_account_sync_settings),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sync-now",
            post(post_v1_email_account_sync_now),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sync-full-resync",
            post(post_v1_email_account_sync_full_resync),
        )
}
```

### `backend/src/app/router/routes/knowledge.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/knowledge.rs`
- Size bytes / Размер в байтах: `1146`
- Included characters / Включено символов: `1146`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/graph/summary", get(get_graph_summary))
        .route("/api/v1/graph/nodes", get(get_graph_nodes))
        .route("/api/v1/graph/neighborhood", get(get_graph_neighborhood))
        .route("/api/v1/graph/search", get(get_graph_search))
        .route("/api/v1/projects", get(get_projects))
        .route("/api/v1/projects/{project_id}", get(get_project_detail))
        .route(
            "/api/v1/projects/{project_id}/link-candidates",
            get(get_project_link_candidates),
        )
        .route(
            "/api/v1/projects/{project_id}/link-reviews",
            put(put_project_link_review),
        )
        .route(
            "/api/v1/documents/{document_id}/processing",
            get(get_document_processing),
        )
        .route(
            "/api/v1/document-processing/jobs",
            get(get_document_processing_jobs),
        )
        .route(
            "/api/v1/document-processing/jobs/{job_id}/retry",
            post(post_document_processing_job_retry),
        )
}
```

### `backend/src/app/router/routes/messaging.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/messaging.rs`
- Size bytes / Размер в байтах: `30169`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/integrations/telegram/capabilities",
            get(get_telegram_capabilities),
        )
        .route(
            "/api/v1/integrations/whatsapp/capabilities",
            get(get_whatsapp_capabilities),
        )
        .route(
            "/api/v1/integrations/zoom/capabilities",
            get(get_zoom_capabilities),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/capabilities",
            get(get_yandex_telemost_capabilities),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/accounts",
            get(get_yandex_telemost_accounts).post(post_yandex_telemost_account),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/runtime/status",
            get(get_yandex_telemost_runtime_status),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/accounts/{account_id}/retention/prune",
            post(post_yandex_telemost_retention_cleanup),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/conferences",
            post(post_yandex_telemost_conference),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}",
            get(get_yandex_telemost_conference).patch(patch_yandex_telemost_conference),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}/cohosts",
            get(get_yandex_telemost_cohosts),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/webview/manifest",
            post(post_yandex_telemost_webview_manifest),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/recording/intent",
            post(post_yandex_telemost_recording_intent),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/runtime-bridge/recordings",
            post(post_yandex_telemost_runtime_bridge_recording),
        )
        .route(
            "/api/v1/integrations/yandex-telemost/runtime-bridge/transcripts",
            post(post_yandex_telemost_runtime_bridge_transcript),
        )
        .route(
            "/api/v1/integrations/zoom/accounts",
            get(get_zoom_accounts).post(post_zoom_account),
        )
        .route(
            "/api/v1/integrations/zoom/oauth/start",
            post(post_zoom_oauth_start),
        )
        .route(
            "/api/v1/integrations/zoom/oauth/complete",
            post(post_zoom_oauth_complete),
        )
        .route(
            "/api/v1/integrations/zoom/oauth/server-to-server/authorize",
            post(post_zoom_server_to_server_authorize),
        )
        .route(
            "/api/v1/integrations/zoom/oauth/refresh",
            post(post_zoom_oauth_refresh),
        )
        .route(
            "/api/v1/integrations/zoom/oauth/maintenance",
            post(post_zoom_oauth_maintenance),
        )
        .route(
            "/api/v1/integrations/zoom/provider-sync/recordings",
            post(post_zoom_provider_sync_recordings),
        )
        .route(
            "/api/v1/integrations/zoom/webhook-subscriptions/status",
            get(get_zoom_webhook_subscription_status),
        )
        .route(
            "/api/v1/integrations/zoom/webhook-subscriptions/reconcile",
            post(post_zoom_webhook_subscription_reconcile),
        )
        .route(
            "/api/v1/integrations/zoom/webhook-subscriptions/remove",
            post(post_zoom_webhook_subscription_remove),
        )
        .route(
            "/api/v1/integrations/zoom/accounts/{account_id}/runtime/status",
            get(get_zoom_account_runtime_status),
        )
        .route(
            "/api/v1/integrations/zoom/accounts/{account_id}/recording-imports",
            get(get_zoom_recording_imports),
        )
        .route(
            "/api/v1/integrations/zoom/accounts/{account_id}/recording-imports/{attachment_id}/remove",
            post(post_zoom_recording_import_remove),
        )
        .route(
            "/api/v1/integrations/zoom/accounts/{account_id}/audit-events",
            get(get_zoom_audit_events),
        )
        .route(
            "/api/v1/integrations/zoom/accounts/{account_id}/retention/prune",
            post(post_zoom_retention_cleanup),
        )
        .route(
            "/api/v1/integrations/zoom/runtime/status",
            get(get_zoom_runtime_status),
        )
        .route(
            "/api/v1/integrations/zoom/runtime/start",
            post(post_zoom_runtime_start),
        )
        .route(
            "/api/v1/integrations/zoom/runtime/stop",
            post(post_zoom_runtime_stop),
        )
        .route(
            "/api/v1/integrations/zoom/runtime/remove",
            post(post_zoom_runtime_remove),
        )
        .route(
            "/api/v1/integrations/zoom/runtime-bridge/meetings",
            post(post_zoom_runtime_bridge_meeting),
        )
        .route(
            "/api/v1/integrations/zoom/runtime-bridge/recordings",
            post(post_zoom_runtime_bridge_recording),
        )
        .route(
            "/api/v1/integrations/zoom/runtime-bridge/transcripts",
            post(post_zoom_runtime_bridge_transcript),
        )
        .route(
            "/api/v1/integrations/zoom/runtime-bridge/transcript-files",
            post(post_zoom_runtime_bridge_transcript_file),
        )
        .route(
            "/api/v1/integrations/zoom/runtime-bridge/webhooks",
            post(post_zoom_runtime_bridge_webhook),
        )
        .route(
            "/api/v1/integrations/zoom/fixtures/accounts",
            post(post_zoom_fixture_account),
        )
        .route(
            "/api/v1/integrations/whatsapp/accounts/{account_id}/capabilities",
            get(get_whatsapp_account_capabilities),
        )
        .route(
            "/api/v1/integrations/whatsapp/accounts",
            get(get_whatsapp_accounts).post(post_whatsapp_account),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/status",
            get(get_whatsapp_runtime_status),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/start",
            post(post_whatsapp_runtime_start),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/stop",
            post(post_whatsapp_runtime_stop),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/revoke",
            post(post_whatsapp_runtime_revoke),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/relink",
            post(post_whatsapp_runtime_relink),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/rotate",
            post(post_whatsapp_runtime_rotate),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/remove",
            post(post_whatsapp_runtime_remove),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime/health",
            get(get_whatsapp_runtime_health),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/messages",
            post(post_whatsapp_runtime_bridge_message),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
            post(post_whatsapp_runtime_bridge_message_update),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
            post(post_whatsapp_runtime_bridge_message_delete),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
            post(post_whatsapp_runtime_bridge_receipt),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks",
            get(get_whatsapp_runtime_bridge_business_cloud_webhook)
                .post(post_whatsapp_runtime_bridge_business_cloud_webhook),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest",
            get(get_whatsapp_runtime_bridge_business_cloud_proxy_manifest),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
            post(post_whatsapp_runtime_bridge_dialog),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/participants",
            post(post_whatsapp_runtime_bridge_participant),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/reactions",
            post(post_whatsapp_runtime_bridge_reaction),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/media",
            post(post_whatsapp_runtime_bridge_media),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
            post(post_whatsapp_runtime_bridge_media_lifecycle),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
            post(post_whatsapp_runtime_bridge_status),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/status-views",
            post(post_whatsapp_runtime_bridge_status_view),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/status-deletes",
            post(post_whatsapp_runtime_bridge_status_delete),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/presence",
            post(post_whatsapp_runtime_bridge_presence),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/calls",
            post(post_whatsapp_runtime_bridge_call),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            post(post_whatsapp_runtime_bridge_runtime_event),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
            post(post_whatsapp_runtime_bridge_sync_lifecycle),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            post(post_whatsapp_runtime_bridge_claim_commands),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/commands/{command_id}/failed",
            post(post_whatsapp_runtime_bridge_command_failed),
        )
        .route(
            "/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized",
            post(post_whatsapp_runtime_bridge_authorized_session),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/chats",
            post(post_whatsapp_sync_chats),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/history",
            post(post_whatsapp_sync_history),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/conversations/{provider_chat_id}/members",
            post(post_whatsapp_sync_members),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/statuses",
            post(post_whatsapp_sync_statuses),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/presence",
            post(post_whatsapp_sync_presence),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/calls",
            post(post_whatsapp_sync_calls),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/contacts",
            post(post_whatsapp_sync_contacts),
        )
        .route(
            "/api/v1/integrations/whatsapp/provider-sync/media",
            post(post_whatsapp_sync_media),
        )
        .route(
            "/api/v1/integrations/whatsapp/login/qr/start",
            post(post_whatsapp_qr_link_start),
        )
        .route(
            "/api/v1/integrations/whatsapp/login
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/router/routes/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/mod.rs`
- Size bytes / Размер в байтах: `1240`
- Included characters / Включено символов: `1240`
- Truncated / Обрезано: `no`

```rust
mod ai;
mod audit_events;
mod calendar;
mod communications;
mod email_accounts;
mod knowledge;
mod messaging;
mod organizations;
mod persons;
mod public;
mod review;
mod settings;
mod signal_hub;
mod status_vault;
mod support;
mod tasks;

use support::*;

pub(super) fn protected_routes(api_secret: String) -> Router<AppState> {
    let routes = Router::<AppState>::new();
    let routes = status_vault::add_routes(routes);
    let routes = communications::add_routes(routes);
    let routes = knowledge::add_routes(routes);
    let routes = persons::add_routes(routes);
    let routes = calendar::add_routes(routes);
    let routes = organizations::add_routes(routes);
    let routes = tasks::add_routes(routes);
    let routes = review::add_routes(routes);
    let routes = settings::add_routes(routes);
    let routes = signal_hub::add_routes(routes);
    let routes = ai::add_routes(routes);
    let routes = messaging::add_routes(routes);
    let routes = email_accounts::add_routes(routes);
    let routes = audit_events::add_routes(routes);

    routes.route_layer(middleware::from_fn_with_state(
        api_secret,
        guard::require_secret,
    ))
}

pub(super) fn public_routes() -> Router<AppState> {
    public::routes()
}
```

### `backend/src/app/router/routes/organizations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/organizations.rs`
- Size bytes / Размер в байтах: `3385`
- Included characters / Включено символов: `3385`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/organizations",
            get(get_organizations).post(post_organization),
        )
        .route("/api/v1/organizations/search", get(get_organization_search))
        .route(
            "/api/v1/organizations/{org_id}",
            get(get_organization).put(put_organization),
        )
        .route(
            "/api/v1/organizations/{org_id}/archive",
            post(post_organization_archive),
        )
        .route(
            "/api/v1/organizations/{org_id}/identities",
            get(get_org_identities).post(post_org_identity),
        )
        .route(
            "/api/v1/organizations/{org_id}/aliases",
            get(get_org_aliases).post(post_org_alias),
        )
        .route(
            "/api/v1/organizations/{org_id}/domains",
            get(get_org_domains),
        )
        .route(
            "/api/v1/organizations/{org_id}/departments",
            get(get_org_departments).post(post_org_department),
        )
        .route(
            "/api/v1/organizations/{org_id}/contacts",
            get(get_org_contacts).post(post_org_contact_link),
        )
        .route(
            "/api/v1/organizations/{org_id}/related",
            get(get_org_related),
        )
        .route(
            "/api/v1/organizations/{org_id}/timeline",
            get(get_org_timeline),
        )
        .route(
            "/api/v1/organizations/{org_id}/portals",
            get(get_org_portals),
        )
        .route(
            "/api/v1/organizations/{org_id}/procedures",
            get(get_org_procedures),
        )
        .route(
            "/api/v1/organizations/{org_id}/playbooks",
            get(get_org_playbooks),
        )
        .route(
            "/api/v1/organizations/{org_id}/templates",
            get(get_org_templates),
        )
        .route(
            "/api/v1/organizations/{org_id}/financial",
            get(get_org_financial),
        )
        .route(
            "/api/v1/organizations/{org_id}/contracts",
            get(get_org_contracts),
        )
        .route(
            "/api/v1/organizations/{org_id}/compliance",
            get(get_org_compliance),
        )
        .route(
            "/api/v1/organizations/{org_id}/services",
            get(get_org_services),
        )
        .route(
            "/api/v1/organizations/{org_id}/products",
            get(get_org_products),
        )
        .route(
            "/api/v1/organizations/{org_id}/enrichment",
            get(get_org_enrichment),
        )
        .route(
            "/api/v1/organizations/{org_id}/enrichment/{rid}/apply",
            post(post_org_enrich_apply),
        )
        .route("/api/v1/organizations/{org_id}/risks", get(get_org_risks))
        .route("/api/v1/organizations/{org_id}/health", get(get_org_health))
        .route(
            "/api/v1/organizations/{org_id}/watchlist",
            post(post_org_watchlist_toggle),
        )
        .route(
            "/api/v1/organizations/{org_id}/dossier",
            get(get_org_dossier),
        )
        .route("/api/v1/organizations/{org_id}/brief", get(get_org_brief))
        .route(
            "/api/v1/organizations/{org_id}/context-pack",
            get(get_org_context_pack),
        )
}
```

### `backend/src/app/router/routes/persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/persons.rs`
- Size bytes / Размер в байтах: `6175`
- Included characters / Включено символов: `6073`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        // ── Legacy /api/v1/persons routes ─────────────────────────────────
        .route("/api/v1/persons", get(get_persons))
        .route("/api/v1/persons/{person_id}", get(get_person))
        .route(
            "/api/v1/persons/owner",
            get(get_owner_persona).put(put_owner_persona),
        )
        .route("/api/v1/persons/search", get(get_person_search))
        .route("/api/v1/persons/health", get(get_persons_health))
        .route("/api/v1/persons/watchlist", get(get_persons_watchlist))
        .route(
            "/api/v1/persons/{person_id}/fingerprint",
            post(post_person_fingerprint),
        )
        .route(
            "/api/v1/persons/{person_id}/favorite",
            post(post_person_favorite),
        )
        .route("/api/v1/persons/{person_id}/notes", put(put_person_notes))
        // ── ADR-0084: /api/v1/personas natively-named routes ──────────────
        .route("/api/v1/personas", get(get_personas))
        .route(
            "/api/v1/personas/{persona_id}",
            get(get_persona).put(put_persona),
        )
        .route(
            "/api/v1/personas/owner",
            get(get_owner_persona).put(put_owner_persona),
        )
        .route("/api/v1/personas/search", get(get_person_search))
        .route("/api/v1/personas/health", get(get_persons_health))
        .route("/api/v1/personas/watchlist", get(get_persons_watchlist))
        .route(
            "/api/v1/personas/{persona_id}/fingerprint",
            post(post_person_fingerprint),
        )
        .route(
            "/api/v1/personas/{persona_id}/favorite",
            post(post_person_favorite),
        )
        .route("/api/v1/personas/{persona_id}/notes", put(put_person_notes))
        .route("/api/v1/identity-candidates", get(get_identity_candidates))
        .route(
            "/api/v1/identity-traces",
            get(get_identity_traces).post(post_identity_trace),
        )
        .route(
            "/api/v1/identity-traces/{identity_id}/assignment",
            put(put_identity_trace_assignment),
        )
        .route(
            "/api/v1/identity-candidates/{identity_candidate_id}/review",
            put(put_identity_candidate_review),
        )
        .route(
            "/api/v1/persons/{person_id}/identity",
            get(get_person_identity),
        )
        .route(
            "/api/v1/persons/{person_id}/identities",
            get(get_person_identities),
        )
        .route(
            "/api/v1/persons/{person_id}/identities",
            post(post_person_identity),
        )
        .route(
            "/api/v1/persons/{person_id}/identities/{identity_id}",
            delete(delete_person_identity),
        )
        .route("/api/v1/persons/{person_id}/roles", get(get_person_roles))
        .route("/api/v1/persons/{person_id}/roles", post(post_person_role))
        .route(
            "/api/v1/persons/{person_id}/roles/{role}",
            delete(delete_person_role),
        )
        .route(
            "/api/v1/persons/{person_id}/personas",
            get(get_person_personas),
        )
        .route(
            "/api/v1/persons/{person_id}/personas",
            post(post_person_persona),
        )
        .route(
            "/api/v1/persons/{person_id}/personas/{persona_id}",
            delete(delete_person_persona),
        )
        .route(
            "/api/v1/persons/{person_id}/facts",
            get(get_person_facts).post(post_person_fact),
        )
        .route(
            "/api/v1/persons/{person_id}/memory-cards",
            get(get_person_memory_cards).post(post_person_memory_card),
        )
        .route(
            "/api/v1/persons/{person_id}/preferences",
            get(get_person_preferences).post(post_person_preference),
        )
        .route(
            "/api/v1/persons/{person_id}/timeline",
            get(get_person_timeline).post(post_relationship_event),
        )
        .route(
            "/api/v1/persons/{person_id}/snapshots",
            get(get_person_snapshots),
        )
        .route(
            "/api/v1/persons/{person_id}/history-diff",
            get(get_person_history_diff),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment",
            get(get_person_enrichment),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment/{result_id}/apply",
            post(post_person_enrichment_apply),
        )
        .route(
            "/api/v1/persons/{person_id}/enrichment/{result_id}/reject",
            post(post_person_enrichment_reject),
        )
        .route(
            "/api/v1/persons/{person_id}/expertise",
            get(get_person_expertise),
        )
        .route(
            "/api/v1/persons/search/expertise",
            get(get_person_expertise_search),
        )
        .route(
            "/api/v1/persons/{person_id}/promises",
            get(get_person_promises),
        )
        .route("/api/v1/persons/{person_id}/risks", get(get_person_risks))
        .route(
            "/api/v1/persons/{person_id}/investigate",
            post(post_person_investigate),
        )
        .route(
            "/api/v1/persons/{person_id}/dossier",
            get(get_person_dossier),
        )
        .route(
            "/api/v1/persons/{person_id}/dossier/review",
            put(put_person_dossier_review),
        )
        .route(
            "/api/v1/persons/{person_id}/meeting-prep",
            get(get_person_meeting_prep),
        )
        .route(
            "/api/v1/persons/{person_id}/analytics",
            get(get_person_analytics),
        )
        .route(
            "/api/v1/persons/{person_id}/export",
            get(get_person_export_handler),
        )
        .route("/api/v1/persons/{person_id}/health", get(get_person_health))
        .route(
            "/api/v1/persons/{person_id}/watchlist",
            post(post_person_watchlist_toggle),
        )
}
```

### `backend/src/app/router/routes/public.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/public.rs`
- Size bytes / Размер в байтах: `505`
- Included characters / Включено символов: `505`
- Truncated / Обрезано: `no`

```rust
use super::super::{healthz, readyz};
use super::support::*;

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/api/v1/integrations/mail/accounts/gmail/oauth/callback",
            get(get_gmail_oauth_callback),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/remote-image",
            get(get_v1_communication_message_remote_image),
        )
}
```

### `backend/src/app/router/routes/review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/review.rs`
- Size bytes / Размер в байтах: `1679`
- Included characters / Включено символов: `1679`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/review/items", get(get_v1_review_items))
        .route("/api/v1/review/items", post(post_v1_review_items))
        .route(
            "/api/v1/review/items/{review_item_id}/approve",
            post(post_v1_review_item_approve),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/dismiss",
            post(post_v1_review_item_dismiss),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/take",
            post(post_v1_review_item_take),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/archive",
            post(post_v1_review_item_archive),
        )
        .route(
            "/api/v1/review/items/{review_item_id}/promote",
            post(post_v1_review_item_promote),
        )
        .route("/api/v1/obligations", get(get_v1_obligations))
        .route(
            "/api/v1/obligations/{obligation_id}/review",
            put(put_v1_obligation_review),
        )
        .route("/api/v1/decisions", get(get_v1_decisions))
        .route(
            "/api/v1/decisions/{decision_id}/review",
            put(put_v1_decision_review),
        )
        .route("/api/v1/relationships", get(get_v1_relationships))
        .route(
            "/api/v1/relationships/{relationship_id}/review",
            put(put_v1_relationship_review),
        )
        .route("/api/v1/contradictions", get(get_v1_contradictions))
        .route(
            "/api/v1/contradictions/{observation_id}/review",
            put(put_v1_contradiction_review),
        )
}
```

### `backend/src/app/router/routes/settings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/settings.rs`
- Size bytes / Размер в байтах: `409`
- Included characters / Включено символов: `409`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/settings", get(get_application_settings))
        .route(
            "/api/v1/settings/accounts",
            get(get_application_settings_accounts),
        )
        .route(
            "/api/v1/settings/{setting_key}",
            put(put_application_setting),
        )
}
```

### `backend/src/app/router/routes/signal_hub.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/signal_hub.rs`
- Size bytes / Размер в байтах: `2953`
- Included characters / Включено символов: `2953`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/signal-hub/sources", get(get_signal_hub_sources))
        .route(
            "/api/v1/signal-hub/sources/{source_code}",
            get(get_signal_hub_source),
        )
        .route(
            "/api/v1/signal-hub/capabilities",
            get(get_signal_hub_capabilities),
        )
        .route(
            "/api/v1/signal-hub/sources/{source_code}/enable",
            post(post_signal_hub_enable_source),
        )
        .route(
            "/api/v1/signal-hub/sources/{source_code}/disable",
            post(post_signal_hub_disable_source),
        )
        .route(
            "/api/v1/signal-hub/profiles",
            get(get_signal_hub_profiles).post(post_signal_hub_profile),
        )
        .route(
            "/api/v1/signal-hub/profiles/{profile_code}/apply",
            post(post_signal_hub_apply_profile),
        )
        .route(
            "/api/v1/signal-hub/profiles/{profile_code}",
            patch(patch_signal_hub_profile).delete(delete_signal_hub_profile),
        )
        .route(
            "/api/v1/signal-hub/connections",
            get(get_signal_hub_connections).post(post_signal_hub_connection),
        )
        .route(
            "/api/v1/signal-hub/connections/{connection_id}",
            patch(patch_signal_hub_connection).delete(delete_signal_hub_connection),
        )
        .route(
            "/api/v1/signal-hub/runtimes",
            get(get_signal_hub_runtime_states).post(post_signal_hub_runtime_state),
        )
        .route(
            "/api/v1/signal-hub/health",
            get(get_signal_hub_health).post(post_signal_hub_health_check),
        )
        .route(
            "/api/v1/signal-hub/replay",
            get(get_signal_hub_replay_requests).post(post_signal_hub_replay_request),
        )
        .route(
            "/api/v1/signal-hub/policies",
            get(get_signal_hub_policies).post(post_signal_hub_policy),
        )
        .route(
            "/api/v1/signal-hub/signals/mute",
            post(post_signal_hub_mute_signals),
        )
        .route(
            "/api/v1/signal-hub/signals/unmute",
            post(post_signal_hub_unmute_signals),
        )
        .route(
            "/api/v1/signal-hub/signals/pause",
            post(post_signal_hub_pause_signals),
        )
        .route(
            "/api/v1/signal-hub/signals/resume",
            post(post_signal_hub_resume_signals),
        )
        .route(
            "/api/v1/signal-hub/fixtures/system/restore",
            post(post_signal_hub_restore_system_fixture),
        )
        .route(
            "/api/v1/signal-hub/fixtures",
            get(get_signal_hub_fixture_sources),
        )
        .route(
            "/api/v1/signal-hub/fixtures/{fixture_id}/emit",
            post(post_signal_hub_emit_fixture_signal),
        )
}
```

### `backend/src/app/router/routes/status_vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/status_vault.rs`
- Size bytes / Размер в байтах: `722`
- Included characters / Включено символов: `722`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/status", get(get_v1_status))
        .route("/api/v1/vault/status", get(get_v1_vault_status))
        .route(
            "/api/v1/vault/collect-entropy",
            post(post_v1_vault_collect_entropy),
        )
        .route("/api/v1/vault/create", post(post_v1_vault_create))
        .route("/api/v1/vault/unlock", post(post_v1_vault_unlock))
        .route(
            "/api/v1/vault/recovery/export",
            post(post_v1_vault_recovery_export),
        )
        .route(
            "/api/v1/vault/recovery/import",
            post(post_v1_vault_recovery_import),
        )
}
```

### `backend/src/app/router/routes/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/support.rs`
- Size bytes / Размер в байтах: `1372`
- Included characters / Включено символов: `1372`
- Truncated / Обрезано: `no`

```rust
pub(super) use axum::routing::{delete, get, patch, post, put};
pub(super) use axum::{Router, middleware};

pub(super) use crate::ai::api::*;
pub(super) use crate::app::AppState;
pub(super) use crate::app::api_support::*;
pub(super) use crate::app::guard;
pub(super) use crate::app::handlers::automation::*;
pub(super) use crate::app::handlers::calendar::*;
pub(super) use crate::app::handlers::calls::*;
pub(super) use crate::app::handlers::communications::*;
pub(super) use crate::app::handlers::consistency::*;
pub(super) use crate::app::handlers::decisions::*;
pub(super) use crate::app::handlers::documents::*;
pub(super) use crate::app::handlers::events::*;
pub(super) use crate::app::handlers::graph::*;
pub(super) use crate::app::handlers::obligations::*;
pub(super) use crate::app::handlers::organizations::*;
pub(super) use crate::app::handlers::persons::*;
pub(super) use crate::app::handlers::projects::*;
pub(super) use crate::app::handlers::relationships::*;
pub(super) use crate::app::handlers::review::*;
pub(super) use crate::app::handlers::settings::*;
pub(super) use crate::app::handlers::signal_hub::*;
pub(super) use crate::app::handlers::tasks::*;
pub(super) use crate::app::handlers::telegram::*;
pub(super) use crate::app::handlers::whatsapp::*;
pub(super) use crate::app::handlers::yandex_telemost::*;
pub(super) use crate::app::handlers::zoom::*;
```

### `backend/src/app/router/routes/tasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/router/routes/tasks.rs`
- Size bytes / Размер в байтах: `2282`
- Included characters / Включено символов: `2282`
- Truncated / Обрезано: `no`

```rust
use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/tasks", get(get_tasks).post(post_task))
        .route("/api/v1/tasks/{task_id}", get(get_task).put(put_task))
        .route("/api/v1/tasks/{task_id}/archive", post(post_task_archive))
        .route("/api/v1/tasks/{task_id}/status", post(post_task_status))
        .route(
            "/api/v1/tasks/{task_id}/context-pack",
            get(get_task_context_pack).post(post_task_context_pack),
        )
        .route(
            "/api/v1/tasks/{task_id}/evidence",
            get(get_task_evidence).post(post_task_evidence),
        )
        .route(
            "/api/v1/tasks/{task_id}/relations",
            get(get_task_relations).post(post_task_relation),
        )
        .route(
            "/api/v1/tasks/{task_id}/checklist",
            get(get_task_checklist).post(post_task_checklist),
        )
        .route(
            "/api/v1/tasks/{task_id}/subtasks",
            get(get_task_subtasks).post(post_task_subtask),
        )
        .route("/api/v1/tasks/{task_id}/analyze", post(post_task_analyze))
        .route("/api/v1/tasks/{task_id}/export", get(get_task_export))
        .route("/api/v1/tasks/{task_id}/external", get(get_task_external))
        .route(
            "/api/v1/tasks/providers",
            get(get_task_providers).post(post_task_provider),
        )
        .route("/api/v1/tasks/brain", post(post_task_brain))
        .route("/api/v1/tasks/search", get(get_task_search))
        .route("/api/v1/tasks/daily-brief", get(get_task_daily_brief))
        .route(
            "/api/v1/tasks/rules",
            get(get_task_rules).post(post_task_rule),
        )
        .route("/api/v1/tasks/rules/{rule_id}", delete(delete_task_rule))
        .route("/api/v1/tasks/templates", get(get_task_templates))
        .route("/api/v1/tasks/watchtower", get(get_task_watchtower))
        .route("/api/v1/tasks/health", get(get_task_health))
        .route("/api/v1/tasks/analytics", get(get_task_analytics))
        .route("/api/v1/task-candidates", get(get_task_candidates))
        .route(
            "/api/v1/task-candidates/{task_candidate_id}/review",
            put(put_task_candidate_review),
        )
}
```

### `backend/src/app/signal_hub_support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/signal_hub_support.rs`
- Size bytes / Размер в байтах: `16449`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::{Map, Value, json};
use sqlx::postgres::PgPool;

use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus;
use crate::domains::communications::core::{CommunicationProviderAccountStore, ProviderAccount};
use crate::domains::signal_hub::{
    SignalHealth, SignalHealthCheckRequest, SignalHealthSnapshotWrite, SignalHubConnectionService,
    SignalHubHealthService, SignalHubStore,
};
use crate::integrations::ai_runtime::AiRuntimeClient;
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::integrations::omniroute::client::{OmniRouteClient, OmniRouteClientConfig};
use crate::platform::communications::CommunicationProviderKind;
use crate::platform::config::{AiRuntimeProvider, AppConfig};
use crate::platform::events::EventStore;
use crate::platform::settings::{AiRuntimeSettings, ApplicationSettingsStore};

pub(crate) async fn provider_account_or_not_found(
    state: &AppState,
    account_id: &str,
) -> Result<ProviderAccount, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationProviderAccountStore::new(pool)
        .get(account_id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn sync_provider_account_signal_connection(
    state: &AppState,
    account: &ProviderAccount,
    secret_ref: Option<&str>,
) -> Result<(), ApiError> {
    let status = provider_account_signal_status(account);
    sync_provider_account_signal_connection_with_status(state, account, status, secret_ref).await
}

pub(crate) async fn sync_provider_account_signal_connection_with_status(
    state: &AppState,
    account: &ProviderAccount,
    status: &str,
    secret_ref: Option<&str>,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let signal_store = SignalHubStore::new(pool.clone());
    let connection_service =
        SignalHubConnectionService::new(signal_store.clone(), EventStore::new(pool));
    signal_store.restore_system_sources().await?;
    let source_code = provider_signal_source_code(account.provider_kind);
    let settings = merged_provider_connection_settings(
        signal_store
            .find_connection_by_account(source_code, &account.account_id)
            .await?
            .as_ref()
            .map(|connection| &connection.settings),
        account,
    );
    connection_service
        .upsert_account_connection(
            source_code,
            &account.account_id,
            &account.display_name,
            status,
            settings,
            secret_ref.map(str::to_owned),
        )
        .await?;
    Ok(())
}

pub(crate) async fn remove_provider_account_signal_connection(
    state: &AppState,
    account: &ProviderAccount,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let source_code = provider_signal_source_code(account.provider_kind);
    SignalHubConnectionService::new(SignalHubStore::new(pool.clone()), EventStore::new(pool))
        .remove_account_connection(source_code, &account.account_id)
        .await?;
    Ok(())
}

pub(crate) async fn sync_whatsapp_runtime_signal_connection(
    state: &AppState,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    sync_whatsapp_runtime_signal_connection_for_pool(
        &pool,
        account,
        status,
        status.session_secret_ref.clone(),
    )
    .await
    .map_err(ApiError::from)
}

pub(crate) async fn sync_whatsapp_runtime_signal_connection_for_pool(
    pool: &PgPool,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
    secret_ref: Option<String>,
) -> Result<(), crate::domains::signal_hub::SignalHubError> {
    let signal_store = SignalHubStore::new(pool.clone());
    let connection_service =
        SignalHubConnectionService::new(signal_store.clone(), EventStore::new(pool.clone()));
    signal_store.restore_system_sources().await?;
    let source_code = provider_signal_source_code(account.provider_kind);
    let settings = merged_whatsapp_runtime_connection_settings(
        signal_store
            .find_connection_by_account(source_code, &account.account_id)
            .await?
            .as_ref()
            .map(|connection| &connection.settings),
        account,
        status,
    );
    connection_service
        .upsert_account_connection(
            source_code,
            &account.account_id,
            &account.display_name,
            whatsapp_runtime_signal_status(status),
            settings,
            secret_ref,
        )
        .await?;
    Ok(())
}

pub(crate) async fn run_signal_hub_health_check(
    config: &AppConfig,
    pool: PgPool,
    request: &SignalHealthCheckRequest,
) -> Result<SignalHealth, crate::domains::signal_hub::SignalHubError> {
    let service = SignalHubHealthService::new(
        SignalHubStore::new(pool.clone()),
        EventStore::new(pool.clone()),
    );

    if request.source_code == "ai" && request.connection_id.is_none() {
        let runtime_state =
            crate::platform::events::source_runtime_state_from_policies(&pool, "ai").await?;
        let snapshot = match runtime_state {
            "stopped" => SignalHealthSnapshotWrite {
                level: "disabled".to_owned(),
                summary: "AI source is disabled by Signal Hub policy".to_owned(),
                last_ok_at: None,
                last_failure_at: Some(chrono::Utc::now()),
                failure_count: 1,
                consecutive_failure_count: 1,
                next_retry_at: None,
                evidence: json!({
                    "source_code": "ai",
                    "runtime_state": runtime_state,
                    "health_origin": "signal_hub_policy"
                }),
            },
            "paused" | "muted" => SignalHealthSnapshotWrite {
                level: "degraded".to_owned(),
                summary: format!("AI source is {runtime_state} by Signal Hub policy"),
                last_ok_at: None,
                last_failure_at: Some(chrono::Utc::now()),
                failure_count: 1,
                consecutive_failure_count: 1,
                next_retry_at: None,
                evidence: json!({
                    "source_code": "ai",
                    "runtime_state": runtime_state,
                    "health_origin": "signal_hub_policy"
                }),
            },
            _ => ai_runtime_health_snapshot(config, &pool).await?,
        };

        return service.record_snapshot(request, snapshot).await;
    }

    service.run_health_check(request).await
}

fn provider_signal_source_code(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::Gmail
        | CommunicationProviderKind::Icloud
        | CommunicationProviderKind::Imap => "mail",
        CommunicationProviderKind::TelegramUser | CommunicationProviderKind::TelegramBot => {
            "telegram"
        }
        CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud => "whatsapp",
        CommunicationProviderKind::ZoomUser | CommunicationProviderKind::ZoomServerToServer => {
            "zoom"
        }
        CommunicationProviderKind::YandexTelemostUser => "yandex_telemost",
    }
}

async fn ai_runtime_health_snapshot(
    config: &AppConfig,
    pool: &PgPool,
) -> Result<SignalHealthSnapshotWrite, crate::domains::signal_hub::SignalHubError> {
    let settings = ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(config)
        .await?;
    let runtime = ai_runtime_client_from_settings(config, &settings);
    let runtime_name = runtime
        .as_ref()
        .map(AiRuntimeClient::runtime_name)
        .unwrap_or(match settings.provider {
            AiRuntimeProvider::Ollama => "ollama",
            AiRuntimeProvider::OmniRoute => "omniroute",
        });

    let version = match runtime.as_ref() {
        Some(runtime) => runtime.version().await,
        None => Ok(None),
    };
    let models = match runtime.as_ref() {
        Some(runtime) => runtime.models().await,
        None => Ok(vec![]),
    };
    let chat_model_available = models
        .as_ref()
        .map(|items| items.iter().any(|item| item == &settings.chat_model))
        .unwrap_or(false);
    let embedding_model_available = models
        .as_ref()
        .map(|items| items.iter().any(|item| item == &settings.embedding_model))
        .unwrap_or(false);
    let healthy = version.is_ok()
        && models.is_ok()
        && chat_model_available
        && embedding_model_available
        && runtime.is_some();

    let runtime_error = runtime
        .is_none()
        .then_some("runtime client could not be initialized".to_owned())
        .or_else(|| version.as_ref().err().map(ToString::to_string))
        .or_else(|| models.as_ref().err().map(ToString::to_string));

    Ok(SignalHealthSnapshotWrite {
        level: if healthy {
            "healthy".to_owned()
        } else {
            "degraded".to_owned()
        },
        summary: if healthy {
            format!("AI runtime {runtime_name} is healthy")
        } else {
            format!("AI runtime {runtime_name} requires attention")
        },
        last_ok_at: healthy.then(chrono::Utc::now),
        last_failure_at: (!healthy).then(chrono::Utc::now),
        failure_count: if healthy { 0 } else { 1 },
        consecutive_failure_count: if healthy { 0 } else { 1 },
        next_retry_at: (!healthy).then(|| chrono::Utc::now() + chrono::Duration::minutes(5)),
        evidence: json!({
            "source_code": "ai",
            "health_origin": "ai_runtime_status",
            "runtime": runtime_name,
            "provider": settings.provider.as_str(),
            "base_url": settings.base_url,
            "version": version.ok().flatten(),
            "chat_model": settings.chat_model,
            "embedding_model": settings.embedding_model,
            "chat_model_available": chat_model_available,
            "embedding_model_available": embedding_model_available,
            "runtime_error": runtime_error,
        }),
    })
}

fn ai_runtime_client_from_settings(
    config: &AppConfig,
    settings: &AiRuntimeSettings,
) -> Option<AiRuntimeClient> {
    match settings.provider {
        AiRuntimeProvider::Ollama => OllamaClient::new(
            OllamaClientConfig::new(
                &settings.base_url,
                &settings.chat_model,
                &settings.embedding_model,
            )
            .with_timeout_seconds(settings.timeout_seconds),
        )
        .ok()
        .map(AiRuntimeClient::Ollama),
        AiRuntimeProvider::OmniRoute => config.omniroute_api_key().cloned().and_then(|api_key| {
            OmniRouteClient::new(
                OmniRouteClientConfig::new(
                    &settings.base_url,
                    &settings.chat_model,
                    &settings.embedding_model,
                    api_key,
                )
                .with_timeout_seconds(settings.timeout_seconds),
            )
            .ok()
            .map(AiRuntimeClient::OmniRoute)
        }),
    }
}

fn provider_account_signal_status(account: &ProviderAccount) -> &'static str {
    match account.provider_kind {
        CommunicationProviderKind::Gmail
        | CommunicationProviderKind::Icloud
        | CommunicationProviderKind::Imap => {
            if account.config.get("auth_state").and_then(Value::as_str) == Some("logged_out") {
                "disconnected"
            } else {
                "connected"

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/state.rs`
- Size bytes / Размер в байтах: `1063`
- Included characters / Включено символов: `1063`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::integrations::mail::accounts::GmailOAuthPendingGrant;
use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::integrations::telegram::tdjson::PendingQrLoginMap;
use crate::integrations::zoom::client::ZoomOAuthPendingGrant;
use crate::platform::config::AppConfig;
use crate::platform::events::EventBus;
use crate::platform::storage::Database;
use crate::vault::HostVault;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: AppConfig,
    pub(crate) database: Database,
    pub(crate) vault: HostVault,
    pub(crate) account_setup: AccountSetupState,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: EventBus,
}

#[derive(Clone, Default)]
pub(crate) struct AccountSetupState {
    pub(crate) pending_gmail_oauth: Arc<Mutex<HashMap<String, GmailOAuthPendingGrant>>>,
    pub(crate) pending_zoom_oauth: Arc<Mutex<HashMap<String, ZoomOAuthPendingGrant>>>,
    pub(crate) pending_telegram_qr_login: PendingQrLoginMap,
}
```

### `backend/src/app/vault_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation.rs`
- Size bytes / Размер в байтах: `206`
- Included characters / Включено символов: `206`
- Truncated / Обрезано: `no`

```rust
mod calendar_restore;
mod errors;
mod lifecycle;
mod manifest_enrichment;
mod metadata;
mod provider_recovery;
mod service;
mod summary;

pub(crate) use lifecycle::spawn_host_vault_manifest_reconciliation;
```

### `backend/src/app/vault_reconciliation/calendar_restore.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/calendar_restore.rs`
- Size bytes / Размер в байтах: `1798`
- Included characters / Включено символов: `1798`
- Truncated / Обрезано: `no`

```rust
use crate::domains::calendar::events::CalendarAccountStore;
use crate::domains::communications::core::{EmailProviderKind, ProviderAccountSecretPurpose};

use super::errors::HostVaultReconciliationError;
use super::provider_recovery::RecoverableProviderSecret;

pub(super) async fn restore_linked_calendar_account(
    calendar_store: &CalendarAccountStore,
    secret: &RecoverableProviderSecret,
) -> Result<bool, HostVaultReconciliationError> {
    match secret.provider_kind {
        EmailProviderKind::Gmail => {
            let calendar_account_id = format!("google-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .restore_google_workspace_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        EmailProviderKind::Icloud => {
            if secret.secret_purpose != ProviderAccountSecretPurpose::ImapPassword {
                return Ok(false);
            }
            let calendar_account_id = format!("icloud-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .restore_apple_icloud_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
```
