### Summary / Резюме

Страница `operations/documentation-map.md` должна предоставлять централизованную карту всей доступной документации Макошь, сгруппированную по слоям (платформа, интеграции) и отражающую актуальные статусы реализации.
На основе встроенных исходных файлов создаётся обзорная страница с навигационными ссылками на ключевые документы пакетов `platform` и `integrations/zoom`, включая их статус (например, `FOUNDATION_IMPLEMENTED`), чтобы инженеры и операторы могли быстро оценить завершённость и найти нужный артефакт.
Контент пишется на русском, ссылки сохраняют оригинальные идентификаторы файлов.

---

### Proposed pages / Предлагаемые страницы

#### `operations/documentation-map.md`

```markdown
# Карта документации

Последнее обновление: 2026-06-28.

Эта страница даёт обзор всей документации Макошь, отражённой в текущем репозитории.
Статусы соответствуют реализации на указанную дату; для деталей переходите по ссылкам.

## Платформа (Platform)

[[../platform/README|README платформы]] — Введение в слой платформы, перечень технологических примитивов и правило документирования.
Статус: `documentation package aligned`.

### Пакеты платформы

#### Event Tracing

- [[../platform/event-tracing/README|Пакет Event Tracing]] — Назначение и структура.
- [[../platform/event-tracing/architecture|Архитектура]] — Статус: целевая архитектура с частичной backend-реализацией.
- [[../platform/event-tracing/data-model|Модель данных]] — Каноническое отображение графа трасс.
- [[../platform/event-tracing/api|API]] — Конечные точки трассировки, форма ответа, требования к realtime-событиям.
- [[../platform/event-tracing/status|Статус реализации]]
- [[../platform/event-tracing/gap-analysis|Анализ пробелов]]
- [[../platform/event-tracing/operations|Операции]] — Отладка, replay, работа с устаревшими событиями.
- `testing.md` упомянут в README, но не включён в текущий контекст.

#### Application Settings

[[../platform/settings/README|README]] — Упомянут в README платформы, содержимое не представлено в данном контексте.

#### Realtime Conversation

[[../platform/realtime-conversation/README|README]] — Упомянут в README платформы, содержимое не представлено в данном контексте.

## Интеграции (Integrations)

### Zoom

Статус: `FOUNDATION_IMPLEMENTED`.

#### Обзорные документы

- [[../integrations/zoom/integration|Обзор интеграции]] — Границы, runtime shape, целевые маршруты, события, виды провайдеров и назначения секретов.
- [[../integrations/zoom/architecture|Архитектура провайдера]] — Модель владения, inbound flow, контракты событий, сохранение evidence, границы санитизации и авторизации, инварианты безопасности.
- [[../integrations/zoom/modules|Карта модулей]] — Структура backend/frontend модулей, миграции, зависимости платформы, граничные правила.
- [[../integrations/zoom/status|Статус реализации]] — Детальная матрица текущее/целевое состояние, доказательства наличия артефактов в репозитории, результаты тестов (файл обрезан, подтверждает `FOUNDATION_IMPLEMENTED`).
  - [[../integrations/zoom/status/README|Status Details]] — переход к pass-log.
  - [[../integrations/zoom/status/pass-log|Журнал прохождения проверок]] — Закрытие документации и имплементации, прогоны тестов backend/frontend.

#### API

- [[../integrations/zoom/api|API Reference]] — Сводка маршрутов, общие модели аккаунта и рантайма, политика ошибок, realtime-события, retention, ключи кэша.
- [[../integrations/zoom/api/README|API Details]] — Навигация по детальным страницам API.
  - [[../integrations/zoom/api/accounts|Accounts API]] — Фикстуры, живые аккаунты, OAuth / Server-to-Server авторизация, обновление и обслуживание токенов, валидация.
  - [[../integrations/zoom/api/runtime|Runtime API]] — Подписки на вебхуки, ручная синхронизация облачных записей, статус, запуск/остановка/удаление.
  - [[../integrations/zoom/api/runtime-bridge|Runtime Bridge API]] — Приём наблюдений встреч/записей/транскриптов, импорт файлов транскриптов, верифицированный вебхук-мост, санитизация.

#### Планирование и анализ

- [[../integrations/zoom/implementation-plan|План реализации]] — Фазы 1–9; завершены 1–7, частично 8–9.
- [[../integrations/zoom/gap-analysis|Анализ пробелов]] — Сравнение текущего и целевого состояния, архитектурные и продуктовые пробелы.
- [[../integrations/zoom/blockers|Блокеры]] — Фундаментальные, архитектурные, безопасностные и UI блокеры.
- [[../integrations/zoom/fixture-test-matrix|Матрица фикстурных тестов]] — Набор сценариев для валидации без живого провайдера.
- [[../integrations/zoom/live-smoke-checklist|Чек-лист проверки на живом окружении]] — Проверки blocked-live и authorized-live режимов, фикстурного моста.
- [[../integrations/zoom/provider-runtime-research|Заметки об исследовании рантайма]] — Решения по формам рантайма, нерушимые правила, открытые вопросы.
```

---

### Source coverage / Покрытие источников

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

### Drift candidates / Кандидаты на drift

Из предоставленного контекста не видно расхождений между кодом, документацией и ADR.
Все страницы содержат согласованные статусы (`FOUNDATION_IMPLEMENTED`, `Accepted`) и взаимно непротиворечивые описания.
Единственное замечание: файл `docs/integrations/zoom/status.md` обрезан на 12000 символов, но видимая часть соответствует остальным документам.
