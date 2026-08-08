# ADR-0366: Communication task candidate extraction and reviewed Task promotion

Статус: Принято

Дата: 2026-07-31

Состояние реализации: implemented. Clean-room slice реализует отдельные extraction
API/core/persistence/runtime units и Communications-owned task-source event
contract с exact envelope builders. Managed runtime предоставляет typed
Start/Get/replayable realtime, event outbox, owner-local recovery и
target-bound Blob materialization для детерминированной extraction.
Communications source producer уже consume-ит exact task-source command,
атомарно пишет prepared/rejected result через Communications outbox и передаёт
typed subject/body только через target-bound Blob custody. Отдельная assembly
формирует unsigned descriptor/settings/storage/release fragment для generic
distribution compiler. Review staged slice также реализует отдельные generated
task-candidate API и pure Review core: deterministic review identity, immutable
approve/reject, expected-revision fence и отдельный promotion lifecycle без
расширения `review-attention`. Review owner-local persistence резервирует
submission до Blob read, хранит state/operation/inbox/outbox/realtime атомарно и
восстанавливает незавершённые submission/promotion. Review public contract
также строит exact submit/submitted/rejected/approved durable envelopes,
отделяет module actor от authenticated owner-device actor и фиксирует distinct
Review- и Tasks-target Blob audiences. Review assembly формирует отдельные
unsigned descriptor/settings/storage/release inputs. Review managed
runtime уже выполняет exact submission consume, target-bound Blob
materialization, owner-local completion/rejection, outbox relay, authenticated
owner-device approve/reject, Tasks-target-bound Blob write и replayable client
realtime. Domain-separated 16-byte actor evidence строится только из
`authenticated_device_id`, переданного Gateway вне client payload. Runtime
package сохраняет exact Domain descriptor, семь отдельных
client/Blob/event/storage capabilities и пустую typed Settings schema, но
Tasks staged slice теперь реализует отдельные `makosh-tasks-command-api` и
`makosh-tasks-core`: exact target-owned durable command/results, deterministic
Task identity, source/review provenance и hints без создания Calendar, Contact,
Project или Obligation truth. Tasks owner-local persistence теперь резервирует
exact command envelope hash/fingerprint до Blob read, атомарно сохраняет Task и
terminal outbox, восстанавливает незавершённую работу и отдельно завершает Blob
cleanup без cross-owner SQL. Отдельный Tasks managed runtime consume-ит только
Tasks-owned durable command, читает уже Tasks-bound Blob, применяет deadline и
owner/generation fences, восстанавливает inbox до новых delivery, сохраняет
terminal result перед Ack, освобождает Blob custody и relay-ит exact outbox
bytes. Он не импортирует Review, Communications или provider packages.
Tasks assembly отдельно материализует canonical descriptor, пустую Settings
schema, owner-local Storage bundle и unsigned release fragment без runtime
launch или signing authority. Development release contour теперь собирает все
три runtime/assembly пары и передаёт их exact artifact fragments единому
distribution compiler для подписи; отдельный module не получает signing
authority. Live managed contour теперь устанавливает одну signed release,
регистрирует и authorizes exact Communications producer, extraction workflow,
Review domain и Tasks domain, выдаёт каждому отдельный owner-local Storage
binding и поднимает их через реальные Vault, Blob, PostgreSQL/PgBouncer и NATS
границы. Kernel admission использует exact descriptor requests, а conformance
фиксирует distinct module/owner identities и runtime generations. Extraction
workflow теперь после детерминированного результата записывает каждый typed
candidate в Review-target-bound Blob, строит exact durable
`SubmitTaskCandidateForReviewCommandV1` только через Review public contract и
атомарно сохраняет terminal extraction state, replayable realtime и все Review
submission envelopes в собственном outbox. Runtime не импортирует Review core,
persistence или runtime implementation; общий outbox relay публикует exact
source и Review envelopes без re-encode. Aggregate
managed E2E теперь начинает extraction через generated Gateway Start/Get из
реального Communications source, а не из seeded Review rows; доказывает exact
request replay/conflict, wrong-owner, stale source, runtime generation/grant
fences, отсутствие Task до approve, ровно один Task после approve и отсутствие
Task после reject. Extraction и Review публикуют отдельные terminal frames в
общий replayable SSE; после owner cache revoke и независимого restart обоих
runtimes восстанавливаются те же cursors без source body или candidate
presentation bytes. Live Tasks negative дополнительно доказывает, что
просроченный или недействительный custody receipt классифицируется как terminal
`BlobMismatch`, не превращается в retryable outage и не создаёт Task.
Aggregate gate закрыт после итогового clean-room аудита и полного pre-push на
финальном дереве реализации.
Отдельный live PostgreSQL conformance после остановки managed runtimes
доказывает exact approval/result duplicate replay, conflicting envelope/outbox,
unknown Tasks command и stale candidate correlation без публикации тестовых
outbox-записей в event flow.
Наличие документа, legacy task scanner, frontend card или отдельного extraction
result не открывает `communication_task_candidate_extraction_v1`.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0207](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0226](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0253](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0315](ADR-0315-communications-message-body-content-read.md);
- [ADR-0351](ADR-0351-review-communications-attention-owner-admission.md);
- [ADR-0356](ADR-0356-renewable-blob-authority-for-durable-ai-workflows.md);
- [ADR-0365](ADR-0365-communication-recipient-suggestion-workflow-and-source-boundary.md).

