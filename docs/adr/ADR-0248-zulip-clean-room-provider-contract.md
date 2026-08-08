# ADR-0248: Zulip clean-room provider contract

Status: accepted; implementation in progress.

## Decision

Zulip is an integration owner, not a Communications domain, Gateway facade, or
generic provider implementation. Its clean-room package graph is exact:

```text
makosh-zulip-api
makosh-zulip-core
makosh-zulip-http
makosh-zulip-persistence
makosh-zulip-runtime
```

`api` owns only typed Zulip operational contracts. `core` owns the Zulip
anti-corruption mapper and validation. `http` is the only Zulip REST protocol
adapter. `persistence` owns Zulip queue cursor, command/reconciliation state,
operational projections, and exact-byte Communications outbox. `runtime`
coordinates the other public owner contracts and has no business-domain
dependency.

## Required provider semantics

- stream messages require distinct `stream` and `topic` fields;
- direct messages preserve an ordered non-empty recipient set and distinguish
  all-numeric Zulip user IDs from email recipient references;
- uploads are provider operations; upload bytes are materialized only through
  the Blob owner, then uploaded before an upload-backed message command;
- update, delete, add reaction, and remove reaction remain distinct commands;
- an uploaded/downloaded URL must be same-realm and under `/user_uploads`;
- event queue ID and last event ID remain Zulip-private durable cursor state;
- API key and provider content must not enter logs, diagnostics, event subjects,
  or Communications persistence.

## Communications boundary

The only Zulip to Communications dependency is
`makosh-communications-ingress`. Zulip publishes typed, metadata-only neutral
observations through its own exact-byte outbox. Communications never imports a
Zulip DTO, HTTP adapter, cursor, or persistence crate. No command is completed
solely by HTTP acceptance when its provider observation/reconciliation contract
requires later confirmation.

## Consequences

The former `/api/v1/integrations/zulip/*` REST facade and legacy generic
provider/connector contracts are not migration targets. Zulip frontend work is
secondary until generated owner-specific Gateway contracts exist.
