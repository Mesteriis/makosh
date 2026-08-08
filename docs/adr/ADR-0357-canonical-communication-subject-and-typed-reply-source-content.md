# ADR-0357: Canonical communication subject and typed reply source content

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано. Communications ingress и canonical
evidence major 1 переведены на revision 2, Mail передаёт sender/subject,
canonical persistence хранит bounded subject, а Communications AI source
revision 2 передаёт один typed sender/subject/body Blob в Reply Suggestion
workflow. Live negative managed admission/orchestration реализованы
full-ensemble conformance ADR-0353; успешный Ollama inference остаётся
отдельным закрытым gate.

Уточняет:

- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0240](ADR-0240-canonical-communications-owner-clean-room-migration.md);
- [ADR-0353](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md);
- [ADR-0356](ADR-0356-renewable-blob-authority-for-durable-ai-workflows.md).

## Контекст

Historical AI Reply использовал sender, subject и body canonical message.
Clean-room Mail runtime уже извлекает sender и subject из IMAP/Gmail, но
Communications ingress revision 1 переносит только sender display label и body
Blob receipt. Subject остаётся только в integration-owned operational
projection, а Communications AI source создаёт body-only Blob. Reply workflow
поэтому вынужден кодировать пустые sender/subject, что честно отражено в
ADR-0353/0356, но не даёт reference parity.

Копирование Mail operational DTO или чтение Mail persistence из
Communications/workflow запрещены. Передача трёх отдельных Blob receipts также
размножила бы custody/retry lifecycle без отдельной продуктовой необходимости.

## Решение

### Canonical bounded subject

Communications ingress major 1 получает revision 2 и optional
`message_subject`:

- поле является provider-neutral display evidence, а не identity, locator,
  routing key или authorization input;
- UTF-8 значение trim-normalized;
- пустое значение, control characters и размер свыше 998 bytes запрещены;
- integration может не передавать subject, если provider не имеет такого
  понятия;
- Mail передаёт exact parsed IMAP/Gmail subject, не создавая fallback;
- другие integrations не обязаны фабриковать subject из chat title/topic.

Communications domain переносит subject в canonical evidence summary и
owner-local PostgreSQL migration. Canonical evidence event major 1 получает
revision 2 вместе с новым schema digest. Persistence другого owner и provider
operational projection не читаются.

### Один typed source-content Blob

`communications_ai_context_source_v1` получает coordinated revision 2 без
compatibility facade:

```text
CommunicationReplySourceContentV1
  sender_utf8
  subject_utf8
  body_utf8
```

Communications:

1. читает exact admitted body Blob;
2. берёт sender/subject только из того же canonical evidence snapshot;
3. валидирует UTF-8 и bounds;
4. protobuf-кодирует один typed source content;
5. записывает один target-bound Blob для
   `communication_reply_suggestion`;
6. публикует только receipt/proof в durable result.

Старые `CommunicationReplyBodySourceReceiptV1` и `body_source` удаляются.
Новый `CommunicationReplySourceContentReceiptV1`/`source_content` занимает тот
же bounded event role, но его Blob payload имеет explicit schema. Raw
sender/subject/body не попадают в NATS, Gateway, logs или workflow
persistence.

Reply workflow декодирует Communications-owned content, затем кодирует
AI-owned `AiReplySourceContentV1` в отдельный AI-target-bound Blob. Workflow не
импортирует Communications persistence или Mail integration. AI engine и
Ollama integration не импортируют Communications contract.

### Units и SRP

- `makosh-communications-ingress` владеет public observation wire contract;
- `makosh-mail-core` только маппит Mail observation в public ingress draft;
- `makosh-communications-domain` валидирует canonical semantics;
- `makosh-communications-persistence` владеет canonical subject storage;
- `makosh-communications-ai-source-api` владеет typed source-content schema;
- `makosh-communications-runtime` materializes source content;
- `makosh-communication-reply-suggestion-runtime` выполняет custody,
  translation в AI contract и cleanup.

Ни один из этих build units не становится facade другого owner.

## Phase gate

Slice считается реализованным только после:

1. ingress/source contract revision bump и schema-bound admission;
2. Mail IMAP/Gmail sender+subject producer coverage;
3. canonical persistence migration и current-revision snapshot;
4. source-content encode/decode bounds и negative tests;
5. workflow propagation без raw private-content persistence;
6. architecture, Cargo и frontend fast gates.

Live negative managed Reply Suggestion admission/orchestration теперь доказаны
ADR-0353, но успешный Ollama inference остаётся отдельным gate и не открывается
этим решением.

## Отклонённые варианты

### Workflow читает Mail operational query

Создаёт integration dependency и provider branching в workflow.

### Communications импортирует AI contract

Связывает domain с engine. Между owners остаются два independently typed Blob
payloads и явная workflow translation.

### Оставить body-only alias

Создаёт competing compatibility facade и позволяет новым consumers продолжать
терять sender/subject.

### Фабриковать subject из conversation title

Меняет provider evidence и скрывает отсутствие source semantics.
