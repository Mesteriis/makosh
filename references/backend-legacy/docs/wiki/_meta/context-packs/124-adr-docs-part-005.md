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

- Chunk ID / ID чанка: `124-adr-docs-part-005`
- Group / Группа: `docs`
- Role / Роль: `adr`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `decisions/adr-index.md`

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

### `docs/adr/ADR-0100-trace-first-event-observability.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0100-trace-first-event-observability.md`
- Size bytes / Размер в байтах: `4964`
- Included characters / Включено символов: `4964`
- Truncated / Обрезано: `no`

```markdown
# ADR-0100 Trace-First Event Observability

Status: Accepted
Date: 2026-06-24

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0012 OpenTelemetry Observability
- ADR-0014 Canonical Event Envelope
- ADR-0018 Provider Adapter Boundary
- ADR-0034 Event Replay and Projection Cursors
- ADR-0095 Event-Driven Domain Communication and DLQ
- ADR-0097 Communications Channel Domains To Integrations
- ADR-0098 Provider-Neutral Communications API And Strict Boundaries
- ADR-0099 Signal Hub Event Platform

## Context

Макошь already uses an event-driven architecture with an append-only
`event_log`, `EventEnvelope`, outbox, event consumers, DLQ, Signal Hub and
provider integrations. However, events are not yet consistently usable as one
causal trace graph:

- some events have nullable or empty `correlation_id`;
- some derived events do not inherit trace context;
- observation events are not always treated as root trace events;
- Signal Hub, provider and Communications chains are not always connected;
- API and realtime surfaces do not consistently expose full trace context;
- Timeline projection and causal trace reconstruction can be confused.

Макошь needs to explain why a domain object exists without requiring Jaeger,
Tempo, Loki, Grafana or another telemetry server.

## Decision

Макошь treats canonical events as spans.

`event_id` is the span identifier.
`correlation_id` is the trace identifier.
`causation_id` is the parent span identifier.

The append-only `event_log` is the canonical trace store.

Every event written through the canonical event builder must have a non-empty
`correlation_id`.

Every event created as a consequence of another event must set
`causation_id = parent.event_id` and inherit `correlation_id` from the parent.

No separate telemetry server is required to explain why a domain object exists.

Root events may have no `causation_id`, but they must still have
`correlation_id`. Derived events must have `causation_id`. Trace reconstruction
uses only deterministic links stored in `event_log`; AI may summarize a trace
after reconstruction, but must not infer missing links.

OpenTelemetry may be used as an export or diagnostic layer. OpenTelemetry is
not the canonical trace store. The canonical trace store is `event_log`.

## Trace Semantics

| Concept | Макошь field |
|---|---|
| Trace | `correlation_id` |
| Span | `event_id` |
| Parent span | `causation_id` |
| Trace store | PostgreSQL append-only `event_log` |

Rules:

- root event: `causation_id = null`, `correlation_id` is non-empty;
- derived event: `causation_id = parent.event_id`;
- derived event: `correlation_id = parent.correlation_id`;
- legacy events with null `correlation_id` are displayed as legacy orphan
  traces unless migrated;
- consumer status, retries and DLQ records are trace annotations, not domain
  facts by themselves.

## Layer Boundary

Trace reconstruction belongs to `platform/events`.

Timeline projection belongs to `engines/timeline`.

Timeline may display trace links, but it must not be the source of truth for
trace reconstruction. A trace graph is a causal and provenance graph. A
timeline is a user or domain chronological projection.

Provider integrations remain source boundaries. Telegram, WhatsApp and Mail do
not own product-domain trace models. Communications owns canonical
communication state and emits canonical communication events with inherited
trace context.

## Consequences

Positive:

- every persistent event can be inspected as part of a causal graph;
- provider, Signal Hub, Communications and workflow chains become explainable;
- trace APIs can be implemented directly from PostgreSQL;
- Timeline Engine remains focused on chronological user/domain projection;
- OpenTelemetry can export traces without becoming source of truth.

Negative:

- builder normalization changes expectations for events that previously stored
  null `correlation_id`;
- derived event writers must pass parent context explicitly;
- legacy rows may appear as orphan traces until migrated or backfilled;
- realtime/API DTOs need trace fields even for events that older UI code did
  not display.

## Validation

The repository should enforce:

- canonical builder never produces an empty `correlation_id`;
- `TraceContext::root` and `TraceContext::child_of` preserve the rules above;
- observation capture creates a root trace event;
- raw provider/source signals are children of observation events;
- Signal Hub accepted/rejected/muted/paused events are children of raw signals;
- Communications message events inherit trace context from accepted signals;
- EventStore can return trace by event id, trace by correlation id and children
  by causation id;
- trace reconstruction does not depend on Timeline Engine;
- trace API responses include events, edges, roots, orphans, missing parents,
  consumer annotations and DLQ annotations;
- realtime event payloads expose trace fields without leaking private content.
```

