# ADR-0225: Первый production slice — recovery-only Kernel и фазовые ворота

Статус: Принято
Дата: 2026-07-16
Состояние реализации: recovery foundation, Control Store anchor/export/restore/reset
fencing, atomic v1→v15 migrations, bounded single-writer actor и architecture
self-tests реализованы. `module_control_plane_v1` также
открыт с private owner-control, module-registration и external-runtime-session
IPC, Module Registry, capability router и Settings Registry.
`server_bootstrap_pairing_v1` открыт с one-shot TLS Linux bootstrap listener
и atomic Control Store consumption; `managed_launch_trust_v1` открыт на
file-backed P-256 release authority. `vault_v1` открыт на exact five-package
Vault graph: file-backed key, SQLCipher single-writer store, recovery slot,
HPKE session, fenced leases, owner-authorized managed runtime и classified
offline backup/restore conformance реализованы.
`telemetry_v1` открыт: exact Collector packages, private Unix transport,
fixed-shape privacy/quotas, bounded retention, signed managed launch,
three-attempt failure isolation и owner-private aggregate diagnostics имеют
runtime conformance. Client-facing telemetry export остаётся частью будущего
`client_gateway_v1` и не открывается этим gate.
Production Kernel не содержит development profile или development operator;
simulation-команды живут только в отдельном
`makosh-development-kernel-operator`. `clock_v1` открыт отдельным ADR-0229:
exact UTC/monotonic Clock packages и deterministic conformance существуют без
Scheduler, module timers или data plane.
`browser_client_v1` открыт как отдельный owner-neutral local Gateway phase:
signed single-document browser bootstrap, owner-approved WebAuthn pairing,
cookie-backed session и bounded `BrowserSessionService/GetStatus` существуют
без business owner, remote/public listener, HTTP/3 или owner receipt mapping.
Полный `client_gateway_v1` остаётся следующей отдельной фазой.

Уточняет:

- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL ownership, PgBouncer и extensions](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и subscription control plane](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0211: Backend workspace и физическая структура исходного кода](ADR-0211-backend-workspace-and-source-layout.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0214: Durable Job Platform и Scheduler](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0217: Нулевой внешний bootstrap Kernel](ADR-0217-zero-external-dependency-kernel-bootstrap.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane и owner-scoped PostgreSQL](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).
- [ADR-0227: Deployment profiles и server bootstrap pairing](ADR-0227-deployment-profiles-and-server-bootstrap-pairing.md).

## Контекст

Clean-room ADR уже определили финальную конституцию Kernel, но до этого решения
первый production package оставался заблокирован неопределёнными capability и
domain ownership inventory. Если сразу реализовать весь финальный graph, первый
срез одновременно потребует PostgreSQL, PgBouncer, Storage Control, NATS,
Vault, Telemetry Collector, modules, settings и client API. Такой срез нельзя
проверить изолированно и невозможно безопасно откатить по ответственности.

Обратная крайность — создать пустые crates, fake handlers или состояние
`ready` без capabilities — также запрещена. Первый срез обязан доказывать
завершённое полезное поведение: Kernel запускается без внешних сервисов,
владеет private Control Store, удерживает single-instance boundary и всегда
оставляет безопасный локальный recovery surface.

Это решение закрывает inventory только для первого среза. Оно не объявляет
готовыми business domains, managed infrastructure или data plane.

## Решение

### Текущий implementation slice

Первый разрешённый production slice имеет stable ID:

```text
kernel_recovery_only_v1
```

Его максимально достижимое состояние — `recovery_only`. Пустой business или
required-capability inventory не превращает Kernel в `ready`. Если позже
появится HTTP readiness endpoint, он возвращает not-ready с sanitized reason
`production_phase_recovery_only`.

Переход к следующему slice выполняется только явным изменением ADR, executable
policy и tests. Feature flag, settings value, environment variable, manifest
entry или обнаруженный executable не открывает фазовый gate.

### Точный production package inventory

Базовый `kernel_recovery_only_v1` разрешил ровно шесть production packages:

| Package | Макошь metadata | Ответственность |
|---|---|---|
| `makosh-events-protocol` | `platform:events:contract` | Реальный binary `DurableEnvelopeV1` contract ADR-0220 без NATS/outbox runtime |
| `makosh-runtime-protocol` | `platform:runtime_protocol:contract` | Реальные lifecycle, health, descriptor и settings wire types ADR-0221 без module activation |
| `makosh-gateway-protocol` | `api:gateway:contract` | Typed local recovery requests, results и safe errors |
| `makosh-kernel-control-store` | `core:kernel:contract` | Узкий Control Store port, models и typed errors без `rusqlite` |
| `makosh-kernel-control-store-sqlite` | `core:kernel:persistence` | Private SQLite schema, migrations, integrity validation и Control Store export/restore adapter |
| `makosh-kernel` | `core:kernel:runtime` | Один process/binary: bootstrap, lock, state machine, recovery routing и bounded shutdown |

Пустой scaffold не считается package. Protocol packages содержат принятые
Protobuf V1 messages и conformance tests, даже если соответствующий data plane
ещё выключен.

Допустимый production dependency graph:

```text
makosh-kernel-control-store-sqlite
  -> makosh-kernel-control-store

makosh-kernel
  -> makosh-kernel-control-store
  -> makosh-kernel-control-store-sqlite
  -> makosh-gateway-protocol
  -> makosh-runtime-protocol

makosh-events-protocol
  -> no production implementation
```

`makosh-gateway-protocol` может зависеть только от необходимых platform
contracts. Он не переиспользует internal durable envelope как client frame.
Protocol packages не зависят от Kernel, SQLite, SQLx, NATS, HTTP clients,
filesystem или owner packages.

Test-only package не входит в production inventory. Если нужен shared process
harness, он живёт только в `backend/tests/support/kernel-recovery/` как
`makosh-kernel-recovery-testkit` с metadata `test:test:test_support`, запускает
Kernel как внешний child и не импортирует production composition root.

Текущий executable inventory имеет slice `clock_v1`: к базовым
шести packages добавлены ровно `makosh-vault-protocol`,
`makosh-vault-key-provider`, `makosh-vault-key-provider-file`,
`makosh-vault-store-sqlcipher`, `makosh-vault-runtime`,
`makosh-clock-protocol` и `makosh-clock-runtime`. Это exact inventory открытых
`vault_v1` и `clock_v1`; ни один data-plane package этим не авторизуется.
Текущий `gateway_session_foundation_v1` добавляет один API implementation
package `makosh-gateway-session`: он владеет только memory-only session fences
через gateway-owned authority port и не зависит от Kernel/SQLite implementation,
не запускает listener и не открывает `client_gateway_v1`.

### Пустой business ownership inventory

Для текущего slice inventory фиксируется явно:

```text
domains      = []
integrations = []
workflows    = []
engines      = []
business capabilities = []
```

`domains.registered` и `domains.developmentAllowlist` остаются product
governance ADR-0207/0208. Allowlist означает «разрешено проектировать и
реализовывать после открытия owner gate», а не «уже входит в текущую
distribution». До `first_owner_v1` никакой domain, integration, workflow,
engine или AI production package не допускается.

Test-only delivery scaffolds may reserve exact owner-local PostgreSQL schemas
and validate durable outbox/inbox mechanics for the development allowlist.
They are not owner packages, do not contain business tables, handlers, public
contracts or migrations, and do not open `first_owner_v1`.

### Активные Kernel components

ADR-0206 сохраняет полный закрытый перечень обязанностей Kernel. В текущем
slice реально активны:

- `supervisor` — process state machine, data-directory lock, bounded shutdown
  и lifecycle без managed children; SIGTERM/SIGINT опрашиваются с bounded local
  interval и удаляют все private control-plane sockets перед exit;
- `core_gateway` — owner-private local recovery IPC surface;
- `module_registry` — pending registration, owner-authorized approval/revoke и
  effective GrantSet lifecycle через private IPC;
- `capability_router` — current-generation runtime session fencing и policy
  admission без business payload routing;
- `settings_registry` — immutable schema binding и desired/effective settings
  lifecycle через private IPC.

