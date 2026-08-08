---
chunk_id: 119-doc-docs-part-010
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 19
generated_by: code-wiki-ru
---

# 119-doc-docs-part-010 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `19`

## Резюме

Добавить страницу `operations/documentation-map.md` на русском языке — навигационный обзор всех пакетов документации, зафиксированных в репозитории Макошь. Карта группирует документы по областям (roadmap, vision, workflows, UI, vault, site), указывает цели каждого пакета и связывает их внутренними ссылками Obsidian. Это устраняет отсутствие единой точки входа в документацию для русскоязычной wiki.

## Предложенные страницы

**`operations/documentation-map.md`**

```markdown
# Карта документации

Обзорный навигатор по документационным пакетам репозитория Макошь.

## Roadmap (дорожные карты)

Контрольные списки версий фиксируют цели, объём и состояние приёмки каждого крупного релиза.

- ``Графовое ядро V2` (`../roadmap/v2-graph-core-checklist`)` — цель: построить детерминированную, read-only проекцию Knowledge Graph из Persona-совместимых записей идентичности, сообщений и документов.
- ``Закрытие V2` (`../roadmap/v2-closure-checklist`)` — цель: сделать память на основе графа центральной, включая проекты, персоналии, документы и workflow-кандидатов.
- ``Закрытие V3` (`../roadmap/v3-closure-checklist`)` — цель: запустить локальные AI-рабочие процессы с Ollama, pgvector-семантическим поиском и цитируемыми ответами.
- ``Закрытие V4` (`../roadmap/v4-closure-checklist`)` — цель: предоставить Telegram-клиент с несколькими аккаунтами, аудитом автоматических отправок и транскрипцией звонков.
- ``Закрытие V5` (`../roadmap/v5-closure-checklist`)` — цель: подключить WhatsApp Web как локальный компаньон-источник с сохранением происхождения и приватности.

## Vision (видение продукта)

Пакет `docs/vision` хранит долгосрочное направление продукта. Канонические детали продукта находятся в `docs/product/` и `docs/foundation/` (не включены в текущий контекст).

- ``Документ видения` (`../vision/vision-document`)` — миссия, «Северная звезда», непродуктовые границы, долгосрочная ценность, принципы продукта и критерии успеха.

## Workflows (рабочие процессы)

Рабочие процессы описывают путь свидетельств через систему, а не конкретные API или реализацию. Они координируют домены и движки, но не владеют долговременными сущностями.

Центральный принцип:

```text
Коммуникация → Свидетельство → Знание → Память → Отношения → Контекст → Обязательства / Задачи / Решения / Проекты
```

Документы, встречи, звонки и заметки также могут создавать свидетельства, но коммуникации — самый частый вход.

Доступные спецификации:

- ``От коммуникации к знанию` (`../workflows/communication-to-knowledge`)`
- ``От коммуникации к обязательству` (`../workflows/communication-to-obligation`)`
- ``От встреч к решениям` (`../workflows/meeting-to-decisions`)`
- ``От документов к контексту` (`../workflows/document-to-context`)`
- ``Обзор противоречий` (`../workflows/contradiction-review`)` (пользовательское название: Polygraph review)
- ``Генерация досье` (`../workflows/dossier-generation`)`
- ``Ассистируемый агентом поиск` (`../workflows/agent-assisted-recall`)`

См. ``Каталог рабочих процессов` (`../workflows/README`)`.

## UI (пользовательский интерфейс)

Пакет `docs/ui` содержит направление продуктового интерфейса и ограничения дизайн-системы. Файлы реализации лежат в `frontend/`.

- ``Видение дизайн-системы` (`../ui/design-system-vision`)` — ощущение продукта (быстрый, спокойный, плотный, современный, объяснимый), источники вдохновения (Arc, Raycast, Linear, Notion, Obsidian), ключевые принципы, состав ядра компонентов, визуальное и интерактивное направления.

См. ``Обзор пакета UI` (`../ui/README`)`.

## Vault (хранилище секретов)

Документация Vault зеркалит слой `backend/src/vault`, который отвечает за жизненный цикл локального host-хранилища, границы хранения секретных данных, обращение с ключевым материалом и поддержку восстановления. PostgreSQL хранит только метаданные и ссылки на секреты; новые учётные данные провайдеров не должны находиться в таблицах БД.

См. ``Обзор Vault` (`../vault/README`)`.

## Site (сайт документации)

Пакет `docs/site` содержит статический портал документации на GitHub Pages и сопутствующие ресурсы. Канонический контент остаётся в Markdown-пакетах.

См. ``Описание пакета site` (`../site/README`)`.

## Неподтверждённые области

В предоставленном контексте отсутствуют пакеты `docs/product/`, `docs/foundation/`, а также возможные ADR-документы, упомянутые в контрольных списках (ADR-0049, ADR-0050, ADR-0051, ADR-0089). Их наполнение и точное расположение не подтверждены.
```

## Покрытие источников

