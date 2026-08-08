# ADR-0091 Telegram Production Client Capability Model

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0018 Provider Adapter Boundary
- ADR-0026 Desktop First Responsive UI
- ADR-0031 Temporary Desktop Only UI Scope
- ADR-0046 Persistent Dev Mail Cache and Blob Storage
- ADR-0050 V4 Telegram Client, Policy Automation and Call Intelligence
- ADR-0052 Capability Runtime and Action Confirmation Policy
- ADR-0076 Host Vault on macOS
- ADR-0083 Telegram Live User Client Runtime
- ADR-0085 Communication Spine and Consistency / Contradiction Engine

Clarified by:

- ADR-0094 Telegram Base Domain Completion Boundary

## Context

Макошь already has a Telegram foundation: provider account records,
fixture accounts, QR/live-blocked account setup, a TDLib-oriented runtime
manager, chat/history sync endpoints, manual send routing, media download
facade, policy dry-runs, call metadata, transcript storage and a desktop
Telegram page.

The production target is larger. The owner expects Telegram account lifecycle,
multi-account operation, proxies, chat management, messages, soft delete,
message history, media, voice/video messages, calls, channels, groups, forums,
search, drafts, notifications, address book data, media gallery, offline mode,
exports and desktop UX.

This must not turn Макошь into a generic Telegram clone. Telegram remains a
Communication channel feeding source evidence into a local-first Personal Memory
System. Provider behavior is preserved at adapter and source-record boundaries;
canonical Communication, Memory, Relationship, Obligation, Decision, Task,
Document and Context behavior remains Макошь-owned.

External protocol references used by this ADR:

- Telegram documents SOCKS5 and MTProto proxy support for Telegram clients:
  <https://core.telegram.org/proxy>
- TDLib documents `addProxy` as a network proxy function that can be called
  before authorization:
  <https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1add_proxy.html>
- TDLib documents SOCKS5 and MTProto proxy type objects:
  <https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1proxy_type_socks5.html>
  and
  <https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1proxy_type_mtproto.html>

## Decision

Implement production Telegram as a capability-gated Communication channel, not
as a separate messenger domain.

### Capability States

Every Telegram operation must be represented in a backend capability contract.
The UI may hide, disable or explain actions, but the backend remains the source
of authority.

Capability states:

- `available`: implementation, storage, policy, audit and validation exist.
- `blocked`: the feature is architecturally allowed but missing a required
  adapter, permission, secret, runtime dependency or validation gate.
- `degraded`: the feature was available but the current account/runtime cannot
  execute it reliably.
- `planned`: the feature is intentionally deferred to a named initiative and is
  not part of base Telegram channel capability completion.
- `unsupported`: the feature is intentionally out of current scope or conflicts
  with Макошь policy.

Capability action classes follow ADR-0052:

- `read`;
- `local_write`;
- `provider_write`;
- `destructive`;
- `export`;
- `secret_access`;
- `automation`.

Provider writes, destructive actions, sensitive exports and secret access require
explicit owner confirmation unless a future scoped automation policy permits a
narrow operation. AI and automation must not choose account, destination,
template, delete scope, export scope, call state or admin authority from
retrieved content.

### Accounts And Sessions

Telegram supports multiple `telegram_user` and `telegram_bot` accounts.

Rules:

- Account records store non-secret metadata only.
- Credential lookup uses `account_id + secret_purpose`; provider kind alone must
  never select credentials.
- New credential payloads live in the host vault per ADR-0076.
- Each live user account has an account-scoped runtime actor and account-scoped
  TDLib state path.
- Multiple account actors may run at the same time. Cross-account identity
  linking is a Persona/Relationship problem, not a Telegram account problem.
- Add account, authorization, logout, session import and session export are
  lifecycle commands and must emit auditable events.
- Account removal disables the account and stops its runtime. Existing local
  source evidence is retained unless a separate explicit destructive purge
  capability is implemented.
- Logout may revoke or discard provider session state only after explicit owner
  confirmation and must not delete canonical Communication evidence.
- Session import/export bundles are sensitive. They must be encrypted,
  account-scoped, manifest-backed and require host-vault unlock. They must not
  contain plaintext API hashes, bot tokens, session encryption keys or proxy
  secrets.

### Proxies

Telegram user-account runtime may support SOCKS5 and MTProto proxy profiles.

Rules:

