# ADR-0206: Конституция Kernel и автомат запуска и восстановления

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Связанные решения:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

Kernel должен переживать отказ PostgreSQL, PgBouncer, NATS, vault и отдельных
module runtimes настолько, чтобы сохранить безопасный control/recovery surface.
Одновременно он не должен постепенно превратиться в god service, который
содержит бизнес-логику, междоменную координацию, provider mapping или общий
доступ к данным всех модулей.

Для clean-room реализации нужна явная конституция Kernel: исчерпывающий список
его обязанностей, запрещённые причины изменения, состояния процесса и правила
доступности операций при частичном отказе. Без этого понятия `core`, `runtime`
и `recovery mode` останутся соглашениями, которые невозможно проверить.

## Решение

### Kernel является техническим control plane

Kernel владеет только следующими обязанностями:

1. boot/recovery state machine всего локального экземпляра Макошь;
2. supervisor managed infrastructure и independently restartable module
   runtimes;
3. module registry, проверка `ModuleDescriptorV1`, protocol, signed distribution
   inventory и exact bytes каждого managed launch;
4. Settings Registry: проверка schema catalog, durable desired/effective typed
   configuration revisions и supervised application без интерпретации module
   semantics;
5. построение и проверка startup dependency graph;
6. корневая runtime identity/capability authority и выдача scoped runtime
   capabilities через отдельные platform boundaries;
7. Core Gateway, client session bootstrap и transport-level authorization;
8. маршрутизация synchronous public contracts и reconciliation durable catalog
   по ADR-0220 без интерпретации business payload или нахождения Kernel на
   normal NATS payload path;
9. sanitized health, diagnostics, lifecycle audit и recovery surface;
10. infrastructure supervision registry, а также техническая конфигурация
    процесса, listeners и resource budgets;
11. Event Hub: catalog event contracts, проверка publishers/subscribers,
    reconciliation NATS streams/consumers/permissions и delivery health без
    чтения business payload;
12. Telemetry Hub control surface: telemetry identity, schema, redaction,
    quotas, health и authorized diagnostics при отдельном supervised Collector
    process.

Storage authorization и lifecycle routing входят в пункты 2, 5 и 6, но SQL
introspection не входит в Kernel. Cluster/schema/extension/role/grant
reconciliation выполняет отдельный Storage Control runtime и возвращает typed
attestation ADR-0224.

Это закрытый список. Новая ответственность Kernel требует изменения этого ADR
или нового superseding ADR.

### Kernel не владеет предметной логикой

В Kernel запрещены:

- domain entities, domain policies и domain state;
- provider protocol, provider mapping и neutral evidence mapping;
- cross-domain orchestration, workflows и business read composition;
- business schedulers и фоновые product jobs;
- AI, search, ranking, memory и context algorithms;
- module-owned SQL, tables, repositories и generic SQL proxy;
- PostgreSQL client, cluster/schema introspection, role/grant provisioning,
  migration execution и PgBouncer administration;
- NATS broker/data plane, Telemetry Collector, vault, blob или storage
  implementation как библиотека внутри Kernel;
- provider accounts, prompts, user automation rules, cursors, checkpoints и
  другое business/operational state, замаскированное под settings;
- private message/document/media content в health, diagnostics или lifecycle
  audit;
- download, unpack, install или automatic rollback executable code.

Kernel Settings Registry хранит только typed configuration control state
ADR-0222. Declaring module владеет stable IDs, constraints, defaults, смыслом,
semantic validation и применением своих settings; Kernel владеет schema catalog,
durable values/revisions и lifecycle orchestration, но не переносит module
business rules в core. Secrets, provider sessions и private content не являются
settings.

Vault, blobs, event data plane, Telemetry Collector и storage доступны только
через отдельные platform capability boundaries. Для module runtimes desired
lifecycle mode и topology принадлежат Module Registry registration/grant
approval state ADR-0215; для platform services supervision mode принадлежит
отдельному infrastructure registry state ADR-0203. Оба хранятся в доверенном
Control Store, но не в Settings Registry. Signed distribution manifest
ADR-0219 лишь ограничивает доступные проверенные bundled bytes. Kernel управляет
их lifecycle и capability routing, Event Hub контролирует event topology, а
Telemetry Hub — telemetry policy; Kernel не становится владельцем business
payload или implementation managed services.

