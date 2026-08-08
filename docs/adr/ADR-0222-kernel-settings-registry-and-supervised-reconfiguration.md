# ADR-0222: Kernel Settings Registry и supervised reconfiguration

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Частично реализовано: `SettingsSchemaV1`, bounded
structural validation и проверка exact `setting_id`/value-type against schema
существуют в `makosh-runtime-protocol`; private Control Store хранит exact
schema binding, desired protobuf snapshot, независимую effective revision,
explicit apply state и sanitized reason code с optimistic-concurrency fencing.
Schema admission, constraints, owner-specific semantic validation, Gateway API
и runtime apply protocol ещё не созданы.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md).

Уточняется:

- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Уточняет и частично заменяет прежние формулировки ADR-0206 и ADR-0216, по
которым durable module settings должны были оставаться в owner storage. После
этого решения module по-прежнему владеет смыслом и schema своих настроек, но
Kernel владеет единым configuration control plane, revisions, persistence и
supervised application. Business state остаётся у module owner.

## Контекст

Каждый domain, integration, workflow, engine и platform module имеет
конфигурацию. Часть полей меняет владелец через UI, часть в будущем должна
управляться техническими алгоритмами Kernel. Изменение может применяться без
остановки, требовать restart одной capability либо всего module runtime.

Если каждый module самостоятельно хранит и наблюдает settings:

- Kernel не знает, с какой configuration revision запущен process;
- restart может поднять runtime со stale либо частично записанным state;
- UI должен подключаться к каждому process;
- boot/recovery зависит от PostgreSQL и доступности module;
- нет общей optimistic concurrency, audit и failure status;
- domain может начать собирать settings integrations и тем самым нарушить
  ownership/isolation.

Одновременно generic settings service не должен интерпретировать business
semantics, хранить secrets, checkpoints или Scheduler state и становиться
новым shared domain.

## Решение

### Ownership разделён явно

Ответственности распределяются так:

| Ответственность | Владелец |
|---|---|
| stable setting IDs, типы, defaults, constraints и business meaning | declaring module |
| schema artifact и capability association | declaring module |
| schema validation, registry, desired/effective revisions и audit | Kernel Settings Registry |
| durable configuration values | private Kernel Control Store |
| owner authentication и client API | Core Gateway / OwnerAuthority |
| apply/restart/quiesce/drain | Kernel supervisor + module control protocol |
| secret material | Vault, не Settings Registry |
| schedules, job runs и leases | Scheduler, не Settings Registry |
| module/domain/provider operational state | соответствующий module owner |

Domain не импортирует integration settings, не читает их values и не мержит
их со своим configuration document. Каждый module регистрирует собственную
schema напрямую в Kernel. Kernel строит composed catalog view для UI, сохраняя
owner/capability каждого поля.

### Settings Registry является component Kernel

`settings_registry` — обязательный и exclusive Kernel component рядом с
`module_registry`, `capability_router`, `event_hub` и supervisor.

Registry:

- принимает и проверяет settings schema artifacts из verified descriptor;
- хранит exact schema digest и normalized bounded definitions;
- materializes initial values/defaults;
- принимает authorized revisioned mutations;
- вычисляет resolved snapshot одной registration/target;
- orchestrates validation, hot apply или supervised restart;
- хранит desired/effective revisions и sanitized result;
- публикует только metadata изменения, не private values.

Registry не выполняет domain/provider algorithms и не предоставляет modules
generic key-value database.

### SettingsSchemaV1

Schema является отдельным exact Protobuf artifact, который descriptor
ADR-0221 связывает через major/revision/size/SHA-256. Artifact не содержит
runtime values.

`SettingsSchemaV1` содержит упорядоченный список `SettingDefinitionV1`:

```text
setting_id
capability_id?
value_type
mutation_authority
target_scope
default_value?
constraints
apply_mode
client_visibility
fresh_owner_proof_required
kernel_controller_id?
display_metadata
```

Stable `setting_id` уникален внутри module schema и никогда не используется
повторно для другого смысла. Global identity складывается из
`ModuleRegistrationId`, schema major, capability/target scope и setting ID.

`display_metadata` допускает только bounded plain text либо first-party
localization keys. HTML, Markdown, scripts, arbitrary URLs и remote UI bundle
запрещены. Клиент рассматривает metadata внешнего module как untrusted text.

### Типы значений

Первая версия поддерживает закрытый typed union:

- boolean;
- signed/unsigned bounded integer;
- bounded decimal;
- bounded UTF-8 string;
- duration;
- timestamp, только когда это действительно configuration;
- closed enum;
- bounded repeated scalar/enum;
- opaque non-secret resource reference утверждённого kind.

