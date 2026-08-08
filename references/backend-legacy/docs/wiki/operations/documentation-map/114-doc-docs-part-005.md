---
chunk_id: 114-doc-docs-part-005
batch_id: batch-20260628T214902
group: docs
role: doc
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 114-doc-docs-part-005 — docs/doc

- Target index: [[operations/documentation-map]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создать в русской Obsidian‑wiki страницу `operations/documentation-map.md`.
Страница описывает структуру документации Макошь: разделы Foundation, Engines, Интеграции и ссылки на ADR, чтобы служить точкой входа для навигации.
Всё содержание опирается только на встроенные в задачу исходные файлы.

## Предложенные страницы

- `operations/documentation-map.md`

```markdown
# Карта документации Макошь

Эта страница описывает организацию и назначение документации Макошь и помогает быстро найти нужный раздел.

## Foundation (Основа)

Документы Foundation определяют **канонический словарь**, **модель продукта** и **долгосрочные архитектурные ограничения**.
Если нижележащие документы противоречат Foundation, предпочтение отдаётся Foundation, если только более новый ADR не отменяет его.

- ``foundation/vision`` — видение продукта, его identity и north star.
- ``foundation/glossary`` — канонический глоссарий терминов.
- ``foundation/world-model`` — модель мира: первичные сущности, производные объекты, модель отношений.
- ``foundation/architecture-principles`` — 12 архитектурных принципов.
- ``foundation/domain-map`` — карта доменов и движков, меж‑доменные правила.
- ``foundation/engines`` — обзорная карта движков и их назначения.

## Engines (Движки)

Движки — переиспользуемые механизмы, работающие поверх доменов. Они не владеют первичными сущностями доменов.

Полный список движков приведён в ``foundation/engines``. Подробные спецификации находятся в каталоге `engines/`.
В текущем контексте представлены детальные документы только для трёх движков:

- ``engines/timeline`` — Timeline Engine: строит хронологические представления из событий и доменных записей.
- ``engines/trust`` — Trust Engine: вычисляет сигналы доверия к отношениям и источникам.
- ``engines/speaker-identity`` — Speaker Diarization and Identity: задача слияния (merge), а не поиска по одному источнику.

Остальные движки описаны в общей карте в ``foundation/engines``, но их детальные спецификации не входят в этот пакет контекста.

## Интеграции

Интеграции — это адаптеры провайдеров и протоколов. Они наблюдают за внешними системами, сохраняют источник происхождения (provenance) и генерируют события или свидетельства для доменов.
Интеграция **не является** доменом продукта Макошь.

Каталог интеграций: ``integrations/README``.

### Поставщики (Providers)

| Провайдер | Пакет документации |
|---|---|
| Mail (почта) | ``integrations/mail/README`` |
| Telegram | ``integrations/telegram/README`` |
| WhatsApp | ``integrations/whatsapp/README`` |
| Zoom | ``integrations/zoom/README`` |
| Yandex Telemost | ``integrations/yandex-telemost/README`` |
| Ollama (локальный AI) | ``integrations/ollama/README`` |
| OmniRoute (опциональный AI‑runtime) | ``integrations/omniroute/README`` |

Для каждого провайдера могут существовать дополнительные страницы: архитектура, API, модульная карта, статус, анализ пробелов, блокеры.
Эталонной формой пакета документации считается Zoom; создавать пустые файлы‑заглушки не нужно.

### Каналы и домены

Инвариант: **канал никогда не является доменом**. Канал — это интеграция. Коммуникация (Communication) — доменный объект.
Например, Email и Telegram являются коммуникационными каналами, а не отдельными продуктами.

## ADR (Architecture Decision Records)

Решения по архитектуре фиксируются в ADR. Многие страницы документации ссылаются на конкретные ADR.
Подробная карта ADR не входит в этот пакет контекста, но примеры упоминаемых ADR:

- ADR-0001 (Event sourcing as system spine)
- ADR-0009 (Local AI through Ollama)
- ADR-0041–0055 (email‑provider ingestion, секреты, хранилище)
- ADR-0081–0082 (OmniRoute, AI Settings Control Center)

## Как использовать эту карту

- Чтобы понять **общий язык и модель**, начните с ``foundation/glossary`` и ``foundation/world-model``.
- Чтобы понять **границы ответственности**, изучите ``foundation/domain-map`` и ``foundation/engines``.
- Для деталей **конкретного движка** перейдите в `engines/`.
- Для работы с **внешними сервисами** — в каталог интеграций и соответствующий пакет провайдера.
```

## Покрытие источников

- `docs/foundation/README.md` — назначение Foundation: «Foundation documents define canonical vocabulary, product model and long-term architecture constraints.»
- `docs/foundation/vision.md` — identity продукта: «Макошь is a local-first Personal Memory System.»
- `docs/foundation/glossary.md` — канонический глоссарий терминов.
- `docs/foundation/world-model.md` — модель мира: первичные сущности, производные объекты, модель отношений.
- `docs/foundation/architecture-principles.md` — 12 архитектурных принципов.
- `docs/foundation/domain-map.md` — карта доменов, движков и меж‑доменные правила.
- `docs/foundation/engines.md` — полный список движков и их назначения.
- `docs/engines/timeline/README.md` — детали Timeline Engine.
- `docs/engines/trust/README.md` — детали Trust Engine.
- `docs/engines/speaker-identity/README.md` — Speaker Diarization and Identity.
- `docs/integrations/README.md` — каталог интеграций, форма пакета, граничные правила, таблица провайдеров.
- `docs/integrations/mail/README.md` — Email‑канал, принципы, lifecycle, навигация.
- `docs/integrations/mail/api.md` (truncated) — API‑справочник почтового канала.
- `docs/integrations/mail/architecture.md` — архитектура Email‑канала, слои, ADR.
- `docs/integrations/mail/blockers.md` — архитектурные блокеры почтового канала.
- `docs/integrations/mail/gap-analysis.md` (truncated) — анализ покрытия.
- `docs/integrations/mail/modules.md` — карта модулей.
- `docs/integrations/mail/status.md` (truncated) — статус реализации.
- `docs/integrations/ollama/README.md` — интеграция Ollama, граничное правило.
- `docs/integrations/omniroute/README.md` — интеграция OmniRoute, граничное правило.
- `docs/integrations/telegram/README.md` — Telegram‑канал, завершённая граница, отложенные инициативы.
- `docs/integrations/telegram/api.md` — обзор API Telegram, scope notes.
- `docs/integrations/telegram/api/README.md` — навигация по деталям API Telegram.
- `docs/integrations/telegram/api/conversations.md` — маршруты бесед и сообщений Telegram.
- `docs/integrations/telegram/api/foundation.md` — маршруты foundation и capability‑контракта.

## Исходные файлы

- [`docs/engines/speaker-identity/README.md`](../../../engines/speaker-identity/README.md)
- [`docs/engines/timeline/README.md`](../../../engines/timeline/README.md)
- [`docs/engines/trust/README.md`](../../../engines/trust/README.md)
- [`docs/foundation/README.md`](../../../foundation/README.md)
- [`docs/foundation/architecture-principles.md`](../../../foundation/architecture-principles.md)
- [`docs/foundation/domain-map.md`](../../../foundation/domain-map.md)
- [`docs/foundation/engines.md`](../../../foundation/engines.md)
- [`docs/foundation/glossary.md`](../../../foundation/glossary.md)
- [`docs/foundation/vision.md`](../../../foundation/vision.md)
- [`docs/foundation/world-model.md`](../../../foundation/world-model.md)
- [`docs/integrations/README.md`](../../../integrations/README.md)
- [`docs/integrations/mail/README.md`](../../../integrations/mail/README.md)
- [`docs/integrations/mail/api.md`](../../../integrations/mail/api.md)
- [`docs/integrations/mail/architecture.md`](../../../integrations/mail/architecture.md)
- [`docs/integrations/mail/blockers.md`](../../../integrations/mail/blockers.md)
- [`docs/integrations/mail/gap-analysis.md`](../../../integrations/mail/gap-analysis.md)
- [`docs/integrations/mail/modules.md`](../../../integrations/mail/modules.md)
- [`docs/integrations/mail/status.md`](../../../integrations/mail/status.md)
- [`docs/integrations/ollama/README.md`](../../../integrations/ollama/README.md)
- [`docs/integrations/omniroute/README.md`](../../../integrations/omniroute/README.md)
- [`docs/integrations/telegram/README.md`](../../../integrations/telegram/README.md)
- [`docs/integrations/telegram/api.md`](../../../integrations/telegram/api.md)
- [`docs/integrations/telegram/api/README.md`](../../../integrations/telegram/api/README.md)
- [`docs/integrations/telegram/api/conversations.md`](../../../integrations/telegram/api/conversations.md)
- [`docs/integrations/telegram/api/foundation.md`](../../../integrations/telegram/api/foundation.md)

## Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно.
Все встроенные документы Foundation и Engines последовательно утверждают одни и те же инварианты (например, «движок не является доменом», «канал не является доменом»).
Часть файлов обрезана из‑за ограничения в 12 000 символов, но это не свидетельствует о drift, а только о неполноте видимых данных.