- Proxy profiles are account-scoped or runtime-scoped non-default configuration.
- Proxy secrets and SOCKS5 passwords are secret payloads, not application
  settings.
- Proxy host, port, kind, label and non-secret status may be stored as metadata.
- A proxy change requires runtime restart or TDLib proxy command execution
  through the account actor.
- Proxy status, last error and active profile must be visible in the UI.
- Proxy testing must not leak credentials into logs, audit records or telemetry.

### Source Evidence And Local Truth

Telegram source data must enter Макошь through append-only raw source records and
canonical events.

Rules:

- Raw provider records are immutable source evidence.
- Canonical `communication_messages` are projections from source records.
- Search indexes, media galleries, unread views, folders, pinned views and
  desktop notifications are derived or provider-overlay state.
- Message bodies, media bytes, call audio and document bytes must not be written
  to audit records.
- Media bytes live in local blob storage, not PostgreSQL. PostgreSQL stores
  metadata, hashes, scanner state and local references.
- Attachment metadata must pass through the attachment safety scanner boundary.
  A no-op scanner records `not_scanned`; it must not mark Telegram attachments
  as `clean`.

### Chats

Chats, groups, channels, forums, archived chats, pinned chats, folders and hidden
views are account-scoped projections over provider state plus local owner
overlays.

Rules:

- Listing, sorting, filtering and searching chats are read operations.
- Mark read/unread, archive/unarchive, pin/unpin and folder changes are provider
  writes when mirrored to Telegram; local-only overlays are local writes and must
  be labeled as local-only.
- Hide is a local owner overlay unless a provider-backed equivalent is
  explicitly implemented.
- Delete chat is destructive. It must create a local tombstone and require
  explicit confirmation before any provider-side deletion request.
- Forum topics are first-class thread projections under a chat. Topic create,
  close, delete, pin and reply actions are provider writes or destructive
  actions according to their effect.

### Messages

Message send, edit, delete, reply, forward, reaction, pin, saved-message,
export, jump-to-message and reply-thread actions must be modeled as explicit
commands.

Rules:

- Drafting is local. Sending is a provider write.
- Reply, forward, edit, reaction and pin commands must preserve target message
  identifiers, account, chat, actor and preview hash in audit metadata. Message
  body content must not be stored in audit records.
- Multi-select is UI state until a bulk command is submitted. Bulk commands must
  expand to per-message command records so failures are explainable.
- Copy to clipboard is local UI behavior and should not create durable state
  unless the copied item is saved or exported.
- Save message creates a local source-backed saved item, not a Telegram source
  mutation unless Telegram Saved Messages write support is explicitly selected.
- Jump-to-message and open-reply-thread are read/navigation operations.

### Soft Delete And Message History

Макошь never physically deletes local Telegram message evidence as the default
delete behavior.

Rules:

- Local delete creates a tombstone event.
- Tombstones record reason, actor class, observed time, target message and
  source event.
- Supported reason classes include `deleted_by_owner`,
  `deleted_by_counterparty`, `deleted_by_provider`, `moderation_removed`,
  `account_removed`, `retention_policy` and `unknown`.
- UI must show deletion reason, deletion time and deletion history where known.
- Provider-side delete requests are destructive commands and must still leave
  the local tombstone and historical source evidence.
- Message edit history is versioned from observed updates. Макошь must not claim
  to reconstruct provider edit versions that were never observed locally.
- Version diff views compare local observed versions and must cite source
  records or events.

### Attachments And Media

Telegram media is modeled as Communication attachments and local blob objects.

Supported target content classes:

- images;
- video;
- documents;
- PDF;
- DOCX;
- XLSX;
- ZIP;
- audio;
- voice messages;
- video messages;
- contact cards;
- locations;
- GIF;
- stickers;
- links and link previews.

Rules:

- Download is a provider read plus local blob write.
- Open and preview read local cached bytes where available and otherwise request
  an explicit download.
- Save local and save to Documents are local writes; save to Documents creates or
  links a Document domain artifact.
- Share and export are export actions and require policy/audit handling.
- Auto-download defaults must be conservative by size, chat trust and media
  class.
- Media gallery views are derived from attachment metadata and local blob state.

### Voice Messages, Video Messages And Calls

Voice messages and video messages are message attachments with capture, playback,
download and send workflows. Recording requires desktop permission handling and a
visible recording state.

Audio and video calls are separate from message sync and require their own live
runtime validation before becoming available.

Rules:

- Playback speed and replay are local UI state.
- Voice/video message send is a provider write with attachment upload.
- Call history can be source evidence.
- Accept, decline and redial are provider writes.
- Audio/video call start, accept and device selection require a native desktop
  permission and media-device boundary.
- Hidden recording is unsupported.
- Transcription remains local by default and must be account/chat/policy scoped.
- Video calls, group calls and screen sharing require separate ADR-backed runtime
  work before being marked `available`.

### Channels, Groups And Forums

Channels, groups and forums are Telegram chat types with additional provider
permissions.

Rules:

- Subscribe/join, unsubscribe/leave, create group, delete group, invite/remove
  participants and topic moderation are provider writes or destructive actions.
- Participant lists and searches are reads but may be rate-limited by provider
  visibility and permissions.
- Channel publication reading and search are read operations.
- Grouping, archiving and pinning are overlays or provider writes depending on
  whether they are synchronized to Telegram.
- Admin actions must check account permission state before command submission.

### Search, Drafts, Notifications And Offline

Global Telegram search participates in the Макошь Search Engine. Search results
must distinguish local cache hits from provider search results.

Rules:

- Search supports messages, files, images, video, links, Personas, groups,
  channels, date ranges, sender and attachment type as target filters.
- Search indexes are derived and rebuildable.
- Drafts are local-first records with autosave, restore and delete lifecycle.
- Notifications are derived from provider updates and local settings. OS
  notifications must support privacy-safe previews.
- Mute settings may be local-only or provider-synced; UI must label the mode.
- Offline mode uses local cache plus an outbox queue for provider writes.
- Queued commands use stable idempotency keys and retry state.
- Background sync and manual sync must expose progress, checkpoints and errors.

### Exports And Desktop UX

Exports are sensitive. Export chat, group, channel, files, messages and Markdown
must require explicit scope selection.

Rules:

- Exports include source citations and tombstone/history metadata when selected.
- Export bundles must not include secrets, access tokens, session keys or proxy
  passwords.
- Markdown export preserves message order, sender, time, attachment references
  and deletion/edit history when included.
- Drag and drop for files/images creates an upload draft until sent.
- Context menus and hotkeys submit the same backend commands as visible buttons.
- Multi-window chat views share one account/runtime state and must not spawn
  duplicate account actors.
- Tray mode and OS notifications must not imply background sync when sync is
  disabled.
- Opening multiple chats at once is a UI projection over the same local cache.

## Consequences

Positive:

- Telegram can reach production depth without violating Макошь local-first,
  source-evidence and Personal Memory System boundaries.
- High-risk actions have one policy/audit model instead of ad-hoc UI checks.
- Session, proxy, media and export handling stays aligned with the host-vault
  and local blob storage decisions.
- Missing features can be shown honestly as `blocked` or `unsupported` instead
  of hidden runtime gaps.

Negative:

- The capability matrix is larger than the current implementation and requires
  staged delivery.
- Some Telegram parity features depend on TDLib behavior, native desktop
  permissions and provider-side account privileges.
- Soft delete/history semantics require additional event and projection work
  before destructive commands can be safe.

Risk handling:

- Do not mark a provider-write, destructive, call, export, proxy or session
  import/export capability `available` until storage, audit, capability checks,
  UI state and tests exist.
- Do not auto-download all media.
- Do not hide tombstones for provider deletions.
- Do not store secrets in `application_settings`, audit records, source payload
  logs, export manifests or blob paths.
- Do not allow AI or automation to perform destructive Telegram actions.
- Do not validate production Telegram with live accounts in CI. Live validation
  remains opt-in and fixture runtime remains mandatory.

## Non-Goals

- Mobile UI.
- Cloud relay service.
- Turning Макошь into a generic Telegram clone.
- Physical deletion of local source evidence as the default delete action.
- Hidden recording.
- Fine-tuning or training on private Telegram data.
- Bypassing Telegram provider limitations or account permissions.

## Required Follow-Up

- Keep the detailed product and implementation capability matrix in
  `docs/integrations/telegram/status.md` and `docs/integrations/telegram/gap-analysis.md`.
- Add migrations and implementation ADRs before introducing tombstone/message
  version persistence, proxy persistence, outbox persistence, call runtime or
  session bundle schemas.
- Extend `/api/v1/integrations/telegram/capabilities` before exposing new UI controls.
- Add fixture tests for every new capability state and opt-in smoke tests for
  live TDLib flows.
