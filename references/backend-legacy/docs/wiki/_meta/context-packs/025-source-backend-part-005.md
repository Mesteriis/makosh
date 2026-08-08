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

- Chunk ID / ID чанка: `025-source-backend-part-005`
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

### `backend/src/app/api_support/whatsapp_capability_catalog.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/api_support/whatsapp_capability_catalog.rs`
- Size bytes / Размер в байтах: `20960`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use crate::application::provider_runtime_contracts::WhatsAppProviderRuntimeShape;

use super::whatsapp_capabilities::{
    WhatsAppActionClass, WhatsAppCapabilityState, WhatsappCapabilityStatus,
};

pub(crate) fn whatsapp_capability_rows() -> Vec<WhatsappCapabilityStatus> {
    vec![
        WhatsappCapabilityStatus::new(
            "runtime.fixture",
            "runtime",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Fixture WhatsApp runtime, append-only evidence ingest and projection are available for CI and local validation.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sessions.manual_state",
            "sessions",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Companion session metadata is stored without raw session secrets in PostgreSQL.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sessions.restore",
            "sessions",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::SecretAccess,
            "Authorized session material can be restored from host vault bindings, but only the fixture/runtime-safe restore path is implemented today.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "auth.qr_link_start",
            "auth",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "QR link lifecycle state and sanitized events exist, but live QR material is not emitted yet.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "auth.pair_code_link_start",
            "auth",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Pair-code lifecycle state and sanitized events exist, but live pair-code material is not emitted yet.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sync.chats",
            "sync",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Projected WhatsApp conversations can be synced through the fixture/runtime-safe control surface.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "sync.history",
            "sync",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Projected WhatsApp message history can be replayed through the shared Communications read model.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.read_projection",
            "messages",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Canonical Communications reads already serve WhatsApp messages, reply refs and forward refs.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "search.messages",
            "search",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Provider-neutral message search already returns WhatsApp projection data.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "search.media",
            "search",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Provider-neutral media search already returns projected WhatsApp attachments.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.send_text",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Durable outbox, audit metadata and provider-observed fixture reconciliation exist, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.reply",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Reply commands use the durable provider outbox and canonical reply refs, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.forward",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Forward commands use the durable provider outbox and canonical forward refs, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.edit",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Observed message-version projection and fixture reconciliation exist, but live edit execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.delete",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::Destructive,
            "Observed tombstones and fixture reconciliation exist, but live delete execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.react",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Reaction outbox rows and provider-observed fixture reconciliation exist, but live reaction execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "messages.unreact",
            "messages",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Reaction removal uses the same durable command/reconciliation path, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "media.upload_send",
            "media",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Media upload/send commands preserve blob metadata and fixture reconciliation, but live transfer execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "media.download",
            "media",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::Read,
            "Media download commands preserve blob/hash contracts and fixture reconciliation, but live transfer execution remains blocked.",
            false,
            false,
        ),
        WhatsappCapabilityStatus::new(
            "media.voice_send",
            "media",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Voice-note sending shares the durable media outbox path, but live upload/execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.join_group",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Group join commands are durable and fixture-reconciled, but live provider execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.leave_group",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Group leave commands are durable and fixture-reconciled, but live provider execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.archive",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.unarchive",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.mute",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.unmute",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.pin",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.unpin",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Dialog state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.mark_read",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Read-state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "conversations.mark_unread",
            "conversations",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Unread-state commands reconcile against observed fixture dialog evidence, but live runtime execution remains blocked.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "status.observe",
            "status",
            WhatsAppCapabilityState::Available,
            WhatsAppActionClass::Read,
            "Fixture status evidence already projects into canonical Communications and Timeline-facing events.",
            false,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "status.publish",
            "status",
            WhatsAppCapabilityState::Degraded,
            WhatsAppActionClass::ProviderWrite,
            "Status publish uses the durable provider outbox and observed fixture reconciliation, but live runtime execution remains blocked.",
            true,
            true,
        ),
        WhatsappCapabilityStatus::new(
            "presence.observe",
            "presence",
            WhatsAppCapabilityState::Available,
            W
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/connectrpc.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/connectrpc.rs`
- Size bytes / Размер в байтах: `768`
- Included characters / Включено символов: `768`
- Truncated / Обрезано: `no`

```rust
mod communications;
mod signal_hub;

use axum::Router;
use axum::middleware;
use connectrpc::Router as ConnectRouter;
use sqlx::postgres::PgPool;

use crate::app::guard;
use crate::app::state::AppState;
use crate::platform::config::AppConfig;

pub(crate) fn protected_routes(
    pool: Option<PgPool>,
    config: AppConfig,
    api_secret: String,
) -> Router<AppState> {
    let connect_router = signal_hub::register(
        communications::register(ConnectRouter::new(), pool.clone(), config.clone()),
        pool,
        config,
    );
    Router::<AppState>::new()
        .fallback_service(connect_router.into_axum_router().into_service())
        .layer(middleware::from_fn_with_state(
            api_secret,
            guard::require_secret,
        ))
}
```

### `backend/src/app/connectrpc/communications.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/connectrpc/communications.rs`
- Size bytes / Размер в байтах: `156976`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, Router as ConnectRouter, ServiceRequest,
    ServiceResult,
};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use crate::app::ApiError;
use crate::app::handlers::communications::{
    WorkflowActionInput as HandlerWorkflowActionInput,
    WorkflowActionKind as HandlerWorkflowActionKind,
    WorkflowActionRequest as HandlerWorkflowActionRequest,
    WorkflowActionResponse as HandlerWorkflowActionResponse,
    WorkflowActionSource as HandlerWorkflowActionSource,
    WorkflowActionStatus as HandlerWorkflowActionStatus,
    WorkflowActionTargetKind as HandlerWorkflowActionTargetKind, execute_workflow_action,
};
use crate::application::communication_send::{
    CommunicationSendDependencies, CommunicationSendError, CommunicationSendRequest, send_email,
};
use crate::contracts::makosh::common::v1::PageResponse;
use crate::contracts::makosh::communications::v1::{
    AddMessageLabelResponse, AiReplyRequest as ProtoAiReplyRequest,
    AiReplyResponse as ProtoAiReplyResponse, AiReplyVariantsRequest as ProtoAiReplyVariantsRequest,
    AiReplyVariantsResponse as ProtoAiReplyVariantsResponse, AnalyzeMessageRequest,
    AnalyzeMessageResponse, ArchiveInspectionEntry as ProtoArchiveInspectionEntry,
    ArchiveInspectionReport as ProtoArchiveInspectionReport,
    AttachmentSearchItem as ProtoAttachmentSearchItem, AttachmentSearchRequest,
    AttachmentSearchResponse, BulkMessageActionRequest as ProtoBulkMessageActionRequest,
    BulkMessageActionResponse as ProtoBulkMessageActionResponse,
    CommunicationArchitectureBlocker as ProtoCommunicationArchitectureBlocker,
    CommunicationDraft as ProtoCommunicationDraft, CommunicationFolder as ProtoCommunicationFolder,
    CommunicationMessage as ProtoCommunicationMessage,
    CommunicationMessageAttachment as ProtoCommunicationMessageAttachment,
    CommunicationOutboxItem as ProtoCommunicationOutboxItem,
    CommunicationPersona as ProtoCommunicationPersona,
    CommunicationSavedSearch as ProtoCommunicationSavedSearch,
    CommunicationSearchResult as ProtoCommunicationSearchResult,
    CommunicationThread as ProtoCommunicationThread, CommunicationsService,
    CommunicationsServiceExt, CopyMessageToFolderRequest, CopyMessageToFolderResponse,
    CreateDraftRequest, CreateDraftResponse, CreateFolderRequest, CreateFolderResponse,
    CreateSavedSearchRequest, CreateSavedSearchResponse, DeleteDraftRequest, DeleteDraftResponse,
    DeleteFolderRequest, DeleteFolderResponse, DeleteMessageFromProviderRequest,
    DeleteMessageFromProviderResponse, DeleteRichTemplateRequest, DeleteRichTemplateResponse,
    DeleteSavedSearchRequest, DeleteSavedSearchResponse, DetectMessageLanguageRequest,
    DetectMessageLanguageResponse, ExplainMessageRequest, ExplainMessageResponse,
    ExtractMessageNotesRequest, ExtractMessageNotesResponse, ExtractMessageTasksRequest,
    ExtractMessageTasksResponse, ExtractedNote as ProtoExtractedNote,
    ExtractedTask as ProtoExtractedTask, FolderMessage as ProtoFolderMessage,
    FolderMessageActionResult as ProtoFolderMessageActionResult,
    GetAttachmentArchiveInspectionRequest, GetAttachmentArchiveInspectionResponse,
    GetAttachmentPreviewRequest, GetAttachmentPreviewResponse, GetMailboxHealthRequest,
    GetMailboxHealthResponse, GetMessageAuthRequest, GetMessageAuthResponse,
    GetMessageExportRequest, GetMessageExportResponse, GetMessageRequest, GetMessageResponse,
    GetMessageSignatureRequest, GetMessageSignatureResponse, GetMessageSmartCcRequest,
    GetMessageSmartCcResponse, ListCommunicationBlockersRequest, ListCommunicationBlockersResponse,
    ListCommunicationPersonasRequest, ListCommunicationPersonasResponse, ListDraftsRequest,
    ListDraftsResponse, ListFolderMessagesRequest, ListFolderMessagesResponse, ListFoldersRequest,
    ListFoldersResponse, ListMessageWorkflowStateCountsRequest,
    ListMessageWorkflowStateCountsResponse, ListMessagesRequest, ListMessagesResponse,
    ListOutboxRequest, ListOutboxResponse, ListRichTemplatesRequest, ListRichTemplatesResponse,
    ListSavedSearchesRequest, ListSavedSearchesResponse, ListSubscriptionsRequest,
    ListSubscriptionsResponse, ListThreadMessagesRequest, ListThreadMessagesResponse,
    ListThreadsRequest, ListThreadsResponse, ListTopSendersRequest, ListTopSendersResponse,
    MailboxHealth as ProtoMailboxHealth, MarkMessageReadRequest, MarkMessageReadResponse,
    MessageAuthReport as ProtoMessageAuthReport, MessageAuthResult as ProtoMessageAuthResult,
    MessageAuthRiskReport as ProtoMessageAuthRiskReport,
    MessageKnowledgeCandidate as ProtoMessageKnowledgeCandidate,
    MessageSummaryContract as ProtoMessageSummaryContract, MessageToggleRequest,
    MoveMessageToFolderRequest, MoveMessageToFolderResponse, RedirectMessageRequest,
    RedirectMessageResponse, RemoveMessageLabelResponse,
    RenderedRichTemplate as ProtoRenderedRichTemplate, RichTemplate as ProtoRichTemplate,
    RichTemplateMailMergePreviewItem as ProtoRichTemplateMailMergePreviewItem,
    RichTemplateMailMergePreviewRequest, RichTemplateMailMergePreviewResponse,
    RichTemplateRenderRequest, RichTemplateRenderResponse, SearchMessagesRequest,
    SearchMessagesResponse, SendMessageRequest, SendMessageResponse,
    SenderStats as ProtoSenderStats, SnoozeMessageRequest, SnoozeMessageResponse,
    SubscriptionSource as ProtoSubscriptionSource, ThreadMessage as ProtoThreadMessage,
    ThreadTranslationItem as ProtoThreadTranslationItem, ToggleMessageImportantResponse,
    ToggleMessageMuteResponse, ToggleMessagePinResponse, TransitionMessageWorkflowStateRequest,
    TransitionMessageWorkflowStateResponse, TranslateAttachmentRequest,
    TranslateAttachmentResponse, TranslateMessageRequest, TranslateMessageResponse,
    TranslateThreadRequest, TranslateThreadResponse, UndoOutboxItemRequest, UndoOutboxItemResponse,
    UpdateFolderRequest, UpdateFolderResponse, UpdateMessageLabelRequest,
    UpdateMessageLocalStateRequest, UpdateMessageLocalStateResponse, UpdateSavedSearchRequest,
    UpdateSavedSearchResponse, UpsertRichTemplateRequest, UpsertRichTemplateResponse,
    WorkflowActionProvenance as ProtoWorkflowActionProvenance,
    WorkflowActionRequest as ProtoWorkflowActionRequest,
    WorkflowActionResponse as ProtoWorkflowActionResponse,
    WorkflowActionTarget as ProtoWorkflowActionTarget,
    WorkflowStateCount as ProtoWorkflowStateCount,
};
use crate::domains::communications::analytics::{
    EmailAnalyticsError, EmailAnalyticsStore, MailboxHealth, SenderStats,
};
use crate::domains::communications::archive_inspection::{
    ArchiveInspectionLimits, ArchiveInspectionReport, inspect_zip_bytes,
};
use crate::domains::communications::attachment_search::{
    AttachmentSearchError, AttachmentSearchPage, AttachmentSearchQuery, AttachmentSearchResult,
    AttachmentSearchStore,
};
use crate::domains::communications::bulk_actions::{
    BulkMessageAction, BulkMessageActionError, BulkMessageActionOutcome, BulkMessageActionStore,
};
use crate::domains::communications::drafts::{
    CommunicationDraft, CommunicationDraftError, CommunicationDraftStore, DraftStatus,
};
use crate::domains::communications::folders::{
    CommunicationFolder, CommunicationFolderError, CommunicationFolderListQuery,
    CommunicationFolderStore, FolderMessage, FolderMessageActionResponse, FolderMessageListQuery,
    NewCommunicationFolder, UpdateCommunicationFolder,
};
use crate::domains::communications::messages::{
    LocalMessageState, MessageProjectionError, MessageProjectionStore, MessageSearchMatchMode,
    ProjectedMessage, ProjectedMessagePageQuery, ProjectedMessageSummary, WorkflowState,
};
use crate::domains::communications::outbox::{
    CommunicationOutboxError, CommunicationOutboxItem, CommunicationOutboxStatus,
    CommunicationOutboxStore,
};
use crate::domains::communications::personas::{
    CommunicationPersona, CommunicationPersonaError, CommunicationPersonaStore,
};
use crate::domains::communications::saved_searches::{
    CommunicationSavedSearch, CommunicationSavedSearchError, CommunicationSavedSearchListQuery,
    CommunicationSavedSearchStore, NewCommunicationSavedSearch, UpdateCommunicationSavedSearch,
};
use crate::domains::communications::service::{
    CommunicationCommandService, CommunicationDraftUpsertCommand,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::communications::subscriptions::{
    SubscriptionError, SubscriptionSource, SubscriptionStore,
};
use crate::domains::communications::templates::{
    CommunicationMergePreview, CommunicationMergePreviewItem, CommunicationMergePreviewRow,
    CommunicationTemplate, CommunicationTemplateError, CommunicationTemplateStore,
    NewCommunicationTemplate, RenderedTemplate,
};
use crate::domains::communications::threads::{
    CommunicationThread, CommunicationThreadError, CommunicationThreadStore, ThreadMessage,
    ThreadMessageAttachment,
};
use crate::integrations::ai_runtime::{AiRuntimeClient, AiRuntimeError};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::integrations::omniroute::client::{
    OmniRouteClient, OmniRouteClientConfig, OmniRouteError,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};
use crate::platform::config::{AiRuntimeProvider, AppConfig};
use crate::platform::settings::ApplicationSettingsStore;

const AI_REQUEST_RUNTIME: &str = "ai_request_runtime";
const ATTACHMENT_TRANSLATION_SOURCE: &str = "caller_provided_extracted_text";
const MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS: usize = 64_000;
const MAX_TEXT_PREVIEW_BYTES: usize = 64 * 1024;
const MAX_IMAGE_PREVIEW_BYTES: usize = 5 * 1024 * 1024;
const MAX_AUDIO_PREVIEW_BYTES: usize = 24 * 1024 * 1024;
const MAX_VIDEO_PREVIEW_BYTES: usize = 32 * 1024 * 1024;
const MAX_PDF_PREVIEW_BYTES: usize = 16 * 1024 * 1024;

pub(crate) fn register(
    router: ConnectRouter,
    pool: Option<PgPool>,
    config: AppConfig,
) -> ConnectRouter {
    let Some(pool) = pool else {
        return router;
    };

    Arc::new(CommunicationsConnectService::new(pool, config)).register(router)
}

struct CommunicationsConnectService {
    config: AppConfig,
    pool: PgPool,
    message_store: MessageProjectionStore,
    analytics_store: EmailAnalyticsStore,
    subscription_store: SubscriptionStore,
    persona_store: CommunicationPersonaStore,
    template_store: CommunicationTemplateStore,
    thread_store: CommunicationThreadStore,
    draft_store: CommunicationDraftStore,
    outbox_store: CommunicationOutboxStore,
    storage_store: CommunicationStorageStore,
    attachment_search_store: AttachmentSearchStore,
    saved_search_store: CommunicationSavedSearchStore,
    folder_store: CommunicationFolderStore,
    audit_log: ApiAuditLog,
}

impl CommunicationsConnectService {
    fn new(pool: PgPool, config: AppConfig) -> Self {
        Self {
            config,
            pool: pool.clone(),
            message_store: MessageProjectionStore::new(pool.clone()),
            analytics_store: EmailAnalyticsStore::new(pool.clone()),
            subscription_store: SubscriptionStore::new(pool.clone()),
            persona_store: CommunicationPersonaStore::new(pool.clone()),
            template_store: CommunicationTemplateStore::new(pool.clone()),
            thread_store: CommunicationThreadStore::new(pool.clone()),
            draft_store: CommunicationDraftStore::new(pool.clone()),
            outbox_store: CommunicationOutboxStore::new(pool.clone()),
            storage_store: CommunicationStorageStore::new(pool.clone()),
            attachment_search_store: AttachmentSearchStore::new(pool.clone()),
            saved_search_store: CommunicationSavedSearchStore::new(pool.clone()),

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/connectrpc/signal_hub.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/connectrpc/signal_hub.rs`
- Size bytes / Размер в байтах: `39189`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;

use chrono::{DateTime, Utc};
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, Router as ConnectRouter, ServiceRequest,
    ServiceResult,
};
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::app::signal_hub_support::run_signal_hub_health_check;
use crate::application::SignalHubReplayService;
use crate::contracts::makosh::signal_hub::v1::{
    ApplyProfileRequest, ApplyProfileResponse, CreateConnectionRequest, CreateConnectionResponse,
    CreatePolicyRequest, CreatePolicyResponse, CreateProfileRequest, CreateProfileResponse,
    DisableSignalsRequest, DisableSignalsResponse, DisableSourceRequest, DisableSourceResponse,
    EmitFixtureSignalRequest, EmitFixtureSignalResponse, EnableSignalsRequest,
    EnableSignalsResponse, EnableSourceRequest, EnableSourceResponse, GetSourceRequest,
    GetSourceResponse, ListCapabilitiesRequest, ListCapabilitiesResponse, ListConnectionsRequest,
    ListConnectionsResponse, ListFixtureSourcesRequest, ListFixtureSourcesResponse,
    ListHealthRequest, ListHealthResponse, ListPoliciesRequest, ListPoliciesResponse,
    ListProfilesRequest, ListProfilesResponse, ListReplayRequestsRequest,
    ListReplayRequestsResponse, ListRuntimeStatesRequest, ListRuntimeStatesResponse,
    ListSourcesRequest, ListSourcesResponse, MuteSignalsRequest, MuteSignalsResponse,
    PauseSignalsRequest, PauseSignalsResponse, RemoveConnectionRequest, RemoveConnectionResponse,
    RemoveProfileRequest, RemoveProfileResponse, RequestReplayRequest, RequestReplayResponse,
    RestoreSystemFixtureRequest, RestoreSystemFixtureResponse, ResumeSignalsRequest,
    ResumeSignalsResponse, RunHealthCheckRequest, RunHealthCheckResponse,
    SignalCapability as ProtoSignalCapability, SignalConnection as ProtoSignalConnection,
    SignalFixtureSource as ProtoSignalFixtureSource, SignalHealth as ProtoSignalHealth,
    SignalHubService, SignalHubServiceExt, SignalPolicy as ProtoSignalPolicy,
    SignalProfile as ProtoSignalProfile, SignalProfilePolicy as ProtoSignalProfilePolicy,
    SignalReplayRequest as ProtoSignalReplayRequest, SignalRuntimeState as ProtoSignalRuntimeState,
    SignalSource as ProtoSignalSource, UnmuteSignalsRequest, UnmuteSignalsResponse,
    UpdateConnectionRequest, UpdateConnectionResponse, UpdateProfileRequest, UpdateProfileResponse,
    UpdateRuntimeStateRequest, UpdateRuntimeStateResponse,
};
use crate::domains::signal_hub::{
    SignalCapability, SignalConnection, SignalConnectionCreate, SignalConnectionUpdate,
    SignalFixtureEmitRequest, SignalFixtureSource, SignalFixtureSourceService, SignalHealth,
    SignalHealthCheckRequest as DomainSignalHealthCheckRequest, SignalHubCapabilityService,
    SignalHubConnectionService, SignalHubControlRequest, SignalHubControlService, SignalHubError,
    SignalHubHealthService, SignalHubProfileService, SignalHubStore, SignalPolicy,
    SignalPolicyMode, SignalPolicyScope, SignalProfileCreate, SignalProfilePolicy,
    SignalProfileSummary, SignalProfileUpdate, SignalReplayRequest, SignalReplayRequestCreate,
    SignalRuntimeState, SignalRuntimeStateUpdate, SignalSource,
};
use crate::platform::config::AppConfig;
use crate::platform::settings::ApplicationSettingsStore;

pub(crate) fn register(
    router: ConnectRouter,
    pool: Option<PgPool>,
    config: AppConfig,
) -> ConnectRouter {
    let Some(pool) = pool else {
        return router;
    };

    Arc::new(SignalHubConnectService::new(pool, config)).register(router)
}

struct SignalHubConnectService {
    config: AppConfig,
    capability_service: SignalHubCapabilityService,
    fixture_service: SignalFixtureSourceService,
    connection_service: SignalHubConnectionService,
    control_service: SignalHubControlService,
    profile_service: SignalHubProfileService,
    replay_service: SignalHubReplayService,
    store: SignalHubStore,
}

impl SignalHubConnectService {
    fn new(pool: PgPool, config: AppConfig) -> Self {
        let store = SignalHubStore::new(pool.clone());
        Self {
            config,
            capability_service: SignalHubCapabilityService::new(store.clone()),
            fixture_service: SignalFixtureSourceService::new(
                store.clone(),
                crate::platform::events::EventStore::new(pool.clone()),
            ),
            connection_service: SignalHubConnectionService::new(
                store.clone(),
                crate::platform::events::EventStore::new(pool.clone()),
            ),
            control_service: SignalHubControlService::new(
                store.clone(),
                crate::platform::events::EventStore::new(pool.clone()),
            ),
            profile_service: SignalHubProfileService::new(
                store.clone(),
                ApplicationSettingsStore::new(pool.clone()),
                crate::platform::events::EventStore::new(pool.clone()),
            ),
            replay_service: SignalHubReplayService::new(
                store.clone(),
                crate::platform::events::EventStore::new(pool),
            ),
            store,
        }
    }
}

#[allow(refining_impl_trait)]
impl SignalHubService for SignalHubConnectService {
    async fn list_sources(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListSourcesRequest>,
    ) -> ServiceResult<ListSourcesResponse> {
        let items = self
            .store
            .list_sources()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListSourcesResponse {
            items: items.into_iter().map(proto_source).collect(),
            ..Default::default()
        })
    }

    async fn get_source(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetSourceRequest>,
    ) -> ServiceResult<GetSourceResponse> {
        let item = self
            .store
            .get_source(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(GetSourceResponse {
            item: Some(proto_source(item)).into(),
            ..Default::default()
        })
    }

    async fn list_capabilities(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ListCapabilitiesRequest>,
    ) -> ServiceResult<ListCapabilitiesResponse> {
        let items = self
            .capability_service
            .list_capabilities(req.source_code, req.connection_id)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListCapabilitiesResponse {
            items: items.into_iter().map(proto_capability).collect(),
            ..Default::default()
        })
    }

    async fn list_fixture_sources(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListFixtureSourcesRequest>,
    ) -> ServiceResult<ListFixtureSourcesResponse> {
        let items = self
            .fixture_service
            .list_fixture_sources()
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListFixtureSourcesResponse {
            items: items.into_iter().map(proto_fixture_source).collect(),
            ..Default::default()
        })
    }

    async fn list_connections(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListConnectionsRequest>,
    ) -> ServiceResult<ListConnectionsResponse> {
        let items = self
            .store
            .list_connections()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListConnectionsResponse {
            items: items.into_iter().map(proto_connection).collect(),
            ..Default::default()
        })
    }

    async fn list_profiles(
        &self,
        _ctx: RequestContext,
        _req: ServiceRequest<'_, ListProfilesRequest>,
    ) -> ServiceResult<ListProfilesResponse> {
        let items = self
            .profile_service
            .list_profiles()
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ListProfilesResponse {
            items: items.into_iter().map(proto_profile).collect(),
            ..Default::default()
        })
    }

    async fn apply_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, ApplyProfileRequest>,
    ) -> ServiceResult<ApplyProfileResponse> {
        let item = self
            .profile_service
            .apply_profile(req.code)
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(ApplyProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn create_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, CreateProfileRequest>,
    ) -> ServiceResult<CreateProfileResponse> {
        let item = self
            .profile_service
            .create_profile(&SignalProfileCreate {
                code: req.code.to_owned(),
                display_name: req.display_name.to_owned(),
                description: req.description.to_owned(),
                source_policies: req
                    .source_policies
                    .iter()
                    .map(|policy| {
                        Ok(SignalProfilePolicy {
                            scope: SignalPolicyScope::parse(policy.scope).ok_or_else(|| {
                                SignalHubError::InvalidPolicyScope(policy.scope.to_string())
                            })?,
                            source_code: policy.source_code.map(str::to_owned),
                            connection_id: policy.connection_id.map(str::to_owned),
                            event_pattern: policy.event_pattern.map(str::to_owned),
                            mode: SignalPolicyMode::parse(policy.mode).ok_or_else(|| {
                                SignalHubError::InvalidPolicyMode(policy.mode.to_string())
                            })?,
                            reason: policy.reason.to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(signal_hub_connect_error)?,
            })
            .await
            .map_err(signal_hub_connect_error)?;
        Response::ok(CreateProfileResponse {
            item: Some(proto_profile(item)).into(),
            ..Default::default()
        })
    }

    async fn update_profile(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, UpdateProfileRequest>,
    ) -> ServiceResult<UpdateProfileResponse> {
        let source_policies = if req.update_source_policies {
            Some(
                req.source_policies
                    .iter()
                    .map(|policy| {
                        Ok(SignalProfilePolicy {
                            scope: SignalPolicyScope::parse(policy.scope).ok_or_else(|| {
                                SignalHubError::InvalidPolicyScope(policy.scope.to_string())
                            })?,
                            source_code: policy.source_code.map(str::to_owned),
                            connection_id: policy.connection_id.map(str::to_owned),
                            event_pattern: policy.event_pattern.map(str::to_owned),
                            mode: SignalPolicyMode::parse(policy.mode).ok_or_else(|| {
                                SignalHubError::InvalidPolicyMode(policy.mode.to_string())
                            })?,
                            reason: policy.reason.to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(signal_hub_connect_error)?,
            )
        } else {
            None
        };

        let item = self
            .profile_service
            .update_profile(&SignalProfileUpdate {
                code: req.code.to_owned(),
                display_name: req.display_name.map(ToOwned::to_owned),
                description
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/error.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error.rs`
- Size bytes / Размер в байтах: `87`
- Included characters / Включено символов: `87`
- Truncated / Обрезано: `no`

```rust
mod conversions;
mod response;
mod types;

pub(crate) use types::{ApiError, AppError};
```

### `backend/src/app/error/conversions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions.rs`
- Size bytes / Размер в байтах: `147`
- Included characters / Включено символов: `147`
- Truncated / Обрезано: `no`

```rust
mod ai;
mod calendar;
mod communications;
mod documents;
mod integrations;
mod knowledge;
mod organizations;
mod persons;
mod platform;
mod tasks;
```

### `backend/src/app/error/conversions/ai.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/ai.rs`
- Size bytes / Размер в байтах: `1254`
- Included characters / Включено символов: `1254`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;

impl From<AiError> for ApiError {
    fn from(error: AiError) -> Self {
        match error {
            AiError::RunNotFound => Self::AiRunNotFound,
            _ => Self::Ai(error),
        }
    }
}

impl From<AiControlCenterError> for ApiError {
    fn from(error: AiControlCenterError) -> Self {
        Self::AiControlCenter(error)
    }
}

impl From<crate::integrations::ollama::client::OllamaError> for ApiError {
    fn from(error: crate::integrations::ollama::client::OllamaError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::Ollama(error),
        ))
    }
}

impl From<crate::integrations::omniroute::client::OmniRouteError> for ApiError {
    fn from(error: crate::integrations::omniroute::client::OmniRouteError) -> Self {
        Self::Ai(AiError::Runtime(
            crate::integrations::ai_runtime::AiRuntimeError::OmniRoute(error),
        ))
    }
}

impl From<crate::integrations::ai_runtime::AiRuntimeError> for ApiError {
    fn from(error: crate::integrations::ai_runtime::AiRuntimeError) -> Self {
        Self::Ai(AiError::Runtime(error))
    }
}
```

### `backend/src/app/error/conversions/calendar.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/calendar.rs`
- Size bytes / Размер в байтах: `5728`
- Included characters / Включено символов: `5728`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::application::CalendarMeetingOutcomeApplicationError;
use crate::domains::calendar::brain::CalendarBrainError;
use crate::domains::calendar::core::CalendarCoreError;
use crate::domains::calendar::events::CalendarError;
use crate::domains::calendar::health::CalendarHealthError;
use crate::domains::calendar::meetings::MeetingsError;
use crate::domains::calendar::reminders::ReminderError;
use crate::domains::calendar::rules::CalendarRuleError;
use crate::domains::calendar::scheduling::SchedulingError;
use crate::domains::calendar::service::CalendarCommandServiceError;

impl From<CalendarCoreError> for ApiError {
    fn from(error: CalendarCoreError) -> Self {
        match error {
            CalendarCoreError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar core operation failed");
                ApiError::InvalidCommunicationQuery("calendar core operation failed")
            }
        }
    }
}

impl From<MeetingsError> for ApiError {
    fn from(error: MeetingsError) -> Self {
        match error {
            MeetingsError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "meetings operation failed");
                ApiError::InvalidCommunicationQuery("meetings operation failed")
            }
        }
    }
}

impl From<SchedulingError> for ApiError {
    fn from(error: SchedulingError) -> Self {
        match error {
            SchedulingError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "scheduling operation failed");
                ApiError::InvalidCommunicationQuery("scheduling operation failed")
            }
        }
    }
}

impl From<CalendarHealthError> for ApiError {
    fn from(error: CalendarHealthError) -> Self {
        tracing::error!(error = %error, "calendar health operation failed");
        ApiError::InvalidCommunicationQuery("calendar health operation failed")
    }
}

impl From<CalendarBrainError> for ApiError {
    fn from(error: CalendarBrainError) -> Self {
        match error {
            CalendarBrainError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar brain operation failed");
                ApiError::InvalidCommunicationQuery("calendar brain operation failed")
            }
        }
    }
}

impl From<ReminderError> for ApiError {
    fn from(error: ReminderError) -> Self {
        tracing::error!(error = %error, "reminder operation failed");
        ApiError::InvalidCommunicationQuery("reminder operation failed")
    }
}

impl From<CalendarRuleError> for ApiError {
    fn from(error: CalendarRuleError) -> Self {
        match error {
            CalendarRuleError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar rule operation failed");
                ApiError::InvalidCommunicationQuery("calendar rule operation failed")
            }
        }
    }
}

impl From<CalendarError> for ApiError {
    fn from(error: CalendarError) -> Self {
        match error {
            CalendarError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "calendar operation failed");
                ApiError::InvalidCommunicationQuery("calendar operation failed")
            }
        }
    }
}

impl From<CalendarCommandServiceError> for ApiError {
    fn from(error: CalendarCommandServiceError) -> Self {
        match error {
            CalendarCommandServiceError::ObservationCapture(source) => {
                tracing::error!(error = %source, "calendar command observation capture failed");
                ApiError::InvalidCommunicationQuery("calendar command observation capture failed")
            }
            CalendarCommandServiceError::Calendar(source) => ApiError::from(source),
            CalendarCommandServiceError::CalendarCore(source) => ApiError::from(source),
            CalendarCommandServiceError::Meetings(source) => ApiError::from(source),
            CalendarCommandServiceError::Reminder(source) => ApiError::from(source),
            CalendarCommandServiceError::CalendarRule(source) => ApiError::from(source),
            CalendarCommandServiceError::Scheduling(source) => ApiError::from(source),
        }
    }
}

impl From<CalendarMeetingOutcomeApplicationError> for ApiError {
    fn from(error: CalendarMeetingOutcomeApplicationError) -> Self {
        match error {
            CalendarMeetingOutcomeApplicationError::Meetings(source) => ApiError::from(source),
            CalendarMeetingOutcomeApplicationError::Decision(source) => ApiError::from(source),
            CalendarMeetingOutcomeApplicationError::Obligation(source) => ApiError::from(source),
            CalendarMeetingOutcomeApplicationError::Observation(source) => {
                tracing::error!(error = %source, "calendar meeting outcome observation failed");
                ApiError::InvalidCommunicationQuery("calendar meeting outcome observation failed")
            }
            CalendarMeetingOutcomeApplicationError::ReviewMirror(source) => {
                tracing::error!(error = %source, "calendar meeting outcome review mirror failed");
                ApiError::InvalidCommunicationQuery("calendar meeting outcome review mirror failed")
            }
            CalendarMeetingOutcomeApplicationError::Sqlx(source) => {
                tracing::error!(error = %source, "calendar meeting outcome operation failed");
                ApiError::InvalidCommunicationQuery("calendar meeting outcome operation failed")
            }
        }
    }
}
```

### `backend/src/app/error/conversions/communications.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/communications.rs`
- Size bytes / Размер в байтах: `17958`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::super::types::ApiError;
use crate::application::email_intelligence::EmailIntelligenceError;
use crate::domains::communications::core::CommunicationIngestionError;
use crate::domains::communications::messages::MessageProjectionError;
use crate::domains::communications::service::CommunicationCommandServiceError;
use crate::domains::communications::storage::CommunicationStorageError;
use crate::integrations::mail::accounts::EmailAccountSetupError;

impl From<CommunicationIngestionError> for ApiError {
    fn from(error: CommunicationIngestionError) -> Self {
        Self::CommunicationIngestion(error)
    }
}

impl From<MessageProjectionError> for ApiError {
    fn from(error: MessageProjectionError) -> Self {
        match error {
            MessageProjectionError::MessageNotFound => ApiError::CommunicationMessageNotFound,
            error => Self::Messages(error),
        }
    }
}

impl From<CommunicationStorageError> for ApiError {
    fn from(error: CommunicationStorageError) -> Self {
        Self::CommunicationStorage(error)
    }
}

impl From<crate::domains::communications::threads::CommunicationThreadError> for ApiError {
    fn from(error: crate::domains::communications::threads::CommunicationThreadError) -> Self {
        match error {
            crate::domains::communications::threads::CommunicationThreadError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid thread cursor")
            }
            error => {
                tracing::error!(error = %error, "email thread operation failed");
                ApiError::InvalidCommunicationQuery("email thread operation failed")
            }
        }
    }
}

impl From<EmailIntelligenceError> for ApiError {
    fn from(error: EmailIntelligenceError) -> Self {
        match error {
            EmailIntelligenceError::ParseError(_msg) => {
                ApiError::InvalidCommunicationQuery("failed to parse AI analysis result")
            }
            _ => {
                tracing::error!(error = %error, "email intelligence operation failed");
                ApiError::InvalidCommunicationQuery("email intelligence operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::drafts::CommunicationDraftError> for ApiError {
    fn from(error: crate::domains::communications::drafts::CommunicationDraftError) -> Self {
        match error {
            crate::domains::communications::drafts::CommunicationDraftError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid draft request")
            }
            crate::domains::communications::drafts::CommunicationDraftError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid draft cursor")
            }
            error => {
                tracing::error!(error = %error, "email draft operation failed");
                ApiError::InvalidCommunicationQuery("email draft operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::outbox::CommunicationOutboxError> for ApiError {
    fn from(error: crate::domains::communications::outbox::CommunicationOutboxError) -> Self {
        match error {
            crate::domains::communications::outbox::CommunicationOutboxError::UndoUnavailable => {
                ApiError::NotFound
            }
            crate::domains::communications::outbox::CommunicationOutboxError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid outbox cursor")
            }
            error => {
                tracing::error!(error = %error, "email outbox operation failed");
                ApiError::InvalidCommunicationQuery("email outbox operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::bulk_actions::BulkMessageActionError> for ApiError {
    fn from(error: crate::domains::communications::bulk_actions::BulkMessageActionError) -> Self {
        match error {
            crate::domains::communications::bulk_actions::BulkMessageActionError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid bulk message action request")
            }
            error => {
                tracing::error!(error = %error, "bulk message action failed");
                ApiError::InvalidCommunicationQuery("bulk message action failed")
            }
        }
    }
}

impl From<crate::domains::communications::saved_searches::CommunicationSavedSearchError>
    for ApiError
{
    fn from(
        error: crate::domains::communications::saved_searches::CommunicationSavedSearchError,
    ) -> Self {
        match error {
            crate::domains::communications::saved_searches::CommunicationSavedSearchError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid saved search request")
            }
            crate::domains::communications::saved_searches::CommunicationSavedSearchError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid saved search cursor")
            }
            error => {
                tracing::error!(error = %error, "mail saved search operation failed");
                ApiError::InvalidCommunicationQuery("mail saved search operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::folders::CommunicationFolderError> for ApiError {
    fn from(error: crate::domains::communications::folders::CommunicationFolderError) -> Self {
        match error {
            crate::domains::communications::folders::CommunicationFolderError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail folder request")
            }
            crate::domains::communications::folders::CommunicationFolderError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid mail folder cursor")
            }
            error => {
                tracing::error!(error = %error, "mail folder operation failed");
                ApiError::InvalidCommunicationQuery("mail folder operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::ai_state::CommunicationAiStateError> for ApiError {
    fn from(error: crate::domains::communications::ai_state::CommunicationAiStateError) -> Self {
        match error {
            crate::domains::communications::ai_state::CommunicationAiStateError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail AI state request")
            }
            error => {
                tracing::error!(error = %error, "mail AI state operation failed");
                ApiError::InvalidCommunicationQuery("mail AI state operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::read_receipts::CommunicationReadReceiptError>
    for ApiError
{
    fn from(
        error: crate::domains::communications::read_receipts::CommunicationReadReceiptError,
    ) -> Self {
        match error {
            crate::domains::communications::read_receipts::CommunicationReadReceiptError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail read receipt request")
            }
            error => {
                tracing::error!(error = %error, "mail read receipt operation failed");
                ApiError::InvalidCommunicationQuery("mail read receipt operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::templates::CommunicationTemplateError> for ApiError {
    fn from(error: crate::domains::communications::templates::CommunicationTemplateError) -> Self {
        match error {
            crate::domains::communications::templates::CommunicationTemplateError::InvalidTemplate(_) => {
                ApiError::InvalidCommunicationQuery("invalid email template request")
            }
            error => {
                tracing::error!(error = %error, "email template operation failed");
                ApiError::InvalidCommunicationQuery("email template operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::delivery_notifications::CommunicationDeliveryNotificationError>
    for ApiError
{
    fn from(
        error: crate::domains::communications::delivery_notifications::CommunicationDeliveryNotificationError,
    ) -> Self {
        match error {
            crate::domains::communications::delivery_notifications::CommunicationDeliveryNotificationError::Invalid(_) => {
                ApiError::InvalidCommunicationQuery("invalid mail delivery notification request")
            }
            crate::domains::communications::delivery_notifications::CommunicationDeliveryNotificationError::SignalControlBlocked(_) => {
                ApiError::InvalidCommunicationQuery("mail delivery notification deferred by Signal Hub control")
            }
            error => {
                tracing::error!(error = %error, "mail delivery notification operation failed");
                ApiError::InvalidCommunicationQuery("mail delivery notification operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::finance::CommunicationFinanceError> for ApiError {
    fn from(error: crate::domains::communications::finance::CommunicationFinanceError) -> Self {
        tracing::error!(error = %error, "email finance operation failed");
        ApiError::InvalidCommunicationQuery("email finance operation failed")
    }
}

impl From<crate::domains::communications::analytics::EmailAnalyticsError> for ApiError {
    fn from(error: crate::domains::communications::analytics::EmailAnalyticsError) -> Self {
        match error {
            crate::domains::communications::analytics::EmailAnalyticsError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid analytics cursor")
            }
            error => {
                tracing::error!(error = %error, "email analytics operation failed");
                ApiError::InvalidCommunicationQuery("email analytics operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::personas::CommunicationPersonaError> for ApiError {
    fn from(error: crate::domains::communications::personas::CommunicationPersonaError) -> Self {
        tracing::error!(error = %error, "email persona operation failed");
        ApiError::InvalidCommunicationQuery("email persona operation failed")
    }
}

impl From<crate::domains::communications::search::IndexEmailError> for ApiError {
    fn from(error: crate::domains::communications::search::IndexEmailError) -> Self {
        tracing::error!(error = %error, "email search operation failed");
        ApiError::InvalidCommunicationQuery("email search operation failed")
    }
}

impl From<crate::domains::communications::flags::MessageFlagsError> for ApiError {
    fn from(error: crate::domains::communications::flags::MessageFlagsError) -> Self {
        match error {
            crate::domains::communications::flags::MessageFlagsError::NotFound => {
                ApiError::CommunicationMessageNotFound
            }
            crate::domains::communications::flags::MessageFlagsError::MessageProjection(inner) => {
                ApiError::from(inner)
            }
        }
    }
}

impl From<crate::domains::communications::subscriptions::SubscriptionError> for ApiError {
    fn from(error: crate::domains::communications::subscriptions::SubscriptionError) -> Self {
        match error {
            crate::domains::communications::subscriptions::SubscriptionError::InvalidCursor => {
                ApiError::InvalidCommunicationQuery("invalid subscription cursor")
            }
            error => {
                tracing::error!(error = %error, "subscriptions operation failed");
                ApiError::InvalidCommunicationQuery("subscriptions operation failed")
            }
        }
    }
}

impl From<crate::domains::communications::attachment_dedup::At
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/error/conversions/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/documents.rs`
- Size bytes / Размер в байтах: `917`
- Included characters / Включено символов: `917`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::domains::documents::processing::{
    DocumentProcessingCommandServiceError, DocumentProcessingError,
};

impl From<DocumentProcessingError> for ApiError {
    fn from(error: DocumentProcessingError) -> Self {
        Self::DocumentProcessing(error)
    }
}

impl From<DocumentProcessingCommandServiceError> for ApiError {
    fn from(error: DocumentProcessingCommandServiceError) -> Self {
        match error {
            DocumentProcessingCommandServiceError::DocumentProcessing(error) => Self::from(error),
            DocumentProcessingCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "document processing retry observation capture failed");
                ApiError::InvalidCommunicationQuery(
                    "document processing retry observation capture failed",
                )
            }
        }
    }
}
```

### `backend/src/app/error/conversions/integrations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/integrations.rs`
- Size bytes / Размер в байтах: `1982`
- Included characters / Включено символов: `1982`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::application::communication_fixture_ingest::CommunicationFixtureIngestError;
use crate::application::communication_provider_writes::TelegramMessageWriteError;
use crate::application::provider_runtime_contracts::{
    TelegramError, WhatsappWebError, YandexTelemostError, ZoomError,
};
use crate::application::review_inbox::ReviewInboxWorkflowError;
use crate::engines::automation::AutomationError;
use crate::platform::calls::CallError;

impl From<TelegramError> for ApiError {
    fn from(error: TelegramError) -> Self {
        Self::Telegram(error)
    }
}

impl From<WhatsappWebError> for ApiError {
    fn from(error: WhatsappWebError) -> Self {
        Self::WhatsappWeb(error)
    }
}

impl From<ZoomError> for ApiError {
    fn from(error: ZoomError) -> Self {
        Self::Zoom(error)
    }
}

impl From<YandexTelemostError> for ApiError {
    fn from(error: YandexTelemostError) -> Self {
        Self::YandexTelemost(error)
    }
}

impl From<AutomationError> for ApiError {
    fn from(error: AutomationError) -> Self {
        Self::Automation(error)
    }
}

impl From<CallError> for ApiError {
    fn from(error: CallError) -> Self {
        Self::Call(error)
    }
}

impl From<CommunicationFixtureIngestError> for ApiError {
    fn from(_error: CommunicationFixtureIngestError) -> Self {
        Self::InvalidCommunicationQuery("communication fixture ingest failed")
    }
}

impl From<TelegramMessageWriteError> for ApiError {
    fn from(error: TelegramMessageWriteError) -> Self {
        tracing::error!(error = %error, "telegram message write failed");
        Self::InvalidCommunicationQuery("telegram message write failed")
    }
}

impl From<ReviewInboxWorkflowError> for ApiError {
    fn from(error: ReviewInboxWorkflowError) -> Self {
        tracing::error!(error = %error, "communication review inbox sync failed");
        Self::InvalidCommunicationQuery("communication review inbox sync failed")
    }
}
```

### `backend/src/app/error/conversions/knowledge.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/knowledge.rs`
- Size bytes / Размер в байтах: `17737`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::super::types::ApiError;
use crate::application::consistency_review::ContradictionReviewServiceError;
use crate::application::review_promotion::ReviewPromotionError;
use crate::application::{
    DecisionReviewApplicationError, ObligationReviewApplicationError,
    RelationshipReviewApplicationError, TaskCandidateReviewApplicationError,
};
use crate::domains::decisions::{DecisionCommandServiceError, DecisionStoreError};
use crate::domains::obligations::{ObligationCommandServiceError, ObligationStoreError};
use crate::domains::projects::core::ProjectStoreError;
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewError, ProjectLinkReviewServiceError,
};
use crate::domains::relationships::{RelationshipCommandServiceError, RelationshipStoreError};
use crate::domains::review::{ReviewInboxError, ReviewInboxServiceError};
use crate::domains::tasks::candidates::{TaskCandidateError, TaskCandidateReviewServiceError};
use crate::engines::consistency::ConsistencyError;