Vault является отдельным verified managed process владельца `platform/vault`
по ADR-0223. Kernel supervisor управляет его lifecycle и маршрутизирует только
versioned ciphertext frames с authorization/fencing context. Kernel не линкует
`makosh-vault-runtime`, SQLCipher, crypto или file-key adapters, не получает
credential plaintext/keys и не хранит Vault anchor, key slots, secret bindings
или leases в Control Store. Vault failure блокирует только capabilities с
явной credential dependency; остальной recovery/control plane продолжает
работать.

Storage Control является отдельным verified managed process владельца
`platform/storage` по ADR-0224. Kernel supervises PostgreSQL, PgBouncer и этот
runtime, маршрутизирует opaque control messages и сверяет typed readiness, но
не линкует storage implementation packages и не получает bootstrap/runtime
database credential plaintext. Modules получают `StorageBindingV1` только для
current storage/runtime/grant/role generations, а secret доставляет Vault.

### Автомат состояний Kernel

Kernel использует один явный автомат:

```text
cold_start
    ↓
bootstrap
    ↓
recovery_only
    ↓
infrastructure_starting
    ↓
modules_starting
    ↓
ready

modules_starting / ready
    ↔ degraded

recovery_only / infrastructure_starting / modules_starting / ready / degraded
    → quiescing → draining → stopped

любое состояние
    → fatal
```

- `cold_start` — процесс создан, доверенная runtime identity ещё не
  установлена.
- `bootstrap` — compiled defaults выбирают OS-standard private data/runtime
  directories либо единственный explicit `--data-dir`; затем проверяются
  single-instance lock, executable/distribution inventory и локальный recovery
  endpoint. Обязательного configuration file нет.
- `recovery_only` — supervisor, локальный Gateway recovery surface и
  diagnostics доступны без PostgreSQL, PgBouncer, NATS, vault и модулей.
  При trustworthy Control Store могут быть доступны разрешённые registry,
  topology и infrastructure recovery actions. При unavailable/untrusted store
  online остаются только sanitized status и store validate/export;
  restore/reset являются offline operations ADR-0218. Telemetry использует
  Collector либо bounded emergency log.
- `infrastructure_starting` — запускаются и проверяются declared managed
  platform services.
- `modules_starting` — capability dependency graph из verified
  `ModuleDescriptorV1` проверен, runtime запускаются в порядке обязательных
  capabilities.
- `ready` — все обязательные capabilities distribution готовы.
- `degraded` — Kernel и часть capabilities работают, но один или несколько
  необязательных runtime либо scoped capabilities недоступны.
- `quiescing` — новые mutations больше не принимаются.
- `draining` — in-flight work завершается в bounded deadlines; declared
  durable checkpoints выполняются только для runtime, который их поддерживает
  и требует.
- `stopped` — managed children остановлены в установленном порядке.
- `fatal` — Kernel не может безопасно поддерживать даже доверенный recovery
  control plane.

`fatal` не используется для обычного отказа domain, workflow, integration,
PostgreSQL, PgBouncer, NATS или vault. Такие отказы переводят затронутые
capabilities в `blocked`, Kernel — в `degraded` или `recovery_only` и сохраняют
возможность диагностики и явного восстановления.

### Доступность операций по состояниям

Таблица ниже является целевой конституцией после открытия соответствующих
phase gates ADR-0225. В текущем `kernel_recovery_only_v1` даже при trustworthy
Control Store online доступны только status, validate, export и lifecycle
shutdown; settings, infrastructure, Vault и whole-instance backup actions ещё
не авторизованы.

| Состояние | Разрешено | Запрещено |
|---|---|---|
| `bootstrap` | только process-local liveness | client business и recovery operations |
| `recovery_only`, Control Store trustworthy | sanitized health/diagnostics, status, authorized retry/start/stop managed service, typed settings catalog/correction/retry ADR-0222, vault unlock/recovery entrypoints, authorized backup validation/restore entrypoints, shutdown | business queries, commands и mutations |
| `recovery_only`, Control Store unavailable/untrusted | online sanitized status и Control Store validate/export | online restore/reset, infrastructure actions, registry/grant/device/settings mutations, topology actions и весь business data plane |
| `infrastructure_starting` | recovery operations и startup progress | business operations |
| `modules_starting` | recovery operations; запросы только к уже ready runtime, если их contract это разрешает | operation с любой unavailable required capability |
| `ready` | обычные authorized operations | всё, что запрещено contract/capability policy |
| `degraded` | операции healthy runtime с полным набором требуемых capabilities | operation, зависящая от blocked/unavailable capability |
| `quiescing` / `draining` | health, progress, bounded reads, завершение уже принятых работ | новые mutations и новые background claims |
| `stopped` / `fatal` | внешний watchdog может получить только process status/exit evidence | application operations |

