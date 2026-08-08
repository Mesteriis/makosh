### Summary / Резюме

Создаётся страница `operations/documentation-map.md` в русской Obsidian wiki, которая описывает структуру документации Макошь и служит навигационным хабом. Страница объединяет информацию из README всех задокументированных пакетов (`product`, `roadmap`, `refactoring`, `research`, `reviews`, `platform`), а также ссылается на упомянутые в источниках пакеты (`domains`, `engines`, `workflows`, `ai`, `architecture`, `foundation`, `integrations`, `adr`). Это позволяет разработчикам быстро находить нужные документы и понимать их статус.

### Proposed pages / Предлагаемые страницы

**`operations/documentation-map.md`**

```markdown
# Карта документации

Статус: актуально на 2026-06-28, соответствует структуре репозитория.

Эта страница описывает структуру документации Макошь и служит навигационным хабом. Каждый пакет содержит `README.md` с описанием и ссылками на ключевые документы.

## Пакеты документации

### Продуктовые документы

[[../../product/README|product]] — определяют скоуп продукта, мастер-спецификацию и дорожную карту. Это продуктовый источник истины, а не отчёты о статусе реализации.

- [[../../product/master-spec|Master Spec]] — целевая продуктовая модель и инвентаризация текущей реализации.
- [[../../product/product-charter|Product Charter]] — миссия продукта, пользователь, ключевые сценарии и ограничения.
- [[../../product/product-scope|Product Scope]] — что входит и не входит в продукт, карта способностей.
- [[../../product/development-roadmap|Development Roadmap]] — дорожная карта разработки, разрывы между целевой моделью и реализацией, план срезов.

### Дорожные карты и чеклисты версий

[[../../roadmap/README|roadmap]] — версионные планы и записи о закрытии. Текущее продуктовое направление находится в `docs/product/`.

- [[../../roadmap/product-roadmap|Product Roadmap]] — версии от 0.1 до 5.0 с целями, функциями и рисками.
- [[../../roadmap/v1-closure-checklist|V1 Closure Checklist]] — критерии готовности версии 1.0.
- Аналогичные чеклисты для V2, V3, V4, V5 (упомянуты в индексе).

### Рефакторинг и выравнивание

[[../../refactoring/README|refactoring]] — отслеживает известные разрывы между реализацией и документацией, планы миграции. Не заменяет ADR.

- [[../../refactoring/implementation-alignment-plan|Implementation Alignment Plan]] — отображение текущей реализации на целевую модель, исправленные расхождения.
- [[../../refactoring/product-alignment-plan|Product Alignment Plan]] — продуктовые разрывы и планы рефакторинга (Communication, Persona, Relationships, Polygraph, Decisions/Obligations, Engine boundaries, UI vocabulary).
- [[../../refactoring/documentation-code-alignment-report|Documentation Code Alignment Report]] — аудит структуры документации и бэкенд/фронтенд модулей, подтверждённые границы, будущие разрывы.
- [[../../refactoring/naming-conflicts-inventory|Naming Conflicts Inventory]] — инвентаризация конфликтов Persons ↔ Personas в API, схеме, модулях, типах, фронтенде.
- [[../../refactoring/ui-states-inventory|UI States Inventory]] — размеры компонентов (>500 строк), состояния Loading/Empty/Error/Skeleton, отсутствующие сторы, cross-domain импорты.

### Исследования

[[../../research/README|research]] — открытые вопросы и заметки. Решения из исследований должны переходить в продуктовые документы, архитектуру или ADR.

- [[../../research/open-questions|Open Questions]] — неразрешённые вопросы по провайдерам, хранилищу, AI, UI, безопасности.

### Обзоры

[[../../reviews/README|reviews]] — исторические записи для прослеживаемости, если только действующий ADR, архитектурный или продуктовый документ явно не повышает их статус.

- [[../../reviews/backend-architecture-review-2026-06-06|Backend Architecture Review 2026-06-06]] — обзор бэкенд-архитектуры от 2026-06-06 (исторический).

### Платформенная документация

`docs/platform/` — документация платформенных компонентов.

- [[../../platform/event-tracing/testing|Event Tracing Testing]] — модульные, интеграционные, API и фронтенд-тесты причинного трейсинга событий.
- [[../../platform/settings/README|Application Settings]] — настройки приложения: разрешённый список, типы, границы, реализация.
- [[../../platform/realtime-conversation/README|Realtime Conversation Platform]] — провайдер-нейтральный слой для живых разговоров (Zoom, Telemost, Meet, Jitsi, Discord).
  - [[../../platform/realtime-conversation/architecture|Architecture]] — границы, поток, владение, язык событий, политика источников.
  - [[../../platform/realtime-conversation/providers|Providers]] — адаптеры провайдеров, поверхность возможностей, поток команд и доказательств.
  - [[../../platform/realtime-conversation/recording-bundle|Recording Bundle]] — макет бандла записи, манифест, политика артефактов, приватность, неизменяемость.
  - [[../../platform/realtime-conversation/replay-and-live-notes|Replay and Live Notes]] — панель живых заметок, модель синхронизированного воспроизведения, трэк событий, поиск.

### Домены

`docs/domains/` — спецификации доменов (Communication, Personas, Organizations, Tasks, Decisions, Obligations, Projects, Documents, Events, Relationships). В индексе рефакторинга зафиксированы созданные документы для `review`, `communications` и других. Текущий статус уточняется в [[../../refactoring/documentation-code-alignment-report|отчёте о выравнивании]].

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

- Для понимания продукта начните с [[../../product/master-spec|Master Spec]].
- Для отслеживания текущих разрывов используйте [[../../refactoring/implementation-alignment-plan|Implementation Alignment Plan]].
- Для исследования открытых вопросов обращайтесь к [[../../research/open-questions|Open Questions]].
- Для исторического контекста смотрите [[../../reviews/README|Reviews]] и [[../../roadmap/README|Roadmap]].
- При противоречиях между документами доверяйте ADR, затем продуктовым спецификациям, затем отчётам о выравнивании.
```

### Source coverage / Покрытие источников

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

### Drift candidates / Кандидаты на drift

Из представленного контекста расхождения кода/документации не обнаружены. Предлагаемая страница опирается исключительно на структуру и содержимое встроенных файлов. Ряд пакетов (`docs/ai/`, `docs/architecture/`, `docs/foundation/`, `docs/domains/`, `docs/engines/`, `docs/workflows/`, `docs/integrations/`, `docs/adr/`) упомянут в источниках, но их README не встроены — страница корректно отмечает их наличие без детализации содержимого.
