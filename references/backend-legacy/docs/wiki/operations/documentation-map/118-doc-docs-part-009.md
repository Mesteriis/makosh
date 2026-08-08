---
chunk_id: 118-doc-docs-part-009
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 118-doc-docs-part-009 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создаётся страница `operations/documentation-map.md` в русской Obsidian wiki, которая описывает структуру документации Макошь и служит навигационным хабом. Страница объединяет информацию из README всех задокументированных пакетов (`product`, `roadmap`, `refactoring`, `research`, `reviews`, `platform`), а также ссылается на упомянутые в источниках пакеты (`domains`, `engines`, `workflows`, `ai`, `architecture`, `foundation`, `integrations`, `adr`). Это позволяет разработчикам быстро находить нужные документы и понимать их статус.

## Предложенные страницы

**`operations/documentation-map.md`**

```markdown
# Карта документации

Статус: актуально на 2026-06-28, соответствует структуре репозитория.

Эта страница описывает структуру документации Макошь и служит навигационным хабом. Каждый пакет содержит `README.md` с описанием и ссылками на ключевые документы.

## Пакеты документации

### Продуктовые документы

``product` (`../../product/README`)` — определяют скоуп продукта, мастер-спецификацию и дорожную карту. Это продуктовый источник истины, а не отчёты о статусе реализации.

- ``Master Spec` (`../../product/master-spec`)` — целевая продуктовая модель и инвентаризация текущей реализации.
- ``Product Charter` (`../../product/product-charter`)` — миссия продукта, пользователь, ключевые сценарии и ограничения.
- ``Product Scope` (`../../product/product-scope`)` — что входит и не входит в продукт, карта способностей.
- ``Development Roadmap` (`../../product/development-roadmap`)` — дорожная карта разработки, разрывы между целевой моделью и реализацией, план срезов.

### Дорожные карты и чеклисты версий

``roadmap` (`../../roadmap/README`)` — версионные планы и записи о закрытии. Текущее продуктовое направление находится в `docs/product/`.

- ``Product Roadmap` (`../../roadmap/product-roadmap`)` — версии от 0.1 до 5.0 с целями, функциями и рисками.
- ``V1 Closure Checklist` (`../../roadmap/v1-closure-checklist`)` — критерии готовности версии 1.0.
- Аналогичные чеклисты для V2, V3, V4, V5 (упомянуты в индексе).

### Рефакторинг и выравнивание

``refactoring` (`../../refactoring/README`)` — отслеживает известные разрывы между реализацией и документацией, планы миграции. Не заменяет ADR.

- ``Implementation Alignment Plan` (`../../refactoring/implementation-alignment-plan`)` — отображение текущей реализации на целевую модель, исправленные расхождения.
- ``Product Alignment Plan` (`../../refactoring/product-alignment-plan`)` — продуктовые разрывы и планы рефакторинга (Communication, Persona, Relationships, Polygraph, Decisions/Obligations, Engine boundaries, UI vocabulary).
- ``Documentation Code Alignment Report` (`../../refactoring/documentation-code-alignment-report`)` — аудит структуры документации и бэкенд/фронтенд модулей, подтверждённые границы, будущие разрывы.
- ``Naming Conflicts Inventory` (`../../refactoring/naming-conflicts-inventory`)` — инвентаризация конфликтов Persons ↔ Personas в API, схеме, модулях, типах, фронтенде.
- ``UI States Inventory` (`../../refactoring/ui-states-inventory`)` — размеры компонентов (>500 строк), состояния Loading/Empty/Error/Skeleton, отсутствующие сторы, cross-domain импорты.

### Исследования

``research` (`../../research/README`)` — открытые вопросы и заметки. Решения из исследований должны переходить в продуктовые документы, архитектуру или ADR.

- ``Open Questions` (`../../research/open-questions`)` — неразрешённые вопросы по провайдерам, хранилищу, AI, UI, безопасности.

### Обзоры

``reviews` (`../../reviews/README`)` — исторические записи для прослеживаемости, если только действующий ADR, архитектурный или продуктовый документ явно не повышает их статус.

- ``Backend Architecture Review 2026-06-06` (`../../reviews/backend-architecture-review-2026-06-06`)` — обзор бэкенд-архитектуры от 2026-06-06 (исторический).

### Платформенная документация

`docs/platform/` — документация платформенных компонентов.

- ``Event Tracing Testing` (`../../platform/event-tracing/testing`)` — модульные, интеграционные, API и фронтенд-тесты причинного трейсинга событий.
- ``Application Settings` (`../../platform/settings/README`)` — настройки приложения: разрешённый список, типы, границы, реализация.
- ``Realtime Conversation Platform` (`../../platform/realtime-conversation/README`)` — провайдер-нейтральный слой для живых разговоров (Zoom, Telemost, Meet, Jitsi, Discord).
  - ``Architecture` (`../../platform/realtime-conversation/architecture`)` — границы, поток, владение, язык событий, политика источников.
  - ``Providers` (`../../platform/realtime-conversation/providers`)` — адаптеры провайдеров, поверхность возможностей, поток команд и доказательств.
  - ``Recording Bundle` (`../../platform/realtime-conversation/recording-bundle`)` — макет бандла записи, манифест, политика артефактов, приватность, неизменяемость.
  - ``Replay and Live Notes` (`../../platform/realtime-conversation/replay-and-live-notes`)` — панель живых заметок, модель синхронизированного воспроизведения, трэк событий, поиск.

### Домены

`docs/domains/` — спецификации доменов (Communication, Personas, Organizations, Tasks, Decisions, Obligations, Projects, Documents, Events, Relationships). В индексе рефакторинга зафиксированы созданные документы для `review`, `communications` и других. Текущий статус уточняется в ``отчёте о выравнивании` (`../../refactoring/documentation-code-alignment-report`)`.

### Движки

`docs/engines/` — спецификации движков (Memory, Timeline, Trust, Search, Enrichment, Obligation, Risk, Consistency/Contradiction). В индексе рефакторинга зафиксированы созданные документы для `automation`, `context-packs`, `identity-resolution`, `relationships`.

### Workflows

`docs/workflows/` — документирует сквозные процессы (communication-to-knowledge, meeting-to-decisions и др.). В индексе рефакторинга отмечено, что продуктовая документация workflow не зеркалит каждый конкретный модуль бэкенда.

### AI и агенты

`docs/ai/` — документация AI-агентов. Упомянута в дорожной карте разработки (`docs/ai/agents/`).

### Архитектура

`docs/architecture/` — архитектурные диаграммы и принципы. Упомянуты в отчёте о выравнивании.

### Фундамент

`docs/foundation/` — видение, мировая модель, архитектурные принципы. Упомянуты в `product/product-charter`.

### Интеграции

`docs/integrations/` — документация по провайдерским интеграциям (Telegram, WhatsApp, Zoom, Mail, Ollama, OmniRoute). Упомянуты в отчёте о выравнивании и `product/development-roadmap`.

### ADR

`docs/adr/` — архитектурные решения (ADR). Являются источником истины при расхождениях между кодом и документацией. Множество ADR процитированы в предоставленных документах.

## Как пользоваться картой

- Для понимания продукта начните с ``Master Spec` (`../../product/master-spec`)`.
- Для отслеживания текущих разрывов используйте ``Implementation Alignment Plan` (`../../refactoring/implementation-alignment-plan`)`.
- Для исследования открытых вопросов обращайтесь к ``Open Questions` (`../../research/open-questions`)`.
- Для исторического контекста смотрите ``Reviews` (`../../reviews/README`)` и ``Roadmap` (`../../roadmap/README`)`.
- При противоречиях между документами доверяйте ADR, затем продуктовым спецификациям, затем отчётам о выравнивании.
```