## Контекст

Legacy task-candidate path находился внутри Tasks implementation, читал
Communications и Documents persistence, смешивал deterministic text scanning,
Obligation Engine, generic Review records и немедленное создание Task при
подтверждении. Его полезное observable поведение состоит в другом: сообщение с
явным action/request signal создаёт reviewable candidate, а durable Task не
появляется до явного решения владельца.

Clean-room перенос не может вернуть этот код как Tasks facade над
Communications, поместить Tasks state в Communications или дать Review доступ к
чужим таблицам. Extraction, review decision и durable Task имеют разные причины
изменения и разных owners.

## Решение

### Owners и build units

Extraction принадлежит workflow owner
`communication_task_candidate_extraction` и состоит из пяти units:

- `makosh-communication-task-candidate-api` — generated Start/Get/realtime
  contract;
- `makosh-communication-task-candidate-core` — pure extraction lifecycle,
  bounded deterministic V1 rules и validation;
- `makosh-communication-task-candidate-persistence` — owner-local run state,
  inbox/outbox и realtime replay;
- `makosh-communication-task-candidate-runtime` — managed orchestration;
- `makosh-communication-task-candidate-assembly` — descriptor, settings,
  Storage bundle и release fragment.

Communications отдельно публикует exact source contract unit
`makosh-communications-task-source-api`; source producer остаётся в
Communications runtime и persistence. Workflow не импортирует Communications
implementation или storage.

Review получает отдельный task-candidate capability, не режим существующего
attention API:

- `makosh-review-task-candidate-api`;
- `makosh-review-task-candidate-core`;
- `makosh-review-task-candidate-persistence`;
- `makosh-review-task-candidate-runtime`;
- `makosh-review-task-candidate-assembly`.

Tasks получает минимальный production slice для создания Task только из
подтверждённого candidate:

- `makosh-tasks-command-api`;
- `makosh-tasks-core`;
- `makosh-tasks-persistence`;
- `makosh-tasks-runtime`;
- `makosh-tasks-assembly`.

Одинаковый domain owner у нескольких units не объединяет их функциональные
ответственности. Ни один domain package не импортирует implementation,
persistence или runtime другого owner. Cross-owner runtime flow использует
только typed durable commands/results/events и target-bound Blob custody.

### Extraction contract

Start принимает stable operation ID, canonical communication message ID и
expected active source revision. Get возвращает run/source identity, monotonic
revision, completeness и ordered bounded candidates.

Каждый candidate содержит:

- stable candidate ID и immutable candidate digest;
- bounded owner-visible title;
- optional bounded due-text hint без Calendar semantics;
- optional bounded assignee-label hint без Contact/Persona identity claim;
- exact source basis `subject`, `body` или `combined`;
- signal kind `explicit_action`, `direct_request` или `follow_up`;
- confidence basis points и source evidence reference.

Candidate не является Task, Obligation, Decision, Calendar event или accepted
business truth. V1 не содержит project ID, provider/account identity, parsed
assignee identity, authoritative deadline, arbitrary JSON/map, prompt, model или
provider selection.

### Deterministic extraction V1

V1 сохраняет bounded observable semantics legacy scanner без переноса его
cross-domain coupling. Core анализирует только validated UTF-8 subject/body из
exact Communications source receipt. Фиксированный versioned rule set находит:

1. explicit action markers (`action`, `task`, `действие`, `задача` и принятые
   локализованные варианты);
2. direct request marker вместе с action verb;
3. explicit follow-up/next-step marker.

Из каждого matched line получается не более одного candidate; duplicate
normalized title схлопывается детерминированно, порядок следует source order,
а отсутствие сигнала возвращает пустой список. Due text и assignee label
остаются hints. Obligation inference, broad natural-language extraction и AI
не входят в V1.

`AiContextReceiptV1`, AI Engine и Ollama не используются: ADR-0226 требует
distinct typed AI request только когда use case действительно использует AI.
Если measured quality потребует inference, новый revision обязан добавить
отдельные AI/provider contracts; Ollama останется concrete integration, а не
частью Communications, Tasks или workflow core.

### Event-only source, review и promotion flow

```text
Authenticated client Start
  -> communication_task_candidate_extraction client_rpc
  -> durable PrepareCommunicationTaskSource command
  -> Communications source producer
  -> target-bound Blob + typed prepared/rejected result event
  -> extraction workflow
  -> owner-local immutable candidate result
  -> durable SubmitTaskCandidateForReview command
  -> Review task-candidate owner
  -> explicit owner approve/reject command through Gateway
  -> durable TaskCandidateApprovedForPromotion event
  -> reviewed_task_candidate_promotion workflow
  -> Tasks CreateTaskFromReviewedCandidate command consumer
  -> Tasks owner-local Task + durable terminal result
  -> reviewed_task_candidate_promotion workflow
  -> Review-owned promotion result
  -> Review promotion projection
```

Private source bytes и candidate presentation fields не попадают в durable
envelopes. Communications передаёт source через target-bound Blob workflow
audience. Workflow передаёт exact candidate payload в новую target-bound Blob
custody Review. После approval Review создаёт новую Tasks-target-bound custody;
Tasks сверяет candidate ID/digest, decision revision, human owner и fresh
runtime/grant coordinates до mutation.

Reject не вызывает Tasks. Approve не означает, что Task создан: terminal Tasks
result приходит отдельным durable event. Review хранит decision и promotion
status; Tasks хранит только durable Task и command receipt; extraction workflow
хранит только run/candidate result. Преобразование Review event в Tasks command
и terminal Tasks result в Review-owned promotion result выполняет отдельный
workflow по ADR-0368. Клиент композирует эти public projections, а Gateway не
становится task-candidate facade.

### Review semantics

Task-candidate Review имеет states `pending`, `approved`, `rejected` и отдельный
promotion status `not_requested`, `pending`, `succeeded`, `failed`. Это exact
capability Review owner, а не расширение generic attention oneof и не запись в
workflow/Tasks таблицу.

Approve/reject требует expected Review revision и authenticated human owner.
Exact duplicate operation replayable; conflicting reuse operation ID и stale
revision отклоняются. После terminal approve/reject решение immutable в V1.

### Tasks command semantics

`CreateTaskFromReviewedCandidateCommandV1` является exact durable command, а не
generic `create(entity_kind, payload)`. Он принимает opaque candidate/decision
references, digests, source evidence reference и Tasks-bound Blob receipt.
Tasks создаёт bounded title, optional due-text/assignee hints и provenance;
hints не материализуют Calendar, Contact, Persona, Project или Obligation.