impl From<crate::domains::graph::core::GraphStoreError> for ApiError {
    fn from(error: crate::domains::graph::core::GraphStoreError) -> Self {
        Self::Graph(error)
    }
}

impl From<ProjectLinkReviewError> for ApiError {
    fn from(error: ProjectLinkReviewError) -> Self {
        match error {
            ProjectLinkReviewError::ProjectNotFound | ProjectLinkReviewError::TargetNotFound => {
                Self::ProjectLinkTargetNotFound
            }
            _ => Self::ProjectLinkReview(error),
        }
    }
}

impl From<ProjectLinkReviewServiceError> for ApiError {
    fn from(error: ProjectLinkReviewServiceError) -> Self {
        match error {
            ProjectLinkReviewServiceError::ProjectLinkReview(error) => Self::from(error),
            ProjectLinkReviewServiceError::Observation(error) => {
                tracing::error!(error = %error, "project link review observation capture failed");
                Self::InvalidCommunicationQuery("project link review observation capture failed")
            }
        }
    }
}

impl From<ProjectStoreError> for ApiError {
    fn from(error: ProjectStoreError) -> Self {
        Self::Projects(error)
    }
}

impl From<TaskCandidateError> for ApiError {
    fn from(error: TaskCandidateError) -> Self {
        match error {
            TaskCandidateError::TaskCandidateNotFound => Self::TaskCandidateNotFound,
            _ => Self::TaskCandidate(error),
        }
    }
}

