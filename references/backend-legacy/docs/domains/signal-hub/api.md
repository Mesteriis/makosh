# Signal Hub API

Status: target ConnectRPC API.

Signal Hub APIs are command/query APIs for the local owner and UI. They do not
expose raw provider protocols. They must be implemented as contract-first
Protobuf + ConnectRPC services, with Axum hosting the HTTP transport.

REST endpoints may exist only as temporary compatibility shims during migration.
They must not become the canonical API contract.

## Service

```proto
service SignalHubService {
  rpc ListSources(ListSourcesRequest) returns (ListSourcesResponse);
  rpc GetSource(GetSourceRequest) returns (GetSourceResponse);
  rpc ListCapabilities(ListCapabilitiesRequest) returns (ListCapabilitiesResponse);
  rpc ListFixtureSources(ListFixtureSourcesRequest) returns (ListFixtureSourcesResponse);
  rpc ListConnections(ListConnectionsRequest) returns (ListConnectionsResponse);
  rpc CreateConnection(CreateConnectionRequest) returns (CreateConnectionResponse);
  rpc UpdateConnection(UpdateConnectionRequest) returns (UpdateConnectionResponse);
  rpc RemoveConnection(RemoveConnectionRequest) returns (RemoveConnectionResponse);

  rpc EnableSource(EnableSourceRequest) returns (EnableSourceResponse);
  rpc DisableSource(DisableSourceRequest) returns (DisableSourceResponse);
  rpc DisableSignals(DisableSignalsRequest) returns (DisableSignalsResponse);
  rpc EnableSignals(EnableSignalsRequest) returns (EnableSignalsResponse);
  rpc MuteSignals(MuteSignalsRequest) returns (MuteSignalsResponse);
  rpc UnmuteSignals(UnmuteSignalsRequest) returns (UnmuteSignalsResponse);
  rpc PauseSignals(PauseSignalsRequest) returns (PauseSignalsResponse);
  rpc ResumeSignals(ResumeSignalsRequest) returns (ResumeSignalsResponse);

  rpc ListHealth(ListHealthRequest) returns (ListHealthResponse);
  rpc RunHealthCheck(RunHealthCheckRequest) returns (RunHealthCheckResponse);
  rpc ListRuntimeStates(ListRuntimeStatesRequest) returns (ListRuntimeStatesResponse);
  rpc UpdateRuntimeState(UpdateRuntimeStateRequest) returns (UpdateRuntimeStateResponse);
  rpc ListPolicies(ListPoliciesRequest) returns (ListPoliciesResponse);
  rpc CreatePolicy(CreatePolicyRequest) returns (CreatePolicyResponse);

  rpc ListProfiles(ListProfilesRequest) returns (ListProfilesResponse);
  rpc CreateProfile(CreateProfileRequest) returns (CreateProfileResponse);
  rpc UpdateProfile(UpdateProfileRequest) returns (UpdateProfileResponse);
  rpc RemoveProfile(RemoveProfileRequest) returns (RemoveProfileResponse);
  rpc ApplyProfile(ApplyProfileRequest) returns (ApplyProfileResponse);

  rpc RequestReplay(RequestReplayRequest) returns (RequestReplayResponse);
  rpc ListReplayRequests(ListReplayRequestsRequest) returns (ListReplayRequestsResponse);
  rpc EmitFixtureSignal(EmitFixtureSignalRequest) returns (EmitFixtureSignalResponse);
  rpc RestoreSystemFixture(RestoreSystemFixtureRequest) returns (RestoreSystemFixtureResponse);
}
```

Current implemented contract note: the generated ConnectRPC slice now carries
non-secret `settings_json` for connections, `evidence_json` for health rows,
runtime timestamps/error diagnostics and durable capability rows in addition to
the base control/query surface; profile responses now also carry their policy
definitions so custom profile authoring/editing can stay inside Settings
without dropping back to ad hoc JSON or REST-only shims.

The same root contract set now also has a live provider-neutral
`makosh.communications.v1.CommunicationsService` backend slice for:

- `ListMessages`
- `GetMessage`
- `TransitionMessageWorkflowState`
- `TrashMessage`
- `RestoreMessage`
- `MarkMessageRead`
- `DeleteMessageFromProvider`
- `BulkMessageAction`
- `ToggleMessagePin`
- `ToggleMessageImportant`
- `ToggleMessageMute`
- `SnoozeMessage`
- `AddMessageLabel`
- `RemoveMessageLabel`
- `ListMessageWorkflowStateCounts`
- `RunWorkflowAction`
- `ListSubscriptions`
- `GetMailboxHealth`
- `ListTopSenders`
- `ListCommunicationBlockers`
- `ListCommunicationPersonas`
- `ListRichTemplates`
- `UpsertRichTemplate`
- `DeleteRichTemplate`
- `RenderRichTemplate`
- `PreviewRichTemplateMailMerge`
- `SearchMessages`
- `AnalyzeMessage`
- `GetMessageExplain`
- `GetMessageSmartCc`
- `GetMessageExport`
- `GetMessageAuth`
- `GetMessageSignature`
- `GenerateAiReply`
- `GenerateAiReplyVariants`
- `DetectMessageLanguage`
- `TranslateMessage`
- `ExtractMessageTasks`
- `ExtractMessageNotes`
- `SearchAttachments`
- `GetAttachmentPreview`
- `GetAttachmentArchiveInspection`
- `TranslateAttachment`
- `ListThreads`
- `ListThreadMessages`
- `TranslateThread`
- `ListSavedSearches`
- `CreateSavedSearch`
- `UpdateSavedSearch`
- `DeleteSavedSearch`
- `ListFolders`
- `CreateFolder`
- `UpdateFolder`
- `DeleteFolder`
- `ListFolderMessages`
- `CopyMessageToFolder`
- `MoveMessageToFolder`
- `ListDrafts`
- `CreateDraft`
- `DeleteDraft`
- `ListOutbox`
- `UndoOutboxItem`
- `SendMessage`
- `RedirectMessage`

That communications slice currently reuses existing Communications stores and
confirmed send path under the same router-level `X-Макошь-Secret` boundary; it
does not replace all legacy REST endpoints yet. The frontend now also exposes a
dedicated typed wrapper around this `communications/v1` service for targeted
query/command usage and regression coverage, and the current provider-neutral
frontend query entrypoints for messages, message detail, saved searches,
folders, folder messages, drafts, outbox, threads, thread messages,
attachment search, attachment preview, attachment archive inspection,
attachment translation, message analysis, message explain, smart-cc,
message export, SPF/DKIM auth review, signature detection,
AI reply drafting and reply variants, detect-language, single-message
translation, task extraction, note extraction, workflow-state transition,
workflow-state counts, workflow actions, local trash/restore, mark-read, provider-delete
alias, bulk message action, pin/important/mute, snooze, message labels,
message search, subscriptions, mailbox health, top senders, blockers,
communication personas,
`sendEmail`, `redirectMessage`,
`translateThread`, saved-search CRUD, folder CRUD/message actions, draft
save/delete, rich-template CRUD/render/preview and outbox undo already use this ConnectRPC layer.
The remaining
legacy REST surface is now concentrated in still-unmigrated provider-specific
operations elsewhere in the repository rather than the main Communications UI path.

## Realtime

Realtime is not ConnectRPC streaming in the first browser surface. Browser
updates use Axum SSE by default:

```text
GET /api/v1/events/stream
```

WebSocket delivery is also available through the shared event realtime bus:

```text
GET /api/events/realtime/ws
```

Realtime event families:

```text
signal.source.updated
signal.connection.updated
signal.health.updated
signal.policy.updated
signal.replay.updated
projection.signal_hub.updated
```

The frontend patches generated-client query caches from these event families.
SSE replays directly from durable `event_log`; websocket delivery now also
follows the persisted outbox-dispatch path for published `signal.*` events. The
realtime layer does not replace durable event processing.

## Command Semantics

### EnableSource

Enables source runtime and publication policy.

Current implementation also resumes existing durable `signal_runtime_states`
rows for the same `source_code` back to `running`.

Must emit:

```text
signal.source.enabled
```

### DisableSource

Stops source runtime and prevents capture/publication.

Current implementation also moves existing durable `signal_runtime_states` rows
for the same `source_code` to `stopped`.

Must emit:

```text
signal.source.disabled
```

### DisableSignals

Applies a `disabled` policy to `global`, `source`, `connection` or
`event_pattern` scope without forcing callers through the source-only endpoint.

Current implementation uses the same policy evaluator priority as all other
runtime and publication controls: `disabled > paused > muted > running`.

Must emit:

```text
signal.signals.disabled
```

### EnableSignals

