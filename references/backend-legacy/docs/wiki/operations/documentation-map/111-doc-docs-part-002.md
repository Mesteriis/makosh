---
chunk_id: 111-doc-docs-part-002
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 111-doc-docs-part-002 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создать новую страницу `operations/documentation-map.md` в русской Obsidian‑wiki, которая описывает структуру проектной документации Макошь и служит навигационной картой. Страница собирает информацию из доступных исходных файлов пакетов `architecture`, `development` и `domains`, а также ссылается на смежные пакеты (`integrations`, `foundation`, `platform`), упомянутые в каталоге доменов.

## Предложенные страницы

### `operations/documentation-map.md`

```markdown
# Карта документации Макошь

Дата актуализации: состояние репозитория на 2026-06-18 (архитектурные страницы)
и 2026-06-23 (инфраструктура тестирования), а также актуальный каталог доменов.

Страница описывает структуру проектной документации и помогает быстро найти
нужный пакет.

## Структура пакетов

Корень документации — `docs/`. Основные разделы:

- **`architecture/`** — канонические архитектурные решения и видение продукта.
- **`development/`** — процессы разработчика, тестирование, CI.
- **`domains/`** — спецификации и архитектура предметных областей.
- **`integrations/`** — документация по интеграционным адаптерам (Telegram, Mail, WhatsApp, Zoom).
- **`foundation/`** — фундаментальные концепции: карта доменов, глоссарий, модель мира.
- **`engines/`** — документация на движки (поиск, таймлайн, обогащение, риски и пр.).
- **`platform/`** — платформенные сервисы (настройки, события, безопасность).
- **`product/`** — продуктовая спецификация.

> **Примечание:** не все пакеты представлены в текущем срезе исходных текстов. Перечислены
> только те, на которые есть прямые ссылки в доступном исходном материале.

## Архитектура (`architecture/`)

Ключевые документы:

- **``architecture/vision``** — Каноническое видение архитектуры: цель, не-идентичность, ответственности,
  границы, форма системы, связи доменов.
- **``architecture/ui-architecture``** — Цели UI, основные поверхности, модель навигации и состояния,
  модель AI-взаимодействия.
- **``architecture/ui``** — Каноническая архитектура UI, утверждённая 2026-06-18:
  технологический стек (Vue 3, TypeScript, Vite, Tauri 2, Pinia для временного UI‑состояния,
  TanStack Query для серверного состояния), модель поверхностей, правила взаимодействия.
- **``architecture/storage-architecture``** — Архитектура хранения: цели, компоненты
  (PostgreSQL, Host Vault, Object Storage, индексы Tantivy и векторный), распределение
  обязанностей, модель резервного копирования.

## Разработка (`development/`)

Общий индекс: **``development/README``**.

### Тестирование (`development/testing/`)

- **``development/testing/README``** — Обзор инфраструктуры тестирования, все цели `make`,
  логическая классификация тестов (unit / integration / e2e / architecture / snapshot),
  отчёты (JUnit и аналитика).
- **``development/testing/status``** — Статус модернизации тестов: матрица критериев приёмки
  (13/16 завершено, 3/16 частично), подтверждённые результаты запусков.
- **``development/testing/ci``** — Топология CI: pull request (архитектура, формат, clippy,
  unit, snapshot, фронтенд), push в `main` (добавляет integration, coverage, security),
  nightly (добавляет e2e и мутационное тестирование).
- **``development/testing/coverage``** — Покрытие кода через `cargo-llvm-cov`, запуск через
  `cargo nextest` в сессионной обвязке `makosh_test_session`.
- **``development/testing/mutation-testing``** — Мутационное тестирование `cargo-mutants`,
  цель `make mutants`, выполняется в nightly CI и вручную.
- **``development/testing/nextest``** — `cargo-nextest` как основной Rust-раннер, профили
  (`default`, `ci`, `integration`), обязательное использование через `makosh_test_session`.
- **``development/testing/security``** — Безопасность и гигиена зависимостей:
  `cargo-audit`, `cargo-deny`, `cargo-udeps`, текущее состояние на 2026-06-23.
- **``development/testing/snapshots``** — Снапшот-тестирование через `insta`, приёмка
  новых снапшотов.

## Домены (`domains/`)

Каталог доменов: **``domains/README``**.

### Канонические домены

| Домен | Основные документы | Статус |
|---|---|---|
| Signal Hub | ``domains/signal-hub/spec``, ``domains/signal-hub/README`` | целевая системная область |
| Communications | ``domains/communications/README``, ``domains/communications/architecture`` | реализован как единый коммуникационный домен |
| Personas | ``domains/persons/spec``, ``domains/persons/README`` | частично реализован |
| Relationships | ``domains/relationships/README`` | частично реализован |
| Organizations | ``domains/organizations/spec``, ``domains/organizations/README`` | реализован |
| Projects | ``domains/projects/README`` | реализован |
| Documents | ``domains/documents/README`` | реализован |
| Tasks | ``domains/tasks/spec``, ``domains/tasks/README`` | реализован |
| Calendar/Events | ``domains/calendar/spec``, ``domains/calendar/README``, ``domains/calendar/api``, ``domains/calendar/architecture``, ``domains/calendar/data-model``, ``domains/calendar/status`` | реализован (91 % покрытия спеки) |
| Decisions | ``domains/decisions/README`` | частично реализован |
| Obligations | ``domains/obligations/README`` | частично реализован |
| Review | ``domains/review/README`` | реализован |
| Knowledge Graph | ``domains/graph/README`` | реализован |
| Agents | ``domains/agents/README`` | частично реализован |
| Notes | ``domains/notes/README`` | не выделен в отдельный домен |

Подробнее об ограничениях текущей реализации см. **``domains/README``** (раздел
«Current Implementation Caveats»).

### Канальные спецификации (`integrations/`)

Документы описывают поведение провайдеров без повышения до уровня домена:

- **``integrations/telegram/README``** — Telegram Channel Capability Spec.
- **``integrations/mail/README``** — Email Channel Capability Spec.
- **``integrations/whatsapp/README``** — WhatsApp Provider Stage.
- **``integrations/zoom/README``** — Zoom Provider Stage.

Каналы являются интеграциями. Доменом остаётся Communications.

## Как читать эту карту

- Для понимания архитектурных границ начните с **``architecture/vision``**.
- Для работы с кодом и тестами — **``development/testing/README``**.
- Для изучения предметной области — **``domains/README``** и документы конкретного домена.
- Для интеграций — **``integrations/telegram/README``** и аналогичные страницы.

Карта обновляется по мере консолидации документации. Последняя значительная консолидация —
2026-06-18 для архитектурных страниц, 2026-06-23 для инфраструктуры тестирования.
```

