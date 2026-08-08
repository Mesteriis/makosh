# ADR-0182: WhatsApp Hidden WebView Runtime Only

Status: Accepted
Date: 2026-07-13

Supersedes:

- ADR-0051 V5 WhatsApp Web Companion Boundary
- ADR-0101 WhatsApp Provider Runtime Selection

Clarifies: ADR-0181 Backend Workspace Modularity and Provider Runtime Topology

## Context

The prior WhatsApp design accumulated three independent provider shapes:
an owner-visible WhatsApp Web companion, a Native Multi-Device experiment and
the WhatsApp Business Cloud adapter. They have different credential models,
runtime lifecycles, command paths, health contracts and test fixtures. That
made a personal communication source look like a provider platform and kept
large, unrelated branches in the backend composition root.

Макошь currently needs one local-first WhatsApp acquisition boundary. It does
not need a Business API, a native protocol implementation or automatic runtime
topology selection.

## Decision

WhatsApp has exactly one provider runtime: an account-scoped hidden Tauri
WebView for `https://web.whatsapp.com/`.

- The runtime remains `provider_kind = whatsapp_web` and
  `provider_shape = whatsapp_web_companion`; the latter is now an internal
  provider identifier, not a visible-window promise.
- The WebView is created with `visible(false)` and is never shown or focused by
  a Макошь command. Its lifecycle is controlled through the existing explicit
  start/stop/revoke/relink/remove runtime operations.
- The injected contract remains main-frame-only, origin-guarded and
  metadata-only. It must not read cookies, Web Storage, IndexedDB, browser
  profile secrets, session material, message bodies or media bytes. It must not
  call arbitrary network APIs, domain APIs or complete provider commands.
- The only outbound path from the WebView is the account-scoped, allowlisted
  Tauri relay to protected local runtime-bridge routes. Canonical evidence and
  projection ownership remain in Communications and Signal Hub.
- WhatsApp Business Cloud, Native Multi-Device, `wa-rs`, their command
  executors, vault secret purposes, runtime restore logic, webhook/edge-proxy
  routes, frontend setup choices and fixtures are removed. No automatic
  fallback or alternate WhatsApp topology exists.
- Existing durable rows whose provider kind or shape is no longer the single
  WebView path are not migrated or interpreted by runtime code. No destructive
  down-migration is introduced in this refactoring slice.

## Consequences

Positive:

- one runtime has one session model, lifecycle and failure surface;
- the desktop process has no WhatsApp-native SDK or business credential path;
- build and test scope shrink with the removed provider branches;
- all WhatsApp observations still preserve provenance and enter the canonical
  evidence flow through protected runtime-bridge routes.

Negative:

- Business Cloud and Native MD setup, webhooks, commands and live-session
  experiments are intentionally unavailable;
- a hidden WebView has no owner-facing browser control surface;
- a future alternate WhatsApp runtime requires a new ADR rather than adding a
  shape to this boundary.

## Validation

The runtime must reject every provider shape except
`whatsapp_web_companion`; no production source may retain Business Cloud,
Native MD, `wa-rs` or Business Cloud edge-proxy references. Tests use only
fixtures and the metadata-only relay contract; no live WhatsApp account,
credential, session or private content is used.
