# ADR-0371: Bounded attachment text extraction workflow

Статус: Принято

Дата: 2026-08-01

Состояние реализации: реализовано. Production gate
`attachment_text_extraction_v1` закрыт exact eleven-unit topology:
`makosh-attachment-text-extraction-api`, `-ingress`, `-core`,
`-parser-contract`, `-plain`, `-pdf`, `-docx`, `-ocr`, `-persistence`,
`-runtime` и `-assembly`. Workflow владеет versioned Start/Get/ReadText и
realtime contracts, order-independent owner-local evidence join, exact
message/hash inbox, durable custody outbox, fenced jobs и derived Blob receipt;
PostgreSQL не хранит extracted plaintext. Attachment Security отдельно владеет
target-owned custody command/result, current-custodian redelegation и своим
owner-local replay/job/outbox, не становясь частью workflow.

Managed production contour доказал реальный UTF-8, PDF, DOCX и local
Tesseract `eng+rus` OCR; authenticated Gateway Start/Get/ReadText; replayable
SSE; NATS outage; runtime restart; one-use Blob transfer; source receipt/hash,
wrong-owner, collision, stale runtime/grant/parser revision и stale custody
proof fences; malformed, unsupported, oversized, missing parser, Blob outage и
Vault outage fail-closed behavior. Events, SSE, errors и diagnostics не несут
private text или Blob authority. Architecture/SRP/Cargo/clippy, managed
conformance и full pre-push gates пройдены перед переводом inventory в
`implemented`.

Зависит от:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0273](ADR-0273-attachment-security-engine-and-event-only-verdict-authority.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0360](ADR-0360-current-custodian-target-bound-blob-redelegation.md).

## Контекст

Legacy Communications выполнял UTF-8, PDF, DOCX и image OCR extraction внутри
Communications service, читал domain-owned filesystem path и сохранял derived
text рядом с canonical attachment rows. Он также вызывал preview/CDR behavior
как побочный эффект extraction. Перенос такой структуры вернул бы content
processor, parser/runtime configuration, Blob data-plane и несколько причин
изменения в Communications domain.

Clean-room Communications владеет только canonical attachment anchor и safety
projection. Attachment Security является единственным authority для
`safe_for_delivery`, Blob владеет bytes, а text extraction является отдельным
cross-owner use-case workflow. Extraction result является derived content, а не
canonical Communications truth и не Attachment Security verdict.

## Решение

Вводится отдельный workflow:

```text
owner_id  = attachment_text_extraction
module_id = makosh-attachment-text-extraction-runtime
kind      = workflow
```

Пользователь отправляет через Core Gateway только stable operation ID и
canonical attachment anchor. Клиент не передаёт Blob reference, MIME,
filename, provider/account identity, source hash, filesystem path или bytes.

Workflow owner-locally и независимо от порядка объединяет:

1. authenticated Start request;
2. exact provider-neutral Attachment Security scan candidate;
3. canonical Communications transition `blob_admitted -> safe_for_delivery`.

Join создаёт durable custody-delegation command, но не parser job. Attachment
Security проверяет собственный completed safe verdict и current custody,
получает current-custodian target-bound proof по ADR-0360 и публикует exact
delegated/rejected result. Только exact command-linked delegated result создаёт
fenced extraction job.

```text
client -> Gateway -> text-extraction request_rpc
scan candidate event -------------------------\
safe_for_delivery event -----------------------> owner-local durable join
                                                  -> delegation command outbox

delegation command -> Attachment Security inbox
  -> safe verdict/current custody verification
  -> Kernel current-custodian redelegation
  -> delegated/rejected result event
  -> text-extraction result inbox
  -> target-bound Blob custody
  -> receipt-bound one-use read
  -> exact parser adapter
  -> owner-local derived-text Blob and status
```

Workflow не вызывает Communications или Attachment Security RPC, не читает их
storage и не импортирует их implementation. Kernel, Gateway, Event Hub и Blob
не выбирают parser и не интерпретируют text.

## Event-only custody contract

Target-owned `makosh-attachment-text-extraction-ingress` содержит три exact
durable contracts:

```text
RequestAttachmentTextCustodyDelegationV1
AttachmentTextCustodyDelegatedV1
AttachmentTextCustodyDelegationRejectedV1
```

Request содержит только request/run/anchor, exact candidate message/hash,
safety message/evidence и logical owner. Target triple является константой:

```text
owner      = attachment_text_extraction
module     = makosh-attachment-text-extraction-runtime
capability = attachment_text_extraction.blob.v1
```

Delegated result несёт opaque source reference, declared size/digest и bounded
target-bound proof только во внутренних event/runtime/persistence surfaces.
Proof, reference, bytes, filename, MIME и provider identity отсутствуют в
client API, SSE, logs, health и telemetry.

Attachment Security импортирует только target-owned ingress contract. Workflow
может импортировать public Attachment Security и Communications attachment
contracts, но не engine/domain implementation. Это contract edge, не
engine-inside-workflow и не facade.

## Client boundary

`makosh-attachment-text-extraction-api` разделяет три причины вызова:

- `Start` — command/request receipt;
- `Get` — metadata-only status query;
- `ReadText` — отдельный authenticated bounded private-content request.