Gateway проверяет состояние и required capability set каждого метода до
маршрутизации. Ошибка одного модуля не блокирует несвязанный healthy module.
Kernel не подменяет недоступный runtime другим implementation, transport или
topology.

Recovery surface по умолчанию доступен только через доверенный local channel.
Remote recovery listener не включается автоматически. Операции, способные
изменить data или credentials, требуют отдельной owner authorization policy и
никогда не запускаются из одного health probe.

### Dependency graph и порядок запуска

Каждый `ModuleDescriptorV1` ADR-0221 объявляет:

- предоставляемые capabilities;
- обязательные capabilities;
- необязательные capabilities;
- readiness condition;
- lifecycle protocol version;
- публикуемые и потребляемые event contracts;
- required/optional subscriptions и их resource budgets;
- telemetry signal capabilities и quotas;
- exact settings schema reference и поддерживаемые validation/apply lifecycle,
  когда module имеет configuration ADR-0222.

Kernel строит граф до запуска business runtime. Цикл startup dependencies,
неизвестная обязательная capability, неоднозначный provider capability или
несовместимая protocol version блокируют затронутые runtime до provider call
или domain mutation.

Runtime запускается топологически; независимые ветви могут стартовать
параллельно. Event subscription сама по себе не является startup dependency и
не создаёт скрытый цикл. Поведение при недоступности:

- обязательная capability модуля недоступна — модуль остаётся `blocked`;
- необязательная capability недоступна — модуль может стать `degraded` и обязан
  явно отключить связанную функцию;
- обязательная platform capability всей distribution недоступна — Kernel
  остаётся `recovery_only`;
- необязательный module runtime недоступен — Kernel может обслуживать остальные
  healthy capabilities в `degraded`.

После восстановления capability зависимые runtime возобновляются только через
явный lifecycle transition и повторную readiness-проверку. Silent fallback и
обход dependency graph запрещены.

### Граница boot inputs и будущей конфигурации

До открытия Control Store Kernel использует только:

- compiled safe defaults и stable application identity;
- OS-standard private data/runtime locations;
- единственный explicit non-secret `--data-dir` override;
- immutable signed bundled distribution inventory.

Обязательного configuration file, Макошь-specific environment overlay,
directory scanning или silent fallback нет. Полная граница определена
ADR-0217.

Module lifecycle mode `managed`/`external` не является setting: он принадлежит
Module Registry registration/grant approval state ADR-0215. Infrastructure
supervision mode также хранится отдельной typed registry model, а не в
Settings Registry. Оба mode не являются pre-store bootstrap inputs и проходят
через trustworthy Control Store.

Endpoints, listeners, lifecycle/restart budgets, event topology, telemetry
policy и module settings также не являются pre-store bootstrap inputs. Только
configuration fields, объявленные через `SettingsSchemaV1`, получают
desired/effective revisions и supervised apply lifecycle ADR-0222; соседние
registry, grant и topology models не становятся settings из-за общего
физического Control Store.

Каждый module владеет смыслом и применением собственной configuration schema,
но не её authoritative persistence. Kernel сохраняет exact schema binding и
typed desired/effective values, выполняет structural validation и orchestrates
apply/restart, не интерпретируя business semantics и не предоставляя generic KV
store.

Secrets, credential leases и secret bootstrap material не передаются через
argv, environment, logs или diagnostics. Явный non-secret `--data-dir`
разрешён ADR-0217. Локальная module identity и capability authorization
определены ADR-0215; owner/device identity и offline recovery authorization —
ADR-0218. Provider-account и agent identities определяются отдельными ADR, не
расширяя предметную роль Kernel.

### Критерии глобального состояния

- `ready` определяется distribution manifest и readiness обязательных
  capabilities, а не числом запущенных процессов.
- Module `ready` требует `ModuleDescriptorV1`/handshake, current effective
  settings revision и подтверждённую Event Hub readiness обязательных
  subscriptions. Capability с storage request дополнительно требует exact
  applied `StorageBundleV1` и current-generation `StorageBindingV1`.
- Один unhealthy probe не меняет state без классификации и bounded policy.
- Kernel возвращается из `degraded` или `recovery_only` только после повторной
  проверки полного набора обязательных capabilities.
- Глобальный `fatal` допускается только при невозможности установить
  безопасный single-instance lock, защитить local recovery endpoint либо
  продолжать supervisor/recovery loop. Недоступный или недоверенный Control
  Store оставляет Kernel в restricted `recovery_only`, а не переводит его в
  `fatal`. Publisher signature не требуется для external registration,
  но exact-byte verification обязательна перед любым managed launch по
  ADR-0219.

