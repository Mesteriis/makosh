# ADR-0209: Kernel Event Hub и контроль подписок

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Foundation in progress. `makosh-events-jetstream`
реализует NATS adapter boundary: отдельные Event Hub administration и runtime
publisher connections, bounded topology reconciliation и strict subject grammar.
Control Store atomically сохраняет exact descriptor-declared `EventRouteRequestV1`
вместе с pending registration; Kernel resolves read-only catalog entries только
из approved grants текущего epoch и fail-closed группирует их в canonical contract
catalog, отклоняя конфликтующие revision/schema до broker reconciliation. Из catalog
Kernel также строит детерминированный broker-neutral desired topology plan: exact subjects,
используемые streams, publish permits и durable consumer identities с already-declared
`max_in_flight`. Consumer descriptor теперь также обязан явно назвать requirement
(`required`/`optional`), bounded `max_deliver` и `ack_wait_millis`; legacy route без
этого policy не получает consumer topology после Control Store migration. Retention пока
не объявлен descriptor contract, поэтому plan не создаёт для него фиктивный default.
JetStream adapter также проверяет local
publish permit against exact runtime generation, grant epoch и subject до broker
publish. Adapter также имеет ciphertext-only Vault lease foundation для
per-runtime broker credential: create/resolve проходят по HPKE route с exact
registration/runtime/grant fences, а unavailable Vault не получает local-secret
fallback; revoke audience инвалидирует active Vault leases по тому же route.
Для Event Hub существует отдельный reserved Kernel audience, не маскируемый под
module registration: он может получить только предварительно импортированный
file-backed `nats-event-hub-password` через тот же ciphertext route, а identity
не раскрывает credential в diagnostics. Отдельный JWT foundation выпускает
короткоживущий non-bearer NATS user JWT для одной runtime/generation/grant-epoch
fence и exact publish/subscribe subjects; единственный wildcard — обязательный
reply inbox `_INBOX.>`. Его ephemeral Docker resolver conformance создаёт test
Operator/Account/signing key вне репозитория, подтверждает NKey challenge,
broker allowlist и отказ неизвестному account signing key. Managed Events
authority then resolves a fenced System Account credential through Vault,
publishes the revoked Account JWT to a full resolver, and proves broker-side
forced disconnect. Это не выдаёт account authority Kernel. Для production
foundation Kernel хранит только public account key и monotonic signer credential
revision в owner-authorized Control Store record, а Events authority запускается
как release-bound managed child после проверки current Vault generation.
Изменение configuration останавливает child; account signing seed по-прежнему
разрешён только Vault и возвращается authority исключительно как ciphertext
route. Test-only PostgreSQL scaffolds доказывают owner-local outbox/inbox,
exact-byte relay и pending replay при NATS outage. Reconciled broker ACL,
runtime-generation fencing и managed authority lifecycle открывают
`nats_data_plane_v1` как platform gate; production owner delivery остаётся
`first_owner_v1`.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Связано с:

- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md).

Event Hub остаётся конституционной обязанностью Kernel, но не активен в
`kernel_recovery_only_v1`. Его adapter packages, topology reconciliation и
JetStream operations открываются только `nats_data_plane_v1` после Vault,
Storage Control и managed-launch trust ADR-0225.

## Контекст

NATS JetStream обеспечивает durable delivery, replay и consumer state, но сам
по себе не является каталогом контрактов Макошь и не проверяет согласованность
signed `DistributionManifestV1`, exact validated `ModuleDescriptorV1`,
capability grants и фактически созданных streams/consumers. Event Hub также не
устанавливает executable trust: managed-launch integrity проверяется до запуска
по ADR-0219.

Без единого control plane невозможно надёжно ответить:

- какие event contracts существуют и какой модуль ими владеет;
- кто имеет право публиковать и потреблять каждый contract;
- какие подписки объявлены, созданы и действительно готовы;
- где растут lag, redelivery и dead letters;
- является ли отсутствие consumer ошибкой, отключённой optional capability или
  ожидаемым состоянием lifecycle.

При этом пропуск каждого event через Kernel превратит его в bottleneck и общую
failure domain. Поэтому контроль topology и доставка payload должны оставаться
разделёнными.

## Решение

### Роль Event Hub

**Event Hub** — обязательная подсистема Kernel, являющаяся control plane над
event topology Макошь. NATS JetStream остаётся data plane.

