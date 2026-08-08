# ADR-0365: Communication recipient suggestion workflow and source boundary

Статус: Принято

Дата: 2026-07-31

Состояние реализации: implemented. Отдельные contract/core/persistence/runtime/
assembly units, Communications-owned source producer, exact signed release
admission и managed Gateway/SSE conformance реализованы. Live gate проходит
через Vault, Storage Control, PostgreSQL, Blob, NATS, Communications и workflow;
он проверяет restart/replay, revoke, stale source, duplicate/conflict,
generation/grant fences и отсутствие private source bytes в realtime. Legacy
`smart_cc_suggestions`, Explanation и frontend presentation не используются как
доказательство `communication_recipient_suggestion_v1`.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0231](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0253](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0315](ADR-0315-communications-message-body-content-read.md);
- [ADR-0356](ADR-0356-renewable-blob-authority-for-durable-ai-workflows.md);
- [ADR-0364](ADR-0364-communication-explanation-workflow-and-ai-contracts.md).

## Контекст

Legacy `smart_cc_suggestions` читал внутренний `ProjectedMessage` и по трём
наборам строковых маркеров возвращал три английские presentation strings:
accounting/bookkeeping, legal counsel и project stakeholders. Он не находил
конкретные адреса, не проверял Contacts и не вызывал AI. Функция находилась в
одном файле с explanation только по исторической случайности.

Clean-room recipient suggestion отвечает на один вопрос: какую bounded
организационную роль владелец может явно рассмотреть для добавления в recipients
при работе с одним canonical communication item. Это не изменение To/CC, не
поиск Contact, не provider command, не Explanation, не task/note extraction и
не AI inference.

`communications_content_read_v1` нельзя передавать workflow runtime: его ticket
привязан к authenticated client session и `client_blob`. Поэтому reconstruction
требует отдельного Communications-owned source port, а не повторного
использования client ticket или generic content API.

## Решение

### Владельцы и build units

Recipient Suggestion является owner `communication_recipient_suggestion` и
реализуется в пяти независимых workflow units:

- `makosh-communication-recipient-suggestion-api` — generated Start/Get и
  realtime contract;
- `makosh-communication-recipient-suggestion-core` — pure lifecycle,
  deterministic signal evaluation и validation;
- `makosh-communication-recipient-suggestion-persistence` — owner-local
  PostgreSQL state, inbox/outbox и realtime replay;
- `makosh-communication-recipient-suggestion-runtime` — managed workflow
  orchestration;
- `makosh-communication-recipient-suggestion-assembly` — unsigned descriptor,
  settings schema, Storage bundle и release fragment.

Communications отдельно владеет public source contract unit
`makosh-communications-recipient-source-api` и своей runtime implementation.
Ни одна workflow unit не импортирует Communications implementation или storage.
Communications не импортирует workflow implementation.

### Клиентский контракт

Start принимает stable operation ID, canonical source message ID и expected
source revision. Get возвращает stable run/source identity, monotonic state
revision и ordered bounded role candidates.

Каждый candidate содержит только:

- exact role: `accounting_or_bookkeeping`, `legal_counsel` или
  `project_stakeholder`;
- exact rationale: `financial_document_or_payment`,
  `legal_or_contractual_review` или `project_status_or_update`;
- exact source basis `body`;
- bounded confidence basis points.

V1 не возвращает email address, contact/person/organization ID, provider
identity, account ID, prompt, arbitrary label/map или готовую строку интерфейса.
Клиент локализует role и rationale enum. Пустой список является корректным
результатом: workflow не фабрикует recipient, если exact signal отсутствует.

Result является только read candidate. Добавление конкретного адреса требует
явного действия владельца, отдельного Contacts resolution contract и exact
provider compose/delivery command; эти действия не входят в этот gate.

Start/Get проходят через owner-neutral Gateway capability router, terminal
status — через общий replayable SSE. Gateway не читает body, не вычисляет
candidates и не становится recipient facade.

### Communications source flow

```text
Authenticated client Start
↓ client_rpc
communication_recipient_suggestion workflow
↓ durable prepare command
Communications recipient source port
↓ target-bound Blob receipt
communication_recipient_suggestion workflow
↓ pure deterministic evaluation
owner-local result and shared Gateway SSE
```

