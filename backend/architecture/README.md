# Executable architecture policy

Файл [`policy.json`](policy.json) является executable companion для active ADR
clean-room системы. ADR объясняют решение, а policy и linters запрещают
нарушающие его packages, paths, SQL ownership и dependency edges.

[ADR-0211](../../docs/adr/ADR-0211-backend-workspace-and-source-layout.md)
принимает `backend/` как единственную физическую границу. Policy, linters,
architecture tests и virtual Cargo workspace находятся внутри этой границы;
root-level compatibility paths отсутствуют и запрещены layout guard.

[ADR-0212](../../docs/adr/ADR-0212-crate-topology-and-compile-isolation.md)
фиксирует owner-local Cargo graph. Guard запрещает aggregate packages,
module-to-Kernel и Kernel/Gateway-to-module dependencies, runtime aggregation,
cross-owner persistence edges и любой integration-to-domain contract, кроме
точного `makosh-communications-ingress`.
Правило одинаково для Mail, Telegram, Zulip и будущих integrations; WhatsApp
помечен host-only и не может иметь backend implementation, persistence или
runtime package.

[ADR-0213](../../docs/adr/ADR-0213-code-ownership-and-module-autonomy.md)
определяет внутреннее качество owner packages: одна ответственность и причина
изменения, явные dependencies/side effects, owner-local lifecycle и
самостоятельные tests. Текущий guard реализует Cargo/storage/test-layout часть;
process lifecycle и code-shape evidence добавляются с production code.

[ADR-0214](../../docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md)
фиксирует общий background-job boundary для всех owners. Scheduler является
отдельным platform runtime, а handlers, executions и checkpoints остаются в
owner packages. Текущая policy уже резервирует platform owner `scheduler`, но
package graph, runtime и persistence ещё не реализованы.

[ADR-0215](../../docs/adr/ADR-0215-open-module-registration-and-capability-grants.md)
фиксирует открытую локальную регистрацию, deny-by-default до approval, typed
GrantSet и разные lifecycle guarantees для `managed` и `external` runtime.
Текущий static guard проверяет hard package/storage boundaries; runtime
registration, grant epoch и downstream credential revocation потребуют
executable tests вместе с production Kernel.

[ADR-0216](../../docs/adr/ADR-0216-private-kernel-control-store-with-sqlite.md)
выбирает private SQLite Control Store, доступный без PostgreSQL, NATS и Vault.
Port и `rusqlite` adapter являются отдельными core-owned packages; raw SQLite
dependency разрешена только persistence surface и не попадает в module graphs.

[ADR-0217](../../docs/adr/ADR-0217-zero-external-dependency-kernel-bootstrap.md)
запрещает обязательный configuration file и внешние service prerequisites до
`recovery_only`. Policy разрешает только OS-standard default либо explicit
CLI data directory; недоступный Control Store оставляет local IPC recovery без
business data plane и automatic reset.

[ADR-0218](../../docs/adr/ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md)
отделяет logical owner authority от OS identity и module processes. Каждое
device использует отдельную ES256 keypair через platform signer; public identity
и revoke state принадлежат Control Store. При недоверенном store online
разрешены только status/validate/export, а restore/reset требуют stopped Kernel,
explicit data directory, exclusive lock и interactive confirmation.

[ADR-0219](../../docs/adr/ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md)
разделяет open external registration и managed-launch integrity. Policy
требует exact-byte verification перед каждым managed launch, signed
distribution manifest для bundled entries и owner-pinned digest для
перевода external executable в `managed`. Kernel download/install и automatic
rollback/fallback запрещены.

[ADR-0220](../../docs/adr/ADR-0220-canonical-durable-envelope-and-contract-evolution.md)
фиксирует exact internal durable wire contract и его эволюцию.
Блок `events` резервирует единственный canonical durable-envelope package
`makosh-events-protocol` с metadata `platform:events:contract`. Контракт
использует binary Protobuf envelope major version 1 и ровно пять `oneof` kinds:
`command`, `event`, `observation`, `result`, `ack`. Owner payload остаётся
непрозрачным для Kernel/Event Hub и связывается с catalog contract, version и
SHA-256 schema hash; relay публикует сохранённые outbox bytes без повторной
сериализации. Client envelope и broker ack не подменяют durable envelope/ack,
неизвестный major отклоняется без format fallback. Cargo guard требует этот
package вместе с первым production workspace и запрещает ему NATS, SQL/SQLite
clients и `serde_json` через normal, build и dev dependencies.