- `docs/roadmap/v2-closure-checklist.md` — цель релиза V2, объём in/out of scope, статус приёмки.
- `docs/roadmap/v2-graph-core-checklist.md` — цель графового ядра V2, объём и статус.
- `docs/roadmap/v3-closure-checklist.md` — цель V3, объём и статус.
- `docs/roadmap/v4-closure-checklist.md` — цель V4, объём и статус.
- `docs/roadmap/v5-closure-checklist.md` — цель V5, объём и статус.
- `docs/site/README.md` — назначение пакета `docs/site`: статический портал GitHub Pages, каноничный контент в Markdown.
- `docs/ui/README.md` — назначение пакета `docs/ui`: описание направления интерфейса и ограничений дизайн-системы, расположение реализации в `frontend/`.
- `docs/ui/design-system-vision.md` — ощущение продукта, источники вдохновения, принципы, компоненты, визуальное и интерактивное направления.
- `docs/vault/README.md` — назначение пакета, зеркало `backend/src/vault`, границы хранения секретов, ключевой материал, восстановление, правила документирования.
- `docs/vision/README.md` — назначение пакета `docs/vision`: сохранение долгосрочного направления, каноничные детали в `product/` и `foundation/`.
- `docs/vision/vision-document.md` — миссия, «Северная звезда», границы, ценность, принципы, критерии успеха.
- `docs/workflows/README.md` — центральный принцип (коммуникация → свидетельство → …), таблица спецификаций, граничное правило.
- `docs/workflows/agent-assisted-recall.md` — существование workflow, триггер, поток, границы доменов, текущая реализация, план миграции.
- `docs/workflows/communication-to-knowledge.md` — существование workflow, триггер, поток, границы доменов, текущая реализация, план миграции.
- `docs/workflows/communication-to-obligation.md` — существование workflow, триггер, поток, границы доменов, текущая реализация, план миграции.
- `docs/workflows/contradiction-review.md` — существование workflow, триггер, поток, границы доменов, псевдоним Polygraph, текущая реализация, план миграции.
- `docs/workflows/document-to-context.md` — существование workflow, триггер, поток, границы доменов, текущая реализация, план миграции.
- `docs/workflows/dossier-generation.md` — существование workflow, триггер, поток, границы доменов, текущая реализация, план миграции.
- `docs/workflows/meeting-to-decisions.md` — существование workflow, триггер, поток, границы доменов, текущая реализация, план миграции.

## Исходные файлы

- [`docs/roadmap/v2-closure-checklist.md`](../../../roadmap/v2-closure-checklist.md)
- [`docs/roadmap/v2-graph-core-checklist.md`](../../../roadmap/v2-graph-core-checklist.md)
- [`docs/roadmap/v3-closure-checklist.md`](../../../roadmap/v3-closure-checklist.md)
- [`docs/roadmap/v4-closure-checklist.md`](../../../roadmap/v4-closure-checklist.md)
- [`docs/roadmap/v5-closure-checklist.md`](../../../roadmap/v5-closure-checklist.md)
- [`docs/site/README.md`](../../../site/README.md)
- [`docs/ui/README.md`](../../../ui/README.md)
- [`docs/ui/design-system-vision.md`](../../../ui/design-system-vision.md)
- [`docs/vault/README.md`](../../../vault/README.md)
- [`docs/vision/README.md`](../../../vision/README.md)
- [`docs/vision/vision-document.md`](../../../vision/vision-document.md)
- [`docs/workflows/README.md`](../../../workflows/README.md)
- [`docs/workflows/agent-assisted-recall.md`](../../../workflows/agent-assisted-recall.md)
- [`docs/workflows/communication-to-knowledge.md`](../../../workflows/communication-to-knowledge.md)
- [`docs/workflows/communication-to-obligation.md`](../../../workflows/communication-to-obligation.md)
- [`docs/workflows/contradiction-review.md`](../../../workflows/contradiction-review.md)
- [`docs/workflows/document-to-context.md`](../../../workflows/document-to-context.md)
- [`docs/workflows/dossier-generation.md`](../../../workflows/dossier-generation.md)
- [`docs/workflows/meeting-to-decisions.md`](../../../workflows/meeting-to-decisions.md)

## Кандидаты на drift

- Контрольные списки V3, V4, V5 ссылаются на архитектурные решения ADR-0049, ADR-0050, ADR-0051; workflow-спецификации `meeting-to-decisions.md` упоминает ADR-0089. Эти ADR-файлы не включены в контекстный пакет — их расположение и содержание не подтверждены.
- `docs/vision/README.md` и `docs/vision/vision-document.md` указывают, что каноничные детали продукта находятся в `docs/product/` и `docs/foundation/`. Эти каталоги не встроены в контекст; их фактическое содержимое не проверено.
- Ряд рабочих процессов содержит разделы «Current Implementation Evidence», ссылающиеся на конкретные модули (`backend/src/domains/…`, `backend/src/engines/…`). Без предоставленных исходных файлов кода невозможно верифицировать актуальность этих утверждений — между задокументированным состоянием и реальной кодовой базой может существовать расхождение.
