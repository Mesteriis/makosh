# ADR-0236: Integration owners, protocol adapters и configuration instances

Статус: Предложено
Дата: 2026-07-20
Состояние реализации: Не реализовано. ADR уточняет terminology и granularity
integration boundary. Production inventory integrations остаётся пустым,
`first_owner_v1` этим решением не открывается и первый owner не выбирается.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0225: Первый production slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Связано с:

- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob Platform — opaque references and owner-local metadata](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0233: Scoped local recovery export and PostgreSQL dump](ADR-0233-whole-instance-backup-and-fenced-restore.md).

## Контекст

Термин `integration` сейчас легко смешать как минимум с пятью разными вещами:

- клиентом внешнего SDK;
- wire protocol, например IMAP или POP3;
- внешним provider или сервисом, например Gmail;
- настроенным аккаунтом пользователя;
- запущенным OS-процессом Макошь.

Такое смешение даёт две противоположные ошибки. Если объявлять integration
каждый protocol client, Mail распадается на независимые IMAP, POP3 и SMTP
modules, хотя один пользовательский mail account часто требует согласованной
композиции inbound и outbound capabilities. Если объявлять integration весь
vendor, один Google module начинает агрегировать Mail, Calendar, Contacts и
другие разные operational models.

ADR-0204 уже определяет integration-плагин как владельца provider protocol,
auth/session, cursors, operational state и mapper в neutral evidence. ADR-0212
показывает Mail как один owner graph `api + core + imap + smtp + persistence +
runtime`, а IMAP и SMTP — как protocol-specific adapters. Не определена
достаточно точно сама единица integration, её отличие от configured account и
условие, при котором новый protocol требует нового owner/runtime.

Существующий frontend смешивает vendor и transport в `provider_kind` и
использует generic configuration records. Он остаётся migration/product
evidence и не переносится в clean-room contracts как нормативная модель.

## Решение

### Канонические термины

Макошь различает следующие сущности:

| Термин | Значение | Не является |
|---|---|---|
| **Integration module / integration owner** | Самостоятельный architectural owner одной целостной внешней operational-модели, её contracts, state, lifecycle и neutral evidence mapping; при наличии backend runtime использует `module_kind = integration` | business domain, protocol client или configured account |
| **Protocol adapter** | Owner-local implementation port для конкретного protocol/SDK, например IMAP, POP3, SMTP, Gmail API, TDLib или Zulip HTTP | Kernel registration, отдельная integration или public generic provider API |
| **External protocol client** | Конкретный объект/library client внутри adapter, включая connection pool или SDK handle | архитектурная identity Макошь |
| **External provider/service** | Внешняя система или deployment, например Gmail, iCloud, generic mail server, Telegram или Zulip workspace | автоматический owner boundary или business discriminator |
| **Connector profile** | Closed typed owner setting, выбирающая совместимый adapter behavior в пределах effective GrantSet connection | новый module, capability authority, GrantSet или generic extension mechanism |
| **Configuration instance** | Stable opaque `configuration_instance_id` ADR-0222 для одного независимо настроенного подключения/account slot | `ModuleRegistrationId`, process или внешний account identifier |
| **External principal/account** | Mailbox, user, bot, workspace или другой remote principal за configuration instance | Kernel identity или canonical Макошь business entity |
| **Module registration** | Kernel-owned admission, grants и lifecycle record ADR-0215 | provider account или connection settings |
| **Runtime instance/generation** | Ephemeral identity конкретного module process launch | stable integration или configuration instance |
| **Frontend experience** | Bundled first-party operational surface integration owner | remote plugin code или business domain |

Слово `client` разрешено для concrete protocol implementation, но не
используется как имя integration entity: оно не выражает ownership state,
contracts, evidence mapping и lifecycle.

Отдельный authoritative `IntegrationDefinition` не создаётся. Catalog view
имеет discriminated origin:

- module-backed entry выводится из verified distribution binding, exact
  `ModuleDescriptorV1`, effective grants, observed runtime state и bundled
  frontend surface;
- host-only entry выводится из signed application-bundle surface и explicit
  host-only policy. До отдельного bridge ADR для него не создаются fake
  descriptor, grants, registration или backend runtime state.

Эти объекты сохраняют разные authority по ADR-0221 и не сливаются в generic
record.

