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
  -> TDLib downloadFile
  -> local blob write
  -> communication_attachments row
  -> scan status from attachment scanner boundary
```

Целевой lifecycle:

```text
Telegram media metadata
  -> media projection
  -> optional download command
  -> local blob storage
  -> scanner backend
  -> preview artifact
  -> media gallery/search index
  -> timeline attachment evidence
```

Media types:

- photo;
- video;
- document;
- voice note;
- video note;
- audio;
- sticker;
- GIF/animation;
- media album;
- contact/location/poll metadata.

## Attachment Boundary

Telegram attachments reuse Communication attachment metadata and local blob storage.

Current compatibility issue:

```text
MailStorageStore
communication_mail_blobs
communication_attachment_imports
```

This is an implementation compatibility label. Target architecture should expose
provider-neutral facade:

```text
CommunicationBlobStore
communication_blobs
communication_attachments
```

No table rename is required in documentation/audit phase.

## Search Architecture

Search layers:

1. local loaded chat filter;
2. local loaded message filter;
3. shared Communication full-text search;
4. provider-side Telegram search;
5. media search;
6. dialog/member/topic search.

Current implementation has projection-backed local, shared Communication,
provider, media, dialog, member and topic search paths for the base Telegram
domain. Richer provider media filters and scheduled provider-side saved-search
execution are future search expansion, not base channel ownership.

## Realtime Architecture

Generic transports already exist:

```text
WebSocket
SSE
Long Poll
Replay
Heartbeat
```

Telegram needs typed event contracts:

```text
telegram.sync.started
telegram.sync.progress
telegram.sync.completed
telegram.sync.failed

telegram.message.created
telegram.message.updated
telegram.message.deleted
telegram.message.tombstoned
telegram.message.visibility_restored

telegram.reaction.changed
telegram.chat.updated
telegram.chat.pinned
telegram.chat.archived
telegram.chat.muted
telegram.topic.updated
telegram.media.downloaded
telegram.media.upload.started
telegram.media.upload.completed
telegram.media.upload.failed
telegram.command.status_changed
```

Current topic realtime implementation covers provider-observed
`updateForumTopicInfo` updates from TDLib. The runtime bridge resolves the
projected chat, upserts the existing topic projection, emits sanitized
`telegram.topic.updated` and lets the frontend patch topic list/search caches
before replay invalidation. This is not a topic write-command model.

Current provider-observed unread implementation covers TDLib
`updateChatReadInbox` and `updateChatUnreadMentionCount`. The runtime bridge
resolves the projected chat, updates chat metadata counters, emits sanitized
`telegram.chat.updated` with a projected chat snapshot and lets the frontend
reuse the existing chat list/detail patch path. This is not full message-level
read receipt history.

Events must never include:

- message body;
- media bytes;
- tokens;
- passwords;
- app secrets;
- raw provider payload;
- rendered automation variables.

Frontend should patch TanStack Query caches before invalidation, following the
Mail pattern.

## Provider Write Command Model

Manual text send currently exists. Target command model must support:

- send text;
- send media;
- edit;
- delete;
- restore local visibility;
- react/unreact;
- pin/unpin;
- mark read/unread;
- mute/unmute;
- archive/unarchive;
- join/leave;
- admin actions if ever scoped.

Provider participant state is projection data owned by Telegram as a
Communication Channel. `telegram_chat_participants` stores TDLib-observed
member roster evidence for supergroups/channels, including provider member id,
role/status/admin/owner state, permissions and raw TDLib payload metadata. It
does not create Persona, Organization, Memory, Knowledge, Obligation or
Decision records. Message-sender aggregation is allowed only as an explicit
read fallback (`source=message_heuristic`) when provider roster rows do not
exist for a chat.

Join/leave are provider-write commands, not direct projection mutations:

```text
POST /telegram/chats/join or /telegram/chats/{id}/leave
  -> telegram_provider_write_commands(command_kind=join|leave)
  -> TDLib joinChat/leaveChat dispatch by active actor
  -> awaiting_provider after TDLib ACK
  -> join completed only when TDLib roster sync observes active self membership
  -> join/leave may also complete when TDLib history sync ingests explicit
     self-targeted participant service-message evidence
  -> silent/admin membership changes still await stronger provider evidence
```

Each command should have:

- command_id;
- account_id;
- provider target;
- idempotency key;
- capability decision;
- user confirmation decision;
- audit metadata;
- retry/degraded state;
- per-target result rows;
- sanitized realtime events.

Runtime manager methods that need stores, secret resolution, config and runtime
event bridge state should accept the shared Telegram runtime context structs
instead of long repeated argument lists:

- `TelegramRuntimeStartContext` for runtime start/restart actor lifecycle;
- `TelegramRuntimeOperationContext` for sync, send, search and topic refresh;
- `TelegramMediaDownloadContext` and `TelegramMemberSyncContext` for scoped
  media/member slices that need additional boundaries.

## Calls / Voice / STT

Current calls support fixture metadata and fixture transcripts.

Target architecture requires separate Tauri/native permission ADR for:

- microphone;
- camera;
- speaker/device selection;
- call control;
- voice recording;
- video recording;
- STT provider;
- storage retention;
- visible recording consent.

Hidden recording remains unsupported.

## Scope Boundary

Telegram Channel may prepare candidates for shared engines, but must not implement:

- Obligation Engine;
- Decision Engine;
- Memory Engine;
- Knowledge Engine;
- Persona Intelligence;
- Organization Intelligence;
- Project Intelligence.

It may emit source-backed observations and review candidates only.
