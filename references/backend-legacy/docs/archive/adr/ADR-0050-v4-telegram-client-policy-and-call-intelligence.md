# ADR-0050 V4 Telegram Client, Policy Automation and Call Intelligence

Status: Proposed

## Context

Version 4 expands Макошь from read-only memory and source-backed AI into a Telegram-capable local client with controlled outbound automation and call-derived task context. Telegram user accounts and bot accounts have different API surfaces, credentials, and limits. Telegram user accounts require a full client runtime, while bot accounts are constrained by Bot API visibility.

AI and automation are now allowed to send Telegram messages without per-message confirmation, but only when an explicit user-configured policy and approved template authorize the send. Call transcription is required for enabled Telegram accounts/chats and must remain local.

## Decision

Implement V4 around explicit boundaries:

- Telegram supports multiple `telegram_user` and `telegram_bot` provider accounts.
- Telegram user accounts use a TDLib-first runtime boundary. TDLib local state must be account-scoped and stored under ignored local data paths, encrypted where supported.
- Telegram bot accounts use a Bot API-compatible runtime boundary.
- PostgreSQL stores account metadata, raw source records, checkpoints, canonical projections, policy state, call metadata, transcript metadata and audit records. It does not store Telegram API hashes, bot tokens, session encryption keys or other secret values.
- Telegram credentials are resolved by `account_id + secret_purpose`; provider kind alone must never select credentials.
- V4 accepts a fixture Telegram runtime for tests and local smoke validation. Live Telegram validation is opt-in.
- AI and automation may send Telegram messages only through enabled policies configured in the UI. Policies bind templates, accounts, chats, triggers, limits, quiet hours and expiry.
- AI may fill only declared template variables. It cannot choose destinations, templates, policy authority or send scope from retrieved content.
- Every automated send writes canonical event/audit metadata with policy ID, template ID, account ID, chat ID, preview hash and actor context.
- V4 call scope is 1:1 audio call MVP. Video calls, group calls and screen sharing are V4.x or later.
- Call transcription is local, policy/account/chat scoped, visible in UI, and stored with source provenance.
- Telegram data may be used for local workflows, retrieval and task extraction, but not for fine-tuning or training models.
- V4 exposes a protected capability contract that reports available fixture capabilities, blocked live-runtime capabilities and unsupported V4.x features to both UI and tests.

## Consequences

Positive:

- Telegram becomes a first-class local communication channel without making provider data the source of truth.
- Automated sends are possible while preserving user-configured authority and auditability.
- Call transcripts can feed existing task candidate and project memory workflows.
- Fixture runtime keeps CI independent from live Telegram credentials and audio devices.

Negative:

- TDLib/native media integration is a larger operational dependency than prior HTTP-only providers.
- Call capture and transcription introduce privacy, storage and platform permission complexity.
- The policy evaluator becomes security-critical and must be covered by regression tests before live sends.

Risk handling:

- Live TDLib sessions, live Bot API sends, desktop audio capture and `whisper-rs` transcription are not silent runtime gaps. They must report `blocked` through the V4 capability contract until their adapters, permissions, secret resolution and smoke validation exist.
- Fixture Telegram runtime, automated-send dry-run, call metadata storage and fixture speech-to-text are V4 closure capabilities and must report `available`.
- Video calls, group calls, screen sharing, hidden recording, Telegram-data fine-tuning and third-party plugin execution are unsupported V4 features and must remain outside V4 closure gates.

## Non-Goals

- Video calls.
- Group calls.
- Screen sharing.
- Training or fine-tuning models on Telegram data.
- Hidden recording.
- Third-party plugin code execution.
