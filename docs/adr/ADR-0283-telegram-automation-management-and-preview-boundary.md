# ADR-0283: Telegram automation management and preview boundary

Статус: Принято
Дата: 2026-07-26
Состояние реализации: generated API, pure policy/rendering core, owner-local
persistence migration/repositories, exact runtime routes, descriptor
capabilities, release-assembly storage composition и frontend cutover
реализованы. Disposable PostgreSQL/live managed conformance доказал exact
create/update/query/preview retry, successor process replay, stale runtime
fencing и отсутствие provider/Communications side effects.
`telegram_automation_v1` имеет reconstruction state `implemented`; production
admission остаётся закрытым до `telegram_full_operational_v1`.

Уточняет:

- [ADR-0213: code ownership and module autonomy](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0214: Durable Job Platform](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0240: Telegram clean-room provider boundary](ADR-0240-telegram-clean-room-provider-boundary.md);
- [ADR-0256: owner-declared client RPC route admission](ADR-0256-owner-declared-client-rpc-route-admission.md);
- [ADR-0266: Telegram admission and event-only Communications handoff](ADR-0266-telegram-kernel-admission-and-event-only-communications-handoff.md);
- [ADR-0282: full Communications and Settings reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md).

## Контекст

Historical implementation содержала общий `engines/automation`, общие REST
routes и PostgreSQL tables. Для Telegram подтверждены только следующие
работающие use cases:

- создать или обновить текстовый шаблон;
- получить список шаблонов;
- создать или обновить Telegram policy;
- получить список policies;
- выполнить dry-run для exact account/chat scope;
- проверить declared variables, enabled/expiry state и получить rendered
  preview с hash и audit identity.

Historical `trigger_kind`, `conditions`, `quiet_hours` и
`max_sends_per_hour` сохранялись как generic values, но не управляли реальным
Telegram execution. Historical dry-run не отправлял provider command. Поэтому
копирование общего engine или объявление scheduled/automatic send
восстановленной возможностью создало бы fake implementation.

Automation относится к Telegram operational policy. Это не Communications
domain, не cross-owner workflow, не Kernel behavior и не app setting.

## Решение

### Owner и границы

`telegram_automation_v1` принадлежит Telegram integration owner:

```text
Telegram automation UI
        ↓
generated Telegram automation clients
        ↓
Core Gateway owner-neutral routing
        ↓
Telegram runtime automation ports
        ↓
Telegram automation persistence/core
```

Kernel допускает exact routes, descriptor capabilities, runtime identity,
storage lease и grants, но не декодирует template, policy, variables или
preview. Core Gateway переносит typed payload и session authority, но не
содержит automation methods.

Communications не импортирует Telegram automation contracts, не хранит
templates/policies/previews и не получает dry-run event: dry-run не является
external signal или canonical communication evidence.

### Единицы сборки

Automation добавляет три Telegram-owned Cargo packages:

```text
makosh-telegram-automation-api
makosh-telegram-automation-core
makosh-telegram-automation-persistence
```

Их ответственности:

- `api` — generated Protobuf, exact query/command contracts и schema digests;
- `core` — bounded models, validation, scope evaluation и deterministic
  template rendering; без PostgreSQL, TDLib, Kernel или Communications;
- `persistence` — Telegram-owned schema, optimistic revisions, idempotent
  preview receipts и repository ports.

Существующие units сохраняют собственные причины изменения:

- `makosh-telegram-runtime` адаптирует admitted client routes к automation
  ports и применяет runtime/storage fences;
- `makosh-telegram-assembly` materializes descriptor and storage artifacts;
- frontend `src/integrations/telegram` владеет generated clients, controllers
  и presentation;
- `makosh-telegram-tdlib` не участвует в dry-run, потому что provider command
  не выполняется.

Ни automation package, ни runtime не становятся assembly. Assembly не
исполняет policy и не хранит state.

### Public contracts

Вводятся две независимые capability/route units:

| Capability | Contract | Route |
|---|---|---|
| `telegram.automation.query.v1` | `telegram.automation.query.v1` | `/makosh.telegram.automation.v1.TelegramAutomationQueryService/Query` |
| `telegram.automation.command.v1` | `telegram.automation.command.v1` | `/makosh.telegram.automation.v1.TelegramAutomationCommandService/Execute` |

Query contract содержит только:

- `ListTemplates`;
- `ListPolicies`;
- `GetTemplate`;
- `GetPolicy`;
- `GetPreviewReceipt`.

Command contract содержит только:

- `UpsertTemplate`;
- `UpsertPolicy`;
- `PreviewPolicy`.

Каждая mutation имеет client-generated idempotency key. Upsert использует
`expected_revision`; create требует revision `0`, update — exact current
revision. Retry той же mutation возвращает тот же typed result. Conflicting
payload под тем же idempotency key и stale revision отклоняются.

Delete, automatic execution, provider send, arbitrary script, AI invocation и
generic rule DSL не входят в подтверждённый historical contract и не
добавляются.

### Typed policy

Policy содержит:

- stable `policy_id`;
- exact `template_id`;
- display name;
- `enabled`;
- exact Telegram `account_id`;
- bounded non-empty set exact `provider_chat_id`;
- optional UTC expiry;
- monotonic revision and timestamps.

Generic scope kinds, provider switch, arbitrary JSON conditions, generic maps,
opaque bytes и `Any` запрещены. Telegram chat scope не переносится в
Communications identity.

Template содержит stable identity, display name, bounded body, declared
variable names и revision. Variable name использует ASCII letters, digits and
underscore. Preview variables передаются как bounded repeated typed
`name/value` entries. Duplicate/undeclared/missing variables fail closed.

### Preview semantics

`PreviewPolicy`:

1. validates request bounds and idempotency key;
2. loads policy and referenced template from the same Telegram storage
   revision;
3. requires enabled, not expired policy and exact account/chat scope;
4. deterministically renders declared string variables;
5. rejects unresolved placeholders and output above the contract limit;
6. persists an idempotent Telegram-owned preview receipt containing identities,
   revisions, rendered hash, status and audit metadata;
7. returns rendered text only to the authorized caller.

Rendered text, template bodies and variables are private provider operational
content. Они не попадают в logs, health, errors, subjects или descriptor
metadata. Persistence may store the rendered result required for exact retry,
but sanitized diagnostics expose only identities, revisions, status and hash.

Preview не создаёт TDLib command, Communications observation, canonical message
или Scheduler run.

### Scheduler

Текущий historical capability не выполняет automation автоматически, поэтому
`telegram_automation_v1` не зависит от `scheduler_v1`.

Любое automatic/event/scheduled execution требует нового отдельного gate
`telegram_automation_execution_v1` и нового ADR. Такой ADR обязан определить:

- exact Scheduler `JobKind` and payload revision;
- Telegram owner-local executor;
- rate, quiet-hours, misfire, retry and expiry semantics;
- provider command idempotency and terminal result;
- explicit evidence boundary;
- owner authorization and emergency disable.

Scheduler хранит schedule/run/fence, но никогда не исполняет Telegram policy и
не импортирует Telegram packages.

## Persistence и failure semantics

Automation persistence входит в Telegram owner storage bundle отдельными
objects и migrations. Tables другого owner и cross-owner SQL запрещены.

Обязательны:

- unique owner-local identities;
- optimistic revision checks;
- deterministic ordering and bounded pagination;
- transactionally consistent policy/template references;
- idempotency uniqueness for mutations and previews;
- exact result replay after process restart;
- no state mutation on validation, authorization or render failure.

Runtime crash до commit не создаёт receipt. Crash после commit возвращает
persisted exact result при retry. Gateway timeout не означает failure и не
разрешает клиенту менять idempotency key автоматически.

## Frontend

Frontend живёт только в `frontend/src/integrations/telegram`:

- generated query and command clients are separate transport units;
- controller owns one management/preview use case;
- presentation receives typed view state and emits intents;
- route/app composition only mounts the Telegram-owned surface;
- no handwritten REST, generic Settings store or Communications import.

Automation policy является Telegram operational configuration, поэтому её UI
не переносится в application/platform Settings. Settings page может только
смонтировать provider-owned panel after admission.

## Admission evidence

`telegram_automation_v1` открывается только после:

1. exact package inventory and compile isolation;
2. generated query/command Protobuf with stable schema digests;
3. separate descriptor capabilities and Gateway routes;
4. Telegram-owned StorageBundle and disposable PostgreSQL conformance;
5. optimistic revision, duplicate, restart/replay and stale-fence tests;
6. bounded validation and negative-output privacy tests;
7. frontend generated-client cutover without REST/fallback;
8. clean-room architecture, Rust, Clippy, frontend and bundle gates;
9. live managed route proving create/update/query/preview/retry through the
   admitted runtime.

Полный evidence переводит slice в reconstruction state `implemented`, но не
добавляет Telegram в production owner inventory и не закрывает
`telegram_full_operational_v1`.
