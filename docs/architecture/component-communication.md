# Контракт взаимодействия компонентов

Статус: Принятая архитектура, реализация не начата
Дата: 2026-07-16

Источники active policy:

- [ADR-0200: Модульная модель и изоляция runtime](../adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](../adr/ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](../adr/ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](../adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](../adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](../adr/ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](../adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md).
- [ADR-0207: Канонический реестр бизнес-доменов Макошь](../adr/ADR-0207-canonical-business-domain-registry.md).
- [ADR-0208: Allowlist разработки доменов и запрет проекций](../adr/ADR-0208-domain-development-allowlist-and-projection-freeze.md).
- [ADR-0209: Kernel Event Hub и контроль подписок](../adr/ADR-0209-kernel-event-hub-and-subscription-control-plane.md).
- [ADR-0210: Telemetry Hub и локальная диагностика](../adr/ADR-0210-telemetry-hub-and-local-diagnostics.md).
- [ADR-0211: Backend workspace и физическая структура исходного кода](../adr/ADR-0211-backend-workspace-and-source-layout.md).
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](../adr/ADR-0212-crate-topology-and-compile-isolation.md).
- [ADR-0213: Конституция кода, ownership и автономность модулей](../adr/ADR-0213-code-ownership-and-module-autonomy.md).
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](../adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md).
- [ADR-0215: Открытая регистрация модулей и capability grants](../adr/ADR-0215-open-module-registration-and-capability-grants.md).
- [ADR-0216: Private Kernel Control Store на SQLite](../adr/ADR-0216-private-kernel-control-store-with-sqlite.md).
- [ADR-0217: Нулевой внешний bootstrap Kernel](../adr/ADR-0217-zero-external-dependency-kernel-bootstrap.md).
- [ADR-0218: Owner/device identity, enrollment и offline recovery](../adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md).
- [ADR-0219: Целостность managed modules и explicit updates](../adr/ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md).
- [ADR-0220: Канонический durable envelope и эволюция контрактов](../adr/ADR-0220-canonical-durable-envelope-and-contract-evolution.md).
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](../adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md).
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](../adr/ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md).
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](../adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md).
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](../adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).
- [ADR-0225: Первый production recovery-only Kernel slice и фазовые ворота](../adr/ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).
- [ADR-0226: Контекст для AI через use-case workflows](../adr/ADR-0226-ai-context-acquisition-through-use-case-workflows.md).

Архивные ADR и legacy executable contract не являются policy новой системы.

## Компоненты

- Core runtime — supervisor, module registry, capability router и внешний API
  gateway без business logic. Его список обязанностей закрыт ADR-0206.
- Module Registry — техническое состояние local registrations, approval,
  lifecycle mode и grant revision. Неизвестный process остаётся `pending` без
  data-plane capabilities до явного решения владельца.
- Settings Registry — exclusive Kernel component для verified module schemas,
  typed desired/effective revisions, optimistic concurrency и supervised
  application. Он компонует owner sections для UI, но не мержит ownership.
- Kernel Control Store — private SQLite с desired technical state Registry,
  grants, module settings и managed infrastructure. Он доступен до
  PostgreSQL/Vault, не содержит business/runtime data или secrets и не является
  module storage capability. Его
  исправность нужна для data plane, но не для restricted local recovery.
- Event Hub — Kernel control plane для event catalog, subscription
  reconciliation, NATS permissions и delivery health без чтения payload.
- Telemetry Hub — Kernel policy/control surface и отдельный managed Collector
  для logs, metrics, traces и crash/lifecycle diagnostics.
- Domain module — владелец одного bounded context и его durable truth.
- Integration plugin — владелец provider protocol, auth/session runtime,
  cursor, operational contract/projection и neutral evidence mapper, но не
  business truth.
- Provider experience surface — bundled frontend controller конкретного
  integration-плагина; это не domain и не удалённо загружаемый код.
- Desktop client — Vue/Tauri application, использующее local embedded Gateway
  profile.
- Android client — planned first-party application, использующее тот же public
  contract через local embedded или paired remote Gateway profile.
