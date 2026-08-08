# Макошь Signal Hub

Status: `IMPLEMENTATION STARTED`, 2026-06-23.

Signal Hub is the system control plane for external and synthetic signal
sources in Макошь. It is not a messenger UI, not an email client and not a
provider-specific integration folder. Signal Hub owns the durable registry of
sources, connections, capabilities, runtime state, health, profiles, mute/pause
policies and recovery fixtures.

Макошь receives signals from the world and turns them into Communications,
Radar, Review, domain objects, Memory and Knowledge. Signal Hub controls the
first boundary of that chain.

```text
External World
  -> Signal Hub
  -> Event Backbone
  -> Communications / Calendar / Documents / Tasks / Knowledge
  -> Radar
  -> Review
  -> Domain Objects
  -> Memory / Knowledge / Projections
```

## Position

Signal Hub is a system domain. It manages all signal sources, not only
communication providers.

Examples:

- Email / Mail;
- Telegram;
- WhatsApp;
- GitHub;
- Browser capture;
- RSS;
- Calendar providers;
- Filesystem watchers;
- Home Assistant;
- voice input;
- deterministic fixture sources.
- system/internal runtime sources;
- local AI runtime sources.

Signal Hub does not own provider protocol code. Provider protocol/runtime code
continues to live under `backend/src/integrations/*`. Signal Hub owns the
source registry and control state used to decide whether a source can publish,
be muted, be paused, replayed, restored or used in tests.

## Core Invariants

- A provider is not a domain.
- A source is not automatically a Communication.
- A signal is evidence from the external or synthetic world.
- Signal Hub controls sources and signal flow.
- No source class is hardcoded as unpausable or unmutable: system, internal and
  AI signals must go through the same policy model as provider signals.
- Event Backbone transports versioned events.
- Communications owns messages, conversations, participants and attachments.
- Radar owns attention and candidate incubation.
- Review owns promotion decisions.
- Domain objects are created by their owning domains.
- AI can suggest candidates, but cannot become source of truth.

## Current Repository Context

The repository already contains:

- append-only `event_log` migration;
- canonical `EventEnvelope` model;
- an in-process broadcast `EventBus`;
- durable event consumers, retry state and DLQ tables;
- Communications as the provider-neutral message domain;
- Telegram and Mail documentation that states channels are integrations, not
  domains.

Signal Hub now has the first backend and Settings UI foundation:

- `backend/src/domains/signal_hub`;
- Signal Hub source registry tables;
- schema-agnostic system source recovery fixture;
- raw signal policy processing for accept, reject, mute and pause;
- runtime state control for core subscriber and scheduler loops through
  `signal_runtime_states`;
- connection metadata/status lifecycle control through `signal_connections`;
- PostgreSQL event outbox foundation;
- NATS JetStream adapter, bootstrap dispatcher with in-memory realtime fan-out
  for published events, and local development service;
- generated ConnectRPC server/client slice for source listing, connection
  get/enable/disable, generic scoped disable/enable, scoped
  mute/unmute/pause/resume, connection listing/create/update/remove, runtime
  state listing/updates, health listing, policy list/create, replay request
  create/list, fixture catalog listing, fixture emission and fixture restore;
- protected local REST compatibility endpoints still exist for compatibility
  and for the remaining not-yet-migrated surfaces;
- root Protobuf contract files plus generated Rust/TypeScript code for the
  implemented service slice;
- Settings UI section for source registry, connections, runtime state, health,
  replay request create/list with pattern and position/time selectors,
  policies and recovery state.

Remaining implementation work includes broader provider producer migration,
accepted-signal Communications consumers beyond the current slices, broader
replay semantics and broader UI/control coverage for the not-yet-migrated
surfaces.

