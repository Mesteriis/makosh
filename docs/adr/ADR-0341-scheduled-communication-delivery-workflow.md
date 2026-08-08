# ADR-0341: Scheduled communication delivery workflow

Статус: Принято

Дата: 2026-07-29

Состояние реализации: реализовано. Gate
`communication_delayed_delivery_v1` открыт как `implemented`. `scheduler_v1`
реализован с live restart/revoke и hot-reconciliation evidence, а
module-originated schedule-control transport принят Kernel и Scheduler.
Реализованы отдельные `api`, `core` и persistence units delayed-delivery с
generated Schedule/Cancel/Status/realtime contract, hard request/body/time
bounds, cancellation-race policy и owner-local Storage bundle. Persistence
сохраняет только bounded Blob custody receipt, Scheduler inbox/outbox correlation
и execution fences; plaintext body в workflow SQL запрещён. Managed runtime
теперь материализует private body в encrypted Blob custody до создания
operation. Создание operation
и Ensure outbox атомарно, Cancel revision-fenced и атомарен со своим outbox,
Scheduler result дедуплицируется по inbox ID/hash до mutation. Encrypted Blob
custody receipt используется затем для one-use due execution. Persistence также предоставляет
bounded exact Scheduler command/receipt outbox relay с hash-bound idempotent
publication, owner-scoped Status с authoritative created/updated timestamps и
атомарный replayable client-realtime transition log. Due command уже atomically проходит
`scheduled|cancel_requested -> due -> dispatching`, сохраняет exact
run/schedule/lease fence и acceptance receipt outbox. Delivery-intent acceptance
может завершить operation и записать terminal Scheduler result только при живом
exact lease. Отдельная execution unit реализует owner-local due orchestration
через compile-isolated ports: one-use body read с проверкой custody size/digest,
stable delivery-intent request, fenced accepted/failed transition, terminal
Scheduler receipt и атомарный durable cleanup job. Отдельная cleanup
orchestration responsibility завершает Blob custody только после terminal
business commit; failure сохраняет retry с bounded exponential backoff и
переживает новый persistence connection. Отдельная event-adapter
unit уже строит exact Scheduler command envelope с runtime/grant fences и
проверяет correlated Scheduler result до persistence mutation. Она также
строго допускает только due `ScheduledJobCommandV1` своего exact JobKind,
связывает command metadata, scheduled time и RuntimeLease и строит стабильные
acceptance/terminal Scheduler receipts без синтетических command-объектов.
Отдельная runtime-adapters unit реализует bounded Blob write,
receipt-bound Blob read,
terminal-reason-bound custody release и exact delivery-intent `request_rpc`
через один sequential managed-control port. Managed runtime реализует inherited
control authentication, owner-local Storage binding, method-exact
Schedule/Cancel/Status routing и cursor-based client realtime publication в
единый Gateway SSE stream. Schedule и Cancel используют разные exact contracts
по ADR-0345. Runtime также получает fenced Event Hub credential, публикует exact
Scheduler command/receipt outbox и принимает correlated schedule results только
если causation ссылается на owner-local сохранённый command. Due-command
execution подключён к отдельным execution/runtime/store adapter units: exact
Scheduler command декодируется до claim, Retryable не подтверждает JetStream
delivery, а accepted/rejected подтверждает его только после durable terminal
receipt. Descriptor revision 2 отдельно материализует exact
`communication_delivery_intent.command` dependency в Kernel Control Store,
чтобы `request_rpc` не зависел от устаревшей нормализованной записи предыдущей
descriptor revision. Scheduler materializes due commands с canonical module
source `makosh-scheduler-runtime`, а не с private registration id: consumer
проверяет source как public runtime identity и не принимает Kernel-private
registration identity за producer contract. Отдельная assembly unit
материализует runtime binary, exact descriptor, settings schema и Storage
bundle как unsigned release fragment без runtime поведения. Development release
compiler принимает этот exact fragment в общий подписываемый distribution
input.

Live development evidence: новая delayed operation прошла
`schedule_pending -> scheduled -> dispatching -> failed`; controlled terminal
failure получен от Delivery Intent для отсутствующего тестового conversation,
а due command подтверждён в JetStream только после durable terminal transition
и попытки Blob custody cleanup. Это доказывает полный runtime contour до
provider-neutral Delivery Intent request; provider execution остаётся
ответственностью выбранной integration и не входит в этот workflow.

Отдельная test-only assembly unit
`makosh-communication-delayed-delivery-testkit` теперь запускается через
authenticated-storage runner на disposable PostgreSQL. Live conformance
доказывает exact create idempotency и hash conflict, Scheduler result
inbox duplicate/conflict, stale cancel revision, Scheduler-authoritative
`too_late` race, due claim duplicate/lease conflict, terminal receipt outbox,
replayable transition log и восстановление terminal state после нового
persistence connection. Этот storage evidence не заменяет managed-process
restart/revoke/outage contour.