## Покрытие источников

| Исходный файл | Факты, покрытые в предлагаемой странице |
|---|---|
| `docs/platform/event-tracing/testing.md` | Ссылка на страницу тестирования трейсинга событий. |
| `docs/platform/realtime-conversation/README.md` | Описание платформы реального времени, ссылки на архитектуру, провайдеров, бандл записи, повтор и живые заметки. |
| `docs/platform/realtime-conversation/architecture.md` | Упомянуто в навигации как часть пакета. |
| `docs/platform/realtime-conversation/providers.md` | Упомянуто. |
| `docs/platform/realtime-conversation/recording-bundle.md` | Упомянуто. |
| `docs/platform/realtime-conversation/replay-and-live-notes.md` | Упомянуто. |
| `docs/platform/settings/README.md` | Описание пакета настроек приложения. |
| `docs/product/README.md` | Описание пакета `product`, его роли как продуктового источника истины. |
| `docs/product/development-roadmap.md` (truncated) | Ссылка на дорожную карту разработки, упоминания `docs/ai/agents/` и планов документации. |
| `docs/product/master-spec.md` (truncated) | Ссылка на Master Spec как целевую модель; упоминание структуры доменов и движков. |
| `docs/product/product-charter.md` | Описание Product Charter и ссылки на foundation-документы. |
| `docs/product/product-scope.md` | Описание Product Scope. |
| `docs/refactoring/README.md` | Описание пакета `refactoring`, его назначения. |
| `docs/refactoring/documentation-code-alignment-report.md` | Перечисление ADR, созданных doc-пакетов, подтверждённых границ; упоминание `docs/domains/`, `docs/engines/`, `docs/integrations/`. |
| `docs/refactoring/implementation-alignment-plan.md` (truncated) | Перечисление исправленных расхождений, ссылки на ADR, созданные doc-пакеты. |
| `docs/refactoring/naming-conflicts-inventory.md` | Упоминание конфликтов Persons ↔ Personas, cross-domain импортов. |
| `docs/refactoring/product-alignment-plan.md` | Описание продуктового плана выравнивания, разрывов по Communication, Persona, Relationships, Polygraph, Decisions/Obligations; план создания doc-пакетов. |
| `docs/refactoring/ui-states-inventory.md` | Описание инвентаризации UI-состояний и размеров компонентов. |
| `docs/research/README.md` | Описание пакета `research`. |
| `docs/research/open-questions.md` | Ссылка на открытые вопросы. |
| `docs/reviews/README.md` | Описание пакета `reviews` как исторических записей. |
| `docs/reviews/backend-architecture-review-2026-06-06.md` (truncated) | Упомянут как исторический обзор. |
| `docs/roadmap/README.md` | Описание пакета `roadmap`, его отличия от `product/`. |
| `docs/roadmap/product-roadmap.md` | Ссылка на продуктовую дорожную карту версий. |
| `docs/roadmap/v1-closure-checklist.md` | Ссылка на чеклист V1. |

