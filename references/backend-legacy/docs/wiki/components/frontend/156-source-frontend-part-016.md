---
chunk_id: 156-source-frontend-part-016
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 156-source-frontend-part-016 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Обновить страницу `components/frontend.md` русской wiki Obsidian, чтобы задокументировать платформенные модули фронтенда, представленные в данном чанке: интернационализация, клиент настроек приложения, система тем (настройки, персистентность, токены), клиенты реального времени (SSE с long‑poll‑фолбэком и WebSocket), общие типы коммуникаций (Telegram, WhatsApp) и общие композаблы. Страница должна содержать только факты, подтверждённые встроенным исходным кодом, без домысливания внешних знаний или невидимых частей системы.

## Предложенные страницы

### `components/frontend.md`

```markdown
# Frontend

## Интернационализация (i18n)

Модуль `frontend/src/platform/i18n/` предоставляет механизм локализации на основе Vue‑композабла `useI18n`.

- Поддерживаемые локали: `ru` (значение по умолчанию) и `en`.
- Выбранная локаль хранится в `localStorage` под ключом `hh-locale`.
- Функция `setLocale(locale)` меняет активную локаль и сохраняет выбор в `localStorage`.
- `useI18n()` возвращает:
  - `t(key, params?): string` — функция перевода. Если ключ отсутствует в словаре, возвращает сам ключ (identity‑fallback).
  - `locale` — реактивный `Ref<Locale>` с текущей локалью.
  - `setLocale` — для смены локали.
- Поддерживается интерполяция параметров: шаблон `{name}` в строке перевода заменяется значением из переданного `params`.
- Словари импортируются из `ru.json` и `en.json` (тип `Dictionary = Record<string, string>`).

## Клиент настроек приложения

`frontend/src/platform/settings/applicationSettingsClient.ts` — слой для работы с настройками приложения через API бэкенда.

- Использует `ApiClient` (из `../api/ApiClient`, в чанке не раскрыт).
- `fetchApplicationSettings()` выполняет `GET /api/v1/settings` и возвращает `ApplicationSettingsResponse { items: ApplicationSetting[] }`.
- `saveApplicationSetting(key, value)` отправляет `PUT /api/v1/settings/{key}` с телом `{ value }` и возвращает сохранённую настройку.
- Определены константы ключей:
  - `FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout'`
  - `FRONTEND_SIDEBAR_SETTING_KEY = 'frontend.sidebar'`
  - `FRONTEND_LOCALE_SETTING_KEY = 'frontend.locale'`
  - `FRONTEND_THEME_SETTING_KEY = 'frontend.theme'`
  - `FRONTEND_UI_STATE_SETTING_KEY = 'frontend.ui_state'`
- Тип `ApplicationSetting` содержит поля: `setting_key`, `category`, `value_kind` (`'boolean' | 'integer' | 'string' | 'json'`), `value`, `label`, `description`, `metadata`, `is_editable`, `updated_by_actor_id`, `created_at`, `updated_at`.

## Тема

Система тем разделена на настройки (settings), персистентность (persistence) и общие токены (tokens).

### Настройки темы

`frontend/src/platform/theme/settings.ts` определяет схему и валидацию настроек темы.

- `THEME_SCHEMA_VERSION = 1` — версия схемы.
- `ThemeSettings` — тип с полями:
  - `schemaVersion: 1`
  - `shellBackground` — один из 11 идентификаторов (`'none'`, `'network-mesh'`, `'data-stream'`, `'node-frame'`, `'eclipse-grid'`, `'dna-blueprint'`, `'forest-network'`, `'forest-stream'`, `'knowledge-map'`, `'rune-gold'`, `'rune-teal'`).
  - `backgroundBrightness` — одно из значений `30 | 40 | 50 | 60 | 70 | 80 | 90 | 100`.
  - `accentColor` — `'teal' | 'cyan' | 'blue' | 'violet' | 'amber' | 'rose'`.
  - `panelOpacity` — `40 | 50 | 60 | 70 | 80 | 90 | 100`.
  - `panelBlur` — `0 | 4 | 8 | 12 | 16 | 20 | 24`.
  - `spacingDensity` — `'compact' | 'normal' | 'comfortable'`.