Live managed NATS-outage conformance теперь дополнительно доказывает, что
Schedule RPC во время остановленного Event Hub сохраняет exact operation и
Scheduler command в состоянии `schedule_pending`, не останавливая
delayed-delivery runtime. После подтверждённого NATS reconnect тест дожидается
terminal delivery из прежнего durable command без повторного Schedule RPC и
без private body в realtime. Scheduler остаётся active: transient
receive/publish/ack failures проходят bounded exponential backoff, а
cross-stream terminal-before-acceptance redelivery не превращается в process
failure. Этот evidence закрывает только NATS outage item; отдельные outage
items не выводятся из него.

Отдельный live Scheduler-outage contour останавливает active Scheduler при
здоровом NATS, принимает новый Schedule RPC в `schedule_pending` и подтверждает,
что delayed-delivery runtime остаётся active. Fenced Scheduler successor затем
завершает сохранённую operation из прежнего command/outbox без повторного
Schedule RPC; terminal SSE снова не содержит private body. Этот evidence
закрывает Scheduler outage item.

Отдельный live Blob-outage contour останавливает только active Blob и
подтверждает typed `UNAVAILABLE` на Schedule RPC. Status того же operation ID
возвращает `NOT_FOUND`: без custody receipt workflow не создаёт durable
operation и не сохраняет plaintext fallback, а delayed-delivery runtime
остаётся active. После запуска fenced Blob generation successor тест повторяет
exact те же protobuf request bytes, получает terminal delivery и доказывает
отсутствие private body в realtime. Этот evidence закрывает Blob outage item.

Отдельный live ambiguous-request contour пропускает первый delivery-intent
`request_rpc` через настоящий Kernel capability route и настоящий
Delivery Intent runtime до успешной provider-neutral mutation, после чего
test-only transport decorator теряет только ответ вызывающему workflow.
Delayed-delivery не подтверждает due command, получает его повторно после
JetStream `ack_wait` и отправляет в Delivery Intent exact те же request bytes
с тем же stable delivery operation ID. Delivery Intent возвращает существующий
accepted receipt без повторной business mutation, workflow доходит до
`delivery_accepted`, а terminal SSE не содержит private body. Этот evidence
закрывает ambiguous request/result-loss item, но не подменяет отдельный
cancellation-race contour.

Live managed cancellation contour теперь сначала дожидается настоящего
Scheduler `ensured`, останавливает только Scheduler и через generated Cancel
route атомарно сохраняет `cancel_requested` вместе с exact CancelOneShot
outbox. Exact повтор того же client request возвращает сохранённый `existing`
receipt без пересборки time-dependent envelope, а новый expected revision
возвращает typed `STALE_REVISION`. Fenced Scheduler successor получает уже
опубликованную durable command, возвращает `cancelled`, workflow связывает
результат с отдельным `schedule_id`, а не с Cancel command `operation_id`, и
публикует sanitized terminal SSE без private body. Disposable PostgreSQL
conformance отдельно сохраняет Scheduler-authoritative `too_late` race; live
test не синтезирует этот result в обход Scheduler. Этот evidence закрывает
managed Cancel routing, duplicate/stale negatives и cancellation-successor
recovery.

Live managed contour дополнительно доказывает, что и delivery acceptance, и
successful cancellation проходят exact capability-routed Blob custody release:
Blob crash-safe ledger содержит committed deletion reservation, а private body
не попадает в durable event, status или SSE. Совместно с disposable PostgreSQL
retry/reconnect conformance и полным project gate это закрывает последний
admission item и открывает workflow gate.

Уточняет:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0214](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0330](ADR-0330-provider-neutral-communication-delivery-intent-workflow.md);
- [ADR-0333](ADR-0333-delivery-intent-target-bound-blob-materialization.md);
- [ADR-0339](ADR-0339-capability-routed-module-request-rpc.md);
- [ADR-0343](ADR-0343-capability-routed-blob-custody-release.md).

## Контекст

Clean-room reconstruction требует отдельный workflow для scheduled delivery,
acceptance и cancellation. Это не таймер внутри Mail/Telegram/WhatsApp/Zulip,
не поле `deliver_at` в Communications и не business scheduler в Kernel.

Responsibilities уже разделены:

- Communications владеет canonical evidence;
- provider integrations владеют compose и provider execution;
- `communication_delivery_intent` владеет немедленным provider-neutral
  acceptance одного outbound intent;
- Scheduler владеет time policy, due points, run leases и technical dispatch;
- новый workflow должен владеть только lifecycle отложенной доставки.

Существующий owner-control `UpsertSchedulerSchedule` не является module API.
Workflow не получает owner session, не вызывает Gateway и не импортирует
Kernel implementation. Для independently restartable owners нужен durable
module-to-Scheduler contract через event spine.

