# V5 Closure Checklist

## Release Goal

Version 5.0 is complete when Макошь can use WhatsApp Web as a local-first, user-visible companion source while preserving source provenance, privacy, auditability, graph-backed recall and long-horizon personal knowledge workflows.

## In Scope

- ADR-0051 governs WhatsApp Web companion boundaries.
- `whatsapp_web` provider account metadata and account-scoped secret references.
- Fixture/manual WhatsApp Web session state for CI and local development.
- Append-only WhatsApp Web raw records and canonical message projections.
- Desktop-only WhatsApp Web account, session and sync status surfaces.
- Explicit capability reporting for fixture/manual runtime, blocked live runtime and unsupported automation.
- Long-horizon analytics and decision/relationship/project memory improvements from V5 roadmap.

## Out Of Scope For V5 Foundation

- Hidden WhatsApp Web scraping.
- Reverse-engineered protocol runtime as a production dependency.
- Bulk messaging, auto-messaging or auto-dialing.
- Live outbound WhatsApp sends.
- Mobile UI.
- Training or fine-tuning on WhatsApp data.
- Treating WhatsApp Business Platform Cloud API as personal WhatsApp Web.

## Acceptance Gate Status

- [x] ADR-0051 documents WhatsApp Web companion constraints.
- [x] V5 roadmap closure checklist exists.
- [x] Provider account model accepts `whatsapp_web` without breaking email or Telegram providers.
- [x] WhatsApp Web session secret purpose is account-scoped and compatible only with non-plaintext session protection secrets.
- [x] Backend migration extends provider/message constraints and creates WhatsApp Web session metadata storage.
- [x] Backend exposes protected `/api/v1/integrations/whatsapp/*` fixture/manual foundation endpoints.
- [x] WhatsApp Web fixture ingestion projects raw messages into canonical communication messages.
- [x] Protected `/api/v1/integrations/whatsapp/capabilities` exposes fixture-ready, live-blocked and unsupported WhatsApp capabilities.
- [x] Desktop WhatsApp account/session/status surfaces call protected backend APIs.
- [x] `make backend-whatsapp-smoke-dev` covers WhatsApp Web fixture/manual foundation.
- [x] `make validate`, frontend checks and desktop browser smoke pass after WhatsApp UI integration.

## Remaining V5 Risk

- Live WhatsApp Web runtime and live outbound sends remain blocked until an explicit runtime ADR, capability model and user-visible consent flow are approved.