- `defaultThemeSettings()` возвращает значения по умолчанию (`shellBackground: 'network-mesh'`, `backgroundBrightness: 70`, `accentColor: 'teal'`, `panelOpacity: 70`, `panelBlur: 12`, `spacingDensity: 'normal'`).
- `parseThemeSettings(value)` принимает произвольное значение и возвращает либо валидный `ThemeSettings`, либо значения по умолчанию, если входящие данные не соответствуют схеме или типу.
- Функции‑хелперы формируют CSS‑классы:
  - `shellBackgroundClass(settings)` → `shell-bg-{id}`
  - `shellBrightnessClass(settings)` → `shell-bg-brightness-{value}`
  - `shellAccentClass(settings)` → `theme-accent-{color}`
  - `shellPanelOpacityClass(settings)` → `panel-opacity-{value}`
  - `shellPanelBlurClass(settings)` → `panel-blur-{value}`
  - `shellSpacingDensityClass(settings)` → `spacing-density-{density}`

### Сохранение и загрузка темы

`frontend/src/platform/theme/persistence.ts` управляет персистентностью настроек темы, используя в качестве основного источника бэкенд, а в качестве запасного — `localStorage`.

- `loadPersistedThemeSettings()`:
  - Запрашивает настройки через `fetchApplicationSettings`, ищет запись с ключом `frontend.theme`.
  - При успехе парсит через `parseThemeSettings`, сохраняет в `localStorage` под ключом `makosh-theme-settings` и возвращает с `source: 'application_settings'`.
  - При ошибке или отсутствии настройки на бэкенде возвращает локальные настройки с `source: 'local_storage'` и сообщением `errorMessage` (если была ошибка — `'Theme settings backend unavailable; using local browser settings.'`, иначе пустая строка).
- `savePersistedThemeSettings(settings)`:
  - Отправляет настройки на бэкенд через `saveApplicationSetting`, затем дублирует в `localStorage`.
  - При ошибке бэкенда сохраняет только локально и возвращает `source: 'local_storage'` с сообщением `'Theme saved locally only. Application settings backend is unavailable.'`.
- `loadLocalThemeSettings()` — чистое чтение из `localStorage` с парсингом и фолбэком на `defaultThemeSettings()`.

### Дизайн‑токены

`frontend/src/platform/theme/tokens.ts` экспортирует константу `theme` с набором низкоуровневых значений:

- `font.sans`: строка `font-family` из семейства `Inter`, `SF Pro Display`, системных шрифтов.
- `color`: `bg` (`#02090b`), `bgRaised` (`#020d10`), `surface` (`#06181b`), `surfaceDeep` (`#041215`), `text` (`#eefefb`), `textStrong`, `textBright`, `textSoft`, `textMuted`, `textSubtle`, `textDim`, `accent` (`#2df0ce`), `accentStrong`, `accentSoft`, `accentContrast`, `danger` (`#ffabab`), `dangerStrong`.
- `radius`: значения от `4px` до `50%`.
- `space`: от `4px` до `24px`.
- `layout`: `row: '37px'`, `gap: '10px'`, `columns: 12`.

## Реальное время (Realtime)

Пакет `frontend/src/platform/sse/` предоставляет два клиента для защищённого real‑time‑взаимодействия: `SseClient` (SSE с fallback на long polling) и `WebSocketClient`. Оба используют секрет `X-Макошь-Secret` для аутентификации, хранят позицию последнего полученного события (`lastEventId`) и поддерживают автоматическое переподключение.

### SSE‑клиент (SseClient)

`SseClient` — класс в `SseClient.ts`. Построен на `fetch` с `ReadableStream`, потому что браузерный `EventSource` не позволяет передавать кастомные заголовки.

- Конструктор принимает `SseClientOptions`:
  - `url` — URL потока SSE.
  - `secret` — обязательный `X-Макошь-Secret`.
  - `lastEventId?` — начальная позиция для реплея.
  - `onMessage?`, `onError?`, `onStatus?` — колбэки.
  - `reconnectDelay?` (по умолчанию 3000 мс), `maxReconnectAttempts?` (10).
  - `longPollUrl?`, `longPollDelay?` (3000 мс), `longPollBatchSize?` (100), `longPollWaitSeconds?` (15).
  - `fetchImpl?` — кастомная реализация `fetch`.