## Покрытие источников

| Source file | Facts used |
|---|---|
| `docs/architecture/storage-architecture.md` | Компоненты хранения, обязанности PostgreSQL/Host Vault/Object Storage/индексов, модель резервного копирования, замечание о `encrypted_secret_vault_entries` как легаси после ADR-0076. |
| `docs/architecture/ui-architecture.md` | Цели UI, основные поверхности, модель навигации и состояния, модель AI-взаимодействия. |
| `docs/architecture/ui.md` | Каноническая архитектура UI: стек (Vue 3, TypeScript, Vite, Tauri 2, Pinia, TanStack Query), модель поверхностей, правила взаимодействия. |
| `docs/architecture/vision.md` | Цель, не-идентичность, ответственности, границы, форма системы, связи доменов, причины существования. |
| `docs/development/README.md` | Индекс пакета разработки, ссылка на тестирование. |
| `docs/development/testing/README.md` | Обзор тестовой инфраструктуры, карта команд `make`, классификация тестов, отчёты. |
| `docs/development/testing/ci.md` | Разделение CI по pull request / push в main / nightly. |
| `docs/development/testing/coverage.md` | Команды покрытия, запуск через `makosh_test_session`. |
| `docs/development/testing/mutation-testing.md` | `cargo-mutants`, выполнение вне PR-гейта. |
| `docs/development/testing/nextest.md` | `cargo-nextest` как основной раннер, профили, ограничение: предпочитать `make`‑цели с `makosh_test_session`. |
| `docs/development/testing/security.md` | Команды `make audit`, `make deny`, `make security`, `make udeps`, текущее состояние на 2026-06-23. |
| `docs/development/testing/snapshots.md` | Снапшот-тестирование `insta`, приёмка новых снапшотов. |
| `docs/development/testing/status.md` | Матрица готовности (13/16 завершено, 3/16 частично), результаты запусков валидации. |
| `docs/domains/README.md` | Каталог доменов, канальные спецификации, движки, оговорки текущей реализации (легаси‑имена, `settings` как пустой модуль). |
| `docs/domains/agents/README.md` | Ответственности домена агентов, модель персоны `ai_agent`, workflow, текущий статус, план миграции. |
| `docs/domains/calendar/README.md` | Ключевые возможности календаря, используемые движки, навигация. |
| `docs/domains/calendar/api.md` | Описание 42 эндпоинтов календаря. |
| `docs/domains/calendar/architecture.md` | Модули, слои, паттерны календаря. |
| `docs/domains/calendar/data-model.md` | 15 таблиц, ID-форматы, индексы. |
| `docs/domains/calendar/spec.md` | Спецификация, типы событий, доказательства реализации, план миграции. |
| `docs/domains/calendar/status.md` | Покрытие спеки: 68/75 разделов (91 %), отложенные разделы, метрики. |
| `docs/domains/communications/README.md` | Коммуникации как ingestion spine, инвариант «канал не домен», trace context, extraction pipeline, engine use, доказательства реализации, план миграции. |
| `docs/domains/communications/architecture.md` | Адаптеры, провайдерские аккаунты, канонические объекты, использование движков, исходящие сообщения, threading. |
| `docs/domains/decisions/README.md` | Ответственности, источники решений, модель `Decision`, текущая реализация, план миграции. |
| `docs/domains/documents/README.md` | Ответственности, поддерживаемые типы, граница Notes, жизненный цикл, версионирование, linking. |

