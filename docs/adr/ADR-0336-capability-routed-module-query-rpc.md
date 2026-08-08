# ADR-0336: Capability-routed module query RPC

Статус: Принято

Дата: 2026-07-29

Состояние реализации: private versioned request/delivery/response wire,
structural bounds, Control Store schema v45 provider/dependency catalog,
descriptor extraction и owner-neutral Kernel authorization/provider relay
реализованы. Managed conformance покрывает success, zero/ambiguous provider,
stale caller/provider binding, revoke и response mismatch. Platform gate
`capability_routed_module_query_rpc_v1` реализован; первый delivery-intent
adapter остаётся отдельным незакрытым gate. Communications runtime объявляет
exact `communications.query` как отдельный `query_rpc` surface и обрабатывает
provider delivery через отдельный module port; browser `client_rpc` при этом
остаётся независимым transport surface.

Уточняет:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0330: Provider-neutral communication delivery intent workflow](ADR-0330-provider-neutral-communication-delivery-intent-workflow.md).

## Контекст

`communication_delivery_intent` принимает canonical `conversation_id`, но не
имеет права получать provider/account locator от клиента. Route принадлежит
Communications canonical evidence и должен разрешаться через его public typed
contract. Прямой импорт Communications runtime/persistence, общий SQL, module
socket, provider selector в client request или перенос plaintext body через
event spine запрещены.

`ModuleDescriptorV1` уже различает `query_rpc`, `request_rpc` и `client_rpc`,
но clean-room runtime transport реализует только последнее. Использовать
`client_rpc` как скрытый module-to-module channel нельзя: browser principal и
managed runtime имеют разные authority и failure model.

Durable events остаются правильной границей для mutations, observations и
replayable business results. Однако bounded current-state lookup без mutation
не должен искусственно создавать durable command, outbox и terminal job только
ради чтения одного owner contract.

## Решение

### Отдельный private transport

Managed runtime может инициировать только typed `query_rpc` к exact dependency,
объявленной в capability descriptor:

```text
caller managed runtime
  -> inherited private Kernel control FD
  -> ModuleQueryRequestV1
  -> Kernel capability/dependency router
  -> exact target managed runtime
  -> ModuleQueryResponseV1
  -> caller
```

`ModuleQueryRequestV1` содержит:

- non-zero random `request_id`;
- exact `ContractReferenceV1`;
- bounded Protobuf request bytes;
- deadline не больше Kernel hard ceiling.

Caller не передаёт target registration, process address, capability ID,
provider identity или grant epoch. Kernel сам разрешает ровно одну current
approved implementation exact contract.

`request_rpc` и mutation в этот gate не входят.

### Kernel authorization

Kernel разрешает query только если одновременно:

1. caller registration approved и exact runtime instance/generation/grant epoch
   current;
2. один из effective caller capabilities содержит exact contract в
   `dependencies`;
3. target descriptor предоставляет exact contract как `query_rpc`;
4. target capability granted, target runtime current и managed;
5. logical owner один и тот же;
6. request и response укладываются в hard byte/deadline limits.

Zero или несколько current providers fail closed. Kernel не декодирует query
payload/response, не выбирает business behavior и не сохраняет payload.
Correlation существует только на private bounded control exchange и не
является durable receipt.

Revoke/restart любого участника инвалидирует следующий route. In-flight
response принимается только если caller и target fences всё ещё current.
Automatic fallback на другую revision, client route или legacy REST запрещён.

### Descriptor и Control Store

`query_rpc` provider surface получает отдельный exact private route identity.
Control Store сохраняет:

- provider registration/capability;
- exact contract reference;
- private route path;
- caller capability dependency как exact contract reference.

Эти records являются boot/control metadata, не business data. Approval
capability не означает approval всех dependencies: effective dependency route
строится заново как пересечение descriptor, grants, current runtimes и hard
Kernel policy.

`client_rpc` и `query_rpc` могут использовать один Protobuf message/service
method только когда descriptor явно объявляет две разные surfaces. Наличие
client route само по себе не открывает module query.

### Первый consumer

Первым consumer становится `communication_delivery_intent`:

1. client Submit доставляется Core Gateway в workflow как existing generated
   `client_rpc`;
2. workflow вызывает Communications exact canonical query dependency;
3. Communications возвращает provider-neutral conversation/message summaries
   с opaque provider provenance cursors;
4. workflow pure core проверяет conversation/reply и строит
   `PlannedDeliveryIntentV1`;
5. body записывается через existing target-bound Blob path, а persistence
   атомарно сохраняет intent/outbox;
6. Submit возвращает `accepted`; terminal provider result остаётся event-only.

Workflow импортирует только `makosh-communications-api`, а не Communications
runtime, persistence или integration packages. Kernel/Gateway не импортируют
оба owner API.

### Realtime не подменяется query RPC

Query RPC закрывает Submit route resolution и GetStatus. Replayable terminal
client realtime остаётся отдельной частью completion gate ADR-0330. Polling
status не считается доказательством realtime, а принятие этого ADR не переводит
`communication_delivery_intent_v1` в `implemented`.

## Units и SRP

```text
runtime protocol
  private typed wire and validation

Kernel capability router
  dependency/provider resolution and fence authorization

owner runtime query port
  owner contract decode, application call and response encode

workflow query adapter
  exact public contract request/response mapping

Gateway
  unchanged opaque client transport
```

Количество строк не определяет unit. Protocol package не знает owners, Kernel
не знает Communications semantics, Communications не знает workflow, а
workflow не знает provider integrations.

## Phase gates

### `capability_routed_module_query_rpc_v1`

1. exact versioned private query request/response;
2. descriptor-declared provider surface и caller dependency;
3. Control Store persistence and current approved resolution;
4. same-owner, grant, runtime-generation and grant-epoch fencing;
5. bounded payload, deadline, correlation and sanitized errors;
6. zero/ambiguous provider, stale/revoke/restart and response mismatch tests;
7. no owner-specific dependency in Kernel/Core packages;
8. architecture, SRP, Cargo, Clippy and managed conformance.

### Delivery-intent client closure

После platform gate отдельно требуются:

1. Communications exact `query_rpc` provider;
2. delivery-intent dependency and adapter;
3. generated Submit/GetStatus client delivery;
4. invalid/missing/cross-owner/reply mismatch negatives;
5. live managed Gateway proof without private content leakage;
6. replayable terminal client realtime.

До выполнения всех шести пунктов ADR-0330 и reconstruction inventory остаются
`planned`.

## Отклонённые варианты

### Provider selector в Submit

Отклонено: client начал бы создавать integration routing truth.

### Workflow import Communications runtime или SQL

Отклонено: domain и workflow перестали бы быть independently restartable build
units.

### Использовать `client_rpc` для managed caller

Отклонено: смешивает browser authority с module dependency authority.

### Durable event для каждого current-state lookup

Отклонено для этого exact read: создаёт ложную durable mutation semantics.
Durable commands/events сохраняются для mutations и terminal provider results.