[ADR-0221](../../docs/adr/ADR-0221-module-descriptor-and-capability-lifecycle-contract.md)
отделяет self-declared runtime descriptor от executable trust, effective
GrantSet и observed runtime state. Блок `runtimeProtocol` резервирует
единственный `makosh-runtime-protocol` с metadata
`platform:runtime_protocol:contract`: descriptor передаётся как exact binary
Protobuf artifact, хешируется по полученным bytes и не содержит собственного
digest. Approval, readiness и revoke определены на уровне capability;
dependencies ссылаются только на contracts/capabilities, а не module/process
identity. Static Cargo guard требует protocol package вместе с первым
production workspace и запрещает ему NATS, SQL/SQLite clients и `serde_json`
через normal, build и dev dependencies.

[ADR-0222](../../docs/adr/ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md)
закрепляет `settings_registry` как обязательный и exclusive Kernel component.
Блок `settings` фиксирует private Kernel Control Store как source of truth,
authority `operator_managed`/`kernel_managed`, независимую client visibility,
owner-preserving composition, typed scopes и apply modes. `kernel_managed`
никогда не является editable, а managed restart выполняет checkpoint только
когда capability его поддерживает. Блок также запрещает merged cross-owner
mutation, secret values, изменение grants через settings и automatic rollback. Это
declared static policy: фактические schema validation, optimistic concurrency,
desired/effective persistence, hot apply и supervised restart должны быть
доказаны production conformance/integration tests.

[ADR-0223](../../docs/adr/ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md)
фиксирует Vault как отдельный bundled managed process и закрытый набор из пяти
packages. Как только workspace содержит любое имя, owner claim или dependency
этого boundary, Cargo guard требует точные protocol, key-provider, runtime,
SQLCipher-store и macOS Keychain adapter; `vault_service` может объявлять
только canonical runtime. Production packages вне Vault зависят только от
`makosh-vault-protocol`, а Vault не зависит от Kernel, Gateway, modules, NATS
или PostgreSQL clients. Блоки `vault`, `controlStore` и `events` fail-closed
фиксируют storage/key/recovery profile, unlock modes, payload limits,
memory-only single-use leases, TTL, binding к exact revision/runtime/grant
generation и запрет secret material или opaque credential bindings в settings,
Control Store, durable events, NATS, SSE, argv, environment, logs и filesystem
spool. Эти guards проверяют declarations и Cargo graph; отдельный OS process,
HPKE channel, SQLCipher и отсутствие plaintext доказываются только production
conformance/integration tests.

[ADR-0224](../../docs/adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md)
фиксирует отдельный managed Storage Control process и закрытый набор из шести
packages. Блок `storage` резервирует protocol, SQL-free control, runtime,
PostgreSQL, PgBouncer и migration-admission surfaces; production packages вне
Storage могут зависеть только от `makosh-storage-protocol`. SQL client разрешён
только persistence surfaces, а migration AST parser — только canonical
migrations package. Static policy также запрещает Kernel/Storage Control SQL
proxy, module self-migrations, cross-owner business SQL и PgBouncer как
единственную budget boundary. Modules выполняют business SQL напрямую через
PgBouncer, а Storage Control занимается только bootstrap, grants, budgets,
migration admission и readiness. Эти guards доказывают declarations и Cargo
graph, но не OS-level network/socket isolation: без отдельного sandbox и
process conformance suite нельзя утверждать, что same-UID process физически не
может попытаться обойти PgBouncer. Canonical operational summary находится в
[Storage Control Plane](../../docs/architecture/storage-control-plane.md).