impl From<TaskCandidateReviewServiceError> for ApiError {
    fn from(error: TaskCandidateReviewServiceError) -> Self {
        match error {
            TaskCandidateReviewServiceError::TaskCandidate(error) => Self::from(error),
            TaskCandidateReviewServiceError::Observation(error) => {
                tracing::error!(error = %error, "task candidate review observation capture failed");
                Self::InvalidTaskCandidateQuery("task candidate review observation capture failed")
            }
        }
    }
}

impl From<TaskCandidateReviewApplicationError> for ApiError {
    fn from(error: TaskCandidateReviewApplicationError) -> Self {
        match error {
            TaskCandidateReviewApplicationError::TaskCandidate(error) => Self::from(error),
            TaskCandidateReviewApplicationError::TaskCandidateNotFound => {
                Self::TaskCandidateNotFound
            }
            TaskCandidateReviewApplicationError::Sqlx(error) => {
                Self::TaskCandidate(TaskCandidateError::Sqlx(error))
            }
            TaskCandidateReviewApplicationError::ReviewMirror(error) => {
                tracing::error!(error = %error, "task candidate review mirror sync failed");
                Self::InvalidTaskCandidateQuery("task candidate review mirror sync failed")
            }
        }
    }
}

impl From<ObligationStoreError> for ApiError {
    fn from(error: ObligationStoreError) -> Self {
        match error {
            ObligationStoreError::ObligationNotFound => Self::ObligationNotFound,
            ObligationStoreError::UnknownEntityKind(_) => Self::InvalidObligationQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            ObligationStoreError::MissingEvidence => {
                Self::InvalidObligationQuery("obligation evidence is required")
            }
            ObligationStoreError::ObservationNotFound(_) => {
                Self::InvalidObligationQuery("obligation evidence observation was not found")
            }
            ObligationStoreError::InvalidObservationEvidenceSource => Self::InvalidObligationQuery(
                "obligation observation evidence must use the same source_id and observation_id",
            ),
            ObligationStoreError::UnknownEvidenceSourceKind(_) => {
                Self::InvalidObligationQuery("obligation evidence source kind is invalid")
            }
            ObligationStoreError::UnknownReviewState(_) => Self::InvalidObligationReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Obligation(error),
        }
    }
}

