# ADR-0399: Communications bounded observation backlog drain

- Status: Accepted
- Implementation status: Implemented
- Date: 2026-08-04

## Context

Communications owns fourteen exact Event Hub consumers. The runtime previously
rotated after every single delivery. An empty pull is deadline-bounded, so a
large `communication_observed.v1` backlog advanced by only one item after
probing thirteen unrelated idle consumers. A Mail repair sync therefore
published valid evidence much faster than Communications could canonicalize
it, leaving the client on stale body state for many minutes.

## Decision

The owner-local consumer scheduler may drain at most 512 consecutive
`communication_observed.v1` deliveries before rotating through the remaining
exact consumers. If the observation consumer is empty or unavailable, it
rotates immediately. A rotation probes exactly one secondary consumer and then
returns to the observation consumer; the secondary cursor advances fairly
across bursts and empty observation probes.

This is scheduling only. Subjects, contracts, grants, durable envelopes,
inbox/hash checks, transactions and acknowledgements remain unchanged. The
runtime still processes one authorized delivery at a time and gives every
other consumer a bounded fairness point after each observation burst.

Body custody transfer is maintenance work after observation admission, not an
Event Hub consumer. One maintenance tick may complete at most 64 custody
transfers, stopping immediately when the queue is empty. The separate derived
index bound remains unchanged. This prevents a historical PendingBlob queue
from delaying newly repaired bodies by hundreds of maintenance intervals while
keeping every maintenance slice exact and bounded.

An already-due maintenance deadline is checked before starting the next durable
consumer pull. A continuously non-empty Event Hub backlog therefore cannot
starve custody/index maintenance; client control delivery keeps its existing
first priority, and no concurrent mutation path is introduced.

Custody claims are ordered by canonical evidence `observed_at` newest-first,
then by evidence ID for determinism. This prevents an expired historical source
lease from head-of-line blocking a newly admitted body; older work remains
durable and eligible in the same queue.

## Consequences

- Provider backfills and repair syncs no longer multiply their latency by all
  idle consumer pull deadlines.
- Other contracts cannot starve: the burst is exact and bounded, while each
  secondary contract receives its turn through a persistent round-robin cursor.
- Observation latency is bounded by at most one secondary pull deadline rather
  than the sum of every unrelated consumer deadline.
- A future adaptive scheduler or multi-delivery pull contract requires a
  separate decision and conformance evidence.

## Validation

- Scheduler tests prove the 512-delivery fairness boundary and immediate
  rotation for an empty observation consumer.
- Runtime validation proves the custody loop retains its exact 64-item bound
  and stops on the first empty claim.
- Managed/browser validation must show the repaired Mail body reaching
  canonical Communications content and the selected reader.
