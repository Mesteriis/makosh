# ADR-0181: Backend Workspace Modularity and Provider Runtime Topology

Status: Accepted
Date: 2026-07-13

Supersedes: ADR-0099 Signal Hub Event Platform

Implementation topology and migration roadmap: superseded by ADR-0184.

The Signal Hub, canonical event, evidence, fixture-first and provider-isolation
principles in this ADR remain inputs to the clean-room design. Its concrete
workspace layout is no longer the target architecture.

Clarifies:

- ADR-0076 Host Vault on macOS
- ADR-0097 Communications Channel Domains To Integrations
- ADR-0098 Provider-Neutral Communications API And Strict Boundaries
- ADR-0101 WhatsApp Provider Runtime Selection
- ADR-0102 Zoom Provider Runtime Boundary
- ADR-0104 Yandex Telemost Provider Runtime Boundary
- ADR-0105 Zulip Reference Provider and Макошь Lab
- ADR Architecture Communication Contract

## Context

Макошь has a single large backend crate. Module boundaries communicate intent,
but they do not keep provider SDKs, SQL stores, vault implementation and domain
code out of the same compiler unit. The result is broad rebuilds and runtime
composition that is difficult to isolate or test independently.

ADR-0099 correctly established Signal Hub, the canonical event backbone and
fixture-first provider testing. Its initial exclusion of provider runtime
sidecars is no longer appropriate once provider contracts are compiler-enforced.
This ADR preserves every Signal Hub and event-platform decision from ADR-0099
and replaces only the runtime topology decision.

## Decision

### Signal Hub and event platform

Signal Hub remains the owner of source registry, connections, capabilities,
runtime state, health, policies, profiles, replay requests and system recovery
fixtures. It does not own provider protocol code, secrets, raw private content
or any other business domain state.

PostgreSQL `event_log` remains the audit and recovery source of truth. NATS
JetStream provides durable delivery and fan-out, the in-memory EventBus remains
the deterministic unit-test transport, Axum SSE remains the browser realtime
path, and Protobuf plus ConnectRPC remain typed command/query contracts.

Every real source requires a deterministic fixture source. Recovery fixtures
remain schema-agnostic, idempotent and free of IDs, secret references and user
data. Redis, Kafka and RabbitMQ are not event substrates for Макошь.

### Cargo workspace boundaries

Макошь evolves through a layered Cargo workspace. The initial stable boundaries
are `makosh-kernel`, event/provider/vault/blob/observation API crates, provider
implementations, domain API and persistence crates, application/workflow
orchestration, and runtime composition crates. Observation API crates own only
stable observation models and validation; `makosh-observations-postgres` owns
observation transactions and PostgreSQL behavior as a persistence adapter.
`makosh-communications-api` owns provider-account and evidence ports;
`makosh-communications-postgres` owns their SQL implementations, including
the atomic raw-evidence/observation write and ingestion checkpoints. Credential
resolution remains outside that adapter, behind the vault boundary.
`makosh-signal-hub-api` owns provider-neutral raw-signal command and runtime
query contracts. Its PostgreSQL persistence adapter will be extracted by
responsibility (connections, profiles, policies, runtime state, replay and
health), rather than moving the existing store as a single unit.
HTTP composition may invoke an explicit public workflow API directly; this
replaces compatibility re-export facades and does not authorize HTTP handlers
to construct stores or provider runtime internals.

Dependencies point toward stable contracts. Domain APIs and application code do
not depend on provider implementations. Provider crates do not depend on
domains, app handlers, SQLx, canonical PostgreSQL, or vault implementation.
Only composition/runtime crates may wire provider implementations to core
implementations. Migration slices cut internal imports directly to their new
crate paths; compatibility re-exports and wildcard facades are forbidden.

`CommunicationProviderKind` remains the compatibility enum for current
storage and public contracts. An open `ProviderId` is introduced at the runtime
contract boundary; replacing legacy database constraints is outside this ADR.

### Provider topology

Every integration receives a provider crate. Its runtime topology is selected
per provider/account rather than by a universal process rule:

- `in_process` is the default for simple adapters and remains supported for all
  migrations;
- `shared_connector` is a separately supervised process for a provider with
  safely shared runtime state;