- Workflow module — координатор нескольких public contracts.
- Platform capabilities — independently owned storage, events, vault, blobs,
  clock и scheduler boundaries.
- Storage Control — отдельный managed control-plane runtime для PostgreSQL
  bootstrap, roles/grants/budgets, migration admission и readiness. Он не
  проксирует business SQL и не является module storage API.
- Vault runtime — отдельный verified managed process, который владеет encrypted
  credential store, key hierarchy и process-bound leases. Kernel routes только
  ciphertext и не видит credential plaintext.
- Scheduler runtime — отдельный platform module, который хранит time policy и
  создаёт durable commands, но не содержит owner job handlers.
- Product projection module — зарезервированная будущая роль; implementation
  полностью заблокирована ADR-0208.

Каждый independently restartable runtime является отдельным OS-процессом.

Supervisor является подсистемой Kernel. Он управляет managed PostgreSQL,
PgBouncer, Storage Control, NATS, Vault, Blob, Telemetry Collector и managed
module runtimes, но не является отдельным обязательным Макошь-процессом. External runtime он
авторизует, fences и наблюдает без process control. Kernel перезапускается Tauri
или OS watchdog.

Kernel обязан достичь `recovery_only` без PostgreSQL, PgBouncer, Storage Control,
NATS, vault и module runtimes. Domain, workflow или integration failure переводит только
затронутые capabilities и Kernel в `degraded`; глобальный `fatal` зарезервирован
для потери доверия к самому control plane.

Six-package `kernel_recovery_only_v1` остаётся boot baseline. Текущий
`vault_v1` дополнительно реализует private module control plane,
managed-launch trust и isolated Vault runtime. `clock_v1` добавляет standalone
UTC/monotonic Clock boundary и deterministic fake only. NATS, business RPC,
public listeners и переход в `ready` отсутствуют; data-plane gates закрыты.
Описание остальных компонентов является target contract, а не
доказательством активного runtime.

Это boot-инвариант: Vault не расшифровывает и не открывает Kernel Control Store.
При недоступном Vault, Registry, non-secret settings, desired topology и local
recovery actions остаются доступны, а credential-dependent operations
блокируются.

Обязательного bootstrap configuration file нет. Kernel выбирает OS-standard
private data directory либо один explicit `--data-dir`. Если Control Store
недоступен или недоверен, Kernel не переключается на другую directory и не
запускает infrastructure по defaults: online остаются только local
status/validate/export. Restore/reset выполняются offline при stopped Kernel,
exclusive lock и explicit target по ADR-0218.

## Допустимые interaction kinds

- `local_call` — вызов внутри одного module implementation;
- `control_rpc` — lifecycle и health через Protobuf RPC по Unix socket;
- `query_rpc` — синхронный read-only запрос через capability router;
- `request_rpc` — синхронная typed операция с immediate result;
- `durable_command` — требование изменить state через PostgreSQL outbox и NATS
  JetStream;
- `event` — immutable факт владельца state через outbox и JetStream;
- `observation` — факт внешнего наблюдения с provenance и cursor;
- `result` — только terminal outcome durable command;
- `ack` — отдельный durable Макошь message о durable acceptance, canonical
  persistence или terminal handling; это не JetStream ACK;
- `projection` — rebuildable read/search model, построенная из owned facts;
- `client_rpc` — owner-specific ConnectRPC query/request/command через Gateway;
- `client_realtime` — replayable multiplexed SSE через Gateway;
- `client_blob` — bounded HTTP transfer по opaque BlobRef capability;
- `host_bridge` — только OS/bootstrap capability desktop или Android.
- `telemetry_signal` — bounded structured log, metric, trace или lifecycle
  record через private local Telemetry Collector channel.

Contract выбирает один delivery mode. Одна и та же операция не отправляется
одновременно как RPC и NATS command.

Provided `request_rpc` по умолчанию принадлежит owner provider module. Узкое
исключение ADR-0354 разрешает только `Integration` module реализовать exact
foreign-owned provider extension port после explicit capability approval.
Contract/schema authority при этом не переходит integration; `query_rpc`,
client surfaces, events и все остальные module kinds сохраняют same-owner
правило.

