# Zoom Implementation Plan

Status date: 2026-06-27.

This plan keeps Zoom as an integration provider and moves business meaning
through workflows and domain-owned command/query ports.

## Phase 1 - Documentation and boundary closure

Status: `complete`.

Deliverables:

- Zoom README, architecture, API, modules, status, gap analysis and blockers;
- ADR-0102 provider runtime boundary;
- fixture bridge documentation;
- live smoke checklist;
- no `domains/zoom` ownership.

Completion condition: documentation merged with status values that match the
current repository.

## Phase 2 - Foundation implementation

Status: `complete`.

Deliverables:

- backend `integrations/zoom` module;
- migration for provider kinds and secret purposes;
- account setup/list/status lifecycle routes;
- runtime status/start/stop/remove routes;
- meeting, recording and transcript runtime bridge routes;
- event constants and sanitized canonical event append path;
- Signal Hub fixture source registration;
- frontend integration API/types/query key modules.

## Phase 3 - Test coverage hardening

Status: `complete`.

Deliverables:

- backend fixture integration tests for account setup/list/status lifecycle;
- meeting bridge test with provider call evidence assertion;
- recording bridge event append/broadcast assertion;
- transcript bridge call transcript assertion;
- sanitization regression test for nested token-like fields;
- idempotency test for stable event ids and stable call ids;
- frontend query key and invalidation tests.

## Phase 4 - Provider setup UI

Status: `implemented_foundation_ui`.

Deliverables:

- integration settings card for Zoom; implemented;
- fixture account setup form for local/dev; implemented;
- live account metadata form with blocked state explanation; implemented;
- runtime status card; implemented;
- runtime blocker list; implemented;
- runtime bridge lab for local meeting, recording, transcript and transcript
  file ingestion through existing integration mutations; implemented;
- read-only observed call evidence and transcript inspection panel for the
  selected Zoom account through provider-neutral call APIs; implemented;
- OAuth start/complete, Server-to-Server authorize, token refresh and token
  maintenance controls; implemented;
- no product meeting inbox under `frontend/src/integrations/zoom`.

## Phase 5 - Live authorization boundary

Status: `authorization_refresh_implemented`.

Deliverables:

- OAuth user setup flow; implemented for start/complete token exchange;
- Server-to-Server account credential exchange; implemented;
- secret-reference lifecycle; implemented for initial token/client-secret
  HostVault storage and provider-account bindings;
- explicit token refresh/renew route; implemented for OAuth refresh-token and
  Server-to-Server account-credentials renewal;
- token maintenance runner; implemented for scanning authorized accounts and
  refreshing expiring HostVault token bundles;
- scheduled token maintenance daemon; implemented through backend bootstrap,
  Signal Hub runtime gating, HostVault unlock gating and
  `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`;
- token rotation policy; implemented through explicit refresh thresholds,
  proactive maintenance threshold metadata, expiry handling and
  `zoom_token_rotation_required` failure/expiry blocker exposure in runtime
  status;
- audit trail for credential lifecycle; implemented through account-scoped
  authorization-complete plus token-refresh success/skip/failure events on the
  existing Zoom audit surface;
- explicit capability transitions from `blocked` to `available/degraded`;
  implemented for authorization and degraded live runtime metadata.

## Phase 6 - Webhook receiver and verification

Status: `edge_proxy_implemented`.

Deliverables:

- webhook endpoint under integration runtime boundary; implemented for
  protected account-scoped runtime bridge;
- public/edge receiver; implemented as `makosh-zoom-edge-proxy`, which
  forwards raw public Zoom webhook bodies and `x-zm-*` headers to the
  protected runtime bridge;
- signature verification before bridge ingestion; implemented for normal
  webhook events;
- endpoint URL validation response; implemented;
- replay/idempotency guard; timestamp replay window and stable bridge ids are
  implemented, DLQ-backed event processing remains planned;
- normalized bridge calls for meetings and recordings; implemented;
- transcript webhook import; implemented for already obtained VTT/SRT/plain
  text via `/runtime-bridge/transcript-files` and for transcript-like textual
  recording files exposed by signed `recording.completed` payloads with
  `download_url` plus `download_token`;
- provider subscription management; implemented for authorized accounts through
  status/reconcile/remove routes plus live provider API calls.

## Phase 7 - Recording and transcript workers

Status: `transcript_download_webhook_implemented`.

Deliverables:

- manual provider-sync route for authorized cloud recording history; implemented
  for recording list ingestion plus best-effort transcript-like text file
  import through the existing HostVault-backed bearer-token boundary;
- cloud recording reference resolver;
- secret-safe download worker;
- local blob/media persistence;
- document/media scan metadata;
- transcript import pipeline; implemented for already obtained VTT, SRT and
  plain text transcript file contents plus signed webhook-driven download of
  transcript-like textual recording files;
- optional local STT fallback only through explicit policy.

## Phase 8 - Domain workflows

Status: `partially_implemented`.

Deliverables:

- calendar event matching workflow; implemented through
  `zoom.meeting.observed` relation projection;
- participant identity candidate workflow; implemented through conservative
  `attach_email_address` candidate generation into Review;
- meeting follow-up Radar signal workflow;
- transcript note/document candidate workflow;
- task/obligation/decision candidate extraction with source, confidence and
  evidence;
- Review promotion before target domain mutation.

## Phase 9 - Provider-neutral product surfaces

Status: `partially_implemented`.

Deliverables:

- Calls/Calendar meeting evidence view; implemented for provider-neutral
  Communications `calls`/`meetings` evidence reads, broader Calendar-side
  product workflows remain open;
- transcript provenance view; implemented on the Zoom evidence surface and
  provider-neutral transcript reads;
- recording reference/document view; implemented on the Zoom evidence surface
  through recording-reference inspection and import audit/removal controls;
- Radar review cards for meeting follow-ups;
- Knowledge graph evidence links;
- search projection updates.

## Non-goals

- Creating `domains/zoom`.
- Building a Zoom client clone.
- Hidden recording or hidden transcription.
- Automatic domain mutation from AI summaries.
- Provider-specific business cache keys for canonical communication/call data.