Clears matching scoped `disabled` policies for `global`, `source`,
`connection` or `event_pattern`.

Must emit:

```text
signal.signals.enabled
```

### MuteSignals

Suppresses publication according to scope and pattern. Runtime may stay active.

Current implemented scopes:

```text
global
source
connection
event_pattern
```

`profile` remains a higher-level composition path through `ApplyProfile`, not a
direct command scope in the current control API implementation.

Must emit:

```text
signal.source.muted
signal.policy.changed
```

### PauseSignals

Captures/buffers eligible signals but does not publish them to downstream
consumers until resumed.

Must emit:

```text
signal.source.paused
signal.policy.changed
```

### ResumeSignals

Clears matching scoped `paused` policies and lets eligible runtimes/processors
continue immediately on the next tick or synchronous runtime gate check.

Must emit:

```text
signal.source.resumed
signal.policy.changed
```

### UnmuteSignals

Clears matching scoped `muted` policies and restores publication for the
selected scope.

Must emit:

```text
signal.source.unmuted
signal.policy.changed
```

### UpdateRuntimeState

Applies a durable runtime state override for a concrete `source_code` plus
`runtime_kind` row.

Current implementation supports at least:

```text
running
paused
muted
stopped
```

These runtime rows are consulted live by subscriber loops and synchronous
Signal Hub helper gates; no process restart is required for the new state to
take effect.

### CreatePolicy

Creates a durable policy row directly.

Current implementation supports:

```text
scope: global | source | connection | event_pattern
mode: disabled | paused | muted
```

The Settings UI mostly uses higher-level control commands for toggles, but the
policy create/list path remains part of the canonical contract and is covered
by the generated client.

### RequestReplay

Creates a replay request. Replay consumes from event store / NATS JetStream / fixture
catalog depending on request scope.

Current implementation supports:

```text
paused-buffer replay into accepted-signal flow
event-log replay by pattern / position / time selectors
consumer-targeted replay by rewinding one consumer cursor over the selected signal slice
projection-targeted replay for `timeline_event_log` by rewinding the projection cursor and emitting `timeline.projection.updated`
projection-targeted replay for `communication_messages` by clearing processed markers for the accepted-signal Communications consumer, rewinding its cursor over the selected signal slice and emitting `communications.projection.updated`
```

Must emit:

```text
signal.replay.requested
signal.replay.completed
signal.replay.failed
```

### RestoreSystemFixture

Restores missing system Signal Hub source definitions from the schema-agnostic
fixture. It must never overwrite user-owned connection secrets or provider
runtime sessions.

Must emit:

```text
signal.fixture.restore_requested
signal.fixture.restored
signal.fixture.restore_failed
```

## Query Semantics

Queries read projections where possible:

```text
signal_hub_dashboard_projection
signal_hub_source_projection
signal_hub_health_projection
```

They must not perform expensive cross-domain joins. If a query needs
Communications, Radar or provider runtime details, it should use read-model
composition at app/BFF level, not mutate ownership boundaries.

## Authorization

Initial local API auth remains the repository's local protected API pattern.
Signal Hub commands are owner-local admin commands and should be guarded as
sensitive local operations.

Command classes:

| Command | Action class |
|---|---|
| EnableSource | admin |
| DisableSource | admin |
| DisableSignals | admin |
| EnableSignals | admin |
| MuteSignals | admin |
| UnmuteSignals | admin |
| PauseSignals | admin |
| ResumeSignals | admin |
| UpdateRuntimeState | admin |
| CreatePolicy | admin |
| RequestReplay | admin/export-sensitive when payload may be replayed |
| EmitFixtureSignal | test/admin |
| RestoreSystemFixture | recovery/admin |
| CreateConnection | secret-bearing when it starts auth |
| RemoveConnection | destructive |

## Error Model

Errors should be typed and redacted:

```text
SOURCE_NOT_FOUND
CONNECTION_NOT_FOUND
CAPABILITY_UNAVAILABLE
POLICY_CONFLICT
SOURCE_DISABLED
SOURCE_MUTED
SOURCE_PAUSED
REPLAY_RANGE_INVALID
REPLAY_ALREADY_RUNNING
FIXTURE_INVALID
FIXTURE_RESTORE_FAILED
SECRET_REF_MISSING
RUNTIME_UNAVAILABLE
```

Errors must not expose secret values, raw message bodies or provider session
material.
