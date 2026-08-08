# ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий

Статус: Принято
Дата: 2026-07-15
Состояние реализации: `scheduler_persistence_foundation_v1` включает owner-neutral
`makosh-scheduler-protocol` и отдельный `makosh-scheduler` reconciliation/lease
component. Они фиксируют versioned JobKind и payload contract binding, opaque
schedule scope, explicit opaque `ConcurrencyKeyV1`, bounded
trigger/overlap/misfire/retry/deadline policies,
revisioned schedule identity и run lease epoch/expiry. Deterministic Clock
conformance подтверждает stale-lease fencing; reconciliation не перезаписывает
equal revision и сохраняет active lease при update future schedule.
ADR-0285 расширяет owner-neutral protocol отдельным exact
`OwnerJobCommandV1`/`UpgradeReconciliation` contract для owner-local Job
Executor. Этот тип не имеет schedule identity или Scheduler policy и
валидационно не взаимозаменяем с `ScheduledJobCommandV1`. Первый
Telegram-specific owner-local executor реализован по ADR-0285 с exact durable
command, bounded checkpoint, runtime-generation/lease fencing и managed
conformance; он не добавляет Telegram handler в Scheduler.
`makosh-scheduler-persistence` предоставляет exact `StorageBundleV1` для
`makosh_platform.scheduler_schedules`, `scheduler_runs`, `scheduler_dispatches`
и bounded `scheduler_concurrency` slots; bundle проходит canonical digest и PostgreSQL
AST admission Storage Control. Schedule policy сохраняется только в
versioned canonical binary form; upsert использует schedule revision и
отклоняет stale или same-revision-conflicting configuration. Его fenced
PostgreSQL adapter в одной
transaction reaps expired leases, reserves a shared concurrency slot, advances
the due point, inserts the unique fire-key run и сохраняет accepted exact
`DurableEnvelopeV1` bytes как pending dispatch; terminal completion releases
the slot under the run lease epoch. Запуск и dispatch record откатываются
вместе: crash между reservation и broker relay не теряет принятый запуск.
Disposable PostgreSQL conformance proves
the concurrent shared-key race, independent-key parallel claim, a shared
`AllowBounded { max_parallelism: 2 }` limit, revisioned schedule insert/update
and stale/conflicting rejection, and release by terminal fence or expired
lease. Тот же persistence adapter выдаёт pending record только через
canonical exact-byte outbox relay: publish failure сохраняет record pending;
только broker acknowledgement атомарно меняет dispatch и run states.
Disposable PostgreSQL + JetStream conformance proves the permitted Scheduler
runtime publishes the stored bytes to its exact command subject.
`JobRunReceiptV1::ACCEPTED` уже принимается Scheduler persistence только в
отдельной PostgreSQL transaction для опубликованного dispatch с exact
`run_id`/command `message_id`/lease epoch; повтор того же acknowledgement
идемпотентен, а foreign или stale receipt не переводит run в `running`.
Terminal `SUCCEEDED`/`FAILED`/`CANCELLED` receipt после такого acceptance
применяется ровно один раз и освобождает fenced concurrency slot. Отдельный
`RETRYABLE_FAILED` сохраняет outcome exact dispatch, atomically переводит run
в `retry_wait` согласно persisted retry snapshot и остаётся идемпотентным при
redelivery; обычный `FAILED` не становится retry по умолчанию. Scheduler
receipt consumer получает exact bytes только через owner-neutral
`SchedulerReceiptDeliveryPortV1`; JetStream adapter открывает ровно
Kernel/Event-Hub-authorized pull consumer. Consumer сначала фиксирует fenced
acceptance/terminal state в PostgreSQL и только затем ACK-ит JetStream. Live
PostgreSQL + JetStream conformance доказывает оба шага для owner receipt.
`makosh-scheduler-runtime` уже является отдельным managed-child binary: он
проверяет descriptor-bound inherited channel, получает fenced PostgreSQL
credential только через Kernel-mediated ciphertext Vault route, открывает
PgBouncer pool только из typed Storage binding и получает отдельный ephemeral
NATS credential для topology-derived publisher и каждого exact receipt
consumer. После successful startup он запускает receipt workers, которые
фиксируют state до JetStream ACK, bounded due/retry materializer и relay уже
сохранённых pending dispatch. Materializer берёт fenced local Clock reading,
fail-closed на wall-clock/suspend discontinuity, сохраняет exact envelope/outbox
в той же PostgreSQL transaction, что и advance/disarm current schedule, и
допускает только subject из Kernel-derived publisher bindings. Retry получает
strictly newer lease epoch и новый immutable dispatch только для всё ещё
current schedule revision. Relay публикует только original exact bytes в
разрешённый command subject; ошибка worker завершает process для последующего
supervised restart с successor identity. Runtime
configuration несёт полный non-secret Storage fence (instance, owner, role
epoch, pool/budget и bundle digest/revision), а также exact `logical_owner_id`
и `runtime_instance_id`; child не восстанавливает identity по эвристике.
Kernel теперь имеет непубличный owner-control launch-контур с отдельными
reserve/bind/start шагами и exact `RestartSchedulerRuntime`: durable
managed-launch reservation создаётся до
выдачи зависимых ресурсов, а `StartReservedSchedulerRuntime` заново читает
ровно её и требует явно названный active Storage capability binding. Reload
повторно fences release binding revision, Kernel generation и grant epoch;
selected Storage binding также сверяется с текущими topology
revision/generation до staging exact verified artifact и configuration из
Vault/Event Hub topology. Одна Scheduler reservation допускает ровно одну
child attempt: после crash автоматический restart с теми же runtime identity,
generation или process-bound leases запрещён; successor требует новый
reserve/bind/start. `RestartSchedulerRuntime` собирает ровно этот successor
flow в одну owner-authorized операцию: он сначала fail-closed переводит
predecessor Storage binding в revoke, передаёт exact revoke активному Storage
runtime и останавливает active Scheduler child; только затем создаёт новую
reservation, выдаёт свежий Storage binding для её exact identity и запускает
verified artifact. Predecessor identity/lease не принимается и не
переиспользуется. Kernel Control Plane также держит fail-closed lifecycle worker
для уже admitted Scheduler: active Storage binding является единственным
durable desired-running intent, `Revoking` binding не может resurrect-ить
child, а missing child получает только fresh reserve/bind/start successor.
Тот же worker вычисляет owner-neutral fingerprint exact dispatch/receipt
topology, schedule-control grants и Event Hub revision. Изменение current
runtime generation, grant epoch или approved JobKind запускает fenced
Scheduler successor с новой Storage/Vault/Event credential identity; старый
static grant snapshot не продолжает принимать команды.
Kernel синхронно фиксирует initial Scheduler topology fingerprint после
foundation launch и до запуска registration/external-runtime workers. Поэтому
lifecycle worker сравнивает изменения с доказанным launch snapshot, а не с
гонкой своего первого poll, и не создаёт лишний successor при каждом boot.
Изменившийся fingerprint должен оставаться exact одинаковым в восьми
последовательных reconcile observations (около двух секунд при current poll)
до fence/launch successor. Это coalesces массовый module-plan refresh и не
выдаёт Scheduler промежуточный topology snapshot.
После Storage подтвердил новый binding, Scheduler persistence adapter делает
не более 120 попыток PgBouncer readiness с bounded 250 ms backoff (не более
30 секунд между быстрыми отказами); permanent
authentication/endpoint failure остаётся launch failure и не маскируется.
После трёх consecutive launch failures worker не retry-ит бесконечно и ждёт
explicit healthy owner start/restart. Disposable PostgreSQL+JetStream
conformance уже доказывает exact-byte relay, receipt commit до JetStream ACK,
acceptance/terminal/retry fencing. Live authenticated managed-runtime
conformance теперь соединяет этот contour с Kernel lifecycle: schedule revision
1 → 2 применяется без restart, crash приводит к fresh successor generation,
role epoch и credential lease revision, сохранённый one-shot dispatch
доставляется после restart, а revoked binding не resurrect-ит runtime. Receipt
delivery, materialization и relay не являются owner execution.
`SchedulerJobRequestV1` из validated `ModuleDescriptorV1` теперь сохраняется
в private Control Store как exact owner-bound JobKind contract request и
становится Scheduler catalog entry только после capability-level approval;
pending, foreign и duplicate request fail closed. Этот catalog не является
schedule-control API, не upsert-ит schedules и не заменяет exact owner
contract admission будущего gate. Kernel Event Catalog выводит из каждого
такого approved JobKind отдельный exact command publisher для единственного
approved Scheduler dispatch authority; provider/domain-specific список
publish routes в Scheduler descriptor не кодируется. Ambiguous Scheduler
authority fail-closed, а publisher отсутствует после revoke JobKind grant.
`JobContractBindingV1` и persisted Scheduler
schedule теперь несут nonzero contract revision; старая row без revision
отклоняется при decode до явного revisioned owner update. Private owner-control
теперь содержит typed `UpsertSchedulerSchedule`: после owner-session check
Kernel сверяет exact JobKind/revision/schema с current approved Scheduler
catalog и передаёт mutation только по authenticated inherited channel active
Scheduler runtime. Runtime сам декодирует versioned canonical policy и
upsert-ит свою PostgreSQL row; Kernel не получает SQL pool или schedule table
access. Это закрывает mutation seam. Live conformance подтверждает hot
revisioned reconciliation и automatic successor lifecycle только для durable
active binding; revoke останавливает runtime и не создаёт successor.
Runtime protocol теперь дополнительно фиксирует bounded non-secret набор
Event-Hub-authorized command publisher bindings и receipt bindings: publisher
содержит exact command subject, а для каждого approved owner receipt contract
есть по одной acceptance на `MAKOSH_ACK_V1` и terminal на
`MAKOSH_RESULT_V1`. Receipt binding содержит exact durable consumer, subject и
bounded JetStream budget; Kernel обязан сверить все bindings с approved
topology перед передачей managed child. Это не даёт Scheduler права самому
выбирать broker subject или consumer.
Pure deterministic planner уже фиксирует `skip`, one-shot и bounded catch-up
для fixed interval, а `fixed_delay` требует terminal completion перед
перевооружением. Cron expression пока намеренно fail-closed: timezone/DST
исполнитель ещё не реализован и не подменяется фиксированным offset. PostgreSQL
foundation уже materializes bounded pending queue/coalescing, включая
idempotent repeat fire. Pending fire переходит в fenced run через одну
PostgreSQL transaction с slot reservation, current-policy verification и
single-use deletion. Fixed-delay run не меняет due point на claim и rearm
только в terminal fenced completion как `finished_at + delay`; обновлённая или
disabled schedule не перезаписывается старым run. Atomic due-claim integration
и dispatch входят в реализованный Scheduler runtime. Retry snapshot хранится
вместе с run: после transient failure
тот же `JobRunId` ждёт bounded backoff, а retry claim требует строго больший
lease epoch. Старый worker после этого не может terminally complete run или
освободить его concurrency slot. Disposable PostgreSQL conformance подтверждает
этот переход, включая отказ stale completion; это всё ещё persistence
foundation, а не NATS delivery или owner execution runtime.

