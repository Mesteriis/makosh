# ADR-0221: ModuleDescriptorV1 и capability-level lifecycle

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Частично реализовано: `makosh-runtime-protocol` содержит
V1 descriptor/lifecycle/health wire types, bounded descriptor decoder и
bounded `SettingsSchemaV1` structural validator. Development Control Store
сохраняет exact descriptor digest только как registry binding. Registration
handshake, schema admission/persistence, runtime activation и full conformance
suite остаются закрытыми до `module_control_plane_v1`.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md).

Уточняет ADR-0206, ADR-0212, ADR-0215 и ADR-0219.

Уточняется:

- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).

## Контекст

ADR-0215 разрешает любому локальному process пройти недоверенную регистрацию,
а ADR-0219 требует проверять exact bytes перед каждым managed launch. Этого
недостаточно для построения runtime graph: Kernel должен точно знать, какие
capabilities, contracts, jobs и platform resources заявляет запущенный module,
какая часть модуля обязательна для readiness и что изменилось после update.

Одно слово `manifest` использовалось сразу для двух разных задач:

- доказать, какие executable bytes установлены и разрешены к запуску;
- описать поведение уже запущенного module runtime.

Смешение этих задач создаёт циклическое доверие. Self-declared capability не
доказывает executable integrity, а подписанный список файлов не должен
дублировать evolving business/runtime contract. Одобрение целого модуля также
слишком грубое: отказ отправки Telegram не обязан отключать чтение истории, а
недоступный Vault не обязан останавливать capability, которой secrets не нужны.

Нужен один bounded protocol, одинаковый для domain, integration, workflow,
engine и independently supervised platform service, но не связывающий их
исходный код и owners.

## Решение

### Четыре разных объекта

Макошь различает четыре объекта и не использует их как взаимозаменяемые:

1. `DistributionManifestV1` ADR-0219 доказывает состав установленной release,
   exact executable bytes и digest descriptor artifact.
2. `ModuleDescriptorV1` заявляет, что конкретная версия module умеет и какие
   capabilities просит у платформы.
3. `GrantSet` ADR-0215 фиксирует, что Kernel и владелец реально разрешили этой
   registration/session.
4. `RuntimeState` фиксирует observed lifecycle, readiness и effective
   revisions текущего process generation.

`ModuleDescriptorV1` является декларацией, а не authority. Он не выдаёт права,
не доказывает identity, не является настройками и не заменяет health/readiness.

### Владелец protocol package

Точный wire contract принадлежит единственному Cargo package:

```text
backend/src/platform/runtime_protocol/
    makosh-runtime-protocol
    metadata: platform:runtime_protocol:contract
```

Package содержит только Protobuf wire types, bounded validation primitives и
version semantics для:

- registration `Hello`/`Describe`;
- `ModuleDescriptorV1` и capability declarations;
- lifecycle/control requests и responses;
- health/readiness wire state;
- settings schema/snapshot references ADR-0222.

Он не содержит Kernel implementation, process spawning, NATS client, SQL,
SQLite, Vault implementation, provider SDK, owner-specific contract payloads,
JSON negotiation или filesystem operations. Kernel и все module runtimes могут
зависеть от protocol package; module runtime не зависит от `makosh-kernel`.

### Wire artifact и digest

Первая версия использует только Protobuf binary. Descriptor передаётся как
exact bounded byte artifact; Kernel сохраняет SHA-256 полученных bytes до
decode и не вычисляет identity через повторную сериализацию.

Начальные ограничения:

- serialized descriptor — не более 256 KiB;
- не более 128 capabilities;
- не более 128 contract references на одну capability;
- не более 128 dependency references на одну capability;
- identifier — не более 128 ASCII bytes;
- display text — не более 4096 UTF-8 bytes, plain text без HTML/Markdown;
- duplicate IDs, unordered duplicate entries и unknown required enum values
  отклоняются fail closed.

Producer descriptor artifact обязан использовать canonical ordering по stable
ID. Kernel проверяет ordering и uniqueness. Hash относится к exact artifact
bytes, а не к `.proto` source, decoded object или generated Rust type.

Повтор тех же `ModuleRegistrationId`, `descriptor_major` и
`descriptor_revision` с тем же digest является idempotent. Та же identity и
revision с другими exact bytes/digest является `descriptor_revision_collision`,
переводит registration в `blocked_incompatible` и никогда не трактуется как
обычный update.

Descriptor не содержит собственного digest: self-referential hash не является
корректным contract. Digest хранится во внешнем distribution/approval binding
и в Kernel Registry.

### Module identity и kind

`ModuleDescriptorV1` содержит:

```text
descriptor_major = 1
descriptor_revision
module_id
owner_id
module_kind
module_version
build_id
runtime_protocol_range
capabilities[]
settings_schema_ref?
runtime_budget_request
```

