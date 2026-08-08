# ADR-0203: Управление локальной инфраструктурой и восстановление

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md).

Уточняется:

- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

Макошь должен самостоятельно управлять локальными PostgreSQL, PgBouncer, NATS
JetStream, Telemetry Collector, Vault и module runtimes. При этом отдельный
обязательный Host Supervisor рядом с Kernel увеличил бы количество процессов и
создал ещё одну lifecycle-границу до появления доказанной необходимости.

Kernel не может перезапустить собственный процесс. Для этого всегда нужен
внешний watchdog, но в desktop topology эту роль уже может выполнять Tauri, а в
headless topology — операционная система.

Автоматический restart инфраструктуры опасен, если он запускается по любому
красному health check. Connection exhaustion, recovery, disk full, повреждение
данных и configuration mismatch требуют разных действий. Restart не должен
скрывать причину, создавать crash loop или заменять повреждённое хранилище новым
пустым cluster.

## Решение

### Иерархия supervision

Supervisor является независимой подсистемой Макошь Kernel, а не отдельным
обязательным Макошь-процессом:

```text
Tauri или OS watchdog
        ↓
Макошь Kernel process
  ├── supervisor subsystem
  ├── Settings Registry
  ├── capability router и Event Hub
  ├── Telemetry Hub control surface
  ├── Telemetry Collector
  ├── PostgreSQL
  ├── PgBouncer
  ├── Storage Control managed process
  ├── NATS JetStream
  ├── Vault managed process
  └── module runtimes
```

Ответственность распределяется так:

| Уровень | Ответственность |
|---|---|
| Tauri / OS watchdog | запустить и bounded-перезапустить Kernel process |
| Kernel supervisor | управлять Telemetry Collector, PostgreSQL, PgBouncer, Storage Control, NATS, Vault и module runtimes без SQL introspection |
| Storage Control | bootstrap/reconciliation PostgreSQL, roles/grants, migrations, PgBouncer bindings и typed storage attestation |
| Module runtime | управлять только собственными workers и provider resources |

Tauri не принимает storage, routing или business решения. В headless режиме та
же граница реализуется через `launchd`, `systemd` или другой OS supervisor.

### Независимость supervisor subsystem

Supervisor event loop не зависит от доступности Telemetry Collector,
PostgreSQL, PgBouncer, NATS, Vault, module runtime или внешнего API. Он обязан
продолжать работать в recovery mode, когда любой из этих компонентов
недоступен.

Минимальное supervisor state:

- desired service state;
- child process handle и проверенная identity;
- runtime instance ID;
- restart counters и backoff deadline;
- последний sanitized health result;
- control endpoint metadata;
- startup/shutdown phase.

Это состояние находится в памяти. Если для crash reconciliation нужен локальный
runtime-state file, он:

- хранится в private runtime directory;
- записывается атомарно;
- не содержит credentials или private content;
- является rebuildable metadata, а не canonical truth;
- не требуется для восстановления business data.

### Режим supervision и module lifecycle

Каждый infrastructure service имеет явный supervision mode:

- `managed_child` — Kernel запустил приватный child process и имеет право его
  останавливать и перезапускать;
- `external` — сервис запущен пользователем или операционной системой; Kernel
  только проверяет health и никогда не посылает ему stop/restart signal.

Для module runtime точные lifecycle modes `managed`/`external` принадлежат
Module Registry registration/grant approval state ADR-0215. Для platform
infrastructure supervision mode хранится как отдельное typed infrastructure
registry state. Оба состояния живут в trustworthy Kernel Control Store, но не
являются `SettingsSchemaV1` entries ADR-0222 и не могут быть изменены через
Settings Registry.

Изменение module lifecycle mode выполняется только отдельной owner-authorized
registration/approval operation ADR-0215 и сохраняется до lifecycle side
effect. Эти modes не являются pre-store bootstrap inputs; Kernel не определяет
их эвристически по порту, PID или имени процесса.

Desktop default:

- managed Telemetry Collector с private local store;
- приватный managed PostgreSQL cluster в Макошь data directory;
- managed PgBouncer;
- managed Storage Control runtime ADR-0224;
- managed NATS с приватным JetStream directory;
- managed Vault как отдельный verified OS process ADR-0223;
- managed module runtimes.

Системный, Homebrew, Docker или иной внешний PostgreSQL не становится managed
автоматически.

### Идентификация managed child

PID сам по себе недостаточен из-за повторного использования PID. Перед adopt,
signal или kill Kernel проверяет одновременно:

- runtime instance ID и owner nonce;
- verified `ManagedLaunchBinding`, exact executable digest и file identity
  ADR-0219;
- process start identity;
- private runtime/data directory;
- control handshake, если он поддерживается сервисом.

Kernel не посылает signal процессу, identity которого нельзя однозначно
подтвердить. После crash Kernel новый instance либо безопасно принимает
подтверждённый managed child под supervision, либо выполняет controlled stop и
restart. Произвольный процесс, найденный только по PID или порту, не adoption и
не завершается.

### Startup order

Порядок ниже является target после последовательного открытия phase gates
ADR-0225. В `kernel_recovery_only_v1` шаги managed launch, Telemetry, Vault,
Storage, NATS, Event Hub, Scheduler и modules не выполняются вообще; Kernel
останавливается на private Control Store и local recovery surface.

Стандартный managed startup:

1. получить single-instance lock на Макошь runtime/data directory;
2. проверить permissions, ownership, доступное место и version compatibility;
3. открыть Kernel Control Store и проверить integrity/schema version;
4. загрузить trustworthy desired infrastructure state, Settings Registry
   schemas, desired revisions и last-acknowledged effective hints;
5. проверить signed `DistributionManifestV1` и exact artifact kind bindings:
   descriptor/settings только для `module_runtime`, exact bytes для
   `infrastructure_executable`, digest/owner/revision для `storage_bundle`
   ADR-0219/ADR-0224;
6. проверить отсутствие неподтверждённых orphan children;
7. запустить Telemetry Collector; при отказе включить bounded emergency log и
   продолжить startup в `degraded`;
8. проверить exact Vault executable, запустить отдельный process в `sealed` и
   выполнять unlock только по policy ADR-0223;
9. запустить PostgreSQL и дождаться process/WAL recovery readiness без чтения
   business schema Kernel;
10. запустить verified Storage Control runtime;
11. получить от Storage Control typed attestation cluster identity, schema
    compatibility, extensions, roles, grants и admitted bundle ledger;
12. через Storage Control выполнить разрешённый migration/bootstrap phase;
13. запустить PgBouncer, а Storage Control проверяет normal pooled connection
    path и выдаёт только current-generation bindings;
14. запустить NATS и проверить JetStream storage и stream definitions;
15. выполнить Event Hub reconciliation catalog, streams, consumers и
    permissions;
16. запустить capability routing и outbox relay;
17. повторно проверить exact bytes и запустить module runtimes в порядке
    declared required capabilities, передавая каждому только его полный
    resolved settings snapshot;
18. объявить Kernel ready только после прохождения обязательных checks и
    подтверждения effective settings revisions обязательных capabilities.

Независимые модули могут запускаться параллельно после готовности их
dependencies. Ошибка необязательного модуля не блокирует readiness остальных
capabilities.

Если Control Store отсутствует или недоверен, startup останавливается в
restricted `recovery_only`: Kernel не проверяет orphan children, не принимает
их под supervision и не запускает managed infrastructure либо modules по
defaults.

### Ordered shutdown

Стандартный shutdown выполняется в обратном направлении:

1. API и routers прекращают принимать новые mutations;
2. modules получают `Quiesce` и прекращают claim новых работ;
3. in-flight operations завершаются в пределах deadline;
4. cursors, inbox и outbox state фиксируются;
5. module runtimes останавливаются;
6. outbox relay и data-plane consumers выполняют финальный checkpoint;
7. NATS останавливается без удаления JetStream state;
8. Storage Control переводит bindings в quiescing, запрещает новую credential
   delivery и подтверждает bounded drain старых aliases/sessions;
9. PgBouncer прекращает выдавать новые connections и останавливается;
10. Storage Control останавливается после фиксации sanitized reconciliation
    state;
11. PostgreSQL выполняет штатный checkpoint/shutdown;
12. Vault выполняет `Quiesce`, bounded `Drain`, zeroize и `Stop` после
    остановки всех потребителей credential leases ADR-0223/ADR-0224;
13. Telemetry Collector сохраняет финальные lifecycle/shutdown records,
    завершает bounded flush и останавливается;
14. supervisor освобождает runtime lock и завершает Kernel.

По истечении deadline следующий уровень может выполнить forced termination.
Каждый forced step попадает в sanitized shutdown report.

### Restart policy по сервисам