Остальные Kernel-owned components зарезервированы конституцией, но не
объявляются реализованными и не получают fake handlers:

- Event Hub;
- Telemetry control;

Emergency bootstrap/crash log является bounded внутренней boot
ответственностью и не выдаётся за Telemetry Collector.

### Runtime states и операции

В текущем slice разрешены состояния:

```text
cold_start
bootstrap
recovery_only
quiescing
draining
stopped
fatal
```

Запрещены `infrastructure_starting`, `modules_starting`, `ready` и `degraded`:
они требуют capabilities, которых в inventory нет.

Online local IPC предоставляет ровно:

- `GetRecoveryStatusV1`;
- `ValidateControlStoreV1`;
- `ExportControlStoreV1`;
- `ShutdownKernelV1`.

`shutdown` является lifecycle operation и не превращает unavailable Control
Store в writable recovery surface. Online restore/reset, settings mutation,
module approval, infrastructure start/stop и business request запрещены.

На pristine instance разрешён один bootstrap mutation
`InitialOwnerEnrollmentV1` только через inherited private FD и platform signer
по ADR-0218. Он не доступен через registration listener, HTTP, argv,
environment или обычный local IPC. Повторная enrollment без отдельной
owner-authorized recovery operation отклоняется.

Offline packaged CLI сохраняет только уже принятые Control Store operations:

```text
control-store restore --data-dir <explicit-path>
control-store reset --data-dir <explicit-path>
```

Они требуют остановленный Kernel, explicit data directory, exclusive lock,
interactive local confirmation и generation/epoch fencing. Это не
whole-instance backup/restore и не затрагивает PostgreSQL, Vault, blobs,
provider sessions или business data.

### Нулевые внешние сервисы и exact crate profile

До следующей фазы Kernel не запускает и не подключает:

- PostgreSQL, PgBouncer или Storage Control;
- NATS/JetStream;
- Vault;
- Blob service;
- Telemetry Collector;
- Scheduler;
- module runtimes;
- provider integrations.

В production graph текущего slice запрещены NATS clients, PostgreSQL clients,
Vault implementation, provider SDK и owner packages. При этом recovery
поведение не реализуется самописной криптографией, CLI parser или SQLite file
copy. На 2026-07-16 разрешён только следующий direct crates.io profile:

| Package | Exact direct dependency profile |
|---|---|
| три `makosh-*-protocol` | `prost =0.14.4`, `prost-types =0.14.4`, build-only `prost-build =0.14.4`, `protoc-bin-vendored =3.2.0`; default features |
| `makosh-kernel-control-store-sqlite` | `rusqlite =0.40.1`, defaults off, `backup,bundled`; `sha2 =0.11.0`, defaults off |
| `makosh-kernel` | `clap =4.6.2`, defaults off, `derive,error-context,help,std,usage`; `directories =6.0.0`; `p256 =0.14.0`, defaults off, `ecdsa`; `getrandom =0.4.3`, defaults off; `sha2 =0.11.0`, defaults off; `signal-hook =0.3.18` |

Version requirement, dependency kind, crates.io source, default-feature mode и
feature set проверяются executable policy. Rename, optional edge, git/path
substitution и неразрешённый direct dependency fail closed. Internal edges
проверяются по resolved Cargo package ID, а не только по совпавшему имени.

Разделение ответственности:

- `clap` владеет typed offline/recovery commands, feature `env` не включён;
- `directories::ProjectDirs::data_local_dir()` выбирает OS-standard local
  data path;
- `getrandom` выдаёт challenge bytes из OS CSPRNG, ошибка блокирует operation;
- `p256` только проверяет ES256 proof: private device key остаётся platform
  signer;
- `sha2` в Kernel считает operation/anchor digests, а в SQLite adapter —
  checksum export artifact;
- `signal-hook` устанавливает только SIGTERM/SIGINT flags для bounded polling
  private recovery socket; handler не выполняет I/O или state mutation;
- `rusqlite::backup` создаёт consistent export; открытый database file не
  копируется напрямую.

