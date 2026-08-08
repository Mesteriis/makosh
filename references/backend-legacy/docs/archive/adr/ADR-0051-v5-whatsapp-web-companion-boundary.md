# ADR-0051 V5 WhatsApp Web Companion Boundary

Status: Superseded by ADR-0182

## Context

Version 5 adds WhatsApp as a long-term communication memory source. The reliable integration path is materially different from email and Telegram:

- WhatsApp Web and desktop are linked-device companion experiences, not documented personal-account APIs.
- The official WhatsApp Business Platform Cloud API is for business messaging and requires Meta business assets such as a business portfolio, WhatsApp Business Account and business phone number.
- WhatsApp's consumer terms restrict unauthorized automated access, impermissible collection and unauthorized software or APIs that function like the service.
- Макошь is local-first and personal. It must preserve user-controlled local memory without turning WhatsApp Web into a hidden scraping or automation channel.

References checked during this decision:

- WhatsApp Terms of Service, "Acceptable Use Of Our Services".
- Meta's WhatsApp Cloud API Postman collection, which identifies Cloud API as the official WhatsApp Business Platform API and lists business onboarding requirements.

## Decision

Implement personal WhatsApp support through an explicit `whatsapp_web` companion boundary.

Rules:

- `whatsapp_web` is a communication provider kind for a user-linked local companion session.
- The first implementation path is fixture/manual companion state only. Live WhatsApp Web sessions must report `blocked` until a visible desktop runtime, session lifecycle, permission prompts, local storage policy and smoke validation exist.
- The companion runtime must be user-visible through the desktop shell or an explicitly controlled browser/WebView. Hidden headless scraping is not an accepted provider adapter.
- PostgreSQL stores account metadata, session status, raw record metadata, checkpoints, canonical projections and audit records. It must not store WhatsApp Web session secrets, pairing material or local browser profile secrets.
- Local WhatsApp Web session state lives under ignored local data paths such as `docker/data/whatsapp/<account_id>/` for development, encrypted where the runtime supports it.
- WhatsApp Web credentials and local session protection material are resolved by `account_id + secret_purpose`, using `whatsapp_web_session_key` for session encryption/protection. Provider kind alone must never select credentials.
- Raw WhatsApp Web records are append-only with provider provenance. The initial raw message record kind is `whatsapp_web_message`.
- Checkpoints are account-scoped. Initial stream IDs should be delimiter-safe, for example `whatsapp_web:global` or `whatsapp_web:<provider_chat_id>`.
- Canonical message projections may use `channel_kind = 'whatsapp_web'`.
- Outbound live sends are not part of the first V5 foundation. Future sends require the same policy, template, actor, audit and capability discipline established for Telegram automation, plus WhatsApp-specific validation.
- WhatsApp Business Platform Cloud API is a separate future provider shape, not a substitute for personal WhatsApp Web. If added, it should use a distinct provider kind such as `whatsapp_business_cloud` and its own ADR/update.

## Consequences

Positive:

- Макошь can model WhatsApp Web without pretending that an unofficial stable personal API exists.
- The provider boundary keeps WhatsApp Web runtime fragility away from canonical messages, graph projections and AI workflows.
- Fixture/manual state allows backend and UI work to start without live WhatsApp credentials or hidden browser automation.
- Session state and secrets stay outside PostgreSQL, preserving backup and debugging safety.

Negative:

- V5 cannot claim live WhatsApp Web sync until the visible runtime and safety checks exist.
- Companion sessions may remain more fragile than email, Telegram TDLib or future official business APIs.
- Manual linking, revocation and degraded-session UX become first-class product work.

Risk handling:

- Capability reporting must distinguish fixture/manual readiness from blocked live runtime.
- Provider adapters must never log message bodies, session secrets, pairing codes or browser profile paths containing private state.
- Tests must cover idempotent raw records, account-scoped credential lookup and refusal of live sends before any live runtime is enabled.

## Non-Goals

- Hidden WhatsApp Web scraping.
- Reverse engineering WhatsApp protocols as a production dependency.
- Bulk messaging, auto-messaging or auto-dialing.
- Live outbound WhatsApp sends in the V5 foundation slice.
- Replacing personal WhatsApp Web with WhatsApp Business Platform Cloud API.
- Training or fine-tuning models on WhatsApp data.