| Сервис | Автоматический restart | Ограничение |
|---|---|---|
| Module runtime | да | bounded backoff и crash budget |
| Telemetry Collector | да | bounded restart; fallback только в private emergency log |
| PgBouncer | да | не считать pool exhaustion причиной restart PostgreSQL |
| Storage Control | да | bounded restart; existing data plane не reset-ится, новые bindings/migrations блокируются до reconciliation |
| NATS | да | сохранить JetStream state; PostgreSQL outbox удерживает backlog |
| Vault | да | bounded restart; `sealed` не crash, cipher/schema/integrity failure требует explicit recovery ADR-0223 |
| PostgreSQL | ограниченно | только process exit или классифицированный recoverable failure |
| Kernel | внешний watchdog | bounded restart без reset managed data |

Ни один сервис не получает бесконечный restart loop. После исчерпания budget он
переходит в blocker state до явного действия или изменения внешнего состояния.

### Классификация отказов PostgreSQL

| Состояние | Действие |
|---|---|
| Process exit | bounded restart, затем readiness и recovery checks |
| Временная недоступность | продолжать probes до deadline, не restart сразу |
| Startup/WAL recovery | ждать завершения в пределах recovery policy |
| Connection exhaustion | backpressure и pool diagnostics; не restart database |
| PgBouncer failure | restart PgBouncer; не restart database |
| Disk full | fail closed и blocker; restart запрещён |
| Data/WAL corruption | fail closed и recovery workflow; restart loop запрещён |
| Configuration/auth mismatch | blocker; автоматический restart запрещён |
| Binary/data version mismatch | blocker; автоматический upgrade или reset запрещён |

Красный readiness probe сам по себе не является основанием для restart.

### Координированный restart PostgreSQL

При planned restart:

1. supervisor переводит storage capability в `quiescing` и резервирует next
   storage generation;
2. Core прекращает новые mutations;
3. DB-dependent modules завершают транзакции; checkpoint вызывается только для
   runtime, который объявляет его поддержку и требует его перед restart;
4. Storage Control переводит generation-scoped roles в `NOLOGIN`, прекращает
   credential leases, disable/drain/kill-ит old PgBouncer aliases и
   подтверждает zero matching PostgreSQL sessions;
5. PostgreSQL штатно останавливается и запускается;
6. supervisor дожидается process/WAL recovery, а Storage Control проверяет
   cluster identity;
7. Storage Control проверяет schema versions, extensions, admitted bundle
   ledger, roles и grants и возвращает typed attestation Kernel;
8. Storage Control commit-ит generation, ротирует roles/credentials и проверяет
   новые PgBouncer aliases;
9. modules получают только new-generation `StorageBindingV1` и scoped Vault
   credential leases ADR-0223/ADR-0224;
10. modules возобновляются;
11. NATS/outbox replay обрабатывает накопленный backlog.

При аварийном падении graceful steps до restart недоступны. Незавершённые
database transactions восстанавливаются PostgreSQL; Kernel не пытается
имитировать их commit или повторять non-idempotent external operations.

### Recovery mode

Kernel остаётся жив и предоставляет sanitized local health/control surface,
даже если storage или NATS недоступны. В recovery mode разрешены только:

- status и diagnostics без private content;
- controlled restart/stop для managed services только при trustworthy Control
  Store и owner-authorized device session;
- authorized correction/retry typed settings ADR-0222 при trustworthy Control
  Store; применение может ждать недоступный module или infrastructure;
- authorized Vault unlock и recovery entrypoints ADR-0223 только при
  trustworthy Control Store; destructive restore остаётся offline;
- online validate/export недоверенного Control Store;
- shutdown приложения.

Business commands и writes fail closed до восстановления обязательных
capabilities.

При unavailable/untrusted Control Store online restore/reset и infrastructure
mutations запрещены. Restore/reset выполняются offline при остановленном
Kernel, explicit `--data-dir`, exclusive lock и local confirmation по
ADR-0218.

ADR-0206 расширяет это правило до явного `recovery_only` state, который обязан
достигаться без PostgreSQL, PgBouncer, NATS, vault и module runtimes, и задаёт
доступность Gateway operations во всех состояниях Kernel.

### Неприкосновенность данных

Никакой автоматический restart не имеет права:

- удалять или переименовывать PostgreSQL data directory;
- выполнять `initdb` поверх отсутствующего или повреждённого cluster;
- создавать новый пустой cluster как fallback;
- очищать JetStream directory, streams или consumer state;
- применять destructive migration;
- сбрасывать roles/grants для обхода ошибки;
- удалять Vault database/anchor/key slots, credentials или provider sessions;
- переключать managed service на другой endpoint, storage driver или topology;
- выбирать прежнюю settings revision либо менять desired intent как
  автоматический fallback;
