# ADR-0335: Provider-owned delivery-intent consumers and route locators

Статус: Принято

Дата: 2026-07-29

Состояние реализации: provider-owned locator projections, durable inbox/job
lifecycle, Blob custody workers, operational adapters, terminal-result outbox
и exact relay loops реализованы отдельно для Mail, Telegram, WhatsApp и Zulip.
Exact contract/admission, duplicate decoding, custody integrity, generation
fencing и provider-neutral result tests проходят. Completion gate остаётся
открытым до полного outage/replay/ambiguity conformance и live managed
validation всех четырёх integrations; workflow Event Hub и client closure
принадлежат отдельным gates ADR-0330/ADR-0332. До их закрытия
`communication_delivery_intent_v1` остаётся `planned`.

## Контекст

ADR-0331 определяет четыре независимых integration-owned execute/result
контракта. Execute payload намеренно не содержит provider account, recipient,
chat, stream, topic, email address или SDK type. Вместо них он несёт только
opaque 32-byte source cursors из canonical Communications evidence.

Эти cursors являются односторонними SHA-256 identifiers. Provider runtime не
может восстановить из cursor исходный operational locator. Текущие Mail,
Telegram, WhatsApp и Zulip stores хранят provider operational state, но не
имеют полной reverse projection:

```text
account source cursor -> provider account locator
conversation source cursor -> provider destination locator
reply source cursor -> provider message locator
```

Сканирование всех provider rows и повторное вычисление hashes на каждый command
не является bounded operational contract. Передача raw locator в workflow или
Communications нарушила бы provider boundary ADR-0204 и ADR-0331.

Кроме route resolution provider consumer обязан согласовать три разных
lifecycle:

1. durable command inbox и Ack;
2. target-bound Blob custody transfer;
3. provider operational completion с terminal result.

Нельзя считать enqueue provider command успешной доставкой. Это создало бы
ложный terminal result при crash, provider rejection или ambiguous timeout.

## Решение

Каждая integration реализует собственный consumer внутри своей runtime и
persistence units:

```text
Mail      -> makosh-mail-runtime / makosh-mail-persistence
Telegram  -> makosh-telegram-runtime / makosh-telegram-persistence
WhatsApp  -> makosh-whatsapp-runtime / makosh-whatsapp-persistence
Zulip     -> makosh-zulip-runtime / makosh-zulip-persistence
```

Общего provider consumer, enum dispatch, runtime facade, persistence crate или
generic route table не вводится. Общими остаются только platform Events/Blob
protocols и уже принятые Communications ingress cursor derivation rules.

Каждая provider persistence unit владеет тремя отдельными responsibilities:

- operational route locator projection;
- delivery-intent inbox/job lifecycle;
- delivery-intent terminal-result outbox.

Это отдельные модули и schema tables внутри integration-owned build unit. Они
не становятся новым business domain.

## Provider route locator

Locator projection записывается атомарно с provider operational projection и
observation outbox, из которых был получен canonical source cursor. Она хранит:

- exact account source cursor;
- optional conversation source cursor;
- optional source-message cursor;
- provider-owned bounded operational locator;
- provider projection revision и lifecycle state.

Cursor derivation использует exact public functions из
`makosh-communications-ingress`; provider не импортирует Communications domain,
runtime или persistence. Collision одного cursor с другим locator отклоняется
fail-closed. Removed/unlinked account не разрешается как active route.

Raw locator никогда не возвращается workflow, Communications, Kernel, Gateway
или client.

## Inbox и job lifecycle

Provider consumer принимает только свой exact execute contract и проверяет:

- canonical `DurableEnvelopeV1`;
- exact contract owner/name/major/revision/schema hash;
- `command` semantics и expected audience;
- message, command, correlation и partition identities;
- payload validation своего contract;
- payload logical owner против current runtime admission;
- current runtime generation, grant epoch и consume permit.

В одной transaction integration:

1. вставляет inbox message ID и envelope SHA-256;
2. exact duplicate возвращает existing state;
3. hash collision отклоняет command;
4. разрешает opaque cursors только через собственный active locator;
5. создаёт provider-owned pending delivery job.

Ack разрешён после commit либо exact duplicate. Blob transfer и provider I/O
не выполняются внутри inbox transaction.

## Blob custody worker

Claimed job связывает custody transfer с:

- consumed command message ID и envelope SHA-256;
- exact source reference ID, declared bytes и SHA-256;
- provider owner/module/Blob capability constants своего contract;
- current runtime registration, generation и grant epoch.

Worker запрашивает target-bound custody transfer, читает полный body, проверяет
declared bytes, SHA-256 и UTF-8, затем создаёт provider operational command.
Provider store сохраняет operation binding к intent/job до внешнего I/O.

Повтор exact job использует тот же provider operation ID. Custody proof,
integrity, stale fence и route failure переводятся в закрытый typed rejection,
не в свободный error text.

## Provider execution и results

Существующие provider operational seams остаются authority выполнения:

- Mail delivery queue и SMTP/Gmail dispatcher;
- Telegram operation queue и TDLib send;
- WhatsApp host-only hidden WebView command queue;
- Zulip command operation queue и HTTP executor.

WhatsApp consumer не получает сетевой SDK executor: он только ставит команду в
существующую host-only очередь.

После реального terminal operational outcome provider persistence атомарно:

1. завершает delivery-intent job;
2. создаёт exact succeeded либо rejected `DurableEnvelopeV1`;
3. сохраняет exact bytes/hash в provider-owned result outbox.

`succeeded` содержит opaque provider operation receipt. Accepted/enqueued,
retryable outage и outcome-unknown не являются success. Ambiguous provider
outcome маппится в `PROVIDER_AMBIGUOUS`; safe retry сохраняет тот же operation
binding.

Outbox relay публикует сохранённые bytes без re-encode. Published marker не
меняет provider terminal truth.

## Build-unit и dependency rules

- provider runtime импортирует только contract unit своего owner;
- provider persistence не импортирует provider contract и хранит opaque exact
  envelope bytes;
- provider contract не импортирует runtime/persistence/SDK;
- `makosh-communications-ingress` остаётся единственным разрешённым public
  contract для source cursor derivation;
- Communications domain, workflow store и provider stores не импортируют друг
  друга;
- assembly отдельно выдаёт exact consume/publish/Blob capabilities;
- Kernel/Core маршрутизирует exact bytes и fences, но не декодирует provider
  payload и не разрешает routes.

## Failure и recovery

- invalid envelope/payload/audience: no inbox mutation, no Ack;
- exact duplicate: no repeated provider mutation, Ack after state lookup;
- inbox hash collision: fail-closed, no Ack;
- missing/inactive locator: terminal `ROUTE_NOT_FOUND`;
- invalid custody proof/hash/UTF-8: terminal `CUSTODY_REJECTED`;
- Blob/control/provider outage: bounded retry with the same job/operation;
- stale runtime/grant fence: stop processing without terminal fabrication;
- provider rejection: terminal `PROVIDER_REJECTED`;
- ambiguous provider result: terminal `PROVIDER_AMBIGUOUS`;
- crash before commit: redelivery;
- crash after commit before Ack/publish: exact duplicate or exact outbox replay.

## Completion gate

Решение считается реализованным только когда для всех четырёх integrations
есть:

1. provider-owned locator persistence and collision tests;
2. exact inbox validation, duplicate/hash-conflict tests and Ack discipline;
3. current-fence custody transfer plus size/hash/UTF-8 validation;
4. adaptation to the real provider operational queue;
5. terminal succeeded/rejected outbox coupled to provider completion;
6. exact consume/publish/Blob descriptor permits and assembly admission;
7. outage, replay, stale-generation and ambiguity conformance;
8. live managed runtime evidence without private payload in logs/health.
