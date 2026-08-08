# ADR-0378: Bounded attachment translation workflow

Статус: Принято

Дата: 2026-08-02

Состояние реализации: реализовано. Phase gate
`attachment_translation_source_producer_v1` закрывает все шесть workflow production units:
generated private-content-free client API, exact durable source ingress, pure
lifecycle core, owner-local PostgreSQL persistence, managed runtime и release
assembly. Persistence атомарно
владеет idempotency, inbox/outbox, recovery и realtime, а SQL хранит только
authority receipts и result metadata без source/translated text. Кроме них AI
Engine получил distinct `AttachmentTranslationInferenceRequestV1`, отдельный
use-case receipt, owner-local additive persistence и worker, переиспользующий
только нижний `ai.provider.translate.v1`. Runtime реализует event-only source
consumer, request-routed AI, result Blob materialization, Start/Get/Read,
actor-bound one-use ticket и shared SSE; assembly выдаёт отдельные unsigned
runtime и Storage artifacts. Text Extraction теперь потребляет exact target-owned
command, проверяет current ready run/revision, создаёт отдельную target-bound
Blob-копию, атомарно сохраняет owner-local inbox/outbox и ACK'ает только после
commit. Signed managed negative-provider contour материализует exact release,
поднимает отдельные Communications, Attachment Security, Text Extraction,
Attachment Translation, AI Engine и Ollama processes через
Vault/Storage/Blob/NATS, проводит safe attachment до ready extraction revision и
доказывает authenticated Start/Get/Read, terminal transition через заранее
открытый replayable SSE без polling, deterministic duplicate, conflict,
unsupported language, stale source revision, provider failure, explicit stale
runtime/grant fences, generation successor, metadata/SSE replay после restart,
owner revoke и privacy. Отдельный positive contour прошёл через настоящий
loopback Ollama process и модель `makosh-conformance`: workflow получил реальный
переведённый candidate, выдал actor-bound one-use Blob ticket, закрыл wrong-actor
и ticket replay, а после restart запретил новую выдачу authority для артефакта
предыдущего generation. Поэтому `attachment_translation_v1` переведён в
`implemented`.

Restart recovery выделен из inference execution в отдельную функциональную
единицу. Он не переиспользует сохранённый runtime-bound Blob proof: current
generation заново материализует AI source из durable workflow authority,
перезапечатывает тот же canonical request (custody proof не входит в digest),
проверяет неизменность request/source identity и только после terminal state
выполняет custody release и owner-local cleanup.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0226](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0356](ADR-0356-renewable-blob-authority-for-durable-ai-workflows.md);
- [ADR-0360](ADR-0360-current-custodian-target-bound-blob-redelegation.md);
- [ADR-0363](ADR-0363-communication-translation-workflow-and-ai-contracts.md);
- [ADR-0371](ADR-0371-bounded-attachment-text-extraction-workflow.md).

## Контекст

Historical frontend предлагал attachment translation как режим общего
translation handler. Такой surface скрывал четыре разных authority:

- Communications или provider integration владеет исходным attachment evidence;
- Attachment Security допускает обработку immutable bytes;
- Attachment Text Extraction владеет derived textual artifact;
- AI Engine и concrete provider выполняют inference.

Clean-room уже разделяет этих owners. `attachment_text_extraction_v1` создаёт
bounded derived text после safety и custody checks, а
`communication_translation_v1` переводит только один canonical communication
evidence item. Ни один из них не является facade для attachment translation.

Прямой вызов Text Extraction query/read API из нового workflow нарушил бы
event-only owner boundary и связал бы две independently restartable runtime.
Передача client content ticket также неверна: ticket session-bound и имеет
client audience, тогда как durable workflow требует target-bound Blob custody.

## Решение

### Owner и scope

`attachment_translation_v1` принадлежит отдельному workflow owner
`attachment_translation`.

В scope V1 входят:

- перевод одного уже извлечённого attachment text artifact;
- exact target languages `english`, `russian` и `spanish`;
- bounded translated UTF-8 candidate;
- detected source language, completeness и confidence;
- durable lifecycle, idempotency, restart и replay;
- private result read через one-use client Blob ticket.

Не входят:

- extraction, OCR, archive traversal или preview rendering;
- изменение canonical attachment или Communications evidence;
- thread/batch translation;
- caller-selected provider, model, endpoint, prompt или arbitrary options;
- recording, transcription, summary или task/note promotion.

### Build units и SRP

Workflow вводит шесть отдельных Cargo packages:

```text
makosh-attachment-translation-api
makosh-attachment-translation-ingress
makosh-attachment-translation-core
makosh-attachment-translation-persistence
makosh-attachment-translation-runtime
makosh-attachment-translation-assembly
```

