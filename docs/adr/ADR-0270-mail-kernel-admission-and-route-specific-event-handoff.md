# ADR-0270: Mail Kernel admission and route-specific event handoff

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Backend inbound, bounded plain-text SMTP и Gmail delivery
gates реализованы. Generated Mail API и descriptor теперь имеют независимые
`mail.sync.v1`, `mail.delivery.v1` и `mail.delivery.query.v1` routes,
три provider-purpose credential capabilities, три route-specific event
capabilities и один canonical module ID `makosh-mail-runtime` во всех
Mail-produced envelopes. Umbrella `mail.client` и `mail.events.v1` удалены из
production code; assembly повторно доказала signed exact descriptor revision 2
и settings schema revision 3.
Signed managed launch теперь проходит через exact Kernel registration,
owner-approved IMAP sync subset и Kernel-issued Storage/Vault/Blob/Event Hub
bindings. Kernel до relay отклоняет отсутствующий delivery grant и stale
runtime generation. Owner-authorized revoke повышает grant epoch, выполняет
exact Storage/Vault/PgBouncer/PostgreSQL fence, останавливает только Mail worker
и оставляет Communications активным. Live Mail-owned outbox → NATS →
Communications delivery теперь доказана вместе с inbox deduplication и outage
replay. Live provider sync и attachment anchor → Mail mapping → Kernel-issued
Blob write → terminal Communications CAS conformance теперь доказаны.
`mail_runtime_admission_v1` открыт для exact inbound sync subset. Отдельный
SMTP delivery gate доказывает durable acceptance, terminal query, один
provider execution и event-only outage replay. Отдельный Gmail delivery gate
доказывает outbound-only GrantSet, bounded HTTPS mutation, один provider
execution и event-only outage replay. Outbound attachments и frontend cutover
остаются отдельными закрытыми slices.
Frontend generator и раздельные `MailSyncService`,
`MailDeliveryCommandService` и `MailDeliveryQueryService` Connect client units
реализованы как prerequisite; legacy Mail surfaces ими ещё не заменены.
Отдельная optional scan-candidate publish capability, Mail-owned durable outbox
и live integration-to-engine observation реализованы как prerequisite
Attachment Security gate; они не дают engine runtime production admission.

Уточняет:

- ADR-0201: Core/module communication and NATS;
- ADR-0204: integration and provider-neutral context boundary;
- ADR-0205: Core Gateway and client transport;
- ADR-0215: module registration and capability grants;
- ADR-0219: managed distribution integrity;
- ADR-0221: module descriptor and capability lifecycle;
- ADR-0256: owner-declared ClientRpc route admission;
- ADR-0261: Communications attachment-anchor handoff;
- ADR-0262: Mail attachment Blob-admission extension;
- ADR-0263: Mail settings and Storage admission artifacts;
- ADR-0269: Mail release assembly unit.

## Контекст

Mail является integration owner. Он взаимодействует с Kernel/Core для
registration, managed launch, settings, Storage, Vault, Blob, Event Hub и
provider-operational ClientRpc routing. Это platform control plane, а не
business-вызов Communications.

Provider-neutral Mail evidence пересекает owner boundary только через durable
typed events. Kernel проверяет grant, runtime generation и route metadata, но
не декодирует Mail payload и не вызывает Communications.

Один event capability также не может объединять разные authority. Publish
neutral observation, consume Communications attachment anchor и publish
attachment terminal observation имеют разные operational причины и выдаются
как независимые approval units.

Текущий Mail operational contract объединяет две независимые операции в одном
`mail.client`:

- inbox sync, который читает внешний provider и создаёт observations;
- delivery, которая изменяет внешний provider.

Один grant на обе операции выдаёт лишнее право. Аналогично один
`mail.credentials.v1` объединяет IMAP password, Gmail access token и SMTP
password, хотя configuration instance использует только необходимое
подмножество.

