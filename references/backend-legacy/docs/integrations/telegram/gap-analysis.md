# Telegram Gap Analysis

Status date: 2026-07-12.

Base Telegram channel capability status: `COMPLETED`.

This document tracks active Telegram channel capability gaps only. Deferred
initiatives are not base Telegram channel gaps; ADR-0094 and ADR-0097 move them
to separate future work.

## Closure Summary

| Area | Status | Evidence |
|---|---|---|
| Provider reconciliation | CLOSED | Edit, delete, pin, archive, mute, read, unread, reactions, topics, folder add, folder remove and folder reassign use durable provider-write commands and reconcile from provider-observed state or returned provider snapshots. Outgoing `sendMessage` responses preserve TDLib `pending` and `failed` sending states; late `updateMessageSendFailed` is persisted as a durable delivery-state observation. `updateMessageSendSucceeded` transactionally rebinds the temporary provider locator to TDLib's final locator while retaining the canonical Макошь message id; the durable raw-to-accepted-to-projection path is regression-tested including replay. |
| Message lifecycle | CLOSED | Edit versions, tombstones, provider edit/delete evidence and diff metadata are persisted and surfaced through projection APIs. |
| Reply/forward parity | CLOSED | Reply refs are idempotent, reply graph traversal is bounded with cycle guard, forward attribution is idempotent, and forward chains traverse projected local evidence without raw TDLib UI dependency. |
| Topic parity | CLOSED | Topic unread state, realtime topic patching and topic command reconciliation are implemented through `telegram_topics`, runtime topic events and command reconciliation. |
| Dialog parity | CLOSED | Pinned, archived, mute, unread and folder state use provider evidence when TDLib state is available; local projection remains the read model, not a success substitute. |
| Search parity | CLOSED | Message, media, topic and member search use projection-backed Communications routes; provider search is runtime/control sync-assist only and does not return UI-visible business items. |
| Media parity | CLOSED | TDLib photo, video, document, audio, voice, video-note, sticker and animation metadata use the command/query model and shared Communication attachment boundary; repeat history sync enriches legacy projections without rewriting raw evidence. |
| Frontend state | CLOSED | Telegram production components use TanStack Query composables and shared realtime bootstrap; no component-level `fetch(` remains in Telegram production UI. |
| Architecture guardrails | CLOSED | Telegram remains a Communication Channel; Memory, Knowledge, Persona, Organization, Project, Obligation and Decision lifecycle stay outside Telegram. |

## Deferred, Not Gaps

| Capability | Capability state |
|---|---|
| Bot Runtime | `planned` |
| Voice Recording | `planned` |
| Voice Send | `planned` |
| Video Recording | `planned` |
| Live Calls | `planned` |
| Session Export | `planned` |
| Session Import | `planned` |
| MTProxy | `planned` |
| SOCKS5 | `planned` |
| AI Summary | `planned` |
| Translation | `planned` |
| Bilingual Reply | `planned` |
| AI Review Flows | `planned` |

## Closure Gates

| Gate | Status |
|---|---|
| Provider writes use outbox | CLOSED |
| Destructive actions use audit | CLOSED |
| Realtime events use shared event bus/bootstrap | CLOSED |
| Polling is only a bounded fallback while realtime recovers | CLOSED |
| Telegram implementation, test, docs and frontend files stay under 700 lines | CLOSED |
| Documentation matches channel capability scope | CLOSED |

Live Telegram validation remains opt-in because TDLib credentials, native
library loading and account QR authorization are local machine concerns.
Fixture/projection/outbox/realtime tests are the deterministic closure gate.

## Reconciliation Evidence

TDLib can first return an outgoing message with a temporary provider message ID
and later publish `updateMessageSendSucceeded` or `updateMessageSendFailed`.
Макошь persists both initial states and late failure updates. A success update
now rebinds the canonical projection's provider locator in one transaction,
retains the Макошь `message_id`, marks delivery as `sent`, and records an
observation linking the old and final provider ids. A locator collision fails
visibly instead of merging two canonical messages. The accepted event is replay
safe: a repeated projection keeps the original temporary locator evidence and
does not create a second canonical message.

## Sanitized Live Evidence

On 2026-07-12, a locally QR-authorized TDLib account was verified with the
native `make dev` runtime:

- startup reconciliation restored the enabled account and listed 100 dialogs;
- TDLib chat-position updates with client-only or incomplete list metadata were
  ignored without stopping the actor;
- opening a dialog with no local projection performed one latest-history read,
  then projected its returned messages through the Communications event flow;
- the user-facing dialog list and reader were verified in the local browser;
- realtime remains primary, with 15-second dialog and 8-second selected-message
  query fallback while a local event stream reconnects.