[ADR-0225](../../docs/adr/ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md)
отделяет полную конституцию от текущей реализации. Блок `implementation`
разрешает только exact six-package `kernel_recovery_only_v1`, пустой owner
inventory, Kernel components `supervisor`/`core_gateway` и отсутствие внешних
services, managed children, network listener, NATS и business data plane.
Отдельный current-inventory guard начинает требовать весь exact set, как только
в workspace появляется первый production package. Он также запрещает hidden
production files вне registered exact package root, сверяет internal edges по
resolved Cargo package ID и фиксирует direct crate version/features/source.
Блок `phaseGates` оставляет
module control plane, managed launch, NATS, Blob, public Clock, Scheduler,
public client gateway, whole-instance backup и первый owner запрещёнными до
нового ADR и executable evidence.

[ADR-0226](../../docs/adr/ADR-0226-ai-context-acquisition-through-use-case-workflows.md)
фиксирует блок `aiContext`: AI не читает чужие tables/query contracts и не
выполняет cross-owner orchestration. Use-case workflow запрашивает явные owner
contracts и передаёт distinct generated request с common
`AiContextReceiptV1` и concrete use-case context; global fragment union,
opaque payload bytes, `Any`, generic maps, read-all grants, generic Context API
и durable Context projection запрещены. Cargo и SQL negative tests проверяют
эту границу.

[ADR-0353](../../docs/adr/ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md)
фиксирует первый concrete AI use case. Communications передаёт current body
только через event-backed target-bound custody, reply-suggestion остаётся
отдельным workflow, inference принадлежит AI engine, а Ollama protocol —
отдельной integration unit. Ни client content ticket, ни Gateway, ни generic AI
facade не используются.

## Проверки

```sh
make -C backend test
```

- В состав `test` входит проверка согласованности allowlist/blocklist,
  constitutional Kernel components, exact recovery-only implementation,
  fail-closed phase gates, AI context boundary, bootstrap/recovery invariants,
  production paths и SQL ownership.
- `cargo-boundaries-check` использует `cargo metadata --no-deps`, поэтому не
  собирает workspace и не требует загрузки dependencies. Он также проверяет,
  что все production `Cargo.toml` зарегистрированы в workspace, фактический
  production inventory точно соответствует ADR-0225, а SQL-файлы принадлежат
  persistence package своего владельца.
- `test-architecture` запускает positive и negative self-tests из
  `tests/architecture/`.
- `architecture-check` запускает оба production linter.

Virtual clean-room Cargo workspace существует, даже когда в нём ещё нет
production packages. Empty и test-only workspace допустимы; появление любого
production package атомарно требует exact ADR-0225 set. Каждый package обязан
иметь metadata, жить в production или test-only root и проходить полный
dependency check.

## Cargo metadata contract

Каждый workspace package объявляет ровно одну роль, одного владельца и один
тип поверхности:

```toml
[package.metadata.makosh]
role = "domain"
owner = "contacts"
surface = "contract"
```

Допустимые роли и surfaces определены только в `policy.json`. Неизвестная или
отсутствующая роль немедленно ломает guard.

В текущем recovery-only slice Kernel объявляет только реально активные
компоненты:

```toml
[package.metadata.makosh]
role = "core"
owner = "kernel"
surface = "runtime"
components = [
  "supervisor",
  "core_gateway",
]
```

`module_registry`, `capability_router`, `event_hub`, `telemetry_control` и
`settings_registry` остаются в конституционном registry Kernel, но не могут
появиться в текущем Cargo metadata до открытия соответствующей фазы. Unknown
component запрещён всегда; current-inventory guard дополнительно требует exact
phase-specific subset.

Остальные core-owned packages также перечислены по имени и surface в
`kernel.packages`. Сейчас разрешены только port и SQLite adapter ADR-0216:

```toml
# backend/src/kernel/control_store/contract/Cargo.toml
[package.metadata.makosh]
role = "core"
owner = "kernel"
surface = "contract"
```

```toml
# backend/src/kernel/control_store/sqlite/Cargo.toml
[package.metadata.makosh]
role = "core"
owner = "kernel"
surface = "persistence"
```

