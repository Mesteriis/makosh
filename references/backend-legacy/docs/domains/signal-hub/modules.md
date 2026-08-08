# Signal Hub Modules

Status: target module layout.

Signal Hub remains a modular bounded context. ADR-0181 permits optional provider
connectors only behind the same event, policy and runtime-state contracts; it
does not make sidecars a Signal Hub ownership concern.

## Backend Target Layout

```text
backend/src/domains/signal_hub/
├── mod.rs
├── api/
│   ├── commands.rs
│   ├── queries.rs
│   └── dto.rs
├── core/
│   ├── entities.rs
│   ├── errors.rs
│   ├── policies.rs
│   ├── capabilities.rs
│   ├── health.rs
│   ├── profiles.rs
│   └── validation.rs
├── store/
│   ├── mod.rs
│   ├── sources.rs
│   ├── connections.rs
│   ├── capabilities.rs
│   ├── runtime.rs
│   ├── health.rs
│   ├── policies.rs
│   └── replay.rs
├── runtime/
│   ├── registry.rs
│   ├── controller.rs
│   ├── source.rs
│   └── fixture_source.rs
├── fixtures/
│   ├── loader.rs
│   ├── recovery.rs
│   ├── schema.rs
│   └── system.toml
└── projections/
    ├── dashboard.rs
    ├── source_list.rs
    └── health.rs
```

## Platform Event Target Layout

```text
backend/src/platform/events/
├── envelope.rs / models.rs
├── bus.rs
├── transport.rs
├── nats.rs
├── publisher.rs
├── dispatcher.rs
├── consumers.rs
├── cursors.rs
├── store.rs
├── replay.rs
└── validation.rs
```

Current repository files under `platform/events` should evolve instead of being
bypassed. New code must not create a second event subsystem.

## Contracts Target Layout

```text
contracts/
├── proto/
│   └── makosh/
│       ├── signal_hub/v1/signal_hub.proto
│       ├── events/v1/event_envelope.proto
│       ├── communications/v1/communications.proto
│       └── common/v1/ids.proto
└── README.md
```

If contracts remain inside backend at first, keep the same logical layout under:

```text
backend/contracts/proto/
```

## Frontend Target Layout

```text
frontend/src/domains/settings/
├── api/
│   └── signalHub.ts
├── queries/
│   └── useSignalHubQuery.ts
├── components/
│   └── SignalHubSettings.vue
├── views/
│   └── SettingsPage.vue
└── lib/
    └── signalHubReplay.ts
```

Current implementation keeps Signal Hub under Settings rather than a standalone
top-level frontend domain. Provider setup/runtime details can stay under
`frontend/src/integrations/*`, but the Settings-owned Signal Hub UI owns the
cross-source overview and source control commands.

## Required Traits / Ports

```rust
trait SignalSource {
    async fn start(&self) -> Result<(), SignalSourceError>;
    async fn stop(&self) -> Result<(), SignalSourceError>;
    async fn pause(&self) -> Result<(), SignalSourceError>;
    async fn resume(&self) -> Result<(), SignalSourceError>;
    async fn health_check(&self) -> Result<SignalHealthSnapshot, SignalSourceError>;
}

trait SignalPublisher {
    async fn publish_signal(&self, signal: NewEventEnvelope) -> Result<(), EventError>;
}

trait SignalFixtureSource {
    async fn emit_fixture(&self, fixture_id: &str) -> Result<(), SignalFixtureError>;
}
```

These are conceptual contracts. Exact Rust shape can change, but all provider
sources must be replaceable by deterministic fixture sources.

## Module Boundaries

Allowed:

- Signal Hub imports `platform/events`, `platform/settings`, `platform/audit`,
  `platform/secrets` resolver abstractions and its own modules.
- Signal Hub emits events for Communications, Radar, Calendar, Documents and
  other owners to consume.
- Integration adapters ask Signal Hub for source policy/runtime decisions through
  public ports.

Forbidden:

- Signal Hub directly writes `communication_*`, `task_*`, `document_*`,
  `calendar_*`, `persona_*` or `radar_*` tables.
- Signal Hub imports provider-specific stores from `backend/src/integrations/*`.
- Integration adapters import business domain stores or services.
- Signal Hub stores secret values or raw private message bodies.

## Code Size Rule

Signal Hub must follow existing Макошь anti-god-file rules:

- files over 700 lines require written justification;
- files over 1000 lines are architectural problems;
- avoid `manager.rs`, `service.rs`, `helper.rs` and `utils.rs` as large dumping
  grounds.

Preferred names are ownership-specific: `source_registry.rs`, `policy_store.rs`,
`fixture_loader.rs`, `runtime_controller.rs`, `health_projection.rs`.
