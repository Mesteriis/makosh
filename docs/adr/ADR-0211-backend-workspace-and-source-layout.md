# ADR-0211: Backend workspace и физическая структура исходного кода

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0207: Канонический реестр бизнес-доменов Макошь](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0208: Allowlist разработки доменов и запрет проекций](ADR-0208-domain-development-allowlist-and-projection-freeze.md).

Уточняется:

- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR фиксирует физическое размещение packages, source, migrations,
backend-инструментов и tests. Он не меняет ownership и dependency direction
предыдущих решений.

## Контекст

Clean-room backend строится как набор independently restartable module runtime,
а не как один package. Если разнести Kernel, domains, integrations, services,
scripts и migrations по корню репозитория, физическая структура перестаёт
показывать границу backend и загрязняет общий namespace проекта.

Одновременно tests не должны находиться внутри production source. Test-only
code, fixtures и container harness имеют другой lifecycle, dependency graph и
причины изменения. Их размещение рядом с production modules создаёт риск
случайной production dependency и возвращает test composition в собираемый
продуктовый контур.

Нужна одна каноническая backend-граница, внутри которой находятся workspace,
production packages, linters, infrastructure configuration и tests. Desktop и
Android остаются отдельными клиентскими деревьями и не становятся частью
backend workspace.

## Решение

### Корневая граница backend

Все production source files backend находятся только под `backend/src/`.

Все backend-specific supporting assets находятся только под `backend/`:

- Cargo workspace и Rust tool configuration;
- executable architecture policy;
- backend Makefile и scripts;
- PostgreSQL, PgBouncer, NATS и container configuration;
- architecture, contract, integration, lifecycle и infrastructure tests;
- test fixtures и test-support packages.

В корне репозитория не создаются backend packages или каталоги `kernel/`,
`platform/`, `api/`, `domains/`, `integrations/`, `workflows/`, `engines/`,
`services/`, backend-specific `scripts/`, backend architecture policy или
backend test suites.

Project-wide documentation, repository governance, frontend, Android,
`.github/` и `references/` остаются в корне, потому что не принадлежат backend
workspace.

### Cargo workspace

`backend/Cargo.toml` является virtual workspace и не содержит `[package]`.
Корневой Cargo workspace для backend за пределами `backend/` запрещён.

Production workspace members могут находиться только под `backend/src/`.
Test-only workspace members могут находиться только под
`backend/tests/support/` и обязаны иметь:

```toml
[package.metadata.makosh]
role = "test"
owner = "test"
surface = "test_support"
```

Desktop Tauri package и будущий Android build не являются members backend
Cargo workspace.

Каждый package внутри `backend/src/` сохраняет обычную Cargo layout с
собственным `Cargo.toml` и внутренним `src/`. Поэтому путь вида
`backend/src/domains/tasks/runtime/src/main.rs` является ожидаемым: первый
`src` обозначает production boundary всего backend, второй — source root
конкретного Cargo package.

### Каноническая структура

Нормативная целевая структура:

```text
backend/
├── Cargo.toml
├── Makefile
├── architecture/
│   ├── policy.json
│   └── README.md
├── scripts/
│   ├── check-architecture-policy.mjs
│   ├── check-cargo-boundaries.mjs
│   └── lib/
├── infrastructure/             # когда появится конкретная конфигурация
│   ├── postgres/
│   ├── pgbouncer/
│   ├── nats/
│   └── containers/
├── src/                        # после появления первого production package
│   ├── kernel/
│   ├── platform/
│   ├── api/
│   ├── domains/
│   ├── integrations/
│   ├── workflows/
│   ├── engines/
│   └── services/
└── tests/
    ├── architecture/
    ├── contracts/              # создаётся по фактической потребности
    ├── integration/            # создаётся по фактической потребности
    ├── lifecycle/              # создаётся по фактической потребности
    ├── infrastructure/         # создаётся по фактической потребности
    ├── e2e/                    # создаётся по фактической потребности
    ├── fixtures/               # создаётся по фактической потребности
    └── support/                # только test-support Cargo packages
```

