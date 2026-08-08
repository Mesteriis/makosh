---
chunk_id: 155-source-frontend-part-015
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 155-source-frontend-part-015 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/frontend.md` отсутствует в русской wiki. На основе предоставленных исходных файлов платформенного слоя фронтенда (реальное время, конфигурация, Connect-клиенты, трейсинг событий, загрузка медиа) необходимо создать её описание. Страница документирует основные подсистемы фронтенда makosh: патчинг кешей реального времени, инвалидацию запросов, загрузку медиафайлов в Telegram, переменные окружения, gRPC-клиенты и трейсинг событий.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Фронтенд (платформенные компоненты)

Платформенный слой фронтенда (`frontend/src/platform/`) содержит общие компоненты, не привязанные к конкретному провайдеру коммуникаций. Сюда входят: обработка событий реального времени (патчинг кеша и инвалидация через React Query), конфигурация окружения, Connect-клиенты для gRPC-Web, панель трейсинга событий и рабочий процесс загрузки медиафайлов.

## Обработка событий реального времени

Функция `handleRealtimeEvent` (из `frontend/src/platform/bootstrap/realtime.ts`) мутирует кеш `queryClient` (React Query) в ответ на события, полученные через SSE или WebSocket. Обработка делится на два механизма: **патчинг кеша** (немедленное обновление данных через `setQueryData`) и **инвалидация** (помечает запросы как устаревшие через `invalidateQueries`).

### Патчинг кеша (Telegram)

На основе тестов `realtimeTelegramCachePatches.test.ts`, `realtimeTelegramCommandPatches.test.ts`, `realtimeTelegramMediaCachePatches.test.ts`, `realtimeTelegramParticipantPatches.test.ts`, `realtimeTelegramProviderChatUpdates.test.ts`, `realtimeTelegramTopicPatches.test.ts` и `realtimeTelegramMembersSync.test.ts`:

- **typing.changed**: обновляет `metadata.active_typing` в кеше чатов и деталей чата (sender_id, action, is_active, expires_at).
- **chat.updated (unread)**: обновляет `metadata.unread_count`, `provider_unread_count`, `last_read_inbox_provider_message_id`.
- **folders.updated**: полностью заменяет список папок в кеше `[communications, telegram, folders]`.
- **reaction.changed**: обновляет `metadata.reaction_summary.reactions` — добавляет/изменяет реакцию с эмодзи и счётчиком.
- **message.updated**: патчит `metadata.lifecycle.latest_version_number`, `metadata.is_pinned`, а также вставляет сообщение в кеш закреплённых сообщений и поисковую выдачу.
- **message.created**: вставляет (upsert) новое сообщение в кеш сообщений чата.
- **command.status_changed**: обновляет поля команды (status, retry_count, last_error, next_attempt_at, dead_lettered_at), а также обрабатывает `telegram.command.reconciled` (completed/failed, reconciliation_status, provider_state, result_payload). Команды вставляются только в те кеши, которые соответствуют фильтрам по `provider_chat_id`, `provider_message_id` и `command_kind` (`realtimeTelegramCommandQueryFilters.test.ts`).
- **media.download.started / progress / failed / downloaded**: патчит `metadata.attachments` в кеше сообщений и элементы медиа-поиска (`communications.telegram.search.media`) — обновляет `download_state`, `tdlib_file_id`, `expected_size_bytes`, `downloaded_size_bytes`, `local_path` и др.
- **participant.updated**: вставляет (upsert) участника в кеш `chat-members` или удаляет его из кеша, если статус `absent_exhaustive` или `left`.
- **chat.updated (provider)**: патчит метаданные чата при обновлении настроек уведомлений (`is_muted`), позиции чата (`is_archived`, `is_pinned`, `provider_folder_id`) и меток папок (`folder_labels`, `folder_name`, `provider_folder_id`).
- **topic.updated**: заменяет или добавляет элемент топика в кеш топиков и топик-поиска.
- **sync.completed (members)**: обновляет runtime-статус Telegram-аккаунта полями `last_sync_scope`, `last_sync_status`, `last_synced_count`, `last_sync_provider_chat_id`.

### Патчинг кеша (WhatsApp)

На основе тестов `realtimeWhatsAppCachePatches.test.ts` (файл обрезан, включено ~12000 символов):

- **dialog.updated**: обновляет `metadata` диалога (is_pinned, is_archived, is_muted, is_unread, unread_count, participant_count) в списке диалогов и деталях.
- **reaction.changed**: обновляет `metadata.reaction_summary.reactions` (reaction, count).
- **session.link_state_changed** и **runtime.status_changed**: обновляет `link_state` сессии и статус рантайма, а также `metadata.runtime_status` и `metadata.runtime_status_source` в кеше сессий.
- **command.status_changed**: обновляет поля команды (status, reconciliation_status, provider_observed_at, completed_at).
- **presence.changed** и **call.updated**: патчит данные синхронизации присутствия и звонков в кешах `sync-presence` и `sync-calls` (presence_state, display_name, call_state и др.).

### Патчинг кеша (Zoom)

Для Zoom в предоставленных источниках определена только инвалидация (`realtimeZoomInvalidation.test.ts`). Патчинг кеша не показан.

### Инвалидация запросов

События реального времени также вызывают `queryClient.invalidateQueries` для определённых ключей запросов. Правила инвалидации описаны в тестах `*Invalidation.test.ts`.

#### Telegram

| Событие | Инвалидируемые ключи запросов |
| --- | --- |
| `telegram.message.created` | `communications.telegram.messages`, `communications.telegram.chats` |
| `telegram.sync.progress` | `communications.telegram.chats`, `communications.telegram.messages`, `integrations.telegram.runtime` |
| `telegram.command.status_changed` | `communications.telegram.messages`, `integrations.telegram.runtime`, `integrations.telegram.commands` |
| `telegram.media.download.progress` | `communications.telegram.messages`, `communications.telegram.search.media` |
| `telegram.media.upload.failed` | `integrations.telegram.commands`, `integrations.telegram.runtime` |
| `telegram.typing.changed` | `communications.telegram.chats`, `integrations.telegram.runtime` |
| `telegram.participant.updated` | `communications.telegram.chat-members`, `communications.telegram.chats` |
| `telegram.chat.updated` (позиция/метки папок) | `communications.telegram.folders`, `communications.telegram.chats` |

#### WhatsApp

| Событие | Инвалидируемые ключи запросов |
| --- | --- |
| `whatsapp.message.created` | `communications.whatsapp.messages` |
| `whatsapp.dialog.updated` | `communications.whatsapp.conversations`, `communications.whatsapp.conversation-detail`, `communications.whatsapp.messages` |
| `whatsapp.runtime.status_changed` | `integrations.whatsapp.sessions`, `integrations.whatsapp.capabilities`, `integrations.whatsapp.account-capabilities`, `integrations.whatsapp.runtime.status`, `integrations.whatsapp.runtime.health` |
| `whatsapp.media.download.progress` | `integrations.whatsapp.commands`, `integrations.whatsapp.sessions`, `integrations.whatsapp.capabilities`, `integrations.whatsapp.account-capabilities`, `integrations.whatsapp.runtime.status`, `integrations.whatsapp.runtime.health`, `integrations.whatsapp.runtime.sync-media` |
| `whatsapp.participant.changed` | `communications.whatsapp.conversations`, `communications.whatsapp.conversation-detail`, `communications.whatsapp.chat-members`, `integrations.whatsapp.runtime.sync-contacts` |
| `whatsapp.presence.changed` | `integrations.whatsapp.runtime.sync-presence` |
| `whatsapp.call.updated` | `integrations.whatsapp.runtime.sync-calls` |
| `whatsapp.status.updated` | `integrations.whatsapp.runtime.sync-statuses` |
| `whatsapp.sync.completed` (scope=history) | `integrations.whatsapp.runtime.sync-chats`, `integrations.whatsapp.runtime.sync-history`, `integrations.whatsapp.runtime.sync-members` |

#### Zoom

| Событие | Инвалидируемые ключи запросов |
| --- | --- |
| `zoom.transcript.observed` | `integrations.zoom.accounts`, `integrations.zoom.capabilities`, `integrations.zoom.runtime.status`, `integrations.zoom.webhook-subscriptions`, `integrations.zoom.provider-calls`, `integrations.zoom.provider-call-transcript`, `integrations.zoom.recording-imports`, `integrations.zoom.audit-events` |

## Загрузка медиафайлов в Telegram

Реализована в `frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.ts`.

- **`useTelegramMediaUploadMutation`** — мутация `@tanstack/vue-query`, которая вызывает `uploadTelegramMediaFile`. При успехе вызывает `primeTelegramUploadCommandQueues` и инвалидирует runtime и очереди команд Telegram.
- **`primeTelegramUploadCommandQueues`** — обходит все активные кеши команд Telegram (по ключу `['integrations', 'telegram', 'commands']`) и вставляет синтетическую команду `send_media` со статусом `queued`. Использует функцию `patchTelegramCommandList` для проверки необходимости вставки.
- **`telegramMediaTypeForFile`** — маппинг MIME-типов файла на `TelegramMediaUploadKind`:
  - `image/gif` → `animation`
  - `image/*` → `photo`
  - `video/*` → `video`
  - `audio/*` → `audio`
  - остальное → `document`
- **`uploadTelegramMediaFile`** — валидирует входные данные через Zod-схему, конвертирует файл в Base64, импортирует вложение через `importCommunicationAttachment`, затем вызывает `uploadTelegramMedia`.

## Переменные окружения фронтенда

Определены в `frontend/src/platform/config/env.ts`.

Функция `loadFrontendConfig` принимает объект `EnvSource` (по умолчанию `import.meta.env`) и возвращает `FrontendConfig`:

| Поле | Переменная окружения | Значение по умолчанию |
| --- | --- | --- |
| `apiBaseUrl` | `VITE_MAKOSH_API_BASE_URL` | `http://127.0.0.1:8080` |
| `apiSecret` | `VITE_MAKOSH_LOCAL_API_SECRET` | обязательно (иначе ошибка) |
| `sseUrl` | `VITE_MAKOSH_SSE_URL` | `{apiBaseUrl}/api/events/stream` |
| `webSocketUrl` | `VITE_MAKOSH_WEBSOCKET_URL` | формируется из `apiBaseUrl` с заменой протокола на `ws:`/`wss:` и путём `/api/events/ws` |
| `realtimeTransport` | `VITE_MAKOSH_REALTIME_TRANSPORT` | `'sse'`; значение `'websocket'` включает WebSocket |

Требование: `VITE_MAKOSH_LOCAL_API_SECRET` должен быть непустой строкой, иначе выбрасывается ошибка `"VITE_MAKOSH_LOCAL_API_SECRET is required"`.

## Connect-клиенты (gRPC-Web)

Расположены в `frontend/src/platform/connect/`.

- **`communicationsClient.ts`** — клиент для `CommunicationsService`. Создаётся через `@connectrpc/connect-web`, передаёт заголовок `X-Макошь-Secret` с секретом из `ApiClient.instance`. Синглтон, с функцией сброса для тестов (`resetCommunicationsConnectClientForTests`).
- **`signalHubClient.ts`** — клиент для `SignalHubService`. Аналогичная реализация.

Оба используют `useBinaryFormat: false`, что означает JSON-сериализацию.

## Трейсинг событий (Event Tracing)

Модуль `frontend/src/platform/event-tracing/` предоставляет API, React Query хуки и типы для просмотра трассировки событий Макошь.

### API

Функции в `api.ts` (вызывают `ApiClient.instance.get`):

- `fetchEventTraceByEventId(eventId, limit)` — `GET /api/v1/events/{eventId}/trace?limit={limit}`
- `fetchEventTraceByCorrelationId(correlationId, limit)` — `GET /api/v1/event-traces/{correlationId}?limit={limit}`
- `fetchEventChildren(eventId, limit)` — `GET /api/v1/events/{eventId}/children?limit={limit}`

Лимит ограничен диапазоном 1–1000 (по умолчанию 1000). Идентификаторы проверяются на непустоту.

### React Query хуки

В `queries.ts` определены:

- `useEventTraceByEventIdQuery(eventId, limit)` — ключ `['events', eventId, 'trace']`, `staleTime: 10_000`, включается только при непустом `eventId`.
- `useEventTraceByCorrelationIdQuery(correlationId, limit)` — ключ `['event-traces', correlationId]`.
- `useEventChildrenQuery(eventId, limit)` — ключ `['events', eventId, 'children']`.

Ключи запросов не содержат указаний на Telegram или WhatsApp — это подтверждено тестом в `queries.test.ts`.

### Типы

Описаны в `types.ts`:

- `EventEnvelope` — обёртка события с полями `event_id`, `event_type`, `occurred_at`, `payload` и др.
- `StoredEventEnvelope` — `EventEnvelope` с позицией (`position`).
- `EventTraceEdge` — связь родитель-потомок (`parent_event_id`, `child_event_id`).
- `EventConsumerAnnotation` — статус обработки события консьюмером.
- `EventDeadLetterAnnotation` — информация о dead-letter.
- `EventTrace` — полная трасса: `correlation_id`, `root_event_ids`, `events[]`, `edges[]`, `orphan_event_ids[]`, `missing_parent_ids[]`, `consumer_annotations[]`, `dead_letters[]`.

### Границы владения

Тест `EventTracePanel.boundary.test.ts` проверяет, что компонент `EventTracePanel.vue` не содержит ссылок на Telegram или WhatsApp и остаётся в границах платформенного трейсинга (содержит `consumer_annotations`, `dead_letters`, `missing_parent_ids`, но не `['telegram'`, `['whatsapp'` и пути `domains/telegram`, `domains/whatsapp`).

## Структура исходников

Основные пути (относительно репозитория):

- `frontend/src/platform/bootstrap/*` — обработка событий реального времени, инвалидация, загрузка медиа.
- `frontend/src/platform/config/env.ts` — конфигурация окружения.
- `frontend/src/platform/connect/*` — Connect-клиенты.
- `frontend/src/platform/event-tracing/*` — трейсинг событий.
```

## Покрытие источников

- `frontend/src/platform/bootstrap/realtimeTelegramCachePatches.test.ts` — патчинг typing, unread, folders, reactions, lifecycle, создание сообщений.
- `frontend/src/platform/bootstrap/realtimeTelegramCommandPatches.test.ts` — патчинг команд (retry, dead-letter, reconciliation, mismatch, вставка send_media).
- `frontend/src/platform/bootstrap/realtimeTelegramCommandQueryFilters.test.ts` — фильтрация вставки команд по ключам запросов.
- `frontend/src/platform/bootstrap/realtimeTelegramInvalidation.test.ts` — правила инвалидации Telegram.
- `frontend/src/platform/bootstrap/realtimeTelegramMediaCachePatches.test.ts` — патчинг загрузок медиа (started, progress, failed, completed).
- `frontend/src/platform/bootstrap/realtimeTelegramMembersSync.test.ts` — патчинг runtime при синхронизации участников.
- `frontend/src/platform/bootstrap/realtimeTelegramParticipantPatches.test.ts` — патчинг участников (upsert, удаление absent/left).
- `frontend/src/platform/bootstrap/realtimeTelegramProviderChatUpdates.test.ts` — патчинг чатов (уведомления, позиция, метки папок).
- `frontend/src/platform/bootstrap/realtimeTelegramTopicPatches.test.ts` — патчинг топиков.
- `frontend/src/platform/bootstrap/realtimeWhatsAppCachePatches.test.ts` — патчинг WhatsApp (диалоги, реакции, сессии, команды, присутствие, звонки).
- `frontend/src/platform/bootstrap/realtimeWhatsAppInvalidation.test.ts` — правила инвалидации WhatsApp.
- `frontend/src/platform/bootstrap/realtimeZoomInvalidation.test.ts` — правила инвалидации Zoom.
- `frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.test.ts` и `useTelegramMediaUploadWorkflow.ts` — маппинг типов файлов, мутация загрузки, вставка команд.
- `frontend/src/platform/config/env.test.ts` и `env.ts` — `loadFrontendConfig`, переменные окружения, умолчания, обязательный секрет.
- `frontend/src/platform/connect/communicationsClient.ts` — Connect-клиент для CommunicationsService.
- `frontend/src/platform/connect/signalHubClient.ts` — Connect-клиент для SignalHubService.
- `frontend/src/platform/event-tracing/api.ts` и `api.test.ts` — API трейсинга (fetch функций, лимиты).
- `frontend/src/platform/event-tracing/queries.ts` и `queries.test.ts` — React Query хуки и ключи.
- `frontend/src/platform/event-tracing/types.ts` — типы `EventTrace`, `EventEnvelope` и др.
- `frontend/src/platform/event-tracing/index.ts` — экспорты модуля.
- `frontend/src/platform/event-tracing/EventTracePanel.boundary.test.ts` — границы владения EventTracePanel.

## Исходные файлы

- [`frontend/src/platform/bootstrap/realtimeTelegramCachePatches.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramCachePatches.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramCommandPatches.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramCommandPatches.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramCommandQueryFilters.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramCommandQueryFilters.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramInvalidation.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramInvalidation.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramMediaCachePatches.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramMediaCachePatches.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramMembersSync.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramMembersSync.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramParticipantPatches.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramParticipantPatches.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramProviderChatUpdates.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramProviderChatUpdates.test.ts)
- [`frontend/src/platform/bootstrap/realtimeTelegramTopicPatches.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeTelegramTopicPatches.test.ts)
- [`frontend/src/platform/bootstrap/realtimeWhatsAppCachePatches.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeWhatsAppCachePatches.test.ts)
- [`frontend/src/platform/bootstrap/realtimeWhatsAppInvalidation.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeWhatsAppInvalidation.test.ts)
- [`frontend/src/platform/bootstrap/realtimeZoomInvalidation.test.ts`](../../../../frontend/src/platform/bootstrap/realtimeZoomInvalidation.test.ts)
- [`frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.test.ts`](../../../../frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.test.ts)
- [`frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.ts`](../../../../frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.ts)
- [`frontend/src/platform/config/env.test.ts`](../../../../frontend/src/platform/config/env.test.ts)
- [`frontend/src/platform/config/env.ts`](../../../../frontend/src/platform/config/env.ts)
- [`frontend/src/platform/connect/communicationsClient.ts`](../../../../frontend/src/platform/connect/communicationsClient.ts)
- [`frontend/src/platform/connect/signalHubClient.ts`](../../../../frontend/src/platform/connect/signalHubClient.ts)
- [`frontend/src/platform/event-tracing/EventTracePanel.boundary.test.ts`](../../../../frontend/src/platform/event-tracing/EventTracePanel.boundary.test.ts)
- [`frontend/src/platform/event-tracing/api.test.ts`](../../../../frontend/src/platform/event-tracing/api.test.ts)
- [`frontend/src/platform/event-tracing/api.ts`](../../../../frontend/src/platform/event-tracing/api.ts)
- [`frontend/src/platform/event-tracing/index.ts`](../../../../frontend/src/platform/event-tracing/index.ts)
- [`frontend/src/platform/event-tracing/queries.test.ts`](../../../../frontend/src/platform/event-tracing/queries.test.ts)
- [`frontend/src/platform/event-tracing/queries.ts`](../../../../frontend/src/platform/event-tracing/queries.ts)
- [`frontend/src/platform/event-tracing/types.ts`](../../../../frontend/src/platform/event-tracing/types.ts)

## Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно. Все утверждения основаны на встроенных исходных файлах (в основном тестах). Сами файлы реализации (`realtime.ts`, `*.vue`, `patch*` функции) в этом чанке не предоставлены, поэтому возможное расхождение с действительным поведением не может быть подтверждено или опровергнуто.