Термин `provider-specific operational contract` из ADR-0204 означает contract,
специфичный для integration owner и его внешней operational-модели. Он не
требует, чтобы owner всегда совпадал с коммерческим vendor или одним wire
protocol.

### Единица integration owner

Integration выделяется вокруг **одной cohesive operational-модели и одной
причины изменения**, а не вокруг названия protocol, SDK или vendor.

Один owner сохраняется, когда варианты подключения имеют:

- один operational language и один versioned public contract family;
- общий core state machine и neutral evidence semantics;
- согласованный account/session/cursor lifecycle;
- одну owner-local persistence и outbox/replay boundary;
- одну explicit runtime/host responsibility;
- общую причину изменения с точки зрения пользователя Макошь.

Новая integration требуется, когда необходимы одновременно существенные
независимые границы:

- отдельный operational language, API и invariants;
- собственные auth/session/cursor/persistence semantics;
- отдельно approved/revoked capabilities и resource budgets;
- independently admitted, restarted, fenced и removed runtime либо explicit
  host-bound lifecycle;
- отдельная failure boundary, без которой один owner core превращается в набор
  provider switches, opaque maps или несвязанных state machines.

Новый wire protocol, новый authentication method, новый endpoint или ещё один
account сами по себе не удовлетворяют этому тесту.

Один vendor может иметь несколько integrations, если его продукты имеют разные
operational models. Одна integration может иметь несколько vendor/protocol
adapters, если они реализуют ту же operational-модель. Vendor identity
сохраняется в provenance, но не выбирает behavior business domains.

### Protocol adapters

Protocol adapter является compile-time implementation package своего owner и
зависит от integration core port. Integration core не зависит от concrete SDK:

```text
integration core ← protocol adapter
integration core ← persistence adapter
api + core + selected adapters + persistence ← integration runtime
```

Adapter:

- переводит protocol/SDK types в owner core types и обратно;
- владеет timeout, bounded retry, rate-limit и protocol error mapping;
- не объявляет собственный `ModuleDescriptorV1` и не регистрируется в Kernel;
- не получает grants или credentials в обход owner runtime;
- не публикует provider payload напрямую в business/context domain;
- не импортируется другим integration owner;
- не загружается динамически во время работы Макошь.

Runtime может включать несколько bundled adapters одного owner. Supported
connector profiles и связь profile requirements с capability IDs принадлежат
versioned operational contract/typed settings semantics. Descriptor объявляет
только capability IDs, contract references и settings schema reference по
ADR-0221. Settings выбирают поведение только в пределах effective GrantSet и не
изменяют его. Произвольный adapter name, dynamic library, URL или generic JSON
config не является extension mechanism.

Profile, которому нужна ещё не declared или не approved capability, не
становится effective. Undeclared capability сначала требует нового descriptor
revision; declared, но не approved capability — отдельного owner approval с
повышением grant epoch. Settings mutation сама не запрашивает, не сужает и не
расширяет GrantSet.

Adapter создаётся как отдельный package только при реальной причине изменения,
а не заранее для симметрии. Small cohesive implementation может оставаться
внутри owner package до появления независимой SDK/protocol/test boundary.

### Mail как определяющий пример

Mail является одной integration operational-моделью. Начальная логическая
топология выглядит так:

```text
Mail integration owner
├── operational API, core state и neutral evidence mapper
├── inbound adapters
│   ├── IMAP
│   ├── POP3, если будет реализован
│   └── Gmail API, если используется для mail semantics
├── outbound adapters
│   ├── SMTP
│   └── Gmail API, если используется для send/draft semantics
├── owner-local persistence/outbox
└── one registered Mail runtime boundary
```

Один configuration instance может использовать согласованный connector profile:

```text
generic mailbox A = IMAP + SMTP
legacy mailbox B  = POP3 + SMTP
Gmail mailbox C   = Gmail API
```

IMAP и POP3 являются альтернативными inbound adapters, а SMTP — complementary
outbound adapter. POP3 не притворяется полным эквивалентом IMAP: folders,
server-side flags, incremental synchronization и remote retention выражаются
через честные capability differences. Отсутствующая capability не скрывается
в `metadata` и не эмулируется silent fallback.

Gmail API остаётся Mail adapter, пока реализует Mail operational contract. Если
будущая Gmail experience потребует самостоятельного operational language,
state, lifecycle, grants и независимого runtime, это будет отдельное
owner-boundary решение, а не автоматическое следствие названия Gmail.

