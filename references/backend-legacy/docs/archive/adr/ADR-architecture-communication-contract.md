# ADR Architecture Communication Contract

Status: Accepted
Date: 2026-06-20

Supersedes:

- ADR-0073 Backend Module Organization, for direct Graph imports and broad
  layer wording that allowed cross-domain shortcuts.
- ADR-0095 Event-Driven Domain Communication and DLQ, for the temporary
  architecture boundary baseline.

Clarifies:

- ADR-0014 Canonical Event Envelope
- ADR-0035 Local Event API Command Boundary
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0097 Communications Channel Domains To Integrations

## Context

Макошь has enough domain and provider surface that "do not import another
domain" is no longer precise enough. The architecture needs one communication
contract that applies to backend modules, frontend modules, events, projections,
provider runtimes and AI outputs.

The old boundary baseline made this ambiguous by allowing exact legacy
violations to stay green. That baseline is now removed. Architecture violations
must be fixed, not registered as exceptions.

## Decision

Макошь uses exactly these component interaction kinds:

```text
direct_call
command_port
query_port
event
projection
runtime_integration_api
```

All component boundaries are described in
`scripts/architecture-contract.json`. That JSON file is executable policy and is
validated by `make architecture-check`.

Backend rules:

- `app/` owns route composition, HTTP handlers, app state and top-level errors.
  It may call domain command/query ports, integration runtime/setup APIs and
  explicit public workflow APIs. It must not import concrete domain stores or
  provider runtime internals.
  It must not own business orchestration or durable stores.
- `domains/*` own one bounded context. A domain may import its own modules,
  `platform/*`, and pure/domain-neutral engines. It must not import other
  domains, integrations, app handlers or workflows for business behavior.
- `integrations/*` own external provider protocol, setup and runtime state.
  They may import platform, vault and external SDKs. They must not import
  business domains or mutate business truth directly.
- `workflows/*` coordinate multiple domains through command/query ports and
  events. They must be idempotent and carry causation/correlation metadata.
  They must not own HTTP handlers, domain stores or integration clients.
- `engines/*` are pure or domain-neutral. They may own their own projections and
  indexes. They must not mutate business domains or import integrations.
- `ai/*` produces candidates, summaries, classifications and embeddings. It is
  not a source of truth and must not mutate domains directly.
- `platform/*` is importable by all layers and must not import domains,
  integrations or workflows.
- `vault/*` owns secrets, sessions and runtime credential state only.

Frontend rules:

- `frontend/src/app` composes routes, multiple domain views and multiple domain
  stores.
- `frontend/src/domains/*` must not import other frontend domains.
- `frontend/src/integrations/*` is provider setup/runtime UI only.
- Provider business query/cache roots `['telegram', ...]`,
  `['whatsapp', ...]` and `['mail', ...]` are forbidden. Business data uses
  `['communications', ...]`. Provider runtime state may use
  `['integrations', provider, 'runtime', ...]`.

## Consequences

Positive:

- The repository has one canonical interaction vocabulary.
- Baseline files and per-file compatibility exceptions stop hiding coupling.
- Communication channels can be provider integrations without owning product
  domains.
- Frontend cache ownership follows product boundaries.

Negative:

- Existing synchronous projection paths must move to workflows/events or
  command/query ports.
- Some historical provider/domain DTOs need to move to platform-owned contract
  types.
- Cross-domain behaviors become more explicit and sometimes eventually
  consistent.

## Validation

`make architecture-check` must run:

```text
node scripts/check-architecture-contract.test.mjs
node scripts/check-architecture.mjs --self-test
node scripts/check-architecture.mjs
```

The architecture guard must fail if:

- `scripts/architecture-boundary-baseline.json` exists;
- the interaction kind vocabulary changes without updating the contract;
- backend domains import other backend domains;
- backend integrations import business domains;
- frontend domains import other frontend domains;
- provider business cache keys use provider-root roots.