Event Hub launch configuration является capability-scoped и для Domain, и для
Integration runtimes: eventful runtime получает exact endpoint/credential
revision pair, eventless runtime — только `empty + 0`. Фиктивный NATS grant
ради общего launch shape запрещён ADR-0352 и ADR-0355.

Interaction kind `projection` зарезервирован контрактом, но production
projection packages, runtimes, schemas и consumers запрещены ADR-0208 до
отдельной разблокировки.

`telemetry_signal` не является event, command или canonical audit record. Он не
проходит через NATS/PostgreSQL и не содержит business payload.

## Durable background jobs

Scheduler, Event Hub и owner runtime имеют разные responsibilities:

```text
clock → Scheduler → producer outbox → NATS → owner inbox → owner Job Executor
event → owner/workflow outbox ───────────────┘
manual command → owner outbox ──────────────┘
```

- Scheduler владеет revisioned schedules, due time, misfire/overlap policy,
  dispatch identity и schedule leases.
- Kernel только supervises Scheduler, проверяет его exact-byte launch binding и
  runtime descriptor; Kernel не пишет schedule tables и не интерпретирует job
  payload.
- Event Hub управляет declared NATS topology и delivery health, но не вычисляет
  время запуска и не исполняет jobs.
- Исполняемый handler, JobExecution, checkpoint и business result принадлежат
  target module.
- JetStream ACK подтверждает broker delivery после durable owner inbox commit и
  не является Макошь Ack-envelope. При необходимости owner отдельно публикует
  `DURABLE_ACCEPTANCE` Ack; terminal status возвращается result, progress —
  event.

При startup любой module с background work объявляет JobKind catalog reference
в `ModuleDescriptorV1`. Scheduler создаёт global default только при первом появлении
identity. Scope-specific default создаётся после owner `EnsureSchedule`, когда
account/entity/workflow scope уже существует. Existing/user-modified schedule
не перезаписывается; delete сохраняет tombstone, а missing/incompatible handler
переводит schedule в blocked.

Schedule configuration меняется через revisioned typed command без restart.
Изменение Rust handler выполняется owner-local rebuild, explicit host/OS install
или fresh owner-pinned binding и запуск новой verified process generation по
ADR-0219; executable code и scripts в PostgreSQL запрещены.

## Settings и reconfiguration

Каждый domain, integration, workflow, engine или platform module публикует
собственный bounded `SettingsSchemaV1`, связанный digest с
`ModuleDescriptorV1`. Domain не импортирует и не собирает settings integrations.

```text
module-owned schema artifacts
        ↓ Describe + exact digest
Kernel Settings Registry
        ↓ private Control Store desired revision
ValidateSettings / ApplySettings либо supervisor restart
        ↓
runtime control acknowledgement of effective revision
```

- `operator_managed` fields меняет только authenticated owner через Core
  Gateway с `expected_desired_revision`;
- `kernel_managed` fields меняет только compiled/allowlisted Kernel controller;
- Vault выдаёт scoped `CredentialLeaseV1` отдельно от settings snapshot; secret
  values и bindings настройками не являются;
- JobSchedule, run/lease/misfire state остаются Scheduler, даже если UI
  показывает их рядом с module settings;
- `hot_reload`, `restart_capability` и `restart_module` являются explicit apply
  modes; capability restart допустим только при declared local lifecycle;
- failed apply сохраняет desired/effective drift и `blocked_config`, но не
  выполняет automatic rollback;
- external runtime получает `awaiting_external_restart`; Kernel не посылает ему
  OS signals.

Kernel компонует только client-safe catalog view. Каждая mutation остаётся
owner-targeted; cross-owner atomic settings transaction не обещается.

## Маршрутизация

Прямое module-to-module соединение запрещено. Synchronous и durable paths
разделены:

```text
synchronous query/request:
source → Kernel capability router → target local IPC

durable command/event/observation/result/ack:
source outbox → exact DurableEnvelopeV1 bytes → NATS JetStream → target inbox
                     ↑
Event Hub ── catalog / ACL / topology / health control plane
```