Причины изменения разделены функционально:

- `api` — generated client Start/Get/Read и status/result schemas;
- `ingress` — exact durable source request/prepared/rejected schemas;
- `core` — pure validation, idempotency fingerprint и lifecycle;
- `persistence` — owner-local runs, inbox/outbox, jobs и realtime sequence;
- `runtime` — event orchestration, Blob custody и AI request routing;
- `assembly` — unsigned descriptor, settings, Storage bundle и release fragment.

Attachment Text Extraction получает зависимость только на public
`makosh-attachment-translation-ingress`. Attachment Translation runtime не
импортирует Text Extraction, Communications, Attachment Security, AI Engine,
Ollama, Blob, Kernel или Gateway implementation/storage packages.

`attachment_translation` является workflow, не domain и не integration.
Ollama остаётся integration, AI остаётся engine, а Text Extraction остаётся
отдельным workflow owner.

### Client contract

Client command принимает:

```protobuf
message StartAttachmentTranslationRequestV1 {
  uint32 protocol_major = 1;
  bytes operation_id = 2;
  bytes source_extraction_run_id = 3;
  uint64 expected_source_revision = 4;
  AttachmentTranslationLanguageV1 target_language = 5;
}
```

`source_extraction_run_id` является provider-neutral identity готового
Text Extraction run. Provider/account/file path/name/content type в контракте
запрещены. Start возвращает durable receipt; accepted не означает completion.
Get и replayable SSE содержат только identity, state, revision, language,
bounded size/digest metadata и typed error. Source или translated text в них
запрещены.

Terminal result читается отдельным `ReadTranslation` через target-bound Blob и
one-use client ticket с platform response ceiling. Result является candidate и
не mutates source evidence.

### Event-only source handoff

```text
Authenticated client Start
↓ client_rpc
Attachment Translation workflow
↓ attachment_translation.source.requested.v1
Attachment Text Extraction owner consumer
↓ current-run/revision/custody validation
Blob custody redelegation to attachment_translation audience
↓ attachment_translation.source.prepared.v1 | source.rejected.v1
Attachment Translation workflow
↓ ai.attachment-translation.request.v1
AI Engine
↓ ai.provider.translate.v1
configured local provider integration
```

Source request содержит logical owner, workflow/run identity, exact extraction
run identity/revision и target capability. Он не содержит text, Blob authority,
provider identity или caller-selected audience. Text Extraction owner сам
выбирает current derived artifact и получает новый target-bound Blob receipt.

Workflow не вызывает Attachment Text Extraction RPC и не читает его storage.
Text Extraction не вызывает Attachment Translation runtime. Оба обмениваются
только exact `DurableEnvelopeV1` events с inbox ID/hash verification и durable
Ack после committed mutation.

### AI contract

AI Engine получает distinct capability
`ai.attachment-translation.request.v1` и generated
`AttachmentTranslationInferenceRequestV1`. Это отдельный use-case contract, а
не mode в `CommunicationTranslationInferenceRequestV1`.

Request содержит authenticated owner context, `AiContextReceiptV1`, exact
source Blob receipt, target language и fixed output budget. AI Engine может
переиспользовать provider-facing `ai.provider.translate.v1`, поскольку
provider adapter выполняет один exact translation operation, но caller не
выбирает provider/model/endpoint/prompt.

AI persistence хранит receipts, lifecycle и translated candidate. Private
source material материализуется только через Blob authority текущего audience.
Ollama или другой provider не импортирует Attachment Translation workflow.

### Persistence и recovery

Workflow state machine:

```text
accepted
→ awaiting_source
→ awaiting_inference
→ materializing_result
→ ready | rejected
```

Owner-local persistence ключуется `(logical_owner_id, operation_id)` и хранит
request fingerprint. Exact duplicate возвращает тот же receipt/result; тот же
operation ID с другим payload отклоняется.

Persistence хранит только identities, revisions, receipts, digests, lifecycle,
typed failures, inbox/outbox bytes и realtime metadata. Source text и translated
text не попадают в SQL workflow owner: translated private bytes хранятся в Blob
и читаются через bounded ticket. Raw provider response также запрещён.

Recovery выбирает только non-terminal runs current logical owner и повторяет
только idempotent external steps. Runtime generation, grant epoch, Storage
binding, source custody и AI route проверяются перед каждым external action.
Restart, revoke или stale generation инвалидируют старые authorities и не могут
продолжить run. Runtime-bound proof после restart всегда перевыпускается через
current capability session; persisted proof не считается живым credential.

### Kernel и Gateway boundary

Новые Kernel API не вводятся. Используются существующие signed managed workflow
admission, owner-local Storage binding, capability-routed client/request RPC,
NATS durable events/Ack, target-bound Blob custody и shared replayable client
SSE.

