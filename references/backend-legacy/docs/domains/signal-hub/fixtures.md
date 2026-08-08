# Signal Hub Fixtures And Recovery

Status: target fixture and recovery contract with initial implementation.

Signal Hub has two kinds of fixtures:

1. system recovery fixtures;
2. test signal fixtures.

They solve different problems and must not be mixed.

Current implementation note:

- the system recovery fixture is implemented and loaded idempotently from
  `backend/fixtures/signal_hub/system_sources.toml`;
- the initial test signal fixture catalog is implemented at
  `backend/fixtures/signal_hub/test_signals.toml`;
- fixture raw signals can now be emitted through Signal Hub REST/ConnectRPC and
  then flow through the normal `signal_hub_raw_signal_dispatcher` consumer
  path;
- the current test fixture catalog is intentionally narrow and does not yet
  replace the broader provider-specific fixture coverage already present in the
  repository.

## System Recovery Fixture

The system recovery fixture defines the canonical built-in Signal Hub sources
that must exist for Макошь to operate.

It is used for:

- first boot bootstrap;
- migration repair;
- recovery after accidental user deletion of system records;
- consistency checks after schema evolution.

## Hard Rules

The recovery fixture must contain no database references.

Forbidden:

```text
UUID
FK
row id
secret_ref
provider account id
graph id
communication id
task id
document id
```

Allowed:

```text
canonical source code
canonical capability code
canonical profile code
category strings
default booleans
display names
non-secret default settings
```

Reason: the fixture may be loaded during migrations or repair flows when the DB
schema has evolved. Fixed IDs and FK references become lies with confidence,
which is the most annoying kind of lie.

## Suggested Location

```text
backend/src/domains/signal_hub/fixtures/system.toml
```

or, if shared with tooling:

```text
backend/fixtures/signal_hub/system.toml
```

## Example Recovery Fixture

```toml
schema_version = 1

[[sources]]
code = "mail"
display_name = "Mail"
category = "communication"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "messages.read",
  "messages.send",
  "attachments.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "telegram"
display_name = "Telegram"
category = "communication"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "messages.read",
  "messages.send",
  "attachments.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "whatsapp"
display_name = "WhatsApp"
category = "communication"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "messages.read",
  "messages.send",
  "attachments.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "github"
display_name = "GitHub"
category = "development"
default_enabled = false
supports_connections = true
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "issues.read",
  "pull_requests.read",
  "repositories.read",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "browser"
display_name = "Browser"
category = "capture"
default_enabled = false
supports_connections = false
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "browser.capture",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "filesystem"
display_name = "Filesystem"
category = "documents"
default_enabled = false
supports_connections = false
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "files.observe",
  "documents.import",
  "runtime.health_check",
  "runtime.replay",
]

[[sources]]
code = "fixture"
display_name = "Fixture Sources"
category = "test"
default_enabled = true
supports_connections = false
supports_runtime = true
supports_replay = true
supports_pause = true
supports_mute = true
capabilities = [
  "fixture.emit",
  "runtime.health_check",
  "runtime.replay",
]

[[profiles]]
code = "production"
display_name = "Production"

[[profiles]]
code = "development"
display_name = "Development"

[[profiles]]
code = "testing"
display_name = "Testing"

[[profiles]]
code = "maintenance"
display_name = "Maintenance"
```

## Recovery Loader Semantics

The loader must be idempotent.

```text
for each source in fixture:
  if signal_sources.code exists:
    patch missing non-user-owned metadata only
  else:
    create source from current schema mapping

for each capability in source.capabilities:
  if capability exists for source:
    skip
  else:
    create capability definition using current schema mapping
```

Loader must not:

- overwrite user-created connections;
- overwrite secret references;
- overwrite provider runtime sessions;
- delete sources not present in the fixture;
- assume numeric IDs;
- assume current migration shape beyond the loader's own code.

## Test Signal Fixtures

Test fixtures generate deterministic source observations.

Suggested location:

```text
tests/fixtures/signal_hub/
├── telegram_basic.toml
├── mail_basic.toml
├── whatsapp_basic.toml
├── github_issue.toml
└── browser_capture.toml
```

Example:

```toml
schema_version = 1
fixture_id = "telegram_basic_message"
source = "telegram"
event_type = "signal.telegram.message.observed"
source_id = "fixture-telegram-message-001"
occurred_at = "2026-01-01T00:00:00Z"

[payload]
conversation_key = "fixture-chat-1"
message_key = "fixture-message-1"
sender_display_name = "Fixture Sender"
text = "Test message"
```

Test fixture payloads may contain representative text, but must not contain real
private data.

## Fixture Mode

Testing profile should default to:

```text
real sources disabled or muted
fixture source enabled
```

Макошь downstream domains should process fixture events exactly like normal
source events.
