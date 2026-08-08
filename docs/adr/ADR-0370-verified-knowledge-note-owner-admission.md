# ADR-0370: Verified Knowledge Note owner admission

Статус: Принято

Дата: 2026-08-01

Состояние реализации: staged. Этим решением Knowledge разблокирован только для
exact verified-note contract/core/persistence/runtime/assembly slice. Gateway
surface, shared SSE и live conformance ещё не реализованы; Knowledge
phase gate остаётся закрытым до атомарного evidence полного owner contour.

Уточняет:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0207](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0208](ADR-0208-domain-development-allowlist-and-projection-freeze.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0369](ADR-0369-communication-note-candidate-extraction-and-reviewed-knowledge-promotion.md).

## Контекст

Knowledge зарегистрирован ADR-0207 как владелец проверенного долговечного
знания и его evidence, но ADR-0208 запрещал любой production package, contract,
schema или runtime этого owner. `communication_note_candidate_extraction_v1`
теперь требует целевого домена: Review approve не может создать durable truth
сам, а Communications и workflow не могут временно присвоить Knowledge state.

Legacy Notes были document-like capture artifacts, а legacy Knowledge Graph
смешивал accepted facts с rebuildable projections. Ни тот, ни другой boundary
нельзя перенести как facade. Для текущего use case нужен только узкий canonical
объект, который существует после human Review и сохраняет exact provenance.

## Решение

### Узкая разблокировка owner

`knowledge` добавляется в executable development allowlist и удаляется из
blocked list. Разблокировка разрешает только принятый здесь verified-note
contour:

- `makosh-knowledge-command-api` — exact durable command/result contract;
- `makosh-knowledge-core` — pure verified-note aggregate and invariants;
- `makosh-knowledge-persistence` — owner-local inbox/state/outbox и Storage
  bundle;
- `makosh-knowledge-runtime` — managed event consumer, Blob custody client и
  exact outbox relay;
- `makosh-knowledge-assembly` — unsigned descriptor/settings/storage/runtime
  release fragment без signing authority.

Generic note CRUD, Knowledge Graph, Search, Timeline, Context, Memory,
embeddings, arbitrary facts/claims и cross-domain materialized views этим ADR
не разрешаются. Projection freeze ADR-0208 остаётся в силе.

### Canonical entity

`VerifiedKnowledgeNoteV1` — immutable V1 record, созданный только из terminal
approved Review candidate. Он содержит:

- deterministic `note_id` и revision `1`;
- bounded title и excerpt;
- закрытый ordered set topic hints `financial`, `legal`,
  `decision_statement`, `deadline_statement`;
- source basis и confidence basis points;
- exact candidate digest, Communications evidence reference, Review decision
  revision и authenticated owner-device evidence;
- created timestamp и logical human owner.

Topic hint не создаёт Decision, Obligation, Document, Task или Calendar truth.
`decision_statement` остаётся классификацией текста, а не принятой Decision.
Verified note не копирует provider/account identity и не становится глобальной
projection.

V1 не вводит edit/delete/merge. Любое изменение content или knowledge lifecycle
требует отдельного command revision и ADR, чтобы reviewed source нельзя было
тихо переписать.

### Exact command boundary

Promotion workflow публикует
`CreateKnowledgeNoteFromReviewedCandidateCommandV1`. Durable envelope содержит
только stable IDs, digests, revisions, actor evidence и
Knowledge-target-bound Blob receipt. Plaintext presentation находится внутри
`ReviewedKnowledgeNoteContentV1` в target-bound Blob и никогда не попадает в
NATS subject, envelope metadata, SSE, log или error.

Knowledge возвращает один из exact terminal results:

- `KnowledgeNoteCreatedFromReviewedCandidateV1`;
- `KnowledgeNoteCreationFromReviewedCandidateRejectedV1`.

Idempotency задаётся `(logical_owner_id, approved_candidate_id)`. Exact replay
возвращает тот же note/result bytes; conflicting candidate digest или provenance
отклоняются. Accepted command не означает creation до terminal result.

### Owner и Kernel agreement

Knowledge runtime будет отдельным managed Domain process с owner-local
PostgreSQL role/schema, inbox/outbox и Storage/Vault binding. Он не импортирует
Communications, Review, workflow или provider packages и не читает их SQL.
Promotion workflow импортирует только Knowledge public command contract.

Kernel, Gateway и Event Hub остаются owner-neutral: они проверяют descriptor,
grant, runtime/storage generation, exact event routes и client transport, но не
декодируют note payload и не создают Knowledge facade. Cross-owner path остаётся
event-only; module-to-module RPC и polling не вводятся.

### Flow

```text
Review approved event + promotion-workflow-bound Blob
  -> reviewed_note_candidate_promotion workflow
  -> Knowledge-target-bound Blob
  -> CreateKnowledgeNoteFromReviewedCandidate durable command
  -> Knowledge owner-local inbox + note + outbox atomically
  -> terminal created/rejected result
  -> promotion workflow
  -> Review-owned promotion result
  -> shared replayable SSE
```

## Phase gate

Knowledge verified-note gate может стать implemented только после:

1. contract/core/persistence/runtime/assembly существуют как отдельные units;
2. exact signed descriptor, settings, Storage bundle и capability grants
   admitted Kernel;
3. owner-local inbox/outbox/state и duplicate/conflict replay атомарны;
4. Knowledge-target Blob custody, expiry/mismatch и cleanup доказаны;
5. managed E2E создаёт ноль notes до approve и после reject, ровно одну после
   approve;
6. wrong owner, stale review/source, runtime/grant/storage generation и restart
   fences доказаны;
7. generated client query и shared SSE не раскрывают plaintext;
8. architecture, Cargo, managed-runtime и full pre-push gates зелёные.

Наличие allowlist entry, ADR, contract/core или frontend skeleton само по себе
не открывает production phase gate.

## Последствия

- Knowledge получает одного канонического владельца verified note truth.
- Communications остаётся source evidence owner, Review — human decision owner,
  promotion — workflow, а Knowledge — target domain.
- Legacy Notes/Graph не возвращаются как facade или generic aggregate.
- Следующий implementation slice может безопасно добавить exact command/core,
  не обходя blocked-domain guard.