- `connect()` запускает SSE‑соединение.
- `disconnect()` останавливает соединение и предотвращает переподключение.
- Заголовки запроса: `Accept: text/event-stream`, `X-Макошь-Secret`, и при наличии `lastEventId` — `Last-Event-ID`.
- URL дополняется параметром `after_position` для реплея.
- Поток парсится вручную: блоки разделяются `\n\n`, внутри строки с полями `id`, `event`, `data`. Собранные события передаются в `onMessage`.
- При ошибке или закрытии потока запускается механизм переподключения с экспоненциальной задержкой (множитель до 5).
- Когда количество попыток SSE‑переподключений исчерпано (`maxReconnectAttempts`), и если задан `longPollUrl`, клиент автоматически переходит в режим long polling.
- Long polling выполняется циклическим `GET` на `longPollUrl` с параметрами `after_position`, `limit`, `wait_seconds`. Ожидается ответ в формате `{ items: [{ position, event }], next_after_position, has_more }`. Каждый элемент преобразуется в событие с `id = position` и `data = JSON.stringify(item)`.
- Статусные переходы сообщаются через `onStatus` с полями `transport` (`'sse' | 'long_poll'`) и `state` (`'connecting' | 'connected' | 'reconnecting' | 'fallback' | 'disconnected'`), а также `attempt`, `maxAttempts`, `error`.

### WebSocket‑клиент (WebSocketClient)

`WebSocketClient` — класс в `WebSocketClient.ts`. Использует нативный браузерный `WebSocket` и передаёт аутентификацию через query‑параметр, так как кастомные заголовки в WebSocket API не поддерживаются.

- Конструктор принимает `WebSocketClientOptions`: `url`, `secret`, `lastEventId?`, `onMessage?`, `onError?`, `onStatus?`, `reconnectDelay?` (3000), `maxReconnectAttempts?` (10).
- `connect()` инициирует соединение.
- `disconnect()` разрывает соединение и предотвращает переподключение.
- URL формируется функцией `replayUrl()`: к исходному URL добавляется параметр `makosh_secret` со значением секрета, и при наличии `lastEventId` — параметр `after_position`.
- Обработка входящих сообщений (`JSON`):
  - `type: 'heartbeat'` — игнорируется.
  - `type: 'lagged'` — извлекается `data.skipped`; если число > 0, в `onMessage` передаётся событие с `event: 'lagged'`, `id = lastEventId`, `data = JSON.stringify({ skipped })`. Если `skipped` отсутствует или некорректен, вызывается `onError`.
  - `type: 'event'` — из `data` извлекается `position` (должен быть конечным неотрицательным числом); если валиден, событие передаётся в `onMessage` с `id = position`, `event: 'event'`, `data` в виде строки. При отсутствии позиции или некорректном формате вызывается `onError`.
  - Все остальные типы вызывают `onError` с сообщением `"Unknown WebSocket message type …"`.
- Механизм переподключения аналогичен SSE: после ошибки или закрытия сокета запускается таймер с задержкой `reconnectDelay * min(attempt, 5)`, после исчерпания попыток сообщается финальный статус `disconnected`.

### Экспорт модуля

`index.ts` реэкспортирует `SseClient`, `WebSocketClient` и все связанные типы.

## Типы данных коммуникаций

### Telegram

Пакет `frontend/src/shared/communications/types/` содержит большое количество типов для интеграции с Telegram.

- Основной файл `telegram.ts` (в чанке обрезан) определяет более 700 строк типов, включая:
  - Учётные записи (`TelegramAccount`, `TelegramAccountLifecycleState`, …)
  - Возможности (`TelegramCapabilitiesResponse`, `TelegramOperationCapability`, …)
  - Рантайм (`TelegramRuntimeStatus`)
  - Чаты (`TelegramChat`, `TelegramChatSyncState`, …)
  - Сообщения (`TelegramMessage`, `TelegramMediaItem`, …)
  - Синхронизацию (`TelegramChatSyncRequest/Response`, `TelegramHistorySyncRequest/Response`)
  - QR‑вход (`TelegramQrLoginStatus`, `TelegramQrLoginStatusResponse`, …)
  - UI‑типы: фильтры (`TelegramChatFilter`), вкладки (`TelegramThreadTab`, `TelegramRailTab`)
  - Жизненный цикл сообщений (ADR‑0091): `TelegramCommandKind`, `TelegramCommandStatus`, `TelegramLifecycleResponse`, `TelegramTombstoneReasonClass`, `TelegramTombstoneActorClass`, `TelegramMessageVersion`, …
