# ADR-0339: Capability-routed module request RPC

Статус: Принято

Дата: 2026-07-29

Состояние реализации: реализовано полностью. Private versioned
request/delivery/response wire, hard bounds, managed-runtime control delivery,
separate descriptor/Control Store provider inventory, Kernel
authorization/opaque relay и delivery-intent provider request port прошли
unit, architecture и live managed-process conformance. Platform gate
`capability_routed_module_request_rpc_v1` открыт.

Уточняет:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0330](ADR-0330-provider-neutral-communication-delivery-intent-workflow.md);
- [ADR-0336](ADR-0336-capability-routed-module-query-rpc.md).

## Контекст

`communication_bulk_action` должен выполнять bounded fan-out через публичный
contract `communication_delivery_intent`, получать отдельный acceptance receipt
для каждой цели и сохранять собственное состояние orchestration.

Текущий managed runtime transport реализует:

- `query_rpc` из runtime в runtime;
- `client_rpc` из authenticated client через Gateway в runtime;
- durable events/commands через outbox, NATS и inbox.

Использовать `client_rpc` из managed workflow нельзя: browser principal,
session authorization и client delivery не являются module authority. Прямой
импорт delivery-intent runtime/core/persistence, общий SQL или module socket
нарушают restart/build/storage boundary.

Один durable command подходит для асинхронной mutation без немедленного typed
result. Но bulk orchestration требует получить bounded acceptance receipt
каждой идемпотентной Submit-операции, чтобы записать точное соответствие:

```text
bulk target operation_id -> delivery intent receipt
```

`accepted` не означает provider completion. Terminal delivery result остаётся
replayable owner event/status query.

## Решение

### Отдельный interaction kind

Core реализует descriptor-declared `request_rpc`:

```text
caller managed workflow
  -> inherited private Kernel control FD
  -> ModuleRequestRequestV1
  -> Kernel capability router
  -> exact current request_rpc provider
  -> ModuleRequestResponseV1
  -> caller
```

Это не расширение `query_rpc` флагом. Query и request имеют отдельные:

- private wire messages и validation;
- descriptor provider inventories;
- Control Store route records;
- Kernel handlers;
- runtime delivery branches;
- conformance tests.

### Семантика request

`request_rpc` допускает typed операцию, которая:

1. валидирует request;
2. идемпотентно принимает mutation/application request;
3. при необходимости durably сохраняет owner state/outbox до ответа;
4. возвращает immediate typed result или acceptance receipt.

Kernel не повторяет request автоматически. Caller обязан передать
contract-defined stable operation/idempotency ID. Timeout или потерянный
response имеют ambiguous delivery semantics; caller повторяет только тот же
exact idempotent request.

Long-running completion, provider result и progress не удерживают RPC. Они
приходят через owner query/replayable event.

### Private wire

Runtime protocol вводит отдельные сообщения:

```text
ManagedRuntimeModuleRequestRequestV1
  request_id
  contract
  request_payload
  deadline_millis

ManagedRuntimeModuleRequestDeliveryV1
  request_id
  logical_owner_id
  contract
  request_payload

ManagedRuntimeModuleRequestResponseV1
  request_id
  response_payload
  error_code
```

Hard bounds совпадают с конституционными bounds module RPC:

- `request_id`: exact 16 non-zero bytes;
- payload/response: не более 64 KiB;
- deadline: `1..=30_000` ms;
- contract: exact owner/name/major/revision/schema hash;
- error: bounded sanitized code без private payload.

Target registration, process path, capability/grant/runtime identifiers в
caller request отсутствуют.

### Authorization и routing

Kernel разрешает route только если одновременно:

1. caller registration, runtime instance/generation и grant epoch current;
2. caller effective capability содержит exact contract dependency;
3. ровно один current approved provider объявляет exact `request_rpc`;
4. provider capability granted, runtime managed и current;
5. caller и provider принадлежат одному logical Макошь owner;
6. request/delivery/response проходят structural validation;
7. provider и caller fences повторно current после response.

Zero/ambiguous provider, query-only provider, stale/revoked caller или provider,
response mismatch и oversized payload fail closed.

Kernel не декодирует business payload, не сохраняет его, не выполняет retry и
не выбирает workflow behavior.

### Descriptor и Control Store

`ProvidedSurfaceKindV1::RequestRpc` создаёт отдельный exact provider record:

```text
registration_id
capability_id
contract owner/name/major/revision/schema_sha256
```

Contract dependency остаётся generic exact reference у caller capability, но
его наличие не определяет interaction kind. Route выбирается только по
request-specific provider inventory.

Один contract может быть доступен клиенту через `client_rpc` и модулям через
`request_rpc` только если provider descriptor явно объявил обе surfaces.
Client admission не выдаёт module request rights и наоборот.

### Первый consumer: bulk delivery

После platform gate:

1. delivery-intent runtime объявляет Submit contract как exact `request_rpc`;
2. `communication_bulk_action` объявляет этот contract dependency;
3. bulk request содержит bounded список `1..=100` целей и отдельные stable
   target operation IDs;
4. bulk persistence создаёт batch и target rows до fan-out;
5. runtime вызывает delivery-intent request отдельно для каждой pending цели;
6. каждый receipt/error сохраняется отдельно;
7. retry повторяет только unresolved target с тем же operation ID;
8. batch status не подменяет terminal provider delivery status.

Bulk workflow импортирует только public delivery-intent API. Он не импортирует
Communications domain или provider integrations и не хранит provider truth.

## Units и SRP

```text
runtime protocol
  request wire and structural validation

Control Store
  exact request provider inventory

Kernel capability router
  dependency/provider/fence authorization and opaque relay

provider runtime request port
  public contract decode and owner application call

caller workflow adapter
  exact generated request/response mapping and idempotent retry policy
```

Единица сборки определяется ответственностью и boundary, не количеством строк.
`request_rpc` не добавляется в query handler и не превращается в generic
`execute(bytes)` API.

## Phase gate `capability_routed_module_request_rpc_v1`

Gate реализован только при наличии:

1. versioned private request/delivery/response wire;
2. hard payload/deadline/correlation/error validation;
3. separate descriptor extraction и Control Store request provider inventory;
4. exact dependency/provider resolution;
5. logical-owner, grants, runtime-generation и grant-epoch fencing;
6. no Kernel retry и explicit ambiguous timeout semantics;
7. provider delivery branch и caller response correlation;
8. zero/ambiguous/query-only/stale/revoke/response mismatch negatives;
9. live managed conformance;
10. architecture/SRP/Cargo/Clippy/full test gates.

Принятый ADR сам по себе gate не открывает.

## Отклонённые варианты

### Использовать client_rpc

Смешивает browser session authority и managed module authority.

### Расширить query_rpc полем `mutating`

Размывает read-only semantics и позволяет тихо выполнить mutation через query
provider surface.

### Прямой вызов delivery-intent implementation

Нарушает independent build/runtime/storage units.

### Автоматический retry в Kernel

Kernel не знает business idempotency и может повторить mutation.

### Generic workflow execute API

Создаёт untyped mediator и скрывает exact use-case contracts.
