---
chunk_id: 151-source-frontend-part-011
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 151-source-frontend-part-011 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/frontend.md` должна быть создана (или обновлена) для документирования структуры фронтенд-кодовой базы Макошь. На основе приложенных исходных файлов описываются доменные типы (tasks, timeline), API-клиенты, паттерны хранилищ (Pinia) и запросов (TanStack Query), сгенерированный Protobuf-код (common, communications, events, signal_hub), а также интеграционные модули для почты (Mail) и Telegram с их формами, API, запросами и граничными тестами. Всё изложение опирается исключительно на представленный исходный код.

---

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Frontend

## Обзор архитектуры

Фронтенд организован по функциональным слоям:

- **domains** – предметные типы и логика (задачи, таймлайн).
- **integrations** – API, формы, запросы и компоненты для интеграций с почтой и Telegram.
- **gen** – сгенерированный из `.proto`-файлов код (Protobuf-сообщения и сервисы).
- **platform/api** – единый HTTP-клиент `ApiClient`, аутентифицирующий запросы заголовком `X-Макошь-Secret`.
- **shared** – общие типы и запросы, переиспользуемые несколькими модулями (например, синхронизация почты).

Общие паттерны:

- **API-клиент**: `ApiClient.instance` с базовым URL и секретом. Методы `get`, `post`, `put`, `delete` возвращают типизированные промисы, принимают строку сообщения об ошибке.
- **Хранилища (Pinia)**: `defineStore` (например, `timeline-ui`), реактивные `ref` и `computed`.
- **Запросы (TanStack Vue Query)**: `useQuery` / `useMutation` с ключами, `refetchOnMount: 'always'`, `staleTime`.
- **Валидация форм**: `zod` для схем, `vee-validate` с `toTypedSchema` для интеграции.
- **Тесты**: Vitest, проверяют границы (какие импорты используются/не используются) и корректность API-вызовов.

> Все факты ниже подтверждаются приложенными исходными файлами. Усечённые файлы помечены «обрезан»; утверждения основаны только на видимой части.

---

## Домены (domains)

### Tasks (задачи)

Типы описаны в `frontend/src/domains/tasks/types/task.ts`.

#### TaskCandidate

```ts
type TaskCandidateReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

interface TaskCandidate {
  task_candidate_id: string
  source_kind: 'message' | 'document'
  source_id: string
  project_id: string | null
  title: string
  due_text: string | null
  assignee_label: string | null
  confidence: number
  review_state: TaskCandidateReviewState
  evidence_excerpt: string
  generated_at: string
  reviewed_at: string | null
  updated_at: string
}
```

Список кандидатов возвращается как `TaskCandidateListResponse { items: TaskCandidate[] }`.

#### Task

```ts
interface Task {
  task_id: string; task_candidate_id: string | null; title: string; description: string | null;
  source_kind: string; source_id: string; source_type: string; project_id: string | null;
  status: string; makosh_status: string;
  priority_score: number | null; risk_score: number | null; readiness_score: number | null;
  area: string | null; why: string | null; outcome: string | null;
  due_at: string | null; completed_at: string | null; archived_at: string | null;
  waiting_reason: string | null; energy_type: string | null; confidentiality: string;
  tags: unknown[]; task_metadata: Record<string, unknown>;
  linked_person_id: string | null; linked_organization_id: string | null;
  created_from_event_id: string | null; created_by_actor_id: string | null;
  created_at: string; updated_at: string
}
```

Список задач: `TaskRecordsResponse { items: Task[] }`.

#### Decision

```ts
type DecisionEntityKind =
  | 'persona' | 'organization' | 'project' | 'communication'
  | 'document' | 'task' | 'event' | 'decision' | 'obligation' | 'knowledge'

type DecisionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

interface Decision {
  decision_id: string; title: string; status: string; rationale: string; alternatives: unknown;
  decided_by_entity_kind: DecisionEntityKind | null; decided_by_entity_id: string | null;
  decided_at: string | null;
  review_state: DecisionReviewState; confidence: number;
  metadata: Record<string, unknown>;
  created_at: string; updated_at: string
}
```

Запрос обновления статуса ревью: `DecisionReviewRequest { review_state: Exclude<DecisionReviewState, 'suggested'> }`.

#### Obligation

```ts
type ObligationEntityKind = DecisionEntityKind // те же значения
type ObligationReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
type ObligationRiskState = 'none' | 'watch' | 'at_risk' | 'breached'

