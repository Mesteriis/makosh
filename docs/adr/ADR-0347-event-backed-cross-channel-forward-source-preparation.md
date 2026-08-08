# ADR-0347: Event-backed cross-channel forward source preparation

Статус: Принято

Дата: 2026-07-30

Состояние реализации: generated provider-neutral contract build unit
`makosh-communications-cross-channel-forward-source-api` реализован. Он
фиксирует три exact durable contracts, direction-specific route requests,
target-bound Blob audience и валидируемые `DurableEnvelopeV1` outbox records
без provider identity или plaintext. Communications persistence теперь
предоставляет owner-local source/target snapshot, private channel-kind
comparison, revision fencing и атомарный inbox/result-outbox commit с
duplicate/hash-conflict semantics. Communications runtime adapter реализует
exact command decode, Blob read/hash/UTF-8 verification, fixed target-bound
Blob write, prepared/rejected result persistence и ACK-after-commit.
Admission wiring в managed pump, workflow inbox adapter и live managed
evidence ещё не реализованы; contract/runtime gate не закрыт.

## Контекст

ADR-0346 выделил cross-channel forward в самостоятельный workflow. Для
получения private source content workflow не может:

- импортировать Communications domain/runtime/persistence;
- читать Communications tables или Blob custody другого owner;
- использовать generic content/read-all API;
- передавать body через Core RPC;
- синхронно вызывать Communications mutation через `request_rpc`.

Последний вариант технически маршрутизируем, но создаёт request chain между
business owners и не удовлетворяет event-only границе clean-room. В проекте
уже существует проверенный precedent: Communications evidence export
подготавливает target-bound Blob source через durable command/result.

## Решение

Вводится отдельная Communications-owned contract build unit:

```text
makosh-communications-cross-channel-forward-source-api
```

Она содержит три exact durable contracts:

```text
cross_channel_forward_source_prepare.v1   command
cross_channel_forward_source_prepared.v1  result
cross_channel_forward_source_rejected.v1  result
```

Prepare payload содержит только:

- `forward_id`;
- canonical `source_message_id`;
- canonical `target_conversation_id`;
- `logical_owner_id`.

Target workflow не выбирается payload. Contract unit compile-time фиксирует:

```text
target owner      communication_cross_channel_forward
target module     makosh-communication-cross-channel-forward-runtime
target capability communication_cross_channel_forward.blob.v1
```

Prepared result содержит:

- те же correlation identities;
- canonical source evidence ID и revision;
- target-bound Blob `reference_id`;
- declared byte count;
- plaintext SHA-256;
- custody-transfer source proof.

Plaintext, provider/account identity, provider locators и arbitrary target
owner отсутствуют. Rejected result содержит только bounded error code:
invalid request, source missing/inactive, target missing, same channel,
content unavailable, content limit или policy.

## Durable flow

```text
cross-channel workflow outbox
→ NATS JetStream
→ Communications inbox
→ canonical source/target validation
→ target-bound Blob write
→ Communications result outbox
→ NATS JetStream
→ cross-channel workflow inbox
```

Producer сохраняет exact `DurableEnvelopeV1` bytes в owner-local outbox до
publish. Consumer проверяет inbox message ID/hash до mutation и ACK только
после commit. Redelivery не создаёт второй Blob source: idempotency scoped
`(logical_owner_id, forward_id)`, а conflicting command hash reject-ится.

Communications проверяет, что source message active, target conversation
существует у того же logical owner и source/target принадлежат разным admitted
channel kinds. Provider identity не публикуется: comparison остаётся внутри
canonical owner.

## Согласование с Kernel/Core

Kernel и Event Hub:

- авторизуют exact publish/consume routes, schema digest и current
  runtime/grant generation;
- не декодируют business payload;
- не выбирают provider или target owner;
- не retry-ят mutation через RPC;
- не становятся content proxy или business facade.

Core Gateway не участвует в module-to-module source preparation. Клиент
получает только workflow receipt/status/replayable SSE из ADR-0346.

## SRP и compile isolation

- source API владеет только Protobuf schema, envelope builders и exact route
  requests;
- Communications runtime adapter владеет canonical validation и target-bound
  Blob write orchestration;
- Communications persistence владеет inbox/outbox и idempotency;
- cross-channel runtime adapter владеет result admission;
- cross-channel persistence хранит receipt/hash/retry, но не body;
- ни одна unit не импортирует implementation другого owner или integration.

Integration не является domain. Domain не является integration. Provider
runtime не участвует в source preparation.

## Completion gate

Source-preparation seam считается реализованным только после:

1. generated provider-neutral contract build unit;
2. exact command/result envelope builders и route requests;
3. Communications transactional inbox/outbox;
4. target-bound Blob write с fixed consumer binding;
5. workflow result inbox с duplicate/conflict fencing;
6. NATS outage, redelivery, restart, stale generation и privacy tests;
7. live managed proof без direct RPC, cross-owner SQL или content leakage.

ADR сам по себе не открывает `communication_cross_channel_forward_v1`.