Attachment Blob-admission producer также обязан использовать exact admitted
module identity `makosh-mail-runtime`; сокращённый `mail-runtime` создаёт
вторую identity и нарушает runtime/grant fencing.

## Решение

### Owner и единицы сборки

Production runtime owner:

```text
owner_id  = mail
module_id = makosh-mail-runtime
```

Mail source unit:

```text
makosh-mail-api
makosh-mail-core
makosh-mail-imap
makosh-mail-gmail
makosh-mail-smtp
makosh-mail-persistence
makosh-mail-runtime
```

`makosh-mail-assembly` является отдельной integration-owned build-time unit.
Она создаёт unsigned release input, не запускается Kernel и не входит в
runtime inventory или GrantSet.

Communications остаётся отдельным domain owner. Единственная разрешённая
integration → domain compile dependency — typed neutral contract
`makosh-communications-ingress`. Ни один Mail package не импортирует
Communications domain/persistence/runtime/API, а Communications не импортирует
Mail packages.

### Kernel/Core control plane

Kernel:

- регистрирует exact descriptor bytes как `pending`;
- применяет explicit owner-approved capability subset;
- проверяет signed executable/descriptor/settings/storage bindings;
- выдаёт monotonic runtime generation и grant epoch;
- создаёт fenced Storage, Vault, Blob и Event Hub routes;
- маршрутизирует opaque ClientRpc bytes по exact approved contract;
- отзывает routes и leases при suspend, revoke, binding replacement или stale
  runtime identity.

Kernel не:

- декодирует Mail API или Communications ingress payload;
- выбирает provider, mailbox, sync window или delivery recipient;
- хранит provider credential/session или Mail projection;
- создаёт Communications evidence;
- вызывает Communications runtime или SQL.

### Route-specific provider operational contracts

Generated Mail Protobuf descriptor set предоставляет три независимых routes:

| Capability | Contract | Connect path | Responsibility |
|---|---|---|---|
| `mail.sync.v1` | `mail.sync.v1` | `/makosh.mail.v1.MailSyncService/Sync` | bounded inbound sync |
| `mail.delivery.v1` | `mail.delivery.v1` | `/makosh.mail.v1.MailDeliveryCommandService/Send` | durable outbound acceptance |
| `mail.delivery.query.v1` | `mail.delivery.query.v1` | `/makosh.mail.v1.MailDeliveryQueryService/GetOperationStatus` | Mail-owned terminal status |

Все contracts имеют `major = 1`, `revision = 2` и exact SHA-256 одного
generated descriptor set. Общий digest не объединяет capabilities: route,
payload type, authority и failure mode различны.

Mail runtime получает exact contract reference из routed
`ModuleClientRequestV1` и декодирует только соответствующий generated request.
Oneof umbrella `MailOperationalService/Execute`, decode probing, REST alias и
fallback запрещены.

Inbox sync, delivery command и delivery query остаются разными functional
ports, даже если один managed process реализует все три. `accepted` означает
только durable persistence; provider response доступен только через terminal
query и не возвращается синхронным command route.

### Provider credential capabilities

Credential grants разделяются по purpose:

```text
mail.imap.credentials.v1  -> mail_imap_password
mail.gmail.credentials.v1 -> mail_gmail_access_token
mail.smtp.credentials.v1  -> mail_smtp_password
```

Каждый capability optional в descriptor и становится effective только после
explicit approval. Configuration instance с IMAP не получает Gmail/SMTP
credential route; Gmail не получает IMAP; SMTP выдаётся только вместе с
approved delivery configuration.

Runtime не может расширить права через settings: settings содержат только
revision, а Vault route дополнительно проверяет exact capability, purpose,
configuration instance, runtime generation и grant epoch.

### Event route capabilities

Event authority разделена по функциональной ответственности:

```text
mail.communication-observed.publish.v1
mail.attachment-anchor.consume.v1
mail.attachment-blob-admission.publish.v1
mail.attachment.scan-candidate.publish.v1
```