Допустимые `module_kind` первой версии:

- `domain`;
- `integration`;
- `workflow`;
- `engine`;
- `platform`.

Один process registration описывает ровно одного logical owner. Он не может
объединить capabilities нескольких domains/integrations в один descriptor.
Несколько installations или runtime instances с одинаковым `module_id` имеют
разные `ModuleRegistrationId` и не наследуют identity/grants друг друга.

`module_id`, `owner_id`, version, build ID и kind являются consistency metadata.
Они не заменяют managed launch record или external proof-of-possession
ADR-0215. Kernel не авторизует process только по совпавшей строке.

`module_version` является bounded provenance string. Совместимость определяется
protocol ranges, contract major/revision/schema hash и explicit policy, а не
лексикографическим либо SemVer-сравнением этой строки.

### Capability является единицей управления

Каждая `CapabilityDescriptorV1` имеет минимум:

```text
capability_id
capability_revision
criticality = required | optional
provides
requests
dependencies
lifecycle_support
resource_budget_request
settings_definition_ids[]
```

Stable capability ID уникален внутри descriptor и не переиспользуется для
другого смысла. Capability является единицей:

- owner approval и GrantSet;
- startup dependency graph;
- readiness и health;
- resource budget;
- revoke/fencing;
- settings impact;
- degraded/blocked state.

Registration остаётся container identity и lifecycle boundary process. Это не
заставляет Kernel выдавать или отзывать все права process одним флагом.

`required` означает, что без capability сам module runtime не достигает
`ready`. `optional` может стать `degraded`, сохраняя независимые healthy
capabilities. Required/optional classification distribution-wide readiness
остаётся отдельным свойством signed distribution manifest и Kernel policy.

### Что capability предоставляет

`provides` содержит только typed references:

- query/request RPC service contracts;
- client-routable RPC contracts через Core Gateway;
- durable command handlers;
- durable event/observation/result/Ack publishers;
- durable consumers/subscriptions;
- `JobKind` descriptors ADR-0214;
- health/readiness signals.

Durable contract reference использует owner/name/major/revision/exact schema
SHA-256 ADR-0220. Module не объявляет и не выбирает NATS subject, stream,
consumer name или broker credential. Event Hub строит broker topology из
проверенного catalog и effective grants.

Client-routable contract не позволяет module открыть listener клиенту или
поставить remote frontend code. Gateway остаётся единственной client boundary,
а UI поставляется first-party application release.

### Что capability запрашивает

`requests` является закрытым typed union platform capabilities, например:

- `StorageNamespaceRequestV1` ADR-0224: owner, совпадающий с descriptor owner,
  required/optional, closed access profile, bounded connection budget и
  timeouts;
- `EventRouteRequestV1` ADR-0209/0220: exact envelope kind, complete durable
  contract reference, publish/consume direction и bounded `max_in_flight`.
  Он сохраняется atomically вместе с pending registration, но не содержит NATS
  endpoint, stream, wildcard, consumer name или credential и сам по себе не
  выдаёт broker permission;
- `VaultPurposeRequestV1` ADR-0223: stable purpose ID, разрешённые secret
  classes/actions, bounded target scope и requested lease TTL без secret value,
  secret reference или private account label;
- Blob operation и bounded quota;
- Clock/timer capability;
- Scheduler registration/delivery конкретного `JobKind`;
- telemetry signal class и quota;
- host capability, явно разрешённая architecture policy.

Arbitrary permission strings, wildcard owner/resource, общий database/NATS/Vault
credential и generic filesystem/network grant запрещены. Descriptor request
задаёт только верхнюю границу; effective rights вычисляются ADR-0215.

Logical storage namespace означает owner identity/object prefix внутри fixed
Макошь schemas, а не отдельную PostgreSQL schema. Descriptor не содержит SQL,
schema/table/role names, endpoint, credential, migration bytes, extension names
или vendor options. Exact `StorageBundleV1` является отдельно admitted artifact,
pinned distribution/managed binding ADR-0219/ADR-0224, и не расширяет grants.

### Dependencies не указывают implementation

Startup/runtime dependency объявляется только через:

- exact contract reference;
- required capability selector с version range;
- platform capability kind и bounded scope.

Dependency на `module_id`, executable path, process address, Cargo package или
конкретный provider implementation запрещена. Kernel выбирает ровно одного
совместимого и разрешённого provider capability либо блокирует ambiguity.

Это сохраняет правило: domain не знает integrations и соседние domains,
integration не знает business domain implementation, а workflow координирует
только public contracts.

Event subscription по-прежнему не является startup dependency сама по себе.
Если для старта нужен snapshot/query, он объявляется отдельной required
capability, а не маскируется подпиской.

### Lifecycle support