Пустые directories и speculative packages не создаются. Дерево показывает
разрешённые места, а не требует заранее scaffold-ить все возможные modules.
Cargo lockfile, toolchain и runner configuration также создаются только при
реальной необходимости и в любом случае принадлежат `backend/`.

### Kernel

Kernel является package `makosh-kernel`:

```text
backend/src/kernel/
├── Cargo.toml                         # makosh-kernel runtime/composition
├── control_store/
│   ├── contract/                     # makosh-kernel-control-store
│   └── sqlite/                       # makosh-kernel-control-store-sqlite
└── src/
    ├── main.rs                       # composition root only
    ├── cli/                          # production command syntax
    ├── control_store/                # boot lifecycle and offline control
    ├── distribution/                 # signed artifact trust and staging
    ├── identity/
    │   ├── device/
    │   ├── enrollment/
    │   ├── owner/
    │   ├── owner_control/
    │   └── server_pairing/
    ├── infrastructure/               # paths and private filesystem boundary
    ├── modules/
    │   ├── capability/
    │   ├── registration/
    │   └── settings/
    ├── platform/                     # macOS and Vault bindings
    ├── recovery/                     # private recovery IPC and fences
    └── runtime/
        ├── external/
        ├── lifecycle/
        └── managed/
```

`main.rs`, `lib.rs` и `build.rs` остаются маленькими composition roots; плоский
набор файлов с префиксами не заменяет namespace directory. Каждая вложенная
директория Kernel обозначает одного владельца ответственности. Kernel tree не
содержит domain, provider или workflow business logic. Control Store packages
отделены только потому, что port и SQLite adapter имеют разные
dependency/изменение причины; они остаются private owner `kernel` и не являются
module API.

### Семантическая навигация во всём backend

Это правило относится ко всем Cargo packages backend, а не только к Kernel.
Непосредственно в `<package>/src/` могут находиться только `lib.rs` и/или
`main.rs` как composition root. Любой другой Rust source file размещается в
именованной папке ответственности: например `cli/`, `model/`, `time/`,
`providers/`, `tests/recovery/` или `tests/service/`.

Не создаются flat-файлы с package-префиксами вроде `vault_backup.rs` или
`telemetry_retention.rs`, если namespace уже задаётся каталогом владельца.
Небольшой versioned Protobuf directory является исключением: его файлы образуют
один wire-contract и остаются вместе до появления самостоятельного protocol
owner. Исполняемый architecture test проверяет source roots всех production,
development и test-support Cargo packages.

### Platform capabilities

Общие technical contracts располагаются по владельцам:

```text
backend/src/platform/
├── common/
├── runtime_protocol/
├── events/
├── telemetry/
├── storage/
├── vault/
├── blob/
├── clock/
└── scheduler/
```

Каждый independently packaged surface имеет собственный `Cargo.toml` и
metadata. Каталог `platform/` не является dumping ground: business types,
provider DTO и owner-specific policy в нём запрещены.

Public Core Gateway contracts имеют отдельную API boundary:

```text
backend/src/api/gateway/contracts/
```

Gateway transport/composition остаётся responsibility Kernel, но public API
types не помещаются в `platform/` и не зависят от Kernel implementation.

### Domain modules

Разрешённый durable domain использует owner-local topology:

```text
backend/src/domains/<owner>/
├── contracts/
│   ├── Cargo.toml
│   ├── proto/
│   └── src/
├── implementation/
│   ├── Cargo.toml
│   └── src/
├── persistence/
│   ├── Cargo.toml
│   ├── migrations/
│   └── src/
└── runtime/
    ├── Cargo.toml
    └── src/main.rs
```

Surface создаётся только при наличии реальной ответственности. Для
independently restartable durable domain итоговый runtime собирает собственные
implementation и persistence packages, но contract остаётся независимым от
них.

В текущем implementation allowlist допускаются только:

- `communications`;
- `contacts`;
- `organizations`;
- `tasks`;
- `calendar`;
- `documents`;
- `ai`.

