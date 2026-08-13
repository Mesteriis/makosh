# ADR-0409: Search, Timeline and Graph rebuildable projection boundary

Статус: Принято
Дата: 2026-08-13

## Контекст

Canonical owners уже публикуют typed lifecycle/owner events, но продукту нужны
три cross-owner read models: глобальный поиск, хронологическая лента и граф
подтверждённых публичных связей. ADR-0208 запрещал эти projections до
появления owner contracts, RLS, managed runtime и executable admission.

Проекция не может стать вторым владельцем Person, Relationship, Project,
Task, Obligation, Decision, Calendar event, Document или Knowledge note. Она
также не может получить raw provider payload, credential, private locator,
generic JSON или прямой SQL-доступ к таблицам владельцев.

## Решение

Разблокировать ровно три роли `projection` и реализовать их последовательно:

1. `search` — owner-local структурный индекс canonical public IDs, owner kind,
   lifecycle state и keyed bounded search tokens, когда их предоставляет
   explicit typed projection handoff;
2. `timeline` — owner-local chronological projection sanitized lifecycle
   events с provenance, source revision и deterministic order;
3. `graph` — owner-local nodes/edges только из public owner IDs и
   подтверждённых Relationships/lineage/reference events.

Каждая проекция имеет отдельные API/core/persistence/runtime/assembly,
отдельный Storage namespace, read-only generated ClientRpc и собственный
rebuild cursor. Между проекциями нет SQL/FK и ни одна не читает owner tables.
Runtime принимает только exact canonical owner envelopes через granted durable
routes, persists-before-ACK и публикует no business command.

Search не сохраняет plaintext private content. Private searchable values
преобразуются только внутри target-bound handoff/runtime в keyed token digests
через `OwnerDerivedProjectionKey`; API возвращает public owner/entity identity
и match class, а отображаемый canonical объект загружается его owner client.
Timeline хранит только поля, разрешённые исходным sanitized event. Graph хранит
только подтверждённые public references; inference/confidence/risk запрещены.

## Rebuild и удаление

Каждый вход exact-byte idempotent. Изменённые bytes под тем же event ID
конфликтуют. Source revision не регрессирует. Полный rebuild создаёт новую
generation рядом со старой, проверяет cursor/completeness, затем одним CAS
переключает active generation. Частичный rebuild не виден client.

Owner deletion/tombstone является обязательным typed input: Search удаляет
tokens/document, Timeline сохраняет только bounded tombstone entry либо
удаляет owner history согласно source contract, Graph удаляет incident edges
и orphan nodes. Повторное удаление идемпотентно; restart не воскрешает данные.

## Client и privacy boundary

Client routes read-only: Search Query/GetStatus, Timeline List/GetStatus,
Graph Neighbors/Path/GetStatus. Authenticated outer owner заменяет пустой
payload owner и отвергает конфликтующий непустой. Cursor — последний реально
возвращённый элемент, поэтому pagination не имеет gaps/duplicates.

Durable tables используют ENABLE+FORCE RLS, transaction-local owner context и
NOBYPASSRLS conformance. Inbox/outbox/rebuild state не содержит credentials,
provider locators, raw payloads или arbitrary metadata. Ошибки, supervisor
diagnostics, stdout/stderr и client responses проходят sentinel negatives.

## Admission

Task 24 добавляет ровно пятнадцать production-role packages и три managed
modules. Каждая проекция имеет ровно client, owner-event consumer и storage
capability; consumed owner-event contracts принадлежат canonical owners.
Admission требует actual full rebuild, restart,
owner deletion propagation, NOBYPASSRLS, release compiler, generated clients
и отсутствие REST fallback.

`context`, Memory, consistency, risk и Signal Hub остаются заблокированы до
Task 25. Zoom, Telemost и OmniRoute остаются вне этого решения.

## Отклонено

- единая Search/Timeline/Graph база или runtime;
- Graph как canonical owner relationships;
- plaintext cross-owner search index;
- generic event/JSON ingestion или direct owner SQL;
- проекция, создающая команды в canonical owners;
- frontend route до executable backend admission.