Descriptor перечисляет поддерживаемые control operations и bounded deadlines:

- `start`;
- `quiesce`;
- `drain`;
- `checkpoint`;
- `stop`;
- `health`;
- capability-local restart, если он действительно реализован;
- settings validation/apply ADR-0222.

Managed runtime обязан поддерживать `start`, `quiesce`, `drain`, `stop` и
`health`; отсутствие work допускает немедленный успешный response. Forced kill
остаётся supervisor fallback после deadline и отражается в shutdown report.

`restart_capability` можно объявить только для capability с доказанным
capability-local lifecycle. Иначе settings definition обязана требовать
`restart_module`. Kernel не угадывает более слабый lifecycle и не запускает
второй process параллельно как fallback.

### Registration и активация

Последовательность регистрации:

1. process вызывает bounded `Hello` и согласует protocol major/revision;
2. process передаёт exact descriptor bytes через `Describe`;
3. Kernel проверяет size, encoding, canonical ordering, IDs, versions и hard
   architecture policy до выдачи data-plane rights;
4. Kernel вычисляет descriptor/capability digests и сравнивает с прежней
   approved revision;
5. для managed child Kernel сверяет descriptor digest с exact binding
   ADR-0219;
6. Module Registry строит diff и owner approval state;
7. effective GrantSet создаётся отдельно для каждой capability;
8. Event Hub, storage, Vault, Blob, Scheduler, Gateway и telemetry boundaries
   получают только narrow grants;
9. storage capability дополнительно получает exact applied bundle digest и
   current-generation `StorageBindingV1` ADR-0224;
10. readiness оценивается отдельно по capabilities.

Malformed или unknown descriptor major переводит registration в
`blocked_incompatible`. Никакого legacy parser, JSON fallback или предыдущего
descriptor автоматически не выбирается.

### Изменение descriptor

Kernel сравнивает stable capability IDs и exact declarations:

- неизменённая capability с тем же semantic digest может сохранить approval;
- новая capability или расширенный request остаётся `pending`;
- новый publisher/consumer/RPC/job/settings surface считается расширением;
- сужение request немедленно уменьшает effective grants и повышает epoch;
- удаление capability запускает quiesce/revoke зависимого graph;
- несовместимый contract/schema блокирует только затронутые capabilities, если
  hard policy не классифицирует module как required целиком;
- изменение kind/owner/stable capability meaning создаёт новую registration
  identity либо incompatible major transition, а не тихую migration.

Изменение display metadata без semantic change не расширяет grants. Kernel
хранит exact previous/current digests и показывает bounded diff владельцу.

Silent reuse старого approved descriptor, automatic format fallback и
automatic topology substitution запрещены.

### Binding с distribution manifest

Для bundled managed entry signed `DistributionManifestV1` хранит path, size и
SHA-256 exact descriptor artifact рядом с executable digest. Если descriptor
объявляет settings, manifest также pin-ит exact settings schema artifact digest
ADR-0222.

Для owner-pinned managed executable fresh owner approval фиксирует executable,
descriptor и settings schema digests одной binding revision. Изменение любого
из них создаёт новую pending revision.

External runtime может зарегистрировать новый descriptor без publisher
signature, но Kernel сохраняет exact bytes/digest, показывает diff и не
расширяет grants автоматически. Self-reported digest не считается proof.

Distribution manifest больше не дублирует evolving capability/dependency
graph. Его задача — доказать immutable release artifacts; semantic graph
принадлежит проверенному descriptor.

### Связь с Settings Registry

Descriptor содержит только `SettingsSchemaRefV1`: schema major/revision,
artifact size и SHA-256. Values, account IDs, overrides и effective settings в
descriptor/manifest не попадают.

Каждая setting definition принадлежит одной capability либо module-level
scope. Kernel Settings Registry ADR-0222 проверяет schema binding и передаёт
runtime только resolved snapshot собственной registration.

Settings schema не может запросить capability, ослабить hard policy или
изменить GrantSet. Capability requests находятся только в descriptor и проходят
отдельное approval/fencing.

### Связь с Scheduler

Job code остаётся внутри module owner. Descriptor объявляет `JobKind` и default
schedule templates, а Scheduler reconciles их с собственным durable PostgreSQL
state ADR-0214.

Owner-declared default schedule template обязан быть достаточен для создания
валидного initial `JobSchedule`. Поэтому он содержит trigger/time policy и все
обязательные default policy fields ADR-0214: `overlap_policy`,
`misfire_policy`, concurrency key и maximum parallelism, timeout/deadline,
bounded retry policy, а также jitter и timezone/DST policy, когда они применимы.

В descriptor хранится только versioned default template. Persisted
`JobSchedule`, его revision, enabled/tombstone и due state, live либо
user-overridden overlap/misfire/concurrency/retry policies, run state и leases
не являются module settings, не записываются обратно в descriptor и не
дублируются в Kernel Control Store.

