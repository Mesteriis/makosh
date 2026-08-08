# Signal Hub Gap Analysis

Status: target-vs-current gap analysis for the uploaded repository snapshot.

## Summary

The repository now has the first Signal Hub control-plane implementation on top
of the existing event foundation. The remaining gap is end-to-end migration of
provider producers and consumers onto accepted signal events.

The current final-mile validation gap is concrete: backend
`cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings`
still reports the remaining cleanup in the new Signal Hub/communications wiring,
and frontend `pnpm lint` is now reduced to two oversized production files:
`src/domains/settings/components/SignalHubSettings.vue` and
`src/domains/communications/api/connectCommunications.ts`.

The largest gap is not transport. The largest gap is ownership: Макошь needs a
domain that owns source registry, source runtime policy, health, profiles,
fixtures, mute/pause/replay and source recovery without making Telegram, Mail or
WhatsApp separate product domains.

## Current Strengths

- Event envelope and append-only event log already exist.
- Event consumers already have retry/DLQ direction.
- Communications is already documented as the owner of communication state.
- Telegram and Mail docs already demote providers to integrations/channels.
- Architecture boundary ADRs already prohibit direct domain-to-domain mutation.
- PostgreSQL and Axum are already part of the backend stack.

## Missing Pieces

| Gap | Impact |
|---|---|
| Provider producers still use partial migration paths | Telegram provider-observation events now publish canonical raw signals through the durable outbox-dispatch path, Telegram fixture ingestion now emits canonical raw message signals, Telegram manual send/reply/forward responses now re-enter through canonical raw signals, and the TDLib runtime/history/search/background-command ingest paths now persist raw records through a neutral platform port and publish the same canonical raw Telegram signals without direct `domains::signal_hub` imports; central Mail sync/fixture workflows now publish canonical mail raw signals through the accepted-signal projection entry point, Mail delivery/read-notification callbacks now publish canonical `signal.raw.mail.delivery_status|read_receipt.observed` facts instead of writing Communications directly, and WhatsApp fixture ingestion now emits canonical raw signals too. The current repository does not yet implement a separate WhatsApp live send/reply/forward runtime path, so the remaining gap here is provider breadth outside the implemented Telegram/Mail/WhatsApp fixture-or-central slice rather than an undiscovered WhatsApp write bypass |
| Accepted-signal Communications consumers are not fully wired | Telegram provider-observation projection now accepts `signal.accepted.telegram.*` including the base `signal.accepted.telegram.message` path, Telegram fixture/manual-write/runtime-ingest message projection now routes through the accepted-signal owner helper instead of direct projection primitives, mail message acceptance is wired for central sync/fixture flows through the shared accepted-signal projection entry point, mail delivery/read callbacks now also re-enter Communications only from accepted Signal Hub events, WhatsApp fixture projection now accepts `signal.accepted.whatsapp.message`, and the synchronous accepted-signal helpers now also respect the durable runtime gate of `communication_provider_observation_projection` instead of bypassing paused consumer state. Remaining work here is broader accepted-signal consumer breadth and future provider/runtime surfaces that do not yet exist in the current repository slice |
| Capability coverage is still partial | Signal Hub now materializes first-class generic capability snapshots into durable `signal_capabilities` rows, serves them through REST/ConnectRPC and renders them in the Settings source inspector, but richer provider-specific operation matrices and side effects still primarily live on integration surfaces |
| ConnectRPC coverage is still partial | generated server/client wiring now exists in the build for source list/get/enable/disable, generic scoped disable/enable, capability list, scoped mute/unmute/pause/resume, connection CRUD, health, runtime list/update, policy list/create, replay request create/list, fixture catalog listing, fixture emission and fixture restore; root contracts now also include a provider-neutral `communications/v1/communications.proto` foundation alongside `common`, `events` and `signal_hub`, frontend generated code exists for those roots, and the frontend manifest now explicitly declares the required `@bufbuild/protobuf` and `@connectrpc/*` runtime dependencies instead of relying on transitive installation state; the backend now also serves a first provider-neutral `CommunicationsService` ConnectRPC slice for `ListMessages`, `GetMessage`, `TransitionMessageWorkflowState`, `TrashMessage`, `RestoreMessage`, `MarkMessageRead`, `DeleteMessageFromProvider`, `BulkMessageAction`, `ToggleMessagePin`, `ToggleMessageImportant`, `ToggleMessageMute`, `SnoozeMessage`, `AddMessageLabel`, `RemoveMessageLabel`, `ListMessageWorkflowStateCounts`, `RunWorkflowAction`, `ListSubscriptions`, `GetMailboxHealth`, `ListTopSenders`, `ListCommunicationBlockers`, `ListCommunicationPersonas`, `ListRichTemplates`, `UpsertRichTemplate`, `DeleteRichTemplate`, `RenderRichTemplate`, `PreviewRichTemplateMailMerge`, `SearchMessages`, `AnalyzeMessage`, `GetMessageExplain`, `GetMessageSmartCc`, `GetMessageExport`, `GetMessageAuth`, `GetMessageSignature`, `GenerateAiReply`, `GenerateAiReplyVariants`, `DetectMessageLanguage`, `TranslateMessage`, `ExtractMessageTasks`, `ExtractMessageNotes`, `SearchAttachments`, `GetAttachmentPreview`, `GetAttachmentArchiveInspection`, `TranslateAttachment`, `ListThreads`, `ListThreadMessages`, `TranslateThread`, `ListSavedSearches`, `CreateSavedSearch`, `UpdateSavedSearch`, `DeleteSavedSearch`, `ListFolders`, `CreateFolder`, `UpdateFolder`, `DeleteFolder`, `ListFolderMessages`, `CopyMessageToFolder`, `MoveMessageToFolder`, `ListDrafts`, `CreateDraft`, `DeleteDraft`, `ListOutbox`, `UndoOutboxItem`, confirmed `SendMessage` and `RedirectMessage`, and the frontend now also has a typed wrapper with targeted regression coverage for the current query/command surface; the remaining gap here is wider domain coverage that still uses compatibility paths, not the main Communications UI/API surface |
| Frontend boundary coverage is now only partially remaining | targeted frontend tests now prove generated Signal Hub client wrapper calls, query-key stability, realtime invalidation for the declared `signal.raw/accepted/rejected/muted/paused/resumed/replayed` families, the rule that Signal Hub stays under Settings without direct integration imports, and that the Settings component keeps rendering connection/runtime/health diagnostics from Settings-domain data instead of flattening them away; broader interactive component behavior coverage still remains if the UI surface grows materially |
| Signal profiles are still partial | system and custom profiles are now persisted, listed, created, updated, removed and applicable through Signal Hub, but richer profile composition beyond the current policy-list editor is still missing |
| Connection control is still partial | create/update/remove now exist for `signal_connections`, the Settings UI now also exposes connection-scoped policy/replay selectors on top of the backend support, connection status now reconciles connection-scoped operator policies for `paused`, `muted` and `disabled`, and Mail/Telegram/WhatsApp bootstrap+lifecycle handlers now auto-sync provider accounts into Signal Hub connection rows; richer settings validation, provider breadth and capability-driven side effects still remain |
| Health controls are still partial | health can now be listed and recomputed through Signal Hub run-health-check commands, and the `ai` source now has a source-specific runtime/models probe, but richer provider-specific ping/retry orchestration and capability-aware remediation are still missing |
| Dynamic subscriber/workflow controls are only partially implemented | core subscriber/scheduler runtime state can now be listed and switched through `signal_runtime_states`, bootstrap loops honor `running/paused/muted/stopped`, source-level controls now reconcile existing and lazy runtime rows with priority `disabled > paused > muted > running`, the synchronous Telegram/Mail/WhatsApp raw-signal helpers now respect the same `signal_hub_raw_signal_dispatcher` runtime gate before emitting accepted signals, the replay dispatcher also runs behind the same durable `system` runtime control surface, owner-facing AI command routes now respect the durable `ai_request_runtime` gate, communication-facing AI helpers for translation/reply drafting/task extraction now also respect that same `ai_request_runtime` gate and fall back cleanly when `ai` is muted or disabled, the current owner-facing AI run lifecycle/task-extraction path now also emits canonical `signal.raw.ai.*` and accepted `signal.accepted.ai.*` facts, the runtime-backed message translation, thread message translation, bilingual reply flow translations, LLM task extraction, note extraction, reply drafting/reply variant generation and attachment translation helper paths now also emit canonical `signal.raw.ai.*` and accepted `signal.accepted.ai.*` facts where LLM execution actually occurs, the current backend regression suite explicitly proves those named communication-facing helper slices, and dedicated ConnectRPC regressions now prove that pausing then resuming both `signal_hub_raw_signal_dispatcher` and `communication_provider_observation_projection` changes raw-to-accepted publication and accepted-signal materialization behavior immediately without process restart; the live Telegram TDLib runtime bridge now respects its own durable `telegram_runtime_event_bridge` gate before publishing runtime events; provider-wide breadth, broader AI source-event coverage beyond the current owner-facing path, and richer cross-source command semantics remain |
| Replay semantics are still partial | replay requests can now be created and executed through the background dispatcher for paused buffered raw signals, for event-log replay by raw-signal pattern plus position/time range, for consumer-targeted replay that rewinds one consumer cursor over a selected signal slice while preserving consumer idempotency markers, and for projection-targeted rebuild paths `timeline_event_log`, `communication_messages`, `persona_derived_evidence` and `project_link_review_effects` that rewind either a projection cursor or the matching consumer cursor before replay while emitting `timeline.projection.updated`, `communications.projection.updated`, `personas.derived_evidence.updated` or `projects.link_review_effects.updated`; connection-scoped replay also works through non-secret `settings.account_id` bindings, but broader projection rebuild orchestration still remains |
| NATS dispatcher breadth is still narrow | outbox-to-JetStream dispatch is now wired and tested, and the same dispatcher now fans published events into the in-memory realtime bus for websocket delivery, but only the migrated Signal Hub write paths currently rely on it; broader event-source adoption is still pending |

