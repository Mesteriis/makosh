# ADR-0350: Explicit human owner context for managed domain and integration runtimes

Статус: Принято

Дата: 2026-07-30

Состояние реализации: реализовано для managed domain/integration launch
protocol, Kernel staging, Telegram call evidence producer, Communications
consumer/query и client realtime. Подтверждено live managed conformance.

Уточняет:

- [ADR-0204: bundled integration plugins](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0215: module registration and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0224: owner-scoped PostgreSQL](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0349: Communications call evidence](ADR-0349-event-backed-communications-call-evidence.md).

## Контекст

Managed domain и integration конфигурации уже содержали `logical_owner_id`.
Это identity владельца build unit: `communications`, `telegram`, `mail` и так
далее. Kernel использует его для registration, grants, Event Hub и
owner-scoped storage. Он не является identity человека, которому принадлежат
provider accounts и private data.

Использование module owner как human owner ломает client authorization и
cross-owner isolation. Обратная замена human owner в storage/Event Hub
контексте дала бы integration права владельца-пользователя и смешала бы две
независимые authority.

## Решение

Managed domain и integration launch contracts получают отдельное обязательное
поле `logical_human_owner_id`.

```text
logical_owner_id
    = build-unit owner
    = registration / grants / storage / Event Hub authority

logical_human_owner_id
    = authenticated human owner
    = provider tenancy / domain tenancy / client realtime authority
```

Kernel формирует human owner только из уже авторизованного owner session:

- integration launch использует initial owner identity;
- domain launch использует owner, авторизовавшего exact launch request.

Runtime не может подменить это значение через settings, provider payload или
business event. Validation требует bounded canonical owner identity.

## Границы ответственности

- runtime protocol владеет wire field и structural validation;
- Kernel владеет staging authenticated human owner в launch configuration;
- integration mapper помещает human owner в exact public observation;
- domain consumer проверяет, что event owner совпадает с admitted human owner;
- module owner продолжает владеть Event Hub permit, storage binding и grants;
- Gateway realtime publish использует human owner и не принимает module owner
  как замену;
- Kernel, Gateway и Event Hub не декодируют business payload.

Domain не импортирует integration implementation. Integration не импортирует
domain core или persistence. Передача owner context происходит только через
runtime launch contract и typed event contract.

## Build units

Решение не создаёт общий owner service или business facade. Изменения
распределены по существующим единицам:

- `makosh-runtime-protocol` — transport contract;
- Kernel launch adapters — authenticated staging;
- provider integration runtime/mapper — source observation;
- Communications runtime consumer — tenancy validation;
- owner-specific client realtime adapter — delivery authorization.

## Failure semantics

- отсутствующий или malformed human owner отклоняет managed launch;
- event с другим human owner не сохраняется и не ACK-ится как canonical state;
- stale runtime/grant/storage fences проверяются независимо;
- module owner никогда не используется как fallback human owner;
- human owner никогда не расширяет module GrantSet.

## Evidence

`managed_call_evidence_survives_nats_outage_and_replays_through_gateway_sse`
доказывает distinct Telegram module owner и human owner, durable event delivery
после NATS outage, Communications tenancy check, shared SSE authorization и
replay после рестарта domain runtime.
