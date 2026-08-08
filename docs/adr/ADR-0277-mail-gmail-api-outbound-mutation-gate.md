# ADR-0277: Mail Gmail API outbound mutation gate

Статус: Принято
Дата: 2026-07-25
Состояние реализации: Backend gate `mail_gmail_delivery_v1` открыт. Mail
descriptor revision 2 и settings schema revision 3 содержат независимые
delivery command/query, Gmail credential и split event capabilities.
Signed managed admission выдаёт outbound-only Gmail profile, runtime получает
access token через exact Vault purpose и выполняет bounded HTTPS mutation через
отдельный `makosh-mail-gmail` adapter. Live disposable contour доказывает
durable acceptance, отдельный terminal result, один provider mutation,
idempotent duplicate, privacy boundary и exact-byte event replay в
Communications после NATS outage. Outbound MIME attachments и frontend cutover
в этот gate не входят.

## Контекст

ADR-0236 определяет Gmail API как protocol adapter внутри Mail integration,
пока он реализует Mail operational language. ADR-0270 уже разделяет Mail
delivery command/query и Gmail credential capability, но намеренно не
открывает Gmail mutation без отдельного live gate.

Наличие метода `send_raw_message` не доказывает production capability. Нужны
одновременно exact Kernel admission, owner-local durability, реальный TLS
provider exchange, privacy boundary и Communications handoff без direct
runtime/domain dependency.

## Решение

Gmail outbound остаётся adapter-ом Mail owner:

```text
Core Gateway
    ↓ opaque exact mail.delivery.v1 command
Kernel capability router
    ↓
Mail managed runtime
    ↓ owner-local durable queue
makosh-mail-gmail
    ↓ HTTPS Gmail API mutation
Mail terminal delivery state + outbox
    ↓ exact DurableEnvelopeV1
NATS JetStream
    ↓
Communications inbox/canonical evidence
```

Gmail не становится:

- отдельным business domain;
- отдельной integration только из-за provider identity;
- Kernel service или Gateway facade;
- прямым Communications client.

### Exact Kernel/Core agreement

Outbound-only Gmail configuration получает только пересечение descriptor,
owner-approved GrantSet и hard Kernel policy:

```text
mail.delivery.v1
mail.delivery.query.v1
mail.gmail.credentials.v1
mail.storage.v1
mail.communication-observed.publish.v1
```

`mail.sync.v1`, `mail.imap.credentials.v1`,
`mail.smtp.credentials.v1`, `mail.attachment-anchor.consume.v1`,
`mail.attachment-blob-admission.publish.v1`, Blob и scan-candidate capabilities
этому профилю не нужны. Они не выдаются автоматически и не могут быть получены
через settings. Общий `mail.events.v1` запрещён: publish observation, consume
anchor и publish attachment terminal state являются разными единицами approval
и SRP.

Kernel:

- проверяет signed executable/descriptor/settings/storage artifacts;
- выдаёт exact runtime generation и grant epoch;
- маршрутизирует opaque command/query bytes;
- выдаёт Gmail access-token lease только для
  `mail_gmail_access_token`, exact configuration instance и current runtime
  fences;
- fences route, Storage и credential lease при revoke.

Kernel не декодирует Gmail request/response, не выбирает recipient/thread,
не хранит OAuth token, не вызывает Gmail и не создаёт Communications evidence.

### Durable command and terminal state

`mail.delivery.v1` сохраняет exact generated command bytes и RFC822 digest в
Mail-owned PostgreSQL transaction до возврата receipt. Receipt означает только
durable acceptance.

Mail worker claim выполняет Gmail mutation не более одного раза. После claim:

- подтверждённый `2xx` становится terminal `accepted`;
- deterministic invalid request становится terminal `rejected`;
- transport error, non-2xx provider status или malformed response становится
  `outcome_unknown`;
- `outcome_unknown` не получает автоматический retry, потому что provider мог
  принять запрос до потери ответа.

Terminal state читается только через independently approved
`mail.delivery.query.v1`.

### Provider and evidence boundary

`makosh-mail-gmail` владеет HTTPS, Bearer authentication, Gmail JSON и
base64url RFC822 representation. Он не импортирует Mail persistence/runtime,
Communications, Kernel, Gateway или Vault implementation.