interface Obligation {
  obligation_id: string
  obligated_entity_kind: ObligationEntityKind; obligated_entity_id: string
  beneficiary_entity_kind: ObligationEntityKind | null; beneficiary_entity_id: string | null
  statement: string; status: string; review_state: ObligationReviewState
  due_at: string | null; condition: string | null; risk_state: ObligationRiskState
  confidence: number; metadata: Record<string, unknown>
  created_at: string; updated_at: string
}
```

Запрос обновления статуса ревью: `ObligationReviewRequest { review_state: Exclude<ObligationReviewState, 'suggested'> }`.

### Timeline (лента сообщений)

Расположение: `frontend/src/domains/timeline/`.

#### Типы

```ts
interface TimelineMessage {
  message_id: string
  sender_display_name: string | null
  sender: string
  subject: string
  body_text_preview: string
  occurred_at: string | null
  projected_at: string
  channel_kind: string
}

type TimelineFilterKind = 'Messages' | 'Documents' | 'Tasks' | 'Calendar' | 'Notes' | 'Decisions'

interface TimelineFilters {
  Messages: boolean; Documents: boolean; Tasks: boolean;
  Calendar: boolean; Notes: boolean; Decisions: boolean
}
```

#### API

`fetchCommunicationMessages(limit = 500)` выполняет `GET /api/v1/communications/messages?limit=...` и возвращает `{ items: TimelineMessage[] }`.

#### Запрос (query)

`useTimelineMessagesQuery()` – использует ключ `['timeline-messages']`, вызывает `fetchCommunicationMessages(500)`, `refetchOnMount: 'always'`, `staleTime: 30_000`.

#### Хранилище (store)

`useTimelineStore` (id `timeline-ui`, Pinia):

- **state**: `messages`, `error`, `isLoading`, `filters` (все фильтры по умолчанию `true`).
- **getters**: `filteredMessages` – в текущей реализации возвращает все сообщения без фильтрации (заглушка, см. комментарий в коде: «Filter is a placeholder — in the Svelte original, filter state exists but all items pass through. Keep the structure for AC4 compliance.»).
- **actions**: `setMessages`, `setLoading`, `setError`, `toggleFilter(kind)`.

---

## Сгенерированный Protobuf-код (gen)

### common/v1

Файл: `frontend/src/gen/makosh/common/v1/common_pb.ts`

```ts
type PageRequest = { limit: number; cursor: string }
type PageResponse = { nextCursor: string; hasMore: boolean }
```

### communications/v1

Файл: `frontend/src/gen/makosh/communications/v1/communications_pb.ts` (обрезан после 12 000 символов).

Основные видимые сообщения и RPC:

- **CommunicationMessage** (много полей: `message_id`, `raw_record_id`, `observation_id`, `account_id`, `provider_record_id`, `subject`, `sender`, `recipients`, `body_text`, `occurred_at`, `projected_at`, `channel_kind`, `conversation_id`, `sender_display_name`, `delivery_state`, `workflow_state`, `importance_score`, `ai_category`, `ai_summary`, `local_state`, `attachment_count` и др.).
- **CommunicationMessageAttachment** (с полями метаданных, сканирования, хранилища).
- **ListMessagesRequest/Response** (с фильтрами: `account_id`, `workflow_state`, `channel_kind`, `conversation_id`, `query`, `match_mode`, `local_state`, `cursor`, `limit`; ответ `items` + `next_cursor`, `has_more`).
- **GetMessageRequest/Response** (возвращает элемент и `attachments`).
- **TransitionMessageWorkflowState** (message_id, workflow_state; ответ с `previous_state`).
- **UpdateMessageLocalState** (message_id; ответ с `local_state` и флагом `provider_deleted`).
- **MarkMessageRead**, **BulkMessageAction** (action, message_ids, опциональные `label`, `snooze_until`), **ToggleMessagePin**, **ToggleMessageImportant**, **ToggleMessageMute**, **SnoozeMessage**.
- **UpdateMessageLabel** (message_id, label) и ответ Add/Remove.
- **MessageSummaryContract** (key_points, action_items, risks, deadlines, event_candidates, persona_candidates, organization_candidates, document_candidates, agreement_candidates – каждая как `MessageKnowledgeCandidate` с `title` и `evidence`).
- **AnalyzeMessageRequest/Response** (анализ: категория, суммари, контракт, `importance_score`, `workflow_state`, `source`, `confidence`, `evidence`).
- **WorkflowAction** (source, input, request, target, provenance, response) – для запуска workflow-действий с командой и источником.
- **ExplainMessage**, **GetMessageSmartCc**, **GetMessageExport** (формат, content_type, content, filename).
- **MessageAuthResult/Report/RiskReport**, **GetMessageAuth**, **GetMessageSignature** – проверка подлинности писем (SPF, DKIM, DMARC).
- **AiReplyRequest/Response/Variants** – генерация AI-ответов с тоном, языком, контекстом.
- **WorkflowStateCount**, **ListMessageWorkflowStateCounts**.
- **SubscriptionSource**, **ListSubscriptionsRequest/Response** (источники рассылок, количество сообщений, first_seen, last_seen, is_newsletter, has_unsubscribe).
- **MailboxHealth** (total_messages, unread, needs_action, waiting, done…).

Файл обрезан – полный список RPC и сообщений не подтверждён.

### events/v1

Файл: `frontend/src/gen/makosh/events/v1/event_envelope_pb.ts`

```ts
type EventEnvelope = {
  eventId: string; eventType: string; schemaVersion: number;
  occurredAt?: Timestamp; recordedAt?: Timestamp;
  source?: JsonObject; actor?: JsonObject; subject?: JsonObject;
  payload?: JsonObject; provenance?: JsonObject;
  causationId: string; correlationId: string
}
```

### signal_hub/v1

Файл: `frontend/src/gen/makosh/signal_hub/v1/signal_hub_pb.ts` (обрезан после 12 000 символов).

Видимые модели и RPC:

- **SignalSource** (id, code, display_name, category, source_kind, флаги supports_connections/runtime/replay/pause/mute, `capability_schema_version`).
- **SignalCapability** (id, source_code, connection_id?, capability, state, reason?, requires_confirmation, action_class).
- **SignalFixtureSource** (fixture_id, source_code, event_type, correlation_id?, occurred_at, summary).
- **SignalConnection** (id, source_code, display_name, status, profile?, secret_ref?, connected_at?, last_seen_at?, last_signal_at?, last_sync_at?, settings_json).
- **SignalHealth** (id, source_code, connection_id?, level, summary, last_ok_at?, last_failure_at?, failure_count, consecutive_failure_count, next_retry_at?, evidence_json).
- **SignalRuntimeState** (id, source_code, connection_id?, runtime_kind, state, metadata_json, last_started_at?, last_stopped_at?, last_heartbeat_at?, last_error_at?, last_error_code?, last_error_message_redacted?).
- **SignalPolicy** (scope, source_code?, connection_id?, event_pattern?, mode, reason, expires_at?).
- **SignalProfile** (id, code, display_name, description, policy_count, is_system, is_active, source_policies – массив `SignalProfilePolicy`).
- **SignalReplayRequest** (множество параметров для повторного воспроизведения сигналов).

RPC: `ListSources`, `GetSource`, `ListCapabilities`, `ListFixtureSources`, `ListConnections`, `CreateConnection`, `UpdateConnection`, `RemoveConnection`, `ListHealth`, `RunHealthCheck`, `ListRuntimeStates`, `UpdateRuntimeState`, `ListPolicies`, `CreatePolicy`, `EnableSource`, `DisableSource`, `DisableSignals`, `EnableSignals`, `MuteSignals`, `UnmuteSignals`, `PauseSignals`, `ResumeSignals`, `ListProfiles`, `CreateProfile`, `UpdateProfile`, `RemoveProfile`, `ApplyProfile`, `ListReplayRequests`, `RequestReplay` (и другие, файл обрезан).

---

## Интеграции

### Mail (Почта)

Модуль: `frontend/src/integrations/mail/`

#### API

- **accountSetup.ts**
  - `startGmailOAuthSetup(request: GmailOAuthStartRequest): Promise<GmailOAuthStartResponse>` → `POST /api/v1/integrations/mail/accounts/gmail/oauth/start`
  - `setupImapEmailAccount(request: ImapEmailAccountSetupRequest): Promise<EmailAccountSetupResponse>` → `POST /api/v1/integrations/mail/accounts/imap`
  - Типы запросов/ответов: `GmailOAuthStartRequest` (account_id, display_name, external_account_id?, redirect_uri, app_return_url?, scopes?), `GmailOAuthStartResponse` (setup_id, authorization_url, state, redirect_uri), `ImapEmailAccountSetupRequest` (поля хоста, порта, tls, mailbox, smtp-настроек, пароль с secret_kind = `app_password` | `password`), `EmailAccountSetupResponse` (account_id, secret_ref, secret_kind, store_kind).
- **syncApi.ts**
  - `fetchMailSyncStatus()` → `GET /api/v1/integrations/mail/accounts/sync-status`
  - `fetchMailSyncSettings(accountId)` → `GET .../accounts/{id}/sync-settings`
  - `updateMailSyncSettings(accountId, settings)` → `PUT .../accounts/{id}/sync-settings`
  - `runMailSyncNow(accountId)` → `POST .../accounts/{id}/sync-now`
  - `runMailFullResync(accountId)` → `POST .../accounts/{id}/sync-full-resync`

#### Формы

- **accountSetupForm.ts**:
  - Провайдеры: `gmail`, `icloud`, `imap`.
  - Схема Zod: поля провайдера, email (trim + lowercase), пароль, IMAP/SMTP настройки. Кастомная валидация через `superRefine`: для не-Gmail обязательны пароль и IMAP-хост.
  - `accountSetupFormDefaults(provider)` – значения по умолчанию (для iCloud `imap.mail.me.com:993`, TLS).
  - `accountSetupDefaultAccountId(provider, email)` – генерирует стабильный ID вида `mail-{provider}-{slug}`.
  - `accountSetupFormToImapRequest(values)` – преобразует значения формы в `ImapEmailAccountSetupRequest` (выбрасывает ошибку для Gmail).
  - `accountSetupFormToGmailOAuthStart(values, apiBaseUrl)` – преобразует в Gmail OAuth start, формируя `redirect_uri` как `{apiBaseUrl}/api/v1/integrations/mail/accounts/gmail/oauth/callback`.
- **syncSettingsForm.ts**:
  - Поля: `sync_enabled` (boolean), `batch_size` (1–500), `poll_interval_seconds` (60–86400).
  - `syncSettingsFormDefaults(settings)` – значения по умолчанию (sync_enabled=true, batch=100, interval=300).
  - `syncSettingsFormToUpdate(values)` → `MailSyncSettingsUpdate`.

#### Запросы (queries)

- `useStartGmailOAuthSetupMutation()` – `useMutation` для старта OAuth.
- `useSetupImapEmailAccountMutation()` – `useMutation` для создания IMAP-аккаунта.
- `runtimeQueries.ts` реэкспортирует общие запросы синхронизации из `shared/mailSync/runtimeQueries`.

#### Компоненты и граничные тесты

- Тест `AccountSetupModal.boundary.test.ts` проверяет, что компонент использует vee-validate, формы и мутации (`useSetupImapEmailAccountMutation`, `useStartGmailOAuthSetupMutation`, `mutateAsync`) и не содержит `setTimeout`.
- Тест `MailSyncSettingsStrip.boundary.test.ts` проверяет, что компонент содержит элементы управления синхронизацией (`sync_enabled`, `batch_size`, `poll_interval_seconds`, «Provider sync», «Save»), использует emits, но не имеет прямого доступа к API (`ApiClient` или `fetch`).

### Telegram

Модуль: `frontend/src/integrations/telegram/`

#### API (`telegram.ts` – обрезан)

Основные группы маршрутов:

- **Capabilities**: `GET .../capabilities` и `GET .../accounts/{id}/capabilities`.
- **Accounts**: получение списка (`GET .../accounts`), создание (`POST .../accounts`), удаление (`DELETE .../accounts/{id}`), логаут (`POST .../logout`).
- **Folders**: `GET .../conversation-folders` (опциональный `account_id`).
- **Chats**: синхронизация членов (`POST .../provider-sync/conversations/{id}/members`), синхронизация чатов (`POST .../provider-sync/chats`).
- **Диалоговые действия**: pin/unpin, archive/unarchive, mute/unmute, mark read/unread, join/leave, add/remove to/from folder, reassign folders – все через `POST .../provider-commands/conversations/{id}/...`.
- **History**: `POST .../provider-sync/history`.
- **Runtime**: статус (`GET .../runtime/status`), start/stop/restart (`POST .../runtime/start|stop|restart`).
- **Media**: download (`POST .../provider-media/download`).
- **Automation**: dry-run отправки (`POST .../policies/telegram-send/dry-run`).
- **Fixtures**: приём тестового сообщения (`POST .../fixtures/messages`).
- **QR-логин** и **парольный вход** (упомянуты в импортируемых типах, код не входит в лимит).

#### Automation API (`telegramAutomation.ts`)

- `fetchTelegramAutomationPolicies()` → `GET /api/v1/policies`
- `fetchTelegramAutomationTemplates()` → `GET /api/v1/policies/templates`
- `runTelegramSendDryRun(request)` → `POST /api/v1/policies/telegram-send/dry-run`

#### Lifecycle API (`telegramLifecycle.ts`)

- `fetchTelegramCommands(accountId, limit, options?)` → `GET /api/v1/integrations/telegram/commands` с параметрами `account_id`, `limit`, `provider_chat_id`, `provider_message_id`, `command_kinds`.
- `retryTelegramCommand(commandId)` → `POST .../commands/{id}/retry`.

#### Тесты

- `telegramAutomation.test.ts`: проверяет загрузку политик, шаблонов и dry-run.
- `telegramDialogs.test.ts` (обрезан): проверяет вызовы runtime restart/stop, синхронизацию членов, загрузку папок, capabilities, звонков и стенограмм, создание/удаление/логаут аккаунтов, pin/unpin, archive/mute, add/remove folder.
- `telegramLifecycle.test.ts`: проверяет получение команд с фильтрами и повтор команды.

---

## Общие зависимости

- **API Client** (`platform/api/ApiClient`): инициализируется с `baseUrl` и `secret`, используется во всех API-функциях.
- **Состояние**: Pinia (`defineStore`).
- **Запросы к серверу**: TanStack Vue Query (`useQuery`, `useMutation`).
- **Валидация**: Zod + vee-validate (`toTypedSchema`).
- **Тестирование**: Vitest с моком `global.fetch`.

> Детали маршрутов, имён полей и логики, не подтверждённые приложенными исходниками, не включены.
```