- Выделенные файлы (для соблюдения принципа единственной ответственности — SRP):
  - `telegramChatActions.ts` — запросы/ответы действий с чатами (`TelegramChatActionRequest`, `TelegramChatLifecycleCommandResponse`, `TelegramChatFolderReassignRequest/Response`).
  - `telegramLifecycleRequests.ts` — типы команд жизненного цикла (`TelegramEditRequest`, `TelegramReplyRequest`, `TelegramForwardRequest`, `TelegramDeleteRequest`, `TelegramRestoreVisibilityRequest`, `TelegramPinRequest`).
  - `telegramMembers.ts` — участники чатов (`TelegramChatMember`, `TelegramChatMemberListResponse`, `TelegramChatMembersSyncResponse`).
  - `telegramRawEvidence.ts` — сырые записи (`TelegramRawMessageRecord`, `TelegramRawMessageResponse`).
  - `telegramTopics.ts` — форумные топики (`TelegramTopic`, `TelegramTopicListResponse`, `TelegramTopicCreateRequest`, `TelegramTopicCloseRequest`, `TelegramTopicLifecycleResponse`).

### WhatsApp

`frontend/src/shared/communications/types/whatsapp.ts` (в чанке обрезан) описывает аналогичный набор типов для WhatsApp:

- Провайдеры (`WhatsappWebProviderKind`, `WhatsappProviderShape`).
- Возможности (`WhatsappCapabilitiesResponse`, `WhatsappCapabilityAccountScope`, …).
- Рантайм (`WhatsAppRuntimeStatus`, `WhatsAppRuntimeHealth`, …).
- Манифест web‑компаньона (`WhatsAppWebCompanionManifest`, `WhatsAppWebCompanionBridgeRoutes`, …).
- Команды (`WhatsAppProviderCommand`, `WhatsAppProviderCommandListResponse`).
- Синхронизация чатов, участников, присутствия, звонков, контактов, медиа, статусов.
- Жизненный цикл (`WhatsAppLifecycleResponse`).
- QR‑сессии и pair‑code сессии.
- Сессии (`WhatsappWebSession`, `WhatsappWebSessionListResponse`).
- Сообщения (`WhatsappWebMessage`, `WhatsappWebMessageListResponse`, …).

### Утилиты для обновления в реальном времени

`frontend/src/shared/communications/queries/realtimePatchShared.ts` содержит вспомогательные функции и типы для применения real‑time‑патчей к кешу запросов.

- Типы состояний:
  - `WorkflowState` (`'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam'`).
  - `LocalMessageState` (`'active' | 'trash' | 'all'`).
  - `BulkMessageAction` (операции `mark_read`, `archive`, `trash`, `pin`, `add_label` и др.).
  - `CommunicationAiState` (`'NEW' | 'PROCESSING' | 'PROCESSED' | 'REVIEW_REQUIRED' | 'FAILED' | 'ARCHIVED'`).
- Типы данных:
  - `CommunicationFolder`, `FolderMessage` — элементы папок и сообщений в них.
  - `CommunicationSavedSearch` — сохранённые поиски.
- Функции‑парсеры:
  - `storedEventEnvelope(eventData)` — парсит JSON из строки события в `StoredEventEnvelope`.
  - `stringValue`, `numberValue`, `nullableStringValue`, `nullableNumberValue` — безопасное извлечение скалярных значений.
  - `outboxStatusValue` — валидация статуса исходящего сообщения.
  - `aiStateValue` — валидация `CommunicationAiState`.
  - `normalizeBulkAction`, `normalizeMessageIds` — нормализация входящих параметров пакетных операций.
  - `folderValue`, `folderMessageValue`, `savedSearchValue` — строгая валидация и построение объектов папок и сообщений.
- Типы‑заготовки для работы с кешом React Query: `MailRealtimePatchQueryClient`, `CacheKeyFilter`, `StoredEventEnvelope`, `CommunicationMessagePatchPayload`, `OutboxPatchPayload`, `AiStatePatchPayload`, `DraftPatchPayload`, `FolderMessagePatchPayload`, `SyncPatchPayload`.

