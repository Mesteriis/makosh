---
chunk_id: 117-doc-docs-part-008
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 117-doc-docs-part-008 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `operations/documentation-map.md` должна предоставлять централизованную карту всей доступной документации Макошь, сгруппированную по слоям (платформа, интеграции) и отражающую актуальные статусы реализации.
На основе встроенных исходных файлов создаётся обзорная страница с навигационными ссылками на ключевые документы пакетов `platform` и `integrations/zoom`, включая их статус (например, `FOUNDATION_IMPLEMENTED`), чтобы инженеры и операторы могли быстро оценить завершённость и найти нужный артефакт.
Контент пишется на русском, ссылки сохраняют оригинальные идентификаторы файлов.

---

## Предложенные страницы

#### `operations/documentation-map.md`

```markdown
# Карта документации

Последнее обновление: 2026-06-28.

Эта страница даёт обзор всей документации Макошь, отражённой в текущем репозитории.
Статусы соответствуют реализации на указанную дату; для деталей переходите по ссылкам.

## Платформа (Platform)

``README платформы` (`../platform/README`)` — Введение в слой платформы, перечень технологических примитивов и правило документирования.
Статус: `documentation package aligned`.

### Пакеты платформы

#### Event Tracing

- ``Пакет Event Tracing` (`../platform/event-tracing/README`)` — Назначение и структура.
- ``Архитектура` (`../platform/event-tracing/architecture`)` — Статус: целевая архитектура с частичной backend-реализацией.
- ``Модель данных` (`../platform/event-tracing/data-model`)` — Каноническое отображение графа трасс.
- ``API` (`../platform/event-tracing/api`)` — Конечные точки трассировки, форма ответа, требования к realtime-событиям.
- ``Статус реализации` (`../platform/event-tracing/status`)`
- ``Анализ пробелов` (`../platform/event-tracing/gap-analysis`)`
- ``Операции` (`../platform/event-tracing/operations`)` — Отладка, replay, работа с устаревшими событиями.
- `testing.md` упомянут в README, но не включён в текущий контекст.

#### Application Settings

``README` (`../platform/settings/README`)` — Упомянут в README платформы, содержимое не представлено в данном контексте.

#### Realtime Conversation

``README` (`../platform/realtime-conversation/README`)` — Упомянут в README платформы, содержимое не представлено в данном контексте.

## Интеграции (Integrations)

### Zoom

Статус: `FOUNDATION_IMPLEMENTED`.

#### Обзорные документы

- ``Обзор интеграции` (`../integrations/zoom/integration`)` — Границы, runtime shape, целевые маршруты, события, виды провайдеров и назначения секретов.
- ``Архитектура провайдера` (`../integrations/zoom/architecture`)` — Модель владения, inbound flow, контракты событий, сохранение evidence, границы санитизации и авторизации, инварианты безопасности.
- ``Карта модулей` (`../integrations/zoom/modules`)` — Структура backend/frontend модулей, миграции, зависимости платформы, граничные правила.
- ``Статус реализации` (`../integrations/zoom/status`)` — Детальная матрица текущее/целевое состояние, доказательства наличия артефактов в репозитории, результаты тестов (файл обрезан, подтверждает `FOUNDATION_IMPLEMENTED`).
  - ``Status Details` (`../integrations/zoom/status/README`)` — переход к pass-log.
  - ``Журнал прохождения проверок` (`../integrations/zoom/status/pass-log`)` — Закрытие документации и имплементации, прогоны тестов backend/frontend.

#### API

- ``API Reference` (`../integrations/zoom/api`)` — Сводка маршрутов, общие модели аккаунта и рантайма, политика ошибок, realtime-события, retention, ключи кэша.
- ``API Details` (`../integrations/zoom/api/README`)` — Навигация по детальным страницам API.
  - ``Accounts API` (`../integrations/zoom/api/accounts`)` — Фикстуры, живые аккаунты, OAuth / Server-to-Server авторизация, обновление и обслуживание токенов, валидация.
  - ``Runtime API` (`../integrations/zoom/api/runtime`)` — Подписки на вебхуки, ручная синхронизация облачных записей, статус, запуск/остановка/удаление.
  - ``Runtime Bridge API` (`../integrations/zoom/api/runtime-bridge`)` — Приём наблюдений встреч/записей/транскриптов, импорт файлов транскриптов, верифицированный вебхук-мост, санитизация.

#### Планирование и анализ

- ``План реализации` (`../integrations/zoom/implementation-plan`)` — Фазы 1–9; завершены 1–7, частично 8–9.
- ``Анализ пробелов` (`../integrations/zoom/gap-analysis`)` — Сравнение текущего и целевого состояния, архитектурные и продуктовые пробелы.
- ``Блокеры` (`../integrations/zoom/blockers`)` — Фундаментальные, архитектурные, безопасностные и UI блокеры.
- ``Матрица фикстурных тестов` (`../integrations/zoom/fixture-test-matrix`)` — Набор сценариев для валидации без живого провайдера.
- ``Чек-лист проверки на живом окружении` (`../integrations/zoom/live-smoke-checklist`)` — Проверки blocked-live и authorized-live режимов, фикстурного моста.
- ``Заметки об исследовании рантайма` (`../integrations/zoom/provider-runtime-research`)` — Решения по формам рантайма, нерушимые правила, открытые вопросы.
```

