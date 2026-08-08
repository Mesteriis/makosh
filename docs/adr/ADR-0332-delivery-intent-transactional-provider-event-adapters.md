# ADR-0332: Delivery-intent transactional provider event adapters

Статус: Принято

Дата: 2026-07-29

Состояние реализации: четыре exact workflow-owned command encoder/result
decoder adapter, transactional provider-command outbox и idempotent terminal
result inbox реализованы. Runtime entry points сохраняют exact durable command
bytes под current claim fence, а publish completion атомарно переводит intent
в `submitted_to_provider`. Provider-owned terminal result применяется вместе с
inbox identity/hash. Blob materialization и четыре provider-owned
consume/execute/result loops реализованы по ADR-0333 и ADR-0335. Workflow
runtime пока не получает JetStream publish/result-consume permits и не входит
в live managed admission; Gateway client closure также остаётся следующим
слайсом.

## Контекст

ADR-0331 зафиксировал четыре provider-owned wire contract, но одного
Protobuf descriptor недостаточно для выполнения intent. Workflow обязан
сохранить exact command envelope в собственном outbox атомарно с изменением
owner-local state. Обратный terminal result должен применяться атомарно с
integration-envelope inbox record, иначе crash между mutation и Ack допускает
повторное или противоречивое завершение.

Общий provider command facade запрещён. При этом технические durable-envelope
правила, owner-local outbox и inbox остаются ответственностью workflow owner,
а не Mail, Telegram, WhatsApp, Zulip, Communications или Kernel.

## Решение

Вводится отдельная workflow build unit:

```text
makosh-communication-delivery-intent-event-adapters
```

Она имеет `role = "workflow"`, owner
`communication_delivery_intent`, surface `implementation` и содержит четыре
независимых модуля:

```text
mail
telegram
whatsapp
zulip
```

Каждый модуль импортирует только public contract своей integration и
предоставляет:

- exact execute-command envelope builder;
- exact succeeded-result decoder;
- exact rejected-result decoder.

Нет provider discriminator dispatch, общего provider payload, provider SDK,
integration persistence/runtime dependency или generic `execute(any)`.
Provider-neutral типы допустимы только после exact route-specific decode как
workflow state transition input.

## Transactional outbox

Claimed workflow intent сохраняет exact `DurableEnvelopeV1` bytes, message ID
и SHA-256 в PostgreSQL outbox только после проверки current claim fence.
До publish он остаётся в `resolving_route`. После exact JetStream publish
receipt одна transaction помечает outbox published и переводит intent в
`submitted_to_provider`. Поэтому crash до commit приводит только к повторной
публикации тех же exact bytes. До terminal result:

- encrypted body custody остаётся owner-local;
- submission receipt содержит canonical command message ID и заменяется
  provider operation receipt только successful terminal result;
- outbox relay публикует сохранённые bytes без re-encode;
- повторный publish безопасен по canonical message ID.

Outbox rows partitioned по provider provenance только для выбора exact publish
permit. Persistence не декодирует integration payload и не выбирает provider
behavior.

## Idempotent result inbox

Каждый exact result decoder проверяет:

- canonical envelope validation;
- exact owner/name/major/revision/schema hash;
- `result` semantics и expected outcome;
- command ID, command message ID и causation binding;
- payload identity и provider-owned validation.

Workflow persistence в одной transaction:

1. классифицирует inbox message ID/hash;
2. exact duplicate возвращает existing terminal status без mutation;
3. hash conflict отклоняет fail-closed;
4. succeeded записывает bounded provider operation receipt;
5. rejected записывает closed rejection code;
6. очищает encrypted body custody;
7. сохраняет terminal transition и inbox record.

JetStream Ack разрешён только после успешного commit либо exact duplicate.

## Build-unit и ownership rules

- event adapters являются workflow implementation, не integration, domain,
  runtime, persistence или assembly;
- integration contracts остаются четырьмя независимыми integration-owned
  units;
- workflow persistence зависит от generic Events contract, но не от provider
  contracts;
- workflow runtime импортирует event-adapter build unit, но не provider
  implementation;
- Communications domain не участвует в provider command/result path;
- Kernel/Core не декодирует payload и не становится outbox, inbox или provider
  facade.

## Следующий gate

Для executable provider flow ещё обязательны:

1. workflow-owned Blob write/materialization с exact target-bound custody proof;
2. четыре exact JetStream publish permits и relay loops;
3. четыре provider runtime inbox consumers;
4. provider-owned custody transfer и operational command adaptation;
5. provider result outbox и workflow result pull consumers;
6. outage/replay/stale generation/live managed evidence;
7. Gateway client command/status/realtime closure.

До закрытия этих пунктов `communication_delivery_intent_v1` остаётся
`planned`.
