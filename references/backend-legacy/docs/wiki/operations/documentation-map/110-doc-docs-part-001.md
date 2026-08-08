---
chunk_id: 110-doc-docs-part-001
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 110-doc-docs-part-001 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

В русскую wiki проекта Макошь добавляется навигационная карта документации в файле `operations/documentation-map.md`. Карта собирает все основные категории документов (продуктовые, foundational, доменные, интеграционные, движковые, архитектурные, AI, ADR и др.), опираясь исключительно на встроенные исходные файлы. Цель — дать разработчикам, агентам и участникам единую точку входа в документацию с указанием порядка чтения и кратким описанием ключевых разделов.

## Предложенные страницы

### `operations/documentation-map.md`

```markdown
# Карта документации Макошь

> Центральная навигационная карта документации проекта. Основана на [`docs/README.md`](../README.md) и встроенных архитектурных документах.

Макошь — локальная система персональной памяти (local-first Personal Memory System). Документация построена вокруг нескольких опорных слоёв: продукт, фундамент (foundation), предметные области (domains), интеграции, движки (engines), архитектурные решения (ADR) и статус реализации.

## Порядок чтения

Для новых разработчиков и агентов рекомендуется читать документы в указанном порядке:

1.  **[Product Master Spec](../product/master-spec.md)** — продуктовая спецификация, источник истины на уровне продукта.
2.  **[Foundation Vision](../foundation/vision.md)** — видение и базовые ценности системы.
3.  **[Glossary](../foundation/glossary.md)** — глоссарий канонических терминов.
4.  **[World Model](../foundation/world-model.md)** — модель мира, описывающая сущности и их отношения.
5.  **[Product Development Roadmap](../product/development-roadmap.md)** — дорожная карта развития продукта.
6.  **[Domain Map](../foundation/domain-map.md)** — карта предметных областей.
7.  **[Architecture Overview](../architecture/architecture-overview.md)** — обзор архитектуры.
8.  **[ADR Index](../adr/README.md)** — индекс архитектурных решений.

## Канонические источники

При расхождениях между документами предпочтение отдаётся следующим каноническим файлам (если только более новый ADR явно не отменяет их):

- **[Foundation Vision](../foundation/vision.md)**
- **[Glossary](../foundation/glossary.md)**
- **[World Model](../foundation/world-model.md)**
- **[Engines](../foundation/engines.md)**
- **[Architecture Principles](../foundation/architecture-principles.md)**
- **[Domain Map](../foundation/domain-map.md)**

## Разделы документации

### Продуктовые документы

- **[Product Master Spec](../product/master-spec.md)** — единый продуктовый источник истины.
- **[Product Charter](../product/product-charter.md)** — назначение продукта, пользователь и критерии качества.
- **[Product Scope](../product/product-scope.md)** — in-scope и out-of-scope области продукта.
- **[Product Development Roadmap](../product/development-roadmap.md)** — будущие срезы и планы рефакторинга.
- **[Product Alignment Refactoring Plan](../refactoring/product-alignment-plan.md)** — текущие расхождения целевой модели и реализации.
- **[Implementation Alignment Plan](../refactoring/implementation-alignment-plan.md)** — расхождения кода, модулей, схем и UI относительно канонической модели.
- **[Canonical Evidence Final Report](../../canonical-evidence-final-report.md)** — текущий статус реализации, таблица прогресса и следующие срезы.

Исторические роадмапы находятся в `docs/roadmap/`.

### Foundational документы (основы)

- **[Foundation Vision](../foundation/vision.md)**
- **[World Model](../foundation/world-model.md)**
- **[Glossary](../foundation/glossary.md)**
- **[Engines](../foundation/engines.md)**
- **[Architecture Principles](../foundation/architecture-principles.md)**
- **[Domain Map](../foundation/domain-map.md)**

### Доменные документы

Канонические спецификации доменов находятся в каталоге **[Domain Catalog](../domains/README.md)**. Папки доменов по возможности зеркалируют `backend/src/domains/<domain>/`.

Основные домены:

- **[Signal Hub](../domains/signal-hub/spec.md)**, [package](../domains/signal-hub/README.md)
- **[Communications](../domains/communications/README.md)**
- **[Personas / Persona Intelligence](../domains/persons/spec.md)**, [package](../domains/persons/README.md)
- **[Organizations](../domains/organizations/spec.md)**, [package](../domains/organizations/README.md)
- **[Projects](../domains/projects/README.md)**
- **[Documents](../domains/documents/README.md)**
- **[Tasks](../domains/tasks/spec.md)**, [package](../domains/tasks/README.md)
- **[Calendar And Events](../domains/calendar/spec.md)**, [package](../domains/calendar/README.md)
- **[Decisions](../domains/decisions/README.md)**
- **[Obligations](../domains/obligations/README.md)**
- **[Review](../domains/review/README.md)**
- **[Knowledge Graph](../domains/graph/README.md)**
- **[Agents](../domains/agents/README.md)**
- **[Notes Boundary](../domains/notes/README.md)**

### Интеграции

Документация провайдеров и каналов собрана в **[Integration Catalog](../integrations/README.md)**. Интеграции не являются продуктовыми доменами.

- **[Mail](../integrations/mail/README.md)**
- **[Telegram](../integrations/telegram/README.md)**
- **[WhatsApp](../integrations/whatsapp/README.md)**
- **[Zoom](../integrations/zoom/README.md)**
- **[Yandex Telemost](../integrations/yandex-telemost/README.md)**
- **[Ollama](../integrations/ollama/README.md)**
- **[OmniRoute](../integrations/omniroute/README.md)**

### Документы движков (Engines)

Общее описание движков — в **[Foundation Engines](../foundation/engines.md)**. Детальный каталог — в **[Engine Catalog](../engines/README.md)**.

- **[Memory Engine](../engines/memory/README.md)**
- **[Timeline Engine](../engines/timeline/README.md)**
- **[Trust Engine](../engines/trust/README.md)**
- **[Search Engine](../engines/search/README.md)**, [architecture](../engines/search/architecture.md)
- **[Enrichment Engine](../engines/enrichment/README.md)**
- **[Obligation Engine](../engines/obligation/README.md)**
- **[Risk Engine](../engines/risk/README.md)**
- **[Consistency / Contradiction Engine](../engines/consistency/README.md)** (пользовательское имя — Polygraph)
- **[Automation Engine](../engines/automation/README.md)**
- **[Context Packs Engine](../engines/context-packs/README.md)**
- **[Identity Resolution Engine](../engines/identity-resolution/README.md)**
- **[Relationship Candidate Engine](../engines/relationships/README.md)**
- **[Call Intelligence Engine](../engines/call-intelligence/README.md)**
- **[Speaker Identity Engine](../engines/speaker-identity/README.md)**

### Архитектурные документы

Пакет `docs/architecture` содержит сквозную системную архитектуру. Детали, специфичные для доменов, провайдеров и движков, живут в их собственных пакетах.

Навигация по архитектурному пакету (из [`architecture/README.md`](../architecture/README.md)):

- **[Architecture Overview](../architecture/architecture-overview.md)**
- **[Context Diagram](../architecture/context-diagram.md)**
- **[Container Diagram](../architecture/container-diagram.md)**
- **[Component Communication](../architecture/component-communication.md)**
- **[Principles](../architecture/principles.md)**
- **[Domain Map](../architecture/domain-map.md)**
- **[Domains](../architecture/domains.md)**
- **[Communications](../architecture/communications.md)**
- **[Event Model](../architecture/event-model.md)**
- **[Memory](../architecture/memory.md)**
- **[Signal Hub](../architecture/signal-hub.md)**
- **[Storage Architecture](../architecture/storage-architecture.md)**
- **[Security Model](../architecture/security-model.md)**
- **[Privacy Model](../architecture/privacy-model.md)**
- **[Plugin Architecture](../architecture/plugin-architecture.md)**
- **[Agents](../architecture/agents.md)**
- **[Radar](../architecture/radar.md)**
- **[UI Architecture](../architecture/ui-architecture.md)**
- **[UI](../architecture/ui.md)**
- **[Vision](../architecture/vision.md)**
- **[Refactoring Plan](../architecture/refactoring-plan.md)**

### AI-документы

AI-слой (`docs/ai`) описывает локальный доступ к моделям, конфигурацию control-center, семантический поиск, контракты промптов/рантайма и сервисы для агентов. AI-вывод никогда не является источником истины.

- **[AI Layer Overview](../ai/README.md)**
- **[Agent Architecture](../ai/agents/agent-architecture.md)**
- **[Local AI Architecture](../ai/agents/local-ai-architecture.md)**

### Документы слоя приложений

Пакет `docs/app` соответствует `backend/src/app`. Владеет HTTP, ConnectRPC, регистрацией роутов, guard-ами запросов и тонкими обработчиками. Бизнес-логика сюда не входит.

- **[App Layer Overview](../app/README.md)**

Координация между доменами, рабочими процессами и провайдерами описывается в `docs/application/`.

- **[Application Services Overview](../application/README.md)**

### Workflow-документы (рабочие процессы)

Спецификации процессов — в **[Workflow Catalog](../workflows/README.md)**.

- **[Communication To Knowledge](../workflows/communication-to-knowledge.md)**
- **[Communication To Obligation](../workflows/communication-to-obligation.md)**
- **[Meeting To Decisions](../workflows/meeting-to-decisions.md)**
- **[Document To Context](../workflows/document-to-context.md)**
- **[Contradiction Review](../workflows/contradiction-review.md)**
- **[Dossier Generation](../workflows/dossier-generation.md)**
- **[Agent Assisted Recall](../workflows/agent-assisted-recall.md)**

### ADR (записи архитектурных решений)

ADR живут в **[каталоге adr](../adr/)**. Это устойчивые архитектурные решения. Некоторые старые ADR сохраняют исторические термины (например, `Contact`, `Person`), поскольку реализация эволюционировала. При конфликте новый ADR имеет приоритет.

Важные текущие ADR (перечислены в `docs/README.md`):

- **ADR-0001** — событийное ядро системы (event sourcing).
- **ADR-0008** — граф знаний как основа (knowledge graph first).
- **ADR-0022** — запрет файн-тюнинга на приватных данных.
- **ADR-0056** — текущий локальный API-защитник (shared-secret guard).
- **ADR-0055** — полное сетевое взаимодействие почтового провайдера.
- **ADR-0077** — русский и английский интерфейс.
- **ADR-0084** — Persona Intelligence System.
- **ADR-0085** — коммуникационный хребет и Consistency / Contradiction Engine.
- **ADR-0091** — модель возможностей продакшн-клиента Telegram.
- **ADR-0095** — событийно-ориентированная коммуникация доменов и DLQ.
- **ADR-0099** — платформа событий Signal Hub.
- **ADR-0102** — принятая граница рантайма провайдера Zoom.
- **ADR-0104** — предложенная граница рантайма провайдера Yandex Telemost.

### Статус реализации

Файлы статуса и API описывают текущую реализацию. Они полезны, но не всегда используют канонический язык продукта.

Примеры:

- `docs/integrations/mail/status.md`
- `docs/domains/calendar/status.md`
- `docs/domains/tasks/status.md`
- `docs/domains/persons/status.md`
- `docs/domains/*/api.md`
- `docs/integrations/*/api.md`

Централизованный отчёт текущего периода — **[Canonical Evidence Final Report](../../canonical-evidence-final-report.md)**. Отчёты о выравнивании кода и документации — **[Documentation Code Alignment Report](../refactoring/documentation-code-alignment-report.md)**.

### Исторические документы

Исторические обзоры (`docs/reviews/`) и файлы закрытия версий (`docs/roadmap/`) являются traceability-записями, если только на них явно не ссылаются как на активные требования текущие продуктовые, foundational, архитектурные документы или ADR.

## Как пользоваться этой картой

- Для общего понимания начните с раздела **«Порядок чтения»**.
- Для изучения конкретной области используйте ссылки в разделах: **Продукт**, **Домены**, **Движки**, **Архитектура**.
- Для поиска информации о принятых технических решениях обращайтесь к **ADR**.
- Текущее состояние кода и реализованных возможностей смотрите в **Статусе реализации**.
- При нестыковках между документами отдавайте предпочтение **каноническим источникам** и более новым **ADR**.
```