- `per_account_connector` is reserved for account-scoped native, FFI,
  crash-prone or session-heavy runtimes.

The selected topology is desired state in
`SignalConnection.settings.runtime_topology`; the active topology and health
remain in `SignalRuntimeState`. The first proof is Zulip: extract its provider
crate in-process, then validate an opt-in shared connector. Telegram user and
WhatsApp native runtimes remain in-process until their session, media and
fencing requirements are proven.

No runtime automatically falls back between topologies. A switch drains the old
runtime, increments its lease epoch and then starts the new runtime. Failure is
visible as Signal Hub health state.

### Runtime protocol and security

Provider command, result, observation and acknowledgement messages use
versioned provider envelopes. Commands include command and idempotency IDs,
provider/account identity, deadline, attempt, lease epoch, causation and
correlation. Observations include a stable observation ID, provider cursor,
spool sequence, provenance and observed/occurred timestamps.

The JetStream subjects are:

```text
makosh.provider.commands.v1.<provider>.<account>
makosh.provider.results.v1.<provider>.<account>
makosh.provider.observations.v1.<provider>.<account>.<kind>
makosh.provider.acks.v1.<provider>.<account>
```

Existing canonical `signal.*`, `integration.*` and `communication.*` event
types remain inside those envelopes. A connector persists outbound results and
inbound observations in an account-scoped bounded spool until core
acknowledgement. Provider calls are at-least-once at the transport boundary;
ambiguous calls to non-idempotent provider APIs are recorded as
`unknown_outcome` and are never retried automatically.

Connector control is a versioned ConnectRPC service over a mode-`0600` Unix
socket. It exposes `Hello`, `Describe`, `Start`, `Stop`, `Drain`, `Health`,
`BeginAuth`, `CompleteAuth`, `RenewCredentialLease` and
`RevokeCredentialLease`. In-process and RPC adapters implement the same
semantic runtime port.

The desktop runtime retains the vault master key. A connector never receives a
HostVault implementation or canonical PostgreSQL credentials. It receives a
short-lived credential lease scoped to provider, account, purpose and epoch
only through the authenticated control socket. Bootstrap authentication uses an
inherited pipe/file descriptor, not command arguments, environment variables
or logs. Lease material is not persisted by the connector and is zeroized at
expiry, revocation and shutdown. Media crosses the boundary through an opaque
blob reference, never an arbitrary local path or a large NATS payload.

## Migration and compatibility

Migration is incremental. First remove known architecture-guard exceptions and
make the workspace dependency graph executable. Then extract foundation
contracts and the Zulip provider crate while preserving the existing backend
binary, HTTP routes, ConnectRPC services, event types and persisted provider
strings. Domain ports for Communications and Signal Hub precede provider
orchestration and the optional Zulip connector.

The connector phase adds technical `provider_runtime_leases` storage and a
`lease_epoch` on provider commands so core can reject stale completions and
observations. Connectors own only provider-native session state and their
bounded spool; canonical evidence and business state remain in core.

The Tauri package continues to launch `makosh-backend` during migration.
It becomes the desktop composition runtime and may supervise optional connector
children after packaging and smoke validation succeed.

## Consequences

Positive:

- compiler-enforced dependency direction reduces accidental coupling;
- provider edits rebuild provider crates and thin composition roots rather than
  the whole backend where dependency graph permits;
- stateful providers can gain isolation without becoming product domains;
- durable evidence, replay, fixture testing and canonical domain ownership are
  preserved across process boundaries.

Negative:

- the workspace, protocol and packaging surface become more explicit;
- connector lifecycle, lease fencing and spool recovery require dedicated
  tests;
- clean builds may not improve until dependency seams are measured and tuned.

## Validation

The repository must enforce Cargo dependency roles in addition to source-path
architecture checks. Provider crates must have no SQLx, domain, backend or vault
implementation dependency; test session infrastructure must not depend on
production composition crates. No baseline or per-file architecture exception
may be introduced.

Every topology transition and connector must be verified with deterministic
fixtures/testcontainers for protocol mismatch, duplicate delivery, crash and
restart windows, stale epochs, NATS outage, spool replay, drain, secret lease
expiry/revocation, blob capability expiry and privacy-safe diagnostics. Live
provider actions remain explicit manual smoke work.
