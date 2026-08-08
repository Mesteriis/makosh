# ADR-0344: Delayed-delivery execution store adapter

Статус: Принято

Дата: 2026-07-30

Состояние реализации: реализовано. Отдельная build unit
`makosh-communication-delayed-delivery-store-adapters` реализует
`ExecutionStorePortV1` поверх owner-local delayed-delivery persistence.
Adapter сохраняет exact inbox/outbox и execution-fence semantics persistence,
имеет owner-local `persistence` surface, не содержит SQL и не расширяет phase
gate `communication_delayed_delivery_v1`.

Уточняет:

- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0224](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0341](ADR-0341-scheduled-communication-delivery-workflow.md).

## Контекст

ADR-0341 разделил delayed-delivery lifecycle на persistence и execution units.
Persistence уже атомарно:

- дедуплицирует Scheduler due command по inbox ID/hash;
- переводит operation в fenced execution;
- сохраняет acceptance receipt в outbox;
- завершает accepted/failed transition вместе с terminal receipt.

Execution владеет orchestration и объявляет `ExecutionStorePortV1`, но не может
импортировать SQL implementation. Persistence также не должен зависеть от
execution orchestration только ради реализации внешнего порта. Transport
runtime-adapters владеет Blob и `request_rpc`; добавление туда PostgreSQL
composition смешало бы две независимые причины изменения.

## Решение

Добавляется девятая owner-local build unit:

```text
makosh-communication-delayed-delivery-store-adapters
```

Она:

- зависит только от
  `makosh-communication-delayed-delivery-execution` и
  `makosh-communication-delayed-delivery-persistence`;
- реализует `ExecutionStorePortV1`;
- выполняет явное типизированное отображение port models в persistence models
  и обратно;
- сохраняет различие `Claimed` и `Duplicate`;
- fail-closed отображает corrupt persistence row в `Unavailable`, stale
  revision в `Conflict`, а claim loss и not-found оставляет отдельными;
- не содержит SQL, transport I/O, Scheduler protocol decode, Blob access,
  delivery-intent routing или lifecycle policy.

Persistence остаётся единственным владельцем workflow SQL и транзакций.
Execution остаётся единственным владельцем due orchestration. Будущий runtime
composition root создаёт store adapter из уже проверенного owner-local
persistence handle.

## Границы

Store adapter не импортирует:

- Kernel или Gateway implementation;
- Scheduler implementation/persistence;
- Communications domain implementation;
- Mail, Telegram, WhatsApp или Zulip integration;
- NATS, Blob или provider contracts.

Это не repository facade и не generic persistence abstraction. Adapter
существует только для одного exact delayed-delivery execution port.

## Последствия

- SQL и orchestration остаются compile-isolated.
- Ошибки storage boundary становятся typed и не выдают raw database details.
- Runtime assembly получает явную composition seam без дублирования
  persistence transitions.
- Phase gate остаётся `planned` до managed runtime, assembly, client/realtime и
  live end-to-end evidence из ADR-0341.

## Отклонённые варианты

### Реализовать port внутри persistence crate

Создаёт зависимость persistence от execution orchestration и меняет направление
слоёв ради удобства wiring.

### Добавить SQL в runtime-adapters

Смешивает owner-local storage adapter с Kernel-routed Blob/request transport и
нарушает функциональный SRP.

### Повторить transitions в runtime

Создаёт второй lifecycle authority и разрушает атомарность inbox, state и
outbox.
