# ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations

Статус: Принято
Дата: 2026-07-16
Уточнено:
[ADR-0311: Storage successor bundle step lineage](ADR-0311-storage-successor-bundle-step-lineage.md).
Состояние реализации: `storage_control_v1` открыт. Реализован foundation-контур
из шести production packages,
`StorageBundleV1` Protobuf и fail-closed AST admission для узкого additive DDL.
PostgreSQL adapter реализует fixed-schema bootstrap, readiness, owner-bound
DDL/runtime roles, persistent exact binding/fence mapping и catalog-checked
owner-local DML grants. Каждый admitted step выполняется transactionally от
exact DDL role, а immutable PostgreSQL ledger остаётся admin-only. SQL-free
control lifecycle требует `active → revoking → idle` до replacement binding;
alias binding детерминирован registration/runtime generation. SQL-free revoker
подтверждает Vault, PgBouncer `PAUSE`/`DISABLE`/`KILL` и PostgreSQL
`NOLOGIN`/grant-revoke/backend-kill перед переходом в `idle`. Concrete adapters
уже отклоняют чужой binding и передают только exact PgBouncer commands. Storage
умеет сформировать и подтвердить ciphertext-only `RevokeAudience` route, а
также однократно issue-ить `Create` или `Resolve` lease и store/resolve-ить
только `PlatformCredential` с exact runtime/grant fences и одноразовым response
key. Его inherited-FD client
проходит descriptor handshake и передаёт только typed managed route. Kernel
dispatcher сверяет exact registration, caller runtime generation, grant epoch
и active Vault generation до подписания и relay opaque route. Owner-authorized
private control также сохраняет immutable canonical `StorageBundleV1` bytes и
SHA-256; managed binding разрешён только при exact owner/revision/digest match.
После Vault credential bootstrap и platform checks Storage посылает exact
`ManagedRuntimeReadyRequestV1`; Kernel сверяет registration, runtime generation
и grant epoch и только затем разрешает normal status/control relay. Поэтому
обычный control request не может пересечься с bootstrap Vault route в inherited
FD.
Runtime сначала reconcile-ит связанный набор ролей, затем применяет этот bundle
и только после обеих успешных фаз публикует PgBouncer alias. Authenticated
Docker contour подтверждает role/migration ordering с temporary runner secrets.
Отдельная conformance проверка запускает Storage child с encrypted test Vault
recipient и подтверждает `RevokeAudience`, PgBouncer и PostgreSQL fencing для
exact staged binding. Она не является evidence credential rotation или
end-to-end release deployment.
Отдельный composition test запускает real Storage credential bootstrap, exact
Kernel route handler и live Vault service по inherited Unix channels: failed
`Resolve` возвращает typed denial, Vault остаётся жив, после чего Storage
выполняет bounded `Create` и `Resolve`. Отдельный opt-in disposable Docker
runner теперь строит и запускает real Vault и Storage binaries через Kernel
Supervisor, file-imported service credentials и inherited Unix channels, затем
подтверждает `reconciling` Storage status. Это не release-deployment evidence
и не открывает gate.
Storage topology хранится как non-secret, monotonic revision в Control Store:
она содержит только profile, storage generation, identities, exact artifact
digests и validated PostgreSQL/PgBouncer host:port endpoints. At launch Kernel
combines it with the current non-secret Vault instance, runtime generation and
ephemeral HPKE public key in a private, short-lived child configuration;
runtime возвращает `reconciling` только для exact managed runtime/storage
generation/topology revision, которые Kernel сверяет перед успешным return из
launch, а staged Vault runtime generation обязан совпасть с current managed
Vault status. Credential, Vault private key и runtime attestation в topology или
Control Store не попадают. Schema
migration, которая впервые добавляет endpoints, удаляет старую неполную
topology и требует explicit owner reconfiguration. `ready` всё ещё недостижим без реального
PostgreSQL/PgBouncer/Vault evidence. Перед каждым status runtime выполняет
bounded credential-free TCP preflight обоих staged endpoints: недоступность
возвращает sanitized `failed/storage_endpoint_unavailable`, а успешная
достижимость оставляет его только в `reconciling`. Это не аутентифицирует
endpoint и не является credential delivery. Private owner control может bind/start
только exact signed artifact и менять topology только после остановки active
Storage runtime, но это не выдаёт Storage binding или infrastructure
credentials. PgBouncer имеет narrow simple-query admin transport, который
принимает short-lived credential только в Storage process. До normal control
loop managed Storage child по descriptor-bound inherited FD сначала пытается
resolve-ить existing platform credential. Vault initializer может явно
импортировать ровно два file-backed service-scoped secrets из owner-private
directory: `pgbouncer-admin-password` и `postgres-admin-password`; source
не является symlink, не читается Kernel и сохраняется только в encrypted Vault
records. При отсутствии записи Storage всё ещё может issue-ить `Create` и
вызвать Vault-generated opaque token operation, но этот token не может
аутентифицировать уже provisioned PostgreSQL/PgBouncer и не доказывает
readiness. Kernel видит только ciphertext, а ответ на generation содержит
только record identity.
Для PgBouncer и PostgreSQL используются отдельные `PlatformCredential`
purposes. После generation runtime получает plaintext только отдельным fresh
`Resolve` lease; перед normal control loop он выполняет bounded PgBouncer admin
authentication и idempotent PostgreSQL fixed-schema bootstrap. Kernel
по-прежнему видит только ciphertext. Private owner control теперь может
выпустить durable non-secret binding для exact current managed module launch:
проверяются descriptor-declared request, effective grant, runtime identity и
generation, затем binding привязывается к current topology. SQLite принимает
только initial revision `1` и последующие exact successor revisions; role и
credential-lease fence обязаны возрастать, bundle revision не может откатиться.
Перед Storage launch Kernel повторяет managed-launch/grant проверку и stage-ит
только topology-matching bindings. External runtime пока получает лишь
authorization fact, не issuance path. Это всё ещё не выдаёт credential, owner
migration bundle, credential lease или revoke sequence. Перед PgBouncer
publication runtime materializes binding в deterministic opaque DDL/runtime
roles и fail-closed выполняет PostgreSQL role reconciliation.
Disposable development Compose даёт живые PostgreSQL, PgBouncer и NATS smoke
evidence; opt-in integration test проверяет оба endpoint preflight и
PostgreSQL roles/migrations на живом instance. Это не production attestation.
PgBouncer adapter также атомарно заменяет Storage-owned private `[databases]`
include и выполняет exact `RELOAD`. Private child configuration принимает
только topology-matching `desired_bindings` и absolute private include path;
после credential bootstrap runtime применяет binding и сверяет
catalog-visible generation-scoped alias перед `ready`. Exact storage requests
from a module descriptor are persisted by registration and capability. Kernel
derives a credential-free Storage authorization fact only when that exact
capability is owner-approved and has a current external runtime attestation,
or a current managed launch identity, generation and grant epoch. For the
managed case, owner-private control transforms that fact into a durable,
successor-only non-secret binding and stages it into the next verified Storage
child after revalidation. External binding issuance likewise requires a current
exact attestation, runtime generation and grant epoch before it can persist a
non-secret fenced binding. An attested external runtime may then read only its
canonical binding, PgBouncer endpoint and current Vault public context; its
credential remains available solely through the fenced HPKE Vault route.
`make -C backend test-storage-external-process` теперь проверяет external Vault
lease delivery и rotation через owner-control IPC и отдельные Kernel/runtime
processes: temporary signed Kernel bundle запускает real managed Vault child,
а отдельный proof-backed external process получает binding и выполняет HPKE
route. После owner-authorized external revocation прежний binding получает
`runtime_session_stale`, а successor получает другой credential; test записывает
только SHA-256 assertions вместо credential plaintext. Authenticated Docker
contour проверяет owner-local migration/privilege lifecycle against real
PostgreSQL/PgBouncer. Managed
binding revoke уже резервируется в Control Store как `active → revoking` до
live side effects; Storage child принимает только exact staged binding и
выполняет Vault, PgBouncer и PostgreSQL fence. Failure оставляет reservation и
останавливает child после исчерпания bounded retry. Module owner transition
атомарно резервирует revoke всех active Storage bindings registration после
grant-epoch fence, вызывает тот же Storage child path и останавливает affected
module runtime даже при первой ошибке physical fence. После stop он повторяет
тот же exact durable reservation; только повторная ошибка останавливает Storage
child fail-closed. Недоступный Storage child оставляет durable `revoking` и не
считается успешным physical fence. Storage → Vault route после target revoke
разрешает только exact `RevokeAudience` digest для этой reservation и только
текущему verified Storage generation; другие команды и stale Storage
отклоняются. Live owner-authorized Mail transition подтверждает этот полный
порядок против реальных Vault, Storage, PgBouncer и PostgreSQL processes.
Совокупность этих conformance открывает `storage_control_v1`; она не заявляет
physical same-UID sandbox isolation сверх задокументированного evidence.