Все required decision fields и live managed evidence для `scheduler_v1`
закрыты; gate реализован. Это не открывает owner-specific Job Executor,
module-originated schedule-control contract или конкретный workflow:
они остаются отдельными capability gates.

Scheduler строит один canonical `DurableEnvelopeV1` для каждого persisted
dispatch: delivery `message_id` остаётся идентичностью exact-byte outbox
record, а `command_id`, correlation ID и `ScheduledJobCommandV1.job_run_id`
равны одному `JobRunId`. Fire key становится command idempotency key, payload
несёт schedule revision и lease epoch/expiry, а source fence фиксирует
Scheduler runtime generation. Поэтому redelivery не создаёт новый owner job,
а owner ack/result может быть сопоставлен с конкретным fenced run без
provider/domain-specific Scheduler code.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR уточняет запрет ADR-0206 на business schedulers внутри Kernel:
планирование является отдельной platform capability, а код задания всегда
остаётся внутри module-владельца. ADR не разблокирует запрещённые ADR-0208
domains или product projections; Graph rebuild остаётся только будущим
примером job kind до отдельного решения.

Scheduler и Job Plane не входят в `kernel_recovery_only_v1`. Их packages,
storage, NATS consumers и runtime activation открыты отдельным
`scheduler_v1` gate ADR-0225 после реализованных Clock, Storage, NATS, Vault,
Telemetry, module control plane и managed-launch trust.