Idempotency ключуется `(logical_owner_id, approved_candidate_id)`. Duplicate с
тем же digest возвращает тот же Task; conflicting digest отклоняется. Tasks не
читает Communications, Review или workflow storage и не вызывает их runtime.

### Persistence, replay и fences

Каждый owner имеет собственные inbox/outbox, request fingerprints и atomic
state/result transaction. Inbox сверяет event ID и exact envelope hash до
mutation; outbox хранит exact `DurableEnvelopeV1` bytes без re-encode.

Runtime generation, grant epoch, Storage binding, Blob custody и event route
проверяются на каждом внешнем шаге. Restart восстанавливает только non-terminal
work текущего authenticated owner. Suspend, revoke, stale source revision,
stale review revision, stale Blob proof или stale runtime coordinate не может
создать либо повторно создать Task.

Client terminal extraction/review/promotion notifications идут через общий
replayable SSE. Periodic polling не вводится. SSE содержит только stable IDs,
revisions, typed states и error codes, но не source/candidate private text.

### Kernel agreement

Kernel, Gateway и Event Hub остаются owner-neutral. Новые Kernel API,
owner-specific imports, generic business router или payload interpretation не
добавляются. Kernel выдаёт только существующие exact capability grants,
runtime generations, signed admission, Storage/Vault/Blob coordinates и NATS
routes. Gateway компилирует generated client contracts только в owner adapters,
но не extraction rules, Review semantics или Tasks core.

## Phase gate

`communication_task_candidate_extraction_v1` становится `implemented` только
атомарно после:

1. пяти отдельных extraction workflow units;
2. distinct Communications task-source contract и runtime producer;
3. пяти отдельных Review task-candidate units без attention facade;
4. пяти отдельных Tasks units с exact reviewed-candidate command;
5. typed event-only source, submission, decision, promotion и terminal-result
   routes с target-bound Blob custody;
6. deterministic multilingual extraction, empty-result and dedup conformance;
7. owner-local persistence, inbox/outbox, replay и recovery каждого owner;
8. signed release admission всех runtime/storage artifacts;
9. authenticated Gateway Start/Get/Review commands и shared replayable SSE;
10. end-to-end managed proof, что extraction не создаёт Task до approve, reject
    никогда не создаёт Task, approve создаёт ровно один source-backed Task;
11. wrong-owner, stale-source, stale-review, request conflict, duplicate event,
    Blob expiry, restart, revoke, grant/generation fence и privacy negatives;
12. architecture, Cargo boundaries, formatting, Clippy, workspace/integration,
    frontend и full pre-push gates.

Skeleton, legacy scanner внутри Tasks/Communications, direct service call,
shared SQL, generic Review record, generic entity command, frontend-only card
или extraction-only result не открывают aggregate gate.

## Последствия

- Communications остаётся canonical evidence/source owner.
- Extraction остаётся workflow, а не Communications или Tasks submodule.
- Review владеет human decision, Tasks — durable Task truth.
- AI/Ollama не добавляются без отдельного measured revision.
- Полный gate крупнее одного runtime, зато не скрывает незавершённую promotion
  цепочку за facade или UI.

## Отклонённые варианты

### Вернуть legacy task scanner в Tasks

Требует чтения Communications storage и смешивает extraction, review и Task
mutation в одном domain implementation.

### Хранить candidate и review state в Communications

Делает Communications владельцем Tasks/Review semantics и нарушает SRP.

### Подтверждать candidate прямым workflow-to-Tasks RPC

Обходит Review owner, не оставляет durable decision evidence и связывает два
runtime синхронной доступностью.

### Использовать существующий Review attention oneof

Attention и task promotion имеют разные lifecycle, payload, privacy и retry
semantics. Mode switch превратил бы attention API в generic Review facade.

### Сразу вызвать Ollama

V1 имеет bounded deterministic reference behavior. Provider dependency без
измеренной необходимости расширяет grants, privacy surface и failure modes.