### Запрещённое содержимое

Descriptor и settings schema не содержат:

- secret values, tokens, cookies, passwords, provider sessions, secret
  references или credential bindings;
- account email/phone/username и private content;
- SQL, migrations, table names или database credentials;
- NATS subjects, stream/consumer names или broker credentials;
- executable code, scripts, dynamic libraries или remote includes;
- absolute/relative filesystem paths и process addresses;
- arbitrary URLs, frontend bundles, HTML/Markdown или executable UI metadata;
- settings values, cursors, checkpoints, health history или runtime state;
- business/domain entities и owner payload schemas inline.

Contract/schema artifacts указываются content digest и bounded logical
identity. Blob/media/private content передаются только соответствующими
capabilities.

### Security boundary

GrantSet ограничивает Макошь capabilities, но не превращает arbitrary external
process в OS sandbox. Одобренный external process того же desktop user всё ещё
может иметь собственный filesystem/network access вне Макошь.

Полный host permission control требует отдельного managed sandbox/entitlement
ADR. Descriptor не должен обещать enforcement, которого текущий OS process
boundary не предоставляет. Kernel distinguishes:

- enforceable Макошь capabilities;
- declared resource requests;
- отдельно проверяемые host sandbox permissions, если они появятся позже.

## Отклонённые варианты

### Один manifest для distribution и runtime semantics

Отклонено: immutable file inventory и evolving capability graph имеют разные
authority, lifecycle и причины изменения.

### Одобрять только module целиком

Отклонено: отказ/отзыв одной функции отключал бы независимые функции и делал
least privilege слишком грубым.

### Dependencies по module ID

Отклонено: domain/workflow начинал бы знать конкретный implementation, а Kernel
получал скрытый service locator.

### Свободные JSON maps

Отклонено: невозможно исчерпывающе валидировать, diff-ить, version и защищать
такой permission/configuration surface.

### Descriptor как proof executable identity

Отклонено: process может заявить любые fields. Identity и exact bytes
доказываются только launch binding либо proof-of-possession ADR-0215/0219.

### Загружать descriptor или UI по URL

Отклонено: создаёт remote code/schema supply-chain path и противоречит запрету
Kernel module downloader/plugin store.

## Проверка решения

До изменения `Состояние реализации` обязательны tests:

- exact golden Protobuf descriptor bytes и canonical ordering;
- same descriptor identity/revision с другим digest fail closed как collision;
- unknown major/revision, duplicate IDs и oversized artifact fail closed;
- descriptor digest считается по received exact bytes, не reserialization;
- один descriptor не может объявить несколько owners;
- одинаковый `module_id` не объединяет registrations;
- descriptor никогда не выдаёт grant без approval и hard policy;
- approval/revoke/readiness выполняются per capability;
- новый/расширенный capability request остаётся pending;
- сужение/удаление capability отзывает старые grants и повышает fence epoch;
- dependency по module/process/path/package identity отклоняется;
- ambiguous capability provider блокирует dependent runtime;
- unknown contract/schema hash блокирует capability до owner mutation;
- module не может объявить NATS subject или database credential;
- default schedule template без обязательной policy, включая
  `misfire_policy`, fail closed до создания `JobSchedule`, а live/overridden
  Scheduler state отсутствует в descriptor;
- managed descriptor/settings digests совпадают с signed/owner-pinned binding;
- изменённый descriptor artifact не запускается под старым binding;
- external descriptor update не наследует расширенные grants;
- runtime protocol package не зависит от Kernel, NATS, SQL/SQLite, Vault
  implementation, JSON или owner-specific contracts;
- private content/secrets отсутствуют в descriptor, registry diff, logs,
  health, errors и telemetry;
- Kernel/Gateway не линкуют owner-specific module contract packages.

Static architecture policy пока доказывает только exact package ownership,
запрещённые Cargo dependencies и наличие declared invariants. Wire, digest,
registration, lifecycle и authorization semantics потребуют production
conformance/integration tests.

## Последствия

Положительные:

- один protocol работает для всех типов independently restartable modules;
- capability failure/revoke не обязан выключать весь module;
- Kernel строит graph без compile-time dependency на owners;
- manifest, descriptor, grants и runtime state имеют разные authority;
- module update даёт точный bounded diff вместо неявного расширения прав;
- settings и jobs подключаются к тому же owner/capability model.

Стоимость:

- потребуется canonical descriptor builder и conformance suite;
- Module Registry хранит exact artifacts/digests и capability revisions;
- every platform boundary должна понимать capability-level issue/revoke;
- external process остаётся вне полного OS sandbox первой версии;
- изменение shared runtime protocol имеет широкий compile/release impact и
  требует особенно строгого review.
