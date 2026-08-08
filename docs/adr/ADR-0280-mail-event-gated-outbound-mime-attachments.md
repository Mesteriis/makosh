# ADR-0280: Mail event-gated outbound MIME attachments

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Реализовано; phase gate
`mail_outbound_mime_attachments_v1` открыт на exact inventory одного integration
owner `mail`.

Зависит от:

- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0220: durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob data sessions](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0246: attachment admission and safety](ADR-0246-communications-attachment-admission-and-safety.md);
- [ADR-0247: Mail SMTP outbound capability](ADR-0247-mail-smtp-outbound-operational-capability.md);
- [ADR-0260: Communications attachment lifecycle](ADR-0260-communications-attachment-lifecycle-event-authority.md);
- [ADR-0261: attachment-anchor handoff](ADR-0261-communications-attachment-anchor-handoff.md);
- [ADR-0262: Mail attachment Blob admission](ADR-0262-mail-attachment-blob-admission-extension.md);
- [ADR-0277: Gmail API outbound mutation gate](ADR-0277-mail-gmail-api-outbound-mutation-gate.md);
- [ADR-0279: durable Blob custody](ADR-0279-durable-blob-custody-scope-and-operation-scoped-grants.md).

## Контекст

SMTP и Gmail adapters уже получают готовый bounded RFC822 и выполняют
provider mutation не более одного раза. Их текущий request содержит только
plain-text body. Client не может приложить существующий attachment без
нарушения одного из действующих инвариантов:

- raw Blob reference не является business/client contract;
- filesystem path и attachment bytes не должны проходить через Gateway;
- Mail не может синхронно query-ить Communications state или storage;
- scanner verdict ещё не является canonical owner state;
- Kernel/Core не интерпретируют attachment identity, safety или MIME;
- provider adapter не должен собирать business message из чужих contracts.

Mail уже получает canonical attachment anchor для собственного source
observation через typed event. Однако owner-local persistence сохраняет только
source-to-anchor mapping и terminal Blob-admission state. Opaque reference,
receipt digest, bounded MIME metadata и canonical safety projection не
сохраняются. Gmail inbound path дополнительно фиксирует descriptor, но ещё не
materializes exact attachment part в Mail Blob custody.

Простое добавление `attachment_ids` в send request поэтому создало бы фасад:
runtime был бы вынужден читать Communications synchronously, доверять сырому
scanner verdict или восстанавливать Blob identity из provider locator.

## Решение

### Owner и единицы сборки

Outbound attachment delivery остаётся capability одного integration owner
`mail`, а не новым domain, workflow или Kernel service:

```text
makosh-mail-api          generated client contract and bounded value types
makosh-mail-core         deterministic MIME composition and validation
makosh-mail-persistence  Mail-owned materialization/safety/delivery state
makosh-mail-runtime      event consumers, Blob sessions and orchestration
makosh-mail-smtp         ready-RFC822 SMTP adapter
makosh-mail-gmail        ready-RFC822 Gmail adapter
```

`makosh-mail-core` не открывает sockets и не знает Blob, Storage,
Communications runtime или provider implementation. SMTP/Gmail packages не
получают anchor IDs, Blob references, safety state или persistence access.
Runtime остаётся единственным Mail composition root.

### Canonical safety projection только через event

Mail получает canonical
`communication_attachment_safety_state_changed.v1` как independently approved
durable event route. Он не подписывается на raw scanner verdict как authority
для delivery.

Mail применяет event в owner-local projection только если:

1. exact contract/schema и event semantics совпадают;
2. anchor уже связан с Mail-owned source observation;
3. inbox message ID/hash не конфликтуют;
4. persisted state равен `expected_state`;
5. transition допустим canonical lifecycle.

Replay с тем же ID/hash идемпотентен. Conflicting replay, unknown anchor,
skipped transition или stale expected state fail closed. Delivery допускается
только из projected terminal `safe_for_delivery`.

Это event-only cross-owner interaction:

```text
Attachment Security observation
  -> Communications canonical CAS
  -> communication_attachment_safety_state_changed.v1
  -> Mail owner-local projection
```

Mail не импортирует Communications domain/runtime/persistence, не вызывает
Communications RPC и не получает его database role. Public
`makosh-communications-attachment-contract` остаётся единственной compile-time
границей для lifecycle event.

### Mail-owned attachment materialization

