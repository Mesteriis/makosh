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

- Chunk ID / ID чанка: `116-doc-docs-part-007`
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

### `docs/integrations/whatsapp/full-functionality-target.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/full-functionality-target.md`
- Size bytes / Размер в байтах: `33404`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# WhatsApp Full Functionality Target for Макошь

Status: target product/architecture specification.
Date: 2026-06-24.

Goal: implement full WhatsApp functionality inside Макошь without violating the Макошь ownership model.

WhatsApp in Макошь is not a standalone messenger clone. It is a provider/runtime integration that supplies source evidence into the Communications domain and the Макошь memory/intelligence system.

## Core invariant

```text
A channel is never a domain.
A channel is an integration.
A communication is the domain object.
```

Therefore:

```text
WhatsApp Runtime
  -> Provider Observation
  -> Signal Hub / Event Store
  -> Communications Projection
  -> Radar / Review / Timeline / Search / Engines
  -> Domain commands only through workflows
```

## Provider shapes

Макошь should model WhatsApp as a provider family, not as one implementation.

| Provider kind | Purpose | Status | Notes |
|---|---|---|---|
| `whatsapp_web_companion` | Visible desktop WebView/WhatsApp Web companion runtime | Target/safe baseline | Closest to existing ADR-0051 language. |
| `whatsapp_native_md` | Native unofficial WhatsApp multi-device protocol runtime | Proposed experimental provider | Candidate path for full functionality through Rust libraries. |
| `whatsapp_business_cloud` | Official Meta WhatsApp Business Platform Cloud API | Future provider | Business-only semantics; not a personal account substitute. |

Current repository provider kind `whatsapp_web` may remain as compatibility alias while the provider shape is clarified.

## Functional target matrix

### 1. Accounts and sessions

Target capabilities:

- multiple WhatsApp accounts;
- personal WhatsApp account via companion/native provider;
- WhatsApp Business App account via companion/native provider;
- future Meta Business Cloud account as separate provider;
- QR code pairing;
- pair-code/phone-number linking where provider supports it;
- persistent sessions;
- session health and reconnect diagnostics;
- logout, revoke, relink and remove;
- account-scoped local state path;
- account-scoped secret/session protection material;
- no session secrets in PostgreSQL.
- successful QR/pair-code/native authorization must write session material to
  host vault and create an account-scoped `whatsapp_web_session_key` secret
  reference/binding so runtime startup can restore the account without owner
  action.

Lifecycle states:

```text
created
link_required
qr_pending
pair_code_pending
linked
syncing
available
degraded
blocked
revoked
removed
```

### 2. Dialogs and conversation surfaces

Target source dialog kinds:

- private chat;
- group chat;
- community;
- community subgroup;
- broadcast list, if provider exposes it;
- channel/newsletter, if provider exposes it;
- status feed.

Projection target:

```text
communication_conversations
communication_conversation_participants
communication_identities
communication_channels
```

Dialog state:

- title/display name;
- avatar/profile picture metadata;
- unread count;
- last message;
- pinned/archive/mute/starred overlays;
- provider labels/folders where available;
- local Макошь folders/saved searches separately from provider state;
- source-backed participant count and role metadata.

Current fixture foundation:

- fixture dialog records are source-backed through `WhatsAppProviderRuntime`;
- projected WhatsApp conversations already back provider-neutral conversation
  list/detail/member/search reads;
- fixture dialog metadata now carries source-backed unread count,
  participant-count, avatar/profile-picture and provider-label facts through
  canonical conversation metadata, with the same sanitized fields emitted on
  `whatsapp.dialog.updated`;
- live `/runtime-bridge/dialogs` observed reconciliation now preserves
  `provider_observed.runtime_bridge_dialog` provenance and emits canonical
  `conversation.archive.completed` runtime-event evidence through the same
  event spine contract used by fixture dialog reconciliation;
- fixture-backed `/api/v1/integrations/whatsapp/provider-sync/chats` and
  `/provider-sync/history` can return projected conversations/history for
  runtime-control flows and emit sanitized `whatsapp.sync.*` lifecycle events;
- live runtime sync, paging, reconciliation breadth and realtime runtime
  bridging are still required.

### 3. Messages

Target message classes:

- text;
- reply/quote;
- forward;
- edit/update, observed versions only;
- delete/tombstone;
- reaction;
- poll;
- contact card;
- location;
- link preview;
- system/service message;
- ephemeral/disappearing message metadata where observed;
- view-once metadata, without bypassing provider privacy semantics.

Required message metadata:

- `account_id`;
- `channel_kind`;
- provider conversation id;
- provider message id;
- sender provider identity id;
- sender display name;
- participant/identity trace links;
- occurred/observed/projected timestamps;
- raw record id;
- source fingerprint;
- delivery/read/play receipt state;
- provenance;
- confidence.

Message lifecycle must store only observed provider facts. Макошь must not invent edit history, missing deletes, missing receipts or unobserved provider metadata. Apparently facts need evidence now. Humanity had a long run without it.

Current fixture foundation:

- fixture message update records are source-backed through
  `WhatsAppProviderRuntime`;
- fixture message metadata can already carry broader structured provider facts
  for mentions, link previews, polls, location, contact cards, stickers,
  join/leave service updates, system/service messages, ephemeral flags and
  view-once flags through canonical `message_metadata`;
- projected WhatsApp message/status evidence can already reuse the shared
  Communications summary workflow to mirror review candidates for
  `new_person`, `new_organization` and `knowledge_candidate` items, so
  people/organization discovery remains event-driven and review-driven rather
  than a direct provider-to-Personas write path;
- fixture reply/forward evidence can project canonical
  `communication_message_refs` when provider reference metadata is observed;
- accepted update signals project into canonical
  `communication_message_versions`;
- fixture message delete records project into canonical
  `communication_message_tombstones`;
- fixture receipt records project into canonical
  `communication_messages.delivery_state`;
- live `/runtime-bridge/messages`, `/message-updates`, `/message-deletes`,
  `/reactions`, `/media`, `/dialogs`, `/participants` and `/statuses` now also
  stamp raw-evidence provenance with their respective
  `provider_observed.runtime_bridge_*` sources, so accepted Signal Hub evidence
  keeps live-runtime origin instead of collapsing back into fixture-only raw
  metadata before downstream consumers see it;
- live `/runtime-bridge/receipts` now also stamps raw-evidence provenance with
  `provider_observed.runtime_bridge_receipt`, so receipt observations keep
  their live-runtime ingress origin before Signal Hub acceptance;
- accepted WhatsApp receipt events now also participate in the shared
  provider-observation reconciliation consumer, so later receipt evidence can
  advance durable command `delivery_state` after the original
  provider-observed message completion has established a stable
  `provider_message_id`;
- live `/runtime-bridge/runtime-events` now also stamps raw-evidence
  provenance with `provider_observed.runtime_bridge_runtime_event`, while
  live `/runtime-bridge/sync-lifecycle` and `/runtime-bridge/media-lifecycle`
  captured runtime events preserve their lifecycle `source` markers as raw
  `observed_source` values instead of collapsing back into fixture-only or
  internal capture semantics;
- provider-neutral `/api/v1/communications/messages/{message_id}/reply-chain`
  and `/api/v1/communications/messages/{message_id}/forward-chain` can now read
  canonical WhatsApp refs;
- live provider lifecycle consumer, reconciliation and realtime runtime event
  bridge are still required.
- live command workers should receive explicit provider/runtime dispatch
  metadata from `/runtime-bridge/commands/claim` (`provider_kind`,
  `provider_shape`, `runtime_kind`, `lifecycle_state`,
  `session_restore_available`, `runtime_blockers`) plus durable execution
  state (`capability_state`, `action_class`, `confirmation_decision`,
  `provider_state`, `result_payload`) instead of relying on hidden account
  lookups.
- live command failure reports should also be able to carry structured
  `error_code` and retry-delay hints so a replaceable runtime worker can feed
  the durable retry/dead-letter path without collapsing everything into one
  opaque error string.
- the durable retry path should use the same capped exponential backoff policy
  for runtime-reported failures and stale interrupted executions, with
  structured failure metadata preserved in durable command state.

### 4. Reactions

Target capabilities:

- project reaction state;
- add reaction;
- change reaction;
- remove reaction;
- reconcile provider-observed reaction state;
- emit realtime patch events.

Projection target:

```text
communication_message_reactions
```

Current fixture foundation:

- fixture reaction records are source-backed through `WhatsAppProviderRuntime`;
- accepted reaction signals project into `communication_message_reactions`;
- provider-neutral `/api/v1/communications/messages/{message_id}/reactions`
  can now read canonical WhatsApp reaction rows;
- live runtime reaction execution is still required; fixture
  provider-observed reaction reconciliation now completes durable command rows
  through the event spine.

Command path:

```text
UI
  -> Communications reaction command
  -> communication.provider_command.requested
  -> WhatsApp integration executes
  -> provider-observed reaction evidence
  -> communication.provider_command.completed
```

### 5. Media and attachments

Target media classes:

- photo;
- video;
- document;
- audio;
- voice note;
- sticker;
- GIF;
- contact card;
- location;
- avatar/profile picture;
- status media.

Required rules:

- media bytes live in local blob storage, not PostgreSQL;
- PostgreSQL stores metadata, hashes, local blob refs, scanner state and provenance;
- previews require safe preview artifacts;
- no file is marked `clean` without scanner evidence;
- download/upload is command-driven and observable;
- media dedup uses hashes and provider refs;
- voice notes are attachments first, transcripts later.

Media lifecycle events:

```text
whatsapp.media.download.requested
whatsapp.media.download.started
whatsapp.media.download.progress
whatsapp.media.download.completed
whatsapp.media.download.failed
whatsapp.media.upload.requested
whatsapp.media.upload.completed
whatsapp.media.upload.failed
```

Current fixture foundation:

- fixture media metadata is source-backed through `WhatsAppProviderRuntime`;
- media metadata projects into `communication_attachments`;
- `/api/v1/integrations/whatsapp/provider-sync/media` can now return
  projected WhatsApp attachment snapshots from canonical attachment storage,
  optionally scoped by `provider_chat_id` and `content_type`;
- provider-neutral `/api/v1/communications/search/media` can already return
  projected WhatsApp attachments from canonical attachment storage;
- fixture media command retries now emit sanitized
  `whatsapp.media.upload.started|progress|completed` and
  `whatsapp.media.download.started|progress|completed` lifecycle events through
  the event spine while reconciling durable provider-command rows;
- scanner state is `not_scanned` until a real scanner backend provides a
  verdict;
