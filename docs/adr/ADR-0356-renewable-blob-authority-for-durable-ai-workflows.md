# ADR-0356: Renewable Blob authority for durable AI workflows

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано в AI contracts, AI inference persistence,
`makosh-communication-reply-suggestion-runtime` и отдельной unsigned
`makosh-communication-reply-suggestion-assembly`. Workflow принимает
Communications event, переносит custody в собственный runtime, проверяет
bounded UTF-8 body, материализует отдельный AI-target-bound Blob, сохраняет
bounded request/cleanup receipts и подтверждает source event только после
terminal inference и освобождения обоих Blob. Semantic request digest не
включает короткоживущий custody proof; non-terminal exact replay может обновить
только этот proof. Dev release compiler подписывает exact runtime и Storage
artifacts. Signed Kernel admission и live negative managed orchestration
реализованы full-ensemble conformance: Communications event проходит в
workflow, custody переносится к AI, request маршрутизируется через отдельные AI
engine и Ollama integration, terminal cleanup завершается до replayable
Gateway/SSE result, а workflow restart возвращает exact terminal state без
повторного Ollama HTTP. Stale source отклоняется до provider boundary.
Успешный local provider inference и оставшаяся revoke/wrong-owner матрица ещё
не доказаны, поэтому `communication_reply_suggestion_v1` остаётся `planned`.

Уточняет:

- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0223](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0231](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0353](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md).

## Контекст

Communications source result выдаёт Blob proof, target-bound к
`communication_reply_suggestion`. AI engine имеет другой owner, module и Blob
capability. Передача исходного proof напрямую AI нарушила бы capability
binding. Одновременно custody proofs имеют ограниченный срок жизни, поэтому
сохранение proof как части неизменяемой business request identity делало бы
restart-safe выполнение невозможным.

Source event нельзя подтверждать сразу после записи `AwaitingInference`.
Иначе crash между Ack, AI terminal result и Blob cleanup оставил бы workflow
без replay authority либо с утечкой custody.

## Решение

Workflow выполняет две отдельные custody операции:

```text
Communications source Blob
  -> target-bound transfer to reply workflow
  -> exact bounded read and SHA-256 verification
  -> typed AiReplySourceContentV1 encoding
  -> new Blob target-bound to AI inference
  -> exact request_rpc to AI
```

Semantic request digest связывает:

- run и `AiContextReceiptV1`;
- source reference, declared size и SHA-256;
- tone, language, subject, output и local-only egress policies.

Digest не включает `custody_transfer_source_proof`. Proof остаётся
обязательным, bounded и структурно валидируемым authority material, но не
business identity. AI persistence принимает обновление proof только когда:

1. owner, run ID и semantic request полностью совпадают;
2. отличается только custody proof;
3. run остаётся `accepted` или `executing`.

Terminal replay не переписывает authority receipt.

Workflow persistence хранит только encoded bounded inference request и два
cleanup receipts. Source body, provider request, prompt и provider metadata в
таблицы, NATS, Gateway, SSE, logs и health не попадают.

После terminal результата workflow:

1. освобождает AI-target-bound Blob;
2. освобождает Communications-derived workflow Blob;
3. атомарно отмечает cleanup завершённым и удаляет persisted proofs;
4. только затем подтверждает source event.

Crash до Ack приводит к redelivery. Exact inbox/hash fence, semantic replay и
детерминированные cleanup operation IDs продолжают тот же run без повторного
business intent.

## Units и SRP

- Communications создаёт только source evidence и target-bound handoff;
- reply workflow координирует custody, AI request и cleanup;
- Blob Platform выдаёт и проверяет authority, но не читает business payload;
- AI engine проверяет typed request и выполняет inference;
- provider integration не видит Communications contract или workflow state;
- Gateway доставляет client-safe invalidation через общий replayable SSE.

## Typed source content

ADR-0357 перевёл Communications source contract на coordinated revision 2.
Communications формирует один bounded typed sender/subject/body Blob из одного
canonical evidence snapshot, а workflow после custody transfer декодирует его
и создаёт отдельный AI-target-bound `AiReplySourceContentV1`. Raw private
content по-прежнему не сохраняется в workflow persistence и не публикуется в
event payload.

## Phase gate

Workflow gate открывается только после:

1. unsigned assembly и signed exact Kernel admission;
2. live Communications event → workflow → AI request → terminal result;
3. restart/redelivery proof с renewal без request conflict;
4. доказанного terminal cleanup обоих Blob до Ack;
5. успешного local provider inference, а не только negative unavailable path;
6. architecture, Cargo, Clippy, tests и frontend generated-client gates.

## Отклонённые варианты

### Передать Communications proof напрямую AI

Нарушает target owner/module/capability binding.

### Включить proof в semantic digest

Делает authority lease равным business identity и ломает допустимое renewal
после restart.

### Ack после source persistence

Теряет durable recovery contour для inference и custody cleanup.

### Сохранить source body в workflow или AI persistence

Расширяет private-content boundary и создаёт лишнюю canonical copy.