### Решения, остающиеся отдельными

Настоящий ADR фиксирует ownership и state machine, но не выбирает:

- provider-account и agent identity model;
- coordinated whole-instance backup retention, encryption, media format и
  cross-component restore authorization; PostgreSQL lifecycle/fencing
  определены ADR-0224;
- окончательную topology Android Kernel.

Эти решения требуют отдельных ADR. Их implementations не должны добавляться в
Kernel до фиксации соответствующего contract.

## Отклонённые варианты

### Kernel как приложение со всеми domain services

Отклонено: создаёт общий failure domain, скрытые зависимости и невозможность
независимо перезапускать модули.

### Kernel, который запускается только после PostgreSQL и NATS

Отклонено: при отказе infrastructure исчезает сам control/recovery surface.

### Глобальный shutdown при падении любого модуля

Отклонено: противоречит изоляции runtime и делает необязательную capability
точкой отказа всей системы.

### Best-effort запуск при цикле или неизвестной capability

Отклонено: порядок становится недетерминированным, а ошибка проявляется уже во
время business operation.

### Business read composition внутри Gateway

Отклонено: transport adapter начал бы знать domain semantics и стал бы новым
монолитом.

## Последствия

Положительные:

- Kernel имеет небольшой, проверяемый набор причин изменения;
- recovery доступен до запуска canonical storage и data plane;
- ошибка одного модуля не останавливает несвязанные capabilities;
- startup order и partial availability становятся детерминированными;
- Gateway не превращается в business orchestration layer.

Отрицательные:

- каждый public method должен объявлять required capabilities;
- runtime descriptors, settings schemas и distribution inventory становятся
  критическими contracts;
- recovery surface требует отдельной security и UX проверки;
- process-level state-machine tests обязательны с первой реализации.

## Проверка решения

До признания реализации завершённой должны существовать executable tests:

- Kernel запускается без bootstrap configuration file и выбирает только
  OS-standard data directory либо explicit `--data-dir`;
- Kernel достигает `recovery_only` при остановленных PostgreSQL, PgBouncer,
  NATS, vault и всех module runtimes;
- unavailable/untrusted Control Store оставляет restricted local recovery, но
  блокирует infrastructure actions, modules и business data plane;
- untrusted-store online surface допускает только status/validate/export, а
  restore/reset требуют stopped Kernel и exclusive offline lock;
- invalid explicit data directory не вызывает поиск или silent fallback на
  другой store;
- unavailable mandatory platform capability не допускает `ready`;
- crash optional module переводит Kernel в `degraded`, но healthy module
  продолжает обслуживать разрешённые operations;
- crash mandatory capability блокирует только методы с этой dependency либо
  переводит Kernel в `recovery_only` согласно distribution manifest;
- invalid signed distribution manifest не допускает managed data plane;
- managed executable проверяется перед каждым launch/restart;
- integrity failure не вызывает automatic rollback или fallback;
- required/optional capability behavior соответствует `ModuleDescriptorV1` и
  distribution policy;
- startup dependency cycle и unknown capability fail closed до запуска модуля;
- unknown/colliding settings schema и неполный required snapshot блокируют
  только затронутую capability до provider/domain operation;
- module readiness требует acknowledgement effective settings revision текущей
  process generation;
- failed settings apply не вызывает automatic rollback и сохраняет explicit
  desired revision для correction/revert;
- independent modules запускаются параллельно после готовности dependencies;
- Gateway отклоняет mutation в `quiescing` и `draining`;
- восстановление capability требует повторного handshake/readiness;
- `fatal` не используется как реакция на обычный module/infrastructure crash;
- Kernel не импортирует domain, provider, workflow, AI, search, storage, vault
  или blob implementations;
- Kernel storage readiness использует typed Storage Control attestation и не
  открывает SQL connection для schema/extension/role/grant introspection;
- recovery health, diagnostics и audit не содержат secrets или private content;
- после unclean exit stale private IPC path удаляется только если это Unix
  socket текущего owner; symlink и regular file fail closed;
- Event Hub остаётся вне normal event data path и не читает payload;
- required subscription readiness подтверждается через Event Hub;
- Telemetry Collector работает без PostgreSQL/NATS, а его crash переводит
  Kernel в `degraded`, не останавливая modules;
- remote recovery listener не включается автоматически;
- новый Kernel owner или responsibility ломает executable constitution guard.