Каждый capability содержит ровно свой event route и может быть approved/revoked
независимо. Outbound-only delivery получает только neutral observation
publish; оно не получает attachment consume/publish или scan-candidate rights.
Umbrella `mail.events.v1` отсутствует в descriptor и live GrantSet.

### Event-only handoff

Inbound evidence:

```text
External Mail provider
        ↓
Mail runtime
        ↓
Mail-owned PostgreSQL outbox
        ↓ exact DurableEnvelopeV1 bytes
NATS JetStream
        ↓
Communications inbox/deduplication
        ↓
Communications-owned state and events
```

Attachment continuation:

```text
Mail source observation
        ↓ event
Communications attachment anchor
        ↓ communication_attachment_anchor_recorded.v1
Mail owner-local mapping
        ↓ one-use Blob lease
Mail owner-local outbox
        ↓ communication_attachment_blob_admission_observed.v1
Communications CAS projection
```

Во всех Mail-produced envelopes `source.module_id` равен
`makosh-mail-runtime`. Causation, non-zero correlation, exact contract,
runtime generation и grant epoch сохраняются. Kernel маршрутизирует и
ограничивает transport, но не является business producer/consumer.

Запрещены direct Mail → Communications RPC, runtime socket, shared handler,
cross-owner SQL, anchor derivation в Mail и provider download в
Communications.

## Phase gate `mail_runtime_admission_v1`

Backend gate открывается атомарно только при наличии:

1. route-specific generated Protobuf contracts и exact descriptor references;
2. split sync/delivery, IMAP/Gmail/SMTP credential и event-route capability
   units;
3. signed Mail runtime/descriptor/settings/storage artifacts из ADR-0269;
4. pending registration без прав и explicit owner-approved subset;
5. managed launch с exact runtime generation/grant epoch;
6. exact Storage/Vault/Blob/Event Hub issuance и stale/revoke fencing;
7. live sync route через Core capability router без Mail dependency в Kernel;
8. Mail outbox → NATS → Communications inbox delivery с deduplication и outage
   replay;
9. attachment anchor handoff → Mail mapping → Blob terminal observation с CAS
   conflict/replay evidence;
10. отсутствие provider bodies, credentials, locators и sessions в subjects,
    route metadata, logs, errors и health.

Delivery capability и frontend cutover не доказываются inbound sync gate.
SMTP delivery включается только отдельным live provider mutation evidence;
Gmail delivery включается отдельным live HTTPS mutation evidence. Outbound
attachments и frontend требуют собственных gates.
Frontend не используется как proof backend admission.

Открытие gate:

- не расширяет `first_owner_v1`;
- не добавляет Mail в Communications inventory;
- не превращает integration в domain;
- не разрешает Telegram/WhatsApp/Zulip;
- не доказывается одним ADR или только static tests.

## Порядок реализации

1. Разделить generated routes, client ports и credential capabilities.
2. Исправить exact module identity во всех Mail-produced envelopes.
3. Обновить descriptor/assembly regression evidence.
4. Добавить signed managed launch и Kernel fence conformance.
5. Доказать event-only sync и attachment lifecycle.

Каждый крупный slice является отдельным commit и проходит owner tests,
Clippy, architecture/SRP/Cargo boundaries и relevant live conformance.

## Evidence 2026-07-24

Реализованный managed admission slice:

- signed Mail executable/descriptor/settings binding проверяется при managed
  launch;
- explicit approved subset содержит только Blob, Events, Storage, IMAP
  credential и sync; delivery/Gmail/SMTP остаются без grant;
- Mail Storage bundle и все runtime SQL используют owner-scoped
  `makosh_data.mail_*`;
- Mail получает IMAP credential только через exact Vault purpose
  `mail_imap_password` для своей configuration instance;
- focused live test поднимает disposable PostgreSQL, PgBouncer, NATS и реальные
  managed Vault, Storage, Blob, Communications и Mail processes;
