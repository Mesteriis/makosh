# ADR-0364: Communication Explanation workflow and AI contracts

Статус: Принято

Дата: 2026-07-31

Состояние реализации: implemented. Пять отдельных workflow units, distinct
Communications source events, owner-local persistence, AI Engine request и
Ollama provider contract допущены как signed managed runtime graph. Реальный
Ollama process подтверждён тестом
`managed_communication_explanation_completes_real_provider_through_gateway_sse`:
authenticated Gateway Start достигает workflow только через public capability,
private source передаётся target-bound Blob, terminal result приходит через
replayable SSE и затем читается через Get. Отдельный managed contour
`managed_communication_explanation_reaches_ai_and_replays_through_gateway_sse`
покрывает wrong-owner, stale-source, request-conflict, malformed/duplicate
reasons, provider failure, restart replay, revoke, grant/generation fences и
отсутствие private content в durable/realtime surfaces. Legacy
`ExplainMessage`, Summary/Translation и frontend presentation не используются
как facade или доказательство `communication_explanation_v1`.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0226](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0253](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0353](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md);
- [ADR-0356](ADR-0356-renewable-blob-authority-for-durable-ai-workflows.md);
- [ADR-0357](ADR-0357-canonical-communication-subject-and-typed-reply-source-content.md);
- [ADR-0362](ADR-0362-communication-summary-workflow-and-ai-contracts.md);
- [ADR-0363](ADR-0363-communication-translation-workflow-and-ai-contracts.md).

## Контекст

Legacy `explain_importance` читал внутренний `ProjectedMessage`, смешивал score и
поиск строковых маркеров, а затем возвращал массив английских строк. В том же
файле находился `smart_cc_suggestions`, хотя объяснение важности и предложение
получателей имеют разные причины изменения и отдельные reconstruction gates.

Clean-room Explanation покрывает только вопрос: почему один canonical
communication item может требовать внимания владельца. Это не summary, generic
message analysis, recipient suggestion, task/note extraction, finance/legal
verdict и не изменение canonical Communications truth.

## Решение

### Владельцы и build units

Explanation реализуется отдельным owner `communication_explanation` в пяти
units:

- `makosh-communication-explanation-api` — generated Start/Get/realtime
  contract;
- `makosh-communication-explanation-core` — pure lifecycle и validation;
- `makosh-communication-explanation-persistence` — owner-local PostgreSQL
  state, inbox/outbox и realtime replay;
- `makosh-communication-explanation-runtime` — managed workflow orchestration;
- `makosh-communication-explanation-assembly` — unsigned descriptor, settings
  schema, Storage bundle и release fragment.

Workflow не принадлежит Communications, AI Engine или Ollama и не импортирует
их implementation/storage packages. Communications, AI Engine и Ollama получают
только собственные exact contract/runtime extensions.

### Клиентский контракт

V1 принимает stable operation ID, canonical source message ID и expected source
revision. Result содержит stable run/source identity, monotonic state revision,
bounded ordered reason candidates, completeness, aggregate confidence и typed
terminal error.

Каждый reason candidate имеет только:

- exact kind: `urgency`, `financial_attention`, `legal_or_contractual`,
  `reply_requested`, `deadline`, `attachment_reference`, `marketing_or_bulk`
  или `other_attention`;
- bounded owner-visible explanation UTF-8;
- exact source basis: `subject`, `body`, `canonical_metadata` или `combined`;
- bounded confidence.

Пустой список означает, что specific attention signals не подтверждены. V1 не
возвращает recipient suggestions, task/note candidates, provider/model identity,
prompt, endpoint, arbitrary labels/maps или finance/legal verdict. Explanation
является candidate/read result и не изменяет Communications или Review state.

Start/Get идут через owner-neutral Gateway capability router, terminal status —
через общий replayable SSE. Gateway не читает source content, не вычисляет
reasons и не становится explanation facade.

### Source flow

```text
Authenticated client Start
↓ client_rpc
communication_explanation workflow
↓ distinct durable command/event
Communications explanation source port
↓ target-bound Blob receipt
communication_explanation workflow
↓ Blob custody transfer to exact AI Engine audience
AI Engine explanation request_rpc
```

