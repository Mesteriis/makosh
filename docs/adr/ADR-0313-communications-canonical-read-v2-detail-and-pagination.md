# ADR-0313: Communications canonical read v2 detail and pagination

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: implemented. Existing metadata-only
  `communications.query.v1` расширен additive revision 2: owner-local keyset
  pagination, exact message detail, managed multi-page/search-to-detail
  evidence и полный generated frontend path подтверждены.
- Связанные решения: ADR-0201, ADR-0204, ADR-0205, ADR-0212, ADR-0213,
  ADR-0215, ADR-0220, ADR-0240, ADR-0253, ADR-0257, ADR-0281, ADR-0282

## Контекст

Communications уже владеет canonical accounts, conversations, messages,
participants, attachment anchors, message references, evidence history и
derived exact-token search. Existing `communications.query.v1` предоставляет
metadata-only list/search/evidence operations через Core Gateway и managed
Communications runtime.

Этого недостаточно для полного reconstruction gate:

- list operations возвращают только первые `limit` rows без continuation;
- search hit содержит canonical `message_id`, но exact message нельзя получить
  по этому ID, поэтому detail из search результата ненадёжен;
- frontend использует accounts, conversations, messages и search, но не
  использует conversation detail, participants, attachments, references и
  evidence history;
- bounded `limit` без `next_cursor` делает canonical rows после первых ста
  недоступными;
- попытка компенсировать это provider query нарушила бы domain/integration
  boundary и сделала бы provider projection canonical truth.

Historical provider folders, labels, read/archive state, drafts, bodies, AI
actions, Review state и provider execution не входят в этот gate.

## Решение

### Existing owner and route remain authoritative

Gate реализуется в существующих Communications build units:

```text
makosh-communications-api
makosh-communications-persistence
makosh-communications-runtime
makosh-communications-assembly
frontend Communications domain
```

Новый domain, integration, shared account service или compatibility facade не
создаётся. Exact route остаётся:

```text
/makosh.communications.query.v1.CommunicationsQueryService/Query
```

`communications.query.v1` получает additive schema revision и capability
revision 2. Existing field numbers и operation semantics не меняются. Kernel
и Core Gateway продолжают переносить opaque bytes, проверять exact
contract/schema digest, runtime generation и grant epoch, но не декодируют
Communications payload.

### Exact message detail

В query oneof добавляется:

```text
GetMessageRequestV1 {
  bytes message_id
}

GetMessageResponseV1 {
  MessageSummaryV1 message
}
```

`message_id` — только canonical Communications ID длиной 16 bytes. Provider
record ID, source cursor, mailbox/chat locator и integration identity не
принимаются. Unknown ID возвращает существующий sanitized owner error без
provider fallback.

Frontend message detail собирается только из owner query operations:

```text
GetMessage
ListMessageEvidence
ListMessageAttachmentAnchors
ListMessageReferences
ListConversationParticipants
```

Это один read controller над одним owner contract, а не cross-owner
composition. Body/content не возвращается этим gate.

### Opaque keyset continuation

Каждая repeated canonical list и exact-token search получает:

```text
request.cursor
response.next_cursor
```

Cursor:

- непрозрачен для client и ограничен 64 bytes;
- имеет exact version и operation kind;
- привязан к hash своего filter scope;
- содержит только canonical ordering anchor;
- не содержит provider cursor, query text, content, Blob reference или secret;
- отклоняется до SQL при wrong length/version/kind/scope;
- никогда не используется как authorization proof.

Pages используют deterministic keyset ordering:

| Operation | Order |
|---|---|
| accounts | `last_observed DESC, account_id ASC` |
| conversations | `last_observed DESC, conversation_id ASC` |
| messages | `last_observed DESC, message_id ASC` |
| participants | `last_observed DESC, participant_id ASC` |
| attachment anchors | `last_observed DESC, attachment_anchor_id ASC` |
| message evidence | `observed_at DESC, evidence_id ASC` |
| search | `observed_at DESC, message_id ASC` |
| message references | `observed_at ASC, reference_kind ASC, reference_id ASC` |

Runtime запрашивает `limit + 1`, возвращает не более `limit` items и создаёт
`next_cursor` только при доказанном следующем row. Limit остаётся
`1..=100`. Cursor одного account/conversation/message/search scope нельзя
применить к другому.

Search cursor scope строится из keyed token digests после получения current
search-index lease. Raw query не сохраняется и не попадает в cursor, events,
logs или persistence.

### Persistence and indexes

Canonical read выполняет SQL только через Communications owner role. Storage
bundle получает additive indexes для keyset order; canonical rows и evidence
не переписываются. Schema successor сохраняет exact predecessor lineage по
ADR-0311.

Ни один query:

- не читает integration tables;
- не вызывает provider runtime;
- не получает Mail/Telegram/WhatsApp/Zulip operational contract;
- не раскрывает source/provider locators;
- не читает Blob bytes;
- не создаёт canonical state.

Search projection остаётся derived и rebuildable. Evidence summary остаётся
canonical authority.

### Frontend SRP

Frontend разделяется по причинам изменения:

```text
canonicalCommunicationsRead
  generated owner query calls and input bounds

canonicalCommunicationsDetail
  exact detail fan-in over the same owner contract

useCanonicalCommunicationsPage
  list/search selection state

useCanonicalCommunicationDetail
  selected-message lifecycle and stale-request fencing

canonicalCommunicationsPageModel
  pure list/search presentation mapping

canonicalCommunicationDetailModel
  pure detail/evidence/attachment/reference mapping

CanonicalCommunicationsPage
  list/search presentation and message selection

CanonicalCommunicationDetail
  metadata-only detail presentation
```

Presentation не импортирует generated client, Gateway transport или
integration code. Search result selection сначала получает exact message и
conversation from Communications; отсутствие detail не заменяется provider
query.

### Gate `communications_canonical_read_v2`

Gate становится `implemented` только атомарно при наличии:

1. additive generated owner contract with exact `GetMessage`;
2. scoped opaque continuation for every repeated list/search operation;
3. bounded inputs and deterministic keyset order;
4. owner-local persistence and additive index bundle successor;
5. exact capability revision/schema hash admission;
6. current runtime/storage/grant fencing through the existing managed route;
7. negative tests for malformed, cross-kind and cross-scope cursors;
8. managed multi-page list, search-to-detail and evidence-history proof;
9. generated frontend adapter for all detail operations;
10. SRP list/search and detail controllers/presentation;
11. exact frontend capability guard and no provider fallback;
12. privacy tests proving absence of provider locators, content and Blob
    references.

`communications_content_read_v1`, saved searches, sender insights, export,
workflows, AI and Review remain separate planned gates.

## Rollback

- revoke exact `communications.query.v1` grant or route;
- hide the canonical owner surface when the exact capability is unavailable;
- stop Communications runtime without stopping integrations;
- keep accepted canonical rows and evidence intact;
- leave additive indexes in place;
- never fall back to legacy REST, provider operational query or cross-owner SQL.

Rollback frontend detail does not change canonical state. Rollback search index
does not remove canonical evidence and only disables derived search.

## Последствия

Communications becomes navigable beyond the first bounded page and a search hit
can open exact canonical metadata/evidence without consulting its provider.
Domain ownership remains clean: integrations continue to own operational
experiences, while Communications owns provider-neutral evidence identity and
history.