Kernel проверяет identity/capability для routed RPC. Event Hub reconciles
durable contract catalog и permissions, но не проксирует normal payload.
Producer/consumer adapters проверяют ADR-0220 envelope/catalog без
интерпретации business payload platform layer.

Kernel и Core Gateway не имеют Cargo dependencies на owner-specific module
packages. Они обнаруживают capabilities через exact `ModuleDescriptorV1` в
`makosh-runtime-protocol`. Descriptor является runtime declaration, но не
executable trust и не GrantSet. Signed bundled distribution manifest или
owner-pinned binding отдельно pin-ит executable, descriptor и settings schema
digests для managed launch. Module runtime зависит от общего runtime protocol,
но никогда не зависит от Kernel implementation.

Cross-domain behavior принадлежит workflow:

```text
source domain event
    ↓
workflow
    ↓
target domain command
```

Cross-owner AI context использует тот же принцип, но не становится generic
workflow или projection:

```text
owner event / user command
    ↓
explicit use-case workflow
    ↓ typed owner query contracts
distinct generated request + AiContextReceiptV1
    ↓
AI candidate/result
    ↓
workflow policy → target-domain command or review
```

AI не получает cross-owner SQL, grants к чужим tables или прямой query access к
другим owners. Workflow фиксирует `as_of`, source revisions, completeness,
privacy/egress policy и digest; concrete request живёт только в пределах
request/run, не использует global fragment union/opaque payload и не
размораживает Context projection.

## Граница integration-плагина

Provider различия сохраняются до operational frontend surface. Mail, Telegram,
WhatsApp и Zulip не обязаны изображать один универсальный набор операций.
Каждый bundled integration-плагин предоставляет собственный versioned
operational contract и может использовать общий presentation shell через
узкие provider-neutral props.

Provider multi-step mutation не объявляется атомарной только потому, что client
видит один command. Telegram folder reassignment использует один owner-local
durable operation, fresh current snapshot, deterministic add/remove delta и
обязательную final provider equality. Partial success повторно планируется от
current provider state; intent не становится projection до provider
observation, а intermediate `ok` не является terminal completion.

Контекстные домены не видят этот operational contract. Плагин сам публикует
отдельное neutral evidence observation:

```text
External provider
        ↓
Integration plugin
        ├─→ operational projection → provider experience
        └─→ neutral evidence → NATS → context/memory domains
```

Provider identity сохраняется в provenance, но не определяет domain behavior.
Core маршрутизирует оба contract family по descriptor/effective capabilities и не выполняет
provider-to-domain mapping.

На compile boundary integration может импортировать из business domains только
exact public Communications contract units:
`makosh-communications-ingress` для general provider-neutral evidence и
`makosh-communications-attachment-contract` для attachment lifecycle.
Client-facing Communications API, Communications implementation, persistence
и runtime для integration недоступны. Остальное взаимодействие происходит
через описанные envelopes и Core routing, а не через Rust imports.

Bundled integration runtimes присутствуют в signed distribution manifest, а
frontend experiences — в подписанном application bundle. Plugin store, runtime
download и remote frontend code не поддерживаются. Kernel не скачивает,
распаковывает, устанавливает или заменяет executable; это host updater/OS
boundary. Rollback является только explicit owner-authorized host operation.

## Граница клиентских приложений

Desktop и Android общаются только с Core Gateway:

```text
Desktop / Android
        ↓ ConnectRPC + SSE + bounded HTTP
Core Gateway
        ↓ capability router
module runtimes
```

- ConnectRPC/Protobuf обслуживает typed queries, requests и commands.
- Один multiplexed SSE stream на active client session даёт replayable
  realtime, sanitized platform system-status transitions и terminal command
  results. Initial `ClientBootstrap` является snapshot/recovery query; client
  не polling-ит его ради health или latency.
- SSE использует отдельный `ClientRealtimeFrameV1`; internal
  `DurableEnvelopeV1` клиенту не выдаётся.
- Обычный HTTP используется только для health/readiness, OAuth callbacks,
  blobs и SSE.
- Tauri/Android host bridge обслуживает только OS capabilities и bootstrap, а
  не business API.