- blocked-safe `/api/v1/integrations/whatsapp/provider-media/upload` and
  `/provider-media/download` routes now persist durable command rows and emit
  sanitized `whatsapp.media.*.(requested|failed)` events, and those blocked-safe
  media lifecycle phases now also materialize canonical accepted
  `whatsapp.runtime_event` evidence through the shared raw-
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/whatsapp/gap-analysis.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/gap-analysis.md`
- Size bytes / Размер в байтах: `12409`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# WhatsApp Gap Analysis

Status date: 2026-06-17.

Labels:

- `IMPLEMENTED` — implemented and confirmed by current files/tests/docs audit.
- `PARTIAL` — planned/documented only in this starting package.
- `BROKEN` — implementation exists but current evidence shows it does not work.
- `MISSING` — durable production implementation is not counted in this audit.
- `REGRESSION` — current behavior is worse than previously documented behavior.
- `UNSUPPORTED` — intentionally out of current scope or conflicts with policy.

Evidence sources: ADR-0051, ADR-0018, ADR-0052, ADR-0074, ADR-0085,
Communications domain docs and the existing channel documentation structure.

This audit intentionally starts from:

```text
IMPLEMENTED = 0
PARTIAL = planned only
MISSING = all production capabilities
```

No confirmed `BROKEN` or `REGRESSION` capability is listed because no production
WhatsApp Channel implementation is counted in this audit.

## Accounts / Runtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| `whatsapp_personal` account | MISSING | Account kind is required but not counted as implemented. |
| `whatsapp_business` account | MISSING | Business App via WhatsApp Web companion is required; Meta Cloud is separate future provider. |
| Multiple accounts | MISSING | Account-scoped metadata, sessions and runtime actors are required. |
| Secret/session boundary | MISSING | Session protection must use `account_id + secret_purpose`; no secrets in PostgreSQL. |
| WebView companion runtime | MISSING | ADR-0051 requires owner-visible desktop runtime before live support. |
| Fixture/manual runtime | MISSING | Needed for deterministic validation before live provider work. |
| Runtime health | PARTIAL | Sanitized health route exists and now reports blocked/degraded diagnostics for session, WebView/runtime, storage and validation, but real live producer diagnostics remain incomplete until WebView/native/business runtimes exist. |
| Meta Business Cloud provider | UNSUPPORTED | Future provider shape only, not a substitute for personal WhatsApp Web. |

## Dialogs / Chat Management

| Capability | Status | Evidence / Gap |
|---|---|---|
| Private chats | MISSING | Need account-scoped dialog projection and message timeline. |
| Groups | MISSING | Need group projection, participant evidence and join/leave commands. |
| Communities | MISSING | Need community membership and source evidence model. |
| Broadcasts | MISSING | Need broadcast evidence projection without treating it as a separate domain. |
| Status dialog surface | PARTIAL | Status evidence now projects into a synthetic `status-feed` conversation surface backed by canonical `communication_conversations` and provider-neutral conversation reads/search, but live feed sync/media lifecycle is still incomplete. |
| Unread/read state | MISSING | Requires provider-observed evidence and command model if mirrored to provider. |
| Pin/archive/mute overlays | MISSING | Need local-vs-provider state distinction and capability states. |
| Join/leave | MISSING | Must be provider-write commands with outbox and reconciliation. |

## Messaging

| Capability | Status | Evidence / Gap |
|---|---|---|
| Text messages | MISSING | Need raw records, projection and `whatsapp.message.created`. |
| Replies | MISSING | Need reply target projection and reply command. |
| Forwards | PARTIAL | Native `whatsapp_native_md` can submit smoke-gated forwarded-text reemit through `WhatsAppProviderRuntime` and provider-observed reconciliation; richer provider-native forward/copy attribution still depends on live evidence and provider support. |
| Deletes | MISSING | Need tombstones, delete evidence and destructive command path. |
| Edits | MISSING | Only if provider/runtime supports it; must store observed versions only. |
| Raw provider evidence | MISSING | Need sanitized raw evidence route and append-only source records. |
| Message identity | MISSING | Need account/chat/message/sender identifiers and raw record links. |

## Reactions

| Capability | Status | Evidence / Gap |
|---|---|---|
| Reaction projection | MISSING | Need source-backed reaction state. |
| Add/change reaction | MISSING | Provider-write command required. |
| Remove reaction | MISSING | Provider-write command required. |
| Reaction realtime | MISSING | Need `whatsapp.reaction.changed`. |
| Provider reconciliation | MISSING | Need observed state before commands become `completed`. |

## Media

| Capability | Status | Evidence / Gap |
|---|---|---|
| Photos | MISSING | Need metadata, download, preview and local blob path. |
| Videos | MISSING | Need metadata, download, preview and local blob path. |
| Documents | MISSING | Need metadata, scanner state and safe preview route. |
| Audio | MISSING | Need download/playback through local blob. |
| Voice notes | MISSING | Need metadata, attachment and playback; STT out of scope. |
| Contacts | MISSING | Contact cards are identity evidence, not Persona lifecycle. |
| Locations | MISSING | Location evidence must be source-backed and timeline-capable. |
| Stickers | MISSING | Need media metadata and preview after download. |
| GIF | MISSING | Need animation metadata and preview after download. |
| Media upload | MISSING | Must use local attachment/blob and provider-write outbox. |
| Media download | MISSING | Must emit started/progress/completed/failed events. |
| Media gallery/search | MISSING | Needs projection-backed local search. |

## Attachments

| Capability | Status | Evidence / Gap |
|---|---|---|
| Local blob storage | MISSING | Media bytes must stay outside PostgreSQL. |
| Attachment metadata | MISSING | Need provider refs, hash, scanner state and local refs. |
| Deduplication | MISSING | Needs shared blob hash semantics. |
| Scanner-backed clean verdict | MISSING | No attachment may be marked `clean` without scanner backend. |
| Preview artifacts | MISSING | Need safe preview route and generated artifacts. |

## Participants / Identity

| Capability | Status | Evidence / Gap |
|---|---|---|
| Phone traces | MISSING | Phone number is evidence, not Persona truth. |
| `wa_id` traces | MISSING | Provider-specific identity evidence required. |
| Display names | MISSING | Mutable labels need observed history/source evidence. |
| Contact resolution | MISSING | Candidate handoff only; no Persona lifecycle in WhatsApp. |
| Persona candidates | MISSING | Source-backed candidate flow required. |
| Group member evidence | MISSING | Need role/status/admin/member evidence. |
| Community member evidence | MISSING | Need community-scoped evidence. |

## WhatsApp Statuses

| Capability | Status | Evidence / Gap |
|---|---|---|
| Status read | MISSING | Provider read capability required. |
| Status publish | MISSING | Provider-write command required. |
| Status media | MISSING | Media attachment linkage required. |
| Status identity signal | MISSING | Identity traces must remain evidence/candidates only. |
| Status timeline entry | MISSING | Timeline evidence integration required. |

WhatsApp Status is not a separate domain.

## Calls

| Capability | Status | Evidence / Gap |
|---|---|---|
| Call metadata | MISSING | First-version scope allows metadata only. |
| Call evidence | MISSING | Need source-backed call records. |
| Call timeline entries | MISSING | Need Timeline evidence projection. |
| Audio capture | UNSUPPORTED | Out of scope until future ADR. |
| Video capture | UNSUPPORTED | Out of scope until future ADR. |
| Call control | UNSUPPORTED | Out of scope until future ADR. |
| Recording | UNSUPPORTED | Hidden recording is forbidden. |
| Live call handling | UNSUPPORTED | Out of scope until future ADR. |
| STT | UNSUPPORTED | Out of first-version scope. |

## Voice Notes

| Capability | Status | Evidence / Gap |
|---|---|---|
| Voice metadata | MISSING | Need provider metadata projection. |
| Voice attachment | MISSING | Need local blob and attachment row. |
| Voice playback | MISSING | Playback from local blob only after download. |
| Voice send | MISSING | Provider-write command from local blob. |
| Future transcript integration | PARTIAL | Planned integration point only; no STT in first version. |

## Provider Command Outbox

| Capability | Status | Evidence / Gap |
|---|---|---|
| Durable command rows | MISSING | Required before provider writes. |
| `queued` state | MISSING | Required command state. |
| `executing` state | MISSING | Required command state. |
| `retrying` state | MISSING | Required command state. |
| `completed` state | MISSING | Must require provider-observed state. |
| `failed` state | MISSING | Required command state. |
| `dead_letter` state | MISSING | Required command state. |
| Retry policy | MISSING | Needed for transient WebView/provider failures. |
| Reconciliation | MISSING | Needed before marking commands complete. |
| Redacted audit | MISSING | Needed for all provider-write/destructive commands. |

## Realtime

| Capability | Status | Evidence / Gap |
|---|---|---|
| Generic event transport | MISSING | Shared transport exists in Макошь, but WhatsApp contracts are not counted. |
| New message event | MISSING | Need `whatsapp.message.created`. |
| Updated message event | MISSING | Need `whatsapp.message.updated`. |
| Deleted message event | MISSING | Need `whatsapp.message.deleted`. |
| Chat update event | MISSING | Need `whatsapp.chat.updated`. |
| Reaction event | MISSING | Need `whatsapp.reaction.changed`. |
| Media download lifecycle | MISSING | Need started/progress/completed/failed. |
| Command status | MISSING | Need status_changed/reconciled. |
| Frontend cache patching | MISSING | Need shared realtime bootstrap consumers. |

## API / UI

| Capability | Status | Evidence / Gap |
|---|---|---|
| API route set | MISSING | Target routes documented only. |
| Frontend API client | MISSING | Target modules documented only. |
| Workbench | MISSING | Desktop-first UI target only. |
| Account/session setup | MISSING | Required before live runtime. |
| Capability UX | MISSING | Needed before command controls. |
| Command audit panel | MISSING | Needed for outbox visibility. |
| Status surface | MISSING | Needed for WhatsApp Status evidence. |
| Media viewer | MISSING | Needed for attachment preview/playback. |

## AI / Shared Engines

| Capability | Status | Evidence / Gap |
|---|---|---|
| Summary | MISSING | Shared-engine candidate only, not WhatsApp-owned memory. |
| Translation | MISSING | Future shared-engine UX. |
| Task extraction | MISSING | Candidate handoff only. |
| Obligation extraction | MISSING | Candidate handoff only. |
| Decision extraction | MISSING | Candidate handoff only. |
| Persona extraction | MISSING | Identity trace candidate only. |
| Polygraph evidence | MISSING | Source-backed contradiction input only. |
| AI state lifecycle | MISSING | No WhatsApp-specific lifecycle intended. |

## Scope Boundary

| Capability | Status | Evidence / Gap |
|---|---|---|
| Memory Engine | MISSING | Not owned by WhatsApp. |
| Knowledge Engine | MISSING | Not owned by WhatsApp. |
| Persona Intelligence | MISSING | Identity traces only. |
| Organization Intelligence | MISSING | Candidate evidence only. |
| Task lifecycle | MISSING | Candidate evidence only. |
| Decision lifecycle | MISSING | Candidate evidence only. |
| Obligation lifecycle | MISSING | Candidate evidence only. |

## Priority Recommendations

### P0 — Documentation-aligned foundation

1. Keep WhatsApp under Communications and source evidence.
2. Resolve hidden WebView wording against ADR-0051 with owner-visible controls.
3. Define account/session/capability contracts before runtime work.
4. Build fixture/manual evidence path before live WebView automation.

### P1 — Read-only provider evidence

1. Dialog/message/status raw records and projections.
2. Phone-centric identity traces.
3. Media metadata and conservative download.
4. Realtime message/chat/media events.

### P2 — Provider-write parity

1. Durable outbox.
2. Command audit and retry/dead-letter UX.
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/whatsapp/implementation-plan.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/implementation-plan.md`
- Size bytes / Размер в байтах: `40582`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# WhatsApp Implementation Plan

Status: target implementation plan.
Date: 2026-06-24.

This plan starts from the current fixture/runtime foundation and moves toward full WhatsApp functionality plus Макошь intelligence.

## Principle

Implementation must proceed by contracts, not by provider UI temptation.

Correct path:

```text
Provider event
  -> observation/signal
  -> Communications projection
  -> Radar/Review/Timeline/Search/Engines
  -> workflows
  -> domain commands
```

Incorrect path:

```text
WhatsApp adapter
  -> Tasks/Personas/Documents/Knowledge directly
```

That path is how architecture becomes soup.

## Phase P0 — Documentation and decision closure

Goal: make the target explicit before touching live accounts.

Deliverables:

- `docs/integrations/whatsapp/current-audit-2026-06-24.md`;
- `docs/integrations/whatsapp/full-functionality-target.md`;
- `docs/integrations/whatsapp/rust-provider-research.md`;
- `docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md`;
- update `docs/integrations/whatsapp/README.md` with the new document set;
- update `docs/integrations/whatsapp/api.md` after route design is accepted.

Acceptance:

- provider shapes are named;
- full capability matrix exists;
- third-party Rust project choice is documented;
- ToS/account-risk posture is explicit;
- no code claims live runtime support.

## Phase P1 — Provider runtime contract

Goal: define a runtime abstraction that supports WebView, native multi-device and future business cloud providers without changing domain code.

Backend target modules:

```text
backend/src/integrations/whatsapp/runtime/
├── mod.rs
├── provider_runtime.rs
├── supervisor.rs
├── session_store.rs
├── health.rs
├── qr.rs
├── pair_code.rs
├── web_companion.rs
├── native_md.rs
└── business_cloud.rs
```

Core trait sketch:

```rust
pub trait WhatsAppProviderRuntime: Send + Sync {
    fn provider_shape(&self) -> WhatsappProviderShape;
    async fn start(&self, account_id: &str) -> Result<RuntimeStatus, RuntimeError>;
    async fn stop(&self, account_id: &str) -> Result<RuntimeStatus, RuntimeError>;
    async fn link_qr(&self, account_id: &str) -> Result<QrLinkSession, RuntimeError>;
    async fn link_pair_code(&self, account_id: &str, phone: &str) -> Result<PairCodeSession, RuntimeError>;
    async fn health(&self, account_id: &str) -> Result<RuntimeHealth, RuntimeError>;
    async fn store_authorized_session_credential(&self, account_id: &str, session_material: SecretMaterial) -> Result<CredentialBinding, RuntimeError>;
}
```

Acceptance:

- all live capabilities default to `blocked`;
- application code depends on `dyn WhatsAppProviderRuntime`, not a concrete provider library;
- `whatsapp_web_companion` runtime health exposes a bridge contract manifest,
  not a live-availability flag: it requires an owner-visible desktop WebView,
  forbids hidden/headless mode, lists the protected runtime-bridge event routes
  for all inbound/runtime/media/sync families, fixes provider writes to durable
  outbox claim/failure paths with provider-observed reconciliation, and excludes
  session material, cookies, browser profile secrets, QR/pair-code artifacts,
  message bodies and media bytes from health/event-like payloads;
- `whatsapp_web_companion` has a Tauri visible-desktop producer shell:
  `open_whatsapp_web_companion` creates or focuses an account-scoped
  `https://web.whatsapp.com/` WebView and
  `whatsapp_web_companion_manifest` returns the same sanitized event/outbox
  contract. This shell is not live availability: the companion window receives
  only an explicit remote capability for the metadata relay dispatch, the
  commands do not read cookies/session/profile secrets/message bodies/media
  bytes, and public availability remains blocked until manual smoke passes;
- `whatsapp_web_companion` installs only a safe extractor injection contract at
  this stage: a main-frame-only initialization script guarded to
  `https://web.whatsapp.com`, a same-origin navigation guard and a frozen
  metadata contract. It must not read cookies, Web Storage, IndexedDB, browser
  profile secrets, session material, message bodies or media bytes, and must not
  call `fetch`, XHR, `postMessage` or domain APIs. Runtime health reports this
  as `contract_injected_relay_dispatch_available` with
  `tauri_allowlisted_companion_runtime_bridge_dispatch`; the relay posts
  sanitized metadata as `NewWhatsappWebRuntimeEvent` into
  `/runtime-bridge/runtime-events` and does not attempt typed projection until
  richer WebView payloads exist;
- frontend integration code has a typed `@tauri-apps/api` bridge for those
  shell commands. The bridge calls Tauri `invoke` directly and must not route
  through `ApiClient`, `fetch` or backend/domain HTTP APIs; visible runtime-panel
  controls remain gated on UI evidence/design review, while live smoke remains
  the runtime closure gate;
- native provider crates are represented by a driver descriptor; compile-only or
  smoke-gated descriptors must not flip public live runtime capabilities to
  available;
- native runtime health must expose the verified `wa-rs` public SDK methods,
  including the forwarded-text reemit contract for `forward`, and the missing
  safe write APIs for any smoke-gated unsupported command. Missing SDK support
  for `publish_status`, dialog-state writes, `mark_unread` and join-by-invite
  must remain public-blocked and may only produce structured terminal
  dead-letter evidence until a provider API and live smoke prove the path;
- native `native_md` has an account-scoped runtime actor contract whose command
  channel is the durable provider outbox, whose event sink is Signal Hub raw
  evidence, and whose session material policy is host-vault-only with
  PostgreSQL metadata bindings;