## Композаблы (composables)

В `frontend/src/shared/composables/` находятся переиспользуемые Vue‑композаблы.

- `useClickOutside(elRef, callback, options?)` — вызывает `callback`, когда пользователь кликает вне элемента. Опционально можно исключить дополнительный элемент через `excludeElRef`. Использует `document.addEventListener('click', …, true)`.
- `useKeyboard(handlers)` — регистрирует обработчики нажатий клавиш. Каждый `handler` содержит `key` и опциональные `ctrl?`, `meta?`, `shift?`. Совпадение вызывает `event.preventDefault()` и `handler()`. `useEscapeKey(callback)` — удобная обёртка для клавиши Escape.
- `useResizeObserver(elRef, callback)` — возвращает реактивные `width` и `height`, обновляемые при изменении размеров элемента. Использует `ResizeObserver`.
```

## Покрытие источников

| Файл | Покрытые факты |
|------|----------------|
| `frontend/src/platform/i18n/index.ts` | Локаль `currentLocale` как `ref`, сохранение в `localStorage` по ключу `hh-locale`, значение по умолчанию `ru`, функция `setLocale`, композабл `useI18n`, поведение функции `t` (поиск в словаре, identity‑fallback, интерполяция `{param}`). |
| `frontend/src/platform/i18n/types.ts` | Типы `Locale`, `TranslationFunction`, `Dictionary`. |
| `frontend/src/platform/settings/applicationSettingsClient.ts` | Типы `SettingValueKind`, `ApplicationSettingValue`, `ApplicationSetting`, `ApplicationSettingsResponse`. Константы ключей `FRONTEND_*_SETTING_KEY`. Функции `fetchApplicationSettings` (GET `/api/v1/settings`) и `saveApplicationSetting` (PUT `/api/v1/settings/{key}`). |
| `frontend/src/platform/sse/SseClient.ts` | Класс `SseClient`, опции (`url`, `longPollUrl`, `secret`, `lastEventId`, колбэки, тайминги), логика соединения (fetch‑based SSE, заголовки `X-Макошь-Secret`, `Last-Event-ID`, `Accept`), парсинг SSE‑потока (блоки, поля `id`, `event`, `data`), стратегия переподключения с экспоненциальной задержкой, fallback на long polling при исчерпании попыток SSE, формат long‑poll‑запроса (параметры `after_position`, `limit`, `wait_seconds`), трансформация элементов long‑poll в события, статусные события (`transport`, `state`). |
| `frontend/src/platform/sse/SseClient.test.ts` | Подтверждение поведения: статусные переходы `sse:connecting → connected → disconnected`, передача `X-Макошь-Secret` и `Last-Event-ID`, построение replay‑URL (относительные и абсолютные), fallback на long polling после исчерпания попыток SSE с корректным URL и заголовками. |
| `frontend/src/platform/sse/WebSocketClient.ts` | Класс `WebSocketClient`, опции, передача секрета через query‑параметр `makosh_secret` (вместе с `after_position`), обработка типов сообщений (`heartbeat`, `lagged` → извлечение `skipped`, `event` → извлечение `position`, неизвестные типы), механизм переподключения, статусные события `websocket:{connecting,connected,reconnecting,disconnected}`. |
| `frontend/src/platform/sse/WebSocketClient.test.ts` | Проверка обработки `lagged` payloads: событие `lagged` с корректным `skipped` не вызывает ошибку и передаётся в `onMessage`. |
| `frontend/src/platform/sse/index.ts` | Реэкспорт `SseClient`, `WebSocketClient` и их типов. |
| `frontend/src/platform/theme/persistence.ts` | Функции `loadPersistedThemeSettings`, `savePersistedThemeSettings`, `loadLocalThemeSettings`. Ключ `makosh-theme-settings` для `localStorage`. Источник `'application_settings'` vs `'local_storage'`. Сообщения об ошибках загрузки/сохранения. Использование `FRONTEND_THEME_SETTING_KEY`. |
| `frontend/src/platform/theme/persistence.test.ts` | Тесты fallback‑сценариев: при ошибке сохранения настройки сохраняются локально с `source: 'local_storage'` и сообщением `'saved locally only'`; при ошибке загрузки настройки берутся из `localStorage` с сообщением `'backend unavailable'`. |
| `frontend/src/platform/theme/settings.ts` | `THEME_SCHEMA_VERSION`, тип `ThemeSettings`, перечисления разрешённых значений (`shellBackgroundIds`, `backgroundBrightnessValues`, `accentColorIds`, `panelOpacityValues`, `panelBlurValues`, `spacingDensityIds`), `defaultThemeSettings`, `parseThemeSettings` (валидация, fallback к дефолтам), CSS‑хелперы `shellBackgroundClass`, `shellBrightnessClass`, `shellAccentClass`, `shellPanelOpacityClass`, `shellPanelBlurClass`, `shellSpacingDensityClass`. |
| `frontend/src/platform/theme/settings.test.ts` | Подтверждение: возврат дефолтов для `null` и некорректной версии схемы; сохранение валидных разрешённых значений; корректные имена CSS‑классов для заданных настроек. |
| `frontend/src/platform/theme/tokens.ts` | Объект `theme` с полями `font`, `color`, `radius`, `space`, `layout` и их конкретными значениями. |
| `frontend/src/shared/communications/queries/realtimePatchShared.ts` | Перечисление состояний (`WorkflowState`, `LocalMessageState`, `BulkMessageAction`, `CommunicationAiState`), типы `CommunicationFolder`, `FolderMessage`, `CommunicationSavedSearch`, сигнатуры функций‑парсеров и валидаторов (`storedEventEnvelope`, `stringValue`, `numberValue`, `outboxStatusValue`, `aiStateValue`, `normalizeBulkAction`, `normalizeMessageIds`, `folderValue`, `folderMessageValue`, `savedSearchValue`), типы‑заготовки для патчей (`MailRealtimePatchQueryClient`, `StoredEventEnvelope`, `CommunicationMessagePatchPayload`, …). |
| `frontend/src/shared/communications/types/telegram.ts` (truncated) | Основные группы типов: учётные записи, возможности, рантайм, чаты, сообщения, медиа, синхронизация, QR‑вход, UI‑фильтры/вкладки, жизненный цикл (ADR‑0091). |
| `frontend/src/shared/communications/types/telegramChatActions.ts` | Типы `TelegramChatActionRequest`, `TelegramChatActionResponse`, `TelegramChatLifecycleCommandResponse`, `TelegramChatFolderReassignRequest/Response`. |
| `frontend/src/shared/communications/types/telegramLifecycleRequests.ts` | Типы команд жизненного цикла: `TelegramEditRequest`, `TelegramReplyRequest`, `TelegramForwardRequest`, `TelegramDeleteRequest`, `TelegramRestoreVisibilityRequest`, `TelegramPinRequest`. |
| `frontend/src/shared/communications/types/telegramMembers.ts` | Типы `TelegramChatMember`, `TelegramChatMemberListResponse`, `TelegramChatMembersSyncResponse`. |
| `frontend/src/shared/communications/types/telegramRawEvidence.ts` | Типы `TelegramRawMessageRecord`, `TelegramRawMessageResponse`. |
| `frontend/src/shared/communications/types/telegramTopics.ts` | Типы `TelegramTopic`, `TelegramTopicListResponse`, `TelegramTopicCreateRequest`, `TelegramTopicCloseRequest`, `TelegramTopicLifecycleResponse`. |
| `frontend/src/shared/communications/types/whatsapp.ts` (truncated) | Основные группы типов: провайдеры, возможности, рантайм, web‑компаньон, команды, синхронизация (чаты, участники, присутствие, звонки, контакты, медиа, статусы), жизненный цикл, сессии, сообщения. |
| `frontend/src/shared/composables/index.ts` | Реэкспорт `useClickOutside`, `useKeyboard`, `useResizeObserver`. |
| `frontend/src/shared/composables/useClickOutside.ts` | Логика: слушатель `click` на `document` с `capture: true`, проверка `el.contains(event.target)`, исключение через `excludeElRef`, вызов `callback`. |
| `frontend/src/shared/composables/useKeyboard.ts` | Приём массива `KeyHandler`, проверка `key`, `ctrl`, `meta`, `shift`, `preventDefault`. `useEscapeKey` как обёртка. |
| `frontend/src/shared/composables/useResizeObserver.ts` | Использование `ResizeObserver`, возврат `ref(width)` и `ref(height)`, вызов колбэка для каждой записи, очистка при размонтировании. |

## Исходные файлы

- [`frontend/src/platform/i18n/index.ts`](../../../../frontend/src/platform/i18n/index.ts)
- [`frontend/src/platform/i18n/types.ts`](../../../../frontend/src/platform/i18n/types.ts)
- [`frontend/src/platform/settings/applicationSettingsClient.ts`](../../../../frontend/src/platform/settings/applicationSettingsClient.ts)
- [`frontend/src/platform/sse/SseClient.test.ts`](../../../../frontend/src/platform/sse/SseClient.test.ts)
- [`frontend/src/platform/sse/SseClient.ts`](../../../../frontend/src/platform/sse/SseClient.ts)
- [`frontend/src/platform/sse/WebSocketClient.test.ts`](../../../../frontend/src/platform/sse/WebSocketClient.test.ts)
- [`frontend/src/platform/sse/WebSocketClient.ts`](../../../../frontend/src/platform/sse/WebSocketClient.ts)
- [`frontend/src/platform/sse/index.ts`](../../../../frontend/src/platform/sse/index.ts)
- [`frontend/src/platform/theme/persistence.test.ts`](../../../../frontend/src/platform/theme/persistence.test.ts)
- [`frontend/src/platform/theme/persistence.ts`](../../../../frontend/src/platform/theme/persistence.ts)
- [`frontend/src/platform/theme/settings.test.ts`](../../../../frontend/src/platform/theme/settings.test.ts)
- [`frontend/src/platform/theme/settings.ts`](../../../../frontend/src/platform/theme/settings.ts)
- [`frontend/src/platform/theme/tokens.ts`](../../../../frontend/src/platform/theme/tokens.ts)
- [`frontend/src/shared/communications/queries/realtimePatchShared.ts`](../../../../frontend/src/shared/communications/queries/realtimePatchShared.ts)
- [`frontend/src/shared/communications/types/telegram.ts`](../../../../frontend/src/shared/communications/types/telegram.ts)
- [`frontend/src/shared/communications/types/telegramChatActions.ts`](../../../../frontend/src/shared/communications/types/telegramChatActions.ts)
- [`frontend/src/shared/communications/types/telegramLifecycleRequests.ts`](../../../../frontend/src/shared/communications/types/telegramLifecycleRequests.ts)
- [`frontend/src/shared/communications/types/telegramMembers.ts`](../../../../frontend/src/shared/communications/types/telegramMembers.ts)
- [`frontend/src/shared/communications/types/telegramRawEvidence.ts`](../../../../frontend/src/shared/communications/types/telegramRawEvidence.ts)
- [`frontend/src/shared/communications/types/telegramTopics.ts`](../../../../frontend/src/shared/communications/types/telegramTopics.ts)
- [`frontend/src/shared/communications/types/whatsapp.ts`](../../../../frontend/src/shared/communications/types/whatsapp.ts)
- [`frontend/src/shared/composables/index.ts`](../../../../frontend/src/shared/composables/index.ts)
- [`frontend/src/shared/composables/useClickOutside.ts`](../../../../frontend/src/shared/composables/useClickOutside.ts)
- [`frontend/src/shared/composables/useKeyboard.ts`](../../../../frontend/src/shared/composables/useKeyboard.ts)
- [`frontend/src/shared/composables/useResizeObserver.ts`](../../../../frontend/src/shared/composables/useResizeObserver.ts)

## Кандидаты на drift

- Файлы `frontend/src/shared/communications/types/telegram.ts` и `frontend/src/shared/communications/types/whatsapp.ts` обрезаны; полный перечень типов этих модулей не может быть подтверждён текущим контекстом, что может привести к расхождению между задокументированным обзором и фактическим содержимым.
- В коде жизненного цикла Telegram (ADR‑0091) фигурирует ссылка на ADR, но сам документ в чанке отсутствует, поэтому его актуальное состояние не проверено.
- Клиент настроек приложения (`applicationSettingsClient.ts`) использует `ApiClient`, чья реализация в чанк не включена; предполагается, что поведение API соответствует документированному, но без исходников `ApiClient` это не может быть верифицировано.
- В остальном из предоставленного контекста заметных расхождений между кодом и документацией не выявлено.