### `docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md`
- Size bytes / Размер в байтах: `22411`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# ADR-0101 WhatsApp Provider Runtime Selection

Status: Accepted
Date: 2026-06-24

Accepted: 2026-06-26

Acceptance scope: this ADR accepts the provider-runtime boundary, provider
shape model and native fallback selection. It does not make any WhatsApp live
runtime publicly available. `whatsapp_native_md`, `whatsapp_web_companion` and
`whatsapp_business_cloud` remain blocked until their live-smoke evidence and
provider-observed reconciliation gates pass.

## Context

Макошь needs full WhatsApp functionality plus Макошь-specific memory, Radar, Review, Timeline, Search, AI and workflow features.

The current repository already contains a WhatsApp fixture/runtime foundation:

- provider kind `whatsapp_web`;
- fixture account/session/message ingestion;
- Signal Hub trace path;
- Communications projection;
- frontend runtime panel;
- ADR-0051 for a visible WhatsApp Web companion boundary;
- ADR-0097 stating that channels are integrations and Communications owns business state.

Full WhatsApp support requires provider capabilities beyond the current fixture foundation:

- QR/pair-code linking;
- persistent sessions;
- live inbound messages;
- media transfer;
- reactions;
- statuses;
- group/community metadata;
- provider commands;
- reconciliation;
- runtime health.

Writing all of this from scratch would mean reimplementing a large portion of WhatsApp Web/multi-device protocol behavior, session storage, E2E encryption flow, media encryption/decryption and provider event handling. That is a charming way to turn a personal OS into a protocol archaeology dig.

Research found these Rust candidates:

- `whatsapp-rust`, a broad async Rust WhatsApp Web API implementation;
- `wa-rs`, a fork of `whatsapp-rust` claiming stable Rust support;
- `whatsappweb-rs`, older and heavily WIP;
- `whatsapp-business-rs` and `wacloudapi`, Rust SDKs for official Meta Business/Cloud API.

Official Business Cloud API remains a separate business provider shape. It does not replace personal WhatsApp account support.

## Decision

Макошь will use a **multi-provider runtime boundary** for WhatsApp.

Provider shapes:

```text
whatsapp_web_companion
whatsapp_native_md
whatsapp_business_cloud
```

Compatibility:

- existing `whatsapp_web` may remain as compatibility provider kind during migration;
- new provider shape naming must be explicit in runtime capability metadata;
- no provider shape may bypass Communications ownership.
- `whatsapp_web_companion` remains blocked until an owner-visible desktop
  WebView producer passes smoke. The Tauri shell now provides the visible
  producer shell commands `open_whatsapp_web_companion` and
  `whatsapp_web_companion_manifest`, but those commands are not public
  availability flags: they open/focus an account-scoped visible
  `https://web.whatsapp.com/` WebView and return only the sanitized
  event/outbox contract. Frontend code may call these commands only through the
  typed Tauri invoke bridge, not through backend/domain HTTP APIs. Events must
  still enter protected runtime-bridge routes, writes must use the durable
  outbox, hidden/headless mode is forbidden, the companion window must not get
  domain-mutating IPC, and session/cookie/browser profile secrets must stay out
  of PostgreSQL, events, logs and health payloads.