- Kernel отклоняет ungranted delivery и stale sync generation до runtime relay;
- owner-control использует реальную ES256 owner session и production dispatcher
  для перевода exact Mail registration в `revoked`;
- revoke сначала повышает grant epoch и резервирует Mail Storage binding как
  `revoking`, затем завершает exact Vault/PgBouncer/PostgreSQL fence и
  останавливает Mail worker;
- current Storage generation может вызвать только exact `RevokeAudience` для
  durable revoking binding; другая operation и stale Storage generation
  отклоняются;
- Communications worker остаётся активным, а прежний Mail sync route после
  revoke отклоняется до runtime relay.

Реализованный event-only handoff slice:

- Mail runtime использует один `ManagedControlChannelV2` для lifecycle,
  provider credentials, Storage/Vault, Event Hub, Blob и client delivery;
  `UnixStream::try_clone`, `MSG_PEEK` и V1 request helpers удалены;
- nested client delivery во время platform request получает correlated
  `RUNTIME_BUSY`, не потребляя ответ ожидающего platform operation;
- provider credential lease запрашивает exact descriptor-bounded
  `MAIL_CREDENTIAL_LEASE_TTL_SECONDS`, а не общий Vault default;
- Mail descriptor не содержит `mail.events.v1`: neutral observation publish,
  attachment-anchor consume и attachment terminal publish выданы раздельно;
- runtime подключается только к выданному PgBouncer `pool_alias` с
  `effective_budgets.max_connections`; миграцию применяет Storage Control,
  runtime DDL при bootstrap удалён;
- live managed Mail process публикует exact Mail-owned outbox envelope в NATS,
  Communications создаёт canonical evidence с исходным causation;
- повторная публикация тех же exact bytes доставляется подписчику как исходное
  observation, но Communications inbox не создаёт второй canonical event;
- при остановленном NATS второй exact envelope остаётся pending, Mail runtime
  остаётся active без recorded failure, а после возврата NATS envelope и
  canonical Communications event воспроизводятся;
- transient недоступность attachment-anchor subscription теперь является
  retryable transport outage, а malformed payload и persistence failure
  остаются fatal.

Проверки:

```text
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_runtime_uses_kernel_leases_and_route_specific_admission node scripts/test-authenticated-storage.mjs 1.97.0
cargo +1.97.0 test -p makosh-mail-api -p makosh-mail-imap -p makosh-mail-runtime -p makosh-mail-persistence
cargo +1.97.0 test -p makosh-mail-api -p makosh-mail-imap --features conformance-test-support
cargo +1.97.0 test -p makosh-kernel-recovery-testkit --no-run
cargo +1.97.0 clippy -p makosh-mail-runtime -p makosh-mail-persistence -p makosh-kernel-recovery-testkit --all-targets -- -D warnings
make -C backend architecture-policy-check srp-policy-check cargo-boundaries-check test-architecture
```

Реализованный live attachment slice:

- test-only loopback IMAP transport отделён compile-time feature; default Mail
  API/runtime сохраняют implicit TLS port `993`, а plaintext разрешён только
  для `localhost`/loopback в conformance build;
- active `mail.sync.v1` route получает exact credential из Vault и читает
  реальный RFC822/MIME provider fixture;
- Communications создаёт canonical attachment anchor и публикует typed handoff;
  Mail проверяет и сохраняет mapping в своей PostgreSQL schema;
- Mail запрашивает Blob write session у Kernel по exact `mail.blob.v1`
  capability, пишет provider bytes напрямую в Blob и публикует typed
  `requested`/`admitted` observations из owner-local outbox;
- Communications применяет две CAS transition и выдаёт public
  `blob_admitted`, не получая provider locator, MIME bytes или Blob path;
- повторный provider sync, exact terminal replay и stale expected-state
  observation не создают повторную admission/canonical transition;
- тот же managed flow до и после attachment slice доказывает ungranted route,
  stale runtime generation и owner-authorized revoke fences.