## Покрытие источников

| Исходный файл | Использованные факты |
|---|---|
| `docs/README.md` | Полная структура документации: порядок чтения, канонические источники, списки продуктовых, foundational, доменных, интеграционных, движковых, workflow документов, ADR, статус реализации, исторические документы. Принцип "если документы конфликтуют — предпочитать foundational и ADR". |
| `docs/ai/README.md` | Назначение AI-слоя, условие "AI output is never source of truth", ссылки на agent-architecture и local-ai-architecture. |
| `docs/ai/agents/README.md` | Пакет агентов, навигация. |
| `docs/ai/agents/agent-architecture.md` | Сущность и роли агентов, архитектура рантайма. |
| `docs/ai/agents/local-ai-architecture.md` | Цели локального AI, компоненты, запрет файн-тюнинга. |
| `docs/app/README.md` | Границы App-слоя, перечень его частей. |
| `docs/application/README.md` | Границы Application Services слоя. |
| `docs/architecture/README.md` | Список архитектурных документов. |
| `docs/architecture/agents.md` | Каноническая архитектура агентов. |
| `docs/architecture/architecture-overview.md` | Архитектурный тезис, верхнеуровневая форма системы, слои, категории состояния. |
| `docs/architecture/communications.md` | Каноническая архитектура Communications. |
| `docs/architecture/component-communication.md` | Контракт взаимодействия компонентов. |
| `docs/architecture/container-diagram.md` | Контейнерная диаграмма и ответственности контейнеров. |
| `docs/architecture/context-diagram.md` | Системный контекст, внешние акторы. |
| `docs/architecture/domain-map.md` | Карта доменов и движков, перекрёстные правила. |
| `docs/architecture/domains.md` | Каноническая архитектура доменов: что владеет каждый домен, что не является доменами, границы движков. |
| `docs/architecture/event-model.md` | Событийная модель как системный хребет. |
| `docs/architecture/memory.md` | Архитектура памяти, состояния памяти, границы знаний. |
| `docs/architecture/plugin-architecture.md` | Архитектура плагинов. |
| `docs/architecture/principles.md` | Канонические архитектурные принципы. |
| `docs/architecture/privacy-model.md` | Модель приватности. |
| `docs/architecture/radar.md` | Позиция Radar как read-model, не домена. |
| `docs/architecture/refactoring-plan.md` | План рефакторинга handlers, но отмечено как устаревшая нотация. |
| `docs/architecture/security-model.md` | Модель безопасности. |
| `docs/architecture/signal-hub.md` | Архитектура Signal Hub. |

