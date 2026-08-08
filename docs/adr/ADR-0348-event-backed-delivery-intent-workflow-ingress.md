# ADR-0348: Event-backed delivery-intent workflow ingress

Статус: Принято

Дата: 2026-07-30

Состояние реализации: generated provider-neutral contract build unit
`makosh-communication-delivery-intent-ingress-api` реализован. Он фиксирует
три exact durable contracts, direction-specific route requests, compile-time
Blob audience и валидируемые `DurableEnvelopeV1` outbox records без plaintext
или provider identity. Cross-channel persistence реализует exact submit
outbox, source-result inbox/hash fence и atomic dispatch transition после
custody transfer. Cross-channel managed runtime реализует verified source Blob
read, fixed delivery-intent target-bound write, submit producer, durable relay
и ACK-after-commit. Delivery-intent runtime реализует exact submit consumer,
source/runtime/correlation validation, target-bound Blob read, canonical route
resolution, provider-bound rematerialization, owner-local inbox/hash fence,
atomic `inbox + intent + result outbox`, durable result relay и ACK-after-commit
либо exact duplicate. Cross-channel runtime реализует exact
`submitted/rejected` consumers, атомарный terminal transition и durable cleanup
исходной Communications custody. Delivery-intent runtime атомарно ставит
входящую target-bound custody в собственную cleanup queue и освобождает её через
Blob platform authority с bounded retry. Submit command сохраняет
deterministic intent/correlation identity, но использует отдельный
deterministic message ID, чтобы не конфликтовать с source-prepare command в
owner-local outbox и JetStream deduplication. Эти границы подтверждены focused
tests и disposable PostgreSQL reconnect conformance. Live managed end-to-end
evidence ещё не реализован. Наличие ADR и отдельных contract/runtime build
units не открывает ни `communication_delivery_intent_v1`, ни
`communication_cross_channel_forward_v1`.

## Контекст

`makosh-communication-delivery-intent-api` является client-facing
ConnectRPC contract. Его `SubmitDeliveryIntentRequestV1` принимает private
`body_utf8`, что допустимо на client boundary, но недопустимо для
workflow-to-workflow communication.

Cross-channel forward не может:

- вызывать delivery-intent client RPC из module runtime;
- импортировать delivery-intent runtime, persistence или private core;
- помещать plaintext body в `DurableEnvelopeV1`;
- передавать arbitrary provider, account, recipient или target module;
- использовать generic workflow command facade.

Нужен отдельный event-only ingress, который сохраняет ownership двух workflow,
не смешивает client contract с module contract и передаёт content только через
target-bound Blob custody.

## Решение

Вводится отдельная delivery-intent-owned contract build unit:

```text
makosh-communication-delivery-intent-ingress-api
```

Она содержит три exact durable contracts:

```text
communication_delivery_intent_submit.v1     command
communication_delivery_intent_submitted.v1  result
communication_delivery_intent_rejected.v1   result
```

Submit payload содержит только:

- deterministic 16-byte `intent_id`;
- bounded ASCII `logical_owner_id`;
- canonical `target_conversation_id`;
- optional canonical `target_reply_to_message_id`;
- target-bound Blob `reference_id`, declared bytes, SHA-256 и custody proof.

Plaintext, provider/account identity, provider locator, recipients, subject,
credentials, arbitrary owner/module/capability, `Any` и generic maps
отсутствуют.

Contract compile-time фиксирует Blob audience:

```text
target owner      communication_delivery_intent
target module     makosh-communication-delivery-intent-runtime
target capability communication_delivery_intent.blob.v1
```

Durable submit command адресуется exact module capability:

```text
communication_delivery_intent.event-ingress.v1
```

`submitted` означает только durable admission delivery-intent workflow и
содержит intent identity плюс logical owner. `rejected` содержит те же
identities и закрытый admission code: invalid request, custody invalid,
canonical target unavailable, policy или unavailable. Provider completion
остаётся состоянием delivery-intent и не подменяется ingress result.

## Durable flow

```text
cross-channel workflow source result inbox
→ cross-channel target-bound Blob read
→ delivery-intent target-bound Blob write
→ cross-channel outbox
→ NATS JetStream
→ delivery-intent inbox
→ owner-local durable admission
→ delivery-intent result outbox
→ NATS JetStream
→ cross-channel result inbox
```

Producer сохраняет exact `DurableEnvelopeV1` bytes до publish. Consumer
проверяет exact contract/source/capability, inbox message ID/hash и Blob
receipt до state mutation. Delivery-intent admission, inbox и submitted или
rejected result outbox коммитятся атомарно. ACK разрешён только после commit
либо exact duplicate.

Command ID, partition key и correlation ID равны `intent_id`. Message ID
детерминирован отдельным hash namespace от `intent_id`, потому что тот же
cross-channel operation ранее публикует source-prepare command. Redelivery
exact bytes не создаёт второй intent. Тот же message ID с другим hash, другой
payload для существующего intent ID или stale runtime fence отклоняются
fail-closed.

## Ownership и SRP

- ingress API владеет только Protobuf schema, exact route requests и envelope
  builders;
- cross-channel runtime является producer и не импортирует delivery-intent
  implementation;
- delivery-intent runtime adapter является consumer и не импортирует
  cross-channel implementation;
- каждый workflow хранит только собственные inbox/outbox/state;
- Blob остаётся platform custody authority;
- Kernel/Event Hub авторизуют exact routes и fences, но не декодируют payload;
- Core Gateway не участвует в module-to-module ingress.

Integration не является domain. Domain не является integration. Этот contract
не выбирает Mail, Telegram, WhatsApp или Zulip; provider route по-прежнему
разрешает только delivery-intent workflow через Communications evidence и
отдельные integration-owned event contracts.

## Completion gate

Ingress считается реализованным только после:

1. отдельной generated contract build unit;
2. exact command/result envelope builders и directional route requests;
3. cross-channel transactional submit outbox;
4. delivery-intent inbox/hash fencing и owner-local admission;
5. target-bound Blob custody transfer и cleanup;
6. submitted/rejected result consumer в cross-channel workflow;
7. duplicate/conflict, outage, restart, stale generation и privacy tests;
8. live managed NATS/Storage/Blob proof без direct RPC или cross-owner SQL.
