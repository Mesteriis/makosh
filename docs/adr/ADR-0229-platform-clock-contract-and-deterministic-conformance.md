# ADR-0229: Platform Clock contract and deterministic conformance

Статус: Принято
Дата: 2026-07-17
Состояние реализации: `clock_v1` открыт. Реализованы exact contract/runtime
packages и isolated deterministic conformance suite; Scheduler, timers for
modules and calendar scheduling remain closed.

Уточняет:

- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0214: Durable Job Platform и Scheduler](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0225: Первый production slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

`SystemTime` и `Instant` имеют разные свойства. Смешение wall-clock с
in-process timeout продлевает deadline при ручной смене времени, а попытка
считать IANA timezone из machine-local settings делает поведение Scheduler
неповторяемым. Нужен маленький platform contract, который даёт одинаковую
семантику Kernel, Vault и будущему Scheduler, но не превращается в timer
service или скрытый Scheduler.

## Решение

Clock имеет ровно два production packages:

| Package | Metadata | Ответственность |
|---|---|---|
| `makosh-clock-protocol` | `platform:clock:contract` | UTC, monotonic reading, discontinuity policy и explicit timezone context |
| `makosh-clock-runtime` | `platform:clock:implementation` | system adapter и deterministic fake clock |

Оба лежат в `backend/src/platform/clock/{protocol,runtime}`. Отдельный
`makosh-clock-testkit` находится только в `backend/tests/support/clock`; он не
может быть production dependency.

### Time semantics

- `UtcMillisV1` — только absolute UTC timestamp; никаких local timestamp как
  durable truth.
- `MonotonicMillisV1` — только elapsed time, timeout и deadline внутри одного
  process lifetime. Его нельзя persist или сравнивать между restarts.
- `ClockReadingV1` несёт явный `Stable`, wall jump forward/backward либо
  `SuspendOrMonotonicGap`. Gap определяется policy threshold, поэтому runtime
  не притворяется, что может достоверно различить sleep и host stall.
- System adapter классифицирует wall drift относительно monotonic delta. Wall
  jump никогда не переписывает monotonic timeline.
- `TimeZoneContextV1` принимает explicit IANA name и observed UTC offset, но
  Clock не конвертирует civil time и не читает local OS timezone. IANA rules,
  DST ambiguous/nonexistent local time и calendar semantics принадлежат
  будущему Scheduler contract ADR-0214.

### Determinism and scope

`DeterministicClockV1` вручную advances both timelines, может отдельно
моделировать wall jump и resume gap. Он обязателен для tests и не читает host
clock. Production system adapter не зависит от SQLite, Vault, Kernel, NATS,
networking, timezone database или module packages.

`clock_v1` не создаёт process, listener, Control Store records, module
capability, recurring timer, job или public API. Поэтому Kernel остаётся без
`ready`, а `scheduler_v1` всё ещё требует Storage, NATS, Telemetry и свою
durable conformance.

## Evidence

Gate требует:

- exact inventory/allowlist в `backend/architecture/policy.json`;
- fake-clock tests для advance, wall jump, suspend/gap и invalid policy;
- explicit timezone-context validation;
- compilation обеих production packages без external dependencies.

## Consequences

Consumers получают одну typed boundary вместо direct ad-hoc `SystemTime` /
`Instant`. Эта boundary не является authorization clock: security expiry,
replay and persisted schedules должны иметь свои owner-specific policies и
fail closed semantics.