`p256 0.14.0` прямо предупреждает, что его curve arithmetic не проходила
независимый полный аудит. Это не скрывается и crate не объявляется
сертифицированным. Для текущего verifier-only boundary решение принимается с
обязательными cross-platform vectors, malformed/non-canonical/length checks,
low-S enforcement и dependency/advisory review до merge первого manifest.
Замена crypto backend меняет policy и security evidence отдельным решением.
Самописная криптография запрещена.

Дополнительный lock/IPC crate в текущем Unix/macOS slice не нужен:
`std::fs::File::try_lock` удерживает advisory single-instance lock, а private
Unix socket использует `std::os::unix::net`. Поддержка Windows local IPC не
объявляется этим slice и требует отдельного platform profile.

### Минимальная модель времени

Полностью исключить clock нельзя: bootstrap challenge, session deadline,
expiry и bounded shutdown уже требуют корректного времени. Поэтому
`makosh-kernel` использует внутренний внедряемый `KernelClock` port:

- wall clock используется только для UTC timestamps и persisted absolute
  instants;
- monotonic clock используется для elapsed duration, timeout и deadline внутри
  process lifetime;
- тесты используют deterministic fake clock;
- wall-clock jump не продлевает monotonic in-process deadline;
- Kernel не публикует module-facing Clock capability в текущем slice.

`clock_v1` открыт ADR-0229 до Scheduler, periodic jobs и module timer requests.
Rust `SystemTime` и `Instant` имеют разные semantics и не подменяют друг друга;
открытая Clock boundary не выдаёт module timer capability.

## Фазовые ворота

### Implementation update: `module_control_plane_v1` (2026-07-16)

После executable evidence для owner-device proof session, bounded descriptor
parser, persisted grant/revoke epoch и private IPC replay/abuse tests gate
`module_control_plane_v1` открыт. Production Kernel получает только private
Unix owner-control, module-registration и external-runtime-session sockets,
Module Registry, capability router и Settings Registry. Public/network Gateway,
managed launch, external services, NATS, Vault, Blob, Scheduler и business
data-plane остаются закрытыми и не входят в этот profile.

`not_authorized` означает запрет package, process, route, dependency, feature
flag и runtime activation. Gate открывается только одним согласованным
изменением ADR + policy + executable evidence.

### `module_control_plane_v1`

Открывает Module Registry, external `pending` registration, owner approval,
GrantSet, capability router и Settings Registry.

До открытия нужны:

- owner/device session conformance ADR-0218;
- exact ModuleDescriptorV1 parser/limits;
- grant/revoke epoch persistence;
- local IPC replay/abuse tests;
- owner-authorized mutation surface.

### `server_bootstrap_pairing_v1`

Открывает только explicit Linux bootstrap command с one-shot TLS listener.
Он выдаёт CSPRNG 256-bit bearer, ephemeral certificate fingerprint и challenge;
file-backed ES256 proof проверяется до atomic initial-owner claim. Control Store
сохраняет только token digest, certificate digest, challenge, expiry и consumed
state. Wrong token, expiry, replay и second enrollment fail closed; listener
закрывается после первого successful claim. Это не normal Gateway и не даёт
Kernel Docker socket, managed launch либо public service authority.

### `managed_launch_trust_v1`

Открывает любой managed child launch. До gate должны быть зафиксированы и
проверены:

- exact binary encoding и schema digest `DistributionManifestV1`;
- detached signature suite;
- источник pinned verification keys и rotation policy;
- release-signing evidence;
- path/symlink/partial-install validation;
- TOCTOU-safe platform spawn adapter;
- exact-byte executable/descriptor/settings verification перед каждым launch.

Tauri updater/platform signature не заменяет внутренний managed-launch
verification. Kernel download/install, automatic rollback и silent fallback
остаются запрещены.

### `vault_v1`

Открывает Vault packages/process только после `managed_launch_trust_v1` и
требует exact package inventory, HPKE session conformance, SQLCipher/platform
key adapter, lease expiry/revoke/epoch fencing, secret non-disclosure tests и
backup/restore classification. Verified launch сам по себе не доказывает Vault
semantics.

### `telemetry_v1`

