# ADR-0324: Empty Protobuf client RPC request semantics

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: implemented в Gateway, Kernel envelope,
  `ModuleClientRequestV1` validation и Mail generated-client adapter. Live
  browser proof подтверждает два Mail targets через provider-owned catalog.
- Связанные решения: ADR-0205, ADR-0215, ADR-0221, ADR-0300, ADR-0320

## Контекст

ConnectRPC unary method может использовать пустое Protobuf message. Его
canonical binary encoding имеет длину zero bytes; это valid payload, а не
отсутствующий transport body.

Core Gateway и canonical `ModuleClientRequestV1` validation отвергали
empty `application/proto` payload на двух последовательных transport gates до
вызова descriptor-declared owner handler. Это делало canonical encoding любого
действительно пустого Protobuf message неотличимым от invalid transport body.
Добавление искусственного поля только ради ненулевого wire body исказило бы
owner contract и перенесло transport defect во все owners.

Отдельно Mail frontend вызывал `MailAccountCatalogService/List` без
обязательного contract-major. Proto3 кодировал default `major = 0` как empty
bytes, но Mail owner contract требует `major = 1`. После исправления transport
gates этот запрос обязан fail-closed на Mail decoder, а generated-client adapter
обязан отправлять typed `{ major: 1 }`. Эти два дефекта не являются одним:
transport принимает valid empty message, а Mail сохраняет свою version
validation.

## Решение

Authenticated descriptor-declared `client_rpc` и canonical
`ModuleClientRequestV1` validator принимают zero-length `application/proto`
payload как valid Protobuf payload и передают exact bytes owner runtime без
добавления sentinel, wrapper или owner-specific semantics.

Gateway продолжает до routing отклонять:

- wrong HTTP method или path;
- query string;
- неподдерживаемый content type;
- invalid session;
- malformed timeout;
- body больше hard limit;
- body collection failure или deadline.

Пустой payload не означает valid business request автоматически. Exact owner
decoder остаётся authority: empty message принимается только контрактом,
который действительно декодирует его; контракт с обязательным полем возвращает
свою sanitized typed ошибку.

`MailAccountCatalogService/List` не является empty-message contract:
frontend adapter передаёт `major = 1`, а Mail decoder отклоняет отсутствующий,
нулевой или неизвестный major.

Connect response semantics не меняются. Empty successful response уже
разрешён и остаётся distinct от Connect error envelope.

## SRP и единицы сборки

- `makosh-gateway-runtime` владеет HTTP/Connect и Protobuf transport semantics;
- owner API package владеет request schema и validation;
- integration/domain runtime владеет meaning пустого request;
- Kernel relay переносит opaque bytes и не интерпретирует owner payload;
- frontend использует generated client и не добавляет fake transport fields.

Mail не получает Gateway special case. Gateway не импортирует Mail contract,
а Mail catalog остаётся integration-owned public query.

## Gate evidence

Решение считается реализованным только при наличии:

1. Gateway unit test, что authenticated declared route передаёт exact empty
   request bytes handler;
2. runtime-protocol unit test, что opaque module request сохраняет empty
   Protobuf payload;
3. отрицательных tests для wrong content type, invalid session, oversize и
   invalid timeout;
4. owner decoder test, что non-empty-required contract по-прежнему fail-closed;
5. frontend regression, что Mail catalog adapter передаёт exact `major = 1`;
6. live proof, что Mail catalog через browser Gateway возвращает два current
   recovery targets;
7. architecture, unit и frontend validation без owner-specific Gateway import.

## Отклонённые варианты

- Добавить `bool requested = true` во все пустые query messages: fake business
  field и массовый contract churn.
- Разрешить empty body только Mail path: owner-specific behavior в Gateway.
- Превратить catalog в Settings query: Settings не владеет provider runtime
  readiness и credential lifecycle.
- Считать HTTP `Content-Length: 0` отсутствующим request: противоречит
  canonical Protobuf encoding empty message.
