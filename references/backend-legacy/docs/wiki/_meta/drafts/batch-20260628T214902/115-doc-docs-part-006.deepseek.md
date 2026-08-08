### Summary / Резюме

Добавляется страница `operations/documentation-map.md` — навигационная карта всей документации по интеграционным каналам (Telegram, WhatsApp) в проекте Макошь. Она собирает ссылки на существующие страницы архитектуры, API, статуса, анализа блокеров и исследовательские документы, делая wiki самодостаточной для поиска нужного артефакта. Страница пишется на русском, использует Obsidian-совместимые wikilinks относительно корня репозитория.

### Proposed pages / Предлагаемые страницы

`operations/documentation-map.md`:

```markdown
# Карта документации

Навигационная карта документации по интеграционным каналам связи Макошь.
Здесь перечислены все страницы, покрывающие архитектуру, API, статус реализации,
анализ блокеров, продуктовые исследования и плановые документы для Telegram и WhatsApp.

## Telegram

### Общая архитектура и статус

- [[../integrations/telegram/architecture|Архитектура Telegram]] — целевая архитектурная спецификация канала Telegram.
- [[../integrations/telegram/modules|Модули Telegram]] — инвентаризация backend- и frontend-модулей Telegram.
- [[../integrations/telegram/status|Статус реализации Telegram]] — сводка состояния базового набора возможностей (`COMPLETED`).
- [[../integrations/telegram/blockers|Архитектурные блокеры Telegram]] — текущие и закрытые архитектурные блокеры.
- [[../integrations/telegram/gap-analysis|Анализ пробелов Telegram]] — анализ закрытых и отложенных возможностей.
- [[../integrations/telegram/product-research|Продуктовое исследование Telegram]] — исследовательский снимок следующих направлений.

### Статус: детали

- [[../integrations/telegram/status/README|Обзор деталей статуса Telegram]]
- [[../integrations/telegram/status/details-core|Основные детали статуса Telegram]]
- [[../integrations/telegram/status/details-extended|Расширенные детали статуса Telegram]]
- [[../integrations/telegram/status/pass-log|Журнал завершения Telegram]]

### API

- [[../integrations/telegram/api/media-search|Telegram API: Медиа и поиск]] — маршруты загрузки/выгрузки медиа, поиска вложений и сообщений.
- [[../integrations/telegram/api/operations-realtime|Telegram API: Операции и Realtime]] — маршруты автоматизации/политик, аудита, звонков, realtime-событий.

> **Примечание:** индексный файл API (`api.md`) и справочник по разговорам (`conversations.md`) упоминаются в исходном коде, но их содержимое не подтверждено данным контекстом.

## WhatsApp

### Общая архитектура и статус

- [[../integrations/whatsapp/README|Обзор WhatsApp Channel]] — введение, роль в Communications Domain, ключевые принципы, навигация.
- [[../integrations/whatsapp/architecture|Архитектура WhatsApp]] — целевая архитектурная спецификация.
- [[../integrations/whatsapp/blockers|Архитектурные блокеры WhatsApp]] — 15 стартовых архитектурных блокеров.
- [[../integrations/whatsapp/current-audit-2026-06-24|Аудит WhatsApp (2026-06-24)]] — текущий аудит репозитория.
- [[../integrations/whatsapp/fixture-test-matrix|Матрица фикстурных тестов WhatsApp]] — coverage map фикстурных тестов.

### API

- [[../integrations/whatsapp/api|WhatsApp API]] — описание реализованных маршрутов runtime/команд/синхронизации.

**Связанные документы, упоминаемые в документации, но не включённые в данный контекст (содержимое не подтверждено):**

- `full-functionality-target.md`
- `rust-provider-research.md`
- `implementation-plan.md`
- `live-smoke-checklist.md`
- `ADR-0101-whatsapp-provider-runtime-selection.md`
- `modules.md`, `status.md`, `gap-analysis.md`

## Ключевые ADR, определяющие архитектуру интеграций

Следующие ADR упоминаются в документации Telegram и WhatsApp. Документы ADR
находятся в директории `docs/adr/`. Их точное содержимое и расположение не
подтверждено данным контекстом.

- **ADR-0001** — Event sourcing как основа системы
- **ADR-0013** — Local-first data ownership
- **ADR-0018** — Provider adapter boundary
- **ADR-0027** — Capability-based permission model
- **ADR-0031** — Desktop-only UI scope
- **ADR-0046** — Blob storage и scanner boundary
- **ADR-0050** — V4 Telegram policy automation и call intelligence
- **ADR-0051** — WhatsApp Web companion boundary
- **ADR-0052** — Capability/action confirmation policy
- **ADR-0056** — Router-level `X-Макошь-Secret` local API auth
- **ADR-0074** — Multi-channel identity traces (WhatsApp/phone)
- **ADR-0076** — Host vault для новых secret payloads
- **ADR-0083** — Account-scoped TDLib runtime slice (Telegram)
- **ADR-0085** — Communication spine и Polygraph integration
- **ADR-0091** — Production Telegram capability model
- **ADR-0093** — Vue 3 frontend
- **ADR-0097** — Channels are integrations; Communications owns domain state
- **ADR-0099** — Signal Hub control (source pause/resume/replay)
- **ADR-0101** — WhatsApp provider/runtime selection (упомянут в WhatsApp README)
```