- the actor contract names every provider event family that must enter the Hub:
  auth, runtime lifecycle, sync lifecycle, messages, updates, deletes,
  receipts, reactions, dialogs, participants, presence, call metadata, statuses,
  status views/deletes, media lifecycle, command reconciliation and unsupported
  evidence;
- `native_md` classifies real `wa-rs` provider events into those families before
  any projection or workflow sees them; raw SDK notifications and provider-only
  business events are retained as unsupported runtime evidence, not silently
  dropped;
- classified native events are wrapped in a raw-evidence envelope that carries
  account id, stable provider event id, provider shape, runtime driver, raw
  record kind, raw Signal Hub event kind, accepted event kind and source
  fingerprint seed before any future append;
- the native raw-evidence envelope allows sanitized metadata only and must not
  carry session/token/cookie/raw secrets, message bodies or media bytes;
- native events are also converted into a sanitized inbound DTO before any
  future live producer writes Hub evidence; the DTO may carry provider ids,
  JIDs, timestamps, state flags, sync counters and payload-shape flags, but must
  exclude QR codes, pair codes, raw SDK nodes, protobuf action payloads,
  history-sync payloads, about text, push names, session material, message
  bodies and media bytes;
- the sanitized native DTO fixes its dispatch target to an existing
  `/api/v1/integrations/whatsapp/runtime-bridge/*` endpoint family instead of
  allowing ad-hoc runtime-to-domain calls;
- native runtime health exposes the driver descriptor, readiness blocker,
  durable outbox command channel, Signal Hub raw-evidence sink and host-vault
  session boundary under sanitized `checks.native_md_driver` metadata;
- native runtime health also exposes the `wa-rs` backend manifest: the future
  live driver must use `NativeMdHostVaultBackend`, which already implements
  `Backend` over `SignalStore`, `AppSyncStore`, `ProtocolStore` and
  `DeviceStore`, uses the account-scoped `whatsapp_web_session_key` host-vault
  binding, and does not use SDK SQLite or PostgreSQL secret payload storage;
- native runtime health also exposes the compile-checked client factory wiring:
  `NativeMdWaRsClientFactory::configured_builder` binds the host-vault backend,
  `TokioWebSocketTransportFactory`, `UreqHttpClient`, optional pair-code
  options and a sanitized event-handler DTO path without calling `build()` or
  claiming live readiness;
- native runtime health also exposes the compile-checked live driver lifecycle:
  `NativeMdLiveDriver` builds the configured `wa_rs::bot::Bot`, starts it with
  `Bot::run()`, stops through `Client::disconnect()` and task abort cleanup, and
  routes inbound provider events only as owned sanitized DTOs into the shared
  `WhatsAppRuntimeEventSink` contract;
- `WhatsappRuntimeSignalIngestService` implements that shared sink at the
  application boundary: sanitized native DTOs become append-only raw evidence,
  `signal.raw.whatsapp.*.observed` and accepted Signal Hub events, with
  recursive secret-like metadata redaction and duplicate provider event
  idempotency;
- native runtime health also exposes the smoke-gated runtime manager:
  `checks.native_md_manager` / `checks.runtime.native_manager` report account
  scoping, explicit `native_md_live_smoke_enabled` opt-in, host-vault session
  binding requirement, SDK feature state, running state and the
  `blocked_until_manual_live_smoke` public availability gate;
- QR/pair-code startup is vault-aware: `WhatsAppProviderRuntime::start_qr_link`
  and `start_pair_code_link` receive `SecretReferenceStore` and `HostVault`
  context, and the native smoke manager can create the account-scoped
  `whatsapp_web_session_key` host-vault bootstrap binding before starting the
  feature-gated driver;
- live QR/pair-code artifacts are exposed only through the native manager's
  in-memory, one-time transient start response channel after provider-observed
  `PairingQrCode` / `PairingCode` events; they must not be written to
  PostgreSQL, events, logs or health payloads;
- startup restore reconciliation now attempts eligible `whatsapp_native_md`
  runtime startup from the host-vault `whatsapp_web_session_key` binding through
  `WhatsAppProviderRuntime::start_runtime`, gated by explicit native smoke
  opt-in, and emits only sanitized restore-start status/session events;
- native reconnect policy is account-scoped and tick-driven: provider-observed
  degraded/recovered lifecycle evidence updates manager state, bounded reconnect
  attempts reuse the same vault-bound session, and restart events go through the
  sanitized Signal Hub sink;
- QR/pair-code and provider-command blockers include provider-shape-specific
  native blockers while the live driver is missing or feature-disabled;
- session storage is account-scoped;
- successful authorization stores session material in host vault and binds it with `whatsapp_web_session_key`;
- runtime status/start/health resolve the account-scoped host-vault session
  binding without returning raw session material;
- fixture runtime now maps account/session state into target-style lifecycle
  values such as `link_required`, `linked`, `available`, `revoked` and
  logical `removed`;
- local runtime data path is ignored by Git;
- runtime emits sanitized lifecycle events;
- no message bodies or secrets in lifecycle events.

## Phase P2 — Third-party Rust library spike

Goal: decide whether to use `whatsapp-rust` or a fallback fork instead of writing the protocol ourselves.

Spike target:

```text
crates/whatsapp-native-spike/
```

Test matrix:

| Test | Required result |
|---|---|
| Compile on Rust 1.89 | Pass without changing global toolchain. |
| QR callback | Can surface QR code through runtime API. |
| Pair-code callback | Can surface pair code if provider supports it. |
| Session persistence | Can store under account-scoped local path. |
| Receive text | Emits provider event with stable ids. |
| Receive media metadata | Emits media ref without storing bytes in DB. |
| Receive reaction/delete/edit | Emits lifecycle event or known unsupported marker. |
| Send text dry path |
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/whatsapp/live-smoke-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/live-smoke-checklist.md`
- Size bytes / Размер в байтах: `10922`
- Included characters / Включено символов: `10922`
- Truncated / Обрезано: `no`

````markdown
# WhatsApp Live Smoke Checklist

Status: manual local validation only.
Date: 2026-06-26.

This checklist exists for ADR-0101 acceptance and must be used only for
owner-visible local runtime validation. It is not a CI workflow.

## Preconditions

- local development machine under owner control;
- `MAKOSH_LOCAL_API_SECRET` configured;
- host vault available;
- no screen-hidden or headless WhatsApp runtime mode;
- explicit owner opt-in for unofficial personal-account runtime risk;
- test account or low-risk account selected for the session.

Before starting live provider actions, run:

```sh
make whatsapp-domain-closure-audit
make whatsapp-live-smoke-readiness
make whatsapp-native-md-sdk-gap-readiness # required for whatsapp_native_md accounts
```

For the strict manual-smoke preflight, set:

```sh
MAKOSH_LIVE_SMOKE_STRICT_ENV=1
MAKOSH_LOCAL_API_SECRET=...
MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID=...
make whatsapp-live-smoke-readiness
```

When Макошь is already running locally, the same readiness target can also
probe the protected runtime API without performing provider actions:

```sh
MAKOSH_WHATSAPP_RUNTIME_API_PROBE=1
MAKOSH_LOCAL_API_SECRET=...
MAKOSH_WHATSAPP_SMOKE_ACCOUNT_ID=...
MAKOSH_WHATSAPP_SMOKE_PROVIDER_SHAPE=whatsapp_web_companion # optional
make whatsapp-live-smoke-readiness
```

The runtime API probe calls only capabilities, account-capabilities,
runtime-status and runtime-health endpoints through `X-Макошь-Secret`. It
checks account scoping, provider-shape contract when requested and the absence
of raw session/token/cookie/media-ref payload markers.

This readiness check is static/preflight evidence only. It does not replace the
manual live smoke run below.

## Evidence artifact

The manual smoke run must produce a sanitized local evidence artifact before the
domain can be treated as closed. The default path is ignored by git:

```text
.local/whatsapp/live-smoke-evidence.json
```

Create the template locally after the preflight:

```sh
mkdir -p .local/whatsapp
node scripts/whatsapp-live-smoke-evidence.mjs --template > .local/whatsapp/live-smoke-evidence.json
```

Fill it with sanitized evidence references only, then validate it:

```sh
node scripts/whatsapp-live-smoke-collect-evidence.mjs --observations-template \
  --provider-shape whatsapp_native_md > .local/whatsapp/live-smoke-observations.json
make whatsapp-live-smoke-collect-evidence
make whatsapp-live-smoke-evidence
make whatsapp-domain-closure-gate
```

`make whatsapp-live-smoke-collect-evidence` is a normalizer, not a bypass. It
reads `.local/whatsapp/live-smoke-observations.json`, writes an ignored
`.local/whatsapp/live-smoke-evidence-<provider_shape>.json` artifact and then
runs the strict validator. Gates without operator-provided sanitized
`evidence_refs` remain pending, and the command fails until every required gate
has real evidence.

The artifact must use `account_fingerprint = sha256:<64 hex chars>` rather
than raw account ids, phone numbers or JIDs. It must not contain message
bodies, provider payloads, QR/pair codes, cookies, authorization headers,
session material, media keys, direct paths, static URLs, access token values,
app secret values or verify token values. Required gates stay failed until
their evidence entry is `status = passed`; unsupported or skipped entries do
not close the domain.

Each passed gate must also include concrete sanitized `evidence_refs`, not a
free-form note. The validator requires refs with prefixes that match the gate:
`raw_record:` for raw evidence, `event_log:` / `signal_hub:` for accepted
events, `command:` plus observed event refs for provider writes,
`vault_binding:` for session/credential binding, `blob:` / `storage:` for
media bytes, `runtime_api:` for protected API probes, `edge_proxy:` for
Business Cloud ingress and `log_scan:` / `ui:` / `audit:` for redaction checks.
Placeholder refs such as `replace-with-*`, `pending`, `todo`, `example` or
`dummy` are rejected.

## Runtime boundary checks

1. Start Макошь with the intended WhatsApp runtime shape enabled.
2. Verify the runtime is surfaced through:
   - `GET /api/v1/integrations/whatsapp/capabilities`
   - `GET /api/v1/integrations/whatsapp/accounts/{account_id}/capabilities`
   - `GET /api/v1/integrations/whatsapp/runtime/status?account_id=...`
3. Confirm the reported provider shape is correct:
   - `whatsapp_web_companion`, or
   - `whatsapp_native_md`, or
   - `whatsapp_business_cloud`.
4. Confirm blocked capabilities stay blocked if the runtime is not fully ready.

## WebView companion checks

Run these only for `whatsapp_web_companion` accounts.

1. From frontend integration code, call `getWhatsappWebCompanionManifest`, which
   invokes Tauri command `whatsapp_web_companion_manifest` for the account, and
   verify it reports:
   - provider shape `whatsapp_web_companion`;
   - target URL `https://web.whatsapp.com/`;
   - protected `/runtime-bridge/*` event routes;
   - authorized-session path
     `/api/v1/integrations/whatsapp/runtime-bridge/sessions/authorized`;
   - durable outbox claim/failure paths;
   - extractor state `contract_injected_relay_dispatch_available`;
   - relay channel `tauri_allowlisted_companion_runtime_bridge_dispatch`;
   - runtime bridge dispatch `runtime_events_bridge_wired_smoke_pending`;
   - no cookie/session/profile/message/media fields.
2. In the WhatsApp Runtime panel, use `Open Companion`. This calls
   `openWhatsappWebCompanion`, which invokes Tauri command
   `open_whatsapp_web_companion`, and verify an owner-visible WhatsApp Web
   window opens or focuses with an account-scoped `whatsapp-companion-*` label.
3. Verify the companion window is not running in hidden/headless mode and is
   granted only the allowlisted metadata relay dispatch command, not
   domain-mutating Tauri IPC or `core:default`.
4. Verify the companion initialization contract is origin-guarded to
   `https://web.whatsapp.com` and does not read cookies, Web Storage, IndexedDB,
   message bodies or media bytes.
5. Verify the allowlisted relay dispatch accepts only sanitized metadata, posts
   a `NewWhatsappWebRuntimeEvent` to `/runtime-bridge/runtime-events` and
   returns `provider_observed_event_reconciliation_required`.
6. Until live smoke proves WebView observations, accepted Hub events and
   projections end-to-end, keep public runtime availability blocked and do not
   mark provider writes completed from WebView UI actions alone.

## Authorization checks

1. Start QR link flow.
2. Verify no QR secret/session payload is written to PostgreSQL, events, logs or API responses.
3. Complete authorization.
4. Verify authorized session material is stored through host vault binding:
   - secret purpose `whatsapp_web_session_key`;
   - account-scoped binding only.
5. Stop Макошь.
6. Start Макошь again.
7. Verify session restore works without re-pairing.
8. Start pair-code flow if the runtime shape supports it.
9. Verify pair-code lifecycle surfaces sanitized state only.
10. Revoke and relink once; verify lifecycle state transitions remain correct.

## Read path checks

1. Receive a private message.
2. Receive a group message.
3. Receive a reply or quoted message.
4. Receive a forward.
5. Receive a reaction update.
6. Receive a media message.
7. Receive a status update.
8. Verify each item reaches:
   - raw evidence;
   - accepted signal;
   - canonical Communications projection;
   - provider-neutral search/timeline surfaces where applicable.

## Write path checks

1. Send text message.
2. Send reply.
3. Send forward.
4. Edit message if runtime/provider supports it.
5. Delete message if runtime/provider supports it.
6. Add reaction.
7. Remove reaction.
8. Upload media.
9. Download media.
10. Send voice note if runtime/provider supports it.
11. Archive/unarchive chat.
12. Mute/unmute chat.
13. Pin/unpin chat.
14. Mark read/unread.
15. Join/leave group if supported.
16. Publish status if supported.

For each write, verify:

- durable command row exists;
- canonical `communication_provider_commands` mirror exists;
- provider-observed completion or failure is recorded;
- no secret/session material appears in command payloads, audit metadata or emitted events.

## Redaction checks

Verify all of the following are redacted or absent:

- session blobs;
- session keys;
- access tokens;
- refresh tokens;
- cookies;
- raw authorization headers;
- vault payload bytes.