- The current WebView companion extractor state is
  `contract_injected_relay_dispatch_available`: the desktop shell installs a
  main-frame-only, origin-guarded initialization script on
  `https://web.whatsapp.com` that exposes only a frozen metadata contract and
  an allowlisted metadata-only relay dispatch. It must not read cookies, Web
  Storage, IndexedDB, browser profile secrets, session material, message bodies
  or media bytes, and it must not call `fetch`, XHR, `postMessage` or domain
  APIs. The relay command is scoped by remote origin and account window label,
  recursively redacts secret-like/private content metadata, maps known event
  families to protected `/runtime-bridge/*` paths and posts the sanitized
  metadata observation to the protected local runtime-events bridge with
  `X-Макошь-Secret` from the Tauri process environment only. It must not mutate
  domains or complete provider commands. The WhatsApp Runtime panel exposes the
  owner-visible `Open Companion` action through the typed Tauri bridge, but
  public WebView runtime availability remains blocked until manual smoke passes.

### Primary experimental native provider

Use `whatsapp-rust` as the first Rust native multi-device provider candidate.

Conditions:

- feature-flagged;
- disabled by default;
- owner-visible opt-in;
- isolated under `backend/src/integrations/whatsapp/runtime/native_md`;
- no direct dependency from domains/workflows/engines/frontend domains;
- no session secrets in PostgreSQL/events/logs/frontend;
- successful authorization must persist account-scoped session material in host vault and bind it through `whatsapp_web_session_key`;
- no live runtime requirement in CI;
- no provider writes without durable command outbox and capability checks.

### Fallback native provider

Evaluate `wa-rs` only if `whatsapp-rust` fails the Rust 1.89/toolchain/stability spike.

Current spike result, 2026-06-26:

- `whatsapp-rust 0.6.0` does not pass the stable Rust compile spike:
  default features fail on `portable_simd`, and a no-`simd` feature set still
  fails because `wacore 0.6.0` uses experimental `if let` guards.
- `wa-rs 0.2.0` passes compile-only spike on the current local stable toolchain
  with `default-features = false` and `tokio-native`, `tokio-transport`,
  `ureq-client`.
- `wa-rs 0.2.0` with the same transport feature set fails on Rust 1.88 because
  transitive `tokio-websockets 0.13.3` requires Rust 1.89.
- Макошь backend MSRV is raised to Rust 1.89 so the native transport boundary
  can compile without weakening the selected SDK feature set.
- Макошь therefore wires the experimental `whatsapp-native-md-runtime` feature
  to optional `wa-rs` dependency for compile-boundary validation only. This does
  not make live runtime support accepted or available.
- The public runtime availability gate remains false for `native_md` and
  `business_cloud` until provider-observed live smoke evidence exists; compile
  features are not capability flags.
- `native_md` exposes a driver descriptor that reports `wa-rs` as
  smoke-gated/unverified under the feature and emits
  `whatsapp_native_md_public_availability_blocked` instead of claiming public
  live readiness. Without the feature it remains
  `whatsapp_native_md_runtime_feature_disabled`. Session restore remains
  account-scoped through the `whatsapp_web_session_key` host-vault binding.
- `native_md` now compile-checks the selected `wa-rs` storage and builder
  boundary: `NativeMdHostVaultBackend` implements the required `SignalStore`,
  `AppSyncStore`, `ProtocolStore` and `DeviceStore` families over an encrypted
  account-scoped host-vault snapshot, and
  `NativeMdWaRsClientFactory::configured_builder` wires that backend with
  `TokioWebSocketTransportFactory`, `UreqHttpClient`, optional
  `PairCodeOptions` and the sanitized event DTO path without calling
  `build()` or claiming live readiness.
