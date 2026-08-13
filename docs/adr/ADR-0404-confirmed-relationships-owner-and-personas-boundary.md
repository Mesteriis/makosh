# ADR-0404: Confirmed Relationships owner и Personas boundary

- Статус: принято
- Дата: 2026-08-13
- Состояние реализации: authority принято; runtime и production admission не
  реализованы
- Заменяет: только часть ADR-0208, блокирующую разработку домена
  `relationships`
- Связанные решения: ADR-0200, ADR-0204, ADR-0207, ADR-0208, ADR-0220,
  ADR-0402

## Контекст

ADR-0207 назначает Relationships каноническим владельцем подтверждённых
отношений между субъектами, но ADR-0208 намеренно оставил домен заблокированным
до стабилизации первых owner slices. Persons, Organizations и их typed client
границы теперь определены. Personas UI уже содержит read-only визуальный
компонент, но не имеет источника истины и поэтому обязан показывать unavailable
state.

Graph остаётся отдельной rebuildable projection и не может стать владельцем
отношений. Persons и Organizations также не могут хранить отношения в своих
таблицах или принимать неограниченный generic metadata вместо публичного
контракта.

## Решение

### Development authority

`relationships` переносится из `domains.blocked` в
`domains.developmentAllowlist`. Остальные заблокированные домены и все
projection freezes остаются без изменений. Это разрешает repository-local
contracts, core, persistence, runtime, assembly, managed conformance и frontend
adapter, но само по себе не меняет `implementation.currentSlice`, production
package inventory или release admission.

### Каноническая модель

Relationships владеет только подтверждённым relationship fact:

- owner-scoped stable Relationship ID;
- двумя typed public participant references, ограниченными Person и
  Organization;
- closed relationship type;
- temporal validity interval;
- confirmed/ended lifecycle и checked revision;
- bounded public evidence references и evidence digests.

Participant reference содержит только public owner kind и 16-byte owner ID.
Relationships не читает Persons/Organizations storage, не копирует profile и не
делает foreign SQL/FK. Направленность является частью relationship type;
симметричные типы канонизируют participant order.

Создание relationship является явным подтверждением пользователя. Suggested,
system-accepted, trust, confidence и scoring не являются canonical state этого
домена. Будущий proposal должен проходить отдельный Review/promotion contour.

### Evidence и temporal validity

Evidence reference ограничена public source-owner ID, source-record ID,
revision, observed time и SHA-256 digest. Raw provider payload, private locator,
credential, communication body и arbitrary map запрещены.

`valid_from` обязателен. Optional `valid_until` строго позже `valid_from`.
Завершение relationship закрывает lifecycle, но сохраняет temporal truth и
evidence. Exact operation replay возвращает сохранённый response; изменённые
bytes конфликтуют.

### Client и public event

Typed client предоставляет Create, UpdateValidity, End, Reactivate,
AddEvidence, RemoveEvidence, Get, ListForParticipant и ListEvidence. Query
owner всегда берётся из authenticated outer context; conflicting payload owner
отклоняется. Pagination bounded и использует last-returned public ID как
exclusive cursor.

Sanitized RelationshipChanged event содержит только event/Relationship/owner
IDs, participant kinds/IDs, type, revision, lifecycle и temporal interval.
Evidence locator, evidence record and private owner data в public event не
попадают.

### Personas activation

После production admission Personas Relationships section использует только
generated Relationships client и отображает bounded owner-local relationships
выбранной Person. Это UI над canonical Relationships query, а не durable Graph
projection. Timeline, trust, dossier, identity scoring и global Graph остаются
замороженными.

## Отклонённые варианты

### Хранить отношения в Persons или Organizations

Отклонено: это создаёт двух владельцев одного факта и связывает независимые
lifecycle.

### Сделать Graph источником истины

Отклонено: Graph должен полностью перестраиваться из owner events и не владеет
canonical relationship facts.

### Принимать generic kind/metadata/confidence

Отклонено: произвольные строки и JSON скрывают schema evolution, proposal
state и provider-private content.

## Проверка решения

- executable policy разрешает `relationships` development source и продолжает
  блокировать Projects, Obligations, Decisions и projections;
- architecture guard требует ровно пять Relationships packages и три public
  capabilities, но production slice переключается только последовательно;
- storage conformance доказывает FORCE RLS, replay, temporal conflicts,
  restart и cross-owner denial;
- actual managed ClientRpc доказывает create/evidence/end/reactivate,
  pagination, public-event privacy и restart equality;
- Personas adapter активируется только вместе с generated Relationships
  client; REST alias и Graph storage не создаются.