Открывает Telemetry Collector только после `managed_launch_trust_v1` и требует
exact package inventory, private local transport, schema/redaction/quotas,
bounded retention, collector failure isolation и negative tests на secrets и
private content.

### `storage_control_v1`

Открывает PostgreSQL, PgBouncer и Storage Control после
`managed_launch_trust_v1` и `vault_v1`. До gate нужны exact packages/artifacts,
role/grant/pool/budget conformance, AST migration admission, Vault credential
fencing, evidence ограничения PgBouncer bypass и readiness/recovery tests.

### `nats_data_plane_v1`

Открывает managed NATS, Event Hub reconciliation и durable delivery. Gate
зависит от `managed_launch_trust_v1`, `vault_v1` и `storage_control_v1`:
Vault выдаёт per-runtime credentials, а PostgreSQL outbox/inbox остаётся
durable truth. До gate требуются:

- exact NATS artifact/version/listener profile;
- Event Hub/NATS adapter package inventory;
- subject catalog version;
- concrete stream retention/bytes/storage/replica budgets;
- concrete consumer ack/delivery/backoff/deadline/DLQ budgets;
- per-runtime NATS identity;
- credential authority, protected delivery, expiry, rotation, revoke, forced
  disconnect и runtime/grant-generation fencing;
- outage, replay, duplicate и stale-generation tests.

Shared broker token и временный wildcard `makosh.>` запрещены.

Current implementation evidence includes a managed Events authority that
receives an already signed bounded Account JWT only through owner-private
Kernel control, validates its bound Account NKey, resolves a fenced System
Account credential from Vault and applies the update through the NATS resolver.
The JWT conformance contour uses that path for an Account-claim revocation and
proves broker disconnect of the active runtime. Together with the exact-byte
outbox/inbox contract, owner-neutral PostgreSQL outage/replay conformance,
Event Hub topology reconciliation and runtime/grant fencing, this opens the
gate as a platform capability. It does not admit a business owner or make a
test scaffold a production owner: a real owner-local PostgreSQL
business-mutation/outbox transaction remains required by `first_owner_v1`.

### `whole_instance_backup_v1`

Зависит от `vault_v1`, `telemetry_v1`, `storage_control_v1` и
`nats_data_plane_v1` и обязателен до production rollout первого durable
owner/user data. Gate требует:

- component inclusion matrix;
- quiesce/consistency order;
- retention и encryption/key authority;
- signed media/manifest format;
- PostgreSQL roles/grants/storage-ledger handling;
- Control Store, Vault, provider session, Blob и JetStream inclusion policy;
- restore authorization, order и generation/epoch fencing;
- полный restore test в disposable environment.

Control Store export или component-local SQLite/PostgreSQL backup не закрывает
этот gate.

### `blob_v1`

Зависит от `managed_launch_trust_v1` и `vault_v1`. Обязателен до первой выдачи
`BlobRef` и до attachments, documents или media. Gate требует exact
protocol/package topology, opaque ref bindings, encryption,
owner/account/runtime/grant/expiry scopes, quotas, retention/GC, range policy,
path-traversal defense, revoke semantics, backup classification и replay tests.
Если Blob уже включён, `whole_instance_backup_v1` дополнительно зависит от
`blob_v1`.

`blob_v1` открыт как platform gate: exact protocol/runtime/service packages,
opaque reference and fence validation, encrypted atomic filesystem storage with
aggregate quota ledger, bounded range and path safety, two-phase retention/GC,
and a live signed Blob-to-file-backed-Vault contour are present. This does not
create a first owner or a generic content API. Owner-local metadata and the
owner-specific content-session issuer remain under `first_owner_v1`; complete
instance restore remains under `whole_instance_backup_v1`.

Если Scheduler уже включён, `whole_instance_backup_v1` также условно зависит
от `scheduler_v1` и включает его schedules, runs, leases и fencing state.

### `clock_v1`

Зависит от `module_control_plane_v1` и `managed_launch_trust_v1`. Обязателен до
Scheduler, periodic jobs и module timer capability. Gate требует public
protocol/package topology, wall/monotonic semantics, UTC/timezone/DST, clock
jump/drift/suspend behavior, deadline contract и deterministic fake-clock
suite. Внутренний `KernelClock` текущего slice не открывает этот gate.