Prepare command содержит только run/message/revision и exact target runtime
binding. Communications проверяет logical owner, current active canonical
revision и bounded admitted UTF-8 body, затем создаёт target-bound Blob custody.
Body, provider/account identity и participant addresses не попадают в durable
envelopes, status, SSE, diagnostics или errors.

Source port использует distinct command/result contract names, schema hash,
capability и Blob audience. AI source events, Explanation receipts и client
content tickets не переиспользуются.

### Deterministic evaluation V1

Core получает только validated UTF-8 body bytes из exact source receipt,
выполняет Unicode lowercase и применяет фиксированный ordered rule set:

1. `invoice`, `factura` или `payment` создаёт
   `accounting_or_bookkeeping`;
2. `contract`, `legal` или `nda` создаёт `legal_counsel`;
3. `project` вместе с `update` или `status` создаёт
   `project_stakeholder`.

Каждая роль появляется не более одного раза, порядок стабилен, совпадения не
создают Contact или provider truth. Revision V1 намеренно сохраняет
проверяемую product semantics legacy Smart CC без переноса in-process facade и
presentation strings. Новая taxonomy, multilingual rules или Contacts-backed
resolution требуют следующей contract revision или отдельного workflow.

### Persistence, replay и fences

State machine:

```text
accepted
→ preparing_source
→ evaluating
→ ready | rejected
```

Persistence ключуется `(logical_owner_id, operation_id)` и хранит request
fingerprint. Exact duplicate возвращает тот же run/result; conflicting request
с тем же operation ID отклоняется. Inbox проверяет message ID и envelope hash
до mutation. Outbox хранит exact envelope bytes до publish. Terminal result и
client realtime фиксируются атомарно.

Runtime generation, grant epoch, Storage binding, Blob custody и event route
проверяются на каждом внешнем шаге. Recovery выбирает только non-terminal runs
authenticated human owner. Suspend, revoke, stale source revision или stale
coordinate не могут продолжить workflow.

### Kernel agreement

Новые Kernel API, owner-specific imports и business interpretation не вводятся.
Используются существующие signed managed admission, owner-local Storage/Vault
binding, capability-routed `client_rpc`, NATS durable events, target-bound Blob
custody и shared client realtime. Kernel/Gateway не компилируют Recipient
Suggestion schema, source schema или workflow package.

## Phase gate

`communication_recipient_suggestion_v1` становится `implemented` только
атомарно после:

1. пяти отдельных workflow units и exact package metadata;
2. отдельной Communications source contract unit и domain runtime producer;
3. distinct command/result events и target-bound Blob custody;
4. typed deterministic candidate evaluation без presentation strings;
5. owner-local atomic persistence, replay и recovery;
6. signed release admission всех изменённых runtime artifacts;
7. authenticated Gateway Start/Get и replayable SSE;
8. empty, accounting, legal, project, combined, wrong-owner, stale-source,
   request-conflict, duplicate event, restart, revoke, grant/generation fence и
   privacy conformance;
9. architecture, Cargo boundaries, formatting, Clippy, workspace/integration,
   frontend и full pre-push gates.

Skeleton, legacy helper внутри Communications, REST facade, AI/Explanation mode
switch или frontend-only panel не открывают gate.

## Последствия

- Communications остаётся canonical evidence/source owner, а не владельцем
  recipient decision logic.
- Workflow возвращает роль для рассмотрения, но не меняет recipients.
- Ollama и AI Engine не участвуют в этом gate.
- UI получает stable enums и не парсит backend-generated English strings.

## Отклонённые варианты

### Оставить `smart_cc_suggestions` в Communications

Смешивает evidence ownership и product decision logic, сохраняет in-process
facade и presentation strings в domain.

### Добавить Smart CC как режим Explanation

Смешивает причины важности с предложением recipient role и связывает
детерминированную функцию с ненужным AI/provider lifecycle.

### Возвращать конкретные email addresses из body

Создаёт непроверенную identity truth без Contacts owner, provenance и явного
owner decision.

### Передать workflow client content ticket

Нарушает session-bound authority и превращает Gateway/client contract в
межмодульный private-content facade.