Inspect:

- API responses;
- raw-evidence endpoint;
- audit rows;
- event payloads;
- logs.

## Business Cloud edge proxy checks

Run these only for `whatsapp_business_cloud` accounts.

1. Start Макошь locally with ADR-0056 `MAKOSH_LOCAL_API_SECRET` configured.
2. Run the static edge proxy readiness preflight:
   `make whatsapp-business-cloud-edge-readiness`.
   If the proxy is already running locally, the same target can also probe the
   public proxy surface without touching Meta:
   `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE=1 make whatsapp-business-cloud-edge-readiness`.
   Add `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE=1` only when Макошь is
   running and `/readyz` should reach the protected local proxy manifest.
3. Validate the edge profile:
   `make whatsapp-business-cloud-edge-config`.
4. Start `makosh-whatsapp-business-cloud-edge-proxy` with:
   `make whatsapp-business-cloud-edge-up`.
   The Compose profile reads:
   - `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_BASE_URL`;
   - `MAKOSH_LOCAL_API_SECRET`;
   - optional `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID`.
   `MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_SECRET` may be used instead of
   the shared local API secret when running the binary outside Compose.
5. Verify `GET /readyz` succeeds only when the protected Макошь proxy manifest
   is reachable with local auth.
6. Expose only the proxy path `/webhooks/whatsapp/business-cloud` through the
   chosen public ingress; do not expose Макошь `/api/v1` directly.
7. Verify Meta challenge `GET` succeeds through the proxy and reaches the
   protected Макошь runtime-bridge route.
8. Verify signed webhook `POST` forwards the exact raw body and
   `X-Hub-Signature-256`; Макошь performs app-secret verification and Signal Hub
   ingestion.
9. Verify proxy failures are sanitized and do not return upstream bodies,
   access tokens, app secrets, verify tokens or raw provider payloads.

## Failure and recovery checks

1. Stop runtime during pending command execution.
2. Verify command is retried or dead-lettered through durable policy.
3. Restore runtime.
4. Verify canonical state is not corrupted by partial execution.
5. Verify capability degradation is surfaced instead of silent failure.

## Exit criteria

The smoke run passes only if:

- session restore works from vault-bound state;
- raw evidence remains source-backed;
- writes reconcile through observed provider evidence;
- all inspected surfaces remain redacted for secrets;
- no hidden or headless runtime behavior was required.
````

### `docs/integrations/whatsapp/modules.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/modules.md`
- Size bytes / Размер в байтах: `10961`
- Included characters / Включено символов: `10867`
- Truncated / Обрезано: `no`

````markdown
# WhatsApp Modules

Статус: target module map на 2026-06-17.

WhatsApp остаётся Communication Channel. Модули ниже не создают отдельный
product domain.

This module map intentionally treats production WhatsApp Channel implementation
as not yet existing. Any compatibility or fixture files that may exist in the
repository are not counted as production coverage until a later audit aligns
them with this package.

## Backend Modules

| Module | Target files | Назначение | Status |
|---|---|---|---|
| `api` | `backend/src/integrations/whatsapp/api/` | Protected HTTP handlers for capabilities, accounts, sessions, chats, messages, media, statuses and commands | MISSING |
| `accounts` | `backend/src/integrations/whatsapp/client/accounts/` | Account metadata, lifecycle, secret/session references and account kind validation | MISSING |
| `capabilities` | `backend/src/domains/api_support/` | Operation-level capability matrix with account/runtime overrides | MISSING |
| `runtime_webview` | `backend/src/integrations/whatsapp/runtime/`, `frontend/src-tauri/src/whatsapp_companion.rs`, `frontend/src/integrations/whatsapp/api/whatsappCompanion.ts`, `frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue` | Account-scoped desktop WebView companion runtime and runtime event bridge | PARTIAL: visible Tauri shell + typed frontend invoke bridge + runtime-panel `Open Companion` action + backend bridge contract + origin-guarded extractor contract + allowlisted runtime-event relay dispatch; live smoke missing |
| `sessions` | `backend/src/integrations/whatsapp/client/sessions/` | Local WebView session metadata and lifecycle state | MISSING |
| `adapter` | `backend/src/integrations/whatsapp/client/` | Provider event normalization, validation and source-record conversion | MISSING |
| `dialogs` | WhatsApp client/store projection module | Private, group, community, broadcast and status dialog projection | MISSING |
| `messages` | WhatsApp client/store message module | Raw record ingestion and canonical Communication message projection | MISSING |
| `message_versions` | WhatsApp lifecycle module | Observed edit/update versions and diff metadata | MISSING |
| `message_tombstones` | WhatsApp lifecycle module | Delete evidence, local visibility and tombstone history | MISSING |
| `reactions` | WhatsApp reactions module | Reaction evidence, commands and reconciliation | MISSING |
| `participants` | WhatsApp participants module | Phone-centric participant and role evidence | MISSING |
| `identity_traces` | WhatsApp identity module | Phone number, `wa_id`, display-name and contact-card traces | MISSING |
| `statuses` | WhatsApp status module | Status source evidence, media evidence and timeline signals | MISSING |
| `media` | WhatsApp media module | Provider media metadata, download/upload and projection refresh | MISSING |
| `attachments` | Communication attachment/blob boundary | Local blob persistence, metadata and scanner state | MISSING |
| `voice_notes` | WhatsApp voice module | Voice metadata, download/playback and future transcript handoff | MISSING |
| `calls` | Platform calls integration | Call metadata, call evidence and Timeline entries | MISSING |
| `command_outbox` | WhatsApp provider command module | Durable provider-write queue, retry and reconciliation | MISSING |
| `realtime_events` | shared event bus integration | Sanitized `whatsapp.*` event contracts | MISSING |
| `search` | WhatsApp search module | Message/media/status/participant search over local projections | MISSING |
| `audit` | platform audit boundary | Redacted lifecycle, capability and provider-write audit records | MISSING |

## Missing / Target Backend Modules

| Module | Назначение | Why needed |
|---|---|---|
| `webview_companion_runtime` | Owner-linked WhatsApp Web session | Visible shell, runtime-panel action, safe extractor injection contract and allowlisted runtime-event relay dispatch exist; live smoke required before live sync or provider writes |
| `capability_matrix` | Per-operation capability model | Required before exposing UI commands |
| `command_outbox` | Durable provider-write command queue | Required for send/reply/forward/reaction/delete/media/status/voice commands |
| `provider_observed_reconciliation` | Confirm provider state after command execution | Required before marking commands `completed` |
| `status_projection` | WhatsApp Status evidence projection | Required for status read/publish and Timeline evidence |
| `phone_identity_trace_projection` | Phone/wa_id/display-name evidence | Required for Persona candidates without implementing Persona lifecycle |
| `media_transfer` | Download/upload orchestration through runtime | Required for attachments, media gallery and voice notes |
| `call_metadata_projection` | Call source evidence only | Required without enabling live call handling |
| `attachment_scanner_integration` | Scanner-backed verdicts | Required before safe preview can mark files `clean` |
| `provider_search` | Optional WhatsApp provider search | Required only after WebView runtime can support it safely |

## Frontend Modules

| Module | Target files | Назначение | Status |
|---|---|---|---|
| `page` | `frontend/src/integrations/whatsapp/views/` | Desktop WhatsApp workbench | MISSING |
| `api` | `frontend/src/integrations/whatsapp/api/` | Typed backend route calls | MISSING |
| `queries` | `frontend/src/integrations/whatsapp/queries/` | TanStack Query hooks/mutations | MISSING |
| `store` | `frontend/src/integrations/whatsapp/stores/` | Local UI state, filters and selected context | MISSING |
| `account_setup` | WhatsApp account/session components | Account setup, link flow, logout/remove and runtime status | MISSING |
| `dialogs` | Dialog list components | Private/group/community/broadcast/status list | MISSING |
| `messages` | Message thread components | Source-backed timeline, reply/forward/delete/reaction UI | MISSING |
| `composer` | Composer components | Capability-gated send/reply/media/voice/status commands | MISSING |
| `participants` | Participant inspector components | Phone-centric participant evidence and role labels | MISSING |
| `media_viewer` | Media components | Photo/video/document/audio/sticker/gif preview and download state | MISSING |
| `voice_player` | Voice-note components | Local playback and transcript placeholder without STT | MISSING |
| `status_view` | Status components | Status evidence timeline and publish command surface | MISSING |
| `calls_panel` | Call evidence components | Read-only call metadata and Timeline evidence | MISSING |
| `command_audit` | Command inspector | Outbox status, retry/dead-letter diagnostics | MISSING |
| `realtime_status` | Shared realtime status consumption | WS/SSE/long-poll state and cache patch diagnostics | MISSING |

## Missing / Target Frontend Modules

| Module | Назначение |
|---|---|
| `webview_link_flow` | Owner-visible QR/link/session lifecycle |
| `runtime_status` | Companion runtime blocked/degraded/available diagnostics |
| `capability_matrix` | Account-scoped operation status and explanations |
| `dialog_inspector` | Participants, phone traces, admin/member evidence |
| `message_actions` | Send/reply/forward/reaction/delete with capability gates |
| `media_gallery` | Media grouped by chat, type and local availability |
| `status_surface` | WhatsApp Status read/publish evidence surface |
| `voice_note_player` | Voice playback without automatic STT |
| `call_evidence_panel` | Metadata-only call timeline entries |
| `outbox_panel` | Command queue status, retry and dead-letter review |
| `realtime_patches` | Cache patching for `whatsapp.*` events |

## Functional Module Map

| Capability module | Назначение | Current production status |
|---|---|---|
| `accounts` | Account metadata, lifecycle and secret/session refs | MISSING |
| `runtime_fixture` | Deterministic local/test runtime | MISSING |
| `runtime_webview` | Desktop WhatsApp Web companion runtime | PARTIAL: visible shell, typed frontend bridge, runtime-panel action, event/outbox contract, metadata-only extractor contract and allowlisted runtime-event relay dispatch; live smoke missing |
| `runtime_business_cloud` | Future Meta Business provider | UNSUPPORTED |
| `dialogs` | Private/group/community/broadcast/status projection | MISSING |
| `private_chats` | 1:1 chat projection | MISSING |
| `groups` | Group chat projection and participant evidence | MISSING |
| `communities` | Community evidence and member projection | MISSING |
| `broadcasts` | Broadcast evidence projection | MISSING |
| `statuses` | WhatsApp Status source/timeline evidence | MISSING |
| `messages` | Source-backed message projection | MISSING |
| `message_versions` | Observed edits/updates | MISSING |
| `message_tombstones` | Deletes/local visibility history | MISSING |
| `replies` | Reply target projection | MISSING |
| `forwards` | Forward attribution when provider exposes it | MISSING |
| `reactions` | Reaction evidence and provider commands | MISSING |
| `participants` | Phone-centric member/admin/community evidence | MISSING |
| `identity_traces` | Phone, `wa_id`, display name and contact-card traces | MISSING |
| `media` | Media metadata, download/upload and gallery | MISSING |
| `attachments` | Communication attachment rows and local blobs | MISSING |
| `voice_notes` | Voice attachment, playback and transcript handoff | MISSING |
| `calls` | Metadata-only call evidence | MISSING |
| `search` | Local projection search | MISSING |
| `provider_search` | WhatsApp provider-side search | MISSING |
| `sync` | Chat/message/status/media sync | MISSING |
| `realtime` | Shared realtime event contracts and cache patching | MISSING |
| `outbox` | Durable provider-write command lifecycle | MISSING |
| `audit` | Redacted provider-write/lifecycle audit | MISSING |
| `ai` | Shared-engine integration points only | MISSING |

## Module Boundary Rules

WhatsApp code may depend on:

```text
Communications
Events
Timeline interfaces
Shared attachment/blob boundary
Search engine interface
Risk/enrichment candidate interfaces
Audit
Secret resolver / host vault
Capability runtime
```

WhatsApp code must not own or implement:

```text
Obligation lifecycle
Decision lifecycle
Memory lifecycle
Knowledge lifecycle
Persona Intelligence
Organization Intelligence
Project Intelligence
```

WhatsApp may produce evidence and candidates for those systems only.

## Naming Rules

Use provider/product names precisely:

- `WhatsApp Channel` for the Макошь communication channel.
- `WhatsApp Web` for the primary provider source.
- `whatsapp_web` for the provider kind.
- `whatsapp_personal` and `whatsapp_business` for account kinds.
- `whatsapp_business_cloud` only for a future Meta Business API provider.

Do not describe this package as a CRM, contact manager, task tracker, note app
or standalone WhatsApp client.
````

### `docs/integrations/whatsapp/rust-provider-research.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/rust-provider-research.md`
- Size bytes / Размер в байтах: `21943`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Rust WhatsApp Provider Research

Status: research note.
Date: 2026-06-26.

Goal: identify existing Rust projects that can reduce custom WhatsApp protocol work for Макошь.

This is not a final dependency decision. WhatsApp personal-account automation has policy and account-risk implications. Any unofficial provider must stay behind explicit owner-controlled capability gates and must not become invisible infrastructure.

## Short conclusion

Recommended direction:

```text
Primary experiment result: whatsapp-rust blocked on stable Rust compile spike
Selected compile-boundary fallback: wa-rs
Do not use: old whatsappweb-rs as foundation
Future official business provider: whatsapp-business-rs or wacloudapi
Reference only: whatsmeow / Baileys
```

Reason:

- `whatsapp-rust` is the most complete native Rust candidate found for WhatsApp Web multi-device style functionality, but `0.6.0` does not pass the current stable compile spike.
- `wa-rs` is a fork with stable Rust support and bug fixes; `0.2.0` passes a compile-only spike with SDK SQLite storage disabled.
- `whatsappweb-rs` is old and marked heavily WIP/no releases.
- `whatsapp-business-rs` and `wacloudapi` target the official Meta Business/Cloud API, which is useful later but does not solve personal WhatsApp account ingestion.
- `whatsmeow` and Baileys are mature references, but they are Go/TypeScript, not Rust.

