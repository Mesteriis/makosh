---
chunk_id: 113-doc-docs-part-004
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 113-doc-docs-part-004 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создаётся страница `operations/documentation-map.md` — обзорная карта документации проекта Макошь. Она собирает ссылки на существующие документирующие файлы доменов (`signal-hub`, `tasks`) и движков (`engines`), группирует их по категориям и указывает краткое описание и статус каждого документа, взятые непосредственно из встроенных исходных текстов. Карта служит навигационным указателем для разработчиков и сопровождающих, опираясь только на реально присутствующие в репозитории файлы документации.

## Предложенные страницы

#### `operations/documentation-map.md`

```markdown
# Карта документации Макошь

Статус: обзорный навигационный документ.

Эта страница перечисляет основные документирующие файлы проекта, обнаруженные
в текущем снимке репозитория. Для каждого элемента указан путь от корня
репозитория и краткое описание, взятое из содержимого самого файла.

## Домены (Domains)

### Signal Hub

| Документ | Путь в репозитории | Описание |
|---|---|---|
| Спецификация Signal Hub | `docs/domains/signal-hub/spec.md` | Владение, границы и правила домена источников сигналов. |
| Статус реализации Signal Hub | `docs/domains/signal-hub/status.md` | По состоянию на 2026-06-23 статус `IMPLEMENTATION STARTED`; треки готовности и текущие закрытые/открытые работы. |
| Gap-анализ Signal Hub | `docs/domains/signal-hub/gap-analysis.md` | Сравнение целевого и текущего состояния Signal Hub с перечислением оставшихся пробелов. |
| Модули Signal Hub | `docs/domains/signal-hub/modules.md` | Целевое разбиение на модули бэкенда, платформенных событий, контрактов и фронтенда. |
| Операции Signal Hub | `docs/domains/signal-hub/operations.md` | Контракт управления источниками (mute, pause, replay, профили, проверки здоровья, UI). |
| Тестирование Signal Hub | `docs/domains/signal-hub/testing.md` | Стратегия тестирования без живых провайдеров, фикстурные источники, CI-профиль. |

### Tasks

| Документ | Путь в репозитории | Описание |
|---|---|---|
| Обзор Tasks | `docs/domains/tasks/README.md` | Общее введение в домен задач, границы и навигация. |
| Спецификация Tasks | `docs/domains/tasks/spec.md` | Ответственности, источники задач, жизненный цикл, правила извлечения. |
| API задач | `docs/domains/tasks/api.md` | Текущие совместимые маршруты REST (GET/POST/PUT/DELETE) для ядра, контекста, интеллекта, экспорта и пр. |
| Архитектура Tasks | `docs/domains/tasks/architecture.md` | Позиция домена, модули, слои и доменные правила. |
| Модель данных Tasks | `docs/domains/tasks/data-model.md` | Таблицы, форматы идентификаторов, статусы и граница с обязательствами. |
| Статус реализации Tasks | `docs/domains/tasks/status.md` | 84 % разделов спеки реализовано; детальная таблица реализованных и отложенных возможностей. |

## Движки (Engines)

| Документ | Путь в репозитории | Описание |
|---|---|---|
| Каталог движков | `docs/engines/README.md` | Обзорная страница: определение движка, список спецификаций, признак миграции доменно-локального поведения. |
| Automation Engine | `docs/engines/automation/README.md` | Политики и dry-run для автоматизации, текущая реализация Telegram send automation. |
| Call Intelligence Engine | `docs/engines/call-intelligence/README.md` | Преобразование Call Bundle в структурированные кандидаты: транскрипция, диаризация, темы, решения, действия. |
| Consistency / Contradiction Engine | `docs/engines/consistency/README.md` | Обнаружение противоречий между новыми данными и принятой памятью (Polygraph). |
| Context Packs Engine | `docs/engines/context-packs/README.md` | Перестраиваемые контекстные пакеты для сущностей на основе ADR-0096. |
| Enrichment Engine | `docs/engines/enrichment/README.md` | Кандидаты на обогащение сущностей: избранное, наблюдения, конфликты. |
| Identity Resolution Engine | `docs/engines/identity-resolution/README.md` | Кандидаты на объединение субъектов (ADR-0096). |
| Memory Engine | `docs/engines/memory/README.md` | Сборка долговременной памяти: факты, карточки, контекстные пакеты, пробелы. |
| Obligation Engine | `docs/engines/obligation/README.md` | Выделение обязательств, follow-up и кандидатов в задачи из доказательств. |
| Relationship Candidate Engine | `docs/engines/relationships/README.md` | Кандидаты на связи между сущностями (ADR-0096). |
| Risk Engine | `docs/engines/risk/README.md` | Сигналы риска, внимания и блокировок на основе доказательств. |
| Search Engine | `docs/engines/search/README.md` | Полнотекстовый и семантический поиск, memory queries, требования к результатам. |
| Архитектура поиска | `docs/engines/search/architecture.md` | Детали конвейера индексации, режимы поиска, источники ранжирования. |

## Упомянутые, но отсутствующие в текущем контексте

Нижеперечисленные файлы упоминаются в документах выше, однако не включены в
данный снимок. Их содержимое не подтверждено.

- `docs/foundation/glossary.md` — глоссарий, на который ссылается `docs/domains/tasks/api.md`.
- `docs/foundation/engines.md` — карта движков, упомянутая в `docs/engines/README.md`.
- Архитектурные решения (ADR): `docs/adr/ADR-0070`, `docs/adr/ADR-0087`, `docs/adr/ADR-0088`, `docs/adr/ADR-0096` — ссылки присутствуют в документации движков и доменов.
```

