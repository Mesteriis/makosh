---
chunk_id: 008-doc-supergoal-part-001
batch_id: batch-20260628T214902
group: .supergoal
role: doc
source_status: pending
source_count: 19
generated_by: code-wiki-ru
---

# 008-doc-supergoal-part-001 — .supergoal/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `19`

## Резюме

Добавить в русскую Obsidian‑wiki страницу `operations/documentation-map.md` – централизованную карту текущих и запланированных работ по выравниванию документации и реализации в Макошь. Страница формируется исключительно на основе встроенных в контекст файлов суперцелей `makosh-docs-alignment` и `makosh-frontend-migration-vue-3`. Описывается статус, фазы, выявленные расхождения (gaps), архитектурные ограничения и связанные ADR для инициативы «Макошь Docs Alignment», а также даётся ссылка на завершённую миграцию фронтенда.

## Предложенные страницы

### `operations/documentation-map.md`

```markdown
# Карта документации

Обзор инициатив по выравниванию документации и реализации в Макошь.

## Supergoal: Макошь Docs Alignment

**Каталог запуска:** `.supergoal/makosh-docs-alignment-Q2wXmK`
**Статус:** планирование (по состоянию на 2026-06-14)
**Текущая фаза:** 0 (ни одна не начата)
**Всего фаз:** 10

### Цель

Устранить расхождения между документацией и реализацией:
- Persona naming (Persons ↔ Personas)
- Communication domain (выделение Communication как primary ingestion spine)
- Notes backend (отсутствует)
- Frontend states (Loading/Empty/Error/Skeleton)
- Telegram/Mail parity
- Тестовая инфраструктура (только 1 placeholder‑тест)

### Выявленные расхождения (12 Gaps)

| # | Gap | Привязка к фазе |
|---|-----|-----------------|
| 1 | God directory `domains/mail/` (~100+ файлов) | P3 |
| 2 | Dual naming Persons↔Personas (ADR‑0084 vs код) | P2, P7 |
| 3 | `SemanticSourceKind::Person` → `"contact"` (legacy) | P2 |
| 4 | Notes domain – нет backend | P4 |
| 5 | `CommunicationsPage.vue` – God Component (891 строка) | P5 |
| 6 | Отсутствуют WhatsApp и Organizations stores | P6 |
| 7 | Отсутствуют UI‑состояния (Loading/Empty/Error/Skeleton) в большинстве компонентов | P6 |
| 8 | Только 1 placeholder‑тест (нет реальных тестов) | P1, P10 |
| 9 | Telegram модуль – partial | P8 |
| 10 | Mail модуль – partial | P9 |
| 11 | Cross‑domain imports в review‑store (из personas, tasks, knowledge) | P5 |
| 12 | Смесь `raw fetch()` и TanStack Query в CommunicationsPage | P5 |

### Фазы и статус

| # | Фаза | Статус | Зависит от | Ключевой результат |
|---|------|--------|------------|-------------------|
| 1 | Foundation & Safety Net | pending | — | Testcontainers, characterization‑тесты, инвентаризация naming/UI‑states |
| 2 | Backend: Persona Naming | pending | P1 | `/api/v1/personas`, `Persona = Person`, `SemanticSourceKind::Person` → `"person"` |
| 3 | Backend: Communication Domain | pending | P2 | `domains/communications/` с core‑фасадом и mail‑каналом |
| 4 | Backend: Notes Domain | pending | P2 | `domains/notes/` с моделями, store, API `/api/v1/notes` |
| 5 | Frontend: God Component Refactoring | pending | P3 | CommunicationsPage < 500 строк, composables, устранение raw‑fetch и cross‑domain imports |
| 6 | Frontend: Missing Stores & States | pending | P5 | WhatsApp‑store, Organizations‑store, Loading/Empty/Error/Skeleton во всех компонентах |
| 7 | Frontend: Persona Alignment | pending | P2, P6 | Роуты, компоненты, stores, i18n переименованы в Personas |
| 8 | Telegram Module Parity | pending | P3 | Полный паритет с Telegram Desktop |
| 9 | Mail Module Parity | pending | P3 | Полный паритет с Outlook/Apple Mail |
| 10 | Polish & Harden | pending | все | Финальный аудит, тесты, безопасность, обновление IMPLEMENTATION_STATUS.md |

### Архитектурные ограничения (из ADR и master‑spec)

- **Event sourcing as spine** (ADR‑0001) – все изменения проходят через event log.
- **Knowledge graph first** (ADR‑0008) – ребра графа являются первичными кросс‑доменными связями.
- **Local‑first** – ни одна операция не требует сервера.
- **AI output never source of truth** – AI‑наблюдения всегда reviewable.
- **Persona model** (ADR‑0084) – Personas, не Contacts/CRM; `PersonaType`: `human`, `ai_agent`, `organization_proxy`, `system`; один Owner Persona с `is_self = true`.
- **Communication spine** (ADR‑0085) – Communication → Source Evidence → Extracted Knowledge → Memory → Context.
- **First‑class Relationships** (ADR‑0086) – Relationship‑записи с evidence, trust‑score, strength‑score.
- **Domain/Engine separation** – Domains владеют durable entities; Engines производят derived views.
- **Desktop‑first** (ADR‑0026, ADR‑0031) – mobile UI вне скоупа.
- **i18n RU/EN** (ADR‑0077) – English keys, Russian translations.
- **Frontend architecture** (ADR‑0093) – Vue 3 + Pinia + TanStack Query, Domain‑Driven структура.
- **Backend module organization** (ADR‑0073) – структура модулей backend.

### Ключевые риски (top‑3 из ROADMAP)

1. **Breaking API changes при rename Persons→Personas**
   *Mitigation:* `/api/v1/personas` как новый route, `/api/v1/persons` как redirect/compatibility.

2. **God component CommunicationsPage (891 строка) невозможно рефакторить без регрессий**
   *Mitigation:* декомпозиция пошагово с верификацией build pass на каждом шаге.

3. **Telegram/Mail parity – огромный объём**
   *Mitigation:* разбивка на must‑have / nice‑to‑have; baseline сохраняется.

### Связанные суперцели

- **Миграция фронтенда на Vue 3** (`.supergoal/makosh-frontend-migration-vue-3-WzENWm`)
  **Статус:** завершена (2026‑06‑14). 15 фаз выполнены. Фронтенд полностью на Vue 3, Pinia, TanStack Query.
  Эта работа обеспечила целевую архитектуру фронтенда, на которую опирается выравнивание документации.

### Примечание

Все детальные спецификации фаз, acceptance criteria и инвентаризационные отчёты находятся в рабочем каталоге `.supergoal/makosh-docs-alignment-Q2wXmK/`. Данная страница является операционной сводкой и не заменяет исходные планы.
```