## Spike result, 2026-06-26

Local toolchain: `rustc 1.93.1`; Макошь backend now declares
`rust-version = "1.89"` for the native WhatsApp runtime boundary.

Compile-only results:

| Candidate | Feature set | Result | Notes |
|---|---|---|---|
| `whatsapp-rust 0.6.0` | default | Failed | `wacore-binary` enables `portable_simd`, which is not available on stable. |
| `whatsapp-rust 0.6.0` | `default-features = false`, `sqlite-storage`, `tokio-transport`, `tokio-runtime`, `tokio-native`, `ureq-client`, `signal` | Failed | `wacore 0.6.0` uses experimental `if let` guards on stable Rust. |
| `wa-rs 0.2.0` | default | Passed | Useful proof of crate health, but default pulls SDK SQLite storage. |
| `wa-rs 0.2.0` | `default-features = false`, `tokio-native`, `tokio-transport`, `ureq-client` | Passed | Selected Макошь compile boundary; avoids SDK SQLite storage so session material can remain behind Макошь host-vault/runtime storage design. |
| `wa-rs 0.2.0` | `default-features = false`, `tokio-native`, `tokio-transport`, `ureq-client` on Rust 1.88 | Failed | Transitive `tokio-websockets 0.13.3` requires Rust 1.89. |
| `wa-rs 0.2.0` | `default-features = false`, `tokio-native`, `tokio-transport`, `ureq-client` on Rust 1.89 | Passed | Validated with `cargo +1.89.0 check --manifest-path backend/Cargo.toml --features whatsapp-native-md-runtime --lib`. |

Current implementation implication:

- `backend/Cargo.toml` wires `whatsapp-native-md-runtime` to optional `wa-rs`.
- The dependency is disabled by default and compiled only under the native
  runtime feature.
- Backend, Tauri and Docker development toolchain declarations now use Rust
  1.89 so the selected native transport dependency is not hidden behind a newer
  developer machine toolchain.
- The native compile feature intentionally does not make provider capabilities
  publicly available. Runtime availability stays blocked until provider-observed
  live smoke evidence flows through the Signal Hub contract.
- `backend/src/integrations/whatsapp/runtime/native_md.rs` contains a
  smoke-gated driver descriptor for the selected SDK boundary. Under the feature
  it reports `wa-rs` as present but blocked by
  `whatsapp_native_md_public_availability_blocked`; without the feature it
  remains `whatsapp_native_md_runtime_feature_disabled`.
- The same module now defines an account-scoped runtime actor contract. It binds
  to real `wa-rs` API types (`Bot`, `BotBuilder`, provider `Event`, storage
  `Backend`, `TransportFactory`, `HttpClient`, `Device`, `MessageInfo`,
  `PairCodeOptions`) while keeping public capability gated by smoke evidence.
  Commands are constrained to Макошь durable provider outbox claims, events to
  Signal Hub raw evidence, and provider session material to host-vault metadata
  bindings.
- `wa-rs` requires a full backend implementation before live startup: the
  backend must implement `SignalStore`, `AppSyncStore`, `ProtocolStore` and
  `DeviceStore`. `native_md` now provides `NativeMdHostVaultBackend` for those
  store families and persists their secret material as an encrypted,
  account-scoped host-vault snapshot under the `whatsapp_web_session_key`
  binding, with SDK SQLite disabled and PostgreSQL secret payloads forbidden.
- The native adapter now also has a compile-checked client factory for the real
  `wa-rs` builder surface. `NativeMdWaRsClientFactory::configured_builder`
  wires `NativeMdHostVaultBackend`, `TokioWebSocketTransportFactory`,
  `UreqHttpClient`, optional `PairCodeOptions` and a sanitized event-handler
  DTO path into `wa_rs::bot::BotBuilder`. The factory deliberately returns the
  builder and does not call `build()` in health/compile paths.
- `NativeMdLiveDriver` now compile-checks the live lifecycle surface around
  that builder: `build().await`, `Bot::run().await`, `Client::disconnect().await`
  and task abort cleanup. Its event path accepts only owned sanitized DTOs
  through the shared `WhatsAppRuntimeEventSink`, preserving the
  runtime-to-event-spine boundary.
- `WhatsappRuntimeSignalIngestService` is now the first real writer behind that
  sink contract. It persists sanitized native DTOs as append-only raw
  communication evidence and dispatches them through Signal Hub accepted events
  without importing the native provider SDK outside the adapter boundary.
- `native_md` now has a smoke-gated account manager behind
  `WhatsAppProviderRuntime` lifecycle hooks. It requires explicit
  `native_md_live_smoke_enabled` account config plus a restored
  `whatsapp_web_session_key` host-vault binding before starting the
  feature-gated driver, and reports manager state in runtime health metadata.
- QR/pair-code startup is now vault-aware through `WhatsAppProviderRuntime`.
  The native manager can create the `whatsapp_web_session_key` host-vault
  bootstrap snapshot and account binding before starting the feature-gated
  driver.
- QR/pair-code artifacts are now captured from provider-observed
  `PairingQrCode` / `PairingCode` events into a one-time in-memory start
  response channel. Sanitized runtime DTOs and health payloads still redact the
  raw artifacts; PostgreSQL, Signal Hub events and logs do not receive them.
- Startup restore reconciliation now attempts eligible native runtime startup
  from the account-scoped host-vault `whatsapp_web_session_key` binding through
  `WhatsAppProviderRuntime::start_runtime`. The attempt remains smoke-gated and
  produces only sanitized status/session events.
- Native reconnect policy is now account-scoped and tick-driven. Provider
  connection failures schedule bounded reconnect from the same vault-bound
  session, provider-observed `Connected` emits recovered lifecycle evidence,
  and manager restart attempts emit sanitized lifecycle events through the same
  `WhatsAppRuntimeEventSink`.
- `native_md` also has a feature-gated `wa-rs` event classifier. It maps real
  `wa-rs::types::events::Event` variants to Макошь raw record kinds and
  accepted Signal Hub event families, including protobuf inspection for message
  reactions, media, calls, edits and deletes. Unknown/raw provider notifications
  stay as unsupported runtime evidence rather than being discarded.
- Classified native events now pass through a `NativeMdRawEvidenceEnvelope`
  contract that records provider shape, runtime driver, raw record kind, raw
  Signal Hub event kind, accepted event kind and stable source-fingerprint seed.
  The envelope is sanitized-metadata-only and explicitly excludes session
  material, tokens, cookies, raw secrets, message bodies and media bytes.
- A compile-checked `NativeMdSanitizedProviderEventDto` now sits after
  classification/envelope construction for real `wa-rs::types::events::Event`
  values. It keeps metadata needed for idempotency and later projection
  routing, including ids, JIDs, timestamps, presence/receipt state, sync counts
  and payload-shape flags, while excluding QR codes, pair codes, raw SDK nodes,
  protobuf action payloads, history-sync payloads, about text, push names,
  session material, message bodies and media bytes. The DTO also carries a
  compile-checked dispatch target for the existing runtime-bridge endpoint
  family, so the future native actor has a fixed event-spine path instead of an
  ad-hoc direct domain call.
- Native session restore is still required to use the account-scoped
  `whatsapp_web_session_key` host-vault binding. The four `wa-rs` store
  families, concrete builder wiring, live driver lifecycle and smoke-gated
  account manager, vault-aware link startup, transient QR/pair-code response
  channel, startup restore attempt, reconnect policy and verified-subset command
  execution boundary now compile against the selected SDK; manual smoke plus
  media/status/archive/mute/pin/join/unread/forward command coverage are still
  pending.
- Runtime health now reports the verified SDK submission subset and unsupported
  live command set explicitly, so SDK presence cannot be misread as full command
  availability.

## Candidate comparison

| Project | Language | Provider type | Strengths | Risks | Макошь recommendation |
|---|---|---|---|---|---|
| `oxidezap/whatsapp-rust` / `whatsapp-rust` crate | Rust | Unofficial WhatsApp Web/native multi-device protocol | Broad feature set: auth, E2E messaging, media, groups, communities, status, contacts, presence, chat actions, privacy; modular storage/transport/runtime | Unofficial; ToS/account risk; protocol drift; needs adapter isolation | Use as first experimental native provider behind feature flag and ADR. |
| `homun-app/wa-rs` / `wa-rs` crate | Rust | Fork of `whatsapp-rust` | Stable Rust support claim, QR/pair-code, persistent sessions, messaging/media/groups/presence; MIT | Very small history compared with upstream; fork divergence | Evaluate if upstream requires nightly or breaks current toolchain. |
| `wiomoc/whatsappweb-rs` / `whatsappweb` crate | Rust | Older WhatsApp Web reverse-engineered client | Some text/media/group/contact features | Marked heavily WIP, no releases, old architecture | Do not use as foundation. Reference only if some protocol detail is useful. |
| `veecore/whatsapp-business-rs` | Rust | Official Meta WhatsApp Business Platform SDK | Type-safe Business Platform SDK: messages, webhooks, WABA, catalogs, onboarding flows | Business-only; does not support personal account history; policy/template semantics differ | Use only for future `whatsapp_business_cloud`. |
| `wacloudapi` | Rust | Official Meta WhatsApp Cloud API SDK | Type-safe async SDK; messages, media, templates, phone numbers, products, flows, analytics, QR, webhooks | v0.1.0; business-only; not a personal local-first provider | Evaluate for future official cloud provider. |
| `tulir/whatsmeow` | Go | Unofficial WhatsApp Web multi-device library | Mature reference implementation with many core features | Go dependency/sidecar or rewrite needed; MPL-2.0 license | Reference architecture/protocol behavior only. Avoid Go sidecar unless Rust path fails. |
| `WhiskeySockets/Baileys` | TypeScript | Unofficial socket-based WhatsApp Web library | Very mature ecosystem reference | Node sidecar; JS runtime; protocol drift; license/ops overhead | Reference only, not target for Макошь Rust backend. |

## Recommended provider architecture

Do not embed a third-party WhatsApp library directly into domain code.

Use a strict adapter boundary:

```text
whatsapp-rust / wa-rs
  -> integrations/whatsapp/runtime/native_md
  -> integrations/whatsapp/adapter
  -> platform/contracts/events
  -> platform/events or observations
  -> Communications consumer/projection
```
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/whatsapp/status.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/whatsapp/status.md`
- Size bytes / Размер в байтах: `49608`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# WhatsApp Implementation Status

Статус на 2026-06-27.

Это сводный статус по текущей цели: *runtime-первый слой + fixture-полумеханизм*, не *готовый production live WhatsApp-провайдер*.

Invariant remains:

- Канал не является доменом.
- WhatsApp — integration.
- Communication — доменный объект.
- Все факты идут как source evidence и затем проектируются в Communications.

Implementation evidence checkpoints and remaining closure gates:

```text
IMPLEMENTED CHECKPOINTS = 67
DOMAIN CLOSURE          = not achieved
LIVE BLOCKERS           = manual smoke, remaining safe write APIs,
                          WebView live smoke,
                          Business Cloud public exposure/smoke