## Исходные файлы

- [`docs/README.md`](../../../README.md)
- [`docs/ai/README.md`](../../../ai/README.md)
- [`docs/ai/agents/README.md`](../../../ai/agents/README.md)
- [`docs/ai/agents/agent-architecture.md`](../../../ai/agents/agent-architecture.md)
- [`docs/ai/agents/local-ai-architecture.md`](../../../ai/agents/local-ai-architecture.md)
- [`docs/app/README.md`](../../../app/README.md)
- [`docs/application/README.md`](../../../application/README.md)
- [`docs/architecture/README.md`](../../../architecture/README.md)
- [`docs/architecture/agents.md`](../../../architecture/agents.md)
- [`docs/architecture/architecture-overview.md`](../../../architecture/architecture-overview.md)
- [`docs/architecture/communications.md`](../../../architecture/communications.md)
- [`docs/architecture/component-communication.md`](../../../architecture/component-communication.md)
- [`docs/architecture/container-diagram.md`](../../../architecture/container-diagram.md)
- [`docs/architecture/context-diagram.md`](../../../architecture/context-diagram.md)
- [`docs/architecture/domain-map.md`](../../../architecture/domain-map.md)
- [`docs/architecture/domains.md`](../../../architecture/domains.md)
- [`docs/architecture/event-model.md`](../../../architecture/event-model.md)
- [`docs/architecture/memory.md`](../../../architecture/memory.md)
- [`docs/architecture/plugin-architecture.md`](../../../architecture/plugin-architecture.md)
- [`docs/architecture/principles.md`](../../../architecture/principles.md)
- [`docs/architecture/privacy-model.md`](../../../architecture/privacy-model.md)
- [`docs/architecture/radar.md`](../../../architecture/radar.md)
- [`docs/architecture/refactoring-plan.md`](../../../architecture/refactoring-plan.md)
- [`docs/architecture/security-model.md`](../../../architecture/security-model.md)
- [`docs/architecture/signal-hub.md`](../../../architecture/signal-hub.md)

## Кандидаты на drift

В предоставленных исходных файлах видимого расхождения нет.

Единственный заслуживающий внимания момент: файл `docs/architecture/refactoring-plan.md` оперирует устаревшими терминами (`persons`, `health`, `watchlist`, `promises`), однако сам же явно помечает их как «compatibility labels» и ссылается на каноническую модель из `../foundation/`. Таким образом, это документированное временное несоответствие, а не скрытый drift. Все остальные файлы архитектуры и `docs/README.md` согласованы между собой в части классификации доменов, движков и назначения документации.