POP3 support сейчас не реализован и этим ADR не обещается. Его добавление
потребует отдельного implementation slice, exact capability contract и tests.

### Configuration instance и external account

Configuration instance представляет локально настроенное подключение, а не сам
external account. Между ними не предполагается скрытая identity или обязательная
cardinality 1:1: duplicate/reconnect policy принадлежит exact owner contract.
Одна configuration instance может координировать несколько protocol endpoints
одного operational owner, например IMAP и SMTP.

Rebind или recreate connection обязан либо сохранить explicit stable external
source lineage для observation deduplication, либо создать новую source identity
с observable duplicate-handling policy. Тихое переиспользование или разрыв
source identity запрещены; exact rule фиксирует первый owner ADR.

Kernel знает только opaque `configuration_instance_id`, target scope, settings
revision и выданные scopes. Integration owner хранит связь этого opaque target
с:

- external principal/source identity;
- effective settings target/revision без копирования authoritative config;
- operational account metadata;
- stable Vault purpose association и sanitized credential status без secret
  reference, record/lease ID или Vault location;
- cursors, checkpoints и synchronization state;
- rate-limit/retry state и account-local resource budgets;
- operational projection, outbox/spool и replay fences;
- sanitized account lifecycle/readiness reason.

Точная wire schema connection/account record и его enum lifecycle принадлежат
первому owner ADR. Этот документ фиксирует boundary, но не выдумывает общий
`IntegrationAccountV1` для всех providers.

### Process и account isolation

V1 default topology для bundled backend integration — один independently
managed runtime process на approved integration registration. Он может
multiplex bounded configuration instances одного owner. Новый account или новый
protocol adapter не создаёт отдельный Kernel process автоматически.

Изоляция разделяется по уровням:

| Уровень | Boundary |
|---|---|
| Code/dependencies | protocol adapter package внутри одного integration owner |
| Kernel admission и grants | module registration и capability |
| Process crash/restart | runtime instance/generation integration owner |
| Account configuration/state | configuration instance в owner-local persistence |
| Credentials | Vault purpose/lease, scoped к configuration instance и runtime audience |
| Provider operations | connection/account scope, bounded concurrency и typed fences |
| Business truth | neutral evidence owner и последующий Review/workflow/domain path |

Auth failure, expired credentials, provider throttling или invalid cursor одной
configuration instance делают degraded/blocked только эту connection, если
runtime core остаётся healthy. Crash runtime затрагивает все обслуживаемые им
configuration instances, но не соседние integrations и не уже сохранённое
canonical evidence.

Configuration-instance scope является логической authorization/state boundary,
а не sandbox от скомпрометированного owner runtime. Один multiplexed process
может последовательно получить leases нескольких разрешённых connections.
Confidentiality между взаимно недоверенными connections требует отдельной
per-account process/registration topology и не заявляется этим ADR.

Process-per-account, adapter helper process или несколько одновременно
`approved` и data-plane-capable registrations одного owner в production topology
допустимы только после отдельного решения с доказанной
security/failure/resource причиной, explicit supervision, identity, grants и
state fencing. Это не ограничивает открытое `pending` discovery ADR-0215. Новая
production topology не выводится автоматически из account count или protocol
name.

Host-only WhatsApp остаётся явным исключением ADR-0212: fake backend package,
registration или runtime ради симметрии не создаются. Versioned host bridge и
его identity/supervision boundary требуют отдельного ADR.

### Независимые оси lifecycle

Один generic `status` запрещён. Как минимум разделяются:

1. registration/admission state ADR-0215;
2. capability grant/readiness state ADR-0221;
3. runtime process health и generation;
4. settings desired/effective/apply state ADR-0222;
5. configuration instance auth/connection lifecycle;
6. synchronization cursor/checkpoint/replay state.

`approved` не означает `ready`, healthy process не означает authenticated
account, а successful settings apply не означает completed provider sync.

Pause/retire configuration instance сначала является owner-local intent и
блокирует новые operations внутри integration runtime. Текущие Vault/Scheduler
contracts не доказывают account-local revoke уже выданного lease/job только по
этому state. Первый owner ADR обязан определить monotonic connection fence и
его проверку всеми consumers либо использовать более широкий grant-epoch/runtime
restart fence. До подтверждения fencing connection нельзя сообщать как fully
retired/revoked. Это не заменяет отдельные
`suspended`/`revoked` states module registration ADR-0215.

