---
chunk_id: 115-doc-docs-part-006
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 18
generated_by: code-wiki-ru
---

# 115-doc-docs-part-006 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `18`

## Резюме

Добавляется страница `operations/documentation-map.md` — навигационная карта всей документации по интеграционным каналам (Telegram, WhatsApp) в проекте Макошь. Она собирает ссылки на существующие страницы архитектуры, API, статуса, анализа блокеров и исследовательские документы, делая wiki самодостаточной для поиска нужного артефакта. Страница пишется на русском, использует Obsidian-совместимые wikilinks относительно корня репозитория.

## Предложенные страницы

`operations/documentation-map.md`:

```markdown
# Карта документации

Навигационная карта документации по интеграционным каналам связи Макошь.
Здесь перечислены все страницы, покрывающие архитектуру, API, статус реализации,
анализ блокеров, продуктовые исследования и плановые документы для Telegram и WhatsApp.

## Telegram

### Общая архитектура и статус

- ``Архитектура Telegram` (`../integrations/telegram/architecture`)` — целевая архитектурная спецификация канала Telegram.
- ``Модули Telegram` (`../integrations/telegram/modules`)` — инвентаризация backend- и frontend-модулей Telegram.
- ``Статус реализации Telegram` (`../integrations/telegram/status`)` — сводка состояния базового набора возможностей (`COMPLETED`).
- ``Архитектурные блокеры Telegram` (`../integrations/telegram/blockers`)` — текущие и закрытые архитектурные блокеры.
- ``Анализ пробелов Telegram` (`../integrations/telegram/gap-analysis`)` — анализ закрытых и отложенных возможностей.
- ``Продуктовое исследование Telegram` (`../integrations/telegram/product-research`)` — исследовательский снимок следующих направлений.

### Статус: детали

- ``Обзор деталей статуса Telegram` (`../integrations/telegram/status/README`)`
- ``Основные детали статуса Telegram` (`../integrations/telegram/status/details-core`)`
- ``Расширенные детали статуса Telegram` (`../integrations/telegram/status/details-extended`)`
- ``Журнал завершения Telegram` (`../integrations/telegram/status/pass-log`)`

### API

- ``Telegram API: Медиа и поиск` (`../integrations/telegram/api/media-search`)` — маршруты загрузки/выгрузки медиа, поиска вложений и сообщений.
- ``Telegram API: Операции и Realtime` (`../integrations/telegram/api/operations-realtime`)` — маршруты автоматизации/политик, аудита, звонков, realtime-событий.

> **Примечание:** индексный файл API (`api.md`) и справочник по разговорам (`conversations.md`) упоминаются в исходном коде, но их содержимое не подтверждено данным контекстом.

## WhatsApp

### Общая архитектура и статус

- ``Обзор WhatsApp Channel` (`../integrations/whatsapp/README`)` — введение, роль в Communications Domain, ключевые принципы, навигация.
- ``Архитектура WhatsApp` (`../integrations/whatsapp/architecture`)` — целевая архитектурная спецификация.
- ``Архитектурные блокеры WhatsApp` (`../integrations/whatsapp/blockers`)` — 15 стартовых архитектурных блокеров.
- ``Аудит WhatsApp (2026-06-24)` (`../integrations/whatsapp/current-audit-2026-06-24`)` — текущий аудит репозитория.
- ``Матрица фикстурных тестов WhatsApp` (`../integrations/whatsapp/fixture-test-matrix`)` — coverage map фикстурных тестов.

### API

- ``WhatsApp API` (`../integrations/whatsapp/api`)` — описание реализованных маршрутов runtime/команд/синхронизации.

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

## Покрытие источников

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

## Исходные файлы

- [`docs/integrations/telegram/api/media-search.md`](../../../integrations/telegram/api/media-search.md)
- [`docs/integrations/telegram/api/operations-realtime.md`](../../../integrations/telegram/api/operations-realtime.md)
- [`docs/integrations/telegram/architecture.md`](../../../integrations/telegram/architecture.md)
- [`docs/integrations/telegram/blockers.md`](../../../integrations/telegram/blockers.md)
- [`docs/integrations/telegram/gap-analysis.md`](../../../integrations/telegram/gap-analysis.md)
- [`docs/integrations/telegram/modules.md`](../../../integrations/telegram/modules.md)
- [`docs/integrations/telegram/product-research.md`](../../../integrations/telegram/product-research.md)
- [`docs/integrations/telegram/status.md`](../../../integrations/telegram/status.md)
- [`docs/integrations/telegram/status/README.md`](../../../integrations/telegram/status/README.md)
- [`docs/integrations/telegram/status/details-core.md`](../../../integrations/telegram/status/details-core.md)
- [`docs/integrations/telegram/status/details-extended.md`](../../../integrations/telegram/status/details-extended.md)
- [`docs/integrations/telegram/status/pass-log.md`](../../../integrations/telegram/status/pass-log.md)
- [`docs/integrations/whatsapp/README.md`](../../../integrations/whatsapp/README.md)
- [`docs/integrations/whatsapp/api.md`](../../../integrations/whatsapp/api.md)
- [`docs/integrations/whatsapp/architecture.md`](../../../integrations/whatsapp/architecture.md)
- [`docs/integrations/whatsapp/blockers.md`](../../../integrations/whatsapp/blockers.md)
- [`docs/integrations/whatsapp/current-audit-2026-06-24.md`](../../../integrations/whatsapp/current-audit-2026-06-24.md)
- [`docs/integrations/whatsapp/fixture-test-matrix.md`](../../../integrations/whatsapp/fixture-test-matrix.md)

## Кандидаты на drift

1. **Provider reconciliation для folder labels/mutations**
   `telegram/gap-analysis.md` (2026-06-18) объявляет «Provider reconciliation CLOSED» и включает «folder add, folder remove and folder reassign» как закрытые.
   `telegram/architecture.md` (статус на 2026-06-17, блок «Provider Command Outbox») явно перечисляет «Folder labels/mutations» среди элементов, которым всё ещё нужно «stronger provider-observed reconciliation before they can be marked completed».
   Расхождение: либо архитектурный документ устарел и не отражает последние изменения, либо gap analysis излишне оптимистичен.

2. **Событие `telegram.sync.failed`**
   В `operations-realtime.md` перечень событий realtime включает `telegram.sync.started`, `telegram.sync.progress`, `telegram.sync.completed`, но не `telegram.sync.failed`. Однако ниже, в списке «Current emission scope», указано: «chat/history sync -> telegram.sync.started/progress/completed/failed». Это мелкое расхождение в документации (список событий неполон относительно прозы).

Других расхождений между документами, включёнными в контекст, не обнаружено.