Эти evidence закрывают пункты 1–10 phase gate
`mail_runtime_admission_v1` для exact inbound sync subset. Gate не расширяет
`first_owner_v1` и не допускает outbound delivery, Gmail/SMTP mutation,
scanner verdict producer или frontend.

Реализованный live SMTP delivery slice:

- отдельные optional `mail.delivery.v1` command и
  `mail.delivery.query.v1` query capabilities могут быть одобрены независимо
  от inbound sync route;
- command route сохраняет exact generated request и RFC822 digest в
  Mail-owned PostgreSQL transaction до возврата operation receipt;
- owner-local queue claim делает ambiguous provider outcome
  `outcome_unknown` и не выполняет автоматический retry, исключая скрытую
  повторную отправку;
- SMTP runtime получает только exact `mail_smtp_password` lease и выполняет
  bounded implicit-TLS exchange через отдельный `makosh-mail-smtp` package;
- successful provider mutation и neutral Communications observation
  фиксируются атомарно в Mail state/outbox;
- focused managed test останавливает NATS до command, получает durable
  acceptance и terminal `accepted`, подтверждает одну SMTP mutation и
  отсутствие второй mutation для exact duplicate;
- после возврата NATS exact Mail observation воспроизводится в Communications
  с исходным causation, а private recipient/body отсутствуют в
  `DurableEnvelopeV1`;
- generated frontend descriptor синхронизирован тем же canonical generator,
  без handwritten REST или compatibility facade.

Проверка:

```text
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_runtime_accepts_then_completes_smtp_delivery_and_replays_event node scripts/test-authenticated-storage.mjs 1.97.0
```

Этот gate открывает только bounded plain-text SMTP delivery. Gmail delivery
доказывается отдельным gate ниже; outbound MIME attachments и frontend cutover
остаются отдельными slices.

Реализованный live Gmail delivery slice:

- Gmail profile получает только delivery command/query, Gmail credential,
  Storage и neutral observation publish capabilities;
- signed settings schema revision 3 содержит production-fixed
  `gmail.googleapis.com:443`; custom CA/loopback endpoint доступны только в
  conformance build;
- `makosh-mail-gmail` выполняет bounded TLS/HTTP mutation и не зависит от Mail
  persistence/runtime, Communications, Kernel или Vault implementation;
- command receipt возвращается после Mail-owned persistence, terminal provider
  result читается отдельно, а ambiguous HTTP outcome не retry-ится;
- focused managed test проверяет exact Gmail path, Bearer access token, thread
  ID, RFC822 bytes и ровно одну mutation для exact duplicate;
- NATS останавливается до command: provider result и neutral outbox
  фиксируются, Mail остаётся active, после восстановления NATS exact
  observation доходит до Communications;
- provider access token, recipient, body и Gmail receipt отсутствуют в durable
  event bytes.

Проверка:

```text
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_mail_gmail_runtime_mutates_once_and_replays_event_without_private_payload node scripts/test-authenticated-storage.mjs 1.97.0
```

Gmail gate не открывает sync, IMAP, SMTP, Blob или attachment capabilities.
Outbound MIME attachments и frontend cutover остаются отдельными slices.

## Отклонённые варианты

### Оставить один `mail.client`

Отклонено: sync grant не должен разрешать outbound delivery.

### Оставить один credential capability

Отклонено: активный IMAP account не должен получать Gmail или SMTP secret
purpose.

### Пусть Kernel вызывает Communications

Отклонено: Kernel стал бы owner-specific business facade и интерпретировал бы
payload.

### Пусть Mail вызывает Communications API

Отклонено: direct cross-owner dependency обходит durable event, inbox
deduplication и failure isolation.

## Последствия

Mail получает узкие, независимо выдаваемые capability units и точную границу:
integration общается с Kernel/Core только для platform control/routing, а с
Communications — только durable typed events. Цена — миграция umbrella
contract и дополнительные conformance slices; это обязательная стоимость
least privilege и SRP.