Current migration note: Telegram provider-observation events now enter Макошь as
`signal.raw.telegram.*.observed` and the Communications projection consumes the
accepted Signal Hub family for that slice. These provider-observation raw
events now also use the durable outbox-dispatch path instead of only appending
to `event_log`. Central Mail sync and email fixture
workflows now emit `signal.raw.mail.message.observed` and Communications
projects mail messages only from accepted Signal Hub events through the shared
accepted-signal projection entry point. Mail delivery-status and read-receipt
callbacks now also publish canonical raw Signal Hub events
(`signal.raw.mail.delivery_status.observed` and
`signal.raw.mail.read_receipt.observed`) before the accepted-signal consumer
updates Communications outbox/read-receipt state. WhatsApp fixture ingestion now emits
`signal.raw.whatsapp.message.observed`, and Communications project WhatsApp
messages only from `signal.accepted.whatsapp.message`. The current repository
slice for WhatsApp still exposes fixture ingest plus session/message read
surfaces, but does not yet implement a separate live send/reply/forward runtime
write path. The legacy
`integration.telegram.*` fallback has been removed from the Communications
projection path. Telegram fixture ingestion now also emits
`signal.raw.telegram.message.observed`, and its message projection runs only
from `signal.accepted.telegram.message` before the legacy compatibility UI event
is emitted. The Communications accepted-signal owner now also consumes the base
`signal.accepted.telegram.message` path directly, and workflow/app callers use
the same owner helper instead of calling projection primitives themselves.
Telegram manual send/reply/forward response projection now also
stores the raw record first and re-enters Communications only through accepted
Signal Hub events. Provider breadth outside the implemented
Telegram/Mail/WhatsApp fixture-or-central slice remains part of the outstanding
migration work. Targeted backend regression coverage for Telegram and WhatsApp
message seeding now also follows the same raw -> Signal Hub -> accepted ->
Communications path instead of calling the old direct projection helper. The
old production-facing application shim for provider direct projection has now
been removed, and the legacy direct projection helper has been deleted
entirely. TDLib runtime-created messages, TDLib history/search ingestion and
background Telegram command reconciliation now also persist provider raw
records through a neutral platform raw-record port and publish
`signal.raw.telegram.message.observed` without importing `domains::signal_hub`
from the integration runtime layer.

Current runtime note: the bootstrap-managed loops now register and honor
runtime state for core subscribers/schedulers. Users can pause, mute, stop or
resume these loops through the Settings Signal Hub runtime section, including
the `event_outbox_dispatcher` that controls PostgreSQL-to-JetStream fan-out.
`EnableSource` and `DisableSource` now also orchestrate the durable
`signal_runtime_states` rows for existing source-owned loops, so source-level
runtime control no longer lives only in the separate runtime panel.
Published outbox events now also re-enter the in-memory realtime bus from the
same dispatcher, so websocket clients observe the same persisted `signal.*`
event families that already exist in `event_log` and JetStream.
The synchronous Telegram/Mail/WhatsApp raw-signal helper paths now consult the
same durable runtime state for `signal_hub_raw_signal_dispatcher` before
attempting immediate acceptance, so pausing or stopping that system dispatcher
also blocks direct helper-driven accepted-signal emission and leaves only the
durable raw fact queued for later processing.
The live TDLib-backed Telegram runtime event bridge now also owns a durable
`telegram_runtime_event_bridge` runtime row under the `telegram` source, so
source-level runtime control and the runtime panel can pause live Telegram
subscription event publication before raw Signal Hub append/broadcast.
The synchronous accepted-signal projection helpers used by fixture ingest,
mail sync/fixture flows and Telegram manual-send projection now also consult
the durable runtime state of `communication_provider_observation_projection`
before materializing `communication_messages`, so pausing that consumer no
longer leaves hidden sync app/workflow bypasses around the accepted-signal
owner path.
Lazy runtime rows now also inherit source-level disabled state: if a source is
disabled before a given runtime kind appears for the first time, the first gate
check creates that runtime row in `stopped` instead of silently defaulting it
back to `running`.
That inheritance now follows the same source-control priority everywhere:
`disabled > paused > muted > running`. Source-level pause/mute/unpause/resume
reconcile existing runtime rows to the same state that future lazy runtime rows
will get on first use.
Dedicated
runtime-change SSE events are not complete yet; current UI refresh for these
controls relies on mutation invalidation plus the existing Signal Hub reads.

