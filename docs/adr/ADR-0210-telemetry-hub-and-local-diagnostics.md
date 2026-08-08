# ADR-0210: Telemetry Hub и локальная диагностика

Статус: Принято
Дата: 2026-07-15
Состояние реализации: `telemetry_foundation_v1` реализован: exact protocol и
Collector packages, private Unix transport, fixed-shape privacy validation,
per-source quota/reserve, bounded segment retention, inherited managed-runtime
handshake, signed release binding и crash/restart segment preservation. Полный
Signed release-bundle admission теперь требует exact `telemetry` platform
descriptor и owner; Kernel-managed crash loop ограничен тремя attempts и не
останавливает Kernel. `telemetry_v1` открыт: Collector capability имеет exact
package inventory, private IPC, fixed-shape privacy/quotas, bounded retention,
failure isolation и negative privacy conformance. Реализованная owner-private
diagnostics операция возвращает только `segment_count` и `total_bytes` через
authenticated inherited typed Protobuf control channel; она не открывает raw
logs, segment files или diagnostic export.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Связано с:

- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md).

Telemetry control остаётся конституционной обязанностью Kernel. Collector
foundation существует вне `kernel_recovery_only_v1`, но теперь является
открытой `telemetry_v1` platform capability. Kernel по-прежнему не включает
Collector автоматически и не предоставляет client-facing diagnostics/export
route: private owner-control aggregate diagnostics не является client-facing
route, а filtered client export остаётся частью `client_gateway_v1`.

## Контекст

Изолированные module runtimes, managed PostgreSQL, PgBouncer, NATS и Core
Gateway требуют общей технической наблюдаемости. Без единого telemetry contract
логи становятся несопоставимыми, command/event flow нельзя проследить между
процессами, а ошибка infrastructure скрывает собственные диагностические
данные.

Телеметрия не может зависеть от PostgreSQL или NATS: именно их отказ является
одним из главных диагностируемых сценариев. Она также не должна превращаться в
канал утечки message bodies, документов, provider sessions или credentials.

Если collector и Kernel находятся в одной process failure domain, log flood,
parser bug или исчерпание диска может остановить supervisor. Поэтому логическое
ownership ядра и process isolation collector должны быть разделены.

## Решение

### Роль Telemetry Hub

**Telemetry Hub** — обязательная technical capability Макошь для
структурированных logs, metrics, traces, crash/lifecycle reports и их
sanitized diagnostics.

Telemetry Hub логически принадлежит Kernel control plane, но состоит из двух
частей:

1. Kernel-owned policy/control surface: identity, capabilities, redaction,
   budgets, health и authorized diagnostics;
2. отдельный managed Telemetry Collector process: ingestion, buffering,
   rotation, retention и local query adapter.

```text
Kernel / module runtimes / managed services
                 ↓ private local telemetry channel
      Telemetry Collector process
                 ↓
 bounded private local telemetry store
                 ↓
 Kernel diagnostics policy → authorized sanitized client view/export
```

Collector не является business module и не владеет canonical truth.

### Telemetry signals

Поддерживаются четыре signal family:

- structured logs;
- metrics;
- distributed traces;
- crash, lifecycle и shutdown reports.

Каждая запись имеет versioned schema и минимум следующие technical fields,
когда они применимы:

- timestamp и severity/signal kind;
- `module_id` и `runtime_instance_id`;
- component и stable operation name;
- `trace_id` и `span_id`;
- `correlation_id` и `causation_id`;
- `message_id` или command identifier;
- lifecycle state;
- sanitized error class;
- schema version.

Свободный message text не является transport для произвольного payload.
Высококардинальные labels, user-provided keys и arbitrary nested metadata
запрещены.

### Correlation

Gateway создаёт или принимает trusted trace root. Kernel, workflows, domains и
integrations передают trace context через versioned RPC/event envelopes.
Каждый hop создаёт собственный span и сохраняет `causation_id`/
`correlation_id` исходной business operation.