impl From<DecisionStoreError> for ApiError {
    fn from(error: DecisionStoreError) -> Self {
        match error {
            DecisionStoreError::DecisionNotFound => Self::DecisionNotFound,
            DecisionStoreError::UnknownEntityKind(_) => Self::InvalidDecisionQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            DecisionStoreError::MissingEvidence => {
                Self::InvalidDecisionQuery("decision evidence is required")
            }
            DecisionStoreError::ObservationNotFound(_) => {
                Self::InvalidDecisionQuery("decision evidence observation was not found")
            }
            DecisionStoreError::InvalidObservationEvidenceSource => Self::InvalidDecisionQuery(
                "decision observation evidence must use the same source_id and observation_id",
            ),
            DecisionStoreError::UnknownEvidenceSourceKind(_) => {
                Self::InvalidDecisionQuery("decision evidence source kind is invalid")
            }
            DecisionStoreError::UnknownReviewState(_) => Self::InvalidDecisionReview(
                "review_state must be suggested, user_confirmed, or user_rejected",
            ),
            _ => Self::Decision(error),
        }
    }
}

impl From<RelationshipStoreError> for ApiError {
    fn from(error: RelationshipStoreError) -> Self {
        match error {
            RelationshipStoreError::RelationshipNotFound => Self::RelationshipNotFound,
            RelationshipStoreError::UnknownEntityKind(_) => Self::InvalidRelationshipQuery(
                "entity_kind must be persona, organization, project, communication, document, task, event, decision, obligation, or knowledge",
            ),
            RelationshipStoreError::MissingEvidence => {
                Self::InvalidRelationshipQuery("relationship evidence is required")
            }
            RelationshipStoreError::ObservationNotFound(_) => {
                Self::InvalidRelationshipQuery("relationship evidence observation was not found")
            }
            RelationshipStoreError::InvalidObservationEvidenceSource => {
                Self::InvalidRelationshipQuery(
                    "relationship observation evidence must use the same source_id and observation_id",
                )
            }
            RelationshipStoreError::UnknownEvidenceSourceKind(_) => {
                Self::InvalidRelationshipQuery("relationship evidence source kind is invalid")
            }
            RelationshipStoreError::UnknownReviewState(_) => Self::InvalidRelationshipReview(
                "review_state must be suggested, system_accepted, user_confirmed, or user_rejected",
            ),
            _ => Self::Relationship(error),
        }
    }
}