Arbitrary JSON, nested objects, maps, Protobuf `Any`, SQL fragments, scripts и
unbounded lists запрещены. Large text, prompt templates, documents, media и
binary payload не являются settings и должны жить у подходящего owner/blob
boundary.

Начальные limits:

- settings schema artifact — не более 256 KiB;
- не более 512 definitions на module;
- one string value — не более 8192 UTF-8 bytes;
- repeated value — не более 256 elements;
- resolved snapshot одного target — не более 256 KiB;
- duplicate IDs/enum values и unknown required types fail closed.

Hard policy может устанавливать меньший budget для конкретного module kind.

### Два класса mutation authority

Термины `public/private` используются в product UX, но wire contract называет
authority однозначно:

#### `operator_managed`

- изменить setting может только authenticated owner через явно разрешённый
  owner control surface;
- `public` не означает unauthenticated/network-public;
- mutation использует expected revision и actor/device audit;
- Kernel может потребовать fresh operation-bound owner proof независимо от
  более слабой декларации module;
- module не может изменить такое значение самостоятельно.

#### `kernel_managed`

- setting не редактируется обычным client UI или module;
- записать значение может только compiled/allowlisted Kernel controller с
  точным `kernel_controller_id`;
- module может публиковать bounded health/telemetry input, но не выбирает
  алгоритм и не подтверждает собственную запись;
- отсутствие известного controller блокирует definition, а не создаёт generic
  self-tuning hook.

`kernel_managed` резервируется для узких технических policies, например
process-local protective limits и throttling thresholds внутри уже выданных
capabilities. Он не переносит business rules в Kernel. Каждый новый controller
и его inputs требуют явного policy/review; первая реализация может не иметь ни
одного такого поля.

Lifecycle mode `managed`/`external`, registration status, effective GrantSet,
desired module topology и `ManagedLaunchBinding` не являются settings. Это
отдельное Module Registry registration/grant approval state ADR-0215 и
integrity state ADR-0219. Ни `operator_managed`, ни `kernel_managed` setting не
может изменить lifecycle mode, grants, executable binding или topology.

Mutation authority не является sensitivity class. Secret values запрещены в
обоих классах.

`client_visibility` является отдельной закрытой осью и не выводится из
`mutation_authority`:

- `editable` — generic client может читать и менять значение через
  owner-authorized settings method;
- `read_only` — значение либо sanitized representation можно показать owner,
  но client не получает mutation method;
- `hidden` — обычный settings query не возвращает value; diagnostics может
  показать только sanitized state/reason.

Допустимые комбинации определены исчерпывающе:

| `mutation_authority` | `client_visibility` | Client read | Generic client mutation |
|---|---|---|---|
| `operator_managed` | `editable` | typed value | да, с owner authorization и expected revision |
| `operator_managed` | `read_only` | typed value либо schema-defined sanitized representation | нет |
| `operator_managed` | `hidden` | value не возвращается | нет |
| `kernel_managed` | `editable` | invalid schema, fail closed | invalid schema, fail closed |
| `kernel_managed` | `read_only` | authorized typed value либо schema-defined sanitized representation | нет |
| `kernel_managed` | `hidden` | value не возвращается | нет |

Таким образом, `operator_managed` не означает автоматически `editable` или
visible, а `kernel_managed` никогда не бывает `editable`. Visibility не
расширяет authorization и не превращает setting в secret store. Для `hidden`
entry diagnostics может вернуть только sanitized state/reason, но не value.

### Secrets и private data

Password, token, cookie, OAuth secret, provider session, Vault master material
и private key никогда не являются setting value.

Schema может объявить необходимость Vault purpose/capability в descriptor.
Runtime получает scoped credential lease отдельно от settings snapshot по
ADR-0223. Secret reference, opaque credential binding, record ID и Vault
location не являются setting даже тогда, когда сами не раскрывают plaintext.
Связь account/configuration instance с Vault purpose хранит integration owner в
своём operational state, а не Settings Registry.

Settings values, diffs и validation errors не попадают в logs, telemetry,
health, NATS subjects или durable event payload. Audit содержит actor, target,
setting IDs, revisions, outcome и reason code, но не old/new raw values.

### Target scopes

Definition выбирает ровно один допустимый scope:

- `module_registration` — один logical module registration;
- `capability` — конкретная capability этой registration;
- `configuration_instance` — Kernel-generated stable opaque target, например
  independently configured account/runtime slot.

`configuration_instance_id` отличается от ephemeral `runtime_instance_id`,
который меняется при restart. Descriptor не содержит реальные account IDs,
email, phone, username или provider cursor. Связь opaque target с
provider/domain state остаётся внутри owner contract.