## Исходные файлы

- [`docs/architecture/storage-architecture.md`](../../../architecture/storage-architecture.md)
- [`docs/architecture/ui-architecture.md`](../../../architecture/ui-architecture.md)
- [`docs/architecture/ui.md`](../../../architecture/ui.md)
- [`docs/architecture/vision.md`](../../../architecture/vision.md)
- [`docs/development/README.md`](../../../development/README.md)
- [`docs/development/testing/README.md`](../../../development/testing/README.md)
- [`docs/development/testing/ci.md`](../../../development/testing/ci.md)
- [`docs/development/testing/coverage.md`](../../../development/testing/coverage.md)
- [`docs/development/testing/mutation-testing.md`](../../../development/testing/mutation-testing.md)
- [`docs/development/testing/nextest.md`](../../../development/testing/nextest.md)
- [`docs/development/testing/security.md`](../../../development/testing/security.md)
- [`docs/development/testing/snapshots.md`](../../../development/testing/snapshots.md)
- [`docs/development/testing/status.md`](../../../development/testing/status.md)
- [`docs/domains/README.md`](../../../domains/README.md)
- [`docs/domains/agents/README.md`](../../../domains/agents/README.md)
- [`docs/domains/calendar/README.md`](../../../domains/calendar/README.md)
- [`docs/domains/calendar/api.md`](../../../domains/calendar/api.md)
- [`docs/domains/calendar/architecture.md`](../../../domains/calendar/architecture.md)
- [`docs/domains/calendar/data-model.md`](../../../domains/calendar/data-model.md)
- [`docs/domains/calendar/spec.md`](../../../domains/calendar/spec.md)
- [`docs/domains/calendar/status.md`](../../../domains/calendar/status.md)
- [`docs/domains/communications/README.md`](../../../domains/communications/README.md)
- [`docs/domains/communications/architecture.md`](../../../domains/communications/architecture.md)
- [`docs/domains/decisions/README.md`](../../../domains/decisions/README.md)
- [`docs/domains/documents/README.md`](../../../domains/documents/README.md)

## Кандидаты на drift

Из предоставленного контекста не видно прямых расхождений между кодом и документацией — все исходные файлы сами являются документацией. Однако в документации явно описаны несколько состояний, которые можно считать плановыми расхождениями (известные оговорки, ожидающие будущих ADR или доработок):

- **Пустой модуль `backend/src/domains/settings`** при том, что рабочая логика настроек находится в `platform/settings/` — зафиксировано в **``domains/README``** как известный факт, а не ошибка документации.
- **Легаси-схема `encrypted_secret_vault_entries`** в PostgreSQL — в **``storage-architecture` (`architecture/storage-architecture`)`** обозначена как «legacy/migration state после ADR-0076».
- **Имена совместимости** в backend-модулях Communications (почтовые имена, `person_id` и `contacts`) — оговорены в **``domains/README``** как миграционные артефакты, требующие будущих ADR.

Ни один из этих пунктов не является непреднамеренным дрейфом; все они документированы в исходных файлах как известные ограничения текущего состояния репозитория. Дрейф между разными пакетами документации в данном chunk не обнаружен.
