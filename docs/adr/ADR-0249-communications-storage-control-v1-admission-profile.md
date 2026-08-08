# ADR-0249: Communications admission profile for `storage_control_v1`

Статус: Принято
Дата: 2026-07-23
Состояние реализации: platform gate `storage_control_v1` уже открыт ADR-0224 и
текущей executable policy. Этот ADR не открывает `first_owner_v1`; он фиксирует
точный Storage profile, который Communications обязан пройти перед production
admission.

Зависит от:

- [ADR-0223: Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: phase gates](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md);
- [ADR-0240: Communications clean-room migration](ADR-0240-canonical-communications-owner-clean-room-migration.md).

## Контекст

Platform Storage уже умеет выдавать fenced owner-local binding, запускать
PostgreSQL/PgBouncer lifecycle и маршрутизировать Vault ciphertext. Для
Communications всё ещё требуется exact owner profile. Без него общий факт
открытия platform gate не доказывает, что Communications не использует чужую
schema, роль или migration lifecycle.

## Решение

Communications использует только owner-local PostgreSQL через
`makosh-communications-persistence`. Его runtime композирует persistence, но
domain и public contracts не зависят от `sqlx`, Storage Control или Vault.

Exact owner graph:

```text
makosh-communications-api
        ↑
makosh-communications-domain
        ↑
makosh-communications-persistence
        ↑
makosh-communications-runtime
```

`makosh-communications-ingress` является event contract и не получает SQL
dependency. Integration packages не импортируют Communications persistence и
не получают его database binding.

Communications StorageBundle обязан задавать:

- отдельный database identity и runtime role для owner `communications`;
- additive AST-admitted migrations только для canonical evidence, inbox,
  outbox и owner projections;
- exact storage capability, registration, runtime generation, grant epoch,
  role epoch и credential revision;
- budgets для connections, statements, rows, transaction duration и storage;
- successor-only binding replacement и полный revoke lifecycle.

Запрещены:

- cross-owner SQL, shared business schema и direct PostgreSQL role reuse;
- выдача integration runtime роли Communications;
- SQL через Kernel, Gateway, Event Hub или Storage Control proxy;
- provider cursor, credential, session state или raw provider payload в
  Communications tables;
- readiness по одному TCP preflight без authenticated PostgreSQL/PgBouncer
  evidence.

## Последовательность реализации

1. Зафиксировать exact `StorageBundleV1` и digest для Communications.
2. Применить owner migrations через admitted DDL role.
3. Выдать fenced runtime binding через Storage Control и Vault lease.
4. Доказать transactional inbox + canonical mutation + owner outbox.
5. Доказать revoke, stale generation rejection и successor binding.
6. Добавить Communications database в whole-instance backup/restore matrix.

## Evidence для owner admission

- live disposable PostgreSQL/PgBouncer/Vault contour;
- catalog assertions для отсутствия cross-owner grants;
- transaction rollback и duplicate inbox conformance;
- credential rotation/revoke без plaintext в Kernel/logs;
- migration replay, incompatible digest rejection и recovery;
- backup/restore round trip для Communications schema.

## Rollback

До `first_owner_v1` rollback удаляет Communications binding из desired
topology и production inventory, останавливает runtime и оставляет platform
`storage_control_v1` открытым. Owner data не удаляется автоматически.