## Migration Risks

- Duplicating event systems instead of extending `platform/events`.
- Treating Signal Hub as another integration folder.
- Letting Signal Hub write Communications/Tasks/Documents tables directly.
- Storing provider secrets or raw message bodies in Signal Hub state.
- Encoding fixture row IDs or FK references that break on future migrations.
- Introducing a connector before its crate boundary, durable acknowledgement
  and lease-fencing contracts are stable.
- Adding Redis as a second event system without a clear ownership problem.

## Closure Conditions

Signal Hub documentation can be considered implemented when:

1. `backend/src/domains/signal_hub` exists. Done.
2. Signal Hub tables and projections exist. Partially done for core tables.
3. System recovery fixture exists and is loaded idempotently. Done. The
   backend restore path now actually loads canonical source definitions from
   `backend/fixtures/signal_hub/system_sources.toml` instead of maintaining a
   second hardcoded source catalog in Rust.
4. Signal Hub can list and control built-in source definitions. Partially done:
   source list/get/restore and source enable/disable now exist, profile
   list/apply exists, scoped mute/unmute/pause/resume exists, connection list
   and metadata/status control exist, provider bootstrap/lifecycle handlers now
   auto-sync Mail/Telegram/WhatsApp accounts into those connection rows,
   runtime list/update exists, health list exists and policy writes still
   exist for lower-level control; richer provider-side connection workflows
   remain.