```text
Module outbox relay ───────────────→ NATS JetStream ───────────────→ Consumer
                                       ↑
Kernel Event Hub ── catalog / ACL / stream-consumer reconciliation / health
```

Event Hub не проксирует normal event traffic и не требуется на hot path между
publisher и consumer. Он может быть временно недоступен без потери уже
сохранённых outbox, JetStream messages и consumer state.

### Event catalog

Event Hub строит versioned desired catalog только из Kernel hard policy и exact
validated `ModuleDescriptorV1` bytes/digests, сохранённых Module Registry для
одобренных registrations ADR-0215/0221. В actual topology и runtime permissions
managed capability попадает только после успешной проверки executable,
descriptor и optional settings schema artifact binding ADR-0219/0222; external
capability — после authorization и session proof, который не считается
доказательством integrity binary. `pending`, `suspended`, `revoked`,
`blocked_incompatible` и `blocked_integrity` capability не добавляет permission
в actual topology.
Для каждого contract catalog содержит минимум:

- стабильные `owner`, `contract_name`, contract major и revision;
- message kind;
- fully qualified root Protobuf message и canonical descriptor SHA-256;
- поддерживаемые envelope revisions и exact schema hash каждой contract
  revision;
- разрешённых publishers и subscribers;
- required и optional subscription semantics;
- delivery, retry, dead-letter и retention profile;
- partition-key contract;
- максимальные размеры payload, outer header и serialized envelope;
- privacy classification и разрешённые reference types;
- schema identity/hash;
- compatibility policy.

Два владельца одного contract, неизвестная version, несовместимый schema hash,
незаявленный publisher/subscriber или расширяющий wildcard fail closed до
готовности затронутого runtime.

Catalog является техническим rebuildable state. Он не хранит domain entities,
provider payload или business policy.

### Декларативные подписки

Module runtime не создаёт произвольную production subscription самостоятельно.
`CapabilityDescriptorV1` объявляет exact contract reference, consumer identity
template, обязательность, partition policy и resource budget. Event Hub
проверяет declaration и согласует её с выданными NATS permissions. Сам module
не объявляет subject, stream или broker credential.

Subscription использует состояния:

```text
declared
  → authorized
  → provisioning
  → ready
  → lagging / degraded
  → draining
  → stopped

любое несовместимое состояние
  → blocked
```

Required subscription входит в readiness модуля. Optional subscription может
перевести только связанную capability в `degraded`. Отсутствующая, лишняя или
несовместимая subscription никогда не исправляется выдачей широкого wildcard.

### Reconciliation с NATS

После готовности NATS Event Hub:

1. проверяет JetStream identity и совместимость topology;
2. сверяет streams с contract catalog;
3. создаёт или обновляет только объявленные bounded consumers;
4. сверяет publish/subscribe ACL с module capabilities;
5. проверяет ack, delivery, backlog и resource limits;
6. публикует sanitized readiness и delivery health;
7. повторяет reconciliation после restart NATS или module runtime.

Event Hub не удаляет stream, consumer или retained messages только потому, что
runtime временно offline или descriptor revision перестала быть active.
Destructive topology change требует отдельного migration/authorization
contract.

### Наблюдаемое состояние

Для каждой subscription Event Hub отслеживает только технические metadata:

- desired и observed state;
- runtime instance и durable consumer identity;
- last delivery/ack time;
- pending, ack-pending, lag и redelivery counters;
- retry/dead-letter state;
- последнее sanitized failure class;
- resource-budget pressure;
- readiness и drain progress.

Event Hub не читает и не сохраняет event payload. Он не создаёт второй
canonical event log. Durable payload history остаётся в owner outbox и NATS
согласно retention policy ADR-0201. Техническая история delivery передаётся в
Telemetry Hub без private content.

### Event envelope и tracing

Event Hub проверяет declared `DurableEnvelopeV1` contract/catalog и topology, но
не перехватывает каждый envelope для runtime validation. Producer и consumer
adapters обязаны проверять header, subject/kind, envelope revision, exact
contract revision/schema hash, size и source fence на своей границе до owner
payload decode/mutation.

Producer cutover на новую contract revision/hash разрешается только после
reconciliation actual subscribers. Required несовместимый consumer блокирует
readiness; optional consumer должен быть явно disabled/drained и получает
scoped degraded state, а не несовместимый payload.

Envelope переносит fixed-size message/causation/correlation/trace identities и
bounded trace context ADR-0220. Произвольный baggage, user identifiers и private
content в trace context запрещены. Schema hash и self-asserted source не
являются authorization; authority задают effective grants, NATS ACL и catalog.

