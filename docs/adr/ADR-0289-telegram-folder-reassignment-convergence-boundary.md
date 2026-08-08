# ADR-0289: Telegram folder reassignment convergence boundary

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Реализовано в
`telegram_folder_reassignment_v1`: normalized target, fresh provider delta,
final exact verification, ambiguous partial-failure retry, zero-order
projection removal и managed restart conformance существуют. Slice остаётся
вне production owner inventory до Telegram umbrella admission; этот ADR сам по
себе gate не открывает.

Уточняет:

- [ADR-0240: Telegram clean-room provider boundary](ADR-0240-telegram-clean-room-provider-boundary.md);
- [ADR-0266: Telegram admission and event-only handoff](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md);
- [ADR-0282: full Communications reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Telegram provider model не предоставляет одну атомарную операцию «заменить
membership чата во всех folders». TDLib требует:

1. прочитать current chat positions;
2. добавить отсутствующие memberships;
3. отдельно изменить каждый folder, из которого chat должен быть удалён;
4. снова прочитать provider state.

Historical facade принимал один UI request, но создавал несколько queued
provider commands. Это не доказывало provider atomicity: ошибка или process
crash между командами оставляли частично применённый target.

Clean-room Telegram уже имеет один typed
`ReassignChatFolders` provider command внутри exact
`telegram.command.v1` route. Adapter вычисляет delta от fresh `getChat`, но
terminal completion пока следует из успешных intermediate responses без
обязательной финальной provider-проверки. Это недостаточно для отдельного
`telegram_folder_reassignment_v1` gate.

## Решение

Folder reassignment остаётся Telegram integration behavior:

```text
owner_id  = telegram
module_id = makosh-telegram-runtime
route     = /makosh.telegram.v1.TelegramOperationalService/ExecuteCommand
contract  = telegram.command.v1
command   = ReassignChatFolders
gate      = telegram_folder_reassignment_v1
```

Это не новый domain, workflow или runtime. Функциональные units остаются в
существующих integration-owned packages:

```text
makosh-telegram-api          bounded target-set command
makosh-telegram-core         operation/idempotency/retry policy
makosh-telegram-tdlib        provider delta and verification adapter
makosh-telegram-persistence  durable command and terminal state
makosh-telegram-runtime      execution/reconciliation orchestration
```

Release assembly остаётся downstream unit и не получает provider semantics.
Communications, Kernel и Gateway не импортируют folder contract или logic.

### Client contract

Request содержит:

- exact `operation_id`;
- exact `account_id`;
- exact `provider_chat_id`;
- `1..=64` unique positive `target_provider_folder_ids`.

Порядок client list не является semantic state. Runtime normalizes target IDs
в возрастающий unique set до provider execution. Duplicate, zero, negative,
empty или oversized target fail до durable mutation.

Command acceptance создаёт ровно один owner-local durable operation и один
typed accepted receipt. `accepted` не означает, что folders уже изменены.
Повтор с тем же operation/idempotency identity и другим target rejected как
collision; exact retry использует существующий record.

### Provider convergence

Каждая execution attempt:

1. получает fresh correlated `getChat`;
2. извлекает current positive folder set;
3. вычисляет sorted `target - current` и `current - target`;
4. выполняет adds, затем removals в deterministic order;
5. получает второй fresh correlated `getChat`;
6. сравнивает normalized provider set с exact target.

Adds идут первыми, чтобы chat не потерял все requested memberships в середине
attempt. Это не превращает последовательность в атомарную provider transaction.

Operation становится `completed` только если final provider snapshot exact
равен target set. Успешный `ok` отдельного add/edit без final equality является
недостаточным evidence.

### Partial failure, crash и retry

При provider error, timeout, malformed snapshot, final mismatch или process
crash operation остаётся retryable/failed согласно существующей Telegram
durable operation policy. Retry:

- не воспроизводит сохранённый stale delta;
- снова читает current provider state;
- вычисляет только оставшиеся steps;
- безопасно завершает ранее частично применённую mutation;
- сохраняет тот же operation identity и audit lineage.

Отдельный generic saga owner не вводится: одна Telegram provider operation
координирует только Telegram-owned state. Scheduler не подменяется fake job;
существующий owner worker claim/retry является execution authority.

Provider projection обновляется только из provider observations/snapshots.
Client intent не записывается как будто он уже provider truth.

### Fencing и privacy

Execution требует current:

- exact `telegram.command.v1` grant;
- managed runtime generation and grant epoch;
- Telegram account runtime lease;
- owner-local Storage and provider credential leases.

Kernel/Core передают opaque command и не декодируют folder IDs. Folder IDs,
chat IDs и target set не попадают в route metadata, health, subjects или
sanitized errors. Durable Telegram command payload остаётся owner-private.

Communications не получает folder state: это provider operational truth, а не
provider-neutral evidence.

## Gate `telegram_folder_reassignment_v1`

Gate становится `implemented` только при наличии:

1. bounded typed command validation and deterministic normalization;
2. one durable accepted operation with collision-safe idempotency;
3. fresh provider snapshot before mutation;
4. deterministic add/remove delta;
5. final provider snapshot and exact target equality;
6. partial-failure retry that converges without duplicate durable operation;
7. restart-safe operation/status and folder projection;
8. exact command grant/generation/lease fences;
9. package, Clippy, architecture and live managed conformance;
10. no REST alias, Communications facade or client-side fake completion.

Gate сам по себе не открывает `telegram_full_operational_v1` и не добавляет
Telegram в production inventory.

## Отклонённые варианты

### Обещать provider atomicity

Отклонено: TDLib выполняет несколько provider mutations; Макошь не может
сделать их одной Telegram transaction.

### Сохранить один раз вычисленный delta

Отклонено: после partial success или внешнего provider change stale plan может
повторить уже применённый step или удалить новое membership.

### Считать intermediate `ok` terminal result

Отклонено: acknowledgement отдельного mutation не доказывает итоговый exact
folder set.

### Реализовать это в Communications или generic workflow

Отклонено: операция читает и изменяет только Telegram operational truth и не
координирует несколько owners.

## Rollback

Revoke exact Telegram command capability останавливает новые attempts. Already
accepted operation сохраняет status/audit. Восстановление прежнего folder set
не выполняется автоматически: оно само является новым explicit provider
command с новым operation identity. Legacy REST fallback запрещён.