5. Real source publication can be globally and selectively muted. Core policy
   evaluator and policy API exist for raw signals, including `system` and `ai`,
   and connection-scoped raw-signal policy can now bind through non-secret
   `settings.account_id`;
   Telegram provider-observation, fixture, TDLib runtime/history/search/
   background-command ingest and current send/reply/forward response producer
   migration is in place through canonical raw signals; central Mail sync/
   fixture producer migration is in place; WhatsApp fixture producer migration
   is in place; remaining provider breadth and subscriber controls remain.
6. Fixture sources can emit deterministic events through the normal EventBus.
   Done for the initial Signal Hub fixture catalog: fixture raw signals can now
   be emitted through REST/ConnectRPC and then consumed by the normal
   `signal_hub_raw_signal_dispatcher` path. Provider-specific fixture breadth
   still remains limited.
7. Event write path supports PostgreSQL event log and NATS JetStream transport.
   Partially done: event log/outbox/NATS dispatcher path is implemented and
   covered by targeted integration tests, but broader producer adoption remains.
8. Signal Hub UI reads projections and uses generated ConnectRPC clients.
   Partially done: Settings UI now uses generated ConnectRPC clients for
   sources, source enable/disable, scoped mute/unmute/pause/resume, connection
   CRUD, runtime states, health, policy list/create, replay request create/list
   with pattern, position/time selectors and optional target consumer or
   projection target, fixture catalog listing, fixture emission and fixture
   restore; backend ConnectRPC now also serves the first provider-neutral
   `CommunicationsService` slice for message/thread/saved-search/folder/draft/
   outbox, local-state/bulk-action, message-flag/label/snooze, multilingual/
   AI-helper and attachment query/command surfaces, including
   workflow-state transition, workflow-state counts, local trash/restore,
   mark-read, provider-delete alias, bulk message action, pin/important/mute
   toggles, snooze, label add/remove, message search, message
   analysis/explain/smart-cc, message export, auth review, signature
   detection, AI reply drafting/variants, detect-language,
   single-message translation, task extraction, note extraction, attachment
   search, attachment preview, attachment archive inspection, attachment
   translation, thread translation, saved-search CRUD, folder CRUD/message
   actions, draft create/delete, outbox undo, send enqueue and redirect
   enqueue, and the current main
   provider-neutral frontend query entrypoints for messages, message detail,
   saved searches, folders, folder messages, drafts, outbox, threads, thread
   messages, attachment helpers and the current provider-neutral multilingual/
   AI-helper calls plus workflow-state transition, workflow-state counts,
   message search, mark-read, provider-delete alias, message
   analysis/explain/smart-cc, export/auth/signature, AI reply/variants,
   thread translation, saved-search CRUD, folder CRUD/message actions, draft
   save/delete, `sendEmail`, `redirectMessage`, communication personas,
   subscription/health/sender/blocker reads, rich-template CRUD/render/preview,
   workflow actions and outbox undo already use that ConnectRPC layer; broader
   domain surfaces still rely on compatibility APIs or are not implemented yet.
9. SSE updates Signal Hub UI state. Done for cache invalidation of `signal.*`.
10. `make validate` passes without architecture boundary exceptions. Pending
    the remaining backend `clippy` cleanup and the split of the two oversized
    frontend production files named above.
