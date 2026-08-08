### Summary / Резюме

Необходимо создать (или обновить) русскую вики-страницу `components/backend.md`, описывающую архитектуру бэкенда Макошь на основе предоставленных исходников. В чанк вошли ключевые модули: маршрутизация и аутентификация ConnectRPC, сервисы Communications и Signal Hub, каталог возможностей WhatsApp, а также структура обработки ошибок (типы, преобразования, HTTP-ответы). Документация должна отражать только факты, прямо подтверждённые кодом в данном контексте; все обрезанные файлы явно помечаются как неполные.

### Proposed pages / Предлагаемые страницы

`components/backend.md`:

```markdown
# Бэкенд (Backend)

## Обзор

Бэкенд Макошь реализован на Rust с использованием [`axum`](https://crates.io/crates/axum) как HTTP-фреймворка и [`connectrpc`](https://crates.io/crates/connectrpc) для gRPC-подобных сервисов. Все запросы проходят через middleware-аутентификацию по секрету API и направляются в зарегистрированные RPC-сервисы. Централизованная обработка ошибок конвертирует доменные ошибки в HTTP-статусы и JSON-тела ответов.

## Маршрутизация и аутентификация

Функция `protected_routes` (файл `backend/src/app/connectrpc.rs`) создаёт основной маршрутизатор `axum::Router<AppState>`:

1. В `ConnectRouter` регистрируются сервисы `communications` и `signal_hub`.
2. Полученный `ConnectRouter` устанавливается как `fallback_service`.
3. Применяется middleware `guard::require_secret`, который требует наличие корректного секрета API (значение `api_secret` передаётся в состоянии middleware).

```rust
// app/connectrpc.rs (сокращено)
pub(crate) fn protected_routes(
    pool: Option<PgPool>,
    config: AppConfig,
    api_secret: String,
) -> Router<AppState> {
    let connect_router = signal_hub::register(
        communications::register(ConnectRouter::new(), pool.clone(), config.clone()),
        pool, config,
    );
    Router::<AppState>::new()
        .fallback_service(connect_router.into_axum_router().into_service())
        .layer(middleware::from_fn_with_state(api_secret, guard::require_secret))
}
```

Подключение к PostgreSQL передаётся через `PgPool`.

## Сервисы ConnectRPC

### Communications Service

Модуль `connectrpc/communications.rs` реализует трейт `CommunicationsService` и предоставляет RPC-эндпоинты для работы с коммуникациями (почта, сообщения):

- **Папки:** `ListFolders`, `CreateFolder`, `UpdateFolder`, `DeleteFolder`, `ListFolderMessages`, `CopyMessageToFolder`, `MoveMessageToFolder`
- **Черновики:** `ListDrafts`, `CreateDraft`, `DeleteDraft`
- **Сообщения:** `ListMessages`, `GetMessage`, `SendMessage`, `MarkMessageRead`, `SnoozeMessage`, `BulkMessageAction`, `ToggleMessagePin`, `ToggleMessageImportant`, `ToggleMessageMute`, `DeleteMessageFromProvider`, `RedirectMessage`, `TransitionMessageWorkflowState`
- **Поиск и аналитика:** `SearchMessages`, `ListThreads`, `ListTopSenders`, `GetMailboxHealth`, `ListSubscriptions`
- **Шаблоны:** `ListRichTemplates`, `UpsertRichTemplate`, `DeleteRichTemplate`, `RichTemplateRender`, `RichTemplateMailMergePreview`
- **AI-функции:** `AiReply`, `AiReplyVariants`, `AnalyzeMessage`, `ExplainMessage`, `ExtractMessageNotes`, `ExtractMessageTasks`, `DetectMessageLanguage`
- **Аутбокс:** `ListOutbox`, `UndoOutboxItem`
- **Вложения:** `AttachmentSearch`, `GetAttachmentPreview`, `GetAttachmentArchiveInspection`, `TranslateAttachment`
- **Безопасность:** `GetMessageAuth`, `GetMessageSignature`
- **Экспорт:** `GetMessageExport`

Сервис хранит ссылки на множество доменных хранилищ: `MessageProjectionStore`, `EmailAnalyticsStore`, `CommunicationThreadStore`, `CommunicationDraftStore`, `CommunicationOutboxStore`, `CommunicationStorageStore`, `AttachmentSearchStore`, `CommunicationSavedSearchStore`, `CommunicationFolderStore` и др. Полный файл (`156976` байт) обрезан в данном контексте, поэтому перечень не исчерпывающий.

### Signal Hub Service

Модуль `connectrpc/signal_hub.rs` реализует трейт `SignalHubService` и предоставляет RPC для управления Signal Hub:

- **Источники:** `ListSources`, `GetSource`, `ListFixtureSources`, `EnableSource`, `DisableSource`
- **Подключения:** `ListConnections`, `CreateConnection`, `UpdateConnection`, `RemoveConnection`
- **Профили:** `ListProfiles`, `CreateProfile`, `UpdateProfile`, `RemoveProfile`, `ApplyProfile` — профили задают политики маршрутизации сигналов с указанием `SignalPolicyScope`, `SignalPolicyMode`, `source_code`, `connection_id`, `event_pattern`
- **Политики:** `ListPolicies`, `CreatePolicy`
- **Сигналы:** `EnableSignals`, `DisableSignals`, `PauseSignals`, `ResumeSignals`, `MuteSignals`, `UnmuteSignals`
- **Воспроизведение:** `ListReplayRequests`, `RequestReplay` — управление воспроизведением событий
- **Здоровье:** `ListHealth`, `RunHealthCheck`
- **Состояния runtime:** `ListRuntimeStates`, `UpdateRuntimeState`
- **Фикстуры:** `EmitFixtureSignal`, `RestoreSystemFixture`
- **Возможности:** `ListCapabilities`

Сервис внутренне использует `SignalHubStore`, `SignalHubCapabilityService`, `SignalFixtureSourceService`, `SignalHubConnectionService`, `SignalHubControlService`, `SignalHubProfileService`, `SignalHubReplayService`. Файл также обрезан.

### WhatsApp Capability Catalog

Статический список возможностей WhatsApp определён в `api_support/whatsapp_capability_catalog.rs`. Каждая запись (`WhatsappCapabilityStatus`) содержит:

- `code` — строковый идентификатор
- `category` — категория (runtime, sessions, auth, sync, messages, search, media, conversations, status, presence)
- `state` — `Available` (доступно), `Degraded` (ограничено/заблокировано живое выполнение)
- `action_class` — `Read`, `ProviderWrite`, `SecretAccess`, `Destructive`
- `description` — краткое описание
- `confirmation_required` — требуется ли явное подтверждение для записи
- `enabled` — включена ли возможность

Ниже представлена таблица на основе видимой части файла (полный файл также обрезан после 12000 символов):

| Код | Категория | Состояние | Класс | Описание | Подтверждение | Включено |
|-----|-----------|-----------|-------|----------|:---:|:---:|
| `runtime.fixture` | runtime | Available | Read | Фикстурный WhatsApp runtime, CI/валидация | нет | да |
| `sessions.manual_state` | sessions | Available | Read | Метаданные сессий в PostgreSQL, без секретов | нет | да |
| `sessions.restore` | sessions | Degraded | SecretAccess | Восстановление через host vault, только fixture/runtime-safe путь | нет | да |
| `auth.qr_link_start` | auth | Degraded | ProviderWrite | Состояние QR и события есть, но реальный QR не генерируется | да | да |
| `auth.pair_code_link_start` | auth | Degraded | ProviderWrite | Состояние пары кодов и события есть, но код не генерируется | да | да |
| `sync.chats` | sync | Available | Read | Проекции чатов синхронизируются через fixture/runtime-safe путь | нет | да |
| `sync.history` | sync | Available | Read | История сообщений доступна через общую модель Communications | нет | да |
| `messages.read_projection` | messages | Available | Read | Канонические чтения Communications обслуживают WhatsApp-сообщения | нет | да |
| `search.messages` | search | Available | Read | Провайдер-нейтральный поиск возвращает данные WhatsApp | нет | да |
| `search.media` | search | Available | Read | Провайдер-нейтральный поиск возвращает вложения WhatsApp | нет | да |
| `messages.send_text` | messages | Degraded | ProviderWrite | Durable outbox и сверка с фикстурами есть; живое выполнение заблокировано | да | да |
| `messages.reply` | messages | Degraded | ProviderWrite | Ответы через outbox; живое выполнение заблокировано | да | да |
| `messages.forward` | messages | Degraded | ProviderWrite | Пересылка через outbox; живое выполнение заблокировано | да | да |
| `messages.edit` | messages | Degraded | ProviderWrite | Проекция версий и сверка есть; живое редактирование заблокировано | да | да |
| `messages.delete` | messages | Degraded | Destructive | Tombstones и сверка есть; живое удаление заблокировано | да | да |
| `messages.react` | messages | Degraded | ProviderWrite | Реакции через outbox; живое выполнение заблокировано | да | да |
| `messages.unreact` | messages | Degraded | ProviderWrite | Удаление реакций через тот же путь; заблокировано | да | да |
| `media.upload_send` | media | Degraded | ProviderWrite | Данные blob/hash сохраняются; живая отправка заблокирована | да | да |
| `media.download` | media | Degraded | Read | Загрузка медиа аналогично; живое выполнение заблокировано | нет | нет |
| `media.voice_send` | media | Degraded | ProviderWrite | Голосовые заметки через outbox; заблокированы | да | да |
| `conversations.join_group` | conversations | Degraded | ProviderWrite | Команды входа в группу долговременны и сверены; заблокированы | да | да |
| `conversations.leave_group` | conversations | Degraded | ProviderWrite | Команды выхода аналогично; заблокированы | да | да |
| `conversations.archive` | conversations | Degraded | ProviderWrite | Состояние диалога сверено с фикстурами; живое выполнение заблокировано | да | да |
| `conversations.unarchive` | conversations | Degraded | ProviderWrite | Аналогично архивной команде | да | да |
| `conversations.mute` | conversations | Degraded | ProviderWrite | Аналогично | да | да |
| `conversations.unmute` | conversations | Degraded | ProviderWrite | Аналогично | да | да |
| `conversations.pin` | conversations | Degraded | ProviderWrite | Аналогично | да | да |
| `conversations.unpin` | conversations | Degraded | ProviderWrite | Аналогично | да | да |
| `conversations.mark_read` | conversations | Degraded | ProviderWrite | Команда чтения сверена; заблокирована | нет | да |
| `conversations.mark_unread` | conversations | Degraded | ProviderWrite | Команда непрочтения сверена; заблокирована | нет | да |
| `status.observe` | status | Available | Read | Статусы WhatsApp проецируются в Communications и Timeline | нет | да |
| `status.publish` | status | Degraded | ProviderWrite | Публикация статуса через outbox и сверку; заблокирована | да | да |
| `presence.observe` | presence | Available | Read | (строка обрезана, точное описание не подтверждено) | нет | да |

*Примечание: файл обрезан, поэтому таблица неполна. Последующие строки каталога не видны в данном контексте.*

## Обработка ошибок

Модуль `app::error` включает:

- **Типы ошибок** (`error/types.rs`): перечисление `ApiError` с вариантами, сгруппированными по доменам.
- **Преобразования** (`error/conversions/`): реализации `From<DomainError> for ApiError` для каждого домена.
- **HTTP-ответы** (`error/response/`): функция `parts` сопоставляет `ApiError` с `(StatusCode, error_code, message, authenticate)`, и реализует `IntoResponse` для `axum`.

### Структура ApiError

Основные группы вариантов `ApiError` (из файла `error/response.rs`):

- **Platform:** `DatabaseNotConfigured`, `SecretVaultNotConfigured`, `HostVault`, `InvalidEnvelope`, `Store`, `Settings`, `SignalHub`, `Audit`, `SettingNotFound`, `FailedPrecondition`
- **Knowledge:** `Graph`, `Projects`, `ProjectLinkReview`, `ProjectLinkTargetNotFound`, `GraphNotFound`, `ProjectNotFound`, `NotFound`
- **Review:** `TaskCandidate`, `Obligation`, `Decision`, `Relationship`, `ContradictionObservationNotFound`, `Consistency`, `ReviewItemNotFound`, `ReviewInbox`, `ReviewPromotion`
- **Tasks:** `InvalidTaskQuery`
- **Persons:** `PersonIdentity`, `PersonProjection`, `InvalidPersonaQuery`, `PersonIdentityNotFound`, `InvalidPersonIdentityReview`
- **Communication:** `Messages`, `CommunicationIngestion`, `CommunicationStorage`, `CommunicationMessageNotFound`, `AccountSetup`, `AccountSetupState`, `AccountSetupPendingGrantNotFound`, `AccountSetupStateMismatch`, `ProviderWriteConfirmationRequired`, `EmailAccountDeleteConflict`, `InvalidCommunicationQuery`
- **Documents:** `DocumentProcessing`, `InvalidDocumentProcessingQuery`
- **AI:** `Ai`, `AiRunNotFound`, `AiControlCenter`
- **Integrations:** `Telegram`, `WhatsappWeb`, `Zoom`, `YandexTelemost`, `Automation`, `Call`

### Преобразования доменных ошибок

Каждая подпапка `conversions` содержит преобразования для соответствующего домена. Например:

- `conversions/communications.rs`: преобразует `CommunicationIngestionError`, `MessageProjectionError`, `CommunicationStorageError`, `CommunicationThreadError`, `EmailIntelligenceError`, `CommunicationDraftError`, `CommunicationOutboxError`, `BulkMessageActionError`, `CommunicationSavedSearchError`, `CommunicationFolderError`, `CommunicationAiStateError`, `CommunicationReadReceiptError`, `CommunicationTemplateError`, `CommunicationDeliveryNotificationError`, `CommunicationFinanceError`, `EmailAnalyticsError`, `CommunicationPersonaError`, `IndexEmailError`, `MessageFlagsError`, `SubscriptionError` (файл обрезан, список неполный).
- `conversions/ai.rs`: преобразует `AiError`, `AiControlCenterError`, `OllamaError`, `OmniRouteError`, `AiRuntimeError`.
- `conversions/calendar.rs`: преобразует `CalendarCoreError`, `MeetingsError`, `SchedulingError`, `CalendarHealthError`, `CalendarBrainError`, `ReminderError`, `CalendarRuleError`, `CalendarError`, `CalendarCommandServiceError`, `CalendarMeetingOutcomeApplicationError`.
- `conversions/knowledge.rs`: преобразует ошибки графов, проектов, обзоров (кандидаты задач, обязательства, решения, отношения, противоречия).
- `conversions/organizations.rs`: преобразует `OrgCoreError`, `OrgMemoryError`, `OrgWorkflowError`, `OrgFinanceError`, `OrgEnrichmentError`, `OrgHealthError`, `InvestigatorError`, `OrganizationError`, `OrganizationCommandServiceError`, `OrganizationContactLinkApplicationError`.
- `conversions/persons.rs`: преобразует `PersonIdentityError`, `PersonProjectionError`, `PersonEnrichmentError`, `PersonMemoryError`, `PersonCoreError`, `PersonCommandServiceError`.
- `conversions/platform.rs`: преобразует `EventEnvelopeError`, `EventStoreError`, `SettingsError`, `SignalHubError`, `ApiAuditError`, `HostVaultError`.
- `conversions/tasks.rs`: преобразует `TaskError`, `TaskCoreError`, `TaskHealthError`, `TaskRuleError`, `TaskBrainError`, `TaskCommandServiceError`.
- `conversions/documents.rs`: преобразует `DocumentProcessingError`, `DocumentProcessingCommandServiceError`.
- `conversions/integrations.rs`: преобразует `TelegramError`, `WhatsappWebError`, `ZoomError`, `YandexTelemostError`, `AutomationError`, `CallError`, `CommunicationFixtureIngestError`, `TelegramMessageWriteError`, `ReviewInboxWorkflowError`.

Все преобразования для ненайденных сущностей обычно возвращают `ApiError::NotFound`; внутренние ошибки логируются и возвращают обобщённые коды (`invalid_*_query` и т.п.).

### HTTP-ответы

Файл `error/response.rs` и подмодули определяют функцию `parts`, возвращающую `(StatusCode, &str, String, bool)` для каждого варианта `ApiError`. Примеры:

- `ApiError::InvalidCommunicationQuery(message)` → `(400, "invalid_communication_query", message, false)`
- `ApiError::CommunicationMessageNotFound` → `(404, "communication_message_not_found", "communication message was not found", false)`
- `ApiError::ProviderWriteConfirmationRequired` → `(400, "provider_write_confirmation_required", "explicit provider write confirmation is required", false)`
- `ApiError::AiRunNotFound` → `(404, "ai_run_not_found", "AI run was not found", false)`
- `ApiError::Ai(AiError::Runtime(_))` → `(502, "ai_runtime_error", error_text, false)`
- `ApiError::Automation(AutomationError::PolicyNotFound)` → `(404, "automation_policy_not_found", "automation policy was not found", false)`
- Ошибки хранения (`Store`, `CommunicationStorage`, `Messages`, `CommunicationIngestion`) → `(500, соответствующий код, общее сообщение, false)`, с предварительным логированием `tracing::error!`.
- Ошибки настройки email-аккаунтов (`AccountSetupStateMismatch`) → `(400, "account_setup_state_mismatch", ...)`
- Ошибки валидации документов (`InvalidDocumentProcessingQuery`) → `(400, "invalid_document_processing_query", message, false)`

Ответ клиенту всегда JSON: `{ "error": "<код>", "message": "<сообщение>" }`. При флаге `authenticate: true` добавляется заголовок `WWW-Authenticate: Bearer`.

## Примечание о полноте

В данный чанк вошли обрезанные версии файлов `communications.rs`, `signal_hub.rs`, `whatsapp_capability_catalog.rs` и `conversions/communications.rs`. Их полное содержание недоступно, поэтому документация по этим модулям может быть неполной.
```

