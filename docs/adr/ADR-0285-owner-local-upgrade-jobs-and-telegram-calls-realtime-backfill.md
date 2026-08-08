# ADR-0285: Owner-local upgrade jobs и Telegram Calls realtime backfill

Статус: Принято
Дата: 2026-07-26
Состояние реализации: реализовано. `makosh-scheduler-protocol` предоставляет
отдельные `OwnerJobCommandV1`,
`OwnerJobTriggerKindV1::UpgradeReconciliation`, owner-local lease/scope builder,
exact descriptor set и отрицательную scheduled/upgrade interchange
conformance. Telegram V6 storage bundle содержит только additive DDL для
execution/checkpoint и replay-order projections. Telegram-owned executor
принимает exact durable command, bounded-batches старые frames, сохраняет
checkpoint, fences runtime generation/lease и завершает backfill до readiness.
Disposable PostgreSQL conformance подтверждает 257-frame batching,
crash/restart resume, newer-generation takeover, stale-lease rejection,
duplicate command idempotency и cursor-preserving mixed V3/V4 replay. Managed
signed Telegram conformance подтверждает backfill legacy frame до provider
polling/client delivery и terminal replay после restart.

Уточняет:

- [ADR-0214: Durable Job Platform](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0224: Storage Control](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0268: Telegram release assembly](ADR-0268-telegram-release-assembly-unit-and-signed-distribution-fragment.md);
- [ADR-0284: Telegram one-to-one audio calls](ADR-0284-telegram-one-to-one-audio-calls-operational-boundary.md).

## Контекст

Telegram Calls storage revision 3 уже содержала
`telegram_call_realtime_frames`. Revision 4 ввела единый
`telegram_call_realtime_events`, в котором call и operation events получают
один replay cursor. Additive Storage migration обязана содержать только DDL и
поэтому не может копировать существующие owner rows.

Без backfill старые call frames доступны через history tables, но отсутствуют
в новом realtime replay. Открытие Calls Command при таком состоянии создало бы
частично мигрированный operational surface.

ADR-0214 разделяет Job Platform на Scheduler и owner-local Job Executor.
Scheduler нужен для time policy. Этот backfill запускается ровно один раз из
факта принятого owner storage upgrade и не имеет расписания, пользовательской
time policy или cross-module target. Создание фиктивного cron/one-shot schedule
смешало бы migration lifecycle с Scheduler configuration.

Текущий `makosh-scheduler-protocol` имеет только
`ScheduledJobCommandV1`. Использовать его без `schedule_id` или подделывать
schedule запрещено. Нужен отдельный owner-neutral command для owner-local
upgrade reconciliation.

## Решение

### Owner-local upgrade command

`makosh-scheduler-protocol` получает `OwnerJobCommandV1` с отдельным
`OwnerJobTriggerKindV1::UpgradeReconciliation`. Он содержит только:

- stable 16-byte `job_run_id`;
- versioned `JobKindV1`;
- bounded opaque `scope_id`;
- exact trigger kind;
- positive owner-local acceptance timestamp;
- bounded execution lease с `run_id`, epoch и expiry.

Команда валидируется отдельно от `ScheduledJobCommandV1`. У неё нет
`schedule_id`, `schedule_revision`, retry/misfire/overlap policy или Scheduler
receipt. Она не может быть опубликована Scheduler runtime.

Owner runtime оборачивает payload в обычный exact
`DurableEnvelopeV1.command` с `target_capability = job_execute`, current runtime
source fence и deterministic idempotency key. Producer и consumer находятся в
одном owner runtime, поэтому cross-process NATS hop не создаётся: exact
envelope принимается owner-local inbox/job persistence в одной transaction.
Это допустимый `local_call` внутри одной integration unit, а не обход
межмодульного event spine.

JobKind для этого slice:

```text
owner: telegram
name: calls_realtime_backfill
major: 1
scope: owner
trigger: upgrade_reconciliation
```

Он не объявляется как `SchedulerJobRequestV1`, не получает default schedule и
не добавляется в Kernel Scheduler catalog.

### Storage и execution

Telegram storage bundle revision 6 добавляет только DDL для owner-local
execution/checkpoint table, replay-order mapping и replay cursor allocator.
Storage migration не содержит `INSERT`, `UPDATE`, `DELETE` или copy DML.

После успешного Storage admission Telegram runtime:

1. создаёт или повторно читает одно deterministic execution через exact
   command envelope;
2. atomically claims execution current runtime generation и повышает lease
   epoch;
3. materializes отсутствующий внешний replay order для уже существующих V4
   events bounded batches;
4. копирует старые frames bounded batches в порядке legacy
   `frame_sequence`;
5. вставляет call references в unified realtime events idempotently по
   `(call_session_id, call_revision)`;
6. в той же transaction двигает durable source checkpoint и проверяет current
   runtime generation/lease epoch;
7. помечает execution `succeeded`, когда за checkpoint больше нет source rows,
   и открывает allocator для следующих provider/operation events.

Revision 3 не имела local mute commands, поэтому backfilled call events получают
`local_muted = false`. Call payload по-прежнему восстанавливается join-ом с
owner-local append-only call state history; private media material не
копируется.

`event_sequence` остаётся внутренней identity unified projection и никогда не
переписывается. Для внешнего replay используется отдельный
`telegram_call_realtime_replay_order`. Если до V6 клиент уже получил cursor
`E`, executor выбирает offset строго выше `E + source_count`, помещает
отсутствующие V3 frames в `(E, offset)`, а уже существующие V4 events отображает
после offset с сохранением их внутреннего порядка. Поэтому любой прежний cursor
`<= E` получает сначала backfill, затем безопасный повтор уже виденных V4
events, но не silent gap. После terminal success один owner-local cursor row
атомарно выдаёт sequence новым call/operation events.

Provider polling и Calls client delivery не начинаются до terminal success
backfill. Поэтому новые realtime events не могут обогнать старые frames, а
клиент не наблюдает частичный replay.

### Restart, fencing и failures

- Crash до commit не двигает checkpoint и не создаёт частичную batch.
- Crash после commit возобновляет работу после persisted checkpoint.
- Новый runtime generation обязан claim-ить новый lease epoch.
- Stale runtime не может checkpoint или завершить execution.
- Duplicate command возвращает существующее execution и не создаёт второй
  backfill.
- Database unavailability не меняет execution state; managed runtime может
  быть безопасно перезапущен.
- Invalid persisted execution/envelope, incompatible JobKind или exhausted
  explicit execution policy fail closed до provider/client processing.
- Terminal `succeeded` не переоткрывается при обычном module restart.

## Build units и SRP

- owner-neutral wire types и validation остаются в
  `makosh-scheduler-protocol`;
- Telegram-specific job identity и policy остаются в
  `makosh-telegram-calls-core`;
- execution, lease, checkpoint и owner SQL остаются в
  `makosh-telegram-calls-persistence`;
- runtime только создаёт exact command, запускает bounded executor и
  координирует admission order;
- Telegram assembly только включает новую DDL migration в exact bundle.

Новый generic worker package, Scheduler dependency на Telegram, Kernel
business handler, cross-owner SQL и direct DML из Storage migration запрещены.
SRP определяется этими причинами изменения, а не размером файлов.

## Admission evidence

Backfill gate закрыт следующими evidence:

1. protocol tests для exact upgrade command и negative scheduled/upgrade
   interchange;
2. core tests для stable JobKind, command identity и bounded policy;
3. disposable PostgreSQL conformance для duplicate acceptance, bounded
   checkpoint, crash/restart resume, stale lease и terminal replay;
4. migration guard, доказывающий DDL-only bundle;
5. managed Telegram conformance, где legacy revision-3 frames существуют до
   runtime launch, а после owner job доступны через unified realtime replay;
6. architecture tests, запрещающие Scheduler/Kernel/Communications dependency
   на Telegram job implementation.

Backfill prerequisite ADR-0284 выполнен. Последующий Calls Command signaling
gate также закрыт отдельно; real media gate остаётся открытым.

## Отклонённые варианты

### DML внутри Storage migration

Отклонено: Storage Control проверяет schema transition, но не владеет
integration data reconciliation.

### Fake Scheduler schedule

Отклонено: backfill не имеет time policy, а synthetic schedule создаёт ложную
конфигурацию и лишний cross-process lifecycle.

### Startup SQL без job execution

Отклонено: нет durable acceptance, checkpoint, lease fence и доказуемого
restart behavior.

### Общий migration worker

Отклонено: он агрегирует handlers разных owners, нарушает compile/failure
isolation и превращается в скрытый второй runtime.
