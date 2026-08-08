---
chunk_id: 112-doc-docs-part-003
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 24
generated_by: code-wiki-ru
---

# 112-doc-docs-part-003 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `24`

## Резюме

Создать страницу `operations/documentation-map.md` — обзорную карту документации русской Obsidian‑wiki проекта Макошь. Карта перечисляет все задокументированные домены из предоставленного пакета исходников, кратко описывает их обязанности и границы, даёт перекрёстные ссылки и фиксирует состояние документации. Это позволит ориентироваться в структуре wiki и понимать связи между доменами.

## Предложенные страницы

#### `operations/documentation-map.md`

```markdown
# Карта документации Макошь

Эта страница представляет карту документации проекта Макошь —
персональной системы памяти. Здесь перечислены все задокументированные домены,
их основные обязанности и взаимосвязи, а также ссылки на соответствующие
страницы wiki.

## Введение

Макошь — локальная система персональной памяти, построенная вокруг
концепции событийного ядра (Event Sourcing) и графа знаний. Документация
разделена по доменам, каждый из которых отвечает за определённую часть
предметной области.

Текущий комплект документации включает следующие домены:

- ``Граф знаний` (`domains/graph/README`)`
- ``Заметки` (`domains/notes/README`)`
- ``Обязательства` (`domains/obligations/README`)`
- ``Организации` (`domains/organizations/README`)`
- ``Персоны (Persona Intelligence)` (`domains/persons/README`)`
- ``Проекты` (`domains/projects/README`)`
- ``Отношения (Relationships)` (`domains/relationships/README`)`
- ``Инбокс проверки (Review)` (`domains/review/README`)`
- ``Signal Hub (шина сигналов)` (`domains/signal-hub/README`)`

Каждый домен обладает чёткими границами: он владеет определёнными сущностями и
не вторгается в зону ответственности других доменов. Движки (Timeline, Memory,
Trust, Risk, Search) являются общими и используются доменами через
проекции, а не через прямое владение.

## Обзор доменов

### ``Граф знаний` (`domains/graph/README`)`

Хранит долговременные связи между сущностями мира Макошь с указанием
происхождения, уверенности и состояния проверки. Является основой для памяти и
контекста, учитывающей отношения.

- Первоклассные сущности: Persona, Organization, Project, Document, Communication, Event, Task, Decision, Obligation, Location, ChannelAccount, Attachment.
- Отношения — это самостоятельные записи, а не анонимные рёбра.
- Поддерживает нечёткую идентификацию и разрешение конфликтов.
- Каждое выведенное отношение должно ссылаться на доказательства.
- Граф — это граница проекции; поиск, Timeline, Trust, Risk и Memory используют граф, но не владеют источником истины.

### ``Заметки` (`domains/notes/README`)`

Лёгкие артефакты захвата: заметки владельца, встреч, быстрые наблюдения, черновики. Не являются самостоятельным доменом первого класса, пока это не утвердит отдельный ADR.

- Заметки могут становиться доказательствами для Knowledge, Tasks, Decisions, Obligations, Projects, Personas или Organizations после проверки и связывания.
- В настоящее время интерфейс содержит поверхность «Заметки», но на бэкенде выделенного модуля notes нет; документация рассматривает лёгкие заметки как документоподобные артефакты.

### ``Обязательства` (`domains/obligations/README`)`

Обязательства — это обещания, обязанности или ответственность, подтверждённые доказательствами. В отличие от задач, обязательство объясняет, почему что-то может потребоваться сделать.

- Владеет записями обязательств, обязанными сторонами, бенефициарами, сроками, состоянием выполнения, связанными задачами и рисками.
- Не владеет каждой задачей или статусом её жизненного цикла.
- Реализация на бэкенде включает миграции SQL, модуль `backend/src/domains/obligations/*`, API-маршруты для получения и проверки обязательств.
- Обязательства из коммуникаций, встреч, событий календаря и ручного ввода проходят этап предложения и проверки.
- Проецируются в граф знаний для поддерживаемых типов сущностей.

### ``Организации` (`domains/organizations/README`)`

Организации — это первоклассные якоря памяти для компаний, учреждений, агентств и других коллективных субъектов.

- Владеет идентичностью, доменами, идентификаторами, отношениями к Персонам и Проектам, порталами, процедурами, плейбуками и памятью об организации.
- Не владеет идентичностью Персон, жизненным циклом Проектов или глобальными движками.
- Документация включает: ``API` (`domains/organizations/api`)`, ``Архитектуру` (`domains/organizations/architecture`)`, ``Модель данных` (`domains/organizations/data-model`)`, ``Спецификацию` (`domains/organizations/spec`)`.
- Реализация на бэкенде охватывает модули `core`, `memory`, `workflows`, `finance`, `enrichment`, `health`, `investigator`, `api` и миграции 0038–0043.

### ``Персоны (Persona Intelligence)` (`domains/persons/README`)`

Домен персон (Persona Intelligence) формирует слой понимания людей, их идентичностей, отношений и контекста. **Это не CRM, не адресная книга и не менеджер контактов.**

- Корневая сущность — **Persona** с типами: `human`, `ai_agent`, `organization_proxy`, `system`.
- Ровно одна Персона владельца (`is_self: true`).
- Отношения — первоклассные записи, а не поля на Персоне.
- Память Персоны — структурированные, подтверждённые доказательствами записи (факты, знания, предпочтения, карточки памяти, конфликты).
- Досье (Dossier) — генерируемая модель чтения со ссылками на источники.
- Persona Intelligence объединяет ранее разрозненные концепты: communication fingerprint, trust analytics, health status, watchlist, investigator, analytics.
- Документация включает: ``Архитектуру` (`domains/persons/architecture`)`, ``Модель данных` (`domains/persons/data-model`)`, ``API (совместимость)` (`domains/persons/api`)`, ``Статус рефакторинга` (`domains/persons/status`)`, ``Блокеры` (`domains/persons/blockers`)`, ``Спецификацию` (`domains/persons/spec`)`.

### ``Проекты` (`domains/projects/README`)`

Проекты — это ограниченные контексты работы, связывающие коммуникации, документы, задачи, решения, обязательства, Персон, Организации и события.

- Макошь не является инструментом управления проектами.
- Владеет идентичностью проекта, целями, состоянием, пакетом контекста, связями с сущностями и решениями.
- Отличается от Организации (контекст работы vs. коллективный субъект) и от Задачи (контекст vs. конкретное действие).
- Контекстный пакет проекта — это производная модель чтения.
- Реализация включает модули `core`, `link_reviews`, миграции 0013–0014 и API.

### ``Отношения` (`domains/relationships/README`)`

Отношения — это первоклассные записи, связывающие сущности мира Макошь.

- Владеет долговременными записями отношений, типами, оценками доверия и силы, уверенностью, доказательствами, периодом действия.
- Не владеет обходом графа, рендерингом Timeline, вычислением доверия или рисков — это зона движков.
- Поддерживаемые типы сущностей: Persona, Organization, Project, Communication, Document, Task, Event, Decision, Obligation, Knowledge.
- Реализация включает миграции 0060–0061, 0068, модуль `backend/src/domains/relationships/*`, API и тесты.
- Совместимые адаптеры для `person_roles`, `organization_contact_links`, `task_relations` и связей проектов.

### ``Инбокс проверки (Review)` (`domains/review/README`)`

Долговременный инбокс для материалов, требующих сортировки, одобрения или отклонения владельцем, прежде чем они станут принятой истиной домена.

- Владеет элементами инбокса, жизненным циклом, ссылками на доказательства, целями продвижения.
- Не владеет окончательной истиной других доменов.
- Типы элементов: кандидаты идентичности, связей проектов, противоречий, задач, обязательств, решений, отношений, проектов и знаний.
- Логика продвижения находится в `backend/src/workflows/review_promotion`, а не в самом домене Review.

### ``Signal Hub` (`domains/signal-hub/README`)`

Шина сигналов — это системный control plane для внешних и синтетических источников сигналов. Это не интерфейс мессенджера и не папка интеграций.

- Управляет реестром источников, подключений, возможностей, состоянием выполнения, здоровьем, профилями, политиками mute/pause и восстановлением.
- Все сигналы (Email, Telegram, WhatsApp, GitHub, Browser, RSS, Календарь, Файловая система, Home Assistant и т.д.) входят через одну управляемую границу.
- Использует событийный каркас (Event Backbone) с хранением в PostgreSQL `event_log` и доставкой через NATS JetStream.
- API — контракт-first на Protobuf + ConnectRPC.
- Документация включает: ``Архитектуру` (`domains/signal-hub/architecture`)`, ``Модель данных` (`domains/signal-hub/data-model`)`, ``API` (`domains/signal-hub/api`)`, ``Блокеры` (`domains/signal-hub/blockers`)`, ``Фикстуры и восстановление` (`domains/signal-hub/fixtures`)`.

## Связи между доменами

- **Граф знаний** потребляется почти всеми доменами для навигации по отношениям, но не владеет источником истины отношений — это зона домена **Relationships**.
- **Обязательства** тесно связаны с **Задачами** (Tasks, документированы отдельно — не включены в данный чанк), **Коммуникациями**, **Событиями** и **Проектами**. Они проецируются в граф.
- **Организации** связаны с **Персонами** через отношения, а не через встроенные поля.
- **Персоны** — центральный домен для представления субъектов. Их отношения реализуются доменом **Relationships**. Досье и выводы Persona Intelligence строятся на основе данных из Графа, Timeline, Communications и т.д.
- **Проекты** связывают множество сущностей: Персон, Организации, Задачи, Обязательства, Документы. Связи проходят через проверку (Review).
- **Review** — перекрёстный домен, обслуживающий инбокс для проверки кандидатов из разных доменов.
- **Signal Hub** — точка входа для внешних сигналов; после акцепта сигналы превращаются в события доменов (Communications, Radar, и далее).

Все домены используют общие движки (Timeline, Memory, Trust, Risk, Search), но не владеют их реализацией.

## Состояние документации

Документация большинства доменов обновлена и соответствует целевому видению архитектуры. Некоторые страницы (например, для Persons) содержат подробные статусы миграции от старой модели «контактов» к новой Persona Intelligence. Домен **Заметки** пока не имеет статуса полноценного домена и документирован как переходный.

Актуальный список реализованных и запланированных изменений можно найти в ADR и плане выравнивания реализации (Implementation Alignment Plan, не включены в данный чанк).

## Примечания

- Данная карта основана исключительно на исходных файлах документации, включённых в контекст чанка `112-doc-docs-part-003`.
- Некоторые упомянутые страницы (например, `domains/tasks/README`) не были включены в контекст и не могут быть подтверждены из него; они указаны в предположении, что соответствующие страницы существуют в wiki.
- Для углублённого изучения каждого домена переходите по ссылкам выше.
```

## Покрытие источников

| Исходный файл | Использованные факты |
|---|---|
| `docs/domains/graph/README.md` | Назначение графа знаний, список сущностей, модель отношений, разрешение идентичности, требования к происхождению, граница движка. |
| `docs/domains/notes/README.md` | Определение заметок как лёгких артефактов, текущий статус «не домен первого класса», план миграции. |
| `docs/domains/obligations/README.md` | Обязанности домена, модель обязательства, источники, реализация на бэкенде, маршруты API, план миграции. |
| `docs/domains/organizations/README.md` | Назначение домена организаций, границы, использование движков, навигация на api, architecture, data-model. |
| `docs/domains/organizations/api.md` | Перечень маршрутов API (core, identities, departments, timeline, portals, finance, enrichment, risk, dossier). |
| `docs/domains/organizations/architecture.md` | Позиция домена, модули (`core`, `memory`, `workflows`, `finance`, `enrichment`, `health`, `investigator`, `api`), потоки данных, список ADR. |
| `docs/domains/organizations/data-model.md` | Структура таблицы `organizations`, граница доверия/отношений, другие таблицы, миграции 0038–0043. |
| `docs/domains/organizations/spec.md` | Обязанности домена, граница отношений и памяти, текущее состояние реализации, план миграции. |
| `docs/domains/persons/README.md` | Видение домена (Persona Intelligence), core model, типы Persona, Self Persona, Relationship-first, Memory-first, Dossier, Persona Intelligence, навигация. |
| `docs/domains/persons/api.md` | Правила интерпретации существующих API‑концептов (person → Persona и т.д.), граница документирования. |
| `docs/domains/persons/architecture.md` | Архитектурная позиция, границы, Persona, Self Persona, Identity Resolution, Relationship First, Memory First, Timeline Engine, Dossier, Persona Intelligence, совместимость с текущим бэкендом, источник истины, список ADR. |
| `docs/domains/persons/blockers.md` | Текущие блокеры и не-блокеры миграции к целевой архитектуре Persona. |
| `docs/domains/persons/data-model.md` | Целевая логическая модель: Persona, PersonaType, PersonaIdentity, IdentityResolutionCandidate, Relationship, PersonaMemory, Dated Events, Communication, Dossier, таблица совместимости. |
| `docs/domains/persons/spec.md` | Обязанности домена, представление Persona, типы отношений, поддержка merge/split. |
| `docs/domains/persons/status.md` | Статус документации и реализации, срезы миграции, удалённая старая система оценок. |
| `docs/domains/projects/README.md` | Обязанности домена проектов, отличие от организаций и задач, контекстный пакет, реализация, план миграции. |
| `docs/domains/relationships/README.md` | Назначение, обязанности, отношения Persona, доказательства, отличие от графового ребра, текущая реализация, план миграции. |
| `docs/domains/review/README.md` | Назначение, обязанности, реализация, граничное правило. |
| `docs/domains/signal-hub/README.md` | Назначение, позиция, ключевые инварианты, контекст репозитория, текущая реализация. |
| `docs/domains/signal-hub/api.md` | Контракт ConnectRPC сервиса, семантика команд, realtime, авторизация. |
| `docs/domains/signal-hub/architecture.md` | Цель, высокоуровневый поток, владение слоями, событийный каркас, разделение Event Store/Transport, семейства субъектов, конверт события, управление сигналами, модель выполнения, UI реального времени, граница ConnectRPC, граница безопасности. |
| `docs/domains/signal-hub/blockers.md` | Технические, архитектурные и продуктовые блокеры. |
| `docs/domains/signal-hub/data-model.md` | Сущности данных (SignalSource, SignalConnection, SignalCapability, SignalRuntime, SignalHealth, SignalPolicy, SignalProfile, SignalReplayRequest), модель событий, принципы хранения, предлагаемые таблицы, идемпотентность. |
| `docs/domains/signal-hub/fixtures.md` | Типы фикстур, жёсткие правила для системной фикстуры, расположение, пример восстановительной фикстуры, семантика загрузчика, тестовые фикстуры, режим fixture. |

## Исходные файлы

- [`docs/domains/graph/README.md`](../../../domains/graph/README.md)
- [`docs/domains/notes/README.md`](../../../domains/notes/README.md)
- [`docs/domains/obligations/README.md`](../../../domains/obligations/README.md)
- [`docs/domains/organizations/README.md`](../../../domains/organizations/README.md)
- [`docs/domains/organizations/api.md`](../../../domains/organizations/api.md)
- [`docs/domains/organizations/architecture.md`](../../../domains/organizations/architecture.md)
- [`docs/domains/organizations/data-model.md`](../../../domains/organizations/data-model.md)
- [`docs/domains/organizations/spec.md`](../../../domains/organizations/spec.md)
- [`docs/domains/persons/README.md`](../../../domains/persons/README.md)
- [`docs/domains/persons/api.md`](../../../domains/persons/api.md)
- [`docs/domains/persons/architecture.md`](../../../domains/persons/architecture.md)
- [`docs/domains/persons/blockers.md`](../../../domains/persons/blockers.md)
- [`docs/domains/persons/data-model.md`](../../../domains/persons/data-model.md)
- [`docs/domains/persons/spec.md`](../../../domains/persons/spec.md)
- [`docs/domains/persons/status.md`](../../../domains/persons/status.md)
- [`docs/domains/projects/README.md`](../../../domains/projects/README.md)
- [`docs/domains/relationships/README.md`](../../../domains/relationships/README.md)
- [`docs/domains/review/README.md`](../../../domains/review/README.md)
- [`docs/domains/signal-hub/README.md`](../../../domains/signal-hub/README.md)
- [`docs/domains/signal-hub/api.md`](../../../domains/signal-hub/api.md)
- [`docs/domains/signal-hub/architecture.md`](../../../domains/signal-hub/architecture.md)
- [`docs/domains/signal-hub/blockers.md`](../../../domains/signal-hub/blockers.md)
- [`docs/domains/signal-hub/data-model.md`](../../../domains/signal-hub/data-model.md)
- [`docs/domains/signal-hub/fixtures.md`](../../../domains/signal-hub/fixtures.md)

## Кандидаты на drift

Из предоставленного контекста расхождений (drift) между кодом, документацией и ADR не видно. Все исходные файлы являются согласованными документационными пакетами, выровненными по текущей структуре репозитория. В нескольких местах упоминаются страницы (например, `domains/tasks/README`, Implementation Alignment Plan), которые не включены в данный чанк, но их отсутствие в контексте не является drift — это просто неполнота переданного набора исходников.
