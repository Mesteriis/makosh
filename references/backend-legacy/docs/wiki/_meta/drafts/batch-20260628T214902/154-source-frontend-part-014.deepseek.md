### Summary / Резюме

Страница `components/frontend.md` должна быть создана как опорная точка для всей фронтенд-части вики. Предлагается наполнить её обзором ключевых слоёв (API-клиент, реальное время, интеграции, запросы Vue Query, модульные границы) на основе предоставленных исходных файлов. Каждое утверждение подкреплено кодом из контекстного пакета.

### Proposed pages / Предлагаемые страницы

#### `components/frontend.md`

```markdown
# Фронтенд (Frontend)

## Обзор

Фронтенд Макошь построен как SPA на **Vue 3** с **Pinia** (состояние) и **@tanstack/vue-query** (запросы и кеширование). Точка входа — `main.ts`, которая инициализирует приложение, конфигурацию окружения, API-клиент и клиент реального времени (SSE/WebSocket).

## API-клиент (`ApiClient`)

Клиент реализован как синглтон с хранимым `baseUrl` и секретом. Инициализируется через `ApiClient.init(baseUrl, secret)` (секрет не может быть пустым) и предоставляет методы:

- `get<T>(path)`, `post<T>(path, body)`, `put<T>(path, body)`, `patch<T>(path, body)`, `delete<T>(path)`, `deleteWithBody<T>(path, body)`.

Каждый запрос добавляет заголовок `X-Макошь-Secret` с переданным секретом. При HTTP‑ошибках выбрасывается `ApiError` с полями `message`, `status`, `code`. Для статуса 204 No Content ответ интерпретируется как `undefined`.

Клиент используется всеми API-функциями интеграций, например, запросы к Zoom проходят через `ApiClient.instance.get`/`.post`.

Утилита `initializeApiClient(config: FrontendConfig)` из `platform/bootstrap/api.ts` вызывает `ApiClient.init(...)` и возвращает экземпляр для дальнейшей проверки в тестах.

## Интеграции

### Zoom

Типы (`types/zoom.ts`) описывают сущности учётной записи, рантайма, OAuth, вебхуков, событий аудита, записей встреч, транскриптов и пр. Ключевые интерфейсы:

- `ZoomAccount`, `ZoomRuntimeStatus`, `ZoomCapabilitiesResponse`.
- OAuth: `ZoomOAuthStartResponse`, `ZoomAuthorizationResult`, `ZoomTokenRefreshResult`, `ZoomTokenMaintenanceResult`.
- Синхронизация записей: `ZoomRecordingSyncResult`.
- Бриджинг: `ZoomMeetingObservationRequest` / `ZoomMeetingIngestResult`, `ZoomRecordingObservationRequest` / `ZoomRecordingIngestResult`, `ZoomTranscriptObservationRequest` / `ZoomTranscriptIngestResult`.

API-функции (`api/zoom.ts`) оборачивают соответствующие REST‑ручки (`/api/v1/integrations/zoom/...`). Среди них:

- `fetchZoomCapabilities()`, `fetchZoomAccounts()`, `setupZoomFixtureAccount()`, `setupZoomLiveAccount()`, `startZoomOAuth()`, `completeZoomOAuth()`, `authorizeZoomServerToServer()`, `refreshZoomToken()`, `maintainZoomTokens()`, `syncZoomRecordings()`, `fetchZoomWebhookSubscriptionStatus()`, `reconcileZoomWebhookSubscription()`, `removeZoomWebhookSubscription()`, `fetchZoomRuntimeStatus()`, `startZoomRuntime()`, `stopZoomRuntime()`, `removeZoomRuntime()`, `fetchZoomRecordingImports()`, `removeZoomRecordingImport()`, `fetchZoomAuditEvents()`, `cleanupZoomRetention()`, `fetchZoomProviderCalls()`, `fetchZoomCallTranscript()`, а также функции бриджинга: `bridgeZoomMeeting()`, `bridgeZoomRecording()`, `bridgeZoomTranscript()`, `importZoomTranscriptFile()`.

Слой запросов (`queries/useZoomRuntimeQuery.ts`) предоставляет Vue Query хуки (query/mutation) для всех перечисленных операций. При успешных мутациях выполняются инвалидации:
- `invalidateZoomRuntime` – сбрасывает кеш `accounts`, `capabilities`, `runtimeStatus`, `webhookSubscriptions`.
- `invalidateZoomDerived` – сбрасывает кеш `providerCalls`, `callTranscript`, `recordingImports`, `auditEvents`.

Ключи запросов заданы в `zoomQueryKeys.ts` и все начинаются с `['integrations', 'zoom', ...]`. Среди них: `accounts`, `capabilities`, `oauth`, `runtimeStatus`, `webhookSubscriptions`, `providerCalls`, `callTranscript`, `recordingImports`, `auditEvents`, `meetingsBridge`, `recordingsBridge`, `transcriptsBridge`, `transcriptFilesBridge`.

Вспомогательные функции для отображения доказательств (`zoomEvidence.ts`):
- `extractZoomRecordingRefs` фильтрует массив `recording_refs`, оставляя только объекты с непустым строковым `recording_id`.
- `formatZoomTranscriptProvenance` возвращает отсортированный JSON‑string от источника транскрипта; при отсутствии данных – `"—"`.

### YandexTelemost

Типы (`types/yandexTelemost.ts`) описывают возможности, политику локальной записи (отдельно для macOS, Linux, Windows с переменными `ffmpeg_path_env` и `ffmpeg_input_env`), учётные записи, создание/обновление конференций, манифест веб-представления, companion-открытие, сессии записи и бриджинг.

Примеры интерфейсов:
- `YandexTelemostCapabilitiesResponse` (провайдер `yandex_telemost_user`).
- `YandexTelemostLocalRecordingManifest` – политика записи и `consent_required`.
- `YandexTelemostConferenceCreateRequest`, `YandexTelemostConferenceOperationResponse`.
- `YandexTelemostRecordingSession`, `YandexTelemostRecordingBridgeRequest`.

В предоставленном чанке нет файлов с API‑функциями или хуками для YandexTelemost – только типы.

## Работа с реальным временем (Realtime bootstrap)

`platform/bootstrap/realtime.ts` инициализирует подключение к серверному потоку событий (SSE или WebSocket) и обрабатывает входящие сообщения.

- Транспорт выбирается полем `realtimeTransport` в `FrontendConfig`.
- SSE: `SseClientOptions` (url, longPollUrl, lastEventId, secret, onMessage…).
- WebSocket: `WebSocketClientOptions` (url, lastEventId, secret, onMessage…).
- При потере WebSocket‑соединения происходит автоматический переход на SSE. Ручной вызов `reconnect` возвращает предпочтение WebSocket.

Курсор воспроизведения (replay cursor) хранится в `localStorage` с ключом `makosh.realtime.lastEventId` и обновляется при каждом не‑`lagged` событии. Для событий `lagged` курсор не сдвигается, но вызывается инвалидация широкого набора запросов (почта, Telegram, WhatsApp, Zoom, Signal Hub).

Метод `handleRealtimeEvent` маршрутизирует события:

- `heartbeat` – игнорируется.
- `lagged` – инвалидация всех перечисленных ниже query‑ключей.
- По типу события (canonical `event_type`) выбираются целевые ключи для инвалидации и, при необходимости, применяются «патчи» к кешу без повторного запроса.

**Ключи инвалидации (основные группы)**:

| Группа | Пример ключей |
|--------|---------------|
| Общие communications | `['communications-list']`, `['communications-message']`, `['communications-ai-state']`, `['communications-saved-searches']`, … |
| Mail runtime | `['communications', 'mail', 'sync-statuses']`, `['communications', 'mail', 'mailbox-health']` |
| Telegram | `['integrations', 'telegram', 'runtime']`, `['communications', 'telegram', 'messages']`, … |
| WhatsApp | `['integrations', 'whatsapp', 'sessions']`, `['communications', 'whatsapp', 'conversations']`, … |
| Zoom | `zoomQueryKeys` – `accounts`, `capabilities`, `runtimeStatus`, `webhookSubscriptions`, `providerCalls`, `callTranscript`, `recordingImports`, `auditEvents` |
| Signal Hub | `['signal-hub']` |

**Патчи кеша (без доп. запросов)** применяются для событий:

- `mail.folder_message.moved` – перемещение сообщений между папками.
- `mail.saved_search.updated` – обновление сохранённого поиска.
- `mail.sync.progress` – обновление статуса синхронизации.
- `mail.draft.deleted` – удаление черновика.
- `mail.folder.updated` / `mail.folder.deleted` – изменение/удаление папки.
- `mail.outbox.delivery_status_changed` / `mail.read_receipt.recorded` – обновление метаданных отправленного письма.
- `mail.ai_state.changed` – обновление AI‑состояния сообщения.

Тесты (`realtime*.test.ts`) подтверждают как инвалидацию, так и прямые изменения кеша через `setQueryData`.

## Модульные границы

Тест `businessCommunicationOwnership.boundary.test.ts` фиксирует архитектурные соглашения:

1. Модули в `shared/communications` должны быть DTO‑only: не содержать прямых API‑вызовов, `useQuery`, `queryKey` или `fetch`.
2. Интеграции (`integrations/`) не импортируют бизнес‑модели коммуникаций (`shared/communications/*Business`), query‑ключи `communications` или эндпоинты `/api/v1/communications/conversations|messages|search|topics`.
3. Домен `domains/communications` не ссылается на эндпоинты управления провайдерами (`/api/v1/integrations/telegram|whatsapp|mail`).

Это гарантирует однонаправленную зависимость: UI‑интеграции → shared коммуникации → домены, без обратных ссылок.
```

### Source coverage / Покрытие источников

- `frontend/src/integrations/yandexTelemost/types/yandexTelemost.ts` — перечислены типы (`YandexTelemostCapabilitiesResponse`, `YandexTelemostLocalRecordingManifest`, `YandexTelemostConferenceCreateRequest`, `YandexTelemostRecordingSession`, `YandexTelemostRecordingBridgeRequest`) и отмечены особенности (`consent_required`, `ffmpeg_path_env`, `ffmpeg_input_env`).
- `frontend/src/integrations/zoom/api/zoom.test.ts` — подтверждены маршруты HTTP‑запросов для API‑функций Zoom; в вики упомянуто, что каждая функция использует `ApiClient.instance`.
- `frontend/src/integrations/zoom/api/zoom.ts` — документированы все публичные API‑функции и их пути; в вики перечислен список функций с их URL.
- `frontend/src/integrations/zoom/components/ZoomBridgeLab.boundary.test.ts` — зафиксировано, что компонент использует мутации бриджинга (а не `fetch`).
- `frontend/src/integrations/zoom/components/ZoomObservedCallsPanel.boundary.test.ts` — зафиксировано использование `useZoomProviderCallsQuery`, `useZoomCallTranscriptQuery` и вспомогательных функций.
- `frontend/src/integrations/zoom/components/ZoomSettingsPanel.boundary.test.ts` — подтверждено подключение вложенных панелей и наличие настроек `privacy.zoom_remote_transcript_download_enabled`, `privacy.zoom_recording_import_retention_days` и др.
- `frontend/src/integrations/zoom/components/zoomEvidence.test.ts` / `.ts` — описаны функции `extractZoomRecordingRefs` и `formatZoomTranscriptProvenance`; в вики приведено их поведение.
- `frontend/src/integrations/zoom/queries/useZoomRuntimeQuery.boundary.test.ts` — подтверждена инвалидация `providerCalls` и `callTranscript` при мутациях бриджинга и синхронизации.
- `frontend/src/integrations/zoom/queries/useZoomRuntimeQuery.ts` — перечислены все хуки и функции инвалидации (`invalidateZoomRuntime`, `invalidateZoomDerived`).
- `frontend/src/integrations/zoom/queries/zoomQueryKeys.test.ts` / `zoomQueryKeys.ts` — зафиксированы все ключи под пространством `['integrations', 'zoom', ...]`; в вики перечислены основные ключи.
- `frontend/src/integrations/zoom/types/zoom.ts` — упомянуты ключевые интерфейсы, на которых основаны API‑функции и хуки.
- `frontend/src/main.ts` — описан процесс начальной загрузки: Vue, Pinia, VueQuery, `loadFrontendConfig`, инициализация API и realtime.
- `frontend/src/platform/api/ApiClient.ts` — полностью задокументирован синглтон: методы, заголовок `X-Макошь-Secret`, обработка 204, ошибки.
- `frontend/src/platform/api/index.ts` / `types.ts` — подтверждены экспорты `ApiClient`, `ApiError`, `PaginatedResponse`.
- `frontend/src/platform/bootstrap/api.test.ts` / `api.ts` — показана функция `initializeApiClient` и её тест.
- `frontend/src/platform/bootstrap/businessCommunicationOwnership.boundary.test.ts` — изложены три правила модульных границ.
- `frontend/src/platform/bootstrap/realtime.test.ts` — подтверждены: создание SSE/WebSocket клиентов, фолбэк с WebSocket на SSE, reconnect, работа с replay‑курсором, инвалидация при `lagged` и по типам событий.
- `frontend/src/platform/bootstrap/realtime.ts` — перечислены группы query‑ключей (`REALTIME_QUERY_KEYS`, `MAIL_…`, `TELEGRAM_…`, `WHATSAPP_…`, `ZOOM_…`, `SIGNAL_HUB_…`) и логика маршрутизации событий; описаны патчи кеша.
- `frontend/src/platform/bootstrap/realtimeCachePatches.test.ts` — подтверждены патчи для `mail.folder_message.moved`, `mail.saved_search.updated`, `mail.sync.progress`, `mail.draft.*`.
- `frontend/src/platform/bootstrap/realtimeMailCachePatches.test.ts` — подтверждены патчи для `mail.outbox.*`, `mail.draft.deleted`, `mail.folder.*`.
- `frontend/src/platform/bootstrap/realtimeSignalHubInvalidation.test.ts` — подтверждена инвалидация Signal Hub при событиях `signal.*` и `lagged`.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений (drift) между исходным кодом и документацией не выявлено. Все утверждения в предложенной странице напрямую подтверждаются встроенными исходными файлами.
