# ADR-0362: Communication Summary workflow and AI contracts

Статус: Принято

Дата: 2026-07-31

Состояние реализации: implemented. Gate `communication_summary_v1` закрыт
атомарно: distinct contracts, Communications source events, AI Engine и Ollama
summary paths, пять отдельных workflow units, owner-local persistence/replay,
signed release artifacts, authenticated Gateway Start/Get, replayable SSE,
managed negative/restart/revoke/fencing contour и положительный live contour
через настоящий Ollama process доказаны независимо от Reply Suggestion.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0226](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0315](ADR-0315-communications-message-body-content-read.md);
- [ADR-0339](ADR-0339-capability-routed-module-request-rpc.md);
- [ADR-0353](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md);
- [ADR-0356](ADR-0356-renewable-blob-authority-for-durable-ai-workflows.md);
- [ADR-0357](ADR-0357-canonical-communication-subject-and-typed-reply-source-content.md).

## Контекст

Clean-room inventory требует отдельный `communication_summary` workflow.
Существующий Reply Suggestion доказывает transport и custody primitives, но не
даёт права переиспользовать reply-specific messages, prompts, persistence или
provider operation как summary implementation.

Historical email intelligence объединял TL;DR, action items, deadlines, risks,
persona candidates и promotion effects. Эта форма не переносится: task/note
candidate extraction являются отдельными inventory gates, а summary не создаёт
durable business truth и ничего не продвигает в другие domains.

## Решение

### Владельцы и build units

Summary реализуется отдельным owner `communication_summary` в пяти units:

- `makosh-communication-summary-api` — generated Start/Get/realtime contract;
- `makosh-communication-summary-core` — pure lifecycle и validation;
- `makosh-communication-summary-persistence` — owner-local PostgreSQL state,
  inbox/outbox и realtime replay;
- `makosh-communication-summary-runtime` — managed workflow orchestration;
- `makosh-communication-summary-assembly` — unsigned descriptor, settings
  schema, Storage bundle и release fragment.

Ни одна из этих units не принадлежит Communications, AI или Ollama. Workflow
не импортирует их implementations или storage.

Communications расширяет существующий AI source contract отдельными exact
summary command/result/content messages. Общая внутренняя materialization logic
может быть переиспользована внутри Communications, но reply и summary durable
event names, schema hashes, target capability и Blob audience различны.

AI Engine расширяет public `makosh-ai-contracts` отдельными summary
request/result messages и предоставляет отдельную capability
`ai.summary.request.v1`. Ollama Integration предоставляет отдельную capability
`ai.provider.summarize.v1`. Общие engine/provider runtimes могут обслуживать
несколько exact capabilities, но не получают generic `execute(any)` или
payload-selected operation.

### Клиентский контракт

V1 summarises ровно одно canonical communication evidence item:

```protobuf
message StartCommunicationSummaryRequestV1 {
  uint32 protocol_major = 1;
  bytes operation_id = 2;
  bytes source_message_id = 3;
  uint64 expected_source_revision = 4;
  CommunicationSummaryLanguageV1 language = 5;
  CommunicationSummaryLengthV1 length = 6;
}
```

Generated result содержит только:

- stable run and source identity;
- expected source revision and monotonic state revision;
- bounded UTF-8 `summary_utf8`;
- resolved language and length;
- completeness and bounded confidence;
- typed terminal error.

V1 не возвращает action items, tasks, notes, deadlines, recipients, personas,
organizations, provider/model identity, prompt, endpoint или arbitrary JSON.

Start/Get идут через existing owner-neutral Gateway capability router. Terminal
status идёт через общий replayable SSE. Gateway не читает source body, не
вызывает AI и не является summary facade.

### Source flow

```text
Authenticated client Start
↓ client_rpc
communication_summary workflow
↓ durable command/event
Communications AI source port
↓ target-bound Blob receipt in durable result
communication_summary workflow
↓ Blob custody transfer to exact AI Engine audience
AI Engine summary request_rpc
```

Communications проверяет canonical message active/current revision, bounded
UTF-8 sender/subject/body и owner. Private content существует только внутри
target-bound Blob. Durable envelope, outbox, inbox, status, SSE, logs and errors
не содержат source content.