---

## Покрытие источников

Приведённые исходные файлы покрывают следующие факты:

- `frontend/src/domains/tasks/types/task.ts` — все интерфейсы и типы для TaskCandidate, Task, Decision, Obligation и связанных с ними структур запросов/ответов.
- `frontend/src/domains/timeline/api/timeline.ts` — функция `fetchCommunicationMessages`, её параметр `limit`, URL `GET /api/v1/communications/messages`.
- `frontend/src/domains/timeline/queries/useTimelineQuery.ts` — query `useTimelineMessagesQuery`, ключ `['timeline-messages']`, вызов `fetchCommunicationMessages(500)`, `refetchOnMount: 'always'`, `staleTime: 30_000`.
- `frontend/src/domains/timeline/stores/timeline.ts` — хранилище Pinia `timeline-ui`, поля `messages`, `error`, `isLoading`, `filters`, геттер `filteredMessages` (без фактической фильтрации), actions `setMessages`, `setLoading`, `setError`, `toggleFilter`.
- `frontend/src/domains/timeline/types/timeline.ts` — интерфейсы `TimelineMessage`, `TimelineFilters` и тип `TimelineFilterKind`.
- `frontend/src/gen/makosh/common/v1/common_pb.ts` — сообщения `PageRequest`, `PageResponse`.
- `frontend/src/gen/makosh/communications/v1/communications_pb.ts` — сообщения `CommunicationMessage`, `CommunicationMessageAttachment`, RPC для списков, получения, перехода workflow, обновления локального состояния, пометки прочитанным, массовых операций, toggle, snooze, анализа, объяснения, экспорта, AI-ответов, аутентификации писем, подписок, состояния почтового ящика; файл обрезан, подтверждена только начальная часть.
- `frontend/src/gen/makosh/events/v1/event_envelope_pb.ts` — сообщение `EventEnvelope`.
- `frontend/src/gen/makosh/signal_hub/v1/signal_hub_pb.ts` — модели `SignalSource`, `SignalCapability`, `SignalFixtureSource`, `SignalConnection`, `SignalHealth`, `SignalRuntimeState`, `SignalPolicy`, `SignalProfile`, `SignalReplayRequest` и соответствующие RPC; файл обрезан.
- `frontend/src/integrations/mail/api/accountSetup.test.ts` — тесты для `startGmailOAuthSetup` и `setupImapEmailAccount` с проверкой URL, метода, заголовка `X-Макошь-Secret` и тела запроса.
- `frontend/src/integrations/mail/api/accountSetup.ts` — функции `startGmailOAuthSetup` и `setupImapEmailAccount`, типы запросов и ответов.
- `frontend/src/integrations/mail/api/syncApi.ts` — функции `fetchMailSyncStatus`, `fetchMailSyncSettings`, `updateMailSyncSettings`, `runMailSyncNow`, `runMailFullResync`.
- `frontend/src/integrations/mail/components/AccountSetupModal.boundary.test.ts` — граничные проверки на использование vee-validate, форм, мутаций и отсутствие `setTimeout`.
- `frontend/src/integrations/mail/components/MailSyncSettingsStrip.boundary.test.ts` — граничные проверки на наличие полей управления синхронизацией, emits, отсутствие прямого доступа к API.
- `frontend/src/integrations/mail/forms/accountSetupForm.test.ts` — тесты нормализации iCloud/IMAP, отклонения отсутствующих полей, построения Gmail OAuth, генерации ID.
- `frontend/src/integrations/mail/forms/accountSetupForm.ts` — схема Zod, defaults, `accountSetupDefaultAccountId`, `accountSetupFormToImapRequest`, `accountSetupFormToGmailOAuthStart`.
- `frontend/src/integrations/mail/forms/syncSettingsForm.ts` — схема Zod, defaults, `syncSettingsFormToUpdate`.
- `frontend/src/integrations/mail/queries/accountSetupQueries.ts` — мутации `useStartGmailOAuthSetupMutation`, `useSetupImapEmailAccountMutation`.
- `frontend/src/integrations/mail/queries/runtimeQueries.ts` — реэкспорт общих запросов синхронизации.
- `frontend/src/integrations/telegram/api/telegram.ts` — широкий набор API-функций (capabilities, accounts, chats, runtime, media, automation, fixtures); файл обрезан.
- `frontend/src/integrations/telegram/api/telegramAutomation.test.ts` — тест для политик, шаблонов и dry-run.
- `frontend/src/integrations/telegram/api/telegramAutomation.ts` — функции `fetchTelegramAutomationPolicies`, `fetchTelegramAutomationTemplates`, `runTelegramSendDryRun`.
- `frontend/src/integrations/telegram/api/telegramDialogs.test.ts` — тесты runtime операций, синхронизации, папок, capabilities, звонков, аккаунтов, диалоговых действий; файл обрезан.
- `frontend/src/integrations/telegram/api/telegramLifecycle.test.ts` — тест получения команд и retry.
- `frontend/src/integrations/telegram/api/telegramLifecycle.ts` — функции `fetchTelegramCommands`, `retryTelegramCommand`.