- `NativeMdLiveDriver` now compile-checks the next lifecycle step: it can build
  the configured `wa_rs::bot::Bot`, start `Bot::run()`, stop through
  `Client::disconnect()` plus task abort cleanup, and route inbound provider
  events only as owned sanitized DTOs into the shared event-spine sink contract.
- `WhatsappRuntimeSignalIngestService` now implements the shared native runtime
  sink at the application boundary: sanitized DTOs become append-only raw
  communication evidence and `signal.raw.whatsapp.*.observed` /
  `signal.accepted.whatsapp.*` events without importing `wa-rs` outside the
  adapter boundary.
- `native_md` now has an account-scoped runtime manager wired behind
  `WhatsAppProviderRuntime` lifecycle hooks. It can start the feature-gated live
  driver only for an explicitly smoke-opted-in account with a
  `whatsapp_web_session_key` host-vault session binding, and reports manager
  state through sanitized runtime health metadata.
- QR/pair-code startup now receives `SecretReferenceStore` and `HostVault`
  context through `WhatsAppProviderRuntime`, so the native manager can create an
  account-scoped host-vault bootstrap binding before starting the feature-gated
  driver.
- Live QR/pair-code display artifacts now use an in-process, account-scoped,
  one-time transient channel fed by provider-observed `PairingQrCode` /
  `PairingCode` events. The start response may expose QR SVG or pair code with
  expiry, while sanitized runtime DTOs and health payloads still exclude the
  raw artifacts and no artifact is stored in PostgreSQL/events/logs.
- Startup restore reconciliation now attempts eligible `whatsapp_native_md`
  runtime startup from the account-scoped host-vault session binding through
  `WhatsAppProviderRuntime::start_runtime`, without asking the owner to re-pair.
  The worker is still gated by explicit native smoke opt-in and emits sanitized
  status/session events only; session material stays out of PostgreSQL/events/logs.
- `native_md` now has a smoke-gated, account-scoped reconnect policy. It records
  provider-observed degraded/recovered lifecycle state, schedules bounded
  reconnect attempts from the same vault-bound session, and emits only sanitized
  runtime lifecycle evidence through the shared Signal Hub sink.
- `native_md` now has a smoke-gated provider command execution boundary for the
  verified `wa-rs` subset: send text, reply, edit, delete/revoke, react/unreact,
  mark-read with provider message ids, leave group, send media upload and send
  voice-note upload. The backend claims only durable outbox rows for
  `provider_shape = whatsapp_native_md`, executes through
  `WhatsAppProviderRuntime::execute_live_provider_command`, records SDK success
  as sanitized provider-submission metadata, clears the execution lock, and keeps
  `reconciliation_status = awaiting_provider` until provider-observed evidence
  completes the command. Forward is supported only as smoke-gated forwarded-text
  reemit: Communications projection text is submitted through
  `wa-rs::Client::send_message` as an `ExtendedTextMessage` with
  `ContextInfo.is_forwarded = true` and `forwarding_score = 1`, while source
  provider message ids are recorded only as sanitized metadata. Media bytes are
  read from local blob storage by the application worker, passed to the runtime
  only as a redacted in-memory field excluded from serialization, uploaded
  through `wa-rs::Client::upload`, and never written to
  PostgreSQL/events/logs/frontend; `media_key` and provider URL are also
  excluded from result metadata. Live `download_media` now consumes host-vault
  `whatsapp_media_download_ref` payloads in memory, downloads through
  `wa-rs::Client::download_from_params`, writes bytes only to local blob storage
  and projects through runtime-bridge media observation evidence. Status/
  archive/mute/pin/join/unread remain structured unsupported paths until SDK/API
  support and live smoke evidence are verified.