## Контекст

Макошь должен выполнять несколько разных видов фоновой работы:

- периодический опрос внешних систем, например получение почты;
- задания, вызванные domain или integration event;
- ручные и отложенные операции;
- длительные AI jobs с progress, checkpoint и cancellation;
- будущие rebuild и maintenance operations.

Эта работа должна переживать restart Scheduler, module runtime и NATS,
поддерживать bounded concurrency/retry и не переносить business code в Kernel.
Расписания и operational policy должны меняться без остановки системы, но
динамическая загрузка произвольного executable code, scripts или Rust libraries
из database запрещена.

Один центральный worker с реализациями разных owners нарушил бы
compile isolation, failure isolation и ownership. Использование только NATS
также недостаточно: broker доставляет сообщения, но не является canonical
источником schedule configuration, module checkpoints и business result.

## Область применения

ADR обязателен для любой технической фоновой работы независимо от owner:

- integrations: provider polling, reconnect, backfill, attachment download и
  outbound delivery;
- domains: reminders, retention, validation и owner-specific maintenance;
- AI: analysis, extraction, classification, summarization и embeddings;
- workflows: delayed step, timeout, compensation и resumable coordination;
- platform: outbox delivery, bounded cleanup, backup verification и technical
  reconciliation;
- будущие engines и projections после их явной разблокировки.

Ни один owner не получает собственный обходной scheduler, detached timer или
необъявленную background queue. Различается только owner handler и его
параметры; registration, scheduling, delivery и lifecycle следуют этому ADR.

## Решение

Вводится platform capability **Макошь Job Platform** из двух частей:

1. отдельный independently restartable Scheduler runtime;
2. owner-local Job Executor внутри каждого module runtime, которому нужна
   фоновая работа.

Kernel supervises Scheduler как managed module runtime и перед каждым launch
проверяет его exact-byte binding по ADR-0219, затем identity и capabilities.
Kernel не хранит schedules, не создаёт job rows и не интерпретирует job payload.
Event Hub управляет NATS catalog, consumers, permissions и delivery health, но
не вычисляет время запуска и не исполняет jobs.