### Source coverage / Покрытие источников

- **`backend/src/app/api_support/whatsapp_capability_catalog.rs`** (обрезан): каталог возможностей WhatsApp — структура `WhatsappCapabilityStatus`, таблица с кодами, состояниями, классами действий и описаниями.
- **`backend/src/app/connectrpc.rs`**: функция `protected_routes` — регистрация сервисов `communications` и `signal_hub` в `ConnectRouter`, middleware `guard::require_secret`.
- **`backend/src/app/connectrpc/communications.rs`** (обрезан): структура `CommunicationsConnectService`, перечень RPC-методов сервиса `CommunicationsService`, используемые доменные хранилища.
- **`backend/src/app/connectrpc/signal_hub.rs`** (обрезан): структура `SignalHubConnectService`, перечень RPC-методов `SignalHubService`, используемые сервисы и хранилища.
- **`backend/src/app/error.rs`**: публичные элементы модуля `error` — `ApiError`, `AppError`.
- **`backend/src/app/error/conversions.rs`**: список доменных подмодулей преобразований.
- **`backend/src/app/error/conversions/ai.rs`**: преобразования `AiError`, `AiControlCenterError`, `OllamaError`, `OmniRouteError`, `AiRuntimeError` в `ApiError`.
- **`backend/src/app/error/conversions/calendar.rs`**: преобразования ошибок календаря: `CalendarCoreError`, `MeetingsError`, `SchedulingError`, `CalendarHealthError`, `CalendarBrainError`, `ReminderError`, `CalendarRuleError`, `CalendarError`, `CalendarCommandServiceError`, `CalendarMeetingOutcomeApplicationError`.
- **`backend/src/app/error/conversions/communications.rs`** (обрезан): преобразования ошибок коммуникаций: `CommunicationIngestionError`, `MessageProjectionError`, `CommunicationStorageError`, `CommunicationThreadError`, `EmailIntelligenceError`, `CommunicationDraftError`, `CommunicationOutboxError`, `BulkMessageActionError`, `CommunicationSavedSearchError`, `CommunicationFolderError`, `CommunicationAiStateError`, `CommunicationReadReceiptError`, `CommunicationTemplateError`, `CommunicationDeliveryNotificationError`, `CommunicationFinanceError`, `EmailAnalyticsError`, `CommunicationPersonaError`, `IndexEmailError`, `MessageFlagsError`, `SubscriptionError`.
- **`backend/src/app/error/conversions/documents.rs`**: преобразования `DocumentProcessingError`, `DocumentProcessingCommandServiceError`.
- **`backend/src/app/error/conversions/integrations.rs`**: преобразования ошибок интеграций: `TelegramError`, `WhatsappWebError`, `ZoomError`, `YandexTelemostError`, `AutomationError`, `CallError`, `CommunicationFixtureIngestError`, `TelegramMessageWriteError`, `ReviewInboxWorkflowError`.
- **`backend/src/app/error/conversions/knowledge.rs`** (обрезан): преобразования ошибок доменов знаний: `GraphStoreError`, `ProjectLinkReviewError`, `ProjectStoreError`, `TaskCandidateError`, `ObligationStoreError`, `DecisionStoreError`, `RelationshipStoreError`, `DecisionCommandServiceError`, `ObligationCommandServiceError`, `RelationshipCommandServiceError`, `DecisionReviewApplicationError`, `ObligationReviewApplicationError`, `RelationshipReviewApplicationError`, `TaskCandidateReviewApplicationError`, `ProjectLinkReviewServiceError`, `TaskCandidateReviewServiceError`.
- **`backend/src/app/error/conversions/organizations.rs`**: преобразования ошибок организаций: `OrgCoreError`, `OrgMemoryError`, `OrgWorkflowError`, `OrgFinanceError`, `OrgEnrichmentError`, `OrgHealthError`, `InvestigatorError`, `OrganizationError`, `OrganizationCommandServiceError`, `OrganizationContactLinkApplicationError`.
- **`backend/src/app/error/conversions/persons.rs`**: преобразования ошибок персон: `PersonIdentityError`, `PersonProjectionError`, `PersonEnrichmentError`, `PersonMemoryError`, `PersonCoreError`, `PersonCommandServiceError`.
- **`backend/src/app/error/conversions/platform.rs`**: преобразования платформенных ошибок: `EventEnvelopeError`, `EventStoreError`, `SettingsError`, `SignalHubError`, `ApiAuditError`, `HostVaultError`.
- **`backend/src/app/error/conversions/tasks.rs`**: преобразования ошибок задач: `TaskError`, `TaskCoreError`, `TaskHealthError`, `TaskRuleError`, `TaskBrainError`, `TaskCommandServiceError`.
- **`backend/src/app/error/response.rs`**: функция `parts` и реализация `IntoResponse` для `ApiError` — сопоставление вариантов с группами (platform, knowledge, review, tasks, persons, communication, documents, ai, integrations).
- **`backend/src/app/error/response/ai.rs`**, **`ai/control_center.rs`**, **`ai/runtime.rs`**: HTTP-части для AI-ошибок; коды 404, 502, 500.
- **`backend/src/app/error/response/communication.rs`**: HTTP-части для коммуникационных ошибок (400, 404, 409, 500).
- **`backend/src/app/error/response/communications.rs`**: HTTP-части для ошибок настройки email-аккаунтов (`EmailAccountSetupError`).
- **`backend/src/app/error/response/document_processing.rs`**: HTTP-части для ошибок обработки документов (400, 404, 409, 500).
- **`backend/src/app/error/response/integrations.rs`** и вложенные модули (`automation`, `call`, `telegram`, `whatsapp`, `yandex_telemost`, `zoom`): маршрутизация и HTTP-части для ошибок интеграций; пример `automation.rs` с кодами 400, 404, 403, 500.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом и документацией не видны. Текущее содержимое вики-страницы `components/backend.md` не включено в чанк, поэтому сравнить с существующей документацией невозможно. Часть файлов обрезана после 12000 символов — это ограничение полноты покрытия, а не drift.