Уточняет:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: ModuleDescriptorV1 и capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Disposable authenticated Docker conformance создаёт временный signed macOS
release bundle с exact built Vault и Storage artifacts. Kernel принимает
platform bindings только через production release-binding path и запускает
реальные children через production launch path; restart Storage получает новую
runtime generation и снова проверяет signed artifact. Это доказательство
disposable signed-release execution, но не Developer ID signing, notarization
или production release attestation. Storage runtime для каждой staged binding
создаёт или fresh-resolve-ит credential в Vault scope exact runtime principal и
`credential_lease_revision`, затем устанавливает пароль соответствующей
PostgreSQL role без помещения plaintext в Kernel, Control Store или environment.
Live external credential delivery и rotation через owner-control IPC и
отдельные Kernel/runtime processes теперь покрыты disposable conformance.
Authenticated Docker contour покрывает owner migration/privilege lifecycle.
`storage_control_v1` открыт при честно зафиксированном ограничении same-UID
direct-endpoint evidence.

Storage packages, PostgreSQL/PgBouncer artifacts и runtime activation не входят
в `kernel_recovery_only_v1`; они открываются только `storage_control_v1` после
managed-launch trust и Vault ADR-0225.

## Контекст

ADR-0202 уже выбрал PostgreSQL, одну logical database, owner-scoped roles,
PgBouncer и transactional outbox. Этого недостаточно для реализации. Не были
зафиксированы:

- процесс, который создаёт cluster, roles, grants и connection budgets;
- точный storage capability contract между module, Kernel и этим процессом;
- immutable migration artifact и admission policy;
- fencing уже открытых database sessions после revoke или restart;
- fixed schemas и запрет неявного `search_path`;
- граница между Kernel supervision и привилегированным storage control plane;
- отказоустойчивость migration и schema rollback policy.

Без этих решений Kernel рискует получить PostgreSQL implementation, module —
DDL или admin credential, а migration runner — право выполнить произвольный SQL
из недоверенного runtime process. PgBouncer сам по себе также не отзывает уже
открытую PostgreSQL session при истечении credential lease.

## Решение

### Владение и process topology

Storage является platform owner `storage`. Kernel Supervisor управляет OS
lifecycle managed PostgreSQL, PgBouncer и отдельного `makosh-storage-runtime`.
Дополнительный host supervisor не вводится.

```text
Kernel Supervisor
├─ PostgreSQL
├─ PgBouncer
└─ makosh-storage-runtime        control plane
       ├─ cluster/bootstrap adapter
       ├─ roles/grants/budgets
       ├─ migration coordinator
       └─ readiness/reconciliation

module runtime ── scoped binding ──> PgBouncer ──> PostgreSQL
```

`makosh-storage-runtime`:

- bootstrap-ит и сверяет cluster identity;
- управляет extensions, roles, grants, budgets и storage generation;
- принимает только доверенные migration bundles;
- ведёт canonical migration/grant ledger;
- выдаёт typed readiness и bindings через control plane;
- не проксирует business SQL и не декодирует business payload.

Kernel:

- хранит desired infrastructure state и lifecycle hints в private Control
  Store;
- запускает, останавливает и bounded-restart-ит managed processes;
- маршрутизирует opaque storage control messages;
- не подключается к business tables и не реализует PostgreSQL bootstrap,
  migration или grant logic.