Directories `relationships`, `projects`, `obligations`, `decisions`,
`knowledge` и `review` не создаются до отдельного superseding/unblocking ADR.

### Integration modules

Integration использует ту же owner-local физическую форму:

```text
backend/src/integrations/<provider>/
├── contracts/
├── implementation/
├── persistence/
└── runtime/
```

Provider protocol, SDK, auth/session, cursor и neutral evidence mapper остаются
внутри integration owner. Integration не размещает code в domain directory и
не создаёт provider-root business domain.

WhatsApp desktop host bridge остаётся в Tauri client tree, а не переносится в
backend source. Его backend-facing contract может принадлежать integration,
но WebView implementation не становится Rust service внутри backend.

### Workflows, engines и services

- `backend/src/workflows/<workflow_id>/` содержит explicit coordination только
  через public contracts.
- `backend/src/engines/<engine_id>/` содержит pure mechanisms без durable
  product projection state.
- `backend/src/services/` содержит independently managed technical processes,
  включая Telemetry Collector, event relay и storage bootstrap.

Product projection directories `graph`, `timeline`, `search` и `context`
запрещены во всех production branches до разблокировки ADR-0208. Технический
код с похожим словом в имени файла не становится projection; guard проверяет
owner/package identity, а не случайное совпадение текста.

### Contracts и generated code

Owner-specific Protobuf source располагается внутри `contracts/` владельца.
Общий hand-written каталог `contracts/` в корне репозитория не является
источником новых backend contracts.

Generated Rust code создаётся build process и не редактируется вручную.
Generated TypeScript/Kotlin code принадлежит соответствующему client tree и не
копируется в `backend/src`.

### Persistence и migrations

SQL и migrations находятся только в owner persistence package:

```text
backend/src/domains/tasks/persistence/migrations/
backend/src/integrations/telegram/persistence/migrations/
backend/src/services/event_relay/persistence/migrations/
```

Глобальный `backend/migrations/` запрещён. Migration filename не определяет
owner: owner выводится из package metadata и physical package boundary.

### Tests

Весь test code backend находится под `backend/tests/`.

В production tree `backend/src/` запрещены:

- `tests/` directories;
- `*_test.rs` и `test_*.rs`;
- inline `#[cfg(test)] mod tests`;
- test fixtures, snapshots и testcontainers bootstrap;
- test-only feature branches, меняющие production behavior.
- symlink на code или test assets за пределами production tree.

Tests проверяют package через public contract или специально принятый
test-support contract. Production package может использовать test-support
только как `dev-dependency`. Production → test dependency другого вида и
test-support → Kernel composition запрещены.

Test-support packages находятся в `backend/tests/support/`, а не в
`backend/src/platform/` или общем production `crates/` каталоге.

### Backend tooling

Канонические backend-команды после реализации этого ADR запускаются через:

```sh
make -C backend architecture-check
make -C backend test-architecture
make -C backend validate
```

Backend-specific logic не хранится в root Makefile или root scripts. Root CI
может вызвать backend command, но не дублирует его реализацию.

## Executable policy

Architecture guard обязан различать два непересекающихся workspace root:

- production packages: `backend/src/**/Cargo.toml`;
- test-only packages: `backend/tests/support/**/Cargo.toml`.

Guard обязан:

- запускать `cargo metadata --manifest-path backend/Cargo.toml --no-deps`;
- отклонять production package за пределами `backend/src`;
- отклонять test package за пределами `backend/tests/support`;
- отклонять production role внутри test-only root и test role внутри
  production root;
- сканировать production source только из `backend/src`;
- запрещать inline tests и test directories в production source;
- запрещать symlink внутри production source;
- связывать SQL с ближайшим owning persistence package;
- запрещать глобальные migrations и root-level backend packages;
- сохранять существующие dependency, blocked-domain, projection, Kernel и
  Telemetry rules без baseline или per-file exceptions.