---

## Покрытие источников

- **`docs/integrations/zoom/api.md`** — описание маршрутов, модели аккаунта/рантайма, политика ошибок, realtime-события, retention, ключи кэша.
- **`docs/integrations/zoom/api/README.md`** — навигация по детальным страницам API.
- **`docs/integrations/zoom/api/accounts.md`** — фикстуры, живые аккаунты, OAuth / S2S, обновление токенов, обслуживание, валидация.
- **`docs/integrations/zoom/api/runtime-bridge.md`** — meeting/recording/transcript bridge, импорт файлов транскриптов, вебхуки, санитизация.
- **`docs/integrations/zoom/api/runtime.md`** — подписки на вебхуки, provider-sync, статус, start/stop/remove, поля статуса.
- **`docs/integrations/zoom/architecture.md`** — владение, inbound flow, runtime shapes, контракты событий, персистентность evidence, границы санитизации и авторизации, downstream интерпретация, инварианты безопасности.
- **`docs/integrations/zoom/blockers.md`** — foundation/live runtime/архитектурные/безопасность/UI блокеры.
- **`docs/integrations/zoom/fixture-test-matrix.md`** — тестовые сценарии для аккаунтов, моста, санитизации, идемпотентности.
- **`docs/integrations/zoom/gap-analysis.md`** — текущее vs целевое состояние, архитектурные и продуктовые пробелы, пробелы в тестировании.
- **`docs/integrations/zoom/implementation-plan.md`** — фазы 1–9 со статусами, не-цели.
- **`docs/integrations/zoom/integration.md`** — границы, runtime shape, целевые маршруты, события, виды провайдеров, назначения секретов.
- **`docs/integrations/zoom/live-smoke-checklist.md`** — blocked-live и authorized-live проверки, фикстурные проверки.
- **`docs/integrations/zoom/modules.md`** — карта backend/frontend модулей, зависимости платформы, миграция, граничные правила.
- **`docs/integrations/zoom/provider-runtime-research.md`** — решения по формам рантайма, нерушимые правила, открытые вопросы.
- **`docs/integrations/zoom/status.md`** (truncated) — доказательства наличия в репозитории, сводка foundation, состояние возможностей.
- **`docs/integrations/zoom/status/README.md`** — навигация на pass-log.
- **`docs/integrations/zoom/status/pass-log.md`** — documentation pass, implementation pass с результатами тестов.
- **`docs/platform/README.md`** — обзор слоя платформы, текущие пакеты, кодовые области, правило документирования.
- **`docs/platform/event-tracing/README.md`** — назначение, структура пакета event tracing.
- **`docs/platform/event-tracing/api.md`** — конечные точки, форма ответа, требования к realtime-событиям, frontend-поверхность.
- **`docs/platform/event-tracing/architecture.md`** — слои, backend/frontend слои, контракты событий, API, тестирование, операции.
- **`docs/platform/event-tracing/data-model.md`** — каноническое отображение, модель графа трасс, таблицы, устаревшие события.
- **`docs/platform/event-tracing/gap-analysis.md`** — известные пробелы, watchpoints.
- **`docs/platform/event-tracing/operations.md`** — отладка, устаревшие события, replay, приватность.
- **`docs/platform/event-tracing/status.md`** — реализованное, частично реализованное, запланированное, заблокированное, устаревшее.

