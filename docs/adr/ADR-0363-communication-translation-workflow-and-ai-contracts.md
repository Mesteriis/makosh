# ADR-0363: Communication Translation workflow and AI contracts

Статус: Принято

Дата: 2026-07-31

Состояние реализации: implemented. Gate `communication_translation_v1` закрыт
атомарно: distinct contracts, Communications source events, AI Engine и Ollama
translation paths, пять отдельных workflow units, owner-local persistence и
replay, signed release artifacts, authenticated Gateway Start/Get, replayable
SSE, managed negative/restart/revoke/fencing contour и положительный live
contour через настоящий Ollama process доказаны независимо от Summary.

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
- [ADR-0357](ADR-0357-canonical-communication-subject-and-typed-reply-source-content.md);
- [ADR-0362](ADR-0362-communication-summary-workflow-and-ai-contracts.md).

## Контекст

Historical Communications предоставлял single-message, thread и attachment
translation через один handler/service surface. Он напрямую читал message body,
выбирал AI runtime и возвращал provider model identity. Такой surface смешивал
Communications, workflow, AI Engine и provider integration.

Clean-room inventory разделяет эти причины изменения. Этот ADR покрывает только
перевод одного canonical communication evidence item. Attachment translation
остаётся отдельным `attachment_translation_v1`. Thread translation в V1 не
является неявным batch mode: для него потребуется отдельный bounded workflow
или явная композиция уже завершённых single-message runs.

## Решение

### Владельцы и build units

Translation реализуется отдельным owner `communication_translation` в пяти
units:

- `makosh-communication-translation-api` — generated Start/Get/realtime
  contract;
- `makosh-communication-translation-core` — pure lifecycle и validation;
- `makosh-communication-translation-persistence` — owner-local PostgreSQL
  state, inbox/outbox и realtime replay;
- `makosh-communication-translation-runtime` — managed workflow orchestration;
- `makosh-communication-translation-assembly` — unsigned descriptor, settings
  schema, Storage bundle и release fragment.

Workflow не принадлежит Communications, AI или Ollama и не импортирует их
implementation/storage packages. Communications, AI Engine и Ollama получают
только собственные exact contract/runtime extensions.

### Клиентский контракт

V1 принимает:

```protobuf
message StartCommunicationTranslationRequestV1 {
  uint32 protocol_major = 1;
  bytes operation_id = 2;
  bytes source_message_id = 3;
  uint64 expected_source_revision = 4;
  CommunicationTranslationLanguageV1 target_language = 5;
}
```

Поддерживаются exact target languages `english`, `russian` и `spanish`.
Свободная строка языка, provider/model/endpoint/prompt и generic options
запрещены. Result содержит stable run/source identity, monotonic state revision,
bounded translated UTF-8 candidate, typed detected source language, exact target
language, completeness, confidence и typed terminal error. Translation является
candidate/read result и не изменяет canonical Communications evidence.

Start/Get идут через owner-neutral Gateway capability router, terminal status —
через общий replayable SSE. Gateway не читает source content, не выбирает AI
provider и не становится translation facade.

### Source flow

```text
Authenticated client Start
↓ client_rpc
communication_translation workflow
↓ distinct durable command/event
Communications translation source port
↓ target-bound Blob receipt
communication_translation workflow
↓ Blob custody transfer to exact AI Engine audience
AI Engine translation request_rpc
```

Communications проверяет logical owner, active/current canonical revision и
bounded sender/subject/body. Private content существует только внутри
target-bound Blob. Durable envelopes, inbox/outbox, status, SSE, diagnostics и
errors не содержат private source bytes.

Summary source events и Blob audience не переиспользуются: translation имеет
собственные event names, schema hashes, target capability и receipts. Client
content ticket из ADR-0315 также не подходит, поскольку он session-bound.

### AI и provider contracts

AI Engine предоставляет capability `ai.translation.request.v1` с distinct
`CommunicationTranslationInferenceRequestV1` и result. Request содержит
`AiContextReceiptV1`, translation-specific private source receipt, exact target
language, fixed output budget, local-only egress и authenticated logical owner.
AI сохраняет только receipts, lifecycle и translated candidate.

AI Engine вызывает capability `ai.provider.translate.v1` через Kernel
`request_rpc`. Ollama Integration использует fixed translation instruction и
закрытую JSON Schema для translated text и detected source language. Caller не
выбирает provider, model, endpoint или prompt. Ollama остаётся concrete
integration и не становится частью Communications, Translation workflow или AI
Engine.

### Persistence, replay и fences

Workflow state machine:

```text
accepted
→ preparing_source
→ awaiting_inference
→ ready | rejected
```

Persistence ключуется `(logical_owner_id, operation_id)` и хранит request
fingerprint. Exact duplicate возвращает тот же run/result; conflict с тем же
operation ID отклоняется. Inbox проверяет message ID и envelope hash до
mutation. Outbox хранит exact envelope bytes до publish. Terminal result и
realtime event фиксируются атомарно.

Restart/recovery выбирает только non-terminal runs текущего authenticated owner.
Runtime generation, grant epoch, Storage binding, Blob custody и request route
проверяются на каждом внешнем шаге. Suspend/revoke или stale coordinate не может
продолжить перевод.

### Kernel agreement

Новые Kernel API и owner-specific imports не вводятся. Используются только
существующие signed managed admission, owner-local Storage/Vault binding,
capability-routed `client_rpc`/`request_rpc`, NATS durable events, target-bound
Blob custody и shared client realtime. Kernel/Gateway не компилируют Translation
schema или workflow package.

## Phase gate

`communication_translation_v1` становится `implemented` только атомарно после:

1. пяти отдельных workflow units и exact package metadata;
2. distinct Communications translation source events и target-bound Blob;
3. distinct AI Engine и provider translation request/result contracts;
4. owner-local atomic persistence, replay и recovery;
5. signed release admission всех изменённых runtime artifacts;
6. authenticated Gateway Start/Get и replayable SSE;
7. live managed positive contour через настоящий Ollama process;
8. wrong-owner, unsupported-language, stale-source, request-conflict, provider
   failure, restart, revoke, grant/generation fence и privacy negatives;
9. architecture, Cargo boundaries, formatting, Clippy, workspace/integration,
   frontend и full pre-push gates.

Skeleton, identity translation, canned provider response, Summary mode switch,
legacy REST facade или frontend-only panel не открывают gate.

## Последствия

- Communications не становится AI orchestrator и не хранит translation state.
- Summary и Translation используют общие platform primitives, но разные
  contracts, lifecycle и persistence authority.
- Attachment/thread/bilingual flows не прячутся в generic translation mode.
- Provider/model metadata не становится частью client-facing business result.

## Отклонённые варианты

### Добавить `operation = translate` в Summary

Смешивает разные причины изменения, contracts, output semantics и grants.

### Добавить Translate method в Communications domain

Делает Communications владельцем cross-owner AI orchestration.

### Передать body напрямую из Gateway в AI

Обходит canonical evidence owner, event provenance и Blob custody.

### Сохранить legacy single/thread/attachment handler

Создаёт facade над тремя независимыми workflow и provider selection.
