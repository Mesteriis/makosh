# ADR-0216: Private Kernel Control Store на SQLite

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Реализовано для foundation runtime. Production port и
SQLite adapter, schema v1→v15, atomic migrations, single-writer actor,
integrity/export и fenced offline restore/reset имеют executable evidence.
Schema assertions cumulative: версия v15 обязана доказать все requirements
v1…v15, а не только объекты последнего migration step.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md).

Уточняется:

- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR выбирает физическое хранилище boot-critical control state, которое
ADR-0206, ADR-0215 и ADR-0222 требуют читать до запуска PostgreSQL, PgBouncer,
NATS и Vault. PostgreSQL остаётся canonical business/module database; SQLite не
заменяет его и не становится общей persistence capability.

## Контекст

Целевой Kernel после открытия соответствующих gates должен достигать
`recovery_only`, показывать registrations/grants, typed settings state и
управлять managed infrastructure, когда PostgreSQL и Vault ещё не запущены,
сломаны, заблокированы или остановлены. Текущий `kernel_recovery_only_v1`
ограничен status/validate/export и shutdown; registrations, settings и
infrastructure actions закрыты ADR-0225. Если единственная
копия desired topology, module approvals, grant epochs или settings revisions
находится в PostgreSQL либо для её чтения нужен Vault key, возникает
неразрешимый boot cycle:

```text
Kernel должен прочитать control state, чтобы запустить PostgreSQL
                              ↓
            control state доступен только после запуска PostgreSQL
```

Обычный JSON/CBOR-файл убрал бы external service dependency, но заставил бы
Макошь самостоятельно реализовать multi-record transactions, crash-safe
rename/fsync, locking, migrations, constraints и corruption recovery. Для
состояния authorization и lifecycle частично записанный transition опаснее,
чем отказ всей операции.

Нужен маленький embedded store с:

- ACID transaction для approval, grant revision и epoch;
- автоматическим rollback после process/OS crash;
- schema constraints и monotonic migrations;
- single-writer ownership;
- диагностируемым локальным форматом;
- отсутствием отдельного daemon и network listener;
- доступностью до managed infrastructure.

## Решение

### SQLite является private technical store Kernel

Kernel использует один private SQLite database в Макошь Kernel data directory.
Доступ осуществляется через `rusqlite` внутри отдельного kernel-owned
persistence package.

По ADR-0217 data directory по умолчанию определяется из OS-standard per-user
local application-data location. Обязательного `bootstrap.toml` нет;
единственный override первой версии — explicit `--data-dir`, выбирающий
отдельный Макошь instance без scanning, merge или silent fallback.

Store:

- открывается непосредственно Kernel process, без PgBouncer или network;
- имеет одного writer owner;
- не публикует SQL, file handle или connection как capability;
- недоступен modules, clients, workflows, Scheduler и platform services;
- не используется для business/domain/provider state;
- доступен в `bootstrap`/`recovery_only` до запуска PostgreSQL, PgBouncer,
  NATS, Vault, Telemetry Collector и любых module runtimes;
- не синхронизируется через cloud/network filesystem.

Термин `Kernel Control Store` означает logical port и его private SQLite
adapter. Он не означает generic storage abstraction для modules.

### Жёсткий boot-инвариант

Для запуска Kernel process и перехода в минимальный local `recovery_only` не
требуются:

- PostgreSQL или PgBouncer;
- NATS/JetStream;
- Vault, master key, credential lease или provider secret;
- Blob service;
- Telemetry Collector;
- Scheduler;
- любой domain, integration, workflow или AI runtime;
- сеть или внешний server.

Единственные boot prerequisites — исполняемый Kernel, OS primitives, compiled
safe defaults и безопасная private runtime/data boundary, выбранная через
OS-standard path либо explicit `--data-dir`. После этого Kernel пытается
открыть Control Store. Если SQLite unavailable/corrupt, процесс всё равно
поднимает урезанный bootstrap recovery surface, достаточный для sanitized
diagnostics и online status/validate/export. Restore/reset доступны только
offline по ADR-0218. Business routing, module
approval changes и запуск managed data plane остаются заблокированы до
восстановления trustworthy Control Store.

Vault является отдельным managed process ADR-0223 и одной из supervised
capabilities после `recovery_only`, а не условием запуска Kernel. После
открытия `module_control_plane_v1` и `vault_v1` недоступный Vault не мешает
показывать health, registrations и desired topology, но Kernel не выдаёт
credential leases и не запускает операции, которым нужны secrets. В текущем
slice этих surfaces ещё нет.

