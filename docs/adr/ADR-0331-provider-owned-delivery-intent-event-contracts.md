# ADR-0331: Provider-owned delivery-intent event contracts

Статус: Принято

Дата: 2026-07-29

Состояние реализации: четыре независимые integration-owned contract build
units, их generated Protobuf descriptors, schema hashes и exact
publish/consume route requests реализованы. Четыре workflow publisher/result
adapter, transactional outbox и idempotent terminal-result inbox реализованы
по ADR-0332. Target-bound Blob materialization, четыре provider runtime
consumers, exact provider descriptor routes, operational execution и
terminal-result outbox relay реализованы по ADR-0333 и ADR-0335. Workflow
Event Hub loops, Gateway client closure и полное live managed evidence ещё не
реализованы. Поэтому
`communication_delivery_intent_v1` остаётся `planned`.

## Контекст

ADR-0330 вводит provider-neutral workflow
`communication_delivery_intent`. После canonical planning workflow должен
передать один intent ровно той integration, которой принадлежит исходный
provider route. Прямой вызов Mail, Telegram, WhatsApp или Zulip runtime,
integration store либо provider SDK нарушил бы process и owner isolation.
Один общий `provider_delivery` contract с discriminator создал бы facade,
который меняется при изменении любого provider.

Plaintext body также нельзя помещать в durable event. При этом integration
должна получить provider-readable body без доступа к workflow storage и без
cross-owner Blob read. Для этого уже существует target-bound custody proof по
ADR-0257 и ADR-0275.

## Решение

Каждая integration владеет собственной contract build unit:

```text
makosh-mail-delivery-intent-contract
makosh-telegram-delivery-intent-contract
makosh-whatsapp-delivery-intent-contract
makosh-zulip-delivery-intent-contract
```

Это четыре самостоятельные единицы сборки с разными owners, Protobuf packages,
schema hashes, command/result names и capability routes. Общей
provider-enum/union crate, generic adapter API или facade нет.

Каждая unit определяет три durable контракта:

```text
<provider>_delivery_intent_execute    command
<provider>_delivery_intent_succeeded  result
<provider>_delivery_intent_rejected   result
```

Execute payload содержит только:

- exact 16-byte `intent_id`;
- bounded ASCII `logical_owner_id`;
- exact 32-byte opaque account и conversation source cursors;
- optional exact 32-byte reply source cursor;
- source Blob reference ID, declared size и SHA-256 receipt;
- bounded target-bound custody-transfer source proof.

В command запрещены plaintext body, subject, recipients, phone/email/chat IDs,
provider SDK types, filesystem paths, credentials, provider selector,
`google.protobuf.Any` и generic maps. Integration разрешает opaque cursors
только через собственную operational projection.

Succeeded result содержит только intent identity, logical owner и bounded
opaque provider operation receipt. Rejected result содержит intent identity,
logical owner и закрытый provider-owned rejection code. Private provider
payload и свободный error text в results запрещены.

## Blob custody

До публикации command workflow materializes body в своей Blob custody и
получает proof, связанный с точной target fence выбранной integration:

| Owner | Module | Blob capability | Custody scope |
|---|---|---|---|
| `mail` | `makosh-mail-runtime` | `mail.blob.v1` | `mail.delivery-intent-body.v1` |
| `telegram` | `makosh-telegram-runtime` | `telegram.blob.v1` | `telegram.delivery-intent-body.v1` |
| `whatsapp` | `makosh-whatsapp-runtime` | `whatsapp.blob.v1` | `whatsapp.delivery-intent-body.v1` |
| `zulip` | `makosh-zulip-runtime` | `zulip.blob.v1` | `zulip.delivery-intent-body.v1` |

Contract unit владеет этими target constants. Свободный recipient в client
payload отсутствует. Integration consumer связывает proof с exact consumed
durable envelope, запрашивает одноразовый Blob custody transfer в собственную
fence и читает только полученный integration-owned target receipt.

Workflow не получает integration Blob credential. Integration не получает
workflow Storage/Blob credential. Kernel/Core проверяет capability, runtime и
grant fences и маршрутизирует exact bytes, но не декодирует body, не выбирает
provider и не становится delivery facade.

## Result flow и идемпотентность

Command partition/correlation identity основана на `intent_id`.
Integration-owned inbox подавляет duplicate exact envelope до provider
mutation. Provider ambiguity после timeout является typed rejection/ambiguous
state, а не автоматическим повтором без operation evidence.

Terminal result causally ссылается на consumed command message. Workflow
consume adapter меняет только owner-local delivery-intent state. Provider
acceptance не создаёт Communications evidence: подтверждённое outbound
observation по-прежнему приходит отдельным integration-owned ingress event.

## Build-unit и dependency rules

- contract unit имеет `role = "integration"`, точного provider owner и
  `surface = "contract"`;
- contract unit зависит только от generic Events/Runtime protocols и wire
  libraries;
- contract unit не зависит от Communications, workflow, другой integration,
  provider SDK, SQL, NATS implementation или runtime;
- provider runtime может импортировать только contract unit своего owner;
- workflow adapter может импортировать exact public contract units, но не
  integration implementation;
- assembly и admission выполняются отдельным последующим slice; contract unit
  не является runtime или integration assembly.

## Completion gate

Этот ADR закрывает только wire/route contract slice. Для перевода
`communication_delivery_intent_v1` в `implemented` дополнительно обязательны:

1. workflow-owned Blob capability и target-bound source proof creation;
2. четыре exact publisher adapters без generic provider facade;
3. четыре provider runtime consumers с inbox idempotency и custody transfer;
4. provider command adaptation внутри соответствующей integration;
5. succeeded/rejected result consumption и owner-local workflow transition;
6. duplicate, replay, stale grant/runtime, Blob/Vault outage и provider
   ambiguity tests;
7. descriptor/assembly/live managed proof и privacy-negative evidence;
8. client command/status/realtime closure из ADR-0330.