## Покрытие источников

Каждый исходный файл предоставил факты, прямо отражённые в предложенной странице `operations/documentation-map.md`:

- **`docs/domains/signal-hub/gap-analysis.md`**: заголовок «Signal Hub Gap Analysis», статус «target-vs-current gap analysis».
- **`docs/domains/signal-hub/modules.md`**: заголовок «Signal Hub Modules», целевое разбиение на бэкенд/платформенные события/контракты/фронтенд.
- **`docs/domains/signal-hub/operations.md`**: заголовок «Signal Hub Operations», контракт состояний (Enabled, Disabled, Muted, Paused, Fixture‑only, Replay‑only), глобальные и селективные контролы.
- **`docs/domains/signal-hub/spec.md`**: заголовок «Signal Hub Domain», список владения и не-владения, доменные правила.
- **`docs/domains/signal-hub/status.md`**: заголовок «Signal Hub Status», дата статуса «2026-06-23», общий статус «IMPLEMENTATION STARTED», таблицы готовности.
- **`docs/domains/signal-hub/testing.md`**: заголовок «Signal Hub Testing», принцип заменяемых фикстурных источников, слои тестирования.
- **`docs/domains/tasks/README.md`**: заголовок «Макошь Tasks», предметная граница.
- **`docs/domains/tasks/api.md`**: заголовок «Tasks — API Reference», перечисление REST‑маршрутов.
- **`docs/domains/tasks/architecture.md`**: заголовок «Tasks Architecture», описание модулей и слоёв.
- **`docs/domains/tasks/data-model.md`**: заголовок «Tasks Data Model», таблицы, ID‑форматы, статусы.
- **`docs/domains/tasks/spec.md`**: заголовок «Tasks Domain», жизненный цикл, правила извлечения.
- **`docs/domains/tasks/status.md`**: заголовок «Tasks — Статус реализации», факт 84 % реализации (87/104 разделов).
- **`docs/engines/README.md`**: заголовок «Макошь Engine Catalog», определение движка, список спецификаций и текущих реализаций.
- **`docs/engines/automation/README.md`**: заголовок «Automation Engine», статус «code‑aligned documentation package», текущая реализация.
- **`docs/engines/call-intelligence/README.md`**: заголовок «Call Intelligence Engine», описание конвейера.
- **`docs/engines/consistency/README.md`**: заголовок «Consistency / Contradiction Engine», псевдоним «Polygraph», статус и текущая реализация.
- **`docs/engines/context-packs/README.md`**: заголовок «Context Packs Engine», ссылка на ADR-0096.
- **`docs/engines/enrichment/README.md`**: заголовок «Enrichment Engine», статус, текущая реализация.
- **`docs/engines/identity-resolution/README.md`**: заголовок «Identity Resolution Engine», ссылка на ADR-0096.
- **`docs/engines/memory/README.md`**: заголовок «Memory Engine», описание ответственностей, планов миграции.
- **`docs/engines/obligation/README.md`**: заголовок «Obligation Engine», текущее состояние реализации.
- **`docs/engines/relationships/README.md`**: заголовок «Relationship Candidate Engine», ссылка на ADR-0096.
- **`docs/engines/risk/README.md`**: заголовок «Risk Engine», признак миграции доменно‑локального поведения.
- **`docs/engines/search/README.md`**: заголовок «Search Engine», ссылка на архитектуру.
- **`docs/engines/search/architecture.md`**: заголовок «Search Engine Architecture», конвейер индексации и режимы поиска.