Modules подключаются к PostgreSQL только через PgBouncer. Storage runtime crash
не является причиной restart PostgreSQL/PgBouncer и не разрывает здоровый
existing data plane. Он блокирует новые bindings, migrations и reconciliation,
после чего Supervisor применяет обычный bounded restart. Kernel обязан сохранять
`recovery_only` без PostgreSQL, PgBouncer, Vault и Storage Control.

### Cargo packages

Первая package topology фиксирована:

```text
backend/src/platform/storage/protocol/     makosh-storage-protocol
backend/src/platform/storage/control/      makosh-storage-control
backend/src/platform/storage/vault/        makosh-storage-vault
backend/src/platform/storage/runtime/      makosh-storage-runtime
backend/src/platform/storage/postgres/     makosh-storage-postgres
backend/src/platform/storage/pgbouncer/    makosh-storage-pgbouncer
backend/src/platform/storage/migrations/   makosh-storage-migrations
```

Metadata:

| Package | role:owner:surface | Responsibility |
|---|---|---|
| `makosh-storage-protocol` | `platform:storage:contract` | capability request, binding, lifecycle и typed errors |
| `makosh-storage-control` | `platform:storage:implementation` | SQL-free application orchestration и internal adapter ports |
| `makosh-storage-vault` | `platform:storage:implementation` | shared ciphertext-only Vault route and fenced Storage credential lifecycle |
| `makosh-storage-runtime` | `platform:storage:runtime`, component `storage_control` | privileged process composition |
| `makosh-storage-postgres` | `platform:storage:persistence` | cluster, roles, grants, ledger и direct admin adapter |
| `makosh-storage-pgbouncer` | `platform:storage:implementation` | pooler config, readiness, pause/drain/kill |
| `makosh-storage-migrations` | `platform:storage:implementation` | bundle parser, AST admission и migration plan |

`makosh-storage-protocol` не зависит от SQL/SQLite/NATS clients, runtime
implementations, `serde_json` или owner modules. PostgreSQL client разрешён
только `makosh-storage-postgres` и owner persistence packages. Migration AST
parser разрешён `makosh-storage-migrations`, но этот package не получает
database credential или business owner implementation.

`makosh-storage-control` содержит lifecycle use cases и internal ports, но не
SQL, PgBouncer commands или process bootstrap. `makosh-storage-vault` владеет
одним reusable encrypted Vault route и exact Storage lease fencing; он не
открывает PostgreSQL или PgBouncer. `makosh-storage-runtime` собирает control
layer с concrete adapters. Такое разделение позволяет использовать тот же
Vault route у independently supervised platform runtime без дублирования HPKE
protocol и не превращает runtime entrypoint в god
package.

Kernel, Gateway и modules могут зависеть только от public protocol. Единственное
явное platform исключение — `makosh-scheduler-runtime` может импортировать
`makosh-storage-vault`: это узкий ciphertext-only adapter для уже выданного
Scheduler Storage lease, без SQL, PgBouncer или Storage Control API. Private
storage packages не импортируют owner module packages. Owner migration bundle
является packaged artifact, а не Cargo dependency Storage Control на все
persistence packages. Изменение persistence одного owner не должно
пересобирать Kernel, storage protocol или другого owner.

### Physical layout и namespaces

Макошь управляет одним PostgreSQL cluster и одной logical Макошь database. В
ней фиксированы три platform-created schemas:

| Schema | Содержимое |
|---|---|
| `makosh_data` | owner-prefixed business и provider operational objects |
| `makosh_platform` | storage/event/job technical state, ledgers и shared technical tables |
| `makosh_extensions` | extension-owned objects, если extension это поддерживает |

Schema на каждый module не создаётся. Module migration не создаёт schemas,
roles или extensions. `CREATE` у `PUBLIC` отозван во всех Макошь schemas.
Runtime role получает `search_path = pg_catalog`; production SQL всегда
schema-qualified. User-writable schema не входит в `search_path`.

Каждый object имеет owner prefix даже внутри fixed schema. Cross-owner foreign
keys и business SQL запрещены. Shared technical tables являются узкой platform
capability, а не общим доступом к `makosh_platform`.

### Roles, principals и grants

Storage Control создаёт:

- bootstrap/admin identities только для cluster/bootstrap операций;
- migration coordinator role;
- отдельную `NOLOGIN` DDL owner role для каждого durable owner;
- отдельный `LOGIN NOINHERIT` runtime principal для каждой пары
  `registration_id + runtime_generation`;
- отдельные event relay, migration, PgBouncer auth и observability roles.

Runtime principal получает прямые DML grants только на own objects и точные
shared technical capabilities. Он не получает membership в DDL role,
`SET ROLE`, DDL, catalog secrets, `BYPASSRLS`, `CREATEROLE`, `CREATEDB` или
`SUPERUSER`. Имя login principal выводится из opaque identifiers и не содержит
provider account, email или иные private identifiers.

Shared outbox/inbox/event tables принадлежат owner `events`. Module runtime не
получает прямой DML grant на них. Вместо этого он получает `EXECUTE` только на
versioned transaction-local functions наподобие
`makosh_platform.events_append_outbox_v1`. Функция:

- выводит caller из аутентифицированного runtime principal;
- сверяет current registration/runtime/storage/grant/role generations;
- проверяет owner, bounded metadata, payload size и hash;
- использует fixed `search_path` и fully-qualified objects;
- не декодирует owner business payload;
- выполняет append в вызывающей PostgreSQL transaction.

Для inbox используются такие же exact functions. Module не получает общий
`EXECUTE` на schema routines. `FORCE ROW LEVEL SECURITY` может применяться как
defense in depth, но не является cross-owner authorization boundary: authority
дают function contract, grants и caller mapping. Identity не задаётся mutable
session variable. Business mutation и append exact `DurableEnvelopeV1` bytes в
outbox остаются одной локальной PostgreSQL transaction.

V1 storage policy разрешает только
`makosh_platform.events_append_outbox_v1` и
`makosh_platform.events_accept_inbox_v1`. Новая shared technical function
добавляется решением её platform owner и exact policy update, а не проходит по
шаблону имени.

Event relay имеет отдельный privilege profile. Его cross-owner technical read
не превращается в право module runtime читать чужой business state. Static
guard распознаёт только канонические shared technical objects; database grants,
RLS и integration tests остаются authoritative enforcement.

### Connection budgets и PgBouncer

PgBouncer остаётся единственным runtime front door и работает в `transaction`
pooling mode. Module code не использует session-level state, `LISTEN`, SQL
`PREPARE`, persistent temporary tables, session advisory locks, role switching
или isolation через `search_path`.

Бюджет применяется на четырёх уровнях:

1. bounded client pool в module runtime;
2. `max_user_client_connections` и `max_user_connections` PgBouncer;
3. PostgreSQL role `CONNECTION LIMIT` и role-scoped timeouts;
4. PostgreSQL `max_connections` с relay/migration/admin reserve.

PgBouncer является pool/queue boundary, но не единственной security или budget
boundary. Direct PostgreSQL path разрешён только bootstrap, migrations,
backup/restore и controlled administration. Runtime credential для direct path
не выдаётся.

В первой desktop topology roles/grants и скрытый direct endpoint уменьшают
риск bypass, но same-UID process без OS socket/network sandbox не изолирован
криптографически от попытки открыть direct endpoint. Поэтому policy не заявляет
«физическую невозможность bypass» до sandbox conformance. PostgreSQL role hard
limits и grants остаются действительны даже при таком defect, а production
readiness обязана проверять endpoint exposure.

Первый supported PgBouncer release не ниже `1.25.2`; дальнейший upgrade
выполняется как verified infrastructure update. PgBouncer administrative
endpoint не доступен module processes. Topology не
полагается на passwordless same-UID Unix-socket admin access. Production auth
использует SCRAM. Если применяется `auth_query`, он выполняется узкой
non-superuser role через audited `SECURITY DEFINER` function с fixed
`search_path`; прямой доступ PgBouncer к `pg_authid` запрещён.

### Storage capability contract

`ModuleDescriptorV1` может содержать только `StorageNamespaceRequestV1`:

- owner, совпадающий с descriptor owner;
- required/optional;
- access profile;
- requested client/server connection budget;
- statement, lock, queue и idle-in-transaction timeouts.