Базовый поток:

```text
time trigger      domain/integration event      manual client command
     ↓                        ↓                           ↓
 Scheduler              owner/workflow                owner API
     └───────────────────────┴───────────────────────────┘
                             ↓
                  producer PostgreSQL outbox
                             ↓
                     NATS JetStream command
                             ↓
                    target module inbox
                             ↓
                    owner-local Job Executor
                             ↓
          owner state/checkpoint + result/event outbox
```

Scheduler является producer только для time-based trigger. Event-triggered
работа создаётся consumer/workflow владельца, а manual work — owner-specific
command handler. Все три пути используют один Job Command Envelope, но не
обязаны проходить через Scheduler как центральный proxy.

## Термины

- `JobKind` — versioned тип технической фоновой работы, объявленный module,
  например `mail.fetch.v1`, `ai.analyze-evidence.v1` или
  `documents.extract-text.v1`;
- `JobSchedule` — изменяемая time policy для одного JobKind и scope;
- `JobRunId` — стабильный ID конкретного запуска;
- `JobCommand` — durable требование владельцу принять запуск;
- `JobExecution` — owner-local состояние фактического выполнения;
- `JobCheckpoint` — owner-local durable позиция продолжения длительной работы;
- `JobProgress` — bounded sanitized progress metadata;
- `JobResult` — terminal результат запуска;
- `JobLease` — ограниченное по времени право исполнять конкретный run;
- `ScheduleRevision` — версия persisted schedule configuration.

Термин `Task` не используется для технического job, потому что `Tasks` является
отдельным business domain Макошь.

## Где находится исполняемый код

Код задания принадлежит module-владельцу и компилируется в его runtime.
Универсальная физическая форма следует owner topology ADR-0211/ADR-0212:

```text
backend/src/<owner-kind>/<owner>/<core-or-implementation>/jobs/<job>.rs
    owner-specific job algorithm and policy

backend/src/<owner-kind>/<owner>/<adapter>/
    external protocol or technical adapter, when required

backend/src/<owner-kind>/<owner>/persistence/
    cursor, checkpoint and execution persistence

backend/src/<owner-kind>/<owner>/runtime/
    handler registration, lifecycle, cancellation and concurrency
```

Для integration `owner-kind` равен `integrations`, для domain — `domains`, для
workflow — `workflows`, для engine — `engines`, для platform owner —
`platform` или `services` согласно ADR-0211. Package не создаётся только ради
папки `jobs`: handler остаётся внутри cohesive owner core/implementation.

Scheduler не импортирует packages конкретного owner, не содержит глобальный
`match` по domains/providers/job kinds и не знает Rust function name. В
PostgreSQL и NATS запрещено хранить executable code, SQL fragments, module
paths, dynamic library paths или scripts.

Примеры применяют одну и ту же границу:

| Owner | JobKind | Где находится код | Чем владеет executor |
|---|---|---|---|
| Mail integration | `mail.fetch.v1` | Mail core + IMAP adapter | provider cursor и полученные records |
| AI domain | `ai.analyze-evidence.v1` | AI implementation + model adapter | analysis execution и AI result candidate |
| Documents domain | `documents.extract-text.v1` | Documents implementation + parser adapter | extraction checkpoint и document-owned result |
| Calendar domain | `calendar.deliver-reminder.v1` | Calendar implementation | reminder execution и Calendar event outcome |
| Workflow | owner-specific delayed step | конкретный workflow implementation | saga step/checkpoint/compensation |
| Platform service | owner-specific maintenance kind | соответствующий platform owner | только собственное technical state |

Graph/search/context rebuild jobs не создаются, пока product projections
заблокированы ADR-0208.

`ModuleDescriptorV1` capability объявляет descriptor JobKind:

- stable owner и job kind ID;
- contract и payload version;
- допустимые trigger kinds;
- default schedule template, если он существует;
- concurrency/overlap capabilities;
- cancellation/checkpoint capabilities;
- resource class и bounded limits;
- minimum compatible runtime protocol.

Descriptor описывает capability, а не передаёт реализацию.

## Package boundaries

Общий technical contract получает самостоятельный platform protocol, потому
что его потребляют Scheduler и несколько независимых module runtimes:

```text
backend/src/platform/scheduler/protocol/       makosh-scheduler-protocol
backend/src/platform/scheduler/implementation/ makosh-scheduler
backend/src/platform/scheduler/persistence/    makosh-scheduler-persistence
backend/src/platform/scheduler/runtime/        makosh-scheduler-runtime
```

`makosh-scheduler-protocol` содержит только JobKind descriptor, job payload
messages, schedule/execution lifecycle states и typed errors. Он не определяет
второй outer envelope: любой job command/result использует
`DurableEnvelopeV1` из `makosh-events-protocol` ADR-0220. Scheduler protocol не
содержит SQLx, NATS client, provider SDK, domain types или runtime bootstrap.

