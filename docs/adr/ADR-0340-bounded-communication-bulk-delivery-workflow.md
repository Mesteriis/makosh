# ADR-0340: Bounded communication bulk delivery workflow

Статус: Принято

Дата: 2026-07-29

Состояние реализации: implemented. Gate
`communication_bulk_action_v1` открыт атомарно вместе с отдельными contract,
core, persistence, managed runtime и assembly units. Live managed contour
доказывает Gateway Start/Status, replayable SSE `accepted -> completed`,
capability-routed `request_rpc` в отдельный delivery-intent runtime и
восстановление stable cursor после restart.

Уточняет:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0214](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0330](ADR-0330-provider-neutral-communication-delivery-intent-workflow.md);
- [ADR-0339](ADR-0339-capability-routed-module-request-rpc.md).

## Контекст

Clean-room reconstruction требует отдельный `communication_bulk_action`
workflow с bounded fan-out и receipt для каждой цели. Это не bulk SQL update
Communications и не provider batch API.

Communications владеет canonical evidence/read model, delivery-intent workflow
владеет одним provider-neutral outbound intent, а Mail, Telegram, WhatsApp и
Zulip владеют provider execution. Bulk orchestration не получает права читать
их storage или импортировать implementation packages.

Legacy bulk mail flags, archive/trash, labels, pin/snooze и provider-specific
batch commands не переносятся этим gate. Они остаются отдельными
integration/domain command capabilities. Текущий gate восстанавливает именно
зафиксированный ADR-0282 bounded fan-out нескольких delivery intents.

## Решение

Вводится отдельный workflow owner `communication_bulk_action` с единицами
сборки:

```text
makosh-communication-bulk-action-api
makosh-communication-bulk-action-core
makosh-communication-bulk-action-persistence
makosh-communication-bulk-action-runtime
makosh-communication-bulk-action-assembly
```

API содержит только generated Protobuf request/result contracts. Core
валидирует batch и target transitions. Persistence владеет durable batch,
targets и retry state. Runtime обслуживает client RPC/status и вызывает exact
public delivery-intent command через `request_rpc`. Assembly создаёт отдельный
signed runtime/storage fragment.

Ни одна из этих units не импортирует Communications implementation,
integration runtime/persistence или Kernel implementation.

## Public contract

### Start

Client передаёт:

- `protocol_major = 1`;
- exact non-zero 16-byte `batch_operation_id`;
- `1..=100` targets;
- для каждой цели exact non-zero 16-byte `target_operation_id`;
- canonical `conversation_id`;
- optional canonical `reply_to_message_id`;
- private UTF-8 body.

`target_operation_id` уникален внутри batch и является exact operation ID
соответствующего delivery intent. Общий encoded request ограничен client
transport ceiling 1 MiB; каждая target request после encoding обязана
помещаться в 64 KiB module `request_rpc`.

Start атомарно сохраняет batch и все targets до fan-out и возвращает
`accepted`/existing receipt. `accepted` не означает, что любой provider
принял, выполнил или доставил сообщение.

### Status

Status query возвращает:

- batch state и monotonic revision;
- bounded target page;
- для каждой цели только target operation ID, orchestration state,
  delivery-intent ID при acceptance и typed sanitized error;
- opaque continuation cursor.

Private body, provider/account identity, runtime coordinates и raw error text
не возвращаются.

## Durable execution

Target lifecycle:

```text
pending
  -> dispatching
  -> accepted
  -> rejected
  -> retryable
```

Runtime берёт одну bounded lease, вызывает public
`communication.delivery_intent.command` через `request_rpc` и сохраняет exact
receipt до следующей цели. Kernel не retry-ит mutation.

Timeout или потерянный response переводит target в `retryable`, потому что
delivery ambiguous. Повтор использует тот же `target_operation_id`; idempotency
принадлежит delivery-intent owner. Retry ограничен attempt ceiling и
deterministic backoff. Crash/restart снимает только expired lease и продолжает
неразрешённые targets.

Batch state вычисляется из durable target states:

- `accepted` — batch сохранён, есть pending/dispatching/retryable;
- `completed` — все targets получили delivery-intent acceptance;
- `completed_with_errors` — terminal mix accepted/rejected;
- `rejected` — ни одна цель не принята.

Terminal provider delivery не является состоянием bulk batch и наблюдается
через delivery-intent status/realtime.

## Security и privacy

- logical owner берётся из Kernel delivery, не из payload;
- body хранится только в owner-local bulk persistence до передачи
  delivery-intent и не попадает в logs/events/errors/status;
- request contract dependency и provider route exact;
- caller/provider grants, runtime generation и grant epoch проверяет Kernel;
- maximum 100 targets, 1 MiB client request, 64 KiB per target request,
  bounded status page и bounded attempts fail closed;
- cross-owner SQL, direct sockets, NATS business payload и generic maps/`Any`
  запрещены.

## Client realtime

Runtime публикует через общий managed `client_realtime` только client-safe
invalidation:

```text
batch_id
batch_state
state_revision
occurred_at_unix_millis
```

Target bodies, provider identity и per-target errors в realtime не попадают.
Клиент после invalidation читает typed paged status.

## Phase gate `communication_bulk_action_v1`

Gate открывается только вместе с:

1. пятью отдельными packages и exact Cargo boundaries;
2. generated Start/Status/realtime contracts и hard bounds;
3. additive owner-local Storage bundle;
4. atomic idempotent batch/target creation;
5. target lease, bounded retry и crash recovery;
6. exact delivery-intent `request_rpc` dependency;
7. per-target receipt/error persistence;
8. managed client RPC и shared SSE invalidation;
9. zero/oversized/duplicate/stale/revoke/timeout/restart negatives;
10. live managed contour через реальный delivery-intent runtime;
11. architecture, SRP, Cargo, Clippy и full test gates.

Gate открыт как `implemented` только после прохождения перечисленного evidence.
Принятый ADR сам по себе gate не открывает.

## Отклонённые варианты

### Bulk method в Communications

Делает canonical evidence owner оркестратором provider delivery и смешивает
domain с workflow.

### Provider batch API

Создаёт provider-dependent semantics и не работает для cross-channel batch.

### Один delivery-intent с repeated targets

Размывает idempotency, receipt и terminal state одного intent.

### Синхронно держать Start до завершения 100 requests

Нарушает bounded latency и теряет restart-safe progress.

### Generic `execute(action, targets)`

Возвращает untyped mediator и смешивает независимые action capabilities.
