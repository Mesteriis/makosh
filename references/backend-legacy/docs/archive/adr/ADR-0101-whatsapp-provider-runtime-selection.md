# ADR-0101 WhatsApp Provider Runtime Selection

Status: Superseded by ADR-0182
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
  complete a command without provider-observed reconciliation.
- Inbound native media DTOs now expose only a sanitized download-reference
  contract for direct image/video/audio/document/sticker payloads: media type,
  content type, lengths, file hashes, hashed `direct_path` / `static_url`, hashed
  `media_key`, a stable provider media ref fingerprint and the required
  `whatsapp_media_download_ref` host-vault secret purpose. The live native event
  handler now materializes raw provider refs into host vault before redaction,
  under deterministic account-scoped `secret_ref` values with metadata-only
  `secret_references` rows. The raw `media_key`, `direct_path`, `static_url`,
  URLs, captions, filenames and bytes remain excluded from DTOs, raw evidence,
  logs and frontend payloads. Live media upload submissions also persist a
  sanitized provider-observed completion target: accepted
  `signal.accepted.whatsapp.media` evidence can reconcile `send_media` /
  `send_voice_note` when the observed `provider_message_id` matches the stored
  `wa-rs` `provider_request_id`, while fixture blob-id matching remains as a
  fallback. This remains publicly blocked until the upload/download paths pass
  manual live smoke.
- Runtime health now exposes the same command-capability matrix explicitly:
  verified SDK submission subset, unsupported live command set, public
  availability gate, provider-observed reconciliation rule and sanitized payload
  policy. The UI/worker surface must not infer broader live capability from the
  presence of the SDK feature.
- Runtime availability still remains false until the manual live smoke checklist
  passes and the remaining live media/status/group/dialog coverage is verified.
- Local validation passed with `cargo +1.89.0 check --manifest-path
  backend/Cargo.toml --features whatsapp-native-md-runtime --lib`.

### Rejected foundation

Do not use `whatsappweb-rs` as the main provider foundation because it is old, heavily WIP and has no release posture suitable for Макошь.

### Future official business provider

For official business accounts, evaluate:

- `whatsapp-business-rs`;
- `wacloudapi`;
- direct `reqwest` implementation if SDKs do not fit.

This provider must use:

```text
provider_kind = whatsapp_business_cloud
```

and must follow WABA/template/webhook/business policy semantics.
The setup credential boundary is account-scoped host vault storage:
`whatsapp_business_cloud_access_token` stores the API token,
`whatsapp_business_cloud_app_secret` stores the webhook signature secret, and
`whatsapp_business_cloud_webhook_verify_token` stores the challenge verify
token, while PostgreSQL keeps only non-secret metadata and the provider-account
secret binding.
The first implemented slice uses a direct `reqwest` adapter behind
`WhatsAppProviderRuntime` for a smoke-gated submission subset:
`send_text`, `send_template`, `send_media` and `send_voice_note`.
`runtime = business_cloud_smoke`, `business_cloud_live_smoke_enabled = true`
and a host-vault token binding are required before the worker claims a durable
outbox command. Text and template commands submit through the Graph messages
endpoint. Media and voice-note commands read local blob bytes in worker memory,
validate size/SHA-256, upload through the Graph media endpoint and send the
returned media id through the Graph messages endpoint. SDK/API success records
sanitized provider-submission metadata and keeps completion waiting for
provider-observed webhook/event
reconciliation. Graph submission outcomes also persist a sanitized
provider-observed completion target: webhook `statuses[]` receipt evidence must
match the stored Graph message id/provider request id before `send_text`,
`send_template`, `send_media` or `send_voice_note` can complete. The local
runtime-bridge webhook ingest path now normalizes
Meta-like Business Cloud text messages and delivery statuses into the existing
message/receipt evidence spine, while unsupported webhook entries are preserved
as sanitized degraded runtime evidence. The local bridge verifies challenge
tokens and `X-Hub-Signature-256` raw-body HMAC-SHA256 signatures using
host-vault secrets before ingestion. Because ADR-0056 keeps Макошь `/api/v1`
behind `X-Макошь-Secret`, public Meta webhook exposure still requires an
explicit proxy/edge bridge; Макошь itself is not opened as an unauthenticated
public endpoint. Макошь exposes a protected proxy manifest at
`/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest`
that states the edge forwarding contract, raw-body/signature policy and
per-account binding readiness without reading or returning secret values.
The repository also provides `makosh-whatsapp-business-cloud-edge-proxy` as the
standalone local bridge artifact: it exposes public
`/webhooks/whatsapp/business-cloud`, forwards GET challenge queries and exact
POST raw bodies/signatures to protected Макошь with `X-Макошь-Secret`, and does
not parse webhook JSON or read host-vault secrets. Optional edge `account_id`
scope is applied only to GET challenge forwarding; manifest readiness checks
do not carry account query state. The edge bridge is packaged as an opt-in
Docker Compose profile named `whatsapp-business-cloud-edge` with a dedicated
runtime image target and Makefile config/up/stop/logs targets. The default bind
is loopback and `docker/.env.example` contains only non-secret placeholders, so
public ingress and Meta webhook registration remain explicit operator actions
and live smoke gates. GET challenge forwarding may include optional account
scope, while POST webhook delivery stays account-query-free so signature
verification is performed over the provider-observed raw POST body plus
headers.
Business Cloud `send_text` failures now map HTTP 429 and `Retry-After` into the
existing structured outbox retry metadata without storing raw provider payloads.
WABA asset discovery, broader rate-limit policy, external public edge
deployment and manual live smoke remain future Business Cloud work, and Cloud
remains a separate provider shape rather than a personal WhatsApp substitute.