Owner-local handler остаётся в существующих `domain`, `integration`,
`workflow` или `engine` packages владельца. Отдельный общий
`makosh-worker-runtime`, registry всех handlers или Celery-like application
package запрещён.

## Владение persisted state

Исполняемый код и persisted state разделяются по ответственности.

### Scheduler владеет

- revisioned JobSchedule;
- enabled/disabled/tombstone state;
- `next_due_at`, `last_fired_at` и bounded misfire metadata;
- time-triggered JobRunId и dispatch record;
- schedule lease/fencing state;
- sanitized acceptance/terminal control status, полученный из owner result;
- schedule change и dispatch outbox records под scheduler identity.

Это technical control state, а не business result или product projection.

### Module-владелец владеет

- inbox deduplication конкретного command;
- JobExecution state, execution lease, heartbeat и attempt history;
- JobCheckpoint и resumable cursor;
- provider/domain-specific input validation;
- фактический business или operational result;
- result/event outbox;
- собственные accounts, provider cursors, messages, documents и другие owned
  records.

Scheduler для любого owner хранит только факт, что конкретный JobKind для
opaque scope должен быть вызван по schedule. Target module хранит собственные
cursor/checkpoint, execution outcome и owned result. Например, Mail хранит IMAP
cursor и provider records, AI — analysis checkpoint и candidate result,
Documents — parser checkpoint и document-owned extraction state. Credentials,
private input и provider session material остаются в соответствующих protected
owner/Vault boundaries и никогда не попадают в Scheduler state.

### Job configuration не является module settings

`JobSchedule`, enabled/tombstone, due time, retry, overlap, misfire, lease и
run state остаются canonical PostgreSQL state Scheduler. Kernel Settings
Registry ADR-0222 не хранит, не копирует и не применяет эти records.

Module settings могут влиять на owner-local behavior executor, например bounded
batch size, но не заменяют `JobSchedule` и не создают второй timing source of
truth. Общий client screen может визуально показать Scheduler section рядом с
module settings; queries и mutations всё равно идут в разные owner contracts и
не обещают cross-owner transaction.

### Shared platform state

Outbox/inbox/event tables остаются shared technical tables ADR-0202 с
role-aware RLS. Scheduler и module runtimes видят только строки собственной
identity. NATS JetStream хранит delivery message до acknowledgement/replay, но
не заменяет PostgreSQL source of truth.

Ни Scheduler, ни Gateway не читают owner tables для построения общего job
экрана. Client query получает schedule control state у Scheduler и подробное
execution/business state у owner через contracts; cross-owner SQL запрещён.

## Registration и startup reconciliation

При старте используется следующий protocol:

1. Module runtime выполняет `Hello`/`Describe` и передаёт exact bounded
   `ModuleDescriptorV1` с JobKind descriptors.
2. Kernel применяет registration state, runtime identity, protocol
   compatibility и effective GrantSet ADR-0215.
3. Проверенный descriptor становится доступен Scheduler через platform
   capability/catalog protocol; Kernel не интерпретирует его business fields и
   не пишет schedule tables.
4. Scheduler сверяет active JobKind catalog со своим persisted state.
5. Только после успешной сверки time-triggered capability получает readiness.

Reconciliation fail closed:

- JobKind pending/suspended/revoked module или без effective grant не
  регистрируется;
- несовместимая contract version блокирует schedule до provider call или owner
  mutation;
- schedule без активного handler становится `blocked_missing_handler`;
- удалённый JobKind не удаляет history и schedule автоматически;
- повторная регистрация того же `ModuleDescriptorV1` после restart не создаёт
  duplicate schedule или run;
- Kernel restart не изменяет persisted schedule configuration.

### Default schedules

Default schedule является versioned template из `ModuleDescriptorV1`, а не
неизменяемым hard-coded timer внутри runtime.

Template является owner-declared complete initial policy: он содержит
trigger/time policy и все обязательные поля валидного schedule из раздела
`Scheduling policies`, включая default `overlap_policy`, `misfire_policy`,
concurrency key/maximum parallelism, timeout/deadline и bounded retry policy,
а также jitter и timezone/DST policy, когда они применимы. После первого
создания эти значения становятся revisioned canonical `JobSchedule` Scheduler
и могут изменяться только через его typed commands. Live IDs/revisions,
enabled/tombstone и due state, leases, runs и user overrides не входят в
default template и никогда не записываются обратно в `ModuleDescriptorV1`.

- Global default создаётся Scheduler только при первом появлении identity, если
  persisted schedule и tombstone отсутствуют.
- Scope-specific default создаётся только после owner command
  `EnsureSchedule`, когда scope уже существует: account для integration,
  document/evidence scope для owner job или workflow instance для delayed step.
- Existing schedule никогда не перезаписывается default template при restart.
- Изменение default в новой версии module не меняет существующие schedules
  молча; нужна explicit schedule migration или `ResetToDefault` command.
