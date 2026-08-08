# ADR-0316: Communications saved-search derived projection

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: implemented
- Связанные решения: ADR-0204, ADR-0205, ADR-0212, ADR-0213, ADR-0223,
  ADR-0240, ADR-0253, ADR-0254, ADR-0281, ADR-0282, ADR-0313

## Контекст

`communications.search.index.v1` уже выполняет exact-token поиск по
owner-local keyed-digest projection. Текст запроса существует только внутри
одного runtime request и не сохраняется. Historical saved searches смешивали
Mail folders, provider state, workflow state и plaintext query в общей
Communications таблице.

Clean-room capability должна сохранить полезный use case «назвать, сохранить,
применить, заменить и удалить поиск», но не вернуть Mail-specific smart
folders и не ослабить private-content boundary ADR-0254.

## Решение

### Owner и build units

Saved search остаётся derived projection домена `communications`.

- `makosh-communications-saved-query-api` — отдельная contract build unit;
- `makosh-communications-domain` — owner-local validation и normalization;
- `makosh-communications-persistence` — owner-local PostgreSQL projection;
- `makosh-communications-runtime` — transient key lease, digest construction,
  client dispatch и execution;
- `makosh-communications-assembly` — существующая Communications assembly,
  без отдельного process или integration.

Новая integration, workflow, provider branch или independently managed runtime
не создаётся. Build unit не объявляется отдельным domain owner.

### Exact capability

`communications.saved-search.v1` предоставляет один generated
`client_rpc` contract:

```text
List
Create
Replace
Delete
Execute
```

Request использует typed `oneof`; generic payload, map, SQL predicate и legacy
filter expression запрещены. `Create` принимает client-generated 16-byte
`saved_search_id`, что делает повтор exact request идемпотентным. `Replace` и
`Delete` требуют exact `expected_revision`.

### Persisted definition

Owner-local projection хранит:

- opaque 16-byte saved-search ID;
- bounded display name and optional description;
- optional canonical Communications account ID;
- ordered unique keyed token digests;
- exact search-key schema revision;
- lifecycle/revision and timestamps;
- mutation audit fingerprint.

Plaintext query, normalized token, provider ID, source cursor, Blob reference,
credential, provider folder/label, workflow/review state и arbitrary metadata
не сохраняются.

`Create`/`Replace` получают текущий `communications.search.index` owner-derived
key lease, нормализуют не более 16 exact tokens и сохраняют только keyed
digests. Key material zeroized runtime adapter after request. `Execute` читает
digests и выполняет существующий derived-index matcher; response содержит
только canonical evidence/message/conversation IDs, timestamp и match count.

Account scope является optional canonical account ID. Persistence сверяет его
с owner-local canonical account projection. Integration account ID или
provider locator contract не принимает.

### Lifecycle и fencing

- active definition имеет monotonically increasing `revision`;
- replace atomically swaps all digests and increments revision;
- delete writes a tombstone, removes digests and increments revision;
- stale expected revision returns typed conflict and does not mutate state;
- list returns only active definitions with bounded keyset pagination;
- execute cursor is bound to saved-search ID, definition revision, account
  scope and digest set, поэтому replace/delete инвалидирует старую страницу;
- deleted ID cannot be silently reused;
- route/grant/runtime revoke fences the whole capability at Core/Kernel;
- search-key schema mismatch makes a definition unavailable until explicit
  replace; implicit plaintext recovery or provider fallback отсутствует.

Mutations are synchronous owner-local client RPC operations. `accepted` is not
used: successful response means the PostgreSQL transaction committed.
Cross-owner event не публикуется, потому что saved-search projection не
является authority другого owner.

### Frontend

Communications owner surface получает отдельные responsibilities:

```text
canonicalCommunicationsSavedSearches
  generated client adapter

useCanonicalCommunicationsSavedSearches
  capability guard, CRUD/execute lifecycle and stale request fencing

canonicalSavedSearchPanelModel
  pure wire-to-view transformation

CanonicalSavedSearchPanel
  presentation and typed intents
```

Controls доступны только при exact bootstrap capability
`communications.saved-search.v1`. UI не может показать сохранённый plaintext
query, потому что backend его не хранит. Replace явно использует текущую
строку canonical search и не подставляет фиктивное значение.

## Gate `communications_saved_search_v1`

Gate становится `implemented` только атомарно при наличии:

1. отдельной contract build unit и exact descriptor/capability;
2. bounded typed list/create/replace/delete/execute requests and responses;
3. no-query-plaintext persistence and owner-derived digest construction;
4. idempotent create, CAS replace/delete and durable tombstone/audit;
5. account-scope validation and cursor revision/scope fencing;
6. authenticated Gateway -> managed Communications runtime conformance;
7. generated frontend adapter/controller/presentation with exact guard;
8. replay/conflict/delete/key-revision/privacy negative tests;
9. architecture/SRP/Cargo/frontend/full relevant gates.

## Rollback

- revoke `communications.saved-search.v1`;
- remove its ClientRpc route while keeping `communications.query.v1`,
  `communications.search.index.v1` and canonical evidence available;
- keep saved definitions/tombstones for a later explicitly admitted runtime;
- never fall back to legacy REST, provider folders or plaintext query storage.