---

## Исходные файлы

- [`docs/integrations/zoom/api.md`](../../../integrations/zoom/api.md)
- [`docs/integrations/zoom/api/README.md`](../../../integrations/zoom/api/README.md)
- [`docs/integrations/zoom/api/accounts.md`](../../../integrations/zoom/api/accounts.md)
- [`docs/integrations/zoom/api/runtime-bridge.md`](../../../integrations/zoom/api/runtime-bridge.md)
- [`docs/integrations/zoom/api/runtime.md`](../../../integrations/zoom/api/runtime.md)
- [`docs/integrations/zoom/architecture.md`](../../../integrations/zoom/architecture.md)
- [`docs/integrations/zoom/blockers.md`](../../../integrations/zoom/blockers.md)
- [`docs/integrations/zoom/fixture-test-matrix.md`](../../../integrations/zoom/fixture-test-matrix.md)
- [`docs/integrations/zoom/gap-analysis.md`](../../../integrations/zoom/gap-analysis.md)
- [`docs/integrations/zoom/implementation-plan.md`](../../../integrations/zoom/implementation-plan.md)
- [`docs/integrations/zoom/integration.md`](../../../integrations/zoom/integration.md)
- [`docs/integrations/zoom/live-smoke-checklist.md`](../../../integrations/zoom/live-smoke-checklist.md)
- [`docs/integrations/zoom/modules.md`](../../../integrations/zoom/modules.md)
- [`docs/integrations/zoom/provider-runtime-research.md`](../../../integrations/zoom/provider-runtime-research.md)
- [`docs/integrations/zoom/status.md`](../../../integrations/zoom/status.md)
- [`docs/integrations/zoom/status/README.md`](../../../integrations/zoom/status/README.md)
- [`docs/integrations/zoom/status/pass-log.md`](../../../integrations/zoom/status/pass-log.md)
- [`docs/platform/README.md`](../../../platform/README.md)
- [`docs/platform/event-tracing/README.md`](../../../platform/event-tracing/README.md)
- [`docs/platform/event-tracing/api.md`](../../../platform/event-tracing/api.md)
- [`docs/platform/event-tracing/architecture.md`](../../../platform/event-tracing/architecture.md)
- [`docs/platform/event-tracing/data-model.md`](../../../platform/event-tracing/data-model.md)
- [`docs/platform/event-tracing/gap-analysis.md`](../../../platform/event-tracing/gap-analysis.md)
- [`docs/platform/event-tracing/operations.md`](../../../platform/event-tracing/operations.md)
- [`docs/platform/event-tracing/status.md`](../../../platform/event-tracing/status.md)

## Кандидаты на drift

Из предоставленного контекста не видно расхождений между кодом, документацией и ADR.
Все страницы содержат согласованные статусы (`FOUNDATION_IMPLEMENTED`, `Accepted`) и взаимно непротиворечивые описания.
Единственное замечание: файл `docs/integrations/zoom/status.md` обрезан на 12000 символов, но видимая часть соответствует остальным документам.