- Disable сохраняет schedule и прекращает будущие runs.
- Delete создаёт durable tombstone, поэтому startup reconciliation не
  воскрешает пользовательски удалённое расписание.
- ResetToDefault является отдельным authorized command, увеличивает revision и
  явно удаляет tombstone.

Таким образом module может гарантировать наличие разумного initial schedule,
но не отбирает у пользователя последующее управление.

## Job command payload

Job dispatch использует обычный `DurableEnvelopeV1.command`. Общие
`message_id`, logical `command_id`, source/target, partition, deadline,
idempotency, causation/correlation, trace и source fence принадлежат outer
envelope ADR-0220 и не дублируются Scheduler schema.

Typed job payload содержит только Scheduler/job semantics:

- `job_run_id`;
- `job_kind` и `job_contract_version`;
- opaque `scope_id`, когда применимо;
- `schedule_id` и `schedule_revision` для time-triggered run;
- `trigger_kind` и `scheduled_for`;
- execution lease scope/epoch;
- bounded owner input или opaque owner-controlled reference.

Subject содержит только stable owner/contract tokens. Account IDs, private
identifiers и payload не помещаются в subject, logs или health.

## Durable acceptance и выполнение

Module consumer не удерживает JetStream ACK на всё время длительного job.

```text
BEGIN
  deduplicate command in owner-visible inbox
  create or return existing JobExecution
  persist initial execution/checkpoint state
  append DURABLE_ACCEPTANCE Ack-envelope when required
COMMIT
JetStream ACK

owner-local executor claims durable JobExecution
  → running / heartbeat / checkpoint
  → succeeded | failed | cancelled | expired | unknown_outcome
  → terminal result + optional progress/domain event outbox
```

JetStream ACK означает только broker acknowledgement после durable owner inbox
acceptance. При необходимости отдельный `AckMetadataV1` сообщает
`DURABLE_ACCEPTANCE`; это не один и тот же protocol. Terminal outcome
доставляется только `result`, progress — отдельным event. Crash после broker
ACK не теряет работу: owner-local executor снова находит persisted non-terminal
execution.

Длительная работа обязана поддерживать bounded execution lease. Stale worker
не может checkpoint или завершить run после lease epoch change.

## Scheduling policies

Поддерживаются только явно versioned policies:

- one-shot `at`;
- `cron` с timezone и определённым DST behavior;
- `fixed_interval` от planned fire time;
- `fixed_delay` от terminal completion;
- manual trigger;
- delayed/deferred command;
- event-triggered command, создаваемый owner consumer или workflow.

Для каждого schedule обязательны:

- overlap policy: `forbid`, `queue` с explicit `max_pending_runs`,
  `coalesce_latest` или explicitly bounded `allow`;
- misfire policy: `skip`, `fire_once` или `catch_up_bounded`;
- concurrency key и maximum parallelism;
- timeout/deadline;
- bounded retry policy;
- optional bounded jitter;
- timezone/DST policy для calendar schedules.

Unbounded catch-up, concurrency, queue и retry запрещены.

`concurrency_key` — opaque technical key конфликтующего ресурса, а не ID
schedule. Один polling schedule mailbox получает отдельный key на mailbox;
поэтому два разных mailbox могут идти параллельно, но два запуска одного
mailbox делят один slot. Keys не содержат mailbox address, secret или payload.
`forbid`, `queue` и `coalesce_latest` резервируют не более одного active run
на key; `allow` резервирует ровно объявленный `max_parallelism`. `queue`
сохраняет не более `max_pending_runs` durable due points, `coalesce_latest`
заменяет единственный pending point самым свежим. Очередь и coalescing
определяют судьбу следующего due point, но никогда не открывают
второй active run без `allow`.

PostgreSQL хранит slot отдельно от schedule row, поскольку один key может быть
намеренно разделён несколькими schedules. Claim сначала atomically увеличивает
`active_runs` только ниже pinned limit, затем сдвигает `next_due_at` и вставляет
deduplicated run; любая неудача откатывает всё. Terminal completion или expiry
lease освобождает тот же slot. Изменить max при active run нельзя; это требует
drain или нового revisioned key.

Deadline и expiry — это fence, а не обещание, что зависший provider SDK или
внешний HTTP request физически остановился. Scheduler сначала делает старый
lease неавторитетным, и owner executor обязан прекратить работу при
cancellation/deadline. Пока такой worker продолжает жить, его checkpoint,
terminal result и освобождение slot требуют exact current lease epoch и
отклоняются. Любая внешняя side effect operation должна использовать stable
`JobRunId` как owner-defined idempotency key; после неизвестного внешнего
исхода Scheduler не создаёт silent automatic retry.

Owner executor продлевает lease короткими heartbeat только до immutable
deadline исходного run. Late heartbeat не resurrects expired run; завершение и
failure report также требуют, чтобы lease был действителен в указанное время.
После expiry следующий run того же key получает новый fencing state только
через Scheduler claim, а не из ответа старого worker.