- `native_md` runtime health exposes a `wa_rs_sdk_command_gap` manifest. The
  manifest is based on local `wa-rs 0.2.0` public API inspection, names the
  verified SDK methods used for the smoke-gated subset, including the
  forwarded-text reemit contract for `forward`, and names the missing safe write
  APIs for status publish, dialog-state writes, `mark_unread` and
  join-by-invite. Unsupported native commands may be claimed only to write
  structured terminal dead-letter evidence with
  `native_md_command_kind_unsupported`; execution performs this unsupported
  preflight before smoke-gate and runtime-driver lookup so missing SDK/API
  support is not masked as `native_md_runtime_not_running` or retried as a
  transient provider failure. SDK failure or unsupported status must not
  complete a command without provider-observed r
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/adr/ADR-0102-zoom-provider-runtime-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0102-zoom-provider-runtime-boundary.md`
- Size bytes / Размер в байтах: `4250`
- Included characters / Включено символов: `4250`
- Truncated / Обрезано: `no`

````markdown
# ADR-0102 Zoom Provider Runtime Boundary

Status: Accepted
Date: 2026-06-27

## Context

Макошь integrates communication providers without turning each provider into a
product domain. Zoom contains meetings, recordings, participants, transcripts
and webhook/runtime concerns, but Макошь must treat those as provider
observations that feed memory and context.

Макошь already follows the provider/channel rule for Telegram and WhatsApp:
provider-specific runtime code belongs under integrations, while product
meaning belongs to provider-neutral domains, workflows and engines.

This ADR defines the boundary for the Zoom foundation implementation.
Validation in this checkout is complete:

- Backend targeted suite:
  `CARGO_TARGET_DIR=target/zoom-verify ./scripts/test/run-nextest.sh integration --test zoom_provider_foundation --test zoom_signal_detection --test zoom_calendar_matching --test zoom_participant_identity`
  (27 passed, 0 failed).
- Backend static checks:
  `make backend-fmt-check`, `make backend-clippy`,
  `node scripts/check-architecture.mjs`, `node scripts/check-code-boundaries.mjs`,
  `git diff --check`.
- Frontend targeted checks:
  `cd frontend && pnpm lint`, `cd frontend && pnpm typecheck`,
  `cd frontend && pnpm exec vitest run src/integrations/zoom/api/zoom.test.ts src/integrations/zoom/queries/zoomQueryKeys.test.ts src/platform/bootstrap/realtimeZoomInvalidation.test.ts src/domains/communications/api/callApi.test.ts`.

## Decision

Zoom lives under:

```text
backend/src/integrations/zoom
frontend/src/integrations/zoom
```

Zoom integration may:

- manage provider account metadata;
- expose runtime status and local lifecycle controls;
- register fixture accounts for deterministic validation;
- register live account metadata and secret references in blocked mode;
- exchange OAuth user and Server-to-Server credentials and store credential
  payloads through HostVault-backed secret references;
- explicitly refresh or renew OAuth user and Server-to-Server token bundles
  through HostVault-backed secret references;
- scan authorized accounts and refresh expiring token bundles through the same
  HostVault-backed secret boundary;
- run a configurable local scheduler that invokes the token maintenance scan
  without exposing raw token material;
- expose token rotation policy metadata, refresh due state and
  failure/expiry blockers in runtime status without exposing raw token
  material;
- accept runtime-bridge observations for meetings, recordings and transcripts;
- persist call/transcript evidence through platform call intelligence
  primitives;
- emit `zoom.*` events with canonical envelope metadata;
- sanitize token-like fields before event append/broadcast.

Zoom integration must not:

- become `domains/zoom`;
- create tasks, Personas, notes, organizations, documents or calendar events
  directly;
- call business domains to mutate state;
- store raw secrets in provider account config or event payloads;
- treat AI extraction as source of truth;
- perform hidden recording, hidden transcription or automatic meeting joining
  without explicit owner-visible setup.

## Event contract

Zoom emits:

```text
zoom.authorization.completed
zoom.runtime.status_changed
zoom.token.refreshed
zoom.token.refresh.skipped
zoom.token.refresh.failed
zoom.meeting.observed
zoom.recording.observed
zoom.transcript.observed
zoom.recording.import.removed
zoom.transcript.removed
zoom.retention.cleanup.completed
```

Events must use canonical envelopes and preserve causation/correlation when
supplied.

## Consequences

Zoom meeting evidence can flow into Calls, Calendar preparation, Radar, Review,
Tasks and Knowledge through events/workflows. The provider adapter remains
replaceable: additional provider workers can be added later without changing
domain ownership.

Live provider execution now includes the authorized recording-sync worker,
including privacy-gated recording media download/import and transcript-like file
download/import, explicit owner-visible recording import removal and explicit
retention cleanup for expired recording/transcript evidence, including local
scheduled retention cleanup automation. It remains partial until downstream
workflow boundaries are explicitly implemented.
````

### `docs/adr/ADR-0104-yandex-telemost-provider-runtime-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0104-yandex-telemost-provider-runtime-boundary.md`
- Size bytes / Размер в байтах: `2271`
- Included characters / Включено символов: `2271`
- Truncated / Обрезано: `no`

````markdown
# ADR-0104: Yandex Telemost Provider Runtime Boundary

Status: Proposed
Date: 2026-06-28

## Context

Макошь needs Yandex Telemost support for conference creation, conference links,
visible WebView joining, local audio capture and later transcription. Telemost is
an external provider, not a Макошь domain.

The integration also needs local desktop behavior that the backend provider API
cannot own: visible WebView opening, system/loopback audio capture and speaker
hint extraction from the WebView.

## Decision

Add Yandex Telemost as a provider runtime integration:

```text
backend/src/integrations/yandex_telemost
frontend/src/integrations/yandexTelemost
frontend/src-tauri/src/yandex_telemost_companion.rs
```

Use provider kind:

```text
yandex_telemost_user
```

Use secret purpose:

```text
yandex_telemost_oauth_token
```

The backend stores raw OAuth tokens only in HostVault. The provider account
stores a secret reference and lifecycle/capability metadata.

The desktop companion opens Telemost only in a visible owner-controlled WebView.
Hidden recording is forbidden. Local MP3 recording requires
`consent_attested=true`.

Speaker timeline files derived from WebView DOM are hints only:

```text
truth_status = hint_not_truth
```

## Consequences

Positive:

- Telemost does not become a product domain.
- API credentials stay out of business state and events.
- Local recording is explicit and owner-visible.
- Whisper/diarization workflows get useful hint files without trusting the DOM
  as evidence.

Negative:

- macOS and Windows require explicit loopback audio setup.
- WebView speaker extraction is heuristic and must be improved against real DOM
  observations.
- Review and owner-domain promotion quality still depends on conservative
  provider-neutral workflow mapping rather than Telemost-specific direct domain
  mutation.

## Rejected options

### Create `domains/yandex_telemost`

Rejected because providers are external systems, not bounded contexts.

### Hidden background browser join and capture

Rejected because it breaks the Макошь owner-visible runtime model and creates a
hidden capture path that the provider runtime must not own.

### Treat WebView active-speaker DOM as truth

Rejected. It is only a weak hint for diarization.
````

### `docs/adr/ADR-architecture-communication-contract.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-architecture-communication-contract.md`
- Size bytes / Размер в байтах: `4342`
- Included characters / Включено символов: `4342`
- Truncated / Обрезано: `no`

````markdown
# ADR Architecture Communication Contract

Status: Accepted
Date: 2026-06-20

Supersedes:

- ADR-0073 Backend Module Organization, for direct Graph imports and broad
  layer wording that allowed cross-domain shortcuts.
- ADR-0095 Event-Driven Domain Communication and DLQ, for the temporary
  architecture boundary baseline.

Clarifies:

- ADR-0014 Canonical Event Envelope
- ADR-0035 Local Event API Command Boundary
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0097 Communications Channel Domains To Integrations

## Context

Макошь has enough domain and provider surface that "do not import another
domain" is no longer precise enough. The architecture needs one communication
contract that applies to backend modules, frontend modules, events, projections,
provider runtimes and AI outputs.

The old boundary baseline made this ambiguous by allowing exact legacy
violations to stay green. That baseline is now removed. Architecture violations
must be fixed, not registered as exceptions.

## Decision

Макошь uses exactly these component interaction kinds:

```text
direct_call
command_port
query_port
event
projection
runtime_integration_api
```

All component boundaries are described in
`scripts/architecture-contract.json`. That JSON file is executable policy and is
validated by `make architecture-check`.

Backend rules:

- `app/` owns route composition, HTTP handlers, app state and top-level errors.
  It may call domain command/query ports and integration runtime/setup APIs.
  It must not own business orchestration or durable stores.
- `domains/*` own one bounded context. A domain may import its own modules,
  `platform/*`, and pure/domain-neutral engines. It must not import other
  domains, integrations, app handlers or workflows for business behavior.
- `integrations/*` own external provider protocol, setup and runtime state.
  They may import platform, vault and external SDKs. They must not import
  business domains or mutate business truth directly.
- `workflows/*` coordinate multiple domains through command/query ports and
  events. They must be idempotent and carry causation/correlation metadata.
  They must not own HTTP handlers, domain stores or integration clients.
- `engines/*` are pure or domain-neutral. They may own their own projections and
  indexes. They must not mutate business domains or import integrations.
- `ai/*` produces candidates, summaries, classifications and embeddings. It is
  not a source of truth and must not mutate domains directly.
- `platform/*` is importable by all layers and must not import domains,
  integrations or workflows.
- `vault/*` owns secrets, sessions and runtime credential state only.

Frontend rules:

- `frontend/src/app` composes routes, multiple domain views and multiple domain
  stores.
- `frontend/src/domains/*` must not import other frontend domains.
- `frontend/src/integrations/*` is provider setup/runtime UI only.
- Provider business query/cache roots `['telegram', ...]`,
  `['whatsapp', ...]` and `['mail', ...]` are forbidden. Business data uses
  `['communications', ...]`. Provider runtime state may use
  `['integrations', provider, 'runtime', ...]`.

## Consequences

Positive:

- The repository has one canonical interaction vocabulary.
- Baseline files and per-file compatibility exceptions stop hiding coupling.
- Communication channels can be provider integrations without owning product
  domains.
- Frontend cache ownership follows product boundaries.

Negative:

- Existing synchronous projection paths must move to workflows/events or
  command/query ports.
- Some historical provider/domain DTOs need to move to platform-owned contract
  types.
- Cross-domain behaviors become more explicit and sometimes eventually
  consistent.

## Validation

`make architecture-check` must run:

```text
node scripts/check-architecture-contract.test.mjs
node scripts/check-architecture.mjs --self-test
node scripts/check-architecture.mjs
```

The architecture guard must fail if:

- `scripts/architecture-boundary-baseline.json` exists;
- the interaction kind vocabulary changes without updating the contract;
- backend domains import other backend domains;
- backend integrations import business domains;
- frontend domains import other frontend domains;
- provider business cache keys use provider-root roots.
````

### `docs/adr/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/README.md`
- Size bytes / Размер в байтах: `3768`
- Included characters / Включено символов: `3768`
- Truncated / Обрезано: `no`

```markdown
# Architecture Decision Records

ADR status vocabulary:

- Proposed: accepted as initial architecture direction, pending implementation validation.
- Accepted: validated by implementation and retained.
- Temporary: intentionally time-bounded decision that must be revisited before expanding scope.
- Superseded: replaced by a later ADR.

## Index

- [ADR-0001 Event Sourcing as System Spine](ADR-0001-event-sourcing-as-system-spine.md)
- [ADR-0002 Rust Backend](ADR-0002-rust-backend.md)
- [ADR-0003 SvelteKit Frontend](ADR-0003-sveltekit-frontend.md)
- [ADR-0004 Tauri Desktop Shell](ADR-0004-tauri-desktop-shell.md)
- [ADR-0005 PostgreSQL Primary Store](ADR-0005-postgresql-primary-store.md)
- [ADR-0006 Tantivy Full Text Search](ADR-0006-tantivy-full-text-search.md)
- [ADR-0007 Replaceable Vector Search](ADR-0007-replaceable-vector-search.md)
- [ADR-0008 Knowledge Graph First](ADR-0008-knowledge-graph-first.md)
- [ADR-0009 Local AI Through Ollama](ADR-0009-local-ai-through-ollama.md)
- [ADR-0010 Specialized Agent System](ADR-0010-specialized-agent-system.md)
- [ADR-0011 Plugin Architecture](ADR-0011-plugin-architecture.md)
- [ADR-0012 OpenTelemetry Observability](ADR-0012-opentelemetry-observability.md)
- [ADR-0013 Local First Data Ownership](ADR-0013-local-first-data-ownership.md)
- [ADR-0014 Canonical Event Envelope](ADR-0014-canonical-event-envelope.md)
- [ADR-0015 Command Query Separation](ADR-0015-command-query-separation.md)
- [ADR-0016 Secrets and Encryption Boundary](ADR-0016-secrets-and-encryption-boundary.md)
- [ADR-0017 Document Processing Pipeline](ADR-0017-document-processing-pipeline.md)
- [ADR-0018 Provider Adapter Boundary](ADR-0018-provider-adapter-boundary.md)
- [ADR-0019 Contact Identity Resolution](ADR-0019-contact-identity-resolution.md)
- [ADR-0020 Task Candidate Lifecycle](ADR-0020-task-candidate-lifecycle.md)
- [ADR-0021 Calendar as Event Source](ADR-0021-calendar-as-event-source.md)
- [ADR-0022 No Fine Tuning on Private Data](ADR-0022-no-fine-tuning-on-private-data.md)
- [ADR-0023 Rebuildable Projections](ADR-0023-rebuildable-projections.md)
- [ADR-0024 Idempotent Imports](ADR-0024-idempotent-imports.md)
- [ADR-0025 Keyboard First Command Palette](ADR-0025-keyboard-first-command-palette.md)
- [ADR-0026 Desktop First Responsive UI](ADR-0026-desktop-first-responsive-ui.md)
- [ADR-0027 Capability Based Permission Model](ADR-0027-capability-based-permission-model.md)
- [ADR-0028 Backup and Restore as Core Feature](ADR-0028-backup-and-restore-as-core-feature.md)
- [ADR-0029 Explicit Schema Evolution](ADR-0029-explicit-schema-evolution.md)
- [ADR-0030 Documentation First Monorepo](ADR-0030-documentation-first-monorepo.md)
- [ADR-0031 Temporary Desktop Only UI Scope](ADR-0031-temporary-desktop-only-ui-scope.md)
- [ADR-0032 Docker Compose Development Environment](ADR-0032-docker-compose-development-environment.md)
- [ADR-0033 Backend Managed Local Schema Migrations](ADR-0033-backend-managed-local-schema-migrations.md)
- [ADR-0034 Event Replay and Projection Cursors](ADR-0034-event-replay-and-projection-cursors.md)
- [ADR-0035 Local Event API Command Boundary](ADR-0035-local-event-api-command-boundary.md)
- [ADR-0036 Projection Runner Checkpoint Semantics](ADR-0036-projection-runner-checkpoint-semantics.md)
- [ADR-0037 Local Write Capability Token](ADR-0037-local-write-capability-token.md)
- [ADR-0038 Local Event API Capability Token](ADR-0038-local-event-api-capability-token.md)
- [ADR-0039 Local Event API Access Audit Log](ADR-0039-local-event-api-access-audit-log.md)

## Recent Provider ADRs

- [ADR-0102 Zoom Provider Runtime Boundary](ADR-0102-zoom-provider-runtime-boundary.md)
- [ADR-0104 Yandex Telemost Provider Runtime Boundary](ADR-0104-yandex-telemost-provider-runtime-boundary.md)
```