Trace context является bounded technical metadata. Email, phone, account name,
chat title, document name, prompt, message fragment и другой private baggage в
нём запрещены.

Event Hub публикует только delivery/topology telemetry. Он не копирует event
payload в trace или log.

### Local ingestion

Production processes отправляют telemetry через versioned private local IPC с
runtime identity и scoped capability. Endpoint не публикуется наружу. Exact
wire encoding является implementation detail при сохранении versioned signal
schema и contract tests.

Telemetry не проходит через NATS, PostgreSQL outbox или business API. Это
предотвращает recursion и сохраняет диагностику при отказе data/storage plane.

Module stdout/stderr считается untrusted emergency input. Supervisor может
перехватить, ограничить и сохранить его только через отдельный bounded
emergency path. Module по-прежнему запрещено выводить private content или
secrets в stdout/stderr.

### Bounded delivery и backpressure

Telemetry никогда не блокирует business operation или supervisor loop на
неограниченное время.

Для каждого producer обязательны:

- bounded in-memory queue;
- maximum record size;
- per-signal rate и byte budget;
- bounded send deadline;
- overflow counters;
- deterministic shedding policy.

При overflow сначала отбрасываются verbose/debug, затем informational records.
Warning/error/crash records используют отдельный малый reserve budget, но также
не могут расти или блокироваться бесконечно. Любая потеря telemetry отражается
через `telemetry_dropped_total` и sanitized health state без рекурсивного
логирования каждой потери.

Один module не может исчерпать общий telemetry budget остальных runtime.

### Локальное хранение

Collector хранит telemetry в отдельном private local store, не являющемся
PostgreSQL, JetStream, vault или module-owned storage.

Store:

- находится в Макошь private data directory;
- использует directories `0700` и files `0600`;
- имеет bounded retention одновременно по age и total bytes;
- использует rotation и atomic segment finalization;
- переживает restart PostgreSQL, NATS и modules;
- допускает удаление истёкшей telemetry без влияния на canonical data;
- не включается в business backup по умолчанию.

Точный file/segment format и default retention values выбираются при
implementation и фиксируются configuration contract вместе с disk-pressure
tests. Unbounded retention запрещена.

### Privacy и redaction

В telemetry запрещены:

- message bodies, email subjects и chat text;
- document contents, filenames пользователя и media bytes;
- prompts, model input/output с private content;
- email addresses, phone numbers, usernames и provider account names;
- OAuth tokens, cookies, passwords, keys, bootstrap secrets и credential
  leases;
- raw provider payload, arbitrary headers и SQL parameter values;
- `BlobRef` capability tokens и vault references, если они позволяют доступ.

Producer использует allowlisted structured fields. Collector повторно
проверяет schema, size и forbidden field classes, выполняет redaction/truncation
и помечает rejected records. Sanitizer не считается заменой запрету producer
логировать private data.

Stable opaque technical IDs допускаются только при необходимости корреляции и
не должны быть обратимо получены из private identifier без keyed privacy
mechanism.

### Доступ и export

По умолчанию telemetry остаётся локальной и не отправляется во внешний cloud,
vendor или analytics service.

Core Gateway предоставляет:

- sanitized health и aggregate metrics;
- owner-authorized filtered local diagnostics;
- явный diagnostic export с preview, redaction и audit.

Paired Android получает только client-safe health/metrics. Доступ к raw local
logs или diagnostic export не наследуется из обычной client session и требует
отдельной owner capability.

Remote export, support bundle upload или third-party telemetry backend требуют
отдельного ADR и явного opt-in. Silent export запрещён.

### Lifecycle и failure isolation

Telemetry Collector запускается после Kernel bootstrap, но до PostgreSQL,
PgBouncer, NATS и module runtimes. До его готовности Kernel пишет минимальный
sanitized bootstrap/crash log в bounded emergency file.

Отказ Collector:

- не завершает Kernel или module runtimes;
- переводит telemetry capability и Kernel в `degraded`;
- активирует bounded emergency path;
- не переключает telemetry на NATS, PostgreSQL или remote service;
- вызывает bounded restart Collector через supervisor;
- после исчерпания restart budget создаёт blocker без crash loop.

Collector останавливается после modules, NATS, PgBouncer и PostgreSQL, чтобы
сохранить shutdown diagnostics. Forced termination и потерянные records входят
в sanitized shutdown report.

## Запрещено

- использовать telemetry как business event bus или canonical audit log;
- отправлять telemetry через NATS или PostgreSQL как обязательный path;
- неограниченные queues, labels, record size или retention;
- блокировать business mutation из-за заполненной telemetry queue;
- общий module-controlled log file без quotas;
- private content, secrets или raw payload в любом signal;
- remote export по умолчанию;
- выдавать raw logs обычной desktop/Android session;
- считать frontend analytics заменой runtime telemetry;
- автоматически удалять canonical data при disk pressure telemetry store.

## Отклонённые варианты

### Логи только в stdout/stderr

Отклонено: нет versioned schema, identity, correlation, bounded retention и
надёжной privacy boundary.

### Telemetry в PostgreSQL

Отклонено: log flood конкурирует с canonical data, а отказ PostgreSQL лишает
диагностики его собственного восстановления.

### Telemetry через NATS

Отклонено: создаёт recursion с Event Hub, конкурирует с durable messages и
исчезает именно при отказе event data plane.

### Collector внутри Kernel process

Отклонено: parser/storage/log-flood failure расширяет failure domain Kernel и
может остановить supervisor.

### Обязательная cloud observability

Отклонено: нарушает local-first и privacy model, создаёт внешнюю dependency и
не работает offline.

## Последствия

Положительные:

- все процессы используют единую correlation и signal schema;
- диагностика доступна при отказе PostgreSQL и NATS;
- log flood одного module ограничен quotas;
- collector failure не завершает Kernel;
- local-first исключает silent external telemetry;
- Event Hub наблюдаем без копирования event payload.

Отрицательные:

- появляется ещё один managed process и local store;
- producer libraries и schemas должны поддерживаться во всех runtimes;
- privacy tests и cardinality budgets становятся обязательными;
- diagnostic export требует отдельной authorization и UX поверхности.

## Проверка решения

Architecture guard уже требует `telemetry_control` внутри Kernel, отдельный
Telemetry Collector platform runtime и запрещает telemetry implementation
зависеть от NATS/PostgreSQL clients. Foundation evidence покрывает private
transport, exact signed descriptor admission, byte-bounded retention,
three-attempt managed crash isolation и owner-authorized aggregate diagnostics
through the inherited control channel. Эти доказательства открывают
`telemetry_v1`; filtered client export и Gateway/event correlation остаются
отдельными последующими slices и не расширяют текущую private capability.

До признания решения реализованным должны существовать tests:

- telemetry продолжает принимать Kernel diagnostics при остановленных
  PostgreSQL и NATS;
- crash Collector не завершает Kernel или modules;
- Collector restart не удаляет retained segments и не создаёт duplicate crash
  loop;
- module flood ограничивается собственным quota;
- overflow не блокирует business operation и увеличивает loss counter;
- debug/info shedding сохраняет отдельный bounded error/crash reserve;
- retention ограничена age и bytes;
- disk full создаёт telemetry blocker и не удаляет canonical data;
- trace/correlation/causation проходят Gateway → module → event → consumer;
- forbidden private fields и oversized records отклоняются;
- secrets/private content отсутствуют в logs, metrics, traces, health и export;
- paired Android не получает raw logs по обычной session;
- remote export отсутствует без отдельного opt-in ADR;
- ordered shutdown сохраняет telemetry до остановки PostgreSQL и NATS.

Client-facing export, Gateway/event correlation и shutdown ordering не входят
в текущую private Collector capability и должны получить отдельное executable
evidence в соответствующих later gates.