После успешного Mail Blob write owner-local storage атомарно связывает:

- canonical attachment anchor ID;
- Mail source observation ID;
- opaque Blob reference ID;
- exact receipt SHA-256 и declared byte size;
- bounded filename, media type и disposition;
- current canonical safety state/evidence.

Business metadata принадлежит Mail storage. Bytes, encryption, content key,
quota и at-rest custody принадлежат Blob Platform. Ни одна Mail table не имеет
foreign key или SQL lookup в Communications.

IMAP и Gmail используют один bounded MIME extraction contract из
`makosh-mail-core`. Gmail materialization не копирует Gmail API semantics в
core: adapter возвращает exact raw RFC822, core выбирает exact bounded part, а
runtime выполняет тот же Mail Blob admission/event flow, что и IMAP.

### Client delivery contract

`SendMailRequestV1` получает только ordered canonical attachment anchor IDs.
Он не принимает:

- Blob reference, receipt или custody scope;
- filesystem path, URL или provider locator;
- attachment bytes или arbitrary MIME fragment;
- scanner verdict или client-declared safety flag.

Каждый ID имеет exact 16-byte representation, список non-empty IDs уникален и
bounded. В первом gate допускается не более 16 attachments, не более 16 MiB
decoded bytes суммарно и не более 24 MiB готового RFC822. Existing text/body
limits сохраняются.

Mail принимает command durable только после проверки, что каждый anchor:

- принадлежит Mail-owned mapping;
- имеет committed Blob metadata и non-zero receipt digest;
- projected как `safe_for_delivery`;
- укладывается в individual/aggregate bounds.

Exact command bytes остаются idempotency authority. Повтор того же operation ID
с теми же bytes не создаёт вторую delivery; конфликтующие bytes отклоняются.

### Delivery-bound Blob reads и MIME composition

Claimed Mail delivery materializes каждый attachment через exact
`ReadRange` operation для custody `mail.attachment.content.v1`. Session request
связан с persisted reference, declared size и receipt SHA-256. Runtime:

1. получает short-lived process-bound Blob session;
2. читает ровно declared range один раз в claimed operation path;
3. сверяет length и SHA-256;
4. передаёт validated part в Mail core;
5. сохраняет exact готовый RFC822 digest до provider mutation.

Один attachment может участвовать в разных explicit delivery operations, но
один claimed operation не materializes его повторно. Capability/grant сам по
себе не является business authorization: anchor ownership и canonical safety
проверяются Mail persistence до выдачи session request.

Mail core строит deterministic `multipart/mixed`:

- CRLF normalization и header-injection rejection обязательны;
- boundary детерминирован и не зависит от provider;
- text и attachment bodies имеют bounded transfer encoding;
- filename/media type/disposition проходят typed validation;
- raw part headers и client MIME fragments не переиспользуются;
- output limit проверяется до provider call.

SMTP и Gmail получают только готовые RFC822 bytes. Они не знают, был ли message
plain-text или multipart.

### Durable execution и failure semantics

Receipt клиенту означает только persisted Mail command. Provider result
остаётся terminal query/event outcome.

Mail claim по-прежнему разрешает максимум одну provider mutation. Ошибка
projection, missing Blob, receipt/hash mismatch, bounded read или MIME
composition до provider call становится deterministic `rejected`. Ошибка после
начала SMTP/Gmail mutation становится `outcome_unknown` и автоматически не
retry-ится.

Exact duplicate не делает второй Blob materialization или provider mutation.
NATS outage после provider success сохраняет terminal state и exact neutral
Communications observation в Mail outbox; attachment IDs, filenames, content,
recipient и subject в observation/subject/log/error не попадают.

### Kernel/Core boundary

Kernel:

- validates exact descriptor operations and custody scope;
- authorizes event route и Blob session;
- fences runtime/generation/grant epoch;
- routes opaque client/control payloads.

Kernel не декодирует Mail command, не читает safety state, не выбирает
attachments, не строит MIME и не вызывает provider. Core Gateway не принимает
Blob bytes и не становится attachment facade.

## Phase gate `mail_outbound_mime_attachments_v1`

Gate открывается атомарно только при наличии:

1. ADR implementation state и executable inventory/policy update;
2. generated client contract с bounded canonical anchor IDs;
3. отдельного Mail subscribe capability для canonical safety-state event;
4. Mail Blob quota с exact `Write` и `ReadRange`, без foreign custody/read-all;
5. immutable Mail Storage bundle revision для Blob metadata, safety projection
   и delivery manifest;
