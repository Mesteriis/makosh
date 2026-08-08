# ADR-0346: Cross-channel communication forward workflow

Статус: Принято

Дата: 2026-07-30

Состояние реализации: exact public contract, pure core и owner-local
persistence реализованы отдельными build units. Persistence предоставляет
owner-scoped idempotent admission, conflicting replay fence, monotonic state,
lease/epoch work claims, bounded reconnect-safe retry, client realtime ledger
и durable Blob custody cleanup queue; plaintext body не хранится. Disposable
PostgreSQL contour доказывает migration, reconnect, stale claim, retry deadline,
owner isolation и cleanup completion. Event-backed Communications
source-preparation contract реализован отдельной domain-owned build unit.
Communications persistence реализует owner-local source/target snapshot,
channel-kind validation, revision fence и atomic inbox/result-outbox commit.
Communications runtime adapter реализует exact command decode, verified Blob
custody transfer/read, fixed target-bound Blob write, typed result,
ACK-after-commit и exact managed admission wiring. Workflow persistence реализует
owner-local event inbox/hash fence, exact source-command/delivery-submit
outbox, source rejection и atomic prepared-result → dispatching transition.
Отдельная workflow-owned runtime build unit реализует managed admission,
generation/grant/storage fences, exact source-command producer, source
prepared/rejected consumers, verified Blob read, fixed delivery-intent Blob
write, transactional submit handoff, exact outbox relay и ACK-after-commit.
Delivery-intent consumer, downstream terminal-result consumer и обе durable
Blob custody cleanup очереди реализованы. Workflow runtime предоставляет
method-exact public command/query port и публикует owner-local transition
ledger через общий replayable client SSE. Отдельная deterministic assembly
build unit формирует signed release fragment без runtime behavior. Live managed
conformance доказывает Gateway command, event-only Communications handoff,
две Kernel-authorized Blob custody передачи, delivery-intent ingress, terminal
result, durable custody cleanup, client SSE и replay после runtime restart.
Live evidence закрывает exact gate `communication_cross_channel_forward_v1`;
общий `communications_settings_reconstruction_complete_v1` остаётся закрыт до
итогового полного clean-room parity audit.

## Контекст

Legacy Communications позволял переслать сообщение между каналами из общего
клиентского экрана. В clean-room это поведение нельзя вернуть методом
Communications, generic provider facade или импортом нескольких integrations:

- Communications владеет canonical evidence и private source content, но не
  provider execution;
- Mail, Telegram, WhatsApp и Zulip владеют своими operational commands и не
  импортируют Communications;
- `communication_delivery_intent` уже владеет provider-neutral доставкой в
  существующий canonical conversation;
- Kernel/Core маршрутизируют exact contracts, но не координируют business
  workflow и не декодируют payload.

Нужна отдельная функциональная единица, которая связывает source evidence с
новой доставкой, не превращаясь в facade над integrations.

## Решение

Вводится отдельный workflow owner `communication_cross_channel_forward` с
независимыми build units:

```text
makosh-communication-cross-channel-forward-api
makosh-communication-cross-channel-forward-core
makosh-communication-cross-channel-forward-persistence
makosh-communication-cross-channel-forward-runtime
makosh-communication-cross-channel-forward-assembly
```

Public V1 command принимает только:

- stable 16-byte `forward_operation_id`;
- canonical `source_message_id`;
- canonical `target_conversation_id`;
- optional canonical `target_reply_to_message_id`.

Caller не передаёт provider, account, provider chat locator или plaintext body.
Attachments, arbitrary recipients, native same-provider forwarding и editable
forward text не входят в V1.

Runtime сначала сохраняет accepted operation в owner-local persistence. Worker
публикует exact durable Communications command
`cross_channel_forward_source_prepare.v1` по ADR-0347. Communications
проверяет current owner, active source message, target conversation и отличие
source/target channel, затем создаёт idempotent target-bound Blob delegation
для consumer owner `communication_cross_channel_forward`. Durable result содержит только
canonical identities, source revision, bounded content metadata, Blob
reference и custody proof; body не проходит через Core RPC.

