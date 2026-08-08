# ADR-0315: Communications message body content read

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: implemented. Gate закрыт отдельным content API,
  owner-local receipt authorization, one-use runtime tickets, authenticated
  client Blob delivery, generated frontend adapter и inert UTF-8 presentation.
- Связанные решения: ADR-0204, ADR-0205, ADR-0212, ADR-0213, ADR-0230,
  ADR-0231, ADR-0240, ADR-0254, ADR-0257, ADR-0279, ADR-0281, ADR-0313,
  ADR-0314

## Контекст

Communications canonical read v2 возвращает metadata, evidence, participants,
attachment anchors и references, но намеренно не возвращает message body или
Blob locator. Canonical owner уже хранит Communications-owned Blob receipt
после event-backed custody transfer и читает его для derived exact-token
search.

Frontend detail без body неполон. Provider fallback запрещён: Mail, Telegram,
WhatsApp и Zulip operational projections не являются canonical content
authority, а domain не импортирует их clients или storage.

## Решение

### Отдельная content capability

Создаётся отдельная build unit `makosh-communications-content-api` и exact
capability `communications.content.v1`. Metadata contract
`communications.query.v1` не расширяется content bytes или Blob fields.

Capability предоставляет две поверхности:

1. generated `client_rpc` ticket issuance:

```text
IssueMessageBodyRead(message_id)
  -> opaque_read_capability
  -> declared_bytes
  -> expires_at
```

2. `client_blob` request, который принимает только opaque read capability и
   возвращает bytes через ADR-0314.

Capability отдельно запрашивает `read_range` для существующего
`communications.evidence.body.v1` custody scope. Она не получает write или
custody-transfer operation. Existing `communications.blob.v1` остаётся
worker authority для custody/search и не становится public client surface.

### Canonical authorization

Ticket issuance принимает только 16-byte canonical `message_id`. Persistence
одним owner-local read связывает message с его current evidence и допускает
ticket только когда:

- message существует и не находится в deleted lifecycle state;
- current body state равен `admitted_blob`;
- exact Communications-owned receipt имеет 16-byte reference ID;
- declared size находится в `1..=256 KiB`;
- receipt SHA-256 имеет 32 bytes.

Provider ID, source cursor, account, mailbox/chat locator, filename, arbitrary
BlobRef и requested range не принимаются.

Ticket является CSPRNG-generated, 32-byte, one-use, in-memory capability с TTL
не более 30 seconds. Она привязана к current runtime generation, logical owner,
canonical message ID и exact receipt. Ticket не сохраняется в PostgreSQL,
Control Store, localStorage, URL, logs или telemetry. Runtime restart, grant
revoke, successful consume или expiry уничтожает authority.

`client_blob` authorization atomарно consume-ит ticket и возвращает Core
Gateway только technical `ModuleClientBlobAuthorizationV1`. Повтор,
expired/wrong owner token и заменённый current message receipt fail closed.
Перед выдачей authorization runtime повторно сверяет current owner-local
message/receipt, поэтому edit/delete не оставляет stale read authority.

### Content semantics

Первая revision возвращает exact admitted UTF-8 message-body bytes как
`application/octet-stream`; frontend декодирует UTF-8 с fatal validation,
sanitizes presentation и не исполняет markup. Invalid UTF-8 показывается как
content unavailable и не преобразуется lossy decoder.

Attachment content, inline media, previews, exports, provider raw MIME,
rendered HTML и AI context не входят в этот gate. Attachment anchor metadata
не является body capability.

### Frontend SRP

```text
canonicalCommunicationsContent
  generated ticket issue + authenticated client_blob fetch

useCanonicalCommunicationContent
  selected-message lifecycle, abort and stale-request fencing

canonicalCommunicationContentModel
  fatal UTF-8 decode and display state

CanonicalCommunicationContent
  inert text presentation
```

Metadata detail controller не импортирует Blob transport. Provider integration
screens не импортируют Communications content adapter.

## Gate `communications_content_read_v1`

Gate становится `implemented` только атомарно при наличии:

1. отдельной content API build unit и exact descriptor contracts;
2. read-only shared-custody capability без write/transfer;
3. owner-local current-message/receipt authorization;
4. short-lived one-use capability with replay/expiry/restart fences;
5. ADR-0314 authenticated client_blob full read with digest binding;
6. managed message observation -> custody transfer -> ticket -> exact bytes;
7. edit/delete/stale receipt/revoke/Blob outage negative evidence;
8. generated frontend adapter and inert UTF-8 presentation;
9. no provider fallback and no BlobRef/grant/digest exposure;
10. backend/frontend/architecture/SRP/Cargo/browser validation.

## Rollback

- revoke `communications.content.v1` without revoking metadata query;
- remove its ClientRpc/client_blob routes from effective composition;
- invalidate all runtime-local tickets;
- keep canonical evidence and Blob custody unchanged;
- never fall back to provider operational content or legacy REST.
