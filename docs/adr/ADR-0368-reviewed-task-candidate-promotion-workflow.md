# ADR-0368: Reviewed task candidate promotion workflow

Статус: Принято

Дата: 2026-08-01

Состояние реализации: implemented после final full pre-push и clean-room audit.
Review-owned terminal promotion-result API,
pure workflow correlation core, owner-local workflow persistence, managed
event runtime, Review-owned terminal-result consumer и unsigned workflow
assembly реализованы как отдельные compile-isolated units. Persistence
атомарно связывает approval inbox с Tasks command outbox и Tasks
terminal-result inbox с Review result outbox, не сохраняя candidate content,
Blob proof или provider identity. Runtime запрашивает только пять exact event
routes и owner-local Storage, без client, realtime или Blob authority.
Review consumer принимает только exact Review-owned event, атомарно сохраняет
inbox и Review realtime transition и делает ack после commit. Workflow runtime
и Storage bundle включены в signed distribution отдельными artifacts; Kernel
managed admission доказан вместе с extraction, Review и Tasks через real
Vault/Storage/NATS readiness. Live managed E2E доказывает approve/reject через
typed Gateway routes, начиная с реального Communications source и generated
extraction Start/Get без прямого seed Review, exact event-only workflow, ровно
один Task для approve, отсутствие Task до approve и для reject, Review
`succeeded`, extraction/Review frames в shared SSE и replay тех же cursors
после owner cache revoke и независимого рестарта extraction и Review runtimes.
Тот же contour доказывает wrong-owner, stale source/review,
request duplicate/conflict, runtime generation/grant fences и отсутствие
private source/candidate presentation bytes в SSE. Tasks принимает
target-bound candidate Blob только после собственного custody transfer по
exact command evidence; прямое чтение Review reference запрещено. Promotion
client boundary теперь проверяет persisted operation до current Review
revision/state: exact retry после terminal promotion replay-ит сохранённую
операцию, а reuse operation ID с другим request hash/fingerprint fail closed
как `operation_conflict` без повторного решения или Task. Live managed gate
доказывает оба случая. Feature-gated live persistence conformance на disposable
PostgreSQL дополнительно доказывает exact approval/result duplicate,
conflicting envelope/outbox, unknown Tasks command и stale candidate
correlation; duplicate terminal result теперь сверяет и inbox, и сохранённый
Review-result outbox до replay. Promotion gate закрыт итоговым clean-room
аудитом и полным pre-push. Отдельный live Tasks negative подтверждает terminal
`BlobMismatch` для stale/expired custody receipt без Task materialization;
transport `Unavailable` остаётся retryable.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0207](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0366](ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md).

## Контекст

ADR-0366 задаёт правильные owner boundaries: Review владеет human decision,
Tasks владеет Task truth, а взаимодействие происходит через durable events и
commands. Реализованный Review runtime публикует
`TaskCandidateApprovedForPromotionV1`, а Tasks runtime принимает только
`CreateTaskFromReviewedCandidateCommandV1`. Это два разных public contracts.

Kernel, Gateway или один из domains не может преобразовать первый контракт во
второй:

- Kernel и Gateway owner-neutral и не интерпретируют business payload;
- Review не должен импортировать Tasks implementation или выдавать target
  command от имени workflow;
- Tasks не должен импортировать Review implementation или читать Review SQL;
- прямой adapter внутри любого domain снова смешал бы decision и Task mutation.

Поэтому отсутствующий переход является отдельным cross-owner workflow, а не
helper, facade или расширением одного из domains.

## Решение

Ввести owner `reviewed_task_candidate_promotion` с role `workflow` и пять
отдельных build units:

1. `makosh-review-task-candidate-promotion-api` — Review-owned exact terminal
   promotion result contract, доступный producer workflow и consumer Review;
2. `makosh-reviewed-task-candidate-promotion-core` — pure deterministic mapping
   и correlation rules без transport, SQL, Blob и domain implementations;
3. `makosh-reviewed-task-candidate-promotion-persistence` — owner-local
   PostgreSQL inbox/outbox и durable correlation между approval, Tasks command
   и terminal result;