---

## Исходные файлы

- [`frontend/src/domains/tasks/types/task.ts`](../../../../frontend/src/domains/tasks/types/task.ts)
- [`frontend/src/domains/timeline/api/timeline.ts`](../../../../frontend/src/domains/timeline/api/timeline.ts)
- [`frontend/src/domains/timeline/queries/useTimelineQuery.ts`](../../../../frontend/src/domains/timeline/queries/useTimelineQuery.ts)
- [`frontend/src/domains/timeline/stores/timeline.ts`](../../../../frontend/src/domains/timeline/stores/timeline.ts)
- [`frontend/src/domains/timeline/types/timeline.ts`](../../../../frontend/src/domains/timeline/types/timeline.ts)
- [`frontend/src/gen/makosh/common/v1/common_pb.ts`](../../../../frontend/src/gen/makosh/common/v1/common_pb.ts)
- [`frontend/src/gen/makosh/communications/v1/communications_pb.ts`](../../../../frontend/src/gen/makosh/communications/v1/communications_pb.ts)
- [`frontend/src/gen/makosh/events/v1/event_envelope_pb.ts`](../../../../frontend/src/gen/makosh/events/v1/event_envelope_pb.ts)
- [`frontend/src/gen/makosh/signal_hub/v1/signal_hub_pb.ts`](../../../../frontend/src/gen/makosh/signal_hub/v1/signal_hub_pb.ts)
- [`frontend/src/integrations/mail/api/accountSetup.test.ts`](../../../../frontend/src/integrations/mail/api/accountSetup.test.ts)
- [`frontend/src/integrations/mail/api/accountSetup.ts`](../../../../frontend/src/integrations/mail/api/accountSetup.ts)
- [`frontend/src/integrations/mail/api/syncApi.ts`](../../../../frontend/src/integrations/mail/api/syncApi.ts)
- [`frontend/src/integrations/mail/components/AccountSetupModal.boundary.test.ts`](../../../../frontend/src/integrations/mail/components/AccountSetupModal.boundary.test.ts)
- [`frontend/src/integrations/mail/components/MailSyncSettingsStrip.boundary.test.ts`](../../../../frontend/src/integrations/mail/components/MailSyncSettingsStrip.boundary.test.ts)
- [`frontend/src/integrations/mail/forms/accountSetupForm.test.ts`](../../../../frontend/src/integrations/mail/forms/accountSetupForm.test.ts)
- [`frontend/src/integrations/mail/forms/accountSetupForm.ts`](../../../../frontend/src/integrations/mail/forms/accountSetupForm.ts)
- [`frontend/src/integrations/mail/forms/syncSettingsForm.ts`](../../../../frontend/src/integrations/mail/forms/syncSettingsForm.ts)
- [`frontend/src/integrations/mail/queries/accountSetupQueries.ts`](../../../../frontend/src/integrations/mail/queries/accountSetupQueries.ts)
- [`frontend/src/integrations/mail/queries/runtimeQueries.ts`](../../../../frontend/src/integrations/mail/queries/runtimeQueries.ts)
- [`frontend/src/integrations/telegram/api/telegram.ts`](../../../../frontend/src/integrations/telegram/api/telegram.ts)
- [`frontend/src/integrations/telegram/api/telegramAutomation.test.ts`](../../../../frontend/src/integrations/telegram/api/telegramAutomation.test.ts)
- [`frontend/src/integrations/telegram/api/telegramAutomation.ts`](../../../../frontend/src/integrations/telegram/api/telegramAutomation.ts)
- [`frontend/src/integrations/telegram/api/telegramDialogs.test.ts`](../../../../frontend/src/integrations/telegram/api/telegramDialogs.test.ts)
- [`frontend/src/integrations/telegram/api/telegramLifecycle.test.ts`](../../../../frontend/src/integrations/telegram/api/telegramLifecycle.test.ts)
- [`frontend/src/integrations/telegram/api/telegramLifecycle.ts`](../../../../frontend/src/integrations/telegram/api/telegramLifecycle.ts)

## Кандидаты на drift

Из приложенного контекста видно следующее потенциальное расхождение:

- **Timeline‑фильтр является заглушкой**: в `frontend/src/domains/timeline/stores/timeline.ts` геттер `filteredMessages` возвращает все сообщения без учёта `filters`. Комментарий в коде поясняет: «Filter is a placeholder — in the Svelte original, filter state exists but all items pass through. Keep the structure for AC4 compliance.» Это может быть расхождением между ожидаемым поведением (фильтрация по типам `Messages`, `Documents`, …) и текущей реализацией. Следует считать это известной особенностью, требующей доработки.
- **Сгенерированные protobuf‑файлы могут не соответствовать актуальным `.proto`**: файлы `gen/...` автоматически сгенерированы protoc-gen-es v2.12.0. Без доступа к исходным `.proto`-определениям невозможно подтвердить их актуальность либо дрейф. В рамках данного чанка эта проверка не выполнена.

Никаких других расхождений между кодом, документацией или ADR из представленных исходных файлов не выявлено.