6. deterministic MIME unit/conformance tests с injection, size, duplicate-ID,
   invalid metadata и hash-mismatch negatives;
7. IMAP и Gmail exact-part materialization evidence;
8. live canonical `safe_for_delivery` event projection и rejection для
   quarantined/rejected/stale/unknown anchors;
9. live Blob/Vault read, SMTP и Gmail delivery с exact decoded attachment;
10. exact duplicate, runtime restart, NATS outage и provider ambiguity evidence;
11. architecture/SRP/Cargo/Clippy/full backend gates.

Frontend не является proof этого gate.

### Текущее implementation evidence

| Критерий | Состояние | Evidence |
|---|---|---|
| Exact owner/package inventory | Complete | `backend/architecture/policy.json` допускает ровно domain `communications`, engine `attachment_security` и integration `mail`; Mail состоит из восьми самостоятельных API/core/provider/persistence/runtime/assembly Cargo units. |
| Generated bounded client contract | Complete | `SendMailRequestV1` переносит только ordered non-zero unique 16-byte canonical anchor IDs, максимум 16; Blob references, bytes, paths, provider locators и safety flags отсутствуют. |
| Event-only canonical safety | Complete | Mail имеет отдельный durable consume capability для `communication_attachment_safety_state_changed.v1`; projection проверяет exact contract/source/partition/lineage и применяет owner-local CAS без Communications query/runtime/storage edge. |
| Mail Blob custody | Complete | Descriptor запрашивает только `Write` и `ReadRange` для `mail.attachment.content.v1`; delivery сверяет persisted reference, declared size и receipt SHA-256 до MIME/provider call. |
| Durable Mail state | Complete | Immutable Storage bundle V5 добавляет materialization, safety projection и delivery manifest; V6 вводит monotonic `causal_sequence`, чтобы body observation всегда предшествовал attachment update при одинаковом wall-clock timestamp. |
| Deterministic MIME and adapter SRP | Complete | `makosh-mail-core::outbound_mime` владеет bounded RFC822/MIME, injection/metadata/hash/size validation; SMTP и Gmail получают только готовые bytes и не импортируют Communications/Blob/persistence. |
| Live SMTP and Gmail conformance | Complete | `managed_mail_delivers_only_canonical_safe_attachment_from_its_blob_custody` и `managed_gmail_materializes_then_delivers_canonical_safe_attachment` проходят через managed Vault, Blob, Storage, NATS, Communications CAS и exact decoded provider attachment. |
| Failure/replay matrix | Complete | Live evidence покрывает unknown/stale/quarantined safety, exact duplicate, successor Mail runtime restart, NATS outage, post-DATA provider ambiguity и отсутствие автоматической повторной provider mutation. |
| Executable architecture and quality gates | Complete | Policy schema, exact Cargo dependency/feature inventory, Communications/domain isolation, Mail SRP boundary, Cargo/Clippy/workspace/integration/full backend gates являются обязательной commit evidence. |

## Последствия

- Mail получает необходимую read capability только для собственного stable
  custody scope.
- Communications остаётся canonical safety owner, но не входит в Mail call
  path.
- Provider adapters остаются заменяемыми и не зависят от domain contracts.
- Attachment selection становится typed public Mail operation, а не generic
  cross-domain command.
- Gmail inbound Blob materialization закрывается тем же owner path, без второй
  Gmail-specific attachment architecture.

## Отклонённые варианты

### Синхронный Communications query из Mail

Отклонено: создаёт runtime availability coupling и domain/integration facade.

### Доверять raw scanner verdict

Отклонено: только Communications CAS является canonical lifecycle authority.

### Передавать Blob reference или bytes через client/Gateway

Отклонено: platform reference не является business contract, а Gateway не
является Blob/content proxy.

### Собирать MIME в SMTP или Gmail adapter

Отклонено: дублирует owner logic и связывает safety/Blob semantics с provider.

### Хранить attachment bytes в Mail PostgreSQL

Отклонено: large binary content остаётся в Blob; Mail SQL хранит только typed
metadata, projection, receipts и delivery state.

### Разрешить generic Mail read-all Blob capability

Отклонено: descriptor объявляет exact operation и custody scope, а каждый read
дополнительно связан с persisted delivery/receipt.