impl From<DecisionCommandServiceError> for ApiError {
    fn from(error: DecisionCommandServiceError) -> Self {
        match error {
            DecisionCommandServiceError::Decision(error) => Self::from(error),
            DecisionCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "decision review observation capture failed");
                Self::InvalidDecisionReview("decision review observation capture failed")
            }
        }
    }
}

impl From<DecisionReviewApplicationError> for ApiError {
    fn from(error: DecisionReviewApplicationError) -> Self {
        match error {
            DecisionReviewApplicationError::Decision(error) => Self::from(error),
            DecisionReviewApplicationError::Observation(error) => {
                tracing::error!(error = %error, "decision review observation capture failed");
                Self::InvalidDecisionReview("decision review observation capture failed")
            }
            DecisionReviewApplicationError::ReviewMirror(error) => {
                tracing::error!(error = %error, "decision review inbox sync failed");
                Self::InvalidDecisionReview("decision review inbox sync failed")
            }
        }
    }
}

impl From<ObligationCommandServiceError> for ApiError {
    fn from(error: ObligationCommandServiceError) -> Self {
        match error {
            ObligationCommandServiceError::Obligation(error) => Self::from(error),
            ObligationCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "obligation review observation capture failed");
                Self::InvalidObligationReview("obligation review observation capture failed")
            }
        }
    }
}