Communications проверяет logical owner, active/current canonical revision и
bounded sender/subject/body. Private content существует только внутри
target-bound Blob. Durable envelopes, inbox/outbox, status, SSE, diagnostics и
errors не содержат private source bytes.

Summary/Translation source events и Blob audience не переиспользуются:
Explanation имеет собственные event names, schema hashes, target capability и
receipts. Client content ticket также не используется, потому что он связан с
client session, а workflow является отдельным managed recipient.

### AI и provider contracts

AI Engine предоставляет capability `ai.explanation.request.v1` с distinct
`CommunicationExplanationInferenceRequestV1` и result. Request содержит
`AiContextReceiptV1`, explanation-specific private source receipt, fixed reason
count/text budgets, local-only egress policy и authenticated logical owner. AI
сохраняет только receipts, lifecycle и bounded candidate result.

AI Engine вызывает capability `ai.provider.explain.v1` через Kernel
`request_rpc`. Ollama Integration использует fixed importance-explanation
instruction и закрытую JSON Schema с exact reason kind/source-basis enums.
Caller не выбирает provider, model, endpoint, prompt или arbitrary taxonomy.
Ollama остаётся concrete integration и не становится частью Communications,
Explanation workflow или AI Engine.

### Persistence, replay и fences

Workflow state machine:

```text
accepted
→ preparing_source
→ awaiting_inference
→ ready | rejected
```

Persistence ключуется `(logical_owner_id, operation_id)` и хранит request
fingerprint. Exact duplicate возвращает тот же run/result; conflicting request
с тем же operation ID отклоняется. Inbox проверяет message ID и envelope hash
до mutation. Outbox хранит exact envelope bytes до publish. Terminal result и
realtime event фиксируются атомарно.

Recovery выбирает только non-terminal runs текущего authenticated owner.
Runtime generation, grant epoch, Storage binding, Blob custody и request route
проверяются на каждом внешнем шаге. Suspend/revoke или stale coordinate не
может продолжить Explanation run.

### Kernel agreement

Новые Kernel API и owner-specific imports не вводятся. Используются только
существующие signed managed admission, owner-local Storage/Vault binding,
capability-routed `client_rpc`/`request_rpc`, NATS durable events, target-bound
Blob custody и shared client realtime. Kernel/Gateway не компилируют
Explanation schema или workflow package.

## Phase gate

`communication_explanation_v1` становится `implemented` только атомарно после:

1. пяти отдельных workflow units и exact package metadata;
2. distinct Communications explanation source events и target-bound Blob;
3. distinct AI Engine и provider explanation request/result contracts;
4. owner-local atomic persistence, replay и recovery;
5. signed release admission всех изменённых runtime artifacts;
6. authenticated Gateway Start/Get и replayable SSE;
7. live managed positive contour через настоящий Ollama process;
8. wrong-owner, stale-source, request-conflict, malformed/duplicate reasons,
   provider failure, restart, revoke, grant/generation fence и privacy
   negatives;
9. architecture, Cargo boundaries, formatting, Clippy, workspace/integration,
   frontend и full pre-push gates.

Skeleton, legacy heuristic copy inside Communications, free-form model answer,
Summary mode switch, REST facade или frontend-only panel не открывают gate.

## Последствия

- Communications остаётся source evidence owner, а не AI orchestrator.
- Smart CC остаётся отдельным `communication_recipient_suggestion_v1`.
- Finance/legal classification здесь является attention candidate, а не
  authoritative business verdict.
- UI локализует exact reason kinds, не парсит provider-generated taxonomy.

## Отклонённые варианты

### Вернуть `ExplainMessage` в Communications

Смешивает evidence ownership, inference orchestration и presentation strings.

### Добавить `operation = explain` в Summary

Смешивает output semantics, reason taxonomy, grants и lifecycle двух workflows.

### Объединить Explanation и Smart CC

Предложение получателей является отдельным decision/candidate flow и не
следует автоматически из reasons важности.

### Разрешить free-form provider response

Создаёт неограниченную taxonomy, ломает локализацию и затрудняет replay/audit.