## Исходные файлы

- [`docs/domains/signal-hub/gap-analysis.md`](../../../domains/signal-hub/gap-analysis.md)
- [`docs/domains/signal-hub/modules.md`](../../../domains/signal-hub/modules.md)
- [`docs/domains/signal-hub/operations.md`](../../../domains/signal-hub/operations.md)
- [`docs/domains/signal-hub/spec.md`](../../../domains/signal-hub/spec.md)
- [`docs/domains/signal-hub/status.md`](../../../domains/signal-hub/status.md)
- [`docs/domains/signal-hub/testing.md`](../../../domains/signal-hub/testing.md)
- [`docs/domains/tasks/README.md`](../../../domains/tasks/README.md)
- [`docs/domains/tasks/api.md`](../../../domains/tasks/api.md)
- [`docs/domains/tasks/architecture.md`](../../../domains/tasks/architecture.md)
- [`docs/domains/tasks/data-model.md`](../../../domains/tasks/data-model.md)
- [`docs/domains/tasks/spec.md`](../../../domains/tasks/spec.md)
- [`docs/domains/tasks/status.md`](../../../domains/tasks/status.md)
- [`docs/engines/README.md`](../../../engines/README.md)
- [`docs/engines/automation/README.md`](../../../engines/automation/README.md)
- [`docs/engines/call-intelligence/README.md`](../../../engines/call-intelligence/README.md)
- [`docs/engines/consistency/README.md`](../../../engines/consistency/README.md)
- [`docs/engines/context-packs/README.md`](../../../engines/context-packs/README.md)
- [`docs/engines/enrichment/README.md`](../../../engines/enrichment/README.md)
- [`docs/engines/identity-resolution/README.md`](../../../engines/identity-resolution/README.md)
- [`docs/engines/memory/README.md`](../../../engines/memory/README.md)
- [`docs/engines/obligation/README.md`](../../../engines/obligation/README.md)
- [`docs/engines/relationships/README.md`](../../../engines/relationships/README.md)
- [`docs/engines/risk/README.md`](../../../engines/risk/README.md)
- [`docs/engines/search/README.md`](../../../engines/search/README.md)
- [`docs/engines/search/architecture.md`](../../../engines/search/architecture.md)

## Кандидаты на drift

Из предоставленного контекста расхождения между документацией, кодом и ADR не видны. Все рассмотренные документы являются описательными спецификациями, статусами и планами; без доступа к соответствующим файлам исходного кода (`backend/src/`, `frontend/src/`, `contracts/`) невозможно подтвердить или опровергнуть совпадение текущего состояния реализации с заявленным в документации.