### `scheduler_v1`

Открывает Scheduler и Job Plane только после `module_control_plane_v1`,
`managed_launch_trust_v1`, `vault_v1`, `telemetry_v1`,
`storage_control_v1`, `nats_data_plane_v1` и `clock_v1`. Gate требует:

- exact Scheduler protocol/runtime/package inventory;
- exact `JobSpec`/`JobKind`/owner contract binding;
- PostgreSQL schedule/run/lease/fencing storage;
- NATS acceptance/result/ack contract;
- hot reconciliation без dynamic Rust loading;
- retry/idempotency/misfire/recovery semantics;
- deterministic clock, lease-race и crash tests.

Код job остаётся в owner module; Scheduler владеет временем, scheduling state,
run identity и fencing, но не business execution.

### Implementation update: `scheduler_v1` (2026-07-19)

Gate открыт как owner-neutral platform capability. Exact package inventory,
revisioned `JobSpec`/`JobKind` binding, PostgreSQL schedule/run/lease fencing,
exact-byte JetStream dispatch and receipt-before-ACK, hot schedule
reconciliation, retry/idempotency/recovery и deterministic Clock conformance
имеют executable evidence в `test-scheduler-conformance` и managed Scheduler
Docker contour. `backend/architecture/policy.json` и executable phase-gate
policy теперь разрешают только `scheduler_v1`.

Это не открывает first owner, не добавляет business handler, domain package или
public client transport: Scheduler принимает только approved typed owner
contract после отдельного owner ADR, а owner command/result outcome остаётся
вне этого gate.

### `client_gateway_v1`

Открывает public client transport только после `module_control_plane_v1`,
`telemetry_v1` и `nats_data_plane_v1`. Gate требует:

- exact gateway package и listener inventory;
- owner-device session/authentication/authorization conformance;
- ConnectRPC deadlines, typed errors и durable command receipt mapping;
- один SSE stream с replay, gap/reset и explicit disconnect semantics;
- отделение ConnectRPC от health/OAuth/blob/SSE HTTP surface;
- remote HTTP/2 + TLS baseline;
- HTTP/3 fallback, raw-QUIC prohibition и no-0-RTT conformance;
- abuse, replay, privacy, redaction и Android suspension/reconnect tests.

Текущий local recovery `core_gateway` не открывает public listener и не
считается реализацией этого gate.

### `first_owner_v1`

Открывает первый owner module любого типа: domain, integration, workflow или
engine — только после отдельного owner ADR с exact packages, public contracts,
StorageBundle, capabilities, dependencies и tests. Gate требует
`module_control_plane_v1`, `managed_launch_trust_v1`, `vault_v1`,
`telemetry_v1`, `storage_control_v1`, `nats_data_plane_v1` и
`client_gateway_v1` и `whole_instance_backup_v1`. Для owner с blobs
дополнительно требуется `blob_v1`; для owner с jobs, schedules или timers —
`scheduler_v1` (который уже требует `clock_v1`).

Единственное исключение до `client_gateway_v1` — exact read-only
Mail/Communications package set, утверждённый ADR-0239. Оно записывается в
`phaseGates.ownerAdmissionExceptions` с exact ADR, package inventory и уже
открытыми prerequisites. Это не открывает `first_owner_v1`, не допускает
другой owner package и не разрешает mutation, provider execution или generic
owner admission.

Если Scheduler state уже включён к моменту whole-instance backup,
`whole_instance_backup_v1` условно зависит от `scheduler_v1` и обязан включить
его PostgreSQL state, leases/fencing и restore order. Наличие schedules у
первого owner никогда не открывает Scheduler автоматически.

Таким образом, flat toggle открыть позднюю фазу не может. `phaseGates.requires`
и `conditionalRequires` executable policy фиксируют тот же порядок и меняются
атомарно с ADR и evidence.

## Всё ещё запрещено до соответствующих platform gates