`Get` и realtime содержат run ID, anchor, state/revision, exact format,
extracted byte count, truncation и bounded error enum. Они не содержат text или
Blob authority. `ReadText` работает только для `ready`, возвращает максимум
64 KiB UTF-8 на вызов и сообщает, был ли visible result truncated. Full derived
artifact ограничен 1 MiB и остаётся owner-local Blob content.

`accepted` не означает completion. Terminal status приходит через exact Get и
общий replayable SSE; polling не вводится.

## Parser units и V1 support

Parser selection основан на bounded byte validation/magic, а не на
caller/provider filename или MIME. V1 восстанавливает поддержанное legacy
поведение следующими независимыми adapter units:

- bounded plain UTF-8 text/JSON/XML/YAML/CSV normalization;
- bounded PDF text extraction;
- bounded DOCX ZIP/XML text extraction;
- bounded PNG/JPEG/TIFF/BMP OCR with exact `eng+rus` local language policy.

Каждый adapter имеет одну причину изменения и не зависит от workflow runtime,
persistence, Communications, integrations или Kernel. PDF/DOCX/OCR выполняются
в отдельном verified managed parser process или в эквивалентно изолированном
adapter execution contour: no external network, no shared writable source
mount, bounded input/output/time/memory/process count and fail-closed parser
errors. Наличие только mock loopback response не закрывает provider/parser gate.

Unsupported formats завершаются typed `unsupported`, не считаются empty text.
Invalid UTF-8, malformed PDF/DOCX/image, timeout, output overflow и unavailable
parser завершаются bounded failure без частичного artifact.

## Derived artifact

Runtime нормализует line endings, валидирует UTF-8 и hard 1 MiB output bound,
записывает derived text через собственную Blob write capability и только после
успешного Blob commit атомарно переводит run в `ready`. Owner-local persistence
хранит opaque derived receipt, source evidence/hash binding, parser identity,
state/revision, inbox/outbox and replay state; extracted bytes и plaintext не
хранятся в PostgreSQL и не индексируются этим gate.

Source replacement, safety evidence mismatch, stale custody proof, parser
revision mismatch или missing derived Blob делают старый result unreadable.
Search indexing, preview, translation и AI egress являются отдельными consumers
и gates; они не запускаются как side effect extraction.

## Единицы сборки и SRP

```text
makosh-attachment-text-extraction-api
  generated Start/Get/ReadText/realtime client contract

makosh-attachment-text-extraction-ingress
  target-owned custody command/result event contracts

makosh-attachment-text-extraction-core
  pure join, lifecycle, format/output bounds and terminal decisions

makosh-attachment-text-extraction-parser-contract
  byte-only parser request/result/error contract and exact format detection

makosh-attachment-text-extraction-plain
makosh-attachment-text-extraction-pdf
makosh-attachment-text-extraction-docx
makosh-attachment-text-extraction-ocr
  independent bounded parser adapters

makosh-attachment-text-extraction-persistence
  owner-local PostgreSQL inbox/outbox/run/job/artifact/realtime state

makosh-attachment-text-extraction-runtime
  managed request/query/Event/Blob/parser orchestration only

makosh-attachment-text-extraction-assembly
  descriptor/settings/Storage artifacts and unsigned release fragment only
```

API/ingress/core/parser-contract/parser units не зависят от runtime или
persistence. Persistence является единственным SQL owner. Runtime не
материализует release artifacts.
Assembly не запускает runtime и не подписывает manifest. Communications,
integration и Attachment Security packages не получают parser dependency.

## Phase gate `attachment_text_extraction_v1`

Gate становится `implemented` атомарно только после:

1. exact eleven-unit package inventory и compile isolation;
2. versioned Start/Get/ReadText/realtime and custody event contracts;
3. owner-local request replay, exact message/hash inbox, outbox and fenced jobs;
4. order-independent request/candidate/safety join;
5. current-custodian target-bound delegation and one-use Blob read;
6. real bounded UTF-8, PDF, DOCX and local `eng+rus` OCR parser evidence;
7. derived owner-local Blob write without PostgreSQL plaintext;
8. wrong-owner, collision, stale revision/generation/grant/proof and source-hash
   negative matrix;
9. unavailable Blob/Vault/Event/parser and malformed/oversized input fail closed;
10. restart/NATS outage replay without second custody transfer or parser run;
11. authenticated Gateway Start/Get/ReadText and exact SSE cursor replay;
12. privacy-negative event/SSE/log/error/health/telemetry evidence;
13. architecture, SRP, Cargo, clippy, managed and full pre-push gates.

До выполнения всех пунктов inventory state остаётся `planned`.

## Отклонённые варианты

### Вернуть extraction service в Communications

Отклонено: domain получил бы parser, Blob I/O, derived artifact storage и
content-processing lifecycle.

### Добавить extraction в Attachment Security

Отклонено: safety verdict и content derivation имеют разные authority, failure
semantics и release cadence.

### Передать filename/MIME/Blob reference из клиента

Отклонено: клиент не является source evidence или custody authority.

### Публиковать extracted text в event или SSE

Отклонено: durable spine и realtime status не являются private-content
transport. Текст выдаётся только exact authenticated bounded request.

### Объединить preview, translation и search indexing с extraction

Отклонено: это разные use cases, owners, egress rules и phase gates.
