---
chunk_id: 152-source-frontend-part-012
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 152-source-frontend-part-012 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

В этот чанк предоставлены исходники фронтенд-слоя интеграции с Telegram: API-клиенты, хуки запросов и мутаций на базе `@tanstack/vue-query`, логика реального времени для команд, форма настройки аккаунта, хранилище аудита команд и граничные тесты компонентов Vue. На основе этих файлов требуется написать (или обновить) страницу wiki `components/frontend.md`, описывающую устройство фронтенда интеграции Telegram. Предлагаемая страница покрывает все значимые факты, подтверждённые встроенным кодом.

---

## Предложенные страницы

**Wiki-relative path**: `components/frontend.md`

```markdown
# Фронтенд интеграции Telegram

## API-клиент

Фронтенд обращается к бэкенду через `ApiClient` (путь `../../../platform/api/ApiClient`). Инициализация выполняется вызовом `ApiClient.init(baseUrl, secret)`.

### Загрузка медиа

Типы запроса и ответа:

- `TelegramMediaUploadRequest` — поля `account_id`, `provider_chat_id`, `media_type` (`TelegramMediaUploadKind`), опционально `command_id`, `attachment_id`, `blob_id`, `caption`, `filename`.
- `TelegramMediaUploadResponse` — возвращает `command_id`, `account_id`, `provider_chat_id`, `blob_id`, `media_type`, `status`, `reconciliation_status`.
- `TelegramMediaUploadKind` — `'photo' | 'video' | 'document' | 'audio' | 'voice' | 'sticker' | 'animation'`.

Функция `uploadTelegramMedia` посылает `POST` на `/api/v1/integrations/telegram/provider-media/upload`. Сообщение об ошибке: `'Telegram media upload failed'`.

### Поиск сообщений провайдера

Функция `searchTelegramProviderMessages` принимает параметры `q`, `account_id`, опциональные `provider_chat_id` и `limit`, отправляет `POST` на `/api/v1/integrations/telegram/provider-search`. Ответ имеет тип `TelegramProviderSearchCommandResponse` (поля `account_id`, `provider_chat_id?`, `query`, `limit`, `status`, `error?`). Ошибка: `'Telegram provider search trigger failed'`.

### QR-авторизация

Определены четыре API-вызова:

- `startTelegramQrLogin` — `POST` на `.../login/qr/start`.
- `getTelegramQrLoginStatus(setupId)` — `GET` на `.../login/qr/{setupId}`.
- `submitTelegramQrPassword(setupId, password)` — `POST` на `.../login/qr/{setupId}/password`.
- `cancelTelegramQrLogin(setupId)` — `DELETE` на `.../login/qr/{setupId}`.

Ответ статуса QR-входа содержит поля `setup_id`, `account_id`, `status` (`'waiting_qr_scan'`, `'waiting_password'`, `'ready'` и др.), `qr_svg`, `telegram_user_id`, `suggested_account_id` и т.д.

## Ключи запросов (Query Keys)

Константы собраны в объекте `telegramQueryKeys` (файл `queries/telegramQueryKeys.ts`):

- `capabilities` — `['integrations', 'telegram', 'capabilities']`
- `accountCapabilities` — `['integrations', 'telegram', 'account-capabilities']`
- `accounts` — `['integrations', 'telegram', 'accounts']`
- `chats` — `['integrations', 'telegram', 'provider-conversations']`
- `folders` — `['integrations', 'telegram', 'provider-folders']`
- `chatDetail` — `['integrations', 'telegram', 'provider-conversation-detail']`
- `chatMembers` — `['integrations', 'telegram', 'provider-conversation-members']`
- `runtime` — `['integrations', 'telegram', 'runtime']`
- `calls` — `['integrations', 'telegram', 'provider-calls']`
- `callTranscript` — `['integrations', 'telegram', 'provider-call-transcript']`

## Хуки запросов (Queries)

Файлы `queries/useTelegramQuery.ts`, `useTelegramRuntimeQuery.ts`, `useTelegramQrLoginQuery.ts`, `useTelegramLifecycleQuery.ts`, `useTelegramAutomationQuery.ts` предоставляют хуки `useQuery` из `@tanstack/vue-query`.

### Общие возможности

- `useTelegramCapabilitiesQuery()` — общие возможности интеграции.
- `useTelegramAccountCapabilitiesQuery(accountId)` — возможности конкретного аккаунта; запрос включается, только если `accountId` истинно.

### Аккаунты и папки

- `useTelegramAccountsQuery()` — список аккаунтов (`TelegramAccount[]`).
- `useTelegramFoldersQuery(accountId?)` — папки (фильтры чатов); ключ содержит `accountId` или `'all'`.

### Звонки

- `useTelegramCallsQuery(accountId?, limit?)` — спроецированные звонки; ключ содержит `accountId` (или `'all'`) и `limit`.
- `useTelegramCallTranscriptQuery(callId)` — расшифровка звонка; включается когда `callId` истинно; возвращает `transcript` из ответа.

### Runtime

- `useTelegramRuntimeStatusQuery(accountId)` — статус рантайма (ключа: `runtime` + `accountId`); включается при истинном `accountId`.

### QR-вход

- `useTelegramQrLoginStatusQuery(setupId)` — статус QR-сессии; ключ: `qr-login-status` + `setupId`; возвращает `null`, если `setupId` ложный.

### Команды (Lifecycle)

- `useTelegramCommandsQuery(accountId, limit?, enabled?, filters?)` — запрос списка команд `TelegramProviderWriteCommand[]`. Фильтры (`filters`) позволяют ограничить по `providerChatId`, `providerMessageId`, `commandKinds`. Ключ запроса сложный: позиции 0–2 фиксированы (`['integrations', 'telegram', 'commands']`), далее `accountId`, `limit`, `providerChatId`, `providerMessageId`, коды команд (отсортированы, соединены через `|`). Запрос выполняет `fetchTelegramCommands`, возвращает `items`.

### Автоматизация

- `useTelegramAutomationPoliciesQuery(accountId)` — получает политики, фильтрует по `account_id` на клиенте.
- `useTelegramAutomationTemplatesQuery()` — шаблоны автоматизации.

## Мутации (Mutations)

Файлы `queries/useTelegramMutations.ts`, `useTelegramRuntimeQuery.ts`, `useTelegramQrLoginQuery.ts`, `useTelegramLifecycleQuery.ts`, `useTelegramMembersQuery.ts`, `useTelegramParticipantLifecycleQuery.ts`, `useTelegramAutomationQuery.ts` определяют мутации через `useMutation`.

### Аккаунт и рантайм

- `useSetupTelegramAccountMutation` — мутация `setupTelegramAccount`; инвалидирует `accounts`, `capabilities`, `runtime`.
- `useLogoutTelegramAccountMutation` — `logoutTelegramAccount(accountId)`; инвалидирует `accounts`, `runtime`, `capabilities`.
- `useRemoveTelegramAccountMutation` — `removeTelegramAccount(accountId)`; инвалидирует `accounts`, `runtime`, `capabilities`, `chats`, `folders`.
- `useStartTelegramRuntimeMutation`, `useStopTelegramRuntimeMutation`, `useRestartTelegramRuntimeMutation` — каждая принимает `{ account_id }`, вызывает соответствующий API, инвалидирует `runtime` и `accounts`.

### Синхронизация и загрузка

- `useSyncTelegramChatsMutation` — `syncTelegramChats`; инвалидирует `chats`, `folders`, `runtime`.
- `useSyncTelegramHistoryMutation` — `syncTelegramHistory`; инвалидирует `chats`.
- `useIngestTelegramFixtureMutation` — `ingestTelegramFixtureMessage`; инвалидирует `chats`.
- `useDownloadTelegramMediaMutation` — `downloadTelegramMedia`; инвалидирует `runtime`.

### Управление чатами (pin/archive/mute/папки/read)

Используется вспомогательная функция `useTelegramChatLifecycleMutation`, которая инвалидирует `chats` и, опционально, `folders` и `chatDetail`. Через неё построены:

- `usePinTelegramChatMutation` / `useUnpinTelegramChatMutation`
- `useArchiveTelegramChatMutation` / `useUnarchiveTelegramChatMutation`
- `useMuteTelegramChatMutation` / `useUnmuteTelegramChatMutation`

Отдельно:

- `useAddTelegramChatToFolderMutation` / `useRemoveTelegramChatFromFolderMutation` — инвалидируют `chats`, `folders`, `chatDetail`.
- `useReassignTelegramChatFoldersMutation` — также инвалидирует `chats`, `folders`, `chatDetail`.
- `useMarkReadTelegramChatMutation` / `useMarkUnreadTelegramChatMutation` — инвалидируют `chats`, `chatDetail`.

### Присоединение/выход из чата

- `useJoinTelegramChatMutation` — `joinTelegramChat`, передаёт `accountId`, `providerChatId`. При успехе вызывает `primeTelegramParticipantLifecycleCommandCache` (оптимистичное добавление команды в кеш) и инвалидирует `chats`, `chatDetail`, `chatMembers`, `runtime`, `commands`.
- `useLeaveTelegramChatMutation` — `leaveTelegramChat`, требует `telegramChatId`. Аналогично кеширует команду и инвалидирует те же ключи.

### Синхронизация участников

- `useSyncTelegramChatMembersMutation` — `syncTelegramChatMembers(telegramChatId)`; инвалидирует `chatMembers`, `chats`, `chatDetail`, `runtime`.

### QR-вход

- `useStartTelegramQrLoginMutation` — мутация `startTelegramQrLogin`.
- `useCancelTelegramQrLoginMutation(setupId)` — мутация `cancelTelegramQrLogin(setupId)`; при успехе удаляет запрос статуса с ключом `[..., setupId]`.
- `useSubmitTelegramQrPasswordMutation(setupId)` — мутация `submitTelegramQrPassword(setupId, request)`; требует истинный `setupId`.

### Команды

- `useTelegramCommandRetryMutation` — `retryTelegramCommand`; при успехе инвалидирует `['integrations', 'telegram', 'commands', command.account_id]` и общий ключ `['integrations', 'telegram', 'commands']`.

### Автоматизация

- `useTelegramSendDryRunMutation()` — выполняет `runTelegramSendDryRun`.

## Realtime-обновления команд

Файл `queries/realtimeTelegramCommandPatches.ts` содержит логику применения realtime-патчей к кешу команд.

- `applyTelegramCommandRealtimePatch(eventData, queryClient)` — принимает строку JSON с конвертом `storedEventEnvelope`, извлекает `event_type`. Если тип начинается с `'telegram.'`, ищет все активные запросы с ключом `['integrations', 'telegram', 'commands']` и вызывает `patchTelegramCommandList`.

- `patchTelegramCommandList(queryKey, commands, eventType, payload)` обрабатывает события:
  - `'telegram.command.status_changed'`
  - `'telegram.command.reconciled'`
  - `'telegram.media.upload.started'`
  - `'telegram.media.upload.progress'`

  Если команда с `command_id` из `payload` уже есть в списке, обновляются поля: `status`, `retry_count`, `max_retries`, `last_error`, `result_payload`, `next_attempt_at`, `last_attempt_at`, `provider_observed_at`, `provider_state`, `reconciliation_status`, `reconciled_at`, `dead_lettered_at`, `completed_at`, `updated_at`.
  Если команда отсутствует и в `payload` есть `account_id`, создаётся новая запись (вставка в начало списка). При этом соблюдаются фильтры, закодированные в `queryKey`: `accountId` (позиция 3), `providerChatId` (5), `providerMessageId` (6) и `commandKinds` (7). Новый `command_kind` определяется по `eventType` или `payload.command_kind`, а также по `payload.action` (для `'join'` и `'leave'`). `actor_id` по умолчанию `'makosh-frontend'`.

## Форма настройки аккаунта Telegram

Файл `forms/telegramAccountSetupForm.ts`.

Схема `telegramAccountSetupSchema` построена на `zod` через `toTypedSchema` (vee-validate). Поля:

- `account_id` (строка, обязательное)
- `provider_kind`: `'telegram_user' | 'telegram_bot'`
- `display_name`, `external_account_id` — обязательные строки
- `api_id` (опционально, положительное целое) и `api_hash` (строка)
- `bot_token` (строка)
- `session_encryption_key` (строка)
- `tdlib_data_path` (строка)
- `qr_authorized` (boolean)
- `transcription_enabled` (boolean)

Логика `superRefine`:

- Для `telegram_user`:
  - Если `qr_authorized`, то требуется `tdlib_data_path`.
  - Иначе требуются `api_id` (положительное) и `api_hash`.
- Для `telegram_bot`: требуется `bot_token`.

Тип `TelegramAccountSetupFormValues` соответствует схеме. Функция `defaultTelegramAccountSetupValues()` возвращает значения по умолчанию: `provider_kind: 'telegram_user'`, `qr_authorized: false`, `transcription_enabled: false`, остальные строки пустые.

## Хранилище аудита команд (Store)

Файл `stores/telegramCommandAudit.ts` (протестирован в `telegramCommandAudit.test.ts`) экспортирует функции для отображения состояния команд в UI:

- `telegramCommandRetrySummary(command)` — возвращает строку вида `"N/M retries used"` или `"No retry budget"` (если `max_retries === 0`).
- `isTelegramCommandDeadLetter(command)` — `true`, если статус `'dead_letter'` или (`status === 'failed'` и `retry_count >= max_retries > 0`).
- `telegramCommandAuditState(command)` — возвращает объект `{ label, detail, tone, is_dead_letter }`.
  Учитывает:
  - статус команды (`queued`, `executing`, `completed`, `failed`, `dead_letter`)
  - тип команды (`send_media`, `mark_read`, `mark_unread`, `folder_add`, `folder_remove`, `edit`, `delete`, `react`, `unreact`, `pin`, `unpin`, `archive`, `unarchive` и др.)
  - содержимое `payload` и `provider_state` для формирования читаемого `detail`
  - `reconciliation_status === 'mismatch'` для описаний расхождений с провайдером.
- `telegramCommandSubject(command)` — короткое пользовательское описание команды (например, `"Edit message"`, `"Add reaction 👍"`, `"Pin chat"`, `"Read through chat-1:777"`, `"Remove chat from folder 9"`).

Эти функции не делают сетевых запросов и работают с объектами типа `TelegramProviderWriteCommand`.

## Компоненты (Vue)

Граничные тесты (`.boundary.test.ts`) подтверждают зависимости ключевых компонентов без прямого доступа к исходным `.vue` файлам. Каждый компонент использует хуки `@tanstack/vue-query`, не содержит прямых `fetch()`, а также (где применимо) не владеет WebSocket/EventSource.

### TelegramAccountManager.vue

Использует:

- `vee-validate`
- `useTelegramAccountsQuery`, `useSetupTelegramAccountMutation`, `useLogoutTelegramAccountMutation`, `useRemoveTelegramAccountMutation`
- `telegramAccountSetupSchema`
- дочерние компоненты `TelegramCapabilityMatrix`, `TelegramQrLoginPanel`
- `setFieldValue`
- `props.selectedAccountId`

### TelegramCallTranscriptPanel.vue

Использует:

- `useTelegramCallTranscriptQuery`
- переводы `t('Transcript')`, `t('No transcript projected for this call yet.')`

### TelegramCallsPanel.vue

Использует:

- `useTelegramCallsQuery`
- локальное вычисляемое свойство `filteredCalls`
- дочерний компонент `TelegramCallTranscriptPanel`
- переводы `t('Search projected calls')`, `t('Recent Calls')`

### TelegramCapabilityMatrix.vue

Использует:

- `useTelegramAccountCapabilitiesQuery`
- свойства `planned_features`, `unsupported_features`, `capability.operation`, `capability.status`
- `confirmation_required`, `closure_gate`
- перевод `t('Capabilities')`

### TelegramCommandAuditPanel.vue

Использует:

- `useTelegramCommandsQuery`
- `providerChatId: computed(() => ...)`
- `telegramCommandAuditState`, `telegramCommandSubject`, `telegramCommandRetrySummary`
- `filteredCommands`
- переводы `t('Current chat only')`, `t('Search command rows')`, `t('Recent Commands')`
- CSS-класс `telegram-command-audit__item--dead-letter`

### TelegramQrLoginPanel.vue

Использует:

- `useStartTelegramQrLoginMutation`, `useTelegramQrLoginStatusQuery`, `useCancelTelegramQrLoginMutation`, `useSubmitTelegramQrPasswordMutation`
- переводы `t('Start QR')`, `t('Apply Suggested Account')`

### TelegramStatusMessages.vue

Получает через пропсы (или computed) строковые поля:

- `realtimeStatusLabel: string`
- `realtimeStatusDetail: string`
- `realtimeRecoveryDetail: string`
- `realtimeStatusTone: 'neutral' | 'success' | 'warning' | 'danger'`

Использует `useRealtimeStatusStore`, вызывая `realtimeStatus.canTriggerReconnect` и `realtimeStatus.requestReconnect()`. В шаблоне присутствуют интерполяции `{{ t('Realtime') }}: {{ realtimeStatusLabel }}` и `{{ t('Recovery') }}: {{ realtimeRecoveryDetail }}`, а также кнопка с текстом `t('Reconnect realtime')`. Компонент не содержит `fetch(`, `new WebSocket`, `EventSource`.
```

---

## Покрытие источников

| Файл | Факты, покрытые в предложенной странице |
|------|----------------------------------------|
| `frontend/src/integrations/telegram/api/telegramMediaUpload.test.ts` | Тестирует endpoint, поля запроса/ответа. Описан как тест API-клиента. |
| `frontend/src/integrations/telegram/api/telegramMediaUpload.ts` | Типы `TelegramMediaUploadRequest`, `TelegramMediaUploadResponse`, `TelegramMediaUploadKind`; функция `uploadTelegramMedia` и endpoint. |
| `frontend/src/integrations/telegram/api/telegramQr.test.ts` | Маршруты QR-входа (`start`, `status`, `password`, `cancel`), методы, ответы. |
| `frontend/src/integrations/telegram/api/telegramSearch.test.ts` | Endpoint поиска, форма тела запроса. |
| `frontend/src/integrations/telegram/api/telegramSearch.ts` | Функция `searchTelegramProviderMessages`, endpoint, тип ответа. |
| `frontend/src/integrations/telegram/components/TelegramAccountManager.boundary.test.ts` | Зависимости компонента: vee-validate, хуки, схема, дочерние компоненты, `setFieldValue`, пропс `selectedAccountId`. |
| `frontend/src/integrations/telegram/components/TelegramCallTranscriptPanel.boundary.test.ts` | Использование `useTelegramCallTranscriptQuery`, переводы, отсутствие `fetch`. |
| `frontend/src/integrations/telegram/components/TelegramCallsPanel.boundary.test.ts` | Использование `useTelegramCallsQuery`, `filteredCalls`, дочерний компонент, переводы, отсутствие `fetch`. |
| `frontend/src/integrations/telegram/components/TelegramCapabilityMatrix.boundary.test.ts` | Использование `useTelegramAccountCapabilitiesQuery`, свойства `planned_features` и др., переводы, `confirmation_required`, `closure_gate`, отсутствие `fetch`. |
| `frontend/src/integrations/telegram/components/TelegramCommandAuditPanel.boundary.test.ts` | Использование `useTelegramCommandsQuery`, `providerChatId`, функций аудита, `filteredCommands`, переводы, CSS-класс, отсутствие `fetch`. |
| `frontend/src/integrations/telegram/components/TelegramQrLoginPanel.boundary.test.ts` | Использование QR-мутаций и запроса статуса, переводы, отсутствие `fetch`. |
| `frontend/src/integrations/telegram/components/TelegramStatusMessages.boundary.test.ts` | Пропсы `realtimeStatusLabel`, `realtimeRecoveryDetail`, `realtimeStatusTone`; использование `useRealtimeStatusStore`, `requestReconnect()`, `canTriggerReconnect`; переводы; отсутствие fetch/WebSocket/EventSource. |
| `frontend/src/integrations/telegram/forms/telegramAccountSetupForm.ts` | Схема zod, поля, логика валидации, `TelegramAccountSetupFormValues`, `defaultTelegramAccountSetupValues`. |
| `frontend/src/integrations/telegram/queries/realtimeTelegramCommandPatches.ts` | Функции `applyTelegramCommandRealtimePatch`, `patchTelegramCommandList`; поддерживаемые типы событий, фильтры по queryKey, логика обновления/вставки, установка `actor_id`, `command_kind` и прочих полей. |
| `frontend/src/integrations/telegram/queries/telegramQueryKeys.ts` | Все ключи запросов, перечисленные на странице. |
| `frontend/src/integrations/telegram/queries/useTelegramAutomationQuery.ts` | Хуки `useTelegramAutomationPoliciesQuery`, `useTelegramAutomationTemplatesQuery`, `useTelegramSendDryRunMutation`. |
| `frontend/src/integrations/telegram/queries/useTelegramLifecycleQuery.ts` | Хук `useTelegramCommandsQuery`, ключ запроса с фильтрами, `useTelegramCommandRetryMutation` и инвалидация. |
| `frontend/src/integrations/telegram/queries/useTelegramMembersQuery.ts` | Мутация `useSyncTelegramChatMembersMutation`, инвалидация ключей. |
| `frontend/src/integrations/telegram/queries/useTelegramMutations.ts` | Все перечисленные мутации и инвалидируемые ключи. |
| `frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.test.ts` | Функция `primeTelegramParticipantLifecycleCommandCache`, её поведение при вставке join/leave команд. |
| `frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.ts` | `primeTelegramParticipantLifecycleCommandCache`, мутации `useJoinTelegramChatMutation`, `useLeaveTelegramChatMutation` и инвалидация. |
| `frontend/src/integrations/telegram/queries/useTelegramQrLoginQuery.ts` | Ключи QR-статуса, мутации `useStartTelegramQrLoginMutation`, `useCancelTelegramQrLoginMutation`, `useSubmitTelegramQrPasswordMutation`, запрос `useTelegramQrLoginStatusQuery`. |
| `frontend/src/integrations/telegram/queries/useTelegramQuery.ts` | Хуки запросов: `useTelegramCapabilitiesQuery`, `useTelegramAccountCapabilitiesQuery`, `useTelegramAccountsQuery`, `useTelegramFoldersQuery`, `useTelegramCallsQuery`, `useTelegramCallTranscriptQuery`; вычисляемые ключи. |
| `frontend/src/integrations/telegram/queries/useTelegramRuntimeQuery.ts` | Хук `useTelegramRuntimeStatusQuery`, мутации `useStartTelegramRuntimeMutation`, `useStopTelegramRuntimeMutation`, `useRestartTelegramRuntimeMutation` и их инвалидация. |
| `frontend/src/integrations/telegram/stores/telegramCommandAudit.test.ts` (частично) | Функции `telegramCommandRetrySummary`, `isTelegramCommandDeadLetter`, `telegramCommandAuditState`, `telegramCommandSubject`; описание логики для различных видов команд и статусов. |

---

## Исходные файлы

- [`frontend/src/integrations/telegram/api/telegramMediaUpload.test.ts`](../../../../frontend/src/integrations/telegram/api/telegramMediaUpload.test.ts)
- [`frontend/src/integrations/telegram/api/telegramMediaUpload.ts`](../../../../frontend/src/integrations/telegram/api/telegramMediaUpload.ts)
- [`frontend/src/integrations/telegram/api/telegramQr.test.ts`](../../../../frontend/src/integrations/telegram/api/telegramQr.test.ts)
- [`frontend/src/integrations/telegram/api/telegramSearch.test.ts`](../../../../frontend/src/integrations/telegram/api/telegramSearch.test.ts)
- [`frontend/src/integrations/telegram/api/telegramSearch.ts`](../../../../frontend/src/integrations/telegram/api/telegramSearch.ts)
- [`frontend/src/integrations/telegram/components/TelegramAccountManager.boundary.test.ts`](../../../../frontend/src/integrations/telegram/components/TelegramAccountManager.boundary.test.ts)
- [`frontend/src/integrations/telegram/components/TelegramCallTranscriptPanel.boundary.test.ts`](../../../../frontend/src/integrations/telegram/components/TelegramCallTranscriptPanel.boundary.test.ts)
- [`frontend/src/integrations/telegram/components/TelegramCallsPanel.boundary.test.ts`](../../../../frontend/src/integrations/telegram/components/TelegramCallsPanel.boundary.test.ts)
- [`frontend/src/integrations/telegram/components/TelegramCapabilityMatrix.boundary.test.ts`](../../../../frontend/src/integrations/telegram/components/TelegramCapabilityMatrix.boundary.test.ts)
- [`frontend/src/integrations/telegram/components/TelegramCommandAuditPanel.boundary.test.ts`](../../../../frontend/src/integrations/telegram/components/TelegramCommandAuditPanel.boundary.test.ts)
- [`frontend/src/integrations/telegram/components/TelegramQrLoginPanel.boundary.test.ts`](../../../../frontend/src/integrations/telegram/components/TelegramQrLoginPanel.boundary.test.ts)
- [`frontend/src/integrations/telegram/components/TelegramStatusMessages.boundary.test.ts`](../../../../frontend/src/integrations/telegram/components/TelegramStatusMessages.boundary.test.ts)
- [`frontend/src/integrations/telegram/forms/telegramAccountSetupForm.ts`](../../../../frontend/src/integrations/telegram/forms/telegramAccountSetupForm.ts)
- [`frontend/src/integrations/telegram/queries/realtimeTelegramCommandPatches.ts`](../../../../frontend/src/integrations/telegram/queries/realtimeTelegramCommandPatches.ts)
- [`frontend/src/integrations/telegram/queries/telegramQueryKeys.ts`](../../../../frontend/src/integrations/telegram/queries/telegramQueryKeys.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramAutomationQuery.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramAutomationQuery.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramLifecycleQuery.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramLifecycleQuery.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramMembersQuery.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramMembersQuery.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramMutations.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramMutations.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.test.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.test.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramParticipantLifecycleQuery.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramQrLoginQuery.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramQrLoginQuery.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramQuery.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramQuery.ts)
- [`frontend/src/integrations/telegram/queries/useTelegramRuntimeQuery.ts`](../../../../frontend/src/integrations/telegram/queries/useTelegramRuntimeQuery.ts)
- [`frontend/src/integrations/telegram/stores/telegramCommandAudit.test.ts`](../../../../frontend/src/integrations/telegram/stores/telegramCommandAudit.test.ts)

## Кандидаты на drift

Из предоставленного контекста расхождения не видны. Граничные тесты проверяют, что определённые компоненты Vue используют конкретные хуки и не содержат прямых `fetch()`-вызовов, однако сами `.vue` файлы не встроены в этот чанк, поэтому подтвердить или опровергнуть соответствие невозможно. Внутренних противоречий между встроенными файлами не обнаружено.