Module получает только snapshots своих targets. Cross-owner read/write и
wildcard target запрещены.

### Defaults и materialization

При первой approval schema Kernel:

1. проверяет binding и definitions;
2. создаёт target record;
3. materializes declared defaults в первый desired revision;
4. оставляет required field без default в `blocked_config` до owner input;
5. не запускает affected capability с неполным snapshot.

Default после materialization становится обычным persisted value. Изменение
default в новой module release не переписывает существующий value молча.

Новый optional field при additive schema revision materializes свой default в
новую auditable desired revision. Новый required field без default блокирует
только связанную capability. Существующие values других capabilities остаются
действующими.

### Revision и schema evolution

Settings используют независимые версии:

- schema major — breaking semantic/wire boundary;
- schema revision — additive compatible definitions;
- desired revision — monotonic configuration intent target;
- effective revision — последний snapshot, подтверждённый runtime;
- runtime process generation — lifecycle identity ADR-0215.

Та же schema identity/major/revision с другим SHA-256 является
`settings_schema_revision_collision` и блокируется fail closed. Kernel не
выбирает более новую по времени получения копию.

Внутри одного schema major запрещено менять для существующего setting:

- value type;
- mutation authority;
- target scope;
- capability owner;
- stable meaning;
- apply mode на более слабый;
- constraints так, чтобы persisted valid value стал invalid.

Compatible revision может добавить optional definition, enum value или
ослабить bounded constraint. Removed definition сначала становится deprecated;
его value сохраняется для audit, но не передаётся новому snapshot после
explicit schema activation.

Breaking change требует нового schema major и owner-confirmed migration/reset.
Первая версия не выполняет module-supplied migration code и не угадывает value
conversion. Unknown major/revision/hash даёт `blocked_incompatible`.

### Revisioned mutation

Каждая mutation содержит:

```text
mutation_id
settings_target
expected_desired_revision
schema major/revision/hash
typed field changes
actor/controller authority
```

Kernel проверяет authorization, expected revision, schema/type/constraints и
hard policy. Stale revision возвращает typed conflict с текущей revision, но не
values других полей.

Одна command меняет только один owner target. Composed settings screen может
отправить несколько owner-specific commands, но cross-owner atomic apply не
обещается. UI показывает status каждой операции отдельно.

### Structural и semantic validation

Kernel всегда выполняет structural validation самостоятельно. Module не может
переопределить type, limits, authority или hard policy.

Cross-field/provider-specific semantic validation выполняет owner runtime через
bounded `ValidateSettings` control RPC:

- running module получает candidate snapshot без secrets;
- stopped managed module может быть запущен в restricted configuration phase
  без data-plane grants;
- disconnected external module оставляет revision в `pending_validation`;
- response содержит setting IDs и typed violation codes без raw private values.

Успешная validation не является application. Desired revision сохраняется
отдельно, затем выполняется выбранный apply lifecycle.

### Desired и effective state

Для каждого target Kernel хранит:

- current schema binding;
- `desired_revision` и desired values;
- `effective_revision` и effective values;
- apply state;
- affected capabilities/process generation;
- sanitized validation/apply result.

Допустимые состояния:

```text
current
pending_validation
pending_apply
applying
awaiting_external_restart
blocked_config
```

`desired_revision == effective_revision` требуется для `current`. Persisted
actual process state после Kernel restart считается только hint; Kernel заново
проверяет runtime generation/readiness и подтверждённую revision.

### Apply modes

Каждая definition объявляет минимально безопасный режим:

#### `hot_reload`

Kernel отправляет полный resolved snapshot и revision через `ApplySettings`.
Runtime применяет его атомарно для своей capability и отвечает applied/rejected.
Patch без полного resolved state не является source of truth.

#### `restart_capability`

Разрешён только когда descriptor объявляет capability-local quiesce/drain/start.
Kernel останавливает claims этой capability, drains, передаёт snapshot,
перезапускает capability и проверяет readiness. Соседние capabilities process
могут продолжать работу, если implementation действительно изолировано.

#### `restart_module`

Для `managed` runtime Kernel:

1. quiesces affected intake;
2. drains in-flight work в bounded deadline;
3. выполняет checkpoint только если `ModuleDescriptorV1` объявляет поддержку
   этой operation и lifecycle affected runtime требует её перед restart;
4. stops process;
5. повторно проверяет executable/descriptor/settings artifact binding;
6. запускает новую process generation с полным resolved snapshot;
7. выдаёт data-plane grants только после validation/readiness;
8. записывает effective revision.