Layout guard проверяет обязательные backend-owned paths и отклоняет прежние
root-level `architecture/`, `scripts/`, `tests/architecture/`, `Makefile`,
Cargo workspace и owner directories. Compatibility symlink также считается
нарушением.

## Реализованный переход

Layout migration выполнена одним законченным срезом:

1. создан `backend/Cargo.toml` как virtual workspace без production package;
2. executable policy перенесена в `backend/architecture/`;
3. backend scripts перенесены в `backend/scripts/`;
4. architecture tests перенесены в `backend/tests/architecture/`;
5. backend Makefile перенесён в `backend/`;
6. linters переключены на production/test roots настоящего ADR;
7. documentation links и command entrypoints обновлены;
8. ставшие пустыми root-level backend directories и команды удалены;
9. ADR-0225 разрешил exact six-package recovery-only set, а после отдельного
   evidence — exact `vault_v1` package cut; любой другой production
   package остаётся закрыт phase gate и owner contract.

Dual layout и compatibility symlink запрещены: после среза существует только
новое каноническое расположение.

## Запрещено

- возвращать монолитный `backend/src/lib.rs` или `backend/src/app/`, который
  собирает business domains в одном package;
- создавать backend Cargo package в корне репозитория;
- размещать domain package непосредственно в `backend/src` без owner directory;
- держать общие global migrations или hand-written global contracts;
- помещать test code, fixtures или snapshots в production package;
- создавать пустые packages «на будущее»;
- добавлять compatibility re-export между старым и новым path;
- использовать `references/backend-legacy` как workspace member или source
  root.

## Отклонённые варианты

### Packages верхнего уровня репозитория

Отклонено: `kernel/`, `modules/`, `crates/` и `services/` загрязняют project
root и размывают границу backend относительно desktop, Android, docs и
reference implementation.

### Один Cargo package в `backend/`

Отклонено: возвращает общий compile graph, допускает скрытые implementation
imports и противоречит независимо перезапускаемым module runtime.

### Tests внутри каждого package

Отклонено: test code и fixtures смешиваются с production source, а test-support
начинает восприниматься как часть product package.

### Общие migrations и contracts в корне backend

Отклонено: physical owner становится неочевидным, cross-owner SQL и DTO легче
скрыть за общим каталогом.

### Одновременная поддержка двух layouts

Отклонено: symlinks, forwarding scripts и duplicate Make targets создают
неоднозначный source of truth до первого production package.

## Последствия

Положительные:

- весь backend имеет одну физическую границу;
- owner и surface видны из пути до чтения Cargo metadata;
- tests и production source имеют разные roots и dependency policy;
- migrations и contracts физически принадлежат владельцу;
- project root остаётся пространством всего продукта, а не Rust workspace;
- architecture guard может проверять layout без per-file exceptions.

Отрицательные:

- package paths содержат два уровня `src`;
- backend commands требуют `make -C backend` или явного CI working directory;
- test-only workspace root требует отдельной проверки Cargo membership.

## Проверка решения

ADR считается реализованным только когда одновременно выполнено следующее:

- `backend/Cargo.toml` является единственным backend workspace manifest;
- все production workspace members находятся под `backend/src`;
- все test-only members находятся под `backend/tests/support`;
- `cargo metadata --manifest-path backend/Cargo.toml --no-deps` проходит;
- root не содержит backend packages, backend scripts или backend test suites;
- architecture policy и self-tests находятся под `backend/`;
- negative tests отклоняют package в неправильном root;
- negative tests отклоняют inline/package-local tests в `backend/src`;
- negative tests отклоняют global migrations и SQL вне owner persistence;
- blocked domains и projections отсутствуют в production tree;
- `make -C backend validate` проходит;
- documentation links указывают только на канонические paths;
- legacy reference не входит в Cargo metadata и source scan.

Executable evidence реализации: `cargo metadata` использует только
`backend/Cargo.toml`; architecture policy проверяет layout, source и Cargo
roots; negative self-tests покрывают прежний root layout и test code внутри
production source. Наличие первого production package этим ADR не требуется.