### Source coverage / Покрытие источников

| Исходный файл | Факты, покрытые на странице |
|---|---|
| `docs/integrations/telegram/api/media-search.md` | Заголовок «Media and Search», описание маршрутов, примечания. |
| `docs/integrations/telegram/api/operations-realtime.md` | Заголовок «Operations and Realtime», основные темы: политики, аудит, звонки, realtime-события. |
| `docs/integrations/telegram/architecture.md` | Заголовок «Telegram Architecture», статус и целевая природа документа, упомянутые ADR. |
| `docs/integrations/telegram/blockers.md` | Статус «нет активных архитектурных блокировок», перечень закрытых блокеров. |
| `docs/integrations/telegram/gap-analysis.md` | Статус «COMPLETED», матрица закрытых областей, список отложенных инициатив. |
| `docs/integrations/telegram/modules.md` | Инвентаризация backend/frontend модулей, статус «DONE» и «planned», boundary rules. |
| `docs/integrations/telegram/product-research.md` | Тема «Product Research And Next Bets», контекст исследования. |
| `docs/integrations/telegram/status.md` | Сводный статус реализации, перечень областей «DONE», отложенные инициативы. |
| `docs/integrations/telegram/status/README.md` | Назначение пакета «Status Details», ссылки на дочерние документы. |
| `docs/integrations/telegram/status/details-core.md` | Детализация аккаунтов, capability, диалогов, сообщений, реакций, тем. |
| `docs/integrations/telegram/status/details-extended.md` | Детализация поиска, медиа, realtime, фронтенда, аудита и scope boundary. |
| `docs/integrations/telegram/status/pass-log.md` | Журнал закрытия проверок, evidence и deferred passes. |
| `docs/integrations/whatsapp/README.md` | Введение в канал WhatsApp, ключевые принципы, навигация, упоминания связанных документов. |
| `docs/integrations/whatsapp/api.md` | Текущая API поверхность runtime/команд/синхронизации, упоминание `runtime/health` и `whatsapp_web_companion`. |
| `docs/integrations/whatsapp/architecture.md` | Целевая архитектура, trace-контракт, ключевые ADR, backend/frontend слои. |
| `docs/integrations/whatsapp/blockers.md` | 15 блокеров, статус `blocked`, планы решения. |
| `docs/integrations/whatsapp/current-audit-2026-06-24.md` | Аудит репозитория, раздел «Existing API surface» и «Existing repository assets». |
| `docs/integrations/whatsapp/fixture-test-matrix.md` | Матрица покрытия фикстурных тестов, таблицы source records и command classes. |
| Упоминания в файлах (Telegram architecture, WhatsApp README) | Список ADR, извлечённый из таблиц «Key ADR» и текста документов. |

### Drift candidates / Кандидаты на drift

1. **Provider reconciliation для folder labels/mutations**
   `telegram/gap-analysis.md` (2026-06-18) объявляет «Provider reconciliation CLOSED» и включает «folder add, folder remove and folder reassign» как закрытые.
   `telegram/architecture.md` (статус на 2026-06-17, блок «Provider Command Outbox») явно перечисляет «Folder labels/mutations» среди элементов, которым всё ещё нужно «stronger provider-observed reconciliation before they can be marked completed».
   Расхождение: либо архитектурный документ устарел и не отражает последние изменения, либо gap analysis излишне оптимистичен.

2. **Событие `telegram.sync.failed`**
   В `operations-realtime.md` перечень событий realtime включает `telegram.sync.started`, `telegram.sync.progress`, `telegram.sync.completed`, но не `telegram.sync.failed`. Однако ниже, в списке «Current emission scope», указано: «chat/history sync -> telegram.sync.started/progress/completed/failed». Это мелкое расхождение в документации (список событий неполон относительно прозы).

Других расхождений между документами, включёнными в контекст, не обнаружено.