- Host bridge владеет platform `DeviceSigner`; Vue/WebView и Kernel получают
  только public key, proof и short-lived session capability.
- NATS, PostgreSQL, PgBouncer и module sockets клиенту не видны.

Owner является logical authority, а каждое desktop/Android/operator device
имеет отдельную ES256 keypair. Private key не покидает platform signer.
Первый desktop enrollment проходит только через inherited FD на pristine
instance; последующие sessions используют challenge-response и independent
device revocation.

Desktop local profile использует private loopback HTTP. Paired Android profile
использует защищённый HTTP/2 baseline и preferred HTTP/3 over QUIC после
conformance проверки. HTTP/3 меняет network transport, но не Protobuf contracts
или application semantics. При блокировке UDP разрешён наблюдаемый fallback на
защищённый HTTP/2; 0-RTT запрещён.

Android background suspension не останавливает Kernel. На resume клиент
восстанавливает SSE по собственному cursor. Offline cache не является
canonical truth, а push notification не переносит domain state/private content.

## Control plane

Control plane не зависит от NATS и остаётся доступным при его отказе. Он
поддерживает handshake, descriptor/settings validation, start, quiesce, drain,
stop, health, settings application и capability renewal/revocation.

Module process не передаёт bootstrap secrets через argv, environment или logs.
Secret-bearing Vault frames проходят только как HPKE ciphertext через versioned
capability routing и не попадают в NATS, SSE или filesystem spool.

Storage Control принимает только versioned storage control messages для
bootstrap, binding, migration, readiness и revoke. Normal business queries не
маршрутизируются через Kernel или Storage Control: после выдачи binding module
подключается напрямую к PgBouncer. Storage credentials остаются scoped Vault
leases и не входят в control message, settings, argv, environment или logs.

Local registration endpoint принимает bounded `Hello`/`Describe` от любого
локального process, но discovery не является authorization. Effective rights:

```text
requested by module
∩ owner-approved grant state
∩ hard Kernel policy
= effective GrantSet
```

`pending`, `suspended`, `revoked` и `blocked_incompatible` registrations не
получают PostgreSQL, NATS, Vault, Blob, Job Platform или business RPC rights.
Изменение approval повышает `grant_epoch`, закрывает старую session и fences
downstream credentials до canonical mutation. Для завершения уже
зарезервированного Storage revoke текущий verified Storage runtime может после
этой mutation передать в Vault только exact `RevokeAudience` по durable
`revoking` binding; это узкий lifecycle route, а не восстановление data-plane
прав target module.

Blob data authority разделяет current access и durable custody. Descriptor
объявляет exact `custody_scope_id`, quota и непустой operation set; Kernel
выдаёт signed one-use session только для operation из текущего approved
capability. Registration, runtime generation и grant epoch fences session и
Vault lease, но не входят в at-rest identity. Ciphertext, Vault content-key
scope и technical quota ledger связываются с logical owner, stable custody
scope, opaque reference и key schema revision. Поэтому successor runtime или
отдельный read-only capability того же scope может получить новый current
lease к существующим bytes, не наследуя write/transfer authority. Другой scope,
legacy format или persisted metadata без fresh current operation grant fail
closed.

`managed` module запускается и перезапускается Kernel через private inherited
control channel только после проверки exact-byte `ManagedLaunchBinding` перед
каждым launch. Bundled binding происходит из signed immutable distribution
manifest; external → managed transition требует fresh owner-approved executable,
descriptor и settings schema digest pin. Integrity mismatch даёт
`blocked_integrity` без repin, другого path/version или automatic rollback.
Managed successor не резервирует новую generation одновременно со старым
worker. После durable `revoking` supervisor сначала idempotently quiesce-ит
exact predecessor через `stop_requested`, затем Storage/Vault physically
fence-ят прежние credentials/alias/role, Kernel join-ит worker и только после
этого выдаёт generation `N + 1`. Quiesce запрещает autorestart во время fence,
но не является authorization или data-plane fence.