- domain/integration/workflow/engine/AI production packages;
- business API, SSE, public HTTP/TCP listener или Android connection;
- NATS/Event Hub data plane;
- PostgreSQL/PgBouncer/Storage Control, online Vault service, credential lease,
  HPKE secret transport, Blob, Scheduler или public client gateway;
- provider-account/agent identity;
- whole-instance backup claim;
- legacy import, migration или compatibility facade;
- автоматический reset, fallback или переход в `ready`.

## Проверка первого recovery slice

До изменения `Состояние реализации` обязательны:

- policy принимает exact six-package recovery inventory и отдельно exact
  `vault_v1` inventory без owner/domain packages;
- empty и test-only workspace остаются допустимыми;
- лишний domain/integration/platform implementation package ломает guard;
- любой file под `backend/src`, не принадлежащий registered root текущего exact
  inventory, ломает guard, включая hidden `.rs`, `.proto`, `.sql` и `build.rs`;
- Kernel metadata объявляет только реально active components текущего slice;
- package dependency graph не содержит NATS/PostgreSQL/HTTP clients,
  Storage/provider packages или undeclared internal edges; Vault edges
  ограничены exact `vault_v1` allowlist, а любой internal edge
  разрешается только к exact Cargo package ID, а не registry namesake;
- direct third-party dependencies совпадают по crate, exact version, kind,
  crates.io source, default-feature mode и feature set;
- Cargo features не могут скрыто открывать phase, а `makosh-kernel` имеет один
  binary target без собственного build script;
- runtime state set не содержит `ready`, `degraded` или startup states будущих
  managed capabilities;
- boot без PostgreSQL/NATS/Vault/Blob достигает `recovery_only`;
- пустой capability inventory не достигает `ready`;
- invalid/missing Control Store оставляет только status/validate/export/shutdown;
- online mutation, network listener и managed spawn отсутствуют;
- pristine enrollment принимает только inherited-FD proof;
- second enrollment, replay и OS-identity-only authorization отклоняются;
- single-instance lock и bounded shutdown доказаны process tests;
- wall/monotonic/fake-clock semantics доказаны deterministic tests;
- diagnostics не содержат secrets или private content;
- `make -C backend validate` проходит.

## Последствия

Положительные:

- запрет первого production package заменён точным проверяемым scope;
- Kernel можно реализовать без одновременного запуска всей инфраструктуры;
- будущие возможности не маскируются fake components;
- опасные managed/NATS/backup/blob paths fail closed до своих решений;
- product allowlist отделён от фактически реализованного inventory.

Отрицательные:

- первый slice намеренно не является usable product backend;
- два protocol packages реализуются раньше их transport adapters;
- переход к каждому следующему gate требует отдельного ADR/policy change;
- whole-instance backup становится обязательной ранней работой до user data.

## Ссылки

- [prost 0.14.4](https://docs.rs/prost/0.14.4/prost/)
- [clap 4.6.2](https://docs.rs/clap/4.6.2/clap/)
- [directories 6.0.0 `ProjectDirs`](https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html)
- [p256 0.14.0 и security warning](https://docs.rs/p256/0.14.0/p256/)
- [getrandom 0.4.3](https://docs.rs/getrandom/0.4.3/getrandom/)
- [sha2 0.11.0](https://docs.rs/sha2/0.11.0/sha2/)
- [rusqlite 0.40.1](https://docs.rs/rusqlite/0.40.1/rusqlite/)
- [Rust `File::try_lock`](https://doc.rust-lang.org/stable/std/fs/struct.File.html#method.try_lock)
- [Rust `UnixListener`](https://doc.rust-lang.org/stable/std/os/unix/net/struct.UnixListener.html)
- [Rust `SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html)
- [Rust `Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html)
- [NATS authentication](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/auth_intro)
- [NATS authorization](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/authorization)
- [NATS credential revocation](https://docs.nats.io/using-nats/nats-tools/nsc/revocation)
- [Tauri updater security model](https://v2.tauri.app/plugin/updater/)
- [SQLite Online Backup API](https://www.sqlite.org/backup.html)
- [PostgreSQL backup approaches](https://www.postgresql.org/docs/current/backup.html)