Policy выбирается owner contract, а не глобальным default для всех jobs:

- provider polling обычно использует `fixed_delay`, scope partition,
  `forbid` overlap и bounded jitter;
- AI analysis обычно использует event/manual trigger, idempotency по input и
  model/prompt revision, bounded concurrency по resource class;
- document processing использует checkpoint и `coalesce_latest`, если новая
  revision документа делает старую queued работу устаревшей;
- reminder использует one-shot `at` и explicit timezone/DST semantics;
- workflow timeout/compensation использует one-shot/deferred command и saga
  correlation.

Это примеры policy, а не особые архитектурные ветки. Каждый JobKind явно
объявляет поддерживаемые policies и bounded limits.

## Retry layers

Три разных механизма не смешиваются:

1. JetStream redelivery повторяет доставку до durable owner acceptance.
2. Owner execution retry повторяет только typed transient operation после
   acceptance.
3. Schedule misfire policy решает, что делать с пропущенным временем.

Каждый слой bounded и наблюдаем. Validation, authorization, incompatible
version и stale lease являются terminal. После неоднозначного внешнего
non-idempotent action результат становится `unknown_outcome`; automatic retry
запрещён.

End-to-end semantics остаётся at least once. Stable JobRunId, message ID и
idempotency key обязательны; exactly-once не обещается.

## Hot reload расписания

Без restart разрешено менять:

- schedule expression, interval или delay;
- enabled state;
- future effective time;
- concurrency, overlap и misfire policy;
- timeout, retry, jitter и resource limits;
- bounded owner input/reference.

Изменение выполняется только через typed command с expected
`ScheduleRevision`:

```text
BEGIN
  compare current revision
  persist next revision
  append schedule-changed outbox event
COMMIT
refresh active scheduler state
```

Текущий run закреплён за revision, с которой он был создан. Изменение влияет на
будущие runs. Disable не отменяет in-flight execution; cancellation является
отдельной authorized command. Scheduler периодически сверяет in-memory due set
с PostgreSQL, поэтому NATS outage или потерянное notification не оставляют
configuration навсегда устаревшей.

Прямая правка schedule tables, session-dependent `LISTEN/NOTIFY` как
единственный reload path и NATS KV как canonical configuration запрещены.

## Обновление исполняемого кода

Rust code не hot-loadится в существующий process, а Kernel и Scheduler не
скачивают и не устанавливают executable. Изменение job handler сначала
пересобирает только packages владельца и его runtime, после чего следует
explicit update ADR-0219:

1. Scheduler прекращает выдачу новых executions старому runtime;
2. старый runtime выполняет bounded drain/checkpoint и останавливается;
3. host updater/OS атомарно устанавливает signed bundled release либо владелец
   отдельно подтверждает новый owner-pinned `ManagedLaunchBinding`;
4. Kernel проверяет exact installed bytes по `DistributionManifestV1` либо
   owner-pinned `ManagedLaunchBinding`, затем совместимость `ModuleDescriptorV1`
   и job contract versions;
5. запускает новую process generation;
6. повышает lease epoch и явно переключает capability.

Queued commands старой contract version должны быть либо совместимы с новой
version, либо явно migrated/cancelled до cutover. Silent payload reinterpretation
и automatic fallback/rollback на старый binary запрещены.

## Failure behavior

| Отказ | Поведение |
|---|---|
| Scheduler runtime | новые time triggers временно не создаются; queued и owner-local jobs продолжаются |
| Один module runtime | останавливаются только jobs этого owner; persisted executions возобновляются после restart |
| NATS | producer outbox сохраняется; accepted owner-local jobs продолжаются |
| PostgreSQL/PgBouncer | новые claims/commits блокируются; processes остаются управляемыми и bounded reconnect |
| Event Hub | topology reconciliation unavailable; Scheduler не создаёт in-memory fallback transport |
| Telemetry Collector | execution продолжается; diagnostics degraded без подмены canonical state |
| Incompatible handler | schedule blocked; запуск и provider call не выполняются |

Scheduler claim использует короткую PostgreSQL transaction, row lease и
fencing, совместимые с PgBouncer transaction pooling. Session advisory lock не
используется. Атомарно создаются JobRunId, dispatch record, outbox message и
новый `next_due_at`, чтобы concurrent Scheduler instances не создавали два
разных run для одного fire point.

## Security и privacy

- JobKind descriptor и effective module grant обязательны; arbitrary
  target/function name запрещены. Publisher signature не является условием
  external registration по ADR-0215, но Scheduler и любой managed Job Executor
  запускаются только с verified `ManagedLaunchBinding` ADR-0219. Self-declared
  descriptor не является executable integrity proof.
- Schedule command авторизуется по owner capability и scope.
- Secrets, provider sessions, message bodies, documents, prompts и media bytes
  не хранятся в Scheduler и не передаются в NATS.
