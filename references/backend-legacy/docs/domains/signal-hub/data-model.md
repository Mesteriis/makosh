# Signal Hub Data Model

Status: target data model.

Signal Hub data is split into durable domain state, technical event state and
schema-agnostic fixture definitions.

## Core Entities

### SignalSource

Canonical source type known to Макошь.

Fields:

```text
id
code
display_name
category
source_kind
default_enabled
supports_connections
supports_runtime
supports_replay
supports_pause
supports_mute
capability_schema_version
created_at
updated_at
```

`settings` is non-secret control metadata. It may carry binding keys such as
`account_id` that let Signal Hub map raw provider signals onto a specific
connection scope for policy and replay decisions. Secrets still belong outside
Signal Hub.

`code` is canonical and stable, for example:

```text
mail
telegram
whatsapp
github
browser
rss
calendar
filesystem
home_assistant
voice
fixture_mail
fixture_telegram
fixture_github
```

`code` is the only value a system recovery fixture may depend on.

### SignalConnection

User-created or system-created connection to a source.

Fields:

```text
id
source_code
display_name
status
profile
settings
secret_ref
connected_at
last_seen_at
last_signal_at
last_sync_at
created_at
updated_at
```

Connection status:

```text
not_configured
connecting
awaiting_user_action
connected
degraded
disconnected
paused
muted
disabled
error
removed
```

### SignalCapability

Capability published by a source or runtime.

Fields:

```text
id
source_code
connection_id optional
capability
state
reason
requires_confirmation
action_class
updated_at
```

Capability examples:

```text
messages.read
messages.send
attachments.read
attachments.write
contacts.read
calendar.events.read
calendar.events.write
files.observe
browser.capture
voice.transcribe
runtime.health_check
runtime.replay
runtime.pause
runtime.mute
```

Capability state:

```text
available
degraded
blocked
unsupported
unknown
```

Action classes:

```text
read
write
destructive
admin
recording
export
secret-bearing
```

### SignalRuntime

Current runtime state for a source/connection.

Fields:

```text
id
source_code
connection_id optional
runtime_kind
state
last_started_at
last_stopped_at
last_heartbeat_at
last_error_at
last_error_code
last_error_message_redacted
metadata
updated_at
```

Runtime state:

```text
stopped
starting
running
reconnecting
paused
muted
stopping
error
```

### SignalHealth

Health projection for UI and diagnostics.

Fields:

```text
id
source_code
connection_id optional
level
summary
last_ok_at
last_failure_at
failure_count
consecutive_failure_count
next_retry_at
evidence
updated_at
```

Health levels:

```text
healthy
degraded
failing
disabled
unknown
```

### SignalPolicy

Control-plane policy for mute/pause/filter/replay behavior.

Fields:

```text
id
scope
source_code optional
connection_id optional
event_pattern
mode
reason
created_by
created_at
expires_at optional
metadata
```

Scopes:

```text
global
source
connection
event_pattern
profile
```

Modes:

```text
enabled
disabled
muted
paused
replay_only
fixture_only
```

Examples:

```text
global muted during test
telegram muted
telegram.message.* muted
mail paused
fixture_* enabled for testing profile
```

### SignalProfile

Named policy bundle.

Fields:

```text
id
code
display_name
description
source_policies
is_system
created_at
updated_at
```

Profiles:

```text
production
development
testing
maintenance
```

### SignalReplayRequest

Replay command and audit record.

Fields:

```text
id
source_code optional
connection_id optional
event_type_pattern optional
from_position optional
to_position optional
from_time optional
to_time optional
target_consumer optional
state
requested_by
requested_at
started_at
finished_at
error_message_redacted
```

States:

```text
requested
running
completed
failed
cancelled
```

## Event Model

Signal Hub events:

```text
signal.source.registered
signal.source.enabled
signal.source.disabled
signal.source.muted
signal.source.unmuted
signal.source.paused
signal.source.resumed
signal.source.health_changed
signal.connection.created
signal.connection.updated
signal.connection.removed
signal.capability.changed
signal.profile.applied
signal.replay.requested
signal.replay.completed
signal.replay.failed
```

Source observation events:

```text
signal.mail.message.observed
signal.telegram.message.observed
signal.whatsapp.message.observed
signal.github.issue.observed
signal.browser.page.observed
signal.filesystem.file.observed
signal.calendar.event.observed
```

## Storage Principles

- Signal Hub source-of-truth state is relational PostgreSQL domain state.
- Cross-boundary facts are appended to `event_log`.
- Delivery uses NATS JetStream subjects.
- UI reads projections, not raw domain tables.
- Secrets are referenced by `secret_ref` only.
- Raw provider payloads are not stored in Signal Hub tables.
- System recovery fixtures never contain row IDs, UUIDs, FK values or direct
  table references.

## Suggested Tables

```text
signal_sources
signal_connections
signal_capabilities
signal_runtimes
signal_health
signal_policies
signal_profiles
signal_replay_requests
signal_fixture_sources
```

Projection tables can be separate:

```text
signal_hub_dashboard_projection
signal_hub_source_projection
signal_hub_health_projection
```

## Idempotency

All source observations must carry idempotency material in the event `source`
object:

```json
{
  "kind": "signal_source",
  "source": "telegram",
  "connection_code": "personal_telegram",
  "source_id": "provider-native-event-id"
}
```

If a provider cannot provide a stable source ID, the adapter must derive one
from source code, provider account, timestamp bucket and stable payload hash.
The derived hash must not include secrets or full raw private bodies.