## Решение

Вводится отдельный workflow owner `communication_delayed_delivery` с единицами
сборки:

```text
makosh-communication-delayed-delivery-api
makosh-communication-delayed-delivery-core
makosh-communication-delayed-delivery-persistence
makosh-communication-delayed-delivery-execution
makosh-communication-delayed-delivery-event-adapters
makosh-communication-delayed-delivery-runtime-adapters
makosh-communication-delayed-delivery-store-adapters
makosh-communication-delayed-delivery-runtime
makosh-communication-delayed-delivery-assembly
```

API содержит generated Schedule/Cancel/Status/realtime contracts. Core
валидирует lifecycle и cancellation race. Persistence владеет workflow
operation, body custody reference, Scheduler correlation и owner-local inbox /
outbox. Execution unit владеет только fenced due orchestration через public
ports. Event-adapters unit владеет exact Scheduler `DurableEnvelopeV1`
construction/admission mapping и не выполняет transport I/O. Runtime обслуживает
client contract и managed lifecycle. Runtime-adapters unit владеет только
Kernel-routed Blob/request transport, не persistence и не lifecycle. Assembly
создаёт отдельный signed runtime/storage fragment. Store-adapters unit
реализует только execution persistence port и явно отображает owner-local
execution models в persistence models без SQL или transport logic.

Ни одна unit не импортирует Communications implementation, integration
runtime/persistence, Scheduler implementation/persistence или Kernel
implementation.

## Согласование с Kernel, Scheduler и delivery-intent

### Kernel и Core

Kernel:

- проверяет registration, exact capability grants, runtime generation и grant
  epoch;
- маршрутизирует opaque durable envelopes и exact `request_rpc`;
- не декодирует body, conversation identity или schedule semantics;
- не создаёт schedule и не выбирает provider.

Core Gateway:

- передаёт generated Schedule/Cancel/Status request в exact workflow runtime;
- возвращает immediate workflow receipt;
- доставляет client-safe invalidation через общий replayable SSE;
- не вызывает Scheduler или provider integration от имени клиента.

### Scheduler

Scheduler остаётся единственным platform time authority. Он получает только
exact durable schedule-control commands:

```text
scheduler.schedule.command.v1
  EnsureOneShot
  CancelOneShot

scheduler.schedule.result.v1
  ensured | cancelled | too_late | rejected
```

Module-originated schedule control проходит через producer outbox,
`DurableEnvelopeV1`, NATS JetStream и Scheduler inbox. Result возвращается
через Scheduler outbox тем же event spine. Это не `control_rpc`,
`client_rpc`, direct socket или cross-owner SQL.

One-shot due dispatch использует существующий `ScheduledJobCommandV1`:

```text
job_kind = communication.delayed_delivery.execute.v1
scope_id = delayed_operation_id
trigger_kind = scheduled
```

Command не содержит body, conversation/provider identity или Blob reference.
Scheduler хранит opaque scope, schedule/run identity, policy, due point и
technical result; workflow хранит business orchestration state.

### Delivery intent

После durable acceptance due command owner-local executor:

1. дедуплицирует Scheduler command в workflow inbox;
2. создаёт/возвращает existing execution по exact run/lease fence;
3. получает private body из workflow-owned Blob custody;
4. вызывает public `communication.delivery_intent.command` через exact
   capability-routed `request_rpc`;
5. сохраняет acceptance receipt;
6. публикует fenced Scheduler terminal result.

Один и тот же `delivery_operation_id` используется при ambiguous retry.
`accepted` delivery-intent не означает provider completion. Provider terminal
delivery остаётся в delivery-intent status/realtime и не становится Scheduler
state.

## Public contract

### Schedule

Client передаёт:

- `protocol_major = 1`;
- exact non-zero 16-byte `delayed_operation_id`;
- exact non-zero 16-byte `delivery_operation_id`;
- canonical `conversation_id`;
- optional canonical `reply_to_message_id`;
- private non-empty UTF-8 body не более 64 KiB;
- absolute UTC `deliver_at_unix_millis`.

`deliver_at` обязан быть не раньше чем через 5 секунд и не дальше чем через
366 дней от authenticated Kernel Clock reading. Client wall clock не является
authority. Один request ограничен 128 KiB.

Schedule сначала durably сохраняет workflow operation и Blob custody receipt,
затем публикует idempotent `EnsureOneShot`. Immediate receipt имеет состояния
`accepted` или `existing`; он не обещает, что Scheduler уже подтвердил
schedule.

### Cancel

Cancel принимает exact `delayed_operation_id` и expected workflow revision.
Он сохраняет `cancel_requested`, затем публикует idempotent
`CancelOneShot`.

Scheduler является authority cancellation race:

- `cancelled` допустим только до durable acceptance due dispatch;
- после accepted/running due run возвращается `too_late`;
- workflow не помечает operation cancelled до Scheduler result;
- cancellation после delivery-intent acceptance не является undo.

Повтор Cancel идемпотентен. Stale revision, terminal operation и foreign owner
fail closed.

### Status

Status возвращает только:

- delayed operation ID;
- sanitized state и monotonic revision;
- requested due time;
- delivery-intent ID после acceptance;
- typed bounded error code;
- client-safe timestamps.

Body, Blob reference/proof, provider/account identity, Scheduler runtime
coordinates и raw error text не возвращаются.

## Durable lifecycle

```text
accepted
  -> schedule_pending
  -> scheduled
  -> due
  -> dispatching
  -> delivery_accepted

schedule_pending | scheduled
  -> cancel_requested
  -> cancelled | scheduled

accepted | schedule_pending | scheduled | due | dispatching
  -> failed
```

`scheduled` означает только Scheduler acceptance. `delivery_accepted` означает
только delivery-intent acceptance. Provider delivered/failed не копируется в
delayed workflow.

Каждый transition требует current workflow revision. Job execution additionally
requires exact Scheduler run ID, schedule revision, lease epoch/expiry and
current runtime/grant fences. Stale worker не может materialize body, вызвать
delivery-intent, завершить run или удалить custody.

## Private body custody

Workflow PostgreSQL не хранит plaintext body. До due body находится в
workflow-owned encrypted Blob custody:

- Blob write uses exact owner/runtime/operation binding;
- persistence хранит только opaque reference, digest, size и custody proof;
- runtime получает one-use scoped read lease только для current fenced due
  execution;
- terminal cancellation, rejection or delivery-intent acceptance удаляет
  workflow custody через idempotent cleanup command;
- cleanup failure остаётся durable technical retry и не меняет business
  outcome;
- body не попадает в Scheduler payload, subjects, logs, errors, health,
  realtime или status.

Blob orphan cleanup является bounded platform maintenance, а не скрытым timer
workflow runtime.

## Units и SRP

```text
delayed-delivery API
  generated client request/status/realtime schemas

delayed-delivery core
  time bounds, lifecycle and cancellation race policy

delayed-delivery persistence
  operation, custody reference, inbox/outbox and execution fences

Scheduler durable adapter
  exact Ensure/Cancel command and result mapping

owner-local Job Executor
  due claim, Blob materialization and delivery-intent request

assembly
  descriptor, settings, Storage bundle and release fragment
```

Scheduler protocol owns generic schedule/run contracts. Delayed workflow owns
only `communication.delayed_delivery.execute.v1` semantics. Delivery-intent
owns outbound acceptance. Сходство полей не является причиной объединить units.

## Phase gate `communication_delayed_delivery_v1`

Gate открывается только вместе с:

1. implemented `scheduler_v1`, включая live successor restart/revoke и hot
   reconciliation;
2. exact module-originated Scheduler command/result contracts и grants;
3. девятью отдельными delayed-delivery packages и Cargo boundaries;
4. generated Schedule/Cancel/Status/realtime contracts и hard bounds;
5. owner-local Storage bundle, idempotent operation and state transitions;
6. encrypted Blob custody without plaintext workflow SQL;
7. exact one-shot schedule correlation and `ScheduledJobCommandV1` executor;
8. cancellation-race authority and `too_late` semantics;
9. exact delivery-intent `request_rpc` with stable operation ID;
10. managed client RPC and shared SSE invalidation;
11. restart, stale lease, revoke, NATS outage, Scheduler outage, Blob outage,
    duplicate, cancellation race and ambiguous request negatives;
12. live managed contour through real Scheduler and delivery-intent runtimes;
13. architecture, SRP, Cargo, Clippy and full test gates.

Gate имеет состояние `implemented`: весь перечисленный evidence пройден.
Frontend обязан использовать generated contracts и shared replayable SSE;
fake scheduled records или fake completion по-прежнему запрещены.

## Отклонённые варианты

### Таймер внутри workflow runtime

Не переживает restart и создаёт второй источник времени/расписания.

### Вызов Kernel owner-control из workflow

Подменяет module authority owner session и связывает workflow с private Kernel
API.

### Client координирует workflow и Scheduler

Создаёт неатомарный cross-owner saga в UI и ломает mobile/headless clients.

### Schedule хранится в Communications

Смешивает canonical evidence domain с outbound orchestration.

### Provider-specific scheduled send

Не даёт единых cancellation/receipt semantics и смешивает provider capability
с provider-neutral workflow. Если provider имеет собственную scheduled-send
функцию, она остаётся отдельным integration capability.

### Body в Scheduler payload или schedule state

Передаёт private content platform owner и нарушает payload/logging boundary.
