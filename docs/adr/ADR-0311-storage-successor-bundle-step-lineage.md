# ADR-0311: Storage successor bundle step lineage

Статус: Принято
Дата: 2026-07-28
Состояние реализации: Implemented. PostgreSQL migration executor распознаёт
exact, predecessor и missing step lineage, не повторяет DDL принятого
предшественника, сохраняет immutable acceptance row для текущей bundle revision
и fail closed отклоняет digest drift и downgrade. Pure Rust regression tests,
live disposable PostgreSQL successor test и static architecture gate зелёные.
Существующая development database прошла Mail V14 → V15 в `make dev`
generation `20` без удаления volume или owner data.

Уточняет:

- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0306: repeatable development release refresh](ADR-0306-repeatable-development-release-refresh-and-successor-fencing.md).

## Контекст

Owner persistence bundles в Макошь являются cumulative: bundle revision `N`
содержит immutable steps прежних revisions и новый forward step. PostgreSQL
ledger V1 сохраняет:

```text
(owner_id, bundle_revision, step_revision, step_digest)
```

Прежний executor искал step только внутри текущей `bundle_revision`. Поэтому
переход Mail V14 → V15 не видел уже выполненные steps V1–V14 и пытался повторно
выполнить их DDL. Fresh disposable database проходила, а сохранённая
development database fail closed останавливалась на первом non-idempotent
statement.

Повторный DDL не является допустимым reconciliation. Замена его на
`IF NOT EXISTS`, silent error suppression или data reset скрыла бы digest drift
и разрушила бы authority canonical ledger.

## Решение

Для текущего Storage V1 один owner имеет один canonical bundle lineage.
`step_revision` и exact `step_digest` являются immutable identity step во всех
successor bundle revisions этого owner.

Перед выполнением каждого admitted step executor в той же PostgreSQL
transaction читает все ledger rows:

```text
owner_id = current owner
step_revision = current step revision
```

Результат классифицируется так:

1. `exact`: row текущей bundle revision существует и digest совпадает. Executor
   commit-ит no-op transaction.
2. `predecessor`: существуют только меньшие bundle revisions и все digests
   точно совпадают. Executor не выполняет DDL, а добавляет immutable acceptance
   row текущей bundle revision с тем же digest.
3. `missing`: rows отсутствуют. Executor выполняет DDL под exact owner DDL role
   и в той же transaction добавляет ledger row.
4. `digest drift`: любой row для этого owner/step содержит другой digest.
   Bundle отклоняется до DDL.
5. `downgrade`: существует row с bundle revision больше текущей. Bundle
   отклоняется до DDL даже при совпадающем digest.

Acceptance row predecessor не утверждает повторное выполнение migration. Он
фиксирует, что текущий exact cumulative bundle наследует уже применённый step.
После начала successor application прежний bundle не может снова стать active:
первый inherited row уже создаёт monotonic authority fence.

Partial successor остаётся `blocked_migration`. Повтор exact successor
продолжает с ledger boundary; automatic rollback, previous bundle fallback и
data reset запрещены.

## Units of assembly

```text
makosh-storage-protocol    immutable StorageBundleV1 transport, unchanged
makosh-storage-migrations AST admission before execution, unchanged
makosh-storage-postgres   lineage classification, DDL and ledger transaction
makosh-storage-runtime    sanitized failure mapping, unchanged
owner persistence         cumulative exact steps and digests
Kernel Control Store      exact bundle bytes/digest admission, no SQL lineage
```

Storage lineage остаётся platform responsibility. Mail, Telegram, WhatsApp,
Zulip, Communications и другие owners не реализуют собственный migration
runner и не получают доступ к `makosh_platform.storage_migration_ledger`.

## Gate `storage_successor_step_lineage_v1`

Gate закрывается только при наличии:

1. exact/predecessor/missing classifier;
2. predecessor acceptance без DDL replay;
3. digest drift rejection до DDL;
4. future bundle revision rejection;
5. transactionally recorded successor acceptance;
6. pure unit tests всех classifier states;
7. live PostgreSQL V1 → V2 successor test с non-idempotent predecessor DDL;
8. повторного `make dev` на существующей database без удаления volume или
   owner data.

## Отклонённые варианты

### Сделать все старые DDL statements idempotent

Отклонено. Это не доказывает immutable lineage и может скрыть изменённый SQL
под прежним step revision.

### Игнорировать duplicate-object errors

Отклонено. Error class не доказывает, что существующий object создан exact
admitted migration bytes.

### Удалить development database

Отклонено. Clean bootstrap не является upgrade evidence и уничтожает
пользовательское provider state.

### Хранить applied state только в Kernel

Отклонено. Canonical schema state остаётся в PostgreSQL Storage ledger по
ADR-0224; Kernel не становится SQL migration authority.
