## Summary / Резюме

Создать новую страницу `components/frontend.md` в русской Obsidian wiki с систематизированным описанием фронтенд-компонентов для интеграций Telegram, WhatsApp, Yandex Telemost и Zoom. Страница документирует назначение каждого компонента, его пропсы, события и структуру шаблона на основе предоставленного исходного кода, чтобы улучшить навигацию и понимание фронтенд-части проекта.

## Proposed pages / Предлагаемые страницы

### components/frontend.md

```markdown
# Компоненты фронтенда

Данная страница содержит описание Vue-компонентов, реализующих UI для интеграций Макошь. Компоненты расположены в `frontend/src/integrations/<integration>/components/` и `frontend/src/integrations/<integration>/views/`.

## Telegram

### TelegramAccountManager.vue
- **Назначение**: Управление аккаунтами Telegram (добавление, просмотр, выход, удаление).
- **Пропсы**: `selectedAccountId` (строка или null).
- **Состояние**: Локально управляет формой настройки аккаунта через `vee-validate`. Поля: `account_id`, `provider_kind` (`telegram_user` / `telegram_bot`), `display_name`, `external_account_id`, `api_id`, `api_hash`, `bot_token`, `session_encryption_key`, `tdlib_data_path`, `qr_authorized`, `transcription_enabled`.
- **Мутации/Запросы**: `useSetupTelegramAccountMutation`, `useLogoutTelegramAccountMutation`, `useRemoveTelegramAccountMutation`, `useTelegramAccountsQuery`.
- **Шаблон**: Заголовок с кнопкой открытия формы настройки. Форма с полями, зависящими от типа провайдера: для `telegram_user` поля `api_id`, `api_hash` и QR-авторизация через `TelegramQrLoginPanel`; для `telegram_bot` – поле `bot_token`. Общие поля: `session_encryption_key`, `tdlib_data_path`, чекбокс транскрипции. Список аккаунтов с жизненным циклом (`lifecycle_state` влияет на стиль: active/muted/danger), кнопками Logout/Remove. Внизу компонент `TelegramCapabilityMatrix`.

### TelegramCallTranscriptPanel.vue
- **Назначение**: Отображение транскрипта конкретного звонка Telegram.
- **Пропсы**: `calls` (массив `TelegramCall`).
- **Запрос**: `useTelegramCallTranscriptQuery(selectedCallId)` для получения транскрипта по выбранному `call_id`.
- **Шаблон**: Список кнопок-строк звонков (статус, `provider_chat_id`, дата). При выборе звонка отображается карточка транскрипта с полями: `transcript_status`, `stt_provider`, `language_code`, `source_audio_ref`, `transcript_text`.

### TelegramCallsPanel.vue
- **Назначение**: Панель последних звонков Telegram с фильтрацией.
- **Пропсы**: `selectedAccountId`.
- **Запрос**: `useTelegramCallsQuery(selectedAccountId, 10)`.
- **Шаблон**: Заголовок со счётчиком звонков. Строка поиска (при наличии звонков). Плейсхолдеры: «выберите аккаунт», «загрузка», «нет звонков», «нет совпадений». Передаёт отфильтрованные звонки в `TelegramCallTranscriptPanel`.

### TelegramCapabilityMatrix.vue
- **Назначение**: Матрица возможностей аккаунта Telegram с деталями операций.
- **Пропсы**: `accountId`.
- **Запрос**: `useTelegramAccountCapabilitiesQuery(accountId)`.
- **Шаблон**: Если нет аккаунта – плейсхолдер. При загрузке – индикатор. При наличии данных: секция `account_scope` (`account_id`, `provider_kind`, `runtime_kind`, `lifecycle_state`), плановые фичи (`planned_features`), неподдерживаемые фичи (`unsupported_features`), список capability-строк с полями: `operation`, `status` (цветовая индикация), `category`, `action_class`, `reason`, `confirmation_required`, `closure_gate`.

### TelegramCommandAuditPanel.vue
- **Назначение**: Аудит провайдерских команд Telegram (отложенные/завершённые).
- **Пропсы**: `selectedAccountId`, `selectedProviderChatId`.
- **Запросы/Мутации**: `useTelegramCommandsQuery` (с опциональным фильтром по `providerChatId` и `currentChatOnly`), `useTelegramCommandRetryMutation`.
- **Хранилище**: `telegramCommandAudit` (функции `telegramCommandAuditState`, `telegramCommandRetrySummary`, `telegramCommandSubject`).
- **Шаблон**: Заголовок с переключателем «Только текущий чат». Строка поиска. Список команд: `command_kind`, `status`, `happened_at`, `subject`, бейджи состояния и сводки повторов, детали (`capability_state`, `action_class`, `confirmation_decision`, `reconciliation_status`). Для команд со статусом `dead_letter` или `failed` доступна кнопка повтора.

### TelegramQrLoginPanel.vue
- **Назначение**: Панель авторизации пользовательского аккаунта Telegram через QR-код (TDLib).
- **Пропсы**: `formValues` (объект `TelegramAccountSetupFormValues`).
- **События**: `applySuggested` – передаёт предложенные данные аккаунта после успешного QR-входа.
- **Управление сессией**: `useStartTelegramQrLoginMutation`, `useTelegramQrLoginStatusQuery`, `useCancelTelegramQrLoginMutation`, `useSubmitTelegramQrPasswordMutation`.
- **Шаблон**: Кнопка «Start QR» (активна при заполненных `account_id`, `display_name`, `external_account_id`, `tdlib_data_path`). Отображение статуса и сообщения QR-сессии. SVG QR-кода. Поле ввода пароля 2FA (при статусе `waiting_password`). Кнопки: обновить статус, отправить пароль, применить предложенный аккаунт, отменить QR. После успешной авторизации – блок с предложенными `account_id`, `display_name`, `external_account_id`.

### TelegramStatusMessages.vue
- **Назначение**: Отображение статусных сообщений реального времени и ошибок.
- **Пропсы**: `actionMessage`, `error`, `realtimeStatusLabel`, `realtimeStatusDetail`, `realtimeRecoveryDetail`, `realtimeStatusTone`.
- **Хранилище**: `useRealtimeStatusStore`.
- **Шаблон**: Строки: состояние realtime с тоном, состояние восстановления с кнопкой переподключения (если разрешено). Сообщение действия (success) и ошибка.

### TelegramRuntimePanel.vue (вид)
- **Назначение**: Страница управления Telegram Runtime.
- **Состояние**: `selectedAccountId`, `actionMessage`, `actionError`.
- **Запросы/Мутации**: `useTelegramAccountsQuery`, `useTelegramCapabilitiesQuery`, `useTelegramRuntimeStatusQuery`, `useStartTelegramRuntimeMutation`, `useStopTelegramRuntimeMutation`, `useRestartTelegramRuntimeMutation`.
- **Хранилище**: `useRealtimeStatusStore`.
- **Шаблон**: Заголовок с кнопкой обновления. Индикаторы realtime, сообщений действия и ошибки. Сетка: левая панель «Accounts» (выпадающий список аккаунтов, `TelegramAccountManager`), правая панель «Runtime» (кнопки Start/Stop/Restart, детали: аккаунт, режим, доступность TDLib, последняя синхронизация). Внизу `TelegramCapabilityMatrix`.

## WhatsApp

### WhatsAppRail.vue
- **Назначение**: Боковая панель сводки WhatsApp (аккаунты, guardrails, фикстурные сообщения).
- **Пропсы**: `whatsappCapabilities`, `whatsappClosureCapabilities`, `whatsappBlockedCapabilities`, `whatsappProviderAccounts`, `isWhatsappActionSubmitting`, `openAccountDrawer`, `ingestWhatsappWebMessageFixture`, `whatsappMessageForm`.
- **Шаблон**: Три секции:
  1. «Accounts»: количество аккаунтов, кнопка «Open Wizard».
  2. «Runtime Guardrails»: режим (`runtime_mode`), списки closure/blocked capabilities и unsupported features с метками.
  3. «Ingest Fixture Message»: форма с полями `account_id`, `provider_chat_id`, `chat_title`, `sender_id`, `sender_display_name`, `text`, кнопка «Ingest Fixture».

### WhatsAppRuntimeAccountList.vue
- **Назначение**: Список аккаунтов WhatsApp с выбором.
- **Пропсы**: `accounts` (массив `WhatsappAccountSummary`), `selectedAccountId`, `includeRemovedAccounts`.
- **События**: `update:includeRemovedAccounts`, `select-account`.
- **Шаблон**: Заголовок со счётчиком. Чекбокс «Include removed». Список кнопок-строк аккаунтов: `display_name`, `account_id`, `provider_shape` или `provider_kind`, `lifecycle_state`. Пустое состояние при отсутствии аккаунтов.

### WhatsAppRuntimeAccountProvisioning.vue
- **Назначение**: Форма создания (провижининга) живого аккаунта WhatsApp.
- **Пропсы**: `capabilities`, `liveAccountProviderKind`, `liveAccountShape`, `liveAccountId`, `liveAccountDisplayName`, `liveAccountExternalId`, `liveAccountDeviceName`, `liveAccountLocalStatePath`, `liveAccountSupportsDeviceFields`, `selectedProviderShapeMeta`, `liveAccountSessionMode`, `isSubmitting`.
- **События**: Двусторонняя привязка (`update:...`) для полей аккаунта; `create-live-account`.
- **Шаблон**: Заголовок с `provider_kind`. Сетка полей: выбор `provider_shape` из `capabilities.provider_shapes`, вводы `account_id`, `display_name`, `external_account_id`, `device_name` (если поддерживается), `local_state_path`. Информационная строка с `reason` выбранной формы. Детали: `runtime_mode`, `capability_status`. Кнопка «Create Live Account».

### WhatsAppRuntimeCapabilities.vue
- **Назначение**: Отображение возможностей (capabilities) аккаунта WhatsApp.
- **Пропсы**: `runtimeCapabilities` (тип `WhatsappCapabilitiesResponse` или null).
- **Шаблон**: Заголовок с версией. Сетка провайдерских форм (`provider_shapes`) с полями: `provider_shape`, `status`, `reason`. Список `capabilities` с парами `capability`/`status`.

### WhatsAppRuntimeCommandAudit.vue
- **Назначение**: Аудит провайдерских команд WhatsApp.
- **Пропсы**: `providerCommands` (массив `WhatsAppProviderCommand`), `isRuntimeBusy`.
- **События**: `retry`, `dead-letter`.
- **Хелперы**: `canDeadLetterCommand`, `canRetryCommand`, `commandStatusTone`, `commandTimestamp`, `providerTargetLabel` из `WhatsAppRuntimePanel.helpers`.
- **Шаблон**: Заголовок со счётчиком. Список команд: `command_kind`, `status`, `target`, детали (`capability_state`, `reconciliation_status`, `retry_count`/`max_retries`, `updated`), `last_error`. Кнопки «Retry» и «Dead-letter» (с учётом разрешений).

### WhatsAppRuntimeControl.vue
- **Назначение**: Панель управления рантаймом WhatsApp.
- **Пропсы**: `selectedAccountId`, `selectedAccountSummary`, `runtimeStatus`, `runtimeCapabilities`, `runtimeHealth`, `runtimeHealthChecks`, `companionOpenManifest`, `canOpenWebCompanion`, `isRuntimeBusy`.
- **События**: `open-companion`, `set-runtime-state` (действия: start, stop, revoke, relink, rotate, remove).
- **Шаблон**: Заголовок со сводкой аккаунта. Кнопки действий. Детали рантайма: `lifecycle`, `provider_shape`, `runtime_kind`, `session_restore_available`, `healthy`, `last_error`. Блокировщики (`runtime_blockers`). Информация о WebView-компаньоне (`window_label`, `relay_channel`, `runtime_bridge_dispatch`). Диагностика здоровья (список проверок с `checkName`, `healthCheckStatus`, `healthCheckDetail`).

### WhatsAppRuntimeLinking.vue
- **Назначение**: Панель линковки WhatsApp (QR, Pair Code).
- **Пропсы**: `runtimeStatus`, `selectedAccountId`, `isRuntimeBusy`, `pairCodePhoneNumber`, `activeQrSession`, `activePairCodeSession`.
- **События**: `update:pairCodePhoneNumber`, `set-runtime-state` (qr, pair_code).
- **Шаблон**: Заголовок со статусом. Кнопки «Start QR Link», «Start Pair Code» (последняя требует номера телефона). Поле ввода номера. Отображение QR-сессии (qr_svg) и Pair Code (pair_code, phone_number).

### WhatsAppRuntimeSnapshots.vue
- **Назначение**: Отображение снимков синхронизированных данных WhatsApp.
- **Пропсы**: `selectedAccountId`, `selectedSyncChatIdResolved`, `chatItems`, `historyItems`, `memberItems`, `statusItems`, `presenceItems`, `callItems`, `contactItems`, `mediaItems`, `statusPublishText`, `isRuntimeBusy`.
- **События**: `select-chat`, `update:statusPublishText`, `publish-status`.
- **Хелперы**: функции форматирования меток и меток времени.
- **Шаблон**: Сетка карточек:
  - «Chats»: список чатов с кнопкой выбора.
  - «History»: история сообщений выбранного чата.
  - «Members»: участники чата (роль, статус).
  - «Statuses»: текстовая область для публикации статуса, кнопка «Publish Status», список статусов.
  - «Presence»: состояния присутствия.
  - «Calls»: звонки.
  - «Contacts»: контакты.
  - «Media»: медиа-вложения.

### WhatsAppSessionList.vue
- **Назначение**: Виртуализированный список сессий WhatsApp Web.
- **Пропсы**: `whatsappSessions`, `selectedWhatsappSessionId`, `isWhatsappLoading`.
- **События**: `selectSession`.
- **Шаблон**: Строка поиска. Контейнер с виртуальным скроллом (`@tanstack/vue-virtual`). Каждая строка: иконка, `device_name`, `link_state` (цветовая индикация ok/warn/err), `companion_runtime`.

### WhatsAppStatusMessages.vue
- **Назначение**: Простой компонент для отображения сообщений действия и ошибок.
- **Пропсы**: `actionMessage`, `error`.
- **Шаблон**: Условный вывод success-сообщения и блока ошибки.

### WhatsAppRuntimePanel.vue (вид)
- **Назначение**: Главная страница управления WhatsApp Runtime. Оркеструет все вышеописанные компоненты, подключая запросы, мутации и хранилище.
- **Запросы/Мутации** (среди прочего): `useWhatsappAccountsQuery`, `useWhatsappCapabilitiesQuery`, `useWhatsappAccountCapabilitiesQuery`, `useWhatsappSessionsQuery`, `useWhatsappRuntimeStatusQuery`, `useWhatsappRuntimeHealthQuery`, `useWhatsappProviderCommandsQuery`, запросы синхронизации (`useWhatsappSyncChatsQuery`, `useWhatsappSyncHistoryQuery`, `useWhatsappSyncMembersQuery`, `useWhatsappSyncStatusesQuery`, `useWhatsappSyncPresenceQuery`, `useWhatsappSyncCallsQuery`, `useWhatsappSyncContactsQuery`, `useWhatsappSyncMediaQuery`), мутации управления рантаймом, линковкой, командами, статусами.
- **Хранилище**: `useWhatsappStore`, `useRealtimeStatusStore`.
- **Шаблон**: Заголовок с кнопкой обновления, индикатор realtime. Двухколоночная сетка: левая колонка – `WhatsAppRuntimeAccountList`, `WhatsAppRuntimeAccountProvisioning`, `WhatsAppSessionList`; правая – `WhatsAppRuntimeControl`, `WhatsAppRuntimeLinking`, `WhatsAppRuntimeCommandAudit`, `WhatsAppRuntimeSnapshots`. Внизу `WhatsAppRail`.
- **Примечание**: Исходный файл обрезан в предоставленном контексте (лимит 12000 символов), полное описание может быть неполным.

### WhatsAppRuntimePanel.css
- **Назначение**: Стили для `WhatsAppRuntimePanel.vue`. Определяет CSS-классы для сетки, карточек, деталей, кнопок и адаптивности (переключение на одну колонку при ширине ≤ 1024px).

## Yandex Telemost

### YandexTelemostSettingsPanel.vue
- **Назначение**: Панель настройки интеграции с Яндекс.Телемост: подключение аккаунта, создание конференций, открытие WebView, запись MP3.
- **Пропсы**: `selectedAccount` (объект `ProviderAccount` или null).
- **Запросы/Мутации**: `useSetupYandexTelemostAccountMutation`, `useYandexTelemostCapabilitiesQuery`, `useYandexTelemostRuntimeStatusQuery`.
- **API-функции**: `createYandexTelemostConference`, `openYandexTelemostCompanion`, `startYandexTelemostRecording`, `stopYandexTelemostRecording`, `completeYandexTelemostRecording`.
- **Состояние**: `setupForm`, `conferenceForm`, `manualOpenForm`, `lastConference`, `activeRecording`.
- **Шаблон**:
  1. Форма подключения аккаунта: `account_id`, `display_name`, `external_account_id`, `oauth_token`, `oauth_token_ref`, `api_base_url`.
  2. (При выбранном аккаунте) Информация о рантайме (`lifecycle_state`, `blockers`).
  3. Форма создания конференции: `waiting_room_level`, чекбокс авто-саммари, кнопка «Create conference».
  4. Блок открытия WebView и записи: поля `join_url`, `conference_id`, кнопки «Open in Макошь WebView», «Start local MP3 recording», «Stop recording».
  5. Отображение последней конференции и активной записи.
  6. Строка «Safety boundary» с данными из `capabilities`.

## Zoom

### ZoomAuditEventsPanel.vue
- **Назначение**: Отображение аудит-событий Zoom для выбранного аккаунта.
- **Пропсы**: `selectedAccount` (объект `ProviderAccount` или null).
- **Запрос**: `useZoomAuditEventsQuery(selectedAccountId, 12)`.
- **Шаблон**: Заголовок со счётчиком событий. Плейсхолдеры: «выберите аккаунт», «загрузка», «нет событий». Список событий: `event_type`, `occurred_at`, `subject_kind`, `subject_entity_id`, `position`, `correlation_id`.
```

## Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `TelegramAccountManager.vue` | Назначение, проп `selectedAccountId`, поля формы, мутации `setup/logout/remove`, `useTelegramAccountsQuery`, структура шаблона (форма, список аккаунтов, кнопки, `TelegramQrLoginPanel`, `TelegramCapabilityMatrix`). |
| `TelegramCallTranscriptPanel.vue` | Назначение, проп `calls`, запрос `useTelegramCallTranscriptQuery`, шаблон (список звонков, карточка транскрипта с полями). |
| `TelegramCallsPanel.vue` | Назначение, проп `selectedAccountId`, запрос `useTelegramCallsQuery`, фильтрация, плейсхолдеры, передача в `TelegramCallTranscriptPanel`. |
| `TelegramCapabilityMatrix.vue` | Назначение, проп `accountId`, запрос `useTelegramAccountCapabilitiesQuery`, выводимые данные (`account_scope`, `planned_features`, `unsupported_features`, capability-строки). |
| `TelegramCommandAuditPanel.vue` | Назначение, пропсы `selectedAccountId`/`selectedProviderChatId`, запросы/мутации (`useTelegramCommandsQuery`, `useTelegramCommandRetryMutation`), фильтр `currentChatOnly`, шаблон (заголовок, переключатель, поиск, список команд с деталями и кнопкой повтора). |
| `TelegramQrLoginPanel.vue` | Назначение, проп `formValues`, событие `applySuggested`, мутации/запросы QR-сессии, шаблон (кнопки, статус, qr_svg, поле пароля, предложенные данные). |
| `TelegramStatusMessages.vue` | Назначение, пропсы, использование `realtimeStatusStore`, шаблон (realtime, recovery, actionMessage, error). |
| `TelegramRuntimePanel.vue` | Назначение, состояние `selectedAccountId`, запросы/мутации (`useTelegramAccountsQuery`, `useTelegramCapabilitiesQuery`, `useTelegramRuntimeStatusQuery`, управление рантаймом), шаблон (сетка, панели аккаунтов и runtime). |
| `WhatsAppRail.vue` | Назначение, пропсы, функция `capabilityLabel`, шаблон (секции Accounts, Runtime Guardrails, Ingest Fixture Message). |
| `WhatsAppRuntimeAccountList.vue` | Назначение, пропсы, события, шаблон (список аккаунтов, чекбокс include removed). |
| `WhatsAppRuntimeAccountProvisioning.vue` | Назначение, пропсы (включая `capabilities`, поля формы), события, шаблон (сетка полей, выбор provider shape, детали, кнопка создания). |
| `WhatsAppRuntimeCapabilities.vue` | Назначение, проп `runtimeCapabilities`, шаблон (заголовок с версией, сетка provider_shapes, список capability). |
| `WhatsAppRuntimeCommandAudit.vue` | Назначение, пропсы `providerCommands`/`isRuntimeBusy`, события `retry`/`dead-letter`, использование хелперов, шаблон (список команд с деталями и кнопками). |
| `WhatsAppRuntimeControl.vue` | Назначение, пропсы, события `open-companion`/`set-runtime-state`, шаблон (кнопки действий, детали рантайма, блокировщики, компаньон, диагностика здоровья). |
| `WhatsAppRuntimeLinking.vue` | Назначение, пропсы, события `set-runtime-state` (qr/pair_code), шаблон (кнопки, поле телефона, отображение QR/пары). |
| `WhatsAppRuntimeSnapshots.vue` | Назначение, пропсы, события `select-chat`/`publish-status`, шаблон (сетка карточек для чатов, истории, участников, статусов, присутствия, звонков, контактов, медиа). |
| `WhatsAppSessionList.vue` | Назначение, пропсы, событие `selectSession`, виртуализация, поиск, шаблон (строка поиска, скролл-список с device_name, link_state). |
| `WhatsAppStatusMessages.vue` | Назначение, пропсы, шаблон (условное отображение actionMessage и error). |
| `WhatsAppRuntimePanel.vue` (частично) | Назначение, список импортируемых запросов/мутаций, хранилищ, общая структура (двухколоночная сетка, композиция дочерних компонентов). |
| `WhatsAppRuntimePanel.css` | Назначение, ключевые CSS-классы для макета и адаптивности. |
| `YandexTelemostSettingsPanel.vue` | Назначение, проп `selectedAccount`, мутации/запросы/API-функции, состояние форм, шаблон (подключение аккаунта, создание конференции, WebView/запись, вывод конференции/записи, safety boundary). |
| `ZoomAuditEventsPanel.vue` | Назначение, проп `selectedAccount`, запрос `useZoomAuditEventsQuery`, шаблон (заголовок со счётчиком, плейсхолдеры, список событий с полями). |

## Drift candidates / Кандидаты на drift

Расхождений между кодом, документацией и ADR в предоставленном контексте не обнаружено. Все описания компонентов основаны исключительно на встроенных исходных файлах; файлов конфигурации, документации или ADR в чанке нет.