Descriptor не содержит SQL, schema/table/role names, endpoint, credential,
migration bytes или vendor-specific options.

После approval, migration и readiness Storage Control выдаёт
`StorageBindingV1`:

- protocol major/revision;
- `storage_instance_id`, `storage_generation` и `database_id`;
- owner, registration и runtime instance identifiers;
- `runtime_generation`, `grant_epoch` и `role_epoch`;
- opaque runtime principal;
- opaque PgBouncer database alias, отдельный для runtime generation;
- fixed schema и owner object prefix;
- PgBouncer endpoint и logical database;
- effective budgets/timeouts;
- scoped credential lease purpose/revision;
- exact applied `StorageBundleV1` revision и SHA-256 digest.

Контракт остаётся vendor-neutral: module не получает PostgreSQL driver types,
SQLSTATE или PgBouncer admin model. Замена PostgreSQL потребует нового adapter и
migration implementation, но не изменения domain/application contracts.

### Credential delivery и session fencing

Bootstrap/admin и runtime credentials принадлежат Vault. Они не хранятся в
Kernel Control Store, PostgreSQL ledger, descriptor, settings, environment,
argv или logs. Secret material передаётся только scoped
`CredentialLease` по ADR-0223 и связывается с exact audience, purpose,
registration, runtime generation, grant epoch, role epoch и storage generation.

Для initial cluster bootstrap Storage Control сначала получает generated admin
secret из Vault. Portable baseline создаёт one-shot password file через
exclusive create внутри process-private `0700` runtime directory, устанавливает
mode `0600`, передаёт path в `initdb --pwfile`, немедленно удаляет file после
open/exit и zeroize-ит buffer. Crash cleanup и отсутствие file в backup входят
в conformance suite. Anonymous FD/FIFO разрешён только после отдельного
macOS/Linux conformance test, потому что `initdb` документирует filename, но не
stdin. `trust`, command line, environment и persistent plaintext file запрещены;
явно задаются `--auth-local=scram-sha-256` и
`--auth-host=scram-sha-256`.

Истечение lease прекращает новую выдачу, но не считается отзывом уже открытой
database session. Revoke выполняется fail-closed sequence:

1. binding переводится в `revoking`, повышается `role_epoch`;
2. выдача новых sessions и credential leases останавливается;
3. runtime process quiesce/stop-ится, runtime role получает `NOLOGIN`;
4. dedicated PgBouncer alias disable/drain/kill-ится;
5. matching PostgreSQL backends завершаются и проверяется zero sessions;
6. runtime role credential ротируется;
7. после privilege/readiness audit создаются new generation role/alias и
   выдаётся новый binding.

Тот же sequence обязателен при смене `storage_generation`, `grant_epoch`,
registration или runtime generation. Старый binding, result или session не
принимается только потому, что credential ещё технически проходит SCRAM.

### StorageBundleV1

Owner migrations поставляются как canonical binary Protobuf artifact
`StorageBundleV1`. Directory scanning, JSON/YAML manifest или произвольный SQL
из running module не являются production contract.

Bundle содержит:

- bundle contract major/revision;
- exact owner и persistence package identity;
- target schema revision;
- previous accepted bundle digest/revision;
- ordered migration step identifiers;
- exact SQL bytes и SHA-256 каждого step;
- required allowlisted extensions;
- backward-read compatibility declaration.

Для bundled managed module digest bundle входит в signed
`DistributionManifestV1`. Для promoted external managed module owner отдельно
pin-ит exact bundle digest. Runtime descriptor не загружает и не меняет bundle.
Unmanaged external process не получает storage mutation capability.

Admission и непосредственно execution повторно проверяют:

- exact artifact digest и trust binding;
- owner/package identity;
- monotonic revision и previous digest;
- отсутствие duplicate/missing/reordered steps;
- PostgreSQL AST каждого statement;
- object namespace и owner prefix;
- required extensions;
- compatibility declaration;
- effective grants и post-apply privilege state.

Production authority использует PostgreSQL-compatible AST parser. Regex guard
в repository является только быстрым heuristic precondition и не заменяет AST,
roles/grants или testcontainers tests.