- Большой или private input передаётся через opaque owner record, `BlobRef` или
  `EvidenceRef` с capability/expiry, когда это разрешено owner contract.
- Logs, metrics, traces и health содержат только job/run identity, duration,
  state, bounded error class и sanitized resource metrics.
- Progress не является каналом для private content.
- Scheduler не получает Vault capability для provider credentials.
- Settings Registry не получает `JobSchedule`, JobRun, lease, checkpoint,
  retry/misfire state или owner job payload.

## Проверка решения

Перед изменением `Состояние реализации` обязательны tests:

- first registration создаёт default schedule ровно один раз;
- complete default template создаёт valid schedule, а отсутствие обязательной
  policy, включая `misfire_policy`, блокирует reconciliation до persistence;
- module/Kernel restart не создаёт duplicate и не меняет revision;
- user-modified schedule переживает restart и module upgrade;
- disable/delete tombstone не воскрешается reconciliation;
- scope-specific schedule создаётся только после owner `EnsureSchedule`;
- missing/incompatible JobKind blocks before dispatch;
- concurrent Scheduler claims создают один JobRunId на fire point;
- crash до/после schedule commit и до/после NATS publish acknowledgement;
- duplicate command создаёт одну owner JobExecution;
- ACK после durable acceptance и crash после ACK возобновляет execution;
- stale execution lease/epoch не может checkpoint или завершить run;
- owner overlap/concurrency policy соблюдается для integration, domain, AI,
  workflow и platform job fixtures;
- один concurrency key не допускает duplicate active run, independent keys
  допускают bounded parallel work, а exhausted shared limit не сдвигает due
  point;
- hot schedule update применяется к future run и не меняет in-flight revision;
- settings catalog не содержит Scheduler records, а composed client screen не
  превращает settings mutation в schedule mutation;
- disable, cancel, timeout, bounded retry и `unknown_outcome`;
- NATS outage/replay и Scheduler restart/misfire;
- owner restart не влияет на jobs соседнего module;
- explicit verified code replacement сохраняет compatible queued work;
- managed Scheduler/Executor update не запускает bytes без valid binding и не
  выполняет automatic fallback/rollback;
- diagnostics и persisted technical state не содержат private content или
  secrets.

Integration tests используют PostgreSQL, PgBouncer и NATS JetStream через
testcontainers. Live provider accounts не используются.

## Последствия

### Положительные

- schedules меняются без restart и без executable code в database;
- integrations, domains, AI, workflows и platform owners сохраняют собственный
  code/storage/lifecycle;
- Scheduler failure не завершает module runtimes;
- NATS, outbox/inbox и owner checkpoints дают durable restart semantics;
- изменение одного handler пересобирает только owner packages;
- Kernel и Event Hub не превращаются в business orchestration monolith.

### Цена

- Scheduler получает собственные persistence, leases и reconciliation tests;
- каждый owner обязан реализовать durable execution state и idempotency;
- result/status для клиента требует owner query, а не cross-owner SQL;
- schedule/handler version compatibility становится обязательной частью
  module rollout.

## Отклонённые варианты

### Kernel создаёт и исполняет jobs

Отклонено: Kernel начал бы знать business schedule, provider/account state и
handlers, нарушая закрытый список обязанностей ADR-0206.

### Один центральный Celery-like worker со всеми handlers

Отклонено: связывает owners compile-time и runtime, увеличивает rebuild fan-out
и превращает падение одного handler в общий failure domain.

### Хранить executable code или scripts в PostgreSQL

Отклонено: обходит signed distribution/owner-pinned launch binding, ломает
reproducible builds, code review, explicit update/rollback model и security
boundary.

### Использовать NATS как единственное job storage

Отклонено: JetStream обеспечивает durable delivery и redelivery, но не заменяет
canonical schedule revisions, owner checkpoints и business state.

### Event Hub одновременно является Scheduler

Отклонено: event topology/subscription reconciliation и вычисление времени
запуска имеют разные owners, причины изменения и failure semantics.

### Temporal как обязательный initial runtime

Отклонено для первого clean-room implementation: добавляет отдельный critical
workflow control/persistence service рядом с уже обязательными PostgreSQL и
NATS. Решение можно пересмотреть, если появятся доказанные многодневные durable
workflow requirements, которые Job Platform не покрывает.

### Apalis как cross-module architecture contract

Отклонено: библиотека может быть исследована как owner-local implementation
detail, но не должна владеть Макошь envelopes, module boundaries или
inter-module delivery semantics.

## Ссылки

- [NATS JetStream consumers](https://docs.nats.io/nats-concepts/jetstream/consumers)
- [NATS JetStream](https://docs.nats.io/nats-concepts/jetstream)
- [Temporal schedules](https://docs.temporal.io/develop/go/workflows/schedules)
- [Temporal self-hosted deployment](https://docs.temporal.io/self-hosted-guide/deployment)
- [Apalis](https://github.com/apalis-dev/apalis)