impl From<ObligationReviewApplicationError> for ApiError {
    fn from(error: ObligationReviewApplicationError) -> Self {
        match error {
            ObligationReviewApplicationError::Obligation(error) => Self::from(error),
            ObligationReviewApplicationError::Observation(error) => {
                tracing::error!(error = %error, "obligation review observation capture failed");
                Self::InvalidObligationReview("obligation review observation capture failed")
            }
            ObligationReviewApplicationError::ReviewMirror(error) => {
                tracing::error!(error = %error, "obligation review inbox sync failed");
                Self::InvalidObligationReview("obligation review inbox sync failed")
            }
        }
    }
}

impl From<RelationshipCommandServiceError> for ApiError {
    fn from(error: RelationshipCommandServiceError) -> Self {
        match error {
            RelationshipCommandServiceError::Relationship(error) => Self::from(error),
            RelationshipCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "relationship review observation capture failed");
                Self::InvalidRelationshipReview("relationship review observation capture failed")
            }
        }
    }
}

impl From<RelationshipReviewApplicationError> for ApiError {
    fn from(error: RelationshipReviewApplicationError) -> Self {
        match error {
            RelationshipReviewApplicationError::Relationship(error) => Self::from(error),
            RelationshipReviewApplicationError::Observation(error) => {
                tracing::error!(error = %error, "relationship review observation capture failed");
                Self::InvalidRelationshipReview("relationship review observation capture failed")
            }
            RelationshipReviewApplicationError::ReviewMirror(error) => {
                tracing::error!(error = %error, "relationship review inbox sync failed");
                Self::InvalidRelationshipReview("relationship review inbox sync failed")
            }
        }
    }
}

impl From<ConsistencyError> for ApiError {

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/error/conversions/organizations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/organizations.rs`
- Size bytes / Размер в байтах: `4361`
- Included characters / Включено символов: `4361`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::application::OrganizationContactLinkApplicationError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::service::OrganizationCommandServiceError;

impl From<crate::domains::organizations::core::OrgCoreError> for ApiError {
    fn from(error: crate::domains::organizations::core::OrgCoreError) -> Self {
        tracing::error!(error = %error, "org core operation failed");
        ApiError::InvalidCommunicationQuery("org core operation failed")
    }
}

impl From<crate::domains::organizations::memory::OrgMemoryError> for ApiError {
    fn from(error: crate::domains::organizations::memory::OrgMemoryError) -> Self {
        tracing::error!(error = %error, "org memory operation failed");
        ApiError::InvalidCommunicationQuery("org memory operation failed")
    }
}

impl From<crate::domains::organizations::workflows::OrgWorkflowError> for ApiError {
    fn from(error: crate::domains::organizations::workflows::OrgWorkflowError) -> Self {
        tracing::error!(error = %error, "org workflow operation failed");
        ApiError::InvalidCommunicationQuery("org workflow operation failed")
    }
}

impl From<crate::domains::organizations::finance::OrgFinanceError> for ApiError {
    fn from(error: crate::domains::organizations::finance::OrgFinanceError) -> Self {
        tracing::error!(error = %error, "org finance operation failed");
        ApiError::InvalidCommunicationQuery("org finance operation failed")
    }
}

impl From<crate::domains::organizations::enrichment::OrgEnrichmentError> for ApiError {
    fn from(error: crate::domains::organizations::enrichment::OrgEnrichmentError) -> Self {
        tracing::error!(error = %error, "org enrichment operation failed");
        ApiError::InvalidCommunicationQuery("org enrichment operation failed")
    }
}

impl From<crate::domains::organizations::health::OrgHealthError> for ApiError {
    fn from(error: crate::domains::organizations::health::OrgHealthError) -> Self {
        tracing::error!(error = %error, "org health operation failed");
        ApiError::InvalidCommunicationQuery("org health operation failed")
    }
}

impl From<crate::domains::organizations::investigator::InvestigatorError> for ApiError {
    fn from(error: crate::domains::organizations::investigator::InvestigatorError) -> Self {
        match error {
            crate::domains::organizations::investigator::InvestigatorError::NotFound => {
                ApiError::NotFound
            }
            _ => {
                tracing::error!(error = %error, "investigator operation failed");
                ApiError::InvalidCommunicationQuery("investigator operation failed")
            }
        }
    }
}

impl From<OrganizationError> for ApiError {
    fn from(error: OrganizationError) -> Self {
        match error {
            OrganizationError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "organization operation failed");
                ApiError::InvalidCommunicationQuery("organization operation failed")
            }
        }
    }
}

impl From<OrganizationCommandServiceError> for ApiError {
    fn from(error: OrganizationCommandServiceError) -> Self {
        match error {
            OrganizationCommandServiceError::Observation(source) => {
                tracing::error!(error = %source, "organization command observation capture failed");
                ApiError::InvalidCommunicationQuery(
                    "organization command observation capture failed",
                )
            }
            OrganizationCommandServiceError::Organization(source) => ApiError::from(source),
            OrganizationCommandServiceError::Core(source) => ApiError::from(source),
            OrganizationCommandServiceError::Enrichment(source) => ApiError::from(source),
            OrganizationCommandServiceError::Health(source) => ApiError::from(source),
        }
    }
}

impl From<OrganizationContactLinkApplicationError> for ApiError {
    fn from(error: OrganizationContactLinkApplicationError) -> Self {
        match error {
            OrganizationContactLinkApplicationError::Organization(source) => Self::from(source),
            OrganizationContactLinkApplicationError::Relationship(source) => Self::from(source),
        }
    }
}
```

### `backend/src/app/error/conversions/persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/persons.rs`
- Size bytes / Размер в байтах: `3937`
- Included characters / Включено символов: `3937`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::core::PersonCoreError;
use crate::domains::persons::identity::PersonIdentityError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::persons::service::PersonCommandServiceError;

impl From<PersonIdentityError> for ApiError {
    fn from(error: PersonIdentityError) -> Self {
        match error {
            PersonIdentityError::IdentityCandidateNotFound => Self::PersonIdentityNotFound,
            PersonIdentityError::InvalidLimit | PersonIdentityError::InvalidReviewState(_) => {
                Self::InvalidPersonIdentityReview(
                    "review_state or limit must be valid for person identity candidates",
                )
            }
            PersonIdentityError::InvalidPayload(_)
            | PersonIdentityError::MissingPayloadField(_)
            | PersonIdentityError::MissingActorId => {
                Self::InvalidPersonIdentityReview("invalid identity candidate review payload")
            }
            PersonIdentityError::Observation(_) => {
                Self::InvalidPersonIdentityReview("identity candidate evidence observation failed")
            }
            _ => Self::PersonIdentity(error),
        }
    }
}

impl From<PersonProjectionError> for ApiError {
    fn from(error: PersonProjectionError) -> Self {
        Self::PersonProjection(error)
    }
}

impl From<crate::domains::persons::enrichment::PersonEnrichmentError> for ApiError {
    fn from(error: crate::domains::persons::enrichment::PersonEnrichmentError) -> Self {
        match error {
            crate::domains::persons::enrichment::PersonEnrichmentError::NotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person enrichment failed");
                ApiError::InvalidCommunicationQuery("person enrichment failed")
            }
        }
    }
}

impl From<PersonMemoryError> for ApiError {
    fn from(error: PersonMemoryError) -> Self {
        match error {
            PersonMemoryError::NotFound => ApiError::PersonIdentityNotFound,
            _ => {
                tracing::error!(error = %error, "person memory operation failed");
                ApiError::InvalidCommunicationQuery("person memory operation failed")
            }
        }
    }
}

impl From<PersonCoreError> for ApiError {
    fn from(error: PersonCoreError) -> Self {
        match error {
            PersonCoreError::IdentityNotFound | PersonCoreError::PersonaNotFound => {
                ApiError::PersonIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "person core operation failed");
                ApiError::InvalidCommunicationQuery("person core operation failed")
            }
        }
    }
}

impl From<PersonCommandServiceError> for ApiError {
    fn from(error: PersonCommandServiceError) -> Self {
        match error {
            PersonCommandServiceError::Projection(error) => Self::from(error),
            PersonCommandServiceError::Core(error) => Self::from(error),
            PersonCommandServiceError::Enrichment(error) => Self::from(error),
            PersonCommandServiceError::EnrichmentEngine(error) => Self::from(error),
            PersonCommandServiceError::Memory(error) => Self::from(error),
            PersonCommandServiceError::Health(error) => Self::from(error),
            PersonCommandServiceError::Identity(error) => Self::from(error),
            PersonCommandServiceError::Investigator(error) => Self::from(error),
            PersonCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "person manual observation capture failed");
                ApiError::InvalidCommunicationQuery("person manual observation capture failed")
            }
        }
    }
}
```

### `backend/src/app/error/conversions/platform.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/platform.rs`
- Size bytes / Размер в байтах: `1151`
- Included characters / Включено символов: `1151`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::domains::signal_hub::SignalHubError;
use crate::platform::audit::ApiAuditError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::settings::SettingsError;
use crate::vault::HostVaultError;

impl From<EventEnvelopeError> for ApiError {
    fn from(error: EventEnvelopeError) -> Self {
        Self::InvalidEnvelope(error)
    }
}

impl From<EventStoreError> for ApiError {
    fn from(error: EventStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<SettingsError> for ApiError {
    fn from(error: SettingsError) -> Self {
        match error {
            SettingsError::SettingNotFound { .. } => Self::SettingNotFound,
            _ => Self::Settings(error),
        }
    }
}

impl From<SignalHubError> for ApiError {
    fn from(error: SignalHubError) -> Self {
        Self::SignalHub(error)
    }
}

impl From<ApiAuditError> for ApiError {
    fn from(error: ApiAuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<HostVaultError> for ApiError {
    fn from(error: HostVaultError) -> Self {
        Self::HostVault(error)
    }
}
```

### `backend/src/app/error/conversions/tasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/conversions/tasks.rs`
- Size bytes / Размер в байтах: `5602`
- Included characters / Включено символов: `5602`
- Truncated / Обрезано: `no`

