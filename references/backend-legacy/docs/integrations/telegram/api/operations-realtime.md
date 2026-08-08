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
also appended into canonical `event_log`, so they are available through:

- `GET /api/events/ws?after_position=&makosh_secret=`;
- `GET /api/events/stream?after_position=`;
- `GET /api/v1/events?after_position=&limit=&wait_seconds=`.

Extended parity deferred outside base closure:

- richer provider-observed folder state/mutations beyond the current folder-label projection refresh, folder fallback refresh, and main/archive pin-state observation;
- true message-level read receipt records beyond projected chat counters;
- custom mute-shape reconciliation beyond the current exact TDLib request contract;
- richer Telegram media retry/dead-letter UX beyond request/progress/failed/completed events;
- fully patched provider-dialog cache surfaces without fallback invalidation.

## Frontend API Client

Current frontend client:

```text
frontend/src/integrations/telegram/api/telegram.ts
frontend/src/integrations/telegram/queries/useTelegramQuery.ts
```

Covered:

- capabilities;
- accounts;
- chats;
- messages;
- runtime;
- QR login;
- send;
- media download;
- dry-run;
- calls.

Not covered because backend routes do not exist:

- topic/provider-command parity beyond current lifecycle, reaction, pin, archive, mute, read/unread and search execution paths;
- media gallery;
- richer provider-backed folder parity beyond the current add/remove/reassign delta model and label sync parity;
- richer Telegram realtime cache patching beyond the current chat/message/topic/command/media/member surfaces.
