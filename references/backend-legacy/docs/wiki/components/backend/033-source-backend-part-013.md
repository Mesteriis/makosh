---
chunk_id: 033-source-backend-part-013
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 033-source-backend-part-013 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Текущий чанк предоставляет исходники HTTP-роутера и обработчиков backend-компонента. Wiki-страницу `components/backend.md` необходимо наполнить на основе встроенных файлов: описать структуру роутера, группы маршрутов, интеграционные обработчики (WhatsApp, Yandex Telemost, Zoom), вспомогательный слой Signal Hub и механизм сверки хранилища (vault reconciliation). Каждое утверждение должно подкрепляться буквальным текстом источников.

## Предложенные страницы

#### `components/backend.md`

```markdown
# Компонент Backend

## Обзор

Backend Макошь — HTTP-сервер на Rust (фреймворк `axum`), обслуживающий REST API и WebSocket-подключения. Точка входа — `backend/src/app/router.rs`.

## Запуск и логирование

Публичная точка входа — функция `run(config: AppConfig)`. Она:

- соединяется с базой данных через `Database::connect(config.database_url())`;
- привязывается к адресу `config.http_addr()`;
- строит роутер вызовом `build_router_with_database(config, database)`.

Логирование настраивается функцией `init_tracing`. Формат определяется переменной окружения `MAKOSH_LOG_FORMAT` (значения `"json"` или `"plain"`). Фильтр инициализируется из переменной `RUST_LOG` с fallback-уровнем `info`.

## Состояние приложения (`AppState`)

```rust
pub(crate) struct AppState {
    pub(crate) config: AppConfig,
    pub(crate) database: Database,
    pub(crate) vault: HostVault,
    pub(crate) account_setup: AccountSetupState,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: EventBus,
}
```

- `config` — конфигурация приложения (`AppConfig`).
- `database` — абстракция над подключением к БД (пул `sqlx::PgPool`).
- `vault` — локальное зашифрованное хранилище (`HostVault`).
- `account_setup` — состояние ожидающих OAuth-грантов (Gmail, Zoom, Telegram) через `Arc<Mutex<HashMap<…>>>`, а также `PendingQrLoginMap`.
- `telegram_runtime` — менеджер рантайма Telegram (`TelegramRuntimeManager`).
- `event_bus` — шина событий (`EventBus`).

В процессе построения роутера хранилище разблокируется вызовом `vault.unlock_existing()`, и запускается фоновая сверка манифеста хранилища (`spawn_host_vault_manifest_reconciliation`).

## Роутер и композиция маршрутов

Роутер строится в `build_router_with_database` и объединяет:

1. Публичные маршруты — `routes::public_routes()`:
   - `/healthz` — статус OK.
   - `/readyz` — статус готовности (OK или `SERVICE_UNAVAILABLE` с детализацией `DatabaseReadiness` / `MigrationReadiness`).
   - `/api/v1/integrations/mail/accounts/gmail/oauth/callback`.
   - `/api/v1/communications/messages/:message_id/remote-image`.

2. Connect RPC-маршруты — `crate::app::connectrpc::protected_routes(…)`.

3. Защищённые API-маршруты — `routes::protected_routes(api_secret)` с применением middleware `guard::require_secret`, проверяющего заголовок `x-makosh-secret`.

CORS разрешает только локальные источники:
- схема `http`/`https` на `localhost`, `127.0.0.1`, `::1`;
- `tauri.localhost` (схема `http`/`https`);
- схема `tauri` на `localhost`.

## Группы защищённых маршрутов

В `backend/src/app/router/routes/mod.rs` перечислены модули, каждый из которых добавляет свою группу маршрутов:

- **status_vault** — статус приложения и управление хранилищем: `/api/v1/status`, `/api/v1/vault/*`.
- **communications** — работа с сообщениями: поиск, потоки, метки, папки, черновики, шаблоны, вложения, сертификаты, переводы, AI-ответы, аналитика (весь префикс `/api/v1/communications`).
- **knowledge** — граф знаний, проекты, обработка документов (`/api/v1/graph`, `/api/v1/projects`, `/api/v1/documents`).
- **persons** — персоны и персонажи: CRUD, идентификаторы, роли, заметки, таймлайн, обогащение, досье (`/api/v1/persons`, `/api/v1/personas`; ADR-0084).
- **calendar** — календарь: аккаунты, события, участники, заметки, транскрипты, дедлайны, фокус-блоки, аналитика, правила, синхронизация (`/api/v1/calendar`).
- **organizations** — организации: структура, контакты, обогащение, порталы, контракты, риски, досье (`/api/v1/organizations`).
- **tasks** — задачи: подзадачи, связи, чеклисты, провайдеры, шаблоны, кандидаты, аналитика (`/api/v1/tasks`).
- **review** — инбокс ревью, обязательства, решения, связи, противоречия (`/api/v1/review`, `/api/v1/obligations`, `/api/v1/decisions`, `/api/v1/relationships`, `/api/v1/contradictions`).
- **settings** — настройки приложения и аккаунтов (`/api/v1/settings`).
- **signal_hub** — источники сигналов, профили, соединения, состояния рантаймов, проверки здоровья, политики (`/api/v1/signal-hub`).
- **ai** — AI-провайдеры, модели, промпты, агенты, runs, ответы, подготовка к встречам (`/api/v1/ai`).
- **messaging** — интеграции с Telegram, WhatsApp, Zoom, Yandex Telemost: аккаунты, возможности, рантайм-мосты, синхронизация, вебхуки (префиксы `/api/v1/integrations/telegram`, `/whatsapp`, `/zoom`, `/yandex-telemost`).
- **email_accounts** — почтовые аккаунты (Gmail OAuth, IMAP, синхронизация) (`/api/v1/integrations/mail/accounts`).
- **audit_events** — аудит-события, WebSocket-потоки (`/api/v1/audit/events`, `/api/events/ws`, `/api/events/realtime/ws`, `/api/events/stream`).

Модуль `routes/support.rs` централизованно подключает все необходимые функции‑обработчики из `crate::app::handlers::*`, `crate::ai::api::*`, `crate::app::api_support::*` и соответствующих модулей провайдеров.

## Интеграции с провайдерами

### WhatsApp (`whatsapp.rs`)

Обработчики для управления WhatsApp-аккаунтами, статусом рантайма, логином (QR, pair‑code), синхронизацией чатов, истории, участников, статусов, присутствия, звонков, контактов и медиа. Структуры запросов/ответов: `WhatsappAccountSummary`, `WhatsAppChatSyncItem`, `WhatsAppHistorySyncResponse`, `WhatsAppMembersSyncItem`, `WhatsAppStatusSyncResponse`, `WhatsAppPresenceSyncItem`, `WhatsAppCallsSyncItem`, `WhatsAppContactsSyncItem`. Поддерживаются обе формы — WhatsApp Web (multi‑device) и WhatsApp Business Cloud (валидация подписи `x-hub-signature-256`, ручка вебхука и proxy‑manifest). Входящие runtime‑bridge сообщения (обычные, обновления, удаления, квитанции о доставке) сохраняются через `message_store` с проверкой безопасности вложений через `AttachmentSafetyScanStatus`. Все операции используют внутренние секретные ссылки (`whatsapp_secret_reference_store`). Для событий ведётся монотонный счётчик `WHATSAPP_EVENT_SEQUENCE` (`AtomicU64`).

### Yandex Telemost (`yandex_telemost.rs`)

Обработчики возможностей (`YandexTelemostCapabilitiesResponse`, включая политики локальной записи и спикер-таймлайна), управления аккаунтами, статуса рантайма, очистки записей по сроку хранения. Операции с конференциями: создание, получение, обновление (`PATCH`), просмотр участников (cohosts) с пагинацией. Генерация манифеста веб-вью для открытия конференции в компаньоне, а также интента на локальную запись (с consent-флагом). Runtime‑bridge: приём аудиозаписей и транскриптов с последующим сохранением через `ObservationStore` и интеграцию в конвейер `realtime_conversation_memory_pipeline` / `realtime_conversation_radar_projection`.

### Zoom (`zoom.rs`)

Детальные возможности сервиса (`ZoomCapabilitiesResponse`) с перечислением статусов каждой capability (например, `accounts.fixture`, `auth.oauth_user`, `bridge.meetings`, `retention.cleanup`). Управление аккаунтами: OAuth‑потоки (старт/завершение пользовательского OAuth, server‑to‑server авторизация, обновление токенов, плановое обслуживание токенов). Синхронизация облачных записей встреч (с проверкой настроек приватности `privacy.zoom_remote_recording_download_enabled` и `privacy.zoom_remote_transcript_download_enabled`), импорт транскрипт‑файлов (VTT, SRT, plain‑text). Приём вебхуков с валидацией подписи (`x-zm-signature`, `x-zm-request-timestamp`). Очистка записей и управление импортированными блобами (`POST .../remove`). Runtime‑bridge: встречи, записи, транскрипты. Поддержка фикстурных аккаунтов (доступно только при включённых фикстурных маршрутах).

## Signal Hub поддержка

Модуль `signal_hub_support.rs` связывает аккаунты провайдеров с системой Signal Hub:

- `provider_signal_source_code` маппит `CommunicationProviderKind` на код источника:
  - `Gmail`/`Icloud`/`Imap` → `mail`
  - `TelegramUser`/`TelegramBot` → `telegram`
  - `WhatsappWeb`/`WhatsappBusinessCloud` → `whatsapp`
  - `ZoomUser`/`ZoomServerToServer` → `zoom`
  - `YandexTelemostUser` → `yandex_telemost`
- `sync_provider_account_signal_connection` обновляет соединение Signal Hub на основе состояния аккаунта (connected/disconnected).
- Для WhatsApp статус определяется через `WhatsAppRuntimeStatus`, и соединение синхронизируется с дополнительными настройками (`merged_whatsapp_runtime_connection_settings`) и секретной ссылкой сессии.
- `run_signal_hub_health_check` для источника `ai` проверяет доступность AI-рантайма (Ollama или OmniRoute): инициализирует клиент через `ai_runtime_client_from_settings`, проверяет доступность `chat_model` и `embedding_model`, формирует снапшот здоровья (`SignalHealthSnapshotWrite`) со статусом `healthy` или `degraded`.

## Сверка хранилища (Vault Reconciliation)

При старте роутера вызывается `spawn_host_vault_manifest_reconciliation`. Логика восстановления календарных аккаунтов описана в `vault_reconciliation/calendar_restore.rs`:

- Для Gmail (`EmailProviderKind::Gmail`) восстанавливается Google Calendar‑аккаунт (идентификатор вида `google-calendar:{account_id}`) при наличии секрета.
- Для iCloud (`EmailProviderKind::Icloud`) восстанавливается Apple iCloud Calendar‑аккаунт (идентификатор вида `icloud-calendar:{account_id}`), если секрет имеет purpose `ImapPassword`.

## Примечания (ADR)

- **ADR-0073** (комментарий в `router.rs`): app router владеет HTTP-композицией; группы маршрутов вынесены в целенаправленные модули, чтобы регистрация эндпойнтов оставалась аудируемой без единого «god‑файла».
- **ADR-0084** (комментарий в `routes/persons.rs`): продублированы legacy‑маршруты `/api/v1/persons` и нативно именованные `/api/v1/personas` для перехода на сущность «персона» (persona).
```

## Покрытие источников

| Файл | Использованные факты |
|------|-----------------------|
| `backend/src/app/provider_runtime_handlers/whatsapp.rs` | Множество структур запросов/ответов WhatsApp (AccountSummary, ChatSyncItem, HistorySyncResponse, MembersSyncItem и т.д.), счётчик последовательности событий `WHATSAPP_EVENT_SEQUENCE`, ссылки на сервисы `communication_provider_account_store`, `whatsapp_secret_reference_store`, `message_store`, `AttachmentSafetyScanStatus`. |
| `backend/src/app/provider_runtime_handlers/yandex_telemost.rs` | `YandexTelemostCapabilitiesResponse` с политиками записи и таймлайна, обработчики конференций, веб-вью, интента записи, runtime‑bridge для записей и транскриптов, интеграция с `ObservationStore`, `realtime_conversation_memory_pipeline`, `realtime_conversation_radar_projection`. |
| `backend/src/app/provider_runtime_handlers/zoom.rs` | `ZoomCapabilitiesResponse` с детальным статусом capabilities, OAuth‑потоки, синхронизация записей с проверкой privacy‑настроек, управление импортами и удалением, валидация подписи вебхуков (заголовки `x-zm-signature`, `x-zm-request-timestamp`), поддержка фикстур. |
| `backend/src/app/router.rs` | `run`, `init_tracing`, `build_router`, `build_router_with_database`, публичные эндпойнты `/healthz` и `/readyz`, CORS-слой с разрешёнными локальными origin, `AppState`, запуск фоновой сверки хранилища, ADR-0073 в комментарии. |
| `backend/src/app/router/routes/ai.rs` | Маршруты AI: `/api/v1/ai/*` (статус, провайдеры, модели, промпты, агенты, runs, ответы). |
| `backend/src/app/router/routes/audit_events.rs` | Маршруты аудит-событий и WebSocket: `/api/v1/audit/events`, `/api/events/ws`, `/api/events/realtime/ws` и т.д. |
| `backend/src/app/router/routes/calendar.rs` | Обширный API календаря: аккаунты, события, участники, напоминания, аналитика, дедлайны, фокус-блоки, правила, синхронизация. |
| `backend/src/app/router/routes/communications.rs` | Коммуникационный API: сообщения, потоки, поиск, папки, шаблоны, вложения, сертификаты, AI‑ответы, переводы, аналитика. |
| `backend/src/app/router/routes/email_accounts.rs` | Почтовые аккаунты: Gmail OAuth, IMAP, синхронизация, экспорт. |
| `backend/src/app/router/routes/knowledge.rs` | Граф знаний, проекты, обработка документов. |
| `backend/src/app/router/routes/messaging.rs` | Интеграции Telegram, WhatsApp, Zoom, Yandex Telemost (capabilities, аккаунты, рантайм, синхронизация, вебхуки, runtime‑bridge). |
| `backend/src/app/router/routes/mod.rs` | Композиция защищённых маршрутов из 14 модулей, применение middleware `require_secret`. |
| `backend/src/app/router/routes/organizations.rs` | API организаций: структура, обогащение, контракты, риски, досье. |
| `backend/src/app/router/routes/persons.rs` | API персон/персонажей (legacy + persona), идентификационные кандидаты, обогащение, досье; ADR‑0084 в комментарии. |
| `backend/src/app/router/routes/public.rs` | Публичные маршруты: `/healthz`, `/readyz`, Gmail OAuth callback, отдача удалённых изображений. |
| `backend/src/app/router/routes/review.rs` | Инбокс ревью, обязательства, решения, связи, противоречия. |
| `backend/src/app/router/routes/settings.rs` | Настройки приложения и аккаунтов. |
| `backend/src/app/router/routes/signal_hub.rs` | Источники сигналов, профили, соединения, проверки здоровья, политики, фикстуры. |
| `backend/src/app/router/routes/status_vault.rs` | Статус приложения и операции с хранилищем: создание, разблокировка, экспорт/импорт восстановления. |
| `backend/src/app/router/routes/support.rs` | Централизованный импорт всех функций‑обработчиков из `handlers::*`, `ai::api::*`, `api_support::*` и провайдерных модулей. |
| `backend/src/app/router/routes/tasks.rs` | API задач: CRUD, чеклисты, подзадачи, провайдеры, шаблоны, кандидаты, аналитика. |
| `backend/src/app/signal_hub_support.rs` | Маппинг `provider_signal_source_code` для `CommunicationProviderKind`, синхронизация соединений Signal Hub, проверка здоровья AI‑рантайма (Ollama/OmniRoute) через `ai_runtime_health_snapshot`. |
| `backend/src/app/state.rs` | Структура `AppState` и `AccountSetupState`. |
| `backend/src/app/vault_reconciliation.rs` | Реэкспорт `spawn_host_vault_manifest_reconciliation`. |
| `backend/src/app/vault_reconciliation/calendar_restore.rs` | Восстановление Google Calendar и iCloud Calendar‑аккаунтов из секретов почтовых провайдеров. |

## Исходные файлы

- [`backend/src/app/provider_runtime_handlers/whatsapp.rs`](../../../../backend/src/app/provider_runtime_handlers/whatsapp.rs)
- [`backend/src/app/provider_runtime_handlers/yandex_telemost.rs`](../../../../backend/src/app/provider_runtime_handlers/yandex_telemost.rs)
- [`backend/src/app/provider_runtime_handlers/zoom.rs`](../../../../backend/src/app/provider_runtime_handlers/zoom.rs)
- [`backend/src/app/router.rs`](../../../../backend/src/app/router.rs)
- [`backend/src/app/router/routes/ai.rs`](../../../../backend/src/app/router/routes/ai.rs)
- [`backend/src/app/router/routes/audit_events.rs`](../../../../backend/src/app/router/routes/audit_events.rs)
- [`backend/src/app/router/routes/calendar.rs`](../../../../backend/src/app/router/routes/calendar.rs)
- [`backend/src/app/router/routes/communications.rs`](../../../../backend/src/app/router/routes/communications.rs)
- [`backend/src/app/router/routes/email_accounts.rs`](../../../../backend/src/app/router/routes/email_accounts.rs)
- [`backend/src/app/router/routes/knowledge.rs`](../../../../backend/src/app/router/routes/knowledge.rs)
- [`backend/src/app/router/routes/messaging.rs`](../../../../backend/src/app/router/routes/messaging.rs)
- [`backend/src/app/router/routes/mod.rs`](../../../../backend/src/app/router/routes/mod.rs)
- [`backend/src/app/router/routes/organizations.rs`](../../../../backend/src/app/router/routes/organizations.rs)
- [`backend/src/app/router/routes/persons.rs`](../../../../backend/src/app/router/routes/persons.rs)
- [`backend/src/app/router/routes/public.rs`](../../../../backend/src/app/router/routes/public.rs)
- [`backend/src/app/router/routes/review.rs`](../../../../backend/src/app/router/routes/review.rs)
- [`backend/src/app/router/routes/settings.rs`](../../../../backend/src/app/router/routes/settings.rs)
- [`backend/src/app/router/routes/signal_hub.rs`](../../../../backend/src/app/router/routes/signal_hub.rs)
- [`backend/src/app/router/routes/status_vault.rs`](../../../../backend/src/app/router/routes/status_vault.rs)
- [`backend/src/app/router/routes/support.rs`](../../../../backend/src/app/router/routes/support.rs)
- [`backend/src/app/router/routes/tasks.rs`](../../../../backend/src/app/router/routes/tasks.rs)
- [`backend/src/app/signal_hub_support.rs`](../../../../backend/src/app/signal_hub_support.rs)
- [`backend/src/app/state.rs`](../../../../backend/src/app/state.rs)
- [`backend/src/app/vault_reconciliation.rs`](../../../../backend/src/app/vault_reconciliation.rs)
- [`backend/src/app/vault_reconciliation/calendar_restore.rs`](../../../../backend/src/app/vault_reconciliation/calendar_restore.rs)

## Кандидаты на drift

Нет видимых расхождений между кодом и документацией, так как текущее содержимое wiki-страницы `components/backend.md` не было предоставлено в этом контекстном пакете.
