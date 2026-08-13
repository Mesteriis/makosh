# ADR-0405: Projects owner, expected outcomes и product boundary

- Статус: принято
- Дата: 2026-08-13
- Состояние реализации: authority принято; runtime и production admission не реализованы
- Заменяет: только часть ADR-0208, блокирующую разработку домена `projects`
- Связанные решения: ADR-0200, ADR-0204, ADR-0207, ADR-0208, ADR-0220,
  ADR-0402, ADR-0404

## Контекст

ADR-0207 назначает Projects каноническим владельцем project state, expected
outcomes и lifecycle, но ADR-0208 оставил домен заблокированным до
стабилизации базовых владельцев. Исторический frontend Projects surface ходит
в отсутствующий REST API и смешивает Project с сообщениями, документами,
Personas, Timeline и Graph projections.

Projects не может копировать canonical state Persons, Organizations,
Relationships, Tasks, Documents или Calendar. Эти владельцы могут быть
связаны только typed public references. Search, Timeline и Graph остаются
отдельными rebuildable projections.

## Решение

### Development authority

`projects` переносится из `domains.blocked` в
`domains.developmentAllowlist`. Obligations, Decisions и все projection
freezes остаются без изменений. Это разрешает repository-local contracts,
core, persistence, runtime, assembly, managed conformance и frontend adapter,
но не меняет `implementation.currentSlice`, production package inventory или
release admission.

### Каноническая модель

Projects владеет только:

- owner-scoped stable Project ID;
- bounded name и optional description;
- closed Planning/Active/OnHold/Completed/Archived lifecycle;
- optional start и target timestamps;
- checked project revision;
- ordered expected outcomes с closed Pending/Achieved/Missed/Cancelled state;
- typed public references на Person, Organization, Relationship, Task,
  Document и CalendarEvent.

Typed reference содержит только owner kind, 16-byte public ID и bounded public
label. Projects не читает чужую storage, не создаёт cross-owner SQL/FK и не
копирует profile, document bytes, task state, calendar payload, communication
body или provider-private data.

### Expected outcomes и lifecycle

Expected outcome имеет stable owner/project-scoped ID, bounded title и
optional description, optional target timestamp, closed state и checked
revision. Project completion требует хотя бы один outcome и запрещает Pending
outcomes; Archive разрешён только после Completed. Reactivate возвращает
Completed/Archived project в Active, не переписывая outcome history.

Create, Update, SetState, AddOutcome, UpdateOutcome, SetOutcomeState,
RemoveOutcome, AddReference и RemoveReference являются exact replay-safe
mutations с nonzero operation ID. Changed bytes conflict. Get, List,
ListOutcomes и ListReferences используют authenticated outer owner и bounded
last-returned exclusive cursors.

### Public event и frontend

Sanitized ProjectChanged event содержит только event/Project/owner IDs,
revision, lifecycle, schedule interval и occurrence time. Project name,
description, outcome text, reference label и foreign IDs не публикуются.

Generated Projects client заменяет исторический REST scaffold. Активный UI
показывает project metadata, lifecycle, expected outcomes и typed references.
Messages, documents, Personas, Timeline and Graph aggregates не имитируются и
не восстанавливаются как Project truth.

## Отклонённые варианты

### Хранить project state в Tasks или Graph

Отклонено: это создаёт второго владельца canonical Project lifecycle и делает
rebuildable projection источником истины.

### Копировать owner data или принимать generic JSON

Отклонено: это скрывает schema evolution, нарушает owner deletion и позволяет
private provider data пересечь boundary.

### Вычислять progress percent как canonical field

Отклонено: progress является projection из closed outcome state, а не
независимой изменяемой истиной.

## Проверка решения

- executable policy разрешает только Projects development source и продолжает
  блокировать Obligations, Decisions и projections;
- architecture guard требует ровно пять Projects packages и три public
  capabilities, но production slice переключается только последовательно;
- storage conformance доказывает FORCE RLS, exact replay, lifecycle/outcome
  conflicts, restart и cross-owner denial;
- actual managed ClientRpc доказывает project/outcome/reference lifecycle,
  pagination, sanitized outbox и restart equality;
- generated Projects client заменяет REST, не добавляя projection clients.
