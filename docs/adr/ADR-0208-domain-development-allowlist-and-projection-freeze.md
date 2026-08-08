# ADR-0208: Allowlist разработки доменов и запрет проекций

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0207: Канонический реестр бизнес-доменов Макошь](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

ADR-0207 фиксирует полный канонический реестр из тринадцати business domains.
Одновременная реализация всего реестра вернёт clean-room проект к широкому
фронту работ, преждевременным abstractions и связанным полуреализованным
модулям.

Первые owner slices после открытия `first_owner_v1` должны доказать module
isolation, contracts, storage ownership, communication flow и client boundary
на минимальном полезном наборе доменов. `kernel_recovery_only_v1` не содержит
ни одного owner; текущий executable inventory равен нулю. Остальные домены
сохраняются в архитектурном реестре, но не должны появляться в production code
до отдельного решения.

Производные проекции также откладываются: они создают дополнительные storage,
event-consumer и consistency boundaries до стабилизации canonical owners.

## Решение

### Разрешённые домены

После открытия `first_owner_v1` разработка production implementation разрешена
только для следующих семи business domains:

| `module_id` | Каноническое имя |
|---|---|
| `communications` | Коммуникации |
| `contacts` | Контакты |
| `organizations` | Организации |
| `tasks` | Задачи |
| `calendar` | Календарь |
| `documents` | Документы |
| `ai` | AI |

Для этих доменов разрешено создавать public contracts, runtime
implementations, owned schema, client API, frontend surfaces, tests и
необходимые workflows при соблюдении ADR-0200—ADR-0207.

### Заблокированные домены

Следующие зарегистрированные ADR-0207 домены имеют статус `blocked`:

| `module_id` | Каноническое имя |
|---|---|
| `relationships` | Отношения |
| `projects` | Проекты |
| `obligations` | Обязательства |
| `decisions` | Решения |
| `knowledge` | Знания |
| `review` | Обзор |

Для заблокированного домена запрещено создавать или переносить из legacy:

- production package или runtime;
- public application contract, command, query или event;
- PostgreSQL schema, table, migration, role или seed;
- NATS subject, consumer, outbox/inbox handler или capability;
- Gateway service, HTTP route или client transport;
- frontend product surface, store или generated client;
- workflow, который реализует business behavior заблокированного домена;
- fixture, fake implementation или placeholder, изображающий будущий domain.

Обсуждение терминов и создание будущего superseding ADR разрешены. Они не
считаются разрешением на executable implementation.

### Запрет обхода allowlist

Разрешённый домен не может временно присвоить ответственность
заблокированного домена. В частности:

- Contacts и Organizations не реализуют Relationships;
- Tasks не реализует Projects, Obligations или Decisions;
- Communications не реализует Review или Knowledge;
- Documents не реализует Knowledge;
- Calendar не реализует Projects или Obligations;
- generic `metadata`, untyped JSON и feature flags не используются как скрытый
  контракт заблокированного домена.

Public identifier другого разрешённого owner допустим как ссылка, но не даёт
права копировать его canonical state или policy.

### Projection freeze

Разработка любых rebuildable product projections и projection runtimes
запрещена. В freeze входят как минимум:

- Graph;
- Timeline;
- Search;
- Context;
- cross-domain materialized views;
- event-built read models;
- vector и embedding indexes;
- dossiers, context packs и агрегированные memory views;
- сохранённые cross-domain AI summaries, classifications и scores.

Запрещены projection packages, schemas/tables, consumers, rebuild jobs,
frontend projection surfaces и временные in-memory replacements.

Запрет не распространяется на:

- canonical owner state разрешённого домена;
- обычные PostgreSQL indexes над owned canonical tables;
- request-time read composition без сохранения derived state;
- provider-owned operational state integration-плагина по ADR-0204.
- AI run, его provenance и typed result внутри owned state домена AI, если
  результат не становится projection или business truth другого домена.

Provider operational state не становится канонической product projection и
не может использоваться как обход запрета Graph, Timeline, Search или Context.

### Разрешённая supporting implementation

Этот ADR не блокирует разработку Kernel, Gateway, platform capabilities,
PostgreSQL/PgBouncer/NATS infrastructure, vault, blob storage, test
infrastructure и architecture guards.

Integration-плагины разрешены только в рамках operational experiences и
neutral evidence flow Communications по ADR-0204. Они не могут создавать
state или frontend surface заблокированного business domain.

Model runtimes и remote model providers разрешены как adapters/integrations
домена AI. Они не получают ownership данных других доменов и не могут
создавать запрещённые projections.

Workflow разрешён только тогда, когда все его business owners входят в
allowlist и workflow не создаёт projection или семантику заблокированного
домена.

### Разблокировка

Разблокировать домен или projection можно только новым принятым ADR, который:

1. называет конкретный разблокируемый owner или projection;
2. определяет его ownership и public contract boundary;
3. объясняет, почему текущих семи разрешённых доменов недостаточно;
4. задаёт failure isolation, storage и privacy constraints;
5. добавляет executable allowlist/guard change в том же implementation slice.

Feature flag, скрытый package, experimental directory, legacy copy или
отключённый route не считаются разблокировкой.

## Отклонённые варианты

### Создать пустые crates всех тринадцати доменов заранее

Отклонено: пустой scaffold создаёт ложную архитектурную завершённость,
закрепляет непроверенные contracts и расширяет build graph.

### Разрешить проекции без frontend

Отклонено: даже невидимая projection требует schema, consumers, rebuild и
consistency policy и увеличивает сложность первой реализации.

### Использовать один из разрешённых доменов как временного владельца

Отклонено: временный owner почти неизбежно становится постоянным coupling и
нарушает канонический реестр ADR-0207.

## Последствия

Положительные:

- первые owner slices ограничены семью владельцами business state, а текущий
  recovery-only slice не содержит owners;
- package и runtime graph остаётся малым;
- canonical contracts стабилизируются до появления derived models;
- невозможно незаметно вернуть legacy Graph или broad context layer;
- критерий допустимости нового production package становится бинарным.

Отрицательные:

- функции Relationships, Projects, Obligations, Decisions, Knowledge и Review
  отсутствуют до отдельной разблокировки;
- глобальный поиск, timeline, graph и context views отсутствуют;
- часть прежнего frontend не переносится в clean-room систему на первом этапе.

## Проверка решения

Статическая часть решения закреплена в
[`backend/architecture/policy.json`](../../backend/architecture/policy.json) и проверяется
`make -C backend architecture-check`: allowlist/blocklist, package roles, production
paths, standalone SQL ownership и projection packages имеют negative
self-tests. ADR остаётся `Не реализовано`, пока отсутствуют новый API/frontend
guard, `ModuleDescriptorV1`/schema-role evidence и runtime новой системы.

До признания решения реализованным должны существовать:

- executable allowlist с семью разрешёнными domain `module_id`;
- package-role guard, запрещающий production packages шести blocked domains;
- schema guard, запрещающий их tables, roles и migrations;
- API/frontend guard, запрещающий blocked service names и surfaces;
- projection guard, запрещающий projection roles, packages, schemas,
  consumers и materialized views;
- negative fixtures, доказывающие отказ guard для каждого blocked domain и
  каждого класса projection;
- absence check, не находящий executable legacy copies за пределами
  `references/`.

До появления этих executable evidence поле `Состояние реализации` остаётся
`Не реализовано`.