### Разделение packages

Dependency inversion и compile isolation фиксируются отдельными packages:

```text
backend/src/kernel/control_store/contract/
    makosh-kernel-control-store
    typed records, commands, errors и KernelControlStore port

backend/src/kernel/control_store/sqlite/
    makosh-kernel-control-store-sqlite
    rusqlite, schema, migrations, integrity и backup adapter

backend/src/kernel/
    makosh-kernel
    boot/recovery state machine и composition
```

`makosh-kernel-control-store` не зависит от SQLite, SQL clients, Tokio runtime,
Kernel executable или module packages. SQLite adapter зависит от этого port.
`makosh-kernel` runtime композит port и adapter, но остальные modules не видят
ни один из них.

`rusqlite` разрешён только package с surface `persistence`. Он запрещён в
Kernel runtime, domain/integration implementation, workflows, engines и
contracts тем же executable rule, что и другие SQL clients.

### Единственный writer actor

`rusqlite` имеет synchronous API. Connection принадлежит одной выделенной
blocking thread/actor, а не Tokio worker и не произвольным concurrent callers.

Kernel обращается к actor только через bounded typed requests:

- facade `ControlStoreHandle` принимает узкие typed ports health/recovery,
  owner identity, module registry, settings registry и runtime trust;
- queue bounded ровно 64 requests;
- normal deadline равен 2 секундам, maintenance deadline — 30 секундам;
- backpressure/termination возвращают `QueueFull`, `DeadlineExceeded` или
  `ActorStopped`;
- mutation выполняется короткой явной transaction;
- long-running integrity/backup operation не блокирует supervisor loop;
- raw SQL и SQLite row types не пересекают adapter boundary.

Read operations также идут через actor и составные snapshots читаются в одной
transaction. В частности, `ModuleGrantSnapshot` возвращает registration и
соответствующий grant set из одного read snapshot; capability authorization не
склеивает результаты двух разных actor requests. Online
registry/settings/runtime methods не открывают независимые connections;
trustworthy online recovery export использует maintenance request того же
boot-created actor. Отдельный offline adapter допустим только под exclusive
instance lock для restore/reset и для recovery export, когда trustworthy online
actor отсутствует.

### Что хранится

Control Store хранит только technical desired/control state Kernel:

- store schema/version и instance metadata;
- installation identity/initialization state, logical owner record и identity
  epoch;
- device ID, public key, capabilities, status и revocation revision;
- `ModuleRegistrationId`, declared descriptor hash/metadata и registration
  state;
- exact `ModuleDescriptorV1` и settings schema artifact identities, revisions,
  sizes/digests и approval status ADR-0221/ADR-0222;
- external registration public key, но не private key;
- approved capability scopes и их revision history;
- current `grant_epoch` и fencing metadata;
- выбранный lifecycle mode `managed`/`external`;
- sanitized managed launch specification, origin, SHA-256 digest, size, bounded
  file identity, binding revision и restart policy ADR-0219;
- reference на signed bundled distribution entry либо owner-approved digest
  pin; bundled trust root при этом остаётся immutable release input;
- desired ownership/topology profile PostgreSQL, PgBouncer, NATS, Telemetry
  Collector и module runtimes;
- desired Storage Control lifecycle/artifact binding, target `StorageBundleV1`
  identities/digests, reserved monotonic `storage_generation` и bounded
  last-acknowledged hint; issued `StorageBindingV1` и applied
  schema/bundle/grant state остаются outside Control Store, а canonical actual
  state находится только в PostgreSQL storage ledger ADR-0224;
- technical resource budgets, listener profile references и lifecycle policy;
- opaque settings target identities, typed desired values/revisions и apply
  state;
- last-acknowledged effective settings values/revision и process generation
  только как diagnostics/reconciliation hint;
- bounded sanitized control mutation audit, необходимый для recovery;
- last-known observed lifecycle metadata только как diagnostics hint.

Persisted actual state никогда не считается истиной после restart. Kernel
заново проверяет process handles, endpoints, health, storage identity и
capabilities. Desired state можно восстановить из store; actual state всегда
получается reconciliation. Для storage Kernel принимает только typed
attestation Storage Control и не выполняет SQL/schema introspection.

### Что хранить запрещено

В Control Store запрещены:

- domain entities и business truth;
- Communications evidence, messages, contacts, tasks, calendar или documents;
- provider operational records, cursors, mail bodies и attachments;
- JobSchedule, JobExecution, workflow state и outbox/inbox messages;
- NATS events, telemetry payload или product audit;
- business preferences, user automation rules и arbitrary module state,
  замаскированные под settings;
- credentials, tokens, passwords, cookies, Vault master/unlock material;
- PostgreSQL/PgBouncer passwords, SCRAM verifiers, credential record IDs,
  runtime principal secrets и admin/bootstrap material;
- Vault database/anchor, key slots, secret IDs, credential bindings и leases;
- provider sessions, owner/device private keys и external registration private
  keys;
- release/update private keys, executable bytes, downloaded artifacts и
  arbitrary installer commands;
- raw `StorageBundleV1`, migration SQL/AST, applied schema truth, PgBouncer auth
  files, issued `StorageBindingV1` и active database sessions;
- blobs, private content, prompts, embeddings и search indexes.

Typed module configuration values/revisions принадлежат Kernel Settings Registry
ADR-0222; module владеет schema meaning, semantic validation и применением, но
не authoritative persistence. Scheduler state остаётся в его PostgreSQL owner
boundary. Business/operational state остаётся у соответствующего owner,
credentials — в Vault. Telemetry Collector использует собственный independent
bounded local store по ADR-0210 и не читает Kernel SQLite.

### Atomic invariants

Одна transaction обязана атомарно сохранять связанные изменения:

- approval state + approved scopes + grant revision + new epoch;
- suspend/revoke + epoch increment + desired session state;
- lifecycle mode transition + drained predecessor marker + new generation;
- managed runtime specification + verified binding + revision + new epoch;
- infrastructure desired topology + revision;
- settings schema activation + materialized defaults + desired revision;
- settings desired values + snapshot hash + apply intent/status;
- migration version + все schema/data changes этой migration.

Внешний side effect не считается частью SQLite transaction. Сначала durable
desired state/fencing, затем service-native revoke/start/stop, затем actual
state reconciliation. Crash между этими шагами приводит к повторной
idempotent reconciliation, а не к откату authorization epoch.

Monotonic epoch никогда не уменьшается при restore, retry или rollback code.
Restore более старой backup требует явного recovery procedure, которое
создаёт новую generation/epoch до выдачи capabilities.

### SQLite durability profile

Первая реализация использует консервативный low-write profile:

- database и parent directory находятся только на local filesystem;
- directory permission `0700`, database/journal/backup files `0600`;
- rollback journal `journal_mode=DELETE`;
- `synchronous=FULL` минимум; более строгий platform mode допускается после
  conformance tests;
- `foreign_keys=ON`;
- `trusted_schema=OFF`;
- extension loading запрещён;
- transactions короткие, explicit и bounded;
- WAL не включается без доказанной concurrent-read необходимости и отдельного
  crash/checkpoint suite.

WAL не нужен для первого single-writer actor. Он добавил бы persistent
`-wal`/`-shm` files и checkpoint lifecycle без реального выигрыша для десятков
или сотен небольших control records.

### Schema и migrations

SQLite schema принадлежит `makosh-kernel-control-store-sqlite`, а не общей
`makosh-schema` и не PostgreSQL migrations.

Правила:

- schema имеет отдельную monotonic version;
- migrations embedded в adapter и выполняются по порядку;
- каждый `MigrationStep` выполняет DDL/data change и обновление
  `schema_version` в одной explicit `BEGIN IMMEDIATE` transaction;
- assertion после step проверяет base metadata и каждый накопленный schema
  feature до объявленной версии;
- неизвестная более новая schema version fails closed;
- объявленная текущая версия с отсутствующим ранним table/column fails closed;
- destructive automatic downgrade запрещён;
- migration не импортирует legacy PostgreSQL или business data;
- schema constraints проверяют states, revisions, uniqueness и references;
- executable suite останавливает update границу каждого source version
  `1…14`, доказывает rollback DDL/version и затем migration до v15; отдельно
  проверяются newer schema и partially incompatible v15.

Control-store migration запускается до managed infrastructure. Ошибка оставляет
Kernel в bounded recovery surface с blocker; она не запускает PostgreSQL/NATS
или modules по непроверенному state.

### Integrity, backup и recovery

При startup adapter выполняет schema/header validation и `quick_check` до
выдачи capabilities. Полный `integrity_check` доступен как explicit maintenance
operation и не выполняется на hot path каждого старта.

Kernel поддерживает versioned last-known-good backup/export:

- создаётся SQLite backup mechanism, а не копированием открытого database file;
- temporary file и parent directory синхронизируются до atomic rename;
- backup содержит store generation/schema version и checksum metadata;
- backup хранится в том же private trust boundary;
- backup не содержит secrets, потому что primary store их не содержит;
- online backup/export требует private local boundary; destructive
  restore/reset выполняются только offline по ADR-0218.

При corruption Kernel:

1. не удаляет и не пересоздаёт store автоматически;
2. не запускает modules или managed data plane по guessed defaults;
3. поднимает минимальный local recovery surface из bootstrap boundary;
4. показывает sanitized `control_store_corrupt` blocker;
5. online разрешает только status/validate/export;
6. требует остановить Kernel для explicit offline restore/reset под exclusive
   lock;
7. до замены БД резервирует fixed binary `0600`
   `.makosh-recovery-fence-v1` через temp file, file `fsync`, atomic rename и
   directory `fsync`;
8. вычисляет generation/identity/grant high-water как maximum внешнего fence,
   trustworthy current store и backup плюс один;
9. в staged transaction suspends registrations, назначает новый grant epoch и
   удаляет external attestations, active sessions и managed launch records;
10. после restore/reset mismatch reservation/store оставляет Kernel только в
    `recovery_only`; retry создаёт следующую generation;
11. только затем повторяет full reconciliation.

Если нельзя безопасно открыть recovery surface или установить trustworthy
generation, Kernel переходит в `fatal` по ADR-0206. PostgreSQL не используется
как скрытый fallback для Registry.

### Backup scope

Kernel Control Store включается в technical recovery backup Макошь вместе с
его schema/generation metadata. Он не включается в domain export как business
database и не заменяет отдельные backup policies PostgreSQL, Vault, provider
sessions, blobs, JetStream или Telemetry.

Restore components выполняется явно и в определённом порядке. Несогласованный
restore только SQLite без fencing всех issued capabilities запрещён.

### Безопасность

- файл и backup доступны только владельцу Макошь;
- SQLite URI, path и error не содержат secrets/private content;
- SQL parameters bound, а не interpolated;
- extension loading и arbitrary ATTACH запрещены;
- modules не получают filesystem path или query surface;
- settings показывает только authorized typed Registry data через Core Gateway,
  а не SQL rows; raw values/diffs не попадают в logs, health, telemetry или SSE;
- corruption diagnostics не дампит raw pages/records в logs;
- SQLCipher/Vault-derived encryption key не вводятся: store не имеет права
  хранить secret material, а зависимость открытия store от Vault создала бы
  boot cycle; host filesystem encryption остаётся defense-in-depth.

## Почему `rusqlite`, а не другие варианты

SQLite предоставляет serializable ACID transactions и automatic recovery
после interrupted transaction. `rusqlite` является узким Rust binding и с
`bundled` feature позволяет Kernel контролировать shipped SQLite version.