Current replay note: Signal Hub can now create replay requests through REST
compatibility and ConnectRPC, and the background replay dispatcher executes
queued requests against the paused-event buffer, against raw-signal slices
selected from `event_log` by position/time range, and against one selected
consumer by rewinding that consumer cursor over a matching signal slice.
Replayed raw events re-enter the accepted-signal path with replay provenance.
Connection-scoped replay now also works for raw events that carry `account_id`
and match a Signal Hub connection bound through non-secret
`settings.account_id`. Consumer-targeted replay intentionally preserves
consumer idempotency markers, so it reopens missed or dead-lettered work for a
single consumer without acting as a full projection rebuild. The current
Settings UI now exposes pattern replay, position/time selectors and an
optional target consumer. Replay requests can now also target the first
projection rebuild paths: `timeline_event_log` rewinds the projection cursor,
replays the selected event-log slice through the shared Timeline Engine mapper
and emits `timeline.projection.updated`, while `communication_messages`
clears processed markers for the accepted-signal Communications consumer,
rewinds that consumer cursor over the selected signal slice and emits
`communications.projection.updated`. Signal Hub now also supports first-class
projection-targeted rebuilds for `persona_derived_evidence` and
`project_link_review_effects`, each rewinding the matching consumer cursor,
clearing processed markers for the selected event-log slice and emitting
`personas.derived_evidence.updated` or `projects.link_review_effects.updated`.
Broader projection rebuild coverage is still separate work for future targets.

Current fixture note: Signal Hub now has an initial deterministic fixture
catalog and can emit fixture raw signals through REST compatibility and
ConnectRPC. These raw fixture events then flow through the normal
`signal_hub_raw_signal_dispatcher` path into accepted/rejected/muted/paused
Signal Hub outcomes. The current catalog is still intentionally narrow and does
not yet replace the broader provider-specific fixture coverage already present
elsewhere in the repository. The current Settings Signal Hub UI can list the
catalog through ConnectRPC and trigger fixture emission from the `fixture`
source inspector without typing fixture ids manually.

Current connection note: the Settings Signal Hub connection section can now
create, update and remove Signal Hub connection records and switch their
status/profile metadata. Raw-signal policy and replay can now bind a connection
scope through non-secret `settings.account_id` when provider raw events publish
the same `account_id` in source/subject/provenance metadata, and the current
Settings UI now exposes those connection-scoped policy and replay selectors
directly instead of leaving them API-only. Connection status updates now also
reconcile connection-scoped operator policies for `paused`, `muted` and
`disabled`, so an operator changing connection state no longer needs to issue a
separate policy mutation just to affect signal flow. Mail account
setup/import/logout/delete, Telegram account setup/logout/remove and WhatsApp
fixture-account setup now also auto-sync provider accounts into
`signal_connections`, so Signal Hub control surfaces observe provider lifecycle
changes without a second manual registration step. Richer provider-specific
authorization/capability side effects are still separate work.

Current capability note: Signal Hub now materializes first-class generic
capability snapshots into durable `signal_capabilities` rows and exposes them
through both REST compatibility and generated ConnectRPC. The current Settings
source inspector renders those rows directly, including control-plane
capabilities such as `signals.observe`, `connections.manage`,
`runtime.health_check`, `runtime.pause`, `runtime.mute` and `runtime.replay`
plus a small set of source-specific generic capabilities. Capability `state`
now also reflects the effective source-level policy state
(`available/degraded/blocked`) for cases such as muted, paused or disabled
system, AI and provider sources. Richer
provider-specific operation matrices and side effects are still separate work.