```rust
use super::super::types::ApiError;
use crate::domains::tasks::api::TaskError;
use crate::domains::tasks::brain::TaskBrainError;
use crate::domains::tasks::core::TaskCoreError;
use crate::domains::tasks::health::TaskHealthError;
use crate::domains::tasks::rules::TaskRuleError;
use crate::domains::tasks::service::TaskCommandServiceError;

impl From<TaskError> for ApiError {
    fn from(error: TaskError) -> Self {
        match error {
            TaskError::NotFound => ApiError::NotFound,
            TaskError::MissingProvenance => {
                ApiError::InvalidTaskQuery("task provenance is required")
            }
            TaskError::InvalidProvenanceSpec => {
                ApiError::InvalidTaskQuery("invalid task provenance specification")
            }
            TaskError::UnknownProvenanceKind => {
                ApiError::InvalidTaskQuery("unknown task provenance kind")
            }
            TaskError::MissingProvenanceObservation => {
                ApiError::InvalidTaskQuery("missing task provenance observation")
            }
            TaskError::MissingProvenanceReference => {
                ApiError::InvalidTaskQuery("task provenance reference does not exist")
            }
            TaskError::MissingProvenanceEvidence => {
                ApiError::InvalidTaskQuery("task provenance reference has no observation evidence")
            }
            TaskError::MissingSourceIdentifier => {
                ApiError::InvalidTaskQuery("missing task source identifier")
            }
            _ => {
                tracing::error!(error = %error, "task operation failed");
                ApiError::InvalidCommunicationQuery("task operation failed")
            }
        }
    }
}

impl From<TaskCoreError> for ApiError {
    fn from(error: TaskCoreError) -> Self {
        match error {
            TaskCoreError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task core operation failed");
                ApiError::InvalidCommunicationQuery("task core operation failed")
            }
        }
    }
}

impl From<TaskHealthError> for ApiError {
    fn from(error: TaskHealthError) -> Self {
        tracing::error!(error = %error, "task health failed");
        ApiError::InvalidCommunicationQuery("task health failed")
    }
}

impl From<TaskRuleError> for ApiError {
    fn from(error: TaskRuleError) -> Self {
        match error {
            TaskRuleError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task rule failed");
                ApiError::InvalidCommunicationQuery("task rule failed")
            }
        }
    }
}

impl From<TaskBrainError> for ApiError {
    fn from(error: TaskBrainError) -> Self {
        match error {
            TaskBrainError::NotFound => ApiError::NotFound,
            _ => {
                tracing::error!(error = %error, "task brain failed");
                ApiError::InvalidCommunicationQuery("task brain failed")
            }
        }
    }
}

impl From<TaskCommandServiceError> for ApiError {
    fn from(error: TaskCommandServiceError) -> Self {
        match error {
            TaskCommandServiceError::ObservationCapture { operation, source } => {
                tracing::error!(error = %source, operation, "task command observation capture failed");
                match operation {
                    "task update" => {
                        ApiError::InvalidTaskQuery("task update observation capture failed")
                    }
                    "task status" => {
                        ApiError::InvalidTaskQuery("task status observation capture failed")
                    }
                    "task archive" => {
                        ApiError::InvalidTaskQuery("task archive observation capture failed")
                    }
                    "task analyze" => {
                        ApiError::InvalidTaskQuery("task analyze observation capture failed")
                    }
                    "task evidence" => {
                        ApiError::InvalidTaskQuery("task evidence observation capture failed")
                    }
                    "task relation" => {
                        ApiError::InvalidTaskQuery("task relation observation capture failed")
                    }
                    "task checklist" => {
                        ApiError::InvalidTaskQuery("task checklist observation capture failed")
                    }
                    "task subtask" => {
                        ApiError::InvalidTaskQuery("task subtask observation capture failed")
                    }
                    _ => ApiError::InvalidTaskQuery("task observation capture failed"),
                }
            }
            TaskCommandServiceError::MissingEvidenceSourceId => {
                ApiError::InvalidTaskQuery("task evidence source id is required")
            }
            TaskCommandServiceError::ObservationStore(source) => {
                tracing::error!(error = %source, "task observation store operation failed");
                ApiError::InvalidTaskQuery("task observation store operation failed")
            }
            TaskCommandServiceError::Task(inner) => ApiError::from(inner),
            TaskCommandServiceError::Core(inner) => ApiError::from(inner),
            TaskCommandServiceError::Sqlx(source) => {
                tracing::error!(error = %source, "task command sql operation failed");
                ApiError::InvalidTaskQuery("task command operation failed")
            }
        }
    }
}
```

### `backend/src/app/error/response.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response.rs`
- Size bytes / Размер в байтах: `4600`
- Included characters / Включено символов: `4600`
- Truncated / Обрезано: `no`

```rust
mod ai;
mod communication;
mod communications;
mod document_processing;
mod integrations;
mod knowledge;
mod persons;
mod platform;
mod review;
mod tasks;

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use serde::Serialize;

use super::types::ApiError;

pub(super) type ErrorParts = (StatusCode, &'static str, String, bool);

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error, message, authenticate) = parts(self);
        let mut response = (status, Json(ErrorResponse { error, message })).into_response();
        if authenticate {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::DatabaseNotConfigured
        | ApiError::SecretVaultNotConfigured
        | ApiError::HostVault(_)
        | ApiError::InvalidEnvelope(_)
        | ApiError::FailedPrecondition(_)
        | ApiError::Audit(_)
        | ApiError::Store(_)
        | ApiError::SettingNotFound
        | ApiError::Settings(_)
        | ApiError::SignalHub(_) => platform::parts(error),
        ApiError::Graph(_)
        | ApiError::InvalidGraphQuery(_)
        | ApiError::Projects(_)
        | ApiError::InvalidProjectQuery(_)
        | ApiError::InvalidProjectLinkReview(_)
        | ApiError::ProjectLinkTargetNotFound
        | ApiError::ProjectLinkReview(_)
        | ApiError::GraphNotFound
        | ApiError::ProjectNotFound
        | ApiError::NotFound => knowledge::parts(error),
        ApiError::InvalidTaskCandidateQuery(_)
        | ApiError::InvalidTaskCandidateReview(_)
        | ApiError::InvalidObligationQuery(_)
        | ApiError::InvalidObligationReview(_)
        | ApiError::InvalidDecisionQuery(_)
        | ApiError::InvalidDecisionReview(_)
        | ApiError::InvalidRelationshipQuery(_)
        | ApiError::InvalidRelationshipReview(_)
        | ApiError::InvalidContradictionQuery(_)
        | ApiError::InvalidContradictionReview(_)
        | ApiError::InvalidReviewQuery(_)
        | ApiError::InvalidReviewItem(_)
        | ApiError::TaskCandidateNotFound
        | ApiError::TaskCandidate(_)
        | ApiError::ObligationNotFound
        | ApiError::Obligation(_)
        | ApiError::DecisionNotFound
        | ApiError::Decision(_)
        | ApiError::RelationshipNotFound
        | ApiError::Relationship(_)
        | ApiError::ContradictionObservationNotFound
        | ApiError::Consistency(_)
        | ApiError::ReviewItemNotFound
        | ApiError::ReviewInbox(_)
        | ApiError::ReviewPromotion(_) => review::parts(error),
        ApiError::InvalidTaskQuery(_) => tasks::parts(error),
        ApiError::InvalidPersonaQuery(_)
        | ApiError::InvalidPersonIdentityReview(_)
        | ApiError::PersonIdentityNotFound
        | ApiError::PersonProjection(_)
        | ApiError::PersonIdentity(_) => persons::parts(error),
        ApiError::Messages(_)
        | ApiError::CommunicationIngestion(_)
        | ApiError::CommunicationStorage(_)
        | ApiError::InvalidCommunicationQuery(_)
        | ApiError::EmailAccountDeleteConflict
        | ApiError::ProviderWriteConfirmationRequired
        | ApiError::CommunicationMessageNotFound
        | ApiError::AccountSetup(_)
        | ApiError::AccountSetupState
        | ApiError::AccountSetupPendingGrantNotFound
        | ApiError::AccountSetupStateMismatch => communication::parts(error),
        ApiError::InvalidDocumentProcessingQuery(message) => {
            document_processing::invalid_query_parts(message)
        }
        ApiError::DocumentProcessing(error) => document_processing::parts(error),
        ApiError::AiRunNotFound => ai::ai_run_not_found_parts(),
        ApiError::Ai(error) => ai::ai_error_parts(error),
        ApiError::AiControlCenter(error) => ai::control_center_error_parts(error),
        ApiError::Telegram(error) => integrations::telegram_error_parts(error),
        ApiError::WhatsappWeb(error) => integrations::whatsapp_web_error_parts(error),
        ApiError::Zoom(error) => integrations::zoom_error_parts(error),
        ApiError::YandexTelemost(error) => integrations::yandex_telemost_error_parts(error),
        ApiError::Automation(error) => integrations::automation_error_parts(error),
        ApiError::Call(error) => integrations::call_error_parts(error),
    }
}
```

### `backend/src/app/error/response/ai.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/ai.rs`
- Size bytes / Размер в байтах: `479`
- Included characters / Включено символов: `479`
- Truncated / Обрезано: `no`

```rust
mod control_center;
mod runtime;

use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;

use super::ErrorParts;

pub(super) fn ai_run_not_found_parts() -> ErrorParts {
    runtime::ai_run_not_found_parts()
}

pub(super) fn ai_error_parts(error: AiError) -> ErrorParts {
    runtime::ai_error_parts(error)
}

pub(super) fn control_center_error_parts(error: AiControlCenterError) -> ErrorParts {
    control_center::control_center_error_parts(error)
}
```

### `backend/src/app/error/response/ai/control_center.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/ai/control_center.rs`
- Size bytes / Размер в байтах: `2577`
- Included characters / Включено символов: `2577`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::ai::control_center::AiControlCenterError;

use super::super::ErrorParts;

pub(super) fn control_center_error_parts(error: AiControlCenterError) -> ErrorParts {
    match error {
        AiControlCenterError::ProviderNotFound => {
            not_found("ai_provider_not_found", "AI provider was not found")
        }
        AiControlCenterError::ModelNotFound => {
            not_found("ai_model_not_found", "AI model was not found")
        }
        AiControlCenterError::PromptNotFound => {
            not_found("ai_prompt_not_found", "AI prompt was not found")
        }
        AiControlCenterError::PromptVersionNotFound => not_found(
            "ai_prompt_version_not_found",
            "AI prompt version was not found",
        ),
        AiControlCenterError::InvalidRequest(_)
        | AiControlCenterError::EmptyField { .. }
        | AiControlCenterError::SecretLikePayload => (
            StatusCode::BAD_REQUEST,
            "invalid_ai_control_center_request",
            error.to_string(),
            false,
        ),
        AiControlCenterError::HostVault(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "host_vault_error",
            error.to_string(),
            false,
        ),
        AiControlCenterError::ObservationStore(error) => {
            tracing::error!(error = %error, "AI control center observation trail operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_control_center_observation_error",
                "AI control center observation trail operation failed".to_owned(),
                false,
            )
        }
        AiControlCenterError::SecretReference(error) => {
            tracing::error!(error = %error, "AI control center secret reference operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_secret_reference_error",
                "AI provider secret reference operation failed".to_owned(),
                false,
            )
        }
        AiControlCenterError::Sqlx(error) => {
            tracing::error!(error = %error, "AI control center store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_control_center_error",
                "AI control center operation failed".to_owned(),
                false,
            )
        }
    }
}