`external` module запускается вне Kernel: Kernel проверяет proof-of-possession
owner-bound runtime identity. Первый proof включает exact 32-byte observed
distribution digest, runtime generation, current grant epoch и одноразовый
challenge; только успешная проверка создаёт external attestation. Kernel
выдаёт/отзывает grants и наблюдает disconnect, но не посылает signals и не
обещает restart или publisher integrity его installation. Publisher signature
не требуется, чтобы local process стал `pending` или после approval работал как
`external`; runtime digest proof не заменяет managed binding или OCI trust.
Lifecycle mode не переключается автоматически.

Registrations, approvals, settings revisions и epoch сохраняются через пять
узких Control Store ports. `ControlStoreHandle` владеет единственным online
SQLite connection, queue 64 и deadlines 2/30 секунд; raw SQL, rows и file path
не пересекают adapter boundary. Один созданный при boot store/actor передаётся
во все production registration, owner-control, external-runtime и recovery
IPC workers; ни один request handler не переоткрывает SQLite. Trustworthy
online export выполняется maintenance-task того же actor.
Если store повреждён, Kernel
не reset-ит его молча и не запускает data plane по defaults: minimal bootstrap
recovery surface online допускает только sanitized status/validate/export.
Restore/reset являются exclusive offline operations и используют внешний
`.makosh-recovery-fence-v1`; reservation/store mismatch оставляет Kernel в
recovery-only.

## Vault и credential leases

Vault не является частью Kernel process и не открывает Kernel Control Store.
Он запускается supervisor только после trustworthy Control Store и exact-byte
verification managed executable. Первая версия допускает только bundled managed
Vault; external Vault и automatic implementation/topology fallback запрещены.

```text
module descriptor Vault purpose request
        ∩ owner-approved GrantSet
        ∩ hard Kernel/Vault policy
        ∩ current runtime session and generation
        ↓
Kernel authorizes and routes HPKE ciphertext
        ↓
Vault validates purpose / audience / epoch before decrypt
        ↓
process-bound CredentialLeaseV1
```

Kernel видит только sanitized Vault state, generation и blocker code. Он не
имеет HPKE recipient private key, `VaultRootKey`, record keys или credential
plaintext. Vault не предоставляет generic enumeration или
`GetSecret(secret_ref)` API.

Lease связан с logical owner, opaque configuration instance, purpose, actions,
module/runtime audience, current `grant_epoch`, Vault generation и exact secret
revision. Default TTL V1 — 10 минут, hard maximum — 1 час; resolve single-use,
renewal заново проверяет grants. Vault lock/restart/restore, module restart,
suspend/revoke или epoch/generation change инвалидируют lease.

Vault хранит только bounded credential material и wrapping keys. Большие или
high-churn provider session databases остаются в private integration-owned
encrypted stores. Hidden WhatsApp WebView сохраняет cookies/storage только в
OS-managed per-account profile. Подробный canonical contract находится в
[Vault and credential leases](vault-and-credential-leases.md).

ADR-0223 принято, но Vault runtime, SQLCipher store, file-key adapter,
transport и conformance tests ещё не реализованы.

## Infrastructure lifecycle

Supervisor subsystem не зависит от PostgreSQL или NATS и остаётся доступным в
recovery mode. Managed services могут перезапускаться только по
service-specific bounded policy и только после повторной exact-byte verification
ADR-0219. External services никогда не получают signal от Kernel. Restart
процесса не удаляет и не заменяет PostgreSQL, JetStream, encrypted Vault records
или provider session state. Vault restart создаёт новую runtime generation и
инвалидирует все active leases.

Storage Control crash перезапускает только Storage Control и блокирует новые
bindings/migrations/reconciliation; здоровый существующий module → PgBouncer →
PostgreSQL data path не обязан разрываться. PgBouncer crash перезапускает только
pooler, а pool exhaustion не является причиной restart PostgreSQL.

## Data plane

- PostgreSQL является canonical relational source of truth для module-owned
  business и operational state. Kernel control state, Vault credentials, blobs
  и telemetry имеют отдельные owners.
- Mutation и outbox append выполняются одной локальной транзакцией owner
  module.
- Producer сериализует `DurableEnvelopeV1` один раз в outbox; relay публикует
  byte-for-byte тот же buffer.