Удаление локальной connection не удаляет remote data и не уничтожает canonical
evidence. Destructive remote action требует отдельной typed command, explicit
capability и owner-authorized intent.

### Settings, secrets и operational state

| Данные | Владелец |
|---|---|
| Connector profile и non-secret operator configuration | typed schema integration owner + Kernel Settings Registry |
| Desired/effective configuration revisions | private Kernel Control Store |
| Password, OAuth refresh credential, API key и bounded credential blob | Vault |
| Link configuration instance → stable Vault purpose и sanitized credential status | private integration operational state; без secret reference, record/lease ID или Vault location |
| Large/high-churn SDK session store | encrypted private integration store с `SessionStoreKeyLease` |
| External account identity, cursors, checkpoints, retries | private integration operational state |
| Provider operational projection | integration owner |
| Neutral canonical evidence | exact allowlisted downstream owner; сейчас только Communications/evidence owner |
| Message/media bytes | Blob boundary через opaque references, если owner contract это разрешает |

Settings не содержат secret reference, credential binding, provider session,
cursor или synchronization state. Kernel Module Registry не хранит private
account identifiers, credentials или provider operational records.

### Contracts и routing

Protocol adapters не имеют public client contract. Public operational contract
принадлежит integration owner и выражает его честную operational-модель.
Frontend experience обращается к нему только через generated client и Core
Gateway.

Inbound adapter передаёт observation в integration core. Core orchestrates
owner persistence через свой port. Когда provider state/cursor и evidence
observation должны commit-иться вместе, owner persistence adapter атомарно
пишет их с outbox. Canonical evidence owner сохраняет observation отдельной
idempotent inbox transaction. Текущая executable policy разрешает integrations
только `makosh-communications-ingress`; Calendar и другие non-communications
integrations требуют отдельного ingress ADR и изменения policy, а не exception.
Provider/protocol identity может сохраняться как provenance, но
business/context domain не branch-ится по adapter или provider.

Outbound request адресует разрешённую configuration instance и owner
capability. Integration выбирает только connector profile, уже связанный с этой
connection. Automatic IMAP ↔ POP3, Gmail API ↔ IMAP, account или endpoint
fallback запрещён: смена connector profile является explicit settings/account
lifecycle operation с validation, fencing и observable outcome.

Frontend catalog показывает module-backed и host-only integration types из
соответствующего verified catalog origin. Настроенные connections запрашиваются
через operational contract integration owner. Kernel registry не становится
generic provider account catalog.

### Backup и restore classification

ADR-0233 не включает provider state и OS-managed profiles в текущие recovery
artifacts. До открытия первого owner его ADR обязан отдельно классифицировать:

- configuration records и effective settings revisions;
- external source lineage, cursors, checkpoints и outbox/inbox state;
- operational projections и rebuildable caches;
- encrypted high-churn session stores;
- host-only OS profiles, которые нельзя безопасно экспортировать.

Restore обязан менять runtime/connection fences, инвалидировать leases и,
где они существуют, in-flight jobs старой generation, а также выполнять
explicit provider reconciliation до возобновления side effects. Формат backup,
inclusion policy и возможность
восстановления конкретного provider session остаются first-owner/
`whole_instance_backup_v1` решениями; silent empty-session fallback запрещён.

## Не определяет

Этот ADR не определяет:

- какой integration owner будет первым production owner;
- exact Mail, Telegram, Zulip или другого owner contract;
- wire schema account/connection lifecycle;
- exact connector profile wire representation;
- duplicate/rebind и stable external source lineage policy;
- onboarding UI, OAuth/QR callback flows и provider-specific auth policy;
- cross-owner sharing одного vendor OAuth grant;
- process sharding или per-account helper topology;
- exact provider-state backup/restore format и inclusion policy;
- POP3 implementation или remote deletion defaults;
- schema/migrations и package inventory первого owner.

Каждый такой production slice требует собственного owner ADR и evidence по
`first_owner_v1`.

## Запрещено

- считать каждый protocol/SDK client отдельной integration автоматически;
- считать каждый vendor одним integration owner автоматически;
- регистрировать protocol adapter в Kernel отдельно от owner runtime;
- создавать module registration или process на каждый account без отдельного
  topology decision;
