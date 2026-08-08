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

- Chunk ID / ID чанка: `115-doc-docs-part-006`
- Group / Группа: `docs`
- Role / Роль: `doc`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/documentation-map.md`

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

### `docs/integrations/telegram/api/media-search.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/api/media-search.md`
- Size bytes / Размер в байтах: `2143`
- Included characters / Включено символов: `2141`
- Truncated / Обрезано: `no`

````markdown
# Telegram API Reference — Media and Search

See also: [API Index](../api.md), [Conversations](conversations.md), and [Operations and Realtime](operations-realtime.md).

## Ownership

Telegram provider runtime and debug surfaces live under:

```text
/api/v1/integrations/telegram/*
```

Shared Communication attachment and business read-model APIs live under:

```text
/api/v1/communications/*
```

## Media Routes

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/integrations/telegram/provider-media/download` | Download TDLib media and persist local attachment/blob state |
| POST | `/api/v1/integrations/telegram/provider-media/upload` | Queue provider-side media send from a local attachment or blob |
| GET | `/api/v1/communications/attachments/{attachment_id}/preview` | Shared safe preview endpoint |
| POST | `/api/v1/communications/attachments/import` | Shared local attachment import |
| GET | `/api/v1/communications/attachments/search` | Shared attachment search |
| GET | `/api/v1/communications/search/media` | Макошь projected media search/filter |

## Search Routes

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/integrations/telegram/provider-search` | Provider raw search/refresh trigger; returns status metadata only |
| GET | `/api/v1/communications/conversations/search` | Макошь projected conversation search |
| GET | `/api/v1/communications/search/messages` | Макошь projected message search |
| GET | `/api/v1/communications/search/media` | Макошь projected media search |
| GET | `/api/v1/communications/search` | Макошь email/full-text business search over the projected read-model |
| GET | `/api/v1/communications/saved-searches` | Shared saved-search surface |

## Notes

- Provider media upload does not accept raw file bytes; it works from already imported local attachments/blobs.
- Shared Communication attachment APIs remain the canonical safe access path for local preview and import.
- Normal user-facing Communication search uses `/api/v1/communications/search/*`; provider search is runtime/debug/sync-assist only and must not return projected message or media items.
````

### `docs/integrations/telegram/api/operations-realtime.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/api/operations-realtime.md`
- Size bytes / Размер в байтах: `13622`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Telegram API Reference — Operations and Realtime

См. также: [API Index](../api.md) и [Media and Search](media-search.md).

## Automation / Policies

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/policies/templates` | List automation templates |
| POST | `/api/v1/policies/templates` | Create/update automation template |
| GET | `/api/v1/policies` | List automation policies |
| POST | `/api/v1/policies` | Create/update automation policy |
| POST | `/api/v1/policies/telegram-send/dry-run` | Telegram send policy/template validation and sanitized audit |

Automation live send is blocked. Dry-run stores preview hashes and redacted metadata.
Telegram composer now also surfaces this route as a read-only preflight panel in
the send dropdown: it loads policies/templates, filters enabled policies for the
selected account/chat, accepts template variables and renders the returned
`rendered_text` plus `rendered_preview_hash` before any live send.

## Audit

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/audit/events?target_id=&actor_id=&after_audit_id=&limit=` | Shared protected audit-event reader |

Telegram audit records must never include:

- message bodies;
- rendered variables;
- media bytes;
- passwords;
- tokens;
- app secrets;
- raw TDLib payloads.

## Calls

### Текущие маршруты

| Method | Path | Описание |
|---|---|---|
| GET | `/api/v1/calls?account_id=&limit=` | List Telegram call metadata rows |
| POST | `/api/v1/calls` | Create/update fixture Telegram call metadata |
| GET | `/api/v1/calls/{call_id}/transcript` | Get latest stored transcript; inspector context now surfaces this through a first-class calls panel with local metadata search |
| POST | `/api/v1/calls/{call_id}/transcript` | Create fixture transcript through fixture STT provider |

Telegram workbench now consumes `GET /api/v1/calls?account_id=&limit=` through
a first-class inspector calls panel for the selected account, and that same
panel can locally search projected call metadata before loading
`GET /api/v1/calls/{call_id}/transcript` as read-only transcript evidence for
the selected projected call. Live call controls and recording flows remain
outside the current UI slice.

### Заблокированные live возможности

- live call control;
- audio capture;
- device selection;
- real STT;
- hidden recording.

## Realtime / Events

### Текущие generic routes

| Method | Path | Описание |
|---|---|---|
| GET | `/api/events/ws?after_position=&makosh_secret=` | Protected WebSocket event stream with replay/heartbeat |
| GET | `/api/events/stream?after_position=` | Protected SSE stream |
| GET | `/api/v1/events?after_position=&limit=&wait_seconds=` | Protected JSON replay/long-poll fallback |
| POST | `/api/v1/events` | Local event API command boundary |
| GET | `/api/v1/events/{event_id}` | Read single event |

When the shared backend event bus overruns a WebSocket subscriber, `/api/events/ws`
may emit a transport-level frame:

```json
{"type":"lagged","data":{"skipped":7}}
```

Frontend consumers must treat this as a replay-gap signal, invalidate
Telegram/communication caches broadly, and offer reconnect/replay recovery from
the last persisted cursor.

### Текущие Telegram realtime contracts

```text
telegram.sync.started
telegram.sync.progress
telegram.sync.completed

telegram.message.created
telegram.message.updated
telegram.message.deleted
telegram.message.visibility_restored

telegram.chat.pinned
telegram.chat.archived
telegram.chat.muted
telegram.chat.updated
telegram.typing.changed
telegram.topic.updated
telegram.participant.updated
telegram.media.download.started
telegram.media.download.progress
telegram.media.download.failed
telegram.media.downloaded
telegram.media.upload.started
telegram.media.upload.progress
telegram.media.upload.failed
telegram.media.upload.completed

telegram.reaction.changed
telegram.command.status_changed
telegram.command.reconciled
```

Frontend cache patching still accepts the legacy `telegram.message.edited`
event name for compatibility with older event-log rows or stale clients, but
new backend emissions use `telegram.message.updated`.

Telegram UI consumers should use the shared frontend realtime bootstrap backed
by `/api/events/ws`, `/api/events/stream` and `/api/v1/events`; the Telegram
workbench no longer opens a dedicated page-local realtime socket.

Current emission scope:

- fixture ingest and manual send -> `telegram.message.created`;
- lifecycle edit/delete/restore -> corresponding `telegram.message.*`;
- TDLib `updateNewMessage` now feeds provider-origin
  `telegram.message.created` through the actor event bridge after ingesting
  the observed message snapshot into the Communication projection;
- TDLib `updateMessageContent` now feeds provider-origin
  `telegram.message.updated` after refreshing projected message text/metadata,
  recording an observed edit version and reconciling matching edit commands;
- TDLib `updateMessageEdited` now also feeds provider-origin
  `telegram.message.updated` after refreshing projected edit metadata such as
  `provider_edit_timestamp` and reply-markup evidence;
- TDLib `updateMessageIsPinned` now also feeds provider-origin
  `telegram.message.updated` after refreshing projected `is_pinned` metadata
  and reconciling matching message-targeted `pin` commands;
- chat/history sync -> `telegram.sync.started/progress/completed/failed`;
- local dialog pin/archive/mute toggles -> `telegram.chat.pinned/archived/muted`;
- local dialog read/unread toggles -> `telegram.chat.updated`;
- TDLib `updateChatReadInbox` and `updateChatUnreadMentionCount` parsing now
  feed provider-observed unread/mention counter updates into
  `telegram.chat.updated` after refreshing projected chat metadata;
- TDLib `updateChatIsMarkedAsUnread` parsing now feeds provider-observed
  unread-flag updates into `telegram.chat.updated` after refreshing projected
  chat metadata and reconciling matching read/unread commands;
- TDLib `updateChatNotificationSettings` parsing now feeds provider-observed
  mute-state updates into both `telegram.chat.updated` and
  `telegram.chat.muted` after refreshing projected chat metadata and
  reconciling matching exact-shape mute/unmute commands;
- TDLib `updateChatPosition` parsing now feeds provider-observed archive/main
  list-state updates into `telegram.chat.updated`,
  `telegram.chat.archived` and `telegram.chat.pinned` after refreshing
  projected chat metadata and reconciling matching archive/unarchive plus
  pin/unpin commands;
- TDLib `updateDeleteMessages` now feeds provider-origin
  `telegram.message.deleted` through the actor event bridge after recording an
  idempotent `deleted_by_provider` tombstone and reconciling matching delete
  commands;
- TDLib `updateUserChatAction` parsing now feeds `telegram.typing.changed`
  from runtime-started TDLib actors through an actor event bridge into the
  canonical event log and realtime bus;
- TDLib `updateForumTopicInfo` parsing feeds `telegram.topic.updated` through
  the same actor event bridge after resolving the projected chat and upserting
  the topic projection;
- TDLib `updateNewMessage` now feeds provider-origin
  `telegram.message.created` through the actor event bridge after ingesting
  the observed message snapshot into the Communication projection;
- TDLib `updateDeleteMessages` now feeds provider-origin
  `telegram.message.deleted` through the actor event bridge after recording an
  idempotent `deleted_by_provider` tombstone and reconciling matching delete
  commands;
- TDLib `updateMessageInteractionInfo` now feeds provider-origin
  `telegram.reaction.changed` through the actor event bridge after refreshing
  reaction summary metadata, syncing sender-level provider rows and
  reconciling matching self reaction commands;
- local reaction add/remove -> `telegram.reaction.changed`;
- media download request/failure/non-completed TDLib snapshot -> `telegram.media.download.started/progress/failed`;
- completed TDLib media download persistence -> `telegram.media.downloaded`;
- manual send, lifecycle edit/delete/restore, reaction add/remove and current dialog actions (`pin/unpin/archive/unarchive/mute/unmute/read/unread`) -> `telegram.command.status_changed`.
- provider-observed command completion -> `telegram.command.reconciled`.

Current message-event payload shape is richer than the initial coarse contract:

- top-level lifecycle fields such as `provider_chat_id`, `provider_message_id`,
  `version_number`, `reason_class`, `tombstone_id`, `is_pinned`;
- sanitized projected `message` snapshot for created/updated/deleted/visibility
  restore events;
- sanitized projected `chat` snapshot for message events when a corresponding
  `telegram_chats` projection row exists;
- deterministic `telegram_chat_id` derived from projection row or
  `account_id + provider_chat_id`, so frontend pinned-message caches can patch
  even when a dedicated `telegram_chats` row has not been materialized yet.

Current frontend realtime runtime-state usage also relies on:

- `event.metadata.account_id` for account-scoped runtime query matching;
- `telegram.sync.*` payload fields such as `scope`, `status`, `synced_count`,
  `has_more`, `provider_chat_id`;
- `telegram.command.status_changed` / `telegram.command.reconciled` payload
  fields such as `command_id`, `status`, `provider_chat_id`,
  `telegram_chat_id`, `message_id`, `reconciliation_status`,
  `provider_observed_at`, `reconciled_at` and `next_attempt_at`.

Current local dialog command payloads are also richer than the original coarse
command-status contract:

- `action` for the local dialog operation kind;
- sanitized projected `chat` snapshot for current local dialog actions, so
  frontend chat list/detail caches can patch immediately after pin/archive/
  mute/read/unread changes.

Current local dialog flag events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- one changed flag: `is_pinned`, `is_archived` or `is_muted`;
- sanitized projected `chat` snapshot.

Current local read/unread chat update events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `action`: `mark_read` or `mark_unread`;
- optional `message_id` on the paired command-status event for targeted
  mark-read requests;
- sanitized projected `chat` snapshot after unread counter recomputation.

Current provider-observed unread chat update events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `action`: `provider_unread_update`;
- optional `unread_count`;
- optional `unread_mention_count`;
- optional `last_read_inbox_provider_message_id`;
- sanitized projected `chat` snapshot after provider counter projection.

Current provider-observed unread-flag chat update events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `action`: `provider_marked_as_unread_update`;
- `is_marked_as_unread`;
- sanitized projected `chat` snapshot after provider flag projection.

Current provider-observed mute chat events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `is_muted`;
- `use_default_mute_for`;
- `mute_for`;
- sanitized projected `chat` snapshot after provider notification projection.

Current provider-observed archive chat events carry:

- `provider_chat_id`;
- `telegram_chat_id`;
- `is_archived`;
- `list_kind`;
- optional `provider_folder_id`;
- `order`;
- `is_pinned`;
- sanitized projected `chat` snapshot after provider position projection.

Current media download lifecycle events carry:

- `provider_chat_id`;
- `provider_message_id`;
- `tdlib_file_id`;
- `provider_attachment_id`;
- `download_state`;
- progress fields (`expected_size_bytes`, `downloaded_size_bytes`,
  `is_downloading_active`, `is_downloading_completed`) when TDLib returns a
  non-completed file snapshot;
- sanitized projected `message` and `chat` snapshots only on completed
  `telegram.media.downloaded` events after local blob/attachment persistence.

When the database-backed event log is configured, these same typed events are
also append
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/telegram/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/architecture.md`
- Size bytes / Размер в байтах: `18636`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Telegram Architecture

Статус: архитектурная ревизия и целевая спецификация на 2026-06-17.

## Позиция

Telegram принадлежит Communications Domain как **channel/source boundary**.
Он не является отдельным продуктом, не владеет памятью, знаниями, задачами,
обязательствами, решениями, проектами, организациями или персонами.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

Telegram должен поставлять Макошь:

- raw provider evidence;
- provider-specific metadata;
- messages/media/calls source records;
- provider commands;
- realtime updates;
- identity traces;
- integration panels inside the `/communications` workspace.

## Canonical Flow

```text
Telegram Provider
  -> Raw Records
  -> Communication Projection
  -> Events
  -> Timeline
  -> Shared Engines
```

Текущий backend flow для сообщений:

```text
Fixture / TDLib snapshot
  -> communication_raw_records
  -> project_raw_telegram_message
  -> communication_messages
  -> candidate refresh / shared engine integration points
```

Целевой flow:

```text
Telegram runtime event
  -> observation.captured.v1
  -> signal.raw.telegram.message.observed
  -> signal.accepted.telegram.message
  -> communication.message.recorded / communication.message.updated
  -> Timeline evidence / trace link
  -> Search / Risk / Enrichment / AI candidates
  -> UI cache patch + replay
```

## Trace Contract

Telegram runtime events must not create product domain state directly.

Telegram provider observations enter the canonical trace as:

```text
observation.captured.v1
  -> signal.raw.telegram.message.observed
  -> signal.accepted.telegram.message
  -> communication.message.recorded / communication.message.updated
```

Telegram integration code owns provider protocol and runtime state. Signal Hub
owns source acceptance policy. Communications owns canonical message state.
Trace reconstruction belongs to `platform/events`.

## Key ADR

| ADR | Значение для Telegram |
|---|---|
| ADR-0001 | Event sourcing is system spine |
| ADR-0018 | Provider adapter boundary |
| ADR-0031 | Desktop-only UI scope |
| ADR-0046 | Blob storage and scanner boundary for attachment bytes |
| ADR-0050 | V4 Telegram policy automation and call intelligence |
| ADR-0052 | Capability/action confirmation policy |
| ADR-0056 | Router-level `X-Макошь-Secret` local API auth |
| ADR-0076 | Host vault for new secret payloads |
| ADR-0083 | Account-scoped TDLib runtime slice |
| ADR-0085 | Communication spine and Polygraph integration |
| ADR-0091 | Production Telegram capability model |
| ADR-0093 | Vue 3 frontend |
| ADR-0097 | Channels are integrations; Communications owns domain state |

## Backend Layers

| Layer | Current files | Назначение |
|---|---|---|
| API routes | `backend/src/integrations/telegram/api/` | Account, capability, runtime, QR, chat, message and media endpoints |
| Runtime manager | `backend/src/integrations/telegram/runtime/` | Fixture and TDLib account actor orchestration |
| Client/store | `backend/src/integrations/telegram/client/` | Account metadata, chat projection, message ingestion, queries, attachment anchors |
| TDLib boundary | `backend/src/integrations/telegram/tdjson/` | JSON request builders, parsing, QR login, native TDLib loading; contract tests are split by environment, request builders, parsing snapshots and QR-login flows |
| Source records | `backend/src/domains/communications/core/` compatibility boundary | Raw provider records and provider accounts |
| Projection | `backend/src/domains/communications/messages/` compatibility boundary | Canonical `communication_messages` projection |
| Media storage | `backend/src/domains/communications/storage/` compatibility boundary | Local blob and attachment metadata/scanner boundary |
| Audit | `backend/src/platform/audit/telegram.rs` | Redacted provider-write, automation and lifecycle audit records |
| Calls | `backend/src/platform/calls/` | Telegram call metadata and fixture transcripts |

## Frontend Layers

| Layer | Current files | Назначение |
|---|---|---|
| Runtime/setup panels | `frontend/src/integrations/telegram/views/TelegramPage.vue` | Telegram integration panels embedded in Communications |
| API client | `frontend/src/integrations/telegram/api/telegram.ts` | Typed calls to protected backend routes |
| Query hooks | `frontend/src/integrations/telegram/queries/useTelegramQuery.ts` | TanStack Query integration |
| Store/helpers | `frontend/src/integrations/telegram/stores/telegram.ts` | Local UI state, filters, derived lists |
| Components | `frontend/src/integrations/telegram/components/` | Chat list, timeline, composer, action rail, inspector |

Realtime delivery on the frontend is shared with the rest of Макошь through
`frontend/src/platform/bootstrap/realtime.ts`; Telegram panels consume
communications-scoped query state and cache patches instead of opening a second
channel-scoped socket.

## Runtime Kinds

Текущие runtime modes:

```text
fixture
live_blocked
tdlib_qr_authorized
```

Целевой runtime map:

| Runtime | Назначение | Статус |
|---|---|---|
| Fixture runtime | Deterministic local/test validation | implemented |
| TDLib user runtime | QR-authorized live user account | implemented |
| Bot API runtime | Bot account runtime | planned |
| Offline command runtime | durable local command replay | implemented |
| Media capture runtime | voice/video/call capture boundary | planned |

## Account Boundary

Telegram account должен хранить:

- provider kind: `telegram_user`, `telegram_bot`;
- lifecycle state: active/logged_out/removed;
- non-secret config;
- secret references;
- runtime state;
- TDLib/session state refs;
- capability snapshot;
- audit links.

Секреты не должны попадать в account config, audit records, events или frontend state.

## Capability Boundary

Перед появлением операции в UI backend обязан вернуть capability state.

Состояния:

```text
available
degraded
blocked
unsupported
```

Каждая capability должна иметь:

- operation name;
- provider kind;
- runtime kind;
- action class;
- scope;
- reason;
- confirmation requirement;
- closure gate.

Action classes:

```text
read
write
destructive
admin
recording
export
secret-bearing
```

## Message Lifecycle

Текущий lifecycle:

```text
Fixture message or TDLib message snapshot
  -> validated NewTelegramMessage
  -> raw source record
  -> projected communication message
  -> recent message query / selected chat timeline
```

Текущие delivery states ограничены shared `communication_messages.delivery_state`:

```text
received
sent
send_dry_run
send_blocked
```

Целевой lifecycle:

```text
provider message created
  -> raw event
  -> projected message
  -> telegram.message.created

provider message edited
  -> raw edit event
  -> version row
  -> diff metadata
  -> telegram.message.updated

provider message pin state observed
  -> provider pin-state event
  -> projected message metadata refresh
  -> message pin command reconciliation
  -> telegram.message.updated

provider/local delete observed
  -> raw delete evidence
  -> tombstone row
  -> telegram.message.deleted

local restore visibility
  -> local visibility state
  -> telegram.message.visibility_restored
```

## Message Identity

Минимально необходимая identity model:

- account_id;
- provider_chat_id;
- provider_message_id;
- provider_sender_id;
- message_timestamp;
- raw_record_id;
- communication_message_id;
- optional topic_id;
- optional reply_to_message_id;
- optional forward_source;
- optional edit_version;
- optional tombstone state.

## Provider Command Outbox

Telegram provider writes must use the provider command path. UI components do
not call TDLib/Bot API upload or write primitives directly.

Current durable outbox foundation:

```text
API command route
  -> telegram_provider_write_commands
  -> atomic claim/lock
  -> runtime actor dispatch
  -> provider-observed state
  -> Communication projection refresh
  -> telegram.command.status_changed / telegram.command.reconciled
```

Command rows currently carry:

- status including `queued`, `executing`, `completed`, `failed`, `retrying`,
  `cancelled` and `dead_letter`;
- retry counters and due timestamps;
- execution lock owner/timestamp;
- provider-observed state and reconciliation status;
- result payload and redacted audit metadata.

Current provider-observed reconciliation coverage includes:

- send/reply/forward/media upload from returned TDLib message snapshots;
- edit from TDLib `updateMessageContent` when the observed provider body text
  matches the queued command payload;
- delete from TDLib `updateDeleteMessages` provider tombstone observation;
- self `join` from TDLib member-roster presence;
- self `join` / `leave` from explicit TDLib service-message evidence;
- self `react` / `unreact` from TDLib message `interaction_info.reactions`
  chosen emoji state, including unsolicited `updateMessageInteractionInfo`
  runtime updates;
- `mark_read` / `mark_unread` from TDLib `updateChatIsMarkedAsUnread`;
- `pin` / `unpin` from TDLib `updateChatPosition` main/archive list pin state;
- `archive` / `unarchive` from TDLib `updateChatPosition` main/archive list presence;
- exact-shape `mute` / `unmute` from TDLib `updateChatNotificationSettings`.

Folder labels/mutations, custom mute shapes, silent/admin participant
lifecycle, edit/topic writes, non-self reaction removal parity and Bot API
writes still need stronger provider-observed reconciliation before they can be
marked `completed`. Provider-observed edit source evidence currently lands in
append-only `telegram_message_versions` rows plus realtime/event-log payloads;
the raw `telegram_message` communication record remains append-only by
`provider_record_id`.

Media upload follows the same provider-command boundary:

```text
Communication attachment import
  -> local blob + communication_attachment_imports
Telegram media upload API
  -> send_media command
Outbox executor
  -> TDLib sendMessage media request from local blob path
  -> provider-observed message snapshot
  -> Communication projection refresh
```

UI must pass `attachment_id` or `blob_id`; it must not upload directly to TDLib
or Bot API.

`completed` is reserved for provider-observed state. A successful TDLib ACK is
not enough for completion unless the actor returns a concrete provider message
snapshot. ACK-only writes remain `executing/awaiting_provider` until a later
provider reconciliation pass observes the target state.
For participant lifecycle, TDLib member sync is a provider-observed source:
when `getSupergroupMembers` returns the selected account's own active
`user:<telegram_id>` roster row, matching self `join` commands may be marked
`completed` and emit `telegram.command.reconciled`. The current recent roster
page is not an authoritative absence proof. TDLib history sync is also a
provider-observed source when explicit `messageChatAddMembers` or
`messageChatDeleteMember` service messages name the selected account; those
events can reconcile matching self `join`/`leave` commands. Silent/admin
membership state changes still require a stronger provider observation path.

## Dialog / Chat Model

Текущий chat kind:

```text
private
group
channel
bot
```

Целевые distinctions:

- private chat;
- bot dialog;
- basic group;
- supergroup;
- channel;
- forum/topic-enabled supergroup;
- saved messages;
- archived chats;
- pinned chats;
- muted chats;
- folders/chat lists.

## Replies, Forwards, Reactions

Telegram требует first-class projection для:

- reply target;
- reply chain;
- forward attribution;
- forward chain;
- mentions;
- reactions;
- pinned messages;
- topic identity.

Raw TDLib JSON недостаточно для устойчивого UI и provider parity. Оно должно
сохраняться как evidence, но UI/queries должны работать через projection contract.

## Media Lifecycle

Текущий lifecycle:

```text
TDLib message raw metadata
  -> UI attachment hint from message metadata
  -> POST /api/v1/integrations/telegram/provider-media/download
  -> TDL
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/telegram/blockers.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/blockers.md`
- Size bytes / Размер в байтах: `2013`
- Included characters / Включено символов: `2013`
- Truncated / Обрезано: `no`

```markdown
# Telegram Architectural Blockers

Status date: 2026-06-18.

The Telegram channel capability set has no active architectural blockers.

## Closed Blockers

| Blocker | Resolution |
|---|---|
| Capability contract granularity | Operation-level capability contract includes `available`, `blocked`, `degraded`, `unsupported` and `planned`, with action class, reason and confirmation flags. |
| Provider-write command parity | Base provider writes use durable command rows, audit and provider-observed reconciliation. ACK is not success. |
| Message lifecycle evidence | Edit versions, tombstones, provider observations and diff metadata are durable. |
| Reply/forward projection | Reply/forward refs are idempotent; chains are bounded and cycle-guarded. |
| Topic projection/realtime | Topic projection, unread state, runtime topic events and reconciliation are implemented. |
| Provider search parity | Provider message/media search refreshes projection before returning UI-visible results. |
| Media boundary | Gallery, album metadata, preview, upload and download use the command/query model and shared Communication attachment boundary. |
| Frontend state ownership | Telegram production UI uses TanStack Query composables and shared realtime bootstrap. |

## Planned Work Outside Base Telegram

| Initiative | Reason |
|---|---|
| Bot Runtime | Separate provider/runtime model from TDLib user runtime. |
| Voice Recording / Voice Send | Requires explicit desktop microphone permission boundary. |
| Video Recording / Live Calls | Requires separate native device/call permission design. |
| Session Export / Session Import | Requires encrypted local-first session bundle ADR and audit. |
| MTProxy / SOCKS5 | Requires proxy profile model and connection policy. |
| AI Summary / Translation / Bilingual Reply / AI Review Flows | Belongs to AI Layer and shared review engines, not base Telegram. |

Hidden recording, Telegram private-data fine-tuning and untrusted third-party
plugin execution remain unsupported.
```

### `docs/integrations/telegram/gap-analysis.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/gap-analysis.md`
- Size bytes / Размер в байтах: `3255`
- Included characters / Включено символов: `3255`
- Truncated / Обрезано: `no`

```markdown
# Telegram Gap Analysis

Status date: 2026-06-18.

Base Telegram channel capability status: `COMPLETED`.

This document tracks active Telegram channel capability gaps only. Deferred
initiatives are not base Telegram channel gaps; ADR-0094 and ADR-0097 move them
to separate future work.

## Closure Summary

| Area | Status | Evidence |
|---|---|---|
| Provider reconciliation | CLOSED | Edit, delete, pin, archive, mute, read, unread, reactions, topics, folder add, folder remove and folder reassign use durable provider-write commands and reconcile from provider-observed state or returned provider snapshots. |
| Message lifecycle | CLOSED | Edit versions, tombstones, provider edit/delete evidence and diff metadata are persisted and surfaced through projection APIs. |
| Reply/forward parity | CLOSED | Reply refs are idempotent, reply graph traversal is bounded with cycle guard, forward attribution is idempotent, and forward chains traverse projected local evidence without raw TDLib UI dependency. |
| Topic parity | CLOSED | Topic unread state, realtime topic patching and topic command reconciliation are implemented through `telegram_topics`, runtime topic events and command reconciliation. |
| Dialog parity | CLOSED | Pinned, archived, mute, unread and folder state use provider evidence when TDLib state is available; local projection remains the read model, not a success substitute. |
| Search parity | CLOSED | Message, media, topic and member search use projection-backed Communications routes; provider search is runtime/control sync-assist only and does not return UI-visible business items. |
| Media parity | CLOSED | Gallery, album metadata, preview, attachment lifecycle, upload lifecycle and download lifecycle use the command/query model and shared Communication attachment boundary. |
| Frontend state | CLOSED | Telegram production components use TanStack Query composables and shared realtime bootstrap; no component-level `fetch(` remains in Telegram production UI. |
| Architecture guardrails | CLOSED | Telegram remains a Communication Channel; Memory, Knowledge, Persona, Organization, Project, Obligation and Decision lifecycle stay outside Telegram. |

## Deferred, Not Gaps

| Capability | Capability state |
|---|---|
| Bot Runtime | `planned` |
| Voice Recording | `planned` |
| Voice Send | `planned` |
| Video Recording | `planned` |
| Live Calls | `planned` |
| Session Export | `planned` |
| Session Import | `planned` |
| MTProxy | `planned` |
| SOCKS5 | `planned` |
| AI Summary | `planned` |
| Translation | `planned` |
| Bilingual Reply | `planned` |
| AI Review Flows | `planned` |

## Closure Gates

| Gate | Status |
|---|---|
| Provider writes use outbox | CLOSED |
| Destructive actions use audit | CLOSED |
| Realtime events use shared event bus/bootstrap | CLOSED |
| Polling is not used where realtime path exists | CLOSED |
| Telegram implementation, test, docs and frontend files stay under 700 lines | CLOSED |
| Documentation matches channel capability scope | CLOSED |

Live Telegram validation remains opt-in because TDLib credentials, native
library loading and account QR authorization are local machine concerns.
Fixture/projection/outbox/realtime tests are the deterministic closure gate.
```

### `docs/integrations/telegram/modules.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/modules.md`
- Size bytes / Размер в байтах: `3454`
- Included characters / Включено символов: `3454`
- Truncated / Обрезано: `no`

```markdown
# Telegram Modules

Status: `COMPLETED` base-domain module map, 2026-06-18.

Telegram remains a Communication Channel. The modules below supply evidence,
commands, projections, realtime events, identity traces, timeline evidence and
media evidence. They do not implement shared Memory, Knowledge, Persona,
Organization, Project, Obligation or Decision lifecycle.

## Backend Modules

| Module | Files | Status |
|---|---|---|
| API | `backend/src/integrations/telegram/api/` | DONE |
| Accounts | `backend/src/integrations/telegram/client/accounts/` | DONE |
| Capabilities | `backend/src/domains/api_support/telegram_capabilities.rs`, `telegram_capability_catalog*.rs` | DONE |
| Dialogs | `client/chats.rs`, `client/chat_state.rs`, runtime chat events | DONE |
| Messages | `client/messages/`, `api/messages.rs`, `api/messages/` | DONE |
| Message lifecycle | `client/lifecycle/` | DONE |
| Reply/forward references | `client/references.rs` | DONE |
| Reactions | `client/reactions.rs`, `api/messages/reactions.rs` | DONE |
| Topics | `client/topics.rs`, runtime topic events | DONE |
| Runtime | `backend/src/integrations/telegram/runtime/` | DONE |
| TDLib bridge | `backend/src/integrations/telegram/tdjson/` | DONE |
| Search | `api/search.rs`, runtime search manager | DONE |
| Media | `api/media.rs`, runtime media/download manager | DONE |
| Attachments | `client/messages/attachments.rs`, shared Communication attachment boundary | DONE |
| Audit | `backend/src/platform/audit/telegram.rs` | DONE |
| Realtime | `backend/src/platform/events/`, Telegram runtime event bridge | DONE |

## Frontend Modules

| Module | Files | Status |
|---|---|---|
| Workbench page | `frontend/src/integrations/telegram/views/TelegramPage.vue` | DONE |
| API clients | `frontend/src/integrations/telegram/api/` | DONE |
| TanStack Query composables | `frontend/src/integrations/telegram/queries/` | DONE |
| Local UI store | `frontend/src/integrations/telegram/stores/telegram.ts` | DONE |
| Dialog list/actions | `TelegramChatList.vue`, `TelegramActionRail.vue`, `dialogActionHelpers.ts` | DONE |
| Thread/timeline | `TelegramMessageThread.vue`, `thread/TelegramMessageList.vue` | DONE |
| Lifecycle/reference evidence | `thread/TelegramMessageReferencePanel.vue`, lifecycle/reference queries | DONE |
| Composer | `thread/TelegramComposer.vue`, send action composables | DONE |
| Media gallery/viewer | `thread/TelegramThreadSideSections.vue`, `thread/TelegramMediaViewer.vue`, media search store | DONE |
| Search results | `TelegramSearchResultsPanel.vue`, search query composables | DONE |
| Inspector | `TelegramRail.vue`, account/runtime/capability/member/audit panels | DONE |
| Realtime cache patching | `queries/realtimeTelegramPatches.ts` and shared realtime bootstrap | DONE |

## Deferred Initiative Modules

| Initiative | Status |
|---|---|
| Bot Runtime | planned |
| Voice Recording / Voice Send | planned |
| Video Recording / Live Calls | planned |
| Session Export / Session Import | planned |
| MTProxy / SOCKS5 | planned |
| AI Summary / Translation / Bilingual Reply / AI Review Flows | planned |

## Boundary Rules

Telegram code may depend on Communications, Events, Timeline interfaces, shared
attachment/blob storage, Search, Audit and Secret resolver/host vault
boundaries.

Telegram code must not own shared engine lifecycle. It may only emit evidence,
provider commands, projections and reviewable traces consumed by those engines.
```

### `docs/integrations/telegram/product-research.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/product-research.md`
- Size bytes / Размер в байтах: `9588`
- Included characters / Включено символов: `9588`
- Truncated / Обрезано: `no`

```markdown
# Telegram Product Research And Next Bets

Status: research snapshot, 2026-06-23.

Scope: exploratory product research for the next Telegram-adjacent work after
the base channel capability set. This document is not an ADR, implementation
plan or committed roadmap. Treat it as input for future specs, ADRs and
provider-neutral Communications work.

## Local Context

Telegram in Макошь is an integration channel, not a product domain. ADR-0097
keeps durable communication state inside the Communications domain. ADR-0099
keeps source control, pause/resume/replay and source health inside Signal Hub.

The useful next product work should therefore avoid Telegram-client parity.
Telegram can be the first rich evidence source for provider-neutral
Communications, Radar, Memory, Persona, Relationship, Obligation and Decision
workflows.

Observed local surfaces at the time of this research:

- backend Telegram integration covers account setup and lifecycle, chat
  metadata and reconciliation, commands, evidence, messages, attachments,
  manual send, reaction metadata, references, search, topics, participants,
  TDLib parsing, runtime actors, sync, media download and realtime events;
- frontend Telegram work exists inside the Communications workbench through
  Telegram panels, business queries and realtime patches for messages, media,
  topics and participants;
- base channel status and gap documents mark the core Telegram capability set
  as complete for daily desktop work.

## External Product Patterns

### Telegram

Sources:

- <https://telegram.org/blog/folders>
- <https://telegram.org/blog/shareable-folders-custom-wallpapers>
- <https://telegram.org/blog/new-saved-messages-and-9-more>
- <https://telegram.org/blog/reply-revolution>
- <https://telegram.org/blog/ultimate-privacy-topics-2-0>

Useful patterns:

- Chat folders separate noisy chat lists into work, unread, channel and other
  focused views.
- Shareable folders package a set of groups and channels behind an invite link.
- Saved Messages behaves like personal storage for links, media, bookmarks and
  tagged saved items.
- Replies can quote precise fragments and move context across chats.
- Topics organize high-volume groups, while privacy and deletion controls make
  provider history mutable.

Макошь interpretation:

- folders and topics are useful as local view and workflow inputs, not as a
  reason to build a Telegram clone;
- saved messages map well to a source-backed evidence shelf with tags,
  provenance and later Memory or Document promotion;
- quote fragments are a strong primitive for stable evidence capture across
  noisy conversations;
- provider edit/delete behavior should become local evidence, tombstones and
  history, not silent loss of truth.

### Beeper

Source:

- <https://www.beeper.com/faq>

Useful patterns:

- unified inbox across many chat networks;
- cross-device sync and a strong "one app for chats" posture;
- emphasis on trust, encryption and local/on-device connection where possible.

Макошь interpretation:

- unified inbox is useful, but Макошь should differentiate on local-first
  evidence, auditability and provider-neutral context rather than chat-client
  aggregation alone.

### Missive

Source:

- <https://missiveapp.com/docs/core-features/team-inboxes>

Useful patterns:

- shared inbox triage;
- assignment, transfer, close and reopen workflows;
- conversation ownership and status as first-class working state.

Макошь interpretation:

- the team-inbox model can be adapted to solo/operator workflows: assign to
  self, defer, close, reopen, track owner state and surface unresolved
  obligations without turning Макошь into a help desk.

### Superhuman

Source:

- <https://superhuman.com/products/mail>

Useful patterns:

- split inbox for priority lanes;
- reminders and follow-up workflows;
- keyboard-first triage;
- AI drafting and live sharing/commenting.

Макошь interpretation:

- priority lanes and follow-up obligations are higher-value than more raw
  Telegram UI parity;
- reply drafts can be useful, but they must stay behind explicit user
  confirmation and capability gates.

### Shortwave

Sources:

- <https://www.shortwave.com/>
- <https://www.shortwave.com/changelog/>

Useful patterns:

- AI filters written in plain English;
- AI-powered search over messages and attachments;
- automated tasklets for drafts, labels, comments and todo extraction.

Макошь interpretation:

- start with local suggestions, drafts, classifications and candidate
  obligations;
- do not allow automated provider writes or destructive provider-side actions
  without explicit confirmation.

## Research Themes

1. Unified Telegram inbox work should not imitate Telegram. It should expose
   workable conversations across accounts with local overlays: priority, owner
   state, obligation, trust, source evidence and runtime posture.
2. Folders, topics and saved messages are not only UI categories. In Макошь
   they can become local saved/search/task/memory surfaces with provenance.
3. Follow-up and reminder workflows fit Макошь better than generic chat parity.
4. AI should draft, summarize, classify and suggest actions locally first.
   Provider writes stay manually confirmed.
5. Team-inbox ideas can be reduced to solo/operator workflows: assign to self,
   defer, close, reopen, "needs decision" and "waiting".
6. Privacy and evidence boundaries are product differentiators: local
   tombstones, edit/version history, no hidden provider mutation and clear
   capability states.

## Consensus Next Bets

### 1. Context Inbox Lanes

Build provider-neutral lanes such as:

- Now;
- Waiting;
- Needs reply;
- Decision needed;
- Snoozed;
- High-trust;
- Noise.

Start with local owner-state overlays. Add Persona and Relationship-derived
lanes after identity quality is good enough.

Validation ideas:

- dogfood whether "Now" replaces scanning "All";
- fixture tests for snooze, reopen and realtime wake behavior.

### 2. Obligation And Decision Radar

Generate evidence-backed candidates from Telegram messages:

- "I owe";
- "they owe";
- "follow up";
- "decision pending".

The owner reviews and promotes candidates. Макошь must not automatically create
tasks, obligations or decisions from private messages without review.

Validation ideas:

- candidate acceptance rate;
- false-positive review cost;
- regression fixtures that separate casual chat from explicit commitments.

### 3. Source-Backed Evidence Capture

Allow saving a message, quote fragment, link, media item or file into a local
evidence shelf with:

- source citation;
- version or content hash where available;
- tags;
- Persona or project context;
- optional Memory or Document promotion.

Validation ideas:

- saved item retrieval through search;
- quote stability across edited or deleted messages;
- scanner-state and storage-boundary tests for media.

### 4. Forensic Conversation Timeline

Surface local evidence that normal chat clients hide:

- observed edit versions;
- tombstones;
- provider deletions;
- local delete reasons;
- source citations;
- scoped export.

This is a Макошь-native differentiator, but it needs careful privacy language
and UI framing.

Validation ideas:

- edit/delete fixture matrix;
- export scope tests;
- UX review for "valuable evidence" versus surprising retention.

### 5. Signal Hub Control And Runtime Posture

Expose source posture as enabling infrastructure:

- enabled;
- paused;
- muted;
- degraded;
- replaying;
- unhealthy;
- fixture mode.

Telegram account runtime health should appear here, but source policy belongs
to Signal Hub, not to a Telegram product domain.

Validation ideas:

- fixture source pause/resume/replay;
- disabled Telegram source stops projections without losing raw evidence;
- degraded source state is visible without implying data loss.

## Preserved Disagreements

- Persona Inbox is strategically useful, but it belongs to provider-neutral
  Communications plus Persona and Relationship systems, not to a Telegram
  module.
- Context Inbox Lanes are the better first slice because local owner state can
  ship before fuzzy identity resolution is perfect.
- AI reply drafts are useful but are not the primary Telegram bet. Keep them
  behind AI/runtime capability gates and explicit confirmation.
- Topics, media gallery, search, folders and reactions remain important
  integration polish. They are not the headline next product direction if the
  base Telegram status documents are accurate.

## Explicit Rejects

- Telegram clone parity as a product goal;
- automated live sends;
- provider-side destructive automation;
- mobile UI before ADR-0031 is superseded;
- calls, recording and screen sharing as part of this next slice;
- auto-download-all-media behavior;
- cloud indexing of private messages;
- physical delete by default;
- Telegram-owned task, decision or memory state.

## Open Questions

- What is the canonical provider-neutral schema for lane and workflow state?
- How should Persona resolution represent groups, channels, bots and
  organization proxies?
- What evidence snapshot is required for quote fragments across edited or
  deleted messages?
- What retention and privacy language makes forensic history understandable
  without surprising the owner?

## Research Provenance

This snapshot combines:

- repository context from Telegram status and gap documentation;
- ADR constraints from ADR-0097 and ADR-0099;
- external product research into Telegram, Beeper, Missive, Superhuman and
  Shortwave;
- a dual-model brainstorm and debate pass using Codex and Claude Code.
```

### `docs/integrations/telegram/status.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/status.md`
- Size bytes / Размер в байтах: `1401`
- Included characters / Включено символов: `1401`
- Truncated / Обрезано: `no`

```markdown
# Telegram Implementation Status

Status date: 2026-06-18.

Base Telegram channel capability set: `COMPLETED`.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

## Summary

| Area | Status |
|---|---|
| Communication Channel framing | DONE |
| Provider account/runtime metadata | DONE |
| Capability contract with `planned` state | DONE |
| Provider-write outbox and audit | DONE |
| Provider-observed reconciliation | DONE |
| Dialogs, folders and unread state | DONE |
| Message lifecycle evidence | DONE |
| Reply/forward graph evidence | DONE |
| Topics | DONE |
| Reactions | DONE |
| Search | DONE |
| Media and attachments | DONE |
| Realtime event bus/bootstrap | DONE |
| TanStack Query frontend state | DONE |
| Documentation closure | DONE |

## Deferred Initiatives

ADR-0094 and ADR-0097 move the following outside the base Telegram channel
capability set: Bot Runtime, Voice, Video/Calls, Session import/export, MTProxy,
SOCKS5 and Telegram-specific AI flows. Their capability state is `planned`.

## Validation Policy

Live TDLib validation is opt-in and depends on local credentials/native
resources. Deterministic closure uses fixture, projection, outbox and realtime
regression tests.

## Navigation

- [Pass Log](status/pass-log.md)
- [Core Details](status/details-core.md)
- [Extended Details](status/details-extended.md)
```

### `docs/integrations/telegram/status/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/status/README.md`
- Size bytes / Размер в байтах: `320`
- Included characters / Включено символов: `320`
- Truncated / Обрезано: `no`

```markdown
# Telegram Status Details

Status: documentation package aligned to the current repository structure.

This package stores detailed Telegram status evidence split from the parent
status document.

## Navigation

- [Pass Log](./pass-log.md)
- [Core Details](./details-core.md)
- [Extended Details](./details-extended.md)
```

### `docs/integrations/telegram/status/details-core.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/status/details-core.md`
- Size bytes / Размер в байтах: `2210`
- Included characters / Включено символов: `2210`
- Truncated / Обрезано: `no`

```markdown
# Telegram Core Status Details

Status date: 2026-06-18.

Base Telegram channel capability set: `COMPLETED`.

## Account And Runtime

- Fixture and live user account metadata are implemented.
- QR authorization routes and UI are implemented.
- Runtime status/start/stop/restart diagnostics are implemented.
- Bot Runtime is deferred as `planned` under ADR-0094.

## Capability Contract

- Global and account-scoped capability responses expose operation status,
  action class, reason, confirmation requirement and account overrides.
- Supported states are `available`, `blocked`, `degraded`, `unsupported` and
  `planned`.
- Deferred initiatives are API-visible as `planned`.

## Dialogs

- Chat projections, detail routes, folder filters and selected-chat controls are
  implemented.
- Pin/archive/mute/read/unread/folder commands use provider-write command rows.
- TDLib chat-state events reconcile provider-observed state before commands are
  completed.
- Folder add/remove/reassign use provider folder evidence and realtime patches.

## Messages

- Source-backed message projection and sanitized raw evidence access are
  implemented.
- Text send, edit, delete, restore, pin, reply, forward and reaction commands
  use capability gates and command/audit records.
- Provider-observed edits and deletes produce versions/tombstones and realtime
  events.
- Diff metadata records previous/new previews, lengths and hashes.

## References

- Reply refs and forward refs are idempotent.
- Reply chains traverse ancestors and descendants with fixed bounds and cycle
  guard.
- Forward chains traverse locally projected provider attribution with fixed
  bounds and cycle guard.
- UI consumes projected summaries, not raw TDLib payloads.

## Topics

- Forum topics are projected, listed, searched and shown in the workbench.
- Topic unread state and last-message state are persisted.
- Runtime topic events patch frontend caches and reconcile close/reopen
  commands from provider-observed state.

## Reactions

- Add/remove commands use the outbox and audit.
- TDLib history/runtime interaction updates project provider reaction aggregates.
- Self reaction commands reconcile from provider-observed chosen state.
```

### `docs/integrations/telegram/status/details-extended.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/status/details-extended.md`
- Size bytes / Размер в байтах: `2250`
- Included characters / Включено символов: `2250`
- Truncated / Обрезано: `no`

```markdown
# Telegram Extended Status Details

Status date: 2026-06-18.

Base Telegram channel capability set: `COMPLETED`.

## Search

- Dialog, message, provider, media, topic and member search routes are
  implemented.
- Provider message/media search refreshes TDLib results into local projections
  before returning UI-visible results.
- Search UI uses TanStack Query composables and projection-backed result panels.

## Media And Attachments

- Telegram media metadata is projected from source evidence.
- Album metadata is preserved in message attachment metadata and gallery views.
- Upload uses shared Communication attachment import plus Telegram provider
  command rows.
- Download emits started/progress/failed/completed realtime events, persists
  local blob/attachment rows and patches projected attachment metadata.
- Preview uses the shared Communication attachment preview boundary and local
  downloaded media paths.

## Realtime

- Telegram runtime and API events use the shared event bus/bootstrap.
- Frontend cache patching covers message, chat, command, media, typing, topic,
  pinned/search and runtime event families.
- Realtime is preferred over polling where a realtime path exists. QR login
  status remains a bounded authorization-status flow because it is not a
  message/dialog runtime event stream.

## Frontend

- Production Telegram server state is accessed through TanStack Query
  composables.
- Component-level `fetch(` is absent from Telegram production components.
- Page-level orchestration delegates backend state to query/mutation composables
  and keeps Pinia for local UI state only.

## Audit And Safety

- Destructive and provider-write actions record redacted audit metadata.
- Message bodies, media bytes and secrets are not logged in Telegram event
  payloads.
- Secret payloads remain outside PostgreSQL; provider account records store
  metadata and secret references only.

## Scope Boundary

Telegram emits evidence and traces for shared systems. It does not implement
Memory, Knowledge, Persona, Organization, Project, Obligation or Decision
lifecycle.

The following remain separate planned initiatives: Bot Runtime, Voice,
Video/Calls, Session import/export, MTProxy, SOCKS5 and Telegram-specific AI
flows.
```

### `docs/integrations/telegram/status/pass-log.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/telegram/status/pass-log.md`
- Size bytes / Размер в байтах: `1096`
- Included characters / Включено символов: `1096`
- Truncated / Обрезано: `no`

```markdown
# Telegram Completion Pass Log

Status date: 2026-06-18.

## Closure Pass

| Pass | Result |
|---|---|
| Provider reconciliation | CLOSED |
| Message lifecycle evidence | CLOSED |
| Reply/forward parity | CLOSED |
| Topic parity | CLOSED |
| Dialog parity | CLOSED |
| Search parity | CLOSED |
| Media parity | CLOSED |
| Frontend query/realtime boundary | CLOSED |
| Documentation alignment | CLOSED |

## Evidence

- ADR-0094 defines base Telegram completion and deferred initiatives.
- Capability contract exposes `planned` for deferred initiatives.
- Provider writes are represented as durable provider-write commands.
- Destructive actions use audit records.
- Realtime events flow through shared backend event bus and frontend bootstrap.
- Telegram production UI uses TanStack Query composables for server state.
- Telegram implementation, frontend, test and docs files are kept under the
  700-line architecture guardrail.

## Deferred Passes

Future work must be opened as separate initiatives:

- Bot Runtime;
- Voice;
- Calls / video recording;
- AI Layer;
- Session/proxy portability.
```

### `docs/integrations/whatsapp/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/README.md`
- Size bytes / Размер в байтах: `11149`
- Included characters / Включено символов: `9412`
- Truncated / Обрезано: `no`

````markdown
# Макошь Communications — WhatsApp Channel

Статус: стартовый audit/spec набор на 2026-06-17.

WhatsApp в Макошь — это **Communication Channel** внутри Communications Domain.
WhatsApp не является отдельным продуктом, отдельным мессенджером и не владеет
Memory, Knowledge, Tasks, Projects, Personas, Organizations, Decisions или
Obligations.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

Макошь не проектируется как WhatsApp-клиент-клон. WhatsApp поставляет:

- source evidence;
- provider records;
- provider commands;
- attachments;
- media;
- identity traces;
- realtime events;
- timeline evidence.

```text
Hidden desktop WebView
  -> WhatsApp Adapter
  -> Communication Projection
  -> Events
  -> Timeline
  -> Shared Engines
```

`Hidden desktop WebView` означает контролируемую desktop companion-сессию с
явным owner-visible lifecycle, permissions и status UI. Это не headless scraping,
не невидимая запись и не попытка заменить WhatsApp Web неофициальным API.


## 2026-06-24 planning documents

The following documents extend the original WhatsApp Channel audit toward full
functionality and provider-runtime selection:

- [`current-audit-2026-06-24.md`](./current-audit-2026-06-24.md) — current repository audit and implemented fixture foundation.
- [`full-functionality-target.md`](./full-functionality-target.md) — complete WhatsApp capability target plus Макошь intelligence flows.
- [`rust-provider-research.md`](./rust-provider-research.md) — Rust project evaluation: `whatsapp-rust`, `wa-rs`, `whatsappweb-rs`, Business Cloud SDKs and references.
- [`implementation-plan.md`](./implementation-plan.md) — phased implementation plan from fixture foundation to live provider and Макошь intelligence.
- [`api.md`](./api.md) — current backend integration/runtime API surface.
- [`fixture-test-matrix.md`](./fixture-test-matrix.md) — fixture/runtime-safe source-record and command-class coverage map.
- [`live-smoke-checklist.md`](./live-smoke-checklist.md) — manual local checklist for live runtime validation, redaction verification and vault-backed session restore.
- [`../../adr/ADR-0101-whatsapp-provider-runtime-selection.md`](../../adr/ADR-0101-whatsapp-provider-runtime-selection.md) — proposed runtime/provider selection decision.

## Роль в Communications Domain

WhatsApp использует те же базовые границы, что и другие каналы коммуникации:

- provider state не является source of truth для Макошь;
- raw provider records сохраняются как append-only evidence;
- canonical `communication_messages` являются проекцией;
- provider writes проходят через capability, outbox, policy и audit boundary;
- AI output, indexes и summaries не заменяют source evidence;
- realtime является частью канала, а не косметическим обновлением UI.

Первичный provider source:

```text
WhatsApp Web
```

Первичная реализация:

```text
Local First
Desktop First
Personal Use
```

Meta Business API не является основным источником архитектуры. Он допускается
только как отдельный будущий provider shape с отдельным capability и ADR
решением.

## Ключевые принципы WhatsApp Channel

### Evidence First

Любое WhatsApp-сообщение, статус, медиафайл, реакция, удаление, provider update
или runtime event должны сохраняться как проверяемое evidence. Производные поля
для UI, AI, поиска или Timeline не должны заменять исходные provider records.

### Capability Gated

Каждая provider-side операция должна иметь capability state до появления в UI:

```text
available
degraded
blocked
unsupported
```

Минимальный capability set:

- send;
- reply;
- forward;
- reaction;
- delete;
- media upload;
- media download;
- join group;
- leave group;
- status read;
- status publish;
- voice send.

### Local First

Личные данные, raw evidence, attachment metadata, local blobs, audit и derived
context остаются локальными. WhatsApp Web используется как provider/source
boundary, а не как долговременное хранилище Макошь.

### Owner Controlled

Provider-write commands предлагают действия, но исполняются только через
backend-controlled capability and confirmation boundary. UI не вызывает WebView
или provider adapter напрямую для отправки, удаления, реакции, публикации
статуса или загрузки медиа.

### No Hidden Recording

Calls, voice capture, screen capture, recording and STT не могут быть скрытыми.
Первая версия поддерживает только call metadata, call evidence и timeline
entries. Audio/video capture, call control, recording, live call handling и STT
остаются out of scope до отдельного ADR.

## Provider Accounts

Поддерживаемые account kinds на уровне продукта:

```text
whatsapp_personal
whatsapp_business
```

Для первой архитектурной формы оба account kinds используют WhatsApp Web
companion boundary. `whatsapp_business` означает аккаунт WhatsApp Business App,
связанный через Web companion. Это не Meta Business Platform Cloud API.

Provider kind для первой реализации:

```text
whatsapp_web
```

Future provider kind для Meta Business API должен быть отдельным, например:

```text
whatsapp_business_cloud
```

## Dialogs

WhatsApp Channel должен распознавать dialog source records для:

- private chat;
- group;
- community;
- broadcast;
- status.

Dialog projections являются Communication projections. Они не создают Project,
Persona, Organization или Memory lifecycle.

## Messages

Целевые message classes:

- text;
- reply;
- forward;
- reaction;
- delete;
- edit, если provider/runtime reliably supports it.

Edit history не должен реконструироваться задним числом. Макошь может хранить
только observed versions и source-backed update evidence.

## Media

Целевые media classes:

- photo;
- video;
- document;
- audio;
- voice note;
- contact;
- location;
- sticker;
- gif.

Media bytes не должны храниться в PostgreSQL. PostgreSQL хранит metadata,
hashes, scanner state и local blob references.

## Identity

WhatsApp является phone-centric provider.

WhatsApp Channel сохраняет identity traces:

- phone number;
- `wa_id`;
- display names;
- contact-card evidence;
- group member evidence;
- admin/member role evidence;
- community member evidence.

Эти traces могут создавать Persona candidates, relationship candidates и contact
resolution evidence. Текущая fixture foundation уже материализует source-backed
unattached `person_identities` traces для `whatsapp` и `phone` через existing
Persona compatibility boundary, а также `message_participant` traces для group /
community member evidence и phone traces из contact-card metadata. Эти traces
теперь также сохраняют source-backed WhatsApp evidence metadata, включая push
name, business profile, profile photo ref, participant role/status и
contact-card payload, но WhatsApp Channel не реализует Persona Domain и не
объявляет номер телефона окончательной Persona truth.

## WhatsApp Statuses

WhatsApp Status рассматривается как:

```text
source evidence
timeline evidence
identity signal
```

Statuses не являются отдельным доменом. Они входят в WhatsApp Channel как
provider evidence и могут создавать Timeline evidence, identity signals and
review candidates.

## Voice Notes

Voice notes поддерживаются архитектурно как:

- voice metadata;
- voice attachment;
- local playback;
- future transcript integration point.

STT не входит в первую версию. Transcript integration должен быть отдельным
shared-engine flow с source references и explicit permission boundary.

## Calls

Первая версия поддерживает только:

- call metadata;
- call evidence;
- call timeline entries.

Out of scope:

- audio capture;
- video capture;
- call control;
- recording;
- live call handling;
- STT.

Эти возможности требуют будущего ADR.

## Связь с Timeline

WhatsApp messages, statuses, reactions, edits, deletes, media downloads,
provider-write commands, account lifecycle events and call metadata должны
становиться ordered Timeline evidence.

Timeline не владеет provider adapter logic. WhatsApp Channel поставляет
source-backed events and projections.

## Главные незакрытые области

- WhatsApp Web desktop companion runtime;
- account/session lifecycle and local WebView storage policy;
- live-complete capability policy/runtime enablement over the existing operation-level capability contract;
- durable provider-write outbox;
- dialog/message/status/media projections;
- phone-centric identity trace model;
- media download/upload pipeline;
- voice-note playback and future transcript boundary;
- call metadata and explicit no-recording boundary;
- realtime `whatsapp.*` event contracts;
- local validation fixtures and smoke tests.

## Навигация

- [Current Audit](current-audit-2026-06-24.md)
- [Full Functionality Target](full-functionality-target.md)
- [Implementation Plan](implementation-plan.md)
- [API Reference](api.md)
- [Fixture Test Matrix](fixture-test-matrix.md)
- [Rust Provider Research](rust-provider-research.md)
- [Live Smoke Checklist](live-smoke-checklist.md)

## Navigation

- [Architecture](./architecture.md)
- [API Reference](./api.md)
- [Modules](./modules.md)
- [Status](./status.md)
- [Gap Analysis](./gap-analysis.md)
- [Blockers](./blockers.md)
- [Implementation Plan](./implementation-plan.md)
- [Fixture Test Matrix](./fixture-test-matrix.md)
- [Live Smoke Checklist](./live-smoke-checklist.md)
- [Current Audit](./current-audit-2026-06-24.md)
- [Full Functionality Target](./full-functionality-target.md)
- [Rust Provider Research](./rust-provider-research.md)
````

### `docs/integrations/whatsapp/api.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/api.md`
- Size bytes / Размер в байтах: `25637`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# WhatsApp Integration API

Status: current repository API surface for the WhatsApp integration runtime and
fixture foundation.
Date: 2026-06-26.

This document describes the currently implemented backend routes under:

```text
/api/v1/integrations/whatsapp/*
```

These routes are integration/runtime surfaces. Provider-neutral user-facing
writes and reads belong under:

```text
/api/v1/communications/*
```

## Runtime and capability routes

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/api/v1/integrations/whatsapp/capabilities` | Provider-family capability matrix. |
| `GET` | `/api/v1/integrations/whatsapp/accounts` | List WhatsApp integration accounts; `include_removed=true` includes logically removed entries. |
| `GET` | `/api/v1/integrations/whatsapp/accounts/{account_id}/capabilities` | Account-scoped capability overlay. |
| `POST` | `/api/v1/integrations/whatsapp/accounts` | Create blocked live account for explicit provider shape. For `whatsapp_business_cloud`, `api_access_token` is required and is stored only in host vault as `whatsapp_business_cloud_access_token`; optional `app_secret` and `webhook_verify_token` are also stored only in host vault for webhook signature/challenge verification. PostgreSQL stores metadata and binding refs only. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/status` | Runtime lifecycle/account status. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/start` | Start runtime or return blocked-safe lifecycle state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/stop` | Stop runtime or return blocked-safe lifecycle state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/revoke` | Revoke linked runtime/session state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/relink` | Move account back to relink-required state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/rotate` | Alias for relink-safe session migration/rotation into link-required state. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/remove` | Logical runtime/account removal surface. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/health` | Sanitized runtime health summary with `available` / `degraded` / `blocked` status plus nested `session`, `storage`, `runtime`, `webview` and `validation` diagnostics. |
| `POST` | `/api/v1/integrations/whatsapp/login/qr/start` | Start QR link flow when provider shape supports it. |
| `POST` | `/api/v1/integrations/whatsapp/login/pair-code/start` | Start pair-code flow when provider shape supports it. |

## Provider sync and listing routes

For `whatsapp_web_companion`, `runtime/health` also exposes
`checks.web_companion_bridge` and `checks.runtime.web_companion_bridge`. This is
a contract manifest, not a live availability flag. It requires an owner-visible
desktop WebView, forbids hidden/headless mode, binds restorable session state to
the host-vault `whatsapp_web_session_key`, lists the protected
`/runtime-bridge/*` event routes the producer must use, fixes provider writes to
the durable outbox claim/failure paths and excludes session material, cookies,
browser profile secrets, QR/pair-code artifacts, message bodies and media bytes
from health/event-like payloads.

The desktop shell also exposes Tauri commands
`open_whatsapp_web_companion` and `whatsapp_web_companion_manifest`. They are
local shell commands, not backend HTTP API. The opener creates or focuses an
owner-visible `https://web.whatsapp.com/` companion WebView and returns a
sanitized manifest containing only runtime-bridge paths, command-channel policy
and secret-storage policy. It does not read or return cookies, session material,
browser profile secrets, message bodies or media bytes. The WhatsApp Runtime
panel exposes this path through its owner-visible `Open Companion` action. The
visible shell is still blocked for public availability until manual smoke
passes.
Frontend code uses `openWhatsappWebCompanion` and
`getWhatsappWebCompanionManifest` from the integration API layer to invoke
these Tauri commands directly; this bridge deliberately does not use
`ApiClient` or backend HTTP routes.

The companion window installs a main-frame-only initialization script guarded to
`https://web.whatsapp.com`, and navigation is constrained to that origin. The
script exposes a frozen `__MAKOSH_WHATSAPP_COMPANION__` metadata contract plus a
local DOM readiness event and an allowlisted metadata-only relay dispatch. It
does not read cookies, Web Storage, IndexedDB, browser profile secrets, session
material, message bodies or media bytes, and it does not call `fetch`, XHR,
`postMessage` or domain APIs. Runtime health and the frontend manifest report
this as `contract_injected_relay_dispatch_available` with
`tauri_allowlisted_companion_runtime_bridge_dispatch`. The Tauri relay command
posts sanitized metadata observations to the protected local
`/api/v1/integrations/whatsapp/runtime-bridge/runtime-events` route using
`X-Макошь-Secret` from the Tauri process environment only. This creates
runtime-event evidence but does not project typed messages/status/media until a
richer typed WebView payload exists.

For `whatsapp_native_md`, `runtime/health` also exposes
`checks.native_md_driver.provider_command_surface.wa_rs_sdk_command_gap` and
the same block under `checks.runtime.native_driver`. This is evidence metadata,
not live availability. It names the verified `wa-rs 0.2.0` public SDK methods
used by the smoke-gated subset, including the forwarded-text reemit contract for
`forward`, and names the missing safe write APIs for `publish_status`,
dialog-state writes, `mark_unread` and join-by-invite. Commands in that gap may
only produce structured
terminal dead-letter evidence with `native_md_command_kind_unsupported`; they
must not complete until a provider-observed event reconciles them. The native
executor performs this unsupported-command preflight before the smoke gate and
runtime driver lookup, so missing SDK/API support is not reported as a transient
runtime-not-running condition and is not retried as if the provider might later
accept it.

For native media uploads, `send_media` / `send_voice_note` provider submissions
include a sanitized `provider_observed_completion_target`. The command may only
complete after accepted `signal.accepted.whatsapp.media` evidence observes the
provider message id returned by `wa-rs::Client::send_message`; fixture blob-id
matching remains a fallback for deterministic tests. Raw bytes, media keys,
direct paths and provider URLs are not part of the target payload.

For Business Cloud, Graph `send_text`, `send_template`, `send_media` and
`send_voice_note` submissions also include a sanitized
`provider_observed_completion_target`. The target points at accepted
`signal.accepted.whatsapp.receipt` evidence from webhook `statuses[]`; webhook
status `id` must match the stored Graph message id/provider request id before
the durable outbox command can complete. Access tokens, raw provider payloads,
template components and media bytes are excluded from the target payload.

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/chats` | Fixture-backed/provider-runtime chat sync control surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/history` | Fixture-backed/provider-runtime history sync control surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/conversations/{provider_chat_id}/members` | Fixture-backed/provider-runtime roster sync control surface backed by canonical participant projection. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/statuses` | Fixture-backed/provider-runtime status-feed sync control surface backed by canonical projected `status-feed` messages. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/presence` | Fixture-backed/provider-runtime presence sync control surface backed by canonical identity presence metadata; optional `provider_chat_id` scopes the snapshot to one chat. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/calls` | Fixture-backed/provider-runtime call sync control surface backed by the shared calls read model; optional `provider_chat_id` scopes the snapshot to one chat. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/contacts` | Fixture-backed/provider-runtime contact sync control surface backed by canonical communication identities plus compatibility WhatsApp/phone trace metadata. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/media` | Fixture-backed/provider-runtime media sync control surface backed by canonical communication attachments and local blob metadata; optional `provider_chat_id` and `content_type` filter the snapshot. |
| `GET` | `/api/v1/integrations/whatsapp/sessions` | Provider session list. Accountless listing aggregates across provider shapes. |
| `GET` | `/api/v1/communications/messages?channel_kind=whatsapp` | Provider-neutral WhatsApp message list. Accountless listing aggregates across provider shapes through Communications projections. |

## Provider command routes

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/send` | Queue send-text provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reply` | Queue reply provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/forward` | Queue forward provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/edit` | Queue edit provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/delete` | Queue delete provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Queue add-reaction provider command. |
| `DELETE` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Queue remove-reaction provider command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-media/upload` | Queue media upload/send command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-media/download` | Queue media download command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/voice-note` | Queue voice-note send command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/statuses/publish` | Queue status publish command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/join` | Queue join-group command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/leave` | Queue leave-group command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/read` | Queue mark-read command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unread` | Queue mark-unread command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/archive` | Queue archive command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unarchive` | Queue unarchive command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/mute` | Queue mute command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unmute` | Queue unmute command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/pin` | Queue pin command. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/{conversation_id}/unpin` | Queue unpin command. |
| `GET` | `/api/v1/integrations/whatsapp/commands` | List durable provider commands. |
| `POST` | `/api/v1/integrations/whatsapp/commands/{command_id}/retry` | Manual retry transition. |
| `POST` | `/api/v1/integrations/whatsapp/commands/{command_id}/dead-letter` | Manual dead-letter transition. |

`whatsapp_business_cloud` does not use the persona
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/whatsapp/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/architecture.md`
- Size bytes / Размер в байтах: `16456`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# WhatsApp Architecture

Статус: целевая архитектурная спецификация на 2026-06-17.

## Позиция

WhatsApp принадлежит Communications Domain как **channel/source boundary**.
Он не является Memory, Knowledge, Task, Project, Persona, Organization,
Decision или Obligation.

Invariant: A channel is never a domain. A channel is an integration. A
communication is the domain object.

WhatsApp должен поставлять Макошь:

- raw provider evidence;
- provider-specific metadata;
- messages/statuses/media/calls source records;
- provider commands;
- realtime updates;
- phone-centric identity traces;
- local desktop companion state.

## Canonical Flow

Целевой provider flow:

```text
Owner-visible desktop WebView companion
  -> WhatsApp Adapter / runtime bridge
  -> Signal Hub raw + accepted events
  -> Communications projection
  -> Events
  -> Timeline
  -> Shared Engines
```

Detailed flow:

```text
WhatsApp Web runtime event
  -> observation.captured.v1
  -> signal.raw.whatsapp.message.observed
  -> signal.accepted.whatsapp.message
  -> communication.message.recorded / communication.message.updated
  -> Timeline evidence / trace link
  -> Search / Risk / Enrichment / AI candidates
  -> UI cache patch + replay
```

`Owner-visible desktop WebView companion` is a desktop-owned companion surface.
It may run as a secondary or background WebView after explicit owner setup, but
it must expose session state, linking, revocation, permission and failure status
to the owner. It must not become hidden headless scraping. Its bridge contract is
machine-readable through `runtime/health`: runtime events enter protected
`/api/v1/integrations/whatsapp/runtime-bridge/*` routes, provider writes use the
durable outbox claim/failure paths, and session/cookie/profile secrets stay out
of PostgreSQL, events, logs and health payloads.

## Trace Contract

WhatsApp runtime events must not create product domain state directly.

WhatsApp provider observations enter the canonical trace as:

```text
observation.captured.v1
  -> signal.raw.whatsapp.message.observed
  -> signal.accepted.whatsapp.message
  -> communication.message.recorded / communication.message.updated
```

WhatsApp integration code owns companion runtime and provider protocol state.
Signal Hub owns source acceptance policy. Communications owns canonical message
state. Trace reconstruction belongs to `platform/events`.

## Key ADR

| ADR | Значение для WhatsApp |
|---|---|
| ADR-0001 | Event sourcing is system spine |
| ADR-0013 | Local-first data ownership |
| ADR-0018 | Provider adapter boundary |
| ADR-0027 | Capability-based permission model |
| ADR-0031 | Desktop-only UI scope |
| ADR-0046 | Blob storage and scanner boundary for attachment bytes |
| ADR-0051 | WhatsApp Web companion boundary |
| ADR-0052 | Capability/action confirmation policy |
| ADR-0056 | Router-level `X-Макошь-Secret` local API auth |
| ADR-0074 | Multi-channel identity traces including WhatsApp/phone |
| ADR-0076 | Host vault for new secret payloads |
| ADR-0085 | Communication spine and Polygraph integration |
| ADR-0091 | Provider channel capability model pattern |
| ADR-0093 | Vue 3 frontend |
| ADR-0097 | Channels are integrations; Communications owns domain state |

ADR-0051 is the controlling WhatsApp-specific decision. It keeps WhatsApp Web
as the first provider boundary and keeps Meta Business API as a separate future
provider shape.

## Provider Source Model

Primary source:

```text
WhatsApp Web
```

Initial product posture:

```text
Local First
Desktop First
Personal Use
```

Rules:

- WhatsApp Web is a linked-device companion experience.
- Live runtime requires visible owner setup, session lifecycle, local storage
  policy and smoke validation before any live capability becomes `available`.
- Session secrets, pairing material and local browser profile secrets must not
  be stored in PostgreSQL.
- Raw provider records are append-only.
- Provider quirks stay inside the adapter/source-record boundary.
- Meta Business API is not a fallback for personal WhatsApp Web.

Future source:

```text
whatsapp_business_cloud
```

That provider must use its own account, capability, auth, command and source
record contracts.

## Backend Layers

Target layers:

| Layer | Target module | Назначение |
|---|---|---|
| API routes | `backend/src/integrations/whatsapp/api/` | Account, capability, session, dialog, message, status, media and command endpoints |
| Companion runtime | `backend/src/integrations/whatsapp/runtime/` | Account-scoped WebView companion orchestration and runtime event bridge |
| Adapter/client | `backend/src/integrations/whatsapp/client/` | Provider record normalization, validation, queries and projection helpers |
| Source records | Communications compatibility boundary | Append-only raw WhatsApp provider records |
| Projection | Communications compatibility boundary | Canonical `communication_messages`, conversations, participants and attachments |
| Media storage | Communication blob/attachment boundary | Local blob storage, metadata, hashes and scanner state |
| Outbox | WhatsApp provider-write command store | Durable command lifecycle, retry and reconciliation |
| Realtime | Shared event bus | Sanitized `whatsapp.*` event emission |
| Audit | Platform audit boundary | Redacted provider-write, lifecycle and capability audit records |

Implementation status is intentionally not counted in this package. This is a
target architecture and starting audit.

## Frontend Layers

Target layers:

| Layer | Target module | Назначение |
|---|---|---|
| Runtime/setup panels | `frontend/src/integrations/whatsapp/views/` | WhatsApp integration panels embedded in Communications |
| API client | `frontend/src/integrations/whatsapp/api/` | Typed calls to protected backend routes |
| Query hooks | `frontend/src/integrations/whatsapp/queries/` | TanStack Query integration |
| Store/helpers | `frontend/src/integrations/whatsapp/stores/` | Local UI state, selected account/dialog, filters and command status |
| Components | `frontend/src/integrations/whatsapp/components/` | Dialog list, thread, composer, media/status/call panels and inspector |
| Realtime patches | shared platform bootstrap | Cache patching from `whatsapp.*` events |

WhatsApp panels must consume the shared frontend realtime bootstrap. They must
not open a channel-specific event transport.

## Runtime Kinds

Target runtime map:

| Runtime | Назначение | Initial state |
|---|---|---|
| Fixture/manual runtime | Deterministic local validation and docs-driven smoke data | planned |
| WebView companion runtime | Owner-linked WhatsApp Web session with health-exposed bridge contract | blocked until active desktop producer smoke |
| Offline command runtime | Durable local outbox and replay | planned |
| Media transfer runtime | Download/upload orchestration through companion runtime | blocked |
| Meta Business Cloud runtime | Future official business provider | unsupported |

Live WebView runtime remains `blocked` until session lifecycle, desktop
visibility, local storage, secret resolution, audit and smoke validation exist.

## Account Boundary

Supported account kinds:

```text
whatsapp_personal
whatsapp_business
```

Provider kind for the first architecture:

```text
whatsapp_web
```

Account metadata may include:

- account_id;
- account kind;
- provider kind;
- display label;
- lifecycle state;
- non-secret runtime metadata;
- local session reference;
- capability snapshot;
- audit links.

Account metadata must not include:

- pairing codes;
- session keys;
- browser cookies;
- local profile secrets;
- message bodies;
- media bytes;
- contact book exports.

Credential lookup uses:

```text
account_id + secret_purpose
```

Provider kind alone must never select credentials.

## Capability Boundary

Backend capability state is required before a WhatsApp operation appears in UI.

States:

```text
available
degraded
blocked
unsupported
```

Required operations:

| Operation | Action class | Initial state |
|---|---|---|
| send | provider_write | blocked |
| reply | provider_write | blocked |
| forward | provider_write | blocked |
| reaction | provider_write | blocked |
| delete | destructive | blocked |
| media upload | provider_write | blocked |
| media download | read/local_write | blocked |
| join group | provider_write | blocked |
| leave group | provider_write/destructive | blocked |
| status read | read | blocked |
| status publish | provider_write | blocked |
| voice send | provider_write | blocked |

`unsupported` is reserved for operations that conflict with Макошь policy or
are intentionally outside the current provider shape. `blocked` means
architecturally allowed but missing required runtime, permission, validation,
secret or adapter support.

## Dialog / Chat Model

Supported dialog classes:

```text
private chat
group
community
broadcast
status
```

Dialog records are account-scoped provider projections. They may expose:

- provider_chat_id;
- dialog kind;
- title or display name;
- participant count when observed;
- last activity timestamp;
- unread/mention counters when observed;
- mute/archive/pin local overlay when implemented;
- provider permissions and admin/member evidence when observed.

Dialog projections must not own Persona, Organization, Project, Memory,
Decision or Obligation lifecycle.

## Message Lifecycle

Target message lifecycle:

```text
provider message created
  -> raw event
  -> projected message
  -> whatsapp.message.created

provider message updated
  -> raw update evidence
  -> observed version metadata
  -> whatsapp.message.updated

provider/local delete observed
  -> raw delete evidence
  -> tombstone row
  -> whatsapp.message.deleted
```

Supported message classes:

- text;
- reply;
- forward;
- reaction;
- delete;
- edit, if supported by the provider/runtime.

Message identity needs:

- account_id;
- provider_chat_id;
- provider_message_id;
- provider_sender_id;
- message timestamp;
- raw_record_id;
- communication_message_id;
- optional reply target;
- optional forward source;
- optional edit version;
- optional tombstone state.

Макошь must not claim provider edit history that was never observed locally.

## Provider Command Outbox

WhatsApp provider writes must use the provider-write command model.

UI command path:

```text
UI intent
  -> protected backend command route
  -> capability decision
  -> provider command outbox
  -> WebView companion adapter
  -> provider-observed state
  -> Communication projection refresh
  -> whatsapp.command.status_changed / whatsapp.command.reconciled
```

Required command states:

```text
queued
executing
retrying
completed
failed
dead_letter
```

Rules:

- UI must not call WebView provider primitives directly.
- `completed` is reserved for provider-observed state, not merely a local ACK.
- Command rows must carry idempotency keys, account scope, provider target,
  capability decision, confirmation decision, retry state and redacted audit
  metadata.
- Message bodies, media bytes, secrets and raw provider payloads must not be
  stored in audit records.

## Replies, Forwards, Reactions

WhatsApp requires first-class projection for:

- reply target;
- reply context;
- forward attribution when provider exposes it;
- reaction state;
- delete/tombstone state;
- edit/update state when provider exposes it.

Raw WebView/provider data remains evidence, but UI and queries should use
projection contracts rather than parsing raw payloads directly.

## Media Lifecycle

Target lifecycle:

```text
WhatsApp media metadata
  -> media projection
  -> optional download command
  -> local blob storage
  -> scanner backend
  -> preview artifact
  -> media gallery/search index
  -> timeline attachment evidence
```

Supported media classes:

- photo;
- video;
- document;
- audio;
- voice note;
- contact;
- location;
- sticker;
- gif.

Media bytes stay out of PostgreSQL. Attachment metadata must pass through the
shared attachment safety scanner boundary. A no-op
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/whatsapp/blockers.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/blockers.md`
- Size bytes / Размер в байтах: `10887`
- Included characters / Включено символов: `10311`
- Truncated / Обрезано: `no`

```markdown
# WhatsApp Architectural Blockers

Статус: стартовые audit blockers на 2026-06-17.

Блокеры ниже фиксируют причины, последствия и план решения. Они не являются
разрешением на реализацию новых крупных подсистем вне WhatsApp Channel.

Актуализация на 2026-06-27: WhatsApp closure is currently **blocked**, not
complete. The audit gate must continue to report `closure_achieved = false`
until sanitized live-smoke evidence exists for `whatsapp_native_md`,
`whatsapp_web_companion` and `whatsapp_business_cloud`, and until Native MD has
safe provider API coverage plus smoke evidence for `archive`, `unarchive`,
`mute`, `unmute`, `pin`, `unpin`, `mark_unread`, `join_group` and
`publish_status`. Do not mark the provider domain closed before
`make whatsapp-domain-closure-gate` passes.

## 1. WebView Companion Boundary

**Причина**: ADR-0051 permits WhatsApp Web only through an explicit companion
boundary. The product request uses `Hidden WebView`, but ADR-0051 rejects hidden
headless scraping and requires owner-visible desktop/runtime controls.

**Последствия**: A silent WebView runtime would violate the provider boundary
and create privacy, session and terms-of-use risk.

**План решения**:

- define Hidden WebView as desktop-owned companion runtime, not headless
  scraping;
- expose link, session, revocation and runtime status to the owner;
- keep live runtime `blocked` until smoke validation exists;
- log only redacted lifecycle/audit metadata;
- do not store session secrets in PostgreSQL.

## 2. No Official Personal WhatsApp API

**Причина**: Personal WhatsApp support has no stable documented API equivalent
to email IMAP or a first-party personal messaging API. WhatsApp Web is a
linked-device companion surface.

**Последствия**: Provider behavior may change, live runtime may degrade, and
tests cannot depend on live personal accounts.

**План решения**:

- preserve fixture/manual validation path;
- isolate provider quirks inside the adapter;
- report runtime fragility through capability states;
- avoid hard dependencies on unverified selectors or private protocols in
  canonical architecture;
- keep Meta Business API as a separate future provider.

## 3. Account And Session Lifecycle Missing

**Причина**: WhatsApp needs account-scoped session state, local WebView profile
storage, lifecycle transitions and revocation semantics before live runtime can
be safe.

**Последствия**: Sync, provider writes and media transfer cannot be reliably
started, stopped, audited or recovered.

**План решения**:

- define `whatsapp_personal` and `whatsapp_business` account metadata;
- define lifecycle states;
- use `account_id + secret_purpose` for session protection material;
- store local WebView state only under ignored local data paths;
- preserve local evidence on logout/remove unless explicit destructive purge is
  separately implemented.

## 4. Capability Contract Granularity

**Причина**: WhatsApp operations need the same operation-level capability model
as other provider channels before UI exposure.

**Последствия**: UI cannot reliably distinguish available, degraded, blocked and
unsupported operations for send, reply, forward, reaction, delete, media,
groups, status and voice.

**План решения**:

- add backend capability contract;
- model action class, scope, reason and confirmation requirement;
- start live provider writes as `blocked`;
- mark calls control, recording and STT as `unsupported`;
- add fixture tests before enabling UI controls.

## 5. Provider-Write Command Outbox Missing

**Причина**: WhatsApp sends, replies, forwards, reactions, deletes, media
uploads, group joins/leaves, status publishes and voice sends are provider-side
effects.

**Последствия**: Direct UI-to-provider calls would bypass capability policy,
audit, retry, idempotency and reconciliation.

**План решения**:

- route every provider write through backend command APIs;
- persist command states `queued`, `executing`, `retrying`, `completed`,
  `failed`, `dead_letter`;
- require provider-observed evidence before `completed`;
- emit `whatsapp.command.status_changed` and
  `whatsapp.command.reconciled`;
- keep audit metadata redacted.

## 6. Raw Evidence And Projection Schema Missing

**Причина**: WhatsApp needs append-only raw records and canonical Communication
projections for messages, statuses, media, participants and calls.

**Последствия**: UI or AI would be tempted to work from runtime snapshots
without replayable source evidence.

**План решения**:

- define raw record kinds such as `whatsapp_web_message`,
  `whatsapp_web_status`, `whatsapp_web_media` and `whatsapp_web_call`;
- keep raw records append-only;
- project into canonical Communications;
- preserve provider identifiers and observed timestamps;
- avoid storing media bytes or secrets in raw/audit/event payloads.

## 7. Phone-Centric Identity Resolution

**Причина**: WhatsApp identity is centered on phone numbers, display names and
provider-specific `wa_id` values, not email addresses or stable global person
IDs.

**Последствия**: Incorrect merges could corrupt Persona/Relationship state if
phone traces are treated as truth.

**План решения**:

- store phone numbers, `wa_id` and display names as identity traces;
- preserve source and confidence metadata;
- emit Persona/Relationship candidates only;
- keep merge/split lifecycle outside WhatsApp;
- track display-name history as observed evidence.

## 8. Dialog And Status Model Missing

**Причина**: WhatsApp has private chats, groups, communities, broadcasts and
statuses. Statuses are not equivalent to ordinary chats, but they still produce
Communication evidence.

**Последствия**: Treating Status as a separate domain or ignoring it would break
Timeline, identity-signal and source-evidence consistency.

**План решения**:

- model status as provider evidence under WhatsApp Channel;
- produce Timeline evidence and identity signals;
- keep status publish as provider-write command;
- do not create a Status domain;
- keep broadcast/community distinctions at provider projection boundary.

## 9. Media And Attachment Safety Missing

**Причина**: WhatsApp media includes photos, videos, documents, audio, voice
notes, contacts, locations, stickers and GIFs. Media bytes are untrusted input.

**Последствия**: Unsafe preview, PostgreSQL blob storage or missing scanner
state would violate local-first storage and attachment safety rules.

**План решения**:

- store bytes in local blob storage, not PostgreSQL;
- store metadata, hashes, scanner state and local refs in database records;
- default scanner state to `not_scanned`;
- never mark attachments `clean` without scanner backend;
- use explicit download commands and progress events.

## 10. Realtime Contracts Missing

**Причина**: WhatsApp requires typed events for messages, chats, reactions, media
downloads and command reconciliation.

**Последствия**: Frontend would fall back to broad reloads and could miss
source-backed state changes.

**План решения**:

- define sanitized `whatsapp.*` event payloads;
- emit events at projection and command boundaries;
- include stable identifiers for cache patching;
- exclude message bodies, media bytes, raw payloads and secrets;
- consume events through the shared realtime bootstrap.

## 11. Calls Boundary Requires Future ADR

**Причина**: Calls touch microphone, camera, speakers, live media devices,
recording and potentially STT. The first version permits metadata only.

**Последствия**: Implementing call control, capture or transcription now would
violate the no-hidden-recording boundary.

**План решения**:

- support only call metadata, call evidence and Timeline entries;
- mark audio/video capture, call control, recording, live handling and STT
  `unsupported`;
- require future ADR before any live call runtime;
- keep UI explicit that no recording occurs.

## 12. Voice Notes Need Attachment-First Design

**Причина**: Voice notes are media attachments, but they are often treated like
messages or transcripts in product surfaces.

**Последствия**: Early STT or hidden recording could bypass permission,
provenance and source-evidence boundaries.

**План решения**:

- model voice notes as metadata plus attachment;
- support local playback only after download;
- leave transcript integration as future shared-engine handoff;
- require explicit permission before any recording or STT work;
- keep transcript output source-backed and reviewable.

## 13. Meta Business API Provider Confusion

**Причина**: Meta Business Platform Cloud API is official for business messaging
but has different onboarding, auth, account, template and policy semantics than
personal WhatsApp Web.

**Последствия**: Using it as the primary architecture would break the personal
local-first WhatsApp Web boundary and mix incompatible provider assumptions.

**План решения**:

- keep WhatsApp Web as primary provider source;
- document Meta Business API as future provider only;
- use distinct provider kind such as `whatsapp_business_cloud`;
- create separate ADR before implementation;
- do not reuse personal WebView commands for Cloud API semantics.

## 14. Test And Validation Path Missing

**Причина**: Live WhatsApp Web validation cannot be a default CI dependency.
Fixture/manual state is required before any live companion runtime can be
validated safely.

**Последствия**: Without deterministic tests, provider changes and session
fragility would create hidden regressions.

**План решения**:

- create fixture/manual records first;
- test idempotent raw record ingestion;
- test capability states;
- test outbox state transitions;
- keep live WebView smoke tests opt-in;
- never require live WhatsApp credentials in CI.

## 15. Cross-Domain Temptation

**Причина**: WhatsApp naturally exposes people, groups, obligations, decisions,
tasks, organizations, projects, locations and documents.

**Последствия**: Макошь would get channel-specific mini-domains and duplicated
business logic if WhatsApp implements those lifecycles directly.

**План решения**:

- WhatsApp may emit evidence and candidates only;
- lifecycle belongs to shared domains/engines;
- do not implement Memory, Knowledge, Persona, Organization, Project, Decision,
  Obligation or Task logic inside WhatsApp Channel;
- preserve source references for every candidate.
```

### `docs/integrations/whatsapp/current-audit-2026-06-24.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/current-audit-2026-06-24.md`
- Size bytes / Размер в байтах: `54158`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# WhatsApp Current Audit — 2026-06-24

Status: current repository audit based on `makosh-os-main (3).zip`.

This document records what exists now before starting the full WhatsApp target implementation. It intentionally separates:

- implemented fixture/runtime foundation;
- provider-neutral Communications foundation already present;
- missing production WhatsApp functionality;
- architectural risks that must not be bypassed.

## Executive summary

WhatsApp is already present as a **provider/runtime foundation**, not as a full channel implementation.

The current repository contains:

- WhatsApp documentation under `docs/integrations/whatsapp/`;
- ADR-0051 for the WhatsApp Web companion boundary;
- ADR-0097 for channel-to-integration ownership;
- backend integration skeleton under `backend/src/integrations/whatsapp/`;
- runtime/setup routes under `/api/v1/integrations/whatsapp/*`;
- provider-neutral communication read route support through `/api/v1/communications/messages`;
- PostgreSQL migrations for `whatsapp_web` provider kind, session state, observation kind and canonical Communication tables;
- frontend runtime/setup panel under `frontend/src/integrations/whatsapp/`;
- tests proving fixture ingestion can produce raw signal events, accepted signal events, canonical `communication.message.recorded` traces and downstream candidate extraction.

The repository does **not** yet contain live WhatsApp sync, live QR/pair-code login, native protocol runtime, WebView runtime, live provider-write execution, live media transfer, live reactions, live statuses, live group/community lifecycle, live identity trace handoff, live provider event bridge from a real runtime into accepted WhatsApp observation events, or business cloud API support.

## Current architecture posture

Current target posture is still correct:

```text
WhatsApp provider/runtime
  -> Integration boundary
  -> Provider observation / signal
  -> Communications projection
  -> Timeline / Search / Radar / Review / shared engines
  -> target domains through workflows only
```

WhatsApp must not become:

- a backend domain;
- a frontend product domain;
- a Persona implementation;
- a Task implementation;
- a Knowledge implementation;
- an AI source of truth;
- a hidden automation/scraping daemon.

## Existing repository assets

### Documentation

| File | Current role |
|---|---|
| `docs/integrations/whatsapp/README.md` | Channel framing and principles. |
| `docs/integrations/whatsapp/architecture.md` | Target architecture and trace flow. |
| `docs/integrations/whatsapp/api.md` | Runtime/setup API foundation. |
| `docs/integrations/whatsapp/modules.md` | Target module inventory. |
| `docs/integrations/whatsapp/status.md` | Earlier zero-based production audit. |
| `docs/integrations/whatsapp/gap-analysis.md` | Missing capability matrix. |
| `docs/integrations/whatsapp/blockers.md` | Architectural blockers. |
| `docs/adr/ADR-0051-v5-whatsapp-web-companion-boundary.md` | WhatsApp Web companion boundary. |
| `docs/adr/ADR-0097-communications-channel-domains-to-integrations.md` | Channels are integrations, Communications owns domain state. |

### Backend source

| Path | Current role |
|---|---|
| `backend/src/integrations/whatsapp/mod.rs` | WhatsApp integration module root. |
| `backend/src/integrations/whatsapp/client.rs` | Public re-exports for the current client/store foundation. |
| `backend/src/integrations/whatsapp/client/models.rs` | Fixture account, session and message DTOs. |
| `backend/src/integrations/whatsapp/client/store.rs` | Store constructor and dependencies. |
| `backend/src/integrations/whatsapp/client/store/accounts.rs` | Fixture account setup. |
| `backend/src/integrations/whatsapp/client/store/sessions.rs` | Session upsert/list and session observations. |
| `backend/src/integrations/whatsapp/client/store/ingestion.rs` | Fixture message ingestion and projection path. |
| `backend/src/integrations/whatsapp/client/store/queries.rs` | Recent projected WhatsApp message query. |
| `backend/src/app/provider_runtime_handlers/whatsapp.rs` | Runtime/setup handlers. |
| `backend/src/app/api_support/messaging_integrations.rs` | WhatsApp capability response and DTO wrappers. |
| `backend/src/app/router/routes/messaging.rs` | Integration routes for WhatsApp fixture/setup/session operations. |

### Frontend source

| Path | Current role |
|---|---|
| `frontend/src/integrations/whatsapp/api/whatsapp.ts` | Typed runtime/setup client. |
| `frontend/src/integrations/whatsapp/queries/useWhatsappQuery.ts` | Runtime/session query hooks. |
| `frontend/src/integrations/whatsapp/stores/whatsapp.ts` | Integration runtime UI state. |
| `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue` | Fixture/manual runtime panel. |
| `frontend/src/integrations/whatsapp/components/*` | Runtime rail, session list and status messages. |
| `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue` | Communications-facing provider surface with projected conversation/message/member/media views plus provider-neutral send/reply/forward/edit/delete, conversation read/unread, pin/unpin, archive/unarchive, mute/unmute and reaction actions. |
| `frontend/src/domains/communications/api/whatsappBusinessApi.ts` | Provider-neutral WhatsApp Communications helper for projected reads and message command routes. |

### Migrations

| Migration | Current role |
|---|---|
| `0021_create_v5_whatsapp_web_foundation.sql` | Adds `whatsapp_web`, `whatsapp_web_session_key`, `whatsapp_web_sessions`. |
| `0125_add_whatsapp_session_observation_kind.sql` | Adds `WHATSAPP_WEB_SESSION` observation kind. |
| `0149_create_canonical_communication_tables.sql` | Adds canonical Communication table family. |
| `0157_create_whatsapp_provider_write_commands.sql` | Adds WhatsApp provider-write command compatibility outbox and mirrors rows into canonical `communication_provider_commands`. |

## Existing API surface

Implemented foundation routes:

| Method | Path | Status |
|---|---|---|
| `GET` | `/api/v1/integrations/whatsapp/capabilities` | Implemented foundation. |
| `GET` | `/api/v1/integrations/whatsapp/accounts` | Implemented WhatsApp account list surface with optional removed-account inclusion. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/accounts` | Implemented fixture-only setup. |
| `GET` | `/api/v1/integrations/whatsapp/sessions` | Implemented session list. |
| `GET` | `/api/v1/communications/messages?channel_kind=whatsapp` | Implemented provider-neutral WhatsApp message list; accountless reads now aggregate across provider shapes through Communications projections. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/messages` | Implemented fixture-only message ingest. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/message-updates` | Implemented fixture-only message update ingest and canonical version projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/message-deletes` | Implemented fixture-only message delete ingest and canonical tombstone projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/receipts` | Implemented fixture-only receipt ingest and canonical delivery-state projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/dialogs` | Implemented fixture-only dialog ingest and canonical conversation projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/participants` | Implemented fixture-only participant ingest and canonical identity/participant projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/reactions` | Implemented fixture-only reaction ingest and canonical projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/media` | Implemented fixture-only media metadata ingest and attachment projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/statuses` | Implemented fixture-only status evidence ingest and Communication projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/status-views` | Implemented fixture-only status-view evidence ingest and status metadata projection. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/status-deletes` | Implemented fixture-only status-delete evidence ingest, status tombstone projection and realtime event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/presence` | Implemented fixture-only presence evidence ingest, identity-metadata patching and realtime event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/calls` | Implemented fixture-only call-metadata evidence ingest, calls read-model upsert and realtime event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/runtime-events` | Implemented fixture-only runtime-event evidence ingest, accepted Signal Hub event emission and sanitized realtime runtime-event emission. |
| `POST` | `/api/v1/integrations/whatsapp/fixtures/sessions/authorized` | Implemented fixture auth-complete vault persistence. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/status` | Implemented blocked-safe runtime status. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/start` | Implemented blocked-safe runtime start. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/stop` | Implemented blocked-safe runtime stop. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/revoke` | Implemented blocked-safe session revoke surface. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/relink` | Implemented blocked-safe relink-preparation surface. |
| `POST` | `/api/v1/integrations/whatsapp/runtime/remove` | Implemented blocked-safe remove surface with session secret cleanup. |
| `GET` | `/api/v1/integrations/whatsapp/runtime/health` | Implemented blocked-safe runtime health. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/chats` | Implemented fixture-backed provider sync surface for projected WhatsApp conversations plus sanitized sync lifecycle events. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/history` | Implemented fixture-backed provider sync surface for projected WhatsApp conversation history plus sanitized sync lifecycle events. |
| `POST` | `/api/v1/integrations/whatsapp/provider-sync/media` | Implemented fixture-backed provider sync surface for projected WhatsApp attachments plus sanitized sync lifecycle events. |
| `POST` | `/api/v1/integrations/whatsapp/login/qr/start` | Implemented blocked-safe QR login surface. |
| `POST` | `/api/v1/integrations/whatsapp/login/pair-code/start` | Implemented blocked-safe pair-code login surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/send` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reply` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/forward` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/edit` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/delete` | Implemented blocked-safe provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Implemented blocked-safe add-reaction provider command surface. |
| `DELETE` | `/api/v1/integrations/whatsapp/provider-commands/messages/{message_id}/reactions` | Implemented blocked-safe remove-reaction provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/messages/voice-note` | Implemented blocked-safe voice-note provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/statuses/publish` | Implemented blocked-safe status publish provider command surface. |
| `POST` | `/api/v1/integrations/whatsapp/provider-commands/conversations/join` | Implemented blocked-safe group join provider command surface. |
| `POST` | `/ap
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/whatsapp/fixture-test-matrix.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/fixture-test-matrix.md`
- Size bytes / Размер в байтах: `7472`
- Included characters / Включено символов: `7472`
- Truncated / Обрезано: `no`

````markdown
# WhatsApp Fixture Test Matrix

Status: current fixture/runtime-safe coverage map.
Date: 2026-06-26.

This matrix tracks the current repository evidence for acceptance criterion 11:

```text
Fixture tests cover every source record and command class.
```

It is intentionally scoped to fixture/runtime-safe coverage. Live provider smoke
validation remains separately documented in
[`live-smoke-checklist.md`](./live-smoke-checklist.md).

Fast static guard coverage:

- `backend/tests/whatsapp_signal_hub.rs` checks that every documented source
  event family has a Signal Hub raw/accepted mapping, a sanitized realtime
  event family and matrix coverage.
- The same test verifies that command completion remains tied to
  provider-observed reconciliation metadata, not provider SDK success alone.
- `backend/tests/communications_architecture_target.rs` guards provider-library
  import confinement and rejects direct WhatsApp runtime dependencies from
  domains, engines and workflows.

## Source record kinds

| Source record kind | Current evidence |
|---|---|
| `whatsapp_message` | `whatsapp_fixture_message_ingestion_refreshes_decision_and_obligation_candidates_against_postgres` |
| `whatsapp_message_update` | `whatsapp_fixture_message_update_reconciles_provider_command_via_observed_event` |
| `whatsapp_message_delete` | `whatsapp_fixture_message_delete_reconciles_provider_command_via_observed_event` |
| `whatsapp_reaction` | `whatsapp_fixture_reaction_reconciles_provider_command_via_observed_event`, `whatsapp_fixture_unreact_reconciles_provider_command_via_observed_event` |
| `whatsapp_receipt` | `whatsapp_fixture_receipt_projects_source_record_and_emits_realtime_event` |
| `whatsapp_dialog` | `whatsapp_fixture_dialog_reconciles_archive_command_via_observed_event`, `whatsapp_fixture_dialog_reconciles_mute_and_mark_unread_commands_via_observed_event`, `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `whatsapp_participant` | `whatsapp_fixture_participant_reconciles_join_and_leave_group_commands_via_observed_event` |
| `whatsapp_media` | `whatsapp_fixture_media_reconciles_send_media_command_via_observed_event`, `whatsapp_fixture_media_reconciles_download_media_command_via_observed_event`, `whatsapp_fixture_media_reconciles_send_voice_note_command_via_observed_event` |
| `whatsapp_status` | `whatsapp_fixture_status_reconciles_publish_status_command_via_observed_event` |
| `whatsapp_status_view` | `whatsapp_fixture_status_view_and_delete_project_source_records_and_emit_realtime_events` |
| `whatsapp_status_delete` | `whatsapp_fixture_status_view_and_delete_project_source_records_and_emit_realtime_events` |
| `whatsapp_presence` | `whatsapp_fixture_presence_projects_source_record_and_emits_realtime_event` |
| `whatsapp_call_metadata` | `whatsapp_fixture_call_projects_source_record_and_emits_realtime_event` |
| `whatsapp_runtime_event` | `whatsapp_fixture_runtime_event_is_captured_as_signal_and_sanitized_realtime_event`, `whatsapp_unknown_runtime_event_defaults_to_degraded_warning_markers` |
| `whatsapp_web_session` | `whatsapp_web_session_metadata_is_account_scoped_against_postgres`, `whatsapp_authorized_session_material_is_stored_in_host_vault_against_postgres` |

## Provider command classes

### Provider-observed reconciliation coverage

| Command class | Current evidence |
|---|---|
| `send_text` | `whatsapp_fixture_message_reconciles_send_text_command_via_observed_event` |
| `reply` | reply branch inside `whatsapp_api_exercises_web_fixture_foundation` |
| `forward` | forward branch inside `whatsapp_api_exercises_web_fixture_foundation` |
| `edit` | `whatsapp_fixture_message_update_reconciles_provider_command_via_observed_event` |
| `delete` | `whatsapp_fixture_message_delete_reconciles_provider_command_via_observed_event` |
| `react` | `whatsapp_fixture_reaction_reconciles_provider_command_via_observed_event` |
| `unreact` | `whatsapp_fixture_unreact_reconciles_provider_command_via_observed_event` |
| `send_media` | `whatsapp_fixture_media_reconciles_send_media_command_via_observed_event` |
| `download_media` | `whatsapp_fixture_media_reconciles_download_media_command_via_observed_event` |
| `send_voice_note` | `whatsapp_fixture_media_reconciles_send_voice_note_command_via_observed_event` |
| `archive` | `whatsapp_fixture_dialog_reconciles_archive_command_via_observed_event` |
| `mute` | `whatsapp_fixture_dialog_reconciles_mute_and_mark_unread_commands_via_observed_event` |
| `mark_unread` | `whatsapp_fixture_dialog_reconciles_mute_and_mark_unread_commands_via_observed_event` |
| `unarchive` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `unpin` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `unmute` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `mark_read` | `whatsapp_fixture_dialog_reconciles_unarchive_unpin_unmute_and_mark_read_commands_via_observed_event` |
| `join_group` | `whatsapp_fixture_participant_reconciles_join_and_leave_group_commands_via_observed_event` |
| `leave_group` | `whatsapp_fixture_participant_reconciles_join_and_leave_group_commands_via_observed_event` |
| `publish_status` | `whatsapp_fixture_status_reconciles_publish_status_command_via_observed_event` |

### Retried background executor coverage

| Command class | Current evidence |
|---|---|
| `send_text` | `whatsapp_background_command_executor_completes_retried_fixture_send_text_command` |
| `reply` | `whatsapp_background_command_executor_completes_retried_fixture_reply_command` |
| `forward` | `whatsapp_background_command_executor_completes_retried_fixture_forward_command` |
| `edit` | `whatsapp_background_command_executor_completes_retried_fixture_edit_command` |
| `delete` | `whatsapp_background_command_executor_completes_retried_fixture_delete_command` |
| `react` | `whatsapp_background_command_executor_completes_retried_fixture_react_command` |
| `unreact` | `whatsapp_background_command_executor_completes_retried_fixture_unreact_command` |
| `send_media` | `whatsapp_background_command_executor_completes_retried_fixture_send_media_command` |
| `download_media` | `whatsapp_background_command_executor_completes_retried_fixture_download_media_command` |
| `send_voice_note` | `whatsapp_background_command_executor_completes_retried_fixture_send_voice_note_command` |
| `archive` / `mute` / `pin` / `mark_unread` | `whatsapp_background_command_executor_completes_retried_fixture_dialog_state_commands` |
| `unarchive` / `unmute` / `unpin` / `mark_read` | `whatsapp_background_command_executor_completes_retried_fixture_inverse_dialog_state_commands` |
| `join_group` / `leave_group` | `whatsapp_background_command_executor_completes_retried_fixture_join_and_leave_group_commands` |
| `publish_status` | `whatsapp_background_command_executor_completes_retried_fixture_publish_status_command` |

## Known gaps

- This matrix tracks fixture/runtime-safe coverage only. It does not prove live
  provider execution.
- Business Cloud runtime behavior remains capability-model and account-shape
  coverage, not official live API execution coverage.
- Exact DB-backed execution of the listed tests may still depend on local
  Docker/Testcontainers health; CI-safe compile coverage remains `--no-run`
  unless a narrower environment-safe test path is added later.
````