- NATS JetStream выполняет durable delivery, fan-out и replay; единственный
  обязательный transport header — derived `Nats-Msg-Id`.
- Consumer фиксирует inbox `message_id` + envelope hash и local mutation до
  JetStream ACK. Same ID с другими bytes fail closed в quarantine.
- End-to-end semantics — at least once; exactly-once не обещается.
- Ordering существует только внутри явного partition key.
- Retry bounded; `unknown_outcome` не повторяется автоматически.
- Private bodies, documents, media и secrets не проходят через NATS; сообщения
  используют bounded metadata и opaque blob/evidence references.
- Dead letter является отдельным sanitized technical record; original bytes
  остаются в bounded owner quarantine, automatic replay запрещён.

## Storage boundary

Kernel supervises PostgreSQL, PgBouncer и отдельный Storage Control process.
Storage Control не находится на normal data path: module runtime использует
собственную generation-scoped PostgreSQL role и выполняет business SQL напрямую
через PgBouncer. Он не читает чужие business tables и не получает direct
administrative connection. Cross-owner SQL и foreign keys запрещены.

`StorageBindingV1` связывает endpoint/principal/budgets с storage, runtime,
grant и role generations и exact applied migration bundle digest. Runtime
credential выдаётся только как process-bound Vault lease. Migration code
поставляется immutable owner bundle, проходит AST/ownership admission и
исполняется Storage Control с отдельной DDL role; module runtime не передаёт SQL
и не выполняет DDL.

PgBouncer является runtime front door и bounded pool, но не единственным
security/budget boundary. PostgreSQL grants, role connection limits, timeouts и
session fencing остаются обязательными. Без OS-level socket/network isolation
нельзя доказать, что same-UID process физически не попробует direct PostgreSQL
endpoint; это отдельный обязательный conformance gate, а не свойство Cargo roles
или PgBouncer configuration.

Shared outbox/inbox/event tables являются platform state и защищаются grants и
RLS по module identity.

Secret material не хранится в PostgreSQL и не проходит через outbox/inbox.
Module получает его только как scoped lease через ADR-0223 Vault boundary.

Canonical operational details находятся в
[Storage Control Plane](storage-control-plane.md). ADR-0224 принято, но
реализован только foundation-контур packages/Protobuf/AST admission; managed
services и PostgreSQL/PgBouncer conformance tests ещё отсутствуют.

## Запрещено

- domain-to-domain implementation import;
- integration-to-domain implementation import;
- domain dependency/subscription на provider operational contract;
- shared presentation component с импортом provider generated types;
- client-to-module/NATS/PostgreSQL direct connection;
- business REST/JSON рядом с эквивалентным ConnectRPC method;
- business operation через Tauri/Android host bridge;
- HTTP/3-only remote API, raw QUIC RPC или 0-RTT requests;
- module-to-module socket;
- чтение или запись чужих tables как API;
- прямой AI → owner query orchestration, read-all database grants или generic
  Context API;
- durable AI-owned Context projection или копия чужого canonical state;
- business SQL через Kernel или Storage Control proxy;
- module self-migrations, DDL/admin credential или direct PostgreSQL runtime
  binding;
- shared in-memory production event bus;
- durable fire-and-forget через Core NATS;
- прямой internal durable envelope в client SSE;
- decode/re-encode outbox envelope внутри relay;
- generic Vault enumeration/read-by-reference API;
- credential plaintext или Vault keys в Kernel, PostgreSQL, Control Store,
  settings, NATS, SSE, argv, environment, logs или filesystem spool;
- secrets, private content или user-provided identifiers в NATS subjects и
  diagnostics;
- unbounded queues, retries или connection pools;
- automatic runtime/topology fallback;
- Kernel executable download/install и automatic rollback/downgrade;
- remote/plugin-store executable code loading.

## Состояние реализации

Этот документ описывает принятую clean-room архитектуру, а не существующий
runtime. Legacy scripts и код в `references/backend-legacy/` не являются
доказательством реализации. Статус изменяется только после появления новых
executable guards и process-level integration tests.