- один generic `Provider`, `Connector` или `IntegrationAccount` contract с
  arbitrary JSON/metadata для всех owners;
- provider/adapter switches внутри business/context domains;
- хранить external account IDs, credentials, sessions или cursors в descriptor,
  Module Registry или generic Settings records;
- передавать concrete adapter SDK types в operational public contract, shared UI
  или neutral evidence;
- silent adapter/account/endpoint fallback;
- динамически загружать adapter code или remote frontend surface;
- удалять remote state как побочный эффект local disconnect/remove.

## Отклонённые варианты

### IMAP, POP3 и SMTP как независимые integrations

Отклонено как default: они делят Mail operational language и часто составляют
одну connection. Такая модель дублирует account state, auth, UI, evidence
mapping и routing, а SMTP ошибочно выглядит самостоятельным продуктом.

### Один integration owner на коммерческого vendor

Отклонено: vendor может объединять несвязанные Mail, Calendar, Contacts и file
products. Corporate boundary не является bounded context Макошь.

### Один process на каждый configured account

Отклонено как V1 default: это смешивает operational state isolation с process
identity, умножает registrations/grants/updates и не имеет доказанной общей
необходимости. Такая topology остаётся возможным отдельным решением для
конкретного unsafe SDK или failure model.

### Protocol adapters как dynamic plugins

Отклонено: Макошь не имеет plugin store/sandbox/update model для remote code.
Adapters поставляются внутри exact verified owner runtime executable текущей
release.

### Один общий provider abstraction для всех integrations

Отклонено: он либо теряет реальные operational semantics, либо превращается в
provider enum и opaque maps. Общей остаётся архитектурная форма owner boundary,
а не универсальный runtime API.

## Последствия

Положительные:

- Mail остаётся cohesive integration, а IMAP/POP3/SMTP получают честные
  capability differences;
- новый adapter не увеличивает Kernel registrations и processes без причины;
- account operations и secret leases scoped по configuration instance без
  ложной process-level confidentiality;
- vendor и protocol details не протекают в business truth;
- критерий создания нового integration owner становится проверяемым;
- frontend может различать integration type, configured connection и runtime
  health без одного ambiguous status.

Отрицательные:

- crash одного owner runtime затрагивает все его multiplexed connections;
- integration core должен поддерживать несколько connector capability profiles;
- connection lifecycle, runtime lifecycle и settings lifecycle требуют разных
  contracts и UI states;
- provider-specific feature, не помещающаяся честно в owner operational model,
  потребует отдельного boundary decision, а не generic metadata shortcut.

## Проверка решения

До принятия и реализации первого integration owner должны существовать:

- architecture test, где owner runtime собирает core, API, persistence и только
  свои protocol adapters без Kernel/Communications implementation dependency;
- negative test, запрещающий отдельную adapter registration и cross-owner SDK
  dependency;
- descriptor/settings tests без external account identifiers, secret refs,
  cursors и generic maps;
- account isolation tests: auth/cursor/rate-limit failure одной configuration
  instance не меняет state соседней;
- grant/Vault tests с exact configuration instance, runtime generation и grant
  epoch fencing;
- connection retire test с доказанным account-local fence либо explicit broader
  grant/runtime fencing до terminal completion;
- contract tests каждого adapter на deterministic operational mapping, stable
  neutral observation identity через rebind/recreate, provenance, duplicate
  delivery и bounded retry;
- capability tests каждого adapter/profile, входящего в exact first-owner
  inventory, без silent fallback; отсутствующий POP3 не реализуется тестовым
  scaffold и должен отсутствовать в advertised capabilities;
- runtime crash/restart test, сохраняющий owner state/outbox и доступность
  canonical evidence;
- removal test, доказывающий отсутствие implicit remote delete;
- backup classification и restore tests только для connection state, cursors,
  outbox, encrypted session stores или host-only profiles, которые входят в
  exact first-owner inventory, с invalidation старых fences;
- exact first-owner packages, contracts, `StorageBundleV1`, capabilities,
  dependencies, signed release binding и live process evidence по ADR-0225.

Текущий architecture self-test уже проверяет допустимую форму Mail owner с
`imap` и `smtp` adapter packages. Это evidence package boundary, но не
реализация Mail runtime, POP3 support или открытие `first_owner_v1`.
