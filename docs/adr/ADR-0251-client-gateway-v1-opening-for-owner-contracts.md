# ADR-0251: Opening `client_gateway_v1` for owner-owned contracts

Статус: Принято
Дата: 2026-07-23
Состояние реализации: `client_gateway_v1` открыт. Owner-neutral Gateway
transport, owner-device session fencing, bounded ConnectRPC routing,
deadline/error mapping, replayable SSE с explicit gap/revoke semantics,
HTTP/2+TLS и HTTP/3 transport реализованы. Открытый platform gate не означает
admission конкретного owner: без approved owner route и realtime publisher
business RPC возвращает unavailable/not found, а SSE остаётся fail-closed.

Зависит от:

- [ADR-0205: Core Gateway](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0225: phase gates](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md);
- [ADR-0232: browser client identity](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0250: Communications NATS profile](ADR-0250-communications-nats-data-plane-v1-admission-profile.md).

## Решение

Core Gateway остаётся transport/capability router и не становится generic
business API. Public business services принадлежат owner contract packages.
Gateway:

- аутентифицирует owner device/session;
- выбирает approved registration по exact capability и grant epoch;
- маршрутизирует opaque typed payload через managed module client protocol;
- применяет transport deadlines, size limits и sanitized typed errors;
- не декодирует business semantics и не импортирует owner Cargo package.

Для Communications public query path принадлежит
`makosh-communications-api`:

```text
/makosh.communications.query.v1.CommunicationsQueryService/Query
```

Gateway wrapper service, provider endpoint, runtime command facade, handwritten
REST business API и fallback route запрещены.

Client transports:

- ConnectRPC/Protobuf для query/request/command;
- один replayable SSE stream на active client process;
- HTTP только для health/readiness, OAuth, Blob и SSE;
- paired remote baseline HTTP/2+TLS, HTTP/3 только после conformance;
- 0-RTT и raw QUIC запрещены.

Durable command возвращает receipt, а не provider completion. Terminal result
приходит через replay/status contract.

## Exact package inventory

```text
makosh-gateway-protocol
makosh-gateway-session-contract
makosh-gateway-session
makosh-gateway-runtime
```

Kernel композирует эти adapters, но Gateway packages не зависят от Kernel,
Communications или integration implementations.

## Реализованный transition

1. Owner contract payload маршрутизируется opaque через approved capability.
2. Connect request ограничен по размеру и deadline, ошибки sanitized.
3. SSE хранит bounded per-owner replay, не смешивает owners, явно сообщает gap
   и завершает stream при revoke.
4. Local, paired HTTP/2 и HTTP/3 используют один session authorization policy.
5. `client_gateway_v1` удалён из `phaseGates.notAuthorized`.

Generated client adapters и доступность конкретных business routes являются
частью admission соответствующего owner, а не условием существования Gateway.

## Gate evidence

- owner-device authorization and revoke fencing;
- wrong owner/capability/grant epoch fail closed;
- ConnectRPC deadline/error/receipt conformance;
- SSE replay, gap, reset, disconnect and suspension recovery;
- HTTP surface separation;
- remote HTTP/2+TLS and HTTP/3 fallback with disabled 0-RTT;
- abuse, replay, privacy, redaction and payload-limit tests;
- architecture guard против owner package dependencies и Gateway facades.

## Rollback

Gate возвращается в `notAuthorized`; owner routes не монтируются, sessions и
technical recovery surface продолжают работать. Owner runtimes и durable
events не удаляются.