Client content ticket ADR-0315 не используется: он связан с client session, а
workflow является отдельным managed recipient.

### AI и provider contracts

`CommunicationSummaryInferenceRequestV1` имеет собственный
`AiContextReceiptV1`, summary-specific private source receipt, requested
language/length, fixed output budgets, local-only egress policy и authenticated
logical owner. Это не reply request с другим prompt.

`CommunicationSummaryInferenceResultV1` содержит summary candidate,
resolved language/length, completeness, confidence, terminal status и
`AiInferenceReceiptV1`. AI Engine сохраняет только receipts, lifecycle и
candidate; private source bytes и provider request body не сохраняются.

AI Engine вызывает `AiProviderSummaryGenerationRequestV1` через отдельный
Kernel `request_rpc`. Ollama adapter использует отдельную fixed summary policy и
закрытую JSON Schema. Caller не выбирает provider, model, endpoint или prompt.
Ollama остаётся concrete integration и не становится частью Communications,
workflow или AI Engine.

### Persistence и replay

Workflow state machine:

```text
accepted
→ preparing_source
→ awaiting_inference
→ ready | rejected
```

Persistence ключуется `(logical_owner_id, operation_id)` и хранит request
fingerprint. Duplicate exact request возвращает тот же run/result; conflicting
request с тем же operation ID отклоняется. Inbox проверяет exact message ID и
envelope hash до mutation. Outbox сохраняет exact canonical envelope bytes до
publish. Terminal result и realtime event фиксируются атомарно.

Recovery выбирает только non-terminal runs authenticated owner. Restart,
generation change, grant epoch change, suspend или revoke не разрешают stale
process продолжить Blob, event, Storage или request routes.

### Kernel agreement

Новые Kernel API или owner-specific imports не вводятся. Используются только:

- existing managed workflow admission and signed distribution binding;
- owner-local Storage binding and Vault-issued database credentials;
- capability-routed `client_rpc` and `request_rpc`;
- NATS durable event route with exact ACL;
- target-bound Blob custody operations;
- shared client realtime publish and Gateway SSE.

Kernel/Gateway не компилируют summary schema или workflow package. Descriptor
объявляет rights, но effective GrantSet, current runtime generation/grant epoch
и exact capability route остаются authority Kernel.

## Phase gate

`communication_summary_v1` становится `implemented` только атомарно после:

1. пяти отдельных workflow units и exact package metadata;
2. distinct Communications summary source events and target-bound Blob content;
3. distinct AI Engine and provider summary request/result contracts;
4. owner-local atomic persistence, replay and recovery;
5. signed release admission всех изменённых runtime artifacts;
6. authenticated Gateway Start/Get и replayable SSE;
7. live managed positive contour через настоящий Ollama process;
8. wrong-owner, stale-source, request-conflict, provider failure, restart,
   revoke, grant/generation fence and privacy negatives;
9. architecture, Cargo boundaries, formatting, Clippy, workspace/integration,
   frontend and full pre-push gates.

Skeleton, canned provider response, reuse reply contract или frontend-only card
не открывают gate.

## Последствия

- Summary остаётся candidate/read result и не меняет canonical Communications.
- Task/note/deadline extraction не смешивается с summarization.
- Reply и Summary могут использовать одни platform primitives без общего
  business workflow или shared persistence authority.
- Следующие translation/explanation workflows обязаны получить собственные
  ADR и exact contracts; summary не становится их compatibility facade.

## Отклонённые варианты

### Добавить `mode = summary` в Reply Suggestion

Смешивает две причины изменения, contracts, persistence and client semantics.

### Добавить `Summarize` method в Communications domain

Делает business domain AI orchestrator и скрывает cross-owner custody.

### AI Engine напрямую читает Communications

Нарушает ADR-0226 и превращает engine в cross-domain context aggregator.

### Generic AI operation enum

Создаёт catch-all endpoint, payload-selected behavior и не даёт capability-level
approval/revoke.

### Вернуть historical structured email intelligence целиком

Смешивает summary, extraction, Review promotion и provider-specific pipeline.