[`redb`](https://github.com/cberner/redb) является стабильным pure-Rust ACID
KV-store и остаётся допустимым fallback, если C dependency станет жёстким
ограничением. Для текущих relational constraints, revision history,
diagnostics и migrations он потребовал бы собственных indexes/schema tools.

[`fjall`](https://github.com/fjall-rs/fjall) предоставляет LSM key-value store,
transactions, compression и background maintenance. Это лишняя operational
сложность для малого low-write control state.

Atomic JSON/CBOR остаётся допустимым только для минимальной bootstrap
configuration и human-readable export. Он не является authoritative store
registrations/grants.

Primary sources:

- [SQLite transactional guarantees](https://sqlite.org/transactional.html);
- [SQLite atomic commit](https://sqlite.org/atomiccommit.html);
- [SQLite WAL trade-offs](https://sqlite.org/wal.html);
- [`rusqlite`](https://github.com/rusqlite/rusqlite).

## Отклонённые варианты

### Хранить control state только в PostgreSQL

Отклонено: создаёт circular boot dependency и уничтожает `recovery_only` при
отказе PostgreSQL/PgBouncer.

### Использовать PostgreSQL как primary и SQLite как нестрогий cache

Отклонено: два competing sources of truth для grants/topology создают
split-brain именно в recovery scenario.

### Общая SQLite database для Kernel и modules

Отклонено: нарушает owner roles/grants, превращает file access в обход
capability router и создаёт общий failure/security domain.

### JSON/CBOR как authoritative registry

Отклонено: atomic rewrite одного файла не даёт готовых relational constraints,
transactional migrations, concurrent read discipline и проверенного recovery
protocol для нескольких связанных records.

### SQLx и connection pool

Отклонено для этого adapter: async pool и широкий compile graph не дают пользы
single-writer local actor. PostgreSQL adapters могут выбрать другой client в
своих persistence packages независимо.

### redb

Отклонено как default, но оставлено fallback: pure Rust привлекателен, однако
schema/index/migration/inspection слой пришлось бы создавать самим.

### Fjall/RocksDB-style LSM

Отклонено: compaction, caches, compression и write-heavy optimization не
соответствуют объёму и critical recovery role store.

### WAL с первого дня

Отклонено: single-writer actor не требует concurrent reader/writer profile, а
checkpoint и дополнительные files расширяют recovery surface.

## Проверка решения

До изменения `Состояние реализации` обязательны tests:

- запуск без bootstrap configuration file выбирает OS-standard data directory;
- explicit `--data-dir` выбирает ровно один instance и не смешивается с
  default store;
- Макошь-specific environment не переопределяет data directory;
- invalid explicit data directory не вызывает silent fallback;
- Kernel process стартует и достигает `recovery_only` без
  PostgreSQL/PgBouncer/NATS/Vault/Blob/Scheduler/modules;
- missing/corrupt/incompatible SQLite оставляет restricted local recovery
  доступным;
- online недоверенного store допускает только status/validate/export;
- restore/reset требуют stopped Kernel, explicit `--data-dir`, exclusive lock
  и confirmation;
- при недоверенном store managed services, modules и data plane не запускаются;
- locked или unavailable Vault не препятствует чтению Registry и local
  recovery operations;
- Control Store не требует Vault-derived key или credential lease;
- directory/file permissions fail closed;
- SQLite находится только на local filesystem policy path;
- approval/scopes/revision/epoch сохраняются атомарно;
- settings schema/materialized defaults и первая desired revision сохраняются
  атомарно;
- settings mutation с expected revision сохраняет desired snapshot hash и apply
  intent одной transaction;
- last-acknowledged effective revision после restart считается только hint и
  подтверждается текущей process generation заново;
- owner-pinned digest binding сохраняется в одной transaction с
  lifecycle transition, revision и epoch;
- restart не repin-ит изменившиеся executable bytes автоматически;
- crash в каждой точке commit оставляет old либо new complete state;
- stale epoch не возрождается после restart;
- actual runtime state всегда re-probe/reconcile, а не доверяется persisted
  hint;
- suspend/revoke переживает Kernel crash до downstream revoke и после него;
- migration crash восстанавливает old либо complete new schema;
- unknown newer schema остаётся recovery blocker;
- `quick_check` failure не вызывает silent reset;
- last-known-good backup создаётся через safe backup path;
- corrupt primary требует explicit restore/reset;
- restore повышает generation/epochs до выдачи capabilities;
- module/client не может открыть store или выполнить raw query;
- initial owner/device enrollment атомарно и разрешено только pristine
  installation через inherited FD;
- private owner/device keys отсутствуют в schema, backup и fixtures;
- device revoke атомарно повышает identity epoch;
- forbidden business/private/secret fields отсутствуют в schema и fixtures;
- JobSchedule, JobExecution, cursors, checkpoints и provider state не попадают
  в settings/control-store records;
- `rusqlite` dependency разрешена только kernel persistence package;
- Kernel actor backpressure не блокирует supervisor loop без bounded timeout;
- PostgreSQL restart/restore не меняет Kernel control-store truth;
- clean-room build доказывает, что owner modules не компилируют `rusqlite` или
  Kernel control-store packages.

## Последствия

Положительные:

- Kernel boot/recovery не зависит от PostgreSQL и NATS;
- grants, epochs, desired topology и typed settings revisions имеют одну
  transactional truth;
- SQLite complexity изолирована от modules и Kernel logic;
- owner modules не пересобирают и не линкуют SQLite adapter;
- recovery можно тестировать crash-injection без запуска всего data plane.

Стоимость:

- Макошь имеет отдельный маленький technical database помимо PostgreSQL;
- нужно поддерживать отдельные schema migrations, backup и corruption tests;
- bundled SQLite добавляет C compilation только в Kernel persistence graph;
- synchronous adapter требует отдельный actor и bounded queue;
- path/lock/recovery boundary требует platform-specific tests; ADR-0217
  запрещает обязательный bootstrap file и оставляет только OS-standard path и
  explicit `--data-dir`.