- восстанавливать backup без явного авторизованного действия.

Restart меняет только состояние процесса, но не identity и содержимое storage.

### Внешний watchdog

В desktop topology Tauri запускает Kernel как sidecar, следит за его process
exit и применяет bounded restart policy. Tauri не интерпретирует внутренние
module или storage errors и не перезапускает отдельные children Kernel.

Внешний watchdog не очищает runtime/data directories. После restart Kernel сам
выполняет managed-child reconciliation и полную readiness sequence.

Отдельный постоянный `makosh-host` process не вводится. Он может быть рассмотрен
новым ADR только при доказанной необходимости, например:

- headless runtime должен переживать закрытие Tauri;
- несколько клиентов подключаются к одному постоянно работающему Макошь;
- supervisor должен переживать полный crash Kernel process;
- требуется независимое обновление Kernel без остановки managed services.

## Отклонённые варианты

### Обязательный отдельный Host Supervisor process

Отклонён для первой topology как дополнительная lifecycle и packaging boundary
без доказанной необходимости. Логическая supervision boundary сохраняется
внутри Kernel.

### Supervisor, зависящий от PostgreSQL или NATS

Отклонён, потому что при отказе зависимости он потеряет возможность управлять
её восстановлением.

### Restart по любому failed health check

Отклонён: маскирует resource/configuration/corruption failures и создаёт crash
loops.

### Автоматический reset или fallback storage

Отклонён из-за риска необратимой потери canonical data и provider state.

## Последствия

Положительные:

- Kernel самостоятельно управляет всей приватной local infrastructure;
- отдельный обязательный Макошь supervisor process не нужен;
- отказ PostgreSQL или NATS не выключает control/recovery loop;
- отказ Telemetry Collector не останавливает Kernel или modules;
- managed и external services не смешиваются;
- restart policy не превращается в скрытый data reset;
- Tauri и headless watchdog имеют одну узкую ответственность.

Отрицательные:

- Kernel supervisor subsystem должен быть независим от остальных core tasks;
- требуется безопасная child identity и orphan reconciliation;
- startup/shutdown/recovery становятся отдельным test surface;
- полный crash Kernel всё равно требует внешнего watchdog;
- одна physical database остаётся общей infrastructure failure domain.

## Проверка решения

- kill каждого module runtime не завершает Kernel и соседние modules;
- kill Telemetry Collector включает bounded emergency log и scoped degraded
  state без остановки modules;
- kill PgBouncer приводит к его bounded restart без restart PostgreSQL;
- kill Storage Control блокирует новые bindings/migrations, но не
  перезапускает и не очищает PostgreSQL/PgBouncer; после restart требуется
  typed reconciliation attestation;
- kill NATS сохраняет PostgreSQL outbox и JetStream state после restart;
- kill PostgreSQL приводит к recovery checks и возобновлению через PgBouncer;
- connection exhaustion не перезапускает PostgreSQL;
- disk full, corruption и version mismatch переходят в blocker без restart
  loop;
- planned PostgreSQL restart выполняет quiesce, optional declared checkpoint и
  full generation-scoped role/alias/session fencing до new binding и resume;
  runtime без checkpoint support также проходит корректный drain;
- external service никогда не получает signal от Kernel;
- PID reuse и неподтверждённый orphan не приводят к завершению чужого процесса;
- managed executable с несовпавшим digest не запускается и получает
  `blocked_integrity` без fallback;
- supervisor restart повторно проверяет exact executable bytes;
- managed startup сверяет `ModuleDescriptorV1` и settings schema artifact
  digests до передачи desired snapshot;
- settings correction в trustworthy `recovery_only` создаёт новую desired
  revision и не требует PostgreSQL, NATS или Vault;
- unavailable/untrusted Control Store не допускает settings query/mutation
  сверх sanitized store status/validate/export;
- Kernel restart reconciles managed children без удаления state;
- ordered shutdown останавливает PostgreSQL после NATS/PgBouncer, а Telemetry
  Collector — последним managed service;
- Tauri restart policy ограничена и не управляет children Kernel напрямую;
- recovery health surface не содержит secrets или private content;
- ни один restart test не удаляет PostgreSQL, JetStream, vault или provider
  session state.