## Покрытие источников

- **`.supergoal/makosh-docs-alignment-Q2wXmK/ROADMAP.md`** (первые 12000 символов, файл обрезан)
  Факты: название задачи, тип, дата создания, количество фаз (10), стек (Rust/Axum, Vue 3/Pinia/TanStack, Tauri 2), перечень 12 gaps, карта фаз и их зависимости, deliverable-описания фаз 1–… (фазы 1 и далее присутствуют в видимой части), топ-3 рисков, assumptions (совместимость API, депрекация `"contact"`, Notes как легковесный domain и т.д.). Использовано для таблицы фаз, списка gaps, рисков и архитектурного контекста.

- **`.supergoal/makosh-docs-alignment-Q2wXmK/STATE.md`** (1841 символов, не обрезан)
  Факты: статус `PLANNING`, текущая фаза 0, дата начала и последнего обновления, таблица статуса всех фаз (все `pending`), инженерные проверки (`pending`), notable events (создание плана, выявление 12 gaps). Использовано для заполнения статуса суперцели.

- **`.supergoal/makosh-docs-alignment-Q2wXmK/THINKING.md`** (первые 12000 символов, файл обрезан)
  Факты: цели (5 пунктов), архитектурные и продуктовые ограничения (ADR-0001, 0008, 0084, 0085, 0086, 0077, 0026, 0031, 0093, master‑spec), кодовые рамки (Rust/Axum, ~300+ файлов, 12 domains, 11 engines, 200+ эндпоинтов, 74 миграции), риски (7 пунктов), Memory Hits (миграция завершена, God‑component, cross‑domain imports и т.д.). Использовано для раздела «Архитектурные ограничения» и для подтверждения завершённости миграции.

- **Файлы фаз 1–10** (каждый вложен полностью или без обрезки в пределах лимита)
  Факты: structure‑спецификации каждой фазы, обязательные команды, acceptance criteria, зависимости. Использованы для заполнения таблицы фаз с ключевыми результатами.

- **`.supergoal/makosh-frontend-migration-vue-3-WzENWm/STATE.md`** (первые 12000 символов, файл обрезан)
  Факты: статус `COMPLETED`, текущая фаза 15, завершены все 15 фаз, build pass на каждой, notable events с перечислением результатов каждой фазы. Использовано для упоминания завершённой миграции в разделе «Связанные суперцели».

## Исходные файлы

- [`.supergoal/makosh-docs-alignment-Q2wXmK/ROADMAP.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/ROADMAP.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/STATE.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/STATE.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/THINKING.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/THINKING.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-1.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-1.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-10.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-10.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-2.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-2.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-3.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-3.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-4.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-4.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-5.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-5.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-6.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-6.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-7.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-7.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-8.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-8.md)
- [`.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-9.md`](../../../../.supergoal/makosh-docs-alignment-Q2wXmK/phases/phase-9.md)
- [`.supergoal/makosh-frontend-migration-vue-3-WzENWm/PROTOCOL.md`](../../../../.supergoal/makosh-frontend-migration-vue-3-WzENWm/PROTOCOL.md)
- [`.supergoal/makosh-frontend-migration-vue-3-WzENWm/ROADMAP.md`](../../../../.supergoal/makosh-frontend-migration-vue-3-WzENWm/ROADMAP.md)
- [`.supergoal/makosh-frontend-migration-vue-3-WzENWm/STATE.md`](../../../../.supergoal/makosh-frontend-migration-vue-3-WzENWm/STATE.md)
- [`.supergoal/makosh-frontend-migration-vue-3-WzENWm/THINKING.md`](../../../../.supergoal/makosh-frontend-migration-vue-3-WzENWm/THINKING.md)
- [`.supergoal/makosh-frontend-migration-vue-3-WzENWm/context.md`](../../../../.supergoal/makosh-frontend-migration-vue-3-WzENWm/context.md)
- [`.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-1.md`](../../../../.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-1.md)

## Кандидаты на drift

Из предоставленного контекста расхождения кода/документации/ADR не видны – все встроенные файлы являются плановыми артефактами самой суперцели `makosh-docs-alignment` и архивом завершённой миграции фронтенда. Они не содержат исходного кода продукта, файлов ADR или текущих wiki‑страниц, которые можно было бы сравнить с планом. Поэтому дрифт не может быть подтверждён или опровергнут.
