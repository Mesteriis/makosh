# ADR-0314: Core Gateway authenticated client Blob routing

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: implemented. Descriptor/Control Store authority,
  authenticated Gateway adapter, live Blob/Vault full-read contour и первый
  owner-specific route Communications Export реализованы. Loopback development
  assembly проксирует exact `/api/blobs/` prefix server-side тем же
  per-process proof boundary, не открывая generic frontend route.
- Связанные решения: ADR-0200, ADR-0205, ADR-0212, ADR-0213, ADR-0215,
  ADR-0221, ADR-0230, ADR-0231, ADR-0279

## Контекст

Core Gateway уже маршрутизирует descriptor-declared `client_rpc`, но
`client_blob` существует только как architecture interaction kind. Если
вернуть private content через `ModuleClientResponseV1`, content bytes пройдут
через managed control channel и Kernel relay. Если выдать browser внутренний
`BlobRefV1`, private Blob socket или `BlobDataSessionGrantV1`, client получит
module data-plane capability и сможет обойти owner authorization.

Нельзя закрыть разрыв owner-specific HTTP handler в Kernel: Core Gateway не
импортирует domain/integration packages и не интерпретирует message, attachment
или provider identity.

## Решение

### Descriptor-declared public Blob surface

`ModuleDescriptorV1` получает additive provided-surface kind `client_blob` и
typed route:

```text
ClientBlobRouteV1 {
  path
  max_response_bytes
}
```

Route принадлежит ровно одному capability и одному exact contract reference.
Path находится только под `/api/blobs/`, не принимает query string и уникален
среди approved registrations. `max_response_bytes` обязателен, ограничен hard
Kernel policy и не может быть больше capability Blob quota.

Control Store сохраняет route отдельно от `client_rpc`. Approval, revoke,
descriptor successor, runtime generation и grant epoch применяются к
`client_blob` независимо; совпадение custody scope не выдаёт route другой
capability.

### Двухфазная authorization без выдачи BlobRef client

Owner сначала выдаёт через свой generated `client_rpc` короткоживущую opaque
read capability. Client передаёт её в Protobuf body exact `client_blob` route.
Capability не помещается в URL, query, cookie, log или telemetry.

Core Gateway:

1. аутентифицирует существующую browser/device session;
2. выбирает exact descriptor-declared `client_blob` route;
3. пересылает bounded opaque authorization request текущему module runtime
   через generic module-client delivery;
4. принимает только platform-owned
   `ModuleClientBlobAuthorizationV1` с reference ID, declared size, full-read
   SHA-256 binding и backup class;
5. проверяет current registration, exact route capability, runtime generation,
   grant epoch, read-only Blob operation и response bound;
6. выпускает одноразовую digest-bound `read_range` session для полного
   declared range;
7. читает bytes по private Blob data socket и возвращает их через authenticated
   Gateway response.

Owner authorization response содержит technical Blob metadata, но не content.
Content bytes идут только:

```text
Blob service -> Core Gateway client_blob adapter -> authenticated client
```

Они не проходят через module control response, NATS, SSE, PostgreSQL, Control
Store, telemetry или error payload. Gateway не декодирует и не преобразует
content.

Первая revision поддерживает только bounded full reads. Partial public range,
upload, preview, attachment download и resumable transfer требуют отдельных
route contracts; отсутствие Range header не превращается в unbounded Blob
fallback, потому что owner authorization возвращает exact declared size, а
Kernel создаёт explicit full-range session с digest binding.

### HTTP и privacy

Public route:

- принимает только `POST` с `application/proto`;
- использует текущую authenticated session cookie;
- запрещает query string, redirect и token в URL;
- возвращает `application/octet-stream`, exact `Content-Length`,
  `Cache-Control: no-store` и `X-Content-Type-Options: nosniff`;
- не возвращает BlobRef, digest, filesystem path, provider locator, runtime
  identity или grant;
- применяет общий deadline и descriptor/hard byte limits;
- переводит failure в sanitized typed transport code без private details.

Browser не кэширует opaque read capability. Capability является one-use,
short-lived и runtime-local; restart/revoke/expiry делает её недействительной.

## SRP и единицы сборки

- `makosh-runtime-protocol` владеет descriptor surface и technical
  authorization response;
- Kernel descriptor/control-store packages владеют admission и current route
  authority;
- `makosh-gateway-runtime` владеет authenticated HTTP semantics;
- Blob client/platform packages владеют private byte session;
- owner module владеет meaning opaque request и решением, какой canonical
  object разрешён;
- frontend owner adapter владеет вызовом exact generated ticket/read
  contracts.

Gateway и Kernel не импортируют owner API/implementation. Owner module не
получает browser cookie и не выдаёт Blob platform grant. Blob Platform не
читает owner tables.

## Gate evidence

`client_blob_v1` считается реализованным только при наличии:

1. additive descriptor/protocol validation и isolated Control Store route;
2. exact approval/revoke/successor persistence;
3. authenticated no-query HTTP route с bounded Protobuf request;
4. module authorization metadata без content bytes;
5. digest-bound one-use full read через live Blob/Vault contour;
6. denial для wrong route/capability/owner, stale runtime/grant, replay,
   malformed authorization, oversize и digest mismatch;
7. tests, что BlobRef/grant/path/digest/content отсутствуют в public metadata,
   SSE, logs и errors;
8. architecture/SRP/Cargo/Clippy и managed Gateway conformance.

## Отклонённые варианты

- Body bytes в ConnectRPC/Protobuf: private content пересекает module control
  channel и делает query transport Blob proxy.
- BlobRef или Blob session grant в browser: client получает внутреннюю
  data-plane capability.
- Owner-specific Gateway handler: Core начинает импортировать domain contract.
- Generic read by client-supplied reference: наличие opaque ID не является
  owner authorization.