Новый package с ролью `core`, неверный surface или `rusqlite` вне persistence
ломают guard; расширение списка требует изменения active architecture policy.

Telemetry Collector является отдельным platform runtime:

```toml
[package.metadata.makosh]
role = "platform"
owner = "telemetry"
surface = "runtime"
components = ["telemetry_collector"]
```

## Dependency rules

- domain не зависит от другого domain или integration;
- AI как domain также не зависит от contracts других owners; cross-owner AI
  context собирает только explicit use-case workflow;
- integration зависит из business domains только от exact neutral contract
  units `makosh-communications-ingress` и
  `makosh-communications-attachment-contract`; остальные domain contracts
  запрещены;
- workflow и API используют чужие packages только через `contract`;
- contract не зависит от runtime, implementation или persistence своего
  владельца;
- implementation не зависит от persistence; runtime своего владельца является
  единственной process surface, которая собирает implementation и persistence
  вместе;
- optional `assembly` является build-time leaf своего owner: она может читать
  exact runtime/persistence artifact builders для release composition, но
  runtime/implementation/persistence не могут зависеть от assembly, а Kernel,
  Gateway и другие owners не могут её импортировать;
- core использует только platform/API contracts и не линкует module или
  Telemetry Collector implementations;
- Kernel/Gateway не линкуют даже owner-specific module contracts; module
  discovery использует protocols, descriptors и bundled manifests;
- runtime не зависит от другого runtime, включая runtime того же owner;
- persistence package не зависит от persistence package другого owner;
- aggregate packages `makosh-backend`, `makosh-api`,
  `makosh-worker-runtime`, `makosh-desktop-runtime`, `makosh-schema`,
  `makosh-common` и `makosh-provider-api` запрещены;
- cross-owner dependency на implementation запрещена для normal, build и dev
  edges;
- test support разрешён production package только как dev dependency;
- PostgreSQL client crates и SQL-файлы разрешены только `persistence` surface;
- SQL identifiers используют owner prefix; cross-owner reads, writes и foreign
  keys запрещены;
- Telemetry Collector не зависит от NATS или PostgreSQL clients;
- canonical `makosh-events-protocol` не зависит от broker, persistence или JSON
  implementations и является единственным `platform:events:contract` owner;
- canonical `makosh-runtime-protocol` не зависит от broker, persistence или
  JSON implementations и является единственным
  `platform:runtime_protocol:contract` owner;
- Event Hub, telemetry control и Settings Registry могут быть компонентами
  только Kernel;
- blocked domains и projections запрещены в package metadata, package names,
  production paths и SQL ownership declarations.

## Область source scan

Source scan применяется только к `backend/src` через relative root `src` из
`policy.json`. `references/`, documentation, tests и generated/build
directories не входят в production scan; symlink внутри source запрещён, а не
игнорируется. Текущие legacy frontend и protobuf contracts не объявляются как
clean-room module roots; для их замены потребуется отдельный synchronized
client/contracts cutover и собственный guard.

Baseline-файлы и per-file exceptions не поддерживаются. Изменение allowlist,
blocked projections или Kernel ownership требует принятого ADR и одновременного
изменения policy и negative tests.

Guard анализирует standalone `.sql` migrations и ownership declarations.
Строковые SQL queries внутри Rust дополнительно ограничены тем, что SQL client
dependency разрешена только persistence package; окончательное фактическое
разделение таблиц обеспечивают owner roles/grants из ADR-0202 и их integration
tests. Static policy теперь проверяет declared `runtimeProtocol`/`settings`
invariants, canonical package ownership и Cargo dependencies, но не wire,
digest, registration, authorization, persistence или restart semantics:
production protocol и runtime отсутствуют. Supply-chain invariants ADR-0219 уже
зафиксированы статически, но managed launch остаётся закрытым gate ADR-0225;
signature, tamper и platform-safe spawn semantics должны быть доказаны до его
открытия. SQLite schema/runtime semantics будут проверяться integration/crash
tests recovery-only production slice ADR-0216/ADR-0225.
