# ADR-0184: Backend Clean-Room Restart

Status: Accepted
Date: 2026-07-15

Supersedes:

- the concrete workspace and migration roadmap in ADR-0181;
- ADR-0183 Backend API Cutover and Canonical Schema Reset.

Preserves as design inputs:

- canonical evidence and provenance;
- the canonical event envelope and replay semantics;
- Signal Hub as provider-neutral runtime policy;
- Host Vault ownership of credentials and provider sessions;
- fixture-first provider validation;
- explicit runtime lifecycle and provider isolation.

## Context

The incremental extraction produced many crates while the original composition,
storage ownership and frontend contract still controlled the system. This made
the workspace look modular before its reasons to change were actually isolated.
It also preserved unverified provider surfaces and repeatedly forced new code to
accommodate legacy APIs, migrations and test composition.

The owner has chosen a clean-room restart. Local PostgreSQL data may be rebuilt;
vault credentials and provider session state must not be deleted automatically.

## Decision

The complete previous Rust workspace is archived under
`references/backend-legacy/`. It is excluded from production builds and may
never be a dependency of the new backend.

The new backend starts without a Cargo package. Before implementation, Макошь
will establish an evidence-backed inventory of supported product capabilities,
contracts, domain ownership, runtime lifecycle and fresh-schema requirements.
Legacy code may be consulted only to recover a verified invariant or observable
behavior. Every recovered behavior enters the new system through a new contract
and focused test; whole legacy modules are not moved into production.

Mail, Telegram and Zulip are the only providers currently considered working.
Other provider surfaces are unsupported until an isolated executable fixture or
runtime test proves them. Unsupported code does not receive a production route,
navigation entry, scheduler or background task merely because it existed in the
legacy tree.

The first implementation milestone will be a thin vertical walking skeleton,
chosen only after the clean-room capability and ownership inventory is agreed.
It must include its production contract, application use case, persistence
boundary, runtime composition, frontend consumer and external integration test
without compatibility facades.

## Guardrails

- Domain boundaries follow responsibility and reason to change, not file size.
- Router construction has no side effects and task factories do not spawn.
- Tests live outside production modules except private pure invariants.
- Test support cannot depend on production composition.
- Provider code cannot own SQL, domain state or vault implementation.
- Unverified providers fail closed and remain absent from product surfaces.
- New packages are created by demonstrated ownership, not by roadmap symmetry.
- Legacy database migrations are reference material; the new schema is designed
  from current owners and boots from an empty database.
- Frontend and backend contracts change atomically because the frontend is the
  only product API consumer.

## Consequences

The repository is intentionally not backend-buildable between the archive step
and the first approved walking skeleton. Existing backend Make targets and
architecture scripts are legacy evidence until they are replaced. Frontend code
continues to exist but cannot be treated as proof that its old backend surface
is supported.

Rollback is restoring the archived workspace as the production workspace. No
database down migration is required; development data is disposable, while
vault and provider session state remain untouched.