Для `external` runtime Kernel не посылает OS signals и не обещает restart.
Он может отправить cooperative control request, но до подтверждения новой
generation/revision состояние остаётся `awaiting_external_restart`.

Kernel hard policy может потребовать более строгий apply mode, но никогда не
ослабляет declared minimum. Auto restart всего Макошь запрещён.

### Change watcher

Kernel является единственным writer Settings Registry. Apply pipeline
запускается после durable Control Store commit конкретной desired revision, а
не через polling PostgreSQL или filesystem watcher.

Каждая revision обрабатывается idempotently один раз на target/process
generation. Crash между store commit и side effect приводит к reconciliation
desired/effective state. Он не создаёт вторую revision и не выполняет
неограниченный restart loop.

### Failure и rollback

Validation/apply/start failure:

- не удаляет desired revision;
- не утверждает, что effective revision изменена;
- переводит только affected capability/runtime в `blocked_config` либо
  `awaiting_external_restart`;
- сохраняет соседние независимые capabilities;
- не выбирает прежний executable, transport, topology или settings revision
  автоматически;
- требует explicit owner correction/revert либо Kernel controller mutation.

Explicit revert является новой monotonic desired revision с прежними typed
values, а не уменьшением revision counter. Restart budgets и crash-loop policy
supervisor применяются как обычно.

### Persistence и boot

Authoritative settings state хранится в private SQLite Kernel Control Store
ADR-0216, потому что Kernel обязан достигать `recovery_only` без PostgreSQL,
PgBouncer, NATS, Vault и modules.

Control Store хранит:

- exact/normalized bounded schema artifact и digest;
- target identities без provider-private labels;
- desired/effective typed values и revisions;
- apply state и sanitized audit;
- controller identity и process generation references.

Он не хранит business state, provider sessions, account records, cursors,
checkpoints, job state, prompts/documents/blobs или secrets.

Settings Registry является конституционной обязанностью Kernel, но не активен в
`kernel_recovery_only_v1`. Только после `module_control_plane_v1` при
trustworthy Control Store settings catalog, current state и authorized
configuration corrections доступны в `recovery_only`; apply может ждать
необходимую infrastructure/module capability. При unavailable/corrupt Control
Store settings mutations запрещены вместе с registry/grant mutations.
PostgreSQL не используется как fallback source of truth.

### Core Gateway и client UI

Desktop и Android получают settings только через один authenticated Core
Gateway. Gateway возвращает composed catalog grouped по product area/module,
но сохраняет для каждого entry:

- owner/registration/capability target;
- `mutation_authority` и `client_visibility`;
- desired/effective revisions;
- apply state;
- sanitized validation metadata.

Уже реализованный browser-local bootstrap передаёт ровно тот минимум, который
нужен до first owner UI: approved module composition, effective capability IDs,
grant epoch, desired/effective revisions, apply state и typed values только
для `editable`/`read_only` definitions. `hidden` entries и non-current values
не сериализуются; non-current module остаётся в composition с
`sections_enabled=false`. Он не заменяет будущий `makosh.settings.v1`.

Первая public client surface — generated platform service
`makosh.settings.v1` с bounded methods:

- `ListSections`;
- `GetSettings`;
- `UpdateOperatorSettings` с expected revision;
- `RevertSettings` как новая monotonic desired revision;
- `RetryApply` без изменения values.

`GetSettings` соблюдает точную authority/visibility matrix выше.
`UpdateOperatorSettings` и `RevertSettings` принимают только
`operator_managed + editable`.
Client method для записи `kernel_managed`, `read_only` или `hidden` entry
отсутствует. Client-safe apply status может приходить через общий replayable
SSE, но raw internal snapshot и settings values через realtime не передаются.

Один экран может визуально объединять Communications, Mail и Telegram, но
domain backend не импортирует integration schema и не получает их values.
Client mutation адресуется конкретному owner target и generated settings
contract.

Authorized `kernel_managed + read_only` query может вернуть typed value либо
schema-defined sanitized representation. `kernel_managed + hidden` не
возвращает value; diagnostics может показать только sanitized state/reason при
соответствующей owner capability.

### Что не является settings

Settings Registry запрещено использовать для:

- inbox/outbox, events и command results;
- provider cursors, checkpoints и synchronization state;
- process health history и restart counters;
- JobSchedule, JobRun, leases, retries и misfire policy ADR-0214;
- domain entities, provider operational records и canonical evidence;
- AI prompts, documents, embeddings, search indexes и large templates;
- credentials, sessions, private keys, secret references и credential
  bindings;
- arbitrary module cache или generic feature flag dump.