Kernel и Gateway не компилируют Attachment Translation schema, не читают
private content, не выбирают AI provider и не становятся workflow facade.

## Phase gate

`attachment_translation_contracts_v1` является отдельным атомарным foundation
gate и включает только:

1. `makosh-attachment-translation-api`;
2. `makosh-attachment-translation-ingress`;
3. `makosh-attachment-translation-core`;
4. generated schema digests, event envelope validation, pure lifecycle tests и
   compile-isolation evidence.

Этот foundation gate не выдаёт runtime grants, Blob authority или AI route и
не меняет состояние `attachment_translation_v1`.

Следующий phase gate `attachment_translation_persistence_v1` добавляет только
`makosh-attachment-translation-persistence`: owner-local additive Storage
bundle, operation fingerprint, commit-before-Ack inbox, exact outbox bytes,
recoverable non-terminal runs и replayable realtime sequence. Он не вводит
runtime, AI provider или cross-owner SQL и также не меняет состояние полного
inventory gate.

Phase gate `attachment_translation_ai_engine_v1` добавляет backward-compatible
revision общего AI contract, но не добавляет Attachment Translation behavior в
Communication Translation. AI Engine предоставляет отдельную capability
`ai.attachment-translation.request.v1`, отдельные core lifecycle/persistence
records и worker. Только provider-facing translation operation остаётся общей;
provider/model/endpoint/prompt не попадают в новый request.

Phase gate `attachment_translation_runtime_assembly_v1` добавляет только
workflow runtime и downstream assembly. Runtime импортирует public Text
Extraction ingress и AI contracts, но не их runtime/storage/implementation;
source handoff остаётся durable event-only. Translated UTF-8 существует только
в bounded memory и Blob, а PostgreSQL хранит digest/size/reference и current
runtime/grant fences. `client_blob` использует отдельный generated read schema и
одноразовый ticket, привязанный к logical owner, authenticated device, artifact
revision, runtime generation и grant epoch. Этот gate ещё не утверждает source
producer или live contour и не меняет reconstruction inventory.

Phase gate `attachment_translation_source_producer_v1` добавляет только public
ingress dependency в Text Extraction runtime, exact consume/publish capability,
owner-local additive inbox/outbox migration и target-bound Blob source copy.
Он не добавляет прямой workflow RPC, cross-owner SQL, provider identity или
plaintext в PostgreSQL. Duplicate delivery переиспользует committed result,
а durable Ack выполняется только после inbox/outbox commit. Этот gate ещё не
утверждает signed managed release или live contour и не меняет reconstruction
inventory.

`attachment_translation_v1` переведён в `implemented` атомарно после:

1. шести отдельных workflow units и compile-isolation checks;
2. generated Start/Get/Read/realtime contracts без provider/model/prompt;
3. exact event-only Text Extraction source request/prepared/rejected flow;
4. target-bound Blob redelegation и stale source/revision/custody negatives;
5. distinct AI Engine attachment-translation contract;
6. owner-local atomic persistence, inbox/outbox, replay и recovery;
7. signed release admission runtime/storage artifacts;
8. authenticated Gateway Start/Get/Read и replayable SSE;
9. wrong-owner, conflict, unsupported language, oversize, provider failure,
   restart, revoke, grant/generation fence и privacy negatives;
10. managed positive contour через настоящий local AI provider process;
11. architecture, Cargo boundaries, formatting, Clippy, workspace/integration,
    frontend и full pre-push gates.

Все пункты закрыты repository, managed-runtime и live-provider evidence.
Skeleton, identity translation, fixture/canned provider response, direct source
RPC, client-ticket reuse или legacy REST facade не считались доказательством
этого gate.

## Последствия

- Attachment translation получает собственную lifecycle и persistence
  authority без превращения Communications в AI facade.
- Text Extraction остаётся source owner и не становится orchestration runtime.
- AI use-case contracts остаются distinct, а concrete provider остаётся
  integration.
- Цена решения — отдельные event contracts и build units; это необходимо для
  restart, privacy, grants и функционального SRP.

## Отклонённые варианты

### Добавить attachment mode в Communication Translation

Смешивает canonical communication source и derived attachment artifact,
разные custody proofs и разные причины изменения.

### Читать Text Extraction через client RPC

Создаёт synchronous workflow-to-workflow dependency и переиспользует
client-scoped authority для durable orchestration.

### Передать extracted text в durable event

Размещает private content в NATS/outbox/inbox и нарушает privacy boundary.

### Реализовать перевод внутри Text Extraction runtime

Смешивает extraction/parser lifecycle с AI orchestration и provider policy.
