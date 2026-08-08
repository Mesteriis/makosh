## Summary / Резюме

Добавлена страница `components/frontend.md` с описанием архитектуры фронтенд-слоя системных интеграций. На основе предоставленных исходных файлов документированы:

- интеграция Telegram (типы данных, realtime-события ADR-0091, аудит провайдер-команд, UI-состояние runtime, boundary-тест панели);
- интеграция WhatsApp (API-слой, Tauri-компаньон, realtime-патчи TanStack Query, Pinia-стор, хуки запросов, вспомогательные функции представлений, граничные тесты панели);
- интеграция Yandex Telemost (API, хуки запросов, ключи кэша).

Страница написана на русском языке и совместима с Obsidian Markdown. Все утверждения о поведении опираются на встроенный текст исходников.

## Proposed pages / Предлагаемые страницы

### `components/frontend.md`

```markdown
# Frontend-компоненты системных интеграций

## Общее описание

Фронтенд-часть Макошь построена с использованием:

- **Vue 3** и **Pinia** — стейт-менеджмент;
- **TanStack Vue Query** — запросы и реактивное кэширование данных;
- **Tauri** (импорт `invoke` из `@tauri-apps/api/core`) — десктопный WebView-компаньон.

Взаимодействие с бэкендом реализовано через единый HTTP-клиент
`ApiClient` (`frontend/src/platform/api/ApiClient`) либо напрямую через
Tauri-инвоки для операций, требующих доступа к локальному окружению
(видимые WebView, запись аудио и т.п.).

---

## Интеграция Telegram

### Типы данных

Базовые типы Telegram реэкспортируются из общей библиотеки
`../../../shared/communications/types/telegram` (
`frontend/src/integrations/telegram/types/telegram.ts`).

Отдельный файл `frontend/src/integrations/telegram/types/telegramRealtime.ts`
(с аннотацией **ADR-0091**) определяет поверхность realtime-событий:

- `TelegramRealtimeEventType` — перечисление событий синхронизации,
  жизненного цикла сообщений, чатов, медиа и команд (`telegram.sync.*`,
  `telegram.message.*`, `telegram.chat.*`, `telegram.media.*`,
  `telegram.command.*` и др.).
- `TelegramRealtimeEvent` — структура события с полями `event_type`,
  `event_id`, `occurred_at`, `subject`, `payload`.
- `TelegramRealtimeMessage` — дискриминируемое объединение `{ type: 'event' }`
  или `{ type: 'lagged' }`.

### Аудит команд Telegram

Файл `frontend/src/integrations/telegram/stores/telegramCommandAudit.ts`
содержит audit-состояние для команд провайдера (`TelegramProviderWriteCommand`).

- Тип `TelegramCommandAuditTone` — `'neutral' | 'progress' | 'success' | 'warning' | 'danger'`.
- Тип `TelegramCommandAuditState` — лейбл, детализация, тон и флаг `is_dead_letter`.
- Функции `providerStateString`, `providerStateBoolean`, `payloadString`,
  `payloadNumber` — безопасное извлечение значений из `provider_state` и
  `payload`.

Ключевые детекторы mismatch-состояний:

- `editMismatchDetail` — сверка ожидаемого и наблюдаемого текста (в
  символах).
- `reactionMismatchDetail` — проверка присутствия/отсутствия реакции
  (`observed_is_chosen`).
- `pinMismatchDetail` — проверка состояния закрепления сообщения или чата.
- `chatLifecycleMismatchDetail` — агрегирует `pinMismatchDetail`, а также
  проверки для `mark_unread`, `archive`/`unarchive`, `mute`/`unmute`.

Функция `messageLifecycleDetail` собирает детали для всех команд
жизненного цикла: `edit`, `delete`, `restore_visibility`, `mark_unread`,
`pin`/`unpin`, `archive`/`unarchive`, `mute`/`unmute`,
`folder_add`/`folder_remove`, `react`/`unreact`. Для каждой ветки
учитываются наблюдаемые состояния провайдера и статус команды.

Функция `executingCommandDetail` является верхнеуровневой точкой
сборки: сначала проверяется `participantLifecycle`, затем
`mark_read`-прогресс, затем `messageLifecycleDetail`, затем извлечение
`progress_detail` и `upload_phase` (включая фазу
`'dispatching_to_provider'`). Если reconciliation_status равен
`'awaiting_provider'`, возвращается `'Awaiting provider-observed state'`.
В остальных случаях используется `telegramCommandRetrySummary`.

- `telegramCommandRetrySummary` — строка вида `"<n>/<max> retries used"`.
- `telegramCommandSubject` — человекочитаемый заголовок команды (напр.,
  `'Read through <id>'`, `'Edit message'`, `'Pin chat'`,
  `'Add chat to folder <id>'` и т. д.), учитывающий наличие
  `provider_message_id` и `provider_folder_id`.

### Статус runtime и цель команды

Модуль `frontend/src/integrations/telegram/stores/telegramRuntimeStatus.ts`
предоставляет функцию `telegramRuntimeCommandTarget`, которая
возвращает строковую цель последней выполненной команды:

- Для `last_command_kind === 'mark_read'` и наличия
  `last_command_message_id` — `"Read through <id>"`.
- В остальных случаях — первый непустой из
  `last_command_message_id`, `last_command_telegram_chat_id`,
  `last_command_provider_chat_id`, `last_command_id`.

Тест `telegramRuntimeStatus.test.ts` проверяет оба этих сценария.

### Панель runtime (граничный тест)

`TelegramRuntimePanel.boundary.test.ts` фиксирует контракт:

- панель **не** открывает отдельный Telegram-сокет (`createTelegramRealtimeConnection`,
  `onMounted`/`onUnmounted` отсутствуют);
- используются композиции: `useRealtimeStatusStore`,
  `useTelegramAccountsQuery`, `useTelegramCapabilitiesQuery`,
  `useTelegramRuntimeStatusQuery`, мутации `useStopTelegramRuntimeMutation`,
  `useStartTelegramRuntimeMutation`, `useRestartTelegramRuntimeMutation`;
- отображаются `realtimeStatus.realtimeStatusDetail` и
  `realtimeStatus.realtimeStatusTone`;
- действия `setTelegramRuntime('start')`, `('stop')`, `('restart')`;
- запросы для сообщений, чатов и медиа **не** используются;
- класс `telegram-runtime-panel` присутствует, `telegram-page` — нет.

---

## Интеграция WhatsApp

### API-слой

Файл `frontend/src/integrations/whatsapp/api/whatsapp.ts`
инкапсулирует все HTTP-вызовы к бэкенду через `ApiClient`:

- **Capabilities**: глобальные (`/api/v1/integrations/whatsapp/capabilities`)
  и на уровне аккаунта.
- **Аккаунты**: список с опциональным параметром `include_removed`,
  создание live-аккаунта (`POST /api/v1/integrations/whatsapp/accounts`).
- **Сессии**: список Web-сессий (фильтрация по account_id, limit).
- **Управление runtime**: получение статуса и здоровья, старт/стоп/
  отзыв/перепривязка/ротация/удаление runtime.
- **Логин**: запуск QR-линка (`/api/v1/integrations/whatsapp/login/qr/start`)
  и pair-code-линка.
- **Команды провайдера**: выборка, ретрай, перевод в dead-letter.
- **Синхронизация**: presence, chats, history, members, calls, contacts,
  statuses, media — все через POST-эндпоинты `/api/v1/integrations/whatsapp/provider-sync/*`.
- **Публикация статуса**: `POST .../provider-commands/statuses/publish`.
- **Фикстуры**: настройка аккаунта, ingest сообщений.
- `loadWhatsappWebWorkspace` — утилита, загружающая capabilities и сессии
  параллельно и выбирающая валидный session_id.

Тесты в `whatsappRuntime.test.ts` подтверждают корректность URL,
методов и тел запросов для большинства эндпоинтов.

### Tauri-компаньон WebView

Модуль `frontend/src/integrations/whatsapp/api/whatsappCompanion.ts`
описывает взаимодействие с видимым WebView-компаньоном WhatsApp только
через Tauri-инвоки:

- `getWhatsappWebCompanionManifest` → `invoke('whatsapp_web_companion_manifest')`
- `openWhatsappWebCompanion` → `invoke('open_whatsapp_web_companion')`
- `relayWhatsappWebCompanionObservation` → `invoke('whatsapp_web_companion_relay_observation')`

Все вызовы требуют непустой `account_id` (валидация с выбросом ошибки).

Тесты (`whatsappCompanion.test.ts`) проверяют:

- **отсутствие** прямых HTTP-вызовов (глобальная `fetch` не используется);
- структуру манифеста:
  - `event_extractor.state`: `'contract_injected_relay_dispatch_available'`
  - `event_extractor.forbidden_reads`: `'message_bodies'`, `'media_bytes'` и др.
  - `secret_policy.cookies`: `'not_read_or_returned_by_tauri_command'`
  - `remaining_blockers`: `'whatsapp_webview_runtime_panel_action_not_implemented'`,
    `'whatsapp_webview_live_smoke_required'`
- relay observation возвращает детали диспетчеризации и статус `200`.

### Realtime-патчи TanStack Query

Realtime-инфраструктура WhatsApp обновляет кэш в реальном времени
без перезапроса данных с бэкенда.

**Хелперы** (`realtimeWhatsAppRuntimePatchValues.ts`):

- `integerValue`, `booleanValue`, `stringArray`, `nullableStringValue`.

**Главный диспетчер** (`realtimeWhatsAppRuntimePatches.ts`):

- `applyWhatsAppRuntimeRealtimePatch(eventData, queryClient)` парсит
  конверт события (вызывая `storedEventEnvelope`), проверяет префикс
  `'whatsapp.'` и последовательно патчит закэшированные списки:
  - **сессии** — только при `whatsapp.runtime.status_changed`,
    `whatsapp.session.link_state_changed` или `whatsapp.runtime.event`;
  - **runtime-статус** — аналогично;
  - **команды** — при `whatsapp.command.status_changed`;
  - **sync-списки**: presence (`whatsapp.presence.changed`),
    calls, statuses, chats, members, contacts.
- Каждая функция патчинга (`patchSession`, `patchRuntimeStatus`,
  `patchCommandList`) мерджит поля из payload события с сохранением
  существующих значений при отсутствии новых.

**Патчи синхронизации** (`realtimeWhatsAppRuntimeSyncPatches.ts`):

- `patchPresenceList`, `patchCallList`, `patchStatusesList`,
  `patchChatsList`, `patchMembersList` — каждая добавляет или обновляет
  элементы в соответствующих массивах, используя идентификаторы
  (`identity_id`, `call_id`, `message_id`, `provider_chat_id`,
   `participant_id` и т.д.), и фильтрует по `account_id` и
  `provider_chat_id`, учитывая query-ключи.

### Хуки запросов (@tanstack/vue-query)

**Базовые запросы** (`useWhatsappQuery.ts`):

- `useWhatsappCapabilitiesQuery`
- `useWhatsappAccountsQuery(includeRemoved)` — возвращает `items[]`
- `useWhatsappAccountCapabilitiesQuery(accountId)`
- `useWhatsappSessionsQuery(accountId, limit)`

**Runtime-запросы и мутации** (`useWhatsappRuntimeQuery.ts`):

- `useWhatsappRuntimeStatusQuery(accountId)`
- `useWhatsappRuntimeHealthQuery(accountId)`
- Мутации: `useStartWhatsappRuntimeMutation`,
  `useStopWhatsappRuntimeMutation`, `useRevokeWhatsappRuntimeMutation`,
  `useRelinkWhatsappRuntimeMutation`, `useRotateWhatsappRuntimeMutation`,
  `useRemoveWhatsappRuntimeMutation`, `useStartWhatsappQrLinkMutation`,
  `useStartWhatsappPairCodeLinkMutation`, `useSetupWhatsappLiveAccountMutation`
- Запросы команд: `useWhatsappProviderCommandsQuery`
- Синхронизация: `useWhatsappSyncChatsQuery`, `useWhatsappSyncHistoryQuery`,
  `useWhatsappSyncMembersQuery`, `useWhatsappSyncPresenceQuery`,
  `useWhatsappSyncStatusesQuery`, `useWhatsappSyncCallsQuery`,
  `useWhatsappSyncContactsQuery`, `useWhatsappSyncMediaQuery`
- Все мутации по успешному завершению инвалидируют ключи:
  accounts, sessions, capabilities, accountCapabilities, runtimeStatus,
  runtimeHealth, commands, syncChats, syncHistory, syncMembers,
  syncStatuses, syncPresence, syncCalls, syncContacts, syncMedia.

**Ключи запросов** (`whatsappQueryKeys.ts`):

- Все ключи используют префикс `['integrations', 'whatsapp', ...]`.

### UI-состояние (Pinia Store)

Стор `frontend/src/integrations/whatsapp/stores/whatsapp.ts`
(`useWhatsappStore`) содержит:

- **Состояния**: `whatsappSessions`, `whatsappCapabilities`,
  `selectedWhatsappSessionId`, `whatsappError`, `whatsappActionMessage`,
  `isWhatsappLoading`, `isWhatsappActionSubmitting`, `whatsappMessageForm`.
- **Вычисляемые свойства**: `selectedWhatsappSession`,
  `whatsappClosureCapabilities` (capabilities с полем `closure_gate`),
  `whatsappBlockedCapabilities` (статус `'blocked'`).
- **Действия**: `setWhatsappData`, `selectWhatsappSession`,
  `setWhatsappLoading`, `setWhatsappActionSubmitting`, `setWhatsappError`,
  `setWhatsappActionMessage`, `resetWhatsappMessageForm`.

### Вспомогательные функции представлений

`WhatsAppRuntimePanel.helpers.ts` — чистые функции:

- `commandStatusTone` — маппинг статуса команды в тон:
  `completed` → `available`, `executing | queued | retrying` → `degraded`,
  иначе → `blocked`.
- `canRetryCommand` — `true` для `failed`, `dead_letter`, `retrying`, `cancelled`.
- `canDeadLetterCommand` — `false` только для `completed` и `dead_letter`.
- `commandTimestamp` — форматирование ближайшей значимой даты.
- `providerTargetLabel` — склейка `provider_chat_id` с `provider_message_id`.
- `runtimeHealthCheckStatus`/`runtimeHealthCheckDetail` — извлечение
  статуса и подробностей из структуры health-чека.
- Функции лейблов для каждого синхронизируемого типа: `presenceLabel`,
  `chatLabel`, `chatMeta`, `historyLabel`, `statusLabel`,
  `statusPreview`, `callLabel`, `contactLabel`, `mediaLabel`,
  `memberLabel`.
- `snapshotTimestamp` — человекочитаемое представление временной метки.

### Граничные тесты панели runtime

`WhatsAppRuntimePanel.boundary.test.ts` проверяет контракт панели:

- использование хуков синхронизации (`useWhatsappSyncChatsQuery` и др.)
  с конкретными параметрами (limit = 8 для некоторых);
- наличие и корректные подписи заголовков в `WhatsAppRuntimeSnapshots.vue`;
- присутствие кнопки **Rotate** и вызова мутации
  `useRotateWhatsappRuntimeMutation`;
- открытие **WebView-компаньона** через `openWhatsappWebCompanion`
  (только для `provider_shape === 'whatsapp_web_companion'`) с проверкой
  поля `event_extractor.relay_channel`; прямые вызовы `window.fetch` или
  `ApiClient` отсутствуют;
- рендеринг вложенной health-диагностики (`runtimeHealthChecks`,
  `runtimeHealthCheckStatus`, `runtimeHealthCheckDetail`,
  `runtimeHealth?.checked_at`).

---

## Интеграция Yandex Telemost

### API-слой

Файл `frontend/src/integrations/yandexTelemost/api/yandexTelemost.ts`
включает:

- HTTP-вызовы через `ApiClient`:
  - capabilities: `GET /api/v1/integrations/yandex-telemost/capabilities`
  - аккаунты: список и создание
  - конференции: create, read, update
  - кохосты: `GET .../cohosts?offset=&limit=`
  - манифест webview: `POST .../webview/manifest`
  - запись: intent, завершение (runtime-bridge)
- Tauri-инвоки:
  - `openYandexTelemostCompanion` → `invoke('open_yandex_telemost_companion')`
  - `prepareYandexTelemostAudioDevice` → `invoke('yandex_telemost_prepare_audio_device')`
  - `startYandexTelemostRecording` → `invoke('yandex_telemost_recording_start')`
  - `stopYandexTelemostRecording` → `invoke('yandex_telemost_recording_stop')`

### Хуки запросов

- `useYandexTelemostCapabilitiesQuery`
- `useYandexTelemostAccountsQuery(includeRemoved)`
- `useYandexTelemostRuntimeStatusQuery(accountId)`
- `useSetupYandexTelemostAccountMutation` — после успеха
  инвалидирует ключ `accounts`.

### Ключи запросов

`yandexTelemostQueryKeys.ts`: `capabilities`, `accounts`, `runtimeStatus`
— все с префиксом `['integrations', 'yandex-telemost', ...]`.
```

## Source coverage / Покрытие источников

| Исходный файл | Факты, покрытые в wiki |
|---|---|
| `frontend/src/integrations/telegram/stores/telegramCommandAudit.ts` | Типы `TelegramCommandAuditTone`, `TelegramCommandAuditState`; извлечение полей из `provider_state` и `payload`; mismatch-детекторы для edit, reaction, pin, chat lifecycle; агрегация `messageLifecycleDetail` и `executingCommandDetail`; retry-сводка и human-readable subject для всех `command_kind` |
| `frontend/src/integrations/telegram/stores/telegramRuntimeStatus.test.ts` | Тестовое покрытие `telegramRuntimeCommandTarget`: форматирование mark_read и fallback |
| `frontend/src/integrations/telegram/stores/telegramRuntimeStatus.ts` | Логика `telegramRuntimeCommandTarget` для mark_read и fallback-цели |
| `frontend/src/integrations/telegram/types/telegram.ts` | Реэкспорт общих типов Telegram |
| `frontend/src/integrations/telegram/types/telegramRealtime.ts` | ADR-0091, `TelegramRealtimeEventType` (sync, message, chat, media, command), структуры `TelegramRealtimeEvent` и `TelegramRealtimeMessage` |
| `frontend/src/integrations/telegram/views/TelegramRuntimePanel.boundary.test.ts` | Контракт панели: отсутствие прямого realtime-сокета; используемые хуки и мутации; структура шаблона |
| `frontend/src/integrations/whatsapp/api/whatsapp.ts` | Все эндпоинты WhatsApp (capabilities, accounts, sessions, runtime management, login, commands, sync, statuses, fixtures, `loadWhatsappWebWorkspace`) |
| `frontend/src/integrations/whatsapp/api/whatsappCompanion.test.ts` | Манифест компаньона (event_extractor, forbidden_reads, secret_policy, blockers); валидация account_id; relay observation; отсутствие прямого fetch |
| `frontend/src/integrations/whatsapp/api/whatsappCompanion.ts` | Функции `getWhatsappWebCompanionManifest`, `openWhatsappWebCompanion`, `relayWhatsappWebCompanionObservation` через Tauri-инвок; валидация account_id |
| `frontend/src/integrations/whatsapp/api/whatsappRuntime.test.ts` | Тесты API-вызовов: URL, методы, body для accounts, account setup, capabilities, runtime lifecycle, commands, sync-эндпоинтов |
| `frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimePatchValues.ts` | Хелперы `integerValue`, `booleanValue`, `stringArray`, `nullableStringValue` |
| `frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimePatches.ts` | Главный диспетчер `applyWhatsAppRuntimeRealtimePatch`; патчинг sessions, runtime status, commands, sync-списков; фильтрация по account_id и query-ключам |
| `frontend/src/integrations/whatsapp/queries/realtimeWhatsAppRuntimeSyncPatches.ts` | Функции `patchPresenceList`, `patchCallList`, `patchStatusesList`, `patchChatsList`, `patchMembersList`; маппинг полей событий в структуры синхронизации |
| `frontend/src/integrations/whatsapp/queries/useWhatsappQuery.ts` | Базовые хуки запросов: capabilities, accounts, account capabilities, sessions |
| `frontend/src/integrations/whatsapp/queries/useWhatsappRuntimeQuery.ts` | Runtime-хуки и мутации (start/stop/revoke/…, login, commands, все sync-запросы); инвалидация кэша после мутаций |
| `frontend/src/integrations/whatsapp/queries/whatsappQueryKeys.ts` | Все ключи запросов WhatsApp |
| `frontend/src/integrations/whatsapp/stores/whatsapp.ts` | Pinia-стор: состояния, вычисляемые свойства (`selectedWhatsappSession`, `whatsappClosureCapabilities`, `whatsappBlockedCapabilities`), действия |
| `frontend/src/integrations/whatsapp/types/whatsapp.ts` | Реэкспорт общих типов WhatsApp |
| `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.boundary.test.ts` | Контракт панели: sync-хуки, кнопка Rotate, открытие компаньона через Tauri, health-диагностика, отсутствие прямого HTTP |
| `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.helpers.ts` | Вспомогательные функции: тон команды, проверка retry/dead-letter, форматирование timestamp, лейблы для presence/chat/history/status/call/contact/media/member, health-чеки |
| `frontend/src/integrations/yandexTelemost/api/yandexTelemost.ts` | Эндпоинты Telemost (capabilities, accounts, conferences, cohosts, webview, recording) — HTTP и Tauri-инвоки |
| `frontend/src/integrations/yandexTelemost/queries/useYandexTelemostRuntimeQuery.ts` | Хуки запросов: capabilities, accounts, runtime status, мутация создания аккаунта |
| `frontend/src/integrations/yandexTelemost/queries/yandexTelemostQueryKeys.ts` | Ключи запросов Telemost |

## Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения кода с документацией или ADR не видны. В коде присутствует ссылка на **ADR-0091** (файл `telegramRealtime.ts`), что соответствует заявленной архитектуре realtime-событий. Манифест WebView-компаньона WhatsApp содержит поле `remaining_blockers` с незавершёнными задачами (`'whatsapp_webview_runtime_panel_action_not_implemented'` и `'whatsapp_webview_live_smoke_required'`), но эти значения являются ожидаемыми индикаторами незавершённой разработки, а не расхождением с утверждённой документацией. Без доступа к актуальным ADR и wiki-страницам подтвердить или опровергнуть drift невозможно.