Внутреннее state, которое меняется в результате обычной работы module, не
становится `kernel_managed` setting.

### Events и telemetry

Settings lifecycle может публиковать sanitized technical metadata:

- desired revision committed;
- validation passed/failed;
- apply started/completed/blocked;
- restart required/completed;
- controller changed value.

Raw values и diffs не публикуются в NATS, SSE, logs или telemetry. Client
получает authorized values только query RPC. Correlation/actor/audit metadata
следует ADR-0218/0220, когда durable notification действительно требуется.

## Отклонённые варианты

### Domain мержит settings integrations

Отклонено: domain начал бы знать install topology, provider identity и private
configuration соседнего owner. Kernel компонует только read model для UI.

### Каждый module хранит settings самостоятельно

Отклонено: Kernel не может доказать effective revision, безопасно restart-ить
runtime или работать в recovery без module/PostgreSQL.

### PostgreSQL является authoritative settings store

Отклонено: создаёт boot cycle и делает recovery configuration недоступной при
отказе PostgreSQL/PgBouncer.

### Settings являются одним merged JSON document

Отклонено: исчезают ownership, typed validation, per-capability impact,
optimistic concurrency и bounded schema evolution.

### Secret value как скрытое setting field

Отклонено: UI masking не меняет persistence/trust boundary. Secrets остаются в
Vault и выдаются scoped lease.

### Module сам пишет kernel-managed settings

Отклонено: self-tuning превращается в обход owner/Kernel authority и позволяет
process менять собственные resource/security limits.

### Автоматический rollback при failed apply

Отклонено: скрывает desired intent и может запустить stale/incompatible
configuration. Revert является explicit новой revision.

### Произвольный migration callback внутри Kernel

Отклонено: Kernel начал бы исполнять owner code и интерпретировать business
configuration. Breaking migration остаётся explicit и typed.

## Проверка решения

До изменения `Состояние реализации` обязательны tests:

- schema artifact digest совпадает с descriptor/distribution binding;
- same schema identity/version с другим digest fail closed как collision;
- duplicate IDs, oversized schema/snapshot и arbitrary maps отклоняются;
- operator setting меняет только authenticated owner с expected revision;
- kernel-managed setting меняет только known compiled controller;
- module не может изменить оба authority class самостоятельно;
- secrets/private content не принимаются и не попадают в audit/logs/events;
- domain не читает и не мержит integration settings;
- one mutation не изменяет targets нескольких owners;
- default materializes один раз и не переписывается после module update;
- additive schema revision сохраняет существующие values;
- breaking schema/unknown hash остаётся blocked до explicit migration/reset;
- structural validation выполняется до module call;
- stopped managed module validates settings без data-plane grants;
- disconnected external module остаётся pending/awaiting, Kernel не посылает
  process signals;
- hot reload подтверждает effective revision только после typed control
  acknowledgement module;
- restart capability недоступен без declared local lifecycle;
- restart module проходит quiesce/drain, выполняет checkpoint только когда он
  declared и required, затем stop/start/readiness;
- managed runtime без поддержки checkpoint проходит restart после успешного
  bounded drain;
- failed apply не меняет effective revision и не вызывает automatic rollback;
- crash после desired commit reconciles одну и ту же revision idempotently;
- Control Store восстанавливает desired/effective state без PostgreSQL/Vault;
- unavailable Control Store запрещает settings mutations;
- Scheduler state/cursors/checkpoints/business entities не принимаются как
  settings;
- Gateway composed view сохраняет owner target и точно соблюдает все допустимые
  комбинации authority/client visibility;
- settings registry не добавляет dependency Kernel на owner packages.

Static architecture policy пока доказывает только declared ownership,
source-of-truth, authority/apply enums, запрет cross-owner merge/secrets и
package dependency boundaries. SQLite durability, RPC, restart и UI behavior
требуют production integration/conformance tests.

## Последствия

Положительные:

- Kernel всегда знает desired и effective configuration revision runtime;
- modules остаются независимыми и не читают чужие settings;
- configuration доступна для recovery без PostgreSQL/Vault;
- UI получает один catalog без превращения domain в integration aggregator;
- hot reload и restart используют один supervised lifecycle;
- будущая Kernel automation имеет зарезервированную typed authority boundary.

Стоимость:

- Control Store становится владельцем ещё одного boot-critical typed state;
- нужен generic schema/form protocol без arbitrary JSON;
- module runtimes реализуют validation/apply lifecycle;
- cross-owner settings screen не обещает atomic distributed application;
- schema evolution и explicit rollback требуют отдельного UX;
- `kernel_managed` controller нельзя добавлять без hard policy и tests.