## Boundary

Third-party provider libraries live only behind:

```text
backend/src/integrations/whatsapp/runtime/*
```

Макошь application, workflow, domain, engine and UI-facing backend code must
depend on the replaceable Rust boundary:

```rust
trait WhatsAppProviderRuntime
```

The current `whatsapp_web` fixture/Web companion foundation is one implementation
of that trait. Future native multi-device, `wa-rs`, official Business Cloud API
or custom runtime implementations must replace or extend the runtime
implementation without changing Communications, Radar, Timeline, Search,
Personas or AI call sites.

They may produce:

- runtime lifecycle events;
- provider observations;
- provider command execution results;
- media transfer results;
- sanitized health diagnostics.

They may not produce or mutate:

- Tasks;
- Personas;
- Organizations;
- Documents;
- Notes;
- Knowledge;
- Decisions;
- Obligations;
- Memory;
- Search truth;
- AI truth.

Inbound flow:

```text
External provider
  -> WhatsApp runtime
  -> WhatsApp adapter
  -> observation/signal event
  -> Communications projection
  -> Radar/Review/Timeline/Search/Engines
```

Outbound flow:

```text
UI/App
  -> Communications command
  -> communication.outbox.queued
  -> communication.provider_command.requested
  -> WhatsApp integration command consumer
  -> provider execution
  -> provider-observed evidence
  -> communication.provider_command.completed/failed
```

## Capability defaults

All live capabilities default to blocked:

```text
whatsapp.native_md.runtime = blocked
whatsapp.native_md.send = blocked
whatsapp.native_md.media_download = blocked
whatsapp.native_md.media_upload = blocked
whatsapp.native_md.reaction = blocked
whatsapp.native_md.status_publish = blocked
```

They can become available only after:

- runtime spike passes;
- owner opt-in UX exists;
- capability policy exists;
- outbox/reconciliation exists for writes;
- redaction tests pass;
- manual smoke checklist exists.

## Consequences

Positive:

- Макошь avoids writing the WhatsApp protocol from scratch.
- Provider instability is isolated.
- The existing Communications/Radar/Review architecture remains intact.
- Business Cloud support can be added later without corrupting personal provider assumptions.

Negative:

- Unofficial provider risk remains.
- Protocol drift may break live runtime.
- Account suspension risk must be surfaced to the owner.
- Runtime testing requires manual live smoke tests outside CI.
- The adapter boundary becomes critical infrastructure.

## Risk handling

- Explicit owner-facing warnings for unofficial runtime.
- No bulk messaging, auto-messaging or hidden automation.
- No hidden scraping.
- Feature flag and capability gate.
- Provider writes require confirmation/audit/reconciliation.
- Session secrets stay in host/local runtime storage only.
- Provider startup restores authorization from vault-bound session material, not by asking the owner to re-pair every time.
- Live failures degrade capabilities instead of corrupting canonical state.
- AI output remains candidates with Source, Confidence and Evidence.

## Acceptance criteria

This ADR can move from Proposed to Accepted only when:

1. `whatsapp-rust` compiles on Макошь Rust toolchain or fallback is selected.
2. QR/pair-code session lifecycle can be surfaced without leaking secrets.
3. Inbound message events can be mapped to source-backed observations.
4. Media metadata can be mapped without storing bytes in PostgreSQL.
5. Provider write execution can be routed through the outbox contract.
6. Multiple account isolation is demonstrated.
7. Redaction tests exist.
8. Manual live smoke-test checklist exists.
9. Architecture guard prevents provider library imports outside integration runtime.
10. Documentation clearly distinguishes personal unofficial runtime from official Business Cloud API.

## External references

- `whatsapp-rust`: <https://github.com/oxidezap/whatsapp-rust>
- `wa-rs`: <https://github.com/homun-app/wa-rs>
- `whatsappweb-rs`: <https://github.com/wiomoc/whatsappweb-rs>
- `whatsapp-business-rs`: <https://docs.rs/whatsapp-business-rs>
- `wacloudapi`: <https://docs.rs/wacloudapi>
- WhatsApp Business Developer Hub: <https://whatsappbusiness.com/developers/developer-hub/>
- WhatsApp Terms of Service: <https://www.whatsapp.com/legal/terms-of-service>