fn not_found(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::NOT_FOUND, error, message.to_owned(), false)
}
```

### `backend/src/app/error/response/ai/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/ai/runtime.rs`
- Size bytes / Размер в байтах: `2982`
- Included characters / Включено символов: `2982`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::ai::core::AiError;

use super::super::ErrorParts;

pub(super) fn ai_run_not_found_parts() -> ErrorParts {
    (
        StatusCode::NOT_FOUND,
        "ai_run_not_found",
        "AI run was not found".to_owned(),
        false,
    )
}

pub(super) fn ai_error_parts(error: AiError) -> ErrorParts {
    match error {
        AiError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_ai_request",
            message.to_owned(),
            false,
        ),
        AiError::UnknownAgent(agent_id) => (
            StatusCode::BAD_REQUEST,
            "unknown_ai_agent",
            format!("unknown AI agent `{agent_id}`"),
            false,
        ),
        AiError::RunNotFound => ai_run_not_found_parts(),
        AiError::Runtime(error) => (
            StatusCode::BAD_GATEWAY,
            "ai_runtime_error",
            error.to_string(),
            false,
        ),
        AiError::InvalidEmbeddingDimension { .. } => (
            StatusCode::BAD_GATEWAY,
            "invalid_embedding_dimension",
            "embedding provider returned an unexpected vector dimension".to_owned(),
            false,
        ),
        AiError::Json(error) => (
            StatusCode::BAD_GATEWAY,
            "ai_provider_json_error",
            error.to_string(),
            false,
        ),
        AiError::InvalidSourceKind(value) => {
            tracing::error!(source_kind = %value, "AI runtime saw invalid semantic source kind");
            internal_runtime_error()
        }
        AiError::EventEnvelope(error) => {
            tracing::error!(error = %error, "AI runtime operation failed");
            internal_runtime_error()
        }
        AiError::EventStore(error) => {
            tracing::error!(error = %error, "AI event store operation failed");
            internal_runtime_error()
        }
        AiError::PersonaAttribution(error) => {
            tracing::error!(error = %error, "AI persona attribution operation failed");
            internal_runtime_error()
        }
        AiError::PersonaAttributionUnavailable => {
            tracing::error!("AI persona attribution port was not configured");
            internal_runtime_error()
        }
        AiError::ReviewInboxWorkflow(error) => {
            tracing::error!(error = %error, "AI review inbox mirroring failed");
            internal_runtime_error()
        }
        AiError::ObservationStore(error) => {
            tracing::error!(error = %error, "AI observation trail operation failed");
            internal_runtime_error()
        }
        AiError::Sqlx(error) => {
            tracing::error!(error = %error, "AI database operation failed");
            internal_runtime_error()
        }
    }
}

fn internal_runtime_error() -> ErrorParts {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "ai_runtime_error",
        "AI runtime operation failed".to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/communication.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/communication.rs`
- Size bytes / Размер в байтах: `3113`
- Included characters / Включено символов: `3113`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use super::super::types::ApiError;
use super::{ErrorParts, communications};

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::Messages(error) => internal_store(
            error,
            "communication message API store operation failed",
            "communication_message_store_error",
            "communication message store operation failed",
        ),
        ApiError::CommunicationIngestion(error) => internal_store(
            error,
            "communication account API store operation failed",
            "communication_account_store_error",
            "communication account store operation failed",
        ),
        ApiError::CommunicationStorage(error) => internal_store(
            error,
            "communication attachment API store operation failed",
            "communication_attachment_store_error",
            "communication attachment store operation failed",
        ),
        ApiError::InvalidCommunicationQuery(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_communication_query",
            message.to_owned(),
            false,
        ),
        ApiError::EmailAccountDeleteConflict => (
            StatusCode::CONFLICT,
            "email_account_delete_conflict",
            "email account has retained communication evidence and cannot be deleted".to_owned(),
            false,
        ),
        ApiError::ProviderWriteConfirmationRequired => (
            StatusCode::BAD_REQUEST,
            "provider_write_confirmation_required",
            "explicit provider write confirmation is required".to_owned(),
            false,
        ),
        ApiError::CommunicationMessageNotFound => (
            StatusCode::NOT_FOUND,
            "communication_message_not_found",
            "communication message was not found".to_owned(),
            false,
        ),
        ApiError::AccountSetup(error) => communications::account_setup_error_parts(error),
        ApiError::AccountSetupState => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "account_setup_state_error",
            "account setup state is unavailable".to_owned(),
            false,
        ),
        ApiError::AccountSetupPendingGrantNotFound => (
            StatusCode::NOT_FOUND,
            "account_setup_pending_grant_not_found",
            "pending Gmail OAuth setup was not found".to_owned(),
            false,
        ),
        ApiError::AccountSetupStateMismatch => (
            StatusCode::BAD_REQUEST,
            "account_setup_state_mismatch",
            "Gmail OAuth state does not match the pending setup".to_owned(),
            false,
        ),
        _ => unreachable!("communication response mapper received non-communication ApiError"),
    }
}

fn internal_store(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
    message: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        message.to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/communications.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/communications.rs`
- Size bytes / Размер в байтах: `765`
- Included characters / Включено символов: `765`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::integrations::mail::accounts::EmailAccountSetupError;

use super::ErrorParts;

pub(super) fn account_setup_error_parts(error: EmailAccountSetupError) -> ErrorParts {
    let status = if matches!(
        error,
        EmailAccountSetupError::InvalidRequest { .. }
            | EmailAccountSetupError::MissingProviderField { .. }
    ) {
        StatusCode::BAD_REQUEST
    } else {
        tracing::error!(error = %error, "account setup failed");
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (
        status,
        "account_setup_error",
        if status == StatusCode::BAD_REQUEST {
            error.to_string()
        } else {
            "account setup failed".to_owned()
        },
        false,
    )
}
```

### `backend/src/app/error/response/document_processing.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/document_processing.rs`
- Size bytes / Размер в байтах: `2374`
- Included characters / Включено символов: `2374`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::domains::documents::processing::DocumentProcessingError;

use super::ErrorParts;

pub(super) fn invalid_query_parts(message: &'static str) -> ErrorParts {
    (
        StatusCode::BAD_REQUEST,
        "invalid_document_processing_query",
        message.to_owned(),
        false,
    )
}

pub(super) fn parts(error: DocumentProcessingError) -> ErrorParts {
    let (status, message) = match error {
        DocumentProcessingError::InvalidLimit => (
            StatusCode::BAD_REQUEST,
            "document processing limit must be between 1 and 100",
        ),
        DocumentProcessingError::EmptyField(_)
        | DocumentProcessingError::InvalidStep(_)
        | DocumentProcessingError::InvalidStatus(_)
        | DocumentProcessingError::InvalidArtifactKind(_) => (
            StatusCode::BAD_REQUEST,
            "invalid document processing request payload",
        ),
        DocumentProcessingError::DocumentNotFound | DocumentProcessingError::JobNotFound => (
            StatusCode::NOT_FOUND,
            "document processing job was not found",
        ),
        DocumentProcessingError::RetryRequiresFailedJob => (
            StatusCode::BAD_REQUEST,
            "document processing retry requires a failed job",
        ),
        DocumentProcessingError::RetryCommandConflict => (
            StatusCode::CONFLICT,
            "document processing retry command conflicts with existing event",
        ),
        DocumentProcessingError::EventStore(error) if error.is_unique_violation() => (
            StatusCode::CONFLICT,
            "document processing retry command conflicts with existing event",
        ),
        DocumentProcessingError::ObservationStore(error) => {
            tracing::error!(error = %error, "document processing observation trail failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "document processing observation trail failed",
            )
        }
        _ => {
            tracing::error!(error = %error, "document processing store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "document processing store operation failed",
            )
        }
    };

    (
        status,
        "document_processing_store_error",
        message.to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/integrations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/integrations.rs`
- Size bytes / Размер в байтах: `1046`
- Included characters / Включено символов: `1046`
- Truncated / Обрезано: `no`

```rust
mod automation;
mod call;
mod telegram;
mod whatsapp;
mod yandex_telemost;
mod zoom;

use crate::application::provider_runtime_contracts::{
    TelegramError, WhatsappWebError, YandexTelemostError, ZoomError,
};
use crate::engines::automation::AutomationError;
use crate::platform::calls::CallError;

use super::ErrorParts;

pub(super) fn telegram_error_parts(error: TelegramError) -> ErrorParts {
    telegram::telegram_error_parts(error)
}

pub(super) fn whatsapp_web_error_parts(error: WhatsappWebError) -> ErrorParts {
    whatsapp::whatsapp_web_error_parts(error)
}

pub(super) fn zoom_error_parts(error: ZoomError) -> ErrorParts {
    zoom::zoom_error_parts(error)
}

pub(super) fn yandex_telemost_error_parts(error: YandexTelemostError) -> ErrorParts {
    yandex_telemost::yandex_telemost_error_parts(error)
}

pub(super) fn automation_error_parts(error: AutomationError) -> ErrorParts {
    automation::automation_error_parts(error)
}

pub(super) fn call_error_parts(error: CallError) -> ErrorParts {
    call::call_error_parts(error)
}
```

### `backend/src/app/error/response/integrations/automation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/integrations/automation.rs`
- Size bytes / Размер в байтах: `1779`
- Included characters / Включено символов: `1779`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::engines::automation::AutomationError;

use super::super::ErrorParts;

pub(super) fn automation_error_parts(error: AutomationError) -> ErrorParts {
    match error {
        AutomationError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_automation_request",
            message,
            false,
        ),
        AutomationError::PolicyNotFound => (
            StatusCode::NOT_FOUND,
            "automation_policy_not_found",
            "automation policy was not found".to_owned(),
            false,
        ),
        AutomationError::PolicyDisabled
        | AutomationError::ChatNotAllowed
        | AutomationError::MissingTemplateVariable(_)
        | AutomationError::UndeclaredTemplateVariable(_) => (
            StatusCode::FORBIDDEN,
            "automation_policy_denied",
            error.to_string(),
            false,
        ),
        AutomationError::EventEnvelope(error) => (
            StatusCode::BAD_REQUEST,
            "invalid_automation_event",
            error.to_string(),
            false,
        ),
        AutomationError::EventStore(error) => {
            internal(error, "automation event store operation failed")
        }
        AutomationError::ObservationStore(error) => {
            internal(error, "automation observation store operation failed")
        }
        AutomationError::Sqlx(error) => internal(error, "automation database operation failed"),
    }
}

fn internal(error: impl std::fmt::Display, log: &'static str) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "automation_store_error",
        "automation operation failed".to_owned(),
        false,
    )
}
```