Event Hub связывает delivery telemetry только по техническим identifiers и не
копирует payload в logs, metrics, traces, health или diagnostics.

### Отказ и восстановление

При недоступности NATS Event Hub:

- сохраняет desired catalog в памяти из approved runtime descriptors и Kernel
  hard policy, используя сохранённые exact validated descriptor bytes/digests;
- помечает observed topology как unavailable;
- не создаёт альтернативный in-memory transport;
- не подтверждает required subscriptions как ready;
- продолжает отдавать sanitized declared/last-observed diagnostics;
- выполняет reconciliation после восстановления NATS.

Crash Event Hub является отказом Kernel subsystem и требует bounded restart
Kernel внешним watchdog. Уже сохранённые messages и consumers не очищаются.
Несовместимая topology переводит затронутые capabilities в `blocked`, а не
запускает destructive repair.

### Client и operator surface

Через Core Gateway доступны только authorized sanitized views:

- catalog contracts без schema payload;
- publisher/subscriber topology;
- readiness и lifecycle state;
- lag, retry и DLQ counters;
- blocker class и recovery action category.

Raw event payload, private subjects, credentials и arbitrary NATS admin API
клиентам не выдаются. Paired Android не получает расширенные operator
capabilities автоматически.

## Запрещено

- пропускать normal event payload через Kernel как обязательный proxy;
- интерпретировать event payload или принимать business decisions;
- создавать subject, stream или consumer вне
  `ModuleDescriptorV1`/Event Hub catalog;
- выдавать module wildcard `makosh.>`;
- хранить второй canonical event log внутри Event Hub;
- автоматически удалять orphaned stream/consumer/message;
- автоматически replay-ить DLQ или `unknown_outcome`;
- выдавать internal `DurableEnvelopeV1` через client SSE;
- логировать payload, private identifiers, secrets или blob content;
- использовать Telemetry Hub или local memory как fallback data plane.

## Отклонённые варианты

### Event Hub как собственный message broker

Отклонено: дублирует JetStream, создаёт второй delivery protocol и новую
failure domain.

### Kernel proxy для каждого event

Отклонено: делает Kernel bottleneck и связывает доступность всех модулей с его
data-path latency.

### Динамические подписки без ModuleDescriptorV1

Отклонено: невозможно доказать authorization, resource budget и compatibility
до начала обработки.

### Полная история event payload в Event Hub

Отклонено: дублирует canonical/outbox/JetStream state и расширяет privacy
surface. Для диагностики достаточно bounded technical telemetry.

## Последствия

Положительные:

- все event contracts и subscriptions имеют единый проверяемый каталог;
- Kernel видит delivery health, не становясь data-plane proxy;
- restart NATS и модулей приводит к детерминированной reconciliation;
- lag, retry и DLQ становятся частью readiness и diagnostics;
- payload и business semantics остаются у владельцев contracts.

Отрицательные:

- `CapabilityDescriptorV1` должен подробно описывать contract topology и
  budgets;
- Event Hub требует NATS administration adapter и reconciliation tests;
- topology migrations становятся явными операциями;
- Kernel получает ещё одну критическую control-plane подсистему.

## Проверка решения

Architecture guard уже требует `event_hub` в составе единственного
`makosh-kernel` и запрещает отдельный Event Hub package. Это только статическая
предпосылка: Event Hub runtime и перечисленные ниже executable scenarios ещё
не реализованы.

До признания решения реализованным должны существовать tests:

- conflicting contract owners и schema versions fail closed;
- undeclared publisher/subscriber не получает NATS permission;
- missing required subscription блокирует readiness только её capability;
- optional subscription failure создаёт scoped degraded state;
- restart NATS восстанавливает declared consumers без удаления messages;
- лишний consumer не удаляется автоматически;
- lag, redelivery и DLQ видны без чтения payload;
- Event Hub отсутствует на normal publisher-to-consumer data path;
- NATS outage не создаёт in-memory fallback transport;
- diagnostics не содержат event payload, private content или secrets;
- Event Hub не создаёт canonical event history;
- destructive topology change требует отдельной authorization/migration path.
- Event Hub не принимает self-declared `ModuleDescriptorV1` или registration
  proof как managed executable integrity evidence; authority остаётся у exact
  artifact binding ADR-0219.

До появления executable evidence поле `Состояние реализации` остаётся
`Не реализовано`.