`makosh-storage-postgres` и owner persistence adapters используют SQLx как
единственный concrete PostgreSQL driver V1. Другой direct database client
требует изменения policy и ADR, а не добавляется package-local решением.
Checksum/lock primitives SQLx Migrator могут переиспользоваться внутри Макошь
wrapper, но SQLx migration directory/table не являются public artifact или
canonical authority. Макошь всё равно проверяет signed bundle digest, AST,
owner namespace, quiesce/fencing и own immutable ledger. `refinery` не вводится:
вторая migration abstraction не решает эти platform obligations.

### Migration V1 policy

Migration выполняет только Storage Control через direct administrative
connection под exact owner DDL role. Module runtime и Kernel migrations не
запускают.

Первая версия разрешает только transactional owner-local changes. Запрещены:

- `CREATE/ALTER/DROP ROLE`, `DATABASE`, `SCHEMA`, `EXTENSION` или `TABLESPACE`;
- `GRANT`, `REVOKE`, `SET ROLE` и privilege escalation;
- cross-owner objects, references и foreign keys;
- `DROP`, `TRUNCATE`, rename и destructive `ALTER`;
- `DO`, dynamic SQL, user-defined `SECURITY DEFINER`, event triggers и FDW;
- `COPY ... PROGRAM`, `LOAD`, `ALTER SYSTEM` и filesystem/process access;
- explicit transaction control внутри migration step;
- non-transactional DDL, включая `CREATE INDEX CONCURRENTLY`;
- down migrations.

Разрешены additive owner-local DDL и bounded owner-local data transformations с
statement/lock timeout. Более широкая online migration policy требует нового
ADR и conformance suite.

Storage Control получает global coordinator lock и owner-scoped advisory lock.
Каждый step и его immutable ledger record commit-ятся одной transaction. Если
step N падает, committed steps 1..N-1 не откатываются, bundle остаётся
`blocked_migration`, owner runtime не запускается, а исправление поставляется
новым forward step/bundle. Другие owners продолжают работу.

Перед schema cutover текущий owner runtime quiesce/drain-ится. Новый runtime
запускается только если exact applied bundle digest совпадает с binding.
Automatic down migration и automatic binary fallback запрещены. Предыдущая
binary может быть явно запущена только при declared compatibility с уже
применённой schema.

### Canonical storage ledger

`makosh_platform.storage_*` tables принадлежат owner `storage` и содержат:

- cluster/database identity и storage generation;
- accepted owner bundles, steps, digests и results;
- role bindings, grant/role epochs и effective budgets;
- readiness/reconciliation state и sanitized failure codes.

Canonical applied schema/bundle state находится в PostgreSQL ledger. Kernel
Control Store хранит только desired lifecycle state и last-acknowledged hints,
достаточные для recovery UI. Secrets, credential record IDs, private content и
business payload в ledger запрещены.

### Transaction boundary

Owner persistence adapter управляет local transaction:

```text
owner state mutation
+ shared technical outbox append
-------------------------------
one PostgreSQL transaction
```

Storage Control и Kernel не предоставляют generic distributed transaction
service. Cross-owner consistency идёт через durable command/event, owner inbox,
idempotency и compensation. Database transaction не включает provider/network
call, NATS wait или user interaction.

### Restart и recovery

Planned PostgreSQL restart:

1. Kernel quiesce-ит storage-dependent capabilities и резервирует next storage
   generation;
2. owner runtimes drain-ят active transactions, old roles получают `NOLOGIN`;
3. Storage Control pause-ит PgBouncer database, ждёт bounded drain и завершает
   remaining old sessions;
4. PostgreSQL выполняет checkpoint и штатный stop/start;
5. Storage Control проверяет cluster identity, WAL/recovery state, extensions,
   ledger, schemas, roles, grants и budgets;
6. `storage_generation` commit-ится, credentials/roles ротируются;
7. PgBouncer resume-ится, затем выдаются только new-generation bindings;
8. owner runtimes возобновляются, outbox/inbox replay продолжается.

PgBouncer crash перезапускает только pooler. Storage Control crash
перезапускает только Storage Control. Pool exhaustion не перезапускает
PostgreSQL. Automatic reset, data-directory replacement, restore или grant
loosening запрещены.