Workflow materializes source Blob через собственный scoped capability,
проверяет receipt/hash/size, создаёт новый Blob с compile-time
delivery-intent target binding и публикует exact durable
`communication_delivery_intent_submit.v1` command по ADR-0348. Plaintext не
попадает в event. Delivery-intent самостоятельно разрешает target provider
provenance через Communications и вызывает ровно одну integration.
Cross-channel workflow не импортирует provider contracts, SDK, runtime или
persistence.

Workflow хранит:

- canonical source/target identities;
- source evidence revision и content hash;
- source-preparation receipt;
- downstream delivery-intent ID;
- monotonic workflow state и retry metadata.

Workflow не хранит plaintext body. После downstream terminal acceptance или
rejection он durable-release target-bound Blob custody. Provider completion
остаётся состоянием delivery-intent/provider evidence и не подменяется
workflow receipt.

## Состояния и идемпотентность

Owner-scoped key `(logical_owner_id, forward_operation_id)` возвращает тот же
workflow receipt для эквивалентного retry и reject-ит conflicting replay.

Допустимые состояния:

```text
accepted
→ preparing_source
→ dispatching
→ delivery_accepted | rejected
```

Transient Communications, Blob или delivery-intent outage сохраняет durable
retry с bounded exponential backoff. Runtime restart не меняет operation ID,
source-preparation ID или downstream delivery operation ID. Terminal state
не переоткрывается.

## Согласование с Kernel/Core

Kernel:

- регистрирует workflow как отдельный `Workflow` owner;
- выдаёт exact grants для ClientRpc, ClientRealtime, Communications
  command/result routes, Blob read/release, Storage и delivery-intent request;
- проверяет current runtime/grant/storage generations;
- не декодирует source metadata, content или delivery payload.

Core capability router:

- маршрутизирует только exact generated contracts;
- не выбирает integration и не содержит cross-channel business method;
- не retry-ит mutation после ambiguous delivery;
- не выдаёт internal `DurableEnvelopeV1` клиенту.

Gateway возвращает immediate workflow receipt. `accepted` означает только
durable admission. Status query и общий replayable SSE несут client-safe
identity/state/error code без body, Blob reference, provider locator или
credential material.

## Границы SRP и сборки

- API владеет только generated public command/query/realtime schema.
- Core валидирует provider-neutral identities и pure state transitions.
- Persistence владеет operation, replay и durable retry tables.
- Runtime координирует exact public ports.
- Assembly создаёт descriptor/settings/storage/release artifacts и не
  исполняет workflow.
- Communications расширяется отдельным event-backed source-preparation
  contract и adapter, но не импортирует workflow implementation.
- Delivery-intent остаётся независимым workflow и не знает о forwarding.
- Delivery-intent event ingress является отдельной contract build unit и не
  смешивается с client ConnectRPC API.

Integration не является domain. Domain не является integration. Ни одна из
этих единиц не импортирует другую business implementation.

## Ошибки и privacy

Публичные error codes ограничены:

- invalid request;
- source not found or inactive;
- target not found;
- same-channel target;
- source content unavailable;
- delivery rejected;
- unavailable.

Private body, provider locators, Blob references, custody proofs и credentials
не попадают в logs/events/errors/status/health. Durable events содержат только
owner-scoped identity, state, causation, correlation и timestamps.

## Completion gate

`communication_cross_channel_forward_v1` является `implemented`, поскольку
выполнены все условия:

1. exact public command/query/realtime contract и pure core;
2. owner-local idempotent persistence, retry и replay;
3. exact event-backed Communications source-preparation contract с
   target-bound Blob delegation;
4. managed runtime с Storage/Vault/Blob/Communications/delivery-intent ports,
   generation fencing и revoke;
5. method-exact Gateway command/status и shared SSE replay;
6. duplicate, conflicting replay, restart, ambiguous response, dependency
   outage и custody cleanup tests;
7. live managed proof от source evidence до accepted delivery-intent без
   provider imports, content leakage или cross-owner SQL.

Reconstruction matrix фиксирует gate в состоянии `implemented`; это не
подменяет оставшиеся независимые Communications/Settings gates.