Current frontend-validation note: the Signal Hub Settings slice now has
targeted regression tests for generated client wrappers, query-key stability,
boundary rules that keep Signal Hub under Settings and prevent direct imports
from integration internals, plus explicit realtime invalidation coverage for
the declared `signal.raw.*`, `signal.accepted.*`, `signal.rejected.*`,
`signal.muted.*`, `signal.paused.*`, `signal.resumed.*` and
`signal.replayed.*` families.

Current profile note: Signal Hub now restores and persists system profiles,
tracks the active profile through application settings and applies
profile-managed policies through Signal Hub itself. The current UI can list,
create, update, remove and apply custom profiles while leaving fixture-owned
system profiles immutable. Richer profile composition beyond the current
policy-list editor is still separate work.

Current health note: Signal Hub health is no longer read-only. The current
control-plane slice can run health checks through REST compatibility and
ConnectRPC, recompute durable `signal_health` rows from source/connection/runtime
state and emit `signal.source.health_changed`. The `ai` source now also uses a
source-specific runtime probe that checks configured AI runtime/model
availability and stores that result in Signal Hub health instead of reporting
only generic runtime-row state. This is still not a full provider-specific
ping/remediation engine for every source family.

Current AI runtime-control note: owner-facing AI command routes now also honor
the durable Signal Hub runtime gate for source `ai`, so `muted`, `paused` or
`disabled` source-level control blocks answer/task-refresh/meeting-prep
execution before the runtime client is invoked. Those owner-facing AI run
lifecycles now also emit canonical `signal.raw.ai.run_requested.observed`,
`signal.raw.ai.run_completed.observed` and
`signal.raw.ai.task_extraction.observed` events, and the allowed paths
materialize matching `signal.accepted.ai.*` facts through the normal Signal
Hub decision flow instead of remaining outside source-level policies and
replay. The same `ai` source gate now also controls communication-facing AI
helpers such as message translation, reply drafting and LLM task extraction:
when `ai` is muted, paused or disabled those handlers fall back to their
existing no-LLM/heuristic behavior instead of invoking the runtime anyway. The
current communication helper slice now also emits canonical
`signal.raw.ai.message_translation.observed` and
`signal.raw.ai.message_task_extraction.observed` events, with the allowed paths
materializing matching accepted `signal.accepted.ai.message_*` facts through
the same Signal Hub decision flow. Runtime-backed reply drafting and attachment
translation now also publish canonical `signal.raw.ai.reply_drafting.observed`,
`signal.raw.ai.reply_variant_generation.observed` and
`signal.raw.ai.attachment_translation.observed` events, again flowing through
normal Signal Hub accepted/muted/paused/rejected handling. Thread-level message
translation now also emits canonical
`signal.raw.ai.thread_message_translation.observed` per translated message
instead of remaining a silent batch helper. The bilingual reply flow now also
emits canonical `signal.raw.ai.bilingual_reply_inbound_translation.observed`
and `signal.raw.ai.bilingual_reply_back_translation.observed` events for its
two runtime-backed translation steps. The current targeted backend regression
coverage now explicitly proves publication for message translation, thread
translation, task extraction, reply drafting, reply variant generation,
attachment translation and bilingual reply flow translations.

Current API note: generated ConnectRPC server/client wiring is now live for
source listing/get plus capability listing, source enable/disable, scoped
mute/unmute/pause/resume, connection create/update/remove, health listing,
runtime listing/updates, policy list/create, replay request create/list,
fixture catalog listing, fixture emission and fixture restore. REST
compatibility endpoints still exist for compatibility and for the remaining
not-yet-migrated surfaces.

## Navigation

- [Architecture](architecture.md)
- [Data Model](data-model.md)
- [Modules](modules.md)
- [API Reference](api.md)
- [Operations](operations.md)
- [Testing](testing.md)
- [Fixtures And Recovery](fixtures.md)
- [Status](status.md)
- [Gap Analysis](gap-analysis.md)
- [Blockers](blockers.md)