## Исходные файлы

- [`docs/platform/event-tracing/testing.md`](../../../platform/event-tracing/testing.md)
- [`docs/platform/realtime-conversation/README.md`](../../../platform/realtime-conversation/README.md)
- [`docs/platform/realtime-conversation/architecture.md`](../../../platform/realtime-conversation/architecture.md)
- [`docs/platform/realtime-conversation/providers.md`](../../../platform/realtime-conversation/providers.md)
- [`docs/platform/realtime-conversation/recording-bundle.md`](../../../platform/realtime-conversation/recording-bundle.md)
- [`docs/platform/realtime-conversation/replay-and-live-notes.md`](../../../platform/realtime-conversation/replay-and-live-notes.md)
- [`docs/platform/settings/README.md`](../../../platform/settings/README.md)
- [`docs/product/README.md`](../../../product/README.md)
- [`docs/product/development-roadmap.md`](../../../product/development-roadmap.md)
- [`docs/product/master-spec.md`](../../../product/master-spec.md)
- [`docs/product/product-charter.md`](../../../product/product-charter.md)
- [`docs/product/product-scope.md`](../../../product/product-scope.md)
- [`docs/refactoring/README.md`](../../../refactoring/README.md)
- [`docs/refactoring/documentation-code-alignment-report.md`](../../../refactoring/documentation-code-alignment-report.md)
- [`docs/refactoring/implementation-alignment-plan.md`](../../../refactoring/implementation-alignment-plan.md)
- [`docs/refactoring/naming-conflicts-inventory.md`](../../../refactoring/naming-conflicts-inventory.md)
- [`docs/refactoring/product-alignment-plan.md`](../../../refactoring/product-alignment-plan.md)
- [`docs/refactoring/ui-states-inventory.md`](../../../refactoring/ui-states-inventory.md)
- [`docs/research/README.md`](../../../research/README.md)
- [`docs/research/open-questions.md`](../../../research/open-questions.md)
- [`docs/reviews/README.md`](../../../reviews/README.md)
- [`docs/reviews/backend-architecture-review-2026-06-06.md`](../../../reviews/backend-architecture-review-2026-06-06.md)
- [`docs/roadmap/README.md`](../../../roadmap/README.md)
- [`docs/roadmap/product-roadmap.md`](../../../roadmap/product-roadmap.md)
- [`docs/roadmap/v1-closure-checklist.md`](../../../roadmap/v1-closure-checklist.md)

## Кандидаты на drift

Из представленного контекста расхождения кода/документации не обнаружены. Предлагаемая страница опирается исключительно на структуру и содержимое встроенных файлов. Ряд пакетов (`docs/ai/`, `docs/architecture/`, `docs/foundation/`, `docs/domains/`, `docs/engines/`, `docs/workflows/`, `docs/integrations/`, `docs/adr/`) упомянут в источниках, но их README не встроены — страница корректно отмечает их наличие без детализации содержимого.