Backup/restore обязан сохранять database state вместе с roles/grants и storage
ledger и проверяться в disposable cluster. Отдельный operations ADR определит
retention, encryption и media format до production rollout.

## Отклонённые варианты

### Kernel как storage implementation или SQL proxy

Отклонено: Kernel потеряет zero-external-dependency bootstrap и станет owner
business query semantics.

### Module self-migrations

Отклонено: running code не должен менять собственную security boundary или
предоставлять migration SQL после admission.

### SQL directory/regex как migration authority

Отклонено: directory scanning не pin-ит exact artifact, а regex не является SQL
parser и не закрывает dynamic/destructive statements.

### Общий runtime login или PgBouncer как единственный limiter

Отклонено: общий principal смешивает ownership/failure budget, а pooler revoke
не гарантирует завершение PostgreSQL backend session.

### Schema/database на каждый module

Отклонено как обязательная topology для single-owner local product. Fixed
schemas, owner-prefixed objects, roles/grants и RLS дают нужную boundary без
отдельной operational database на каждый module.

### Automatic down migration и data reset

Отклонено: schema rollback после partial forward progress недостоверен, а
автоматический reset разрушителен. Исправление всегда forward-only.

## Последствия

Положительные:

- Kernel не получает PostgreSQL implementation или plaintext credentials;
- module defect ограничен principal, grants, RLS и connection budget;
- migration bytes immutable и связаны с admitted executable distribution;
- revoke реально fences existing sessions, а не только будущие leases;
- storage failure не смешивается с lifecycle остальных managed services;
- owner persistence changes сохраняют compile isolation.

Отрицательные:

- Storage Control, role/grant reconciler и migration admission становятся
  отдельным production subsystem;
- fixed schemas и fully-qualified SQL делают persistence более многословным;
- PgBouncer и PostgreSQL limits/credentials нужно согласованно ротировать;
- destructive/online schema changes требуют явного будущего решения.

## Проверка решения

Static architecture policy обязана fail closed проверять:

- exact storage package set, metadata и dependency direction;
- единственный SQL client surface и запрет SQLite в business persistence;
- schema-qualified owner-prefixed SQL;
- отсутствие global migrations и module runtime DDL;
- migration bundle ownership/layout/digest preconditions;
- отсутствие private storage dependencies у Kernel/modules;
- отсутствие compatibility exceptions.

Static SQL inspection не объявляется security proof. Первый implementation
slice добавляет Testcontainers PostgreSQL + PgBouncer и проверяет:

- CRUD только own tables и запрет cross-owner SQL/FK;
- exact shared table grants и `FORCE RLS` isolation;
- direct PostgreSQL path недоступен runtime principal;
- transaction pooling и отсутствие session-state assumptions;
- hard PostgreSQL role limit сохраняется при обходе pooler endpoint;
- pool exhaustion одного runtime не блокирует другой;
- tampered/unknown/reordered/destructive bundle fail closed;
- crash до/после step commit и повторное reconciliation;
- state + exact outbox bytes atomicity;
- stale storage/runtime/grant/role epochs rejected;
- revoke завершает PgBouncer и PostgreSQL sessions до нового binding;
- PgBouncer, Storage Control и PostgreSQL failure/restart classes;
- fresh bootstrap и restore в disposable cluster;
- отсутствие credentials/private data в logs, ledger, errors и health.

Наличие ADR и JSON policy не доказывает работающий runtime. Состояние реализации
меняется только после появления packages, contracts и зелёного integration
suite.

## Ссылки

- [PostgreSQL `initdb`](https://www.postgresql.org/docs/current/app-initdb.html)
- [PostgreSQL privileges](https://www.postgresql.org/docs/current/ddl-priv.html)
- [PostgreSQL schemas and `search_path`](https://www.postgresql.org/docs/current/ddl-schemas.html)
- [PostgreSQL advisory locks](https://www.postgresql.org/docs/current/explicit-locking.html)
- [PgBouncer configuration](https://www.pgbouncer.org/config)
- [PgBouncer administration console](https://www.pgbouncer.org/usage)
- [`pg_query.rs`](https://github.com/pganalyze/pg_query.rs)
- [SQLx](https://github.com/launchbadge/sqlx)