```

Blocked closure mark on 2026-06-27:

- The current WhatsApp goal is **blocked**, not complete.
- `make whatsapp-domain-closure-audit` is expected to pass only as an honesty
  check while reporting `closure_achieved = false`.
- Closure is blocked by missing sanitized live-smoke evidence artifacts for
  `whatsapp_native_md`, `whatsapp_web_companion` and
  `whatsapp_business_cloud`.
- Native MD also still lacks verified safe provider APIs and smoke evidence for
  `archive`, `unarchive`, `mute`, `unmute`, `pin`, `unpin`, `mark_unread`,
  `join_group` and `publish_status`.
- Do not claim WhatsApp domain closure until the live smoke artifacts validate
  and `make whatsapp-domain-closure-gate` passes.

Реализация уже есть (fixture/runtime-safe foundation) для:

1. `provider/account model` — `whatsapp_web`, `whatsapp_business_cloud`, `provider_shape` (`whatsapp_web_companion` / `native_md` / `business_cloud`), session metadata, account lifecycle state transitions.
   Fixture account setup can now also pin `provider_shape = whatsapp_native_md`
   on top of the compatibility `whatsapp_web` provider kind, so the native MD
   runtime boundary is no longer testable only as a blocked live shape.
2. `host-vault boundary` — привязка сессии через `whatsapp_web_session_key`, перезапись/очистка при `revoke/relink/remove`, восстановление старта без ручного вмешательства.
   Повторная успешная авторизация того же аккаунта теперь трактуется как
   `session_rotated`: тот же account-scoped secret binding сохраняется, payload
   в host vault перезаписывается, а lifecycle/runtime events больше не выглядят
   как первичная авторизация.
3. `runtime health diagnostics` — sanitized `runtime/health` теперь различает
   `available` / `degraded` / `blocked` и отдает вложенные diagnostics-блоки
   по `session`, `storage`, `runtime`, `webview` и `validation` без утечки
   session material.
4. `capability contract` — capability-маршруты и состояния `available/degraded/blocked/unsupported`.
5. `fixture/manual runtime` — детерминированный API для message/status/media/media commands/reactions/receipt/dialog/participant/call/presence.
   The same fixture/runtime-safe path can now exercise the `whatsapp_native_md`
   provider shape through account metadata and runtime status/session surfaces,
   not only `whatsapp_web_companion`.
6. `provider-write command model` — durable outbox + reconciler + retry/retry-policy + dead-letter + audit-safe events.
   Live runtime-bridge claim дополнительно отсекает команды без
   `session_restore_available = true`, а также `fixture`, `live_blocked`,
   empty-runtime и unlinked lifecycle states, даже если stale command row
   ошибочно выглядит `queued` / `available`.
7. `message ingestion/projection` — raw/signals → accepted → `communication_messages`, `communication_conversations`, `communication_identities`, tombstones/versions/reactions/attachments и raw evidence.
8. `fixture commands` — send/reply/forward/edit/delete/reactions/media/media-download/media-upload/status publish/media status, dialog state, call/join/leave, voice.
9. `realtime` — sanitized websocket/event-log events для message/dialog/status/reaction/receipt/runtime/call/presence.
10. `frontend workbench` — runtime panel, command audit/retry/dead-letter UI and fixture control surfaces.
11. `timeline/search/shared-engine hooks` — событийному потоку доступны нужные trace-цепочки.
12. `telemetry/event evidence` — наблюдательные события и projection-пути сведены в event-sourcing spine.
13. `native runtime compile boundary` — `whatsapp-rust 0.6.0` провален на stable compile spike, fallback `wa-rs 0.2.0` выбран как optional dependency за `whatsapp-native-md-runtime` без SDK SQLite storage. Rust 1.88 провален из-за `tokio-websockets`, поэтому MSRV поднят до Rust 1.89 и проверен через `cargo +1.89.0 check`. Compile feature не является public capability flag: `native_md` и `business_cloud` live availability остаются blocked до smoke/live evidence. При feature-сборке `native_md` отдает `wa-rs` smoke-gated descriptor с readiness `smoke_gated_unverified_public_blocked` и blocker `whatsapp_native_md_public_availability_blocked`; без feature остается `whatsapp_native_md_runtime_feature_disabled`. Session restore привязан к account-scoped `whatsapp_web_session_key` в host vault.
14. `native runtime actor contract` — `native_md` now has an explicit account-scoped actor contract over the selected `wa-rs` API surface. The contract fixes the command channel to the durable provider outbox, the event sink to Signal Hub raw evidence, the storage boundary to host-vault metadata bindings only, and the event-family matrix for auth/runtime/sync/messages/updates/deletes/receipts/reactions/dialogs/participants/presence/calls/status/media/command reconciliation/unsupported evidence. Public live capability remains blocked until manual live smoke and provider-observed evidence exist.
15. `native wa-rs event classifier contract` — `native_md` now has a feature-gated classifier for real `wa-rs::types::events::Event` variants. It maps auth, connection, sync, message, receipt, presence, dialog/group, participant/contact and unknown/provider-only events to Макошь raw record kinds and accepted Signal Hub event families. `Event::Message` also inspects protobuf reaction/media/call/edit/delete markers plus `MessageInfo.edit`, so reactions, edits, deletes and media metadata are not collapsed into generic messages. Raw `Notification` and `BusinessStatusUpdate` become unsupported runtime evidence, not dropped.
16. `native raw evidence envelope contract` — classified `wa-rs` events now flow through a compile-only `NativeMdRawEvidenceEnvelope` contract before any future writer can append them. The envelope fixes `provider_shape = whatsapp_native_md`, `runtime_driver = wa-rs`, raw record kind, `signal.raw.whatsapp.*.observed` event kind, accepted Signal Hub kind, stable `source_fingerprint:v5` seed, and sanitized payload policy. It explicitly forbids session/token/cookie/raw secrets, message bodies and media bytes in runtime metadata/events/log-like payloads.
17. `native sanitized inbound DTO contract` — feature-gated `native_md` now builds a compile-checked `NativeMdSanitizedProviderEventDto` for real `wa-rs::types::events::Event` values. The DTO pairs the raw-evidence envelope with metadata-only provider details: message ids, JIDs, timestamps, receipt/presence states, sync counters and safe payload-shape flags. It also fixes the dispatch target to the existing `/api/v1/integrations/whatsapp/runtime-bridge/*` family for messages, updates, deletes, receipts, reactions, media, media lifecycle, statuses, status views/deletes, presence, calls, dialogs, participants, runtime events and sync lifecycle. It deliberately excludes QR codes, pair codes, raw `Node`, protobuf action payloads, history-sync payloads, about text, push names, session material, message bodies and media bytes before any future live producer can append Hub evidence.
18. `native runtime health surface` — `runtime/health` for `whatsapp_native_md` now includes the native driver descriptor in `checks.native_md_driver` and `checks.runtime.native_driver`: driver id/readiness, live-runtime blocker, account-scoped actor scope, durable outbox command channel, Signal Hub raw-evidence sink, host-vault session purpose and metadata-only database policy. QR/pair-code and provider-command blockers also include the provider-shape blocker, so native auth/write surfaces cannot look publicly available while the live driver is smoke-gated or feature-disabled.
19. `native wa-rs host-vault backend` — `native_md` now records the real `wa-rs` runtime builder prerequisites in the compile-checked actor contract: `Backend`, `TransportFactory`, `HttpClient`, wrapper `Device`, core serializable `Device` and `PairCodeOptions`. The adapter also implements the required `wa-rs` store families (`SignalStore`, `AppSyncStore`, `ProtocolStore`, `DeviceStore`) as `NativeMdHostVaultBackend`: account-scoped encrypted host-vault snapshot under `whatsapp_web_session_key`, SDK SQLite disabled, and PostgreSQL secret payloads forbidden.
20. `native wa-rs client factory` — `native_md` now has a compile-checked `NativeMdWaRsClientFactory::configured_builder` that wires `NativeMdHostVaultBackend`, `TokioWebSocketTransportFactory`, `UreqHttpClient`, optional pair-code options and a sanitized event handler into `wa_rs::bot::BotBuilder`. The event handler derives a stable provider event id from sanitized metadata and builds the same sanitized DTO path used by the Signal Hub fixture contract. This still does not make live native execution publicly available; manual live smoke and remaining capability coverage stay blocked.
21. `native wa-rs live driver lifecycle` — `native_md` now has a compile-checked `NativeMdLiveDriver` entrypoint that builds the configured `wa_rs::bot::Bot`, starts it through `Bot::run()`, and stops it through `Client::disconnect()` plus task abort cleanup. Inbound events are converted into owned sanitized DTOs and handed to the shared `WhatsAppRuntimeEventSink` contract, not to Communications/Personas/etc. directly. Public native runtime availability remains blocked until manual live smoke and the remaining live capability matrix pass.
22. `native runtime Signal Hub sink` — `WhatsappRuntimeSignalIngestService` now implements the shared sink contract for sanitized native runtime DTOs. It records append-only `communication_raw_records`, redacts secret-like metadata recursively, dispatches `signal.raw.whatsapp.*.observed`, verifies the resulting `signal.accepted.whatsapp.*` kind and stays idempotent for duplicate provider-observed events. This is an application-level event-spine writer; it still does not enable public live runtime by itself.
23. `native runtime smoke manager` — `native_md` now has an account-scoped runtime manager wired through `WhatsAppProviderRuntime` lifecycle hooks. It can start the feature-gated live driver only when the account config explicitly opts into `native_md_live_smoke_enabled` and a `whatsapp_web_session_key` host-vault binding exists. Health exposes `checks.native_md_manager` / `checks.runtime.native_manager` with manager wiring, opt-in, feature, running, link-start vault binding, reconnect policy and public-availability gate metadata. This is a controlled smoke path, not a capability flag; public availability remains blocked until manual live smoke and the remaining live capability matrix pass.
24. `native vault-aware link startup` — `WhatsAppProviderRuntime::start_qr_link` and `start_pair_code_link` now receive `SecretReferenceStore` and `HostVault` context. For `whatsapp_native_md`, the smoke manager can create a metadata-only secret reference plus `whatsapp_web_session_key` host-vault bootstrap snapshot before starting the feature-gated driver; pair-code startup passes the phone number into `wa_rs::bot::BotBuilder::with_pair_code`. Without explicit smoke opt-in the API stays blocked and returns no QR/pair-code artifact.
25. `native transient auth artifact channel` — feature-gated `native_md` now captures `wa-rs` `PairingQrCode` / `PairingCode` events in an in-process, account-scoped, one-time transient channel. The runtime start response can expose QR SVG or pair code with expiry after the live driver
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/integrations/yandex-telemost/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/README.md`
- Size bytes / Размер в байтах: `3663`
- Included characters / Включено символов: `3645`
- Truncated / Обрезано: `no`

````markdown
# Макошь Communications - Yandex Telemost Provider Stage

Status: `FOUNDATION_PATCH_APPLIED`, 2026-06-28.

Yandex Telemost is an external communication provider adapter. It is not a
Макошь domain, not a calendar source of truth and not a meeting CRM. Telemost
can provide conference metadata, join links, cohost metadata, local desktop
recording artifacts, WebView speaker-timeline hints and provider runtime
signals.

Invariant: A provider is never a domain. A conference is provider evidence. The
business object belongs to Calendar, Communications, Calls, Radar, Timeline,
Documents or another owner domain/workflow.

```text
Yandex Telemost Provider
  -> Provider Runtime API
  -> Visible Desktop WebView
  -> Local MP3 Recorder
  -> Speaker Timeline Hint Files
  -> Canonical Integration Events
  -> Shared Workflows and Engines
```

## Foundation scope

The Yandex Telemost foundation provides:

- provider kind `yandex_telemost_user`;
- secret purpose `yandex_telemost_oauth_token`;
- HostVault-backed OAuth token storage and provider-account secret binding;
- provider API client for conference create/read/update;
- provider API client for cohost list reads;
- matched Telemost cohost observations projected into Calendar `event_participants`;
- sanitized integration events with causation/correlation-ready envelopes;
- runtime status and capability surface;
- backend routes under `/api/v1/integrations/yandex-telemost/*`;
- frontend integration API, query keys and settings panel;
- desktop Tauri command for opening a conference in a visible Макошь WebView;
- local desktop recorder command that writes `audio.mp3` through `ffmpeg`;
- local speaker timeline hint files: `speaker-timeline.jsonl` and
  `speaker-timeline.txt`;
- owner-visible retention policy for local MP3 and speaker hint artifacts;
- explicit consent gate before any local recording starts.

## Current scope

```text
target available:
  account setup through token or token secret ref
  runtime status
  conference create/read/update
  cohost list read
  visible WebView open
  local MP3 recording start/stop
  WebView-derived speaker timeline hints
  sanitized integration events

unsupported until later:
  hidden capture
  automatic meeting join
  provider webhooks
  provider-side recording download
  provider-side transcript download
  treating WebView speaker hints as truth
  installing macOS/windows kernel audio drivers silently
```

## Provider kind

```text
yandex_telemost_user
```

## Secret purpose

```text
yandex_telemost_oauth_token
```

Domains store only references and lifecycle state. Raw OAuth tokens stay in
HostVault and never appear in event payloads, settings JSON or frontend state.

## Local recording artifacts

A recording session writes files under:

```text
app_data_dir/telemost-recordings/{account_id}/{recording_session_id}/
├── audio.mp3
├── speaker-timeline.jsonl
└── speaker-timeline.txt
```

`audio.mp3` is the later transcription source. The speaker timeline files are
only diarization hints for Whisper-side processing. They can help estimate the
number of speakers and rough speaking intervals, but they are not a source of
truth.

## Navigation

- [Architecture](./architecture.md)
- [API](./api.md)
- [Modules](./modules.md)
- [Local recording](./local-recording.md)
- [Implementation plan](./implementation-plan.md)
- [Live smoke checklist](./live-smoke-checklist.md)
- [Status](./status.md)
- [Realtime Conversation Platform](../../platform/realtime-conversation/README.md)
- [Call Intelligence Engine](../../engines/call-intelligence/README.md)
- [Speaker Identity Engine](../../engines/speaker-identity/README.md)
````

### `docs/integrations/yandex-telemost/api.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/api.md`
- Size bytes / Размер в байтах: `7681`
- Included characters / Включено символов: `7681`
- Truncated / Обрезано: `no`

````markdown
# Yandex Telemost API Surface

## Backend routes

```text
GET  /api/v1/integrations/yandex-telemost/capabilities
GET  /api/v1/integrations/yandex-telemost/accounts
POST /api/v1/integrations/yandex-telemost/accounts
GET  /api/v1/integrations/yandex-telemost/runtime/status?account_id={account_id}
POST /api/v1/integrations/yandex-telemost/accounts/{account_id}/retention/prune
POST /api/v1/integrations/yandex-telemost/conferences
GET  /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}
PATCH /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}
GET  /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}/cohosts
POST /api/v1/integrations/yandex-telemost/webview/manifest
POST /api/v1/integrations/yandex-telemost/recording/intent
POST /api/v1/integrations/yandex-telemost/runtime-bridge/recordings
POST /api/v1/integrations/yandex-telemost/runtime-bridge/transcripts
```

## Account setup

```json
{
  "account_id": "telemost-main",
  "display_name": "Yandex Telemost",
  "external_account_id": "user@yandex.ru",
  "oauth_token": "<redacted>",
  "oauth_token_ref": null,
  "api_base_url": "https://cloud-api.yandex.net/v1/telemost-api",
  "metadata": { "source": "settings_panel" }
}
```

Either `oauth_token` or `oauth_token_ref` must be supplied. If `oauth_token` is
present, it is stored in HostVault and bound under
`yandex_telemost_oauth_token`.

## Create conference

```json
{
  "account_id": "telemost-main",
  "body": {
    "waiting_room_level": "PUBLIC",
    "cohosts": [{ "email": "cohost@yandex.ru" }],
    "is_auto_summarization_enabled": true,
    "metadata": { "source": "calendar_workflow" }
  }
}
```

The provider payload drops `metadata` before sending to Yandex. Metadata remains
local provenance.

## WebView manifest

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "join_url": "https://telemost.yandex.ru/j/abcdef",
  "display_name": "Макошь Owner"
}
```

The backend returns the expected visible WebView contract. The actual window is
opened by the Tauri command `open_yandex_telemost_companion`.

## Tauri commands

```text
open_yandex_telemost_companion
yandex_telemost_companion_manifest
yandex_telemost_prepare_audio_device
yandex_telemost_recording_start
yandex_telemost_recording_stop
yandex_telemost_speaker_timeline_append
```

`yandex_telemost_recording_start` requires:

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "join_url": "https://telemost.yandex.ru/j/abcdef",
  "consent_attested": true
}
```

## Recording completion bridge

After local `ffmpeg` capture stops, the desktop client posts the completed
recording manifest back to Макошь:

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "join_url": "https://telemost.yandex.ru/j/abcdef",
  "recording_session_id": "session-123",
  "output_dir": "/.../telemost-recordings/telemost-main/session-123",
  "audio_path": "/.../audio.mp3",
  "speaker_jsonl_path": "/.../speaker-timeline.jsonl",
  "speaker_txt_path": "/.../speaker-timeline.txt",
  "started_at_epoch_ms": 1719550000000,
  "stopped_at_epoch_ms": 1719550300000,
  "consent_attested": true
}
```

Макошь validates that all paths stay under `output_dir`, materializes the
provider-neutral Call Bundle files (`manifest.json`, `meeting.json`,
`provider.json`, `participants.json`, `event-track.jsonl`,
`speaker-hints.jsonl`), publishes
`integration.yandex_telemost.local_recording.completed`, and queues the
provider-neutral realtime conversation pipeline bootstrap events.

Each Radar candidate is also captured as a provider-neutral
`REALTIME_CONVERSATION_RADAR_SIGNAL` observation and mirrored into the existing
Review Inbox with the closest existing owner-domain review kind:

- `unknown_cohosts` -> `potential_relationship`;
- `unmatched_meeting_link` -> `potential_project`;
- remaining Telemost radar artifacts -> `knowledge_candidate`.

The Call Bundle manifest also snapshots the active owner-visible retention
policy for local Telemost artifacts under `provenance.retention_policy`.

## Transcript completion bridge

After local STT finishes, the desktop/runtime side posts the transcript result
back to Макошь:

```json
{
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "bundle_id": "session-123",
  "bundle_root": "/.../telemost-recordings/telemost-main/session-123",
  "transcript_text": "Owner: ship the Telemost runtime.",
  "segments": [
    {
      "speaker": "Owner",
      "start_ms": 0,
      "end_ms": 1200,
      "text": "ship the Telemost runtime"
    }
  ],
  "language_code": "en",
  "stt_provider": "whisper-local",
  "summary": "Decision to ship the Telemost runtime.",
  "confidence": 0.91,
  "metadata": { "engine_version": "local-dev" }
}
```

Макошь validates the bundle root and manifest, writes `transcript.json`,
`transcript.md` and optional `summary.md`, updates `manifest.json`, publishes
`realtime_conversation.transcript.completed`, and projects the transcript into
the `documents` domain through the provider-neutral transcript workflow.

When the Call Bundle already carries `calendar_event_id`, the same workflow also
projects the transcript into calendar meeting state by creating
`event_transcripts` evidence and attaching the transcript to the matching
`event_recordings` row.

## Automatic local transcription execution

The `realtime_conversation.transcript.requested` payload now includes the local
runtime paths required for STT execution:

```json
{
  "bundle_id": "session-123",
  "account_id": "telemost-main",
  "conference_id": "abcdef",
  "provider_kind": "yandex_telemost",
  "bundle_root": "/.../telemost-recordings/telemost-main/session-123",
  "manifest_path": "/.../telemost-recordings/telemost-main/session-123/manifest.json",
  "audio_path": "/.../telemost-recordings/telemost-main/session-123/audio.mp3"
}
```

If `MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER` is present, Макошь runs that
local executable and passes bundle metadata through environment variables:

```text
MAKOSH_TRANSCRIPT_BUNDLE_ID
MAKOSH_TRANSCRIPT_ACCOUNT_ID
MAKOSH_TRANSCRIPT_CONFERENCE_ID
MAKOSH_TRANSCRIPT_PROVIDER_KIND
MAKOSH_TRANSCRIPT_BUNDLE_ROOT
MAKOSH_TRANSCRIPT_MANIFEST_PATH
MAKOSH_TRANSCRIPT_AUDIO_PATH
MAKOSH_TRANSCRIPT_MANIFEST_JSON
```

Optional settings:

```text
MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_ARGS_JSON='["--flag","value"]'
MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_TIMEOUT_SECONDS=900
```

The executable must emit JSON on stdout with:
`transcript_text`, `segments`, `stt_provider`, and optional
`language_code`, `summary`, `confidence`, `metadata`.

## Local artifact retention cleanup

Макошь now supports owner-visible retention cleanup for local Telemost files:

```json
{
  "remove_audio": true,
  "remove_speaker_hints": true,
  "limit": 100
}
```

The cleanup route removes expired local files according to the application
settings:

```text
privacy.yandex_telemost_recording_retention_days
privacy.yandex_telemost_speaker_timeline_retention_days
```

`0` disables automatic cleanup for that artifact class.

When a bundle is cleaned, Макошь removes:

- `audio.mp3` when the recording retention policy has expired;
- `speaker-timeline.jsonl`, `speaker-timeline.txt`, and
  `speaker-hints.jsonl` when the speaker-hint retention policy has expired;
- and records the cleanup result back into `manifest.json`.

## Provider API dependency

The Yandex provider calls use:

```text
Authorization: OAuth <token>
POST  /v1/telemost-api/conferences
GET   /v1/telemost-api/conferences/{id}
PATCH /v1/telemost-api/conferences/{id}
GET   /v1/telemost-api/conferences/{id}/cohosts?offset={offset}&limit={limit}
```
````

### `docs/integrations/yandex-telemost/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/architecture.md`
- Size bytes / Размер в байтах: `3633`
- Included characters / Включено символов: `3603`
- Truncated / Обрезано: `no`

````markdown
# Yandex Telemost Architecture

## Boundary

Yandex Telemost lives in:

```text
backend/src/integrations/yandex_telemost
frontend/src/integrations/yandexTelemost
frontend/src-tauri/src/yandex_telemost_companion.rs
```

It must not create `domains/yandex_telemost` and must not write directly to
Calendar, Communications, Calls, Radar or Documents.

## Inbound provider flow

```text
Yandex Telemost API / WebView / local recorder
↓
integrations/yandex_telemost or Tauri companion
↓
integration.yandex_telemost.* event or local artifact manifest
↓
workflow/projection
↓
Calendar / Calls / Radar / Documents
```

Matched conference evidence can enrich Calendar in two stages:

- conference events create the provider-neutral Calendar relation;
- cohost observations can populate `event_participants` for the matched
  Calendar event without making Telemost the owner of attendee truth.

## Outbound provider flow

```text
Calendar/App intent
↓
workflow or provider runtime route
↓
Yandex Telemost client command
↓
provider API result
↓
integration.yandex_telemost.conference.created/updated/observed
```

## WebView model

The desktop command opens a visible owner-controlled WebView. Hidden mode is
forbidden. The WebView may observe active-speaker-like DOM signals and forward
speaker hints to the local recorder. This signal is explicitly marked:

```text
truth_status = hint_not_truth
source = webview_dom_multi_selector_heuristic
confidence ~= 0.42
```

The hint is useful only as a warm start for diarization. Whisper or another
transcription/diarization stage must remain the evidence-producing stage.

## Local recorder model

The recorder is a Tauri-local runtime, not a backend provider API feature.

```text
visible WebView audio output
↓
local loopback/virtual audio device
↓
ffmpeg
↓
audio.mp3
```

Platform strategy:

```text
Linux:
  try to create PulseAudio/PipeWire null sink `makosh_telemost`
  record `makosh_telemost.monitor`

macOS:
  require explicit external loopback device such as BlackHole 2ch
  Макошь does not silently install system audio drivers

Windows:
  use WASAPI loopback or an explicit virtual audio cable/input
```

The recorder refuses to start unless `consent_attested=true`. This preserves
the owner-visible runtime model and prevents hidden capture from becoming part
of the provider integration.

## Events

```text
integration.yandex_telemost.authorization.completed
integration.yandex_telemost.runtime.status_changed
integration.yandex_telemost.conference.created
integration.yandex_telemost.conference.observed
integration.yandex_telemost.conference.updated
integration.yandex_telemost.cohosts.observed
integration.yandex_telemost.webview.open_requested
integration.yandex_telemost.local_recording.requested
integration.yandex_telemost.local_recording.completed
```

## Secret policy

```text
OAuth token value: HostVault only
Provider account config: secret_ref only
Event payloads: sanitized, no token/cookie/audio bytes
Frontend: no raw token after setup response
```

## Provider-neutral meeting memory

Yandex Telemost should project into the shared Meeting Platform instead of
creating Telemost-owned meeting memory. The provider opens or creates the
conference, the desktop companion captures local evidence, and the
provider-neutral Call Bundle becomes the durable object consumed by
transcription, diarization, Call Intelligence, Radar and Timeline workflows.

```text
Yandex Telemost conference
↓
visible Макошь WebView + local capture
↓
Call Bundle
↓
Call Intelligence / Speaker Identity
↓
Radar / Timeline / Knowledge Graph / Tasks candidates
```
````

### `docs/integrations/yandex-telemost/implementation-plan.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/implementation-plan.md`
- Size bytes / Размер в байтах: `2018`
- Included characters / Включено символов: `2018`
- Truncated / Обрезано: `no`

```markdown
# Yandex Telemost Implementation Plan

## Stage 1 - Foundation

- Add provider kind `yandex_telemost_user`.
- Add secret purpose `yandex_telemost_oauth_token`.
- Add integration module and error model.
- Add account setup with HostVault token storage.
- Add runtime status/capabilities route.

## Stage 2 - Conference API

- Add create conference.
- Add read conference.
- Add update conference.
- Add cohost list read.
- Emit sanitized integration events.

## Stage 3 - Desktop companion

- Add visible Telemost WebView.
- Add origin/navigation guard.
- Add WebView active-speaker hint bridge.
- Add local MP3 recorder using explicit ffmpeg process.
- Add speaker timeline JSONL/TXT output.
- Add consent gate before recording.

## Stage 4 - Provider-neutral projection

- Listen to `integration.yandex_telemost.conference.*` events.
- Project conference evidence into provider-neutral Calls/Calendar link model.
- Create Radar signals for unmatched meeting links, live streams, unknown cohosts
  and local recording artifacts.
- Mirror Radar signals into the Review Inbox as observation-backed candidates
  before any owner-domain promotion.
- Stamp matched `calendar_event_id` into the Call Bundle when the conference URL
  resolves to an existing calendar event, so later transcript/recording workflows
  do not emit false unmatched-link signals.

## Stage 5 - Transcription workflow

- Import `audio.mp3` as document/call evidence.
- Run transcription/diarization.
- Use `speaker-timeline.jsonl` and `speaker-timeline.txt` only as hints.
- Store transcript with Source, Confidence and Evidence.
- Accept transcript completion through a provider-neutral runtime bridge and
  project the completed transcript into `documents` as evidence-backed meeting memory.

## Explicit non-goals

- No hidden recording.
- No direct integration-to-domain mutation.
- No raw OAuth token in app responses.
- No belief that a DOM class called `active` is a person speaking with divine
  certainty. Computers lie. Web UIs lie more.
```

### `docs/integrations/yandex-telemost/live-smoke-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/live-smoke-checklist.md`
- Size bytes / Размер в байтах: `1620`
- Included characters / Включено символов: `1620`
- Truncated / Обрезано: `no`

````markdown
# Yandex Telemost Live Smoke Checklist

## Preconditions

```text
MAKOSH_SECRET_VAULT_KEY configured
HostVault unlocked
ffmpeg installed and available on PATH or MAKOSH_TELEMOST_FFMPEG_PATH set
Yandex OAuth token has Telemost scopes
```

## Backend smoke

1. `GET /api/v1/integrations/yandex-telemost/capabilities`
2. `POST /api/v1/integrations/yandex-telemost/accounts`
3. `GET /api/v1/integrations/yandex-telemost/runtime/status?account_id={account_id}`
4. `POST /api/v1/integrations/yandex-telemost/conferences`
5. `GET /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}`
6. `PATCH /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}`
7. `GET /api/v1/integrations/yandex-telemost/conferences/{account_id}/{conference_id}/cohosts`

## Desktop smoke

1. Open `open_yandex_telemost_companion` with a valid join URL.
2. Confirm a visible WebView appears.
3. Start recording with `consent_attested=true`.
4. Speak in the meeting.
5. Confirm files are created:

```text
audio.mp3
speaker-timeline.jsonl
speaker-timeline.txt
```

6. Stop recording.
7. Import `audio.mp3` into the future transcription workflow.
8. Use timeline hints as diarization hints only.

## Platform audio notes

Linux:

```text
yandex_telemost_prepare_audio_device
route WebView output to makosh_telemost
record makosh_telemost.monitor
```

macOS:

```text
install/configure BlackHole 2ch or equivalent manually
set MAKOSH_TELEMOST_FFMPEG_INPUT if ffmpeg needs a device index
```

Windows:

```text
configure WASAPI loopback or virtual audio cable
set MAKOSH_TELEMOST_FFMPEG_INPUT when needed
```
````

### `docs/integrations/yandex-telemost/local-recording.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/local-recording.md`
- Size bytes / Размер в байтах: `4668`
- Included characters / Включено символов: `4634`
- Truncated / Обрезано: `no`

````markdown
# Yandex Telemost Local Recording Contract

## Purpose

The local recorder captures the audio that reaches the owner-visible Telemost
WebView and stores it as later transcription evidence. It is a desktop runtime
feature, not a Telemost cloud API feature and not a backend secret boundary.

```text
visible Telemost WebView
↓
owner-configured loopback / virtual audio source
↓
ffmpeg
↓
audio.mp3
↓
transcription + diarization workflow
```

## Consent gate

`yandex_telemost_recording_start` refuses to start unless:

```json
{
  "consent_attested": true
}
```

Макошь does not start hidden captures, does not join a conference in the
background and does not silently install system audio drivers.

## Output layout

Each recording session writes:

```text
app_data_dir/telemost-recordings/{account_id}/{recording_session_id}/
├── audio.mp3
├── speaker-timeline.jsonl
└── speaker-timeline.txt
```

`audio.mp3` is the source artifact for Whisper or another transcription engine.
The timeline files are hints only.

## Speaker timeline hint format

`yandex_telemost_speaker_timeline_append` appends one JSON line per WebView
active-speaker observation:

```json
{
  "observed_at_epoch_ms": 1780000000000,
  "speaker_label": "Alice",
  "confidence": 0.42,
  "source": "webview_dom_multi_selector_heuristic",
  "truth_status": "hint_not_truth"
}
```

The adjacent text file mirrors this as tab-separated rows:

```text
epoch_ms    speaker_label    event          confidence       source
1780000000  Alice            speaker_hint   confidence=0.42  source=webview_dom_multi_selector_heuristic
```

These rows are not authoritative. They only help the downstream diarization
stage estimate speaker count and rough speaker turns before Whisper/audio-based
analysis corrects or rejects the WebView hint.

The desktop companion now scans multiple DOM patterns instead of a single text
match. It prefers:

- explicit speaking/activity attributes such as `data-speaking`,
  `data-speaker-active` and `data-active-speaker`;
- participant-oriented containers or test ids;
- nearby participant-name fields such as `data-participant-name`,
  `data-display-name`, `aria-label` and `title`.

This remains a heuristic path. Макошь still records the result as
`truth_status=hint_not_truth`.
The current companion source label is
`webview_dom_multi_selector_heuristic`.

## Platform strategy

### Linux

`yandex_telemost_prepare_audio_device` attempts to create a PulseAudio/PipeWire
null sink:

```text
makosh_telemost
```

The recorder defaults to:

```text
makosh_telemost.monitor
```

The user still needs to route the Telemost WebView audio output to that sink.

### macOS

macOS requires an explicitly configured loopback device, for example BlackHole
2ch. Макошь reports the requirement and records from the device selected through
`MAKOSH_TELEMOST_FFMPEG_INPUT` or the command argument.

### Windows

Windows uses an explicit WASAPI/virtual-device path selected through
`MAKOSH_TELEMOST_FFMPEG_INPUT` or the command argument.

## Environment variables

```text
MAKOSH_TELEMOST_FFMPEG_PATH   optional ffmpeg binary path
MAKOSH_TELEMOST_FFMPEG_INPUT  optional platform-specific ffmpeg input selector
```

## Retention policy

Local Telemost artifacts now follow owner-visible application settings:

```text
privacy.yandex_telemost_recording_retention_days
privacy.yandex_telemost_speaker_timeline_retention_days
```

These settings control automatic cleanup for:

- `audio.mp3`;
- `speaker-timeline.jsonl`;
- `speaker-timeline.txt`;
- and the provider-neutral copied hint file `speaker-hints.jsonl`.

Макошь snapshots the active retention policy into the Call Bundle manifest and
an hourly backend cleanup pass removes expired files. A manual cleanup route is
also available through
`POST /api/v1/integrations/yandex-telemost/accounts/{account_id}/retention/prune`.

## Event/projection policy

Local recording artifacts should later be imported through a provider-neutral
workflow:

```text
local recording receipt
↓
document/call evidence import
↓
transcription/diarization
↓
transcript with Source, Confidence, Evidence
↓
Calendar / Calls / Radar / Timeline projections
```

The recorder itself does not mutate Calendar, Calls or Radar directly. It owns
only the local runtime process and local artifact manifest.

The current backend contract exposes a transcript completion bridge. The local
runtime can hand the finished STT result back to Макошь, which writes
`transcript.json` and `transcript.md`, emits
`realtime_conversation.transcript.completed`, and lets the provider-neutral
workflow project the transcript into durable meeting memory.
````

### `docs/integrations/yandex-telemost/modules.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/modules.md`
- Size bytes / Размер в байтах: `1604`
- Included characters / Включено символов: `1526`
- Truncated / Обрезано: `no`

````markdown
# Yandex Telemost Modules

## Backend

```text
backend/src/integrations/yandex_telemost/
├── mod.rs
├── runtime.rs
└── client/
    ├── mod.rs
    ├── errors.rs
    ├── models.rs
    ├── store.rs
    └── validation.rs
```

Responsibilities:

```text
client/models.rs      DTOs, provider kind constants, capability contract
client/errors.rs      typed provider runtime errors
client/validation.rs  URL/token/payload validation and sanitizer
client/store.rs       provider account setup, HostVault binding, API calls, event publication
runtime.rs            runtime-facing reexports
```

## App routes

```text
backend/src/app/provider_runtime_handlers/yandex_telemost.rs
backend/src/app/handlers/yandex_telemost.rs
```

Routes are provider runtime/setup routes only. They are not Calendar or Calls
business routes.

## Desktop runtime

```text
frontend/src-tauri/src/yandex_telemost_companion.rs
```

Responsibilities:

```text
visible WebView open
allowed-origin navigation guard
WebView active-speaker heuristic bridge
local virtual/loopback audio preparation
ffmpeg MP3 recording start/stop
speaker timeline JSONL/TXT append
```

## Frontend

```text
frontend/src/integrations/yandexTelemost/
├── api/yandexTelemost.ts
├── components/YandexTelemostSettingsPanel.vue
├── queries/yandexTelemostQueryKeys.ts
├── queries/useYandexTelemostRuntimeQuery.ts
└── types/yandexTelemost.ts
```

The settings panel may create conferences and launch the visible companion
WebView. Product-level meeting views must remain provider-neutral.
````

### `docs/integrations/yandex-telemost/status.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/yandex-telemost/status.md`
- Size bytes / Размер в байтах: `1960`
- Included characters / Включено символов: `1960`
- Truncated / Обрезано: `no`

````markdown
# Yandex Telemost Status

Status: `FOUNDATION_PATCH_APPLIED`, 2026-06-28.

## Proposed code paths

```text
backend/src/integrations/yandex_telemost
backend/src/app/provider_runtime_handlers/yandex_telemost.rs
frontend/src/integrations/yandexTelemost
frontend/src-tauri/src/yandex_telemost_companion.rs
```

## Validation state

The source patch has been applied to the current repository structure. Local
validation must use the repository-configured tooling and package manager.

Required Telemost-domain validation:

```text
git diff --check
make architecture-check
make code-boundaries-check
make backend-fmt-check
make backend-clippy
cargo test --manifest-path backend/Cargo.toml --lib app::provider_runtime_handlers::yandex_telemost::tests::unknown_cohosts_review_item_uses_relationship_flow -- --exact
cargo test --manifest-path backend/Cargo.toml --lib app::provider_runtime_handlers::yandex_telemost::tests::unmatched_meeting_link_review_item_uses_project_flow -- --exact
cargo test --manifest-path backend/Cargo.toml --lib workflows::yandex_telemost_calendar_matching::tests::telemost_cohosts_are_projected_into_matched_calendar_event_participants -- --exact
cd frontend && pnpm lint
cd frontend && pnpm typecheck
cargo test --manifest-path frontend/src-tauri/Cargo.toml yandex_telemost_companion::tests::initialization_script_contains_multi_selector_speaker_heuristics -- --exact
```

Broader repository suites such as `make backend-test` and `cd frontend &&
pnpm test:unit` remain useful CI signals, but they are not treated here as
Telemost-domain completion gates because they currently include unrelated
modules and unrelated failing boundary tests outside the Telemost scope.

## Known follow-up work

- No known Telemost-domain follow-up gaps remain from the available documentation and implemented provider/runtime boundary. Remaining unsupported items in README are explicit non-goals or later-scope capabilities, not foundation-domain gaps.
````

### `docs/integrations/zoom/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/integrations/zoom/README.md`
- Size bytes / Размер в байтах: `7341`
- Included characters / Включено символов: `7341`
- Truncated / Обрезано: `no`

````markdown
# Макошь Communications - Zoom Provider Stage

Status: `FOUNDATION_IMPLEMENTED`, 2026-06-28.

Implementation evidence in this checkout: foundation code is present under
`backend/src/integrations/zoom` and `frontend/src/integrations/zoom`, with a
Zoom migration, backend routes, targeted backend tests and targeted frontend
tests. ADR-0102 is `Accepted` after target backend and frontend zoom validation.

Zoom in Макошь is an external communication provider adapter. It is not a
Макошь domain, not a meeting CRM and not a calendar source of truth. Zoom can
provide meeting evidence, recording evidence, transcript evidence, provider
account metadata and runtime lifecycle signals.

Invariant: A provider is never a domain. A meeting observation is evidence. The
business object belongs to Calls, Communications, Calendar, Radar, Timeline,
Documents or another owner domain/workflow.

```text
Zoom Provider
  -> Runtime Bridge
  -> Source Evidence
  -> Provider Call Projection
  -> Canonical Events
  -> Shared Workflows and Engines
```

## Foundation scope

The Zoom foundation provides:

- fixture Zoom account setup for deterministic local validation;
- live account metadata setup with `oauth_user` and `server_to_server` auth
  shapes;
- OAuth user and Server-to-Server authorization token exchange with HostVault
  credential storage and PostgreSQL secret-reference bindings only;
- explicit OAuth/S2S token refresh and renewal route that updates HostVault
  token bundles without returning raw access tokens;
- token maintenance route for scanning authorized accounts and refreshing
  expiring HostVault token bundles;
- scheduled token maintenance daemon with Signal Hub runtime gating,
  HostVault unlock gating and `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`
  operational toggle;
- managed webhook subscription status/reconcile/remove routes for authorized
  accounts, using app-owned access tokens and HostVault-backed webhook secret
  bindings;
- manual provider-sync route for authorized Zoom cloud recordings, including
  provider-neutral meeting/recording evidence ingestion plus best-effort
  recording media and transcript-like text file import;
- owner-visible privacy policy setting
  `privacy.zoom_remote_recording_download_enabled`, which must be enabled
  before Макошь fetches recording media files directly from Zoom;
- owner-visible privacy policy setting
  `privacy.zoom_remote_transcript_download_enabled`, which must be enabled
  before Макошь fetches transcript-like text files directly from Zoom;
- owner-visible retention policy settings
  `privacy.zoom_recording_import_retention_days` and
  `privacy.zoom_transcript_retention_days`, which stamp retention metadata and
  expiry intent onto imported recording blobs and transcript evidence;
- runtime status/start/stop/remove lifecycle controls;
- meeting observation ingestion into provider-neutral call evidence;
- recording observation ingestion as sanitized event evidence;
- transcript observation ingestion into call transcript persistence;
- VTT, SRT and plain text transcript file import into call transcript
  persistence after the file content has already been obtained;
- automatic transcript-like file download/import from signed
  `recording.completed` webhook payloads when Zoom includes textual recording
  files with `download_url`;
- automatic recording media download/import from signed `recording.completed`
  webhook payloads when Zoom includes non-transcript recording files with
  `download_url`;
- protected account-scoped webhook URL validation and signed
  meeting/recording webhook normalization;
- `makosh-zoom-edge-proxy` public/edge ingress that preserves raw Zoom webhook
  bodies and `x-zm-*` headers before forwarding to the protected bridge;
- canonical Zoom events with causation and correlation support;
- provider account listing with optional removed-account visibility;
- frontend API client, query keys and runtime query hook;
- read-only recording import audit route and settings-panel inspection for
  imported Zoom recording blobs;
- explicit recording import remove control for deleting imported Zoom
  recording blobs from local audit/storage state;
- explicit retention cleanup control for pruning expired imported recording
  blobs and expired transcript evidence using stamped retention expiry intent;
- scheduled retention cleanup daemon that periodically prunes expired imported
  recording blobs and transcript evidence through the same local retention
  boundary, gated by `MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED`;
- read-only account-scoped Zoom event audit route and settings-panel inspection
  for recent authorization, refresh/maintenance, runtime and bridge events;
- provider-neutral Communications `calls` and `meetings` sections that read the
  shared call evidence store and surface projected Zoom meeting/transcript
  evidence outside integration settings;
- realtime invalidation coverage for Zoom runtime and observation events;
- Signal Hub source registration for Zoom provider signals;
- downstream Signal Hub detection workflow that turns
  `zoom.meeting.observed`, `zoom.recording.observed` and
  `zoom.transcript.observed` into policy-aware `signal.raw.zoom.*` and derived
  Signal Hub detection events;
- downstream calendar-event matching and participant-identity workflows exposed
  as available capabilities rather than remaining planned-only items;
- ADR coverage for provider runtime boundary.

## Current scope

The stage is intentionally conservative:

```text
target available:
  fixture accounts
  runtime lifecycle metadata
  authorized live recording-sync worker
  recording media download/import through provider sync
  recording media download/import through signed recording webhooks
  recording import audit inspection
  recording import retention/remove control
  runtime, bridge and credential lifecycle audit inspection
  provider-neutral communications calls/meetings evidence view
  webhook subscription management
  meeting bridge ingestion
  recording bridge ingestion
  transcript bridge ingestion
  transcript file import
  protected verified webhook bridge
  public/edge webhook proxy
  OAuth/S2S authorization boundary
  explicit OAuth/S2S token refresh
  token maintenance runner
  scheduled token maintenance daemon
  retention cleanup scheduler
  token rotation policy
  owner-visible opt-in policy for remote transcript downloads
  sanitized event publication

unsupported:
  hidden recording
  automatic meeting joining without explicit setup
  auto-dialing
  model training on Zoom content by default
```

## Provider kinds

```text
zoom_user
zoom_server_to_server
```

## Secret purposes

```text
zoom_oauth_token
zoom_client_secret
zoom_webhook_secret
```

Domains store only references and lifecycle state. Raw credentials stay outside
business models and event payloads.

## Navigation

- [Architecture](architecture.md)
- [API Reference](api.md)
- [Modules](modules.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
- [Implementation Plan](implementation-plan.md)
- [Fixture Test Matrix](fixture-test-matrix.md)
- [Live Smoke Checklist](live-smoke-checklist.md)
- [Provider Runtime Research](provider-runtime-research.md)
- [Runtime Boundary ADR](../../adr/ADR-0102-zoom-provider-runtime-boundary.md)
````
