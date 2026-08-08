# ADR-0250: Communications admission profile for `nats_data_plane_v1`

Статус: Принято
Дата: 2026-07-23
Состояние реализации: platform gate `nats_data_plane_v1` уже открыт ADR-0201 и
текущей executable policy. Communications ingress/outbox/consumer foundation
существует; production owner transaction и live replay остаются evidence
`first_owner_v1`.

Зависит от:

- [ADR-0201: IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0209: Kernel Event Hub](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0220: DurableEnvelopeV1](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0249: Communications Storage profile](ADR-0249-communications-storage-control-v1-admission-profile.md).

## Решение

Все cross-owner сообщения Communications проходят только через event spine:

```text
integration-owned transaction
  -> integration outbox exact DurableEnvelopeV1
  -> NATS JetStream
  -> Communications inbox
  -> Communications domain decision
  -> Communications owner mutation + outbox
  -> ACK after commit
```

Integration может зависеть только от
`makosh-communications-ingress`. Она не зависит от
`makosh-communications-api`, domain, persistence или runtime. Communications
не импортирует integration contracts, runtimes, SDK или persistence.

Допустимые входы:

- versioned provider-neutral observation;
- hashed source/account/conversation/participant/media identities;
- typed provenance, evidence kind, direction and timestamps;
- typed Blob receipt/failure and bounded attachment descriptor.

Недопустимы generic map/JSON payload, provider cursor, credential, session
material, direct module socket и request/reply вызов integration runtime.

Event Hub управляет catalog, schema digest, subject ACL, stream/consumer
budgets и topology. Он не декодирует business payload. Relay публикует exact
outbox bytes без re-encode.

## Последовательность реализации

1. Зафиксировать ingress и canonical event contract revisions/schema hashes.
2. Зарегистрировать exact publish/subscribe routes и budgets.
3. Доказать credential issue, rotation, revoke и stale fence rejection.
4. Запустить integration outbox → JetStream → Communications inbox.
5. Доказать duplicate/replay, NATS outage и ACK-after-commit.
6. Запустить canonical Communications outbox для downstream owners.

## Evidence для owner admission

- live NATS authority/Vault/runtime credential contour;
- exact-byte integration relay для каждого admitted integration owner;
- owner-local PostgreSQL inbox/outbox transaction;
- duplicate message ID/hash conflict rejection;
- redelivery after crash and pending outbox after broker outage;
- revoked grant/runtime generation cannot publish, consume or ACK;
- no private content in subjects, logs, errors or health.

## Rollback

Owner routes удаляются из desired Event Hub topology, credentials revokes,
runtime останавливается. Pending inbox/outbox остаются owner-local и могут быть
replayed после повторного admission. Platform `nats_data_plane_v1` не
закрывается.

