# ADR-0305: Mail-owned composition, drafts, templates and signatures

- Статус: принято
- Дата: 2026-07-27
- Состояние реализации: реализовано полностью. Backend, generated client,
  frontend и live managed Gateway conformance подтверждён атомарно; gate
  `mail_composition_v1` имеет состояние `implemented`.
- Связанные решения: ADR-0204, ADR-0205, ADR-0212, ADR-0213, ADR-0220,
  ADR-0247, ADR-0277, ADR-0281, ADR-0282, ADR-0298

## Контекст

Mail является integration owner, а не Communications domain. Текущий
clean-room Mail runtime уже имеет:

- `mail.delivery.v1` и отдельный `mail.delivery.query.v1`;
- owner-local durable provider delivery;
- `mail.operational.query.v1` для folders, threads и message summaries;
- provider-neutral observation handoff в Communications только через events.

Frontend при этом содержит только transient plain-text send form. Она не
восстанавливает подтверждённые use cases drafts, templates, signatures,
reply/forward/redirect composition и mail-merge preview. Хранение этих
объектов в app state теряет данные при restart, а перенос legacy
Communications tables вернул бы неверного owner и прямую domain/integration
связь.

## Решение

### Ownership и exact routes

Mail получает два независимых public contracts:

```text
mail.composition.command.v1
mail.composition.query.v1
```

Они маршрутизируются:

```text
first-party client
  -> Core Gateway authenticated exact route
  -> Mail managed runtime
  -> Mail-owned PostgreSQL role
  -> Mail composition state
```

Core Gateway проверяет session, exact capability grant, runtime generation и
contract hash. Он не декодирует drafts, templates, signatures, recipients или
body. Kernel/Gateway не импортируют Mail packages. Mail не импортирует
Communications implementation, другой integration, workflow или чужой
storage.

`mail.composition.command.v1` и `mail.composition.query.v1` являются разными
approval/readiness/revoke units. Ни один из них не выдаёт
`mail.delivery.v1`. Сохранение draft не отправляет сообщение, а delivery
receipt не означает provider completion.

### Composition model

Mail owner хранит три независимых operational aggregates.

`MailDraftV1` содержит:

- exact `connection_id` and `draft_id`;
- monotonic revision;
- `new`, `reply`, `reply_all`, `forward` или `redirect` mode;
- optional provider conversation and in-reply-to message identities;
- bounded To, Cc and Bcc recipient lists;
- bounded subject and plain-text body;
- optional Mail-owned template and signature references;
- created/updated timestamps.

`MailTemplateV1` содержит:

- exact connection/template identity and revision;
- bounded name, subject template and plain-text body template;
- declared bounded variable names;
- optional BCP-47-like locale label;
- created/updated timestamps.

`MailSignatureV1` содержит:

- exact connection/signature identity and revision;
- bounded display name and plain-text signature body;
- explicit per-account default flag;
- created/updated timestamps.

HTML, arbitrary metadata maps, provider SDK payloads, credentials, sessions,
raw MIME and attachment bytes запрещены.

### Commands, idempotency and concurrency

Command contract содержит exact create/update/delete operations for drafts,
templates and signatures. Каждая mutation имеет non-empty bounded
`operation_id`.

Mail persistence сохраняет canonical command digest и terminal mutation
receipt. Exact duplicate возвращает исходный receipt. Повторное использование
`operation_id` с другим payload отклоняется.

Create требует отсутствия entity и не принимает expected revision. Update и
delete требуют exact positive expected revision. Stale revision,
cross-account reference и missing entity fail closed; silent last-write-wins
запрещён.

Удаление default signature не выбирает новую подпись автоматически. Upsert
новой default signature атомарно снимает default flag с остальных signatures
этого connection.

### Queries and template preview

Query contract содержит:

- list/get draft;
- list/get template;
- bounded template preview;
- list/get signature.

List queries имеют page size `1..=100` и opaque scoped cursor. Unknown,
altered или wrong-scope cursor отклоняется без silent restart с первой
страницы.

Template syntax ограничена placeholders `{{variable_name}}`. Variable name
содержит только ASCII alphanumeric, `_`, `.` или `-`. Preview принимает
bounded repeated key/value fields, не `map`, JSON или `Any`, и возвращает:

- rendered subject and body;
- missing declared variables;
- unresolved placeholders;
- malformed placeholders;
- explicit `ready` flag.

Preview не отправляет сообщение и не создаёт durable business truth.

### Delivery composition

Frontend Mail composition controller явно собирает delivery request из
сохранённого или transient draft:

```text
Mail draft/template/signature query
  -> Mail-owned local composition controller
  -> mail.delivery.v1
  -> mail.delivery.query.v1
```

Template rendering выполняется только через Mail query contract. Default
signature применяется явно и видимо пользователю; runtime не добавляет
скрытую подпись.

Reply/forward/redirect mode и provider references являются Mail operational
state. Реальная provider mutation по-прежнему выполняется только delivery
capability.

Outbound attachments не входят в этот gate. Их admission остаётся отдельным
bounded MIME + Blob lease + Attachment Security slice. Client filesystem
paths, inbound attachment store lookup и bypass scan запрещены.

### Functional units and SRP

- `makosh-mail-api`: composition language, validation and canonical wire;
- `makosh-mail-persistence`: owner-local schema, optimistic concurrency,
  idempotency and scoped queries;
- `makosh-mail-runtime`: exact route dispatch only;
- generated Mail clients: one client per exact service;
- Mail controllers: draft, template/signature and delivery composition
  responsibilities separated;
- Mail presentation: pure view models and provider experience panels;
- Mail release assembly: immutable artifact composition only.

Runtime не является assembly. Integration не является domain. Общий размер
файла не определяет SRP; причина изменения определяет unit.

## Gate `mail_composition_v1`

Gate становится `implemented` только атомарно при наличии:

1. exact command/query Protobuf services and generated descriptors;
2. independent descriptor capabilities and grant/revoke routing;
3. owner-local additive Storage bundle;
4. idempotent commands and optimistic concurrency negative tests;
5. bounded scoped cursor and template parser negative tests;
6. managed runtime and Core Gateway conformance;
7. generated first-party clients and Mail-owned frontend cutover;
8. restart persistence and cross-account rejection evidence;
9. architecture guards for integration/domain/build-unit boundaries;
10. no secrets, private bodies or recipients in logs, subjects, health or
    sanitized errors.

ADR, schema или UI отдельно не открывают gate.

Gate открыт после managed conformance через Kernel/Core Gateway: команды и
query прошли exact capability routing, cross-account/cursor/idempotency/
optimistic-revision negative paths не остановили runtime, а drafts, templates
и signatures сохранились после fenced successor runtime и повторного readiness.

## Последствия

Drafts, templates and signatures возвращаются как Mail operational features,
не как Communications entities. Core остаётся owner-neutral, cross-owner
communication остаётся event-based, а provider delivery сохраняет отдельный
authority и terminal status. `mail_operational_command_v1`, delayed delivery,
bulk/cross-channel workflows и outbound attachments остаются отдельными
gates.