4. `makosh-reviewed-task-candidate-promotion-runtime` — managed event adapter;
5. `makosh-reviewed-task-candidate-promotion-assembly` — descriptor, empty
   typed Settings schema, Storage bundle и unsigned release fragment.

Review promotion contract unit принадлежит Review contract surface, но не
содержит Review core/persistence/runtime. Остальные четыре units принадлежат
workflow. Runtime может импортировать только public Review, Tasks и platform
contracts плюс собственные core/persistence; ни одна domain implementation
dependency не разрешена.

### Event flow

```text
Review approved event
  -> promotion workflow inbox
  -> deterministic Tasks command
  -> promotion workflow outbox
  -> Tasks command consumer
  -> Tasks owner-local mutation or rejection
  -> typed Tasks terminal result
  -> promotion workflow result inbox
  -> Review-owned promotion result
  -> promotion workflow outbox
  -> Review promotion-result consumer
  -> Review owner-local promotion projection + replayable SSE
```

Workflow не читает candidate Blob и не получает Blob capability. Он переносит
opaque Tasks-target-bound receipt из already authenticated Review event в exact
Tasks command без re-encode содержимого. Только Tasks получает custody и читает
candidate bytes.

### Correlation и idempotency

Workflow command identity детерминированно зависит от exact approval event
message ID, Review ID, candidate ID и decision revision. Approval inbox и Tasks
command outbox сохраняются одной транзакцией. Повтор exact approval возвращает
тот же command; reuse event ID с другим envelope hash или payload отклоняется.

Tasks terminal result принимается только если его command ID/message ID и
logical owner совпадают с сохранённой workflow correlation. Result inbox и
Review promotion-result outbox также сохраняются атомарно. Duplicate exact
result replayable; conflicting hash, unknown command или stale correlation
fail closed.

Review promotion result содержит только review/candidate IDs, expected decision
revision, bounded outcome и optional Task ID. Title, hints, source body, Blob
proof, provider/account identity и private content в нём запрещены. Review
сверяет owner, current pending promotion и expected revision до mutation.

### Runtime и admission

Workflow descriptor запрашивает шесть независимых capabilities:

- required consumer Review approved event;
- publisher Tasks create command;
- required consumer Tasks created result;
- required consumer Tasks rejected result;
- publisher Review promotion result;
- owner-local Storage namespace.

Runtime не предоставляет client RPC, realtime или Blob surface. Review client
по-прежнему видит projection только через Review query и shared replayable SSE.
Periodic polling и handwritten REST не вводятся.

## Phase gate

`reviewed_task_candidate_promotion_v1` становится implemented только после:

1. пяти exact build units и compile isolation;
2. versioned typed Review promotion-result contract;
3. atomic workflow approval inbox/Tasks outbox;
4. atomic workflow Tasks-result inbox/Review outbox;
5. Review runtime consumer и owner-local promotion transition;
6. one signed release и distinct managed workflow admission;
7. Gateway approve/reject и shared SSE E2E;
8. доказательства: до approve Task отсутствует, reject не создаёт Task,
   approve создаёт ровно один Task и Review становится `succeeded`;
9. duplicate/conflict, wrong-owner, stale revision, unknown command, restart,
   revoke, generation/grant и privacy negatives;
10. architecture, Cargo, unit, persistence, managed runtime и full pre-push
    gates.

После закрытия этого gate aggregate
`communication_task_candidate_extraction_v1` имеет состояние `implemented`.

## Последствия

- Domains остаются автономными и не импортируют implementation друг друга.
- Kernel/Gateway/Event Hub не становятся business mediator или facade.
- Workflow имеет собственные release, Storage, restart и revoke boundaries.
- Terminal Tasks result становится наблюдаемым Review projection, а не
  предположением после accepted command.
- Добавляется отдельный runtime, но его responsibility и authority exact и
  проверяемы.

## Отклонённые варианты

### Review напрямую публикует Tasks command

Смешивает human decision owner и cross-owner orchestration, а Review начинает
выбирать target-domain command.

### Tasks напрямую читает Review state или SQL

Нарушает owner-local storage, event-only boundary и независимый restart.

### Kernel или Gateway преобразует payload

Создаёт generic business facade в owner-neutral control/client plane.

### Считать approve успешным созданием Task

Accepted command не является terminal result и скрывает outage/rejection между
Review и Tasks.