Подтверждённая mutation и neutral Communications observation фиксируются
атомарно в Mail-owned state/outbox. Provider message ID, HTTP response body,
Bearer token, recipient address, subject и message body не входят в
`DurableEnvelopeV1`, subjects, health или diagnostics. Observation сохраняет
только допустимую source provenance, causation и correlation.

### Conformance transport

Production endpoint фиксирован:

```text
gmail.googleapis.com:443
platform trust store
```

Произвольный Gmail-compatible endpoint не является production capability.
Loopback host/port и bounded custom CA разрешены только compile-time feature
`conformance-test-support`. Conformance runtime и его settings schema остаются
exact signed artifacts; environment override, plaintext HTTP и disabled TLS
verification запрещены.

Весь HTTPS exchange имеет bounded deadline и bounded response size.

## Phase gate `mail_gmail_delivery_v1`

Gate открывается только при наличии:

1. exact outbound-only approved capability subset;
2. signed managed launch с Gmail configuration и без IMAP/SMTP grants;
3. exact Vault lease только для Gmail access-token purpose;
4. реального loopback TLS Gmail fixture с проверкой method, path, Bearer token,
   thread ID и decoded RFC822 bytes;
5. durable receipt до provider result и отдельного terminal query;
6. exact duplicate command без второй provider mutation;
7. NATS outage после provider success без остановки Mail runtime;
8. replay exact Mail observation в Communications после восстановления NATS;
9. сохранённого causation и inbox deduplication;
10. отсутствия token, recipient и body в durable event bytes.

Outbound MIME attachments не входят в этот gate и остаются отдельным slice.
Frontend не используется как proof backend admission.

## Evidence 2026-07-25

Реализованный backend slice:

- production API endpoint фиксирован как `gmail.googleapis.com:443` с platform
  trust store; loopback port и bounded custom CA доступны только через
  compile-time `conformance-test-support`;
- whole-operation HTTPS deadline ограничивает connect, TLS, request и response,
  а response body имеет отдельный byte bound;
- exact Gmail settings snapshot подписывается вместе с descriptor/settings
  schema и не допускает environment endpoint override;
- approved GrantSet содержит только `mail.delivery.v1`,
  `mail.delivery.query.v1`, `mail.gmail.credentials.v1`, `mail.storage.v1` и
  `mail.communication-observed.publish.v1`;
- общий `mail.events.v1` удалён: observation publish, attachment-anchor consume
  и attachment terminal publish являются тремя независимыми capability units;
- command сохраняется в Mail-owned queue до receipt, terminal result читается
  через отдельный query route, а ambiguous provider outcome не retry-ится;
- loopback TLS Gmail fixture проверяет exact `POST` path, Bearer token,
  provider thread ID и decoded RFC822 content;
- exact duplicate operation не выполняет вторую Gmail mutation;
- при остановленном NATS provider mutation завершается, Mail runtime остаётся
  active, observation остаётся pending и после восстановления NATS доходит до
  Communications с исходным causation;
- access token, recipient, body и provider receipt отсутствуют в
  `DurableEnvelopeV1`.

Live и regression proof:

```text
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_gmail_runtime_mutates_once_and_replays_event_without_private_payload node scripts/test-authenticated-storage.mjs 1.97.0
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_runtime_uses_kernel_leases_and_route_specific_admission node scripts/test-authenticated-storage.mjs 1.97.0
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_runtime_accepts_then_completes_smtp_delivery_and_replays_event node scripts/test-authenticated-storage.mjs 1.97.0
```

## Отклонённые варианты

### Выдать Gmail configuration полный Mail GrantSet

Отклонено: outbound mutation не требует sync, IMAP, SMTP, Blob или attachment
прав.

### Повторять запрос после ambiguous HTTP failure

Отклонено: Gmail мог принять mutation до потери ответа; silent retry создаёт
duplicate provider write.

### Передать Gmail response в Communications

Отклонено: provider receipt и operational diagnostics принадлежат Mail.
Communications получает только neutral typed evidence.

### Разрешить runtime environment endpoint override

Отклонено: hidden environment overlay обходит signed settings binding и
превращает conformance seam в production SSRF/configuration surface.

## Последствия

Gmail mutation использует те же Mail operational contracts и owner-local
durability, что SMTP, но получает отдельное provider evidence. Kernel/Core
остаётся neutral admission/control/router boundary, integration не становится
domain, а Communications не компилирует и не вызывает Gmail adapter.
